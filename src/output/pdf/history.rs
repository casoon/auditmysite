//! History/trend and methodology section renderers.

use renderreport::components::advanced::{KeyValueList, List};
use renderreport::components::text::{Label, TextBlock};
use renderreport::components::{AuditTable, TableColumn};
use renderreport::prelude::*;

use crate::i18n::I18n;
use crate::output::report_model::*;

use super::helpers::{component_json, soft_flow_group};

pub(super) fn render_history_section(
    mut builder: renderreport::engine::ReportBuilder,
    history: &HistoryTrendBlock,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = i18n.locale() == "en";
    let trend_box_title = if en {
        "Trend vs. last run"
    } else {
        "Trend zum letzten Lauf"
    };
    let mut kv = KeyValueList::new().with_title(trend_box_title);
    for (key, value) in &history.metrics {
        kv = kv.add(key, value);
    }

    let trend_color = match history.trend_label.as_str() {
        "Deutlich verbessert" | "Verbessert" | "Significantly improved" | "Improved" => "#22c55e",
        "Stabil" | "Stable" => "#2563eb",
        _ => "#ef4444",
    };

    let history_section_title = if en {
        "History and trend"
    } else {
        "Historie und Trend"
    };
    builder = builder.add_component(soft_flow_group(
        "240pt",
        vec![
            component_json(Section::new(history_section_title).with_level(1)),
            component_json(
                Label::new(&history.trend_label)
                    .with_size("13pt")
                    .bold()
                    .with_color(trend_color),
            ),
            component_json(TextBlock::new(&history.summary)),
            component_json(kv),
        ],
    ));

    if !history.timeline_rows.is_empty() {
        let (col_date, col_a11y, col_overall, col_grade, col_issues, table_title) = if en {
            (
                "Date",
                "Accessibility",
                "Overall",
                "Grade",
                "Issues",
                "Recent runs",
            )
        } else {
            (
                "Datum",
                "Accessibility",
                "Gesamt",
                "Note",
                "Issues",
                "Verlauf der letzten Läufe",
            )
        };
        let mut table = AuditTable::new(vec![
            TableColumn::new(col_date),
            TableColumn::new(col_a11y),
            TableColumn::new(col_overall),
            TableColumn::new(col_grade),
            TableColumn::new(col_issues),
        ])
        .with_title(table_title);

        for row in &history.timeline_rows {
            table = table.add_row(vec![
                row.0.clone(),
                row.1.clone(),
                row.2.clone(),
                row.3.clone(),
                row.4.clone(),
            ]);
        }
        builder = builder.add_component(table);
    }

    if !history.new_findings.is_empty() {
        let title = if en {
            "New topics since the last run"
        } else {
            "Neue Themen seit dem letzten Lauf"
        };
        let mut list = List::new().with_title(title);
        for finding in &history.new_findings {
            list = list.add_item(finding);
        }
        builder = builder.add_component(list);
    }

    if !history.resolved_findings.is_empty() {
        let title = if en {
            "Resolved topics since the last run"
        } else {
            "Behobene Themen seit dem letzten Lauf"
        };
        let mut list = List::new().with_title(title);
        for finding in &history.resolved_findings {
            list = list.add_item(finding);
        }
        builder = builder.add_component(list);
    }

    builder
}

pub(super) fn render_methodology_section(
    mut builder: renderreport::engine::ReportBuilder,
    methodology: &MethodologyBlock,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = i18n.locale() == "en";
    let (
        aspect_col,
        value_col,
        ctx_title,
        confidence_signal_col,
        confidence_eval_col,
        confidence_title,
        cap_signal,
        cap_source,
        cap_confidence,
        cap_outputs,
        cap_note,
        cap_title,
        method_section_title,
    ) = if en {
        (
            "Aspect",
            "Value",
            "Audit context",
            "Signal",
            "Classification",
            "Confidence & classification",
            "Signal",
            "Source",
            "Confidence",
            "Outputs",
            "Note",
            "Capabilities & coverage",
            "Methodology & limitations",
        )
    } else {
        (
            "Aspekt",
            "Wert",
            "Audit-Kontext",
            "Signal",
            "Einordnung",
            "Vertrauen & Einordnung",
            "Signal",
            "Quelle",
            "Vertrauen",
            "Outputs",
            "Hinweis",
            "Capabilities & Coverage",
            "Methodik & Einschränkungen",
        )
    };

    let mut table = AuditTable::new(vec![
        TableColumn::new(aspect_col),
        TableColumn::new(value_col),
    ])
    .with_title(ctx_title);

    for (key, value) in &methodology.audit_facts {
        table = table.add_row(vec![key.clone(), value.clone()]);
    }

    let mut confidence_table = AuditTable::new(vec![
        TableColumn::new(confidence_signal_col),
        TableColumn::new(confidence_eval_col),
    ])
    .with_title(confidence_title);
    for (key, value) in &methodology.confidence_summary {
        confidence_table = confidence_table.add_row(vec![key.clone(), value.clone()]);
    }

    let mut capability_table = AuditTable::new(vec![
        TableColumn::new(cap_signal).with_width("24%"),
        TableColumn::new(cap_source).with_width("18%"),
        TableColumn::new(cap_confidence).with_width("14%"),
        TableColumn::new(cap_outputs).with_width("18%"),
        TableColumn::new(cap_note).with_width("26%"),
    ])
    .with_title(cap_title);
    for cap in &methodology.capabilities {
        capability_table = capability_table.add_row(vec![
            cap.signal.clone(),
            cap.source.clone(),
            cap.confidence.clone(),
            cap.surfaces.join(", "),
            cap.note.clone(),
        ]);
    }

    builder = builder
        .add_component(Section::new(method_section_title).with_level(1))
        .add_component(TextBlock::new(&methodology.scope))
        .add_component(TextBlock::new(&methodology.method))
        .add_component(table)
        .add_component(confidence_table)
        .add_component(capability_table)
        .add_component(
            Callout::info(&methodology.limitations).with_title(i18n.t("callout-limitations-title")),
        )
        .add_component(
            Callout::warning(&methodology.disclaimer).with_title(i18n.t("callout-note-title")),
        )
        .add_component(TextBlock::new(i18n.t("certificate-thresholds")));

    builder
}
