//! Differences between two AXSnapshots — the basic primitive used by
//! interactive journey tests (modal open/close, disclosure expand, …).
//!
//! Phase 1 covers added/removed nodes, focus moves, title and URL changes.
//! Detailed property-level diffing (e.g. `aria-expanded` flips per node) is
//! intentionally deferred to Phase 2 — once we know which property changes
//! we actually care to surface.

use serde::{Deserialize, Serialize};

use super::snapshot::AXSnapshot;

/// Difference between two captured snapshots.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AXTreeDiff {
    /// AXTree node ids present in `after` but not in `before`.
    pub added: Vec<String>,
    /// AXTree node ids present in `before` but not in `after`.
    pub removed: Vec<String>,
    /// Per-node property changes (filled in Phase 2).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub property_changes: Vec<PropertyChange>,
    /// Focus moved between snapshots.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub focus_moved: Option<FocusMove>,
    /// `document.title` changed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title_changed: Option<(String, String)>,
    /// URL changed without a full page reload (SPA navigation indicator).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url_changed: Option<(String, String)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyChange {
    pub node_id: String,
    /// Property name, e.g. `"aria-expanded"`.
    pub property: String,
    pub before: String,
    pub after: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusMove {
    pub before: Option<i64>,
    pub after: Option<i64>,
}

/// ARIA state properties tracked for property-level diffs.
const TRACKED_PROPERTIES: &[&str] = &["expanded", "hidden", "selected", "invalid", "modal"];

impl AXTreeDiff {
    /// Compute the structural diff between two snapshots.
    /// Phase 2: adds property-level diffing for key ARIA state properties.
    pub fn between(before: &AXSnapshot, after: &AXSnapshot) -> Self {
        let mut diff = AXTreeDiff::default();

        if before.document_title != after.document_title {
            diff.title_changed =
                Some((before.document_title.clone(), after.document_title.clone()));
        }
        if before.url != after.url {
            diff.url_changed = Some((before.url.clone(), after.url.clone()));
        }
        if before.focus.active_backend_node_id != after.focus.active_backend_node_id {
            diff.focus_moved = Some(FocusMove {
                before: before.focus.active_backend_node_id,
                after: after.focus.active_backend_node_id,
            });
        }

        for id in after.tree.nodes.keys() {
            if !before.tree.nodes.contains_key(id) {
                diff.added.push(id.clone());
            }
        }
        for id in before.tree.nodes.keys() {
            if !after.tree.nodes.contains_key(id) {
                diff.removed.push(id.clone());
            }
        }
        diff.added.sort();
        diff.removed.sort();

        // Property-level diffing: track meaningful ARIA state changes.
        for (node_id, after_node) in &after.tree.nodes {
            if let Some(before_node) = before.tree.nodes.get(node_id) {
                for prop_name in TRACKED_PROPERTIES {
                    let before_val = before_node.get_property_bool(prop_name);
                    let after_val = after_node.get_property_bool(prop_name);
                    if before_val != after_val {
                        diff.property_changes.push(PropertyChange {
                            node_id: node_id.clone(),
                            property: prop_name.to_string(),
                            before: before_val.map(|b| b.to_string()).unwrap_or_default(),
                            after: after_val.map(|b| b.to_string()).unwrap_or_default(),
                        });
                    }
                }
            }
        }

        diff
    }

    /// True when nothing structural changed between the two snapshots.
    pub fn is_empty(&self) -> bool {
        self.added.is_empty()
            && self.removed.is_empty()
            && self.property_changes.is_empty()
            && self.focus_moved.is_none()
            && self.title_changed.is_none()
            && self.url_changed.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::super::snapshot::FocusSnapshot;
    use super::super::tree::AXTree;
    use super::*;

    fn snap(label: &str, title: &str, url: &str, focus: Option<i64>) -> AXSnapshot {
        AXSnapshot::new(
            label,
            url,
            title,
            0,
            AXTree::new(),
            FocusSnapshot {
                active_backend_node_id: focus,
                ..Default::default()
            },
        )
    }

    #[test]
    fn empty_diff_when_identical() {
        let a = snap("a", "T", "https://x", Some(1));
        let b = snap("b", "T", "https://x", Some(1));
        assert!(AXTreeDiff::between(&a, &b).is_empty());
    }

    #[test]
    fn detects_title_and_focus_change() {
        let a = snap("a", "Old", "https://x", Some(1));
        let b = snap("b", "New", "https://x", Some(2));
        let d = AXTreeDiff::between(&a, &b);
        assert_eq!(d.title_changed, Some(("Old".into(), "New".into())));
        assert_eq!(d.focus_moved.unwrap().after, Some(2));
        assert!(d.url_changed.is_none());
    }

    #[test]
    fn detects_spa_navigation() {
        let a = snap("a", "T", "https://x/one", Some(1));
        let b = snap("b", "T", "https://x/two", Some(1));
        let d = AXTreeDiff::between(&a, &b);
        assert!(d.url_changed.is_some());
    }
}
