//! Single-report page-section renderers for `generate_pdf`.
//!
//! Each function takes ownership of the builder, appends its section's
//! components, and returns the builder for further chaining.

use renderreport::components::advanced::{
    ChecklistPanel, ChecklistRow, DiagnosisPanel, DiagnosisRow, PageBreak, PhaseBlock,
    SectionHeaderSplit,
};
use renderreport::prelude::*;

use super::appendix::build_cli_snapshot_table;
use super::detail_modules::{
    render_ai_visibility, render_budget_violations, render_content_visibility, render_dark_mode,
    render_journey, render_mobile, render_performance, render_security, render_seo,
    render_source_quality, render_tech_stack, render_ux,
};
use super::diagnosis::render_diagnosis_section;
use super::findings::render_finding_technical;
use super::helpers::map_severity;
use super::modules::{build_was_jetzt_tun_table, WasJetztTunContent};
use super::wcag_coverage::render_wcag_coverage_section;
use crate::audit::AuditReport;
use crate::cli::ReportLevel;
use crate::i18n::I18n;
use crate::output::report_model::*;

/// Section 4 — Action Plan: quick wins, prioritized actions, execution note.
pub(super) fn render_action_plan(
    mut builder: renderreport::engine::ReportBuilder,
    vm: &ReportViewModel,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    builder = builder.add_component(PageBreak::new()).add_component(
        SectionHeaderSplit::new(
            &vm.executive.action_plan_title,
            &vm.executive.action_plan_intro,
        )
        .with_eyebrow(if i18n.locale() == "en" {
            "IMPLEMENTATION PLAN"
        } else {
            "UMSETZUNGSPLAN"
        })
        .with_level(1),
    );

    // "Jetzt starten" — top 1-3 highest-priority actions
    let top_actions: Vec<&RoadmapItemData> = vm
        .actions
        .roadmap_columns
        .first()
        .map(|col| col.items.iter().take(3).collect())
        .unwrap_or_default();

    if !top_actions.is_empty() {
        let body = top_actions
            .iter()
            .enumerate()
            .map(|(i, item)| {
                if item.user_effect.is_empty() {
                    format!("{}. {}", i + 1, item.action)
                } else {
                    format!("{}. {} — {}", i + 1, item.action, item.user_effect)
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        let title = if i18n.locale() == "en" {
            "Start here — highest leverage"
        } else {
            "Jetzt starten — höchste Hebelwirkung"
        };
        builder = builder.add_component(Callout::warning(&body).with_title(title));
    }

    // Empfohlene Vorgehensweise
    builder = builder.add_component(
        Callout::info(&vm.executive.action_plan_callout_body)
            .with_title(&vm.executive.action_plan_callout_title),
    );

    if !vm.actions.phase_preview.is_empty() {
        for (idx, phase) in vm.actions.phase_preview.iter().enumerate() {
            builder = builder.add_component(
                PhaseBlock::new((idx + 1) as u8, &phase.phase_label, &phase.description)
                    .with_items(phase.top_items.clone())
                    .with_total(phase.item_count)
                    .with_color(&phase.accent_color),
            );
        }
    }

    // QuickWins — immediate actions
    let quick_items: Vec<&RoadmapItemData> = vm
        .actions
        .roadmap_columns
        .iter()
        .flat_map(|col| col.items.iter())
        .filter(|i| {
            i.execution_priority.contains("Direkt") || i.execution_priority.contains("Sofort")
        })
        .take(5)
        .collect();

    if !quick_items.is_empty() {
        let rows: Vec<ChecklistRow> = quick_items
            .iter()
            .map(|item| ChecklistRow::new(&item.action, &item.user_effect).with_status("warn"))
            .collect();
        builder = builder
            .add_component(ChecklistPanel::new(rows).with_title(i18n.t("panel-quick-actions")));
    }

    // ActionTable — full prioritized table
    let wjt_table = build_was_jetzt_tun_table(vm);
    match wjt_table {
        WasJetztTunContent::Table(t) => builder = builder.add_component(t),
        WasJetztTunContent::Empty(c) => builder = builder.add_component(c),
    }

    builder
}

/// Section 5 — Tech Entry: intro + module health diagnosis panel.
pub(super) fn render_tech_entry(
    mut builder: renderreport::engine::ReportBuilder,
    vm: &ReportViewModel,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    builder = builder.add_component(PageBreak::new()).add_component(
        SectionHeaderSplit::new(&vm.executive.technical_title, &vm.executive.technical_intro)
            .with_eyebrow("TECHNICAL HANDOFF")
            .with_level(1),
    );

    if !vm.modules.dashboard.is_empty() {
        let diag_rows: Vec<DiagnosisRow> = vm
            .modules
            .dashboard
            .iter()
            .map(|module| {
                let status = if module.score >= 80 {
                    "good"
                } else if module.score >= 50 {
                    "warn"
                } else {
                    "bad"
                };
                let display_name = if module.measurement_type == "heuristic" {
                    let suffix = if i18n.locale() == "en" {
                        "Indicator"
                    } else {
                        "Indikator"
                    };
                    format!("{} ({suffix})", module.name)
                } else {
                    module.name.clone()
                };
                DiagnosisRow::new(&display_name, format!("{}/100", module.score))
                    .with_status(status)
            })
            .collect();
        builder = builder.add_component(
            DiagnosisPanel::new(diag_rows).with_title(i18n.t("panel-modules-overview")),
        );
    }

    builder
}

/// Sections 5b + 6+ — diagnosis, WCAG details, module metrics, appendix.
pub(super) fn render_tech_details(
    mut builder: renderreport::engine::ReportBuilder,
    vm: &ReportViewModel,
    report: &AuditReport,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    // Section 5b — System Diagnosis
    if vm.severity.has_issues {
        builder = render_diagnosis_section(builder, &vm.diagnosis, i18n);
    }

    // Section 6+ — Tech Details: WCAG findings
    if vm.severity.has_issues {
        for group in &vm.findings.all_findings {
            builder = render_finding_technical(builder, group, i18n);
        }
    }

    // WCAG Coverage (issue #37)
    if vm.meta.report_level != ReportLevel::Executive {
        builder = render_wcag_coverage_section(builder, report, i18n);
    }

    // Module Detail Metrics
    if vm.module_details.has_any {
        builder = builder
            .add_component(PageBreak::new())
            .add_component(Section::new(i18n.t("section-tech-detail-metrics")).with_level(1));
    }

    if let Some(ref perf) = vm.module_details.performance {
        builder = render_performance(builder, perf, i18n);
    }
    if !report.budget_violations.is_empty() {
        builder = render_budget_violations(builder, &report.budget_violations, i18n);
    }
    if let Some(ref seo) = vm.module_details.seo {
        builder = render_seo(builder, seo, i18n);
    }
    if let Some(ref sec) = vm.module_details.security {
        builder = render_security(builder, sec, i18n);
    }
    if let Some(ref mobile) = vm.module_details.mobile {
        builder = render_mobile(builder, mobile, i18n);
    }
    if let Some(ref ux) = vm.module_details.ux {
        builder = render_ux(builder, ux, i18n);
    }
    if let Some(ref journey) = vm.module_details.journey {
        builder = render_journey(builder, journey, i18n);
    }
    if let Some(ref dm) = vm.module_details.dark_mode {
        builder = render_dark_mode(builder, dm, i18n);
    }
    if let Some(ref sq) = vm.module_details.source_quality {
        builder = render_source_quality(builder, sq, i18n);
    }
    if let Some(ref av) = vm.module_details.ai_visibility {
        builder = render_ai_visibility(builder, av, i18n);
    }
    if let Some(ref ts) = vm.module_details.tech_stack {
        builder = render_tech_stack(builder, ts, i18n);
    }
    if let Some(ref cv) = vm.module_details.content_visibility {
        builder = render_content_visibility(builder, cv, i18n);
    }

    // Appendix
    if vm.appendix.has_violations {
        builder = builder
            .add_component(PageBreak::new())
            .add_component(Section::new(i18n.t("section-appendix")).with_level(1));

        builder = builder.add_component(build_cli_snapshot_table(vm, i18n));

        if vm.meta.report_level == ReportLevel::Technical {
            for v in &vm.appendix.violations {
                let mut desc = v.message.clone();
                if let Some(ref fix) = v.fix_suggestion {
                    desc.push_str(&format!("\n\nFix: {}", fix));
                }
                desc.push_str(&format!(
                    "\n\n{} Elemente betroffen",
                    v.affected_elements.len()
                ));
                let useful_selectors: Vec<&str> = v
                    .affected_elements
                    .iter()
                    .map(|e| e.selector.as_str())
                    .filter(|s| {
                        s.contains('.')
                            || s.contains('#')
                            || s.contains('[')
                            || s.contains('>')
                            || s.contains(' ')
                    })
                    .collect();
                if !useful_selectors.is_empty() {
                    desc.push_str(&format!("\nSelektoren: {}", useful_selectors.join(", ")));
                }
                builder = builder.add_component(Finding::new(
                    format!("{} — {}", v.rule, v.rule_name),
                    map_severity(&v.severity),
                    &desc,
                ));
            }
        } else {
            let rows: Vec<ChecklistRow> = vm
                .appendix
                .violations
                .iter()
                .map(|v| {
                    let status = match v.severity {
                        crate::wcag::Severity::Critical => "bad",
                        crate::wcag::Severity::High => "warn",
                        _ => "neutral",
                    };
                    ChecklistRow::new(format!("{} — {}", v.rule, v.rule_name), v.message.clone())
                        .with_status(status)
                })
                .collect();
            builder = builder.add_component(
                ChecklistPanel::new(rows).with_title(i18n.t("section-all-violations")),
            );
        }
    }

    builder
}
