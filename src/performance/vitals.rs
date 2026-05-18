//! Core Web Vitals extraction via CDP
//!
//! Collects LCP, FCP, CLS, TBT, TTFB and other performance metrics.

use chromiumoxide::cdp::browser_protocol::performance::GetMetricsParams;
use chromiumoxide::Page;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::error::{AuditError, Result};

/// Core Web Vitals and performance metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WebVitals {
    /// Largest Contentful Paint (ms) - target ≤2500
    pub lcp: Option<VitalMetric>,
    /// First Contentful Paint (ms) - target ≤1800
    pub fcp: Option<VitalMetric>,
    /// Cumulative Layout Shift - target ≤0.1
    pub cls: Option<VitalMetric>,
    /// Time to First Byte (ms)
    pub ttfb: Option<VitalMetric>,
    /// First Input Delay (ms) - deprecated but still tracked
    pub fid: Option<VitalMetric>,
    /// Total Blocking Time (ms)
    pub tbt: Option<VitalMetric>,
    /// Speed Index
    pub speed_index: Option<VitalMetric>,
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

    // TBT via Long Tasks — sum of blocking time (duration > 50ms) per task
    window.__ams_tbt = 0;
    try {
        new PerformanceObserver(function(list) {
            var entries = list.getEntries();
            for (var i = 0; i < entries.length; i++) {
                if (entries[i].duration > 50) {
                    window.__ams_tbt += entries[i].duration - 50;
                }
            }
        }).observe({ type: 'longtask', buffered: true });
    } catch(e) {}

    // CLS via layout-shift observer
    window.__ams_cls = 0;
    try {
        new PerformanceObserver(function(list) {
            var entries = list.getEntries();
            for (var i = 0; i < entries.length; i++) {
                if (!entries[i].hadRecentInput) {
                    window.__ams_cls += entries[i].value;
                }
            }
        }).observe({ type: 'layout-shift', buffered: true });
    } catch(e) {}
})();
"#;

    page.evaluate_on_new_document(script)
        .await
        .map_err(|e| AuditError::CdpError(format!("Failed to inject vitals observer: {}", e)))?;

    Ok(())
}

/// Read the pre-injected globals after navigation to collect LCP, TBT, CLS.
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
        cls: window.__ams_cls || 0
    });
})()
"#;

    let result = page
        .evaluate(js)
        .await
        .map_err(|e| AuditError::CdpError(format!("Preinjected vitals read failed: {}", e)))?;

    let json_str = result.value().and_then(|v| v.as_str()).unwrap_or("{}");
    let parsed: serde_json::Value = serde_json::from_str(json_str).unwrap_or_default();

    Ok(PreinjectedVitals {
        lcp: parsed["lcp"].as_f64().unwrap_or(0.0),
        tbt: parsed["tbt"].as_f64().unwrap_or(0.0),
        cls: parsed["cls"].as_f64().unwrap_or(0.0),
    })
}

/// Values collected by the pre-injected PerformanceObserver script.
struct PreinjectedVitals {
    lcp: f64,
    tbt: f64,
    cls: f64,
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
        }
    });

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

    info!(
        "Web Vitals extracted: LCP={:?}ms, FCP={:?}ms, CLS={:?}",
        vitals.lcp.as_ref().map(|v| v.value as i64),
        vitals.fcp.as_ref().map(|v| v.value as i64),
        vitals.cls.as_ref().map(|v| v.value)
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
