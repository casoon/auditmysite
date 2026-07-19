//! Quantity-stepper operability journey (SC 2.1.1 Keyboard, SC 4.1.2
//! Name/Role/Value).
//!
//! Focuses the detected spinbutton, presses ArrowUp via CDP (a real
//! keyboard event, not a synthetic DOM one), and checks (a) whether the
//! value actually changed — keyboard operability — and (b) for non-native
//! ARIA spinbutton widgets, whether `aria-valuenow` was kept in sync with
//! that change — value exposure. Native `<input type="number">` elements
//! don't need `aria-valuenow`; their value is exposed to the accessibility
//! tree automatically, so they are exempt from check (b).
//!
//! Only ArrowUp is exercised (not ArrowDown or a full min/max walk) and
//! only the one spinbutton the pattern detector found — this is a single
//! representative-instance probe, not an exhaustive stepper audit. The
//! "separate +/- button pair, no single valued element" implementation is
//! not covered (see `patterns::quantity_stepper`'s module doc).
//!
//! Safety: focus + one key press only, no navigation, no state left behind
//! beyond the field's own value (an ephemeral browser context is the
//! cleanup, same convention as the other commerce journeys).

use chromiumoxide::cdp::browser_protocol::dom::{BackendNodeId, ResolveNodeParams};
use chromiumoxide::cdp::js_protocol::runtime::CallFunctionOnParams;
use chromiumoxide::Page;

use crate::audit::normalized::{
    InteractiveFinding, InteractiveFindingKind, InteractiveFindingValues, JourneyStep, JourneyTrace,
};
use crate::error::Result;
use crate::interaction::{keyboard, stability};
use crate::patterns::JourneyCandidate;
use crate::taxonomy::Severity;

/// Resolve a backend node id and call a JS function on it, returning its
/// string result (or `None` on any failure/undefined/null return).
async fn call_on_backend_string(
    page: &Page,
    backend_node_id: i64,
    js_fn_body: &str,
) -> Option<String> {
    let resolve = ResolveNodeParams::builder()
        .backend_node_id(BackendNodeId::new(backend_node_id))
        .build();
    let resolved = page.execute(resolve).await.ok()?;
    let object_id = resolved.result.object.object_id.clone()?;

    let call = CallFunctionOnParams::builder()
        .function_declaration(js_fn_body.to_string())
        .object_id(object_id)
        .return_by_value(true)
        .await_promise(false)
        .build()
        .ok()?;
    let result = page.execute(call).await.ok()?;
    result.result.result.value?.as_str().map(|s| s.to_string())
}

/// Same as `call_on_backend_string` but for a boolean result.
async fn call_on_backend_bool(page: &Page, backend_node_id: i64, js_fn_body: &str) -> Option<bool> {
    let resolve = ResolveNodeParams::builder()
        .backend_node_id(BackendNodeId::new(backend_node_id))
        .build();
    let resolved = page.execute(resolve).await.ok()?;
    let object_id = resolved.result.object.object_id.clone()?;

    let call = CallFunctionOnParams::builder()
        .function_declaration(js_fn_body.to_string())
        .object_id(object_id)
        .return_by_value(true)
        .await_promise(false)
        .build()
        .ok()?;
    let result = page.execute(call).await.ok()?;
    result.result.result.value?.as_bool()
}

async fn read_value(page: &Page, backend_node_id: i64) -> Option<String> {
    call_on_backend_string(
        page,
        backend_node_id,
        "function() { return (this.value !== undefined && this.value !== null) ? String(this.value) : ''; }",
    )
    .await
}

async fn read_valuenow(page: &Page, backend_node_id: i64) -> Option<String> {
    call_on_backend_string(
        page,
        backend_node_id,
        "function() { return this.getAttribute('aria-valuenow'); }",
    )
    .await
}

async fn is_native_input(page: &Page, backend_node_id: i64) -> bool {
    call_on_backend_bool(
        page,
        backend_node_id,
        "function() { return !!(this.tagName && this.tagName.toLowerCase() === 'input'); }",
    )
    .await
    .unwrap_or(false)
}

/// Focus the element and confirm focus actually landed on it (some
/// elements — e.g. a non-focusable wrapper — silently no-op on `.focus()`).
async fn focus_and_confirm(page: &Page, backend_node_id: i64) -> bool {
    call_on_backend_bool(
        page,
        backend_node_id,
        "function() { this.focus(); return document.activeElement === this; }",
    )
    .await
    .unwrap_or(false)
}

pub async fn test(
    page: &Page,
    candidate: &JourneyCandidate,
    index: usize,
) -> Result<(JourneyTrace, Vec<InteractiveFinding>)> {
    let journey_name = format!("quantity_stepper_{index}");
    let mut trace = JourneyTrace {
        journey: journey_name.clone(),
        steps: Vec::new(),
    };
    let mut findings: Vec<InteractiveFinding> = Vec::new();

    let Some(stepper_id) = candidate.trigger_backend_id else {
        return Ok((trace, findings));
    };

    let value_before = read_value(page, stepper_id).await;
    let valuenow_before = read_valuenow(page, stepper_id).await;

    trace.steps.push(JourneyStep {
        action: "check_baseline".to_string(),
        target: Some(format!("backend_node:{stepper_id}")),
        focus: None,
        result: Some(format!(
            "value:{value_before:?}, valuenow:{valuenow_before:?}"
        )),
        snapshot_label: Some("before_arrow_up".to_string()),
    });

    let focused = focus_and_confirm(page, stepper_id).await;
    trace.steps.push(JourneyStep {
        action: "focus".to_string(),
        target: Some(format!("backend_node:{stepper_id}")),
        focus: None,
        result: Some(format!("focused:{focused}")),
        snapshot_label: Some("after_focus".to_string()),
    });

    if !focused {
        // Could not even focus the control via script — a real Tab-key user
        // would fare no better. This alone is the keyboard-inoperability
        // finding; there is nothing further to probe.
        findings.push(InteractiveFinding::new(
            "QuantityStepper",
            InteractiveFindingKind::QuantityStepperKeyboardInoperable,
            None,
            Severity::High,
            journey_name.clone(),
            Some("before_arrow_up".to_string()),
            Some("after_focus".to_string()),
            InteractiveFindingValues::default(),
        ));
        return Ok((trace, findings));
    }

    keyboard::press(page, "ArrowUp").await?;
    stability::settle(page).await?;

    let value_after = read_value(page, stepper_id).await;
    let valuenow_after = read_valuenow(page, stepper_id).await;
    let native_input = is_native_input(page, stepper_id).await;

    trace.steps.push(JourneyStep {
        action: "arrow_up".to_string(),
        target: Some(format!("backend_node:{stepper_id}")),
        focus: None,
        result: Some(format!(
            "value:{value_after:?}, valuenow:{valuenow_after:?}, native_input:{native_input}"
        )),
        snapshot_label: Some("after_arrow_up".to_string()),
    });

    let value_changed = value_after.is_some() && value_after != value_before;

    if !value_changed {
        // Focus landed, but ArrowUp had no effect on the value — keyboard
        // users cannot operate the control, regardless of mouse behavior.
        findings.push(InteractiveFinding::new(
            "QuantityStepper",
            InteractiveFindingKind::QuantityStepperKeyboardInoperable,
            None,
            Severity::High,
            journey_name.clone(),
            Some("before_arrow_up".to_string()),
            Some("after_arrow_up".to_string()),
            InteractiveFindingValues::default(),
        ));
        return Ok((trace, findings));
    }

    if !native_input && (valuenow_after.is_none() || valuenow_after == valuenow_before) {
        // A non-native ARIA spinbutton widget must keep aria-valuenow in
        // sync itself — there is no other way for assistive technology to
        // learn the new value. Native <input type="number"> is exempt: its
        // value is exposed to the accessibility tree automatically.
        findings.push(InteractiveFinding::new(
            "QuantityStepper",
            InteractiveFindingKind::QuantityStepperValueNotExposed,
            None,
            Severity::Medium,
            journey_name.clone(),
            Some("before_arrow_up".to_string()),
            Some("after_arrow_up".to_string()),
            InteractiveFindingValues::default(),
        ));
    }

    Ok((trace, findings))
}
