//! UX Heuristic checks
//!
//! AXTree-based checks for common UX anti-patterns:
//! - Cookie-banner detection
//! - Modal/overlay detection
//! - Font diversity advisory
//! - CTA (Call-to-Action) detection

use crate::accessibility::AXTree;
use crate::taxonomy::Severity;
use serde::{Deserialize, Serialize};

/// A single UX heuristic finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UxHeuristicFinding {
    /// Short identifier for the check (e.g. "cookie_banner")
    pub check: String,
    /// Human-readable message
    pub message: String,
    /// Severity of the finding
    pub severity: Severity,
}

/// Result of all UX heuristic checks for a page
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UxHeuristics {
    /// All findings produced by the checks
    pub findings: Vec<UxHeuristicFinding>,
}

/// Keywords that indicate cookie-consent / privacy banners
const COOKIE_KEYWORDS: &[&str] = &[
    "cookie",
    "consent",
    "gdpr",
    "accept",
    "privacy policy",
    "we use cookies",
];

/// Common CTA keyword stems (lower-case, checked with `contains`)
const CTA_KEYWORDS: &[&str] = &[
    "buy",
    "sign up",
    "register",
    "get started",
    "subscribe",
    "download",
    "contact",
    "book",
    "order",
    "start",
];

/// Run all UX heuristic checks against an AXTree and return the results.
pub fn analyze_ux_heuristics(ax_tree: &AXTree) -> UxHeuristics {
    let mut findings = Vec::new();

    check_cookie_banner(ax_tree, &mut findings);
    check_modals(ax_tree, &mut findings);
    check_font_diversity(&mut findings);
    check_ctas(ax_tree, &mut findings);

    UxHeuristics { findings }
}

// ---------------------------------------------------------------------------
// Cookie-banner detection
// ---------------------------------------------------------------------------

fn name_contains_cookie_keyword(name: &str) -> bool {
    let lower = name.to_lowercase();
    COOKIE_KEYWORDS.iter().any(|kw| lower.contains(kw))
}

fn check_cookie_banner(ax_tree: &AXTree, findings: &mut Vec<UxHeuristicFinding>) {
    let banner_found = ax_tree.iter().any(|node| {
        // Match any role if name contains cookie keywords
        if let Some(name) = &node.name {
            if name_contains_cookie_keyword(name) {
                return true;
            }
        }
        // Also check dialog/alertdialog nodes whose description contains keywords
        if matches!(node.role.as_deref(), Some("dialog") | Some("alertdialog")) {
            if let Some(desc) = &node.description {
                if name_contains_cookie_keyword(desc) {
                    return true;
                }
            }
        }
        false
    });

    if banner_found {
        findings.push(UxHeuristicFinding {
            check: "cookie_banner".to_string(),
            message: "Cookie banner detected — verify it doesn't block main content or trap keyboard focus".to_string(),
            severity: Severity::Medium,
        });
    }
}

// ---------------------------------------------------------------------------
// Modal / overlay detection
// ---------------------------------------------------------------------------

fn check_modals(ax_tree: &AXTree, findings: &mut Vec<UxHeuristicFinding>) {
    let dialog_nodes: Vec<_> = ax_tree
        .iter()
        .filter(|n| matches!(n.role.as_deref(), Some("dialog") | Some("alertdialog")) && !n.ignored)
        .collect();

    let dialog_count = dialog_nodes.len();

    if dialog_count == 0 {
        return;
    }

    // Check whether any dialog is missing aria-modal=true
    let missing_aria_modal = dialog_nodes
        .iter()
        .any(|n| !n.get_property_bool("modal").unwrap_or(false));

    if missing_aria_modal {
        findings.push(UxHeuristicFinding {
            check: "modal_no_aria_modal".to_string(),
            message:
                "Modal/overlay may not trap focus properly — consider adding aria-modal=\"true\""
                    .to_string(),
            severity: Severity::Medium,
        });
    }

    if dialog_count > 1 {
        findings.push(UxHeuristicFinding {
            check: "multiple_dialogs".to_string(),
            message: format!(
                "Multiple overlapping dialogs detected ({} dialogs found)",
                dialog_count
            ),
            severity: Severity::Low,
        });
    }
}

// ---------------------------------------------------------------------------
// Font diversity advisory
// ---------------------------------------------------------------------------

fn check_font_diversity(findings: &mut Vec<UxHeuristicFinding>) {
    // Font information is not available in the AXTree; this is a structural advisory.
    findings.push(UxHeuristicFinding {
        check: "font_diversity_advisory".to_string(),
        message: "Font diversity check requires CSS analysis — consider limiting to 2–3 font families for performance and readability".to_string(),
        severity: Severity::Low,
    });
}

// ---------------------------------------------------------------------------
// CTA detection
// ---------------------------------------------------------------------------

fn name_matches_cta(name: &str) -> bool {
    let lower = name.to_lowercase();
    CTA_KEYWORDS.iter().any(|kw| lower.contains(kw))
}

fn check_ctas(ax_tree: &AXTree, findings: &mut Vec<UxHeuristicFinding>) {
    let cta_count = ax_tree
        .iter()
        .filter(|n| {
            matches!(n.role.as_deref(), Some("button") | Some("link"))
                && n.name.as_deref().map(name_matches_cta).unwrap_or(false)
        })
        .count();

    if cta_count == 0 {
        findings.push(UxHeuristicFinding {
            check: "no_cta".to_string(),
            message: "No clear call-to-action elements detected".to_string(),
            severity: Severity::Low,
        });
    } else if cta_count >= 5 {
        findings.push(UxHeuristicFinding {
            check: "too_many_ctas".to_string(),
            message: format!(
                "Multiple call-to-action elements may compete for attention (found {})",
                cta_count
            ),
            severity: Severity::Low,
        });
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::{AXNode, AXProperty, AXTree, AXValue};

    fn node(id: &str, role: &str, name: Option<&str>) -> AXNode {
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

    fn node_with_props(id: &str, role: &str, name: Option<&str>, props: Vec<AXProperty>) -> AXNode {
        let mut n = node(id, role, name);
        n.properties = props;
        n
    }

    // --- cookie banner ---

    #[test]
    fn test_cookie_banner_detected_by_name() {
        let tree = AXTree::from_nodes(vec![
            node("1", "WebArea", Some("Page")),
            node(
                "2",
                "region",
                Some("We use cookies to improve your experience"),
            ),
        ]);
        let result = analyze_ux_heuristics(&tree);
        assert!(
            result.findings.iter().any(|f| f.check == "cookie_banner"),
            "should detect cookie banner from node name"
        );
    }

    #[test]
    fn test_no_cookie_banner() {
        let tree = AXTree::from_nodes(vec![
            node("1", "WebArea", Some("Page")),
            node("2", "heading", Some("Welcome")),
        ]);
        let result = analyze_ux_heuristics(&tree);
        assert!(
            !result.findings.iter().any(|f| f.check == "cookie_banner"),
            "should not report cookie banner on plain page"
        );
    }

    // --- modals ---

    #[test]
    fn test_modal_missing_aria_modal() {
        // dialog without modal=true property
        let tree = AXTree::from_nodes(vec![
            node("1", "WebArea", Some("Page")),
            node("2", "dialog", Some("Settings")),
        ]);
        let result = analyze_ux_heuristics(&tree);
        assert!(
            result
                .findings
                .iter()
                .any(|f| f.check == "modal_no_aria_modal"),
            "should flag dialog missing aria-modal"
        );
    }

    #[test]
    fn test_modal_with_aria_modal_ok() {
        let tree = AXTree::from_nodes(vec![
            node("1", "WebArea", Some("Page")),
            node_with_props(
                "2",
                "dialog",
                Some("Settings"),
                vec![AXProperty {
                    name: "modal".to_string(),
                    value: AXValue::Bool(true),
                }],
            ),
        ]);
        let result = analyze_ux_heuristics(&tree);
        assert!(
            !result
                .findings
                .iter()
                .any(|f| f.check == "modal_no_aria_modal"),
            "should not flag dialog that has aria-modal=true"
        );
    }

    #[test]
    fn test_multiple_dialogs_flagged() {
        let tree = AXTree::from_nodes(vec![
            node("1", "WebArea", Some("Page")),
            node_with_props(
                "2",
                "dialog",
                Some("A"),
                vec![AXProperty {
                    name: "modal".to_string(),
                    value: AXValue::Bool(true),
                }],
            ),
            node_with_props(
                "3",
                "dialog",
                Some("B"),
                vec![AXProperty {
                    name: "modal".to_string(),
                    value: AXValue::Bool(true),
                }],
            ),
        ]);
        let result = analyze_ux_heuristics(&tree);
        assert!(
            result
                .findings
                .iter()
                .any(|f| f.check == "multiple_dialogs"),
            "should flag multiple dialogs"
        );
    }

    // --- font diversity ---

    #[test]
    fn test_font_diversity_advisory_always_present() {
        let tree = AXTree::from_nodes(vec![node("1", "WebArea", Some("Page"))]);
        let result = analyze_ux_heuristics(&tree);
        assert!(
            result
                .findings
                .iter()
                .any(|f| f.check == "font_diversity_advisory"),
            "font diversity advisory should always be present"
        );
    }

    // --- CTAs ---

    #[test]
    fn test_no_cta_detected() {
        let tree = AXTree::from_nodes(vec![
            node("1", "WebArea", Some("Page")),
            node("2", "heading", Some("Welcome")),
        ]);
        let result = analyze_ux_heuristics(&tree);
        assert!(
            result.findings.iter().any(|f| f.check == "no_cta"),
            "should flag missing CTA"
        );
    }

    #[test]
    fn test_single_cta_ok() {
        let tree = AXTree::from_nodes(vec![
            node("1", "WebArea", Some("Page")),
            node("2", "button", Some("Get Started")),
        ]);
        let result = analyze_ux_heuristics(&tree);
        assert!(
            !result.findings.iter().any(|f| f.check == "no_cta"),
            "should not flag when CTA is present"
        );
        assert!(
            !result.findings.iter().any(|f| f.check == "too_many_ctas"),
            "should not flag too-many-ctas for a single CTA"
        );
    }

    #[test]
    fn test_too_many_ctas() {
        let nodes = vec![
            node("1", "WebArea", Some("Page")),
            node("2", "button", Some("Buy Now")),
            node("3", "link", Some("Sign Up")),
            node("4", "button", Some("Download")),
            node("5", "link", Some("Subscribe")),
            node("6", "button", Some("Register")),
        ];
        let tree = AXTree::from_nodes(nodes);
        let result = analyze_ux_heuristics(&tree);
        assert!(
            result.findings.iter().any(|f| f.check == "too_many_ctas"),
            "should flag 5+ CTAs"
        );
    }
}
