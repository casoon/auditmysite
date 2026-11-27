//! The before/after comparison of two reports (#1235) as text: the one view
//! both the terminal table and the PDF render, and the JSON form. The data is
//! `compare::Comparison`; every text is a message of `locales/<lang>/compare.ftl`.

use fluent_bundle::FluentValue;

use crate::compare::{Comparison, FindingRef, ModuleMove, ScoreMove, VerdictMove};
use crate::i18n::{count_text, text, text_args, try_text_args};

/// The comparison as JSON (`docs/json-compare.schema.json`).
pub(crate) fn format_compare_json(comparison: &Comparison) -> String {
    let mut json = serde_json::to_string_pretty(comparison)
        .expect("a comparison is plain data and always serializes");
    json.push('\n');
    json
}

/// A label and its value.
pub(crate) type Row = (String, String);

/// One score row: the name and the old, new and change cells.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ScoreRow {
    pub name: String,
    pub old: String,
    pub new: String,
    pub change: String,
}

/// A group of findings (resolved, new, unchanged) of one page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FindingGroup {
    pub heading: String,
    /// One line per finding; empty means none.
    pub items: Vec<String>,
}

/// A compared page of a batch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PageRow {
    pub label: String,
    pub overall: String,
    pub verdict: String,
    pub findings: String,
    /// The new findings of the page, one line each.
    pub new_items: Vec<String>,
}

/// What the table and the PDF show.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct View {
    pub title: String,
    pub runs: Vec<Row>,
    pub scores_heading: String,
    pub score_columns: [String; 4],
    pub scores: Vec<ScoreRow>,
    pub verdict: Option<Row>,
    pub findings_heading: String,
    pub findings_counts: String,
    /// The finding groups of a single comparison; a batch lists them per page.
    pub groups: Vec<FindingGroup>,
    pub pages_heading: Option<String>,
    pub page_columns: [String; 3],
    pub pages: Vec<PageRow>,
    pub only_old: Option<FindingGroup>,
    pub only_new: Option<FindingGroup>,
    pub result: String,
    pub worse: bool,
    pub note: String,
}

fn number(n: usize) -> FluentValue<'static> {
    FluentValue::from(n.to_string())
}

fn score(value: Option<f64>) -> String {
    match value {
        None => "-".to_string(),
        Some(v) if v.fract() == 0.0 => format!("{v:.0}"),
        Some(v) => format!("{v:.1}"),
    }
}

fn change(delta: Option<f64>) -> String {
    match delta {
        None => "-".to_string(),
        Some(d) if d > 0.0 => format!("+{}", score(Some(d))),
        Some(d) => score(Some(d)),
    }
}

fn score_row(name: String, s: &ScoreMove) -> ScoreRow {
    ScoreRow {
        name,
        old: score(s.old),
        new: score(s.new),
        change: change(s.delta),
    }
}

fn module_name(module: &ModuleMove, en: bool) -> String {
    if module.name == "Accessibility" {
        text("compare-score-accessibility", en)
    } else {
        module.name.clone()
    }
}

fn verdict_word(verdict: &str, en: bool) -> String {
    try_text_args(&format!("compare-verdict-{verdict}"), &[], en)
        .unwrap_or_else(|| verdict.to_string())
}

fn verdict_text(verdict: &VerdictMove, en: bool) -> String {
    let old = verdict_word(&verdict.old, en);
    if verdict.changed {
        let new = verdict_word(&verdict.new, en);
        text_args(
            "compare-verdict-changed",
            &[("old", old.into()), ("new", new.into())],
            en,
        )
    } else {
        text_args("compare-verdict-unchanged", &[("verdict", old.into())], en)
    }
}

fn finding_line(f: &FindingRef) -> String {
    match &f.severity {
        Some(severity) => format!("{}  {} ({}, {severity})", f.id, f.title, f.rule),
        None => format!("{}  {} ({})", f.id, f.title, f.rule),
    }
}

fn counts_text(resolved: usize, new: usize, unchanged: usize, en: bool) -> String {
    text_args(
        "compare-findings-counts",
        &[
            ("resolved", number(resolved)),
            ("new", number(new)),
            ("unchanged", number(unchanged)),
        ],
        en,
    )
}

fn run_row(label: String, name: &str, info: &crate::compare::RunInfo, en: bool) -> Row {
    let kind = if info.report_type == "single" {
        text("compare-run-single", en)
    } else {
        text("compare-run-batch", en)
    };
    let mut parts = vec![
        name.to_string(),
        kind,
        count_text("compare-run-pages", info.page_count, en),
    ];
    if let Some(version) = &info.tool_version {
        parts.push(text_args(
            "compare-run-tool",
            &[("version", version.clone().into())],
            en,
        ));
    }
    (label, parts.join(", "))
}

/// The view of `comparison`; `old_name` and `new_name` name the two files.
pub(crate) fn view(comparison: &Comparison, old_name: &str, new_name: &str, en: bool) -> View {
    let single = comparison.old.report_type == "single" && comparison.new.report_type == "single";
    let t = |key: &str| text(key, en);

    let mut scores = vec![score_row(t("compare-score-overall"), &comparison.overall)];
    if comparison.modules.iter().all(|m| m.name != "Accessibility") {
        scores.push(score_row(
            t("compare-score-accessibility"),
            &comparison.accessibility,
        ));
    }
    scores.extend(
        comparison
            .modules
            .iter()
            .map(|m| score_row(module_name(m, en), &m.score)),
    );

    let summary = &comparison.summary;
    let groups = match comparison.pages.first() {
        Some(page) if single => vec![
            ("compare-resolved", &page.findings.resolved),
            ("compare-new", &page.findings.new),
            ("compare-unchanged", &page.findings.unchanged),
        ]
        .into_iter()
        .map(|(key, list)| FindingGroup {
            heading: t(key),
            items: list.iter().map(finding_line).collect(),
        })
        .collect(),
        _ => Vec::new(),
    };

    let pages = if single {
        Vec::new()
    } else {
        comparison
            .pages
            .iter()
            .map(|page| PageRow {
                label: page.url.clone(),
                overall: format!(
                    "{} → {} ({})",
                    score(page.overall.old),
                    score(page.overall.new),
                    change(page.overall.delta)
                ),
                verdict: page
                    .verdict
                    .as_ref()
                    .map(|v| verdict_text(v, en))
                    .unwrap_or_else(|| "-".to_string()),
                findings: counts_text(
                    page.findings.resolved.len(),
                    page.findings.new.len(),
                    page.findings.unchanged.len(),
                    en,
                ),
                new_items: page.findings.new.iter().map(finding_line).collect(),
            })
            .collect()
    };

    let only = |key: &str, urls: &[String]| {
        (!urls.is_empty()).then(|| FindingGroup {
            heading: t(key),
            items: urls.to_vec(),
        })
    };

    let result = if summary.worse {
        let reasons: Vec<String> = summary
            .reasons
            .iter()
            .map(|r| t(&format!("compare-reason-{}", r.replace('_', "-"))))
            .collect();
        text_args(
            "compare-result-worse",
            &[("reasons", reasons.join(", ").into())],
            en,
        )
    } else {
        t("compare-result-ok")
    };

    let mut runs = vec![
        run_row(t("compare-run-old"), old_name, &comparison.old, en),
        run_row(t("compare-run-new"), new_name, &comparison.new, en),
    ];
    if !single {
        runs.push((
            String::new(),
            count_text("compare-pages-compared", summary.pages_compared, en),
        ));
    }
    if let (true, Some(page)) = (single, comparison.pages.first()) {
        if let Some(old_url) = &page.old_url {
            runs.push((
                page.url.clone(),
                text_args(
                    "compare-page-old-url",
                    &[("url", old_url.clone().into())],
                    en,
                ),
            ));
        }
    }

    View {
        title: t("compare-title"),
        runs,
        scores_heading: t(if single {
            "compare-scores"
        } else {
            "compare-scores-average"
        }),
        score_columns: [
            t("compare-col-score"),
            t("compare-col-old"),
            t("compare-col-new"),
            t("compare-col-change"),
        ],
        scores,
        verdict: comparison
            .verdict
            .as_ref()
            .map(|v| (t("compare-verdict"), verdict_text(v, en))),
        findings_heading: t("compare-findings"),
        findings_counts: counts_text(summary.resolved, summary.new, summary.unchanged, en),
        groups,
        pages_heading: (!single).then(|| t("compare-pages")),
        page_columns: [
            t("compare-col-page"),
            t("compare-col-score"),
            t("compare-col-findings"),
        ],
        pages,
        only_old: only("compare-only-old", &comparison.only_in_old),
        only_new: only("compare-only-new", &comparison.only_in_new),
        result,
        worse: summary.worse,
        note: t("compare-note"),
    }
}

fn pad(text: &str, width: usize) -> String {
    let len = text.chars().count();
    format!("{text}{}", " ".repeat(width.saturating_sub(len)))
}

fn pad_left(text: &str, width: usize) -> String {
    let len = text.chars().count();
    format!("{}{text}", " ".repeat(width.saturating_sub(len)))
}

/// The terminal table: plain text, no colors, one section after the other.
pub(crate) fn render_table(view: &View) -> String {
    let mut out = String::new();
    let line = |out: &mut String, text: &str| {
        out.push_str(text);
        out.push('\n');
    };
    line(&mut out, &view.title);
    line(&mut out, &"=".repeat(view.title.chars().count()));
    let label_width = view
        .runs
        .iter()
        .map(|(l, _)| l.chars().count())
        .max()
        .unwrap_or(0);
    for (label, value) in &view.runs {
        line(&mut out, &format!("{}  {value}", pad(label, label_width)));
    }

    line(&mut out, "");
    line(&mut out, &view.scores_heading);
    let name_width = view
        .scores
        .iter()
        .map(|r| r.name.chars().count())
        .chain([view.score_columns[0].chars().count()])
        .max()
        .unwrap_or(0);
    let widths = [1usize, 2, 3].map(|i| {
        let cell = |r: &ScoreRow| match i {
            1 => r.old.chars().count(),
            2 => r.new.chars().count(),
            _ => r.change.chars().count(),
        };
        view.scores
            .iter()
            .map(cell)
            .chain([view.score_columns[i].chars().count()])
            .max()
            .unwrap_or(0)
    });
    line(
        &mut out,
        &format!(
            "  {}  {}  {}  {}",
            pad(&view.score_columns[0], name_width),
            pad_left(&view.score_columns[1], widths[0]),
            pad_left(&view.score_columns[2], widths[1]),
            pad_left(&view.score_columns[3], widths[2]),
        ),
    );
    for row in &view.scores {
        line(
            &mut out,
            &format!(
                "  {}  {}  {}  {}",
                pad(&row.name, name_width),
                pad_left(&row.old, widths[0]),
                pad_left(&row.new, widths[1]),
                pad_left(&row.change, widths[2]),
            ),
        );
    }
    if let Some((label, value)) = &view.verdict {
        line(&mut out, "");
        line(&mut out, &format!("{label}: {value}"));
    }

    line(&mut out, "");
    line(
        &mut out,
        &format!("{}: {}", view.findings_heading, view.findings_counts),
    );
    for group in &view.groups {
        if group.items.is_empty() {
            continue;
        }
        line(&mut out, &format!("  {}", group.heading));
        for item in &group.items {
            line(&mut out, &format!("    {item}"));
        }
    }

    if let Some(heading) = &view.pages_heading {
        line(&mut out, "");
        line(&mut out, heading);
        for page in &view.pages {
            line(&mut out, &format!("  {}", page.label));
            line(
                &mut out,
                &format!(
                    "    {}: {}; {}; {}",
                    view.page_columns[1], page.overall, page.verdict, page.findings
                ),
            );
            for item in &page.new_items {
                line(&mut out, &format!("    + {item}"));
            }
        }
    }
    for group in [&view.only_old, &view.only_new].into_iter().flatten() {
        line(&mut out, "");
        line(&mut out, &group.heading);
        for item in &group.items {
            line(&mut out, &format!("  {item}"));
        }
    }

    line(&mut out, "");
    line(&mut out, &view.result);
    line(&mut out, &view.note);
    out
}
