//! Unified assessment type system (issue #51).
//!
//! Provides common types for assessment-level results, evidence confidence,
//! and content signals that all analysis modules can share. This is the
//! foundation for Content Visibility Analysis (#54) and Evidence Model (#52).
//!
//! # Design
//!
//! Every finding from every module maps to a [`ContentSignal`]:
//! - WCAG violations → `Violation` or `Warning` level, `Accessibility` area
//! - SEO signal checks → `Pass` / `Warning`, `Seo` area
//! - Pattern detections → `Positive`, `Pattern` area
//! - Untestable criteria → `NotTestable`
//!
//! [`EvidenceConfidence`] expresses how reliable the automated detection is —
//! not whether the content claim is true.

use serde::{Deserialize, Serialize};

use crate::patterns::{PatternConfidence, RecognizedPattern};
use crate::seo::profile::SignalCheck;
use crate::wcag::types::FindingKind;

// ─── Content Area ────────────────────────────────────────────────────────────

/// Top-level domain a signal belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentArea {
    Accessibility,
    Seo,
    SourceQuality,
    AiVisibility,
    Performance,
    Security,
    Mobile,
    Pattern,
    Content,
}

impl ContentArea {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Accessibility => "Accessibility",
            Self::Seo => "SEO",
            Self::SourceQuality => "Source Quality",
            Self::AiVisibility => "AI Visibility",
            Self::Performance => "Performance",
            Self::Security => "Security",
            Self::Mobile => "Mobile",
            Self::Pattern => "Pattern",
            Self::Content => "Content",
        }
    }
}

// ─── Assessment Level ─────────────────────────────────────────────────────────

/// Assessment outcome of a single check — generalized across all modules.
///
/// This is the cross-module counterpart of [`crate::wcag::types::FindingKind`].
/// `Pass` is the only variant added here that `FindingKind` does not have.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssessmentLevel {
    /// Check passed cleanly.
    Pass,
    /// Concrete problem detected automatically.
    Violation,
    /// Heuristic suspicion; automated tool cannot confirm without more context.
    Warning,
    /// Good pattern actively detected (positive evidence).
    Positive,
    /// Criterion exists but cannot be evaluated automatically.
    NotTestable,
}

impl AssessmentLevel {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Pass => "Pass",
            Self::Violation => "Violation",
            Self::Warning => "Warning",
            Self::Positive => "Positive",
            Self::NotTestable => "Not Testable",
        }
    }

    pub fn is_problem(&self) -> bool {
        matches!(self, Self::Violation | Self::Warning)
    }
}

impl From<FindingKind> for AssessmentLevel {
    fn from(kind: FindingKind) -> Self {
        match kind {
            FindingKind::Violation => Self::Violation,
            FindingKind::Warning => Self::Warning,
            FindingKind::Positive => Self::Positive,
            FindingKind::NotTestable => Self::NotTestable,
        }
    }
}

// ─── Evidence Confidence ─────────────────────────────────────────────────────

/// How reliable the automated signal is — not whether the content claim is true.
///
/// - `High`: direct DOM/AXTree attribute or known structural fact
/// - `Medium`: heuristic with good precision in practice
/// - `Low`: weak proxy, presence-only, or highly context-dependent
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceConfidence {
    Low,
    Medium,
    High,
}

impl EvidenceConfidence {
    pub fn label(&self) -> &'static str {
        match self {
            Self::High => "High",
            Self::Medium => "Medium",
            Self::Low => "Low",
        }
    }
}

impl From<PatternConfidence> for EvidenceConfidence {
    fn from(pc: PatternConfidence) -> Self {
        match pc {
            PatternConfidence::Strong => Self::High,
            PatternConfidence::Partial => Self::Medium,
        }
    }
}

// ─── Evidence Source ─────────────────────────────────────────────────────────

/// Where the automated signal was detected.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceSource {
    /// HTML `<meta>` tag
    Meta,
    /// JSON-LD structured data
    JsonLd,
    /// Visible text content on the page
    VisibleText,
    /// `<a>` href or link text
    Link,
    /// HTTP response header
    HttpHeader,
    /// Browser Accessibility Tree (AXTree) node
    AxTree,
    /// HTML DOM attribute (e.g. `alt`, `title`, `aria-*`)
    DomAttribute,
    /// Computed CSS property
    CssProperty,
    /// Derived or aggregated value
    Computed,
}

// ─── Content Evidence ─────────────────────────────────────────────────────────

/// Machine-readable provenance for a single signal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentEvidence {
    /// Where the value was found.
    pub source: EvidenceSource,
    /// Dotted path to the field, e.g. `"LocalBusiness.geo.latitude"`.
    pub field_path: Option<String>,
    /// Short value excerpt for display in reports.
    pub value_excerpt: Option<String>,
    /// How reliable this evidence is.
    pub confidence: EvidenceConfidence,
}

impl ContentEvidence {
    pub fn new(source: EvidenceSource, confidence: EvidenceConfidence) -> Self {
        Self {
            source,
            field_path: None,
            value_excerpt: None,
            confidence,
        }
    }

    pub fn with_field(mut self, path: impl Into<String>) -> Self {
        self.field_path = Some(path.into());
        self
    }

    pub fn with_value(mut self, excerpt: impl Into<String>) -> Self {
        self.value_excerpt = Some(excerpt.into());
        self
    }
}

// ─── Content Signal ───────────────────────────────────────────────────────────

/// A single, self-contained assessment result from any module.
///
/// This is the common output unit across all analysis dimensions. Use the
/// builder methods to construct signals. Evidence is optional but recommended
/// wherever it can be derived automatically.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentSignal {
    /// Which analysis domain produced this signal.
    pub area: ContentArea,
    /// Outcome of the check.
    pub level: AssessmentLevel,
    /// How reliably the automated tool detected this signal.
    pub confidence: EvidenceConfidence,
    /// Short human-readable label (suitable for table/list display).
    pub title: String,
    /// Full description or finding detail.
    pub detail: String,
    /// Machine-readable provenance (may be empty for derived signals).
    pub evidence: Vec<ContentEvidence>,
    /// Stable kind for content-visibility signals, enabling localized
    /// re-derivation of `title`/`detail` from `cv_values` in the PDF layer.
    /// `None` for signals produced outside content-visibility analysis.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cv_kind: Option<crate::content_visibility::ContentVisibilitySignalKind>,
    /// Interpolated values needed to reproduce `title`/`detail` in another
    /// language. Only meaningful when `cv_kind` is set.
    #[serde(
        default,
        skip_serializing_if = "crate::content_visibility::ContentSignalValues::is_empty"
    )]
    pub cv_values: crate::content_visibility::ContentSignalValues,
}

impl ContentSignal {
    pub fn new(
        area: ContentArea,
        level: AssessmentLevel,
        confidence: EvidenceConfidence,
        title: impl Into<String>,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            area,
            level,
            confidence,
            title: title.into(),
            detail: detail.into(),
            evidence: vec![],
            cv_kind: None,
            cv_values: crate::content_visibility::ContentSignalValues::default(),
        }
    }

    /// Attach a content-visibility kind and its interpolated values so the PDF
    /// layer can re-derive localized `title`/`detail`.
    pub fn with_cv(
        mut self,
        kind: crate::content_visibility::ContentVisibilitySignalKind,
        values: crate::content_visibility::ContentSignalValues,
    ) -> Self {
        self.cv_kind = Some(kind);
        self.cv_values = values;
        self
    }

    pub fn with_evidence(mut self, ev: ContentEvidence) -> Self {
        self.evidence.push(ev);
        self
    }

    pub fn with_evidence_list(mut self, list: Vec<ContentEvidence>) -> Self {
        self.evidence = list;
        self
    }

    // ── Convenience constructors ─────────────────────────────────────────

    pub fn pass(
        area: ContentArea,
        confidence: EvidenceConfidence,
        title: impl Into<String>,
        detail: impl Into<String>,
    ) -> Self {
        Self::new(area, AssessmentLevel::Pass, confidence, title, detail)
    }

    pub fn violation(
        area: ContentArea,
        confidence: EvidenceConfidence,
        title: impl Into<String>,
        detail: impl Into<String>,
    ) -> Self {
        Self::new(area, AssessmentLevel::Violation, confidence, title, detail)
    }

    pub fn warning(
        area: ContentArea,
        confidence: EvidenceConfidence,
        title: impl Into<String>,
        detail: impl Into<String>,
    ) -> Self {
        Self::new(area, AssessmentLevel::Warning, confidence, title, detail)
    }

    pub fn positive(
        area: ContentArea,
        confidence: EvidenceConfidence,
        title: impl Into<String>,
        detail: impl Into<String>,
    ) -> Self {
        Self::new(area, AssessmentLevel::Positive, confidence, title, detail)
    }
}

// ─── From conversions ────────────────────────────────────────────────────────

impl From<&crate::wcag::Violation> for ContentSignal {
    fn from(v: &crate::wcag::Violation) -> Self {
        let level = AssessmentLevel::from(v.kind);
        let confidence = match v.kind {
            FindingKind::Violation => EvidenceConfidence::High,
            FindingKind::Warning => EvidenceConfidence::Medium,
            _ => EvidenceConfidence::Medium,
        };
        let mut sig = Self::new(
            ContentArea::Accessibility,
            level,
            confidence,
            format!("WCAG {} – {}", v.rule, v.rule_name),
            v.message.clone(),
        );
        // Map ViolationEvidence items from the violation itself.
        for ev in &v.evidence {
            let source = match ev.source.as_str() {
                "ax_tree" => EvidenceSource::AxTree,
                "dom_attribute" => EvidenceSource::DomAttribute,
                "meta" => EvidenceSource::Meta,
                "css_property" => EvidenceSource::CssProperty,
                "http_header" => EvidenceSource::HttpHeader,
                _ => EvidenceSource::Computed,
            };
            let mut ce = ContentEvidence::new(source, confidence);
            if let Some(f) = &ev.field {
                ce = ce.with_field(f);
            }
            if let Some(val) = &ev.value {
                ce = ce.with_value(val);
            }
            sig.evidence.push(ce);
        }
        // Fallback: if no structured evidence, use the selector.
        if sig.evidence.is_empty() {
            if let Some(sel) = &v.selector {
                sig.evidence.push(
                    ContentEvidence::new(EvidenceSource::AxTree, EvidenceConfidence::High)
                        .with_value(sel),
                );
            }
        }
        sig
    }
}

/// Convert a SEO signal check into a `ContentSignal`.
/// `category` is the parent `SignalCategory::name`, used to build the title.
pub fn signal_from_check(check: &SignalCheck, category: &str) -> ContentSignal {
    let level = if check.passed {
        AssessmentLevel::Pass
    } else {
        AssessmentLevel::Warning
    };
    let detail = check.detail.clone().unwrap_or_else(|| check.label.clone());
    let mut sig = ContentSignal::new(
        ContentArea::Seo,
        level,
        EvidenceConfidence::Medium,
        format!("{} – {}", category, check.label),
        detail,
    );
    sig.evidence.push(ContentEvidence::new(
        EvidenceSource::Computed,
        EvidenceConfidence::Medium,
    ));
    sig
}

impl From<&RecognizedPattern> for ContentSignal {
    fn from(p: &RecognizedPattern) -> Self {
        let confidence = EvidenceConfidence::from(p.confidence);
        ContentSignal::positive(
            ContentArea::Pattern,
            confidence,
            p.pattern.clone(),
            p.message.clone(),
        )
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assessment_level_from_finding_kind() {
        assert_eq!(
            AssessmentLevel::from(FindingKind::Violation),
            AssessmentLevel::Violation
        );
        assert_eq!(
            AssessmentLevel::from(FindingKind::Warning),
            AssessmentLevel::Warning
        );
        assert_eq!(
            AssessmentLevel::from(FindingKind::Positive),
            AssessmentLevel::Positive
        );
        assert_eq!(
            AssessmentLevel::from(FindingKind::NotTestable),
            AssessmentLevel::NotTestable
        );
    }

    #[test]
    fn assessment_level_is_problem() {
        assert!(AssessmentLevel::Violation.is_problem());
        assert!(AssessmentLevel::Warning.is_problem());
        assert!(!AssessmentLevel::Pass.is_problem());
        assert!(!AssessmentLevel::Positive.is_problem());
        assert!(!AssessmentLevel::NotTestable.is_problem());
    }

    #[test]
    fn evidence_confidence_from_pattern_confidence() {
        assert_eq!(
            EvidenceConfidence::from(PatternConfidence::Strong),
            EvidenceConfidence::High
        );
        assert_eq!(
            EvidenceConfidence::from(PatternConfidence::Partial),
            EvidenceConfidence::Medium
        );
    }

    #[test]
    fn evidence_confidence_ordering() {
        assert!(EvidenceConfidence::High > EvidenceConfidence::Medium);
        assert!(EvidenceConfidence::Medium > EvidenceConfidence::Low);
    }

    #[test]
    fn content_signal_violation_from_wcag_violation() {
        use crate::cli::WcagLevel;
        use crate::wcag::types::Violation;

        let v = Violation::new(
            "1.1.1",
            "Non-text Content",
            WcagLevel::A,
            crate::wcag::Severity::High,
            "Image missing alt",
            "img#logo",
        )
        .with_selector("img#logo");
        let sig = ContentSignal::from(&v);
        assert_eq!(sig.area, ContentArea::Accessibility);
        assert_eq!(sig.level, AssessmentLevel::Violation);
        assert!(!sig.evidence.is_empty());
    }

    #[test]
    fn content_signal_from_recognized_pattern() {
        let p = RecognizedPattern {
            pattern: "MainNavigation".to_string(),
            message: "nav[aria-label] with 5 links found".to_string(),
            confidence: PatternConfidence::Strong,
        };
        let sig = ContentSignal::from(&p);
        assert_eq!(sig.area, ContentArea::Pattern);
        assert_eq!(sig.level, AssessmentLevel::Positive);
        assert_eq!(sig.confidence, EvidenceConfidence::High);
    }

    #[test]
    fn content_signal_from_signal_check_pass() {
        let check = SignalCheck {
            label: "Title vorhanden".to_string(),
            passed: true,
            detail: Some("42 Zeichen".to_string()),
        };
        let sig = signal_from_check(&check, "Meta-Tags");
        assert_eq!(sig.area, ContentArea::Seo);
        assert_eq!(sig.level, AssessmentLevel::Pass);
        assert!(sig.title.contains("Meta-Tags"));
    }

    #[test]
    fn content_signal_from_signal_check_fail() {
        let check = SignalCheck {
            label: "Description vorhanden".to_string(),
            passed: false,
            detail: None,
        };
        let sig = signal_from_check(&check, "Meta-Tags");
        assert_eq!(sig.level, AssessmentLevel::Warning);
    }

    #[test]
    fn content_signal_builder_with_evidence() {
        let sig = ContentSignal::violation(
            ContentArea::Seo,
            EvidenceConfidence::High,
            "Missing canonical",
            "No canonical URL found",
        )
        .with_evidence(
            ContentEvidence::new(EvidenceSource::Meta, EvidenceConfidence::High)
                .with_field("canonical")
                .with_value("(absent)"),
        );
        assert_eq!(sig.evidence.len(), 1);
        assert_eq!(sig.evidence[0].field_path.as_deref(), Some("canonical"));
    }

    #[test]
    fn content_signal_from_violation_maps_violation_evidence() {
        use crate::cli::WcagLevel;
        use crate::wcag::types::{Violation, ViolationEvidence};

        let v = Violation::new(
            "4.1.2",
            "Name, Role, Value",
            WcagLevel::A,
            crate::wcag::Severity::High,
            "Missing role",
            "btn-1",
        )
        .with_evidence_item(ViolationEvidence::ax_tree("button.nav-toggle"))
        .with_evidence_item(ViolationEvidence::dom_attribute("aria-label", None));

        let sig = ContentSignal::from(&v);
        assert_eq!(sig.evidence.len(), 2);
        assert!(matches!(sig.evidence[0].source, EvidenceSource::AxTree));
        assert!(matches!(
            sig.evidence[1].source,
            EvidenceSource::DomAttribute
        ));
        assert_eq!(
            sig.evidence[0].value_excerpt.as_deref(),
            Some("button.nav-toggle")
        );
    }

    #[test]
    fn content_signal_from_violation_uses_selector_fallback_when_no_evidence() {
        use crate::cli::WcagLevel;
        use crate::wcag::types::Violation;

        let v = Violation::new(
            "1.1.1",
            "Non-text Content",
            WcagLevel::A,
            crate::wcag::Severity::High,
            "No alt",
            "img-1",
        )
        .with_selector("img.hero");

        let sig = ContentSignal::from(&v);
        assert_eq!(sig.evidence.len(), 1);
        assert!(matches!(sig.evidence[0].source, EvidenceSource::AxTree));
        assert_eq!(sig.evidence[0].value_excerpt.as_deref(), Some("img.hero"));
    }
}
