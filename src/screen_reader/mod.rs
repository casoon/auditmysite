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
    let bfsg_compliance = bfsg_compliance(&issues);
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
            audit_quality: detect_audit_quality(reading_items.len(), &navigation_views),
        },
        reading_sequence,
        navigation_views,
        issues,
        bfsg_compliance,
    }
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
/// only. Used to scope `passed_criteria` to the module's real evaluation
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

fn bfsg_compliance(issues: &[SrAuditIssue]) -> BfsgCompliance {
    let violations = issues
        .iter()
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
        .collect::<Vec<_>>();
    let failed: std::collections::HashSet<_> = violations
        .iter()
        .map(|violation| violation.wcag_criterion.as_str())
        .collect();
    let passed_criteria = EVALUATED_WCAG_CRITERIA
        .iter()
        .filter(|criterion| !failed.contains(*criterion))
        .map(|criterion| criterion.to_string())
        .collect();

    BfsgCompliance {
        verdict: if violations.is_empty() {
            BfsgVerdict::Compliant
        } else {
            BfsgVerdict::NonCompliant
        },
        violations,
        passed_criteria,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::screen_reader::navigator::navigation_views;
    use crate::screen_reader::ReadingItem;

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
            wcag_criterion: Some(wcag_criterion.to_string()),
            severity: "low".into(),
            affected_node_ids: vec![],
            message: "test issue".into(),
        }
    }

    #[test]
    fn bfsg_compliance_does_not_claim_unevaluated_criteria_as_passed() {
        // Regression (sauerstoffzentrum-nordost.de, 2026-08-31): a page with
        // 7 real structural issues, all tagged "1.3.6" (deliberately outside
        // BFSG A/AA scope), previously still produced verdict "compliant"
        // with all 50 WCAG 2.1 A/AA criteria listed as "passed" -- most of
        // which (contrast, keyboard, timing, forms, ...) this module never
        // evaluates at all. passed_criteria must only ever contain this
        // module's actual evaluation scope.
        let issues = vec![issue("1.3.6"), issue("1.3.6"), issue("1.3.6")];
        let compliance = bfsg_compliance(&issues);
        assert_eq!(compliance.verdict, BfsgVerdict::Compliant);
        assert!(compliance.violations.is_empty());
        assert_eq!(
            compliance.passed_criteria.len(),
            EVALUATED_WCAG_CRITERIA.len()
        );
        for criterion in EVALUATED_WCAG_CRITERIA {
            assert!(
                compliance.passed_criteria.contains(&criterion.to_string()),
                "expected {criterion} in passed_criteria"
            );
        }
        // Never claims a criterion outside its own evaluation scope, e.g. the
        // contrast/keyboard/forms criteria this module never looks at.
        assert!(!compliance.passed_criteria.contains(&"1.4.3".to_string()));
        assert!(!compliance.passed_criteria.contains(&"2.1.1".to_string()));
    }

    #[test]
    fn bfsg_compliance_excludes_a_failed_criterion_from_passed_criteria() {
        let issues = vec![issue("2.4.4")];
        let compliance = bfsg_compliance(&issues);
        assert_eq!(compliance.verdict, BfsgVerdict::NonCompliant);
        assert_eq!(compliance.violations.len(), 1);
        assert!(!compliance.passed_criteria.contains(&"2.4.4".to_string()));
        assert_eq!(
            compliance.passed_criteria.len(),
            EVALUATED_WCAG_CRITERIA.len() - 1
        );
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
}
