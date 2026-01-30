//! WCAG 1.4.3 - Contrast (Minimum)
//!
//! Text and images of text must have sufficient contrast ratio:
//! - Normal text: at least 4.5:1
//! - Large text (18pt+ or 14pt+ bold): at least 3:1
//! - Level AAA: 7:1 for normal, 4.5:1 for large

use chromiumoxide::Page;
use tracing::{debug, warn};

use crate::accessibility::{extract_text_styles, AXTree, ComputedStyles};
use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation};

/// Rule metadata for 1.4.3
pub const CONTRAST_RULE: RuleMetadata = RuleMetadata {
    id: "1.4.3",
    name: "Contrast (Minimum)",
    level: WcagLevel::AA,
    severity: Severity::Serious,
    description: "Text must have sufficient color contrast with background",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/contrast-minimum.html",
};

/// WCAG 1.4.3: Contrast (Minimum)
pub struct ContrastRule;

impl ContrastRule {
    /// Check contrast ratios for text elements (legacy - no CDP access)
    pub fn check(_tree: &AXTree, _level: WcagLevel) -> Vec<Violation> {
        // This function is called by the engine but can't do contrast checking
        // without access to the Page. Use check_with_page instead.
        Vec::new()
    }

    /// Check contrast ratios for text elements (with CDP access)
    ///
    /// Note: Prefer `check_with_styles` when styles are already extracted
    /// (e.g., via parallel extraction in the pipeline)
    pub async fn check_with_page(page: &Page, _tree: &AXTree, level: WcagLevel) -> Vec<Violation> {
        debug!("Running contrast check with CDP integration...");

        // Extract computed styles for text elements
        let styles_result = extract_text_styles(page).await;
        let styles = match styles_result {
            Ok(s) => s,
            Err(e) => {
                warn!("Failed to extract text styles: {}", e);
                return Vec::new();
            }
        };

        Self::check_with_styles(&styles, level)
    }

    /// Check contrast ratios using pre-fetched styles
    ///
    /// This is more efficient when styles are extracted in parallel with AXTree
    pub fn check_with_styles(styles: &[ComputedStyles], level: WcagLevel) -> Vec<Violation> {
        let mut violations = Vec::new();

        debug!("Checking contrast for {} elements", styles.len());

        // Check each element's contrast
        for style in styles {
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

            let bg_color_str = style.background_color().unwrap_or("rgb(255, 255, 255)");

            // Parse colors
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
                    // Try to handle rgba(0, 0, 0, 0) - transparent
                    if bg_color_str.contains("rgba") && bg_color_str.contains(", 0)") {
                        Color::new(255, 255, 255) // Default to white
                    } else {
                        continue;
                    }
                }
            };

            // Calculate contrast ratio
            let ratio = Self::calculate_contrast_ratio(&fg_color, &bg_color);
            let is_large = style.is_large_text();

            // Check if it meets requirements
            if !Self::meets_requirement(ratio, is_large, level) {
                let selector = style.selector.as_deref().unwrap_or("unknown");
                let message = format!(
                    "Insufficient color contrast ratio: {:.2}:1 ({}text, requires {}:1)",
                    ratio,
                    if is_large { "large " } else { "" },
                    if is_large {
                        if level == WcagLevel::AAA {
                            "4.5"
                        } else {
                            "3.0"
                        }
                    } else if level == WcagLevel::AAA {
                        "7.0"
                    } else {
                        "4.5"
                    }
                );

                let fix = format!(
                    "Adjust colors to improve contrast. Current: foreground={}, background={}",
                    fg_color_str, bg_color_str
                );

                let violation = Violation::new(
                    CONTRAST_RULE.id,
                    CONTRAST_RULE.name,
                    CONTRAST_RULE.level,
                    Severity::Serious,
                    &message,
                    format!("{}#{}", selector, style.node_id),
                )
                .with_fix(&fix)
                .with_help_url(CONTRAST_RULE.help_url);

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

/// Check if a node represents text content
#[allow(dead_code)]
fn is_text_node(node: &crate::accessibility::AXNode) -> bool {
    matches!(
        node.role.as_deref(),
        Some("StaticText")
            | Some("InlineTextBox")
            | Some("text")
            | Some("paragraph")
            | Some("heading")
    )
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
}
