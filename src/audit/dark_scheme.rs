//! The contrast rules under `prefers-color-scheme: dark` (#917).
//!
//! A page that declares a dark colour scheme is rendered a second time under
//! dark emulation and the shared `contrast/*` rules run over that render. The
//! findings are the same rules with the same ids; they differ only in the
//! `computed` evidence attribute `color_scheme: dark`. Nothing here judges
//! the dark scheme as a design choice — whether the page offers one is not an
//! accessibility result, whether its text is readable in it is.
//!
//! The rules themselves live in `a11y-rules` (barrierlab); this module only
//! orchestrates the emulated pass and feeds them the dark computed styles.

use chromiumoxide::cdp::browser_protocol::emulation::{MediaFeature, SetEmulatedMediaParams};
use chromiumoxide::Page;
use tracing::warn;

use crate::accessibility::{fetch_dom_capture, AXTree};
use crate::audit::collection::{evaluate_js_value, Collected};
use crate::audit::exclusion::ExclusionScope;
use crate::audit::PipelineConfig;
use crate::cli::WcagLevel;
use crate::interaction::stability::wait_for_finite_animations;
use crate::wcag::{self, Violation, WcagResults};

/// Reason on the contrast outcomes when the page declares a dark scheme but
/// the dark pass could not be measured.
pub(crate) const DARK_PASS_FAILED: &str = "dark_scheme_not_measured";

/// Whether the page declares a dark colour scheme: an
/// `@media (prefers-color-scheme: dark)` rule (also nested, also on a
/// `<link media>`), `<meta name="color-scheme">` naming `dark`, or a computed
/// root `color-scheme` that does.
const DECLARES_DARK_JS: &str = r#"
(() => {
  const darkQuery = t => /prefers-color-scheme\s*:\s*dark/.test(t || '');
  const walk = rules => {
    for (const r of rules) {
      if (r instanceof CSSMediaRule && darkQuery(r.conditionText || (r.media && r.media.mediaText))) return true;
      if (r.cssRules && walk(r.cssRules)) return true;
    }
    return false;
  };
  for (const sheet of document.styleSheets) {
    if (darkQuery(sheet.media && sheet.media.mediaText)) return true;
    let rules;
    try { rules = sheet.cssRules; } catch (_) { continue; }
    if (rules && walk(rules)) return true;
  }
  const names = v => /(^|\s)dark(\s|$)/.test(v || '');
  const meta = document.querySelector('meta[name="color-scheme"]');
  if (meta && names(meta.getAttribute('content'))) return true;
  return names(getComputedStyle(document.documentElement).colorScheme);
})()
"#;

/// Identity of a finding for the comparison with the light render: the same
/// element, the same rule and the same message (which carries the measured
/// colours and ratio). A dark finding equal to a light one means the
/// emulation changed nothing for that element — a page that is dark by
/// default, say — and is not a second finding.
fn identity(v: &Violation) -> (String, String, String) {
    let element = match v.backend_node_id {
        Some(id) => format!("backend:{id}"),
        None => v.selector.clone().unwrap_or_else(|| v.node_id.clone()),
    };
    (
        v.rule_id.clone().unwrap_or_default(),
        element,
        v.message.clone(),
    )
}

fn is_contrast(v: &Violation) -> bool {
    v.rule_id
        .as_deref()
        .is_some_and(|id| id.starts_with("contrast/"))
}

/// Confirmed contrast failures of the dark render that the light render does
/// not already report, marked with the dark colour-scheme evidence.
fn new_dark_failures(dark: WcagResults, light: &[Violation]) -> Vec<Violation> {
    let known: std::collections::HashSet<_> = light
        .iter()
        .filter(|v| is_contrast(v))
        .map(identity)
        .collect();
    dark.violations
        .into_iter()
        .filter(|v| is_contrast(v) && !known.contains(&identity(v)))
        .map(Violation::in_dark_color_scheme)
        .collect()
}

async fn emulate_scheme(
    page: &Page,
    base: &[MediaFeature],
    dark: bool,
) -> std::result::Result<(), chromiumoxide::error::CdpError> {
    let extra = dark.then(|| MediaFeature {
        name: "prefers-color-scheme".to_string(),
        value: "dark".to_string(),
    });
    page.execute(
        SetEmulatedMediaParams::builder()
            .media("")
            .features(base.iter().cloned().chain(extra).collect::<Vec<_>>())
            .build(),
    )
    .await
    .map(|_| ())
}

fn mark_not_measured(results: &mut WcagResults, viewport: &str) {
    for outcome in results
        .rule_outcomes
        .iter_mut()
        .filter(|o| o.viewport.as_deref() == Some(viewport) && o.rule_id.starts_with("contrast/"))
    {
        if outcome.reason.is_none() {
            outcome.reason = Some(DARK_PASS_FAILED.to_string());
        }
    }
}

/// Runs the contrast rules a second time under dark emulation when the page
/// declares a dark scheme and adds what they find on top of the light run:
/// into `results` with the outcome counts raised accordingly, and — for
/// findings inside an excluded subtree — into `excluded`.
///
/// Only the top document is measured (iframes are not), and only confirmed
/// failures are carried over: an undeterminable background is a review of
/// the light render already. Nothing is added at WCAG level A, where the
/// contrast rules do not run.
pub(crate) async fn extend_with_dark_scheme(
    page: &Page,
    ax_tree: &AXTree,
    config: &PipelineConfig,
    viewport: &'static str,
    exclusion: &ExclusionScope,
    results: &mut WcagResults,
    excluded: &mut Vec<Violation>,
) {
    if !matches!(config.wcag_level, WcagLevel::AA | WcagLevel::AAA) {
        return;
    }
    match evaluate_js_value::<bool>(page, DECLARES_DARK_JS).await {
        Collected::Measured(true) => {}
        Collected::Measured(false) => return,
        other => {
            warn!("{viewport}: dark colour scheme not detectable: {other:?}");
            mark_not_measured(results, viewport);
            return;
        }
    }

    let base = crate::display::media_features(config.display_mode);
    if let Err(e) = emulate_scheme(page, &base, true).await {
        warn!("{viewport}: could not emulate the dark colour scheme: {e}");
        mark_not_measured(results, viewport);
        return;
    }
    let settle = wait_for_finite_animations(page, config.stability_budget_ms).await;
    if !settle.settled {
        warn!(
            "{viewport}: {} animation(s) still running after switching to the dark scheme",
            settle.running
        );
    }

    let measured = match fetch_dom_capture(page).await {
        Ok(capture) => capture
            .document(ax_tree)
            .map(|doc| wcag::shared::run_shared_rules(&doc, &config.lang))
            .map_err(|e| e.to_string()),
        Err(e) => Err(e.to_string()),
    };
    if let Err(e) = emulate_scheme(page, &base, false).await {
        warn!("{viewport}: could not restore the colour scheme: {e}");
    }
    let mut dark = match measured {
        Ok(dark) => dark,
        Err(e) => {
            warn!("{viewport}: DOM of the dark colour scheme not available: {e}");
            mark_not_measured(results, viewport);
            return;
        }
    };
    wcag::shared::retain_up_to_level(&mut dark, config.wcag_level);
    wcag::shared::retain_allowed(&mut dark, &config.rule_filter);
    results.localized_texts.extend(dark.localized_texts.clone());

    let (dropped, kept): (Vec<_>, Vec<_>) = new_dark_failures(dark, &results.violations)
        .into_iter()
        .partition(|v| exclusion.excludes_located(v, ax_tree));
    excluded.extend(dropped);

    for v in &kept {
        let Some(id) = v.rule_id.as_deref() else {
            continue;
        };
        if let Some(outcome) = results
            .rule_outcomes
            .iter_mut()
            .find(|o| o.rule_id == id && o.viewport.as_deref() == Some(viewport))
        {
            outcome.findings += 1;
        }
    }
    results.violations.extend(kept);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn contrast(selector: &str, message: &str) -> Violation {
        Violation::new(
            "1.4.3",
            "Contrast (Minimum)",
            WcagLevel::AA,
            wcag::Severity::High,
            message,
            "n",
        )
        .with_rule_id("contrast/text-insufficient")
        .with_selector(selector)
    }

    fn results(violations: Vec<Violation>) -> WcagResults {
        let mut r = WcagResults::new();
        r.violations = violations;
        r
    }

    #[test]
    fn dark_failures_carry_the_color_scheme_attribute_and_the_same_rule() {
        let found = new_dark_failures(
            results(vec![contrast("p#a", "ratio 2.1:1")]),
            &[contrast("p#b", "ratio 2.1:1")],
        );
        assert_eq!(found.len(), 1);
        assert_eq!(
            found[0].rule_id.as_deref(),
            Some("contrast/text-insufficient")
        );
        assert!(found[0].is_dark_color_scheme());
    }

    #[test]
    fn a_dark_finding_equal_to_the_light_one_is_not_reported_twice() {
        let light = [contrast("p#a", "ratio 2.1:1")];
        let found = new_dark_failures(results(vec![contrast("p#a", "ratio 2.1:1")]), &light);
        assert!(found.is_empty(), "emulation changed nothing for p#a");
    }

    #[test]
    fn the_same_element_failing_differently_in_dark_is_reported() {
        let light = [contrast("p#a", "ratio 2.1:1")];
        let found = new_dark_failures(results(vec![contrast("p#a", "ratio 1.4:1")]), &light);
        assert_eq!(found.len(), 1);
    }

    #[test]
    fn only_confirmed_contrast_failures_are_carried_over() {
        let mut dark = results(vec![Violation::new(
            "1.1.1",
            "Alt",
            WcagLevel::A,
            wcag::Severity::High,
            "no alt",
            "n",
        )
        .with_rule_id("images/alt-missing")]);
        dark.warnings
            .push(contrast("p#u", "undetermined").as_warning());
        assert!(new_dark_failures(dark, &[]).is_empty());
    }

    #[test]
    fn light_findings_are_not_marked() {
        assert!(!contrast("p#a", "m").is_dark_color_scheme());
    }
}
