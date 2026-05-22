//! WCAG 1.4.3 - Contrast (Minimum)
//!
//! Text and images of text must have sufficient contrast ratio:
//! - Normal text: at least 4.5:1
//! - Large text (18pt+ or 14pt+ bold): at least 3:1
//! - Level AAA: 7:1 for normal, 4.5:1 for large

use chromiumoxide::Page;
use tracing::{debug, warn};

use crate::accessibility::{extract_text_styles, AXTree};
use crate::audit::ViewportScreenshot;
use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation};

/// Rule metadata for 1.4.3
pub const CONTRAST_RULE: RuleMetadata = RuleMetadata {
    id: "1.4.3",
    name: "Contrast (Minimum)",
    level: WcagLevel::AA,
    severity: Severity::High,
    description: "Text must have sufficient color contrast with background",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/contrast-minimum.html",
    axe_id: "color-contrast",
    tags: &["wcag2aa", "wcag143", "cat.color"],
};

/// Outcome of a single text element's contrast evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContrastVerdict {
    /// Meets the required contrast ratio.
    Pass,
    /// Confidently below the threshold against a known, solid background.
    Violation,
    /// Below the threshold, but the effective background is an image or gradient
    /// that could not be resolved from CSS. Reported as a manual-review warning
    /// rather than a confirmed failure, to avoid false positives (#264).
    NeedsReview,
}

/// WCAG 1.4.3: Contrast (Minimum)
pub struct ContrastRule;

impl ContrastRule {
    /// Check contrast ratios for text elements (with CDP access)
    /// Check contrast ratios for text elements (with CDP access)
    pub async fn check_with_page(
        page: &Page,
        _tree: &AXTree,
        level: WcagLevel,
        screenshot: Option<&ViewportScreenshot>,
    ) -> Vec<Violation> {
        debug!("Running contrast check with CDP integration...");

        let mut violations = Vec::new();

        // Extract computed styles for text elements
        let styles_result = extract_text_styles(page).await;
        let styles = match styles_result {
            Ok(s) => s,
            Err(e) => {
                warn!("Failed to extract text styles: {}", e);
                return violations;
            }
        };

        debug!("Checking contrast for {} elements", styles.len());

        let mut sample_tasks = Vec::new();
        if screenshot.is_some() {
            for style in &styles {
                let fg_color_str = match style.color() {
                    Some(c) => c,
                    None => continue,
                };
                let background_uncertain = style
                    .get("background-uncertain")
                    .map(|value| value == "true")
                    .unwrap_or(false);

                if background_uncertain {
                    let fg_color = match Color::from_css(fg_color_str) {
                        Some(c) => c,
                        None => continue,
                    };
                    let bg_color_str = style.background_color().unwrap_or("rgb(255, 255, 255)");
                    let bg_color = match Color::from_css(bg_color_str) {
                        Some(c) => c,
                        None => continue,
                    };
                    let ratio = Self::calculate_contrast_ratio(&fg_color, &bg_color);
                    let is_large = style.is_large_text();

                    if !Self::meets_requirement(ratio, is_large, level) {
                        if let Some(ref sel) = style.selector {
                            let threshold = if is_large {
                                if level == WcagLevel::AAA {
                                    4.5
                                } else {
                                    3.0
                                }
                            } else if level == WcagLevel::AAA {
                                7.0
                            } else {
                                4.5
                            };

                            sample_tasks.push(serde_json::json!({
                                "selector": sel,
                                "fgColor": fg_color_str,
                                "threshold": threshold,
                            }));
                        }
                    }
                }
            }
        }

        let mut sampled_verdicts = std::collections::HashMap::new();
        if !sample_tasks.is_empty() {
            if let Some(shot) = screenshot {
                let base64_data = to_base64(&shot.bytes);
                let tasks_json = serde_json::to_string(&sample_tasks).unwrap();
                let js_script = format!(
                    r#"(async () => {{
  const tasks = {};
  const screenshotBase64 = "{}";
  
  const img = new Image();
  const loaded = new Promise((resolve, reject) => {{
    img.onload = () => resolve(true);
    img.onerror = (e) => reject(new Error("Failed to load screenshot image"));
  }});
  img.src = "data:image/png;base64," + screenshotBase64;
  try {{
    await loaded;
  }} catch (err) {{
    return {{ error: err.message, results: [] }};
  }}

  const canvas = document.createElement('canvas');
  canvas.width = img.width;
  canvas.height = img.height;
  const ctx = canvas.getContext('2d');
  if (!ctx) {{
    return {{ error: "Failed to get 2d canvas context", results: [] }};
  }}
  ctx.drawImage(img, 0, 0);

  const dpr = {};
  const scrollX = {};
  const scrollY = {};

  function parseColor(cStr) {{
    const m = cStr.match(/rgba?\((\d+),\s*(\d+),\s*(\d+)(?:,\s*([\d.]+))?\)/);
    if (!m) return {{ r: 0, g: 0, b: 0 }};
    return {{
      r: parseInt(m[1], 10),
      g: parseInt(m[2], 10),
      b: parseInt(m[3], 10)
    }};
  }}

  function relativeLuminance(c) {{
    const srgb = [c.r / 255, c.g / 255, c.b / 255];
    const linear = srgb.map(v => {{
      return v <= 0.04045 ? v / 12.92 : Math.pow((v + 0.055) / 1.055, 2.4);
    }});
    return 0.2126 * linear[0] + 0.7152 * linear[1] + 0.0722 * linear[2];
  }}

  function contrastRatio(l1, l2) {{
    const lighter = Math.max(l1, l2);
    const darker = Math.min(l1, l2);
    return (lighter + 0.05) / (darker + 0.05);
  }}

  const results = [];

  for (const task of tasks) {{
    try {{
      const el = document.querySelector(task.selector);
      if (!el) {{
        results.push({{ selector: task.selector, verdict: "NeedsReview", reason: "Element not found" }});
        continue;
      }}
      
      const rect = el.getBoundingClientRect();
      if (rect.width === 0 || rect.height === 0) {{
        results.push({{ selector: task.selector, verdict: "NeedsReview", reason: "Zero area rect" }});
        continue;
      }}

      const left = Math.round(rect.left * dpr);
      const top = Math.round(rect.top * dpr);
      const width = Math.round(rect.width * dpr);
      const height = Math.round(rect.height * dpr);

      if (left < 0 || top < 0 || left + width > img.width || top + height > img.height) {{
        results.push({{ selector: task.selector, verdict: "NeedsReview", reason: "Element outside screenshot bounds" }});
        continue;
      }}

      const imgData = ctx.getImageData(left, top, width, height);
      const pixels = imgData.data;

      const fg = parseColor(task.fgColor);
      const fgLum = relativeLuminance(fg);

      const ratios = [];
      for (let i = 0; i < pixels.length; i += 4) {{
        const a = pixels[i + 3] / 255;
        const r = Math.round(pixels[i] * a + 255 * (1 - a));
        const g = Math.round(pixels[i + 1] * a + 255 * (1 - a));
        const b = Math.round(pixels[i + 2] * a + 255 * (1 - a));

        const bgLum = relativeLuminance({{ r, g, b }});
        ratios.push(contrastRatio(fgLum, bgLum));
      }}

      if (ratios.length === 0) {{
        results.push({{ selector: task.selector, verdict: "NeedsReview", reason: "No pixels sampled" }});
        continue;
      }}

      ratios.sort((x, y) => x - y);

      const worstIdx = Math.floor(ratios.length * 0.40);
      const worstCaseRatio = ratios[worstIdx];

      const medianIdx = Math.floor(ratios.length * 0.50);
      const medianRatio = ratios[medianIdx];

      let verdict = "NeedsReview";
      if (medianRatio >= task.threshold && worstCaseRatio >= task.threshold) {{
        verdict = "Pass";
      }} else if (medianRatio < task.threshold) {{
        verdict = "Violation";
      }}

      results.push({{
        selector: task.selector,
        verdict,
        medianRatio,
        worstCaseRatio,
        threshold: task.threshold
      }});
    }} catch (e) {{
      results.push({{ selector: task.selector, verdict: "NeedsReview", reason: "Error: " + e.message }});
    }}
  }}

  return {{ results }};
}})();"#,
                    tasks_json, base64_data, shot.device_scale_factor, shot.scroll_x, shot.scroll_y
                );

                match page.evaluate(js_script.as_str()).await {
                    Ok(res) => {
                        if let Some(val) = res.value() {
                            if let Some(arr) = val.get("results").and_then(|v| v.as_array()) {
                                for item in arr {
                                    if let Some(sel) = item.get("selector").and_then(|v| v.as_str())
                                    {
                                        if let Some(verdict) =
                                            item.get("verdict").and_then(|v| v.as_str())
                                        {
                                            let median_ratio =
                                                item.get("medianRatio").and_then(|v| v.as_f64());
                                            let worst_case_ratio =
                                                item.get("worstCaseRatio").and_then(|v| v.as_f64());
                                            sampled_verdicts.insert(
                                                sel.to_string(),
                                                (
                                                    verdict.to_string(),
                                                    median_ratio,
                                                    worst_case_ratio,
                                                ),
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Pixel sampling script execution failed: {}", e);
                    }
                }
            }
        }

        // Check each element's contrast
        for style in &styles {
            // Skip invisible elements
            if let Some(visibility) = style.get("visibility") {
                if visibility == "hidden" {
                    continue;
                }
            }
            if let Some(display) = style.get("display") {
                if display == "none" {
                    continue;
                }
            }

            // Get foreground and background colors
            let fg_color_str = match style.color() {
                Some(c) => c,
                None => continue, // No color specified
            };

            // JS background extraction traverses the DOM, composites alpha
            // colors, and marks image/gradient backgrounds as uncertain.
            let bg_color_str = style.background_color().unwrap_or("rgb(255, 255, 255)");
            let background_uncertain = style
                .get("background-uncertain")
                .map(|value| value == "true")
                .unwrap_or(false);

            let fg_color = match Color::from_css(fg_color_str) {
                Some(c) => c,
                None => {
                    debug!("Failed to parse foreground color: {}", fg_color_str);
                    continue;
                }
            };

            let bg_color = match Color::from_css(bg_color_str) {
                Some(c) => c,
                None => {
                    debug!("Failed to parse background color: {}", bg_color_str);
                    continue;
                }
            };

            // Calculate contrast ratio
            let ratio = Self::calculate_contrast_ratio(&fg_color, &bg_color);
            let is_large = style.is_large_text();

            // Pass / confirmed violation / needs-review (uncertain background).
            let mut verdict = Self::verdict(ratio, is_large, level, background_uncertain);
            let mut final_ratio = ratio;
            let mut is_warning = background_uncertain;

            if background_uncertain && verdict == ContrastVerdict::NeedsReview {
                if let Some(ref sel) = style.selector {
                    if let Some((sampled_verdict, median, _worst)) = sampled_verdicts.get(sel) {
                        if sampled_verdict == "Pass" {
                            verdict = ContrastVerdict::Pass;
                        } else if sampled_verdict == "Violation" {
                            verdict = ContrastVerdict::Violation;
                            is_warning = false;
                            if let Some(m) = median {
                                final_ratio = *m;
                            }
                        } else {
                            verdict = ContrastVerdict::NeedsReview;
                            is_warning = true;
                            if let Some(m) = median {
                                final_ratio = *m;
                            }
                        }
                    }
                }
            }

            if verdict != ContrastVerdict::Pass {
                let selector = style.selector.as_deref().unwrap_or("unknown");
                let threshold = if is_large {
                    if level == WcagLevel::AAA {
                        "4.5"
                    } else {
                        "3.0"
                    }
                } else if level == WcagLevel::AAA {
                    "7.0"
                } else {
                    "4.5"
                };
                let message = if is_warning {
                    format!(
                        "Potential insufficient color contrast ratio: {:.2}:1 ({}text, requires {}:1). Background includes an image or gradient and needs manual review.",
                        final_ratio,
                        if is_large { "large " } else { "" },
                        threshold
                    )
                } else {
                    format!(
                        "Insufficient color contrast ratio: {:.2}:1 ({}text, requires {}:1)",
                        final_ratio,
                        if is_large { "large " } else { "" },
                        threshold
                    )
                };

                let fix = if is_warning {
                    format!(
                        "Verify contrast against the rendered image/gradient background. Estimated from CSS colors: foreground={}, background={}",
                        fg_color_str, bg_color_str
                    )
                } else {
                    format!(
                        "Adjust colors to improve contrast. Current: foreground={}, background={}",
                        fg_color_str, bg_color_str
                    )
                };

                let mut violation = Violation::new(
                    CONTRAST_RULE.id,
                    CONTRAST_RULE.name,
                    CONTRAST_RULE.level,
                    Severity::High,
                    &message,
                    format!("{}#{}", selector, style.node_id),
                )
                .with_selector(selector)
                .with_fix(&fix)
                .with_help_url(CONTRAST_RULE.help_url);
                if let Some(snippet) = &style.html_snippet {
                    violation = violation.with_html_snippet(snippet);
                }
                if is_warning {
                    violation = violation.as_warning();
                }

                violations.push(violation);
            }
        }

        debug!("Found {} contrast violations", violations.len());
        violations
    }

    /// Calculate contrast ratio between two colors
    ///
    /// Formula: (L1 + 0.05) / (L2 + 0.05)
    /// where L1 is the relative luminance of the lighter color
    /// and L2 is the relative luminance of the darker color
    pub fn calculate_contrast_ratio(color1: &Color, color2: &Color) -> f64 {
        let lum1 = color1.relative_luminance();
        let lum2 = color2.relative_luminance();

        let lighter = lum1.max(lum2);
        let darker = lum1.min(lum2);

        (lighter + 0.05) / (darker + 0.05)
    }

    /// Decide whether a measured ratio is a pass, a confirmed violation, or a
    /// manual-review case.
    ///
    /// When the effective background is uncertain (text over an image/gradient
    /// that CSS could not resolve), a sub-threshold ratio is demoted to
    /// `NeedsReview` instead of a confirmed `Violation`, because the CSS-derived
    /// background is only an estimate of what is actually rendered (#264).
    pub fn verdict(
        contrast_ratio: f64,
        is_large_text: bool,
        level: WcagLevel,
        background_uncertain: bool,
    ) -> ContrastVerdict {
        if Self::meets_requirement(contrast_ratio, is_large_text, level) {
            ContrastVerdict::Pass
        } else if background_uncertain {
            ContrastVerdict::NeedsReview
        } else {
            ContrastVerdict::Violation
        }
    }

    /// Check if contrast ratio meets WCAG requirements
    pub fn meets_requirement(contrast_ratio: f64, is_large_text: bool, level: WcagLevel) -> bool {
        let threshold = match (level, is_large_text) {
            (WcagLevel::AAA, false) => 7.0,   // AAA normal text
            (WcagLevel::AAA, true) => 4.5,    // AAA large text
            (WcagLevel::AA, false) => 4.5,    // AA normal text
            (WcagLevel::AA, true) => 3.0,     // AA large text
            (WcagLevel::A, _) => return true, // Level A has no contrast requirement
        };

        contrast_ratio >= threshold
    }
}

/// RGB Color representation
#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    /// Create a new color from RGB values
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Check if a CSS color string represents a fully transparent color
    pub fn is_transparent(css: &str) -> bool {
        let css = css.trim();
        if css == "transparent" {
            return true;
        }
        if css.starts_with("rgba") {
            if let Some(start) = css.find('(') {
                if let Some(end) = css.find(')') {
                    let parts: Vec<&str> =
                        css[start + 1..end].split(',').map(|s| s.trim()).collect();
                    if parts.len() == 4 {
                        if let Ok(alpha) = parts[3].parse::<f64>() {
                            return alpha == 0.0;
                        }
                    }
                }
            }
        }
        false
    }

    /// Parse color from CSS color string
    ///
    /// Supports:
    /// - rgb(r, g, b)
    /// - rgba(r, g, b, a)
    /// - #RRGGBB
    /// - #RGB
    pub fn from_css(css: &str) -> Option<Self> {
        let css = css.trim();

        // rgb(r, g, b) or rgba(r, g, b, a)
        if css.starts_with("rgb") {
            return Self::parse_rgb(css);
        }

        // Hex colors #RRGGBB or #RGB
        if css.starts_with('#') {
            return Self::parse_hex(css);
        }

        None
    }

    /// Parse rgb(r, g, b) or rgba(r, g, b, a)
    fn parse_rgb(css: &str) -> Option<Self> {
        let start = css.find('(')?;
        let end = css.find(')')?;
        let values = &css[start + 1..end];

        let parts: Vec<&str> = values.split(',').map(|s| s.trim()).collect();
        if parts.len() < 3 {
            return None;
        }

        let r = parts[0].parse::<u8>().ok()?;
        let g = parts[1].parse::<u8>().ok()?;
        let b = parts[2].parse::<u8>().ok()?;

        Some(Self::new(r, g, b))
    }

    /// Parse hex color #RRGGBB or #RGB
    fn parse_hex(css: &str) -> Option<Self> {
        let hex = css.trim_start_matches('#');

        match hex.len() {
            3 => {
                // #RGB -> #RRGGBB
                let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
                let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
                let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
                Some(Self::new(r, g, b))
            }
            6 => {
                // #RRGGBB
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some(Self::new(r, g, b))
            }
            _ => None,
        }
    }

    /// Calculate relative luminance
    ///
    /// Formula from WCAG 2.1:
    /// L = 0.2126 * R + 0.7152 * G + 0.0722 * B
    /// where R, G, B are sRGB values converted to linear RGB
    pub fn relative_luminance(&self) -> f64 {
        let r = Self::srgb_to_linear(self.r);
        let g = Self::srgb_to_linear(self.g);
        let b = Self::srgb_to_linear(self.b);

        0.2126 * r + 0.7152 * g + 0.0722 * b
    }

    /// Convert sRGB 8-bit value to linear RGB
    fn srgb_to_linear(value: u8) -> f64 {
        let v = value as f64 / 255.0;

        if v <= 0.04045 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    }
}

fn to_base64(bytes: &[u8]) -> String {
    const CHARS: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity((bytes.len() + 2) / 3 * 4);
    for chunk in bytes.chunks(3) {
        match chunk.len() {
            3 => {
                let n = ((chunk[0] as u32) << 16) | ((chunk[1] as u32) << 8) | (chunk[2] as u32);
                result.push(CHARS[((n >> 18) & 63) as usize] as char);
                result.push(CHARS[((n >> 12) & 63) as usize] as char);
                result.push(CHARS[((n >> 6) & 63) as usize] as char);
                result.push(CHARS[(n & 63) as usize] as char);
            }
            2 => {
                let n = ((chunk[0] as u32) << 8) | (chunk[1] as u32);
                result.push(CHARS[((n >> 10) & 63) as usize] as char);
                result.push(CHARS[((n >> 4) & 63) as usize] as char);
                result.push(CHARS[((n << 2) & 63) as usize] as char);
                result.push('=');
            }
            1 => {
                let n = chunk[0] as u32;
                result.push(CHARS[((n >> 2) & 63) as usize] as char);
                result.push(CHARS[((n << 4) & 63) as usize] as char);
                result.push('=');
                result.push('=');
            }
            _ => unreachable!(),
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_parsing_rgb() {
        let color = Color::from_css("rgb(255, 0, 0)").unwrap();
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 0);
        assert_eq!(color.b, 0);
    }

    #[test]
    fn test_color_parsing_rgba() {
        let color = Color::from_css("rgba(0, 128, 255, 0.5)").unwrap();
        assert_eq!(color.r, 0);
        assert_eq!(color.g, 128);
        assert_eq!(color.b, 255);
    }

    #[test]
    fn test_color_parsing_hex6() {
        let color = Color::from_css("#FF0000").unwrap();
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 0);
        assert_eq!(color.b, 0);
    }

    #[test]
    fn test_color_parsing_hex3() {
        let color = Color::from_css("#F00").unwrap();
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 0);
        assert_eq!(color.b, 0);
    }

    #[test]
    fn test_relative_luminance_white() {
        let white = Color::new(255, 255, 255);
        let lum = white.relative_luminance();
        assert!((lum - 1.0).abs() < 0.01); // White has luminance ~1.0
    }

    #[test]
    fn test_relative_luminance_black() {
        let black = Color::new(0, 0, 0);
        let lum = black.relative_luminance();
        assert!(lum < 0.01); // Black has luminance ~0.0
    }

    #[test]
    fn test_contrast_ratio_black_white() {
        let black = Color::new(0, 0, 0);
        let white = Color::new(255, 255, 255);
        let ratio = ContrastRule::calculate_contrast_ratio(&black, &white);
        assert!((ratio - 21.0).abs() < 0.1); // Black/white has ratio ~21:1
    }

    #[test]
    fn test_contrast_ratio_same_color() {
        let red = Color::new(255, 0, 0);
        let ratio = ContrastRule::calculate_contrast_ratio(&red, &red);
        assert!((ratio - 1.0).abs() < 0.01); // Same color has ratio 1:1
    }

    #[test]
    fn test_meets_requirement_aa_normal() {
        // Normal text AA requires 4.5:1
        assert!(ContrastRule::meets_requirement(4.5, false, WcagLevel::AA));
        assert!(ContrastRule::meets_requirement(5.0, false, WcagLevel::AA));
        assert!(!ContrastRule::meets_requirement(4.0, false, WcagLevel::AA));
    }

    #[test]
    fn test_meets_requirement_aa_large() {
        // Large text AA requires 3:1
        assert!(ContrastRule::meets_requirement(3.0, true, WcagLevel::AA));
        assert!(ContrastRule::meets_requirement(4.0, true, WcagLevel::AA));
        assert!(!ContrastRule::meets_requirement(2.5, true, WcagLevel::AA));
    }

    #[test]
    fn test_meets_requirement_aaa_normal() {
        // Normal text AAA requires 7:1
        assert!(ContrastRule::meets_requirement(7.0, false, WcagLevel::AAA));
        assert!(ContrastRule::meets_requirement(8.0, false, WcagLevel::AAA));
        assert!(!ContrastRule::meets_requirement(6.5, false, WcagLevel::AAA));
    }

    #[test]
    fn test_meets_requirement_level_a() {
        // Level A has no contrast requirement
        assert!(ContrastRule::meets_requirement(1.0, false, WcagLevel::A));
        assert!(ContrastRule::meets_requirement(2.0, true, WcagLevel::A));
    }

    #[test]
    fn test_is_transparent() {
        assert!(Color::is_transparent("transparent"));
        assert!(Color::is_transparent("rgba(0, 0, 0, 0)"));
        assert!(Color::is_transparent("rgba(255, 255, 255, 0)"));
        assert!(Color::is_transparent("rgba(0, 0, 0, 0.0)"));
        assert!(!Color::is_transparent("rgba(0, 0, 0, 0.5)"));
        assert!(!Color::is_transparent("rgba(0, 0, 0, 1)"));
        assert!(!Color::is_transparent("rgb(255, 255, 255)"));
        assert!(!Color::is_transparent("#FFFFFF"));
    }

    #[test]
    fn verdict_passes_when_ratio_meets_threshold() {
        // Above threshold is a pass regardless of background certainty.
        assert_eq!(
            ContrastRule::verdict(5.0, false, WcagLevel::AA, false),
            ContrastVerdict::Pass
        );
        assert_eq!(
            ContrastRule::verdict(5.0, false, WcagLevel::AA, true),
            ContrastVerdict::Pass
        );
    }

    #[test]
    fn verdict_confirms_violation_on_solid_background() {
        // Sub-threshold over a known/solid background → confirmed violation.
        assert_eq!(
            ContrastRule::verdict(3.0, false, WcagLevel::AA, false),
            ContrastVerdict::Violation
        );
    }

    #[test]
    fn verdict_demotes_image_background_to_review() {
        // Sub-threshold over an uncertain image/gradient background → manual
        // review, NOT a confirmed failure (avoids the old white-bg false positive).
        assert_eq!(
            ContrastRule::verdict(3.0, false, WcagLevel::AA, true),
            ContrastVerdict::NeedsReview
        );
        // Large-text threshold (3:1) — a 3.0 ratio passes, so even uncertain bg is a pass.
        assert_eq!(
            ContrastRule::verdict(3.0, true, WcagLevel::AA, true),
            ContrastVerdict::Pass
        );
    }
}
