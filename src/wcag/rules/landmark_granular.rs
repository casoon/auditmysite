//! Granular WCAG landmark rules
//!
//! Seven individual landmark checks, each exposed as its own function so they
//! can be registered and filtered independently via `run_if_allowed!`:
//!
//! 1. `landmark-unique`                     — same-role landmarks need unique names
//! 2. `landmark-banner-is-top-level`        — banner must not nest in another landmark
//! 3. `landmark-contentinfo-is-top-level`   — contentinfo must not nest
//! 4. `landmark-main-is-top-level`          — main must not nest
//! 5. `landmark-no-duplicate-banner`        — at most one banner
//! 6. `landmark-no-duplicate-contentinfo`   — at most one contentinfo
//! 7. `landmark-no-duplicate-main`          — at most one main
//! 8. `landmark-banner-present`             — page must have a banner landmark
//! 9. `landmark-main-present`               — page must have a main landmark

use std::collections::HashMap;

use chromiumoxide::Page;
use tracing::warn;

use crate::accessibility::{AXNode, AXTree};
use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation, WcagResults};

// ── Rule metadata ──────────────────────────────────────────────────────────────

pub const RULE_LANDMARK_UNIQUE: RuleMetadata = RuleMetadata {
    id: "1.3.1",
    name: "Landmark Unique",
    level: WcagLevel::A,
    severity: Severity::Medium,
    description: "Landmark regions of the same type must have unique accessible names",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/info-and-relationships.html",
    axe_id: "landmark-unique",
    tags: &["wcag2a", "wcag131", "cat.semantics"],
};

pub const RULE_BANNER_IS_TOP_LEVEL: RuleMetadata = RuleMetadata {
    id: "1.3.1",
    name: "Banner Is Top Level",
    level: WcagLevel::A,
    severity: Severity::Medium,
    description: "The banner landmark must not be contained within another landmark",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/info-and-relationships.html",
    axe_id: "landmark-banner-is-top-level",
    tags: &["wcag2a", "wcag131", "cat.semantics"],
};

pub const RULE_CONTENTINFO_IS_TOP_LEVEL: RuleMetadata = RuleMetadata {
    id: "1.3.1",
    name: "Contentinfo Is Top Level",
    level: WcagLevel::A,
    severity: Severity::Medium,
    description: "The contentinfo landmark must not be contained within another landmark",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/info-and-relationships.html",
    axe_id: "landmark-contentinfo-is-top-level",
    tags: &["wcag2a", "wcag131", "cat.semantics"],
};

pub const RULE_MAIN_IS_TOP_LEVEL: RuleMetadata = RuleMetadata {
    id: "1.3.1",
    name: "Main Is Top Level",
    level: WcagLevel::A,
    severity: Severity::Medium,
    description: "The main landmark must not be contained within another landmark",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/info-and-relationships.html",
    axe_id: "landmark-main-is-top-level",
    tags: &["wcag2a", "wcag131", "cat.semantics"],
};

pub const RULE_NO_DUPLICATE_BANNER: RuleMetadata = RuleMetadata {
    id: "1.3.1",
    name: "No Duplicate Banner",
    level: WcagLevel::A,
    severity: Severity::Medium,
    description: "The page must not have more than one banner landmark",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/info-and-relationships.html",
    axe_id: "landmark-no-duplicate-banner",
    tags: &["wcag2a", "wcag131", "cat.semantics"],
};

pub const RULE_NO_DUPLICATE_CONTENTINFO: RuleMetadata = RuleMetadata {
    id: "1.3.1",
    name: "No Duplicate Contentinfo",
    level: WcagLevel::A,
    severity: Severity::Medium,
    description: "The page must not have more than one contentinfo landmark",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/info-and-relationships.html",
    axe_id: "landmark-no-duplicate-contentinfo",
    tags: &["wcag2a", "wcag131", "cat.semantics"],
};

pub const RULE_NO_DUPLICATE_MAIN: RuleMetadata = RuleMetadata {
    id: "1.3.1",
    name: "No Duplicate Main",
    level: WcagLevel::A,
    severity: Severity::Medium,
    description: "The page must not have more than one main landmark",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/info-and-relationships.html",
    axe_id: "landmark-no-duplicate-main",
    tags: &["wcag2a", "wcag131", "cat.semantics"],
};

pub const RULE_LANDMARK_BANNER_PRESENT: RuleMetadata = RuleMetadata {
    id: "1.3.1",
    name: "Landmark Banner Present",
    level: WcagLevel::A,
    severity: Severity::Medium,
    description: "The page must have a banner landmark (<header> or role=\"banner\")",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/info-and-relationships.html",
    axe_id: "landmark-banner-present",
    tags: &["wcag2a", "wcag131", "cat.semantics"],
};

pub const RULE_LANDMARK_MAIN_PRESENT: RuleMetadata = RuleMetadata {
    id: "1.3.1",
    name: "Landmark Main Present",
    level: WcagLevel::A,
    severity: Severity::Medium,
    description: "The page must have a main landmark (<main> or role=\"main\")",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/info-and-relationships.html",
    axe_id: "landmark-main-present",
    tags: &["wcag2a", "wcag131", "cat.semantics"],
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

// ── Shared helpers ─────────────────────────────────────────────────────────────

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

/// Returns `true` when the given role string is a landmark role.
fn is_landmark_role(role: &str) -> bool {
    LANDMARK_ROLES.contains(&role.to_lowercase().as_str())
}

/// Returns `true` when none of the node's ancestors have a landmark role (i.e.
/// the node is at the top level of the landmark hierarchy).
fn is_top_level_landmark(node: &AXNode, tree: &AXTree) -> bool {
    let mut current = node.parent_id.as_deref();
    while let Some(pid) = current {
        if let Some(parent) = tree.nodes.get(pid) {
            if let Some(ref role) = parent.role {
                if is_landmark_role(role) {
                    return false;
                }
            }
            current = parent.parent_id.as_deref();
        } else {
            break;
        }
    }
    true
}

// ── Generic check helpers ──────────────────────────────────────────────────────

/// Check that every node with `target_role` is a top-level landmark.
///
/// `implicit_role_tag` names the HTML element that *implicitly* maps to this
/// role (e.g. `HEADER` → banner, `FOOTER` → contentinfo). When the node has
/// that htmlTag AND sits inside HTML sectioning content (main / article /
/// aside / nav / section), the HTML spec does not assign the implicit role —
/// browsers that still report it are wrong, so we skip the violation.
fn check_top_level_landmark(
    tree: &AXTree,
    target_role: &str,
    meta: &RuleMetadata,
    fix_hint: &str,
    implicit_role_tag: Option<&str>,
) -> WcagResults {
    let mut results = WcagResults::new();
    let nodes = tree.nodes_with_role(target_role);
    for node in &nodes {
        if is_top_level_landmark(node, tree)
            || implicit_role_tag
                .map(|tag| is_tag_in_sectioning_content(node, tag, tree))
                .unwrap_or(false)
        {
            results.passes += 1;
        } else {
            results.add_violation(
                Violation::new(
                    meta.id,
                    meta.name,
                    meta.level,
                    meta.severity,
                    format!("{} landmark is nested inside another landmark", target_role),
                    &node.node_id,
                )
                .with_role(node.role.clone())
                .with_fix(fix_hint)
                .with_rule_id(meta.axe_id)
                .with_help_url(meta.help_url),
            );
        }
    }
    results
}

/// Returns true when `node.htmlTag` equals `tag` (case-insensitive) AND the
/// node has an ancestor whose htmlTag/role denotes HTML sectioning content.
fn is_tag_in_sectioning_content(node: &AXNode, tag: &str, tree: &AXTree) -> bool {
    if node.get_property_str("htmlTag").map(|t| t.to_uppercase()) != Some(tag.to_uppercase()) {
        return false;
    }
    const SECTIONING_TAGS: &[&str] = &["MAIN", "ARTICLE", "ASIDE", "NAV", "SECTION"];
    const SECTIONING_ROLES: &[&str] = &["main", "article", "complementary", "navigation", "region"];
    let mut current = node.parent_id.as_deref();
    while let Some(pid) = current {
        let parent = match tree.nodes.get(pid) {
            Some(p) => p,
            None => break,
        };
        if let Some(t) = parent.get_property_str("htmlTag") {
            if SECTIONING_TAGS.contains(&t.to_uppercase().as_str()) {
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

/// Check that at most one node with `target_role` exists on the page.
fn check_no_duplicate_landmark(
    tree: &AXTree,
    target_role: &str,
    meta: &RuleMetadata,
    fix_hint: &str,
) -> WcagResults {
    let mut results = WcagResults::new();
    let nodes = tree.nodes_with_role(target_role);
    if nodes.len() > 1 {
        results.add_violation(
            Violation::new(
                meta.id,
                meta.name,
                meta.level,
                meta.severity,
                format!(
                    "Page has {} {} landmarks; only one is permitted",
                    nodes.len(),
                    target_role
                ),
                &nodes[0].node_id,
            )
            .with_fix(fix_hint)
            .with_rule_id(meta.axe_id)
            .with_help_url(meta.help_url),
        );
    } else {
        results.passes += 1;
    }
    results
}

// ── Public check functions (one per rule) ──────────────────────────────────────

/// **landmark-unique** — Multiple landmarks of the same role must have unique
/// accessible names.
pub fn check_landmark_unique(tree: &AXTree) -> WcagResults {
    let mut results = WcagResults::new();

    // Group nodes by landmark role
    let mut by_role: HashMap<&str, Vec<&AXNode>> = HashMap::new();
    for role in LANDMARK_ROLES {
        let nodes = tree.nodes_with_role(role);
        if !nodes.is_empty() {
            by_role.insert(role, nodes);
        }
    }

    for (role, nodes) in &by_role {
        if nodes.len() < 2 {
            results.passes += 1;
            continue;
        }

        // Count occurrences per normalised accessible name
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
                            "Multiple '{}' landmarks share the same accessible name; \
                             they cannot be distinguished",
                            role
                        ),
                        &node.node_id,
                    )
                    .with_role(node.role.clone())
                    .with_fix(format!(
                        "Add a unique aria-label to each '{}' landmark so they \
                         can be told apart",
                        role
                    ))
                    .with_rule_id(RULE_LANDMARK_UNIQUE.axe_id)
                    .with_help_url(RULE_LANDMARK_UNIQUE.help_url),
                );
            } else {
                results.passes += 1;
            }
        }
    }

    results
}

/// **landmark-banner-is-top-level**
pub fn check_landmark_banner_is_top_level(tree: &AXTree) -> WcagResults {
    check_top_level_landmark(
        tree,
        "banner",
        &RULE_BANNER_IS_TOP_LEVEL,
        "Move the <header> / role=\"banner\" element to the top level, \
         outside all other landmarks",
        Some("HEADER"),
    )
}

/// **landmark-contentinfo-is-top-level**
pub fn check_landmark_contentinfo_is_top_level(tree: &AXTree) -> WcagResults {
    check_top_level_landmark(
        tree,
        "contentinfo",
        &RULE_CONTENTINFO_IS_TOP_LEVEL,
        "Move the <footer> / role=\"contentinfo\" element to the top level, \
         outside all other landmarks",
        Some("FOOTER"),
    )
}

/// **landmark-main-is-top-level**
pub fn check_landmark_main_is_top_level(tree: &AXTree) -> WcagResults {
    check_top_level_landmark(
        tree,
        "main",
        &RULE_MAIN_IS_TOP_LEVEL,
        "Move the <main> / role=\"main\" element to the top level, \
         not inside other landmarks",
        None,
    )
}

/// **landmark-no-duplicate-banner**
pub fn check_landmark_no_duplicate_banner(tree: &AXTree) -> WcagResults {
    check_no_duplicate_landmark(
        tree,
        "banner",
        &RULE_NO_DUPLICATE_BANNER,
        "Ensure the page has at most one <header> / role=\"banner\" \
         at the top level",
    )
}

/// **landmark-no-duplicate-contentinfo**
pub fn check_landmark_no_duplicate_contentinfo(tree: &AXTree) -> WcagResults {
    check_no_duplicate_landmark(
        tree,
        "contentinfo",
        &RULE_NO_DUPLICATE_CONTENTINFO,
        "Ensure the page has at most one <footer> / role=\"contentinfo\" \
         at the top level",
    )
}

/// **landmark-no-duplicate-main**
pub fn check_landmark_no_duplicate_main(tree: &AXTree) -> WcagResults {
    check_no_duplicate_landmark(
        tree,
        "main",
        &RULE_NO_DUPLICATE_MAIN,
        "Ensure the page has exactly one <main> / role=\"main\"",
    )
}

/// **landmark-banner-present** — page must have at least one banner landmark.
///
/// Only fires when the tree has enough nodes to represent a real page (> 2 nodes),
/// avoiding false positives on error pages or minimal shells.
pub fn check_landmark_banner_present(tree: &AXTree) -> WcagResults {
    let mut results = WcagResults::new();
    let node_count = tree.nodes.len();
    if node_count <= 2 {
        return results;
    }
    let banners = tree.nodes_with_role("banner");
    if banners.is_empty() {
        results.add_violation(
            Violation::new(
                RULE_LANDMARK_BANNER_PRESENT.id,
                RULE_LANDMARK_BANNER_PRESENT.name,
                RULE_LANDMARK_BANNER_PRESENT.level,
                RULE_LANDMARK_BANNER_PRESENT.severity,
                "Page has no banner landmark — assistive technologies cannot identify the site header",
                "root",
            )
            .with_fix(
                "Add a top-level <header> element (or an element with role=\"banner\") \
                 that is not nested inside <main>, <article>, <aside>, <nav>, or <section>",
            )
            .with_rule_id(RULE_LANDMARK_BANNER_PRESENT.axe_id)
            .with_help_url(RULE_LANDMARK_BANNER_PRESENT.help_url),
        );
    } else {
        results.passes += 1;
    }
    results
}

/// **landmark-main-present** — page must have at least one main landmark.
pub fn check_landmark_main_present(tree: &AXTree) -> WcagResults {
    let mut results = WcagResults::new();
    let node_count = tree.nodes.len();
    if node_count <= 2 {
        return results;
    }
    let mains = tree.nodes_with_role("main");
    if mains.is_empty() {
        results.add_violation(
            Violation::new(
                RULE_LANDMARK_MAIN_PRESENT.id,
                RULE_LANDMARK_MAIN_PRESENT.name,
                RULE_LANDMARK_MAIN_PRESENT.level,
                RULE_LANDMARK_MAIN_PRESENT.severity,
                "Page has no main landmark — assistive technologies cannot skip to the primary content",
                "root",
            )
            .with_fix(
                "Wrap the page's primary content in a <main> element \
                 (or add role=\"main\" to the container)",
            )
            .with_rule_id(RULE_LANDMARK_MAIN_PRESENT.axe_id)
            .with_help_url(RULE_LANDMARK_MAIN_PRESENT.help_url),
        );
    } else {
        results.passes += 1;
    }
    results
}

/// DOM supplement for landmark parity. Some headless pages expose less
/// landmark structure through the AX tree than is visible in the DOM; this
/// mirrors the two deterministic axe cases we care about here: missing main
/// landmark and same-role landmarks with the same accessible name.
pub async fn check_landmarks_with_page(page: &Page) -> Vec<Violation> {
    let js = [
        "(function() {",
        crate::accessibility::js_helpers::CSS_SELECTOR_JS,
        r#"
        function isHidden(el) {
          if (el.hasAttribute('hidden') || el.getAttribute('aria-hidden') === 'true') return true;
          var cur = el;
          while (cur && cur.nodeType === 1 && cur !== document.documentElement) {
            var s = window.getComputedStyle(cur);
            if (s.display === 'none') return true;
            if (s.visibility === 'hidden' || s.visibility === 'collapse') return true;
            cur = cur.parentElement;
          }
          return false;
        }
        function nameFor(el) {
          var label = (el.getAttribute('aria-label') || '').trim();
          if (label) return label;
          var labelledBy = (el.getAttribute('aria-labelledby') || '').trim();
          if (labelledBy) {
            var text = labelledBy.split(/\s+/).map(function(id) {
              var ref = document.getElementById(id);
              return ref ? ref.textContent.trim() : '';
            }).join(' ').trim();
            if (text) return text;
          }
          return '';
        }
        function implicitRole(el) {
          var tag = el.tagName.toLowerCase();
          if (tag === 'main') return 'main';
          if (tag === 'nav') return 'navigation';
          if (tag === 'aside') return 'complementary';
          if (tag === 'header') return 'banner';
          if (tag === 'footer') return 'contentinfo';
          return '';
        }

        var results = [];
        var landmarks = [];
        var candidates = document.querySelectorAll(
          'main, nav, aside, header, footer, [role="main"], [role="navigation"], ' +
          '[role="banner"], [role="contentinfo"], [role="complementary"], [role="search"], [role="region"]'
        );
        for (var i = 0; i < candidates.length; i++) {
          var el = candidates[i];
          if (isHidden(el)) continue;
          var role = (el.getAttribute('role') || implicitRole(el)).toLowerCase();
          if (!role) continue;
          landmarks.push({
            role: role,
            name: nameFor(el).toLowerCase(),
            selector: __amsCssSelector(el),
            snippet: el.outerHTML.substring(0, 200)
          });
        }

        if (!landmarks.some(function(l) { return l.role === 'main'; })) {
          results.push({ rule_id: 'landmark-main-present', selector: 'document', snippet: '' });
        }

        var byKey = {};
        for (var j = 0; j < landmarks.length; j++) {
          var l = landmarks[j];
          var key = l.role + '\u0000' + l.name;
          if (!byKey[key]) byKey[key] = [];
          byKey[key].push(l);
        }
        Object.keys(byKey).forEach(function(key) {
          var group = byKey[key];
          if (group.length < 2) return;
          for (var k = 0; k < group.length; k++) {
            results.push({
              rule_id: 'landmark-unique',
              role: group[k].role,
              selector: group[k].selector,
              snippet: group[k].snippet
            });
          }
        });

        return results;
        "#,
        "})()",
    ]
    .concat();

    let result = match page.evaluate(js.as_str()).await {
        Ok(r) => r,
        Err(e) => {
            warn!("landmark DOM JS failed: {}", e);
            return vec![crate::wcag::technical_rule_failure_for(
                "landmarks-dom",
                crate::cli::WcagLevel::A,
                "page_evaluation_failed",
            )];
        }
    };

    let Some(value) = result.value() else {
        return vec![crate::wcag::technical_rule_failure_for(
            "landmarks-dom",
            crate::cli::WcagLevel::A,
            "missing_evaluation_value",
        )];
    };
    let Some(items) = value.as_array() else {
        return vec![];
    };

    items
        .iter()
        .filter_map(|item| {
            let rule_id = item.get("rule_id")?.as_str()?;
            let selector = item
                .get("selector")
                .and_then(|v| v.as_str())
                .unwrap_or("document");
            let (meta, message, fix) = match rule_id {
                "landmark-main-present" => (
                    &RULE_LANDMARK_MAIN_PRESENT,
                    "Page has no main landmark — assistive technologies cannot skip to the primary content".to_string(),
                    "Wrap the page's primary content in a <main> element (or add role=\"main\" to the container)".to_string(),
                ),
                "landmark-unique" => {
                    let role = item
                        .get("role")
                        .and_then(|v| v.as_str())
                        .unwrap_or("landmark");
                    (
                        &RULE_LANDMARK_UNIQUE,
                        format!(
                            "Multiple '{}' landmarks share the same accessible name; they cannot be distinguished",
                            role
                        ),
                        format!(
                            "Add a unique aria-label to each '{}' landmark so they can be told apart",
                            role
                        ),
                    )
                }
                _ => return None,
            };

            let mut violation = Violation::new(
                meta.id,
                meta.name,
                meta.level,
                meta.severity,
                message,
                selector,
            )
            .with_selector(selector)
            .with_rule_id(meta.axe_id)
            .with_tags(meta.tags.iter().map(|s| s.to_string()).collect())
            .with_fix(fix)
            .with_help_url(meta.help_url);

            if let Some(snippet) = item.get("snippet").and_then(|v| v.as_str()) {
                if !snippet.is_empty() {
                    violation = violation.with_html_snippet(snippet);
                }
            }

            Some(violation)
        })
        .collect()
}

/// **skip-link** — page with a navigation landmark must have a skip-navigation link.
pub fn check_skip_link(tree: &AXTree) -> WcagResults {
    let mut results = WcagResults::new();

    let has_nav = !tree.nodes_with_role("navigation").is_empty();
    if !has_nav {
        return results;
    }

    let has_skip_link = tree.iter().any(|n| {
        if n.ignored || !matches!(n.role.as_deref(), Some("link")) {
            return false;
        }
        let name = n.name.as_deref().unwrap_or("").to_lowercase();
        // CDP exposes a link's target as "url", never "href" (#QA-032).
        let href = n.get_property_str("url").unwrap_or("").to_lowercase();

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
        results.add_violation(
            Violation::new(
                RULE_SKIP_LINK.id,
                RULE_SKIP_LINK.name,
                RULE_SKIP_LINK.level,
                RULE_SKIP_LINK.severity,
                "Page has navigation landmarks but no skip-navigation link was found",
                "root",
            )
            .with_fix(
                "Add a visually hidden or visible link at the top of the page pointing to \
                 the main content anchor (e.g. <a href=\"#main\">Skip to main content</a>)",
            )
            .with_rule_id(RULE_SKIP_LINK.axe_id)
            .with_help_url(RULE_SKIP_LINK.help_url),
        );
    }

    results
}

// ── Tests ──────────────────────────────────────────────────────────────────────

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

    // ── landmark-unique ────────────────────────────────────────────────────

    #[test]
    fn landmark_unique_same_role_same_name_violation() {
        let tree = AXTree::from_nodes(vec![
            node("root", "RootWebArea", Some("Page"), None),
            node("n1", "navigation", Some("Main Nav"), Some("root")),
            node("n2", "navigation", Some("Main Nav"), Some("root")),
        ]);
        let r = check_landmark_unique(&tree);
        assert!(
            r.violations
                .iter()
                .any(|v| v.rule_id.as_deref() == Some("landmark-unique")),
            "Two navs with the same name should trigger a violation"
        );
    }

    #[test]
    fn landmark_unique_same_role_different_names_pass() {
        let tree = AXTree::from_nodes(vec![
            node("root", "RootWebArea", Some("Page"), None),
            node("n1", "navigation", Some("Primary"), Some("root")),
            node("n2", "navigation", Some("Footer"), Some("root")),
        ]);
        let r = check_landmark_unique(&tree);
        assert!(
            !r.violations
                .iter()
                .any(|v| v.rule_id.as_deref() == Some("landmark-unique")),
            "Two navs with distinct names should pass"
        );
    }

    #[test]
    fn landmark_unique_single_landmark_pass() {
        let tree = AXTree::from_nodes(vec![
            node("root", "RootWebArea", Some("Page"), None),
            node("n1", "navigation", Some("Nav"), Some("root")),
        ]);
        let r = check_landmark_unique(&tree);
        assert!(r.violations.is_empty());
        assert!(r.passes > 0);
    }

    // ── landmark-banner-is-top-level ───────────────────────────────────────

    #[test]
    fn banner_nested_in_main_violation() {
        let tree = AXTree::from_nodes(vec![
            node("root", "RootWebArea", Some("Page"), None),
            node("m", "main", Some("Content"), Some("root")),
            node("b", "banner", Some("Header"), Some("m")),
        ]);
        let r = check_landmark_banner_is_top_level(&tree);
        assert!(r
            .violations
            .iter()
            .any(|v| v.rule_id.as_deref() == Some("landmark-banner-is-top-level")));
    }

    #[test]
    fn banner_top_level_pass() {
        let tree = AXTree::from_nodes(vec![
            node("root", "RootWebArea", Some("Page"), None),
            node("b", "banner", Some("Header"), Some("root")),
        ]);
        let r = check_landmark_banner_is_top_level(&tree);
        assert!(r.violations.is_empty());
        assert!(r.passes > 0);
    }

    #[test]
    fn header_in_main_does_not_trigger_banner_nested_violation() {
        // Per HTML spec, a <header> nested in <main> has no implicit banner
        // role. If Chrome still reports role="banner", we must not flag it.
        use crate::accessibility::{AXProperty, AXValue};
        let mut header = node("b", "banner", Some("Section"), Some("m"));
        header.properties.push(AXProperty {
            name: "htmlTag".into(),
            value: AXValue::String("HEADER".into()),
        });
        let tree = AXTree::from_nodes(vec![
            node("root", "RootWebArea", Some("Page"), None),
            node("m", "main", Some("Content"), Some("root")),
            header,
        ]);
        let r = check_landmark_banner_is_top_level(&tree);
        assert!(
            r.violations.is_empty(),
            "<header> inside <main> must not be flagged as nested banner"
        );
    }

    // ── landmark-contentinfo-is-top-level ──────────────────────────────────

    #[test]
    fn contentinfo_nested_violation() {
        let tree = AXTree::from_nodes(vec![
            node("root", "RootWebArea", Some("Page"), None),
            node("m", "main", Some("Content"), Some("root")),
            node("f", "contentinfo", Some("Footer"), Some("m")),
        ]);
        let r = check_landmark_contentinfo_is_top_level(&tree);
        assert!(r
            .violations
            .iter()
            .any(|v| v.rule_id.as_deref() == Some("landmark-contentinfo-is-top-level")));
    }

    #[test]
    fn contentinfo_top_level_pass() {
        let tree = AXTree::from_nodes(vec![
            node("root", "RootWebArea", Some("Page"), None),
            node("f", "contentinfo", Some("Footer"), Some("root")),
        ]);
        let r = check_landmark_contentinfo_is_top_level(&tree);
        assert!(r.violations.is_empty());
        assert!(r.passes > 0);
    }

    #[test]
    fn footer_in_article_does_not_trigger_contentinfo_nested_violation() {
        use crate::accessibility::{AXProperty, AXValue};
        let mut footer = node("f", "contentinfo", Some("Article footer"), Some("a"));
        footer.properties.push(AXProperty {
            name: "htmlTag".into(),
            value: AXValue::String("FOOTER".into()),
        });
        let mut art = node("a", "article", Some("Article"), Some("root"));
        art.properties.push(AXProperty {
            name: "htmlTag".into(),
            value: AXValue::String("ARTICLE".into()),
        });
        let tree = AXTree::from_nodes(vec![
            node("root", "RootWebArea", Some("Page"), None),
            art,
            footer,
        ]);
        let r = check_landmark_contentinfo_is_top_level(&tree);
        assert!(
            r.violations.is_empty(),
            "<footer> inside <article> must not be flagged as nested contentinfo"
        );
    }

    // ── landmark-main-is-top-level ─────────────────────────────────────────

    #[test]
    fn main_nested_in_navigation_violation() {
        let tree = AXTree::from_nodes(vec![
            node("root", "RootWebArea", Some("Page"), None),
            node("nav", "navigation", Some("Nav"), Some("root")),
            node("m", "main", Some("Content"), Some("nav")),
        ]);
        let r = check_landmark_main_is_top_level(&tree);
        assert!(r
            .violations
            .iter()
            .any(|v| v.rule_id.as_deref() == Some("landmark-main-is-top-level")));
    }

    #[test]
    fn main_top_level_pass() {
        let tree = AXTree::from_nodes(vec![
            node("root", "RootWebArea", Some("Page"), None),
            node("m", "main", Some("Content"), Some("root")),
        ]);
        let r = check_landmark_main_is_top_level(&tree);
        assert!(r.violations.is_empty());
        assert!(r.passes > 0);
    }

    // ── landmark-no-duplicate-banner ───────────────────────────────────────

    #[test]
    fn duplicate_banner_violation() {
        let tree = AXTree::from_nodes(vec![
            node("root", "RootWebArea", Some("Page"), None),
            node("b1", "banner", Some("H1"), Some("root")),
            node("b2", "banner", Some("H2"), Some("root")),
        ]);
        let r = check_landmark_no_duplicate_banner(&tree);
        assert!(r
            .violations
            .iter()
            .any(|v| v.rule_id.as_deref() == Some("landmark-no-duplicate-banner")));
    }

    #[test]
    fn single_banner_pass() {
        let tree = AXTree::from_nodes(vec![
            node("root", "RootWebArea", Some("Page"), None),
            node("b", "banner", Some("Header"), Some("root")),
        ]);
        let r = check_landmark_no_duplicate_banner(&tree);
        assert!(r.violations.is_empty());
        assert!(r.passes > 0);
    }

    // ── landmark-no-duplicate-contentinfo ──────────────────────────────────

    #[test]
    fn duplicate_contentinfo_violation() {
        let tree = AXTree::from_nodes(vec![
            node("root", "RootWebArea", Some("Page"), None),
            node("f1", "contentinfo", Some("F1"), Some("root")),
            node("f2", "contentinfo", Some("F2"), Some("root")),
        ]);
        let r = check_landmark_no_duplicate_contentinfo(&tree);
        assert!(r
            .violations
            .iter()
            .any(|v| v.rule_id.as_deref() == Some("landmark-no-duplicate-contentinfo")));
    }

    #[test]
    fn single_contentinfo_pass() {
        let tree = AXTree::from_nodes(vec![
            node("root", "RootWebArea", Some("Page"), None),
            node("f", "contentinfo", Some("Footer"), Some("root")),
        ]);
        let r = check_landmark_no_duplicate_contentinfo(&tree);
        assert!(r.violations.is_empty());
        assert!(r.passes > 0);
    }

    // ── landmark-no-duplicate-main ─────────────────────────────────────────

    #[test]
    fn duplicate_main_violation() {
        let tree = AXTree::from_nodes(vec![
            node("root", "RootWebArea", Some("Page"), None),
            node("m1", "main", Some("M1"), Some("root")),
            node("m2", "main", Some("M2"), Some("root")),
        ]);
        let r = check_landmark_no_duplicate_main(&tree);
        assert!(r
            .violations
            .iter()
            .any(|v| v.rule_id.as_deref() == Some("landmark-no-duplicate-main")));
    }

    #[test]
    fn single_main_pass() {
        let tree = AXTree::from_nodes(vec![
            node("root", "RootWebArea", Some("Page"), None),
            node("m", "main", Some("Content"), Some("root")),
        ]);
        let r = check_landmark_no_duplicate_main(&tree);
        assert!(r.violations.is_empty());
        assert!(r.passes > 0);
    }

    // ── skip-link ──────────────────────────────────────────────────────────

    #[test]
    fn skip_link_missing_with_nav_violation() {
        let tree = AXTree::from_nodes(vec![
            node("root", "RootWebArea", Some("Page"), None),
            node("nav", "navigation", Some("Main"), Some("root")),
            node("main", "main", Some("Content"), Some("root")),
        ]);
        let r = check_skip_link(&tree);
        assert!(r
            .violations
            .iter()
            .any(|v| v.rule_id.as_deref() == Some("skip-link")));
    }

    #[test]
    fn skip_link_no_nav_no_violation() {
        let tree = AXTree::from_nodes(vec![
            node("root", "RootWebArea", Some("Page"), None),
            node("main", "main", Some("Content"), Some("root")),
        ]);
        let r = check_skip_link(&tree);
        assert!(r.violations.is_empty());
    }

    // ── Regression: WCAG criterion-id taxonomy (issue #242) ────────────────
    //
    // Landmark-structure findings (duplicate/nested landmarks, unique names)
    // MUST be filed under WCAG 1.3.1, never under 2.4.1. Only `skip-link`
    // belongs under 2.4.1, so the "Fehlende Sprungnavigation" group in the
    // report no longer mixes unrelated landmark findings.

    fn dup_nav_tree() -> AXTree {
        AXTree::from_nodes(vec![
            node("root", "RootWebArea", Some("Page"), None),
            node("n1", "navigation", Some("Nav"), Some("root")),
            node("n2", "navigation", Some("Nav"), Some("root")),
        ])
    }

    fn nested_banner_tree() -> AXTree {
        AXTree::from_nodes(vec![
            node("root", "RootWebArea", Some("Page"), None),
            node("m", "main", Some("Content"), Some("root")),
            node("b", "banner", Some("Header"), Some("m")),
        ])
    }

    #[test]
    fn landmark_structure_findings_are_filed_under_131_not_241() {
        let checks: Vec<(&str, WcagResults)> = vec![
            ("landmark-unique", check_landmark_unique(&dup_nav_tree())),
            (
                "landmark-banner-is-top-level",
                check_landmark_banner_is_top_level(&nested_banner_tree()),
            ),
            (
                "landmark-no-duplicate-banner",
                check_landmark_no_duplicate_banner(&AXTree::from_nodes(vec![
                    node("root", "RootWebArea", Some("Page"), None),
                    node("b1", "banner", Some("H1"), Some("root")),
                    node("b2", "banner", Some("H2"), Some("root")),
                ])),
            ),
            (
                "landmark-no-duplicate-main",
                check_landmark_no_duplicate_main(&AXTree::from_nodes(vec![
                    node("root", "RootWebArea", Some("Page"), None),
                    node("m1", "main", Some("M1"), Some("root")),
                    node("m2", "main", Some("M2"), Some("root")),
                ])),
            ),
        ];

        for (axe_id, r) in checks {
            assert!(
                !r.violations.is_empty(),
                "{axe_id}: expected at least one violation in the test fixture"
            );
            for v in &r.violations {
                assert_eq!(
                    v.rule, "1.3.1",
                    "{axe_id} produced a violation under WCAG '{}', expected '1.3.1' \
                     (landmark structure must not be aggregated under 2.4.1 — issue #242)",
                    v.rule
                );
            }
        }
    }

    #[test]
    fn skip_link_is_filed_under_241() {
        let r = check_skip_link(&AXTree::from_nodes(vec![
            node("root", "RootWebArea", Some("Page"), None),
            node("nav", "navigation", Some("Main"), Some("root")),
            node("main", "main", Some("Content"), Some("root")),
        ]));
        assert!(!r.violations.is_empty());
        for v in &r.violations {
            assert_eq!(v.rule, "2.4.1", "skip-link must stay under WCAG 2.4.1");
            assert_eq!(v.rule_id.as_deref(), Some("skip-link"));
        }
    }

    // ── landmark-banner-present ────────────────────────────────────────────

    #[test]
    fn banner_absent_violation() {
        let tree = AXTree::from_nodes(vec![
            node("root", "RootWebArea", Some("Page"), None),
            node("nav", "navigation", Some("Main"), Some("root")),
            node("main", "main", Some("Content"), Some("root")),
        ]);
        let r = check_landmark_banner_present(&tree);
        assert!(
            r.violations
                .iter()
                .any(|v| v.rule_id.as_deref() == Some("landmark-banner-present")),
            "No banner landmark should trigger a violation"
        );
    }

    #[test]
    fn banner_present_pass() {
        let tree = AXTree::from_nodes(vec![
            node("root", "RootWebArea", Some("Page"), None),
            node("b", "banner", Some("Header"), Some("root")),
            node("main", "main", Some("Content"), Some("root")),
        ]);
        let r = check_landmark_banner_present(&tree);
        assert!(r.violations.is_empty());
        assert!(r.passes > 0);
    }

    #[test]
    fn banner_absent_tiny_tree_no_violation() {
        // Two-node trees (root + one node) must not fire
        let tree = AXTree::from_nodes(vec![
            node("root", "RootWebArea", Some("Page"), None),
            node("div", "generic", None, Some("root")),
        ]);
        let r = check_landmark_banner_present(&tree);
        assert!(r.violations.is_empty());
    }

    // ── landmark-main-present ──────────────────────────────────────────────

    #[test]
    fn main_absent_violation() {
        let tree = AXTree::from_nodes(vec![
            node("root", "RootWebArea", Some("Page"), None),
            node("b", "banner", Some("Header"), Some("root")),
            node("nav", "navigation", Some("Nav"), Some("root")),
        ]);
        let r = check_landmark_main_present(&tree);
        assert!(
            r.violations
                .iter()
                .any(|v| v.rule_id.as_deref() == Some("landmark-main-present")),
            "No main landmark should trigger a violation"
        );
    }

    #[test]
    fn main_present_pass() {
        let tree = AXTree::from_nodes(vec![
            node("root", "RootWebArea", Some("Page"), None),
            node("b", "banner", Some("Header"), Some("root")),
            node("m", "main", Some("Content"), Some("root")),
        ]);
        let r = check_landmark_main_present(&tree);
        assert!(r.violations.is_empty());
        assert!(r.passes > 0);
    }
}
