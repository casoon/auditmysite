//! Accessibility-Journey-Layer — runs interactive journeys against a page
//! after the static AXTree-based audit has finished.
//!
//! `run()` is the **single** pipeline hook; phases 2–5 extend its body
//! without changing the signature.
//!
//! Phase 2: tab-walk evaluation, skip-link, disclosure, modal, tabs, menu journeys.

pub mod disclosure_journey;
pub mod evaluate;
pub mod menu_journey;
pub mod modal_journey;
pub mod skip_link;
pub mod tab_walk;
pub mod tabs_journey;

use std::time::Instant;

use chromiumoxide::Page;

use crate::audit::normalized::{AccessibilityJourney, InteractiveFinding};
use crate::cli::InteractiveMode;
use crate::error::Result;
use crate::patterns::{JourneyKind, PatternAnalysis};

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

/// Output of one journey run. The trace bundle and findings are kept
/// separate so the caller can route them to the right slots on
/// `AuditReport` / `NormalizedReport` (`accessibility_journey` vs.
/// `interactive_findings`).
#[derive(Default)]
pub struct RunOutput {
    pub journey: AccessibilityJourney,
    pub findings: Vec<InteractiveFinding>,
}

/// Default journey budget per URL (ms).
pub const DEFAULT_BUDGET_MS: u64 = 5000;

/// Single entry point invoked from `audit/pipeline.rs::audit_page`.
///
/// Returns `None` for `--interactive=off` so the rest of the pipeline
/// pays zero cost. Otherwise records journeys, evaluates them, and
/// returns both pieces.
pub async fn run(ctx: RunContext<'_>) -> Result<Option<RunOutput>> {
    if !ctx.mode.is_enabled() {
        return Ok(None);
    }

    let deadline = Instant::now() + std::time::Duration::from_millis(ctx.budget_ms);
    let mut out = RunOutput::default();

    let max_steps = match ctx.mode {
        InteractiveMode::Off => 0,
        InteractiveMode::Basic => 25,
        InteractiveMode::Full => 60,
    };

    // Tab walk + evaluation.
    let record = tab_walk::record(ctx.page, max_steps).await?;
    out.findings
        .extend(evaluate::tab_walk(&record.trace, &record.snapshots));
    out.findings
        .extend(evaluate::tab_walk_order(&record.trace, &record.dom_order));
    out.journey.traces.push(record.trace);

    // Pattern-based journeys — only if we have candidates and time remains.
    if let Some(patterns) = ctx.patterns {
        let mut skip_link_idx = 0usize;
        let mut disclosure_idx = 0usize;
        let mut modal_idx = 0usize;
        let mut tabs_idx = 0usize;
        let mut menu_idx = 0usize;

        for candidate in &patterns.journey_candidates {
            if Instant::now() >= deadline {
                tracing::info!("Journey budget exhausted, stopping pattern journeys early.");
                break;
            }
            if candidate.confidence < 0.7 {
                continue;
            }

            let result = match candidate.required_journey {
                JourneyKind::SkipLinkActivate => {
                    let idx = skip_link_idx;
                    skip_link_idx += 1;
                    skip_link::test(ctx.page, candidate, idx).await
                }
                JourneyKind::DisclosureToggle | JourneyKind::AccordionToggle => {
                    let idx = disclosure_idx;
                    disclosure_idx += 1;
                    disclosure_journey::test(ctx.page, candidate, idx).await
                }
                JourneyKind::ModalOpen => {
                    let idx = modal_idx;
                    modal_idx += 1;
                    modal_journey::test(ctx.page, candidate, idx).await
                }
                JourneyKind::TabsNavigate => {
                    let idx = tabs_idx;
                    tabs_idx += 1;
                    tabs_journey::test(ctx.page, candidate, idx).await
                }
                JourneyKind::MenuOpen => {
                    let idx = menu_idx;
                    menu_idx += 1;
                    menu_journey::test(ctx.page, candidate, idx).await
                }
                JourneyKind::FormErrorSubmit => {
                    // Phase 3.
                    continue;
                }
            };

            match result {
                Ok((trace, findings)) => {
                    out.journey.traces.push(trace);
                    out.findings.extend(findings);
                }
                Err(e) => {
                    tracing::warn!("Journey {:?} failed: {}", candidate.required_journey, e);
                }
            }
        }
    }

    Ok(Some(out))
}
