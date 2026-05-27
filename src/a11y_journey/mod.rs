//! Accessibility-Journey-Layer — runs interactive journeys against a page
//! after the static AXTree-based audit has finished.
//!
//! Phase 1: foundation only.
//! - `RunContext` defines the data the orchestrator needs.
//! - `run()` is the **single** pipeline hook; Phase 2-5 extend its body
//!   without changing the signature.
//! - `tab_walk` produces a reproducible focus sequence trace, but does not
//!   yet emit findings.
//!
//! Higher-level evaluation (`FocusTrap`, `SpaNavigation`, `FormError`, …)
//! lands in Phase 2-3.

pub mod tab_walk;

use chromiumoxide::Page;

use crate::audit::normalized::AccessibilityJourney;
use crate::cli::InteractiveMode;
use crate::error::Result;
use crate::patterns::PatternAnalysis;

/// Inputs the journey orchestrator needs. Kept narrow on purpose so the
/// pipeline only has to pass what is actually used.
pub struct RunContext<'a> {
    pub page: &'a Page,
    pub mode: InteractiveMode,
    /// Pattern analysis from the static phase — provides journey candidates.
    pub patterns: Option<&'a PatternAnalysis>,
    /// URL at audit start (used for SPA-navigation detection in Phase 3).
    pub initial_url: &'a str,
    /// Maximum wall-clock time the journey phase is allowed to consume.
    pub budget_ms: u64,
}

/// Default journey budget per URL (ms).
pub const DEFAULT_BUDGET_MS: u64 = 5000;

/// Single entry point invoked from `audit/pipeline.rs::audit_page`.
///
/// Returns `None` for `--interactive=off` so the rest of the pipeline pays
/// zero cost. Otherwise dispatches the configured set of journeys and
/// produces an `AccessibilityJourney` with at least one trace.
pub async fn run(ctx: RunContext<'_>) -> Result<Option<AccessibilityJourney>> {
    if !ctx.mode.is_enabled() {
        return Ok(None);
    }

    let mut journey = AccessibilityJourney::default();

    // Phase 1: tab walk only (no evaluation, just a recorded trace).
    let max_steps = match ctx.mode {
        InteractiveMode::Off => 0,
        InteractiveMode::Basic => 25,
        InteractiveMode::Full => 60,
    };
    let trace = tab_walk::record(ctx.page, max_steps).await?;
    journey.traces.push(trace);

    Ok(Some(journey))
}
