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
    Fallback,
}

/// Wait until the DOM has been quiet for 200 ms, an application-provided
/// `window.__AUDITMYSITE_READY__ === true` signal is present, or the bounded
/// budget is exhausted. This deliberately does not wait for network idle.
pub async fn wait_for_page_stability(
    page: &Page,
    viewport: &str,
    budget_ms: u64,
) -> StabilityProvenance {
    let budget_ms = budget_ms.clamp(200, 10_000);
    let expression = format!(
        r#"new Promise(resolve => {{
            const started = performance.now();
            let mutations = 0;
            let quietTimer;
            let done = false;
            const finish = (status, reason) => {{
                if (done) return;
                done = true;
                observer.disconnect();
                clearTimeout(quietTimer);
                clearTimeout(budgetTimer);
                resolve({{ status, waited_ms: Math.round(performance.now() - started), mutation_count: mutations, reason }});
            }};
            const observer = new MutationObserver(records => {{
                mutations += records.length;
                clearTimeout(quietTimer);
                quietTimer = setTimeout(() => finish('stable', null), 200);
            }});
            observer.observe(document.documentElement, {{subtree:true, childList:true, attributes:true, characterData:true}});
            const budgetTimer = setTimeout(() => finish('budget_exhausted', 'DOM did not remain quiet within the configured budget'), {budget_ms});
            if (window.__AUDITMYSITE_READY__ === true || document.documentElement.dataset.auditReady === 'true') {{
                finish('ready_signal', null);
            }} else {{
                quietTimer = setTimeout(() => finish('stable', null), 200);
            }}
        }})"#
    );
    let params = match EvaluateParams::builder()
        .expression(expression)
        .await_promise(true)
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
    if page.execute(params).await.is_err() {
        tokio::time::sleep(Duration::from_millis(duration_ms)).await;
    }
    Ok(())
}

/// Convenience: settle for the default duration.
pub async fn settle(page: &Page) -> Result<()> {
    wait_for_stable(page, DEFAULT_SETTLE_MS).await
}
