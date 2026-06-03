//! Single-report page-section renderers for `generate_pdf`.
//!
//! Each function takes ownership of the builder, appends its section's
//! components, and returns the builder for further chaining.

use renderreport::components::advanced::{
    ChecklistPanel, ChecklistRow, DiagnosisPanel, DiagnosisRow, KeyValueList, List, PageBreak,
    PhaseBlock, SectionHeaderSplit,
};
use renderreport::components::{AuditTable, TableColumn};
use renderreport::prelude::*;

use super::appendix::build_cli_snapshot_table;
use super::detail_modules::{
    render_a11y_journey_findings, render_ai_visibility, render_best_practices,
    render_budget_violations, render_content_visibility, render_dark_mode, render_journey,
    render_mobile, render_performance, render_security, render_seo, render_source_quality,
    render_tech_stack, render_ux,
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

/// Render a part divider — visually separates the three report parts (#246).
///
/// Each divider produces a page break, a strong level=1 section header tagged
/// with "TEIL N / 3" (or "PART N OF 3"), an audience callout, and a contents list.
#[allow(clippy::too_many_arguments)]
pub(super) fn render_part_divider(
    mut builder: renderreport::engine::ReportBuilder,
    part_num: u8,
    title: &str,
    intro: &str,
    audience_title: &str,
    audience_body: &str,
    contents_title: &str,
    contents: &[&str],
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = i18n.locale() == "en";
    let eyebrow = if en {
        format!("PART {} OF 3", part_num)
    } else {
        format!("TEIL {} VON 3", part_num)
    };
    builder = builder
        .add_component(PageBreak::new())
        .add_component(
            SectionHeaderSplit::new(title, intro)
                .with_eyebrow(eyebrow)
                .with_level(1),
        )
        .add_component(Callout::info(audience_body).with_title(audience_title));
    if !contents.is_empty() {
        let mut list = List::new().with_title(contents_title);
        for item in contents {
            list = list.add_item(*item);
        }
        builder = builder.add_component(list);
    }
    builder
}

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

    builder = render_management_risk_table(builder, vm, i18n);

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

    builder = render_decision_action_table(builder, vm, i18n);

    // ActionTable — full prioritized table
    let wjt_table = build_was_jetzt_tun_table(vm);
    match wjt_table {
        WasJetztTunContent::Table(t) => builder = builder.add_component(t),
        WasJetztTunContent::Empty(c) => builder = builder.add_component(c),
    }

    builder
}

fn render_management_risk_table(
    builder: renderreport::engine::ReportBuilder,
    vm: &ReportViewModel,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = i18n.locale() == "en";
    let legal_level = if vm.severity.critical > 0 {
        if en {
            "High"
        } else {
            "Hoch"
        }
    } else if vm.severity.high > 0 {
        if en {
            "Medium"
        } else {
            "Mittel"
        }
    } else if en {
        "Low"
    } else {
        "Niedrig"
    };
    let conversion_level = if vm.summary.score < 60 {
        if en {
            "High"
        } else {
            "Hoch"
        }
    } else if vm.summary.score < 80 || vm.severity.high > 0 {
        if en {
            "Medium"
        } else {
            "Mittel"
        }
    } else if en {
        "Low"
    } else {
        "Niedrig"
    };
    let project_level = if vm.severity.component_issues >= 3 {
        if en {
            "High"
        } else {
            "Hoch"
        }
    } else if vm.severity.component_issues > 0 {
        if en {
            "Medium"
        } else {
            "Mittel"
        }
    } else if en {
        "Low"
    } else {
        "Niedrig"
    };

    let mut kv = KeyValueList::new().with_title(if en {
        "Management risk view"
    } else {
        "Management-Risikoansicht"
    });
    kv = kv
        .add(
            if en {
                "Legal / BFSG-EAA"
            } else {
                "Recht / BFSG-EAA"
            },
            format!(
                "{} — {} critical/high findings",
                legal_level,
                vm.severity.critical + vm.severity.high
            ),
        )
        .add(
            if en {
                "Conversion / usability"
            } else {
                "Conversion / Nutzbarkeit"
            },
            format!(
                "{} — Accessibility {} / 100",
                conversion_level, vm.summary.score
            ),
        )
        .add(
            if en { "Project risk" } else { "Projektrisiko" },
            format!(
                "{} — {} component/template issue(s), {} occurrence(s)",
                project_level, vm.severity.component_issues, vm.severity.component_occurrences
            ),
        );
    builder.add_component(kv)
}

fn render_decision_action_table(
    builder: renderreport::engine::ReportBuilder,
    vm: &ReportViewModel,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = i18n.locale() == "en";
    let mut rows = Vec::new();
    for item in vm
        .actions
        .roadmap_columns
        .iter()
        .flat_map(|column| column.items.iter())
        .take(8)
    {
        rows.push(vec![
            item.action.clone(),
            item.priority.clone(),
            item.execution_priority.clone(),
            item.effort.clone(),
            item.risk_effect.clone(),
            item.user_effect.clone(),
        ]);
    }
    if rows.is_empty() {
        return builder;
    }

    let mut table = AuditTable::new(vec![
        TableColumn::new(if en { "Action" } else { "Maßnahme" }).with_width("30%"),
        TableColumn::new(if en { "Risk" } else { "Risiko" }).with_width("12%"),
        TableColumn::new(if en { "Priority" } else { "Priorität" }).with_width("14%"),
        TableColumn::new(if en { "Complexity" } else { "Komplexität" }).with_width("14%"),
        TableColumn::new(if en { "Risk effect" } else { "Risiko-Effekt" }).with_width("15%"),
        TableColumn::new(if en { "User effect" } else { "Nutzerwirkung" }).with_width("15%"),
    ])
    .with_title(if en {
        "Decision-oriented top actions"
    } else {
        "Entscheidungsorientierte Top-Maßnahmen"
    });
    for row in rows {
        table = table.add_row(row);
    }
    builder.add_component(table)
}

/// Section 5 — Tech Entry: intro + module health diagnosis panel.
pub(super) fn render_tech_entry(
    mut builder: renderreport::engine::ReportBuilder,
    vm: &ReportViewModel,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    // Part 2 — Technical Accessibility Report (#246).
    let en = i18n.locale() == "en";
    let (p2_title, p2_intro, p2_audience_title, p2_audience_body, p2_contents_title) = if en {
        (
            "Technical Accessibility Report",
            "WCAG compliance, diagnosed findings with code snippets, and the complete list of detected issues.",
            "Audience",
            "Developers and engineering teams. Read this part to understand each finding with its WCAG criterion, HTML evidence, fix guidance, and component attribution.",
            "What's in this part",
        )
    } else {
        (
            "Technischer Accessibility-Report",
            "WCAG-Konformität, diagnostizierte Befunde mit Code-Snippets und die vollständige Liste erkannter Probleme.",
            "Zielgruppe",
            "Entwickler und Engineering-Teams. Dieser Teil zeigt jeden Befund mit WCAG-Kriterium, HTML-Evidenz, Fix-Hinweis und Komponentenzuordnung.",
            "Inhalt dieses Teils",
        )
    };
    let p2_contents: Vec<&str> = if en {
        vec![
            "Module health overview",
            "System diagnosis",
            "Findings by criticality (mandatory vs. optimization)",
            "HTML snippets and fix guidance per finding",
            "WCAG coverage matrix",
            "Complete list of detected violations",
        ]
    } else {
        vec![
            "Modul-Übersicht",
            "Systemdiagnose",
            "Befunde nach Kritikalität (Pflicht vs. Optimierung)",
            "HTML-Snippets und Fix-Hinweise pro Befund",
            "WCAG-Coverage-Matrix",
            "Vollständige Liste aller erkannten Verstöße",
        ]
    };
    builder = render_part_divider(
        builder,
        2,
        p2_title,
        p2_intro,
        p2_audience_title,
        p2_audience_body,
        p2_contents_title,
        &p2_contents,
        i18n,
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
    builder = render_findings_section(builder, vm, en, i18n);
    // WCAG Coverage (issue #37)
    if vm.meta.report_level != ReportLevel::Executive {
        builder = render_wcag_coverage_section(builder, report, i18n);
    }
    // Interactive Accessibility-Journey findings (Phase 2+)
    if !report.interactive_findings.is_empty() {
        builder = render_a11y_journey_findings(
            builder,
            &report.interactive_findings,
            report.accessibility_journey.as_ref(),
            i18n,
        );
    }
    builder = render_appendix_section(builder, vm, i18n);
    builder = render_part3_header(builder, vm, report, i18n);
    builder = render_module_sections(builder, vm, report, i18n);
    builder
}

fn render_findings_section(
    mut builder: renderreport::engine::ReportBuilder,
    vm: &ReportViewModel,
    en: bool,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    // Section 6+ — Findings grouped by criticality tier, then severity (#245).
    if vm.severity.has_issues {
        if !vm.findings.by_tier.is_empty() {
            let (findings_title, findings_intro) = if en {
                (
                    "Findings by Criticality",
                    "All technical findings, separated into mandatory (BFSG-relevant) and optimization.",
                )
            } else {
                (
                    "Befunde nach Kritikalität",
                    "Alle technischen Befunde, getrennt in Pflicht (BFSG-relevant) und Optimierung.",
                )
            };
            builder = builder.add_component(PageBreak::new()).add_component(
                SectionHeaderSplit::new(findings_title, findings_intro)
                    .with_eyebrow(if en { "FINDINGS" } else { "BEFUNDE" })
                    .with_level(2),
            );

            for (tier_idx, criticality) in vm.findings.by_tier.iter().enumerate() {
                let tier_summary = if en {
                    format!(
                        "{} finding(s) — {} occurrence(s) total. {}",
                        criticality.total_findings,
                        criticality.total_occurrences,
                        criticality.intro
                    )
                } else {
                    format!(
                        "{} Befund(e) — {} Vorkommen gesamt. {}",
                        criticality.total_findings,
                        criticality.total_occurrences,
                        criticality.intro
                    )
                };
                // Keep the first tier on the same page as the section divider so
                // the divider page is not nearly empty (#365); break only before
                // subsequent tiers.
                if tier_idx > 0 {
                    builder = builder.add_component(PageBreak::new());
                }
                builder = builder.add_component(
                    SectionHeaderSplit::new(&criticality.label, &tier_summary)
                        .with_eyebrow(&criticality.eyebrow)
                        .with_level(2),
                );
                let tier_callout = match criticality.tier {
                    crate::output::report_model::CriticalityTier::Mandatory => {
                        Callout::error(&criticality.intro).with_title(if en {
                            "Mandatory — legal risk"
                        } else {
                            "Pflicht — rechtliches Risiko"
                        })
                    }
                    crate::output::report_model::CriticalityTier::Optimization => {
                        Callout::info(&criticality.intro).with_title(if en {
                            "Optimization — no direct legal risk"
                        } else {
                            "Optimierung — kein unmittelbares rechtliches Risiko"
                        })
                    }
                };
                builder = builder.add_component(tier_callout);

                for tier in &criticality.by_severity {
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

                    if tier.severity == crate::wcag::Severity::Critical
                        && criticality.tier
                            == crate::output::report_model::CriticalityTier::Mandatory
                    {
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
            }
        } else {
            // Fallback: tier classification unavailable for all findings
            for group in &vm.findings.all_findings {
                builder = render_finding_technical(builder, group, i18n);
            }
        }
    }
    builder
}

fn render_appendix_section(
    mut builder: renderreport::engine::ReportBuilder,
    vm: &ReportViewModel,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = i18n.locale() == "en";
    // Appendix — full violations list, conclusion of Part 2 (#246). Only render
    // the section header when there are violations to list; otherwise it
    // promises a list that never appears (#364).
    if vm.appendix.has_violations {
        let (appendix_title, appendix_intro) = if en {
            (
                "Complete Findings List",
                "Raw audit data and the full list of detected violations.",
            )
        } else {
            (
                "Vollständige Fundstellen",
                "Rohdaten des Audits und die vollständige Liste aller erkannten Verstöße.",
            )
        };
        builder = builder.add_component(PageBreak::new()).add_component(
            SectionHeaderSplit::new(appendix_title, appendix_intro)
                .with_eyebrow(if en { "APPENDIX" } else { "ANHANG" })
                .with_level(2),
        );

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

fn render_part3_header(
    mut builder: renderreport::engine::ReportBuilder,
    vm: &ReportViewModel,
    report: &AuditReport,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = i18n.locale() == "en";
    // Part 3 — SEO, AI & Quality (optional). Only render if any module is present (#246).
    let has_part3 = vm.module_details.seo.is_some()
        || vm.module_details.ai_visibility.is_some()
        || vm.module_details.content_visibility.is_some()
        || vm.module_details.performance.is_some()
        || !report.budget_violations.is_empty()
        || vm.module_details.security.is_some()
        || vm.module_details.mobile.is_some()
        || vm.module_details.ux.is_some()
        || vm.module_details.journey.is_some()
        || vm.module_details.dark_mode.is_some()
        || vm.module_details.source_quality.is_some()
        || vm.module_details.tech_stack.is_some()
        || vm.module_details.best_practices.is_some();

    if has_part3 {
        let (p3_title, p3_intro, p3_audience_title, p3_audience_body, p3_contents_title) = if en {
            (
                "SEO, AI & Quality",
                "Search engine optimization, AI discoverability, and technical quality signals.",
                "Audience",
                "Marketing, SEO specialists, and engineering teams. This part is optional and complements the accessibility findings with discoverability and technical quality metrics.",
                "What's in this part",
            )
        } else {
            (
                "SEO, KI & Qualität",
                "Suchmaschinenoptimierung, KI-Auffindbarkeit und technische Qualitätssignale.",
                "Zielgruppe",
                "Marketing, SEO-Spezialisten und Engineering-Teams. Dieser Teil ist optional und ergänzt die Accessibility-Befunde um Auffindbarkeits- und Qualitätsmetriken.",
                "Inhalt dieses Teils",
            )
        };
        let mut p3_contents: Vec<&str> = Vec::new();
        if vm.module_details.seo.is_some()
            || vm.module_details.ai_visibility.is_some()
            || vm.module_details.content_visibility.is_some()
        {
            p3_contents.push(if en {
                "SEO, AI visibility, and content authority signals"
            } else {
                "SEO, KI-Sichtbarkeit und inhaltliche Autoritätssignale"
            });
        }
        if vm.module_details.performance.is_some() || !report.budget_violations.is_empty() {
            p3_contents.push(if en {
                "Performance details and budget violations"
            } else {
                "Performance-Details und Budget-Verstöße"
            });
        }
        if vm.module_details.security.is_some() {
            p3_contents.push(if en {
                "Security headers"
            } else {
                "Sicherheits-Header"
            });
        }
        if vm.module_details.mobile.is_some() {
            p3_contents.push(if en {
                "Mobile usability"
            } else {
                "Mobile Nutzbarkeit"
            });
        }
        if vm.module_details.ux.is_some() || vm.module_details.journey.is_some() {
            p3_contents.push(if en {
                "UX and user journey"
            } else {
                "UX und Nutzerführung"
            });
        }
        if vm.module_details.dark_mode.is_some() {
            p3_contents.push(if en {
                "Dark mode support"
            } else {
                "Dark-Mode-Unterstützung"
            });
        }
        if vm.module_details.source_quality.is_some()
            || vm.module_details.tech_stack.is_some()
            || vm.module_details.best_practices.is_some()
        {
            p3_contents.push(if en {
                "Source quality, tech stack, and best practices"
            } else {
                "Quellqualität, Tech-Stack und Best Practices"
            });
        }
        builder = render_part_divider(
            builder,
            3,
            p3_title,
            p3_intro,
            p3_audience_title,
            p3_audience_body,
            p3_contents_title,
            &p3_contents,
            i18n,
        );
    }
    builder
}

fn render_module_sections(
    mut builder: renderreport::engine::ReportBuilder,
    vm: &ReportViewModel,
    report: &AuditReport,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = i18n.locale() == "en";
    if let Some(ref seo) = vm.module_details.seo {
        builder = render_seo(builder, seo, i18n);
    }
    if let Some(ref av) = vm.module_details.ai_visibility {
        builder = render_ai_visibility(builder, av, i18n);
    }
    if let Some(ref cv) = vm.module_details.content_visibility {
        builder = render_content_visibility(builder, cv, i18n);
    }

    // Technology & Quality sub-section (level=2 inside Part 3).
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
            // Sub-divider that groups the technical modules — must sit one level
            // above the L2 module headings it introduces, otherwise two equal
            // L2 headings collide with no content between them (#363).
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
    builder
}
