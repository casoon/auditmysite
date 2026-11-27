//! Page 2 of the batch PDF: the overview (#1212). The scope of the run, the
//! one status statement and the answers to three questions — how the audited
//! pages score, what repeats, which page deserves a single report — all from
//! data the audit already measured.

use fluent_bundle::FluentValue;
use renderreport::components::advanced::{KeyValueList, SectionHeaderSplit};
use renderreport::components::text::Label;

use crate::audit::BatchReport;
use crate::i18n::I18n;
use crate::output::localized::is_english;
use crate::output::report_model::*;

use super::super::cover::build_batch_cover_score_row;
use super::super::helpers::severity_label_i18n;
use super::compare::page_label;
use super::management::{batch_wcag_headline, batch_wcag_headline_text};
use super::overview::{render_batch_rating, render_batch_unaudited_urls, unaudited_args};
use super::portfolio::{leading_items, localized_module_name};

/// How many of the most important findings the overview names.
const GLANCE_FINDINGS: usize = 3;

/// The scope of the run in one sentence: sample or complete, how many of how
/// many, how the pages were chosen and that a sample is no representative
/// choice.
pub(super) fn batch_scope_text(batch: &BatchReport, i18n: &I18n) -> Option<String> {
    let sample = batch.sample.as_ref()?;
    let source = i18n.t(&format!("batch-source-{}", sample.source));
    Some(if sample.is_sample {
        let pct = if sample.total_discovered > 0 {
            let pct = sample.audited as f64 * 100.0 / sample.total_discovered as f64;
            crate::i18n::number::decimal(pct, 1, is_english(i18n))
        } else {
            "0".to_string()
        };
        i18n.t_args(
            match sample.selection.as_str() {
                "first_n" => "batch-scope-sample",
                "spread" => "batch-scope-sample-spread",
                "stratified" => "batch-scope-sample-stratified",
                _ => "batch-scope-sample-filtered",
            },
            &[
                ("audited", sample.audited.to_string()),
                ("total", sample.total_discovered.to_string()),
                ("pct", pct),
                ("source", source),
                ("groups", sample.strata.len().to_string()),
                (
                    "covered",
                    sample
                        .strata
                        .iter()
                        .filter(|stratum| stratum.selected > 0)
                        .count()
                        .to_string(),
                ),
            ],
        )
    } else {
        i18n.t_args(
            "batch-scope-full",
            &[
                ("total", sample.total_discovered.to_string()),
                ("source", source),
            ],
        )
    })
}

/// The scope rows of the overview: what was audited, what was only checked
/// over HTTP and what could not be audited — the three sets a reader must not
/// mix up.
fn scope_rows(batch: &BatchReport, i18n: &I18n) -> Vec<(String, String)> {
    let mut rows = Vec::new();
    if let Some(scope) = batch_scope_text(batch, i18n) {
        rows.push((i18n.t("batch-cover-frame-scope"), scope));
    }
    if let Some(diagnostics) = &batch.sitemap_diagnostics {
        rows.push((
            i18n.t("batch-cover-frame-http"),
            i18n.t_args(
                "batch-cover-frame-http-value",
                &[
                    ("checked", diagnostics.checked_urls.to_string()),
                    ("audited", batch.reports.len().to_string()),
                ],
            ),
        ));
    }
    if !batch.errors.is_empty() {
        rows.push((
            i18n.t("batch-cover-frame-failed"),
            i18n.t_args(
                "batch-cover-frame-failed-value",
                &[
                    ("failed", batch.errors.len().to_string()),
                    (
                        "selected",
                        (batch.errors.len() + batch.reports.len()).to_string(),
                    ),
                ],
            ),
        ));
        rows.push((
            i18n.t("batch-cover-frame-score-basis"),
            i18n.t_args("batch-score-coverage", &unaudited_args(batch)),
        ));
    }
    rows
}

/// The answer to "how do the pages score?".
fn scores_answer(pres: &BatchPresentation, i18n: &I18n) -> String {
    let ranking = &pres.url_ranking;
    let min = ranking.first().map_or(0.0, |u| u.score).round() as i64;
    let max = ranking.last().map_or(0.0, |u| u.score).round() as i64;
    let severe = ranking.iter().filter(|u| u.critical_violations > 0).count() as i64;
    let args = [
        ("min", min),
        ("max", max),
        ("passed", pres.portfolio_summary.passed as i64),
        ("total", pres.portfolio_summary.total_urls as i64),
        ("severe", severe),
    ];
    i18n.t_args(
        if min == max {
            "batch-overview-a-scores-equal"
        } else {
            "batch-overview-a-scores"
        },
        &args,
    )
}

/// The answer to "what repeats?": how the finding types spread over the
/// audited pages.
fn repeats_answer(pres: &BatchPresentation, i18n: &I18n) -> String {
    let spread = &pres.finding_spread;
    if spread.types == 0 {
        return i18n.t("batch-overview-a-repeats-none");
    }
    let mut answer = i18n.t_args(
        "batch-overview-a-repeats",
        &[
            ("types", spread.types as i64),
            ("pages", spread.pages_with_findings as i64),
            ("total", pres.portfolio_summary.total_urls as i64),
            ("shared", spread.shared as i64),
            ("all", spread.on_all_pages as i64),
            ("specific", spread.page_specific as i64),
        ],
    );
    // The most important finding types, in the order of the batch's top
    // actions; the cards of the causes section tell each of them in full.
    let titles: Vec<String> = pres
        .top_issues
        .iter()
        .take(GLANCE_FINDINGS)
        .map(|group| group.title.clone())
        .collect();
    answer.push(' ');
    answer.push_str(&i18n.t_args(
        "batch-overview-a-repeats-top",
        &[("list", leading_items(&titles, GLANCE_FINDINGS, i18n))],
    ));
    answer
}

/// Why the recommended page deserves a single report, from the measured
/// counts of its pick; a finding of another module only when the scores
/// cannot tell the pages apart.
pub(super) fn pick_reason(
    pick: &SingleReportPick,
    pres: &BatchPresentation,
    i18n: &I18n,
) -> String {
    let mut reason = i18n.t_args(
        "batch-overview-pick-reason",
        &[
            ("types", pick.types as i64),
            ("batch_types", pres.finding_spread.types as i64),
            ("severe", pick.severe_types as i64),
            ("shared_on", pick.shared_types_on_page as i64),
            ("shared", pick.shared_types as i64),
        ],
    );
    if let Some(other) = pick.other_module.as_ref().filter(|_| pick.scores_tied) {
        reason.push(' ');
        reason.push_str(&i18n.t_args(
            "batch-overview-pick-tied",
            &[
                (
                    "module",
                    FluentValue::from(localized_module_name(
                        other.module.unwrap_or_default(),
                        i18n,
                    )),
                ),
                ("title", FluentValue::from(other.title.as_str())),
                (
                    "severity",
                    FluentValue::from(severity_label_i18n(other.severity, i18n)),
                ),
            ],
        ));
    }
    reason
}

/// The most important recurring finding type on the recommended page, named
/// as the cause to check first.
pub(super) fn pick_lead_text(
    pick: &SingleReportPick,
    pres: &BatchPresentation,
    i18n: &I18n,
) -> Option<String> {
    let lead = pick
        .lead_shared_rule
        .as_ref()
        .and_then(|rule| pres.top_issues.iter().find(|g| &g.finding.rule_id == rule))?;
    Some(i18n.t_args(
        "batch-overview-pick-lead",
        &[("title", lead.title.as_str())],
    ))
}

/// The answer to "which page deserves a single report?"; none for a batch of
/// one page, where there is nothing to choose from.
fn pick_answer(pres: &BatchPresentation, i18n: &I18n) -> Option<String> {
    match &pres.single_report_pick {
        Some(pick) => Some(i18n.t_args(
            "batch-overview-a-pick",
            &[
                ("page", page_label(pres, &pick.url)),
                ("reason", pick_reason(pick, pres, i18n)),
            ],
        )),
        None if pres.finding_spread.types == 0 && pres.portfolio_summary.total_urls >= 2 => {
            Some(i18n.t("batch-overview-a-pick-none"))
        }
        None => None,
    }
}

pub(super) fn render_batch_glance(
    mut builder: renderreport::engine::ReportBuilder,
    batch: &BatchReport,
    pres: &BatchPresentation,
    site_quality_score: u32,
    i18n: &I18n,
) -> anyhow::Result<renderreport::engine::ReportBuilder> {
    let accessibility_mean = pres
        .portfolio_summary
        .average_score
        .expect(super::UNMEASURED_REFUSED);
    let accessibility_average = accessibility_mean.round() as u32;

    builder = builder.add_component(
        SectionHeaderSplit::new(
            i18n.t("batch-overview-title"),
            i18n.t("batch-overview-kicker"),
        )
        .with_eyebrow(i18n.t("batch-cover-eyebrow"))
        .with_level(1),
    );

    let rows = scope_rows(batch, i18n);
    if !rows.is_empty() {
        let mut scope = KeyValueList::new();
        for (key, value) in rows {
            scope = scope.add(key, value);
        }
        builder = builder.add_component(scope);
    }

    // The grade card sits under the scope; the site-quality average (the
    // mixed score over the optional modules) is the labelled secondary note
    // below the row.
    builder = builder
        .add_component(build_batch_cover_score_row(
            accessibility_mean,
            &pres.portfolio_summary.grade,
            batch.summary.decision.verdict,
            batch.summary.audit_quality.qualified_results,
            pres.portfolio_summary.total_urls as u32,
            pres.portfolio_summary.passed as u32,
            i18n,
        )?)
        .add_component(
            Label::new(i18n.t_args(
                "batch-cover-site-quality-score-note",
                &[("score", site_quality_score as i64)],
            ))
            .with_size("8.5pt")
            .with_color(crate::output::pdf::design::tokens::MUTED),
        );
    if let Some(baseline) = &pres.portfolio_summary.baseline {
        builder = builder.add_component(
            Label::new(crate::audit::baseline::summary_text(
                baseline.known,
                baseline.new,
                baseline.fixed,
                Some(baseline.unmatched_pages),
                is_english(i18n),
            ))
            .with_size("8.5pt")
            .with_color(crate::output::pdf::design::tokens::MUTED),
        );
    }

    // What the automated WCAG check found on the audited pages, stated
    // factually; a finding of another module never decides it (#1208).
    builder = builder.add_component(
        Label::new(batch_wcag_headline_text(
            batch_wcag_headline(pres),
            pres.portfolio_summary.total_urls,
            i18n,
        ))
        .bold()
        .with_size("11pt"),
    );

    // The one status statement of the report: verdict and its reasons.
    builder = render_batch_rating(builder, batch, accessibility_average, i18n);
    builder = render_batch_unaudited_urls(builder, batch, i18n);

    let mut glance = KeyValueList::new();
    // The verdict sentence of the report answers the first question; the
    // counts continue it.
    glance = glance.add(
        i18n.t("batch-overview-q-scores"),
        format!(
            "{} {}",
            pres.portfolio_summary.verdict_text,
            scores_answer(pres, i18n)
        ),
    );
    glance = glance.add(
        i18n.t("batch-overview-q-repeats"),
        repeats_answer(pres, i18n),
    );
    if let Some(answer) = pick_answer(pres, i18n) {
        glance = glance.add(i18n.t("batch-overview-q-pick"), answer);
    }
    builder = builder.add_component(glance);

    Ok(builder)
}
