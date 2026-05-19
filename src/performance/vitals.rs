//! Core Web Vitals extraction via CDP
//!
//! Collects LCP, FCP, CLS, TBT, TTFB and other performance metrics.

use chromiumoxide::cdp::browser_protocol::performance::GetMetricsParams;
use chromiumoxide::Page;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::error::{AuditError, Result};

/// Source element attribution for a single CLS layout-shift entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClsSource {
    /// Simplified node description, e.g. "DIV#banner" or "IMG"
    pub node: String,
    /// Bounding rect before the shift (may be absent on first paint)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_rect: Option<ShiftRect>,
    /// Bounding rect after the shift
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_rect: Option<ShiftRect>,
}

/// Axis-aligned bounding rect (viewport-relative pixels).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShiftRect {
    pub top: f64,
    pub left: f64,
    pub bottom: f64,
    pub right: f64,
}

/// A single layout-shift entry with per-element attribution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClsShift {
    /// Score contribution of this individual shift
    pub value: f64,
    /// When the shift occurred (ms from navigation start)
    pub start_time_ms: f64,
    /// Elements that moved (may be empty when browser does not expose attribution)
    pub sources: Vec<ClsSource>,
}

/// Core Web Vitals and performance metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WebVitals {
    /// Largest Contentful Paint (ms) - target ≤2500
    pub lcp: Option<VitalMetric>,
    /// First Contentful Paint (ms) - target ≤1800
    pub fcp: Option<VitalMetric>,
    /// Cumulative Layout Shift - target ≤0.1
    pub cls: Option<VitalMetric>,
    /// Per-shift CLS attribution (#135)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cls_attribution: Vec<ClsShift>,
    /// Time to First Byte (ms)
    pub ttfb: Option<VitalMetric>,
    /// First Input Delay (ms) - deprecated; always null in CDP/headless audits
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fid: Option<VitalMetric>,
    /// Total Blocking Time (ms)
    pub tbt: Option<VitalMetric>,
    /// Speed Index — heuristic proxy: 0.35·FCP + 0.65·LCP (#137)
    pub speed_index: Option<VitalMetric>,
    /// Time to Interactive — simplified: max(domInteractive, last long-task end) (#137)
    pub tti: Option<VitalMetric>,
    /// Interaction to Next Paint — only populated when interaction events fire during audit
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inp: Option<VitalMetric>,
    /// DOM Content Loaded (ms)
    pub dom_content_loaded: Option<f64>,
    /// Page Load Time (ms)
    pub load_time: Option<f64>,
    /// DOM Nodes count
    pub dom_nodes: Option<i64>,
    /// JavaScript Heap Size (bytes)
    pub js_heap_size: Option<i64>,
}

/// Individual vital metric with value and rating
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VitalMetric {
    /// The metric value
    pub value: f64,
    /// Rating: "good", "needs-improvement", "poor"
    pub rating: String,
    /// Target threshold for "good"
    pub target: f64,
}

impl VitalMetric {
    pub fn new(value: f64, good_threshold: f64, poor_threshold: f64) -> Self {
        let rating = if value <= good_threshold {
            "good"
        } else if value <= poor_threshold {
            "needs-improvement"
        } else {
            "poor"
        };

        Self {
            value,
            rating: rating.to_string(),
            target: good_threshold,
        }
    }

    pub fn is_good(&self) -> bool {
        self.rating == "good"
    }
}

impl WebVitals {
    /// Count how many vitals pass the "good" threshold
    pub fn good_count(&self) -> usize {
        [&self.lcp, &self.fcp, &self.cls, &self.tbt]
            .iter()
            .filter(|v| v.as_ref().map(|m| m.is_good()).unwrap_or(false))
            .count()
    }

    /// Count how many vitals are available
    pub fn available_count(&self) -> usize {
        [&self.lcp, &self.fcp, &self.cls, &self.tbt]
            .iter()
            .filter(|v| v.is_some())
            .count()
    }
}

/// Inject PerformanceObserver script before navigation so LCP, TBT, and CLS
/// entries are captured from the very first paint (Lighthouse-style).
///
/// Must be called on a fresh `Page` BEFORE `browser.navigate()`. The observers
/// store results in `window.__ams_lcp`, `window.__ams_tbt`, `window.__ams_cls`.
pub async fn prepare_vitals_collection(page: &Page) -> Result<()> {
    let script = r#"
(function() {
    // LCP observer — captures the largest contentful paint before load
    window.__ams_lcp = 0;
    try {
        var _lcpObs = new PerformanceObserver(function(list) {
            var entries = list.getEntries();
            if (entries.length > 0) {
                window.__ams_lcp = entries[entries.length - 1].startTime;
            }
        });
        _lcpObs.observe({ type: 'largest-contentful-paint', buffered: true });
        window.__ams_lcp_obs = _lcpObs;
    } catch(e) {}

    // TBT + TTI via Long Tasks — sum blocking time; track last task end for TTI
    window.__ams_tbt = 0;
    window.__ams_last_task_end = 0;
    try {
        new PerformanceObserver(function(list) {
            var entries = list.getEntries();
            for (var i = 0; i < entries.length; i++) {
                var e = entries[i];
                if (e.duration > 50) {
                    window.__ams_tbt += e.duration - 50;
                }
                var end = e.startTime + e.duration;
                if (end > window.__ams_last_task_end) {
                    window.__ams_last_task_end = end;
                }
            }
        }).observe({ type: 'longtask', buffered: true });
    } catch(e) {}

    // CLS — accumulate total and capture per-shift attribution (#135)
    window.__ams_cls = 0;
    window.__ams_cls_shifts = [];
    try {
        new PerformanceObserver(function(list) {
            var entries = list.getEntries();
            for (var i = 0; i < entries.length; i++) {
                var e = entries[i];
                if (!e.hadRecentInput) {
                    window.__ams_cls += e.value;
                    var sources = [];
                    try {
                        if (e.sources) {
                            for (var j = 0; j < e.sources.length; j++) {
                                var s = e.sources[j];
                                var nodeDesc = 'unknown';
                                try {
                                    if (s.node) {
                                        nodeDesc = s.node.nodeName;
                                        if (s.node.id) nodeDesc += '#' + s.node.id;
                                        else if (s.node.className && typeof s.node.className === 'string') {
                                            var cls = s.node.className.trim().split(/\s+/)[0];
                                            if (cls) nodeDesc += '.' + cls;
                                        }
                                    }
                                } catch(_) {}
                                sources.push({
                                    node: nodeDesc,
                                    previousRect: s.previousRect ? {top: s.previousRect.top, left: s.previousRect.left, bottom: s.previousRect.bottom, right: s.previousRect.right} : null,
                                    currentRect: s.currentRect ? {top: s.currentRect.top, left: s.currentRect.left, bottom: s.currentRect.bottom, right: s.currentRect.right} : null
                                });
                            }
                        }
                    } catch(_) {}
                    window.__ams_cls_shifts.push({value: e.value, startTime: e.startTime, sources: sources});
                }
            }
        }).observe({ type: 'layout-shift', buffered: true });
    } catch(e) {}

    // INP via event-timing (#134) — captures real interaction durations
    window.__ams_inp = 0;
    try {
        new PerformanceObserver(function(list) {
            var entries = list.getEntries();
            for (var i = 0; i < entries.length; i++) {
                var e = entries[i];
                if (e.duration > window.__ams_inp) {
                    window.__ams_inp = e.duration;
                }
            }
        }).observe({ type: 'event', durationThreshold: 16, buffered: true });
    } catch(e) {}
    // Also observe first-input for environments that don't support event-timing
    try {
        new PerformanceObserver(function(list) {
            var entries = list.getEntries();
            for (var i = 0; i < entries.length; i++) {
                var e = entries[i];
                var delay = e.processingStart - e.startTime;
                if (delay > window.__ams_inp) window.__ams_inp = delay;
            }
        }).observe({ type: 'first-input', buffered: true });
    } catch(e) {}
})();
"#;

    page.evaluate_on_new_document(script)
        .await
        .map_err(|e| AuditError::CdpError(format!("Failed to inject vitals observer: {}", e)))?;

    Ok(())
}

/// Read the pre-injected globals after navigation to collect LCP, TBT, CLS, TTI, INP.
///
/// Flushes and disconnects the LCP observer to capture any buffered entries.
async fn collect_preinjected_vitals(page: &Page) -> Result<PreinjectedVitals> {
    let js = r#"
(function() {
    // Finalize LCP — flush and disconnect observer
    var lcp = 0;
    try {
        if (window.__ams_lcp_obs) {
            var recs = window.__ams_lcp_obs.takeRecords();
            if (recs.length > 0) {
                window.__ams_lcp = recs[recs.length - 1].startTime;
            }
            window.__ams_lcp_obs.disconnect();
        }
        lcp = window.__ams_lcp || 0;
    } catch(e) {}

    return JSON.stringify({
        lcp: lcp,
        tbt: window.__ams_tbt || 0,
        cls: window.__ams_cls || 0,
        lastTaskEnd: window.__ams_last_task_end || 0,
        clsShifts: window.__ams_cls_shifts || [],
        inpDuration: window.__ams_inp || 0
    });
})()
"#;

    let result = page
        .evaluate(js)
        .await
        .map_err(|e| AuditError::CdpError(format!("Preinjected vitals read failed: {}", e)))?;

    let json_str = result.value().and_then(|v| v.as_str()).unwrap_or("{}");
    let parsed: serde_json::Value = serde_json::from_str(json_str).unwrap_or_default();

    let cls_shifts = parse_cls_shifts(&parsed["clsShifts"]);

    Ok(PreinjectedVitals {
        lcp: parsed["lcp"].as_f64().unwrap_or(0.0),
        tbt: parsed["tbt"].as_f64().unwrap_or(0.0),
        cls: parsed["cls"].as_f64().unwrap_or(0.0),
        last_task_end: parsed["lastTaskEnd"].as_f64().unwrap_or(0.0),
        cls_shifts,
        inp_duration: parsed["inpDuration"].as_f64().unwrap_or(0.0),
    })
}

/// Parse CLS shift entries from the JSON value returned by the browser.
fn parse_cls_shifts(val: &serde_json::Value) -> Vec<ClsShift> {
    val.as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|entry| {
                    let value = entry["value"].as_f64()?;
                    let start_time_ms = entry["startTime"].as_f64().unwrap_or(0.0);
                    let sources = entry["sources"]
                        .as_array()
                        .map(|srcs| {
                            srcs.iter()
                                .map(|s| {
                                    let node = s["node"].as_str().unwrap_or("unknown").to_string();
                                    let previous_rect = parse_shift_rect(&s["previousRect"]);
                                    let current_rect = parse_shift_rect(&s["currentRect"]);
                                    ClsSource {
                                        node,
                                        previous_rect,
                                        current_rect,
                                    }
                                })
                                .collect()
                        })
                        .unwrap_or_default();
                    Some(ClsShift {
                        value,
                        start_time_ms,
                        sources,
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn parse_shift_rect(val: &serde_json::Value) -> Option<ShiftRect> {
    if val.is_null() {
        return None;
    }
    Some(ShiftRect {
        top: val["top"].as_f64().unwrap_or(0.0),
        left: val["left"].as_f64().unwrap_or(0.0),
        bottom: val["bottom"].as_f64().unwrap_or(0.0),
        right: val["right"].as_f64().unwrap_or(0.0),
    })
}

/// Values collected by the pre-injected PerformanceObserver script.
struct PreinjectedVitals {
    lcp: f64,
    tbt: f64,
    cls: f64,
    /// End time of the last long task (ms from navigation start); 0 = no long tasks
    last_task_end: f64,
    /// Per-shift CLS entries (#135)
    cls_shifts: Vec<ClsShift>,
    /// Max event/interaction duration captured by event-timing or first-input (#134)
    inp_duration: f64,
}

/// Read the current LCP value from the pre-injected global.
///
/// Returns `None` if no LCP was captured yet or the script was not injected.
pub async fn finalize_lcp(page: &Page) -> Result<Option<f64>> {
    let result = page
        .evaluate("window.__ams_lcp || 0")
        .await
        .map_err(|e| AuditError::CdpError(format!("LCP read failed: {}", e)))?;

    let val = result.value().and_then(|v| v.as_f64()).unwrap_or(0.0);
    Ok(if val > 0.0 { Some(val) } else { None })
}

/// Wait for LCP and TBT to stabilize before reading vitals.
///
/// **Phase 1 – LCP**: polls `window.__ams_lcp` every 250 ms (initial 300 ms wait,
/// up to ~3 s) until an LCP entry is captured.
///
/// **Phase 2 – TBT settle**: JS bundles execute *after* LCP fires, so Long Tasks
/// are not yet visible when LCP > 0. Polls `window.__ams_tbt` every 200 ms and
/// breaks only when the value is stable for three consecutive reads (≥ 600 ms
/// unchanged). Max 5 s — matches Lighthouse's quiet-window heuristic.
async fn wait_for_vitals_stable(page: &Page) {
    // Phase 1: wait for first LCP entry
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    for _ in 0..11u8 {
        let lcp = page
            .evaluate("window.__ams_lcp || 0")
            .await
            .ok()
            .and_then(|r| r.value().and_then(|v| v.as_f64()))
            .unwrap_or(0.0);
        if lcp > 0.0 {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
    }

    // Phase 2: wait for TBT to stop growing (Long Tasks settle after LCP)
    let mut last_tbt = -1.0_f64;
    let mut stable_ticks: u8 = 0;
    for _ in 0..25u8 {
        // max 5 s (25 × 200 ms)
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        let tbt = page
            .evaluate("window.__ams_tbt || 0")
            .await
            .ok()
            .and_then(|r| r.value().and_then(|v| v.as_f64()))
            .unwrap_or(0.0);
        if (tbt - last_tbt).abs() < 1.0 {
            stable_ticks += 1;
            if stable_ticks >= 3 {
                debug!(
                    "TBT stable at {:.0}ms after {} settle ticks",
                    tbt, stable_ticks
                );
                break;
            }
        } else {
            stable_ticks = 0;
        }
        last_tbt = tbt;
    }
}

/// Extract Core Web Vitals from a page
///
/// # Arguments
/// * `page` - The chromiumoxide Page to extract metrics from
///
/// # Returns
/// * `Ok(WebVitals)` - The extracted performance metrics
/// * `Err(AuditError)` - If extraction fails
pub async fn extract_web_vitals(page: &Page) -> Result<WebVitals> {
    info!("Extracting Core Web Vitals...");

    wait_for_vitals_stable(page).await;

    let mut vitals = WebVitals::default();

    // Read pre-injected observer globals first (Lighthouse-style, captures from first paint).
    // Falls back gracefully when prepare_vitals_collection was not called.
    let preinjected = collect_preinjected_vitals(page).await.unwrap_or_else(|e| {
        warn!(
            "Pre-injected vitals unavailable ({}), falling back to CDP",
            e
        );
        PreinjectedVitals {
            lcp: 0.0,
            tbt: 0.0,
            cls: 0.0,
            last_task_end: 0.0,
            cls_shifts: vec![],
            inp_duration: 0.0,
        }
    });
    let last_task_end = preinjected.last_task_end;

    // Apply pre-injected LCP and TBT when non-zero (0 = not captured / not injected)
    if preinjected.lcp > 0.0 && preinjected.lcp < 300_000.0 {
        vitals.lcp = Some(VitalMetric::new(preinjected.lcp, 2500.0, 4000.0));
        debug!("LCP (preinjected): {:.0}ms", preinjected.lcp);
    }
    // TBT: 0 ms is a valid perfect score (no long tasks), always include like CLS.
    vitals.tbt = Some(VitalMetric::new(preinjected.tbt, 200.0, 600.0));
    debug!("TBT (preinjected): {:.0}ms", preinjected.tbt);
    // CLS: 0.0 is a valid perfect score, always apply
    vitals.cls = Some(VitalMetric::new(preinjected.cls, 0.1, 0.25));
    debug!("CLS (preinjected): {:.4}", preinjected.cls);

    // CLS attribution (#135)
    vitals.cls_attribution = preinjected.cls_shifts;

    // INP (#134): event-timing or first-input duration; 0 = no interaction captured.
    // In headless audits interactions rarely fire, so 0 means "not measurable" and
    // we leave the field empty rather than reporting a misleading 0ms.
    if preinjected.inp_duration > 0.0 {
        vitals.inp = Some(VitalMetric::new(preinjected.inp_duration, 200.0, 500.0));
        debug!("INP (event-timing): {:.0}ms", preinjected.inp_duration);
    }

    // Get performance metrics via CDP for FCP, DCL, heap, node count
    let metrics_response = page
        .execute(GetMetricsParams::default())
        .await
        .map_err(|e| AuditError::CdpError(format!("Failed to get metrics: {}", e)))?;

    for metric in &metrics_response.metrics {
        let value = metric.value;
        match metric.name.as_str() {
            "FirstContentfulPaint" => {
                let ms = value * 1000.0;
                if ms > 0.0 && ms < 300_000.0 {
                    vitals.fcp = Some(VitalMetric::new(ms, 1800.0, 3000.0));
                    debug!("FCP (CDP): {:.0}ms", ms);
                }
            }
            "LargestContentfulPaint" if vitals.lcp.is_none() => {
                // Only use CDP LCP as fallback when pre-injected observer missed it
                let ms = value * 1000.0;
                if ms > 0.0 && ms < 300_000.0 {
                    vitals.lcp = Some(VitalMetric::new(ms, 2500.0, 4000.0));
                    debug!("LCP (CDP fallback): {:.0}ms", ms);
                }
            }
            "DomContentLoaded" => {
                let ms = value * 1000.0;
                if ms > 0.0 && ms < 300_000.0 {
                    vitals.dom_content_loaded = Some(ms);
                }
            }
            "JSHeapUsedSize" => {
                vitals.js_heap_size = Some(value as i64);
            }
            "Nodes" => {
                vitals.dom_nodes = Some(value as i64);
            }
            _ => {}
        }
    }

    // Extract additional metrics via JavaScript (FCP fallback, TTFB, load time)
    let js_metrics = extract_js_metrics(page).await?;

    if vitals.fcp.is_none() && js_metrics.fcp.is_some() {
        vitals.fcp = js_metrics.fcp;
    }
    if vitals.lcp.is_none() && js_metrics.lcp.is_some() {
        vitals.lcp = js_metrics.lcp;
    }
    if js_metrics.ttfb.is_some() {
        vitals.ttfb = js_metrics.ttfb;
    }
    if js_metrics.load_time.is_some() {
        vitals.load_time = js_metrics.load_time;
    }
    if vitals.dom_content_loaded.is_none() {
        if let Some(js_dcl) = js_metrics.dom_content_loaded {
            if js_dcl > 0.0 && js_dcl < 300_000.0 {
                vitals.dom_content_loaded = Some(js_dcl);
            }
        }
    }

    // TTI (#137): max(domInteractive, last_long_task_end).
    // domInteractive = domContentLoadedEventStart (navigation timing).
    // Thresholds from Lighthouse: good ≤3800ms, poor >7300ms.
    let dom_interactive_ms: f64 = page
        .evaluate("(function(){ var n = performance.getEntriesByType('navigation')[0]; return n ? n.domInteractive : 0; })()")
        .await
        .ok()
        .and_then(|r| r.value().and_then(|v| v.as_f64()))
        .unwrap_or(0.0);
    let tti_ms = dom_interactive_ms.max(last_task_end);
    if tti_ms > 0.0 && tti_ms < 300_000.0 {
        vitals.tti = Some(VitalMetric::new(tti_ms, 3800.0, 7300.0));
        debug!("TTI (approx): {:.0}ms", tti_ms);
    }

    // Speed Index (#137): heuristic proxy = 0.35·FCP + 0.65·LCP.
    // Real Speed Index requires video-frame analysis; this approximation
    // correlates well for typical content-heavy pages.
    // Thresholds from Lighthouse: good ≤3400ms, poor >5800ms.
    if let (Some(fcp), Some(lcp)) = (&vitals.fcp, &vitals.lcp) {
        let si = 0.35 * fcp.value + 0.65 * lcp.value;
        vitals.speed_index = Some(VitalMetric::new(si, 3400.0, 5800.0));
        debug!("Speed Index (heuristic): {:.0}ms", si);
    }

    info!(
        "Web Vitals extracted: LCP={:?}ms, FCP={:?}ms, CLS={:?}, TTI={:?}ms, INP={:?}ms",
        vitals.lcp.as_ref().map(|v| v.value as i64),
        vitals.fcp.as_ref().map(|v| v.value as i64),
        vitals.cls.as_ref().map(|v| v.value),
        vitals.tti.as_ref().map(|v| v.value as i64),
        vitals.inp.as_ref().map(|v| v.value as i64),
    );

    Ok(vitals)
}

/// Extract metrics via JavaScript Performance API
async fn extract_js_metrics(page: &Page) -> Result<WebVitals> {
    let js_code = r#"
    (() => {
        const result = {};

        // Navigation Timing
        const nav = performance.getEntriesByType('navigation')[0];
        if (nav) {
            result.ttfb = nav.responseStart - nav.requestStart;
            result.loadTime = nav.loadEventEnd - nav.startTime;
            result.domContentLoaded = nav.domContentLoadedEventEnd - nav.startTime;
        }

        // Paint Timing
        const paints = performance.getEntriesByType('paint');
        for (const paint of paints) {
            if (paint.name === 'first-contentful-paint') {
                result.fcp = paint.startTime;
            }
        }

        // LCP (if available)
        const lcpEntries = performance.getEntriesByType('largest-contentful-paint');
        if (lcpEntries.length > 0) {
            result.lcp = lcpEntries[lcpEntries.length - 1].startTime;
        }

        return JSON.stringify(result);
    })()
    "#;

    let js_result = page
        .evaluate(js_code)
        .await
        .map_err(|e| AuditError::CdpError(format!("JS metrics extraction failed: {}", e)))?;

    let json_str = js_result.value().and_then(|v| v.as_str()).unwrap_or("{}");

    let parsed: serde_json::Value = serde_json::from_str(json_str).unwrap_or_default();

    let mut vitals = WebVitals::default();

    if let Some(lcp) = parsed["lcp"].as_f64() {
        vitals.lcp = Some(VitalMetric::new(lcp, 2500.0, 4000.0));
    }
    if let Some(fcp) = parsed["fcp"].as_f64() {
        vitals.fcp = Some(VitalMetric::new(fcp, 1800.0, 3000.0));
    }
    if let Some(ttfb) = parsed["ttfb"].as_f64() {
        vitals.ttfb = Some(VitalMetric::new(ttfb, 800.0, 1800.0));
    }
    if let Some(load_time) = parsed["loadTime"].as_f64() {
        vitals.load_time = Some(load_time);
    }
    if let Some(dcl) = parsed["domContentLoaded"].as_f64() {
        vitals.dom_content_loaded = Some(dcl);
    }

    Ok(vitals)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vital_metric_good() {
        let metric = VitalMetric::new(1500.0, 2500.0, 4000.0);
        assert!(metric.is_good());
        assert_eq!(metric.rating, "good");
    }

    #[test]
    fn test_vital_metric_needs_improvement() {
        let metric = VitalMetric::new(3000.0, 2500.0, 4000.0);
        assert!(!metric.is_good());
        assert_eq!(metric.rating, "needs-improvement");
    }

    #[test]
    fn test_vital_metric_poor() {
        let metric = VitalMetric::new(5000.0, 2500.0, 4000.0);
        assert!(!metric.is_good());
        assert_eq!(metric.rating, "poor");
    }

    #[test]
    fn test_web_vitals_good_count() {
        let vitals = WebVitals {
            lcp: Some(VitalMetric::new(2000.0, 2500.0, 4000.0)),
            fcp: Some(VitalMetric::new(1500.0, 1800.0, 3000.0)),
            cls: Some(VitalMetric::new(0.05, 0.1, 0.25)),
            ..Default::default()
        };

        assert_eq!(vitals.good_count(), 3);
        assert_eq!(vitals.available_count(), 3);
    }
}
