//! Taxonomy — Standard-Report-Taxonomie für das Audit-Produkt
//!
//! Zentrales Ordnungssystem für Dimensionen, Issue-Klassen, Severity-Stufen,
//! Score-Logik und Regelobjekte. Single Source of Truth für alle Module.

pub mod dimensions;
pub mod issue_class;
pub mod rules;
pub mod score;
pub mod severity;

pub use dimensions::{Dimension, Subcategory};
pub use issue_class::IssueClass;
pub use rules::{ReportVisibility, Rule, RuleLookup};
pub use score::{ScoreImpact, Scaling};
pub use severity::Severity;
