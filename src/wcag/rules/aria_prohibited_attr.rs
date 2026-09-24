//! WCAG 4.1.2 - ARIA Prohibited Attributes
//!
//! Validates that elements do not use ARIA attributes that are explicitly
//! prohibited for their role.

use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation};
use chromiumoxide::Page;
use tracing::warn;

/// Rule metadata for ARIA prohibited attributes
pub const RULE_META: RuleMetadata = RuleMetadata {
    id: "aria-prohibited-attr",
    name: "ARIA Prohibited Attributes",
    level: WcagLevel::A,
    severity: Severity::High,
    description: "ARIA attributes that are prohibited on a role must not be used",
    help_url: "https://www.w3.org/WAI/WCAG22/Understanding/name-role-value.html",
    axe_id: "aria-prohibited-attr",
    tags: &["wcag2a", "wcag412", "cat.aria"],
};

/// Roles and the ARIA attributes that are prohibited on them.
///
/// Checked entirely via `check_aria_prohibited_attr_with_page` (DOM-based),
/// not the AX tree: `aria-label`/`aria-labelledby` never surface as their own
/// AX property at all — CDP folds them directly into the computed accessible
/// `name` instead of exposing a raw "label"/"labelledby" property the way it
/// does for e.g. `aria-expanded` → `expanded` (#564). `aria-roledescription`
/// *is* exposed as an unprefixed `roledescription` property, but is kept
/// here too rather than split across two detection paths, since this table
/// is the single source of truth the JS selector below is generated from —
/// keep the two in sync when editing either.
const PROHIBITED_ATTRS: &[(&str, &[&str])] = &[
    ("presentation", &["aria-label", "aria-labelledby"]),
    ("none", &["aria-label", "aria-labelledby"]),
    (
        "generic",
        &["aria-label", "aria-labelledby", "aria-roledescription"],
    ),
    ("code", &["aria-label", "aria-labelledby"]),
    ("emphasis", &["aria-label", "aria-labelledby"]),
    ("strong", &["aria-label", "aria-labelledby"]),
    ("subscript", &["aria-label", "aria-labelledby"]),
    ("superscript", &["aria-label", "aria-labelledby"]),
    ("deletion", &["aria-label", "aria-labelledby"]),
    ("insertion", &["aria-label", "aria-labelledby"]),
];

/// DOM-based: elements without an explicit `role` attribute default to an
/// implicit `generic`/`none`-like role depending on tag, so `div`/`span`
/// carrying a prohibited attribute are checked directly by tag. Elements
/// with an *explicit* `role="..."` matching `PROHIBITED_ATTRS` are checked
/// by role. See `PROHIBITED_ATTRS`'s doc comment for why this can't be an
/// AX-tree check for `aria-label`/`aria-labelledby` (#564).
pub async fn check_aria_prohibited_attr_with_page(page: &Page) -> Vec<Violation> {
    // Built from PROHIBITED_ATTRS so it stays the single source of truth —
    // one `[role="x"][aria-y]` clause per (role, attribute) pair.
    let explicit_role_selector = PROHIBITED_ATTRS
        .iter()
        .flat_map(|(role, attrs)| {
            attrs
                .iter()
                .map(move |attr| format!("[role=\"{role}\"][{attr}]"))
        })
        .collect::<Vec<_>>()
        .join(",");

    let js = [
        "(function() {",
        crate::accessibility::js_helpers::CSS_SELECTOR_JS,
        r#"
        var issues = [];
        var implicitSelector = [
          'div:not([role])[aria-label]',
          'div:not([role])[aria-labelledby]',
          'div:not([role])[aria-roledescription]',
          'span:not([role])[aria-label]',
          'span:not([role])[aria-labelledby]',
          'span:not([role])[aria-roledescription]'
        ];
        var selector = implicitSelector.join(',') + ',__EXPLICIT_ROLE_SELECTOR__';
        var elements = document.querySelectorAll(selector);
        for (var i = 0; i < elements.length; i++) {
          var el = elements[i];
          var s = window.getComputedStyle(el);
          if (s.display === 'none' || s.visibility === 'hidden') continue;
          var attrs = [];
          ['aria-label', 'aria-labelledby', 'aria-roledescription'].forEach(function(attr) {
            if (el.hasAttribute(attr)) attrs.push(attr);
          });
          if (attrs.length === 0) continue;
          issues.push({
            selector: __amsCssSelector(el),
            snippet: el.outerHTML.substring(0, 200),
            attrs: attrs.join(', '),
            role: el.getAttribute('role') || el.tagName.toLowerCase()
          });
        }
        return issues;
        "#,
        "})()",
    ]
    .concat()
    .replace("__EXPLICIT_ROLE_SELECTOR__", &explicit_role_selector);

    let result = match page.evaluate(js.as_str()).await {
        Ok(r) => r,
        Err(e) => {
            warn!("aria-prohibited-attr DOM JS failed: {}", e);
            return vec![crate::wcag::technical_rule_failure_for(
                "aria-prohibited-attr",
                crate::cli::WcagLevel::A,
                "page_evaluation_failed",
            )];
        }
    };

    let Some(value) = result.value() else {
        return vec![crate::wcag::technical_rule_failure_for(
            "aria-prohibited-attr",
            crate::cli::WcagLevel::A,
            "missing_evaluation_value",
        )];
    };
    let Some(issues) = value.as_array() else {
        return vec![];
    };

    issues
        .iter()
        .filter_map(|issue| {
            let selector = issue.get("selector")?.as_str()?.to_string();
            let attrs = issue.get("attrs")?.as_str()?.to_string();
            let role = issue
                .get("role")
                .and_then(|v| v.as_str())
                .unwrap_or("generic");
            let mut violation = Violation::new(
                RULE_META.id,
                RULE_META.name,
                RULE_META.level,
                RULE_META.severity,
                format!("ARIA attribute(s) '{attrs}' are prohibited on role '{role}'"),
                &selector,
            )
            .with_selector(&selector)
            .with_rule_id(RULE_META.axe_id)
            .with_tags(RULE_META.tags.iter().map(|s| s.to_string()).collect())
            .with_fix("Remove the prohibited ARIA attribute or add a valid semantic role.")
            .with_help_url(RULE_META.help_url);

            if let Some(snippet) = issue.get("snippet").and_then(|v| v.as_str()) {
                violation = violation.with_html_snippet(snippet);
            }

            Some(violation)
        })
        .collect()
}

// No offline unit tests: the entire check now lives in DOM-evaluated JS
// (see PROHIBITED_ATTRS's doc comment for why — aria-label/aria-labelledby
// never surface as AX-tree properties, so there's no pure-Rust logic left to
// exercise without a live Chrome session). Covered by the detection corpus
// (tests/fixtures/detection_corpus/aria_attribute_validation.*, #556),
// verified against real Chrome via tests/detection_corpus_test.rs.
