use crate::audit::normalized::{AuditContext, NormalizedReport};
use crate::i18n::I18n;
use crate::output::report_model::{
    AnimationPresentation, CoveragePresentation, CriticalChainPresentation, DarkModePresentation,
    FrictionPointPresentation, ImageEfficiencyPresentation, JourneyDimensionPresentation,
    JourneyPresentation, MinificationPresentation, MobilePresentation, ModuleDetailsBlock,
    OversizedImageRow, PerformancePresentation, PerformanceViewport, RobotsPresentation,
    SecurityPresentation, SeoPresentation, SeoProfilePresentation, SignalDetails,
    ThirdPartyOriginRow, ThirdPartyPresentation, ThrottledPerfEntry, UxDimensionPresentation,
    UxIssuePresentation, UxPresentation,
};
use crate::output::search_experience::build_search_experience;

use super::super::helpers::{truncate_list, truncate_url_list, yes_no};
use super::super::modules::{
    build_tracking_summary_text, build_vitals_list, derive_performance_recommendations,
    derive_security_recommendations,
};
use super::super::seo::{
    build_seo_interpretation, page_profile_optimization_note, summarize_page_profile,
};
use super::serp::{build_page_health_presentation, build_serp_presentation};
use crate::util::truncate_url;

/// Read the pre-computed interpretation for a module. Falls back to an empty
/// string if the interpretation layer was not populated (should not happen
/// after a full normalize() call).
fn module_interpretation(normalized: &NormalizedReport, module: &str, locale: &str) -> String {
    normalized
        .interpretation
        .as_ref()
        .and_then(|i| i.per_module.get(module))
        .map(|t| t.for_locale(locale).to_string())
        .unwrap_or_default()
}

/// Joins phrases into a natural list ("a, b and c" / "a, b und c").
fn join_phrases(items: &[String], en: bool) -> String {
    match items.len() {
        0 => String::new(),
        1 => items[0].clone(),
        _ => {
            let (last, head) = items.split_last().unwrap();
            format!(
                "{} {} {}",
                head.join(", "),
                if en { "and" } else { "und" },
                last
            )
        }
    }
}

/// Appends threshold-based qualifiers to the performance interpretation so that
/// concrete weak metrics (DOM complexity, throttled LCP, render-blocking
/// resources, unminified weight, non-composited animations) are named instead
/// of left to the score band alone, while genuinely strong metrics (CLS, TBT)
/// are acknowledged as contrast (#367).
fn append_performance_qualifiers(
    base: String,
    p: &crate::PerformanceResults,
    normalized: &AuditContext,
    en: bool,
) -> String {
    let mut critical: Vec<String> = Vec::new();

    if let Some(nodes) = p.vitals.dom_nodes.filter(|n| *n > 3000) {
        critical.push(if en {
            format!("high DOM complexity ({nodes} nodes)")
        } else {
            format!("die hohe DOM-Komplexität ({nodes} Knoten)")
        });
    }

    // Worst LCP measured under network throttling (e.g. Slow 3G).
    let throttled_lcp = normalized
        .raw_throttled_performance
        .iter()
        .filter_map(|t| t.lcp_ms)
        .fold(0.0_f64, f64::max);
    if throttled_lcp > 4000.0 {
        critical.push(if en {
            format!("a late largest contentful paint under throttling ({throttled_lcp:.0} ms)")
        } else {
            format!(
                "der spät erscheinende Hauptinhalt unter Drosselung ({throttled_lcp:.0} ms LCP)"
            )
        });
    }

    let rb_count = p
        .render_blocking
        .as_ref()
        .map(|rb| rb.blocking_scripts.len() + rb.blocking_css.len())
        .unwrap_or(0);
    if rb_count > 0 {
        critical.push(if en {
            format!("{rb_count} render-blocking resources")
        } else {
            format!("{rb_count} render-blockierende Ressourcen")
        });
    }

    let unminified_kb = p
        .minification
        .as_ref()
        .map(|m| m.total_savings_bytes as f64 / 1024.0)
        .unwrap_or(0.0);
    if unminified_kb > 100.0 {
        critical.push(if en {
            format!("unminified assets (~{unminified_kb:.0} KB savings)")
        } else {
            format!("unminifizierte Assets (~{unminified_kb:.0} KB Einsparpotenzial)")
        });
    }

    let anim_count = p.animations.as_ref().map(|a| a.total_count).unwrap_or(0);
    if anim_count > 10 {
        critical.push(if en {
            format!("{anim_count} non-composited animations")
        } else {
            format!("{anim_count} nicht-composited Animationen")
        });
    }

    if critical.is_empty() {
        return base;
    }

    let mut positive: Vec<String> = Vec::new();
    if p.vitals.cls.as_ref().is_some_and(|v| v.rating == "good") {
        positive.push(
            if en {
                "layout stability (CLS)"
            } else {
                "Layout-Stabilität (CLS)"
            }
            .to_string(),
        );
    }
    if p.vitals.tbt.as_ref().is_some_and(|v| v.rating == "good") {
        positive.push(
            if en {
                "main-thread blocking (TBT)"
            } else {
                "Hauptthread-Blockierung (TBT)"
            }
            .to_string(),
        );
    }

    let mut out = base;
    if !out.is_empty() {
        out.push(' ');
    }
    if en {
        out.push_str(&format!(
            "Critical here are {}.",
            join_phrases(&critical, en)
        ));
        if !positive.is_empty() {
            let verb = if positive.len() == 1 {
                "remains"
            } else {
                "remain"
            };
            out.push_str(&format!(
                " In contrast, {} {verb} unobtrusive.",
                join_phrases(&positive, en)
            ));
        }
    } else {
        out.push_str(&format!(
            "Kritisch sind hier {}.",
            join_phrases(&critical, en)
        ));
        if !positive.is_empty() {
            let verb = if positive.len() == 1 {
                "bleibt"
            } else {
                "bleiben"
            };
            out.push_str(&format!(
                " Im Kontrast dazu {verb} {} unauffällig.",
                join_phrases(&positive, en)
            ));
        }
    }
    out
}

fn build_performance_details(
    normalized: &AuditContext,
    i18n: &I18n,
) -> Option<PerformancePresentation> {
    let locale = i18n.locale();
    normalized.raw_performance.as_ref().map(|p| {
        let performance_score =
            normalized_module_score(normalized, "Performance").unwrap_or(p.score.overall);
        let performance_grade = normalized_module_grade(normalized, "Performance")
            .unwrap_or_else(|| p.score.grade.label().to_string());
        let vitals = build_vitals_list(p, i18n);
        let desktop_viewport =
            normalized
                .raw_performance_desktop
                .as_ref()
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

        let mut additional = Vec::new();
        if let Some(nodes) = p.vitals.dom_nodes {
            additional.push(("DOM-Knoten".to_string(), nodes.to_string()));
        }
        if let Some(heap) = p.vitals.js_heap_size {
            additional.push((
                "JS Heap".to_string(),
                format!("{:.1} MB", heap as f64 / 1_048_576.0),
            ));
        }
        if let Some(load) = p.vitals.load_time {
            additional.push(("Ladezeit".to_string(), format!("{:.0}ms", load)));
        }
        if let Some(dcl) = p.vitals.dom_content_loaded {
            additional.push(("DOM Content Loaded".to_string(), format!("{:.0}ms", dcl)));
        }
        if let Some(ref cw) = p.content_weight {
            additional.push((
                "CO2e pro View".to_string(),
                format!(
                    "{:.2} g ({})",
                    cw.carbon.grams_co2e_per_view, cw.carbon.rating
                ),
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
                render_blocking_suggestions = rb.suggestions.clone();
            }
        }

        // If CWV are all good but score is below 85, explain the gap
        let cwv_all_good = p.vitals.lcp.as_ref().is_none_or(|v| v.rating == "good")
            && p.vitals.fcp.as_ref().is_none_or(|v| v.rating == "good")
            && p.vitals.cls.as_ref().is_none_or(|v| v.rating == "good");

        // When render-blocking resources exist but all vitals are good, clarify that
        // they had no measured impact on this run (fast server / warm cache).
        if has_render_blocking && cwv_all_good && !render_blocking_suggestions.is_empty() {
            render_blocking_suggestions.push(
                "Kein messbarer Einfluss auf die gemessenen Vitals — trotzdem vorbeugend beheben, \
                 da langsame Verbindungen oder kalte Caches stärker betroffen sein können."
                    .to_string(),
            );
        }
        let en = locale == "en";
        let base_perf = module_interpretation(normalized, "performance", locale);
        // When all core vitals are green but score < 85, append the cause for the gap.
        let score_below_excellent = performance_score < 85;
        let perf_interpretation = if cwv_all_good && score_below_excellent {
            let mut reasons = Vec::new();
            if p.vitals.dom_nodes.is_some_and(|n| n > 1500) {
                reasons.push(if en { "DOM size" } else { "DOM-Größe" });
            }
            if has_render_blocking {
                reasons.push(if en {
                    "render-blocking resources"
                } else {
                    "Render-blockierende Ressourcen"
                });
            }
            if p.vitals.tbt.as_ref().is_some_and(|v| v.rating != "good") {
                reasons.push("Total Blocking Time");
            }
            if reasons.is_empty() {
                base_perf
            } else if en {
                format!(
                    "{} Score reduced by {} although Core Web Vitals are in the green.",
                    base_perf,
                    reasons.join(", ")
                )
            } else {
                format!(
                    "{} Score durch {} reduziert, obwohl Core Web Vitals im grünen Bereich liegen.",
                    base_perf,
                    reasons.join(", ")
                )
            }
        } else {
            base_perf
        };
        let mut perf_interpretation =
            append_performance_qualifiers(perf_interpretation, p, normalized, en);

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
                        request_count: o.request_count,
                        transfer_kb: o.transfer_bytes as f64 / 1024.0,
                        resource_kinds: o.resource_kinds.join(", "),
                    })
                    .collect(),
                total_origins: tp.total_origins,
                total_kb: tp.total_bytes as f64 / 1024.0,
                total_requests: tp.total_requests,
                is_significant: tp.is_significant(page_total),
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
                            format!("{:.1} KB", a.savings_bytes as f64 / 1024.0),
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
                            format!("{:.1} KB", a.wasted_bytes as f64 / 1024.0),
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

fn build_seo_details(normalized: &AuditContext, i18n: &I18n) -> Option<SeoPresentation> {
    let locale = i18n.locale();
    let en = locale == "en";
    normalized.raw_seo.as_ref().map(|s| {
        let seo_score = normalized_module_score(normalized, "SEO").unwrap_or(s.score);
        let mut meta_tags = Vec::new();
        if let Some(ref title) = s.meta.title {
            meta_tags.push(("Titel".to_string(), title.clone()));
        }
        if let Some(ref desc) = s.meta.description {
            meta_tags.push(("Beschreibung".to_string(), desc.clone()));
        }
        if let Some(ref viewport) = s.meta.viewport {
            meta_tags.push(("Viewport".to_string(), viewport.clone()));
        }

        let meta_issues: Vec<(String, crate::wcag::Severity, String)> = s
            .meta_issues
            .iter()
            .map(|i| (i.field.clone(), i.severity, i.message.clone()))
            .collect();

        let profile = s.content_profile.as_ref().map(|cp| {
            use crate::seo::profile::SchemaExtracted;

            let schema_rows: Vec<(String, String, String)> = cp
                .schema_inventory
                .schemas
                .iter()
                .map(|sd| {
                    let detail = match &sd.extracted {
                        SchemaExtracted::Organization { name, .. } => {
                            name.clone().unwrap_or_default()
                        }
                        SchemaExtracted::LocalBusiness { name, address, .. } => format!(
                            "{}{}",
                            name.as_deref().unwrap_or(""),
                            address
                                .as_ref()
                                .map(|a| format!(", {}", a))
                                .unwrap_or_default()
                        ),
                        SchemaExtracted::Article {
                            headline, author, ..
                        } => format!(
                            "{}{}",
                            headline.as_deref().unwrap_or(""),
                            author
                                .as_ref()
                                .map(|a| format!(" ({})", a))
                                .unwrap_or_default()
                        ),
                        SchemaExtracted::FAQPage { question_count, .. } => {
                            format!("{} Fragen", question_count)
                        }
                        SchemaExtracted::Product {
                            name,
                            price,
                            currency,
                            ..
                        } => format!(
                            "{}{}",
                            name.as_deref().unwrap_or(""),
                            price
                                .as_ref()
                                .map(|p| format!(" — {} {}", p, currency.as_deref().unwrap_or("")))
                                .unwrap_or_default()
                        ),
                        SchemaExtracted::WebSite {
                            name,
                            has_search_action,
                            ..
                        } => format!(
                            "{}{}",
                            name.as_deref().unwrap_or(""),
                            if *has_search_action { " (Suche)" } else { "" }
                        ),
                        SchemaExtracted::WebPage {
                            name,
                            author,
                            in_language,
                            ..
                        } => format!(
                            "{}{}{}",
                            name.as_deref().unwrap_or(""),
                            author
                                .as_ref()
                                .map(|a| format!(" ({})", a))
                                .unwrap_or_default(),
                            in_language
                                .as_ref()
                                .map(|lang| format!(" · {}", lang))
                                .unwrap_or_default()
                        ),
                        SchemaExtracted::Service {
                            name,
                            address,
                            area_served_count,
                            ..
                        } => format!(
                            "{}{}{}",
                            name.as_deref().unwrap_or(""),
                            address
                                .as_ref()
                                .map(|a| format!(" — {}", a))
                                .unwrap_or_default(),
                            if *area_served_count > 0 {
                                format!(" · {} Regionen", area_served_count)
                            } else {
                                String::new()
                            }
                        ),
                        SchemaExtracted::BreadcrumbList { item_count } => {
                            format!("{} Ebenen", item_count)
                        }
                        SchemaExtracted::Generic { key_fields } => key_fields
                            .first()
                            .map(|(k, v)| format!("{}: {}", k, v))
                            .unwrap_or_default(),
                    };
                    (
                        sd.schema_type.clone(),
                        format!("{}%", sd.completeness_pct),
                        detail,
                    )
                })
                .collect();

            let signal_rows: Vec<(String, String, String)> = cp
                .signal_strength
                .categories
                .iter()
                .map(|cat| {
                    let rating = match cat.score_pct {
                        90..=100 => "Sehr gut",
                        67..=89 => "Gut",
                        34..=66 => "Teilweise",
                        1..=33 => "Minimal",
                        _ => "Fehlt",
                    };
                    (
                        cat.name.clone(),
                        format!("{}%", cat.score_pct),
                        rating.to_string(),
                    )
                })
                .collect();

            let signal_details: SignalDetails = cp
                .signal_strength
                .categories
                .iter()
                .map(|cat| {
                    let checks = cat
                        .checks
                        .iter()
                        .map(|c| {
                            (
                                c.label.clone(),
                                c.passed,
                                c.detail.clone().unwrap_or_default(),
                            )
                        })
                        .collect();
                    (cat.name.clone(), checks)
                })
                .collect();

            SeoProfilePresentation {
                identity_summary: cp.content_identity.summary.clone(),
                site_name: cp
                    .content_identity
                    .site_name
                    .clone()
                    .unwrap_or_else(|| "—".to_string()),
                content_type: cp.content_identity.content_type.clone(),
                language: cp
                    .content_identity
                    .language
                    .clone()
                    .unwrap_or_else(|| "—".to_string()),
                category_hints: cp.content_identity.category_hints.clone(),
                identity_facts: vec![
                    (
                        "Seitentitel".to_string(),
                        cp.content_identity
                            .site_name
                            .clone()
                            .unwrap_or_else(|| "—".to_string()),
                    ),
                    (
                        "Inhaltstyp".to_string(),
                        cp.content_identity.content_type.clone(),
                    ),
                    (
                        "Sprache".to_string(),
                        cp.content_identity
                            .language
                            .clone()
                            .unwrap_or_else(|| "—".to_string()),
                    ),
                    (
                        "Themenhinweise".to_string(),
                        if cp.content_identity.category_hints.is_empty() {
                            "Keine klaren Themenhinweise erkannt".to_string()
                        } else {
                            cp.content_identity.category_hints.join(", ")
                        },
                    ),
                ],
                page_type: cp.page_classification.primary_type.label(en).to_string(),
                page_attributes: cp.page_classification.attributes.clone(),
                content_depth_score: cp.page_classification.content_depth_score,
                structural_richness_score: cp.page_classification.structural_richness_score,
                media_text_balance_score: cp.page_classification.media_text_balance_score,
                intent_fit_score: cp.page_classification.intent_fit_score,
                page_profile_summary: summarize_page_profile(locale, cp),
                optimization_note: page_profile_optimization_note(locale, cp),
                page_profile_facts: vec![
                    (
                        "Seitentyp".to_string(),
                        cp.page_classification.primary_type.label(en).to_string(),
                    ),
                    (
                        "Merkmale".to_string(),
                        if cp.page_classification.attributes.is_empty() {
                            "Keine prägenden Merkmale erkannt".to_string()
                        } else {
                            format!("{}.", cp.page_classification.attributes.join(", "))
                        },
                    ),
                    ("Einordnung".to_string(), summarize_page_profile(locale, cp)),
                    (
                        "Empfehlung".to_string(),
                        page_profile_optimization_note(locale, cp),
                    ),
                ],
                schema_rows,
                schema_count: cp.schema_inventory.total_count,
                signal_rows,
                signal_overall_pct: cp.signal_strength.overall_pct,
                signal_details,
                maturity_level: cp.maturity.label(en).to_string(),
                maturity_description: cp.maturity.description(en).to_string(),
                maturity_techniques_used: cp.maturity_techniques,
                maturity_techniques_total: 13,
            }
        });

        SeoPresentation {
            score: seo_score,
            interpretation: build_seo_interpretation(locale, s),
            meta_tags,
            meta_issues,
            heading_summary: format!(
                "{} H1-Überschrift(en), {} Überschriften gesamt, {} Probleme",
                s.headings.h1_count,
                s.headings.total_count,
                s.headings.issues.len()
            ),
            social_summary: format!(
                "Open Graph: {}, Twitter Card: {}, Vollständigkeit: {}%",
                if s.social.open_graph.is_some() {
                    "vorhanden"
                } else {
                    "fehlt"
                },
                if s.social.twitter_card.is_some() {
                    "vorhanden"
                } else {
                    "fehlt"
                },
                s.social.completeness
            ),
            technical_summary: vec![
                ("HTTPS".to_string(), yes_no(locale, s.technical.https)),
                (
                    "Canonical".to_string(),
                    yes_no(locale, s.technical.has_canonical),
                ),
                (
                    "Sprachangabe".to_string(),
                    yes_no(locale, s.technical.has_lang),
                ),
                ("Wortanzahl".to_string(), s.technical.word_count.to_string()),
                (
                    "Interne Links".to_string(),
                    s.technical.internal_links.to_string(),
                ),
                (
                    "Externe Links".to_string(),
                    s.technical.external_links.to_string(),
                ),
                (
                    "Dofollow-Links".to_string(),
                    s.technical.dofollow_links.to_string(),
                ),
                (
                    "Nofollow-Links".to_string(),
                    s.technical.nofollow_links.to_string(),
                ),
            ],
            tracking_summary: vec![
                (
                    "Google Fonts (extern)".to_string(),
                    if s.technical.uses_remote_google_fonts {
                        format!(
                            "Ja ({})",
                            truncate_url_list(&s.technical.google_fonts_sources, 2, 48)
                        )
                    } else {
                        "Nein".to_string()
                    },
                ),
                (
                    "Tracking-Cookies".to_string(),
                    if s.technical.tracking_cookies.is_empty() {
                        "Keine erkannt".to_string()
                    } else {
                        format!(
                            "{} ({})",
                            s.technical.tracking_cookies.len(),
                            s.technical
                                .tracking_cookies
                                .iter()
                                .map(|c| c.name.clone())
                                .collect::<Vec<_>>()
                                .join(", ")
                        )
                    },
                ),
                (
                    "Tracking-Signale".to_string(),
                    if s.technical.tracking_signals.is_empty() {
                        "Keine erkannt".to_string()
                    } else {
                        truncate_list(&s.technical.tracking_signals, 3)
                    },
                ),
                (
                    "Zaraz".to_string(),
                    if s.technical.zaraz.detected {
                        format!("Erkannt ({})", truncate_list(&s.technical.zaraz.signals, 2))
                    } else {
                        "Nicht erkannt".to_string()
                    },
                ),
            ],
            tracking_summary_text: build_tracking_summary_text(i18n, &s.technical),
            profile,
            page_health: s
                .page_health
                .as_ref()
                .map(|p| build_page_health_presentation(locale, p)),
            serp: s
                .serp
                .as_ref()
                .map(|serp| build_serp_presentation(locale, serp)),
            robots: s.robots.as_ref().map(|r| {
                use crate::seo::BotClass;
                let bot_rows: Vec<(String, String, usize, usize, bool)> = r
                    .groups
                    .iter()
                    .map(|g| {
                        let fully_blocked = g.disallows.iter().any(|d| d == "/");
                        (
                            g.user_agent.clone(),
                            g.bot_class.label(en).to_string(),
                            g.allows.len(),
                            g.disallows.len(),
                            fully_blocked,
                        )
                    })
                    .collect();

                let blocked_ai_bots: Vec<String> = r
                    .groups
                    .iter()
                    .filter(|g| {
                        matches!(
                            g.bot_class,
                            BotClass::AiTraining
                                | BotClass::AiCitation
                                | BotClass::AiMixed
                                | BotClass::UnknownAi
                        ) && g.disallows.iter().any(|d| d == "/")
                    })
                    .map(|g| g.user_agent.clone())
                    .collect();

                RobotsPresentation {
                    error: r.error.clone(),
                    has_wildcard_disallow_all: r.has_wildcard_disallow_all,
                    blocks_ai_crawlers: r.blocks_ai_crawlers,
                    blocks_ai_citation: r.blocks_ai_citation,
                    inferred_policy: r.inferred_policy.clone(),
                    sitemaps: r.sitemaps.clone(),
                    crawl_delays: r.crawl_delays.clone(),
                    bot_rows,
                    blocked_ai_bots,
                    noindex_in_sitemap: r.noindex_in_sitemap,
                }
            }),
            image_efficiency: s
                .image_efficiency
                .as_ref()
                .filter(|ie| ie.total_images > 0)
                .map(|ie| ImageEfficiencyPresentation {
                    total_images: ie.total_images,
                    modern_format_pct: ie.modern_format_pct,
                    legacy_count: ie.legacy_format_count,
                    oversized: ie
                        .oversized_images
                        .iter()
                        .take(5)
                        .map(|o| OversizedImageRow {
                            src: truncate_url(&o.src, 60),
                            natural: format!("{}×{}", o.natural_width, o.natural_height),
                            display: format!("{}×{}", o.display_width, o.display_height),
                        })
                        .collect(),
                }),
            technical_issues: s
                .technical
                .issues
                .iter()
                .map(|i| {
                    (
                        i.issue_type.clone(),
                        i.message.clone(),
                        i.severity.label().to_string(),
                    )
                })
                .collect(),
        }
    })
}

fn build_security_details(normalized: &AuditContext, i18n: &I18n) -> Option<SecurityPresentation> {
    let locale = i18n.locale();
    normalized.raw_security.as_ref().map(|sec| {
        let security_score = normalized_module_score(normalized, "Security").unwrap_or(sec.score);
        let header_checks: Vec<(&str, &Option<String>)> = vec![
            (
                "Content-Security-Policy",
                &sec.headers.content_security_policy,
            ),
            (
                "Strict-Transport-Security",
                &sec.headers.strict_transport_security,
            ),
            (
                "X-Content-Type-Options",
                &sec.headers.x_content_type_options,
            ),
            ("X-Frame-Options", &sec.headers.x_frame_options),
            ("Referrer-Policy", &sec.headers.referrer_policy),
            ("Permissions-Policy", &sec.headers.permissions_policy),
            (
                "Cross-Origin-Opener-Policy",
                &sec.headers.cross_origin_opener_policy,
            ),
            (
                "Cross-Origin-Resource-Policy",
                &sec.headers.cross_origin_resource_policy,
            ),
        ];

        SecurityPresentation {
            score: security_score,
            grade: normalized_module_grade(normalized, "Security")
                .unwrap_or_else(|| sec.grade.clone()),
            interpretation: module_interpretation(normalized, "security", locale),
            headers: header_checks
                .iter()
                .map(|(name, value)| {
                    let (status, val) = match value {
                        Some(v) => ("Vorhanden".to_string(), truncate_url(v, 50)),
                        None => ("Fehlt".to_string(), "—".to_string()),
                    };
                    (name.to_string(), status, val)
                })
                .collect(),
            ssl_info: vec![
                ("HTTPS".to_string(), yes_no(locale, sec.ssl.https)),
                (
                    "Gültiges Zertifikat".to_string(),
                    yes_no(locale, sec.ssl.valid_certificate),
                ),
                ("HSTS".to_string(), yes_no(locale, sec.ssl.has_hsts)),
                (
                    "HSTS Max-Age".to_string(),
                    sec.ssl
                        .hsts_max_age
                        .map(|v| format!("{}s", v))
                        .unwrap_or_else(|| "—".to_string()),
                ),
                (
                    "Subdomains".to_string(),
                    yes_no(locale, sec.ssl.hsts_include_subdomains),
                ),
                ("Preload".to_string(), yes_no(locale, sec.ssl.hsts_preload)),
            ],
            issues: sec
                .issues
                .iter()
                .map(|i| (i.header.clone(), i.severity, i.message.clone()))
                .collect(),
            recommendations: derive_security_recommendations(i18n, sec),
            protection: sec
                .protection
                .services
                .iter()
                .map(|s| (s.name.clone(), s.kind.clone()))
                .collect(),
            has_waf: sec.protection.has_waf,
            has_cdn: sec.protection.has_cdn,
        }
    })
}

fn build_mobile_details(normalized: &AuditContext, i18n: &I18n) -> Option<MobilePresentation> {
    let locale = i18n.locale();
    normalized.raw_mobile.as_ref().map(|m| {
        let mobile_score = normalized_module_score(normalized, "Mobile").unwrap_or(m.score);
        let small_targets = m.touch_targets.small_targets;
        let en = locale == "en";
        let context_hint = if !m.touch_targets.small_by_context.is_empty() {
            let parts: Vec<String> = m
                .touch_targets
                .small_by_context
                .iter()
                .take(3)
                .map(|(ctx, count)| {
                    if en {
                        format!("{} in {}", count, ctx)
                    } else {
                        format!("{} im Bereich {}", count, ctx)
                    }
                })
                .collect();
            format!(" ({})", parts.join(", "))
        } else {
            String::new()
        };
        let base_mobile = module_interpretation(normalized, "mobile", locale);
        let mobile_interpretation = if small_targets >= 10 {
            // When context is known (e.g. "footer", "navigation"), qualify as locally scoped
            // so a high overall score paired with a large touch-target count reads correctly.
            let scope_note = if !context_hint.is_empty() {
                if en {
                    " — locally scoped"
                } else {
                    " — lokal begrenzt"
                }
            } else {
                ""
            };
            if en {
                format!(
                    "{} {} touch targets smaller than recommended (44×44 px){}{}.",
                    base_mobile, small_targets, context_hint, scope_note,
                )
            } else {
                format!(
                    "{} {} Touch-Targets kleiner als empfohlen (44×44 px){}{}.",
                    base_mobile, small_targets, context_hint, scope_note,
                )
            }
        } else {
            base_mobile
        };
        MobilePresentation {
            score: mobile_score,
            interpretation: mobile_interpretation,
            viewport: vec![
                (
                    "Viewport-Tag".to_string(),
                    yes_no(locale, m.viewport.has_viewport),
                ),
                (
                    "device-width".to_string(),
                    yes_no(locale, m.viewport.uses_device_width),
                ),
                (
                    "Initial Scale".to_string(),
                    yes_no(locale, m.viewport.has_initial_scale),
                ),
                (
                    "Skalierbar".to_string(),
                    yes_no(locale, m.viewport.is_scalable),
                ),
                (
                    "Korrekt konfiguriert".to_string(),
                    yes_no(locale, m.viewport.is_properly_configured),
                ),
            ],
            touch_targets: vec![
                (
                    "Gesamt".to_string(),
                    m.touch_targets.total_targets.to_string(),
                ),
                (
                    "Ausreichend (≥44px)".to_string(),
                    m.touch_targets.adequate_targets.to_string(),
                ),
                (
                    "Zu klein".to_string(),
                    m.touch_targets.small_targets.to_string(),
                ),
                (
                    "Zu eng beieinander".to_string(),
                    m.touch_targets.crowded_targets.to_string(),
                ),
            ],
            font_analysis: vec![
                (
                    "Basis-Schriftgröße".to_string(),
                    format!("{:.0}px", m.font_sizes.base_font_size),
                ),
                (
                    "Kleinste Schrift".to_string(),
                    format!("{:.0}px", m.font_sizes.smallest_font_size),
                ),
                (
                    "Lesbarer Text".to_string(),
                    format!("{:.0}%", m.font_sizes.legible_percentage),
                ),
                (
                    "Relative Einheiten".to_string(),
                    yes_no(locale, m.font_sizes.uses_relative_units),
                ),
            ],
            content_sizing: vec![
                (
                    "Passt in Viewport".to_string(),
                    yes_no(locale, m.content_sizing.fits_viewport),
                ),
                (
                    "Kein hor. Scrollen".to_string(),
                    yes_no(locale, !m.content_sizing.has_horizontal_scroll),
                ),
                (
                    "Responsive Bilder".to_string(),
                    yes_no(locale, m.content_sizing.uses_responsive_images),
                ),
                (
                    "Media Queries".to_string(),
                    yes_no(locale, m.content_sizing.uses_media_queries),
                ),
            ],
            issues: m
                .issues
                .iter()
                .map(|i| (i.category.clone(), i.severity, i.message.clone()))
                .collect(),
        }
    })
}

fn build_dark_mode_details(normalized: &AuditContext) -> Option<DarkModePresentation> {
    normalized
        .raw_dark_mode
        .as_ref()
        .map(|dm| DarkModePresentation {
            supported: dm.supported,
            score: dm.score,
            detection_methods: dm.detection_methods.clone(),
            color_scheme_css: dm.color_scheme_css,
            meta_color_scheme: dm.meta_color_scheme.clone(),
            css_custom_properties: dm.css_custom_properties,
            dark_contrast_violations: dm.dark_contrast_violations,
            dark_only_violations: dm.dark_only_violations,
            light_only_violations: dm.light_only_violations,
            issues: dm
                .issues
                .iter()
                .map(|i| (i.severity.clone(), i.description.clone()))
                .collect(),
        })
}

fn build_ux_details(normalized: &AuditContext, i18n: &I18n) -> Option<UxPresentation> {
    let locale = i18n.locale();
    normalized.raw_ux.as_ref().map(|u| {
        let ux_score = normalized_module_score(normalized, "UX").unwrap_or(u.score);
        UxPresentation {
            score: ux_score,
            grade: normalized_module_grade(normalized, "UX").unwrap_or_else(|| u.grade.clone()),
            interpretation: module_interpretation(normalized, "ux", locale),
            dimensions: vec![
                UxDimensionPresentation {
                    name: u.cta_clarity.name.clone(),
                    score: u.cta_clarity.score,
                    summary: u.cta_clarity.summary.clone(),
                },
                UxDimensionPresentation {
                    name: u.visual_hierarchy.name.clone(),
                    score: u.visual_hierarchy.score,
                    summary: u.visual_hierarchy.summary.clone(),
                },
                UxDimensionPresentation {
                    name: u.content_clarity.name.clone(),
                    score: u.content_clarity.score,
                    summary: u.content_clarity.summary.clone(),
                },
                UxDimensionPresentation {
                    name: u.trust_signals.name.clone(),
                    score: u.trust_signals.score,
                    summary: u.trust_signals.summary.clone(),
                },
                UxDimensionPresentation {
                    name: u.cognitive_load.name.clone(),
                    score: u.cognitive_load.score,
                    summary: u.cognitive_load.summary.clone(),
                },
            ],
            issues: u
                .issues
                .iter()
                .map(|i| UxIssuePresentation {
                    dimension: i.dimension.clone(),
                    severity: i.severity.clone(),
                    problem: i.problem.clone(),
                    impact: i.impact.clone(),
                    recommendation: i.recommendation.clone(),
                })
                .collect(),
        }
    })
}

fn build_journey_details(normalized: &AuditContext, i18n: &I18n) -> Option<JourneyPresentation> {
    let locale = i18n.locale();
    let en = locale == "en";
    normalized.raw_journey.as_ref().map(|j| {
        let journey_score = normalized_module_score(normalized, "Journey").unwrap_or(j.score);
        // Detect page type mismatch between SEO profile and Journey module
        let seo_type: Option<String> = normalized
            .raw_seo
            .as_ref()
            .and_then(|s| s.content_profile.as_ref())
            .map(|cp| cp.page_classification.primary_type.label(en).to_lowercase());
        let journey_type = j.page_intent.label(en).to_lowercase();
        let type_note = match seo_type {
            Some(ref st) if !st.is_empty() && !journey_type.is_empty() && st != &journey_type => {
                if locale == "en" {
                    format!(
                        " (Primary classification: {}. Secondary signals point to {}.)",
                        st, journey_type
                    )
                } else {
                    format!(
                        " (Primäre Einordnung: {}. Sekundäre Signale deuten auf {} hin.)",
                        st, journey_type
                    )
                }
            }
            _ => String::new(),
        };
        let base_journey = module_interpretation(normalized, "journey", locale);
        let journey_interpretation = if type_note.is_empty() {
            base_journey
        } else {
            format!("{}{}", base_journey, type_note)
        };
        JourneyPresentation {
            score: journey_score,
            grade: normalized_module_grade(normalized, "Journey")
                .unwrap_or_else(|| j.grade.clone()),
            page_intent: j.page_intent.label(en).to_string(),
            interpretation: journey_interpretation,
            dimensions: vec![
                JourneyDimensionPresentation {
                    name: j.entry_clarity.name.clone(),
                    score: j.entry_clarity.score,
                    weight_pct: (j.entry_clarity.weight * 100.0).round() as u32,
                    summary: j.entry_clarity.summary.clone(),
                },
                JourneyDimensionPresentation {
                    name: j.orientation.name.clone(),
                    score: j.orientation.score,
                    weight_pct: (j.orientation.weight * 100.0).round() as u32,
                    summary: j.orientation.summary.clone(),
                },
                JourneyDimensionPresentation {
                    name: j.navigation.name.clone(),
                    score: j.navigation.score,
                    weight_pct: (j.navigation.weight * 100.0).round() as u32,
                    summary: j.navigation.summary.clone(),
                },
                JourneyDimensionPresentation {
                    name: j.interaction.name.clone(),
                    score: j.interaction.score,
                    weight_pct: (j.interaction.weight * 100.0).round() as u32,
                    summary: j.interaction.summary.clone(),
                },
                JourneyDimensionPresentation {
                    name: j.conversion.name.clone(),
                    score: j.conversion.score,
                    weight_pct: (j.conversion.weight * 100.0).round() as u32,
                    summary: j.conversion.summary.clone(),
                },
            ],
            friction_points: j
                .friction_points
                .iter()
                .map(|fp| FrictionPointPresentation {
                    step: fp.step.clone(),
                    severity: fp.severity.clone(),
                    problem: fp.problem.clone(),
                    impact: fp.impact.clone(),
                    recommendation: fp.recommendation.clone(),
                })
                .collect(),
        }
    })
}

pub(super) fn build_module_details_from_normalized(
    i18n: &I18n,
    normalized: &AuditContext,
) -> ModuleDetailsBlock {
    let performance = build_performance_details(normalized, i18n);
    let search_experience = build_search_experience(normalized, i18n);
    let seo = build_seo_details(normalized, i18n);
    let security = build_security_details(normalized, i18n);
    let mobile = build_mobile_details(normalized, i18n);
    let dark_mode = build_dark_mode_details(normalized);
    let ux = build_ux_details(normalized, i18n);
    let journey = build_journey_details(normalized, i18n);

    let source_quality = normalized.raw_source_quality.clone();
    let ai_visibility = normalized.raw_ai_visibility.clone();
    let tech_stack = normalized.raw_tech_stack.clone();
    let content_visibility = normalized.raw_content_visibility.clone();
    let best_practices = normalized.raw_best_practices.clone();
    let patterns = normalized.raw_patterns.clone();

    let has_any = performance.is_some()
        || search_experience.is_some()
        || seo.is_some()
        || security.is_some()
        || mobile.is_some()
        || ux.is_some()
        || journey.is_some()
        || dark_mode.is_some()
        || source_quality.is_some()
        || ai_visibility.is_some()
        || tech_stack.is_some()
        || content_visibility.is_some()
        || best_practices.is_some()
        || patterns.is_some();

    ModuleDetailsBlock {
        search_experience,
        performance,
        seo,
        security,
        mobile,
        ux,
        journey,
        dark_mode,
        source_quality,
        ai_visibility,
        tech_stack,
        content_visibility,
        best_practices,
        patterns,
        has_any,
    }
}

/// Static set of module keys covered by [`ModuleDetailsBlock`].
///
/// Every optional field in `ModuleDetailsBlock` that carries module data must
/// appear here. The parity test compares this against `active_modules()` to
/// detect future coverage gaps.
#[cfg(test)]
pub(super) fn pdf_rendered_modules() -> std::collections::BTreeSet<&'static str> {
    [
        "performance",
        "seo",
        "security",
        "mobile",
        "ux",
        "journey",
        "dark_mode",
        "source_quality",
        "ai_visibility",
        "tech_stack",
        "content_visibility",
        "best_practices",
        "patterns",
    ]
    .into_iter()
    .collect()
}

pub(super) fn normalized_module_score(
    normalized: &NormalizedReport,
    module_name: &str,
) -> Option<u32> {
    normalized
        .module_scores
        .iter()
        .find(|m| m.name == module_name)
        .map(|m| m.score)
}

pub(super) fn normalized_module_grade(
    normalized: &NormalizedReport,
    module_name: &str,
) -> Option<String> {
    normalized
        .module_scores
        .iter()
        .find(|m| m.name == module_name)
        .map(|m| m.grade.clone())
}
