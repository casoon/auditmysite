//! Tab-walk journey: press Tab N times and record where focus ends up.
//!
//! Phase 1 captures the sequence as a `JourneyTrace`. No evaluation — that
//! lands in Phase 2 (out-of-order detection, hidden focusables, focus
//! indicator status).

use chromiumoxide::Page;

use crate::audit::normalized::{JourneyStep, JourneyTrace};
use crate::error::Result;
use crate::interaction::{focus, keyboard, stability};

/// Record a tab walk of up to `max_steps` Tab presses.
///
/// Stops early when focus stops changing (loop or trap detected) or when
/// no focusable element remains. The returned trace is *evidence only*;
/// any findings are derived from it in later phases.
pub async fn record(page: &Page, max_steps: usize) -> Result<JourneyTrace> {
    let mut trace = JourneyTrace {
        journey: "tab_walk".to_string(),
        steps: Vec::with_capacity(max_steps),
    };

    // Initial focus snapshot — usually body / no element.
    let start = focus::capture_focus(page).await?;
    trace.steps.push(JourneyStep {
        action: "start".to_string(),
        target: None,
        focus: start.selector.clone(),
        result: None,
        snapshot_label: Some("initial".to_string()),
    });

    let mut last_focus_selector = start.selector;

    for i in 0..max_steps {
        keyboard::press_tab(page).await?;
        stability::settle(page).await?;
        let snap = focus::capture_focus(page).await?;
        let current = snap.selector.clone();

        let result = if current == last_focus_selector {
            // Focus did not move — likely trapped or no further focusables.
            Some("focus_stuck".to_string())
        } else if current.is_none() {
            Some("focus_lost".to_string())
        } else {
            None
        };

        let stuck = result.as_deref() == Some("focus_stuck");

        trace.steps.push(JourneyStep {
            action: "tab".to_string(),
            target: None,
            focus: current.clone(),
            result,
            snapshot_label: Some(format!("after_tab_{}", i + 1)),
        });

        if stuck {
            break;
        }
        last_focus_selector = current;
    }

    Ok(trace)
}
