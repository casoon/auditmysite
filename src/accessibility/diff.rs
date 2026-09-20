//! Die Differenz zweier [`AXSnapshot`]s — das Primitiv, mit dem der
//! Journey-Layer eine Interaktion bewertet: aufnehmen, handeln, aufnehmen,
//! vergleichen.
//!
//! Erfasst werden hinzugekommene und verschwundene Knoten, Änderungen an
//! ARIA-Zustandseigenschaften je Knoten, Fokusbewegungen sowie Titel- und
//! URL-Wechsel.
//!
//! # Knotenidentität
//!
//! Verglichen wird über die **Backend-Node-ID**, nicht über die
//! AXTree-Knotenkennung. Chrome vergibt `nodeId` je `getFullAXTree`-Aufruf
//! neu; ein Vergleich darüber würde bei jedem zweiten Aufruf den halben Baum
//! als „hinzugekommen" und „verschwunden" melden und keine einzige
//! Eigenschaftsänderung finden. Die Backend-ID zeigt dagegen auf den
//! DOM-Knoten und bleibt über die Lebensdauer der Seite stabil. Nur Knoten
//! ohne Backend-ID fallen auf die AXTree-Kennung zurück.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::snapshot::AXSnapshot;
use super::tree::AXNode;

/// Stabile Kennung eines Knotens über zwei Aufnahmen hinweg.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum NodeKey {
    /// Der Regelfall: zeigt auf den DOM-Knoten, stabil über Aufnahmen.
    Backend(i64),
    /// Rückfall für Knoten ohne DOM-Gegenstück.
    Ax(String),
}

fn key_of(node: &AXNode) -> NodeKey {
    match node.backend_dom_node_id {
        Some(id) => NodeKey::Backend(id),
        None => NodeKey::Ax(node.node_id.clone()),
    }
}

fn index(snapshot: &AXSnapshot) -> HashMap<NodeKey, &AXNode> {
    snapshot
        .tree
        .iter_all()
        .map(|node| (key_of(node), node))
        .collect()
}

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
    /// AXTree-Kennung aus der *späteren* Aufnahme. Nur zur Anzeige — die
    /// Zuordnung läuft über `backend_node_id`.
    pub node_id: String,
    /// Backend-Node-ID, sofern der Knoten ein DOM-Gegenstück hat. Der
    /// Schlüssel, über den ein Aufrufer eine Änderung dem Element zuordnet,
    /// das er bedient hat.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub backend_node_id: Option<i64>,
    /// Property name, e.g. `"expanded"`.
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

        let before_index = index(before);
        let after_index = index(after);

        for (key, node) in &after_index {
            if !before_index.contains_key(key) {
                diff.added.push(node.node_id.clone());
            }
        }
        for (key, node) in &before_index {
            if !after_index.contains_key(key) {
                diff.removed.push(node.node_id.clone());
            }
        }
        diff.added.sort();
        diff.removed.sort();

        for (key, after_node) in &after_index {
            let Some(before_node) = before_index.get(key) else {
                continue;
            };
            for prop_name in TRACKED_PROPERTIES {
                let before_val = before_node.get_property_bool(prop_name);
                let after_val = after_node.get_property_bool(prop_name);
                if before_val != after_val {
                    diff.property_changes.push(PropertyChange {
                        node_id: after_node.node_id.clone(),
                        backend_node_id: after_node.backend_dom_node_id,
                        property: prop_name.to_string(),
                        before: before_val.map(|b| b.to_string()).unwrap_or_default(),
                        after: after_val.map(|b| b.to_string()).unwrap_or_default(),
                    });
                }
            }
        }
        diff.property_changes.sort_by(|a, b| {
            (a.backend_node_id, &a.property).cmp(&(b.backend_node_id, &b.property))
        });

        diff
    }

    /// Die Änderung einer verfolgten Eigenschaft an genau dem Knoten, den der
    /// Aufrufer bedient hat.
    ///
    /// Das ist der Unterschied zwischen „irgendwo auf der Seite hat sich
    /// `expanded` geändert" und „der geklickte Auslöser hat sich geändert".
    pub fn property_change_for(
        &self,
        backend_node_id: i64,
        property: &str,
    ) -> Option<&PropertyChange> {
        self.property_changes
            .iter()
            .find(|c| c.backend_node_id == Some(backend_node_id) && c.property == property)
    }

    /// Ob im Accessibility-Tree Knoten hinzugekommen sind — der Hinweis
    /// darauf, dass durch die Handlung Inhalt wahrnehmbar geworden ist.
    pub fn has_additions(&self) -> bool {
        !self.added.is_empty()
    }

    /// Ob Knoten aus dem Accessibility-Tree verschwunden sind.
    pub fn has_removals(&self) -> bool {
        !self.removed.is_empty()
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
    use super::super::tree::{AXNode, AXTree};
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

    /// Baut einen AX-Knoten mit einer booleschen Eigenschaft.
    fn node(ax_id: &str, backend: i64, prop: Option<(&str, bool)>) -> AXNode {
        AXNode {
            node_id: ax_id.to_string(),
            ignored: false,
            ignored_reasons: Vec::new(),
            role: Some("button".to_string()),
            name: None,
            name_source: None,
            description: None,
            value: None,
            properties: prop
                .map(|(name, value)| {
                    vec![super::super::tree::AXProperty {
                        name: name.to_string(),
                        value: super::super::tree::AXValue::Bool(value),
                    }]
                })
                .unwrap_or_default(),
            child_ids: Vec::new(),
            parent_id: None,
            backend_dom_node_id: Some(backend),
        }
    }

    fn snap_with(label: &str, nodes: Vec<AXNode>) -> AXSnapshot {
        AXSnapshot::new(
            label,
            "https://x",
            "T",
            0,
            AXTree::from_nodes(nodes),
            FocusSnapshot::default(),
        )
    }

    /// Der Kern der Identitätskorrektur: Chrome vergibt `nodeId` je Abruf neu.
    /// Über die Backend-ID bleibt derselbe DOM-Knoten trotzdem derselbe —
    /// sonst wäre der halbe Baum „hinzugekommen" und „verschwunden".
    #[test]
    fn neue_ax_kennungen_erzeugen_keine_scheinaenderung() {
        let before = snap_with("a", vec![node("ax-1", 42, Some(("expanded", false)))]);
        let after = snap_with("b", vec![node("ax-999", 42, Some(("expanded", false)))]);

        let d = AXTreeDiff::between(&before, &after);
        assert!(d.added.is_empty(), "added: {:?}", d.added);
        assert!(d.removed.is_empty(), "removed: {:?}", d.removed);
        assert!(d.property_changes.is_empty());
    }

    #[test]
    fn eigenschaftswechsel_wird_dem_richtigen_knoten_zugeordnet() {
        let before = snap_with(
            "a",
            vec![
                node("ax-1", 42, Some(("expanded", false))),
                node("ax-2", 77, Some(("expanded", false))),
            ],
        );
        // Nur Knoten 77 klappt auf; 42 bekommt zusätzlich eine neue Kennung.
        let after = snap_with(
            "b",
            vec![
                node("ax-9", 42, Some(("expanded", false))),
                node("ax-8", 77, Some(("expanded", true))),
            ],
        );

        let d = AXTreeDiff::between(&before, &after);
        assert_eq!(d.property_changes.len(), 1);
        assert!(d.property_change_for(42, "expanded").is_none());
        let change = d
            .property_change_for(77, "expanded")
            .expect("Änderung an 77");
        assert_eq!(change.before, "false");
        assert_eq!(change.after, "true");
    }

    #[test]
    fn neuer_inhalt_erscheint_als_zugang() {
        let before = snap_with("a", vec![node("ax-1", 42, None)]);
        let after = snap_with("b", vec![node("ax-1", 42, None), node("ax-2", 43, None)]);

        let d = AXTreeDiff::between(&before, &after);
        assert!(d.has_additions());
        assert!(!d.has_removals());
    }

    #[test]
    fn detects_spa_navigation() {
        let a = snap("a", "T", "https://x/one", Some(1));
        let b = snap("b", "T", "https://x/two", Some(1));
        let d = AXTreeDiff::between(&a, &b);
        assert!(d.url_changed.is_some());
    }
}
