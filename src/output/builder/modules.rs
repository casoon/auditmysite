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
        // Stored title is canonical English (#406); re-derive the localized
        // taxonomy title for non-English reports.
        let title = if i18n.locale() == "en" {
            finding.title.clone()
        } else {
            crate::taxonomy::RuleLookup::by_id(&finding.rule_id)
                .map(|r| r.title.to_string())
                .unwrap_or_else(|| finding.title.clone())
        };
        i18n.t_args(
            "lever-accessibility-biggest",
            &[("finding", title.as_str())],
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

    // Issue-driven recommendations. Testing only for *absent* headers left a
    // present-but-misconfigured header with nothing to say: on casoon.de
    // (2026-09-19) a High "CSP allows unsafe-inline scripts without nonce/hash
    // protection" produced no recommendation at all, so the list fell through
    // to `recommendation-security-default` — whose text asserts the headers
    // are cleanly set, directly below the finding (plan 33).
    //
    // Keyed on `SecurityIssueKind` rather than on the message text, so a
    // reworded finding cannot silently drop its recommendation.
    for issue in &sec.issues {
        let Some(kind) = issue.kind() else { continue };
        use crate::security::SecurityIssueKind;
        let key = match kind {
            SecurityIssueKind::CspUnsafeInlineScript | SecurityIssueKind::CspUnsafeInlineStyle => {
                "recommendation-security-csp-unsafe-inline"
            }
            SecurityIssueKind::CspUnsafeEvalScript => "recommendation-security-csp-unsafe-eval",
            SecurityIssueKind::CspWildcardScriptSource | SecurityIssueKind::CspWildcardSource => {
                "recommendation-security-csp-wildcard"
            }
            SecurityIssueKind::CspMissingDirective => "recommendation-security-csp-directive",
            SecurityIssueKind::CorsWildcardCredentials => {
                "recommendation-security-cors-credentials"
            }
            SecurityIssueKind::PermissionsPolicyPermissive => {
                "recommendation-security-permissions-permissive"
            }
            SecurityIssueKind::PublicSourceMap => "recommendation-security-sourcemap",
            SecurityIssueKind::HstsPreloadIneligible => "recommendation-security-hsts-preload",
            // Missing headers are already covered by the checks above; adding
            // them here would duplicate every entry.
            _ => continue,
        };
        let text = i18n.t(key);
        if !recommendations.contains(&text) {
            recommendations.push(text);
        }
    }

    // The default text states that the basic headers are cleanly set, so it
    // may only appear when there is genuinely nothing to report.
    if recommendations.is_empty() && sec.issues.is_empty() {
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
        if technical.tracking_cookies.is_empty()
            && technical.tracking_signals.is_empty()
            && technical.storage_items.is_empty()
        {
            return i18n.t("tracking-summary-zaraz-clean");
        }
        return i18n.t("tracking-summary-zaraz-signals");
    }

    if technical.uses_remote_google_fonts {
        return i18n.t("tracking-summary-fonts");
    }

    if !technical.tracking_cookies.is_empty()
        || !technical.tracking_signals.is_empty()
        || !technical.storage_items.is_empty()
    {
        return i18n.t("tracking-summary-signals");
    }

    i18n.t("tracking-summary-clean")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::{
        HeaderTier, ProtectionDetection, SecurityAnalysis, SecurityHeaders, SecurityIssue,
        SourceMapLeakAudit, SslInfo,
    };
    use crate::taxonomy::Severity;

    fn analysis(headers: SecurityHeaders, issues: Vec<SecurityIssue>) -> SecurityAnalysis {
        SecurityAnalysis {
            score: 95,
            grade: "A".to_string(),
            headers,
            ssl: SslInfo {
                https: true,
                ..Default::default()
            },
            issues,
            recommendations: Vec::new(),
            protection: ProtectionDetection::default(),
            sourcemap_leaks: SourceMapLeakAudit::default(),
        }
    }

    fn all_headers_present() -> SecurityHeaders {
        SecurityHeaders {
            content_security_policy: Some("default-src 'self'; script-src 'unsafe-inline'".into()),
            strict_transport_security: Some("max-age=31536000".into()),
            x_content_type_options: Some("nosniff".into()),
            x_frame_options: Some("SAMEORIGIN".into()),
            referrer_policy: Some("strict-origin".into()),
            permissions_policy: Some("camera=()".into()),
            cross_origin_opener_policy: Some("same-origin".into()),
            cross_origin_resource_policy: Some("cross-origin".into()),
            ..Default::default()
        }
    }

    fn csp_issue(issue_type: &str, severity: Severity) -> SecurityIssue {
        SecurityIssue {
            header: "Content-Security-Policy".to_string(),
            issue_type: issue_type.to_string(),
            message: "canonical english message".to_string(),
            severity,
            tier: HeaderTier::Baseline,
            values: Default::default(),
        }
    }

    /// Plan 33: every header present but the CSP misconfigured. The old
    /// derivation only tested for *absent* headers, produced nothing, and fell
    /// through to the default text — which asserts the headers are cleanly
    /// set, directly below the finding. Confirmed live on casoon.de
    /// (2026-09-19).
    #[test]
    fn misconfigured_header_produces_a_recommendation_not_the_all_clear() {
        let i18n = I18n::new("de").expect("locale loads");
        let sec = analysis(
            all_headers_present(),
            vec![
                csp_issue("unsafe_inline_script", Severity::High),
                csp_issue("unsafe_inline_style", Severity::Medium),
                csp_issue("missing_frame-ancestors", Severity::Medium),
            ],
        );

        let recs = derive_security_recommendations(&i18n, &sec);

        assert!(!recs.is_empty(), "a misconfigured CSP must yield an action");
        let default = i18n.t("recommendation-security-default");
        assert!(
            !recs.contains(&default),
            "the all-clear text must not appear next to open issues: {recs:#?}",
        );
        assert!(
            recs.iter().any(|r| r.contains("unsafe-inline")),
            "expected the unsafe-inline remediation: {recs:#?}",
        );
    }

    /// The all-clear stays available when it is true.
    #[test]
    fn clean_security_analysis_keeps_the_default_recommendation() {
        let i18n = I18n::new("de").expect("locale loads");
        let sec = analysis(all_headers_present(), Vec::new());

        let recs = derive_security_recommendations(&i18n, &sec);
        assert_eq!(recs, vec![i18n.t("recommendation-security-default")]);
    }

    /// Missing headers must not be reported twice now that issues also feed
    /// the list.
    #[test]
    fn missing_header_recommendations_are_not_duplicated() {
        let i18n = I18n::new("de").expect("locale loads");
        let sec = analysis(
            SecurityHeaders::default(),
            vec![SecurityIssue {
                header: "Content-Security-Policy".to_string(),
                issue_type: "missing_header".to_string(),
                message: "Content-Security-Policy header is missing".to_string(),
                severity: Severity::High,
                tier: HeaderTier::Baseline,
                values: Default::default(),
            }],
        );

        let recs = derive_security_recommendations(&i18n, &sec);
        let csp = i18n.t("recommendation-security-csp");
        assert_eq!(
            recs.iter().filter(|r| **r == csp).count(),
            1,
            "missing-CSP recommendation duplicated: {recs:#?}",
        );
    }
}
