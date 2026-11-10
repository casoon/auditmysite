//! Wait for the page to settle after an interaction.
//!
//! Phase 2: waits for two requestAnimationFrame cycles to flush DOM mutations
//! and CSS transitions before reading focus or AXTree state. Falls back to a
//! fixed sleep when the JS evaluation fails.

use std::time::Duration;

use chromiumoxide::cdp::js_protocol::runtime::EvaluateParams;
use chromiumoxide::Page;
use serde::{Deserialize, Serialize};

use crate::error::{AuditError, Result};

/// Default settle duration used as fallback when JS evaluation is unavailable.
pub const DEFAULT_SETTLE_MS: u64 = 150;
pub const DEFAULT_STABILITY_BUDGET_MS: u64 = 1_500;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StabilityProvenance {
    pub viewport: String,
    pub status: StabilityStatus,
    pub waited_ms: u64,
    pub mutation_count: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StabilityStatus {
    Stable,
    ReadySignal,
    #[default]
    BudgetExhausted,
    /// The budget expired, but the ongoing DOM mutations were confined to a
    /// small, stable set of elements with no real content growth (and/or a
    /// native animation was observed running) — consistent with an
    /// intentional, continuously running page animation rather than an
    /// unsettled/still-loading page. Does not count as a quality issue.
    OngoingAnimation,
    Fallback,
}

/// `js/page_stability.js` with its `__BUDGET_MS__` placeholder filled in.
fn page_stability_script(budget_ms: u64) -> String {
    include_str!("js/page_stability.js").replace("__BUDGET_MS__", &budget_ms.to_string())
}

/// `js/finite_animations.js` with its `__BUDGET_MS__` placeholder filled in.
fn finite_animations_script(budget_ms: u64) -> String {
    include_str!("js/finite_animations.js").replace("__BUDGET_MS__", &budget_ms.to_string())
}

/// Wait until the DOM has been quiet for 200 ms with content rendered, an
/// application-provided `window.__AUDITMYSITE_READY__ === true` signal is
/// present, or the bounded budget is exhausted. This deliberately does not
/// wait for network idle. A quiet page without any content yet (an empty
/// splash screen while a client-rendered app loads, #718) is not settled.
///
/// When the budget is exhausted, a heuristic distinguishes a genuinely
/// unsettled page from one with a legitimate, continuously running
/// animation (e.g. a marquee, ticker, or carousel): if mutations stayed
/// confined to a small set of elements without real content growth — or a
/// native CSS/Web Animation is still running — the page is reported as
/// `OngoingAnimation` instead of `BudgetExhausted`, so it is not treated as
/// a data-quality problem.
pub async fn wait_for_page_stability(
    page: &Page,
    viewport: &str,
    budget_ms: u64,
) -> StabilityProvenance {
    let budget_ms = budget_ms.clamp(200, 10_000);
    let expression = page_stability_script(budget_ms);
    // `return_by_value`: the promise resolves to an object. Without it CDP
    // hands back a remote-object reference, `value` stays empty, and every
    // call fell through to the defaults below — `budget_exhausted` after the
    // full budget with 0 mutations, whatever the page did. That marked
    // practically every audit `partial` (`page_stability_budget_exhausted`)
    // and hid how long a journey click really waited (plan 53).
    let params = match EvaluateParams::builder()
        .expression(expression)
        .await_promise(true)
        .return_by_value(true)
        .build()
    {
        Ok(params) => params,
        Err(error) => {
            tokio::time::sleep(Duration::from_millis(DEFAULT_SETTLE_MS)).await;
            return StabilityProvenance {
                viewport: viewport.to_string(),
                status: StabilityStatus::Fallback,
                waited_ms: DEFAULT_SETTLE_MS,
                mutation_count: 0,
                reason: Some(format!("Stability script could not be built: {error}")),
            };
        }
    };
    match tokio::time::timeout(Duration::from_millis(budget_ms + 500), page.execute(params)).await {
        Ok(Ok(result)) => {
            let value = result.result.result.value.clone().unwrap_or_default();
            let status = match value.get("status").and_then(serde_json::Value::as_str) {
                Some("stable") => StabilityStatus::Stable,
                Some("ready_signal") => StabilityStatus::ReadySignal,
                Some("ongoing_animation") => StabilityStatus::OngoingAnimation,
                _ => StabilityStatus::BudgetExhausted,
            };
            StabilityProvenance {
                viewport: viewport.to_string(),
                status,
                waited_ms: value
                    .get("waited_ms")
                    .and_then(serde_json::Value::as_u64)
                    .unwrap_or(budget_ms),
                mutation_count: value
                    .get("mutation_count")
                    .and_then(serde_json::Value::as_u64)
                    .unwrap_or(0),
                reason: value
                    .get("reason")
                    .and_then(serde_json::Value::as_str)
                    .map(str::to_string),
            }
        }
        _ => StabilityProvenance {
            viewport: viewport.to_string(),
            status: StabilityStatus::Fallback,
            waited_ms: budget_ms,
            mutation_count: 0,
            reason: Some("Stability evaluation failed or timed out".to_string()),
        },
    }
}

/// Result of [`wait_for_finite_animations`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnimationSettle {
    /// No finite animation or transition was running when the wait ended.
    pub settled: bool,
    pub waited_ms: u64,
    /// Finite animations still running when the wait ended.
    pub running: u64,
}

impl AnimationSettle {
    /// Anything unreadable counts as not settled; the target-size rules then
    /// still check each target for a running animation themselves.
    fn from_value(value: &serde_json::Value) -> Self {
        let running = value
            .get("running")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);
        Self {
            settled: value.get("settled").and_then(serde_json::Value::as_bool) == Some(true)
                && running == 0,
            waited_ms: value
                .get("waited_ms")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0),
            running,
        }
    }
}

/// Wait until running finite CSS animations and transitions have finished,
/// so size-based rules measure the settled layout (#706). A logo intro that
/// grows from `max-width: 0` was measured at 3×39 px instead of its final
/// 147×39 px — the audit browser starts a fresh session each time, so a
/// once-per-session intro always runs.
///
/// Infinite animations (spinners, marquees) are ignored: they never finish.
/// An animation whose remaining time exceeds the remaining budget is not
/// waited for — that would only burn the budget. After each batch finishes,
/// two animation frames pass before looking again, so animations chained on
/// `animationend` are caught too. Without `document.getAnimations` this
/// returns at once.
///
/// Whatever still runs afterwards is left to the rules: the target-size
/// helpers (`isAnimating`) report such a target as not measured instead of
/// reporting its mid-animation size.
pub async fn wait_for_finite_animations(page: &Page, budget_ms: u64) -> AnimationSettle {
    let budget_ms = budget_ms.clamp(200, 10_000);
    let expression = finite_animations_script(budget_ms);
    let params = match EvaluateParams::builder()
        .expression(expression)
        .await_promise(true)
        .return_by_value(true)
        .build()
    {
        Ok(params) => params,
        Err(_) => return AnimationSettle::from_value(&serde_json::Value::Null),
    };
    match tokio::time::timeout(Duration::from_millis(budget_ms + 500), page.execute(params)).await {
        Ok(Ok(result)) => {
            AnimationSettle::from_value(&result.result.result.value.clone().unwrap_or_default())
        }
        _ => AnimationSettle::from_value(&serde_json::Value::Null),
    }
}

/// Wait for the page to settle after an interaction.
///
/// Runs a JS promise that resolves after two animation frames, which flushes
/// pending DOM mutations and CSS transitions. Falls back to a fixed sleep if
/// JS evaluation fails (e.g. page is navigating or JS context was destroyed).
pub async fn wait_for_stable(page: &Page, duration_ms: u64) -> Result<()> {
    let js = "new Promise(function(r) { requestAnimationFrame(function() { requestAnimationFrame(r); }); })";
    let params = EvaluateParams::builder()
        .expression(js.to_string())
        .await_promise(true)
        .build()
        .map_err(|e| AuditError::InteractionFailed {
            reason: format!("settle build failed: {e}"),
        })?;
    // Bounded, like `wait_for_page_stability`'s own CDP call below — without
    // this, a concurrent batch run can leave this awaitPromise command
    // pending on chromiumoxide's side well past its intended budget (up to
    // its internal command timeout), which silently eats into the calling
    // page's overall per-audit timeout budget under concurrency (#url-file
    // batch hang investigation).
    let outcome = tokio::time::timeout(
        Duration::from_millis(duration_ms + 500),
        page.execute(params),
    )
    .await;
    if !matches!(outcome, Ok(Ok(_))) {
        tokio::time::sleep(Duration::from_millis(duration_ms)).await;
    }
    Ok(())
}

/// Convenience: settle for the default duration.
pub async fn settle(page: &Page) -> Result<()> {
    wait_for_stable(page, DEFAULT_SETTLE_MS).await
}

/// Zeitbudget, in dem eine Seite auf eine Handlung reagieren darf.
///
/// Gemessen an `www.uni-jena.de`: nach [`settle`] — zwei Animationsframes,
/// rund 30 ms — war der Accessibility-Tree **unverändert**. 400 ms später
/// standen 113 wahrnehmbar gewordene Knoten darin, darunter der Dialog, den
/// der Klick geöffnet hatte. Die Journey hatte bis dahin „kein Dialog
/// erschienen" gemeldet.
pub const JOURNEY_SETTLE_BUDGET_MS: u64 = 600;

/// Warten, bis die Seite auf eine Handlung reagiert hat.
///
/// [`settle`] wartet zwei Animationsframes. Das reicht, um anstehende
/// Layout-Arbeit zu leeren, aber nicht, um eine Reaktion abzuwarten: zwischen
/// Klick und sichtbarer Wirkung liegen Ereignis-Handler, oft ein
/// Framework-Tick und meist eine Übergangsanimation.
///
/// Hier wartet stattdessen [`wait_for_page_stability`] auf 200 ms Ruhe im DOM,
/// höchstens aber [`JOURNEY_SETTLE_BUDGET_MS`]. Auf einer Seite, die schnell
/// reagiert, kostet das rund 250 ms; nur eine unruhige Seite schöpft das
/// Budget aus.
///
/// Der Preis ist Abdeckung: mehr Zeit je Schritt heißt weniger Kandidaten
/// innerhalb des Journey-Budgets. Eine zu früh genommene Aufnahme misst
/// allerdings die falsche Seite.
pub async fn settle_after_action(page: &Page) -> StabilityProvenance {
    wait_for_page_stability(page, "journey", JOURNEY_SETTLE_BUDGET_MS).await
}

#[cfg(test)]
mod tests {
    use super::{finite_animations_script, page_stability_script, AnimationSettle};
    use serde_json::json;

    #[test]
    fn page_stability_script_fills_the_budget_placeholder() {
        let js = page_stability_script(1234);
        assert!(js.contains("}, 1234);"), "{js}");
        assert!(!js.contains("__BUDGET_MS__"));
    }

    #[test]
    fn finite_animations_script_fills_the_budget_placeholder() {
        let js = finite_animations_script(4321);
        assert!(js.contains("const left = 4321 - elapsed;"), "{js}");
        assert!(!js.contains("__BUDGET_MS__"));
    }

    #[test]
    fn animation_settle_reads_settled_result() {
        let settle = AnimationSettle::from_value(
            &json!({ "settled": true, "waited_ms": 812, "running": 0 }),
        );
        assert_eq!(
            settle,
            AnimationSettle {
                settled: true,
                waited_ms: 812,
                running: 0
            }
        );
    }

    #[test]
    fn animation_settle_with_running_animations_is_not_settled() {
        let settle = AnimationSettle::from_value(
            &json!({ "settled": false, "waited_ms": 1500, "running": 2 }),
        );
        assert!(!settle.settled);
        assert_eq!(settle.running, 2);
    }

    #[test]
    fn animation_settle_without_result_is_not_settled() {
        assert!(!AnimationSettle::from_value(&serde_json::Value::Null).settled);
    }
}
