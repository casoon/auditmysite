//! Extended landmark region rules
//!
//! Supplements `landmarks.rs` (which covers presence of main/banner/nav/contentinfo
//! and multi-nav labelling) with granular axe-core landmark rules:
//!
//! - `landmark-no-duplicate-banner`:          max one banner landmark
//! - `landmark-no-duplicate-contentinfo`:     max one contentinfo landmark
//! - `landmark-banner-is-top-level`:          banner must not be nested inside another landmark
//! - `landmark-contentinfo-is-top-level`:     contentinfo must not be nested inside another landmark
//! - `landmark-main-is-top-level`:            main must not be nested inside another landmark
//! - `landmark-unique`:                       multiple same-type landmarks need distinct names
//! - `skip-link`:                             page should have a skip-navigation link

use std::collections::HashMap;

use crate::accessibility::AXTree;
use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation, WcagResults};

// ── Rule metadata ──────────────────────────────────────────────────────────────

pub const RULE_NO_DUPLICATE_BANNER: RuleMetadata = RuleMetadata {
    id: "2.4.1",
    name: "No Duplicate Banner",
    level: WcagLevel::A,
    severity: Severity::Medium,
    description: "The page must not have more than one banner landmark",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/bypass-blocks.html",
    axe_id: "landmark-no-duplicate-banner",
    tags: &["wcag2a", "wcag241", "cat.semantics"],
};

pub const RULE_NO_DUPLICATE_CONTENTINFO: RuleMetadata = RuleMetadata {
    id: "2.4.1",
    name: "No Duplicate Contentinfo",
    level: WcagLevel::A,
    severity: Severity::Medium,
    description: "The page must not have more than one contentinfo landmark",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/bypass-blocks.html",
    axe_id: "landmark-no-duplicate-contentinfo",
    tags: &["wcag2a", "wcag241", "cat.semantics"],
};

pub const RULE_BANNER_IS_TOP_LEVEL: RuleMetadata = RuleMetadata {
    id: "2.4.1",
    name: "Banner Is Top Level",
    level: WcagLevel::A,
    severity: Severity::Medium,
    description: "The banner landmark must not be contained within another landmark",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/bypass-blocks.html",
    axe_id: "landmark-banner-is-top-level",
    tags: &["wcag2a", "wcag241", "cat.semantics"],
};

pub const RULE_CONTENTINFO_IS_TOP_LEVEL: RuleMetadata = RuleMetadata {
    id: "2.4.1",
    name: "Contentinfo Is Top Level",
    level: WcagLevel::A,
    severity: Severity::Medium,
    description: "The contentinfo landmark must not be contained within another landmark",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/bypass-blocks.html",
    axe_id: "landmark-contentinfo-is-top-level",
    tags: &["wcag2a", "wcag241", "cat.semantics"],
};

pub const RULE_MAIN_IS_TOP_LEVEL: RuleMetadata = RuleMetadata {
    id: "2.4.1",
    name: "Main Is Top Level",
    level: WcagLevel::A,
    severity: Severity::Medium,
    description: "The main landmark must not be contained within another landmark",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/bypass-blocks.html",
    axe_id: "landmark-main-is-top-level",
    tags: &["wcag2a", "wcag241", "cat.semantics"],
};

pub const RULE_LANDMARK_UNIQUE: RuleMetadata = RuleMetadata {
    id: "2.4.1",
    name: "Landmark Unique",
    level: WcagLevel::A,
    severity: Severity::Medium,
    description: "Landmark roles should be unique, or distinguishable by accessible name when multiple instances exist",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/bypass-blocks.html",
    axe_id: "landmark-unique",
    tags: &["wcag2a", "wcag241", "cat.semantics"],
};

pub const RULE_SKIP_LINK: RuleMetadata = RuleMetadata {
    id: "2.4.1",
    name: "Skip Link",
    level: WcagLevel::A,
    severity: Severity::Medium,
    description: "The page should provide a mechanism to skip repeated navigation blocks",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/bypass-blocks.html",
    axe_id: "skip-link",
    tags: &["wcag2a", "wcag241", "cat.keyboard"],
};

// ── Landmark roles ─────────────────────────────────────────────────────────────

const LANDMARK_ROLES: &[&str] = &[
    "banner",
    "complementary",
    "contentinfo",
    "form",
    "main",
    "navigation",
    "region",
    "search",
];

// ── Public check function ──────────────────────────────────────────────────────

/// Run extended landmark checks.
pub fn check_landmark_extended(tree: &AXTree) -> WcagResults {
    let mut results = WcagResults::new();

    // Collect all landmark nodes by role
    let mut by_role: HashMap<&str, Vec<&crate::accessibility::AXNode>> = HashMap::new();
    for role in LANDMARK_ROLES {
        let nodes = tree.nodes_with_role(role);
        if !nodes.is_empty() {
            by_role.insert(role, nodes);
        }
    }

    // ── landmark-no-duplicate-banner ──────────────────────────────────────
    if let Some(nodes) = by_role.get("banner") {
        if nodes.len() > 1 {
            results.add_violation(
                Violation::new(
                    RULE_NO_DUPLICATE_BANNER.id,
                    RULE_NO_DUPLICATE_BANNER.name,
                    RULE_NO_DUPLICATE_BANNER.level,
                    RULE_NO_DUPLICATE_BANNER.severity,
                    format!(
                        "Page has {} banner landmarks; only one is permitted",
                        nodes.len()
                    ),
                    &nodes[0].node_id,
                )
                .with_fix(
                    "Ensure the page has at most one <header> / role=\"banner\" at the top level",
                )
                .with_rule_id(RULE_NO_DUPLICATE_BANNER.axe_id)
                .with_help_url(RULE_NO_DUPLICATE_BANNER.help_url),
            );
        } else {
            results.passes += 1;
        }
    }

    // ── landmark-no-duplicate-contentinfo ─────────────────────────────────
    if let Some(nodes) = by_role.get("contentinfo") {
        if nodes.len() > 1 {
            results.add_violation(
                Violation::new(
                    RULE_NO_DUPLICATE_CONTENTINFO.id,
                    RULE_NO_DUPLICATE_CONTENTINFO.name,
                    RULE_NO_DUPLICATE_CONTENTINFO.level,
                    RULE_NO_DUPLICATE_CONTENTINFO.severity,
                    format!(
                        "Page has {} contentinfo landmarks; only one is permitted",
                        nodes.len()
                    ),
                    &nodes[0].node_id,
                )
                .with_fix("Ensure the page has at most one <footer> / role=\"contentinfo\" at the top level")
                .with_rule_id(RULE_NO_DUPLICATE_CONTENTINFO.axe_id).with_help_url(RULE_NO_DUPLICATE_CONTENTINFO.help_url),
            );
        } else {
            results.passes += 1;
        }
    }

    // ── landmark-banner-is-top-level ──────────────────────────────────────
    if let Some(nodes) = by_role.get("banner") {
        for node in nodes {
            if is_inside_landmark(node, tree) {
                // Per HTML spec, a <header> nested inside <main>/<article>/
                // <aside>/<nav>/<section> does NOT have an implicit banner
                // role. If Chrome still reports role="banner" here (and the
                // author did not set role="banner" explicitly), skip.
                if is_header_in_sectioning_content(node, tree) {
                    results.passes += 1;
                    continue;
                }
                results.add_violation(
                    Violation::new(
                        RULE_BANNER_IS_TOP_LEVEL.id,
                        RULE_BANNER_IS_TOP_LEVEL.name,
                        RULE_BANNER_IS_TOP_LEVEL.level,
                        RULE_BANNER_IS_TOP_LEVEL.severity,
                        "banner landmark is nested inside another landmark",
                        &node.node_id,
                    )
                    .with_role(node.role.clone())
                    .with_fix("Move the <header> / role=\"banner\" element to the top level, outside all other landmarks")
                    .with_rule_id(RULE_BANNER_IS_TOP_LEVEL.axe_id).with_help_url(RULE_BANNER_IS_TOP_LEVEL.help_url),
                );
            } else {
                results.passes += 1;
            }
        }
    }

    // ── landmark-contentinfo-is-top-level ─────────────────────────────────
    if let Some(nodes) = by_role.get("contentinfo") {
        for node in nodes {
            if is_inside_landmark(node, tree) {
                // Mirror of the banner case: a <footer> inside sectioning
                // content has no implicit contentinfo role per HTML spec.
                if is_footer_in_sectioning_content(node, tree) {
                    results.passes += 1;
                    continue;
                }
                results.add_violation(
                    Violation::new(
                        RULE_CONTENTINFO_IS_TOP_LEVEL.id,
                        RULE_CONTENTINFO_IS_TOP_LEVEL.name,
                        RULE_CONTENTINFO_IS_TOP_LEVEL.level,
                        RULE_CONTENTINFO_IS_TOP_LEVEL.severity,
                        "contentinfo landmark is nested inside another landmark",
                        &node.node_id,
                    )
                    .with_role(node.role.clone())
                    .with_fix("Move the <footer> / role=\"contentinfo\" element to the top level, outside all other landmarks")
                    .with_rule_id(RULE_CONTENTINFO_IS_TOP_LEVEL.axe_id).with_help_url(RULE_CONTENTINFO_IS_TOP_LEVEL.help_url),
                );
            } else {
                results.passes += 1;
            }
        }
    }

    // ── landmark-main-is-top-level ────────────────────────────────────────
    if let Some(nodes) = by_role.get("main") {
        for node in nodes {
            if is_inside_landmark(node, tree) {
                results.add_violation(
                    Violation::new(
                        RULE_MAIN_IS_TOP_LEVEL.id,
                        RULE_MAIN_IS_TOP_LEVEL.name,
                        RULE_MAIN_IS_TOP_LEVEL.level,
                        RULE_MAIN_IS_TOP_LEVEL.severity,
                        "main landmark is nested inside another landmark",
                        &node.node_id,
                    )
                    .with_role(node.role.clone())
                    .with_fix("Move the <main> / role=\"main\" element to the top level, not inside other landmarks")
                    .with_rule_id(RULE_MAIN_IS_TOP_LEVEL.axe_id).with_help_url(RULE_MAIN_IS_TOP_LEVEL.help_url),
                );
            } else {
                results.passes += 1;
            }
        }
    }

    // ── landmark-unique ───────────────────────────────────────────────────
    // Flag landmark roles that appear multiple times with the same (or absent) name.
    for (role, nodes) in &by_role {
        if nodes.len() < 2 {
            continue;
        }
        // Group nodes by accessible name (None counts as empty string)
        let mut name_counts: HashMap<String, usize> = HashMap::new();
        for node in nodes {
            let key = node.name.clone().unwrap_or_default().to_lowercase();
            *name_counts.entry(key).or_insert(0) += 1;
        }
        for node in nodes {
            let key = node.name.clone().unwrap_or_default().to_lowercase();
            if name_counts.get(&key).copied().unwrap_or(0) > 1 {
                results.add_violation(
                    Violation::new(
                        RULE_LANDMARK_UNIQUE.id,
                        RULE_LANDMARK_UNIQUE.name,
                        RULE_LANDMARK_UNIQUE.level,
                        RULE_LANDMARK_UNIQUE.severity,
                        format!(
                            "Multiple '{}' landmarks share the same accessible name; they cannot be distinguished",
                            role
                        ),
                        &node.node_id,
                    )
                    .with_role(node.role.clone())
                    .with_fix(format!(
                        "Add a unique aria-label to each '{}' landmark so they can be told apart",
                        role
                    ))
                    .with_rule_id(RULE_LANDMARK_UNIQUE.axe_id).with_help_url(RULE_LANDMARK_UNIQUE.help_url),
                );
            } else {
                results.passes += 1;
            }
        }
    }

    // ── skip-link ─────────────────────────────────────────────────────────
    // Heuristic: look for a link near the top of the tree whose name or target
    // suggests it is a skip-navigation / skip-to-content link.
    let has_skip_link = tree.iter().any(|n| {
        if n.ignored {
            return false;
        }
        if !matches!(n.role.as_deref(), Some("link")) {
            return false;
        }
        let name = n.name.as_deref().unwrap_or("").to_lowercase();
        let href = n
            .get_property_str("url")
            .or_else(|| n.get_property_str("href"))
            .unwrap_or("")
            .to_lowercase();

        let name_hints = [
            "skip",
            "überspringen",
            "zum inhalt",
            "zum hauptinhalt",
            "direkt zum",
            "navigation überspringen",
        ];
        let href_hints = [
            "#main",
            "#content",
            "#inhalt",
            "#skip",
            "#maincontent",
            "#hauptinhalt",
        ];

        name_hints.iter().any(|h| name.contains(h)) || href_hints.iter().any(|h| href.contains(h))
    });

    if has_skip_link {
        results.passes += 1;
    } else {
        // Only flag this if the page actually has a navigation landmark
        // (pages without nav don't need a skip link).
        let has_nav = by_role.contains_key("navigation");
        if has_nav {
            results.add_violation(
                Violation::new(
                    RULE_SKIP_LINK.id,
                    RULE_SKIP_LINK.name,
                    RULE_SKIP_LINK.level,
                    RULE_SKIP_LINK.severity,
                    "Page has navigation landmark(s) but no skip-navigation link was found",
                    "root",
                )
                .with_fix("Add a visually hidden or visible link at the top of the page pointing to the main content anchor (e.g. <a href=\"#main\">Skip to main content</a>)")
                .with_rule_id(RULE_SKIP_LINK.axe_id).with_help_url(RULE_SKIP_LINK.help_url),
            );
        }
    }

    results
}

// ── Private helpers ───────────────────────────────────────────────────────────

/// Returns true when the node is a `<header>` element that is nested inside
/// HTML sectioning content (`<main>`, `<article>`, `<aside>`, `<nav>`,
/// `<section>`). Per HTML spec, such a `<header>` does NOT carry an implicit
/// banner role — so when Chrome reports `role="banner"` for it, treat that as
/// a browser quirk rather than an author error.
fn is_header_in_sectioning_content(node: &crate::accessibility::AXNode, tree: &AXTree) -> bool {
    if node.get_property_str("htmlTag").map(|t| t.to_uppercase()) != Some("HEADER".to_string()) {
        return false;
    }
    has_sectioning_ancestor(node, tree)
}

/// Mirror of `is_header_in_sectioning_content` for `<footer>`.
fn is_footer_in_sectioning_content(node: &crate::accessibility::AXNode, tree: &AXTree) -> bool {
    if node.get_property_str("htmlTag").map(|t| t.to_uppercase()) != Some("FOOTER".to_string()) {
        return false;
    }
    has_sectioning_ancestor(node, tree)
}

fn has_sectioning_ancestor(node: &crate::accessibility::AXNode, tree: &AXTree) -> bool {
    const SECTIONING_TAGS: &[&str] = &["MAIN", "ARTICLE", "ASIDE", "NAV", "SECTION"];
    const SECTIONING_ROLES: &[&str] = &["main", "article", "complementary", "navigation", "region"];

    let mut current = node.parent_id.as_deref();
    while let Some(pid) = current {
        let parent = match tree.nodes.get(pid) {
            Some(p) => p,
            None => break,
        };
        if let Some(tag) = parent.get_property_str("htmlTag") {
            if SECTIONING_TAGS.contains(&tag.to_uppercase().as_str()) {
                return true;
            }
        }
        if let Some(role) = parent.role.as_deref() {
            if SECTIONING_ROLES.contains(&role.to_lowercase().as_str()) {
                return true;
            }
        }
        current = parent.parent_id.as_deref();
    }
    false
}

/// Returns true if any ancestor of `node` (up to the document root) has a
/// landmark role.  Used to enforce top-level placement of banner / contentinfo
/// / main.
fn is_inside_landmark(node: &crate::accessibility::AXNode, tree: &AXTree) -> bool {
    let mut current_parent_id = node.parent_id.as_deref();
    while let Some(pid) = current_parent_id {
        if let Some(parent) = tree.nodes.get(pid) {
            if let Some(role) = parent.role.as_deref() {
                if LANDMARK_ROLES.contains(&role.to_lowercase().as_str()) {
                    return true;
                }
            }
            current_parent_id = parent.parent_id.as_deref();
        } else {
            break;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::{AXNode, AXTree};

    fn node(id: &str, role: &str, name: Option<&str>, parent: Option<&str>) -> AXNode {
        AXNode {
            node_id: id.into(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some(role.into()),
            name: name.map(String::from),
            name_source: None,
            description: None,
            value: None,
            properties: vec![],
            child_ids: vec![],
            parent_id: parent.map(String::from),
            backend_dom_node_id: None,
        }
    }

    #[test]
    fn test_duplicate_banner_flagged() {
        let tree = AXTree::from_nodes(vec![
            node("root", "RootWebArea", Some("Page"), None),
            node("1", "banner", Some("Header 1"), Some("root")),
            node("2", "banner", Some("Header 2"), Some("root")),
        ]);
        let r = check_landmark_extended(&tree);
        assert!(r
            .violations
            .iter()
            .any(|v| v.rule_id.as_deref() == Some("landmark-no-duplicate-banner")));
    }

    #[test]
    fn test_single_banner_passes() {
        let tree = AXTree::from_nodes(vec![
            node("root", "RootWebArea", Some("Page"), None),
            node("1", "banner", Some("Site Header"), Some("root")),
        ]);
        let r = check_landmark_extended(&tree);
        assert!(!r
            .violations
            .iter()
            .any(|v| v.rule_id.as_deref() == Some("landmark-no-duplicate-banner")));
    }

    #[test]
    fn test_banner_nested_in_main_flagged() {
        let tree = AXTree::from_nodes(vec![
            node("root", "RootWebArea", Some("Page"), None),
            node("main", "main", Some("Content"), Some("root")),
            node("banner", "banner", Some("Header"), Some("main")),
        ]);
        let r = check_landmark_extended(&tree);
        assert!(r
            .violations
            .iter()
            .any(|v| v.rule_id.as_deref() == Some("landmark-banner-is-top-level")));
    }

    #[test]
    fn test_top_level_banner_passes() {
        let tree = AXTree::from_nodes(vec![
            node("root", "RootWebArea", Some("Page"), None),
            node("banner", "banner", Some("Header"), Some("root")),
        ]);
        let r = check_landmark_extended(&tree);
        assert!(!r
            .violations
            .iter()
            .any(|v| v.rule_id.as_deref() == Some("landmark-banner-is-top-level")));
    }

    #[test]
    fn test_duplicate_nav_same_name_flagged() {
        let tree = AXTree::from_nodes(vec![
            node("root", "RootWebArea", Some("Page"), None),
            node("1", "navigation", Some("Main Nav"), Some("root")),
            node("2", "navigation", Some("Main Nav"), Some("root")),
        ]);
        let r = check_landmark_extended(&tree);
        assert!(r
            .violations
            .iter()
            .any(|v| v.rule_id.as_deref() == Some("landmark-unique")));
    }

    #[test]
    fn test_duplicate_nav_distinct_names_passes() {
        let tree = AXTree::from_nodes(vec![
            node("root", "RootWebArea", Some("Page"), None),
            node("1", "navigation", Some("Primary Nav"), Some("root")),
            node("2", "navigation", Some("Footer Nav"), Some("root")),
        ]);
        let r = check_landmark_extended(&tree);
        assert!(!r
            .violations
            .iter()
            .any(|v| v.rule_id.as_deref() == Some("landmark-unique")));
    }

    #[test]
    fn test_no_skip_link_with_nav_flagged() {
        let tree = AXTree::from_nodes(vec![
            node("root", "RootWebArea", Some("Page"), None),
            node("nav", "navigation", Some("Main"), Some("root")),
            node("main", "main", Some("Content"), Some("root")),
        ]);
        let r = check_landmark_extended(&tree);
        assert!(r
            .violations
            .iter()
            .any(|v| v.rule_id.as_deref() == Some("skip-link")));
    }
}
