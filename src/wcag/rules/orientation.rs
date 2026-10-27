//! WCAG 1.3.4 Orientation
//!
//! Content does not restrict its view and operation to a single display
//! orientation, such as portrait or landscape, unless a specific display
//! orientation is essential.
//! Level AA
//!
//! Checks whether `<html>` or `<body>` is rotated in the audited viewport
//! (computed `transform` matrix or `rotate` property). The
//! stylesheet half (orientation media queries that hide or rotate content)
//! runs as `orientation/content-hidden` in the shared rules since a11y-rules
//! 0.19 (`a11y_rules::run_stylesheets`); the computed style of the live page
//! stays here.

use chromiumoxide::Page;

use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation};

pub(super) const ORIENTATION_RULE: RuleMetadata = RuleMetadata {
    id: "1.3.4",
    name: "Orientation",
    level: WcagLevel::AA,
    severity: Severity::High,
    description: "Content must not be restricted to a single display orientation",
    help_url: "https://www.w3.org/WAI/WCAG22/Understanding/orientation.html",
    axe_id: "css-orientation-lock",
    tags: &["wcag2aa", "wcag134", "cat.sensory-and-visual-cues"],
};

// Returns a JSON object: { locked: bool, element: string, detail: string }.
//
// The computed `transform` is never a `rotate(...)` string: Chrome serialises
// it as `none` or `matrix(...)` / `matrix3d(...)`. A rotation shows in the
// matrix as b = sin θ and c = -sin θ; skew or scale alone do not produce that
// pair. The individual `rotate` property computes to `none` or an angle.
// Query errors are not caught: a probe that cannot read the computed style
// has measured nothing and must surface as a technical failure, not a pass.
const ORIENTATION_LOCK_JS: &str = r#"
(function() {
  function rotatedBy(style) {
    var t = style.transform;
    if (t && t !== 'none') {
      var m = t.match(/^matrix(3d)?\(([^)]*)\)$/);
      if (m) {
        var v = m[2].split(',').map(Number);
        var b = v[1];
        var c = m[1] ? v[4] : v[2];
        if (Math.abs(b) > 0.01 && Math.abs(b + c) < 0.01) {
          return Math.round(Math.atan2(b, v[0]) * 180 / Math.PI) + 'deg';
        }
      }
    }
    var r = style.rotate;
    if (r && r !== 'none') {
      var deg = parseFloat(r);
      if (!isNaN(deg) && Math.abs(deg % 180) > 0.5) return r;
    }
    return null;
  }
  var pairs = [['html', document.documentElement], ['body', document.body]];
  for (var i = 0; i < pairs.length; i++) {
    if (!pairs[i][1]) continue;
    var angle = rotatedBy(window.getComputedStyle(pairs[i][1]));
    if (angle) {
      return { locked: true, element: pairs[i][0], detail: pairs[i][0] + ' is rotated by ' + angle + ' in this viewport' };
    }
  }
  return { locked: false, element: '', detail: '' };
})()
"#;

pub async fn check_orientation_with_page(page: &Page) -> Vec<Violation> {
    let val =
        match crate::wcag::types::evaluate_or_fail(page, &ORIENTATION_RULE, ORIENTATION_LOCK_JS)
            .await
        {
            Ok(v) => v,
            Err(violations) => return violations,
        };

    // The probe always answers `locked`; without it nothing was measured.
    let Some(locked) = val.get("locked").and_then(|v| v.as_bool()) else {
        return vec![crate::wcag::technical_rule_failure(
            &ORIENTATION_RULE,
            "invalid_evaluation_value",
        )];
    };
    if !locked {
        return vec![];
    }

    let detail = val
        .get("detail")
        .and_then(|v| v.as_str())
        .unwrap_or("orientation lock detected")
        .to_string();

    vec![Violation::new(
        ORIENTATION_RULE.id,
        ORIENTATION_RULE.name,
        ORIENTATION_RULE.level,
        Severity::High,
        format!(
            "Page appears to lock display orientation — {detail}. Users who cannot rotate their device will be unable to access content."
        ),
        "document",
    )
    .with_selector(
        val.get("element")
            .and_then(|v| v.as_str())
            .filter(|e| !e.is_empty())
            .unwrap_or("html"),
    )
    .with_fix(
        "Remove orientation-specific CSS that hides or rotates content. \
         Let the OS and browser handle orientation changes. \
         Only lock orientation if it is essential to the content (e.g. a piano keyboard).",
    )
    .with_rule_id(ORIENTATION_RULE.axe_id)
    .with_help_url(ORIENTATION_RULE.help_url)]
}
