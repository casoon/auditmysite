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

use chromiumoxide::cdp::browser_protocol::css::{
    StartRuleUsageTrackingParams, StopRuleUsageTrackingParams,
};
use chromiumoxide::cdp::js_protocol::profiler::{
    EnableParams as ProfilerEnableParams, StartPreciseCoverageParams, TakePreciseCoverageParams,
};
use chromiumoxide::Page;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::error::{AuditError, Result};

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
    /// Overall JS usage percentage (0–100)
    pub used_pct: f64,
}

/// Unused CSS analysis (#107).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnusedCssAnalysis {
    /// Total CSS rules observed by the browser
    pub total_rules: u32,
    /// Rules that matched at least one element during the audit
    pub used_rules: u32,
    /// Percentage of rules that were used (0–100), or null when CDP returned no CSS data.
    pub used_pct: Option<f64>,
    /// Measurement state for CSS rule usage data.
    pub measurement: String,
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

    Ok(())
}

/// Read JS and CSS coverage results **after** the page has loaded.
pub async fn take_coverage_results(page: &Page) -> Result<CoverageAnalysis> {
    info!("Taking JS and CSS coverage results...");

    // ── JS coverage ───────────────────────────────────────────────────────────
    let js_coverage = match page.execute(TakePreciseCoverageParams::default()).await {
        Ok(resp) => resp.result.result,
        Err(e) => {
            warn!("Profiler.takePreciseCoverage failed: {e}");
            vec![]
        }
    };

    let mut scripts: Vec<ScriptCoverageEntry> = Vec::new();
    let mut total_bytes: u64 = 0;
    let mut total_unused_bytes: u64 = 0;

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
        }
    }

    scripts.sort_by_key(|s| std::cmp::Reverse(s.unused_bytes));

    let overall_used_pct = if total_bytes > 0 {
        ((total_bytes - total_unused_bytes) as f64 / total_bytes as f64 * 100.0).clamp(0.0, 100.0)
    } else {
        100.0
    };

    let unused_js = UnusedJsAnalysis {
        scripts,
        total_bytes,
        unused_bytes: total_unused_bytes,
        used_pct: overall_used_pct,
    };

    // ── CSS coverage ──────────────────────────────────────────────────────────
    let css_coverage = match page.execute(StopRuleUsageTrackingParams::default()).await {
        Ok(resp) => resp.result.rule_usage,
        Err(e) => {
            warn!("CSS.stopRuleUsageTracking failed: {e}");
            vec![]
        }
    };

    let total_rules = css_coverage.len() as u32;
    let used_rules = css_coverage.iter().filter(|r| r.used).count() as u32;
    let (css_used_pct, css_measurement, measurement_warnings) = if total_rules > 0 {
        (
            Some((used_rules as f64 / total_rules as f64 * 100.0).clamp(0.0, 100.0)),
            "measured".to_string(),
            vec![],
        )
    } else {
        (
            None,
            "not_available".to_string(),
            vec!["css_coverage_unavailable".to_string()],
        )
    };

    let unused_css = UnusedCssAnalysis {
        total_rules,
        used_rules,
        used_pct: css_used_pct,
        measurement: css_measurement,
    };

    info!(
        "Coverage: JS {:.0}% used ({:.1} KB unused), CSS {} rules used ({}/{})",
        overall_used_pct,
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
    })
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_string();
    }
    let boundary = s
        .char_indices()
        .take_while(|(i, _)| *i <= max.saturating_sub(3))
        .last()
        .map(|(i, _)| i)
        .unwrap_or(0);
    format!("{}…", &s[..boundary])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unused_js_zero_bytes() {
        let analysis = UnusedJsAnalysis {
            scripts: vec![],
            total_bytes: 0,
            unused_bytes: 0,
            used_pct: 100.0,
        };
        assert_eq!(analysis.used_pct, 100.0);
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
}
