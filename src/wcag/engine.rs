//! WCAG Rule Engine - Executes all WCAG rules against an AXTree
//!
//! The engine loads all rules and runs them against the accessibility tree.

use tracing::{debug, info};

pub use super::rules::check_reflow_with_page;
use super::rules::{check_focus_visible, check_keyboard};
use super::types::{Violation, WcagResults};
use crate::accessibility::AXTree;
use crate::wcag::WcagLevel;

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
    // #698), 2.4.9 (`links/purpose-link-only`), 2.4.10
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
    // 2.1.2 No Keyboard Trap: whether focus gets out again shows only real
    // operation -- one note per modal dialog and one page-wide.
    TreeRule {
        axe_id: "keyboard-trap",
        min_level: WcagLevel::A,
        check: check_keyboard,
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
    use crate::accessibility::{AXNode, AXProperty, AXTree, AXValue};

    /// A page with one modal dialog: `keyboard-trap` notes the dialog for
    /// review (node "2").
    fn create_test_tree() -> AXTree {
        let node = |id: &str, role: &str, parent: Option<&str>, children: &[&str]| AXNode {
            node_id: id.to_string(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some(role.to_string()),
            name: Some("Test".to_string()),
            name_source: None,
            description: None,
            value: None,
            properties: vec![],
            child_ids: children.iter().map(|c| c.to_string()).collect(),
            parent_id: parent.map(String::from),
            backend_dom_node_id: None,
        };
        let mut dialog = node("2", "dialog", Some("1"), &[]);
        dialog.properties.push(AXProperty {
            name: "modal".to_string(),
            value: AXValue::Bool(true),
        });
        AXTree::from_nodes(vec![node("1", "RootWebArea", None, &["2"]), dialog])
    }

    fn trap_notes(results: &WcagResults) -> usize {
        results
            .warnings
            .iter()
            .filter(|v| v.rule_id.as_deref() == Some("keyboard-trap"))
            .count()
    }

    fn ran(results: &WcagResults, id: &str) -> bool {
        results.rule_outcomes.iter().any(|o| o.rule_id == id)
    }

    #[test]
    fn test_check_all_level_a() {
        let tree = create_test_tree();
        let results = check_all(&tree, WcagLevel::A);

        assert_eq!(trap_notes(&results), 1);
        assert!(ran(&results, "keyboard-trap"));
        assert!(!ran(&results, "focus-visible"));
    }

    #[test]
    fn check_all_excluding_drops_per_rule_and_keeps_the_outcome() {
        let tree = create_test_tree();
        let (kept, dropped) =
            check_all_excluding(&tree, WcagLevel::A, &RuleFilterConfig::default(), &|v| {
                v.node_id == "2"
            });
        assert_eq!(trap_notes(&kept), 0);
        assert!(dropped.iter().any(|v| v.rule == "2.1.2"));
        // The rule still ran; only its finding was excluded.
        assert!(ran(&kept, "keyboard-trap"));
    }

    #[test]
    fn test_check_all_level_aa() {
        let tree = create_test_tree();
        let results = check_all(&tree, WcagLevel::AA);

        // Should run both A and AA rules
        assert!(ran(&results, "keyboard-trap"));
        assert!(ran(&results, "focus-visible"));
    }

    #[test]
    fn test_rule_filter_disabled_rules() {
        let tree = create_test_tree();
        let filter = RuleFilterConfig {
            disabled_rules: vec!["keyboard-trap".to_string()],
            enabled_only_rules: vec![],
        };
        let results = check_all_with_config(&tree, WcagLevel::A, &filter);
        assert_eq!(trap_notes(&results), 0);
        assert!(!ran(&results, "keyboard-trap"));
    }

    #[test]
    fn test_rule_filter_wcag_level() {
        let tree = create_test_tree();
        let runs = |level| check_all(&tree, level).rule_outcomes.len();
        // A higher level runs at least as many rules as a lower one.
        assert!(runs(WcagLevel::AA) >= runs(WcagLevel::A));
        assert!(runs(WcagLevel::AAA) >= runs(WcagLevel::AA));
    }

    #[test]
    fn test_rule_filter_default_runs_all() {
        let tree = create_test_tree();
        let default_filter = RuleFilterConfig::default();
        let filtered = check_all_with_config(&tree, WcagLevel::A, &default_filter);
        let unfiltered = check_all(&tree, WcagLevel::A);
        // Default (empty) filter behaves identically to no filter
        assert_eq!(filtered.warnings.len(), unfiltered.warnings.len());
        assert_eq!(filtered.rule_outcomes.len(), unfiltered.rule_outcomes.len());
    }
}
