//! Turns recorded journey traces + focus snapshots into
//! `InteractiveFinding`s.
//!
//! Each evaluator function takes the *evidence* from a journey runner
//! and produces findings without itself touching the browser. That keeps
//! the runtime side effects (CDP, JS evaluate, …) and the evaluation
//! logic in separate, testable units.
//!
//! Currently covers:
//! - hidden focusables (aria-hidden, inert, hidden-by-style)
//! - missing focus indicator (no outline / box-shadow / border on :focus)
//! - tab order vs. DOM order (`tab_walk_order`, separate evaluator)

use crate::accessibility::{FocusIndicatorStatus, FocusSnapshot};
use crate::audit::normalized::{InteractiveFinding, JourneyTrace};
use crate::taxonomy::Severity;

/// Evaluate a tab-walk trace and its per-step focus snapshots.
///
/// Pure function: deterministic given the inputs. No browser calls.
pub fn tab_walk(trace: &JourneyTrace, snapshots: &[FocusSnapshot]) -> Vec<InteractiveFinding> {
    let mut findings = Vec::new();

    // Walk steps + snapshots in lockstep. `trace.steps[i]` corresponds to
    // `snapshots[i]` by construction in `tab_walk::record`.
    for (step, snap) in trace.steps.iter().zip(snapshots.iter()) {
        // Hidden focusables are only meaningful for tab-press steps. The
        // "start" step records the initial state and isn't an interaction.
        if step.action != "tab" {
            continue;
        }
        let Some(selector) = &step.focus else {
            continue;
        };

        if snap.aria_hidden_chain {
            findings.push(InteractiveFinding {
                category: "HiddenFocusable".to_string(),
                maps_to_finding: Some("a11y.aria_hidden_focus.invalid".to_string()),
                severity: Severity::High,
                journey: trace.journey.clone(),
                before_snapshot_label: None,
                after_snapshot_label: step.snapshot_label.clone(),
                message: format!(
                    "Keyboard focus lands on an element inside an aria-hidden \
                     region ({selector}). Screen reader users reach an element \
                     that is hidden from the accessibility tree."
                ),
                fix_suggestion: Some(
                    "Remove the element from the aria-hidden region or set \
                     tabindex=\"-1\" on it."
                        .to_string(),
                ),
            });
        } else if snap.inert_chain {
            findings.push(InteractiveFinding {
                category: "HiddenFocusable".to_string(),
                maps_to_finding: Some("a11y.aria_hidden_focus.invalid".to_string()),
                severity: Severity::High,
                journey: trace.journey.clone(),
                before_snapshot_label: None,
                after_snapshot_label: step.snapshot_label.clone(),
                message: format!(
                    "Keyboard focus lands on an element inside an inert \
                     region ({selector}). Inert regions should not be \
                     reachable by keyboard."
                ),
                fix_suggestion: Some(
                    "Remove the element from the inert region or \
                     correct the tabindex/focus chain."
                        .to_string(),
                ),
            });
        } else if snap.hidden_by_style {
            findings.push(InteractiveFinding {
                category: "HiddenFocusable".to_string(),
                // CSS-hidden elements (display:none / visibility:hidden / opacity:0) are a
                // different failure type from aria-hidden-focus — no existing static rule covers
                // this exact case, so there is nothing to reference.
                maps_to_finding: None,
                severity: Severity::Medium,
                journey: trace.journey.clone(),
                before_snapshot_label: None,
                after_snapshot_label: step.snapshot_label.clone(),
                message: format!(
                    "Keyboard focus lands on a visually hidden element \
                     ({selector}: display:none, visibility:hidden, or \
                     opacity:0). Keyboard users lose orientation."
                ),
                fix_suggestion: Some(
                    "Remove the element from the tab sequence (tabindex=\"-1\") \
                     or make it visible before it receives focus."
                        .to_string(),
                ),
            });
        } else if matches!(
            snap.focus_indicator,
            Some(FocusIndicatorStatus::NotDetected)
        ) {
            findings.push(InteractiveFinding {
                category: "FocusIndicator".to_string(),
                maps_to_finding: None,
                severity: Severity::Medium,
                journey: trace.journey.clone(),
                before_snapshot_label: None,
                after_snapshot_label: step.snapshot_label.clone(),
                message: format!(
                    "Element ({selector}) shows no visible focus indicator when focused \
                     (no outline, no box-shadow, no border change). \
                     Keyboard users lose orientation."
                ),
                fix_suggestion: Some(
                    "Add a CSS :focus-visible rule with a clear outline, \
                     box-shadow, or border change compared to the unfocused state."
                        .to_string(),
                ),
            });
        }
    }

    findings
}

/// Evaluate tab order against the DOM order of focusable elements.
///
/// Emits a single Warning-severity finding when the tab sequence jumps
/// **backwards** relative to the DOM — that is, the focus moves to an
/// element that comes earlier in the DOM than the previously focused one.
/// Forward gaps (skipping elements) are ignored: Grid/Flex/Sticky layouts
/// make them too ambiguous to call.
///
/// Pure function — takes the evidence from the tab-walk runner.
pub fn tab_walk_order(trace: &JourneyTrace, dom_order: &[String]) -> Vec<InteractiveFinding> {
    if dom_order.is_empty() {
        return Vec::new();
    }

    // Build selector → DOM index lookup.
    let dom_index =
        |selector: &str| -> Option<usize> { dom_order.iter().position(|s| s == selector) };

    let mut last_dom_index: Option<usize> = None;
    let mut reverse_jumps: Vec<String> = Vec::new();

    for step in &trace.steps {
        if step.action != "tab" {
            continue;
        }
        let Some(selector) = &step.focus else {
            continue;
        };
        let Some(idx) = dom_index(selector) else {
            // Element not in our pre-walk focusable list — could be a
            // dynamically inserted control. Skip rather than flag.
            continue;
        };
        if let Some(prev) = last_dom_index {
            if idx < prev {
                reverse_jumps.push(selector.clone());
            }
        }
        last_dom_index = Some(idx);
    }

    if reverse_jumps.is_empty() {
        return Vec::new();
    }

    // Aggregate one finding per page, not per jump — avoids spam on pages
    // with several legitimate-but-noisy layouts.
    let preview = reverse_jumps
        .iter()
        .take(3)
        .cloned()
        .collect::<Vec<_>>()
        .join(", ");
    let count = reverse_jumps.len();
    let suffix = if count > 3 { " (…)" } else { "" };

    vec![InteractiveFinding {
        category: "TabOrder".to_string(),
        maps_to_finding: None,
        severity: Severity::Medium,
        journey: trace.journey.clone(),
        before_snapshot_label: None,
        after_snapshot_label: None,
        message: format!(
            "Tab order deviates from DOM order: {count} backward {} \
             observed. First affected elements: {preview}{suffix}. \
             Keyboard users may not be able to follow the reading flow.",
            if count == 1 { "jump" } else { "jumps" }
        ),
        fix_suggestion: Some(
            "Avoid negative or high tabindex values. \
             Arrange the reading/DOM order to match the visual order."
                .to_string(),
        ),
    }]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::normalized::JourneyStep;

    fn step_tab(selector: &str, label: &str) -> JourneyStep {
        JourneyStep {
            action: "tab".to_string(),
            target: None,
            focus: Some(selector.to_string()),
            result: None,
            snapshot_label: Some(label.to_string()),
        }
    }

    fn snap_with(aria: bool, inert: bool, hidden: bool) -> FocusSnapshot {
        snap_full(aria, inert, hidden, None)
    }

    fn snap_full(
        aria: bool,
        inert: bool,
        hidden: bool,
        indicator: Option<FocusIndicatorStatus>,
    ) -> FocusSnapshot {
        FocusSnapshot {
            selector: Some("a".to_string()),
            aria_hidden_chain: aria,
            inert_chain: inert,
            hidden_by_style: hidden,
            focus_indicator: indicator,
            ..Default::default()
        }
    }

    fn trace_with(steps: Vec<JourneyStep>) -> JourneyTrace {
        JourneyTrace {
            journey: "tab_walk".to_string(),
            steps,
        }
    }

    #[test]
    fn clean_walk_produces_no_findings() {
        let trace = trace_with(vec![step_tab("a#one", "after_tab_1")]);
        let snaps = vec![snap_with(false, false, false)];
        assert!(tab_walk(&trace, &snaps).is_empty());
    }

    #[test]
    fn aria_hidden_chain_emits_high_finding() {
        let trace = trace_with(vec![step_tab("a#one", "after_tab_1")]);
        let snaps = vec![snap_with(true, false, false)];
        let findings = tab_walk(&trace, &snaps);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].category, "HiddenFocusable");
        assert_eq!(findings[0].severity, Severity::High);
        assert!(findings[0].message.contains("aria-hidden"));
    }

    #[test]
    fn inert_chain_emits_high_finding() {
        let trace = trace_with(vec![step_tab("a#one", "after_tab_1")]);
        let snaps = vec![snap_with(false, true, false)];
        let findings = tab_walk(&trace, &snaps);
        assert_eq!(findings.len(), 1);
        assert!(findings[0].message.contains("inert"));
    }

    #[test]
    fn hidden_by_style_emits_medium_finding() {
        let trace = trace_with(vec![step_tab("a#one", "after_tab_1")]);
        let snaps = vec![snap_with(false, false, true)];
        let findings = tab_walk(&trace, &snaps);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::Medium);
    }

    #[test]
    fn aria_hidden_takes_precedence_over_inert() {
        // Both flags set — only the highest-priority finding is emitted.
        let trace = trace_with(vec![step_tab("a#one", "after_tab_1")]);
        let snaps = vec![snap_with(true, true, true)];
        let findings = tab_walk(&trace, &snaps);
        assert_eq!(findings.len(), 1);
        assert!(findings[0].message.contains("aria-hidden"));
    }

    #[test]
    fn start_step_is_ignored() {
        let mut trace = trace_with(vec![]);
        trace.steps.push(JourneyStep {
            action: "start".to_string(),
            target: None,
            focus: Some("body".to_string()),
            result: None,
            snapshot_label: Some("initial".to_string()),
        });
        let snaps = vec![snap_with(true, false, false)];
        assert!(tab_walk(&trace, &snaps).is_empty());
    }

    #[test]
    fn focus_indicator_not_detected_emits_finding() {
        let trace = trace_with(vec![step_tab("a#one", "after_tab_1")]);
        let snaps = vec![snap_full(
            false,
            false,
            false,
            Some(FocusIndicatorStatus::NotDetected),
        )];
        let findings = tab_walk(&trace, &snaps);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].category, "FocusIndicator");
        assert_eq!(findings[0].severity, Severity::Medium);
        assert!(findings[0].message.contains("focus indicator"));
    }

    #[test]
    fn focus_indicator_detected_emits_no_finding() {
        let trace = trace_with(vec![step_tab("a#one", "after_tab_1")]);
        let snaps = vec![snap_full(
            false,
            false,
            false,
            Some(FocusIndicatorStatus::Detected),
        )];
        assert!(tab_walk(&trace, &snaps).is_empty());
    }

    #[test]
    fn focus_indicator_ambiguous_emits_no_finding() {
        // Ambiguous is suppressed in evaluation — too noisy to surface as a
        // finding. Phase 3 may aggregate it.
        let trace = trace_with(vec![step_tab("a#one", "after_tab_1")]);
        let snaps = vec![snap_full(
            false,
            false,
            false,
            Some(FocusIndicatorStatus::Ambiguous),
        )];
        assert!(tab_walk(&trace, &snaps).is_empty());
    }

    #[test]
    fn hidden_focusable_takes_precedence_over_indicator() {
        // If aria-hidden / inert / hidden-by-style fires, we don't also flag
        // missing indicator on the same element.
        let trace = trace_with(vec![step_tab("a#one", "after_tab_1")]);
        let snaps = vec![snap_full(
            true,
            false,
            false,
            Some(FocusIndicatorStatus::NotDetected),
        )];
        let findings = tab_walk(&trace, &snaps);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].category, "HiddenFocusable");
        assert_eq!(
            findings[0].maps_to_finding.as_deref(),
            Some("a11y.aria_hidden_focus.invalid")
        );
    }

    // ── tab_walk_order tests ─────────────────────────────────────────────

    #[test]
    fn forward_walk_produces_no_order_finding() {
        let trace = trace_with(vec![
            step_tab("a", "after_tab_1"),
            step_tab("b", "after_tab_2"),
            step_tab("c", "after_tab_3"),
        ]);
        let dom = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        assert!(tab_walk_order(&trace, &dom).is_empty());
    }

    #[test]
    fn reverse_jump_emits_warning() {
        // Tab goes a → c → b (b is before c in DOM) → reverse jump on step 3.
        let trace = trace_with(vec![
            step_tab("a", "after_tab_1"),
            step_tab("c", "after_tab_2"),
            step_tab("b", "after_tab_3"),
        ]);
        let dom = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let findings = tab_walk_order(&trace, &dom);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].category, "TabOrder");
        assert_eq!(findings[0].severity, Severity::Medium);
        assert!(findings[0].message.contains("b"));
    }

    #[test]
    fn forward_gap_is_not_a_finding() {
        // Skipping "b" forward is allowed — Grid/Flex layouts.
        let trace = trace_with(vec![
            step_tab("a", "after_tab_1"),
            step_tab("c", "after_tab_2"),
        ]);
        let dom = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        assert!(tab_walk_order(&trace, &dom).is_empty());
    }

    #[test]
    fn empty_dom_order_produces_no_finding() {
        let trace = trace_with(vec![step_tab("a", "after_tab_1")]);
        assert!(tab_walk_order(&trace, &[]).is_empty());
    }

    #[test]
    fn unknown_selector_does_not_panic() {
        // Selector that's not in the DOM list — dynamic content. Skip rather
        // than flag.
        let trace = trace_with(vec![
            step_tab("a", "after_tab_1"),
            step_tab("z", "after_tab_2"),
        ]);
        let dom = vec!["a".to_string(), "b".to_string()];
        assert!(tab_walk_order(&trace, &dom).is_empty());
    }
}
