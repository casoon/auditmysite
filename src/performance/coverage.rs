//! Unused JavaScript (#106) and Unused CSS (#107) detection via CDP Coverage API.
//!
//! **Protocol flow:**
//! 1. Before navigation: call `prepare_coverage_collection` → enables the Profiler
//!    and starts precise JS coverage + CSS rule usage tracking.
//! 2. After page load (in `extract_snapshot`): call `take_coverage_results` →
//!    reads back JS and CSS coverage and returns a `CoverageAnalysis`.
//!
//! JS coverage uses `Profiler.startPreciseCoverage` / `Profiler.takePreciseCoverage`
//! (JavaScript protocol).  CSS coverage uses `CSS.startRuleUsageTracking` /
//! `CSS.stopRuleUsageTracking` (browser protocol).
//!
//! **Limitations:**
//! - JS coverage only covers scripts that ran *after* `startPreciseCoverage`.
//!   Parser-executed scripts that ran before navigation started may show 0 % usage.
//! - CSS `stopRuleUsageTracking` returns counts of rules seen and used at the time
//!   of the call.  Rules triggered only on hover/focus/scroll may be reported as
//!   unused in a headless audit.
//! - Data URIs and cross-origin scripts that fail to load are excluded.
//!
//! **Duplicate asset detection (#551):** `take_coverage_results` additionally
//! fetches the full source of already-loaded scripts/stylesheets via
//! `Debugger.getScriptSource` / `CSS.getStyleSheetText` — content already
//! resident in the browser's memory, so this adds no new network request —
//! and hashes it to detect the same file served under 2+ different URLs.
//! Purely advisory: never affects the performance score.

use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use chromiumoxide::cdp::browser_protocol::css::{
    EnableParams as CssEnableParams, EventStyleSheetAdded, EventStyleSheetRemoved,
    GetStyleSheetTextParams, StartRuleUsageTrackingParams, StopRuleUsageTrackingParams,
    StyleSheetId,
};
use chromiumoxide::cdp::browser_protocol::dom::EnableParams as DomEnableParams;
use chromiumoxide::cdp::browser_protocol::target::TargetId;
use chromiumoxide::cdp::js_protocol::debugger::{
    EnableParams as DebuggerEnableParams, EventScriptParsed, GetScriptSourceParams,
};
use chromiumoxide::cdp::js_protocol::profiler::{
    EnableParams as ProfilerEnableParams, StartPreciseCoverageParams, TakePreciseCoverageParams,
};
use chromiumoxide::cdp::js_protocol::runtime::ScriptId;
use chromiumoxide::listeners::EventStream;
use chromiumoxide::Page;
use futures::{FutureExt, StreamExt};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::sync::Mutex;
use tracing::{info, warn};

use crate::error::{AuditError, Result};
use crate::util::truncate_ellipsis as truncate;

/// Minimum size (bytes) a fetched script/stylesheet source must have to be
/// considered for duplicate-content detection (#551) — mirrors the same
/// threshold `unused_js`'s `ScriptCoverageEntry` filtering already uses, to
/// avoid noise from trivial snippets.
const MIN_DUPLICATE_ASSET_BYTES: usize = 1024;

/// Safety cap on how many `CSS.styleSheetAdded` events are drained per pass
/// — mirrors the "cap the fan-out" philosophy used elsewhere in this codebase
/// (e.g. `design_quality`'s `MAX_SAMPLE_TASKS`) rather than trusting an
/// unbounded page to stay small.
const MAX_DUPLICATE_STYLESHEET_EVENTS: usize = 500;

/// Same cap, for `Debugger.scriptParsed` events (#551) — see
/// `MAX_DUPLICATE_STYLESHEET_EVENTS`.
const MAX_DUPLICATE_SCRIPT_EVENTS: usize = 500;

/// Pre-navigation `CSS.styleSheetAdded` / `CSS.styleSheetRemoved` listener
/// pair for one page. A sheet the page adds and removes again while loading
/// (an inline `<style>` a script swaps out) no longer exists when
/// `take_coverage_results` reads it back: `getStyleSheetText` fails with
/// "No style sheet with given id found", and counting it would mark the CSS
/// coverage `partial` although every sheet still on the page was read.
struct StylesheetStreams {
    added: EventStream<EventStyleSheetAdded>,
    removed: EventStream<EventStyleSheetRemoved>,
}

/// Pre-navigation `CSS.styleSheetAdded` listeners, keyed by CDP target id.
///
/// `prepare_coverage_collection` must register this listener *before*
/// navigation — chromiumoxide's event streams only deliver events emitted
/// after subscription, so subscribing once the page has already loaded (i.e.
/// inside `take_coverage_results`) would silently miss every stylesheet.
/// `take_coverage_results` reads this back (and removes the entry) once per
/// viewport pass to resolve `style_sheet_id -> URL` for duplicate-CSS
/// detection (#551) — `CSS.getStyleSheetText` itself never returns a URL,
/// only `styleSheetAdded` does (see the CSS domain's own PDL doc comment).
/// Routed through a target-id-keyed static instead of threading a new
/// parameter through `ModuleContext`/the `AuditModule` catalog: nothing in
/// that machinery carries state from `prepare_coverage_collection` (called
/// directly in the pipeline, pre-navigation) to `take_coverage_results`
/// (called several layers away, from `PerformanceModule::collect`, with only
/// a `&Page`) — same constraint and same fix shape as `network::dns`'s
/// per-host cache.
static STYLESHEET_LISTENERS: OnceLock<Mutex<HashMap<TargetId, StylesheetStreams>>> =
    OnceLock::new();

fn stylesheet_listeners() -> &'static Mutex<HashMap<TargetId, StylesheetStreams>> {
    STYLESHEET_LISTENERS.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Pre-navigation `Debugger.scriptParsed` listeners, keyed by CDP target id.
///
/// Same shape and same reason as `STYLESHEET_LISTENERS` above, and for a
/// related but distinct problem: unlike stylesheets (which never carry a URL
/// via `CSS.getStyleSheetText`, so the listener's only job is resolving one),
/// `Profiler.ScriptCoverage` already gives us each script's URL directly.
/// This listener instead confirms *which* script ids the Debugger domain has
/// actually registered a retrievable source for — live testing against a
/// real site showed that calling `Debugger.getScriptSource` for a
/// `Profiler.ScriptCoverage.script_id` fails with "No script for id" even
/// right after `Debugger.enable`, for effectively every script, not just
/// short-lived ones — the enable call's documented "retroactively backfills
/// already-parsed scripts" behavior does not reliably apply synchronously.
/// Draining this listener's buffered `scriptParsed` events before attempting
/// `getScriptSource` (mirroring the stylesheet listener's drain pattern)
/// avoids firing calls we already know will fail.
static SCRIPT_PARSED_LISTENERS: OnceLock<Mutex<HashMap<TargetId, EventStream<EventScriptParsed>>>> =
    OnceLock::new();

fn script_parsed_listeners() -> &'static Mutex<HashMap<TargetId, EventStream<EventScriptParsed>>> {
    SCRIPT_PARSED_LISTENERS.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Coverage summary for a single JS script.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptCoverageEntry {
    /// Script URL (truncated to 120 chars)
    pub url: String,
    /// Total bytes in the script
    pub total_bytes: u64,
    /// Bytes that were NOT executed during the audit
    pub unused_bytes: u64,
    /// Percentage of the script that was executed (0–100)
    pub used_pct: f64,
}

/// Unused JavaScript analysis (#106).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnusedJsAnalysis {
    /// Per-script breakdown, sorted by unused_bytes descending
    pub scripts: Vec<ScriptCoverageEntry>,
    /// Total bytes across all measured scripts
    pub total_bytes: u64,
    /// Total unused bytes
    pub unused_bytes: u64,
    /// Overall JS usage percentage (0–100); `None` when no script bytes
    /// were measured — no scripts, or the profiler gave no coverage (#769).
    pub used_pct: Option<f64>,
}

/// Unused CSS analysis (#107).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnusedCssAnalysis {
    /// Style rules in the stylesheets whose text could be read (including
    /// those nested in `@media`/`@supports`; no `@keyframes`/`@font-face`).
    pub total_rules: u32,
    /// Of those, the rules the browser reported as used between navigation
    /// and the end of the analysis.
    pub used_rules: u32,
    /// Percentage of rules that were used (0–100), or null when CDP returned no CSS data.
    pub used_pct: Option<f64>,
    /// Measurement state for CSS rule usage data.
    pub measurement: String,
}

/// A group of 2+ distinct URLs serving byte-identical content (#551) —
/// purely advisory/informational, does not affect the performance score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateAssetGroup {
    /// "script" or "stylesheet"
    pub kind: String,
    /// 2+ distinct URLs serving byte-identical content
    pub urls: Vec<String>,
    /// Size of one copy (all copies are byte-identical)
    pub bytes: u64,
}

/// Combined coverage analysis returned to the pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageAnalysis {
    /// Unused JavaScript (#106)
    pub unused_js: UnusedJsAnalysis,
    /// Unused CSS (#107)
    pub unused_css: UnusedCssAnalysis,
    /// Coverage-specific measurement warnings.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub measurement_warnings: Vec<String>,
    /// Same file served under 2+ different URLs (#551), sorted by `bytes`
    /// descending (most wasteful first). Purely advisory — never affects the
    /// performance score.
    #[serde(default)]
    pub duplicate_assets: Vec<DuplicateAssetGroup>,
}

/// Enable JS and CSS coverage collection **before** the page navigates.
///
/// Must be called after `prepare_vitals_collection` and before `browser.navigate()`.
/// Errors are non-fatal: the pipeline logs a warning and omits coverage from results.
pub async fn prepare_coverage_collection(page: &Page) -> Result<()> {
    // Enable the JS Profiler domain
    page.execute(ProfilerEnableParams::default())
        .await
        .map_err(|e| AuditError::CdpError(format!("Profiler.enable failed: {e}")))?;

    // Start precise JS coverage (block-level granularity, no call counts needed)
    page.execute(
        StartPreciseCoverageParams::builder()
            .call_count(false)
            .detailed(true)
            .allow_triggered_updates(false)
            .build(),
    )
    .await
    .map_err(|e| AuditError::CdpError(format!("Profiler.startPreciseCoverage failed: {e}")))?;

    // Start CSS rule usage tracking
    page.execute(StartRuleUsageTrackingParams::default())
        .await
        .map_err(|e| AuditError::CdpError(format!("CSS.startRuleUsageTracking failed: {e}")))?;

    // `CSS.enable` requires the DOM domain to already be enabled (per CDP
    // spec — confirmed live: it otherwise fails with "DOM agent needs to be
    // enabled first"). Nothing else in this codebase issues a raw
    // `DOM.enable` (AXTree extraction goes through chromiumoxide's
    // higher-level `Page` API, which doesn't need it), so this is a new,
    // explicit prerequisite just for the CSS agent. Best-effort: if it
    // fails, `CSS.enable` below will fail too and duplicate-CSS detection
    // is silently skipped.
    if let Err(e) = page.execute(DomEnableParams::default()).await {
        warn!("DOM.enable failed, skipping duplicate CSS detection: {e}");
    } else if let Err(e) = page.execute(CssEnableParams::default()).await {
        warn!("CSS.enable failed, skipping duplicate CSS detection: {e}");
    } else {
        // Register the `styleSheetAdded`/`styleSheetRemoved` listeners
        // *before* navigation, so `take_coverage_results` can later resolve
        // `style_sheet_id -> URL` for duplicate-CSS detection (#551) and skip
        // sheets the page removed again. Best-effort/non-fatal: duplicate CSS
        // detection is silently skipped if this fails.
        let streams = match (
            page.event_listener::<EventStyleSheetAdded>().await,
            page.event_listener::<EventStyleSheetRemoved>().await,
        ) {
            (Ok(added), Ok(removed)) => Some(StylesheetStreams { added, removed }),
            (Err(e), _) | (_, Err(e)) => {
                warn!("CSS stylesheet listener setup failed: {e}");
                None
            }
        };
        if let Some(streams) = streams {
            stylesheet_listeners()
                .lock()
                .await
                .insert(page.target_id().clone(), streams);
        }
    }

    // Enable the Debugger domain *before* navigation too (#551), and
    // register a `scriptParsed` listener right after — mirroring the CSS
    // stylesheet listener above. `Debugger.enable`'s documented "retroactively
    // backfills already-parsed scripts" behavior turned out, live, not to make
    // every backfilled script's source reliably retrievable via
    // `Debugger.getScriptSource` right away (confirmed against a real site:
    // "No script for id" for effectively every script, not just short-lived
    // ones). `take_coverage_results` instead drains this listener's buffered
    // events to know which script ids the Debugger domain has actually
    // registered before attempting `getScriptSource`, avoiding calls we
    // already know will fail. Best-effort/non-fatal: duplicate JS detection
    // is silently skipped if either step fails.
    match page.execute(DebuggerEnableParams::default()).await {
        Ok(_) => match page.event_listener::<EventScriptParsed>().await {
            Ok(stream) => {
                script_parsed_listeners()
                    .lock()
                    .await
                    .insert(page.target_id().clone(), stream);
            }
            Err(e) => warn!("Debugger.scriptParsed listener setup failed: {e}"),
        },
        Err(e) => warn!("Debugger.enable failed, skipping duplicate JS detection: {e}"),
    }

    Ok(())
}

/// Read JS and CSS coverage results **after** the page has loaded.
/// JS coverage via `Profiler.takePreciseCoverage`.
pub(crate) static JS_COVERAGE: crate::audit::collection::CollectorSpec =
    crate::audit::collection::CollectorSpec {
        id: "performance.js_coverage",
        module: "performance",
        method: crate::audit::collection::CollectionMethod::Cdp,
        method_version: 1,
        scoring: false,
    };

/// CSS rule usage via `CSS.stopRuleUsageTracking`.
pub(crate) static CSS_COVERAGE: crate::audit::collection::CollectorSpec =
    crate::audit::collection::CollectorSpec {
        id: "performance.css_coverage",
        module: "performance",
        method: crate::audit::collection::CollectionMethod::Cdp,
        method_version: 1,
        scoring: false,
    };

/// JS coverage: no answer from the profiler is a failure, an answer without
/// script bytes a valid empty measurement.
fn js_coverage_state(
    js_measured: bool,
    total_bytes: u64,
) -> crate::audit::collection::Collected<()> {
    use crate::audit::collection::{reason, Collected};
    if !js_measured {
        Collected::Failed(reason::CDP_COMMAND_FAILED)
    } else if total_bytes == 0 {
        Collected::Empty(())
    } else {
        Collected::Measured(())
    }
}

/// CSS coverage from its `measurement` label: rule tracking that never
/// answered is a failure, tracking without countable rules empty.
fn css_coverage_state(measurement: &str, tracked: bool) -> crate::audit::collection::Collected<()> {
    use crate::audit::collection::{reason, Collected};
    match measurement {
        "measured" => Collected::Measured(()),
        "partial" => Collected::Partial {
            value: (),
            gaps: vec!["unreadable_stylesheet"],
        },
        _ if tracked => Collected::Empty(()),
        _ => Collected::Failed(reason::CDP_COMMAND_FAILED),
    }
}

/// Counter-cases of [`JS_COVERAGE`] and [`CSS_COVERAGE`] (#769): a profiler
/// without an answer is no "100 % used".
#[cfg(test)]
pub(crate) fn counter_cases() {
    use crate::audit::collection::CollectionState;
    assert_eq!(js_coverage_state(false, 0).state(), CollectionState::Failed);
    assert_eq!(js_coverage_state(true, 0).state(), CollectionState::Empty);
    assert_eq!(
        js_coverage_state(true, 10).state(),
        CollectionState::Measured
    );
    assert_eq!(
        css_coverage_state("not_available", false).state(),
        CollectionState::Failed
    );
    assert_eq!(
        css_coverage_state("not_available", true).state(),
        CollectionState::Empty
    );
    assert_eq!(
        css_coverage_state("partial", true).state(),
        CollectionState::Partial
    );
}

pub async fn take_coverage_results(
    page: &Page,
    log: &crate::audit::collection::CollectionLog,
) -> Result<CoverageAnalysis> {
    info!("Taking JS and CSS coverage results...");

    // Re-issue `Debugger.enable` defensively — `prepare_coverage_collection`
    // already enables it before navigation (see its own comment for why
    // that timing matters), this is just idempotent insurance in case that
    // earlier call didn't run for some reason. Non-fatal: duplicate JS
    // detection is silently skipped if this fails.
    let debugger_enabled = match page.execute(DebuggerEnableParams::default()).await {
        Ok(_) => true,
        Err(e) => {
            warn!("Debugger.enable failed, skipping duplicate JS detection: {e}");
            false
        }
    };

    // Drain the `scriptParsed` listener registered in
    // `prepare_coverage_collection` to find out which script ids the
    // Debugger domain has actually registered a retrievable source for
    // (#551) — see that function's comment for why this gate exists.
    let mut parsed_script_ids: HashSet<ScriptId> = HashSet::new();
    if let Some(mut stream) = script_parsed_listeners()
        .lock()
        .await
        .remove(page.target_id())
    {
        while let Some(Some(event)) = stream.next().now_or_never() {
            parsed_script_ids.insert(event.script_id.clone());
            if parsed_script_ids.len() >= MAX_DUPLICATE_SCRIPT_EVENTS {
                break;
            }
        }
    }

    // ── JS coverage ───────────────────────────────────────────────────────────
    // A failed take is no coverage — not "every byte used" (#769).
    let (js_coverage, js_measured) = match page.execute(TakePreciseCoverageParams::default()).await
    {
        Ok(resp) => (resp.result.result, true),
        Err(e) => {
            warn!("Profiler.takePreciseCoverage failed: {e}");
            (vec![], false)
        }
    };

    let mut scripts: Vec<ScriptCoverageEntry> = Vec::new();
    let mut total_bytes: u64 = 0;
    let mut total_unused_bytes: u64 = 0;
    // Candidates for duplicate-content detection (#551), collected alongside
    // the existing unused-bytes computation so the (already filtered) script
    // list isn't walked a second time.
    let mut js_duplicate_candidates: Vec<(ScriptId, String)> = Vec::new();

    for script in &js_coverage {
        let url = script.url.as_str();

        // Skip internal Chrome scripts, data URIs, and extensions
        if url.is_empty()
            || url.starts_with("chrome-extension://")
            || url.starts_with("data:")
            || url.contains("extensions::")
        {
            continue;
        }

        let ranges = &script.functions;
        if ranges.is_empty() {
            continue;
        }

        // Determine covered byte ranges from all function ranges
        let mut covered_ranges: Vec<(i64, i64)> = Vec::new();
        let mut script_end: i64 = 0;
        for func in ranges {
            for range in &func.ranges {
                if range.count > 0 {
                    covered_ranges.push((range.start_offset, range.end_offset));
                }
                if range.end_offset > script_end {
                    script_end = range.end_offset;
                }
            }
        }

        if script_end <= 0 {
            continue;
        }

        // Merge covered ranges and compute used bytes
        covered_ranges.sort_by_key(|r| r.0);
        let mut used_bytes: i64 = 0;
        let mut cursor = 0i64;
        for (start, end) in covered_ranges {
            let start = start.max(cursor);
            if end > start {
                used_bytes += end - start;
                cursor = end;
            }
        }

        let script_bytes = script_end as u64;
        let unused = script_bytes.saturating_sub(used_bytes as u64);
        let used_pct = if script_bytes > 0 {
            (used_bytes as f64 / script_bytes as f64 * 100.0).clamp(0.0, 100.0)
        } else {
            0.0
        };

        total_bytes += script_bytes;
        total_unused_bytes += unused;

        // Only include scripts with meaningful size
        if script_bytes >= 1024 {
            scripts.push(ScriptCoverageEntry {
                url: truncate(url, 120),
                total_bytes: script_bytes,
                unused_bytes: unused,
                used_pct,
            });
            js_duplicate_candidates.push((script.script_id.clone(), url.to_string()));
        }
    }

    scripts.sort_by_key(|s| std::cmp::Reverse(s.unused_bytes));

    // Fetch full source text for duplicate-content detection (#551) — these
    // scripts are already resident in the browser's memory, so this is a
    // local CDP round-trip, not a new network download. Only attempt this
    // for script ids `parsed_script_ids` actually confirmed via a
    // `scriptParsed` event — calling `getScriptSource` for an unconfirmed id
    // reliably fails ("No script for id"), so skip those quietly rather than
    // firing (and warning on) calls already known to fail.
    let mut js_duplicate_entries: Vec<(String, String, String)> = Vec::new();
    if debugger_enabled {
        for (script_id, url) in &js_duplicate_candidates {
            if !parsed_script_ids.contains(script_id) {
                continue;
            }
            match page
                .execute(GetScriptSourceParams::new(script_id.clone()))
                .await
            {
                Ok(resp) => js_duplicate_entries.push((
                    script_id.inner().clone(),
                    url.clone(),
                    resp.result.script_source,
                )),
                Err(e) => {
                    tracing::debug!("Debugger.getScriptSource failed for {url}: {e}")
                }
            }
        }
    }

    let overall_used_pct = (total_bytes > 0).then(|| {
        ((total_bytes - total_unused_bytes) as f64 / total_bytes as f64 * 100.0).clamp(0.0, 100.0)
    });
    log.record(&JS_COVERAGE, js_coverage_state(js_measured, total_bytes));

    let unused_js = UnusedJsAnalysis {
        scripts,
        total_bytes,
        unused_bytes: total_unused_bytes,
        used_pct: overall_used_pct,
    };

    // ── CSS coverage ──────────────────────────────────────────────────────────
    // `stopRuleUsageTracking` returns the rules *used* since tracking began —
    // not every rule — so its length is no denominator: it made 316 of 316
    // (100 %) on inros-lackner, and nearly every page 100 % (plan 78). The
    // denominator is the style rules in each stylesheet the page added, read
    // back with `getStyleSheetText`.
    let css_coverage = match page.execute(StopRuleUsageTrackingParams::default()).await {
        Ok(resp) => Some(resp.result.rule_usage),
        Err(e) => {
            warn!("CSS.stopRuleUsageTracking failed: {e}");
            None
        }
    };

    let mut stylesheets: Vec<(StyleSheetId, String)> = Vec::new();
    if let Some(mut streams) = stylesheet_listeners().lock().await.remove(page.target_id()) {
        let mut removed: HashSet<StyleSheetId> = HashSet::new();
        while let Some(Some(event)) = streams.removed.next().now_or_never() {
            removed.insert(event.style_sheet_id.clone());
        }
        let mut seen: HashSet<StyleSheetId> = HashSet::new();
        while let Some(Some(event)) = streams.added.next().now_or_never() {
            let url = event.header.source_url.clone();
            if url.starts_with("chrome-extension://")
                || removed.contains(&event.header.style_sheet_id)
            {
                continue;
            }
            if seen.insert(event.header.style_sheet_id.clone()) {
                stylesheets.push((event.header.style_sheet_id.clone(), url));
            }
            if stylesheets.len() >= MAX_DUPLICATE_STYLESHEET_EVENTS {
                break;
            }
        }
    }

    let mut css_duplicate_entries: Vec<(String, String, String)> = Vec::new();
    let mut read_sheets: HashSet<StyleSheetId> = HashSet::new();
    let mut total_rules = 0u32;
    for (style_sheet_id, url) in &stylesheets {
        match page
            .execute(GetStyleSheetTextParams::new(style_sheet_id.clone()))
            .await
        {
            Ok(resp) => {
                total_rules += count_style_rules(&resp.result.text);
                read_sheets.insert(style_sheet_id.clone());
                // Duplicate CSS detection (#551): only files with a URL.
                if !url.is_empty() && !url.starts_with("data:") {
                    css_duplicate_entries.push((
                        style_sheet_id.inner().clone(),
                        url.clone(),
                        resp.result.text,
                    ));
                }
            }
            Err(e) => warn!("CSS.getStyleSheetText failed for {url}: {e}"),
        }
    }
    let used_rules = css_coverage
        .iter()
        .flatten()
        .filter(|r| r.used && read_sheets.contains(&r.style_sheet_id))
        .count() as u32;
    let complete = read_sheets.len() == stylesheets.len();
    let (css_used_pct, css_measurement, mut measurement_warnings) =
        if css_coverage.is_none() || total_rules == 0 {
            (
                None,
                "not_available".to_string(),
                vec!["css_coverage_unavailable".to_string()],
            )
        } else {
            let pct = (used_rules as f64 / total_rules as f64 * 100.0).clamp(0.0, 100.0);
            if complete {
                (Some(pct), "measured".to_string(), vec![])
            } else {
                (
                    Some(pct),
                    "partial".to_string(),
                    vec!["css_coverage_partial".to_string()],
                )
            }
        };

    log.record(
        &CSS_COVERAGE,
        css_coverage_state(&css_measurement, css_coverage.is_some()),
    );
    if !js_measured {
        measurement_warnings.push("js_coverage_unavailable".to_string());
    }

    let unused_css = UnusedCssAnalysis {
        total_rules,
        used_rules: used_rules.min(total_rules),
        used_pct: css_used_pct,
        measurement: css_measurement,
    };

    let mut duplicate_assets = group_duplicate_assets("script", js_duplicate_entries);
    duplicate_assets.extend(group_duplicate_assets("stylesheet", css_duplicate_entries));
    duplicate_assets.sort_by_key(|g| std::cmp::Reverse(g.bytes));

    info!(
        "Coverage: JS {} used ({:.1} KB unused), CSS {} rules used ({}/{})",
        overall_used_pct
            .map(|v| format!("{v:.0}%"))
            .unwrap_or_else(|| "not available".to_string()),
        total_unused_bytes as f64 / 1024.0,
        css_used_pct
            .map(|v| format!("{v:.0}%"))
            .unwrap_or_else(|| "not available".to_string()),
        used_rules,
        total_rules,
    );

    Ok(CoverageAnalysis {
        unused_js,
        unused_css,
        measurement_warnings,
        duplicate_assets,
    })
}

/// Style rules in a stylesheet, the unit Chrome reports usage for: every
/// style rule, also inside `@media`/`@supports`/`@layer` blocks and nested
/// rules; `@keyframes`, `@font-face` and other at-rules are not style rules.
fn count_style_rules(css: &str) -> u32 {
    fn count(rules: &[stylesheet_parse::Rule]) -> u32 {
        rules
            .iter()
            .map(|rule| match rule {
                stylesheet_parse::Rule::Style(style) => 1 + count(&style.nested),
                stylesheet_parse::Rule::Media(media) => count(&media.rules),
                stylesheet_parse::Rule::Group(group) => count(&group.rules),
                _ => 0,
            })
            .sum()
    }
    count(&stylesheet_parse::parse_stylesheet(css).rules)
}

/// Groups fetched source texts by content hash, returning one
/// `DuplicateAssetGroup` per hash shared by 2+ distinct URLs (#551).
/// `entries` is `(id, url, text)` — `id` dedupes the same script/stylesheet
/// id appearing more than once in the raw coverage list (e.g. a script that
/// ran more than once) before grouping by URL, so the same URL is never
/// counted twice. Pure/no I/O so it's testable without a CDP connection.
fn group_duplicate_assets(
    kind: &str,
    entries: Vec<(String, String, String)>,
) -> Vec<DuplicateAssetGroup> {
    let mut by_id: HashMap<String, (String, String)> = HashMap::new();
    for (id, url, text) in entries {
        by_id.entry(id).or_insert((url, text));
    }

    let mut by_url: HashMap<String, String> = HashMap::new();
    for (url, text) in by_id.into_values() {
        by_url.entry(url).or_insert(text);
    }

    let mut groups: HashMap<String, (Vec<String>, u64)> = HashMap::new();
    for (url, text) in by_url {
        if text.len() < MIN_DUPLICATE_ASSET_BYTES {
            continue;
        }
        let hash = sha256_hex(&text);
        let group = groups
            .entry(hash)
            .or_insert_with(|| (Vec::new(), text.len() as u64));
        group.0.push(url);
    }

    let mut result: Vec<DuplicateAssetGroup> = groups
        .into_values()
        .filter(|(urls, _)| urls.len() >= 2)
        .map(|(mut urls, bytes)| {
            urls.sort();
            DuplicateAssetGroup {
                kind: kind.to_string(),
                urls,
                bytes,
            }
        })
        .collect();

    result.sort_by_key(|g| std::cmp::Reverse(g.bytes));
    result
}

fn sha256_hex(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {

    /// Plan 78: the denominator counts style rules wherever they sit, and
    /// nothing that is no style rule.
    #[test]
    fn count_style_rules_counts_nested_and_grouped_rules_only() {
        let css = "a{color:red} b,c{color:blue} @media (min-width:1px){d{x:1} e{x:2}} \
                   @supports (display:grid){f{x:1}} @keyframes k{from{x:1} to{x:2}} \
                   @font-face{font-family:x} g{x:1; &:hover{x:2}}";
        // a, "b,c" (one rule), d, e, f, g and its nested &:hover.
        assert_eq!(count_style_rules(css), 7);
        assert_eq!(count_style_rules(""), 0);
    }

    use super::*;

    #[test]
    fn test_unused_js_zero_bytes() {
        let analysis = UnusedJsAnalysis {
            scripts: vec![],
            total_bytes: 0,
            unused_bytes: 0,
            used_pct: None,
        };
        assert_eq!(analysis.used_pct, None);
        assert!(analysis.scripts.is_empty());
    }

    #[test]
    fn test_unused_css_all_used() {
        let analysis = UnusedCssAnalysis {
            total_rules: 50,
            used_rules: 50,
            used_pct: Some(100.0),
            measurement: "measured".to_string(),
        };
        assert_eq!(analysis.used_pct, Some(100.0));
        assert_eq!(analysis.total_rules, analysis.used_rules);
    }

    #[test]
    fn test_unused_css_unavailable_is_not_zero_percent() {
        let analysis = UnusedCssAnalysis {
            total_rules: 0,
            used_rules: 0,
            used_pct: None,
            measurement: "not_available".to_string(),
        };
        assert_eq!(analysis.used_pct, None);
        assert_eq!(analysis.measurement, "not_available");
    }

    #[test]
    fn test_truncate() {
        let long = "a".repeat(200);
        let result = truncate(&long, 120);
        assert!(result.len() <= 123);
    }

    fn big_text(marker: &str) -> String {
        // Above MIN_DUPLICATE_ASSET_BYTES so it isn't filtered out as trivial.
        format!("{marker}{}", "x".repeat(MIN_DUPLICATE_ASSET_BYTES))
    }

    #[test]
    fn test_group_duplicate_assets_finds_two_urls_same_content() {
        let text = big_text("shared");
        let entries = vec![
            (
                "id-1".to_string(),
                "https://a.example/app.js".to_string(),
                text.clone(),
            ),
            (
                "id-2".to_string(),
                "https://a.example/vendor/app.js".to_string(),
                text.clone(),
            ),
        ];
        let groups = group_duplicate_assets("script", entries);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].kind, "script");
        assert_eq!(groups[0].bytes, text.len() as u64);
        assert_eq!(
            groups[0].urls,
            vec![
                "https://a.example/app.js".to_string(),
                "https://a.example/vendor/app.js".to_string(),
            ]
        );
    }

    #[test]
    fn test_group_duplicate_assets_single_url_is_not_a_duplicate() {
        let entries = vec![(
            "id-1".to_string(),
            "https://a.example/app.js".to_string(),
            big_text("solo"),
        )];
        assert!(group_duplicate_assets("script", entries).is_empty());
    }

    #[test]
    fn test_group_duplicate_assets_different_content_is_not_grouped() {
        let entries = vec![
            (
                "id-1".to_string(),
                "https://a.example/a.js".to_string(),
                big_text("one"),
            ),
            (
                "id-2".to_string(),
                "https://a.example/b.js".to_string(),
                big_text("two"),
            ),
        ];
        assert!(group_duplicate_assets("script", entries).is_empty());
    }

    #[test]
    fn test_group_duplicate_assets_below_size_threshold_is_ignored() {
        let text = "tiny".to_string();
        let entries = vec![
            (
                "id-1".to_string(),
                "https://a.example/a.css".to_string(),
                text.clone(),
            ),
            (
                "id-2".to_string(),
                "https://a.example/b.css".to_string(),
                text,
            ),
        ];
        assert!(group_duplicate_assets("stylesheet", entries).is_empty());
    }

    #[test]
    fn test_group_duplicate_assets_same_id_repeated_counts_once() {
        // Same script id appearing twice in the raw coverage list (e.g. it ran
        // more than once) must not turn a single URL into a fake 2-URL group.
        let text = big_text("repeat");
        let entries = vec![
            (
                "id-1".to_string(),
                "https://a.example/app.js".to_string(),
                text.clone(),
            ),
            (
                "id-1".to_string(),
                "https://a.example/app.js".to_string(),
                text,
            ),
        ];
        assert!(group_duplicate_assets("script", entries).is_empty());
    }

    #[test]
    fn test_group_duplicate_assets_sorted_by_bytes_descending() {
        let small = big_text("small");
        let large = format!("{small}{}", "y".repeat(2048));
        let entries = vec![
            (
                "id-1".to_string(),
                "https://a.example/small-a.js".to_string(),
                small.clone(),
            ),
            (
                "id-2".to_string(),
                "https://a.example/small-b.js".to_string(),
                small,
            ),
            (
                "id-3".to_string(),
                "https://a.example/large-a.js".to_string(),
                large.clone(),
            ),
            (
                "id-4".to_string(),
                "https://a.example/large-b.js".to_string(),
                large,
            ),
        ];
        let groups = group_duplicate_assets("script", entries);
        assert_eq!(groups.len(), 2);
        assert!(groups[0].bytes > groups[1].bytes);
    }
}
