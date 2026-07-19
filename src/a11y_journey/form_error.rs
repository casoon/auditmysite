//! Form-Error-Announcement journey.
//!
//! Submits a form without filling required fields, then checks whether the
//! error state is properly announced via:
//!   1. A live region (role="alert" or aria-live) appearing in the DOM.
//!   2. aria-invalid="true" set on the field.
//!   3. The field linked to the error message via aria-describedby or
//!      aria-errormessage.
//!
//! Focus strategy (first invalid field, error summary, or prominent live
//! region) is recorded but not counted as a violation on its own.

use chromiumoxide::cdp::js_protocol::runtime::EvaluateParams;
use chromiumoxide::Page;

use crate::audit::normalized::{
    InteractiveFinding, InteractiveFindingKind, InteractiveFindingValues, JourneyStep, JourneyTrace,
};
use crate::error::Result;
use crate::interaction::{focus, pointer, stability};
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

async fn eval_int(page: &Page, js: &str) -> Option<i64> {
    let params = EvaluateParams::builder()
        .expression(js.to_string())
        .return_by_value(true)
        .build()
        .ok()?;
    let result = page.execute(params).await.ok()?;
    result.result.result.value?.as_i64()
}

async fn eval_string(page: &Page, js: &str) -> Option<String> {
    let params = EvaluateParams::builder()
        .expression(js.to_string())
        .return_by_value(true)
        .build()
        .ok()?;
    let result = page.execute(params).await.ok()?;
    result.result.result.value?.as_str().map(|s| s.to_string())
}

/// Check whether a live announcement appeared after submit.
///
/// Returns (live_announcement_present, aria_invalid_count, linked_to_error_count).
async fn check_error_state(page: &Page) -> (bool, usize, usize) {
    // 1. Live region / alert node present.
    let live_present = eval_bool(
        page,
        "document.querySelector('[role=\"alert\"],[aria-live]') !== null",
    )
    .await
    .unwrap_or(false);

    // 2. Count fields with aria-invalid="true".
    let invalid_count = eval_int(
        page,
        "document.querySelectorAll('[aria-invalid=\"true\"]').length",
    )
    .await
    .unwrap_or(0) as usize;

    // 3. Count fields linked to an error message (aria-describedby or aria-errormessage).
    let linked_count = eval_int(
        page,
        r#"(function() {
            var fields = document.querySelectorAll(
                'input[aria-invalid="true"], textarea[aria-invalid="true"], select[aria-invalid="true"]'
            );
            var linked = 0;
            Array.from(fields).forEach(function(f) {
                if (f.getAttribute('aria-describedby') || f.getAttribute('aria-errormessage')) {
                    linked++;
                }
            });
            return linked;
        })()"#,
    )
    .await
    .unwrap_or(0) as usize;

    (live_present, invalid_count, linked_count)
}

pub async fn test(
    page: &Page,
    candidate: &JourneyCandidate,
    index: usize,
) -> Result<(JourneyTrace, Vec<InteractiveFinding>)> {
    let journey_name = format!("form_error_{index}");
    let mut trace = JourneyTrace {
        journey: journey_name.clone(),
        steps: Vec::new(),
    };
    let mut findings: Vec<InteractiveFinding> = Vec::new();

    // Capture baseline — no errors yet.
    let (live_before, invalid_before, _linked_before) = check_error_state(page).await;

    trace.steps.push(JourneyStep {
        action: "check_baseline_errors".to_string(),
        target: None,
        focus: None,
        result: Some(format!(
            "live_region:{}, aria_invalid:{}",
            live_before, invalid_before
        )),
        snapshot_label: Some("before_submit".to_string()),
    });

    // Capture the URL before submit so we can later distinguish a silently
    // swallowed error from a navigation (HTML5 native validation / server-side
    // validation page both change the URL).
    let initial_href = eval_string(page, "window.location.href").await;

    // Click submit trigger without filling required fields.
    let Some(trigger_id) = candidate.trigger_backend_id else {
        return Ok((trace, findings));
    };

    if let Err(e) = pointer::synthetic_click_backend(page, trigger_id).await {
        tracing::warn!("form_error: click on backend node {trigger_id} failed: {e}");
        return Ok((trace, findings));
    }
    trace.steps.push(JourneyStep {
        action: "synthetic_click".to_string(),
        target: Some(format!("backend_node:{trigger_id}")),
        focus: None,
        result: None,
        snapshot_label: Some("after_submit_click".to_string()),
    });

    stability::settle(page).await?;

    // Check error state after submit.
    let (live_after, invalid_after, linked_after) = check_error_state(page).await;

    // Did any new live announcements appear?
    let new_live = live_after && !live_before;
    // Did any fields become aria-invalid?
    let new_invalid = invalid_after > invalid_before;

    // Focus position after submit.
    let focus_snap = focus::capture_focus(page).await?;
    let focus_sel = focus_snap.selector.clone();
    let focus_on_body = focus_sel.is_none()
        || focus_sel
            .as_deref()
            .map(|s| {
                let l = s.to_lowercase();
                l == "body" || l == "html"
            })
            .unwrap_or(false);

    trace.steps.push(JourneyStep {
        action: "check_error_state".to_string(),
        target: None,
        focus: focus_sel,
        result: Some(format!(
            "live_region:{new_live}, aria_invalid:{new_invalid}, linked:{linked_after}, focus_on_body:{focus_on_body}"
        )),
        snapshot_label: Some("after_submit_click".to_string()),
    });

    // If neither live region nor aria-invalid appeared, the form submits
    // silently — errors are not announced at all.
    if !new_live && !new_invalid {
        // Could be: (a) form performs HTML5 native validation or navigates to a
        // server-side validation/success page (OK — the error is not silently
        // swallowed), or (b) the form silently swallows the error (bad). We
        // distinguish the two by comparing the URL against the pre-submit value:
        // a changed URL means a navigation happened, so we do not flag it.
        let url_changed = match (
            &initial_href,
            eval_string(page, "window.location.href").await,
        ) {
            (Some(before), Some(after)) => before != &after,
            // If we could not read the URL on either side, fall back to treating
            // it as "no navigation" so a genuinely silent form is still caught.
            _ => false,
        };

        if url_changed {
            return Ok((trace, findings));
        }

        // No navigation and neither aria-invalid nor a live region appeared:
        // the form swallowed the error silently.
        findings.push(InteractiveFinding::new(
            "FormError",
            InteractiveFindingKind::FormErrorSilentFailure,
            None,
            Severity::High,
            journey_name.clone(),
            Some("before_submit".to_string()),
            Some("after_submit_click".to_string()),
            InteractiveFindingValues::default(),
        ));
        return Ok((trace, findings));
    }

    // aria-invalid appeared but no live announcement.
    if new_invalid && !new_live {
        findings.push(InteractiveFinding::new(
            "FormError",
            InteractiveFindingKind::FormErrorInvalidWithoutLiveRegion,
            None,
            Severity::High,
            journey_name.clone(),
            Some("before_submit".to_string()),
            Some("after_submit_click".to_string()),
            InteractiveFindingValues::default(),
        ));
    }

    // Invalid fields not linked to their error message.
    if new_invalid && linked_after < invalid_after {
        let unlinked = invalid_after - linked_after;
        findings.push(InteractiveFinding::new(
            "FormError",
            InteractiveFindingKind::FormErrorUnlinkedFields,
            None,
            Severity::Medium,
            journey_name.clone(),
            Some("before_submit".to_string()),
            Some("after_submit_click".to_string()),
            InteractiveFindingValues {
                count: Some(unlinked as u32),
                ..Default::default()
            },
        ));
    }

    // An error was announced (live region and/or aria-invalid), but focus
    // stayed on the document body instead of moving to the error — the user
    // is not led to what needs fixing. Independent of the two checks above
    // (a form can correctly expose aria-invalid/a live region and still
    // fail to manage focus).
    if (new_live || new_invalid) && focus_on_body {
        findings.push(InteractiveFinding::new(
            "FormError",
            InteractiveFindingKind::FormErrorFocusNotManaged,
            None,
            Severity::Medium,
            journey_name.clone(),
            Some("before_submit".to_string()),
            Some("after_submit_click".to_string()),
            InteractiveFindingValues::default(),
        ));
    }

    Ok((trace, findings))
}
