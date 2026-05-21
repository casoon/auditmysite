//! WCAG 2.4.9 Link Purpose – Link Only (Level AAA)
//!
//! A mechanism is available to allow the purpose of each link to be identified
//! from link text alone. This is stricter than 2.4.4 in that contextual clues
//! from surrounding content are not sufficient — the link text must stand alone.

use crate::accessibility::AXTree;
use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation, WcagResults};

pub const LINK_PURPOSE_LINK_ONLY_RULE: RuleMetadata = RuleMetadata {
    id: "2.4.9",
    name: "Link Purpose (Link Only)",
    level: WcagLevel::AAA,
    severity: Severity::Medium,
    description: "The purpose of each link can be determined from the link text alone",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/link-purpose-link-only.html",
    axe_id: "link-name",
    tags: &["wcag2aaa", "wcag249", "cat.links"],
};

/// Generic/vague phrases that fail the link-only test (stricter than 2.4.4).
fn is_vague_link_text(text: &str) -> bool {
    let vague = [
        // English
        "click here",
        "click",
        "here",
        "read more",
        "more",
        "learn more",
        "info",
        "information",
        "details",
        "link",
        "this link",
        "go",
        "continue",
        "download",
        "view",
        "see more",
        "see all",
        "read",
        "start",
        "begin",
        "submit",
        "next",
        "previous",
        "page",
        "article",
        "post",
        // German
        "weiterlesen",
        "mehr erfahren",
        "mehr",
        "hier klicken",
        "klicken sie hier",
        "hier",
        "weiter",
        "alle anzeigen",
        "ansehen",
        "jetzt lesen",
        "öffnen",
        "seite",
        // Short/vague single words that provide no context
        "yes",
        "no",
        "ok",
        "okay",
        "open",
        "close",
        "show",
        "hide",
        "toggle",
        "buy",
        "get",
        "add",
        "remove",
        "edit",
        "update",
        "save",
        "send",
        "try",
        "see",
        "go",
        // Symbols
        "...",
        ">",
        ">>",
        "→",
        "»",
    ];

    let lower = text.to_lowercase();
    vague.iter().any(|&p| lower == p)
}

pub fn check_link_purpose_link_only(tree: &AXTree) -> WcagResults {
    let mut results = WcagResults::new();

    for node in tree.iter() {
        if node.ignored || node.role.as_deref() != Some("link") {
            continue;
        }

        results.nodes_checked += 1;
        let link_text = node.name.as_deref().unwrap_or("").trim();

        if link_text.is_empty() {
            results.add_violation(
                Violation::new(
                    LINK_PURPOSE_LINK_ONLY_RULE.id,
                    LINK_PURPOSE_LINK_ONLY_RULE.name,
                    LINK_PURPOSE_LINK_ONLY_RULE.level,
                    Severity::Critical,
                    "Link has no accessible text (fails 2.4.9)",
                    &node.node_id,
                )
                .with_role(node.role.clone())
                .with_fix("Add meaningful link text that describes the destination without relying on context.")
                .with_rule_id(LINK_PURPOSE_LINK_ONLY_RULE.axe_id)
                .with_help_url(LINK_PURPOSE_LINK_ONLY_RULE.help_url),
            );
        } else if is_vague_link_text(link_text) {
            results.add_violation(
                Violation::new(
                    LINK_PURPOSE_LINK_ONLY_RULE.id,
                    LINK_PURPOSE_LINK_ONLY_RULE.name,
                    LINK_PURPOSE_LINK_ONLY_RULE.level,
                    Severity::Medium,
                    format!(
                        "Link text '{}' is too vague to identify the link purpose without context (WCAG 2.4.9 requires link text alone to be sufficient).",
                        link_text
                    ),
                    &node.node_id,
                )
                .with_role(node.role.clone())
                .with_name(node.name.clone())
                .with_fix("Use descriptive link text that explains the destination or purpose without requiring surrounding context.")
                .with_rule_id(LINK_PURPOSE_LINK_ONLY_RULE.axe_id)
                .with_help_url(LINK_PURPOSE_LINK_ONLY_RULE.help_url),
            );
        } else {
            results.passes += 1;
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::{AXNode, AXTree};

    fn link(id: &str, name: Option<&str>) -> AXNode {
        AXNode {
            node_id: id.to_string(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some("link".to_string()),
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

    #[test]
    fn test_empty_link_fails() {
        let tree = AXTree::from_nodes(vec![link("1", None)]);
        let r = check_link_purpose_link_only(&tree);
        assert!(!r.violations.is_empty());
        assert!(r.violations[0].severity == Severity::Critical);
    }

    #[test]
    fn test_vague_text_fails() {
        let tree = AXTree::from_nodes(vec![link("1", Some("more"))]);
        let r = check_link_purpose_link_only(&tree);
        assert!(!r.violations.is_empty());
    }

    #[test]
    fn test_descriptive_text_passes() {
        let tree = AXTree::from_nodes(vec![link("1", Some("View WCAG 2.1 guidelines"))]);
        let r = check_link_purpose_link_only(&tree);
        assert!(r.violations.is_empty());
    }
}
