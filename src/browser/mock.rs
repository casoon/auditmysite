//! Mock browser session for CDP-free unit and integration tests.
//!
//! Use `MockAXSession::new(nodes)` to build a test AXTree without Chrome.

use crate::accessibility::{AXNode, AXTree};

/// A pre-built AXTree + raw HTML pair that replaces a real browser session in tests.
pub struct MockAXSession {
    pub tree: AXTree,
    pub html: String,
}

impl MockAXSession {
    pub fn new(nodes: Vec<AXNode>, html: impl Into<String>) -> Self {
        Self {
            tree: AXTree::from_nodes(nodes),
            html: html.into(),
        }
    }

    /// Minimal well-formed page: WebArea root, titled, no violations expected.
    pub fn minimal_valid() -> Self {
        Self::new(
            vec![AXNode {
                node_id: "1".to_string(),
                ignored: false,
                ignored_reasons: vec![],
                role: Some("WebArea".to_string()),
                name: Some("Test Page".to_string()),
                name_source: None,
                description: None,
                value: None,
                properties: vec![],
                child_ids: vec![],
                parent_id: None,
                backend_dom_node_id: None,
            }],
            "<html lang=\"en\"><head><title>Test Page</title></head><body></body></html>",
        )
    }

    /// Page with an image missing alt text — triggers 1.1.1.
    pub fn image_missing_alt() -> Self {
        Self::new(
            vec![
                AXNode {
                    node_id: "1".to_string(),
                    ignored: false,
                    ignored_reasons: vec![],
                    role: Some("WebArea".to_string()),
                    name: Some("Test Page".to_string()),
                    name_source: None,
                    description: None,
                    value: None,
                    properties: vec![],
                    child_ids: vec!["2".to_string()],
                    parent_id: None,
                    backend_dom_node_id: None,
                },
                AXNode {
                    node_id: "2".to_string(),
                    ignored: false,
                    ignored_reasons: vec![],
                    role: Some("image".to_string()),
                    name: None,
                    name_source: None,
                    description: None,
                    value: None,
                    properties: vec![],
                    child_ids: vec![],
                    parent_id: Some("1".to_string()),
                    backend_dom_node_id: None,
                },
            ],
            "<html><body><img src=\"photo.jpg\"></body></html>",
        )
    }
}
