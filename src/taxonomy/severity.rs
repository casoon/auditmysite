//! Severity-System
//!
//! 4 produktorientierte Schweregrade, einheitlich über alle Module.
//!
//! Die Stufen kommen aus dem geteilten `a11y-report`-Crate, damit derselbe
//! Befund auf allen drei Oberflächen — astro-post-audit, auditmysite,
//! LiveAudit — dieselbe Schwere trägt. Varianten, Reihenfolge und
//! JSON-Darstellung (`"low"`/`"medium"`/`"high"`/`"critical"`) sind identisch
//! zur vorherigen lokalen Definition; die Umstellung ändert die Ausgabe nicht.
//!
//! Die auditmysite-eigenen Zusätze — deutsche Labels und die Umsetzer aus den
//! Alt-Systemen (WCAG Minor/Moderate/Serious/Critical, Security-Strings,
//! Mobile-Strings) — hängen als [`SeverityExt`] daran, weil an einem fremden
//! Typ keine inhärenten Methoden ergänzt werden können.

pub use a11y_report::Severity;

/// Die auditmysite-eigenen Ergänzungen zu [`Severity`].
///
/// Muss im Modul sichtbar sein, damit `severity.label()` bzw.
/// `Severity::from_legacy_wcag(..)` aufgelöst werden — deshalb re-exportiert
/// `crate::taxonomy` das Trait mit.
pub trait SeverityExt {
    /// Nutzerfreundliches Label (deutsch)
    fn label(&self) -> &'static str;

    /// Englisches Label für Reports
    fn label_en(&self) -> &'static str;

    /// Konvertierung vom alten WCAG-Severity-System
    fn from_legacy_wcag(old: &str) -> Severity;

    /// Konvertierung von Security-Modul-Strings
    fn from_legacy_security(old: &str) -> Severity;

    /// Konvertierung von Mobile/SEO-Modul-Strings
    fn from_legacy_module(old: &str) -> Severity;
}

impl SeverityExt for Severity {
    fn label(&self) -> &'static str {
        match self {
            Severity::Critical => "Kritisch",
            Severity::High => "Hoch",
            Severity::Medium => "Mittel",
            Severity::Low => "Niedrig",
        }
    }

    fn label_en(&self) -> &'static str {
        match self {
            Severity::Critical => "CRITICAL",
            Severity::High => "HIGH",
            Severity::Medium => "MEDIUM",
            Severity::Low => "LOW",
        }
    }

    fn from_legacy_wcag(old: &str) -> Severity {
        match old {
            "critical" => Severity::Critical,
            "serious" => Severity::High,
            "moderate" => Severity::Medium,
            "minor" => Severity::Low,
            _ => Severity::Medium,
        }
    }

    fn from_legacy_security(old: &str) -> Severity {
        match old {
            "critical" => Severity::Critical,
            "high" => Severity::High,
            "medium" => Severity::Medium,
            "low" => Severity::Low,
            _ => Severity::Medium,
        }
    }

    fn from_legacy_module(old: &str) -> Severity {
        match old {
            "error" => Severity::High,
            "warning" => Severity::Medium,
            "info" => Severity::Low,
            _ => Severity::Medium,
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

    /// Die JSON-Darstellung muss der vorherigen lokalen Definition entsprechen —
    /// sonst wäre die Umstellung auf das geteilte Crate eine stille
    /// Formatänderung.
    #[test]
    fn serialisiert_wie_zuvor() {
        for (sev, erwartet) in [
            (Severity::Low, "\"low\""),
            (Severity::Medium, "\"medium\""),
            (Severity::High, "\"high\""),
            (Severity::Critical, "\"critical\""),
        ] {
            assert_eq!(serde_json::to_string(&sev).unwrap(), erwartet);
        }
    }

    /// `as_str` aus dem geteilten Crate liefert genau das, was vorher das
    /// lokale `Display` schrieb.
    #[test]
    fn as_str_entspricht_altem_display() {
        assert_eq!(Severity::Critical.as_str(), "critical");
        assert_eq!(Severity::High.as_str(), "high");
        assert_eq!(Severity::Medium.as_str(), "medium");
        assert_eq!(Severity::Low.as_str(), "low");
    }
}
