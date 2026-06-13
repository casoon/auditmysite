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
    name: "Label in Name (Enhanced)",
    level: WcagLevel::AAA,
    severity: Severity::Medium,
    description: "The accessible name of a button contains its visible label text",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/label-in-name.html",
    axe_id: "label-content-name-mismatch",
    tags: &["wcag2aaa", "wcag253", "cat.semantics"],
};

const LABEL_IN_NAME_JS: &str = r#"
(function() {
  var violations = [];
  var buttons = document.querySelectorAll('button[aria-label], [role="button"][aria-label]');
  for (var i = 0; i < Math.min(buttons.length, 50); i++) {
    var el = buttons[i];
    var ariaLabel = (el.getAttribute('aria-label') || '').trim().toLowerCase();
    var visibleText = (el.textContent || '').trim().toLowerCase();
    if (visibleText && ariaLabel && ariaLabel.indexOf(visibleText) === -1 && visibleText.indexOf(ariaLabel) === -1) {
      violations.push({
        selector: el.tagName.toLowerCase() + (el.id ? '#' + el.id : ''),
        ariaLabel: el.getAttribute('aria-label').substring(0, 60),
        visibleText: el.textContent.trim().substring(0, 60)
      });
    }
  }
  return { violations: violations };
})()
"#;

pub async fn check_label_in_name_with_page(page: &Page) -> Vec<Violation> {
    let result = match page.evaluate(LABEL_IN_NAME_JS).await {
        Ok(r) => r,
        Err(_) => return vec![],
    };

    let val = match result.value() {
        Some(v) => v.clone(),
        None => return vec![],
    };

    let violations = match val.get("violations").and_then(|v| v.as_array()) {
        Some(arr) => arr.clone(),
        None => return vec![],
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

            Violation::new(
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
            .with_help_url(LABEL_IN_NAME_PAGE_RULE.help_url)
        })
        .collect()
}
