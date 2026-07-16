//! WCAG 1.4.3 - Contrast (Minimum)
//!
//! Text and images of text must have sufficient contrast ratio:
//! - Normal text: at least 4.5:1
//! - Large text (18pt+ or 14pt+ bold): at least 3:1
//! - Level AAA: 7:1 for normal, 4.5:1 for large

use std::collections::HashMap;

use chromiumoxide::Page;
use tracing::{debug, warn};

use crate::accessibility::{extract_text_styles, AXTree, ComputedStyles};
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

type SampledVerdicts = HashMap<String, (String, Option<f64>, Option<f64>)>;

/// WCAG 1.4.3: Contrast (Minimum)
pub struct ContrastRule;

impl ContrastRule {
    /// Check contrast ratios for text elements (with CDP access)
    pub async fn check_with_page(
        page: &Page,
        _tree: &AXTree,
        level: WcagLevel,
        screenshot: Option<&ViewportScreenshot>,
    ) -> Vec<Violation> {
        debug!("Running contrast check with CDP integration...");

        let styles = match extract_text_styles(page).await {
            Ok(s) => s,
            Err(e) => {
                warn!("Failed to extract text styles: {}", e);
                return vec![crate::wcag::technical_rule_failure(
                    &CONTRAST_RULE,
                    "style_extraction_failed",
                )];
            }
        };

        debug!("Checking contrast for {} elements", styles.len());

        let sampled = match screenshot {
            Some(shot) => {
                let tasks = Self::build_sample_tasks(&styles, level);
                if tasks.is_empty() {
                    HashMap::new()
                } else {
                    Self::run_pixel_sampling(page, shot, tasks).await
                }
            }
            None => HashMap::new(),
        };

        let violations: Vec<Violation> = styles
            .iter()
            .filter_map(|style| Self::evaluate_style(style, level, &sampled))
            .collect();

        debug!("Found {} contrast violations", violations.len());
        violations
    }

    /// Calculate contrast ratio between two colors
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
        if level == WcagLevel::A {
            return true;
        }
        contrast_ratio >= Self::contrast_threshold(is_large_text, level)
    }

    // ── Private helpers ──────────────────────────────────────────────────────

    /// Required contrast ratio for the given text category and WCAG level.
    fn contrast_threshold(is_large: bool, level: WcagLevel) -> f64 {
        match (level, is_large) {
            (WcagLevel::AAA, false) => 7.0,
            (WcagLevel::AAA, true) => 4.5,
            (_, false) => 4.5,
            (_, true) => 3.0,
        }
    }

    /// Threshold as a display string for violation messages.
    fn contrast_threshold_str(is_large: bool, level: WcagLevel) -> &'static str {
        match (level, is_large) {
            (WcagLevel::AAA, false) => "7.0",
            (WcagLevel::AAA, true) => "4.5",
            (_, false) => "4.5",
            (_, true) => "3.0",
        }
    }

    fn is_invisible(style: &ComputedStyles) -> bool {
        style.get("visibility").is_some_and(|v| v == "hidden")
            || style.get("display").is_some_and(|v| v == "none")
    }

    /// Collect pixel-sampling tasks for uncertain-background elements that appear
    /// to fail the contrast check based on their CSS colors alone.
    fn build_sample_tasks(styles: &[ComputedStyles], level: WcagLevel) -> Vec<serde_json::Value> {
        let mut tasks = Vec::new();
        for style in styles {
            let fg_str = match style.color() {
                Some(c) => c,
                None => continue,
            };
            if style
                .get("background-uncertain")
                .is_none_or(|v| v != "true")
            {
                continue;
            }
            let fg = match Color::from_css(fg_str) {
                Some(c) => c,
                None => continue,
            };
            let bg_str = style.background_color().unwrap_or("rgb(255, 255, 255)");
            let bg = match Color::from_css(bg_str) {
                Some(c) => c,
                None => continue,
            };
            let white = Color::new(255, 255, 255);
            let bg_eff = bg.composite_over(&white);
            let fg_eff = fg.composite_over(&bg_eff);
            let ratio = Self::calculate_contrast_ratio(&fg_eff, &bg_eff);
            let is_large = style.is_large_text();
            if !Self::meets_requirement(ratio, is_large, level) {
                if let Some(ref sel) = style.selector {
                    tasks.push(serde_json::json!({
                        "selector": sel,
                        "fgColor": fg_str,
                        "threshold": Self::contrast_threshold(is_large, level),
                    }));
                }
            }
        }
        tasks
    }

    /// Run the in-browser canvas pixel-sampling script and return per-selector verdicts.
    async fn run_pixel_sampling(
        page: &Page,
        shot: &ViewportScreenshot,
        tasks: Vec<serde_json::Value>,
    ) -> SampledVerdicts {
        let mut verdicts = SampledVerdicts::new();
        let base64_data = to_base64(&shot.bytes);
        let tasks_json = match serde_json::to_string(&tasks) {
            Ok(json) => json,
            Err(e) => {
                warn!("Pixel sampling task serialization failed: {}", e);
                for task in tasks {
                    if let Some(sel) = task.get("selector").and_then(|v| v.as_str()) {
                        verdicts.insert(sel.to_string(), ("NeedsReview".to_string(), None, None));
                    }
                }
                return verdicts;
            }
        };
        let js = Self::pixel_sampling_script(&tasks_json, &base64_data, shot);

        match page.evaluate(js.as_str()).await {
            Ok(res) => {
                if let Some(val) = res.value() {
                    if let Some(arr) = val.get("results").and_then(|v| v.as_array()) {
                        for item in arr {
                            if let (Some(sel), Some(verdict)) = (
                                item.get("selector").and_then(|v| v.as_str()),
                                item.get("verdict").and_then(|v| v.as_str()),
                            ) {
                                let median = item.get("medianRatio").and_then(|v| v.as_f64());
                                let worst = item.get("worstCaseRatio").and_then(|v| v.as_f64());
                                verdicts
                                    .insert(sel.to_string(), (verdict.to_string(), median, worst));
                            }
                        }
                    }
                }
            }
            Err(e) => warn!("Pixel sampling script execution failed: {}", e),
        }
        verdicts
    }

    /// Build the JS canvas pixel-sampling script.
    fn pixel_sampling_script(
        tasks_json: &str,
        base64_data: &str,
        shot: &ViewportScreenshot,
    ) -> String {
        format!(
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
        )
    }

    /// Refine a verdict using pixel-sampled results when the background was uncertain.
    fn resolve_sampled(
        verdict: ContrastVerdict,
        is_warning: bool,
        ratio: f64,
        selector: Option<&str>,
        sampled: &SampledVerdicts,
    ) -> (ContrastVerdict, bool, f64) {
        if !is_warning || verdict != ContrastVerdict::NeedsReview {
            return (verdict, is_warning, ratio);
        }
        let selector = match selector {
            Some(s) => s,
            None => return (verdict, is_warning, ratio),
        };
        match sampled.get(selector) {
            Some((sv, median, _worst)) => match sv.as_str() {
                "Pass" => (ContrastVerdict::Pass, false, ratio),
                "Violation" => (ContrastVerdict::Violation, false, median.unwrap_or(ratio)),
                _ => (ContrastVerdict::NeedsReview, true, median.unwrap_or(ratio)),
            },
            None => (verdict, is_warning, ratio),
        }
    }

    /// Evaluate a single style entry and return a violation if contrast fails.
    fn evaluate_style(
        style: &ComputedStyles,
        level: WcagLevel,
        sampled: &SampledVerdicts,
    ) -> Option<Violation> {
        if Self::is_invisible(style) {
            return None;
        }

        let fg_str = style.color()?;
        let bg_str = style.background_color().unwrap_or("rgb(255, 255, 255)");
        let bg_uncertain = style
            .get("background-uncertain")
            .map(|v| v == "true")
            .unwrap_or(false);

        let fg = Color::from_css(fg_str)?;
        let bg = match Color::from_css(bg_str) {
            Some(c) => c,
            None => {
                debug!("Failed to parse background color: {}", bg_str);
                return None;
            }
        };

        let white = Color::new(255, 255, 255);
        let bg_eff = bg.composite_over(&white);
        let fg_eff = fg.composite_over(&bg_eff);
        let ratio = Self::calculate_contrast_ratio(&fg_eff, &bg_eff);
        let is_large = style.is_large_text();

        let initial_verdict = Self::verdict(ratio, is_large, level, bg_uncertain);
        let (verdict, is_warning, final_ratio) = Self::resolve_sampled(
            initial_verdict,
            bg_uncertain,
            ratio,
            style.selector.as_deref(),
            sampled,
        );

        if verdict == ContrastVerdict::Pass {
            return None;
        }
        let apca_lc = apca_contrast(&fg_eff, &bg_eff);
        Some(Self::build_violation(
            style,
            final_ratio,
            apca_lc,
            is_large,
            is_warning,
            level,
            fg_str,
            bg_str,
        ))
    }

    /// Build a contrast violation from evaluated element data.
    #[allow(clippy::too_many_arguments)]
    fn build_violation(
        style: &ComputedStyles,
        final_ratio: f64,
        apca_lc: f64,
        is_large: bool,
        is_warning: bool,
        level: WcagLevel,
        fg_str: &str,
        bg_str: &str,
    ) -> Violation {
        let selector = style.selector.as_deref().unwrap_or("unknown");
        let threshold = Self::contrast_threshold_str(is_large, level);

        let message = if is_warning {
            format!(
                "Potential insufficient color contrast ratio: {:.2}:1 ({}text, requires {}:1). APCA Lc {:.1} (supplementary, not a conformance gate). Background includes an image or gradient and needs manual review.",
                final_ratio,
                if is_large { "large " } else { "" },
                threshold,
                apca_lc,
            )
        } else {
            format!(
                "Insufficient color contrast ratio: {:.2}:1 ({}text, requires {}:1). APCA Lc {:.1} (supplementary, not a conformance gate).",
                final_ratio,
                if is_large { "large " } else { "" },
                threshold,
                apca_lc,
            )
        };

        let fix = if is_warning {
            format!(
                "Verify contrast against the rendered image/gradient background. Estimated from CSS colors: foreground={}, background={}",
                fg_str, bg_str
            )
        } else {
            format!(
                "Adjust colors to improve contrast. Current: foreground={}, background={}",
                fg_str, bg_str
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
        .with_help_url(CONTRAST_RULE.help_url)
        // Measured values as machine-readable evidence (evidence-grade
        // findings, slice 3) — canonical English, JSON-safe (#406), rendered
        // in the PDF as "Contrast X:Y (required A:B)" in the run locale.
        .with_evidence_item(crate::wcag::types::ViolationEvidence::computed(
            "contrast_ratio",
            format!("{:.2}:1", final_ratio),
        ))
        .with_evidence_item(crate::wcag::types::ViolationEvidence::computed(
            "required_ratio",
            format!("{}:1", threshold),
        ));

        if let Some(snippet) = &style.html_snippet {
            violation = violation.with_html_snippet(snippet);
        }
        if is_warning {
            violation = violation.as_warning();
        }
        violation
    }
}

/// RGB Color representation
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: f64,
}

impl Color {
    /// Create a new color from RGB values
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    /// Composite this color (as foreground) over another color (as background)
    pub fn composite_over(&self, background: &Color) -> Self {
        let a_fg = self.a;
        let a_bg = background.a;

        let a_out = a_fg + a_bg * (1.0 - a_fg);
        if a_out == 0.0 {
            return Self {
                r: 0,
                g: 0,
                b: 0,
                a: 0.0,
            };
        }

        let r_out = ((self.r as f64 * a_fg + background.r as f64 * a_bg * (1.0 - a_fg)) / a_out)
            .round() as u8;
        let g_out = ((self.g as f64 * a_fg + background.g as f64 * a_bg * (1.0 - a_fg)) / a_out)
            .round() as u8;
        let b_out = ((self.b as f64 * a_fg + background.b as f64 * a_bg * (1.0 - a_fg)) / a_out)
            .round() as u8;

        Self {
            r: r_out,
            g: g_out,
            b: b_out,
            a: a_out,
        }
    }

    /// Check if a CSS color string represents a fully transparent color
    pub fn is_transparent(css: &str) -> bool {
        let css = css.trim();
        if css == "transparent" {
            return true;
        }
        if !css.starts_with("rgba") {
            return false;
        }
        let Some(start) = css.find('(') else {
            return false;
        };
        let Some(end) = css.rfind(')') else {
            return false;
        };
        css[start + 1..end]
            .split(',')
            .nth(3)
            .and_then(|s| s.trim().parse::<f64>().ok())
            .is_some_and(|a| a <= 0.001)
    }

    /// Parse color from CSS color string
    pub fn from_css(css: &str) -> Option<Self> {
        let css = css.trim();
        if css.starts_with("rgb") {
            return Self::parse_rgb(css);
        }
        if css.starts_with('#') {
            return Self::parse_hex(css);
        }
        None
    }

    fn parse_rgb(css: &str) -> Option<Self> {
        let start = css.find('(')?;
        let end = css.find(')')?;
        let parts: Vec<&str> = css[start + 1..end].split(',').map(|s| s.trim()).collect();
        if parts.len() < 3 {
            return None;
        }
        let r = parts[0].parse::<u8>().ok()?;
        let g = parts[1].parse::<u8>().ok()?;
        let b = parts[2].parse::<u8>().ok()?;
        let a = if parts.len() >= 4 {
            parts[3].parse::<f64>().unwrap_or(1.0)
        } else {
            1.0
        };
        Some(Self { r, g, b, a })
    }

    fn parse_hex(css: &str) -> Option<Self> {
        let hex = css.trim_start_matches('#');
        match hex.len() {
            3 => {
                let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
                let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
                let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
                Some(Self { r, g, b, a: 1.0 })
            }
            4 => {
                let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
                let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
                let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
                let a_val = u8::from_str_radix(&hex[3..4].repeat(2), 16).ok()?;
                Some(Self {
                    r,
                    g,
                    b,
                    a: a_val as f64 / 255.0,
                })
            }
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some(Self { r, g, b, a: 1.0 })
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a_val = u8::from_str_radix(&hex[6..8], 16).ok()?;
                Some(Self {
                    r,
                    g,
                    b,
                    a: a_val as f64 / 255.0,
                })
            }
            _ => None,
        }
    }

    pub fn relative_luminance(&self) -> f64 {
        let r = Self::srgb_to_linear(self.r);
        let g = Self::srgb_to_linear(self.g);
        let b = Self::srgb_to_linear(self.b);
        0.2126 * r + 0.7152 * g + 0.0722 * b
    }

    fn srgb_to_linear(value: u8) -> f64 {
        let v = value as f64 / 255.0;
        if v <= 0.04045 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    }

    /// APCA's own luminance-like value Y: a straight gamma-2.4 power curve on
    /// sRGB channel values, with APCA's slightly different channel
    /// coefficients — not the piecewise sRGB→linear curve `relative_luminance`
    /// uses for the WCAG 2.x ratio. Two distinct formulas by design; do not
    /// substitute one for the other.
    fn apca_luminance(&self) -> f64 {
        const R_CO: f64 = 0.2126729;
        const G_CO: f64 = 0.7151522;
        const B_CO: f64 = 0.0721750;
        (self.r as f64 / 255.0).powf(2.4) * R_CO
            + (self.g as f64 / 255.0).powf(2.4) * G_CO
            + (self.b as f64 / 255.0).powf(2.4) * B_CO
    }
}

/// APCA (Accessible Perceptual Contrast Algorithm) lightness contrast, Lc.
///
/// Reference implementation: APCA-W3 0.1.9 ("Bridge-PVR", Myndex). Supplied
/// as supplementary information alongside the WCAG 2.x ratio — APCA is not
/// yet a WCAG conformance requirement, so it never gates `verdict()`. Unlike
/// the WCAG ratio, APCA is signed: positive Lc means dark text on a light
/// background, negative means light text on a dark background, and the
/// scale (roughly -108..108) is not directly comparable to the WCAG 1:1–21:1
/// ratio.
pub fn apca_contrast(text: &Color, background: &Color) -> f64 {
    const BLACK_THRESHOLD: f64 = 0.022;
    const BLACK_CLAMP: f64 = 1.414;
    const DELTA_Y_MIN: f64 = 0.0005;
    const NORM_BG: f64 = 0.56;
    const NORM_TEXT: f64 = 0.57;
    const REV_TEXT: f64 = 0.62;
    const REV_BG: f64 = 0.65;
    const SCALE_BOW: f64 = 1.14;
    const SCALE_WOB: f64 = 1.14;
    const LO_BOW_OFFSET: f64 = 0.027;
    const LO_WOB_OFFSET: f64 = 0.027;
    const LO_CLIP: f64 = 0.1;

    fn clamp_black(y: f64) -> f64 {
        if y > BLACK_THRESHOLD {
            y
        } else {
            y + (BLACK_THRESHOLD - y).powf(BLACK_CLAMP)
        }
    }

    let txt_y = clamp_black(text.apca_luminance());
    let bg_y = clamp_black(background.apca_luminance());

    if (bg_y - txt_y).abs() < DELTA_Y_MIN {
        return 0.0;
    }

    let output = if bg_y > txt_y {
        // Normal polarity: dark text on a light background.
        let sapc = (bg_y.powf(NORM_BG) - txt_y.powf(NORM_TEXT)) * SCALE_BOW;
        if sapc < LO_CLIP {
            0.0
        } else {
            sapc - LO_BOW_OFFSET
        }
    } else {
        // Reverse polarity: light text on a dark background.
        let sapc = (bg_y.powf(REV_BG) - txt_y.powf(REV_TEXT)) * SCALE_WOB;
        if sapc > -LO_CLIP {
            0.0
        } else {
            sapc + LO_WOB_OFFSET
        }
    };

    output * 100.0
}

fn to_base64(bytes: &[u8]) -> String {
    const CHARS: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity(bytes.len().div_ceil(3) * 4);
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
        assert!((color.a - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_color_parsing_hex8() {
        let color = Color::from_css("#0080FF7F").unwrap();
        assert_eq!(color.r, 0);
        assert_eq!(color.g, 128);
        assert_eq!(color.b, 255);
        assert!((color.a - 127.0 / 255.0).abs() < 0.001);
    }

    #[test]
    fn test_color_parsing_hex4() {
        let color = Color::from_css("#08F7").unwrap();
        assert_eq!(color.r, 0);
        assert_eq!(color.g, 136);
        assert_eq!(color.b, 255);
        assert!((color.a - 119.0 / 255.0).abs() < 0.001);
    }

    #[test]
    fn test_alpha_compositing_and_blending() {
        let fg = Color::from_css("rgba(0, 0, 0, 0.1)").unwrap(); // 10% black
        let bg = Color::new(255, 255, 255); // opaque white
        let effective = fg.composite_over(&bg);
        assert_eq!(effective.r, 230); // 255 * 0.9 = 229.5 -> 230
        assert_eq!(effective.g, 230);
        assert_eq!(effective.b, 230);
        assert_eq!(effective.a, 1.0);
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
        assert!((lum - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_relative_luminance_black() {
        let black = Color::new(0, 0, 0);
        let lum = black.relative_luminance();
        assert!(lum < 0.01);
    }

    #[test]
    fn test_contrast_ratio_black_white() {
        let black = Color::new(0, 0, 0);
        let white = Color::new(255, 255, 255);
        let ratio = ContrastRule::calculate_contrast_ratio(&black, &white);
        assert!((ratio - 21.0).abs() < 0.1);
    }

    #[test]
    fn test_contrast_ratio_same_color() {
        let red = Color::new(255, 0, 0);
        let ratio = ContrastRule::calculate_contrast_ratio(&red, &red);
        assert!((ratio - 1.0).abs() < 0.01);
    }

    fn low_contrast_style(node_id: i64) -> ComputedStyles {
        let mut properties = std::collections::HashMap::new();
        properties.insert("color".to_string(), "rgb(200, 200, 200)".to_string());
        properties.insert(
            "background-color".to_string(),
            "rgb(255, 255, 255)".to_string(),
        );
        ComputedStyles {
            node_id,
            selector: Some("p.low-contrast".to_string()),
            html_snippet: None,
            properties,
        }
    }

    #[test]
    fn evaluate_style_attaches_computed_contrast_evidence() {
        let style = low_contrast_style(1);
        let sampled: SampledVerdicts = std::collections::HashMap::new();
        let violation = ContrastRule::evaluate_style(&style, WcagLevel::AA, &sampled)
            .expect("sub-threshold contrast must produce a violation");

        let contrast_ev = violation
            .evidence
            .iter()
            .find(|e| e.source == "computed" && e.field.as_deref() == Some("contrast_ratio"))
            .expect("violation must carry computed contrast_ratio evidence");
        let required_ev = violation
            .evidence
            .iter()
            .find(|e| e.source == "computed" && e.field.as_deref() == Some("required_ratio"))
            .expect("violation must carry computed required_ratio evidence");

        assert_eq!(required_ev.value.as_deref(), Some("4.5:1"));
        assert!(contrast_ev.value.as_deref().unwrap().ends_with(":1"));
    }

    #[test]
    fn computed_contrast_evidence_has_no_german_umlauts() {
        // #406 guard: canonical evidence values are locale-neutral
        // (numeric ratios), never baked with German text.
        let style = low_contrast_style(2);
        let sampled: SampledVerdicts = std::collections::HashMap::new();
        let violation = ContrastRule::evaluate_style(&style, WcagLevel::AA, &sampled)
            .expect("sub-threshold contrast must produce a violation");

        let has_umlaut = |s: &str| s.chars().any(|c| "äöüÄÖÜß".contains(c));
        for ev in &violation.evidence {
            if let Some(value) = &ev.value {
                assert!(
                    !has_umlaut(value),
                    "evidence value contains German umlaut: {value}"
                );
            }
        }
    }

    #[test]
    fn test_meets_requirement_aa_normal() {
        assert!(ContrastRule::meets_requirement(4.5, false, WcagLevel::AA));
        assert!(ContrastRule::meets_requirement(5.0, false, WcagLevel::AA));
        assert!(!ContrastRule::meets_requirement(4.0, false, WcagLevel::AA));
    }

    #[test]
    fn test_meets_requirement_aa_large() {
        assert!(ContrastRule::meets_requirement(3.0, true, WcagLevel::AA));
        assert!(ContrastRule::meets_requirement(4.0, true, WcagLevel::AA));
        assert!(!ContrastRule::meets_requirement(2.5, true, WcagLevel::AA));
    }

    #[test]
    fn test_meets_requirement_aaa_normal() {
        assert!(ContrastRule::meets_requirement(7.0, false, WcagLevel::AAA));
        assert!(ContrastRule::meets_requirement(8.0, false, WcagLevel::AAA));
        assert!(!ContrastRule::meets_requirement(6.5, false, WcagLevel::AAA));
    }

    #[test]
    fn test_meets_requirement_level_a() {
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
        assert_eq!(
            ContrastRule::verdict(3.0, false, WcagLevel::AA, false),
            ContrastVerdict::Violation
        );
    }

    #[test]
    fn apca_contrast_black_on_white_matches_reference() {
        let black = Color::new(0, 0, 0);
        let white = Color::new(255, 255, 255);
        let lc = apca_contrast(&black, &white);
        // Published APCA reference value for black-on-white is ~106.
        assert!((lc - 106.0).abs() < 1.0, "expected ~106, got {lc}");
    }

    #[test]
    fn apca_contrast_white_on_black_matches_reference() {
        let white = Color::new(255, 255, 255);
        let black = Color::new(0, 0, 0);
        let lc = apca_contrast(&white, &black);
        // Published APCA reference value for white-on-black is ~-107.9.
        assert!((lc + 107.9).abs() < 1.0, "expected ~-107.9, got {lc}");
    }

    #[test]
    fn apca_contrast_sign_reflects_polarity() {
        let dark_text_light_bg = apca_contrast(&Color::new(50, 50, 50), &Color::new(240, 240, 240));
        let light_text_dark_bg = apca_contrast(&Color::new(240, 240, 240), &Color::new(50, 50, 50));
        assert!(dark_text_light_bg > 0.0);
        assert!(light_text_dark_bg < 0.0);
    }

    #[test]
    fn apca_contrast_same_color_is_zero() {
        let gray = Color::new(128, 128, 128);
        assert_eq!(apca_contrast(&gray, &gray), 0.0);
    }

    #[test]
    fn verdict_demotes_image_background_to_review() {
        assert_eq!(
            ContrastRule::verdict(3.0, false, WcagLevel::AA, true),
            ContrastVerdict::NeedsReview
        );
        assert_eq!(
            ContrastRule::verdict(3.0, true, WcagLevel::AA, true),
            ContrastVerdict::Pass
        );
    }
}
