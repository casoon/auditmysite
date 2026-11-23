//! Text export of the interpretation layer (#936, slice 27): the SEO,
//! performance and score interpretation sentences (`seo::interpretation`,
//! `audit::performance_interpretation`, `audit::interpretation`), the
//! verdict reasons, the complexity and expected-impact sentences
//! (`audit::finding_derive`), the audit summary narrative (`audit::summary`),
//! the baseline line, the long-heading message and the score area labels in
//! both languages across an enumerated set of inputs, as records in
//! `reports/audit-interpretation-texts.json` (gitignored; the path can be set
//! with `AUDIT_INTERPRETATION_TEXT_EXPORT`). Run it before and after a change
//! of how the texts are stored and compare the two files byte for byte.

use super::finding_derive::{
    complexity_text, expected_impact_text, ComplexityKind, ExpectedImpactKind, ScoreEffect,
};
use super::interpretation::{
    accessibility_claim_text, interpret_score_localized, interpret_security_score_localized,
    AccessibilityClaim, AccessibilityClaimKind, InterpretArea, Interpretation, LocalizedText,
};
use super::normalized::{NormalizedFinding, NormalizedReport, RiskAssessment, RiskLevel};
use super::performance_interpretation::{
    append_performance_qualifiers_text, performance_gap_text, LATE_THROTTLED_LCP_THRESHOLD_MS,
};
use super::summary::analyze_with_locale;
use super::verdict::VerdictReasonKind;
use super::{AuditQualityStatus, AuditReport, PerformanceResults};
use crate::model::WcagLevel;
use crate::performance::render_blocking::{BlockingResource, RenderBlockingAnalysis};
use crate::performance::{calculate_performance_score, VitalMetric, WebVitals};
use crate::security::{SecurityAnalysis, SecurityHeaders, SslInfo};
use crate::seo::headings::{long_heading_text, HeadingIssue, LongHeading};
use crate::seo::interpretation::{
    page_profile_optimization_note_text, page_profile_summary_text, seo_interpretation_text,
};
use crate::seo::profile::{PageType, SeoContentProfile};
use crate::seo::SeoAnalysis;
use crate::taxonomy::{ScoreArea, Severity};
use crate::wcag::WcagResults;
use serde_json::{json, Value};

const LANGS: [&str; 2] = ["en", "de"];

/// A finding of the summary fixtures: rule id, title, severity, occurrences.
type FindingSpec = (String, String, Severity, usize);

fn push(records: &mut Vec<Value>, name: &str, input: String, lang: &str, output: Value) {
    records.push(json!([name, input, lang, output]));
}

fn is_en(lang: &str) -> bool {
    lang == "en"
}

// ─── SEO interpretation ─────────────────────────────────────────────────────

fn profile(
    page_type: &PageType,
    depth: u32,
    structure: u32,
    media: u32,
    intent: u32,
    attributes: &[&str],
) -> SeoContentProfile {
    let primary_type = serde_json::to_value(page_type).expect("page type serializes");
    serde_json::from_value(json!({
        "content_identity": {
            "summary": "",
            "site_name": "",
            "content_type": "",
            "language": "en",
            "category_hints": [],
        },
        "page_classification": {
            "primary_type": primary_type,
            "attributes": attributes,
            "content_depth_score": depth,
            "structural_richness_score": structure,
            "media_text_balance_score": media,
            "intent_fit_score": intent,
        },
        "schema_inventory": { "schemas": [], "total_count": 0 },
        "signal_strength": { "categories": [], "overall_pct": 0 },
        "maturity": "basic",
        "maturity_techniques": 0,
    }))
    .expect("profile fixture deserializes")
}

fn seo_texts(records: &mut Vec<Value>) {
    for lang in LANGS {
        let en = is_en(lang);
        // Every band edge, with and without a profile.
        for score in [0u32, 34, 35, 54, 55, 69, 70, 89, 90, 100] {
            let seo = SeoAnalysis {
                score,
                ..Default::default()
            };
            push(
                records,
                "seo_interpretation_text",
                format!("score {score}, no profile"),
                lang,
                json!(seo_interpretation_text(&seo, en)),
            );
        }
        // The page-type context and the content-depth caveat.
        for page_type in [
            PageType::Editorial,
            PageType::StructuredContent,
            PageType::MarketingLanding,
            PageType::MediaHeavy,
            PageType::Utility,
            PageType::NavigationHub,
            PageType::ThinContent,
        ] {
            for (score, depth) in [(95u32, 80u32), (95, 40), (80, 54), (80, 55), (60, 40)] {
                let seo = SeoAnalysis {
                    score,
                    content_profile: Some(profile(&page_type, depth, 60, 60, 82, &[])),
                    ..Default::default()
                };
                push(
                    records,
                    "seo_interpretation_text",
                    format!("{page_type:?} score {score} depth {depth}"),
                    lang,
                    json!(seo_interpretation_text(&seo, en)),
                );
            }
        }
        // The profile summary: every quality bucket (average of the four
        // scores), without and with attributes.
        for avg in [100u32, 85, 84, 70, 69, 50, 49, 0] {
            for attributes in [&[][..], &["a long read"][..], &["x", "y", "z"][..]] {
                let p = profile(&PageType::Editorial, avg, avg, avg, avg, attributes);
                push(
                    records,
                    "page_profile_summary_text",
                    format!("average {avg}, attributes {}", attributes.len()),
                    lang,
                    json!(page_profile_summary_text(&p, en)),
                );
            }
        }
        // The optimization note: each gate in turn.
        for (label, page_type, depth, structure, media, intent) in [
            ("thin", PageType::Editorial, 44, 80, 80, 80),
            ("flat", PageType::Editorial, 45, 54, 80, 80),
            ("visual", PageType::Editorial, 45, 55, 54, 80),
            ("off-type", PageType::NavigationHub, 80, 80, 80, 64),
            (
                "off-type, type constant",
                PageType::MediaHeavy,
                80,
                80,
                80,
                62,
            ),
            ("fine", PageType::Editorial, 80, 80, 80, 65),
        ] {
            let p = profile(&page_type, depth, structure, media, intent, &[]);
            push(
                records,
                "page_profile_optimization_note_text",
                label.to_string(),
                lang,
                json!(page_profile_optimization_note_text(&p, en)),
            );
        }
    }
}

// ─── Performance interpretation ─────────────────────────────────────────────

fn vital(value: f64, rating: &str) -> VitalMetric {
    let mut metric = VitalMetric::new(value, 1.0, 2.0);
    metric.rating = rating.to_string();
    metric
}

#[allow(clippy::too_many_arguments)]
fn performance(
    dom_nodes: Option<i64>,
    blocking: usize,
    unminified_kb: u64,
    animations: u32,
    cls: Option<&str>,
    tbt: Option<&str>,
) -> PerformanceResults {
    let vitals = WebVitals {
        dom_nodes,
        cls: cls.map(|rating| vital(0.05, rating)),
        tbt: tbt.map(|rating| vital(100.0, rating)),
        ..Default::default()
    };
    let score = calculate_performance_score(&vitals, None);
    let resource = |kind: &str| BlockingResource {
        url: "https://example.com/a".to_string(),
        transfer_bytes: 1,
        kind: kind.to_string(),
    };
    PerformanceResults {
        vitals,
        score,
        render_blocking: (blocking > 0).then(|| RenderBlockingAnalysis {
            blocking_scripts: (0..blocking.div_ceil(2))
                .map(|_| resource("script"))
                .collect(),
            blocking_css: (0..blocking / 2).map(|_| resource("css")).collect(),
            blocking_transfer_bytes: 0,
            first_party_bytes: 0,
            third_party_bytes: 0,
            third_party_origin_count: 0,
            suggestions: Vec::new(),
        }),
        content_weight: None,
        third_party: None,
        critical_chain: None,
        minification: (unminified_kb > 0).then(|| {
            crate::performance::minification::MinificationAnalysis {
                unminified_scripts: Vec::new(),
                unminified_styles: Vec::new(),
                total_savings_bytes: unminified_kb * 1024,
                total_unminified_count: 1,
                legacy_scripts: Vec::new(),
                total_legacy_wasted_bytes: 0,
            }
        }),
        animations: (animations > 0).then(|| crate::performance::animations::AnimationAnalysis {
            findings: Vec::new(),
            total_count: animations,
            affected_properties: Vec::new(),
        }),
        coverage: None,
        measurement_warnings: Vec::new(),
    }
}

fn performance_texts(records: &mut Vec<Value>) {
    // Each qualifier alone, in pairs and all together; singular and plural
    // positives; blocking counts of one and many.
    let late = LATE_THROTTLED_LCP_THRESHOLD_MS + 1200.0;
    let scenarios: Vec<(String, PerformanceResults, f64)> = vec![
        ("none".into(), performance(None, 0, 0, 0, None, None), 0.0),
        (
            "none, both positive".into(),
            performance(None, 0, 0, 0, Some("good"), Some("good")),
            0.0,
        ),
        (
            "dom".into(),
            performance(Some(4200), 0, 0, 0, None, None),
            0.0,
        ),
        (
            "dom 20368, cls good".into(),
            performance(Some(20368), 0, 0, 0, Some("good"), None),
            0.0,
        ),
        (
            "dom 999999, tbt good".into(),
            performance(Some(999_999), 0, 0, 0, None, Some("good")),
            0.0,
        ),
        ("lcp".into(), performance(None, 0, 0, 0, None, None), late),
        (
            "render-blocking 1".into(),
            performance(None, 1, 0, 0, None, None),
            0.0,
        ),
        (
            "render-blocking 7, both positive".into(),
            performance(None, 7, 0, 0, Some("good"), Some("good")),
            0.0,
        ),
        (
            "unminified".into(),
            performance(None, 0, 180, 0, None, None),
            0.0,
        ),
        (
            "animations".into(),
            performance(None, 0, 0, 14, Some("needs-improvement"), Some("poor")),
            0.0,
        ),
        (
            "dom and lcp".into(),
            performance(Some(3500), 0, 0, 0, Some("good"), None),
            late,
        ),
        (
            "all".into(),
            performance(Some(4200), 7, 180, 14, Some("good"), Some("good")),
            late,
        ),
    ];
    for lang in LANGS {
        let en = is_en(lang);
        for (label, p, lcp) in &scenarios {
            for base in ["", "Base sentence."] {
                push(
                    records,
                    "append_performance_qualifiers_text",
                    format!("{label}, base {base:?}"),
                    lang,
                    json!(append_performance_qualifiers_text(
                        base.to_string(),
                        p,
                        *lcp,
                        en
                    )),
                );
            }
        }
        // The gap sentence: every reason alone and together, and the gates.
        for (label, p, cwv_good, below, blocking) in [
            (
                "gate closed (cwv)",
                performance(Some(2000), 0, 0, 0, None, None),
                false,
                true,
                true,
            ),
            (
                "gate closed (score)",
                performance(Some(2000), 0, 0, 0, None, None),
                true,
                false,
                true,
            ),
            (
                "no reason",
                performance(Some(1500), 0, 0, 0, None, None),
                true,
                true,
                false,
            ),
            (
                "dom",
                performance(Some(1501), 0, 0, 0, None, None),
                true,
                true,
                false,
            ),
            (
                "blocking",
                performance(None, 0, 0, 0, None, None),
                true,
                true,
                true,
            ),
            (
                "tbt",
                performance(None, 0, 0, 0, None, Some("poor")),
                true,
                true,
                false,
            ),
            (
                "tbt good",
                performance(None, 0, 0, 0, None, Some("good")),
                true,
                true,
                false,
            ),
            (
                "all",
                performance(Some(1501), 0, 0, 0, None, Some("needs-improvement")),
                true,
                true,
                true,
            ),
        ] {
            push(
                records,
                "performance_gap_text",
                label.to_string(),
                lang,
                json!(performance_gap_text(
                    "Base sentence.".to_string(),
                    &p,
                    cwv_good,
                    below,
                    blocking,
                    en,
                )),
            );
        }
    }
}

// ─── Score interpretation and the technical overview ────────────────────────

fn localized(text: &LocalizedText, lang: &str) -> Value {
    json!(if is_en(lang) { &text.en } else { &text.de })
}

fn clean_report() -> AuditReport {
    let mut results = WcagResults::new();
    results.nodes_checked = 500;
    let mut report = AuditReport::new(
        "https://example.com".to_string(),
        WcagLevel::AA,
        results,
        100,
    );
    report.timestamp = chrono::DateTime::from_timestamp(1_700_000_000, 0).unwrap();
    report
}

fn security(score: u32, all_headers: bool, issues: usize) -> SecurityAnalysis {
    let header = || all_headers.then(|| "x".to_string());
    SecurityAnalysis {
        score,
        grade: "A".to_string(),
        headers: SecurityHeaders {
            content_security_policy: header(),
            x_content_type_options: header(),
            x_frame_options: header(),
            referrer_policy: header(),
            permissions_policy: header(),
            strict_transport_security: header(),
            cross_origin_opener_policy: header(),
            cross_origin_resource_policy: header(),
            ..Default::default()
        },
        ssl: SslInfo::default(),
        issues: (0..issues)
            .map(|_| crate::security::SecurityIssue {
                header: "Content-Security-Policy".to_string(),
                issue_type: "weak_csp".to_string(),
                message: "CSP is weak.".to_string(),
                severity: Severity::High,
                tier: crate::security::HeaderTier::Baseline,
                values: Default::default(),
                score_deduction: 0,
            })
            .collect(),
        protection: Default::default(),
        sourcemap_leaks: Default::default(),
        fetch: Default::default(),
        recommendations: Vec::new(),
    }
}

fn score_texts(records: &mut Vec<Value>) {
    let areas = [
        ("accessibility", InterpretArea::Accessibility),
        ("performance", InterpretArea::Performance),
        ("security", InterpretArea::Security),
        ("mobile", InterpretArea::Mobile),
        ("ux", InterpretArea::Ux),
        ("journey", InterpretArea::Journey),
        ("html_conform", InterpretArea::HtmlConform),
    ];
    let scores = [
        100.0_f32, 90.0, 89.0, 75.0, 74.0, 60.0, 59.0, 40.0, 39.0, 0.0,
    ];
    for lang in LANGS {
        for (name, area) in areas {
            for score in scores {
                push(
                    records,
                    "interpret_score_localized",
                    format!("{name} {score}"),
                    lang,
                    localized(&interpret_score_localized(area, score), lang),
                );
            }
        }
        // The security wording band follows the open issues, not only the score.
        for score in [95.0_f32, 80.0, 20.0] {
            for issues in [0usize, 1] {
                push(
                    records,
                    "interpret_security_score_localized",
                    format!("{score} with {issues} issues"),
                    lang,
                    localized(
                        &interpret_security_score_localized(
                            score,
                            &security(80, true, issues).issues,
                        ),
                        lang,
                    ),
                );
            }
        }
    }

    // The accessibility claim: every kind, the violation shapes and counts.
    for lang in LANGS {
        let en = is_en(lang);
        let severity = |critical, high, total| crate::audit::normalized::SeverityCounts {
            critical,
            high,
            medium: 0,
            low: 0,
            total,
        };
        let claim = |kind| AccessibilityClaim {
            kind,
            coverage: AuditQualityStatus::Complete,
            wcag_version: "2.2".to_string(),
            automated_criteria: 31,
            total_criteria: 50,
            manual_review_criteria: Vec::new(),
            limits: crate::wcag::coverage::ClaimLimits::with_manual_review(Vec::new()),
        };
        for kind in [
            AccessibilityClaimKind::NotMeasured,
            AccessibilityClaimKind::IncompleteRun,
            AccessibilityClaimKind::NoAutomatedViolationsFound,
        ] {
            push(
                records,
                "accessibility_claim_text",
                format!("{kind:?}"),
                lang,
                json!(accessibility_claim_text(
                    &claim(kind),
                    &severity(0, 0, 0),
                    0,
                    en
                )),
            );
        }
        for (label, critical, high, total, rules) in [
            ("systematic", 5usize, 3usize, 31usize, 12usize),
            ("systematic edge", 5, 0, 30, 12),
            ("critical blockers", 3, 1, 8, 2),
            ("critical blockers, two and many rules", 2, 4, 9, 5),
            ("distributed", 0, 4, 11, 6),
            ("focused", 1, 1, 10, 3),
            ("focused, one violation", 0, 1, 1, 1),
        ] {
            push(
                records,
                "accessibility_claim_text",
                label.to_string(),
                lang,
                json!(accessibility_claim_text(
                    &claim(AccessibilityClaimKind::ViolationsFound),
                    &severity(critical, high, total),
                    rules,
                    en,
                )),
            );
        }
    }

    // The overview, the overall impact and the benchmark, from real contexts.
    let seo_scores: [Option<u32>; 6] = [None, Some(100), Some(85), Some(84), Some(65), Some(45)];
    let seo_low = Some(44u32);
    let security_cases: [(&str, Option<SecurityAnalysis>); 7] = [
        ("none", None),
        ("fully set", Some(security(95, true, 0))),
        ("high, open findings", Some(security(80, false, 1))),
        ("55", Some(security(55, true, 0))),
        ("30", Some(security(30, true, 0))),
        ("10", Some(security(10, false, 0))),
        ("79", Some(security(79, true, 0))),
    ];
    // (label, results, WCAG nodes checked): the performance score is set
    // directly so that every class of the tech-complexity bullet is reached.
    let scored = |dom: Option<i64>, overall: u32| {
        let mut p = performance(dom, 0, 0, 0, None, None);
        p.score.overall = overall;
        Some(p)
    };
    let performance_cases: Vec<(String, Option<PerformanceResults>, usize)> = vec![
        ("none".into(), None, 500),
        ("none, many nodes".into(), None, 2001),
        ("none, 2000 nodes".into(), None, 2000),
        ("dom 2001, perf 59".into(), scored(Some(2001), 59), 500),
        ("dom 2001, perf 60".into(), scored(Some(2001), 60), 500),
        ("dom 2001, perf 74".into(), scored(Some(2001), 74), 500),
        ("dom 2001, perf 75".into(), scored(Some(2001), 75), 500),
        ("dom 2000, perf 59".into(), scored(Some(2000), 59), 500),
        ("dom 2000, perf 60".into(), scored(Some(2000), 60), 500),
        ("dom 2000, perf 79".into(), scored(Some(2000), 79), 500),
        ("dom 2000, perf 80".into(), scored(Some(2000), 80), 500),
        ("dom 20368, perf 90".into(), scored(Some(20368), 90), 500),
        (
            "dom 1000, blocking".into(),
            Some(performance(Some(1000), 9, 300, 20, None, None)),
            500,
        ),
        ("no dom, perf 70".into(), scored(None, 70), 3000),
        ("no dom, perf 90".into(), scored(None, 90), 30),
    ];
    for seo_score in seo_scores.iter().chain([&seo_low]) {
        for (sec_label, sec) in &security_cases {
            for (perf_label, perf, nodes) in &performance_cases {
                let mut report = clean_report();
                report.accessibility.nodes_analyzed = *nodes;
                if let Some(score) = seo_score {
                    report = report.with_seo(SeoAnalysis {
                        score: *score,
                        ..Default::default()
                    });
                }
                if let Some(sec) = sec {
                    report = report.with_security(sec.clone());
                }
                if let Some(perf) = perf {
                    report = report.with_performance(perf.clone());
                }
                let context = super::validation::normalize_checked(&report)
                    .expect("fixture report passes the report contract");
                let interpretation = Interpretation::from_context(&context);
                for lang in LANGS {
                    let label = format!(
                        "seo {seo_score:?}, security {sec_label}, performance {perf_label}"
                    );
                    for (n, bullet) in interpretation.technical_overview.iter().enumerate() {
                        push(
                            records,
                            "technical_overview",
                            format!("{label}, bullet {n}"),
                            lang,
                            localized(bullet, lang),
                        );
                    }
                    for (module, text) in &interpretation.per_module {
                        push(
                            records,
                            "per_module",
                            format!("{label}, {module}"),
                            lang,
                            localized(text, lang),
                        );
                    }
                }
            }
        }
    }
    // Overall impact and benchmark across scores and open findings.
    for score in [
        100u32, 96, 95, 94, 90, 89, 80, 79, 75, 70, 69, 55, 54, 50, 49, 40, 39, 0,
    ] {
        for (critical, high) in [(0usize, 0usize), (1, 0), (2, 0), (0, 2), (0, 3), (1, 2)] {
            let report = clean_report();
            let mut context = super::validation::normalize_checked(&report)
                .expect("fixture report passes the report contract");
            context.normalized.score = score;
            context.normalized.site_quality_score = score;
            context.normalized.severity_counts.critical = critical;
            context.normalized.severity_counts.high = high;
            context.normalized.severity_counts.total = critical + high;
            let interpretation = Interpretation::from_context(&context);
            for lang in LANGS {
                let label = format!("score {score}, critical {critical}, high {high}");
                for (n, (name, value)) in interpretation.overall_impact.iter().enumerate() {
                    push(
                        records,
                        "overall_impact_label",
                        format!("row {n}"),
                        lang,
                        localized(name, lang),
                    );
                    push(
                        records,
                        "overall_impact_value",
                        format!("{label}, row {n}"),
                        lang,
                        localized(value, lang),
                    );
                }
                push(
                    records,
                    "benchmark_context",
                    label.clone(),
                    lang,
                    localized(&interpretation.benchmark_context, lang),
                );
                push(
                    records,
                    "per_module_accessibility",
                    label,
                    lang,
                    localized(&interpretation.per_module["accessibility"], lang),
                );
            }
        }
    }
}

// ─── Verdict reasons, complexity and expected impact ────────────────────────

fn derive_texts(records: &mut Vec<Value>) {
    for lang in LANGS {
        let en = is_en(lang);
        let mut kinds: Vec<VerdictReasonKind> = Vec::new();
        for n in [0usize, 1, 2, 17] {
            kinds.push(VerdictReasonKind::LegalFlags(n));
            kinds.push(VerdictReasonKind::TechnicalBlockers(n));
            kinds.push(VerdictReasonKind::FindingsPresent(n));
        }
        for (total, keyboard) in [(1usize, 0usize), (3, 1), (2, 2), (5, 0)] {
            kinds.push(VerdictReasonKind::BlockingIssues { total, keyboard });
        }
        for (score, threshold) in [(10u32, 40u32), (0, 100)] {
            kinds.push(VerdictReasonKind::ScoreBelowFailThreshold { score, threshold });
            kinds.push(VerdictReasonKind::ScoreBelowWarnThreshold { score, threshold });
        }
        kinds.push(VerdictReasonKind::AuditQualityNotComplete);
        for kind in kinds {
            push(
                records,
                "verdict_reason_text",
                format!("{kind:?}"),
                lang,
                json!(kind.text(en)),
            );
        }
        for count in [0usize, 1, 2, 5, 10, 1234] {
            for kind in [
                ComplexityKind::HighOccurrence {
                    occurrence_count: count,
                },
                ComplexityKind::ModerateOccurrence {
                    occurrence_count: count,
                },
            ] {
                push(
                    records,
                    "complexity_text",
                    format!("{kind:?}"),
                    lang,
                    json!(complexity_text(kind, en)),
                );
            }
            for kind in [
                ExpectedImpactKind::Wcag {
                    occurrence_count: count,
                    score_effect: ScoreEffect::Low,
                    wcag_level: "AA".to_string(),
                    score_gain: None,
                },
                ExpectedImpactKind::Other {
                    occurrence_count: count,
                    score_effect: ScoreEffect::High,
                },
            ] {
                push(
                    records,
                    "expected_impact_text",
                    format!("{kind:?}"),
                    lang,
                    json!(expected_impact_text(&kind, en)),
                );
            }
        }
        for kind in [ComplexityKind::TechnicalPattern, ComplexityKind::LowScope] {
            push(
                records,
                "complexity_text",
                format!("{kind:?}"),
                lang,
                json!(complexity_text(kind, en)),
            );
        }
    }
}

// ─── The audit summary narrative ────────────────────────────────────────────

/// A normalized report whose findings, scores and module scores are set from
/// the arguments; the base comes through the report contract.
fn summary_report(
    template: &NormalizedFinding,
    module_template: &super::normalized::ModuleScoreEntry,
    score: u32,
    findings: &[(&str, &str, Severity, usize)],
    modules: &[(&str, u32)],
) -> NormalizedReport {
    let mut report = super::validation::normalize_checked(&clean_report())
        .expect("fixture report passes the report contract")
        .normalized;
    report.score = score;
    report.site_quality_score = score;
    report.findings = findings
        .iter()
        .map(|(rule_id, title, severity, count)| {
            let mut finding = template.clone();
            finding.rule_id = rule_id.to_string();
            finding.title = title.to_string();
            finding.severity = *severity;
            finding.occurrence_count = *count;
            finding
        })
        .collect();
    let sum = |severity: Severity| -> (usize, usize) {
        let of_severity = report.findings.iter().filter(|f| f.severity == severity);
        (
            of_severity.clone().count(),
            of_severity.map(|f| f.occurrence_count).sum(),
        )
    };
    let (critical, critical_occ) = sum(Severity::Critical);
    let (high, high_occ) = sum(Severity::High);
    let (medium, medium_occ) = sum(Severity::Medium);
    let (low, low_occ) = sum(Severity::Low);
    report.severity_counts.critical = critical;
    report.severity_counts.high = high;
    report.severity_counts.medium = medium;
    report.severity_counts.low = low;
    report.severity_counts.total = report.findings.len();
    report.occurrence_counts.critical = critical_occ;
    report.occurrence_counts.high = high_occ;
    report.occurrence_counts.medium = medium_occ;
    report.occurrence_counts.low = low_occ;
    report.occurrence_counts.total = critical_occ + high_occ + medium_occ + low_occ;
    report.module_scores = modules
        .iter()
        .map(|(name, score)| {
            let mut entry = module_template.clone();
            entry.name = name.to_string();
            entry.score = *score;
            entry
        })
        .collect();
    report
}

fn summary_texts(records: &mut Vec<Value>) {
    let mut results = WcagResults::new();
    results.nodes_checked = 100;
    results.add_violation(crate::wcag::Violation::new(
        "1.1.1",
        "Non-text Content",
        WcagLevel::A,
        Severity::High,
        "Missing alt text",
        "n1",
    ));
    let mut seeded = AuditReport::new(
        "https://example.com".to_string(),
        WcagLevel::AA,
        results,
        100,
    )
    .with_seo(SeoAnalysis::default());
    seeded.timestamp = chrono::DateTime::from_timestamp(1_700_000_000, 0).unwrap();
    let context = super::validation::normalize_checked(&seeded)
        .expect("fixture report passes the report contract");
    let template = context.normalized.findings[0].clone();
    let module_template = context.normalized.module_scores[0].clone();

    let many = |severity: Severity, rules: usize| -> Vec<FindingSpec> {
        (0..rules)
            .map(|i| (format!("rule-{i:02}"), format!("Title {i}"), severity, 1))
            .collect()
    };
    let mut finding_sets: Vec<(String, Vec<FindingSpec>)> = vec![
        ("none".into(), vec![]),
        (
            "one high".into(),
            vec![("r1".into(), "Missing alt".into(), Severity::High, 1)],
        ),
        (
            "dominant, many occurrences".into(),
            vec![
                ("r1".into(), "Missing alt".into(), Severity::Critical, 5),
                ("r2".into(), "Contrast".into(), Severity::High, 1),
            ],
        ),
        (
            "dominant, two groups".into(),
            vec![
                ("r1".into(), "Missing alt".into(), Severity::High, 3),
                ("r1".into(), "Missing alt".into(), Severity::High, 3),
                ("r2".into(), "Contrast".into(), Severity::High, 1),
            ],
        ),
        (
            "dominant among medium findings".into(),
            vec![
                ("r1".into(), "Missing alt".into(), Severity::Medium, 9),
                ("r2".into(), "Contrast".into(), Severity::Low, 1),
            ],
        ),
        ("three spread".into(), {
            let mut set = many(Severity::High, 3);
            set[0].2 = Severity::Critical;
            set
        }),
        ("three spread, all high".into(), many(Severity::High, 3)),
        ("four spread".into(), many(Severity::High, 4)),
        ("two spread".into(), {
            let mut set = many(Severity::Medium, 3);
            set.truncate(3);
            set
        }),
        ("systematic by total".into(), many(Severity::Low, 41)),
        ("systematic by critical".into(), {
            let mut set = many(Severity::Low, 26);
            for item in set.iter_mut().take(5) {
                item.2 = Severity::Critical;
            }
            set
        }),
        ("structural".into(), {
            let mut set = many(Severity::High, 4);
            for item in set.iter_mut() {
                item.3 = 9;
            }
            set
        }),
        ("heading finding".into(), {
            let mut set = many(Severity::Medium, 3);
            set[0].0 = "headings/skipped".into();
            set[1].1 = "Überschrift fehlt".into();
            set
        }),
    ];
    finding_sets.push((
        "h1 title".into(),
        vec![
            ("r1".into(), "Missing H1".into(), Severity::Medium, 1),
            ("r2".into(), "Other".into(), Severity::Medium, 1),
            ("r3".into(), "Third".into(), Severity::Medium, 1),
        ],
    ));
    let module_sets: [(&str, Vec<(&str, u32)>); 5] = [
        ("no modules", vec![]),
        ("seo weak", vec![("SEO", 40)]),
        (
            "security and mobile weak",
            vec![("Security", 30), ("Mobile", 50)],
        ),
        (
            "performance and mobile weak",
            vec![("Performance", 30), ("Mobile", 50)],
        ),
        (
            "all weak",
            vec![
                ("SEO", 40),
                ("Security", 30),
                ("Mobile", 50),
                ("Performance", 30),
            ],
        ),
    ];
    let mut reports: Vec<(String, NormalizedReport)> = Vec::new();
    for (finding_label, set) in &finding_sets {
        let findings: Vec<(&str, &str, Severity, usize)> = set
            .iter()
            .map(|(a, b, c, d)| (a.as_str(), b.as_str(), *c, *d))
            .collect();
        for score in [95u32, 80, 65, 50, 30] {
            for (module_label, modules) in &module_sets {
                reports.push((
                    format!("{finding_label}, score {score}, {module_label}"),
                    summary_report(&template, &module_template, score, &findings, modules),
                ));
            }
        }
    }
    // Counts that the normalization never produces together (group counts
    // against occurrence counts), so that every sentence of the narrative is
    // reached.
    let spread = |severity: Severity| -> Vec<(&'static str, &'static str, Severity, usize)> {
        vec![
            ("r1", "A", severity, 1),
            ("r2", "B", severity, 1),
            ("r3", "C", severity, 1),
        ]
    };
    for (label, score, severity, urgent_groups, urgent_occurrences) in [
        (
            "polished, three urgent groups",
            95u32,
            Severity::High,
            3usize,
            0usize,
        ),
        ("polished, one urgent group", 95, Severity::Medium, 1, 0),
        ("needs work, one urgent group", 80, Severity::Medium, 1, 1),
        (
            "needs work, three urgent groups",
            80,
            Severity::Medium,
            3,
            3,
        ),
        ("needs work, clustered", 80, Severity::Medium, 5, 5),
    ] {
        let mut report = summary_report(&template, &module_template, score, &spread(severity), &[]);
        report.severity_counts.critical = 0;
        report.severity_counts.high = urgent_groups;
        report.occurrence_counts.critical = 0;
        report.occurrence_counts.high = urgent_occurrences;
        reports.push((label.to_string(), report));
    }
    for (label, report) in &reports {
        for lang in LANGS {
            let summary = analyze_with_locale(report, lang);
            push(
                records,
                "summary_verdict_intro",
                label.clone(),
                lang,
                json!(summary.verdict_intro),
            );
            push(
                records,
                "summary_problem_type_label",
                label.clone(),
                lang,
                json!(summary.problem_type_label),
            );
            push(
                records,
                "summary_dominant_issue_note",
                label.clone(),
                lang,
                json!(summary.dominant_issue_note),
            );
            push(
                records,
                "summary_cross_impacts",
                label.clone(),
                lang,
                json!(summary
                    .cross_impacts
                    .iter()
                    .map(|c| (c.dimensions.clone(), c.description.clone()))
                    .collect::<Vec<_>>()),
            );
        }
    }
}

// ─── The small ones ─────────────────────────────────────────────────────────

fn small_texts(records: &mut Vec<Value>) {
    for lang in LANGS {
        let en = is_en(lang);
        for (known, new, fixed, unmatched) in [
            (0usize, 0usize, 0usize, None),
            (3, 1, 2, None),
            (3, 1, 2, Some(0usize)),
            (1, 0, 0, Some(1)),
            (1234, 5, 678, Some(12)),
        ] {
            push(
                records,
                "baseline_summary_text",
                format!("{known}/{new}/{fixed}/{unmatched:?}"),
                lang,
                json!(super::baseline::summary_text(
                    known, new, fixed, unmatched, en
                )),
            );
        }
        for (level, length, excerpt) in [
            (1u8, 80usize, "A very long"),
            (3, 120, ""),
            (6, 5, "Überschrift"),
        ] {
            let values = LongHeading {
                level,
                length,
                excerpt: excerpt.to_string(),
            };
            push(
                records,
                "long_heading_text",
                format!("{level}/{length}/{excerpt}"),
                lang,
                json!(long_heading_text(&values, en)),
            );
            let issue = HeadingIssue {
                issue_type: "long_heading".to_string(),
                message: "stored".to_string(),
                severity: Severity::Medium,
                long_heading: Some(values),
            };
            push(
                records,
                "heading_issue_localized_message",
                format!("{level}/{length}/{excerpt}"),
                lang,
                json!(issue.localized_message(en)),
            );
        }
        for area in ScoreArea::ALL {
            push(
                records,
                "score_area_localized_label",
                area.label().to_string(),
                lang,
                json!(area.localized_label(en)),
            );
        }
        for level in [
            RiskLevel::Critical,
            RiskLevel::High,
            RiskLevel::Medium,
            RiskLevel::Low,
        ] {
            for (legal, blocking, critical, high, interactive_critical, interactive_high) in [
                (3usize, 2usize, 4usize, 5usize, 0usize, 0usize),
                (1, 0, 4, 5, 0, 0),
                (0, 0, 0, 3, 2, 0),
                (0, 0, 0, 3, 0, 4),
                (0, 0, 1, 3, 0, 0),
                (0, 0, 0, 0, 0, 1),
            ] {
                let risk = RiskAssessment {
                    level,
                    score: 0,
                    threshold: 0,
                    driven_by: String::new(),
                    critical_issues: critical,
                    high_issues: high,
                    legal_flags: legal,
                    severe_legal_flags: legal,
                    blocking_issues: blocking,
                    keyboard_blocking_issues: 0,
                    interactive_critical_issues: interactive_critical,
                    interactive_high_issues: interactive_high,
                    summary: "stored summary".to_string(),
                };
                push(
                    records,
                    "risk_summary_for",
                    format!(
                        "{level:?} {legal}/{blocking}/{critical}/{high}/{interactive_critical}/{interactive_high}"
                    ),
                    lang,
                    json!(risk.summary_for(lang)),
                );
            }
        }
    }
}

/// Every text of the interpretation layer in both languages.
pub(crate) fn interpretation_texts() -> Vec<Value> {
    let mut records = Vec::new();
    seo_texts(&mut records);
    performance_texts(&mut records);
    score_texts(&mut records);
    derive_texts(&mut records);
    summary_texts(&mut records);
    small_texts(&mut records);
    records
}

#[test]
#[ignore = "writes reports/audit-interpretation-texts.json; run before and after a change of the text storage"]
fn export_audit_interpretation_texts() {
    let records = interpretation_texts();
    let path = std::env::var("AUDIT_INTERPRETATION_TEXT_EXPORT").unwrap_or_else(|_| {
        format!(
            "{}/reports/audit-interpretation-texts.json",
            env!("CARGO_MANIFEST_DIR")
        )
    });
    std::fs::create_dir_all(std::path::Path::new(&path).parent().unwrap()).unwrap();
    std::fs::write(&path, serde_json::to_string_pretty(&records).unwrap()).unwrap();
    println!(
        "{} interpretation text records written to {path}",
        records.len()
    );
}
