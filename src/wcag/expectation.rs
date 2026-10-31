//! Expected rule outcomes of a run (#778).
//!
//! What a run is expected to account for is derived from the registrations
//! the pipeline executes -- the shared rules (`shared::SHARED_RULES`), the
//! AX-tree rules (`engine::TREE_RULES`), the page-rule table
//! (`rules::PAGE_RULES`) and the two inline page rules (reflow, HTML
//! content model) -- never from a second list. Each expected rule needs
//! exactly one outcome per viewport: ran, or not run with a reason.
//!
//! Frame rules need no entries of their own: a frame runs a subset of the
//! tree and shared rules (`audit::frames::FRAME_TREE_RULES`,
//! `FRAME_SHARED_RULES`) and folds its outcome into the page's
//! `(rule_id, viewport)` entry, so a frame cannot add or drop an outcome.
//!
//! **Counting unit.** A page-rule executor (`PageRuleEntry::rule_id`) and its
//! concrete rules (`outcome_ids`) are one check: when the executor fails,
//! the executor row and every concrete row carry the failure, but it is one
//! failed check. Execution and error counters count [`check_unit`]s, finding
//! counters count the concrete rule that owns the finding.

use std::collections::BTreeSet;

use crate::wcag::engine::{RuleFilterConfig, TREE_RULES};
use crate::wcag::rules::{HTML_CONTENT_MODEL_RULE, PAGE_RULES, REFLOW_RULE};
use crate::wcag::shared::SHARED_RULES;
use crate::wcag::RuleRun;
use crate::WcagLevel;

pub const DESKTOP: &str = "desktop";
pub const MOBILE: &str = "mobile";

/// A rule the run must account for, and the viewports it runs in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpectedRule {
    pub rule_id: String,
    pub viewports: Vec<&'static str>,
}

/// The rules a run with this level, filter and module selection must account
/// for. Rules above the level or removed by the filter are not expected --
/// except the shared rules, which always report (disabled, with a reason).
pub fn expected_rules(
    level: WcagLevel,
    filter: &RuleFilterConfig,
    html_conform: bool,
) -> Vec<ExpectedRule> {
    let both = vec![DESKTOP, MOBILE];
    let mut expected: Vec<ExpectedRule> = Vec::new();
    let mut add = |rule_id: &str, viewports: &[&'static str]| {
        if !expected.iter().any(|e| e.rule_id == rule_id) {
            expected.push(ExpectedRule {
                rule_id: rule_id.to_string(),
                viewports: viewports.to_vec(),
            });
        }
    };
    for rule in SHARED_RULES {
        add(rule.id, &both);
    }
    for rule in TREE_RULES {
        if level >= rule.min_level && filter.should_run(rule.axe_id) {
            add(rule.axe_id, &both);
        }
    }
    for rule in PAGE_RULES {
        if level < rule.min_level {
            continue;
        }
        add(rule.rule_id, &both);
        for id in rule.outcome_ids {
            add(id, &both);
        }
    }
    if level >= WcagLevel::AA && filter.should_run(REFLOW_RULE.axe_id) {
        add(REFLOW_RULE.axe_id, &[MOBILE]);
    }
    if html_conform && filter.should_run(HTML_CONTENT_MODEL_RULE.axe_id) {
        add(HTML_CONTENT_MODEL_RULE.axe_id, &[MOBILE]);
    }
    expected
}

/// Lowest level at which the registered rule runs; `None` for an id no
/// registration knows.
pub fn registered_min_level(rule_id: &str) -> Option<WcagLevel> {
    if let Some(rule) = SHARED_RULES.iter().find(|r| r.id == rule_id) {
        return Some(rule.level);
    }
    if let Some(rule) = TREE_RULES.iter().find(|r| r.axe_id == rule_id) {
        return Some(rule.min_level);
    }
    if let Some(rule) = PAGE_RULES
        .iter()
        .find(|r| r.rule_id == rule_id || r.outcome_ids.contains(&rule_id))
    {
        return Some(rule.min_level);
    }
    (rule_id == REFLOW_RULE.axe_id).then_some(WcagLevel::AA)
}

/// The check an outcome belongs to: the page-rule executor for a concrete
/// rule it owns, otherwise the rule itself.
pub fn check_unit(rule_id: &str) -> &str {
    PAGE_RULES
        .iter()
        .find(|rule| rule.outcome_ids.contains(&rule_id))
        .map_or(rule_id, |rule| rule.rule_id)
}

/// How many distinct checks (per viewport) the matching outcomes stand for.
/// An executor and its concrete rules count once.
pub fn count_checks(outcomes: &[RuleRun], matching: impl Fn(&RuleRun) -> bool) -> usize {
    outcomes
        .iter()
        .filter(|run| matching(run))
        .map(|run| (run.viewport.as_deref(), check_unit(&run.rule_id)))
        .collect::<BTreeSet<_>>()
        .len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids(expected: &[ExpectedRule]) -> BTreeSet<&str> {
        expected.iter().map(|e| e.rule_id.as_str()).collect()
    }

    #[test]
    fn every_registration_is_expected_at_its_level() {
        let filter = RuleFilterConfig::default();
        let a = expected_rules(WcagLevel::A, &filter, true);
        let aaa = expected_rules(WcagLevel::AAA, &filter, true);
        for rule in SHARED_RULES {
            assert!(ids(&a).contains(rule.id), "{}", rule.id);
        }
        for rule in TREE_RULES {
            assert_eq!(
                ids(&a).contains(rule.axe_id),
                rule.min_level == WcagLevel::A,
                "{}",
                rule.axe_id
            );
            assert!(ids(&aaa).contains(rule.axe_id), "{}", rule.axe_id);
        }
        for rule in PAGE_RULES {
            assert_eq!(
                ids(&a).contains(rule.rule_id),
                rule.min_level == WcagLevel::A,
                "{}",
                rule.rule_id
            );
            for id in rule.outcome_ids {
                assert!(ids(&aaa).contains(id), "{id}");
            }
        }
    }

    #[test]
    fn filter_removes_tree_rules_but_not_shared_ones() {
        let filter = RuleFilterConfig {
            disabled_rules: vec!["image-alt".into(), SHARED_RULES[0].id.into()],
            enabled_only_rules: vec![],
        };
        let expected = expected_rules(WcagLevel::AA, &filter, false);
        assert!(!ids(&expected).contains("image-alt"));
        assert!(ids(&expected).contains(SHARED_RULES[0].id));
    }

    #[test]
    fn inline_rules_are_mobile_only_and_gated() {
        let filter = RuleFilterConfig::default();
        let aa = expected_rules(WcagLevel::AA, &filter, true);
        for id in [REFLOW_RULE.axe_id, HTML_CONTENT_MODEL_RULE.axe_id] {
            let rule = aa.iter().find(|e| e.rule_id == id).unwrap();
            assert_eq!(rule.viewports, vec![MOBILE], "{id}");
        }
        let a = expected_rules(WcagLevel::A, &filter, false);
        assert!(!ids(&a).contains(REFLOW_RULE.axe_id));
        assert!(!ids(&a).contains(HTML_CONTENT_MODEL_RULE.axe_id));
    }

    /// The frame pass only runs ids the page registers, so folding its
    /// outcomes cannot introduce an unexpected one.
    #[test]
    fn frame_rules_are_registered_rules() {
        use crate::audit::frames::{FRAME_SHARED_RULES, FRAME_TREE_RULES};
        let all = expected_rules(WcagLevel::AAA, &RuleFilterConfig::default(), true);
        for id in FRAME_TREE_RULES.iter().chain(FRAME_SHARED_RULES) {
            assert!(ids(&all).contains(id), "{id}");
        }
    }

    #[test]
    fn executor_and_concrete_rules_are_one_check() {
        let rule = PAGE_RULES
            .iter()
            .find(|r| r.outcome_ids.len() > 1)
            .expect("a page rule with several concrete ids");
        let mut outcomes = vec![RuleRun::not_run(rule.rule_id, crate::wcag::NotRun::Errored)];
        for id in rule.outcome_ids {
            outcomes.push(RuleRun::not_run(*id, crate::wcag::NotRun::Errored));
        }
        assert_eq!(count_checks(&outcomes, crate::wcag::rule_run_errored), 1);
        // Another viewport is another execution.
        outcomes
            .push(RuleRun::not_run(rule.rule_id, crate::wcag::NotRun::Errored).in_viewport(MOBILE));
        assert_eq!(count_checks(&outcomes, crate::wcag::rule_run_errored), 2);
    }

    /// The static and the live half of `context/on-input` are one rule: one
    /// outcome per viewport, findings added, a failed half fails the rule.
    #[test]
    fn two_halves_of_one_rule_fold_into_one_outcome() {
        use crate::wcag::{NotRun, WcagResults};
        let mut results = WcagResults::new();
        results.fold_outcome(RuleRun::ran("context/on-input", 1).in_viewport(MOBILE));
        results.fold_outcome(RuleRun::ran("context/on-input", 2).in_viewport(MOBILE));
        results.fold_outcome(RuleRun::ran("context/on-input", 5).in_viewport(DESKTOP));
        assert_eq!(results.rule_outcomes.len(), 2);
        assert_eq!(results.rule_outcomes[0].findings, 3);
        results.fold_outcome(
            RuleRun::not_run("context/on-input", NotRun::Errored).in_viewport(MOBILE),
        );
        assert_eq!(results.rule_outcomes.len(), 2);
        assert_eq!(results.rule_outcomes[0].not_run, Some(NotRun::Errored));
    }
}
