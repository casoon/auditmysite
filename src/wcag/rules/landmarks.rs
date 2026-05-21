//! WCAG 2.4.1 / 1.3.1 - Landmark Presence
//!
//! Checks that pages contain the expected landmark regions (main, navigation,
//! banner, contentinfo).
//!
//! Taxonomy split (see issue #242):
//! - Missing `main` → WCAG 2.4.1 (axe `landmark-one-main` convention — main
//!   is the canonical skip target).
//! - Missing nav / banner / contentinfo → WCAG 1.3.1 (structural).
//!
//! Duplicate / nested / unique-name checks live in `landmark_granular.rs`
//! under WCAG 1.3.1; the skip-link check lives there under WCAG 2.4.1.

use crate::accessibility::AXTree;
use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation, WcagResults};

/// Rule metadata for landmark region checks (2.4.1)
pub const RULE_META: RuleMetadata = RuleMetadata {
    id: "2.4.1",
    name: "Landmark Regions",
    level: WcagLevel::A,
    severity: Severity::Medium,
    description: "Pages should use landmark regions to help users navigate content",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/bypass-blocks.html",
    axe_id: "landmark-one-main",
    tags: &["wcag2a", "wcag241", "cat.semantics"],
};

/// WCAG criterion + help URL for structural landmark-presence findings
/// (missing nav / banner / contentinfo). 1.3.1 = Info and Relationships.
const STRUCTURE_CRITERION: &str = "1.3.1";
const STRUCTURE_NAME: &str = "Landmark Regions";
const STRUCTURE_HELP_URL: &str =
    "https://www.w3.org/WAI/WCAG21/Understanding/info-and-relationships.html";

/// Check landmark structure across the accessibility tree
///
/// # Arguments
/// * `tree` - The accessibility tree to check
///
/// # Returns
/// Results with violations for missing or improperly labeled landmark regions
pub fn check_landmarks(tree: &AXTree) -> WcagResults {
    let mut results = WcagResults::new();
    results.nodes_checked += tree.len();

    let main_nodes = tree.nodes_with_role("main");
    let nav_nodes = tree.nodes_with_role("navigation");
    let banner_nodes = tree.nodes_with_role("banner");
    let contentinfo_nodes = tree.nodes_with_role("contentinfo");

    // --- Missing main landmark — WCAG 2.4.1 (axe `landmark-one-main`) ---
    // Duplicate-main is covered by `check_landmark_no_duplicate_main` under 1.3.1.
    if main_nodes.is_empty() {
        results.add_violation(
            Violation::new(
                RULE_META.id,
                RULE_META.name,
                WcagLevel::A,
                Severity::Medium,
                "Page has no main landmark",
                "root",
            )
            .with_fix("Add a <main> element or an element with role=\"main\"")
            .with_rule_id(RULE_META.axe_id)
            .with_help_url(RULE_META.help_url),
        );
    } else {
        results.passes += 1;
    }

    // --- Missing nav / banner / contentinfo — WCAG 1.3.1 (structural) ---
    if nav_nodes.is_empty() {
        results.add_violation(missing_landmark_violation(
            "navigation",
            "Page has no navigation landmark",
            "Add a <nav> element or an element with role=\"navigation\"",
        ));
    } else {
        results.passes += 1;
    }

    if banner_nodes.is_empty() {
        results.add_violation(missing_landmark_violation(
            "banner",
            "Page has no banner landmark",
            "Add a <header> element at the top level or role=\"banner\"",
        ));
    } else {
        results.passes += 1;
    }

    if contentinfo_nodes.is_empty() {
        results.add_violation(missing_landmark_violation(
            "contentinfo",
            "Page has no contentinfo landmark",
            "Add a <footer> element at the top level or role=\"contentinfo\"",
        ));
    } else {
        results.passes += 1;
    }

    results
}

/// Build a structural "missing landmark" violation under WCAG 1.3.1.
fn missing_landmark_violation(
    _role: &'static str,
    message: &'static str,
    fix: &'static str,
) -> Violation {
    Violation::new(
        STRUCTURE_CRITERION,
        STRUCTURE_NAME,
        WcagLevel::A,
        Severity::Low,
        message,
        "root",
    )
    .with_fix(fix)
    .with_help_url(STRUCTURE_HELP_URL)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::{AXNode, AXTree};

    fn make_node(id: &str, role: &str, name: Option<&str>) -> AXNode {
        AXNode {
            node_id: id.to_string(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some(role.to_string()),
            name: name.map(String::from),
            name_source: None,
            description: None,
            value: None,
            properties: vec![],
            child_ids: vec![],
            parent_id: None,
            backend_dom_node_id: None,
        }
    }

    fn make_full_landmark_tree() -> AXTree {
        AXTree::from_nodes(vec![
            make_node("1", "WebArea", Some("Test Page")),
            make_node("2", "banner", Some("Site Header")),
            make_node("3", "navigation", Some("Main Nav")),
            make_node("4", "main", Some("Main Content")),
            make_node("5", "contentinfo", Some("Site Footer")),
        ])
    }

    #[test]
    fn test_full_landmark_set_passes() {
        let tree = make_full_landmark_tree();
        let results = check_landmarks(&tree);
        let landmark_violations: Vec<_> = results
            .violations
            .iter()
            .filter(|v| {
                v.message.contains("no main")
                    || v.message.contains("no navigation")
                    || v.message.contains("no banner")
                    || v.message.contains("no contentinfo")
            })
            .collect();
        assert!(
            landmark_violations.is_empty(),
            "Complete landmark set should not produce missing-landmark violations"
        );
    }

    #[test]
    fn test_missing_main_landmark_flagged() {
        let nodes = vec![
            make_node("1", "WebArea", Some("Test Page")),
            make_node("2", "banner", Some("Header")),
        ];
        let tree = AXTree::from_nodes(nodes);
        let results = check_landmarks(&tree);
        assert!(
            results
                .violations
                .iter()
                .any(|v| v.message.contains("no main landmark")),
            "Missing main landmark should be flagged"
        );
    }

    #[test]
    fn test_duplicate_main_not_flagged_here() {
        // Duplicate-main is now exclusively covered by
        // `landmark_granular::check_landmark_no_duplicate_main` under WCAG 1.3.1.
        // `check_landmarks` must not double-report it under 2.4.1 (issue #242).
        let nodes = vec![
            make_node("1", "WebArea", Some("Test")),
            make_node("2", "main", Some("Main 1")),
            make_node("3", "main", Some("Main 2")),
        ];
        let tree = AXTree::from_nodes(nodes);
        let results = check_landmarks(&tree);
        assert!(
            !results
                .violations
                .iter()
                .any(|v| v.message.contains("main landmarks")),
            "Duplicate-main must not be flagged by check_landmarks — \
             it is covered by check_landmark_no_duplicate_main under 1.3.1"
        );
    }

    // ── Regression: WCAG criterion mapping (issue #242) ───────────────────

    #[test]
    fn missing_main_filed_under_241() {
        let tree = AXTree::from_nodes(vec![make_node("1", "WebArea", Some("Test"))]);
        let results = check_landmarks(&tree);
        let missing_main = results
            .violations
            .iter()
            .find(|v| v.message.contains("no main landmark"))
            .expect("missing-main violation expected");
        assert_eq!(
            missing_main.rule, "2.4.1",
            "missing-main is the canonical bypass-blocks target (axe landmark-one-main)"
        );
    }

    #[test]
    fn missing_nav_banner_contentinfo_filed_under_131() {
        let tree = AXTree::from_nodes(vec![
            make_node("1", "WebArea", Some("Test")),
            make_node("2", "main", Some("Content")),
        ]);
        let results = check_landmarks(&tree);
        for needle in ["no navigation", "no banner", "no contentinfo"] {
            let v = results
                .violations
                .iter()
                .find(|v| v.message.contains(needle))
                .unwrap_or_else(|| panic!("expected violation containing '{}'", needle));
            assert_eq!(
                v.rule, "1.3.1",
                "missing-{needle} is structural and must be filed under WCAG 1.3.1, \
                 not aggregated under 2.4.1 (issue #242)"
            );
        }
    }

    #[test]
    fn multi_landmark_naming_not_flagged_here() {
        // Multi-landmark uniqueness is covered by
        // `landmark_granular::check_landmark_unique` under WCAG 1.3.1.
        // `check_landmarks` must not double-report it (formerly under 1.3.6).
        let tree = AXTree::from_nodes(vec![
            make_node("1", "WebArea", Some("Test")),
            make_node("2", "main", Some("Content")),
            make_node("3", "banner", Some("Header")),
            make_node("4", "contentinfo", Some("Footer")),
            make_node("5", "navigation", None),
            make_node("6", "navigation", None),
        ]);
        let results = check_landmarks(&tree);
        assert!(
            !results
                .violations
                .iter()
                .any(|v| v.message.contains("no accessible name")),
            "Multi-landmark naming must not be flagged by check_landmarks — \
             it is covered by check_landmark_unique under 1.3.1"
        );
    }
}
