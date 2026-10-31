//! Collection contract for the capture layer (#777): everything the audit
//! gathers *around* the analysis modules — navigation snapshot, accessibility
//! tree, consent handling, exclusion scope, frames, page stability, the
//! browser-bound WCAG rules and the interactive journeys.
//!
//! These sites already kept a state of their own (consent status, frame
//! coverage, stability provenance, rule outcomes, journey runs). The
//! functions here project each one onto [`Collected`], so the inventory
//! ([`super::collection::COLLECTORS`]) names them, their state survives into
//! `module_runs[].subchecks[]`, and an invalid or missing capture is never
//! read as a regular empty result.

use super::collection::{
    Collected, CollectionMethod, CollectionRecord, CollectionState, CollectorSpec,
};
use super::normalized::JourneyRun;
use super::ExecutionStatus;

const fn capture(id: &'static str, method: CollectionMethod) -> CollectorSpec {
    CollectorSpec {
        id,
        module: "accessibility",
        method,
        method_version: 1,
        scoring: false,
    }
}

const fn journey(id: &'static str) -> CollectorSpec {
    CollectorSpec {
        id,
        module: "accessibility_journey",
        method: CollectionMethod::Cdp,
        method_version: 1,
        scoring: false,
    }
}

/// `location.href`, status and redirect count of the loaded document.
pub(crate) static NAVIGATION_SNAPSHOT: CollectorSpec = capture(
    "accessibility.navigation_snapshot",
    CollectionMethod::JavaScript,
);
/// The browser's accessibility tree of the page.
pub(crate) static AX_TREE: CollectorSpec = capture("accessibility.ax_tree", CollectionMethod::Cdp);
/// Consent banner detection (and dismissal, when requested).
pub(crate) static CONSENT_BANNER: CollectorSpec =
    capture("accessibility.consent_banner", CollectionMethod::JavaScript);
/// The subtrees the user excluded from the audit.
pub(crate) static EXCLUSION_SCOPE: CollectorSpec =
    capture("accessibility.exclusion_scope", CollectionMethod::Cdp);
/// Iframe discovery and the element-level audit of each frame.
pub(crate) static FRAMES: CollectorSpec = capture("accessibility.frames", CollectionMethod::Cdp);
/// DOM-settling wait before the capture.
pub(crate) static PAGE_STABILITY: CollectorSpec =
    capture("accessibility.page_stability", CollectionMethod::JavaScript);
/// Execution of the WCAG rules, the browser-bound ones (scripts evaluated on
/// the live page) among them.
pub(crate) static RULE_EXECUTION: CollectorSpec =
    capture("accessibility.rule_execution", CollectionMethod::JavaScript);

pub(crate) static JOURNEY_TAB_WALK: CollectorSpec = journey("accessibility_journey.tab_walk");
pub(crate) static JOURNEY_SKIP_LINK: CollectorSpec = journey("accessibility_journey.skip_link");
pub(crate) static JOURNEY_DISCLOSURE: CollectorSpec = journey("accessibility_journey.disclosure");
pub(crate) static JOURNEY_MODAL: CollectorSpec = journey("accessibility_journey.modal");
pub(crate) static JOURNEY_TABS: CollectorSpec = journey("accessibility_journey.tabs");
pub(crate) static JOURNEY_MENU: CollectorSpec = journey("accessibility_journey.menu");
pub(crate) static JOURNEY_FORM_ERROR: CollectorSpec = journey("accessibility_journey.form_error");
pub(crate) static JOURNEY_ADD_TO_CART: CollectorSpec = journey("accessibility_journey.add_to_cart");
pub(crate) static JOURNEY_QUANTITY_STEPPER: CollectorSpec =
    journey("accessibility_journey.quantity_stepper");
pub(crate) static JOURNEY_SPA_NAVIGATION: CollectorSpec =
    journey("accessibility_journey.spa_navigation");

/// Reason codes of the capture layer.
pub(crate) mod reason {
    pub(crate) const CONSENT_DETECTION_FAILED: &str = "consent_detection_failed";
    pub(crate) const BUDGET_EXHAUSTED: &str = "budget_exhausted";
    pub(crate) const NO_AX_NODES: &str = "ax_tree_without_nodes";
    pub(crate) const JOURNEY_NOT_RUN: &str = "journey_not_run";
}

/// The navigation script's answer. `status` and `redirects` are `null` when
/// the browser has no navigation timing entry for the document.
#[derive(Debug, serde::Deserialize)]
pub(crate) struct NavigationRaw {
    #[serde(rename = "finalUrl")]
    pub final_url: String,
    pub status: Option<u64>,
    pub redirects: Option<u64>,
    #[serde(rename = "readyState")]
    pub ready_state: String,
}

/// A navigation answer with its gaps: an absent status or redirect count is
/// a gap, never `0` redirects.
pub(crate) fn navigation_collection(raw: Collected<NavigationRaw>) -> Collected<NavigationRaw> {
    match raw {
        Collected::Measured(v) => {
            let mut gaps = Vec::new();
            if v.status.is_none() {
                gaps.push("main_document_status");
            }
            if v.redirects.is_none() {
                gaps.push("redirect_count");
            }
            if gaps.is_empty() {
                Collected::Measured(v)
            } else {
                Collected::Partial { value: v, gaps }
            }
        }
        other => other,
    }
}

/// A tree without a single node is a failed capture, not an empty page.
pub(crate) fn ax_tree_collection(node_count: usize) -> Collected<()> {
    if node_count == 0 {
        Collected::Failed(reason::NO_AX_NODES)
    } else {
        Collected::Measured(())
    }
}

/// Consent handling as a collection: an evaluation that could not tell
/// whether a banner is there is a failure. A banner that was seen but not
/// dismissed is measured — the audited content state records the outcome of
/// the dismissal.
pub(crate) fn consent_collection(
    status: crate::browser::consent::ConsentHandlingStatus,
) -> Collected<()> {
    use crate::browser::consent::ConsentHandlingStatus as Status;
    match status {
        Status::NotDetected => Collected::Empty(()),
        Status::Detected | Status::Dismissed | Status::DismissFailed => Collected::Measured(()),
        Status::Unknown => Collected::Failed(reason::CONSENT_DETECTION_FAILED),
    }
}

/// Frame coverage as a collection. Hidden and excluded frames are decisions;
/// frames the audit could not reach or read are gaps.
pub(crate) fn frames_collection(coverage: &super::frames::FrameCoverage) -> Collected<()> {
    use super::frames::FrameSkipReason as Skip;
    if coverage.audited.is_empty() && coverage.skipped.is_empty() {
        return Collected::Empty(());
    }
    let mut gaps = Vec::new();
    for entry in &coverage.skipped {
        let gap = match entry.reason {
            Some(Skip::CrossOrigin) => "cross_origin_frame",
            Some(Skip::AxTreeFailed) => "frame_ax_tree_failed",
            Some(Skip::DocumentUnavailable) => "frame_document_unavailable",
            Some(Skip::Hidden | Skip::Excluded) => continue,
            None => "frame_skipped_without_reason",
        };
        if !gaps.contains(&gap) {
            gaps.push(gap);
        }
    }
    if gaps.is_empty() {
        Collected::Measured(())
    } else {
        Collected::Partial { value: (), gaps }
    }
}

/// Page stability as a collection. A page that never settled was captured
/// mid-change: the measurement exists but its basis is incomplete.
pub(crate) fn stability_collection(
    passes: &[crate::interaction::stability::StabilityProvenance],
) -> Collected<()> {
    use crate::interaction::stability::StabilityStatus as Status;
    if passes.is_empty() {
        return Collected::Failed(super::collection::reason::NO_RESULT);
    }
    if passes
        .iter()
        .any(|p| matches!(p.status, Status::BudgetExhausted | Status::Fallback))
    {
        Collected::Partial {
            value: (),
            gaps: vec![reason::BUDGET_EXHAUSTED],
        }
    } else {
        Collected::Measured(())
    }
}

/// The WCAG rule executions as one collection: measured when every rule
/// ran, partial when some were lost to a technical failure, failed when all
/// were. The gaps name the rules that did not run. `None` when no rule was
/// requested.
pub(crate) fn rule_execution_record(outcomes: &[crate::wcag::RuleRun]) -> Option<CollectionRecord> {
    if outcomes.is_empty() {
        return None;
    }
    let mut lost: Vec<String> = outcomes
        .iter()
        .filter(|o| crate::wcag::rule_run_errored(o))
        .map(|o| o.rule_id.clone())
        .collect();
    lost.sort();
    lost.dedup();
    let all_lost = outcomes.iter().all(crate::wcag::rule_run_errored);
    let state = if lost.is_empty() {
        CollectionState::Measured
    } else if all_lost {
        CollectionState::Failed
    } else {
        CollectionState::Partial
    };
    Some(CollectionRecord {
        collector: RULE_EXECUTION.id.to_string(),
        module: RULE_EXECUTION.module.to_string(),
        state,
        method: RULE_EXECUTION.method,
        method_version: RULE_EXECUTION.method_version,
        scoring: RULE_EXECUTION.scoring,
        reason_code: (state == CollectionState::Failed).then(|| "all_rules_failed".to_string()),
        gaps: lost,
    })
}

/// The collector a journey run belongs to. Run names differ between the
/// skipped/failed path (`SkipLinkActivate`, lowercased) and the completed
/// one (`skip_link_0`), so both are normalized first.
pub(crate) fn journey_spec(run_name: &str) -> Option<&'static CollectorSpec> {
    let name: String = run_name
        .chars()
        .filter(|c| c.is_ascii_alphabetic())
        .collect::<String>()
        .to_lowercase();
    [
        ("tabwalk", &JOURNEY_TAB_WALK),
        ("skiplink", &JOURNEY_SKIP_LINK),
        ("disclosure", &JOURNEY_DISCLOSURE),
        ("accordion", &JOURNEY_DISCLOSURE),
        ("modal", &JOURNEY_MODAL),
        ("tabs", &JOURNEY_TABS),
        ("menu", &JOURNEY_MENU),
        ("formerror", &JOURNEY_FORM_ERROR),
        ("addtocart", &JOURNEY_ADD_TO_CART),
        ("quantity", &JOURNEY_QUANTITY_STEPPER),
        ("spanavigation", &JOURNEY_SPA_NAVIGATION),
    ]
    .into_iter()
    .find(|(prefix, _)| name.starts_with(prefix))
    .map(|(_, spec)| spec)
}

/// One journey run as a collection state. A skipped journey did not look at
/// the page — it is `Unsupported` (budget exhausted) or `NotApplicable` (the
/// page or mode gave no reason to run it), never a clean result.
pub(crate) fn journey_state(run: &JourneyRun) -> Collected<()> {
    let code = |fallback: &'static str| match run.reason_code.as_deref() {
        Some("budget_exhausted") => reason::BUDGET_EXHAUSTED,
        Some("journey_timeout") => "journey_timeout",
        Some("journey_execution_failed") => "journey_execution_failed",
        _ => fallback,
    };
    match run.status {
        ExecutionStatus::Completed => Collected::Measured(()),
        ExecutionStatus::Partial => Collected::Partial {
            value: (),
            gaps: vec![code("journey_coverage_partial")],
        },
        ExecutionStatus::Failed => Collected::Failed(code("journey_failed")),
        ExecutionStatus::Skipped if run.reason_code.as_deref() == Some("budget_exhausted") => {
            Collected::Unsupported(reason::BUDGET_EXHAUSTED)
        }
        ExecutionStatus::Skipped
        | ExecutionStatus::NotApplicable
        | ExecutionStatus::NotRequested => Collected::NotApplicable(reason::JOURNEY_NOT_RUN),
    }
}

/// Record every journey run of the accessibility journey layer.
pub(crate) fn record_journeys(log: &super::collection::CollectionLog, runs: &[JourneyRun]) {
    for run in runs {
        if let Some(spec) = journey_spec(&run.journey) {
            log.record(spec, journey_state(run));
        }
    }
}

#[cfg(test)]
pub(crate) fn counter_cases() {
    use super::collection::reason as common;
    use serde_json::json;

    // Navigation: no answer, wrong schema and a missing timing entry are
    // not "no redirects".
    for (value, expected) in [
        (None, common::NO_RESULT),
        (Some(json!(null)), common::NO_RESULT),
        (Some(json!("text")), common::SCHEMA_MISMATCH),
        (Some(json!({"finalUrl": 3})), common::SCHEMA_MISMATCH),
    ] {
        let state = navigation_collection(common_decode(value.as_ref()));
        assert_eq!(state.state(), CollectionState::Failed, "{value:?}");
        assert_eq!(
            match state {
                Collected::Failed(r) => r,
                _ => "",
            },
            expected
        );
    }
    let no_entry = common_decode(Some(&json!({
        "finalUrl": "https://a/", "status": null, "redirects": null, "readyState": "complete"
    })));
    match navigation_collection(no_entry) {
        Collected::Partial { gaps, .. } => {
            assert_eq!(gaps, vec!["main_document_status", "redirect_count"])
        }
        other => panic!("a missing timing entry is partial: {other:?}"),
    }
    let measured = common_decode(Some(&json!({
        "finalUrl": "https://a/", "status": 200, "redirects": 0, "readyState": "complete"
    })));
    assert_eq!(
        navigation_collection(measured).state(),
        CollectionState::Measured
    );

    // AX tree.
    assert_eq!(ax_tree_collection(0).state(), CollectionState::Failed);
    assert_eq!(ax_tree_collection(3).state(), CollectionState::Measured);

    // Consent.
    use crate::browser::consent::ConsentHandlingStatus as Consent;
    assert_eq!(
        consent_collection(Consent::Unknown).state(),
        CollectionState::Failed
    );
    assert_eq!(
        consent_collection(Consent::NotDetected).state(),
        CollectionState::Empty
    );
    assert_eq!(
        consent_collection(Consent::DismissFailed).state(),
        CollectionState::Measured
    );
    assert_eq!(
        consent_collection(Consent::Dismissed).state(),
        CollectionState::Measured
    );

    // Frames: no frames is empty, an unreachable one a gap, a hidden one
    // a decision.
    use super::frames::{FrameCoverage, FrameEntry, FrameSkipReason};
    let entry = |reason| FrameEntry {
        selector: "iframe".into(),
        reason,
        ..Default::default()
    };
    assert_eq!(
        frames_collection(&FrameCoverage::default()).state(),
        CollectionState::Empty
    );
    let coverage = |skipped| FrameCoverage {
        audited: vec![entry(None)],
        skipped,
    };
    assert_eq!(
        frames_collection(&coverage(vec![entry(Some(FrameSkipReason::Hidden))])).state(),
        CollectionState::Measured
    );
    assert_eq!(
        frames_collection(&coverage(vec![entry(Some(FrameSkipReason::AxTreeFailed))])).state(),
        CollectionState::Partial
    );
    assert_eq!(
        frames_collection(&coverage(vec![entry(None)])).state(),
        CollectionState::Partial
    );

    // Stability.
    use crate::interaction::stability::{StabilityProvenance, StabilityStatus};
    let pass = |status| StabilityProvenance {
        status,
        ..Default::default()
    };
    assert_eq!(stability_collection(&[]).state(), CollectionState::Failed);
    assert_eq!(
        stability_collection(&[pass(StabilityStatus::Stable)]).state(),
        CollectionState::Measured
    );
    assert_eq!(
        stability_collection(&[
            pass(StabilityStatus::Stable),
            pass(StabilityStatus::Fallback)
        ])
        .state(),
        CollectionState::Partial
    );

    // Journeys, under both naming schemes.
    for name in [
        "tab_walk",
        "skiplinkactivate",
        "skip_link_0",
        "disclosure_0",
        "accordiontoggle",
        "modalopen",
        "tabsnavigate",
        "menu_1",
        "formerrorsubmit",
        "add_to_cart_0",
        "quantitystepper",
        "spa_navigation",
    ] {
        assert!(journey_spec(name).is_some(), "{name} has no collector");
    }
    assert!(journey_spec("unknown_journey").is_none());
    let run = |status, reason: Option<&str>| JourneyRun {
        journey: "modal_0".into(),
        status,
        reason_code: reason.map(str::to_string),
    };
    for (status, reason, expected) in [
        (ExecutionStatus::Completed, None, CollectionState::Measured),
        (
            ExecutionStatus::Failed,
            Some("journey_timeout"),
            CollectionState::Failed,
        ),
        (ExecutionStatus::Partial, None, CollectionState::Partial),
        (
            ExecutionStatus::Skipped,
            Some("budget_exhausted"),
            CollectionState::Unsupported,
        ),
        (
            ExecutionStatus::Skipped,
            Some("low_confidence"),
            CollectionState::NotApplicable,
        ),
        (
            ExecutionStatus::NotApplicable,
            Some("no_spa_candidate"),
            CollectionState::NotApplicable,
        ),
    ] {
        assert_eq!(journey_state(&run(status, reason)).state(), expected);
    }
}

/// Test support: decode a navigation script result the way the pipeline does.
#[cfg(test)]
fn common_decode(value: Option<&serde_json::Value>) -> Collected<NavigationRaw> {
    super::collection::decode_js_value(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capture_counter_cases() {
        counter_cases();
    }

    #[test]
    fn rule_execution_state_follows_the_rule_outcomes() {
        use crate::wcag::{NotRun, RuleRun};
        let ran = |rule: &str| RuleRun::ran(rule, 0).in_viewport("mobile");
        let lost = |rule: &str| {
            RuleRun::not_run(rule, NotRun::Errored)
                .with_reason("dom_evaluation_failed")
                .in_viewport("mobile")
        };
        assert!(rule_execution_record(&[]).is_none());
        let state =
            |outcomes: &[RuleRun]| rule_execution_record(outcomes).map(|r| (r.state, r.gaps));
        assert_eq!(
            state(&[ran("reflow"), ran("text-spacing")]),
            Some((CollectionState::Measured, vec![]))
        );
        assert_eq!(
            state(&[lost("reflow"), ran("text-spacing")]),
            Some((CollectionState::Partial, vec!["reflow".to_string()]))
        );
        assert_eq!(
            state(&[lost("reflow"), lost("text-spacing")]).map(|(state, _)| state),
            Some(CollectionState::Failed)
        );
    }
}
