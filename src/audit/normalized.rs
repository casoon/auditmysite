//! Normalized Audit Model — Single Source of Truth für alle Outputs
//!
//! Transformiert den rohen AuditReport in ein normalisiertes Modell mit:
//! - Korrigiertem Score (nach Suppressions)
//! - Taxonomie-angereichertem Findings
//! - Einheitlicher Severity-Terminologie
//! - Konsistenter Grade/Certificate-Berechnung

use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::audit::interpretation::Interpretation;
use crate::audit::module_scores::build_module_scores;
use crate::audit::report::{AuditReport, PerformanceResults, ViewportScores};
use crate::audit::risk_assessment::compute_risk_assessment;
use crate::audit::scoring::{AccessibilityScorer, PrincipleCoverage};
use crate::cli::WcagLevel;
use crate::dark_mode::DarkModeAnalysis;
use crate::mobile::MobileFriendliness;
use crate::security::SecurityAnalysis;
use crate::seo::SeoAnalysis;
use crate::taxonomy::{ReportVisibility, RuleLookup, Scaling, Severity};
use crate::wcag::WcagResults;

use crate::audit::finding_derive::{
    calculate_priority_score, derive_bfsg_relevance, derive_complexity, derive_confidence,
    derive_expected_impact, derive_false_positive_risk, derive_remediation_priority,
    derive_verification,
};
pub use crate::audit::finding_derive::{
    complexity_text, expected_impact_text, ComplexityKind, ExpectedImpactKind, ScoreEffect,
};
pub use crate::audit::interactive_finding::{
    finding_uncertainty_text, interactive_finding_text, AccessibilityJourney, FindingUncertainty,
    InteractiveFinding, InteractiveFindingKind, InteractiveFindingValues, JourneyExecution,
    JourneyRun, JourneyStep, JourneyTrace,
};

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
    /// Grade aus `score` (Barrierefreiheit) — dem Gegenstand des Berichts.
    /// Siehe plan 29, D1.
    pub grade: String,
    /// Certificate aus `score`, gegebenenfalls durch das Risiko-Veto begrenzt
    pub certificate: String,

    /// Normalisierte, gruppierte Findings mit Taxonomie-Feldern
    pub findings: Vec<NormalizedFinding>,
    /// Severity-Zähler — zählt **Findings** (eine Zeile pro Regel + Severity).
    pub severity_counts: SeverityCounts,
    /// Severity-Zähler — zählt **Element-Occurrences** (alle betroffenen Elemente).
    #[serde(default)]
    pub occurrence_counts: SeverityCounts,

    /// Non-violation accessibility signals that remain actionable: heuristic
    /// warnings, manual review items and detected positive patterns.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub accessibility_assessments: Vec<AccessibilityAssessment>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rule_outcomes: Vec<crate::wcag::RuleRun>,

    /// Requested scope, execution provenance and completeness qualification.
    #[serde(default)]
    pub execution: crate::audit::AuditExecution,

    /// Modul-Scores
    pub module_scores: Vec<ModuleScoreEntry>,
    /// Gewichteter Gesamtscore über alle aktiven Module
    pub overall_score: u32,

    /// Per-subcategory Accessibility score breakdown (plan/5-module-
    /// accessibility-security-driver-detail.md) — explains what specifically
    /// drives the single Accessibility module score.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub accessibility_subcategory_scores: Vec<SubcategoryScoreEntry>,
    /// Per-category Security score breakdown, same purpose as
    /// `accessibility_subcategory_scores` for the Security module. Empty
    /// when the Security module didn't run.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub security_category_scores: Vec<SecurityCategoryScoreEntry>,

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
#[derive(Clone)]
pub struct AuditContext<'a> {
    pub normalized: NormalizedReport,
    pub raw_dual_viewport: Option<&'a crate::audit::report::DualViewportResults>,
    pub raw_performance: Option<&'a PerformanceResults>,
    pub raw_performance_desktop: Option<&'a PerformanceResults>,
    pub raw_seo: Option<&'a SeoAnalysis>,
    pub raw_security: Option<&'a SecurityAnalysis>,
    pub raw_html_conform: Option<&'a crate::html_conform::HtmlConformAnalysis>,
    pub raw_mobile: Option<&'a MobileFriendliness>,
    pub raw_ux: Option<&'a crate::ux::UxAnalysis>,
    pub raw_journey: Option<&'a crate::journey::JourneyAnalysis>,
    pub raw_dark_mode: Option<&'a DarkModeAnalysis>,
    pub raw_design_quality: Option<&'a crate::design_quality::DesignQualityAnalysis>,
    pub raw_ai_transparency: Option<&'a crate::ai_transparency::AiTransparencyAnalysis>,
    pub raw_network_dns: Option<&'a crate::network::dns::NetworkDnsAnalysis>,
    pub raw_source_quality: Option<&'a crate::source_quality::SourceQualityAnalysis>,
    pub raw_ai_visibility: Option<&'a crate::ai_visibility::AiVisibilityAnalysis>,
    pub raw_tech_stack: Option<&'a crate::tech_stack::TechStackAnalysis>,
    pub raw_content_visibility: Option<&'a crate::content_visibility::ContentVisibilityAnalysis>,
    pub raw_wcag: &'a WcagResults,
    pub raw_patterns: Option<&'a crate::patterns::PatternAnalysis>,
    pub raw_throttled_performance: &'a [crate::audit::report::ThrottledPerfResult],
    pub raw_best_practices: Option<&'a crate::best_practices::BestPracticesAnalysis>,
    pub raw_commerce: Option<&'a crate::commerce::CommerceAnalysis>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessibilityAssessment {
    pub kind: String,
    pub rule_id: String,
    pub wcag_criterion: String,
    pub severity: Severity,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fix_suggestion: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub viewport: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<crate::wcag::Evidence>,
}

pub(crate) fn normalize_assessments(results: &WcagResults) -> Vec<AccessibilityAssessment> {
    let sources = [
        ("warning", results.warnings.as_slice()),
        ("manual_review", results.not_testables.as_slice()),
        ("positive", results.positives.as_slice()),
    ];
    sources
        .into_iter()
        .flat_map(|(kind, findings)| {
            findings.iter().map(move |finding| AccessibilityAssessment {
                kind: kind.to_string(),
                rule_id: finding
                    .rule_id
                    .clone()
                    .unwrap_or_else(|| finding.rule.clone()),
                wcag_criterion: wcag_criterion_of(&finding.rule),
                severity: finding.severity,
                message: finding.message.clone(),
                fix_suggestion: finding.fix_suggestion.clone(),
                selector: finding.selector.clone(),
                viewport: finding
                    .tags
                    .iter()
                    .find(|tag| {
                        matches!(
                            tag.as_str(),
                            "desktop-only" | "mobile-only" | "both-viewports"
                        )
                    })
                    .cloned(),
                evidence: finding.evidence.clone(),
            })
        })
        .collect()
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
    /// Stable identifier for `complexity_reason`'s sentence shape (for localized
    /// re-derivation by [`complexity_text`], #406).
    #[serde(default)]
    pub complexity_kind: ComplexityKind,
    /// Expected effect of fixing this finding group.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub expected_impact: String,
    /// Stable identifier for `expected_impact`'s sentence shape (for localized
    /// re-derivation by [`expected_impact_text`], #406).
    #[serde(default)]
    pub expected_impact_kind: ExpectedImpactKind,
    /// Cautious BFSG/EAA relevance classification.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub bfsg_relevance: String,
    /// Execution priority label, separate from severity/risk.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub remediation_priority: String,
    /// Einzelne Vorkommen
    pub occurrences: Vec<OccurrenceDetail>,
}

impl NormalizedFinding {
    /// Raises a legal flag: a WCAG Level-A finding of High or Critical
    /// severity. A convention rule (`display/*`) never does (#704) — it is
    /// only anchored to a criterion, it does not test it.
    pub fn is_legal_flag(&self) -> bool {
        self.wcag_level == "A"
            && matches!(self.severity, Severity::Critical | Severity::High)
            && !crate::taxonomy::is_convention_rule(&self.rule_id)
    }
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
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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
    /// Machine-readable provenance for this occurrence (DOM path, computed
    /// measurements like contrast ratio, …) — mirrors `Violation::evidence`.
    /// Canonical English, JSON-safe (#406); additive JSON field.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<crate::wcag::Evidence>,
    /// Cropped element screenshot (evidence-grade findings). In-memory only —
    /// never part of the JSON report or cache.
    #[serde(skip)]
    pub evidence_screenshot: Option<Vec<u8>>,
    /// Which viewport pass produced `evidence_screenshot`. In-memory only.
    #[serde(skip)]
    pub evidence_viewport: Option<&'static str>,
}

/// Score-Eintrag pro Modul
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleScoreEntry {
    pub name: String,
    pub score: u32,
    /// Letter grade — **only** for modules that carry weight in the overall
    /// score (plan 29, D2). A zero-weight indicator feeds nothing, so grading
    /// it on the same A–F scale as Accessibility gave it the same visual
    /// authority as a result that does count: "Dark Mode: F" sat next to an F
    /// for excluding screen-reader users. Those modules carry `band` instead.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grade: Option<String>,
    /// Canonical English qualitative band for `score` (`registry::FIVE_BAND`),
    /// present on every entry. This is the only qualitative label a zero-weight
    /// module gets. Canonical English per the localisation contract (#406); the
    /// PDF re-derives the localised label from `score`.
    pub band: String,
    pub weight_pct: u32,
    /// True when this module's score feeds directly into overall_score.
    /// False for supplemental dimensions (UX, Journey) that are displayed
    /// but not part of the core weighted average.
    pub contributes_to_overall: bool,
    /// Modules this one re-reads instead of measuring anything new (plan 29,
    /// D3). Empty for a module that measures its own subject. A reader who
    /// sees Source Quality agree with SEO and Accessibility is not looking at
    /// converging independent evidence — this field says so. Canonical module
    /// keys, lowercase.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub derived_from: Vec<String>,
    /// Whether this module uses direct measurement or heuristic inference.
    /// One of `"measured"`, `"composite"`, `"heuristic"`, `"optional"`,
    /// `"not_measured"`, `"c2pa_manifest"`, `"dns_query"`. See
    /// `output::report_model::ModuleTaxonomyClass` for the coarse
    /// classification derived from this value (#577).
    pub measurement_type: String,
}

impl ModuleScoreEntry {
    /// The weight decides the qualitative label (plan 29, D2): a module that
    /// carries weight in the overall score gets a letter grade, an indicator
    /// that feeds nothing gets the band word only. The weight is looked up
    /// from the single table rather than passed in, so the two can never
    /// disagree.
    pub(crate) fn new(
        name: &str,
        score: u32,
        measurement_type: &str,
        contributes_to_overall: bool,
    ) -> Self {
        let weight_pct = crate::taxonomy::module_weight(name);
        Self {
            name: name.to_string(),
            score,
            grade: (weight_pct > 0).then(|| {
                crate::registry::LETTER_GRADE
                    .label(score as f32, false)
                    .to_string()
            }),
            band: crate::registry::FIVE_BAND
                .label(score as f32, true)
                .to_string(),
            derived_from: crate::taxonomy::module_derived_from(name)
                .iter()
                .map(|m| m.to_string())
                .collect(),
            weight_pct,
            contributes_to_overall,
            measurement_type: measurement_type.to_string(),
        }
    }
}

/// Per-subcategory Accessibility score (plan/5-module-accessibility-
/// security-driver-detail.md): explains *why* the Accessibility score is
/// what it is at a finer grain than the single module number — e.g. a low
/// overall score driven specifically by broken ARIA/landmark structure
/// while forms and images are fine. `name` is the canonical English
/// `taxonomy::Subcategory` label (#406); `subcategory_kind` is the PDF-only
/// re-derivation key, not serialized.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubcategoryScoreEntry {
    pub name: String,
    #[serde(skip)]
    pub subcategory_kind: crate::taxonomy::Subcategory,
    pub score: u32,
}

/// Per-category Security score, same purpose as `SubcategoryScoreEntry`
/// but for `security::SecurityCategory` (Security has no per-check taxonomy
/// registry like WCAG's `Subcategory`, see that type's doc comment).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityCategoryScoreEntry {
    pub name: String,
    #[serde(skip)]
    pub category_kind: crate::security::SecurityCategory,
    pub score: u32,
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

/// Veto a misleadingly positive certificate when the audit does not pass.
///
/// Critical risk never passes. Beyond that, the certificate must follow the
/// default verdict (see `verdict.rs`): a legal-relevant WCAG Level-A violation
/// (`legal_flags`) or a blocking interactive issue (`blocking_issues`) fails the
/// audit regardless of the risk band — so a positive tier (e.g. "GUT" on a
/// medium-risk page) must be downgraded too, not just on High/Critical risk.
fn gate_certificate_by_risk(
    certificate: String,
    risk_level: &RiskLevel,
    legal_flags: usize,
    blocking_issues: usize,
) -> String {
    if matches!(risk_level, RiskLevel::Critical) {
        return "NICHT BESTANDEN".to_string();
    }
    let is_positive = matches!(
        certificate.as_str(),
        "AUSBAUFÄHIG" | "STABIL" | "GUT" | "SEHR GUT"
    );
    let does_not_pass =
        matches!(risk_level, RiskLevel::High) || legal_flags > 0 || blocking_issues > 0;
    if is_positive && does_not_pass {
        return "EINGESCHRÄNKT".to_string();
    }
    certificate
}

fn viewport_score_calculation_note() -> String {
    "accessibility_score is the 70% mobile / 30% desktop blend of the displayed \
     viewport accessibility scores. viewport_scores.weighted_overall blends the \
     viewport module scores before security; overall_score is the canonical final \
     score after the optional security blend."
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
    /// Raw accessibility scores displayed for the two viewport passes.
    pub desktop_accessibility: u32,
    pub mobile_accessibility: u32,
    /// Canonical accessibility score (mobile 70% + desktop 30%).
    pub viewport_blended_accessibility: u32,
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
                    evidence: v.evidence.clone(),
                    evidence_screenshot: v.evidence_screenshot.clone(),
                    evidence_viewport: v.evidence_viewport,
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
            // Prefer the violation's own axe/rule id over the taxonomy rule's
            // axe_id: a merged group can still contain violations from
            // several rule files sharing one taxonomy entry by design (see
            // `wcag_group_key`), and the taxonomy's axe_id is only ever
            // representative of ONE of them — reporting it regardless of
            // which check actually fired misattributes the finding for any
            // axe-parity consumer (SARIF, Studio) (#QA-011).
            let axe_id = first
                .rule_id
                .clone()
                .or_else(|| taxonomy_rule.and_then(|r| r.axe_id).map(String::from));
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
            let (complexity, complexity_kind) =
                derive_complexity(occurrence_count, &tax_id, issue_class_de);
            let complexity_reason = complexity_text(complexity_kind, true);
            let expected_impact_kind = derive_expected_impact(
                severity,
                occurrence_count,
                "wcag",
                first.level.to_string().as_str(),
            );
            let expected_impact = expected_impact_text(&expected_impact_kind, true);
            let bfsg_relevance = derive_bfsg_relevance(
                &tax_id,
                "wcag",
                &wcag_criterion_of(&first.rule),
                first.level.to_string().as_str(),
                severity,
            );
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
                wcag_criterion: wcag_criterion_of(&first.rule),
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
                complexity_kind,
                expected_impact,
                expected_impact_kind,
                bfsg_relevance,
                remediation_priority,
                occurrences,
            }
        })
        .collect();

    findings
}

/// One finding of a batch: a rule aggregated over every page it occurs on.
#[derive(Debug, Clone)]
pub struct BatchFinding {
    /// The rule's finding with batch-wide `occurrence_count`, `severity` and
    /// everything derived from them (priority, complexity, expected impact).
    pub finding: NormalizedFinding,
    /// Pages the rule occurs on, in batch order.
    pub urls: Vec<String>,
}

/// Aggregates the findings of a batch per rule, in `action_order` — the one
/// source for `top_actions` in the batch JSON and the top actions, finding
/// list and action plan in the batch PDF (plan 64).
///
/// Severity is the highest any page reported, as within a page (#288). The
/// count-dependent fields are re-derived from the batch-wide count, so the
/// expected impact no longer quotes one page's count next to the batch reach.
/// A WCAG rule and an SEO check reporting the same defect under one title
/// appear once, as the WCAG finding, as in the single report (plan 59).
pub fn aggregate_batch_findings(reports: &[NormalizedReport]) -> Vec<BatchFinding> {
    let mut aggregated: Vec<BatchFinding> = Vec::new();
    let mut index: HashMap<String, usize> = HashMap::new();
    for report in reports {
        for finding in &report.findings {
            match index.get(&finding.aggregation_key) {
                Some(&i) => {
                    let entry = &mut aggregated[i];
                    entry.finding.occurrence_count += finding.occurrence_count;
                    entry.finding.severity = entry.finding.severity.max(finding.severity);
                    if !entry.urls.contains(&report.url) {
                        entry.urls.push(report.url.clone());
                    }
                }
                None => {
                    index.insert(finding.aggregation_key.clone(), aggregated.len());
                    aggregated.push(BatchFinding {
                        finding: finding.clone(),
                        urls: vec![report.url.clone()],
                    });
                }
            }
        }
    }

    let wcag_titles: HashSet<String> = aggregated
        .iter()
        .filter(|b| b.finding.category == "wcag")
        .map(|b| b.finding.title.trim().to_lowercase())
        .collect();
    aggregated.retain(|b| {
        b.finding.category == "wcag"
            || !wcag_titles.contains(&b.finding.title.trim().to_lowercase())
    });

    for entry in &mut aggregated {
        rederive_count_dependent_fields(&mut entry.finding);
    }
    aggregated.sort_by(|a, b| crate::audit::prioritization::action_order(&a.finding, &b.finding));
    aggregated
}

/// Re-derives what `build_wcag_findings`/`aggregate_seo_findings` compute
/// from severity and occurrence count, after both changed in aggregation.
fn rederive_count_dependent_fields(f: &mut NormalizedFinding) {
    let count = f.occurrence_count;
    let issue_class_de = if f.category == "wcag" {
        f.issue_class_kind.label(false)
    } else {
        "issue"
    };
    f.priority_score = calculate_priority_score(f.severity, count, &f.rule_id);
    let (complexity, complexity_kind) = derive_complexity(count, &f.rule_id, issue_class_de);
    f.complexity_reason = complexity_text(complexity_kind, true);
    f.complexity = complexity;
    f.complexity_kind = complexity_kind;
    f.expected_impact_kind = derive_expected_impact(f.severity, count, &f.category, &f.wcag_level);
    f.expected_impact = expected_impact_text(&f.expected_impact_kind, true);
    f.remediation_priority = derive_remediation_priority(f.severity, count, &f.complexity);
    f.bfsg_relevance = derive_bfsg_relevance(
        &f.rule_id,
        &f.category,
        &f.wcag_criterion,
        &f.wcag_level,
        f.severity,
    );
}

pub(crate) fn wcag_group_key(violation: &crate::wcag::Violation) -> &str {
    // Prefer the violation's own axe/rule id as the group key whenever the
    // taxonomy has a dedicated entry for it. Several distinct checks share
    // one raw WCAG success criterion (e.g. many 4.1.2 checks: missing name,
    // invalid role, required-parent, required-attr, …); grouping by the raw
    // criterion alone collapsed them into a single merged finding that took
    // one arbitrary check's title/severity/fix guidance — including
    // escalating the whole group to the max severity of its worst member
    // (#QA-009). Falls back to the raw criterion id so checks that share a
    // taxonomy entry by design (e.g. several "missing accessible name"
    // checks under 4.1.2) still group together as intended. This also
    // subsumes what used to be a hardcoded 5-entry escape hatch for a few
    // rules that needed it — any axe id with a dedicated LEGACY_WCAG_MAP
    // entry gets the same treatment now.
    if let Some(axe_id) = violation.rule_id.as_deref() {
        if RuleLookup::by_legacy_wcag_id(axe_id).is_some() {
            return axe_id;
        }
    }
    violation.rule.as_str()
}

/// Per-`taxonomy::Subcategory` Accessibility score (plan/5-module-
/// accessibility-security-driver-detail.md): partitions the same raw
/// `Violation`s the overall Accessibility score is computed from by their
/// resolved subcategory (same `wcag_group_key` resolution the rest of
/// normalization uses, so this never disagrees with how findings are
/// grouped elsewhere), then re-runs the exact same
/// `AccessibilityScorer::calculate_score` on each subcategory's slice. This
/// deliberately reuses the real scorer rather than inventing a new
/// penalty formula — a subcategory's score means the same thing the module
/// score does, just scoped down. All 7 Accessibility subcategories are
/// always returned, in the taxonomy's declaration order; a subcategory
/// with zero violations scores 100 (the scorer's own empty-input result).
fn compute_accessibility_subcategory_scores(
    violations: &[crate::wcag::Violation],
) -> Vec<SubcategoryScoreEntry> {
    use crate::taxonomy::Subcategory;

    const ACCESSIBILITY_SUBCATEGORIES: [Subcategory; 7] = [
        Subcategory::ContentAlternatives,
        Subcategory::StructureSemantics,
        Subcategory::NavigationInteraction,
        Subcategory::FormsInteraction,
        Subcategory::LanguageClarity,
        Subcategory::TechnicalRobustness,
        Subcategory::VisualPresentation,
    ];

    let subcategory_for = |v: &crate::wcag::Violation| -> Subcategory {
        RuleLookup::by_legacy_wcag_id(wcag_group_key(v))
            .map(|r| r.subcategory)
            .unwrap_or_default()
    };

    ACCESSIBILITY_SUBCATEGORIES
        .into_iter()
        .map(|subcategory| {
            let subset: Vec<crate::wcag::Violation> = violations
                .iter()
                .filter(|v| subcategory_for(v) == subcategory)
                .cloned()
                .collect();
            let score = AccessibilityScorer::calculate_score(&subset).round() as u32;
            SubcategoryScoreEntry {
                name: subcategory.label(true).to_string(),
                subcategory_kind: subcategory,
                score,
            }
        })
        .collect()
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
        let (complexity, complexity_kind) = derive_complexity(occurrence_count, &rule_id, "issue");
        let complexity_reason = complexity_text(complexity_kind, true);
        let expected_impact_kind =
            derive_expected_impact(first.severity, occurrence_count, "seo", "");
        let expected_impact = expected_impact_text(&expected_impact_kind, true);
        let bfsg_relevance = derive_bfsg_relevance("", "seo", "", "", first.severity);
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
            complexity_kind,
            expected_impact,
            expected_impact_kind,
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
                    ..Default::default()
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
        .filter(|v| v.rule_id.as_deref() == Some("keyboard/hidden-focusable"))
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

/// Normalisiert einen rohen AuditReport.
///
/// - Gruppert Violations nach Regel-ID
/// - Reichert mit Taxonomie-Feldern an (via RuleLookup)
/// - Berechnet Grade/Certificate aus korrigiertem Score
pub fn normalize<'a>(report: &'a AuditReport) -> AuditContext<'a> {
    let violations = &report.accessibility.wcag_results.violations;

    let seo_reports_lang = report
        .discoverability
        .seo
        .as_ref()
        .is_some_and(|s| s.technical.has_lang);
    let had_311 = violations.iter().any(|v| v.rule == "3.1.1");

    let mut findings = build_wcag_findings(violations);
    if let Some(seo) = &report.discoverability.seo {
        findings.extend(aggregate_seo_findings(seo, MAX_OCCURRENCES));
    }

    findings.sort_by(crate::audit::prioritization::action_order);

    let mut interactive_findings = report.interactive_findings.clone();
    filter_aria_hidden_interactive(&mut interactive_findings, &findings, violations);

    let score = report
        .viewport_scores
        .as_ref()
        .map(ViewportScores::weighted_accessibility)
        .unwrap_or_else(|| report.accessibility.score.round().max(1.0) as u32);

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

    let module_scores =
        build_module_scores(report, score, &occurrence_counts, vuln_security_penalty);

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
                desktop_accessibility: vs.desktop.accessibility,
                mobile_accessibility: vs.mobile.accessibility,
                viewport_blended_accessibility: vs.weighted_accessibility(),
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

    // Grade and certificate describe the subject of the report: accessibility.
    //
    // They used to hang off `overall_score` (#233, to stop grade and
    // certificate contradicting each other). That fixed the internal
    // contradiction but moved the headline away from what the tool is for: an
    // accessibility score of 20 appeared on the cover as an overall 39,
    // because Performance 45, Mobile 80 and SEO 65 pulled it up. Both now read
    // the same number, so they still cannot contradict each other, and the
    // number is the one the report is about (plan 29, D1).
    //
    // The weighted `overall_score` remains in the report as a labelled
    // secondary value, with the weight basis it was computed over.
    let grade = AccessibilityScorer::calculate_grade(score as f32).to_string();
    let certificate = AccessibilityScorer::calculate_certificate(score as f32).to_string();

    let mut audit_flags = Vec::new();
    if report.accessibility.execution.quality.qualified_results {
        audit_flags.push(AuditFlag {
            kind: "incomplete_audit".to_string(),
            related_rule: None,
            source: "audit.execution".to_string(),
            message: format!(
                "Audit coverage is {:?}: {} rule checks failed and {} modules were partial or failed. Scores only describe the successfully measured scope.",
                report.accessibility.execution.quality.status,
                report.accessibility.execution.quality.failed_rule_checks,
                report.accessibility.execution.quality.partial_or_failed_modules,
            ),
        });
    }
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
            let violation_count: usize = report.accessibility.wcag_results.violations.len();
            if report.accessibility.nodes_analyzed > 0
                && violation_count > report.accessibility.nodes_analyzed
            {
                audit_flags.push(AuditFlag {
                    kind: "consent_wall_artifact".to_string(),
                    related_rule: None,
                    source: "browser.consent".to_string(),
                    message: format!(
                        "Possible consent wall artifact: {} violations with only {} analyzed nodes. \
                         Scores may be measuring the consent dialog rather than the actual page content. \
                         Recommendation: re-run the audit with --dismiss-consent.",
                        violation_count, report.accessibility.nodes_analyzed
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
    let certificate = gate_certificate_by_risk(
        certificate,
        &risk.level,
        risk.legal_flags,
        risk.blocking_issues,
    );

    let accessibility_subcategory_scores = compute_accessibility_subcategory_scores(violations);
    let security_category_scores = report
        .security
        .as_ref()
        .map(|s| {
            crate::security::calculate_security_category_scores(&s.issues)
                .into_iter()
                .map(|(category, score)| SecurityCategoryScoreEntry {
                    name: category.label(true).to_string(),
                    category_kind: category,
                    score,
                })
                .collect()
        })
        .unwrap_or_default();

    let normalized_data = NormalizedReport {
        url: report.url.clone(),
        wcag_level: report.wcag_level,
        timestamp: report.timestamp,
        duration_ms: report.duration_ms,
        nodes_analyzed: report.accessibility.nodes_analyzed,
        score,
        grade,
        certificate,
        findings,
        severity_counts,
        occurrence_counts,
        accessibility_assessments: normalize_assessments(&report.accessibility.wcag_results),
        rule_outcomes: report.accessibility.wcag_results.rule_outcomes.clone(),
        execution: report.accessibility.execution.clone(),
        module_scores,
        overall_score,
        accessibility_subcategory_scores,
        security_category_scores,
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
        raw_seo: report.discoverability.seo.as_ref(),
        raw_security: report.security.as_ref(),
        raw_html_conform: report.html_conform.as_ref(),
        raw_mobile: report.experience.mobile.as_ref(),
        raw_ux: report.ux.as_ref(),
        raw_journey: report.journey.as_ref(),
        raw_dark_mode: report.experience.dark_mode.as_ref(),
        raw_design_quality: report.experience.design_quality.as_ref(),
        raw_ai_transparency: report.experience.ai_transparency.as_ref(),
        raw_network_dns: report.experience.network_dns.as_ref(),
        raw_source_quality: report.discoverability.source_quality.as_ref(),
        raw_ai_visibility: report.discoverability.ai_visibility.as_ref(),
        raw_tech_stack: report.discoverability.tech_stack.as_ref(),
        raw_content_visibility: report.discoverability.content_visibility.as_ref(),
        raw_wcag: &report.accessibility.wcag_results,
        raw_patterns: report.patterns.as_ref(),
        raw_throttled_performance: &report.throttled_performance,
        raw_best_practices: report.best_practices.as_ref(),
        raw_commerce: report.commerce.as_ref(),
    };
    ctx.normalized.interpretation = Some(Interpretation::from_context(&ctx));
    ctx
}

/// The WCAG criterion a violation belongs to. A few page rules (e.g.
/// `aria-prohibited-attr`) carry their own slug in `Violation::rule` instead of
/// a criterion; the taxonomy maps those, so `wcag_criterion` never holds a
/// rule slug (plan 61).
pub(crate) fn wcag_criterion_of(rule: &str) -> String {
    crate::taxonomy::criterion_for_rule(rule).unwrap_or_else(|| rule.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wcag::{Violation, WcagResults};

    /// The page rules that emit their own slug as `Violation::rule` still land
    /// on their WCAG criterion (plan 61: dm.de showed
    /// `wcag_criterion: "aria-prohibited-attr"`).
    #[test]
    fn slug_page_rules_normalize_to_their_criterion() {
        for slug in ["aria-prohibited-attr", "aria-hidden-focus", "frame-tested"] {
            assert_eq!(wcag_criterion_of(slug), "4.1.2", "{slug}");
        }
    }

    /// #704: a display-convention rule is only anchored to 1.1.1; it must
    /// not raise a legal flag or count as highly BFSG-relevant, while a real
    /// 1.1.1 finding of the same severity still does.
    #[test]
    fn display_convention_rules_are_not_legal_flags() {
        let mut results = WcagResults::new();
        results.add_violation(
            Violation::new(
                "1.1.1",
                "Display modes: text layer reachable",
                WcagLevel::A,
                Severity::High,
                "hidden",
                "div#desc",
            )
            .with_rule_id("display/text-hidden")
            .with_tags(vec!["best-practice".into()]),
        );
        let report = AuditReport::new("https://example.com".into(), WcagLevel::AA, results, 1);
        let norm = normalize(&report).normalized;
        let finding = &norm.findings[0];
        assert_eq!(finding.rule_id, "a11y.display_text_layer.hidden");
        assert_eq!(finding.bfsg_relevance, "low");
        assert!(!finding.is_legal_flag());
        assert_eq!(norm.risk.legal_flags, 0);

        let mut results = WcagResults::new();
        results.add_violation(Violation::new(
            "1.1.1",
            "Non-text Content",
            WcagLevel::A,
            Severity::High,
            "Missing alt",
            "n1",
        ));
        let report = AuditReport::new("https://example.com".into(), WcagLevel::AA, results, 1);
        let norm = normalize(&report).normalized;
        assert_eq!(norm.findings[0].bfsg_relevance, "high");
        assert_eq!(norm.risk.legal_flags, 1);
    }

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
            kind: InteractiveFindingKind::FocusTrapEscaped,
            maps_to_finding: None,
            severity: Severity::Critical,
            journey: "modal".to_string(),
            before_snapshot_label: None,
            after_snapshot_label: None,
            message: "Modal has no focus trap.".to_string(),
            fix_suggestion: None,
            values: InteractiveFindingValues::default(),
            uncertainty: None,
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
            kind: InteractiveFindingKind::SkipLinkFocusNotMoved,
            maps_to_finding: None,
            severity: Severity::High,
            journey: "skip-link".to_string(),
            before_snapshot_label: None,
            after_snapshot_label: None,
            message: "Skip link is present but does not move focus to the target.".to_string(),
            fix_suggestion: None,
            values: InteractiveFindingValues::default(),
            uncertainty: None,
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
        report.discoverability.seo = Some(crate::seo::SeoAnalysis {
            technical: crate::seo::TechnicalSeo {
                has_lang: true,
                ..Default::default()
            },
            ..Default::default()
        });
        let norm_with_seo = normalize(&report);
        assert_eq!(norm_with_seo.normalized.findings.len(), 1);
        assert_eq!(
            norm_with_seo.normalized.score,
            report.accessibility.score.round() as u32
        );
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
        // A non-legal, non-blocking violation (Level AA, Low severity): keeps the
        // page out of the certificate veto (no legal_flags / blocking_issues), so
        // the ungated score→grade→certificate mapping is what we assert here.
        results.add_violation(Violation::new(
            "1.4.3",
            "Contrast",
            WcagLevel::AA,
            Severity::Low,
            "Low contrast",
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
            gate_certificate_by_risk("STABIL".to_string(), &RiskLevel::High, 0, 0),
            "EINGESCHRÄNKT"
        );
        assert_eq!(
            gate_certificate_by_risk("SEHR GUT".to_string(), &RiskLevel::Critical, 0, 0),
            "NICHT BESTANDEN"
        );
        // High risk also vetoes the bronze "AUSBAUFÄHIG" band: a failing page
        // must not surface a label that reads milder than a Critical fail.
        assert_eq!(
            gate_certificate_by_risk("AUSBAUFÄHIG".to_string(), &RiskLevel::High, 0, 0),
            "EINGESCHRÄNKT"
        );
        // "UNGENÜGEND" is already terminal and stays as-is under High risk.
        assert_eq!(
            gate_certificate_by_risk("UNGENÜGEND".to_string(), &RiskLevel::High, 0, 0),
            "UNGENÜGEND"
        );
    }

    #[test]
    fn legal_or_blocking_vetoes_positive_certificate_at_medium_risk() {
        // A legal-relevant WCAG Level-A violation fails the default verdict even
        // at medium risk — the certificate must not read "GUT" (regression: a
        // medium-risk page with legal_flags=1 showed a passing tier).
        assert_eq!(
            gate_certificate_by_risk("GUT".to_string(), &RiskLevel::Medium, 1, 0),
            "EINGESCHRÄNKT"
        );
        assert_eq!(
            gate_certificate_by_risk("STABIL".to_string(), &RiskLevel::Medium, 0, 3),
            "EINGESCHRÄNKT"
        );
        // A clean medium/low-risk page (no legal flags, no blockers) keeps its
        // positive tier — e.g. a "warn" verdict that still passes.
        assert_eq!(
            gate_certificate_by_risk("GUT".to_string(), &RiskLevel::Low, 0, 0),
            "GUT"
        );
        assert_eq!(
            gate_certificate_by_risk("GUT".to_string(), &RiskLevel::Medium, 0, 0),
            "GUT"
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
                duplicate_libraries: vec![],
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
            sourcemap_leaks: Default::default(),
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
                duplicate_libraries: vec![],
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
    fn test_unmeasured_performance_excluded_from_overall_score() {
        // QA-023: a performance module that ran but collected zero Core Web
        // Vitals must not drag the overall score down as if it scored 0 — it
        // should be excluded from the weighted sum (measurement_type
        // "not_measured"), so a clean a11y page keeps its overall score.
        use crate::performance::{PerformanceGrade, PerformanceScore, WebVitals};

        let report = AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            WcagResults::new(),
            100,
        )
        .with_performance(PerformanceResults {
            vitals: WebVitals::default(),
            score: PerformanceScore {
                overall: 0,
                grade: PerformanceGrade::Bronze,
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
        });

        let norm = normalize(&report).normalized;

        let perf_entry = norm
            .module_scores
            .iter()
            .find(|m| m.name == "Performance")
            .expect("performance module entry must be present");
        assert!(!perf_entry.contributes_to_overall);
        assert_eq!(perf_entry.measurement_type, "not_measured");

        // Overall score must match the accessibility-only score, not be
        // dragged down by an unmeasured 0-scored performance module.
        assert_eq!(norm.overall_score, norm.score);
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
                accessibility: 80,
                performance: None,
                overall: 80,
            },
            mobile: ViewportScoreSet {
                accessibility: 20,
                performance: None,
                overall: 20,
            },
            weighted_overall: 38,
        });
        let norm = normalize(&report);
        assert_eq!(norm.normalized.score, 38);
        assert_eq!(
            norm.normalized
                .module_scores
                .iter()
                .find(|module| module.name == "Accessibility")
                .map(|module| module.score),
            Some(38)
        );
        assert_eq!(
            norm.normalized.score_calculation_method,
            "viewport_weighted"
        );
        assert!(norm.normalized.score_breakdown.as_ref().is_some_and(|b| {
            b.calculation_note.contains("70% mobile") && b.viewport_blended_accessibility == 38
        }));
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
