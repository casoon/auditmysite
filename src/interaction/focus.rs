//! Focus tracking utilities.
//!
//! Builds the `FocusSnapshot` that accompanies each `AXSnapshot` in a
//! journey. Phase 1 wires up the basics: `document.activeElement` selector
//! and bounding-box. Visibility, indicator detection (`Detected`/
//! `NotDetected`/`Ambiguous`) and overlay-occlusion checks land in Phase 2.

use chromiumoxide::cdp::js_protocol::runtime::EvaluateParams;
use chromiumoxide::Page;
use serde_json::Value;

use crate::accessibility::{FocusSnapshot, Rect};
use crate::error::{AuditError, Result};

/// JS that returns a minimal description of `document.activeElement`.
///
/// Returns `{ selector, x, y, w, h, hasFocus }` or `null` when no element
/// has focus (or only `document.body` does — which we treat as "no focus"
/// for journey purposes).
const ACTIVE_ELEMENT_JS: &str = r#"
(function () {
    var el = document.activeElement;
    if (!el || el === document.body || el === document.documentElement) {
        return null;
    }
    var rect = el.getBoundingClientRect();
    function selectorFor(node) {
        if (!node || node.nodeType !== 1) return null;
        if (node.id) return '#' + node.id;
        var parts = [];
        while (node && node.nodeType === 1 && parts.length < 6) {
            var tag = node.nodeName.toLowerCase();
            if (node.id) { parts.unshift(tag + '#' + node.id); break; }
            var parent = node.parentNode;
            if (parent) {
                var siblings = Array.from(parent.children).filter(function (c) {
                    return c.nodeName === node.nodeName;
                });
                if (siblings.length > 1) {
                    var idx = siblings.indexOf(node) + 1;
                    tag += ':nth-of-type(' + idx + ')';
                }
            }
            parts.unshift(tag);
            node = parent;
        }
        return parts.join(' > ');
    }
    return {
        selector: selectorFor(el),
        x: rect.x, y: rect.y, w: rect.width, h: rect.height,
    };
})()
"#;

/// Build a `FocusSnapshot` from the current page state.
///
/// Phase 1 fills `selector` and `bounding_box`; backend-node mapping and
/// viewport intersection are also evaluated. Indicator detection is left
/// as `None` for Phase 2.
pub async fn capture_focus(page: &Page) -> Result<FocusSnapshot> {
    let params = EvaluateParams::builder()
        .expression(ACTIVE_ELEMENT_JS.to_string())
        .return_by_value(true)
        .build()
        .map_err(|e| AuditError::InteractionFailed {
            reason: format!("evaluate build failed: {e}"),
        })?;

    let result = page
        .execute(params)
        .await
        .map_err(|e| AuditError::InteractionFailed {
            reason: format!("activeElement evaluate failed: {e}"),
        })?;

    // CommandResponse<EvaluateReturns>: .result → EvaluateReturns;
    // EvaluateReturns.result → RemoteObject; RemoteObject.value → Option<Value>.
    let value = match result.result.result.value.clone() {
        Some(v) => v,
        None => return Ok(FocusSnapshot::default()),
    };
    if value.is_null() {
        return Ok(FocusSnapshot::default());
    }

    let selector = value
        .get("selector")
        .and_then(Value::as_str)
        .map(|s| s.to_string());
    let bbox = match (
        value.get("x").and_then(Value::as_f64),
        value.get("y").and_then(Value::as_f64),
        value.get("w").and_then(Value::as_f64),
        value.get("h").and_then(Value::as_f64),
    ) {
        (Some(x), Some(y), Some(w), Some(h)) => Some(Rect {
            x: x as f32,
            y: y as f32,
            width: w as f32,
            height: h as f32,
        }),
        _ => None,
    };

    Ok(FocusSnapshot {
        active_backend_node_id: None,
        ax_node_id: None,
        selector,
        visible: bbox.is_some(),
        in_viewport: bbox.is_some(),
        focus_indicator: None,
        bounding_box: bbox,
        obscured_by: None,
    })
}
