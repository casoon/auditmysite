use crate::audit::BatchReport;
/// `render` refuses a batch without an average before any section runs.
const UNMEASURED_REFUSED: &str = "batch PDF is refused without a measured average";

use crate::i18n::I18n;
use crate::model::ReportLevel;
use crate::output::builder::build_batch_presentation_with_locale;
use crate::output::report_model::*;

mod actions;
mod appendix;
mod consistency;
mod interactive;
mod management;
mod overview;
mod portfolio;
mod privacy;
mod seo;

use actions::*;
use appendix::*;
use consistency::*;
use interactive::*;
use management::*;
use overview::*;
use portfolio::*;
use privacy::render_batch_privacy;
use seo::*;

pub fn generate_batch_pdf(batch: &BatchReport, config: &ReportConfig) -> anyhow::Result<Vec<u8>> {
    generate_batch_pdf_with_annotations(batch, config, None)
}

/// [`generate_batch_pdf`] with the external assessments of `--annotations`
/// (#961): the batch report lists them in its "Assessments" section.
pub fn generate_batch_pdf_with_annotations(
    batch: &BatchReport,
    config: &ReportConfig,
    annotations: Option<&crate::annotations::Resolved>,
) -> anyhow::Result<Vec<u8>> {
    let (engine, mut built_report) = build_batch_report(batch, config, annotations)?;
    super::sanitize::render_pdf_sanitized(&engine, &mut built_report)
}

/// Render the intermediate Typst source for a batch report (hidden `--debug-typ`).
pub fn generate_batch_typ(batch: &BatchReport, config: &ReportConfig) -> anyhow::Result<String> {
    generate_batch_typ_with_annotations(batch, config, None)
}

pub fn generate_batch_typ_with_annotations(
    batch: &BatchReport,
    config: &ReportConfig,
    annotations: Option<&crate::annotations::Resolved>,
) -> anyhow::Result<String> {
    let (engine, mut built_report) = build_batch_report(batch, config, annotations)?;
    super::sanitize::sanitize_request(&mut built_report, &Default::default());
    Ok(engine.render_typ(&built_report)?)
}

fn build_batch_report(
    batch: &BatchReport,
    config: &ReportConfig,
    annotations: Option<&crate::annotations::Resolved>,
) -> anyhow::Result<(
    std::sync::Arc<renderreport::Engine>,
    renderreport::RenderRequest,
)> {
    crate::audit::validate_batch(batch)?;
    anyhow::ensure!(
        batch.summary.average_score.is_some(),
        "no URL of the batch was measured; there is no score, grade or band to report"
    );
    let engine = super::helpers::create_engine();
    let i18n = I18n::new(&config.locale)?;
    // Use the locale-aware presentation builder — `build_batch_presentation`
    // hardcodes German and would override `--lang` (#406).
    let pres = build_batch_presentation_with_locale(batch, &i18n);

    let domain = &pres.portfolio_summary.domain;
    let score = pres.portfolio_summary.average_overall_score;

    let mut builder = engine
        .report("wcag-batch-audit")
        .title(&pres.cover.title)
        .subtitle(&pres.cover.url)
        .metadata("date", &pres.cover.date)
        .metadata("version", &pres.cover.version)
        .metadata("author", domain)
        .metadata("lang", i18n.locale())
        .metadata("footer_prefix", "Audit:")
        .metadata("footer_link_url", "")
        .metadata(
            "footer_tagline",
            "A technical auditing platform by casoon.de",
        );

    let normalized_reports: Vec<crate::audit::normalized::NormalizedReport> = batch
        .reports
        .iter()
        .map(|r| crate::audit::normalized::normalize(r).normalized)
        .collect();
    let en301549_rollup = crate::wcag::en301549::derive_batch_rollup(
        normalized_reports.iter().map(|r| r.findings.as_slice()),
    );
    let failed_en_criteria: std::collections::BTreeSet<String> = normalized_reports
        .iter()
        .flat_map(|report| report.rule_outcomes.iter())
        .filter(|outcome| crate::wcag::rule_run_errored(outcome))
        .flat_map(|outcome| outcome.wcag.iter().cloned())
        .collect();

    builder = render_batch_cover(builder, batch, &pres, config, score, &i18n)?;
    builder = render_batch_status_section(
        builder,
        batch,
        &normalized_reports,
        &pres,
        &en301549_rollup,
        &failed_en_criteria,
        config,
        &i18n,
    );
    builder = render_batch_module_portfolio(builder, &pres, &normalized_reports, &i18n);
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
    builder = render_batch_commerce(builder, batch, &i18n);
    builder = render_batch_privacy(builder, batch, &i18n);
    builder = render_batch_action_plan_section(builder, &pres, &i18n);
    builder = render_batch_tech_url_matrix(builder, &pres, config, &i18n);
    builder = render_batch_seo_section(builder, &pres, &i18n);

    if let Some(ref consistency) = batch.consistency {
        builder = render_batch_consistency(builder, consistency, &pres, &i18n);
    }

    if let Some(resolved) = annotations {
        builder = super::annotations::render_assessments_section(builder, resolved, true, &i18n);
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
    use crate::model::WcagLevel;
    use crate::wcag::WcagResults;

    fn report(url: &str) -> AuditReport {
        AuditReport::new(url.to_string(), WcagLevel::AA, WcagResults::new(), 100)
    }

    /// #768: a batch in which every URL failed has no score, so there is no
    /// grade or band to print; the PDF refuses instead of showing a 0.
    #[test]
    fn batch_without_a_measured_url_gets_no_pdf() {
        let errors = vec![crate::audit::BatchError {
            url: "https://a.example".to_string(),
            error: "navigation failed".to_string(),
            blocked_reason: None,
        }];
        let batch = BatchReport::from_reports(vec![], errors, 10);
        let err = generate_batch_typ(&batch, &ReportConfig::default())
            .expect_err("nothing measured, nothing to render");
        assert!(
            err.to_string().contains("no URL of the batch was measured"),
            "{err}"
        );
    }

    /// A page with one high-severity Level AA contrast violation: one legal
    /// flag, so the page and the batch fail.
    fn contrast_page(url: &str) -> AuditReport {
        let mut results = WcagResults::new();
        results.add_violation(
            crate::wcag::Violation::new(
                "1.4.3",
                "Contrast (Minimum)",
                WcagLevel::AA,
                crate::wcag::Severity::High,
                "Insufficient contrast",
                "node-1",
            )
            .with_rule_id("contrast/text-insufficient")
            .with_selector("span.separator"),
        );
        AuditReport::new(url.to_string(), WcagLevel::AA, results, 100)
    }

    fn failing_batch() -> BatchReport {
        BatchReport::from_reports(
            ["a", "b", "c"]
                .iter()
                .map(|p| contrast_page(&format!("https://example.com/{p}")))
                .collect(),
            vec![],
            100,
        )
    }

    fn typ(batch: &BatchReport, locale: &str) -> String {
        generate_batch_typ(
            batch,
            &ReportConfig {
                locale: locale.to_string(),
                ..ReportConfig::default()
            },
        )
        .expect("batch Typst source")
        .replace("\\\"", "\"")
    }

    /// #1039: the batch decision and its reason are in the report, and no
    /// cover or status text reads as a pass next to it.
    #[test]
    fn failing_batch_states_its_verdict_and_reason() {
        use crate::audit::verdict::{batch_reason_kinds, BatchReasonKind, Verdict};
        let batch = failing_batch();
        assert_eq!(batch.summary.decision.verdict, Verdict::Fail);
        assert_eq!(batch.summary.passed, 0);
        let legal_flags = batch.summary.legal_flags;
        assert!(legal_flags > 0);
        let kinds = batch_reason_kinds(&batch.summary);
        assert_eq!(kinds, vec![BatchReasonKind::LegalFlags(legal_flags)]);
        // The band sentence is gated like a single page's (#356).
        assert_eq!(batch.summary.score_band_key, "verdict-batch-deficient");

        for locale in ["en", "de"] {
            let i18n = I18n::new(locale).unwrap();
            let en = locale == "en";
            let out = typ(&batch, locale);
            assert!(
                out.contains(&i18n.t("batch-rating-verdict-fail")),
                "{locale}"
            );
            assert!(
                out.contains(&overview::batch_reason_text(kinds[0], &i18n)),
                "{locale}: reason missing"
            );
            let score = batch.summary.average_score.unwrap().round() as u32;
            assert!(
                out.contains(&super::super::cover::cover_band_phrase(
                    score,
                    Verdict::Fail,
                    false,
                    en
                )),
                "{locale}: cover does not name the failed check"
            );
            // The legal key point and the impact row follow the legal flags,
            // not critical findings alone.
            assert!(!out.contains("No Level A/AA violations"), "{locale}");
            assert!(!out.contains("Keine Level-A/AA-Verstöße"), "{locale}");
            assert!(!out.contains("No critical violations detected automatically"));
        }
    }

    /// #1039: the management risk view shows the levels and rationales the
    /// JSON publishes (`summary.management_risks`), from one derivation.
    #[test]
    fn management_risk_view_matches_the_json_derivation() {
        use crate::audit::management_risk::RiskTier;
        let batch = failing_batch();
        let normalized: Vec<_> = batch
            .reports
            .iter()
            .map(|r| crate::audit::normalized::normalize(r).normalized)
            .collect();
        let kinds = crate::audit::management_risk::build_management_risk_kinds(&normalized);
        let i18n = I18n::new("en").unwrap();
        let out = typ(&batch, "en");
        for kind in kinds.iter().filter(|k| k.tier() != RiskTier::Unknown) {
            let level = i18n.t(match kind.tier() {
                RiskTier::High => "risk-level-high",
                RiskTier::Medium => "risk-level-medium",
                _ => "risk-level-low",
            });
            let value = format!("{level} — {}", kind.rationale(true));
            assert!(
                out.contains(&format!(
                    "key: \"{}\", value: \"{value}\"",
                    kind.dimension(true)
                )),
                "missing {value}"
            );
        }
    }

    /// #1039: a module row claims a clean state only when the data shows one.
    #[test]
    fn module_effect_follows_open_findings_and_spread() {
        let i18n = I18n::new("en").unwrap();
        // High average, a high finding open: no "clean" claim.
        assert_eq!(
            portfolio::module_effect_sentence("Security", 95, Some(95), 5, &i18n),
            i18n.count("batch-effect-open-high", 5)
        );
        // High average, one page below 90.
        assert_eq!(
            portfolio::module_effect_sentence("HTML Conformance", 96, Some(88), 0, &i18n),
            i18n.t_args("batch-effect-uneven", &[("min", 88i64)])
        );
        // Every page at 100, nothing open.
        assert_eq!(
            portfolio::module_effect_sentence("Best Practices", 100, Some(100), 0, &i18n),
            i18n.t("batch-effect-other-good")
        );
        // The same 85 on every page is no variance.
        assert_eq!(
            portfolio::module_effect_sentence("Performance", 85, Some(85), 0, &i18n),
            i18n.t("batch-effect-performance-weak")
        );
        let batch = failing_batch();
        let normalized: Vec<_> = batch
            .reports
            .iter()
            .map(|r| crate::audit::normalized::normalize(r).normalized)
            .collect();
        assert_eq!(
            portfolio::module_high_findings(&normalized, "Accessibility"),
            3
        );
        assert_eq!(portfolio::module_high_findings(&normalized, "Security"), 0);
    }

    /// #1039: the cover counts WCAG occurrences and says so; the scope note
    /// names an appendix only when the report has one; the manual-review
    /// share is the criteria count the JSON reports.
    #[test]
    fn batch_texts_name_what_their_numbers_are() {
        let batch = failing_batch();
        let i18n = I18n::new("en").unwrap();
        let out = typ(&batch, "en");
        assert!(out.contains(&i18n.count(
            "cover-card-wcag-occurrences",
            batch.summary.total_violations
        )));
        assert!(out.contains(&i18n.t("batch-scope-note-json")));
        assert!(!out.contains(&i18n.t("batch-scope-note-appendix")));
        let (automated, total) = crate::wcag::coverage::coverage_stats();
        assert!(out.contains(&i18n.t_args(
            "batch-next-steps-manual",
            &[("automated", automated as i64), ("total", total as i64)],
        )));
        // The URL matrix column holds the accessibility score.
        assert!(out.contains(&format!(
            "header: \"{}\"",
            i18n.t("batch-matrix-col-accessibility")
        )));
    }

    /// A failing page with SEO data: 1392 words make the profile attribute
    /// "text-rich" and need a thousands separator.
    fn content_page(url: &str) -> AuditReport {
        let mut seo = crate::seo::SeoAnalysis::default();
        seo.technical.word_count = 1392;
        seo.content_profile = Some(crate::seo::build_content_profile(&seo, "en"));
        contrast_page(url).with_seo(seo)
    }

    /// #1039: numbers, module names, profile attributes and the ranking
    /// headers auditmysite supplies follow the run language.
    #[test]
    fn batch_pdf_writes_the_run_language() {
        let batch = BatchReport::from_reports(
            ["a", "b", "c"]
                .iter()
                .map(|p| content_page(&format!("https://example.com/{p}")))
                .collect(),
            vec![],
            100,
        );
        let seo = batch.reports[0].discoverability.seo.as_ref().unwrap();
        for (locale, words, foreign_words) in [("en", "1,392", "1.392"), ("de", "1.392", "1,392")] {
            let en = locale == "en";
            let i18n = I18n::new(locale).unwrap();
            let out = typ(&batch, locale);
            assert!(out.contains(&format!("\"{words}\"")), "{locale}");
            assert!(!out.contains(&format!("\"{foreign_words}\"")), "{locale}");
            // Attributes in the run language and not cut mid-word.
            let attributes = crate::seo::build_content_profile(seo, locale)
                .page_classification
                .attributes
                .join(", ");
            assert!(out.contains(&format!("\"{attributes}\"")), "{locale}");
            // renderreport's built-in German table title is gone; the extra
            // ranking columns carry the run language's labels.
            assert!(!out.contains("title: \"Website-Ranking\""), "{locale}");
            assert!(out.contains(&format!("label: \"{}\"", i18n.t("batch-ranking-col-seo"))));
            assert!(!en || !out.contains("label: \"Konf.\""));
            for key in [
                "batch-col-links-to",
                "batch-col-links-from",
                "batch-col-score",
            ] {
                assert!(
                    out.contains(&format!("header: \"{}\"", i18n.t(key))),
                    "{key}"
                );
            }
            // One module name throughout: the module comparison and the
            // cover frame use the localized one.
            let accessibility = portfolio::localized_module_name("Accessibility", en);
            assert!(
                out.contains(&format!("((\"{accessibility}\", ")),
                "{locale}"
            );
            if !en {
                assert!(!out.contains("(\"Accessibility\", "));
                assert!(!out.contains("value: \"Accessibility"));
            }
        }
    }

    /// #1039: counts select singular and plural.
    #[test]
    fn batch_counts_agree_with_their_nouns() {
        let en = I18n::new("en").unwrap();
        let de = I18n::new("de").unwrap();
        assert_eq!(en.count("batch-meta-occurrences", 1), "1 occurrence");
        assert_eq!(en.count("batch-meta-occurrences", 7), "7 occurrences");
        assert_eq!(
            crate::i18n::count_text("batch-topic-pages", 1, true),
            "Present on 1 audited page"
        );
        assert_eq!(
            crate::i18n::count_text("batch-topic-pages", 1, false),
            "Auf 1 geprüften Seite vertreten"
        );
        let summary = crate::output::report_model::InteractiveJourneySummary {
            total_pages_tested: 5,
            pages_with_issues: 1,
            categories: vec![],
            has_critical: false,
        };
        assert_eq!(
            interactive::interactive_callout_text(&summary, &en),
            "1 of 5 tested pages has interactive issues — 4 pages without issues."
        );
        assert_eq!(
            interactive::interactive_callout_text(&summary, &de),
            "1 von 5 geprüften Seiten hat interaktive Befunde — 4 Seiten ohne Befunde."
        );
        assert_eq!(
            en.t_args(
                "batch-module-spread-constant",
                &[("value", 100i64), ("pages", 1i64)]
            ),
            "100 on the 1 measured page"
        );
    }

    /// #1039: "some pages" was printed when every page carried the level.
    #[test]
    fn risk_summary_counts_the_pages_at_the_level() {
        let batch = failing_batch();
        let i18n = I18n::new("en").unwrap();
        let pres = build_batch_presentation_with_locale(&batch, &i18n);
        let normalized: Vec<_> = batch
            .reports
            .iter()
            .map(|r| crate::audit::normalized::normalize(r).normalized)
            .collect();
        let level = crate::audit::compute_worst_risk(&normalized);
        let at_level = normalized.iter().filter(|r| r.risk.level >= level).count();
        let key = match level {
            crate::audit::normalized::RiskLevel::Low => return,
            crate::audit::normalized::RiskLevel::Medium => "batch-risk-summary-medium",
            crate::audit::normalized::RiskLevel::High => "batch-risk-summary-high",
            _ => "batch-risk-summary-critical",
        };
        assert_eq!(
            pres.portfolio_summary.risk_summary,
            i18n.t_args(key, &[("count", at_level as i64), ("total", 3i64)])
        );
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
