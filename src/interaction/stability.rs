//! Wait for the page to settle after an interaction.
//!
//! Phase 1: a pragmatic fixed pause that the journey orchestrator can call
//! after each action. Phase 2 replaces this with DOM + AXTree quiescence
//! detection (MutationObserver settle window).
//!
//! The current implementation is intentionally simple so the rest of the
//! journey layer can be wired up against a stable API surface.

use std::time::Duration;

use chromiumoxide::Page;

use crate::error::Result;

/// Default settle duration after an interaction.
pub const DEFAULT_SETTLE_MS: u64 = 150;

/// Wait for the page to settle. Currently a fixed sleep — kept behind a
/// function so Phase 2 can swap in MutationObserver-based quiescence
/// without touching every caller.
pub async fn wait_for_stable(_page: &Page, duration_ms: u64) -> Result<()> {
    tokio::time::sleep(Duration::from_millis(duration_ms)).await;
    Ok(())
}

/// Convenience: settle for the default duration.
pub async fn settle(page: &Page) -> Result<()> {
    wait_for_stable(page, DEFAULT_SETTLE_MS).await
}
