//! Zuordnung von Accessibility-Regeln zu den Bereichen des Score-Breakdowns.
//!
//! Die Bereiche sind die kundenseitigen Kategorien in
//! `summary.accessibility_score_breakdown[]`. Die Zuordnung ist bewusst eine
//! explizite Tabelle über `rule_id` statt einer Ableitung aus Regeltexten:
//! Textsuche über `rule_id + title + description + subcategory` hat in
//! ausgelieferten Reports wiederholt falsch zugeordnet — `"Querformat"` traf
//! `form`, `"conformance"` traf `form`, `"div.alt-service-hero-card"` traf
//! `alt`, und das geteilte Subkategorie-Label `"Navigation & Operation"`
//! leitete rund 26 fachfremde Regeln nach `Landmarks` um (Plan 38).
//!
//! Jede neue Regelmeldung konnte die Zuordnung still neu brechen, und eine
//! falsche Zuordnung ist im Report unsichtbar — sie erscheint als plausible
//! Treiberzeile unter der falschen Überschrift.

use super::dimensions::Subcategory;

/// Ein Bereich des Accessibility-Score-Breakdowns.
///
/// Die Labels sind Teil des JSON-Vertrags
/// (`docs/OUTPUT_CONTRACT.md#accessibility-score-breakdown`) und dürfen sich
/// nicht ändern, ohne den Vertrag mitzuziehen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScoreArea {
    Semantics,
    Forms,
    Keyboard,
    FocusManagement,
    ImagesAlternativeText,
    Aria,
    HeadingStructure,
    Landmarks,
}

impl ScoreArea {
    pub const ALL: [ScoreArea; 8] = [
        ScoreArea::Semantics,
        ScoreArea::Forms,
        ScoreArea::Keyboard,
        ScoreArea::FocusManagement,
        ScoreArea::ImagesAlternativeText,
        ScoreArea::Aria,
        ScoreArea::HeadingStructure,
        ScoreArea::Landmarks,
    ];

    /// Kanonisch englisches Label — so steht es im JSON.
    pub fn label(self) -> &'static str {
        match self {
            ScoreArea::Semantics => "Semantics",
            ScoreArea::Forms => "Forms",
            ScoreArea::Keyboard => "Keyboard",
            ScoreArea::FocusManagement => "Focus management",
            ScoreArea::ImagesAlternativeText => "Images / alternative text",
            ScoreArea::Aria => "ARIA",
            ScoreArea::HeadingStructure => "Heading structure",
            ScoreArea::Landmarks => "Landmarks / page structure",
        }
    }

    /// Umkehrung von [`ScoreArea::label`] für Konsumenten, die den Bereich nur
    /// als JSON-Label kennen (der PDF-Bericht liest den Breakdown so).
    pub fn from_label(label: &str) -> Option<ScoreArea> {
        ScoreArea::ALL.into_iter().find(|a| a.label() == label)
    }

    /// Label in der Berichtssprache. Nur für die Präsentation; das JSON trägt
    /// immer [`ScoreArea::label`] (#406).
    pub fn localized_label(self, en: bool) -> &'static str {
        if en {
            return self.label();
        }
        match self {
            ScoreArea::Semantics => "Semantik",
            ScoreArea::Forms => "Formulare",
            ScoreArea::Keyboard => "Tastatur",
            ScoreArea::FocusManagement => "Fokusführung",
            ScoreArea::ImagesAlternativeText => "Bilder / Alternativtexte",
            ScoreArea::Aria => "ARIA",
            ScoreArea::HeadingStructure => "Überschriftenstruktur",
            ScoreArea::Landmarks => "Landmarken / Seitenstruktur",
        }
    }
}

use ScoreArea::*;

/// Vollständige Zuordnung aller Accessibility-Regeln.
///
/// Der Inventar-Test `every_accessibility_rule_has_a_score_area` schlägt fehl,
/// sobald eine Regel im Register keinen Eintrag hier hat — eine neue Regel
/// kann also nicht still in den Sammelbereich fallen.
///
/// `Semantics` ist dabei der bewusste Sammelbereich für inhaltliche Kriterien,
/// die keiner der sieben spezifischen Kategorien entsprechen: Kontrast und
/// visuelle Darstellung (1.4.x), Zeitlimits (2.2.x), Sprache (3.1.x),
/// Tabellen-/Listensemantik und Linktext-Qualität. Dass diese Gruppe unter
/// „Semantics" firmiert, ist eine Schwäche des Bereichsschnitts, keine
/// Fehlzuordnung — siehe Plan 38, „Offen".
static RULE_AREAS: &[(&str, ScoreArea)] = &[
    // ── 1.1.x / 1.2.x — Nicht-Text-Inhalte und Medien-Alternativen ──
    ("image-alt", ImagesAlternativeText),
    ("images/alt-missing", ImagesAlternativeText),
    ("images/alt-suspicious", ImagesAlternativeText),
    ("images/area-alt-missing", ImagesAlternativeText),
    ("images/server-side-map", ImagesAlternativeText),
    ("images/input-alt-missing", ImagesAlternativeText),
    ("objects/alt-missing", ImagesAlternativeText),
    ("svg/name-missing", ImagesAlternativeText),
    ("video-caption", ImagesAlternativeText),
    ("media-alt", ImagesAlternativeText),
    ("frames/name-missing", ImagesAlternativeText),
    // Display-mode convention (#653): the visualisation's text layer.
    ("display/text-not-visible", ImagesAlternativeText),
    ("viz/text-missing", ImagesAlternativeText),
    ("display/text-hidden", ImagesAlternativeText),
    ("viz/caption-missing", ImagesAlternativeText),
    ("viz/table-missing", Semantics),
    // Display-mode convention (#653): control over visual/moving content,
    // with the other 2.2.x rules in the collection area.
    ("display/toggle-missing", Semantics),
    ("display/init-missing", Semantics),
    ("display/text-media-visible", Semantics),
    ("viz/static-missing", Semantics),
    // ── 1.3.1 — Landmarks ──
    ("landmarks/banner-missing", Landmarks),
    ("landmarks/contentinfo-missing", Landmarks),
    ("landmarks/navigation-missing", Landmarks),
    ("landmarks/banner-duplicate", Landmarks),
    ("landmarks/contentinfo-duplicate", Landmarks),
    ("landmarks/main-duplicate", Landmarks),
    ("landmarks/not-top-level", Landmarks),
    ("landmarks/content-outside", Landmarks),
    ("landmarks/not-unique", Landmarks),
    // ── 1.3.1 — Formularsemantik ──
    ("forms/group-missing", Forms),
    ("forms/title-only-label", Forms),
    // ── 1.3.1 — ARIA-Missbrauch ──
    // ── 1.3.1 — Tabellen-, Listen- und Dokumentstruktur ──
    ("lists/term-without-definition", Semantics),
    ("html-content-model", Semantics),
    ("lists/invalid-structure", Semantics),
    ("lists/empty", Semantics),
    ("lists/item-outside-list", Semantics),
    ("tables/data-undetermined", Semantics),
    ("tables/header-without-data", Semantics),
    ("tables/header-missing", Semantics),
    ("tables/headers-attr-invalid", Semantics),
    ("tables/name-missing", Semantics),
    ("tables/presentational-with-headers", Semantics),
    // ── 1.3.2 / 2.4.2 — Seitenaufbau ──
    ("order/visual-mismatch", Landmarks),
    ("document-title", Landmarks),
    ("document/title-empty", Landmarks),
    ("document/title-missing", Landmarks),
    // ── 1.3.4 / 1.4.x / 2.3.3 — Darstellung und Wahrnehmbarkeit ──
    ("orientation/root-rotated", Semantics),
    ("orientation/content-hidden", Semantics),
    ("color/link-indistinct", Semantics),
    ("contrast/text-insufficient", Semantics),
    ("contrast/text-undetermined", Semantics),
    ("contrast/text-enhanced", Semantics),
    ("zoom/viewport-locked", Semantics),
    ("zoom/viewport-missing", Semantics),
    ("zoom/viewport-scale-limited", Semantics),
    ("media/audio-autoplay", Semantics),
    ("text/justified", Semantics),
    ("text/line-height-tight", Semantics),
    ("visual-presentation", Semantics),
    ("css-overflow-hidden", Semantics),
    ("contrast/control-insufficient", Semantics),
    ("contrast/control-undetermined", Semantics),
    ("text-spacing/clipped", Semantics),
    ("patterns/tooltip-unreferenced", Semantics),
    ("motion/reduced-motion-ignored", Semantics),
    // ── 1.3.5 / 3.2.2 / 3.3.x / 4.1.2 — Formulare ──
    ("forms/autocomplete-invalid", Forms),
    ("forms/purpose-missing", Forms),
    ("forms/no-submit", Forms),
    ("forms/error-unidentified", Forms),
    ("forms/group-name-missing", Forms),
    ("forms/instructions-missing", Forms),
    ("forms/label-missing", Forms),
    ("forms/placeholder-as-label", Forms),
    ("forms/required-unmarked", Forms),
    ("manual/help", Forms),
    ("forms/redundant-entry", Forms),
    ("auth/paste-blocked", Forms),
    ("auth/captcha", Forms),
    ("aria/combobox-popup-missing", Forms),
    // ── 2.1.x / 2.5.x — Tastatur und Zeigerbedienung ──
    ("keyboard/click-handler-not-focusable", Keyboard),
    ("keyboard/pointer-only", Keyboard),
    ("keyboard/focusable-no-role", Keyboard),
    ("keyboard/interactive-not-focusable", Keyboard),
    ("keyboard/scrollable-region-not-focusable", Keyboard),
    ("keyboard-trap", Keyboard),
    ("manual/pointer-gestures", Keyboard),
    ("pointer-cancellation", Keyboard),
    ("manual/motion-actuation", Keyboard),
    ("targets/size-enhanced", Keyboard),
    ("targets/size", Keyboard),
    // ── 2.2.x — Zeitlimits ──
    ("meta-refresh", Semantics),
    ("timing/meta-refresh", Semantics),
    ("motion/infinite-animation", Semantics),
    ("manual/no-timing", Semantics),
    ("manual/interruptions", Semantics),
    ("manual/re-authenticating", Semantics),
    ("timeouts", Semantics),
    // ── 2.4.1 — Sprungmechanismen ──
    ("headings/none", Landmarks),
    ("landmarks/main-missing", Landmarks),
    ("keyboard/skip-link-missing", Landmarks),
    // ── 2.4.3 / 2.4.7 / 2.4.11 / 2.4.12 / 3.2.1 — Fokus ──
    ("dialog/focusable-missing", FocusManagement),
    ("keyboard/positive-tabindex", FocusManagement),
    ("focus/outline-removed", FocusManagement),
    ("focus-visible", FocusManagement),
    ("focus/obscured", FocusManagement),
    ("focus/obscured-enhanced", FocusManagement),
    ("focus/indicator-missing", FocusManagement),
    ("context/autofocus", FocusManagement),
    ("context/on-focus", FocusManagement),
    // ── 2.4.4 / 2.4.8 / 2.4.9 — Linktexte und Orientierung ──
    ("links/generic-name", Semantics),
    ("links/single-character-name", Semantics),
    ("links/url-name", Semantics),
    ("links/purpose-link-only", Semantics),
    ("links/new-window-unannounced", Semantics),
    ("navigation/location-missing", Semantics),
    // ── 2.4.6 / 2.4.10 — Überschriften ──
    ("headings/section-without-heading", HeadingStructure),
    // 1.3.1, aber Ueberschriften: Bis Plan 56 liefen sie ueber
    // `a11y.structure.missing` und damit unter `Semantics`.
    ("headings/skip-level", HeadingStructure),
    ("headings/empty", HeadingStructure),
    ("headings/h1-missing", HeadingStructure),
    ("headings/h1-multiple", HeadingStructure),
    // ── 3.1.x — Sprache ──
    ("document/lang-missing", Semantics),
    ("html-has-lang", Semantics),
    ("document/lang-mismatch", Semantics),
    ("language/part-undetermined", Semantics),
    ("language/part-unmarked", Semantics),
    ("document/lang-invalid", Semantics),
    ("manual/unusual-words", Semantics),
    ("language/abbreviation-unexpanded", Semantics),
    // ── 3.2.2 — Kontextwechsel ──
    ("context/on-input", Semantics),
    // ── 1.3.6 / 2.5.3 / 4.1.x — Name, Rolle, Wert ──
    ("label-in-name/mismatch", Aria),
    ("aria/owns-conflict", Aria),
    ("ids/duplicate", Aria),
    ("aria/attribute-not-allowed", Aria),
    ("aria/attribute-unknown", Aria),
    ("keyboard/hidden-focusable", Aria),
    ("aria/attribute-prohibited", Aria),
    ("aria/reference-missing", Aria),
    ("aria/required-attribute-missing", Aria),
    ("aria/required-children-missing", Aria),
    ("aria/required-parent-missing", Aria),
    ("aria/role-abstract", Aria),
    ("aria/role-invalid", Aria),
    ("aria/attribute-value-invalid", Aria),
    ("buttons/name-missing", Aria),
    ("links/name-missing", Aria),
    ("dialog/name-missing", Aria),
    ("frames/cross-origin", Aria),
    ("names/required-missing", Aria),
    ("names/symbol-only", Aria),
    ("names/title-only", Aria),
    ("description-duplicates-name", Aria),
    ("links/used-as-button", Aria),
    ("inert/dialog-inert", Aria),
    ("modern-attribute-misuse", Aria),
    ("popover/target-invalid", Aria),
    ("popover/target-missing", Aria),
    ("aria/role-redundant", Aria),
    ("lists/role-redundant", Aria),
    ("summary/name-missing", Aria),
    ("aria/tabpanel-missing", Aria),
    ("aria/tab-selected-missing", Aria),
    ("patterns/accordion-controls-missing", Aria),
    ("dialog/modal-unmarked", Aria),
    ("status/live-overridden", Aria),
];

/// Bereich einer Regel über ihre `rule_id`.
pub fn score_area_for_rule(rule_id: &str) -> Option<ScoreArea> {
    RULE_AREAS
        .iter()
        .find(|(id, _)| *id == rule_id)
        .map(|(_, area)| *area)
}

/// Grober Rückfall über die Subkategorie — nur für Befunde, deren `rule_id`
/// gar nicht im Regelregister steht (z. B. dynamisch erzeugte Regeln). Für
/// registrierte Regeln greift immer [`score_area_for_rule`]; der
/// Inventar-Test stellt das sicher.
pub fn score_area_for_subcategory(subcategory: Subcategory) -> ScoreArea {
    match subcategory {
        Subcategory::ContentAlternatives => ImagesAlternativeText,
        Subcategory::FormsInteraction => Forms,
        Subcategory::TechnicalRobustness => Aria,
        Subcategory::NavigationInteraction => Keyboard,
        _ => Semantics,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::taxonomy::dimensions::Dimension;
    use crate::taxonomy::rules::RULES;

    /// Plan 38: die Zuordnung muss total sein. Ohne diesen Test fällt eine neu
    /// registrierte Accessibility-Regel still in den Subkategorie-Rückfall und
    /// landet dort plausibel aussehend im falschen Bereich.
    #[test]
    fn every_accessibility_rule_has_a_score_area() {
        let missing: Vec<&str> = RULES
            .iter()
            .filter(|r| r.dimension == Dimension::Accessibility)
            .map(|r| r.id)
            .filter(|id| score_area_for_rule(id).is_none())
            .collect();

        assert!(
            missing.is_empty(),
            "these accessibility rules have no score area in RULE_AREAS: {missing:#?}",
        );
    }

    /// Umgekehrt: kein Eintrag darf auf eine Regel zeigen, die es nicht mehr
    /// gibt — sonst bleibt beim Umbenennen einer Regel eine tote Zeile stehen
    /// und die echte Regel fällt in den Rückfall.
    #[test]
    fn score_area_table_has_no_stale_entries() {
        let stale: Vec<&str> = RULE_AREAS
            .iter()
            .map(|(id, _)| *id)
            .filter(|id| !RULES.iter().any(|r| r.id == *id))
            .collect();

        assert!(
            stale.is_empty(),
            "these RULE_AREAS entries reference unknown rules: {stale:#?}",
        );
    }

    #[test]
    fn score_area_table_has_no_duplicates() {
        let mut seen = std::collections::HashSet::new();
        let dupes: Vec<&str> = RULE_AREAS
            .iter()
            .map(|(id, _)| *id)
            .filter(|id| !seen.insert(*id))
            .collect();

        assert!(dupes.is_empty(), "duplicate RULE_AREAS entries: {dupes:#?}");
    }

    /// Die Labels stehen im JSON-Vertrag.
    #[test]
    fn area_labels_match_the_output_contract() {
        let labels: Vec<&str> = ScoreArea::ALL.iter().map(|a| a.label()).collect();
        assert_eq!(
            labels,
            vec![
                "Semantics",
                "Forms",
                "Keyboard",
                "Focus management",
                "Images / alternative text",
                "ARIA",
                "Heading structure",
                "Landmarks / page structure",
            ],
        );
    }
}
