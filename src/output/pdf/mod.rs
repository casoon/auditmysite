//! PDF Report Generator using renderreport/Typst
//!
//! Pure layout layer — receives pre-computed ViewModel blocks and maps them
//! directly to renderreport components. Zero data transformation here.

mod appendix;
mod batch;
mod batch_report;
mod cover;
mod design;
mod detail_modules;
mod diagnosis;
mod findings;
mod helpers;
mod modules;
mod single_report;
mod wcag_coverage;

pub use self::batch_report::{generate_batch_pdf, generate_batch_typ};

use renderreport::components::advanced::{
    ChecklistPanel, ChecklistRow, DevicePreview, Divider, DominantIssueSpotlight, ImpactGrid,
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
use self::cover::{build_cover_score_row_gauges, certificate_badge_path};
use self::diagnosis::{business_relevance, format_word_count, output_scope_callout};
use self::findings::render_key_finding_block;
use self::helpers::{component_json, create_engine, extract_domain, soft_flow_group};
use self::modules::{build_summary_overview, render_next_steps_single};
use self::single_report::{
    render_appendix_full, render_management_page, render_module_sections, render_part_divider,
    render_root_cause_analysis, render_tech_details, render_timeframe_roadmap,
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
    let en = i18n.locale() == "en";
    let risk_summary_text = if en {
        format!(
            "Status: {}  ·  {} Total Issues ({} Critical/High)",
            vm.cover.certificate, vm.cover.total_issues, vm.cover.critical_issues
        )
    } else {
        format!(
            "Status: {}  ·  {} Befunde gesamt ({} kritisch/hoch)",
            vm.cover.certificate, vm.cover.total_issues, vm.cover.critical_issues
        )
    };

    builder = builder
        .add_component(Image::new(cover_logo_asset).with_width("120pt"))
        .add_component(
            Label::new(&vm.executive.cover_eyebrow)
                .with_size("10pt")
                .bold()
                .with_color("#0f766e"),
        )
        .add_component(Label::new(&vm.cover.title).with_size("34pt").bold())
        .add_component(
            Label::new(format!(
                "{}  ·  {}",
                extract_domain(&vm.cover.domain),
                vm.cover.date
            ))
            .with_size("14pt")
            .bold()
            .with_color("#0f766e"),
        )
        .add_component(
            Label::new(&risk_summary_text)
                .with_size("11pt")
                .bold()
                .with_color("#475569"),
        )
        .add_component(
            Label::new(&vm.executive.cover_kicker)
                .with_size("12pt")
                .with_color("#475569"),
        );

    builder = builder.add_component(build_cover_score_row_gauges(
        &vm.cover,
        &vm.summary,
        &vm.modules.dashboard,
        &i18n,
    ));

    let dominant_cause_text = if vm.severity.component_issues > 0 {
        let comp_ch_occurrences: usize = vm
            .findings
            .all_findings
            .iter()
            .filter(|f| {
                f.is_component_issue
                    && (f.severity == crate::wcag::Severity::Critical
                        || f.severity == crate::wcag::Severity::High)
            })
            .map(|f| f.occurrence_count)
            .sum();
        let total_ch_occurrences: usize = vm
            .findings
            .all_findings
            .iter()
            .filter(|f| {
                f.severity == crate::wcag::Severity::Critical
                    || f.severity == crate::wcag::Severity::High
            })
            .map(|f| f.occurrence_count)
            .sum();
        let share_pct = if total_ch_occurrences > 0 {
            (comp_ch_occurrences * 100)
                .checked_div(total_ch_occurrences)
                .unwrap_or(0)
        } else {
            0
        };
        let is_en = i18n.locale() == "en";
        if is_en {
            if vm.severity.component_issues == 1 {
                format!(
                    "Main cause: 1 component error causes {}% of all critical/high findings.",
                    share_pct
                )
            } else {
                format!(
                    "Main cause: {} component errors cause {}% of all critical/high findings.",
                    vm.severity.component_issues, share_pct
                )
            }
        } else {
            if vm.severity.component_issues == 1 {
                format!("Hauptursache: 1 Komponentenfehler verursacht {} % aller kritischen/hohen Befunde.", share_pct)
            } else {
                format!("Hauptursache: {} Komponentenfehler verursachen {} % aller kritischen/hohen Befunde.", vm.severity.component_issues, share_pct)
            }
        }
    } else {
        String::new()
    };

    if !dominant_cause_text.is_empty() {
        builder = builder.add_component(Divider {
            style: "solid".to_string(),
            thickness: "0pt".to_string(),
            color: Some("#ffffff".to_string()),
            spacing_above: "40pt".to_string(),
            spacing_below: "0pt".to_string(),
        });
        builder = builder.add_component(
            Label::new(dominant_cause_text)
                .with_size("11pt")
                .bold()
                .with_color("#b91c1c"),
        );
    }

    // --- Page 2: Management Summary ---
    builder = builder.add_component(PageBreak::new());
    builder = render_management_page(builder, &vm, &i18n);

    if vm.meta.report_level != ReportLevel::Executive {
        // --- Page 3: Table of Contents ---
        builder = builder.add_component(PageBreak::new());
        let toc_title = if en { "Contents" } else { "Inhalt" };
        builder = builder.add_component(TableOfContents::new().with_title(toc_title));

        // --- Part 1: Befunde nach Ursache (Divider) ---
        let (p1_title, p1_intro, p1_audience_title, p1_audience_body) = if en {
            (
                "Findings by Root Cause",
                "Distribution of identified template and component issues and prioritized roadmaps.",
                "Audience",
                "Developers and IT managers. Read this part to understand systemic issues and the remediation roadmap.",
            )
        } else {
            (
                "Befunde nach Ursache",
                "Verteilung der erkannten Template- und Komponentenfehler sowie der priorisierte Maßnahmenplan.",
                "Zielgruppe",
                "Entwickler und IT-Verantwortliche. Dieser Teil zeigt die systemischen Ursachen und den Ablaufplan.",
            )
        };
        builder = render_part_divider(
            builder,
            1,
            p1_title,
            p1_intro,
            p1_audience_title,
            p1_audience_body,
            &i18n,
        );

        // Ursachenanalyse (Table, components list, leverage)
        builder = render_root_cause_analysis(builder, &vm, &i18n);

        // Maßnahmenplan (Roadmap) & confidence notes
        builder = render_timeframe_roadmap(builder, &vm, &i18n);
        builder = render_audit_confidence_notes(builder, &normalized.audit_flags, &i18n);
    }

    // Executive level stops here
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

    // --- Part 2: Technische Details für Entwickler (Divider) ---
    let (p2_title, p2_intro, p2_audience_title, p2_audience_body) = if en {
        (
            "Technical Details for Developers",
            "Detailed WCAG violations, HTML evidence, and code examples grouped by systemic components vs. local instances.",
            "Audience",
            "Developers and QA teams. Read this part to implement fixes for all detected violations.",
        )
    } else {
        (
            "Technische Details für Entwickler",
            "Detaillierte WCAG-Verstöße, HTML-Evidenzen und Code-Beispiele, gruppiert nach systemischen Komponenten vs. Einzelfällen.",
            "Zielgruppe",
            "Entwickler und QA-Teams. Dieser Teil liefert konkrete Code-Befunde zur Behebung aller Barrieren.",
        )
    };
    builder = render_part_divider(
        builder,
        2,
        p2_title,
        p2_intro,
        p2_audience_title,
        p2_audience_body,
        &i18n,
    );

    // Technical details findings list
    builder = render_tech_details(builder, &vm, report, &i18n);

    // --- Part 3: Qualitäts-Analysen (Divider) ---
    let (p3_title, p3_intro, p3_audience_title, p3_audience_body) = if en {
        (
            "Quality Analyses",
            "Search engine optimization, load speed, mobile layout, and security header metrics.",
            "Audience",
            "Marketing, SEO specialists, and product owners. Read this part to optimize discoverability and performance.",
        )
    } else {
        (
            "Qualitäts-Analysen",
            "Suchmaschinenoptimierung, Ladezeiten, mobiles Layout und Sicherheits-Metriken.",
            "Zielgruppe",
            "Marketing, SEO-Spezialisten und Product Owner. Dieser Teil ergänzt die Barrierefreiheit um Qualitäts-Indikatoren.",
        )
    };
    builder = render_part_divider(
        builder,
        3,
        p3_title,
        p3_intro,
        p3_audience_title,
        p3_audience_body,
        &i18n,
    );

    // SEO, performance, mobile, ux, journey etc.
    builder = render_module_sections(builder, &vm, report, &i18n);

    // --- Appendix: Rohdaten & Methodik ---
    builder = render_appendix_full(builder, &vm, report, &i18n);

    let built_report = builder.build();
    Ok((engine, built_report))
}

fn build_recurring_pattern_panel(vm: &ReportViewModel, i18n: &I18n) -> ChecklistPanel {
    let en = i18n.locale() == "en";
    let rows: Vec<ChecklistRow> = vm
        .findings
        .top_findings
        .iter()
        .filter(|f| f.is_component_issue)
        .take(4)
        .map(|f| {
            let scope = if f.occurrence_count >= 25 {
                if en {
                    "component/template pattern"
                } else {
                    "Komponenten-/Template-Muster"
                }
            } else {
                if en {
                    "repeated pattern"
                } else {
                    "wiederholtes Muster"
                }
            };
            let detail = if en {
                format!(
                    "{} occurrence(s) — likely a {}. Customer effect: {}",
                    f.occurrence_count,
                    scope,
                    first_customer_sentence(&f.user_impact)
                )
            } else {
                format!(
                    "{} Vorkommen — wahrscheinlich ein {}. Kundenauswirkung: {}",
                    f.occurrence_count,
                    scope,
                    first_customer_sentence(&f.user_impact)
                )
            };
            ChecklistRow::new(&f.title, detail).with_status("warn")
        })
        .collect();

    ChecklistPanel::new(rows).with_title(if en {
        "Recurring issue patterns"
    } else {
        "Wiederkehrende Fehlerbilder"
    })
}

fn render_audit_confidence_notes(
    mut builder: renderreport::engine::ReportBuilder,
    audit_flags: &[crate::audit::normalized::AuditFlag],
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = i18n.locale() == "en";
    let base = if en {
        "Automated findings are reliable for detectable patterns, but context-dependent WCAG criteria, semantic content quality and user journeys still require manual verification. Indicator modules such as AI visibility, content visibility and UX are best-effort signals, not guarantees."
    } else {
        "Automatisierte Befunde sind belastbar für erkennbare Muster, aber kontextabhängige WCAG-Kriterien, semantische Inhaltsqualität und Nutzerwege benötigen weiterhin manuelle Prüfung. Indikator-Module wie KI-Sichtbarkeit, Content Visibility und UX sind Hinweiswerte, keine Garantien."
    };
    let title = if en {
        "Notes on report confidence"
    } else {
        "Hinweise zur Aussagekraft"
    };
    let text = format!("{}: {}", title, base);
    builder = builder.add_component(Label::new(&text).with_size("10.5pt").with_color("#475569"));

    if !audit_flags.is_empty() {
        let mut rows = Vec::new();
        for flag in audit_flags.iter().take(4) {
            rows.push(
                ChecklistRow::new(
                    audit_flag_title(&flag.kind, en),
                    audit_flag_customer_text(flag, en),
                )
                .with_status("warn"),
            );
        }
        builder = builder.add_component(ChecklistPanel::new(rows).with_title(if en {
            "Audit caveats"
        } else {
            "Audit-Hinweise"
        }));
    }
    builder
}

fn audit_flag_title(kind: &str, en: bool) -> &'static str {
    match (kind, en) {
        ("consent_banner", true) => "Consent banner",
        ("consent_banner", false) => "Consent-Banner",
        ("bypass_blocks_untested", true) => "Skip link verification",
        ("bypass_blocks_untested", false) => "Skip-Link-Prüfung",
        ("conflicting_signal", true) => "Conflicting signal",
        ("conflicting_signal", false) => "Widersprüchliches Signal",
        ("viewport_gap", true) => "Desktop/mobile difference",
        ("viewport_gap", false) => "Desktop-/Mobile-Unterschied",
        ("consent_wall_artifact", true) => "Consent wall artifact",
        ("consent_wall_artifact", false) => "Consent-Wall-Artefakt",
        (_, true) => "Audit note",
        (_, false) => "Audit-Hinweis",
    }
}

fn audit_flag_customer_text(flag: &crate::audit::normalized::AuditFlag, en: bool) -> String {
    match (flag.kind.as_str(), en) {
        ("consent_banner", true) => "A consent banner was detected. Parts of the page may have been hidden or not fully measurable without consent.".to_string(),
        ("consent_banner", false) => "Ein Consent-Banner wurde erkannt. Teile der Seite können ohne Zustimmung verborgen oder nicht vollständig messbar gewesen sein.".to_string(),
        ("bypass_blocks_untested", true) => "A skip link exists, but the usage journey indicates that keyboard focus may not reach the intended target.".to_string(),
        ("bypass_blocks_untested", false) => "Ein Skip-Link ist vorhanden, der Nutzungstest deutet aber darauf hin, dass der Tastaturfokus sein Ziel nicht zuverlässig erreicht.".to_string(),
        ("conflicting_signal", true) => "Two checks report different signals. Treat this result as a review point rather than a final conclusion.".to_string(),
        ("conflicting_signal", false) => "Zwei Prüfungen melden unterschiedliche Signale. Dieses Ergebnis sollte als Prüfhilfe statt als endgültige Aussage gelesen werden.".to_string(),
        ("viewport_gap", true) => "Desktop and mobile results differ strongly. The page experience should be checked separately for both viewports.".to_string(),
        ("viewport_gap", false) => "Desktop- und Mobile-Ergebnis unterscheiden sich deutlich. Die Nutzung sollte für beide Ansichten getrennt geprüft werden.".to_string(),
        _ => flag.message.clone(),
    }
}

fn first_customer_sentence(text: &str) -> &str {
    text.split(". ").next().unwrap_or(text)
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
