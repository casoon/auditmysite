//! AXTree Extractor - Extract Accessibility Tree via CDP
//!
//! Uses Chrome DevTools Protocol to extract the full Accessibility Tree.

use std::collections::HashMap;
use std::time::Duration;

use chromiumoxide::cdp::browser_protocol::accessibility::GetFullAxTreeParams;
use chromiumoxide::Page;
use tracing::{debug, info, warn};

use crate::error::{AuditError, Result};
use a11y_perception::{AXNode, AXProperty, AXTree, AXValue, NameSource, RelatedNode};

/// Extract the full Accessibility Tree from a page
///
/// # Arguments
/// * `page` - The chromiumoxide Page to extract from
///
/// # Returns
/// * `Ok(AXTree)` - The extracted accessibility tree
/// * `Err(AuditError)` - If extraction fails
pub async fn extract_ax_tree(page: &Page) -> Result<AXTree> {
    Ok(extract_ax_tree_with_name_sources(page).await?.0)
}

/// Chrome's winning name source per backend DOM node, in Chrome's own terms
/// (see `name_source_label`).
///
/// `a11y_perception::NameSource` folds `aria-label`, `alt` and `value` into
/// `Attribute` and `aria-labelledby` and `<label>` into `RelatedElement`; the
/// accname differential needs them apart. The coarse enum stays as it is —
/// the WCAG rules read it — and the detail travels next to the tree.
pub type ChromeNameSources = HashMap<i64, String>;

/// Like [`extract_ax_tree`], plus Chrome's detailed name source per backend
/// DOM node — from the same `getFullAXTree` response, no second round trip.
pub async fn extract_ax_tree_with_name_sources(page: &Page) -> Result<(AXTree, ChromeNameSources)> {
    info!("Extracting Accessibility Tree...");
    fetch_ax_tree(page, GetFullAxTreeParams::default()).await
}

/// The AX tree of one child frame's document.
///
/// `getFullAXTree` without `frameId` returns the main frame only; the content
/// of an iframe is a separate tree. Works for frames rendered in the page's
/// own process — an out-of-process (site-isolated) frame is not reachable
/// through the page session.
pub async fn extract_frame_ax_tree(page: &Page, frame_id: &str) -> Result<AXTree> {
    let params = GetFullAxTreeParams::builder()
        .frame_id(frame_id.to_string())
        .build();
    Ok(fetch_ax_tree(page, params).await?.0)
}

async fn fetch_ax_tree(
    page: &Page,
    params: GetFullAxTreeParams,
) -> Result<(AXTree, ChromeNameSources)> {
    // Request the full AX tree via CDP.
    // Some pages (WAF challenges, heavy SPAs) never respond to getFullAXTree — cap at 60s.
    let response = tokio::time::timeout(Duration::from_secs(60), page.execute(params))
        .await
        .map_err(|_| AuditError::AXTreeExtractionFailed {
            reason: "AX tree extraction timed out after 60s".to_string(),
        })?
        .map_err(|e| AuditError::AXTreeExtractionFailed {
            reason: format!("CDP command failed: {}", e),
        })?;

    // Get nodes from response - serialize just the nodes array
    let nodes_json =
        serde_json::to_value(&response.nodes).map_err(|e| AuditError::AXTreeExtractionFailed {
            reason: format!("JSON serialization failed: {}", e),
        })?;

    let nodes = extract_nodes_from_json(&nodes_json)?;
    let sources = extract_name_sources_from_json(&nodes_json);

    let tree = AXTree::from_nodes(nodes);
    info!(
        "Extracted AXTree with {} nodes (root: {:?})",
        tree.len(),
        tree.root_id
    );

    Ok((tree, sources))
}

/// Collects the detailed winning name source of every node that has one and
/// is tied to a DOM node.
fn extract_name_sources_from_json(json: &serde_json::Value) -> ChromeNameSources {
    json.as_array()
        .into_iter()
        .flatten()
        .filter_map(|node| {
            let backend = node["backendDOMNodeId"].as_i64()?;
            let label = name_source_label(winning_name_source(&node["name"])?)?;
            Some((backend, label))
        })
        .collect()
}

/// The source Chrome took the name from: the first entry of `name.sources`
/// that carries a value. Chrome lists the sources in precedence order and
/// flags every later source with a value as `superseded`; an empty value
/// still wins (`alt=""` names an image with the empty string).
fn winning_name_source(name: &serde_json::Value) -> Option<&serde_json::Value> {
    name["sources"]
        .as_array()?
        .iter()
        .find(|s| !s["value"].is_null())
}

/// Chrome's name source as a short label, taken from what CDP reports
/// rather than inferred from the DOM:
///
/// - `attribute` → the attribute (`aria-label`, `alt`, `title`, `value`, …)
/// - `relatedElement` → `aria-labelledby`, or the native source: `labelfor`
///   and `labelwrapped` become `label`; an SVG `<title>` child becomes
///   `title-element` (not to be confused with the `title` attribute); others
///   (`legend`, `tablecaption`, `figcaption`, …) keep Chrome's name
/// - `placeholder` (also `aria-placeholder`) → `placeholder`
/// - `contents` → `contents`
fn name_source_label(source: &serde_json::Value) -> Option<String> {
    let label = match source["type"].as_str()? {
        "attribute" => source["attribute"].as_str()?,
        "relatedElement" => match source["attribute"].as_str() {
            Some(attribute) => attribute,
            None => match source["nativeSource"].as_str()? {
                "labelfor" | "labelwrapped" | "label" => "label",
                "title" => "title-element",
                other => other,
            },
        },
        "placeholder" => "placeholder",
        other => other,
    };
    Some(label.to_string())
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
        .filter_map(|node| {
            convert_json_node(node)
                .map_err(|e| {
                    warn!("Skipping unparseable AX node: {}", e);
                    e
                })
                .ok()
        })
        .collect();

    Ok(nodes)
}

/// Convert a JSON node to our AXNode format
fn convert_json_node(json: &serde_json::Value) -> Result<AXNode> {
    let node_id = json["nodeId"].as_str().unwrap_or_default().to_string();

    let ignored = json["ignored"].as_bool().unwrap_or(false);

    // Extract role
    let role = json["role"]["value"].as_str().map(String::from);

    // Extract name
    let name = json["name"]["value"].as_str().map(String::from);

    // Extract name source.
    //
    // CDP never emits a top-level `type: "title"` source — confirmed live: a
    // title-derived name arrives as `type: "attribute"` with a nested
    // `attribute: "title"` field (the same top-level `type` a real
    // `aria-label` source also uses), so matching only on the top-level
    // `type` string made `NameSource::Title` permanently unreachable and
    // broke `label_title_only.rs`'s detection (#566). Disambiguate the two
    // "attribute" cases by that nested field before falling back to the
    // generic `Attribute` source.
    let name_source = json["name"]["sources"].as_array().and_then(|sources| {
        sources.iter().find_map(|s| {
            if s["value"].is_null() {
                return None;
            }
            match s["type"].as_str()? {
                "attribute" if s["attribute"].as_str() == Some("title") => Some(NameSource::Title),
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
    let description = json["description"]["value"].as_str().map(String::from);

    // Extract value. Sliders, spinbuttons, progressbars and meters carry a
    // number here, not a string; keep it in its string form (#656).
    let value = match &json["value"]["value"] {
        serde_json::Value::String(s) => Some(s.clone()),
        v @ (serde_json::Value::Number(_) | serde_json::Value::Bool(_)) => Some(v.to_string()),
        _ => None,
    };

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
    let parent_id = json["parentId"].as_str().map(String::from);

    // Extract backend DOM node ID
    let backend_dom_node_id = json["backendDOMNodeId"].as_i64();

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
    // CDP sends relationship attributes (aria-controls, aria-owns, etc.) as
    // {"type": "idref"/"idrefList", "relatedNodes": [...], "value": null}.
    // Handle these before checking "value", which is always null for references.
    if let Some(related) = json["relatedNodes"].as_array() {
        let nodes: Vec<RelatedNode> = related
            .iter()
            .map(|n| RelatedNode {
                backend_dom_node_id: n["backendDOMNodeId"].as_i64(),
                idref: n["idref"].as_str().map(String::from),
                text: n["text"].as_str().map(String::from),
            })
            .collect();
        if !nodes.is_empty() {
            return Some(AXValue::Node {
                related_nodes: nodes,
            });
        }
    }

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

    /// Chrome sends a slider's, spinbutton's or progressbar's value as a
    /// JSON number; it used to be dropped (#656).
    #[test]
    fn numeric_and_boolean_values_are_kept_as_strings() {
        let value_of = |value: serde_json::Value| {
            let json = serde_json::json!({
                "nodeId": "1",
                "role": {"value": "slider"},
                "value": {"type": "number", "value": value},
            });
            convert_json_node(&json).unwrap().value
        };
        assert_eq!(value_of(serde_json::json!(50)), Some("50".into()));
        assert_eq!(value_of(serde_json::json!(0.25)), Some("0.25".into()));
        assert_eq!(value_of(serde_json::json!(true)), Some("true".into()));
        assert_eq!(value_of(serde_json::json!("Apple")), Some("Apple".into()));
        assert_eq!(value_of(serde_json::Value::Null), None);
    }

    #[test]
    fn test_name_source_conversion() {
        // Was a tautology (`assert_eq!(NameSource::Attribute,
        // NameSource::Attribute)`, always true regardless of the actual
        // conversion) — exercises `convert_json_node`'s `name.sources[].type`
        // → `NameSource` mapping for real (see judge `tautological-test`).
        let json = serde_json::json!({
            "nodeId": "1",
            "ignored": false,
            "name": {
                "value": "Submit",
                "sources": [
                    {"type": "placeholder", "value": null},
                    {"type": "attribute", "value": "Submit"},
                ],
            },
        });

        let node = convert_json_node(&json).unwrap();
        assert_eq!(node.name_source, Some(NameSource::Attribute));
    }

    #[test]
    fn test_title_derived_name_source_is_distinguished_from_generic_attribute() {
        // Real CDP traffic (confirmed live, #566): a title-derived name's
        // winning source has top-level `type: "attribute"`, same as
        // aria-label — the nested `attribute: "title"` field is the only
        // signal distinguishing the two.
        let json = serde_json::json!({
            "nodeId": "1",
            "ignored": false,
            "name": {
                "value": "Search the site",
                "sources": [
                    {"type": "relatedElement", "value": null, "attribute": "aria-labelledby"},
                    {"type": "attribute", "value": null, "attribute": "aria-label"},
                    {"type": "relatedElement", "value": null, "nativeSource": "label"},
                    {"type": "attribute", "value": "Search the site", "attribute": "title"},
                ],
            },
        });

        let node = convert_json_node(&json).unwrap();
        assert_eq!(node.name_source, Some(NameSource::Title));
    }

    /// Sources as Chrome reports them (captured live from `getFullAXTree`).
    fn name_with_sources(sources: serde_json::Value) -> serde_json::Value {
        serde_json::json!({ "value": "x", "sources": sources })
    }

    #[test]
    fn detailed_name_source_labels_follow_cdp() {
        let cases = [
            (
                serde_json::json!([
                    {"type": "attribute", "attribute": "aria-label", "value": {"value": "AL"}},
                    {"type": "contents", "value": {"value": "c"}, "superseded": true},
                ]),
                "aria-label",
            ),
            (
                serde_json::json!([
                    {"type": "relatedElement", "attribute": "aria-labelledby", "value": {"value": "L"}},
                    {"type": "attribute", "attribute": "aria-label", "value": {"value": "x"}, "superseded": true},
                ]),
                "aria-labelledby",
            ),
            (
                serde_json::json!([{"type": "attribute", "attribute": "alt", "value": {"value": ""}}]),
                "alt",
            ),
            (
                serde_json::json!([
                    {"type": "relatedElement", "nativeSource": "labelfor", "value": {"value": "L"}},
                ]),
                "label",
            ),
            (
                serde_json::json!([
                    {"type": "relatedElement", "nativeSource": "labelwrapped", "value": {"value": "L"}},
                ]),
                "label",
            ),
            (
                serde_json::json!([
                    {"type": "relatedElement", "nativeSource": "title", "value": {"value": "T"}},
                ]),
                "title-element",
            ),
            (
                serde_json::json!([
                    {"type": "relatedElement", "nativeSource": "legend", "value": {"value": "Leg"}},
                ]),
                "legend",
            ),
            (
                serde_json::json!([{"type": "attribute", "attribute": "value", "value": {"value": "Send"}}]),
                "value",
            ),
            (
                serde_json::json!([
                    {"type": "placeholder", "attribute": "placeholder", "value": {"value": "PH"}},
                ]),
                "placeholder",
            ),
            (
                serde_json::json!([
                    {"type": "relatedElement", "attribute": "aria-labelledby", "invalid": true},
                    {"type": "contents", "value": {"value": "c"}},
                ]),
                "contents",
            ),
        ];
        for (sources, expected) in cases {
            let name = name_with_sources(sources);
            let label = winning_name_source(&name).and_then(name_source_label);
            assert_eq!(label.as_deref(), Some(expected), "{name}");
        }
    }

    /// The detail travels next to the tree; the coarse `NameSource` the WCAG
    /// rules read (`text_alternatives::is_decorative_empty_name`,
    /// `accessible_name`, `instructions`) must not change.
    #[test]
    fn detailed_sources_leave_the_coarse_name_source_unchanged() {
        let nodes = serde_json::json!([
            {
                "nodeId": "1", "backendDOMNodeId": 10,
                "name": name_with_sources(serde_json::json!([
                    {"type": "attribute", "attribute": "alt", "value": {"value": ""}},
                ])),
            },
            {
                "nodeId": "2", "backendDOMNodeId": 11,
                "name": name_with_sources(serde_json::json!([
                    {"type": "relatedElement", "nativeSource": "labelfor", "value": {"value": "L"}},
                ])),
            },
            {
                "nodeId": "3", "backendDOMNodeId": 12,
                "name": name_with_sources(serde_json::json!([
                    {"type": "attribute", "attribute": "aria-label", "value": {"value": "AL"}},
                ])),
            },
            { "nodeId": "4", "name": {"value": ""} },
        ]);

        let coarse: Vec<_> = extract_nodes_from_json(&nodes)
            .unwrap()
            .into_iter()
            .map(|n| n.name_source)
            .collect();
        assert_eq!(
            coarse,
            vec![
                Some(NameSource::Attribute),
                Some(NameSource::RelatedElement),
                Some(NameSource::Attribute),
                None,
            ]
        );

        let detail = extract_name_sources_from_json(&nodes);
        assert_eq!(detail.len(), 3);
        assert_eq!(detail[&10], "alt");
        assert_eq!(detail[&11], "label");
        assert_eq!(detail[&12], "aria-label");
    }
}
