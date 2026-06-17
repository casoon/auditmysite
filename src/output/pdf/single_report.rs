//! Single-report page-section renderers for `generate_pdf`.
//!
//! Each function takes ownership of the builder, appends its section's
//! components, and returns the builder for further chaining.

use renderreport::components::advanced::{
    ChecklistPanel, ChecklistRow, DevicePreview, DiagnosisPanel, DiagnosisRow, List, PageBreak,
    SectionHeaderSplit,
};
use renderreport::components::text::Label;
use renderreport::components::{AuditTable, Finding, TableColumn};
use renderreport::prelude::*;

use super::appendix::build_cli_snapshot_table;
use super::detail_modules::{
    render_a11y_journey_findings, render_ai_visibility, render_best_practices,
    render_budget_violations, render_content_visibility, render_dark_mode, render_journey,
    render_mobile, render_performance, render_screen_reader_section, render_search_experience,
    render_security, render_seo, render_source_quality, render_tech_stack, render_ux,
};
use super::diagnosis::render_diagnosis_section;
use super::findings::render_finding_technical;
use super::helpers::map_severity;
use super::wcag_coverage::render_wcag_coverage_section;
use crate::audit::AuditReport;
use crate::cli::ReportLevel;
use crate::i18n::I18n;
use crate::output::module::active_report_modules;
use crate::output::report_model::*;

/// Render a part divider — visually separates the three report parts (#246).
///
/// Each divider produces a page break, a strong level=1 section header tagged
/// with "TEIL N / 3" (or "PART N OF 3"), an audience callout, and a contents list.
pub(super) fn render_part_divider(
    mut builder: renderreport::engine::ReportBuilder,
    part_num: u8,
    title: &str,
    intro: &str,
    audience_title: &str,
    audience_body: &str,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = i18n.locale() == "en";
    let eyebrow = if en {
        format!("PART {} OF 3", part_num)
    } else {
        format!("TEIL {} VON 3", part_num)
    };
    if part_num > 1 {
        builder = builder.add_component(PageBreak::new());
    }
    builder = builder
        .add_component(
            SectionHeaderSplit::new(title, intro)
                .with_eyebrow(eyebrow)
                .with_level(1),
        )
        .add_component(
            Label::new(format!("{}: {}", audience_title, audience_body))
                .with_size("10.5pt")
                .with_color("#475569"),
        );
    builder
}

pub(super) fn build_customer_diagnosis_panel(vm: &ReportViewModel, i18n: &I18n) -> ChecklistPanel {
    let en = i18n.locale() == "en";
    let mut rows = Vec::new();
    let accessibility = if vm.severity.total > 0 {
        if en {
            format!(
                "{} accessibility occurrence(s) affect operation, orientation or perception.",
                vm.severity.total
            )
        } else {
            format!(
                "{} Accessibility-Vorkommen betreffen Bedienbarkeit, Orientierung oder Wahrnehmbarkeit.",
                vm.severity.total
            )
        }
    } else if en {
        "No automatically confirmed accessibility barriers were detected in the tested scope."
            .to_string()
    } else {
        "Im automatisch geprüften Umfang wurden keine bestätigten Accessibility-Barrieren erkannt."
            .to_string()
    };
    rows.push(
        ChecklistRow::new(
            if en {
                "Accessibility"
            } else {
                "Barrierefreiheit"
            },
            accessibility,
        )
        .with_status(if vm.severity.total > 0 {
            "warn"
        } else {
            "good"
        }),
    );

    for module in vm
        .modules
        .dashboard
        .iter()
        .filter(|m| m.name != "Accessibility")
    {
        if module.score >= 85 {
            continue;
        }
        let status = if module.score < 50 { "bad" } else { "warn" };
        let detail = if module.interpretation.is_empty() {
            format!("{}/100", module.score)
        } else {
            format!("{}/100 — {}", module.score, module.interpretation)
        };
        rows.push(ChecklistRow::new(&module.name, detail).with_status(status));
        if rows.len() >= 6 {
            break;
        }
    }

    if rows.len() == 1 && !vm.modules.dashboard.is_empty() {
        let text = if en {
            "The audited modules are mostly stable; the detailed sections document remaining quality signals."
        } else {
            "Die geprüften Module sind überwiegend stabil; die Detailkapitel zeigen verbleibende Qualitätssignale."
        };
        rows.push(
            ChecklistRow::new(if en { "Overall picture" } else { "Gesamtbild" }, text)
                .with_status("good"),
        );
    }

    ChecklistPanel::new(rows).with_title(if en {
        "What the audit says about the site"
    } else {
        "Was der Audit über die Seite aussagt"
    })
}

pub(super) fn build_top_risks_checklist(vm: &ReportViewModel, i18n: &I18n) -> ChecklistPanel {
    let en = i18n.locale() == "en";
    let mut rows = Vec::new();

    // 1. Usability / Nutzbarkeit
    let user_desc = if vm.severity.critical > 0 || vm.severity.high > 0 {
        if en {
            "Screen reader and keyboard navigation blocked at critical points."
        } else {
            "Screenreader und Tastaturnavigation an kritischen Stellen blockiert oder beeinträchtigt."
        }
    } else {
        if en {
            "No major usability limitations detected."
        } else {
            "Keine wesentlichen Einschränkungen der Nutzbarkeit erkannt."
        }
    };
    let user_status = if vm.severity.critical > 0 {
        "bad"
    } else {
        "warn"
    };
    rows.push(
        ChecklistRow::new(
            if en { "1. Usability" } else { "1. Nutzbarkeit" },
            user_desc,
        )
        .with_status(user_status),
    );

    // 2. Compliance / Rechtliches
    let legal_desc = if vm.cover.critical_issues > 0 {
        if en {
            "BFSG-relevant violations found (WCAG Level A/AA) — resolution recommended."
        } else {
            "BFSG-relevante Verstöße (WCAG Level A/AA) gefunden — Behebung empfohlen."
        }
    } else {
        if en {
            "Low compliance risk."
        } else {
            "Geringes Compliance-Risiko."
        }
    };
    let legal_status = if vm.cover.critical_issues > 0 {
        "bad"
    } else {
        "warn"
    };
    rows.push(
        ChecklistRow::new(
            if en {
                "2. Legal Conformance"
            } else {
                "2. Rechtliche Konformität (BFSG)"
            },
            legal_desc,
        )
        .with_status(legal_status),
    );

    // 3. Conversion / Business-Risiko
    let business_desc = if vm.summary.score < 60 {
        if en {
            "High risk of process abandonment and conversion loss."
        } else {
            "Hohes Risiko von Prozessabbrüchen und Conversion-Verlusten."
        }
    } else if vm.summary.score < 80 {
        if en {
            "Usability hurdles may reduce conversion rate."
        } else {
            "Nutzungshürden können Conversion-Rate reduzieren."
        }
    } else {
        if en {
            "Low business risk, minor optimizations recommended."
        } else {
            "Geringes geschäftliches Risiko, kleine Optimierungen empfohlen."
        }
    };
    let business_status = if vm.summary.score < 60 { "bad" } else { "warn" };
    rows.push(
        ChecklistRow::new(
            if en {
                "3. Conversion & Absprungrate"
            } else {
                "3. Conversion & Business-Risiko"
            },
            business_desc,
        )
        .with_status(business_status),
    );

    // 4. SEO & AI Visibility
    let seo_score = vm
        .modules
        .dashboard
        .iter()
        .find(|m| m.name.contains("SEO"))
        .map(|m| m.score)
        .unwrap_or(100);
    let seo_desc = if seo_score < 70 {
        if en {
            "Missing metadata or structured schemas reduce search engine and AI crawlability."
        } else {
            "Fehlende Metadaten oder strukturierte Daten behindern Google- und KI-Crawler."
        }
    } else if seo_score < 90 {
        if en {
            "Some optimization potential for search engines and AI agents."
        } else {
            "Optimierungspotenzial für Suchmaschinen und KI-Agenten vorhanden."
        }
    } else {
        if en {
            "Excellent discoverability and schema definition."
        } else {
            "Sehr gute Auffindbarkeit und strukturierte Schema-Daten."
        }
    };
    let seo_status = if seo_score < 70 {
        "bad"
    } else if seo_score < 90 {
        "warn"
    } else {
        "good"
    };
    rows.push(
        ChecklistRow::new(
            if en {
                "4. SEO & AI Visibility"
            } else {
                "4. SEO & KI-Sichtbarkeit"
            },
            seo_desc,
        )
        .with_status(seo_status),
    );

    // 5. Loading & Mobile experience
    let perf_score = vm
        .modules
        .dashboard
        .iter()
        .find(|m| m.name.contains("Performance") || m.name.contains("Ladezeit"))
        .map(|m| m.score)
        .unwrap_or(100);
    let mobile_score = vm
        .modules
        .dashboard
        .iter()
        .find(|m| m.name.contains("Mobile") || m.name.contains("Mobilfreundlichkeit"))
        .map(|m| m.score)
        .unwrap_or(100);
    let speed_desc = if perf_score < 60 || mobile_score < 60 {
        if en {
            "High loading latency or touch target layout issues frustrate mobile visitors."
        } else {
            "Hohe Ladezeiten oder Touch-Target-Mängel frustrieren mobile Besucher."
        }
    } else if perf_score < 85 || mobile_score < 85 {
        if en {
            "Moderate performance bottlenecks on mobile networks."
        } else {
            "Mäßige Performance-Verzögerungen im Mobilfunknetz."
        }
    } else {
        if en {
            "Fast load speed and optimal responsive viewport."
        } else {
            "Schnelle Ladezeit und optimale responsive Darstellung."
        }
    };
    let speed_status = if perf_score < 60 || mobile_score < 60 {
        "bad"
    } else if perf_score < 85 || mobile_score < 85 {
        "warn"
    } else {
        "good"
    };
    rows.push(
        ChecklistRow::new(
            if en {
                "5. Performance & Mobile"
            } else {
                "5. Ladezeit & Mobilfreundlichkeit"
            },
            speed_desc,
        )
        .with_status(speed_status),
    );

    ChecklistPanel::new(rows).with_title(if en {
        "5 Key Risks"
    } else {
        "Die 5 wichtigsten Risiken"
    })
}

pub(super) fn build_top_measures_list(vm: &ReportViewModel, i18n: &I18n) -> List {
    let en = i18n.locale() == "en";
    let list_title = if en {
        "5 Key Measures"
    } else {
        "Die 5 wichtigsten Maßnahmen"
    };
    let mut list = List::new().with_title(list_title);

    for (idx, group) in vm.findings.top_findings.iter().take(5).enumerate() {
        let text = format!("{}. {}: {}", idx + 1, group.title, group.recommendation);
        list = list.add_item(&text);
    }

    if vm.findings.top_findings.is_empty() {
        let no_measures = if en {
            "No urgent actions required."
        } else {
            "Keine dringenden Maßnahmen erforderlich."
        };
        list = list.add_item(no_measures);
    }

    list
}

pub(super) fn render_management_page(
    mut builder: renderreport::engine::ReportBuilder,
    vm: &ReportViewModel,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = i18n.locale() == "en";
    let mgt_title = if en {
        "Management Summary"
    } else {
        "Management-Sicht"
    };
    let mgt_subtitle = if en {
        "Overall status, key risks, prioritized actions, and leverage at a glance."
    } else {
        "Gesamtstatus, Hauptrisiken, wichtigste Maßnahmen und Hebel auf einen Blick."
    };

    builder = builder.add_component(
        SectionHeaderSplit::new(mgt_title, mgt_subtitle)
            .with_eyebrow(if en {
                "MANAGEMENT SUMMARY"
            } else {
                "MANAGEMENT-SUMMARY"
            })
            .with_level(1),
    );

    // 1. Gesamturteil Callout
    builder = builder.add_component(Callout::info(&vm.summary.verdict).with_title(if en {
        "Overall Verdict"
    } else {
        "Gesamturteil"
    }));

    // 2. Score overview panel
    builder = builder.add_component(build_customer_diagnosis_panel(vm, i18n));

    // 3. Risks
    builder = builder.add_component(build_top_risks_checklist(vm, i18n));

    // 4. Measures
    builder = builder.add_component(build_top_measures_list(vm, i18n));

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

    // Technical modules overview DiagnosisPanel
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

    if vm.severity.has_issues {
        builder = render_diagnosis_section(builder, &vm.diagnosis, i18n);
    }
    builder = render_findings_section(builder, vm, en, i18n);

    // Interactive Accessibility-Journey findings (Phase 2+)
    if !report.interactive_findings.is_empty() {
        builder = render_a11y_journey_findings(
            builder,
            &report.interactive_findings,
            report.accessibility_journey.as_ref(),
            i18n,
        );
    }

    // Screen-reader reading-order audit (#411)
    if let Some(sr) = report.screen_reader_audit.as_ref() {
        builder = render_screen_reader_section(builder, sr, i18n);
    }

    builder
}

pub(super) fn render_appendix_full(
    mut builder: renderreport::engine::ReportBuilder,
    vm: &ReportViewModel,
    report: &AuditReport,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = i18n.locale() == "en";
    let (app_title, app_intro) = if en {
        (
            "Appendix & Methodology",
            "Technical scope, methodology, WCAG coverage, and the complete violations list.",
        )
    } else {
        (
            "Anhang & Methodik",
            "Prüfumfang, Methodik, WCAG-Coverage und die vollständige Fundstellenliste.",
        )
    };
    builder = builder.add_component(PageBreak::new()).add_component(
        SectionHeaderSplit::new(app_title, app_intro)
            .with_eyebrow(if en { "APPENDIX" } else { "ANHANG" })
            .with_level(1),
    );

    // WCAG Coverage (issue #37)
    if vm.meta.report_level != ReportLevel::Executive {
        builder = render_wcag_coverage_section(builder, report, i18n);
    }

    // Methodology / disclaimer text blocks
    let limitations_title = i18n.t("callout-limitations-title");
    let limitations_text = format!("{}: {}", limitations_title, vm.methodology.limitations);
    builder = builder.add_component(
        Label::new(&limitations_text)
            .with_size("10.5pt")
            .with_color("#475569"),
    );

    let disclaimer_title = i18n.t("callout-note-title");
    let disclaimer_text = format!("{}: {}", disclaimer_title, vm.methodology.disclaimer);
    builder = builder.add_component(
        Label::new(&disclaimer_text)
            .with_size("10.5pt")
            .with_color("#475569"),
    );

    // Complete violations list
    builder = render_appendix_section(builder, vm, i18n);

    builder
}

fn render_findings_section(
    mut builder: renderreport::engine::ReportBuilder,
    vm: &ReportViewModel,
    en: bool,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    if vm.severity.has_issues {
        let (findings_title, findings_intro) = if en {
            (
                "Classification of Findings",
                "All technical findings, categorized by systemic template relevance and individual page occurrences.",
            )
        } else {
            (
                "Klassifizierung der Befunde",
                "Alle technischen Befunde, getrennt nach systemischen Komponentenfehlern und Einzelfällen.",
            )
        };
        builder = builder.add_component(PageBreak::new()).add_component(
            SectionHeaderSplit::new(findings_title, findings_intro)
                .with_eyebrow(if en { "FINDINGS" } else { "BEFUNDE" })
                .with_level(2),
        );

        let all_findings = &vm.findings.all_findings;

        let systemic_mandatory: Vec<&FindingGroup> = all_findings
            .iter()
            .filter(|f| f.is_component_issue && f.criticality_tier == CriticalityTier::Mandatory)
            .collect();

        let systemic_optimization: Vec<&FindingGroup> = all_findings
            .iter()
            .filter(|f| f.is_component_issue && f.criticality_tier == CriticalityTier::Optimization)
            .collect();

        let local_mandatory: Vec<&FindingGroup> = all_findings
            .iter()
            .filter(|f| !f.is_component_issue && f.criticality_tier == CriticalityTier::Mandatory)
            .collect();

        let local_optimization: Vec<&FindingGroup> = all_findings
            .iter()
            .filter(|f| {
                !f.is_component_issue && f.criticality_tier == CriticalityTier::Optimization
            })
            .collect();

        // 1. Systemic Mandatory
        if !systemic_mandatory.is_empty() {
            let title = if en {
                "Systemic Template & Component Issues (WCAG A/AA)"
            } else {
                "Systemische Template- & Komponentenfehler (WCAG A/AA)"
            };
            let desc = if en {
                "These findings affect recurring templates or component structures. A central template fix will automatically resolve these issues across all pages."
            } else {
                "Diese Befunde betreffen wiederkehrende Template- oder Komponentenfehler. Eine zentrale Behebung in der Vorlage behebt die Fehler auf allen betroffenen Seiten gleichzeitig."
            };
            builder = builder.add_component(
                SectionHeaderSplit::new(title, desc)
                    .with_eyebrow(if en {
                        "SYSTEMIC COMPLIANCE"
                    } else {
                        "SYSTEMISCHE PFLICHT"
                    })
                    .with_level(2),
            );
            for group in systemic_mandatory {
                builder = render_finding_technical(builder, group, i18n);
            }
        }

        // 2. Systemic Optimization
        if !systemic_optimization.is_empty() {
            let title = if en {
                "Systemic Quality & SEO Optimizations"
            } else {
                "Systemische Qualitäts- & SEO-Optimierungen"
            };
            let desc = if en {
                "Recurring quality and search engine discoverability recommendations at the template level."
            } else {
                "Wiederkehrende Empfehlungen zur Qualitätsverbesserung und Suchmaschinen-Auffindbarkeit im Template."
            };
            builder = builder.add_component(PageBreak::new()).add_component(
                SectionHeaderSplit::new(title, desc)
                    .with_eyebrow(if en {
                        "SYSTEMIC OPTIMIZATION"
                    } else {
                        "SYSTEMISCHE OPTIMIERUNG"
                    })
                    .with_level(2),
            );
            for group in systemic_optimization {
                builder = render_finding_technical(builder, group, i18n);
            }
        }

        // 3. Local Mandatory
        if !local_mandatory.is_empty() {
            let title = if en {
                "Local & Editorial Findings (WCAG A/AA)"
            } else {
                "Einzelfälle & Redaktionelle Befunde (WCAG A/AA)"
            };
            let desc = if en {
                "Single instances of accessibility barriers affecting specific pages, images, or editorial content, usually requiring individual resolution."
            } else {
                "Punktuelle Barrieren, die nur einzelne Seiten, spezifische Bilder oder redaktionelle Texte betreffen und meist individuell behoben werden müssen."
            };
            builder = builder.add_component(PageBreak::new()).add_component(
                SectionHeaderSplit::new(title, desc)
                    .with_eyebrow(if en {
                        "LOCAL COMPLIANCE"
                    } else {
                        "LOKALE PFLICHT"
                    })
                    .with_level(2),
            );
            for group in local_mandatory {
                builder = render_finding_technical(builder, group, i18n);
            }
        }

        // 4. Local Optimization
        if !local_optimization.is_empty() {
            let title = if en {
                "Additional Quality & SEO Recommendations"
            } else {
                "Ergänzende Qualitäts- & SEO-Empfehlungen"
            };
            let desc = if en {
                "Usability, performance, or SEO recommendations for specific pages."
            } else {
                "Ergänzende Empfehlungen zur Verbesserung der Ladezeiten, Suchmaschinenoptimierung und Benutzerfreundlichkeit auf bestimmten Seiten."
            };
            builder = builder.add_component(PageBreak::new()).add_component(
                SectionHeaderSplit::new(title, desc)
                    .with_eyebrow(if en {
                        "LOCAL OPTIMIZATION"
                    } else {
                        "LOKALE OPTIMIERUNG"
                    })
                    .with_level(2),
            );
            for group in local_optimization {
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

fn render_positive_signals_section(
    mut builder: renderreport::engine::ReportBuilder,
    signals: &PositiveSignalsBlock,
    is_first: bool,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    if signals.is_empty() {
        return builder;
    }

    if !is_first {
        builder = builder.add_component(PageBreak::new());
    }

    let en = i18n.locale() == "en";
    let mut rows = Vec::new();
    for signal in &signals.items {
        let status = if signal.strong {
            if en {
                "Strong"
            } else {
                "Stark"
            }
        } else if en {
            "Present"
        } else {
            "Vorhanden"
        };
        rows.push(
            ChecklistRow::new(&signal.title, format!("{}: {}", status, signal.description))
                .with_status(if signal.strong { "good" } else { "info" }),
        );
    }

    builder.add_component(ChecklistPanel::new(rows).with_title(if en {
        "Recognized structural patterns"
    } else {
        "Erkannte Strukturmuster"
    }))
}

fn render_dual_viewport_summary_section(
    mut builder: renderreport::engine::ReportBuilder,
    vm: &ReportViewModel,
    report: &AuditReport,
    is_first: bool,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let has_scores = vm.cover.desktop_score.is_some() && vm.cover.mobile_score.is_some();
    let has_screenshot_status = report.page_screenshots.is_some()
        || matches!(
            report.screenshot_status,
            crate::audit::ScreenshotStatus::Failed(_)
        );
    if !has_scores && !has_screenshot_status {
        return builder;
    }

    if !is_first {
        builder = builder.add_component(PageBreak::new());
    }

    let en = i18n.locale() == "en";
    if report.page_screenshots.is_some() {
        builder = builder.add_component(
            DevicePreview::new(
                super::PAGE_DESKTOP_SCREENSHOT_ASSET,
                super::PAGE_MOBILE_SCREENSHOT_ASSET,
            )
            .with_height(210.0),
        );
    }

    let mut rows = Vec::new();
    if let (Some(desktop), Some(mobile)) = (vm.cover.desktop_score, vm.cover.mobile_score) {
        rows.push(
            ChecklistRow::new(
                if en {
                    "Desktop viewport"
                } else {
                    "Desktop-Viewport"
                },
                format!("{desktop}/100"),
            )
            .with_status(if desktop >= 75 { "good" } else { "warn" }),
        );
        rows.push(
            ChecklistRow::new(
                if en {
                    "Mobile viewport"
                } else {
                    "Mobile-Viewport"
                },
                format!("{mobile}/100"),
            )
            .with_status(if mobile >= 75 { "good" } else { "warn" }),
        );
    }

    let screenshot_status = match &report.screenshot_status {
        crate::audit::ScreenshotStatus::Captured => Some(
            if en {
                "Screenshots captured"
            } else {
                "Screenshots erfasst"
            }
            .to_string(),
        ),
        crate::audit::ScreenshotStatus::Failed(reason) => Some(if en {
            format!("Screenshot capture failed: {reason}")
        } else {
            format!("Screenshot-Erfassung fehlgeschlagen: {reason}")
        }),
        crate::audit::ScreenshotStatus::NotRequested => None,
    };
    if let Some(status) = screenshot_status {
        rows.push(
            ChecklistRow::new(if en { "Preview" } else { "Vorschau" }, status).with_status(
                if report.page_screenshots.is_some() {
                    "good"
                } else {
                    "warn"
                },
            ),
        );
    }

    rows.push(
        ChecklistRow::new(
            if en { "Interpretation" } else { "Einordnung" },
            if en {
                "JSON detail includes a compact dual_viewport summary with per-viewport finding counts and module availability."
            } else {
                "Das JSON-Detail enthält eine kompakte dual_viewport-Zusammenfassung mit Befundzahlen und Modulverfügbarkeit je Viewport."
            },
        )
        .with_status("info"),
    );

    builder.add_component(ChecklistPanel::new(rows).with_title(if en {
        "Dual viewport summary"
    } else {
        "Dual-Viewport-Zusammenfassung"
    }))
}

pub(super) fn render_module_sections(
    mut builder: renderreport::engine::ReportBuilder,
    vm: &ReportViewModel,
    report: &AuditReport,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let mut is_first = true;
    let has_dual_viewport_scores =
        vm.cover.desktop_score.is_some() && vm.cover.mobile_score.is_some();
    let has_screenshot_status = report.page_screenshots.is_some()
        || matches!(
            report.screenshot_status,
            crate::audit::ScreenshotStatus::Failed(_)
        );
    if has_dual_viewport_scores || has_screenshot_status {
        builder = render_dual_viewport_summary_section(builder, vm, report, is_first, i18n);
        is_first = false;
    }
    if let Some(ref sx) = vm.module_details.search_experience {
        builder = render_search_experience(builder, sx, is_first, i18n);
        is_first = false;
    }

    for module in active_report_modules(report) {
        let (next_builder, rendered) =
            render_active_module_section(builder, module.module_key(), vm, report, is_first, i18n);
        builder = next_builder;
        if rendered {
            is_first = false;
        }
    }

    builder
}

fn render_active_module_section(
    mut builder: renderreport::engine::ReportBuilder,
    module_key: &str,
    vm: &ReportViewModel,
    report: &AuditReport,
    is_first: bool,
    i18n: &I18n,
) -> (renderreport::engine::ReportBuilder, bool) {
    match module_key {
        "performance" => {
            if let Some(ref perf) = vm.module_details.performance {
                builder = render_performance(builder, perf, is_first, i18n);
                if !report.budget_violations.is_empty() {
                    builder = render_budget_violations(builder, &report.budget_violations, i18n);
                }
                return (builder, true);
            }
        }
        "seo" => {
            if let Some(ref seo) = vm.module_details.seo {
                return (render_seo(builder, seo, is_first, i18n), true);
            }
        }
        "security" => {
            if let Some(ref sec) = vm.module_details.security {
                return (render_security(builder, sec, is_first, i18n), true);
            }
        }
        "mobile" => {
            if let Some(ref mobile) = vm.module_details.mobile {
                return (render_mobile(builder, mobile, is_first, i18n), true);
            }
        }
        "ux" => {
            if let Some(ref ux) = vm.module_details.ux {
                return (render_ux(builder, ux, is_first, i18n), true);
            }
        }
        "journey" => {
            if let Some(ref journey) = vm.module_details.journey {
                return (render_journey(builder, journey, is_first, i18n), true);
            }
        }
        "dark_mode" => {
            if let Some(ref dm) = vm.module_details.dark_mode {
                return (render_dark_mode(builder, dm, is_first, i18n), true);
            }
        }
        "source_quality" => {
            if let Some(ref sq) = vm.module_details.source_quality {
                return (render_source_quality(builder, sq, is_first, i18n), true);
            }
        }
        "ai_visibility" => {
            if let Some(ref av) = vm.module_details.ai_visibility {
                return (render_ai_visibility(builder, av, is_first, i18n), true);
            }
        }
        "content_visibility" => {
            if let Some(ref cv) = vm.module_details.content_visibility {
                return (render_content_visibility(builder, cv, is_first, i18n), true);
            }
        }
        "best_practices" => {
            if let Some(ref bp) = vm.module_details.best_practices {
                return (render_best_practices(builder, bp, is_first, i18n), true);
            }
        }
        "tech_stack" => {
            if let Some(ref ts) = vm.module_details.tech_stack {
                return (render_tech_stack(builder, ts, is_first, i18n), true);
            }
        }
        "patterns" => {
            if !vm.positive_signals.is_empty() {
                return (
                    render_positive_signals_section(builder, &vm.positive_signals, is_first, i18n),
                    true,
                );
            }
        }
        _ => {}
    }

    (builder, false)
}

pub(super) fn render_root_cause_analysis(
    mut builder: renderreport::engine::ReportBuilder,
    vm: &ReportViewModel,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = i18n.locale() == "en";
    let title = if en {
        "Root Cause Analysis"
    } else {
        "Ursachenanalyse"
    };
    let subtitle = if en {
        "Grouping individual findings into systemic component/template root causes."
    } else {
        "Bündelung der Einzelfunde in systemische Komponenten- und Template-Ursachen."
    };

    // No leading PageBreak: the preceding part divider ("Befunde nach Ursache")
    // already opened the page — an extra break left that divider page 3/4 empty.
    builder = builder.add_component(
        SectionHeaderSplit::new(title, subtitle)
            .with_eyebrow(if en { "ROOT CAUSE" } else { "URSACHENANALYSE" })
            .with_level(2),
    );

    // Let's filter the accessibility findings from all_findings, sorted by occurrence count desc
    let mut findings: Vec<&FindingGroup> = vm
        .findings
        .all_findings
        .iter()
        .filter(|f| f.occurrence_count > 0)
        .collect();
    findings.sort_by_key(|f| std::cmp::Reverse(f.occurrence_count));

    if findings.is_empty() {
        let msg = if en {
            "No accessibility findings were detected, so no root cause analysis is necessary."
        } else {
            "Es wurden keine Barrierefreiheits-Befunde erkannt, daher ist keine Ursachenanalyse erforderlich."
        };
        builder = builder.add_component(Callout::success(msg));
        return builder;
    }

    // Render list of component errors (assigning letters A, B, C, etc.)
    let mut list = List::new().with_title(if en {
        "Systemic Root Causes"
    } else {
        "Erkannte Kernursachen (Templates)"
    });
    let mut table_rows = Vec::new();
    let total_occurrences: usize = findings.iter().map(|f| f.occurrence_count).sum();

    for (idx, finding) in findings.iter().enumerate().take(6) {
        let letter = (b'A' + idx as u8) as char;
        let item_title = format!(
            "{} {}: {} Vorkommen — {}",
            if en {
                "Component Issue"
            } else {
                "Komponentenfehler"
            },
            letter,
            finding.occurrence_count,
            finding.title
        );
        list = list.add_item(&item_title);

        let share_pct = if total_occurrences > 0 {
            (finding.occurrence_count * 100)
                .checked_div(total_occurrences)
                .unwrap_or(0)
        } else {
            0
        };

        table_rows.push(vec![
            format!("{} {}", if en { "Component" } else { "Ursache" }, letter),
            finding.occurrence_count.to_string(),
            format!("{} %", share_pct),
        ]);
    }

    builder = builder.add_component(list);

    // Render the table: Ursache | Vorkommen | Anteil
    let mut table = AuditTable::new(vec![
        TableColumn::new(if en { "Root Cause" } else { "Ursache" }).with_width("40%"),
        TableColumn::new(if en { "Occurrences" } else { "Vorkommen" }).with_width("30%"),
        TableColumn::new(if en { "Share" } else { "Anteil" }).with_width("30%"),
    ])
    .with_title(if en {
        "Distribution of Issues by Root Cause"
    } else {
        "Verteilung der Mängel nach Ursache"
    });

    for row in table_rows {
        table = table.add_row(row);
    }
    builder = builder.add_component(table);

    builder
}

pub(super) fn render_timeframe_roadmap(
    mut builder: renderreport::engine::ReportBuilder,
    vm: &ReportViewModel,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = i18n.locale() == "en";
    let title = if en { "Action Plan" } else { "Maßnahmenplan" };
    let subtitle = if en {
        "Prioritized roadmap structured by implementation timeframe."
    } else {
        "Priorisierter Ablaufplan strukturiert nach Umsetzungs-Zeithorizont."
    };

    builder = builder.add_component(PageBreak::new()).add_component(
        SectionHeaderSplit::new(title, subtitle)
            .with_eyebrow(if en { "ROADMAP" } else { "MASSNAHMENPLAN" })
            .with_level(2),
    );

    let timeframes = if en {
        vec![
            (
                "Priority 1: Critical Barriers",
                "Acute barriers — highest risk, must be resolved before anything else.",
            ),
            (
                "Priority 2: Usability & Compliance",
                "Significant barriers with direct usability impact.",
            ),
            (
                "Priority 3: Structural Optimizations",
                "Quality improvements with moderate accessibility benefit.",
            ),
        ]
    } else {
        vec![
            (
                "Priorität 1: Kritische Hürden",
                "Akute Barrieren — vor allen anderen Punkten beheben.",
            ),
            (
                "Priorität 2: Nutzbarkeit & Konformität",
                "Relevante Barrieren mit direktem Impact auf Nutzbarkeit.",
            ),
            (
                "Priorität 3: Strukturelle Optimierungen",
                "Qualitätsverbesserungen mit moderatem Barrierefreiheits-Nutzen.",
            ),
        ]
    };

    for (idx, (header, desc)) in timeframes.into_iter().enumerate() {
        let mut column_items: &[RoadmapItemData] = &[];
        if let Some(col) = vm.actions.roadmap_columns.get(idx) {
            column_items = &col.items;
        }

        if column_items.is_empty() {
            let search_term = match idx {
                0 => {
                    if en {
                        "priority 1"
                    } else {
                        "priorität 1"
                    }
                }
                1 => {
                    if en {
                        "priority 2"
                    } else {
                        "priorität 2"
                    }
                }
                _ => {
                    if en {
                        "priority 3"
                    } else {
                        "priorität 3"
                    }
                }
            };
            if let Some(col) = vm
                .actions
                .roadmap_columns
                .iter()
                .find(|c| c.title.to_lowercase().contains(search_term))
            {
                column_items = &col.items;
            }
        }

        builder = builder.add_component(SectionHeaderSplit::new(header, desc).with_level(3));

        if column_items.is_empty() {
            let empty_msg = if en {
                "No recommendations for this timeframe."
            } else {
                "Keine Maßnahmen für diesen Zeitraum empfohlen."
            };
            builder = builder.add_component(Label::new(empty_msg).with_color("#475569"));
        } else {
            let mut table = AuditTable::new(vec![
                TableColumn::new(if en { "Action" } else { "Maßnahme" }).with_width("40%"),
                TableColumn::new(if en { "Complexity" } else { "Komplexität" }).with_width("20%"),
                TableColumn::new(if en { "Responsible" } else { "Zuständig" }).with_width("20%"),
                TableColumn::new(if en {
                    "Risk Reduction"
                } else {
                    "Risiko-Reduktion"
                })
                .with_width("20%"),
            ]);
            for item in column_items {
                table = table.add_row(vec![
                    item.action.clone(),
                    item.effort.clone(),
                    item.role.clone(),
                    item.risk_effect.clone(),
                ]);
            }
            builder = builder.add_component(table);
        }
    }

    builder
}
