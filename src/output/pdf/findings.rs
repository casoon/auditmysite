//! Finding renderers for PDF reports.

use renderreport::components::advanced::WrongRightBlock;
use renderreport::components::advanced::{KeyValueList, List};
use renderreport::components::text::{Label, TextBlock};
use renderreport::components::Finding;
use renderreport::prelude::*;

use crate::i18n::I18n;
use crate::output::report_model::*;
use crate::util::truncate_url;

use super::helpers::{effort_label_i18n, map_severity, priority_label_i18n, role_label_i18n};

pub(super) fn render_key_finding_block(
    mut builder: renderreport::engine::ReportBuilder,
    group: &FindingGroup,
    _i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    // Compact finding card: 4 lines only — Problem / Impact / Ursache / Fix
    let sev_label = match group.severity {
        crate::wcag::Severity::Critical => "KRITISCH",
        crate::wcag::Severity::High => "HOCH",
        crate::wcag::Severity::Medium => "MITTEL",
        crate::wcag::Severity::Low => "GERING",
    };
    let mut kv = KeyValueList::new().with_title(format!("{} — {}", sev_label, group.title));

    // Problem — one sentence from customer_description
    let problem = first_sentence(&group.customer_description);
    kv = kv.add("Problem", problem);

    // Impact — one sentence
    if !group.user_impact.is_empty() {
        kv = kv.add("Impact", first_sentence(&group.user_impact));
    }

    // Ursache — one sentence
    if !group.typical_cause.is_empty() {
        kv = kv.add("Ursache", first_sentence(&group.typical_cause));
    }

    // Fix — one sentence
    kv = kv.add("Fix", first_sentence(&group.recommendation));

    builder = builder.add_component(kv);
    builder
}

/// Extract the first sentence from a text (up to first period + space, or full text).
fn first_sentence(text: &str) -> &str {
    if let Some(pos) = text.find(". ") {
        &text[..pos + 1]
    } else {
        text
    }
}

pub(super) fn render_finding_technical(
    mut builder: renderreport::engine::ReportBuilder,
    group: &FindingGroup,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    // Header: title + WCAG reference
    let header = if !group.wcag_criterion.is_empty() {
        format!(
            "{} — WCAG {} ({})",
            group.title, group.wcag_criterion, group.wcag_level
        )
    } else {
        format!("{} — {}", group.title, group.rule_id)
    };
    builder = builder.add_component(Label::new(&header).bold().with_size("14pt"));

    // Compact meta: priority | owner | effort | elements
    let meta = format!(
        "{}: {} | {}: {} | {}: {} | {} Elemente, {} Vorkommen",
        i18n.t("label-priority"),
        priority_label_i18n(group.priority, i18n),
        i18n.t("label-owner"),
        role_label_i18n(group.responsible_role, i18n),
        i18n.t("label-effort"),
        effort_label_i18n(group.effort, i18n),
        group.affected_elements,
        group.occurrence_count,
    );
    builder = builder.add_component(TextBlock::new(meta));

    // Recommendation only — no repeated problem description
    builder = builder.add_component(
        Callout::success(&group.recommendation).with_title("Empfehlung"),
    );

    // Code examples — the core of the tech section
    for example in &group.examples {
        builder = builder.add_component(
            WrongRightBlock::new(&example.bad, &example.good)
                .code()
                .with_labels("✕ Falsch", "✓ Richtig"),
        );
        if let Some(ref dec) = example.decorative {
            builder =
                builder.add_component(Callout::info(dec).with_title(i18n.t("label-decorative")));
        }
    }

    // Affected URLs (compact)
    if !group.affected_urls.is_empty() && group.affected_urls.len() <= 10 {
        let mut url_list = List::new().with_title(i18n.t("label-affected-urls"));
        for url in &group.affected_urls {
            url_list = url_list.add_item(truncate_url(url, 70));
        }
        builder = builder.add_component(url_list);
    }

    builder
}

pub(super) fn render_finding_group(
    mut builder: renderreport::engine::ReportBuilder,
    group: &FindingGroup,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let section_title = if !group.wcag_criterion.is_empty() {
        format!("{} (WCAG {})", group.title, group.wcag_criterion)
    } else if let Some(ref dim) = group.dimension {
        format!("{} ({})", group.title, dim)
    } else {
        group.title.clone()
    };
    builder = builder.add_component(Section::new(&section_title).with_level(2));

    if matches!(
        group.severity,
        crate::wcag::Severity::Critical | crate::wcag::Severity::High
    ) {
        builder = builder.add_component(Callout::error(&group.customer_description).with_title(
            format!(
                "{} — {}: {}",
                group.title,
                i18n.t("label-priority"),
                priority_label_i18n(group.priority, i18n)
            ),
        ));
    }

    let mut finding = Finding::new(
        &group.title,
        map_severity(&group.severity),
        &group.customer_description,
    )
    .with_recommendation(&group.recommendation)
    .with_category(format!(
        "{}: {} | {}: {} | {}: {}",
        i18n.t("label-priority"),
        priority_label_i18n(group.priority, i18n),
        i18n.t("label-owner"),
        role_label_i18n(group.responsible_role, i18n),
        i18n.t("label-effort"),
        effort_label_i18n(group.effort, i18n)
    ));

    if group.occurrence_count > 0 {
        finding = finding.with_affected(format!(
            "{} Vorkommen, {} Elemente betroffen{}",
            group.occurrence_count,
            group.affected_elements,
            if group.affected_urls.is_empty() {
                String::new()
            } else {
                format!(", {} URLs", group.affected_urls.len())
            }
        ));
    }

    builder = builder.add_component(finding);

    if !group.user_impact.is_empty() {
        builder = builder.add_component(
            Callout::info(&group.user_impact).with_title(i18n.t("label-user-impact")),
        );
    }
    if !group.typical_cause.is_empty() {
        builder = builder.add_component(TextBlock::new(format!(
            "{}: {}",
            i18n.t("label-typical-cause"),
            &group.typical_cause
        )));
    }
    if !group.technical_note.is_empty() {
        builder = builder.add_component(TextBlock::new(format!(
            "{}: {}",
            i18n.t("label-tech-note"),
            &group.technical_note
        )));
    }

    if !group.examples.is_empty() {
        builder = builder.add_component(Section::new(i18n.t("label-code-example")).with_level(3));
        for example in &group.examples {
            builder = builder.add_component(
                WrongRightBlock::new(&example.bad, &example.good)
                    .code()
                    .with_labels("✕ Falsch", "✓ Richtig"),
            );
            if let Some(ref dec) = example.decorative {
                builder = builder
                    .add_component(Callout::info(dec).with_title(i18n.t("label-decorative")));
            }
        }
    }

    if !group.affected_urls.is_empty() && group.affected_urls.len() <= 10 {
        let mut url_list = List::new().with_title(i18n.t("label-affected-urls"));
        for url in &group.affected_urls {
            url_list = url_list.add_item(truncate_url(url, 70));
        }
        builder = builder.add_component(url_list);
    } else if group.affected_urls.len() > 10 {
        builder = builder.add_component(TextBlock::new(format!(
            "Betrifft {} URLs (zu viele für Einzelauflistung — siehe Anhang).",
            group.affected_urls.len()
        )));
    }

    builder
}
