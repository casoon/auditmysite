//! Finding renderers for PDF reports.

use renderreport::components::advanced::WrongRightBlock;
use renderreport::components::advanced::{KeyValueList, List};
use renderreport::components::text::{Label, TextBlock};
use renderreport::components::SummaryBox;
use renderreport::prelude::*;

use crate::i18n::I18n;
use crate::output::report_model::*;
use crate::util::truncate_url;

use super::helpers::{effort_label_i18n, priority_label_i18n, role_label_i18n};

pub(super) fn render_key_finding_block(
    mut builder: renderreport::engine::ReportBuilder,
    group: &FindingGroup,
    i18n: &I18n,
    include_technical_context: bool,
) -> renderreport::engine::ReportBuilder {
    let en = i18n.locale() == "en";
    let sev_label = match (group.severity, en) {
        (crate::wcag::Severity::Critical, true) => "CRITICAL",
        (crate::wcag::Severity::Critical, false) => "KRITISCH",
        (crate::wcag::Severity::High, true) => "HIGH",
        (crate::wcag::Severity::High, false) => "HOCH",
        (crate::wcag::Severity::Medium, true) => "MEDIUM",
        (crate::wcag::Severity::Medium, false) => "MITTEL",
        (crate::wcag::Severity::Low, true) => "LOW",
        (crate::wcag::Severity::Low, false) => "GERING",
    };
    let is_quick_win = group.effort == crate::output::report_model::Effort::Quick;
    let title = if is_quick_win {
        format!("{} — {} [Quick Win]", sev_label, group.title)
    } else {
        format!("{} — {}", sev_label, group.title)
    };

    let arc = &group.narrative;
    let en = i18n.locale() == "en";
    let recommendation_label = if en { "Recommendation" } else { "Empfehlung" };

    builder = builder
        .add_component(Label::new(&title).bold().with_size("11pt"))
        .add_component(
            TextBlock::new(first_sentence(&arc.wirkung))
                .with_size("10.5pt")
                .with_line_height("1.4em"),
        )
        .add_component(
            Callout::success(first_sentence(&arc.umsetzung)).with_title(recommendation_label),
        );

    if include_technical_context {
        let tech_context_title = if !group.wcag_criterion.is_empty() {
            format!(
                "{} — WCAG {}",
                i18n.t("finding-tech-context"),
                group.wcag_criterion
            )
        } else {
            i18n.t("finding-tech-context")
        };
        builder = builder.add_component(
            SummaryBox::new(tech_context_title)
                .add_item(i18n.t("finding-tech-rule"), &group.rule_id)
                .add_item("WCAG", &group.wcag_criterion)
                .add_item(
                    i18n.t("finding-tech-instances"),
                    group.occurrence_count.to_string(),
                )
                .add_item(
                    i18n.t("finding-tech-affected-elements"),
                    group.affected_elements.to_string(),
                )
                .add_item(
                    i18n.t("finding-tech-other-occurrences"),
                    group.additional_occurrences.to_string(),
                )
                .add_item(
                    i18n.t("finding-tech-affected-urls"),
                    group.affected_urls.len().to_string(),
                ),
        );
    }
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
            let bytes = text.as_bytes();
            let b0 = bytes[pos - 2];
            let b1 = bytes[pos - 1];
            if b0 == b' ' && b1.is_ascii_alphabetic() {
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
    let header = if !group.wcag_criterion.is_empty() {
        format!(
            "{} — WCAG {} ({})",
            group.title, group.wcag_criterion, group.wcag_level
        )
    } else {
        format!("{} — {}", group.title, group.rule_id)
    };
    builder = builder.add_component(Label::new(&header).bold().with_size("14pt"));

    let meta_kv = KeyValueList::new()
        .add(
            i18n.t("label-priority"),
            priority_label_i18n(group.priority, i18n),
        )
        .add(
            i18n.t("label-owner"),
            role_label_i18n(group.responsible_role, i18n),
        )
        .add(
            i18n.t("label-effort"),
            effort_label_i18n(group.effort, i18n),
        )
        .add(
            i18n.t("finding-elements"),
            group.affected_elements.to_string(),
        )
        .add(
            i18n.t("finding-occurrences"),
            group.occurrence_count.to_string(),
        );
    builder = builder.add_component(meta_kv);

    // AffectedElements: element-type summary + deduplicated selector list
    if !group.representative_occurrences.is_empty() {
        // Count occurrences per element type
        let mut type_counts: std::collections::BTreeMap<&str, usize> =
            std::collections::BTreeMap::new();
        for occ in &group.representative_occurrences {
            let tag = extract_element_type(&occ.selector);
            if !tag.is_empty() {
                *type_counts.entry(tag).or_insert(0) += 1;
            }
        }
        if !type_counts.is_empty() {
            let summary = type_counts
                .iter()
                .map(|(tag, count)| format!("{}× {}", count, tag))
                .collect::<Vec<_>>()
                .join("  ·  ");
            builder = builder
                .add_component(KeyValueList::new().add(i18n.t("finding-element-types"), summary));
        }
    }

    if !group.recommendation.is_empty() {
        builder = builder.add_component(
            Callout::success(&group.recommendation).with_title(i18n.t("finding-recommendation")),
        );
    }

    for example in &group.examples {
        builder = builder.add_component(
            WrongRightBlock::new(&example.bad, &example.good)
                .code()
                .with_labels(i18n.t("finding-wrong"), i18n.t("finding-right")),
        );
        if let Some(ref dec) = example.decorative {
            builder =
                builder.add_component(Callout::info(dec).with_title(i18n.t("label-decorative")));
        }
    }

    if !group.affected_urls.is_empty() && group.affected_urls.len() <= 10 {
        let mut url_list = List::new().with_title(i18n.t("label-affected-urls"));
        for url in &group.affected_urls {
            url_list = url_list.add_item(truncate_url(url, 70));
        }
        builder = builder.add_component(url_list);
    }

    if !group.representative_occurrences.is_empty() {
        let mut table = renderreport::components::AuditTable::new(vec![
            renderreport::components::TableColumn::new(i18n.t("finding-location"))
                .with_width("26%"),
            renderreport::components::TableColumn::new(i18n.t("finding-note")).with_width("74%"),
        ])
        .with_title(i18n.t("finding-representative-occurrences"));

        for occ in &group.representative_occurrences {
            table = table.add_row(vec![
                truncate_url(&occ.selector, 48),
                first_sentence(&occ.message).to_string(),
            ]);
        }
        builder = builder.add_component(table);

        let en = i18n.locale() == "en";
        let hidden = group.representative_occurrences.len().saturating_sub(3);
        for occ in group.representative_occurrences.iter().take(3) {
            let mut snapshot = SummaryBox::new(format!(
                "{}: {}",
                i18n.t("finding-occurrence"),
                truncate_url(&occ.selector, 60)
            ))
            .add_item("Node", &occ.node_id)
            .add_item(i18n.t("finding-note"), first_sentence(&occ.message));

            if let Some(html) = occ
                .html_snippet
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
            {
                snapshot = snapshot.add_item("HTML", truncate_url(html, 110));
            }

            builder = builder.add_component(snapshot);

            if let Some(code) = occ
                .suggested_code
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
            {
                builder = builder.add_component(
                    Callout::info(truncate_url(code, 180))
                        .with_title(i18n.t("finding-suggested-fix")),
                );
            }
        }
        if hidden > 0 {
            let msg = if en {
                format!("{hidden} more occurrence(s) documented in the technical appendix.")
            } else {
                format!("{hidden} weitere(s) Vorkommen im technischen Anhang dokumentiert.")
            };
            builder = builder.add_component(Callout::info(&msg));
        }
    }

    if !group.pattern_clusters.is_empty() {
        let mut table = renderreport::components::AuditTable::new(vec![
            renderreport::components::TableColumn::new(i18n.t("finding-pattern")).with_width("70%"),
            renderreport::components::TableColumn::new(i18n.t("finding-occurrences"))
                .with_width("30%"),
        ])
        .with_title(i18n.t("finding-frequent-patterns"));

        for cluster in &group.pattern_clusters {
            table = table.add_row(vec![
                truncate_url(&cluster.label, 72),
                cluster.occurrences.to_string(),
            ]);
        }
        builder = builder.add_component(table);
    }

    builder
}

/// Extract the HTML element type from a CSS selector path.
/// Examples: "div.main > img" → "img", "a#skip-link" → "a", "button" → "button"
fn extract_element_type(selector: &str) -> &str {
    let last_segment = selector.split('>').next_back().unwrap_or(selector).trim();
    let last_token = last_segment
        .split_whitespace()
        .last()
        .unwrap_or(last_segment);
    let end = last_token
        .find(|c: char| !c.is_ascii_alphanumeric() && c != '-')
        .unwrap_or(last_token.len());
    &last_token[..end]
}
