//! WCAG 2.4.8 Location (Level AAA)
//!
//! Information about the user's location within a set of Web pages is
//! available. A breadcrumb trail, site map, or aria-current="page" indicator
//! satisfies this criterion.

use chromiumoxide::Page;

use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation};

pub const LOCATION_RULE: RuleMetadata = RuleMetadata {
    id: "2.4.8",
    name: "Location",
    level: WcagLevel::AAA,
    severity: Severity::Low,
    description: "Users can determine their location within a set of web pages",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/location.html",
    axe_id: "location",
    tags: &["wcag2aaa", "wcag248", "cat.navigation"],
};

const LOCATION_JS: &str = r#"
(function() {
  var hasBreadcrumb = !!(
    document.querySelector('nav[aria-label*="breadcrumb" i]') ||
    document.querySelector('[role="navigation"][aria-label*="breadcrumb" i]') ||
    document.querySelector('ol[aria-label*="breadcrumb" i]') ||
    document.querySelector('[aria-label*="breadcrumb" i]')
  );
  var hasAriaCurrent = !!document.querySelector('[aria-current="page"]');
  var hasNav = !!document.querySelector('nav a, [role="navigation"] a');
  return { hasBreadcrumb: hasBreadcrumb, hasAriaCurrent: hasAriaCurrent, hasNav: hasNav };
})()
"#;

pub async fn check_location_with_page(page: &Page) -> Vec<Violation> {
    let result = match page.evaluate(LOCATION_JS).await {
        Ok(r) => r,
        Err(_) => {
            return vec![crate::wcag::technical_rule_failure(
                &LOCATION_RULE,
                "page_evaluation_failed",
            )]
        }
    };

    let val = match result.value() {
        Some(v) => v.clone(),
        None => {
            return vec![crate::wcag::technical_rule_failure(
                &LOCATION_RULE,
                "missing_evaluation_value",
            )]
        }
    };

    let has_breadcrumb = val
        .get("hasBreadcrumb")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let has_aria_current = val
        .get("hasAriaCurrent")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let has_nav = val.get("hasNav").and_then(|v| v.as_bool()).unwrap_or(false);

    // Only flag if the page has navigation (implying multi-level site) but no location indicator
    if has_nav && !has_breadcrumb && !has_aria_current {
        return vec![Violation::new(
            LOCATION_RULE.id,
            LOCATION_RULE.name,
            LOCATION_RULE.level,
            Severity::Low,
            "No breadcrumb navigation or aria-current=\"page\" indicator found. \
             Users with cognitive disabilities may not be able to determine their \
             location within the site.",
            "page",
        )
        .with_fix(
            "Add a breadcrumb navigation (nav[aria-label=\"Breadcrumb\"]) or mark the \
             current page link with aria-current=\"page\" to help users understand \
             where they are in the site structure.",
        )
        .with_rule_id(LOCATION_RULE.axe_id)
        .with_help_url(LOCATION_RULE.help_url)];
    }

    vec![]
}
