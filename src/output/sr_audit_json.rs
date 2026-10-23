use std::fs;
use std::path::Path;

use crate::error::{AuditError, Result};
use crate::screen_reader::SrAuditReport;

pub fn export_sr_audit(report: &SrAuditReport, path: &Path) -> Result<()> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent).map_err(|e| AuditError::FileError {
            path: parent.to_path_buf(),
            reason: e.to_string(),
        })?;
    }

    let json = serde_json::to_string_pretty(report).map_err(|e| AuditError::OutputError {
        reason: e.to_string(),
    })?;
    fs::write(path, json).map_err(|e| AuditError::FileError {
        path: path.to_path_buf(),
        reason: e.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use crate::accessibility::{AXNode, AXTree};
    use crate::screen_reader::build_sr_audit_report;

    use super::export_sr_audit;

    /// Recursively sorts object keys so this snapshot assertion is stable
    /// regardless of `serde_json`'s `preserve_order` feature — which the
    /// optional `ai-transparency` feature's `c2pa` dependency unconditionally
    /// enables (hardcoded in its own `Cargo.toml`, not one of its own
    /// optional features), changing `serde_json::Value::Object`'s backing map
    /// from a sorted `BTreeMap` to an insertion-order `IndexMap` for the
    /// *whole* binary via Cargo feature unification — not just this module.
    fn sort_json_keys(value: serde_json::Value) -> serde_json::Value {
        match value {
            serde_json::Value::Object(map) => {
                let mut entries: Vec<(String, serde_json::Value)> = map
                    .into_iter()
                    .map(|(k, v)| (k, sort_json_keys(v)))
                    .collect();
                entries.sort_by(|a, b| a.0.cmp(&b.0));
                serde_json::Value::Object(entries.into_iter().collect())
            }
            serde_json::Value::Array(arr) => {
                serde_json::Value::Array(arr.into_iter().map(sort_json_keys).collect())
            }
            other => other,
        }
    }

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

    #[test]
    fn exports_valid_screen_reader_json() {
        let tree = AXTree::from_nodes(vec![
            node("1", "WebArea", Some("Example"), vec!["2", "3"]),
            node("2", "main", Some("Inhalt"), vec![]),
            node("3", "link", Some("Hier"), vec![]),
        ]);
        let report = build_sr_audit_report(
            "https://example.com",
            chrono::DateTime::<chrono::Utc>::UNIX_EPOCH,
            &tree,
            "de",
            None,
        );
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("sr-audit.json");

        export_sr_audit(&report, &path).expect("export succeeds");

        let raw = std::fs::read_to_string(path).expect("read output");
        let mut json: serde_json::Value = serde_json::from_str(&raw).expect("valid json");
        json["timestamp"] = serde_json::Value::String("[timestamp]".into());
        json["tool_version"] = serde_json::Value::String("[version]".into());
        let json = sort_json_keys(json);
        insta::assert_json_snapshot!(json, @r###"
        {
          "bfsg_compliance": {
            "passed_criteria": [
              "1.3.1",
              "2.4.1",
              "2.4.6",
              "3.3.2",
              "4.1.2"
            ],
            "verdict": "non_compliant",
            "violations": [
              {
                "affected_node_ids": [
                  "3"
                ],
                "bfsg_reference": "§ 12 Nr. 3 BFSGV",
                "en_301_549_clause": "9.2.4.4",
                "fix_required": true,
                "wcag_criterion": "2.4.4"
              }
            ]
          },
          "issues": [
            {
              "affected_node_ids": [
                "3"
              ],
              "message": "Interactive name \"Hier\" is not meaningful without context.",
              "severity": "medium",
              "wcag_criterion": "2.4.4"
            },
            {
              "affected_node_ids": [],
              "message": "No header area (banner landmark) present. Screen reader users cannot reach the page header through landmark navigation; its content stays reachable in reading mode.",
              "severity": "medium",
              "wcag_criterion": "1.3.6"
            },
            {
              "affected_node_ids": [],
              "message": "No navigation landmark present. Screen reader users cannot jump to the navigation through landmark navigation; the links stay reachable in reading and tab order.",
              "severity": "medium",
              "wcag_criterion": "1.3.6"
            },
            {
              "affected_node_ids": [],
              "message": "No footer landmark (contentinfo) present. Screen reader users cannot reach the footer through landmark navigation; its content stays reachable in reading mode.",
              "severity": "medium",
              "wcag_criterion": "1.3.6"
            }
          ],
          "navigation_views": {
            "form_controls": [],
            "headings": [],
            "landmarks": [
              {
                "name": "Inhalt",
                "node_id": "2",
                "quality": "ok",
                "role": "main",
                "seq": 1
              }
            ],
            "links": [
              {
                "count": 1,
                "node_ids": [
                  "3"
                ],
                "quality": "non_descriptive",
                "seq_positions": [
                  2
                ],
                "text": "Hier"
              }
            ],
            "tables": []
          },
          "reading_sequence": [
            {
              "announcement": "Example, WebArea",
              "depth": 0,
              "description": null,
              "name": "Example",
              "node_id": "1",
              "role": "WebArea",
              "seq": 0,
              "states": [],
              "tab_stop": false,
              "value": null
            },
            {
              "announcement": "Inhalt, main",
              "depth": 1,
              "description": null,
              "name": "Inhalt",
              "node_id": "2",
              "role": "main",
              "seq": 1,
              "states": [],
              "tab_stop": false,
              "value": null
            },
            {
              "announcement": "Hier, link",
              "depth": 1,
              "description": null,
              "name": "Hier",
              "node_id": "3",
              "role": "link",
              "seq": 2,
              "states": [],
              "tab_stop": false,
              "value": null
            }
          ],
          "report_type": "screen_reader_audit",
          "schema_version": "1.0",
          "summary": {
            "audit_quality": "ok",
            "bfsg_violations": 1,
            "heading_quality_score": 100,
            "landmark_quality_score": 25,
            "name_quality_score": 0,
            "quality_score_context": {
              "higher_is_better": true,
              "maximum": 100,
              "meaning": "Structural quality of detected accessible names, landmarks, or headings; announced nodes and tab stops are counts, not quality scores.",
              "minimum": 0
            },
            "tab_stops": 0,
            "total_announced_nodes": 3
          },
          "timestamp": "[timestamp]",
          "tool_version": "[version]",
          "url": "https://example.com"
        }
        "###);
    }
}
