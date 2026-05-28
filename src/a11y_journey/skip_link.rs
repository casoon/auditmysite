//! Skip-link journey: activate a skip link and verify focus moves to target.

use chromiumoxide::Page;

use crate::audit::normalized::{InteractiveFinding, JourneyStep, JourneyTrace};
use crate::error::Result;
use crate::interaction::{focus, pointer, stability};
use crate::patterns::JourneyCandidate;
use crate::taxonomy::Severity;

pub async fn test(
    page: &Page,
    candidate: &JourneyCandidate,
    index: usize,
) -> Result<(JourneyTrace, Vec<InteractiveFinding>)> {
    let journey_name = format!("skip_link_{index}");
    let mut trace = JourneyTrace {
        journey: journey_name.clone(),
        steps: Vec::new(),
    };
    let mut findings: Vec<InteractiveFinding> = Vec::new();

    let trigger_id = match candidate.trigger_backend_id {
        Some(id) => id,
        None => return Ok((trace, findings)),
    };

    // Activate the skip link via synthetic click.
    if let Err(e) = pointer::synthetic_click_backend(page, trigger_id).await {
        tracing::warn!("skip_link: click on backend node {trigger_id} failed: {e}");
        return Ok((trace, findings));
    }
    trace.steps.push(JourneyStep {
        action: "synthetic_click".to_string(),
        target: Some(format!("backend_node:{trigger_id}")),
        focus: None,
        result: None,
        snapshot_label: Some("after_skip_click".to_string()),
    });

    stability::settle(page).await?;

    // Capture focus after activation.
    let snap = focus::capture_focus(page).await?;
    let focus_selector = snap.selector.clone();

    // Determine if focus actually moved to a meaningful target.
    let focus_on_body = focus_selector.is_none()
        || focus_selector
            .as_deref()
            .map(|s| {
                let low = s.to_lowercase();
                low == "body" || low == "html"
            })
            .unwrap_or(false);

    trace.steps.push(JourneyStep {
        action: "check_focus".to_string(),
        target: None,
        focus: focus_selector.clone(),
        result: Some(if focus_on_body {
            "focus_not_moved".to_string()
        } else {
            "focus_moved".to_string()
        }),
        snapshot_label: Some("after_skip_link".to_string()),
    });

    if focus_on_body {
        findings.push(InteractiveFinding {
            category: "SkipLink".to_string(),
            maps_to_finding: None,
            severity: Severity::High,
            journey: journey_name,
            before_snapshot_label: None,
            after_snapshot_label: Some("after_skip_link".to_string()),
            message: "Skip link is present but does not move focus to the target. \
                Keyboard users cannot bypass navigation."
                .to_string(),
            fix_suggestion: Some(
                "Ensure the skip link target has tabindex=\"-1\" and receives focus via \
                an anchor link, or explicitly call target.focus() after navigation."
                    .to_string(),
            ),
        });
    }

    Ok((trace, findings))
}
