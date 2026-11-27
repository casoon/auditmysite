use crate::output::localized::is_english;
use renderreport::components::advanced::{List, PageBreak, SectionHeaderSplit, TableOfContents};
use renderreport::components::text::Label;
use renderreport::components::{AuditTable, TableColumn};
use renderreport::components::{CoverModuleGauge, CoverPage};
use renderreport::prelude::*;

use crate::audit::BatchReport;
use crate::i18n::I18n;
use crate::model::ReportLevel;
use crate::output::report_model::*;
use crate::util::short_url;

// ─── Batch Report Sections ──────────────────────────────────────────────────

pub(super) fn render_batch_cover(
    mut builder: renderreport::engine::ReportBuilder,
    batch: &BatchReport,
    pres: &BatchPresentation,
    config: &ReportConfig,
    score: u32,
    i18n: &I18n,
) -> anyhow::Result<renderreport::engine::ReportBuilder> {
    let domain = &pres.portfolio_summary.domain;
    // The cover names the site as the single report does, without "www.".
    let cover_domain = domain.strip_prefix("www.").unwrap_or(domain);
    let en = is_english(i18n);

    // The cover's headline number is the accessibility average, the figure
    // the grade card on the next page is banded from. The site-quality average
    // (the mixed score over the optional modules) is the labelled secondary
    // note further down.
    let accessibility_mean = pres
        .portfolio_summary
        .average_score
        .expect(super::UNMEASURED_REFUSED);
    let accessibility_average = accessibility_mean.round() as u32;
    let verdict = batch.summary.decision.verdict;
    let partial = batch.summary.audit_quality.qualified_results;

    // White-label: only a user-supplied custom logo takes the brand slot on
    // the cover, as in the single report.
    let cover_logo_asset = super::super::cover_logo_asset(config);
    let custom_logo = if cover_logo_asset == super::super::CUSTOM_COVER_LOGO_ASSET {
        builder = super::super::register_cover_logo_asset(builder, config, cover_logo_asset);
        Some(cover_logo_asset)
    } else {
        None
    };

    // One subtitle line: what was audited. The frame table on the next page
    // carries the full scope text and its caveats.
    let subtitle = match &batch.sample {
        Some(sample) if sample.is_sample => i18n.t_args(
            "batch-cover-subtitle-sample",
            &[
                ("audited", sample.audited.to_string()),
                ("total", sample.total_discovered.to_string()),
                ("source", i18n.t(&format!("batch-source-{}", sample.source))),
            ],
        ),
        _ => pres.cover.url.clone(),
    };
    let gauges: Vec<CoverModuleGauge> = pres
        .portfolio_summary
        .module_averages
        .iter()
        .map(|(name, average)| {
            CoverModuleGauge::new(
                super::portfolio::localized_module_name(name, i18n),
                *average,
            )
        })
        .collect();
    let occurrences = &pres.portfolio_summary.occurrence_distribution;
    let mut cover = CoverPage::new(
        &i18n.t("batch-cover-title"),
        cover_domain,
        accessibility_average,
        &pres.portfolio_summary.grade,
    )
    .with_brand(cover_domain)
    .with_subtitle(&subtitle)
    .with_date(&pres.cover.date)
    .with_band_phrase(super::super::cover::cover_band_phrase(
        accessibility_average,
        verdict,
        partial,
        en,
    ))
    .with_issues(
        pres.portfolio_summary.total_violations as u32,
        (occurrences.critical + occurrences.high) as u32,
    )
    .with_module_gauges(gauges)
    .with_labels(
        i18n.t("pdf-frame-cover-score-label"),
        i18n.t("pdf-frame-cover-findings-label"),
        i18n.t("batch-cover-modules-label"),
        i18n.t("pdf-frame-cover-critical-label"),
        i18n.t("pdf-frame-cover-no-critical"),
    );
    if let Some(logo) = custom_logo {
        cover = cover.with_logo(logo);
    }
    // One decimal when the integer would hide a mean below 100 (#1255).
    let mut cover_data =
        super::super::cover::with_displayed_score(cover.to_data(), accessibility_mean);
    // The Accessibility ring next to it carries the same figure.
    if let Some(accessibility_ring) = pres
        .portfolio_summary
        .module_averages
        .iter()
        .position(|(name, _)| name == "Accessibility")
        .and_then(|index| cover_data.pointer_mut(&format!("/module_gauges/{index}")))
    {
        *accessibility_ring = super::super::cover::with_displayed_score(
            accessibility_ring.take(),
            accessibility_mean,
        );
    }
    let mut cover = renderreport::components::DynamicComponent::new(cover.component_id());
    if let Some(fields) = cover_data.as_object() {
        cover.data = fields.clone().into_iter().collect();
    }
    builder = builder.add_component(cover);

    // Page 2: the overview. Scope, the one status statement and the three
    // answers (#1212).
    builder = super::glance::render_batch_glance(builder, batch, pres, score, i18n)?;

    // The scope note used to close page 2, which it overflowed onto an
    // otherwise empty page 3. It now opens the contents page; an
    // executive report has no contents page and keeps it on page 2.
    let scope_note = batch_scope_callout(
        config.level == ReportLevel::Technical && !pres.appendix.per_url.is_empty(),
        i18n,
    );
    if !config.level.is_summary() {
        builder = builder
            .add_component(PageBreak::new())
            .add_component(scope_note)
            .add_component(
                TableOfContents::new()
                    .with_title(i18n.t("batch-report-contents"))
                    .with_depth(1),
            );
    } else {
        builder = builder
            .add_component(scope_note)
            .add_component(PageBreak::new());
    }

    Ok(builder)
}

/// What this PDF leaves out and where it is. It names the technical appendix
/// only when this report has one — the same condition `build_batch_report`
/// renders it under (#1039).
fn batch_scope_callout(has_appendix: bool, i18n: &I18n) -> Callout {
    Callout::info(i18n.t(if has_appendix {
        "batch-scope-note-appendix"
    } else {
        "batch-scope-note-json"
    }))
    .with_title(i18n.t("batch-scope-note-title"))
}

/// The batch decision and its reasons, as the single report states its own
/// (`render_rating`). The decision appeared nowhere in the batch PDF,
/// next to "a very good result" on a batch that failed on legal flags
/// (#1039).
pub(super) fn render_batch_rating(
    builder: renderreport::engine::ReportBuilder,
    batch: &BatchReport,
    score: u32,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    use crate::audit::verdict::Verdict;
    use fluent_bundle::FluentValue;
    let summary = &batch.summary;
    let verdict = summary.decision.verdict;
    // The defined band words (Sehr gut / Excellent …); the viewport-card
    // "range" wording read "Excellent range" next to "Sehr guter Bereich"
    // (#1039).
    let rating = crate::registry::FIVE_BAND.label(score as f32, is_english(i18n));
    let verdict_word = i18n.t(match verdict {
        Verdict::Pass => "batch-rating-verdict-pass",
        Verdict::Warn => "batch-rating-verdict-warn",
        Verdict::Fail => "batch-rating-verdict-fail",
    });
    let mut body = i18n.t_args(
        "batch-rating-body",
        &[
            ("rating", FluentValue::from(rating.to_string())),
            ("verdict", FluentValue::from(verdict_word)),
            ("passed", FluentValue::from(summary.passed as i64)),
            ("total", FluentValue::from(summary.total_urls as i64)),
        ],
    );
    let reasons: Vec<String> = crate::audit::verdict::batch_reason_kinds(summary)
        .into_iter()
        .map(|kind| batch_reason_text(kind, i18n))
        .collect();
    if !reasons.is_empty() {
        body.push(' ');
        body.push_str(&i18n.t_args("batch-rating-reasons", &[("reasons", reasons.join(", "))]));
    }
    // A passed batch is already said by the grade card above ("5 of 5
    // passed"); the reasons are what the card cannot carry (#1212).
    // A failed or warned batch states its reasons; the note that the rating is
    // no statement of conformance is a note under a passed one and part of the
    // method otherwise.
    match verdict {
        Verdict::Fail => {
            builder.add_component(Callout::warning(&body).with_title(i18n.t("batch-rating-title")))
        }
        Verdict::Warn => {
            builder.add_component(Callout::info(&body).with_title(i18n.t("batch-rating-title")))
        }
        Verdict::Pass => builder.add_component(
            Label::new(i18n.t("batch-rating-note"))
                .with_size("8.5pt")
                .with_color(crate::output::pdf::design::tokens::MUTED),
        ),
    }
}

/// Reader-facing text of one batch decision reason in the run language.
pub(super) fn batch_reason_text(
    kind: crate::audit::verdict::BatchReasonKind,
    i18n: &I18n,
) -> String {
    use crate::audit::verdict::BatchReasonKind as K;
    use fluent_bundle::FluentValue;
    let n = |v: usize| FluentValue::from(v as i64);
    let s = |v: u32| FluentValue::from(i64::from(v));
    match kind {
        K::NoUrlAudited { attempted } => i18n.count("batch-reason-no-url-audited", attempted),
        K::LegalFlags(count) => i18n.count("batch-reason-legal-flags", count),
        K::BlockingIssues(count) => i18n.count("batch-reason-blocking-issues", count),
        K::TechnicalBlockers(count) => i18n.count("batch-reason-technical-blockers", count),
        K::AverageBelowFailThreshold { score, threshold } => i18n.t_args(
            "batch-reason-average-below-fail",
            &[("score", s(score)), ("threshold", s(threshold))],
        ),
        K::IncompleteScope { audited, requested } => i18n.t_args(
            "batch-reason-incomplete-scope",
            &[("audited", n(audited)), ("requested", n(requested))],
        ),
        K::AuditQualityNotComplete => i18n.t("batch-reason-audit-quality"),
        K::AverageBelowWarnThreshold { score, threshold } => i18n.t_args(
            "batch-reason-average-below-warn",
            &[("score", s(score)), ("threshold", s(threshold))],
        ),
        K::FailedUrls(count) => i18n.count("batch-reason-failed-urls", count),
        K::WarnedUrls(count) => i18n.count("batch-reason-warned-urls", count),
        K::ViolationsFound(count) => i18n.count("batch-reason-violations-found", count),
    }
}

/// At most this many unaudited URLs are listed by name; the count is always
/// complete.
const MAX_LISTED_UNAUDITED_URLS: usize = 20;

/// Fluent arguments for the score-coverage texts: scored, attempted and
/// unaudited URL counts (#651).
pub(super) fn unaudited_args(batch: &BatchReport) -> [(&'static str, i64); 3] {
    let scored = batch.summary.total_urls as i64;
    let failed = batch.errors.len() as i64;
    [
        ("scored", scored),
        ("attempted", scored + failed),
        ("failed", failed),
    ]
}

/// Pages that could not be audited are not in any score — say so next to the
/// scores and name them, instead of letting the batch silently shrink (#651).
pub(super) fn render_batch_unaudited_urls(
    builder: renderreport::engine::ReportBuilder,
    batch: &BatchReport,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    if batch.errors.is_empty() {
        return builder;
    }
    let args = unaudited_args(batch);
    let mut list = List::new().with_title(i18n.t("batch-unaudited-list-title"));
    for error in batch.errors.iter().take(MAX_LISTED_UNAUDITED_URLS) {
        list = list.add_item(short_url(&error.url, 90));
    }
    if batch.errors.len() > MAX_LISTED_UNAUDITED_URLS {
        list = list.add_item(i18n.t_args(
            "batch-unaudited-more",
            &[(
                "count",
                (batch.errors.len() - MAX_LISTED_UNAUDITED_URLS) as i64,
            )],
        ));
    }
    builder
        .add_component(
            Callout::warning(i18n.t_args("batch-unaudited-body", &args))
                .with_title(i18n.t_args("batch-unaudited-title", &args)),
        )
        .add_component(list)
}

/// Domain-wide EN 301 549 clause roll-up for batch reports: one compact
/// table (clause → status → pages affected), never one annex per page (see
/// `wcag::en301549::derive_batch_rollup`). Opt-in only via `--annex en301549`.
pub(super) fn render_batch_en301549_rollup(
    mut builder: renderreport::engine::ReportBuilder,
    rollup: &[crate::wcag::en301549::BatchClauseRollup],
    failed_en_criteria: &std::collections::BTreeSet<String>,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    use crate::wcag::en301549::ClauseStatus;

    let en = is_english(i18n);
    builder = builder.add_component(
        SectionHeaderSplit::new(
            i18n.t("batch-report-en301549-title"),
            i18n.t("batch-report-en301549-intro"),
        )
        .with_level(2),
    );

    let disclaimer = if en {
        crate::wcag::en301549::EN301549_DISCLAIMER_EN
    } else {
        crate::wcag::en301549::EN301549_DISCLAIMER_DE
    };
    builder = builder.add_component(
        Callout::warning(disclaimer).with_title(i18n.t("batch-report-en301549-disclaimer-title")),
    );

    let status_label = |status: ClauseStatus| -> String {
        i18n.t(match status {
            ClauseStatus::ViolationsFound => "batch-report-en301549-status-violations",
            ClauseStatus::NoViolationsAutomated => "batch-report-en301549-status-no-violations",
            ClauseStatus::ManualReviewRequired => "batch-report-en301549-status-manual",
        })
    };

    let mut table = AuditTable::new(vec![
        TableColumn::new(i18n.t("batch-report-en301549-col-clause")).with_width("12%"),
        TableColumn::new(i18n.t("batch-report-en301549-col-title")).with_width("42%"),
        TableColumn::new("Status").with_width("28%"),
        TableColumn::new(i18n.t("batch-report-en301549-col-pages")).with_width("18%"),
    ])
    .with_title(i18n.t_args(
        "batch-report-en301549-table-title",
        &[(
            "count",
            fluent_bundle::FluentValue::from(
                crate::wcag::en301549::EN301549_WEB_CLAUSES.len() as i64
            ),
        )],
    ));
    for r in rollup {
        let clause_title = if en {
            r.clause.title_en
        } else {
            r.clause.title_de
        };
        table = table.add_row(vec![
            r.clause.en_clause.to_string(),
            clause_title.to_string(),
            if !matches!(r.status, ClauseStatus::ViolationsFound)
                && failed_en_criteria.contains(r.clause.wcag)
            {
                i18n.t("batch-report-en301549-incomplete")
            } else {
                status_label(r.status)
            },
            r.affected_pages.to_string(),
        ]);
    }
    builder = builder.add_component(table);

    let mut out_of_scope_list =
        List::new().with_title(i18n.t("batch-report-en301549-out-of-scope-title"));
    for chapter in crate::wcag::en301549::OUT_OF_SCOPE_CHAPTERS {
        let chapter_title = if en {
            chapter.title_en
        } else {
            chapter.title_de
        };
        out_of_scope_list = out_of_scope_list.add_item(i18n.t_args(
            "batch-report-en301549-chapter",
            &[
                ("chapter", chapter.chapter.to_string()),
                ("title", chapter_title.to_string()),
            ],
        ));
    }
    builder.add_component(out_of_scope_list)
}
