//! Module-level score and context derivation helpers (accessibility, performance, SEO, security, mobile).

use crate::audit::NormalizedReport;
use crate::wcag::Severity;

use crate::i18n::I18n;

// ─── Accessibility ───────────────────────────────────────────────────────────

pub(super) fn derive_accessibility_lever(i18n: &I18n, normalized: &NormalizedReport) -> String {
    if let Some(finding) = normalized
        .findings
        .iter()
        .max_by_key(|f| f.occurrence_count)
    {
        i18n.t_args(
            "lever-accessibility-biggest",
            &[("finding", finding.title.as_str())],
        )
    } else {
        i18n.t("lever-accessibility-default")
    }
}

pub(super) fn derive_accessibility_context(i18n: &I18n, normalized: &NormalizedReport) -> String {
    let high = normalized
        .findings
        .iter()
        .filter(|f| matches!(f.severity, Severity::High | Severity::Critical))
        .count();
    let total = normalized.findings.len();
    if total == 0 {
        return i18n.t("context-accessibility-none");
    }
    i18n.t_args(
        "context-accessibility-summary",
        &[("total", total.to_string()), ("high", high.to_string())],
    )
}

pub(super) fn derive_accessibility_card_context(
    i18n: &I18n,
    normalized: &NormalizedReport,
) -> String {
    let high = normalized
        .findings
        .iter()
        .filter(|f| matches!(f.severity, Severity::High | Severity::Critical))
        .count();
    if high == 0 {
        i18n.t("card-accessibility-none")
    } else {
        i18n.t_args("card-accessibility-summary", &[("high", high.to_string())])
    }
}

// ─── Performance ─────────────────────────────────────────────────────────────

pub(super) fn derive_performance_lever(
    i18n: &I18n,
    perf: &crate::audit::PerformanceResults,
) -> String {
    if let Some(dom_nodes) = perf.vitals.dom_nodes {
        if dom_nodes > 1500 {
            return i18n.t_args(
                "lever-performance-dom",
                &[("dom_nodes", dom_nodes.to_string())],
            );
        }
    }
    if let Some(load) = perf.vitals.load_time {
        if load > 2_500.0 {
            return i18n.t_args(
                "lever-performance-load",
                &[("load", format!("{:.0}", load))],
            );
        }
    }
    i18n.t("lever-performance-default")
}

pub(super) fn derive_performance_context(
    i18n: &I18n,
    perf: &crate::audit::PerformanceResults,
) -> String {
    let fcp_good = perf
        .vitals
        .fcp
        .as_ref()
        .map(|v| v.rating == "good")
        .unwrap_or(false);
    let lcp_good = perf
        .vitals
        .lcp
        .as_ref()
        .map(|v| v.rating == "good")
        .unwrap_or(false);
    let vitals_measured = perf.vitals.fcp.is_some() || perf.vitals.lcp.is_some();
    let high_dom = perf.vitals.dom_nodes.map(|n| n > 1500).unwrap_or(false);
    let has_blocking = perf
        .render_blocking
        .as_ref()
        .map(|rb| rb.has_blocking())
        .unwrap_or(false);

    // If user-perceived vitals are good but overall score is dragged down by complexity, say so.
    if vitals_measured
        && (fcp_good || lcp_good)
        && perf.score.overall < 75
        && (high_dom || has_blocking)
    {
        let fcp_str = perf
            .vitals
            .fcp
            .as_ref()
            .map(|v| format!("FCP {:.0} ms", v.value))
            .unwrap_or_else(|| "FCP n/a".to_string());
        return i18n.t_args("context-performance-good-vitals", &[("fcp", fcp_str)]);
    }

    let fcp = perf
        .vitals
        .fcp
        .as_ref()
        .map(|v| format!("FCP {:.0} ms", v.value))
        .unwrap_or_else(|| "FCP n/a".to_string());
    let ttfb = perf
        .vitals
        .ttfb
        .as_ref()
        .map(|v| format!("TTFB {:.0} ms", v.value))
        .unwrap_or_else(|| "TTFB n/a".to_string());
    let dom = perf
        .vitals
        .dom_nodes
        .map(|n| i18n.t_args("context-performance-dom-nodes", &[("n", n.to_string())]))
        .unwrap_or_else(|| i18n.t("context-performance-dom-na"));
    i18n.t_args(
        "context-performance-summary",
        &[("fcp", fcp), ("ttfb", ttfb), ("dom", dom)],
    )
}

pub(super) fn derive_performance_card_context(
    i18n: &I18n,
    perf: &crate::audit::PerformanceResults,
) -> String {
    if let Some(dom_nodes) = perf.vitals.dom_nodes {
        return i18n.t_args(
            "card-performance-dom",
            &[("dom_nodes", dom_nodes.to_string())],
        );
    }
    if let Some(load) = perf.vitals.load_time {
        return i18n.t_args("card-performance-load", &[("load", format!("{:.0}", load))]);
    }
    i18n.t("card-performance-default")
}

/// Build a `(name, formatted_value, rating)` vitals list from a PerformanceResults.
/// Used for both desktop and mobile viewport presentations.
///
/// Estimated lab metrics (INP, TTI, Speed Index) carry a localized "(lab
/// estimate)" suffix so they cannot be mistaken for directly measured — or, more
/// importantly, real field/RUM — values (#262). All values are local headless
/// lab data.
pub(super) fn build_vitals_list(
    p: &crate::audit::PerformanceResults,
    i18n: &I18n,
) -> Vec<(String, String, String)> {
    let estimated_suffix = i18n.t("perf-lab-estimate-suffix");
    let label = |base: &str, m: &crate::performance::VitalMetric| {
        if m.is_estimated() {
            format!("{base}{estimated_suffix}")
        } else {
            base.to_string()
        }
    };

    let mut vitals = Vec::new();
    if let Some(ref lcp) = p.vitals.lcp {
        vitals.push((
            label("LCP", lcp),
            format!("{:.0}ms", lcp.value),
            lcp.rating.clone(),
        ));
    }
    if let Some(ref fcp) = p.vitals.fcp {
        vitals.push((
            label("FCP", fcp),
            format!("{:.0}ms", fcp.value),
            fcp.rating.clone(),
        ));
    }
    if let Some(ref cls) = p.vitals.cls {
        vitals.push((
            label("CLS", cls),
            format!("{:.3}", cls.value),
            cls.rating.clone(),
        ));
    }
    if let Some(ref ttfb) = p.vitals.ttfb {
        vitals.push((
            label("TTFB", ttfb),
            format!("{:.0}ms", ttfb.value),
            ttfb.rating.clone(),
        ));
    }
    if let Some(ref tbt) = p.vitals.tbt {
        vitals.push((
            label("TBT", tbt),
            format!("{:.0}ms", tbt.value),
            tbt.rating.clone(),
        ));
    }
    if let Some(ref tti) = p.vitals.tti {
        vitals.push((
            label("TTI", tti),
            format!("{:.0}ms", tti.value),
            tti.rating.clone(),
        ));
    }
    if let Some(ref inp) = p.vitals.inp {
        vitals.push((
            label("INP", inp),
            format!("{:.0}ms", inp.value),
            inp.rating.clone(),
        ));
    }
    if let Some(ref si) = p.vitals.speed_index {
        vitals.push((
            label("Speed Index", si),
            format!("{:.0}ms", si.value),
            si.rating.clone(),
        ));
    }
    vitals
}

pub(super) fn derive_performance_recommendations(
    i18n: &I18n,
    perf: &crate::audit::PerformanceResults,
) -> Vec<String> {
    let mut recommendations = Vec::new();

    if let Some(lcp) = &perf.vitals.lcp {
        if lcp.value > 2500.0 {
            recommendations.push(i18n.t("recommendation-performance-lcp"));
        }
    }

    if let Some(fcp) = &perf.vitals.fcp {
        if fcp.value > 1800.0 {
            recommendations.push(i18n.t("recommendation-performance-fcp"));
        }
    }

    if let Some(interactivity) = perf.vitals.tbt.as_ref() {
        if interactivity.value > 200.0 {
            recommendations.push(i18n.t("recommendation-performance-tbt"));
        }
    }

    if let Some(cls) = &perf.vitals.cls {
        if cls.value > 0.1 {
            recommendations.push(i18n.t("recommendation-performance-cls"));
        }
    }

    if let Some(dom_nodes) = perf.vitals.dom_nodes {
        if dom_nodes > 1200 {
            recommendations.push(i18n.t("recommendation-performance-dom"));
        }
    }

    if let Some(load_time) = perf.vitals.load_time {
        if load_time > 3000.0 {
            recommendations.push(i18n.t("recommendation-performance-load"));
        }
    }

    if recommendations.is_empty() {
        recommendations.push(i18n.t("recommendation-performance-default"));
    }

    recommendations.truncate(3);
    recommendations
}

// ─── SEO ─────────────────────────────────────────────────────────────────────

pub(super) fn derive_seo_lever(i18n: &I18n, seo: &crate::seo::SeoAnalysis) -> String {
    if !seo.meta_issues.is_empty() {
        return i18n.t_args(
            "lever-seo-meta",
            &[("open_issues", seo.meta_issues.len().to_string())],
        );
    }
    if seo.social.completeness < 80 {
        return i18n.t("lever-seo-social");
    }
    i18n.t("lever-seo-default")
}

pub(super) fn derive_seo_context(i18n: &I18n, seo: &crate::seo::SeoAnalysis) -> String {
    let meta_issues = seo.meta_issues.len();
    let schema_count = seo.structured_data.json_ld.len();
    let h1 = seo.headings.h1_count;
    i18n.t_args(
        "context-seo-summary",
        &[
            ("meta_issues", meta_issues.to_string()),
            ("h1", h1.to_string()),
            ("schema_count", schema_count.to_string()),
        ],
    )
}

pub(super) fn derive_seo_card_context(i18n: &I18n, seo: &crate::seo::SeoAnalysis) -> String {
    if !seo.meta_issues.is_empty() {
        i18n.t_args(
            "card-seo-meta",
            &[("meta_issues", seo.meta_issues.len().to_string())],
        )
    } else {
        i18n.t_args(
            "card-seo-schema",
            &[(
                "schema_count",
                seo.structured_data.json_ld.len().to_string(),
            )],
        )
    }
}

// ─── Security ────────────────────────────────────────────────────────────────

pub(super) fn derive_security_lever(
    i18n: &I18n,
    sec: &crate::security::SecurityAnalysis,
) -> String {
    let missing_headers = sec.headers.content_security_policy.is_none() as usize
        + sec.headers.strict_transport_security.is_none() as usize
        + sec.headers.permissions_policy.is_none() as usize
        + sec.headers.referrer_policy.is_none() as usize;
    if missing_headers > 0 {
        return i18n.t_args(
            "lever-security-headers",
            &[("missing_headers", missing_headers.to_string())],
        );
    }
    i18n.t("lever-security-default")
}

pub(super) fn derive_security_context(
    i18n: &I18n,
    sec: &crate::security::SecurityAnalysis,
) -> String {
    let present_headers = [
        sec.headers.content_security_policy.is_some(),
        sec.headers.strict_transport_security.is_some(),
        sec.headers.x_content_type_options.is_some(),
        sec.headers.x_frame_options.is_some(),
        sec.headers.referrer_policy.is_some(),
        sec.headers.permissions_policy.is_some(),
        sec.headers.cross_origin_opener_policy.is_some(),
        sec.headers.cross_origin_resource_policy.is_some(),
    ]
    .into_iter()
    .filter(|p| *p)
    .count();
    if sec.ssl.https {
        i18n.t_args(
            "context-security-summary-https",
            &[("present_headers", present_headers.to_string())],
        )
    } else {
        i18n.t_args(
            "context-security-summary-nohttps",
            &[("present_headers", present_headers.to_string())],
        )
    }
}

pub(super) fn derive_security_card_context(
    i18n: &I18n,
    sec: &crate::security::SecurityAnalysis,
) -> String {
    let present_headers = [
        sec.headers.content_security_policy.is_some(),
        sec.headers.strict_transport_security.is_some(),
        sec.headers.x_content_type_options.is_some(),
        sec.headers.x_frame_options.is_some(),
        sec.headers.referrer_policy.is_some(),
        sec.headers.permissions_policy.is_some(),
        sec.headers.cross_origin_opener_policy.is_some(),
        sec.headers.cross_origin_resource_policy.is_some(),
    ]
    .into_iter()
    .filter(|p| *p)
    .count();
    i18n.t_args(
        "card-security-summary",
        &[("present_headers", present_headers.to_string())],
    )
}

pub(super) fn derive_security_recommendations(
    i18n: &I18n,
    sec: &crate::security::SecurityAnalysis,
) -> Vec<String> {
    let mut recommendations = Vec::new();

    if !sec.ssl.https {
        recommendations.push(i18n.t("recommendation-security-https"));
    }

    if sec.headers.content_security_policy.is_none() {
        recommendations.push(i18n.t("recommendation-security-csp"));
    }

    if sec.headers.strict_transport_security.is_none() && sec.ssl.https {
        recommendations.push(i18n.t("recommendation-security-hsts"));
    }

    if sec.headers.cross_origin_opener_policy.is_none() {
        recommendations.push(i18n.t("recommendation-security-coop"));
    }

    if sec.headers.cross_origin_resource_policy.is_none() {
        recommendations.push(i18n.t("recommendation-security-corp"));
    }

    if sec.headers.permissions_policy.is_none() {
        recommendations.push(i18n.t("recommendation-security-permissions"));
    }

    if sec.headers.referrer_policy.is_none() {
        recommendations.push(i18n.t("recommendation-security-referrer"));
    }

    if recommendations.is_empty() {
        recommendations.push(i18n.t("recommendation-security-default"));
    }

    recommendations.truncate(4);
    recommendations
}

// ─── Mobile ──────────────────────────────────────────────────────────────────

pub(super) fn derive_mobile_lever(
    i18n: &I18n,
    mobile: &crate::mobile::MobileFriendliness,
) -> String {
    if mobile.touch_targets.small_targets > 0 {
        return i18n.t_args(
            "lever-mobile-small",
            &[(
                "small_targets",
                mobile.touch_targets.small_targets.to_string(),
            )],
        );
    }
    if mobile.touch_targets.crowded_targets > 0 {
        return i18n.t_args(
            "lever-mobile-crowded",
            &[(
                "crowded_targets",
                mobile.touch_targets.crowded_targets.to_string(),
            )],
        );
    }
    i18n.t("lever-mobile-default")
}

pub(super) fn derive_mobile_context(
    i18n: &I18n,
    mobile: &crate::mobile::MobileFriendliness,
) -> String {
    if mobile.viewport.is_properly_configured {
        i18n.t_args(
            "context-mobile-proper",
            &[
                (
                    "small_targets",
                    mobile.touch_targets.small_targets.to_string(),
                ),
                (
                    "crowded_targets",
                    mobile.touch_targets.crowded_targets.to_string(),
                ),
            ],
        )
    } else {
        i18n.t_args(
            "context-mobile-improper",
            &[
                (
                    "small_targets",
                    mobile.touch_targets.small_targets.to_string(),
                ),
                (
                    "crowded_targets",
                    mobile.touch_targets.crowded_targets.to_string(),
                ),
            ],
        )
    }
}

pub(super) fn derive_mobile_card_context(
    i18n: &I18n,
    mobile: &crate::mobile::MobileFriendliness,
) -> String {
    if mobile.touch_targets.small_targets > 0 {
        i18n.t_args(
            "card-mobile-small",
            &[(
                "small_targets",
                mobile.touch_targets.small_targets.to_string(),
            )],
        )
    } else if mobile.touch_targets.crowded_targets > 0 {
        i18n.t_args(
            "card-mobile-crowded",
            &[(
                "crowded_targets",
                mobile.touch_targets.crowded_targets.to_string(),
            )],
        )
    } else if mobile.viewport.is_properly_configured {
        i18n.t("card-mobile-proper")
    } else {
        i18n.t("card-mobile-improper")
    }
}

// ─── Tracking ────────────────────────────────────────────────────────────────

pub(super) fn build_tracking_summary_text(
    i18n: &I18n,
    technical: &crate::seo::technical::TechnicalSeo,
) -> String {
    if technical.zaraz.detected {
        if technical.tracking_cookies.is_empty() && technical.tracking_signals.is_empty() {
            return i18n.t("tracking-summary-zaraz-clean");
        }
        return i18n.t("tracking-summary-zaraz-signals");
    }

    if technical.uses_remote_google_fonts {
        return i18n.t("tracking-summary-fonts");
    }

    if !technical.tracking_cookies.is_empty() || !technical.tracking_signals.is_empty() {
        return i18n.t("tracking-summary-signals");
    }

    i18n.t("tracking-summary-clean")
}
