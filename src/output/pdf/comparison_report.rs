use renderreport::components::advanced::{List, PageBreak};
use renderreport::components::text::{Label, TextBlock};
use renderreport::components::{BenchmarkRow, BenchmarkTable, ComparisonModule, ModuleComparison};
use renderreport::prelude::Image;
use renderreport::prelude::*;

use crate::i18n::I18n;
use crate::output::report_model::ReportConfig;

use super::helpers::{create_engine, extract_domain};
use super::{cover_logo_asset, register_cover_logo_asset};

// ─── Comparison Report ───────────────────────────────────────────────────────

/// Generate a competitive comparison PDF report.
pub fn generate_comparison_pdf(
    comparison: &crate::audit::ComparisonReport,
    config: &ReportConfig,
) -> anyhow::Result<Vec<u8>> {
    let engine = create_engine()?;

    let avg_score = if comparison.entries.is_empty() {
        0u32
    } else {
        (comparison
            .entries
            .iter()
            .map(|e| e.overall_score as u64)
            .sum::<u64>()
            / comparison.entries.len() as u64) as u32
    };

    let author = comparison
        .entries
        .first()
        .map(|e| extract_domain(&e.url))
        .unwrap_or_default();

    let i18n = I18n::new(&config.locale)?;
    let en_cmp = i18n.locale() == "en";
    let date_format = if en_cmp { "%Y-%m-%d" } else { "%d.%m.%Y" };
    let mut builder = engine
        .report("wcag-comparison")
        .metadata("date", chrono::Local::now().format(date_format).to_string())
        .metadata("author", &author)
        .metadata("footer_prefix", "Audit:")
        .metadata("footer_link_url", "")
        .metadata(
            "footer_tagline",
            "A technical auditing platform by casoon.de",
        );

    let cover_logo_asset = cover_logo_asset(config);
    builder = register_cover_logo_asset(builder, config, cover_logo_asset);

    let comparison_subline = if en_cmp {
        format!(
            "Comparison of {} domains — avg overall: {}/100",
            comparison.entries.len(),
            avg_score
        )
    } else {
        format!(
            "Vergleich von {} Domains — Ø Gesamt: {}/100",
            comparison.entries.len(),
            avg_score
        )
    };

    builder = builder
        .add_component(Image::new(cover_logo_asset).with_width("22%"))
        .add_component(
            Label::new(&author)
                .with_size("10pt")
                .bold()
                .with_color("#0f766e"),
        )
        .add_component(
            Label::new(i18n.t("comparison-cover-title"))
                .with_size("28pt")
                .bold(),
        )
        .add_component(
            Label::new(comparison_subline)
                .with_size("12pt")
                .with_color("#475569"),
        )
        .add_component(
            Label::new(format!("auditmysite v{}", env!("CARGO_PKG_VERSION")))
                .with_size("9pt")
                .with_color("#94a3b8"),
        )
        .add_component(PageBreak::new());

    // ── 1. Domain-Ranking ────────────────────────────────────────────
    let ranking_intro = if en_cmp {
        format!(
            "Comparison of {} domains based on a full audit of each home page. \
             Average overall score: {}/100.",
            comparison.entries.len(),
            avg_score,
        )
    } else {
        format!(
            "Vergleich von {} Domains anhand eines vollständigen Audits der jeweiligen Startseite. \
             Durchschnittlicher Gesamtscore: {}/100.",
            comparison.entries.len(),
            avg_score,
        )
    };
    builder = builder
        .add_component(Section::new(i18n.t("comparison-domain-ranking")).with_level(1))
        .add_component(TextBlock::new(ranking_intro));

    let rows: Vec<BenchmarkRow> = comparison
        .entries
        .iter()
        .enumerate()
        .map(|(i, e)| {
            let mut row = BenchmarkRow::new(
                (i + 1) as u32,
                &e.domain,
                e.overall_score,
                e.accessibility_score,
                e.critical_violations as u32,
            );
            if let Some(s) = e.seo_score {
                row = row.with_seo(s);
            }
            if let Some(s) = e.performance_score {
                row = row.with_performance(s);
            }
            if let Some(s) = e.security_score {
                row = row.with_security(s);
            }
            row
        })
        .collect();

    builder = builder
        .add_component(BenchmarkTable::new(rows).with_title(&i18n.t("comparison-domain-ranking")));

    // ── 2. Modul-Vergleich ───────────────────────────────────────────
    let has_module_data = comparison
        .entries
        .iter()
        .any(|e| e.seo_score.is_some() || e.performance_score.is_some());

    if has_module_data {
        builder = builder
            .add_component(Section::new(i18n.t("comparison-module-comparison")).with_level(1));

        let comparison_modules: Vec<ComparisonModule> = comparison
            .entries
            .iter()
            .map(|e| ComparisonModule::new(&e.domain, e.overall_score))
            .collect();
        builder = builder.add_component(ModuleComparison::new(comparison_modules));
    }

    // ── 3. Top Findings je Domain ────────────────────────────────────
    let has_issues = comparison.entries.iter().any(|e| !e.top_issues.is_empty());
    if has_issues {
        builder = builder.add_component(
            Section::new(i18n.t("comparison-top-findings-per-domain")).with_level(1),
        );

        let top_findings_label = if en_cmp {
            "Top findings"
        } else {
            "Top Findings"
        };
        for entry in &comparison.entries {
            if entry.top_issues.is_empty() {
                continue;
            }
            builder = builder.add_component(
                Section::new(format!("{} — {}", entry.domain, top_findings_label)).with_level(2),
            );
            let mut list = List::new();
            for issue in &entry.top_issues {
                list = list.add_item(issue.clone());
            }
            builder = builder.add_component(list);
        }
    }

    let built_report = builder.build();
    Ok(engine.render_pdf(&built_report)?)
}
