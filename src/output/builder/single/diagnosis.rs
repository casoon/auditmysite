use crate::audit::normalized::{NormalizedReport, SeverityCounts};
use crate::i18n::{text, text_args};
use crate::output::report_model::{
    CriticalityTier, DiagnosisBlock, FindingCluster, FindingCriticalityGroup, FindingGroup,
    FindingSeverityTier, FindingSummary,
};
use crate::wcag::Severity;
use fluent_bundle::FluentValue;

#[cfg(test)]
mod text_export;
#[cfg(test)]
mod text_guard;

pub(super) fn build_finding_summary(
    locale: &str,
    counts: &SeverityCounts,
    audit_summary: &crate::audit::summary::AuditSummary,
) -> FindingSummary {
    let en = locale == "en";
    let cross_impact_notes = audit_summary
        .cross_impacts
        .iter()
        .map(|c| format!("{}: {}", c.dimensions, c.description))
        .collect();
    let issue_pattern_label = text(
        match &audit_summary.issue_pattern {
            crate::audit::summary::IssuePattern::Minimal => "diagnosis-pattern-minimal",
            crate::audit::summary::IssuePattern::SingleDominant => {
                "diagnosis-pattern-single-dominant"
            }
            crate::audit::summary::IssuePattern::Clustered => "diagnosis-pattern-clustered",
            crate::audit::summary::IssuePattern::Scattered => "diagnosis-pattern-scattered",
        },
        en,
    );
    FindingSummary {
        total: counts.total,
        critical: counts.critical,
        high: counts.high,
        medium: counts.medium,
        low: counts.low,
        verdict: audit_summary.verdict_intro.clone(),
        dominant_issue_note: audit_summary.dominant_issue_note.clone(),
        cross_impact_notes,
        issue_pattern_label,
    }
}

pub(super) fn build_severity_tiers(
    locale: &str,
    findings: &[FindingGroup],
) -> Vec<FindingSeverityTier> {
    let en = locale == "en";
    [
        Severity::Critical,
        Severity::High,
        Severity::Medium,
        Severity::Low,
    ]
    .iter()
    .filter_map(|sev| {
        let tier_findings: Vec<FindingGroup> = findings
            .iter()
            .filter(|f| f.finding.severity == *sev)
            .cloned()
            .collect();
        if tier_findings.is_empty() {
            return None;
        }
        let total_occurrences = tier_findings
            .iter()
            .map(|f| f.finding.occurrence_count)
            .sum();
        Some(FindingSeverityTier {
            severity: *sev,
            label: severity_label(*sev, en),
            findings: tier_findings,
            total_occurrences,
        })
    })
    .collect()
}

pub(super) fn build_criticality_groups(
    locale: &str,
    findings: &[FindingGroup],
) -> Vec<FindingCriticalityGroup> {
    let en = locale == "en";

    [CriticalityTier::Mandatory, CriticalityTier::Optimization]
        .iter()
        .filter_map(|tier| {
            let tier_findings: Vec<FindingGroup> = findings
                .iter()
                .filter(|f| f.criticality_tier == *tier)
                .cloned()
                .collect();
            if tier_findings.is_empty() {
                return None;
            }
            let by_severity = build_severity_tiers(locale, &tier_findings);
            let total_findings = tier_findings.len();
            let total_occurrences = tier_findings
                .iter()
                .map(|f| f.finding.occurrence_count)
                .sum();
            let (label, eyebrow, intro) = match tier {
                CriticalityTier::Mandatory => (
                    "diagnosis-tier-mandatory",
                    "diagnosis-tier-mandatory-eyebrow",
                    "diagnosis-tier-mandatory-intro",
                ),
                CriticalityTier::Optimization => (
                    "diagnosis-tier-optimization",
                    "diagnosis-tier-optimization-eyebrow",
                    "diagnosis-tier-optimization-intro",
                ),
            };
            Some(FindingCriticalityGroup {
                tier: *tier,
                label: text(label, en),
                eyebrow: text(eyebrow, en),
                intro: text(intro, en),
                by_severity,
                total_findings,
                total_occurrences,
            })
        })
        .collect()
}

pub(super) fn build_thematic_clusters(
    locale: &str,
    findings: &[FindingGroup],
) -> Vec<FindingCluster> {
    use std::collections::BTreeMap;

    let en = locale == "en";
    let mut by_dimension: BTreeMap<String, Vec<&FindingGroup>> = BTreeMap::new();
    for f in findings {
        by_dimension
            .entry(f.finding.dimension.clone())
            .or_default()
            .push(f);
    }

    let mut clusters: Vec<FindingCluster> = by_dimension
        .into_iter()
        .filter(|(_, groups)| groups.len() >= 2)
        .map(|(dimension, groups)| {
            let worst_severity = groups
                .iter()
                .map(|g| g.finding.severity)
                .max()
                .unwrap_or(crate::wcag::Severity::Low);
            let occurrence_total: usize = groups.iter().map(|g| g.finding.occurrence_count).sum();
            let label = dimension_cluster_label(en, &dimension, groups.len());
            FindingCluster {
                label,
                dimension: dimension.clone(),
                finding_count: groups.len(),
                occurrence_total,
                severity_label: severity_label(worst_severity, en),
                finding_titles: groups.iter().map(|g| g.title.clone()).collect(),
            }
        })
        .collect();

    clusters.sort_by_key(|c| std::cmp::Reverse(c.occurrence_total));
    clusters
}

/// A finding category (the canonical English `dimension` of a finding) in the
/// run language, through the same `module-*` entries as the module names.
pub(super) fn category_name(dimension: &str, en: bool) -> String {
    if en {
        return dimension.to_string();
    }
    match crate::i18n::I18n::new(if en { "en" } else { "de" }) {
        Ok(i18n) => crate::output::builder::helpers::localized_module_name(dimension, &i18n),
        Err(_) => dimension.to_string(),
    }
}

/// A severity in the run language.
fn severity_label(severity: Severity, en: bool) -> String {
    text(
        match severity {
            Severity::Critical => "severity-critical",
            Severity::High => "severity-high",
            Severity::Medium => "severity-medium",
            Severity::Low => "severity-low",
        },
        en,
    )
}

fn dimension_cluster_label(en: bool, dimension: &str, count: usize) -> String {
    let findings = FluentValue::from(crate::i18n::count_text("count-findings", count, en));
    let key = match dimension {
        "Accessibility" => "diagnosis-cluster-accessibility",
        "SEO" => "diagnosis-cluster-seo",
        "Performance" => "diagnosis-cluster-performance",
        "Security" => "diagnosis-cluster-security",
        "Mobile" => "diagnosis-cluster-mobile",
        other => {
            return text_args(
                "diagnosis-cluster-other",
                &[
                    ("category", FluentValue::from(category_name(other, en))),
                    ("findings", findings),
                ],
                en,
            )
        }
    };
    text_args(key, &[("findings", findings)], en)
}

pub(super) fn build_diagnosis_block(
    locale: &str,
    normalized: &NormalizedReport,
    audit_summary: &crate::audit::summary::AuditSummary,
) -> DiagnosisBlock {
    use std::collections::BTreeMap;
    let en = locale == "en";

    let section_title = text("diagnosis-section-title", en);

    let (pattern_label, pattern_description) = match &audit_summary.issue_pattern {
        crate::audit::summary::IssuePattern::Minimal => (
            "diagnosis-pattern-minimal",
            "diagnosis-pattern-minimal-description",
        ),
        crate::audit::summary::IssuePattern::SingleDominant => (
            "diagnosis-pattern-single-dominant-problem",
            "diagnosis-pattern-single-dominant-description",
        ),
        crate::audit::summary::IssuePattern::Clustered => (
            "diagnosis-pattern-clustered",
            "diagnosis-pattern-clustered-description",
        ),
        crate::audit::summary::IssuePattern::Scattered => (
            "diagnosis-pattern-distributed",
            "diagnosis-pattern-scattered-description",
        ),
    };
    let pattern_label = text(pattern_label, en);
    let pattern_description = text(pattern_description, en);

    let mut dim_map: BTreeMap<String, (usize, crate::wcag::Severity)> = BTreeMap::new();
    for f in &normalized.findings {
        let dim = f.dimension.clone();
        let entry = dim_map
            .entry(dim)
            .or_insert((0, crate::wcag::Severity::Low));
        entry.0 += f.occurrence_count;
        if f.severity > entry.1 {
            entry.1 = f.severity;
        }
    }
    let mut category_breakdown: Vec<(String, usize, String)> = dim_map
        .into_iter()
        .map(|(dim, (count, sev))| (category_name(&dim, en), count, severity_label(sev, en)))
        .collect();
    category_breakdown.sort_by_key(|c| std::cmp::Reverse(c.1));

    let dominant_issue = audit_summary
        .dominant_issue
        .as_ref()
        .map(|d| d.title.clone());

    let clusters = {
        let mut by_dim: BTreeMap<String, Vec<(String, usize, crate::wcag::Severity)>> =
            BTreeMap::new();
        for f in &normalized.findings {
            // Stored title is canonical English (#406); re-derive the localized
            // title (taxonomy + SEO headings) for non-English reports.
            let title = crate::output::builder::actions::localized_finding_text(locale, f).0;
            by_dim.entry(f.dimension.clone()).or_default().push((
                title,
                f.occurrence_count,
                f.severity,
            ));
        }
        let mut result: Vec<FindingCluster> = by_dim
            .into_iter()
            .filter(|(_, items)| items.len() >= 2)
            .map(|(dim, items)| {
                let worst = items
                    .iter()
                    .map(|(_, _, s)| *s)
                    .max()
                    .unwrap_or(crate::wcag::Severity::Low);
                let total_occ: usize = items.iter().map(|(_, c, _)| c).sum();
                let sev_label = severity_label(worst, en);
                FindingCluster {
                    label: dimension_cluster_label(en, &dim, items.len()),
                    dimension: dim.clone(),
                    finding_count: items.len(),
                    occurrence_total: total_occ,
                    severity_label: sev_label,
                    finding_titles: items.into_iter().map(|(t, _, _)| t).collect(),
                }
            })
            .collect();
        result.sort_by_key(|c| std::cmp::Reverse(c.occurrence_total));
        result
    };

    DiagnosisBlock {
        section_title,
        pattern_label,
        pattern_description,
        is_systematic: audit_summary.is_systematic,
        category_breakdown,
        dominant_issue,
        verdict_intro: audit_summary.verdict_intro.clone(),
        clusters,
    }
}
