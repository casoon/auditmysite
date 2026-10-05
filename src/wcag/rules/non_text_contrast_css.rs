//! WCAG 1.4.11 Non-text Contrast — CSS-level check
//!
//! Complements the AXTree-based non_text_contrast rule, which only checks
//! whether a checkbox/radio/switch exposes an accessible checked state —
//! a property real controls (native or ARIA) expose almost universally, so
//! that check essentially never fires against real-world markup.
//!
//! This check instead inspects the actual rendered boundary color of custom
//! (author-restyled, `appearance: none`) checkboxes/radios/switches/range
//! inputs against their surrounding background, and flags insufficient
//! (<3:1) contrast — the pattern axe/Pa11y catch in practice (e.g. a custom
//! checkbox whose "checked" state only changes to a barely-different shade).
//! Native, un-restyled controls are skipped: their contrast is governed by
//! OS/browser chrome that CSS inspection cannot verify, and flagging them
//! would reintroduce false positives.

use chromiumoxide::Page;

use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation};

pub(super) const NON_TEXT_CONTRAST_CSS_RULE: RuleMetadata = RuleMetadata {
    id: "1.4.11",
    name: "Non-text Contrast",
    level: WcagLevel::AA,
    severity: Severity::Medium,
    description: "UI components and graphical objects have a contrast ratio of at least 3:1",
    help_url: "https://www.w3.org/WAI/WCAG22/Understanding/non-text-contrast.html",
    axe_id: "non-text-contrast-css",
    tags: &["wcag2aa", "wcag1411", "cat.color"],
};

const NON_TEXT_CONTRAST_JS: &str = r#"
(function() {
  /*CSS_SELECTOR*/
  const results = [];
  const els = Array.from(document.querySelectorAll(
    'input[type=checkbox], input[type=radio], input[type=range], [role=checkbox], [role=radio], [role=switch]'
  ));

  function effectiveBackground(el) {
    let node = el.parentElement;
    for (let i = 0; i < 6 && node; i++) {
      const bg = getComputedStyle(node).backgroundColor;
      if (bg && bg !== 'transparent' && bg !== 'rgba(0, 0, 0, 0)') return bg;
      node = node.parentElement;
    }
    return 'rgb(255, 255, 255)';
  }

  for (const el of els) {
    const cs = getComputedStyle(el);
    const rect = el.getBoundingClientRect();
    if (rect.width === 0 || rect.height === 0) continue;
    if (cs.visibility === 'hidden' || cs.display === 'none') continue;

    // Only custom-styled controls: native rendering is governed by the
    // browser/OS and cannot be verified (or meaningfully flagged) via CSS.
    const appearance = cs.appearance || cs.webkitAppearance || '';
    if (appearance !== 'none') continue;

    const borderWidth = parseFloat(cs.borderTopWidth) || 0;
    const hasBorder = borderWidth > 0 && cs.borderTopStyle !== 'none';
    const boundaryColor = hasBorder ? cs.borderTopColor : cs.backgroundColor;
    if (!boundaryColor || boundaryColor === 'transparent' || boundaryColor === 'rgba(0, 0, 0, 0)') continue;

    const parentBg = effectiveBackground(el);
    const selector = __amsCssSelector(el);
    const role = el.getAttribute('role') || el.type || el.tagName.toLowerCase();

    results.push({ selector, role, boundaryColor, parentBg });
  }

  return { results };
})()
"#;

pub async fn check_non_text_contrast_css_with_page(page: &Page) -> Vec<Violation> {
    let val = match crate::wcag::types::evaluate_or_fail(
        page,
        &NON_TEXT_CONTRAST_CSS_RULE,
        &NON_TEXT_CONTRAST_JS.replace(
            "/*CSS_SELECTOR*/",
            crate::accessibility::js_helpers::CSS_SELECTOR_JS,
        ),
    )
    .await
    {
        Ok(v) => v,
        Err(violations) => return violations,
    };

    let entries = match val.get("results").and_then(|v| v.as_array()) {
        Some(a) => a,
        None => {
            return vec![crate::wcag::technical_rule_failure(
                &NON_TEXT_CONTRAST_CSS_RULE,
                "invalid_evaluation_shape",
            )]
        }
    };

    let mut violations = Vec::new();

    for entry in entries {
        let selector = entry.get("selector").and_then(|v| v.as_str()).unwrap_or("");
        let role = entry
            .get("role")
            .and_then(|v| v.as_str())
            .unwrap_or("control");
        let boundary_str = match entry.get("boundaryColor").and_then(|v| v.as_str()) {
            Some(s) => s,
            None => continue,
        };
        let bg_str = match entry.get("parentBg").and_then(|v| v.as_str()) {
            Some(s) => s,
            None => continue,
        };

        let boundary = match Color::from_css(boundary_str) {
            Some(c) => c,
            None => continue,
        };
        let bg = match Color::from_css(bg_str) {
            Some(c) => c,
            None => continue,
        };

        let white = Color::new(255, 255, 255);
        let bg_eff = bg.composite_over(&white);
        let boundary_eff = boundary.composite_over(&bg_eff);
        let ratio = boundary_eff.contrast_ratio(&bg_eff);

        if ratio >= 3.0 {
            continue;
        }

        violations.push(
            Violation::new(
                NON_TEXT_CONTRAST_CSS_RULE.id,
                NON_TEXT_CONTRAST_CSS_RULE.name,
                NON_TEXT_CONTRAST_CSS_RULE.level,
                NON_TEXT_CONTRAST_CSS_RULE.severity,
                format!(
                    "Custom {} control boundary color {} has a contrast ratio of {:.2}:1 against its background {} — requires at least 3:1",
                    role, boundary_str, ratio, bg_str
                ),
                selector,
            )
            .with_selector(selector)
            .with_fix("Increase the contrast of the control's border or fill color against its surrounding background to at least 3:1")
            .with_rule_id(NON_TEXT_CONTRAST_CSS_RULE.axe_id)
            .with_help_url(NON_TEXT_CONTRAST_CSS_RULE.help_url),
        );
    }

    violations
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
        if start + 1 > end {
            return false;
        }
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

    /// WCAG-Kontrastverhältnis zweier deckender Farben.
    pub fn contrast_ratio(&self, other: &Color) -> f64 {
        let lighter = self.relative_luminance().max(other.relative_luminance());
        let darker = self.relative_luminance().min(other.relative_luminance());
        (lighter + 0.05) / (darker + 0.05)
    }

    fn srgb_to_linear(value: u8) -> f64 {
        let v = value as f64 / 255.0;
        if v <= 0.04045 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    }
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
        let ratio = black.contrast_ratio(&white);
        assert!((ratio - 21.0).abs() < 0.1);
    }

    #[test]
    fn test_contrast_ratio_same_color() {
        let red = Color::new(255, 0, 0);
        let ratio = red.contrast_ratio(&red);
        assert!((ratio - 1.0).abs() < 0.01);
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
    fn test_is_transparent_malformed_parens_does_not_panic() {
        // Regression: last `)` occurring before the first `(` must not panic
        // on the `start + 1..end` slice.
        assert!(!Color::is_transparent("rgba)("));
    }
}
