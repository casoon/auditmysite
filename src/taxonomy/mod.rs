//! Taxonomy — Standard-Report-Taxonomie für das Audit-Produkt
//!
//! Zentrales Ordnungssystem für Dimensionen, Issue-Klassen, Severity-Stufen,
//! Score-Logik und Regelobjekte. Single Source of Truth für alle Module.

pub mod best_practice;
pub mod criteria;
pub mod dimensions;
pub mod issue_class;
pub mod rule_docs;
pub mod rules;
pub mod score;
pub mod score_area;
pub mod severity;

pub use best_practice::{
    best_practice_decision, BestPracticeDecision, OutcomeClass, Source, BEST_PRACTICE_DECISIONS,
};
pub use criteria::{criterion_for_rule, principle_for_criterion, WcagPrinciple};
pub use dimensions::{Dimension, Subcategory};
pub use issue_class::IssueClass;
pub use rule_docs::{is_addressable_rule_id, rule_doc_url, RULE_DOCS_BASE_URL};
pub use rules::{ReportVisibility, Rule, RuleLookup};
pub use score::{
    criteria_with_impact, criterion_impact, module_derived_from, module_score_grade, module_weight,
    Scaling, ScoreImpact, MODULE_WEIGHTS,
};
pub use score_area::{score_area_for_rule, score_area_for_subcategory, ScoreArea};
pub use severity::{Severity, SeverityExt};

/// Engine rules that report a control without accessible name or role.
///
/// `summary/name-missing` and `tables/name-missing` are on the list because
/// the `a11y.<kind>_name.missing` ids they carried before the two rule
/// catalogues were unified matched the former pattern definition. Criterion
/// 4.1.2 alone is too wide: it also holds duplicate IDs, prohibited ARIA
/// attributes and focusable content under `aria-hidden`.
const MISSING_NAME_OR_ROLE_RULES: &[&str] = &[
    "names/required-missing",
    "names/symbol-only",
    "buttons/name-missing",
    "links/name-missing",
    "summary/name-missing",
    "tables/name-missing",
];

/// Whether a rule id reports a control without accessible name or role.
///
/// The one definition behind "blocking controls", the `NamedControls`
/// source-quality signal and the lint cross-check.
pub fn is_missing_name_or_role(rule_id: &str) -> bool {
    MISSING_NAME_OR_ROLE_RULES.contains(&rule_id)
}

/// Whether a rule id is a convention rule: a best-practice rule whose decision
/// row (`BEST_PRACTICE_DECISIONS`, #919) marks it so (the BarrierLab
/// display-mode convention `display/*`, `viz/*` (#653) and the rules split off
/// a criterion they do not test: #711, #713, `zoom/viewport-scale-limited`).
/// Such a rule is only anchored to a criterion, so it must neither count as
/// BFSG-relevant nor raise a legal flag (#704): the page's own conformance to
/// that criterion decides that. The other best-practice rules (`region`,
/// `redundant-role`, ...) keep their current classification.
pub fn is_convention_rule(rule_id: &str) -> bool {
    best_practice_decision(rule_id).is_some_and(|d| d.convention)
}
