//! Before/after comparison of two JSON reports (#1235): `auditmysite compare
//! <OLD> <NEW>` reads two reports written by `-f json` and answers what moved
//! between the runs: the scores, the findings that are gone, new or still
//! there, and the verdict.
//!
//! Offline and browser-free: the input is the canonical JSON only. Findings are
//! matched by their stable id (`F-` plus a hash of the rule id,
//! [`finding_id_of`](crate::audit::baseline::finding_id_of)), so one entry
//! means "this rule is reported on this page", as in the findings baseline; a
//! new element violating a known rule is not a new finding. A report whose ids
//! this tool would not derive from the rule (another `FINDING_ID_SCHEME`, or
//! hand-edited) is refused instead of reading as "everything is new".
//!
//! A batch is compared per URL present in both reports; URLs in only one are
//! listed and enter no score and no count. This module imports neither `cli`
//! nor `output`; the table and the PDF are rendered by `output::compare`.

mod input;

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use crate::audit::baseline::FINDING_ID_SCHEME;

pub(crate) use input::{read_report, Report};

/// Contract of the comparison JSON (`docs/json-compare.schema.json`).
pub(crate) const COMPARE_CONTRACT: &str = "auditmysite.compare/1";

/// A score of the old and the new run. Averages over several pages carry one
/// decimal; `None` where a side has no such score.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub(crate) struct ScoreMove {
    pub old: Option<f64>,
    pub new: Option<f64>,
    pub delta: Option<f64>,
}

impl ScoreMove {
    fn of(old: Option<f64>, new: Option<f64>) -> Self {
        Self {
            old: old.map(round1),
            new: new.map(round1),
            delta: old.zip(new).map(|(o, n)| round1(n - o)),
        }
    }

    /// The new score is lower than the old one.
    pub(crate) fn lost(&self) -> bool {
        self.delta.is_some_and(|d| d < 0.0)
    }
}

fn round1(value: f64) -> f64 {
    let rounded = (value * 10.0).round() / 10.0;
    // `-0.0` would print and serialize as a negative zero.
    if rounded == 0.0 {
        0.0
    } else {
        rounded
    }
}

/// One module score of both runs (per page, or averaged over the pages
/// compared).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct ModuleMove {
    pub name: String,
    #[serde(flatten)]
    pub score: ScoreMove,
}

/// The verdict of both runs (`pass`, `warn`, `fail`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct VerdictMove {
    pub old: String,
    pub new: String,
    pub changed: bool,
}

/// A finding as listed in the comparison.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct FindingRef {
    pub id: String,
    pub rule: String,
    pub title: String,
    pub severity: Option<String>,
}

/// The findings of one page, by what happened to them.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub(crate) struct FindingMoves {
    /// In the old run, not in the new one.
    pub resolved: Vec<FindingRef>,
    /// In the new run, not in the old one.
    pub new: Vec<FindingRef>,
    /// In both runs.
    pub unchanged: Vec<FindingRef>,
}

/// One page present in both reports.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct PageComparison {
    pub url: String,
    /// The URL of the old run, only where it differs from `url` (two single
    /// reports of different URLs are compared as one pair).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_url: Option<String>,
    pub overall: ScoreMove,
    pub accessibility: ScoreMove,
    pub modules: Vec<ModuleMove>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verdict: Option<VerdictMove>,
    pub findings: FindingMoves,
}

/// What the comparison says of one run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct RunInfo {
    pub report_type: String,
    pub tool_version: Option<String>,
    pub page_count: usize,
}

/// Counts over every page compared.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct Summary {
    pub pages_compared: usize,
    pub resolved: usize,
    pub new: usize,
    pub unchanged: usize,
    /// The new run is worse: it has new findings or lost score.
    pub worse: bool,
    /// Why: `new_findings`, `score_loss`.
    pub reasons: Vec<&'static str>,
}

/// The comparison of two reports.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct Comparison {
    pub contract: &'static str,
    pub id_scheme: u32,
    pub old: RunInfo,
    pub new: RunInfo,
    /// `site_quality_score`, averaged over the pages compared.
    pub overall: ScoreMove,
    /// `accessibility_score`, averaged over the pages compared.
    pub accessibility: ScoreMove,
    /// Module scores, averaged over the pages compared that have the module
    /// in both runs.
    pub modules: Vec<ModuleMove>,
    /// The decision of the report as a whole (the batch decision of a batch).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verdict: Option<VerdictMove>,
    pub summary: Summary,
    pub pages: Vec<PageComparison>,
    /// URLs only the old report has.
    pub only_in_old: Vec<String>,
    /// URLs only the new report has.
    pub only_in_new: Vec<String>,
}

impl Comparison {
    /// The process exit code: `0` when the new run has no new finding and
    /// lost no score, `2` when it is worse (`3`, malformed input, is an error
    /// before a comparison exists).
    pub(crate) fn exit_code(&self) -> i32 {
        if self.summary.worse {
            2
        } else {
            0
        }
    }
}

fn verdict_move(old: Option<&str>, new: Option<&str>) -> Option<VerdictMove> {
    let (old, new) = (old?, new?);
    Some(VerdictMove {
        old: old.to_string(),
        new: new.to_string(),
        changed: old != new,
    })
}

fn finding_ref(f: &input::Finding) -> FindingRef {
    FindingRef {
        id: f.id.clone(),
        rule: f.rule_id.clone(),
        title: f.title.clone().unwrap_or_else(|| f.rule_id.clone()),
        severity: f.severity.clone(),
    }
}

fn finding_moves(old: &input::Page, new: &input::Page) -> FindingMoves {
    let old_by_id: BTreeMap<&str, &input::Finding> =
        old.findings.iter().map(|f| (f.id.as_str(), f)).collect();
    let new_by_id: BTreeMap<&str, &input::Finding> =
        new.findings.iter().map(|f| (f.id.as_str(), f)).collect();
    let mut moves = FindingMoves::default();
    for (id, finding) in &old_by_id {
        if !new_by_id.contains_key(id) {
            moves.resolved.push(finding_ref(finding));
        }
    }
    for (id, finding) in &new_by_id {
        if old_by_id.contains_key(id) {
            moves.unchanged.push(finding_ref(finding));
        } else {
            moves.new.push(finding_ref(finding));
        }
    }
    for list in [&mut moves.resolved, &mut moves.new, &mut moves.unchanged] {
        list.sort_by(|a, b| a.rule.cmp(&b.rule));
    }
    moves
}

fn module_moves(old: &input::Page, new: &input::Page) -> Vec<ModuleMove> {
    let score_of = |page: &input::Page, name: &str| {
        page.module_scores
            .iter()
            .find(|m| m.name == name)
            .map(|m| f64::from(m.score))
    };
    let mut seen = BTreeSet::new();
    let mut moves = Vec::new();
    for module in new.module_scores.iter().chain(&old.module_scores) {
        if !seen.insert(module.name.as_str()) {
            continue;
        }
        moves.push(ModuleMove {
            name: module.name.clone(),
            score: ScoreMove::of(score_of(old, &module.name), score_of(new, &module.name)),
        });
    }
    moves
}

fn mean(values: impl Iterator<Item = f64>) -> Option<f64> {
    let (sum, n) = values.fold((0.0, 0usize), |(s, n), v| (s + v, n + 1));
    (n > 0).then(|| sum / n as f64)
}

/// Compares `old` with `new`.
pub(crate) fn compare(old: &Report, new: &Report) -> Comparison {
    let mut only_in_old = Vec::new();
    let mut only_in_new = Vec::new();

    // Two single reports are one pair, whatever their URLs; otherwise the
    // pages are matched by URL.
    let pairs: Vec<(&input::Page, &input::Page)> = if old.is_single() && new.is_single() {
        vec![(&old.pages[0], &new.pages[0])]
    } else {
        for page in &old.pages {
            if !new.pages.iter().any(|p| p.url == page.url) {
                only_in_old.push(page.url.clone());
            }
        }
        for page in &new.pages {
            if !old.pages.iter().any(|p| p.url == page.url) {
                only_in_new.push(page.url.clone());
            }
        }
        new.pages
            .iter()
            .filter_map(|n| old.pages.iter().find(|o| o.url == n.url).map(|o| (o, n)))
            .collect()
    };

    let pages: Vec<PageComparison> = pairs
        .iter()
        .map(|(o, n)| PageComparison {
            url: n.url.clone(),
            old_url: (o.url != n.url).then(|| o.url.clone()),
            overall: ScoreMove::of(
                Some(f64::from(o.site_quality_score)),
                Some(f64::from(n.site_quality_score)),
            ),
            accessibility: ScoreMove::of(
                Some(f64::from(o.accessibility_score)),
                Some(f64::from(n.accessibility_score)),
            ),
            modules: module_moves(o, n),
            verdict: verdict_move(o.verdict.as_deref(), n.verdict.as_deref()),
            findings: finding_moves(o, n),
        })
        .collect();

    let average = |pick: fn(&input::Page) -> u32| -> ScoreMove {
        ScoreMove::of(
            mean(pairs.iter().map(|(o, _)| f64::from(pick(o)))),
            mean(pairs.iter().map(|(_, n)| f64::from(pick(n)))),
        )
    };
    let overall = average(|p| p.site_quality_score);
    let accessibility = average(|p| p.accessibility_score);

    // Module averages: unrounded page values, only pages with the module in
    // both runs.
    let mut module_names: Vec<&str> = Vec::new();
    for (o, n) in &pairs {
        for module in n.module_scores.iter().chain(&o.module_scores) {
            if !module_names.contains(&module.name.as_str()) {
                module_names.push(&module.name);
            }
        }
    }
    let modules: Vec<ModuleMove> = module_names
        .into_iter()
        .map(|name| {
            let both: Vec<(f64, f64)> = pairs
                .iter()
                .filter_map(|(o, n)| {
                    let find = |p: &input::Page| {
                        p.module_scores
                            .iter()
                            .find(|m| m.name == name)
                            .map(|m| f64::from(m.score))
                    };
                    find(o).zip(find(n))
                })
                .collect();
            ModuleMove {
                name: name.to_string(),
                score: ScoreMove::of(
                    mean(both.iter().map(|(o, _)| *o)),
                    mean(both.iter().map(|(_, n)| *n)),
                ),
            }
        })
        .collect();

    let count = |f: fn(&FindingMoves) -> usize| pages.iter().map(|p| f(&p.findings)).sum::<usize>();
    let (resolved, new_findings, unchanged) = (
        count(|m| m.resolved.len()),
        count(|m| m.new.len()),
        count(|m| m.unchanged.len()),
    );
    let mut reasons = Vec::new();
    if new_findings > 0 {
        reasons.push("new_findings");
    }
    if overall.lost() || accessibility.lost() {
        reasons.push("score_loss");
    }

    Comparison {
        contract: COMPARE_CONTRACT,
        id_scheme: FINDING_ID_SCHEME,
        old: old.info(),
        new: new.info(),
        overall,
        accessibility,
        modules,
        verdict: verdict_move(old.verdict.as_deref(), new.verdict.as_deref()),
        summary: Summary {
            pages_compared: pages.len(),
            resolved,
            new: new_findings,
            unchanged,
            worse: !reasons.is_empty(),
            reasons,
        },
        pages,
        only_in_old,
        only_in_new,
    }
}

#[cfg(test)]
mod tests;
