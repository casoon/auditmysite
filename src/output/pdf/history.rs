//! History/trend and methodology section renderers.

use renderreport::components::advanced::{KeyValueList, List, PageBreak};
use renderreport::components::text::{Label, TextBlock};
use renderreport::components::{AuditTable, TableColumn};
use renderreport::prelude::*;

use crate::i18n::I18n;
use crate::output::report_model::*;

use super::helpers::{component_json, soft_flow_group};

#[allow(dead_code)]
pub(super) fn render_history_section(
    mut builder: renderreport::engine::ReportBuilder,
    history: &HistoryTrendBlock,
) -> renderreport::engine::ReportBuilder {
    let mut kv = KeyValueList::new().with_title("Trend zum letzten Lauf");
    for (key, value) in &history.metrics {
        kv = kv.add(key, value);
    }

    let trend_color = match history.trend_label.as_str() {
        "Deutlich verbessert" | "Verbessert" => "#22c55e",
        "Stabil" => "#2563eb",
        _ => "#ef4444",
    };

    builder = builder.add_component(soft_flow_group(
        "240pt",
        vec![
            component_json(Section::new("Historie und Trend").with_level(1)),
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
        let mut table = AuditTable::new(vec![
            TableColumn::new("Datum"),
            TableColumn::new("Accessibility"),
            TableColumn::new("Gesamt"),
            TableColumn::new("Note"),
            TableColumn::new("Issues"),
        ])
        .with_title("Verlauf der letzten Läufe");

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
        let mut list = List::new().with_title("Neue Themen seit dem letzten Lauf");
        for finding in &history.new_findings {
            list = list.add_item(finding);
        }
        builder = builder.add_component(list);
    }

    if !history.resolved_findings.is_empty() {
        let mut list = List::new().with_title("Behobene Themen seit dem letzten Lauf");
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
    let mut table = AuditTable::new(vec![TableColumn::new("Aspekt"), TableColumn::new("Wert")])
        .with_title("Audit-Kontext");

    for (key, value) in &methodology.audit_facts {
        table = table.add_row(vec![key.clone(), value.clone()]);
    }

    let mut confidence_table = AuditTable::new(vec![
        TableColumn::new("Signal"),
        TableColumn::new("Einordnung"),
    ])
    .with_title("Vertrauen & Einordnung");
    for (key, value) in &methodology.confidence_summary {
        confidence_table = confidence_table.add_row(vec![key.clone(), value.clone()]);
    }

    let mut capability_table = AuditTable::new(vec![
        TableColumn::new("Signal").with_width("24%"),
        TableColumn::new("Quelle").with_width("18%"),
        TableColumn::new("Vertrauen").with_width("14%"),
        TableColumn::new("Outputs").with_width("18%"),
        TableColumn::new("Hinweis").with_width("26%"),
    ])
    .with_title("Capabilities & Coverage");
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
        .add_component(PageBreak::new())
        .add_component(Section::new("Methodik & Einschränkungen").with_level(1))
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
