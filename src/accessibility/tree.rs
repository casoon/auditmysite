//! Accessibility Tree (AXTree) data structures
//!
//! Represents Chrome's Accessibility Tree as extracted via CDP.
//! The AXTree provides semantic information about page elements.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The complete Accessibility Tree for a page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AXTree {
    /// All nodes in the tree, indexed by node_id
    pub nodes: HashMap<String, AXNode>,
    /// The root node ID
    pub root_id: Option<String>,
}

impl AXTree {
    /// Create a new empty AXTree
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            root_id: None,
        }
    }

    /// Build tree from a list of nodes
    pub fn from_nodes(nodes: Vec<AXNode>) -> Self {
        let mut tree = Self::new();

        for node in nodes {
            // First node is typically the root
            if tree.root_id.is_none() {
                tree.root_id = Some(node.node_id.clone());
            }
            tree.nodes.insert(node.node_id.clone(), node);
        }

        tree
    }

    /// Get a node by ID
    pub fn get_node(&self, node_id: &str) -> Option<&AXNode> {
        self.nodes.get(node_id)
    }

    /// Get the root node
    pub fn root(&self) -> Option<&AXNode> {
        self.root_id.as_ref().and_then(|id| self.nodes.get(id))
    }

    /// Iterate over all nodes
    pub fn iter(&self) -> impl Iterator<Item = &AXNode> {
        self.nodes.values()
    }

    /// Get all nodes with a specific role
    pub fn nodes_with_role(&self, role: &str) -> Vec<&AXNode> {
        self.nodes
            .values()
            .filter(|n| n.role.as_deref() == Some(role))
            .collect()
    }

    /// Get all image nodes
    pub fn images(&self) -> Vec<&AXNode> {
        self.nodes
            .values()
            .filter(|n| {
                matches!(n.role.as_deref(), Some("image") | Some("img"))
            })
            .collect()
    }

    /// Get all heading nodes
    pub fn headings(&self) -> Vec<&AXNode> {
        self.nodes
            .values()
            .filter(|n| {
                matches!(
                    n.role.as_deref(),
                    Some("heading")
                )
            })
            .collect()
    }

    /// Get all form control nodes (excluding buttons, which are checked separately)
    pub fn form_controls(&self) -> Vec<&AXNode> {
        self.nodes
            .values()
            .filter(|n| {
                matches!(
                    n.role.as_deref(),
                    Some("textbox")
                        | Some("checkbox")
                        | Some("radio")
                        | Some("combobox")
                        | Some("listbox")
                        | Some("spinbutton")
                        | Some("slider")
                        | Some("searchbox")
                )
            })
            .collect()
    }

    /// Get all link nodes
    pub fn links(&self) -> Vec<&AXNode> {
        self.nodes_with_role("link")
    }

    /// Count total nodes
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Check if tree is empty
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

impl Default for AXTree {
    fn default() -> Self {
        Self::new()
    }
}

/// A single node in the Accessibility Tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AXNode {
    /// Unique identifier for this node
    pub node_id: String,
    /// Whether this node is ignored for accessibility
    #[serde(default)]
    pub ignored: bool,
    /// Reasons why this node is ignored
    #[serde(default)]
    pub ignored_reasons: Vec<AXProperty>,
    /// The accessibility role (e.g., "button", "heading", "image")
    pub role: Option<String>,
    /// The accessible name (what screen readers announce)
    pub name: Option<String>,
    /// Source of the accessible name
    pub name_source: Option<NameSource>,
    /// The accessible description
    pub description: Option<String>,
    /// The accessible value (for form controls)
    pub value: Option<String>,
    /// Additional properties
    #[serde(default)]
    pub properties: Vec<AXProperty>,
    /// Child node IDs
    #[serde(default)]
    pub child_ids: Vec<String>,
    /// Parent node ID
    pub parent_id: Option<String>,
    /// Backend DOM node ID (for correlation with DOM)
    pub backend_dom_node_id: Option<i64>,
}

impl AXNode {
    /// Check if this node has an accessible name
    pub fn has_name(&self) -> bool {
        self.name.as_ref().is_some_and(|n| !n.trim().is_empty())
    }

    /// Check if this node is focusable
    pub fn is_focusable(&self) -> bool {
        self.get_property_bool("focusable").unwrap_or(false)
    }

    /// Check if this node is interactive
    pub fn is_interactive(&self) -> bool {
        matches!(
            self.role.as_deref(),
            Some("button")
                | Some("link")
                | Some("textbox")
                | Some("checkbox")
                | Some("radio")
                | Some("combobox")
                | Some("menuitem")
                | Some("tab")
        ) || self.is_focusable()
    }

    /// Get heading level (1-6) if this is a heading
    pub fn heading_level(&self) -> Option<u8> {
        if self.role.as_deref() != Some("heading") {
            return None;
        }

        self.get_property_int("level")
            .map(|l| l.clamp(1, 6) as u8)
    }

    /// Get a boolean property value
    pub fn get_property_bool(&self, name: &str) -> Option<bool> {
        self.properties
            .iter()
            .find(|p| p.name == name)
            .and_then(|p| p.value.as_bool())
    }

    /// Get an integer property value
    pub fn get_property_int(&self, name: &str) -> Option<i64> {
        self.properties
            .iter()
            .find(|p| p.name == name)
            .and_then(|p| p.value.as_int())
    }

    /// Get a string property value
    pub fn get_property_str(&self, name: &str) -> Option<&str> {
        self.properties
            .iter()
            .find(|p| p.name == name)
            .and_then(|p| p.value.as_str())
    }

    /// Check if the node has a specific role
    pub fn has_role(&self, role: &str) -> bool {
        self.role.as_deref() == Some(role)
    }

    /// Get the required property (for form validation)
    pub fn is_required(&self) -> bool {
        self.get_property_bool("required").unwrap_or(false)
    }

    /// Get the invalid property (for form validation)
    pub fn is_invalid(&self) -> bool {
        self.get_property_bool("invalid").unwrap_or(false)
    }
}

/// Source of an accessible name
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NameSource {
    /// Name from attribute (aria-label, alt, title)
    Attribute,
    /// Name from associated label element
    RelatedElement,
    /// Name from content/children
    Contents,
    /// Name from placeholder
    Placeholder,
    /// Name from title attribute
    Title,
}

/// A property of an AXNode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AXProperty {
    /// Property name
    pub name: String,
    /// Property value
    pub value: AXValue,
}

/// Value of an AXProperty
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AXValue {
    /// Boolean value
    Bool(bool),
    /// Integer value
    Int(i64),
    /// Float value
    Float(f64),
    /// String value
    String(String),
    /// Related node
    Node { related_nodes: Vec<RelatedNode> },
    /// List of values
    List(Vec<AXValue>),
}

impl AXValue {
    /// Get as boolean if applicable
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            AXValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Get as integer if applicable
    pub fn as_int(&self) -> Option<i64> {
        match self {
            AXValue::Int(i) => Some(*i),
            AXValue::Float(f) => Some(*f as i64),
            _ => None,
        }
    }

    /// Get as string if applicable
    pub fn as_str(&self) -> Option<&str> {
        match self {
            AXValue::String(s) => Some(s),
            _ => None,
        }
    }
}

/// A related node reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedNode {
    /// The related node's backend DOM node ID
    pub backend_dom_node_id: Option<i64>,
    /// The related node's IDREF
    pub idref: Option<String>,
    /// Text content of the related node
    pub text: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_node(id: &str, role: &str, name: Option<&str>) -> AXNode {
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

    #[test]
    fn test_axtree_from_nodes() {
        let nodes = vec![
            create_test_node("1", "WebArea", Some("Page")),
            create_test_node("2", "heading", Some("Title")),
            create_test_node("3", "image", None),
        ];

        let tree = AXTree::from_nodes(nodes);
        assert_eq!(tree.len(), 3);
        assert_eq!(tree.root_id, Some("1".to_string()));
    }

    #[test]
    fn test_axtree_images() {
        let nodes = vec![
            create_test_node("1", "WebArea", Some("Page")),
            create_test_node("2", "image", Some("Logo")),
            create_test_node("3", "image", None),
            create_test_node("4", "heading", Some("Title")),
        ];

        let tree = AXTree::from_nodes(nodes);
        let images = tree.images();
        assert_eq!(images.len(), 2);
    }

    #[test]
    fn test_axtree_headings() {
        let nodes = vec![
            create_test_node("1", "WebArea", Some("Page")),
            create_test_node("2", "heading", Some("Title")),
            create_test_node("3", "heading", Some("Subtitle")),
        ];

        let tree = AXTree::from_nodes(nodes);
        let headings = tree.headings();
        assert_eq!(headings.len(), 2);
    }

    #[test]
    fn test_axnode_has_name() {
        let node_with_name = create_test_node("1", "image", Some("Logo"));
        let node_without_name = create_test_node("2", "image", None);
        let node_empty_name = create_test_node("3", "image", Some("  "));

        assert!(node_with_name.has_name());
        assert!(!node_without_name.has_name());
        assert!(!node_empty_name.has_name());
    }

    #[test]
    fn test_axnode_heading_level() {
        let mut heading = create_test_node("1", "heading", Some("Title"));
        heading.properties.push(AXProperty {
            name: "level".to_string(),
            value: AXValue::Int(2),
        });

        assert_eq!(heading.heading_level(), Some(2));

        let non_heading = create_test_node("2", "paragraph", Some("Text"));
        assert_eq!(non_heading.heading_level(), None);
    }
}
