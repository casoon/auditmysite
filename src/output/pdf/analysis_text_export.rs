//! Text export of the search experience and analysis sections, slice 24 of
//! #936: everything `output/search_experience.rs`, `root_causes.rs`,
//! `problem_profile.rs` and `wcag_coverage.rs` put into the report, across an
//! enumerated input space in both languages, as records in
//! `reports/pdf-analysis-texts.json` (gitignored; the path can be set with
//! `PDF_ANALYSIS_TEXT_EXPORT`). The search experience is built from a
//! context whose module results are replaced one by one, the render functions
//! are driven with a bare `ReportBuilder` and their components are recorded as
//! JSON. Run it before and after a change of how the texts are stored and
//! compare the two files byte for byte.

use super::problem_profile::{
    build_overall_leverage_note, build_problem_concentration_note, render_problem_profile,
};
use super::root_causes::{render_root_cause_analysis, render_timeframe_roadmap};
use super::wcag_coverage::render_wcag_coverage_section;
use crate::audit::management_risk::ManagementRiskKind;
use crate::audit::AuditReport;
use crate::i18n::I18n;
use crate::model::WcagLevel;
use crate::output::report_model::*;
use crate::wcag::{Severity, WcagResults};
use serde_json::{json, Value};

const LANGS: [&str; 2] = ["en", "de"];

fn components(builder: renderreport::engine::ReportBuilder) -> Value {
    Value::Array(builder.build().components)
}

fn bare() -> renderreport::engine::ReportBuilder {
    renderreport::engine::ReportBuilder::new("export")
}

fn push(records: &mut Vec<Value>, name: &str, input: String, lang: &str, output: Value) {
    records.push(json!([name, input, lang, output]));
}

fn violation(criterion: &str, rule_id: &str) -> crate::wcag::Violation {
    crate::wcag::Violation::new(
        criterion,
        "Rule name",
        WcagLevel::A,
        Severity::High,
        "Message.",
        "node-1",
    )
    .with_rule_id(rule_id)
    .with_selector("div.main > img.hero")
    .with_html_snippet("<img class=\"hero\">")
    .with_fix("Fix it")
}

/// A clean page and pages with violations, at a fixed time.
fn reports() -> Vec<(&'static str, AuditReport)> {
    let fixed = chrono::DateTime::from_timestamp(1_700_000_000, 0).unwrap();
    let mut out = Vec::new();
    for (name, violations) in [
        ("clean", vec![]),
        (
            "one violation",
            vec![violation("1.1.1", "images/alt-missing")],
        ),
        (
            "two violations",
            vec![
                violation("1.1.1", "images/alt-missing"),
                violation("1.4.3", "contrast/text-insufficient"),
            ],
        ),
    ] {
        let mut results = WcagResults::new();
        results.nodes_checked = 50;
        results.passes = 10;
        for v in violations {
            results.add_violation(v);
        }
        let mut report =
            AuditReport::new("https://example.com".into(), WcagLevel::AA, results, 100);
        report.timestamp = fixed;
        out.push((name, report));
    }
    out
}

/// The view model of the one-violation page and a finding group to copy.
fn view_model(lang: &str) -> (ReportViewModel, FindingGroup) {
    let (_, report) = reports().remove(1);
    let context = crate::audit::validation::normalize_checked(&report)
        .expect("fixture report passes the report contract");
    let config = ReportConfig {
        locale: lang.to_string(),
        ..ReportConfig::default()
    };
    let vm = crate::output::builder::build_view_model(&context, &config);
    let template = vm.findings.all_findings[0].clone();
    (vm, template)
}

// ─── Search experience ──────────────────────────────────────────────────────

fn ux_dimension(
    kind: crate::ux::UxDimensionKind,
    name: &str,
    score: u32,
) -> crate::ux::UxDimension {
    crate::ux::UxDimension {
        kind,
        name: name.to_string(),
        score,
        weight: 0.2,
        summary: "Canonical English summary.".to_string(),
    }
}

fn ux_issue(kind: crate::ux::UxIssueKind) -> crate::ux::UxIssue {
    crate::ux::UxIssue {
        kind,
        dimension: "Dimension".to_string(),
        severity: "high".to_string(),
        problem: "Problem.".to_string(),
        impact: "Impact.".to_string(),
        recommendation: "Recommendation.".to_string(),
        values: crate::ux::UxIssueValues::default(),
    }
}

/// (content clarity, trust signals, visual hierarchy) scores and whether the
/// dimensions carry an issue.
fn ux(name: &str) -> Option<crate::ux::UxAnalysis> {
    use crate::ux::{UxDimensionKind as K, UxIssueKind as I};
    let (content, trust, hierarchy, issues) = match name {
        "none" => return None,
        "good" => (90, 90, 90, vec![]),
        "weak" => (
            40,
            30,
            50,
            vec![
                ux_issue(I::LittleText),
                ux_issue(I::NoContact),
                ux_issue(I::NoH1),
            ],
        ),
        _ => (69, 70, 71, vec![ux_issue(I::NoCta)]),
    };
    Some(crate::ux::UxAnalysis {
        score: 60,
        grade: "C".to_string(),
        cta_clarity: ux_dimension(K::CtaClarity, "CTA Clarity", 70),
        visual_hierarchy: ux_dimension(K::VisualHierarchy, "Visual Hierarchy", hierarchy),
        content_clarity: ux_dimension(K::ContentClarity, "Content Clarity", content),
        trust_signals: ux_dimension(K::TrustSignals, "Trust Signals", trust),
        cognitive_load: ux_dimension(K::CognitiveLoad, "Cognitive Load", 80),
        issues,
    })
}

fn content_profile(name: &str) -> Option<crate::seo::profile::SeoContentProfile> {
    let (page_type, categories) = match name {
        "none" => return None,
        "article" => (
            "editorial",
            json!([
                {"name": "Überschriften", "passed": 3, "total": 4, "checks": []},
                {"name": "Structured data", "passed": 1, "total": 4, "checks": []},
            ]),
        ),
        _ => ("media_heavy", json!([])),
    };
    Some(
        serde_json::from_value(json!({
            "content_identity": {
                "summary": "",
                "site_name": "",
                "content_type": "",
                "language": "en",
                "category_hints": [],
            },
            "page_classification": {
                "primary_type": page_type,
                "attributes": [],
                "content_depth_score": 60,
                "structural_richness_score": 60,
                "media_text_balance_score": 60,
                "intent_fit_score": 82,
            },
            "schema_inventory": { "schemas": [], "total_count": 0 },
            "signal_strength": { "categories": categories, "overall_pct": 55 },
            "maturity": "basic",
            "maturity_techniques": 0,
        }))
        .expect("profile fixture deserializes"),
    )
}

fn seo(score: u32, profile: &str, image_led: bool) -> crate::seo::SeoAnalysis {
    let mut seo = crate::seo::SeoAnalysis {
        score,
        content_profile: content_profile(profile),
        ..Default::default()
    };
    seo.technical.word_count = if image_led { 100 } else { 1500 };
    if image_led {
        seo.image_efficiency = Some(crate::seo::image_efficiency::ImageEfficiencyAnalysis {
            total_images: 40,
            modern_format_count: 0,
            legacy_format_count: 40,
            modern_format_pct: 0.0,
            oversized_images: vec![],
            legacy_format_urls: vec![],
            score: 50,
        });
    }
    seo
}

fn mobile(score: u32) -> crate::mobile::MobileFriendliness {
    crate::mobile::MobileFriendliness {
        score,
        viewport: Default::default(),
        touch_targets: Default::default(),
        font_sizes: Default::default(),
        content_sizing: Default::default(),
        issues: vec![],
    }
}

fn content_visibility(problem: bool) -> crate::content_visibility::ContentVisibilityAnalysis {
    use crate::assessment::{AssessmentLevel, ContentArea, ContentSignal, EvidenceConfidence as C};
    let signal =
        |level| ContentSignal::new(ContentArea::Content, level, C::High, "Signal", "Detail");
    crate::content_visibility::ContentVisibilityAnalysis {
        local_business: if problem {
            vec![
                signal(AssessmentLevel::Warning),
                signal(AssessmentLevel::Violation),
                signal(AssessmentLevel::Pass),
            ]
        } else {
            vec![signal(AssessmentLevel::Pass)]
        },
        ..Default::default()
    }
}

fn presentation(p: Option<SearchExperiencePresentation>) -> Value {
    match p {
        None => Value::Null,
        Some(p) => json!({
            "score": p.score,
            "label": p.label,
            "interpretation": p.interpretation,
            "components": p.components.iter().map(|c| json!({
                "label": c.label,
                "score": c.score,
                "weight_pct": c.weight_pct,
                "explanation": c.explanation,
            })).collect::<Vec<_>>(),
            "warnings": p.warnings,
        }),
    }
}

fn export_search_experience(records: &mut Vec<Value>) {
    let (_, report) = reports().remove(1);
    let context = crate::audit::validation::normalize_checked(&report)
        .expect("fixture report passes the report contract");
    let ai_base = crate::ai_visibility::analyze_ai_visibility(&report);
    let ai = |score: u32| {
        let mut ai = ai_base.clone();
        ai.readability.dimension.score = score;
        ai.citation.dimension.score = score;
        ai.chunks.dimension.score = score;
        ai
    };
    let ai_values = [("none", None), ("90", Some(ai(90))), ("20", Some(ai(20)))];
    let mobile_values = [
        ("none", None),
        ("90", Some(mobile(90))),
        ("20", Some(mobile(20))),
    ];
    let cv_values = [
        ("none", None),
        ("clean", Some(content_visibility(false))),
        ("problem", Some(content_visibility(true))),
    ];
    let ux_names = ["none", "good", "weak", "mid"];
    let ux_values: Vec<_> = ux_names.iter().map(|name| (*name, ux(name))).collect();

    for lang in LANGS {
        let i18n = I18n::new(lang).unwrap();

        // No SEO module: no section.
        let bare_context = context.clone();
        push(
            records,
            "build_search_experience",
            "no seo".to_string(),
            lang,
            presentation(crate::output::search_experience::build_search_experience(
                &bare_context,
                &i18n,
            )),
        );

        for seo_score in [95, 80, 60, 20] {
            for profile in ["none", "article", "media"] {
                for image_led in [false, true] {
                    let seo = seo(seo_score, profile, image_led);
                    for (ux_name, ux) in &ux_values {
                        for (ai_name, ai) in &ai_values {
                            for (mobile_name, mobile) in &mobile_values {
                                for (cv_name, cv) in &cv_values {
                                    let mut ctx = context.clone();
                                    ctx.raw_seo = Some(&seo);
                                    ctx.raw_ux = ux.as_ref();
                                    ctx.raw_ai_visibility = ai.as_ref();
                                    ctx.raw_mobile = mobile.as_ref();
                                    ctx.raw_content_visibility = cv.as_ref();
                                    push(
                                        records,
                                        "build_search_experience",
                                        format!(
                                            "seo={seo_score} profile={profile} image_led={image_led} ux={ux_name} ai={ai_name} mobile={mobile_name} cv={cv_name}"
                                        ),
                                        lang,
                                        presentation(
                                            crate::output::search_experience::build_search_experience(
                                                &ctx, &i18n,
                                            ),
                                        ),
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ─── Findings of the view model ─────────────────────────────────────────────

fn finding(
    template: &FindingGroup,
    index: usize,
    count: usize,
    tier: CriticalityTier,
) -> FindingGroup {
    let mut group = template.clone();
    group.finding.rule_id = format!("rule/{index}");
    group.title = format!("Finding {index}");
    group.finding.occurrence_count = count;
    group.finding.wcag_criterion = "1.4.3".to_string();
    group.criticality_tier = tier;
    group
}

fn findings(template: &FindingGroup, counts: &[usize]) -> Vec<FindingGroup> {
    counts
        .iter()
        .enumerate()
        .map(|(i, count)| finding(template, i, *count, CriticalityTier::Mandatory))
        .collect()
}

/// The finding sets of the problem profile and the root causes: none, a few
/// occurrences, a concentrated set, an evenly spread one and long ones.
const FINDING_SETS: [(&str, &[usize]); 7] = [
    ("none", &[]),
    ("one occurrence", &[1]),
    ("three occurrences", &[2, 1]),
    ("concentrated", &[60, 30]),
    ("spread", &[10, 10, 10, 10, 10, 10, 10, 10, 10, 10]),
    ("six causes", &[9, 8, 7, 6, 5, 4]),
    ("nine causes", &[90, 80, 70, 60, 50, 40, 30, 20, 10]),
];

// ─── Problem profile ────────────────────────────────────────────────────────

/// Module score sets: no weak module, a weak one that is not critical, and one
/// to four critical ones (the names include one without a message).
const MODULE_SETS: [(&str, &[(&str, u32)]); 8] = [
    ("none", &[]),
    ("all good", &[("Accessibility", 99), ("SEO", 90)]),
    ("weak only", &[("Accessibility", 99), ("Performance", 60)]),
    ("one critical", &[("Security", 30), ("SEO", 90)]),
    ("one unknown", &[("Brand New Module", 10)]),
    (
        "two critical",
        &[("Accessibility", 20), ("Security", 30), ("Mobile", 80)],
    ),
    (
        "three critical",
        &[("Accessibility", 12), ("Performance", 34), ("Security", 22)],
    ),
    (
        "four critical",
        &[
            ("Accessibility", 12),
            ("Performance", 34),
            ("Security", 22),
            ("SEO", 10),
        ],
    ),
];

fn export_problem_profile(records: &mut Vec<Value>, lang: &str, i18n: &I18n) {
    let (mut vm, template) = view_model(lang);
    for (modules_name, modules) in MODULE_SETS {
        vm.modules.module_scores = modules
            .iter()
            .map(|(name, score)| {
                crate::audit::normalized::ModuleScoreEntry::new(name, *score, "measured", true)
            })
            .collect();
        for (findings_name, counts) in FINDING_SETS {
            vm.findings.all_findings = findings(&template, counts);
            for structural in [0, 1, 2] {
                vm.management_risks = vec![ManagementRiskKind::Project {
                    count: structural,
                    pages: None,
                }];
                push(
                    records,
                    "render_problem_profile",
                    format!(
                        "modules={modules_name} findings={findings_name} structural={structural}"
                    ),
                    lang,
                    components(render_problem_profile(bare(), &vm, i18n)),
                );
            }
        }
    }
    for (findings_name, counts) in FINDING_SETS {
        vm.findings.all_findings = findings(&template, counts);
        push(
            records,
            "build_problem_concentration_note",
            findings_name.to_string(),
            lang,
            json!(build_problem_concentration_note(&vm, i18n)),
        );
        push(
            records,
            "build_overall_leverage_note",
            findings_name.to_string(),
            lang,
            json!(build_overall_leverage_note(&vm, i18n)),
        );
    }
}

// ─── Root causes and roadmap ────────────────────────────────────────────────

fn security(issues: usize) -> SecurityPresentation {
    SecurityPresentation {
        score: 80,
        grade: "B".to_string(),
        interpretation: String::new(),
        band_label: String::new(),
        band_score: 80,
        core_headers_present: 0,
        core_headers_total: 0,
        band_note: None,
        headers: vec![],
        score_rows: vec![],
        ssl_info: vec![],
        issues: (0..issues)
            .map(|i| {
                (
                    format!("Security issue {i}"),
                    Severity::High,
                    format!("Security message {i}."),
                )
            })
            .collect(),
        recommendations: vec![],
        protection: vec![],
        has_waf: false,
        has_cdn: false,
    }
}

fn mobile_presentation(issues: usize) -> MobilePresentation {
    MobilePresentation {
        score: 70,
        interpretation: String::new(),
        viewport: vec![],
        touch_targets: vec![],
        font_analysis: vec![],
        content_sizing: vec![],
        issues: (0..issues)
            .map(|i| {
                (
                    "touch_targets".to_string(),
                    Severity::Medium,
                    format!("Mobile message {i}."),
                )
            })
            .collect(),
    }
}

fn roadmap_item(
    rule_id: &str,
    occurrence_count: usize,
    benefit: &str,
    risk_effect: &str,
    hint: &str,
) -> RoadmapItemData {
    RoadmapItemData {
        action: "Do the action".to_string(),
        hint: hint.to_string(),
        role: "Developer".to_string(),
        priority: "High".to_string(),
        execution_priority: "Soon".to_string(),
        effort: "Low".to_string(),
        benefit: benefit.to_string(),
        user_effect: "User effect".to_string(),
        risk_effect: risk_effect.to_string(),
        occurrence_count,
        rule_id: rule_id.to_string(),
        leverage: "High".to_string(),
    }
}

fn export_root_causes(records: &mut Vec<Value>, lang: &str, i18n: &I18n) {
    let (mut vm, template) = view_model(lang);

    // No mandatory finding: the evidence classes decide what is said.
    vm.findings.all_findings = vec![];
    for (warnings, manual_checks) in [(0, 0), (1, 0), (0, 1), (2, 3), (1_234_567, 1_234_567)] {
        vm.findings.evidence = EvidenceClasses {
            confirmed_violations: 0,
            warnings,
            manual_checks,
        };
        push(
            records,
            "render_root_cause_analysis",
            format!("no findings warnings={warnings} manual={manual_checks}"),
            lang,
            components(render_root_cause_analysis(bare(), &vm, i18n)),
        );
    }
    // Only an optimization finding: not a root cause.
    let optimization = finding(&template, 0, 5, CriticalityTier::Optimization);
    vm.findings.all_findings = vec![optimization];
    push(
        records,
        "render_root_cause_analysis",
        "optimization only".to_string(),
        lang,
        components(render_root_cause_analysis(bare(), &vm, i18n)),
    );

    for (name, counts) in FINDING_SETS {
        vm.findings.all_findings = findings(&template, counts);
        push(
            records,
            "render_root_cause_analysis",
            format!("findings={name}"),
            lang,
            components(render_root_cause_analysis(bare(), &vm, i18n)),
        );
    }
    vm.findings.all_findings = findings(&template, &[1_234_567, 2]);
    push(
        records,
        "render_root_cause_analysis",
        "findings=large count".to_string(),
        lang,
        components(render_root_cause_analysis(bare(), &vm, i18n)),
    );
}

fn export_roadmap(records: &mut Vec<Value>, lang: &str, i18n: &I18n) {
    let (mut vm, template) = view_model(lang);
    vm.actions.roadmap_columns = vec![];

    // No column: the other modules' measures decide what is said.
    for (security_issues, mobile_issues) in [(0, 0), (1, 0), (1, 1), (2, 3)] {
        vm.module_details.security = (security_issues > 0).then(|| security(security_issues));
        vm.module_details.mobile = (mobile_issues > 0).then(|| mobile_presentation(mobile_issues));
        for (name, counts) in FINDING_SETS {
            vm.findings.all_findings = findings(&template, counts);
            push(
                records,
                "render_timeframe_roadmap",
                format!(
                    "no columns security={security_issues} mobile={mobile_issues} findings={name}"
                ),
                lang,
                components(render_timeframe_roadmap(bare(), &vm, i18n)),
            );
        }
    }

    // One column: every combination of the texts an item can carry.
    vm.module_details.security = None;
    vm.module_details.mobile = None;
    for (name, counts) in [
        ("concentrated", &[60usize, 30, 10][..]),
        ("three occurrences", &[2, 1][..]),
    ] {
        vm.findings.all_findings = findings(&template, counts);
        for benefit in ["", "Benefit."] {
            for risk_effect in ["", "Risk effect."] {
                for hint in ["", "Hint."] {
                    for occurrences in [1, 5, 1_234_567] {
                        for rule_id in ["rule/0", "rule/2", "rule/other"] {
                            vm.actions.roadmap_columns = vec![RoadmapColumnData {
                                title: "Column".to_string(),
                                description: "Column description.".to_string(),
                                accent_color: "#000000".to_string(),
                                items: vec![roadmap_item(
                                    rule_id,
                                    occurrences,
                                    benefit,
                                    risk_effect,
                                    hint,
                                )],
                            }];
                            push(
                                records,
                                "render_timeframe_roadmap",
                                format!(
                                    "findings={name} benefit={benefit:?} risk={risk_effect:?} hint={hint:?} occurrences={occurrences} rule={rule_id}"
                                ),
                                lang,
                                components(render_timeframe_roadmap(bare(), &vm, i18n)),
                            );
                        }
                    }
                }
            }
        }
    }

    // A column and the cards of the other modules' measures.
    vm.module_details.security = Some(security(2));
    vm.module_details.mobile = Some(mobile_presentation(1));
    push(
        records,
        "render_timeframe_roadmap",
        "column with module measures".to_string(),
        lang,
        components(render_timeframe_roadmap(bare(), &vm, i18n)),
    );
}

// ─── WCAG coverage ──────────────────────────────────────────────────────────

fn export_wcag_coverage(records: &mut Vec<Value>, lang: &str, i18n: &I18n) {
    for (name, report) in reports() {
        push(
            records,
            "render_wcag_coverage_section",
            name.to_string(),
            lang,
            components(render_wcag_coverage_section(bare(), &report, i18n)),
        );
    }
}

/// Every record of the export, in both languages.
pub(super) fn records() -> Vec<Value> {
    let mut records: Vec<Value> = Vec::new();
    export_search_experience(&mut records);
    for lang in LANGS {
        let i18n = I18n::new(lang).unwrap();
        export_problem_profile(&mut records, lang, &i18n);
        export_root_causes(&mut records, lang, &i18n);
        export_roadmap(&mut records, lang, &i18n);
        export_wcag_coverage(&mut records, lang, &i18n);
    }
    records
}

#[test]
#[ignore = "writes reports/pdf-analysis-texts.json; run before and after a change of the text storage"]
fn export_pdf_analysis_texts() {
    let records = records();
    let path = std::env::var("PDF_ANALYSIS_TEXT_EXPORT").unwrap_or_else(|_| {
        format!(
            "{}/reports/pdf-analysis-texts.json",
            env!("CARGO_MANIFEST_DIR")
        )
    });
    std::fs::create_dir_all(std::path::Path::new(&path).parent().unwrap()).unwrap();
    std::fs::write(&path, serde_json::to_string_pretty(&records).unwrap()).unwrap();
    println!("{} analysis text records written to {path}", records.len());
}
