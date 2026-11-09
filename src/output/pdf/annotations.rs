//! PDF rendering of `--annotations` (#961).
//!
//! An assessment is shown below the finding it belongs to, as its own
//! labelled callout and never inside the finding's text. The "Assessments"
//! section summarises counts per author kind and lists what matched nothing.
//! All wording comes from the `annotations-*` keys of the locale files.

use renderreport::components::advanced::PageBreak;
use renderreport::components::text::Label;
use renderreport::components::{AuditTable, TableColumn};
use renderreport::engine::ReportBuilder;
use renderreport::prelude::*;

use crate::annotations::{
    Assessment, AuthorKind, EvidenceKind, Matched, Resolved, Unmatched, UnmatchedReason,
};
use crate::i18n::I18n;

fn author_label(kind: AuthorKind, i18n: &I18n) -> String {
    i18n.t(match kind {
        AuthorKind::Agent => "annotations-label-agent",
        AuthorKind::Human => "annotations-label-human",
    })
}

fn assessment_text(assessment: Assessment, i18n: &I18n) -> String {
    i18n.t(match assessment {
        Assessment::LikelyIssue => "annotations-value-likely-issue",
        Assessment::LikelyFine => "annotations-value-likely-fine",
        Assessment::Undetermined => "annotations-value-undetermined",
    })
}

fn reason_text(reason: UnmatchedReason, i18n: &I18n) -> String {
    i18n.t(match reason {
        UnmatchedReason::FindingNotFound => "annotations-reason-finding-not-found",
        UnmatchedReason::PatternNotFound => "annotations-reason-pattern-not-found",
        UnmatchedReason::UrlNotAudited => "annotations-reason-url-not-audited",
        UnmatchedReason::UrlRequired => "annotations-reason-url-required",
        UnmatchedReason::NoReadingFlow => "annotations-reason-no-reading-flow",
    })
}

fn target_text(annotation: &crate::annotations::Annotation, i18n: &I18n) -> String {
    match annotation.scope {
        crate::annotations::Scope::ReadingFlow => i18n.t("annotations-scope-reading-flow"),
        crate::annotations::Scope::Finding => {
            let mut text = annotation.finding_id.clone().unwrap_or_default();
            if let Some(pattern) = &annotation.pattern_id {
                text.push_str(" / ");
                text.push_str(pattern);
            }
            text
        }
    }
}

/// One assessment as a callout: label and value in the title, the reasoning
/// and the evidence in the body.
fn callout(m: &Matched, i18n: &I18n) -> Callout {
    let a = &m.annotation;
    let title = format!(
        "{}: {}",
        author_label(a.author.kind, i18n),
        assessment_text(a.assessment, i18n)
    );
    let mut body = a.reasoning.trim().to_string();
    if let Some(selector) = &m.pattern_selector {
        body.push_str(&format!(
            "\n{}: {}",
            i18n.t("annotations-pattern"),
            selector
        ));
    }
    for evidence in &a.evidence {
        let value = match evidence.kind {
            EvidenceKind::Screenshot => evidence.path.as_deref(),
            EvidenceKind::Link => evidence.url.as_deref(),
            EvidenceKind::Text => evidence.text.as_deref(),
        };
        if let Some(value) = value {
            body.push_str(&format!("\n{}: {}", i18n.t("annotations-evidence"), value));
        }
    }
    let by = match &a.created {
        Some(date) => format!("{}, {}", a.author.name, date),
        None => a.author.name.clone(),
    };
    body.push_str(&format!("\n— {by}"));
    match a.assessment {
        Assessment::LikelyIssue => Callout::warning(body),
        _ => Callout::info(body),
    }
    .with_title(title)
}

/// The assessments of one finding, below its detail card.
pub(super) fn render_finding_annotations(
    mut builder: ReportBuilder,
    resolved: Option<&Resolved>,
    finding_id: &str,
    i18n: &I18n,
) -> ReportBuilder {
    let Some(resolved) = resolved else {
        return builder;
    };
    if finding_id.is_empty() {
        return builder;
    }
    for m in resolved.for_finding(finding_id) {
        builder = builder.add_component(callout(m, i18n));
    }
    builder
}

/// The assessments of the reading flow, below the screen-reader section.
pub(super) fn render_reading_flow_annotations(
    mut builder: ReportBuilder,
    resolved: Option<&Resolved>,
    i18n: &I18n,
) -> ReportBuilder {
    let Some(resolved) = resolved else {
        return builder;
    };
    for m in resolved.reading_flow() {
        builder = builder.add_component(callout(m, i18n));
    }
    builder
}

fn unmatched_row(u: &Unmatched, i18n: &I18n) -> Vec<String> {
    vec![
        target_text(&u.annotation, i18n),
        u.annotation.url.clone().unwrap_or_default(),
        author_label(u.annotation.author.kind, i18n),
        reason_text(u.reason, i18n),
    ]
}

/// The "Assessments" section: counts per author kind, the unmatched ones and,
/// in a batch report (where findings are aggregated across pages), the
/// matched ones per page.
pub(super) fn render_assessments_section(
    mut builder: ReportBuilder,
    resolved: &Resolved,
    list_matched: bool,
    i18n: &I18n,
) -> ReportBuilder {
    let counts = resolved.counts();
    builder = builder
        .add_component(PageBreak::new())
        .add_component(Section::new(i18n.t("annotations-section-title")).with_level(2))
        .add_component(
            Label::new(i18n.t("annotations-section-intro"))
                .with_size("10.5pt")
                .with_color(crate::output::pdf::design::tokens::NEUTRAL),
        )
        .add_component(Label::new(i18n.t_args(
            "annotations-counts",
            &[
                ("total", counts.total as i64),
                ("agent", counts.by_author_kind.agent as i64),
                ("human", counts.by_author_kind.human as i64),
                ("matched", counts.matched as i64),
                ("unmatched", counts.unmatched as i64),
            ],
        )));

    if list_matched && !resolved.matched.is_empty() {
        let mut table = AuditTable::new(vec![
            TableColumn::new(i18n.t("annotations-col-page")).with_width("24%"),
            TableColumn::new(i18n.t("annotations-col-target")).with_width("18%"),
            TableColumn::new(i18n.t("annotations-col-author")).with_width("16%"),
            TableColumn::new(i18n.t("annotations-col-assessment")).with_width("16%"),
            TableColumn::new(i18n.t("annotations-col-reasoning")).with_width("26%"),
        ])
        .with_title(i18n.t("annotations-matched-title"));
        for m in &resolved.matched {
            table = table.add_row(vec![
                m.url.clone(),
                target_text(&m.annotation, i18n),
                author_label(m.annotation.author.kind, i18n),
                assessment_text(m.annotation.assessment, i18n),
                m.annotation.reasoning.trim().to_string(),
            ]);
        }
        builder = builder.add_component(table);
    }

    if !resolved.unmatched.is_empty() {
        builder = builder.add_component(Label::new(i18n.t("annotations-unmatched-intro")));
        let mut table = AuditTable::new(vec![
            TableColumn::new(i18n.t("annotations-col-target")).with_width("22%"),
            TableColumn::new(i18n.t("annotations-col-page")).with_width("22%"),
            TableColumn::new(i18n.t("annotations-col-author")).with_width("26%"),
            TableColumn::new(i18n.t("annotations-col-reason")).with_width("30%"),
        ])
        .with_title(i18n.t("annotations-unmatched-title"));
        for u in &resolved.unmatched {
            table = table.add_row(unmatched_row(u, i18n));
        }
        builder = builder.add_component(table);
    }

    if !resolved.warnings.is_empty() {
        builder = builder.add_component(Label::new(i18n.t("annotations-warnings-title")));
        for warning in &resolved.warnings {
            builder = builder.add_component(Label::new(warning.clone()));
        }
    }
    builder
}
