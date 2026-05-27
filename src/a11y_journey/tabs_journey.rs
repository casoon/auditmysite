//! Tabs journey: click first tab, verify ArrowRight moves selection.

use chromiumoxide::cdp::js_protocol::runtime::EvaluateParams;
use chromiumoxide::Page;

use crate::audit::normalized::{InteractiveFinding, JourneyStep, JourneyTrace};
use crate::error::Result;
use crate::interaction::{focus, keyboard, pointer, stability};
use crate::patterns::JourneyCandidate;
use crate::taxonomy::Severity;

/// Collect `aria-selected` values for all `[role="tab"]` elements.
const COLLECT_TABS_JS: &str = r#"
(function() {
    var tabs = Array.from(document.querySelectorAll('[role="tab"]'));
    return tabs.map(function(t) { return t.getAttribute('aria-selected'); });
})()
"#;

async fn collect_tab_states(page: &Page) -> Option<Vec<Option<String>>> {
    let params = EvaluateParams::builder()
        .expression(COLLECT_TABS_JS.to_string())
        .return_by_value(true)
        .build()
        .ok()?;
    let result = page.execute(params).await.ok()?;
    let value = result.result.result.value?;
    let arr = value.as_array()?;
    Some(
        arr.iter()
            .map(|v| {
                if v.is_null() {
                    None
                } else {
                    v.as_str().map(|s| s.to_string())
                }
            })
            .collect(),
    )
}

pub async fn test(
    page: &Page,
    candidate: &JourneyCandidate,
    index: usize,
) -> Result<(JourneyTrace, Vec<InteractiveFinding>)> {
    let journey_name = format!("tabs_{index}");
    let mut trace = JourneyTrace {
        journey: journey_name.clone(),
        steps: Vec::new(),
    };
    let mut findings: Vec<InteractiveFinding> = Vec::new();

    let trigger_id = match candidate.trigger_backend_id {
        Some(id) => id,
        None => return Ok((trace, findings)),
    };

    // Click the first tab.
    if let Err(e) = pointer::synthetic_click_backend(page, trigger_id).await {
        tracing::warn!("tabs: click on backend node {trigger_id} failed: {e}");
        return Ok((trace, findings));
    }
    trace.steps.push(JourneyStep {
        action: "synthetic_click".to_string(),
        target: Some(format!("backend_node:{trigger_id}")),
        focus: None,
        result: None,
        snapshot_label: Some("after_first_tab_click".to_string()),
    });

    stability::settle(page).await?;

    let states_after_click = collect_tab_states(page).await;

    // Press ArrowRight to navigate to second tab.
    if let Err(e) = keyboard::press_arrow(page, "Right").await {
        tracing::warn!("tabs: ArrowRight failed: {e}");
        return Ok((trace, findings));
    }
    trace.steps.push(JourneyStep {
        action: "arrow_right".to_string(),
        target: None,
        focus: None,
        result: None,
        snapshot_label: None,
    });

    stability::settle(page).await?;

    let states_after_arrow = collect_tab_states(page).await;
    let focus_snap = focus::capture_focus(page).await?;

    // Check: aria-selected changed for at least one tab.
    let selection_moved = match (&states_after_click, &states_after_arrow) {
        (Some(before), Some(after)) => before != after,
        _ => true, // Can't determine, don't emit false positive.
    };

    trace.steps.push(JourneyStep {
        action: "check_selection".to_string(),
        target: None,
        focus: focus_snap.selector.clone(),
        result: Some(if selection_moved {
            "selection_moved".to_string()
        } else {
            "selection_unchanged".to_string()
        }),
        snapshot_label: None,
    });

    if !selection_moved {
        findings.push(InteractiveFinding {
            category: "TabsJourney".to_string(),
            severity: Severity::High,
            journey: journey_name.clone(),
            before_snapshot_label: Some("after_first_tab_click".to_string()),
            after_snapshot_label: None,
            message: "Arrow key navigation does not move selection between tabs. \
                Keyboard users cannot navigate the tab list."
                .to_string(),
            fix_suggestion: Some(
                "Implement the roving tabindex pattern: ArrowRight moves focus and \
                aria-selected to the next tab."
                    .to_string(),
            ),
        });
    }

    // Check: focus is on a tab element after the arrow key press.
    let focus_on_tab = focus_snap
        .selector
        .as_deref()
        .map(|s| {
            // Heuristic: if we can read focus and it's not body, assume it's on a tab.
            !s.to_lowercase().contains("body")
        })
        .unwrap_or(false);

    if !focus_on_tab {
        findings.push(InteractiveFinding {
            category: "TabsJourney".to_string(),
            severity: Severity::Medium,
            journey: journey_name,
            before_snapshot_label: None,
            after_snapshot_label: None,
            message: "After pressing ArrowRight in the tab list, focus is not on a tab element."
                .to_string(),
            fix_suggestion: Some(
                "Ensure arrow key navigation also moves focus (not just selection) \
                to the next tab in the roving tabindex pattern."
                    .to_string(),
            ),
        });
    }

    Ok((trace, findings))
}
