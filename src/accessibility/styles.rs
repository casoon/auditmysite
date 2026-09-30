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
    /// Truncated outerHTML snippet for reporting
    pub html_snippet: Option<String>,
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

    /// Colour the glyphs are actually painted with: an explicitly set
    /// `-webkit-text-fill-color` overrides `color`.
    pub fn painted_text_color(&self) -> Option<&str> {
        self.get("text-fill-color").or_else(|| self.color())
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

/// Body of the style-extraction script (wrapped in an IIFE at call time,
/// after the shared `__amsCssSelector` / `__amsIsVisuallyHidden` helpers).
const STYLES_EXTRACT_JS: &str = r#"
    const canvas = typeof document !== 'undefined' ? document.createElement('canvas') : null;
    if (canvas) {
        canvas.width = 1;
        canvas.height = 1;
    }
    const ctx = canvas ? canvas.getContext('2d') : null;

    function parseCssColor(color) {
        if (!color || color === 'transparent') return null;
        if (ctx) {
            ctx.globalCompositeOperation = 'copy';
            ctx.fillStyle = 'rgba(0, 0, 0, 0)';
            ctx.fillStyle = color;
            ctx.fillRect(0, 0, 1, 1);
            const data = ctx.getImageData(0, 0, 1, 1).data;
            return {
                r: data[0],
                g: data[1],
                b: data[2],
                a: data[3] / 255
            };
        }
        const match = color.match(/rgba?\((\d+),\s*(\d+),\s*(\d+)(?:,\s*([\d.]+))?\)/);
        if (!match) return null;
        return {
            r: parseInt(match[1], 10),
            g: parseInt(match[2], 10),
            b: parseInt(match[3], 10),
            a: match[4] !== undefined ? parseFloat(match[4]) : 1
        };
    }

    function composite(top, bottom) {
        const alpha = Math.max(0, Math.min(1, top.a));
        return {
            r: Math.round(top.r * alpha + bottom.r * (1 - alpha)),
            g: Math.round(top.g * alpha + bottom.g * (1 - alpha)),
            b: Math.round(top.b * alpha + bottom.b * (1 - alpha)),
            a: 1
        };
    }

    // #716: ancestors in the composed (flat) tree — a slotted element
    // renders inside its slot, and the root of a shadow tree inside its
    // host, so backgrounds and `background-clip` are inherited across both.
    function composedParent(node) {
        if (node.assignedSlot) return node.assignedSlot;
        if (node.parentElement) return node.parentElement;
        const root = node.getRootNode ? node.getRootNode() : null;
        return (root && root.host) ? root.host : null;
    }

    // Selector through open shadow roots: `host >>> inner`, one hop per
    // shadow boundary. The pixel sampler resolves it the same way.
    function composedSelector(el) {
        let sel = __amsCssSelector(el);
        let root = el.getRootNode ? el.getRootNode() : null;
        while (root && root.host) {
            sel = __amsCssSelector(root.host) + ' >>> ' + sel;
            root = root.host.getRootNode ? root.host.getRootNode() : null;
        }
        return sel;
    }

    function hasPaintedBackgroundImage(styles) {
        return styles.backgroundImage && styles.backgroundImage !== 'none';
    }

    // #527: a non-default blend mode means the painted result depends on
    // layers below it, not just this element's own CSS colors.
    function hasBlendEffect(styles) {
        return (styles.mixBlendMode && styles.mixBlendMode !== 'normal') ||
               (styles.backgroundBlendMode && styles.backgroundBlendMode !== 'normal');
    }

    // #716: whether a positioned element paints an image anywhere below it —
    // an <img>/<picture>/<video>/<canvas> or a background-image, also inside
    // open shadow roots (a hero image as `<p-responsive-image>` in an
    // absolutely positioned wrapper). Memoised: siblings repeat across text
    // elements.
    const paintsImageMemo = new WeakMap();
    function paintsImage(root) {
        if (paintsImageMemo.has(root)) return paintsImageMemo.get(root);
        let found = false;
        const stack = [root];
        let visited = 0;
        while (stack.length > 0 && !found && visited < 500) {
            const node = stack.pop();
            visited++;
            if (node.nodeType === Node.ELEMENT_NODE) {
                const tag = node.tagName;
                if (tag === 'IMG' || tag === 'PICTURE' || tag === 'VIDEO' || tag === 'CANVAS') {
                    found = true;
                    break;
                }
                if (hasPaintedBackgroundImage(window.getComputedStyle(node))) {
                    found = true;
                    break;
                }
                if (node.shadowRoot) stack.push(node.shadowRoot);
            }
            for (const child of node.children || []) stack.push(child);
        }
        paintsImageMemo.set(root, found);
        return found;
    }

    // #527: bounded check for a sibling of `node` that is either an <img>
    // or a positioned (absolute/fixed) element with a painted background or
    // an image inside it (#716), whose box overlaps `targetRect` — covers
    // both the "hero <img> behind overlaid text" and "translucent overlay div
    // on top of text" patterns without a full paint-order simulation.
    // Siblings are taken in the tree `node` lives in (light DOM or shadow
    // root).
    function hasOverlappingPaintedSibling(node, targetRect) {
        const container = node.parentElement || node.parentNode;
        if (!container || !container.children) return false;
        for (const sibling of container.children) {
            if (sibling === node) continue;
            const siblingStyles = window.getComputedStyle(sibling);
            const isImg = sibling.tagName === 'IMG';
            if (!isImg) {
                const isPositioned = siblingStyles.position === 'absolute' || siblingStyles.position === 'fixed';
                if (!isPositioned) continue;
                const bg = parseCssColor(siblingStyles.backgroundColor);
                const painted = (bg && bg.a > 0) || paintsImage(sibling);
                if (!painted) continue;
            }
            const r = sibling.getBoundingClientRect();
            if (r.width === 0 || r.height === 0) continue;
            const overlaps = !(r.right <= targetRect.left || r.left >= targetRect.right ||
                                r.bottom <= targetRect.top || r.top >= targetRect.bottom);
            if (overlaps) return true;
        }
        return false;
    }

    // #640: `background-clip: text` (prefixed or not) on the element or an
    // ancestor paints the glyphs with that element's background (typically a
    // gradient), so the CSS `color` is not what is rendered.
    function hasTextClippedBackground(el) {
        let current = el;
        while (current && current !== document.documentElement) {
            const styles = window.getComputedStyle(current);
            if (styles.backgroundClip === 'text' || styles.webkitBackgroundClip === 'text') {
                return true;
            }
            current = composedParent(current);
        }
        return false;
    }

    // #640: a (near-)transparent text colour or text fill means the visible
    // glyphs come from something other than `color` (background-clip,
    // text-stroke) — compositing it would yield a fabricated ~1:1 ratio.
    const NEAR_TRANSPARENT_ALPHA = 0.05;
    function isNearTransparent(colorStr) {
        const c = parseCssColor(colorStr);
        return !c || c.a < NEAR_TRANSPARENT_ALPHA;
    }

    // `textRect`: box of the text's element — a slot has none of its own.
    function getEffectiveBackground(el, textRect) {
        let current = el;
        const layers = [];
        let hasUncertainBackground = false;
        const elRect = textRect;
        const MAX_OVERLAY_CHECK_DEPTH = 6;
        let overlayCheckDepth = 0;

        while (current && current !== document.documentElement) {
            const styles = window.getComputedStyle(current);
            if (hasPaintedBackgroundImage(styles)) {
                hasUncertainBackground = true;
            }
            const opacity = parseFloat(styles.opacity);
            if (!Number.isNaN(opacity) && opacity < 1) {
                hasUncertainBackground = true;
            }
            if (hasBlendEffect(styles)) {
                hasUncertainBackground = true;
            }
            if (overlayCheckDepth < MAX_OVERLAY_CHECK_DEPTH) {
                if (hasOverlappingPaintedSibling(current, elRect)) {
                    hasUncertainBackground = true;
                }
                overlayCheckDepth++;
            }

            const bg = parseCssColor(styles.backgroundColor);
            if (bg && bg.a > 0) {
                layers.push(bg);
                if (bg.a >= 1) break;
            }

            current = composedParent(current);
        }

        const htmlStyles = window.getComputedStyle(document.documentElement);
        if (hasPaintedBackgroundImage(htmlStyles)) {
            hasUncertainBackground = true;
        }
        const htmlBg = parseCssColor(htmlStyles.backgroundColor);
        if (htmlBg && htmlBg.a > 0) {
            layers.push(htmlBg);
        }

        let color = { r: 255, g: 255, b: 255, a: 1 };
        for (let i = layers.length - 1; i >= 0; i--) {
            color = composite(layers[i], color);
        }

        return {
            color: `rgb(${color.r}, ${color.g}, ${color.b})`,
            uncertain: hasUncertainBackground
        };
    }

    const results = [];
    const seen = new Set();
    let idx = 0;

    // Walk all text nodes to find every element that renders visible text.
    // This matches axe-core's approach of checking all rendered text, not
    // just a fixed list of element types.
    // Open shadow roots are walked too (#716): text inside web components
    // (a cookie banner as `p-cookie-banner`) is rendered text like any other.
    const textNodes = [];
    const roots = [document.body];
    while (roots.length > 0) {
        const root = roots.shift();
        const walker = document.createTreeWalker(root, NodeFilter.SHOW_ELEMENT | NodeFilter.SHOW_TEXT, null);
        let node = walker.currentNode;
        while (node) {
            if (node.nodeType === Node.TEXT_NODE) {
                textNodes.push(node);
            } else if (node.shadowRoot) {
                roots.push(node.shadowRoot);
            }
            node = walker.nextNode();
        }
    }
    for (const textNode of textNodes) {
        if (!textNode.textContent.trim()) continue;
        const el = textNode.parentElement ||
            (textNode.parentNode && textNode.parentNode.host) || null;
        if (!el || seen.has(el)) continue;
        seen.add(el);

        // Skip visually-hidden / .sr-only text — WCAG 1.4.3 does not apply
        if (__amsIsVisuallyHidden(el)) continue;
        // Skip aria-hidden="true" subtrees and role=presentation/none — WCAG 1.4.3 exempts
        // decorative elements that are not exposed to assistive technology (#395)
        if (__amsIsAriaHidden(el)) continue;

        // #716: a text node directly under a shadow host renders through the
        // slot it is assigned to and inherits colour and font from there, not
        // from the host; without a slot it is not rendered at all.
        if (el.shadowRoot && textNode.parentElement === el && !textNode.assignedSlot) continue;
        const styleEl = textNode.assignedSlot || el;

        const styles = window.getComputedStyle(styleEl);
        if (styles.display === 'none' || styles.visibility === 'hidden') continue;

        const effectiveBackground = getEffectiveBackground(styleEl, el.getBoundingClientRect());
        const fg = parseCssColor(styles.color);
        const bg = effectiveBackground.color;
        const bgParsed = parseCssColor(bg) || { r: 255, g: 255, b: 255, a: 1 };

        let finalFg = fg || { r: 0, g: 0, b: 0, a: 1 };
        if (fg && fg.a < 1) {
            finalFg = composite(fg, bgParsed);
        }

        const foregroundUncertain = hasTextClippedBackground(styleEl) ||
            isNearTransparent(styles.color) ||
            (styles.webkitTextFillColor !== undefined && isNearTransparent(styles.webkitTextFillColor));

        // Browsers paint glyphs with -webkit-text-fill-color, not `color`;
        // it only differs from `color` when set explicitly. Reported
        // separately so the rule picks the painted colour.
        let textFillColor = null;
        const fill = styles.webkitTextFillColor !== undefined && styles.webkitTextFillColor !== styles.color
            ? parseCssColor(styles.webkitTextFillColor)
            : null;
        if (fill && fill.a >= NEAR_TRANSPARENT_ALPHA) {
            const fillFinal = fill.a < 1 ? composite(fill, bgParsed) : fill;
            textFillColor = `rgb(${fillFinal.r}, ${fillFinal.g}, ${fillFinal.b})`;
        }

        const rect = el.getBoundingClientRect();

        results.push({
            cssPath: composedSelector(el),
            snippet: el.outerHTML.substring(0, 200),
            index: idx++,
            color: `rgb(${finalFg.r}, ${finalFg.g}, ${finalFg.b})`,
            textFillColor,
            backgroundColor: bg,
            backgroundUncertain: effectiveBackground.uncertain,
            foregroundUncertain,
            fontSize: styles.fontSize,
            fontWeight: styles.fontWeight,
            visibility: styles.visibility,
            display: styles.display,
            width: rect.width,
            height: rect.height,
            excluded: __amsIsExcludedEl(el)
        });
    }

    return results;
"#;

/// Extract computed styles for text elements using JavaScript evaluation
///
/// This function uses JavaScript to extract computed styles directly from the DOM,
/// which is more reliable than CDP GetComputedStyleForNode.
pub async fn extract_text_styles(page: &Page) -> Result<Vec<ComputedStyles>> {
    info!("Extracting computed styles via JavaScript...");

    // Use JavaScript to extract styles for all text elements.
    // Shared CSS-path and visually-hidden helpers are prepended so this rule
    // uses the same heuristics as the other in-page WCAG checks.
    let js_code = [
        "(() => {",
        crate::accessibility::js_helpers::CSS_SELECTOR_JS,
        crate::accessibility::js_helpers::IS_VISUALLY_HIDDEN_JS,
        crate::accessibility::js_helpers::IS_ARIA_HIDDEN_JS,
        STYLES_EXTRACT_JS,
        "})();",
    ]
    .concat();

    let eval_result = page.evaluate(js_code.as_str()).await;

    let styles_vec: Vec<ComputedStyles> = match eval_result {
        Ok(result) => {
            // Parse the JSON result
            if let Some(value) = result.value() {
                match serde_json::from_value::<Vec<serde_json::Value>>(value.clone()) {
                    Ok(items) => {
                        let parsed: Vec<ComputedStyles> = items
                            .iter()
                            .enumerate()
                            .map(|(idx, item)| {
                                let mut properties = HashMap::new();

                                if let Some(color) = item.get("color").and_then(|v| v.as_str()) {
                                    properties.insert("color".to_string(), color.to_string());
                                }
                                if let Some(fill) =
                                    item.get("textFillColor").and_then(|v| v.as_str())
                                {
                                    properties
                                        .insert("text-fill-color".to_string(), fill.to_string());
                                }
                                if let Some(bg) =
                                    item.get("backgroundColor").and_then(|v| v.as_str())
                                {
                                    properties
                                        .insert("background-color".to_string(), bg.to_string());
                                }
                                if let Some(uncertain) =
                                    item.get("backgroundUncertain").and_then(|v| v.as_bool())
                                {
                                    properties.insert(
                                        "background-uncertain".to_string(),
                                        uncertain.to_string(),
                                    );
                                }
                                if let Some(uncertain) =
                                    item.get("foregroundUncertain").and_then(|v| v.as_bool())
                                {
                                    properties.insert(
                                        "foreground-uncertain".to_string(),
                                        uncertain.to_string(),
                                    );
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
                                if let Some(width) = item.get("width").and_then(|v| v.as_f64()) {
                                    properties.insert("width".to_string(), width.to_string());
                                }
                                if let Some(height) = item.get("height").and_then(|v| v.as_f64()) {
                                    properties.insert("height".to_string(), height.to_string());
                                }
                                if item.get("excluded").and_then(|v| v.as_bool()) == Some(true) {
                                    properties.insert("excluded".to_string(), "true".to_string());
                                }

                                let selector = item
                                    .get("cssPath")
                                    .and_then(|v| v.as_str())
                                    .filter(|s| !s.is_empty())
                                    .map(String::from);

                                let html_snippet = item
                                    .get("snippet")
                                    .and_then(|v| v.as_str())
                                    .filter(|s| !s.is_empty())
                                    .map(String::from);

                                ComputedStyles {
                                    node_id: idx as i64,
                                    selector,
                                    html_snippet,
                                    properties,
                                }
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
            html_snippet: None,
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
            html_snippet: None,
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
