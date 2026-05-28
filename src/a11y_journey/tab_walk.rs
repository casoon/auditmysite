//! Tab-walk journey: press Tab N times and record where focus ends up.
//!
//! Records a reproducible `JourneyTrace` along with the per-step
//! `FocusSnapshot`s. The trace is the evidence; `evaluate::tab_walk()`
//! turns it into `InteractiveFinding`s.

use chromiumoxide::Page;

use crate::accessibility::FocusSnapshot;
use crate::audit::normalized::{JourneyStep, JourneyTrace};
use crate::error::Result;
use crate::interaction::{focus, keyboard, stability};

/// Record of one tab-walk run: the trace plus the rich `FocusSnapshot`
/// per step. Snapshots are kept alongside (not embedded in the trace)
/// so the JSON output stays compact.
pub struct TabWalkRecord {
    pub trace: JourneyTrace,
    /// `snapshots[i]` corresponds to `trace.steps[i]`.
    pub snapshots: Vec<FocusSnapshot>,
    /// Selectors of focusable elements as they appear in the DOM, captured
    /// *before* the walk starts. Used to detect reverse jumps in tab order.
    pub dom_order: Vec<String>,
}

/// Record a tab walk of up to `max_steps` Tab presses.
///
/// Stops early when focus stops changing (loop or trap detected) or when
/// no focusable element remains. The returned record is *evidence only*;
/// findings come from the evaluator.
pub async fn record(page: &Page, max_steps: usize) -> Result<TabWalkRecord> {
    let mut trace = JourneyTrace {
        journey: "tab_walk".to_string(),
        steps: Vec::with_capacity(max_steps),
    };
    let mut snapshots: Vec<FocusSnapshot> = Vec::with_capacity(max_steps);

    // Capture the DOM order of focusable elements *before* the walk so the
    // evaluator can detect reverse jumps. Empty list on JS failure — the
    // evaluator treats that as "no order data".
    let dom_order = focus::collect_focusable_dom_order(page).await;

    // Initial focus snapshot — usually body / no element.
    let start = focus::capture_focus(page).await?;
    trace.steps.push(JourneyStep {
        action: "start".to_string(),
        target: None,
        focus: start.selector.clone(),
        result: None,
        snapshot_label: Some("initial".to_string()),
    });
    let mut last_focus_selector = start.selector.clone();
    snapshots.push(start);

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
        snapshots.push(snap);

        if stuck {
            break;
        }
        last_focus_selector = current;
    }

    Ok(TabWalkRecord {
        trace,
        snapshots,
        dom_order,
    })
}
