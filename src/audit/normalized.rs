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

use crate::audit::interpretation::Interpretation;
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
    /// Cookie metadata snapshot before/after consent interaction.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub consent_privacy: Option<crate::audit::ConsentPrivacySnapshot>,
    /// Whether desktop/mobile cover screenshots were captured for this audit.
    #[serde(default)]
    pub has_screenshots: bool,
    /// Per-viewport scores from dual-pass audit (70 % mobile / 30 % desktop).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub viewport_scores: Option<ViewportScores>,
    /// How `overall_score` was computed: `"module_weighted"` (standard) or
    /// `"viewport_weighted"` (dual-pass: 70 % mobile + 30 % desktop + 10 % security).
    pub score_calculation_method: String,
    /// Exact inputs used to produce `overall_score`.
    /// Present only for `viewport_weighted`; absent for `module_weighted` (module
    /// `weight_pct` values are already exact in that case).
    #[serde(default, skip_serializing_if = "Option::is_none")]
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
    /// Compact screen-reader audit (reading-order quality scores, issues, BFSG
    /// verdict). Kept separate from `findings[]` so WCAG severity counts stay
    /// rechtsrelevant; the full reading sequence stays in the sidecar JSON.
    /// Contributes to the risk score (#411).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub screen_reader: Option<crate::screen_reader::ScreenReaderSummary>,

    /// Pre-computed interpretation (evaluation texts, score bands). Always
    /// present after `normalize()`. Skipped in the `#[serde(skip)]` raw fields
    /// below so it IS serialized — consumers can read it without recomputing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interpretation: Option<Interpretation>,
}

/// In-memory wrapper for a live audit run.
///
/// Holds the serializable `NormalizedReport` plus the raw module results needed
/// by output builders. Distinct from `NormalizedReport`: a deserialized
/// `NormalizedReport` is a complete, valid snapshot without raw data, whereas
/// `AuditContext` always carries live module results alongside it.
pub struct AuditContext<'a> {
    pub normalized: NormalizedReport,
    pub raw_dual_viewport: Option<&'a crate::audit::report::DualViewportResults>,
    pub raw_performance: Option<&'a PerformanceResults>,
    pub raw_performance_desktop: Option<&'a PerformanceResults>,
    pub raw_seo: Option<&'a SeoAnalysis>,
    pub raw_security: Option<&'a SecurityAnalysis>,
    pub raw_mobile: Option<&'a MobileFriendliness>,
    pub raw_ux: Option<&'a crate::ux::UxAnalysis>,
    pub raw_journey: Option<&'a crate::journey::JourneyAnalysis>,
    pub raw_dark_mode: Option<&'a DarkModeAnalysis>,
    pub raw_source_quality: Option<&'a crate::source_quality::SourceQualityAnalysis>,
    pub raw_ai_visibility: Option<&'a crate::ai_visibility::AiVisibilityAnalysis>,
    pub raw_tech_stack: Option<&'a crate::tech_stack::TechStackAnalysis>,
    pub raw_content_visibility: Option<&'a crate::content_visibility::ContentVisibilityAnalysis>,
    pub raw_wcag: &'a WcagResults,
    pub raw_patterns: Option<&'a crate::patterns::PatternAnalysis>,
    pub raw_throttled_performance: &'a [crate::audit::report::ThrottledPerfResult],
    pub raw_best_practices: Option<&'a crate::best_practices::BestPracticesAnalysis>,
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

    /// Audit-Dimension (kanonisch englischer Label-String, für JSON)
    pub dimension: String,
    /// Subkategorie (kanonisch englischer Label-String, für JSON)
    pub subcategory: String,
    /// Issue-Klasse (kanonisch englischer Label-String, für JSON)
    pub issue_class: String,
    /// Canonical taxonomy key for the dimension — kept internal for PDF
    /// re-derivation in the runtime locale. Not serialized (JSON uses the
    /// English `dimension` label above).
    #[serde(skip)]
    pub dimension_kind: crate::taxonomy::Dimension,
    /// Canonical taxonomy key for the subcategory (see `dimension_kind`).
    #[serde(skip)]
    pub subcategory_kind: crate::taxonomy::Subcategory,
    /// Canonical taxonomy key for the issue class (see `dimension_kind`).
    #[serde(skip)]
    pub issue_class_kind: crate::taxonomy::IssueClass,
    /// Schweregrad
    pub severity: Severity,
    /// Auswirkung auf den Nutzer
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub user_impact: String,
    /// Technische Auswirkung
    #[serde(default, skip_serializing_if = "String::is_empty")]
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
    /// Detection confidence for the automated finding. Does not affect severity.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub confidence: String,
    /// Estimated false-positive risk for this automated finding.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub false_positive_risk: String,
    /// Verification wording: confirmed automatically or manual review recommended.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub verification: String,
    /// Implementation complexity class, independent from severity.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub complexity: String,
    /// Short explanation of the complexity classification.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub complexity_reason: String,
    /// Expected effect of fixing this finding group.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub expected_impact: String,
    /// Cautious BFSG/EAA relevance classification.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub bfsg_relevance: String,
    /// Execution priority label, separate from severity/risk.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub remediation_priority: String,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
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
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    #[default]
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
    /// Number of critical findings from the interactive journey layer.
    pub interactive_critical_issues: usize,
    /// Number of high-severity findings from the interactive journey layer.
    #[serde(default)]
    pub interactive_high_issues: usize,
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
            // Breadth-driven vs. volume-driven Critical (#457).
            RiskLevel::Critical if self.legal_flags >= 3 => format!(
                "Critical risk: {} WCAG Level A violations with legal relevance (BFSG). {} blocking issues on interactive controls.",
                self.legal_flags, self.blocking_issues
            ),
            RiskLevel::Critical => format!(
                "Critical risk: {} critical violations on interactive controls and content.",
                self.critical_issues
            ),
            RiskLevel::High => format!(
                "High risk: {} critical and {} severe issues. Users are actively excluded.",
                self.critical_issues, self.high_issues
            ),
            RiskLevel::Medium => {
                if self.interactive_critical_issues > 0 {
                    format!(
                        "Medium risk: {} critical interactive findings detected.",
                        self.interactive_critical_issues
                    )
                } else if self.interactive_high_issues > 0 {
                    format!(
                        "Medium risk: {} severe interactive findings detected.",
                        self.interactive_high_issues
                    )
                } else {
                    format!(
                        "Medium risk: {} severe issues detected. Limitations for certain user groups.",
                        self.high_issues + self.critical_issues
                    )
                }
            }
            RiskLevel::Low => {
                let notable = self.interactive_high_issues + self.interactive_critical_issues;
                if notable > 0 {
                    format!(
                        "Low risk: no critical violations — keyboard journey has {} requiring manual review.",
                        if notable == 1 { "1 notable finding".to_string() } else { format!("{notable} notable findings") }
                    )
                } else {
                    "Low risk: no critical violations — improvement potential remains.".to_string()
                }
            }
        }
    }
}

fn gate_certificate_by_risk(certificate: String, risk_level: &RiskLevel) -> String {
    match risk_level {
        RiskLevel::Critical => "NICHT BESTANDEN".to_string(),
        RiskLevel::High if matches!(certificate.as_str(), "STABIL" | "GUT" | "SEHR GUT") => {
            "EINGESCHRÄNKT".to_string()
        }
        _ => certificate,
    }
}

fn viewport_score_calculation_note() -> String {
    "viewport_scores.weighted_overall is the desktop/mobile blend before security; \
     overall_score is the canonical final score after the optional security blend."
        .to_string()
}

/// Transparent breakdown of how `overall_score` was computed in viewport_weighted mode.
/// Allows consumers to reproduce the exact score from its inputs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreBreakdown {
    /// Human-readable note clarifying that `viewport_scores.weighted_overall`
    /// is pre-security while `overall_score` is the canonical final score.
    pub calculation_note: String,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub security_score: Option<u32>,
    /// Weight given to security in the final formula (always 10 when security present)
    #[serde(default, skip_serializing_if = "Option::is_none")]
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
    /// WCAG finding rule ID this journey finding confirms, when it maps 1:1.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub maps_to_finding: Option<String>,
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

/// Explicit audit caveat or conflicting signal surfaced to downstream outputs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditFlag {
    pub kind: String,
    pub related_rule: Option<String>,
    pub source: String,
    pub message: String,
}

// Maximum selector-deduplicated occurrences stored per finding.
// occurrence_count always reflects the true total; this only caps what is
// serialized to keep JSON payloads compact.
const MAX_OCCURRENCES: usize = 5;

fn build_wcag_findings(violations: &[crate::wcag::Violation]) -> Vec<NormalizedFinding> {
    // Group violations by rule ID
    let mut groups: HashMap<&str, Vec<&crate::wcag::Violation>> = HashMap::new();
    for v in violations {
        groups.entry(wcag_group_key(v)).or_default().push(v);
    }

    // Build normalized findings
    let findings: Vec<NormalizedFinding> = groups
        .into_iter()
        .map(|(rule_id, violations)| {
            let first = violations[0];
            let taxonomy_rule = RuleLookup::by_legacy_wcag_id(rule_id);

            use crate::taxonomy::{Dimension, IssueClass, Subcategory};
            let (
                tax_id,
                dimension_kind,
                subcategory_kind,
                issue_class_kind,
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
                    rule.dimension,
                    rule.subcategory,
                    rule.issue_class,
                    // JSON carries the canonical English label; PDF re-derives
                    // the runtime-locale label from the *_kind fields.
                    rule.dimension.label(true).to_string(),
                    rule.subcategory.label(true).to_string(),
                    rule.issue_class.label(true).to_string(),
                    rule.user_impact_en.to_string(),
                    rule.technical_impact_en.to_string(),
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
                    Dimension::Accessibility,
                    Subcategory::ContentAlternatives,
                    IssueClass::Missing,
                    "Accessibility".to_string(),
                    "Unknown".to_string(),
                    "Unknown".to_string(),
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
            // The confidence/risk/complexity heuristics match lowercased tokens that
            // historically saw the German labels. Pass the German labels (label(false))
            // so the classification stays byte-for-byte identical now that the stored
            // string fields are canonical English (e.g. "Weak"/"Content" must not start
            // matching the English "weak"/"content" token branches).
            let subcategory_de = subcategory_kind.label(false);
            let issue_class_de = issue_class_kind.label(false);
            let confidence = derive_confidence(&tax_id, subcategory_de, issue_class_de);
            let false_positive_risk =
                derive_false_positive_risk(&tax_id, subcategory_de, issue_class_de);
            let verification = derive_verification(&false_positive_risk);
            let (complexity, complexity_reason) =
                derive_complexity(occurrence_count, &tax_id, issue_class_de);
            let expected_impact = derive_expected_impact(
                severity,
                occurrence_count,
                "wcag",
                first.level.to_string().as_str(),
            );
            let bfsg_relevance =
                derive_bfsg_relevance("wcag", first.level.to_string().as_str(), severity);
            let remediation_priority =
                derive_remediation_priority(severity, occurrence_count, &complexity);

            // Prefer the taxonomy title (canonical English) over the raw
            // rule_name from the WCAG engine — ensures JSON `title` and PDF
            // narrative refer to the same name (see issue #252). JSON stays
            // canonical English (#406); the PDF re-derives the localized title
            // from the taxonomy at render time.
            let display_title = taxonomy_rule
                .map(|r| r.title_en.to_string())
                .unwrap_or_else(|| first.rule_name.clone());

            NormalizedFinding {
                category: "wcag".to_string(),
                rule_id: tax_id.clone(),
                wcag_criterion: first.rule.clone(),
                axe_id,
                wcag_level: first.level.to_string(),
                dimension,
                subcategory,
                issue_class,
                dimension_kind,
                subcategory_kind,
                issue_class_kind,
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
                confidence,
                false_positive_risk,
                verification,
                complexity,
                complexity_reason,
                expected_impact,
                bfsg_relevance,
                remediation_priority,
                occurrences,
            }
        })
        .collect();

    findings
}

fn wcag_group_key(violation: &crate::wcag::Violation) -> &str {
    match violation.rule_id.as_deref() {
        Some(
            "frame-title"
            | "form-no-submit"
            | "landmark-main-present"
            | "landmark-unique"
            | "presentation-semantic-children",
        ) => violation
            .rule_id
            .as_deref()
            .unwrap_or(violation.rule.as_str()),
        _ => violation.rule.as_str(),
    }
}

/// (title, technical_impact) for an SEO heading-issue type, in the requested
/// language. Single source of truth: the analysis bakes English (canonical JSON),
/// the PDF presentation re-derives German at render time (#406).
pub fn seo_heading_finding_text(
    issue_type: &str,
    en: bool,
    fallback_message: &str,
) -> (String, String) {
    let title = match (issue_type, en) {
        ("long_heading", true) => "Heading too long".to_string(),
        ("long_heading", false) => "Überschrift zu lang".to_string(),
        ("missing_h1", true) => "Missing H1 heading".to_string(),
        ("missing_h1", false) => "Fehlende H1-Überschrift".to_string(),
        ("multiple_h1", true) => "Multiple H1 headings".to_string(),
        ("multiple_h1", false) => "Mehrere H1-Überschriften".to_string(),
        ("skipped_level", true) => "Skipped heading level".to_string(),
        ("skipped_level", false) => "Übersprungene Überschriftenebene".to_string(),
        ("empty_heading", true) => "Empty heading".to_string(),
        ("empty_heading", false) => "Leere Überschrift".to_string(),
        (other, _) => other.replace('_', " "),
    };
    let technical_impact = match (issue_type, en) {
        ("skipped_level", true) => "Skipped heading levels break the tree structure for screen readers and SEO crawlers — keep a logical H1→H2→H3 hierarchy.".to_string(),
        ("skipped_level", false) => "Übersprungene Heading-Ebenen zerstören die Baumstruktur für Screenreader und SEO-Crawler — logische Hierarchie H1→H2→H3 einhalten.".to_string(),
        ("missing_h1", true) => "Missing H1 heading — page purpose not recognizable for search engines and screen readers.".to_string(),
        ("missing_h1", false) => "Fehlende H1-Überschrift — Seitenzweck für Suchmaschinen und Screenreader nicht erkennbar.".to_string(),
        ("multiple_h1", true) => "Multiple H1 headings undermine the content hierarchy; search engines cannot derive a single main focus.".to_string(),
        ("multiple_h1", false) => "Mehrere H1-Überschriften untergraben die inhaltliche Hierarchie; Suchmaschinen können keinen eindeutigen Hauptfokus ableiten.".to_string(),
        ("long_heading", true) => "Overly long headings are truncated in SERPs and make quick scanning harder for users.".to_string(),
        ("long_heading", false) => "Überlange Überschriften werden in SERPs abgeschnitten und erschweren das schnelle Scannen für Nutzer.".to_string(),
        ("empty_heading", true) => "Empty headings cause navigation problems for screen readers and are treated as a poor signal by SEO crawlers.".to_string(),
        ("empty_heading", false) => "Leere Überschriften erzeugen Navigationsprobleme für Screenreader und werden von SEO-Crawlern als schlechtes Signal gewertet.".to_string(),
        _ => fallback_message.to_string(),
    };
    (title, technical_impact)
}

fn aggregate_seo_findings(
    seo: &crate::seo::SeoAnalysis,
    max_occurrences: usize,
) -> Vec<NormalizedFinding> {
    let mut heading_groups: HashMap<&str, Vec<&crate::seo::HeadingIssue>> = HashMap::new();
    for issue in &seo.headings.issues {
        heading_groups
            .entry(&issue.issue_type)
            .or_default()
            .push(issue);
    }
    let mut findings = Vec::new();
    for (issue_type, issues) in heading_groups {
        let first = issues[0];
        let occurrence_count = issues.len();
        let rule_id = format!("seo.headings.{}", issue_type);
        // Canonical English is baked into the finding (→ JSON); the PDF re-derives
        // German via `seo_heading_finding_text(.., false)` at render time (#406).
        let (title, technical_impact) = seo_heading_finding_text(issue_type, true, &first.message);
        let priority_score = calculate_priority_score(first.severity, occurrence_count, &rule_id);
        let confidence = derive_confidence(&rule_id, "Content", "issue");
        let false_positive_risk = derive_false_positive_risk(&rule_id, "Content", "issue");
        let verification = derive_verification(&false_positive_risk);
        let (complexity, complexity_reason) =
            derive_complexity(occurrence_count, &rule_id, "issue");
        let expected_impact = derive_expected_impact(first.severity, occurrence_count, "seo", "");
        let bfsg_relevance = derive_bfsg_relevance("seo", "", first.severity);
        let remediation_priority =
            derive_remediation_priority(first.severity, occurrence_count, &complexity);
        findings.push(NormalizedFinding {
            category: "seo".to_string(),
            rule_id: rule_id.clone(),
            wcag_criterion: String::new(),
            axe_id: None,
            wcag_level: String::new(),
            dimension: "SEO".to_string(),
            subcategory: "Content".to_string(),
            issue_class: "issue".to_string(),
            dimension_kind: crate::taxonomy::Dimension::Seo,
            subcategory_kind: crate::taxonomy::Subcategory::ContentStructure,
            issue_class_kind: crate::taxonomy::IssueClass::Weak,
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
            confidence,
            false_positive_risk,
            verification,
            complexity,
            complexity_reason,
            expected_impact,
            bfsg_relevance,
            remediation_priority,
            occurrences: issues
                .iter()
                .take(max_occurrences)
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
    findings
}

fn filter_aria_hidden_interactive(
    interactive_findings: &mut Vec<InteractiveFinding>,
    findings: &[NormalizedFinding],
    wcag_violations: &[crate::wcag::Violation],
) {
    let mut aria_hidden_selectors: std::collections::HashSet<String> = wcag_violations
        .iter()
        .filter(|v| v.rule == "aria-hidden-focus")
        .filter_map(|v| v.selector.clone())
        .collect();

    aria_hidden_selectors.extend(
        findings
            .iter()
            .filter(|f| f.rule_id == "a11y.aria_hidden_focus.invalid")
            .flat_map(|f| f.occurrences.iter().filter_map(|o| o.selector.clone())),
    );

    if aria_hidden_selectors.is_empty() {
        return;
    }

    fn normalize_selector(sel: &str) -> String {
        let mut normalized = String::new();
        let mut chars = sel.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '.' {
                while let Some(&next_c) = chars.peek() {
                    if next_c.is_alphanumeric() || next_c == '-' || next_c == '_' {
                        chars.next();
                    } else {
                        break;
                    }
                }
            } else if c.is_alphabetic() {
                let mut tag = String::new();
                tag.push(c);
                while let Some(&next_c) = chars.peek() {
                    if next_c.is_alphanumeric() || next_c == '-' || next_c == '_' {
                        tag.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                if chars.peek() == Some(&'#') {
                    // discard tag prefix before ID
                } else {
                    normalized.push_str(&tag);
                }
            } else if c == ' ' {
                let is_around_gt = normalized.ends_with('>') || chars.peek() == Some(&'>');
                if !is_around_gt {
                    normalized.push(' ');
                }
            } else {
                normalized.push(c);
            }
        }
        normalized.trim().replace(" >", ">").replace("> ", ">")
    }

    fn extract_selector_from_message(message: &str) -> Option<String> {
        let start_idx = message.find('(')?;
        let mut depth = 0;
        let mut end_idx = None;
        for (i, c) in message[start_idx..].char_indices() {
            if c == '(' {
                depth += 1;
            } else if c == ')' {
                depth -= 1;
                if depth == 0 {
                    end_idx = Some(start_idx + i);
                    break;
                }
            }
        }
        let mut sel = &message[start_idx + 1..end_idx?];
        if let Some(colon_idx) = sel.find(": display:") {
            sel = &sel[..colon_idx];
        } else if let Some(colon_idx) = sel.find(": visibility:") {
            sel = &sel[..colon_idx];
        } else if let Some(colon_idx) = sel.find(": opacity:") {
            sel = &sel[..colon_idx];
        }
        Some(sel.trim().to_string())
    }

    let normalized_aria_hidden_selectors: std::collections::HashSet<String> = aria_hidden_selectors
        .iter()
        .map(|s| normalize_selector(s))
        .collect();

    interactive_findings.retain(|inf| {
        if inf.category == "HiddenFocusable" {
            if let Some(sel) = extract_selector_from_message(&inf.message) {
                let norm_sel = normalize_selector(&sel);
                !normalized_aria_hidden_selectors.iter().any(|s| {
                    norm_sel == *s
                        || norm_sel.ends_with(&format!(">{}", s))
                        || s.ends_with(&format!(">{}", norm_sel))
                })
            } else {
                true
            }
        } else {
            true
        }
    });
}

fn build_module_scores(
    report: &AuditReport,
    occurrence_counts: &SeverityCounts,
    vuln_security_penalty: u32,
) -> Vec<ModuleScoreEntry> {
    let score = report.score.round().max(1.0) as u32;
    let accessibility_grade = AccessibilityScorer::calculate_grade(report.score).to_string();

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
            // Indicator module: does not feed the overall score, so its weight is
            // 0 — a non-zero weight on a non-contributing module made the weight
            // column sum to >100% (#447).
            weight_pct: 0,
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
            // Indicator module — weight 0, see UX note above (#447).
            weight_pct: 0,
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

    // Indicator modules that compute a 0–100 score but do not feed the overall
    // score. Previously their score was serialized raw with no grade and no
    // entry here, so the report showed a bare number with no relation to the
    // rest (#447). They now appear consistently as graded, non-contributing
    // indicators (weight 0).
    let mut push_indicator = |name: &str, score: u32, measurement_type: &str| {
        module_scores.push(ModuleScoreEntry {
            name: name.to_string(),
            score,
            grade: AccessibilityScorer::calculate_grade(score as f32).to_string(),
            weight_pct: 0,
            contributes_to_overall: false,
            measurement_type: measurement_type.to_string(),
        });
    };
    if let Some(ref dm) = report.dark_mode {
        push_indicator("Dark Mode", dm.score, "measured");
    }
    if let Some(ref ai) = report.ai_visibility {
        push_indicator("AI Visibility", ai.score, "heuristic");
    }
    if let Some(ref sq) = report.source_quality {
        push_indicator("Source Quality", sq.score, "heuristic");
    }
    if let Some(ref cv) = report.content_visibility {
        if cv.signal_count > 0 {
            let cv_score = (cv.signal_count.saturating_sub(cv.problem_count) as u32 * 100)
                / cv.signal_count as u32;
            push_indicator("Content Visibility", cv_score, "heuristic");
        }
    }
    // Tech Stack is detection-only — no score in module_scores.
    // Stack-specific security findings (WordPress admin exposure, etc.) flow
    // into the Security module score instead.

    module_scores
}

fn compute_risk_assessment(
    findings: &[NormalizedFinding],
    occurrence_counts: &SeverityCounts,
    interactive_findings: &[InteractiveFinding],
    screen_reader: Option<&crate::screen_reader::ScreenReaderSummary>,
    score: u32,
    overall_score: u32,
) -> RiskAssessment {
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

    // Blocking issues: interactive elements without accessible names (4.1.2/2.1.1).
    // Only Medium+ severity — Low findings (e.g. accordion advisory) are not blockers.
    let blocking_issues = findings
        .iter()
        .filter(|f| {
            (f.wcag_criterion == "4.1.2" || f.wcag_criterion == "2.1.1")
                && matches!(
                    f.severity,
                    Severity::Medium | Severity::High | Severity::Critical
                )
        })
        .map(|f| f.occurrence_count)
        .sum::<usize>();
    let interactive_critical_issues = interactive_findings
        .iter()
        .filter(|f| f.severity == Severity::Critical)
        .count();
    let interactive_high_issues = interactive_findings
        .iter()
        .filter(|f| f.severity == Severity::High)
        .count();

    // Screen-reader audit issues are heuristic quality signals (reading order,
    // landmark/heading quality). They contribute to the risk score but never to
    // the legally-relevant severity_counts/legal_flags (#411). SR emits no
    // Critical severity — only low/medium/high strings.
    let (sr_high_issues, sr_medium_issues) = screen_reader
        .map(|sr| (sr.count_severity("high"), sr.count_severity("medium")))
        .unwrap_or((0, 0));

    let risk_score = (legal_flags as u32 * 20
        + critical_issues as u32 * 10
        + high_issues as u32 * 3
        + blocking_issues as u32 * 2
        + interactive_critical_issues as u32 * 10
        + interactive_high_issues as u32 * 5
        + sr_high_issues as u32 * 3
        + sr_medium_issues as u32)
        .min(100);

    // Risk level — explicit precedence; legal_flags and blocking_issues both
    // raise the floor even when critical_issues is zero (see issue #250).
    //
    // "Critical" is reserved for *systemic* legal exposure: breadth (≥3 distinct
    // WCAG Level A rules with High/Critical severity) OR volume (≥5 critical
    // occurrences). The breadth path must NOT also require a critical occurrence
    // — legal_flags already counts High-severity Level A rules, so gating it on
    // `critical_issues > 0` made a site with 4 High-severity legal barriers rank
    // *below* one with a single flag but 5+ critical occurrences (#457). A single
    // isolated legal flag with a few critical occurrences stays High (#250).
    let level = if legal_flags >= 3 || critical_issues >= 5 {
        RiskLevel::Critical
    } else if (legal_flags > 0 && critical_issues > 0)
        || critical_issues >= 3
        || blocking_issues >= 5
        || risk_score >= 80
    {
        RiskLevel::High
    } else if (high_issues >= 3 && score < 80)
        || critical_issues >= 1
        || legal_flags > 0
        || blocking_issues >= 1
        || interactive_critical_issues > 0
        || interactive_high_issues > 0
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
                "Critical risk despite high overall score"
            } else {
                "Critical risk"
            };
            if legal_flags >= 3 {
                // Breadth-driven: multiple distinct legally-relevant barriers.
                format!(
                    "{}: {} with legal relevance (BFSG). {}.",
                    prefix,
                    plural(legal_flags, "WCAG Level A violation", "WCAG Level A violations"),
                    plural(
                        blocking_issues,
                        "blocker on interactive elements",
                        "blockers on interactive elements"
                    )
                )
            } else {
                // Volume-driven: a high number of critical occurrences.
                format!(
                    "{}: {} on interactive elements and content.",
                    prefix,
                    plural(critical_issues, "critical violation", "critical violations")
                )
            }
        }
        RiskLevel::High => format!(
            "High risk: {} and {}. Users are actively excluded.",
            plural(critical_issues, "critical issue", "critical issues"),
            plural(
                high_issues,
                "serious issue",
                "serious issues"
            )
        ),
        RiskLevel::Medium => {
            if legal_flags > 0 {
                format!(
                    "Medium risk: {} with legal relevance (BFSG){}.",
                    plural(legal_flags, "WCAG Level A violation", "WCAG Level A violations"),
                    if blocking_issues > 0 {
                        format!(
                            ", {} on interactive elements",
                            plural(blocking_issues, "blocker", "blockers")
                        )
                    } else {
                        String::new()
                    }
                )
            } else if blocking_issues > 0 {
                format!(
                    "Medium risk: {} detected on interactive elements. \
                     Restrictions for certain user groups.",
                    plural(blocking_issues, "blocker", "blockers")
                )
            } else if interactive_critical_issues > 0 {
                format!(
                    "Medium risk: {} from interactive keyboard and state-transition tests.",
                    plural(
                        interactive_critical_issues,
                        "critical finding",
                        "critical findings"
                    )
                )
            } else if interactive_high_issues > 0 {
                format!(
                    "Medium risk: {} from interactive keyboard and state-transition tests.",
                    plural(
                        interactive_high_issues,
                        "serious finding",
                        "serious findings"
                    )
                )
            } else {
                format!(
                    "Medium risk: {} detected. Restrictions for certain user groups.",
                    plural(
                        high_issues + critical_issues,
                        "serious issue",
                        "serious issues"
                    )
                )
            }
        }
        RiskLevel::Low => {
            let notable = interactive_high_issues + interactive_critical_issues;
            if notable > 0 {
                format!(
                    "Low risk: No critical violations — keyboard journey contains {}, manual review recommended.",
                    plural(notable, "notable finding", "notable findings")
                )
            } else {
                "Low risk: No critical violations — improvement potential exists."
                    .to_string()
            }
        }
    };
    let (threshold, driven_by) = match level {
        RiskLevel::Critical => (
            60u32,
            // Attribute to the condition that actually triggered Critical:
            // breadth of legal exposure (≥3 distinct Level A rules) vs. a high
            // volume of critical occurrences. A single flag that rode in on
            // volume must not be labelled "Legal Compliance" (#457).
            if legal_flags >= 3 {
                "Legal Compliance"
            } else {
                "Accessibility"
            }
            .to_string(),
        ),
        RiskLevel::High => (30u32, "Accessibility".to_string()),
        RiskLevel::Medium => (
            10u32,
            if interactive_critical_issues > 0 {
                "Accessibility Journey"
            } else if score <= 20 {
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
        interactive_critical_issues,
        interactive_high_issues,
        summary,
    }
}

/// Normalisiert einen rohen AuditReport.
///
/// - Gruppert Violations nach Regel-ID
/// - Reichert mit Taxonomie-Feldern an (via RuleLookup)
/// - Berechnet Grade/Certificate aus korrigiertem Score
pub fn normalize<'a>(report: &'a AuditReport) -> AuditContext<'a> {
    let violations = &report.wcag_results.violations;

    let seo_reports_lang = report.seo.as_ref().is_some_and(|s| s.technical.has_lang);
    let had_311 = violations.iter().any(|v| v.rule == "3.1.1");

    let mut findings = build_wcag_findings(violations);
    if let Some(seo) = &report.seo {
        findings.extend(aggregate_seo_findings(seo, MAX_OCCURRENCES));
    }

    // Sort by priority score (highest first), then by severity
    findings.sort_by(|a, b| {
        b.priority_score
            .partial_cmp(&a.priority_score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| b.severity.cmp(&a.severity))
    });

    let mut interactive_findings = report.interactive_findings.clone();
    filter_aria_hidden_interactive(&mut interactive_findings, &findings, violations);

    let score = report.score.round().max(1.0) as u32;

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

    let module_scores = build_module_scores(report, &occurrence_counts, vuln_security_penalty);

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
                calculation_note: viewport_score_calculation_note(),
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
                "Consent banner detected and automatically dismissed{}. Audit results reflect the page content after consent.",
                report.consent_banner_cmp.as_ref().map(|c| format!(" ({})", c)).unwrap_or_default()
            )
        } else {
            format!(
                "Consent banner detected{} — audit performed without consent. Accessibility and SEO results may be incomplete. Recommendation: use --dismiss-consent.",
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
                        "Possible consent wall artifact: {} violations with only {} analyzed nodes. \
                         Scores may be measuring the consent dialog rather than the actual page content. \
                         Recommendation: re-run the audit with --dismiss-consent.",
                        violation_count, report.nodes_analyzed
                    ),
                });
            }
        }
    }

    // Skip-link functional failure: static bypass_blocks check passes when a skip link
    // exists, but the journey may find that it does not actually move focus. If the
    // journey detected a broken skip link and no static bypass_blocks violation was
    // raised, emit an audit_flag to surface the discrepancy explicitly.
    let has_broken_skip_link = interactive_findings
        .iter()
        .any(|f| f.category == "SkipLink");
    let has_bypass_blocks_violation = findings
        .iter()
        .any(|f| f.rule_id == "a11y.bypass_blocks.missing");
    if has_broken_skip_link && !has_bypass_blocks_violation {
        audit_flags.push(AuditFlag {
            kind: "bypass_blocks_untested".to_string(),
            related_rule: Some("a11y.bypass_blocks.missing".to_string()),
            source: "a11y_journey.skip_link".to_string(),
            message: "A skip link is present (WCAG 2.4.1 static check passed) but the \
                journey found it does not move keyboard focus to the target. The page \
                effectively fails WCAG 2.4.1 — verify and fix the skip-link target."
                .to_string(),
        });
    }

    let screen_reader = report
        .screen_reader_audit
        .as_ref()
        .map(crate::screen_reader::ScreenReaderSummary::from_report);

    // ── Risk Assessment (independent from score) ──────────────────
    let risk = compute_risk_assessment(
        &findings,
        &occurrence_counts,
        &interactive_findings,
        screen_reader.as_ref(),
        score,
        overall_score,
    );
    let certificate = gate_certificate_by_risk(certificate, &risk.level);

    let normalized_data = NormalizedReport {
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
        consent_privacy: report.consent_privacy.clone(),
        has_screenshots: report.page_screenshots.is_some(),
        viewport_scores: report.viewport_scores.clone(),
        score_calculation_method,
        score_breakdown,
        interactive_findings,
        accessibility_journey: report.accessibility_journey.clone(),
        screen_reader,
        interpretation: None,
    };
    let mut ctx = AuditContext {
        normalized: normalized_data,
        raw_dual_viewport: report.dual_viewport.as_ref(),
        raw_performance: report.performance.as_ref(),
        raw_performance_desktop: report
            .dual_viewport
            .as_ref()
            .and_then(|d| d.desktop.performance.as_ref()),
        raw_seo: report.seo.as_ref(),
        raw_security: report.security.as_ref(),
        raw_mobile: report.mobile.as_ref(),
        raw_ux: report.ux.as_ref(),
        raw_journey: report.journey.as_ref(),
        raw_dark_mode: report.dark_mode.as_ref(),
        raw_source_quality: report.source_quality.as_ref(),
        raw_ai_visibility: report.ai_visibility.as_ref(),
        raw_tech_stack: report.tech_stack.as_ref(),
        raw_content_visibility: report.content_visibility.as_ref(),
        raw_wcag: &report.wcag_results,
        raw_patterns: report.patterns.as_ref(),
        raw_throttled_performance: &report.throttled_performance,
        raw_best_practices: report.best_practices.as_ref(),
    };
    ctx.normalized.interpretation = Some(Interpretation::from_context(&ctx));
    ctx
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

fn derive_confidence(rule_id: &str, subcategory: &str, issue_class: &str) -> String {
    let key = format!(
        "{} {} {}",
        rule_id.to_ascii_lowercase(),
        subcategory.to_ascii_lowercase(),
        issue_class.to_ascii_lowercase()
    );
    if key.contains("alt_text.weak")
        || key.contains("understand")
        || key.contains("readability")
        || key.contains("content")
    {
        "medium".to_string()
    } else if key.contains("aria")
        || key.contains("heading")
        || key.contains("landmark")
        || key.contains("focus")
    {
        "high".to_string()
    } else {
        "very_high".to_string()
    }
}

fn derive_false_positive_risk(rule_id: &str, subcategory: &str, issue_class: &str) -> String {
    let key = format!(
        "{} {} {}",
        rule_id.to_ascii_lowercase(),
        subcategory.to_ascii_lowercase(),
        issue_class.to_ascii_lowercase()
    );
    if key.contains("weak")
        || key.contains("alt_text.weak")
        || key.contains("understand")
        || key.contains("content")
    {
        "medium".to_string()
    } else if key.contains("aria") || key.contains("heading") || key.contains("landmark") {
        "low".to_string()
    } else {
        "very_low".to_string()
    }
}

fn derive_verification(false_positive_risk: &str) -> String {
    match false_positive_risk {
        "medium" | "high" => "manual_review_recommended",
        _ => "automatically_confirmed",
    }
    .to_string()
}

fn derive_complexity(
    occurrence_count: usize,
    rule_id: &str,
    issue_class: &str,
) -> (String, String) {
    let key = format!(
        "{} {}",
        rule_id.to_ascii_lowercase(),
        issue_class.to_ascii_lowercase()
    );
    if occurrence_count >= 10 {
        (
            "high".to_string(),
            format!(
                "{} occurrences indicate a component- or template-level issue.",
                occurrence_count
            ),
        )
    } else if key.contains("aria") || key.contains("focus") || key.contains("keyboard") {
        (
            "medium".to_string(),
            "The fix is technical but affects a limited number of patterns.".to_string(),
        )
    } else if occurrence_count >= 5 {
        (
            "medium".to_string(),
            format!(
                "{} occurrences require consistent updates across content or templates.",
                occurrence_count
            ),
        )
    } else {
        (
            "low".to_string(),
            "Few occurrences and a clearly scoped fix.".to_string(),
        )
    }
}

fn derive_expected_impact(
    severity: Severity,
    occurrence_count: usize,
    category: &str,
    wcag_level: &str,
) -> String {
    let score_effect = match (severity, occurrence_count) {
        (Severity::Critical | Severity::High, n) if n >= 5 => "high",
        (Severity::Critical | Severity::High, _) => "medium",
        (_, n) if n >= 10 => "medium",
        _ => "low",
    };
    if category == "wcag" {
        format!(
            "Fixes {} occurrences; expected score impact: {}; WCAG level: {}.",
            occurrence_count, score_effect, wcag_level
        )
    } else {
        format!(
            "Fixes {} occurrences; expected visibility/structure impact: {}.",
            occurrence_count, score_effect
        )
    }
}

fn derive_bfsg_relevance(category: &str, wcag_level: &str, severity: Severity) -> String {
    if category != "wcag" {
        return "low".to_string();
    }
    match (wcag_level, severity) {
        ("A", Severity::Critical | Severity::High) => "high",
        ("A" | "AA", _) => "medium",
        _ => "low",
    }
    .to_string()
}

fn derive_remediation_priority(
    severity: Severity,
    occurrence_count: usize,
    complexity: &str,
) -> String {
    match (severity, occurrence_count, complexity) {
        (Severity::Critical, _, _) => "immediate",
        (Severity::High, _, "low") => "quick_win",
        (Severity::High, _, _) => "high",
        (Severity::Medium, n, _) if n >= 10 => "high",
        (Severity::Medium, _, "low") => "quick_win",
        _ => "normal",
    }
    .to_string()
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

        assert_eq!(norm.normalized.score, 100);
        assert_eq!(norm.normalized.grade, "A");
        assert_eq!(norm.normalized.certificate, "SEHR GUT");
        assert!(norm.normalized.findings.is_empty());
        assert_eq!(norm.normalized.severity_counts.total, 0);
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

        assert_eq!(norm.normalized.findings.len(), 2);
        let alt = norm
            .normalized
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

        let finding = &norm.normalized.findings[0];
        assert_eq!(finding.rule_id, "a11y.alt_text.missing");
        assert_eq!(finding.dimension, "Accessibility");
        assert_eq!(finding.subcategory, "Content & Alternatives");
        assert_eq!(finding.issue_class, "Missing");
        assert!(finding.score_impact.base_penalty > 0.0);
        assert!(finding.score_impact.max_penalty >= finding.score_impact.base_penalty);
        assert!(!finding.score_impact.scaling.is_empty());
        assert!(!finding.user_impact.is_empty());
        // JSON-stored finding text must be canonical English (#406).
        assert_eq!(finding.title, "Missing alternative text on images");
        assert_eq!(
            finding.user_impact,
            "Screen reader users receive no image information."
        );
        assert_eq!(finding.technical_impact, "Non-conformant image markup.");
        // Guard: no German diacritics leak into the canonical-English JSON.
        for field in [
            &finding.title,
            &finding.user_impact,
            &finding.technical_impact,
        ] {
            assert!(
                !field.chars().any(|c| "äöüÄÖÜß".contains(c)),
                "canonical-English JSON field contains German diacritics: {field}"
            );
        }
    }

    #[test]
    fn test_frame_title_keeps_wcag_241_and_specific_taxonomy() {
        let mut results = WcagResults::new();
        results.add_violation(
            Violation::new(
                "2.4.1",
                "Frame title",
                WcagLevel::A,
                Severity::High,
                "Iframe is missing an accessible name",
                "iframe:nth-of-type(1)",
            )
            .with_rule_id("frame-title"),
        );
        results.add_violation(
            Violation::new(
                "2.4.1",
                "Bypass Blocks",
                WcagLevel::A,
                Severity::High,
                "No bypass mechanism found",
                "document",
            )
            .with_rule_id("bypass"),
        );

        let report = AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            results,
            100,
        );
        let norm = normalize(&report);

        let frame = norm
            .normalized
            .findings
            .iter()
            .find(|f| f.rule_id == "a11y.frame_title.missing")
            .expect("frame-title finding should keep its own taxonomy rule");
        assert_eq!(frame.wcag_criterion, "2.4.1");
        assert_eq!(frame.axe_id.as_deref(), Some("frame-title"));

        assert!(norm
            .normalized
            .findings
            .iter()
            .any(|f| f.rule_id == "a11y.bypass_blocks.missing"));
    }

    #[test]
    fn test_dom_parity_rules_keep_specific_taxonomy() {
        let mut results = WcagResults::new();
        results.add_violation(
            Violation::new(
                "3.2.2",
                "On Input",
                WcagLevel::A,
                Severity::Medium,
                "Form has input controls but no explicit submit button",
                "form",
            )
            .with_rule_id("form-no-submit"),
        );
        results.add_violation(
            Violation::new(
                "1.3.1",
                "Info and Relationships",
                WcagLevel::A,
                Severity::Medium,
                "Presentational container contains semantic child",
                "div[role=\"presentation\"]",
            )
            .with_rule_id("presentation-semantic-children"),
        );
        results.add_violation(
            Violation::new(
                "1.3.1",
                "Landmark Main Present",
                WcagLevel::A,
                Severity::High,
                "Page has no main landmark",
                "document",
            )
            .with_rule_id("landmark-main-present"),
        );
        results.add_violation(
            Violation::new(
                "1.3.1",
                "Landmark Unique",
                WcagLevel::A,
                Severity::Medium,
                "Multiple navigation landmarks share the same accessible name",
                "nav",
            )
            .with_rule_id("landmark-unique"),
        );

        let report = AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            results,
            100,
        );
        let norm = normalize(&report);

        let form = norm
            .normalized
            .findings
            .iter()
            .find(|f| f.rule_id == "a11y.form_no_submit.missing")
            .expect("form-no-submit finding should keep its own taxonomy rule");
        assert_eq!(form.wcag_criterion, "3.2.2");
        assert_eq!(form.axe_id.as_deref(), Some("form-no-submit"));

        let presentation = norm
            .normalized
            .findings
            .iter()
            .find(|f| f.rule_id == "a11y.presentation_semantic_children.invalid")
            .expect("presentation-semantic-children should keep its own taxonomy rule");
        assert_eq!(presentation.wcag_criterion, "1.3.1");
        assert_eq!(
            presentation.axe_id.as_deref(),
            Some("presentation-semantic-children")
        );

        assert!(norm
            .normalized
            .findings
            .iter()
            .any(|f| f.rule_id == "a11y.landmark_main.missing"
                && f.axe_id.as_deref() == Some("landmark-main-present")));
        assert!(norm
            .normalized
            .findings
            .iter()
            .any(|f| f.rule_id == "a11y.landmark_unique.invalid"
                && f.axe_id.as_deref() == Some("landmark-unique")));
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
        assert_eq!(norm.normalized.severity_counts.high, 1);
        assert_eq!(norm.normalized.severity_counts.medium, 1);
        assert_eq!(norm.normalized.severity_counts.total, 2);

        // occurrence_counts: 3 element occurrences (2 for 1.1.1 + 1 for 2.4.4).
        assert_eq!(norm.normalized.occurrence_counts.high, 2);
        assert_eq!(norm.normalized.occurrence_counts.medium, 1);
        assert_eq!(norm.normalized.occurrence_counts.total, 3);
    }

    #[test]
    fn critical_interactive_finding_raises_risk_without_wcag_counts() {
        let mut report = AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            WcagResults::new(),
            100,
        );
        report.interactive_findings.push(InteractiveFinding {
            category: "FocusTrap".to_string(),
            maps_to_finding: None,
            severity: Severity::Critical,
            journey: "modal".to_string(),
            before_snapshot_label: None,
            after_snapshot_label: None,
            message: "Modal has no focus trap.".to_string(),
            fix_suggestion: None,
        });

        let norm = normalize(&report);

        assert_eq!(norm.normalized.risk.level, RiskLevel::Medium);
        assert_eq!(norm.normalized.severity_counts.total, 0);
        assert_eq!(norm.normalized.score, 100);
        assert_eq!(norm.normalized.risk.interactive_critical_issues, 1);
    }

    #[test]
    fn high_interactive_finding_raises_risk_without_wcag_counts() {
        let mut report = AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            WcagResults::new(),
            100,
        );
        report.interactive_findings.push(InteractiveFinding {
            category: "SkipLink".to_string(),
            maps_to_finding: None,
            severity: Severity::High,
            journey: "skip-link".to_string(),
            before_snapshot_label: None,
            after_snapshot_label: None,
            message: "Skip link is present but does not move focus to the target.".to_string(),
            fix_suggestion: None,
        });

        let norm = normalize(&report);

        assert_eq!(norm.normalized.risk.level, RiskLevel::Medium);
        assert_eq!(norm.normalized.risk.score, 5);
        assert_eq!(norm.normalized.risk.interactive_high_issues, 1);
        assert_eq!(norm.normalized.severity_counts.total, 0);
        assert_eq!(norm.normalized.score, 100);
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
        assert_eq!(norm_no_seo.normalized.findings.len(), 1);
        assert!(norm_no_seo.normalized.audit_flags.is_empty());

        // With SEO indicating has_lang — 3.1.1 should remain but be marked as a conflicting signal
        report.seo = Some(crate::seo::SeoAnalysis {
            technical: crate::seo::TechnicalSeo {
                has_lang: true,
                ..Default::default()
            },
            ..Default::default()
        });
        let norm_with_seo = normalize(&report);
        assert_eq!(norm_with_seo.normalized.findings.len(), 1);
        assert_eq!(norm_with_seo.normalized.score, report.score.round() as u32);
        assert_eq!(norm_with_seo.normalized.audit_flags.len(), 1);
        assert_eq!(
            norm_with_seo.normalized.audit_flags[0]
                .related_rule
                .as_deref(),
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
        let expected_grade =
            AccessibilityScorer::calculate_grade(norm.normalized.overall_score as f32);
        let expected_cert =
            AccessibilityScorer::calculate_certificate(norm.normalized.overall_score as f32);
        assert_eq!(norm.normalized.grade, expected_grade);
        assert_eq!(norm.normalized.certificate, expected_cert);
    }

    #[test]
    fn high_risk_vetoes_positive_certificate() {
        assert_eq!(
            gate_certificate_by_risk("STABIL".to_string(), &RiskLevel::High),
            "EINGESCHRÄNKT"
        );
        assert_eq!(
            gate_certificate_by_risk("SEHR GUT".to_string(), &RiskLevel::Critical),
            "NICHT BESTANDEN"
        );
        assert_eq!(
            gate_certificate_by_risk("AUSBAUFÄHIG".to_string(), &RiskLevel::High),
            "AUSBAUFÄHIG"
        );
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
            .normalized
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

        let sec_entry = norm
            .normalized
            .module_scores
            .iter()
            .find(|m| m.name == "Security");
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
        assert_eq!(
            norm.normalized.score_calculation_method,
            "viewport_weighted"
        );
        assert!(norm
            .normalized
            .score_breakdown
            .as_ref()
            .is_some_and(|b| b.calculation_note.contains("canonical final score")));
        let names_contributing: Vec<&str> = norm
            .normalized
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
