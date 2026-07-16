//! WCAG 2.4.3 Focus Order
//!
//! If a Web page can be navigated sequentially and the navigation sequences
//! affect meaning or operation, focusable components receive focus in an
//! order that preserves meaning and operability.
//! Level A
//!
//! The positive-tabindex check is a DOM-level rule: `tabindex` is not exposed
//! as an AX property, so it must be read from the live DOM via CDP (#QA-030).
//! It used to be duplicated in `keyboard.rs` under 2.1.1 — positive tabindex
//! is a focus-*order* problem, so this file is now its single home.

use chromiumoxide::Page;
use tracing::warn;

use crate::accessibility::AXTree;
use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation, WcagResults};

pub const FOCUS_ORDER_RULE: RuleMetadata = RuleMetadata {
    id: "2.4.3",
    name: "Focus Order",
    level: WcagLevel::A,
    severity: Severity::High,
    description: "Focusable components receive focus in an order that preserves meaning",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/focus-order.html",
    axe_id: "focus-order-semantics",
    tags: &["wcag2a", "wcag243", "cat.keyboard"],
};

const POSITIVE_TABINDEX_CAP: usize = 250;

const POSITIVE_TABINDEX_BODY: &str = r#"
  var issues = [];
  var total = 0;
  var elems = document.querySelectorAll('[tabindex]');
  for (var i = 0; i < elems.length; i++) {
    var el = elems[i];
    if (el.tabIndex > 0) {
      total++;
      if (issues.length < CAP) {
        issues.push({ selector: __amsCssSelector(el), tabindex: el.tabIndex });
      }
    }
  }
  return { count: total, issues: issues };
"#;

/// Check for elements with a positive `tabindex`, which disrupts the natural
/// (DOM-order) focus sequence.
pub async fn check_positive_tabindex_with_page(page: &Page) -> Vec<Violation> {
    let js = [
        "(function() {",
        crate::accessibility::js_helpers::CSS_SELECTOR_JS,
        &POSITIVE_TABINDEX_BODY.replace("CAP", &POSITIVE_TABINDEX_CAP.to_string()),
        "})()",
    ]
    .concat();

    let result = match page.evaluate(js.as_str()).await {
        Ok(r) => r,
        Err(e) => {
            warn!("positive-tabindex JS failed: {}", e);
            return vec![crate::wcag::technical_rule_failure_for(
                "positive-tabindex",
                crate::cli::WcagLevel::A,
                "page_evaluation_failed",
            )];
        }
    };

    let val = match result.value() {
        Some(v) => v.clone(),
        None => {
            return vec![crate::wcag::technical_rule_failure_for(
                "positive-tabindex",
                crate::cli::WcagLevel::A,
                "missing_evaluation_value",
            )]
        }
    };

    let issues = match val.get("issues").and_then(|v| v.as_array()) {
        Some(arr) => arr.clone(),
        None => return vec![],
    };

    issues
        .iter()
        .filter_map(|issue| {
            let selector = issue.get("selector")?.as_str()?.to_string();
            let tabindex = issue.get("tabindex").and_then(|v| v.as_i64()).unwrap_or(0);

            Some(
                Violation::new(
                    FOCUS_ORDER_RULE.id,
                    FOCUS_ORDER_RULE.name,
                    FOCUS_ORDER_RULE.level,
                    Severity::High,
                    format!(
                        "Element has positive tabindex={} which disrupts natural focus order",
                        tabindex
                    ),
                    selector.clone(),
                )
                .with_selector(selector)
                .with_fix("Remove positive tabindex values. Use tabindex=\"0\" for natural order or tabindex=\"-1\" for programmatic focus only")
                .with_rule_id(FOCUS_ORDER_RULE.axe_id)
                .with_help_url(FOCUS_ORDER_RULE.help_url),
            )
        })
        .collect()
}

/// Check for focusable elements inside `aria-hidden` containers.
/// Tree-based: `hidden` is a real CDP AX property (unlike `tabindex`).
pub fn check_focus_order(tree: &AXTree) -> WcagResults {
    let mut results = WcagResults::new();

    for node in tree.iter() {
        if node.ignored {
            continue;
        }

        let is_aria_hidden = node.get_property_bool("hidden").unwrap_or(false);

        if is_aria_hidden && node.is_focusable() {
            let violation = Violation::new(
                FOCUS_ORDER_RULE.id,
                FOCUS_ORDER_RULE.name,
                FOCUS_ORDER_RULE.level,
                Severity::Critical,
                "Focusable element inside aria-hidden context",
                node.node_id.clone(),
            )
            .with_role(node.role.clone())
            .with_name(node.name.clone())
            .with_fix(
                "Either remove aria-hidden or make the element not focusable with tabindex=\"-1\"",
            )
            .with_help_url(FOCUS_ORDER_RULE.help_url);

            results.add_violation(violation);
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::{AXNode, AXProperty, AXTree, AXValue};

    fn node_with_hidden_and_focusable(
        id: &str,
        role: &str,
        hidden: bool,
        focusable: bool,
    ) -> AXNode {
        AXNode {
            node_id: id.to_string(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some(role.to_string()),
            name: Some("Test".to_string()),
            name_source: None,
            description: None,
            value: None,
            properties: vec![
                AXProperty {
                    name: "hidden".to_string(),
                    value: AXValue::Bool(hidden),
                },
                AXProperty {
                    name: "focusable".to_string(),
                    value: AXValue::Bool(focusable),
                },
            ],
            child_ids: vec![],
            parent_id: None,
            backend_dom_node_id: None,
        }
    }

    #[test]
    fn test_focusable_in_hidden_context_flagged() {
        let tree = AXTree::from_nodes(vec![node_with_hidden_and_focusable(
            "1", "button", true, true,
        )]);
        let results = check_focus_order(&tree);
        assert_eq!(results.violations.len(), 1);
    }

    #[test]
    fn test_focusable_not_hidden_passes() {
        let tree = AXTree::from_nodes(vec![node_with_hidden_and_focusable(
            "1", "button", false, true,
        )]);
        let results = check_focus_order(&tree);
        assert_eq!(results.violations.len(), 0);
    }

    #[test]
    fn test_hidden_not_focusable_passes() {
        let tree = AXTree::from_nodes(vec![node_with_hidden_and_focusable(
            "1", "button", true, false,
        )]);
        let results = check_focus_order(&tree);
        assert_eq!(results.violations.len(), 0);
    }
}
