//! Differentialtest: das eigene `accname`-Crate gegen Chromes native Berechnung.
//!
//! # Warum das ohne zusätzlichen Aufbau geht
//!
//! [`CdpDocument`] trägt beides nebeneinander — den DOM als Arena samt
//! Attributen und die nativen Accessibility-Werte, die Blink berechnet hat,
//! über die Backend-Node-ID verbunden. Damit lässt sich die eigene Namens- und
//! Rollenberechnung gegen eine unabhängige zweite Implementierung stellen:
//! kein Screenreader, keine VM, kein zweiter Browserlauf.
//!
//! # Chrome ist nicht die Spezifikation
//!
//! Eine Abweichung ist ein **Hinweis, kein Urteil**. Sie kann ein eigener
//! Fehler sein, eine bekannte Chrome-Eigenheit, oder eine Stelle, an der
//! [accname 1.2](https://w3c.github.io/accname/) mehrdeutig ist. Deshalb
//! klassifiziert dieses Modul nach Form der Abweichung und nach Namensquelle,
//! statt eine Trefferquote zu behaupten. Erst die Klassen sagen, wo
//! hingeschaut werden muss.
//!
//! # Was verglichen wird und was nicht
//!
//! Verglichen werden nur Elemente, die im Accessibility-Tree vorkommen und
//! dort nicht als `ignored` markiert sind — für alle anderen hat Chrome gar
//! keinen Namen berechnet, ein Vergleich wäre Rauschen. Beide Mengen werden
//! trotzdem gezählt, damit sichtbar bleibt, wie groß der nicht verglichene
//! Teil ist.

use std::collections::BTreeMap;

use a11y_dom::{ArenaNode, Document, NameSource, Node, NodeKind, Semantics};
use accname::IdIndex;
use serde::{Deserialize, Serialize};

use super::dom_document::CdpDocument;

/// Vorgabe für die Zahl der mitgeführten Beispiele je Abweichungsart.
/// Die Zählungen bleiben davon unberührt und immer vollständig.
pub const DEFAULT_MAX_SAMPLES: usize = 200;

/// Die Form einer Abweichung — unabhängig davon, wer recht hat.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Shape {
    /// Beide Seiten liefern denselben Text, aber mit unterschiedlichem
    /// Leerraum. Fast immer harmlos, wird getrennt geführt, damit es die
    /// Statistik der echten Abweichungen nicht dominiert.
    Whitespace,
    /// Chrome hat einen Namen, `accname` keinen.
    MissingLocally,
    /// `accname` hat einen Namen, Chrome keinen.
    MissingInChrome,
    /// Beide haben einen Namen, die Texte unterscheiden sich.
    Mismatch,
}

impl Shape {
    pub fn as_str(self) -> &'static str {
        match self {
            Shape::Whitespace => "whitespace",
            Shape::MissingLocally => "missing_locally",
            Shape::MissingInChrome => "missing_in_chrome",
            Shape::Mismatch => "mismatch",
        }
    }
}

/// Eine einzelne Namensabweichung, mit genug Kontext, um sie ohne den
/// laufenden Browser nachzuvollziehen.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NameDivergence {
    pub shape: Shape,
    /// Tagname des Elements.
    pub tag: String,
    /// Backend-Node-ID — der Rückweg in die Seite.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backend_node_id: Option<i64>,
    /// Knotenkennung im Accessibility-Tree.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ax_node_id: Option<String>,
    /// Wo Chrome den Namen hergenommen hat, sofern es das mitteilt. Die
    /// nützlichste Achse der Klassifikation, weil sie direkt auf den
    /// Abschnitt der Spezifikation zeigt.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name_source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chrome: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accname: Option<String>,
}

/// Eine Rollenabweichung. Siehe [`AccnameDiff::roles_not_comparable`] zur
/// Einschränkung.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleDivergence {
    pub tag: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backend_node_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ax_node_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chrome: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accname: Option<String>,
}

/// Das Ergebnis eines Differentiallaufs über ein Dokument.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AccnameDiff {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// Elementknoten im materialisierten DOM.
    pub elements_total: usize,
    /// Davon verglichen: im Accessibility-Tree vorhanden und nicht ignoriert.
    pub elements_compared: usize,
    /// Übersprungen, weil kein Gegenstück im Accessibility-Tree existiert.
    pub skipped_not_in_ax_tree: usize,
    /// Übersprungen, weil Chrome den Knoten als `ignored` führt.
    pub skipped_ignored: usize,

    /// Verglichene Knoten, bei denen beide Seiten zeichengleich sind
    /// (einschließlich „beide ohne Namen").
    pub names_equal: usize,
    /// Zahl der Abweichungen je [`Shape`] — vollständig, unabhängig davon,
    /// wie viele Beispiele mitgeführt werden.
    pub names_by_shape: BTreeMap<String, usize>,
    /// Zahl der Abweichungen je Namensquelle laut Chrome.
    pub names_by_source: BTreeMap<String, usize>,
    /// Beispiele, gedeckelt durch `max_samples` je [`Shape`].
    pub name_divergences: Vec<NameDivergence>,

    /// Rollen, bei denen beide Seiten übereinstimmen.
    pub roles_equal: usize,
    /// Rollen, die nicht vergleichbar sind, weil Chrome eine interne
    /// Bezeichnung liefert statt eines ARIA-Rollennamens (`RootWebArea`,
    /// `StaticText`, `LineBreak`, …). Erkannt an einem Großbuchstaben — eine
    /// Heuristik, keine Garantie, deshalb als eigene Zahl ausgewiesen.
    pub roles_not_comparable: usize,
    pub roles_divergent: usize,
    /// Beispiele, gedeckelt durch `max_samples`.
    pub role_divergences: Vec<RoleDivergence>,
}

impl AccnameDiff {
    /// Zahl aller Namensabweichungen, Leerraumfälle eingeschlossen.
    pub fn names_divergent(&self) -> usize {
        self.names_by_shape.values().sum()
    }

    /// Zahl der Namensabweichungen ohne die reinen Leerraumfälle — die Zahl,
    /// die tatsächlich Arbeit bedeutet.
    pub fn names_divergent_substantive(&self) -> usize {
        self.names_divergent()
            - self
                .names_by_shape
                .get(Shape::Whitespace.as_str())
                .copied()
                .unwrap_or(0)
    }
}

/// Vergleicht die eigene Berechnung mit Chromes nativen Werten.
///
/// `max_samples` deckelt nur die mitgeführten Beispiele je Abweichungsart;
/// alle Zählungen bleiben vollständig.
pub fn compare(doc: &CdpDocument, max_samples: usize) -> AccnameDiff {
    let mut out = AccnameDiff::default();
    let ids = IdIndex::build(doc.root());
    let mut name_samples: BTreeMap<Shape, usize> = BTreeMap::new();

    visit(
        doc,
        doc.root(),
        &ids,
        max_samples,
        &mut name_samples,
        &mut out,
    );
    out
}

fn visit<'n>(
    doc: &'n CdpDocument,
    node: ArenaNode<'n>,
    ids: &IdIndex<'n, ArenaNode<'n>>,
    max_samples: usize,
    name_samples: &mut BTreeMap<Shape, usize>,
    out: &mut AccnameDiff,
) {
    if node.kind() == NodeKind::Element {
        out.elements_total += 1;
        if doc.ax_node_id(node).is_none() {
            out.skipped_not_in_ax_tree += 1;
        } else if doc.is_ignored(node) {
            out.skipped_ignored += 1;
        } else {
            out.elements_compared += 1;
            compare_name(doc, node, ids, max_samples, name_samples, out);
            compare_role(doc, node, max_samples, out);
        }
    }

    for child in node.children() {
        visit(doc, child, ids, max_samples, name_samples, out);
    }
}

fn compare_name<'n>(
    doc: &'n CdpDocument,
    node: ArenaNode<'n>,
    ids: &IdIndex<'n, ArenaNode<'n>>,
    max_samples: usize,
    name_samples: &mut BTreeMap<Shape, usize>,
    out: &mut AccnameDiff,
) {
    let chrome = doc.accessible_name(node);
    let own = accname::name(node, ids);

    let chrome_n = chrome.as_deref().map(normalize).filter(|s| !s.is_empty());
    let own_n = own.as_deref().map(normalize).filter(|s| !s.is_empty());

    let Some(shape) = shape_of(chrome.as_deref(), own.as_deref(), &chrome_n, &own_n) else {
        out.names_equal += 1;
        return;
    };

    *out.names_by_shape
        .entry(shape.as_str().to_string())
        .or_insert(0) += 1;
    let source = doc.name_source(node).map(name_source_str);
    *out.names_by_source
        .entry(source.unwrap_or("unknown").to_string())
        .or_insert(0) += 1;

    let seen = name_samples.entry(shape).or_insert(0);
    if *seen < max_samples {
        *seen += 1;
        out.name_divergences.push(NameDivergence {
            shape,
            tag: node.local_name().to_string(),
            backend_node_id: doc.backend_node_id(node),
            ax_node_id: doc.ax_node_id(node).map(str::to_string),
            name_source: source.map(str::to_string),
            chrome,
            accname: own,
        });
    }
}

/// `None` heißt: keine Abweichung.
fn shape_of(
    chrome_raw: Option<&str>,
    own_raw: Option<&str>,
    chrome_n: &Option<String>,
    own_n: &Option<String>,
) -> Option<Shape> {
    match (chrome_n, own_n) {
        (None, None) => None,
        (Some(_), None) => Some(Shape::MissingLocally),
        (None, Some(_)) => Some(Shape::MissingInChrome),
        (Some(a), Some(b)) if a != b => Some(Shape::Mismatch),
        // Nach Normalisierung gleich: nur dann wirklich identisch, wenn auch
        // der Rohtext übereinstimmt.
        (Some(_), Some(_)) => (chrome_raw != own_raw).then_some(Shape::Whitespace),
    }
}

fn compare_role(doc: &CdpDocument, node: ArenaNode<'_>, max_samples: usize, out: &mut AccnameDiff) {
    let chrome = doc.role(node);
    let Some(chrome_role) = chrome.as_deref() else {
        out.roles_not_comparable += 1;
        return;
    };
    if chrome_role.chars().any(|c| c.is_ascii_uppercase()) {
        out.roles_not_comparable += 1;
        return;
    }

    let own = accname::role(node);
    if own == Some(chrome_role) {
        out.roles_equal += 1;
        return;
    }

    out.roles_divergent += 1;
    if out.role_divergences.len() < max_samples {
        out.role_divergences.push(RoleDivergence {
            tag: node.local_name().to_string(),
            backend_node_id: doc.backend_node_id(node),
            ax_node_id: doc.ax_node_id(node).map(str::to_string),
            chrome: chrome.clone(),
            accname: own.map(str::to_string),
        });
    }
}

/// Leerraum am Rand entfernen und im Inneren auf ein Leerzeichen
/// zusammenziehen. `char::is_whitespace` deckt auch das geschützte
/// Leerzeichen ab, das in echten Seiten regelmäßig vorkommt.
fn normalize(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn name_source_str(source: NameSource) -> &'static str {
    match source {
        NameSource::AriaLabel => "aria-label",
        NameSource::AriaLabelledBy => "aria-labelledby",
        NameSource::Label => "label",
        NameSource::Title => "title",
        NameSource::Alt => "alt",
        NameSource::Placeholder => "placeholder",
        NameSource::Contents => "contents",
        NameSource::Value => "value",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::dom_document::build_document;
    use crate::accessibility::{AXNode, AXTree};
    use chromiumoxide::cdp::browser_protocol::dom::Node as CdpNode;

    fn cdp(json: serde_json::Value) -> CdpNode {
        serde_json::from_value(json).expect("CDP-Knoten")
    }

    fn element(
        backend: i64,
        name: &str,
        attrs: &[&str],
        children: serde_json::Value,
    ) -> serde_json::Value {
        serde_json::json!({
            "nodeId": backend,
            "backendNodeId": backend,
            "nodeType": 1,
            "nodeName": name.to_uppercase(),
            "localName": name,
            "nodeValue": "",
            "attributes": attrs,
            "children": children,
        })
    }

    fn text(backend: i64, value: &str) -> serde_json::Value {
        serde_json::json!({
            "nodeId": backend,
            "backendNodeId": backend,
            "nodeType": 3,
            "nodeName": "#text",
            "localName": "",
            "nodeValue": value,
        })
    }

    fn document(body: serde_json::Value) -> CdpNode {
        cdp(serde_json::json!({
            "nodeId": 1,
            "backendNodeId": 1,
            "nodeType": 9,
            "nodeName": "#document",
            "localName": "",
            "nodeValue": "",
            "children": [
                element(2, "html", &["lang", "de"], serde_json::json!([
                    element(3, "body", &[], body),
                ]))
            ]
        }))
    }

    /// Ein AXTree-Knoten mit Rolle und Namen, verbunden über die Backend-ID.
    fn ax_node(id: &str, backend: i64, role: &str, name: Option<&str>) -> AXNode {
        AXNode {
            node_id: id.to_string(),
            ignored: false,
            ignored_reasons: Vec::new(),
            role: Some(role.to_string()),
            name: name.map(str::to_string),
            name_source: None,
            description: None,
            value: None,
            properties: Vec::new(),
            child_ids: Vec::new(),
            parent_id: None,
            backend_dom_node_id: Some(backend),
        }
    }

    fn ax_tree(nodes: Vec<AXNode>) -> AXTree {
        AXTree::from_nodes(nodes)
    }

    #[test]
    fn gleiche_namen_zaehlen_als_uebereinstimmung() {
        let dom = document(serde_json::json!([element(
            10,
            "button",
            &[],
            serde_json::json!([text(11, "Senden")])
        ),]));
        let ax = ax_tree(vec![ax_node("ax10", 10, "button", Some("Senden"))]);
        let doc = build_document(&dom, &ax).unwrap();

        let diff = compare(&doc, DEFAULT_MAX_SAMPLES);
        assert_eq!(diff.elements_compared, 1);
        assert_eq!(diff.names_equal, 1);
        assert_eq!(diff.names_divergent(), 0);
    }

    /// `accname` zieht Leerraum selbst zusammen. Ein Leerraumfall entsteht
    /// deshalb nur, wenn Chrome ihn stehen lässt — etwa bei geschütztem
    /// Leerzeichen oder nachlaufendem Leerraum im nativen Namen.
    #[test]
    fn reiner_leerraum_wird_getrennt_gefuehrt() {
        let dom = document(serde_json::json!([element(
            10,
            "button",
            &[],
            serde_json::json!([text(11, "Jetzt senden")])
        ),]));
        let ax = ax_tree(vec![ax_node(
            "ax10",
            10,
            "button",
            Some("Jetzt\u{a0}senden "),
        )]);
        let doc = build_document(&dom, &ax).unwrap();

        let diff = compare(&doc, DEFAULT_MAX_SAMPLES);
        assert_eq!(diff.names_divergent(), 1);
        assert_eq!(diff.names_divergent_substantive(), 0);
        assert_eq!(diff.name_divergences[0].shape, Shape::Whitespace);
    }

    /// Der Normalfall: `accname` bekommt den Rohtext aus dem DOM und
    /// normalisiert ihn selbst — das ist keine Abweichung.
    #[test]
    fn eigene_normalisierung_erzeugt_keine_abweichung() {
        let dom = document(serde_json::json!([element(
            10,
            "button",
            &[],
            serde_json::json!([text(11, "  Senden  ")])
        ),]));
        let ax = ax_tree(vec![ax_node("ax10", 10, "button", Some("Senden"))]);
        let doc = build_document(&dom, &ax).unwrap();

        assert_eq!(compare(&doc, DEFAULT_MAX_SAMPLES).names_divergent(), 0);
    }

    #[test]
    fn aria_label_gewinnt_gegen_inhalt_auf_beiden_seiten() {
        let dom = document(serde_json::json!([element(
            10,
            "button",
            &["aria-label", "Menü öffnen"],
            serde_json::json!([text(11, "☰")])
        ),]));
        let ax = ax_tree(vec![ax_node("ax10", 10, "button", Some("Menü öffnen"))]);
        let doc = build_document(&dom, &ax).unwrap();

        assert_eq!(compare(&doc, DEFAULT_MAX_SAMPLES).names_equal, 1);
    }

    #[test]
    fn echte_abweichung_wird_als_mismatch_gemeldet() {
        // Chrome meldet den Titel, die eigene Berechnung den Inhalt.
        let dom = document(serde_json::json!([element(
            10,
            "a",
            &["href", "/x", "title", "Startseite"],
            serde_json::json!([text(11, "Mehr")])
        ),]));
        let ax = ax_tree(vec![ax_node("ax10", 10, "link", Some("Startseite"))]);
        let doc = build_document(&dom, &ax).unwrap();

        let diff = compare(&doc, DEFAULT_MAX_SAMPLES);
        assert_eq!(diff.names_divergent_substantive(), 1);
        assert_eq!(diff.name_divergences[0].shape, Shape::Mismatch);
        assert_eq!(diff.name_divergences[0].tag, "a");
        assert_eq!(diff.name_divergences[0].backend_node_id, Some(10));
    }

    #[test]
    fn knoten_ohne_ax_gegenstueck_werden_nicht_verglichen() {
        let dom = document(serde_json::json!([element(
            10,
            "button",
            &[],
            serde_json::json!([text(11, "Senden")])
        ),]));
        let doc = build_document(&dom, &AXTree::new()).unwrap();

        let diff = compare(&doc, DEFAULT_MAX_SAMPLES);
        assert_eq!(diff.elements_compared, 0);
        assert!(diff.skipped_not_in_ax_tree >= 1);
        assert_eq!(diff.names_divergent(), 0);
    }

    #[test]
    fn interne_chrome_rollen_gelten_als_nicht_vergleichbar() {
        let dom = document(serde_json::json!([element(
            10,
            "p",
            &[],
            serde_json::json!([text(11, "Text")])
        ),]));
        let ax = ax_tree(vec![ax_node("ax10", 10, "StaticText", None)]);
        let doc = build_document(&dom, &ax).unwrap();

        let diff = compare(&doc, DEFAULT_MAX_SAMPLES);
        assert_eq!(diff.roles_not_comparable, 1);
        assert_eq!(diff.roles_divergent, 0);
    }

    #[test]
    fn beispiele_sind_gedeckelt_die_zaehlung_nicht() {
        let mut kinder = Vec::new();
        let mut ax_nodes = Vec::new();
        for i in 0..5i64 {
            let backend = 100 + i * 2;
            kinder.push(element(
                backend,
                "a",
                &["href", "/x", "title", "Titel"],
                serde_json::json!([text(backend + 1, "Mehr")]),
            ));
            ax_nodes.push(ax_node(
                &format!("ax{backend}"),
                backend,
                "link",
                Some("Titel"),
            ));
        }
        let doc = build_document(
            &document(serde_json::Value::Array(kinder)),
            &ax_tree(ax_nodes),
        )
        .unwrap();

        let diff = compare(&doc, 2);
        assert_eq!(diff.names_divergent_substantive(), 5);
        assert_eq!(diff.name_divergences.len(), 2);
    }
}
