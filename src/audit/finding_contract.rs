//! One finding contract for every audit domain (#780).
//!
//! Every problem a module collects reaches `NormalizedReport.findings` with
//! the same contract — source, outcome, applicability, cause and scoring
//! responsibility — or is recorded in `finding_inventory` with an explicit
//! exclusion reason. Nothing collected stays only in a module detail.
//!
//! Grouping, occurrence and scoring are separate on purpose:
//!
//! - **Grouping:** one finding per `(source, check)` on a page; occurrences
//!   are the collected issues of that check.
//! - **Cause:** `contract.cause` names the underlying defect. Several domains
//!   may report the same cause (a missing `lang` attribute is a WCAG failure,
//!   an SEO technical issue and an SEO meta issue). Each keeps its own
//!   finding, so the cause stays traceable in every domain.
//! - **Scoring:** `contract.scoring` names the one score that charges the
//!   finding. Every other finding with the same cause is a `reference`: shown,
//!   never charged again. `validate` rejects a cause charged twice in one
//!   score.
//!
//! The scoring functions themselves stay in their modules; this contract only
//! states, per check, which of them charges it. A test per domain pins that
//! statement against the module's scorer.

use serde::{Deserialize, Serialize};

#[cfg(test)]
use crate::audit::finding_derive::EvidenceQuality;
use crate::audit::finding_derive::{
    calculate_priority_score, derive_assurance, derive_bfsg_relevance, derive_complexity,
    derive_expected_impact, derive_remediation_priority, evidence_quality, AssuranceBasis,
    DetectionMethod, OccurrenceAnchor,
};
use crate::audit::normalized::{
    complexity_text, expected_impact_text, NormalizedFinding, OccurrenceDetail,
    ReportVisibilityData, ScoreImpactData,
};
use crate::audit::report::AuditReport;
use crate::taxonomy::{Dimension, IssueClass, Severity, Subcategory};

/// Audit domain a finding belongs to. One per registered module that
/// collects problems, plus the accessibility layers outside the catalog.
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum FindingDomain {
    #[default]
    Accessibility,
    AccessibilityJourney,
    ScreenReader,
    Seo,
    Security,
    Mobile,
    Performance,
    HtmlConformance,
    BestPractices,
    Ux,
    Journey,
    Commerce,
    Privacy,
}

impl FindingDomain {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Accessibility => "accessibility",
            Self::AccessibilityJourney => "accessibility_journey",
            Self::ScreenReader => "screen_reader",
            Self::Seo => "seo",
            Self::Security => "security",
            Self::Mobile => "mobile",
            Self::Performance => "performance",
            Self::HtmlConformance => "html_conformance",
            Self::BestPractices => "best_practices",
            Self::Ux => "ux",
            Self::Journey => "journey",
            Self::Commerce => "commerce",
            Self::Privacy => "privacy",
        }
    }

    /// The `category` string the JSON has carried since 1.x: `wcag` for
    /// accessibility rule findings, the domain name otherwise.
    pub fn category(self) -> &'static str {
        match self {
            Self::Accessibility => "wcag",
            other => other.as_str(),
        }
    }

    /// Taxonomy dimension and subcategory used by the presentation layer.
    /// Domains outside the five scored dimensions keep their own `dimension`
    /// label in the JSON; the internal kind only picks the closest group.
    fn taxonomy(self) -> (Dimension, Subcategory, &'static str) {
        match self {
            Self::Accessibility | Self::AccessibilityJourney | Self::ScreenReader => (
                Dimension::Accessibility,
                Subcategory::NavigationInteraction,
                "Accessibility",
            ),
            Self::Seo => (Dimension::Seo, Subcategory::Indexability, "SEO"),
            Self::Security => (
                Dimension::Security,
                Subcategory::ServerConfiguration,
                "Security",
            ),
            Self::Privacy => (
                Dimension::Security,
                Subcategory::ServerConfiguration,
                "Privacy",
            ),
            Self::Mobile => (Dimension::Mobile, Subcategory::ResponsiveLayout, "Mobile"),
            Self::Performance | Self::BestPractices => (
                Dimension::Performance,
                Subcategory::ResourceUsage,
                "Performance",
            ),
            Self::HtmlConformance => (
                Dimension::Seo,
                Subcategory::SemanticSignals,
                "HTML Conformance",
            ),
            Self::Ux => (
                Dimension::Accessibility,
                Subcategory::NavigationInteraction,
                "UX",
            ),
            Self::Journey => (
                Dimension::Accessibility,
                Subcategory::NavigationInteraction,
                "Journey",
            ),
            Self::Commerce => (Dimension::Seo, Subcategory::SemanticSignals, "Commerce"),
        }
    }
}

/// Whether the collected problem is a confirmed failure or needs a reader's
/// judgement before it counts as one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum FindingOutcome {
    #[default]
    Fail,
    /// Heuristic or context-dependent: recorded, never presented as a
    /// confirmed defect.
    Review,
}

/// Whether the check applies to this page. A check that does not apply never
/// becomes a finding; it is recorded in the inventory as excluded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Applicability {
    #[default]
    Applies,
    /// Whether the check applies depends on a page goal nobody declared
    /// (for example noindex without a declared indexability expectation).
    Unknown,
}

/// What a finding does to scores and decisions. Exactly one score owns a
/// cause; findings that repeat it elsewhere are references.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ScoringEffect {
    /// Charged in the owner's weighted module score.
    Deduction,
    /// A technical blocker: caps the owner's score and is a decision reason.
    Gate,
    /// Charged only in an unweighted indicator (`contributes_to_overall`
    /// false); never in the overall score or the verdict.
    Indicator,
    /// Feeds the risk assessment (risk level, legal flags, blocking issues),
    /// not a score.
    Risk,
    /// The same cause is charged by another finding; shown, never charged.
    Reference,
    /// Recorded; no score or decision reads it.
    #[default]
    None,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FindingScoring {
    /// Module score (`module_scores[].name`) or `risk` that is responsible.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    pub effect: ScoringEffect,
}

impl FindingScoring {
    fn new(owner: &str, effect: ScoringEffect) -> Self {
        Self {
            owner: Some(owner.to_string()),
            effect,
        }
    }

    fn none() -> Self {
        Self::default()
    }

    /// Whether this finding moves a number or a decision.
    pub fn charges(&self) -> bool {
        matches!(
            self.effect,
            ScoringEffect::Deduction | ScoringEffect::Gate | ScoringEffect::Indicator
        )
    }
}

/// Where the finding was observed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum LocationScope {
    /// Elements in the rendered DOM (occurrences carry selectors).
    #[default]
    Element,
    /// The page document as a whole (head, headers, metadata).
    Page,
    /// A resource the page loads (script, image, manifest).
    Resource,
    /// The origin or domain (DNS, robots.txt).
    Site,
    /// A state reached by interaction (journey, focus walk).
    Interaction,
}

/// The finding contract shared by every domain.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FindingContract {
    pub domain: FindingDomain,
    /// Collector that produced the finding, e.g. `seo.technical`.
    pub source: String,
    /// Stable check code within the source, e.g. `noindex`.
    pub check: String,
    pub outcome: FindingOutcome,
    pub applicability: Applicability,
    /// What the applicability rests on, e.g. `indexability:declared`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub applicability_basis: Option<String>,
    /// Canonical cause key. Findings of different domains with the same
    /// cause describe one defect.
    pub cause: String,
    pub scoring: FindingScoring,
    pub scope: LocationScope,
}

impl FindingContract {
    /// A primary finding is one the action and decision views list on its
    /// own. A reference repeats a cause charged elsewhere, and an indicator
    /// finding is derived from measurements that have their own findings;
    /// both stay in `findings` but are no separate action.
    pub fn is_primary(&self) -> bool {
        !matches!(
            self.scoring.effect,
            ScoringEffect::Reference | ScoringEffect::Indicator
        )
    }

    /// Contract of a WCAG rule finding: charged in the accessibility score.
    pub(crate) fn wcag(rule_id: &str, finding_id: &str) -> Self {
        Self {
            domain: FindingDomain::Accessibility,
            source: "wcag".to_string(),
            check: rule_id.to_string(),
            outcome: FindingOutcome::Fail,
            applicability: Applicability::Applies,
            applicability_basis: None,
            cause: shared_cause_for_wcag(rule_id)
                .unwrap_or(finding_id)
                .to_string(),
            scoring: FindingScoring::new("Accessibility", ScoringEffect::Deduction),
            scope: LocationScope::Element,
        }
    }
}

/// Causes that more than one domain reports. Keys are stable; the first
/// entry of each list owns the score.
const LANG_MISSING: &str = "page.lang.missing";
const CANONICAL_MISSING: &str = "page.canonical.missing";
const VIEWPORT_MISSING: &str = "page.viewport.missing";
const NOT_HTTPS: &str = "page.transport.not_https";
/// No `<h1>` in the document. SEO charges it; the UX and Journey indicators and
/// the accessibility journey report the same absence from their own angle and
/// stay references to it (#779). It is no WCAG failure (best practice), so
/// Accessibility does not charge it either.
const H1_MISSING: &str = "page.h1.missing";
/// Zoom locked by the viewport meta (`user-scalable=no`). WCAG 1.4.4 charges it
/// in Accessibility; the Mobile finding of the same observation is a reference
/// (#921).
const ZOOM_LOCKED: &str = "page.zoom.locked";
/// Interactive elements below the minimum target size. WCAG 2.5.8 charges it in
/// Accessibility; Mobile lists its own 44 px measurement as a reference (#921).
const TARGET_SIZE_SMALL: &str = "page.target_size.small";

fn shared_cause_for_wcag(rule_id: &str) -> Option<&'static str> {
    match rule_id {
        "document/lang-missing" => Some(LANG_MISSING),
        "zoom/viewport-locked" => Some(ZOOM_LOCKED),
        "targets/size" => Some(TARGET_SIZE_SMALL),
        _ => None,
    }
}

/// Indexability the page is meant to have — context for `noindex`, not a
/// measurement. Unknown unless declared or implied by the run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct IndexabilityExpectation {
    /// `Some(true)`: the page should be indexed; `Some(false)`: excluding it
    /// is intended; `None`: nobody said.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected: Option<bool>,
    pub source: ExpectationSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ExpectationSource {
    /// `--expect-indexable` or `[audit] expect_indexable`.
    Declared,
    /// The URL came from a sitemap, which lists URLs meant to be indexed.
    Sitemap,
    #[default]
    Undeclared,
}

impl IndexabilityExpectation {
    pub fn declared(expected: bool) -> Self {
        Self {
            expected: Some(expected),
            source: ExpectationSource::Declared,
        }
    }

    pub fn sitemap() -> Self {
        Self {
            expected: Some(true),
            source: ExpectationSource::Sitemap,
        }
    }

    /// A declaration wins over what the run implies.
    pub fn or(self, implied: Self) -> Self {
        if self.source == ExpectationSource::Declared {
            self
        } else {
            implied
        }
    }

    fn basis(self) -> String {
        let source = match self.source {
            ExpectationSource::Declared => "declared",
            ExpectationSource::Sitemap => "sitemap",
            ExpectationSource::Undeclared => "undeclared",
        };
        format!("indexability:{source}")
    }
}

/// How every collected problem of a page was represented.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct InventoryEntry {
    pub domain: FindingDomain,
    pub source: String,
    pub check: String,
    /// Collected occurrences of this check.
    pub collected: usize,
    pub disposition: Disposition,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[non_exhaustive]
pub enum Disposition {
    /// Represented by the finding with this id in `findings`.
    Finding { finding_id: String },
    /// Deliberately not a finding.
    Excluded { reason: ExclusionReason },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ExclusionReason {
    /// The page goal makes the check inapplicable (intended noindex).
    NotApplicable,
    /// The same element is already a WCAG finding (aria-hidden focusables).
    DuplicateOfWcagFinding,
}

/// Domains of the audit catalog and how each one reaches the findings.
/// `every_catalog_module_has_a_finding_policy` fails for a module added
/// without an entry, so new collected data cannot bypass the contract.
pub const DOMAIN_POLICIES: &[(&str, DomainPolicy)] = &[
    (
        "performance",
        DomainPolicy::Findings(FindingDomain::Performance),
    ),
    ("seo", DomainPolicy::Findings(FindingDomain::Seo)),
    ("security", DomainPolicy::Findings(FindingDomain::Security)),
    ("mobile", DomainPolicy::Findings(FindingDomain::Mobile)),
    (
        "html_conform",
        DomainPolicy::Findings(FindingDomain::HtmlConformance),
    ),
    ("privacy", DomainPolicy::Findings(FindingDomain::Privacy)),
    (
        "tech_stack",
        DomainPolicy::SignalsOnly("detection only: read-only fingerprints, no problem model"),
    ),
    ("ux", DomainPolicy::Findings(FindingDomain::Ux)),
    (
        "best_practices",
        DomainPolicy::Findings(FindingDomain::BestPractices),
    ),
    ("journey", DomainPolicy::Findings(FindingDomain::Journey)),
    ("commerce", DomainPolicy::Findings(FindingDomain::Commerce)),
    (
        "source_quality",
        DomainPolicy::SignalsOnly("derived indicator: signals with fulfilment, no problem model"),
    ),
    (
        "ai_visibility",
        DomainPolicy::SignalsOnly("derived indicator: signals with fulfilment, no problem model"),
    ),
    (
        "content_visibility",
        DomainPolicy::SignalsOnly("derived indicator: assessment signals, no problem model"),
    ),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DomainPolicy {
    /// Collected problems become findings of this domain.
    Findings(FindingDomain),
    /// The module publishes signals (present / fulfilment), not problems;
    /// they stay in its indicator detail.
    SignalsOnly(&'static str),
}

/// One collected problem before grouping.
struct Collected {
    domain: FindingDomain,
    source: &'static str,
    check: String,
    severity: Severity,
    title: Option<String>,
    message: String,
    selector: Option<String>,
    outcome: FindingOutcome,
    applicability: Applicability,
    applicability_basis: Option<String>,
    cause: Option<&'static str>,
    scoring: FindingScoring,
    scope: LocationScope,
    /// The message as a function of the language (#898); `message` is its
    /// `en = true` rendering.
    text: Option<crate::audit::finding_text::MessageText>,
    /// Elements this one collected problem stands for (1 unless the module
    /// reports an aggregate such as "79 touch targets too small", #952-3).
    elements: usize,
}

impl Collected {
    fn new(
        domain: FindingDomain,
        source: &'static str,
        check: impl Into<String>,
        severity: Severity,
        message: impl Into<String>,
        scoring: FindingScoring,
        scope: LocationScope,
    ) -> Self {
        Self {
            domain,
            source,
            check: check.into(),
            severity,
            title: None,
            message: message.into(),
            selector: None,
            outcome: FindingOutcome::Fail,
            applicability: Applicability::Applies,
            applicability_basis: None,
            cause: None,
            scoring,
            scope,
            text: None,
            elements: 1,
        }
    }

    fn elements(mut self, elements: u32) -> Self {
        self.elements = (elements as usize).max(1);
        self
    }

    /// Bind the module's text function: the PDF re-derives the message in the
    /// run language from it.
    fn text(mut self, text: impl Fn(bool) -> String + Send + Sync + 'static) -> Self {
        self.text = Some(crate::audit::finding_text::MessageText::new(text));
        self
    }

    fn selector(mut self, selector: Option<String>) -> Self {
        self.selector = selector.filter(|s| !s.is_empty());
        self
    }

    fn review(mut self) -> Self {
        self.outcome = FindingOutcome::Review;
        self
    }

    fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }
}

/// Severity from the string scales some modules store.
fn severity_from(s: &str) -> Severity {
    match s.to_ascii_lowercase().as_str() {
        "critical" => Severity::Critical,
        "high" | "error" | "serious" => Severity::High,
        "medium" | "warning" | "moderate" => Severity::Medium,
        _ => Severity::Low,
    }
}

/// Stable snake_case code of a serde-serialized kind enum.
pub(crate) fn kind_code<T: Serialize>(kind: &T) -> String {
    let raw = match serde_json::to_value(kind) {
        Ok(serde_json::Value::String(s)) => s,
        Ok(serde_json::Value::Object(map)) => map.keys().next().cloned().unwrap_or_default(),
        _ => String::new(),
    };
    let mut out = String::with_capacity(raw.len() + 4);
    for (i, c) in raw.chars().enumerate() {
        if c.is_ascii_uppercase() {
            if i > 0 {
                out.push('_');
            }
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

/// Collect every problem of every non-WCAG domain in the report.
///
/// WCAG violations are grouped by `build_wcag_findings`; this covers the rest.
/// `interactive_findings` are the journey findings after the aria-hidden
/// filter, `removed_interactive` the count that filter dropped.
fn collect(
    report: &AuditReport,
    interactive: &[crate::audit::normalized::InteractiveFinding],
    removed_interactive: usize,
    screen_reader: Option<&crate::screen_reader::ScreenReaderSummary>,
    inventory: &mut Vec<InventoryEntry>,
) -> Vec<Collected> {
    use FindingDomain as D;
    use LocationScope as L;
    use ScoringEffect as E;
    let mut out = Vec::new();

    if let Some(seo) = &report.discoverability.seo {
        // Headings: charged in the SEO score (`calculate_seo_score`).
        for issue in &seo.headings.issues {
            let (title, _) = crate::audit::normalized::seo_heading_finding_text(
                &issue.issue_type,
                true,
                &issue.message,
            );
            let mut c = Collected::new(
                D::Seo,
                "seo.headings",
                issue.issue_type.clone(),
                issue.severity,
                issue.message.clone(),
                FindingScoring::new("SEO", E::Deduction),
                L::Element,
            )
            .title(title);
            let heading = issue.clone();
            c = c.text(move |en| heading.localized_message(en));
            if issue.issue_type == "missing_h1" {
                c.cause = Some(H1_MISSING);
            }
            out.push(c);
        }
        // Meta: lang and canonical are charged once, in the technical
        // checks, viewport by Mobile (plan 105) — here they are references.
        for issue in &seo.meta_issues {
            let (scoring, cause) = match issue.field.as_str() {
                "lang" => (FindingScoring::new("SEO", E::Reference), Some(LANG_MISSING)),
                "canonical" => (
                    FindingScoring::new("SEO", E::Reference),
                    Some(CANONICAL_MISSING),
                ),
                "viewport" => (
                    FindingScoring::new("Mobile", E::Reference),
                    Some(VIEWPORT_MISSING),
                ),
                _ => (FindingScoring::new("SEO", E::Deduction), None),
            };
            let mut c = Collected::new(
                D::Seo,
                "seo.meta",
                issue.field.clone(),
                issue.severity,
                issue.message.clone(),
                scoring,
                L::Page,
            );
            let meta = issue.clone();
            c = c.text(move |en| meta.localized_message(en));
            c.cause = cause;
            out.push(c);
        }
        // Technical: `calculate_seo_score` charges https, canonical and lang;
        // noindex is a gate when the page is meant to be indexed.
        let technical = std::sync::Arc::new(seo.technical.clone());
        for issue in &seo.technical.issues {
            let mut c = Collected::new(
                D::Seo,
                "seo.technical",
                issue.issue_type.clone(),
                issue.severity,
                issue.message.clone(),
                FindingScoring::none(),
                L::Page,
            );
            {
                let technical = technical.clone();
                let issue_type = issue.issue_type.clone();
                let fallback = issue.message.clone();
                let nth = seo
                    .technical
                    .issues
                    .iter()
                    .take_while(|other| !std::ptr::eq(*other, issue))
                    .filter(|other| other.issue_type == issue.issue_type)
                    .count();
                c = c.text(move |en| {
                    crate::seo::technical::collect_technical_issues(&technical, en)
                        .into_iter()
                        .filter(|other| other.issue_type == issue_type)
                        .nth(nth)
                        .map_or_else(|| fallback.clone(), |other| other.message)
                });
            }
            match issue.issue_type.as_str() {
                "no_https" => {
                    c.scoring = FindingScoring::new("SEO", E::Deduction);
                    c.cause = Some(NOT_HTTPS);
                    // Security charges the same cause (`missing_https`).
                    if report.security.is_some() {
                        c.scoring.effect = E::Reference;
                    }
                }
                "no_canonical" => {
                    c.scoring = FindingScoring::new("SEO", E::Deduction);
                    c.cause = Some(CANONICAL_MISSING);
                }
                "no_lang" => {
                    // Accessibility charges the same cause for screen readers;
                    // SEO charges its own consequence (search language), like
                    // a missing viewport in Mobile and Accessibility (F2).
                    c.cause = Some(LANG_MISSING);
                    c.scoring = FindingScoring::new("SEO", E::Deduction);
                }
                "noindex" => {
                    let expectation = seo.indexability.expectation;
                    c.applicability_basis = Some(expectation.basis());
                    match expectation.expected {
                        Some(true) => {
                            c.scoring = FindingScoring::new("SEO", E::Gate);
                            c.severity = Severity::Critical;
                        }
                        Some(false) => {
                            inventory.push(InventoryEntry {
                                domain: D::Seo,
                                source: "seo.technical".to_string(),
                                check: "noindex".to_string(),
                                collected: 1,
                                disposition: Disposition::Excluded {
                                    reason: ExclusionReason::NotApplicable,
                                },
                            });
                            continue;
                        }
                        None => {
                            c.applicability = Applicability::Unknown;
                            c.outcome = FindingOutcome::Review;
                        }
                    }
                }
                _ => {}
            }
            out.push(c);
        }
        for issue in &seo.structured_data.schema_issues {
            let severity = match issue.severity {
                crate::seo::schema::SchemaIssueSeverity::Required => Severity::Medium,
                crate::seo::schema::SchemaIssueSeverity::Recommended => Severity::Low,
            };
            let schema = issue.clone();
            out.push(
                Collected::new(
                    D::Seo,
                    "seo.schema",
                    issue.issue_type.clone(),
                    severity,
                    issue.message.clone(),
                    FindingScoring::none(),
                    L::Page,
                )
                .text(move |en| crate::seo::schema::schema_issue_text(&schema, en)),
            );
        }
        if let Some(health) = &seo.page_health {
            let analysis = std::sync::Arc::new(health.clone());
            for issue in &health.issues {
                let nth = health
                    .issues
                    .iter()
                    .take_while(|other| !std::ptr::eq(*other, issue))
                    .filter(|other| other.issue_type == issue.issue_type)
                    .count();
                let analysis = analysis.clone();
                let issue_type = issue.issue_type.clone();
                let fallback = issue.message.clone();
                out.push(
                    Collected::new(
                        D::Seo,
                        "seo.page_health",
                        issue.issue_type.clone(),
                        severity_from(&issue.severity),
                        issue.message.clone(),
                        FindingScoring::none(),
                        L::Resource,
                    )
                    .text(move |en| {
                        crate::seo::page_health::collect_issues(&analysis, en)
                            .into_iter()
                            .filter(|other| other.issue_type == issue_type)
                            .nth(nth)
                            .map_or_else(|| fallback.clone(), |other| other.message)
                    }),
                );
            }
        }
    }

    if let Some(security) = &report.security {
        for issue in &security.issues {
            let effect = if issue.score_deduction > 0 {
                E::Deduction
            } else {
                E::None
            };
            let mut c = Collected::new(
                D::Security,
                "security.headers",
                issue.issue_type.clone(),
                issue.severity,
                issue.message.clone(),
                FindingScoring::new("Security", effect),
                L::Page,
            );
            if issue.kind() == Some(crate::security::SecurityIssueKind::MissingHttps) {
                c.cause = Some(NOT_HTTPS);
            }
            let security_issue = issue.clone();
            c = c.text(move |en| security_issue.localized_message(en));
            out.push(c);
        }
    }

    if let Some(mobile) = &report.experience.mobile {
        // `mobile_penalty` charges every issue except the two WCAG causes:
        // zoom lock (1.4.4) and small targets (2.5.8) are charged once, in
        // Accessibility, and stay here as measured references (#921).
        for issue in &mobile.issues {
            let kind = issue.kind();
            let scoring = match kind {
                Some(
                    crate::mobile::MobileIssueKind::NotScalable
                    | crate::mobile::MobileIssueKind::SmallTargets,
                ) => FindingScoring::new("Accessibility", E::Reference),
                _ => FindingScoring::new("Mobile", E::Deduction),
            };
            let mut c = Collected::new(
                D::Mobile,
                "mobile",
                issue.issue_type.clone(),
                issue.severity,
                issue.message.clone(),
                scoring,
                L::Page,
            );
            match kind {
                Some(crate::mobile::MobileIssueKind::MissingViewport) => {
                    c.cause = Some(VIEWPORT_MISSING);
                }
                Some(crate::mobile::MobileIssueKind::NotScalable) => c.cause = Some(ZOOM_LOCKED),
                Some(crate::mobile::MobileIssueKind::SmallTargets) => {
                    c.cause = Some(TARGET_SIZE_SMALL);
                }
                _ => {}
            }
            if let Some(count) = issue.values.small_target_count {
                c = c.elements(count);
            }
            let mobile_issue = issue.clone();
            c = c.text(move |en| mobile_issue.localized_message(en));
            out.push(c);
        }
    }

    // Budgets are CI thresholds of the user, not a score input.
    for violation in &report.experience.budget_violations {
        let severity = match violation.severity {
            crate::audit::BudgetSeverity::Error => Severity::High,
            crate::audit::BudgetSeverity::Warning => Severity::Medium,
        };
        out.push(Collected::new(
            D::Performance,
            "performance.budget",
            violation.metric.clone(),
            severity,
            format!(
                "{} is {} (budget {})",
                violation.metric, violation.actual_label, violation.budget_label
            ),
            FindingScoring::none(),
            L::Page,
        ));
    }

    // An unchecked module was not measured: whatever it carries is no
    // observation, so it yields no finding (and is not scored either).
    if let Some(html) = report.html_conform.as_ref().filter(|html| html.checked) {
        // `score_findings` charges errors, warnings and infos.
        for finding in &html.findings {
            out.push(
                Collected::new(
                    D::HtmlConformance,
                    "html_conform",
                    finding.rule_id.clone(),
                    severity_from(&finding.severity),
                    finding.message.clone(),
                    FindingScoring::new("HTML Conformance", E::Deduction),
                    L::Page,
                )
                .text({
                    let (rule_id, message, detail) = (
                        finding.rule_id.clone(),
                        finding.message.clone(),
                        finding.detail.clone(),
                    );
                    move |en| {
                        crate::html_conform::message::message_text(
                            &rule_id,
                            &message,
                            detail.as_ref(),
                            en,
                        )
                    }
                })
                .selector(finding.location.clone()),
            );
        }
    }

    if let Some(bp) = &report.best_practices {
        // Vulnerable libraries are charged in Security (`normalize`), not in
        // the Best Practices indicator.
        for lib in &bp.vulnerable_libraries.vulnerable {
            out.push(Collected::new(
                D::BestPractices,
                "best_practices.vulnerable_libraries",
                lib.name.to_ascii_lowercase(),
                severity_from(&lib.severity),
                format!("{} {}: {}", lib.name, lib.version, lib.description),
                FindingScoring::new("Security", E::Deduction),
                L::Resource,
            ));
        }
        for error in &bp.console_errors.errors {
            out.push(Collected::new(
                D::BestPractices,
                "best_practices.console",
                "error",
                Severity::Low,
                error.message.clone(),
                FindingScoring::new("Best Practices", E::Indicator),
                L::Page,
            ));
        }
    }

    if let Some(ux) = &report.ux {
        for issue in &ux.issues {
            // UX issues are heuristic suspicions (signal contract `ux`,
            // #784): recorded for review, never confirmed defects.
            let mut c = Collected::new(
                D::Ux,
                "ux",
                kind_code(&issue.kind),
                severity_from(&issue.severity),
                issue.problem.clone(),
                FindingScoring::new("UX", E::Indicator),
                L::Page,
            )
            .text({
                let (kind, values) = (issue.kind, issue.values.clone());
                move |en| crate::ux::ux_issue_text(kind, &values, en).0
            })
            .review();
            if issue.kind == crate::ux::UxIssueKind::NoH1 {
                c.cause = Some(H1_MISSING);
            }
            out.push(c);
        }
    }

    if let Some(journey) = &report.journey {
        for point in &journey.friction_points {
            // Friction points are heuristic (signal contract `journey`, #784).
            let mut c = Collected::new(
                D::Journey,
                "journey",
                kind_code(&point.kind),
                severity_from(&point.severity),
                point.problem.clone(),
                FindingScoring::new("Journey", E::Indicator),
                L::Interaction,
            )
            .text({
                let (kind, values) = (point.kind, point.values.clone());
                move |en| crate::page_flow::journey_friction_text(kind, &values, en).0
            })
            .review();
            if point.kind == crate::page_flow::FrictionKind::NoH1 {
                c.cause = Some(H1_MISSING);
            }
            out.push(c);
        }
    }

    if let Some(commerce) = &report.commerce {
        for finding in &commerce.findings {
            let kind = finding.kind;
            out.push(
                Collected::new(
                    D::Commerce,
                    "commerce",
                    kind_code(&finding.kind),
                    finding.severity,
                    finding.message.clone(),
                    FindingScoring::none(),
                    L::Page,
                )
                .text(move |en| crate::commerce::commerce_finding_text(kind, en)),
            );
        }
    }

    // Privacy indicators are technical observations: score-neutral, and an
    // indication (it rests on an assumption) is a review item, not a defect.
    if let Some(privacy) = &report.experience.privacy {
        for finding in &privacy.findings {
            let kind = serde_json::to_value(finding.kind)
                .ok()
                .and_then(|v| v.as_str().map(str::to_string))
                .expect("PrivacyFindingKind serializes as a string");
            let mut c = Collected::new(
                D::Privacy,
                "privacy",
                kind,
                Severity::Low,
                finding.message.clone(),
                FindingScoring::none(),
                L::Site,
            );
            {
                let provider = finding
                    .provider
                    .as_ref()
                    .map(|p| p.name.clone())
                    .unwrap_or_default();
                let (kind, document) = (finding.kind, finding.document);
                c = c.text(move |en| {
                    crate::privacy::findings::finding_text(kind, &provider, document, en)
                });
            }
            if finding.confidence == crate::privacy::ConfidenceClass::Indication {
                c = c.review();
            }
            out.push(c);
        }
    }

    // Journey findings feed the risk assessment; one that confirms a WCAG
    // finding names that finding's rule as its cause.
    for finding in interactive {
        let mut c = Collected::new(
            D::AccessibilityJourney,
            "a11y_journey",
            kind_code(&finding.kind),
            finding.severity,
            finding.message.clone(),
            FindingScoring::new("risk", E::Risk),
            L::Interaction,
        );
        {
            let (kind, values) = (finding.kind, finding.values.clone());
            c = c.text(move |en| {
                crate::audit::interactive_finding::interactive_finding_text(kind, &values, en).0
            });
        }
        if finding.uncertainty.is_some() {
            c.outcome = FindingOutcome::Review;
        }
        if let Some(rule) = &finding.maps_to_finding {
            c.cause = Some(journey_cause(rule));
        }
        if finding.kind
            == crate::audit::interactive_finding::InteractiveFindingKind::HeadingMissingH1
        {
            c.cause = Some(H1_MISSING);
        }
        out.push(c);
    }
    if removed_interactive > 0 {
        inventory.push(InventoryEntry {
            domain: D::AccessibilityJourney,
            source: "a11y_journey".to_string(),
            check: "hidden_focusable_aria_hidden".to_string(),
            collected: removed_interactive,
            disposition: Disposition::Excluded {
                reason: ExclusionReason::DuplicateOfWcagFinding,
            },
        });
    }

    if let Some(sr) = screen_reader {
        let localized = std::sync::Arc::new(sr.localized_texts.clone());
        for issue in &sr.issues {
            let check = issue
                .wcag_criterion
                .as_deref()
                .map(|c| format!("wcag_{}", c.replace('.', "_")))
                .unwrap_or_else(|| "reading_sequence".to_string());
            let mut c = Collected::new(
                D::ScreenReader,
                "screen_reader",
                check,
                severity_from(&issue.severity),
                issue.message.clone(),
                FindingScoring::new("risk", E::Risk),
                L::Element,
            )
            .text({
                let (english, texts) = (issue.message.clone(), localized.clone());
                move |en| match texts.get(&english) {
                    Some(german) if !en => german.clone(),
                    _ => english.clone(),
                }
            })
            .selector(issue.affected_node_ids.first().cloned());
            if issue.review {
                c = c.review();
                c.scoring = FindingScoring::none();
            }
            out.push(c);
        }
    }

    out
}

/// The cause a journey finding confirms: the WCAG rule's register id, which
/// is also the cause key of that rule's finding.
fn journey_cause(rule: &str) -> &'static str {
    crate::taxonomy::RuleLookup::by_id(rule)
        .map(|r| r.id)
        .unwrap_or("a11y.unmapped")
}

/// Findings of every non-WCAG domain, grouped per `(source, check)`, plus the
/// inventory that maps each collected problem to its representation.
pub(crate) fn build_domain_findings(
    report: &AuditReport,
    interactive: &[crate::audit::normalized::InteractiveFinding],
    removed_interactive: usize,
    screen_reader: Option<&crate::screen_reader::ScreenReaderSummary>,
    max_occurrences: usize,
) -> (Vec<NormalizedFinding>, Vec<InventoryEntry>) {
    let mut inventory = Vec::new();
    let collected = collect(
        report,
        interactive,
        removed_interactive,
        screen_reader,
        &mut inventory,
    );

    // Group in collection order, keyed by source and check.
    let mut groups: Vec<Vec<Collected>> = Vec::new();
    let mut index: std::collections::HashMap<(&'static str, String), usize> =
        std::collections::HashMap::new();
    for c in collected {
        let key = (c.source, c.check.clone());
        match index.get(&key) {
            Some(&i) => groups[i].push(c),
            None => {
                index.insert(key, groups.len());
                groups.push(vec![c]);
            }
        }
    }

    let findings: Vec<NormalizedFinding> = groups
        .into_iter()
        .map(|group| group_finding(group, max_occurrences))
        .collect();
    for finding in &findings {
        inventory.push(InventoryEntry {
            domain: finding.contract.domain,
            source: finding.contract.source.clone(),
            check: finding.contract.check.clone(),
            collected: finding.occurrence_count,
            disposition: Disposition::Finding {
                finding_id: finding.id.clone(),
            },
        });
    }
    (findings, inventory)
}

/// Detection method of a grouped finding (#784): any review occurrence or
/// unknown applicability keeps the whole finding a review.
fn collected_method(group: &[Collected]) -> DetectionMethod {
    if group
        .iter()
        .any(|c| c.outcome == FindingOutcome::Review || c.applicability == Applicability::Unknown)
    {
        DetectionMethod::Review
    } else {
        DetectionMethod::Measured
    }
}

/// An element occurrence is located by its selector; a page, resource, site
/// or interaction occurrence without one is a document-wide observation.
fn collected_anchor(c: &Collected) -> OccurrenceAnchor {
    match (c.selector.is_some(), c.scope) {
        (true, _) => OccurrenceAnchor::Element,
        (false, LocationScope::Element) => OccurrenceAnchor::Missing,
        (false, _) => OccurrenceAnchor::Document,
    }
}

fn group_finding(group: Vec<Collected>, max_occurrences: usize) -> NormalizedFinding {
    let first = &group[0];
    let rule_id = format!("{}.{}", first.source, first.check);
    let severity = group
        .iter()
        .map(|c| c.severity)
        .max()
        .unwrap_or(first.severity);
    // Element count, not problem count: the scope ("Lokal · n"), complexity
    // and priority all read it (#952-3).
    let occurrence_count: usize = group.iter().map(|c| c.elements).sum();
    let (dimension_kind, subcategory_kind, dimension) = first.domain.taxonomy();
    let subcategory_kind = match first.source {
        "seo.headings" => Subcategory::ContentStructure,
        "seo.meta" => Subcategory::SnippetMetadata,
        "seo.schema" => Subcategory::SemanticSignals,
        _ => subcategory_kind,
    };
    let category = first.domain.category();
    let issue_class = if first.outcome == FindingOutcome::Review {
        IssueClass::Risk
    } else {
        IssueClass::Weak
    };
    let subcategory_label = subcategory_kind.label(true).to_string();
    let issue_class_label = "issue";
    let priority_score = calculate_priority_score(severity, occurrence_count, &rule_id);
    let assurance = AssuranceBasis {
        method: collected_method(&group),
        evidence: evidence_quality(group.iter().map(collected_anchor)),
    };
    let derived = derive_assurance(assurance.method, assurance.evidence);
    let (complexity, complexity_kind) =
        derive_complexity(occurrence_count, &rule_id, issue_class_label);
    let complexity_reason = complexity_text(complexity_kind, true);
    let expected_impact_kind = derive_expected_impact(severity, occurrence_count, category, "");
    let expected_impact = expected_impact_text(&expected_impact_kind, true);
    let bfsg_relevance = derive_bfsg_relevance("", category, "", "", severity);
    let remediation_priority = derive_remediation_priority(severity, occurrence_count, &complexity);

    let id = a11y_report::FindingId::new(&rule_id, std::iter::empty()).to_string();
    let (patterns, pattern_count) = if first.scope == LocationScope::Element {
        crate::audit::normalized::finding_patterns(
            &rule_id,
            group
                .iter()
                .map(|c| c.selector.as_deref().unwrap_or("document")),
        )
    } else {
        (Vec::new(), 0)
    };
    let gate = group
        .iter()
        .any(|c| c.scoring.effect == ScoringEffect::Gate);
    // Outside accessibility only what blocks or hurts most is shown in the
    // narrative report; the JSON always carries everything.
    let primary = !matches!(
        first.scoring.effect,
        ScoringEffect::Reference | ScoringEffect::Indicator
    );
    let visible = gate || (primary && severity >= Severity::High);
    let contract = FindingContract {
        domain: first.domain,
        source: first.source.to_string(),
        check: first.check.clone(),
        outcome: first.outcome,
        applicability: first.applicability,
        applicability_basis: first.applicability_basis.clone(),
        cause: first
            .cause
            .map(str::to_string)
            .unwrap_or_else(|| id.clone()),
        scoring: first.scoring.clone(),
        scope: first.scope,
    };
    let title = first
        .title
        .clone()
        .or_else(|| crate::audit::finding_titles::finding_title(&rule_id, true))
        .unwrap_or_else(|| check_title(&first.check));

    let help_url = crate::audit::rule_docs::finding_help_url(&rule_id);

    NormalizedFinding {
        known: false,
        id,
        patterns,
        pattern_count,
        category: category.to_string(),
        rule_id: rule_id.clone(),
        wcag_criterion: String::new(),
        axe_id: None,
        wcag_level: String::new(),
        dimension: dimension.to_string(),
        subcategory: subcategory_label,
        issue_class: issue_class_label.to_string(),
        dimension_kind,
        subcategory_kind,
        issue_class_kind: issue_class,
        severity,
        user_impact: first.message.clone(),
        technical_impact: first.message.clone(),
        score_impact: ScoreImpactData {
            base_penalty: 0.0,
            max_penalty: 0.0,
            scaling: "none".to_string(),
        },
        report_visibility: ReportVisibilityData {
            executive: visible,
            standard: visible,
            technical: visible,
        },
        aggregation_key: rule_id,
        title,
        description: first.message.clone(),
        help_url,
        understanding_url: None,
        occurrence_count,
        priority_score,
        confidence: derived.confidence.to_string(),
        false_positive_risk: derived.false_positive_risk.to_string(),
        verification: derived.verification.to_string(),
        assurance,
        complexity,
        complexity_reason,
        complexity_kind,
        expected_impact,
        expected_impact_kind,
        bfsg_relevance,
        remediation_priority,
        occurrences: group
            .iter()
            .take(max_occurrences)
            .map(|c| OccurrenceDetail {
                node_id: c.check.clone(),
                message: c.message.clone(),
                selector: c.selector.clone(),
                tags: vec![c.domain.as_str().to_string()],
                text: c.text.clone(),
                ..Default::default()
            })
            .collect(),
        contract,
    }
}

/// Readable title from a check code: `no_canonical` → `No canonical`.
pub fn check_title(check: &str) -> String {
    let text = check.replace(['_', '.'], " ");
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// #952-3: "79 touch targets too small" is one collected problem standing
    /// for 79 elements. The finding's occurrence count, which drives scope
    /// ("Lokal · n"), complexity and the project-risk count, is the element
    /// count, not 1.
    #[test]
    fn small_touch_targets_count_their_elements_not_one_problem() {
        let mut report = AuditReport::new(
            "https://example.com".into(),
            crate::model::WcagLevel::AA,
            WcagResults::new(),
            100,
        );
        report = report.with_mobile(crate::mobile::MobileFriendliness {
            score: 80,
            viewport: Default::default(),
            touch_targets: Default::default(),
            font_sizes: Default::default(),
            content_sizing: Default::default(),
            issues: vec![crate::mobile::MobileIssue {
                category: "touch_targets".into(),
                issue_type: "small_targets".into(),
                message: "79 touch targets are too small".into(),
                severity: Severity::High,
                impact: "Difficult to tap".into(),
                values: crate::mobile::MobileIssueValues {
                    small_target_count: Some(79),
                    ..Default::default()
                },
            }],
        });
        let n = normalize(&report).normalized;
        let finding = n
            .findings
            .iter()
            .find(|f| f.contract.domain == FindingDomain::Mobile)
            .expect("mobile finding");
        assert_eq!(finding.occurrence_count, 79);
        let kinds =
            crate::audit::management_risk::build_management_risk_kinds(std::slice::from_ref(&n));
        let project = kinds
            .iter()
            .find_map(|k| match k {
                crate::audit::management_risk::ManagementRiskKind::Project { count, .. } => {
                    Some(*count)
                }
                _ => None,
            })
            .unwrap();
        assert_eq!(
            project, 1,
            "the project-risk count reads the same element count"
        );
    }

    /// #921: small targets (2.5.8) and a locked zoom (1.4.4) are observed by
    /// Accessibility and by Mobile. Accessibility charges each cause; the Mobile
    /// finding stays in the report with its values as a reference, so one
    /// defect never lowers two weighted modules.
    #[test]
    fn small_targets_and_locked_zoom_are_charged_once() {
        use crate::wcag::{Violation, WcagLevel};
        let mut results = WcagResults::new();
        results.add_violation(
            Violation::new("2.5.8", "x", WcagLevel::AA, Severity::Medium, "x", "n1")
                .with_rule_id("targets/size"),
        );
        results.add_violation(
            Violation::new("1.4.4", "x", WcagLevel::AA, Severity::High, "x", "n2")
                .with_rule_id("zoom/viewport-locked"),
        );
        let report = AuditReport::new("https://example.com".into(), WcagLevel::AA, results, 1)
            .with_mobile(crate::mobile::MobileFriendliness {
                score: 100,
                viewport: Default::default(),
                touch_targets: Default::default(),
                font_sizes: Default::default(),
                content_sizing: Default::default(),
                issues: vec![
                    crate::mobile::MobileIssue {
                        category: "viewport".into(),
                        issue_type: "not_scalable".into(),
                        message: "Page disables zooming".into(),
                        severity: Severity::Critical,
                        impact: "x".into(),
                        values: Default::default(),
                    },
                    crate::mobile::MobileIssue {
                        category: "touch_targets".into(),
                        issue_type: "small_targets".into(),
                        message: "3 touch targets are too small".into(),
                        severity: Severity::Low,
                        impact: "x".into(),
                        values: crate::mobile::MobileIssueValues {
                            small_target_count: Some(3),
                            ..Default::default()
                        },
                    },
                ],
            });
        let n = normalize(&report).normalized;
        n.validate().unwrap();
        for (a11y_rule, mobile_rule) in [
            ("targets/size", "mobile.small_targets"),
            ("zoom/viewport-locked", "mobile.not_scalable"),
        ] {
            let a11y = n.findings.iter().find(|f| f.rule_id == a11y_rule).unwrap();
            let mobile = n
                .findings
                .iter()
                .find(|f| f.rule_id == mobile_rule)
                .unwrap();
            assert_eq!(a11y.contract.cause, mobile.contract.cause, "{a11y_rule}");
            assert_eq!(a11y.contract.scoring.effect, ScoringEffect::Deduction);
            assert_eq!(mobile.contract.scoring.effect, ScoringEffect::Reference);
            assert!(!mobile.contract.scoring.charges());
            assert!(!mobile.contract.is_primary());
        }
    }

    #[test]
    fn kind_codes_are_snake_case() {
        assert_eq!(
            kind_code(&crate::ux::UxIssueKind::CompetingCtas),
            "competing_ctas"
        );
        assert_eq!(
            kind_code(&crate::commerce::CommerceFindingKind::MissingPrice),
            "missing_price"
        );
    }

    /// #784: UX issues and journey friction points are heuristic. They were
    /// collected as `fail` and, because their rule ids matched none of the
    /// old name substrings, reported as `automatically_confirmed`.
    #[test]
    fn heuristic_domains_are_never_automatically_confirmed() {
        let mut report = AuditReport::new(
            "https://example.com".into(),
            crate::model::WcagLevel::AA,
            WcagResults::new(),
            100,
        );
        let tree = crate::accessibility::AXTree::from_nodes(Vec::new());
        report.ux = Some(crate::ux::analyze_ux(&tree));
        report.journey = Some(crate::page_flow::analyze_journey_with_page_context(
            &tree, false, None,
        ));
        let n = normalize(&report).normalized;
        n.validate().unwrap();
        let heuristic: Vec<_> = n
            .findings
            .iter()
            .filter(|f| {
                matches!(
                    f.contract.domain,
                    FindingDomain::Ux | FindingDomain::Journey
                )
            })
            .collect();
        assert!(
            heuristic
                .iter()
                .any(|f| f.contract.domain == FindingDomain::Ux)
                && heuristic
                    .iter()
                    .any(|f| f.contract.domain == FindingDomain::Journey),
            "an empty page yields UX and journey findings"
        );
        for f in heuristic {
            assert_eq!(f.contract.outcome, FindingOutcome::Review, "{}", f.rule_id);
            assert_eq!(f.assurance.method, DetectionMethod::Review, "{}", f.rule_id);
            assert_eq!(f.verification, "manual_review_recommended", "{}", f.rule_id);
            assert_ne!(f.confidence, "very_high", "{}", f.rule_id);
        }
    }

    /// #784 counter-case for collected findings: the same check, once
    /// measured with a selector, once a review, once without anything that
    /// locates its element.
    #[test]
    fn same_check_with_different_evidence_differs_in_assurance() {
        let make = |selector: Option<&str>, review: bool| {
            let mut c = Collected::new(
                FindingDomain::Mobile,
                "mobile",
                "low_contrast",
                Severity::Medium,
                "text too light",
                FindingScoring::none(),
                LocationScope::Element,
            )
            .selector(selector.map(str::to_string));
            if review {
                c = c.review();
            }
            group_finding(vec![c], 5)
        };
        let located = make(Some("p.lead"), false);
        assert_eq!(located.rule_id, "mobile.low_contrast");
        assert_eq!(located.verification, "automatically_confirmed");
        assert_eq!(located.assurance.evidence, EvidenceQuality::Located);

        let review = make(Some("p.lead"), true);
        assert_eq!(review.rule_id, located.rule_id);
        assert_eq!(review.verification, "manual_review_recommended");

        let unlocated = make(None, false);
        assert_eq!(unlocated.rule_id, located.rule_id);
        assert_eq!(unlocated.assurance.evidence, EvidenceQuality::Unlocated);
        assert_eq!(unlocated.verification, "manual_review_recommended");
    }

    /// Unknown applicability never confirms (#784).
    #[test]
    fn unknown_applicability_is_a_review() {
        let mut c = Collected::new(
            FindingDomain::Seo,
            "seo.technical",
            "noindex",
            Severity::Medium,
            "noindex",
            FindingScoring::none(),
            LocationScope::Page,
        );
        c.applicability = Applicability::Unknown;
        let f = group_finding(vec![c], 5);
        assert_eq!(f.assurance.method, DetectionMethod::Review);
        assert_eq!(f.verification, "manual_review_recommended");
    }

    /// #929: a finding of the security, structured-data and library checks
    /// links to its page by the id it carries; any other collected check has
    /// no page and no link.
    #[test]
    fn collected_findings_link_to_their_page_by_their_id() {
        let finding = |source: &'static str, check: &str| {
            group_finding(
                vec![Collected::new(
                    FindingDomain::Security,
                    source,
                    check,
                    Severity::High,
                    "x".to_string(),
                    FindingScoring::none(),
                    LocationScope::Page,
                )],
                5,
            )
        };
        let url = |source, check| finding(source, check).help_url;
        assert_eq!(
            url("security.headers", "missing_header").as_deref(),
            Some("https://casoon.github.io/auditmysite/rules/security.headers.missing_header/")
        );
        assert_eq!(
            url("best_practices.vulnerable_libraries", "jquery").as_deref(),
            Some(
                "https://casoon.github.io/auditmysite/rules/best_practices.vulnerable_libraries.jquery/"
            )
        );
        assert_eq!(
            url("seo.schema", "schema_rule_missing_product_snippet_offers").as_deref(),
            Some(
                "https://casoon.github.io/auditmysite/rules/seo.schema.schema_rule_missing_product_snippet/"
            )
        );
        assert_eq!(url("seo.meta", "description"), None);
        assert_eq!(
            finding("security.headers", "missing_header").understanding_url,
            None
        );
    }

    #[test]
    fn check_titles_are_readable() {
        assert_eq!(check_title("no_canonical"), "No canonical");
        assert_eq!(check_title("noindex"), "Noindex");
    }

    /// A module added to the catalog without a policy would collect data the
    /// finding contract never sees.
    #[test]
    fn every_catalog_module_has_a_finding_policy() {
        let catalog = crate::audit::AuditCatalog::standard();
        let modules = catalog.topo_sorted().unwrap();
        for module in &modules {
            assert!(
                DOMAIN_POLICIES.iter().any(|(id, _)| *id == module.id()),
                "catalog module `{}` has no finding policy",
                module.id()
            );
        }
        for (id, _) in DOMAIN_POLICIES {
            assert!(
                modules.iter().any(|m| m.id() == *id),
                "policy for unknown module `{id}`"
            );
        }
    }

    use crate::audit::normalized::{normalize, NormalizedReport};
    use crate::audit::AuditReport;
    use crate::seo::technical::TechnicalIssue;
    use crate::seo::SeoAnalysis;
    use crate::wcag::WcagResults;

    /// A page with complete SEO basics, so the score reflects only what a
    /// test changes.
    fn seo(noindex: bool) -> SeoAnalysis {
        let mut seo = SeoAnalysis::default();
        seo.meta.title = Some("Title".into());
        seo.meta.description = Some("Description".into());
        seo.technical.https = true;
        seo.technical.has_canonical = true;
        seo.technical.has_lang = true;
        seo.headings.h1_count = 1;
        seo.social.open_graph = Some(Default::default());
        seo.social.twitter_card = Some(Default::default());
        if noindex {
            seo.technical.robots_meta = Some("noindex, follow".into());
            seo.technical.issues.push(TechnicalIssue {
                issue_type: "noindex".into(),
                message: "Page has a noindex directive".into(),
                severity: Severity::High,
            });
        }
        seo
    }

    fn page(noindex: bool, expectation: IndexabilityExpectation) -> AuditReport {
        let mut report = AuditReport::new(
            "https://example.com".into(),
            crate::model::WcagLevel::AA,
            WcagResults::new(),
            100,
        );
        let mut seo = seo(noindex);
        crate::seo::rescore(&mut seo, expectation);
        report.discoverability.seo = Some(seo);
        report.run_context.indexability = expectation;
        report
    }

    fn seo_score(n: &NormalizedReport) -> u32 {
        n.module_scores
            .iter()
            .find(|m| m.name == "SEO")
            .unwrap()
            .score
    }

    fn noindex(n: &NormalizedReport) -> Option<&NormalizedFinding> {
        n.findings
            .iter()
            .find(|f| f.rule_id == "seo.technical.noindex")
    }

    /// U15 reproduction: an indexable page and a noindex page both scored SEO
    /// 90 with `findings: []`. Without a declared goal the noindex page now
    /// carries a finding to review, still without a score effect: an
    /// intended noindex page is no SEO defect.
    #[test]
    fn undeclared_noindex_is_a_finding_to_review_without_score_effect() {
        let indexable = normalize(&page(false, Default::default())).normalized;
        let n = normalize(&page(true, Default::default())).normalized;
        n.validate().unwrap();

        assert!(noindex(&indexable).is_none());
        let finding = noindex(&n).expect("noindex reaches the canonical findings");
        assert_eq!(finding.contract.domain, FindingDomain::Seo);
        assert_eq!(finding.contract.applicability, Applicability::Unknown);
        assert_eq!(finding.contract.outcome, FindingOutcome::Review);
        assert_eq!(finding.contract.scoring.effect, ScoringEffect::None);
        assert_eq!(
            finding.contract.applicability_basis.as_deref(),
            Some("indexability:undeclared")
        );
        assert_eq!(seo_score(&n), seo_score(&indexable));
        assert_eq!(crate::audit::verdict::technical_blockers(&n), 0);
    }

    /// Meant to be indexed (declared or from a sitemap): noindex is a
    /// blocker — SEO capped, decision reason, configurable to fail.
    #[test]
    fn expected_indexable_noindex_is_a_gate() {
        for expectation in [
            IndexabilityExpectation::declared(true),
            IndexabilityExpectation::sitemap(),
        ] {
            let indexable = normalize(&page(false, expectation)).normalized;
            let mut report = page(true, expectation);
            let n = normalize(&report).normalized;
            n.validate().unwrap();

            let finding = noindex(&n).unwrap();
            assert_eq!(finding.contract.scoring.effect, ScoringEffect::Gate);
            assert_eq!(finding.contract.scoring.owner.as_deref(), Some("SEO"));
            assert_eq!(finding.severity, Severity::Critical);
            assert!(seo_score(&indexable) > crate::seo::INDEXABILITY_BLOCKER_CAP);
            assert!(seo_score(&n) <= crate::seo::INDEXABILITY_BLOCKER_CAP);
            assert_eq!(
                report.discoverability.seo.as_ref().unwrap().score,
                seo_score(&n),
                "raw SEO score and module score agree"
            );
            assert_eq!(n.decision.verdict, crate::audit::Verdict::Warn);
            assert!(n
                .decision
                .reasons
                .contains(&"technical_blockers: 1".to_string()));

            report.run_context.decision_profile =
                crate::audit::DecisionProfile::resolve(None, None, Some(true), None, None);
            let strict = normalize(&report).normalized;
            assert_eq!(strict.decision.verdict, crate::audit::Verdict::Fail);
            assert_eq!(strict.decision.profile.name, "custom");
        }
    }

    /// An intended noindex is not applicable: no finding, no deduction, but
    /// an explicit inventory exclusion.
    #[test]
    fn intended_noindex_is_an_explicit_exclusion() {
        let expectation = IndexabilityExpectation::declared(false);
        let indexable = normalize(&page(false, expectation)).normalized;
        let n = normalize(&page(true, expectation)).normalized;
        n.validate().unwrap();
        assert!(noindex(&n).is_none());
        assert_eq!(seo_score(&n), seo_score(&indexable));
        assert!(n.finding_inventory.iter().any(|e| e.check == "noindex"
            && e.disposition
                == Disposition::Excluded {
                    reason: ExclusionReason::NotApplicable
                }));
    }

    /// A published SEO score above the blocker cap contradicts the gate.
    #[test]
    fn gate_above_its_cap_is_a_contract_violation() {
        let mut n = normalize(&page(true, IndexabilityExpectation::declared(true))).normalized;
        n.module_scores
            .iter_mut()
            .find(|m| m.name == "SEO")
            .unwrap()
            .score = 90;
        let codes: Vec<_> = n
            .validate()
            .unwrap_err()
            .violations
            .iter()
            .map(|v| v.code)
            .collect();
        assert!(codes.contains(&"gate_score_cap"), "{codes:?}");
    }

    /// Cross-module case: a missing `lang` is a WCAG failure (screen
    /// readers), an SEO technical issue (search language) and an SEO meta
    /// issue. All three stay visible under one cause; Accessibility and SEO
    /// each charge their own consequence once, the meta repeat is a
    /// reference. Making the repeat charge too breaks the contract.
    #[test]
    fn one_cause_across_modules_is_grouped_and_charged_once_per_score() {
        let mut results = WcagResults::new();
        results.add_violation(
            crate::wcag::Violation::new(
                "3.1.1",
                "Language of page",
                crate::model::WcagLevel::A,
                Severity::High,
                "html element has no lang attribute",
                "n1",
            )
            .with_rule_id("document/lang-missing")
            .with_selector("html"),
        );
        let mut report = AuditReport::new(
            "https://example.com".into(),
            crate::model::WcagLevel::AA,
            results,
            100,
        );
        let mut seo = seo(false);
        seo.technical.has_lang = false;
        seo.technical.issues.push(TechnicalIssue {
            issue_type: "no_lang".into(),
            message: "No lang attribute".into(),
            severity: Severity::Medium,
        });
        seo.meta_issues.push(crate::seo::MetaValidation {
            field: "lang".into(),
            message: "Missing lang attribute".into(),
            severity: Severity::Medium,
            suggestion: None,
            kind: None,
            values: Default::default(),
        });
        crate::seo::rescore(&mut seo, Default::default());
        report.discoverability.seo = Some(seo);

        let mut n = normalize(&report).normalized;
        n.validate().unwrap();
        let same_cause: Vec<_> = n
            .findings
            .iter()
            .filter(|f| f.contract.cause == LANG_MISSING)
            .map(|f| {
                (
                    f.rule_id.as_str(),
                    f.contract.scoring.owner.clone().unwrap_or_default(),
                    f.contract.scoring.effect,
                )
            })
            .collect();
        for expected in [
            (
                "document/lang-missing",
                "Accessibility".to_string(),
                ScoringEffect::Deduction,
            ),
            (
                "seo.technical.no_lang",
                "SEO".to_string(),
                ScoringEffect::Deduction,
            ),
            ("seo.meta.lang", "SEO".to_string(), ScoringEffect::Reference),
        ] {
            assert!(
                same_cause.contains(&expected),
                "{expected:?} in {same_cause:?}"
            );
        }
        assert_eq!(same_cause.len(), 3, "{same_cause:?}");
        // Counting stays separate: only the WCAG finding is a violation.
        assert_eq!(n.severity_counts.total, 1);

        n.findings
            .iter_mut()
            .find(|f| f.rule_id == "seo.meta.lang")
            .unwrap()
            .contract
            .scoring
            .effect = ScoringEffect::Deduction;
        let codes: Vec<_> = n
            .validate()
            .unwrap_err()
            .violations
            .iter()
            .map(|v| v.code)
            .collect();
        assert!(codes.contains(&"cause_charged_twice"), "{codes:?}");
    }

    /// One missing `<h1>` is reported by SEO, UX and Journey (#779). SEO
    /// charges it; the other two are indicators of the same cause, and no
    /// WCAG finding exists for it (best practice). Without the shared cause
    /// each looked like an independent defect.
    #[test]
    fn missing_h1_is_one_cause_charged_by_seo_only() {
        let mut report = AuditReport::new(
            "https://example.com".into(),
            crate::model::WcagLevel::AA,
            WcagResults::new(),
            100,
        );
        let mut seo = seo(false);
        seo.headings.h1_count = 0;
        seo.headings
            .issues
            .push(crate::seo::headings::HeadingIssue {
                issue_type: "missing_h1".into(),
                message: "No h1".into(),
                severity: Severity::High,
                long_heading: None,
            });
        crate::seo::rescore(&mut seo, Default::default());
        report.discoverability.seo = Some(seo);
        let mut ux = crate::ux::analyze_ux(&crate::AXTree::new());
        ux.issues = vec![crate::ux::UxIssue {
            kind: crate::ux::UxIssueKind::NoH1,
            dimension: "Visual Hierarchy".to_string(),
            severity: "high".to_string(),
            problem: "No H1.".to_string(),
            impact: "Orientation suffers.".to_string(),
            recommendation: "Add an H1.".to_string(),
            values: Default::default(),
        }];
        report = report.with_ux(ux);
        let mut journey = crate::page_flow::analyze_journey(&crate::AXTree::new());
        journey.friction_points = vec![crate::page_flow::FrictionPoint {
            kind: crate::page_flow::FrictionKind::NoH1,
            step: "entry".to_string(),
            severity: "high".to_string(),
            problem: "No H1.".to_string(),
            impact: "Orientation suffers.".to_string(),
            recommendation: "Add an H1.".to_string(),
            values: Default::default(),
        }];
        report = report.with_journey(journey);

        let n = normalize(&report).normalized;
        n.validate().unwrap();
        let mut same_cause: Vec<_> = n
            .findings
            .iter()
            .filter(|f| f.contract.cause == H1_MISSING)
            .map(|f| {
                (
                    f.rule_id.clone(),
                    f.contract.scoring.owner.clone().unwrap_or_default(),
                    f.contract.scoring.effect,
                )
            })
            .collect();
        same_cause.sort_by(|a, b| a.0.cmp(&b.0));
        assert_eq!(
            same_cause,
            vec![
                (
                    "journey.no_h1".to_string(),
                    "Journey".to_string(),
                    ScoringEffect::Indicator
                ),
                (
                    "seo.headings.missing_h1".to_string(),
                    "SEO".to_string(),
                    ScoringEffect::Deduction
                ),
                (
                    "ux.no_h1".to_string(),
                    "UX".to_string(),
                    ScoringEffect::Indicator
                ),
            ]
        );
        // The only primary (actionable) finding of the cause is the SEO one.
        let primary: Vec<_> = n
            .findings
            .iter()
            .filter(|f| f.contract.cause == H1_MISSING && f.contract.is_primary())
            .map(|f| f.rule_id.as_str())
            .collect();
        assert_eq!(primary, ["seo.headings.missing_h1"]);
        // No WCAG finding: best practice is not a conformance failure.
        assert!(n
            .findings
            .iter()
            .all(|f| !f.rule_id.starts_with("headings/h1-")));
    }
}
