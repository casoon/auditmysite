//! Form-Error-Announcement journey.
//!
//! Submits a form without filling required fields, then checks whether the
//! error state is properly announced via:
//!   1. Eine Live-Region, die **gefüllt wird** (beobachtet, nicht abgefragt).
//!   2. aria-invalid="true" set on the field.
//!   3. The field linked to the error message via aria-describedby or
//!      aria-errormessage.
//!
//! Focus strategy (first invalid field, error summary, or prominent live
//! region) is recorded but not counted as a violation on its own.
//!
//! # Warum die Ankündigung beobachtet und nicht abgefragt wird
//!
//! Bis Plan 53 lautete die Prüfung: existiert jetzt ein
//! `[role="alert"],[aria-live]`, das vorher nicht existierte. Daraus folgte
//! `new_live = live_after && !live_before` — und damit war sie für die
//! **empfohlene** Umsetzung blind. Wer den Live-Container von Anfang an leer
//! im Markup stehen hat (so soll es sein, denn Screenreader registrieren
//! Live-Regionen beim Aufbau des Baums), hatte `live_before == true`, also
//! `new_live == false` für immer. Die Folge war ein
//! `FormErrorInvalidWithoutLiveRegion` mit Severity High auf genau der
//! Implementierung, die richtig ist.
//!
//! Gemessen wird jetzt über [`crate::interaction::live_regions`]: ein
//! `MutationObserver` zeichnet auf, **dass** eine Live-Region Inhalt bekommen
//! hat, mit Zeitpunkt und Dringlichkeit. Das trägt auch den Fall, den zwei
//! Aufnahmen nie fassen — eine Meldung, die eingefügt und gleich wieder
//! entfernt wird.
//!
//! Beobachtet ist nicht angesagt: dass der Browser den Anlass hatte, heißt
//! nicht, dass ein Screenreader die Meldung vorgelesen hat.

use chromiumoxide::cdp::js_protocol::runtime::EvaluateParams;
use chromiumoxide::Page;

use super::{eval_bool, eval_string};
use crate::audit::normalized::{
    InteractiveFinding, InteractiveFindingKind, InteractiveFindingValues, JourneyStep, JourneyTrace,
};
use crate::error::Result;
use crate::interaction::{focus, live_regions, pointer, stability};
use crate::patterns::JourneyCandidate;
use crate::taxonomy::Severity;

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

    // Der Beobachter muss vor der Handlung stehen, deren Wirkung er messen
    // soll. Schlägt das fehl, ist „keine Meldung beobachtet" hinterher keine
    // Aussage über die Seite — das wird unten getrennt behandelt.
    let observing = live_regions::install(page).await;
    let _ = live_regions::drain(page).await;

    // Capture baseline — no errors yet.
    let (live_before, invalid_before, _linked_before) = check_error_state(page).await;

    trace.steps.push(JourneyStep {
        action: "check_baseline_errors".to_string(),
        target: None,
        focus: None,
        result: Some(format!(
            "live_region_present:{live_before}, aria_invalid:{invalid_before}, observing:{observing}"
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

    let _ = stability::settle_after_action(page).await;

    // Check error state after submit.
    let (live_after, invalid_after, linked_after) = check_error_state(page).await;

    // Wurde eine Live-Region gefüllt? Das ist die Ankündigung — nicht, ob
    // eine Region existiert.
    let events = live_regions::drain(page).await;
    let announced = events.iter().any(|e| e.is_announcement());
    let assertive = events
        .iter()
        .any(|e| e.is_announcement() && e.politeness == "assertive");
    // Der Container kam erst nach dem Absenden ins Dokument — eine eigene,
    // schwächere Aussage als „es wurde angekündigt".
    let late_insertion = announced && live_after && !live_before;
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
            "announced:{announced}, assertive:{assertive}, events:{}, \
             aria_invalid:{new_invalid}, linked:{linked_after}, focus_on_body:{focus_on_body}",
            events.len()
        )),
        snapshot_label: Some("after_submit_click".to_string()),
    });

    // The live-region container itself was not present on initial page load
    // and was only inserted into the DOM after submission (plan/17). This is
    // distinct from `FormErrorInvalidWithoutLiveRegion`/the silent-failure
    // check below (both of which fire when NO live region appears at all) —
    // here a live region did end up announcing the error, but screen readers
    // register live regions when the accessibility tree is first built, so a
    // container inserted only after the interaction is not reliably
    // announced by every browser/AT combination. Advisory severity: this is
    // a robustness recommendation, not a confirmed failure (the region did
    // announce correctly in this run).
    if late_insertion {
        findings.push(InteractiveFinding::new(
            "FormError",
            InteractiveFindingKind::FormErrorLiveRegionLateInsertion,
            None,
            Severity::Low,
            journey_name.clone(),
            Some("before_submit".to_string()),
            Some("after_submit_click".to_string()),
            InteractiveFindingValues::default(),
        ));
    }

    // If neither live region nor aria-invalid appeared, the form submits
    // silently — errors are not announced at all.
    if !announced && !new_invalid {
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

        // Ohne Beobachter ist „nichts angekündigt" kein Befund, sondern eine
        // Lücke in der Messung. Der Grund steht im Trace.
        if !observing {
            trace.steps.push(JourneyStep {
                action: "check_error_state".to_string(),
                target: None,
                focus: None,
                result: Some("live_observer_unavailable".to_string()),
                snapshot_label: Some("after_submit_click".to_string()),
            });
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
    if new_invalid && !announced && observing {
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
    if (announced || new_invalid) && focus_on_body {
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
