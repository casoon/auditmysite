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
}

impl Default for ReportConfig {
    fn default() -> Self {
        Self {
            level: ReportLevel::Standard,
            logo_path: None,
            locale: "de".to_string(),
        }
    }
}

// ─── Shared Enums ───────────────────────────────────────────────────────────

/// Priority level for findings and actions
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

impl Priority {
    pub fn label(&self) -> &'static str {
        match self {
            Priority::Critical => "Kritisch",
            Priority::High => "Hoch",
            Priority::Medium => "Mittel",
            Priority::Low => "Niedrig",
        }
    }
}

/// Responsible role for a fix
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Role {
    Development,
    Editorial,
    DesignUx,
    ProjectManagement,
}

impl Role {
    pub fn label(&self) -> &'static str {
        match self {
            Role::Development => "Entwicklung",
            Role::Editorial => "Redaktion",
            Role::DesignUx => "Design / UX",
            Role::ProjectManagement => "Projektleitung",
        }
    }
}

/// Effort estimate for a fix
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effort {
    Quick,
    Medium,
    Structural,
}

impl Effort {
    pub fn label(&self) -> &'static str {
        match self {
            Effort::Quick => "Quick Win",
            Effort::Medium => "Mittlerer Aufwand",
            Effort::Structural => "Strukturell",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ExecutionPriority {
    Optional,
    Important,
    Immediate,
}

impl ExecutionPriority {
    pub fn label(&self) -> &'static str {
        match self {
            ExecutionPriority::Immediate => "Sofort beheben",
            ExecutionPriority::Important => "Wichtig",
            ExecutionPriority::Optional => "Optional",
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
    /// Recognized structural patterns (positive signals).
    pub positive_signals: PositiveSignalsBlock,
}

/// Positive structural patterns recognized in the page.
/// Rendered as a green "what's working well" section in the PDF.
pub struct PositiveSignalsBlock {
    pub items: Vec<PositiveSignal>,
}

pub struct PositiveSignal {
    pub title: String,
    pub description: String,
    /// True when all structural criteria for the pattern matched.
    pub strong: bool,
}

impl PositiveSignalsBlock {
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
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
    pub status_title: String,
    pub risk_title: String,
    pub metrics_title: String,
    pub key_points_title: String,
    pub key_points: Vec<String>,
    pub impact_title: String,
    pub impact_rows: Vec<(String, String)>,
    pub quick_actions_title: String,
    pub quick_actions: Vec<String>,
    pub spotlight_eyebrow: String,
    pub spotlight_body: String,
    pub spotlight_impact: String,
    pub spotlight_recommendation: String,
    pub leverage_title: String,
    pub leverage_text: Option<String>,
    pub findings_title: String,
    pub findings_intro: String,
    pub action_plan_title: String,
    pub action_plan_intro: String,
    pub action_plan_callout_title: String,
    pub action_plan_callout_body: String,
    pub technical_title: String,
    pub technical_intro: String,
    pub next_steps_title: String,
    pub next_steps_intro: String,
    pub next_steps_callout_title: String,
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
}

/// A single module's score data
pub struct ModuleScore {
    pub name: String,
    pub score: u32,
    pub measurement_type: String,
    pub interpretation: String,
    pub card_context: String,
    pub score_context: String,
    pub key_lever: String,
    pub good_threshold: u32,
    pub warn_threshold: u32,
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

/// Grouped findings, already sorted by impact
pub struct FindingsBlock {
    pub summary: FindingSummary,
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
    pub performance: Option<PerformancePresentation>,
    pub seo: Option<SeoPresentation>,
    pub security: Option<SecurityPresentation>,
    pub mobile: Option<MobilePresentation>,
    pub ux: Option<UxPresentation>,
    pub journey: Option<JourneyPresentation>,
    pub dark_mode: Option<DarkModePresentation>,
    pub source_quality: Option<crate::source_quality::SourceQualityAnalysis>,
    pub ai_visibility: Option<crate::ai_visibility::AiVisibilityAnalysis>,
    pub tech_stack: Option<crate::tech_stack::TechStackAnalysis>,
    pub content_visibility: Option<crate::content_visibility::ContentVisibilityAnalysis>,
    pub best_practices: Option<crate::best_practices::BestPracticesAnalysis>,
    pub patterns: Option<crate::patterns::PatternAnalysis>,
    pub has_any: bool,
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
    /// (severity, description) pairs for issues
    pub issues: Vec<(String, String)>,
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
    /// Conversion/UX effect (e.g. "Verbessert Orientierung und Klickrate")
    pub conversion_effect: String,
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

pub struct RepresentativeOccurrence {
    pub selector: String,
    pub node_id: String,
    pub message: String,
    pub html_snippet: Option<String>,
    pub suggested_code: Option<String>,
}

pub struct FindingPatternCluster {
    pub label: String,
    pub occurrences: usize,
}

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
}

/// JS/CSS coverage (unused code) summary
pub struct CoveragePresentation {
    pub js_used_pct: Option<f64>,
    pub js_unused_kb: Option<f64>,
    pub css_used_pct: Option<f64>,
    pub css_total_rules: Option<u32>,
    pub css_used_rules: Option<u32>,
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
    /// Rich result types eligible (e.g. "FAQ", "Breadcrumb")
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
    pub headers: Vec<(String, String, String)>,
    pub ssl_info: Vec<(String, String)>,
    pub issues: Vec<(String, Severity, String)>,
    pub recommendations: Vec<String>,
    /// (service name, kind label) pairs detected from response headers
    pub protection: Vec<(String, String)>,
    pub has_waf: bool,
    pub has_cdn: bool,
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
    pub name: String,
    pub score: u32,
    pub summary: String,
}

pub struct UxIssuePresentation {
    pub dimension: String,
    pub severity: String,
    pub problem: String,
    pub impact: String,
    pub recommendation: String,
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
    pub name: String,
    pub score: u32,
    pub weight_pct: u32,
    pub summary: String,
}

pub struct FrictionPointPresentation {
    pub step: String,
    pub severity: String,
    pub problem: String,
    pub impact: String,
    pub recommendation: String,
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
}

pub struct ActionPlan {
    pub quick_wins: Vec<ActionItem>,
    pub medium_term: Vec<ActionItem>,
    pub structural: Vec<ActionItem>,
    pub role_assignments: Vec<RoleAssignment>,
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
    /// Certificate label based on the primary WCAG/accessibility score
    pub certificate: String,
    /// Grade based on the primary WCAG/accessibility score
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
