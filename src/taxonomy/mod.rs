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
