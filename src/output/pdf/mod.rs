//! PDF Report Generator using renderreport/Typst
//!
//! Pure layout layer — receives pre-computed ViewModel blocks and maps them
//! directly to renderreport components. Zero data transformation here.

mod annotations;
mod appendix;
mod batch_report;
mod bik_guide;
mod cover;
pub(crate) mod design;
mod detail_modules;
mod diagnosis;
mod en301549;
mod findings;
mod helpers;
mod problem_profile;
mod root_causes;
mod sanitize;
mod score_drivers;
mod single_report;
mod wcag_coverage;

pub use self::batch_report::{
    generate_batch_pdf, generate_batch_pdf_with_annotations, generate_batch_typ,
    generate_batch_typ_with_annotations,
};

use crate::output::localized::is_english;
use renderreport::components::advanced::{
    ChecklistPanel, ChecklistRow, PageBreak, TableOfContents,
};
use renderreport::components::text::Label;
use renderreport::components::{CoverModuleGauge, CoverPage};
use renderreport::prelude::*;

// Re-export helpers used by sibling sub-modules via `super::`.
use self::appendix::{cover_logo_asset, register_cover_logo_asset};
use self::helpers::create_engine;
use self::root_causes::{render_root_cause_analysis, render_timeframe_roadmap};
use self::single_report::{
    render_appendix_full, render_management_page, render_module_sections, render_part_divider,
    render_tech_details,
};
use crate::audit::{normalize_checked, AuditReport};
use crate::i18n::I18n;
use crate::model::ReportLevel;
use crate::output::builder::{build_view_model, extract_domain};
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
    generate_pdf_with_annotations(report, config, None)
}

/// [`generate_pdf`] with the external assessments of `--annotations` (#961).
/// `None` renders exactly what `generate_pdf` renders.
pub fn generate_pdf_with_annotations(
    report: &AuditReport,
    config: &ReportConfig,
    annotations: Option<&crate::annotations::Resolved>,
) -> anyhow::Result<Vec<u8>> {
    let pdf_bytes: anyhow::Result<Vec<u8>> = (|| {
        let (engine, mut built_report) = build_single_report(report, config, annotations)?;
        self::sanitize::render_pdf_sanitized(&engine, &mut built_report)
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
    generate_typ_with_annotations(report, config, None)
}

/// [`generate_typ`] with the external assessments of `--annotations` (#961).
pub fn generate_typ_with_annotations(
    report: &AuditReport,
    config: &ReportConfig,
    annotations: Option<&crate::annotations::Resolved>,
) -> anyhow::Result<String> {
    let typ: anyhow::Result<String> = (|| {
        let (engine, mut built_report) = build_single_report(report, config, annotations)?;
        self::sanitize::sanitize_request(&mut built_report, &Default::default());
        Ok(engine.render_typ(&built_report)?)
    })();
    cleanup_screenshot_temps(report);
    typ
}

fn build_single_report(
    report: &AuditReport,
    config: &ReportConfig,
    annotations: Option<&crate::annotations::Resolved>,
) -> anyhow::Result<(
    std::sync::Arc<renderreport::Engine>,
    renderreport::RenderRequest,
)> {
    let normalized = normalize_checked(report)?;
    let engine = create_engine();
    let mut vm = build_view_model(&normalized, config);
    vm.annotations = annotations.cloned();
    let i18n = I18n::new(&config.locale)?;

    let mut builder = engine
        .report("wcag-audit")
        .metadata("date", &vm.meta.date)
        .metadata("version", &vm.meta.version)
        .metadata("author", &vm.meta.author)
        .metadata("score", &vm.meta.score_label)
        .metadata("lang", i18n.locale())
        .metadata("footer_prefix", "Audit:")
        .metadata("footer_link_url", "")
        .metadata(
            "footer_tagline",
            "A technical auditing platform by casoon.de",
        );

    builder = register_page_screenshot_assets(builder, report)?;

    // ── Cover Page (single composed component) ───────────────────────
    let en = is_english(&i18n);
    // The headline is the accessibility score — the subject of the report.
    // The weighted overall value across all six modules is a different
    // question and is reported as such, further in (plan 29, D1).
    let headline_score = vm.summary.score;
    let band_phrase = self::cover::cover_band_phrase(
        headline_score,
        vm.summary.ci_verdict,
        vm.summary.run_is_partial,
        en,
    );
    let domain = extract_domain(&vm.cover.domain);
    // Two strips: the six modules that make up the overall score, and below
    // them, smaller and neutral, the indicators that only re-read them
    // (composite, UX, Journey) — one row made a derived 71 look like the
    // SEO score the overall counted as 65 (plan 105).
    let (measured, indicators): (Vec<_>, Vec<_>) = vm
        .modules
        .dashboard
        .iter()
        .partition(|m| m.measurement_type == "measured");
    let module_gauges: Vec<CoverModuleGauge> = measured
        .iter()
        .map(|m| CoverModuleGauge::new(&m.name, m.score))
        .collect();
    let indicator_gauges: Vec<CoverModuleGauge> = indicators
        .iter()
        .map(|m| CoverModuleGauge::new(&m.name, m.score))
        .collect();
    // The gauge strip's label carries the scope qualifier: a bare
    // "Barrierefreiheit 100" on the cover reads as "fully accessible", which
    // no automated run can establish.
    let score_lbl = i18n.t("pdf-frame-cover-score-label");
    let find_lbl = i18n.t("pdf-frame-cover-findings-label");
    let mod_lbl = i18n.t("pdf-frame-cover-modules-label");
    let crit_lbl = i18n.t("pdf-frame-cover-critical-label");
    let no_crit = i18n.t("pdf-frame-cover-no-critical");

    // White-label: only a user-supplied custom logo takes the brand slot on the
    // cover; the default keeps the clean text brand (no auto wordmark image).
    let cover_logo_asset = self::appendix::cover_logo_asset(config);
    let custom_logo = if cover_logo_asset == CUSTOM_COVER_LOGO_ASSET {
        builder = self::appendix::register_cover_logo_asset(builder, config, cover_logo_asset);
        Some(cover_logo_asset)
    } else {
        None
    };

    // What was actually audited belongs on the cover, not only on page 2 —
    // "Website-Qualitätsbericht casoon.de · 92" otherwise reads as a verdict
    // on the whole site when exactly one page was checked.
    //
    // The cover has room for exactly one subtitle line: a second line pushes
    // the lower gauge row's labels off the page (the template gives each label
    // a fixed 22pt box and the page does not grow). `narrative-cover-kicker`
    // is therefore deliberately terse — the areas it used to enumerate are
    // named by the gauge strip right below it anyway.
    let cover_subtitle = format!(
        "{} · {}",
        vm.executive.cover_kicker,
        self::single_report::audit_scope_line(en)
    );
    let mut cover = CoverPage::new(&vm.cover.title, &domain, headline_score, &vm.cover.grade)
        .with_brand(&vm.cover.brand)
        .with_subtitle(&cover_subtitle)
        .with_date(&vm.cover.date)
        .with_band_phrase(&band_phrase)
        .with_issues(vm.cover.total_issues, vm.cover.critical_issues)
        .with_module_gauges(module_gauges)
        .with_indicator_gauges(i18n.t("pdf-frame-cover-indicators-label"), indicator_gauges)
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
        builder = builder.add_component(
            TableOfContents::new()
                .with_title(i18n.t("pdf-frame-toc-title"))
                .with_depth(2),
        );

        // --- Part 1: Befunde nach Ursache (Divider) ---
        builder = render_part_divider(
            builder,
            1,
            &i18n.t("pdf-frame-part1-title"),
            &i18n.t("pdf-frame-part1-intro"),
            &i18n.t("pdf-frame-part-audience-title"),
            &i18n.t("pdf-frame-part1-audience"),
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
        // No finding cards at this level, so the assessments are listed here.
        if let Some(resolved) = vm.annotations.as_ref() {
            builder = self::annotations::render_assessments_section(builder, resolved, true, &i18n);
        }
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
    builder = render_part_divider(
        builder,
        2,
        &i18n.t("pdf-frame-part2-title"),
        &i18n.t("pdf-frame-part2-intro"),
        &i18n.t("pdf-frame-part-audience-title"),
        &i18n.t("pdf-frame-part2-audience"),
        &i18n,
    );

    // Technical details findings list
    builder = render_tech_details(builder, &vm, report, &i18n);

    // --- Part 3: Qualitäts-Analysen (Divider) ---
    builder = render_part_divider(
        builder,
        3,
        &i18n.t("pdf-frame-part3-title"),
        &i18n.t("pdf-frame-part3-intro"),
        &i18n.t("pdf-frame-part-audience-title"),
        &i18n.t("pdf-frame-part3-audience"),
        &i18n,
    );

    // SEO, performance, mobile, ux, journey etc.
    builder = render_module_sections(builder, &vm, report, &i18n);

    // --- Appendix: Rohdaten & Methodik ---
    builder = render_appendix_full(
        builder,
        &vm,
        report,
        &normalized.normalized.findings,
        config,
        &i18n,
    );

    let built_report = builder.build();
    Ok((engine, built_report))
}

fn render_audit_confidence_notes(
    mut builder: renderreport::engine::ReportBuilder,
    audit_flags: &[crate::audit::normalized::AuditFlag],
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let text = format!(
        "{}: {}",
        i18n.t("pdf-frame-confidence-title"),
        i18n.t("pdf-frame-confidence-base")
    );
    builder = builder.add_component(Label::new(&text).with_size("10.5pt").with_color("#475569"));

    if !audit_flags.is_empty() {
        let mut rows = Vec::new();
        for flag in audit_flags {
            rows.push(
                ChecklistRow::new(
                    audit_flag_title(&flag.kind, i18n),
                    audit_flag_customer_text(flag, i18n),
                )
                .with_status("warn"),
            );
        }
        builder = builder.add_component(
            ChecklistPanel::new(rows).with_title(i18n.t("pdf-frame-confidence-caveats-title")),
        );
    }
    builder
}

/// The titles of the flags the batch report also lists are its
/// `batch-management-flag-*` messages.
fn audit_flag_title(kind: &str, i18n: &I18n) -> String {
    i18n.t(match kind {
        "consent_banner" => "batch-management-flag-consent-banner",
        "bypass_blocks_untested" => "batch-management-flag-bypass-blocks-untested",
        "conflicting_signal" => "batch-management-flag-conflicting-signal",
        "viewport_gap" => "batch-management-flag-viewport-gap",
        "incomplete_audit" => "pdf-frame-flag-title-incomplete-audit",
        "side_check_unanswered" => "pdf-frame-flag-title-side-check-unanswered",
        "consent_wall_artifact" => "batch-management-flag-consent-wall-artifact",
        _ => "batch-management-flag-other",
    })
}

/// Names the side checks that got no answer. The page itself was measured in
/// full, so the sentence says which check is missing and that nothing else is.
fn side_check_unanswered_text(source: &str, i18n: &I18n) -> String {
    let names: Vec<String> = source
        .split(',')
        .map(|id| {
            i18n.t(match id {
                "seo.www_redirect" | "seo.page_health.www_consolidation" => {
                    "pdf-frame-side-check-www"
                }
                "security.certificate" => "pdf-frame-side-check-certificate",
                _ => "pdf-frame-side-check-other",
            })
        })
        .fold(Vec::new(), |mut unique, name| {
            if !unique.contains(&name) {
                unique.push(name);
            }
            unique
        });
    i18n.t_args("pdf-frame-side-check-text", &[("list", names.join("; "))])
}

fn audit_flag_customer_text(flag: &crate::audit::normalized::AuditFlag, i18n: &I18n) -> String {
    match flag.kind.as_str() {
        "consent_banner" => i18n.t("pdf-frame-flag-text-consent-banner"),
        "bypass_blocks_untested" => i18n.t("pdf-frame-flag-text-bypass-blocks-untested"),
        "conflicting_signal" => i18n.t("pdf-frame-flag-text-conflicting-signal"),
        "viewport_gap" => i18n.t("pdf-frame-flag-text-viewport-gap"),
        "incomplete_audit" => i18n.t("pdf-frame-flag-text-incomplete-audit"),
        "side_check_unanswered" => side_check_unanswered_text(&flag.source, i18n),
        _ => flag.message.clone(),
    }
}

fn cleanup_screenshot_temps(report: &AuditReport) {
    let ts = report.timestamp.timestamp_nanos_opt().unwrap_or(0);
    if let Some(ref _shots) = report.page_screenshots {
        remove_temp_file(&std::env::temp_dir().join(format!("ams-desktop-{}.png", ts)));
        remove_temp_file(&std::env::temp_dir().join(format!("ams-mobile-{}.png", ts)));
    }
    // Element-evidence crops (evidence-grade findings): named
    // `ams-evidence-{ts}-{n}.png` by `render_finding_technical`, n in
    // 0..MAX_ELEMENT_CROPS. Removing a non-existent path is a harmless no-op,
    // so this runs unconditionally rather than tracking whether captures
    // actually happened.
    for n in 0..crate::accessibility::MAX_ELEMENT_CROPS {
        remove_temp_file(&std::env::temp_dir().join(format!("ams-evidence-{}-{}.png", ts, n)));
    }
}

/// Best-effort removal of a temporary render file; failures are only logged at debug level.
fn remove_temp_file(path: &std::path::Path) {
    if let Err(e) = remove_file_if_exists(path) {
        tracing::debug!("Failed to remove temporary file {}: {}", path.display(), e);
    }
}

/// Like `std::fs::remove_file`, but a file that was never created is not an error.
fn remove_file_if_exists(path: &std::path::Path) -> std::io::Result<()> {
    match std::fs::remove_file(path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        result => result,
    }
}

/// Target width:height ratio for the mobile device-preview crop, derived
/// from the `DevicePreview` component's own layout (70/30 desktop/mobile
/// column split at a shared box height — see `device_preview.typ`): the
/// mobile column is `(1-0.7)/0.7 ≈ 0.43` as wide as the desktop column, and
/// desktop's 1280:800 screenshot already fits its column without cropping,
/// so mobile's target ratio is `0.43 * (1280/800)`.
const MOBILE_PREVIEW_CROP_RATIO: f64 = 0.686;

/// The raw mobile screenshot is a full-viewport capture (e.g. 390×844 CSS px
/// at 2x DPR), several times taller than it is wide. Typst's box+clip
/// overflow handling for an image this much taller than its display box was
/// observed to lose the *top* of the image — exactly the page header a
/// reader most needs to see — instead of reliably keeping it in frame (see
/// `device_preview.typ`). Crop to a display-appropriate aspect ratio here,
/// in Rust, so the asset handed to Typst rarely needs any further clipping.
fn crop_mobile_preview(png_bytes: &[u8]) -> Vec<u8> {
    let Ok(img) = image::load_from_memory(png_bytes) else {
        return png_bytes.to_vec();
    };
    let (width, height) = (img.width(), img.height());
    let target_height = ((width as f64) / MOBILE_PREVIEW_CROP_RATIO).round() as u32;
    if target_height >= height {
        return png_bytes.to_vec();
    }
    let cropped = img.crop_imm(0, 0, width, target_height);
    let mut out = Vec::new();
    if cropped
        .write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
        .is_err()
    {
        return png_bytes.to_vec();
    }
    out
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
    std::fs::write(&mobile_path, crop_mobile_preview(&shots.mobile))?;

    builder = builder
        .asset(PAGE_DESKTOP_SCREENSHOT_ASSET, desktop_path)
        .asset(PAGE_MOBILE_SCREENSHOT_ASSET, mobile_path);
    Ok(builder)
}

#[cfg(test)]
mod frame_text_export;
#[cfg(test)]
mod frame_text_guard;

#[cfg(all(test, feature = "pdf_test"))]
#[path = "tests.rs"]
mod tests;
