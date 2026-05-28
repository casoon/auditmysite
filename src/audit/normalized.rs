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
    /// Severity-Zähler — zählt **Findings** (eine Zeile pro Regel + Severity).
    pub severity_counts: SeverityCounts,
    /// Severity-Zähler — zählt **Element-Occurrences** (alle betroffenen Elemente).
    #[serde(default)]
    pub occurrence_counts: SeverityCounts,

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
    /// Exact inputs used to produce `overall_score`.
    /// Present only for `viewport_weighted`; absent for `module_weighted` (module
    /// `weight_pct` values are already exact in that case).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score_breakdown: Option<ScoreBreakdown>,

    /// Findings produced by the Accessibility-Journey-Layer (Phase 1+).
    /// Kept separate from `findings[]` so WCAG severity counts remain
    /// rechtsrelevant.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub interactive_findings: Vec<InteractiveFinding>,
    /// Reproducible journey traces (tab walks, modal opens, …) produced by
    /// the Accessibility-Journey-Layer. `None` when `--interactive=off`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accessibility_journey: Option<AccessibilityJourney>,
    /// Optional semantic / LLM advisory findings. Never influence score or
    /// risk — explicitly advisory.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub advisory_findings: Vec<AdvisoryFinding>,

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
    #[serde(skip)]
    pub raw_best_practices: Option<crate::best_practices::BestPracticesAnalysis>,
}

/// Einheitliche Severity-Zähler
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SeverityCounts {
    pub critical: usize,
    pub high: usize,
    pub medium: usize,
    pub low: usize,
    pub total: usize,
}

fn default_finding_category() -> String {
    "wcag".to_string()
}

/// Ein normalisiertes Finding — gruppiert nach Regel, mit Taxonomie-Feldern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedFinding {
    /// Category: "wcag" for WCAG accessibility findings, "seo" for SEO findings.
    #[serde(default = "default_finding_category")]
    pub category: String,
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
    #[serde(skip_serializing_if = "String::is_empty")]
    pub user_impact: String,
    /// Technische Auswirkung
    #[serde(skip_serializing_if = "String::is_empty")]
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
    /// Offizielle Referenz zum Kriterium (z.B. WCAG-Understanding-Seite)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub help_url: Option<String>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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
    /// Numeric risk score 0–100 (higher = more risk)
    pub score: u32,
    /// Minimum score at which the current level is triggered
    pub threshold: u32,
    /// Module or factor primarily driving the risk level
    pub driven_by: String,
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

/// Transparent breakdown of how `overall_score` was computed in viewport_weighted mode.
/// Allows consumers to reproduce the exact score from its inputs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreBreakdown {
    /// Blending weights for the two viewport passes
    pub desktop_weight_pct: u32,
    pub mobile_weight_pct: u32,
    /// Raw overall scores from each viewport pass
    pub desktop_overall: u32,
    pub mobile_overall: u32,
    /// Blended result before security is mixed in (mobile*70% + desktop*30%)
    pub viewport_blended_overall: u32,
    /// Weight given to the viewport blend in the final formula (always 90 when security present)
    pub viewport_blend_weight_pct: u32,
    /// Security score after vulnerable-library penalty
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_score: Option<u32>,
    /// Weight given to security in the final formula (always 10 when security present)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_weight_pct: Option<u32>,
}

// ---------------------------------------------------------------------------
// Accessibility-Journey-Layer (Phase 1: Foundation only — types live here so
// `NormalizedReport` is schema-stable for all future phases.)
// ---------------------------------------------------------------------------

/// Bundle of accessibility-journey results for one page.
/// Populated only when `--interactive != off`; otherwise the report's
/// `accessibility_journey` field stays `None`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AccessibilityJourney {
    /// Reproducible step sequences (one per journey: tab walk, modal open, …).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub traces: Vec<JourneyTrace>,
}

/// One reproducible journey — an ordered list of interaction steps and the
/// snapshots captured along the way. The trace is the *evidence* attached to
/// every interactive finding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JourneyTrace {
    /// Journey identifier: "tab_walk", "skip_link", "modal_contact", ...
    pub journey: String,
    /// Ordered steps that compose the journey.
    pub steps: Vec<JourneyStep>,
}

/// A single step in a journey. Designed to read naturally as JSON so a
/// developer can reproduce the journey by replaying the actions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JourneyStep {
    /// "tab" | "shift_tab" | "enter" | "escape" | "arrow_down" | "click"
    /// | "synthetic_click" (fallback) | "type" | "wait"
    pub action: String,
    /// Selector or descriptive label of the target, if applicable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    /// Selector of `document.activeElement` after the action.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub focus: Option<String>,
    /// Human-readable outcome marker, e.g. "modal_opened",
    /// "focus_lost_to_body", "no_change".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,
    /// Label of the AXSnapshot captured after this step (matches
    /// `AXSnapshot.label`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub snapshot_label: Option<String>,
}

/// Finding produced by an interactive (journey) test. Distinct from WCAG
/// `findings[]` — does not feed `severity_counts` or `legal_flags`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractiveFinding {
    /// "TabOrder" | "FocusTrap" | "StateTransition" | "FocusRestoration"
    /// | "FormError" | "SpaNavigation" | "HiddenFocusable" | "SkipLink"
    /// | "FocusIndicator" | "MenuJourney" | "TabsJourney"
    pub category: String,
    pub severity: Severity,
    /// Which journey produced this finding (matches `JourneyTrace.journey`).
    pub journey: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub before_snapshot_label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub after_snapshot_label: Option<String>,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fix_suggestion: Option<String>,
}

/// Advisory finding from semantic / LLM evaluation. Explicitly advisory —
/// never influences score or risk level. Off unless `--semantic-eval` is set.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvisoryFinding {
    /// "link_text" | "heading_outline" | "form_label_coherence"
    /// | "blind_user_perspective"
    pub category: String,
    pub message: String,
    /// "llm" | "static_heuristic"
    pub source: String,
    /// 0.0..1.0 — model confidence or heuristic strength.
    pub confidence: f32,
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

    // Maximum selector-deduplicated occurrences stored per finding.
    // occurrence_count always reflects the true total; this only caps what is
    // serialized to keep JSON payloads compact.
    const MAX_OCCURRENCES: usize = 5;

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
            // Capped at MAX_OCCURRENCES to keep JSON output compact; the true total
            // is always available via occurrence_count.
            let occurrence_count = violations.len();
            let mut seen_selectors = std::collections::HashSet::new();
            let mut occurrences: Vec<OccurrenceDetail> = violations
                .iter()
                .filter(|v| {
                    let key = v.selector.as_deref().unwrap_or(&v.node_id);
                    seen_selectors.insert(key.to_string())
                })
                .take(MAX_OCCURRENCES)
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
            // Deduplicate fix_suggestion: if all stored occurrences share the same
            // fix, suppress it from occurrences[1..] — it's readable from [0].
            if let Some(shared_fix) = occurrences.first().and_then(|o| o.fix_suggestion.clone()) {
                if occurrences[1..]
                    .iter()
                    .all(|o| o.fix_suggestion.as_deref() == Some(shared_fix.as_str()))
                {
                    for occ in &mut occurrences[1..] {
                        occ.fix_suggestion = None;
                    }
                }
            }
            let axe_id = taxonomy_rule.and_then(|r| r.axe_id).map(String::from);
            // Use the max severity across all violation instances for this rule.
            // Rules deliberately use Low for minor sub-cases (e.g. empty lists, multiple h1);
            // the taxonomy severity is a classification label, not a floor override (#288).
            let severity = violations
                .iter()
                .map(|v| v.severity)
                .max()
                .unwrap_or(first.severity);
            let priority_score = calculate_priority_score(severity, occurrence_count, &tax_id);

            // Prefer the taxonomy title (customer-facing, localized) over the
            // raw rule_name from the WCAG engine — ensures JSON `title` and PDF
            // narrative refer to the same name (see issue #252).
            let display_title = taxonomy_rule
                .map(|r| r.title.to_string())
                .unwrap_or_else(|| first.rule_name.clone());

            NormalizedFinding {
                category: "wcag".to_string(),
                rule_id: tax_id.clone(),
                wcag_criterion: rule_id.to_string(),
                axe_id,
                wcag_level: first.level.to_string(),
                dimension,
                subcategory,
                issue_class,
                severity,
                user_impact,
                technical_impact,
                score_impact,
                report_visibility: visibility,
                aggregation_key: tax_id,
                title: display_title,
                description: first.message.clone(),
                help_url: first.help_url.clone(),
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
            let technical_impact = match issue_type {
                "skipped_level" => {
                    "Übersprungene Heading-Ebenen zerstören die Baumstruktur für Screenreader \
                     und SEO-Crawler — logische Hierarchie H1→H2→H3 einhalten."
                        .to_string()
                }
                "missing_h1" => {
                    "Fehlende H1-Überschrift — Seitenzweck für Suchmaschinen und Screenreader \
                     nicht erkennbar."
                        .to_string()
                }
                "multiple_h1" => {
                    "Mehrere H1-Überschriften untergraben die inhaltliche Hierarchie; \
                     Suchmaschinen können keinen eindeutigen Hauptfokus ableiten."
                        .to_string()
                }
                "long_heading" => {
                    "Überlange Überschriften werden in SERPs abgeschnitten und erschweren \
                     das schnelle Scannen für Nutzer."
                        .to_string()
                }
                "empty_heading" => {
                    "Leere Überschriften erzeugen Navigationsprobleme für Screenreader \
                     und werden von SEO-Crawlern als schlechtes Signal gewertet."
                        .to_string()
                }
                _ => first.message.clone(),
            };
            let priority_score =
                calculate_priority_score(first.severity, occurrence_count, &rule_id);
            findings.push(NormalizedFinding {
                category: "seo".to_string(),
                rule_id: rule_id.clone(),
                wcag_criterion: String::new(),
                axe_id: None,
                wcag_level: String::new(),
                dimension: "SEO".to_string(),
                subcategory: "Content".to_string(),
                issue_class: "issue".to_string(),
                severity: first.severity,
                user_impact: String::new(),
                technical_impact,
                score_impact: ScoreImpactData {
                    base_penalty: 0.0,
                    max_penalty: 0.0,
                    scaling: "none".to_string(),
                },
                report_visibility: ReportVisibilityData::default(),
                aggregation_key: rule_id,
                title,
                description: first.message.clone(),
                help_url: None,
                occurrence_count,
                priority_score,
                occurrences: issues
                    .iter()
                    .take(MAX_OCCURRENCES)
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

    // Severity counts — only WCAG findings count (not SEO findings).
    // `severity_counts` zählt Findings (eine Zeile pro Regel/Severity),
    // `occurrence_counts` zählt Element-Occurrences (Summe aller betroffenen Elemente).
    let wcag_findings: Vec<_> = findings.iter().filter(|f| f.category == "wcag").collect();
    let severity_counts = SeverityCounts {
        critical: wcag_findings
            .iter()
            .filter(|f| f.severity == Severity::Critical)
            .count(),
        high: wcag_findings
            .iter()
            .filter(|f| f.severity == Severity::High)
            .count(),
        medium: wcag_findings
            .iter()
            .filter(|f| f.severity == Severity::Medium)
            .count(),
        low: wcag_findings
            .iter()
            .filter(|f| f.severity == Severity::Low)
            .count(),
        total: wcag_findings.len(),
    };
    let occurrence_counts = SeverityCounts {
        critical: wcag_findings
            .iter()
            .filter(|f| f.severity == Severity::Critical)
            .map(|f| f.occurrence_count)
            .sum(),
        high: wcag_findings
            .iter()
            .filter(|f| f.severity == Severity::High)
            .map(|f| f.occurrence_count)
            .sum(),
        medium: wcag_findings
            .iter()
            .filter(|f| f.severity == Severity::Medium)
            .map(|f| f.occurrence_count)
            .sum(),
        low: wcag_findings
            .iter()
            .filter(|f| f.severity == Severity::Low)
            .map(|f| f.occurrence_count)
            .sum(),
        total: wcag_findings.iter().map(|f| f.occurrence_count).sum(),
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
    // Vulnerable JS libraries (Best Practices) count as security findings.
    // The penalty is applied to the Security module score so that XSS/RCE-level
    // library issues move the security signal — not just an informational entry.
    let vuln_security_penalty: u32 = report
        .best_practices
        .as_ref()
        .map(|bp| {
            bp.vulnerable_libraries
                .vulnerable
                .iter()
                .map(|v| match v.severity.as_str() {
                    "high" => 15,
                    "medium" => 8,
                    _ => 3,
                })
                .sum::<u32>()
                .min(30)
        })
        .unwrap_or(0);
    if let Some(ref sec) = report.security {
        let adjusted = sec.score.saturating_sub(vuln_security_penalty);
        module_scores.push(ModuleScoreEntry {
            name: "Security".to_string(),
            score: adjusted,
            grade: AccessibilityScorer::calculate_grade(adjusted as f32).to_string(),
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
        // Rationale: for users with disabilities, Accessibility IS the UX.
        // Penalty thresholds reflect total affected elements, not distinct rules.
        let a11y_penalty = {
            let critical = occurrence_counts.critical;
            let high = occurrence_counts.high;
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
        // Journey also gets a11y penalty — inaccessible journeys are broken journeys.
        // Threshold uses occurrence-level severity, not finding count.
        let a11y_penalty = {
            let critical = occurrence_counts.critical;
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
    if let Some(ref bp) = report.best_practices {
        module_scores.push(ModuleScoreEntry {
            name: "Best Practices".to_string(),
            score: bp.score,
            grade: AccessibilityScorer::calculate_grade(bp.score as f32).to_string(),
            weight_pct: 0,
            contributes_to_overall: false,
            measurement_type: "measured".to_string(),
        });
    }

    // Weighted overall score — 70/30 viewport weighting when dual-pass data present
    let (overall_score, score_calculation_method, score_breakdown) =
        if let Some(ref vs) = report.viewport_scores {
            let security_adjusted = report
                .security
                .as_ref()
                .map(|s| s.score.saturating_sub(vuln_security_penalty));
            let blend_weight = if security_adjusted.is_some() {
                90u32
            } else {
                100u32
            };
            let mut weighted = vs.weighted_overall as f64 * blend_weight as f64;
            let mut total = blend_weight as f64;
            if let Some(sec) = security_adjusted {
                weighted += sec as f64 * 10.0;
                total += 10.0;
            }
            let breakdown = ScoreBreakdown {
                desktop_weight_pct: 30,
                mobile_weight_pct: 70,
                desktop_overall: vs.desktop.overall,
                mobile_overall: vs.mobile.overall,
                viewport_blended_overall: vs.weighted_overall,
                viewport_blend_weight_pct: blend_weight,
                security_score: security_adjusted,
                security_weight_pct: security_adjusted.map(|_| 10u32),
            };
            (
                (weighted / total).round() as u32,
                "viewport_weighted".to_string(),
                Some(breakdown),
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
                None,
            )
        };

    let grade = AccessibilityScorer::calculate_grade(overall_score as f32).to_string();
    // Certificate is derived from the overall weighted score so it stays
    // consistent with `grade`. See issue #233 — previously certificate used
    // `report.score` (accessibility only), which produced contradictory labels
    // like "Grade A — Cert AUSBAUFÄHIG" when other module scores were high.
    let certificate = AccessibilityScorer::calculate_certificate(overall_score as f32).to_string();

    let mut audit_flags = Vec::new();
    if seo_reports_lang && had_311 {
        audit_flags.push(AuditFlag {
            kind: "conflicting_signal".to_string(),
            related_rule: Some("3.1.1".to_string()),
            source: "seo.technical.has_lang".to_string(),
            message: "SEO detected a language declaration while WCAG still reported 3.1.1. The finding remains in the report and should be verified against the rendered DOM.".to_string(),
        });
    }
    if let Some(ref vs) = report.viewport_scores {
        let desktop_a11y = vs.desktop.accessibility as i32;
        let mobile_a11y = vs.mobile.accessibility as i32;
        let gap = (desktop_a11y - mobile_a11y).abs();
        if gap >= 20 {
            let (higher, lower, higher_score, lower_score) = if desktop_a11y >= mobile_a11y {
                ("Desktop", "Mobile", desktop_a11y, mobile_a11y)
            } else {
                ("Mobile", "Desktop", mobile_a11y, desktop_a11y)
            };
            audit_flags.push(AuditFlag {
                kind: "viewport_gap".to_string(),
                related_rule: None,
                source: "viewport_scores.accessibility".to_string(),
                message: format!(
                    "{higher} scored {higher_score}, {lower} scored {lower_score} — a {gap}-point gap suggests {lower}-specific rendering differences (e.g. lazy-loaded components, injected markup, or different DOM paths) rather than a site-wide failure.",
                ),
            });
        }
    }

    // Consent banner flag
    if report.consent_banner_detected {
        let msg = if report.consent_banner_dismissed {
            format!(
                "Consent-Banner erkannt und automatisch geschlossen{}. Audit-Ergebnisse spiegeln den Seiteninhalt nach Zustimmung wider.",
                report.consent_banner_cmp.as_ref().map(|c| format!(" ({})", c)).unwrap_or_default()
            )
        } else {
            format!(
                "Consent-Banner erkannt{} — Audit ohne Zustimmung durchgeführt. Barrierefreiheits- und SEO-Ergebnisse können unvollständig sein. Empfehlung: --dismiss-consent nutzen.",
                report.consent_banner_cmp.as_ref().map(|c| format!(" ({})", c)).unwrap_or_default()
            )
        };
        audit_flags.push(AuditFlag {
            kind: "consent_banner".to_string(),
            related_rule: None,
            source: "browser.consent".to_string(),
            message: msg,
        });

        // Consent-wall artifact heuristic: more violations than analyzed nodes is a
        // strong signal that the AXTree captured the consent dialog DOM rather than
        // actual page content.
        if !report.consent_banner_dismissed {
            let violation_count: usize = report.wcag_results.violations.len();
            if report.nodes_analyzed > 0 && violation_count > report.nodes_analyzed {
                audit_flags.push(AuditFlag {
                    kind: "consent_wall_artifact".to_string(),
                    related_rule: None,
                    source: "browser.consent".to_string(),
                    message: format!(
                        "Mögliches Consent-Wall-Artefakt: {} Violations bei nur {} analysierten Nodes. \
                         Scores könnten den Consent-Dialog statt den eigentlichen Seiteninhalt messen. \
                         Empfehlung: Audit mit --dismiss-consent wiederholen.",
                        violation_count, report.nodes_analyzed
                    ),
                });
            }
        }
    }

    // ── Risk Assessment (independent from score) ──────────────────
    let risk = {
        // Risk thresholds reflect total affected elements (occurrence_counts),
        // not the number of distinct rules (severity_counts).
        let critical_issues = occurrence_counts.critical;
        let high_issues = occurrence_counts.high;

        // Legal flags: count distinct WCAG Level A rules with High/Critical severity.
        // Per-occurrence counting would inflate the number (e.g. 1000 images without
        // alt text is one rule violation, not 1000 legal flags).
        let legal_flags = findings
            .iter()
            .filter(|f| {
                f.wcag_level == "A" && matches!(f.severity, Severity::Critical | Severity::High)
            })
            .count();

        // Blocking issues: interactive elements without accessible names (4.1.2)
        let blocking_issues = findings
            .iter()
            .filter(|f| f.wcag_criterion == "4.1.2" || f.wcag_criterion == "2.1.1")
            .map(|f| f.occurrence_count)
            .sum::<usize>();

        let risk_score = (legal_flags as u32 * 20
            + critical_issues as u32 * 10
            + high_issues as u32 * 3
            + blocking_issues as u32 * 2)
            .min(100);

        // Risk level — explicit precedence; legal_flags and blocking_issues both
        // raise the floor even when critical_issues is zero (see issue #250).
        let level = if legal_flags > 0 && critical_issues > 0 {
            RiskLevel::Critical
        } else if critical_issues >= 3 || blocking_issues >= 5 || risk_score >= 80 {
            RiskLevel::High
        } else if (high_issues >= 3 && score < 80)
            || critical_issues >= 1
            || legal_flags > 0
            || blocking_issues >= 1
            || score <= 20
        {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        };

        let plural = |n: usize, singular: &str, plural: &str| -> String {
            if n == 1 {
                format!("{} {}", n, singular)
            } else {
                format!("{} {}", n, plural)
            }
        };
        let summary = match level {
            RiskLevel::Critical => {
                // If the overall score is high (≥ 80, i.e. Grade A or B), the
                // Critical risk level alongside a strong grade looks
                // contradictory. Surface the contrast explicitly so the report
                // explains why both signals can hold at once. See issue #237.
                let prefix = if overall_score >= 80 {
                    "Kritisches Risiko trotz gutem Gesamtscore"
                } else {
                    "Kritisches Risiko"
                };
                format!(
                    "{}: {} mit rechtlicher Relevanz (BFSG). {}.",
                    prefix,
                    plural(legal_flags, "WCAG-Level-A-Verstoß", "WCAG-Level-A-Verstöße"),
                    plural(
                        blocking_issues,
                        "Blocker bei Bedienelementen",
                        "Blocker bei Bedienelementen"
                    )
                )
            }
            RiskLevel::High => format!(
                "Hohes Risiko: {} und {}. Nutzer werden aktiv ausgeschlossen.",
                plural(critical_issues, "kritisches Problem", "kritische Probleme"),
                plural(
                    high_issues,
                    "schwerwiegendes Problem",
                    "schwerwiegende Probleme"
                )
            ),
            RiskLevel::Medium => {
                if legal_flags > 0 {
                    format!(
                        "Mittleres Risiko: {} mit rechtlicher Relevanz (BFSG){}.",
                        plural(legal_flags, "WCAG-Level-A-Verstoß", "WCAG-Level-A-Verstöße"),
                        if blocking_issues > 0 {
                            format!(
                                ", {} bei Bedienelementen",
                                plural(blocking_issues, "Blocker", "Blocker")
                            )
                        } else {
                            String::new()
                        }
                    )
                } else if blocking_issues > 0 {
                    format!(
                        "Mittleres Risiko: {} bei Bedienelementen erkannt. \
                         Einschränkungen für bestimmte Nutzergruppen.",
                        plural(blocking_issues, "Blocker", "Blocker")
                    )
                } else {
                    format!(
                        "Mittleres Risiko: {} erkannt. Einschränkungen für bestimmte Nutzergruppen.",
                        plural(
                            high_issues + critical_issues,
                            "schwerwiegendes Problem",
                            "schwerwiegende Probleme"
                        )
                    )
                }
            }
            RiskLevel::Low => {
                "Geringes Risiko: Keine kritischen Verstöße — Verbesserungspotenzial vorhanden."
                    .to_string()
            }
        };
        let (threshold, driven_by) = match level {
            RiskLevel::Critical => (
                60u32,
                if legal_flags > 0 {
                    "Legal Compliance"
                } else {
                    "Accessibility"
                }
                .to_string(),
            ),
            RiskLevel::High => (30u32, "Accessibility".to_string()),
            RiskLevel::Medium => (
                10u32,
                if score <= 20 {
                    "Score"
                } else {
                    "Accessibility"
                }
                .to_string(),
            ),
            RiskLevel::Low => (0u32, String::new()),
        };

        RiskAssessment {
            level,
            score: risk_score,
            threshold,
            driven_by,
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
        occurrence_counts,
        module_scores,
        overall_score,
        risk,
        principle_coverage: AccessibilityScorer::calculate_coverage(violations),
        audit_flags,
        has_screenshots: report.page_screenshots.is_some(),
        viewport_scores: report.viewport_scores.clone(),
        score_calculation_method,
        score_breakdown,
        interactive_findings: report.interactive_findings.clone(),
        accessibility_journey: report.accessibility_journey.clone(),
        advisory_findings: report.advisory_findings.clone(),
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
        raw_best_practices: report.best_practices.clone(),
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

        // severity_counts: 2 distinct findings (one per rule + severity).
        assert_eq!(norm.severity_counts.high, 1);
        assert_eq!(norm.severity_counts.medium, 1);
        assert_eq!(norm.severity_counts.total, 2);

        // occurrence_counts: 3 element occurrences (2 for 1.1.1 + 1 for 2.4.4).
        assert_eq!(norm.occurrence_counts.high, 2);
        assert_eq!(norm.occurrence_counts.medium, 1);
        assert_eq!(norm.occurrence_counts.total, 3);
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

        // Grade and certificate are both derived from overall_score so they
        // remain mutually consistent (see issue #233).
        let expected_grade = AccessibilityScorer::calculate_grade(norm.overall_score as f32);
        let expected_cert = AccessibilityScorer::calculate_certificate(norm.overall_score as f32);
        assert_eq!(norm.grade, expected_grade);
        assert_eq!(norm.certificate, expected_cert);
    }

    #[test]
    fn test_normalize_with_best_practices_produces_module_entry() {
        use crate::best_practices::{
            BestPracticesAnalysis, ConsoleErrorsAnalysis, VulnerableLibrariesAnalysis,
        };

        let mut report = AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            WcagResults::new(),
            100,
        );
        report.best_practices = Some(BestPracticesAnalysis {
            console_errors: ConsoleErrorsAnalysis {
                errors: vec![],
                warnings: vec![],
                error_count: 0,
                warning_count: 0,
            },
            vulnerable_libraries: VulnerableLibrariesAnalysis {
                detected: vec![],
                vulnerable: vec![],
                has_vulnerabilities: false,
            },
            score: 90,
        });

        let norm = normalize(&report);

        let bp_entry = norm
            .module_scores
            .iter()
            .find(|m| m.name == "Best Practices");
        assert!(
            bp_entry.is_some(),
            "Best Practices module score must be present"
        );
        let entry = bp_entry.unwrap();
        assert_eq!(entry.score, 90);
        assert!(!entry.contributes_to_overall);

        assert!(
            norm.raw_best_practices.is_some(),
            "raw_best_practices must be passed through"
        );
    }

    #[test]
    fn test_vulnerable_libraries_reduce_security_score() {
        use crate::best_practices::{
            BestPracticesAnalysis, ConsoleErrorsAnalysis, VulnerableLibrariesAnalysis,
            VulnerableLibrary,
        };
        use crate::security::{SecurityAnalysis, SecurityHeaders, SslInfo};

        let mut report = AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            WcagResults::new(),
            100,
        );
        report.security = Some(SecurityAnalysis {
            score: 80,
            grade: "B".to_string(),
            headers: SecurityHeaders::default(),
            ssl: SslInfo::default(),
            issues: vec![],
            protection: Default::default(),
            recommendations: vec![],
        });
        report.best_practices = Some(BestPracticesAnalysis {
            console_errors: ConsoleErrorsAnalysis {
                errors: vec![],
                warnings: vec![],
                error_count: 0,
                warning_count: 0,
            },
            vulnerable_libraries: VulnerableLibrariesAnalysis {
                detected: vec![],
                vulnerable: vec![
                    VulnerableLibrary {
                        name: "jQuery".to_string(),
                        version: "1.11.3".to_string(),
                        severity: "high".to_string(),
                        description: "XSS".to_string(),
                        safe_version: "3.5.0+".to_string(),
                    },
                    VulnerableLibrary {
                        name: "Lodash".to_string(),
                        version: "4.17.20".to_string(),
                        severity: "medium".to_string(),
                        description: "Prototype pollution".to_string(),
                        safe_version: "4.17.21+".to_string(),
                    },
                ],
                has_vulnerabilities: true,
            },
            score: 60,
        });

        let norm = normalize(&report);

        let sec_entry = norm.module_scores.iter().find(|m| m.name == "Security");
        assert!(sec_entry.is_some());
        // high=15 + medium=8 = 23 penalty; 80 - 23 = 57
        assert_eq!(sec_entry.unwrap().score, 57);
    }

    #[test]
    fn test_performance_results_new_fields_serialize() {
        use crate::performance::{PerformanceGrade, PerformanceScore, WebVitals};

        let perf = PerformanceResults {
            vitals: WebVitals::default(),
            score: PerformanceScore {
                overall: 80,
                grade: PerformanceGrade::Gold,
                lcp_score: None,
                fcp_score: None,
                cls_score: None,
                interactivity_score: None,
                si_score: None,
                metrics_available: 0,
            },
            render_blocking: None,
            content_weight: None,
            third_party: None,
            critical_chain: None,
            minification: None,
            animations: None,
            coverage: None,
            measurement_warnings: vec![],
        };

        let json = serde_json::to_string(&perf).expect("PerformanceResults must serialize");
        // New optional fields are skip_serializing_if = "Option::is_none" so they should be absent
        assert!(!json.contains("\"third_party\""));
        assert!(!json.contains("\"minification\""));
        assert!(!json.contains("\"coverage\""));
        assert!(!json.contains("\"animations\""));
        assert!(!json.contains("\"measurement_warnings\""));
        assert!(json.contains("\"score\""));
    }

    #[test]
    fn test_viewport_weighted_core_modules_contribute() {
        use crate::audit::{ViewportScoreSet, ViewportScores};
        use crate::{WcagLevel, WcagResults};
        let mut report = AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            WcagResults::new(),
            100,
        );
        report.viewport_scores = Some(ViewportScores {
            desktop: ViewportScoreSet {
                accessibility: 100,
                performance: None,
                overall: 100,
            },
            mobile: ViewportScoreSet {
                accessibility: 100,
                performance: None,
                overall: 100,
            },
            weighted_overall: 100,
        });
        let norm = normalize(&report);
        assert_eq!(norm.score_calculation_method, "viewport_weighted");
        let names_contributing: Vec<&str> = norm
            .module_scores
            .iter()
            .filter(|m| m.contributes_to_overall)
            .map(|m| m.name.as_str())
            .collect();
        assert!(
            names_contributing.contains(&"Accessibility"),
            "Accessibility must contribute in viewport_weighted mode, got: {:?}",
            names_contributing
        );
    }
}
