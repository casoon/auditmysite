use crate::audit::BatchReport;
use crate::cli::ReportLevel;
use crate::i18n::I18n;
use crate::output::builder::build_batch_presentation_with_locale;
use crate::output::report_model::*;

mod sections;

use sections::*;

pub fn generate_batch_pdf(batch: &BatchReport, config: &ReportConfig) -> anyhow::Result<Vec<u8>> {
    let (engine, built_report) = build_batch_report(batch, config)?;
    Ok(engine.render_pdf(&built_report)?)
}

/// Render the intermediate Typst source for a batch report (hidden `--debug-typ`).
pub fn generate_batch_typ(batch: &BatchReport, config: &ReportConfig) -> anyhow::Result<String> {
    let (engine, built_report) = build_batch_report(batch, config)?;
    Ok(engine.render_typ(&built_report)?)
}

fn build_batch_report(
    batch: &BatchReport,
    config: &ReportConfig,
) -> anyhow::Result<(renderreport::Engine, renderreport::RenderRequest)> {
    let engine = super::helpers::create_engine()?;
    let i18n = I18n::new(&config.locale)?;
    // Use the locale-aware presentation builder — `build_batch_presentation`
    // hardcodes German and would override `--lang` (#406).
    let pres = build_batch_presentation_with_locale(batch, &i18n);

    let domain = &pres.portfolio_summary.domain;
    let score = pres.portfolio_summary.average_score.round() as u32;

    let mut builder = engine
        .report("wcag-batch-audit")
        .title(&pres.cover.title)
        .subtitle(&pres.cover.url)
        .metadata("date", &pres.cover.date)
        .metadata("version", &pres.cover.version)
        .metadata("author", domain)
        .metadata("footer_prefix", "Audit:")
        .metadata("footer_link_url", "")
        .metadata(
            "footer_tagline",
            "A technical auditing platform by casoon.de",
        );

    let normalized_reports: Vec<crate::audit::normalized::NormalizedReport> = batch
        .reports
        .iter()
        .map(|r| crate::audit::normalize(r).normalized)
        .collect();
    let en301549_rollup = crate::wcag::en301549::derive_batch_rollup(
        normalized_reports.iter().map(|r| r.findings.as_slice()),
    );

    builder = render_batch_cover(builder, batch, &pres, config, score, &i18n)?;
    builder = render_batch_status_section(builder, &pres, &en301549_rollup, config, &i18n);
    builder = render_batch_module_portfolio(builder, &pres, &i18n);
    builder = render_batch_audit_flags(builder, batch, &i18n);
    builder = render_batch_url_ranking(builder, &pres, &i18n);

    if let Some(ref interactive) = pres.interactive_summary {
        if interactive.total_pages_tested > 0 {
            builder = render_batch_interactive_summary(
                builder,
                interactive,
                pres.portfolio_summary.total_urls,
                &i18n,
            );
        }
    }

    builder = render_batch_top_issues(builder, &pres, &i18n);
    builder = render_batch_action_plan_section(builder, &pres, &i18n);
    builder = render_batch_tech_url_matrix(builder, &pres, config, &i18n);
    builder = render_batch_seo_section(builder, &pres, &i18n);

    if let Some(ref consistency) = batch.consistency {
        builder = render_batch_consistency(builder, consistency, &i18n);
    }

    builder = render_next_steps_batch(builder, &pres, &i18n);

    if config.level == ReportLevel::Technical && !pres.appendix.per_url.is_empty() {
        builder = render_batch_appendix(builder, &pres, &i18n);
    }

    let built_report = builder.build();
    Ok((engine, built_report))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::{AuditReport, ViewportScoreSet, ViewportScores};
    use crate::cli::WcagLevel;
    use crate::wcag::WcagResults;

    fn report(url: &str) -> AuditReport {
        AuditReport::new(url.to_string(), WcagLevel::AA, WcagResults::new(), 100)
    }

    #[test]
    fn audit_flag_aggregation_counts_each_kind_once_per_page() {
        let mut first = report("https://example.com/a");
        first.consent_banner_detected = true;
        first.viewport_scores = Some(ViewportScores {
            desktop: ViewportScoreSet {
                accessibility: 95,
                performance: None,
                overall: 95,
            },
            mobile: ViewportScoreSet {
                accessibility: 60,
                performance: None,
                overall: 60,
            },
            weighted_overall: 71,
        });

        let mut second = report("https://example.com/b");
        second.consent_banner_detected = true;

        let summaries = aggregate_audit_flags(&[first, second]);

        let consent = summaries
            .iter()
            .find(|summary| summary.kind == "consent_banner")
            .expect("consent banner summary");
        assert_eq!(consent.affected_pages, 2);

        let viewport = summaries
            .iter()
            .find(|summary| summary.kind == "viewport_gap")
            .expect("viewport gap summary");
        assert_eq!(viewport.affected_pages, 1);
    }
}
