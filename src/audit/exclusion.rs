//! Excluding intentional specimens from an audit (#645).
//!
//! Sites that teach accessibility ship markup that is broken on purpose. Like
//! axe-core's `exclude`, an exclusion never removes a page or a rule: the page
//! is audited as usual, and only findings whose element lies inside an
//! excluded subtree are dropped before scoring. Every exclusion is reported —
//! which selectors were applied, how many elements each matched (also when
//! that is zero), and how many findings were dropped — so excluding never
//! silently hides a finding.
//!
//! Locating a finding's element works on two paths:
//! - by backend DOM node id — AX-tree rules (via the AX node) and the shared
//!   DOM rules (which carry the id directly);
//! - by selector — JavaScript page rules, contrast and journey findings only
//!   know a CSS path. Such a finding is excluded only if its selector matches
//!   at least one element and **every** match lies inside an excluded
//!   subtree; an ambiguous or unparseable selector keeps the finding.
//!
//! Findings without either (page-level findings) are never excluded.
//!
//! The screen-reader layer (#703) reports AX node ids; they are located by
//! backend DOM node id like the AX-tree rules (see
//! [`ExclusionScope::excludes_ax_node`]).

use std::collections::{BTreeMap, HashMap, HashSet};

use chromiumoxide::cdp::browser_protocol::dom::{
    GetDocumentParams, Node as CdpNode, NodeId, QuerySelectorAllParams,
};
use chromiumoxide::Page;
use fluent_bundle::FluentValue;
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::accessibility::{AXNode, AXTree};
use crate::audit::interactive_finding::InteractiveFinding;
use crate::wcag::types::{Outcome, Violation};

/// Always honoured, without configuration: marks a whole subtree as excluded.
pub const BUILTIN_EXCLUDE_SELECTOR: &str = "[data-audit-exclude]";

/// The effective selector list: the built-in attribute first, then the
/// user-supplied selectors (trimmed, empty and duplicate entries dropped).
pub fn effective_selectors(user: &[String]) -> Vec<String> {
    let mut out = vec![BUILTIN_EXCLUDE_SELECTOR.to_string()];
    for selector in user.iter().map(|s| s.trim()).filter(|s| !s.is_empty()) {
        if !out.iter().any(|existing| existing == selector) {
            out.push(selector.to_string());
        }
    }
    out
}

// ── Report block (canonical, language-neutral numbers) ───────────────────────

/// Per page: which exclusions were applied and what they removed.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ExclusionReport {
    /// Every applied selector, including those that matched nothing.
    pub selectors: Vec<ExclusionSelectorResult>,
    /// Excluded WCAG finding occurrences (all outcomes), summed over
    /// `rules`. Per rule, the larger count of the two viewport passes.
    pub excluded_occurrences: usize,
    /// Of `excluded_occurrences`, confirmed violations (outcome `fail`).
    pub excluded_violations: usize,
    /// Excluded occurrences per rule, sorted by rule id.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rules: Vec<ExcludedRuleCount>,
    /// Excluded Accessibility-Journey findings.
    #[serde(default)]
    pub excluded_interactive_findings: usize,
    /// Screen-reader issues dropped because every node they name lies in an
    /// excluded subtree (#703). An issue naming excluded and other nodes
    /// stays, with only the other nodes.
    #[serde(default)]
    pub excluded_screen_reader_issues: usize,
    /// Landmarks inside excluded subtrees, left out of the page-level
    /// landmark counts (duplicate banner/contentinfo, unique names, #726).
    /// The larger count of the two viewport passes.
    #[serde(default)]
    pub excluded_landmarks: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ExclusionSelectorResult {
    pub selector: String,
    /// `true` for the always-honoured `[data-audit-exclude]`.
    pub builtin: bool,
    /// Elements the selector matched (the roots of excluded subtrees); the
    /// larger count of the two viewport passes.
    pub matched_elements: usize,
    /// The browser rejected the selector; it excluded nothing.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub invalid: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ExcludedRuleCount {
    /// Rule id as on the findings (`rule_id`, falling back to the criterion).
    pub rule_id: String,
    /// WCAG criterion of the rule (e.g. `"1.1.1"`).
    pub wcag: String,
    pub occurrences: usize,
    pub violations: usize,
}

impl ExclusionReport {
    /// Worth a note in the PDF: a user-supplied selector was applied, or the
    /// built-in attribute matched something. A default run on a page without
    /// `data-audit-exclude` stays quiet there (the JSON still lists it).
    pub fn is_noteworthy(&self) -> bool {
        self.selectors
            .iter()
            .any(|s| !s.builtin || s.matched_elements > 0)
    }
}

// ── Per-pass scope ────────────────────────────────────────────────────────────

/// What one selector matched in one viewport pass.
#[derive(Debug, Clone, PartialEq)]
pub struct SelectorMatch {
    pub selector: String,
    pub builtin: bool,
    pub matched_elements: usize,
    pub invalid: bool,
}

/// The excluded subtrees of the currently loaded page (one viewport pass).
#[derive(Debug, Clone, Default)]
pub struct ExclusionScope {
    pub matches: Vec<SelectorMatch>,
    /// Backend node ids of every node inside an excluded subtree.
    backend_ids: HashSet<i64>,
}

impl ExclusionScope {
    /// Something to exclude: at least one selector matched.
    pub fn is_active(&self) -> bool {
        self.matches.iter().any(|m| m.matched_elements > 0)
    }

    fn contains(&self, backend_id: i64) -> bool {
        self.backend_ids.contains(&backend_id)
    }

    /// Selectors that matched something — used for the in-page selector check.
    fn active_selectors(&self) -> Vec<&str> {
        self.matches
            .iter()
            .filter(|m| m.matched_elements > 0 && !m.invalid)
            .map(|m| m.selector.as_str())
            .collect()
    }

    #[cfg(test)]
    fn with_backend_ids(matches: Vec<SelectorMatch>, ids: impl IntoIterator<Item = i64>) -> Self {
        Self {
            matches,
            backend_ids: ids.into_iter().collect(),
        }
    }
}

/// Resolve the excluded subtrees on the loaded page.
///
/// One `Runtime.evaluate` counts every selector's matches (and detects
/// invalid selectors). Only if something matched does it fetch the DOM once
/// and collect the backend ids of every matched subtree (children, shadow
/// roots, frame and template content included).
pub async fn resolve_scope(page: &Page, selectors: &[String]) -> ExclusionScope {
    let counts = count_matches(page, selectors).await;
    let mut matches: Vec<SelectorMatch> = selectors
        .iter()
        .zip(counts)
        .map(|(selector, count)| SelectorMatch {
            selector: selector.clone(),
            builtin: selector == BUILTIN_EXCLUDE_SELECTOR,
            matched_elements: count.unwrap_or(0),
            invalid: count.is_none(),
        })
        .collect();

    let mut scope = ExclusionScope::default();
    if !matches.iter().any(|m| m.matched_elements > 0) {
        scope.matches = matches;
        return scope;
    }

    let root = match page
        .execute(GetDocumentParams {
            depth: Some(-1),
            pierce: Some(true),
        })
        .await
    {
        Ok(response) => response.result.root.clone(),
        Err(e) => {
            // Without the DOM no subtree can be resolved; report the
            // selectors as matching nothing rather than claiming exclusions.
            warn!("Exclusion: DOM.getDocument failed: {e}");
            for m in &mut matches {
                m.matched_elements = 0;
            }
            scope.matches = matches;
            return scope;
        }
    };
    let mut by_node_id: HashMap<NodeId, &CdpNode> = HashMap::new();
    index_nodes(&root, &mut by_node_id);

    for m in &mut matches {
        if m.matched_elements == 0 {
            continue;
        }
        let params = QuerySelectorAllParams::new(root.node_id, m.selector.clone());
        match page.execute(params).await {
            Ok(response) => {
                let node_ids = &response.result.node_ids;
                m.matched_elements = node_ids.len();
                for id in node_ids {
                    if let Some(node) = by_node_id.get(id) {
                        collect_subtree(node, &mut scope.backend_ids);
                    }
                }
            }
            Err(e) => {
                warn!(
                    "Exclusion: DOM.querySelectorAll failed for {}: {e}",
                    m.selector
                );
                m.matched_elements = 0;
            }
        }
    }
    scope.matches = matches;
    install_in_page_marker(page, &scope).await;
    scope
}

/// Expose the excluded subtree roots to the page's own rule scripts as
/// `window.__amsIsExcluded(el)`, so capped JavaScript rules do not spend
/// their cap on excluded elements (see `CSS_SELECTOR_JS`). The roots live in
/// a closure-held `WeakSet` — no DOM attribute is written, so no other rule
/// sees a changed document. The marker lives until the next navigation; each
/// viewport pass reloads the page and installs it afresh.
async fn install_in_page_marker(page: &Page, scope: &ExclusionScope) {
    let active = scope.active_selectors();
    if active.is_empty() {
        return;
    }
    let js = format!(
        r#"(function(excl) {{
  var roots = new WeakSet();
  excl.forEach(function(s) {{
    try {{ document.querySelectorAll(s).forEach(function(el) {{ roots.add(el); }}); }} catch (e) {{}}
  }});
  var isExcluded = function(el) {{
    var cur = el;
    while (cur) {{
      if (roots.has(cur)) return true;
      if (cur.parentElement) {{ cur = cur.parentElement; continue; }}
      var root = cur.getRootNode ? cur.getRootNode() : null;
      cur = (root && root.host) ? root.host : null;
    }}
    return false;
  }};
  Object.defineProperty(window, '__amsIsExcluded', {{
    value: isExcluded, configurable: true, writable: true, enumerable: false
  }});
  return true;
}})({})"#,
        serde_json::to_string(&active).unwrap_or_else(|_| "[]".to_string())
    );
    if let Err(e) = page.evaluate(js).await {
        warn!("Exclusion: installing the in-page marker failed: {e}");
    }
}

/// Match count per selector; `None` for a selector the browser rejects.
async fn count_matches(page: &Page, selectors: &[String]) -> Vec<Option<usize>> {
    let js = format!(
        r#"(function(sels) {{
  return sels.map(function(s) {{
    try {{ return document.querySelectorAll(s).length; }} catch (e) {{ return -1; }}
  }});
}})({})"#,
        serde_json::to_string(selectors).unwrap_or_else(|_| "[]".to_string())
    );
    let values: Vec<i64> = match page.evaluate(js).await {
        Ok(result) => result.into_value().unwrap_or_default(),
        Err(e) => {
            warn!("Exclusion: selector count failed: {e}");
            Vec::new()
        }
    };
    selectors
        .iter()
        .enumerate()
        .map(|(i, _)| match values.get(i) {
            Some(n) if *n >= 0 => Some(*n as usize),
            Some(_) => None,
            // Evaluation failed as a whole: nothing matched, nothing excluded.
            None => Some(0),
        })
        .collect()
}

fn child_nodes(node: &CdpNode) -> impl Iterator<Item = &CdpNode> {
    node.children
        .iter()
        .flatten()
        .chain(node.shadow_roots.iter().flatten())
        .chain(node.pseudo_elements.iter().flatten())
        .chain(node.content_document.as_deref())
        .chain(node.template_content.as_deref())
}

fn index_nodes<'a>(node: &'a CdpNode, map: &mut HashMap<NodeId, &'a CdpNode>) {
    map.insert(node.node_id, node);
    for child in child_nodes(node) {
        index_nodes(child, map);
    }
}

fn collect_subtree(node: &CdpNode, out: &mut HashSet<i64>) {
    out.insert(*node.backend_node_id.inner());
    for child in child_nodes(node) {
        collect_subtree(child, out);
    }
}

// ── Filtering ─────────────────────────────────────────────────────────────────

/// How a finding's element can be located.
#[derive(Debug, Clone, PartialEq)]
enum Locator<'a> {
    Backend(i64),
    Selector(&'a str),
    /// The rule's own script found the element inside an excluded subtree.
    MarkedInPage,
    PageLevel,
}

fn locate<'a>(finding: &'a Violation, ax_tree: &AXTree) -> Locator<'a> {
    if finding.in_excluded_subtree {
        return Locator::MarkedInPage;
    }
    if let Some(id) = finding.backend_node_id.or_else(|| {
        ax_tree
            .get_node(&finding.node_id)
            .and_then(|n| n.backend_dom_node_id)
    }) {
        return Locator::Backend(id);
    }
    match finding.selector.as_deref().map(str::trim) {
        Some(sel) if !sel.is_empty() => Locator::Selector(sel),
        _ => Locator::PageLevel,
    }
}

impl ExclusionScope {
    /// Decide without asking the page: true for a finding located — by
    /// backend node id or by its rule's in-page mark — inside an excluded
    /// subtree. Selector-only findings answer false here; they go through
    /// [`filter_findings`].
    pub fn excludes_located(&self, finding: &Violation, ax_tree: &AXTree) -> bool {
        match locate(finding, ax_tree) {
            Locator::Backend(id) => self.contains(id),
            Locator::MarkedInPage => true,
            Locator::Selector(_) | Locator::PageLevel => false,
        }
    }

    /// True for a DOM node inside an excluded subtree — an iframe there is
    /// not audited at all (#715).
    pub fn excludes_backend(&self, backend_id: i64) -> bool {
        self.contains(backend_id)
    }

    /// True for an AX node whose DOM node lies inside an excluded subtree —
    /// the locating path of the screen-reader issues (#703).
    pub fn excludes_ax_node(&self, node_id: &str, ax_tree: &AXTree) -> bool {
        ax_tree
            .get_node(node_id)
            .and_then(|n| n.backend_dom_node_id)
            .is_some_and(|id| self.contains(id))
    }

    fn excludes_node(&self, node: &AXNode) -> bool {
        node.backend_dom_node_id.is_some_and(|id| self.contains(id))
    }

    /// `ax_tree` without the nodes inside excluded subtrees — what the
    /// page-level counting rules read (#726). `None` when nothing in the
    /// tree is excluded.
    pub fn without_excluded(&self, ax_tree: &AXTree) -> Option<AXTree> {
        if !ax_tree.iter_all().any(|n| self.excludes_node(n)) {
            return None;
        }
        Some(AXTree::from_nodes(
            ax_tree
                .iter_all()
                .filter(|n| !self.excludes_node(n))
                .cloned()
                .collect(),
        ))
    }

    /// Landmarks inside excluded subtrees — left out of the page-level
    /// landmark counts (#726).
    pub fn excluded_landmarks(&self, ax_tree: &AXTree) -> usize {
        ax_tree
            .iter()
            .filter(|n| crate::wcag::rules::is_landmark(n) && self.excludes_node(n))
            .count()
    }
}

/// Split `findings` into (kept, excluded). `selector_inside` answers, per
/// selector, whether every element it matches lies in an excluded subtree.
fn partition_findings(
    findings: Vec<Violation>,
    scope: &ExclusionScope,
    ax_tree: &AXTree,
    selector_inside: &HashMap<String, bool>,
) -> (Vec<Violation>, Vec<Violation>) {
    findings
        .into_iter()
        .partition(|f| match locate(f, ax_tree) {
            Locator::Backend(id) => !scope.contains(id),
            Locator::Selector(sel) => !selector_inside.get(sel).copied().unwrap_or(false),
            Locator::MarkedInPage => false,
            Locator::PageLevel => true,
        })
}

/// For each selector: does it match at least one element, all of them inside
/// an excluded subtree? Invalid or unmatched selectors answer `false`.
async fn selectors_inside(
    page: &Page,
    scope: &ExclusionScope,
    selectors: Vec<String>,
) -> HashMap<String, bool> {
    if selectors.is_empty() {
        return HashMap::new();
    }
    let excl = scope.active_selectors();
    let js = format!(
        r#"(function(sels, excl) {{
  function inside(el) {{
    var cur = el;
    while (cur) {{
      for (var i = 0; i < excl.length; i++) {{
        try {{ if (cur.matches && cur.matches(excl[i])) return true; }} catch (e) {{}}
      }}
      if (cur.parentElement) {{ cur = cur.parentElement; continue; }}
      var root = cur.getRootNode ? cur.getRootNode() : null;
      cur = (root && root.host) ? root.host : null;
    }}
    return false;
  }}
  return sels.map(function(s) {{
    var list;
    try {{ list = document.querySelectorAll(s); }} catch (e) {{ return false; }}
    if (!list.length) return false;
    for (var i = 0; i < list.length; i++) {{ if (!inside(list[i])) return false; }}
    return true;
  }});
}})({}, {})"#,
        serde_json::to_string(&selectors).unwrap_or_else(|_| "[]".to_string()),
        serde_json::to_string(&excl).unwrap_or_else(|_| "[]".to_string()),
    );
    let answers: Vec<bool> = match page.evaluate(js).await {
        Ok(result) => result.into_value().unwrap_or_default(),
        Err(e) => {
            warn!("Exclusion: selector check failed: {e}");
            Vec::new()
        }
    };
    selectors
        .into_iter()
        .enumerate()
        .map(|(i, sel)| (sel, answers.get(i).copied().unwrap_or(false)))
        .collect()
}

/// Drop the findings located inside the excluded subtrees; returns
/// `(kept, excluded)`. A no-op when nothing matched.
pub async fn filter_findings(
    page: &Page,
    scope: &ExclusionScope,
    ax_tree: &AXTree,
    findings: Vec<Violation>,
) -> (Vec<Violation>, Vec<Violation>) {
    if !scope.is_active() || findings.is_empty() {
        return (findings, Vec::new());
    }
    let mut pending: Vec<String> = findings
        .iter()
        .filter_map(|f| match locate(f, ax_tree) {
            Locator::Selector(sel) => Some(sel.to_string()),
            _ => None,
        })
        .collect();
    pending.sort();
    pending.dedup();
    let inside = selectors_inside(page, scope, pending).await;
    partition_findings(findings, scope, ax_tree, &inside)
}

/// Drop the journey findings whose selector lies inside the excluded
/// subtrees; returns the number dropped. Findings without a selector stay.
pub async fn filter_interactive_findings(
    page: &Page,
    scope: &ExclusionScope,
    findings: &mut Vec<InteractiveFinding>,
) -> usize {
    if !scope.is_active() {
        return 0;
    }
    let mut pending: Vec<String> = findings
        .iter()
        .filter_map(|f| f.values.selector.as_deref().map(str::trim))
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect();
    pending.sort();
    pending.dedup();
    if pending.is_empty() {
        return 0;
    }
    let inside = selectors_inside(page, scope, pending).await;
    let before = findings.len();
    findings.retain(|f| {
        !f.values
            .selector
            .as_deref()
            .map(str::trim)
            .and_then(|s| inside.get(s).copied())
            .unwrap_or(false)
    });
    before - findings.len()
}

// ── Tally → report ────────────────────────────────────────────────────────────

/// Collects what each viewport pass excluded, then folds it into the report.
#[derive(Debug, Default)]
pub struct ExclusionTally {
    /// Per selector: largest match count over the passes.
    selectors: Vec<SelectorMatch>,
    /// Per (rule id, criterion): (occurrences, violations) per pass.
    desktop: BTreeMap<(String, String), (usize, usize)>,
    mobile: BTreeMap<(String, String), (usize, usize)>,
    interactive: usize,
    screen_reader: usize,
    landmarks: usize,
}

#[derive(Debug, Clone, Copy)]
pub enum Pass {
    Desktop,
    Mobile,
}

impl ExclusionTally {
    pub fn record_scope(&mut self, scope: &ExclusionScope) {
        for m in &scope.matches {
            match self.selectors.iter_mut().find(|s| s.selector == m.selector) {
                Some(existing) => {
                    existing.matched_elements = existing.matched_elements.max(m.matched_elements);
                    existing.invalid |= m.invalid;
                }
                None => self.selectors.push(m.clone()),
            }
        }
    }

    pub fn record_findings(&mut self, pass: Pass, excluded: &[Violation]) {
        let map = match pass {
            Pass::Desktop => &mut self.desktop,
            Pass::Mobile => &mut self.mobile,
        };
        for f in excluded {
            let key = (
                f.rule_id.clone().unwrap_or_else(|| f.rule.clone()),
                f.rule.clone(),
            );
            let entry = map.entry(key).or_default();
            entry.0 += 1;
            if f.kind == Outcome::Fail {
                entry.1 += 1;
            }
        }
    }

    pub fn record_interactive(&mut self, count: usize) {
        self.interactive += count;
    }

    pub fn record_screen_reader(&mut self, count: usize) {
        self.screen_reader += count;
    }

    /// One viewport pass's excluded landmarks; the report keeps the larger.
    pub fn record_landmarks(&mut self, count: usize) {
        self.landmarks = self.landmarks.max(count);
    }

    pub fn into_report(self) -> ExclusionReport {
        let mut keys: Vec<&(String, String)> =
            self.desktop.keys().chain(self.mobile.keys()).collect();
        keys.sort();
        keys.dedup();
        let rules: Vec<ExcludedRuleCount> = keys
            .into_iter()
            .map(|key| {
                let d = self.desktop.get(key).copied().unwrap_or_default();
                let m = self.mobile.get(key).copied().unwrap_or_default();
                ExcludedRuleCount {
                    rule_id: key.0.clone(),
                    wcag: key.1.clone(),
                    occurrences: d.0.max(m.0),
                    violations: d.1.max(m.1),
                }
            })
            .collect();
        ExclusionReport {
            selectors: self
                .selectors
                .into_iter()
                .map(|s| ExclusionSelectorResult {
                    selector: s.selector,
                    builtin: s.builtin,
                    matched_elements: s.matched_elements,
                    invalid: s.invalid,
                })
                .collect(),
            excluded_occurrences: rules.iter().map(|r| r.occurrences).sum(),
            excluded_violations: rules.iter().map(|r| r.violations).sum(),
            rules,
            excluded_interactive_findings: self.interactive,
            excluded_screen_reader_issues: self.screen_reader,
            excluded_landmarks: self.landmarks,
        }
    }
}

// ── Batch aggregate ──────────────────────────────────────────────────────────

/// Audit exclusions summed over the pages of a batch.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct BatchExclusionSummary {
    pub selectors: Vec<BatchExclusionSelector>,
    /// Pages on which at least one selector matched an element.
    pub pages_with_matches: usize,
    pub excluded_occurrences: usize,
    pub excluded_violations: usize,
    pub excluded_interactive_findings: usize,
    #[serde(default)]
    pub excluded_screen_reader_issues: usize,
    #[serde(default)]
    pub excluded_landmarks: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct BatchExclusionSelector {
    pub selector: String,
    pub builtin: bool,
    /// Pages on which the selector matched at least one element.
    pub pages_matched: usize,
    /// Matched elements summed over all pages.
    pub matched_elements: usize,
    /// The browser rejected the selector on at least one page.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub invalid: bool,
}

impl BatchExclusionSummary {
    /// `None` when no page carries an exclusion record.
    pub fn aggregate<'a>(pages: impl IntoIterator<Item = &'a ExclusionReport>) -> Option<Self> {
        let mut summary: Option<Self> = None;
        for page in pages {
            let s = summary.get_or_insert_with(Self::default);
            if page.selectors.iter().any(|sel| sel.matched_elements > 0) {
                s.pages_with_matches += 1;
            }
            s.excluded_occurrences += page.excluded_occurrences;
            s.excluded_violations += page.excluded_violations;
            s.excluded_interactive_findings += page.excluded_interactive_findings;
            s.excluded_screen_reader_issues += page.excluded_screen_reader_issues;
            s.excluded_landmarks += page.excluded_landmarks;
            for sel in &page.selectors {
                let entry = match s
                    .selectors
                    .iter_mut()
                    .position(|e| e.selector == sel.selector)
                {
                    Some(i) => &mut s.selectors[i],
                    None => {
                        s.selectors.push(BatchExclusionSelector {
                            selector: sel.selector.clone(),
                            builtin: sel.builtin,
                            ..Default::default()
                        });
                        s.selectors.last_mut().expect("just pushed")
                    }
                };
                entry.matched_elements += sel.matched_elements;
                if sel.matched_elements > 0 {
                    entry.pages_matched += 1;
                }
                entry.invalid |= sel.invalid;
            }
        }
        summary
    }

    /// Same rule as [`ExclusionReport::is_noteworthy`].
    pub fn is_noteworthy(&self) -> bool {
        self.selectors
            .iter()
            .any(|s| !s.builtin || s.matched_elements > 0)
    }
}

// ── Presentation (PDF) ────────────────────────────────────────────────────────

/// Localized note for the batch cover's audit frame, or `None` when nothing
/// is worth mentioning.
pub fn batch_exclusion_note(
    summary: &BatchExclusionSummary,
    total_pages: usize,
    i18n: &crate::i18n::I18n,
) -> Option<String> {
    if !summary.is_noteworthy() {
        return None;
    }
    let mut parts: Vec<String> = summary
        .selectors
        .iter()
        .filter(|s| !s.builtin || s.matched_elements > 0)
        .map(|s| {
            if s.invalid && s.matched_elements == 0 {
                i18n.t_args(
                    "exclusion-selector-invalid",
                    &[("selector", FluentValue::from(s.selector.clone()))],
                )
            } else if s.matched_elements == 0 {
                i18n.t_args(
                    "exclusion-selector-unmatched",
                    &[("selector", FluentValue::from(s.selector.clone()))],
                )
            } else {
                i18n.t_args(
                    "batch-exclusion-selector-matched",
                    &[
                        ("selector", FluentValue::from(s.selector.clone())),
                        ("count", FluentValue::from(s.matched_elements as i64)),
                        ("pages", FluentValue::from(s.pages_matched as i64)),
                        ("total", FluentValue::from(total_pages as i64)),
                    ],
                )
            }
        })
        .collect();
    parts.push(findings_text(
        i18n,
        summary.excluded_occurrences,
        None,
        summary.excluded_interactive_findings,
        summary.excluded_screen_reader_issues,
        summary.excluded_landmarks,
    ));
    Some(parts.join("; "))
}

fn findings_text(
    i18n: &crate::i18n::I18n,
    occurrences: usize,
    rules: Option<usize>,
    interactive: usize,
    screen_reader: usize,
    landmarks: usize,
) -> String {
    let mut text = match rules {
        Some(rules) => i18n.t_args(
            "exclusion-findings",
            &[("occurrences", occurrences as i64), ("rules", rules as i64)],
        ),
        None => i18n.t_args(
            "batch-exclusion-findings",
            &[("occurrences", occurrences as i64)],
        ),
    };
    if interactive > 0 {
        text.push_str(&i18n.t_args(
            "exclusion-interactive",
            &[("interactive", interactive as i64)],
        ));
    }
    if screen_reader > 0 {
        text.push_str(&i18n.t_args(
            "exclusion-screen-reader",
            &[("issues", screen_reader as i64)],
        ));
    }
    if landmarks > 0 {
        text.push_str(&i18n.t_args("exclusion-landmarks", &[("landmarks", landmarks as i64)]));
    }
    text
}

/// Localized one-paragraph note for the methodology section, or `None` when
/// the report has nothing worth mentioning (see
/// [`ExclusionReport::is_noteworthy`]).
pub fn exclusion_note(report: &ExclusionReport, i18n: &crate::i18n::I18n) -> Option<String> {
    if !report.is_noteworthy() {
        return None;
    }
    let mut parts: Vec<String> = report
        .selectors
        .iter()
        .filter(|s| !s.builtin || s.matched_elements > 0)
        .map(|s| {
            let key = if s.invalid {
                "exclusion-selector-invalid"
            } else if s.matched_elements == 0 {
                "exclusion-selector-unmatched"
            } else {
                "exclusion-selector-matched"
            };
            i18n.t_args(
                key,
                &[
                    ("selector", FluentValue::from(s.selector.clone())),
                    ("count", FluentValue::from(s.matched_elements as i64)),
                ],
            )
        })
        .collect();
    parts.push(findings_text(
        i18n,
        report.excluded_occurrences,
        Some(report.rules.len()),
        report.excluded_interactive_findings,
        report.excluded_screen_reader_issues,
        report.excluded_landmarks,
    ));
    Some(parts.join("; "))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::WcagLevel;
    use crate::wcag::types::Severity;

    fn finding(rule_id: &str, node_id: &str) -> Violation {
        Violation::new(
            "1.1.1",
            "Non-text",
            WcagLevel::A,
            Severity::High,
            "m",
            node_id,
        )
        .with_rule_id(rule_id)
    }

    fn active_scope(ids: impl IntoIterator<Item = i64>) -> ExclusionScope {
        ExclusionScope::with_backend_ids(
            vec![SelectorMatch {
                selector: BUILTIN_EXCLUDE_SELECTOR.to_string(),
                builtin: true,
                matched_elements: 1,
                invalid: false,
            }],
            ids,
        )
    }

    #[test]
    fn effective_selectors_always_start_with_the_builtin_and_dedup() {
        let sel = effective_selectors(&[
            " [data-specimen] ".to_string(),
            "".to_string(),
            "[data-audit-exclude]".to_string(),
            "[data-specimen]".to_string(),
        ]);
        assert_eq!(sel, vec!["[data-audit-exclude]", "[data-specimen]"]);
    }

    #[test]
    fn backend_id_inside_scope_is_excluded_outside_kept() {
        let scope = active_scope([10, 11]);
        let mut inside = finding("image-alt", "document");
        inside.backend_node_id = Some(11);
        let mut outside = finding("image-alt", "document");
        outside.backend_node_id = Some(42);
        let (kept, excluded) = partition_findings(
            vec![inside, outside],
            &scope,
            &AXTree::default(),
            &HashMap::new(),
        );
        assert_eq!(kept.len(), 1);
        assert_eq!(kept[0].backend_node_id, Some(42));
        assert_eq!(excluded.len(), 1);
        assert_eq!(excluded[0].backend_node_id, Some(11));
    }

    #[test]
    fn a_known_backend_id_outside_wins_over_a_selector_answer() {
        let scope = active_scope([10]);
        let mut f = finding("x", "document").with_selector("div.specimen");
        f.backend_node_id = Some(99);
        let inside: HashMap<String, bool> = [("div.specimen".to_string(), true)].into();
        let (kept, excluded) = partition_findings(vec![f], &scope, &AXTree::default(), &inside);
        assert_eq!((kept.len(), excluded.len()), (1, 0));
    }

    #[test]
    fn selector_findings_follow_the_in_page_answer() {
        let scope = active_scope([]);
        let a = finding("click-events-have-key-events", "div#a").with_selector("div#a");
        let b = finding("click-events-have-key-events", "div#b").with_selector("div#b");
        let c = finding("click-events-have-key-events", "div#c").with_selector("div#c");
        let inside: HashMap<String, bool> = [
            ("div#a".to_string(), true),
            ("div#b".to_string(), false),
            // div#c: no answer (e.g. evaluation failed) → kept
        ]
        .into();
        let (kept, excluded) =
            partition_findings(vec![a, b, c], &scope, &AXTree::default(), &inside);
        let kept: Vec<_> = kept.iter().filter_map(|f| f.selector.as_deref()).collect();
        assert_eq!(kept, vec!["div#b", "div#c"]);
        assert_eq!(excluded[0].selector.as_deref(), Some("div#a"));
    }

    #[test]
    fn landmarks_in_an_excluded_specimen_leave_the_page_counts() {
        // #726: a specimen with its own banner must not make the page's
        // banner a duplicate.
        let node = |id: &str, role: &str, parent: Option<&str>, backend: i64| AXNode {
            node_id: id.into(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some(role.into()),
            name: None,
            name_source: None,
            description: None,
            value: None,
            properties: vec![],
            child_ids: vec![],
            parent_id: parent.map(String::from),
            backend_dom_node_id: Some(backend),
        };
        let tree = AXTree::from_nodes(vec![
            node("root", "RootWebArea", None, 1),
            node("b1", "banner", Some("root"), 2),
            node("spec", "generic", Some("root"), 10),
            node("b2", "banner", Some("spec"), 11),
        ]);
        let full = crate::wcag::rules::check_landmark_no_duplicate_banner(&tree);
        assert_eq!(full.violations.len(), 1);

        let scope = active_scope([10, 11]);
        assert_eq!(scope.excluded_landmarks(&tree), 1);
        let counted = scope.without_excluded(&tree).expect("specimen excluded");
        assert!(counted.get_node("b2").is_none() && counted.get_node("b1").is_some());
        let pruned = crate::wcag::rules::check_landmark_no_duplicate_banner(&counted);
        assert!(pruned.violations.is_empty());

        assert!(active_scope([99]).without_excluded(&tree).is_none());
    }

    #[test]
    fn page_level_findings_are_never_excluded() {
        let scope = active_scope([1, 2, 3]);
        let (kept, excluded) = partition_findings(
            vec![finding("document-title", "page")],
            &scope,
            &AXTree::default(),
            &HashMap::new(),
        );
        assert_eq!((kept.len(), excluded.len()), (1, 0));
    }

    #[test]
    fn inactive_scope_reports_nothing_active() {
        let scope = ExclusionScope {
            matches: vec![SelectorMatch {
                selector: ".x".into(),
                builtin: false,
                matched_elements: 0,
                invalid: false,
            }],
            ..Default::default()
        };
        assert!(!scope.is_active());
        assert!(scope.active_selectors().is_empty());
    }

    #[test]
    fn tally_takes_the_larger_pass_per_rule_and_max_matches() {
        let mut tally = ExclusionTally::default();
        let m = |n| ExclusionScope {
            matches: vec![
                SelectorMatch {
                    selector: BUILTIN_EXCLUDE_SELECTOR.into(),
                    builtin: true,
                    matched_elements: 0,
                    invalid: false,
                },
                SelectorMatch {
                    selector: "[data-specimen]".into(),
                    builtin: false,
                    matched_elements: n,
                    invalid: false,
                },
            ],
            ..Default::default()
        };
        tally.record_scope(&m(2));
        tally.record_scope(&m(3));
        let warn = finding("image-alt", "1").as_warning();
        tally.record_findings(
            Pass::Desktop,
            &[finding("image-alt", "1"), finding("image-alt", "2"), warn],
        );
        tally.record_findings(Pass::Mobile, &[finding("image-alt", "1")]);
        tally.record_findings(Pass::Mobile, &[finding("label", "3")]);
        tally.record_interactive(2);
        let report = tally.into_report();

        assert_eq!(report.selectors[1].matched_elements, 3);
        assert_eq!(report.selectors[0].matched_elements, 0);
        let alt = report
            .rules
            .iter()
            .find(|r| r.rule_id == "image-alt")
            .unwrap();
        assert_eq!((alt.occurrences, alt.violations), (3, 2));
        assert_eq!(report.excluded_occurrences, 4);
        assert_eq!(report.excluded_violations, 3);
        assert_eq!(report.excluded_interactive_findings, 2);
        assert!(report.is_noteworthy());
    }

    #[test]
    fn default_run_without_matches_is_not_noteworthy() {
        let report = ExclusionReport {
            selectors: vec![ExclusionSelectorResult {
                selector: BUILTIN_EXCLUDE_SELECTOR.into(),
                builtin: true,
                matched_elements: 0,
                invalid: false,
            }],
            ..Default::default()
        };
        assert!(!report.is_noteworthy());
    }

    #[test]
    fn note_names_unmatched_selectors_in_both_languages() {
        let report = ExclusionReport {
            selectors: vec![
                ExclusionSelectorResult {
                    selector: BUILTIN_EXCLUDE_SELECTOR.into(),
                    builtin: true,
                    matched_elements: 0,
                    invalid: false,
                },
                ExclusionSelectorResult {
                    selector: ".nothing".into(),
                    builtin: false,
                    matched_elements: 0,
                    invalid: false,
                },
                ExclusionSelectorResult {
                    selector: "[data-specimen]".into(),
                    builtin: false,
                    matched_elements: 2,
                    invalid: false,
                },
            ],
            excluded_occurrences: 5,
            excluded_violations: 4,
            rules: vec![ExcludedRuleCount {
                rule_id: "image-alt".into(),
                wcag: "1.1.1".into(),
                occurrences: 5,
                violations: 4,
            }],
            excluded_interactive_findings: 0,
            excluded_screen_reader_issues: 1,
            excluded_landmarks: 2,
        };
        let en = exclusion_note(&report, &crate::i18n::I18n::new("en").unwrap()).unwrap();
        assert!(en.contains(".nothing"), "{en}");
        assert!(en.contains("[data-specimen]"), "{en}");
        assert!(!en.contains(BUILTIN_EXCLUDE_SELECTOR), "{en}");
        assert!(en.contains('5'), "{en}");
        assert!(en.contains("1 screen-reader findings"), "{en}");
        assert!(en.contains("2 landmarks left out"), "{en}");
        assert!(
            !en.chars().any(|c| "äöüÄÖÜß".contains(c)),
            "EN note must not contain German: {en}"
        );
        let de = exclusion_note(&report, &crate::i18n::I18n::new("de").unwrap()).unwrap();
        assert_ne!(en, de);
        assert!(de.contains(".nothing"), "{de}");
    }
}
