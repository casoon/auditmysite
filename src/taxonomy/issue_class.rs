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

    /// Nutzerfreundlicher Label (`en = true` englisch, sonst deutsch): die
    /// `labels-issue-class-*`-Message aus `locales/{en,de}/labels.ftl`.
    pub fn label(&self, en: bool) -> String {
        let key = match self {
            IssueClass::Missing => "labels-issue-class-missing",
            IssueClass::Invalid => "labels-issue-class-invalid",
            IssueClass::Weak => "labels-issue-class-weak",
            IssueClass::Risk => "labels-issue-class-risk",
            IssueClass::Opportunity => "labels-issue-class-opportunity",
            IssueClass::Informational => "labels-issue-class-informational",
        };
        crate::i18n::text(key, en)
    }
}

impl std::fmt::Display for IssueClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label(false))
    }
}
