//! Report ViewModel — structured, block-based presentation model
//!
//! Transforms raw audit data into a ViewModel where each block maps 1:1
//! to a report section. The renderer (pdf.rs) only calls add_component()
//! with data from the ViewModel — zero data transformation in the renderer.

use std::path::PathBuf;

use crate::cli::ReportLevel;
use crate::wcag::Severity;

/// Signal detail: category name mapped to a list of (check_label, passed, detail).
pub type SignalDetails = Vec<(String, Vec<(String, bool, String)>)>;

/// Configuration for PDF report generation
pub struct ReportConfig {
    pub level: ReportLevel,
    pub logo_path: Option<PathBuf>,
    pub locale: String,
    /// Opt-in regulatory PDF appendix section (see `--annex`). `None` by
    /// default — these sections are not part of the default report.
    pub annex: Option<crate::cli::AnnexKind>,
}

impl Default for ReportConfig {
    fn default() -> Self {
        Self {
            level: ReportLevel::Standard,
            logo_path: None,
            locale: "de".to_string(),
            annex: None,
        }
    }
}

// ─── Shared Enums ───────────────────────────────────────────────────────────

/// Priority, Effort and ExecutionPriority are domain prioritization concepts and
/// now live in `audit::prioritization`; re-exported here so existing
/// `report_model::{Priority, Effort, ExecutionPriority}` references keep working.
pub use crate::audit::prioritization::{Effort, ExecutionPriority, Priority};

/// Responsible role for a fix
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Role {
    Development,
    Editorial,
    DesignUx,
    ProjectManagement,
}

impl Role {
    pub fn label(&self, en: bool) -> &'static str {
        match self {
            Role::Development => {
                if en {
                    "Development"
                } else {
                    "Entwicklung"
                }
            }
            Role::Editorial => {
                if en {
                    "Content"
                } else {
                    "Redaktion"
                }
            }
            Role::DesignUx => "Design / UX",
            Role::ProjectManagement => {
                if en {
                    "Project management"
                } else {
                    "Projektleitung"
                }
            }
        }
    }
}

// ─── Evaluation Summary Types ────────────────────────────────────────────────

/// Snapshot summary of all findings — derived from AuditSummary for quick access in renderers.
pub struct FindingSummary {
    pub total: usize,
    pub critical: usize,
    pub high: usize,
    pub medium: usize,
    pub low: usize,
    pub verdict: String,
    pub dominant_issue_note: Option<String>,
    pub cross_impact_notes: Vec<String>,
    pub issue_pattern_label: String,
}

/// Summary of actionable tasks organized by semantic execution priority.
pub struct TaskSummary {
    pub blocker_count: usize,
    pub high_count: usize,
    pub medium_count: usize,
    pub low_count: usize,
    pub total_count: usize,
    /// Label of the role responsible for the most tasks
    pub primary_role: String,
}

/// Thematic cluster of related findings sharing a dimension or subcategory.
pub struct FindingCluster {
    pub label: String,
    pub dimension: String,
    pub finding_count: usize,
    pub occurrence_total: usize,
    pub severity_label: String,
    pub finding_titles: Vec<String>,
}

/// System-level diagnosis section: pattern analysis, clusters, systematic assessment.
pub struct DiagnosisBlock {
    pub section_title: String,
    pub pattern_label: String,
    pub pattern_description: String,
    pub is_systematic: bool,
    /// (dimension, finding_count, severity_label) per category
    pub category_breakdown: Vec<(String, usize, String)>,
    pub dominant_issue: Option<String>,
    pub verdict_intro: String,
    pub clusters: Vec<FindingCluster>,
}

// ─── Report ViewModel (Single Report) ───────────────────────────────────────

/// Complete ViewModel for a single audit report.
/// Each block maps 1:1 to a report section — the renderer does zero data transformation.
pub struct ReportViewModel {
    pub meta: MetaBlock,
    pub cover: CoverBlock,
    pub summary: SummaryBlock,
    pub executive: ExecutiveNarrativeBlock,
    pub methodology: MethodologyBlock,
    pub modules: ModulesBlock,
    pub severity: SeverityBlock,
    pub findings: FindingsBlock,
    pub diagnosis: DiagnosisBlock,
    pub module_details: ModuleDetailsBlock,
    pub actions: ActionsBlock,
    pub appendix: AppendixBlock,
    /// Recognized structural patterns (positive signals). Rendered as a
    /// green "what's working well" section in the PDF.
    pub positive_signals: Vec<PositiveSignal>,
    /// Management risk dimensions, as the analysis layer derived them. Carried
    /// as `ManagementRiskKind` rather than as finished text so the PDF renders
    /// them in the run language while the JSON keeps canonical English
    /// (#406, plan 35).
    pub management_risks: Vec<crate::audit::management_risk::ManagementRiskKind>,
}

pub struct PositiveSignal {
    pub title: String,
    pub description: String,
    /// True when all structural criteria for the pattern matched.
    pub strong: bool,
}

/// Report metadata for engine setup
pub struct MetaBlock {
    pub title: String,
    pub subtitle: String,
    pub date: String,
    pub version: String,
    pub author: String,
    pub report_level: ReportLevel,
    pub score_label: String,
}

/// Cover page data
pub struct CoverBlock {
    pub brand: String,
    pub title: String,
    pub domain: String,
    pub subtitle: String,
    pub date: String,
    pub score: u32,
    pub grade: String,
    pub certificate: String,
    /// 4-level maturity label for cover display
    pub maturity_label: String,
    pub total_issues: u32,
    pub critical_issues: u32,
    pub modules: Vec<String>,
    /// Desktop accessibility score from dual-viewport pass (if available)
    pub desktop_score: Option<u32>,
    /// Mobile accessibility score from dual-viewport pass (if available)
    pub mobile_score: Option<u32>,
}

/// Hero summary / Kurzfazit data
pub struct SummaryBlock {
    /// Accessibility-only score (corrected after suppressions).
    pub score: u32,
    /// Module-weighted overall score across all active modules.
    pub overall_score: u32,
    pub grade: String,
    pub certificate: String,
    /// 4-level maturity classification: "Kritisch" / "Instabil" / "Solide Basis" / "Stark"
    pub maturity_label: String,
    /// Problem distribution: "Strukturelle Defizite" / "Kritische Einzelprobleme" / "Feinschliff"
    pub problem_type: String,
    pub domain: String,
    pub date: String,
    pub executive_lead: String,
    /// When a single rule dominates ≥ 45 % of urgent findings: highlighted note for callout display.
    pub dominant_issue_note: Option<String>,
    /// Set (localized) when `AuditQualityStatus != Complete` -- surfaced as a
    /// callout right under the verdict on the management page, not only in
    /// the methodology appendix, so a downgraded/partial run can't be missed
    /// by a reader who never reaches the appendix.
    pub audit_quality_note: Option<String>,
    /// True only for `AuditQualityStatus::Insufficient` (failed rule checks —
    /// a genuine data-quality problem). False for `Partial` (a stability/
    /// retry budget was hit on individual measurements — a normal scope
    /// limitation of automated auditing, not a defect) — lets the PDF layer
    /// render the two at different severities instead of one alarming
    /// "audit incomplete" warning for both (feedback: "Partial" read as if
    /// the tool itself had failed).
    pub audit_quality_severe: bool,
    pub verdict: String,
    pub score_note: Option<String>,
    pub metrics: Vec<MetricItem>,
    pub top_actions: Vec<String>,
    pub positive_aspects: Vec<String>,
    /// Overall impact assessment: [(label, value), ...] e.g. ("Nutzer", "eingeschränkt")
    pub overall_impact: Vec<(String, String)>,
    /// Cross-module technical insights: short bullet strings
    pub technical_overview: Vec<String>,
    /// Score-range benchmark context (e.g. "Besser als ~60% der geprüften Seiten")
    pub benchmark_context: String,
    /// Concrete current business impact (1 sentence for KV: "Die Seite wird schlechter gefunden…")
    pub business_consequence: String,
    /// Forward-looking consequence: what happens if nothing is fixed
    pub consequence: String,
    /// Risk level label (Gering / Mittel / Hoch / Kritisch)
    pub risk_level: String,
    /// Risk summary (one sentence)
    pub risk_summary: String,
}

/// Precomputed copy and structure for the executive PDF narrative.
pub struct ExecutiveNarrativeBlock {
    pub cover_eyebrow: String,
    pub cover_kicker: String,
    pub key_points: Vec<String>,
    pub next_steps_callout_body: String,
}

/// A single KPI metric for the hero summary
pub struct MetricItem {
    pub title: String,
    pub value: String,
    pub accent_color: Option<String>,
}

/// Methodology section
pub struct MethodologyBlock {
    pub scope: String,
    pub method: String,
    pub limitations: String,
    pub disclaimer: String,
    pub audit_facts: Vec<(String, String)>,
    pub confidence_summary: Vec<(String, String)>,
    pub capabilities: Vec<CapabilitySignal>,
}

/// Module scores for dashboard and comparison
pub struct ModulesBlock {
    pub dashboard: Vec<ModuleScore>,
    pub overall_score: Option<u32>,
    pub overall_interpretation: Option<String>,
    /// The canonical per-module score/weight breakdown that actually feeds
    /// the weighted `overall_score` (`audit::normalized::build_module_scores`
    /// verbatim) — distinct from `dashboard` above, whose cards are reshaped
    /// for narrative presentation (e.g. the "Search Experience" card blends
    /// SEO with heuristic AI-visibility/content-visibility signals into one
    /// composite score that is *not* the raw SEO number carrying SEO's real
    /// weight). Used by the score-driver table (plan/2-score-driver-
    /// breakdown.md), which must show each module's actual weight_pct.
    pub module_scores: Vec<crate::audit::normalized::ModuleScoreEntry>,
    /// Per-subcategory Accessibility score breakdown (plan/5-module-
    /// accessibility-security-driver-detail.md), verbatim from
    /// `NormalizedReport.accessibility_subcategory_scores`.
    pub accessibility_subcategory_scores: Vec<crate::audit::normalized::SubcategoryScoreEntry>,
    /// Per-category Security score breakdown, same purpose as above for
    /// Security. Empty when the Security module didn't run.
    pub security_category_scores: Vec<crate::audit::normalized::SecurityCategoryScoreEntry>,
}

/// A single module's score data
pub struct ModuleScore {
    pub name: String,
    pub score: u32,
    /// Free-form convention shared with `audit::normalized::ModuleScoreEntry`:
    /// `"measured"`, `"composite"`, `"heuristic"`, `"optional"`,
    /// `"not_measured"`, or a module-specific detection method
    /// (`"c2pa_manifest"`, `"dns_query"`). See [`ModuleTaxonomyClass`] for the
    /// coarse classification derived from this value.
    pub measurement_type: String,
    pub interpretation: String,
    pub card_context: String,
    pub score_context: String,
    pub key_lever: String,
    pub good_threshold: u32,
    pub warn_threshold: u32,
}

/// Coarse taxonomy for how a module's 0–100 score was derived (#577).
///
/// The report mixes genuinely different kinds of measurement in one score
/// grid — WCAG compliance findings, directly measured metrics, a
/// presentation-level composite, heuristic estimates, and optional/
/// non-normative product features. This taxonomy makes that distinction
/// explicit so render surfaces can qualify non-normative scores instead of
/// presenting them with the same visual weight as a compliance/measured one.
///
/// Deliberately a derivation over the existing `measurement_type` string
/// rather than a replacement of that field's type: `measurement_type` has
/// ~30 read/write sites across `audit::normalized`, the PDF builder, and
/// JSON tests. Widening its string convention with one new value
/// (`"optional"`) plus this pure classifier is additive; migrating every
/// call site to a typed field is a disproportionate refactor for this scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleTaxonomyClass {
    /// WCAG/legal conformance signal. Accessibility only.
    Compliance,
    /// Directly measured/counted signal (Performance, Security, Mobile, SEO,
    /// Best Practices — all currently stored as `measurement_type: "measured"`).
    Measured,
    /// Presentation-level blend of measured and heuristic sub-signals (the
    /// `search_experience` roll-up, which mixes technical SEO with UX/AI-
    /// visibility-style estimates — neither purely measured nor purely
    /// heuristic).
    Composite,
    /// Automated estimate, not a hard measurement (UX, Journey, AI
    /// Visibility, Source Quality, Content Visibility).
    Heuristic,
    /// Optional/non-normative product feature — its absence is a product
    /// choice, not a compliance deficiency (Dark Mode).
    Optional,
}

impl ModuleTaxonomyClass {
    /// Classifies a `measurement_type` string as stored on `ModuleScore` /
    /// `audit::normalized::ModuleScoreEntry`. `module_name` disambiguates
    /// Accessibility from the other `"measured"` modules, since
    /// `measurement_type` alone cannot distinguish Compliance from Measured.
    /// Any other detection-method value (`"not_measured"`, `"c2pa_manifest"`,
    /// `"dns_query"`, …) falls back to `Measured` — all are directly
    /// observed signals, not heuristic estimates.
    pub fn from_measurement_type(measurement_type: &str, module_name: &str) -> Self {
        match measurement_type {
            "heuristic" => ModuleTaxonomyClass::Heuristic,
            "optional" => ModuleTaxonomyClass::Optional,
            "composite" => ModuleTaxonomyClass::Composite,
            _ if module_name.eq_ignore_ascii_case("Accessibility") => {
                ModuleTaxonomyClass::Compliance
            }
            _ => ModuleTaxonomyClass::Measured,
        }
    }

    /// Whether this class should carry a non-normative name-suffix qualifier
    /// on render surfaces that otherwise give a module the same visual
    /// weight as a Compliance/Measured module (cover gauges, dashboard
    /// cards, technical modules overview).
    pub fn needs_suffix_qualifier(self) -> bool {
        matches!(
            self,
            ModuleTaxonomyClass::Heuristic | ModuleTaxonomyClass::Optional
        )
    }
}

#[cfg(test)]
mod taxonomy_tests {
    use super::ModuleTaxonomyClass;

    #[test]
    fn classifies_accessibility_as_compliance() {
        assert_eq!(
            ModuleTaxonomyClass::from_measurement_type("measured", "Accessibility"),
            ModuleTaxonomyClass::Compliance
        );
    }

    #[test]
    fn classifies_performance_as_measured() {
        assert_eq!(
            ModuleTaxonomyClass::from_measurement_type("measured", "Performance"),
            ModuleTaxonomyClass::Measured
        );
    }

    #[test]
    fn classifies_ux_as_heuristic() {
        assert_eq!(
            ModuleTaxonomyClass::from_measurement_type("heuristic", "UX"),
            ModuleTaxonomyClass::Heuristic
        );
    }

    #[test]
    fn classifies_dark_mode_as_optional() {
        assert_eq!(
            ModuleTaxonomyClass::from_measurement_type("optional", "Dark Mode"),
            ModuleTaxonomyClass::Optional
        );
    }

    #[test]
    fn classifies_search_experience_as_composite() {
        assert_eq!(
            ModuleTaxonomyClass::from_measurement_type("composite", "Search Experience"),
            ModuleTaxonomyClass::Composite
        );
    }

    #[test]
    fn only_heuristic_and_optional_need_a_suffix_qualifier() {
        assert!(ModuleTaxonomyClass::Heuristic.needs_suffix_qualifier());
        assert!(ModuleTaxonomyClass::Optional.needs_suffix_qualifier());
        assert!(!ModuleTaxonomyClass::Compliance.needs_suffix_qualifier());
        assert!(!ModuleTaxonomyClass::Measured.needs_suffix_qualifier());
        assert!(!ModuleTaxonomyClass::Composite.needs_suffix_qualifier());
    }
}

/// Pre-computed severity breakdown
pub struct SeverityBlock {
    pub critical: u32,
    pub high: u32,
    pub medium: u32,
    pub low: u32,
    pub total: u32,
    pub has_issues: bool,
    /// Number of findings classified as component/template issues (occurrence_count >= 10).
    pub component_issues: u32,
    /// Total occurrences attributed to component issues.
    pub component_occurrences: u32,
}

/// Findings grouped into a single severity tier (Critical / High / Medium / Low).
/// Only non-empty tiers are included in `FindingsBlock.by_severity`.
pub struct FindingSeverityTier {
    pub severity: Severity,
    pub label: String,
    pub findings: Vec<FindingGroup>,
    pub total_occurrences: usize,
}

/// Criticality tier — separates mandatory (BFSG-relevant) from optimization findings
/// so the report can show them in clearly distinct sections (#245).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CriticalityTier {
    /// Level 1 — must be fixed: WCAG A/AA violations with BFSG relevance.
    Mandatory,
    /// Level 2 — should/can be fixed: SEO, AI visibility, dark mode, UX heuristics, WCAG AAA.
    Optimization,
}

/// Decide the criticality tier of a finding for the BFSG-vs-Optimierung split (#245).
///
/// Mandatory tier covers WCAG Level A/AA violations (legal/BFSG risk). Everything
/// else (SEO, AI visibility, dark mode, WCAG AAA, …) is classified as Optimization.
pub fn classify_criticality_tier(category: &str, wcag_level: &str) -> CriticalityTier {
    if category == "wcag" && matches!(wcag_level, "A" | "AA") {
        CriticalityTier::Mandatory
    } else {
        CriticalityTier::Optimization
    }
}

/// Findings grouped by criticality tier (Mandatory / Optimization), each containing
/// the same findings further sub-grouped by severity for structured rendering.
pub struct FindingCriticalityGroup {
    pub tier: CriticalityTier,
    /// Localized tier label (e.g. "Pflicht — muss behoben werden").
    pub label: String,
    /// Short eyebrow label (e.g. "EBENE 1 · PFLICHT").
    pub eyebrow: String,
    /// One-sentence explanation of what this tier contains.
    pub intro: String,
    pub by_severity: Vec<FindingSeverityTier>,
    pub total_findings: usize,
    pub total_occurrences: usize,
}

/// How much weight the automated run can actually carry for one statement.
///
/// The report used to know exactly one class — an automatically confirmed WCAG
/// violation — so every count sentence ("0 findings", "no violations in the
/// evaluated scope") silently excluded the heuristic signals that other
/// modules rendered a few pages later. Scores are computed from
/// `confirmed_violations` alone and are unchanged by this split; the classes
/// exist so the prose can name what it is actually counting.
#[derive(Debug, Clone, Copy, Default)]
pub struct EvidenceClasses {
    /// WCAG occurrences the engine confirmed against a success criterion.
    /// This is the number every accessibility score is derived from.
    pub confirmed_violations: u32,
    /// Heuristic accessibility signals: WCAG rule outcomes the engine flagged
    /// as warnings, plus the keyboard-journey and screen-reader modules'
    /// findings. Probably a barrier, not automatically provable against a
    /// success criterion — never scored.
    pub warnings: u32,
    /// Criteria the engine cannot decide by machine and explicitly hands to a
    /// human reviewer.
    pub manual_checks: u32,
}

impl EvidenceClasses {
    /// True when the run produced nothing at all — the only case in which the
    /// report may say that no accessibility findings were detected.
    pub fn is_empty(&self) -> bool {
        self.confirmed_violations == 0 && self.warnings == 0 && self.manual_checks == 0
    }
}

/// Grouped findings, already sorted by impact
pub struct FindingsBlock {
    pub summary: FindingSummary,
    /// Confirmed / warning / manual-check tally behind every count sentence.
    pub evidence: EvidenceClasses,
    pub clusters: Vec<FindingCluster>,
    pub top_findings: Vec<FindingGroup>,
    pub all_findings: Vec<FindingGroup>,
    /// Findings pre-partitioned into severity tiers (Critical → High → Medium → Low).
    /// Renderers can use this for structured, tier-first display without re-sorting.
    pub by_severity: Vec<FindingSeverityTier>,
    /// Findings grouped first by criticality tier (Mandatory / Optimization), then by
    /// severity. Renderers use this to enforce the visual+structural BFSG-vs-Optimierung
    /// split required by issue #245.
    pub by_tier: Vec<FindingCriticalityGroup>,
}

/// Module detail presentations (unchanged from before)
pub struct ModuleDetailsBlock {
    pub search_experience: Option<SearchExperiencePresentation>,
    pub performance: Option<PerformancePresentation>,
    pub seo: Option<SeoPresentation>,
    pub security: Option<SecurityPresentation>,
    pub html_conform: Option<HtmlConformPresentation>,
    pub commerce: Option<CommercePresentation>,
    pub mobile: Option<MobilePresentation>,
    pub ux: Option<UxPresentation>,
    pub journey: Option<JourneyPresentation>,
    pub dark_mode: Option<DarkModePresentation>,
    pub design_quality: Option<DesignQualityPresentation>,
    pub ai_transparency: Option<AiTransparencyPresentation>,
    pub network_dns: Option<crate::network::dns::NetworkDnsAnalysis>,
    pub source_quality: Option<crate::source_quality::SourceQualityAnalysis>,
    pub ai_visibility: Option<crate::ai_visibility::AiVisibilityAnalysis>,
    pub tech_stack: Option<crate::tech_stack::TechStackAnalysis>,
    pub content_visibility: Option<crate::content_visibility::ContentVisibilityAnalysis>,
    pub best_practices: Option<crate::best_practices::BestPracticesAnalysis>,
    pub patterns: Option<crate::patterns::PatternAnalysis>,
    pub has_any: bool,
}

/// Composite front-report score for findability, content clarity, trust and machine readability.
pub struct SearchExperiencePresentation {
    pub score: u32,
    pub label: String,
    pub interpretation: String,
    pub components: Vec<SearchExperienceComponent>,
    pub warnings: Vec<String>,
}

pub struct SearchExperienceComponent {
    pub label: String,
    pub score: u32,
    pub weight_pct: u32,
    pub explanation: String,
}

/// Dark mode analysis presentation block
pub struct DarkModePresentation {
    pub supported: bool,
    pub score: u32,
    pub detection_methods: Vec<String>,
    pub color_scheme_css: bool,
    pub meta_color_scheme: Option<String>,
    pub css_custom_properties: u32,
    pub dark_contrast_violations: u32,
    pub dark_only_violations: u32,
    pub light_only_violations: u32,
    pub print_stylesheet_detected: bool,
    pub print_interactive_chrome_hidden: bool,
    pub print_content_not_clipped: bool,
    pub print_clipped_elements: u32,
    pub forced_colors_detected: bool,
    pub forced_colors_active_matches: bool,
    pub forced_color_adjust_count: u32,
    pub forced_colors_focus_visible: bool,
    pub vision_deficiency_modes: Vec<VisionDeficiencyModePresentation>,
    /// (severity, description) pairs for issues
    pub issues: Vec<(String, String)>,
}

pub struct VisionDeficiencyModePresentation {
    pub mode: String,
    pub contrast_violations: u32,
    pub new_contrast_violations: u32,
    pub use_of_color_violations: u32,
}

/// Design-quality analysis presentation block (#528). Opt-in, score-neutral —
/// never contributes to accessibility/overall score, grade, or certificate.
pub struct DesignQualityPresentation {
    pub warning_count: usize,
    pub advisory_count: usize,
    pub findings: Vec<DesignQualityFindingPresentation>,
}

pub struct DesignQualityFindingPresentation {
    pub rule_id: String,
    pub level_label: String,
    pub confidence_label: String,
    pub selector: String,
    pub evidence: String,
    pub message: String,
}

/// AI-transparency (EU AI Act Art. 50) presentation block. Opt-in,
/// score-neutral — never contributes to accessibility/overall score, grade,
/// or certificate. Single-report only (see `PipelineConfig.check_ai_transparency`).
pub struct AiTransparencyPresentation {
    pub images_checked: usize,
    pub findings: Vec<ImageProvenanceFindingPresentation>,
}

pub struct ImageProvenanceFindingPresentation {
    pub image_url: String,
    pub provenance_label: String,
    pub validation_label: String,
    pub generator: Option<String>,
    pub message: String,
}

/// Action plan as pre-mapped roadmap columns
pub struct ActionsBlock {
    pub roadmap_columns: Vec<RoadmapColumnData>,
    pub role_assignments: Vec<RoleAssignment>,
    pub intro_text: String,
    /// Visual phase overview shown before the detailed roadmap
    pub phase_preview: Vec<PhasePreview>,
    /// Label for the entire action block, context-sensitive
    pub block_title: String,
    pub task_summary: TaskSummary,
}

pub struct RoadmapColumnData {
    pub title: String,
    pub description: String,
    pub accent_color: String,
    pub items: Vec<RoadmapItemData>,
}

pub struct RoadmapItemData {
    pub action: String,
    pub role: String,
    pub priority: String,
    pub execution_priority: String,
    pub effort: String,
    pub benefit: String,
    /// Business-oriented effect on users (e.g. "Screenreader-Nutzer können navigieren")
    pub user_effect: String,
    /// Risk reduction effect (e.g. "Reduziert WCAG-Verstoßrisiko")
    pub risk_effect: String,
    /// Occurrence count of the underlying `FindingGroup` (see `ActionItem`).
    pub occurrence_count: usize,
    /// Rule ID of the underlying `FindingGroup` (see `ActionItem`).
    pub rule_id: String,
    /// "Remediation leverage" tier ("Sehr hoch"/"Hoch"/"Niedrig", already
    /// localized) — a plain-language verdict combining reach
    /// (`occurrence_count`), cost (`effort`), and risk (`priority`) into one
    /// answer to "is this fix worth doing first" (plan/6-remediation-
    /// leverage-metric.md). Computed in `build_actions_block` where the raw
    /// `Effort`/`Priority` enums are still available.
    pub leverage: String,
}

/// A single phase in the visual phase overview (shown before the detailed roadmap)
pub struct PhasePreview {
    pub phase_label: String,
    pub accent_color: String,
    pub description: String,
    pub item_count: usize,
    pub top_items: Vec<String>,
}

/// Technical appendix
pub struct AppendixBlock {
    pub violations: Vec<AppendixViolation>,
    pub score_methodology: String,
    pub has_violations: bool,
}

// ─── Finding Types (shared between single and batch) ────────────────────────

/// Four-stage narrative arc: Diagnose → Ursache → Wirkung → Umsetzung.
/// Each field holds a complete, renderer-ready sentence — no raw labels.
pub struct NarrativeArc {
    /// What was observed — occurrence-enriched diagnosis sentence.
    pub diagnose: String,
    /// Why it happens — root cause with technical context.
    pub ursache: String,
    /// What it means — user and business impact combined.
    pub wirkung: String,
    /// How to fix it — actionable recommendation with effort context.
    pub umsetzung: String,
}

/// A grouped finding with customer-facing explanation
pub struct FindingGroup {
    pub title: String,
    pub rule_id: String,
    pub wcag_criterion: String,
    pub wcag_level: String,
    /// Official reference URL for the criterion (e.g. WCAG Understanding page).
    pub help_url: Option<String>,
    pub dimension: Option<String>,
    pub subcategory: Option<String>,
    pub issue_class: Option<String>,
    pub severity: Severity,
    pub priority: Priority,
    pub customer_description: String,
    pub user_impact: String,
    pub business_impact: String,
    pub typical_cause: String,
    pub recommendation: String,
    pub technical_note: String,
    pub confidence: String,
    pub false_positive_risk: String,
    pub verification: String,
    pub complexity: String,
    pub complexity_reason: String,
    /// Stable identifier for `complexity_reason`'s sentence shape (for
    /// localized re-derivation at render time, #406).
    pub complexity_kind: crate::audit::normalized::ComplexityKind,
    pub expected_impact: String,
    /// Stable identifier for `expected_impact`'s sentence shape (for
    /// localized re-derivation at render time, #406).
    pub expected_impact_kind: crate::audit::normalized::ExpectedImpactKind,
    pub bfsg_relevance: String,
    pub remediation_priority: String,
    pub occurrence_count: usize,
    pub affected_urls: Vec<String>,
    pub affected_elements: usize,
    pub additional_occurrences: usize,
    pub pattern_clusters: Vec<FindingPatternCluster>,
    pub location_hints: Vec<String>,
    pub representative_occurrences: Vec<RepresentativeOccurrence>,
    pub responsible_role: Role,
    pub effort: Effort,
    pub execution_priority: ExecutionPriority,
    pub examples: Vec<ExampleBlock>,
    /// Structural cause hint for findings with high occurrence counts (template/component pattern).
    pub structural_cause: Option<String>,
    /// True when this finding is classified as a component/template issue (occurrence_count >= 10).
    pub is_component_issue: bool,
    /// Criticality tier — Mandatory (BFSG-relevant) or Optimization (#245).
    pub criticality_tier: CriticalityTier,
    /// Precomputed narrative arc for story-flow rendering.
    pub narrative: NarrativeArc,
}

/// Occurrence-analysis result structs are produced by domain logic in
/// `audit::occurrence_analysis`; re-exported here so existing
/// `report_model::{RepresentativeOccurrence, FindingPatternCluster}` references
/// keep working.
pub use crate::audit::occurrence_analysis::{FindingPatternCluster, RepresentativeOccurrence};

pub struct CapabilitySignal {
    pub signal: String,
    pub source: String,
    pub confidence: String,
    pub surfaces: Vec<String>,
    pub note: String,
}

/// Code example showing bad vs. good pattern
pub struct ExampleBlock {
    pub bad: String,
    pub good: String,
    pub decorative: Option<String>,
}

/// Positive aspect of the audit
pub struct PositiveAspect {
    pub area: String,
    pub description: String,
}

// ─── Module Detail Presentations ────────────────────────────────────────────

/// Vitals for one viewport (desktop or mobile).
#[derive(Debug, Clone)]
pub struct PerformanceViewport {
    pub score: u32,
    pub grade: String,
    /// (metric_name, formatted_value, rating)
    pub vitals: Vec<(String, String, String)>,
}

/// One row in the throttled-performance table.
pub struct ThrottledPerfEntry {
    pub profile_name: String,
    pub lcp: String,
    pub tbt: String,
    pub cls: String,
    pub score: u32,
}

/// Per-origin row for third-party attribution display
pub struct ThirdPartyOriginRow {
    pub origin: String,
    pub provider: Option<String>,
    pub category: Option<String>,
    pub request_count: u32,
    pub transfer_kb: f64,
    pub resource_kinds: String,
}

/// Third-party attribution presentation
pub struct ThirdPartyPresentation {
    pub origins: Vec<ThirdPartyOriginRow>,
    pub total_origins: u32,
    pub total_kb: f64,
    pub total_requests: u32,
    pub is_significant: bool,
    /// Isolated per-origin main-thread impact rows (#531), empty unless
    /// `--isolate-third-party-impact` was set.
    pub isolated_impact: Vec<ThirdPartyImpactRow>,
}

/// One row in the isolated third-party impact table (#531) — numeric-only,
/// no message strings, so no i18n kind-enum machinery is needed here.
pub struct ThirdPartyImpactRow {
    pub origin: String,
    pub baseline_tbt_ms: f64,
    pub without_script_tbt_ms: f64,
    pub estimated_impact_ms: f64,
}

/// Critical request chain summary
pub struct CriticalChainPresentation {
    pub max_depth: usize,
    pub critical_path_ms: String,
    pub critical_path_kb: String,
    pub total_requests: usize,
}

/// Unminified assets summary
pub struct MinificationPresentation {
    pub total_count: usize,
    pub total_savings_kb: f64,
    pub top_assets: Vec<(String, String, String)>, // (url_truncated, kind, savings_kb_str)
    pub legacy_count: usize,
    pub legacy_wasted_kb: f64,
    pub legacy_assets: Vec<(String, String, String)>, // (url_truncated, signature, wasted_kb_str)
}

/// JS/CSS coverage (unused code) summary
pub struct CoveragePresentation {
    pub js_used_pct: Option<f64>,
    pub js_unused_kb: Option<f64>,
    pub css_used_pct: Option<f64>,
    pub css_total_rules: Option<u32>,
    pub css_used_rules: Option<u32>,
    /// Same file served under 2+ URLs (#551), empty unless found.
    pub duplicate_assets: Vec<DuplicateAssetRow>,
}

/// One duplicate-asset group (#551) for the compact PDF list.
pub struct DuplicateAssetRow {
    pub kind: String,
    pub kb: f64,
    pub urls: Vec<String>,
}

/// Non-composited animation findings
pub struct AnimationPresentation {
    pub total_count: usize,
    pub affected_properties: Vec<String>,
    pub findings: Vec<(String, String, String)>, // (kind, property, source_truncated)
}

/// Oversized image row for display
pub struct OversizedImageRow {
    pub src: String,
    pub natural: String,
    pub display: String,
}

/// Image efficiency section for SEO presentation
pub struct ImageEfficiencyPresentation {
    pub total_images: usize,
    pub modern_format_pct: f64,
    pub legacy_count: usize,
    pub oversized: Vec<OversizedImageRow>,
}

pub struct PerformancePresentation {
    pub score: u32,
    pub grade: String,
    pub interpretation: String,
    /// Flat vitals list (mobile, or blended if no split available)
    pub vitals: Vec<(String, String, String)>,
    /// Per-viewport breakdown — Some when both desktop and mobile were measured
    pub desktop: Option<PerformanceViewport>,
    pub mobile: Option<PerformanceViewport>,
    pub additional_metrics: Vec<(String, String)>,
    /// Resource/DOM metrics with an established best-practice threshold
    /// (load time, DOM content loaded, DOM node count) — (name, value,
    /// rating, target). Metrics without a defensible universal
    /// threshold (JS heap size, CO2e — which already carries its own letter
    /// rating) stay in `additional_metrics` instead.
    pub resource_ratings: Vec<(String, String, String, String)>,
    pub recommendations: Vec<String>,
    /// Render-blocking: (label, value) pairs for display
    pub render_blocking_metrics: Vec<(String, String)>,
    /// Render-blocking suggestions
    pub render_blocking_suggestions: Vec<String>,
    /// Whether render-blocking or heavy third-party load was detected
    pub has_render_blocking: bool,
    /// Throttled network performance profiles (empty if not measured)
    pub throttled_profiles: Vec<ThrottledPerfEntry>,
    /// CLS shift attribution (top 5, value + start_time + element)
    pub cls_attribution: Vec<(String, String, String)>,
    /// Third-party attribution — None if not collected
    pub third_party: Option<ThirdPartyPresentation>,
    /// Critical request chain summary
    pub critical_chain: Option<CriticalChainPresentation>,
    /// Unminified assets
    pub minification: Option<MinificationPresentation>,
    /// JS/CSS coverage
    pub coverage: Option<CoveragePresentation>,
    /// Non-composited animations
    pub animations: Option<AnimationPresentation>,
    /// Implausible or unmeasurable metrics detected during headless measurement (#291).
    pub measurement_warnings: Vec<String>,
}

pub struct SeoPresentation {
    pub score: u32,
    pub interpretation: String,
    pub meta_tags: Vec<(String, String)>,
    pub meta_issues: Vec<(String, Severity, String)>,
    pub heading_summary: String,
    pub social_summary: String,
    pub technical_summary: Vec<(String, String)>,
    pub tracking_summary: Vec<(String, String)>,
    pub tracking_summary_text: String,
    pub profile: Option<SeoProfilePresentation>,
    /// robots.txt audit — informational only
    pub robots: Option<RobotsPresentation>,
    /// Page health analysis presentation
    pub page_health: Option<PageHealthPresentation>,
    /// SERP pass presentation
    pub serp: Option<SerpPresentation>,
    /// Image efficiency analysis
    pub image_efficiency: Option<ImageEfficiencyPresentation>,
    /// TechnicalSeo issues (noindex, hreflang gaps, crawl budget) — (issue_type, message, severity_label)
    pub technical_issues: Vec<(String, String, String)>,
}

/// SERP pass presentation
pub struct SerpPresentation {
    pub score: u32,
    pub pass_count: u32,
    pub warning_count: u32,
    pub fail_count: u32,
    /// (category, label, status_label, detail)
    pub signals: Vec<(String, String, String, String)>,
    /// Rich-result-related schema types detected (e.g. "FAQ", "Breadcrumb").
    pub rich_result_types: Vec<String>,
}

/// Page health presentation block
pub struct PageHealthPresentation {
    /// (issue_type, message, severity)
    pub issues: Vec<(String, String, String)>,
    /// KV pairs: (label, value)
    pub url_info: Vec<(String, String)>,
    /// (check, count, severity, detail)
    pub html_issues: Vec<(String, u32, String, String)>,
    /// (status, detail)
    pub html_validator: Option<(String, String)>,
    /// (www_status_label, non_www_label, is_consolidated)
    pub www_status: Option<(String, String, bool)>,
    /// (status, is_soft_404)
    pub soft_404: Option<(u16, bool)>,
    pub has_any_issue: bool,
}

/// Pre-processed robots.txt display data
pub struct RobotsPresentation {
    pub error: Option<String>,
    pub has_wildcard_disallow_all: bool,
    pub blocks_ai_crawlers: bool,
    pub blocks_ai_citation: bool,
    pub inferred_policy: String,
    pub sitemaps: Vec<String>,
    pub crawl_delays: Vec<(String, u32)>,
    /// (user-agent, bot_class_label, allows_count, disallows_count, fully_blocked)
    pub bot_rows: Vec<(String, String, usize, usize, bool)>,
    /// AI crawlers that are explicitly blocked
    pub blocked_ai_bots: Vec<String>,
    /// Page has noindex and appears in sitemap.xml (true = problem found)
    pub noindex_in_sitemap: Option<bool>,
}

/// SEO Content Profile presentation data
pub struct SeoProfilePresentation {
    // Content Identity
    pub identity_summary: String,
    pub site_name: String,
    pub content_type: String,
    pub language: String,
    pub category_hints: Vec<String>,
    pub identity_facts: Vec<(String, String)>,
    // Page Classification
    pub page_type: String,
    pub page_attributes: Vec<String>,
    pub content_depth_score: u32,
    pub structural_richness_score: u32,
    pub media_text_balance_score: u32,
    pub intent_fit_score: u32,
    pub page_profile_summary: String,
    pub optimization_note: String,
    pub page_profile_facts: Vec<(String, String)>,
    // Schema Inventory: (type, completeness%, details)
    pub schema_rows: Vec<(String, String, String)>,
    pub schema_count: usize,
    // JSON-LD parser/normalization status, placed directly before the inventory.
    pub schema_status_summary: String,
    pub schema_status_has_errors: bool,
    pub schema_status_has_json_ld: bool,
    // (localized issue label, localized detail)
    pub schema_status_rows: Vec<(String, String)>,
    // Page-type fit derived from visible intent and URL evidence.
    pub schema_fit_summary: Option<String>,
    pub schema_fit_is_success: bool,
    pub schema_fit_is_warning: bool,
    pub schema_fit_facts: Vec<(String, String)>,
    // (feature, requirement status, missing/quality detail)
    pub schema_rule_rows: Vec<(String, String, String)>,
    // Manual checks are deliberately separated from missing field findings.
    pub schema_manual_review_rows: Vec<(String, String)>,
    // (schema/property, parity status, short visible-vs-schema evidence)
    pub schema_parity_rows: Vec<(String, String, String)>,
    // Signal Strength: (category, score%, rating_label)
    pub signal_rows: Vec<(String, String, String)>,
    pub signal_overall_pct: u32,
    // Signal Details: (category_name, [(check_label, passed, detail)])
    pub signal_details: SignalDetails,
    // Maturity
    pub maturity_level: String,
    pub maturity_description: String,
    pub maturity_techniques_used: u32,
    pub maturity_techniques_total: u32,
}

pub struct SecurityPresentation {
    pub score: u32,
    pub grade: String,
    pub interpretation: String,
    /// Band label for the score card. Derived from the same corrected band as
    /// `interpretation`, not from `score` — an open severe finding moves the
    /// wording without moving the number, and the card must not then contradict
    /// the takeaway right below it (plan 33).
    pub band_label: String,
    /// (header name, status, value, classification tier label) — the tier
    /// label distinguishes baseline hygiene from context-/architecture-
    /// dependent headers so a missing-header count doesn't read as uniformly
    /// urgent (#578).
    pub headers: Vec<(String, String, String, String)>,
    pub ssl_info: Vec<(String, String)>,
    pub issues: Vec<(String, Severity, String)>,
    pub recommendations: Vec<String>,
    /// (service name, kind label) pairs detected from response headers
    pub protection: Vec<(String, String)>,
    pub has_waf: bool,
    pub has_cdn: bool,
}

/// Commerce/shop presentation block. Derive-only, single-page-scoped signal
/// (product structured-data completeness + mandatory/trust-page link
/// presence on this one page) — deliberately NOT a checkout/payment/cart
/// audit (this tool has no cross-page session state; see
/// `crate::commerce::CommercePageKind`'s doc comment). The PDF renderer
/// states this scope explicitly so it can't be misread as broader coverage.
pub struct CommercePresentation {
    pub page_kind_label: String,
    pub product: Option<CommerceProductRow>,
    /// (localized page label, is linked from this page) for each of the 6
    /// mandatory/trust-page categories.
    pub trust_pages: Vec<(String, bool)>,
    /// (localized severity label, localized message) pairs.
    pub findings: Vec<(String, String)>,
}

pub struct CommerceProductRow {
    /// Share of the 5 expected product signals exposed (0-100).
    pub score: u32,
    pub price: Option<String>,
    pub availability: Option<String>,
    pub has_shipping_details: bool,
    pub has_return_policy: bool,
    pub rating: Option<String>,
}

/// HTML5 spec-conformance presentation block (html-conform crate).
///
/// `rule_id`/`message` stay opaque canonical-English payload (#406 passthrough
/// pattern, same as best_practices) — only `severity_label` is localized.
pub struct HtmlConformPresentation {
    pub score: u32,
    pub checked: bool,
    pub interpretation: String,
    pub error_count: u32,
    pub warning_count: u32,
    pub info_count: u32,
    /// How many distinct defects the counts above represent.
    pub distinct_defect_count: u32,
    /// (rule_id, localized severity label, message, location, occurrences)
    /// rows — one per *distinct* defect, not per occurrence, matching what
    /// the score is charged against.
    pub findings: Vec<(String, String, String, String, u32)>,
    /// Top distinct-defect messages (errors before warnings before info,
    /// then by occurrence count), capped like the other modules'
    /// recommendation lists (plan/31-html-conform-missing-fix-guidance.md).
    /// Empty when there are no findings.
    pub recommendations: Vec<String>,
}

pub struct MobilePresentation {
    pub score: u32,
    pub interpretation: String,
    pub viewport: Vec<(String, String)>,
    pub touch_targets: Vec<(String, String)>,
    pub font_analysis: Vec<(String, String)>,
    pub content_sizing: Vec<(String, String)>,
    pub issues: Vec<(String, Severity, String)>,
}

/// UX analysis presentation block
pub struct UxPresentation {
    pub score: u32,
    pub grade: String,
    pub interpretation: String,
    pub dimensions: Vec<UxDimensionPresentation>,
    pub issues: Vec<UxIssuePresentation>,
}

pub struct UxDimensionPresentation {
    pub kind: crate::ux::UxDimensionKind,
    pub name: String,
    pub score: u32,
    pub summary: String,
}

pub struct UxIssuePresentation {
    pub kind: crate::ux::UxIssueKind,
    pub dimension: String,
    pub severity: String,
    pub problem: String,
    pub impact: String,
    pub recommendation: String,
    pub values: crate::ux::UxIssueValues,
}

pub struct JourneyPresentation {
    pub score: u32,
    pub grade: String,
    pub page_intent: String,
    pub interpretation: String,
    pub dimensions: Vec<JourneyDimensionPresentation>,
    pub friction_points: Vec<FrictionPointPresentation>,
}

pub struct JourneyDimensionPresentation {
    pub kind: crate::journey::JourneyDimensionKind,
    pub name: String,
    pub score: u32,
    pub weight_pct: u32,
    pub summary: String,
}

pub struct FrictionPointPresentation {
    pub kind: crate::journey::FrictionKind,
    pub step: String,
    pub severity: String,
    pub problem: String,
    pub impact: String,
    pub recommendation: String,
    pub values: crate::journey::FrictionValues,
}

// ─── Shared Helper Types ────────────────────────────────────────────────────

#[derive(Clone)]
pub struct ActionItem {
    pub action: String,
    pub benefit: String,
    pub role: Role,
    pub priority: Priority,
    pub execution_priority: ExecutionPriority,
    pub effort: Effort,
    /// Whether the underlying finding is a systemic template/component issue
    /// (fix once, applies everywhere) vs. a local/individual occurrence. Drives
    /// the action plan's "by problem level" grouping.
    pub is_systemic: bool,
    /// Occurrence count of the underlying `FindingGroup` — used to order the
    /// action plan by real-world impact and to trace an action back to its
    /// root cause in the root-cause analysis section.
    pub occurrence_count: usize,
    /// Rule ID of the underlying `FindingGroup` — used to look up the
    /// matching root-cause letter (A, B, C…) assigned in the root-cause
    /// analysis section.
    pub rule_id: String,
}

pub struct RoleAssignment {
    pub role: Role,
    pub responsibilities: Vec<String>,
}

/// Aggregated violation: one entry per WCAG rule, with all affected elements
pub struct AppendixViolation {
    pub rule: String,
    pub rule_name: String,
    pub severity: Severity,
    pub message: String,
    pub fix_suggestion: Option<String>,
    pub affected_elements: Vec<AffectedElement>,
}

/// Single affected element within an aggregated violation
pub struct AffectedElement {
    pub selector: String,
    pub node_id: String,
}

// ─── Batch Report Presentation ──────────────────────────────────────────────

/// Cover data (used by batch reports)
pub struct CoverData {
    pub title: String,
    pub url: String,
    pub date: String,
    pub version: String,
}

/// One aggregated row per interactive category in the batch report.
pub struct InteractiveCategoryRow {
    pub category: String,
    pub affected_urls: usize,
    pub max_severity: Severity,
}

/// Aggregated summary of interactive journey findings across all audited pages.
pub struct InteractiveJourneySummary {
    /// Total pages that ran the interactive phase (had at least one trace or finding)
    pub total_pages_tested: usize,
    /// Number of pages with at least one interactive finding
    pub pages_with_issues: usize,
    /// Per-category aggregation, sorted by affected_urls descending
    pub categories: Vec<InteractiveCategoryRow>,
    /// True if any Critical interactive finding was found across all pages
    pub has_critical: bool,
}

/// Complete presentation model for a batch audit report
pub struct BatchPresentation {
    pub cover: CoverData,
    pub portfolio_summary: PortfolioSummary,
    pub top_issues: Vec<FindingGroup>,
    pub issue_frequency: Vec<IssueFrequency>,
    pub action_plan: ActionPlan,
    pub url_ranking: Vec<UrlSummary>,
    pub url_details: Vec<CompactUrlSummary>,
    pub url_matrix: Vec<UrlMatrixRow>,
    pub appendix: BatchAppendixData,
    pub interactive_summary: Option<InteractiveJourneySummary>,
    /// WCAG findings verified to share one template/component root cause
    /// across multiple pages (`audit::template_dedup`), with localized
    /// wording for the runtime locale.
    pub template_clusters: Vec<TemplateClusterView>,
}

/// Localized presentation view of a `audit::template_dedup::TemplateCluster`.
///
/// `confidence` is kept as the canonical `"confirmed"` / `"likely"` marker so
/// render code can key off it (e.g. to decide whether to override a
/// decision-action row); `headline` and `decision_label` carry the fully
/// localized sentences for the runtime locale — the strong "resolves N pages"
/// claim is only ever used for `"confirmed"` clusters, `"likely"` clusters
/// always read as unconfirmed/probable.
pub struct TemplateClusterView {
    pub rule_id: String,
    pub selector: String,
    pub confidence: String,
    pub affected_pages: usize,
    pub headline: String,
    pub decision_label: String,
}

pub struct ActionPlan {
    pub quick_wins: Vec<ActionItem>,
    pub medium_term: Vec<ActionItem>,
    pub structural: Vec<ActionItem>,
    pub role_assignments: Vec<RoleAssignment>,
}

/// A group of pages that share an identical SEO content value across the site.
///
/// `kind` is a canonical, language-neutral key (`"title"`, `"meta_description"`,
/// `"h1"`, or `"og_image"`); the PDF layer derives the localized label at
/// render time (#406). `value` is the verbatim shared content (truncated for
/// display).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DuplicateContentGroup {
    pub kind: String,
    pub value: String,
    pub urls: Vec<String>,
}

/// How often one SEO/social tag is missing across the audited set (#536) —
/// the systematic-gap counterpart to [`DuplicateContentGroup`]'s duplicate-
/// value detection. `kind` is a canonical, language-neutral key
/// (`"meta_description"`, `"canonical"`, `"og_image"`, or `"og_title"`); the
/// PDF layer derives the localized label at render time (#406).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MissingTagPrevalence {
    pub kind: String,
    pub missing_count: usize,
    pub total_count: usize,
}

/// A canonical-tag conflict found on a single page (#423).
///
/// `kind` is a canonical, language-neutral key (`"noindex_conflict"` or
/// `"og_url_mismatch"`); the PDF layer derives the localized label at render
/// time (#406). `detail` carries the offending values for context.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CanonicalIssue {
    pub kind: String,
    pub url: String,
    pub detail: String,
}

/// A non-reciprocal hreflang relationship (#423): `source_url` declares an
/// hreflang entry pointing to `target_url` (also in the audited set), but the
/// target does not declare a return link back to the source.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HreflangIssue {
    pub source_url: String,
    pub target_url: String,
    pub lang: String,
}

/// A JS/CSS asset URL that is served minified on some audited pages and
/// unminified on others (#537) — the cross-page consistency counterpart to
/// `performance::minification`'s existing per-page "flagged unminified"
/// finding. Points at a build-config inconsistency between templates/routes
/// (e.g. one content collection not going through the minifier) rather than
/// an isolated per-page issue. `kind` is a canonical, language-neutral key
/// (`"script"` or `"css"`); the PDF layer derives the localized label at
/// render time (#406).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MinificationInconsistency {
    pub url: String,
    pub kind: String,
    pub minified_on_count: usize,
    pub unminified_on_count: usize,
    pub total_pages_with_asset: usize,
}

/// A batch-audited URL whose own navigation involved a long (≥3 hop) or
/// cyclical HTTP redirect chain before reaching its final destination (#546).
/// Reuses the per-page redirect chain already tracked by
/// `seo::page_health::PageHealthAnalysis.redirect_chain` — this is purely a
/// cross-page re-aggregation of that existing signal, not a new detection
/// mechanism, and score-neutral (no scoring path reads this field).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RedirectChainIssue {
    /// The originally-requested (audited) URL.
    pub url: String,
    /// Number of redirect hops observed before the final response.
    pub hop_count: usize,
    /// True when the same URL appears more than once in `chain` — a real
    /// cyclical redirect, distinct from a long-but-terminating chain.
    pub is_loop: bool,
    /// URLs in the chain, in order (capped at 10 entries).
    pub chain: Vec<String>,
}

pub struct PortfolioSummary {
    pub total_urls: usize,
    pub passed: usize,
    pub failed: usize,
    pub average_score: f64,
    /// Weighted overall score across all active modules (averaged over URLs)
    pub average_overall_score: u32,
    pub total_violations: usize,
    pub duration_ms: u64,
    pub verdict_text: String,
    pub worst_urls: Vec<(String, f32)>,
    pub best_urls: Vec<(String, f32)>,
    pub severity_distribution: SeverityDistribution,
    /// Aggregated risk level across all URLs (worst-case)
    pub risk_level: String,
    /// Risk summary text
    pub risk_summary: String,
    /// Averaged module scores across all URLs (module_name, average_score)
    pub module_averages: Vec<(String, u32)>,
    /// List of active module names
    pub active_modules: Vec<String>,
    /// Domain name (extracted from first URL)
    pub domain: String,
    /// Certificate label based on the averaged overall score
    pub certificate: String,
    /// Grade based on the averaged overall score
    pub grade: String,
    pub page_type_distribution: Vec<(String, usize, u32)>,
    pub distribution_insights: Vec<String>,
    pub strongest_content_pages: Vec<(String, String, u32)>,
    pub weakest_content_pages: Vec<(String, String, u32)>,
    pub top_topics: Vec<(String, usize)>,
    pub overlap_pairs: Vec<(String, String, u32)>,
    /// Near-duplicate content pairs detected via SimHash (url_a, url_b, similarity_pct)
    pub near_duplicates: Vec<(String, String, u8)>,
    pub crawl_links: Option<CrawlLinkSummary>,
    /// Aggregated budget violations across all pages (metric, budget_label, #urls_violated, severity_label)
    pub budget_summary: Vec<(String, String, usize, String)>,
    /// Aggregated render-blocking summary across all pages (metric_label, value_label)
    pub render_blocking_summary: Vec<(String, String)>,
    /// Schema type distribution: (schema_type_label, url_count) sorted descending
    pub schema_distribution: Vec<(String, usize)>,
    /// Number of pages with no structured data at all
    pub pages_without_schema: usize,
    /// Cross-page duplicate content groups (identical title / meta description / H1 / og:image)
    pub duplicate_content: Vec<DuplicateContentGroup>,
    /// Cross-page tag-missing prevalence (meta description / canonical / og:image / og:title)
    pub missing_tag_prevalence: Vec<MissingTagPrevalence>,
    /// Per-page canonical-tag conflicts aggregated across the site
    pub canonical_issues: Vec<CanonicalIssue>,
    /// Non-reciprocal hreflang relationships between audited pages
    pub hreflang_issues: Vec<HreflangIssue>,
    /// Sitemap entries with HTTP/indexability issues
    pub sitemap_http_issues: Vec<crate::audit::SitemapHttpIssue>,
    /// Sitemap entries not linked by any audited page
    pub orphan_sitemap_urls: Vec<String>,
    /// Internal linked URLs that are absent from the sitemap
    pub linked_not_in_sitemap: Vec<String>,
    /// Sitemap entries blocked by a robots.txt Disallow rule (#549)
    pub robots_conflicts: Vec<crate::audit::RobotsSitemapConflict>,
    /// Asset URLs served minified on some pages and unminified on others (#537)
    pub minification_inconsistencies: Vec<MinificationInconsistency>,
    /// Audited URLs whose own navigation involved a long (≥3 hop) or
    /// cyclical redirect chain (#546)
    pub redirect_chain_issues: Vec<RedirectChainIssue>,
    /// Crawl-depth diagnostics (BFS click-distance from a heuristic start
    /// page through this batch's internal link graph, #548). `None` when
    /// no reasonable start-page candidate was audited.
    pub crawl_depth_diagnostics: Option<crate::audit::CrawlDepthDiagnostics>,
}

pub struct CrawlLinkSummary {
    pub seed_url: String,
    pub checked_internal_links: usize,
    pub broken_internal_links: Vec<BrokenLinkRow>,
    pub checked_external_links: usize,
    pub broken_external_links: Vec<BrokenLinkRow>,
    pub redirect_chains: Vec<RedirectChainRow>,
}

pub struct BrokenLinkRow {
    pub source_url: String,
    pub target_url: String,
    pub status: String,
    pub is_external: bool,
    pub severity: String,
    pub redirect_hops: u8,
}

pub struct RedirectChainRow {
    pub source_url: String,
    pub target_url: String,
    pub final_url: String,
    pub hops: u8,
    pub is_external: bool,
}

pub struct SeverityDistribution {
    pub critical: usize,
    pub high: usize,
    pub medium: usize,
    pub low: usize,
}

pub struct IssueFrequency {
    pub problem: String,
    pub wcag: String,
    pub occurrences: usize,
    pub affected_urls: usize,
    pub priority: Priority,
}

pub struct UrlSummary {
    pub url: String,
    pub score: f32,
    /// Weighted overall score across all active modules
    pub overall_score: u32,
    pub grade: String,
    pub critical_violations: usize,
    pub total_violations: usize,
    pub passed: bool,
    pub priority: Priority,
}

pub struct CompactUrlSummary {
    pub url: String,
    pub score: f32,
    pub grade: String,
    pub critical_violations: usize,
    pub total_violations: usize,
    pub page_type: Option<String>,
    pub page_attributes: Vec<String>,
    pub page_semantic_score: Option<u32>,
    pub biggest_lever: String,
    pub topic_terms: Vec<String>,
    pub top_issues: Vec<String>,
    pub module_scores: Vec<(String, u32)>,
}

/// One row of the URL matrix table (batch reports)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UrlMatrixRow {
    pub rank: usize,
    pub url: String,
    pub title: Option<String>,
    /// Pages within the batch that link to this URL
    pub inbound_links: usize,
    /// All outgoing links from this page (internal + external)
    pub outbound_links: u32,
    pub word_count: u32,
}

pub struct BatchAppendixData {
    pub per_url: Vec<UrlAppendix>,
}

pub struct UrlAppendix {
    pub url: String,
    pub violations: Vec<AppendixViolation>,
}
