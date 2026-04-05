//! Finding renderers for PDF reports.

use renderreport::components::advanced::WrongRightBlock;
use renderreport::components::advanced::{KeyValueList, List};
use renderreport::components::text::{Label, TextBlock};
use renderreport::prelude::*;

use crate::i18n::I18n;
use crate::output::report_model::*;
use crate::util::truncate_url;

use super::helpers::{effort_label_i18n, priority_label_i18n, role_label_i18n};

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
/// Skips common German abbreviations like "z. B.", "d. h.", "u. a.".
pub(super) fn first_sentence(text: &str) -> &str {
    let mut search_from = 0;
    while let Some(rel) = text[search_from..].find(". ") {
        let pos = search_from + rel;
        // Check for single-letter abbreviation pattern: " X. " (e.g. "z. B.", "d. h.")
        if pos >= 2 {
            let before = &text[pos - 2..pos];
            if before.starts_with(' ') && before.as_bytes()[1].is_ascii_alphabetic() {
                search_from = pos + 2;
                continue;
            }
        }
        return &text[..pos + 1];
    }
    text
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
    builder =
        builder.add_component(Callout::success(&group.recommendation).with_title("Empfehlung"));

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
