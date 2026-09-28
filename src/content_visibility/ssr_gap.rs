//! SSR/hydration content-gap detection (#534).
//!
//! Reloads the audited page a second time with JavaScript execution disabled
//! and compares the resulting visible-text length against the same
//! measurement taken from the normal, JavaScript-enabled load. A large drop
//! signals that a meaningful share of the page's content only exists after
//! client-side hydration — relevant for search crawlers and any assistive
//! technology that does not execute JavaScript. Hybrid/partial-hydration
//! frameworks (e.g. Astro Islands) are the motivating case from the issue,
//! but the check itself is deliberately generic — it does not try to detect
//! any specific framework, only the resulting content gap.
//!
//! # CDP mechanism
//!
//! chromiumoxide does not expose a `Page.setJavaScriptEnabled` method
//! because no such raw CDP command exists — the actual protocol command is
//! [`Emulation.setScriptExecutionDisabled`](https://chromedevtools.github.io/devtools-protocol/tot/Emulation/#method-setScriptExecutionDisabled),
//! generated here as `SetScriptExecutionDisabledParams`. Disabling script
//! execution only stops scripts embedded in the *page's own* HTML/`<script>`
//! tags; the devtools `Runtime.evaluate` channel `Page::evaluate` uses stays
//! available regardless, which is what makes reading the JS-disabled DOM via
//! `page.evaluate()` possible at all.
//!
//! # Opt-in / cost
//!
//! Gated behind `--check-ssr-content`
//! (`PipelineConfig.check_ssr_content`): costs one extra full page reload
//! per audited page, so it is never run by default and only invoked from
//! `run_single_audit` (single-URL mode) — same cost trade-off precedent as
//! `performance::isolate_third_party_impact` (#531).

use chromiumoxide::cdp::browser_protocol::emulation::SetScriptExecutionDisabledParams;
use chromiumoxide::Page;
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::browser::BrowserManager;
use crate::error::{AuditError, Result};

/// Below this fraction of the JS-enabled content length, the JS-disabled
/// render is considered a meaningful SSR/hydration gap.
///
/// 50% is a deliberately conservative cut: it flags pages where at least
/// half of the visible content depends on client-side JavaScript to appear
/// (the "SSR shell + client-hydrated islands" scenario #534 is about), while
/// not flagging pages that merely inject a small JS-driven widget or CTA on
/// top of an otherwise complete server-rendered page.
pub const SSR_GAP_RATIO_THRESHOLD: f64 = 0.5;

/// Below this JS-enabled content length (characters), the ratio comparison
/// is not meaningful — an already near-empty page is a content problem (see
/// `content_visibility`'s own word-count signals), not evidence of a missing
/// SSR render, and dividing by a tiny denominator would make the ratio noisy.
pub const MIN_MEASURABLE_CONTENT_CHARS: usize = 200;

/// Result of comparing visible text content with and without JavaScript.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SsrContentGap {
    /// `document.body.innerText.trim().length` measured on the normal,
    /// JavaScript-enabled load.
    pub with_js_content_chars: usize,
    /// The same measurement taken after reloading with JavaScript disabled.
    pub without_js_content_chars: usize,
    /// `without_js_content_chars / with_js_content_chars`, or `0.0` when
    /// `with_js_content_chars` is `0`.
    pub ratio: f64,
    /// True when the drop is large enough, and the page had enough content
    /// to begin with, to count as a meaningful SSR/hydration gap.
    pub has_gap: bool,
}

/// Pure gap classification — no browser required, unit-testable in isolation
/// from the CDP measurement in [`measure_ssr_content_gap`].
pub fn compute_ssr_content_gap(with_js_chars: usize, without_js_chars: usize) -> SsrContentGap {
    let ratio = if with_js_chars == 0 {
        0.0
    } else {
        without_js_chars as f64 / with_js_chars as f64
    };
    let has_gap = with_js_chars >= MIN_MEASURABLE_CONTENT_CHARS && ratio < SSR_GAP_RATIO_THRESHOLD;
    SsrContentGap {
        with_js_content_chars: with_js_chars,
        without_js_content_chars: without_js_chars,
        ratio,
        has_gap,
    }
}

const CONTENT_LENGTH_JS: &str = "(document.body ? document.body.innerText : '').trim().length";

async fn read_visible_content_chars(page: &Page) -> Result<usize> {
    let result = page
        .evaluate(CONTENT_LENGTH_JS)
        .await
        .map_err(|e| AuditError::CdpError(format!("Visible-content length read failed: {e}")))?;
    Ok(result.value().and_then(|v| v.as_u64()).unwrap_or(0) as usize)
}

async fn set_script_execution_disabled(page: &Page, disabled: bool) -> Result<()> {
    page.execute(SetScriptExecutionDisabledParams::new(disabled))
        .await
        .map_err(|e| {
            AuditError::CdpError(format!("Emulation.setScriptExecutionDisabled failed: {e}"))
        })?;
    Ok(())
}

/// Measure the SSR/hydration content gap for `url` (#534, opt-in via
/// `--check-ssr-content`).
///
/// Reads the visible-content length from the already-loaded `page` (the
/// normal, JavaScript-enabled audit pass), then reloads the same URL with
/// JavaScript execution disabled and re-measures. Always re-enables script
/// execution before returning, regardless of outcome — the page must stay
/// usable for any subsequent pass in `run_single_audit` (same "always
/// restore state after use" precedent as
/// `third_party::isolate_third_party_impact`'s
/// `set_blocked_urls(page, &[])` cleanup after each isolation pass).
///
/// Returns `None` if any CDP step fails; the caller simply omits the
/// finding rather than surfacing a broken/partial measurement.
pub async fn measure_ssr_content_gap(
    page: &Page,
    browser: &BrowserManager,
    url: &str,
) -> Option<SsrContentGap> {
    let with_js_chars = match read_visible_content_chars(page).await {
        Ok(chars) => chars,
        Err(e) => {
            warn!(
                "Failed to read JS-enabled content length for SSR check: {}",
                e
            );
            return None;
        }
    };

    if let Err(e) = set_script_execution_disabled(page, true).await {
        warn!("Failed to disable script execution for SSR check: {}", e);
        return None;
    }

    let without_js_chars = match browser.navigate(page, url).await {
        Ok(_) => match read_visible_content_chars(page).await {
            Ok(chars) => Some(chars),
            Err(e) => {
                warn!(
                    "Failed to read JS-disabled content length for SSR check: {}",
                    e
                );
                None
            }
        },
        Err(e) => {
            warn!("Navigation with JS disabled failed for SSR check: {}", e);
            None
        }
    };

    // Always re-enable, regardless of outcome — the page must remain usable
    // for any subsequent pass (throttled performance, cache persistence, …).
    if let Err(e) = set_script_execution_disabled(page, false).await {
        warn!(
            "Failed to re-enable script execution after SSR check: {}",
            e
        );
    }

    without_js_chars.map(|chars| compute_ssr_content_gap(with_js_chars, chars))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flags_a_large_content_drop_as_a_gap() {
        let gap = compute_ssr_content_gap(2000, 200);
        assert_eq!(gap.ratio, 0.1);
        assert!(gap.has_gap);
    }

    #[test]
    fn does_not_flag_a_small_content_drop() {
        let gap = compute_ssr_content_gap(2000, 1600);
        assert_eq!(gap.ratio, 0.8);
        assert!(!gap.has_gap);
    }

    #[test]
    fn boundary_ratio_just_below_threshold_flags_a_gap() {
        let gap = compute_ssr_content_gap(1000, 499);
        assert!(gap.ratio < SSR_GAP_RATIO_THRESHOLD);
        assert!(gap.has_gap);
    }

    #[test]
    fn boundary_ratio_at_threshold_does_not_flag_a_gap() {
        let gap = compute_ssr_content_gap(1000, 500);
        assert_eq!(gap.ratio, SSR_GAP_RATIO_THRESHOLD);
        assert!(!gap.has_gap);
    }

    #[test]
    fn near_empty_with_js_page_never_flags_a_gap() {
        // A page with almost no content even with JS enabled is a content
        // problem picked up elsewhere (e.g. content_visibility's word-count
        // signals), not evidence of a missing SSR render — must not divide
        // by a tiny denominator and produce a spurious "gap".
        let gap = compute_ssr_content_gap(50, 5);
        assert!(!gap.has_gap);
    }

    #[test]
    fn zero_with_js_content_does_not_divide_by_zero() {
        let gap = compute_ssr_content_gap(0, 0);
        assert_eq!(gap.ratio, 0.0);
        assert!(!gap.has_gap);
    }
}
