//! Brücke vom CDP-DOM auf das geteilte Dokumentmodell aus `a11y-dom`.
//!
//! # Warum nicht direkt über den AXTree
//!
//! Das `Document`-Trait ist **DOM-förmig**, nicht AX-Tree-förmig, und das ist
//! kein Zufall: Die Mehrzahl der Tier-1-Regeln entscheidet über Tags und
//! Attribute — `lang`, `tabindex`, `id`, `role`, `alt`, `content`. Der native
//! Accessibility-Tree gibt die gar nicht her. auditmysite hat das bisher
//! umschifft, indem einzelne Regeln sich ihre Attribute per eingespritztem
//! JavaScript nachgeholt haben (`apply_lang_attribute_check` in
//! `audit::pipeline` ist der deutlichste Fall: „the AX tree does not expose
//! `html[lang]`"). Dieses Modul holt den DOM einmal am Stück und macht ihn den
//! geteilten Regeln zugänglich, statt pro Regel einen Sonderweg zu bauen.
//!
//! # Was auditmysite besser kann als die anderen Oberflächen
//!
//! Rolle und Accessible Name kommen hier aus Chromes echtem
//! Accessibility-Tree, nicht aus einer Nachrechnung. [`CdpDocument`] erfüllt
//! [`Semantics`] deshalb über die nativen Werte; das `accname`-Crate ist der
//! Ersatz für Hosts *ohne* nativen Tree und wird hier bewusst nicht benutzt.
//!
//! # Wie die beiden Bäume zusammenfinden
//!
//! Der Arena-Index eines Knotens ist seine [`a11y_dom::NodeId`]; die Backend-Node-ID des
//! CDP-DOM wird beim Bauen in derselben Reihenfolge mitgeschrieben. Der AXTree
//! führt zu jedem Knoten dieselbe Backend-ID. Über diese ID werden beide Bäume
//! verbunden — nicht über Position oder Tagnamen, die bei Shadow DOM und
//! ignorierten Knoten auseinanderlaufen.

use std::collections::HashMap;

use a11y_dom::{Arena, ArenaNode, Document, NameSource as SharedNameSource, Node, Semantics};
use chromiumoxide::cdp::browser_protocol::dom::{GetDocumentParams, Node as CdpNode};
use chromiumoxide::Page;
use tracing::{debug, warn};

use super::tree::{AXTree, NameSource};
use crate::error::{AuditError, Result};

/// DOM-Knotentypen, die hier vorkommen. Die Zahlen sind die des DOM-Standards,
/// die CDP unverändert durchreicht.
const ELEMENT_NODE: i64 = 1;
const TEXT_NODE: i64 = 3;

/// Was der native Accessibility-Tree über einen Knoten weiß.
#[derive(Debug, Clone, Default)]
struct AxFacts {
    /// Kennung des Knotens im AXTree — der Schluessel, den
    /// `enrichment`/`element_capture` erwarten.
    ax_node_id: String,
    role: Option<String>,
    name: Option<String>,
    name_source: Option<SharedNameSource>,
    ignored: bool,
}

/// Ein über CDP geholtes Dokument, das [`Document`] und [`Semantics`] erfüllt.
///
/// Damit laufen die geteilten Regeln aus `a11y-rules` unverändert gegen eine
/// per Chrome gerenderte Seite — mit denselben Kennungen, die
/// astro-post-audit zur Build-Zeit und LiveAudit in der laufenden Seite
/// vergeben.
pub struct CdpDocument {
    arena: Arena,
    /// Backend-Node-ID je Arena-Index. Gleiche Reihenfolge wie beim Bauen,
    /// deshalb ist der Index identisch mit [`a11y_dom::NodeId`].
    backend_ids: Vec<Option<i64>>,
    /// Native AX-Werte, über die Backend-Node-ID verschlüsselt.
    ax: HashMap<i64, AxFacts>,
}

impl CdpDocument {
    /// Die Backend-Node-ID zu einem Knoten — der Rückweg in auditmysites
    /// bestehende Anreicherung (`enrichment`, `element_capture`), die über
    /// Backend-IDs arbeitet.
    pub fn backend_node_id(&self, node: ArenaNode<'_>) -> Option<i64> {
        self.backend_ids
            .get(node.id().0 as usize)
            .copied()
            .flatten()
    }

    /// Anzahl der Knoten im materialisierten Baum.
    pub fn len(&self) -> usize {
        self.arena.len()
    }

    pub fn is_empty(&self) -> bool {
        self.arena.is_empty()
    }

    /// Der Knoten zu einer [`a11y_dom::NodeId`] — der Rueckweg von einem
    /// Befund (der nur den Arena-Index kennt) auf das Element.
    pub fn node_at(&self, id: a11y_dom::NodeId) -> Option<ArenaNode<'_>> {
        self.arena.get(id)
    }

    /// Die AXTree-Knotenkennung zu einem Knoten, sofern er im
    /// Accessibility-Tree ueberhaupt vorkommt. Ignorierte und rein
    /// praesentationale Elemente haben dort kein Gegenstueck.
    pub fn ax_node_id(&self, node: ArenaNode<'_>) -> Option<&str> {
        Some(self.facts(node)?.ax_node_id.as_str())
    }

    fn facts(&self, node: ArenaNode<'_>) -> Option<&AxFacts> {
        self.ax.get(&self.backend_node_id(node)?)
    }
}

impl Document for CdpDocument {
    type N<'a>
        = ArenaNode<'a>
    where
        Self: 'a;

    fn root(&self) -> Self::N<'_> {
        self.arena.root()
    }

    fn node_count(&self) -> Option<usize> {
        self.arena.node_count()
    }
}

impl Semantics for CdpDocument {
    fn role<'n>(&'n self, node: Self::N<'n>) -> Option<String> {
        self.facts(node)?.role.clone()
    }

    fn accessible_name<'n>(&'n self, node: Self::N<'n>) -> Option<String> {
        self.facts(node)?.name.clone()
    }

    fn name_source<'n>(&'n self, node: Self::N<'n>) -> Option<SharedNameSource> {
        self.facts(node)?.name_source
    }

    fn is_ignored<'n>(&'n self, node: Self::N<'n>) -> bool {
        self.facts(node).is_some_and(|f| f.ignored)
    }
}

/// Übersetzt auditmysites `NameSource` in die geteilte Fassung.
///
/// Bewusst unvollständig: auditmysites Extractor fasst `aria-label`, `alt` und
/// andere Attributquellen zu `Attribute` zusammen und `aria-labelledby` wie
/// `<label for>` zu `RelatedElement`. Diese Information ist an der Stelle schon
/// verloren, und sie hier zu raten hieße, eine Tatsache zu behaupten, die der
/// Baum nicht hergibt. Für die mehrdeutigen Fälle bleibt es deshalb bei `None`
/// — das Trait sieht genau das als Vorgabe vor.
fn map_name_source(src: NameSource) -> Option<SharedNameSource> {
    match src {
        NameSource::Contents => Some(SharedNameSource::Contents),
        NameSource::Placeholder => Some(SharedNameSource::Placeholder),
        NameSource::Title => Some(SharedNameSource::Title),
        NameSource::Attribute | NameSource::RelatedElement => None,
    }
}

/// Baut die Nachschlagetabelle Backend-Node-ID -> AX-Werte.
fn index_ax_tree(ax_tree: &AXTree) -> HashMap<i64, AxFacts> {
    let mut map = HashMap::new();
    for node in ax_tree.iter_all() {
        let Some(backend) = node.backend_dom_node_id else {
            continue;
        };
        map.insert(
            backend,
            AxFacts {
                ax_node_id: node.node_id.clone(),
                role: node.role.clone(),
                name: node.name.clone(),
                name_source: node.name_source.and_then(map_name_source),
                ignored: node.ignored,
            },
        );
    }
    map
}

/// Der lesbare Tagname eines CDP-Knotens, kleingeschrieben und ohne Präfix.
fn local_name_of(node: &CdpNode) -> String {
    if !node.local_name.is_empty() {
        node.local_name.to_ascii_lowercase()
    } else {
        node.node_name.to_ascii_lowercase()
    }
}

/// Zustand beim Umkopieren des CDP-Baums in die Arena.
struct Walk {
    builder: Option<a11y_dom::ArenaBuilder>,
    backend_ids: Vec<Option<i64>>,
}

impl Walk {
    /// Jeder Push in die Arena wird hier gespiegelt, damit Arena-Index und
    /// `backend_ids`-Index nicht auseinanderlaufen.
    fn record(&mut self, backend: Option<i64>) {
        self.backend_ids.push(backend);
    }

    fn with<F>(&mut self, f: F)
    where
        F: FnOnce(a11y_dom::ArenaBuilder) -> a11y_dom::ArenaBuilder,
    {
        let b = self.builder.take().expect("builder liegt immer vor");
        self.builder = Some(f(b));
    }

    fn element(&mut self, node: &CdpNode) {
        let name = local_name_of(node);
        self.with(|b| b.open(&name));
        self.record(Some(*node.backend_node_id.inner()));

        if let Some(attrs) = &node.attributes {
            for pair in attrs.as_chunks::<2>().0 {
                let (k, v) = (pair[0].to_ascii_lowercase(), pair[1].clone());
                self.with(move |b| b.attr(&k, &v));
            }
        }

        // Shadow-Roots zählen für die Barrierefreiheit zum Inhalt des Hosts;
        // ihre Kinder werden deshalb unter den Host gehängt. Der Fragment-
        // Knoten selbst bekommt kein Gegenstück — er hat keinen Tagnamen, den
        // eine Regel sinnvoll prüfen könnte.
        for shadow in node.shadow_roots.iter().flatten() {
            self.children(shadow);
        }

        self.children(node);
        self.with(|b| b.close());
    }

    fn children(&mut self, node: &CdpNode) {
        for child in node.children.iter().flatten() {
            self.node(child);
        }
    }

    fn node(&mut self, node: &CdpNode) {
        match node.node_type {
            ELEMENT_NODE => self.element(node),
            TEXT_NODE => {
                let text = node.node_value.clone();
                self.with(move |b| b.text(&text));
                self.record(Some(*node.backend_node_id.inner()));
            }
            // Kommentare, Doctype, Processing Instructions: für
            // Accessibility-Regeln ohne Bedeutung, siehe `a11y_dom::NodeKind`.
            //
            // `content_document` (Inhalt von iframes) und `template_content`
            // werden bewusst nicht betreten: Ein iframe bringt ein eigenes
            // Dokument mit eigenem `lang` und `title` mit, das als Teil dieses
            // Baums falsche Befunde erzeugen würde. Dafür gibt es in
            // auditmysite die eigenen iframe-Regeln.
            _ => {}
        }
    }
}

/// Findet das `<html>`-Element unter dem Dokumentknoten.
///
/// Die Regeln erwarten `<html>` als Wurzel — `document/lang-missing` etwa
/// prüft `doc.root().attr("lang")` und tut gar nichts, wenn die Wurzel kein
/// `html` ist. Ein Baum, der am `#document`-Knoten hängt, liefe also still
/// ins Leere statt einen Befund zu melden.
fn find_html(root: &CdpNode) -> Option<&CdpNode> {
    if root.node_type == ELEMENT_NODE && local_name_of(root) == "html" {
        return Some(root);
    }
    root.children
        .iter()
        .flatten()
        .find_map(|child| find_html(child))
}

/// Baut ein [`CdpDocument`] aus einem CDP-Dokumentknoten und dem AXTree.
///
/// Getrennt von [`fetch_dom_document`], damit der Umbau ohne laufenden Browser
/// geprüft werden kann.
pub fn build_document(root: &CdpNode, ax_tree: &AXTree) -> Result<CdpDocument> {
    let html = find_html(root).ok_or_else(|| AuditError::AXTreeExtractionFailed {
        reason: "DOM enthält kein <html>-Element".to_string(),
    })?;

    let mut walk = Walk {
        builder: Some(Arena::builder()),
        backend_ids: Vec::new(),
    };
    walk.node(html);

    let arena = walk
        .builder
        .take()
        .expect("builder liegt immer vor")
        .build();
    debug!(
        nodes = arena.len(),
        "DOM für die geteilten Regeln materialisiert"
    );

    Ok(CdpDocument {
        arena,
        backend_ids: walk.backend_ids,
        ax: index_ax_tree(ax_tree),
    })
}

/// Holt den vollständigen DOM über CDP und verbindet ihn mit dem AXTree.
pub async fn fetch_dom_document(page: &Page, ax_tree: &AXTree) -> Result<CdpDocument> {
    let params = GetDocumentParams {
        // -1 = gesamter Teilbaum. Ein flacher Abruf mit Nachladen je Ebene
        // wäre ein CDP-Roundtrip pro Knoten.
        depth: Some(-1),
        // Shadow-Roots mitliefern; ohne das fehlen ganze Komponentenbäume.
        pierce: Some(true),
    };

    let response = page
        .execute(params)
        .await
        .map_err(|e| AuditError::AXTreeExtractionFailed {
            reason: format!("DOM.getDocument fehlgeschlagen: {e}"),
        })?;

    let doc = build_document(&response.result.root, ax_tree)?;
    if doc.is_empty() {
        warn!("DOM für die geteilten Regeln ist leer");
    }
    Ok(doc)
}

#[cfg(test)]
mod tests {
    use super::*;
    use a11y_dom::{elements, NodeKind};

    /// Baut einen CDP-Knotenbaum aus JSON. `Node` ist `Deserialize`, damit
    /// laesst sich der Umbau ohne laufenden Chrome pruefen.
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

    /// `#document` -> `<html lang="de">` mit `<head><title>` und `<body>`.
    fn beispielseite() -> CdpNode {
        cdp(serde_json::json!({
            "nodeId": 1,
            "backendNodeId": 1,
            "nodeType": 9,
            "nodeName": "#document",
            "localName": "",
            "nodeValue": "",
            "children": [
                element(2, "html", &["lang", "de"], serde_json::json!([
                    element(3, "head", &[], serde_json::json!([
                        element(4, "title", &[], serde_json::json!([text(5, "Beispiel")])),
                    ])),
                    element(6, "body", &[], serde_json::json!([
                        element(7, "img", &["src", "logo.png"], serde_json::json!([])),
                        element(8, "a", &["href", "/x"], serde_json::json!([text(9, "Mehr")])),
                    ])),
                ]))
            ]
        }))
    }

    fn leerer_ax() -> AXTree {
        AXTree::new()
    }

    #[test]
    fn wurzel_ist_das_html_element_nicht_der_dokumentknoten() {
        let doc = build_document(&beispielseite(), &leerer_ax()).unwrap();
        assert_eq!(doc.root().local_name(), "html");
        assert_eq!(doc.root().attr("lang"), Some("de"));
    }

    #[test]
    fn fehlendes_html_element_ist_ein_fehler() {
        let ohne_html = cdp(serde_json::json!({
            "nodeId": 1, "backendNodeId": 1, "nodeType": 9,
            "nodeName": "#document", "localName": "", "nodeValue": "",
            "children": []
        }));
        assert!(build_document(&ohne_html, &leerer_ax()).is_err());
    }

    #[test]
    fn attribute_und_text_kommen_mit() {
        let doc = build_document(&beispielseite(), &leerer_ax()).unwrap();

        let img = elements(&doc).find(|n| n.is_element("img")).unwrap();
        assert_eq!(img.attr("src"), Some("logo.png"));
        assert!(img.attr("alt").is_none());

        let titel = elements(&doc).find(|n| n.is_element("title")).unwrap();
        assert_eq!(a11y_dom::subtree_text(titel).trim(), "Beispiel");
    }

    /// Der Rueckweg in die bestehende Anreicherung: Arena-Index -> Backend-ID.
    /// Laeuft das auseinander, zeigen alle Befunde auf falsche Elemente.
    #[test]
    fn backend_ids_bleiben_mit_den_arena_indizes_im_gleichschritt() {
        let doc = build_document(&beispielseite(), &leerer_ax()).unwrap();

        for (erwartet, name) in [(2, "html"), (3, "head"), (4, "title"), (7, "img"), (8, "a")] {
            let n = elements(&doc).find(|n| n.is_element(name)).unwrap();
            assert_eq!(
                doc.backend_node_id(n),
                Some(erwartet),
                "Backend-ID von <{name}>"
            );
        }

        // Auch Textknoten werden mitgezaehlt -- wuerden sie uebersprungen,
        // verschoeben sich alle spaeteren Indizes um eins.
        let text_knoten: Vec<_> = a11y_dom::self_and_descendants(doc.root())
            .filter(|n| n.kind() == NodeKind::Text)
            .collect();
        assert_eq!(text_knoten.len(), 2);
        assert_eq!(doc.backend_node_id(text_knoten[0]), Some(5));
        assert_eq!(doc.backend_node_id(text_knoten[1]), Some(9));
    }

    #[test]
    fn shadow_root_kinder_haengen_unter_dem_host() {
        let mit_shadow = cdp(serde_json::json!({
            "nodeId": 1, "backendNodeId": 1, "nodeType": 9,
            "nodeName": "#document", "localName": "", "nodeValue": "",
            "children": [
                element(2, "html", &[], serde_json::json!([
                    element(3, "body", &[], serde_json::json!([
                        {
                            "nodeId": 4, "backendNodeId": 4, "nodeType": 1,
                            "nodeName": "MY-CARD", "localName": "my-card",
                            "nodeValue": "", "children": [],
                            "shadowRoots": [{
                                "nodeId": 5, "backendNodeId": 5, "nodeType": 11,
                                "nodeName": "#document-fragment", "localName": "",
                                "nodeValue": "",
                                "children": [element(6, "button", &[], serde_json::json!([]))]
                            }]
                        }
                    ]))
                ]))
            ]
        }));

        let doc = build_document(&mit_shadow, &leerer_ax()).unwrap();
        let button = elements(&doc).find(|n| n.is_element("button")).unwrap();
        assert_eq!(doc.backend_node_id(button), Some(6));
        assert_eq!(button.parent().unwrap().local_name(), "my-card");
    }

    fn ax_knoten(
        backend: i64,
        role: &str,
        name: Option<&str>,
        ignored: bool,
    ) -> super::super::AXNode {
        super::super::AXNode {
            node_id: format!("ax-{backend}"),
            ignored,
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

    /// Rolle und Name kommen aus dem nativen Baum, nicht aus einer
    /// Nachrechnung -- das ist der Grund, warum `accname` hier nicht gebraucht
    /// wird.
    #[test]
    fn semantics_liest_die_nativen_ax_werte() {
        let ax = AXTree::from_nodes(vec![
            ax_knoten(7, "image", Some("Firmenlogo"), false),
            ax_knoten(8, "link", None, false),
        ]);
        let doc = build_document(&beispielseite(), &ax).unwrap();

        let img = elements(&doc).find(|n| n.is_element("img")).unwrap();
        assert_eq!(doc.role(img).as_deref(), Some("image"));
        assert_eq!(doc.accessible_name(img).as_deref(), Some("Firmenlogo"));

        let link = elements(&doc).find(|n| n.is_element("a")).unwrap();
        assert_eq!(doc.role(link).as_deref(), Some("link"));
        assert!(doc.accessible_name(link).is_none());

        // Ein Knoten ohne AX-Eintrag behauptet nichts.
        let head = elements(&doc).find(|n| n.is_element("head")).unwrap();
        assert!(doc.role(head).is_none());
    }

    #[test]
    fn ignorierte_knoten_werden_als_solche_gemeldet() {
        let ax = AXTree::from_nodes(vec![ax_knoten(8, "link", None, true)]);
        let doc = build_document(&beispielseite(), &ax).unwrap();
        let link = elements(&doc).find(|n| n.is_element("a")).unwrap();
        assert!(doc.is_ignored(link));
    }

    /// Der eigentliche Zweck des Umbaus: Die geteilten Regeln laufen
    /// unveraendert gegen eine per Chrome gerenderte Seite und vergeben
    /// dieselben Kennungen wie auf den anderen beiden Oberflaechen.
    #[test]
    fn geteilte_regeln_laufen_und_vergeben_die_geteilten_kennungen() {
        let ax = AXTree::from_nodes(vec![
            ax_knoten(7, "image", Some("Firmenlogo"), false),
            // Link ohne zugaenglichen Namen -> Tier-2-Befund.
            ax_knoten(8, "link", None, false),
        ]);
        let doc = build_document(&beispielseite(), &ax).unwrap();

        let report = a11y_rules::run_with_semantics(&doc);
        let ids: Vec<&str> = report.findings.iter().map(|f| f.rule_id.as_str()).collect();

        // Tier 1, allein aus dem DOM: <img> ohne alt.
        assert!(ids.contains(&"images/alt-missing"), "gefunden: {ids:?}");
        // Tier 2, nur mit nativem Accessible Name entscheidbar.
        assert!(ids.contains(&"links/name-missing"), "gefunden: {ids:?}");
        // `lang` und `<title>` sind gesetzt -- kein Befund dazu.
        assert!(!ids.iter().any(|id| id.starts_with("document/")), "{ids:?}");

        // Mit Semantics faellt keine Regel mehr mangels Namensberechnung aus.
        // Was uebrig bleibt, ist Tier 3: Ohne gerenderte Stile koennen die
        // Kontrastregeln nichts sagen -- und sagen genau das, statt zu
        // schweigen. Sie kamen mit a11y-core 0.7.0 dazu; auditmysite bedient
        // `Rendering` noch nicht.
        let mut nicht_gelaufen: Vec<&str> = report
            .rule_runs
            .iter()
            .filter(|r| r.not_run.is_some())
            .map(|r| r.rule_id.as_str())
            .collect();
        nicht_gelaufen.sort_unstable();
        assert_eq!(
            nicht_gelaufen,
            vec!["contrast/text-insufficient", "contrast/text-undetermined"],
            "unerwartet nicht gelaufen: {nicht_gelaufen:?}"
        );

        // `rule_runs` und `findings` teilen sich eine Namensmenge -- ein Join
        // ueber `rule_id` muss aufgehen.
        for f in &report.findings {
            assert!(
                report.rule_runs.iter().any(|r| r.rule_id == f.rule_id),
                "kein RuleRun zu {}",
                f.rule_id
            );
        }
    }

    /// Ein Befund zeigt ueber `location.node` auf den Arena-Index; der muss
    /// sich zurueck in eine Backend-Node-ID uebersetzen lassen, sonst laesst
    /// sich der Befund nicht anreichern.
    #[test]
    fn befund_laesst_sich_auf_eine_backend_id_zurueckfuehren() {
        let ax = AXTree::from_nodes(vec![ax_knoten(8, "link", None, false)]);
        let doc = build_document(&beispielseite(), &ax).unwrap();
        let report = a11y_rules::run_with_semantics(&doc);

        let befund = report
            .findings
            .iter()
            .find(|f| f.rule_id == "links/name-missing")
            .expect("Linkbefund");

        let idx: u32 = befund.location.node.as_deref().unwrap().parse().unwrap();
        let knoten = doc.arena.get(a11y_dom::NodeId(idx)).unwrap();
        assert_eq!(doc.backend_node_id(knoten), Some(8));
    }

    /// iframes bringen ein eigenes Dokument mit eigenem `lang`/`title` mit.
    /// Waere es Teil dieses Baums, meldete `document/title-missing` den
    /// falschen Titel bzw. gar keinen.
    #[test]
    fn iframe_inhalt_wird_nicht_betreten() {
        let mit_iframe = cdp(serde_json::json!({
            "nodeId": 1, "backendNodeId": 1, "nodeType": 9,
            "nodeName": "#document", "localName": "", "nodeValue": "",
            "children": [
                element(2, "html", &[], serde_json::json!([
                    element(3, "body", &[], serde_json::json!([
                        {
                            "nodeId": 4, "backendNodeId": 4, "nodeType": 1,
                            "nodeName": "IFRAME", "localName": "iframe",
                            "nodeValue": "", "children": [],
                            "contentDocument": {
                                "nodeId": 5, "backendNodeId": 5, "nodeType": 9,
                                "nodeName": "#document", "localName": "", "nodeValue": "",
                                "children": [element(6, "html", &[], serde_json::json!([
                                    element(7, "p", &[], serde_json::json!([]))
                                ]))]
                            }
                        }
                    ]))
                ]))
            ]
        }));

        let doc = build_document(&mit_iframe, &leerer_ax()).unwrap();
        assert!(elements(&doc).any(|n| n.is_element("iframe")));
        assert!(
            !elements(&doc).any(|n| n.is_element("p")),
            "Inhalt des iframe-Dokuments darf nicht im Baum stehen"
        );
    }
}
