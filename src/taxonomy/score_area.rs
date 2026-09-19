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
    ("a11y.alt_text.missing", ImagesAlternativeText),
    ("a11y.area_alt.missing", ImagesAlternativeText),
    ("a11y.image_map_server_side.invalid", ImagesAlternativeText),
    ("a11y.input_image_alt.missing", ImagesAlternativeText),
    ("a11y.meter_name.missing", ImagesAlternativeText),
    ("a11y.object_alt.missing", ImagesAlternativeText),
    ("a11y.progressbar_name.missing", ImagesAlternativeText),
    ("a11y.svg_alt.missing", ImagesAlternativeText),
    ("a11y.media.alternative", ImagesAlternativeText),
    ("a11y.captions.missing", ImagesAlternativeText),
    ("a11y.media_alternative.missing", ImagesAlternativeText),
    ("a11y.frame_title.missing", ImagesAlternativeText),
    // ── 1.3.1 — Landmarks ──
    ("a11y.landmark_banner.missing", Landmarks),
    ("a11y.landmark_banner_duplicate.invalid", Landmarks),
    ("a11y.landmark_banner_nested.invalid", Landmarks),
    ("a11y.landmark_contentinfo_duplicate.invalid", Landmarks),
    ("a11y.landmark_contentinfo_nested.invalid", Landmarks),
    ("a11y.landmark_main.missing", Landmarks),
    ("a11y.landmark_main_duplicate.invalid", Landmarks),
    ("a11y.landmark_main_nested.invalid", Landmarks),
    ("a11y.landmark_region.missing", Landmarks),
    ("a11y.landmark_unique.invalid", Landmarks),
    // ── 1.3.1 — Formularsemantik ──
    ("a11y.form_field_group.missing", Forms),
    ("a11y.label_title_only.invalid", Forms),
    // ── 1.3.1 — ARIA-Missbrauch ──
    ("a11y.presentation_semantic_children.invalid", Aria),
    // ── 1.3.1 — Tabellen-, Listen- und Dokumentstruktur ──
    ("a11y.definition_list.invalid", Semantics),
    ("a11y.html_content_model.invalid", Semantics),
    ("a11y.list_structure.missing", Semantics),
    ("a11y.structure.missing", Semantics),
    ("a11y.table_header_data.missing", Semantics),
    ("a11y.table_headers_ref.invalid", Semantics),
    ("a11y.table_structure.invalid", Semantics),
    // ── 1.3.2 / 2.4.2 — Seitenaufbau ──
    ("a11y.meaningful_sequence.invalid", Landmarks),
    ("a11y.page_title.missing", Landmarks),
    // ── 1.3.4 / 1.4.x / 2.3.3 — Darstellung und Wahrnehmbarkeit ──
    ("a11y.orientation.restricted", Semantics),
    ("a11y.color.link_indicator", Semantics),
    ("a11y.contrast.weak", Semantics),
    ("a11y.resize_text.weak", Semantics),
    ("a11y.viewport_zoom.restricted", Semantics),
    ("a11y.background_audio.uncontrolled", Semantics),
    ("a11y.visual_presentation.weak", Semantics),
    ("a11y.reflow.missing", Semantics),
    ("a11y.non_text_contrast.weak", Semantics),
    ("a11y.text_spacing.clipped", Semantics),
    ("a11y.hover.content_visibility", Semantics),
    ("a11y.motion.reduced_motion", Semantics),
    // ── 1.3.5 / 3.2.2 / 3.3.x / 4.1.2 — Formulare ──
    ("a11y.input_purpose.missing", Forms),
    ("a11y.form_no_submit.missing", Forms),
    ("a11y.error_description.missing", Forms),
    ("a11y.error_id.missing_description", Forms),
    ("a11y.form_labels.missing", Forms),
    ("a11y.help.missing", Forms),
    ("a11y.redundant_entry.missing_reuse", Forms),
    ("a11y.combobox_options.missing", Forms),
    ("a11y.input_field_name.missing", Forms),
    ("a11y.toggle_field_name.missing", Forms),
    // ── 2.1.x / 2.5.x — Tastatur und Zeigerbedienung ──
    ("a11y.click_handler_keyboard.missing", Keyboard),
    ("a11y.focusable_no_role.invalid", Keyboard),
    ("a11y.keyboard.missing", Keyboard),
    ("a11y.keyboard_trap.risk", Keyboard),
    ("a11y.pointer_gestures.missing_alternative", Keyboard),
    ("a11y.pointer_cancellation.invalid", Keyboard),
    ("a11y.motion_actuation.missing_alternative", Keyboard),
    ("a11y.target_size_enhanced.small", Keyboard),
    ("a11y.target_size_minimum.small", Keyboard),
    // ── 2.2.x — Zeitlimits ──
    ("a11y.timing.unadjustable", Semantics),
    ("a11y.pause_stop_hide.no_control", Semantics),
    ("a11y.timing.essential", Semantics),
    ("a11y.interruptions.uncontrollable", Semantics),
    ("a11y.re_authenticate.data_loss", Semantics),
    ("a11y.timeouts.unwarned", Semantics),
    // ── 2.4.1 — Sprungmechanismen ──
    ("a11y.bypass_blocks.missing", Landmarks),
    ("a11y.bypass_main_landmark.missing", Landmarks),
    ("a11y.skip_link.missing", Landmarks),
    // ── 2.4.3 / 2.4.7 / 2.4.11 / 2.4.12 / 3.2.1 — Fokus ──
    ("a11y.focus_order.weak", FocusManagement),
    ("a11y.focus_indicator_suppressed.invalid", FocusManagement),
    ("a11y.focus_visible.missing", FocusManagement),
    ("a11y.focus_not_obscured_minimum.hidden", FocusManagement),
    ("a11y.focus_not_obscured_enhanced.hidden", FocusManagement),
    ("a11y.on_focus.risk", FocusManagement),
    // ── 2.4.4 / 2.4.8 / 2.4.9 — Linktexte und Orientierung ──
    ("a11y.link_purpose.weak", Semantics),
    ("a11y.link_purpose_context.weak", Semantics),
    ("a11y.link_purpose_only.weak", Semantics),
    ("a11y.location.missing", Semantics),
    // ── 2.4.6 / 2.4.10 — Überschriften ──
    ("a11y.headings.missing", HeadingStructure),
    ("a11y.section_headings.missing", HeadingStructure),
    // ── 3.1.x — Sprache ──
    ("a11y.language.missing", Semantics),
    ("a11y.language_mismatch.invalid", Semantics),
    ("a11y.language_valid.invalid", Semantics),
    ("a11y.unusual_words.missing_definition", Semantics),
    ("a11y.abbreviations.missing", Semantics),
    // ── 3.2.2 — Kontextwechsel ──
    ("a11y.on_input.risk", Semantics),
    // ── 1.3.6 / 2.5.3 / 4.1.x — Name, Rolle, Wert ──
    ("a11y.identify_purpose.missing", Aria),
    ("a11y.label_in_name.invalid", Aria),
    ("a11y.duplicate_id_aria.invalid", Aria),
    ("a11y.parsing.invalid", Aria),
    ("a11y.aria_allowed_attr.invalid", Aria),
    ("a11y.aria_attr_name.invalid", Aria),
    ("a11y.aria_hidden_focus.invalid", Aria),
    ("a11y.aria_prohibited_attr.invalid", Aria),
    ("a11y.aria_relationships.invalid", Aria),
    ("a11y.aria_required_attr.missing", Aria),
    ("a11y.aria_required_children.missing", Aria),
    ("a11y.aria_required_parent.invalid", Aria),
    ("a11y.aria_roles.invalid", Aria),
    ("a11y.aria_valid_attr_value.invalid", Aria),
    ("a11y.command_name.missing", Aria),
    ("a11y.control_name.missing", Aria),
    ("a11y.dialog_missing_name.invalid", Aria),
    ("a11y.dialog_name.missing", Aria),
    ("a11y.frame_tested.cross_origin", Aria),
    ("a11y.interactive_name.missing", Aria),
    ("a11y.link_as_button.invalid", Aria),
    ("a11y.modern_attributes.invalid", Aria),
    ("a11y.name_role.missing", Aria),
    ("a11y.redundant_role.invalid", Aria),
    ("a11y.summary_name.missing", Aria),
    ("a11y.tablist_tabpanel.missing", Aria),
    ("a11y.treeitem_name.missing", Aria),
    ("a11y.status_messages.broken", Aria),
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
