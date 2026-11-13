//! Diagnosis section and scope helpers for PDF reports.

use renderreport::components::text::Label;
use renderreport::components::{AuditTable, TableColumn};
use renderreport::prelude::*;

use crate::i18n::I18n;
use crate::output::report_model::*;

#[cfg(test)]
mod text_export;
#[cfg(test)]
mod text_guard;

pub(super) fn render_diagnosis_section(
    mut builder: renderreport::engine::ReportBuilder,
    diagnosis: &DiagnosisBlock,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    builder = builder.add_component(Section::new(&diagnosis.section_title).with_level(2));

    // Pattern overview — render as clean Label
    let pattern_intro = diagnosis.pattern_description.clone();
    let pattern_text = format!("{}: {}", diagnosis.pattern_label, pattern_intro);
    builder = builder.add_component(
        Label::new(&pattern_text)
            .with_size("10.5pt")
            .with_color("#475569"),
    );

    // Dominant issue spotlight
    if let Some(ref dominant) = diagnosis.dominant_issue {
        let spotlight_title = i18n.t("diagnosis-pattern-single-dominant");
        let spotlight_body = i18n.t_args(
            "diagnosis-dominant-spotlight",
            &[("issue", dominant.as_str())],
        );
        let spotlight_text = format!("{}: {}", spotlight_title, spotlight_body);
        builder = builder.add_component(
            Label::new(&spotlight_text)
                .with_size("10.5pt")
                .bold()
                .with_color(crate::output::pdf::design::tokens::DANGER),
        );
    }

    // Category breakdown table
    if !diagnosis.category_breakdown.is_empty() {
        let col_dim = i18n.t("diagnosis-col-category");
        let col_count = i18n.t("diagnosis-col-occurrences");
        let col_sev = i18n.t("diagnosis-col-worst-severity");
        let table_title = i18n.t("diagnosis-table-categories");
        let mut table = AuditTable::new(vec![
            TableColumn::new(col_dim),
            TableColumn::new(col_count).with_width("15%"),
            TableColumn::new(col_sev).with_width("25%"),
        ])
        .with_title(table_title);
        for (dim, count, sev_label) in &diagnosis.category_breakdown {
            table = table.add_row(vec![dim.clone(), count.to_string(), sev_label.clone()]);
        }
        builder = builder.add_component(table);
    }

    // Thematic clusters
    if !diagnosis.clusters.is_empty() {
        let clusters_title = i18n.t("diagnosis-table-clusters");
        let col_cluster = "Cluster";
        let col_findings = i18n.t("diagnosis-col-findings");
        let col_occ = i18n.t("diagnosis-col-occurrences");
        let col_sev = i18n.t("diagnosis-col-max-severity");
        let mut table = AuditTable::new(vec![
            TableColumn::new(col_cluster),
            TableColumn::new(col_findings).with_width("12%"),
            TableColumn::new(col_occ).with_width("14%"),
            TableColumn::new(col_sev).with_width("18%"),
        ])
        .with_title(clusters_title);
        for cluster in &diagnosis.clusters {
            table = table.add_row(vec![
                cluster.label.clone(),
                cluster.finding_count.to_string(),
                cluster.occurrence_total.to_string(),
                cluster.severity_label.clone(),
            ]);
        }
        builder = builder.add_component(table);
        // A cluster needs at least two findings in one category; say so when
        // the category overview above lists categories that are not a cluster,
        // instead of leaving two tables of different length unexplained.
        if diagnosis.clusters.len() < diagnosis.category_breakdown.len() {
            builder = builder.add_component(
                Label::new(i18n.t("diagnosis-clusters-note"))
                    .with_size("8.8pt")
                    .with_color(super::design::tokens::NEUTRAL),
            );
        }
    }

    builder
}
