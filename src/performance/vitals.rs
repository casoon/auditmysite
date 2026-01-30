//! Core Web Vitals extraction via CDP
//!
//! Collects LCP, FCP, CLS, INP, TTFB and other performance metrics.

use chromiumoxide::cdp::browser_protocol::performance::GetMetricsParams;
use chromiumoxide::Page;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::error::{AuditError, Result};

/// Core Web Vitals and performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct WebVitals {
    /// Largest Contentful Paint (ms) - target ≤2500
    pub lcp: Option<VitalMetric>,
    /// First Contentful Paint (ms) - target ≤1800
    pub fcp: Option<VitalMetric>,
    /// Cumulative Layout Shift - target ≤0.1
    pub cls: Option<VitalMetric>,
    /// Interaction to Next Paint (ms) - target ≤200
    pub inp: Option<VitalMetric>,
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
        [&self.lcp, &self.fcp, &self.cls, &self.inp]
            .iter()
            .filter(|v| v.as_ref().map(|m| m.is_good()).unwrap_or(false))
            .count()
    }

    /// Count how many vitals are available
    pub fn available_count(&self) -> usize {
        [&self.lcp, &self.fcp, &self.cls, &self.inp]
            .iter()
            .filter(|v| v.is_some())
            .count()
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

    let mut vitals = WebVitals::default();

    // Get performance metrics via CDP
    let metrics_response = page
        .execute(GetMetricsParams::default())
        .await
        .map_err(|e| AuditError::CdpError(format!("Failed to get metrics: {}", e)))?;

    // Parse metrics into our structure
    for metric in &metrics_response.metrics {
        let value = metric.value;
        match metric.name.as_str() {
            "FirstContentfulPaint" => {
                // Convert to ms
                let ms = value * 1000.0;
                vitals.fcp = Some(VitalMetric::new(ms, 1800.0, 3000.0));
                debug!("FCP: {:.0}ms", ms);
            }
            "LargestContentfulPaint" => {
                let ms = value * 1000.0;
                vitals.lcp = Some(VitalMetric::new(ms, 2500.0, 4000.0));
                debug!("LCP: {:.0}ms", ms);
            }
            "DomContentLoaded" => {
                vitals.dom_content_loaded = Some(value * 1000.0);
            }
            "NavigationStart" => {
                // Used for TTFB calculation
            }
            "JSHeapUsedSize" => {
                vitals.js_heap_size = Some(value as i64);
            }
            "Nodes" => {
                vitals.dom_nodes = Some(value as i64);
            }
            "LayoutCount" | "LayoutDuration" => {
                // Used for CLS approximation
            }
            "TaskDuration" => {
                // Total task duration - used for TBT approximation
                let ms = value * 1000.0;
                // TBT is sum of blocking time > 50ms
                if ms > 50.0 {
                    let current = vitals.tbt.as_ref().map(|v| v.value).unwrap_or(0.0);
                    vitals.tbt = Some(VitalMetric::new(current + (ms - 50.0), 200.0, 600.0));
                }
            }
            _ => {}
        }
    }

    // Extract additional metrics via JavaScript
    let js_metrics = extract_js_metrics(page).await?;

    // Merge JS metrics
    if vitals.lcp.is_none() && js_metrics.lcp.is_some() {
        vitals.lcp = js_metrics.lcp;
    }
    if vitals.fcp.is_none() && js_metrics.fcp.is_some() {
        vitals.fcp = js_metrics.fcp;
    }
    if js_metrics.cls.is_some() {
        vitals.cls = js_metrics.cls;
    }
    if js_metrics.ttfb.is_some() {
        vitals.ttfb = js_metrics.ttfb;
    }
    if js_metrics.load_time.is_some() {
        vitals.load_time = js_metrics.load_time;
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

        // CLS (approximate from layout-shift entries)
        const clsEntries = performance.getEntriesByType('layout-shift');
        let cls = 0;
        for (const entry of clsEntries) {
            if (!entry.hadRecentInput) {
                cls += entry.value;
            }
        }
        result.cls = cls;

        return JSON.stringify(result);
    })()
    "#;

    let js_result = page
        .evaluate(js_code)
        .await
        .map_err(|e| AuditError::CdpError(format!("JS metrics extraction failed: {}", e)))?;

    let json_str = js_result.value().and_then(|v| v.as_str()).unwrap_or("{}");

    let parsed: serde_json::Value = serde_json::from_str(json_str).unwrap_or_else(|e| {
        warn!("Failed to parse web vitals JSON: {}", e);
        serde_json::Value::Object(serde_json::Map::new())
    });

    let mut vitals = WebVitals::default();

    if let Some(lcp) = parsed["lcp"].as_f64() {
        vitals.lcp = Some(VitalMetric::new(lcp, 2500.0, 4000.0));
    }
    if let Some(fcp) = parsed["fcp"].as_f64() {
        vitals.fcp = Some(VitalMetric::new(fcp, 1800.0, 3000.0));
    }
    if let Some(cls) = parsed["cls"].as_f64() {
        vitals.cls = Some(VitalMetric::new(cls, 0.1, 0.25));
    }
    if let Some(ttfb) = parsed["ttfb"].as_f64() {
        vitals.ttfb = Some(VitalMetric::new(ttfb, 800.0, 1800.0));
    }
    if let Some(load_time) = parsed["loadTime"].as_f64() {
        vitals.load_time = Some(load_time);
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
        let mut vitals = WebVitals::default();
        vitals.lcp = Some(VitalMetric::new(2000.0, 2500.0, 4000.0));
        vitals.fcp = Some(VitalMetric::new(1500.0, 1800.0, 3000.0));
        vitals.cls = Some(VitalMetric::new(0.05, 0.1, 0.25));

        assert_eq!(vitals.good_count(), 3);
        assert_eq!(vitals.available_count(), 3);
    }
}
