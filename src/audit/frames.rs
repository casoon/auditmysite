//! WCAG rules inside iframes (#715).
//!
//! `getFullAXTree` and the shared DOM rules see the main document only, so a
//! `role="dialog"` without a name or a broken tablist inside an iframe went
//! unreported while axe-core, which runs in every frame, found it. This pass
//! audits every frame rendered in the page's own process — with site
//! isolation switched off in the audit browser (#720), cross-site frames
//! included: it fetches the
//! frame's AX tree (`frameId`) and builds a DOM document from the iframe's
//! `content_document`, then runs the **element-level** rules on both.
//!
//! Page-level rules stay on the top document, as in axe-core: a widget frame
//! has no reason to carry a `main` landmark, an `h1`, a title, a viewport or
//! a skip link, and reporting those for it would be false.
//!
//! Findings keep a composite selector `"{iframe} [frame] {element}"` and a
//! `frame:`-prefixed node id, so later steps that look node ids up in the
//! main AX tree never resolve a frame finding to a main-document element.
//!
//! Which frames were audited and which were skipped — and why — is recorded
//! in the execution block (`execution.frames`), so a skipped frame never
//! reads as a clean one.

use std::collections::BTreeSet;
use std::future::Future;
use std::sync::Arc;
use std::time::{Duration, Instant};

use chromiumoxide::cdp::browser_protocol::dom::{
    BackendNodeId, Node as CdpNode, ResolveNodeParams, ShadowRootType,
};
use chromiumoxide::cdp::js_protocol::runtime::{CallFunctionOnParams, EvaluateParams};
use chromiumoxide::Page;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::accessibility::{AXTree, CdpDocument, DomCapture};
use crate::audit::exclusion::ExclusionScope;
use crate::model::{FrameOrigin, WcagLevel};
use crate::wcag::types::fold_rule_run;
use crate::wcag::{self, RuleFilterConfig, RuleRun, Violation, WcagResults};

/// Tree rules that run inside frames, by the axe id that gates them in
/// `wcag::engine`. Only rules that judge single elements or their
/// relationships: the keyboard-trap note. The names of image-role elements
/// run as the shared `names/required-missing` since #1181. The redundant
/// description runs as the shared `names/description-duplicates-name` since
/// a11y-rules 0.31 (#919). Roles, their required parents and children and
/// ambiguous `aria-owns` run as shared rules since #691, the names of
/// dialogs, summaries and role-based widgets, label-in-name and live-region
/// roles since #692, form labels, structure and errors since #693, keyboard
/// reachability since #694, table headers since #697 (see
/// [`FRAME_SHARED_RULES`]).
///
/// Left out on purpose — they judge the document as a whole and belong to
/// the top frame only: `focus-visible` (fires only when the whole page has
/// nothing focusable), `unusual-words` and `help`.
pub const FRAME_TREE_RULES: &[&str] = &["keyboard-trap"];

/// Shared rules (`wcag::shared::SHARED_RULES`) that run inside frames — the
/// element-level ones. Left out, as page-level: `document/lang-*` (the
/// frame's `lang` stays with `iframe_rules`' `html-has-lang`),
/// `document/title-*`, `headings/h1-*`, `headings/none`,
/// `headings/skip-level`, `zoom/*`, `landmarks/*` and
/// `keyboard/skip-link-missing`.
///
/// From #694 the keyboard reachability (`keyboard/focusable-no-role`,
/// `keyboard/interactive-not-focusable`, until then the tree rule
/// `keyboard`) and two element-level pattern checks
/// (`dialog/focusable-missing`, `patterns/accordion-controls-missing`) run
/// in frames; the pattern checks ran in the top frame only before.
///
/// From #695 the click-handler and fake-link checks run in frames too; as
/// JavaScript page rules (`click-events-have-key-events`, `link-as-button`)
/// they saw the top frame only. `navigation/location-missing` judges the
/// page and stays with the top frame.
///
/// The image and media rules (#696) judge single elements and run in frames;
/// as JavaScript page rules they saw the top frame only. The tree rule
/// `video-caption` (unnamed `application`/`img`, named decorative elements)
/// ran in frames and is gone: the shared rules cover those cases.
///
/// From #697 the table, role, name and tooltip checks and unexpanded
/// abbreviations run in frames; table headers ran there as the tree rule
/// `th-has-data-cells`, the others as page rules in the top frame only. The
/// language and document checks (`document/lang-mismatch`, `language/*`),
/// `timing/meta-refresh` and `headings/section-without-heading` judge the
/// document and stay with the top frame.
///
/// The stylesheet rules (`focus/outline-removed`, `motion/*`,
/// `orientation/content-hidden`, `text/*`) don't run in frames: frame
/// documents get no stylesheets (`run_shared_rules` without sheets), and the
/// JavaScript rules they replace read the top document only.
///
/// The contrast and link-colour rules run in frames since #783: they judge
/// single elements, and the frame document has its layout.
///
/// The form rules (#693) judge single fields and forms and run in frames —
/// an embedded sign-in or contact form is the usual case. Before #693 only
/// the tree-based ones did (labels, groups, errors, title-only labels); the
/// DOM-based ones (`autocomplete`, submit, redundant entry, context change,
/// CAPTCHA) ran in the top frame only.
pub const FRAME_SHARED_RULES: &[&str] = &[
    "headings/empty",
    "ids/duplicate",
    "keyboard/positive-tabindex",
    "keyboard/hidden-focusable",
    "lists/invalid-structure",
    "lists/empty",
    "lists/item-outside-list",
    "lists/term-without-definition",
    "tables/header-missing",
    "tables/name-missing",
    "tables/presentational-with-headers",
    "aria/reference-missing",
    "aria/required-attribute-missing",
    "svg/name-missing",
    "images/alt-missing",
    "images/alt-suspicious",
    "links/name-missing",
    "buttons/name-missing",
    "aria/role-invalid",
    "aria/role-abstract",
    "aria/attribute-unknown",
    "aria/attribute-value-invalid",
    "aria/owns-conflict",
    "aria/tab-selected-missing",
    "aria/tabpanel-missing",
    "aria/combobox-popup-missing",
    "popover/target-missing",
    "popover/target-invalid",
    "inert/dialog-inert",
    "aria/attribute-not-allowed",
    "aria/attribute-prohibited",
    "aria/required-parent-missing",
    "aria/required-children-missing",
    "names/required-missing",
    "names/indicator-name-missing",
    "images/role-img-name-missing",
    "names/symbol-only",
    "names/description-duplicates-name",
    "dialog/name-missing",
    "dialog/modal-unmarked",
    "summary/name-missing",
    "status/live-overridden",
    "label-in-name/mismatch",
    "forms/label-missing",
    "forms/placeholder-as-label",
    "forms/autocomplete-invalid",
    "forms/purpose-missing",
    "forms/error-unidentified",
    "forms/group-missing",
    "forms/group-name-missing",
    "forms/required-unmarked",
    "forms/instructions-missing",
    "forms/title-only-label",
    "forms/no-submit",
    "forms/redundant-entry",
    "context/on-input",
    "context/on-focus",
    "context/autofocus",
    "auth/captcha",
    // The paste probe walks same-origin frames and open shadow roots, so the
    // fields of an embedded sign-in form are measured too (#1006). The other
    // probe rules judge the top document: its frames, its fixed bars, its
    // root (the abolished JavaScript rules read the top document only).
    "auth/paste-blocked",
    "keyboard/focusable-no-role",
    "keyboard/interactive-not-focusable",
    "dialog/focusable-missing",
    "patterns/accordion-controls-missing",
    "keyboard/click-handler-not-focusable",
    "links/used-as-button",
    // Linkzweck: beurteilt einzelne Links, läuft auch in Frames.
    "links/generic-name",
    "links/url-name",
    "links/single-character-name",
    "links/new-window-unannounced",
    "links/purpose-link-only",
    "images/area-alt-missing",
    "images/input-alt-missing",
    "objects/alt-missing",
    "images/server-side-map",
    "media/audio-autoplay",
    // The playback probe walks same-origin frames like the paste probe: a
    // measured medium leaves `media/audio-autoplay`, so its measured verdict
    // has to run there too.
    "media/audio-autoplay-measured",
    // Judges single videos; `video-caption` (track files, embeds) reads the
    // top document only, so a video in a frame is never reported twice.
    "media/video-captions-missing",
    "frames/name-missing",
    "tables/header-without-data",
    "tables/data-undetermined",
    "tables/headers-attr-invalid",
    "aria/role-redundant",
    "names/title-only",
    "patterns/tooltip-unreferenced",
    "language/abbreviation-unexpanded",
    // Contrast and link colour judge single elements against their own
    // document's rendering; the frame document carries the frame's layout
    // from the same DOM snapshot (#783).
    "contrast/text-insufficient",
    "contrast/text-undetermined",
    "contrast/text-enhanced",
    "contrast/control-insufficient",
    "contrast/control-undetermined",
    "color/link-indistinct",
];

// ── Report block (canonical English) ─────────────────────────────────────────

/// Frame coverage of one page: which iframes were audited, which skipped.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FrameCoverage {
    #[serde(default)]
    pub audited: Vec<FrameEntry>,
    #[serde(default)]
    pub skipped: Vec<FrameEntry>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FrameEntry {
    /// The iframe element in the page; nested frames as
    /// `"{outer} [frame] {inner}"`.
    pub selector: String,
    /// The frame document's URL, or the `src` attribute when there is none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Why the frame was not audited (skipped frames only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<FrameSkipReason>,
    /// What the audit could not settle in this audited frame (#1180): its
    /// findings stand, but the document may hold content they did not see.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gap: Option<FrameGap>,
    /// The viewport passes this entry applies to.
    #[serde(default)]
    pub viewports: Vec<String>,
    /// What the frame's rules did, one outcome per rule and viewport
    /// (audited frames only; #778). The page's own `rule_outcomes` hold the
    /// frames' findings folded in; this is where a frame's rule that did not
    /// run stays visible.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub outcomes: Vec<RuleRun>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum FrameSkipReason {
    /// Rendered out of process: not reachable through the page session. The
    /// audit browser runs without site isolation (#720), so this is left for
    /// a Chrome that ignores those flags. `frame-tested` asks for a manual review of it.
    CrossOrigin,
    /// The iframe is not perceivable: hidden, `aria-hidden`, `role=none`/
    /// `presentation`, `display:none`/`visibility:hidden`, at most 1px, or
    /// ignored in the parent's AX tree.
    Hidden,
    /// The iframe lies inside an audit exclusion (#645).
    Excluded,
    /// Chrome returned no AX tree for the frame.
    AxTreeFailed,
    /// The frame's DOM has no `<html>` element to run the rules on.
    DocumentUnavailable,
    /// The frame has a source (`src`, or a lazy-load `data-src`) but still
    /// shows its initial `about:blank` document: it never loaded, also not
    /// after the lazy frames were scrolled into view and waited for
    /// (`load_lazy_frames`). Its content was not audited; the blank
    /// placeholder is no stand-in for it.
    NotLoaded,
}

/// A gap in an audited frame's result (#1180): the frame's result is
/// `partial`, the audit quality says so and the frame is listed for a manual
/// look.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum FrameGap {
    /// The frame's accessibility tree held elements its DOM capture lacked
    /// (the frame kept rendering after the capture), and the DOM and layout
    /// captured again did not catch up within the retry budget
    /// ([`FRAME_SETTLE_BUDGET_MS`]), or could not be captured again. The
    /// DOM-based rules judged a document older than the rendered one.
    DomUnsettled,
}

/// What one viewport pass saw of one frame.
#[derive(Debug, Clone, PartialEq)]
pub struct FrameObservation {
    pub selector: String,
    pub url: Option<String>,
    /// `None` = audited.
    pub skipped: Option<FrameSkipReason>,
    /// The gap of an audited frame, see [`FrameGap`].
    pub gap: Option<FrameGap>,
    /// The frame's rule outcomes in this pass, tagged with its viewport.
    /// Empty for a skipped frame.
    pub outcomes: Vec<RuleRun>,
}

impl FrameCoverage {
    /// Adds one pass's observations; the same frame seen in both passes
    /// with the same result becomes one entry listing both viewports.
    pub fn record(&mut self, viewport: &str, observations: &[FrameObservation]) {
        for obs in observations {
            let list = match obs.skipped {
                None => &mut self.audited,
                Some(_) => &mut self.skipped,
            };
            match list.iter_mut().find(|e| {
                e.selector == obs.selector
                    && e.url == obs.url
                    && e.reason == obs.skipped
                    && e.gap == obs.gap
            }) {
                Some(entry) => {
                    if !entry.viewports.iter().any(|v| v == viewport) {
                        entry.viewports.push(viewport.to_string());
                    }
                    for run in &obs.outcomes {
                        fold_rule_run(&mut entry.outcomes, run.clone());
                    }
                }
                None => list.push(FrameEntry {
                    selector: obs.selector.clone(),
                    url: obs.url.clone(),
                    reason: obs.skipped,
                    gap: obs.gap,
                    viewports: vec![viewport.to_string()],
                    outcomes: obs.outcomes.clone(),
                }),
            }
        }
    }
}

// ── Rule selection ────────────────────────────────────────────────────────────

/// The tree-rule filter for frames: [`FRAME_TREE_RULES`], narrowed by the
/// user's own `[rules]` filter. `None` when nothing is left — an empty
/// `enabled_only_rules` would mean "run everything".
pub fn frame_rule_filter(user: &RuleFilterConfig) -> Option<RuleFilterConfig> {
    let enabled: Vec<String> = FRAME_TREE_RULES
        .iter()
        .filter(|id| user.should_run(id))
        .map(|id| id.to_string())
        .collect();
    (!enabled.is_empty()).then_some(RuleFilterConfig {
        disabled_rules: Vec::new(),
        enabled_only_rules: enabled,
    })
}

/// The rules an audited frame must account for with exactly one outcome per
/// viewport (#778): the tree rules the level and the user's filter let run
/// ([`frame_rule_filter`]) and every frame shared rule -- those report as
/// not run (`above_wcag_level`, `disabled_by_rule_filter`) rather than
/// vanish. Derived from the same lists the pass runs, never from a second one.
pub fn expected_frame_rules(level: WcagLevel, user: &RuleFilterConfig) -> Vec<String> {
    let mut rules: Vec<String> = frame_rule_filter(user)
        .map(|filter| {
            FRAME_TREE_RULES
                .iter()
                .filter(|id| filter.should_run(id))
                .filter(|id| {
                    crate::wcag::expectation::registered_min_level(id)
                        .is_none_or(|min| min <= level)
                })
                .map(|id| id.to_string())
                .collect()
        })
        .unwrap_or_default();
    rules.extend(FRAME_SHARED_RULES.iter().map(|id| id.to_string()));
    rules
}

/// Keeps only the findings and outcomes of [`FRAME_SHARED_RULES`].
fn retain_frame_shared_rules(results: &mut WcagResults) {
    let keep = |id: Option<&str>| id.is_some_and(|id| FRAME_SHARED_RULES.contains(&id));
    for list in [
        &mut results.violations,
        &mut results.warnings,
        &mut results.positives,
        &mut results.not_testables,
    ] {
        list.retain(|v| keep(v.rule_id.as_deref()));
    }
    results
        .rule_outcomes
        .retain(|o| keep(Some(o.rule_id.as_str())));
    // Counted over every shared rule, page-level ones included.
    results.passes = 0;
}

/// `"{iframe} [frame] {inner}"`; without an inner selector the frame itself.
pub fn composite_selector(frame_selector: &str, inner: Option<&str>) -> String {
    match inner.map(str::trim).filter(|s| !s.is_empty()) {
        Some(inner) => format!("{frame_selector} [frame] {inner}"),
        None => format!("{frame_selector} [frame]"),
    }
}

/// Ties a frame finding to its frame: composite selector, backend node id
/// from the frame's AX tree, `frame:`-prefixed node id, `iframe-content` tag,
/// and the origin of the frame against the page (#1109).
fn attach_to_frame(
    v: &mut Violation,
    frame_selector: &str,
    frame_id: &str,
    ax: &AXTree,
    origin: &FrameOrigin,
) {
    v.origin = origin.origin;
    v.provider = origin.provider.clone();
    v.frame_host = origin.host.clone();
    if v.backend_node_id.is_none() {
        v.backend_node_id = ax.get_node(&v.node_id).and_then(|n| n.backend_dom_node_id);
    }
    v.selector = Some(composite_selector(frame_selector, v.selector.as_deref()));
    v.node_id = format!("frame:{frame_id}:{}", v.node_id);
    if !v.tags.iter().any(|t| t == "iframe-content") {
        v.tags.push("iframe-content".to_string());
    }
}

/// Adds the frame's rule outcomes to the page's: the frame is part of the
/// page, and an outcome is keyed `(rule_id, viewport)`.
fn fold_outcomes(page: &mut WcagResults, frame: Vec<RuleRun>) {
    for run in frame {
        page.fold_outcome(run);
    }
}

// ── The pass ──────────────────────────────────────────────────────────────────

pub(crate) struct FramePassOutput {
    pub results: WcagResults,
    pub excluded: Vec<Violation>,
    pub frames: Vec<FrameObservation>,
}

pub(crate) struct FramePassConfig<'a> {
    pub level: WcagLevel,
    pub rule_filter: &'a RuleFilterConfig,
    pub lang: &'a str,
    pub viewport: &'static str,
    /// The page's URL, for the origin of each frame against it (#1109).
    pub page_url: &'a str,
}

/// The frame-owner elements (`iframe`, `frame`) of one document, in document
/// order, including those inside author shadow roots. Does not enter the
/// frames' own documents.
fn frame_owners<'n>(node: &'n CdpNode, out: &mut Vec<&'n CdpNode>) {
    let name = if node.local_name.is_empty() {
        node.node_name.to_ascii_lowercase()
    } else {
        node.local_name.to_ascii_lowercase()
    };
    if node.node_type == 1 && (name == "iframe" || name == "frame") {
        out.push(node);
        return;
    }
    for shadow in node
        .shadow_roots
        .iter()
        .flatten()
        .filter(|s| s.shadow_root_type != Some(ShadowRootType::UserAgent))
    {
        frame_owners(shadow, out);
    }
    for child in node.children.iter().flatten() {
        frame_owners(child, out);
    }
}

fn attribute<'n>(node: &'n CdpNode, name: &str) -> Option<&'n str> {
    node.attributes
        .as_ref()?
        .as_chunks::<2>()
        .0
        .iter()
        .find(|pair| pair[0].eq_ignore_ascii_case(name))
        .map(|pair| pair[1].as_str())
}

/// Fallback when the in-page probe fails: `iframe#id`, else the tag alone.
fn attribute_selector(node: &CdpNode) -> String {
    let tag = node.local_name.to_ascii_lowercase();
    match attribute(node, "id")
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        Some(id) => format!("{tag}#{id}"),
        None => tag,
    }
}

/// What the iframe element itself says, read in its own document.
struct OwnerProbe {
    selector: String,
    hidden: bool,
}

/// The same "not perceivable" criteria as `iframe_rules`' scan.
const OWNER_PROBE_JS: &str = include_str!("js/frame_owner_probe.js");

async fn probe_owner(page: &Page, backend_id: i64) -> Option<OwnerProbe> {
    let resolved = page
        .execute(
            ResolveNodeParams::builder()
                .backend_node_id(BackendNodeId::new(backend_id))
                .build(),
        )
        .await
        .ok()?;
    let object_id = resolved.object.object_id.clone()?;
    let js = [
        "function() {",
        crate::accessibility::js_helpers::CSS_SELECTOR_JS,
        OWNER_PROBE_JS,
        "}",
    ]
    .concat();
    let call = CallFunctionOnParams::builder()
        .function_declaration(js)
        .object_id(object_id)
        .return_by_value(true)
        .build()
        .ok()?;
    let value = page.execute(call).await.ok()?.result.result.value.clone()?;
    Some(OwnerProbe {
        selector: value.get("selector")?.as_str()?.to_string(),
        hidden: value.get("hidden")?.as_bool()?,
    })
}

/// The source a frame is still waiting for: it has no `srcdoc` and either a
/// `src` that is not empty, `about:blank` or `javascript:`, or `loading="lazy"`
/// and a `data-*src*` attribute (`data-src` of lazysizes, `data-lazy-src`, ...)
/// that a lazy-load script copies to `src` once the frame is near the viewport.
/// Together with a document that is still `about:blank` this is a frame that
/// never loaded. A frame without a source (an empty `<iframe>` a script fills)
/// or with `src="about:blank"` is blank by design.
///
/// `src/wcag/rules/js/iframe_scan.js` applies the same test (`awaitsSource`).
fn awaited_source(owner: &CdpNode) -> Option<String> {
    if attribute(owner, "srcdoc").is_some() {
        return None;
    }
    let real = |value: &str| {
        let value = value.trim();
        let lower = value.to_ascii_lowercase();
        !(value.is_empty() || lower == "about:blank" || lower.starts_with("javascript:"))
    };
    if let Some(src) = attribute(owner, "src").filter(|v| real(v)) {
        return Some(src.trim().to_string());
    }
    let lazy = attribute(owner, "loading").is_some_and(|v| v.trim().eq_ignore_ascii_case("lazy"));
    if !lazy {
        return None;
    }
    owner
        .attributes
        .as_ref()?
        .as_chunks::<2>()
        .0
        .iter()
        .find(|pair| {
            let name = pair[0].to_ascii_lowercase();
            name.starts_with("data-") && name.contains("src") && real(&pair[1])
        })
        .map(|pair| pair[1].trim().to_string())
}

/// How long [`load_lazy_frames`] waits for the lazy frames of one page in
/// total, all frames together. Independent of the page-stability budget.
pub(crate) const LAZY_FRAME_BUDGET_MS: u64 = 3_000;

/// What [`load_lazy_frames`] found.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct LazyFrameLoad {
    /// Lazy frames still at `about:blank` that are rendered.
    pub candidates: u64,
    /// Of those, frames that left `about:blank` and fired `load`.
    pub loaded: u64,
    pub waited_ms: u64,
}

/// Makes the lazy-loaded iframes of the page load before the frame pass (#1109).
///
/// A `loading="lazy"` frame far down the page (or one a lazy-load script such
/// as lazysizes fills from `data-src`) sits at `about:blank` until it comes
/// near the viewport, and the audit then judged the empty placeholder — a
/// missing `lang` on a document nobody serves. The script scrolls each such
/// frame into view, waits for them to load together, at most
/// [`LAZY_FRAME_BUDGET_MS`], and restores the scroll position. It does nothing
/// on a page without a lazy blank frame.
///
/// It runs inside `run_rules`, after the snapshot (`extract_snapshot`) has read
/// every module including the performance observers, and before the DOM
/// capture the frame pass builds its documents from; the mobile pass
/// navigates anew, so a load started here reaches neither the performance
/// numbers of this pass nor of the other.
pub(crate) async fn load_lazy_frames(page: &Page) -> LazyFrameLoad {
    let expression = include_str!("js/lazy_frames.js")
        .replace("__BUDGET_MS__", &LAZY_FRAME_BUDGET_MS.to_string());
    let Ok(params) = EvaluateParams::builder()
        .expression(expression)
        .await_promise(true)
        .return_by_value(true)
        .build()
    else {
        return LazyFrameLoad::default();
    };
    let bound = std::time::Duration::from_millis(LAZY_FRAME_BUDGET_MS + 1_000);
    let value = match tokio::time::timeout(bound, page.execute(params)).await {
        Ok(Ok(result)) => result.result.result.value.clone().unwrap_or_default(),
        Ok(Err(e)) => {
            warn!("Loading the lazy frames failed: {e}");
            return LazyFrameLoad::default();
        }
        Err(_) => {
            warn!("Loading the lazy frames timed out");
            return LazyFrameLoad::default();
        }
    };
    let count = |key: &str| {
        value
            .get(key)
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0)
    };
    let load = LazyFrameLoad {
        candidates: count("candidates"),
        loaded: count("loaded"),
        waited_ms: count("waited_ms"),
    };
    if load.candidates > 0 {
        info!(
            "Lazy frames: {} of {} loaded after {} ms",
            load.loaded, load.candidates, load.waited_ms
        );
    }
    load
}

/// How long the frame pass waits for a frame whose DOM lags behind its
/// accessibility tree to catch up, in ms (#1180). Covers the pauses between
/// the re-captures and the re-captures themselves.
pub const FRAME_SETTLE_BUDGET_MS: u64 = 3_000;

/// The pause before each re-capture of a lagging frame, in ms.
const FRAME_SETTLE_STEP_MS: u64 = 250;

/// One reading of a frame: its AX tree and the document built against it.
/// After a re-capture it also holds the capture and the frame's content
/// document it was built from, which the frames nested in it are read from.
struct FrameRead {
    ax: AXTree,
    doc: CdpDocument,
    capture: Option<Arc<DomCapture>>,
    content: Option<CdpNode>,
}

impl FrameRead {
    /// Elements the AX tree has and the document lacks: content the frame
    /// rendered after the DOM was captured.
    fn lagging(&self) -> BTreeSet<i64> {
        self.doc.ax_elements_missing(&self.ax)
    }
}

/// Reads a frame again until its DOM has caught up with its AX tree, within
/// `budget` (#1180). A frame without a lagging element is returned as it is,
/// without a single further call. Otherwise `read` captures it anew after
/// each `step`; it is settled when nothing lags any more, or when the same
/// elements lag in two readings in a row (a mismatch that is no lag: content
/// the DOM capture never lists). It is a gap when the lagging elements keep
/// changing until the budget is spent, or when no reading could be taken.
async fn settle<S, Lag, Read, Fut>(
    first: S,
    lagging: Lag,
    mut read: Read,
    budget: Duration,
    step: Duration,
) -> (S, Option<FrameGap>)
where
    Lag: Fn(&S) -> BTreeSet<i64>,
    Read: FnMut() -> Fut,
    Fut: Future<Output = Option<S>>,
{
    let mut behind = lagging(&first);
    if behind.is_empty() {
        return (first, None);
    }
    let mut state = first;
    let started = Instant::now();
    while started.elapsed() < budget {
        tokio::time::sleep(step).await;
        let Some(next) = read().await else {
            return (state, Some(FrameGap::DomUnsettled));
        };
        let now = lagging(&next);
        state = next;
        if now.is_empty() || now == behind {
            return (state, None);
        }
        behind = now;
    }
    (state, Some(FrameGap::DomUnsettled))
}

/// The node with this backend id anywhere below `node`, frame documents and
/// shadow roots included.
fn find_by_backend(node: &CdpNode, id: i64) -> Option<&CdpNode> {
    if *node.backend_node_id.inner() == id {
        return Some(node);
    }
    node.children
        .iter()
        .flatten()
        .chain(node.shadow_roots.iter().flatten())
        .chain(node.content_document.as_deref())
        .find_map(|child| find_by_backend(child, id))
}

/// One fresh reading of the frame behind the owner element `backend`: DOM and
/// layout captured again, then the AX tree. `None` when any of it fails or
/// the owner is gone from the new capture.
async fn reread_frame(
    page: &Page,
    base: &DomCapture,
    backend: i64,
    frame_id: &str,
) -> Option<FrameRead> {
    let capture = match base.refreshed(page).await {
        Ok(capture) => Arc::new(capture),
        Err(e) => {
            warn!("Frame re-capture failed: {e}");
            return None;
        }
    };
    let content = find_by_backend(&capture.root, backend)?
        .content_document
        .as_deref()?
        .clone();
    let ax = crate::accessibility::extract_frame_ax_tree(page, frame_id)
        .await
        .ok()
        .filter(|ax| !ax.is_empty())?;
    let doc = capture.document_at(&content, &ax).ok()?;
    Some(FrameRead {
        ax,
        doc,
        capture: Some(capture),
        content: Some(content),
    })
}

/// A frame owner still to audit: the element, the tree it sits in, the
/// selector of its parent frame and the capture it was found in.
type FrameWork = (CdpNode, ParentTree, Option<String>, Option<Arc<DomCapture>>);

/// Which AX tree a frame owner belongs to: the page's, or an audited frame's.
#[derive(Clone, Copy)]
enum ParentTree {
    Main,
    Frame(usize),
}

/// Audits every reachable frame of the page (nested ones included) with the
/// frame rule set and returns the findings — already enriched, excluded and
/// tied to their frame — plus what was audited and skipped.
pub(crate) async fn audit_frames(
    page: &Page,
    capture: &DomCapture,
    main_ax: &AXTree,
    config: &FramePassConfig<'_>,
    exclusion: &ExclusionScope,
) -> FramePassOutput {
    let mut out = FramePassOutput {
        results: WcagResults::new(),
        excluded: Vec::new(),
        frames: Vec::new(),
    };
    let tree_filter = frame_rule_filter(config.rule_filter);

    let mut owners = Vec::new();
    frame_owners(&capture.root, &mut owners);
    // Owned, with the capture each was found in (`None`: the page's own): a
    // frame captured anew (#1180) hands its nested frames on from that capture.
    let mut work: Vec<FrameWork> = owners
        .into_iter()
        .map(|o| (o.clone(), ParentTree::Main, None, None))
        .collect();
    // AX trees of the audited frames, for the owners nested in them.
    let mut frame_trees: Vec<AXTree> = Vec::new();
    let mut next = 0;

    while next < work.len() {
        let (owner, parent, prefix, source) = &work[next];
        let parent = *parent;
        let base: &DomCapture = source.as_deref().unwrap_or(capture);
        next += 1;
        let backend = *owner.backend_node_id.inner();
        let probe = probe_owner(page, backend).await;
        let own_selector = probe
            .as_ref()
            .map(|p| p.selector.clone())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| attribute_selector(owner));
        let selector = match prefix {
            Some(prefix) => composite_selector(prefix, Some(&own_selector)),
            None => own_selector,
        };
        let content = owner.content_document.as_deref();
        let not_loaded = content
            .filter(|c| c.document_url.as_deref() == Some("about:blank"))
            .and_then(|_| awaited_source(owner));
        // A frame that never loaded shows the source it waits for, not its
        // blank placeholder.
        let url = not_loaded.clone().or_else(|| {
            content
                .and_then(|c| c.document_url.clone())
                .filter(|u| !u.is_empty())
                .or_else(|| attribute(owner, "src").map(str::to_string))
        });
        let mut observe = |skipped: Option<FrameSkipReason>| {
            out.frames.push(FrameObservation {
                selector: selector.clone(),
                url: url.clone(),
                skipped,
                gap: None,
                outcomes: Vec::new(),
            });
        };

        if exclusion.excludes_backend(backend) {
            observe(Some(FrameSkipReason::Excluded));
            continue;
        }
        let parent_ax = match parent {
            ParentTree::Main => main_ax,
            ParentTree::Frame(i) => &frame_trees[i],
        };
        let ignored_in_parent = parent_ax
            .node_by_backend_id(backend)
            .is_some_and(|n| n.ignored);
        if ignored_in_parent || probe.as_ref().is_some_and(|p| p.hidden) {
            observe(Some(FrameSkipReason::Hidden));
            continue;
        }
        let Some(content) = content else {
            observe(Some(FrameSkipReason::CrossOrigin));
            continue;
        };
        if not_loaded.is_some() {
            observe(Some(FrameSkipReason::NotLoaded));
            continue;
        }
        let Some(frame_id) = owner
            .frame_id
            .as_ref()
            .or(content.frame_id.as_ref())
            .map(|id| id.inner().clone())
        else {
            observe(Some(FrameSkipReason::AxTreeFailed));
            continue;
        };
        let frame_ax = match crate::accessibility::extract_frame_ax_tree(page, &frame_id).await {
            Ok(ax) if !ax.is_empty() => ax,
            Ok(_) => {
                warn!("Frame {selector}: empty AX tree");
                observe(Some(FrameSkipReason::AxTreeFailed));
                continue;
            }
            Err(e) => {
                warn!("Frame {selector}: AX tree unavailable: {e}");
                observe(Some(FrameSkipReason::AxTreeFailed));
                continue;
            }
        };
        let first = match base.document_at(content, &frame_ax) {
            Ok(doc) => FrameRead {
                ax: frame_ax,
                doc,
                capture: None,
                content: None,
            },
            Err(e) => {
                warn!("Frame {selector}: no document: {e}");
                observe(Some(FrameSkipReason::DocumentUnavailable));
                continue;
            }
        };
        // A frame that kept rendering after the DOM capture: its AX tree has
        // elements the document lacks. Capture it anew before any DOM rule
        // runs (#1180); no mismatch, no further call.
        let (read, gap) = settle(
            first,
            FrameRead::lagging,
            || reread_frame(page, base, backend, &frame_id),
            Duration::from_millis(FRAME_SETTLE_BUDGET_MS),
            Duration::from_millis(FRAME_SETTLE_STEP_MS),
        )
        .await;
        if gap.is_some() {
            warn!("Frame {selector}: DOM still behind its AX tree after re-capturing");
        }
        let FrameRead {
            ax: frame_ax,
            doc,
            capture: settled_capture,
            content: settled_content,
        } = read;
        let content = settled_content.as_ref().unwrap_or(content);
        observe(None);
        let observed = out.frames.len() - 1;
        out.frames[observed].gap = gap;

        let mut frame_results = WcagResults::new();
        if let Some(filter) = &tree_filter {
            let (tree, dropped) =
                wcag::check_all_excluding(&frame_ax, config.level, filter, &|v| {
                    exclusion.excludes_located(v, &frame_ax)
                });
            out.excluded.extend(dropped);
            frame_results.merge(tree);
        }
        let mut shared = wcag::shared::run_shared_rules(&doc, config.lang);
        retain_frame_shared_rules(&mut shared);
        wcag::shared::retain_up_to_level(&mut shared, config.level);
        wcag::shared::retain_allowed(&mut shared, config.rule_filter);
        for list in [&mut shared.violations, &mut shared.warnings] {
            let (dropped, kept): (Vec<_>, Vec<_>) = std::mem::take(list)
                .into_iter()
                .partition(|v| exclusion.excludes_located(v, &frame_ax));
            *list = kept;
            for v in &dropped {
                if let Some(outcome) = shared
                    .rule_outcomes
                    .iter_mut()
                    .find(|o| Some(o.rule_id.as_str()) == v.rule_id.as_deref())
                {
                    outcome.findings = outcome.findings.saturating_sub(1);
                }
            }
            out.excluded.extend(dropped);
        }
        frame_results.merge(shared);
        for outcome in &mut frame_results.rule_outcomes {
            outcome.viewport = Some(config.viewport.to_string());
        }

        crate::accessibility::enrich_violations_with_page(
            page,
            &mut frame_results.violations,
            &frame_ax,
        )
        .await;
        // The iframe's `src` first: a lazy-loaded frame is still `about:blank`
        // when it is audited (its `url` above), and only the attribute names
        // where it comes from.
        let origin = crate::model::frame_origin(
            config.page_url,
            &[attribute(owner, "src"), content.document_url.as_deref()],
        );
        for list in [
            &mut frame_results.violations,
            &mut frame_results.warnings,
            &mut frame_results.positives,
            &mut frame_results.not_testables,
        ] {
            for v in list.iter_mut() {
                attach_to_frame(v, &selector, &frame_id, &frame_ax, &origin);
            }
        }
        let found = frame_results.violations.len() + frame_results.warnings.len();
        if found > 0 {
            info!("Frame {selector}: {found} findings");
        }
        let outcomes = std::mem::take(&mut frame_results.rule_outcomes);
        out.frames[observed].outcomes = outcomes.clone();
        fold_outcomes(&mut out.results, outcomes);
        out.results.merge(frame_results);

        // Frames nested in this one, checked against this frame's AX tree.
        let mut nested = Vec::new();
        frame_owners(content, &mut nested);
        let nested_source = settled_capture.or_else(|| source.clone());
        let nested: Vec<_> = nested
            .into_iter()
            .map(|o| {
                (
                    o.clone(),
                    ParentTree::Frame(frame_trees.len()),
                    Some(selector.clone()),
                    nested_source.clone(),
                )
            })
            .collect();
        frame_trees.push(frame_ax);
        work.extend(nested);
    }
    out
}

/// Merges the frame pass into the page's results: findings by kind, rule
/// outcomes into the page's `(rule_id, viewport)` entries.
pub(crate) fn merge_into(page: &mut WcagResults, mut frames: WcagResults) {
    let outcomes = std::mem::take(&mut frames.rule_outcomes);
    fold_outcomes(page, outcomes);
    page.merge(frames);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::FindingOrigin;
    use crate::wcag::types::Severity;

    #[test]
    fn frame_filter_keeps_only_element_rules() {
        let filter = frame_rule_filter(&RuleFilterConfig::default()).unwrap();
        for page_level in ["focus-visible", "heading-order", "unusual-words", "help"] {
            assert!(!filter.should_run(page_level), "{page_level}");
        }
        assert!(filter.should_run("keyboard-trap"));
    }

    #[test]
    fn frame_filter_respects_user_filter_and_never_widens_to_all() {
        let user = RuleFilterConfig {
            disabled_rules: vec!["link-name".into()],
            enabled_only_rules: Vec::new(),
        };
        let filter = frame_rule_filter(&user).unwrap();
        assert!(!filter.should_run("link-name"));
        assert!(filter.should_run("keyboard-trap"));

        // Only page-level rules enabled: nothing to run in frames. An empty
        // `enabled_only_rules` would run every rule.
        let only_page_level = RuleFilterConfig {
            disabled_rules: Vec::new(),
            enabled_only_rules: vec!["focus-visible".into()],
        };
        assert!(frame_rule_filter(&only_page_level).is_none());
    }

    /// Every entry must be a registered tree rule, or the rule silently
    /// never runs in frames.
    #[test]
    fn frame_tree_rules_are_engine_gates() {
        for id in FRAME_TREE_RULES {
            assert!(
                crate::wcag::engine::TREE_RULES
                    .iter()
                    .any(|rule| rule.axe_id == *id),
                "{id} gates no tree rule"
            );
        }
    }

    #[test]
    fn frame_shared_rules_exist_and_exclude_page_level_ids() {
        for id in FRAME_SHARED_RULES {
            assert!(
                wcag::shared::SHARED_RULES.iter().any(|r| r.id == *id),
                "{id} is no shared rule"
            );
        }
        for page_level in [
            "document/lang-missing",
            "document/title-missing",
            "headings/h1-missing",
            "headings/none",
            "landmarks/main-missing",
            "landmarks/not-unique",
            "landmarks/not-top-level",
            "landmarks/banner-duplicate",
            "landmarks/contentinfo-duplicate",
            "landmarks/content-outside",
            "zoom/viewport-missing",
            "keyboard/skip-link-missing",
            "navigation/location-missing",
            "document/lang-mismatch",
            "language/part-unmarked",
            "timing/meta-refresh",
            "headings/section-without-heading",
        ] {
            assert!(!FRAME_SHARED_RULES.contains(&page_level), "{page_level}");
        }
    }

    fn finding(rule_id: &str) -> Violation {
        Violation::new("4.1.2", "x", WcagLevel::A, Severity::High, "msg", "7").with_rule_id(rule_id)
    }

    #[test]
    fn shared_results_are_narrowed_to_frame_rules() {
        let mut r = WcagResults::new();
        r.add_violation(finding("landmarks/main-missing"));
        r.add_violation(finding("lists/item-outside-list"));
        r.rule_outcomes
            .push(crate::wcag::RuleRun::ran("landmarks/main-missing", 1));
        r.rule_outcomes
            .push(crate::wcag::RuleRun::ran("lists/item-outside-list", 1));
        r.passes = 5;
        retain_frame_shared_rules(&mut r);
        assert_eq!(r.violations.len(), 1);
        assert_eq!(r.rule_outcomes.len(), 1);
        assert_eq!(r.passes, 0);
    }

    fn owner(attributes: &[(&str, &str)]) -> CdpNode {
        let flat: Vec<&str> = attributes.iter().flat_map(|(k, v)| [*k, *v]).collect();
        serde_json::from_value(serde_json::json!({
            "nodeId": 1,
            "backendNodeId": 2,
            "nodeType": 1,
            "nodeName": "IFRAME",
            "localName": "iframe",
            "nodeValue": "",
            "attributes": flat,
        }))
        .expect("a frame owner node")
    }

    /// A frame that shows `about:blank` is "not loaded" only when it has a
    /// source it is waiting for (#1109); blank by design stays audited.
    #[test]
    fn a_blank_frame_awaits_a_source_only_when_it_has_one() {
        let url = "https://widgets.example/chart?lang=en";
        // A source the frame never got to.
        assert_eq!(
            awaited_source(&owner(&[("src", url), ("loading", "lazy")])),
            Some(url.to_string())
        );
        assert_eq!(
            awaited_source(&owner(&[("src", &format!("  {url} "))])),
            Some(url.to_string())
        );
        // lazysizes and similar: `src` is filled from a data attribute.
        assert_eq!(
            awaited_source(&owner(&[("loading", "LAZY"), ("data-src", url)])),
            Some(url.to_string())
        );
        assert_eq!(
            awaited_source(&owner(&[("loading", "lazy"), ("data-lazy-src", url)])),
            Some(url.to_string())
        );
        // Blank by design: nothing to wait for.
        for blank in [
            owner(&[]),
            owner(&[("src", "")]),
            owner(&[("src", "about:blank")]),
            owner(&[("src", " ABOUT:BLANK ")]),
            owner(&[("src", "javascript:void(0)")]),
            owner(&[("srcdoc", "<p>hi</p>"), ("src", url)]),
            owner(&[("srcdoc", ""), ("loading", "lazy"), ("data-src", url)]),
            // A data attribute only counts for a lazy frame ...
            owner(&[("data-src", url)]),
            // ... and only when it carries a source.
            owner(&[("loading", "lazy"), ("data-src", "")]),
            owner(&[("loading", "lazy"), ("data-title", url)]),
        ] {
            assert_eq!(awaited_source(&blank), None, "{:?}", blank.attributes);
        }
    }

    #[test]
    fn the_not_loaded_reason_is_canonical_english_snake_case() {
        assert_eq!(
            serde_json::to_string(&FrameSkipReason::NotLoaded).unwrap(),
            "\"not_loaded\""
        );
    }

    #[test]
    fn composite_selectors() {
        assert_eq!(
            composite_selector("iframe#w", Some("div#dlg")),
            "iframe#w [frame] div#dlg"
        );
        assert_eq!(composite_selector("iframe#w", None), "iframe#w [frame]");
        assert_eq!(
            composite_selector("iframe#w", Some("  ")),
            "iframe#w [frame]"
        );
    }

    #[test]
    fn attached_finding_points_into_the_frame() {
        let mut v = finding("dialog-name").with_selector("div#dlg");
        assert_eq!(v.origin, FindingOrigin::FirstParty);
        attach_to_frame(
            &mut v,
            "iframe#w",
            "F1",
            &AXTree::new(),
            &FrameOrigin::of(
                FindingOrigin::CrossOriginFrame,
                Some("YouTube"),
                Some("www.youtube.com"),
            ),
        );
        assert_eq!(v.selector.as_deref(), Some("iframe#w [frame] div#dlg"));
        assert_eq!(v.node_id, "frame:F1:7");
        assert!(v.tags.iter().any(|t| t == "iframe-content"));
        assert_eq!(v.origin, FindingOrigin::CrossOriginFrame);
        assert_eq!(v.provider.as_deref(), Some("YouTube"));
        assert_eq!(v.frame_host.as_deref(), Some("www.youtube.com"));
    }

    #[test]
    fn frame_outcomes_add_to_the_page_outcome() {
        let mut page = WcagResults::new();
        page.rule_outcomes
            .push(crate::wcag::RuleRun::ran("dialog-name", 1).in_viewport("desktop"));
        let mut frame = WcagResults::new();
        frame
            .rule_outcomes
            .push(crate::wcag::RuleRun::ran("dialog-name", 2).in_viewport("desktop"));
        frame.add_violation(finding("dialog-name"));
        merge_into(&mut page, frame);
        assert_eq!(page.rule_outcomes.len(), 1);
        assert_eq!(page.rule_outcomes[0].findings, 3);
        assert_eq!(page.violations.len(), 1);
    }

    #[test]
    fn expected_frame_rules_follow_level_and_filter() {
        let all = RuleFilterConfig::default();
        let at_aa = expected_frame_rules(WcagLevel::AA, &all);
        for id in FRAME_TREE_RULES {
            assert!(at_aa.iter().any(|r| r == id), "{id} expected at AA");
        }
        for id in FRAME_SHARED_RULES {
            assert!(at_aa.iter().any(|r| r == id), "{id} expected at AA");
        }
        // A tree rule above the level does not run; the shared rules still
        // report, as not run.
        let at_a = expected_frame_rules(WcagLevel::A, &all);
        let above = FRAME_TREE_RULES.iter().find(|id| {
            crate::wcag::expectation::registered_min_level(id).is_some_and(|min| min > WcagLevel::A)
        });
        if let Some(id) = above {
            assert!(!at_a.iter().any(|r| r == id), "{id} runs above level A");
        }
        assert_eq!(
            at_a.iter()
                .filter(|r| FRAME_SHARED_RULES.contains(&r.as_str()))
                .count(),
            FRAME_SHARED_RULES.len()
        );
        // The user's filter removes tree rules from the frame scope, and
        // `enabled_only` of a page-level rule leaves only the shared ones.
        let disabled = RuleFilterConfig {
            disabled_rules: vec!["keyboard-trap".into()],
            enabled_only_rules: Vec::new(),
        };
        assert!(!expected_frame_rules(WcagLevel::AA, &disabled)
            .iter()
            .any(|r| r == "keyboard-trap"));
        let page_level_only = RuleFilterConfig {
            disabled_rules: Vec::new(),
            enabled_only_rules: vec!["focus-visible".into()],
        };
        let rules = expected_frame_rules(WcagLevel::AA, &page_level_only);
        assert_eq!(rules.len(), FRAME_SHARED_RULES.len());
    }

    #[test]
    fn coverage_keeps_one_outcome_per_rule_and_viewport_of_a_frame() {
        let obs = |viewport: &str, findings: usize| FrameObservation {
            selector: "iframe#w".into(),
            url: None,
            skipped: None,
            gap: None,
            outcomes: vec![RuleRun::ran("keyboard-trap", findings).in_viewport(viewport)],
        };
        let mut c = FrameCoverage::default();
        c.record("desktop", &[obs("desktop", 1)]);
        c.record("mobile", &[obs("mobile", 2)]);
        // The same selector twice in one pass is one entry; its outcome is
        // folded, not doubled.
        c.record("mobile", &[obs("mobile", 3)]);
        assert_eq!(c.audited.len(), 1);
        let outcomes = &c.audited[0].outcomes;
        assert_eq!(outcomes.len(), 2);
        assert_eq!(
            outcomes
                .iter()
                .find(|o| o.viewport.as_deref() == Some("mobile"))
                .unwrap()
                .findings,
            5
        );
        let json = serde_json::to_value(&c).unwrap();
        assert_eq!(
            json["audited"][0]["outcomes"][0]["rule_id"],
            "keyboard-trap"
        );
    }

    #[test]
    fn coverage_joins_passes_per_frame() {
        let obs = |skipped| FrameObservation {
            selector: "iframe#w".into(),
            url: Some("https://example.org/w".into()),
            skipped,
            gap: None,
            outcomes: Vec::new(),
        };
        let mut c = FrameCoverage::default();
        c.record(
            "desktop",
            &[
                obs(None),
                FrameObservation {
                    selector: "iframe#ad".into(),
                    url: None,
                    skipped: Some(FrameSkipReason::CrossOrigin),
                    gap: None,
                    outcomes: Vec::new(),
                },
            ],
        );
        c.record("mobile", &[obs(None)]);
        assert_eq!(c.audited.len(), 1);
        assert_eq!(c.audited[0].viewports, ["desktop", "mobile"]);
        assert_eq!(c.skipped.len(), 1);
        let json = serde_json::to_value(&c).unwrap();
        assert_eq!(json["skipped"][0]["reason"], "cross_origin");
        assert!(json["audited"][0].get("reason").is_none());
    }

    #[test]
    fn coverage_keeps_a_gapped_frame_apart_from_a_settled_one() {
        let obs = |gap| FrameObservation {
            selector: "iframe#w".into(),
            url: None,
            skipped: None,
            gap,
            outcomes: Vec::new(),
        };
        let mut c = FrameCoverage::default();
        c.record("desktop", &[obs(Some(FrameGap::DomUnsettled))]);
        c.record("mobile", &[obs(None)]);
        assert_eq!(c.audited.len(), 2);
        let json = serde_json::to_value(&c).unwrap();
        assert_eq!(json["audited"][0]["gap"], "dom_unsettled");
        assert!(json["audited"][1].get("gap").is_none());
    }

    /// A reading is the set of lagging ids it carries.
    type Reading = BTreeSet<i64>;

    async fn settled(
        first: &[i64],
        later: Vec<Option<Vec<i64>>>,
        budget_ms: u64,
    ) -> (Reading, Option<FrameGap>, usize) {
        let mut later = later.into_iter();
        let mut calls = 0;
        let (state, gap) = settle(
            first.iter().copied().collect::<Reading>(),
            |r: &Reading| r.clone(),
            || {
                calls += 1;
                let next = later.next().flatten();
                async move { next.map(|ids| ids.into_iter().collect::<Reading>()) }
            },
            Duration::from_millis(budget_ms),
            Duration::from_millis(1),
        )
        .await;
        (state, gap, calls)
    }

    #[tokio::test]
    async fn a_frame_without_a_lagging_element_is_not_read_again() {
        let (_, gap, calls) = settled(&[], vec![Some(vec![9])], 100).await;
        assert_eq!((gap, calls), (None, 0));
    }

    #[tokio::test]
    async fn a_frame_that_catches_up_is_settled_on_the_new_reading() {
        let (state, gap, calls) = settled(&[1, 2], vec![Some(vec![])], 100).await;
        assert_eq!((gap, calls), (None, 1));
        assert!(state.is_empty());
    }

    #[tokio::test]
    async fn the_same_elements_lagging_twice_are_a_mismatch_not_a_lag() {
        let (_, gap, calls) = settled(&[1, 2], vec![Some(vec![1, 2])], 100).await;
        assert_eq!((gap, calls), (None, 1));
    }

    #[tokio::test]
    async fn a_frame_that_keeps_growing_is_a_gap_when_the_budget_is_spent() {
        let growing = (0..200).map(|n| Some((1..=n + 2).collect())).collect();
        let (state, gap, calls) = settled(&[1], growing, 30).await;
        assert_eq!(gap, Some(FrameGap::DomUnsettled));
        assert!(calls >= 1);
        // The last reading stands: the findings are of the newest document.
        assert!(state.len() >= 2);
    }

    #[tokio::test]
    async fn a_failed_recapture_is_a_gap_and_keeps_the_reading_in_hand() {
        let (state, gap, calls) = settled(&[1], vec![None], 100).await;
        assert_eq!((gap, calls), (Some(FrameGap::DomUnsettled), 1));
        assert_eq!(state.into_iter().collect::<Vec<_>>(), [1]);
    }

    #[test]
    fn a_node_below_a_frame_document_or_shadow_root_is_found_by_backend_id() {
        let node = |id: i64, children: Vec<serde_json::Value>, extra: serde_json::Value| {
            let mut v = serde_json::json!({
                "nodeId": id, "backendNodeId": id, "nodeType": 1,
                "nodeName": "DIV", "localName": "div", "nodeValue": "",
                "children": children,
            });
            v.as_object_mut()
                .unwrap()
                .extend(extra.as_object().cloned().unwrap_or_default());
            v
        };
        let inner = node(
            3,
            vec![node(4, vec![], serde_json::json!({}))],
            serde_json::json!({}),
        );
        let owner = node(2, vec![], serde_json::json!({ "contentDocument": inner }));
        let root: CdpNode = serde_json::from_value(node(1, vec![owner], serde_json::json!({})))
            .expect("a CDP node");
        assert_eq!(
            find_by_backend(&root, 4).map(|n| *n.backend_node_id.inner()),
            Some(4)
        );
        assert!(find_by_backend(&root, 99).is_none());
    }
}
