//! WCAG 4.1.2 - Action disguised as navigation
//!
//! `<a href="#" onclick="...">` or `<a href="javascript:void(0)" onclick="...">`
//! used as a substitute for a `<button>`. The link role announces navigation
//! to screen reader users, but activating it triggers an action instead — and
//! it only responds to Enter, not Space, unlike a real button. Deterministic:
//! an `<a>` with a non-navigating `href` (`#`, empty fragment, or a
//! `javascript:` pseudo-URL) that also carries an inline `onclick` handler
//! (plan/18). Only inline `onclick=` is visible from DOM inspection —
//! `addEventListener('click', ...)` is not, same limitation as
//! `click_handlers.rs`.

use chromiumoxide::Page;

use crate::cli::WcagLevel;
use crate::wcag::types::{evaluate_or_fail, RuleMetadata, Severity, Violation};

pub const FAKE_NAVIGATION_LINK_RULE: RuleMetadata = RuleMetadata {
    id: "4.1.2",
    name: "Name, Role, Value",
    level: WcagLevel::A,
    severity: Severity::Low,
    description:
        "A link with no real navigation target must not be used as a substitute for a button",
    help_url: "https://www.w3.org/WAI/WCAG22/Understanding/name-role-value.html",
    axe_id: "link-as-button",
    tags: &["wcag2a", "wcag412", "cat.aria", "best-practice"],
};

// Selector uses the shared `__amsCssSelector` helper (never a bare tag name,
// see `js_helpers::CSS_SELECTOR_JS`) so two same-tag links without an
// id/class remain distinguishable in the report.
const FAKE_NAVIGATION_LINK_JS: &str = r#"
  var findings = [];
  var links = document.querySelectorAll('a[onclick]');
  for (var i = 0; i < links.length && findings.length < 20; i++) {
    var el = links[i];
    var href = (el.getAttribute('href') || '').trim();
    var isFragmentOnly = href === '#' || href === '';
    var isJsPseudoUrl = /^javascript:/i.test(href);
    if (!isFragmentOnly && !isJsPseudoUrl) continue;
    var role = (el.getAttribute('role') || '').toLowerCase();
    if (role === 'button') continue;
    findings.push(__amsCssSelector(el));
  }
  return findings;
"#;

pub async fn check_fake_navigation_link_with_page(page: &Page) -> Vec<Violation> {
    let js = [
        "(function() {",
        crate::accessibility::js_helpers::CSS_SELECTOR_JS,
        FAKE_NAVIGATION_LINK_JS,
        "})()",
    ]
    .concat();
    let value = match evaluate_or_fail(page, &FAKE_NAVIGATION_LINK_RULE, &js).await {
        Ok(v) => v,
        Err(violations) => return violations,
    };

    let Some(selectors) = value.as_array() else {
        return vec![];
    };

    selectors
        .iter()
        .filter_map(|s| s.as_str())
        .map(|selector| {
            Violation::new(
                FAKE_NAVIGATION_LINK_RULE.id,
                FAKE_NAVIGATION_LINK_RULE.name,
                FAKE_NAVIGATION_LINK_RULE.level,
                FAKE_NAVIGATION_LINK_RULE.severity,
                "Link has no real navigation target (href=\"#\" or a javascript: pseudo-URL) and \
                 is used to trigger an action instead — announced as a link, but does not \
                 navigate, and does not respond to the Space key like a real button."
                    .to_string(),
                selector,
            )
            .with_selector(selector)
            .with_rule_id(FAKE_NAVIGATION_LINK_RULE.axe_id)
            .with_tags(
                FAKE_NAVIGATION_LINK_RULE
                    .tags
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
            )
            .with_fix(
                "Use a native <button> for actions that don't navigate, or add role=\"button\" \
                 plus keyboard support for both Enter and Space.",
            )
            .with_help_url(FAKE_NAVIGATION_LINK_RULE.help_url)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rule_metadata_is_low_severity_best_practice() {
        assert_eq!(FAKE_NAVIGATION_LINK_RULE.severity, Severity::Low);
        assert!(FAKE_NAVIGATION_LINK_RULE.tags.contains(&"best-practice"));
    }
}
