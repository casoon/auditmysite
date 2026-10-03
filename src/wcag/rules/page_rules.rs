//! Table-driven catalog of page-level WCAG rule checks (#334).
//!
//! Every check that has the shape `async fn(&Page) -> Vec<Violation>` lives
//! here as a [`PageRuleEntry`]. `pipeline::run_rules` iterates this table
//! instead of hand-listing each call. Two checks remain inline because they
//! don't fit the table shape:
//!
//! - **3.1.1 lang** — verifying subtraction over an existing violation;
//!   reads `html[lang]` from the DOM and removes the violation if present.
//! - **1.4.3 contrast** — needs the AX tree, the configured WCAG level,
//!   and the captured screenshot in addition to the page.
//!
//! Order is significant: it matches the previous hand-written order in
//! `run_rules` so finding-vector layout stays identical. Each rule's
//! `min_level` gates whether it runs at the configured `WcagLevel`.
//!
//! Each entry pairs the function with a `rule_id` (used for logging only;
//! the actual rule strings are produced inside the check functions
//! themselves) and a `name` for the `"Found N <name> violations"` log line.

use chromiumoxide::Page;
use futures::future::BoxFuture;

use crate::cli::WcagLevel;
use crate::wcag::Violation;

use super::{
    check_accessible_authentication_with_page, check_display_modes_with_page,
    check_focus_not_obscured_enhanced_with_page, check_focus_not_obscured_minimum_with_page,
    check_frame_tested_with_page, check_meaningful_sequence_with_page,
    check_media_alternative_with_page, check_modern_attributes_with_page,
    check_motion_actuation_with_page, check_no_interruptions_with_page, check_no_timing_with_page,
    check_non_text_contrast_css_with_page, check_on_input_with_page, check_orientation_with_page,
    check_page_titled_with_page, check_pause_stop_hide_with_page,
    check_pointer_cancellation_with_page, check_pointer_gestures_with_page,
    check_re_authenticate_with_page, check_redundant_role_with_page,
    check_same_origin_iframes_with_page, check_scrollable_region_focusable_with_page,
    check_target_size_enhanced_with_page, check_target_size_minimum_with_page,
    check_text_spacing_with_page, check_timeouts_with_page, check_timing_with_page,
    check_use_of_color_with_page, check_video_caption_tracks_with_page,
    check_visual_presentation_with_page,
};

/// One row of the page-rule catalog.
pub struct PageRuleEntry {
    /// Stable identifier used in log lines only (not the rule string that
    /// appears in findings — that comes from each check's own metadata).
    pub rule_id: &'static str,
    /// Short label for the `"Found N <name> violations"` log line, to match
    /// the previous inline wording exactly.
    pub name: &'static str,
    /// Lowest WCAG level at which this rule runs. The current configured
    /// level must be `>= min_level` for the rule to execute.
    pub min_level: WcagLevel,
    /// Boxed async check. Non-capturing closures coerce to this fn pointer.
    pub check_fn: for<'a> fn(&'a Page) -> BoxFuture<'a, Vec<Violation>>,
}

/// Page-rule table. Order is intentional and matches the previous inline
/// `run_rules` order so finding emission stays stable.
pub const PAGE_RULES: &[PageRuleEntry] = &[
    // ── Level A ───────────────────────────────────────────────────────────────
    PageRuleEntry {
        rule_id: "4.1.2/frame-tested",
        name: "frame-tested",
        min_level: WcagLevel::A,
        check_fn: |p| Box::pin(check_frame_tested_with_page(p)),
    },
    PageRuleEntry {
        rule_id: "1.2.2/video-caption-track",
        name: "video-caption-track",
        min_level: WcagLevel::A,
        check_fn: |p| Box::pin(check_video_caption_tracks_with_page(p)),
    },
    PageRuleEntry {
        rule_id: "iframe/same-origin-content",
        name: "same-origin iframe content",
        min_level: WcagLevel::A,
        check_fn: |p| Box::pin(check_same_origin_iframes_with_page(p)),
    },
    PageRuleEntry {
        rule_id: "2.4.2/document-title",
        name: "document-title",
        min_level: WcagLevel::A,
        check_fn: |p| Box::pin(check_page_titled_with_page(p)),
    },
    PageRuleEntry {
        rule_id: "2.1.1/scrollable-region-focusable",
        name: "scrollable-region-focusable",
        min_level: WcagLevel::A,
        check_fn: |p| Box::pin(check_scrollable_region_focusable_with_page(p)),
    },
    PageRuleEntry {
        rule_id: "3.3.8/accessible-authentication",
        name: "accessible-authentication",
        min_level: WcagLevel::AA,
        check_fn: |p| Box::pin(check_accessible_authentication_with_page(p)),
    },
    // Only the page-wide note on script-driven time limits since #697;
    // `<meta http-equiv="refresh">` runs as `timing/meta-refresh`.
    PageRuleEntry {
        rule_id: "2.2.1/timing-adjustable",
        name: "timing-adjustable note",
        min_level: WcagLevel::A,
        check_fn: |p| Box::pin(check_timing_with_page(p)),
    },
    PageRuleEntry {
        rule_id: "1.3.2/meaningful-sequence",
        name: "meaningful-sequence (CSS order)",
        min_level: WcagLevel::A,
        check_fn: |p| Box::pin(check_meaningful_sequence_with_page(p)),
    },
    PageRuleEntry {
        rule_id: "4.1.2/modern-attributes",
        name: "modern interaction attributes",
        min_level: WcagLevel::A,
        check_fn: |p| Box::pin(check_modern_attributes_with_page(p)),
    },
    PageRuleEntry {
        rule_id: "1.4.1/use-of-color",
        name: "use-of-color",
        min_level: WcagLevel::A,
        check_fn: |p| Box::pin(check_use_of_color_with_page(p)),
    },
    PageRuleEntry {
        rule_id: "3.2.2/on-input",
        name: "on-input context change",
        min_level: WcagLevel::A,
        check_fn: |p| Box::pin(check_on_input_with_page(p)),
    },
    PageRuleEntry {
        rule_id: "4.1.2/redundant-role",
        name: "redundant role",
        min_level: WcagLevel::A,
        check_fn: |p| Box::pin(check_redundant_role_with_page(p)),
    },
    // Display-mode convention (#653): all five `display/*` ids from one
    // DOM pass. Best-practice rules, not WCAG requirements; no-op on pages
    // without `figure[data-viz]`/`html[data-display]`.
    PageRuleEntry {
        rule_id: "display/modes",
        name: "display-mode convention",
        min_level: WcagLevel::A,
        check_fn: |p| Box::pin(check_display_modes_with_page(p)),
    },
    // 2.5.1-2.5.4 are official WCAG 2.1 Level A criteria (only 2.5.5/2.5.6 are
    // AAA) — previously misclassified as AAA here and in each rule's own
    // RULE_META, which meant they silently never ran at the default AA level.
    PageRuleEntry {
        rule_id: "2.5.1/pointer-gestures",
        name: "pointer-gestures",
        min_level: WcagLevel::A,
        check_fn: |p| Box::pin(check_pointer_gestures_with_page(p)),
    },
    PageRuleEntry {
        rule_id: "2.5.2/pointer-cancellation",
        name: "pointer-cancellation",
        min_level: WcagLevel::A,
        check_fn: |p| Box::pin(check_pointer_cancellation_with_page(p)),
    },
    PageRuleEntry {
        rule_id: "2.5.4/motion-actuation",
        name: "motion-actuation",
        min_level: WcagLevel::A,
        check_fn: |p| Box::pin(check_motion_actuation_with_page(p)),
    },
    PageRuleEntry {
        rule_id: "2.2.2/pause-stop-hide",
        name: "pause-stop-hide",
        min_level: WcagLevel::A,
        check_fn: |p| Box::pin(check_pause_stop_hide_with_page(p)),
    },
    // ── Level AA and above ────────────────────────────────────────────────────
    PageRuleEntry {
        rule_id: "1.3.4/orientation",
        name: "orientation",
        min_level: WcagLevel::AA,
        check_fn: |p| Box::pin(check_orientation_with_page(p)),
    },
    PageRuleEntry {
        rule_id: "1.4.11/non-text-contrast-css",
        name: "CSS non-text contrast",
        min_level: WcagLevel::AA,
        check_fn: |p| Box::pin(check_non_text_contrast_css_with_page(p)),
    },
    PageRuleEntry {
        rule_id: "2.5.8/target-size-minimum",
        name: "target-size-minimum",
        min_level: WcagLevel::AA,
        check_fn: |p| Box::pin(check_target_size_minimum_with_page(p)),
    },
    PageRuleEntry {
        rule_id: "1.4.12/text-spacing",
        name: "text-spacing",
        min_level: WcagLevel::AA,
        check_fn: |p| Box::pin(check_text_spacing_with_page(p)),
    },
    PageRuleEntry {
        rule_id: "2.4.11/focus-not-obscured-minimum",
        name: "focus-not-obscured-minimum",
        min_level: WcagLevel::AA,
        check_fn: |p| Box::pin(check_focus_not_obscured_minimum_with_page(p)),
    },
    // ── Level AAA only ────────────────────────────────────────────────────────
    PageRuleEntry {
        rule_id: "1.2.8/media-alternative",
        name: "media-alternative",
        min_level: WcagLevel::AAA,
        check_fn: |p| Box::pin(check_media_alternative_with_page(p)),
    },
    PageRuleEntry {
        rule_id: "1.4.8/visual-presentation",
        name: "visual-presentation",
        min_level: WcagLevel::AAA,
        check_fn: |p| Box::pin(check_visual_presentation_with_page(p)),
    },
    PageRuleEntry {
        rule_id: "2.2.3/no-timing",
        name: "no-timing",
        min_level: WcagLevel::AAA,
        check_fn: |p| Box::pin(check_no_timing_with_page(p)),
    },
    PageRuleEntry {
        rule_id: "2.2.4/no-interruptions",
        name: "no-interruptions",
        min_level: WcagLevel::AAA,
        check_fn: |p| Box::pin(check_no_interruptions_with_page(p)),
    },
    PageRuleEntry {
        rule_id: "2.2.5/re-authenticate",
        name: "re-authenticate",
        min_level: WcagLevel::AAA,
        check_fn: |p| Box::pin(check_re_authenticate_with_page(p)),
    },
    PageRuleEntry {
        rule_id: "2.2.6/timeouts",
        name: "timeouts",
        min_level: WcagLevel::AAA,
        check_fn: |p| Box::pin(check_timeouts_with_page(p)),
    },
    PageRuleEntry {
        rule_id: "2.5.5/target-size-enhanced",
        name: "target-size-enhanced",
        min_level: WcagLevel::AAA,
        check_fn: |p| Box::pin(check_target_size_enhanced_with_page(p)),
    },
    PageRuleEntry {
        rule_id: "2.4.12/focus-not-obscured-enhanced",
        name: "focus-not-obscured-enhanced",
        min_level: WcagLevel::AAA,
        check_fn: |p| Box::pin(check_focus_not_obscured_enhanced_with_page(p)),
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_rules_table_is_non_empty() {
        assert!(!PAGE_RULES.is_empty());
    }

    #[test]
    fn page_rules_have_unique_rule_ids() {
        let mut ids: Vec<&'static str> = PAGE_RULES.iter().map(|r| r.rule_id).collect();
        ids.sort_unstable();
        let before = ids.len();
        ids.dedup();
        assert_eq!(before, ids.len(), "duplicate rule_id in PAGE_RULES");
    }

    #[test]
    fn level_a_filter_includes_only_level_a_rules() {
        let count = PAGE_RULES
            .iter()
            .filter(|r| WcagLevel::A >= r.min_level)
            .count();
        // Level-A page rules in the table; tightening this catches
        // accidental reclassification of a rule's min_level.
        // 7 original + 4 DOM parity checks + parsing (AAA→A)
        // + aria-valid-attr-value + iframe-content + modern attributes = 15
        // + positive-tabindex + on-focus + on-input + image-input-object-alt
        // + server-side-image-map + td-headers-attr + language-extended
        // + invalid-role + checked-state + aria-allowed-attr
        // + invalid-aria-attribute-name (#QA-030) = 26
        // + tab-selected-state (#QA-031) = 27
        // + pointer-gestures/pointer-cancellation/label-in-name/motion-actuation
        // (2.5.1-2.5.4 are Level A per WCAG 2.1, were misclassified as AAA) = 31
        // + redundant-entry (3.3.7, WCAG 2.2 A) = 32
        // + meaningful-sequence (1.3.2, WCAG 2.1 A) = 33
        // + pause-stop-hide (2.2.2, WCAG 2.1 A) = 34
        // + aria-valid-attr DOM supplement (#567) = 35
        // + video-caption-track (1.2.2, DOM+network caption-track deepening,
        //   #video-caption-checks) = 36
        // + redundant-role + link-as-button (plan/18, best-practice ARIA
        //   hygiene checks, both Level A) = 38
        // - parsing (duplicate-id) und positive-tabindex: laufen seit der
        //   Umstellung als `ids/duplicate` bzw. `keyboard/positive-tabindex`
        //   im geteilten Bestand (siehe `wcag::shared`) = 36
        // + form-field-group for named checkbox sets: moved from the AX tree to
        //   the DOM, the tree carries no `name` attribute (#643) = 37
        // - landmark-dom: duplicated the AX tree's landmark-unique and
        //   landmark-main-present, each defect counted twice = 36
        // + value-now: aria-valuenow moved from the AX tree to the DOM, CDP
        //   has no `valuenow` property (#656) = 37
        // + display-mode convention (`display/*`, #653, best-practice) = 38
        // - aria-hidden-focus, aria-valid-attr DOM supplement, checked-state
        //   und value-now: laufen als `keyboard/hidden-focusable`,
        //   `aria/reference-missing` und `aria/required-attribute-missing`
        //   im geteilten Bestand (#690) = 34
        // + scrollable-region-focusable (2.1.1, #717) = 35
        // - aria-prohibited-attr, invalid-role, aria-allowed-attr,
        //   invalid-aria-attribute-name, tab-selected-state und
        //   aria-valid-attr-value: laufen als `aria/*` im geteilten Bestand
        //   (#691) = 29
        // - label-in-name: laeuft als `label-in-name/mismatch` im geteilten
        //   Bestand (#692) = 28
        // - form-no-submit, form-field-group (Kontrollkaestchen),
        //   redundant-entry und on-focus: laufen als `forms/*` und
        //   `context/*` im geteilten Bestand (#693) = 24
        // - click-handler und link-as-button: laufen als
        //   `keyboard/click-handler-not-focusable` und `links/used-as-button`
        //   im geteilten Bestand (#695) = 22
        // - frame-title und server-side-image-map: laufen als
        //   `frames/name-missing` und `images/server-side-map` im geteilten
        //   Bestand (#696); image-input-object-alt bleibt fuer `<area>` als
        //   area-alt = 20
        // - presentation-semantic-children (gestrichen), td-headers-attr und
        //   language-extended: laufen als `tables/headers-attr-invalid` und
        //   `document/lang-mismatch` im geteilten Bestand (#697); meta-refresh
        //   wird als `timing/meta-refresh` geteilt, der 2.2.1-Vermerk bleibt
        //   als timing-adjustable = 17
        // - area-alt: laeuft als `images/area-alt-missing` im geteilten
        //   Bestand, seit a11y-rules 0.19 auch mit Stilen = 16
        assert_eq!(count, 16);
    }

    #[test]
    fn level_aa_filter_includes_aa_plus_a_rules() {
        let count = PAGE_RULES
            .iter()
            .filter(|r| WcagLevel::AA >= r.min_level)
            .count();
        // 33 A + 4 original AA + 2 viewport zoom checks (#QA-030)
        // + target-size-minimum (2.5.8, WCAG 2.2 AA)
        // + text-spacing (1.4.12, WCAG 2.1 AA) = 41.
        // + non-text-contrast-css (1.4.11, replaces the vacuous AX-tree-only
        //   check_non_text_contrast — see non_text_contrast_css.rs) = 42.
        // + focus-not-obscured-minimum (2.4.11, WCAG 2.2 AA) = 43.
        // + pause-stop-hide (2.2.2, WCAG 2.1 A, counted here too since AA >= A) = 44.
        // + conservative language-of-parts heuristic (3.1.2) = 45.
        // + aria-valid-attr DOM supplement (#567, Level A, counted here too since AA >= A) = 46.
        // + video-caption-track (1.2.2, Level A, counted here too since AA >= A) = 47.
        // + redundant-role + link-as-button (plan/18, Level A, counted here
        //   too since AA >= A) = 49.
        // - parsing und positive-tabindex, beide Level A, in den geteilten
        //   Bestand abgegeben = 47.
        // - meta-viewport-large (1.4.4): a11y-rules 0.5.0 trennt den Verstoß
        //   unter 200 % von der Begrenzung zwischen 200 und 500 %, womit die
        //   Regel als zoom/viewport-scale-limited in den geteilten Bestand
        //   wandert = 46.
        // + accessible-authentication (3.3.8, WCAG 2.2 AA, plan 54 §3) = 47.
        // + autocomplete-valid (1.3.5): moved from the AX tree to the DOM,
        //   the tree carries aria-autocomplete, not the HTML attribute = 48.
        // + form-field-group for named checkbox sets (#643, Level A) = 49.
        // - landmark-dom: duplicated the AX tree's landmark-unique and
        //   landmark-main-present, each defect counted twice = 48.
        // + value-now (#656, Level A) = 49.
        // + display-mode convention (#653, Level A) = 50.
        // - aria-hidden-focus, aria-valid-attr, checked-state, value-now
        //   (Level A) und meta-viewport (1.4.4) in den geteilten Bestand
        //   abgegeben (#690) = 45.
        // + scrollable-region-focusable (2.1.1, #717, Level A) = 46.
        // - sechs ARIA-Pruefungen (Level A) in den geteilten Bestand
        //   abgegeben (#691) = 40.
        // - label-in-name (Level A) in den geteilten Bestand abgegeben
        //   (#692) = 39.
        // - vier Formularpruefungen (Level A) und autocomplete-valid (1.3.5)
        //   in den geteilten Bestand abgegeben (#693) = 34.
        // - click-handler und link-as-button (Level A) in den geteilten
        //   Bestand abgegeben (#695) = 32.
        // - frame-title und server-side-image-map (Level A) in den geteilten
        //   Bestand abgegeben (#696) = 30.
        // - drei Level-A-Regeln (siehe oben) sowie language-of-parts und
        //   content-on-hover (AA) in den geteilten Bestand abgegeben
        //   (#697) = 25.
        // - area-alt (Level A) in den geteilten Bestand abgegeben = 24.
        // - focus-visible-css und reduced-motion (beide hier AA): laufen als
        //   `focus/outline-removed` und `motion/reduced-motion-ignored` ueber
        //   die Stylesheets im geteilten Bestand = 22.
        assert_eq!(count, 22);
    }

    #[test]
    fn level_aaa_filter_includes_all_rules() {
        let count = PAGE_RULES
            .iter()
            .filter(|r| WcagLevel::AAA >= r.min_level)
            .count();
        assert_eq!(count, PAGE_RULES.len());
    }
}
