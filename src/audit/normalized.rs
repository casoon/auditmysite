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

use crate::audit::report::{AuditReport, PerformanceResults, ViewportScores};
use crate::audit::scoring::{AccessibilityScorer, PrincipleCoverage};
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
    /// Grade aus overall_score (gewichteter Gesamtscore über alle aktiven Module)
    pub grade: String,
    /// Certificate aus korrigiertem Accessibility-Score
    pub certificate: String,

    /// Normalisierte, gruppierte Findings mit Taxonomie-Feldern
    pub findings: Vec<NormalizedFinding>,
    /// Severity-Zähler (einheitliche Terminologie)
    pub severity_counts: SeverityCounts,

    /// Modul-Scores
    pub module_scores: Vec<ModuleScoreEntry>,
    /// Gewichteter Gesamtscore über alle aktiven Module
    pub overall_score: u32,

    /// Risk assessment — independent from score
    pub risk: RiskAssessment,
    /// WCAG principle coverage — informative secondary indicator, does not
    /// affect the numeric score.
    #[serde(default)]
    pub principle_coverage: PrincipleCoverage,
    /// Audit flags for noteworthy signal conflicts or caveats
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub audit_flags: Vec<AuditFlag>,
    /// Whether desktop/mobile cover screenshots were captured for this audit.
    #[serde(default)]
    pub has_screenshots: bool,
    /// Per-viewport scores from dual-pass audit (70 % mobile / 30 % desktop).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub viewport_scores: Option<ViewportScores>,
    /// How `overall_score` was computed: `"module_weighted"` (standard) or
    /// `"viewport_weighted"` (dual-pass: 70 % mobile + 30 % desktop + 10 % security).
    pub score_calculation_method: String,

    /// Rohdaten für Modul-Details (nicht serialisiert)
    #[serde(skip)]
    pub raw_performance: Option<PerformanceResults>,
    #[serde(skip)]
    pub raw_performance_desktop: Option<PerformanceResults>,
    #[serde(skip)]
    pub raw_seo: Option<SeoAnalysis>,
    #[serde(skip)]
    pub raw_security: Option<SecurityAnalysis>,
    #[serde(skip)]
    pub raw_mobile: Option<MobileFriendliness>,
    #[serde(skip)]
    pub raw_ux: Option<crate::ux::UxAnalysis>,
    #[serde(skip)]
    pub raw_journey: Option<crate::journey::JourneyAnalysis>,
    #[serde(skip)]
    pub raw_dark_mode: Option<DarkModeAnalysis>,
    #[serde(skip)]
    pub raw_source_quality: Option<crate::source_quality::SourceQualityAnalysis>,
    #[serde(skip)]
    pub raw_ai_visibility: Option<crate::ai_visibility::AiVisibilityAnalysis>,
    #[serde(skip)]
    pub raw_tech_stack: Option<crate::tech_stack::TechStackAnalysis>,
    #[serde(skip)]
    pub raw_content_visibility: Option<crate::content_visibility::ContentVisibilityAnalysis>,
    #[serde(skip)]
    pub raw_wcag: WcagResults,
    #[serde(skip)]
    pub raw_patterns: Option<crate::patterns::PatternAnalysis>,
    #[serde(skip)]
    pub raw_throttled_performance: Vec<crate::audit::report::ThrottledPerfResult>,
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
    /// Primary axe-core rule ID, if applicable
    pub axe_id: Option<String>,
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
    /// Viewport tags, e.g. "mobile-only", "desktop-only", "both-viewports"
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
}

/// Score-Eintrag pro Modul
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleScoreEntry {
    pub name: String,
    pub score: u32,
    pub grade: String,
    pub weight_pct: u32,
    /// True when this module's score feeds directly into overall_score.
    /// False for supplemental dimensions (UX, Journey) that are displayed
    /// but not part of the core weighted average.
    pub contributes_to_overall: bool,
    /// Whether this module uses direct measurement or heuristic inference.
    pub measurement_type: String,
}

/// Risk level — independent from score.
/// Score = quality level, Risk = operational/legal relevance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskLevel::Low => write!(f, "Gering"),
            RiskLevel::Medium => write!(f, "Mittel"),
            RiskLevel::High => write!(f, "Hoch"),
            RiskLevel::Critical => write!(f, "Kritisch"),
        }
    }
}

impl RiskLevel {
    /// Localized label via the report I18n bundle.
    pub fn label_localized(&self, i18n: &crate::i18n::I18n) -> String {
        let key = match self {
            RiskLevel::Low => "risk-level-low",
            RiskLevel::Medium => "risk-level-medium",
            RiskLevel::High => "risk-level-high",
            RiskLevel::Critical => "risk-level-critical",
        };
        i18n.t(key)
    }
}

/// Risk assessment — computed separately from score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    /// Overall risk level
    pub level: RiskLevel,
    /// Number of critical accessibility issues
    pub critical_issues: usize,
    /// Number of high-severity issues
    pub high_issues: usize,
    /// Number of WCAG Level A violations (legally relevant under BFSG/EAA)
    pub legal_flags: usize,
    /// Number of blocking interaction issues (buttons/forms without names)
    pub blocking_issues: usize,
    /// Human-readable risk summary
    pub summary: String,
}

impl RiskAssessment {
    /// Locale-aware risk summary. Falls back to the stored `summary` (German)
    /// for unknown locales.
    pub fn summary_for(&self, locale: &str) -> String {
        if locale != "en" {
            return self.summary.clone();
        }
        match self.level {
            RiskLevel::Critical => format!(
                "Critical risk: {} WCAG Level A violations with legal relevance (BFSG). {} blocking issues on interactive controls.",
                self.legal_flags, self.blocking_issues
            ),
            RiskLevel::High => format!(
                "High risk: {} critical and {} severe issues. Users are actively excluded.",
                self.critical_issues, self.high_issues
            ),
            RiskLevel::Medium => format!(
                "Medium risk: {} severe issues detected. Limitations for certain user groups.",
                self.high_issues + self.critical_issues
            ),
            RiskLevel::Low => "Low risk: no critical violations — improvement potential remains.".to_string(),
        }
    }
}

/// Explicit audit caveat or conflicting signal surfaced to downstream outputs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditFlag {
    pub kind: String,
    pub related_rule: Option<String>,
    pub source: String,
    pub message: String,
}

/// Normalisiert einen rohen AuditReport.
///
/// - Gruppert Violations nach Regel-ID
/// - Reichert mit Taxonomie-Feldern an (via RuleLookup)
/// - Berechnet Grade/Certificate aus korrigiertem Score
pub fn normalize(report: &AuditReport) -> NormalizedReport {
    let violations = &report.wcag_results.violations;

    let seo_reports_lang = report.seo.as_ref().is_some_and(|s| s.technical.has_lang);
    let had_311 = violations.iter().any(|v| v.rule == "3.1.1");

    // Group violations by rule ID
    let mut groups: HashMap<&str, Vec<&crate::wcag::Violation>> = HashMap::new();
    for v in violations {
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

            // Deduplicate by selector: multiple DOM nodes that share an identical
            // CSS selector string collapse into a single representative occurrence.
            // occurrence_count still reflects the actual number of affected elements.
            let occurrence_count = violations.len();
            let mut seen_selectors = std::collections::HashSet::new();
            let occurrences: Vec<OccurrenceDetail> = violations
                .iter()
                .filter(|v| {
                    let key = v.selector.as_deref().unwrap_or(&v.node_id);
                    seen_selectors.insert(key.to_string())
                })
                .map(|v| OccurrenceDetail {
                    node_id: v.node_id.clone(),
                    message: v.message.clone(),
                    selector: v.selector.clone(),
                    fix_suggestion: v.fix_suggestion.clone(),
                    html_snippet: v.html_snippet.clone(),
                    suggested_code: v.suggested_code.clone(),
                    tags: v.tags.clone(),
                })
                .collect();
            let axe_id = taxonomy_rule.and_then(|r| r.axe_id).map(String::from);
            let priority_score =
                calculate_priority_score(first.severity, occurrence_count, &tax_id);

            NormalizedFinding {
                rule_id: tax_id.clone(),
                wcag_criterion: rule_id.to_string(),
                axe_id,
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

    // Aggregate SEO heading issues into findings
    if let Some(seo) = &report.seo {
        let mut heading_groups: HashMap<&str, Vec<&crate::seo::HeadingIssue>> = HashMap::new();
        for issue in &seo.headings.issues {
            heading_groups
                .entry(&issue.issue_type)
                .or_default()
                .push(issue);
        }
        for (issue_type, issues) in heading_groups {
            let first = issues[0];
            let occurrence_count = issues.len();
            let rule_id = format!("seo.headings.{}", issue_type);
            let title = match issue_type {
                "long_heading" => "Überschrift zu lang".to_string(),
                "missing_h1" => "Fehlende H1-Überschrift".to_string(),
                "multiple_h1" => "Mehrere H1-Überschriften".to_string(),
                "skipped_level" => "Übersprungene Überschriftenebene".to_string(),
                "empty_heading" => "Leere Überschrift".to_string(),
                other => other.replace('_', " "),
            };
            let priority_score =
                calculate_priority_score(first.severity, occurrence_count, &rule_id);
            findings.push(NormalizedFinding {
                rule_id: rule_id.clone(),
                wcag_criterion: String::new(),
                axe_id: None,
                wcag_level: String::new(),
                dimension: "SEO".to_string(),
                subcategory: "Content".to_string(),
                issue_class: "issue".to_string(),
                severity: first.severity,
                user_impact: String::new(),
                technical_impact: first.message.clone(),
                score_impact: ScoreImpactData {
                    base_penalty: 0.0,
                    max_penalty: 0.0,
                    scaling: "none".to_string(),
                },
                report_visibility: ReportVisibilityData::default(),
                aggregation_key: rule_id,
                title,
                description: first.message.clone(),
                occurrence_count,
                priority_score,
                occurrences: issues
                    .iter()
                    .map(|i| OccurrenceDetail {
                        node_id: i.issue_type.clone(),
                        message: i.message.clone(),
                        selector: None,
                        fix_suggestion: None,
                        html_snippet: None,
                        suggested_code: None,
                        tags: vec!["seo".to_string()],
                    })
                    .collect(),
            });
        }
    }

    // Sort by priority score (highest first), then by severity
    findings.sort_by(|a, b| {
        b.priority_score
            .partial_cmp(&a.priority_score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| b.severity.cmp(&a.severity))
    });

    let score = report.score.round().max(1.0) as u32;
    let accessibility_grade = AccessibilityScorer::calculate_grade(report.score).to_string();
    let certificate = AccessibilityScorer::calculate_certificate(report.score).to_string();

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
        grade: accessibility_grade,
        weight_pct: 40,
        contributes_to_overall: true,
        measurement_type: "measured".to_string(),
    });

    if let Some(ref perf) = report.performance {
        module_scores.push(ModuleScoreEntry {
            name: "Performance".to_string(),
            score: perf.score.overall,
            grade: AccessibilityScorer::calculate_grade(perf.score.overall as f32).to_string(),
            weight_pct: 20,
            contributes_to_overall: true,
            measurement_type: "measured".to_string(),
        });
    }
    if let Some(ref seo) = report.seo {
        module_scores.push(ModuleScoreEntry {
            name: "SEO".to_string(),
            score: seo.score,
            grade: AccessibilityScorer::calculate_grade(seo.score as f32).to_string(),
            weight_pct: 20,
            contributes_to_overall: true,
            measurement_type: "measured".to_string(),
        });
    }
    if let Some(ref sec) = report.security {
        module_scores.push(ModuleScoreEntry {
            name: "Security".to_string(),
            score: sec.score,
            grade: sec.grade.clone(),
            weight_pct: 10,
            contributes_to_overall: true,
            measurement_type: "measured".to_string(),
        });
    }
    if let Some(ref mob) = report.mobile {
        module_scores.push(ModuleScoreEntry {
            name: "Mobile".to_string(),
            score: mob.score,
            grade: AccessibilityScorer::calculate_grade(mob.score as f32).to_string(),
            weight_pct: 10,
            contributes_to_overall: true,
            measurement_type: "measured".to_string(),
        });
    }
    if let Some(ref ux) = report.ux {
        // Accessibility flows into UX: critical a11y issues penalize UX score
        // Rationale: for users with disabilities, Accessibility IS the UX
        let a11y_penalty = {
            let critical = severity_counts.critical;
            let high = severity_counts.high;
            if critical >= 10 {
                25 // severe: many critical barriers
            } else if critical >= 5 {
                15
            } else if critical > 0 {
                10
            } else if high >= 5 {
                5
            } else {
                0
            }
        };
        let adjusted_ux = ux.score.saturating_sub(a11y_penalty);
        let adjusted_grade = match adjusted_ux {
            90..=100 => "A",
            80..=89 => "B",
            70..=79 => "C",
            60..=69 => "D",
            _ => "F",
        };
        module_scores.push(ModuleScoreEntry {
            name: "UX".to_string(),
            score: adjusted_ux,
            grade: adjusted_grade.to_string(),
            weight_pct: 15,
            contributes_to_overall: false,
            measurement_type: "heuristic".to_string(),
        });
    }
    if let Some(ref journey) = report.journey {
        // Journey also gets a11y penalty — inaccessible journeys are broken journeys
        let a11y_penalty = {
            let critical = severity_counts.critical;
            if critical >= 10 {
                20
            } else if critical >= 5 {
                10
            } else if critical > 0 {
                5
            } else {
                0
            }
        };
        let adjusted_journey = journey.score.saturating_sub(a11y_penalty);
        let adjusted_grade = match adjusted_journey {
            90..=100 => "A",
            80..=89 => "B",
            70..=79 => "C",
            60..=69 => "D",
            _ => "F",
        };
        module_scores.push(ModuleScoreEntry {
            name: "Journey".to_string(),
            score: adjusted_journey,
            grade: adjusted_grade.to_string(),
            weight_pct: 10,
            contributes_to_overall: false,
            measurement_type: "heuristic".to_string(),
        });
    }

    // Weighted overall score — 70/30 viewport weighting when dual-pass data present
    let (overall_score, score_calculation_method) = if let Some(ref vs) = report.viewport_scores {
        // Blend in security (10 %) on top of the 70/30 viewport base.
        // module_scores still reflect individual module quality but do NOT
        // additively produce the overall_score — mark them accordingly
        // and clear weight_pct to avoid implying a weighted-average model.
        for m in &mut module_scores {
            m.contributes_to_overall = false;
            m.weight_pct = 0;
        }
        let mut weighted = vs.weighted_overall as f64 * 90.0;
        let mut total = 90.0;
        if let Some(ref security) = report.security {
            weighted += security.score as f64 * 10.0;
            total += 10.0;
        }
        (
            (weighted / total).round() as u32,
            "viewport_weighted".to_string(),
        )
    } else {
        let contributing_modules = module_scores.iter().filter(|m| m.contributes_to_overall);
        let (weighted_sum, total_weight) =
            contributing_modules.fold((0.0, 0.0), |(sum, total), module| {
                (
                    sum + module.score as f64 * module.weight_pct as f64,
                    total + module.weight_pct as f64,
                )
            });

        (
            (weighted_sum / total_weight).round() as u32,
            "module_weighted".to_string(),
        )
    };

    let grade = AccessibilityScorer::calculate_grade(overall_score as f32).to_string();

    let mut audit_flags = Vec::new();
    if seo_reports_lang && had_311 {
        audit_flags.push(AuditFlag {
            kind: "conflicting_signal".to_string(),
            related_rule: Some("3.1.1".to_string()),
            source: "seo.technical.has_lang".to_string(),
            message: "SEO detected a language declaration while WCAG still reported 3.1.1. The finding remains in the report and should be verified against the rendered DOM.".to_string(),
        });
    }

    // ── Risk Assessment (independent from score) ──────────────────
    let risk = {
        let critical_issues = severity_counts.critical;
        let high_issues = severity_counts.high;

        // Legal flags: WCAG Level A violations are legally relevant (BFSG/EAA)
        let legal_flags = findings
            .iter()
            .filter(|f| {
                f.wcag_level == "A" && matches!(f.severity, Severity::Critical | Severity::High)
            })
            .map(|f| f.occurrence_count)
            .sum::<usize>();

        // Blocking issues: interactive elements without accessible names (4.1.2)
        let blocking_issues = findings
            .iter()
            .filter(|f| f.wcag_criterion == "4.1.2" || f.wcag_criterion == "2.1.1")
            .map(|f| f.occurrence_count)
            .sum::<usize>();

        let level = if legal_flags > 0 && critical_issues > 0 {
            RiskLevel::Critical
        } else if critical_issues >= 3 || blocking_issues >= 10 {
            RiskLevel::High
        } else if high_issues >= 3 || critical_issues >= 1 {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        };

        let summary = match level {
            RiskLevel::Critical => format!(
                "Kritisches Risiko: {} WCAG-Level-A-Verstöße mit rechtlicher Relevanz (BFSG). {} Blocker bei Bedienelementen.",
                legal_flags, blocking_issues
            ),
            RiskLevel::High => format!(
                "Hohes Risiko: {} kritische und {} schwerwiegende Probleme. Nutzer werden aktiv ausgeschlossen.",
                critical_issues, high_issues
            ),
            RiskLevel::Medium => format!(
                "Mittleres Risiko: {} schwerwiegende Probleme erkannt. Einschränkungen für bestimmte Nutzergruppen.",
                high_issues + critical_issues
            ),
            RiskLevel::Low => "Geringes Risiko: Keine kritischen Verstöße — Verbesserungspotenzial vorhanden.".to_string(),
        };

        RiskAssessment {
            level,
            critical_issues,
            high_issues,
            legal_flags,
            blocking_issues,
            summary,
        }
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
        risk,
        principle_coverage: AccessibilityScorer::calculate_coverage(violations),
        audit_flags,
        has_screenshots: report.page_screenshots.is_some(),
        viewport_scores: report.viewport_scores.clone(),
        score_calculation_method,
        raw_performance: report.performance.clone(),
        raw_performance_desktop: report
            .dual_viewport
            .as_ref()
            .and_then(|d| d.desktop.performance.clone()),
        raw_seo: report.seo.clone(),
        raw_security: report.security.clone(),
        raw_mobile: report.mobile.clone(),
        raw_ux: report.ux.clone(),
        raw_journey: report.journey.clone(),
        raw_dark_mode: report.dark_mode.clone(),
        raw_source_quality: report.source_quality.clone(),
        raw_ai_visibility: report.ai_visibility.clone(),
        raw_tech_stack: report.tech_stack.clone(),
        raw_content_visibility: report.content_visibility.clone(),
        raw_wcag: report.wcag_results.clone(),
        raw_patterns: report.patterns.clone(),
        raw_throttled_performance: report.throttled_performance.clone(),
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
        assert_eq!(norm.certificate, "SEHR GUT");
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
    fn test_normalize_lang_conflict_flag_keeps_finding() {
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
        assert!(norm_no_seo.audit_flags.is_empty());

        // With SEO indicating has_lang — 3.1.1 should remain but be marked as a conflicting signal
        report.seo = Some(crate::seo::SeoAnalysis {
            technical: crate::seo::TechnicalSeo {
                has_lang: true,
                ..Default::default()
            },
            ..Default::default()
        });
        let norm_with_seo = normalize(&report);
        assert_eq!(norm_with_seo.findings.len(), 1);
        assert_eq!(norm_with_seo.score, report.score.round() as u32);
        assert_eq!(norm_with_seo.audit_flags.len(), 1);
        assert_eq!(
            norm_with_seo.audit_flags[0].related_rule.as_deref(),
            Some("3.1.1")
        );
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
