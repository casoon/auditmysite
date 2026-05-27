//! Pointer input via CDP `Input.dispatchMouseEvent`, with a synthetic
//! `element.click()` fallback.
//!
//! Real mouse events traverse hover, pointerdown/up and click handlers.
//! Synthetic clicks via `Runtime.callFunctionOn` bypass parts of that chain
//! and are therefore marked as `synthetic_click` in journey traces so
//! reviewers know the path was not fully realistic.

use chromiumoxide::cdp::browser_protocol::dom::{BackendNodeId, ResolveNodeParams};
use chromiumoxide::cdp::browser_protocol::input::{
    DispatchMouseEventParams, DispatchMouseEventType, MouseButton,
};
use chromiumoxide::cdp::js_protocol::runtime::CallFunctionOnParams;
use chromiumoxide::layout::Point;
use chromiumoxide::Page;

use crate::error::{AuditError, Result};

/// Dispatch a real mouse click at the given viewport-coordinate point.
/// Mirrors a user pressing and releasing the left button.
pub async fn click_at(page: &Page, point: Point) -> Result<()> {
    // Move first so hover handlers fire.
    page.move_mouse(point)
        .await
        .map_err(|e| AuditError::InteractionFailed {
            reason: format!("mouse move failed: {e}"),
        })?;

    let press = DispatchMouseEventParams::builder()
        .x(point.x)
        .y(point.y)
        .button(MouseButton::Left)
        .click_count(1)
        .r#type(DispatchMouseEventType::MousePressed)
        .build()
        .map_err(|e| AuditError::InteractionFailed {
            reason: format!("mouse press build failed: {e}"),
        })?;
    page.execute(press)
        .await
        .map_err(|e| AuditError::InteractionFailed {
            reason: format!("mouse press dispatch failed: {e}"),
        })?;

    let release = DispatchMouseEventParams::builder()
        .x(point.x)
        .y(point.y)
        .button(MouseButton::Left)
        .click_count(1)
        .r#type(DispatchMouseEventType::MouseReleased)
        .build()
        .map_err(|e| AuditError::InteractionFailed {
            reason: format!("mouse release build failed: {e}"),
        })?;
    page.execute(release)
        .await
        .map_err(|e| AuditError::InteractionFailed {
            reason: format!("mouse release dispatch failed: {e}"),
        })?;

    Ok(())
}

/// Fallback: invoke `element.click()` via `Runtime.callFunctionOn` on the
/// backend node. Used when a real mouse event cannot be placed (element not
/// reachable after scroll, off-viewport overlays, etc.). Journey traces
/// mark these as `synthetic_click`.
pub async fn synthetic_click_backend(page: &Page, backend_node_id: i64) -> Result<()> {
    // Resolve backend node id → remote object id, then call .click().
    let resolve = ResolveNodeParams::builder()
        .backend_node_id(BackendNodeId::new(backend_node_id))
        .build();
    let resolved = page
        .execute(resolve)
        .await
        .map_err(|e| AuditError::InteractionFailed {
            reason: format!("resolve backend node {backend_node_id} failed: {e}"),
        })?;
    let object_id =
        resolved
            .result
            .object
            .object_id
            .clone()
            .ok_or_else(|| AuditError::InteractionFailed {
                reason: format!("resolve returned no objectId for backend node {backend_node_id}"),
            })?;

    let call = CallFunctionOnParams::builder()
        .function_declaration("function() { this.click(); }".to_string())
        .object_id(object_id)
        .await_promise(false)
        .build()
        .map_err(|e| AuditError::InteractionFailed {
            reason: format!("callFunctionOn build failed: {e}"),
        })?;
    page.execute(call)
        .await
        .map_err(|e| AuditError::InteractionFailed {
            reason: format!("synthetic click dispatch failed: {e}"),
        })?;

    Ok(())
}
