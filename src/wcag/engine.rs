//! WCAG Rule Engine - Executes all WCAG rules against an AXTree
//!
//! The engine loads all rules and runs them against the accessibility tree.

use tracing::{debug, info};

pub use super::rules::{
    check_abbreviations_with_page, check_background_audio_with_page,
    check_click_handlers_with_page, check_content_on_hover_with_page,
    check_focus_visible_css_with_page, check_identify_purpose_with_page, check_location_with_page,
    check_motion_actuation_with_page, check_no_interruptions_with_page, check_no_timing_with_page,
    check_orientation_with_page, check_pointer_cancellation_with_page,
    check_pointer_gestures_with_page, check_re_authenticate_with_page,
    check_reduced_motion_with_page, check_reflow_with_page, check_target_size_enhanced_with_page,
    check_timeouts_with_page, check_timing_with_page, check_use_of_color_with_page,
    check_visual_presentation_with_page,
};
use super::rules::{
    check_accessible_name, check_bypass_blocks, check_error_identification, check_focus_visible,
    check_form_rules, check_help, check_instructions, check_keyboard, check_label_title_only,
    check_labels, check_landmark_banner_is_top_level, check_landmark_contentinfo_is_top_level,
    check_landmark_main_is_top_level, check_landmark_no_duplicate_banner,
    check_landmark_no_duplicate_contentinfo, check_landmark_unique, check_link_purpose,
    check_link_purpose_link_only, check_media_rules, check_region, check_section_headings,
    check_table_extended, check_text_alternatives, check_unusual_words,
};
use super::types::{Violation, WcagResults};
use crate::accessibility::AXTree;
use crate::cli::WcagLevel;

/// Rule filtering configuration (subset of cli::config::RulesConfig for the engine)
#[derive(Debug, Clone, Default)]
pub struct RuleFilterConfig {
    /// axe_ids of rules to disable
    pub disabled_rules: Vec<String>,
    /// If non-empty, only run these rules (by axe_id)
    pub enabled_only_rules: Vec<String>,
}

impl RuleFilterConfig {
    /// Returns true if the rule with the given axe_id should be run
    pub fn should_run(&self, axe_id: &str) -> bool {
        if !self.enabled_only_rules.is_empty() {
            return self.enabled_only_rules.iter().any(|r| r == axe_id);
        }
        !self.disabled_rules.iter().any(|r| r == axe_id)
    }
}

/// Run all WCAG checks against an AXTree
///
/// # Arguments
/// * `tree` - The accessibility tree to check
/// * `level` - The WCAG conformance level to check against
///
/// # Returns
/// Results containing all violations found
pub fn check_all(tree: &AXTree, level: WcagLevel) -> WcagResults {
    check_all_with_config(tree, level, &RuleFilterConfig::default())
}

/// Run all WCAG checks against an AXTree with optional rule filtering
///
/// # Arguments
/// * `tree` - The accessibility tree to check
/// * `level` - The WCAG conformance level to check against
/// * `filter` - Rule filter configuration (disabled/enabled_only rules)
///
/// # Returns
/// Results containing all violations found
pub fn check_all_with_config(
    tree: &AXTree,
    level: WcagLevel,
    filter: &RuleFilterConfig,
) -> WcagResults {
    check_all_excluding(tree, tree, level, filter, &|_| false).0
}

/// [`check_all_with_config`], dropping every finding `exclude` matches
/// (audit exclusions, #645) per rule — before the rule's outcome is counted,
/// so `rule_outcomes[].findings` describes what the report shows. Returns the
/// kept results and the dropped findings.
///
/// `counted` is the tree the page-level counting rules (duplicate and
/// unique landmarks) read: `tree` without the excluded subtrees, so a
/// specimen's own landmarks are not counted against the page (#726). Without
/// exclusions it is `tree` itself.
pub fn check_all_excluding(
    tree: &AXTree,
    counted: &AXTree,
    level: WcagLevel,
    filter: &RuleFilterConfig,
    exclude: &dyn Fn(&Violation) -> bool,
) -> (WcagResults, Vec<Violation>) {
    info!("Running WCAG checks at level {}", level);

    let mut run = TreeRun {
        results: WcagResults::new(),
        exclude,
        excluded: Vec::new(),
    };
    run.results.nodes_checked = tree.len();

    // Run Level A rules
    debug!("Running Level A rules...");
    run_level_a_rules(tree, counted, &mut run, filter);

    // Run Level AA rules if requested
    if matches!(level, WcagLevel::AA | WcagLevel::AAA) {
        debug!("Running Level AA rules...");
        run_level_aa_rules(tree, &mut run, filter);
    }

    // Run Level AAA rules if requested
    if level == WcagLevel::AAA {
        debug!("Running Level AAA rules...");
        run_level_aaa_rules(tree, &mut run, filter);
    }

    info!(
        "WCAG check complete: {} violations found",
        run.results.violations.len()
    );

    (run.results, run.excluded)
}

/// Accumulates the tree rules' results, applying the exclusion per rule.
struct TreeRun<'a> {
    results: WcagResults,
    exclude: &'a dyn Fn(&Violation) -> bool,
    excluded: Vec<Violation>,
}

impl TreeRun<'_> {
    fn record(&mut self, axe_id: &str, mut rule_results: WcagResults) {
        for list in [&mut rule_results.violations, &mut rule_results.warnings] {
            let (dropped, kept): (Vec<_>, Vec<_>) = std::mem::take(list)
                .into_iter()
                .partition(|v| (self.exclude)(v));
            *list = kept;
            self.excluded.extend(dropped);
        }
        let finding_count = rule_results.violations.len();
        // Die Regel ist gelaufen -- ob sie etwas fand, steht in
        // `findings`, und *wie sicher* die Aussage ist, am Befund.
        let mut run = crate::wcag::RuleRun::ran(axe_id, finding_count);
        if let Some(criterion) = crate::taxonomy::criterion_for_rule(axe_id) {
            run = run.with_wcag([criterion]);
        }
        self.results.rule_outcomes.push(run);
        self.results.merge(rule_results);
    }
}

/// Merge rule results only if the filter allows the given axe_id
macro_rules! run_if_allowed {
    ($filter:expr, $axe_id:expr, $check_fn:expr, $results:expr, $tree:expr) => {
        if $filter.should_run($axe_id) {
            $results.record($axe_id, $check_fn($tree));
        }
    };
}

/// Run all Level A rules
fn run_level_a_rules(
    tree: &AXTree,
    counted: &AXTree,
    results: &mut TreeRun<'_>,
    filter: &RuleFilterConfig,
) {
    // 1.1.1 Non-text Content (Level A)
    run_if_allowed!(filter, "image-alt", check_text_alternatives, results, tree);
    // 1.1.1 Area / input[type=image] / object alternatives, and server-side
    // image maps, now run as DOM page rules (check_image_input_rules_with_page /
    // check_server_side_image_map_with_page in PAGE_RULES) — htmlTag/type/ismap
    // are not AX properties (#QA-030).

    // 2.1.1 Keyboard (Level A)
    run_if_allowed!(filter, "keyboard", check_keyboard, results, tree);

    // 2.4.1 Bypass Blocks (Level A)
    run_if_allowed!(filter, "bypass", check_bypass_blocks, results, tree);

    // 2.4.2 Page Titled: fehlender und leerer Titel laufen als
    // `document/title-*` im geteilten Bestand, der nichtssagende Titel als
    // DOM-Page-Rule `check_page_titled_with_page`. Der AX-Baum taugt dafuer
    // nicht: Sein Wurzelname faellt ohne Titel auf die URL zurueck.

    // 2.4.4 Link Purpose (In Context) (Level A)
    run_if_allowed!(filter, "link-name", check_link_purpose, results, tree);

    // 3.1.1 Language of Page (Level A) laeuft als geteilte Regel
    // (`document/lang-missing`, `document/lang-invalid`, siehe wcag::shared)
    // gegen den per CDP geholten DOM. Die AX-Eigenschaft `language`
    // synthetisiert Chrome aus Locale und Kontext, auch wenn der Autor nie
    // ein `lang` gesetzt hat -- eine AX-basierte Pruefung ist fuer den
    // haeufigsten Fall also blind.
    // 3.1.1 xml:lang-Abgleich laeuft weiter als DOM-Page-Rule
    // (check_language_extended_with_page in PAGE_RULES).

    // 3.3.2 Labels or Instructions (Level A)
    run_if_allowed!(filter, "label", check_instructions, results, tree);

    // 3.3.1 Error Identification (Level A)
    run_if_allowed!(
        filter,
        "aria-invalid-without-describedby",
        check_error_identification,
        results,
        tree
    );

    // 2.4.3 Focus Order: fokussierbar trotz `aria-hidden` laeuft als
    // `keyboard/hidden-focusable` im geteilten Bestand (siehe wcag::shared),
    // positives `tabindex` als `keyboard/positive-tabindex`.

    // 2.5.3 Label in Name laeuft als `label-in-name/mismatch` im geteilten
    // Bestand (#692).

    // 3.2.1 On Focus and 3.2.2 On Input now run as DOM page rules
    // (check_on_focus_with_page / check_on_input_with_page in PAGE_RULES) —
    // onfocus/onchange/autofocus are HTML attributes, never AX properties (#QA-030).

    // 4.1.2 Name, Role, Value (Level A)
    run_if_allowed!(filter, "label", check_labels, results, tree);

    // 4.1.2 ARIA: Rollen, Attributnamen und -werte, erlaubte und verbotene
    // Attribute, Kontext und Bestandteile laufen als `aria/*` im geteilten
    // Bestand (#691), gegen den DOM -- ebenso
    // `aria/required-attribute-missing` (#690).

    // 1.3.1 Region / Landmark (Level A)
    run_if_allowed!(filter, "region", check_region, results, tree);

    // 4.1.2 Beschreibung wiederholt den Namen (Best Practice, #713). Fehlender
    // und rein symbolischer Name laufen als `names/*` im geteilten Bestand
    // (#692).
    run_if_allowed!(
        filter,
        "description-duplicates-name",
        check_accessible_name,
        results,
        tree
    );

    // 4.1.2 ARIA Relationship Attributes laeuft als `aria/reference-missing`
    // im geteilten Bestand.

    // 4.1.2 Rollen mit Namenspflicht laufen als `names/required-missing` im
    // geteilten Bestand (#692).

    // 1.3.1 / 3.3.1 / 3.3.2 Form Rules (Level A) - P1
    run_if_allowed!(filter, "form-field-group", check_form_rules, results, tree);

    // 4.1.2 Dialoge laufen als `dialog/name-missing` und
    // `dialog/modal-unmarked` im geteilten Bestand (#692).

    // 4.1.2 Tabs und Combobox laufen als `aria/tab-selected-missing`,
    // `aria/tabpanel-missing` und `aria/combobox-popup-missing` im geteilten
    // Bestand (#691).

    // 1.2.1 / 1.1.1 Media Rules (Level A) - P2
    run_if_allowed!(filter, "video-caption", check_media_rules, results, tree);

    // 1.1.1 SVG laeuft als `svg/name-missing` im geteilten Bestand.

    // 2.4.1 Skip Link (Level A)

    // 1.3.1 Granular Landmark Rules (Level A)
    run_if_allowed!(
        filter,
        "landmark-unique",
        check_landmark_unique,
        results,
        counted
    );
    run_if_allowed!(
        filter,
        "landmark-banner-is-top-level",
        check_landmark_banner_is_top_level,
        results,
        tree
    );
    run_if_allowed!(
        filter,
        "landmark-contentinfo-is-top-level",
        check_landmark_contentinfo_is_top_level,
        results,
        tree
    );
    run_if_allowed!(
        filter,
        "landmark-main-is-top-level",
        check_landmark_main_is_top_level,
        results,
        tree
    );
    run_if_allowed!(
        filter,
        "landmark-no-duplicate-banner",
        check_landmark_no_duplicate_banner,
        results,
        counted
    );
    run_if_allowed!(
        filter,
        "landmark-no-duplicate-contentinfo",
        check_landmark_no_duplicate_contentinfo,
        results,
        counted
    );
    // Fehlende main-/banner-Landmark und doppelte main laufen als
    // `landmarks/*` im geteilten Bestand.

    // 1.3.1 th-has-data-cells (Level A) - P1. td-headers-attr now runs as a
    // DOM page rule (check_table_headers_attr_with_page in PAGE_RULES) —
    // `headers` is not an AX property (#QA-030).
    run_if_allowed!(
        filter,
        "th-has-data-cells",
        check_table_extended,
        results,
        tree
    );

    // 4.1.2 `<summary>` laeuft als `summary/name-missing` im geteilten
    // Bestand (#692).

    // 1.3.1 Label Title Only (Level A)
    run_if_allowed!(
        filter,
        "label-title-only",
        check_label_title_only,
        results,
        tree
    );

    // 4.1.2 Mehrdeutiges `aria-owns` laeuft als `aria/owns-conflict` im
    // geteilten Bestand (#691).
}

/// Run all Level AA rules
fn run_level_aa_rules(tree: &AXTree, results: &mut TreeRun<'_>, filter: &RuleFilterConfig) {
    // Note: 1.4.3 Contrast (Minimum) requires CDP page access and is
    // handled separately in the pipeline via ContrastRule::check_with_page

    // 1.3.5 Identify Input Purpose now runs as a DOM page rule
    // (check_input_purpose_with_page in PAGE_RULES) — the AX tree carries
    // `aria-autocomplete`, not the HTML `autocomplete` attribute.

    // 1.4.4 Resize Text: der Viewport laeuft als `zoom/*` im geteilten
    // Bestand -- der AX-Baum hat keine `viewport`-Eigenschaft (#QA-030).

    // 1.4.11 Non-text Contrast (Level AA) now runs as a DOM page rule
    // (check_non_text_contrast_css_with_page in PAGE_RULES) — the AX tree
    // has no CSS/color data, so a tree-only check could not verify real
    // contrast (#QA-non-text-contrast).

    // 2.4.7 Focus Visible (Level AA)
    run_if_allowed!(filter, "focus-visible", check_focus_visible, results, tree);

    // 4.1.3 Status Messages laeuft als `status/live-overridden` im geteilten
    // Bestand (#692).

    // 2.4.1 / 1.3.1 Landmark Regions laufen als `landmarks/*` im geteilten
    // Bestand.
}

/// Run all Level AAA rules
fn run_level_aaa_rules(tree: &AXTree, results: &mut TreeRun<'_>, filter: &RuleFilterConfig) {
    // Note: 1.4.6 Contrast (Enhanced) requires CDP page access and is
    // handled separately in the pipeline via ContrastRule::check_with_page

    // 2.4.10 Section Headings (Level AAA)
    run_if_allowed!(
        filter,
        "heading-order",
        check_section_headings,
        results,
        tree
    );

    // 2.4.9 Link Purpose (Link Only) (Level AAA)
    run_if_allowed!(
        filter,
        "link-name",
        check_link_purpose_link_only,
        results,
        tree
    );

    // 1.2.8 Media Alternative (Prerecorded) (Level AAA) — moved to the
    // page-rule table as check_media_alternative_with_page (DOM + transcript
    // detection needs live page access, not just the AXTree).

    // 3.1.3 Unusual Words (Level AAA)
    run_if_allowed!(filter, "unusual-words", check_unusual_words, results, tree);

    // 3.3.5 Help (Level AAA)
    run_if_allowed!(filter, "help", check_help, results, tree);

    debug!("Level AAA rules executed");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::{AXNode, AXTree};

    fn create_test_tree() -> AXTree {
        let nodes = vec![
            AXNode {
                node_id: "1".to_string(),
                ignored: false,
                ignored_reasons: vec![],
                role: Some("WebArea".to_string()),
                name: Some("Test Page".to_string()),
                name_source: None,
                description: None,
                value: None,
                properties: vec![],
                child_ids: vec!["2".to_string()],
                parent_id: None,
                backend_dom_node_id: None,
            },
            AXNode {
                node_id: "2".to_string(),
                ignored: false,
                ignored_reasons: vec![],
                role: Some("image".to_string()),
                name: None, // Missing alt text!
                name_source: None,
                description: None,
                value: None,
                properties: vec![],
                child_ids: vec![],
                parent_id: Some("1".to_string()),
                backend_dom_node_id: None,
            },
        ];

        AXTree::from_nodes(nodes)
    }

    #[test]
    fn test_check_all_level_a() {
        let tree = create_test_tree();
        let results = check_all(&tree, WcagLevel::A);

        // Should find the missing alt text
        assert!(!results.violations.is_empty());
        assert!(results.violations.iter().any(|v| v.rule == "1.1.1"));
    }

    #[test]
    fn check_all_excluding_drops_per_rule_and_counts_what_remains() {
        let tree = create_test_tree();
        let full = check_all(&tree, WcagLevel::A);
        let alt_run = |r: &WcagResults| {
            r.rule_outcomes
                .iter()
                .find(|o| o.rule_id == "image-alt")
                .map(|o| o.findings)
        };
        assert!(alt_run(&full).unwrap() > 0);

        let (kept, dropped) = check_all_excluding(
            &tree,
            &tree,
            WcagLevel::A,
            &RuleFilterConfig::default(),
            &|v| v.node_id == "2",
        );
        assert!(kept.violations.iter().all(|v| v.node_id != "2"));
        assert!(dropped.iter().any(|v| v.rule == "1.1.1"));
        // The outcome counts what the report shows, not the raw rule output.
        assert_eq!(alt_run(&kept), Some(0));
    }

    #[test]
    fn test_check_all_level_aa() {
        let tree = create_test_tree();
        let results = check_all(&tree, WcagLevel::AA);

        // Should run both A and AA rules
        assert!(!results.violations.is_empty());
    }

    #[test]
    fn test_rule_filter_disabled_rules() {
        let tree = create_test_tree();
        let filter = RuleFilterConfig {
            disabled_rules: vec!["image-alt".to_string()],
            enabled_only_rules: vec![],
        };
        let results = check_all_with_config(&tree, WcagLevel::A, &filter);
        // image-alt disabled → no 1.1.1 violation from text_alternatives
        assert!(results.violations.iter().all(|v| v.rule != "1.1.1"));
    }

    #[test]
    fn test_rule_filter_wcag_level() {
        let tree = create_test_tree();
        let results_a = check_all(&tree, WcagLevel::A);
        let results_aa = check_all(&tree, WcagLevel::AA);
        // AA scan runs at least as many checks as A scan
        assert!(results_aa.violations.len() >= results_a.violations.len());
        // AAA-only rule (section-headings) is absent in an A-only scan
        let results_aaa = check_all(&tree, WcagLevel::AAA);
        assert!(results_aaa.violations.len() >= results_aa.violations.len());
    }

    #[test]
    fn test_rule_filter_default_runs_all() {
        let tree = create_test_tree();
        let default_filter = RuleFilterConfig::default();
        let filtered = check_all_with_config(&tree, WcagLevel::A, &default_filter);
        let unfiltered = check_all(&tree, WcagLevel::A);
        // Default (empty) filter behaves identically to no filter
        assert_eq!(filtered.violations.len(), unfiltered.violations.len());
    }
}
