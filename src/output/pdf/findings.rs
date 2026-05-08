//! Finding renderers for PDF reports.

use renderreport::components::advanced::WrongRightBlock;
use renderreport::components::advanced::{KeyValueList, List};
use renderreport::components::text::Label;
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
    let mut kv = KeyValueList::new().with_title(title);

    let (key_problem, key_impact, key_cause, key_fix, key_effort, qw_value) = if en {
        (
            "Problem",
            "What users experience",
            "Cause",
            "What to do",
            "Effort",
            "Quick win — a few hours, high impact",
        )
    } else {
        (
            "Problem",
            "Was Nutzer erleben",
            "Ursache",
            "Was tun",
            "Aufwand",
            "Quick Win — wenige Stunden, hohe Wirkung",
        )
    };

    let problem = first_sentence(&group.customer_description);
    kv = kv.add(key_problem, problem);

    if !group.user_impact.is_empty() {
        kv = kv.add(key_impact, first_sentence(&group.user_impact));
    }

    if !group.typical_cause.is_empty() {
        kv = kv.add(key_cause, first_sentence(&group.typical_cause));
    }

    kv = kv.add(key_fix, first_sentence(&group.recommendation));

    if is_quick_win {
        kv = kv.add(key_effort, qw_value);
    }

    builder = builder.add_component(kv);
    if include_technical_context {
        let (te_title, te_rule, te_wcag, te_instances, te_affected, te_more, te_urls) = if en {
            (
                "Technical context",
                "Rule",
                "WCAG",
                "Instances",
                "Affected elements",
                "Other similar occurrences",
                "Affected URLs",
            )
        } else {
            (
                "Technische Einordnung",
                "Regel",
                "WCAG",
                "Instanzen",
                "Betroffene Elemente",
                "Weitere ähnliche Vorkommen",
                "Betroffene URLs",
            )
        };
        builder = builder.add_component(
            SummaryBox::new(te_title)
                .add_item(te_rule, &group.rule_id)
                .add_item(te_wcag, &group.wcag_criterion)
                .add_item(te_instances, group.occurrence_count.to_string())
                .add_item(te_affected, group.affected_elements.to_string())
                .add_item(te_more, group.additional_occurrences.to_string())
                .add_item(te_urls, group.affected_urls.len().to_string()),
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
    let en = i18n.locale() == "en";
    let header = if !group.wcag_criterion.is_empty() {
        format!(
            "{} — WCAG {} ({})",
            group.title, group.wcag_criterion, group.wcag_level
        )
    } else {
        format!("{} — {}", group.title, group.rule_id)
    };
    builder = builder.add_component(Label::new(&header).bold().with_size("14pt"));

    let (elements_label, occurrences_label) = if en {
        ("Elements", "Occurrences")
    } else {
        ("Elemente", "Vorkommen")
    };
    let meta_kv = KeyValueList::new()
        .add(i18n.t("label-priority"), priority_label_i18n(group.priority, i18n))
        .add(i18n.t("label-owner"), role_label_i18n(group.responsible_role, i18n))
        .add(i18n.t("label-effort"), effort_label_i18n(group.effort, i18n))
        .add(elements_label, group.affected_elements.to_string())
        .add(occurrences_label, group.occurrence_count.to_string());
    builder = builder.add_component(meta_kv);

    // AffectedElements: compact selector list grouped by element type
    if !group.representative_occurrences.is_empty() {
        let selectors_title = if en { "Affected selectors" } else { "Betroffene Selektoren" };
        let mut sel_list = List::new().with_title(selectors_title);
        let mut seen = std::collections::HashSet::new();
        for occ in &group.representative_occurrences {
            if seen.insert(occ.selector.as_str()) {
                sel_list = sel_list.add_item(truncate_url(&occ.selector, 70));
            }
        }
        builder = builder.add_component(sel_list);
    }

    let recommendation_title = if en { "Recommendation" } else { "Empfehlung" };
    builder = builder
        .add_component(Callout::success(&group.recommendation).with_title(recommendation_title));

    let (wrong_label, right_label) = if en {
        ("✕ Wrong", "✓ Correct")
    } else {
        ("✕ Falsch", "✓ Richtig")
    };
    for example in &group.examples {
        builder = builder.add_component(
            WrongRightBlock::new(&example.bad, &example.good)
                .code()
                .with_labels(wrong_label, right_label),
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
        let (loc_col, hint_col, repr_title, occ_title, html_label, code_callout_title) = if en {
            (
                "Location",
                "Note",
                "Representative occurrences",
                "Occurrence",
                "HTML",
                "Suggested code fix",
            )
        } else {
            (
                "Fundstelle",
                "Hinweis",
                "Repräsentative Fundstellen",
                "Fundstelle",
                "HTML",
                "Vorgeschlagene Code-Korrektur",
            )
        };
        let hint_label = hint_col;
        let mut table = renderreport::components::AuditTable::new(vec![
            renderreport::components::TableColumn::new(loc_col).with_width("26%"),
            renderreport::components::TableColumn::new(hint_col).with_width("74%"),
        ])
        .with_title(repr_title);

        for occ in &group.representative_occurrences {
            table = table.add_row(vec![
                truncate_url(&occ.selector, 48),
                first_sentence(&occ.message).to_string(),
            ]);
        }
        builder = builder.add_component(table);

        for occ in &group.representative_occurrences {
            let mut snapshot = SummaryBox::new(format!(
                "{}: {}",
                occ_title,
                truncate_url(&occ.selector, 60)
            ))
            .add_item("Node", &occ.node_id)
            .add_item(hint_label, first_sentence(&occ.message));

            if let Some(html) = occ
                .html_snippet
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
            {
                snapshot = snapshot.add_item(html_label, truncate_url(html, 110));
            }

            builder = builder.add_component(snapshot);

            if let Some(code) = occ
                .suggested_code
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
            {
                builder = builder.add_component(
                    Callout::info(truncate_url(code, 180)).with_title(code_callout_title),
                );
            }
        }
    }

    if !group.pattern_clusters.is_empty() {
        let (pattern_col, occ_col, table_title) = if en {
            ("Pattern", "Occurrences", "Frequent patterns")
        } else {
            ("Muster", "Vorkommen", "Häufige Muster")
        };
        let mut table = renderreport::components::AuditTable::new(vec![
            renderreport::components::TableColumn::new(pattern_col).with_width("70%"),
            renderreport::components::TableColumn::new(occ_col).with_width("30%"),
        ])
        .with_title(table_title);

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
