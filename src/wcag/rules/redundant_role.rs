//! WCAG 4.1.2 - Redundant native role, `<ul>`/`<ol>` only
//!
//! Since #697 every other tag/role pair runs as `aria/role-redundant` in the
//! shared rules. `<ul>`/`<ol>` with `role="list"` stays here: whether the
//! role is redundant depends on the computed `list-style-type` (#644), which
//! `a11y-dom` does not carry yet (casoon/barrierlab#21), and `a11y-rules`
//! does not judge the pair rather than guess.
//!
//! `<button role="button">`, `<a href="..." role="link">` and similar: the
//! explicit `role` restates the tag's own implicit role. No crash for modern
//! browsers/AT, but older combinations can announce the role twice
//! ("Schaltfläche Schaltfläche …", "button button"). Deterministic: a fixed
//! tag-to-implicit-role table checked against the element's own `role`
//! attribute. Only unambiguous, context-independent tag/role pairs are
//! listed — tags whose implicit role depends on ancestry (e.g. `<header>`/
//! `<footer>`'s banner/contentinfo role only applies outside `<article>`/
//! `<section>`) are deliberately left out to avoid false positives (plan/18).

use chromiumoxide::Page;

use crate::cli::WcagLevel;
use crate::wcag::types::{evaluate_or_fail, RuleMetadata, Severity, Violation};

pub(super) const REDUNDANT_ROLE_RULE: RuleMetadata = RuleMetadata {
    id: "4.1.2",
    name: "Name, Role, Value",
    level: WcagLevel::A,
    severity: Severity::Low,
    description:
        "An explicit role attribute must not merely restate the element's own implicit role",
    help_url: "https://www.w3.org/WAI/WCAG22/Understanding/name-role-value.html",
    axe_id: "redundant-role",
    tags: &["wcag2a", "wcag412", "cat.aria", "best-practice"],
};

// Fixed tag → implicit-role table, checked against `document.querySelectorAll('[role]')`.
// Selector uses the shared `__amsCssSelector` helper (never a bare tag name,
// see `js_helpers::CSS_SELECTOR_JS`) so two same-tag elements without an
// id/class remain distinguishable in the report.
//
// `role="list"` on a `<ul>`/`<ol>` styled `list-style: none` is not
// redundant: WebKit/VoiceOver drops the list semantics of such lists, and
// the explicit role is the established fix that restores them (#644).
const REDUNDANT_ROLE_JS: &str = r#"
  function keepsListSemantics(el) {
    return getComputedStyle(el).listStyleType !== 'none';
  }
  var MAP = [
    ['ul', keepsListSemantics, 'list'],
    ['ol', keepsListSemantics, 'list'],
  ];
  var findings = [];
  var elements = document.querySelectorAll('[role]');
  for (var i = 0; i < elements.length && __amsReal(findings) < 20; i++) {
    var el = elements[i];
    var tag = el.tagName.toLowerCase();
    var role = (el.getAttribute('role') || '').trim().toLowerCase();
    if (!role) continue;
    for (var j = 0; j < MAP.length; j++) {
      if (MAP[j][0] !== tag) continue;
      if (MAP[j][2] !== role) continue;
      if (!MAP[j][1](el)) continue;
      __amsPush(findings, el, { selector: __amsCssSelector(el), role: role }, 20);
      break;
    }
  }
  return findings;
"#;

pub async fn check_redundant_role_with_page(page: &Page) -> Vec<Violation> {
    let js = [
        "(function() {",
        crate::accessibility::js_helpers::CSS_SELECTOR_JS,
        REDUNDANT_ROLE_JS,
        "})()",
    ]
    .concat();

    let value = match evaluate_or_fail(page, &REDUNDANT_ROLE_RULE, &js).await {
        Ok(v) => v,
        Err(violations) => return violations,
    };

    let Some(issues) = value.as_array() else {
        return vec![];
    };

    issues
        .iter()
        .filter_map(|issue| {
            let selector = issue.get("selector")?.as_str()?.to_string();
            let role = issue.get("role")?.as_str()?.to_string();
            Some(
                Violation::new(
                    REDUNDANT_ROLE_RULE.id,
                    REDUNDANT_ROLE_RULE.name,
                    REDUNDANT_ROLE_RULE.level,
                    REDUNDANT_ROLE_RULE.severity,
                    format!(
                        "role=\"{role}\" merely restates this element's own implicit role — some browser/AT combinations announce it twice."
                    ),
                    &selector,
                )
                .with_selector(&selector)
                .with_rule_id(REDUNDANT_ROLE_RULE.axe_id)
                .with_tags(
                    REDUNDANT_ROLE_RULE
                        .tags
                        .iter()
                        .map(|s| s.to_string())
                        .collect(),
                )
                .with_fix("Remove the redundant role attribute; the native element already exposes this role.")
                .with_help_url(REDUNDANT_ROLE_RULE.help_url),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rule_metadata_is_low_severity_best_practice() {
        assert_eq!(REDUNDANT_ROLE_RULE.severity, Severity::Low);
        assert!(REDUNDANT_ROLE_RULE.tags.contains(&"best-practice"));
    }
}
