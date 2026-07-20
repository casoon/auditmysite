//! WCAG 2.5.3 Label in Name
//!
//! For user interface components with labels that include text or images of text,
//! the name contains the text that is presented visually.
//! Checked via the DOM-based page rule for button label mismatch.

use chromiumoxide::Page;

use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation};

pub const LABEL_IN_NAME_PAGE_RULE: RuleMetadata = RuleMetadata {
    id: "2.5.3",
    name: "Label in Name",
    level: WcagLevel::A,
    severity: Severity::Medium,
    description: "The accessible name of a button contains its visible label text",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/label-in-name.html",
    axe_id: "label-content-name-mismatch",
    tags: &["wcag2a", "wcag253", "cat.semantics"],
};

const LABEL_IN_NAME_JS: &str = r#"
(function() {
  // Strips everything but letters/digits before comparing. `textContent`
  // never inserts whitespace at element boundaries (e.g. `<h3>A.</h3><p>B</p>`
  // concatenates to "A.B", not "A. B"), so a naive space-sensitive compare
  // flags plenty of real "aria-label matches, punctuation/whitespace
  // doesn't" cases as violations (#513).
  function normalize(s) {
    return s.replace(/[^a-z0-9]/gi, '').toLowerCase();
  }
  var violations = [];
  var buttons = document.querySelectorAll('button[aria-label], [role="button"][aria-label]');
  for (var i = 0; i < Math.min(buttons.length, 50); i++) {
    var el = buttons[i];
    var ariaLabelRaw = (el.getAttribute('aria-label') || '').trim();
    var visibleTextRaw = (el.textContent || '').trim();
    var ariaLabel = normalize(ariaLabelRaw);
    var visibleText = normalize(visibleTextRaw);
    if (visibleText && ariaLabel && ariaLabel.indexOf(visibleText) === -1 && visibleText.indexOf(ariaLabel) === -1) {
      violations.push({
        selector: el.tagName.toLowerCase() + (el.id ? '#' + el.id : ''),
        ariaLabel: ariaLabelRaw.substring(0, 200),
        visibleText: visibleTextRaw.substring(0, 200),
        ariaLabelNormLen: ariaLabel.length,
        visibleTextNormLen: visibleText.length
      });
    }
  }
  return { violations: violations };
})()
"#;

/// A `visibleText` collected from `textContent` this many times longer than
/// `ariaLabel` (after normalization) is a strong signal of a compound
/// widget whose author wrote a deliberately concise `aria-label` for far
/// more visual content than the check's simple substring-containment logic
/// can verify (e.g. a flip-card exposing both faces' text through one
/// button) -- not a confirmed mismatch of a genuine short label. Downgraded
/// to a warning rather than dropped entirely: real "the label really
/// doesn't match" cases still deserve a look (#513).
const COMPOUND_WIDGET_LENGTH_RATIO: u64 = 2;

pub async fn check_label_in_name_with_page(page: &Page) -> Vec<Violation> {
    let val = match crate::wcag::types::evaluate_or_fail(
        page,
        &LABEL_IN_NAME_PAGE_RULE,
        LABEL_IN_NAME_JS,
    )
    .await
    {
        Ok(v) => v,
        Err(violations) => return violations,
    };

    let violations = match val.get("violations").and_then(|v| v.as_array()) {
        Some(arr) => arr.clone(),
        None => {
            return vec![crate::wcag::technical_rule_failure(
                &LABEL_IN_NAME_PAGE_RULE,
                "invalid_evaluation_shape",
            )]
        }
    };

    violations
        .iter()
        .map(|item| {
            let selector = item
                .get("selector")
                .and_then(|v| v.as_str())
                .unwrap_or("button");
            let aria_label = item.get("ariaLabel").and_then(|v| v.as_str()).unwrap_or("");
            let visible_text = item
                .get("visibleText")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let norm_len = |key: &str| item.get(key).and_then(|v| v.as_u64()).unwrap_or(0);
            let aria_label_norm_len = norm_len("ariaLabelNormLen");
            let visible_text_norm_len = norm_len("visibleTextNormLen");
            let likely_compound_widget = aria_label_norm_len > 0
                && visible_text_norm_len > aria_label_norm_len * COMPOUND_WIDGET_LENGTH_RATIO;

            let violation = Violation::new(
                LABEL_IN_NAME_PAGE_RULE.id,
                LABEL_IN_NAME_PAGE_RULE.name,
                LABEL_IN_NAME_PAGE_RULE.level,
                Severity::Medium,
                format!(
                    "Button aria-label '{}' does not contain its visible text '{}'. \
                     Speech input users who speak the visible label will not activate \
                     the button.",
                    aria_label, visible_text
                ),
                selector,
            )
            .with_selector(selector)
            .with_fix(
                "Ensure the aria-label starts with or contains the visible button text. \
                 For example, if visible text is 'Search', use aria-label=\"Search products\" \
                 not aria-label=\"Find items\".",
            )
            .with_rule_id(LABEL_IN_NAME_PAGE_RULE.axe_id)
            .with_help_url(LABEL_IN_NAME_PAGE_RULE.help_url);

            if likely_compound_widget {
                violation.as_warning()
            } else {
                violation
            }
        })
        .collect()
}
