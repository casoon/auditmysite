//! Menu journey: open menu trigger, verify focus moves, Escape closes.

use chromiumoxide::cdp::js_protocol::runtime::EvaluateParams;
use chromiumoxide::Page;

use crate::audit::normalized::{
    InteractiveFinding, InteractiveFindingKind, InteractiveFindingValues, JourneyStep, JourneyTrace,
};
use crate::error::Result;
use crate::interaction::{focus, keyboard, pointer, stability};
use crate::patterns::JourneyCandidate;
use crate::taxonomy::Severity;

async fn eval_bool(page: &Page, js: &str) -> Option<bool> {
    let params = EvaluateParams::builder()
        .expression(js.to_string())
        .return_by_value(true)
        .build()
        .ok()?;
    let result = page.execute(params).await.ok()?;
    result.result.result.value?.as_bool()
}

async fn eval_str(page: &Page, js: &str) -> Option<String> {
    let params = EvaluateParams::builder()
        .expression(js.to_string())
        .return_by_value(true)
        .build()
        .ok()?;
    let result = page.execute(params).await.ok()?;
    let value = result.result.result.value?;
    if value.is_null() {
        None
    } else {
        value.as_str().map(|s| s.to_string())
    }
}

pub async fn test(
    page: &Page,
    candidate: &JourneyCandidate,
    index: usize,
) -> Result<(JourneyTrace, Vec<InteractiveFinding>)> {
    let journey_name = format!("menu_{index}");
    let mut trace = JourneyTrace {
        journey: journey_name.clone(),
        steps: Vec::new(),
    };
    let mut findings: Vec<InteractiveFinding> = Vec::new();

    let trigger_id = match candidate.trigger_backend_id {
        Some(id) => id,
        None => return Ok((trace, findings)),
    };

    // Capture aria-expanded before click.
    let expanded_before = eval_str(
        page,
        "document.activeElement ? document.activeElement.getAttribute('aria-expanded') : null",
    )
    .await;

    // Click trigger to open the menu.
    if let Err(e) = pointer::synthetic_click_backend(page, trigger_id).await {
        tracing::warn!("menu: click on backend node {trigger_id} failed: {e}");
        return Ok((trace, findings));
    }
    trace.steps.push(JourneyStep {
        action: "synthetic_click".to_string(),
        target: Some(format!("backend_node:{trigger_id}")),
        focus: None,
        result: None,
        snapshot_label: Some("after_open_click".to_string()),
    });

    stability::settle(page).await?;

    // Check menu expanded: look for aria-expanded="true" on any button, or a
    // visible role="menu" element.
    let menu_visible = eval_bool(
        page,
        "document.querySelector('[role=\"menu\"]') !== null || \
         Array.from(document.querySelectorAll('[aria-expanded]')).some(function(e) { return e.getAttribute('aria-expanded') === 'true'; })",
    )
    .await
    .unwrap_or(false);

    trace.steps.push(JourneyStep {
        action: "check_menu_open".to_string(),
        target: None,
        focus: None,
        result: Some(if menu_visible {
            "menu_open".to_string()
        } else {
            "menu_not_open".to_string()
        }),
        snapshot_label: None,
    });

    if !menu_visible {
        findings.push(InteractiveFinding::new(
            "MenuJourney",
            InteractiveFindingKind::MenuNotOpened,
            None,
            Severity::Medium,
            journey_name.clone(),
            None,
            Some("after_open_click".to_string()),
            InteractiveFindingValues::default(),
        ));
        return Ok((trace, findings));
    }

    // Check focus moved into menu.
    let focus_snap = focus::capture_focus(page).await?;
    let focus_in_menu = eval_bool(
        page,
        "document.querySelector('[role=\"menu\"]')?.contains(document.activeElement) ?? false",
    )
    .await
    .unwrap_or(false);

    trace.steps.push(JourneyStep {
        action: "check_focus_in_menu".to_string(),
        target: None,
        focus: focus_snap.selector.clone(),
        result: Some(if focus_in_menu {
            "focus_in_menu".to_string()
        } else {
            "focus_not_in_menu".to_string()
        }),
        snapshot_label: None,
    });

    // Focus in menu is recommended but not always enforced — emit as Info.
    let _ = expanded_before; // unused for now
    if !focus_in_menu {
        findings.push(InteractiveFinding::new(
            "MenuJourney",
            InteractiveFindingKind::MenuFocusNotMoved,
            None,
            Severity::Low,
            journey_name.clone(),
            None,
            Some("after_open_click".to_string()),
            InteractiveFindingValues::default(),
        ));
    }

    // Press Escape and verify menu closes.
    if let Err(e) = keyboard::press_escape(page).await {
        tracing::warn!("menu: Escape press failed: {e}");
        return Ok((trace, findings));
    }
    trace.steps.push(JourneyStep {
        action: "escape".to_string(),
        target: None,
        focus: None,
        result: None,
        snapshot_label: Some("after_escape".to_string()),
    });

    stability::settle(page).await?;

    // Check menu is now closed.
    let menu_closed = eval_bool(
        page,
        "document.querySelector('[role=\"menu\"]') === null && \
         !Array.from(document.querySelectorAll('[aria-expanded]')).some(function(e) { return e.getAttribute('aria-expanded') === 'true'; })",
    )
    .await
    .unwrap_or(true);

    trace.steps.push(JourneyStep {
        action: "check_menu_closed".to_string(),
        target: None,
        focus: None,
        result: Some(if menu_closed {
            "menu_closed".to_string()
        } else {
            "menu_still_open".to_string()
        }),
        snapshot_label: None,
    });

    if !menu_closed {
        findings.push(InteractiveFinding::new(
            "MenuJourney",
            InteractiveFindingKind::MenuEscapeNotClosing,
            None,
            Severity::High,
            journey_name,
            None,
            Some("after_escape".to_string()),
            InteractiveFindingValues::default(),
        ));
    }

    Ok((trace, findings))
}
