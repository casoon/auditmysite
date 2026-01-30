//! AXTree Extractor - Extract Accessibility Tree via CDP
//!
//! Uses Chrome DevTools Protocol to extract the full Accessibility Tree.

use chromiumoxide::cdp::browser_protocol::accessibility::GetFullAxTreeParams;
use chromiumoxide::Page;
use tracing::{debug, info};

use super::tree::{AXNode, AXProperty, AXTree, AXValue, NameSource};
use crate::error::{AuditError, Result};

/// Extract the full Accessibility Tree from a page
///
/// # Arguments
/// * `page` - The chromiumoxide Page to extract from
///
/// # Returns
/// * `Ok(AXTree)` - The extracted accessibility tree
/// * `Err(AuditError)` - If extraction fails
pub async fn extract_ax_tree(page: &Page) -> Result<AXTree> {
    info!("Extracting Accessibility Tree...");

    // Request the full AX tree via CDP
    let params = GetFullAxTreeParams::default();
    let response = page
        .execute(params)
        .await
        .map_err(|e| AuditError::AXTreeExtractionFailed {
            reason: format!("CDP command failed: {}", e),
        })?;

    // Get nodes from response - serialize just the nodes array
    let nodes_json = serde_json::to_value(&response.nodes)
        .map_err(|e| AuditError::AXTreeExtractionFailed {
            reason: format!("JSON serialization failed: {}", e),
        })?;

    // Extract nodes from JSON
    let nodes = extract_nodes_from_json(&nodes_json)?;

    let tree = AXTree::from_nodes(nodes);
    info!(
        "Extracted AXTree with {} nodes (root: {:?})",
        tree.len(),
        tree.root_id
    );

    Ok(tree)
}

/// Extract nodes from the CDP JSON response
fn extract_nodes_from_json(json: &serde_json::Value) -> Result<Vec<AXNode>> {
    let nodes_array = json
        .as_array()
        .ok_or_else(|| AuditError::AXTreeExtractionFailed {
            reason: "No nodes array in response".to_string(),
        })?;

    debug!("Received {} nodes from CDP", nodes_array.len());

    let nodes: Vec<AXNode> = nodes_array
        .iter()
        .filter_map(|node| convert_json_node(node).ok())
        .collect();

    Ok(nodes)
}

/// Convert a JSON node to our AXNode format
fn convert_json_node(json: &serde_json::Value) -> Result<AXNode> {
    let node_id = json["nodeId"]
        .as_str()
        .unwrap_or_default()
        .to_string();

    let ignored = json["ignored"].as_bool().unwrap_or(false);

    // Extract role
    let role = json["role"]["value"]
        .as_str()
        .map(String::from);

    // Extract name
    let name = json["name"]["value"]
        .as_str()
        .map(String::from);

    // Extract name source
    let name_source = json["name"]["sources"]
        .as_array()
        .and_then(|sources| {
            sources.iter().find_map(|s| {
                if s["value"].is_null() {
                    return None;
                }
                match s["type"].as_str()? {
                    "attribute" => Some(NameSource::Attribute),
                    "relatedElement" => Some(NameSource::RelatedElement),
                    "contents" => Some(NameSource::Contents),
                    "placeholder" => Some(NameSource::Placeholder),
                    "title" => Some(NameSource::Title),
                    _ => None,
                }
            })
        });

    // Extract description
    let description = json["description"]["value"]
        .as_str()
        .map(String::from);

    // Extract value
    let value = json["value"]["value"]
        .as_str()
        .map(String::from);

    // Convert properties
    let properties = json["properties"]
        .as_array()
        .map(|props| {
            props
                .iter()
                .filter_map(|p| {
                    let name = p["name"].as_str()?.to_string();
                    let value = convert_json_value(&p["value"]);
                    value.map(|v| AXProperty { name, value: v })
                })
                .collect()
        })
        .unwrap_or_default();

    // Extract child IDs
    let child_ids = json["childIds"]
        .as_array()
        .map(|ids| {
            ids.iter()
                .filter_map(|id| id.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    // Extract parent ID
    let parent_id = json["parentId"]
        .as_str()
        .map(String::from);

    // Extract backend DOM node ID
    let backend_dom_node_id = json["backendDOMNodeId"]
        .as_i64();

    // Extract ignored reasons
    let ignored_reasons = json["ignoredReasons"]
        .as_array()
        .map(|reasons| {
            reasons
                .iter()
                .filter_map(|r| {
                    let name = r["name"].as_str()?.to_string();
                    let value = convert_json_value(&r["value"]);
                    value.map(|v| AXProperty { name, value: v })
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(AXNode {
        node_id,
        ignored,
        ignored_reasons,
        role,
        name,
        name_source,
        description,
        value,
        properties,
        child_ids,
        parent_id,
        backend_dom_node_id,
    })
}

/// Convert a JSON value to our AXValue format
fn convert_json_value(json: &serde_json::Value) -> Option<AXValue> {
    let value = &json["value"];

    if value.is_null() {
        return None;
    }

    Some(if let Some(b) = value.as_bool() {
        AXValue::Bool(b)
    } else if let Some(n) = value.as_i64() {
        AXValue::Int(n)
    } else if let Some(n) = value.as_f64() {
        AXValue::Float(n)
    } else if let Some(s) = value.as_str() {
        AXValue::String(s.to_string())
    } else {
        AXValue::String(value.to_string())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_json_node() {
        let json = serde_json::json!({
            "nodeId": "1",
            "ignored": false,
            "role": {"value": "image"},
            "name": {"value": "Test Image"},
        });

        let node = convert_json_node(&json).unwrap();
        assert_eq!(node.node_id, "1");
        assert!(!node.ignored);
        assert_eq!(node.role, Some("image".to_string()));
        assert_eq!(node.name, Some("Test Image".to_string()));
    }

    #[test]
    fn test_name_source_conversion() {
        assert_eq!(NameSource::Attribute, NameSource::Attribute);
    }
}
