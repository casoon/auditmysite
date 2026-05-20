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
    render_ai_visibility, render_best_practices, render_budget_violations,
    render_content_visibility, render_dark_mode, render_journey, render_mobile, render_performance,
    render_security, render_seo, render_source_quality, render_tech_stack, render_ux,
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
        .with_level(2),
    );

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
    // Group 2 — ACCESSIBILITY (Ebene 2 Umsetzung) — #217/#218
    builder = builder.add_component(PageBreak::new()).add_component(
        SectionHeaderSplit::new(
            "Accessibility",
            if i18n.locale() == "en" {
                "Implementation layer: WCAG compliance, diagnosed findings, and remediation guidance."
            } else {
                "Umsetzungsebene: WCAG-Konformität, diagnostizierte Befunde und Behebungshinweise."
            },
        )
        .with_eyebrow(if i18n.locale() == "en" {
            "IMPLEMENTATION · LEVEL 2"
        } else {
            "UMSETZUNG · EBENE 2"
        })
        .with_level(1),
    );
    builder = builder.add_component(
        SectionHeaderSplit::new(&vm.executive.technical_title, &vm.executive.technical_intro)
            .with_eyebrow(if i18n.locale() == "en" {
                "DEVELOPER"
            } else {
                "ENTWICKLER"
            })
            .with_level(2),
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

/// Sections 5b + 6+ — diagnosis, findings by severity tier, module metrics, appendix.
pub(super) fn render_tech_details(
    mut builder: renderreport::engine::ReportBuilder,
    vm: &ReportViewModel,
    report: &AuditReport,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = i18n.locale() == "en";

    // Section 5b — System Diagnosis
    if vm.severity.has_issues {
        builder = render_diagnosis_section(builder, &vm.diagnosis, i18n);
    }

    // Section 6+ — Findings grouped by severity tier (#201 / #205 / #208)
    if vm.severity.has_issues {
        if !vm.findings.by_severity.is_empty() {
            let (findings_title, findings_intro) = if en {
                (
                    "Findings by Severity",
                    "All technical findings grouped by severity. Critical and high-severity issues require immediate remediation.",
                )
            } else {
                (
                    "Befunde nach Schweregrad",
                    "Alle technischen Befunde gruppiert nach Schweregrad. Kritische und hohe Befunde erfordern sofortige Behebung.",
                )
            };
            builder = builder.add_component(PageBreak::new()).add_component(
                SectionHeaderSplit::new(findings_title, findings_intro)
                    .with_eyebrow(if en { "FINDINGS" } else { "BEFUNDE" })
                    .with_level(2),
            );

            for tier in &vm.findings.by_severity {
                let tier_intro = if en {
                    format!(
                        "{} finding(s) — {} occurrence(s) total",
                        tier.findings.len(),
                        tier.total_occurrences
                    )
                } else {
                    format!(
                        "{} Befund(e) — {} Vorkommen gesamt",
                        tier.findings.len(),
                        tier.total_occurrences
                    )
                };
                builder = builder.add_component(
                    SectionHeaderSplit::new(&tier.label, &tier_intro)
                        .with_eyebrow(tier.label.to_uppercase())
                        .with_level(3),
                );

                if tier.severity == crate::wcag::Severity::Critical {
                    let msg = if en {
                        format!(
                            "{} critical issue(s) blocking accessibility — must be resolved before deployment.",
                            tier.findings.len()
                        )
                    } else {
                        format!(
                            "{} kritische(r) Befund(e) blockiert/blockieren die Barrierefreiheit — vor dem Deployment zu beheben.",
                            tier.findings.len()
                        )
                    };
                    builder = builder.add_component(Callout::error(&msg));
                }

                for group in &tier.findings {
                    builder = render_finding_technical(builder, group, i18n);
                }
            }
        } else {
            // Fallback: severity unknown for all findings
            for group in &vm.findings.all_findings {
                builder = render_finding_technical(builder, group, i18n);
            }
        }
    }

    // WCAG Coverage (issue #37)
    if vm.meta.report_level != ReportLevel::Executive {
        builder = render_wcag_coverage_section(builder, report, i18n);
    }

    // Group 3 — SEO & SICHTBARKEIT (Ebene 3 Analyse) — #217/#218
    if vm.module_details.seo.is_some()
        || vm.module_details.ai_visibility.is_some()
        || vm.module_details.content_visibility.is_some()
    {
        builder = builder.add_component(PageBreak::new()).add_component(
            SectionHeaderSplit::new(
                if en {
                    "SEO & Visibility"
                } else {
                    "SEO & Sichtbarkeit"
                },
                if en {
                    "Search engine optimization, AI discoverability, and content authority signals."
                } else {
                    "Suchmaschinenoptimierung, KI-Auffindbarkeit und inhaltliche Autoritätssignale."
                },
            )
            .with_eyebrow(if en {
                "ANALYSIS · LEVEL 3"
            } else {
                "ANALYSE · EBENE 3"
            })
            .with_level(1),
        );
    }
    if let Some(ref seo) = vm.module_details.seo {
        builder = render_seo(builder, seo, i18n);
    }
    if let Some(ref av) = vm.module_details.ai_visibility {
        builder = render_ai_visibility(builder, av, i18n);
    }
    if let Some(ref cv) = vm.module_details.content_visibility {
        builder = render_content_visibility(builder, cv, i18n);
    }

    // Group 4 — TECHNIK & QUALITÄT (Ebene 3 Analyse) — #217/#218
    if vm.module_details.performance.is_some()
        || !report.budget_violations.is_empty()
        || vm.module_details.security.is_some()
        || vm.module_details.mobile.is_some()
        || vm.module_details.ux.is_some()
        || vm.module_details.journey.is_some()
        || vm.module_details.dark_mode.is_some()
        || vm.module_details.source_quality.is_some()
        || vm.module_details.tech_stack.is_some()
        || vm.module_details.best_practices.is_some()
    {
        builder = builder.add_component(PageBreak::new()).add_component(
            SectionHeaderSplit::new(
                if en { "Technology & Quality" } else { "Technik & Qualität" },
                if en {
                    "Core technical quality: performance, security, mobile usability, UX, and engineering foundations."
                } else {
                    "Technische Kernqualität: Performance, Sicherheit, Mobile-Nutzbarkeit, UX und technische Grundlagen."
                },
            )
            .with_eyebrow(if en { "ANALYSIS" } else { "ANALYSE" })
            .with_level(1),
        );
    }
    if let Some(ref perf) = vm.module_details.performance {
        builder = render_performance(builder, perf, i18n);
    }
    if !report.budget_violations.is_empty() {
        builder = render_budget_violations(builder, &report.budget_violations, i18n);
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
    if let Some(ref ts) = vm.module_details.tech_stack {
        builder = render_tech_stack(builder, ts, i18n);
    }
    if let Some(ref bp) = vm.module_details.best_practices {
        builder = render_best_practices(builder, bp, i18n);
    }

    // Group 5 — ANHANG (Ebene 3, immer sichtbar) — #217/#218
    {
        let (appendix_title, appendix_intro) = if en {
            (
                "Appendix",
                "Raw audit data, methodology, and technical references.",
            )
        } else {
            (
                "Anhang",
                "Rohdaten des Audits, Methodik und technische Referenzen.",
            )
        };
        builder = builder.add_component(PageBreak::new()).add_component(
            SectionHeaderSplit::new(appendix_title, appendix_intro)
                .with_eyebrow(if en { "APPENDIX" } else { "ANHANG" })
                .with_level(1),
        );
    }

    if vm.appendix.has_violations {
        builder = builder.add_component(build_cli_snapshot_table(vm, i18n));

        // JSON hint in appendix (#219)
        let json_note = if en {
            "The accompanying JSON report contains the complete machine-readable issue list with selectors, occurrences, and all detail data for automated processing."
        } else {
            "Der begleitende JSON-Report enthält die vollständige maschinenlesbare Fehlerliste mit Selektoren, Vorkommen und allen Detaildaten für automatisierte Weiterverarbeitung."
        };
        builder = builder.add_component(Callout::info(json_note).with_title(if en {
            "Raw data & processing"
        } else {
            "Rohdaten & Weiterverarbeitung"
        }));

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
