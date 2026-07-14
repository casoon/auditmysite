//! WCAG 2.2.2 Pause, Stop, Hide (Level A)
//!
//! "For moving, blinking, scrolling, or auto-updating information that (a)
//! starts automatically, (b) lasts more than five seconds, and (c) is
//! presented in parallel with other content, there is a mechanism for the
//! user to pause, stop, or hide it, unless the movement, blinking, scrolling,
//! or auto-updating is part of an activity where it is essential."
//!
//! **What this checks.** A single DOM+CSSOM scan (no live timing/observation
//! — no waiting to see whether something is actually still moving after 5
//! seconds), scoped to two concrete, low-false-positive anti-patterns:
//!
//! 1. **`<marquee>` elements.** This legacy tag has no native pause
//!    affordance at all — any use of it is content that starts automatically,
//!    scrolls indefinitely, and cannot be paused/stopped/hidden by the
//!    element itself. Presence alone is a strong signal.
//! 2. **CSS animations/transitions that loop indefinitely for a substantial
//!    duration** — elements whose computed style has
//!    `animation-iteration-count: infinite` and a per-iteration
//!    `animation-duration` over 5 seconds. Short infinite loops (spinners,
//!    subtle pulses well under a second per cycle) are deliberately excluded
//!    — they read as decorative micro-interactions rather than the kind of
//!    slow, attention-grabbing motion this criterion targets, and excluding
//!    them keeps false positives down.
//!
//! Both signals are suppressed if the page appears to already offer *some*
//! plausible pause/stop control (a button/role=button/link anywhere on the
//! page whose accessible text or `aria-label`/`title` mentions
//! pause/stop/animation, in German or English). This is a page-wide, not a
//! per-element, check — this project's existing per-component pattern
//! detectors (`src/patterns/`) have no carousel/slider/ticker detection to
//! reuse, and building precise DOM-proximity ("is this specific control next
//! to this specific animated element") logic from scratch was judged
//! out of scope for a single new rule. The tradeoff is explicit: a global
//! pause control for widget A can incorrectly suppress a genuine violation
//! in unrelated widget B. Kept conservative in the direction of fewer false
//! positives, consistent with this criterion landing as a heuristic warning.
//!
//! CSS-animation findings are also suppressed when the page's stylesheets
//! already contain an `@media (prefers-reduced-motion` rule (mirroring the
//! detection `reduced_motion.rs` uses for 2.3.3) — a site that already reduces
//! or disables animation for that preference has effectively provided a
//! "hide" mechanism for the users who need it most. This does not apply to
//! the `<marquee>` case: `prefers-reduced-motion` is a CSS media feature and
//! cannot affect a `<marquee>` element's built-in scrolling behavior, so a
//! marquee finding fires regardless.
//!
//! Deliberately **not** implemented:
//! - Auto-advancing carousel/slider detection. `src/patterns/` (this
//!   project's UI-pattern-detection module) has no carousel/slider/ticker
//!   pattern to reuse, and fingerprinting carousel libraries (Slick, Swiper,
//!   …) from scratch is exactly the scope-creep this rule avoids — it would
//!   need a much larger, separate effort to do reliably.
//! - Live observation of whether content is *actually* still moving after 5
//!   seconds (e.g. re-checking computed styles after a real wait, or
//!   watching for DOM mutations). This check only reads static, declared
//!   CSS/HTML, which cannot see JS-driven auto-updating regions (e.g. a
//!   `setInterval`-refreshed ticker with no CSS animation at all).
//! - CDP-level `prefers-reduced-motion` emulation/re-diffing (as
//!   `dark_mode/mod.rs` does for `prefers-color-scheme`). The static
//!   stylesheet scan already used by `reduced_motion.rs` answers the same
//!   underlying question ("does the site define reduced-motion behavior at
//!   all") without a second page load.
//!
//! Because absence of a detected pause control does not prove no such
//! mechanism exists (e.g. a keyboard shortcut, or a settings page), findings
//! are reported as [`crate::wcag::types::FindingKind::Warning`] (heuristic,
//! manual-review candidate) — the same treatment as `redundant_entry.rs`,
//! `meaningful_sequence.rs`, and `focus_not_obscured_minimum.rs`.

use chromiumoxide::Page;

use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation};

pub const PAUSE_STOP_HIDE_RULE: RuleMetadata = RuleMetadata {
    id: "2.2.2",
    name: "Pause, Stop, Hide",
    level: WcagLevel::A,
    severity: Severity::Medium,
    description: "Auto-starting moving, blinking, or scrolling content that lasts more than 5 seconds can be paused, stopped, or hidden",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/pause-stop-hide.html",
    axe_id: "pause-stop-hide",
    tags: &["wcag2a", "wcag222", "cat.time-and-media"],
};

/// Per-iteration duration, in seconds, above which an indefinitely-looping
/// CSS animation is treated as a candidate violation. Short infinite loops
/// (spinners, subtle pulses) are excluded to keep false positives down — see
/// module docs.
const MIN_INFINITE_ANIMATION_SECONDS: f64 = 5.0;

/// One indefinitely-looping animated element, as extracted from the live
/// DOM/CSSOM.
#[derive(Debug, Clone)]
struct AnimatedElement {
    selector: String,
    duration_seconds: f64,
}

/// Pure decision function: given the raw page scan results, decide which
/// findings (if any) fire. Mirrors the shape of `redundant_entry`'s
/// `group_and_flag` / `meaningful_sequence`'s `is_visually_reordered` — a
/// pure, unit-testable core wrapped by the `_with_page` JS-collection glue.
fn decide(
    marquee_count: usize,
    infinite_animations: &[AnimatedElement],
    has_pause_control: bool,
    respects_reduced_motion: bool,
) -> (bool, Vec<AnimatedElement>) {
    let marquee_fires = marquee_count > 0 && !has_pause_control;
    let animations_fire = if has_pause_control || respects_reduced_motion {
        Vec::new()
    } else {
        infinite_animations.to_vec()
    };
    (marquee_fires, animations_fire)
}

const PAUSE_STOP_HIDE_JS: &str = r#"
(function() {
  var PAUSE_CONTROL_PATTERN = /\b(pause|stop|anhalten|stopp|bewegung|animation)\b/i;

  function isPlausiblePauseControl(el) {
    var text = (el.getAttribute('aria-label') || el.getAttribute('title') || el.textContent || '').trim();
    return PAUSE_CONTROL_PATTERN.test(text);
  }

  var hasPauseControl = false;
  var controls = document.querySelectorAll('button, [role="button"], a[href]');
  for (var c = 0; c < controls.length && !hasPauseControl; c++) {
    if (isPlausiblePauseControl(controls[c])) hasPauseControl = true;
  }

  var marqueeCount = document.querySelectorAll('marquee').length;

  var respectsReducedMotion = false;
  try {
    for (const sheet of Array.from(document.styleSheets)) {
      let rules;
      try { rules = Array.from(sheet.cssRules || []); }
      catch (e) { continue; }
      const walk = (rs) => {
        for (const r of rs) {
          if (r.type === CSSRule.MEDIA_RULE) {
            const condition = (r.conditionText || (r.media && r.media.mediaText) || '').toLowerCase();
            if (condition.includes('prefers-reduced-motion')) {
              respectsReducedMotion = true;
            }
            if (r.cssRules) walk(Array.from(r.cssRules));
          }
        }
      };
      walk(rules);
    }
  } catch (e) {}

  function parseMaxSeconds(value) {
    var parts = (value || '').split(',');
    var max = 0;
    for (var i = 0; i < parts.length; i++) {
      var v = parts[i].trim();
      var seconds = 0;
      if (v.endsWith('ms')) seconds = parseFloat(v) / 1000;
      else if (v.endsWith('s')) seconds = parseFloat(v);
      if (!isNaN(seconds) && seconds > max) max = seconds;
    }
    return max;
  }

  var animated = [];
  var all = document.querySelectorAll('*');
  for (var i = 0; i < all.length && animated.length < 5; i++) {
    var el = all[i];
    var cs = getComputedStyle(el);
    var iterationCount = cs.animationIterationCount || '';
    if (iterationCount.indexOf('infinite') === -1) continue;
    var duration = parseMaxSeconds(cs.animationDuration);
    if (duration <= 0) continue;
    animated.push({
      selector: el.tagName.toLowerCase() + (el.id ? '#' + el.id : ''),
      duration_seconds: duration
    });
  }

  return {
    marquee_count: marqueeCount,
    has_pause_control: hasPauseControl,
    respects_reduced_motion: respectsReducedMotion,
    animated: animated
  };
})()
"#;

pub async fn check_pause_stop_hide_with_page(page: &Page) -> Vec<Violation> {
    let result = match page.evaluate(PAUSE_STOP_HIDE_JS).await {
        Ok(r) => r,
        Err(_) => return vec![],
    };

    let val = match result.value() {
        Some(v) => v.clone(),
        None => return vec![],
    };

    let marquee_count = val
        .get("marquee_count")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize;
    let has_pause_control = val
        .get("has_pause_control")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let respects_reduced_motion = val
        .get("respects_reduced_motion")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let infinite_animations: Vec<AnimatedElement> = val
        .get("animated")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|entry| {
                    let selector = entry.get("selector").and_then(|v| v.as_str())?.to_string();
                    let duration_seconds =
                        entry.get("duration_seconds").and_then(|v| v.as_f64())?;
                    if duration_seconds <= MIN_INFINITE_ANIMATION_SECONDS {
                        return None;
                    }
                    Some(AnimatedElement {
                        selector,
                        duration_seconds,
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    let (marquee_fires, animations_fire) = decide(
        marquee_count,
        &infinite_animations,
        has_pause_control,
        respects_reduced_motion,
    );

    let mut violations = Vec::new();

    if marquee_fires {
        violations.push(
            Violation::new(
                PAUSE_STOP_HIDE_RULE.id,
                PAUSE_STOP_HIDE_RULE.name,
                PAUSE_STOP_HIDE_RULE.level,
                PAUSE_STOP_HIDE_RULE.severity,
                format!(
                    "Page uses {} legacy `<marquee>` element(s), which scroll automatically and \
                     indefinitely with no native pause, stop, or hide control, and no other \
                     pause control was detected on the page.",
                    marquee_count
                ),
                "marquee",
            )
            .with_selector("marquee")
            .with_fix(
                "Replace `<marquee>` with a modern implementation (CSS animation or a small \
                 script) that includes a visible pause/stop button, or remove the automatic \
                 scrolling entirely.",
            )
            .with_rule_id(PAUSE_STOP_HIDE_RULE.axe_id)
            .with_help_url(PAUSE_STOP_HIDE_RULE.help_url)
            .as_warning(),
        );
    }

    if !animations_fire.is_empty() {
        let sample = animations_fire
            .iter()
            .take(3)
            .map(|a| format!("{} ({:.1}s/cycle)", a.selector, a.duration_seconds))
            .collect::<Vec<_>>()
            .join(", ");
        violations.push(
            Violation::new(
                PAUSE_STOP_HIDE_RULE.id,
                PAUSE_STOP_HIDE_RULE.name,
                PAUSE_STOP_HIDE_RULE.level,
                PAUSE_STOP_HIDE_RULE.severity,
                format!(
                    "{} element(s) run a CSS animation with `animation-iteration-count: infinite` \
                     and a per-cycle duration over {}s ({}), with no pause/stop control and no \
                     `prefers-reduced-motion` handling detected on the page.",
                    animations_fire.len(),
                    MIN_INFINITE_ANIMATION_SECONDS,
                    sample
                ),
                &animations_fire[0].selector,
            )
            .with_selector(&animations_fire[0].selector)
            .with_fix(
                "Add a visible pause/stop control for the animation, or wrap it in \
                 `@media (prefers-reduced-motion: reduce) { animation: none; }` so users who need \
                 to stop it can.",
            )
            .with_rule_id(PAUSE_STOP_HIDE_RULE.axe_id)
            .with_help_url(PAUSE_STOP_HIDE_RULE.help_url)
            .as_warning(),
        );
    }

    violations
}

#[cfg(test)]
mod tests {
    use super::*;

    fn animation(selector: &str, seconds: f64) -> AnimatedElement {
        AnimatedElement {
            selector: selector.to_string(),
            duration_seconds: seconds,
        }
    }

    #[test]
    fn fires_for_marquee_with_no_pause_control() {
        let (marquee_fires, animations_fire) = decide(1, &[], false, false);
        assert!(marquee_fires);
        assert!(animations_fire.is_empty());
    }

    #[test]
    fn fires_for_long_infinite_animation_with_no_pause_control() {
        let animations = vec![animation("div.ticker", 8.0)];
        let (marquee_fires, animations_fire) = decide(0, &animations, false, false);
        assert!(!marquee_fires);
        assert_eq!(animations_fire.len(), 1);
    }

    #[test]
    fn does_not_fire_when_pause_control_detected() {
        let animations = vec![animation("div.ticker", 8.0)];
        let (marquee_fires, animations_fire) = decide(1, &animations, true, false);
        assert!(!marquee_fires);
        assert!(animations_fire.is_empty());
    }

    #[test]
    fn does_not_fire_for_animation_when_reduced_motion_is_respected() {
        let animations = vec![animation("div.ticker", 8.0)];
        let (marquee_fires, animations_fire) = decide(0, &animations, false, true);
        assert!(!marquee_fires);
        assert!(animations_fire.is_empty());
    }

    #[test]
    fn marquee_still_fires_when_reduced_motion_is_respected() {
        // prefers-reduced-motion is a CSS media feature and has no effect on
        // a <marquee> element's native scrolling behavior, so it must not
        // suppress the marquee finding.
        let (marquee_fires, animations_fire) = decide(1, &[], false, true);
        assert!(marquee_fires);
        assert!(animations_fire.is_empty());
    }
}
