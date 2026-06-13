//! Standard-Issue-Klassen
//!
//! Jeder Fund wird einer Issue-Klasse zugeordnet, die beschreibt
//! *was* das Problem ist — unabhängig vom Schweregrad.

use serde::{Deserialize, Serialize};

/// 6 Standard-Issue-Klassen
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IssueClass {
    /// Etwas fehlt, das vorhanden sein sollte
    /// z.B. fehlender Alt-Text, fehlendes lang-Attribut, fehlender HSTS Header
    #[default]
    Missing,

    /// Vorhanden, aber falsch oder ungültig
    /// z.B. ungültiger Sprachcode, defekter Canonical, fehlerhafte strukturierte Daten
    Invalid,

    /// Formal vorhanden, aber qualitativ zu schwach
    /// z.B. Meta Description zu lang, Touch Targets zu klein, unklare Linktexte
    Weak,

    /// Kein direkter Fehler, aber ein relevantes Risiko
    /// z.B. sehr großes DOM, fehlende CSP, hoher JS-Heap
    Risk,

    /// Verbesserungspotenzial ohne akuten Mangel
    /// z.B. strukturierte Daten ergänzen, interne Verlinkung ausbauen
    Opportunity,

    /// Nur Hinweis / Kontext, kein Problem
    /// z.B. H1-Anzahl, Wortanzahl, Open Graph vorhanden
    Informational,
}

impl IssueClass {
    /// Ob diese Issue-Klasse den Score beeinflusst
    pub fn affects_score(&self) -> bool {
        !matches!(self, IssueClass::Opportunity | IssueClass::Informational)
    }

    /// Nutzerfreundlicher Label (`en = true` englisch, sonst deutsch).
    pub fn label(&self, en: bool) -> &'static str {
        if en {
            match self {
                IssueClass::Missing => "Missing",
                IssueClass::Invalid => "Invalid",
                IssueClass::Weak => "Weak",
                IssueClass::Risk => "Risk",
                IssueClass::Opportunity => "Opportunity",
                IssueClass::Informational => "Informational",
            }
        } else {
            match self {
                IssueClass::Missing => "Fehlend",
                IssueClass::Invalid => "Ungültig",
                IssueClass::Weak => "Schwach",
                IssueClass::Risk => "Risiko",
                IssueClass::Opportunity => "Optimierungspotenzial",
                IssueClass::Informational => "Hinweis",
            }
        }
    }
}

impl std::fmt::Display for IssueClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label(false))
    }
}
