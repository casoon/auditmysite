//! Accessibility-Journey-Layer — runs interactive journeys against a page
//! after the static AXTree-based audit has finished.
//!
//! `run()` is the **single** pipeline hook; phases 2–5 extend its body
//! without changing the signature.
//!
//! Phase 2: tab-walk evaluation, skip-link, disclosure, modal, tabs, menu journeys.
//! Phase 3: form-error announcement, SPA-navigation detection, link/heading/landmark inventory.

pub mod add_to_cart;
pub mod disclosure_journey;
pub mod evaluate;
pub mod form_error;
pub mod link_inventory;
pub mod menu_journey;
pub mod modal_journey;
pub mod quantity_stepper;
pub mod skip_link;
pub mod spa_navigation;
pub mod tab_walk;
pub mod tabs_journey;

use std::time::Instant;

use chromiumoxide::Page;

use crate::accessibility::AXTree;
use crate::audit::normalized::{AccessibilityJourney, InteractiveFinding};
use crate::cli::InteractiveMode;
use crate::commerce::{CommerceAnalysis, CommercePageKind};
use crate::error::Result;
use crate::patterns::{JourneyKind, PatternAnalysis};

/// Inputs the journey orchestrator needs. Kept narrow on purpose so the
/// pipeline only has to pass what is actually used.
pub struct RunContext<'a> {
    pub page: &'a Page,
    pub mode: InteractiveMode,
    /// Pattern analysis from the static phase — provides journey candidates.
    pub patterns: Option<&'a PatternAnalysis>,
    /// Initial AXTree snapshot — used by pure-analysis passes (link inventory,
    /// heading outline, landmark inventory) that don't need browser interaction.
    pub ax_tree: &'a AXTree,
    /// URL at audit start (used for SPA-navigation detection).
    pub initial_url: &'a str,
    /// Report locale (e.g. "de", "en") — drives which FTL stopword lists are
    /// loaded for link-text detection in addition to the always-merged defaults.
    pub locale: &'a str,
    /// Maximum wall-clock time the journey phase is allowed to consume.
    pub budget_ms: u64,
    /// Commerce analysis for this page, when derived (shop context detected).
    /// Used only to gate commerce-specific journeys (e.g. `AddToCart` to
    /// `CommercePageKind::ProductDetail`) — `patterns::analyze()` itself has
    /// no commerce context (it runs before the commerce module derives this).
    pub commerce: Option<&'a CommerceAnalysis>,
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

fn journey_allowed(mode: InteractiveMode, journey: JourneyKind) -> bool {
    match mode {
        InteractiveMode::Off => false,
        InteractiveMode::Basic => !matches!(
            journey,
            JourneyKind::FormErrorSubmit | JourneyKind::AddToCart | JourneyKind::QuantityStepper
        ),
        InteractiveMode::Full => true,
    }
}

/// Commerce-specific gate for journeys that must never run outside their
/// intended shop context, on top of `journey_allowed`'s mode check — e.g.
/// `AddToCart`/`QuantityStepper` only make sense (and are only safe to
/// interpret) on a detected shop's product-detail page. Both are restricted
/// to `ProductDetail` only — `CommercePageKind::Cart` was removed (see its
/// doc comment): this tool has no cross-page cart/session state, so a cart
/// page reached cold is (almost) always empty, meaning a Cart-gated
/// `QuantityStepper` run would essentially never find a real line item to
/// test in practice.
fn commerce_gate_allows(journey: JourneyKind, commerce: Option<&CommerceAnalysis>) -> bool {
    match journey {
        JourneyKind::AddToCart | JourneyKind::QuantityStepper => commerce
            .map(|c| c.page_kind == CommercePageKind::ProductDetail)
            .unwrap_or(false),
        _ => true,
    }
}

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
    out.journey.execution.mode = format!("{:?}", ctx.mode).to_lowercase();
    out.journey.execution.budget_ms = ctx.budget_ms;

    let max_steps = match ctx.mode {
        InteractiveMode::Off => 0,
        InteractiveMode::Basic => 25,
        InteractiveMode::Full => 60,
    };

    // ── Tab walk + evaluation ────────────────────────────────────────────────
    out.journey.execution.candidates_detected += 1;
    out.journey.execution.attempted += 1;
    match tab_walk::record(ctx.page, max_steps).await {
        Ok(record) => {
            out.findings
                .extend(evaluate::tab_walk(&record.trace, &record.snapshots));
            out.findings
                .extend(evaluate::tab_walk_order(&record.trace, &record.dom_order));
            out.journey.focus_evidence = record.snapshots;
            out.journey.traces.push(record.trace);
            out.journey.execution.completed += 1;
            out.journey
                .execution
                .runs
                .push(crate::audit::normalized::JourneyRun {
                    journey: "tab_walk".to_string(),
                    status: crate::audit::ExecutionStatus::Completed,
                    reason_code: None,
                });
        }
        Err(e) => {
            tracing::warn!("Tab-walk journey failed: {}", e);
            out.journey.execution.failed += 1;
            out.journey
                .execution
                .runs
                .push(crate::audit::normalized::JourneyRun {
                    journey: "tab_walk".to_string(),
                    status: crate::audit::ExecutionStatus::Failed,
                    reason_code: Some("tab_walk_failed".to_string()),
                });
        }
    }

    // ── Pattern-based journeys ───────────────────────────────────────────────
    if let Some(patterns) = ctx.patterns {
        let mut skip_link_idx = 0usize;
        let mut disclosure_idx = 0usize;
        let mut modal_idx = 0usize;
        let mut tabs_idx = 0usize;
        let mut menu_idx = 0usize;
        let mut form_idx = 0usize;
        let mut add_to_cart_idx = 0usize;
        let mut quantity_stepper_idx = 0usize;

        out.journey.execution.candidates_detected += patterns.journey_candidates.len();
        for (candidate_index, candidate) in patterns.journey_candidates.iter().enumerate() {
            if Instant::now() >= deadline {
                tracing::info!("Journey budget exhausted, stopping pattern journeys early.");
                out.journey.execution.budget_exhausted = true;
                for remaining in &patterns.journey_candidates[candidate_index..] {
                    out.journey.execution.skipped += 1;
                    out.journey
                        .execution
                        .runs
                        .push(crate::audit::normalized::JourneyRun {
                            journey: format!("{:?}", remaining.required_journey).to_lowercase(),
                            status: crate::audit::ExecutionStatus::Skipped,
                            reason_code: Some("budget_exhausted".to_string()),
                        });
                }
                break;
            }
            let skip_reason = if candidate.confidence < 0.7 {
                Some("low_confidence")
            } else if !journey_allowed(ctx.mode, candidate.required_journey) {
                Some("mode_excluded")
            } else if !commerce_gate_allows(candidate.required_journey, ctx.commerce) {
                Some("commerce_gate")
            } else {
                None
            };
            if let Some(reason) = skip_reason {
                out.journey.execution.skipped += 1;
                out.journey
                    .execution
                    .runs
                    .push(crate::audit::normalized::JourneyRun {
                        journey: format!("{:?}", candidate.required_journey).to_lowercase(),
                        status: crate::audit::ExecutionStatus::Skipped,
                        reason_code: Some(reason.to_string()),
                    });
                continue;
            }

            out.journey.execution.attempted += 1;

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
                    let idx = form_idx;
                    form_idx += 1;
                    form_error::test(ctx.page, candidate, idx).await
                }
                JourneyKind::AddToCart => {
                    let idx = add_to_cart_idx;
                    add_to_cart_idx += 1;
                    add_to_cart::test(ctx.page, candidate, idx).await
                }
                JourneyKind::QuantityStepper => {
                    let idx = quantity_stepper_idx;
                    quantity_stepper_idx += 1;
                    quantity_stepper::test(ctx.page, candidate, idx).await
                }
            };

            match result {
                Ok((trace, findings)) => {
                    out.journey.execution.completed += 1;
                    out.journey
                        .execution
                        .runs
                        .push(crate::audit::normalized::JourneyRun {
                            journey: trace.journey.clone(),
                            status: crate::audit::ExecutionStatus::Completed,
                            reason_code: None,
                        });
                    out.journey.traces.push(trace);
                    out.findings.extend(findings);
                }
                Err(e) => {
                    tracing::warn!("Journey {:?} failed: {}", candidate.required_journey, e);
                    out.journey.execution.failed += 1;
                    out.journey
                        .execution
                        .runs
                        .push(crate::audit::normalized::JourneyRun {
                            journey: format!("{:?}", candidate.required_journey).to_lowercase(),
                            status: crate::audit::ExecutionStatus::Failed,
                            reason_code: Some("journey_execution_failed".to_string()),
                        });
                }
            }
        }
    }

    // ── SPA-Navigation detection (Phase 3) ───────────────────────────────────
    // Full mode only: emits findings when actual SPA navigation is observed.
    if matches!(ctx.mode, InteractiveMode::Full) && Instant::now() < deadline {
        out.journey.execution.candidates_detected += 1;
        out.journey.execution.attempted += 1;
        match spa_navigation::run(ctx.page, ctx.initial_url).await {
            Ok(Some((trace, findings))) => {
                out.journey.execution.completed += 1;
                out.journey
                    .execution
                    .runs
                    .push(crate::audit::normalized::JourneyRun {
                        journey: trace.journey.clone(),
                        status: crate::audit::ExecutionStatus::Completed,
                        reason_code: None,
                    });
                out.journey.traces.push(trace);
                out.findings.extend(findings);
            }
            Ok(None) => {
                out.journey.execution.skipped += 1;
                out.journey
                    .execution
                    .runs
                    .push(crate::audit::normalized::JourneyRun {
                        journey: "spa_navigation".to_string(),
                        status: crate::audit::ExecutionStatus::NotApplicable,
                        reason_code: Some("no_spa_candidate".to_string()),
                    });
            }
            Err(e) => {
                tracing::warn!("SPA-navigation journey failed: {}", e);
                out.journey.execution.failed += 1;
                out.journey
                    .execution
                    .runs
                    .push(crate::audit::normalized::JourneyRun {
                        journey: "spa_navigation".to_string(),
                        status: crate::audit::ExecutionStatus::Failed,
                        reason_code: Some("spa_navigation_failed".to_string()),
                    });
            }
        }
    } else if matches!(ctx.mode, InteractiveMode::Full) {
        out.journey.execution.budget_exhausted = true;
        out.journey.execution.candidates_detected += 1;
        out.journey.execution.skipped += 1;
        out.journey
            .execution
            .runs
            .push(crate::audit::normalized::JourneyRun {
                journey: "spa_navigation".to_string(),
                status: crate::audit::ExecutionStatus::Skipped,
                reason_code: Some("budget_exhausted".to_string()),
            });
    }

    // ── Link/Heading/Landmark inventory (Phase 3, Stufe B — pure AXTree) ────
    // Full mode only. No browser interaction, so it can run even if budget is exhausted.
    if matches!(ctx.mode, InteractiveMode::Full) {
        out.findings
            .extend(link_inventory::analyse(ctx.ax_tree, ctx.locale));
    }

    Ok(Some(out))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_mode_excludes_full_only_journeys() {
        assert!(journey_allowed(
            InteractiveMode::Basic,
            JourneyKind::SkipLinkActivate
        ));
        assert!(journey_allowed(
            InteractiveMode::Basic,
            JourneyKind::ModalOpen
        ));
        assert!(!journey_allowed(
            InteractiveMode::Basic,
            JourneyKind::FormErrorSubmit
        ));
    }

    #[test]
    fn full_mode_allows_full_scope_journeys() {
        assert!(journey_allowed(
            InteractiveMode::Full,
            JourneyKind::FormErrorSubmit
        ));
    }

    #[test]
    fn compact_focus_evidence_is_part_of_public_journey_json() {
        let journey = crate::audit::normalized::AccessibilityJourney {
            execution: crate::audit::normalized::JourneyExecution {
                mode: "basic".to_string(),
                budget_ms: 1_000,
                candidates_detected: 1,
                attempted: 1,
                completed: 1,
                ..Default::default()
            },
            traces: Vec::new(),
            focus_evidence: vec![crate::accessibility::FocusSnapshot {
                selector: Some("#submit".to_string()),
                visible: true,
                in_viewport: true,
                focus_indicator: Some(crate::accessibility::FocusIndicatorStatus::Detected),
                ..Default::default()
            }],
        };

        let value = serde_json::to_value(journey).unwrap();
        assert_eq!(value["execution"]["completed"], 1);
        assert_eq!(value["focus_evidence"][0]["selector"], "#submit");
        assert_eq!(value["focus_evidence"][0]["visible"], true);
    }
}
