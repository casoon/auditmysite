//! Display modes (#653): the BarrierLab `data-display` convention.
//!
//! A site following the convention (draft v0, `barrierlab/plan/03`) sets
//! `<html data-display="visual|calm|text">` before the first paint, stores the
//! visitor's choice in `localStorage` under the key `display`, and offers a
//! `[data-display-toggle]` control on every page with a visualisation
//! (`figure[data-viz]`).
//!
//! This module owns the live, CDP-side of that convention — it stays in
//! auditmysite (CDP capture, see CLAUDE.md "Grenze zu barrierlab"):
//!
//! - [`prepare_page`] runs before navigation: it presents the audit browser as
//!   a visitor who picked a mode (`localStorage.display` via
//!   `Page.addScriptToEvaluateOnNewDocument`) and, for `calm`/`text`, emulates
//!   `prefers-reduced-motion: reduce` so sites without the convention that
//!   honour the media query also render their reduced variant. It also
//!   installs an observer that records whether `data-display` was already set
//!   when `<body>` was parsed (input for `display/init-missing`).
//! - [`detect`] reads the rendered page after load and reports whether the
//!   page offers display modes.
//!
//! The `display/*` convention checks themselves live in
//! `wcag::rules::display_modes`.

use chromiumoxide::cdp::browser_protocol::emulation::{MediaFeature, SetEmulatedMediaParams};
use chromiumoxide::cdp::browser_protocol::page::AddScriptToEvaluateOnNewDocumentParams;
use chromiumoxide::Page;
use serde::{Deserialize, Serialize};
use tracing::warn;

/// One of the convention's three display modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DisplayMode {
    /// Everything, including 3D scenes and animation.
    Visual,
    /// No animation; static pictures instead of 3D.
    Calm,
    /// No visualisations; their statement and values as text.
    Text,
}

impl DisplayMode {
    /// All modes in the convention's order.
    pub const ALL: [DisplayMode; 3] = [DisplayMode::Visual, DisplayMode::Calm, DisplayMode::Text];

    /// The value the convention uses for `data-display` and `localStorage.display`.
    pub fn as_str(self) -> &'static str {
        match self {
            DisplayMode::Visual => "visual",
            DisplayMode::Calm => "calm",
            DisplayMode::Text => "text",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "visual" => Some(DisplayMode::Visual),
            "calm" => Some(DisplayMode::Calm),
            "text" => Some(DisplayMode::Text),
            _ => None,
        }
    }

    /// `calm` and `text` stand for a visitor who reduced motion.
    pub fn reduces_motion(self) -> bool {
        matches!(self, DisplayMode::Calm | DisplayMode::Text)
    }
}

/// Which display mode a report's scores belong to. `SiteDefault` is what
/// every audit before #653 measured: whatever the site renders for a fresh
/// visitor, without a stored choice or an emulated preference.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditedDisplayMode {
    #[default]
    SiteDefault,
    Visual,
    Calm,
    Text,
}

impl From<Option<DisplayMode>> for AuditedDisplayMode {
    fn from(mode: Option<DisplayMode>) -> Self {
        match mode {
            None => AuditedDisplayMode::SiteDefault,
            Some(DisplayMode::Visual) => AuditedDisplayMode::Visual,
            Some(DisplayMode::Calm) => AuditedDisplayMode::Calm,
            Some(DisplayMode::Text) => AuditedDisplayMode::Text,
        }
    }
}

/// Per-page detection of the convention. Canonical English identifiers only
/// (#406) — the PDF derives its wording from these fields at render time.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DisplayModesInfo {
    /// `html[data-display]` carries one of the convention's values.
    pub offers_display_modes: bool,
    /// The modes the page offers: the values its toggle declares, else the
    /// convention's three when `html[data-display]` is set.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub offered_modes: Vec<DisplayMode>,
    /// Value of `html[data-display]` when the page was audited.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_mode: Option<String>,
    /// Whether `data-display` was already set when `<body>` was parsed.
    /// `None` when the pre-navigation observer did not run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub set_before_body: Option<bool>,
    /// A `[data-display-toggle]` element exists.
    pub toggle_present: bool,
    /// `figure[data-viz]` count per `data-viz` value.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub visualisations: Vec<VisualisationCount>,
    /// `matchMedia('(prefers-reduced-motion: reduce)')` matched during the audit.
    pub reduced_motion: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VisualisationCount {
    /// `data-viz` value (`chart`, `diagram`, `3d`, `image`, `interactive`, or
    /// whatever the page declared).
    pub kind: String,
    pub count: u32,
}

impl DisplayModesInfo {
    /// Pages that only `visual` can fully audit: 3D scenes and interactive
    /// visualisations exist only there.
    pub fn has_visual_only_content(&self) -> bool {
        self.visualisations
            .iter()
            .any(|v| matches!(v.kind.as_str(), "3d" | "interactive") && v.count > 0)
    }
}

/// Media features for the chosen mode. Every `Emulation.setEmulatedMedia`
/// call replaces the whole feature list, so code that emulates other media
/// features on an audited page (dark mode, forced colours, print) must carry
/// these along or it silently drops the mode's reduced motion.
pub fn media_features(mode: Option<DisplayMode>) -> Vec<MediaFeature> {
    match mode {
        Some(m) if m.reduces_motion() => vec![MediaFeature {
            name: "prefers-reduced-motion".to_string(),
            value: "reduce".to_string(),
        }],
        _ => Vec::new(),
    }
}

/// Records `html[data-display]` at the moment `<body>` is inserted. The
/// parser performs a microtask checkpoint before it runs the next script, so
/// the observer fires before any later script could set the attribute.
const BODY_OBSERVER_JS: &str = r#"(function () {
  if (window.__amsDisplayObserverInstalled) return;
  window.__amsDisplayObserverInstalled = true;
  function record() {
    window.__amsDisplayObserved = true;
    window.__amsDisplayAtBody = document.documentElement
      ? document.documentElement.getAttribute('data-display')
      : null;
  }
  if (document.body) { record(); return; }
  try {
    var obs = new MutationObserver(function () {
      if (document.body) { record(); obs.disconnect(); }
    });
    obs.observe(document, { childList: true, subtree: true });
  } catch (e) {}
})();"#;

fn stored_choice_js(mode: DisplayMode) -> String {
    format!(
        "try {{ window.localStorage.setItem('display', '{}'); }} catch (e) {{}}",
        mode.as_str()
    )
}

/// Before navigation: install the body observer and, for a chosen mode, the
/// stored choice and the reduced-motion preference. Best effort — a failure
/// is logged and the audit runs on in the site default, which the report
/// then shows through [`DisplayModesInfo::active_mode`].
pub async fn prepare_page(page: &Page, mode: Option<DisplayMode>) {
    if let Err(e) = page
        .execute(AddScriptToEvaluateOnNewDocumentParams::new(
            BODY_OBSERVER_JS,
        ))
        .await
    {
        warn!("Display-mode observer injection failed: {e}");
    }
    let Some(mode) = mode else {
        return;
    };
    if let Err(e) = page
        .execute(AddScriptToEvaluateOnNewDocumentParams::new(
            stored_choice_js(mode),
        ))
        .await
    {
        warn!("Display-mode choice injection failed: {e}");
    }
    let features = media_features(Some(mode));
    if !features.is_empty() {
        if let Err(e) = page
            .execute(SetEmulatedMediaParams::builder().features(features).build())
            .await
        {
            warn!("Reduced-motion emulation failed: {e}");
        }
    }
}

const DETECT_JS: &str = r#"(function () {
  var KNOWN = ['visual', 'calm', 'text'];
  var html = document.documentElement;
  var kinds = {};
  var figs = document.querySelectorAll('figure[data-viz]');
  for (var i = 0; i < figs.length; i++) {
    var k = (figs[i].getAttribute('data-viz') || '').trim().toLowerCase() || 'unspecified';
    kinds[k] = (kinds[k] || 0) + 1;
  }
  var toggles = document.querySelectorAll('[data-display-toggle]');
  var offered = [];
  for (var t = 0; t < toggles.length; t++) {
    var cands = [toggles[t]].concat(Array.prototype.slice.call(
      toggles[t].querySelectorAll('option, [value], [data-display-value], [data-display]')));
    for (var c = 0; c < cands.length; c++) {
      var vals = [cands[c].getAttribute('value'), cands[c].getAttribute('data-display-value'),
                  cands[c].getAttribute('data-display')];
      for (var v = 0; v < vals.length; v++) {
        var val = (vals[v] || '').trim().toLowerCase();
        if (KNOWN.indexOf(val) >= 0 && offered.indexOf(val) < 0) offered.push(val);
      }
    }
  }
  return JSON.stringify({
    active: html ? html.getAttribute('data-display') : null,
    toggle: toggles.length > 0,
    offered: offered,
    kinds: kinds,
    observed: window.__amsDisplayObserved === true,
    atBody: window.__amsDisplayAtBody === undefined ? null : window.__amsDisplayAtBody,
    reducedMotion: window.matchMedia('(prefers-reduced-motion: reduce)').matches
  });
})()"#;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawDetection {
    active: Option<String>,
    toggle: bool,
    #[serde(default)]
    offered: Vec<String>,
    #[serde(default)]
    kinds: std::collections::BTreeMap<String, u32>,
    observed: bool,
    at_body: Option<String>,
    reduced_motion: bool,
}

/// Read the convention off the rendered page. `None` when the page neither
/// sets `data-display` nor contains a `figure[data-viz]` — the convention
/// does not apply, and the report stays unchanged for such pages.
pub async fn detect(page: &Page) -> Option<DisplayModesInfo> {
    let raw = match page.evaluate(DETECT_JS).await {
        Ok(result) => result.into_value::<String>().ok()?,
        Err(e) => {
            warn!("Display-mode detection failed: {e}");
            return None;
        }
    };
    let raw: RawDetection = serde_json::from_str(&raw).ok()?;
    info_from_raw(raw)
}

fn info_from_raw(raw: RawDetection) -> Option<DisplayModesInfo> {
    let active_known = raw.active.as_deref().and_then(DisplayMode::parse);
    if raw.active.is_none() && raw.kinds.is_empty() && !raw.toggle {
        return None;
    }
    let mut offered: Vec<DisplayMode> = raw
        .offered
        .iter()
        .filter_map(|v| DisplayMode::parse(v))
        .collect();
    if offered.is_empty() && active_known.is_some() {
        offered = DisplayMode::ALL.to_vec();
    }
    offered.sort();
    offered.dedup();
    Some(DisplayModesInfo {
        offers_display_modes: active_known.is_some(),
        offered_modes: offered,
        active_mode: raw.active,
        set_before_body: raw.observed.then_some(raw.at_body.is_some()),
        toggle_present: raw.toggle,
        visualisations: raw
            .kinds
            .into_iter()
            .map(|(kind, count)| VisualisationCount { kind, count })
            .collect(),
        reduced_motion: raw.reduced_motion,
    })
}

/// Report text for the mode a score belongs to. Single text source for the
/// PDF (#406); `en` selects the language.
pub fn audited_mode_text(mode: AuditedDisplayMode, en: bool) -> String {
    match (mode, en) {
        (AuditedDisplayMode::SiteDefault, true) => "Site default (no mode chosen)".to_string(),
        (AuditedDisplayMode::SiteDefault, false) => {
            "Voreinstellung der Seite (kein Modus gewählt)".to_string()
        }
        (m, true) => format!(
            "{} (chosen via --display{})",
            audited_mode_value(m),
            if matches!(m, AuditedDisplayMode::Calm | AuditedDisplayMode::Text) {
                ", reduced motion emulated"
            } else {
                ""
            }
        ),
        (m, false) => format!(
            "{} (gewählt per --display{})",
            audited_mode_value(m),
            if matches!(m, AuditedDisplayMode::Calm | AuditedDisplayMode::Text) {
                ", reduzierte Bewegung emuliert"
            } else {
                ""
            }
        ),
    }
}

fn audited_mode_value(mode: AuditedDisplayMode) -> &'static str {
    match mode {
        AuditedDisplayMode::SiteDefault => "site_default",
        AuditedDisplayMode::Visual => "visual",
        AuditedDisplayMode::Calm => "calm",
        AuditedDisplayMode::Text => "text",
    }
}

/// "Offers display modes: visual · calm · text" — or why not. Single text
/// source for the PDF (#406).
pub fn offered_modes_text(info: Option<&DisplayModesInfo>, en: bool) -> String {
    let Some(info) = info.filter(|i| i.offers_display_modes) else {
        return if en {
            "None (no html[data-display])".to_string()
        } else {
            "Keine (kein html[data-display])".to_string()
        };
    };
    let modes = info
        .offered_modes
        .iter()
        .map(|m| m.as_str())
        .collect::<Vec<_>>()
        .join(" · ");
    let active = info.active_mode.as_deref().unwrap_or("-");
    if en {
        format!("{modes} (rendered: {active})")
    } else {
        format!("{modes} (dargestellt: {active})")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn raw(active: Option<&str>, kinds: &[(&str, u32)], toggle: bool) -> RawDetection {
        RawDetection {
            active: active.map(str::to_string),
            toggle,
            offered: Vec::new(),
            kinds: kinds.iter().map(|(k, c)| (k.to_string(), *c)).collect(),
            observed: true,
            at_body: active.map(str::to_string),
            reduced_motion: false,
        }
    }

    #[test]
    fn page_without_convention_yields_no_info() {
        assert_eq!(info_from_raw(raw(None, &[], false)), None);
    }

    #[test]
    fn convention_page_offers_the_three_modes_by_default() {
        let info = info_from_raw(raw(Some("calm"), &[("chart", 2)], true)).unwrap();
        assert!(info.offers_display_modes);
        assert_eq!(info.offered_modes, DisplayMode::ALL.to_vec());
        assert_eq!(info.set_before_body, Some(true));
        assert!(!info.has_visual_only_content());
    }

    #[test]
    fn toggle_declared_values_win_over_the_default_three() {
        let mut r = raw(Some("text"), &[], true);
        r.offered = vec!["text".into(), "calm".into(), "bogus".into()];
        let info = info_from_raw(r).unwrap();
        assert_eq!(
            info.offered_modes,
            vec![DisplayMode::Calm, DisplayMode::Text]
        );
    }

    #[test]
    fn unknown_data_display_value_does_not_count_as_offering_modes() {
        let info = info_from_raw(raw(Some("fancy"), &[], false)).unwrap();
        assert!(!info.offers_display_modes);
        assert!(info.offered_modes.is_empty());
    }

    #[test]
    fn unobserved_body_leaves_set_before_body_unknown() {
        let mut r = raw(Some("calm"), &[], true);
        r.observed = false;
        assert_eq!(info_from_raw(r).unwrap().set_before_body, None);
    }

    #[test]
    fn three_d_and_interactive_need_the_visual_pass() {
        let info = info_from_raw(raw(Some("calm"), &[("3d", 1)], true)).unwrap();
        assert!(info.has_visual_only_content());
        let info = info_from_raw(raw(Some("calm"), &[("interactive", 1)], true)).unwrap();
        assert!(info.has_visual_only_content());
    }

    #[test]
    fn reduced_motion_only_for_calm_and_text() {
        assert!(media_features(None).is_empty());
        assert!(media_features(Some(DisplayMode::Visual)).is_empty());
        for mode in [DisplayMode::Calm, DisplayMode::Text] {
            let features = media_features(Some(mode));
            assert_eq!(features.len(), 1);
            assert_eq!(features[0].name, "prefers-reduced-motion");
            assert_eq!(features[0].value, "reduce");
        }
    }

    #[test]
    fn stored_choice_script_sets_the_convention_key() {
        assert!(stored_choice_js(DisplayMode::Text).contains("setItem('display', 'text')"));
    }

    #[test]
    fn audited_mode_serializes_snake_case_and_defaults_to_site_default() {
        assert_eq!(
            serde_json::to_string(&AuditedDisplayMode::default()).unwrap(),
            "\"site_default\""
        );
        assert_eq!(
            AuditedDisplayMode::from(Some(DisplayMode::Calm)),
            AuditedDisplayMode::Calm
        );
    }

    #[test]
    fn english_texts_carry_no_german() {
        let info = info_from_raw(raw(Some("calm"), &[("chart", 1)], true)).unwrap();
        for text in [
            audited_mode_text(AuditedDisplayMode::SiteDefault, true),
            audited_mode_text(AuditedDisplayMode::Text, true),
            offered_modes_text(Some(&info), true),
            offered_modes_text(None, true),
        ] {
            assert!(
                !text.chars().any(|c| "äöüÄÖÜß".contains(c)),
                "German in English text: {text}"
            );
        }
        assert_eq!(
            offered_modes_text(Some(&info), true),
            "visual · calm · text (rendered: calm)"
        );
    }
}
