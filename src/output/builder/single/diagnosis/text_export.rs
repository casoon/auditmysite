//! Text export of the findings overview and the system diagnosis of the
//! single report (`diagnosis.rs`), slice 12 of #936: every text the module
//! builds, across an enumerated input space in both languages, as records in
//! `reports/diagnosis-texts.json` (gitignored; the path can be set with
//! `DIAGNOSIS_TEXT_EXPORT`). The scenarios cover the four issue patterns, the
//! four severities, both criticality tiers, the named and the other finding
//! categories (with two and three findings, and one without a cluster) and an
//! empty finding list. Run it before and after a change of how the texts are
//! stored and compare the two files byte for byte.

use super::*;
use crate::audit::summary::{AuditSummary, IssuePattern};
use crate::audit::AuditReport;
use crate::model::{ReportLevel, WcagLevel};
use crate::output::report_model::ReportConfig;
use crate::wcag::{Severity, Violation, WcagResults};
use serde_json::{json, Value};

const LANGS: [&str; 2] = ["en", "de"];

/// A finding of a scenario: category, severity, occurrences and tier.
type Finding = (String, Severity, usize, CriticalityTier);

const SEVERITIES: [Severity; 4] = [
    Severity::Critical,
    Severity::High,
    Severity::Medium,
    Severity::Low,
];

const PATTERNS: [IssuePattern; 4] = [
    IssuePattern::Minimal,
    IssuePattern::SingleDominant,
    IssuePattern::Clustered,
    IssuePattern::Scattered,
];

/// The finding categories: the five with their own cluster label, the
/// indicator modules and a category without a module name.
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

fn normalized() -> NormalizedReport {
    crate::audit::validation::normalize_checked(&report())
        .expect("fixture report passes the report contract")
        .normalized
}

/// One finding group of the page in the run language, as a template.
fn template_group(lang: &str) -> FindingGroup {
    let report = report();
    let checked = crate::audit::validation::normalize_checked(&report)
        .expect("fixture report passes the report contract");
    let config = ReportConfig {
        level: ReportLevel::Standard,
        locale: lang.to_string(),
        ..ReportConfig::default()
    };
    crate::output::builder::build_view_model(&checked, &config)
        .findings
        .all_findings
        .remove(0)
}

/// The findings of a scenario as (category, severity, occurrences, tier):
/// every category with two or three findings, the first of them with
/// `worst`, plus one category with a single finding.
fn spread(worst: Severity) -> Vec<Finding> {
    let mut out = Vec::new();
    for (i, dimension) in DIMENSIONS.iter().enumerate() {
        for k in 0..(2 + i % 2) {
            let severity = if k == 0 { worst } else { Severity::Low };
            let tier = if i % 2 == 0 {
                CriticalityTier::Mandatory
            } else {
                CriticalityTier::Optimization
            };
            out.push((dimension.to_string(), severity, 10 * i + k + 1, tier));
        }
    }
    out.push((
        "Lonely".to_string(),
        Severity::Medium,
        1,
        CriticalityTier::Optimization,
    ));
    out
}

/// The scenarios of the finding lists: a name and the findings.
fn finding_scenarios() -> Vec<(String, Vec<Finding>)> {
    let mut out = vec![("empty".to_string(), Vec::new())];
    out.push((
        "one-per-severity".to_string(),
        SEVERITIES
            .iter()
            .enumerate()
            .map(|(i, s)| {
                (
                    "Accessibility".to_string(),
                    *s,
                    i + 1,
                    CriticalityTier::Mandatory,
                )
            })
            .collect(),
    ));
    out.push((
        "optimization-only".to_string(),
        vec![
            (
                "SEO".to_string(),
                Severity::High,
                3,
                CriticalityTier::Optimization,
            ),
            (
                "SEO".to_string(),
                Severity::Low,
                2,
                CriticalityTier::Optimization,
            ),
        ],
    ));
    for worst in SEVERITIES {
        out.push((format!("spread worst={worst:?}"), spread(worst)));
    }
    out
}

fn groups(template: &FindingGroup, findings: &[Finding]) -> Vec<FindingGroup> {
    findings
        .iter()
        .enumerate()
        .map(|(i, (dimension, severity, occurrences, tier))| {
            let mut group = template.clone();
            group.title = format!("Finding {i}");
            group.finding.dimension = dimension.clone();
            group.finding.severity = *severity;
            group.finding.occurrence_count = *occurrences;
            group.criticality_tier = *tier;
            group
        })
        .collect()
}

fn report_with(findings: &[Finding]) -> NormalizedReport {
    let mut report = normalized();
    let template = report.findings[0].clone();
    report.findings = findings
        .iter()
        .map(|(dimension, severity, occurrences, _)| {
            let mut finding = template.clone();
            finding.dimension = dimension.clone();
            finding.severity = *severity;
            finding.occurrence_count = *occurrences;
            finding
        })
        .collect();
    report
}

fn summary(report: &NormalizedReport, lang: &str, pattern: &IssuePattern) -> AuditSummary {
    let mut summary = crate::audit::summary::analyze_with_locale(report, lang);
    summary.issue_pattern = pattern.clone();
    summary
}

fn tiers_record(tiers: &[FindingSeverityTier]) -> Value {
    tiers
        .iter()
        .map(|t| {
            json!({
                "severity": format!("{:?}", t.severity),
                "label": t.label,
                "findings": t.findings.len(),
                "occurrences": t.total_occurrences,
            })
        })
        .collect()
}

fn clusters_record(clusters: &[FindingCluster]) -> Value {
    clusters
        .iter()
        .map(|c| {
            json!({
                "label": c.label,
                "dimension": c.dimension,
                "findings": c.finding_count,
                "occurrences": c.occurrence_total,
                "severity_label": c.severity_label,
                "titles": c.finding_titles,
            })
        })
        .collect()
}

fn diagnosis_record(block: &DiagnosisBlock) -> Value {
    json!({
        "section_title": block.section_title,
        "pattern_label": block.pattern_label,
        "pattern_description": block.pattern_description,
        "is_systematic": block.is_systematic,
        "category_breakdown": block.category_breakdown,
        "dominant_issue": block.dominant_issue,
        "verdict_intro": block.verdict_intro,
        "clusters": clusters_record(&block.clusters),
    })
}

/// Every record of the export, in both languages.
pub(super) fn records() -> Vec<Value> {
    let mut records: Vec<Value> = Vec::new();
    for lang in LANGS {
        let template = template_group(lang);
        for (name, findings) in finding_scenarios() {
            let groups = groups(&template, &findings);
            records.push(json!([
                "build_severity_tiers",
                name,
                lang,
                tiers_record(&build_severity_tiers(lang, &groups))
            ]));
            records.push(json!([
                "build_criticality_groups",
                name,
                lang,
                build_criticality_groups(lang, &groups)
                    .iter()
                    .map(|g| json!({
                        "tier": format!("{:?}", g.tier),
                        "label": g.label,
                        "eyebrow": g.eyebrow,
                        "intro": g.intro,
                        "by_severity": tiers_record(&g.by_severity),
                        "findings": g.total_findings,
                        "occurrences": g.total_occurrences,
                    }))
                    .collect::<Vec<_>>()
            ]));
            records.push(json!([
                "build_thematic_clusters",
                name,
                lang,
                clusters_record(&build_thematic_clusters(lang, &groups))
            ]));
            let report = report_with(&findings);
            for pattern in PATTERNS {
                let input = format!("{name} pattern={pattern:?}");
                let audit_summary = summary(&report, lang, &pattern);
                let finding_summary =
                    build_finding_summary(lang, &report.occurrence_counts, &audit_summary);
                records.push(json!([
                    "build_finding_summary",
                    input,
                    lang,
                    {
                        "total": finding_summary.total,
                        "verdict": finding_summary.verdict,
                        "dominant_issue_note": finding_summary.dominant_issue_note,
                        "cross_impact_notes": finding_summary.cross_impact_notes,
                        "issue_pattern_label": finding_summary.issue_pattern_label,
                    }
                ]));
                records.push(json!([
                    "build_diagnosis_block",
                    input,
                    lang,
                    diagnosis_record(&build_diagnosis_block(lang, &report, &audit_summary))
                ]));
            }
        }
    }
    records
}

#[test]
#[ignore = "writes reports/diagnosis-texts.json; run before and after a change of the text storage"]
fn export_diagnosis_texts() {
    let records = records();
    let path = std::env::var("DIAGNOSIS_TEXT_EXPORT").unwrap_or_else(|_| {
        format!(
            "{}/reports/diagnosis-texts.json",
            env!("CARGO_MANIFEST_DIR")
        )
    });
    std::fs::create_dir_all(std::path::Path::new(&path).parent().unwrap()).unwrap();
    std::fs::write(&path, serde_json::to_string_pretty(&records).unwrap()).unwrap();
    println!("{} diagnosis text records written to {path}", records.len());
}
