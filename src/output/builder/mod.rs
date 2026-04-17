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
    use crate::cli::WcagLevel;
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
                    lcp_score: 15,
                    fcp_score: 15,
                    cls_score: 15,
                    interactivity_score: 15,
                },
                render_blocking: None,
                content_weight: None,
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
                    lcp_score: 15,
                    fcp_score: 15,
                    cls_score: 15,
                    interactivity_score: 15,
                },
                render_blocking: None,
                content_weight: None,
            })
            .with_security(crate::security::SecurityAnalysis {
                score: security_score,
                grade: "A".to_string(),
                headers: crate::security::SecurityHeaders {
                    content_security_policy: Some("default-src 'self'".to_string()),
                    x_frame_options: Some("DENY".to_string()),
                    x_content_type_options: Some("nosniff".to_string()),
                    x_xss_protection: Some("1; mode=block".to_string()),
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
                    internal_link_targets: vec![],
                    broken_links: vec![],
                    text_excerpt: text_excerpt.to_string(),
                    uses_remote_google_fonts: false,
                    google_fonts_sources: vec![],
                    tracking_cookies: vec![],
                    tracking_signals: vec![],
                    zaraz: crate::seo::technical::ZarazDetection::default(),
                    issues: vec![],
                };
                seo.content_profile = Some(crate::seo::build_content_profile(&seo));
                seo.score = seo_score;
                seo
            })
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
            internal_link_targets: vec![],
            broken_links: vec![],
            text_excerpt: text_excerpt.to_string(),
            uses_remote_google_fonts: false,
            google_fonts_sources: vec![],
            tracking_cookies: vec![],
            tracking_signals: vec![],
            zaraz: crate::seo::technical::ZarazDetection::default(),
            issues: vec![],
        };
        seo.content_profile = Some(crate::seo::build_content_profile(&seo));
        seo.score = 92;

        let mut report = AuditReport::new(url.to_string(), WcagLevel::AA, WcagResults::new(), 1500)
            .with_seo(seo);
        report.score = score;
        report.grade = helpers::grade_label(score.round() as u32).to_string();
        report
    }
}
