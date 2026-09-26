//! Skip-link journey: activate a skip link and verify focus reaches its target.
//!
//! The link is focused first, as a keyboard user would have it, then
//! activated. The target is resolved from the link's `href` fragment:
//!
//! - Focus inside (or past) the target: passes.
//! - Focus still on the link or on `body`: a non-focusable target (`<main>`
//!   without `tabindex`) leaves focus there, yet browsers move the sequential
//!   focus navigation starting point, so the next Tab lands in the content.
//!   The journey presses Tab once and checks where it lands.
//! - Focus moved to some other element before the target: fails.
//! - A fragment that matches no element: fails — there is nothing to skip to.
//! - No fragment (script-driven link): the only signal left is whether focus
//!   left the link and `body` at all.

use chromiumoxide::cdp::browser_protocol::dom::{BackendNodeId, ResolveNodeParams};
use chromiumoxide::cdp::js_protocol::runtime::{
    CallArgument, CallFunctionOnParams, RemoteObjectId,
};
use chromiumoxide::Page;

use crate::audit::normalized::{
    InteractiveFinding, InteractiveFindingKind, InteractiveFindingValues, JourneyStep, JourneyTrace,
};
use crate::error::Result;
use crate::interaction::{focus, keyboard, pointer, stability};
use crate::patterns::JourneyCandidate;
use crate::taxonomy::Severity;

enum Target {
    Found(RemoteObjectId),
    /// `href="#x"`, but no element with that id or name.
    Missing,
    /// No same-page fragment to resolve.
    NoFragment,
}

/// Focuses the skip link and resolves the element it points to.
///
/// Returns the link's own remote object id with the target. `None` when the
/// lookup itself failed — nothing can be said about the target then.
async fn focus_and_resolve(page: &Page, trigger_id: i64) -> Option<(RemoteObjectId, Target)> {
    let resolve = ResolveNodeParams::builder()
        .backend_node_id(BackendNodeId::new(trigger_id))
        .build();
    let resolved = page.execute(resolve).await.ok()?;
    let trigger = resolved.result.object.object_id.clone()?;
    let call = CallFunctionOnParams::builder()
        .function_declaration(
            "function() {
                this.focus();
                const href = this.getAttribute('href') || '';
                if (!href.startsWith('#') || href.length < 2) return 'no_fragment';
                let id = href.slice(1);
                try { id = decodeURIComponent(id); } catch (e) {}
                return document.getElementById(id) || document.getElementsByName(id)[0] || 'missing';
            }"
            .to_string(),
        )
        .object_id(trigger.clone())
        .return_by_value(false)
        .build()
        .ok()?;
    let result = page.execute(call).await.ok()?.result.result;
    let target = match (
        result.object_id,
        result.value.as_ref().and_then(|v| v.as_str()),
    ) {
        (Some(id), _) => Target::Found(id),
        (None, Some("missing")) => Target::Missing,
        (None, Some("no_fragment")) => Target::NoFragment,
        _ => return None,
    };
    Some((trigger, target))
}

/// Where focus sits relative to the target: `inside`, `after`, `before`,
/// `trigger` (still on the skip link) or `none` (on `body`). `None` when the
/// read failed.
async fn focus_relation(
    page: &Page,
    target: &RemoteObjectId,
    trigger: &RemoteObjectId,
) -> Option<String> {
    let call = CallFunctionOnParams::builder()
        .function_declaration(
            "function(trigger) {
                const a = document.activeElement;
                if (!a || a === document.body || a === document.documentElement) return 'none';
                if (a === this || this.contains(a)) return 'inside';
                if (a === trigger) return 'trigger';
                return (this.compareDocumentPosition(a) & Node.DOCUMENT_POSITION_FOLLOWING)
                    ? 'after' : 'before';
            }"
            .to_string(),
        )
        .object_id(target.clone())
        .argument(CallArgument::builder().object_id(trigger.clone()).build())
        .return_by_value(true)
        .build()
        .ok()?;
    let result = page.execute(call).await.ok()?;
    result.result.result.value?.as_str().map(|s| s.to_string())
}

fn reached(relation: &str) -> bool {
    matches!(relation, "inside" | "after")
}

pub async fn test(
    page: &Page,
    candidate: &JourneyCandidate,
    index: usize,
) -> Result<(JourneyTrace, Vec<InteractiveFinding>)> {
    let journey_name = format!("skip_link_{index}");
    let mut trace = JourneyTrace {
        journey: journey_name.clone(),
        steps: Vec::new(),
    };
    let mut findings: Vec<InteractiveFinding> = Vec::new();

    let trigger_id = match candidate.trigger_backend_id {
        Some(id) => id,
        None => return Ok((trace, findings)),
    };

    let Some((trigger, target)) = focus_and_resolve(page, trigger_id).await else {
        trace.steps.push(JourneyStep {
            action: "resolve_target".to_string(),
            target: Some(format!("backend_node:{trigger_id}")),
            focus: None,
            result: Some("not_observable".to_string()),
            snapshot_label: None,
        });
        return Ok((trace, findings));
    };
    trace.steps.push(JourneyStep {
        action: "resolve_target".to_string(),
        target: Some(format!("backend_node:{trigger_id}")),
        focus: None,
        result: Some(
            match &target {
                Target::Found(_) => "target_resolved",
                Target::Missing => "target_missing",
                Target::NoFragment => "no_fragment",
            }
            .to_string(),
        ),
        snapshot_label: None,
    });

    // Activate the skip link via synthetic click.
    if let Err(e) = pointer::synthetic_click_backend(page, trigger_id).await {
        tracing::warn!("skip_link: click on backend node {trigger_id} failed: {e}");
        return Ok((trace, findings));
    }
    trace.steps.push(JourneyStep {
        action: "synthetic_click".to_string(),
        target: Some(format!("backend_node:{trigger_id}")),
        focus: None,
        result: None,
        snapshot_label: Some("after_skip_click".to_string()),
    });

    let _ = stability::settle_after_action(page).await;

    let moved = match target {
        Target::Found(target) => {
            let Some(relation) = focus_relation(page, &target, &trigger).await else {
                trace
                    .steps
                    .push(check_step("check_focus", None, "not_observable"));
                return Ok((trace, findings));
            };
            if reached(&relation) {
                trace
                    .steps
                    .push(check_step("check_focus", None, "focus_on_target"));
                true
            } else if matches!(relation.as_str(), "none" | "trigger") {
                // Focus stayed put — correct for a non-focusable target if the
                // Tab starting point moved. The next Tab tells.
                keyboard::press_tab(page).await?;
                let _ = stability::settle_after_action(page).await;
                let after_tab = focus_relation(page, &target, &trigger).await;
                let focus_selector = focus::capture_focus(page).await?.selector;
                let Some(after_tab) = after_tab else {
                    trace.steps.push(check_step(
                        "check_focus_after_tab",
                        focus_selector,
                        "not_observable",
                    ));
                    return Ok((trace, findings));
                };
                let ok = reached(&after_tab);
                trace.steps.push(check_step(
                    "check_focus_after_tab",
                    focus_selector,
                    if ok {
                        "tab_reaches_target"
                    } else {
                        "focus_not_moved"
                    },
                ));
                ok
            } else {
                let focus_selector = focus::capture_focus(page).await?.selector;
                trace.steps.push(check_step(
                    "check_focus",
                    focus_selector,
                    "focus_before_target",
                ));
                false
            }
        }
        Target::Missing => {
            trace
                .steps
                .push(check_step("check_focus", None, "target_missing"));
            false
        }
        Target::NoFragment => {
            let on_trigger = focus::capture_focus_backend_id(page).await == Some(trigger_id);
            let snap = focus::capture_focus(page).await?;
            let focus_on_body = snap.selector.as_deref().is_none_or(|s| {
                let low = s.to_lowercase();
                low == "body" || low == "html"
            });
            let stayed = on_trigger || focus_on_body;
            trace.steps.push(check_step(
                "check_focus",
                snap.selector,
                if stayed {
                    "focus_not_moved"
                } else {
                    "focus_moved"
                },
            ));
            !stayed
        }
    };

    if !moved {
        findings.push(InteractiveFinding::new(
            "SkipLink",
            InteractiveFindingKind::SkipLinkFocusNotMoved,
            None,
            Severity::High,
            journey_name,
            None,
            Some("after_skip_link".to_string()),
            InteractiveFindingValues::default(),
        ));
    }

    Ok((trace, findings))
}

fn check_step(action: &str, focus: Option<String>, result: &str) -> JourneyStep {
    JourneyStep {
        action: action.to_string(),
        target: None,
        focus,
        result: Some(result.to_string()),
        snapshot_label: Some("after_skip_link".to_string()),
    }
}
