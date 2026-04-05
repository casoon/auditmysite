//! Normalized Audit Model — Single Source of Truth für alle Outputs
//!
//! Transformiert den rohen AuditReport in ein normalisiertes Modell mit:
//! - Korrigiertem Score (nach Suppressions)
//! - Taxonomie-angereichertem Findings
//! - Einheitlicher Severity-Terminologie
//! - Konsistenter Grade/Certificate-Berechnung

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::audit::report::{AuditReport, PerformanceResults};
use crate::audit::scoring::AccessibilityScorer;
use crate::cli::WcagLevel;
use crate::dark_mode::DarkModeAnalysis;
use crate::mobile::MobileFriendliness;
use crate::security::SecurityAnalysis;
use crate::seo::SeoAnalysis;
use crate::taxonomy::{ReportVisibility, RuleLookup, Scaling, Severity};
use crate::wcag::WcagResults;

/// Normalisiertes Audit-Modell — einzige Score-Quelle für alle Output-Formate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedReport {
    pub url: String,
    pub wcag_level: WcagLevel,
    pub timestamp: DateTime<Utc>,
    pub duration_ms: u64,
    pub nodes_analyzed: usize,

    /// Korrigierter Score (nach Suppressions, gerundet)
    pub score: u32,
    /// Grade aus korrigiertem Score
    pub grade: String,
    /// Certificate aus korrigiertem Score
    pub certificate: String,

    /// Normalisierte, gruppierte Findings mit Taxonomie-Feldern
    pub findings: Vec<NormalizedFinding>,
    /// Severity-Zähler (einheitliche Terminologie)
    pub severity_counts: SeverityCounts,

    /// Modul-Scores
    pub module_scores: Vec<ModuleScoreEntry>,
    /// Gewichteter Gesamtscore über alle aktiven Module
    pub overall_score: u32,

    /// Rohdaten für Modul-Details (nicht serialisiert)
    #[serde(skip)]
    pub raw_performance: Option<PerformanceResults>,
    #[serde(skip)]
    pub raw_seo: Option<SeoAnalysis>,
    #[serde(skip)]
    pub raw_security: Option<SecurityAnalysis>,
    #[serde(skip)]
    pub raw_mobile: Option<MobileFriendliness>,
    #[serde(skip)]
    pub raw_ux: Option<crate::ux::UxAnalysis>,
    #[serde(skip)]
    pub raw_dark_mode: Option<DarkModeAnalysis>,
    #[serde(skip)]
    pub raw_wcag: WcagResults,
}

/// Einheitliche Severity-Zähler
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeverityCounts {
    pub critical: usize,
    pub high: usize,
    pub medium: usize,
    pub low: usize,
    pub total: usize,
}

/// Ein normalisiertes Finding — gruppiert nach Regel, mit Taxonomie-Feldern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedFinding {
    /// Taxonomie-Regel-ID (z.B. "a11y.alt_text.missing")
    pub rule_id: String,
    /// WCAG-Kriterium (z.B. "1.1.1")
    pub wcag_criterion: String,
    /// WCAG-Level (z.B. "A", "AA")
    pub wcag_level: String,

    /// Audit-Dimension
    pub dimension: String,
    /// Subkategorie
    pub subcategory: String,
    /// Issue-Klasse
    pub issue_class: String,
    /// Schweregrad
    pub severity: Severity,
    /// Auswirkung auf den Nutzer
    pub user_impact: String,
    /// Technische Auswirkung
    pub technical_impact: String,
    /// Strukturierter Score-Impact
    pub score_impact: ScoreImpactData,
    /// Report-Sichtbarkeit
    #[serde(skip)]
    pub report_visibility: ReportVisibilityData,
    /// Aggregationsschlüssel (= rule_id)
    pub aggregation_key: String,

    /// Titel der Regel
    pub title: String,
    /// Beschreibung
    pub description: String,
    /// Anzahl Vorkommen
    pub occurrence_count: usize,
    /// Prioritätswert für Maßnahmenplanung (impact × reach / effort)
    pub priority_score: f32,
    /// Einzelne Vorkommen
    pub occurrences: Vec<OccurrenceDetail>,
}

/// Strukturierte Darstellung des Score-Impacts für JSON/API-Verbraucher
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreImpactData {
    pub base_penalty: f32,
    pub max_penalty: f32,
    pub scaling: String,
}

/// Kopie der ReportVisibility für Serialize-Kontext
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ReportVisibilityData {
    pub executive: bool,
    pub standard: bool,
    pub technical: bool,
}

impl From<&ReportVisibility> for ReportVisibilityData {
    fn from(rv: &ReportVisibility) -> Self {
        Self {
            executive: rv.executive,
            standard: rv.standard,
            technical: rv.technical,
        }
    }
}

impl Default for ReportVisibilityData {
    fn default() -> Self {
        Self {
            executive: true,
            standard: true,
            technical: true,
        }
    }
}

/// Detail eines einzelnen Vorkommens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OccurrenceDetail {
    pub node_id: String,
    pub message: String,
    pub selector: Option<String>,
    pub fix_suggestion: Option<String>,
    /// Raw outer HTML of the affected element
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub html_snippet: Option<String>,
    /// Concrete code fix — the corrected HTML
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suggested_code: Option<String>,
}

/// Score-Eintrag pro Modul
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleScoreEntry {
    pub name: String,
    pub score: u32,
    pub grade: String,
    pub weight_pct: u32,
}

/// Normalisiert einen rohen AuditReport.
///
/// - Gruppert Violations nach Regel-ID
/// - Reichert mit Taxonomie-Feldern an (via RuleLookup)
/// - Wendet Score-Korrekturen an (3.1.1-Suppression)
/// - Berechnet Grade/Certificate aus korrigiertem Score
pub fn normalize(report: &AuditReport) -> NormalizedReport {
    let violations = &report.wcag_results.violations;

    // Detect 3.1.1 suppression
    let suppress_lang = report.seo.as_ref().is_some_and(|s| s.technical.has_lang);
    let had_311 = violations.iter().any(|v| v.rule == "3.1.1");

    // Group violations by rule ID
    let mut groups: HashMap<&str, Vec<&crate::wcag::Violation>> = HashMap::new();
    for v in violations {
        // Skip suppressed 3.1.1 violations
        if suppress_lang && v.rule == "3.1.1" {
            continue;
        }
        groups.entry(&v.rule).or_default().push(v);
    }

    // Build normalized findings
    let mut findings: Vec<NormalizedFinding> = groups
        .into_iter()
        .map(|(rule_id, violations)| {
            let first = violations[0];
            let taxonomy_rule = RuleLookup::by_legacy_wcag_id(rule_id);

            let (
                tax_id,
                dimension,
                subcategory,
                issue_class,
                user_impact,
                technical_impact,
                score_impact,
                visibility,
            ) = if let Some(rule) = taxonomy_rule {
                (
                    rule.id.to_string(),
                    rule.dimension.label().to_string(),
                    rule.subcategory.label().to_string(),
                    rule.issue_class.label().to_string(),
                    rule.user_impact.to_string(),
                    rule.technical_impact.to_string(),
                    ScoreImpactData {
                        base_penalty: rule.score_impact.base_penalty,
                        max_penalty: rule.score_impact.max_penalty,
                        scaling: match rule.score_impact.occurrence_scaling {
                            Scaling::Logarithmic => "logarithmic".to_string(),
                            Scaling::Linear => "linear".to_string(),
                            Scaling::Fixed => "fixed".to_string(),
                        },
                    },
                    ReportVisibilityData::from(&rule.report_visibility),
                )
            } else {
                (
                    format!("unknown.{}", rule_id),
                    "Accessibility".to_string(),
                    "Unbekannt".to_string(),
                    "Unbekannt".to_string(),
                    String::new(),
                    String::new(),
                    ScoreImpactData {
                        base_penalty: 0.0,
                        max_penalty: 0.0,
                        scaling: "unknown".to_string(),
                    },
                    ReportVisibilityData::default(),
                )
            };

            let occurrences: Vec<OccurrenceDetail> = violations
                .iter()
                .map(|v| OccurrenceDetail {
                    node_id: v.node_id.clone(),
                    message: v.message.clone(),
                    selector: v.selector.clone(),
                    fix_suggestion: v.fix_suggestion.clone(),
                    html_snippet: v.html_snippet.clone(),
                    suggested_code: v.suggested_code.clone(),
                })
                .collect();

            let occurrence_count = violations.len();
            let priority_score =
                calculate_priority_score(first.severity, occurrence_count, &tax_id);

            NormalizedFinding {
                rule_id: tax_id.clone(),
                wcag_criterion: rule_id.to_string(),
                wcag_level: first.level.to_string(),
                dimension,
                subcategory,
                issue_class,
                severity: first.severity,
                user_impact,
                technical_impact,
                score_impact,
                report_visibility: visibility,
                aggregation_key: tax_id,
                title: first.rule_name.clone(),
                description: first.message.clone(),
                occurrence_count,
                priority_score,
                occurrences,
            }
        })
        .collect();

    // Sort by priority score (highest first), then by severity
    findings.sort_by(|a, b| {
        b.priority_score
            .partial_cmp(&a.priority_score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| b.severity.cmp(&a.severity))
    });

    // Calculate corrected score
    let mut corrected_score = report.score;
    if suppress_lang && had_311 {
        corrected_score += 12.5;
        corrected_score = corrected_score.clamp(0.0, 100.0);
    }
    let score = corrected_score.round() as u32;
    let grade = AccessibilityScorer::calculate_grade(corrected_score).to_string();
    let certificate = AccessibilityScorer::calculate_certificate(corrected_score).to_string();

    // Severity counts from normalized findings
    let severity_counts = SeverityCounts {
        critical: findings
            .iter()
            .filter(|f| f.severity == Severity::Critical)
            .map(|f| f.occurrence_count)
            .sum(),
        high: findings
            .iter()
            .filter(|f| f.severity == Severity::High)
            .map(|f| f.occurrence_count)
            .sum(),
        medium: findings
            .iter()
            .filter(|f| f.severity == Severity::Medium)
            .map(|f| f.occurrence_count)
            .sum(),
        low: findings
            .iter()
            .filter(|f| f.severity == Severity::Low)
            .map(|f| f.occurrence_count)
            .sum(),
        total: findings.iter().map(|f| f.occurrence_count).sum(),
    };

    // Module scores
    let mut module_scores = Vec::new();

    module_scores.push(ModuleScoreEntry {
        name: "Accessibility".to_string(),
        score,
        grade: grade.clone(),
        weight_pct: 40,
    });

    if let Some(ref perf) = report.performance {
        module_scores.push(ModuleScoreEntry {
            name: "Performance".to_string(),
            score: perf.score.overall,
            grade: AccessibilityScorer::calculate_grade(perf.score.overall as f32).to_string(),
            weight_pct: 20,
        });
    }
    if let Some(ref seo) = report.seo {
        module_scores.push(ModuleScoreEntry {
            name: "SEO".to_string(),
            score: seo.score,
            grade: AccessibilityScorer::calculate_grade(seo.score as f32).to_string(),
            weight_pct: 20,
        });
    }
    if let Some(ref sec) = report.security {
        module_scores.push(ModuleScoreEntry {
            name: "Security".to_string(),
            score: sec.score,
            grade: sec.grade.clone(),
            weight_pct: 10,
        });
    }
    if let Some(ref mob) = report.mobile {
        module_scores.push(ModuleScoreEntry {
            name: "Mobile".to_string(),
            score: mob.score,
            grade: AccessibilityScorer::calculate_grade(mob.score as f32).to_string(),
            weight_pct: 10,
        });
    }
    if let Some(ref ux) = report.ux {
        module_scores.push(ModuleScoreEntry {
            name: "UX".to_string(),
            score: ux.score,
            grade: ux.grade.clone(),
            weight_pct: 15,
        });
    }

    // Weighted overall score — use corrected accessibility score, not raw
    let overall_score = {
        let mut weighted_sum = corrected_score as f64 * 40.0;
        let mut total_weight = 40.0;
        if let Some(ref perf) = report.performance {
            weighted_sum += perf.score.overall as f64 * 20.0;
            total_weight += 20.0;
        }
        if let Some(ref seo) = report.seo {
            weighted_sum += seo.score as f64 * 20.0;
            total_weight += 20.0;
        }
        if let Some(ref security) = report.security {
            weighted_sum += security.score as f64 * 10.0;
            total_weight += 10.0;
        }
        if let Some(ref mobile) = report.mobile {
            weighted_sum += mobile.score as f64 * 10.0;
            total_weight += 10.0;
        }
        if let Some(ref ux) = report.ux {
            weighted_sum += ux.score as f64 * 15.0;
            total_weight += 15.0;
        }
        (weighted_sum / total_weight).round() as u32
    };

    NormalizedReport {
        url: report.url.clone(),
        wcag_level: report.wcag_level,
        timestamp: report.timestamp,
        duration_ms: report.duration_ms,
        nodes_analyzed: report.nodes_analyzed,
        score,
        grade,
        certificate,
        findings,
        severity_counts,
        module_scores,
        overall_score,
        raw_performance: report.performance.clone(),
        raw_seo: report.seo.clone(),
        raw_security: report.security.clone(),
        raw_mobile: report.mobile.clone(),
        raw_ux: report.ux.clone(),
        raw_dark_mode: report.dark_mode.clone(),
        raw_wcag: report.wcag_results.clone(),
    }
}

fn calculate_priority_score(severity: Severity, occurrence_count: usize, rule_id: &str) -> f32 {
    let severity_weight = match severity {
        Severity::Critical => 4.0,
        Severity::High => 3.0,
        Severity::Medium => 2.0,
        Severity::Low => 1.0,
    };
    let reach = occurrence_count.max(1) as f32;
    let effort_weight = effort_weight_for_rule(rule_id);
    (severity_weight * reach) / effort_weight
}

fn effort_weight_for_rule(rule_id: &str) -> f32 {
    if let Some(rule) = RuleLookup::by_id(rule_id) {
        use crate::taxonomy::IssueClass;
        match rule.issue_class {
            IssueClass::Missing => 1.0,
            IssueClass::Invalid => 1.2,
            IssueClass::Weak => 1.5,
            IssueClass::Risk => 2.0,
            IssueClass::Opportunity => 1.2,
            IssueClass::Informational => 2.5,
        }
    } else {
        1.5
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wcag::{Violation, WcagResults};

    #[test]
    fn test_normalize_empty() {
        let report = AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            WcagResults::new(),
            100,
        );
        let norm = normalize(&report);

        assert_eq!(norm.score, 100);
        assert_eq!(norm.grade, "A");
        assert_eq!(norm.certificate, "PLATINUM");
        assert!(norm.findings.is_empty());
        assert_eq!(norm.severity_counts.total, 0);
    }

    #[test]
    fn test_normalize_groups_by_rule() {
        let mut results = WcagResults::new();
        results.add_violation(Violation::new(
            "1.1.1",
            "Non-text Content",
            WcagLevel::A,
            Severity::High,
            "Missing alt 1",
            "n1",
        ));
        results.add_violation(Violation::new(
            "1.1.1",
            "Non-text Content",
            WcagLevel::A,
            Severity::High,
            "Missing alt 2",
            "n2",
        ));
        results.add_violation(Violation::new(
            "2.4.4",
            "Link Purpose",
            WcagLevel::A,
            Severity::Medium,
            "Unclear link",
            "n3",
        ));

        let report = AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            results,
            100,
        );
        let norm = normalize(&report);

        assert_eq!(norm.findings.len(), 2);
        let alt = norm
            .findings
            .iter()
            .find(|f| f.wcag_criterion == "1.1.1")
            .unwrap();
        assert_eq!(alt.occurrence_count, 2);
        assert!(alt.priority_score > 0.0);
        assert_eq!(alt.occurrences.len(), 2);
        assert_eq!(alt.dimension, "Accessibility");
        assert!(!alt.rule_id.is_empty());
    }

    #[test]
    fn test_normalize_taxonomy_fields() {
        let mut results = WcagResults::new();
        results.add_violation(Violation::new(
            "1.1.1",
            "Non-text Content",
            WcagLevel::A,
            Severity::High,
            "Missing alt",
            "n1",
        ));

        let report = AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            results,
            100,
        );
        let norm = normalize(&report);

        let finding = &norm.findings[0];
        assert_eq!(finding.rule_id, "a11y.alt_text.missing");
        assert_eq!(finding.dimension, "Accessibility");
        assert_eq!(finding.subcategory, "Inhalte & Alternativen");
        assert_eq!(finding.issue_class, "Fehlend");
        assert!(finding.score_impact.base_penalty > 0.0);
        assert!(finding.score_impact.max_penalty >= finding.score_impact.base_penalty);
        assert!(!finding.score_impact.scaling.is_empty());
        assert!(!finding.user_impact.is_empty());
    }

    #[test]
    fn test_normalize_severity_counts() {
        let mut results = WcagResults::new();
        results.add_violation(Violation::new(
            "1.1.1",
            "Alt",
            WcagLevel::A,
            Severity::High,
            "Err",
            "n1",
        ));
        results.add_violation(Violation::new(
            "1.1.1",
            "Alt",
            WcagLevel::A,
            Severity::High,
            "Err",
            "n2",
        ));
        results.add_violation(Violation::new(
            "2.4.4",
            "Link",
            WcagLevel::A,
            Severity::Medium,
            "Warn",
            "n3",
        ));

        let report = AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            results,
            100,
        );
        let norm = normalize(&report);

        assert_eq!(norm.severity_counts.high, 2);
        assert_eq!(norm.severity_counts.medium, 1);
        assert_eq!(norm.severity_counts.total, 3);
    }

    #[test]
    fn test_normalize_lang_suppression() {
        let mut results = WcagResults::new();
        results.add_violation(Violation::new(
            "3.1.1",
            "Language",
            WcagLevel::A,
            Severity::High,
            "Missing lang",
            "n1",
        ));

        let mut report = AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            results,
            100,
        );
        // Without SEO data — 3.1.1 should remain
        let norm_no_seo = normalize(&report);
        assert_eq!(norm_no_seo.findings.len(), 1);

        // With SEO indicating has_lang — 3.1.1 should be suppressed
        report.seo = Some(crate::seo::SeoAnalysis {
            technical: crate::seo::TechnicalSeo {
                has_lang: true,
                ..Default::default()
            },
            ..Default::default()
        });
        let norm_with_seo = normalize(&report);
        assert!(norm_with_seo.findings.is_empty());
        assert_eq!(norm_with_seo.score, 100);
    }

    #[test]
    fn test_score_consistency() {
        let mut results = WcagResults::new();
        results.add_violation(Violation::new(
            "1.1.1",
            "Alt",
            WcagLevel::A,
            Severity::High,
            "Missing",
            "n1",
        ));

        let report = AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            results,
            100,
        );
        let norm = normalize(&report);

        // Grade and certificate must match score
        let expected_grade = AccessibilityScorer::calculate_grade(norm.score as f32);
        let expected_cert = AccessibilityScorer::calculate_certificate(norm.score as f32);
        assert_eq!(norm.grade, expected_grade);
        assert_eq!(norm.certificate, expected_cert);
    }
}
