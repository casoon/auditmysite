//! Dark Mode Analysis
//!
//! Detects whether a site supports `prefers-color-scheme: dark` and audits
//! the quality of that implementation, including:
//!
//! - Static CSS detection (media queries, `color-scheme` declarations, meta tags)
//! - Dynamic contrast re-check after emulating dark mode via CDP
//! - Comparison of light-mode vs. dark-mode contrast violations
//! - Per-element contrast violation details for dark mode (selector, message, mode)

use chromiumoxide::cdp::browser_protocol::emulation::MediaFeature;
use chromiumoxide::Page;
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::cli::WcagLevel;
use crate::error::{AuditError, Result};
use crate::wcag::rules::ContrastRule;
use crate::wcag::Violation;

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
    /// Per-element contrast violation details (light-and-dark, dark-only, light-only).
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub contrast_violations: Vec<DarkContrastViolation>,
    /// Non-contrast issues (missing support, incomplete implementation, contrast summary).
    pub issues: Vec<DarkModeIssue>,
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
    );

    // ── 5. Score ─────────────────────────────────────────────────────────────
    let score = compute_score(&static_info, dark_contrast_count, dark_only);

    // ── 6. Detection methods ──────────────────────────────────────────────────
    // Only include signals that actually implement dark styling, not mere hints.
    // color-scheme CSS / meta_color_scheme tell the browser which scheme the site
    // prefers but do not apply dark styles — they stay as separate boolean fields.
    let mut detection_methods: Vec<String> = Vec::new();
    if static_info.has_dark_media_query {
        detection_methods.push("@media (prefers-color-scheme: dark)".to_string());
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
        contrast_violations,
        issues,
    })
}

// ─── Issue generation ─────────────────────────────────────────────────────────

fn build_issues(
    info: &StaticDarkModeInfo,
    dark_contrast_count: u32,
    dark_only: u32,
    light_only: u32,
    contrast_violations: &[DarkContrastViolation],
) -> Vec<DarkModeIssue> {
    let mut issues: Vec<DarkModeIssue> = Vec::new();

    if !info.has_dark_media_query {
        issues.push(DarkModeIssue {
            kind: DarkModeIssueKind::NoDarkModeSupport,
            description: "Keine @media (prefers-color-scheme: dark) Regeln gefunden. \
                          Nutzer mit aktiviertem Systemdunkel-Modus erhalten die helle Ansicht."
                .to_string(),
            severity: "medium".to_string(),
        });
        return issues;
    }

    // ── Structural best-practice issues ──────────────────────────────────────
    if !info.color_scheme_css {
        issues.push(DarkModeIssue {
            kind: DarkModeIssueKind::NoColorSchemeDeclaration,
            description: "Kein `color-scheme: dark light` auf :root deklariert. Der Browser nutzt \
                          keine nativen Dark-Mode-Farben für Scrollbars, Formulare und andere UI-Elemente."
                .to_string(),
            severity: "low".to_string(),
        });
    }
    if info.meta_color_scheme.is_none() {
        issues.push(DarkModeIssue {
            kind: DarkModeIssueKind::NoMetaColorScheme,
            description:
                "Kein <meta name=\"color-scheme\"> gefunden. Ohne dieses Meta-Tag kann der \
                          Browser das Rendering-Verhalten vor dem CSSOM-Aufbau nicht optimieren."
                    .to_string(),
            severity: "low".to_string(),
        });
    }
    if info.css_custom_properties < 3 {
        issues.push(DarkModeIssue {
            kind: DarkModeIssueKind::IncompleteImplementation,
            description: "Wenige CSS Custom Properties für Farben erkannt. Vollständige Dark-Mode-\
                          Implementierungen verwenden typischerweise CSS-Variablen (--color-*) auf :root \
                          und überschreiben diese im Media Query."
                .to_string(),
            severity: "low".to_string(),
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
                "{dark_only} Element(e) verlieren Kontrast im Dark Mode (Dark-Mode-Regression). \
                 Diese Elemente sind im Light Mode korrekt, unterschreiten aber im Dark Mode \
                 den WCAG-Mindestkontrastwert.{selectors}"
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
                "{both} Element(e) haben unzureichenden Kontrast in beiden Farbmodi (Light und Dark). \
                 Die Dark-Mode-Implementierung ändert die Farben dieser Elemente nicht ausreichend.{selectors}"
            ),
            severity: "high".to_string(),
        });
    }

    // Light-only: informational (dark mode actually improves these)
    if light_only > 0 {
        issues.push(DarkModeIssue {
            kind: DarkModeIssueKind::DarkModeContrastFailure,
            description: format!(
                "{light_only} Element(e) haben Kontrast-Probleme nur im Light Mode — der Dark Mode \
                 behebt diese. Erwägen Sie, die Light-Mode-Farben entsprechend anzupassen."
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

    tokio::time::sleep(std::time::Duration::from_millis(150)).await;

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

fn compute_score(info: &StaticDarkModeInfo, dark_contrast_count: u32, dark_only: u32) -> u32 {
    if !info.has_dark_media_query {
        return 50;
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

    // ── Score tests ───────────────────────────────────────────────────────────

    #[test]
    fn score_no_dark_mode_is_50() {
        assert_eq!(
            compute_score(&make_info(false, false, false, false, 0), 0, 0),
            50
        );
    }

    #[test]
    fn score_full_implementation_no_violations_is_100() {
        assert_eq!(
            compute_score(&make_info(true, true, true, true, 6), 0, 0),
            100
        );
    }

    #[test]
    fn score_minimal_dark_mode_no_extras_is_70() {
        assert_eq!(
            compute_score(&make_info(true, false, false, false, 0), 0, 0),
            70
        );
    }

    #[test]
    fn score_dark_only_violations_reduce_score() {
        let score = compute_score(&make_info(true, true, true, true, 6), 3, 3);
        assert!(score < 100, "score should be penalised, got {score}");
        assert!(score >= 70, "should retain base points, got {score}");
    }

    #[test]
    fn score_many_both_mode_violations_capped_at_minus_10() {
        // 101 violations in both modes: penalty capped at 10 → 95 - 10 = 85
        let score = compute_score(&make_info(true, true, true, false, 6), 101, 0);
        assert_eq!(score, 85);
    }

    #[test]
    fn score_many_dark_only_violations_capped_at_minus_20() {
        // 101 dark-only violations: penalty capped at 20 → 95 - 20 = 75
        let score = compute_score(&make_info(true, true, true, false, 6), 101, 101);
        assert_eq!(score, 75);
    }

    // ── Issue generation tests ────────────────────────────────────────────────

    #[test]
    fn issues_no_dark_mode_generates_exactly_one_support_issue() {
        let issues = build_issues(&make_info(false, false, false, false, 0), 0, 0, 0, &[]);
        assert_eq!(issues.len(), 1);
        assert!(matches!(
            issues[0].kind,
            DarkModeIssueKind::NoDarkModeSupport
        ));
    }

    #[test]
    fn issues_dark_mode_with_zero_violations_no_contrast_issue() {
        let issues = build_issues(&make_info(true, true, true, false, 5), 0, 0, 0, &[]);
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

        let issues = build_issues(&make_info(true, true, true, false, 5), 5, 0, 0, &violations);
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

        let issues = build_issues(&make_info(true, true, true, false, 5), 3, 3, 0, &violations);
        let regression = issues
            .iter()
            .find(|i| matches!(i.kind, DarkModeIssueKind::DarkModeContrastFailure));

        assert!(
            regression.is_some(),
            "expected regression issue for dark-only violations"
        );
        let desc = &regression.unwrap().description;
        assert!(
            desc.contains("Dark Mode"),
            "should mention Dark Mode regression"
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

        let issues = build_issues(&make_info(true, true, true, false, 5), 2, 0, 0, &violations);
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

        let issues = build_issues(&make_info(true, true, true, false, 5), 0, 0, 1, &violations);
        let light_issue = issues
            .iter()
            .find(|i| matches!(i.kind, DarkModeIssueKind::DarkModeContrastFailure));

        assert!(light_issue.is_some());
        assert_eq!(light_issue.unwrap().severity, "low");
    }
}
