//! `display/*` — checks of the BarrierLab display-mode convention (#653).
//!
//! The convention (draft v0, `barrierlab/plan/03`) is not a WCAG requirement:
//! every rule here is tagged `best-practice` and anchored to the nearest WCAG
//! criterion, the same way the other own best-practice rules are modelled
//! (`link-as-button` → 4.1.2, `prefers-reduced-motion` → 2.3.3):
//!
//! | id | anchor | reads |
//! |---|---|---|
//! | `display/toggle-missing` | 2.2.2 | post-JS DOM |
//! | `display/init-missing` | 2.2.2 | pre-navigation body observer (CDP) |
//! | `display/text-media-visible` | 2.2.2 | rendered layout in `text` |
//! | `display/text-not-visible` | 1.1.1 | rendered layout in `text` |
//! | `display/text-hidden` | 1.1.1 | post-JS DOM + computed style |
//!
//! The text-mode checks key on the mode the page actually rendered in
//! (`html[data-display="text"]`), not on `--display`: a page that ignores the
//! stored choice is audited as what it shows.
//!
//! Barrierlab move candidates (CLAUDE.md "Umzugskandidaten"): the *static*
//! halves of `display/toggle-missing` (a `figure[data-viz]` without any
//! `[data-display-toggle]` in the HTML) and `display/text-hidden` (the
//! `hidden`/`aria-hidden`/`inert` attribute part, the draft's
//! `viz/text-hidden`) need no browser and belong in the shared implementation
//! astro-post-audit will also run. They are here only because this feature
//! needs them against the live DOM; once barrierlab ships them, the static
//! part moves there and this file keeps the rendering-dependent part
//! (computed visibility, the body observer, the text-mode layout checks).

use chromiumoxide::Page;

use crate::cli::WcagLevel;
use crate::wcag::types::{evaluate_or_fail, RuleMetadata, Severity, Violation};

// No public page for the convention yet (draft v0); link the anchor criterion.
const HELP_URL_222: &str = "https://www.w3.org/WAI/WCAG22/Understanding/pause-stop-hide.html";
const HELP_URL_111: &str = "https://www.w3.org/WAI/WCAG22/Understanding/non-text-content.html";

pub const DISPLAY_TOGGLE_MISSING_RULE: RuleMetadata = RuleMetadata {
    id: "2.2.2",
    name: "Display modes: toggle",
    level: WcagLevel::A,
    severity: Severity::Medium,
    description: "A page with visualisations must offer a display-mode toggle",
    help_url: HELP_URL_222,
    axe_id: "display/toggle-missing",
    tags: &["best-practice", "cat.display-modes"],
};

pub const DISPLAY_INIT_MISSING_RULE: RuleMetadata = RuleMetadata {
    id: "2.2.2",
    name: "Display modes: set before first paint",
    level: WcagLevel::A,
    severity: Severity::Low,
    description: "html[data-display] must be set before the page body renders",
    help_url: HELP_URL_222,
    axe_id: "display/init-missing",
    tags: &["best-practice", "cat.display-modes"],
};

pub const DISPLAY_TEXT_MEDIA_VISIBLE_RULE: RuleMetadata = RuleMetadata {
    id: "2.2.2",
    name: "Display modes: no visualisations in text mode",
    level: WcagLevel::A,
    severity: Severity::Medium,
    description: "In text mode no visualisation media may be visible",
    help_url: HELP_URL_222,
    axe_id: "display/text-media-visible",
    tags: &["best-practice", "cat.display-modes"],
};

pub const DISPLAY_TEXT_NOT_VISIBLE_RULE: RuleMetadata = RuleMetadata {
    id: "1.1.1",
    name: "Display modes: text layer visible in text mode",
    level: WcagLevel::A,
    severity: Severity::High,
    description: "In text mode every visualisation must show a non-empty [data-viz-text]",
    help_url: HELP_URL_111,
    axe_id: "display/text-not-visible",
    tags: &["best-practice", "cat.display-modes"],
};

pub const DISPLAY_TEXT_HIDDEN_RULE: RuleMetadata = RuleMetadata {
    id: "1.1.1",
    name: "Display modes: text layer reachable",
    level: WcagLevel::A,
    severity: Severity::High,
    description: "[data-viz-text] must stay reachable for assistive technology in every mode",
    help_url: HELP_URL_111,
    axe_id: "display/text-hidden",
    tags: &["best-practice", "cat.display-modes"],
};

// Returns [{ id, selector, detail }]. `id` is the short key matched in
// `violation_for` below.
const DISPLAY_MODES_JS: &str = r#"
  var findings = [];
  var MAX = 20;
  var html = document.documentElement;
  var mode = (html.getAttribute('data-display') || '').trim().toLowerCase();
  var figs = Array.prototype.slice.call(document.querySelectorAll('figure[data-viz]'));
  var hasToggle = !!document.querySelector('[data-display-toggle]');

  function rendered(el) {
    if (el.checkVisibility && !el.checkVisibility({ checkOpacity: true, checkVisibilityCSS: true })) {
      return false;
    }
    var r = el.getBoundingClientRect();
    return r.width > 1 && r.height > 1;
  }
  function hiddenFromAt(el) {
    for (var n = el; n && n.nodeType === 1; n = n.parentElement) {
      if (n.hasAttribute('hidden')) return 'hidden';
      if ((n.getAttribute('aria-hidden') || '').trim().toLowerCase() === 'true') return 'aria-hidden';
      if (n.hasAttribute('inert')) return 'inert';
    }
    if (el.checkVisibility && !el.checkVisibility({ checkVisibilityCSS: true })) return 'css';
    return null;
  }
  // A text layer that a rendered, reachable element of the same figure
  // (the figure itself included) references by aria-describedby or
  // aria-details (#704).
  function describedIn(fig, el) {
    if (!el.id) return false;
    var refs = [fig].concat(Array.prototype.slice.call(fig.querySelectorAll('[aria-describedby], [aria-details]')));
    for (var i = 0; i < refs.length; i++) {
      var r = refs[i];
      if (r === el || el.contains(r)) continue;
      var ids = ((r.getAttribute('aria-describedby') || '') + ' ' + (r.getAttribute('aria-details') || '')).split(/\s+/);
      if (ids.indexOf(el.id) >= 0 && rendered(r) && !hiddenFromAt(r)) return true;
    }
    return false;
  }
  // Capped through __amsPush so excluded specimens never spend the budget (#645).
  function push(id, el, detail) {
    __amsPush(findings, el, { id: id, selector: el === html ? 'html' : __amsCssSelector(el), detail: detail || '' }, MAX);
  }

  if (figs.length && !hasToggle) push('toggle', html, String(figs.length));

  if (figs.length || hasToggle) {
    if (!html.hasAttribute('data-display')) {
      push('init', html, 'never');
    } else if (window.__amsDisplayObserved === true && window.__amsDisplayAtBody == null) {
      push('init', html, 'late');
    }
  }

  for (var i = 0; i < figs.length && __amsReal(findings) < MAX; i++) {
    var fig = figs[i];
    var texts = Array.prototype.slice.call(fig.querySelectorAll('[data-viz-text]'));
    var hiddenReported = false;
    for (var t = 0; t < texts.length; t++) {
      var reason = hiddenFromAt(texts[t]);
      // A figure that is itself not rendered hides its visual together with
      // its text layer — by CSS (closed <details>, collapsed tab) or by
      // `hidden`/aria-hidden/inert on the figure or an ancestor (an inactive
      // scope panel, #725). Nobody gets the statement in another form, so
      // this is not a convention breach.
      var hiddenWithFig = !rendered(fig) && (reason === 'css' || hiddenFromAt(fig) !== null);
      if (reason && !hiddenWithFig) {
        // accname 1.2 computes a description from a directly referenced
        // node even when it is hidden — by CSS, `hidden` or aria-hidden
        // alike (ARIA15 is a sufficient technique for 1.1.1). The statement
        // then still reaches assistive technology, flattened into a
        // description, so this is a note, not a breach. `inert` is outside
        // accname's notion of hidden; HTML lets user agents drop inert
        // content from the accessibility tree altogether, so it stays one.
        if (!texts[t].closest('[inert]') && describedIn(fig, texts[t])) {
          push('described', texts[t], reason);
        } else {
          push('hidden', texts[t], reason);
          hiddenReported = true;
        }
      }
    }
    if (mode !== 'text') continue;

    var textOk = texts.some(function (el) {
      return el.textContent.trim().length > 0 && rendered(el) && !hiddenFromAt(el);
    });
    if (!textOk && !hiddenReported) push('text', fig, texts.length ? 'empty_or_invisible' : 'missing');

    var media = fig.querySelectorAll('[data-viz-live], [data-viz-static], canvas, video, svg, img, iframe, object, embed');
    for (var m = 0; m < media.length; m++) {
      var el = media[m];
      var textLayer = el.closest('[data-viz-text]');
      if (textLayer && fig.contains(textLayer)) continue;
      if (!rendered(el)) continue;
      var r = el.getBoundingClientRect();
      // Icons stay allowed; the draft leaves the exact boundary open, 32x32
      // is the common icon ceiling.
      if (r.width * r.height <= 32 * 32) continue;
      push('media', el, el.tagName.toLowerCase());
      break;
    }
  }
  return findings;
"#;

pub async fn check_display_modes_with_page(page: &Page) -> Vec<Violation> {
    let js = [
        "(function() {",
        crate::accessibility::js_helpers::CSS_SELECTOR_JS,
        DISPLAY_MODES_JS,
        "})()",
    ]
    .concat();
    let value = match evaluate_or_fail(page, &DISPLAY_TOGGLE_MISSING_RULE, &js).await {
        Ok(v) => v,
        Err(violations) => return violations,
    };
    let Some(items) = value.as_array() else {
        return vec![];
    };
    items
        .iter()
        .filter_map(|item| {
            violation_for(
                item.get("id")?.as_str()?,
                item.get("selector")?.as_str()?,
                item.get("detail").and_then(|d| d.as_str()).unwrap_or(""),
            )
        })
        .collect()
}

fn violation_for(id: &str, selector: &str, detail: &str) -> Option<Violation> {
    let (rule, message, fix) = match id {
        "toggle" => (
            &DISPLAY_TOGGLE_MISSING_RULE,
            format!(
                "The page contains {detail} visualisation(s) (figure[data-viz]), but no \
                 toggle marked [data-display-toggle] was found."
            ),
            "Add a reachable, operable control marked [data-display-toggle] that switches \
             html[data-display] between visual, calm and text and stores the choice in \
             localStorage under the key \"display\".",
        ),
        "init" => (
            &DISPLAY_INIT_MISSING_RULE,
            if detail == "never" {
                "The page uses the display-mode convention but never sets html[data-display], \
                 so the chosen mode is not applied."
                    .to_string()
            } else {
                "html[data-display] was not yet set when the page body was parsed, so the page \
                 can render in the wrong mode (for example with animation) before switching."
                    .to_string()
            },
            "Set html[data-display] in a small blocking script in <head>, before <body>: read \
             localStorage \"display\", fall back to \"calm\" with prefers-reduced-motion.",
        ),
        "media" => (
            &DISPLAY_TEXT_MEDIA_VISIBLE_RULE,
            format!(
                "In text mode a visualisation still shows its media (<{detail}>); text mode \
                 promises the statement and values as text only."
            ),
            "Hide [data-viz-live] and [data-viz-static] (and any canvas/svg/video of the \
             figure) when html[data-display=\"text\"], and show [data-viz-text] instead.",
        ),
        "text" => (
            &DISPLAY_TEXT_NOT_VISIBLE_RULE,
            if detail == "missing" {
                "In text mode this visualisation has no [data-viz-text], so its statement is \
                 lost."
                    .to_string()
            } else {
                "In text mode this visualisation's [data-viz-text] is empty or not visible, so \
                 its statement is lost."
                    .to_string()
            },
            "Give every figure[data-viz] one [data-viz-text] with the statement (for charts the \
             values, preferably as a <table>) and its source, and show it in text mode.",
        ),
        "hidden" => (
            &DISPLAY_TEXT_HIDDEN_RULE,
            format!(
                "The visualisation's text layer [data-viz-text] is removed from assistive \
                 technology ({detail}); screen reader users lose the statement in this mode."
            ),
            "Never put hidden, aria-hidden=\"true\" or inert on [data-viz-text] or its \
             ancestors; to hide it visually in visual/calm mode use a visually-hidden class.",
        ),
        "described" => (
            &DISPLAY_TEXT_HIDDEN_RULE,
            format!(
                "The visualisation's text layer [data-viz-text] is hidden ({detail}), but a \
                 rendered element of the figure references it by aria-describedby or \
                 aria-details, so screen readers still get the statement, only as a \
                 description with its structure flattened."
            ),
            "Hide [data-viz-text] with a visually-hidden class instead, so it stays in the \
             reading order with its structure (tables, lists).",
        ),
        _ => return None,
    };
    let violation = Violation::new(
        rule.id,
        rule.name,
        rule.level,
        rule.severity,
        message,
        selector,
    )
    .with_selector(selector)
    .with_rule_id(rule.axe_id)
    .with_tags(rule.tags.iter().map(|s| s.to_string()).collect())
    .with_fix(fix)
    .with_help_url(rule.help_url);
    // Description-only delivery is a note for review, not a breach (#704).
    Some(if id == "described" {
        Violation {
            severity: Severity::Low,
            ..violation
        }
        .as_warning()
    } else {
        violation
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_finding_key_maps_to_its_own_rule() {
        let cases = [
            ("toggle", "display/toggle-missing"),
            ("init", "display/init-missing"),
            ("media", "display/text-media-visible"),
            ("text", "display/text-not-visible"),
            ("hidden", "display/text-hidden"),
        ];
        for (key, axe_id) in cases {
            let v = violation_for(key, "figure", "x").expect(key);
            assert_eq!(v.rule_id.as_deref(), Some(axe_id));
            assert!(v.tags.iter().any(|t| t == "best-practice"));
        }
        assert!(violation_for("unknown", "html", "").is_none());
    }

    #[test]
    fn messages_are_canonical_english() {
        for key in ["toggle", "init", "media", "text", "hidden", "described"] {
            let v = violation_for(key, "figure", "never").unwrap();
            let text = format!("{} {}", v.message, v.fix_suggestion.unwrap_or_default());
            assert!(!text.chars().any(|c| "äöüÄÖÜß".contains(c)), "{text}");
        }
    }

    #[test]
    fn a_text_layer_delivered_as_description_is_a_low_review_note() {
        // #704: CSS-hidden [data-viz-text] referenced by aria-describedby.
        let v = violation_for("described", "div#layers-home-desc", "css").unwrap();
        assert_eq!(v.rule_id.as_deref(), Some("display/text-hidden"));
        assert_eq!(v.kind, crate::wcag::types::Outcome::Review);
        assert_eq!(v.severity, Severity::Low);
        let hidden = violation_for("hidden", "div#layers-home-desc", "css").unwrap();
        assert_eq!(hidden.kind, crate::wcag::types::Outcome::Fail);
        assert_eq!(hidden.severity, Severity::High);
    }

    #[test]
    fn toggle_message_reports_the_missing_marker_not_a_missing_ability() {
        let v = violation_for("toggle", "html", "2").unwrap();
        assert!(
            v.message
                .contains("no toggle marked [data-display-toggle] was found"),
            "{}",
            v.message
        );
        assert!(!v.message.contains("cannot switch"), "{}", v.message);
    }
}
