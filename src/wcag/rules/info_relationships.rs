//! WCAG 1.3.1 Info and Relationships
//!
//! Information, structure, and relationships conveyed through presentation
//! can be programmatically determined or are available in text.
//! Level A
//!
//! Only the presentational-container check lives here. Table and list
//! structure are checked by the shared rules (`tables/*`, `lists/*`, see
//! `wcag::shared`), radio/checkbox grouping by `forms/group-missing` (#693).

use chromiumoxide::Page;
use tracing::warn;

use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation};

/// Rule metadata for role=presentation/none hiding semantic descendants.
pub(super) const PRESENTATION_SEMANTIC_CHILDREN_RULE: RuleMetadata = RuleMetadata {
    id: "1.3.1",
    name: "Info and Relationships",
    level: WcagLevel::A,
    severity: Severity::Medium,
    description: "Presentational containers must not hide semantic child structure",
    help_url: "https://www.w3.org/WAI/WCAG22/Techniques/failures/F92",
    axe_id: "presentation-semantic-children",
    tags: &["wcag2a", "wcag131", "cat.semantics"],
};

/// DOM check for presentational containers that include semantic descendants.
pub async fn check_presentation_semantic_children_with_page(page: &Page) -> Vec<Violation> {
    let js = [
        "(function() {",
        crate::accessibility::js_helpers::CSS_SELECTOR_JS,
        r#"
        var issues = [];
        var semanticSelector = [
          'h1,h2,h3,h4,h5,h6',
          'main,nav,header,footer,article,aside,section[aria-label],section[aria-labelledby]',
          'ul,ol,dl,table,th',
          'button,input:not([type="hidden"]),select,textarea,a[href]',
          '[role]:not([role="presentation"]):not([role="none"]):not([role="generic"])'
        ].join(',');
        var containers = document.querySelectorAll('[role="presentation"], [role="none"]');
        for (var i = 0; i < containers.length; i++) {
          var el = containers[i];
          if (el.hasAttribute('hidden') || el.getAttribute('aria-hidden') === 'true') continue;
          var style = window.getComputedStyle(el);
          if (style && (style.display === 'none' || style.visibility === 'hidden')) continue;

          var child = el.querySelector(semanticSelector);
          if (!child) continue;
          issues.push({
            selector: __amsCssSelector(el),
            child_selector: __amsCssSelector(child),
            child_role: child.getAttribute('role') || child.tagName.toLowerCase(),
            snippet: el.outerHTML.substring(0, 200)
          });
        }
        return issues;
        "#,
        "})()",
    ]
    .concat();

    let result = match page.evaluate(js.as_str()).await {
        Ok(r) => r,
        Err(e) => {
            warn!("presentation semantic children DOM JS failed: {}", e);
            return vec![crate::wcag::technical_rule_failure_for(
                "presentation-semantic-children",
                crate::cli::WcagLevel::A,
                "page_evaluation_failed",
            )];
        }
    };

    let Some(value) = result.value() else {
        return vec![crate::wcag::technical_rule_failure_for(
            "presentation-semantic-children",
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
            let child_selector = issue
                .get("child_selector")
                .and_then(|v| v.as_str())
                .unwrap_or("semantic descendant");
            let child_role = issue
                .get("child_role")
                .and_then(|v| v.as_str())
                .unwrap_or("semantic role");
            let mut violation = Violation::new(
                PRESENTATION_SEMANTIC_CHILDREN_RULE.id,
                PRESENTATION_SEMANTIC_CHILDREN_RULE.name,
                PRESENTATION_SEMANTIC_CHILDREN_RULE.level,
                PRESENTATION_SEMANTIC_CHILDREN_RULE.severity,
                format!(
                    "Element with role=\"presentation\"/\"none\" contains semantic child {child_role} ({child_selector})"
                ),
                &selector,
            )
            .with_selector(&selector)
            .with_rule_id(PRESENTATION_SEMANTIC_CHILDREN_RULE.axe_id)
            .with_tags(
                PRESENTATION_SEMANTIC_CHILDREN_RULE
                    .tags
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
            )
            .with_fix("Remove role=\"presentation\"/\"none\" from the container, or remove semantic roles from purely decorative descendants.")
            .with_help_url(PRESENTATION_SEMANTIC_CHILDREN_RULE.help_url);

            if let Some(snippet) = issue.get("snippet").and_then(|v| v.as_str()) {
                violation = violation.with_html_snippet(snippet);
            }

            Some(violation)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_presentation_semantic_children_metadata() {
        assert_eq!(PRESENTATION_SEMANTIC_CHILDREN_RULE.id, "1.3.1");
        assert_eq!(
            PRESENTATION_SEMANTIC_CHILDREN_RULE.axe_id,
            "presentation-semantic-children"
        );
        assert!(PRESENTATION_SEMANTIC_CHILDREN_RULE
            .tags
            .contains(&"wcag131"));
    }
}
