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
    pub company_name: Option<String>,
    pub logo_path: Option<PathBuf>,
    pub locale: String,
    pub history_preview: Option<ReportHistoryPreview>,
}

impl Default for ReportConfig {
    fn default() -> Self {
        Self {
            level: ReportLevel::Standard,
            company_name: None,
            logo_path: None,
            locale: "de".to_string(),
            history_preview: None,
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
            Effort::Medium => "Mittelfristig",
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

// ─── Report ViewModel (Single Report) ───────────────────────────────────────

/// Complete ViewModel for a single audit report.
/// Each block maps 1:1 to a report section — the renderer does zero data transformation.
pub struct ReportViewModel {
    pub meta: MetaBlock,
    pub cover: CoverBlock,
    pub summary: SummaryBlock,
    pub history: Option<HistoryTrendBlock>,
    pub methodology: MethodologyBlock,
    pub modules: ModulesBlock,
    pub severity: SeverityBlock,
    pub findings: FindingsBlock,
    pub module_details: ModuleDetailsBlock,
    pub actions: ActionsBlock,
    pub appendix: AppendixBlock,
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
    pub total_issues: u32,
    pub critical_issues: u32,
    pub modules: Vec<String>,
}

/// Hero summary / Kurzfazit data
pub struct SummaryBlock {
    pub score: u32,
    pub grade: String,
    pub certificate: String,
    pub domain: String,
    pub date: String,
    pub verdict: String,
    pub metrics: Vec<MetricItem>,
    pub top_actions: Vec<String>,
    pub positive_aspects: Vec<String>,
}

/// A single KPI metric for the hero summary
pub struct MetricItem {
    pub title: String,
    pub value: String,
    pub accent_color: Option<String>,
}

pub struct ReportHistoryPreview {
    pub previous_date: String,
    pub timeline_entries: usize,
    pub previous_accessibility_score: u32,
    pub previous_overall_score: u32,
    pub delta_accessibility: i32,
    pub delta_overall: i32,
    pub delta_total_issues: i32,
    pub delta_critical_issues: i32,
    pub recent_entries: Vec<(String, u32, u32, String, u32)>,
    pub new_findings: Vec<String>,
    pub resolved_findings: Vec<String>,
}

pub struct HistoryTrendBlock {
    pub previous_date: String,
    pub timeline_entries: usize,
    pub summary: String,
    pub metrics: Vec<(String, String)>,
    pub timeline_rows: Vec<(String, String, String, String, String)>,
    pub new_findings: Vec<String>,
    pub resolved_findings: Vec<String>,
}

/// Methodology section
pub struct MethodologyBlock {
    pub scope: String,
    pub method: String,
    pub limitations: String,
    pub disclaimer: String,
    pub audit_facts: Vec<(String, String)>,
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
    pub interpretation: String,
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
}

/// Grouped findings, already sorted by impact
pub struct FindingsBlock {
    pub top_findings: Vec<FindingGroup>,
    pub all_findings: Vec<FindingGroup>,
}

/// Module detail presentations (unchanged from before)
pub struct ModuleDetailsBlock {
    pub performance: Option<PerformancePresentation>,
    pub seo: Option<SeoPresentation>,
    pub security: Option<SecurityPresentation>,
    pub mobile: Option<MobilePresentation>,
    pub has_any: bool,
}

/// Action plan as pre-mapped roadmap columns
pub struct ActionsBlock {
    pub roadmap_columns: Vec<RoadmapColumnData>,
    pub role_assignments: Vec<RoleAssignment>,
    pub intro_text: String,
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
}

/// Technical appendix
pub struct AppendixBlock {
    pub violations: Vec<AppendixViolation>,
    pub score_methodology: String,
    pub has_violations: bool,
}

// ─── Finding Types (shared between single and batch) ────────────────────────

/// A grouped finding with customer-facing explanation
pub struct FindingGroup {
    pub title: String,
    pub rule_id: String,
    pub wcag_criterion: String,
    pub wcag_level: String,
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
    pub responsible_role: Role,
    pub effort: Effort,
    pub execution_priority: ExecutionPriority,
    pub examples: Vec<ExampleBlock>,
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

pub struct PerformancePresentation {
    pub score: u32,
    pub grade: String,
    pub interpretation: String,
    pub vitals: Vec<(String, String, String)>,
    pub additional_metrics: Vec<(String, String)>,
}

pub struct SeoPresentation {
    pub score: u32,
    pub interpretation: String,
    pub meta_tags: Vec<(String, String)>,
    pub meta_issues: Vec<(String, Severity, String)>,
    pub heading_summary: String,
    pub social_summary: String,
    pub technical_summary: Vec<(String, String)>,
    pub profile: Option<SeoProfilePresentation>,
}

/// SEO Content Profile presentation data
pub struct SeoProfilePresentation {
    // Content Identity
    pub identity_summary: String,
    pub site_name: String,
    pub content_type: String,
    pub language: String,
    pub category_hints: Vec<String>,
    // Page Classification
    pub page_type: String,
    pub page_attributes: Vec<String>,
    pub content_depth_score: u32,
    pub structural_richness_score: u32,
    pub media_text_balance_score: u32,
    pub intent_fit_score: u32,
    pub page_profile_summary: String,
    pub optimization_note: String,
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

// ─── Shared Helper Types ────────────────────────────────────────────────────

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

/// Complete presentation model for a batch audit report
pub struct BatchPresentation {
    pub cover: CoverData,
    pub portfolio_summary: PortfolioSummary,
    pub top_issues: Vec<FindingGroup>,
    pub issue_frequency: Vec<IssueFrequency>,
    pub action_plan: ActionPlan,
    pub url_ranking: Vec<UrlSummary>,
    pub url_details: Vec<CompactUrlSummary>,
    pub appendix: BatchAppendixData,
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
    pub total_violations: usize,
    pub duration_ms: u64,
    pub verdict_text: String,
    pub worst_urls: Vec<(String, f32)>,
    pub best_urls: Vec<(String, f32)>,
    pub severity_distribution: SeverityDistribution,
    pub page_type_distribution: Vec<(String, usize, u32)>,
    pub distribution_insights: Vec<String>,
    pub strongest_content_pages: Vec<(String, String, u32)>,
    pub weakest_content_pages: Vec<(String, String, u32)>,
    pub top_topics: Vec<(String, usize)>,
    pub overlap_pairs: Vec<(String, String, u32)>,
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

pub struct BatchAppendixData {
    pub per_url: Vec<UrlAppendix>,
}

pub struct UrlAppendix {
    pub url: String,
    pub violations: Vec<AppendixViolation>,
}
