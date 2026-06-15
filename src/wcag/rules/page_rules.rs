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
//! themselves) and a `name` for the "Found N <name> violations" log line.

use chromiumoxide::Page;
use futures::future::BoxFuture;

use crate::cli::WcagLevel;
use crate::wcag::Violation;

use super::{
    check_abbreviations_with_page, check_aria_hidden_focus, check_aria_prohibited_attr_with_page,
    check_aria_valid_attr_value_with_page, check_background_audio_with_page,
    check_content_on_hover_with_page, check_focus_visible_css_with_page,
    check_form_no_submit_with_page, check_frame_tested_with_page, check_frame_title_with_page,
    check_identify_purpose_with_page, check_label_in_name_with_page, check_landmarks_with_page,
    check_location_with_page, check_motion_actuation_with_page, check_no_interruptions_with_page,
    check_no_timing_with_page, check_orientation_with_page, check_page_titled_with_page,
    check_parsing_with_page, check_pointer_cancellation_with_page,
    check_pointer_gestures_with_page, check_presentation_semantic_children_with_page,
    check_re_authenticate_with_page, check_reduced_motion_with_page,
    check_same_origin_iframes_with_page, check_target_size_enhanced_with_page,
    check_timeouts_with_page, check_timing_with_page, check_use_of_color_with_page,
    check_visual_presentation_with_page,
};
use crate::wcag::engine::check_click_handlers_with_page;

/// One row of the page-rule catalog.
pub struct PageRuleEntry {
    /// Stable identifier used in log lines only (not the rule string that
    /// appears in findings — that comes from each check's own metadata).
    pub rule_id: &'static str,
    /// Short label for the "Found N <name> violations" log line, to match
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
        rule_id: "4.1.2/aria-hidden-focus",
        name: "aria-hidden-focus",
        min_level: WcagLevel::A,
        check_fn: |p| Box::pin(check_aria_hidden_focus(p)),
    },
    PageRuleEntry {
        rule_id: "4.1.2/aria-prohibited-attr",
        name: "aria-prohibited-attr",
        min_level: WcagLevel::A,
        check_fn: |p| Box::pin(check_aria_prohibited_attr_with_page(p)),
    },
    PageRuleEntry {
        rule_id: "2.4.1/frame-title",
        name: "frame-title",
        min_level: WcagLevel::A,
        check_fn: |p| Box::pin(check_frame_title_with_page(p)),
    },
    PageRuleEntry {
        rule_id: "4.1.2/frame-tested",
        name: "frame-tested",
        min_level: WcagLevel::A,
        check_fn: |p| Box::pin(check_frame_tested_with_page(p)),
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
        rule_id: "2.1.1/click-handler",
        name: "inline click-handler",
        min_level: WcagLevel::A,
        check_fn: |p| Box::pin(check_click_handlers_with_page(p)),
    },
    PageRuleEntry {
        rule_id: "3.2.2/form-no-submit",
        name: "form-no-submit",
        min_level: WcagLevel::A,
        check_fn: |p| Box::pin(check_form_no_submit_with_page(p)),
    },
    PageRuleEntry {
        rule_id: "1.3.1/presentation-semantic-children",
        name: "presentation semantic children",
        min_level: WcagLevel::A,
        check_fn: |p| Box::pin(check_presentation_semantic_children_with_page(p)),
    },
    PageRuleEntry {
        rule_id: "1.3.1/landmark-dom",
        name: "landmark DOM",
        min_level: WcagLevel::A,
        check_fn: |p| Box::pin(check_landmarks_with_page(p)),
    },
    PageRuleEntry {
        rule_id: "2.2.1/meta-refresh",
        name: "meta-refresh",
        min_level: WcagLevel::A,
        check_fn: |p| Box::pin(check_timing_with_page(p)),
    },
    PageRuleEntry {
        rule_id: "1.4.1/use-of-color",
        name: "use-of-color",
        min_level: WcagLevel::A,
        check_fn: |p| Box::pin(check_use_of_color_with_page(p)),
    },
    // ── Level AA and above ────────────────────────────────────────────────────
    PageRuleEntry {
        rule_id: "1.3.4/orientation",
        name: "orientation",
        min_level: WcagLevel::AA,
        check_fn: |p| Box::pin(check_orientation_with_page(p)),
    },
    PageRuleEntry {
        rule_id: "2.4.7/focus-visible-css",
        name: "CSS focus-suppression",
        min_level: WcagLevel::AA,
        check_fn: |p| Box::pin(check_focus_visible_css_with_page(p)),
    },
    PageRuleEntry {
        rule_id: "2.3.3/reduced-motion",
        name: "reduced-motion",
        min_level: WcagLevel::AA,
        check_fn: |p| Box::pin(check_reduced_motion_with_page(p)),
    },
    PageRuleEntry {
        rule_id: "1.4.13/content-on-hover",
        name: "content-on-hover",
        min_level: WcagLevel::AA,
        check_fn: |p| Box::pin(check_content_on_hover_with_page(p)),
    },
    // ── Level AAA only ────────────────────────────────────────────────────────
    PageRuleEntry {
        rule_id: "1.3.6/identify-purpose",
        name: "identify-purpose",
        min_level: WcagLevel::AAA,
        check_fn: |p| Box::pin(check_identify_purpose_with_page(p)),
    },
    PageRuleEntry {
        rule_id: "1.4.7/background-audio",
        name: "background-audio",
        min_level: WcagLevel::AAA,
        check_fn: |p| Box::pin(check_background_audio_with_page(p)),
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
        rule_id: "2.4.8/location",
        name: "location",
        min_level: WcagLevel::AAA,
        check_fn: |p| Box::pin(check_location_with_page(p)),
    },
    PageRuleEntry {
        rule_id: "2.5.1/pointer-gestures",
        name: "pointer-gestures",
        min_level: WcagLevel::AAA,
        check_fn: |p| Box::pin(check_pointer_gestures_with_page(p)),
    },
    PageRuleEntry {
        rule_id: "2.5.2/pointer-cancellation",
        name: "pointer-cancellation",
        min_level: WcagLevel::AAA,
        check_fn: |p| Box::pin(check_pointer_cancellation_with_page(p)),
    },
    PageRuleEntry {
        rule_id: "2.5.3/label-in-name",
        name: "label-in-name",
        min_level: WcagLevel::AAA,
        check_fn: |p| Box::pin(check_label_in_name_with_page(p)),
    },
    PageRuleEntry {
        rule_id: "2.5.4/motion-actuation",
        name: "motion-actuation",
        min_level: WcagLevel::AAA,
        check_fn: |p| Box::pin(check_motion_actuation_with_page(p)),
    },
    PageRuleEntry {
        rule_id: "2.5.5/target-size-enhanced",
        name: "target-size-enhanced",
        min_level: WcagLevel::AAA,
        check_fn: |p| Box::pin(check_target_size_enhanced_with_page(p)),
    },
    PageRuleEntry {
        rule_id: "3.1.4/abbreviations",
        name: "abbreviations",
        min_level: WcagLevel::AAA,
        check_fn: |p| Box::pin(check_abbreviations_with_page(p)),
    },
    PageRuleEntry {
        rule_id: "4.1.1/parsing",
        name: "parsing",
        min_level: WcagLevel::A,
        check_fn: |p| Box::pin(check_parsing_with_page(p)),
    },
    PageRuleEntry {
        rule_id: "4.1.2/aria-valid-attr-value",
        name: "aria-valid-attr-value",
        min_level: WcagLevel::A,
        check_fn: |p| Box::pin(check_aria_valid_attr_value_with_page(p)),
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
        // + aria-valid-attr-value + iframe-content = 14
        assert_eq!(count, 14);
    }

    #[test]
    fn level_aa_filter_includes_aa_plus_a_rules() {
        let count = PAGE_RULES
            .iter()
            .filter(|r| WcagLevel::AA >= r.min_level)
            .count();
        // 14 A + 4 AA = 18.
        assert_eq!(count, 18);
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
