//! WCAG Rule Engine - Executes all WCAG rules against an AXTree
//!
//! The engine loads all rules and runs them against the accessibility tree.

use tracing::{debug, info};

use super::rules::{
    check_accessible_name, check_focus_visible, check_keyboard, check_text_alternatives,
};
pub use super::rules::{
    check_orientation_with_page, check_pointer_cancellation_with_page, check_reflow_with_page,
    check_timeouts_with_page, check_timing_with_page, check_visual_presentation_with_page,
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

    for rule in TREE_RULES {
        if level >= rule.min_level && filter.should_run(rule.axe_id) {
            debug!("Running tree rule {}", rule.axe_id);
            run.record(rule.axe_id, (rule.check)(tree));
        }
    }

    // Level AAA hat keine Baumregeln mehr: 1.4.6 (`contrast/text-enhanced`,
    // #698), 2.4.9 (`links/purpose-link-only`, Plan 98), 2.4.10
    // (`headings/section-without-heading`, #697), 3.1.3/3.3.5 (`manual/*`,
    // #722) laufen im geteilten Bestand, 1.2.8 als Page-Rule
    // (`check_media_alternative_with_page`).

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

/// One AX-tree rule. The engine runs exactly this table, and the expected
/// outcomes of a run (`wcag::expectation`) are derived from it, so a rule
/// cannot be added without it being requested (#778).
pub struct TreeRule {
    pub axe_id: &'static str,
    /// The rule runs when the configured level is at least this one.
    pub min_level: WcagLevel,
    pub check: fn(&AXTree) -> WcagResults,
}

/// The tree rules that remain. Everything else that once ran here moved to
/// the shared rules (`wcag::shared::SHARED_RULES`) or the page rules.
pub const TREE_RULES: &[TreeRule] = &[
    // 1.1.1 Non-text Content
    TreeRule {
        axe_id: "image-alt",
        min_level: WcagLevel::A,
        check: check_text_alternatives,
    },
    // 2.1.2 No Keyboard Trap: whether focus gets out again shows only real
    // operation -- one note per modal dialog and one page-wide.
    TreeRule {
        axe_id: "keyboard-trap",
        min_level: WcagLevel::A,
        check: check_keyboard,
    },
    // 4.1.2 Description repeats the name (best practice, #713).
    TreeRule {
        axe_id: "description-duplicates-name",
        min_level: WcagLevel::A,
        check: check_accessible_name,
    },
    // 2.4.7 Focus Visible
    TreeRule {
        axe_id: "focus-visible",
        min_level: WcagLevel::AA,
        check: check_focus_visible,
    },
];

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
