//! Presentation model for PDF reports
//!
//! Transforms raw audit data into a structured, customer-facing report model
//! with grouped findings, explanations, and prioritized action plans.

use std::path::PathBuf;

use crate::cli::ReportLevel;
use crate::wcag::Severity;

/// Configuration for PDF report generation
pub struct ReportConfig {
    pub level: ReportLevel,
    pub company_name: Option<String>,
    pub logo_path: Option<PathBuf>,
}

impl Default for ReportConfig {
    fn default() -> Self {
        Self {
            level: ReportLevel::Standard,
            company_name: None,
            logo_path: None,
        }
    }
}

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
    /// Quick fix, low effort
    Quick,
    /// Requires some planning
    Medium,
    /// Structural change, significant effort
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

// ─── Single Report Presentation ─────────────────────────────────────────────

/// Complete presentation model for a single audit report
pub struct ReportPresentation {
    pub cover: CoverData,
    pub brief_verdict: BriefVerdict,
    pub methodology: MethodologySection,
    pub executive_summary: ExecutiveSummary,
    pub top_findings: Vec<FindingGroup>,
    pub score_breakdown: ScoreBreakdown,
    pub accessibility_details: Vec<FindingGroup>,
    pub module_details: ModuleDetails,
    pub action_plan: ActionPlan,
    pub positive_aspects: Vec<PositiveAspect>,
    pub appendix: AppendixData,
}

/// Cover page data
pub struct CoverData {
    pub title: String,
    pub url: String,
    pub date: String,
    pub version: String,
}

/// One-page brief verdict
pub struct BriefVerdict {
    pub score: f32,
    pub grade: String,
    pub verdict_text: String,
    pub critical_count: usize,
    pub total_violations: usize,
    pub top_actions: Vec<String>,
}

/// Methodology section explaining what was tested
pub struct MethodologySection {
    pub scope: String,
    pub method: String,
    pub limitations: String,
    pub disclaimer: String,
}

/// Executive summary for management
pub struct ExecutiveSummary {
    pub overall_assessment: String,
    pub key_risks: Vec<String>,
    pub positive_highlights: Vec<String>,
    pub priorities: Vec<String>,
}

/// A grouped finding with customer-facing explanation
pub struct FindingGroup {
    pub title: String,
    pub wcag_criterion: String,
    pub wcag_level: String,
    pub severity: Severity,
    pub priority: Priority,
    pub customer_description: String,
    pub user_impact: String,
    pub typical_cause: String,
    pub recommendation: String,
    pub technical_note: String,
    pub occurrence_count: usize,
    pub affected_urls: Vec<String>,
    pub affected_elements: usize,
    pub responsible_role: Role,
    pub effort: Effort,
    pub examples: Vec<ExampleBlock>,
}

/// Code example showing bad vs. good pattern
pub struct ExampleBlock {
    pub bad: String,
    pub good: String,
    pub decorative: Option<String>,
}

/// Score breakdown with interpretation per module
pub struct ScoreBreakdown {
    pub accessibility: ScoreDetail,
    pub performance: Option<ScoreDetail>,
    pub seo: Option<ScoreDetail>,
    pub security: Option<ScoreDetail>,
    pub mobile: Option<ScoreDetail>,
    pub overall: Option<ScoreDetail>,
}

pub struct ScoreDetail {
    pub score: u32,
    pub label: String,
    pub interpretation: String,
}

/// Details for non-accessibility modules
pub struct ModuleDetails {
    pub performance: Option<PerformancePresentation>,
    pub seo: Option<SeoPresentation>,
    pub security: Option<SecurityPresentation>,
    pub mobile: Option<MobilePresentation>,
}

pub struct PerformancePresentation {
    pub score: u32,
    pub grade: String,
    pub interpretation: String,
    pub vitals: Vec<(String, String, String)>, // (name, value, rating)
    pub additional_metrics: Vec<(String, String)>,
}

pub struct SeoPresentation {
    pub score: u32,
    pub interpretation: String,
    pub meta_tags: Vec<(String, String)>,
    pub meta_issues: Vec<(String, String, String)>, // (field, severity, message)
    pub heading_summary: String,
    pub social_summary: String,
    pub technical_summary: Vec<(String, String)>,
}

pub struct SecurityPresentation {
    pub score: u32,
    pub grade: String,
    pub interpretation: String,
    pub headers: Vec<(String, String, String)>, // (name, status, value)
    pub ssl_info: Vec<(String, String)>,
    pub issues: Vec<(String, String, String)>, // (title, severity, message)
    pub recommendations: Vec<String>,
}

pub struct MobilePresentation {
    pub score: u32,
    pub interpretation: String,
    pub viewport: Vec<(String, String)>,
    pub touch_targets: Vec<(String, String)>,
    pub font_analysis: Vec<(String, String)>,
    pub content_sizing: Vec<(String, String)>,
    pub issues: Vec<(String, String, String)>, // (category, severity, message)
}

/// Action plan with quick wins and structural measures
pub struct ActionPlan {
    pub quick_wins: Vec<ActionItem>,
    pub medium_term: Vec<ActionItem>,
    pub structural: Vec<ActionItem>,
    pub role_assignments: Vec<RoleAssignment>,
}

pub struct ActionItem {
    pub action: String,
    pub benefit: String,
    pub role: Role,
    pub priority: Priority,
}

pub struct RoleAssignment {
    pub role: Role,
    pub responsibilities: Vec<String>,
}

/// Positive aspect of the audit
pub struct PositiveAspect {
    pub area: String,
    pub description: String,
}

/// Technical appendix data
pub struct AppendixData {
    pub violations: Vec<AppendixViolation>,
    pub score_methodology: String,
}

pub struct AppendixViolation {
    pub rule: String,
    pub rule_name: String,
    pub severity: String,
    pub message: String,
    pub node_id: String,
    pub selector: Option<String>,
    pub fix_suggestion: Option<String>,
}

// ─── Batch Report Presentation ──────────────────────────────────────────────

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

/// Portfolio overview for batch reports
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
}

/// Distribution of violations by severity level (for charts)
pub struct SeverityDistribution {
    pub critical: usize,
    pub serious: usize,
    pub moderate: usize,
    pub minor: usize,
}

/// Frequency table for most common issues
pub struct IssueFrequency {
    pub problem: String,
    pub wcag: String,
    pub occurrences: usize,
    pub affected_urls: usize,
    pub priority: Priority,
}

/// URL ranking entry for overview table
pub struct UrlSummary {
    pub url: String,
    pub score: f32,
    pub grade: String,
    pub critical_violations: usize,
    pub total_violations: usize,
    pub passed: bool,
    pub priority: Priority,
}

/// Compact per-URL summary (not full detail)
pub struct CompactUrlSummary {
    pub url: String,
    pub score: f32,
    pub grade: String,
    pub top_issues: Vec<String>,
    pub module_scores: Vec<(String, u32)>,
}

/// Batch appendix with all raw details
pub struct BatchAppendixData {
    pub per_url: Vec<UrlAppendix>,
}

pub struct UrlAppendix {
    pub url: String,
    pub violations: Vec<AppendixViolation>,
}
