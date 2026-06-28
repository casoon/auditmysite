//! MainNavigation pattern (issue #27).
//!
//! Detects semantic main navigation: `<nav>` or `role="navigation"`, with
//! accessible name when multiple nav landmarks exist, and native link children.

use crate::accessibility::AXTree;

use super::{PatternAnalysis, PatternConfidence};

pub fn detect(tree: &AXTree, out: &mut PatternAnalysis) {
    let nav_nodes = tree.nodes_with_role("navigation");
    if nav_nodes.is_empty() {
        return;
    }

    let multiple = nav_nodes.len() > 1;
    let mut named = 0usize;
    let mut with_links = 0usize;

    for nav in &nav_nodes {
        let has_name = nav.name.as_deref().is_some_and(|n| !n.trim().is_empty());
        if has_name {
            named += 1;
        }

        // Count link descendants
        let link_count = nav
            .child_ids
            .iter()
            .filter_map(|id| tree.get_node(id))
            .filter(|child| matches!(child.role.as_deref(), Some("link")))
            .count();
        if link_count > 0 {
            with_links += 1;
        }
    }

    // When multiple navs exist, each one needs a name — without it, screen
    // readers cannot distinguish them. Flag the missing-name case as info.
    if multiple && named < nav_nodes.len() {
        let count_missing = nav_nodes.len() - named;
        out.add_recognized(
            "MainNavigation",
            format!(
                "{} navigation {} found, but {} have no accessible name. Screen reader users cannot distinguish them.",
                nav_nodes.len(),
                if nav_nodes.len() == 1 { "landmark" } else { "landmarks" },
                count_missing
            ),
            PatternConfidence::Partial,
        );
        return;
    }

    if with_links == 0 {
        // Nav landmark without link descendants — likely empty or malformed.
        out.add_recognized(
            "MainNavigation",
            format!(
                "{} navigation {} found, but none contain link children. Pattern is structurally incomplete.",
                nav_nodes.len(),
                if nav_nodes.len() == 1 { "landmark" } else { "landmarks" }
            ),
            PatternConfidence::Partial,
        );
        return;
    }

    out.add_recognized(
        "MainNavigation",
        format!(
            "Semantic main navigation recognized ({} {} with native links). Good foundation for keyboard and screen reader use.",
            nav_nodes.len(),
            if nav_nodes.len() == 1 { "landmark" } else { "landmarks" }
        ),
        PatternConfidence::Strong,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::{AXNode, AXTree};

    fn node(id: &str, role: &str, name: Option<&str>, child_ids: Vec<&str>) -> AXNode {
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
            child_ids: child_ids.into_iter().map(String::from).collect(),
            parent_id: None,
            backend_dom_node_id: None,
        }
    }

    #[test]
    fn test_strong_when_nav_with_links() {
        let tree = AXTree::from_nodes(vec![
            node("1", "navigation", Some("Main"), vec!["2"]),
            node("2", "link", Some("Home"), vec![]),
        ]);
        let mut a = PatternAnalysis::default();
        detect(&tree, &mut a);
        assert_eq!(a.recognized.len(), 1);
        assert_eq!(a.recognized[0].confidence, PatternConfidence::Strong);
    }

    #[test]
    fn test_partial_when_multiple_navs_unnamed() {
        let tree = AXTree::from_nodes(vec![
            node("1", "navigation", None, vec!["3"]),
            node("2", "navigation", None, vec!["4"]),
            node("3", "link", Some("Home"), vec![]),
            node("4", "link", Some("Imprint"), vec![]),
        ]);
        let mut a = PatternAnalysis::default();
        detect(&tree, &mut a);
        assert_eq!(a.recognized.len(), 1);
        assert_eq!(a.recognized[0].confidence, PatternConfidence::Partial);
    }

    #[test]
    fn test_nothing_when_no_nav() {
        let tree = AXTree::from_nodes(vec![node("1", "WebArea", None, vec![])]);
        let mut a = PatternAnalysis::default();
        detect(&tree, &mut a);
        assert!(a.recognized.is_empty());
    }
}
