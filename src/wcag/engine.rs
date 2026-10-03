//! WCAG Rule Engine - Executes all WCAG rules against an AXTree
//!
//! The engine loads all rules and runs them against the accessibility tree.

use tracing::{debug, info};

use super::rules::{
    check_accessible_name, check_focus_visible, check_help, check_keyboard, check_link_purpose,
    check_link_purpose_link_only, check_text_alternatives, check_unusual_words,
};
pub use super::rules::{
    check_focus_visible_css_with_page, check_motion_actuation_with_page,
    check_no_interruptions_with_page, check_no_timing_with_page, check_orientation_with_page,
    check_pointer_cancellation_with_page, check_pointer_gestures_with_page,
    check_re_authenticate_with_page, check_reduced_motion_with_page, check_reflow_with_page,
    check_target_size_enhanced_with_page, check_timeouts_with_page, check_timing_with_page,
    check_use_of_color_with_page, check_visual_presentation_with_page,
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
    check_all_excluding(tree, level, filter, &|_| false).0
}

/// [`check_all_with_config`], dropping every finding `exclude` matches
/// (audit exclusions, #645) per rule — before the rule's outcome is counted,
/// so `rule_outcomes[].findings` describes what the report shows. Returns the
/// kept results and the dropped findings.
///
/// Die Regeln, die Landmarks der ganzen Seite zaehlen, laufen seit #694 im
/// geteilten Bestand; dort gilt fuer sie das Dokument ohne die
/// ausgeschlossenen Teilbaeume (#726, `wcag::shared::PAGE_COUNT_RULES`).
pub fn check_all_excluding(
    tree: &AXTree,
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
    run_level_a_rules(tree, &mut run, filter);

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
fn run_level_a_rules(tree: &AXTree, results: &mut TreeRun<'_>, filter: &RuleFilterConfig) {
    // 1.1.1 Non-text Content (Level A)
    run_if_allowed!(filter, "image-alt", check_text_alternatives, results, tree);
    // 1.1.1 `<area>`, `<input type="image">`, `<object>` und serverseitige
    // Image-Maps laufen als `images/*` und `objects/alt-missing` im geteilten
    // Bestand (#696).

    // 2.1.1 Keyboard laeuft als `keyboard/focusable-no-role` und
    // `keyboard/interactive-not-focusable` im geteilten Bestand (#694). Hier
    // bleibt 2.1.2 No Keyboard Trap: Ob der Fokus wieder herauskommt, zeigt
    // nur echte Bedienung -- ein Hinweis je modalem Dialog und seitenweit.
    run_if_allowed!(filter, "keyboard", check_keyboard, results, tree);

    // 2.4.1 Bypass Blocks: Sprunglink, main-Landmark und die Seite ganz ohne
    // Ueberschriften laufen als `keyboard/skip-link-missing`,
    // `landmarks/main-missing` und `headings/none` im geteilten Bestand
    // (#690, #694).

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
    // 3.1.1 xml:lang-Abgleich laeuft als `document/lang-mismatch` im
    // geteilten Bestand (#697).

    // 3.3.2 Labels or Instructions und 3.3.1 Error Identification laufen als
    // `forms/*` im geteilten Bestand (#693): fehlende Beschriftung,
    // Platzhalter als Beschriftung, Pflichtfeld, Formatangabe, Gruppenname,
    // Fehlerbeschreibung.

    // 2.4.3 Focus Order: fokussierbar trotz `aria-hidden` laeuft als
    // `keyboard/hidden-focusable` im geteilten Bestand (siehe wcag::shared),
    // positives `tabindex` als `keyboard/positive-tabindex`.

    // 2.5.3 Label in Name laeuft als `label-in-name/mismatch` im geteilten
    // Bestand (#692).

    // 3.2.1 On Focus und 3.2.2 On Input laufen als `context/*` im geteilten
    // Bestand (#693). Was eine laufende Seite braucht -- den Quelltext einer
    // aufgerufenen Funktion ueber `window` --, bleibt als DOM-Page-Rule
    // `check_on_input_with_page` in PAGE_RULES.

    // 4.1.2 Formularfelder ohne Namen laufen als `forms/label-missing` und
    // `names/required-missing` im geteilten Bestand (#693).

    // 4.1.2 ARIA: Rollen, Attributnamen und -werte, erlaubte und verbotene
    // Attribute, Kontext und Bestandteile laufen als `aria/*` im geteilten
    // Bestand (#691), gegen den DOM -- ebenso
    // `aria/required-attribute-missing` (#690).

    // 1.3.1 Inhalt ausserhalb jeder Landmark laeuft als
    // `landmarks/content-outside` im geteilten Bestand (#694).

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

    // 1.3.1 / 3.3.1 / 3.3.2 Formularstruktur laeuft als `forms/*` im
    // geteilten Bestand (#693).

    // 4.1.2 Dialoge laufen als `dialog/name-missing` und
    // `dialog/modal-unmarked` im geteilten Bestand (#692).

    // 4.1.2 Tabs und Combobox laufen als `aria/tab-selected-missing`,
    // `aria/tabpanel-missing` und `aria/combobox-popup-missing` im geteilten
    // Bestand (#691).

    // 1.1.1 / 1.2.1 AX-Pruefungen aus `media_rules` (unbenannte
    // `application`/`img`, benannte dekorative Elemente) sind mit #696
    // entfallen: `svg/name-missing`, `images/alt-missing` und
    // `aria/attribute-prohibited` decken sie im geteilten Bestand.

    // 1.1.1 SVG laeuft als `svg/name-missing` im geteilten Bestand.

    // 1.3.1 Landmarks: eindeutige Namen, Verschachtelung und doppelte
    // banner/contentinfo laufen als `landmarks/*` im geteilten Bestand (#694).
    // Fehlende main-/banner-Landmark und doppelte main laufen als
    // `landmarks/*` im geteilten Bestand.

    // 1.3.1 Kopfzellen ohne Daten und `headers`-Verweise laufen als
    // `tables/*` im geteilten Bestand (#697).

    // 4.1.2 `<summary>` laeuft als `summary/name-missing` im geteilten
    // Bestand (#692).

    // 1.3.1 Beschriftung nur per `title` laeuft als `forms/title-only-label`
    // im geteilten Bestand (#693).

    // 4.1.2 Mehrdeutiges `aria-owns` laeuft als `aria/owns-conflict` im
    // geteilten Bestand (#691).
}

/// Run all Level AA rules
fn run_level_aa_rules(tree: &AXTree, results: &mut TreeRun<'_>, filter: &RuleFilterConfig) {
    // Note: 1.4.3 Contrast (Minimum) requires CDP page access and is
    // handled separately in the pipeline via ContrastRule::check_with_page

    // 1.3.5 Identify Input Purpose laeuft als `forms/autocomplete-invalid`
    // und `forms/purpose-missing` im geteilten Bestand (#693).

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

    // 2.4.10 Section Headings laeuft als `headings/section-without-heading`
    // im geteilten Bestand (#697).

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

        let (kept, dropped) =
            check_all_excluding(&tree, WcagLevel::A, &RuleFilterConfig::default(), &|v| {
                v.node_id == "2"
            });
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
