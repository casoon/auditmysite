//! Mobile friendliness analysis module
//!
//! Analyzes viewport, touch targets, font sizes, and responsive layout.

pub mod module;
pub mod ux_heuristics;
pub use module::MobileModule;
pub use ux_heuristics::{analyze_ux_heuristics, UxHeuristicFinding, UxHeuristics};

use chromiumoxide::Page;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::error::{AuditError, Result};
use crate::taxonomy::Severity;

/// Mobile friendliness analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobileFriendliness {
    /// Overall mobile-friendly score (0-100)
    pub score: u32,
    /// Viewport configuration
    pub viewport: ViewportAnalysis,
    /// Touch target analysis
    pub touch_targets: TouchTargetAnalysis,
    /// Font size analysis
    pub font_sizes: FontSizeAnalysis,
    /// Content sizing
    pub content_sizing: ContentSizing,
    /// Issues found
    pub issues: Vec<MobileIssue>,
}

/// Viewport configuration analysis
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ViewportAnalysis {
    /// Has viewport meta tag
    pub has_viewport: bool,
    /// Viewport content value
    pub viewport_content: Option<String>,
    /// Is properly configured
    pub is_properly_configured: bool,
    /// Uses width=device-width
    pub uses_device_width: bool,
    /// Has initial-scale=1
    pub has_initial_scale: bool,
    /// Is scalable (not user-scalable=no)
    pub is_scalable: bool,
}

/// Touch target analysis
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TouchTargetAnalysis {
    /// Total interactive elements
    pub total_targets: u32,
    /// Targets with adequate size (≥44x44px)
    pub adequate_targets: u32,
    /// Targets too small
    pub small_targets: u32,
    /// Targets too close together
    pub crowded_targets: u32,
    /// Small targets grouped by context (navigation, footer, header, etc.)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub small_by_context: Vec<(String, u32)>,
    /// Sample of small targets with selector + dimensions (up to 10)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub small_target_samples: Vec<SmallTargetSample>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmallTargetSample {
    pub selector: String,
    pub width: u32,
    pub height: u32,
    pub context: String,
}

/// Font size analysis
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FontSizeAnalysis {
    /// Base font size in pixels
    pub base_font_size: f32,
    /// Smallest font size found
    pub smallest_font_size: f32,
    /// Percentage of text with legible size (≥12px)
    pub legible_percentage: f32,
    /// Uses relative units
    pub uses_relative_units: bool,
    /// Number of interactive elements (a, button, label) with font < 12px
    pub small_interactive_count: u32,
}

/// Content sizing analysis
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContentSizing {
    /// Content width matches viewport
    pub fits_viewport: bool,
    /// Has horizontal scrolling
    pub has_horizontal_scroll: bool,
    /// Uses responsive images
    pub uses_responsive_images: bool,
    /// Uses media queries
    pub uses_media_queries: bool,
}

/// Mobile friendliness issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobileIssue {
    pub category: String,
    pub issue_type: String,
    /// Canonical English, produced by [`mobile_issue_text`] (#406). The PDF
    /// re-derives the localized wording from `kind()` plus `values`.
    pub message: String,
    pub severity: Severity,
    pub impact: String,
    /// Raw values interpolated into the message, so the presentation layer can
    /// rebuild the sentence in another language instead of parsing `message`.
    #[serde(default, skip_serializing_if = "MobileIssueValues::is_empty")]
    pub values: MobileIssueValues,
}

/// The canonical identity of a mobile finding — the single key
/// [`mobile_issue_text`] renders from. Derived from the stored `issue_type` so
/// reports written by older builds keep localizing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MobileIssueKind {
    MissingViewport,
    ImproperViewport,
    NotScalable,
    SmallTargets,
    SmallFonts,
    HorizontalScroll,
}

/// Values interpolated into a mobile message. Every entry is plain data —
/// counts, selectors, measurements — never prose.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MobileIssueValues {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub small_target_count: Option<u32>,
    /// Canonical English context keys (`navigation`, `footer`, `form`, …) with
    /// their counts; localized for display by [`mobile_context_label`].
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub small_by_context: Vec<(String, u32)>,
    /// Up to three `"selector (W×Hpx)"` samples — language-neutral.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub small_target_samples: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub smallest_font_px: Option<f32>,
}

impl MobileIssueValues {
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }
}

impl MobileIssue {
    /// `None` for an `issue_type` this build does not know — callers then fall
    /// back to the stored canonical-English `message`.
    pub fn kind(&self) -> Option<MobileIssueKind> {
        use MobileIssueKind::*;
        Some(match self.issue_type.as_str() {
            "missing_viewport" => MissingViewport,
            "improper_viewport" => ImproperViewport,
            "not_scalable" => NotScalable,
            "small_targets" => SmallTargets,
            "small_fonts" => SmallFonts,
            "horizontal_scroll" => HorizontalScroll,
            _ => return None,
        })
    }

    /// The finding's message in the requested language.
    pub fn localized_message(&self, en: bool) -> String {
        match self.kind() {
            Some(kind) => mobile_issue_text(kind, &self.values, en),
            None => self.message.clone(),
        }
    }
}

/// Human label for a touch-target context key. Unknown keys pass through, so a
/// future context added in the page script degrades to its raw name rather
/// than disappearing.
pub fn mobile_context_label(context: &str, en: bool) -> String {
    match (context, en) {
        ("navigation", true) => "navigation",
        ("navigation", false) => "Navigation",
        ("footer", true) => "footer",
        ("footer", false) => "Fußbereich",
        ("header", true) => "header",
        ("header", false) => "Kopfbereich",
        ("sidebar", true) => "sidebar",
        ("sidebar", false) => "Seitenleiste",
        ("social/utility", true) => "social/utility",
        ("social/utility", false) => "Social/Utility",
        ("form", true) => "form",
        ("form", false) => "Formular",
        ("button", true) => "button",
        ("button", false) => "Schaltfläche",
        ("other", true) => "other",
        ("other", false) => "Sonstige",
        _ => return context.to_string(),
    }
    .to_string()
}

/// The only source of mobile-finding wording (#406). The analysis layer calls
/// it with `en = true` to bake canonical English into the stored struct; the
/// PDF calls it with the run language.
pub fn mobile_issue_text(kind: MobileIssueKind, values: &MobileIssueValues, en: bool) -> String {
    use MobileIssueKind::*;
    match kind {
        MissingViewport => if en {
            "Missing viewport meta tag"
        } else {
            "Viewport-Meta-Tag fehlt"
        }
        .to_string(),
        ImproperViewport => if en {
            "Viewport is not properly configured"
        } else {
            "Viewport ist nicht korrekt konfiguriert"
        }
        .to_string(),
        NotScalable => if en {
            "Page disables zooming (user-scalable=no)"
        } else {
            "Seite unterbindet Zoomen (user-scalable=no)"
        }
        .to_string(),
        SmallTargets => {
            let count = values.small_target_count.unwrap_or(0);
            let context_detail = if values.small_by_context.is_empty() {
                String::new()
            } else {
                let parts: Vec<String> = values
                    .small_by_context
                    .iter()
                    .map(|(ctx, n)| format!("{n} {}", mobile_context_label(ctx, en)))
                    .collect();
                format!(" ({})", parts.join(", "))
            };
            let sample_detail = if values.small_target_samples.is_empty() {
                String::new()
            } else {
                let lead_in = if en { " — e.g. " } else { " — z. B. " };
                format!("{lead_in}{}", values.small_target_samples.join(", "))
            };
            if en {
                format!(
                    "{count} touch targets are too small (<44x44px){context_detail}{sample_detail}"
                )
            } else {
                format!(
                    "{count} Bedienelemente sind zu klein (<44x44px){context_detail}{sample_detail}"
                )
            }
        }
        SmallFonts => {
            let smallest = values.smallest_font_px.unwrap_or(0.0);
            if en {
                format!("Smallest font size is {smallest:.1}px (recommended: ≥12px)")
            } else {
                format!(
                    "Kleinste Schriftgröße ist {}px (empfohlen: ≥12px)",
                    format!("{smallest:.1}").replace('.', ",")
                )
            }
        }
        HorizontalScroll => if en {
            "Page has horizontal scrolling"
        } else {
            "Seite scrollt horizontal"
        }
        .to_string(),
    }
}

/// Build a finding with its canonical-English message derived from `kind` —
/// the message is never written by hand at a call site (#406).
fn mobile_issue(
    category: &str,
    kind: MobileIssueKind,
    values: MobileIssueValues,
    severity: Severity,
    impact: impl Into<String>,
) -> MobileIssue {
    let issue_type = match kind {
        MobileIssueKind::MissingViewport => "missing_viewport",
        MobileIssueKind::ImproperViewport => "improper_viewport",
        MobileIssueKind::NotScalable => "not_scalable",
        MobileIssueKind::SmallTargets => "small_targets",
        MobileIssueKind::SmallFonts => "small_fonts",
        MobileIssueKind::HorizontalScroll => "horizontal_scroll",
    };
    MobileIssue {
        category: category.to_string(),
        issue_type: issue_type.to_string(),
        message: mobile_issue_text(kind, &values, true),
        severity,
        impact: impact.into(),
        values,
    }
}

/// Analyze mobile friendliness of a page
/// Convert the summed issue penalty into a mobile-friendliness score.
///
/// Penalties up to the knee pass through unchanged (scores ≥ 40, so typical
/// one- or two-issue pages are unaffected). Above the knee a square-root curve
/// compresses growth, so a comprehensively mobile-hostile page (no viewport,
/// tiny targets, horizontal scroll, …) stays distinguishable from a merely
/// poor one instead of all collapsing toward zero under linear subtraction.
/// This mirrors the accessibility soft floor; the score never hard-zeros.
fn mobile_score_from_penalty(raw_penalty: u32) -> u32 {
    const KNEE: f32 = 60.0;
    let p = raw_penalty as f32;
    let effective = if p > KNEE {
        KNEE + (p - KNEE).sqrt() * 2.0
    } else {
        p
    };
    (100.0 - effective).round().clamp(5.0, 100.0) as u32
}

pub async fn analyze_mobile_friendliness(page: &Page) -> Result<MobileFriendliness> {
    info!("Analyzing mobile friendliness...");

    let js_code = r#"
    (() => {
        const result = {
            viewport: {},
            touchTargets: { total: 0, small: 0, crowded: 0 },
            fonts: { base: 16, smallest: 16, legibleCount: 0, totalCount: 0 },
            content: {}
        };

        // Viewport analysis
        const viewport = document.querySelector('meta[name="viewport"]');
        if (viewport) {
            const content = viewport.getAttribute('content') || '';
            result.viewport.content = content;
            result.viewport.hasDeviceWidth = content.includes('width=device-width');
            result.viewport.hasInitialScale = content.includes('initial-scale=1');
            result.viewport.isScalable = !content.includes('user-scalable=no') &&
                                          !content.includes('user-scalable=0');
        }

        // Touch targets analysis
        const interactiveElements = document.querySelectorAll('a, button, input, select, textarea, [onclick], [role="button"]');
        result.touchTargets.total = interactiveElements.length;
        const smallByContext = {};
        const smallTargetDetails = [];

        interactiveElements.forEach(el => {
            const rect = el.getBoundingClientRect();
            // Skip elements that are fully hidden (behind hamburger, display:none, etc.)
            if (rect.width === 0 && rect.height === 0) return;
            // Skip visually hidden elements (sr-only pattern: clipped to 1×1px)
            if (rect.width <= 1 && rect.height <= 1) return;
            if (rect.width < 44 || rect.height < 44) {
                result.touchTargets.small++;
                // Classify context of small target
                const tag = el.tagName.toLowerCase();
                const parent = el.closest('nav, footer, header, aside, .social, [class*="social"], [class*="lang"]');
                let ctx = 'other';
                if (parent) {
                    const pTag = parent.tagName.toLowerCase();
                    if (pTag === 'nav') ctx = 'navigation';
                    else if (pTag === 'footer') ctx = 'footer';
                    else if (pTag === 'header') ctx = 'header';
                    else if (pTag === 'aside') ctx = 'sidebar';
                    else ctx = 'social/utility';
                } else if (tag === 'input' || tag === 'select' || tag === 'textarea') {
                    ctx = 'form';
                } else if (tag === 'button' || el.getAttribute('role') === 'button') {
                    ctx = 'button';
                }
                smallByContext[ctx] = (smallByContext[ctx] || 0) + 1;
                // Capture selector for debugging (up to 10 samples)
                if (smallTargetDetails.length < 10) {
                    let sel = tag;
                    if (el.id) sel = '#' + el.id;
                    else if (el.className && typeof el.className === 'string') {
                        const first = el.className.trim().split(/\s+/)[0];
                        if (first) sel = tag + '.' + first;
                    }
                    smallTargetDetails.push({ selector: sel, width: Math.round(rect.width), height: Math.round(rect.height), context: ctx });
                }
            }
        });
        result.touchTargets.smallByContext = smallByContext;
        result.touchTargets.details = smallTargetDetails;

        // Font analysis
        const textElements = document.querySelectorAll('p, span, a, li, td, th, div, h1, h2, h3, h4, h5, h6');
        let smallestFont = 100;

        textElements.forEach(el => {
            const style = window.getComputedStyle(el);
            const fontSize = parseFloat(style.fontSize);
            if (fontSize > 0) {
                result.fonts.totalCount++;
                if (fontSize >= 12) {
                    result.fonts.legibleCount++;
                }
                if (fontSize < smallestFont) {
                    smallestFont = fontSize;
                }
            }
        });

        result.fonts.smallest = smallestFont < 100 ? smallestFont : 16;
        result.fonts.base = parseFloat(window.getComputedStyle(document.body).fontSize) || 16;

        // Count interactive elements with small font (distinguishes real violations from decorative)
        let smallInteractiveCount = 0;
        document.querySelectorAll('a, button, label, input, select, textarea').forEach(el => {
            const rect = el.getBoundingClientRect();
            if (rect.width === 0 && rect.height === 0) return;
            const style = window.getComputedStyle(el);
            const fontSize = parseFloat(style.fontSize);
            if (fontSize > 0 && fontSize < 12) smallInteractiveCount++;
        });
        result.fonts.smallInteractiveCount = smallInteractiveCount;

        // Content sizing
        result.content.viewportWidth = window.innerWidth;
        result.content.documentWidth = document.documentElement.scrollWidth;
        result.content.hasHorizontalScroll = document.documentElement.scrollWidth > window.innerWidth;

        // Check for responsive images
        const images = document.querySelectorAll('img');
        let responsiveImages = 0;
        images.forEach(img => {
            if (img.srcset || img.sizes || window.getComputedStyle(img).maxWidth === '100%') {
                responsiveImages++;
            }
        });
        result.content.responsiveImages = responsiveImages;
        result.content.totalImages = images.length;

        // Check for media queries (approximate)
        let hasMediaQueries = false;
        for (const sheet of document.styleSheets) {
            try {
                for (const rule of sheet.cssRules) {
                    if (rule.type === CSSRule.MEDIA_RULE) {
                        hasMediaQueries = true;
                        break;
                    }
                }
            } catch (e) {}
            if (hasMediaQueries) break;
        }
        result.content.hasMediaQueries = hasMediaQueries;

        // Check for relative font units (em, rem, %, vw)
        let usesRelativeUnits = false;
        for (const sheet of document.styleSheets) {
            try {
                for (const rule of sheet.cssRules) {
                    if (rule.style && rule.style.fontSize) {
                        const fs = rule.style.fontSize;
                        if (fs.match(/\d+(em|rem|%|vw)/)) {
                            usesRelativeUnits = true;
                            break;
                        }
                    }
                }
            } catch (e) {}
            if (usesRelativeUnits) break;
        }
        result.fonts.usesRelativeUnits = usesRelativeUnits;

        return JSON.stringify(result);
    })()
    "#;

    let js_result = page
        .evaluate(js_code)
        .await
        .map_err(|e| AuditError::CdpError(format!("Mobile analysis failed: {}", e)))?;

    let json_str = js_result.value().and_then(|v| v.as_str()).unwrap_or("{}");

    let parsed: serde_json::Value = serde_json::from_str(json_str).unwrap_or_default();

    // Parse viewport
    let vp = &parsed["viewport"];
    let viewport_content = vp["content"].as_str().map(String::from);
    let viewport = ViewportAnalysis {
        has_viewport: viewport_content.is_some(),
        viewport_content: viewport_content.clone(),
        uses_device_width: vp["hasDeviceWidth"].as_bool().unwrap_or(false),
        has_initial_scale: vp["hasInitialScale"].as_bool().unwrap_or(false),
        is_scalable: vp["isScalable"].as_bool().unwrap_or(true),
        is_properly_configured: vp["hasDeviceWidth"].as_bool().unwrap_or(false)
            && vp["hasInitialScale"].as_bool().unwrap_or(false),
    };

    // Parse touch targets
    let tt = &parsed["touchTargets"];
    let total_targets = tt["total"].as_u64().unwrap_or(0) as u32;
    let small_targets = tt["small"].as_u64().unwrap_or(0) as u32;
    let small_by_context: Vec<(String, u32)> = tt["smallByContext"]
        .as_object()
        .map(|obj| {
            let mut pairs: Vec<(String, u32)> = obj
                .iter()
                .map(|(k, v)| (k.clone(), v.as_u64().unwrap_or(0) as u32))
                .collect();
            pairs.sort_by_key(|b| std::cmp::Reverse(b.1));
            pairs
        })
        .unwrap_or_default();
    let small_target_samples: Vec<SmallTargetSample> = tt["details"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|item| {
                    Some(SmallTargetSample {
                        selector: item["selector"].as_str()?.to_string(),
                        width: item["width"].as_u64().unwrap_or(0) as u32,
                        height: item["height"].as_u64().unwrap_or(0) as u32,
                        context: item["context"].as_str().unwrap_or("").to_string(),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    let touch_targets = TouchTargetAnalysis {
        total_targets,
        adequate_targets: total_targets.saturating_sub(small_targets),
        small_targets,
        crowded_targets: tt["crowded"].as_u64().unwrap_or(0) as u32,
        small_by_context,
        small_target_samples,
    };

    // Parse fonts
    let fonts = &parsed["fonts"];
    let total_count = fonts["totalCount"].as_u64().unwrap_or(1) as f32;
    let legible_count = fonts["legibleCount"].as_u64().unwrap_or(0) as f32;
    let font_sizes = FontSizeAnalysis {
        base_font_size: fonts["base"].as_f64().unwrap_or(16.0) as f32,
        smallest_font_size: fonts["smallest"].as_f64().unwrap_or(16.0) as f32,
        legible_percentage: if total_count > 0.0 {
            (legible_count / total_count) * 100.0
        } else {
            100.0
        },
        uses_relative_units: fonts["usesRelativeUnits"].as_bool().unwrap_or(false),
        small_interactive_count: fonts["smallInteractiveCount"].as_u64().unwrap_or(0) as u32,
    };

    // Parse content sizing
    let content = &parsed["content"];
    let content_sizing = ContentSizing {
        fits_viewport: !content["hasHorizontalScroll"].as_bool().unwrap_or(false),
        has_horizontal_scroll: content["hasHorizontalScroll"].as_bool().unwrap_or(false),
        uses_responsive_images: content["responsiveImages"].as_u64().unwrap_or(0)
            >= content["totalImages"].as_u64().unwrap_or(1) / 2,
        uses_media_queries: content["hasMediaQueries"].as_bool().unwrap_or(false),
    };

    // Generate issues
    let mut issues = Vec::new();

    if !viewport.has_viewport {
        issues.push(mobile_issue(
            "viewport",
            MobileIssueKind::MissingViewport,
            MobileIssueValues::default(),
            Severity::Critical,
            "Page won't scale properly on mobile devices",
        ));
    } else if !viewport.is_properly_configured {
        issues.push(mobile_issue(
            "viewport",
            MobileIssueKind::ImproperViewport,
            MobileIssueValues::default(),
            Severity::Medium,
            "Page may not display correctly on all devices",
        ));
    }

    if !viewport.is_scalable {
        issues.push(mobile_issue(
            "viewport",
            MobileIssueKind::NotScalable,
            MobileIssueValues::default(),
            Severity::Critical,
            "Users with visual impairments cannot zoom",
        ));
    }

    if small_targets > 0 {
        // Severity scales with violation count so fewer violations incur a smaller penalty
        let severity = if small_targets >= 20 {
            Severity::High
        } else if small_targets >= 5 {
            Severity::Medium
        } else {
            Severity::Low
        };
        issues.push(mobile_issue(
            "touch_targets",
            MobileIssueKind::SmallTargets,
            MobileIssueValues {
                small_target_count: Some(small_targets),
                small_by_context: touch_targets.small_by_context.clone(),
                small_target_samples: touch_targets
                    .small_target_samples
                    .iter()
                    .take(3)
                    .map(|s| format!("{} ({}×{}px)", s.selector, s.width, s.height))
                    .collect(),
                ..Default::default()
            },
            severity,
            "Difficult to tap on mobile devices",
        ));
    }

    if font_sizes.smallest_font_size < 12.0 {
        let (severity, impact) = if font_sizes.small_interactive_count > 0 {
            (
                Severity::Medium,
                format!(
                    "{} interactive elements (links, buttons, labels) use text below 12px — difficult to read on mobile",
                    font_sizes.small_interactive_count
                ),
            )
        } else {
            (
                Severity::Low,
                "Sub-12px text appears to be decorative only (not on interactive elements) — lower risk".to_string(),
            )
        };
        issues.push(mobile_issue(
            "fonts",
            MobileIssueKind::SmallFonts,
            MobileIssueValues {
                smallest_font_px: Some(font_sizes.smallest_font_size),
                ..Default::default()
            },
            severity,
            impact,
        ));
    }

    if content_sizing.has_horizontal_scroll {
        issues.push(mobile_issue(
            "content",
            MobileIssueKind::HorizontalScroll,
            MobileIssueValues::default(),
            Severity::High,
            "Poor mobile user experience",
        ));
    }

    // Calculate score
    let raw_penalty: u32 = issues
        .iter()
        .map(|issue| match issue.severity {
            Severity::Critical => 20,
            Severity::High => 20,
            Severity::Medium => 10,
            Severity::Low => 5,
        })
        .sum();
    let score = mobile_score_from_penalty(raw_penalty);

    info!(
        "Mobile friendliness: score={}, issues={}",
        score,
        issues.len()
    );

    Ok(MobileFriendliness {
        score,
        viewport,
        touch_targets,
        font_sizes,
        content_sizing,
        issues,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_viewport_analysis_default() {
        let viewport = ViewportAnalysis::default();
        assert!(!viewport.has_viewport);
        assert!(!viewport.is_properly_configured);
    }

    #[test]
    fn test_mobile_score_softens_without_collapsing() {
        // No issues -> perfect.
        assert_eq!(mobile_score_from_penalty(0), 100);
        // Typical small penalties pass through unchanged (score >= 40).
        assert_eq!(mobile_score_from_penalty(20), 80);
        assert_eq!(mobile_score_from_penalty(40), 60);
        assert_eq!(mobile_score_from_penalty(60), 40);
        // Heavy penalties are compressed and stay distinguishable instead of
        // collapsing to zero: monotonically decreasing, but never a hard 0.
        let p70 = mobile_score_from_penalty(70);
        let p90 = mobile_score_from_penalty(90);
        let p140 = mobile_score_from_penalty(140);
        assert!(p70 < 40 && p70 > p90, "70->{p70} 90->{p90}");
        assert!(p90 > p140, "90->{p90} 140->{p140}");
        assert!(p140 >= 5, "worst case must not hard-zero: {p140}");
    }
}

#[cfg(test)]
mod localization_tests {
    use super::*;

    fn sample_values() -> MobileIssueValues {
        MobileIssueValues {
            small_target_count: Some(4),
            small_by_context: vec![("navigation".to_string(), 3), ("form".to_string(), 1)],
            small_target_samples: vec!["a.inline-flex (37×44px)".to_string()],
            smallest_font_px: Some(11.0),
        }
    }

    /// #406 guard: every kind renders differently per language and the English
    /// side stays free of German characters.
    #[test]
    fn every_mobile_message_is_localized_and_english_stays_english() {
        use MobileIssueKind::*;
        let values = sample_values();
        for kind in [
            MissingViewport,
            ImproperViewport,
            NotScalable,
            SmallTargets,
            SmallFonts,
            HorizontalScroll,
        ] {
            let en = mobile_issue_text(kind, &values, true);
            let de = mobile_issue_text(kind, &values, false);
            assert!(!en.is_empty(), "{kind:?}: empty English text");
            assert_ne!(en, de, "{kind:?}: German text must differ from English");
            assert!(
                !en.chars().any(|c| "äöüÄÖÜß".contains(c)),
                "{kind:?} English text leaks German: {en}"
            );
        }
    }

    #[test]
    fn touch_target_contexts_are_localized_and_unknown_keys_pass_through() {
        let de = mobile_issue_text(MobileIssueKind::SmallTargets, &sample_values(), false);
        assert!(de.contains("3 Navigation"), "{de}");
        assert!(de.contains("1 Formular"), "{de}");
        assert!(de.contains("z. B."), "{de}");
        assert_eq!(
            mobile_context_label("brand-new-context", false),
            "brand-new-context"
        );
    }

    /// The analysis layer must never hand-write a message — it always comes
    /// from `mobile_issue_text` in canonical English.
    #[test]
    fn stored_message_is_canonical_english() {
        let issue = mobile_issue(
            "fonts",
            MobileIssueKind::SmallFonts,
            MobileIssueValues {
                smallest_font_px: Some(11.0),
                ..Default::default()
            },
            Severity::Low,
            "impact",
        );
        assert_eq!(
            issue.message,
            "Smallest font size is 11.0px (recommended: ≥12px)"
        );
        assert_eq!(issue.message, issue.localized_message(true));
        assert_eq!(
            issue.localized_message(false),
            "Kleinste Schriftgröße ist 11,0px (empfohlen: ≥12px)"
        );
    }

    #[test]
    fn unknown_issue_type_falls_back_to_the_stored_message() {
        let issue = MobileIssue {
            category: "viewport".to_string(),
            issue_type: "something_this_build_does_not_know".to_string(),
            message: "stored message".to_string(),
            severity: Severity::Low,
            impact: String::new(),
            values: MobileIssueValues::default(),
        };
        assert_eq!(issue.kind(), None);
        assert_eq!(issue.localized_message(false), "stored message");
    }
}
