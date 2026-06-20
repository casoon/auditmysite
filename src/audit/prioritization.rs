//! Finding prioritization — domain rules that classify findings by urgency and
//! effort, independent of any output format.
//!
//! These types and mappings used to live in `output/report_model.rs` and
//! `output/builder/actions.rs`, which made the presentation layer the home of
//! product rules (how urgent is a finding? what effort does a fix take?). They
//! are pure business logic over domain types (`Severity`, score, dimension), so
//! they belong in the domain layer; the builder now only maps the results into
//! the view model. `report_model` re-exports the enums for backward compatibility.
//!
//! The `label(en)` methods follow the project's localization rule (#406): they
//! take an explicit `en` flag rather than baking the run locale, so the domain
//! stays language-neutral and the output layer chooses the language.

use crate::taxonomy::Severity;

/// Priority level for findings and actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

impl Priority {
    pub fn label(&self, en: bool) -> &'static str {
        match self {
            Priority::Critical => {
                if en {
                    "Critical"
                } else {
                    "Kritisch"
                }
            }
            Priority::High => {
                if en {
                    "High"
                } else {
                    "Hoch"
                }
            }
            Priority::Medium => {
                if en {
                    "Medium"
                } else {
                    "Mittel"
                }
            }
            Priority::Low => {
                if en {
                    "Low"
                } else {
                    "Niedrig"
                }
            }
        }
    }
}

/// Effort estimate for a fix.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effort {
    Quick,
    Medium,
    Structural,
}

impl Effort {
    pub fn label(&self, en: bool) -> &'static str {
        match self {
            Effort::Quick => {
                if en {
                    "Low complexity"
                } else {
                    "Geringe Komplexität"
                }
            }
            Effort::Medium => {
                if en {
                    "Medium effort"
                } else {
                    "Mittlerer Aufwand"
                }
            }
            Effort::Structural => {
                if en {
                    "High complexity"
                } else {
                    "Hohe Komplexität"
                }
            }
        }
    }
}

/// Semantic execution priority — when a fix should be tackled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ExecutionPriority {
    Optional,
    Important,
    Immediate,
}

impl ExecutionPriority {
    pub fn label(&self, en: bool) -> &'static str {
        match self {
            ExecutionPriority::Immediate => {
                if en {
                    "Urgent"
                } else {
                    "Dringend"
                }
            }
            ExecutionPriority::Important => {
                if en {
                    "High"
                } else {
                    "Hoch"
                }
            }
            // "Standard" is identical in both locales.
            ExecutionPriority::Optional => "Standard",
        }
    }
}

/// Map a finding severity to its headline priority.
pub fn severity_to_priority(severity: Severity) -> Priority {
    match severity {
        Severity::Critical => Priority::Critical,
        Severity::High => Priority::High,
        Severity::Medium => Priority::Medium,
        Severity::Low => Priority::Low,
    }
}

/// Map a 0–100 score to a priority band.
pub fn score_to_priority(score: f32) -> Priority {
    if score < 50.0 {
        Priority::Critical
    } else if score < 70.0 {
        Priority::High
    } else if score < 85.0 {
        Priority::Medium
    } else {
        Priority::Low
    }
}

/// Derive when a fix should be executed from its severity, effort and dimension.
/// Accessibility is treated as legally urgent (BFSG), so High-severity
/// accessibility findings are Immediate even when the fix is structural.
pub fn derive_execution_priority(
    severity: Severity,
    effort: Effort,
    dimension: &str,
) -> ExecutionPriority {
    match (severity, effort, dimension) {
        (Severity::Critical, _, _) => ExecutionPriority::Immediate,
        (Severity::High, _, "Accessibility") => ExecutionPriority::Immediate,
        (Severity::High, Effort::Quick, _) => ExecutionPriority::Important,
        (Severity::High, _, _) => ExecutionPriority::Important,
        (Severity::Medium, Effort::Quick, _) => ExecutionPriority::Important,
        _ => ExecutionPriority::Optional,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn score_bands_map_to_priority() {
        assert_eq!(score_to_priority(40.0), Priority::Critical);
        assert_eq!(score_to_priority(60.0), Priority::High);
        assert_eq!(score_to_priority(80.0), Priority::Medium);
        assert_eq!(score_to_priority(95.0), Priority::Low);
    }

    #[test]
    fn severity_maps_to_priority() {
        assert_eq!(severity_to_priority(Severity::Critical), Priority::Critical);
        assert_eq!(severity_to_priority(Severity::Low), Priority::Low);
    }

    #[test]
    fn accessibility_high_is_immediate_even_when_structural() {
        assert_eq!(
            derive_execution_priority(Severity::High, Effort::Structural, "Accessibility"),
            ExecutionPriority::Immediate
        );
        assert_eq!(
            derive_execution_priority(Severity::High, Effort::Structural, "SEO"),
            ExecutionPriority::Important
        );
        assert_eq!(
            derive_execution_priority(Severity::Low, Effort::Medium, "SEO"),
            ExecutionPriority::Optional
        );
    }
}
