//! WCAG rules inside iframes (#715).
//!
//! `getFullAXTree` and the shared DOM rules see the main document only, so a
//! `role="dialog"` without a name or a broken tablist inside an iframe went
//! unreported while axe-core, which runs in every frame, found it. This pass
//! audits every frame rendered in the page's own process: it fetches the
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

use chromiumoxide::cdp::browser_protocol::dom::{
    BackendNodeId, Node as CdpNode, ResolveNodeParams, ShadowRootType,
};
use chromiumoxide::cdp::js_protocol::runtime::CallFunctionOnParams;
use chromiumoxide::Page;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::accessibility::{AXTree, DomCapture};
use crate::audit::exclusion::ExclusionScope;
use crate::cli::WcagLevel;
use crate::wcag::{self, RuleFilterConfig, Violation, WcagResults};

/// Tree rules that run inside frames, by the axe id that gates them in
/// `wcag::engine`. Only rules that judge single elements or their
/// relationships: names (images, links, buttons, form fields, dialogs,
/// commands, summaries), roles and their required parents/children, form
/// structure, table headers, keyboard reachability, live-region roles and
/// ambiguous `aria-owns`.
///
/// Left out on purpose — they judge the document as a whole and belong to
/// the top frame only: `bypass`, `region`, the `landmark-*` rules,
/// `focus-visible` (fires only when the whole page has nothing focusable),
/// `heading-order`, `unusual-words` and `help`.
pub const FRAME_TREE_RULES: &[&str] = &[
    "image-alt",
    "keyboard",
    "link-name",
    "label",
    "aria-invalid-without-describedby",
    "aria-roles",
    "aria-required-parent",
    "aria-label",
    "aria-command-name",
    "form-field-group",
    "dialog-name",
    "aria-required-children",
    "video-caption",
    "th-has-data-cells",
    "summary-name",
    "label-title-only",
    "duplicate-id-aria",
    "aria-live-region-role",
];

/// Shared rules (`wcag::shared::SHARED_RULES`) that run inside frames — the
/// element-level ones. Left out, as page-level: `document/lang-*` (the
/// frame's `lang` stays with `iframe_rules`' `html-has-lang`),
/// `document/title-*`, `headings/h1-*`, `headings/skip-level`, `zoom/*`,
/// `landmarks/*` and `keyboard/skip-link-missing`.
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
];

// ── Report block (canonical English) ─────────────────────────────────────────

/// Frame coverage of one page: which iframes were audited, which skipped.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FrameCoverage {
    #[serde(default)]
    pub audited: Vec<FrameEntry>,
    #[serde(default)]
    pub skipped: Vec<FrameEntry>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
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
    /// The viewport passes this entry applies to.
    #[serde(default)]
    pub viewports: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FrameSkipReason {
    /// Rendered out of process (site isolation): not reachable through the
    /// page session. `frame-tested` asks for a manual review of it.
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
}

/// What one viewport pass saw of one frame.
#[derive(Debug, Clone, PartialEq)]
pub struct FrameObservation {
    pub selector: String,
    pub url: Option<String>,
    /// `None` = audited.
    pub skipped: Option<FrameSkipReason>,
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
            match list
                .iter_mut()
                .find(|e| e.selector == obs.selector && e.url == obs.url && e.reason == obs.skipped)
            {
                Some(entry) => {
                    if !entry.viewports.iter().any(|v| v == viewport) {
                        entry.viewports.push(viewport.to_string());
                    }
                }
                None => list.push(FrameEntry {
                    selector: obs.selector.clone(),
                    url: obs.url.clone(),
                    reason: obs.skipped,
                    viewports: vec![viewport.to_string()],
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
/// from the frame's AX tree, `frame:`-prefixed node id, `iframe-content` tag.
fn attach_to_frame(v: &mut Violation, frame_selector: &str, frame_id: &str, ax: &AXTree) {
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
fn fold_outcomes(page: &mut WcagResults, frame: Vec<crate::wcag::RuleRun>) {
    for run in frame {
        match page
            .rule_outcomes
            .iter_mut()
            .find(|o| o.rule_id == run.rule_id && o.viewport == run.viewport)
        {
            Some(existing) => {
                if existing.not_run.is_none() && run.not_run.is_none() {
                    existing.findings += run.findings;
                }
            }
            None => page.rule_outcomes.push(run),
        }
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
const OWNER_PROBE_JS: &str = r#"
  var el = this;
  var role = (el.getAttribute('role') || '').toLowerCase();
  var hidden = role === 'none' || role === 'presentation'
    || el.hasAttribute('hidden') || el.getAttribute('aria-hidden') === 'true';
  var st = window.getComputedStyle(el);
  if (st && (st.display === 'none' || st.visibility === 'hidden' || st.visibility === 'collapse')) hidden = true;
  var r = el.getBoundingClientRect();
  if (r.width <= 1 || r.height <= 1) hidden = true;
  return { selector: __amsCssSelector(el), hidden: hidden };
"#;

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
    let mut work: Vec<(&CdpNode, ParentTree, Option<String>)> = owners
        .into_iter()
        .map(|o| (o, ParentTree::Main, None))
        .collect();
    // AX trees of the audited frames, for the owners nested in them.
    let mut frame_trees: Vec<AXTree> = Vec::new();
    let mut next = 0;

    while next < work.len() {
        let (owner, parent, prefix) = work[next].clone();
        next += 1;
        let backend = *owner.backend_node_id.inner();
        let probe = probe_owner(page, backend).await;
        let own_selector = probe
            .as_ref()
            .map(|p| p.selector.clone())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| attribute_selector(owner));
        let selector = match &prefix {
            Some(prefix) => composite_selector(prefix, Some(&own_selector)),
            None => own_selector,
        };
        let content = owner.content_document.as_deref();
        let url = content
            .and_then(|c| c.document_url.clone())
            .filter(|u| !u.is_empty())
            .or_else(|| attribute(owner, "src").map(str::to_string));
        let mut observe = |skipped: Option<FrameSkipReason>| {
            out.frames.push(FrameObservation {
                selector: selector.clone(),
                url: url.clone(),
                skipped,
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
        let doc = match capture.document_at(content, &frame_ax) {
            Ok(doc) => doc,
            Err(e) => {
                warn!("Frame {selector}: no document: {e}");
                observe(Some(FrameSkipReason::DocumentUnavailable));
                continue;
            }
        };
        observe(None);

        let svg_nodes = wcag::shared::svg_ax_node_ids(&doc);
        let superseded = |v: &Violation| wcag::rules::is_svg_finding(v, &svg_nodes);
        let mut frame_results = WcagResults::new();
        if let Some(filter) = &tree_filter {
            let (tree, dropped) =
                wcag::check_all_excluding(&frame_ax, config.level, filter, &|v| {
                    superseded(v) || exclusion.excludes_located(v, &frame_ax)
                });
            out.excluded
                .extend(dropped.into_iter().filter(|v| !superseded(v)));
            frame_results.merge(tree);
        }
        let mut shared = wcag::shared::run_shared_rules(&doc, config.lang);
        retain_frame_shared_rules(&mut shared);
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
        for list in [
            &mut frame_results.violations,
            &mut frame_results.warnings,
            &mut frame_results.positives,
            &mut frame_results.not_testables,
        ] {
            for v in list.iter_mut() {
                attach_to_frame(v, &selector, &frame_id, &frame_ax);
            }
        }
        let found = frame_results.violations.len() + frame_results.warnings.len();
        if found > 0 {
            info!("Frame {selector}: {found} findings");
        }
        let outcomes = std::mem::take(&mut frame_results.rule_outcomes);
        fold_outcomes(&mut out.results, outcomes);
        out.results.merge(frame_results);

        // Frames nested in this one, checked against this frame's AX tree.
        let mut nested = Vec::new();
        frame_owners(content, &mut nested);
        frame_trees.push(frame_ax);
        let index = frame_trees.len() - 1;
        work.extend(
            nested
                .into_iter()
                .map(|o| (o, ParentTree::Frame(index), Some(selector.clone()))),
        );
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
    use crate::wcag::types::Severity;

    #[test]
    fn frame_filter_keeps_only_element_rules() {
        let filter = frame_rule_filter(&RuleFilterConfig::default()).unwrap();
        for page_level in [
            "bypass",
            "region",
            "landmark-unique",
            "focus-visible",
            "heading-order",
        ] {
            assert!(!filter.should_run(page_level), "{page_level}");
        }
        for element in [
            "dialog-name",
            "aria-required-children",
            "aria-required-parent",
            "aria-roles",
            "image-alt",
        ] {
            assert!(filter.should_run(element), "{element}");
        }
    }

    #[test]
    fn frame_filter_respects_user_filter_and_never_widens_to_all() {
        let user = RuleFilterConfig {
            disabled_rules: vec!["dialog-name".into()],
            enabled_only_rules: Vec::new(),
        };
        let filter = frame_rule_filter(&user).unwrap();
        assert!(!filter.should_run("dialog-name"));
        assert!(filter.should_run("aria-roles"));

        // Only page-level rules enabled: nothing to run in frames. An empty
        // `enabled_only_rules` would run every rule.
        let only_page_level = RuleFilterConfig {
            disabled_rules: Vec::new(),
            enabled_only_rules: vec!["bypass".into()],
        };
        assert!(frame_rule_filter(&only_page_level).is_none());
    }

    /// Every entry must be an id that gates a tree rule in `wcag::engine`,
    /// or the rule silently never runs in frames.
    #[test]
    fn frame_tree_rules_are_engine_gates() {
        let engine = include_str!("../wcag/engine.rs");
        for id in FRAME_TREE_RULES {
            assert!(
                engine.contains(&format!("filter, \"{id}\""))
                    || engine.contains(&format!("filter,\n        \"{id}\"")),
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
            "landmarks/main-missing",
            "zoom/viewport-missing",
            "keyboard/skip-link-missing",
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
        attach_to_frame(&mut v, "iframe#w", "F1", &AXTree::new());
        assert_eq!(v.selector.as_deref(), Some("iframe#w [frame] div#dlg"));
        assert_eq!(v.node_id, "frame:F1:7");
        assert!(v.tags.iter().any(|t| t == "iframe-content"));
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
    fn coverage_joins_passes_per_frame() {
        let obs = |skipped| FrameObservation {
            selector: "iframe#w".into(),
            url: Some("https://example.org/w".into()),
            skipped,
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
}
