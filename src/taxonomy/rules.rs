//! Regelregister — zentrale Definition aller Audit-Regeln über alle Dimensionen
//!
//! Jede Regel im System hat eine eindeutige ID, gehört zu einer Dimension
//! und Subkategorie, hat Issue-Klasse, Severity, Score-Impact und
//! Report-Sichtbarkeitseinstellungen.

use super::{
    dimensions::{Dimension, Subcategory},
    issue_class::IssueClass,
    score::{Scaling, ScoreImpact},
    severity::Severity,
};

/// Standard-Regelobjekt
pub struct Rule {
    /// Eindeutige Regel-ID (z.B. "a11y.alt_text.missing")
    pub id: &'static str,
    /// Audit-Dimension
    pub dimension: Dimension,
    /// Subkategorie innerhalb der Dimension
    pub subcategory: Subcategory,
    /// Issue-Klasse (Missing/Invalid/Weak/Risk/Opportunity/Informational)
    pub issue_class: IssueClass,
    /// Schweregrad
    pub severity: Severity,
    /// Externe Referenz (z.B. "WCAG 1.1.1", "CWV:LCP")
    pub external_ref: Option<&'static str>,
    /// Externes Level (z.B. "A", "AA", "AAA")
    pub external_level: Option<&'static str>,
    /// Kurztitel
    pub title: &'static str,
    /// Beschreibung des Problems
    pub description: &'static str,
    /// Auswirkung auf den Nutzer
    pub user_impact: &'static str,
    /// Technische Auswirkung
    pub technical_impact: &'static str,
    /// Score-Abzug-Modell
    pub score_impact: ScoreImpact,
    /// Sichtbarkeit in den verschiedenen Report-Ebenen
    pub report_visibility: ReportVisibility,
}

/// Welche Report-Ebenen zeigen diese Regel
pub struct ReportVisibility {
    pub executive: bool,
    pub standard: bool,
    pub technical: bool,
}

const VIS_ALL: ReportVisibility = ReportVisibility {
    executive: true,
    standard: true,
    technical: true,
};

const VIS_STANDARD: ReportVisibility = ReportVisibility {
    executive: false,
    standard: true,
    technical: true,
};

const VIS_TECHNICAL: ReportVisibility = ReportVisibility {
    executive: false,
    standard: false,
    technical: true,
};

/// Lookup-Trait für Regelregister
pub struct RuleLookup;

impl RuleLookup {
    /// Regel nach ID suchen
    pub fn by_id(id: &str) -> Option<&'static Rule> {
        RULES.iter().find(|r| r.id == id)
    }

    /// Regel nach externer WCAG-Referenz suchen (z.B. "1.1.1")
    pub fn by_wcag(criterion: &str) -> Option<&'static Rule> {
        let search = format!("WCAG {}", criterion);
        RULES
            .iter()
            .find(|r| r.external_ref == Some(search.as_str()))
    }

    /// Regel nach Legacy-WCAG-ID suchen (z.B. "1.1.1")
    pub fn by_legacy_wcag_id(wcag_id: &str) -> Option<&'static Rule> {
        LEGACY_WCAG_MAP
            .iter()
            .find(|(wid, _)| *wid == wcag_id)
            .and_then(|(_, rule_id)| Self::by_id(rule_id))
    }

    /// Alle Regeln einer Dimension
    pub fn by_dimension(dim: Dimension) -> Vec<&'static Rule> {
        RULES.iter().filter(|r| r.dimension == dim).collect()
    }
}

/// Mapping von alter WCAG-ID zu neuer Regel-ID
static LEGACY_WCAG_MAP: &[(&str, &str)] = &[
    ("1.1.1", "a11y.alt_text.missing"),
    ("1.3.1", "a11y.structure.missing"),
    ("1.3.5", "a11y.input_purpose.missing"),
    ("1.4.3", "a11y.contrast.weak"),
    ("1.4.4", "a11y.resize_text.weak"),
    ("1.4.11", "a11y.non_text_contrast.weak"),
    ("2.1.1", "a11y.keyboard.missing"),
    ("2.1.2", "a11y.keyboard_trap.risk"),
    ("2.4.1", "a11y.bypass_blocks.missing"),
    ("2.4.2", "a11y.page_title.missing"),
    ("2.4.3", "a11y.focus_order.weak"),
    ("2.4.4", "a11y.link_purpose.weak"),
    ("2.4.6", "a11y.headings.missing"),
    ("2.4.7", "a11y.focus_visible.missing"),
    ("2.4.10", "a11y.section_headings.missing"),
    ("2.5.3", "a11y.label_in_name.invalid"),
    ("3.1.1", "a11y.language.missing"),
    ("3.2.1", "a11y.on_focus.risk"),
    ("3.2.2", "a11y.on_input.risk"),
    ("3.3.2", "a11y.form_labels.missing"),
    ("4.1.2", "a11y.name_role.missing"),
];

/// Alle Regeln im System
pub static RULES: &[Rule] = &[
    // ═══════════════════════════════════════════════════════════════════════════
    // ACCESSIBILITY
    // ═══════════════════════════════════════════════════════════════════════════

    // ── Inhalte & Alternativen ──────────────────────────────────────────────
    Rule {
        id: "a11y.alt_text.missing",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::ContentAlternatives,
        issue_class: IssueClass::Missing,
        severity: Severity::High,
        external_ref: Some("WCAG 1.1.1"),
        external_level: Some("A"),
        title: "Fehlende Alternativtexte bei Bildern",
        description: "Bilder haben keinen beschreibenden Alternativtext.",
        user_impact: "Screenreader-Nutzer erhalten keine Bildinformation.",
        technical_impact: "Nicht-konforme Bildauszeichnung im Markup.",
        score_impact: ScoreImpact {
            base_penalty: 3.0,
            max_penalty: 10.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_ALL,
    },
    // ── Struktur & Semantik ─────────────────────────────────────────────────
    Rule {
        id: "a11y.structure.missing",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::StructureSemantics,
        issue_class: IssueClass::Missing,
        severity: Severity::High,
        external_ref: Some("WCAG 1.3.1"),
        external_level: Some("A"),
        title: "Fehlende semantische Struktur",
        description: "Struktur ist visuell, aber nicht im HTML-Code hinterlegt.",
        user_impact: "Screenreader können Beziehungen zwischen Inhalten nicht erkennen.",
        technical_impact: "Fehlende HTML5-Semantik und ARIA-Rollen.",
        score_impact: ScoreImpact {
            base_penalty: 2.5,
            max_penalty: 8.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_ALL,
    },
    Rule {
        id: "a11y.headings.missing",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::StructureSemantics,
        issue_class: IssueClass::Missing,
        severity: Severity::High,
        external_ref: Some("WCAG 2.4.6"),
        external_level: Some("AA"),
        title: "Fehlende oder unzureichende Überschriftenstruktur",
        description: "Keine oder unlogische Überschriftenhierarchie.",
        user_impact: "Screenreader-Navigation per Überschrift nicht möglich.",
        technical_impact: "Fehlende Heading-Hierarchie im DOM.",
        score_impact: ScoreImpact {
            base_penalty: 20.0,
            max_penalty: 20.0,
            occurrence_scaling: Scaling::Fixed,
        },
        report_visibility: VIS_ALL,
    },
    Rule {
        id: "a11y.section_headings.missing",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::StructureSemantics,
        issue_class: IssueClass::Missing,
        severity: Severity::Low,
        external_ref: Some("WCAG 2.4.10"),
        external_level: Some("AAA"),
        title: "Fehlende Abschnittsüberschriften",
        description: "Längere Inhalte sind nicht durch Abschnittsüberschriften gegliedert.",
        user_impact: "Nutzer können Inhalte nicht gezielt ansteuern.",
        technical_impact: "Fehlende Sub-Headings in langen Inhaltsblöcken.",
        score_impact: ScoreImpact {
            base_penalty: 1.0,
            max_penalty: 3.0,
            occurrence_scaling: Scaling::Fixed,
        },
        report_visibility: VIS_STANDARD,
    },
    // ── Navigation & Bedienung ──────────────────────────────────────────────
    Rule {
        id: "a11y.keyboard.missing",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::NavigationInteraction,
        issue_class: IssueClass::Missing,
        severity: Severity::Critical,
        external_ref: Some("WCAG 2.1.1"),
        external_level: Some("A"),
        title: "Inhalte nicht per Tastatur bedienbar",
        description: "Funktionen nur mit Maus bedienbar.",
        user_impact: "Tastaturnutzer sind von Funktionen ausgeschlossen.",
        technical_impact: "Click-Handler ohne Tastatur-Alternative.",
        score_impact: ScoreImpact {
            base_penalty: 5.0,
            max_penalty: 15.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_ALL,
    },
    Rule {
        id: "a11y.keyboard_trap.risk",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::NavigationInteraction,
        issue_class: IssueClass::Risk,
        severity: Severity::Critical,
        external_ref: Some("WCAG 2.1.2"),
        external_level: Some("A"),
        title: "Tastaturfalle — Fokus kann nicht verlassen werden",
        description: "Tastaturfokus bleibt an bestimmten Stellen hängen.",
        user_impact: "Seite wird faktisch unbenutzbar für Tastaturnutzer.",
        technical_impact: "Fehlende Fokus-Verwaltung in modalen Elementen.",
        score_impact: ScoreImpact {
            base_penalty: 10.0,
            max_penalty: 15.0,
            occurrence_scaling: Scaling::Fixed,
        },
        report_visibility: VIS_ALL,
    },
    Rule {
        id: "a11y.bypass_blocks.missing",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::NavigationInteraction,
        issue_class: IssueClass::Missing,
        severity: Severity::Medium,
        external_ref: Some("WCAG 2.4.1"),
        external_level: Some("A"),
        title: "Fehlende Sprungnavigation",
        description: "Kein Skip-Link oder Mechanismus zum Überspringen wiederkehrender Blöcke.",
        user_impact: "Tastaturnutzer müssen bei jedem Seitenwechsel durch die Navigation tabben.",
        technical_impact: "Fehlender Skip-Link, fehlende HTML5-Landmarks.",
        score_impact: ScoreImpact {
            base_penalty: 2.0,
            max_penalty: 5.0,
            occurrence_scaling: Scaling::Fixed,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "a11y.page_title.missing",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::NavigationInteraction,
        issue_class: IssueClass::Missing,
        severity: Severity::High,
        external_ref: Some("WCAG 2.4.2"),
        external_level: Some("A"),
        title: "Fehlender oder unzureichender Seitentitel",
        description: "Seite hat keinen aussagekräftigen Title-Tag.",
        user_impact: "Screenreader-Nutzer können die Seite nicht identifizieren.",
        technical_impact: "Leeres oder generisches <title>-Tag.",
        score_impact: ScoreImpact {
            base_penalty: 5.0,
            max_penalty: 5.0,
            occurrence_scaling: Scaling::Fixed,
        },
        report_visibility: VIS_ALL,
    },
    Rule {
        id: "a11y.focus_order.weak",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::NavigationInteraction,
        issue_class: IssueClass::Weak,
        severity: Severity::High,
        external_ref: Some("WCAG 2.4.3"),
        external_level: Some("A"),
        title: "Unlogische Fokus-Reihenfolge",
        description: "Tab-Reihenfolge entspricht nicht der visuellen Reihenfolge.",
        user_impact: "Verwirrende Navigation für Tastaturnutzer.",
        technical_impact: "DOM-Reihenfolge weicht von visueller Reihenfolge ab.",
        score_impact: ScoreImpact {
            base_penalty: 2.5,
            max_penalty: 8.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "a11y.link_purpose.weak",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::NavigationInteraction,
        issue_class: IssueClass::Weak,
        severity: Severity::Medium,
        external_ref: Some("WCAG 2.4.4"),
        external_level: Some("A"),
        title: "Unklare oder generische Linktexte",
        description: "Links mit Texten wie 'hier', 'mehr', 'weiterlesen'.",
        user_impact: "Screenreader-Linklisten zeigen nichtssagende Texte.",
        technical_impact: "Generische Link-Texte ohne Kontext.",
        score_impact: ScoreImpact {
            base_penalty: 1.0,
            max_penalty: 5.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "a11y.focus_visible.missing",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::NavigationInteraction,
        issue_class: IssueClass::Missing,
        severity: Severity::High,
        external_ref: Some("WCAG 2.4.7"),
        external_level: Some("AA"),
        title: "Fokus-Indikator nicht sichtbar",
        description: "Kein visueller Fokus-Rahmen bei Tastaturnavigation.",
        user_impact: "Tastaturnutzer sehen nicht, welches Element fokussiert ist.",
        technical_impact: "CSS outline:none ohne Alternative.",
        score_impact: ScoreImpact {
            base_penalty: 3.0,
            max_penalty: 8.0,
            occurrence_scaling: Scaling::Fixed,
        },
        report_visibility: VIS_ALL,
    },
    Rule {
        id: "a11y.label_in_name.invalid",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::NavigationInteraction,
        issue_class: IssueClass::Invalid,
        severity: Severity::High,
        external_ref: Some("WCAG 2.5.3"),
        external_level: Some("A"),
        title: "Sichtbarer Text ≠ zugänglicher Name",
        description: "Sichtbarer Text und aria-label stimmen nicht überein.",
        user_impact: "Sprachsteuerungsnutzer können Element nicht ansprechen.",
        technical_impact: "aria-label weicht vom sichtbaren Text ab.",
        score_impact: ScoreImpact {
            base_penalty: 2.5,
            max_penalty: 8.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    // ── Formulare & Interaktion ─────────────────────────────────────────────
    Rule {
        id: "a11y.input_purpose.missing",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::FormsInteraction,
        issue_class: IssueClass::Missing,
        severity: Severity::Medium,
        external_ref: Some("WCAG 1.3.5"),
        external_level: Some("AA"),
        title: "Fehlende Eingabezweck-Kennzeichnung",
        description: "Formularfelder ohne autocomplete-Attribut.",
        user_impact: "Kein Autofill möglich, Eingabe fehleranfälliger.",
        technical_impact: "Fehlende autocomplete-Attribute.",
        score_impact: ScoreImpact {
            base_penalty: 1.0,
            max_penalty: 5.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "a11y.form_labels.missing",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::FormsInteraction,
        issue_class: IssueClass::Missing,
        severity: Severity::High,
        external_ref: Some("WCAG 3.3.2"),
        external_level: Some("A"),
        title: "Fehlende Formularbeschriftungen",
        description: "Formularfelder ohne sichtbare Labels.",
        user_impact: "Nutzer wissen nicht, welche Eingaben erwartet werden.",
        technical_impact: "Fehlende <label>-Elemente.",
        score_impact: ScoreImpact {
            base_penalty: 2.5,
            max_penalty: 8.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_ALL,
    },
    // ── Sprache & Verständlichkeit ──────────────────────────────────────────
    Rule {
        id: "a11y.language.missing",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::LanguageClarity,
        issue_class: IssueClass::Missing,
        severity: Severity::High,
        external_ref: Some("WCAG 3.1.1"),
        external_level: Some("A"),
        title: "Fehlende Sprachangabe",
        description: "Kein lang-Attribut im HTML-Tag.",
        user_impact: "Screenreader liest mit falscher Aussprache vor.",
        technical_impact: "Fehlendes lang-Attribut.",
        score_impact: ScoreImpact {
            base_penalty: 10.0,
            max_penalty: 12.5,
            occurrence_scaling: Scaling::Fixed,
        },
        report_visibility: VIS_ALL,
    },
    Rule {
        id: "a11y.on_focus.risk",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::LanguageClarity,
        issue_class: IssueClass::Risk,
        severity: Severity::High,
        external_ref: Some("WCAG 3.2.1"),
        external_level: Some("A"),
        title: "Unerwartete Kontextänderung bei Fokus",
        description: "Fokuserhalt löst unerwartete Kontextänderung aus.",
        user_impact: "Verwirrende Seitenänderungen für Tastaturnutzer.",
        technical_impact: "onfocus-Handler lösen Navigation aus.",
        score_impact: ScoreImpact {
            base_penalty: 3.0,
            max_penalty: 8.0,
            occurrence_scaling: Scaling::Fixed,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "a11y.on_input.risk",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::LanguageClarity,
        issue_class: IssueClass::Risk,
        severity: Severity::Medium,
        external_ref: Some("WCAG 3.2.2"),
        external_level: Some("A"),
        title: "Unerwartete Kontextänderung bei Eingabe",
        description: "Formularänderung löst unangekündigte Kontextänderung aus.",
        user_impact: "Verwirrende Änderungen beim Ausfüllen.",
        technical_impact: "Auto-submit bei onchange.",
        score_impact: ScoreImpact {
            base_penalty: 2.0,
            max_penalty: 5.0,
            occurrence_scaling: Scaling::Fixed,
        },
        report_visibility: VIS_STANDARD,
    },
    // ── Visuelle Darstellung / Kontrast ─────────────────────────────────────
    Rule {
        id: "a11y.contrast.weak",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::VisualPresentation,
        issue_class: IssueClass::Weak,
        severity: Severity::High,
        external_ref: Some("WCAG 1.4.3"),
        external_level: Some("AA"),
        title: "Unzureichender Farbkontrast",
        description: "Text hat nicht genügend Kontrast zum Hintergrund.",
        user_impact: "Text bei Sehschwäche oder ungünstigen Lichtverhältnissen schwer lesbar.",
        technical_impact: "Kontrastverhältnis unter 4.5:1 für normalen Text.",
        score_impact: ScoreImpact {
            base_penalty: 5.0,
            max_penalty: 12.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_ALL,
    },
    Rule {
        id: "a11y.resize_text.weak",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::VisualPresentation,
        issue_class: IssueClass::Weak,
        severity: Severity::High,
        external_ref: Some("WCAG 1.4.4"),
        external_level: Some("AA"),
        title: "Text nicht ausreichend skalierbar",
        description: "Texte können nicht auf 200% vergrößert werden.",
        user_impact: "Nutzer mit Sehschwäche verlieren Inhalte bei Vergrößerung.",
        technical_impact: "Feste Pixelwerte, overflow:hidden.",
        score_impact: ScoreImpact {
            base_penalty: 2.5,
            max_penalty: 8.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "a11y.non_text_contrast.weak",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::VisualPresentation,
        issue_class: IssueClass::Weak,
        severity: Severity::Medium,
        external_ref: Some("WCAG 1.4.11"),
        external_level: Some("AA"),
        title: "Unzureichender Kontrast bei UI-Elementen",
        description: "Bedienelemente haben nicht genügend Kontrast.",
        user_impact: "Interaktive Elemente schwer erkennbar.",
        technical_impact: "Kontrastverhältnis UI-Elemente unter 3:1.",
        score_impact: ScoreImpact {
            base_penalty: 1.5,
            max_penalty: 5.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    // ── Technische Robustheit ───────────────────────────────────────────────
    Rule {
        id: "a11y.name_role.missing",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::TechnicalRobustness,
        issue_class: IssueClass::Missing,
        severity: Severity::High,
        external_ref: Some("WCAG 4.1.2"),
        external_level: Some("A"),
        title: "Fehlende Name/Rolle bei Bedienelementen",
        description: "Interaktive Elemente ohne zugänglichen Namen oder erkennbare Rolle.",
        user_impact: "Screenreader kann Funktion nicht vermitteln.",
        technical_impact: "Fehlende ARIA-Labels und -Rollen.",
        score_impact: ScoreImpact {
            base_penalty: 5.0,
            max_penalty: 12.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_ALL,
    },
    // ═══════════════════════════════════════════════════════════════════════════
    // PERFORMANCE
    // ═══════════════════════════════════════════════════════════════════════════
    Rule {
        id: "perf.lcp.slow",
        dimension: Dimension::Performance,
        subcategory: Subcategory::LoadBehavior,
        issue_class: IssueClass::Weak,
        severity: Severity::High,
        external_ref: Some("CWV:LCP"),
        external_level: None,
        title: "Langsamer Largest Contentful Paint",
        description: "Hauptinhalt wird zu spät sichtbar.",
        user_impact: "Nutzer sieht lange leere oder unvollständige Seite.",
        technical_impact: "LCP über 2500ms — Netzwerk, Rendering oder Ressourcen zu langsam.",
        score_impact: ScoreImpact {
            base_penalty: 25.0,
            max_penalty: 25.0,
            occurrence_scaling: Scaling::Fixed,
        },
        report_visibility: VIS_ALL,
    },
    Rule {
        id: "perf.fcp.slow",
        dimension: Dimension::Performance,
        subcategory: Subcategory::LoadBehavior,
        issue_class: IssueClass::Weak,
        severity: Severity::Medium,
        external_ref: Some("CWV:FCP"),
        external_level: None,
        title: "Langsamer First Contentful Paint",
        description: "Erster sichtbarer Inhalt erscheint zu spät.",
        user_impact: "Nutzer wartet zu lange auf erste Rückmeldung.",
        technical_impact: "FCP über 1800ms — Server oder Rendering-Blockade.",
        score_impact: ScoreImpact {
            base_penalty: 25.0,
            max_penalty: 25.0,
            occurrence_scaling: Scaling::Fixed,
        },
        report_visibility: VIS_ALL,
    },
    Rule {
        id: "perf.cls.unstable",
        dimension: Dimension::Performance,
        subcategory: Subcategory::VisualStability,
        issue_class: IssueClass::Weak,
        severity: Severity::Medium,
        external_ref: Some("CWV:CLS"),
        external_level: None,
        title: "Layout-Verschiebungen während des Ladens",
        description: "Seitenelemente springen während des Ladens.",
        user_impact: "Nutzer klickt versehentlich auf falsches Element.",
        technical_impact: "CLS über 0.1 — fehlende Dimensionen, nachladende Inhalte.",
        score_impact: ScoreImpact {
            base_penalty: 25.0,
            max_penalty: 25.0,
            occurrence_scaling: Scaling::Fixed,
        },
        report_visibility: VIS_ALL,
    },
    Rule {
        id: "perf.tbt.high",
        dimension: Dimension::Performance,
        subcategory: Subcategory::Interactivity,
        issue_class: IssueClass::Weak,
        severity: Severity::High,
        external_ref: Some("CWV:TBT"),
        external_level: None,
        title: "Hohe Blocking-Zeit",
        description: "Hauptthread blockiert zu lange durch JavaScript.",
        user_impact: "Seite reagiert verzögert auf Nutzereingaben.",
        technical_impact: "TBT/INP über 200ms — zu viel JavaScript-Arbeit im Hauptthread.",
        score_impact: ScoreImpact {
            base_penalty: 25.0,
            max_penalty: 25.0,
            occurrence_scaling: Scaling::Fixed,
        },
        report_visibility: VIS_ALL,
    },
    Rule {
        id: "perf.ttfb.slow",
        dimension: Dimension::Performance,
        subcategory: Subcategory::LoadBehavior,
        issue_class: IssueClass::Risk,
        severity: Severity::Medium,
        external_ref: Some("CWV:TTFB"),
        external_level: None,
        title: "Langsame Server-Antwortzeit",
        description: "Server reagiert zu langsam auf die Anfrage.",
        user_impact: "Gesamte Ladezeit wird verzögert.",
        technical_impact: "TTFB über 800ms — Server-Performance oder Netzwerk-Latenz.",
        score_impact: ScoreImpact {
            base_penalty: 5.0,
            max_penalty: 10.0,
            occurrence_scaling: Scaling::Fixed,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "perf.resources.heavy",
        dimension: Dimension::Performance,
        subcategory: Subcategory::ResourceUsage,
        issue_class: IssueClass::Risk,
        severity: Severity::Medium,
        external_ref: None,
        external_level: None,
        title: "Hoher Ressourcenverbrauch",
        description: "Seite lädt zu viele oder zu große Ressourcen.",
        user_impact: "Langsame Ladezeiten, hoher Datenverbrauch auf Mobilgeräten.",
        technical_impact: "Gesamtgröße > 3MB, zu viele Requests, unkomprimiert.",
        score_impact: ScoreImpact {
            base_penalty: 5.0,
            max_penalty: 10.0,
            occurrence_scaling: Scaling::Fixed,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "perf.dom.large",
        dimension: Dimension::Performance,
        subcategory: Subcategory::TechnicalComplexity,
        issue_class: IssueClass::Risk,
        severity: Severity::Low,
        external_ref: None,
        external_level: None,
        title: "Großes DOM",
        description: "Übermäßig viele DOM-Elemente.",
        user_impact: "Seite reagiert langsam auf Interaktionen.",
        technical_impact: "DOM-Größe belastet Browser-Rendering und Speicher.",
        score_impact: ScoreImpact {
            base_penalty: 3.0,
            max_penalty: 5.0,
            occurrence_scaling: Scaling::Fixed,
        },
        report_visibility: VIS_TECHNICAL,
    },
    // ═══════════════════════════════════════════════════════════════════════════
    // SEO
    // ═══════════════════════════════════════════════════════════════════════════
    Rule {
        id: "seo.title.missing",
        dimension: Dimension::Seo,
        subcategory: Subcategory::SnippetMetadata,
        issue_class: IssueClass::Missing,
        severity: Severity::Critical,
        external_ref: None,
        external_level: None,
        title: "Fehlender Seitentitel",
        description: "Kein <title>-Tag vorhanden.",
        user_impact: "Seite wird in Suchergebnissen ohne aussagekräftigen Titel angezeigt.",
        technical_impact: "Leeres oder fehlendes <title>-Element.",
        score_impact: ScoreImpact {
            base_penalty: 10.0,
            max_penalty: 10.0,
            occurrence_scaling: Scaling::Fixed,
        },
        report_visibility: VIS_ALL,
    },
    Rule {
        id: "seo.title.weak",
        dimension: Dimension::Seo,
        subcategory: Subcategory::SnippetMetadata,
        issue_class: IssueClass::Weak,
        severity: Severity::Medium,
        external_ref: None,
        external_level: None,
        title: "Ungünstige Titellänge",
        description: "Titel zu kurz (<30 Zeichen) oder zu lang (>60 Zeichen).",
        user_impact: "Snippet wird in Suchergebnissen suboptimal dargestellt.",
        technical_impact: "Title-Tag außerhalb des empfohlenen Zeichenbereichs.",
        score_impact: ScoreImpact {
            base_penalty: 5.0,
            max_penalty: 5.0,
            occurrence_scaling: Scaling::Fixed,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "seo.description.missing",
        dimension: Dimension::Seo,
        subcategory: Subcategory::SnippetMetadata,
        issue_class: IssueClass::Missing,
        severity: Severity::High,
        external_ref: None,
        external_level: None,
        title: "Fehlende Meta-Description",
        description: "Keine Meta-Description vorhanden.",
        user_impact: "Suchmaschine generiert eigene Beschreibung, oft suboptimal.",
        technical_impact: "Fehlendes <meta name=\"description\">-Element.",
        score_impact: ScoreImpact {
            base_penalty: 10.0,
            max_penalty: 10.0,
            occurrence_scaling: Scaling::Fixed,
        },
        report_visibility: VIS_ALL,
    },
    Rule {
        id: "seo.description.weak",
        dimension: Dimension::Seo,
        subcategory: Subcategory::SnippetMetadata,
        issue_class: IssueClass::Weak,
        severity: Severity::Low,
        external_ref: None,
        external_level: None,
        title: "Ungünstige Description-Länge",
        description: "Meta-Description zu kurz (<120) oder zu lang (>160 Zeichen).",
        user_impact: "Snippet wird abgeschnitten oder ist zu knapp.",
        technical_impact: "Description außerhalb des empfohlenen Zeichenbereichs.",
        score_impact: ScoreImpact {
            base_penalty: 5.0,
            max_penalty: 5.0,
            occurrence_scaling: Scaling::Fixed,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "seo.viewport.missing",
        dimension: Dimension::Seo,
        subcategory: Subcategory::SnippetMetadata,
        issue_class: IssueClass::Missing,
        severity: Severity::High,
        external_ref: None,
        external_level: None,
        title: "Fehlende Viewport-Meta-Tag",
        description: "Kein Viewport-Meta-Tag vorhanden.",
        user_impact: "Seite wird auf Mobilgeräten nicht korrekt dargestellt.",
        technical_impact: "Fehlendes <meta name=\"viewport\">-Element.",
        score_impact: ScoreImpact {
            base_penalty: 10.0,
            max_penalty: 10.0,
            occurrence_scaling: Scaling::Fixed,
        },
        report_visibility: VIS_ALL,
    },
    Rule {
        id: "seo.h1.missing",
        dimension: Dimension::Seo,
        subcategory: Subcategory::ContentStructure,
        issue_class: IssueClass::Missing,
        severity: Severity::High,
        external_ref: None,
        external_level: None,
        title: "Fehlende H1-Überschrift",
        description: "Seite hat keine H1-Überschrift.",
        user_impact: "Suchmaschine kann Hauptthema der Seite schlechter erkennen.",
        technical_impact: "Kein <h1>-Element vorhanden.",
        score_impact: ScoreImpact {
            base_penalty: 15.0,
            max_penalty: 15.0,
            occurrence_scaling: Scaling::Fixed,
        },
        report_visibility: VIS_ALL,
    },
    Rule {
        id: "seo.h1.multiple",
        dimension: Dimension::Seo,
        subcategory: Subcategory::ContentStructure,
        issue_class: IssueClass::Weak,
        severity: Severity::Low,
        external_ref: None,
        external_level: None,
        title: "Mehrere H1-Überschriften",
        description: "Seite hat mehr als eine H1-Überschrift.",
        user_impact: "Thematische Fokussierung der Seite unklar.",
        technical_impact: "Mehrere <h1>-Elemente im DOM.",
        score_impact: ScoreImpact {
            base_penalty: 5.0,
            max_penalty: 5.0,
            occurrence_scaling: Scaling::Fixed,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "seo.headings.issue",
        dimension: Dimension::Seo,
        subcategory: Subcategory::ContentStructure,
        issue_class: IssueClass::Weak,
        severity: Severity::Low,
        external_ref: None,
        external_level: None,
        title: "Heading-Probleme",
        description: "Übersprungene Ebenen, leere Headings oder zu lange Headings.",
        user_impact: "Strukturelle Signale für Suchmaschine geschwächt.",
        technical_impact: "Heading-Hierarchie nicht korrekt.",
        score_impact: ScoreImpact {
            base_penalty: 3.0,
            max_penalty: 9.0,
            occurrence_scaling: Scaling::Linear,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "seo.canonical.missing",
        dimension: Dimension::Seo,
        subcategory: Subcategory::Indexability,
        issue_class: IssueClass::Opportunity,
        severity: Severity::Low,
        external_ref: None,
        external_level: None,
        title: "Fehlende Canonical-URL",
        description: "Kein Canonical-Link vorhanden.",
        user_impact: "Mögliche Duplicate-Content-Probleme.",
        technical_impact: "Fehlendes <link rel=\"canonical\">-Element.",
        score_impact: ScoreImpact {
            base_penalty: 5.0,
            max_penalty: 5.0,
            occurrence_scaling: Scaling::Fixed,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "seo.lang.missing",
        dimension: Dimension::Seo,
        subcategory: Subcategory::SemanticSignals,
        issue_class: IssueClass::Missing,
        severity: Severity::Medium,
        external_ref: None,
        external_level: None,
        title: "Fehlende Sprachangabe",
        description: "Kein lang-Attribut im HTML-Tag.",
        user_impact: "Suchmaschine kann Sprache nicht automatisch zuordnen.",
        technical_impact: "Fehlendes lang-Attribut im <html>-Element.",
        score_impact: ScoreImpact {
            base_penalty: 3.0,
            max_penalty: 3.0,
            occurrence_scaling: Scaling::Fixed,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "seo.og.missing",
        dimension: Dimension::Seo,
        subcategory: Subcategory::SemanticSignals,
        issue_class: IssueClass::Opportunity,
        severity: Severity::Low,
        external_ref: None,
        external_level: None,
        title: "Fehlende Open Graph Tags",
        description: "Keine Open Graph Meta-Tags vorhanden.",
        user_impact: "Social-Media-Vorschau der Seite ist unformatiert.",
        technical_impact: "Fehlende og:-Meta-Tags.",
        score_impact: ScoreImpact {
            base_penalty: 5.0,
            max_penalty: 5.0,
            occurrence_scaling: Scaling::Fixed,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "seo.twitter.missing",
        dimension: Dimension::Seo,
        subcategory: Subcategory::SemanticSignals,
        issue_class: IssueClass::Opportunity,
        severity: Severity::Low,
        external_ref: None,
        external_level: None,
        title: "Fehlende Twitter Card Tags",
        description: "Keine Twitter Card Meta-Tags vorhanden.",
        user_impact: "Twitter-Vorschau der Seite ist unformatiert.",
        technical_impact: "Fehlende twitter:-Meta-Tags.",
        score_impact: ScoreImpact {
            base_penalty: 5.0,
            max_penalty: 5.0,
            occurrence_scaling: Scaling::Fixed,
        },
        report_visibility: VIS_TECHNICAL,
    },
    Rule {
        id: "seo.https.missing",
        dimension: Dimension::Seo,
        subcategory: Subcategory::Indexability,
        issue_class: IssueClass::Missing,
        severity: Severity::Critical,
        external_ref: None,
        external_level: None,
        title: "Kein HTTPS",
        description: "Seite wird nicht über HTTPS ausgeliefert.",
        user_impact: "Browser zeigt Sicherheitswarnung, Ranking-Nachteil.",
        technical_impact: "Fehlende TLS-Verschlüsselung.",
        score_impact: ScoreImpact {
            base_penalty: 10.0,
            max_penalty: 10.0,
            occurrence_scaling: Scaling::Fixed,
        },
        report_visibility: VIS_ALL,
    },
    // ═══════════════════════════════════════════════════════════════════════════
    // SECURITY
    // ═══════════════════════════════════════════════════════════════════════════
    Rule {
        id: "sec.https.missing",
        dimension: Dimension::Security,
        subcategory: Subcategory::Transport,
        issue_class: IssueClass::Missing,
        severity: Severity::Critical,
        external_ref: None,
        external_level: None,
        title: "Kein HTTPS",
        description: "Seite wird nicht über HTTPS ausgeliefert.",
        user_impact: "Daten werden unverschlüsselt übertragen.",
        technical_impact: "Fehlende TLS-Verschlüsselung.",
        score_impact: ScoreImpact {
            base_penalty: 25.0,
            max_penalty: 25.0,
            occurrence_scaling: Scaling::Fixed,
        },
        report_visibility: VIS_ALL,
    },
    Rule {
        id: "sec.hsts.missing",
        dimension: Dimension::Security,
        subcategory: Subcategory::Headers,
        issue_class: IssueClass::Missing,
        severity: Severity::High,
        external_ref: None,
        external_level: None,
        title: "HSTS Header fehlt",
        description: "Strict-Transport-Security Header nicht gesetzt.",
        user_impact: "Downgrade-Angriffe auf HTTP möglich.",
        technical_impact: "Fehlender HSTS-Header.",
        score_impact: ScoreImpact {
            base_penalty: 15.0,
            max_penalty: 15.0,
            occurrence_scaling: Scaling::Fixed,
        },
        report_visibility: VIS_ALL,
    },
    Rule {
        id: "sec.csp.missing",
        dimension: Dimension::Security,
        subcategory: Subcategory::Headers,
        issue_class: IssueClass::Missing,
        severity: Severity::High,
        external_ref: None,
        external_level: None,
        title: "Content Security Policy fehlt",
        description: "Kein Content-Security-Policy Header gesetzt.",
        user_impact: "Erhöhtes Risiko für XSS-Angriffe.",
        technical_impact: "Fehlender CSP-Header.",
        score_impact: ScoreImpact {
            base_penalty: 15.0,
            max_penalty: 15.0,
            occurrence_scaling: Scaling::Fixed,
        },
        report_visibility: VIS_ALL,
    },
    Rule {
        id: "sec.xcto.missing",
        dimension: Dimension::Security,
        subcategory: Subcategory::BrowserProtection,
        issue_class: IssueClass::Missing,
        severity: Severity::Medium,
        external_ref: None,
        external_level: None,
        title: "X-Content-Type-Options Header fehlt",
        description: "Browser kann MIME-Typ-Sniffing durchführen.",
        user_impact: "Potenzielle Sicherheitslücke durch falsche MIME-Typen.",
        technical_impact: "Fehlender X-Content-Type-Options: nosniff Header.",
        score_impact: ScoreImpact {
            base_penalty: 10.0,
            max_penalty: 10.0,
            occurrence_scaling: Scaling::Fixed,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "sec.xfo.missing",
        dimension: Dimension::Security,
        subcategory: Subcategory::BrowserProtection,
        issue_class: IssueClass::Missing,
        severity: Severity::Medium,
        external_ref: None,
        external_level: None,
        title: "X-Frame-Options Header fehlt",
        description: "Seite kann in fremden iframes eingebettet werden.",
        user_impact: "Clickjacking-Angriffe möglich.",
        technical_impact: "Fehlender X-Frame-Options Header.",
        score_impact: ScoreImpact {
            base_penalty: 10.0,
            max_penalty: 10.0,
            occurrence_scaling: Scaling::Fixed,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "sec.referrer.missing",
        dimension: Dimension::Security,
        subcategory: Subcategory::BrowserProtection,
        issue_class: IssueClass::Missing,
        severity: Severity::Low,
        external_ref: None,
        external_level: None,
        title: "Referrer-Policy Header fehlt",
        description: "Keine Kontrolle über Referrer-Informationen.",
        user_impact: "Private URLs können an Dritte weitergegeben werden.",
        technical_impact: "Fehlender Referrer-Policy Header.",
        score_impact: ScoreImpact {
            base_penalty: 5.0,
            max_penalty: 5.0,
            occurrence_scaling: Scaling::Fixed,
        },
        report_visibility: VIS_TECHNICAL,
    },
    // ═══════════════════════════════════════════════════════════════════════════
    // MOBILE
    // ═══════════════════════════════════════════════════════════════════════════
    Rule {
        id: "mob.viewport.missing",
        dimension: Dimension::Mobile,
        subcategory: Subcategory::Viewport,
        issue_class: IssueClass::Missing,
        severity: Severity::Critical,
        external_ref: None,
        external_level: None,
        title: "Fehlende Viewport-Konfiguration",
        description: "Kein Viewport-Meta-Tag vorhanden.",
        user_impact: "Seite wird auf Mobilgeräten nicht korrekt dargestellt.",
        technical_impact: "Fehlendes <meta name=\"viewport\">-Element.",
        score_impact: ScoreImpact {
            base_penalty: 20.0,
            max_penalty: 20.0,
            occurrence_scaling: Scaling::Fixed,
        },
        report_visibility: VIS_ALL,
    },
    Rule {
        id: "mob.viewport.improper",
        dimension: Dimension::Mobile,
        subcategory: Subcategory::Viewport,
        issue_class: IssueClass::Weak,
        severity: Severity::Medium,
        external_ref: None,
        external_level: None,
        title: "Unvollständige Viewport-Konfiguration",
        description: "Viewport-Tag fehlt width=device-width oder initial-scale=1.",
        user_impact: "Seite wird möglicherweise nicht optimal skaliert.",
        technical_impact: "Viewport-Meta-Tag mit unvollständiger Konfiguration.",
        score_impact: ScoreImpact {
            base_penalty: 10.0,
            max_penalty: 10.0,
            occurrence_scaling: Scaling::Fixed,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "mob.zoom.disabled",
        dimension: Dimension::Mobile,
        subcategory: Subcategory::Viewport,
        issue_class: IssueClass::Risk,
        severity: Severity::Critical,
        external_ref: None,
        external_level: None,
        title: "Zoom deaktiviert",
        description: "user-scalable=no verhindert das Zoomen.",
        user_impact: "Nutzer mit Sehschwäche können Inhalte nicht vergrößern.",
        technical_impact: "Viewport-Meta mit user-scalable=no.",
        score_impact: ScoreImpact {
            base_penalty: 20.0,
            max_penalty: 20.0,
            occurrence_scaling: Scaling::Fixed,
        },
        report_visibility: VIS_ALL,
    },
    Rule {
        id: "mob.touch_targets.small",
        dimension: Dimension::Mobile,
        subcategory: Subcategory::TouchUsability,
        issue_class: IssueClass::Weak,
        severity: Severity::Medium,
        external_ref: None,
        external_level: None,
        title: "Zu kleine Touch-Targets",
        description: "Interaktive Elemente kleiner als 44x44 Pixel.",
        user_impact: "Schwer bedienbar auf Touchscreens.",
        technical_impact: "Bounding Box unter 44x44px.",
        score_impact: ScoreImpact {
            base_penalty: 10.0,
            max_penalty: 10.0,
            occurrence_scaling: Scaling::Fixed,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "mob.fonts.small",
        dimension: Dimension::Mobile,
        subcategory: Subcategory::Readability,
        issue_class: IssueClass::Weak,
        severity: Severity::Medium,
        external_ref: None,
        external_level: None,
        title: "Zu kleine Schriftgröße",
        description: "Kleinste Schriftgröße unter 12px.",
        user_impact: "Text auf Mobilgeräten schwer lesbar.",
        technical_impact: "Font-size unter 12px detektiert.",
        score_impact: ScoreImpact {
            base_penalty: 10.0,
            max_penalty: 10.0,
            occurrence_scaling: Scaling::Fixed,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "mob.horizontal_scroll",
        dimension: Dimension::Mobile,
        subcategory: Subcategory::ResponsiveLayout,
        issue_class: IssueClass::Weak,
        severity: Severity::High,
        external_ref: None,
        external_level: None,
        title: "Horizontales Scrollen erforderlich",
        description: "Seite ist breiter als der Viewport.",
        user_impact: "Nutzer muss horizontal scrollen, Inhalte abgeschnitten.",
        technical_impact: "scrollWidth > innerWidth.",
        score_impact: ScoreImpact {
            base_penalty: 20.0,
            max_penalty: 20.0,
            occurrence_scaling: Scaling::Fixed,
        },
        report_visibility: VIS_ALL,
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_lookup_by_id() {
        let rule = RuleLookup::by_id("a11y.alt_text.missing");
        assert!(rule.is_some());
        let rule = rule.unwrap();
        assert_eq!(rule.dimension, Dimension::Accessibility);
        assert_eq!(rule.severity, Severity::High);
    }

    #[test]
    fn test_legacy_wcag_lookup() {
        let rule = RuleLookup::by_legacy_wcag_id("1.1.1");
        assert!(rule.is_some());
        assert_eq!(rule.unwrap().id, "a11y.alt_text.missing");
    }

    #[test]
    fn test_all_rules_have_dimension() {
        for rule in RULES {
            assert!(
                rule.subcategory.dimension() == rule.dimension,
                "Rule {} has subcategory {:?} which belongs to {:?}, not {:?}",
                rule.id,
                rule.subcategory,
                rule.subcategory.dimension(),
                rule.dimension
            );
        }
    }

    #[test]
    fn test_all_legacy_wcag_ids_resolve() {
        for (wcag_id, rule_id) in LEGACY_WCAG_MAP {
            assert!(
                RuleLookup::by_id(rule_id).is_some(),
                "Legacy WCAG ID {} maps to rule {} which doesn't exist",
                wcag_id,
                rule_id
            );
        }
    }

    #[test]
    fn test_unique_rule_ids() {
        let mut ids: Vec<&str> = RULES.iter().map(|r| r.id).collect();
        ids.sort();
        let len_before = ids.len();
        ids.dedup();
        assert_eq!(len_before, ids.len(), "Duplicate rule IDs found");
    }
}
