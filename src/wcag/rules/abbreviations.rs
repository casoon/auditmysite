//! WCAG 3.1.4 Abbreviations (Level AAA)
//!
//! A mechanism for identifying the expanded form or meaning of abbreviations
//! is available. The `<abbr>` and `<acronym>` elements should carry a `title`
//! attribute with the expansion.

use chromiumoxide::Page;

use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation};

pub const ABBREVIATIONS_RULE: RuleMetadata = RuleMetadata {
    id: "3.1.4",
    name: "Abbreviations",
    level: WcagLevel::AAA,
    severity: Severity::Low,
    description: "Abbreviations have a mechanism to reveal their expanded form",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/abbreviations.html",
    axe_id: "abbreviations",
    tags: &["wcag2aaa", "wcag314", "cat.language"],
};

const ABBREVIATIONS_JS: &str = r#"
(function() {
  var elements = document.querySelectorAll('abbr:not([title]), acronym:not([title])');
  var results = [];
  for (var i = 0; i < Math.min(elements.length, 10); i++) {
    var el = elements[i];
    results.push({
      text: el.textContent.trim().substring(0, 20),
      tag: el.tagName.toLowerCase()
    });
  }
  return { elements: results };
})()
"#;

pub async fn check_abbreviations_with_page(page: &Page) -> Vec<Violation> {
    let result = match page.evaluate(ABBREVIATIONS_JS).await {
        Ok(r) => r,
        Err(_) => return vec![],
    };

    let val = match result.value() {
        Some(v) => v.clone(),
        None => return vec![],
    };

    let elements = match val.get("elements").and_then(|v| v.as_array()) {
        Some(arr) => arr.clone(),
        None => return vec![],
    };

    elements
        .iter()
        .map(|item| {
            let text = item.get("text").and_then(|v| v.as_str()).unwrap_or("abbr");
            let tag = item.get("tag").and_then(|v| v.as_str()).unwrap_or("abbr");

            Violation::new(
                ABBREVIATIONS_RULE.id,
                ABBREVIATIONS_RULE.name,
                ABBREVIATIONS_RULE.level,
                Severity::Low,
                format!(
                    "<{}> element '{}' has no title attribute to expand the abbreviation.",
                    tag, text
                ),
                tag,
            )
            .with_selector(tag)
            .with_fix(
                "Add a title attribute to <abbr> elements with the full expansion: \
                 e.g. <abbr title=\"World Wide Web Consortium\">W3C</abbr>.",
            )
            .with_rule_id(ABBREVIATIONS_RULE.axe_id)
            .with_help_url(ABBREVIATIONS_RULE.help_url)
        })
        .collect()
}
