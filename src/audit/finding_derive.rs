//! Derived finding attributes: priority score, confidence, false-positive
//! risk, verification, complexity/expected-impact kinds and their #406 en/de
//! texts, BFSG relevance, remediation priority and per-rule effort weight.
//!
//! Moved out of `audit::normalized`; every public item stays reachable at its
//! old `auditmysite::audit::normalized::…` path via `pub use`.

use serde::{Deserialize, Serialize};

use crate::audit::scoring::{ScoreGain, ScoreGainBasis};
use crate::i18n::{count_text, text, text_args};
use crate::taxonomy::{RuleLookup, Severity};
use fluent_bundle::FluentValue;

pub(super) fn calculate_priority_score(
    severity: Severity,
    occurrence_count: usize,
    rule_id: &str,
) -> f32 {
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

/// How the check behind a finding reaches its result (#784).
///
/// Together with [`EvidenceQuality`] the only input to `derive_assurance`:
/// a rule id or category string alone never confirms a finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum DetectionMethod {
    /// The check measures the property the criterion names (a missing
    /// attribute, a computed contrast ratio).
    #[default]
    Measured,
    /// The check detects a pattern that often, not always, means the
    /// failure (a generic link text for 2.4.4). See `PATTERN_RULES`.
    Pattern,
    /// The check reports a suspicion, or whether it applies to the page is
    /// unknown.
    Review,
}

/// What the occurrences of one finding carry as evidence (#784).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum EvidenceQuality {
    /// Every occurrence names its element or carries recorded evidence.
    #[default]
    Located,
    /// Some occurrences are located, others are not.
    Partial,
    /// A document-wide check: no element to name, the document was read.
    Document,
    /// Element occurrences none of which could be located.
    Unlocated,
}

/// Evidence anchor of a single occurrence, folded by [`evidence_quality`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum OccurrenceAnchor {
    /// Selector, HTML snippet or recorded evidence.
    Element,
    /// A document-wide observation without an element.
    Document,
    /// An element occurrence without anything that locates it.
    Missing,
}

pub(super) fn evidence_quality(
    anchors: impl IntoIterator<Item = OccurrenceAnchor>,
) -> EvidenceQuality {
    let (mut element, mut document, mut missing) = (0usize, 0usize, 0usize);
    for anchor in anchors {
        match anchor {
            OccurrenceAnchor::Element => element += 1,
            OccurrenceAnchor::Document => document += 1,
            OccurrenceAnchor::Missing => missing += 1,
        }
    }
    if element > 0 && document == 0 && missing == 0 {
        EvidenceQuality::Located
    } else if element > 0 {
        EvidenceQuality::Partial
    } else if missing > 0 || document == 0 {
        EvidenceQuality::Unlocated
    } else {
        EvidenceQuality::Document
    }
}

/// Taxonomy rules whose check detects a pattern, not the failure of the
/// criterion itself: the pattern is found reliably, whether it fails the
/// criterion needs a person (#784). A declared method, reviewed per
/// rule — not a substring of the id.
pub(crate) const PATTERN_RULES: &[&str] = &[
    // DOM order diverging from the visual order; whether the order is
    // illogical is a judgement.
    "dialog/focusable-missing",
    // Generic link texts; the context may still convey the purpose.
    "links/generic-name",
    "links/url-name",
    "links/single-character-name",
    "links/purpose-link-only",
    // Several h1 elements are allowed; 1.3.1 fails only if the structure
    // they express is wrong.
    "headings/h1-multiple",
    // A name from `title` is a valid accessible name; fragile, not failing.
    "names/title-only",
    // Fixed pixel values and overflow:hidden stand in for an actual
    // resize measurement.
    "zoom/viewport-locked",
    // CSS styling heuristics for line spacing and justified text.
    "text/justified",
    "text/line-height-tight",
];

/// Detection method of a WCAG finding: violations reaching the findings are
/// the rule's own `fail` outcome, so a rule is measured unless declared in
/// [`PATTERN_RULES`].
pub(super) fn wcag_detection_method(rule_id: &str) -> DetectionMethod {
    if PATTERN_RULES.contains(&rule_id) {
        DetectionMethod::Pattern
    } else {
        DetectionMethod::Measured
    }
}

/// Confidence, false-positive risk and verification of one finding, stored
/// as the JSON strings the 1.x vocabulary defines.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Assurance {
    pub confidence: &'static str,
    pub false_positive_risk: &'static str,
    pub verification: &'static str,
}

/// Derives assurance from method and evidence of the single finding (#784).
///
/// Automatic confirmation needs both a measuring check and evidence: a
/// located element, or a document-wide observation. A pattern, a review
/// outcome or occurrences that could not be located always ask for a manual
/// check — whatever the rule is called.
pub(super) fn derive_assurance(method: DetectionMethod, evidence: EvidenceQuality) -> Assurance {
    const REVIEW: Assurance = Assurance {
        confidence: "medium",
        false_positive_risk: "medium",
        verification: "manual_review_recommended",
    };
    match (method, evidence) {
        (DetectionMethod::Pattern | DetectionMethod::Review, _)
        | (DetectionMethod::Measured, EvidenceQuality::Unlocated) => REVIEW,
        (DetectionMethod::Measured, EvidenceQuality::Located) => Assurance {
            confidence: "very_high",
            false_positive_risk: "very_low",
            verification: "automatically_confirmed",
        },
        (DetectionMethod::Measured, EvidenceQuality::Partial | EvidenceQuality::Document) => {
            Assurance {
                confidence: "high",
                false_positive_risk: "low",
                verification: "automatically_confirmed",
            }
        }
    }
}

/// The basis `derive_assurance` decided on, carried in the JSON so the
/// confidence of every finding is traceable to it (#784).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AssuranceBasis {
    pub method: DetectionMethod,
    pub evidence: EvidenceQuality,
}

/// Stable identifier for a [`NormalizedFinding`](crate::audit::normalized::NormalizedFinding)'s `complexity_reason` sentence
/// shape. Together with the embedded `occurrence_count` (for the two
/// count-dependent variants) this fully reproduces the sentence in any
/// language via [`complexity_text`] — and lets a post-merge dedup pass update
/// the count without re-running the branch decision (#406).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub enum ComplexityKind {
    HighOccurrence {
        occurrence_count: usize,
    },
    TechnicalPattern,
    ModerateOccurrence {
        occurrence_count: usize,
    },
    #[default]
    LowScope,
}

impl ComplexityKind {
    /// Update the embedded occurrence count after a post-hoc finding merge
    /// (title-based dedup in the PDF builder) — keeps `complexity_reason`
    /// consistent with the merged `occurrence_count` without re-deciding
    /// which branch applies.
    pub fn with_occurrence_count(self, occurrence_count: usize) -> Self {
        match self {
            ComplexityKind::HighOccurrence { .. } => {
                ComplexityKind::HighOccurrence { occurrence_count }
            }
            ComplexityKind::ModerateOccurrence { .. } => {
                ComplexityKind::ModerateOccurrence { occurrence_count }
            }
            other => other,
        }
    }
}

/// The single source of truth for `NormalizedFinding.complexity_reason`.
///
/// Returns the sentence in German or English for the given `kind`. Analysis
/// calls it with `en = true` to bake canonical English; the PDF layer
/// re-derives in the run language (#406).
pub fn complexity_text(kind: ComplexityKind, en: bool) -> String {
    match kind {
        ComplexityKind::HighOccurrence { occurrence_count } => {
            count_text("audit-interpretation-complexity-high", occurrence_count, en)
        }
        ComplexityKind::TechnicalPattern => text("audit-interpretation-complexity-technical", en),
        ComplexityKind::ModerateOccurrence { occurrence_count } => count_text(
            "audit-interpretation-complexity-moderate",
            occurrence_count,
            en,
        ),
        ComplexityKind::LowScope => text("audit-interpretation-complexity-low", en),
    }
}

pub(super) fn derive_complexity(
    occurrence_count: usize,
    rule_id: &str,
    issue_class: &str,
) -> (String, ComplexityKind) {
    let key = format!(
        "{} {}",
        rule_id.to_ascii_lowercase(),
        issue_class.to_ascii_lowercase()
    );
    if occurrence_count >= 10 {
        (
            "high".to_string(),
            ComplexityKind::HighOccurrence { occurrence_count },
        )
    } else if key.contains("aria") || key.contains("focus") || key.contains("keyboard") {
        ("medium".to_string(), ComplexityKind::TechnicalPattern)
    } else if occurrence_count >= 5 {
        (
            "medium".to_string(),
            ComplexityKind::ModerateOccurrence { occurrence_count },
        )
    } else {
        ("low".to_string(), ComplexityKind::LowScope)
    }
}

/// Expected-score-effect classification embedded in [`ExpectedImpactKind`].
/// A severity × count heuristic, not a simulation: it orders findings and is
/// kept in the JSON for compatibility, but no report sentence states it as a
/// score effect (a simulated value is tracked in #1169).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub enum ScoreEffect {
    High,
    Medium,
    #[default]
    Low,
}

fn score_effect(severity: Severity, occurrence_count: usize) -> ScoreEffect {
    match (severity, occurrence_count) {
        (Severity::Critical | Severity::High, n) if n >= 5 => ScoreEffect::High,
        (Severity::Critical | Severity::High, _) => ScoreEffect::Medium,
        (_, n) if n >= 10 => ScoreEffect::Medium,
        _ => ScoreEffect::Low,
    }
}

/// Stable identifier for a [`NormalizedFinding`](crate::audit::normalized::NormalizedFinding)'s `expected_impact` sentence
/// shape (WCAG findings mention the criterion level; SEO/other findings
/// don't). Together with the embedded `occurrence_count`/`score_effect` this
/// fully reproduces the sentence in any language via [`expected_impact_text`]
/// — and lets a post-merge dedup pass update the count (#406).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ExpectedImpactKind {
    Wcag {
        occurrence_count: usize,
        score_effect: ScoreEffect,
        wcag_level: String,
        /// Simulated accessibility score gain of fixing this rule (#1169);
        /// `None` reads "not quantified".
        #[serde(default, skip_serializing_if = "Option::is_none")]
        score_gain: Option<ScoreGain>,
    },
    Other {
        occurrence_count: usize,
        score_effect: ScoreEffect,
    },
}

impl Default for ExpectedImpactKind {
    fn default() -> Self {
        ExpectedImpactKind::Other {
            occurrence_count: 0,
            score_effect: ScoreEffect::default(),
        }
    }
}

impl ExpectedImpactKind {
    /// The simulated accessibility score gain, where one stands behind the
    /// sentence (#1169).
    pub fn score_gain(&self) -> Option<&ScoreGain> {
        match self {
            ExpectedImpactKind::Wcag { score_gain, .. } => score_gain.as_ref(),
            ExpectedImpactKind::Other { .. } => None,
        }
    }

    /// Update the embedded occurrence count after a post-hoc finding merge
    /// (title-based dedup in the PDF builder) — keeps `expected_impact`
    /// consistent with the merged `occurrence_count`.
    pub fn with_occurrence_count(self, occurrence_count: usize) -> Self {
        match self {
            ExpectedImpactKind::Wcag {
                score_effect,
                wcag_level,
                score_gain,
                ..
            } => ExpectedImpactKind::Wcag {
                occurrence_count,
                score_effect,
                wcag_level,
                score_gain,
            },
            ExpectedImpactKind::Other { score_effect, .. } => ExpectedImpactKind::Other {
                occurrence_count,
                score_effect,
            },
        }
    }
}

/// The single source of truth for `NormalizedFinding.expected_impact`.
///
/// Returns the sentence in German or English for the given `kind`. Analysis
/// calls it with `en = true` to bake canonical English; the PDF layer
/// re-derives in the run language (#406).
pub fn expected_impact_text(kind: &ExpectedImpactKind, en: bool) -> String {
    // Names the reach and, where the scorer could be re-run without the
    // rule's findings, the simulated score gain (#1169). Whether one fix
    // removes every occurrence (shared cause unconfirmed) is not known, so
    // the gain is "up to"; `score_effect` stays in the kind for ordering.
    match kind {
        ExpectedImpactKind::Wcag {
            occurrence_count,
            wcag_level,
            score_gain,
            ..
        } => {
            let reach = text_args(
                "audit-interpretation-expected-impact-wcag",
                &[
                    ("count", FluentValue::from(*occurrence_count as i64)),
                    ("level", FluentValue::from(wcag_level.clone())),
                ],
                en,
            );
            format!("{reach} {}", score_gain_text(score_gain.as_ref(), en))
        }
        ExpectedImpactKind::Other {
            occurrence_count, ..
        } => {
            let reach = count_text(
                "audit-interpretation-expected-impact-other",
                *occurrence_count,
                en,
            );
            format!("{reach} {}", score_gain_text(None, en))
        }
    }
}

/// The score-effect sentence of an action (#1169): the simulated gain, the
/// cap that still binds after it, or "not quantified" where no simulation
/// stands behind it; the second sentence of [`expected_impact_text`].
pub(crate) fn score_gain_text(gain: Option<&ScoreGain>, en: bool) -> String {
    let Some(gain) = gain else {
        return text("audit-interpretation-score-effect-not-quantified", en);
    };
    let key = match (gain.basis, gain.accessibility, gain.capped_at) {
        (ScoreGainBasis::SimulatedMean, 0, _) => "audit-interpretation-score-gain-mean-none",
        (ScoreGainBasis::SimulatedMean, _, _) => "audit-interpretation-score-gain-mean",
        (_, 0, Some(_)) => "audit-interpretation-score-gain-none-capped",
        (_, 0, None) => "audit-interpretation-score-gain-none",
        (_, _, Some(_)) => "audit-interpretation-score-gain-capped",
        (_, _, None) => "audit-interpretation-score-gain",
    };
    text_args(
        key,
        &[
            ("points", FluentValue::from(i64::from(gain.accessibility))),
            (
                "cap",
                FluentValue::from(i64::from(gain.capped_at.unwrap_or(100))),
            ),
        ],
        en,
    )
}

pub(super) fn derive_expected_impact(
    severity: Severity,
    occurrence_count: usize,
    category: &str,
    wcag_level: &str,
) -> ExpectedImpactKind {
    let effect = score_effect(severity, occurrence_count);
    if category == "wcag" {
        ExpectedImpactKind::Wcag {
            occurrence_count,
            score_effect: effect,
            wcag_level: wcag_level.to_string(),
            score_gain: None,
        }
    } else {
        ExpectedImpactKind::Other {
            occurrence_count,
            score_effect: effect,
        }
    }
}

/// `wcag_criterion` gates against `EN301549_WEB_CLAUSES`: a criterion not in
/// that 55-entry WCAG 2.2 A/AA table (EN 301 549 V4.1.1, clauses 9.1-9.4;
/// AAA criteria and the retired 4.1.1 Parsing are not in it) is not covered by
/// the standard's normative clauses and returns "low" regardless of
/// level/severity, rather than "medium" purely from its level string matching
/// "A"/"AA".
///
/// A convention rule (`display/*`) is never more than "low" (#704).
pub(super) fn derive_bfsg_relevance(
    rule_id: &str,
    category: &str,
    wcag_criterion: &str,
    wcag_level: &str,
    severity: Severity,
) -> String {
    if category != "wcag" || crate::taxonomy::is_convention_rule(rule_id) {
        return "low".to_string();
    }
    if !crate::wcag::en301549::EN301549_WEB_CLAUSES
        .iter()
        .any(|c| c.wcag == wcag_criterion)
    {
        return "low".to_string();
    }
    match (wcag_level, severity) {
        ("A", Severity::Critical | Severity::High) => "high",
        ("A" | "AA", _) => "medium",
        _ => "low",
    }
    .to_string()
}

pub(super) fn derive_remediation_priority(
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

    /// Guard against German leaking into the canonical `complexity_reason`/
    /// `expected_impact` text baked with `en = true` (#406), and against the
    /// "Addresses 1 occurrences" singular/plural bug regressing.
    #[test]
    fn complexity_and_expected_impact_text_en_has_no_german_umlauts_and_correct_plural() {
        let has_umlaut = |s: &str| s.chars().any(|c| "äöüÄÖÜß".contains(c));

        let complexity_kinds = [
            ComplexityKind::HighOccurrence {
                occurrence_count: 12,
            },
            ComplexityKind::TechnicalPattern,
            ComplexityKind::ModerateOccurrence {
                occurrence_count: 6,
            },
            ComplexityKind::LowScope,
        ];
        for kind in complexity_kinds {
            let en_text = complexity_text(kind, true);
            assert!(
                !has_umlaut(&en_text),
                "EN complexity_reason for {kind:?} contains German umlaut: {en_text}"
            );
            let de_text = complexity_text(kind, false);
            assert_ne!(
                en_text, de_text,
                "DE/EN complexity_reason identical for {kind:?}"
            );
        }

        let impact_kinds = [
            ExpectedImpactKind::Wcag {
                occurrence_count: 1,
                score_effect: ScoreEffect::High,
                wcag_level: "A".to_string(),
                score_gain: None,
            },
            ExpectedImpactKind::Other {
                occurrence_count: 1,
                score_effect: ScoreEffect::Low,
            },
        ];
        for kind in &impact_kinds {
            let en_text = expected_impact_text(kind, true);
            assert!(
                !has_umlaut(&en_text),
                "EN expected_impact for {kind:?} contains German umlaut: {en_text}"
            );
            assert!(
                en_text.contains("1 occurrence;") || en_text.contains("1 occurrence "),
                "singular phrasing missing for {kind:?}: {en_text}"
            );
            assert!(
                !en_text.contains("1 occurrences"),
                "singular/plural bug regressed for {kind:?}: {en_text}"
            );
            let de_text = expected_impact_text(kind, false);
            assert_ne!(
                en_text, de_text,
                "DE/EN expected_impact identical for {kind:?}"
            );
        }

        // Plural still reads correctly for n > 1.
        let plural = expected_impact_text(
            &ExpectedImpactKind::Other {
                occurrence_count: 3,
                score_effect: ScoreEffect::Medium,
            },
            true,
        );
        assert!(
            plural.contains("3 occurrences;"),
            "plural phrasing missing: {plural}"
        );
    }

    /// The sentence names the reach, never a fix guarantee or a
    /// score effect derived from severity × count — even for `High`.
    #[test]
    fn expected_impact_text_claims_no_fix_or_score_effect() {
        let kind = ExpectedImpactKind::Wcag {
            occurrence_count: 20,
            score_effect: ScoreEffect::High,
            wcag_level: "A".to_string(),
            score_gain: None,
        };
        let en = expected_impact_text(&kind, true);
        let de = expected_impact_text(&kind, false);
        assert_eq!(
            en,
            "Addresses 20 occurrences; WCAG level: A. Score effect not quantified."
        );
        assert_eq!(
            de,
            "Betrifft 20 Vorkommen; WCAG-Level: A. Wirkung auf den Score nicht beziffert."
        );
        for text in [&en, &de] {
            assert!(
                !text.contains("Fixes") && !text.contains("Behebt"),
                "{text}"
            );
            assert!(!text.contains("high") && !text.contains("hoch"), "{text}");
        }
    }

    /// #1169: the score-effect sentence in every shape, EN without German
    /// letters and DE different from EN; no gain reads "not quantified".
    #[test]
    fn score_gain_text_covers_every_shape_in_both_languages() {
        use crate::audit::scoring::{ScoreGain, ScoreGainBasis};
        let gain = |accessibility, basis, capped_at| ScoreGain {
            accessibility,
            basis,
            capped_at,
        };
        let cases = [
            (
                Some(gain(7, ScoreGainBasis::Simulated, None)),
                "up to +7 points",
            ),
            (
                Some(gain(1, ScoreGainBasis::Simulated, None)),
                "up to +1 point ",
            ),
            (
                Some(gain(3, ScoreGainBasis::Simulated, Some(89))),
                "remains bounded by the cap of 89",
            ),
            (
                Some(gain(0, ScoreGainBasis::Simulated, Some(49))),
                "does not change (simulated): it remains bounded by the cap of 49",
            ),
            (
                Some(gain(0, ScoreGainBasis::Simulated, None)),
                "does not change",
            ),
            (
                Some(gain(4, ScoreGainBasis::SimulatedMean, None)),
                "on every page: up to +4 points on the average",
            ),
            (
                Some(gain(0, ScoreGainBasis::SimulatedMean, None)),
                "average accessibility score does not change",
            ),
            (None, "Score effect not quantified."),
        ];
        for (gain, expected) in cases {
            let en = score_gain_text(gain.as_ref(), true);
            let de = score_gain_text(gain.as_ref(), false);
            assert!(en.contains(expected), "{en}");
            assert!(
                !en.chars().any(|c| "äöüÄÖÜß".contains(c)),
                "EN score-effect text contains German letters: {en}"
            );
            assert_ne!(en, de);
        }
        let kind = ExpectedImpactKind::Wcag {
            occurrence_count: 2,
            score_effect: ScoreEffect::Medium,
            wcag_level: "AA".to_string(),
            score_gain: Some(gain(5, ScoreGainBasis::Simulated, None)),
        };
        assert_eq!(
            expected_impact_text(&kind, true),
            "Addresses 2 occurrences; WCAG level: AA. If only this action is done: up to +5 \
             points on the accessibility score (simulated)."
        );
        assert_eq!(
            expected_impact_text(&kind, false),
            "Betrifft 2 Vorkommen; WCAG-Level: AA. Wenn nur diese Maßnahme umgesetzt wird: bis \
             zu +5 Punkte für den Barrierefreiheits-Score (simuliert)."
        );
    }

    /// A generic link text is detected reliably, the 2.4.4 failure
    /// is not — even with a located element the finding asks for review.
    #[test]
    fn pattern_rules_are_never_automatically_confirmed() {
        let a = derive_assurance(
            wcag_detection_method("links/generic-name"),
            EvidenceQuality::Located,
        );
        assert_eq!(a.confidence, "medium");
        assert_eq!(a.verification, "manual_review_recommended");
    }

    /// #784 counter-case: the same rule id with different evidence. A
    /// located occurrence confirms, one that could not be located does not.
    #[test]
    fn same_rule_with_different_evidence_differs_in_assurance() {
        let method = wcag_detection_method("images/alt-missing");
        let located = derive_assurance(method, EvidenceQuality::Located);
        let unlocated = derive_assurance(method, EvidenceQuality::Unlocated);
        assert_eq!(located.verification, "automatically_confirmed");
        assert_eq!(located.confidence, "very_high");
        assert_eq!(unlocated.verification, "manual_review_recommended");
        assert_eq!(unlocated.confidence, "medium");
    }

    /// #784: a review outcome is never confirmed, however good the evidence.
    #[test]
    fn review_method_is_never_automatically_confirmed() {
        for evidence in [
            EvidenceQuality::Located,
            EvidenceQuality::Partial,
            EvidenceQuality::Document,
            EvidenceQuality::Unlocated,
        ] {
            assert_eq!(
                derive_assurance(DetectionMethod::Review, evidence).verification,
                "manual_review_recommended"
            );
        }
    }

    /// The old derivation matched substrings: "weak" in the id made a
    /// computed contrast failure a manual check, "content" in the label
    /// downgraded a measured SEO check. Neither name decides any more.
    #[test]
    fn names_no_longer_decide_assurance() {
        assert_eq!(
            wcag_detection_method("contrast/text-insufficient"),
            DetectionMethod::Measured
        );
        assert_eq!(
            wcag_detection_method("patterns/tooltip-unreferenced"),
            DetectionMethod::Measured
        );
    }

    #[test]
    fn evidence_quality_folds_occurrence_anchors() {
        use OccurrenceAnchor::*;
        assert_eq!(
            evidence_quality([Element, Element]),
            EvidenceQuality::Located
        );
        assert_eq!(
            evidence_quality([Element, Missing]),
            EvidenceQuality::Partial
        );
        assert_eq!(
            evidence_quality([Element, Document]),
            EvidenceQuality::Partial
        );
        assert_eq!(evidence_quality([Document]), EvidenceQuality::Document);
        assert_eq!(
            evidence_quality([Document, Missing]),
            EvidenceQuality::Unlocated
        );
        assert_eq!(evidence_quality([Missing]), EvidenceQuality::Unlocated);
    }

    /// Every declared pattern rule exists in the taxonomy.
    #[test]
    fn pattern_rules_name_taxonomy_rules() {
        for id in PATTERN_RULES {
            assert!(RuleLookup::by_id(id).is_some(), "unknown rule {id}");
        }
    }
}
