//! Derived finding attributes: priority score, confidence, false-positive
//! risk, verification, complexity/expected-impact kinds and their #406 en/de
//! texts, BFSG relevance, remediation priority and per-rule effort weight.
//!
//! Moved out of `audit::normalized`; every public item stays reachable at its
//! old `auditmysite::audit::normalized::…` path via `pub use`.

use serde::{Deserialize, Serialize};

use crate::taxonomy::{RuleLookup, Severity};

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

pub(super) fn derive_confidence(rule_id: &str, subcategory: &str, issue_class: &str) -> String {
    let key = format!(
        "{} {} {}",
        rule_id.to_ascii_lowercase(),
        subcategory.to_ascii_lowercase(),
        issue_class.to_ascii_lowercase()
    );
    if key.contains("alt_text.weak")
        || key.contains("understand")
        || key.contains("readability")
        || subcategory.eq_ignore_ascii_case("content")
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

pub(super) fn derive_false_positive_risk(
    rule_id: &str,
    subcategory: &str,
    issue_class: &str,
) -> String {
    let key = format!(
        "{} {} {}",
        rule_id.to_ascii_lowercase(),
        subcategory.to_ascii_lowercase(),
        issue_class.to_ascii_lowercase()
    );
    if key.contains("weak")
        || key.contains("alt_text.weak")
        || key.contains("understand")
        || subcategory.eq_ignore_ascii_case("content")
    {
        "medium".to_string()
    } else if key.contains("aria") || key.contains("heading") || key.contains("landmark") {
        "low".to_string()
    } else {
        "very_low".to_string()
    }
}

pub(super) fn derive_verification(false_positive_risk: &str) -> String {
    match false_positive_risk {
        "medium" | "high" => "manual_review_recommended",
        _ => "automatically_confirmed",
    }
    .to_string()
}

/// Stable identifier for a [`NormalizedFinding`](crate::audit::normalized::NormalizedFinding)'s `complexity_reason` sentence
/// shape. Together with the embedded `occurrence_count` (for the two
/// count-dependent variants) this fully reproduces the sentence in any
/// language via [`complexity_text`] — and lets a post-merge dedup pass update
/// the count without re-running the branch decision (#406).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
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
            if en {
                format!(
                    "{} occurrence{} indicate a component- or template-level issue.",
                    occurrence_count,
                    if occurrence_count == 1 { "" } else { "s" }
                )
            } else {
                format!(
                    "{} Vorkommen deuten auf ein Komponenten- oder Template-Problem hin.",
                    occurrence_count
                )
            }
        }
        ComplexityKind::TechnicalPattern => {
            if en {
                "The fix is technical but affects a limited number of patterns.".to_string()
            } else {
                "Die Behebung ist technisch, betrifft aber nur wenige Muster.".to_string()
            }
        }
        ComplexityKind::ModerateOccurrence { occurrence_count } => {
            if en {
                format!(
                    "{} occurrence{} require consistent updates across content or templates.",
                    occurrence_count,
                    if occurrence_count == 1 { "" } else { "s" }
                )
            } else {
                format!(
                    "{} Vorkommen erfordern einheitliche Anpassungen in Inhalten oder Templates.",
                    occurrence_count
                )
            }
        }
        ComplexityKind::LowScope => {
            if en {
                "Few occurrences and a clearly scoped fix.".to_string()
            } else {
                "Wenige Vorkommen und ein klar abgegrenzter Fix.".to_string()
            }
        }
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
/// Same three-tier decision `derive_expected_impact` always used — kept as an
/// enum rather than a raw string so bake time and post-merge recompute apply
/// the identical label mapping in either language.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ScoreEffect {
    High,
    Medium,
    #[default]
    Low,
}

impl ScoreEffect {
    fn label(self, en: bool) -> &'static str {
        match (self, en) {
            (ScoreEffect::High, true) => "high",
            (ScoreEffect::High, false) => "hoch",
            (ScoreEffect::Medium, true) => "medium",
            (ScoreEffect::Medium, false) => "mittel",
            (ScoreEffect::Low, true) => "low",
            (ScoreEffect::Low, false) => "niedrig",
        }
    }
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
pub enum ExpectedImpactKind {
    Wcag {
        occurrence_count: usize,
        score_effect: ScoreEffect,
        wcag_level: String,
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
    /// Update the embedded occurrence count after a post-hoc finding merge
    /// (title-based dedup in the PDF builder) — keeps `expected_impact`
    /// consistent with the merged `occurrence_count`.
    pub fn with_occurrence_count(self, occurrence_count: usize) -> Self {
        match self {
            ExpectedImpactKind::Wcag {
                score_effect,
                wcag_level,
                ..
            } => ExpectedImpactKind::Wcag {
                occurrence_count,
                score_effect,
                wcag_level,
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
    match kind {
        ExpectedImpactKind::Wcag {
            occurrence_count,
            score_effect,
            wcag_level,
        } => {
            let n = *occurrence_count;
            if en {
                format!(
                    "Fixes {} occurrence{}; expected score impact: {}; WCAG level: {}.",
                    n,
                    if n == 1 { "" } else { "s" },
                    score_effect.label(true),
                    wcag_level
                )
            } else {
                format!(
                    "Behebt {} Vorkommen; erwartete Auswirkung auf den Score: {}; WCAG-Level: {}.",
                    n,
                    score_effect.label(false),
                    wcag_level
                )
            }
        }
        ExpectedImpactKind::Other {
            occurrence_count,
            score_effect,
        } => {
            let n = *occurrence_count;
            if en {
                format!(
                    "Fixes {} occurrence{}; expected visibility/structure impact: {}.",
                    n,
                    if n == 1 { "" } else { "s" },
                    score_effect.label(true)
                )
            } else {
                format!(
                    "Behebt {} Vorkommen; erwartete Auswirkung auf Sichtbarkeit/Struktur: {}.",
                    n,
                    score_effect.label(false)
                )
            }
        }
    }
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
        }
    } else {
        ExpectedImpactKind::Other {
            occurrence_count,
            score_effect: effect,
        }
    }
}

/// `wcag_criterion` gates against `EN301549_WEB_CLAUSES`: a criterion not in
/// that 50-entry WCAG 2.1 A/AA table (AAA criteria, WCAG-2.2-only criteria
/// such as 2.5.8) is not covered by EN 301 549 V3.2.1 and returns "low"
/// regardless of level/severity, rather than "medium" purely from its level
/// string matching "A"/"AA".
pub(super) fn derive_bfsg_relevance(
    category: &str,
    wcag_criterion: &str,
    wcag_level: &str,
    severity: Severity,
) -> String {
    if category != "wcag" {
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
    /// "Fixes 1 occurrences" singular/plural bug regressing.
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

    // Regression: the taxonomy rule id "a11y.hover.content_visibility" (the
    // content-on-hover/focus check, WCAG 1.4.13) merely contains the
    // substring "content" — a deterministic, DOM-observable interaction check
    // must not be downgraded to the subjective "content quality" confidence
    // bucket just because of that substring collision. Subcategory/issue_class
    // are passed as the German labels actually used at the call site
    // (`subcategory_de`/`issue_class_de`), not their English names.
    #[test]
    fn derive_confidence_does_not_downgrade_content_on_hover_rule() {
        assert_eq!(
            derive_confidence(
                "a11y.hover.content_visibility",
                "Visuelle Darstellung",
                "Schwach"
            ),
            "very_high"
        );
    }

    #[test]
    fn derive_false_positive_risk_does_not_flag_content_on_hover_rule() {
        assert_eq!(
            derive_false_positive_risk(
                "a11y.hover.content_visibility",
                "Visuelle Darstellung",
                "Schwach"
            ),
            "very_low"
        );
    }

    #[test]
    fn derive_confidence_still_downgrades_seo_content_subcategory() {
        assert_eq!(
            derive_confidence("seo.headings.missing_h1", "Content", "issue"),
            "medium"
        );
        assert_eq!(
            derive_false_positive_risk("seo.headings.missing_h1", "Content", "issue"),
            "medium"
        );
    }
}
