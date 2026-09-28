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

use std::collections::{BTreeMap, BTreeSet, HashSet};

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
    /// Beide Namen sind bis auf Groß-/Kleinschreibung gleich, und das Element
    /// oder ein Nachfahre hat ein berechnetes `text-transform` ungleich
    /// `none`. Chrome wendet die CSS-Transformation auf den Namen an,
    /// `accname` rechnet über den DOM-Text. Getrennt geführt, weil diese
    /// Klasse sonst jeden Korpus dominiert. Braucht den berechneten Stil und
    /// entsteht deshalb nur über [`compare_with_text_transform`].
    TextTransform,
}

impl Shape {
    pub fn as_str(self) -> &'static str {
        match self {
            Shape::Whitespace => "whitespace",
            Shape::MissingLocally => "missing_locally",
            Shape::MissingInChrome => "missing_in_chrome",
            Shape::Mismatch => "mismatch",
            Shape::TextTransform => "text_transform",
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
    /// Kreuztabelle Form × Namensquelle, vollständig wie die beiden Zählungen
    /// darüber.
    #[serde(default)]
    pub names_by_shape_and_source: BTreeMap<String, BTreeMap<String, usize>>,
    /// Beispiele, gedeckelt durch `max_samples` je [`Shape`].
    pub name_divergences: Vec<NameDivergence>,

    /// Rollen, bei denen beide Seiten übereinstimmen — bekannte
    /// Schreibvarianten (Chrome `image` = `img`) eingeschlossen.
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

/// Eine Seite, die im Korpuslauf nicht verglichen werden konnte.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageFailure {
    pub url: String,
    pub error: String,
}

/// Ein wiederkehrendes Abweichungsmuster über den Korpus: gleiche Form,
/// Namensquelle, Tag und gleiches Namenspaar. Dieselbe Navigation auf dreißig
/// Unterseiten ist ein Befund, nicht dreißig.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamePattern {
    pub shape: Shape,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name_source: Option<String>,
    pub tag: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chrome: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accname: Option<String>,
    /// Vorkommen über alle Seiten.
    pub occurrences: usize,
    /// Zahl der Seiten, auf denen das Muster vorkommt.
    pub pages: usize,
    /// Erste Seite mit diesem Muster — der Weg zurück zum Element.
    pub first_url: String,
}

/// Aggregat über mehrere Differentialläufe.
///
/// Die Summen sind vollständig. Die Muster entstehen aus den mitgeführten
/// Beispielen je Seite; `patterns_complete` sagt, ob dabei keine Seite an
/// ihre Beispielgrenze gestoßen ist — nur dann sind die Vorkommen je Muster
/// exakt.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AccnameCorpus {
    pub pages_total: usize,
    pub pages_compared: usize,
    pub failures: Vec<PageFailure>,

    pub elements_total: usize,
    pub elements_compared: usize,
    pub skipped_not_in_ax_tree: usize,
    pub skipped_ignored: usize,

    pub names_equal: usize,
    pub names_by_shape: BTreeMap<String, usize>,
    pub names_by_source: BTreeMap<String, usize>,
    pub names_by_shape_and_source: BTreeMap<String, BTreeMap<String, usize>>,

    pub roles_equal: usize,
    pub roles_not_comparable: usize,
    pub roles_divergent: usize,

    pub patterns_complete: bool,
    /// Muster je Form, nach Vorkommen absteigend, gedeckelt durch
    /// `max_samples` je Form.
    pub name_patterns: Vec<NamePattern>,

    /// Die vollständigen Einzelläufe, jeweils mit `url`.
    pub pages: Vec<AccnameDiff>,
}

impl AccnameCorpus {
    pub fn from_pages(
        pages: Vec<AccnameDiff>,
        failures: Vec<PageFailure>,
        max_samples: usize,
    ) -> Self {
        let mut out = AccnameCorpus {
            pages_total: pages.len() + failures.len(),
            pages_compared: pages.len(),
            failures,
            patterns_complete: true,
            ..Default::default()
        };

        type Key = (
            Shape,
            Option<String>,
            String,
            Option<String>,
            Option<String>,
        );
        let mut patterns: BTreeMap<Key, NamePattern> = BTreeMap::new();

        for page in &pages {
            out.elements_total += page.elements_total;
            out.elements_compared += page.elements_compared;
            out.skipped_not_in_ax_tree += page.skipped_not_in_ax_tree;
            out.skipped_ignored += page.skipped_ignored;
            out.names_equal += page.names_equal;
            out.roles_equal += page.roles_equal;
            out.roles_not_comparable += page.roles_not_comparable;
            out.roles_divergent += page.roles_divergent;
            add_counts(&mut out.names_by_shape, &page.names_by_shape);
            add_counts(&mut out.names_by_source, &page.names_by_source);
            for (shape, by_source) in &page.names_by_shape_and_source {
                add_counts(
                    out.names_by_shape_and_source
                        .entry(shape.clone())
                        .or_default(),
                    by_source,
                );
            }
            if page.name_divergences.len() < page.names_divergent() {
                out.patterns_complete = false;
            }

            let url = page.url.clone().unwrap_or_default();
            let mut seen_on_page: BTreeSet<Key> = BTreeSet::new();
            for d in &page.name_divergences {
                let key: Key = (
                    d.shape,
                    d.name_source.clone(),
                    d.tag.clone(),
                    d.chrome.clone(),
                    d.accname.clone(),
                );
                let first_on_page = seen_on_page.insert(key.clone());
                let pattern = patterns.entry(key).or_insert_with(|| NamePattern {
                    shape: d.shape,
                    name_source: d.name_source.clone(),
                    tag: d.tag.clone(),
                    chrome: d.chrome.clone(),
                    accname: d.accname.clone(),
                    occurrences: 0,
                    pages: 0,
                    first_url: url.clone(),
                });
                pattern.occurrences += 1;
                if first_on_page {
                    pattern.pages += 1;
                }
            }
        }

        let mut patterns: Vec<NamePattern> = patterns.into_values().collect();
        patterns.sort_by(|a, b| {
            a.shape
                .cmp(&b.shape)
                .then(b.occurrences.cmp(&a.occurrences))
                .then(b.pages.cmp(&a.pages))
        });
        let mut per_shape: BTreeMap<Shape, usize> = BTreeMap::new();
        patterns.retain(|p| {
            let n = per_shape.entry(p.shape).or_insert(0);
            *n += 1;
            *n <= max_samples
        });
        out.name_patterns = patterns;
        out.pages = pages;
        out
    }

    /// Zahl aller Namensabweichungen, Leerraumfälle eingeschlossen.
    pub fn names_divergent(&self) -> usize {
        self.names_by_shape.values().sum()
    }
}

fn add_counts(into: &mut BTreeMap<String, usize>, from: &BTreeMap<String, usize>) {
    for (k, v) in from {
        *into.entry(k.clone()).or_insert(0) += v;
    }
}

/// Vergleicht die eigene Berechnung mit Chromes nativen Werten.
///
/// `max_samples` deckelt nur die mitgeführten Beispiele je Abweichungsart;
/// alle Zählungen bleiben vollständig. Ohne berechneten Stil entsteht keine
/// [`Shape::TextTransform`]; solche Fälle erscheinen als `mismatch`.
pub fn compare(doc: &CdpDocument, max_samples: usize) -> AccnameDiff {
    compare_with_text_transform(doc, max_samples, &HashSet::new())
}

/// Wie [`compare`], aber mit der Menge der Backend-Node-IDs, deren Element
/// (oder ein Nachfahre) ein berechnetes `text-transform` ungleich `none` hat.
/// Eine Abweichung, die nur in der Groß-/Kleinschreibung besteht, wird für
/// diese Elemente als [`Shape::TextTransform`] geführt statt als `mismatch`.
///
/// Die Menge kommt aus dem Browser; welche IDs dafür überhaupt abgefragt
/// werden müssen, liefert [`case_only_candidates`].
pub fn compare_with_text_transform(
    doc: &CdpDocument,
    max_samples: usize,
    text_transformed: &HashSet<i64>,
) -> AccnameDiff {
    let mut out = AccnameDiff::default();
    let ids = IdIndex::build(doc.root());
    let mut name_samples: BTreeMap<Shape, usize> = BTreeMap::new();

    visit(doc.root(), &mut |node| {
        out.elements_total += 1;
        if doc.ax_node_id(node).is_none() {
            out.skipped_not_in_ax_tree += 1;
        } else if doc.is_ignored(node) {
            out.skipped_ignored += 1;
        } else {
            out.elements_compared += 1;
            compare_name(
                doc,
                node,
                &ids,
                max_samples,
                text_transformed,
                &mut name_samples,
                &mut out,
            );
            compare_role(doc, node, max_samples, &mut out);
        }
    });
    out
}

/// Backend-Node-IDs aller verglichenen Elemente, deren Namen auf beiden Seiten
/// vorhanden sind und sich nur in der Groß-/Kleinschreibung unterscheiden —
/// die Kandidaten für [`Shape::TextTransform`]. Für genau diese muss der
/// berechnete Stil geholt werden.
pub fn case_only_candidates(doc: &CdpDocument) -> Vec<i64> {
    let ids = IdIndex::build(doc.root());
    let mut out = Vec::new();
    visit(doc.root(), &mut |node| {
        if doc.ax_node_id(node).is_none() || doc.is_ignored(node) {
            return;
        }
        let (chrome, own) = names(doc, node, &ids);
        let chrome_n = chrome.as_deref().map(normalize).filter(|s| !s.is_empty());
        let own_n = own.as_deref().map(normalize).filter(|s| !s.is_empty());
        if shape_of(chrome.as_deref(), own.as_deref(), &chrome_n, &own_n) == Some(Shape::Mismatch)
            && equal_up_to_case(&chrome_n, &own_n)
        {
            out.extend(doc.backend_node_id(node));
        }
    });
    out
}

/// Besucht alle Elementknoten in Dokumentreihenfolge.
fn visit<'n>(node: ArenaNode<'n>, f: &mut impl FnMut(ArenaNode<'n>)) {
    if node.kind() == NodeKind::Element {
        f(node);
    }
    for child in node.children() {
        visit(child, f);
    }
}

fn names<'n>(
    doc: &'n CdpDocument,
    node: ArenaNode<'n>,
    ids: &IdIndex<'n, ArenaNode<'n>>,
) -> (Option<String>, Option<String>) {
    let own = match doc.rendered() {
        Some(rendered) => accname::name_rendered(&rendered, node, ids),
        None => accname::name(node, ids),
    };
    (doc.accessible_name(node), own)
}

/// Gleich bis auf Groß-/Kleinschreibung. Verglichen wird über
/// `to_uppercase`, weil `text-transform: uppercase` etwa „ß“ zu „SS“ macht —
/// ein Kleinschreibungsvergleich fände das nicht.
fn equal_up_to_case(a: &Option<String>, b: &Option<String>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => a.to_uppercase() == b.to_uppercase(),
        _ => false,
    }
}

fn compare_name<'n>(
    doc: &'n CdpDocument,
    node: ArenaNode<'n>,
    ids: &IdIndex<'n, ArenaNode<'n>>,
    max_samples: usize,
    text_transformed: &HashSet<i64>,
    name_samples: &mut BTreeMap<Shape, usize>,
    out: &mut AccnameDiff,
) {
    let (chrome, own) = names(doc, node, ids);

    let chrome_n = chrome.as_deref().map(normalize).filter(|s| !s.is_empty());
    let own_n = own.as_deref().map(normalize).filter(|s| !s.is_empty());

    let Some(mut shape) = shape_of(chrome.as_deref(), own.as_deref(), &chrome_n, &own_n) else {
        out.names_equal += 1;
        return;
    };
    if shape == Shape::Mismatch
        && equal_up_to_case(&chrome_n, &own_n)
        && doc
            .backend_node_id(node)
            .is_some_and(|id| text_transformed.contains(&id))
    {
        shape = Shape::TextTransform;
    }

    *out.names_by_shape
        .entry(shape.as_str().to_string())
        .or_insert(0) += 1;
    let source = doc
        .chrome_name_source(node)
        .or_else(|| doc.name_source(node).map(name_source_str));
    let source_key = source.unwrap_or("unknown");
    *out.names_by_source
        .entry(source_key.to_string())
        .or_insert(0) += 1;
    *out.names_by_shape_and_source
        .entry(shape.as_str().to_string())
        .or_default()
        .entry(source_key.to_string())
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
    if own.is_some_and(|own| same_role(chrome_role, own)) {
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

/// Rollen gleich, bis auf bekannte Schreibvarianten desselben Begriffs:
/// Chrome nennt `img` `image` (ARIA 1.3 führt `image` als Synonym). Reine
/// Normalisierung der Messung; `accname` bleibt unberührt.
fn same_role(chrome: &str, own: &str) -> bool {
    const ALIASES: &[(&str, &str)] = &[("image", "img")];
    chrome == own || ALIASES.contains(&(chrome, own))
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
    use std::collections::HashMap;

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

    /// Chrome wendet `text-transform` an, `accname` nicht. Ohne Stil bleibt
    /// das ein `mismatch`; mit Stil wird es getrennt geführt.
    #[test]
    fn nur_gross_klein_mit_text_transform_wird_eigene_klasse() {
        let dom = document(serde_json::json!([
            element(
                10,
                "a",
                &["href", "/l"],
                serde_json::json!([text(11, "Straße")])
            ),
            element(
                12,
                "a",
                &["href", "/m"],
                serde_json::json!([text(13, "Mehr")])
            ),
        ]));
        let ax = ax_tree(vec![
            ax_node("ax10", 10, "link", Some("STRASSE")),
            ax_node("ax12", 12, "link", Some("MEHR")),
        ]);
        let doc = build_document(&dom, &ax).unwrap();

        assert_eq!(case_only_candidates(&doc), vec![10, 12]);

        let ohne_stil = compare(&doc, DEFAULT_MAX_SAMPLES);
        assert_eq!(ohne_stil.names_by_shape.get("mismatch"), Some(&2));

        // Nur 10 hat laut Browser ein text-transform.
        let diff = compare_with_text_transform(&doc, DEFAULT_MAX_SAMPLES, &HashSet::from([10]));
        assert_eq!(diff.names_by_shape.get("text_transform"), Some(&1));
        assert_eq!(diff.names_by_shape.get("mismatch"), Some(&1));
        assert_eq!(
            diff.names_by_shape_and_source["text_transform"].get("unknown"),
            Some(&1)
        );
    }

    #[test]
    fn andere_texte_sind_keine_kandidaten() {
        let dom = document(serde_json::json!([element(
            10,
            "a",
            &["href", "/x", "title", "Startseite"],
            serde_json::json!([text(11, "Mehr")])
        )]));
        let ax = ax_tree(vec![ax_node("ax10", 10, "link", Some("Startseite"))]);
        let doc = build_document(&dom, &ax).unwrap();

        assert!(case_only_candidates(&doc).is_empty());
        let diff = compare_with_text_transform(&doc, DEFAULT_MAX_SAMPLES, &HashSet::from([10]));
        assert_eq!(diff.names_by_shape.get("mismatch"), Some(&1));
    }

    fn page_with(url: &str, chrome: &str, count: usize) -> AccnameDiff {
        let mut kinder = Vec::new();
        let mut ax_nodes = Vec::new();
        for i in 0..count as i64 {
            let backend = 100 + i * 2;
            kinder.push(element(
                backend,
                "a",
                &["href", "/x"],
                serde_json::json!([text(backend + 1, "Mehr")]),
            ));
            ax_nodes.push(ax_node(
                &format!("ax{backend}"),
                backend,
                "link",
                Some(chrome),
            ));
        }
        let doc = build_document(
            &document(serde_json::Value::Array(kinder)),
            &ax_tree(ax_nodes),
        )
        .unwrap();
        let mut diff = compare(&doc, DEFAULT_MAX_SAMPLES);
        diff.url = Some(url.to_string());
        diff
    }

    #[test]
    fn korpus_summiert_und_fasst_muster_zusammen() {
        let corpus = AccnameCorpus::from_pages(
            vec![
                page_with("https://a", "Titel", 2),
                page_with("https://b", "Titel", 1),
            ],
            vec![PageFailure {
                url: "https://c".into(),
                error: "timeout".into(),
            }],
            DEFAULT_MAX_SAMPLES,
        );

        assert_eq!(corpus.pages_total, 3);
        assert_eq!(corpus.pages_compared, 2);
        assert_eq!(corpus.names_by_shape.get("mismatch"), Some(&3));
        assert!(corpus.patterns_complete);
        assert_eq!(corpus.name_patterns.len(), 1);
        let p = &corpus.name_patterns[0];
        assert_eq!((p.occurrences, p.pages), (3, 2));
        assert_eq!(p.first_url, "https://a");
    }

    /// Mit Chromes feinen Namensquellen landet ein `aria-label` auf einem
    /// generischen Element unter `aria-label` statt unter `unknown`.
    #[test]
    fn chromes_feine_namensquelle_steht_in_der_quellachse() {
        let dom = document(serde_json::json!([element(
            10,
            "span",
            &["aria-label", "nicht enthalten"],
            serde_json::json!([text(11, "–")])
        )]));
        let mut node = ax_node("ax10", 10, "generic", Some("nicht enthalten"));
        node.name_source = Some(crate::accessibility::NameSource::Attribute);
        let ax = ax_tree(vec![node]);

        let grob = compare(&build_document(&dom, &ax).unwrap(), DEFAULT_MAX_SAMPLES);
        assert_eq!(grob.names_by_source.get("unknown"), Some(&1));

        let doc = build_document(&dom, &ax)
            .unwrap()
            .with_chrome_name_sources(&HashMap::from([(10, "aria-label".to_string())]));
        let fein = compare(&doc, DEFAULT_MAX_SAMPLES);
        assert_eq!(fein.names_by_source.get("aria-label"), Some(&1));
        assert_eq!(
            fein.names_by_shape_and_source["missing_locally"].get("aria-label"),
            Some(&1)
        );
        assert_eq!(
            fein.name_divergences[0].name_source.as_deref(),
            Some("aria-label")
        );
    }

    /// Chrome `image` und `accname` `img` sind derselbe Begriff.
    #[test]
    fn image_und_img_gelten_als_gleiche_rolle() {
        let dom = document(serde_json::json!([
            element(
                10,
                "img",
                &["src", "a.png", "alt", "Logo"],
                serde_json::json!([])
            ),
            element(12, "header", &[], serde_json::json!([text(13, "Kopf")])),
        ]));
        let ax = ax_tree(vec![
            ax_node("ax10", 10, "image", Some("Logo")),
            ax_node("ax12", 12, "sectionheader", None),
        ]);
        let doc = build_document(&dom, &ax).unwrap();

        let diff = compare(&doc, DEFAULT_MAX_SAMPLES);
        assert_eq!(diff.roles_equal, 1);
        // Kein Alias: `sectionheader` gegen `banner` bleibt eine Abweichung.
        assert_eq!(diff.roles_divergent, 1);
        assert_eq!(diff.role_divergences[0].tag, "header");
    }

    #[test]
    fn same_role_kennt_nur_die_richtung_chrome_image() {
        assert!(same_role("image", "img"));
        assert!(same_role("button", "button"));
        assert!(!same_role("img", "image"));
        assert!(!same_role("sectionheader", "banner"));
    }

    #[test]
    fn korpus_meldet_unvollstaendige_muster() {
        let mut page = page_with("https://a", "Titel", 3);
        page.name_divergences.truncate(1);
        let corpus = AccnameCorpus::from_pages(vec![page], Vec::new(), DEFAULT_MAX_SAMPLES);
        assert!(!corpus.patterns_complete);
    }
}
