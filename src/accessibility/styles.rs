//! Computed Style Extraction via JavaScript Evaluation
//!
//! Extracts computed CSS styles for accessibility analysis,
//! particularly for contrast checking.

use chromiumoxide::Page;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn};

use crate::error::Result;

/// Computed styles for a DOM node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputedStyles {
    /// Node ID (element index)
    pub node_id: i64,
    /// CSS selector for the element
    pub selector: Option<String>,
    /// Map of CSS property names to values
    pub properties: HashMap<String, String>,
}

impl ComputedStyles {
    /// Get the value of a CSS property
    pub fn get(&self, property: &str) -> Option<&str> {
        self.properties.get(property).map(|s| s.as_str())
    }

    /// Get foreground color (text color)
    pub fn color(&self) -> Option<&str> {
        self.get("color")
    }

    /// Get background color
    pub fn background_color(&self) -> Option<&str> {
        self.get("background-color")
    }

    /// Get font size
    pub fn font_size(&self) -> Option<&str> {
        self.get("font-size")
    }

    /// Get font weight
    pub fn font_weight(&self) -> Option<&str> {
        self.get("font-weight")
    }

    /// Check if font is bold (weight >= 700)
    pub fn is_bold(&self) -> bool {
        if let Some(weight) = self.font_weight() {
            if let Ok(weight_num) = weight.parse::<u32>() {
                return weight_num >= 700;
            }
            // Handle named weights
            return matches!(weight, "bold" | "bolder" | "700" | "800" | "900");
        }
        false
    }

    /// Check if text is "large" per WCAG definition
    /// - 18pt+ (24px+) for normal text
    /// - 14pt+ (18.66px+) for bold text
    pub fn is_large_text(&self) -> bool {
        if let Some(size_str) = self.font_size() {
            if let Some(size_px) = parse_font_size_px(size_str) {
                if self.is_bold() {
                    return size_px >= 18.66;
                } else {
                    return size_px >= 24.0;
                }
            }
        }
        false
    }
}

/// Extract computed styles for text elements using JavaScript evaluation
///
/// This function uses JavaScript to extract computed styles directly from the DOM,
/// which is more reliable than CDP GetComputedStyleForNode.
pub async fn extract_text_styles(page: &Page) -> Result<Vec<ComputedStyles>> {
    info!("Extracting computed styles via JavaScript...");

    // Use JavaScript to extract styles for all text elements
    let js_code = r#"
    (() => {
        const selectors = ['p', 'span', 'div', 'a', 'button', 'h1', 'h2', 'h3', 'h4', 'h5', 'h6', 'li', 'td', 'th', 'label', 'strong', 'em'];
        const results = [];

        selectors.forEach(selector => {
            const elements = document.querySelectorAll(selector);
            elements.forEach((el, idx) => {
                // Check if element has text content
                const text = el.textContent || '';
                if (text.trim().length === 0) {
                    return; // Skip elements without text
                }

                const styles = window.getComputedStyle(el);

                // Skip hidden elements
                if (styles.display === 'none' || styles.visibility === 'hidden') {
                    return;
                }

                results.push({
                    selector: selector,
                    index: idx,
                    color: styles.color,
                    backgroundColor: styles.backgroundColor,
                    fontSize: styles.fontSize,
                    fontWeight: styles.fontWeight,
                    visibility: styles.visibility,
                    display: styles.display
                });
            });
        });

        return results;
    })();
    "#;

    let eval_result = page.evaluate(js_code).await;

    let styles_vec: Vec<ComputedStyles> = match eval_result {
        Ok(result) => {
            // Parse the JSON result
            if let Some(value) = result.value() {
                match serde_json::from_value::<Vec<serde_json::Value>>(value.clone()) {
                    Ok(items) => {
                        let parsed: Vec<ComputedStyles> = items
                            .iter()
                            .enumerate()
                            .filter_map(|(idx, item)| {
                                let mut properties = HashMap::new();

                                if let Some(color) = item.get("color").and_then(|v| v.as_str()) {
                                    properties.insert("color".to_string(), color.to_string());
                                }
                                if let Some(bg) =
                                    item.get("backgroundColor").and_then(|v| v.as_str())
                                {
                                    properties
                                        .insert("background-color".to_string(), bg.to_string());
                                }
                                if let Some(size) = item.get("fontSize").and_then(|v| v.as_str()) {
                                    properties.insert("font-size".to_string(), size.to_string());
                                }
                                if let Some(weight) =
                                    item.get("fontWeight").and_then(|v| v.as_str())
                                {
                                    properties
                                        .insert("font-weight".to_string(), weight.to_string());
                                }
                                if let Some(vis) = item.get("visibility").and_then(|v| v.as_str()) {
                                    properties.insert("visibility".to_string(), vis.to_string());
                                }
                                if let Some(disp) = item.get("display").and_then(|v| v.as_str()) {
                                    properties.insert("display".to_string(), disp.to_string());
                                }

                                let selector = item
                                    .get("selector")
                                    .and_then(|v| v.as_str())
                                    .map(String::from);

                                Some(ComputedStyles {
                                    node_id: idx as i64,
                                    selector,
                                    properties,
                                })
                            })
                            .collect();

                        info!("Parsed {} style objects from JavaScript", parsed.len());
                        parsed
                    }
                    Err(e) => {
                        warn!("Failed to parse styles JSON: {}", e);
                        Vec::new()
                    }
                }
            } else {
                warn!("No value returned from JavaScript evaluation");
                Vec::new()
            }
        }
        Err(e) => {
            warn!("Failed to evaluate JavaScript for styles: {}", e);
            Vec::new()
        }
    };

    info!(
        "Extracted computed styles for {} elements",
        styles_vec.len()
    );
    Ok(styles_vec)
}

/// Parse font size string to pixels
///
/// Handles: px, pt, em, rem, %
/// Assumes: 16px base font size, 96 DPI
fn parse_font_size_px(size_str: &str) -> Option<f64> {
    let size_str = size_str.trim();

    if size_str.ends_with("px") {
        size_str.trim_end_matches("px").parse::<f64>().ok()
    } else if size_str.ends_with("pt") {
        // 1pt = 1.333px at 96 DPI
        size_str
            .trim_end_matches("pt")
            .parse::<f64>()
            .ok()
            .map(|pt| pt * 1.333)
    } else if size_str.ends_with("rem") {
        // Assume 16px base
        size_str
            .trim_end_matches("rem")
            .parse::<f64>()
            .ok()
            .map(|em| em * 16.0)
    } else if size_str.ends_with("em") {
        // Assume 16px base
        size_str
            .trim_end_matches("em")
            .parse::<f64>()
            .ok()
            .map(|em| em * 16.0)
    } else if size_str.ends_with('%') {
        // Assume 16px base
        size_str
            .trim_end_matches('%')
            .parse::<f64>()
            .ok()
            .map(|pct| (pct / 100.0) * 16.0)
    } else {
        // Try parsing as raw number (pixels)
        size_str.parse::<f64>().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_font_size_px() {
        assert_eq!(parse_font_size_px("16px"), Some(16.0));
        assert_eq!(parse_font_size_px("24px"), Some(24.0));
    }

    #[test]
    fn test_parse_font_size_pt() {
        let size = parse_font_size_px("18pt").unwrap();
        assert!((size - 24.0).abs() < 0.1); // 18pt ≈ 24px
    }

    #[test]
    fn test_parse_font_size_em() {
        assert_eq!(parse_font_size_px("1.5em"), Some(24.0)); // 1.5 * 16 = 24
        assert_eq!(parse_font_size_px("1rem"), Some(16.0));
    }

    #[test]
    fn test_parse_font_size_percent() {
        assert_eq!(parse_font_size_px("150%"), Some(24.0)); // 150% of 16 = 24
    }

    #[test]
    fn test_is_large_text() {
        let mut styles = ComputedStyles {
            node_id: 1,
            selector: None,
            properties: HashMap::new(),
        };

        // 24px normal text (large)
        styles
            .properties
            .insert("font-size".to_string(), "24px".to_string());
        styles
            .properties
            .insert("font-weight".to_string(), "400".to_string());
        assert!(styles.is_large_text());

        // 20px bold text (large)
        styles
            .properties
            .insert("font-size".to_string(), "20px".to_string());
        styles
            .properties
            .insert("font-weight".to_string(), "700".to_string());
        assert!(styles.is_large_text());

        // 16px normal text (not large)
        styles
            .properties
            .insert("font-size".to_string(), "16px".to_string());
        styles
            .properties
            .insert("font-weight".to_string(), "400".to_string());
        assert!(!styles.is_large_text());
    }

    #[test]
    fn test_is_bold() {
        let mut styles = ComputedStyles {
            node_id: 1,
            selector: None,
            properties: HashMap::new(),
        };

        styles
            .properties
            .insert("font-weight".to_string(), "700".to_string());
        assert!(styles.is_bold());

        styles
            .properties
            .insert("font-weight".to_string(), "bold".to_string());
        assert!(styles.is_bold());

        styles
            .properties
            .insert("font-weight".to_string(), "400".to_string());
        assert!(!styles.is_bold());
    }
}
