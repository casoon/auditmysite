//! Focus tracking utilities.
//!
//! Builds the `FocusSnapshot` that accompanies each `AXSnapshot` in a
//! journey. Phase 2 adds focus-indicator detection via computed style.

use chromiumoxide::cdp::js_protocol::runtime::EvaluateParams;
use chromiumoxide::Page;
use serde_json::Value;

use crate::accessibility::{FocusIndicatorStatus, FocusSnapshot, Rect};
use crate::error::{AuditError, Result};

/// JS that returns a description of `document.activeElement`, including
/// visibility flags used by the journey evaluator. `null` when no element
/// has focus (or only body/documentElement, which we treat as "no focus").
const ACTIVE_ELEMENT_JS: &str = r#"
(function () {
    var el = document.activeElement;
    if (!el || el === document.body || el === document.documentElement) {
        return null;
    }
    var rect = el.getBoundingClientRect();
    var vw = window.innerWidth || document.documentElement.clientWidth;
    var vh = window.innerHeight || document.documentElement.clientHeight;
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
    // Ancestor-chain checks: a focused element is "hidden" if anywhere on
    // the path to <html> there is aria-hidden="true" or an inert attribute.
    var ariaHiddenChain = false;
    var inertChain = false;
    var n = el;
    while (n && n.nodeType === 1) {
        if (n.getAttribute) {
            if (n.getAttribute('aria-hidden') === 'true') ariaHiddenChain = true;
            if (n.hasAttribute('inert')) inertChain = true;
        }
        n = n.parentNode;
    }
    var style = window.getComputedStyle(el);
    var hiddenByStyle =
        style.display === 'none' ||
        style.visibility === 'hidden' ||
        parseFloat(style.opacity) === 0;
    var inViewport = rect.right > 0 && rect.bottom > 0 && rect.left < vw && rect.top < vh;
    return {
        selector: selectorFor(el),
        x: rect.x, y: rect.y, w: rect.width, h: rect.height,
        ariaHiddenChain: ariaHiddenChain,
        inertChain: inertChain,
        hiddenByStyle: hiddenByStyle,
        inViewport: inViewport,
    };
})()
"#;

/// JS that collects all focusable elements on the page in DOM order and
/// returns their selectors. Used by the tab-walk evaluator to detect
/// reverse jumps in tab order.
///
/// Filter logic mirrors what the browser treats as keyboard-reachable:
/// - explicit interactive elements: `<a href>`, `<button>`, form controls
/// - any element with positive or zero `tabindex`
/// - `contenteditable` regions
/// - excludes `disabled` controls and `tabindex="-1"` (HTMLElement.tabIndex < 0)
const COLLECT_FOCUSABLES_JS: &str = r#"
(function () {
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
    var sel = 'a[href], button, input:not([type="hidden"]), select, textarea, ' +
              '[tabindex], [contenteditable=""], [contenteditable="true"]';
    var els = Array.from(document.querySelectorAll(sel)).filter(function (el) {
        if (el.disabled) return false;
        if (typeof el.tabIndex === 'number' && el.tabIndex < 0) return false;
        return true;
    });
    return els.map(selectorFor).filter(function (s) { return s !== null; });
})()
"#;

/// Collect the in-DOM-order selectors of all focusable elements on the
/// page. Used as the reference order by `evaluate::tab_walk_order`.
///
/// Returns an empty vector on evaluation failure rather than an error —
/// missing DOM-order data simply means we cannot detect out-of-order jumps.
pub async fn collect_focusable_dom_order(page: &Page) -> Vec<String> {
    let Ok(params) = EvaluateParams::builder()
        .expression(COLLECT_FOCUSABLES_JS.to_string())
        .return_by_value(true)
        .build()
    else {
        return Vec::new();
    };
    let Ok(result) = page.execute(params).await else {
        return Vec::new();
    };
    let Some(value) = result.result.result.value.clone() else {
        return Vec::new();
    };
    let Some(arr) = value.as_array() else {
        return Vec::new();
    };
    arr.iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect()
}

/// JS that checks whether the currently focused element has a visible focus
/// indicator via outline, box-shadow, or border changes.
const FOCUS_INDICATOR_JS: &str = r#"
(function() {
    var el = document.activeElement;
    if (!el || el === document.body || el === document.documentElement) return 'ambiguous';
    var cs = window.getComputedStyle(el);
    var outlineStyle = cs.outlineStyle;
    var outlineWidth = cs.outlineWidth;
    var boxShadow = cs.boxShadow;
    var hasOutline = outlineStyle !== 'none' && outlineWidth !== '0px';
    var hasBoxShadow = boxShadow !== 'none';
    if (hasOutline || hasBoxShadow) return 'detected';
    var borderWidth = cs.borderWidth;
    if (borderWidth !== '0px') return 'ambiguous';
    return 'not_detected';
})()
"#;

/// Detect whether the currently focused element has a visible focus indicator.
///
/// Returns `None` if the evaluation fails (e.g. page navigating or no focus).
pub async fn detect_focus_indicator(page: &Page) -> Option<FocusIndicatorStatus> {
    let params = EvaluateParams::builder()
        .expression(FOCUS_INDICATOR_JS.to_string())
        .return_by_value(true)
        .build()
        .ok()?;
    let result = page.execute(params).await.ok()?;
    let value = result.result.result.value?;
    let s = value.as_str()?;
    Some(match s {
        "detected" => FocusIndicatorStatus::Detected,
        "not_detected" => FocusIndicatorStatus::NotDetected,
        _ => FocusIndicatorStatus::Ambiguous,
    })
}

/// Build a `FocusSnapshot` from the current page state.
///
/// Fills selector, bounding_box, visibility flags, and focus-indicator status.
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
    let aria_hidden_chain = value
        .get("ariaHiddenChain")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let inert_chain = value
        .get("inertChain")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let hidden_by_style = value
        .get("hiddenByStyle")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let in_viewport = value
        .get("inViewport")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    // visible = element has non-zero box and is not hidden by style.
    let has_area = bbox.is_some_and(|b| b.width > 0.0 && b.height > 0.0);
    let visible = has_area && !hidden_by_style;

    let focus_indicator = detect_focus_indicator(page).await;

    Ok(FocusSnapshot {
        active_backend_node_id: None,
        ax_node_id: None,
        selector,
        visible,
        in_viewport,
        focus_indicator,
        bounding_box: bbox,
        obscured_by: None,
        aria_hidden_chain,
        inert_chain,
        hidden_by_style,
    })
}
