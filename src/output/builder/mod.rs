//! Report builder — transforms raw audit data into ViewModels
//!
//! This module takes raw AuditReport / BatchReport data and produces
//! structured ViewModels with grouped findings, aggregated statistics,
//! and pre-computed presentation data. The renderer does zero data transformation.

mod actions;
mod batch;
mod helpers;
mod modules;
mod seo;
mod single;

pub use batch::build_batch_presentation;
pub use single::build_view_model;

#[cfg(test)]
#[allow(
    clippy::items_after_test_module,
    clippy::too_many_arguments,
    clippy::field_reassign_with_default
)]
mod tests {
    use super::*;
    use crate::audit::{normalize, AuditReport, BatchReport};
    use crate::cli::{ReportLevel, WcagLevel};
    use crate::output::report_model::ReportConfig;
    use crate::performance::{PerformanceGrade, PerformanceScore, WebVitals};
    use crate::seo::technical::TechnicalSeo;
    use crate::seo::SeoAnalysis;
    use crate::seo::{HeadingStructure, MetaTags};
    use crate::wcag::{Violation, WcagResults};

    const NBSP: &str = "\u{00A0}";

    #[test]
    fn test_view_model_uses_accessibility_score_as_primary_score() {
        let mut wcag = WcagResults::new();
        wcag.add_violation(Violation::new(
            "1.1.1",
            "Non-text Content",
            WcagLevel::A,
            crate::taxonomy::Severity::High,
            "Missing alt text",
            "n1",
        ));

        let report = AuditReport::new("https://example.com".into(), WcagLevel::AA, wcag, 1500)
            .with_performance(crate::audit::PerformanceResults {
                vitals: WebVitals::default(),
                score: PerformanceScore {
                    overall: 60,
                    grade: PerformanceGrade::NeedsImprovement,
                    lcp_score: Some(15),
                    fcp_score: Some(15),
                    cls_score: Some(15),
                    interactivity_score: Some(15),
                    metrics_available: 4,
                },
                render_blocking: None,
                content_weight: None,
                third_party: None,
                critical_chain: None,
                minification: None,
                animations: None,
                coverage: None,
            })
            .with_seo(SeoAnalysis::default());

        let normalized = normalize(&report);
        let vm = build_view_model(&normalized, &ReportConfig::default());

        assert_eq!(vm.summary.score, normalized.score);
        assert_eq!(vm.summary.grade, normalized.grade);
        assert_eq!(vm.summary.certificate, normalized.certificate);
        assert!(vm
            .summary
            .metrics
            .iter()
            .any(|m| m.title == format!("Gesamtscore{NBSP}Website")));
        assert!(!vm.executive.key_points.is_empty());
        assert!(!vm.executive.risk_title.is_empty());
    }

    #[test]
    fn test_batch_presentation_includes_topics_and_overlap() {
        let reports = vec![
            make_topic_report(
                "https://example.com/cloud-entwicklung/",
                "Container Deployment Plattform Architektur",
                "Container Deployment fuer Plattformen und Kubernetes Betrieb.",
                &["Container Deployment", "Plattform Architektur"],
                "Container Deployment Kubernetes Plattform Architektur Betrieb",
                72.0,
            ),
            make_topic_report(
                "https://example.com/cloud-migration/",
                "Container Deployment Migration Plattform",
                "Container Deployment fuer Migration und Plattform Betrieb.",
                &["Container Deployment", "Migration Plattform"],
                "Container Deployment Migration Plattform Betrieb",
                68.0,
            ),
        ];

        let batch = BatchReport::from_reports(reports, vec![], 1200);
        let pres = build_batch_presentation(&batch);

        assert!(!pres.portfolio_summary.top_topics.is_empty());
        assert!(pres
            .portfolio_summary
            .top_topics
            .iter()
            .any(|(topic, _)| topic == "container" || topic == "deployment"));
        assert!(!pres.portfolio_summary.overlap_pairs.is_empty());
        assert!(pres
            .url_details
            .iter()
            .all(|detail| !detail.topic_terms.is_empty()));
    }

    #[test]
    fn test_batch_presentation_filters_generic_topic_tokens() {
        let report = make_topic_report(
            "https://example.com/arbeitsweise/",
            "Klare Arbeitsweise fuer digitale Projekte",
            "Willkommen. Drei Schritte fuer transparente Zusammenarbeit.",
            &["Klare Arbeitsweise", "Drei Schritte"],
            "Willkommen transparente Zusammenarbeit drei Schritte fuer Projekte",
            71.0,
        );
        let batch = BatchReport::from_reports(vec![report], vec![], 800);
        let pres = build_batch_presentation(&batch);
        let terms = &pres.url_details[0].topic_terms;

        assert!(!terms.iter().any(|term| term == "fuer" || term == "drei"));
    }

    #[test]
    fn test_batch_presentation_populates_ranking_and_matrix_inputs() {
        let first = make_topic_report_with_modules(
            "https://example.com/arbeitsweise/",
            "Container Deployment Plattform Architektur",
            "Container Deployment fuer Plattformen und Kubernetes Betrieb.",
            &["Container Deployment", "Plattform Architektur"],
            "Container Deployment Kubernetes Plattform Architektur Betrieb",
            72.0,
            91,
            63,
            95,
        );

        let second = make_topic_report_with_modules(
            "https://example.com/datenschutz/",
            "Datenschutz und DSGVO Grundlagen",
            "Datenschutz Hinweise fuer Website und DSGVO Prozesse.",
            &["Datenschutz", "DSGVO Grundlagen"],
            "Datenschutz DSGVO Website Prozesse Hinweise Rechtsgrundlagen",
            68.0,
            88,
            57,
            93,
        );

        let batch = BatchReport::from_reports(vec![first, second], vec![], 1400);
        let pres = build_batch_presentation(&batch);

        assert_eq!(pres.url_details.len(), 2);
        assert!(pres
            .url_details
            .iter()
            .all(|detail| !detail.topic_terms.is_empty()));
        assert!(pres.url_details.iter().all(|detail| detail
            .module_scores
            .iter()
            .any(|(module, _)| module == "SEO")));
        assert!(pres.url_details.iter().all(|detail| detail
            .module_scores
            .iter()
            .any(|(module, _)| module == "Performance")));
        assert!(pres.url_details.iter().all(|detail| detail
            .module_scores
            .iter()
            .any(|(module, _)| module == "Security")));
        assert!(pres
            .portfolio_summary
            .top_topics
            .iter()
            .any(|(topic, _)| topic == "container" || topic == "datenschutz"));
    }

    #[test]
    fn test_batch_presentation_uses_normalized_module_scores() {
        let report = make_topic_report_with_modules(
            "https://example.com/arbeitsweise/",
            "Container Deployment Plattform Architektur",
            "Container Deployment fuer Plattformen und Kubernetes Betrieb.",
            &["Container Deployment", "Plattform Architektur"],
            "Container Deployment Kubernetes Plattform Architektur Betrieb",
            72.0,
            91,
            63,
            95,
        )
        .with_ux(crate::ux::analyze_ux(&crate::AXTree::new()))
        .with_journey(crate::journey::analyze_journey(&crate::AXTree::new()));

        let batch = BatchReport::from_reports(vec![report.clone()], vec![], 1400);
        let pres = build_batch_presentation(&batch);
        let normalized = normalize(&report);

        assert_eq!(pres.url_ranking[0].score as u32, normalized.score);
        assert_eq!(pres.url_ranking[0].grade, normalized.grade);
        assert_eq!(
            pres.url_details[0].module_scores.len(),
            normalized.module_scores.len()
        );
        assert!(pres.url_details[0]
            .module_scores
            .iter()
            .any(|(module, _)| module == "UX"));
        assert!(pres.url_details[0]
            .module_scores
            .iter()
            .any(|(module, _)| module == "Journey"));
    }

    fn make_topic_report_with_modules(
        url: &str,
        title: &str,
        description: &str,
        headings: &[&str],
        text_excerpt: &str,
        score: f32,
        seo_score: u32,
        performance_score: u32,
        security_score: u32,
    ) -> AuditReport {
        make_topic_report(url, title, description, headings, text_excerpt, score)
            .with_performance(crate::audit::PerformanceResults {
                vitals: WebVitals::default(),
                score: PerformanceScore {
                    overall: performance_score,
                    grade: PerformanceGrade::NeedsImprovement,
                    lcp_score: Some(15),
                    fcp_score: Some(15),
                    cls_score: Some(15),
                    interactivity_score: Some(15),
                    metrics_available: 4,
                },
                render_blocking: None,
                content_weight: None,
                third_party: None,
                critical_chain: None,
                minification: None,
                animations: None,
                coverage: None,
            })
            .with_security(crate::security::SecurityAnalysis {
                score: security_score,
                grade: "A".to_string(),
                headers: crate::security::SecurityHeaders {
                    content_security_policy: Some("default-src 'self'".to_string()),
                    x_frame_options: Some("DENY".to_string()),
                    x_content_type_options: Some("nosniff".to_string()),
                    referrer_policy: Some("strict-origin-when-cross-origin".to_string()),
                    permissions_policy: None,
                    strict_transport_security: Some(
                        "max-age=31536000; includeSubDomains".to_string(),
                    ),
                    cross_origin_opener_policy: None,
                    cross_origin_resource_policy: None,
                },
                ssl: crate::security::SslInfo {
                    https: true,
                    valid_certificate: true,
                    has_hsts: true,
                    hsts_max_age: Some(31536000),
                    hsts_include_subdomains: true,
                    hsts_preload: false,
                },
                issues: vec![],
                recommendations: vec![],
                protection: Default::default(),
            })
            .with_seo({
                let mut seo = SeoAnalysis::default();
                seo.meta = MetaTags {
                    title: Some(title.to_string()),
                    description: Some(description.to_string()),
                    keywords: None,
                    robots: None,
                    author: None,
                    viewport: Some("width=device-width, initial-scale=1".to_string()),
                    charset: Some("utf-8".to_string()),
                    canonical: Some(url.to_string()),
                    lang: Some("de".to_string()),
                };
                let mut heading_structure = HeadingStructure::default();
                heading_structure.h1_count = 1;
                heading_structure.h1_text = headings.first().map(|value| (*value).to_string());
                heading_structure.total_count = headings.len();
                seo.headings = heading_structure;
                seo.technical = TechnicalSeo {
                    https: true,
                    has_canonical: true,
                    canonical_url: Some(url.to_string()),
                    has_lang: true,
                    lang: Some("de".to_string()),
                    has_robots_meta: false,
                    robots_meta: None,
                    has_hreflang: false,
                    hreflang: vec![],
                    word_count: 650,
                    internal_links: 12,
                    external_links: 1,
                    dofollow_links: 12,
                    nofollow_links: 1,
                    internal_link_targets: vec![],
                    broken_links: vec![],
                    text_excerpt: text_excerpt.to_string(),
                    uses_remote_google_fonts: false,
                    google_fonts_sources: vec![],
                    tracking_cookies: vec![],
                    tracking_signals: vec![],
                    zaraz: crate::seo::technical::ZarazDetection::default(),
                    has_favicon: true,
                    www_redirect: None,
                    issues: vec![],
                };
                seo.content_profile = Some(crate::seo::build_content_profile(&seo));
                seo.score = seo_score;
                seo
            })
    }

    #[test]
    fn test_all_violation_criteria_reach_viewmodel_findings() {
        // Every criterion from the input must appear in all_findings — no silent drop in the builder.
        let criteria = ["1.1.1", "1.4.3", "2.4.4", "1.3.1", "4.1.2"];
        let mut wcag = WcagResults::new();
        for (i, &criterion) in criteria.iter().enumerate() {
            wcag.add_violation(Violation::new(
                criterion,
                "Test Rule",
                WcagLevel::AA,
                crate::taxonomy::Severity::High,
                "Test violation",
                format!("node-{i}"),
            ));
        }

        let report = AuditReport::new("https://example.com".into(), WcagLevel::AA, wcag, 1500);
        let normalized = normalize(&report);
        let config = ReportConfig {
            level: ReportLevel::Technical,
            ..ReportConfig::default()
        };
        let vm = build_view_model(&normalized, &config);

        let found: Vec<&str> = vm
            .findings
            .all_findings
            .iter()
            .map(|f| f.wcag_criterion.as_str())
            .collect();

        for &criterion in &criteria {
            assert!(
                found.iter().any(|c| c.starts_with(criterion)),
                "Criterion {criterion} missing from ViewModel findings; present: {found:?}"
            );
        }
    }

    #[test]
    fn test_module_scores_in_viewmodel_match_normalized() {
        // Module scores in NormalizedReport must all appear in vm.summary.metrics.
        // Performance, SEO, Security, UX each produce a dashboard card.
        // (Journey is intentionally absent from the dashboard — it has its own detail section.)
        let report = make_topic_report_with_modules(
            "https://example.com/",
            "Test Page",
            "Test description.",
            &["Test Heading"],
            "test content",
            72.0,
            88,
            65,
            90,
        )
        .with_ux(crate::ux::analyze_ux(&crate::AXTree::new()));

        let normalized = normalize(&report);
        let config = ReportConfig {
            locale: "en".to_string(),
            ..ReportConfig::default()
        };
        let vm = build_view_model(&normalized, &config);

        // Each non-Journey module must have a matching dashboard card.
        let dashboard_modules = ["Accessibility", "Performance", "SEO", "Security", "UX"];
        for &expected in &dashboard_modules {
            assert!(
                vm.modules
                    .dashboard
                    .iter()
                    .any(|m| m.name.contains(expected)),
                "Module '{expected}' missing from vm.modules.dashboard"
            );
        }
        assert_eq!(
            vm.modules.dashboard.len(),
            dashboard_modules.len(),
            "Dashboard card count mismatch"
        );
    }

    fn make_topic_report(
        url: &str,
        title: &str,
        description: &str,
        headings: &[&str],
        text_excerpt: &str,
        score: f32,
    ) -> AuditReport {
        let mut seo = SeoAnalysis::default();
        seo.meta = MetaTags {
            title: Some(title.to_string()),
            description: Some(description.to_string()),
            keywords: None,
            robots: None,
            author: None,
            viewport: Some("width=device-width, initial-scale=1".to_string()),
            charset: Some("utf-8".to_string()),
            canonical: None,
            lang: Some("de".to_string()),
        };
        let mut heading_structure = HeadingStructure::default();
        heading_structure.h1_count = 1;
        heading_structure.h1_text = headings.first().map(|value| (*value).to_string());
        heading_structure.total_count = headings.len();
        seo.headings = heading_structure;
        seo.technical = TechnicalSeo {
            https: true,
            has_canonical: true,
            canonical_url: Some(url.to_string()),
            has_lang: true,
            lang: Some("de".to_string()),
            has_robots_meta: false,
            robots_meta: None,
            has_hreflang: false,
            hreflang: vec![],
            word_count: 650,
            internal_links: 12,
            external_links: 1,
            dofollow_links: 12,
            nofollow_links: 1,
            internal_link_targets: vec![],
            broken_links: vec![],
            text_excerpt: text_excerpt.to_string(),
            uses_remote_google_fonts: false,
            google_fonts_sources: vec![],
            tracking_cookies: vec![],
            tracking_signals: vec![],
            zaraz: crate::seo::technical::ZarazDetection::default(),
            has_favicon: true,
            www_redirect: None,
            issues: vec![],
        };
        seo.content_profile = Some(crate::seo::build_content_profile(&seo));
        seo.score = 92;

        let mut report = AuditReport::new(url.to_string(), WcagLevel::AA, WcagResults::new(), 1500)
            .with_seo(seo);
        report.score = score;
        report
    }

    /// Invariants for `interpret_score` wording (CLAUDE.md "Report Wording Style"):
    ///   1. Localization actually happens — `en` must never fall back to the `de`
    ///      sentence (the bug that produced "die accessibility ist …").
    ///   2. Each module/band carries the controlled band label in the right
    ///      locale — locks the vocabulary and keeps the school grade
    ///      "Befriedigend" from creeping back. Asserts the label as a categorical
    ///      *value*, not the free-form prose (mirrors `test_grade_matches_score`).
    #[test]
    fn test_interpret_score_localized_and_label_contract() {
        use helpers::InterpretArea;

        let areas = [
            InterpretArea::Accessibility,
            InterpretArea::Performance,
            InterpretArea::Security,
            InterpretArea::Mobile,
            InterpretArea::Ux,
            InterpretArea::Journey,
        ];
        // (representative score, de label, en label) — one per grade band.
        let bands = [
            (95.0_f32, "Sehr gut", "Excellent"),
            (80.0, "Gut", "Good"),
            (65.0, "Verbesserungswürdig", "Needs improvement"),
            (50.0, "Ausbaufähig", "Inadequate"),
            (20.0, "Kritisch", "Critical"),
        ];

        for area in areas {
            for (score, de_label, en_label) in bands {
                let de = helpers::interpret_score(area, score, "de");
                let en = helpers::interpret_score(area, score, "en");

                assert_ne!(
                    de, en,
                    "de/en interpretation identical at score {score} — en fell back to de"
                );
                assert!(
                    de.starts_with(&format!("{de_label} \u{2014}")),
                    "DE band label wrong at score {score}: {de:?}"
                );
                assert!(
                    en.starts_with(&format!("{en_label} \u{2014}")),
                    "EN band label wrong at score {score}: {en:?}"
                );
                assert!(
                    !de.contains("Befriedigend"),
                    "banned school grade reappeared: {de:?}"
                );
            }
        }
    }

    /// Export every computed module *interpretation* / explanation text into a
    /// single data file for review (issue: report wording quality).
    ///
    /// Ignored by default — run on demand to (re)generate the file:
    ///   cargo test -p auditmysite export_all_interpretations -- --ignored --nocapture
    ///
    /// It exercises the interpretation generators across their full input space:
    ///   * `interpret_score` — every module area × all 5 grade bands × {de,en}
    ///   * `build_seo_interpretation` — 5 score bands × {de,en}
    ///   * the real `build_view_model` path — dashboard module interpretations +
    ///     the overall-score explanation (de + en)
    ///
    /// and writes them to `reports/interpretations.json`. The trailing
    /// `source_map` lists the remaining fixture-dependent generators with their
    /// source location so the review surface is complete.
    #[test]
    #[ignore = "run on demand to regenerate reports/interpretations.json"]
    fn export_all_interpretations() {
        let bands = [95.0_f32, 80.0, 65.0, 50.0, 20.0];
        let mut records: Vec<serde_json::Value> = Vec::new();

        // 1. Per-module, locale-aware interpret_score — the dominant interpreter.
        let areas = [
            ("accessibility", helpers::InterpretArea::Accessibility),
            ("performance", helpers::InterpretArea::Performance),
            ("security", helpers::InterpretArea::Security),
            ("mobile", helpers::InterpretArea::Mobile),
            ("ux", helpers::InterpretArea::Ux),
            ("journey", helpers::InterpretArea::Journey),
        ];
        for (module, area) in areas {
            for locale in ["de", "en"] {
                for score in bands {
                    records.push(serde_json::json!({
                        "source": "interpret_score",
                        "module": module,
                        "area_locale": locale,
                        "score": score as u32,
                        "text": super::helpers::interpret_score(area, score, locale),
                    }));
                }
            }
        }

        // 2. SEO interpretation lead sentences (no content profile).
        for locale in ["de", "en"] {
            for score in [95_u32, 80, 62, 45, 20] {
                let mut seo = crate::seo::SeoAnalysis::default();
                seo.score = score;
                records.push(serde_json::json!({
                    "source": "build_seo_interpretation",
                    "module": "SEO",
                    "area_locale": locale,
                    "score": score,
                    "text": super::seo::build_seo_interpretation(locale, &seo),
                }));
            }
        }

        // 3. Real builder path — overall-score explanation + dashboard module
        //    interpretations exactly as rendered (Performance + SEO present so
        //    the overall explanation and >1 module are produced).
        for locale in ["de", "en"] {
            let mut wcag = WcagResults::new();
            wcag.add_violation(Violation::new(
                "1.1.1",
                "Non-text Content",
                WcagLevel::A,
                crate::taxonomy::Severity::High,
                "Missing alt text",
                "n1",
            ));
            let report = AuditReport::new("https://example.com".into(), WcagLevel::AA, wcag, 1500)
                .with_performance(crate::audit::PerformanceResults {
                    vitals: WebVitals::default(),
                    score: PerformanceScore {
                        overall: 60,
                        grade: PerformanceGrade::NeedsImprovement,
                        lcp_score: Some(15),
                        fcp_score: Some(15),
                        cls_score: Some(15),
                        interactivity_score: Some(15),
                        metrics_available: 4,
                    },
                    render_blocking: None,
                    content_weight: None,
                    third_party: None,
                    critical_chain: None,
                    minification: None,
                    animations: None,
                    coverage: None,
                })
                .with_seo(SeoAnalysis::default());
            let normalized = normalize(&report);
            let config = ReportConfig {
                locale: locale.to_string(),
                ..ReportConfig::default()
            };
            let vm = build_view_model(&normalized, &config);
            if let Some(text) = &vm.modules.overall_interpretation {
                records.push(serde_json::json!({
                    "source": "overall_interpretation",
                    "module": "overall",
                    "area_locale": locale,
                    "text": text,
                }));
            }
            for m in &vm.modules.dashboard {
                records.push(serde_json::json!({
                    "source": "module_dashboard",
                    "module": m.name,
                    "area_locale": locale,
                    "score": m.score,
                    "text": m.interpretation,
                }));
            }
        }

        // Source map for the fixture-dependent generators not exhaustively
        // enumerated above (each combines profile/threshold inputs).
        let source_map = serde_json::json!([
            {"generator": "build_seo_interpretation (page-type appendix)", "file": "src/output/builder/seo.rs:41"},
            {"generator": "summarize_page_profile", "file": "src/output/builder/seo.rs:194"},
            {"generator": "page_profile_optimization_note", "file": "src/output/builder/seo.rs:246"},
            {"generator": "perf_interpretation (CWV-green special case)", "file": "src/output/builder/single/module_details.rs:123"},
            {"generator": "mobile_interpretation (small-target special case)", "file": "src/output/builder/single/module_details.rs:812"},
            {"generator": "journey_interpretation (page-type prefix)", "file": "src/output/builder/single/module_details.rs:1001"},
        ]);

        let doc = serde_json::json!({
            "description": "All computed report interpretation / explanation texts, for wording review.",
            "count": records.len(),
            "interpretations": records,
            "source_map_fixture_dependent": source_map,
        });

        let out = serde_json::to_string_pretty(&doc).expect("serialize interpretations");
        std::fs::write("reports/interpretations.json", out)
            .expect("write reports/interpretations.json");
        eprintln!(
            "Wrote {} interpretation records to reports/interpretations.json",
            records.len()
        );
    }
}
