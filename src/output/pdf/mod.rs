//! PDF Report Generator using renderreport/Typst
//!
//! Pure layout layer — receives pre-computed ViewModel blocks and maps them
//! directly to renderreport components. Zero data transformation here.

mod appendix;
mod batch;
mod batch_report;
mod cover;
pub(crate) mod design;
mod detail_modules;
mod diagnosis;
mod findings;
mod helpers;
mod single_report;
mod wcag_coverage;

pub use self::batch_report::{generate_batch_pdf, generate_batch_typ};

use renderreport::components::advanced::{
    ChecklistPanel, ChecklistRow, PageBreak, TableOfContents,
};
use renderreport::components::text::Label;
use renderreport::components::{CoverModuleGauge, CoverPage};
use renderreport::prelude::*;

// Re-export helpers used by sibling sub-modules via `super::`.
use self::appendix::{cover_logo_asset, register_cover_logo_asset};
use self::diagnosis::{business_relevance, format_word_count, output_scope_callout};
use self::helpers::{create_engine, extract_domain};
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
const PAGE_DESKTOP_SCREENSHOT_ASSET: &str = "/auditmysite-desktop-preview.png";
const PAGE_MOBILE_SCREENSHOT_ASSET: &str = "/auditmysite-mobile-preview.png";

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
    let pdf_bytes: anyhow::Result<Vec<u8>> = (|| {
        let (engine, built_report) = build_single_report(report, config)?;
        Ok(engine.render_pdf(&built_report)?)
    })();
    cleanup_screenshot_temps(report);
    pdf_bytes
}

/// Render the intermediate Typst source for a single-page report.
///
/// Useful for the hidden `--debug-typ` mode and for snapshot/template-regression
/// checks (issue #239) — lets reviewers inspect completeness and wording without
/// going through the heavy PDF + pdftotext pipeline.
pub fn generate_typ(report: &AuditReport, config: &ReportConfig) -> anyhow::Result<String> {
    let typ: anyhow::Result<String> = (|| {
        let (engine, built_report) = build_single_report(report, config)?;
        Ok(engine.render_typ(&built_report)?)
    })();
    cleanup_screenshot_temps(report);
    typ
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

    builder = register_page_screenshot_assets(builder, report)?;

    // ── Cover Page (single composed component) ───────────────────────
    let en = i18n.locale() == "en";
    let overall_score = vm.summary.overall_score;
    let band_phrase = self::cover::cover_band_phrase(overall_score, en);
    let domain = extract_domain(&vm.cover.domain);
    let module_gauges: Vec<CoverModuleGauge> = vm
        .modules
        .dashboard
        .iter()
        .map(|m| CoverModuleGauge::new(m.name.clone(), m.score))
        .collect();
    let (score_lbl, find_lbl, mod_lbl, crit_lbl, no_crit) = if en {
        (
            "OVERALL SCORE",
            "FINDINGS",
            "MODULES AT A GLANCE",
            "Critical",
            "0 Critical",
        )
    } else {
        (
            "GESAMTSCORE",
            "BEFUNDE",
            "MODULE IM ÜBERBLICK",
            "kritisch",
            "0 kritisch",
        )
    };

    // White-label: only a user-supplied custom logo takes the brand slot on the
    // cover; the default keeps the clean text brand (no auto wordmark image).
    let cover_logo_asset = self::appendix::cover_logo_asset(config);
    let custom_logo = if cover_logo_asset == CUSTOM_COVER_LOGO_ASSET {
        builder = self::appendix::register_cover_logo_asset(builder, config, cover_logo_asset);
        Some(cover_logo_asset)
    } else {
        None
    };

    let mut cover = CoverPage::new(&vm.cover.title, &domain, overall_score, &vm.cover.grade)
        .with_brand(&vm.cover.brand)
        .with_subtitle(&vm.executive.cover_kicker)
        .with_date(&vm.cover.date)
        .with_band_phrase(band_phrase)
        .with_issues(vm.cover.total_issues, vm.cover.critical_issues)
        .with_module_gauges(module_gauges)
        .with_labels(score_lbl, find_lbl, mod_lbl, crit_lbl, no_crit);
    if let Some(logo) = custom_logo {
        cover = cover.with_logo(logo);
    }
    builder = builder.add_component(cover);

    // --- Page 2: Management Summary (CoverPage already emits a page break) ---
    builder = render_management_page(builder, &vm, &i18n);

    if vm.meta.report_level != ReportLevel::Executive {
        // --- Page 3: Table of Contents ---
        builder = builder.add_component(PageBreak::new());
        let toc_title = if en { "Contents" } else { "Inhalt" };
        builder = builder.add_component(TableOfContents::new().with_title(toc_title).with_depth(2));

        // --- Part 1: Befunde nach Ursache (Divider) ---
        let (p1_title, p1_intro, p1_audience_title, p1_audience_body) = if en {
            (
                "Findings by Root Cause",
                "Which underlying causes explain the most findings, and the prioritized plan to fix them.",
                "Audience",
                "Site owners, decision-makers, and developers. This part explains what is driving the findings and what to do next.",
            )
        } else {
            (
                "Befunde nach Ursache",
                "Welche Ursachen die meisten Befunde erklären – und der priorisierte Plan, sie zu beheben.",
                "Zielgruppe",
                "Inhaber, Entscheider und Entwickler. Dieser Teil erklärt, was die Befunde verursacht und was als Nächstes zu tun ist.",
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
        builder = render_audit_confidence_notes(builder, &normalized.normalized.audit_flags, &i18n);
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
        for flag in audit_flags {
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

fn cleanup_screenshot_temps(report: &AuditReport) {
    let ts = report.timestamp.timestamp_nanos_opt().unwrap_or(0);
    if let Some(ref _shots) = report.page_screenshots {
        let _ = std::fs::remove_file(std::env::temp_dir().join(format!("ams-desktop-{}.png", ts)));
        let _ = std::fs::remove_file(std::env::temp_dir().join(format!("ams-mobile-{}.png", ts)));
    }
    // Element-evidence crops (evidence-grade findings): named
    // `ams-evidence-{ts}-{n}.png` by `render_finding_technical`, n in
    // 0..MAX_ELEMENT_CROPS. Removing a non-existent path is a harmless no-op,
    // so this runs unconditionally rather than tracking whether captures
    // actually happened.
    for n in 0..crate::accessibility::MAX_ELEMENT_CROPS {
        let _ = std::fs::remove_file(
            std::env::temp_dir().join(format!("ams-evidence-{}-{}.png", ts, n)),
        );
    }
}

fn register_page_screenshot_assets(
    mut builder: renderreport::engine::ReportBuilder,
    report: &AuditReport,
) -> anyhow::Result<renderreport::engine::ReportBuilder> {
    let Some(ref shots) = report.page_screenshots else {
        return Ok(builder);
    };

    let ts = report.timestamp.timestamp_nanos_opt().unwrap_or(0);
    let desktop_path = std::env::temp_dir().join(format!("ams-desktop-{}.png", ts));
    let mobile_path = std::env::temp_dir().join(format!("ams-mobile-{}.png", ts));

    std::fs::write(&desktop_path, &shots.desktop)?;
    std::fs::write(&mobile_path, &shots.mobile)?;

    builder = builder
        .asset(PAGE_DESKTOP_SCREENSHOT_ASSET, desktop_path)
        .asset(PAGE_MOBILE_SCREENSHOT_ASSET, mobile_path);
    Ok(builder)
}

#[cfg(all(test, feature = "pdf_test"))]
#[path = "tests.rs"]
mod tests;
