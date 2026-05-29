//! Screen reader audit primitives built from the browser AXTree.

pub mod analyzer;
pub mod announcer;
pub mod bfsg;
pub mod linearizer;
pub mod navigator;
pub mod types;

pub use analyzer::{analyze_reading_sequence, name_quality_score};
pub use announcer::{announce, announce_localized};
pub use bfsg::{map_to_bfsg, wcag_21_aa_criteria, BfsgMapping};
pub use linearizer::{linearize, linearize_with_ignored};
pub use navigator::{navigation_views, NavigationViews};
pub use types::{
    AnnouncedReadingItem, BfsgCompliance, BfsgVerdict, BfsgViolation, IgnoredReadingNode,
    ReadingItem, SrAuditIssue, SrAuditReport, SrAuditSummary,
};

use crate::accessibility::AXTree;
use crate::i18n::I18n;

pub fn build_sr_audit_report(
    url: &str,
    timestamp: chrono::DateTime<chrono::Utc>,
    tree: &AXTree,
    locale: &str,
) -> SrAuditReport {
    let i18n =
        I18n::new(locale).unwrap_or_else(|_| I18n::new("de").expect("default locale parses"));
    let reading_items = linearize(tree);
    let navigation_views = navigation_views(&reading_items);
    let issues = analyze_reading_sequence(&reading_items, &navigation_views, locale);
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
            name_quality_score: name_quality_score(&reading_items, locale),
            landmark_quality_score: landmark_quality_score(&navigation_views),
            heading_quality_score: heading_quality_score(&navigation_views),
        },
        reading_sequence,
        navigation_views,
        issues,
        bfsg_compliance,
    }
}

fn landmark_quality_score(views: &NavigationViews) -> u32 {
    percentage(
        views
            .landmarks
            .iter()
            .filter(|item| item.quality == navigator::LandmarkQuality::Ok)
            .count(),
        views.landmarks.len(),
    )
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
                deadline: Some(mapping.deadline.to_string()),
                affected_node_ids: issue.affected_node_ids.clone(),
            })
        })
        .collect::<Vec<_>>();
    let failed: std::collections::HashSet<_> = violations
        .iter()
        .map(|violation| violation.wcag_criterion.as_str())
        .collect();
    let passed_criteria = wcag_21_aa_criteria()
        .iter()
        .filter(|criterion| !failed.contains(criterion.wcag))
        .map(|criterion| criterion.wcag.to_string())
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
