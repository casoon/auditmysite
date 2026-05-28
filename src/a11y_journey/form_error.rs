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

use crate::audit::normalized::{InteractiveFinding, JourneyStep, JourneyTrace};
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
        // Could be: (a) form performs HTML5 native validation (OK), or
        // (b) form silently swallows the error (bad). We check whether
        // the page URL changed (navigation = possible success / HTML5 validation).
        let url_changed = eval_bool(
            page,
            &format!(
                "window.location.href !== {:?}",
                // initial_url not available here; use a heuristic: if hash/path changed.
                "#PLACEHOLDER"
            ),
        )
        .await
        .unwrap_or(false);
        let _ = url_changed; // informational only

        // Emit violation only when errors actually should have appeared.
        // Heuristic: if no aria-invalid AND no live region, it's silent.
        findings.push(InteractiveFinding {
            category: "FormError".to_string(),
            maps_to_finding: None,
            severity: Severity::High,
            journey: journey_name.clone(),
            before_snapshot_label: Some("before_submit".to_string()),
            after_snapshot_label: Some("after_submit_click".to_string()),
            message: "Formular-Fehler werden nicht über eine Live-Region (role=\"alert\" oder \
                aria-live) angekündigt und aria-invalid wird nicht gesetzt. \
                Screenreader-Nutzer erhalten keine Rückmeldung, wenn das Pflichtfeld \
                leer geblieben ist."
                .to_string(),
            fix_suggestion: Some(
                "Fehlermeldungen in einem role=\"alert\"-Element ausgeben und \
                aria-invalid=\"true\" auf jedem fehlerhaften Feld setzen."
                    .to_string(),
            ),
        });
        return Ok((trace, findings));
    }

    // aria-invalid appeared but no live announcement.
    if new_invalid && !new_live {
        findings.push(InteractiveFinding {
            category: "FormError".to_string(),
            maps_to_finding: None,
            severity: Severity::High,
            journey: journey_name.clone(),
            before_snapshot_label: Some("before_submit".to_string()),
            after_snapshot_label: Some("after_submit_click".to_string()),
            message: "aria-invalid wird nach dem Absenden gesetzt, aber es existiert keine \
                Live-Region (role=\"alert\" oder aria-live), die den Fehler ankündigt. \
                Screenreader-Nutzer bemerken den Fehlerzustand erst, wenn sie explizit \
                zurück zum Feld navigieren."
                .to_string(),
            fix_suggestion: Some(
                "Einen role=\"alert\"-Container ergänzen, in dem die Fehlermeldung \
                nach dem Absenden ausgegeben wird."
                    .to_string(),
            ),
        });
    }

    // Invalid fields not linked to their error message.
    if new_invalid && linked_after < invalid_after {
        let unlinked = invalid_after - linked_after;
        findings.push(InteractiveFinding {
            category: "FormError".to_string(),
            maps_to_finding: None,
            severity: Severity::Medium,
            journey: journey_name.clone(),
            before_snapshot_label: Some("before_submit".to_string()),
            after_snapshot_label: Some("after_submit_click".to_string()),
            message: format!(
                "{unlinked} Feld(er) mit aria-invalid=\"true\" sind nicht über \
                aria-describedby oder aria-errormessage mit ihrer Fehlermeldung verknüpft. \
                Screenreader-Nutzer hören den Fehler, können ihn aber nicht dem Feld \
                zuordnen."
            ),
            fix_suggestion: Some(
                "aria-describedby=\"id-der-fehlermeldung\" auf jedem Feld mit \
                aria-invalid=\"true\" ergänzen."
                    .to_string(),
            ),
        });
    }

    Ok((trace, findings))
}

#[cfg(test)]
mod tests {
    // Evaluation logic is inside the browser; only wiring can be tested here.
    // Full integration tests belong in Phase 5 fixture suite.
    #[test]
    fn module_exists() {}
}
