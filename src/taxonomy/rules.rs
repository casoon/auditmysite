//! Regelregister — zentrale Definition aller Audit-Regeln über alle Dimensionen
//!
//! Jede Regel im System hat eine eindeutige ID, gehört zu einer Dimension
//! und Subkategorie, hat Issue-Klasse, Severity, Score-Impact und
//! Report-Sichtbarkeitseinstellungen.
//!
//! The texts of a rule (title, user impact, technical impact) are not in this
//! table: they live in `locales/<lang>/rules.ftl` under the key
//! [`rule_text_key`] and are read through [`Rule::title`],
//! [`Rule::user_impact`] and [`Rule::technical_impact`] (#935).

use super::{
    dimensions::{Dimension, Subcategory},
    issue_class::IssueClass,
    score::{Scaling, ScoreImpact},
    severity::Severity,
};

/// Standard-Regelobjekt
pub struct Rule {
    /// Eindeutige Regel-ID: die Kennung, die die Engine auf der `Violation`
    /// trägt (z.B. "images/alt-missing")
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
    /// Primary axe-core rule ID, if applicable
    pub axe_id: Option<&'static str>,
    /// Score-Abzug-Modell
    pub score_impact: ScoreImpact,
    /// Sichtbarkeit in den verschiedenen Report-Ebenen
    pub report_visibility: ReportVisibility,
}

/// The Fluent message of a rule's texts in `locales/<lang>/rules.ftl`:
/// `rule-` plus the rule id, with `/` written as `-` (a Fluent identifier
/// allows only letters, digits, `_` and `-`).
pub fn rule_text_key(id: &str) -> String {
    format!("rule-{}", id.replace('/', "-"))
}

impl Rule {
    /// Short title (`en = true` canonical English, as in the JSON; otherwise
    /// German).
    pub fn title(&self, en: bool) -> String {
        crate::i18n::text(&rule_text_key(self.id), en)
    }

    /// Effect on the user (`en = true` canonical English, otherwise German).
    pub fn user_impact(&self, en: bool) -> String {
        crate::i18n::attr_text(&rule_text_key(self.id), "user-impact", en)
    }

    /// Technical effect (`en = true` canonical English, otherwise German).
    pub fn technical_impact(&self, en: bool) -> String {
        crate::i18n::attr_text(&rule_text_key(self.id), "technical-impact", en)
    }
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

    /// Alle Regeln einer Dimension
    pub fn by_dimension(dim: Dimension) -> Vec<&'static Rule> {
        RULES.iter().filter(|r| r.dimension == dim).collect()
    }
}

/// Alle Regeln im System, je Engine-Kennung (`Violation::rule_id`) ein Eintrag.
pub static RULES: &[Rule] = &[
    // ── Inhalte & Alternativen ────────────────────────────────────────────
    Rule {
        id: "image-alt",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::ContentAlternatives,
        issue_class: IssueClass::Missing,
        severity: Severity::High,
        external_ref: Some("WCAG 1.1.1"),
        external_level: Some("A"),
        axe_id: Some("image-alt"),
        score_impact: ScoreImpact {
            base_penalty: 3.0,
            max_penalty: 10.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_ALL,
    },
    Rule {
        id: "images/alt-missing",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::ContentAlternatives,
        issue_class: IssueClass::Missing,
        severity: Severity::High,
        external_ref: Some("WCAG 1.1.1"),
        external_level: Some("A"),
        axe_id: Some("image-alt"),
        score_impact: ScoreImpact {
            base_penalty: 3.0,
            max_penalty: 10.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_ALL,
    },
    Rule {
        id: "images/alt-suspicious",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::ContentAlternatives,
        issue_class: IssueClass::Weak,
        severity: Severity::Medium,
        external_ref: Some("WCAG 1.1.1"),
        external_level: Some("A"),
        axe_id: Some("images/alt-suspicious"),
        score_impact: ScoreImpact {
            base_penalty: 1.0,
            max_penalty: 4.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "video-caption",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::ContentAlternatives,
        issue_class: IssueClass::Missing,
        severity: Severity::High,
        external_ref: Some("WCAG 1.2.2"),
        external_level: Some("A"),
        axe_id: Some("video-caption"),
        score_impact: ScoreImpact {
            base_penalty: 3.0,
            max_penalty: 10.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_ALL,
    },
    Rule {
        id: "media-alt",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::ContentAlternatives,
        issue_class: IssueClass::Missing,
        severity: Severity::Medium,
        external_ref: Some("WCAG 1.2.8"),
        external_level: Some("AAA"),
        axe_id: Some("media-alt"),
        score_impact: ScoreImpact {
            base_penalty: 0.5,
            max_penalty: 1.5,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    // ── Struktur & Semantik ───────────────────────────────────────────────
    Rule {
        id: "order/visual-mismatch",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::StructureSemantics,
        issue_class: IssueClass::Weak,
        severity: Severity::Medium,
        external_ref: Some("WCAG 1.3.2"),
        external_level: Some("A"),
        axe_id: Some("css-order-reading-sequence"),
        score_impact: ScoreImpact {
            base_penalty: 1.0,
            max_penalty: 3.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "headings/section-without-heading",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::StructureSemantics,
        issue_class: IssueClass::Missing,
        severity: Severity::Low,
        external_ref: Some("WCAG 2.4.10"),
        external_level: Some("AAA"),
        axe_id: Some("heading-order"),
        score_impact: ScoreImpact {
            base_penalty: 1.0,
            max_penalty: 3.0,
            occurrence_scaling: Scaling::Fixed,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "orientation/root-rotated",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::StructureSemantics,
        issue_class: IssueClass::Weak,
        severity: Severity::High,
        external_ref: Some("WCAG 1.3.4"),
        external_level: Some("AA"),
        axe_id: Some("css-orientation-lock"),
        score_impact: ScoreImpact {
            base_penalty: 3.0,
            max_penalty: 10.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_ALL,
    },
    Rule {
        id: "orientation/content-hidden",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::StructureSemantics,
        issue_class: IssueClass::Weak,
        severity: Severity::High,
        external_ref: Some("WCAG 1.3.4"),
        external_level: Some("AA"),
        axe_id: Some("css-orientation-lock"),
        score_impact: ScoreImpact {
            base_penalty: 3.0,
            max_penalty: 10.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_ALL,
    },
    // ── Navigation & Bedienung ────────────────────────────────────────────
    Rule {
        id: "keyboard/interactive-not-focusable",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::NavigationInteraction,
        issue_class: IssueClass::Missing,
        severity: Severity::Critical,
        external_ref: Some("WCAG 2.1.1"),
        external_level: Some("A"),
        axe_id: Some("keyboard"),
        score_impact: ScoreImpact {
            base_penalty: 5.0,
            max_penalty: 15.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_ALL,
    },
    Rule {
        id: "keyboard-trap",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::NavigationInteraction,
        issue_class: IssueClass::Risk,
        severity: Severity::Critical,
        external_ref: Some("WCAG 2.1.2"),
        external_level: Some("A"),
        axe_id: Some("keyboard-trap"),
        score_impact: ScoreImpact {
            base_penalty: 10.0,
            max_penalty: 15.0,
            occurrence_scaling: Scaling::Fixed,
        },
        report_visibility: VIS_ALL,
    },
    Rule {
        id: "headings/none",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::NavigationInteraction,
        issue_class: IssueClass::Missing,
        severity: Severity::High,
        external_ref: Some("WCAG 2.4.1"),
        external_level: Some("A"),
        axe_id: Some("bypass"),
        score_impact: ScoreImpact {
            base_penalty: 3.0,
            max_penalty: 8.0,
            occurrence_scaling: Scaling::Fixed,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "document-title",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::NavigationInteraction,
        issue_class: IssueClass::Missing,
        severity: Severity::High,
        external_ref: Some("WCAG 2.4.2"),
        external_level: Some("A"),
        axe_id: Some("document-title"),
        score_impact: ScoreImpact {
            base_penalty: 5.0,
            max_penalty: 5.0,
            occurrence_scaling: Scaling::Fixed,
        },
        report_visibility: VIS_ALL,
    },
    Rule {
        id: "document/title-empty",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::NavigationInteraction,
        issue_class: IssueClass::Missing,
        severity: Severity::High,
        external_ref: Some("WCAG 2.4.2"),
        external_level: Some("A"),
        axe_id: Some("document-title"),
        score_impact: ScoreImpact {
            base_penalty: 5.0,
            max_penalty: 5.0,
            occurrence_scaling: Scaling::Fixed,
        },
        report_visibility: VIS_ALL,
    },
    Rule {
        id: "document/title-missing",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::NavigationInteraction,
        issue_class: IssueClass::Missing,
        severity: Severity::High,
        external_ref: Some("WCAG 2.4.2"),
        external_level: Some("A"),
        axe_id: Some("document-title"),
        score_impact: ScoreImpact {
            base_penalty: 5.0,
            max_penalty: 5.0,
            occurrence_scaling: Scaling::Fixed,
        },
        report_visibility: VIS_ALL,
    },
    Rule {
        id: "landmarks/not-unique",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::StructureSemantics,
        issue_class: IssueClass::Invalid,
        severity: Severity::Medium,
        external_ref: Some("WCAG 1.3.1"),
        external_level: Some("A"),
        axe_id: Some("landmark-unique"),
        score_impact: ScoreImpact {
            base_penalty: 1.5,
            max_penalty: 4.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "dialog/focusable-missing",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::NavigationInteraction,
        issue_class: IssueClass::Weak,
        severity: Severity::High,
        external_ref: Some("WCAG 2.4.3"),
        external_level: Some("A"),
        axe_id: Some("focus-order-semantics"),
        score_impact: ScoreImpact {
            base_penalty: 2.5,
            max_penalty: 8.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "links/generic-name",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::NavigationInteraction,
        issue_class: IssueClass::Weak,
        severity: Severity::Medium,
        external_ref: Some("WCAG 2.4.4"),
        external_level: Some("A"),
        axe_id: Some("link-name"),
        score_impact: ScoreImpact {
            base_penalty: 1.0,
            max_penalty: 5.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "links/single-character-name",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::NavigationInteraction,
        issue_class: IssueClass::Weak,
        severity: Severity::Medium,
        external_ref: Some("WCAG 2.4.4"),
        external_level: Some("A"),
        axe_id: Some("link-name"),
        score_impact: ScoreImpact {
            base_penalty: 1.0,
            max_penalty: 5.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "links/url-name",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::NavigationInteraction,
        issue_class: IssueClass::Weak,
        severity: Severity::Medium,
        external_ref: Some("WCAG 2.4.4"),
        external_level: Some("A"),
        axe_id: Some("link-name"),
        score_impact: ScoreImpact {
            base_penalty: 1.0,
            max_penalty: 5.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "focus-visible",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::NavigationInteraction,
        issue_class: IssueClass::Missing,
        severity: Severity::High,
        external_ref: Some("WCAG 2.4.7"),
        external_level: Some("AA"),
        axe_id: Some("focus-visible"),
        score_impact: ScoreImpact {
            base_penalty: 3.0,
            max_penalty: 8.0,
            occurrence_scaling: Scaling::Fixed,
        },
        report_visibility: VIS_ALL,
    },
    Rule {
        id: "focus/obscured",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::NavigationInteraction,
        issue_class: IssueClass::Weak,
        severity: Severity::Medium,
        external_ref: Some("WCAG 2.4.11"),
        external_level: Some("AA"),
        axe_id: Some("focus-not-obscured-minimum"),
        score_impact: ScoreImpact {
            base_penalty: 0.5,
            max_penalty: 2.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "focus/obscured-enhanced",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::NavigationInteraction,
        issue_class: IssueClass::Weak,
        severity: Severity::Medium,
        external_ref: Some("WCAG 2.4.12"),
        external_level: Some("AAA"),
        axe_id: Some("focus-not-obscured-enhanced"),
        score_impact: ScoreImpact {
            base_penalty: 0.5,
            max_penalty: 2.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "label-in-name/mismatch",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::NavigationInteraction,
        issue_class: IssueClass::Invalid,
        severity: Severity::High,
        external_ref: Some("WCAG 2.5.3"),
        external_level: Some("A"),
        axe_id: Some("label-in-name"),
        score_impact: ScoreImpact {
            base_penalty: 2.5,
            max_penalty: 8.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "meta-refresh",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::NavigationInteraction,
        issue_class: IssueClass::Weak,
        severity: Severity::High,
        external_ref: Some("WCAG 2.2.1"),
        external_level: Some("A"),
        axe_id: Some("meta-refresh"),
        score_impact: ScoreImpact {
            base_penalty: 3.0,
            max_penalty: 10.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_ALL,
    },
    Rule {
        id: "timing/meta-refresh",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::NavigationInteraction,
        issue_class: IssueClass::Weak,
        severity: Severity::High,
        external_ref: Some("WCAG 2.2.1"),
        external_level: Some("A"),
        axe_id: Some("meta-refresh"),
        score_impact: ScoreImpact {
            base_penalty: 3.0,
            max_penalty: 10.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_ALL,
    },
    Rule {
        id: "motion/infinite-animation",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::NavigationInteraction,
        issue_class: IssueClass::Missing,
        severity: Severity::Medium,
        external_ref: Some("WCAG 2.2.2"),
        external_level: Some("A"),
        axe_id: Some("pause-stop-hide"),
        score_impact: ScoreImpact {
            base_penalty: 0.5,
            max_penalty: 2.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "manual/no-timing",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::NavigationInteraction,
        issue_class: IssueClass::Risk,
        severity: Severity::Medium,
        external_ref: Some("WCAG 2.2.3"),
        external_level: Some("AAA"),
        axe_id: Some("no-timing"),
        score_impact: ScoreImpact {
            base_penalty: 0.5,
            max_penalty: 1.5,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "manual/interruptions",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::NavigationInteraction,
        issue_class: IssueClass::Risk,
        severity: Severity::Low,
        external_ref: Some("WCAG 2.2.4"),
        external_level: Some("AAA"),
        axe_id: Some("no-interruptions"),
        score_impact: ScoreImpact {
            base_penalty: 0.5,
            max_penalty: 1.5,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "manual/re-authenticating",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::NavigationInteraction,
        issue_class: IssueClass::Risk,
        severity: Severity::Low,
        external_ref: Some("WCAG 2.2.5"),
        external_level: Some("AAA"),
        axe_id: Some("re-authenticate"),
        score_impact: ScoreImpact {
            base_penalty: 0.5,
            max_penalty: 1.5,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "timeouts",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::NavigationInteraction,
        issue_class: IssueClass::Risk,
        severity: Severity::Medium,
        external_ref: Some("WCAG 2.2.6"),
        external_level: Some("AAA"),
        axe_id: Some("timeouts"),
        score_impact: ScoreImpact {
            base_penalty: 0.5,
            max_penalty: 1.5,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "navigation/location-missing",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::NavigationInteraction,
        issue_class: IssueClass::Missing,
        severity: Severity::Low,
        external_ref: Some("WCAG 2.4.8"),
        external_level: Some("AAA"),
        axe_id: Some("location"),
        score_impact: ScoreImpact {
            base_penalty: 0.5,
            max_penalty: 1.5,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "links/purpose-link-only",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::NavigationInteraction,
        issue_class: IssueClass::Weak,
        severity: Severity::Medium,
        external_ref: Some("WCAG 2.4.9"),
        external_level: Some("AAA"),
        axe_id: Some("link-name-only"),
        score_impact: ScoreImpact {
            base_penalty: 0.5,
            max_penalty: 1.5,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "links/new-window-unannounced",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::NavigationInteraction,
        issue_class: IssueClass::Weak,
        severity: Severity::Low,
        external_ref: Some("WCAG 3.2.5"),
        external_level: Some("AAA"),
        axe_id: None,
        score_impact: ScoreImpact {
            base_penalty: 0.5,
            max_penalty: 1.5,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "manual/pointer-gestures",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::NavigationInteraction,
        issue_class: IssueClass::Missing,
        severity: Severity::Medium,
        external_ref: Some("WCAG 2.5.1"),
        external_level: Some("A"),
        axe_id: Some("pointer-gestures"),
        score_impact: ScoreImpact {
            base_penalty: 0.5,
            max_penalty: 1.5,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "pointer-cancellation",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::NavigationInteraction,
        issue_class: IssueClass::Invalid,
        severity: Severity::Medium,
        external_ref: Some("WCAG 2.5.2"),
        external_level: Some("A"),
        axe_id: Some("pointer-cancellation"),
        score_impact: ScoreImpact {
            base_penalty: 0.5,
            max_penalty: 1.5,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "manual/motion-actuation",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::NavigationInteraction,
        issue_class: IssueClass::Missing,
        severity: Severity::Medium,
        external_ref: Some("WCAG 2.5.4"),
        external_level: Some("A"),
        axe_id: Some("motion-actuation"),
        score_impact: ScoreImpact {
            base_penalty: 0.5,
            max_penalty: 1.5,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "targets/size-enhanced",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::NavigationInteraction,
        issue_class: IssueClass::Weak,
        severity: Severity::Medium,
        external_ref: Some("WCAG 2.5.5"),
        external_level: Some("AAA"),
        axe_id: Some("target-size"),
        score_impact: ScoreImpact {
            base_penalty: 0.5,
            max_penalty: 1.5,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "targets/size",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::NavigationInteraction,
        issue_class: IssueClass::Weak,
        severity: Severity::Medium,
        external_ref: Some("WCAG 2.5.8"),
        external_level: Some("AA"),
        axe_id: Some("target-size-minimum"),
        score_impact: ScoreImpact {
            base_penalty: 0.5,
            max_penalty: 1.5,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    // ── Formulare & Interaktion ───────────────────────────────────────────
    Rule {
        id: "forms/autocomplete-invalid",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::FormsInteraction,
        issue_class: IssueClass::Missing,
        severity: Severity::Medium,
        external_ref: Some("WCAG 1.3.5"),
        external_level: Some("AA"),
        axe_id: Some("autocomplete-valid"),
        score_impact: ScoreImpact {
            base_penalty: 1.0,
            max_penalty: 5.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "forms/purpose-missing",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::FormsInteraction,
        issue_class: IssueClass::Missing,
        severity: Severity::Medium,
        external_ref: Some("WCAG 1.3.5"),
        external_level: Some("AA"),
        axe_id: Some("autocomplete-valid"),
        score_impact: ScoreImpact {
            base_penalty: 1.0,
            max_penalty: 5.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "forms/group-name-missing",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::FormsInteraction,
        issue_class: IssueClass::Missing,
        severity: Severity::High,
        external_ref: Some("WCAG 3.3.2"),
        external_level: Some("A"),
        axe_id: Some("label"),
        score_impact: ScoreImpact {
            base_penalty: 2.5,
            max_penalty: 8.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_ALL,
    },
    Rule {
        id: "forms/instructions-missing",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::FormsInteraction,
        issue_class: IssueClass::Missing,
        severity: Severity::High,
        external_ref: Some("WCAG 3.3.2"),
        external_level: Some("A"),
        axe_id: Some("label"),
        score_impact: ScoreImpact {
            base_penalty: 2.5,
            max_penalty: 8.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_ALL,
    },
    Rule {
        id: "forms/label-missing",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::FormsInteraction,
        issue_class: IssueClass::Missing,
        severity: Severity::High,
        external_ref: Some("WCAG 3.3.2"),
        external_level: Some("A"),
        axe_id: Some("label"),
        score_impact: ScoreImpact {
            base_penalty: 2.5,
            max_penalty: 8.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_ALL,
    },
    Rule {
        id: "forms/placeholder-as-label",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::FormsInteraction,
        issue_class: IssueClass::Missing,
        severity: Severity::High,
        external_ref: Some("WCAG 3.3.2"),
        external_level: Some("A"),
        axe_id: Some("label"),
        score_impact: ScoreImpact {
            base_penalty: 2.5,
            max_penalty: 8.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_ALL,
    },
    Rule {
        id: "forms/required-unmarked",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::FormsInteraction,
        issue_class: IssueClass::Missing,
        severity: Severity::High,
        external_ref: Some("WCAG 3.3.2"),
        external_level: Some("A"),
        axe_id: Some("label"),
        score_impact: ScoreImpact {
            base_penalty: 2.5,
            max_penalty: 8.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_ALL,
    },
    Rule {
        id: "forms/redundant-entry",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::FormsInteraction,
        issue_class: IssueClass::Weak,
        severity: Severity::Medium,
        external_ref: Some("WCAG 3.3.7"),
        external_level: Some("A"),
        axe_id: Some("redundant-entry"),
        score_impact: ScoreImpact {
            base_penalty: 0.5,
            max_penalty: 2.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "auth/paste-blocked",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::FormsInteraction,
        issue_class: IssueClass::Invalid,
        severity: Severity::High,
        external_ref: Some("WCAG 3.3.8"),
        external_level: Some("AA"),
        axe_id: Some("accessible-auth-paste-blocked"),
        score_impact: ScoreImpact {
            base_penalty: 3.0,
            max_penalty: 8.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_ALL,
    },
    Rule {
        id: "auth/captcha",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::FormsInteraction,
        issue_class: IssueClass::Risk,
        severity: Severity::Medium,
        external_ref: Some("WCAG 3.3.8"),
        external_level: Some("AA"),
        axe_id: Some("accessible-auth-captcha"),
        score_impact: ScoreImpact {
            base_penalty: 0.5,
            max_penalty: 2.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "forms/no-submit",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::FormsInteraction,
        issue_class: IssueClass::Missing,
        severity: Severity::Medium,
        external_ref: Some("WCAG 3.2.2"),
        external_level: Some("A"),
        axe_id: Some("form-no-submit"),
        score_impact: ScoreImpact {
            base_penalty: 1.5,
            max_penalty: 4.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "manual/help",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::FormsInteraction,
        issue_class: IssueClass::Missing,
        severity: Severity::Low,
        external_ref: Some("WCAG 3.3.5"),
        external_level: Some("AAA"),
        axe_id: Some("help"),
        score_impact: ScoreImpact {
            base_penalty: 0.5,
            max_penalty: 1.5,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    // ── Sprache & Verständlichkeit ────────────────────────────────────────
    Rule {
        id: "document/lang-missing",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::LanguageClarity,
        issue_class: IssueClass::Missing,
        severity: Severity::High,
        external_ref: Some("WCAG 3.1.1"),
        external_level: Some("A"),
        // Geteilte Kennung aus `a11y-rules` statt der frueheren axe-core-Kennung
        // `html-has-lang`: Die Pruefung laeuft seit der Umstellung als
        // `document/lang-missing` (siehe `wcag::shared`). Die alte Kennung
        // erzeugt weiterhin `iframe_rules`, aber fuer das Dokument *im* iframe
        // -- ein anderer Befund.
        axe_id: Some("document/lang-missing"),
        score_impact: ScoreImpact {
            base_penalty: 10.0,
            max_penalty: 12.5,
            occurrence_scaling: Scaling::Fixed,
        },
        report_visibility: VIS_ALL,
    },
    Rule {
        id: "html-has-lang",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::LanguageClarity,
        issue_class: IssueClass::Missing,
        severity: Severity::High,
        external_ref: Some("WCAG 3.1.1"),
        external_level: Some("A"),
        // Geteilte Kennung aus `a11y-rules` statt der frueheren axe-core-Kennung
        // `html-has-lang`: Die Pruefung laeuft seit der Umstellung als
        // `document/lang-missing` (siehe `wcag::shared`). Die alte Kennung
        // erzeugt weiterhin `iframe_rules`, aber fuer das Dokument *im* iframe
        // -- ein anderer Befund.
        axe_id: Some("document/lang-missing"),
        score_impact: ScoreImpact {
            base_penalty: 10.0,
            max_penalty: 12.5,
            occurrence_scaling: Scaling::Fixed,
        },
        report_visibility: VIS_ALL,
    },
    Rule {
        id: "context/autofocus",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::NavigationInteraction,
        issue_class: IssueClass::Risk,
        severity: Severity::High,
        external_ref: Some("WCAG 3.2.1"),
        external_level: Some("A"),
        axe_id: Some("focus-no-context-change"),
        score_impact: ScoreImpact {
            base_penalty: 3.0,
            max_penalty: 8.0,
            occurrence_scaling: Scaling::Fixed,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "context/on-focus",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::NavigationInteraction,
        issue_class: IssueClass::Risk,
        severity: Severity::High,
        external_ref: Some("WCAG 3.2.1"),
        external_level: Some("A"),
        axe_id: Some("focus-no-context-change"),
        score_impact: ScoreImpact {
            base_penalty: 3.0,
            max_penalty: 8.0,
            occurrence_scaling: Scaling::Fixed,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "context/on-input",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::NavigationInteraction,
        issue_class: IssueClass::Risk,
        severity: Severity::Medium,
        external_ref: Some("WCAG 3.2.2"),
        external_level: Some("A"),
        axe_id: Some("input-no-context-change"),
        score_impact: ScoreImpact {
            base_penalty: 2.0,
            max_penalty: 5.0,
            occurrence_scaling: Scaling::Fixed,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "manual/unusual-words",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::LanguageClarity,
        issue_class: IssueClass::Missing,
        severity: Severity::Low,
        external_ref: Some("WCAG 3.1.3"),
        external_level: Some("AAA"),
        axe_id: Some("unusual-words"),
        score_impact: ScoreImpact {
            base_penalty: 0.5,
            max_penalty: 1.5,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "language/part-undetermined",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::LanguageClarity,
        issue_class: IssueClass::Missing,
        severity: Severity::Medium,
        external_ref: Some("WCAG 3.1.2"),
        external_level: Some("AA"),
        axe_id: Some("language/part-unmarked"),
        score_impact: ScoreImpact {
            base_penalty: 0.5,
            max_penalty: 2.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "language/part-unmarked",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::LanguageClarity,
        issue_class: IssueClass::Missing,
        severity: Severity::Medium,
        external_ref: Some("WCAG 3.1.2"),
        external_level: Some("AA"),
        axe_id: Some("language/part-unmarked"),
        score_impact: ScoreImpact {
            base_penalty: 0.5,
            max_penalty: 2.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "language/abbreviation-unexpanded",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::LanguageClarity,
        issue_class: IssueClass::Missing,
        severity: Severity::Low,
        external_ref: Some("WCAG 3.1.4"),
        external_level: Some("AAA"),
        axe_id: Some("abbreviations"),
        score_impact: ScoreImpact {
            base_penalty: 0.5,
            max_penalty: 1.5,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    // ── Visuelle Darstellung / Kontrast ───────────────────────────────────
    Rule {
        id: "contrast/text-insufficient",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::VisualPresentation,
        issue_class: IssueClass::Weak,
        severity: Severity::High,
        external_ref: Some("WCAG 1.4.3"),
        external_level: Some("AA"),
        axe_id: Some("color-contrast"),
        score_impact: ScoreImpact {
            base_penalty: 5.0,
            max_penalty: 12.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_ALL,
    },
    Rule {
        id: "contrast/text-undetermined",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::VisualPresentation,
        issue_class: IssueClass::Weak,
        severity: Severity::High,
        external_ref: Some("WCAG 1.4.3"),
        external_level: Some("AA"),
        axe_id: Some("color-contrast"),
        score_impact: ScoreImpact {
            base_penalty: 5.0,
            max_penalty: 12.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_ALL,
    },
    Rule {
        id: "contrast/text-enhanced",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::VisualPresentation,
        issue_class: IssueClass::Weak,
        severity: Severity::Medium,
        external_ref: Some("WCAG 1.4.6"),
        external_level: Some("AAA"),
        axe_id: Some("color-contrast-enhanced"),
        score_impact: ScoreImpact {
            base_penalty: 0.5,
            max_penalty: 1.5,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "zoom/viewport-locked",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::VisualPresentation,
        issue_class: IssueClass::Weak,
        severity: Severity::High,
        external_ref: Some("WCAG 1.4.4"),
        external_level: Some("AA"),
        axe_id: Some("meta-viewport"),
        score_impact: ScoreImpact {
            base_penalty: 2.5,
            max_penalty: 8.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "zoom/viewport-missing",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::VisualPresentation,
        issue_class: IssueClass::Missing,
        severity: Severity::High,
        external_ref: Some("WCAG 1.4.4"),
        external_level: Some("AA"),
        axe_id: Some("zoom/viewport-missing"),
        // Geringeres Gewicht (#702): Desktop-Browser ignorieren das Tag, Zoomen
        // per Geste bleibt moeglich; die Folge trifft nur Mobilgeraete (Seite
        // in Desktop-Breite, verkleinert, beim Vergroessern waagerechtes
        // Scrollen). Befundstufe und Schwere kommen aus `a11y-rules`.
        score_impact: ScoreImpact {
            base_penalty: 1.5,
            max_penalty: 3.0,
            occurrence_scaling: Scaling::Fixed,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "css-overflow-hidden",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::VisualPresentation,
        issue_class: IssueClass::Missing,
        severity: Severity::High,
        external_ref: Some("WCAG 1.4.10"),
        external_level: Some("AA"),
        axe_id: Some("css-overflow-hidden"),
        score_impact: ScoreImpact {
            base_penalty: 3.0,
            max_penalty: 10.0,
            occurrence_scaling: Scaling::Fixed,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "contrast/control-insufficient",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::VisualPresentation,
        issue_class: IssueClass::Weak,
        severity: Severity::Medium,
        external_ref: Some("WCAG 1.4.11"),
        external_level: Some("AA"),
        axe_id: Some("non-text-contrast"),
        score_impact: ScoreImpact {
            base_penalty: 1.5,
            max_penalty: 5.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "contrast/control-undetermined",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::VisualPresentation,
        issue_class: IssueClass::Weak,
        severity: Severity::Medium,
        external_ref: Some("WCAG 1.4.11"),
        external_level: Some("AA"),
        axe_id: Some("non-text-contrast"),
        score_impact: ScoreImpact {
            base_penalty: 1.5,
            max_penalty: 5.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "text-spacing/clipped",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::VisualPresentation,
        issue_class: IssueClass::Invalid,
        severity: Severity::Medium,
        external_ref: Some("WCAG 1.4.12"),
        external_level: Some("AA"),
        axe_id: Some("text-spacing"),
        score_impact: ScoreImpact {
            base_penalty: 1.5,
            max_penalty: 5.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "color/link-indistinct",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::VisualPresentation,
        issue_class: IssueClass::Weak,
        severity: Severity::Medium,
        external_ref: Some("WCAG 1.4.1"),
        external_level: Some("A"),
        axe_id: Some("link-in-text-block"),
        score_impact: ScoreImpact {
            base_penalty: 3.0,
            max_penalty: 8.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "motion/reduced-motion-ignored",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::VisualPresentation,
        issue_class: IssueClass::Missing,
        severity: Severity::Low,
        external_ref: Some("WCAG 2.3.3"),
        external_level: Some("AAA"),
        axe_id: Some("prefers-reduced-motion"),
        score_impact: ScoreImpact {
            base_penalty: 1.0,
            max_penalty: 4.0,
            occurrence_scaling: Scaling::Fixed,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "patterns/tooltip-unreferenced",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::VisualPresentation,
        issue_class: IssueClass::Weak,
        severity: Severity::Medium,
        external_ref: Some("WCAG 1.4.13"),
        external_level: Some("AA"),
        axe_id: Some("content-on-hover-focus"),
        score_impact: ScoreImpact {
            base_penalty: 2.0,
            max_penalty: 6.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "media/audio-autoplay",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::VisualPresentation,
        issue_class: IssueClass::Risk,
        severity: Severity::Medium,
        external_ref: Some("WCAG 1.4.2"),
        external_level: Some("A"),
        axe_id: Some("media/audio-autoplay"),
        score_impact: ScoreImpact {
            base_penalty: 0.5,
            max_penalty: 1.5,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "text/justified",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::VisualPresentation,
        issue_class: IssueClass::Weak,
        severity: Severity::Low,
        external_ref: Some("WCAG 1.4.8"),
        external_level: Some("AAA"),
        axe_id: Some("visual-presentation"),
        score_impact: ScoreImpact {
            base_penalty: 0.5,
            max_penalty: 1.5,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "text/line-height-tight",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::VisualPresentation,
        issue_class: IssueClass::Weak,
        severity: Severity::Low,
        external_ref: Some("WCAG 1.4.8"),
        external_level: Some("AAA"),
        axe_id: Some("visual-presentation"),
        score_impact: ScoreImpact {
            base_penalty: 0.5,
            max_penalty: 1.5,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "visual-presentation",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::VisualPresentation,
        issue_class: IssueClass::Weak,
        severity: Severity::Low,
        external_ref: Some("WCAG 1.4.8"),
        external_level: Some("AAA"),
        axe_id: Some("visual-presentation"),
        score_impact: ScoreImpact {
            base_penalty: 0.5,
            max_penalty: 1.5,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    // ── Technische Robustheit ─────────────────────────────────────────────
    Rule {
        id: "keyboard/hidden-focusable",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::TechnicalRobustness,
        issue_class: IssueClass::Invalid,
        severity: Severity::High,
        external_ref: Some("WCAG 4.1.2"),
        external_level: Some("A"),
        axe_id: Some("aria-hidden-focus"),
        score_impact: ScoreImpact {
            base_penalty: 6.0,
            max_penalty: 15.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_ALL,
    },
    Rule {
        id: "status/live-overridden",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::TechnicalRobustness,
        issue_class: IssueClass::Invalid,
        severity: Severity::Medium,
        external_ref: Some("WCAG 4.1.3"),
        external_level: Some("AA"),
        axe_id: Some("aria-live-region-role"),
        score_impact: ScoreImpact {
            base_penalty: 3.0,
            max_penalty: 8.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "aria/required-attribute-missing",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::TechnicalRobustness,
        issue_class: IssueClass::Missing,
        severity: Severity::Critical,
        external_ref: Some("WCAG 4.1.2"),
        external_level: Some("A"),
        axe_id: Some("aria-required-attr"),
        score_impact: ScoreImpact {
            base_penalty: 6.0,
            max_penalty: 15.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_ALL,
    },
    Rule {
        id: "aria/required-parent-missing",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::TechnicalRobustness,
        issue_class: IssueClass::Missing,
        severity: Severity::Critical,
        external_ref: Some("WCAG 4.1.2"),
        external_level: Some("A"),
        axe_id: Some("aria-required-parent"),
        score_impact: ScoreImpact {
            base_penalty: 6.0,
            max_penalty: 15.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_ALL,
    },
    Rule {
        id: "inert/dialog-inert",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::TechnicalRobustness,
        issue_class: IssueClass::Invalid,
        severity: Severity::High,
        external_ref: Some("WCAG 4.1.2"),
        external_level: Some("A"),
        axe_id: Some("modern-attribute-misuse"),
        score_impact: ScoreImpact {
            base_penalty: 4.0,
            max_penalty: 10.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_ALL,
    },
    Rule {
        id: "modern-attribute-misuse",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::TechnicalRobustness,
        issue_class: IssueClass::Invalid,
        severity: Severity::High,
        external_ref: Some("WCAG 4.1.2"),
        external_level: Some("A"),
        axe_id: Some("modern-attribute-misuse"),
        score_impact: ScoreImpact {
            base_penalty: 4.0,
            max_penalty: 10.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_ALL,
    },
    Rule {
        id: "popover/target-invalid",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::TechnicalRobustness,
        issue_class: IssueClass::Invalid,
        severity: Severity::High,
        external_ref: Some("WCAG 4.1.2"),
        external_level: Some("A"),
        axe_id: Some("modern-attribute-misuse"),
        score_impact: ScoreImpact {
            base_penalty: 4.0,
            max_penalty: 10.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_ALL,
    },
    Rule {
        id: "popover/target-missing",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::TechnicalRobustness,
        issue_class: IssueClass::Invalid,
        severity: Severity::High,
        external_ref: Some("WCAG 4.1.2"),
        external_level: Some("A"),
        axe_id: Some("modern-attribute-misuse"),
        score_impact: ScoreImpact {
            base_penalty: 4.0,
            max_penalty: 10.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_ALL,
    },
    Rule {
        id: "aria/attribute-prohibited",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::TechnicalRobustness,
        issue_class: IssueClass::Invalid,
        severity: Severity::High,
        external_ref: Some("WCAG 4.1.2"),
        external_level: Some("A"),
        axe_id: Some("aria-prohibited-attr"),
        score_impact: ScoreImpact {
            base_penalty: 4.0,
            max_penalty: 10.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_ALL,
    },
    Rule {
        id: "aria/attribute-not-allowed",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::TechnicalRobustness,
        issue_class: IssueClass::Invalid,
        severity: Severity::High,
        external_ref: Some("WCAG 4.1.2"),
        external_level: Some("A"),
        axe_id: Some("aria-allowed-attr"),
        score_impact: ScoreImpact {
            base_penalty: 4.0,
            max_penalty: 10.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_ALL,
    },
    Rule {
        id: "aria/attribute-value-invalid",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::TechnicalRobustness,
        issue_class: IssueClass::Invalid,
        severity: Severity::High,
        external_ref: Some("WCAG 4.1.2"),
        external_level: Some("A"),
        axe_id: Some("aria-valid-attr-value"),
        score_impact: ScoreImpact {
            base_penalty: 4.0,
            max_penalty: 10.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_ALL,
    },
    Rule {
        id: "aria/reference-missing",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::TechnicalRobustness,
        issue_class: IssueClass::Invalid,
        severity: Severity::Medium,
        external_ref: Some("WCAG 4.1.2"),
        external_level: Some("A"),
        axe_id: Some("aria-valid-attr"),
        score_impact: ScoreImpact {
            base_penalty: 3.0,
            max_penalty: 8.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_ALL,
    },
    Rule {
        id: "aria/role-abstract",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::TechnicalRobustness,
        issue_class: IssueClass::Invalid,
        severity: Severity::High,
        external_ref: Some("WCAG 4.1.2"),
        external_level: Some("A"),
        axe_id: Some("aria-roles"),
        score_impact: ScoreImpact {
            base_penalty: 4.0,
            max_penalty: 10.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_ALL,
    },
    Rule {
        id: "aria/role-invalid",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::TechnicalRobustness,
        issue_class: IssueClass::Invalid,
        severity: Severity::High,
        external_ref: Some("WCAG 4.1.2"),
        external_level: Some("A"),
        axe_id: Some("aria-roles"),
        score_impact: ScoreImpact {
            base_penalty: 4.0,
            max_penalty: 10.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_ALL,
    },
    Rule {
        id: "aria/attribute-unknown",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::TechnicalRobustness,
        issue_class: IssueClass::Invalid,
        severity: Severity::High,
        external_ref: Some("WCAG 4.1.2"),
        external_level: Some("A"),
        axe_id: Some("aria-attr-name-invalid"),
        score_impact: ScoreImpact {
            base_penalty: 4.0,
            max_penalty: 10.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_ALL,
    },
    Rule {
        id: "document/lang-mismatch",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::LanguageClarity,
        issue_class: IssueClass::Invalid,
        severity: Severity::Medium,
        external_ref: Some("WCAG 3.1.1"),
        external_level: Some("A"),
        axe_id: Some("html-xml-lang-mismatch"),
        score_impact: ScoreImpact {
            base_penalty: 3.0,
            max_penalty: 8.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_ALL,
    },
    Rule {
        id: "tables/headers-attr-invalid",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::StructureSemantics,
        issue_class: IssueClass::Invalid,
        severity: Severity::High,
        external_ref: Some("WCAG 1.3.1"),
        external_level: Some("A"),
        axe_id: Some("td-headers-attr"),
        score_impact: ScoreImpact {
            base_penalty: 4.0,
            max_penalty: 10.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_ALL,
    },
    Rule {
        id: "tables/data-undetermined",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::StructureSemantics,
        issue_class: IssueClass::Missing,
        severity: Severity::High,
        external_ref: Some("WCAG 1.3.1"),
        external_level: Some("A"),
        axe_id: Some("th-has-data-cells"),
        score_impact: ScoreImpact {
            base_penalty: 3.0,
            max_penalty: 8.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_ALL,
    },
    Rule {
        id: "tables/header-without-data",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::StructureSemantics,
        issue_class: IssueClass::Missing,
        severity: Severity::High,
        external_ref: Some("WCAG 1.3.1"),
        external_level: Some("A"),
        axe_id: Some("th-has-data-cells"),
        score_impact: ScoreImpact {
            base_penalty: 3.0,
            max_penalty: 8.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_ALL,
    },
    Rule {
        id: "aria/tabpanel-missing",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::TechnicalRobustness,
        issue_class: IssueClass::Missing,
        severity: Severity::Medium,
        external_ref: Some("WCAG 4.1.2"),
        external_level: Some("A"),
        axe_id: Some("aria-tablist-tabpanel"),
        score_impact: ScoreImpact {
            base_penalty: 3.0,
            max_penalty: 8.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_ALL,
    },
    Rule {
        id: "aria/tab-selected-missing",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::TechnicalRobustness,
        issue_class: IssueClass::Missing,
        severity: Severity::Low,
        external_ref: Some("WCAG 4.1.2"),
        external_level: Some("A"),
        axe_id: Some("aria/tab-selected-missing"),
        score_impact: ScoreImpact {
            base_penalty: 1.0,
            max_penalty: 4.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "patterns/accordion-controls-missing",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::TechnicalRobustness,
        issue_class: IssueClass::Missing,
        severity: Severity::Low,
        external_ref: Some("WCAG 4.1.2"),
        external_level: Some("A"),
        axe_id: Some("patterns/accordion-controls-missing"),
        score_impact: ScoreImpact {
            base_penalty: 1.0,
            max_penalty: 4.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "dialog/modal-unmarked",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::TechnicalRobustness,
        issue_class: IssueClass::Missing,
        severity: Severity::Low,
        external_ref: Some("WCAG 4.1.2"),
        external_level: Some("A"),
        axe_id: Some("dialog/modal-unmarked"),
        score_impact: ScoreImpact {
            base_penalty: 1.0,
            max_penalty: 4.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "aria/combobox-popup-missing",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::FormsInteraction,
        issue_class: IssueClass::Missing,
        severity: Severity::Medium,
        external_ref: Some("WCAG 4.1.2"),
        external_level: Some("A"),
        axe_id: Some("aria-combobox-options"),
        score_impact: ScoreImpact {
            base_penalty: 3.0,
            max_penalty: 8.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_ALL,
    },
    Rule {
        id: "frames/name-missing",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::ContentAlternatives,
        issue_class: IssueClass::Missing,
        severity: Severity::High,
        external_ref: Some("WCAG 2.4.1"),
        external_level: Some("A"),
        axe_id: Some("frame-title"),
        score_impact: ScoreImpact {
            base_penalty: 4.0,
            max_penalty: 10.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_ALL,
    },
    Rule {
        id: "frames/cross-origin",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::TechnicalRobustness,
        issue_class: IssueClass::Informational,
        severity: Severity::Medium,
        external_ref: Some("WCAG 4.1.2"),
        external_level: Some("A"),
        axe_id: Some("frame-tested"),
        score_impact: ScoreImpact {
            base_penalty: 0.0,
            max_penalty: 0.0,
            occurrence_scaling: Scaling::Linear,
        },
        report_visibility: VIS_ALL,
    },
    Rule {
        id: "ids/duplicate",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::TechnicalRobustness,
        issue_class: IssueClass::Invalid,
        severity: Severity::Critical,
        // Seit WCAG 2.2 gibt es 4.1.1 nicht mehr. Gemeldet wird nur noch die
        // doppelte ID, auf die ein IDREF zeigt (`wcag::shared`) -- das ist ein
        // Verstoss gegen 4.1.2, weil die Beziehung nicht mehr eindeutig
        // aufloesbar ist (Plan 54 §2).
        external_ref: Some("WCAG 4.1.2"),
        external_level: Some("A"),
        // Geteilte Kennung aus `a11y-rules` statt der frueheren axe-core-Kennung
        // `duplicate-id`: Die Pruefung laeuft seit der Umstellung als
        // `ids/duplicate` (siehe `wcag::shared`). Die verwandte Kennung
        // `duplicate-id-aria` (widerspruechliche `aria-owns`) bleibt eine
        // eigene Regel und haengt an `a11y.duplicate_id_aria.invalid`.
        axe_id: Some("ids/duplicate"),
        score_impact: ScoreImpact {
            base_penalty: 5.0,
            max_penalty: 15.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_ALL,
    },
    Rule {
        id: "images/server-side-map",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::ContentAlternatives,
        issue_class: IssueClass::Invalid,
        severity: Severity::Medium,
        external_ref: Some("WCAG 1.1.1"),
        external_level: Some("A"),
        axe_id: Some("server-side-image-map"),
        score_impact: ScoreImpact {
            base_penalty: 1.5,
            max_penalty: 4.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "svg/name-missing",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::ContentAlternatives,
        issue_class: IssueClass::Missing,
        severity: Severity::High,
        external_ref: Some("WCAG 1.1.1"),
        external_level: Some("A"),
        axe_id: Some("svg-img-alt"),
        score_impact: ScoreImpact {
            base_penalty: 2.5,
            max_penalty: 8.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "images/area-alt-missing",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::ContentAlternatives,
        issue_class: IssueClass::Missing,
        severity: Severity::High,
        external_ref: Some("WCAG 1.1.1"),
        external_level: Some("A"),
        axe_id: Some("area-alt"),
        score_impact: ScoreImpact {
            base_penalty: 2.0,
            max_penalty: 6.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "images/input-alt-missing",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::ContentAlternatives,
        issue_class: IssueClass::Missing,
        severity: Severity::High,
        external_ref: Some("WCAG 1.1.1"),
        external_level: Some("A"),
        axe_id: Some("input-image-alt"),
        score_impact: ScoreImpact {
            base_penalty: 2.0,
            max_penalty: 6.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "objects/alt-missing",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::ContentAlternatives,
        issue_class: IssueClass::Missing,
        severity: Severity::High,
        external_ref: Some("WCAG 1.1.1"),
        external_level: Some("A"),
        axe_id: Some("object-alt"),
        score_impact: ScoreImpact {
            base_penalty: 1.5,
            max_penalty: 5.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "html-content-model",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::StructureSemantics,
        issue_class: IssueClass::Invalid,
        severity: Severity::Medium,
        external_ref: Some("WCAG 1.3.1"),
        external_level: Some("A"),
        axe_id: Some("html-content-model"),
        score_impact: ScoreImpact {
            base_penalty: 1.5,
            max_penalty: 4.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "forms/group-missing",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::StructureSemantics,
        issue_class: IssueClass::Missing,
        severity: Severity::Medium,
        external_ref: Some("WCAG 1.3.1"),
        external_level: Some("A"),
        axe_id: Some("form-field-group"),
        score_impact: ScoreImpact {
            base_penalty: 1.2,
            max_penalty: 3.5,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "landmarks/not-top-level",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::StructureSemantics,
        issue_class: IssueClass::Invalid,
        severity: Severity::Medium,
        external_ref: Some("WCAG 1.3.1"),
        external_level: Some("A"),
        axe_id: Some("landmarks/not-top-level"),
        score_impact: ScoreImpact {
            base_penalty: 1.0,
            max_penalty: 3.0,
            occurrence_scaling: Scaling::Fixed,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "landmarks/banner-duplicate",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::StructureSemantics,
        issue_class: IssueClass::Invalid,
        severity: Severity::Medium,
        external_ref: Some("WCAG 1.3.1"),
        external_level: Some("A"),
        axe_id: Some("landmark-no-duplicate-banner"),
        score_impact: ScoreImpact {
            base_penalty: 1.0,
            max_penalty: 3.0,
            occurrence_scaling: Scaling::Fixed,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "landmarks/contentinfo-duplicate",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::StructureSemantics,
        issue_class: IssueClass::Invalid,
        severity: Severity::Medium,
        external_ref: Some("WCAG 1.3.1"),
        external_level: Some("A"),
        axe_id: Some("landmark-no-duplicate-contentinfo"),
        score_impact: ScoreImpact {
            base_penalty: 1.0,
            max_penalty: 3.0,
            occurrence_scaling: Scaling::Fixed,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "landmarks/main-duplicate",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::StructureSemantics,
        issue_class: IssueClass::Invalid,
        severity: Severity::Medium,
        external_ref: Some("WCAG 1.3.1"),
        external_level: Some("A"),
        axe_id: Some("landmark-no-duplicate-main"),
        score_impact: ScoreImpact {
            base_penalty: 1.2,
            max_penalty: 3.5,
            occurrence_scaling: Scaling::Fixed,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "landmarks/banner-missing",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::StructureSemantics,
        issue_class: IssueClass::Missing,
        severity: Severity::Medium,
        external_ref: Some("WCAG 1.3.1"),
        external_level: Some("A"),
        axe_id: Some("landmark-banner-present"),
        score_impact: ScoreImpact {
            base_penalty: 1.0,
            max_penalty: 3.0,
            occurrence_scaling: Scaling::Fixed,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "landmarks/navigation-missing",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::StructureSemantics,
        issue_class: IssueClass::Missing,
        severity: Severity::Low,
        external_ref: Some("WCAG 1.3.1"),
        external_level: Some("A"),
        axe_id: Some("landmark-navigation-present"),
        score_impact: ScoreImpact {
            base_penalty: 2.5,
            max_penalty: 8.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "landmarks/contentinfo-missing",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::StructureSemantics,
        issue_class: IssueClass::Missing,
        severity: Severity::Low,
        external_ref: Some("WCAG 1.3.1"),
        external_level: Some("A"),
        axe_id: Some("landmark-contentinfo-present"),
        score_impact: ScoreImpact {
            base_penalty: 2.5,
            max_penalty: 8.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "lists/item-outside-list",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::StructureSemantics,
        issue_class: IssueClass::Missing,
        severity: Severity::Medium,
        external_ref: Some("WCAG 1.3.1"),
        external_level: Some("A"),
        // Geteilte Kennung `lists/item-outside-list`; die leere Liste hat seit
        // Plan 56 einen eigenen Eintrag (`a11y.list_empty.invalid`).
        axe_id: Some("lists/item-outside-list"),
        score_impact: ScoreImpact {
            base_penalty: 1.2,
            max_penalty: 3.5,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "headings/skip-level",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::StructureSemantics,
        issue_class: IssueClass::Invalid,
        severity: Severity::Medium,
        external_ref: Some("WCAG 1.3.1"),
        external_level: Some("A"),
        axe_id: Some("headings/skip-level"),
        score_impact: ScoreImpact {
            base_penalty: 2.5,
            max_penalty: 8.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "headings/empty",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::StructureSemantics,
        issue_class: IssueClass::Invalid,
        severity: Severity::Medium,
        external_ref: Some("WCAG 2.4.6"),
        external_level: Some("AA"),
        axe_id: Some("headings/empty"),
        score_impact: ScoreImpact {
            base_penalty: 2.5,
            max_penalty: 8.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "headings/h1-missing",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::StructureSemantics,
        issue_class: IssueClass::Missing,
        severity: Severity::Medium,
        // Best practice, no success criterion (#779): the shared rule only hints.
        external_ref: None,
        external_level: None,
        axe_id: Some("headings/h1-missing"),
        score_impact: ScoreImpact {
            base_penalty: 2.5,
            max_penalty: 8.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "headings/h1-multiple",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::StructureSemantics,
        issue_class: IssueClass::Weak,
        severity: Severity::Low,
        external_ref: Some("WCAG 1.3.1"),
        external_level: Some("A"),
        axe_id: Some("headings/h1-multiple"),
        score_impact: ScoreImpact {
            base_penalty: 2.5,
            max_penalty: 8.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "lists/invalid-structure",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::StructureSemantics,
        issue_class: IssueClass::Invalid,
        severity: Severity::Medium,
        external_ref: Some("WCAG 1.3.1"),
        external_level: Some("A"),
        axe_id: Some("lists/invalid-structure"),
        score_impact: ScoreImpact {
            base_penalty: 2.5,
            max_penalty: 8.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "lists/empty",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::StructureSemantics,
        issue_class: IssueClass::Invalid,
        severity: Severity::Low,
        external_ref: Some("WCAG 1.3.1"),
        external_level: Some("A"),
        axe_id: Some("lists/empty"),
        score_impact: ScoreImpact {
            base_penalty: 2.5,
            max_penalty: 8.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "lists/term-without-definition",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::StructureSemantics,
        issue_class: IssueClass::Missing,
        severity: Severity::Medium,
        external_ref: Some("WCAG 1.3.1"),
        external_level: Some("A"),
        axe_id: Some("lists/term-without-definition"),
        score_impact: ScoreImpact {
            base_penalty: 2.5,
            max_penalty: 8.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "tables/header-missing",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::StructureSemantics,
        issue_class: IssueClass::Missing,
        severity: Severity::High,
        external_ref: Some("WCAG 1.3.1"),
        external_level: Some("A"),
        axe_id: Some("tables/header-missing"),
        score_impact: ScoreImpact {
            base_penalty: 2.5,
            max_penalty: 8.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "tables/name-missing",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::StructureSemantics,
        issue_class: IssueClass::Missing,
        severity: Severity::Low,
        external_ref: Some("WCAG 1.3.1"),
        external_level: Some("A"),
        axe_id: Some("tables/name-missing"),
        score_impact: ScoreImpact {
            base_penalty: 2.5,
            max_penalty: 8.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "tables/presentational-with-headers",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::StructureSemantics,
        issue_class: IssueClass::Invalid,
        severity: Severity::Medium,
        external_ref: Some("WCAG 1.3.1"),
        external_level: Some("A"),
        axe_id: Some("tables/presentational-with-headers"),
        score_impact: ScoreImpact {
            base_penalty: 2.5,
            max_penalty: 8.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "document/lang-invalid",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::LanguageClarity,
        issue_class: IssueClass::Invalid,
        severity: Severity::High,
        external_ref: Some("WCAG 3.1.1"),
        external_level: Some("A"),
        axe_id: Some("document/lang-invalid"),
        score_impact: ScoreImpact {
            base_penalty: 10.0,
            max_penalty: 12.5,
            occurrence_scaling: Scaling::Fixed,
        },
        report_visibility: VIS_ALL,
    },
    Rule {
        id: "keyboard/positive-tabindex",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::NavigationInteraction,
        issue_class: IssueClass::Invalid,
        severity: Severity::High,
        external_ref: Some("WCAG 2.4.3"),
        external_level: Some("A"),
        axe_id: Some("keyboard/positive-tabindex"),
        score_impact: ScoreImpact {
            base_penalty: 2.5,
            max_penalty: 8.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "forms/title-only-label",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::StructureSemantics,
        issue_class: IssueClass::Invalid,
        severity: Severity::Medium,
        external_ref: Some("WCAG 1.3.1"),
        external_level: Some("A"),
        axe_id: Some("label-title-only"),
        score_impact: ScoreImpact {
            base_penalty: 1.2,
            max_penalty: 3.5,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "zoom/viewport-scale-limited",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::VisualPresentation,
        issue_class: IssueClass::Invalid,
        severity: Severity::Medium,
        external_ref: Some("WCAG 1.4.4"),
        external_level: Some("AA"),
        axe_id: Some("zoom/viewport-scale-limited"),
        score_impact: ScoreImpact {
            base_penalty: 1.5,
            max_penalty: 4.0,
            occurrence_scaling: Scaling::Fixed,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "keyboard/click-handler-not-focusable",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::NavigationInteraction,
        issue_class: IssueClass::Missing,
        severity: Severity::High,
        external_ref: Some("WCAG 2.1.1"),
        external_level: Some("A"),
        axe_id: Some("click-events-have-key-events"),
        score_impact: ScoreImpact {
            base_penalty: 2.5,
            max_penalty: 8.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "keyboard/pointer-only",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::NavigationInteraction,
        issue_class: IssueClass::Missing,
        severity: Severity::High,
        external_ref: Some("WCAG 2.1.1"),
        external_level: Some("A"),
        axe_id: Some("click-events-have-key-events"),
        score_impact: ScoreImpact {
            base_penalty: 2.5,
            max_penalty: 8.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "landmarks/main-missing",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::NavigationInteraction,
        issue_class: IssueClass::Missing,
        severity: Severity::Medium,
        external_ref: Some("WCAG 2.4.1"),
        external_level: Some("A"),
        axe_id: Some("landmark-one-main"),
        score_impact: ScoreImpact {
            base_penalty: 1.5,
            max_penalty: 4.0,
            occurrence_scaling: Scaling::Fixed,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "keyboard/skip-link-missing",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::NavigationInteraction,
        issue_class: IssueClass::Missing,
        severity: Severity::Medium,
        external_ref: Some("WCAG 2.4.1"),
        external_level: Some("A"),
        axe_id: Some("skip-link"),
        score_impact: ScoreImpact {
            base_penalty: 1.5,
            max_penalty: 4.0,
            occurrence_scaling: Scaling::Fixed,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "focus/indicator-missing",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::NavigationInteraction,
        issue_class: IssueClass::Weak,
        severity: Severity::High,
        external_ref: Some("WCAG 2.4.7"),
        external_level: Some("AA"),
        axe_id: Some("focus-visible"),
        score_impact: ScoreImpact {
            base_penalty: 2.5,
            max_penalty: 8.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "focus/outline-removed",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::NavigationInteraction,
        issue_class: IssueClass::Invalid,
        severity: Severity::High,
        external_ref: Some("WCAG 2.4.7"),
        external_level: Some("AA"),
        axe_id: Some("focus-visible-outline-none"),
        score_impact: ScoreImpact {
            base_penalty: 2.5,
            max_penalty: 8.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "keyboard/scrollable-region-not-focusable",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::NavigationInteraction,
        issue_class: IssueClass::Missing,
        severity: Severity::High,
        external_ref: Some("WCAG 2.1.1"),
        external_level: Some("A"),
        axe_id: Some("scrollable-region-focusable"),
        score_impact: ScoreImpact {
            base_penalty: 3.0,
            max_penalty: 10.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_ALL,
    },
    Rule {
        id: "forms/error-unidentified",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::FormsInteraction,
        issue_class: IssueClass::Missing,
        severity: Severity::Medium,
        external_ref: Some("WCAG 3.3.1"),
        external_level: Some("A"),
        axe_id: Some("input-error-message"),
        score_impact: ScoreImpact {
            base_penalty: 1.5,
            max_penalty: 4.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "aria/owns-conflict",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::TechnicalRobustness,
        issue_class: IssueClass::Invalid,
        severity: Severity::High,
        // 4.1.1 ist in WCAG 2.2 gestrichen; die mehrdeutige ARIA-Referenz
        // bricht Name, Rolle und Wert und faellt damit unter 4.1.2 -- dieselbe
        // Zuordnung wie in axe-core (Plan 54 §2).
        external_ref: Some("WCAG 4.1.2"),
        external_level: Some("A"),
        axe_id: Some("duplicate-id-aria"),
        score_impact: ScoreImpact {
            base_penalty: 3.0,
            max_penalty: 10.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "names/required-missing",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::TechnicalRobustness,
        issue_class: IssueClass::Missing,
        severity: Severity::High,
        external_ref: Some("WCAG 4.1.2"),
        external_level: Some("A"),
        axe_id: Some("aria-label"),
        score_impact: ScoreImpact {
            base_penalty: 2.5,
            max_penalty: 8.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "names/symbol-only",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::TechnicalRobustness,
        issue_class: IssueClass::Missing,
        severity: Severity::High,
        external_ref: Some("WCAG 4.1.2"),
        external_level: Some("A"),
        axe_id: Some("aria-label"),
        score_impact: ScoreImpact {
            base_penalty: 2.5,
            max_penalty: 8.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "names/title-only",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::TechnicalRobustness,
        issue_class: IssueClass::Weak,
        severity: Severity::Medium,
        external_ref: Some("WCAG 4.1.2"),
        external_level: Some("A"),
        axe_id: Some("title-only-description"),
        score_impact: ScoreImpact {
            base_penalty: 1.0,
            max_penalty: 4.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "description-duplicates-name",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::TechnicalRobustness,
        issue_class: IssueClass::Weak,
        severity: Severity::Low,
        external_ref: Some("WCAG 4.1.2"),
        external_level: Some("A"),
        axe_id: Some("description-duplicates-name"),
        score_impact: ScoreImpact {
            base_penalty: 0.5,
            max_penalty: 2.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "aria/required-children-missing",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::TechnicalRobustness,
        issue_class: IssueClass::Missing,
        severity: Severity::Medium,
        external_ref: Some("WCAG 4.1.2"),
        external_level: Some("A"),
        axe_id: Some("aria-required-children"),
        score_impact: ScoreImpact {
            base_penalty: 1.5,
            max_penalty: 4.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "dialog/name-missing",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::TechnicalRobustness,
        issue_class: IssueClass::Missing,
        severity: Severity::High,
        external_ref: Some("WCAG 4.1.2"),
        external_level: Some("A"),
        axe_id: Some("dialog-name"),
        score_impact: ScoreImpact {
            base_penalty: 2.0,
            max_penalty: 6.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "buttons/name-missing",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::TechnicalRobustness,
        issue_class: IssueClass::Missing,
        severity: Severity::High,
        external_ref: Some("WCAG 4.1.2"),
        external_level: Some("A"),
        axe_id: Some("control-missing-label"),
        score_impact: ScoreImpact {
            base_penalty: 2.5,
            max_penalty: 8.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_ALL,
    },
    Rule {
        id: "links/name-missing",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::TechnicalRobustness,
        issue_class: IssueClass::Missing,
        severity: Severity::High,
        external_ref: Some("WCAG 4.1.2"),
        external_level: Some("A"),
        axe_id: Some("control-missing-label"),
        score_impact: ScoreImpact {
            base_penalty: 2.5,
            max_penalty: 8.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_ALL,
    },
    Rule {
        id: "links/used-as-button",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::TechnicalRobustness,
        issue_class: IssueClass::Invalid,
        severity: Severity::Low,
        external_ref: Some("WCAG 4.1.2"),
        external_level: Some("A"),
        axe_id: Some("link-as-button"),
        score_impact: ScoreImpact {
            base_penalty: 0.5,
            max_penalty: 2.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "aria/role-redundant",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::TechnicalRobustness,
        issue_class: IssueClass::Invalid,
        severity: Severity::Low,
        external_ref: Some("WCAG 4.1.2"),
        external_level: Some("A"),
        axe_id: Some("redundant-role"),
        score_impact: ScoreImpact {
            base_penalty: 0.3,
            max_penalty: 1.5,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "lists/role-redundant",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::TechnicalRobustness,
        issue_class: IssueClass::Invalid,
        severity: Severity::Low,
        external_ref: Some("WCAG 4.1.2"),
        external_level: Some("A"),
        axe_id: Some("redundant-role"),
        score_impact: ScoreImpact {
            base_penalty: 0.3,
            max_penalty: 1.5,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "summary/name-missing",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::TechnicalRobustness,
        issue_class: IssueClass::Missing,
        severity: Severity::High,
        external_ref: Some("WCAG 4.1.2"),
        external_level: Some("A"),
        axe_id: Some("summary-name"),
        score_impact: ScoreImpact {
            base_penalty: 1.5,
            max_penalty: 4.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "landmarks/content-outside",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::StructureSemantics,
        issue_class: IssueClass::Missing,
        severity: Severity::Medium,
        external_ref: Some("WCAG 1.3.1"),
        external_level: Some("A"),
        axe_id: Some("region"),
        score_impact: ScoreImpact {
            base_penalty: 1.5,
            max_penalty: 4.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "keyboard/focusable-no-role",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::NavigationInteraction,
        issue_class: IssueClass::Invalid,
        severity: Severity::Critical,
        external_ref: Some("WCAG 2.1.1"),
        external_level: Some("A"),
        axe_id: Some("focusable-no-role"),
        score_impact: ScoreImpact {
            base_penalty: 2.0,
            max_penalty: 6.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "display/toggle-missing",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::NavigationInteraction,
        issue_class: IssueClass::Missing,
        severity: Severity::Medium,
        external_ref: Some("WCAG 2.2.2"),
        external_level: Some("A"),
        axe_id: Some("display/toggle-missing"),
        score_impact: ScoreImpact {
            base_penalty: 1.5,
            max_penalty: 4.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "display/init-missing",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::NavigationInteraction,
        issue_class: IssueClass::Weak,
        severity: Severity::Low,
        external_ref: Some("WCAG 2.2.2"),
        external_level: Some("A"),
        axe_id: Some("display/init-missing"),
        score_impact: ScoreImpact {
            base_penalty: 0.5,
            max_penalty: 1.5,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "display/text-media-visible",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::NavigationInteraction,
        issue_class: IssueClass::Invalid,
        severity: Severity::Medium,
        external_ref: Some("WCAG 2.2.2"),
        external_level: Some("A"),
        axe_id: Some("display/text-media-visible"),
        score_impact: ScoreImpact {
            base_penalty: 1.5,
            max_penalty: 4.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "display/text-not-visible",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::ContentAlternatives,
        issue_class: IssueClass::Missing,
        severity: Severity::High,
        external_ref: Some("WCAG 1.1.1"),
        external_level: Some("A"),
        axe_id: Some("display/text-not-visible"),
        score_impact: ScoreImpact {
            base_penalty: 2.5,
            max_penalty: 8.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "viz/text-missing",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::ContentAlternatives,
        issue_class: IssueClass::Missing,
        severity: Severity::High,
        external_ref: Some("WCAG 1.1.1"),
        external_level: Some("A"),
        axe_id: Some("display/text-not-visible"),
        score_impact: ScoreImpact {
            base_penalty: 2.5,
            max_penalty: 8.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "display/text-hidden",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::ContentAlternatives,
        issue_class: IssueClass::Invalid,
        severity: Severity::High,
        external_ref: Some("WCAG 1.1.1"),
        external_level: Some("A"),
        axe_id: Some("display/text-hidden"),
        score_impact: ScoreImpact {
            base_penalty: 2.5,
            max_penalty: 8.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "viz/caption-missing",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::ContentAlternatives,
        issue_class: IssueClass::Missing,
        severity: Severity::Low,
        external_ref: Some("WCAG 1.1.1"),
        external_level: Some("A"),
        axe_id: Some("viz/caption-missing"),
        score_impact: ScoreImpact {
            base_penalty: 0.5,
            max_penalty: 2.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "viz/static-missing",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::NavigationInteraction,
        issue_class: IssueClass::Missing,
        severity: Severity::Medium,
        external_ref: Some("WCAG 2.2.2"),
        external_level: Some("A"),
        axe_id: Some("viz/static-missing"),
        score_impact: ScoreImpact {
            base_penalty: 1.5,
            max_penalty: 4.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
    Rule {
        id: "viz/table-missing",
        dimension: Dimension::Accessibility,
        subcategory: Subcategory::StructureSemantics,
        issue_class: IssueClass::Weak,
        severity: Severity::Low,
        external_ref: Some("WCAG 1.3.1"),
        external_level: Some("A"),
        axe_id: Some("viz/table-missing"),
        score_impact: ScoreImpact {
            base_penalty: 0.5,
            max_penalty: 2.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        report_visibility: VIS_STANDARD,
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::ftl_entries;

    #[test]
    fn test_rule_lookup_by_id() {
        let rule = RuleLookup::by_id("images/alt-missing");
        assert!(rule.is_some());
        let rule = rule.unwrap();
        assert_eq!(rule.dimension, Dimension::Accessibility);
        assert_eq!(rule.severity, Severity::High);
    }

    #[test]
    fn test_frame_title_taxonomy_uses_wcag_241() {
        let rule = RuleLookup::by_id("frames/name-missing").expect("frames/name-missing");
        assert_eq!(rule.external_ref, Some("WCAG 2.4.1"));
        assert_eq!(rule.axe_id, Some("frame-title"));
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

    /// Plan 56: Jede geteilte Kennung hat einen eigenen Eintrag mit dem
    /// Kriterium, das `SHARED_RULES` führt — ohne ihn fiele sie in einen
    /// Sammelbucket und verschmölze mit fremden Befunden.
    #[test]
    fn test_shared_rules_have_their_own_taxonomy_entry() {
        for shared in crate::wcag::shared::SHARED_RULES {
            let rule = RuleLookup::by_id(shared.id)
                .unwrap_or_else(|| panic!("{} has no taxonomy rule", shared.id));
            // Best practice without a success criterion: the register claims
            // none, although `SHARED_RULES` carries 1.3.1 for the output (#779).
            let expected = if shared.id == "headings/h1-missing" {
                None
            } else {
                Some(format!("WCAG {}", shared.criterion))
            };
            assert_eq!(
                rule.external_ref,
                expected.as_deref(),
                "{} resolves to {} with a different criterion",
                shared.id,
                rule.id
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

    const RULES_FTL_DE: &str = include_str!("../../locales/de/rules.ftl");
    const RULES_FTL_EN: &str = include_str!("../../locales/en/rules.ftl");

    /// The entries `RULES` needs in each `rules.ftl`.
    fn expected_entries() -> std::collections::BTreeSet<String> {
        RULES
            .iter()
            .flat_map(|rule| {
                let key = rule_text_key(rule.id);
                [
                    format!("{key}.user-impact"),
                    format!("{key}.technical-impact"),
                    key,
                ]
            })
            .collect()
    }

    #[test]
    fn rule_text_keys_are_distinct_fluent_identifiers() {
        let mut ids_by_key = std::collections::BTreeMap::new();
        for rule in RULES {
            let key = rule_text_key(rule.id);
            assert!(
                key.chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-'),
                "{key} (rule {}) is not a Fluent identifier",
                rule.id
            );
            if let Some(other) = ids_by_key.insert(key.clone(), rule.id) {
                panic!("rules {other} and {} share the key {key}", rule.id);
            }
        }
    }

    /// Every rule has its title and both impacts in German and English, and
    /// neither file holds a `rule-` message or attribute no rule reads. The
    /// rest of the file is the explanation texts: `output::explanations::tests`
    /// holds everything that is not a `rule-` message to that table.
    #[test]
    fn rules_ftl_holds_exactly_the_texts_of_the_catalogue() {
        let expected = expected_entries();
        for (locale, ftl) in [("de", RULES_FTL_DE), ("en", RULES_FTL_EN)] {
            let entries: std::collections::BTreeSet<String> = ftl_entries(ftl)
                .into_iter()
                .filter(|entry| entry.starts_with("rule-"))
                .collect();
            let missing: Vec<_> = expected.difference(&entries).collect();
            let unused: Vec<_> = entries.difference(&expected).collect();
            assert!(
                missing.is_empty(),
                "missing in {locale}/rules.ftl: {missing:?}"
            );
            assert!(
                unused.is_empty(),
                "in {locale}/rules.ftl without a rule: {unused:?}"
            );
        }
    }

    #[test]
    fn de_and_en_rules_ftl_have_identical_key_sets() {
        let de = ftl_entries(RULES_FTL_DE);
        let en = ftl_entries(RULES_FTL_EN);
        let de_only: Vec<_> = de.difference(&en).collect();
        let en_only: Vec<_> = en.difference(&de).collect();
        assert!(de_only.is_empty(), "only in de/rules.ftl: {de_only:?}");
        assert!(en_only.is_empty(), "only in en/rules.ftl: {en_only:?}");
    }

    /// The lookup the reports use resolves for every rule: a missing message
    /// would come back as its key.
    #[test]
    fn every_rule_text_resolves_in_both_languages() {
        for rule in RULES {
            let key = rule_text_key(rule.id);
            for en in [false, true] {
                for text in [
                    rule.title(en),
                    rule.user_impact(en),
                    rule.technical_impact(en),
                ] {
                    assert!(
                        !text.is_empty() && !text.starts_with(&key),
                        "{} (en = {en}): {text:?}",
                        rule.id
                    );
                }
            }
        }
    }

    #[test]
    fn english_rule_texts_have_no_german_umlauts() {
        for rule in RULES {
            for text in [
                rule.title(true),
                rule.user_impact(true),
                rule.technical_impact(true),
            ] {
                assert!(
                    !text.contains(['ä', 'ö', 'ü', 'Ä', 'Ö', 'Ü', 'ß']),
                    "{}: EN text has an umlaut: {text}",
                    rule.id
                );
            }
        }
    }

    /// Export every rule text of the catalogue into one data file for review:
    /// one record per rule id, language and field, sorted.
    ///
    /// Ignored by default; run on demand to (re)generate the file:
    ///   cargo test --lib export_all_rule_texts -- --ignored --nocapture
    #[test]
    #[ignore = "run on demand to regenerate reports/rule-texts.json"]
    fn export_all_rule_texts() {
        let mut records: Vec<(&str, &str, &str, String)> = Vec::new();
        for rule in RULES {
            for (lang, en) in [("de", false), ("en", true)] {
                records.push((rule.id, lang, "title", rule.title(en)));
                records.push((rule.id, lang, "user_impact", rule.user_impact(en)));
                records.push((rule.id, lang, "technical_impact", rule.technical_impact(en)));
            }
        }
        records.sort();
        let records: Vec<serde_json::Value> = records
            .into_iter()
            .map(|(rule_id, lang, field, text)| {
                serde_json::json!({
                    "rule_id": rule_id,
                    "lang": lang,
                    "field": field,
                    "text": text,
                })
            })
            .collect();

        let out_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("reports");
        std::fs::create_dir_all(&out_dir).expect("create reports/");
        let out_path = out_dir.join("rule-texts.json");
        let out = serde_json::to_string_pretty(&records).expect("serialize rule texts");
        std::fs::write(&out_path, out).expect("write reports/rule-texts.json");
        eprintln!(
            "Wrote {} rule text records to {}",
            records.len(),
            out_path.display()
        );
    }
}
