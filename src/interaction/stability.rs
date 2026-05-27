//! Wait for the page to settle after an interaction.
//!
//! Phase 2: waits for two requestAnimationFrame cycles to flush DOM mutations
//! and CSS transitions before reading focus or AXTree state. Falls back to a
//! fixed sleep when the JS evaluation fails.

use std::time::Duration;

use chromiumoxide::cdp::js_protocol::runtime::EvaluateParams;
use chromiumoxide::Page;

use crate::error::{AuditError, Result};

/// Default settle duration used as fallback when JS evaluation is unavailable.
pub const DEFAULT_SETTLE_MS: u64 = 150;

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
