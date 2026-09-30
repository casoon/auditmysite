//! Taxonomy — Standard-Report-Taxonomie für das Audit-Produkt
//!
//! Zentrales Ordnungssystem für Dimensionen, Issue-Klassen, Severity-Stufen,
//! Score-Logik und Regelobjekte. Single Source of Truth für alle Module.

pub mod criteria;
pub mod dimensions;
pub mod issue_class;
pub mod rules;
pub mod score;
pub mod score_area;
pub mod severity;

pub use criteria::{criterion_for_rule, principle_for_criterion, WcagPrinciple};
pub use dimensions::{Dimension, Subcategory};
pub use issue_class::IssueClass;
pub use rules::{ReportVisibility, Rule, RuleLookup};
pub use score::{
    module_derived_from, module_score_grade, module_weight, Scaling, ScoreImpact, MODULE_WEIGHTS,
};
pub use score_area::{score_area_for_rule, score_area_for_subcategory, ScoreArea};
pub use severity::{Severity, SeverityExt};

/// Whether a taxonomy rule id reports a control without accessible name or
/// role: `a11y.<kind>_name.missing` (interactive, command, control, input
/// field, toggle field, …) and `a11y.name_role.missing`.
///
/// The one definition behind "blocking controls", the `NamedControls`
/// source-quality signal and the lint cross-check. Criterion 4.1.2 alone is
/// too wide: it also holds duplicate IDs, prohibited ARIA attributes and
/// focusable content under `aria-hidden`.
pub fn is_missing_name_or_role(rule_id: &str) -> bool {
    rule_id == "a11y.name_role.missing"
        || (rule_id.starts_with("a11y.") && rule_id.ends_with("_name.missing"))
}

/// Best-practice rules outside the display convention that are likewise only
/// anchored to a criterion: the `title` attribute as only description (#711)
/// and a description that repeats the name (#713). Both were split off rules
/// that do test their criterion (1.4.13, 4.1.2 `aria-label`).
const BEST_PRACTICE_RULES: &[&str] = &[
    "a11y.title_only_description.weak",
    "a11y.description_duplicates_name.redundant",
];

/// Whether a taxonomy rule id checks the BarrierLab display-mode convention
/// (`display/*`, #653) or is one of the `BEST_PRACTICE_RULES`, rather than
/// a WCAG requirement. Such a rule is only anchored to a criterion, so it
/// must neither count as BFSG-relevant nor raise a legal flag (#704): the
/// page's own conformance to that criterion decides that. Scoped to these
/// rules on purpose — the other `best-practice`-tagged rules (`region`,
/// `link-as-button`, `redundant-role`, …) keep their current classification.
pub fn is_convention_rule(rule_id: &str) -> bool {
    rule_id.starts_with("a11y.display_") || BEST_PRACTICE_RULES.contains(&rule_id)
}
