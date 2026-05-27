//! Modal journey: open dialog, verify focus trap, Escape closes, focus restores.

use chromiumoxide::cdp::js_protocol::runtime::EvaluateParams;
use chromiumoxide::Page;

use crate::audit::normalized::{InteractiveFinding, JourneyStep, JourneyTrace};
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

pub async fn test(
    page: &Page,
    candidate: &JourneyCandidate,
    index: usize,
) -> Result<(JourneyTrace, Vec<InteractiveFinding>)> {
    let journey_name = format!("modal_{index}");
    let mut trace = JourneyTrace {
        journey: journey_name.clone(),
        steps: Vec::new(),
    };
    let mut findings: Vec<InteractiveFinding> = Vec::new();

    let trigger_id = match candidate.trigger_backend_id {
        Some(id) => id,
        None => return Ok((trace, findings)),
    };

    // Capture focus before opening (for restoration check later).
    let trigger_snap = focus::capture_focus(page).await?;
    let trigger_selector = trigger_snap.selector.clone();

    // Click trigger to open the modal.
    if let Err(e) = pointer::synthetic_click_backend(page, trigger_id).await {
        tracing::warn!("modal: click on backend node {trigger_id} failed: {e}");
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

    // 1. Check focus moved inside dialog.
    let focus_in_dialog = eval_bool(
        page,
        "document.querySelector('[role=\"dialog\"],[role=\"alertdialog\"]')?.contains(document.activeElement) ?? false",
    )
    .await
    .unwrap_or(false);

    let focus_snap = focus::capture_focus(page).await?;
    trace.steps.push(JourneyStep {
        action: "check_focus_in_dialog".to_string(),
        target: None,
        focus: focus_snap.selector.clone(),
        result: Some(if focus_in_dialog {
            "focus_inside_dialog".to_string()
        } else {
            "focus_not_in_dialog".to_string()
        }),
        snapshot_label: None,
    });

    if !focus_in_dialog {
        findings.push(InteractiveFinding {
            category: "FocusTrap".to_string(),
            severity: Severity::High,
            journey: journey_name.clone(),
            before_snapshot_label: None,
            after_snapshot_label: Some("after_open_click".to_string()),
            message: "After opening modal, focus did not move inside the dialog. \
                Keyboard users cannot interact with it."
                .to_string(),
            fix_suggestion: Some(
                "Move focus to the first focusable element inside the dialog when it opens, \
                or to the dialog element itself (tabindex=\"-1\")."
                    .to_string(),
            ),
        });
    }

    // 2. Check background is inert/aria-hidden.
    let background_hidden = eval_bool(
        page,
        "document.querySelector('[aria-hidden=\"true\"]') !== null",
    )
    .await
    .unwrap_or(true); // If we can't read, don't emit a false positive.

    if !background_hidden {
        findings.push(InteractiveFinding {
            category: "FocusTrap".to_string(),
            severity: Severity::High,
            journey: journey_name.clone(),
            before_snapshot_label: None,
            after_snapshot_label: Some("after_open_click".to_string()),
            message: "Background content is not hidden from assistive technology when modal is open."
                .to_string(),
            fix_suggestion: Some(
                "Set aria-hidden=\"true\" on the application root when a modal is open, \
                or use the inert attribute."
                    .to_string(),
            ),
        });
    }

    // 3. Focus trap check: press Tab 3 times, verify focus stays in dialog.
    let mut focus_escaped = false;
    for i in 0..3 {
        if let Err(e) = keyboard::press_tab(page).await {
            tracing::warn!("modal: Tab press {i} failed: {e}");
            break;
        }
        stability::settle(page).await?;

        let still_in = eval_bool(
            page,
            "document.querySelector('[role=\"dialog\"],[role=\"alertdialog\"]')?.contains(document.activeElement) ?? false",
        )
        .await
        .unwrap_or(true);

        let tab_snap = focus::capture_focus(page).await?;
        trace.steps.push(JourneyStep {
            action: "tab".to_string(),
            target: None,
            focus: tab_snap.selector.clone(),
            result: Some(if still_in {
                "focus_in_dialog".to_string()
            } else {
                "focus_escaped".to_string()
            }),
            snapshot_label: None,
        });

        if !still_in {
            focus_escaped = true;
            break;
        }
    }

    if focus_escaped {
        findings.push(InteractiveFinding {
            category: "FocusTrap".to_string(),
            severity: Severity::High,
            journey: journey_name.clone(),
            before_snapshot_label: None,
            after_snapshot_label: None,
            message: "Focus is not trapped inside the modal dialog. \
                Keyboard users can navigate to background content."
                .to_string(),
            fix_suggestion: Some(
                "Intercept Tab and Shift+Tab inside the dialog to cycle focus among \
                dialog descendants only."
                    .to_string(),
            ),
        });
    }

    // 4. Press Escape and check modal closes.
    if let Err(e) = keyboard::press_escape(page).await {
        tracing::warn!("modal: Escape press failed: {e}");
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

    let dialog_closed = eval_bool(
        page,
        "document.querySelector('[role=\"dialog\"],[role=\"alertdialog\"]') === null || \
         document.querySelector('[role=\"dialog\"],[role=\"alertdialog\"]')?.getAttribute('aria-hidden') === 'true'",
    )
    .await
    .unwrap_or(true);

    trace.steps.push(JourneyStep {
        action: "check_dialog_closed".to_string(),
        target: None,
        focus: None,
        result: Some(if dialog_closed {
            "dialog_closed".to_string()
        } else {
            "dialog_still_open".to_string()
        }),
        snapshot_label: None,
    });

    if !dialog_closed {
        findings.push(InteractiveFinding {
            category: "FocusTrap".to_string(),
            severity: Severity::High,
            journey: journey_name.clone(),
            before_snapshot_label: None,
            after_snapshot_label: Some("after_escape".to_string()),
            message: "Escape key does not close the modal. Keyboard users cannot dismiss it."
                .to_string(),
            fix_suggestion: Some(
                "Add a keydown handler on the dialog or document that calls close() \
                or hides the dialog when Escape is pressed."
                    .to_string(),
            ),
        });
        return Ok((trace, findings));
    }

    // 5. Focus restoration: focus should return to trigger, not body.
    let after_close_snap = focus::capture_focus(page).await?;
    let focus_after = after_close_snap.selector.clone();

    trace.steps.push(JourneyStep {
        action: "check_focus_restored".to_string(),
        target: trigger_selector.clone(),
        focus: focus_after.clone(),
        result: None,
        snapshot_label: Some("after_escape".to_string()),
    });

    let focus_on_body = focus_after.is_none()
        || focus_after
            .as_deref()
            .map(|s| {
                let low = s.to_lowercase();
                low == "body" || low == "html"
            })
            .unwrap_or(false);

    if focus_on_body {
        findings.push(InteractiveFinding {
            category: "FocusRestoration".to_string(),
            severity: Severity::Medium,
            journey: journey_name,
            before_snapshot_label: None,
            after_snapshot_label: Some("after_escape".to_string()),
            message: "After closing the modal, focus returned to body instead of the trigger. \
                Keyboard users lose their place on the page."
                .to_string(),
            fix_suggestion: Some(
                "Store a reference to the trigger element before opening the dialog and \
                call trigger.focus() when the dialog closes."
                    .to_string(),
            ),
        });
    }

    Ok((trace, findings))
}

