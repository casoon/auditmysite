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

pub use batch::{
    build_batch_presentation, build_batch_presentation_with_locale,
    build_batch_presentation_with_normalized,
};
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
                    si_score: Some(15),
                    metrics_available: 5,
                    size_penalty: None,
                    js_penalty: None,
                    request_penalty: None,
                    dom_penalty: None,
                    is_capped: None,
                },
                render_blocking: None,
                content_weight: None,
                third_party: None,
                critical_chain: None,
                minification: None,
                animations: None,
                coverage: None,
                measurement_warnings: vec![],
            })
            .with_seo(SeoAnalysis::default());

        let normalized = normalize(&report);
        let vm = build_view_model(&normalized, &ReportConfig::default());

        assert_eq!(vm.summary.score, normalized.normalized.score);
        assert_eq!(vm.summary.grade, normalized.normalized.grade);
        assert_eq!(vm.summary.certificate, normalized.normalized.certificate);
        assert!(vm
            .summary
            .metrics
            .iter()
            .any(|m| m.title == format!("Gesamtscore{NBSP}Website")));
        assert!(!vm.executive.key_points.is_empty());
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

        assert_eq!(
            pres.url_ranking[0].score as u32,
            normalized.normalized.score
        );
        assert_eq!(pres.url_ranking[0].grade, normalized.normalized.grade);
        assert_eq!(
            pres.url_details[0].module_scores.len(),
            normalized.normalized.module_scores.len()
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
                    si_score: Some(15),
                    metrics_available: 5,
                    size_penalty: None,
                    js_penalty: None,
                    request_penalty: None,
                    dom_penalty: None,
                    is_capped: None,
                },
                render_blocking: None,
                content_weight: None,
                third_party: None,
                critical_chain: None,
                minification: None,
                animations: None,
                coverage: None,
                measurement_warnings: vec![],
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
                    ..Default::default()
                },
                ssl: crate::security::SslInfo {
                    https: true,
                    valid_certificate: true,
                    has_hsts: true,
                    hsts_max_age: Some(31536000),
                    hsts_include_subdomains: true,
                    hsts_preload: false,
                    ..Default::default()
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
                    pagination_detected: false,
                    pagination_prev: None,
                    pagination_next: None,
                    word_count: 650,
                    internal_links: 12,
                    external_links: 1,
                    dofollow_links: 12,
                    nofollow_links: 1,
                    internal_link_targets: vec![],
                    broken_links: vec![],
                    mixed_content: vec![],
                    pwa: crate::seo::technical::PwaAnalysis::default(),
                    amp: crate::seo::technical::AmpAnalysis::default(),
                    form_security: crate::seo::technical::FormSecurityAnalysis::default(),
                    text_excerpt: text_excerpt.to_string(),
                    uses_remote_google_fonts: false,
                    google_fonts_sources: vec![],
                    tracking_cookies: vec![],
                    cookie_inventory: vec![],
                    storage_items: vec![],
                    tracking_signals: vec![],
                    zaraz: crate::seo::technical::ZarazDetection::default(),
                    has_favicon: true,
                    www_redirect: None,
                    issues: vec![],
                    hreflang_has_x_default: false,
                    hreflang_missing_self_reference: false,
                    internal_links_with_query_params: 0,
                };
                seo.content_profile = Some(crate::seo::build_content_profile(&seo, "de"));
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
        // Performance, Search Experience, Security, UX each produce a dashboard card.
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
        let dashboard_modules = [
            "Accessibility",
            "Performance",
            "Search Experience",
            "Security",
            "UX",
        ];
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
        assert!(
            vm.module_details.seo.is_some(),
            "technical SEO detail section must remain available"
        );
        assert!(
            vm.module_details.search_experience.is_some(),
            "composite Search Experience detail section must be present"
        );
    }

    #[test]
    fn test_search_experience_dashboard_corrects_optimistic_seo_score() {
        let report = make_topic_report_with_modules(
            "https://example.com/",
            "Technically OK",
            "Technically complete page.",
            &["Offers"],
            "short image-led content",
            72.0,
            88,
            70,
            90,
        )
        .with_ux(crate::ux::UxAnalysis {
            score: 62,
            grade: "D".into(),
            cta_clarity: crate::ux::UxDimension {
                kind: crate::ux::UxDimensionKind::CtaClarity,
                name: "CTA Clarity".into(),
                score: 70,
                weight: 0.30,
                summary: "CTA ausreichend".into(),
            },
            visual_hierarchy: crate::ux::UxDimension {
                kind: crate::ux::UxDimensionKind::VisualHierarchy,
                name: "Visual Hierarchy".into(),
                score: 60,
                weight: 0.20,
                summary: "Heading-Hierarchie lueckenhaft".into(),
            },
            content_clarity: crate::ux::UxDimension {
                kind: crate::ux::UxDimensionKind::ContentClarity,
                name: "Content Clarity".into(),
                score: 45,
                weight: 0.20,
                summary: "Wenig verwertbarer Inhalt".into(),
            },
            trust_signals: crate::ux::UxDimension {
                kind: crate::ux::UxDimensionKind::TrustSignals,
                name: "Trust Signals".into(),
                score: 35,
                weight: 0.15,
                summary: "Wichtige Vertrauenssignale fehlen".into(),
            },
            cognitive_load: crate::ux::UxDimension {
                kind: crate::ux::UxDimensionKind::CognitiveLoad,
                name: "Cognitive Load".into(),
                score: 100,
                weight: 0.15,
                summary: "Angemessene Komplexität".into(),
            },
            issues: vec![],
        });

        let normalized = normalize(&report);
        let vm = build_view_model(&normalized, &ReportConfig::default());
        let sx = vm
            .modules
            .dashboard
            .iter()
            .find(|m| m.name.contains("Sichtbarkeit"))
            .expect("Search Experience dashboard card missing");

        assert!(
            sx.score < 88,
            "composite score should correct SEO-only score"
        );
        assert!(sx.interpretation.contains("Technisch auffindbar"));
        assert!(vm.modules.dashboard.iter().all(|m| m.name != "SEO"));
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
            pagination_detected: false,
            pagination_prev: None,
            pagination_next: None,
            word_count: 650,
            internal_links: 12,
            external_links: 1,
            dofollow_links: 12,
            nofollow_links: 1,
            internal_link_targets: vec![],
            broken_links: vec![],
            mixed_content: vec![],
            pwa: crate::seo::technical::PwaAnalysis::default(),
            amp: crate::seo::technical::AmpAnalysis::default(),
            form_security: crate::seo::technical::FormSecurityAnalysis::default(),
            text_excerpt: text_excerpt.to_string(),
            uses_remote_google_fonts: false,
            google_fonts_sources: vec![],
            tracking_cookies: vec![],
            cookie_inventory: vec![],
            storage_items: vec![],
            tracking_signals: vec![],
            zaraz: crate::seo::technical::ZarazDetection::default(),
            has_favicon: true,
            www_redirect: None,
            issues: vec![],
            hreflang_has_x_default: false,
            hreflang_missing_self_reference: false,
            internal_links_with_query_params: 0,
        };
        seo.content_profile = Some(crate::seo::build_content_profile(&seo, "de"));
        seo.score = 92;

        let mut report = AuditReport::new(url.to_string(), WcagLevel::AA, WcagResults::new(), 1500)
            .with_seo(seo);
        report.score = score;
        report
    }

    /// Invariants for `interpret_score_localized` wording (CLAUDE.md "Report Wording Style"):
    ///   1. Localization actually happens — `en` must never fall back to the `de` sentence.
    ///   2. Each module/band carries the controlled band label in the right locale.
    ///
    /// Test moved here from audit::interpretation::tests to stay close to the builder
    /// and verify the end-to-end path from the pre-computed interpretation.
    #[test]
    fn test_interpret_score_localized_and_label_contract() {
        use crate::audit::interpretation::{interpret_score_localized, InterpretArea};

        let areas = [
            InterpretArea::Accessibility,
            InterpretArea::Performance,
            InterpretArea::Security,
            InterpretArea::Mobile,
            InterpretArea::Ux,
            InterpretArea::Journey,
        ];
        let bands = [
            (95.0_f32, "Sehr gut", "Excellent"),
            (80.0, "Gut", "Good"),
            (65.0, "Verbesserungswürdig", "Needs improvement"),
            (50.0, "Ausbaufähig", "Inadequate"),
            (20.0, "Kritisch", "Critical"),
        ];

        for area in areas {
            for (score, de_label, en_label) in bands {
                let text = interpret_score_localized(area, score);
                assert_ne!(
                    text.de, text.en,
                    "de/en interpretation identical at score {score} — en fell back to de"
                );
                assert!(
                    text.de.starts_with(&format!("{de_label} \u{2014}")),
                    "DE band label wrong at score {score}: {:?}",
                    text.de
                );
                assert!(
                    text.en.starts_with(&format!("{en_label} \u{2014}")),
                    "EN band label wrong at score {score}: {:?}",
                    text.en
                );
                assert!(
                    !text.de.contains("Befriedigend"),
                    "banned school grade reappeared: {:?}",
                    text.de
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

        // 1. Per-module, locale-aware interpret_score_localized — the dominant interpreter.
        use crate::audit::interpretation::{interpret_score_localized, InterpretArea};
        let areas = [
            ("accessibility", InterpretArea::Accessibility),
            ("performance", InterpretArea::Performance),
            ("security", InterpretArea::Security),
            ("mobile", InterpretArea::Mobile),
            ("ux", InterpretArea::Ux),
            ("journey", InterpretArea::Journey),
        ];
        for (module, area) in areas {
            for score in bands {
                let text = interpret_score_localized(area, score);
                for (locale, t) in [("de", &text.de), ("en", &text.en)] {
                    records.push(serde_json::json!({
                        "source": "interpret_score",
                        "module": module,
                        "area_locale": locale,
                        "score": score as u32,
                        "text": t,
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
                        si_score: Some(15),
                        metrics_available: 5,
                        size_penalty: None,
                        js_penalty: None,
                        request_penalty: None,
                        dom_penalty: None,
                        is_capped: None,
                    },
                    render_blocking: None,
                    content_weight: None,
                    third_party: None,
                    critical_chain: None,
                    minification: None,
                    animations: None,
                    coverage: None,
                    measurement_warnings: vec![],
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

        let out_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("reports")
            .join("interpretations.json");
        let out = serde_json::to_string_pretty(&doc).expect("serialize interpretations");
        std::fs::write(&out_path, out).expect("write reports/interpretations.json");
        eprintln!(
            "Wrote {} interpretation records to {}",
            records.len(),
            out_path.display()
        );
    }
}
