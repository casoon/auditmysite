//! Diagnosis section and scope helpers for PDF reports.

use renderreport::components::{AuditTable, TableColumn};
use renderreport::prelude::*;

use crate::i18n::I18n;
use crate::output::report_model::*;

pub(super) fn render_diagnosis_section(
    mut builder: renderreport::engine::ReportBuilder,
    diagnosis: &DiagnosisBlock,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = i18n.locale() == "en";

    builder = builder.add_component(Section::new(&diagnosis.section_title).with_level(2));

    // Pattern overview — label is the callout title; the body carries only the
    // description so the label is not repeated as a prefix in the body text.
    let pattern_intro = diagnosis.pattern_description.clone();
    let callout = if diagnosis.is_systematic {
        Callout::warning(&pattern_intro).with_title(&diagnosis.pattern_label)
    } else {
        Callout::info(&pattern_intro).with_title(&diagnosis.pattern_label)
    };
    builder = builder.add_component(callout);

    // Dominant issue spotlight
    if let Some(ref dominant) = diagnosis.dominant_issue {
        let spotlight_text = if en {
            format!(
                "\"{}\" accounts for the majority of critical/high findings.",
                dominant
            )
        } else {
            format!(
                "\"{}\" verursacht den Großteil der kritischen/hohen Findings.",
                dominant
            )
        };
        builder = builder.add_component(Callout::warning(&spotlight_text));
    }

    // Category breakdown table
    if !diagnosis.category_breakdown.is_empty() {
        let col_dim = i18n.t("diagnosis-col-category");
        let col_count = i18n.t("diagnosis-col-findings");
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
    }

    builder
}

pub(super) fn output_scope_callout(i18n: &I18n) -> Callout {
    if i18n.locale() == "en" {
        Callout::info(
            "This PDF is a condensed decision and prioritization report. It highlights the most important findings, risks and actions, but does not list every technical occurrence. Details and raw data are available in the technical appendix of this report.",
        )
        .with_title("How to read this report")
    } else {
        Callout::info(
            "Dieser PDF-Report ist eine verdichtete Entscheidungs- und Priorisierungshilfe. Er zeigt die wichtigsten Befunde, Risiken und Maßnahmen, enthält aber nicht jede technische Einzelstelle. Details und Rohdaten finden sich im technischen Anhang dieses Reports.",
        )
        .with_title("Einordnung dieses Reports")
    }
}

/// Map page type + URL to business relevance (hoch/mittel/niedrig)
pub(super) fn format_word_count(n: u32) -> String {
    if n >= 1_000 {
        format!("{}.{:03}", n / 1_000, n % 1_000)
    } else {
        n.to_string()
    }
}

pub(super) fn business_relevance(page_type: Option<&str>, url: &str, locale: &str) -> &'static str {
    let en = locale == "en";
    let high = if en { "high" } else { "hoch" };
    let medium = if en { "medium" } else { "mittel" };
    let low = if en { "low" } else { "niedrig" };

    // URL-based heuristics first
    let path = url.to_lowercase();
    if path.contains("impressum")
        || path.contains("datenschutz")
        || path.contains("agb")
        || path.contains("imprint")
        || path.contains("privacy")
        || path.contains("terms")
    {
        return low;
    }
    if path.ends_with('/') && path.matches('/').count() <= 3 {
        return high; // homepage or top-level pages
    }

    // Page type based
    match page_type {
        Some("Marketing / Landing Page") => high,
        Some("Transaktional / Utility") | Some("Transactional / Utility") => high,
        Some("Editorial / Artikel") | Some("Editorial / Article") => medium,
        Some("Strukturierter Wissensinhalt") | Some("Structured knowledge content") => medium,
        Some("Navigations- / Hub-Seite") | Some("Navigation / Hub page") => medium,
        Some("Medienorientierte Seite") | Some("Media-oriented page") => medium,
        Some("Thin / Minimal Content") => low,
        _ => medium,
    }
}
