//! Add-to-cart feedback journey (SC 4.1.3 Status Messages).
//!
//! Clicks a detected "Add to Cart" trigger and checks whether the resulting
//! state change is exposed to assistive technology: a live-region/status
//! announcement, or focus moving into a cart drawer/dialog. A cart-badge
//! text change with neither is the classic "visual-only" failure — the
//! cart count updates on screen but a screen reader user never learns the
//! item was added.
//!
//! Safety: single synthetic click, no further navigation, no cleanup — the
//! ephemeral browser context for this page is the cleanup. Gated (at
//! `a11y_journey::run`) to a detected shop's product-detail page under
//! `--interactive full` only.

use chromiumoxide::Page;

use super::{eval_bool, eval_string};
use crate::audit::normalized::{
    InteractiveFinding, InteractiveFindingKind, InteractiveFindingValues, JourneyStep, JourneyTrace,
};
use crate::error::Result;
use crate::interaction::{live_regions, pointer, stability};
use crate::patterns::JourneyCandidate;
use crate::taxonomy::Severity;

/// Best-effort cart-badge/counter text. Deliberately heuristic (site-
/// specific markup, no guarantee of a match) — used only as evidence that
/// *something* visibly changed even when no accessible announcement did.
async fn cart_badge_snapshot(page: &Page) -> Option<String> {
    eval_string(
        page,
        r#"(function() {
            var el = document.querySelector(
                '[class*="cart-count"],[class*="cart-badge"],[data-cart-count],[class*="basket-count"]'
            );
            return el ? (el.textContent || '').trim() : null;
        })()"#,
    )
    .await
}

/// Whether the currently focused element is inside a dialog/drawer
/// (`role="dialog"` or `aria-modal="true"`) — the other accessible way an
/// add-to-cart click can surface feedback, besides a live-region
/// announcement (a cart drawer opens and takes focus).
///
/// `closest()` bleibt an der Shadow-Grenze stehen, und `document.activeElement`
/// hält dort den Host. Für diesen Nebenpfad hingenommen; die Ankündigung
/// selbst wird über [`live_regions`] beobachtet, nicht hier.
async fn focus_in_dialog(page: &Page) -> bool {
    eval_bool(
        page,
        r#"document.activeElement !== null &&
            document.activeElement.closest('[role="dialog"],[aria-modal="true"]') !== null"#,
    )
    .await
    .unwrap_or(false)
}

pub async fn test(
    page: &Page,
    candidate: &JourneyCandidate,
    index: usize,
) -> Result<(JourneyTrace, Vec<InteractiveFinding>)> {
    let journey_name = format!("add_to_cart_{index}");
    let mut trace = JourneyTrace {
        journey: journey_name.clone(),
        steps: Vec::new(),
    };
    let mut findings: Vec<InteractiveFinding> = Vec::new();

    // Die Ankündigung wird beobachtet, nicht abgefragt: eine Statusmeldung,
    // die erscheint und nach zwei Sekunden wieder verschwindet, ist zwischen
    // zwei Stichproben unter Umständen nie zu sehen — bei Warenkorb-Feedback
    // der Normalfall. Der Beobachter durchdringt zudem Shadow Roots, die
    // `querySelectorAll` nicht erreicht.
    let observing = live_regions::install(page).await;
    let _ = live_regions::drain(page).await;
    let badge_before = cart_badge_snapshot(page).await;

    trace.steps.push(JourneyStep {
        action: "check_baseline".to_string(),
        target: None,
        focus: None,
        result: Some(format!(
            "live_observer:{observing}, cart_badge:{badge_before:?}"
        )),
        snapshot_label: Some("before_click".to_string()),
    });

    let Some(trigger_id) = candidate.trigger_backend_id else {
        return Ok((trace, findings));
    };

    if let Err(e) = pointer::synthetic_click_backend(page, trigger_id).await {
        tracing::warn!("add_to_cart: click on backend node {trigger_id} failed: {e}");
        return Ok((trace, findings));
    }
    trace.steps.push(JourneyStep {
        action: "synthetic_click".to_string(),
        target: Some(format!("backend_node:{trigger_id}")),
        focus: None,
        result: None,
        snapshot_label: Some("after_click".to_string()),
    });

    let _ = stability::settle_after_action(page).await;

    let events = live_regions::drain(page).await;
    let announced = events.iter().any(|e| e.is_announcement());
    let badge_after = cart_badge_snapshot(page).await;
    let focus_moved_to_dialog = focus_in_dialog(page).await;
    let badge_changed = matches!(
        (&badge_before, &badge_after),
        (Some(before), Some(after)) if before != after
    );

    trace.steps.push(JourneyStep {
        action: "check_feedback".to_string(),
        target: None,
        focus: None,
        result: Some(format!(
            "announced:{announced}, live_events:{}, focus_in_dialog:{focus_moved_to_dialog}, \
             badge_changed:{badge_changed}",
            events.len()
        )),
        snapshot_label: Some("after_click".to_string()),
    });

    if announced || focus_moved_to_dialog {
        // Accessible feedback confirmed — nothing to flag.
        return Ok((trace, findings));
    }

    // Ohne Beobachter ist „nichts angekündigt" keine Aussage über die Seite.
    if !observing {
        trace.steps.push(JourneyStep {
            action: "check_feedback".to_string(),
            target: None,
            focus: None,
            result: Some("live_observer_unavailable".to_string()),
            snapshot_label: Some("after_click".to_string()),
        });
        return Ok((trace, findings));
    }

    if badge_changed {
        // Proof the click "worked" (something visibly changed), but no
        // accessible route noticed it — the classic screen-reader-invisible
        // cart-badge-only update.
        findings.push(InteractiveFinding::new(
            "AddToCart",
            InteractiveFindingKind::AddToCartNoStatusAnnouncement,
            None,
            Severity::High,
            journey_name.clone(),
            Some("before_click".to_string()),
            Some("after_click".to_string()),
            InteractiveFindingValues::default(),
        ));
    } else {
        // Nothing observable changed at all — could be a feedback mechanism
        // this heuristic doesn't recognize, or the click had no effect.
        // Manual-review tone, not a hard claim of failure.
        findings.push(InteractiveFinding::new(
            "AddToCart",
            InteractiveFindingKind::AddToCartNoFeedbackDetected,
            None,
            Severity::Medium,
            journey_name.clone(),
            Some("before_click".to_string()),
            Some("after_click".to_string()),
            InteractiveFindingValues::default(),
        ));
    }

    Ok((trace, findings))
}
