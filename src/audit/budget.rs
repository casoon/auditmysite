//! Performance budget evaluation.
//!
//! Evaluates an AuditReport against the configured budget limits and
//! returns a list of violations, sorted by severity then metric name.

use serde::{Deserialize, Serialize};

use crate::audit::report::AuditReport;

#[derive(Debug, Deserialize, Default, Clone)]
pub struct BudgetConfig {
    /// Maximum LCP in milliseconds (good: ≤ 2500)
    pub max_lcp_ms: Option<f64>,
    /// Maximum FCP in milliseconds (good: ≤ 1800)
    pub max_fcp_ms: Option<f64>,
    /// Maximum CLS (good: ≤ 0.1)
    pub max_cls: Option<f64>,
    /// Maximum TBT in milliseconds (good: ≤ 200)
    pub max_tbt_ms: Option<f64>,
    /// Maximum total JavaScript size in KB
    pub max_js_kb: Option<f64>,
    /// Maximum total CSS size in KB
    pub max_css_kb: Option<f64>,
    /// Maximum total page size in KB
    pub max_total_kb: Option<f64>,
    /// Maximum number of render-blocking scripts
    pub max_blocking_scripts: Option<u32>,
    /// Maximum third-party transfer size in KB
    pub max_third_party_kb: Option<f64>,
    /// Maximum number of HTTP requests
    pub max_request_count: Option<u32>,
}

impl BudgetConfig {
    pub fn is_empty(&self) -> bool {
        self.max_lcp_ms.is_none()
            && self.max_fcp_ms.is_none()
            && self.max_cls.is_none()
            && self.max_tbt_ms.is_none()
            && self.max_js_kb.is_none()
            && self.max_css_kb.is_none()
            && self.max_total_kb.is_none()
            && self.max_blocking_scripts.is_none()
            && self.max_third_party_kb.is_none()
            && self.max_request_count.is_none()
    }
}

/// Severity of a budget violation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum BudgetSeverity {
    /// Actual value exceeds budget by more than 50%
    Error,
    /// Actual value exceeds budget by up to 50%
    Warning,
}

impl BudgetSeverity {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Error => "Error",
            Self::Warning => "Warning",
        }
    }
}

/// A single budget violation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct BudgetViolation {
    /// Metric name (e.g. "LCP", "JS-Größe")
    pub metric: String,
    /// Budget limit as human-readable string (e.g. "≤ 2500 ms")
    pub budget_label: String,
    /// Actual value as human-readable string (e.g. "3200 ms")
    pub actual_label: String,
    /// Raw budget value
    pub budget_value: f64,
    /// Raw actual value
    pub actual_value: f64,
    /// How much the actual exceeds the budget, as a percentage
    pub exceeded_by_pct: f64,
    /// Severity
    pub severity: BudgetSeverity,
}

impl BudgetViolation {
    fn new(
        metric: impl Into<String>,
        budget_label: impl Into<String>,
        actual_label: impl Into<String>,
        budget_value: f64,
        actual_value: f64,
    ) -> Self {
        let exceeded_by_pct = if budget_value > 0.0 {
            ((actual_value - budget_value) / budget_value * 100.0).max(0.0)
        } else {
            0.0
        };
        let severity = if exceeded_by_pct > 50.0 {
            BudgetSeverity::Error
        } else {
            BudgetSeverity::Warning
        };
        Self {
            metric: metric.into(),
            budget_label: budget_label.into(),
            actual_label: actual_label.into(),
            budget_value,
            actual_value,
            exceeded_by_pct,
            severity,
        }
    }
}

/// Evaluate all configured budget limits against the report.
/// Returns an empty vec if no budgets are configured or no performance data is available.
/// Evaluate performance budgets. Metric names are baked in canonical English
/// (so the stored report / JSON is language-independent); the PDF re-localizes
/// the few translatable metric names at render time (#406).
pub fn evaluate_budgets(report: &AuditReport, config: &BudgetConfig) -> Vec<BudgetViolation> {
    if config.is_empty() {
        return Vec::new();
    }

    let mut violations = Vec::new();

    let perf = match report.performance.as_ref() {
        Some(p) => p,
        None => return Vec::new(),
    };

    // ── Core Web Vitals ────────────────────────────────────────────────────
    if let (Some(limit), Some(lcp)) = (config.max_lcp_ms, perf.vitals.lcp.as_ref()) {
        if lcp.value > limit {
            violations.push(BudgetViolation::new(
                "LCP",
                format!("≤ {limit:.0} ms"),
                format!("{:.0} ms", lcp.value),
                limit,
                lcp.value,
            ));
        }
    }

    if let (Some(limit), Some(fcp)) = (config.max_fcp_ms, perf.vitals.fcp.as_ref()) {
        if fcp.value > limit {
            violations.push(BudgetViolation::new(
                "FCP",
                format!("≤ {limit:.0} ms"),
                format!("{:.0} ms", fcp.value),
                limit,
                fcp.value,
            ));
        }
    }

    if let (Some(limit), Some(cls)) = (config.max_cls, perf.vitals.cls.as_ref()) {
        if cls.value > limit {
            violations.push(BudgetViolation::new(
                "CLS",
                format!("≤ {limit:.3}"),
                format!("{:.3}", cls.value),
                limit,
                cls.value,
            ));
        }
    }

    if let (Some(limit), Some(tbt)) = (config.max_tbt_ms, perf.vitals.tbt.as_ref()) {
        if tbt.value > limit {
            violations.push(BudgetViolation::new(
                "TBT",
                format!("≤ {limit:.0} ms"),
                format!("{:.0} ms", tbt.value),
                limit,
                tbt.value,
            ));
        }
    }

    // ── Asset sizes (via content_weight) ───────────────────────────────────
    if let Some(ref cw) = perf.content_weight {
        if let Some(limit_kb) = config.max_js_kb {
            let actual_kb = cw.breakdown.javascript.bytes as f64 / 1024.0;
            if actual_kb > limit_kb {
                violations.push(BudgetViolation::new(
                    "JS size",
                    format!("≤ {limit_kb:.0} KB"),
                    format!("{actual_kb:.0} KB"),
                    limit_kb,
                    actual_kb,
                ));
            }
        }

        if let Some(limit_kb) = config.max_css_kb {
            let actual_kb = cw.breakdown.css.bytes as f64 / 1024.0;
            if actual_kb > limit_kb {
                violations.push(BudgetViolation::new(
                    "CSS size",
                    format!("≤ {limit_kb:.0} KB"),
                    format!("{actual_kb:.0} KB"),
                    limit_kb,
                    actual_kb,
                ));
            }
        }

        if let Some(limit_kb) = config.max_total_kb {
            let actual_kb = cw.total_bytes as f64 / 1024.0;
            if actual_kb > limit_kb {
                violations.push(BudgetViolation::new(
                    "Page size",
                    format!("≤ {limit_kb:.0} KB"),
                    format!("{actual_kb:.0} KB"),
                    limit_kb,
                    actual_kb,
                ));
            }
        }

        if let Some(limit) = config.max_request_count {
            if cw.request_count > limit {
                violations.push(BudgetViolation::new(
                    "Requests",
                    format!("≤ {limit}"),
                    cw.request_count.to_string(),
                    limit as f64,
                    cw.request_count as f64,
                ));
            }
        }
    }

    // ── Render-blocking (via render_blocking) ──────────────────────────────
    if let Some(ref rb) = perf.render_blocking {
        if let Some(limit) = config.max_blocking_scripts {
            let actual = rb.blocking_scripts.len() as u32;
            if actual > limit {
                violations.push(BudgetViolation::new(
                    "Blocking Scripts",
                    format!("≤ {limit}"),
                    actual.to_string(),
                    limit as f64,
                    actual as f64,
                ));
            }
        }

        if let Some(limit_kb) = config.max_third_party_kb {
            let actual_kb = rb.third_party_bytes as f64 / 1024.0;
            if actual_kb > limit_kb {
                violations.push(BudgetViolation::new(
                    "Third-Party",
                    format!("≤ {limit_kb:.0} KB"),
                    format!("{actual_kb:.0} KB"),
                    limit_kb,
                    actual_kb,
                ));
            }
        }
    }

    // Sort: Error before Warning, then by metric name
    violations.sort_by(|a, b| {
        let sev_order = |s: &BudgetSeverity| if *s == BudgetSeverity::Error { 0 } else { 1 };
        sev_order(&a.severity)
            .cmp(&sev_order(&b.severity))
            .then_with(|| a.metric.cmp(&b.metric))
    });

    violations
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_report() -> AuditReport {
        use crate::audit::report::AuditReport;
        use crate::model::WcagLevel;
        use crate::wcag::WcagResults;
        AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            WcagResults::default(),
            0,
        )
    }

    #[test]
    fn test_evaluate_empty_config_returns_no_violations() {
        let report = empty_report();
        let config = BudgetConfig::default();
        assert!(evaluate_budgets(&report, &config).is_empty());
    }

    #[test]
    fn test_evaluate_no_performance_data_returns_no_violations() {
        let report = empty_report();
        let config = BudgetConfig {
            max_lcp_ms: Some(2500.0),
            ..Default::default()
        };
        assert!(evaluate_budgets(&report, &config).is_empty());
    }

    #[test]
    fn test_budget_severity_error_when_exceeds_50_pct() {
        let v = BudgetViolation::new("LCP", "≤ 2500 ms", "4000 ms", 2500.0, 4000.0);
        assert_eq!(v.severity, BudgetSeverity::Error);
        assert!((v.exceeded_by_pct - 60.0).abs() < 1.0);
    }

    #[test]
    fn test_budget_severity_warning_when_slightly_over() {
        let v = BudgetViolation::new("LCP", "≤ 2500 ms", "3000 ms", 2500.0, 3000.0);
        assert_eq!(v.severity, BudgetSeverity::Warning);
    }

    #[test]
    fn test_budget_config_is_empty() {
        assert!(BudgetConfig::default().is_empty());
        let c = BudgetConfig {
            max_lcp_ms: Some(2500.0),
            ..Default::default()
        };
        assert!(!c.is_empty());
    }
}
