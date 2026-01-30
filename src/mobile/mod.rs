//! Mobile friendliness analysis module
//!
//! Analyzes viewport, touch targets, font sizes, and responsive layout.

use chromiumoxide::Page;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::error::{AuditError, Result};

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
    /// Targets with adequate size (≥48x48px)
    pub adequate_targets: u32,
    /// Targets too small
    pub small_targets: u32,
    /// Targets too close together
    pub crowded_targets: u32,
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
    pub message: String,
    pub severity: String,
    pub impact: String,
}

/// Analyze mobile friendliness of a page
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

        interactiveElements.forEach(el => {
            const rect = el.getBoundingClientRect();
            if (rect.width < 44 || rect.height < 44) {
                result.touchTargets.small++;
            }
        });

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

        return JSON.stringify(result);
    })()
    "#;

    let js_result = page
        .evaluate(js_code)
        .await
        .map_err(|e| AuditError::CdpError(format!("Mobile analysis failed: {}", e)))?;

    let json_str = js_result.value().and_then(|v| v.as_str()).unwrap_or("{}");

    let parsed: serde_json::Value = serde_json::from_str(json_str).unwrap_or_else(|e| {
        warn!("Failed to parse mobile analysis JSON: {}", e);
        serde_json::Value::default()
    });

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
    let touch_targets = TouchTargetAnalysis {
        total_targets,
        adequate_targets: total_targets.saturating_sub(small_targets),
        small_targets,
        crowded_targets: tt["crowded"].as_u64().unwrap_or(0) as u32,
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
        uses_relative_units: true, // Would need more analysis
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
        issues.push(MobileIssue {
            category: "viewport".to_string(),
            issue_type: "missing_viewport".to_string(),
            message: "Missing viewport meta tag".to_string(),
            severity: "error".to_string(),
            impact: "Page won't scale properly on mobile devices".to_string(),
        });
    } else if !viewport.is_properly_configured {
        issues.push(MobileIssue {
            category: "viewport".to_string(),
            issue_type: "improper_viewport".to_string(),
            message: "Viewport is not properly configured".to_string(),
            severity: "warning".to_string(),
            impact: "Page may not display correctly on all devices".to_string(),
        });
    }

    if !viewport.is_scalable {
        issues.push(MobileIssue {
            category: "viewport".to_string(),
            issue_type: "not_scalable".to_string(),
            message: "Page disables zooming (user-scalable=no)".to_string(),
            severity: "error".to_string(),
            impact: "Users with visual impairments cannot zoom".to_string(),
        });
    }

    if small_targets > 0 {
        issues.push(MobileIssue {
            category: "touch_targets".to_string(),
            issue_type: "small_targets".to_string(),
            message: format!("{} touch targets are too small (<44x44px)", small_targets),
            severity: "warning".to_string(),
            impact: "Difficult to tap on mobile devices".to_string(),
        });
    }

    if font_sizes.smallest_font_size < 12.0 {
        issues.push(MobileIssue {
            category: "fonts".to_string(),
            issue_type: "small_fonts".to_string(),
            message: format!(
                "Smallest font size is {}px (recommended: ≥12px)",
                font_sizes.smallest_font_size
            ),
            severity: "warning".to_string(),
            impact: "Text may be difficult to read on mobile".to_string(),
        });
    }

    if content_sizing.has_horizontal_scroll {
        issues.push(MobileIssue {
            category: "content".to_string(),
            issue_type: "horizontal_scroll".to_string(),
            message: "Page has horizontal scrolling".to_string(),
            severity: "error".to_string(),
            impact: "Poor mobile user experience".to_string(),
        });
    }

    // Calculate score
    let mut score = 100u32;
    for issue in &issues {
        score = score.saturating_sub(match issue.severity.as_str() {
            "error" => 20,
            "warning" => 10,
            _ => 5,
        });
    }

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
}
