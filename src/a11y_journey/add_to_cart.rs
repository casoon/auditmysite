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

use chromiumoxide::cdp::js_protocol::runtime::EvaluateParams;
use chromiumoxide::Page;

use crate::audit::normalized::{
    InteractiveFinding, InteractiveFindingKind, InteractiveFindingValues, JourneyStep, JourneyTrace,
};
use crate::error::Result;
use crate::interaction::{pointer, stability};
use crate::patterns::JourneyCandidate;
use crate::taxonomy::Severity;

async fn eval_string(page: &Page, js: &str) -> Option<String> {
    let params = EvaluateParams::builder()
        .expression(js.to_string())
        .return_by_value(true)
        .build()
        .ok()?;
    let result = page.execute(params).await.ok()?;
    result.result.result.value?.as_str().map(|s| s.to_string())
}

async fn eval_bool(page: &Page, js: &str) -> Option<bool> {
    let params = EvaluateParams::builder()
        .expression(js.to_string())
        .return_by_value(true)
        .build()
        .ok()?;
    let result = page.execute(params).await.ok()?;
    result.result.result.value?.as_bool()
}

/// Joined, non-empty text content of all live-region/status elements — used
/// to detect whether an announcement appeared or changed after the click.
/// A boolean "exists" check alone would miss a region that is already
/// present but empty before the click and only gets text afterward.
async fn status_region_snapshot(page: &Page) -> String {
    eval_string(
        page,
        r#"Array.from(document.querySelectorAll('[role="status"],[role="alert"],[aria-live]'))
            .map(function(el) { return (el.textContent || '').trim(); })
            .filter(function(t) { return t.length > 0; })
            .join('|')"#,
    )
    .await
    .unwrap_or_default()
}

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

    let status_before = status_region_snapshot(page).await;
    let badge_before = cart_badge_snapshot(page).await;

    trace.steps.push(JourneyStep {
        action: "check_baseline".to_string(),
        target: None,
        focus: None,
        result: Some(format!(
            "status_region_present:{}, cart_badge:{:?}",
            !status_before.is_empty(),
            badge_before
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

    stability::settle(page).await?;

    let status_after = status_region_snapshot(page).await;
    let badge_after = cart_badge_snapshot(page).await;
    let announced = !status_after.is_empty() && status_after != status_before;
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
            "announced:{announced}, focus_in_dialog:{focus_moved_to_dialog}, badge_changed:{badge_changed}"
        )),
        snapshot_label: Some("after_click".to_string()),
    });

    if announced || focus_moved_to_dialog {
        // Accessible feedback confirmed — nothing to flag.
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
