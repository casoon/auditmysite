//! Text export of the single report's builder helpers, slice 26 of #936: the
//! texts of `executive.rs` (positive signals, key points, score driver note),
//! `positive.rs` (positive aspects), `serp.rs` (page health rows), `mod.rs`
//! (audit quality note, stability measurements, the quick-win label),
//! `modules_block.rs` (site-quality score explanation), `diagnosis.rs` (category
//! names), `module_details/performance.rs` and `module_details/platform.rs`
//! (performance, security and mobile detail rows) across an enumerated input
//! space in both languages, as records in `reports/report-builder-texts.json`
//! (gitignored; the path can be set with `REPORT_BUILDER_TEXT_EXPORT`). Run it
//! before and after a change of how the texts are stored and compare the two
//! files byte for byte.

use super::*;
use crate::audit::normalized::{AuditFlag, ModuleScoreEntry, NormalizedReport};
use crate::audit::summary::DominantIssue;
use crate::audit::{
    AuditQualityStatus, AuditReport, PerformanceResults, ViewportScoreSet, ViewportScores,
};
use crate::interaction::stability::{StabilityProvenance, StabilityStatus};
use crate::mobile::MobileFriendliness;
use crate::model::{ReportLevel, WcagLevel};
use crate::patterns::{PatternAnalysis, PatternConfidence, RecognizedPattern};
use crate::performance::{
    calculate_performance_score, BlockingResource, CarbonEstimate, ContentWeight, PerformanceScore,
    RenderBlockingAnalysis, VitalMetric, WebVitals,
};
use crate::security::{HeaderTier, SecurityAnalysis, SecurityIssue};
use crate::seo::{PageHealthAnalysis, SeoAnalysis};
use crate::wcag::{Severity, Violation, WcagResults};
use serde_json::{json, Value};

const LANGS: [&str; 2] = ["en", "de"];

/// The finding categories of the diagnosis: the named ones, the indicator
/// modules and one without a module name.
const DIMENSIONS: [&str; 9] = [
    "Accessibility",
    "SEO",
    "Performance",
    "Security",
    "Mobile",
    "UX",
    "AI Visibility",
    "Best Practices",
    "Unknown",
];

/// Every pattern name the positive signals translate, and one they do not.
const PATTERNS: [&str; 11] = [
    "MainNavigation",
    "DisclosureMenu",
    "ModalDialog",
    "TabList",
    "SkipLink",
    "Accordion",
    "EasyLanguage",
    "Form",
    "AddToCart",
    "QuantityStepper",
    "SomethingElse",
];

/// A page with two accessibility rules, through the report contract.
fn report() -> AuditReport {
    let mut results = WcagResults::new();
    for (criterion, name, severity, rule_id) in [
        (
            "1.1.1",
            "Non-text Content",
            Severity::Critical,
            "images/alt-missing",
        ),
        (
            "4.1.2",
            "Name, Role, Value",
            Severity::High,
            "buttons/name-missing",
        ),
    ] {
        for node in ["node-1", "node-2"] {
            results.add_violation(
                Violation::new(criterion, name, WcagLevel::A, severity, "message", node)
                    .with_rule_id(rule_id),
            );
        }
    }
    AuditReport::new(
        "https://example.com".to_string(),
        WcagLevel::AA,
        results,
        1_200,
    )
}

fn checked(report: &AuditReport) -> AuditContext<'_> {
    crate::audit::validation::normalize_checked(report)
        .expect("fixture report passes the report contract")
}

fn config(lang: &str) -> ReportConfig {
    ReportConfig {
        level: ReportLevel::Standard,
        locale: lang.to_string(),
        ..ReportConfig::default()
    }
}

fn module(
    name: &str,
    weight_pct: u32,
    contributes: bool,
    measurement_type: &str,
) -> ModuleScoreEntry {
    ModuleScoreEntry {
        name: name.to_string(),
        score: 80,
        grade: None,
        band: String::new(),
        weight_pct,
        contributes_to_site_quality_score: contributes,
        derived_from: Vec::new(),
        measurement_type: measurement_type.to_string(),
    }
}

fn flag(kind: &str, message: &str) -> AuditFlag {
    AuditFlag {
        kind: kind.to_string(),
        related_rule: None,
        source: "export".to_string(),
        message: message.to_string(),
    }
}

fn vital(value: f64, rating: &str) -> VitalMetric {
    let mut metric = VitalMetric::new(value, 1.0, 2.0);
    metric.rating = rating.to_string();
    metric
}

/// A performance result whose headline score is `overall`.
fn perf(overall: u32) -> PerformanceResults {
    let vitals = WebVitals::default();
    let mut score: PerformanceScore = calculate_performance_score(&vitals, None);
    score.overall = overall;
    PerformanceResults::new(vitals, score)
}

fn security(score: u32, issues: Vec<SecurityIssue>) -> SecurityAnalysis {
    SecurityAnalysis {
        score,
        grade: "B".to_string(),
        headers: Default::default(),
        ssl: Default::default(),
        issues,
        recommendations: Vec::new(),
        protection: Default::default(),
        sourcemap_leaks: Default::default(),
        fetch: Default::default(),
    }
}

fn security_issue(severity: Severity, deduction: u32) -> SecurityIssue {
    SecurityIssue {
        header: "Content-Security-Policy".to_string(),
        issue_type: "unsafe_inline_script".to_string(),
        message: "CSP allows unsafe-inline scripts".to_string(),
        severity,
        tier: HeaderTier::Baseline,
        values: Default::default(),
        score_deduction: deduction,
    }
}

fn mobile(score: u32) -> MobileFriendliness {
    MobileFriendliness {
        score,
        viewport: Default::default(),
        touch_targets: Default::default(),
        font_sizes: Default::default(),
        content_sizing: Default::default(),
        issues: Vec::new(),
    }
}

fn seo(score: u32) -> SeoAnalysis {
    SeoAnalysis {
        score,
        ..SeoAnalysis::default()
    }
}

// ─── executive.rs ──────────────────────────────────────────────────────────

fn pattern(name: &str, confidence: PatternConfidence) -> RecognizedPattern {
    RecognizedPattern {
        pattern: name.to_string(),
        message: format!("{name} found"),
        confidence,
        detail: None,
    }
}

/// `build_positive_signals`: every pattern title and the WCAG signal.
fn positive_signal_records(records: &mut Vec<Value>) {
    let base = report();
    let mut patterns = PatternAnalysis::default();
    for name in PATTERNS {
        patterns
            .recognized
            .push(pattern(name, PatternConfidence::Strong));
    }
    patterns
        .recognized
        .push(pattern("Accordion", PatternConfidence::Partial));
    let mut wcag = WcagResults::new();
    wcag.positives.push(Violation::new(
        "1.4.3",
        "Contrast",
        WcagLevel::AA,
        Severity::Low,
        "Good contrast",
        "node-1",
    ));
    for lang in LANGS {
        for (name, with_patterns, with_wcag) in [
            ("none", false, false),
            ("patterns", true, false),
            ("wcag", false, true),
            ("both", true, true),
        ] {
            let mut ctx = checked(&base);
            if with_patterns {
                ctx.raw_patterns = Some(&patterns);
            }
            if with_wcag {
                ctx.raw_wcag = &wcag;
            }
            let signals: Vec<Value> = build_positive_signals(&I18n::new(lang).unwrap(), &ctx)
                .iter()
                .map(|s| json!([s.title, s.description, s.strong]))
                .collect();
            records.push(json!(["build_positive_signals", name, lang, signals]));
        }
    }
}

/// `build_executive_narrative`: the key points over severities, the top
/// finding's share, the score driver note, the legal and audit flags and the
/// viewport gap.
fn key_point_records(records: &mut Vec<Value>) {
    let base = report();
    let template = {
        let ctx = checked(&base);
        crate::output::builder::build_view_model(&ctx, &config("en"))
            .findings
            .all_findings
            .remove(0)
    };
    let severities: [(&str, u32, u32); 3] = [("critical", 1, 0), ("high", 0, 1), ("clean", 0, 0)];
    for lang in LANGS {
        let i18n = I18n::new(lang).unwrap();
        for (severity_name, critical, high) in severities {
            let severity = SeverityBlock {
                critical,
                high,
                medium: 0,
                low: 0,
                total: critical + high,
                has_issues: critical + high > 0,
                component_issues: 0,
                component_occurrences: 0,
            };
            for legal in [0usize, 2] {
                for flags in [0usize, 1] {
                    for gap in [false, true] {
                        for drivers in [true, false] {
                            for (top_name, top, dominant) in [
                                ("no top finding", false, None),
                                ("dominant 45", true, Some(45.0_f32)),
                                ("dominant 10", true, Some(10.0)),
                                ("dominant 100", true, Some(100.0)),
                                ("fallback", true, None),
                            ] {
                                let ctx = checked(&base);
                                let mut normalized: NormalizedReport = ctx.normalized.clone();
                                normalized.risk.legal_flags = legal;
                                normalized.risk.severe_legal_flags = legal;
                                normalized.audit_flags.clear();
                                if flags > 0 {
                                    normalized.audit_flags.push(flag("note", "note"));
                                }
                                if gap {
                                    normalized
                                        .audit_flags
                                        .push(flag("viewport_gap", "Viewport gap message"));
                                }
                                if !drivers {
                                    normalized.findings.clear();
                                }
                                let mut summary =
                                    crate::audit::summary::analyze_with_locale(&normalized, lang);
                                summary.dominant_issue = dominant.map(|share_pct| DominantIssue {
                                    rule_id: "images/alt-missing".to_string(),
                                    title: "Title".to_string(),
                                    severity: Severity::Critical,
                                    count: 1,
                                    occurrence_total: 1,
                                    share_pct,
                                });
                                let mut group = template.clone();
                                group.title = "Top finding".to_string();
                                group.finding.occurrence_count = 3;
                                let top_findings = if top { vec![group] } else { Vec::new() };
                                let block = build_executive_narrative(
                                    &i18n,
                                    &normalized,
                                    &summary,
                                    &severity,
                                    &top_findings,
                                );
                                records.push(json!([
                                    "build_executive_narrative",
                                    format!(
                                        "{severity_name} legal={legal} flags={flags} gap={gap} drivers={drivers} {top_name}"
                                    ),
                                    lang,
                                    block.key_points
                                ]));
                            }
                        }
                    }
                }
            }
        }
    }
}

// ─── positive.rs ───────────────────────────────────────────────────────────

/// `derive_positive_aspects_from_normalized`: the accessibility state and the
/// four optional modules above and below 80.
fn positive_aspect_records(records: &mut Vec<Value>) {
    let base = report();
    for lang in LANGS {
        for a11y in ["no findings", "score 85", "score 60"] {
            for mask in 0u32..16 {
                let (good, bad) = (80, 79);
                let pick = |bit: u32| if mask & (1 << bit) != 0 { good } else { bad };
                let perf = perf(pick(0));
                let seo = seo(pick(1));
                let security = security(pick(2), Vec::new());
                let mobile = mobile(pick(3));
                let mut ctx = checked(&base);
                match a11y {
                    "no findings" => ctx.normalized.findings.clear(),
                    "score 85" => ctx.normalized.score = 85,
                    _ => ctx.normalized.score = 60,
                }
                if mask != 0 || a11y == "score 60" {
                    ctx.raw_performance = Some(&perf);
                    ctx.raw_seo = Some(&seo);
                    ctx.raw_security = Some(&security);
                    ctx.raw_mobile = Some(&mobile);
                }
                let aspects: Vec<Value> =
                    derive_positive_aspects_from_normalized(&I18n::new(lang).unwrap(), &ctx)
                        .iter()
                        .map(|a| json!([a.area, a.description]))
                        .collect();
                records.push(json!([
                    "derive_positive_aspects_from_normalized",
                    format!("{a11y} mask={mask}"),
                    lang,
                    aspects
                ]));
            }
        }
    }
}

// ─── mod.rs ────────────────────────────────────────────────────────────────

fn stability(
    viewport: &str,
    status: StabilityStatus,
    mutations: u64,
    reason: Option<&str>,
) -> StabilityProvenance {
    StabilityProvenance {
        viewport: viewport.to_string(),
        status,
        waited_ms: 1_500,
        mutation_count: mutations,
        reason: reason.map(String::from),
    }
}

/// The audit quality note, the stability measurements it names and the
/// quick-win metric label, through `build_view_model`.
fn view_model_records(records: &mut Vec<Value>) {
    let base = report();
    let scenarios: Vec<(&str, AuditQualityStatus, Vec<StabilityProvenance>)> = vec![
        ("complete", AuditQualityStatus::Complete, Vec::new()),
        ("insufficient", AuditQualityStatus::Insufficient, Vec::new()),
        ("partial none", AuditQualityStatus::Partial, Vec::new()),
        (
            "partial stable only",
            AuditQualityStatus::Partial,
            vec![stability("mobile", StabilityStatus::Stable, 0, None)],
        ),
        (
            "partial one",
            AuditQualityStatus::Partial,
            vec![stability(
                "mobile",
                StabilityStatus::BudgetExhausted,
                0,
                None,
            )],
        ),
        (
            "partial mutations and reason",
            AuditQualityStatus::Partial,
            vec![stability(
                "desktop",
                StabilityStatus::BudgetExhausted,
                12,
                Some("late layout shifts"),
            )],
        ),
        (
            "partial empty reason",
            AuditQualityStatus::Partial,
            vec![stability(
                "desktop",
                StabilityStatus::BudgetExhausted,
                3,
                Some(""),
            )],
        ),
        (
            "partial two and an empty viewport",
            AuditQualityStatus::Partial,
            vec![
                stability("mobile", StabilityStatus::BudgetExhausted, 1, None),
                stability("desktop", StabilityStatus::Stable, 0, None),
                stability("", StabilityStatus::BudgetExhausted, 0, None),
            ],
        ),
    ];
    for lang in LANGS {
        for (name, status, entries) in &scenarios {
            let mut ctx = checked(&base);
            ctx.normalized.execution.quality.status = *status;
            ctx.normalized.execution.navigation.stability = entries.clone();
            let vm = crate::output::builder::build_view_model(&ctx, &config(lang));
            records.push(json!([
                "build_view_model",
                name,
                lang,
                {
                    "audit_quality_note": vm.summary.audit_quality_note,
                    "metrics": vm.summary.metrics.iter().map(|m| m.title.clone()).collect::<Vec<_>>(),
                }
            ]));
        }
    }
}

// ─── modules_block.rs ──────────────────────────────────────────────────────

fn viewport_scores() -> ViewportScores {
    ViewportScores {
        desktop: ViewportScoreSet {
            accessibility: 91,
            performance: None,
            site_quality: 90,
        },
        mobile: ViewportScoreSet {
            accessibility: 77,
            performance: None,
            site_quality: 76,
        },
        weighted_site_quality: 81,
    }
}

/// The site-quality score explanation of the module block.
fn modules_block_records(records: &mut Vec<Value>) {
    let base = report();
    let performance = perf(90);
    let module_sets: Vec<(&str, Vec<ModuleScoreEntry>)> = vec![
        ("modules=none", Vec::new()),
        (
            "modules=contributing",
            vec![
                module("Accessibility", 60, true, "measured"),
                module("Performance", 40, true, "measured"),
            ],
        ),
        (
            "modules=indicators",
            vec![
                module("Accessibility", 50, true, "measured"),
                module("Security", 50, true, "measured"),
                module("UX", 0, false, "heuristic"),
                module("AI Visibility", 0, false, "heuristic"),
            ],
        ),
        (
            "modules=indicators-only",
            vec![module("UX", 0, false, "heuristic")],
        ),
    ];
    for lang in LANGS {
        let i18n = I18n::new(lang).unwrap();
        for (name, modules) in &module_sets {
            for viewports in [false, true] {
                let mut ctx = checked(&base);
                ctx.raw_performance = Some(&performance);
                ctx.normalized.module_scores = modules.clone();
                ctx.normalized.viewport_scores = viewports.then(viewport_scores);
                let block = build_modules_block_from_normalized(&i18n, &ctx);
                records.push(json!([
                    "build_modules_block_from_normalized",
                    format!("{name} viewports={viewports}"),
                    lang,
                    block.site_quality_interpretation
                ]));
            }
        }
    }
}

// ─── diagnosis.rs ──────────────────────────────────────────────────────────

fn category_name_records(records: &mut Vec<Value>) {
    for lang in LANGS {
        let en = lang == "en";
        for dimension in DIMENSIONS {
            records.push(json!([
                "category_name",
                dimension,
                lang,
                diagnosis::category_name(dimension, en)
            ]));
        }
    }
}

// ─── serp.rs (page health rows) ────────────────────────────────────────────

fn page_health_records(records: &mut Vec<Value>) {
    let base = report();
    let mut scenarios: Vec<(String, PageHealthAnalysis)> = Vec::new();
    for (name, status, detail) in [
        ("delegated", "delegated", None),
        ("executed", "executed", Some("2 errors")),
        ("detail", "skipped", Some("validator offline")),
        ("no detail", "failed", None),
    ] {
        for (redirect, query) in [(false, false), (true, true)] {
            let ph = PageHealthAnalysis {
                url_length: if redirect { 1_234 } else { 25 },
                url_path_depth: if redirect { 7 } else { 1 },
                url_has_query_params: query,
                own_redirect_detected: redirect,
                own_final_url: redirect.then(|| "https://example.com/final".to_string()),
                html_validator_status: status.to_string(),
                html_validator_detail: detail.map(String::from),
                ..PageHealthAnalysis::default()
            };
            scenarios.push((format!("{name} redirect={redirect}"), ph));
        }
    }
    for lang in LANGS {
        let i18n = I18n::new(lang).unwrap();
        for (name, ph) in &scenarios {
            let mut analysis = seo(80);
            analysis.page_health = Some(ph.clone());
            let mut ctx = checked(&base);
            ctx.raw_seo = Some(&analysis);
            let details = build_module_details_from_normalized(&i18n, &ctx);
            let health = details
                .seo
                .and_then(|s| s.page_health)
                .expect("the page has a page health analysis");
            records.push(json!([
                "build_page_health_presentation",
                name,
                lang,
                {
                    "url_info": health.url_info,
                    "html_validator": health.html_validator,
                }
            ]));
        }
    }
}

// ─── module_details/performance.rs ─────────────────────────────────────────

fn content_weight(total: u64, javascript: u64, requests: u32, grams: f64) -> ContentWeight {
    let mut breakdown = crate::performance::ResourceBreakdown::default();
    breakdown.javascript.bytes = javascript;
    ContentWeight {
        total_bytes: total,
        transfer_bytes: total / 3,
        breakdown,
        request_count: requests,
        carbon: CarbonEstimate {
            grams_co2e_per_view: grams,
            rating: "A".to_string(),
            model: "model".to_string(),
        },
        recommendations: Vec::new(),
    }
}

fn blocking(count: usize) -> Vec<BlockingResource> {
    (0..count)
        .map(|n| BlockingResource {
            url: format!("https://example.com/{n}"),
            transfer_bytes: 1_000,
            kind: "script".to_string(),
        })
        .collect()
}

fn render_blocking(
    scripts: usize,
    css: usize,
    transfer: u64,
    first_party: u64,
    third_party: u64,
    origins: u32,
) -> RenderBlockingAnalysis {
    RenderBlockingAnalysis {
        blocking_scripts: blocking(scripts),
        blocking_css: blocking(css),
        blocking_transfer_bytes: transfer,
        first_party_bytes: first_party,
        third_party_bytes: third_party,
        third_party_origin_count: origins,
        suggestions: Vec::new(),
    }
}

fn performance_scenarios() -> Vec<(String, PerformanceResults)> {
    let mut out: Vec<(String, PerformanceResults)> = Vec::new();
    let mut push = |name: String, change: &dyn Fn(&mut PerformanceResults)| {
        let mut p = perf(90);
        change(&mut p);
        out.push((name, p));
    };
    push("bare".to_string(), &|_| {});
    push("heap".to_string(), &|p| {
        p.vitals.js_heap_size = Some(52_428_800)
    });
    for grams in [0.001, 0.5, 12.345] {
        push(format!("carbon={grams}"), &move |p| {
            p.content_weight = Some(content_weight(1_000_000, 100_000, 10, grams))
        });
    }
    for nodes in [500, 800, 801, 1_500, 1_501, 1_234_567] {
        push(format!("dom nodes={nodes}"), &move |p| {
            p.vitals.dom_nodes = Some(nodes)
        });
    }
    for load in [1_000.0, 3_000.0, 3_001.0, 6_000.0, 6_001.0, 123_456.0] {
        push(format!("load={load}"), &move |p| {
            p.vitals.load_time = Some(load)
        });
    }
    for dcl in [1_000.0, 1_800.0, 3_600.0, 3_601.0] {
        push(format!("dcl={dcl}"), &move |p| {
            p.vitals.dom_content_loaded = Some(dcl)
        });
    }
    for (index, rb) in [
        render_blocking(2, 0, 0, 0, 0, 0),
        render_blocking(1, 1, 1_500, 0, 0, 0),
        render_blocking(0, 0, 0, 0, 200_000, 3),
        render_blocking(1, 2, 2_048, 5_000, 150_000, 4),
        render_blocking(0, 0, 0, 5_000, 0, 0),
    ]
    .into_iter()
    .enumerate()
    {
        for good in [false, true] {
            push(format!("render blocking {index} good={good}"), &|p| {
                p.render_blocking = Some(rb.clone());
                if good {
                    p.vitals.lcp = Some(vital(1.0, "good"));
                    p.vitals.fcp = Some(vital(1.0, "good"));
                    p.vitals.cls = Some(vital(0.0, "good"));
                }
            });
        }
    }
    // The score is capped by the page size, the script weight, the number of
    // requests and the DOM, each alone and all together.
    for (name, total, javascript, requests, nodes) in [
        ("size 12M", 12_000_000, 0, 10, 100),
        ("size 6M", 6_000_000, 0, 10, 100),
        ("size 4M", 4_000_000, 0, 10, 100),
        ("js 4M", 1_000_000, 4_000_000, 10, 100),
        ("js 2M", 1_000_000, 2_000_000, 10, 100),
        ("requests", 1_000_000, 0, 130, 100),
        ("dom 3500", 1_000_000, 0, 10, 3_500),
        ("dom 2500", 1_000_000, 0, 10, 2_500),
        ("all", 12_000_000, 4_000_000, 130, 3_500),
    ] {
        push(format!("capped {name}"), &move |p| {
            p.score.is_capped = Some(true);
            p.score.overall = 39;
            p.content_weight = Some(content_weight(total, javascript, requests, 0.5));
            p.vitals.dom_nodes = Some(nodes);
        });
    }
    out
}

fn performance_records(records: &mut Vec<Value>) {
    let base = report();
    for lang in LANGS {
        let i18n = I18n::new(lang).unwrap();
        for (name, p) in performance_scenarios() {
            let mut ctx = checked(&base);
            ctx.raw_performance = Some(&p);
            let details = build_module_details_from_normalized(&i18n, &ctx);
            let p = details.performance.expect("the page has performance data");
            records.push(json!([
                "build_performance_details",
                name,
                lang,
                {
                    "interpretation": p.interpretation,
                    "additional_metrics": p.additional_metrics,
                    "resource_ratings": p.resource_ratings,
                    "render_blocking_metrics": p.render_blocking_metrics,
                    "render_blocking_suggestions": p.render_blocking_suggestions,
                    "has_render_blocking": p.has_render_blocking,
                }
            ]));
        }
    }
}

// ─── module_details/platform.rs ────────────────────────────────────────────

fn security_scenarios() -> Vec<(String, SecurityAnalysis, Option<u32>)> {
    let mut out: Vec<(String, SecurityAnalysis, Option<u32>)> = Vec::new();
    out.push(("bare".to_string(), security(100, Vec::new()), None));
    let mut tls = security(90, Vec::new());
    tls.ssl.https = true;
    tls.ssl.valid_certificate = true;
    tls.ssl.has_hsts = true;
    tls.ssl.hsts_max_age = Some(31_536_000);
    tls.ssl.hsts_include_subdomains = true;
    tls.ssl.protocol = Some("TLS 1.3".to_string());
    tls.ssl.cipher = Some("AES_256_GCM".to_string());
    tls.ssl.certificate_subject = Some("example.com".to_string());
    tls.ssl.certificate_issuer = Some("Issuer".to_string());
    tls.ssl.certificate_expires_in_days = Some(30);
    tls.ssl.certificate_chain_length = Some(3);
    tls.ssl.certificate_error = Some("expired".to_string());
    tls.headers.content_security_policy = Some("default-src 'self'".to_string());
    tls.headers.x_frame_options = Some("DENY".to_string());
    out.push(("tls and headers".to_string(), tls, None));
    for severity in [
        Severity::Critical,
        Severity::High,
        Severity::Medium,
        Severity::Low,
    ] {
        for (score, deduction) in [(95, 0), (95, 10), (60, 5)] {
            out.push((
                format!("issue {severity:?} score={score} deduction={deduction}"),
                security(score, vec![security_issue(severity, deduction)]),
                None,
            ));
        }
    }
    out.push((
        "two issues".to_string(),
        security(
            80,
            vec![
                security_issue(Severity::High, 10),
                security_issue(Severity::Medium, 5),
            ],
        ),
        None,
    ));
    out.push((
        "vulnerable libraries".to_string(),
        security(90, vec![security_issue(Severity::Low, 5)]),
        Some(80),
    ));
    out
}

fn mobile_scenarios() -> Vec<(String, MobileFriendliness)> {
    let mut out: Vec<(String, MobileFriendliness)> = Vec::new();
    out.push(("bare".to_string(), mobile(90)));
    let mut flags = mobile(90);
    flags.viewport.has_viewport = true;
    flags.viewport.is_scalable = true;
    flags.font_sizes.base_font_size = 16.0;
    flags.font_sizes.smallest_font_size = 9.0;
    flags.font_sizes.legible_percentage = 87.5;
    flags.content_sizing.fits_viewport = true;
    flags.touch_targets.total_targets = 40;
    out.push(("flags".to_string(), flags));
    for small in [3u32, 10, 25] {
        for contexts in [
            Vec::new(),
            vec![("navigation".to_string(), 4u32)],
            vec![
                ("navigation".to_string(), 4),
                ("footer".to_string(), 3),
                ("other".to_string(), 2),
                ("form".to_string(), 1),
            ],
        ] {
            let mut m = mobile(70);
            m.touch_targets.small_targets = small;
            m.touch_targets.small_by_context = contexts.clone();
            out.push((format!("small={small} contexts={}", contexts.len()), m));
        }
    }
    out
}

fn platform_records(records: &mut Vec<Value>) {
    let base = report();
    for lang in LANGS {
        let i18n = I18n::new(lang).unwrap();
        for (name, sec, module_score) in security_scenarios() {
            let mut ctx = checked(&base);
            ctx.raw_security = Some(&sec);
            if let Some(score) = module_score {
                ctx.normalized
                    .module_scores
                    .push(module("Security", 10, true, "measured"));
                ctx.normalized.module_scores.last_mut().unwrap().score = score;
            }
            let details = build_module_details_from_normalized(&i18n, &ctx);
            let s = details.security.expect("the page has a security analysis");
            records.push(json!([
                "build_security_details",
                name,
                lang,
                {
                    "band_label": s.band_label,
                    "band_note": s.band_note,
                    "headers": s.headers,
                    "score_rows": s.score_rows,
                    "ssl_info": s.ssl_info,
                    "issues": s.issues.iter().map(|(title, _, message)| json!([title, message])).collect::<Vec<_>>(),
                }
            ]));
        }
        for (name, m) in mobile_scenarios() {
            let mut ctx = checked(&base);
            ctx.raw_mobile = Some(&m);
            let details = build_module_details_from_normalized(&i18n, &ctx);
            let m = details.mobile.expect("the page has a mobile analysis");
            records.push(json!([
                "build_mobile_details",
                name,
                lang,
                {
                    "interpretation": m.interpretation,
                    "viewport": m.viewport,
                    "touch_targets": m.touch_targets,
                    "font_analysis": m.font_analysis,
                    "content_sizing": m.content_sizing,
                }
            ]));
        }
    }
}

/// Every record of the export, in both languages.
pub(super) fn records() -> Vec<Value> {
    let mut records = Vec::new();
    positive_signal_records(&mut records);
    key_point_records(&mut records);
    positive_aspect_records(&mut records);
    view_model_records(&mut records);
    modules_block_records(&mut records);
    category_name_records(&mut records);
    page_health_records(&mut records);
    performance_records(&mut records);
    platform_records(&mut records);
    records
}

#[test]
#[ignore = "writes reports/report-builder-texts.json; run before and after a change of the text storage"]
fn export_report_builder_texts() {
    let records = records();
    let path = std::env::var("REPORT_BUILDER_TEXT_EXPORT").unwrap_or_else(|_| {
        format!(
            "{}/reports/report-builder-texts.json",
            env!("CARGO_MANIFEST_DIR")
        )
    });
    std::fs::create_dir_all(std::path::Path::new(&path).parent().unwrap()).unwrap();
    std::fs::write(&path, serde_json::to_string_pretty(&records).unwrap()).unwrap();
    println!(
        "{} report builder text records written to {path}",
        records.len()
    );
}
