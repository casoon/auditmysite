use crate::output::localized::is_english;
use renderreport::components::advanced::{
    ChecklistPanel, ChecklistRow, Grid, KeyValueList, List, PageBreak, SectionHeaderSplit,
    TableOfContents,
};
use renderreport::components::text::{Label, TextBlock};
use renderreport::components::MetricCard;
use renderreport::components::{AuditTable, TableColumn};
use renderreport::prelude::Image;
use renderreport::prelude::*;
use renderreport::Component;

use crate::audit::BatchReport;
use crate::cli::ReportLevel;
use crate::i18n::I18n;
use crate::output::report_model::*;
use crate::util::truncate_url;

use super::super::cover::{
    batch_certificate_label, build_batch_cover_score_row, certificate_badge_path,
    certificate_label_localized,
};
use super::super::helpers::score_quality_color;
use super::actions::build_batch_quick_actions;
use super::management::{
    build_batch_assessment, build_batch_key_points, render_batch_internal_comparison,
    render_batch_management_risks,
};
use super::portfolio::render_batch_impact_summary;

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

    let cover_logo_asset = super::super::cover_logo_asset(config);
    builder = super::super::register_cover_logo_asset(builder, config, cover_logo_asset);

    builder = builder
        .add_component(Image::new(cover_logo_asset).with_width("120pt"))
        .add_component(
            Label::new(i18n.t("batch-cover-eyebrow"))
                .with_size("11pt")
                .bold()
                .with_color("#0f766e"),
        )
        .add_component(
            Label::new(i18n.t("batch-cover-title"))
                .with_size("28pt")
                .bold(),
        )
        .add_component(
            Label::new(i18n.t("batch-cover-kicker"))
                .with_size("12pt")
                .with_color("#475569"),
        );

    // Audit-Rahmen box
    {
        let modules_str = pres.portfolio_summary.active_modules.join(", ");
        let mut cover_meta = KeyValueList::new().with_title(i18n.t("batch-cover-frame-title"));
        cover_meta = cover_meta
            .add(i18n.t("batch-cover-frame-domain"), domain)
            .add(i18n.t("batch-cover-frame-date"), &pres.cover.date)
            .add(
                i18n.t("batch-cover-frame-urls"),
                format!("{}", pres.portfolio_summary.total_urls),
            )
            .add(
                i18n.t("batch-cover-frame-certificate"),
                certificate_label_localized(&pres.portfolio_summary.certificate, i18n.locale()),
            )
            .add(i18n.t("batch-cover-frame-modules"), &modules_str)
            .add(
                i18n.t("batch-cover-frame-version"),
                format!("auditmysite v{}", pres.cover.version),
            );
        if let Some(sample) = &batch.sample {
            let source = i18n.t(&format!("batch-source-{}", sample.source));
            let scope = if sample.is_sample {
                i18n.t_args(
                    "batch-scope-sample",
                    &[
                        ("audited", sample.audited.to_string()),
                        ("total", sample.total_discovered.to_string()),
                        ("source", source),
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
            };
            cover_meta = cover_meta.add(i18n.t("batch-cover-frame-scope"), scope);
        }
        // #653: the display mode every score in this report belongs to.
        {
            let en = is_english(i18n);
            let mode = batch
                .reports
                .first()
                .map(|r| r.accessibility.execution.scope.display_mode)
                .unwrap_or_default();
            let mut value = crate::display::audited_mode_text(mode, en);
            let offering = batch
                .reports
                .iter()
                .filter(|r| {
                    r.accessibility
                        .execution
                        .display_modes
                        .as_ref()
                        .is_some_and(|d| d.offers_display_modes)
                })
                .count();
            if offering > 0 {
                value.push_str(" — ");
                value.push_str(&i18n.t_args(
                    "batch-display-offered",
                    &[
                        ("count", offering as i64),
                        ("total", batch.reports.len() as i64),
                    ],
                ));
            }
            cover_meta = cover_meta.add(i18n.t("batch-cover-frame-display"), value);
        }
        if let Some(note) = crate::audit::exclusion::BatchExclusionSummary::aggregate(
            batch
                .reports
                .iter()
                .filter_map(|r| r.accessibility.execution.exclusions.as_ref()),
        )
        .and_then(|summary| {
            crate::audit::exclusion::batch_exclusion_note(&summary, batch.reports.len(), i18n)
        }) {
            cover_meta = cover_meta.add(i18n.t("exclusion-fact-label"), note);
        }
        if !batch.errors.is_empty() {
            cover_meta = cover_meta.add(
                i18n.t("batch-cover-frame-score-basis"),
                i18n.t_args("batch-score-coverage", &unaudited_args(batch)),
            );
        }
        builder = builder.add_component(cover_meta);
    }

    let batch_badge_asset = "/certificate-badge-batch.svg";
    let batch_badge_enabled =
        if let Ok(path) = certificate_badge_path(batch_certificate_label(score)) {
            builder = builder.asset(batch_badge_asset, path);
            true
        } else {
            false
        };

    builder = builder
        .add_component(build_batch_cover_score_row(
            score,
            &pres.portfolio_summary.grade,
            &pres.portfolio_summary.certificate,
            pres.portfolio_summary.total_urls as u32,
            pres.portfolio_summary.total_violations as u32,
            batch_badge_enabled.then_some(batch_badge_asset),
            i18n,
        )?)
        .add_component(
            TextBlock::new(&pres.portfolio_summary.verdict_text)
                .with_size("11pt")
                .with_line_height("1.4em")
                .with_max_width("100%"),
        )
        .add_component(super::super::output_scope_callout(i18n))
        .add_component(PageBreak::new());

    if config.level != ReportLevel::Executive {
        builder = builder.add_component(TableOfContents::new().with_depth(1));
    }

    Ok(builder)
}

/// At most this many unaudited URLs are listed by name; the count is always
/// complete.
const MAX_LISTED_UNAUDITED_URLS: usize = 20;

/// Fluent arguments for the score-coverage texts: scored, attempted and
/// unaudited URL counts (#651).
fn unaudited_args(batch: &BatchReport) -> [(&'static str, i64); 3] {
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
fn render_batch_unaudited_urls(
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
        list = list.add_item(truncate_url(&error.url, 90));
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

pub(super) fn render_batch_status_section(
    mut builder: renderreport::engine::ReportBuilder,
    batch: &BatchReport,
    pres: &BatchPresentation,
    en301549_rollup: &[crate::wcag::en301549::BatchClauseRollup],
    failed_en_criteria: &std::collections::BTreeSet<String>,
    config: &ReportConfig,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let dist = &pres.portfolio_summary.severity_distribution;

    // Assessment callout
    let assessment = build_batch_assessment(&pres.portfolio_summary, dist, i18n);
    let callout = match pres.portfolio_summary.risk_level.as_str() {
        "Kritisch" | "Hoch" | "Critical" | "High" => {
            Callout::warning(&pres.portfolio_summary.risk_summary).with_title(&assessment)
        }
        "Mittel" | "Medium" => {
            Callout::info(&pres.portfolio_summary.risk_summary).with_title(&assessment)
        }
        _ => Callout::success(&pres.portfolio_summary.risk_summary).with_title(&assessment),
    };
    builder = builder
        .add_component(Section::new(i18n.t("batch-section-status")).with_level(1))
        .add_component(callout);
    builder = render_batch_unaudited_urls(builder, batch, i18n);

    // Score overview cards
    let score = pres.portfolio_summary.average_score.round() as u32;
    builder = builder.add_component(build_batch_overview_grid(
        pres.portfolio_summary.total_urls as u32,
        score,
        pres.portfolio_summary.total_violations as u32,
        (dist.critical + dist.high) as u32,
        pres.portfolio_summary.crawl_links.as_ref().map(|links| {
            (links.broken_internal_links.len() + links.broken_external_links.len()) as u32
        }),
        is_english(i18n),
    ));

    // Key points
    let key_points = build_batch_key_points(pres, dist, i18n);
    let mut kp_list = List::new().with_title(i18n.t("narrative-key-points-title"));
    for point in &key_points {
        kp_list = kp_list.add_item(point);
    }
    builder = builder.add_component(kp_list);

    // Impact summary
    builder = render_batch_impact_summary(builder, pres, i18n);

    // Quick actions
    let actions = build_batch_quick_actions(pres, i18n);
    if !actions.is_empty() {
        let rows: Vec<ChecklistRow> = actions
            .iter()
            .map(|action| ChecklistRow::new(action, "").with_status("warn"))
            .collect();
        builder = builder.add_component(
            ChecklistPanel::new(rows).with_title(i18n.t("narrative-quick-actions-title")),
        );
    }

    builder = render_batch_management_risks(builder, pres, i18n);

    // EN 301 549 clause roll-up — opt-in only (see `--annex en301549`).
    if config.annex == Some(crate::cli::AnnexKind::En301549) {
        builder = render_batch_en301549_rollup(builder, en301549_rollup, failed_en_criteria, i18n);
    }

    builder = render_batch_internal_comparison(builder, pres, i18n);

    builder
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
    let (title, intro) = if en {
        (
            "EN 301 549 clause mapping",
            "Domain-wide roll-up of this batch's automated WCAG 2.1 A/AA findings onto EN 301 549, chapter 9 (Web) clause numbers — the worst status observed across all audited pages, plus how many pages have a confirmed violation.".to_string(),
        )
    } else {
        (
            "EN-301-549-Klauselzuordnung",
            "Domainweite Zusammenfassung der automatisch erkannten WCAG-2.1-A/AA-Befunde dieses Batches je EN-301-549-Klausel (Kapitel 9, Web) — der schlechteste über alle geprüften Seiten beobachtete Status, plus die Anzahl betroffener Seiten.".to_string(),
        )
    };
    builder = builder.add_component(SectionHeaderSplit::new(title, &intro).with_level(2));

    let disclaimer_title = if en {
        "Not an accessibility statement"
    } else {
        "Keine Barrierefreiheitserklärung"
    };
    let disclaimer = if en {
        crate::wcag::en301549::EN301549_DISCLAIMER_EN
    } else {
        crate::wcag::en301549::EN301549_DISCLAIMER_DE
    };
    builder = builder.add_component(Callout::warning(disclaimer).with_title(disclaimer_title));

    let status_label = |status: ClauseStatus| -> &'static str {
        match (status, en) {
            (ClauseStatus::ViolationsFound, true) => "Violations found",
            (ClauseStatus::ViolationsFound, false) => "Verstöße gefunden",
            (ClauseStatus::NoViolationsAutomated, true) => "No violations (automated)",
            (ClauseStatus::NoViolationsAutomated, false) => "Keine Verstöße (automatisch)",
            (ClauseStatus::ManualReviewRequired, true) => "Manual review required",
            (ClauseStatus::ManualReviewRequired, false) => "Manuelle Prüfung erforderlich",
        }
    };

    let mut table = AuditTable::new(vec![
        TableColumn::new(if en { "Clause" } else { "Klausel" }).with_width("12%"),
        TableColumn::new(if en { "Title" } else { "Titel" }).with_width("42%"),
        TableColumn::new("Status").with_width("28%"),
        TableColumn::new(if en {
            "Pages affected"
        } else {
            "Betroffene Seiten"
        })
        .with_width("18%"),
    ])
    .with_title(if en {
        "Clause roll-up (all 50 WCAG 2.1 A/AA clauses)"
    } else {
        "Klausel-Übersicht (alle 50 WCAG-2.1-A/AA-Klauseln)"
    });
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
                if en {
                    "Automated evaluation incomplete".to_string()
                } else {
                    "Automatisierte Prüfung unvollständig".to_string()
                }
            } else {
                status_label(r.status).to_string()
            },
            r.affected_pages.to_string(),
        ]);
    }
    builder = builder.add_component(table);

    let out_of_scope_title = if en {
        "Outside this audit's scope"
    } else {
        "Außerhalb dieses Prüfumfangs"
    };
    let mut out_of_scope_list = List::new().with_title(out_of_scope_title);
    for chapter in crate::wcag::en301549::OUT_OF_SCOPE_CHAPTERS {
        let chapter_title = if en {
            chapter.title_en
        } else {
            chapter.title_de
        };
        out_of_scope_list = out_of_scope_list.add_item(if en {
            format!(
                "Chapter {} \u{2013} {} (not assessed)",
                chapter.chapter, chapter_title
            )
        } else {
            format!(
                "Kapitel {} \u{2013} {} (nicht bewertet)",
                chapter.chapter, chapter_title
            )
        });
    }
    builder.add_component(out_of_scope_list)
}

pub(super) fn build_batch_overview_grid(
    total_urls: u32,
    average_score: u32,
    total_violations: u32,
    critical_and_high: u32,
    broken_internal_links: Option<u32>,
    en: bool,
) -> Grid {
    let mut metrics = vec![
        (
            if en {
                "Accessibility average"
            } else {
                "Barrierefreiheit Ø"
            },
            format!("{average_score} / 100"),
            Some(score_quality_color(average_score)),
        ),
        (
            if en {
                "Sites audited"
            } else {
                "Geprüfte Websites"
            },
            total_urls.to_string(),
            Some("#0f766e"),
        ),
        (
            if en {
                "WCAG occurrences"
            } else {
                "WCAG-Vorkommen"
            },
            total_violations.to_string(),
            Some("#b45309"),
        ),
        (
            if en {
                "Critical + High"
            } else {
                "Kritisch + Hoch"
            },
            critical_and_high.to_string(),
            Some("#dc2626"),
        ),
    ];

    if let Some(count) = broken_internal_links {
        metrics.push((
            "Broken Links",
            count.to_string(),
            Some(if count > 0 { "#dc2626" } else { "#0f766e" }),
        ));
    }

    let mut grid = Grid::new(2);
    for (title, value, accent) in metrics {
        let mut card = MetricCard::new(title, value);
        if let Some(color) = accent {
            card = card.with_accent_color(color);
        }
        grid = grid.add_item(serde_json::json!({
            "type": "metric-card",
            "data": card.to_data()
        }));
    }

    grid
}
