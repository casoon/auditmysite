//! Finding renderers for PDF reports.

use renderreport::components::advanced::WrongRightBlock;
use renderreport::components::advanced::{KeyValueList, List};
use renderreport::components::text::{Label, TextBlock};
use renderreport::components::{AuditTable, Finding, TableColumn};
use renderreport::prelude::*;

use crate::i18n::I18n;
use crate::output::report_model::*;
use crate::util::truncate_url;

use super::helpers::{
    effort_label_i18n, execution_priority_label, map_severity, priority_label_i18n, role_label_i18n,
};

pub(super) fn render_key_finding_block(
    mut builder: renderreport::engine::ReportBuilder,
    group: &FindingGroup,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    // ── Explain Layer (Bridge: Business → Tech) ──────────────────────
    {
        let has_cause = !group.typical_cause.is_empty();
        let has_user = !group.user_impact.is_empty();
        let has_business = !group.business_impact.is_empty();
        if has_cause || has_user || has_business {
            let mut kv = KeyValueList::new();
            if has_user {
                kv = kv.add("Nutzer erleben", &group.user_impact);
            }
            if has_business {
                kv = kv.add("Business-Auswirkung", &group.business_impact);
            }
            if has_cause {
                kv = kv.add("Warum passiert das", &group.typical_cause);
            }
            builder = builder.add_component(kv);
        }
    }

    // ── Finding Card ─────────────────────────────────────────────────
    let category = format!(
        "{} | {} | {}",
        execution_priority_label(group.execution_priority),
        role_label_i18n(group.responsible_role, i18n),
        effort_label_i18n(group.effort, i18n)
    );

    let mut finding = Finding::new(
        &group.title,
        map_severity(&group.severity),
        &group.customer_description,
    )
    .with_recommendation(&group.recommendation)
    .with_category(category);

    if group.occurrence_count > 0 {
        finding = finding.with_affected(format!("{} Vorkommen", group.occurrence_count));
    }

    builder = builder.add_component(finding);

    builder
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

    let mut category_parts = vec![
        format!(
            "{}: {}",
            i18n.t("label-priority"),
            priority_label_i18n(group.priority, i18n)
        ),
        format!(
            "{}: {}",
            i18n.t("label-owner"),
            role_label_i18n(group.responsible_role, i18n)
        ),
        format!(
            "{}: {}",
            i18n.t("label-effort"),
            effort_label_i18n(group.effort, i18n)
        ),
    ];
    if let Some(ref dim) = group.dimension {
        category_parts.push(format!("{}: {}", i18n.t("label-module"), dim));
    }
    if let Some(ref cls) = group.issue_class {
        category_parts.push(format!("{}: {}", i18n.t("label-type"), cls));
    }

    let finding = Finding::new(
        &group.title,
        map_severity(&group.severity),
        &group.customer_description,
    )
    .with_recommendation(&group.recommendation)
    .with_category(category_parts.join(" | "))
    .with_affected(format!(
        "{} Vorkommen, {} Elemente",
        group.occurrence_count, group.affected_elements
    ));

    builder = builder.add_component(finding);

    let mut details = KeyValueList::new().with_title("Technische Einordnung");
    details = details
        .add(
            "WCAG-Regel",
            if group.wcag_criterion.is_empty() {
                "—".to_string()
            } else {
                group.wcag_criterion.clone()
            },
        )
        .add(
            "Betroffene Elemente",
            format!("{}", group.affected_elements),
        )
        .add(
            "Umsetzungspriorität",
            execution_priority_label(group.execution_priority).to_string(),
        );
    builder = builder.add_component(details);

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

pub(super) fn build_analysis_focus_table() -> AuditTable {
    AuditTable::new(vec![TableColumn::new("Modul"), TableColumn::new("Fokus")])
        .with_title("Analysefokus")
        .add_row(vec![
            "Performance".to_string(),
            "Nutzerwahrnehmung, Ladezeit und Reaktionsverhalten".to_string(),
        ])
        .add_row(vec![
            "SEO".to_string(),
            "Indexierbarkeit, Struktur und inhaltliche Signale".to_string(),
        ])
        .add_row(vec![
            "Sicherheit".to_string(),
            "HTTP-Header, TLS-Setup und fehlende Schutzmechanismen".to_string(),
        ])
        .add_row(vec![
            "Mobile".to_string(),
            "Bedienbarkeit, Responsiveness und Lesbarkeit".to_string(),
        ])
}
