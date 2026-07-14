//! Element-level evidence screenshots for WCAG violations.
//!
//! Captures a cropped, highlighted screenshot of the DOM element behind a
//! confirmed violation, so the PDF report is self-contained proof rather than
//! a bare selector string. Runs as a dedicated pass after
//! [`super::enrich_violations_with_page`], reusing the same
//! `backend_dom_node_id` → `RemoteObjectId` resolution the enrichment pass
//! already performs.
//!
//! Contrast (1.4.3) violations are intentionally excluded: their `node_id` is
//! a synthetic string from the style extractor (`"{selector}#{node_id}"`),
//! not a real AXTree node id, so `AXTree::get_node` never resolves a backend
//! node for them — they fall through the same `None` path as any other
//! unresolvable element, no special-casing needed. See
//! `plans/evidence-grade-findings.md`'s "Open questions" for why this is a
//! deliberate scope cut rather than a gap: their selectors aren't guaranteed
//! valid CSS, and guessing via `document.querySelector` risks capturing the
//! wrong element — a missing crop is preferable to a misleading one.
//!
//! Every step degrades to `None` + `tracing::warn!`; this module never
//! returns `Err` into the pipeline.

use std::collections::HashSet;

use chromiumoxide::cdp::browser_protocol::dom::{BackendNodeId, ResolveNodeParams};
use chromiumoxide::cdp::browser_protocol::page::{
    CaptureScreenshotFormat, Viewport as ClipViewport,
};
use chromiumoxide::cdp::js_protocol::runtime::{CallFunctionOnParams, RemoteObjectId};
use chromiumoxide::page::ScreenshotParams;
use chromiumoxide::Page;
use tracing::{info, warn};

use super::tree::AXTree;
use crate::wcag::types::{FindingKind, Violation};

/// Hard cap on element-evidence crops per report — bounds PDF size growth.
pub const MAX_ELEMENT_CROPS: usize = 12;

const CROP_PADDING_PX: f64 = 12.0;
const MAX_CROP_WIDTH_PX: f64 = 1280.0;
const MAX_CROP_HEIGHT_PX: f64 = 800.0;
const MIN_ELEMENT_PX: f64 = 4.0;

/// Per-report budget threaded across the desktop and mobile capture passes so
/// a rule is captured at most once, and the whole report never exceeds
/// [`MAX_ELEMENT_CROPS`]. Desktop runs first, so it naturally wins when a
/// violation is confirmed in both viewports (see `merge_wcag_violations` in
/// `audit/pipeline.rs`, which also propagates the crop across the merge).
pub struct ElementEvidenceBudget {
    captured_rules: HashSet<String>,
    remaining: usize,
}

impl ElementEvidenceBudget {
    pub fn new() -> Self {
        Self {
            captured_rules: HashSet::new(),
            remaining: MAX_ELEMENT_CROPS,
        }
    }
}

impl Default for ElementEvidenceBudget {
    fn default() -> Self {
        Self::new()
    }
}

/// Capture one element-evidence crop per confirmed-violation rule, up to the
/// shared [`ElementEvidenceBudget`]. Call once per viewport pass, directly
/// after `enrich_violations_with_page` (same hook point, same page).
pub async fn capture_element_evidence(
    page: &Page,
    violations: &mut [Violation],
    ax_tree: &AXTree,
    viewport_label: &'static str,
    budget: &mut ElementEvidenceBudget,
) {
    let mut captured_this_pass = 0usize;

    for violation in violations.iter_mut() {
        if budget.remaining == 0 {
            break;
        }
        // Only confirmed violations get proof crops — warnings/positives/
        // not-testables aren't findings that need self-contained evidence.
        if violation.kind != FindingKind::Violation {
            continue;
        }
        if violation.evidence_screenshot.is_some() {
            continue;
        }
        if budget.captured_rules.contains(&violation.rule) {
            continue;
        }

        // Contrast (and any other synthetic node_id) has no AXTree entry —
        // this lookup returning `None` is the exclusion mechanism described
        // in the module doc comment, not a bug.
        let Some(backend_id) = ax_tree
            .get_node(&violation.node_id)
            .and_then(|n| n.backend_dom_node_id)
        else {
            continue;
        };

        if let Some(bytes) = capture_one(page, backend_id).await {
            violation.evidence_screenshot = Some(bytes);
            violation.evidence_viewport = Some(viewport_label);
            budget.captured_rules.insert(violation.rule.clone());
            budget.remaining -= 1;
            captured_this_pass += 1;
        }
    }

    if captured_this_pass > 0 {
        info!(
            "Captured {} element crop(s) during the {} pass ({} remaining of {})",
            captured_this_pass, viewport_label, budget.remaining, MAX_ELEMENT_CROPS
        );
    }
}

/// Resolve, highlight, and screenshot a single backend DOM node. Returns
/// `None` on any failure — the caller treats a missing crop as acceptable.
async fn capture_one(page: &Page, backend_node_id: i64) -> Option<Vec<u8>> {
    let resolve = ResolveNodeParams::builder()
        .backend_node_id(BackendNodeId::new(backend_node_id))
        .build();
    let resolved = page
        .execute(resolve)
        .await
        .map_err(|e| warn!("DOM.resolveNode failed during evidence capture: {}", e))
        .ok()?;
    let object_id = resolved.object.object_id.clone()?;

    // Scroll the element into view, read its rect + the viewport size, save
    // the previous inline outline (to restore later), and apply the
    // highlight — all in one round trip.
    let setup_js = r#"function() {
        this.scrollIntoView({block: 'center', inline: 'center'});
        const r = this.getBoundingClientRect();
        const prevOutline = this.style.outline;
        const prevOutlineOffset = this.style.outlineOffset;
        this.style.outline = '2px solid #C0392B';
        this.style.outlineOffset = '2px';
        return {
            x: r.x, y: r.y, width: r.width, height: r.height,
            vw: window.innerWidth || 1280, vh: window.innerHeight || 800,
            prevOutline: prevOutline, prevOutlineOffset: prevOutlineOffset
        };
    }"#;

    let call = CallFunctionOnParams::builder()
        .function_declaration(setup_js)
        .object_id(object_id.clone())
        .return_by_value(true)
        .build()
        .ok()?;

    let setup_result = page
        .execute(call)
        .await
        .map_err(|e| warn!("Element-evidence setup call failed: {}", e))
        .ok()?;
    let value = setup_result.result.result.value.clone()?;

    let x = value.get("x").and_then(|v| v.as_f64());
    let y = value.get("y").and_then(|v| v.as_f64());
    let width = value.get("width").and_then(|v| v.as_f64());
    let height = value.get("height").and_then(|v| v.as_f64());
    let (Some(x), Some(y), Some(width), Some(height)) = (x, y, width, height) else {
        return None;
    };
    let vw = value.get("vw").and_then(|v| v.as_f64()).unwrap_or(1280.0);
    let vh = value.get("vh").and_then(|v| v.as_f64()).unwrap_or(800.0);
    let prev_outline = value
        .get("prevOutline")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let prev_outline_offset = value
        .get("prevOutlineOffset")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    if is_degenerate_rect(width, height) {
        restore_outline(page, &object_id, &prev_outline, &prev_outline_offset).await;
        return None;
    }

    let crop = compute_crop_rect(x, y, width, height, vw, vh);
    if is_degenerate_rect(crop.width, crop.height) {
        restore_outline(page, &object_id, &prev_outline, &prev_outline_offset).await;
        return None;
    }

    let clip = ClipViewport {
        x: crop.x,
        y: crop.y,
        width: crop.width,
        height: crop.height,
        scale: 1.0,
    };
    let shot = page
        .screenshot(
            ScreenshotParams::builder()
                .format(CaptureScreenshotFormat::Png)
                .clip(clip)
                .build(),
        )
        .await;

    restore_outline(page, &object_id, &prev_outline, &prev_outline_offset).await;

    match shot {
        Ok(bytes) => Some(bytes),
        Err(e) => {
            warn!("Element-evidence screenshot capture failed: {}", e);
            None
        }
    }
}

/// Restore the element's previous inline outline style. Best-effort — a
/// failure here doesn't invalidate an already-captured crop, and the mobile
/// pass re-navigates the page anyway (see `plans/evidence-grade-findings.md`
/// risk notes).
async fn restore_outline(
    page: &Page,
    object_id: &RemoteObjectId,
    prev_outline: &str,
    prev_outline_offset: &str,
) {
    let js = format!(
        "function() {{ this.style.outline = {}; this.style.outlineOffset = {}; }}",
        serde_json::to_string(prev_outline).unwrap_or_else(|_| "\"\"".to_string()),
        serde_json::to_string(prev_outline_offset).unwrap_or_else(|_| "\"\"".to_string()),
    );
    let Ok(call) = CallFunctionOnParams::builder()
        .function_declaration(js)
        .object_id(object_id.clone())
        .build()
    else {
        return;
    };
    if let Err(e) = page.execute(call).await {
        warn!(
            "Failed to restore element outline after evidence capture: {}",
            e
        );
    }
}

/// Whether an element's bounding rect is too small/degenerate to crop —
/// `display: none`, a 0×0 tracking pixel, or a barely-visible sliver.
fn is_degenerate_rect(width: f64, height: f64) -> bool {
    width < MIN_ELEMENT_PX || height < MIN_ELEMENT_PX || width * height <= 0.0
}

/// A crop rectangle in CSS pixels, ready for a CDP screenshot clip.
#[derive(Debug, Clone, Copy, PartialEq)]
struct CropRect {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

/// Pad the element rect by [`CROP_PADDING_PX`], cap the crop area so a
/// violation on a huge/`<body>`-sized element doesn't produce a full-page
/// image, then clamp to the viewport bounds (a clip rect can't extend past
/// the captured surface).
fn compute_crop_rect(
    elem_x: f64,
    elem_y: f64,
    elem_w: f64,
    elem_h: f64,
    viewport_w: f64,
    viewport_h: f64,
) -> CropRect {
    let mut x = elem_x - CROP_PADDING_PX;
    let mut y = elem_y - CROP_PADDING_PX;
    let mut w = (elem_w + 2.0 * CROP_PADDING_PX).min(MAX_CROP_WIDTH_PX);
    let mut h = (elem_h + 2.0 * CROP_PADDING_PX).min(MAX_CROP_HEIGHT_PX);

    if x < 0.0 {
        x = 0.0;
    }
    if y < 0.0 {
        y = 0.0;
    }
    if x + w > viewport_w {
        w = (viewport_w - x).max(0.0);
    }
    if y + h > viewport_h {
        h = (viewport_h - y).max(0.0);
    }

    CropRect {
        x,
        y,
        width: w,
        height: h,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pads_a_normal_element_by_the_configured_margin() {
        let crop = compute_crop_rect(100.0, 200.0, 50.0, 20.0, 1280.0, 800.0);
        assert_eq!(crop.x, 88.0);
        assert_eq!(crop.y, 188.0);
        assert_eq!(crop.width, 74.0);
        assert_eq!(crop.height, 44.0);
    }

    #[test]
    fn clamps_negative_offsets_to_the_viewport_origin() {
        let crop = compute_crop_rect(5.0, 5.0, 30.0, 30.0, 1280.0, 800.0);
        assert_eq!(crop.x, 0.0);
        assert_eq!(crop.y, 0.0);
    }

    #[test]
    fn clamps_width_and_height_at_the_viewport_edge() {
        let crop = compute_crop_rect(1250.0, 780.0, 40.0, 40.0, 1280.0, 800.0);
        assert!(crop.x + crop.width <= 1280.0);
        assert!(crop.y + crop.height <= 800.0);
    }

    #[test]
    fn caps_crop_area_for_body_sized_elements() {
        let crop = compute_crop_rect(0.0, 0.0, 4000.0, 3000.0, 1280.0, 800.0);
        assert_eq!(crop.width, MAX_CROP_WIDTH_PX);
        assert_eq!(crop.height, MAX_CROP_HEIGHT_PX);
    }

    #[test]
    fn degenerate_rect_detection() {
        assert!(is_degenerate_rect(0.0, 0.0));
        assert!(is_degenerate_rect(3.0, 50.0));
        assert!(is_degenerate_rect(50.0, 3.9));
        assert!(!is_degenerate_rect(4.0, 4.0));
        assert!(!is_degenerate_rect(50.0, 20.0));
    }

    #[test]
    fn budget_starts_with_the_full_cap() {
        let budget = ElementEvidenceBudget::new();
        assert_eq!(budget.remaining, MAX_ELEMENT_CROPS);
        assert!(budget.captured_rules.is_empty());
    }
}
