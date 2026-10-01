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

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use a11y_dom::{
    Arena, ArenaNode, ComputedStyle, Document, NameSource as SharedNameSource, Node, NodeKind,
    Rect, Rendering, Semantics,
};
use chromiumoxide::cdp::browser_protocol::dom::{
    GetDocumentParams, Node as CdpNode, ShadowRootType,
};
use chromiumoxide::cdp::browser_protocol::dom_snapshot::{
    CaptureSnapshotParams, CaptureSnapshotReturns,
};
use chromiumoxide::Page;
use tracing::{debug, warn};

use super::extractor::ChromeNameSources;
use crate::error::{AuditError, Result};
use a11y_perception::{AXTree, NameSource};

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
    /// Chromes eigene, feinere Angabe der Namensquelle, sofern mitgegeben
    /// (siehe [`CdpDocument::with_chrome_name_sources`]).
    chrome_name_source: Option<String>,
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
    /// Berechnete Layout-Stile, sofern mit [`fetch_dom_document_with_layout`]
    /// gebaut. Nur dann erfüllt [`CdpDocument::rendered`] Tier 3. Geteilt,
    /// weil ein Snapshot alle Dokumente der Seite abdeckt und jedes
    /// iframe-Dokument daraus sein eigenes [`CdpDocument`] baut.
    layout: Option<Arc<LayoutStyles>>,
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

    /// Übernimmt Chromes feine Namensquellen (Backend-Node-ID -> Kennung aus
    /// `extractor::name_source_label`). Danach
    /// liefert [`Semantics::name_source`] auch `aria-label`, `alt`,
    /// `aria-labelledby`, `label` und `value`, die der AXTree allein nicht
    /// mehr unterscheidet, und [`Self::chrome_name_source`] die Kennung selbst.
    pub fn with_chrome_name_sources(mut self, sources: &ChromeNameSources) -> Self {
        for (backend, facts) in &mut self.ax {
            if let Some(label) = sources.get(backend) {
                facts.name_source = shared_name_source(label);
                facts.chrome_name_source = Some(label.clone());
            }
        }
        self
    }

    /// Chromes Namensquelle als Kennung (`aria-label`, `alt`, `label`,
    /// `legend`, …), sofern sie über [`Self::with_chrome_name_sources`]
    /// mitgegeben wurde.
    pub fn chrome_name_source(&self, node: ArenaNode<'_>) -> Option<&str> {
        self.facts(node)?.chrome_name_source.as_deref()
    }

    /// Das Dokument mit Tier 3 ([`Rendering`]), wenn es mit Layout-Stilen
    /// gebaut wurde ([`fetch_dom_document_with_layout`]). Eine eigene Sicht statt einer Implementierung auf
    /// `CdpDocument` selbst: Es gibt nur `display` und `visibility`, keine
    /// Geometrie — Regeln auf Tier 3 sollen daran nicht stillschweigend
    /// laufen.
    pub fn rendered(&self) -> Option<RenderedCdpDocument<'_>> {
        Some(RenderedCdpDocument {
            doc: self,
            styles: self.layout.as_deref()?,
        })
    }

    /// Chromes Rolle, wie der AX-Baum sie meldet — ohne die Übersetzung, die
    /// [`Semantics::role`] vornimmt. Für den Vergleich mit Chrome
    /// (`accname_diff`), nicht für Regeln.
    pub fn chrome_role(&self, node: ArenaNode<'_>) -> Option<&str> {
        self.facts(node)?.role.as_deref()
    }

    fn facts(&self, node: ArenaNode<'_>) -> Option<&AxFacts> {
        self.ax.get(&self.backend_node_id(node)?)
    }
}

/// Chromes Rolle als ARIA-Rolle, wie die geteilten Regeln sie erwarten.
///
/// Chrome meldet im AX-Baum neben den ARIA-Rollen eigene Namen. `image` ist
/// ARIA `img`. Die groß geschriebenen (`LayoutTable`, `LayoutTableRow`,
/// `LayoutTableCell`, `DescriptionList`, `Figcaption`, `LabelText`,
/// `DisclosureTriangle`, `StaticText`, …) haben kein Gegenstück in ARIA 1.2:
/// Es sind Elemente, denen HTML-AAM keine Rolle gibt, oder — bei `Layout*` —
/// eine Tabelle, die Chrome nach eigener Heuristik als Layouttabelle einstuft,
/// also als präsentational. Sie werden zu „keine Rolle“. `a11y-rules` sieht
/// darin einen durchlässigen Zwischenknoten und urteilt nicht über seine
/// Attribute; als unbekannte Rolle hätte sie jede Kontextprüfung über ihm
/// abgebrochen (ein `role="tab"` in einer Layouttabellenzelle blieb ohne
/// Befund).
fn aria_role(chrome: &str) -> Option<&str> {
    match chrome {
        "image" => Some("img"),
        r if r.starts_with(|c: char| c.is_ascii_uppercase()) => None,
        r => Some(r),
    }
}

/// Berechnetes `display` und `visibility` je Backend-Node-ID.
///
/// Aus einem `DOMSnapshot`: Elemente mit Layout-Objekt tragen ihre
/// berechneten Werte. Ein Element **ohne** Layout-Objekt ist entweder
/// `display: none` (oder liegt darunter) oder `display: contents` — Letzteres
/// genau dann, wenn darunter etwas gerendert wird.
///
/// Dazu die Leerraum-Textknoten, die `DOM.getDocument` auslässt, der Snapshot
/// aber führt: Zwischen `<span>a</span> <span>b</span>` trennt nur dieser
/// Knoten die Wörter, sobald Inline-Elemente direkt anschließen.
#[derive(Debug, Clone, Default)]
pub struct LayoutStyles {
    by_backend: HashMap<i64, LayoutStyle>,
    /// Knoten, deren vorheriges Geschwister ein reiner Leerraum-Textknoten ist.
    whitespace_before: HashSet<i64>,
    /// Knoten, deren letztes Kind ein reiner Leerraum-Textknoten ist.
    whitespace_last: HashSet<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LayoutStyle {
    display: String,
    visibility: Option<String>,
}

impl LayoutStyles {
    pub fn len(&self) -> usize {
        self.by_backend.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_backend.is_empty()
    }

    /// Aus der Antwort von `DOMSnapshot.captureSnapshot` mit
    /// `computedStyles = ["display", "visibility"]`.
    pub fn from_snapshot(snapshot: &CaptureSnapshotReturns) -> Self {
        let string = |i: i64| {
            usize::try_from(i)
                .ok()
                .and_then(|i| snapshot.strings.get(i))
        };
        let mut by_backend = HashMap::new();
        let mut whitespace_before = HashSet::new();
        let mut whitespace_last = HashSet::new();
        for document in &snapshot.documents {
            let nodes = &document.nodes;
            let (Some(backend), Some(types), Some(parents)) = (
                nodes.backend_node_id.as_ref(),
                nodes.node_type.as_ref(),
                nodes.parent_index.as_ref(),
            ) else {
                continue;
            };
            let mut styles: Vec<Option<LayoutStyle>> = vec![None; backend.len()];
            let mut renders_below = vec![false; backend.len()];
            for (layout_index, &node_index) in document.layout.node_index.iter().enumerate() {
                let Ok(node_index) = usize::try_from(node_index) else {
                    continue;
                };
                if let Some(values) = document.layout.styles.get(layout_index) {
                    let values = values.inner();
                    styles[node_index] =
                        values
                            .first()
                            .and_then(|d| string(*d.inner()))
                            .map(|display| LayoutStyle {
                                display: display.clone(),
                                visibility: values.get(1).and_then(|v| string(*v.inner())).cloned(),
                            });
                }
                // Jeder Vorfahre eines gerenderten Knotens hat etwas
                // Gerendertes unter sich.
                let mut current = parents.get(node_index).copied().unwrap_or(-1);
                while let Ok(parent) = usize::try_from(current) {
                    if renders_below[parent] {
                        break;
                    }
                    renders_below[parent] = true;
                    current = parents.get(parent).copied().unwrap_or(-1);
                }
            }
            // Der Snapshot steht in Dokumentreihenfolge; je Elternknoten wird
            // vermerkt, ob zuletzt ein Leerraum-Textknoten kam.
            let values = nodes.node_value.as_ref();
            let mut pending = vec![false; backend.len()];
            for (i, id) in backend.iter().enumerate() {
                let Some(parent) = parents.get(i).and_then(|p| usize::try_from(*p).ok()) else {
                    continue;
                };
                let blank = types.get(i) == Some(&TEXT_NODE)
                    && values
                        .and_then(|v| v.get(i))
                        .and_then(|v| string(*v.inner()))
                        .is_some_and(|t| t.trim().is_empty());
                if blank {
                    pending[parent] = true;
                    continue;
                }
                if std::mem::take(&mut pending[parent]) {
                    whitespace_before.insert(*id.inner());
                }
            }
            for (i, open) in pending.into_iter().enumerate() {
                if open {
                    whitespace_last.insert(*backend[i].inner());
                }
            }
            for (i, id) in backend.iter().enumerate() {
                if types.get(i) != Some(&ELEMENT_NODE) {
                    continue;
                }
                let style = styles[i].take().unwrap_or_else(|| LayoutStyle {
                    display: if renders_below[i] { "contents" } else { "none" }.to_string(),
                    visibility: None,
                });
                by_backend.insert(*id.inner(), style);
            }
        }
        Self {
            by_backend,
            whitespace_before,
            whitespace_last,
        }
    }
}

/// Holt [`LayoutStyles`] für die ganze Seite in einem CDP-Aufruf.
async fn fetch_layout_styles(page: &Page) -> Result<LayoutStyles> {
    let params = CaptureSnapshotParams::new(vec!["display".to_string(), "visibility".to_string()]);
    let response = page
        .execute(params)
        .await
        .map_err(|e| AuditError::AXTreeExtractionFailed {
            reason: format!("DOMSnapshot.captureSnapshot fehlgeschlagen: {e}"),
        })?;
    Ok(LayoutStyles::from_snapshot(&response.result))
}

/// [`CdpDocument`] mit Tier 3, siehe [`CdpDocument::rendered`].
pub struct RenderedCdpDocument<'d> {
    doc: &'d CdpDocument,
    styles: &'d LayoutStyles,
}

impl Document for RenderedCdpDocument<'_> {
    type N<'a>
        = ArenaNode<'a>
    where
        Self: 'a;

    fn root(&self) -> Self::N<'_> {
        self.doc.root()
    }
}

/// Rolle und Name wie [`CdpDocument`] — damit die geteilten Regeln mit
/// Stilen laufen können (`a11y_rules::run_full`). Der Geltungsbereich der
/// Regeln sieht dann auch per CSS Verstecktes als verborgen, nicht nur das
/// `hidden`-Attribut.
impl Semantics for RenderedCdpDocument<'_> {
    fn role<'n>(&'n self, node: Self::N<'n>) -> Option<String> {
        self.doc.role(node)
    }

    fn accessible_name<'n>(&'n self, node: Self::N<'n>) -> Option<String> {
        self.doc.accessible_name(node)
    }

    fn name_source<'n>(&'n self, node: Self::N<'n>) -> Option<SharedNameSource> {
        self.doc.name_source(node)
    }

    fn is_ignored<'n>(&'n self, node: Self::N<'n>) -> bool {
        self.doc.is_ignored(node)
    }
}

impl Rendering for RenderedCdpDocument<'_> {
    fn computed_style<'n>(&'n self, node: Self::N<'n>) -> Option<ComputedStyle> {
        let style = self
            .styles
            .by_backend
            .get(&self.doc.backend_node_id(node)?)?;
        Some(ComputedStyle {
            color: None,
            background_color: None,
            font_size_px: None,
            font_weight: None,
            display: Some(style.display.clone()),
            visibility: style.visibility.clone(),
        })
    }

    /// Keine Geometrie — siehe [`CdpDocument::rendered`].
    fn bounds<'n>(&'n self, _node: Self::N<'n>) -> Option<Rect> {
        None
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
        self.chrome_role(node)
            .and_then(aria_role)
            .map(str::to_string)
    }

    fn accessible_name<'n>(&'n self, node: Self::N<'n>) -> Option<String> {
        self.facts(node)?.name.clone()
    }

    fn name_source<'n>(&'n self, node: Self::N<'n>) -> Option<SharedNameSource> {
        self.facts(node)?.name_source
    }

    /// Ein Element ohne Gegenstück im Accessibility-Tree hat Chrome gar nicht
    /// erst exponiert — etwa der Inhalt eines geschlossenen `<details>`, den
    /// Chrome über `content-visibility` ausblendet, während der DOMSnapshot
    /// `display` und `visibility` unverändert meldet. Ohne diese Regel bekam
    /// jeder Link darin einen leeren Namen und einen `FAIL` (geographia.eu:
    /// 22 Karten in `details.more`). Gilt nur, wenn der Baum überhaupt Fakten
    /// trägt; ein gescheiterter Abgleich soll nicht alles verschwinden lassen.
    fn is_ignored<'n>(&'n self, node: Self::N<'n>) -> bool {
        match self.facts(node) {
            Some(f) => f.ignored,
            None => !self.ax.is_empty() && node.kind() == NodeKind::Element,
        }
    }
}

/// Übersetzt auditmysites `NameSource` in die geteilte Fassung.
///
/// Bewusst unvollständig: auditmysites Extractor fasst `aria-label`, `alt` und
/// andere Attributquellen zu `Attribute` zusammen und `aria-labelledby` wie
/// `<label for>` zu `RelatedElement`. Diese Information ist an der Stelle schon
/// verloren, und sie hier zu raten hieße, eine Tatsache zu behaupten, die der
/// Baum nicht hergibt. Für die mehrdeutigen Fälle bleibt es deshalb bei `None`
/// — das Trait sieht genau das als Vorgabe vor. Die feine Angabe kommt getrennt
/// über [`CdpDocument::with_chrome_name_sources`].
fn map_name_source(src: NameSource) -> Option<SharedNameSource> {
    match src {
        NameSource::Contents => Some(SharedNameSource::Contents),
        NameSource::Placeholder => Some(SharedNameSource::Placeholder),
        NameSource::Title => Some(SharedNameSource::Title),
        NameSource::Attribute | NameSource::RelatedElement => None,
    }
}

/// Übersetzt Chromes feine Kennung in die geteilte Fassung. Quellen ohne
/// Gegenstück dort (`legend`, `tablecaption`, `figcaption`, `title-element`,
/// …) bleiben `None`.
fn shared_name_source(label: &str) -> Option<SharedNameSource> {
    Some(match label {
        "aria-label" => SharedNameSource::AriaLabel,
        "aria-labelledby" => SharedNameSource::AriaLabelledBy,
        "label" => SharedNameSource::Label,
        "title" => SharedNameSource::Title,
        "alt" => SharedNameSource::Alt,
        "placeholder" => SharedNameSource::Placeholder,
        "contents" => SharedNameSource::Contents,
        "value" => SharedNameSource::Value,
        _ => return None,
    })
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
                chrome_name_source: None,
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
struct Walk<'l, 'n> {
    builder: Option<a11y_dom::ArenaBuilder>,
    backend_ids: Vec<Option<i64>>,
    /// Wenn gesetzt, werden die Leerraum-Textknoten ergänzt, die
    /// `DOM.getDocument` auslässt (siehe [`LayoutStyles`]).
    layout: Option<&'l LayoutStyles>,
    /// Jeder Knoten nach Backend-ID, damit ein `<slot>` die ihm zugewiesenen
    /// Light-DOM-Knoten findet.
    by_backend: HashMap<i64, &'n CdpNode>,
}

/// Merkt sich jeden Knoten des Dokuments einschließlich der Shadow-Roots
/// nach seiner Backend-ID.
fn index_nodes<'n>(node: &'n CdpNode, map: &mut HashMap<i64, &'n CdpNode>) {
    map.insert(*node.backend_node_id.inner(), node);
    for child in node.shadow_roots.iter().flatten() {
        index_nodes(child, map);
    }
    for child in node.children.iter().flatten() {
        index_nodes(child, map);
    }
}

impl<'n> Walk<'_, 'n> {
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

        // Der flache Baum, wie ihn Browser und Assistenztechnik sehen: Die
        // Kinder eines Shadow-Roots hängen unter dem Host, der Fragment-Knoten
        // selbst bekommt kein Gegenstück. Die Light-DOM-Kinder des Hosts
        // erscheinen nur dort, wo ein `<slot>` sie aufnimmt; ohne Slot werden
        // sie nicht dargestellt. Hingen sie neben dem Shadow-Inhalt, stand auf
        // sachsen-anhalt.de jede `<ul><slot>` leer da und jedes per Slot
        // gelieferte `role="listitem"` ausserhalb einer Liste.
        //
        // User-Agent-Shadow-Roots bleiben aussen vor: Die Tag-/Monat-/Jahr-
        // Felder eines `<input type=date>` sind `role="spinbutton"` ohne
        // `aria-valuenow` -- aber sie gehoeren dem Browser, nicht dem Autor,
        // und kein Autor kann sie beheben. Die frueheren DOM-Regeln kamen per
        // Skript gar nicht an sie heran (#656); ohne diesen Filter meldete
        // `aria/required-attribute-missing` jedes Datumsfeld (#690).
        let mut is_host = false;
        for shadow in node
            .shadow_roots
            .iter()
            .flatten()
            .filter(|s| s.shadow_root_type != Some(ShadowRootType::UserAgent))
        {
            is_host = true;
            self.children(shadow);
        }

        let assigned: Vec<&'n CdpNode> = if name == "slot" {
            node.distributed_nodes
                .iter()
                .flatten()
                .filter_map(|b| self.by_backend.get(b.backend_node_id.inner()).copied())
                .collect()
        } else {
            Vec::new()
        };
        if !assigned.is_empty() {
            for child in assigned {
                self.node(child);
            }
        } else if !is_host {
            // Ein Slot ohne Zuweisung zeigt seinen Ersatzinhalt.
            self.children(node);
        }
        self.with(|b| b.close());
    }

    fn children(&mut self, node: &CdpNode) {
        let layout = self.layout;
        let blank_before = |n: &CdpNode| {
            layout.is_some_and(|l| l.whitespace_before.contains(n.backend_node_id.inner()))
        };
        for child in node.children.iter().flatten() {
            if blank_before(child) {
                self.blank();
            }
            self.node(child);
        }
        if layout.is_some_and(|l| l.whitespace_last.contains(node.backend_node_id.inner())) {
            self.blank();
        }
    }

    /// Ein ergänzter Leerraum-Textknoten, ohne Backend-ID.
    fn blank(&mut self) {
        self.with(|b| b.text(" "));
        self.record(None);
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
            // Baums falsche Befunde erzeugen würde. Jedes iframe-Dokument
            // bekommt ein eigenes `CdpDocument` (`audit::frames`, #715).
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
    build(root, ax_tree, None)
}

fn build(
    root: &CdpNode,
    ax_tree: &AXTree,
    layout: Option<Arc<LayoutStyles>>,
) -> Result<CdpDocument> {
    let html = find_html(root).ok_or_else(|| AuditError::AXTreeExtractionFailed {
        reason: "DOM enthält kein <html>-Element".to_string(),
    })?;

    let mut by_backend = HashMap::new();
    index_nodes(root, &mut by_backend);
    let mut walk = Walk {
        builder: Some(Arena::builder()),
        backend_ids: Vec::new(),
        layout: layout.as_deref(),
        by_backend,
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
        layout,
    })
}

/// Holt den vollständigen DOM über CDP und verbindet ihn mit dem AXTree.
pub async fn fetch_dom_document(page: &Page, ax_tree: &AXTree) -> Result<CdpDocument> {
    let root = get_document(page).await?;
    let doc = build_document(&root, ax_tree)?;
    if doc.is_empty() {
        warn!("DOM für die geteilten Regeln ist leer");
    }
    Ok(doc)
}

/// Wie [`fetch_dom_document`], dazu berechnetes `display`/`visibility` und die
/// Leerraum-Textknoten aus einem `DOMSnapshot` — die Grundlage für
/// [`CdpDocument::rendered`]. Für den accname-Differentiallauf und die
/// geteilten Regeln (`wcag::shared::run_shared_rules`).
pub async fn fetch_dom_document_with_layout(page: &Page, ax_tree: &AXTree) -> Result<CdpDocument> {
    let root = get_document(page).await?;
    let layout = fetch_layout_styles(page).await?;
    build(&root, ax_tree, Some(Arc::new(layout)))
}

/// Der rohe CDP-Dokumentbaum der Seite samt Layout-Stilen, einmal geholt.
///
/// Aus demselben Abruf entstehen das [`CdpDocument`] des Hauptdokuments und
/// je eines für jedes im Prozess der Seite gerenderte iframe: Dessen Inhalt
/// steht als `content_document` am iframe-Element, und der `DOMSnapshot`
/// deckt alle Dokumente ab (über die Backend-ID verschlüsselt).
pub struct DomCapture {
    pub root: CdpNode,
    /// `None`, wenn der `DOMSnapshot` scheiterte -- dann laufen die Regeln
    /// ohne Stile statt gar nicht.
    layout: Option<Arc<LayoutStyles>>,
}

impl DomCapture {
    /// Das Hauptdokument, verbunden mit dessen AXTree.
    pub fn document(&self, ax_tree: &AXTree) -> Result<CdpDocument> {
        self.document_at(&self.root, ax_tree)
    }

    /// Das Dokument unter `root` -- etwa der `content_document` eines
    /// iframes --, verbunden mit dem AXTree genau dieses Dokuments.
    pub fn document_at(&self, root: &CdpNode, ax_tree: &AXTree) -> Result<CdpDocument> {
        build(root, ax_tree, self.layout.clone())
    }
}

/// Holt [`DomCapture`]. Scheitert nur der Snapshot, fehlen die Stile.
pub async fn fetch_dom_capture(page: &Page) -> Result<DomCapture> {
    let root = get_document(page).await?;
    let layout = match fetch_layout_styles(page).await {
        Ok(layout) => Some(Arc::new(layout)),
        Err(e) => {
            warn!("DOMSnapshot fuer die geteilten Regeln fehlgeschlagen: {e}");
            None
        }
    };
    Ok(DomCapture { root, layout })
}

async fn get_document(page: &Page) -> Result<CdpNode> {
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
    Ok(response.result.root.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use a11y_dom::{elements, NodeKind};

    /// Snapshot: `<div>` (block, gerendert) mit `<span>` ohne Layout-Objekt
    /// über gerendertem Text (`display: contents`), einem Leerraum-Textknoten,
    /// einem `<span>` ohne Gerendertes darunter (`none`) und einem
    /// abschließenden Leerraum-Textknoten.
    #[test]
    fn layout_styles_leiten_contents_none_und_leerraum_ab() {
        let snapshot: CaptureSnapshotReturns = serde_json::from_value(serde_json::json!({
            "strings": ["block", "visible", "hidden", " ", "a"],
            "documents": [{
                "documentURL": 0, "title": 0, "baseURL": 0, "contentLanguage": 0,
                "encodingName": 0, "publicId": 0, "systemId": 0, "frameId": 0,
                "nodes": {
                    "parentIndex": [-1, 0, 1, 2, 1, 1, 5, 1],
                    "nodeType": [9, 1, 1, 3, 3, 1, 3, 3],
                    "nodeValue": [-1, -1, -1, 4, 3, -1, 4, 3],
                    "backendNodeId": [1, 2, 3, 4, 5, 6, 7, 8]
                },
                "layout": {
                    "nodeIndex": [1, 3],
                    "styles": [[0, 2], []],
                    "bounds": [[0, 0, 1, 1], [0, 0, 1, 1]],
                    "text": [-1, -1],
                    "stackingContexts": {"index": []}
                },
                "textBoxes": {"layoutIndex": [], "bounds": [], "start": [], "length": []}
            }]
        }))
        .unwrap();
        let styles = LayoutStyles::from_snapshot(&snapshot);
        let get = |id| styles.by_backend.get(&id).cloned();
        assert_eq!(
            get(2),
            Some(LayoutStyle {
                display: "block".into(),
                visibility: Some("hidden".into())
            })
        );
        assert_eq!(get(3).map(|s| s.display), Some("contents".into()));
        assert_eq!(get(6).map(|s| s.display), Some("none".into()));
        assert_eq!(get(4), None, "Textknoten tragen keinen Stil");
        assert_eq!(styles.whitespace_before, HashSet::from([6]));
        assert_eq!(styles.whitespace_last, HashSet::from([2]));
    }

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

    /// Der User-Agent-Shadow-Root eines `<input type=date>` traegt die
    /// Datumsfelder als `role="spinbutton"`. Die gehoeren dem Browser; im
    /// Dokument erscheinen sie nicht, ein offener Shadow-Root daneben schon.
    #[test]
    fn user_agent_shadow_root_bleibt_aussen_vor() {
        let host = |id: i64, tag: &str, typ: &str, kind: serde_json::Value| {
            serde_json::json!({
                "nodeId": id, "backendNodeId": id, "nodeType": 1,
                "nodeName": tag.to_uppercase(), "localName": tag,
                "nodeValue": "", "children": [],
                "shadowRoots": [{
                    "nodeId": id + 1, "backendNodeId": id + 1, "nodeType": 11,
                    "nodeName": "#document-fragment", "localName": "",
                    "nodeValue": "", "shadowRootType": typ,
                    "children": [kind]
                }]
            })
        };
        let doc = build_document(
            &cdp(serde_json::json!({
                "nodeId": 1, "backendNodeId": 1, "nodeType": 9,
                "nodeName": "#document", "localName": "", "nodeValue": "",
                "children": [
                    element(2, "html", &[], serde_json::json!([
                        element(3, "body", &[], serde_json::json!([
                            host(4, "input", "user-agent",
                                element(6, "span", &["role", "spinbutton"], serde_json::json!([]))),
                            host(7, "my-card", "open",
                                element(9, "button", &[], serde_json::json!([])))
                        ]))
                    ]))
                ]
            })),
            &leerer_ax(),
        )
        .unwrap();

        assert!(elements(&doc).all(|n| !n.is_element("span")));
        assert!(elements(&doc).any(|n| n.is_element("button")));
    }

    /// Muster von sachsen-anhalt.de: Die Liste liegt im Shadow-Root, die
    /// Eintraege kommen per Slot aus dem Light-DOM des Hosts. Im flachen Baum
    /// haengen sie unter dem `<slot>` in der `<ul>`; ein nicht zugewiesenes
    /// Light-Kind erscheint gar nicht.
    #[test]
    fn slot_nimmt_die_zugewiesenen_light_kinder_auf() {
        let slot = serde_json::json!({
            "nodeId": 7, "backendNodeId": 7, "nodeType": 1,
            "nodeName": "SLOT", "localName": "slot", "nodeValue": "", "children": [],
            "distributedNodes": [
                {"nodeType": 1, "nodeName": "MUSE-LINK", "backendNodeId": 10},
                {"nodeType": 1, "nodeName": "MUSE-LINK", "backendNodeId": 11}
            ]
        });
        let shadow = serde_json::json!({
            "nodeId": 5, "backendNodeId": 5, "nodeType": 11,
            "nodeName": "#document-fragment", "localName": "", "nodeValue": "",
            "children": [element(6, "ul", &[], serde_json::json!([slot]))]
        });
        let light = serde_json::json!([
            element(
                10,
                "muse-link",
                &["role", "listitem"],
                serde_json::json!([])
            ),
            element(
                11,
                "muse-link",
                &["role", "listitem"],
                serde_json::json!([])
            ),
            element(12, "span", &["slot", "unbenutzt"], serde_json::json!([]))
        ]);
        let list = serde_json::json!({
            "nodeId": 4, "backendNodeId": 4, "nodeType": 1,
            "nodeName": "MUSE-LINK-LIST", "localName": "muse-link-list", "nodeValue": "",
            "children": light, "shadowRoots": [shadow]
        });
        let body = element(3, "body", &[], serde_json::json!([list]));
        let host = cdp(serde_json::json!({
            "nodeId": 1, "backendNodeId": 1, "nodeType": 9,
            "nodeName": "#document", "localName": "", "nodeValue": "",
            "children": [element(2, "html", &[], serde_json::json!([body]))]
        }));

        let doc = build_document(&host, &leerer_ax()).unwrap();
        let items: Vec<_> = elements(&doc)
            .filter(|n| n.is_element("muse-link"))
            .collect();
        assert_eq!(items.len(), 2);
        for item in items {
            assert_eq!(item.parent().unwrap().local_name(), "slot");
            assert_eq!(item.parent().unwrap().parent().unwrap().local_name(), "ul");
        }
        assert!(!elements(&doc).any(|n| n.is_element("span")));
    }

    /// Ein Slot ohne Zuweisung zeigt seinen Ersatzinhalt.
    #[test]
    fn slot_ohne_zuweisung_zeigt_ersatzinhalt() {
        let host = cdp(serde_json::json!({
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
                                "children": [element(6, "slot", &[], serde_json::json!([
                                    element(8, "button", &[], serde_json::json!([]))
                                ]))]
                            }]
                        }
                    ]))
                ]))
            ]
        }));

        let doc = build_document(&host, &leerer_ax()).unwrap();
        let button = elements(&doc).find(|n| n.is_element("button")).unwrap();
        assert_eq!(button.parent().unwrap().local_name(), "slot");
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
        assert_eq!(doc.role(img).as_deref(), Some("img"));
        assert_eq!(doc.chrome_role(img), Some("image"));
        assert_eq!(doc.accessible_name(img).as_deref(), Some("Firmenlogo"));

        let link = elements(&doc).find(|n| n.is_element("a")).unwrap();
        assert_eq!(doc.role(link).as_deref(), Some("link"));
        assert!(doc.accessible_name(link).is_none());

        // Ein Knoten ohne AX-Eintrag behauptet nichts.
        let head = elements(&doc).find(|n| n.is_element("head")).unwrap();
        assert!(doc.role(head).is_none());
    }

    /// Chromes eigene Rollennamen erreichen die geteilten Regeln nicht:
    /// `LayoutTable*` und die übrigen groß geschriebenen haben keine
    /// ARIA-Rolle, `image` ist `img` (B1, auditmysite#691).
    #[test]
    fn chromes_eigene_rollen_werden_uebersetzt() {
        assert_eq!(aria_role("image"), Some("img"));
        assert_eq!(aria_role("tablist"), Some("tablist"));
        assert_eq!(aria_role("generic"), Some("generic"));
        for intern in [
            "LayoutTable",
            "LayoutTableRow",
            "LayoutTableCell",
            "DescriptionList",
            "Figcaption",
            "LabelText",
            "StaticText",
        ] {
            assert_eq!(aria_role(intern), None, "{intern}");
        }
    }

    #[test]
    fn ignorierte_knoten_werden_als_solche_gemeldet() {
        let ax = AXTree::from_nodes(vec![ax_knoten(8, "link", None, true)]);
        let doc = build_document(&beispielseite(), &ax).unwrap();
        let link = elements(&doc).find(|n| n.is_element("a")).unwrap();
        assert!(doc.is_ignored(link));
    }

    /// Ohne Gegenstueck im AX-Baum hat Chrome das Element nicht exponiert
    /// (geographia.eu: Links in einem geschlossenen `<details>`). Mit leerem
    /// AX-Baum bleibt dagegen alles sichtbar -- ein gescheiterter Abgleich
    /// darf nicht jeden Befund verschlucken.
    #[test]
    fn element_ohne_ax_gegenstueck_gilt_als_ignoriert() {
        let ax = AXTree::from_nodes(vec![ax_knoten(7, "image", Some("Firmenlogo"), false)]);
        let doc = build_document(&beispielseite(), &ax).unwrap();
        let link = elements(&doc).find(|n| n.is_element("a")).unwrap();
        assert!(doc.is_ignored(link));

        let ohne_ax = build_document(&beispielseite(), &leerer_ax()).unwrap();
        let link = elements(&ohne_ax).find(|n| n.is_element("a")).unwrap();
        assert!(!ohne_ax.is_ignored(link));
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
        // Was uebrig bleibt, ist Tier 3: Ohne gerenderte Stile koennen
        // Kontrast- und Layoutregeln nichts sagen -- und sagen genau das,
        // statt zu schweigen. Die Liste kommt aus dem Crate selbst, damit eine
        // neue Tier-3-Regel diesen Test nicht bricht, eine fehlende Tier-1/2-
        // Regel aber schon.
        let mut nicht_gelaufen: Vec<&str> = report
            .rule_runs
            .iter()
            .filter(|r| r.not_run.is_some())
            .map(|r| r.rule_id.as_str())
            .collect();
        nicht_gelaufen.sort_unstable();
        let mut tier3: Vec<&str> = a11y_rules::rendering_metas()
            .iter()
            .flat_map(|m| m.ids.iter().copied())
            .collect();
        tier3.sort_unstable();
        assert_eq!(
            nicht_gelaufen, tier3,
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

    fn ax_mit_quelle(backend: i64, source: NameSource) -> AXTree {
        AXTree::from_nodes(vec![a11y_perception::AXNode {
            node_id: format!("ax{backend}"),
            ignored: false,
            ignored_reasons: Vec::new(),
            role: Some("image".into()),
            name: Some("Logo".into()),
            name_source: Some(source),
            description: None,
            value: None,
            properties: Vec::new(),
            child_ids: Vec::new(),
            parent_id: None,
            backend_dom_node_id: Some(backend),
        }])
    }

    /// Ohne feine Quellen bleibt `Attribute` unbestimmt; mit ihnen kommt die
    /// tatsächliche Quelle an, und unbekannte Kennungen bleiben `None`.
    #[test]
    fn feine_namensquellen_ersetzen_die_grobe_zuordnung() {
        let ax = ax_mit_quelle(7, NameSource::Attribute);
        let doc = build_document(&beispielseite(), &ax).unwrap();
        let img = elements(&doc).find(|n| n.is_element("img")).unwrap();
        assert_eq!(doc.name_source(img), None);
        assert_eq!(doc.chrome_name_source(img), None);

        for (label, erwartet) in [
            ("alt", Some(SharedNameSource::Alt)),
            ("aria-label", Some(SharedNameSource::AriaLabel)),
            ("aria-labelledby", Some(SharedNameSource::AriaLabelledBy)),
            ("label", Some(SharedNameSource::Label)),
            ("value", Some(SharedNameSource::Value)),
            ("figcaption", None),
        ] {
            let doc = build_document(&beispielseite(), &ax)
                .unwrap()
                .with_chrome_name_sources(&HashMap::from([(7, label.to_string())]));
            let img = elements(&doc).find(|n| n.is_element("img")).unwrap();
            assert_eq!(doc.name_source(img), erwartet, "{label}");
            assert_eq!(doc.chrome_name_source(img), Some(label));
        }
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
