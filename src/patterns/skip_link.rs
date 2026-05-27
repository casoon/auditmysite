//! SkipLink pattern (issue #31).
//!
//! Detects skip-to-content links. Existing `check_bypass_blocks` validates
//! presence; this pattern adds positional validation (first focusable link)
//! and recognizes the well-formed case as a positive signal.

use crate::accessibility::AXTree;

use super::{JourneyCandidate, JourneyKind, PatternAnalysis, PatternConfidence, PatternKind};

fn collect_links<'a>(
    tree: &'a AXTree,
    node: &'a crate::accessibility::AXNode,
    out: &mut Vec<&'a crate::accessibility::AXNode>,
) {
    if matches!(node.role.as_deref(), Some("link")) {
        out.push(node);
    }
    for child_id in &node.child_ids {
        if let Some(child) = tree.get_node(child_id) {
            collect_links(tree, child, out);
        }
    }
}

const SKIP_KEYWORDS: &[&str] = &[
    "skip to content",
    "skip to main",
    "skip navigation",
    "skip to navigation",
    "zum inhalt",
    "zum hauptinhalt",
    "direkt zum inhalt",
    "navigation überspringen",
    "navigation ueberspringen",
];

pub fn detect(tree: &AXTree, out: &mut PatternAnalysis) {
    // Traverse from root in document order, collecting links.
    let mut links_in_order: Vec<&crate::accessibility::AXNode> = Vec::new();
    if let Some(root) = tree.root() {
        collect_links(tree, root, &mut links_in_order);
    }

    let candidates: Vec<_> = links_in_order
        .iter()
        .copied()
        .filter(|n| {
            n.name
                .as_deref()
                .map(|name| {
                    let lower = name.to_lowercase();
                    SKIP_KEYWORDS.iter().any(|k| lower.contains(k))
                })
                .unwrap_or(false)
        })
        .collect();

    if candidates.is_empty() {
        return;
    }

    let first_link_id = links_in_order.first().map(|n| n.node_id.as_str());
    let is_first = candidates
        .first()
        .map(|c| Some(c.node_id.as_str()) == first_link_id)
        .unwrap_or(false);

    let confidence = if is_first {
        PatternConfidence::Strong
    } else {
        PatternConfidence::Partial
    };
    let message = if is_first {
        format!(
            "Skip link recognized and correctly positioned as first focusable element (\"{}\").",
            candidates[0].name.as_deref().unwrap_or("")
        )
    } else {
        format!(
            "Skip link detected (\"{}\") but not the first focusable element — keyboard users may have to tab past other content first.",
            candidates[0].name.as_deref().unwrap_or("")
        )
    };
    out.add_recognized("SkipLink", message, confidence);

    // Emit journey candidates for interactive skip-link verification.
    for candidate in &candidates {
        if let Some(bid) = candidate.backend_dom_node_id {
            let candidate_confidence = if is_first { 0.9 } else { 0.7 };
            out.journey_candidates.push(JourneyCandidate {
                pattern_kind: PatternKind::SkipLink,
                trigger_backend_id: Some(bid),
                controlled_backend_id: None,
                confidence: candidate_confidence,
                required_journey: JourneyKind::SkipLinkActivate,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::{AXNode, AXTree};

    fn link(id: &str, name: &str) -> AXNode {
        AXNode {
            node_id: id.into(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some("link".into()),
            name: Some(name.into()),
            name_source: None,
            description: None,
            value: None,
            properties: vec![],
            child_ids: vec![],
            parent_id: Some("root".into()),
            backend_dom_node_id: None,
        }
    }

    fn root_with_children(child_ids: Vec<&str>) -> AXNode {
        AXNode {
            node_id: "root".into(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some("WebArea".into()),
            name: None,
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
    fn test_skip_link_first_strong() {
        let tree = AXTree::from_nodes(vec![
            root_with_children(vec!["1", "2"]),
            link("1", "Skip to main content"),
            link("2", "Home"),
        ]);
        let mut a = PatternAnalysis::default();
        detect(&tree, &mut a);
        assert_eq!(a.recognized[0].confidence, PatternConfidence::Strong);
    }

    #[test]
    fn test_skip_link_not_first_partial() {
        let tree = AXTree::from_nodes(vec![
            root_with_children(vec!["1", "2"]),
            link("1", "Home"),
            link("2", "Skip to main content"),
        ]);
        let mut a = PatternAnalysis::default();
        detect(&tree, &mut a);
        assert_eq!(a.recognized[0].confidence, PatternConfidence::Partial);
    }

    #[test]
    fn test_no_skip_link_no_recognition() {
        let tree = AXTree::from_nodes(vec![root_with_children(vec!["1"]), link("1", "Home")]);
        let mut a = PatternAnalysis::default();
        detect(&tree, &mut a);
        assert!(a.recognized.is_empty());
    }
}
