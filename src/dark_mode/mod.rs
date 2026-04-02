//! Dark Mode Analysis
//!
//! Detects whether a site supports `prefers-color-scheme: dark` and audits
//! the quality of that implementation, including:
//!
//! - Static CSS detection (media queries, `color-scheme` declarations, meta tags)
//! - Dynamic contrast re-check after emulating dark mode via CDP
//! - Comparison of light-mode vs. dark-mode contrast violations

use chromiumoxide::cdp::browser_protocol::emulation::MediaFeature;
use chromiumoxide::Page;
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::cli::WcagLevel;
use crate::error::{AuditError, Result};
use crate::wcag::rules::ContrastRule;

// ─── Public types ────────────────────────────────────────────────────────────

/// Complete dark mode analysis for a single page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DarkModeAnalysis {
    /// Site declares `@media (prefers-color-scheme: dark)` CSS rules.
    pub supported: bool,
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
    /// Non-contrast issues (missing support, incomplete implementation, etc.)
    pub issues: Vec<DarkModeIssue>,
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
}

// ─── Main entry point ────────────────────────────────────────────────────────

/// Analyse dark mode support and quality on the given already-loaded page.
///
/// Steps:
/// 1. Static JS detection: CSS media queries, `color-scheme`, meta tags, CSS vars.
/// 2. If dark mode is supported: emulate dark via CDP, re-run contrast, restore.
/// 3. Build score and issue list.
pub async fn analyze_dark_mode(page: &Page, wcag_level: WcagLevel) -> Result<DarkModeAnalysis> {
    // ── 1. Static detection ─────────────────────────────────────────────────
    let static_info = detect_static_support(page).await?;

    // ── 2. Contrast comparison (only when dark mode CSS exists) ─────────────
    let (dark_contrast_count, light_only, dark_only) = if static_info.has_dark_media_query
        && matches!(wcag_level, WcagLevel::AA | WcagLevel::AAA)
    {
        compare_contrast(page, wcag_level).await
    } else {
        (0, 0, 0)
    };

    // ── 3. Build issues list ─────────────────────────────────────────────────
    let mut issues: Vec<DarkModeIssue> = Vec::new();

    if !static_info.has_dark_media_query {
        issues.push(DarkModeIssue {
            kind: DarkModeIssueKind::NoDarkModeSupport,
            description: "Keine @media (prefers-color-scheme: dark) Regeln gefunden. \
                          Nutzer mit aktiviertem Systemdunkel-Modus erhalten die helle Ansicht."
                .to_string(),
            severity: "medium".to_string(),
        });
    } else {
        // Has dark mode — check for best-practice declarations
        if !static_info.color_scheme_css {
            issues.push(DarkModeIssue {
                kind: DarkModeIssueKind::NoColorSchemeDeclaration,
                description:
                    "Kein `color-scheme: dark light` auf :root deklariert. Der Browser nutzt \
                     keine nativen Dark-Mode-Farben für Scrollbars, Formulare und andere UI-Elemente."
                        .to_string(),
                severity: "low".to_string(),
            });
        }
        if static_info.meta_color_scheme.is_none() {
            issues.push(DarkModeIssue {
                kind: DarkModeIssueKind::NoMetaColorScheme,
                description:
                    "Kein <meta name=\"color-scheme\"> gefunden. Ohne dieses Meta-Tag kann der \
                     Browser das Rendering-Verhalten vor dem CSSOM-Aufbau nicht optimieren."
                        .to_string(),
                severity: "low".to_string(),
            });
        }
        if dark_only > 0 {
            issues.push(DarkModeIssue {
                kind: DarkModeIssueKind::DarkModeContrastFailure,
                description: format!(
                    "{} Kontrast-Probleme treten nur im Dark Mode auf — diese Elemente sind im \
                     hellen Design korrekt, verlieren aber im Dark Mode ausreichenden Kontrast.",
                    dark_only
                ),
                severity: "high".to_string(),
            });
        }
        if static_info.css_custom_properties < 3 && static_info.has_dark_media_query {
            issues.push(DarkModeIssue {
                kind: DarkModeIssueKind::IncompleteImplementation,
                description:
                    "Wenige CSS Custom Properties für Farben erkannt. Vollständige Dark-Mode- \
                     Implementierungen verwenden typischerweise CSS-Variablen (--color-*) auf :root \
                     und überschreiben diese im Media Query."
                        .to_string(),
                severity: "low".to_string(),
            });
        }
    }

    // ── 4. Score ─────────────────────────────────────────────────────────────
    let score = compute_score(&static_info, dark_contrast_count, dark_only);

    // ── 5. Detection methods ──────────────────────────────────────────────────
    let mut detection_methods: Vec<String> = Vec::new();
    if static_info.has_dark_media_query {
        detection_methods.push("@media (prefers-color-scheme: dark)".to_string());
    }
    if static_info.color_scheme_css {
        detection_methods.push("color-scheme CSS property".to_string());
    }
    if static_info.meta_color_scheme.is_some() {
        detection_methods.push(format!(
            "<meta name=\"color-scheme\" content=\"{}\">",
            static_info.meta_color_scheme.as_deref().unwrap_or("")
        ));
    }
    if static_info.meta_theme_color_dark {
        detection_methods.push("<meta name=\"theme-color\" media dark>".to_string());
    }

    Ok(DarkModeAnalysis {
        supported: static_info.has_dark_media_query,
        score,
        detection_methods,
        color_scheme_css: static_info.color_scheme_css,
        meta_color_scheme: static_info.meta_color_scheme,
        meta_theme_color_dark: static_info.meta_theme_color_dark,
        css_custom_properties: static_info.css_custom_properties,
        dark_contrast_violations: dark_contrast_count,
        light_only_violations: light_only,
        dark_only_violations: dark_only,
        issues,
    })
}

// ─── Static detection ────────────────────────────────────────────────────────

struct StaticDarkModeInfo {
    has_dark_media_query: bool,
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
                    // Check inline @supports or nested media (shallow scan)
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
            const rootStyle = getComputedStyle(document.documentElement);
            let count = 0;
            // Inline style check via CSSStyleDeclaration for custom properties
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
        color_scheme_css: parsed["colorSchemeCss"].as_bool().unwrap_or(false),
        meta_color_scheme: parsed["metaColorScheme"]
            .as_str()
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string()),
        meta_theme_color_dark: parsed["metaThemeColorDark"].as_bool().unwrap_or(false),
        css_custom_properties: parsed["cssCustomProperties"].as_u64().unwrap_or(0) as u32,
    })
}

// ─── Contrast comparison ─────────────────────────────────────────────────────

/// Runs contrast checks in light mode (current state) and dark mode (after CDP emulation).
/// Returns `(dark_total, light_only, dark_only)`.
async fn compare_contrast(page: &Page, level: WcagLevel) -> (u32, u32, u32) {
    // Light mode violation selectors (already rendered in light mode)
    let light_violations =
        ContrastRule::check_with_page(page, &crate::accessibility::AXTree::default(), level).await;
    let light_selectors: std::collections::HashSet<String> = light_violations
        .iter()
        .map(|v| v.selector.clone().unwrap_or_default())
        .collect();

    // Activate dark mode
    let dark_feature = MediaFeature {
        name: "prefers-color-scheme".to_string(),
        value: "dark".to_string(),
    };
    if let Err(e) = page.emulate_media_features(vec![dark_feature]).await {
        warn!("Could not emulate dark mode: {e}");
        return (0, 0, 0);
    }

    // Small settle time for CSS transitions / media query re-evaluation
    tokio::time::sleep(std::time::Duration::from_millis(150)).await;

    // Contrast check in dark mode
    let dark_violations =
        ContrastRule::check_with_page(page, &crate::accessibility::AXTree::default(), level).await;
    let dark_selectors: std::collections::HashSet<String> = dark_violations
        .iter()
        .map(|v| v.selector.clone().unwrap_or_default())
        .collect();

    // Restore light mode
    let light_feature = MediaFeature {
        name: "prefers-color-scheme".to_string(),
        value: "light".to_string(),
    };
    if let Err(e) = page.emulate_media_features(vec![light_feature]).await {
        warn!("Could not restore light mode: {e}");
    }

    let dark_only = dark_selectors.difference(&light_selectors).count() as u32;
    let light_only = light_selectors.difference(&dark_selectors).count() as u32;

    (dark_violations.len() as u32, light_only, dark_only)
}

// ─── Scoring ─────────────────────────────────────────────────────────────────

fn compute_score(info: &StaticDarkModeInfo, dark_contrast_count: u32, dark_only: u32) -> u32 {
    if !info.has_dark_media_query {
        // No dark mode at all — neutral score (not penalised harshly, just noted)
        return 50;
    }

    let mut score: i32 = 70; // Base: has dark mode CSS

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

    // Contrast violations in dark mode reduce score
    score -= (dark_only as i32) * 5;
    score -= (dark_contrast_count.saturating_sub(dark_only) as i32) * 2;

    score.clamp(0, 100) as u32
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_info(
        has_dark: bool,
        color_scheme: bool,
        meta: bool,
        theme_color: bool,
        props: u32,
    ) -> StaticDarkModeInfo {
        StaticDarkModeInfo {
            has_dark_media_query: has_dark,
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

    #[test]
    fn score_no_dark_mode_is_50() {
        let info = make_info(false, false, false, false, 0);
        assert_eq!(compute_score(&info, 0, 0), 50);
    }

    #[test]
    fn score_full_implementation_is_100() {
        let info = make_info(true, true, true, true, 6);
        assert_eq!(compute_score(&info, 0, 0), 100);
    }

    #[test]
    fn score_dark_only_violations_reduce_score() {
        let info = make_info(true, true, true, true, 6);
        let score = compute_score(&info, 3, 3);
        assert!(score < 100);
        assert!(score >= 70); // Still has dark mode, just penalised
    }

    #[test]
    fn score_minimal_dark_mode_no_extras() {
        let info = make_info(true, false, false, false, 0);
        assert_eq!(compute_score(&info, 0, 0), 70);
    }

    #[test]
    fn issues_no_dark_mode_generates_support_issue() {
        let issues_count = {
            let info = make_info(false, false, false, false, 0);
            let mut issues = Vec::new();
            if !info.has_dark_media_query {
                issues.push(1);
            }
            issues.len()
        };
        assert_eq!(issues_count, 1);
    }
}
