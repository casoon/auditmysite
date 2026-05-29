use std::collections::HashSet;

use crate::accessibility::{AXNode, AXTree, AXValue};

use super::types::{IgnoredReadingNode, ReadingItem};

/// Build the default screen reader reading order from an AXTree.
///
/// Ignored nodes are traversed for their visible descendants, but are not
/// emitted as reading items.
pub fn linearize(tree: &AXTree) -> Vec<ReadingItem> {
    linearize_with_ignored(tree).items
}

/// Build reading order and retain ignored nodes for diagnostics.
pub fn linearize_with_ignored(tree: &AXTree) -> LinearizedReadingOrder {
    let mut order = LinearizedReadingOrder::default();
    let Some(root_id) = tree.root_id.as_deref() else {
        return order;
    };

    let mut visited = HashSet::new();
    visit_node(tree, root_id, 0, &mut visited, &mut order);
    order
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LinearizedReadingOrder {
    pub items: Vec<ReadingItem>,
    pub ignored: Vec<IgnoredReadingNode>,
}

fn visit_node(
    tree: &AXTree,
    node_id: &str,
    depth: usize,
    visited: &mut HashSet<String>,
    order: &mut LinearizedReadingOrder,
) {
    if !visited.insert(node_id.to_string()) {
        return;
    }

    let Some(node) = tree.get_node(node_id) else {
        return;
    };

    let child_depth = if node.ignored {
        order.ignored.push(ignored_node(node, depth));
        depth
    } else {
        order
            .items
            .push(reading_item(node, order.items.len(), depth));
        depth + 1
    };

    for child_id in &node.child_ids {
        visit_node(tree, child_id, child_depth, visited, order);
    }
}

fn reading_item(node: &AXNode, seq: usize, depth: usize) -> ReadingItem {
    ReadingItem {
        seq,
        role: node.role.clone(),
        name: node.name.clone(),
        description: node.description.clone(),
        value: node.value.clone(),
        states: states(node),
        tab_stop: node.is_focusable(),
        depth,
        node_id: node.node_id.clone(),
    }
}

fn ignored_node(node: &AXNode, depth: usize) -> IgnoredReadingNode {
    IgnoredReadingNode {
        node_id: node.node_id.clone(),
        role: node.role.clone(),
        name: node.name.clone(),
        depth,
        reasons: node
            .ignored_reasons
            .iter()
            .map(|property| match property.value.as_str() {
                Some(value) if !value.trim().is_empty() => {
                    format!("{}={}", property.name, value)
                }
                _ => property.name.clone(),
            })
            .collect(),
    }
}

fn states(node: &AXNode) -> Vec<String> {
    node.properties
        .iter()
        .filter_map(|property| match &property.value {
            AXValue::Bool(value) if is_state_property(&property.name) => {
                if *value {
                    Some(property.name.clone())
                } else {
                    Some(format!("{}=false", property.name))
                }
            }
            AXValue::String(value) if is_state_property(&property.name) && !value.is_empty() => {
                Some(format!("{}={}", property.name, value))
            }
            AXValue::Int(value) if is_state_property(&property.name) => {
                Some(format!("{}={}", property.name, value))
            }
            _ => None,
        })
        .collect()
}

fn is_state_property(name: &str) -> bool {
    matches!(
        name,
        "expanded"
            | "checked"
            | "selected"
            | "required"
            | "invalid"
            | "disabled"
            | "pressed"
            | "level"
            | "autocomplete"
            | "haspopup"
    )
}

#[cfg(test)]
mod tests {
    use crate::accessibility::{AXNode, AXProperty, AXTree, AXValue};

    use super::{linearize, linearize_with_ignored};

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

    fn property(name: &str, value: AXValue) -> AXProperty {
        AXProperty {
            name: name.to_string(),
            value,
        }
    }

    #[test]
    fn linearize_uses_dfs_order_and_skips_ignored_nodes() {
        let mut ignored = node("2", "generic", None, vec!["3"]);
        ignored.ignored = true;
        ignored.ignored_reasons = vec![property("ariaHidden", AXValue::Bool(true))];

        let mut heading = node("3", "heading", Some("Willkommen"), vec![]);
        heading.properties = vec![property("level", AXValue::Int(1))];

        let mut button = node("4", "button", Some("Mehr erfahren"), vec![]);
        button.properties = vec![
            property("focusable", AXValue::Bool(true)),
            property("expanded", AXValue::Bool(false)),
            property("required", AXValue::Bool(true)),
        ];

        let tree = AXTree::from_nodes(vec![
            node("1", "WebArea", Some("Example"), vec!["2", "4"]),
            ignored,
            heading,
            button,
        ]);

        let items = linearize(&tree);

        assert_eq!(items.len(), 3);
        assert_eq!(items[0].node_id, "1");
        assert_eq!(items[0].depth, 0);
        assert_eq!(items[0].seq, 0);
        assert_eq!(items[1].node_id, "3");
        assert_eq!(items[1].depth, 1);
        assert_eq!(items[1].states, vec!["level=1"]);
        assert_eq!(items[2].node_id, "4");
        assert_eq!(items[2].depth, 1);
        assert_eq!(items[2].seq, 2);
        assert!(items[2].tab_stop);
        assert_eq!(items[2].states, vec!["expanded=false", "required"]);
    }

    #[test]
    fn linearize_with_ignored_retains_diagnostics() {
        let mut ignored = node("2", "none", None, vec![]);
        ignored.ignored = true;
        ignored.ignored_reasons = vec![property("presentational", AXValue::String("true".into()))];

        let tree = AXTree::from_nodes(vec![node("1", "WebArea", None, vec!["2"]), ignored]);
        let order = linearize_with_ignored(&tree);

        assert_eq!(order.items.len(), 1);
        assert_eq!(order.ignored.len(), 1);
        assert_eq!(order.ignored[0].node_id, "2");
        assert_eq!(order.ignored[0].depth, 1);
        assert_eq!(order.ignored[0].reasons, vec!["presentational=true"]);
    }

    #[test]
    fn linearize_returns_empty_for_empty_tree() {
        assert!(linearize(&AXTree::new()).is_empty());
    }
}
