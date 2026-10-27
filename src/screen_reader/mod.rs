//! Screen reader audit primitives built from the browser AXTree.

pub mod analyzer;
pub mod announcer;
pub mod bfsg;
pub mod navigator;
pub mod types;

pub use a11y_perception::{linearize, linearize_with_ignored};
pub use analyzer::{analyze_reading_sequence, name_quality_score};
pub use announcer::{announce, announce_localized};
pub use bfsg::{map_to_bfsg, wcag_21_aa_criteria, BfsgMapping};
pub use navigator::{navigation_views, NavigationViews};
pub use types::{
    AnnouncedReadingItem, BfsgCompliance, BfsgVerdict, BfsgViolation, IgnoredReadingNode,
    QualityScoreContext, ReadingItem, ScreenReaderSummary, SrAuditIssue, SrAuditQuality,
    SrAuditReport, SrAuditSummary,
};

use crate::accessibility::AXTree;
use crate::i18n::I18n;
use crate::wcag::coverage::{ClaimLimits, ManualReview};

/// Builds the screen-reader audit report. All issue messages and announcements
/// are baked in canonical English so the stored struct, JSON envelope and
/// sidecar file are language-neutral (#406). The PDF presentation layer
/// re-derives localized issues from the reading sequence.
///
/// `detect_locale` is the page/run language used only for *detection* (loading
/// the generic-link-text stopword list and scoring name quality), independent
/// of the English message language — so German pages keep flagging generic
/// German link texts even though messages are stored in English.
///
/// `pattern_analysis`, when available, lets the missing-navigation-landmark
/// check (#504) recognize a collapsed hamburger/disclosure menu instead of
/// reporting the navigation as unreachable — the static AXTree captured
/// before interaction never contains the collapsed nav.
pub fn build_sr_audit_report(
    url: &str,
    timestamp: chrono::DateTime<chrono::Utc>,
    tree: &AXTree,
    detect_locale: &str,
    pattern_analysis: Option<&crate::patterns::PatternAnalysis>,
) -> SrAuditReport {
    let i18n = I18n::new("en").expect("english locale parses");
    let reading_items = linearize(tree);
    let navigation_views = navigation_views(&reading_items);
    let has_disclosure_menu_pattern =
        pattern_analysis.is_some_and(|patterns| patterns.has_recognized("DisclosureMenu"));
    // Detection uses the page/run language so generic link-text stopwords match;
    // messages are baked in canonical English for the language-neutral struct.
    let issues = analyze_reading_sequence(
        &reading_items,
        &navigation_views,
        detect_locale,
        true,
        has_disclosure_menu_pattern,
    );
    let audit_quality = detect_audit_quality(reading_items.len(), &navigation_views);
    let bfsg_compliance = bfsg_compliance(&issues, reading_items.len(), audit_quality);
    let reading_sequence = reading_items
        .iter()
        .cloned()
        .map(|item| AnnouncedReadingItem {
            announcement: announce_localized(&item, &i18n),
            item,
        })
        .collect::<Vec<_>>();

    SrAuditReport {
        schema_version: "1.0",
        report_type: "screen_reader_audit",
        url: url.to_string(),
        timestamp,
        tool_version: env!("CARGO_PKG_VERSION"),
        summary: SrAuditSummary {
            total_announced_nodes: reading_items.len(),
            tab_stops: reading_items.iter().filter(|item| item.tab_stop).count(),
            bfsg_violations: bfsg_compliance.violations.len(),
            name_quality_score: name_quality_score(&reading_items, detect_locale),
            landmark_quality_score: landmark_quality_score(&navigation_views),
            heading_quality_score: heading_quality_score(&navigation_views),
            quality_score_context: QualityScoreContext::default(),
            audit_quality,
        },
        reading_sequence,
        navigation_views,
        issues,
        bfsg_compliance,
        excluded_node_ids: Default::default(),
    }
}

impl SrAuditReport {
    /// Applies the audit exclusions (#645) to the issues (#703): drops the
    /// nodes for which `is_excluded` answers true, then re-derives the BFSG
    /// verdict, so neither it nor the legal flags built on it (#484) count an
    /// excluded specimen. Returns the number of issues dropped entirely.
    pub fn apply_exclusion(&mut self, is_excluded: impl Fn(&str) -> bool) -> usize {
        let excluded: std::collections::BTreeSet<String> = self
            .issues
            .iter()
            .flat_map(|issue| &issue.affected_node_ids)
            .filter(|id| is_excluded(id))
            .cloned()
            .collect();
        if excluded.is_empty() {
            return 0;
        }
        let dropped = drop_excluded_nodes(&mut self.issues, &excluded);
        self.bfsg_compliance = bfsg_compliance(
            &self.issues,
            self.summary.total_announced_nodes,
            self.summary.audit_quality,
        );
        self.summary.bfsg_violations = self.bfsg_compliance.violations.len();
        self.excluded_node_ids = excluded;
        dropped
    }
}

/// Removes `excluded` from every issue's node list and drops an issue once
/// all of its nodes were excluded; returns how many were dropped. An issue
/// that names no node (a page-level finding) is never dropped — same rule as
/// the main pass (`audit::exclusion`). Messages that quote a page-wide count
/// (tab stops, repeated link texts) keep it: that count describes the page,
/// which the exclusion does not change.
pub fn drop_excluded_nodes(
    issues: &mut Vec<SrAuditIssue>,
    excluded: &std::collections::BTreeSet<String>,
) -> usize {
    if excluded.is_empty() {
        return 0;
    }
    let before = issues.len();
    issues.retain_mut(|issue| {
        if issue.affected_node_ids.is_empty() {
            return true;
        }
        issue.affected_node_ids.retain(|id| !excluded.contains(id));
        !issue.affected_node_ids.is_empty()
    });
    before - issues.len()
}

/// Flags a likely consent-blocked audit (#483): too few announced nodes and no
/// structural landmark (banner/navigation/main/contentinfo). On such pages the
/// quality scores and BFSG verdict reflect an audit limitation, not a genuine
/// accessibility failure, so downstream consumers can qualify the findings.
fn detect_audit_quality(total_nodes: usize, views: &NavigationViews) -> SrAuditQuality {
    // Recalibrated from 200 when `linearize` stopped emitting `InlineTextBox`
    // layout nodes: those made up roughly a quarter to a third of the reading
    // order (67 of 237 on www.sachsen-anhalt.de, 2026-09-17), so the old value
    // has to shrink by about the same share to keep flagging the same pages.
    const CONSENT_WALL_NODE_THRESHOLD: usize = 145;
    let has_structural_landmark = views.landmarks.iter().any(|l| {
        matches!(
            l.role.as_str(),
            "banner" | "navigation" | "main" | "contentinfo"
        ) && l.quality != navigator::LandmarkQuality::MissingMain
    });
    if total_nodes < CONSENT_WALL_NODE_THRESHOLD && !has_structural_landmark {
        SrAuditQuality::ConsentWallSuspected
    } else {
        SrAuditQuality::Ok
    }
}

fn landmark_quality_score(views: &NavigationViews) -> u32 {
    // Score based on presence of all four required ARIA landmark roles.
    // A page with only `main` used to score 100 % — that was misleading.
    const REQUIRED: &[&str] = &["banner", "navigation", "contentinfo", "main"];
    let present_count = REQUIRED
        .iter()
        .filter(|&&role| {
            views
                .landmarks
                .iter()
                .any(|l| l.role == role && l.quality != navigator::LandmarkQuality::MissingMain)
        })
        .count();
    percentage(present_count, REQUIRED.len())
}

fn heading_quality_score(views: &NavigationViews) -> u32 {
    percentage(
        views
            .headings
            .iter()
            .filter(|item| item.quality == navigator::HeadingQuality::Good)
            .count(),
        views.headings.len(),
    )
}

fn percentage(passed: usize, total: usize) -> u32 {
    if total == 0 {
        100
    } else {
        ((passed as f64 / total as f64) * 100.0).round() as u32
    }
}

/// WCAG 2.1 A/AA criteria this module's analyzer (`analyzer.rs`) can actually
/// detect issues for -- landmark/heading/name/reading-order structural checks
/// only. Used to scope `criteria_without_findings` to the module's real evaluation
/// coverage instead of claiming all 50 WCAG 2.1 A/AA criteria as "passed"
/// just because no violation happened to be found for them: this module
/// never looks at contrast, keyboard operability, timing, forms, parsing,
/// etc., so those criteria were never evaluated at all, not passed.
/// Confirmed live in the 2026-08-31 report corpus (sauerstoffzentrum-nordost.de):
/// 7 real structural issues were detected, all tagged "1.3.6" (deliberately
/// outside the BFSG A/AA scope, see `push_desert_issue`'s doc comment), so
/// `violations` was empty and the report claimed all 50 criteria "passed".
/// Keep this list in sync with the `wcag_criterion: Some("...")` call sites
/// in `analyzer.rs` that are actually WCAG A/AA (not AAA -- "1.3.6" is
/// deliberately excluded here for the same reason it's excluded from BFSG
/// violations).
const EVALUATED_WCAG_CRITERIA: &[&str] = &["1.3.1", "2.4.1", "2.4.4", "2.4.6", "3.3.2", "4.1.2"];

/// The verdict states only what was found on `EVALUATED_WCAG_CRITERIA`: an
/// empty tree is not evaluated, a suspected consent wall without findings is
/// incomplete, and no finding is its own state — never conformance (#782).
fn bfsg_compliance(
    issues: &[SrAuditIssue],
    total_nodes: usize,
    audit_quality: SrAuditQuality,
) -> BfsgCompliance {
    let mapped = |review: bool| {
        issues
            .iter()
            .filter(|issue| issue.review == review)
            .filter_map(|issue| {
                let wcag = issue.wcag_criterion.as_deref()?;
                let mapping = map_to_bfsg(wcag)?;
                mapping.fix_required.then(|| BfsgViolation {
                    wcag_criterion: wcag.to_string(),
                    en_301_549_clause: Some(mapping.en_301549_clause.to_string()),
                    bfsg_reference: Some(mapping.bfsg_paragraph.to_string()),
                    fix_required: mapping.fix_required,
                    affected_node_ids: issue.affected_node_ids.clone(),
                })
            })
            .collect::<Vec<_>>()
    };
    // Heuristics are no violation, and a criterion with an open heuristic is
    // not passed either (plan 74).
    let violations = mapped(false);
    let needs_review = mapped(true);
    let open: std::collections::HashSet<_> = violations
        .iter()
        .chain(&needs_review)
        .map(|violation| violation.wcag_criterion.as_str())
        .collect();
    let criteria_without_findings = EVALUATED_WCAG_CRITERIA
        .iter()
        .filter(|criterion| !open.contains(*criterion))
        .map(|criterion| criterion.to_string())
        .collect();

    let verdict = if total_nodes == 0 {
        BfsgVerdict::NotEvaluated
    } else if !violations.is_empty() {
        BfsgVerdict::ViolationsFound
    } else if !needs_review.is_empty() {
        BfsgVerdict::NeedsReview
    } else if audit_quality == SrAuditQuality::ConsentWallSuspected {
        BfsgVerdict::Incomplete
    } else {
        BfsgVerdict::NoViolationsFound
    };
    let mut remaining_manual_review = vec![
        ManualReview::ScreenReaderReferenceTest,
        ManualReview::CriteriaOutsideAutomatedScope,
    ];
    if !needs_review.is_empty() {
        remaining_manual_review.push(ManualReview::ReviewHeuristicFindings);
    }
    if total_nodes == 0 || audit_quality == SrAuditQuality::ConsentWallSuspected {
        remaining_manual_review.push(ManualReview::RepeatIncompleteRun);
    }

    BfsgCompliance {
        verdict,
        violations,
        evaluated_criteria: EVALUATED_WCAG_CRITERIA
            .iter()
            .map(|criterion| criterion.to_string())
            .collect(),
        criteria_without_findings,
        needs_review,
        limits: ClaimLimits::with_manual_review(remaining_manual_review),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::screen_reader::navigator::navigation_views;
    use crate::screen_reader::ReadingItem;
    use crate::wcag::coverage::Determination;

    fn item(seq: usize, role: &str) -> ReadingItem {
        ReadingItem {
            seq,
            role: Some(role.to_string()),
            name: Some(role.to_string()),
            description: None,
            value: None,
            states: vec![],
            tab_stop: false,
            depth: 0,
            node_id: format!("node-{seq}"),
        }
    }

    fn issue(wcag_criterion: &str) -> SrAuditIssue {
        SrAuditIssue {
            review: false,
            wcag_criterion: Some(wcag_criterion.to_string()),
            severity: "low".into(),
            affected_node_ids: vec![],
            message: "test issue".into(),
        }
    }

    /// A run over a complete, non-empty tree.
    fn complete_run(issues: &[SrAuditIssue]) -> BfsgCompliance {
        bfsg_compliance(issues, 200, SrAuditQuality::Ok)
    }

    /// Plan 74 (inros-lackner 2026-10-23): "Mehr erfahren" without context
    /// and the heading order are heuristics — they made the sidecar
    /// "non_compliant" next to the confirmed empty links. Review items are
    /// neither violations nor passed.
    #[test]
    fn heuristic_issues_need_review_instead_of_failing_bfsg() {
        let review = |wcag: &str| SrAuditIssue {
            review: true,
            ..issue(wcag)
        };
        let compliance = complete_run(&[review("2.4.4"), review("1.3.1")]);
        assert_eq!(compliance.verdict, BfsgVerdict::NeedsReview);
        assert!(compliance.violations.is_empty());
        assert_eq!(compliance.needs_review.len(), 2);
        assert!(!compliance
            .criteria_without_findings
            .contains(&"2.4.4".to_string()));

        let compliance = complete_run(&[review("2.4.4"), issue("4.1.2")]);
        assert_eq!(compliance.verdict, BfsgVerdict::ViolationsFound);
        assert_eq!(compliance.violations.len(), 1);
    }

    #[test]
    fn bfsg_compliance_does_not_claim_unevaluated_criteria_as_passed() {
        // Regression (sauerstoffzentrum-nordost.de, 2026-08-31): a page with
        // 7 real structural issues, all tagged "1.3.6" (deliberately outside
        // BFSG A/AA scope), previously still produced verdict "compliant"
        // with all 50 WCAG 2.1 A/AA criteria listed as "passed" -- most of
        // which (contrast, keyboard, timing, forms, ...) this module never
        // evaluates at all. criteria_without_findings must only ever contain this
        // module's actual evaluation scope.
        let issues = vec![issue("1.3.6"), issue("1.3.6"), issue("1.3.6")];
        let compliance = complete_run(&issues);
        assert_eq!(compliance.verdict, BfsgVerdict::NoViolationsFound);
        assert!(compliance.violations.is_empty());
        assert_eq!(
            compliance.criteria_without_findings.len(),
            EVALUATED_WCAG_CRITERIA.len()
        );
        for criterion in EVALUATED_WCAG_CRITERIA {
            assert!(
                compliance
                    .criteria_without_findings
                    .contains(&criterion.to_string()),
                "expected {criterion} in criteria_without_findings"
            );
        }
        // Never claims a criterion outside its own evaluation scope, e.g. the
        // contrast/keyboard/forms criteria this module never looks at.
        assert!(!compliance
            .criteria_without_findings
            .contains(&"1.4.3".to_string()));
        assert!(!compliance
            .criteria_without_findings
            .contains(&"2.1.1".to_string()));
    }

    #[test]
    fn bfsg_compliance_excludes_a_failed_criterion_from_criteria_without_findings() {
        let issues = vec![issue("2.4.4")];
        let compliance = complete_run(&issues);
        assert_eq!(compliance.verdict, BfsgVerdict::ViolationsFound);
        assert_eq!(compliance.violations.len(), 1);
        assert!(!compliance
            .criteria_without_findings
            .contains(&"2.4.4".to_string()));
        assert_eq!(
            compliance.criteria_without_findings.len(),
            EVALUATED_WCAG_CRITERIA.len() - 1
        );
    }

    /// #782: no finding is its own state, never conformance — the verdict
    /// says it was bounded by the evaluated criteria, and the JSON says that
    /// conformance, BFSG applicability and a real screen-reader run are open.
    #[test]
    fn no_findings_never_claim_conformance_or_legal_applicability() {
        let compliance = complete_run(&[]);
        assert_eq!(compliance.verdict, BfsgVerdict::NoViolationsFound);
        assert_eq!(
            compliance.evaluated_criteria.len(),
            EVALUATED_WCAG_CRITERIA.len()
        );
        assert_eq!(compliance.limits.conformance, Determination::NotDetermined);
        assert_eq!(
            compliance.limits.legal_applicability,
            Determination::NotDetermined
        );
        assert!(compliance
            .limits
            .remaining_manual_review
            .contains(&ManualReview::ScreenReaderReferenceTest));

        let json = serde_json::to_value(&compliance).unwrap();
        assert_eq!(json["verdict"], "no_violations_found");
        assert_eq!(json["conformance"], "not_determined");
        assert_eq!(json["legal_applicability"], "not_determined");
        assert!(json.get("passed_criteria").is_none());
        assert!(!json.to_string().contains("compliant"));
    }

    /// #782: an empty tree audited nothing, and a suspected consent wall
    /// without findings proves nothing — neither may read as a clean result.
    #[test]
    fn empty_or_incomplete_runs_are_not_a_clean_result() {
        let empty = bfsg_compliance(&[], 0, SrAuditQuality::Ok);
        assert_eq!(empty.verdict, BfsgVerdict::NotEvaluated);
        assert!(empty
            .limits
            .remaining_manual_review
            .contains(&ManualReview::RepeatIncompleteRun));

        let walled = bfsg_compliance(&[], 12, SrAuditQuality::ConsentWallSuspected);
        assert_eq!(walled.verdict, BfsgVerdict::Incomplete);
        assert!(walled
            .limits
            .remaining_manual_review
            .contains(&ManualReview::RepeatIncompleteRun));

        // A confirmed violation stays one, even on an incomplete tree.
        let walled = bfsg_compliance(&[issue("4.1.2")], 12, SrAuditQuality::ConsentWallSuspected);
        assert_eq!(walled.verdict, BfsgVerdict::ViolationsFound);
    }

    /// The full builder on an empty accessibility tree: not evaluated.
    #[test]
    fn report_over_an_empty_tree_is_not_evaluated() {
        let report = build_sr_audit_report(
            "https://x.test/",
            chrono::Utc::now(),
            &AXTree::new(),
            "en",
            None,
        );
        assert_eq!(report.bfsg_compliance.verdict, BfsgVerdict::NotEvaluated);
    }

    #[test]
    fn landmark_quality_score_only_main_is_low() {
        let views = navigation_views(&[item(0, "main")]);
        // 1 of 4 required landmarks → 25 %
        assert_eq!(landmark_quality_score(&views), 25);
    }

    #[test]
    fn landmark_quality_score_all_required_is_100() {
        let views = navigation_views(&[
            item(0, "banner"),
            item(1, "navigation"),
            item(2, "main"),
            item(3, "contentinfo"),
        ]);
        assert_eq!(landmark_quality_score(&views), 100);
    }

    #[test]
    fn small_page_with_structural_landmarks_is_not_flagged_as_consent_wall() {
        // plan/33-screen-reader-thresholds-unvalidated.md: a small but
        // legitimate page (well under the 145-node threshold) must not be
        // mistaken for a consent-wall capture as long as it has real
        // structural landmarks — the threshold is deliberately a two-factor
        // check (few nodes AND no landmarks), not node count alone.
        let items = vec![
            item(0, "banner"),
            item(1, "navigation"),
            item(2, "main"),
            item(3, "contentinfo"),
        ];
        let views = navigation_views(&items);
        assert_eq!(detect_audit_quality(10, &views), SrAuditQuality::Ok);
    }

    #[test]
    fn small_page_without_landmarks_is_flagged_as_consent_wall() {
        let views = navigation_views(&[item(0, "generic")]);
        assert_eq!(
            detect_audit_quality(10, &views),
            SrAuditQuality::ConsentWallSuspected
        );
    }

    #[test]
    fn large_page_without_landmarks_is_not_flagged_as_consent_wall() {
        // Node count alone must not trip the heuristic — a genuinely large,
        // just poorly-landmarked page is a real (separate) finding, not a
        // consent-wall false positive.
        let views = navigation_views(&[item(0, "generic")]);
        assert_eq!(detect_audit_quality(200, &views), SrAuditQuality::Ok);
    }

    fn high(wcag: &str, nodes: &[&str]) -> SrAuditIssue {
        SrAuditIssue {
            review: false,
            wcag_criterion: Some(wcag.to_string()),
            severity: "high".into(),
            affected_node_ids: nodes.iter().map(|n| n.to_string()).collect(),
            message: "m".into(),
        }
    }

    fn report_with(issues: Vec<SrAuditIssue>) -> SrAuditReport {
        let mut report = build_sr_audit_report(
            "https://x.test/",
            chrono::Utc::now(),
            &AXTree::new(),
            "en",
            None,
        );
        report.summary.total_announced_nodes = 200;
        report.summary.audit_quality = SrAuditQuality::Ok;
        report.bfsg_compliance = complete_run(&issues);
        report.summary.bfsg_violations = report.bfsg_compliance.violations.len();
        report.issues = issues;
        report
    }

    #[test]
    fn excluded_specimen_fields_no_longer_fail_the_bfsg_verdict() {
        // #703: both unlabelled inputs sit inside [data-specimen].
        let mut report = report_with(vec![high("3.3.2", &["7", "9"])]);
        assert_eq!(report.bfsg_compliance.verdict, BfsgVerdict::ViolationsFound);
        let dropped = report.apply_exclusion(|id| id == "7" || id == "9");
        assert_eq!(dropped, 1);
        assert!(report.issues.is_empty());
        assert_eq!(
            report.bfsg_compliance.verdict,
            BfsgVerdict::NoViolationsFound
        );
        assert_eq!(report.summary.bfsg_violations, 0);
        assert!(report
            .bfsg_compliance
            .criteria_without_findings
            .contains(&"3.3.2".to_string()));
        assert_eq!(report.excluded_node_ids.len(), 2);
    }

    #[test]
    fn an_issue_with_nodes_outside_the_exclusion_keeps_only_those() {
        let mut report = report_with(vec![high("3.3.2", &["7", "12"])]);
        let dropped = report.apply_exclusion(|id| id == "7");
        assert_eq!(dropped, 0);
        assert_eq!(report.issues[0].affected_node_ids, vec!["12".to_string()]);
        assert_eq!(report.bfsg_compliance.verdict, BfsgVerdict::ViolationsFound);
        assert_eq!(
            report.bfsg_compliance.violations[0].affected_node_ids,
            vec!["12".to_string()]
        );
    }

    #[test]
    fn page_level_issues_are_never_excluded() {
        let mut issues = vec![high("2.4.1", &[]), high("4.1.2", &["3"])];
        let excluded = ["3".to_string()].into_iter().collect();
        assert_eq!(drop_excluded_nodes(&mut issues, &excluded), 1);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].wcag_criterion.as_deref(), Some("2.4.1"));
    }
}
