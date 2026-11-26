//! Which audited page of a batch is worth a single report (#1212).
//!
//! The rule is explicit, measured and deterministic; it reads findings only,
//! never a guess about the rest of the site:
//!
//! 1. A *finding type* is a rule (`aggregation_key`) with a finding on the
//!    page, as the batch's top actions count them (`batch_listed_findings`,
//!    restricted to the rules the batch lists). Findings known from a
//!    baseline are left out.
//! 2. The weight of a page is the sum, over its distinct finding types, of
//!    the weight of the type's severity on that page: critical 4, high 3,
//!    medium 2, low 1. Occurrence counts do not enter: 70 duplicate ids are
//!    one type, not 70.
//! 3. The recommended page has the highest weight. Ties go to the lower
//!    accessibility score, then to more critical or high types, then to the
//!    URL in alphabetical order.
//! 4. When every audited page has the same accessibility score, the score
//!    cannot tell the pages apart; the pick then also names the most severe
//!    finding of another module on the page, preferring one that is not on
//!    every page.
//!
//! No pick for fewer than two pages (nothing to choose from) and none when
//! no page has a finding.

use std::cmp::Reverse;
use std::collections::{BTreeMap, HashMap};

use crate::audit::normalized::batch_listed_findings;
use crate::audit::NormalizedReport;
use crate::output::report_model::*;
use crate::wcag::Severity;

/// The weight of a finding type of this severity on a page.
fn severity_weight(severity: Severity) -> u32 {
    match severity {
        Severity::Critical => 4,
        Severity::High => 3,
        Severity::Medium => 2,
        Severity::Low => 1,
    }
}

/// The module name of `module_scores` for a finding's domain; `None` for the
/// accessibility domains.
fn module_of(domain: crate::audit::finding_contract::FindingDomain) -> Option<&'static str> {
    use crate::audit::finding_contract::FindingDomain as D;
    match domain {
        D::Accessibility | D::AccessibilityJourney | D::ScreenReader => None,
        D::Seo => Some("SEO"),
        D::Security => Some("Security"),
        D::Mobile => Some("Mobile"),
        D::Performance => Some("Performance"),
        D::HtmlConformance => Some("HTML Conformance"),
        D::BestPractices => Some("Best Practices"),
        D::Ux => Some("UX"),
        D::Journey => Some("Journey"),
        D::Commerce => Some("Commerce"),
        D::Privacy => Some("Privacy"),
    }
}

/// What a page has of one finding type.
struct OnPage {
    severity: Severity,
    occurrences: usize,
    module: Option<&'static str>,
}

/// The focus of every page (in the order of `reports`), the recommended
/// page and the spread of the finding types. `groups` are the batch's
/// aggregated findings in action order, the same list as the top actions.
pub(super) fn derive_focus(
    reports: &[NormalizedReport],
    groups: &[FindingGroup],
) -> (Vec<PageFocus>, Option<SingleReportPick>, FindingSpread) {
    let title_of: HashMap<&str, &str> = groups
        .iter()
        .map(|g| (g.finding.rule_id.as_str(), g.title.as_str()))
        .collect();

    let per_page: Vec<BTreeMap<&str, OnPage>> = reports
        .iter()
        .map(|report| {
            let mut types: BTreeMap<&str, OnPage> = BTreeMap::new();
            for finding in batch_listed_findings(report) {
                if finding.known || !title_of.contains_key(finding.aggregation_key.as_str()) {
                    continue;
                }
                let entry = types
                    .entry(finding.aggregation_key.as_str())
                    .or_insert(OnPage {
                        severity: finding.severity,
                        occurrences: 0,
                        module: module_of(finding.contract.domain),
                    });
                entry.severity = entry.severity.max(finding.severity);
                entry.occurrences += finding.occurrence_count;
            }
            types
        })
        .collect();

    let mut pages_of: HashMap<&str, usize> = HashMap::new();
    for types in &per_page {
        for rule in types.keys() {
            *pages_of.entry(rule).or_default() += 1;
        }
    }
    let total_pages = reports.len();
    let spread = FindingSpread {
        types: pages_of.len(),
        on_all_pages: pages_of.values().filter(|n| **n == total_pages).count(),
        shared: pages_of.values().filter(|n| **n >= 2).count(),
        page_specific: pages_of.values().filter(|n| **n == 1).count(),
        pages_with_findings: per_page.iter().filter(|t| !t.is_empty()).count(),
    };

    // The action order of the groups, so every list of types reads the same.
    let order: HashMap<&str, usize> = groups
        .iter()
        .enumerate()
        .map(|(i, g)| (g.finding.rule_id.as_str(), i))
        .collect();
    let focus_type = |rule: &str, on_page: &OnPage| FocusType {
        rule_id: rule.to_string(),
        title: title_of.get(rule).copied().unwrap_or(rule).to_string(),
        severity: on_page.severity,
        module: on_page.module,
        pages: pages_of.get(rule).copied().unwrap_or(0),
    };

    let focus: Vec<PageFocus> = reports
        .iter()
        .zip(&per_page)
        .map(|(report, types)| {
            let mut only_here: Vec<FocusType> = types
                .iter()
                .filter(|(rule, _)| pages_of.get(*rule) == Some(&1))
                .map(|(rule, on_page)| focus_type(rule, on_page))
                .collect();
            only_here.sort_by_key(|t| order.get(t.rule_id.as_str()).copied());
            PageFocus {
                url: report.url.clone(),
                weight: types.values().map(|t| severity_weight(t.severity)).sum(),
                types: types.len(),
                severe_types: types
                    .values()
                    .filter(|t| t.severity >= Severity::High)
                    .count(),
                only_here,
            }
        })
        .collect();

    let pick = (total_pages >= 2)
        .then(|| {
            let best = (0..total_pages).min_by_key(|&i| {
                (
                    Reverse(focus[i].weight),
                    reports[i].score,
                    Reverse(focus[i].severe_types),
                    reports[i].url.as_str(),
                )
            })?;
            (focus[best].weight > 0).then_some(best)
        })
        .flatten()
        .map(|best| {
            let types = &per_page[best];
            let scores_tied = reports.iter().all(|r| r.score == reports[0].score);
            let other_module = scores_tied
                .then(|| {
                    let mut candidates: Vec<(&str, &OnPage)> = types
                        .iter()
                        .filter(|(_, t)| t.module.is_some())
                        .map(|(rule, t)| (*rule, t))
                        .collect();
                    let specific = candidates
                        .iter()
                        .any(|(rule, _)| pages_of.get(rule) != Some(&total_pages));
                    if specific {
                        candidates.retain(|(rule, _)| pages_of.get(rule) != Some(&total_pages));
                    }
                    candidates
                        .into_iter()
                        .min_by_key(|(rule, t)| {
                            (Reverse(t.severity), Reverse(t.occurrences), *rule)
                        })
                        .map(|(rule, t)| focus_type(rule, t))
                })
                .flatten();
            let lead_shared_rule = groups
                .iter()
                .map(|g| g.finding.rule_id.as_str())
                .find(|rule| {
                    types.contains_key(rule) && pages_of.get(rule).is_some_and(|n| *n >= 2)
                })
                .map(str::to_string);
            SingleReportPick {
                url: reports[best].url.clone(),
                weight: focus[best].weight,
                types: focus[best].types,
                severe_types: focus[best].severe_types,
                shared_types_on_page: types
                    .keys()
                    .filter(|rule| pages_of.get(*rule).is_some_and(|n| *n >= 2))
                    .count(),
                shared_types: spread.shared,
                scores_tied,
                other_module,
                lead_shared_rule,
            }
        });

    (focus, pick, spread)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::{AuditReport, BatchReport};
    use crate::model::WcagLevel;
    use crate::wcag::{Violation, WcagResults};

    fn page(url: &str, rules: &[(&str, Severity)]) -> AuditReport {
        let mut results = WcagResults::new();
        results.nodes_checked = 100;
        for (rule, severity) in rules {
            results.add_violation(
                Violation::new(
                    "1.1.1",
                    "Non-text Content",
                    WcagLevel::A,
                    *severity,
                    "message",
                    "n1",
                )
                .with_rule_id(*rule)
                .with_selector("main > a"),
            );
        }
        AuditReport::new(url.to_string(), WcagLevel::AA, results, 100)
    }

    fn pres(pages: Vec<AuditReport>) -> BatchPresentation {
        let batch = BatchReport::from_reports(pages, vec![], 100);
        crate::output::builder::build_batch_presentation_with_locale(
            &batch,
            &crate::i18n::I18n::new("en").unwrap(),
        )
    }

    #[test]
    fn the_page_with_the_most_weighted_finding_types_is_picked() {
        let pres = pres(vec![
            page(
                "https://x.test/a",
                &[("images/alt-missing", Severity::High)],
            ),
            page(
                "https://x.test/b",
                &[
                    ("images/alt-missing", Severity::High),
                    ("buttons/name-missing", Severity::Critical),
                ],
            ),
            page(
                "https://x.test/c",
                &[("images/alt-missing", Severity::High)],
            ),
        ]);
        let pick = pres.single_report_pick.expect("a pick");
        assert_eq!(pick.url, "https://x.test/b");
        assert_eq!(pick.types, 2);
        assert_eq!(pick.weight, 3 + 4);
        assert_eq!(pick.severe_types, 2);
        assert_eq!(pres.finding_spread.types, 2);
        assert_eq!(pres.finding_spread.on_all_pages, 1);
        assert_eq!(pres.finding_spread.shared, 1);
        assert_eq!(pres.finding_spread.page_specific, 1);
        let focus_b = &pres.page_focus[1];
        assert_eq!(focus_b.only_here.len(), 1);
        assert_eq!(focus_b.only_here[0].rule_id, "buttons/name-missing");
    }

    #[test]
    fn occurrences_do_not_weigh_and_ties_resolve_by_url() {
        let pres = pres(vec![
            page(
                "https://x.test/b",
                &[("images/alt-missing", Severity::High)],
            ),
            page(
                "https://x.test/a",
                &[
                    ("images/alt-missing", Severity::High),
                    ("images/alt-missing", Severity::High),
                    ("images/alt-missing", Severity::High),
                ],
            ),
        ]);
        let pick = pres.single_report_pick.expect("a pick");
        assert_eq!(pick.url, "https://x.test/a");
    }

    #[test]
    fn no_pick_for_one_page_or_without_findings() {
        let one = pres(vec![page(
            "https://x.test/a",
            &[("images/alt-missing", Severity::High)],
        )]);
        assert!(one.single_report_pick.is_none());
        let clean = pres(vec![
            page("https://x.test/a", &[]),
            page("https://x.test/b", &[]),
        ]);
        assert!(clean.single_report_pick.is_none());
        assert_eq!(clean.finding_spread, FindingSpread::default());
    }
}
