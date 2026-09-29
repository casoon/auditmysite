use crate::audit::normalized::AuditContext;
use crate::audit::performance_interpretation::{
    append_performance_qualifiers_text, performance_gap_text,
};
use crate::i18n::I18n;
use crate::output::report_model::{
    AnimationPresentation, CoveragePresentation, CriticalChainPresentation, DuplicateAssetRow,
    MinificationPresentation, PerformancePresentation, PerformanceViewport, ThirdPartyImpactRow,
    ThirdPartyOriginRow, ThirdPartyPresentation, ThrottledPerfEntry,
};

use super::super::super::modules::{build_vitals_list, derive_performance_recommendations};
use super::{module_interpretation, normalized_module_grade, normalized_module_score};
use crate::util::truncate_url;

/// Three-tier rating for a metric with a fixed good/poor boundary, matching
/// the "good"/"needs-improvement"/"poor" vocabulary the Core Web Vitals
/// already use downstream (see `vital_status`/`vital_color` in
/// `output::pdf::detail_modules`) so both render through the same styling.
fn rate_threshold(value: f64, good_max: f64, poor_min: f64) -> &'static str {
    if value <= good_max {
        "good"
    } else if value < poor_min {
        "needs-improvement"
    } else {
        "poor"
    }
}

fn localized_decimal(value: f64, decimals: usize, en: bool) -> String {
    let formatted = format!("{value:.decimals$}");
    if en {
        formatted
    } else {
        formatted.replace('.', ",")
    }
}

fn localized_integer(value: i64, en: bool) -> String {
    let separator = if en { ',' } else { '.' };
    let digits = value.abs().to_string();
    let mut grouped = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            grouped.push(separator);
        }
        grouped.push(digit);
    }
    if value < 0 {
        grouped.insert(0, '-');
    }
    grouped
}

pub(super) fn build_performance_details(
    normalized: &AuditContext<'_>,
    i18n: &I18n,
) -> Option<PerformancePresentation> {
    let locale = i18n.locale();
    normalized.raw_performance.map(|p| {
        let performance_score = normalized_module_score(&normalized.normalized, "Performance")
            .unwrap_or(p.score.overall);
        let performance_grade = normalized_module_grade(&normalized.normalized, "Performance")
            .unwrap_or_else(|| p.score.grade.label().to_string());
        let vitals = build_vitals_list(p, i18n);
        let desktop_viewport = normalized
            .raw_performance_desktop
            .map(|d| PerformanceViewport {
                score: d.score.overall,
                grade: d.score.grade.label().to_string(),
                vitals: build_vitals_list(d, i18n),
            });
        let mobile_viewport = Some(PerformanceViewport {
            score: p.score.overall,
            grade: p.score.grade.label().to_string(),
            vitals: vitals.clone(),
        });

        let en = i18n.locale() == "en";
        let mut additional = Vec::new();
        if let Some(heap) = p.vitals.js_heap_size {
            additional.push((
                "JS Heap".to_string(),
                format!("{:.1} MB", heap as f64 / 1_048_576.0),
            ));
        }
        if let Some(ref cw) = p.content_weight {
            additional.push((
                if en { "CO2e per view" } else { "CO2e pro View" }.to_string(),
                format!("{} g ({})", cw.carbon.format_grams(), cw.carbon.rating),
            ));
        }

        // Resource/DOM metrics with an established best-practice threshold.
        // Labelled "guideline/Richtwert", never "target": these are widely
        // cited heuristics, not a standard the page is measured against —
        // rated the same way the Core Web Vitals are ("good"/"needs-
        // improvement"/"poor"), instead of sitting as plain, unrated text
        // indistinguishable from benign values. Thresholds: DOM node count
        // matches Lighthouse's own DOM-size audit guidance (flags past
        // ~1,500 nodes); load time and DOM-content-loaded follow commonly
        // cited page-load guidance (sub-3s / sub-1.8s "good").
        let mut resource_ratings = Vec::new();
        if let Some(nodes) = p.vitals.dom_nodes {
            resource_ratings.push((
                if en { "DOM nodes" } else { "DOM-Knoten" }.to_string(),
                localized_integer(nodes, en),
                rate_threshold(nodes as f64, 800.0, 1500.0).to_string(),
                if en {
                    "Guideline: max. 800"
                } else {
                    "Richtwert: max. 800"
                }
                .to_string(),
            ));
        }
        if let Some(load) = p.vitals.load_time {
            resource_ratings.push((
                if en { "Load time" } else { "Ladezeit" }.to_string(),
                format!("{} s", localized_decimal(load / 1000.0, 1, en)),
                rate_threshold(load, 3000.0, 6000.0).to_string(),
                if en {
                    "Guideline: max. 3.0 s"
                } else {
                    "Richtwert: max. 3,0 s"
                }
                .to_string(),
            ));
        }
        if let Some(dcl) = p.vitals.dom_content_loaded {
            resource_ratings.push((
                if en {
                    "DOM Content Loaded"
                } else {
                    "DOM-Inhalt geladen"
                }
                .to_string(),
                format!("{} s", localized_decimal(dcl / 1000.0, 1, en)),
                rate_threshold(dcl, 1800.0, 3600.0).to_string(),
                if en {
                    "Guideline: max. 1.8 s"
                } else {
                    "Richtwert: max. 1,8 s"
                }
                .to_string(),
            ));
        }

        let recommendations = derive_performance_recommendations(i18n, p);

        let mut render_blocking_metrics = Vec::new();
        let mut render_blocking_suggestions = Vec::new();
        let mut has_render_blocking = false;

        if let Some(ref rb) = p.render_blocking {
            if rb.has_blocking() || rb.third_party_bytes > 100_000 {
                has_render_blocking = true;
                render_blocking_metrics.push((
                    "Blocking Scripts".to_string(),
                    rb.blocking_scripts.len().to_string(),
                ));
                render_blocking_metrics.push((
                    "Blocking CSS".to_string(),
                    rb.blocking_css.len().to_string(),
                ));
                if rb.blocking_transfer_bytes > 0 {
                    render_blocking_metrics.push((
                        "Blocking Transfer".to_string(),
                        format!("{:.1} KB", rb.blocking_transfer_bytes as f64 / 1024.0),
                    ));
                }
                if rb.third_party_bytes > 0 {
                    render_blocking_metrics.push((
                        "Third-Party".to_string(),
                        format!(
                            "{:.1} KB ({} Domains)",
                            rb.third_party_bytes as f64 / 1024.0,
                            rb.third_party_origin_count
                        ),
                    ));
                }
                if rb.first_party_bytes > 0 {
                    render_blocking_metrics.push((
                        "First-Party".to_string(),
                        format!("{:.1} KB", rb.first_party_bytes as f64 / 1024.0),
                    ));
                }
                // Re-derive localized suggestions from the structured counts
                // (rb.suggestions is canonical English for the JSON, #406).
                render_blocking_suggestions = crate::performance::render_blocking_suggestions(
                    rb.blocking_scripts.len(),
                    rb.blocking_css.len(),
                    rb.third_party_bytes,
                    rb.third_party_origin_count,
                    locale == "en",
                );
            }
        }

        // If CWV are all good but score is below 85, explain the gap
        let cwv_all_good = p.vitals.lcp.as_ref().is_none_or(|v| v.rating == "good")
            && p.vitals.fcp.as_ref().is_none_or(|v| v.rating == "good")
            && p.vitals.cls.as_ref().is_none_or(|v| v.rating == "good");

        // When render-blocking resources exist but all vitals are good, clarify that
        // they had no measured impact on this run (fast server / warm cache).
        let en = locale == "en";
        if has_render_blocking && cwv_all_good && !render_blocking_suggestions.is_empty() {
            render_blocking_suggestions.push(if en {
                "No measurable impact on the measured vitals — still worth fixing preventively, \
                 since slow connections or cold caches can be affected more severely."
                    .to_string()
            } else {
                "Kein messbarer Einfluss auf die gemessenen Vitals — trotzdem vorbeugend beheben, \
                 da langsame Verbindungen oder kalte Caches stärker betroffen sein können."
                    .to_string()
            });
        }
        let base_perf = module_interpretation(&normalized.normalized, "performance", locale);
        // Decision + wording live in the domain layer (#406); the builder only
        // supplies the viewport flags and the worst throttled LCP.
        let score_below_excellent = performance_score < 85;
        let perf_interpretation = performance_gap_text(
            base_perf,
            p,
            cwv_all_good,
            score_below_excellent,
            has_render_blocking,
            en,
        );
        let throttled_lcp_max = normalized
            .raw_throttled_performance
            .iter()
            .filter_map(|t| t.lcp_ms)
            .fold(0.0_f64, f64::max);
        let mut perf_interpretation =
            append_performance_qualifiers_text(perf_interpretation, p, throttled_lcp_max, en);

        let mut capping_warnings = Vec::new();
        if p.score.is_capped == Some(true) {
            let cap = p.score.overall;
            let mut size_cap = 100u32;
            let mut js_cap = 100u32;
            let mut req_cap = 100u32;
            let mut dom_cap = 100u32;

            if let Some(ref cw) = p.content_weight {
                if cw.total_bytes > 10_000_000 {
                    size_cap = 39;
                } else if cw.total_bytes > 5_000_000 {
                    size_cap = 59;
                } else if cw.total_bytes > 3_000_000 {
                    size_cap = 74;
                }

                if cw.breakdown.javascript.bytes > 3_000_000 {
                    js_cap = 59;
                } else if cw.breakdown.javascript.bytes > 1_500_000 {
                    js_cap = 74;
                }

                if cw.request_count > 120 {
                    req_cap = 74;
                }
            }
            if let Some(nodes) = p.vitals.dom_nodes {
                if nodes > 3000 {
                    dom_cap = 59;
                } else if nodes > 2000 {
                    dom_cap = 74;
                }
            }

            let min_computed = size_cap.min(js_cap).min(req_cap).min(dom_cap);

            if let Some(ref cw) = p.content_weight {
                if size_cap == min_computed && size_cap < 100 {
                    let mb_str = format!("{:.1}", cw.total_bytes as f64 / 1_000_000.0);
                    capping_warnings.push(i18n.t_args(
                        "perf-capped-size",
                        &[("size", mb_str), ("cap", cap.to_string())],
                    ));
                }
                if js_cap == min_computed && js_cap < 100 {
                    let mb_str =
                        format!("{:.1}", cw.breakdown.javascript.bytes as f64 / 1_000_000.0);
                    capping_warnings.push(i18n.t_args(
                        "perf-capped-js",
                        &[("size", mb_str), ("cap", cap.to_string())],
                    ));
                }
                if req_cap == min_computed && req_cap < 100 {
                    capping_warnings.push(i18n.t_args(
                        "perf-capped-requests",
                        &[
                            ("count", cw.request_count.to_string()),
                            ("cap", cap.to_string()),
                        ],
                    ));
                }
            }
            if let Some(nodes) = p.vitals.dom_nodes {
                if dom_cap == min_computed && dom_cap < 100 {
                    capping_warnings.push(i18n.t_args(
                        "perf-capped-dom",
                        &[("nodes", nodes.to_string()), ("cap", cap.to_string())],
                    ));
                }
            }
        }

        if !capping_warnings.is_empty() {
            if !perf_interpretation.is_empty() {
                perf_interpretation.push(' ');
            }
            perf_interpretation.push_str(&capping_warnings.join(" "));
        }

        let throttled_profiles: Vec<ThrottledPerfEntry> = normalized
            .raw_throttled_performance
            .iter()
            .map(|t| ThrottledPerfEntry {
                profile_name: format!("{:?}", t.profile),
                lcp: t
                    .lcp_ms
                    .map(|v| format!("{:.0} ms", v))
                    .unwrap_or_else(|| "\u{2014}".to_string()),
                tbt: t
                    .tbt_ms
                    .map(|v| format!("{:.0} ms", v))
                    .unwrap_or_else(|| "\u{2014}".to_string()),
                cls: t
                    .cls
                    .map(|v| format!("{:.3}", v))
                    .unwrap_or_else(|| "\u{2014}".to_string()),
                score: t.score,
            })
            .collect();

        let cls_attribution = p
            .vitals
            .cls_attribution
            .iter()
            .take(5)
            .map(|s| {
                (
                    format!("{:.4}", s.value),
                    format!("{:.0}ms", s.start_time_ms),
                    s.sources
                        .first()
                        .map(|src| src.node.clone())
                        .unwrap_or_default(),
                )
            })
            .collect();

        let third_party = p.third_party.as_ref().map(|tp| {
            let page_total = p
                .content_weight
                .as_ref()
                .map(|cw| cw.transfer_bytes)
                .unwrap_or(0);
            ThirdPartyPresentation {
                origins: tp
                    .origins
                    .iter()
                    .take(10)
                    .map(|o| ThirdPartyOriginRow {
                        origin: o.origin.clone(),
                        provider: o.provider.clone(),
                        category: o.category.clone(),
                        request_count: o.request_count,
                        transfer_kb: o.transfer_bytes as f64 / 1024.0,
                        resource_kinds: o.resource_kinds.join(", "),
                    })
                    .collect(),
                total_origins: tp.total_origins,
                total_kb: tp.total_bytes as f64 / 1024.0,
                total_requests: tp.total_requests,
                is_significant: tp.is_significant(page_total),
                isolated_impact: tp
                    .isolated_impact
                    .iter()
                    .map(|i| ThirdPartyImpactRow {
                        origin: i.origin.clone(),
                        baseline_tbt_ms: i.baseline_tbt_ms,
                        without_script_tbt_ms: i.without_script_tbt_ms,
                        estimated_impact_ms: i.estimated_impact_ms,
                    })
                    .collect(),
            }
        });

        let critical_chain = p
            .critical_chain
            .as_ref()
            .map(|cc| CriticalChainPresentation {
                max_depth: cc.max_depth,
                critical_path_ms: format!("{:.0}ms", cc.critical_path_ms),
                critical_path_kb: format!("{:.1} KB", cc.critical_path_bytes as f64 / 1024.0),
                total_requests: cc.total_requests as usize,
            });

        let minification = p
            .minification
            .as_ref()
            .filter(|m| m.total_unminified_count > 0 || !m.legacy_scripts.is_empty())
            .map(|m| {
                let top_assets: Vec<(String, String, String)> = m
                    .unminified_scripts
                    .iter()
                    .chain(m.unminified_styles.iter())
                    .take(5)
                    .map(|a| {
                        (
                            truncate_url(&a.url, 60),
                            a.kind.clone(),
                            format!(
                                "{} KB",
                                localized_decimal(a.savings_bytes as f64 / 1024.0, 1, en)
                            ),
                        )
                    })
                    .collect();
                let legacy_assets: Vec<(String, String, String)> = m
                    .legacy_scripts
                    .iter()
                    .take(5)
                    .map(|a| {
                        (
                            truncate_url(&a.url, 60),
                            a.signature.clone(),
                            format!(
                                "{} KB",
                                localized_decimal(a.wasted_bytes as f64 / 1024.0, 1, en)
                            ),
                        )
                    })
                    .collect();
                MinificationPresentation {
                    total_count: m.total_unminified_count as usize,
                    total_savings_kb: m.total_savings_bytes as f64 / 1024.0,
                    top_assets,
                    legacy_count: m.legacy_scripts.len(),
                    legacy_wasted_kb: m.total_legacy_wasted_bytes as f64 / 1024.0,
                    legacy_assets,
                }
            });

        let coverage = p.coverage.as_ref().map(|cov| CoveragePresentation {
            js_used_pct: Some(cov.unused_js.used_pct),
            js_unused_kb: Some(cov.unused_js.unused_bytes as f64 / 1024.0),
            css_used_pct: cov.unused_css.used_pct,
            css_total_rules: Some(cov.unused_css.total_rules),
            css_used_rules: Some(cov.unused_css.used_rules),
            duplicate_assets: cov
                .duplicate_assets
                .iter()
                .map(|g| DuplicateAssetRow {
                    kind: g.kind.clone(),
                    kb: g.bytes as f64 / 1024.0,
                    urls: g.urls.iter().map(|u| truncate_url(u, 60)).collect(),
                })
                .collect(),
        });

        let animations = p
            .animations
            .as_ref()
            .filter(|a| a.total_count > 0)
            .map(|a| {
                let findings: Vec<(String, String, String)> = a
                    .findings
                    .iter()
                    .take(10)
                    .map(|f| {
                        (
                            f.kind.clone(),
                            f.property.clone(),
                            truncate_url(&f.source, 60),
                        )
                    })
                    .collect();
                AnimationPresentation {
                    total_count: a.total_count as usize,
                    affected_properties: a.affected_properties.clone(),
                    findings,
                }
            });

        PerformancePresentation {
            score: performance_score,
            grade: performance_grade,
            interpretation: perf_interpretation,
            vitals,
            desktop: desktop_viewport,
            mobile: mobile_viewport,
            additional_metrics: additional,
            resource_ratings,
            recommendations,
            render_blocking_metrics,
            render_blocking_suggestions,
            has_render_blocking,
            throttled_profiles,
            cls_attribution,
            third_party,
            critical_chain,
            minification,
            coverage,
            animations,
            measurement_warnings: p.measurement_warnings.clone(),
        }
    })
}
