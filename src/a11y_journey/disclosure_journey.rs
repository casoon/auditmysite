//! Disclosure / accordion journey: click trigger, verify aria-expanded toggles.

use chromiumoxide::cdp::js_protocol::runtime::EvaluateParams;
use chromiumoxide::Page;

use crate::audit::normalized::{
    InteractiveFinding, InteractiveFindingKind, InteractiveFindingValues, JourneyStep, JourneyTrace,
};
use crate::error::Result;
use crate::interaction::{pointer, stability};
use crate::patterns::JourneyCandidate;
use crate::taxonomy::Severity;

/// JS that collects all `[aria-expanded]` buttons and their current state.
const COLLECT_EXPANDED_JS: &str = r#"
(function() {
    var els = Array.from(document.querySelectorAll('[aria-expanded]'));
    return els.map(function(e) {
        return e.getAttribute('aria-expanded');
    });
})()
"#;

async fn collect_expanded_states(page: &Page) -> Option<Vec<String>> {
    let params = EvaluateParams::builder()
        .expression(COLLECT_EXPANDED_JS.to_string())
        .return_by_value(true)
        .build()
        .ok()?;
    let result = page.execute(params).await.ok()?;
    let value = result.result.result.value?;
    let arr = value.as_array()?;
    Some(
        arr.iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect(),
    )
}

pub async fn test(
    page: &Page,
    candidate: &JourneyCandidate,
    index: usize,
) -> Result<(JourneyTrace, Vec<InteractiveFinding>)> {
    let journey_name = format!("disclosure_{index}");
    let mut trace = JourneyTrace {
        journey: journey_name.clone(),
        steps: Vec::new(),
    };
    let mut findings: Vec<InteractiveFinding> = Vec::new();

    let trigger_id = match candidate.trigger_backend_id {
        Some(id) => id,
        None => return Ok((trace, findings)),
    };

    // Capture state before first click.
    let before_states = collect_expanded_states(page).await;

    // First click: should open the disclosure.
    if let Err(e) = pointer::synthetic_click_backend(page, trigger_id).await {
        tracing::warn!("disclosure: click on backend node {trigger_id} failed: {e}");
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

    let after_open_states = collect_expanded_states(page).await;

    // Check whether any button flipped from "false" to "true".
    let opened = match (&before_states, &after_open_states) {
        (Some(before), Some(after)) => before
            .iter()
            .zip(after.iter())
            .any(|(b, a)| b == "false" && a == "true"),
        // If we couldn't read states, assume it may have worked (no false positive).
        _ => true,
    };

    trace.steps.push(JourneyStep {
        action: "check_expanded".to_string(),
        target: None,
        focus: None,
        result: Some(if opened {
            "expanded_true".to_string()
        } else {
            "expanded_unchanged".to_string()
        }),
        snapshot_label: None,
    });

    if !opened {
        findings.push(InteractiveFinding::new(
            "StateTransition",
            InteractiveFindingKind::DisclosureNotOpened,
            None,
            Severity::High,
            journey_name.clone(),
            None,
            Some("after_open_click".to_string()),
            InteractiveFindingValues::default(),
        ));
        // Skip the second click if the first didn't work.
        return Ok((trace, findings));
    }

    // Second click: should close the disclosure.
    if let Err(e) = pointer::synthetic_click_backend(page, trigger_id).await {
        tracing::warn!("disclosure: second click on backend node {trigger_id} failed: {e}");
        return Ok((trace, findings));
    }
    trace.steps.push(JourneyStep {
        action: "synthetic_click".to_string(),
        target: Some(format!("backend_node:{trigger_id}")),
        focus: None,
        result: None,
        snapshot_label: Some("after_close_click".to_string()),
    });

    stability::settle(page).await?;

    let after_close_states = collect_expanded_states(page).await;

    // Check whether it returned to closed (flipped back from "true" to "false").
    let closed = match (&after_open_states, &after_close_states) {
        (Some(open), Some(closed)) => open
            .iter()
            .zip(closed.iter())
            .any(|(o, c)| o == "true" && c == "false"),
        _ => true,
    };

    trace.steps.push(JourneyStep {
        action: "check_collapsed".to_string(),
        target: None,
        focus: None,
        result: Some(if closed {
            "collapsed_true".to_string()
        } else {
            "collapsed_failed".to_string()
        }),
        snapshot_label: None,
    });

    if !closed {
        findings.push(InteractiveFinding::new(
            "StateTransition",
            InteractiveFindingKind::DisclosureNotClosed,
            None,
            Severity::Medium,
            journey_name,
            Some("after_open_click".to_string()),
            Some("after_close_click".to_string()),
            InteractiveFindingValues::default(),
        ));
    }

    Ok((trace, findings))
}
