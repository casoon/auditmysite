//! Severity-System
//!
//! 4 produktorientierte Schweregrade, einheitlich über alle Module.
//! Ersetzt die bisherigen verschiedenen Severity-Systeme
//! (WCAG Minor/Moderate/Serious/Critical, Security-Strings, Mobile-Strings).

use serde::{Deserialize, Serialize};

/// 4 Severity-Stufen (aufsteigend von niedrigster zu höchster)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Niedrige Priorität, verbesserungswürdig
    Low,
    /// Relevantes, aber nicht existenzielles Problem
    Medium,
    /// Klares Problem mit spürbarer Auswirkung
    High,
    /// Schwerwiegendes Problem, hohe Auswirkung oder hohes Risiko
    Critical,
}

impl Severity {
    /// Nutzerfreundlicher Label (deutsch)
    pub fn label(&self) -> &'static str {
        match self {
            Severity::Critical => "Kritisch",
            Severity::High => "Hoch",
            Severity::Medium => "Mittel",
            Severity::Low => "Niedrig",
        }
    }

    /// Englischer Label für Reports
    pub fn label_en(&self) -> &'static str {
        match self {
            Severity::Critical => "CRITICAL",
            Severity::High => "HIGH",
            Severity::Medium => "MEDIUM",
            Severity::Low => "LOW",
        }
    }

    /// Konvertierung vom alten WCAG-Severity-System
    pub fn from_legacy_wcag(old: &str) -> Self {
        match old {
            "critical" => Severity::Critical,
            "serious" => Severity::High,
            "moderate" => Severity::Medium,
            "minor" => Severity::Low,
            _ => Severity::Medium,
        }
    }

    /// Konvertierung von Security-Modul-Strings
    pub fn from_legacy_security(old: &str) -> Self {
        match old {
            "critical" => Severity::Critical,
            "high" => Severity::High,
            "medium" => Severity::Medium,
            "low" => Severity::Low,
            _ => Severity::Medium,
        }
    }

    /// Konvertierung von Mobile/SEO-Modul-Strings
    pub fn from_legacy_module(old: &str) -> Self {
        match old {
            "error" => Severity::High,
            "warning" => Severity::Medium,
            "info" => Severity::Low,
            _ => Severity::Medium,
        }
    }
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Critical => write!(f, "critical"),
            Severity::High => write!(f, "high"),
            Severity::Medium => write!(f, "medium"),
            Severity::Low => write!(f, "low"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Critical > Severity::High);
        assert!(Severity::High > Severity::Medium);
        assert!(Severity::Medium > Severity::Low);
    }

    #[test]
    fn test_legacy_wcag_mapping() {
        assert_eq!(Severity::from_legacy_wcag("critical"), Severity::Critical);
        assert_eq!(Severity::from_legacy_wcag("serious"), Severity::High);
        assert_eq!(Severity::from_legacy_wcag("moderate"), Severity::Medium);
        assert_eq!(Severity::from_legacy_wcag("minor"), Severity::Low);
    }

    #[test]
    fn test_legacy_module_mapping() {
        assert_eq!(Severity::from_legacy_module("error"), Severity::High);
        assert_eq!(Severity::from_legacy_module("warning"), Severity::Medium);
        assert_eq!(Severity::from_legacy_module("info"), Severity::Low);
    }
}
