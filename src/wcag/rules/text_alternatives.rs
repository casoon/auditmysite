//! WCAG 1.1.1 - Non-text Content (Text Alternatives)
//!
//! All non-text content has a text alternative that serves the equivalent purpose.
//! This includes images, icons, charts, and other visual content.
//!
//! Seit #690 prüft der geteilte Bestand die beiden häufigsten Fälle selbst:
//! `<img>` als `images/alt-missing`/`images/alt-suspicious` und `<svg>` als
//! `svg/name-missing` (siehe `wcag::shared`). Hier bleibt, was dort kein
//! Gegenstück hat — Elemente mit `role="img"`, die weder `<img>` noch `<svg>`
//! sind, und die Icon-Heuristik.
//!
//! `<img>` ist im AX-Baum eindeutig: Chrome setzt die Eigenschaft `url`.
//! `<svg>` dagegen ist dort vom `<div role="img">` nicht zu unterscheiden
//! (beide Rolle `image`, keine `url`). Befunde an `<svg>` nimmt deshalb die
//! Pipeline heraus, die den DOM kennt — über [`is_svg_finding`], und nur,
//! wenn die geteilten Regeln tatsächlich gelaufen sind.

use std::collections::HashSet;

use crate::accessibility::{AXTree, NameSource};
use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation, WcagResults};

/// Rule metadata for 1.1.1
pub(super) const RULE_META: RuleMetadata = RuleMetadata {
    id: "1.1.1",
    name: "Non-text Content",
    level: WcagLevel::A,
    severity: Severity::High,
    description: "All non-text content has a text alternative that serves the equivalent purpose",
    help_url: "https://www.w3.org/WAI/WCAG22/Understanding/non-text-content.html",
    axe_id: "image-alt",
    tags: &["wcag2a", "wcag111", "cat.images"],
};

/// Check for missing text alternatives on image-role elements that are not
/// `<img>` (see module docs).
///
/// # Arguments
/// * `tree` - The accessibility tree to check
///
/// # Returns
/// Results with violations for images missing alt text
pub fn check_text_alternatives(tree: &AXTree) -> WcagResults {
    let mut results = WcagResults::new();

    // Alle Bildknoten ausser `<img>`: Die traegt `url` und gehoert zu
    // `images/*` im geteilten Bestand.
    let images: Vec<_> = tree
        .images()
        .into_iter()
        .filter(|n| !is_img_element(n))
        .collect();
    results.nodes_checked = images.len();

    // Node IDs already evaluated as images, so check_icons does not flag the same
    // role="img" node a second time (#487 false-positive double counting).
    let mut flagged_image_ids: HashSet<&str> = HashSet::new();

    for image in images {
        // Skip ignored nodes (they're intentionally hidden from AT)
        if image.ignored {
            continue;
        }

        // Skip explicitly decorative images: an empty name that comes from a name
        // attribute (alt="" / aria-label="") is intentional, not a missing
        // alternative. Lazy-load placeholders (data-URI src) keep the empty alt
        // but are not marked `ignored` in headless Chrome, so they would
        // otherwise be flagged en masse (#487).
        if is_decorative_empty_name(image) {
            results.passes += 1;
            continue;
        }

        if is_graphic_in_named_control(tree, image) {
            results.passes += 1;
            continue;
        }

        // Check if image has an accessible name
        if !image.has_name() {
            flagged_image_ids.insert(image.node_id.as_str());
            let violation = Violation::new(
                RULE_META.id,
                RULE_META.name,
                RULE_META.level,
                RULE_META.severity,
                "Image is missing alternative text",
                &image.node_id,
            )
            .with_role(image.role.clone())
            .with_fix(
                "Add an alt attribute describing the image content, or alt=\"\" if decorative",
            )
            .with_help_url(RULE_META.help_url)
            .with_rule_id(RULE_META.axe_id);

            results.add_violation(violation);
        } else {
            results.passes += 1;
        }
    }

    // Also check for other non-text content
    check_icons(tree, &flagged_image_ids, &mut results);

    results
}

/// Ob der Knoten ein `<img>` ist: Nur dort setzt Chrome die AX-Eigenschaft
/// `url`.
fn is_img_element(node: &crate::accessibility::AXNode) -> bool {
    node.has_property("url")
}

/// Ob ein Befund dieser Regel an einem `<svg>` steht.
///
/// `svg_nodes` sind die AX-Kennungen der `<svg>`-Elemente im DOM. Die
/// Pipeline nimmt solche Befunde heraus, sobald die geteilten Regeln gelaufen
/// sind — `svg/name-missing` meldet denselben Fall, mit derselben Ausnahme
/// für das Icon im benannten Link oder Button.
pub fn is_svg_finding(violation: &Violation, svg_nodes: &HashSet<String>) -> bool {
    violation.rule_id.as_deref() == Some(RULE_META.axe_id) && svg_nodes.contains(&violation.node_id)
}

/// True when the node has no accessible name but the (empty) name was supplied
/// by a name attribute such as `alt=""` or `aria-label=""` — i.e. the author
/// explicitly marked it decorative. A genuinely missing `alt` has no attribute
/// name source, so it stays flagged.
fn is_decorative_empty_name(node: &crate::accessibility::AXNode) -> bool {
    !node.has_name() && node.name.is_some() && node.name_source == Some(NameSource::Attribute)
}

/// A nameless graphic that is not an `<img>`, inside a link or button that
/// has a name of its own — the icon in "MIT@twitter" or "open search".
///
/// The control carries the text alternative; the graphic adds nothing a user
/// misses, so 1.1.1 is met. Only graphics without a `url` property qualify:
/// Chrome sets `url` on `<img>`, and an `<img>` without `alt` stays a failure
/// wherever it sits (F65) — reported by `images/alt-missing` since #690.
/// Inline `<svg>` has none; `svg/name-missing` applies the same exemption. An
/// element with an explicit `role="img"` looks the same in the tree and is
/// exempted too; that is the known imprecision.
///
/// Before, every icon in a named link was a High Level-A finding — seven on
/// www.mit.edu alone, capping the page at 89 by themselves (plan 47 review).
fn is_graphic_in_named_control(tree: &AXTree, node: &crate::accessibility::AXNode) -> bool {
    if node.has_name() || node.has_property("url") {
        return false;
    }
    let mut current = node.parent_id.as_deref().and_then(|id| tree.get_node(id));
    for _ in 0..64 {
        let Some(ancestor) = current else {
            return false;
        };
        if matches!(ancestor.role.as_deref(), Some("link") | Some("button")) {
            return ancestor.has_name();
        }
        current = ancestor
            .parent_id
            .as_deref()
            .and_then(|id| tree.get_node(id));
    }
    false
}

/// Check icon elements for text alternatives
fn check_icons(tree: &AXTree, flagged_image_ids: &HashSet<&str>, results: &mut WcagResults) {
    // Icons might have role="img" but different implementation
    for node in tree.iter() {
        if node.ignored {
            continue;
        }

        // Skip nodes already flagged by the main image loop (#487 dedup) and
        // explicitly-decorative empty-name nodes.
        if flagged_image_ids.contains(node.node_id.as_str())
            || is_img_element(node)
            || is_decorative_empty_name(node)
            || is_graphic_in_named_control(tree, node)
        {
            continue;
        }

        // Check for icon patterns
        let is_icon = node.role.as_deref() == Some("img")
            || node
                .name
                .as_ref()
                .is_some_and(|n| n.contains("icon") || n.contains("Icon"));

        if is_icon && !node.has_name() {
            // Only flag if it seems meaningful (not decorative)
            let likely_decorative = node.get_property_str("hidden").is_some();

            if !likely_decorative {
                let violation = Violation::new(
                    RULE_META.id,
                    RULE_META.name,
                    RULE_META.level,
                    Severity::Medium,
                    "Icon element may need alternative text",
                    &node.node_id,
                )
                .with_role(node.role.clone())
                .with_fix(
                    "Add aria-label for meaningful icons, or aria-hidden=\"true\" for decorative",
                )
                .with_help_url(RULE_META.help_url)
                .with_rule_id(RULE_META.axe_id);

                results.add_violation(violation);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::AXNode;

    fn create_image_node(id: &str, name: Option<&str>) -> AXNode {
        AXNode {
            node_id: id.to_string(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some("image".to_string()),
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

    fn in_link(link_name: Option<&str>, image: AXNode) -> Vec<AXNode> {
        let mut link = create_image_node("L", link_name);
        link.role = Some("link".to_string());
        link.child_ids = vec![image.node_id.clone()];
        let mut image = image;
        image.parent_id = Some("L".to_string());
        vec![link, image]
    }

    /// An inline SVG icon in a named link is decorative: no finding.
    #[test]
    fn svg_icon_in_named_link_is_not_flagged() {
        let tree = AXTree::from_nodes(in_link(Some("MIT@twitter"), create_image_node("1", None)));
        let results = check_text_alternatives(&tree);
        assert!(results.violations.is_empty(), "{:?}", results.violations);
    }

    /// In a link without a name the icon is all there is — still a finding.
    #[test]
    fn svg_icon_in_unnamed_link_is_flagged() {
        let tree = AXTree::from_nodes(in_link(None, create_image_node("1", None)));
        let results = check_text_alternatives(&tree);
        assert_eq!(results.violations.len(), 1);
    }

    fn img_element(id: &str, name: Option<&str>) -> AXNode {
        let mut img = create_image_node(id, name);
        img.properties.push(crate::accessibility::AXProperty {
            name: "url".to_string(),
            value: crate::accessibility::AXValue::String("logo.png".to_string()),
        });
        img
    }

    /// Ein `<img>` (es traegt `url`) ohne alt meldet seit #690
    /// `images/alt-missing` aus dem geteilten Bestand -- hier nicht mehr,
    /// sonst stuende es doppelt im Bericht. Auch nicht ueber die
    /// Icon-Heuristik.
    #[test]
    fn img_element_is_left_to_the_shared_rules() {
        let mut icon = img_element("2", None);
        icon.role = Some("img".to_string());
        let tree = AXTree::from_nodes(vec![img_element("1", None), icon]);
        let results = check_text_alternatives(&tree);
        assert!(results.violations.is_empty(), "{:?}", results.violations);

        let tree = AXTree::from_nodes(in_link(Some("Home"), img_element("1", None)));
        assert!(check_text_alternatives(&tree).violations.is_empty());
    }

    /// Nur die Befunde dieser Regel an `<svg>`-Knoten sind abgeloest.
    #[test]
    fn svg_findings_are_recognised_by_node_and_rule() {
        let tree = AXTree::from_nodes(vec![create_image_node("1", None)]);
        let results = check_text_alternatives(&tree);
        let svg: HashSet<String> = ["1".to_string()].into();
        let andere: HashSet<String> = ["2".to_string()].into();
        assert!(is_svg_finding(&results.violations[0], &svg));
        assert!(!is_svg_finding(&results.violations[0], &andere));
        let fremd = results.violations[0].clone().with_rule_id("svg-img-alt");
        assert!(!is_svg_finding(&fremd, &svg));
    }

    #[test]
    fn test_image_with_alt() {
        let nodes = vec![create_image_node("1", Some("Company Logo"))];
        let tree = AXTree::from_nodes(nodes);
        let results = check_text_alternatives(&tree);

        assert_eq!(results.violations.len(), 0);
        assert_eq!(results.passes, 1);
    }

    #[test]
    fn test_image_without_alt() {
        let nodes = vec![create_image_node("1", None)];
        let tree = AXTree::from_nodes(nodes);
        let results = check_text_alternatives(&tree);

        assert_eq!(results.violations.len(), 1);
        assert_eq!(results.violations[0].rule, "1.1.1");
    }

    #[test]
    fn test_multiple_images() {
        let nodes = vec![
            create_image_node("1", Some("Logo")),
            create_image_node("2", None),
            create_image_node("3", Some("Banner")),
            create_image_node("4", None),
        ];
        let tree = AXTree::from_nodes(nodes);
        let results = check_text_alternatives(&tree);

        assert_eq!(results.violations.len(), 2);
        assert_eq!(results.passes, 2);
    }

    #[test]
    fn test_ignored_image_not_flagged() {
        let mut node = create_image_node("1", None);
        node.ignored = true;

        let tree = AXTree::from_nodes(vec![node]);
        let results = check_text_alternatives(&tree);

        // Ignored nodes should not be flagged
        assert_eq!(results.violations.len(), 0);
    }
}
