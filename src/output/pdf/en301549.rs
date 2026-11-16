//! EN 301 549 (chapter 9, "Web") clause annex — opt-in PDF appendix section.
//!
//! Only rendered when `--annex en301549` is passed (see `ReportConfig.annex`);
//! the underlying JSON `en301549_annex` block is always emitted regardless.
//! Sibling of `wcag_coverage.rs`; both sit in the appendix, directly after
//! each other.
//!
//! This annex is technical evidence only — never a Barrierefreiheitserklärung.
//! Its disclaimer describes the automated scope and the need for additional
//! manual testing; it makes no statutory citation, conformity claim or
//! harmonization claim.

use crate::output::localized::is_english;
use renderreport::components::advanced::{List, SectionHeaderSplit};
use renderreport::components::text::Label;
use renderreport::components::{AuditTable, TableColumn, TagCloud};
use renderreport::prelude::*;

use crate::audit::normalized::NormalizedFinding;
use crate::i18n::I18n;
use crate::wcag::en301549::{
    derive_annex, out_of_standard_finding_count, ClauseStatus, EN301549_DISCLAIMER_DE,
    EN301549_DISCLAIMER_EN, OUT_OF_SCOPE_CHAPTERS,
};
use fluent_bundle::FluentValue;

/// Render the EN 301 549 clause annex. Structure: intro -> disclaimer
/// callout -> violations table (or a clean confirmation line) -> "no
/// violations in automated scope" tag cloud -> "manual review required" tag
/// cloud -> out-of-scope-chapters note. The clean-site case never implies
/// full conformity — the latter three blocks always render regardless of
/// whether violations were found.
pub(super) fn render_en301549_annex(
    mut builder: renderreport::engine::ReportBuilder,
    findings: &[NormalizedFinding],
    rule_outcomes: &[crate::wcag::RuleRun],
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = is_english(i18n);

    builder = builder.add_component(
        SectionHeaderSplit::new(
            i18n.t("pdf-sections-en301549-title"),
            i18n.t("pdf-sections-en301549-intro"),
        )
        .with_level(2),
    );

    let disclaimer = if en {
        EN301549_DISCLAIMER_EN
    } else {
        EN301549_DISCLAIMER_DE
    };
    builder = builder.add_component(
        Callout::warning(disclaimer).with_title(i18n.t("batch-report-en301549-disclaimer-title")),
    );

    let rollups = derive_annex(findings);
    let violations: Vec<_> = rollups
        .iter()
        .filter(|r| matches!(r.status, ClauseStatus::ViolationsFound))
        .collect();

    if violations.is_empty() {
        builder = builder.add_component(
            Label::new(i18n.t("pdf-section-clean"))
                .with_size("10.5pt")
                .with_color(crate::output::pdf::design::tokens::NEUTRAL),
        );
    } else {
        let table_title = format!(
            "{} ({})",
            i18n.t("batch-report-en301549-status-violations"),
            violations.len()
        );
        let mut table = AuditTable::new(vec![
            TableColumn::new(i18n.t("batch-report-en301549-col-clause")).with_width("14%"),
            TableColumn::new(i18n.t("batch-report-en301549-col-title")).with_width("34%"),
            TableColumn::new(i18n.t("pdf-sections-en301549-col-level")).with_width("10%"),
            TableColumn::new(i18n.t("diagnosis-col-findings")).with_width("42%"),
        ])
        .with_title(table_title);
        for r in &violations {
            let clause_title = if en {
                r.clause.title_en
            } else {
                r.clause.title_de
            };
            let findings_text = r
                .findings
                .iter()
                .map(|f| format!("{} ({}\u{00d7})", f.rule_id, f.occurrences))
                .collect::<Vec<_>>()
                .join("; ");
            table = table.add_row(vec![
                r.clause.en_clause.to_string(),
                clause_title.to_string(),
                r.clause.wcag_level.to_string(),
                findings_text,
            ]);
        }
        builder = builder.add_component(table);
    }

    let failed_criteria: std::collections::BTreeSet<&str> = rule_outcomes
        .iter()
        .filter(|outcome| crate::wcag::rule_run_unmeasured(outcome))
        .flat_map(|outcome| outcome.wcag.iter().map(String::as_str))
        .collect();
    let incomplete: Vec<_> = rollups
        .iter()
        .filter(|rollup| {
            !matches!(rollup.status, ClauseStatus::ViolationsFound)
                && failed_criteria.contains(rollup.clause.wcag)
        })
        .collect();
    if !incomplete.is_empty() {
        let mut cloud = TagCloud::new()
            .with_title(format!(
                "{} ({})",
                i18n.t("batch-report-en301549-incomplete"),
                incomplete.len()
            ))
            .with_gap("5pt");
        for rollup in &incomplete {
            cloud = cloud.add(
                format!("{} ({})", rollup.clause.en_clause, rollup.clause.wcag),
                "warn",
            );
        }
        builder = builder.add_component(cloud);
    }

    let no_violations: Vec<_> = rollups
        .iter()
        .filter(|r| {
            matches!(r.status, ClauseStatus::NoViolationsAutomated)
                && !failed_criteria.contains(r.clause.wcag)
        })
        .collect();
    if !no_violations.is_empty() {
        let cloud_title = format!(
            "{} ({})",
            i18n.t("pdf-sections-en301549-no-violations"),
            no_violations.len()
        );
        let mut cloud = TagCloud::new().with_title(cloud_title).with_gap("5pt");
        for r in &no_violations {
            cloud = cloud.add(
                format!("{} ({})", r.clause.en_clause, r.clause.wcag),
                "good",
            );
        }
        builder = builder.add_component(cloud);
    }

    let manual: Vec<_> = rollups
        .iter()
        .filter(|r| matches!(r.status, ClauseStatus::ManualReviewRequired))
        .collect();
    if !manual.is_empty() {
        let cloud_title = format!(
            "{} ({})",
            i18n.t("batch-report-en301549-status-manual"),
            manual.len()
        );
        let mut cloud = TagCloud::new().with_title(cloud_title).with_gap("5pt");
        for r in &manual {
            let clause_title = if en {
                r.clause.title_en
            } else {
                r.clause.title_de
            };
            cloud = cloud.add(
                format!(
                    "{} ({}) \u{2013} {}",
                    r.clause.en_clause, r.clause.wcag, clause_title
                ),
                "info",
            );
        }
        builder = builder.add_component(cloud);
    }

    let mut out_of_scope_list =
        List::new().with_title(i18n.t("batch-report-en301549-out-of-scope-title"));
    for chapter in OUT_OF_SCOPE_CHAPTERS {
        let chapter_title = if en {
            chapter.title_en
        } else {
            chapter.title_de
        };
        out_of_scope_list = out_of_scope_list.add_item(i18n.t_args(
            "batch-report-en301549-chapter",
            &[
                ("chapter", chapter.chapter.to_string()),
                ("title", chapter_title.to_string()),
            ],
        ));
    }
    builder = builder.add_component(out_of_scope_list);

    let out_of_standard = out_of_standard_finding_count(findings);
    if out_of_standard > 0 {
        let note = i18n.t_args(
            "pdf-sections-en301549-out-of-standard",
            &[("count", FluentValue::from(out_of_standard as i64))],
        );
        builder =
            builder.add_component(Label::new(&note).with_size("10.5pt").with_color("#475569"));
    }

    builder
}
