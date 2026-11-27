use crate::audit::normalized::AuditContext;
use crate::audit::performance_interpretation::{
    append_performance_qualifiers_text, lab_lcp_cls_good, performance_gap_text,
};
use crate::i18n::I18n;
use crate::output::localized::is_english;
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

use crate::i18n::number::decimal as localized_decimal;

fn localized_integer(value: i64, en: bool) -> String {
    crate::i18n::number::integer(value, en)
}

/// The sentence on how many runs the headline vitals are the median of and
/// whether the LCP agreed between them (#1231). Without a spread on the LCP
/// there is one measurement and nothing to compare.
fn run_note(i18n: &I18n, p: &crate::audit::PerformanceResults, en: bool) -> String {
    use crate::audit::MeasurementStability;
    let lcp_spread = p.vitals.lcp.as_ref().and_then(|m| m.spread.as_ref());
    match (p.stability, lcp_spread) {
        (MeasurementStability::SingleRun, _) | (_, None) => i18n.t("perf-runs-single"),
        (stability, Some(spread)) => {
            let key = if stability == MeasurementStability::Unstable {
                "perf-runs-multi-unstable"
            } else {
                "perf-runs-multi"
            };
            i18n.t_args(
                key,
                &[
                    ("runs", p.runs.to_string()),
                    ("min", localized_decimal(spread.min / 1000.0, 1, en)),
                    ("max", localized_decimal(spread.max / 1000.0, 1, en)),
                ],
            )
        }
    }
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

        let en = is_english(i18n);
        let mut additional = Vec::new();
        if let Some(heap) = p.vitals.js_heap_size {
            additional.push((
                "JS Heap".to_string(),
                format!("{} MB", localized_decimal(heap as f64 / 1_048_576.0, 1, en)),
            ));
        }
        if let Some(ref cw) = p.content_weight {
            additional.push((
                i18n.t("report-builder-perf-co2e"),
                format!(
                    "{} g ({})",
                    if en {
                        cw.carbon.format_grams()
                    } else {
                        cw.carbon.format_grams().replace('.', ",")
                    },
                    cw.carbon.rating
                ),
            ));
        }

        // Resource/DOM metrics with an established best-practice threshold.
        // Labelled "guideline/Richtwert", never "target": these are widely
        // cited heuristics, not a standard the page is measured against —
        // rated the same way the Core Web Vitals are ("good"/"needs-
        // improvement"/"poor"), instead of sitting as plain, unrated text
        // indistinguishable from benign values. Thresholds: the DOM element
        // count matches Lighthouse's own DOM-size audit (800 / 1,500
        // elements, #1254; the node count is not compared); load time and DOM-content-loaded follow commonly
        // cited page-load guidance (sub-3s / sub-1.8s "good").
        let mut resource_ratings = Vec::new();
        if let Some(elements) = p.vitals.dom_elements {
            resource_ratings.push((
                i18n.t("report-builder-perf-dom-elements"),
                localized_integer(elements, en),
                rate_threshold(elements as f64, 800.0, 1500.0).to_string(),
                i18n.t("report-builder-perf-guideline-dom-elements"),
            ));
        }
        if let Some(load) = p.vitals.load_time {
            resource_ratings.push((
                i18n.t("report-builder-perf-load-time"),
                format!("{} s", localized_decimal(load / 1000.0, 1, en)),
                rate_threshold(load, 3000.0, 6000.0).to_string(),
                i18n.t("report-builder-perf-guideline-load-time"),
            ));
        }
        if let Some(dcl) = p.vitals.dom_content_loaded {
            resource_ratings.push((
                i18n.t("report-builder-perf-dom-content-loaded"),
                format!("{} s", localized_decimal(dcl / 1000.0, 1, en)),
                rate_threshold(dcl, 1800.0, 3600.0).to_string(),
                i18n.t("report-builder-perf-guideline-dom-content-loaded"),
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
                    i18n.t("report-builder-perf-blocking-scripts"),
                    rb.blocking_scripts.len().to_string(),
                ));
                render_blocking_metrics.push((
                    i18n.t("report-builder-perf-blocking-css"),
                    rb.blocking_css.len().to_string(),
                ));
                if rb.blocking_transfer_bytes > 0 {
                    render_blocking_metrics.push((
                        i18n.t("report-builder-perf-blocking-transfer"),
                        format!(
                            "{} KB",
                            localized_decimal(rb.blocking_transfer_bytes as f64 / 1024.0, 1, en)
                        ),
                    ));
                }
                if rb.third_party_bytes > 0 {
                    render_blocking_metrics.push((
                        i18n.t("report-builder-perf-third-party"),
                        format!(
                            "{} KB ({} Domains)",
                            localized_decimal(rb.third_party_bytes as f64 / 1024.0, 1, en),
                            rb.third_party_origin_count
                        ),
                    ));
                }
                if rb.first_party_bytes > 0 {
                    render_blocking_metrics.push((
                        i18n.t("report-builder-perf-first-party"),
                        format!(
                            "{} KB",
                            localized_decimal(rb.first_party_bytes as f64 / 1024.0, 1, en)
                        ),
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

        // If lab LCP and CLS are both measured and good but the score is below 85,
        // explain the gap. A missing metric is unknown, not good.
        let lab_lcp_cls_good = lab_lcp_cls_good(p);

        // When render-blocking resources exist but all vitals are good, clarify that
        // they had no measured impact on this run (fast server / warm cache).
        let en = locale == "en";
        if has_render_blocking && lab_lcp_cls_good && !render_blocking_suggestions.is_empty() {
            render_blocking_suggestions.push(i18n.t("report-builder-perf-no-measurable-impact"));
        }
        let base_perf = module_interpretation(&normalized.normalized, "performance", locale);
        // Decision + wording live in the domain layer (#406); the builder only
        // supplies the viewport flags and the worst throttled LCP.
        let score_below_excellent = performance_score < 85;
        let perf_interpretation = performance_gap_text(
            base_perf,
            p,
            lab_lcp_cls_good,
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
            if let Some(elements) = p.vitals.dom_elements {
                if elements > 3000 {
                    dom_cap = 59;
                } else if elements > 2000 {
                    dom_cap = 74;
                }
            }

            let min_computed = size_cap.min(js_cap).min(req_cap).min(dom_cap);

            if let Some(ref cw) = p.content_weight {
                if size_cap == min_computed && size_cap < 100 {
                    let mb_str = localized_decimal(cw.total_bytes as f64 / 1_000_000.0, 1, en);
                    capping_warnings.push(i18n.t_args(
                        "perf-capped-size",
                        &[("size", mb_str), ("cap", cap.to_string())],
                    ));
                }
                if js_cap == min_computed && js_cap < 100 {
                    let mb_str = localized_decimal(
                        cw.breakdown.javascript.bytes as f64 / 1_000_000.0,
                        1,
                        en,
                    );
                    capping_warnings.push(i18n.t_args(
                        "perf-capped-js",
                        &[("size", mb_str), ("cap", cap.to_string())],
                    ));
                }
                if req_cap == min_computed && req_cap < 100 {
                    capping_warnings.push(i18n.t_args(
                        "perf-capped-requests",
                        &[
                            ("count", localized_integer(cw.request_count as i64, en)),
                            ("cap", cap.to_string()),
                        ],
                    ));
                }
            }
            if let Some(elements) = p.vitals.dom_elements {
                if dom_cap == min_computed && dom_cap < 100 {
                    capping_warnings.push(i18n.t_args(
                        "perf-capped-dom",
                        &[
                            ("elements", localized_integer(elements, en)),
                            ("cap", cap.to_string()),
                        ],
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
                    .map(|v| localized_decimal(v, 3, en))
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
                    localized_decimal(s.value, 4, en),
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
                critical_path_kb: format!(
                    "{} KB",
                    localized_decimal(cc.critical_path_bytes as f64 / 1024.0, 1, en)
                ),
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
            js_used_pct: cov.unused_js.used_pct,
            js_unused_kb: Some(cov.unused_js.unused_bytes as f64 / 1024.0),
            css_used_pct: cov.unused_css.used_pct,
            css_total_rules: Some(cov.unused_css.total_rules),
            css_used_rules: Some(cov.unused_css.used_rules),
            css_partial: cov.unused_css.measurement == "partial",
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
            run_note: run_note(i18n, p, en),
            tti_note: (p.vitals.tti.is_some()
                || normalized
                    .raw_performance_desktop
                    .is_some_and(|d| d.vitals.tti.is_some()))
            .then(|| i18n.t("perf-tti-derived-note")),
            dom_note: p.vitals.dom_elements.map(|_| {
                i18n.t_args(
                    "perf-dom-elements-note",
                    &[(
                        "nodes",
                        p.vitals
                            .dom_nodes
                            .map(|n| localized_integer(n, en))
                            .unwrap_or_else(|| "n/a".to_string()),
                    )],
                )
            }),
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::{MeasurementStability, MetricSpread, PerformanceResults};
    use crate::performance::{calculate_performance_score, VitalMetric, WebVitals};

    fn results(
        stability: MeasurementStability,
        runs: u32,
        lcp: Option<(f64, f64)>,
    ) -> PerformanceResults {
        let mut lcp_metric = VitalMetric::new(2000.0, 2500.0, 4000.0);
        lcp_metric.spread = lcp.map(|(min, max)| MetricSpread {
            min,
            median: 2000.0,
            max,
        });
        let vitals = WebVitals {
            lcp: Some(lcp_metric),
            tti: Some(VitalMetric::new(3000.0, 3800.0, 7300.0).estimated()),
            dom_elements: Some(900),
            ..Default::default()
        };
        let score = calculate_performance_score(&vitals, None);
        let mut p = PerformanceResults::new(vitals, score);
        p.runs = runs;
        p.stability = stability;
        p
    }

    fn note(locale: &str, p: &PerformanceResults) -> String {
        let i18n = I18n::new(locale).unwrap();
        run_note(&i18n, p, locale == "en")
    }

    #[test]
    fn the_note_names_the_runs_and_the_lcp_range() {
        let stable = results(MeasurementStability::Stable, 3, Some((1800.0, 2200.0)));
        let en = note("en", &stable);
        assert!(en.contains("median of 3 runs"), "{en}");
        assert!(en.contains("between 1.8 s and 2.2 s"), "{en}");
        assert!(!en.contains("unstable"), "{en}");
        let de = note("de", &stable);
        assert!(de.contains("Median aus 3 Messungen"), "{de}");
        assert!(de.contains("zwischen 1,8 s und 2,2 s"), "{de}");
    }

    #[test]
    fn an_unstable_measurement_is_called_unstable() {
        let unstable = results(MeasurementStability::Unstable, 3, Some((1000.0, 3500.0)));
        let en = note("en", &unstable);
        assert!(en.contains("between 1.0 s and 3.5 s"), "{en}");
        assert!(en.contains("unstable"), "{en}");
        assert!(note("de", &unstable).contains("instabil"));
    }

    #[test]
    fn a_single_run_says_it_was_not_stability_checked() {
        for p in [
            results(MeasurementStability::SingleRun, 1, None),
            // A spread left over on a single run is not shown.
            results(MeasurementStability::SingleRun, 1, Some((1.0, 2.0))),
        ] {
            let en = note("en", &p);
            assert!(en.contains("single run, not stability-checked"), "{en}");
            assert!(note("de", &p).contains("nicht auf Stabilität geprüft"));
        }
    }

    /// The English texts carry no German umlauts or ß.
    #[test]
    fn the_english_texts_are_english() {
        let i18n = I18n::new("en").unwrap();
        let texts = [
            note(
                "en",
                &results(MeasurementStability::Stable, 3, Some((1800.0, 2200.0))),
            ),
            note(
                "en",
                &results(MeasurementStability::Unstable, 3, Some((1000.0, 3500.0))),
            ),
            note("en", &results(MeasurementStability::SingleRun, 1, None)),
            i18n.t("perf-tti-derived-note"),
            i18n.t_args("perf-dom-elements-note", &[("nodes", "2,826".to_string())]),
            i18n.t("report-builder-perf-guideline-dom-elements"),
        ];
        for text in texts {
            assert!(!text.chars().any(|c| "äöüÄÖÜß".contains(c)), "{text}");
            assert!(!text.starts_with("perf-"), "missing message: {text}");
        }
    }

    #[test]
    fn the_tti_and_dom_notes_say_what_is_derived_and_what_is_counted() {
        let en = I18n::new("en").unwrap();
        assert!(en.t("perf-tti-derived-note").contains("long tasks"));
        assert!(en
            .t("perf-tti-derived-note")
            .contains("not an independent measurement"));
        let dom = en.t_args("perf-dom-elements-note", &[("nodes", "2,826".to_string())]);
        assert!(dom.contains("2,826 nodes") && dom.contains("elements only"));
        assert!(dom.contains("does not enter the rating"));
        let de = I18n::new("de").unwrap();
        assert!(de.t("perf-tti-derived-note").contains("Long Tasks"));
        let dom_de = de.t_args("perf-dom-elements-note", &[("nodes", "2.826".to_string())]);
        assert!(dom_de.contains("2.826 Knoten") && dom_de.contains("nur Elemente"));
    }
}
