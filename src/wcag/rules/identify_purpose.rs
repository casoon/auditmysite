//! WCAG 1.3.6 Identify Purpose (Level AAA)
//!
//! The purpose of user interface components, icons, and regions can be
//! programmatically determined. Inputs collecting personal data should use
//! autocomplete tokens to allow user agents and AT to adapt the interface.

use chromiumoxide::Page;

use crate::cli::WcagLevel;
use crate::wcag::types::{FindingKind, RuleMetadata, Severity, Violation};

pub const IDENTIFY_PURPOSE_RULE: RuleMetadata = RuleMetadata {
    id: "1.3.6",
    name: "Identify Purpose",
    level: WcagLevel::AAA,
    severity: Severity::Low,
    description: "Purpose of UI components, icons, and regions can be programmatically determined",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/identify-purpose.html",
    axe_id: "identify-purpose",
    tags: &["wcag2aaa", "wcag136", "cat.semantics"],
};

const IDENTIFY_PURPOSE_JS: &str = r#"
(function() {
  var missing = [];
  var inputs = document.querySelectorAll('input[type="email"], input[type="tel"], input[type="text"], input[type="search"], input[name*="name" i], input[name*="address" i], input[name*="phone" i]');
  for (var i = 0; i < inputs.length; i++) {
    var el = inputs[i];
    if (!el.getAttribute('autocomplete')) {
      var desc = el.getAttribute('name') || el.getAttribute('id') || el.getAttribute('type') || 'input';
      missing.push(desc);
    }
  }
  return { missing: missing };
})()
"#;

pub async fn check_identify_purpose_with_page(page: &Page) -> Vec<Violation> {
    let not_testable = Violation::new(
        IDENTIFY_PURPOSE_RULE.id,
        IDENTIFY_PURPOSE_RULE.name,
        IDENTIFY_PURPOSE_RULE.level,
        Severity::Low,
        "Full WCAG 1.3.6 compliance (identify purpose of icons and regions) requires manual review. \
         Automated checks only cover inputs missing autocomplete tokens.",
        "page",
    )
    .with_fix(
        "Add appropriate ARIA landmark roles and autocomplete attributes to all interactive \
         components that collect personal information.",
    )
    .with_rule_id(IDENTIFY_PURPOSE_RULE.axe_id)
    .with_help_url(IDENTIFY_PURPOSE_RULE.help_url)
    .with_kind(FindingKind::NotTestable);

    let result = match page.evaluate(IDENTIFY_PURPOSE_JS).await {
        Ok(r) => r,
        Err(_) => return vec![not_testable],
    };

    let val = match result.value() {
        Some(v) => v.clone(),
        None => return vec![not_testable],
    };

    let missing: Vec<String> = val
        .get("missing")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let mut findings = vec![not_testable];

    for name in missing.iter().take(5) {
        findings.push(
            Violation::new(
                IDENTIFY_PURPOSE_RULE.id,
                IDENTIFY_PURPOSE_RULE.name,
                IDENTIFY_PURPOSE_RULE.level,
                Severity::Low,
                format!(
                    "Input '{}' collects personal information but has no autocomplete attribute.",
                    name
                ),
                name.as_str(),
            )
            .with_selector(name.as_str())
            .with_fix(
                "Add an appropriate autocomplete token (e.g. autocomplete=\"email\", \
                 autocomplete=\"tel\", autocomplete=\"name\") to help users and assistive technologies.",
            )
            .with_rule_id(IDENTIFY_PURPOSE_RULE.axe_id)
            .with_help_url(IDENTIFY_PURPOSE_RULE.help_url),
        );
    }

    findings
}
