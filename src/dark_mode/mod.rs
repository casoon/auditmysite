//! Dark Mode Analysis
//!
//! Detects whether a site supports `prefers-color-scheme: dark` and audits
//! the quality of that implementation, including:
//!
//! - Static CSS detection (media queries, `color-scheme` declarations, meta tags)
//! - Dynamic contrast re-check after emulating dark mode via CDP
//! - Comparison of light-mode vs. dark-mode contrast violations
//! - Per-element contrast violation details for dark mode (selector, message, mode)

pub mod module;
pub use module::DarkModeModule;

use chromiumoxide::cdp::browser_protocol::emulation::{
    MediaFeature, SetEmulatedVisionDeficiencyParams, SetEmulatedVisionDeficiencyType,
};
use chromiumoxide::page::MediaTypeParams;
use chromiumoxide::Page;
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::cli::WcagLevel;
use crate::error::{AuditError, Result};
use crate::interaction::stability::settle;
use crate::wcag::rules::{check_use_of_color_with_page, ContrastRule};
use crate::wcag::Violation;

// ─── Public types ────────────────────────────────────────────────────────────

/// Complete dark mode analysis for a single page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DarkModeAnalysis {
    /// Site declares `@media (prefers-color-scheme: dark)` CSS rules.
    pub supported: bool,
    /// Dark mode is implemented via CSS class toggle (html.dark / [data-theme="dark"])
    /// rather than @media (prefers-color-scheme: dark). Contrast testing via CDP is not
    /// possible in this mode.
    pub class_based_dark_mode: bool,
    /// Aggregate quality score 0–100.
    pub score: u32,
    /// Human-readable list of detected implementation methods.
    pub detection_methods: Vec<String>,
    /// `:root { color-scheme: ... }` or `html { color-scheme: ... }` present.
    pub color_scheme_css: bool,
    /// `<meta name="color-scheme" content="...">` value, if found.
    pub meta_color_scheme: Option<String>,
    /// `<meta name="theme-color" media="(prefers-color-scheme: dark)">` found.
    pub meta_theme_color_dark: bool,
    /// Number of CSS custom properties with color-related names on `:root`.
    pub css_custom_properties: u32,
    /// Contrast violations found while dark mode is active.
    pub dark_contrast_violations: u32,
    /// Contrast violations that exist in light mode but disappear in dark mode.
    pub light_only_violations: u32,
    /// New contrast violations that appear only in dark mode.
    pub dark_only_violations: u32,
    /// Per-element contrast violation details (light-and-dark, dark-only, light-only).
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub contrast_violations: Vec<DarkContrastViolation>,
    /// Print stylesheet/readiness checks via CSS inspection and print media emulation.
    #[serde(default)]
    pub print: PrintStylesheetAnalysis,
    /// Forced-colors / Windows High Contrast readiness checks via media emulation.
    #[serde(default)]
    pub forced_colors: ForcedColorsAnalysis,
    /// Color vision deficiency simulation results via CDP.
    #[serde(default)]
    pub vision_deficiency: VisionDeficiencyAnalysis,
    /// Non-contrast issues (missing support, incomplete implementation, contrast summary).
    pub issues: Vec<DarkModeIssue>,
}

/// Print stylesheet and print-rendering readiness.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PrintStylesheetAnalysis {
    /// Any `@media print` rule or print stylesheet link was detected.
    pub stylesheet_detected: bool,
    /// Print media emulation succeeded through CDP.
    pub emulation_supported: bool,
    /// Navigation/header/footer interactive chrome appears hidden or reduced in print.
    pub interactive_chrome_hidden: bool,
    /// Main content appears unclipped under print media.
    pub content_not_clipped: bool,
    /// Number of potentially clipped visible elements after print emulation.
    pub clipped_elements: u32,
}

/// Forced-colors / Windows High Contrast readiness.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ForcedColorsAnalysis {
    /// Any `@media (forced-colors: active)` rule or `forced-color-adjust` usage was detected.
    pub stylesheet_detected: bool,
    /// Forced-colors media emulation succeeded through CDP.
    pub emulation_supported: bool,
    /// The emulated page reports `matchMedia('(forced-colors: active)').matches`.
    pub active_matches: bool,
    /// Number of elements using `forced-color-adjust`.
    pub forced_color_adjust_count: u32,
    /// Focusable controls have visible borders/outlines/background under forced colors.
    pub focus_indicators_visible: bool,
}

/// Color vision deficiency simulation summary.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VisionDeficiencyAnalysis {
    /// CDP vision-deficiency emulation succeeded for at least one mode.
    pub emulation_supported: bool,
    /// Per-mode results for protanopia/deuteranopia/tritanopia/achromatopsia.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub modes: Vec<VisionDeficiencyModeAnalysis>,
}

/// Per deficiency mode contrast/use-of-color result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisionDeficiencyModeAnalysis {
    pub mode: String,
    pub contrast_violations: u32,
    pub new_contrast_violations: u32,
    pub use_of_color_violations: u32,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub new_contrast_selectors: Vec<String>,
}

/// Per-element contrast violation detail from dark mode analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DarkContrastViolation {
    /// CSS selector identifying the affected element.
    pub selector: Option<String>,
    /// Human-readable contrast message (includes fg/bg colors when available).
    pub message: String,
    /// Which rendering mode(s) this violation appears in.
    pub mode: DarkContrastMode,
}

/// Which color scheme mode a contrast violation appears in.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DarkContrastMode {
    /// Violation exists in both light and dark mode.
    LightAndDark,
    /// Violation only in dark mode — dark mode regression.
    DarkOnly,
    /// Violation only in light mode — dark mode fixes it.
    LightOnly,
}

/// A single dark-mode quality issue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DarkModeIssue {
    pub kind: DarkModeIssueKind,
    pub description: String,
    pub severity: String,
}

/// Issue category for dark mode problems.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DarkModeIssueKind {
    NoDarkModeSupport,
    NoColorSchemeDeclaration,
    NoMetaColorScheme,
    DarkModeContrastFailure,
    IncompleteImplementation,
    NoPrintStylesheet,
    PrintLayoutRisk,
    NoForcedColorsSupport,
    ForcedColorsFocusRisk,
    ColorVisionDeficiencyContrastFailure,
}

// ─── Main entry point ────────────────────────────────────────────────────────

/// Analyse dark mode support and quality on the given already-loaded page.
///
/// Steps:
/// 1. Static JS detection: CSS media queries, `color-scheme`, meta tags, CSS vars.
/// 2. If dark mode is supported: emulate dark via CDP, re-run contrast, restore.
/// 3. Build per-element contrast violation list with mode classification.
/// 4. Build score and issue list.
pub async fn analyze_dark_mode(page: &Page, wcag_level: WcagLevel) -> Result<DarkModeAnalysis> {
    // ── 1. Static detection ─────────────────────────────────────────────────
    let static_info = detect_static_support(page).await?;
    let print = analyze_print_stylesheet(page).await.unwrap_or_else(|e| {
        warn!("Print stylesheet analysis failed: {e}");
        PrintStylesheetAnalysis::default()
    });
    let forced_colors = analyze_forced_colors(page).await.unwrap_or_else(|e| {
        warn!("Forced-colors analysis failed: {e}");
        ForcedColorsAnalysis::default()
    });
    let vision_deficiency = analyze_vision_deficiency(page, wcag_level)
        .await
        .unwrap_or_else(|e| {
            warn!("Vision-deficiency analysis failed: {e}");
            VisionDeficiencyAnalysis::default()
        });

    // ── 2. Contrast comparison (only when dark mode CSS exists) ─────────────
    let (light_viols, dark_viols) = if static_info.has_dark_media_query
        && matches!(wcag_level, WcagLevel::AA | WcagLevel::AAA)
    {
        compare_contrast(page, wcag_level).await
    } else {
        (Vec::new(), Vec::new())
    };

    // ── 3. Classify per-element violations ───────────────────────────────────
    let contrast_violations = classify_contrast_violations(&light_viols, &dark_viols);

    let dark_contrast_count = dark_viols.len() as u32;
    let dark_only = contrast_violations
        .iter()
        .filter(|v| v.mode == DarkContrastMode::DarkOnly)
        .count() as u32;
    let light_only = contrast_violations
        .iter()
        .filter(|v| v.mode == DarkContrastMode::LightOnly)
        .count() as u32;

    // ── 4. Build issues list ──────────────────────────────────────────────────
    let issues = build_issues(
        &static_info,
        dark_contrast_count,
        dark_only,
        light_only,
        &contrast_violations,
        &print,
        &forced_colors,
        &vision_deficiency,
    );

    // ── 5. Score ─────────────────────────────────────────────────────────────
    let score = compute_score(
        &static_info,
        dark_contrast_count,
        dark_only,
        &print,
        &forced_colors,
        &vision_deficiency,
    );

    // ── 6. Detection methods ──────────────────────────────────────────────────
    // Only include signals that actually implement dark styling, not mere hints.
    // color-scheme CSS / meta_color_scheme tell the browser which scheme the site
    // prefers but do not apply dark styles — they stay as separate boolean fields.
    let mut detection_methods: Vec<String> = Vec::new();
    if static_info.has_dark_media_query {
        detection_methods.push("@media (prefers-color-scheme: dark)".to_string());
    }
    if static_info.has_class_based_dark_mode {
        detection_methods.push("CSS class-based (.dark / [data-theme=dark])".to_string());
    }
    if static_info.meta_theme_color_dark {
        detection_methods.push("<meta name=\"theme-color\" media dark>".to_string());
    }

    Ok(DarkModeAnalysis {
        supported: static_info.has_dark_media_query || static_info.has_class_based_dark_mode,
        class_based_dark_mode: static_info.has_class_based_dark_mode,
        score,
        detection_methods,
        color_scheme_css: static_info.color_scheme_css,
        meta_color_scheme: static_info.meta_color_scheme,
        meta_theme_color_dark: static_info.meta_theme_color_dark,
        css_custom_properties: static_info.css_custom_properties,
        dark_contrast_violations: dark_contrast_count,
        light_only_violations: light_only,
        dark_only_violations: dark_only,
        contrast_violations,
        print,
        forced_colors,
        vision_deficiency,
        issues,
    })
}

// ─── Issue generation ─────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn build_issues(
    info: &StaticDarkModeInfo,
    dark_contrast_count: u32,
    dark_only: u32,
    light_only: u32,
    contrast_violations: &[DarkContrastViolation],
    print: &PrintStylesheetAnalysis,
    forced_colors: &ForcedColorsAnalysis,
    vision_deficiency: &VisionDeficiencyAnalysis,
) -> Vec<DarkModeIssue> {
    let mut issues: Vec<DarkModeIssue> = Vec::new();

    if !info.has_dark_media_query && !info.has_class_based_dark_mode {
        issues.push(DarkModeIssue {
            kind: DarkModeIssueKind::NoDarkModeSupport,
            description: "No @media (prefers-color-scheme: dark) rules found. \
                          Users with system dark mode enabled will receive the light view."
                .to_string(),
            severity: "medium".to_string(),
        });
    }

    if !info.has_dark_media_query && info.has_class_based_dark_mode {
        issues.push(DarkModeIssue {
            kind: DarkModeIssueKind::IncompleteImplementation,
            description: "Class-based dark mode detected (html.dark / [data-theme=\"dark\"]). \
                          Contrast checking in dark mode via CDP emulation is not possible — \
                          only @media (prefers-color-scheme: dark) can be tested automatically."
                .to_string(),
            severity: "low".to_string(),
        });
    }

    // ── Structural best-practice issues ──────────────────────────────────────
    if (info.has_dark_media_query || info.has_class_based_dark_mode) && !info.color_scheme_css {
        issues.push(DarkModeIssue {
            kind: DarkModeIssueKind::NoColorSchemeDeclaration,
            description: "No `color-scheme: dark light` declared on :root. The browser will not use \
                          native dark mode colors for scrollbars, form controls, and other UI elements."
                .to_string(),
            severity: "low".to_string(),
        });
    }
    if (info.has_dark_media_query || info.has_class_based_dark_mode)
        && info.meta_color_scheme.is_none()
    {
        issues.push(DarkModeIssue {
            kind: DarkModeIssueKind::NoMetaColorScheme,
            description:
                "No <meta name=\"color-scheme\"> found. Without this meta tag the browser cannot \
                          optimize rendering behavior before the CSSOM is built."
                    .to_string(),
            severity: "low".to_string(),
        });
    }
    if (info.has_dark_media_query || info.has_class_based_dark_mode)
        && info.css_custom_properties < 3
    {
        issues.push(DarkModeIssue {
            kind: DarkModeIssueKind::IncompleteImplementation,
            description: "Few CSS custom properties for colors detected. Complete dark mode \
                          implementations typically use CSS variables (--color-*) on :root \
                          and override them inside the media query."
                .to_string(),
            severity: "low".to_string(),
        });
    }

    // ── Print and forced-colors issues (#436) ────────────────────────────────
    if !print.stylesheet_detected {
        issues.push(DarkModeIssue {
            kind: DarkModeIssueKind::NoPrintStylesheet,
            description: "No print stylesheet rules detected. Printed or PDF-saved pages may unnecessarily include navigation, cookie banners, or interactive elements."
                .to_string(),
            severity: "low".to_string(),
        });
    } else if !print.interactive_chrome_hidden || !print.content_not_clipped {
        issues.push(DarkModeIssue {
            kind: DarkModeIssueKind::PrintLayoutRisk,
            description: format!(
                "Print stylesheet detected but the print layout appears risky: interactive chrome hidden = {}, potentially clipped elements = {}.",
                if print.interactive_chrome_hidden { "yes" } else { "no" },
                print.clipped_elements
            ),
            severity: if print.content_not_clipped { "low" } else { "medium" }.to_string(),
        });
    }

    if !forced_colors.stylesheet_detected {
        issues.push(DarkModeIssue {
            kind: DarkModeIssueKind::NoForcedColorsSupport,
            description: "No targeted forced-colors rules detected. Windows high-contrast users may lose focus indicators, borders, or status surfaces when colors are defined purely visually."
                .to_string(),
            severity: "low".to_string(),
        });
    } else if !forced_colors.focus_indicators_visible {
        issues.push(DarkModeIssue {
            kind: DarkModeIssueKind::ForcedColorsFocusRisk,
            description: "Forced-colors rules detected, but focusable elements do not consistently retain visible borders, backgrounds, or outlines under emulation."
                .to_string(),
            severity: "medium".to_string(),
        });
    }

    let affected_modes: Vec<String> = vision_deficiency
        .modes
        .iter()
        .filter(|mode| mode.new_contrast_violations > 0)
        .map(|mode| format!("{} ({})", mode.mode, mode.new_contrast_violations))
        .collect();
    if !affected_modes.is_empty() {
        issues.push(DarkModeIssue {
            kind: DarkModeIssueKind::ColorVisionDeficiencyContrastFailure,
            description: format!(
                "Additional contrast failures under color vision deficiency emulation: {}. Review these elements with non-color state cues and more robust color values.",
                affected_modes.join(", ")
            ),
            severity: "medium".to_string(),
        });
    }

    // ── Contrast issues ───────────────────────────────────────────────────────

    // Dark-only regressions: these elements were fine in light mode but break in dark mode
    if dark_only > 0 {
        let selectors = selector_list(
            contrast_violations
                .iter()
                .filter(|v| v.mode == DarkContrastMode::DarkOnly),
            5,
        );
        issues.push(DarkModeIssue {
            kind: DarkModeIssueKind::DarkModeContrastFailure,
            description: format!(
                "{dark_only} {} lose contrast in dark mode (dark mode regression). \
                 These elements pass in light mode but fall below the WCAG minimum contrast \
                 in dark mode.{selectors}",
                if dark_only == 1 {
                    "element"
                } else {
                    "elements"
                }
            ),
            severity: "high".to_string(),
        });
    }

    // Violations present in both modes: dark mode doesn't fix the underlying problem
    let both = dark_contrast_count.saturating_sub(dark_only);
    if both > 0 {
        let selectors = selector_list(
            contrast_violations
                .iter()
                .filter(|v| v.mode == DarkContrastMode::LightAndDark),
            5,
        );
        issues.push(DarkModeIssue {
            kind: DarkModeIssueKind::DarkModeContrastFailure,
            description: format!(
                "{both} {} insufficient contrast in both color modes (light and dark). \
                 The dark mode implementation does not adjust the colors of these elements sufficiently.{selectors}",
                if both == 1 { "element has" } else { "elements have" }
            ),
            severity: "high".to_string(),
        });
    }

    // Light-only: informational (dark mode actually improves these)
    if light_only > 0 {
        issues.push(DarkModeIssue {
            kind: DarkModeIssueKind::DarkModeContrastFailure,
            description: format!(
                "{light_only} {} contrast issues only in light mode — dark mode resolves them. \
                 Consider adjusting the light mode colors accordingly.",
                if light_only == 1 {
                    "element has"
                } else {
                    "elements have"
                }
            ),
            severity: "low".to_string(),
        });
    }

    issues
}

/// Format up to `max` selectors from an iterator as a readable suffix string.
fn selector_list<'a>(iter: impl Iterator<Item = &'a DarkContrastViolation>, max: usize) -> String {
    let selectors: Vec<&str> = iter
        .filter_map(|v| v.selector.as_deref())
        .filter(|s| !s.is_empty())
        .take(max)
        .collect();

    if selectors.is_empty() {
        return String::new();
    }

    let list = selectors.join(", ");
    format!(" Betroffene Elemente: {list}.")
}

// ─── Violation classification ─────────────────────────────────────────────────

/// Classify each violation from light and dark passes into light_and_dark / dark_only / light_only.
///
/// Dedup key: selector when present (stable across passes), otherwise message.
fn classify_contrast_violations(
    light: &[Violation],
    dark: &[Violation],
) -> Vec<DarkContrastViolation> {
    fn key(v: &Violation) -> String {
        v.selector
            .as_deref()
            .filter(|s| !s.is_empty())
            .map(|s| s.to_owned())
            .unwrap_or_else(|| v.message.clone())
    }

    use std::collections::HashMap;

    let light_map: HashMap<String, &Violation> = light.iter().map(|v| (key(v), v)).collect();
    let dark_map: HashMap<String, &Violation> = dark.iter().map(|v| (key(v), v)).collect();

    let mut result: Vec<DarkContrastViolation> = Vec::new();

    // Dark violations: dark_only or light_and_dark
    for (k, dv) in &dark_map {
        let mode = if light_map.contains_key(k) {
            DarkContrastMode::LightAndDark
        } else {
            DarkContrastMode::DarkOnly
        };
        result.push(DarkContrastViolation {
            selector: dv.selector.clone(),
            message: dv.message.clone(),
            mode,
        });
    }

    // Light-only violations (exist in light but not dark)
    for (k, lv) in &light_map {
        if !dark_map.contains_key(k) {
            result.push(DarkContrastViolation {
                selector: lv.selector.clone(),
                message: lv.message.clone(),
                mode: DarkContrastMode::LightOnly,
            });
        }
    }

    result
}

// ─── Static detection ────────────────────────────────────────────────────────

struct StaticDarkModeInfo {
    has_dark_media_query: bool,
    has_class_based_dark_mode: bool,
    color_scheme_css: bool,
    meta_color_scheme: Option<String>,
    meta_theme_color_dark: bool,
    css_custom_properties: u32,
}

async fn detect_static_support(page: &Page) -> Result<StaticDarkModeInfo> {
    let js = r#"
    (() => {
        const result = {
            hasDarkMediaQuery: false,
            hasClassBasedDarkMode: false,
            colorSchemeCss: false,
            metaColorScheme: null,
            metaThemeColorDark: false,
            cssCustomProperties: 0,
        };

        // 1. Scan stylesheets for @media (prefers-color-scheme: dark)
        try {
            for (const sheet of document.styleSheets) {
                let rules;
                try { rules = sheet.cssRules || sheet.rules; } catch (_) { continue; }
                if (!rules) continue;
                for (const rule of rules) {
                    if (rule instanceof CSSMediaRule) {
                        const text = rule.conditionText || rule.media?.mediaText || '';
                        if (text.includes('prefers-color-scheme') && text.includes('dark')) {
                            result.hasDarkMediaQuery = true;
                        }
                    }
                    if (rule instanceof CSSSupportsRule && rule.cssText &&
                        rule.cssText.includes('prefers-color-scheme')) {
                        result.hasDarkMediaQuery = true;
                    }
                }
            }
        } catch(e) {}

        // 2. Check color-scheme on :root / html
        try {
            const rootStyle = window.getComputedStyle(document.documentElement);
            const colorScheme = rootStyle.getPropertyValue('color-scheme').trim();
            if (colorScheme && colorScheme.length > 0) {
                result.colorSchemeCss = true;
            }
        } catch(e) {}

        // 3. Meta color-scheme tag
        try {
            const meta = document.querySelector('meta[name="color-scheme"]');
            if (meta) result.metaColorScheme = meta.getAttribute('content') || '';
        } catch(e) {}

        // 4. Meta theme-color with dark media
        try {
            const metas = document.querySelectorAll('meta[name="theme-color"]');
            for (const m of metas) {
                const media = m.getAttribute('media') || '';
                if (media.includes('prefers-color-scheme') && media.includes('dark')) {
                    result.metaThemeColorDark = true;
                }
            }
        } catch(e) {}

        // 5. Count CSS custom properties with color-related names on :root
        try {
            let count = 0;
            for (const sheet of document.styleSheets) {
                let rules;
                try { rules = sheet.cssRules || sheet.rules; } catch (_) { continue; }
                if (!rules) continue;
                for (const rule of rules) {
                    if (rule instanceof CSSStyleRule &&
                        (rule.selectorText === ':root' || rule.selectorText === 'html')) {
                        const style = rule.style;
                        for (let i = 0; i < style.length; i++) {
                            const prop = style[i];
                            if (prop.startsWith('--') && (
                                prop.includes('color') || prop.includes('bg') ||
                                prop.includes('background') || prop.includes('text') ||
                                prop.includes('foreground') || prop.includes('primary') ||
                                prop.includes('secondary') || prop.includes('accent') ||
                                prop.includes('surface') || prop.includes('on-')
                            )) {
                                count++;
                            }
                        }
                    }
                }
            }
            result.cssCustomProperties = count;
        } catch(e) {}

        // 6. Check for class-based dark mode in CSS selectors
        try {
            const darkClassPatterns = [
                'html.dark', 'body.dark', ':root.dark',
                '[data-theme="dark"]', "[data-theme='dark']",
                '.dark-mode', '.theme-dark'
            ];
            classLoop:
            for (const sheet of document.styleSheets) {
                let rules;
                try { rules = sheet.cssRules || sheet.rules; } catch (_) { continue; }
                if (!rules) continue;
                for (const rule of rules) {
                    if (rule instanceof CSSStyleRule) {
                        const sel = rule.selectorText || '';
                        for (const pattern of darkClassPatterns) {
                            if (sel.includes(pattern)) {
                                result.hasClassBasedDarkMode = true;
                                break classLoop;
                            }
                        }
                    }
                }
            }
        } catch(e) {}

        return JSON.stringify(result);
    })()
    "#;

    let js_result = page
        .evaluate(js)
        .await
        .map_err(|e| AuditError::CdpError(format!("Dark mode detection failed: {e}")))?;

    let json_str = js_result.value().and_then(|v| v.as_str()).unwrap_or("{}");
    let parsed: serde_json::Value = serde_json::from_str(json_str).unwrap_or_default();

    Ok(StaticDarkModeInfo {
        has_dark_media_query: parsed["hasDarkMediaQuery"].as_bool().unwrap_or(false),
        has_class_based_dark_mode: parsed["hasClassBasedDarkMode"].as_bool().unwrap_or(false),
        color_scheme_css: parsed["colorSchemeCss"].as_bool().unwrap_or(false),
        meta_color_scheme: parsed["metaColorScheme"]
            .as_str()
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string()),
        meta_theme_color_dark: parsed["metaThemeColorDark"].as_bool().unwrap_or(false),
        css_custom_properties: parsed["cssCustomProperties"].as_u64().unwrap_or(0) as u32,
    })
}

async fn analyze_print_stylesheet(page: &Page) -> Result<PrintStylesheetAnalysis> {
    let stylesheet_detected = detect_print_stylesheet(page).await?;
    let mut analysis = PrintStylesheetAnalysis {
        stylesheet_detected,
        ..Default::default()
    };

    if let Err(e) = page.emulate_media_type(MediaTypeParams::Print).await {
        warn!("Could not emulate print media: {e}");
        return Ok(analysis);
    }
    analysis.emulation_supported = true;

    if let Err(e) = settle(page).await {
        warn!("Could not wait for print media layout settle: {e}");
    }

    let metrics = evaluate_print_layout(page).await?;
    analysis.interactive_chrome_hidden = metrics.interactive_chrome_hidden;
    analysis.content_not_clipped = metrics.clipped_elements == 0;
    analysis.clipped_elements = metrics.clipped_elements;

    if let Err(e) = page.emulate_media_type(MediaTypeParams::Screen).await {
        warn!("Could not restore screen media: {e}");
    }

    Ok(analysis)
}

async fn analyze_forced_colors(page: &Page) -> Result<ForcedColorsAnalysis> {
    let stylesheet_detected = detect_forced_colors_stylesheet(page).await?;
    let mut analysis = ForcedColorsAnalysis {
        stylesheet_detected,
        forced_color_adjust_count: count_forced_color_adjust(page).await?,
        ..Default::default()
    };

    let feature = MediaFeature {
        name: "forced-colors".to_string(),
        value: "active".to_string(),
    };
    if let Err(e) = page.emulate_media_features(vec![feature]).await {
        warn!("Could not emulate forced colors: {e}");
        return Ok(analysis);
    }
    analysis.emulation_supported = true;

    if let Err(e) = settle(page).await {
        warn!("Could not wait for forced-colors layout settle: {e}");
    }

    let metrics = evaluate_forced_colors_layout(page).await?;
    analysis.active_matches = metrics.active_matches;
    analysis.focus_indicators_visible = metrics.focus_indicators_visible;

    if let Err(e) = page.emulate_media_features(Vec::new()).await {
        warn!("Could not restore media features after forced-colors check: {e}");
    }

    Ok(analysis)
}

async fn analyze_vision_deficiency(
    page: &Page,
    wcag_level: WcagLevel,
) -> Result<VisionDeficiencyAnalysis> {
    if !matches!(wcag_level, WcagLevel::AA | WcagLevel::AAA) {
        return Ok(VisionDeficiencyAnalysis::default());
    }

    let baseline_contrast = ContrastRule::check_with_page(
        page,
        &crate::accessibility::AXTree::default(),
        wcag_level,
        None,
    )
    .await;
    let baseline_use_of_color = check_use_of_color_with_page(page).await;

    let modes = [
        ("protanopia", SetEmulatedVisionDeficiencyType::Protanopia),
        (
            "deuteranopia",
            SetEmulatedVisionDeficiencyType::Deuteranopia,
        ),
        ("tritanopia", SetEmulatedVisionDeficiencyType::Tritanopia),
        (
            "achromatopsia",
            SetEmulatedVisionDeficiencyType::Achromatopsia,
        ),
    ];

    let mut results = Vec::new();
    let mut emulation_supported = false;
    for (label, deficiency) in modes {
        if let Err(e) = page
            .execute(SetEmulatedVisionDeficiencyParams::new(deficiency))
            .await
        {
            warn!("Could not emulate vision deficiency {label}: {e}");
            continue;
        }
        emulation_supported = true;

        if let Err(e) = settle(page).await {
            warn!("Could not wait for vision-deficiency layout settle: {e}");
        }

        let contrast = ContrastRule::check_with_page(
            page,
            &crate::accessibility::AXTree::default(),
            wcag_level,
            None,
        )
        .await;
        let use_of_color = check_use_of_color_with_page(page).await;
        let new_contrast = violations_only_in(&baseline_contrast, &contrast);

        results.push(VisionDeficiencyModeAnalysis {
            mode: label.to_string(),
            contrast_violations: contrast.len() as u32,
            new_contrast_violations: new_contrast.len() as u32,
            use_of_color_violations: use_of_color.len().max(baseline_use_of_color.len()) as u32,
            new_contrast_selectors: new_contrast.into_iter().take(10).collect(),
        });
    }

    if let Err(e) = page
        .execute(SetEmulatedVisionDeficiencyParams::new(
            SetEmulatedVisionDeficiencyType::None,
        ))
        .await
    {
        warn!("Could not restore normal vision emulation: {e}");
    }

    Ok(VisionDeficiencyAnalysis {
        emulation_supported,
        modes: results,
    })
}

fn violations_only_in(baseline: &[Violation], simulated: &[Violation]) -> Vec<String> {
    use std::collections::HashSet;

    fn key(v: &Violation) -> String {
        v.selector
            .as_deref()
            .filter(|selector| !selector.is_empty())
            .unwrap_or(&v.message)
            .to_string()
    }

    let baseline_keys: HashSet<String> = baseline.iter().map(key).collect();
    simulated
        .iter()
        .map(key)
        .filter(|key| !baseline_keys.contains(key))
        .collect()
}

async fn detect_print_stylesheet(page: &Page) -> Result<bool> {
    let js = r#"
    (() => {
        try {
            for (const el of document.querySelectorAll('link[rel~="stylesheet"], style')) {
                const media = (el.getAttribute('media') || '').toLowerCase();
                if (media.includes('print')) return true;
            }
            for (const sheet of document.styleSheets) {
                let rules;
                try { rules = sheet.cssRules || sheet.rules; } catch (_) { continue; }
                if (!rules) continue;
                for (const rule of rules) {
                    const text = (rule.conditionText || rule.media?.mediaText || rule.cssText || '').toLowerCase();
                    if (text.includes('@media print') || text.includes('print')) return true;
                }
            }
        } catch (_) {}
        return false;
    })()
    "#;
    eval_bool(page, js, "Print stylesheet detection failed").await
}

async fn detect_forced_colors_stylesheet(page: &Page) -> Result<bool> {
    let js = r#"
    (() => {
        try {
            for (const sheet of document.styleSheets) {
                let rules;
                try { rules = sheet.cssRules || sheet.rules; } catch (_) { continue; }
                if (!rules) continue;
                for (const rule of rules) {
                    const text = (rule.conditionText || rule.cssText || '').toLowerCase();
                    if (text.includes('forced-colors') || text.includes('forced-color-adjust')) return true;
                }
            }
            for (const el of document.querySelectorAll('[style*="forced-color-adjust" i]')) {
                if (el) return true;
            }
        } catch (_) {}
        return false;
    })()
    "#;
    eval_bool(page, js, "Forced-colors stylesheet detection failed").await
}

async fn count_forced_color_adjust(page: &Page) -> Result<u32> {
    let js = r#"
    (() => {
        let count = 0;
        try {
            for (const el of document.querySelectorAll('body *')) {
                const value = getComputedStyle(el).getPropertyValue('forced-color-adjust');
                if (value && value !== 'auto') count++;
            }
        } catch (_) {}
        return count;
    })()
    "#;
    eval_u32(page, js, "Forced-color-adjust counting failed").await
}

struct PrintLayoutMetrics {
    interactive_chrome_hidden: bool,
    clipped_elements: u32,
}

async fn evaluate_print_layout(page: &Page) -> Result<PrintLayoutMetrics> {
    let js = r#"
    (() => {
        const isVisible = (el) => {
            const style = getComputedStyle(el);
            const rect = el.getBoundingClientRect();
            return style.display !== 'none' && style.visibility !== 'hidden' &&
                style.opacity !== '0' && rect.width > 1 && rect.height > 1;
        };
        const chrome = Array.from(document.querySelectorAll(
            'nav, aside, [role="navigation"], [aria-modal="true"], .cookie, .cookies, .consent, [class*="cookie" i], [id*="cookie" i], button, input, select, textarea'
        ));
        const visibleChrome = chrome.filter(isVisible).length;
        const clipped = Array.from(document.querySelectorAll('main, article, section, [role="main"], p, h1, h2, h3'))
            .filter(isVisible)
            .filter((el) => {
                const style = getComputedStyle(el);
                if (!['hidden', 'clip', 'scroll', 'auto'].includes(style.overflow) &&
                    !['hidden', 'clip', 'scroll', 'auto'].includes(style.overflowY)) {
                    return false;
                }
                return el.scrollHeight > el.clientHeight + 2 || el.scrollWidth > el.clientWidth + 2;
            }).length;
        return JSON.stringify({
            interactiveChromeHidden: visibleChrome === 0,
            clippedElements: clipped
        });
    })()
    "#;
    let value = eval_json(page, js, "Print layout evaluation failed").await?;
    Ok(PrintLayoutMetrics {
        interactive_chrome_hidden: value["interactiveChromeHidden"].as_bool().unwrap_or(false),
        clipped_elements: value["clippedElements"].as_u64().unwrap_or(0) as u32,
    })
}

struct ForcedColorsMetrics {
    active_matches: bool,
    focus_indicators_visible: bool,
}

async fn evaluate_forced_colors_layout(page: &Page) -> Result<ForcedColorsMetrics> {
    let js = r#"
    (() => {
        const active = matchMedia('(forced-colors: active)').matches;
        const focusables = Array.from(document.querySelectorAll(
            'a[href], button, input, select, textarea, summary, [tabindex]:not([tabindex="-1"])'
        )).filter((el) => {
            const style = getComputedStyle(el);
            const rect = el.getBoundingClientRect();
            return !el.disabled && style.display !== 'none' && style.visibility !== 'hidden' &&
                rect.width > 1 && rect.height > 1;
        }).slice(0, 40);
        const visible = focusables.length === 0 || focusables.every((el) => {
            const style = getComputedStyle(el);
            const border = parseFloat(style.borderTopWidth || '0') +
                parseFloat(style.borderRightWidth || '0') +
                parseFloat(style.borderBottomWidth || '0') +
                parseFloat(style.borderLeftWidth || '0');
            const outline = parseFloat(style.outlineWidth || '0');
            const bg = style.backgroundColor && style.backgroundColor !== 'rgba(0, 0, 0, 0)' && style.backgroundColor !== 'transparent';
            return border > 0 || outline > 0 || bg;
        });
        return JSON.stringify({ activeMatches: active, focusIndicatorsVisible: visible });
    })()
    "#;
    let value = eval_json(page, js, "Forced-colors layout evaluation failed").await?;
    Ok(ForcedColorsMetrics {
        active_matches: value["activeMatches"].as_bool().unwrap_or(false),
        focus_indicators_visible: value["focusIndicatorsVisible"].as_bool().unwrap_or(false),
    })
}

async fn eval_bool(page: &Page, js: &str, context: &str) -> Result<bool> {
    let js_result = page
        .evaluate(js)
        .await
        .map_err(|e| AuditError::CdpError(format!("{context}: {e}")))?;
    Ok(js_result.value().and_then(|v| v.as_bool()).unwrap_or(false))
}

async fn eval_u32(page: &Page, js: &str, context: &str) -> Result<u32> {
    let js_result = page
        .evaluate(js)
        .await
        .map_err(|e| AuditError::CdpError(format!("{context}: {e}")))?;
    Ok(js_result.value().and_then(|v| v.as_u64()).unwrap_or(0) as u32)
}

async fn eval_json(page: &Page, js: &str, context: &str) -> Result<serde_json::Value> {
    let js_result = page
        .evaluate(js)
        .await
        .map_err(|e| AuditError::CdpError(format!("{context}: {e}")))?;
    let json_str = js_result.value().and_then(|v| v.as_str()).unwrap_or("{}");
    Ok(serde_json::from_str(json_str).unwrap_or_default())
}

// ─── Contrast comparison ─────────────────────────────────────────────────────

/// Runs contrast checks in light mode (current state) and dark mode (after CDP emulation).
/// Returns `(light_violations, dark_violations)` as full Violation objects.
async fn compare_contrast(page: &Page, level: WcagLevel) -> (Vec<Violation>, Vec<Violation>) {
    let light_violations =
        ContrastRule::check_with_page(page, &crate::accessibility::AXTree::default(), level, None)
            .await;

    let dark_feature = MediaFeature {
        name: "prefers-color-scheme".to_string(),
        value: "dark".to_string(),
    };
    if let Err(e) = page.emulate_media_features(vec![dark_feature]).await {
        warn!("Could not emulate dark mode: {e}");
        return (light_violations, Vec::new());
    }

    if let Err(e) = settle(page).await {
        warn!("Could not wait for dark mode layout settle: {e}");
    }

    let dark_violations =
        ContrastRule::check_with_page(page, &crate::accessibility::AXTree::default(), level, None)
            .await;

    let light_feature = MediaFeature {
        name: "prefers-color-scheme".to_string(),
        value: "light".to_string(),
    };
    if let Err(e) = page.emulate_media_features(vec![light_feature]).await {
        warn!("Could not restore light mode: {e}");
    }

    (light_violations, dark_violations)
}

// ─── Scoring ─────────────────────────────────────────────────────────────────

fn compute_score(
    info: &StaticDarkModeInfo,
    dark_contrast_count: u32,
    dark_only: u32,
    print: &PrintStylesheetAnalysis,
    forced_colors: &ForcedColorsAnalysis,
    vision_deficiency: &VisionDeficiencyAnalysis,
) -> u32 {
    if !info.has_dark_media_query && !info.has_class_based_dark_mode {
        let mut score = 50;
        if print.stylesheet_detected {
            score += 5;
        }
        if forced_colors.stylesheet_detected {
            score += 5;
        }
        return score;
    }
    if !info.has_dark_media_query && info.has_class_based_dark_mode {
        let mut score = 65;
        if print.stylesheet_detected {
            score += 5;
        }
        if forced_colors.stylesheet_detected {
            score += 5;
        }
        return score.min(75);
    }

    let mut score: i32 = 70;

    if info.color_scheme_css {
        score += 10;
    }
    if info.meta_color_scheme.is_some() {
        score += 10;
    }
    if info.meta_theme_color_dark {
        score += 5;
    }
    if info.css_custom_properties >= 5 {
        score += 5;
    } else if info.css_custom_properties >= 3 {
        score += 3;
    }

    // Cap contrast penalties so structural implementation still gets credit.
    // Dark-mode regressions (dark_only) are penalised harder than pre-existing issues.
    let dark_only_penalty = ((dark_only as i32) * 5).min(20);
    let both_penalty = ((dark_contrast_count.saturating_sub(dark_only) as i32) * 2).min(10);
    score -= dark_only_penalty + both_penalty;

    if print.stylesheet_detected && print.content_not_clipped {
        score += 3;
    } else if !print.stylesheet_detected {
        score -= 3;
    }
    if forced_colors.stylesheet_detected && forced_colors.focus_indicators_visible {
        score += 3;
    } else if !forced_colors.stylesheet_detected {
        score -= 3;
    }
    let vision_penalty = vision_deficiency
        .modes
        .iter()
        .map(|mode| mode.new_contrast_violations)
        .sum::<u32>()
        .min(10) as i32;
    score -= vision_penalty;

    score.clamp(0, 100) as u32
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wcag::Severity;

    fn make_info(
        has_dark: bool,
        color_scheme: bool,
        meta: bool,
        theme_color: bool,
        props: u32,
    ) -> StaticDarkModeInfo {
        StaticDarkModeInfo {
            has_dark_media_query: has_dark,
            has_class_based_dark_mode: false,
            color_scheme_css: color_scheme,
            meta_color_scheme: if meta {
                Some("dark light".into())
            } else {
                None
            },
            meta_theme_color_dark: theme_color,
            css_custom_properties: props,
        }
    }

    fn make_violation(selector: &str, message: &str) -> Violation {
        let mut v = Violation::new(
            "1.4.3",
            "Contrast (Minimum)",
            crate::cli::WcagLevel::AA,
            Severity::High,
            message,
            "node-1",
        );
        v.selector = Some(selector.to_string());
        v
    }

    fn print_default() -> PrintStylesheetAnalysis {
        PrintStylesheetAnalysis::default()
    }

    fn forced_default() -> ForcedColorsAnalysis {
        ForcedColorsAnalysis::default()
    }

    fn vision_default() -> VisionDeficiencyAnalysis {
        VisionDeficiencyAnalysis::default()
    }

    // ── Score tests ───────────────────────────────────────────────────────────

    #[test]
    fn score_no_dark_mode_is_50() {
        assert_eq!(
            compute_score(
                &make_info(false, false, false, false, 0),
                0,
                0,
                &print_default(),
                &forced_default(),
                &vision_default()
            ),
            50
        );
    }

    #[test]
    fn score_full_implementation_no_violations_is_100() {
        assert_eq!(
            compute_score(
                &make_info(true, true, true, true, 6),
                0,
                0,
                &PrintStylesheetAnalysis {
                    stylesheet_detected: true,
                    content_not_clipped: true,
                    ..Default::default()
                },
                &ForcedColorsAnalysis {
                    stylesheet_detected: true,
                    focus_indicators_visible: true,
                    ..Default::default()
                },
                &vision_default()
            ),
            100
        );
    }

    #[test]
    fn score_minimal_dark_mode_with_print_and_forced_colors_bonus_is_76() {
        assert_eq!(
            compute_score(
                &make_info(true, false, false, false, 0),
                0,
                0,
                &PrintStylesheetAnalysis {
                    stylesheet_detected: true,
                    content_not_clipped: true,
                    ..Default::default()
                },
                &ForcedColorsAnalysis {
                    stylesheet_detected: true,
                    focus_indicators_visible: true,
                    ..Default::default()
                },
                &vision_default()
            ),
            76
        );
    }

    #[test]
    fn score_dark_only_violations_reduce_score() {
        let score = compute_score(
            &make_info(true, true, true, true, 6),
            3,
            3,
            &print_default(),
            &forced_default(),
            &vision_default(),
        );
        assert!(score < 100, "score should be penalised, got {score}");
        assert!(score >= 70, "should retain base points, got {score}");
    }

    #[test]
    fn score_many_both_mode_violations_capped_at_minus_10() {
        // 101 violations in both modes: penalty capped at 10 → 95 - 10 = 85
        let score = compute_score(
            &make_info(true, true, true, false, 6),
            101,
            0,
            &print_default(),
            &forced_default(),
            &vision_default(),
        );
        assert_eq!(score, 79);
    }

    #[test]
    fn score_many_dark_only_violations_capped_at_minus_20() {
        // 101 dark-only violations: penalty capped at 20 → 95 - 20 = 75
        let score = compute_score(
            &make_info(true, true, true, false, 6),
            101,
            101,
            &print_default(),
            &forced_default(),
            &vision_default(),
        );
        assert_eq!(score, 69);
    }

    // ── Issue generation tests ────────────────────────────────────────────────

    #[test]
    fn issues_no_dark_mode_generates_exactly_one_support_issue() {
        let issues = build_issues(
            &make_info(false, false, false, false, 0),
            0,
            0,
            0,
            &[],
            &print_default(),
            &forced_default(),
            &vision_default(),
        );
        assert!(matches!(
            issues[0].kind,
            DarkModeIssueKind::NoDarkModeSupport
        ));
    }

    #[test]
    fn issues_dark_mode_with_zero_violations_no_contrast_issue() {
        let issues = build_issues(
            &make_info(true, true, true, false, 5),
            0,
            0,
            0,
            &[],
            &print_default(),
            &forced_default(),
            &vision_default(),
        );
        assert!(
            issues
                .iter()
                .all(|i| !matches!(i.kind, DarkModeIssueKind::DarkModeContrastFailure)),
            "no contrast issue expected when violations=0"
        );
    }

    #[test]
    fn issues_both_mode_violations_generate_contrast_issue() {
        // 5 violations in both modes — the bug that was missing before
        let violations: Vec<DarkContrastViolation> = (0..5)
            .map(|i| DarkContrastViolation {
                selector: Some(format!("p.text-{i}")),
                message: "Low contrast".into(),
                mode: DarkContrastMode::LightAndDark,
            })
            .collect();

        let issues = build_issues(
            &make_info(true, true, true, false, 5),
            5,
            0,
            0,
            &violations,
            &print_default(),
            &forced_default(),
            &vision_default(),
        );
        let contrast_issues: Vec<_> = issues
            .iter()
            .filter(|i| matches!(i.kind, DarkModeIssueKind::DarkModeContrastFailure))
            .collect();

        assert!(
            !contrast_issues.is_empty(),
            "expected contrast issue when both-mode violations > 0"
        );
        assert!(
            contrast_issues[0].description.contains('5')
                || contrast_issues[0].description.contains("5 "),
            "description should mention count, got: {}",
            contrast_issues[0].description
        );
    }

    #[test]
    fn issues_dark_only_violations_generate_regression_issue() {
        let violations: Vec<DarkContrastViolation> = (0..3)
            .map(|i| DarkContrastViolation {
                selector: Some(format!("button.dark-{i}")),
                message: "Low contrast in dark mode".into(),
                mode: DarkContrastMode::DarkOnly,
            })
            .collect();

        let issues = build_issues(
            &make_info(true, true, true, false, 5),
            3,
            3,
            0,
            &violations,
            &print_default(),
            &forced_default(),
            &vision_default(),
        );
        let regression = issues
            .iter()
            .find(|i| matches!(i.kind, DarkModeIssueKind::DarkModeContrastFailure));

        assert!(
            regression.is_some(),
            "expected regression issue for dark-only violations"
        );
        let desc = &regression.unwrap().description;
        assert!(
            desc.contains("dark mode"),
            "should mention dark mode regression"
        );
        assert_eq!(regression.unwrap().severity, "high");
    }

    #[test]
    fn issues_contrast_issue_lists_selectors_in_description() {
        let violations: Vec<DarkContrastViolation> = vec![
            DarkContrastViolation {
                selector: Some("h1.hero".into()),
                message: "Low contrast".into(),
                mode: DarkContrastMode::LightAndDark,
            },
            DarkContrastViolation {
                selector: Some("p.caption".into()),
                message: "Low contrast".into(),
                mode: DarkContrastMode::LightAndDark,
            },
        ];

        let issues = build_issues(
            &make_info(true, true, true, false, 5),
            2,
            0,
            0,
            &violations,
            &print_default(),
            &forced_default(),
            &vision_default(),
        );
        let contrast_issue = issues
            .iter()
            .find(|i| matches!(i.kind, DarkModeIssueKind::DarkModeContrastFailure))
            .expect("contrast issue must exist");

        assert!(
            contrast_issue.description.contains("h1.hero"),
            "selector should appear in description: {}",
            contrast_issue.description
        );
    }

    #[test]
    fn classify_both_modes_correctly() {
        let light = vec![
            make_violation("#a", "low contrast"),
            make_violation("#b", "low contrast"),
        ];
        let dark = vec![
            make_violation("#a", "low contrast"),
            make_violation("#c", "low contrast"),
        ];

        let result = classify_contrast_violations(&light, &dark);

        let a = result
            .iter()
            .find(|v| v.selector.as_deref() == Some("#a"))
            .unwrap();
        assert_eq!(a.mode, DarkContrastMode::LightAndDark);

        let b = result
            .iter()
            .find(|v| v.selector.as_deref() == Some("#b"))
            .unwrap();
        assert_eq!(b.mode, DarkContrastMode::LightOnly);

        let c = result
            .iter()
            .find(|v| v.selector.as_deref() == Some("#c"))
            .unwrap();
        assert_eq!(c.mode, DarkContrastMode::DarkOnly);
    }

    #[test]
    fn classify_no_double_counting() {
        let shared = make_violation("#x", "low contrast");
        let light = vec![shared.clone(), shared.clone()]; // duplicate in light
        let dark = vec![shared.clone()];

        let result = classify_contrast_violations(&light, &dark);
        // HashMap dedup: only one entry per unique key
        let x_count = result
            .iter()
            .filter(|v| v.selector.as_deref() == Some("#x"))
            .count();
        assert_eq!(x_count, 1, "same selector must not be double-counted");
    }

    #[test]
    fn light_only_violations_generate_low_severity_issue() {
        let violations = vec![DarkContrastViolation {
            selector: Some("p.legacy".into()),
            message: "Low contrast".into(),
            mode: DarkContrastMode::LightOnly,
        }];

        let issues = build_issues(
            &make_info(true, true, true, false, 5),
            0,
            0,
            1,
            &violations,
            &print_default(),
            &forced_default(),
            &vision_default(),
        );
        let light_issue = issues
            .iter()
            .find(|i| matches!(i.kind, DarkModeIssueKind::DarkModeContrastFailure));

        assert!(light_issue.is_some());
        assert_eq!(light_issue.unwrap().severity, "low");
    }

    #[test]
    fn score_class_based_dark_mode_is_65() {
        let info = StaticDarkModeInfo {
            has_dark_media_query: false,
            has_class_based_dark_mode: true,
            color_scheme_css: false,
            meta_color_scheme: None,
            meta_theme_color_dark: false,
            css_custom_properties: 0,
        };
        assert_eq!(
            compute_score(
                &info,
                0,
                0,
                &print_default(),
                &forced_default(),
                &vision_default()
            ),
            65
        );
    }

    #[test]
    fn issues_class_based_dark_mode_not_reported_as_no_support() {
        let info = StaticDarkModeInfo {
            has_dark_media_query: false,
            has_class_based_dark_mode: true,
            color_scheme_css: false,
            meta_color_scheme: None,
            meta_theme_color_dark: false,
            css_custom_properties: 0,
        };
        let issues = build_issues(
            &info,
            0,
            0,
            0,
            &[],
            &print_default(),
            &forced_default(),
            &vision_default(),
        );
        assert!(
            issues
                .iter()
                .all(|i| !matches!(i.kind, DarkModeIssueKind::NoDarkModeSupport)),
            "class-based dark mode should not be reported as NoDarkModeSupport"
        );
        assert!(
            issues
                .iter()
                .any(|i| matches!(i.kind, DarkModeIssueKind::IncompleteImplementation)),
            "class-based dark mode should report IncompleteImplementation"
        );
    }

    #[test]
    fn issues_missing_print_stylesheet_are_reported() {
        let issues = build_issues(
            &make_info(true, true, true, false, 5),
            0,
            0,
            0,
            &[],
            &PrintStylesheetAnalysis::default(),
            &ForcedColorsAnalysis {
                stylesheet_detected: true,
                focus_indicators_visible: true,
                ..Default::default()
            },
            &vision_default(),
        );

        assert!(issues
            .iter()
            .any(|i| matches!(i.kind, DarkModeIssueKind::NoPrintStylesheet)));
    }

    #[test]
    fn issues_forced_colors_focus_risk_are_reported() {
        let issues = build_issues(
            &make_info(true, true, true, false, 5),
            0,
            0,
            0,
            &[],
            &PrintStylesheetAnalysis {
                stylesheet_detected: true,
                content_not_clipped: true,
                interactive_chrome_hidden: true,
                ..Default::default()
            },
            &ForcedColorsAnalysis {
                stylesheet_detected: true,
                focus_indicators_visible: false,
                ..Default::default()
            },
            &vision_default(),
        );

        assert!(issues
            .iter()
            .any(|i| matches!(i.kind, DarkModeIssueKind::ForcedColorsFocusRisk)));
    }

    #[test]
    fn issues_vision_deficiency_new_contrast_are_reported() {
        let issues = build_issues(
            &make_info(true, true, true, false, 5),
            0,
            0,
            0,
            &[],
            &PrintStylesheetAnalysis {
                stylesheet_detected: true,
                content_not_clipped: true,
                interactive_chrome_hidden: true,
                ..Default::default()
            },
            &ForcedColorsAnalysis {
                stylesheet_detected: true,
                focus_indicators_visible: true,
                ..Default::default()
            },
            &VisionDeficiencyAnalysis {
                emulation_supported: true,
                modes: vec![VisionDeficiencyModeAnalysis {
                    mode: "protanopia".to_string(),
                    contrast_violations: 2,
                    new_contrast_violations: 2,
                    use_of_color_violations: 0,
                    new_contrast_selectors: vec![".danger".to_string()],
                }],
            },
        );

        assert!(issues.iter().any(|i| matches!(
            i.kind,
            DarkModeIssueKind::ColorVisionDeficiencyContrastFailure
        )));
    }
}
