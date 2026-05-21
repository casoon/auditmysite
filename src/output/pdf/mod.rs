//! PDF Report Generator using renderreport/Typst
//!
//! Pure layout layer — receives pre-computed ViewModel blocks and maps them
//! directly to renderreport components. Zero data transformation here.

mod appendix;
mod batch;
mod batch_report;
mod comparison_report;
mod cover;
mod design;
mod detail_modules;
mod diagnosis;
mod findings;
mod helpers;
mod history;
mod modules;
mod single_report;
mod wcag_coverage;

pub use self::batch_report::{generate_batch_pdf, generate_batch_typ};
pub use self::comparison_report::generate_comparison_pdf;

use renderreport::components::advanced::{
    ChecklistPanel, ChecklistRow, DevicePreview, DominantIssueSpotlight, ImpactGrid,
    ImpactGridCard, List, MetricStrip, MetricStripItem, PageBreak, SectionHeaderSplit,
    TableOfContents,
};
use renderreport::components::text::{Label, TextBlock};
use renderreport::prelude::Image;
use renderreport::prelude::*;

use renderreport::components::SeverityOverview;

use self::appendix::{build_module_strip, build_raw_audit_snapshot, impact_row, risk_status};
// Re-export helpers used by sibling sub-modules via `super::`.
use self::appendix::{cover_logo_asset, register_cover_logo_asset};
use self::cover::{build_cover_score_row, certificate_badge_path};
use self::diagnosis::{business_relevance, format_word_count, output_scope_callout};
use self::findings::render_key_finding_block;
use self::helpers::{component_json, create_engine, extract_domain, soft_flow_group};
use self::history::{render_history_section, render_methodology_section};
use self::modules::{build_summary_overview, render_next_steps_single};
use self::single_report::{
    render_action_plan, render_part_divider, render_tech_details, render_tech_entry,
};
use crate::audit::{normalize, AuditReport};
use crate::cli::ReportLevel;
use crate::i18n::I18n;
use crate::output::builder::build_view_model;
use crate::output::report_model::*;

const WORDMARK_ASSET: &str = "/auditmysite-wordmark.svg";
const CUSTOM_COVER_LOGO_ASSET: &str = "/cover-logo-custom";

// ─── Single Report ──────────────────────────────────────────────────────────
//
// 6-page structure:
//   Page 1 — Hero / Entry (pitch: status, scores, impact, consequences)
//   Page 2 — Dominant Issue (focus: biggest problem, top fixes, leverage)
//   Page 3 — Key Findings (bridge: Problem/Impact/Ursache/Fix cards)
//   Page 4 — Action Plan (decide: quick wins, action table, execution note)
//   Page 5 — Tech Entry (transition: intro + severity overview)
//   Page 6+ — Tech Details (implement: WCAG details, code examples, modules)

pub fn generate_pdf(report: &AuditReport, config: &ReportConfig) -> anyhow::Result<Vec<u8>> {
    let (engine, built_report) = build_single_report(report, config)?;
    let pdf_bytes = engine.render_pdf(&built_report)?;
    cleanup_screenshot_temps(report);
    Ok(pdf_bytes)
}

/// Render the intermediate Typst source for a single-page report.
///
/// Useful for the hidden `--debug-typ` mode and for snapshot/template-regression
/// checks (issue #239) — lets reviewers inspect completeness and wording without
/// going through the heavy PDF + pdftotext pipeline.
pub fn generate_typ(report: &AuditReport, config: &ReportConfig) -> anyhow::Result<String> {
    let (engine, built_report) = build_single_report(report, config)?;
    let typ = engine.render_typ(&built_report)?;
    cleanup_screenshot_temps(report);
    Ok(typ)
}

fn build_single_report(
    report: &AuditReport,
    config: &ReportConfig,
) -> anyhow::Result<(renderreport::Engine, renderreport::RenderRequest)> {
    let engine = create_engine()?;
    let normalized = normalize(report);
    let vm = build_view_model(&normalized, config);
    let i18n = I18n::new(&config.locale)?;

    let mut builder = engine
        .report("wcag-audit")
        .metadata("date", &vm.meta.date)
        .metadata("version", &vm.meta.version)
        .metadata("author", &vm.meta.author)
        .metadata("score", &vm.meta.score_label)
        .metadata("footer_prefix", "Audit:")
        .metadata("footer_link_url", "")
        .metadata(
            "footer_tagline",
            "A technical auditing platform by casoon.de",
        );

    let cover_logo_asset = self::appendix::cover_logo_asset(config);
    builder = self::appendix::register_cover_logo_asset(builder, config, cover_logo_asset);

    // ── Cover Page ───────────────────────────────────────────────────
    builder = builder
        .add_component(Image::new(cover_logo_asset).with_width("120pt"))
        .add_component(
            Label::new(&vm.executive.cover_eyebrow)
                .with_size("10pt")
                .bold()
                .with_color("#0f766e"),
        )
        .add_component(Label::new(&vm.cover.title).with_size("22pt").bold())
        .add_component(
            Label::new(format!(
                "{}  ·  {}  ·  auditmysite v{}",
                extract_domain(&vm.cover.domain),
                vm.cover.date,
                vm.meta.version
            ))
            .with_size("11pt")
            .with_color("#0f766e"),
        )
        .add_component(
            Label::new(&vm.executive.cover_kicker)
                .with_size("12pt")
                .with_color("#475569"),
        );

    let single_badge_asset = "/certificate-badge-single.svg";
    let single_badge_enabled = if let Ok(path) = certificate_badge_path(&vm.cover.certificate) {
        builder = builder.asset(single_badge_asset, path);
        true
    } else {
        false
    };

    // ── Device Preview (desktop + mobile screenshots) — skip for executive
    if vm.meta.report_level != ReportLevel::Executive {
        if let Some(ref shots) = report.page_screenshots {
            let ts = report.timestamp.timestamp_nanos_opt().unwrap_or(0);
            let desktop_path = std::env::temp_dir().join(format!("ams-desktop-{}.png", ts));
            let mobile_path = std::env::temp_dir().join(format!("ams-mobile-{}.png", ts));
            if std::fs::write(&desktop_path, &shots.desktop).is_ok()
                && std::fs::write(&mobile_path, &shots.mobile).is_ok()
            {
                let desktop_key = "/page-screenshot-desktop.png";
                let mobile_key = "/page-screenshot-mobile.png";
                builder = builder
                    .asset(desktop_key, &desktop_path)
                    .asset(mobile_key, &mobile_path)
                    .add_component(DevicePreview::new(desktop_key, mobile_key));
            }
        } else {
            // Differentiate between explicit failure and not-requested (issue #26).
            let title = i18n.t("section-device-preview");
            let en = i18n.locale() == "en";
            let body = match &report.screenshot_status {
                crate::audit::ScreenshotStatus::Failed(reason) => {
                    if en {
                        format!("Screenshots could not be captured: {reason}.")
                    } else {
                        format!("Screenshots konnten nicht erstellt werden: {reason}.")
                    }
                }
                crate::audit::ScreenshotStatus::NotRequested => {
                    if en {
                        "Screenshots were not captured for this audit (batch mode or screenshot capture disabled)."
                            .to_string()
                    } else {
                        "Screenshots wurden für dieses Audit nicht erfasst (Batch-Modus oder Screenshot-Erfassung deaktiviert)."
                            .to_string()
                    }
                }
                crate::audit::ScreenshotStatus::Captured => {
                    i18n.t("section-device-preview-no-screenshots")
                }
            };
            builder = builder.add_component(Callout::info(&body).with_title(&title));
        }
    }

    builder = builder.add_component(build_cover_score_row(
        &vm.cover,
        single_badge_enabled.then_some(single_badge_asset),
        &i18n,
    ));

    if !vm.summary.executive_lead.is_empty() {
        builder = builder.add_component(
            Label::new(&vm.summary.executive_lead)
                .with_size("13pt")
                .bold()
                .with_color("#0f4c42"),
        );
    }

    builder = builder
        .add_component(
            TextBlock::new(&vm.summary.verdict)
                .with_size("11pt")
                .with_line_height("1.4em")
                .with_max_width("100%"),
        )
        .add_component(output_scope_callout(&i18n))
        .add_component(PageBreak::new());

    if vm.meta.report_level != ReportLevel::Executive {
        builder = builder.add_component(TableOfContents::new());
        // Part 1 — Executive Summary (#246).
        let en = i18n.locale() == "en";
        let (title, intro, audience_title, audience_body, contents_title) = if en {
            (
                "Executive Summary",
                "Risk classification, top issues, business impact, and recommended next steps — at a glance.",
                "Audience",
                "Decision makers and stakeholders. Read this part to understand the website's accessibility risk and the business consequences without diving into technical detail.",
                "What's in this part",
            )
        } else {
            (
                "Executive Summary",
                "Risikoeinstufung, Top-Probleme, Geschäftsauswirkungen und empfohlene nächste Schritte — auf einen Blick.",
                "Zielgruppe",
                "Entscheider und Stakeholder. Dieser Teil zeigt das Risiko der Website und die geschäftlichen Konsequenzen, ohne in technische Details einzusteigen.",
                "Inhalt dieses Teils",
            )
        };
        let contents: Vec<&str> = if en {
            vec![
                "Risk classification (BFSG relevance)",
                "Top 5 critical and high-severity issues",
                "Business impact and consequences",
                "Effort estimation (quick wins, mid-term, complex)",
                "Recommended next steps",
            ]
        } else {
            vec![
                "Risikoeinstufung (BFSG-Relevanz)",
                "Top 5 kritische und hohe Probleme",
                "Geschäftliche Auswirkungen und Konsequenzen",
                "Aufwandsschätzung (Quick Wins, mittelfristig, komplex)",
                "Empfohlene nächste Schritte",
            ]
        };
        builder = render_part_divider(
            builder,
            1,
            title,
            intro,
            audience_title,
            audience_body,
            contents_title,
            &contents,
            &i18n,
        );
    }

    // ─────────────────────────────────────────────────────────────────
    // SECTION 1 — STATUS & BEWERTUNG
    // ─────────────────────────────────────────────────────────────────
    {
        let risk_callout = match vm.summary.risk_level.as_str() {
            "Kritisch" | "Critical" => {
                Callout::error(&vm.summary.risk_summary).with_title(&vm.executive.risk_title)
            }
            "Hoch" | "High" => {
                Callout::warning(&vm.summary.risk_summary).with_title(&vm.executive.risk_title)
            }
            _ => Callout::info(&vm.summary.risk_summary).with_title(&vm.executive.risk_title),
        };

        builder = builder.add_component(soft_flow_group(
            "180pt",
            vec![
                component_json(
                    SectionHeaderSplit::new(
                        &vm.executive.status_title,
                        if i18n.locale() == "en" {
                            "Classification, risk and the most important business consequences at a glance."
                        } else {
                            "Einordnung, Risiko und die geschäftlich wichtigsten Konsequenzen auf einen Blick."
                        },
                    )
                    .with_eyebrow("EXECUTIVE SNAPSHOT")
                    .with_level(2)
                ),
                component_json(risk_callout),
            ],
        ));

        builder = builder.add_component(soft_flow_group(
            "200pt",
            vec![
                component_json(build_summary_overview(&vm.summary)),
                component_json(
                    MetricStrip::new(vec![
                        MetricStripItem::new(
                            i18n.t("metric-score"),
                            vm.summary.overall_score.to_string(),
                        )
                        .with_accent("#0f766e"),
                        MetricStripItem::new(
                            i18n.t("metric-issues-detected"),
                            vm.severity.total.to_string(),
                        ),
                        MetricStripItem::new(
                            i18n.t("metric-critical-high"),
                            format!("{}", vm.severity.critical + vm.severity.high),
                        )
                        .with_status("bad")
                        .with_accent("#dc2626"),
                        MetricStripItem::new(i18n.t("metric-risk"), &vm.summary.risk_level)
                            .with_status(risk_status(&vm.summary.risk_level)),
                        MetricStripItem::new(i18n.t("metric-certificate"), &vm.summary.certificate)
                            .with_accent("#7c3aed"),
                    ])
                    .compact(),
                ),
            ],
        ));

        if vm.meta.report_level != ReportLevel::Executive {
            builder = builder.add_component(build_module_strip(&vm, &i18n));
            builder = builder.add_component(build_raw_audit_snapshot(&vm, &i18n));
            // Three-perspective scope blocks (#231)
            {
                let en = i18n.locale() == "en";
                let user_impact = impact_row(
                    &vm.executive.impact_rows,
                    if en { "User" } else { "Nutzer" },
                );
                let risk_text = impact_row(
                    &vm.executive.impact_rows,
                    if en { "Risk" } else { "Risiko" },
                );
                let technical_text = if en {
                    "Automated static analysis of the accessibility tree, DOM structure, and resource loading. Results reflect detectable patterns; manual verification required for context-dependent issues."
                } else {
                    "Automatisierte statische Analyse des Accessibility-Trees, der DOM-Struktur und des Ressourcen-Ladens. Ergebnisse spiegeln erkennbare Muster wider; kontextabhängige Probleme erfordern manuelle Prüfung."
                };
                if !user_impact.is_empty() {
                    builder =
                        builder.add_component(Callout::warning(user_impact).with_title(if en {
                            "User impact"
                        } else {
                            "Auswirkungen auf Nutzer"
                        }));
                }
                if !risk_text.is_empty() {
                    builder = builder.add_component(Callout::error(risk_text).with_title(if en {
                        "Legal risk"
                    } else {
                        "Rechtliches Risiko"
                    }));
                }
                builder = builder.add_component(Callout::info(technical_text).with_title(if en {
                    "Technical basis"
                } else {
                    "Technische Grundlage"
                }));
            }
        }

        {
            let mut kp_list = List::new().with_title(&vm.executive.key_points_title);
            for point in &vm.executive.key_points {
                kp_list = kp_list.add_item(point);
            }
            builder = builder.add_component(kp_list);
        }

        {
            let en = i18n.locale() == "en";
            let (user_key, business_key, risk_key) = if en {
                ("User", "Business", "Risk")
            } else {
                ("Nutzer", "Business", "Risiko")
            };
            let (usability_label, compliance_label, business_eff_label) = if en {
                ("Usability", "Compliance", "Business effect")
            } else {
                ("Nutzbarkeit", "Compliance", "Geschäftswirkung")
            };
            let user = impact_row(&vm.executive.impact_rows, user_key);
            let business = impact_row(&vm.executive.impact_rows, business_key);
            let risk = impact_row(&vm.executive.impact_rows, risk_key);
            builder = builder.add_component(
                ImpactGrid::new(
                    ImpactGridCard::new(user_key, usability_label, user).with_status("warn"),
                    ImpactGridCard::new(risk_key, compliance_label, risk).with_status("bad"),
                    ImpactGridCard::new(business_key, business_eff_label, business)
                        .with_status("info"),
                )
                .with_title(&vm.executive.impact_title),
            );
        }

        if !vm.executive.quick_actions.is_empty() {
            let rows: Vec<ChecklistRow> = vm
                .executive
                .quick_actions
                .iter()
                .map(|action| ChecklistRow::new(action, "").with_status("warn"))
                .collect();
            builder = builder.add_component(
                ChecklistPanel::new(rows).with_title(&vm.executive.quick_actions_title),
            );
        }

        if vm.meta.report_level != ReportLevel::Executive && !vm.summary.positive_aspects.is_empty()
        {
            let strength_label = i18n.t("label-strength");
            let rows: Vec<ChecklistRow> = vm
                .summary
                .positive_aspects
                .iter()
                .take(3)
                .enumerate()
                .map(|(i, item)| {
                    ChecklistRow::new(format!("{} {}", strength_label, i + 1), item)
                        .with_status("good")
                })
                .collect();
            builder = builder
                .add_component(ChecklistPanel::new(rows).with_title(i18n.t("panel-strengths")));
        }

        if vm.meta.report_level != ReportLevel::Executive && !vm.positive_signals.is_empty() {
            let title = if vm.meta.report_level == ReportLevel::Standard && i18n.locale() == "en" {
                "Recognized patterns"
            } else if i18n.locale() == "en" {
                "Recognized structural patterns"
            } else {
                "Erkannte strukturelle Patterns"
            };
            let rows: Vec<ChecklistRow> = vm
                .positive_signals
                .items
                .iter()
                .map(|signal| {
                    ChecklistRow::new(&signal.title, &signal.description)
                        .with_status(if signal.strong { "good" } else { "warning" })
                })
                .collect();
            builder = builder.add_component(ChecklistPanel::new(rows).with_title(title));
        }
    }

    // ─────────────────────────────────────────────────────────────────
    // SECTION 2 — DOMINANT ISSUE
    // ─────────────────────────────────────────────────────────────────
    {
        let total_ch = (vm.severity.critical + vm.severity.high) as usize;

        if let Some(top) = vm.findings.top_findings.first() {
            let total_occurrences: usize = vm
                .findings
                .top_findings
                .iter()
                .map(|f| f.occurrence_count)
                .sum();
            let share = if let Some(v) = (top.occurrence_count * 100).checked_div(total_occurrences)
            {
                v.max(1)
            } else if let Some(v) = (top.occurrence_count * 100).checked_div(total_ch) {
                v.max(1)
            } else {
                100
            };
            let spotlight = DominantIssueSpotlight::new(
                &top.title,
                format!("{:?}", top.severity).to_lowercase(),
                &vm.executive.spotlight_body,
                &vm.executive.spotlight_impact,
                &vm.executive.spotlight_recommendation,
            )
            .with_eyebrow(&vm.executive.spotlight_eyebrow)
            .with_affected_count(share as u32);
            builder = builder.add_component(spotlight);
        }

        if vm.severity.has_issues {
            builder = builder.add_component(
                SeverityOverview::new(
                    vm.severity.critical,
                    vm.severity.high,
                    vm.severity.medium,
                    vm.severity.low,
                )
                .with_title(&i18n.t("section-issue-overview")),
            );

            if vm.severity.component_issues > 0 {
                let en = i18n.locale() == "en";
                let ci = vm.severity.component_issues;
                let co = vm.severity.component_occurrences;
                let msg = if en {
                    format!(
                        "{} of the {} occurrences above stem from {} component issue(s) \
                         in shared templates. Each component fix resolves all its occurrences \
                         at once — prioritize these first.",
                        co, vm.severity.total, ci,
                    )
                } else {
                    format!(
                        "{} der {} Vorkommen oben stammen aus {} Komponentenproblem(en) \
                         in gemeinsamen Templates. Jeder Komponenten-Fix behebt alle \
                         seine Vorkommen auf einmal — diese zuerst priorisieren.",
                        co, vm.severity.total, ci,
                    )
                };
                let label = if en {
                    "Component Issues"
                } else {
                    "Komponentenprobleme"
                };
                builder = builder.add_component(Callout::warning(&msg).with_title(label));
            }
        }

        if total_ch > 0 {
            if let Some(leverage_text) = &vm.executive.leverage_text {
                builder = builder.add_component(
                    Callout::success(leverage_text).with_title(&vm.executive.leverage_title),
                );
            }
        }

        let findings_limit = if vm.meta.report_level == ReportLevel::Executive {
            3
        } else {
            5
        };
        for group in vm.findings.top_findings.iter().take(findings_limit) {
            builder = render_key_finding_block(builder, group, &i18n, false);
        }
    }

    if vm.meta.report_level != ReportLevel::Executive {
        if let Some(ref history) = vm.history {
            builder = render_history_section(builder, history, &i18n);
        }
    }

    // Executive level stops here — slim methodology (limitations only)
    if vm.meta.report_level == ReportLevel::Executive {
        builder = builder
            .add_component(
                Callout::info(&vm.methodology.limitations)
                    .with_title(i18n.t("callout-limitations-title")),
            )
            .add_component(
                Callout::warning(&vm.methodology.disclaimer)
                    .with_title(i18n.t("callout-note-title")),
            );
        let built_report = builder.build();
        return Ok((engine, built_report));
    }

    // ─────────────────────────────────────────────────────────────────
    // SECTIONS 4 + 5 + 5b + 6+ (Standard / Technical only)
    // ─────────────────────────────────────────────────────────────────
    builder = render_action_plan(builder, &vm, &i18n);
    builder = render_tech_entry(builder, &vm, &i18n);
    builder = render_tech_details(builder, &vm, report, &i18n);

    builder = render_next_steps_single(builder, &vm);
    builder = render_methodology_section(builder, &vm.methodology, &i18n);

    let built_report = builder.build();
    Ok((engine, built_report))
}

fn cleanup_screenshot_temps(report: &AuditReport) {
    if let Some(ref _shots) = report.page_screenshots {
        let ts = report.timestamp.timestamp_nanos_opt().unwrap_or(0);
        let _ = std::fs::remove_file(std::env::temp_dir().join(format!("ams-desktop-{}.png", ts)));
        let _ = std::fs::remove_file(std::env::temp_dir().join(format!("ams-mobile-{}.png", ts)));
    }
}

#[cfg(all(test, feature = "pdf_test"))]
#[path = "tests.rs"]
mod tests;
