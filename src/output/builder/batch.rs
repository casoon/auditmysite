//! Batch-report presentation builder.

use crate::output::localized::is_english;
use std::collections::HashMap;

use crate::audit::normalized::NormalizedFinding;
use crate::audit::{normalize, BatchReport, BrokenLinkSeverity, NormalizedReport};
use crate::i18n::I18n;
use crate::output::explanations::resolve_explanation;
use crate::output::report_model::*;
use crate::seo::profile::PageType;
use crate::util::truncate_url;
use crate::wcag::Severity;

use super::actions::{
    build_narrative_arc, derive_action_plan, derive_business_impact, localized_finding_text,
};
use super::helpers::{build_batch_appendix, build_batch_verdict};
use crate::audit::prioritization::{
    derive_execution_priority, score_to_priority, severity_to_priority,
};
use crate::seo::interpretation::page_profile_optimization_note_text;
use crate::seo::{
    average_page_semantic_score, derive_domain_topics, derive_topic_overlap_pairs,
    extract_page_topics,
};

/// Canonical kind for page-type distribution insights (#406: message-baked, locale derived at render time).
enum DistributionInsightKind {
    HighThinContentShare,
    NoEditorialContent,
    MarketingDominated,
    Balanced,
}

fn distribution_insight_text(kind: DistributionInsightKind, en: bool) -> String {
    match (kind, en) {
        (DistributionInsightKind::HighThinContentShare, false) => {
            "Hoher Anteil an Thin-Content-Seiten: Das kann Informationswert und SEO-Potenzial begrenzen."
        }
        (DistributionInsightKind::HighThinContentShare, true) => {
            "High share of thin-content pages: this can limit informational value and SEO potential."
        }
        (DistributionInsightKind::NoEditorialContent, false) => {
            "Editoriale Inhaltsseiten fehlen: Wissensaufbau und Suchintentionen werden kaum bedient."
        }
        (DistributionInsightKind::NoEditorialContent, true) => {
            "No editorial content pages: knowledge-building and informational search intents are barely served."
        }
        (DistributionInsightKind::MarketingDominated, false) => {
            "Marketing- und Landingpages dominieren: Mehr strukturierter Tiefeninhalt würde die Domain ausbalancieren."
        }
        (DistributionInsightKind::MarketingDominated, true) => {
            "Marketing and landing pages dominate: more structured in-depth content would balance the domain."
        }
        (DistributionInsightKind::Balanced, false) => {
            "Die Seitentypen sind insgesamt ausgewogen verteilt, ohne klar dominierende Schwachmuster."
        }
        (DistributionInsightKind::Balanced, true) => {
            "Page types are distributed in a balanced way overall, without a clearly dominant weak pattern."
        }
    }
    .to_string()
}

/// Build a complete presentation model from a batch audit report
pub fn build_batch_presentation(batch: &BatchReport) -> BatchPresentation {
    let i18n = I18n::new("de").expect("default locale must always load");
    build_batch_presentation_with_locale(batch, &i18n)
}

/// Locale-aware variant of [`build_batch_presentation`].
pub fn build_batch_presentation_with_locale(batch: &BatchReport, i18n: &I18n) -> BatchPresentation {
    let normalized_reports: Vec<crate::audit::normalized::NormalizedReport> = batch
        .reports
        .iter()
        .map(|r| normalize(r).normalized)
        .collect();
    build_batch_presentation_with_normalized(batch, i18n, &normalized_reports)
}

/// Locale-aware variant for callers that already normalized the batch pages.
pub fn build_batch_presentation_with_normalized(
    batch: &BatchReport,
    i18n: &I18n,
    normalized_reports: &[NormalizedReport],
) -> BatchPresentation {
    debug_assert_eq!(
        batch.reports.len(),
        normalized_reports.len(),
        "batch presentation requires one normalized report per raw report"
    );
    let localized: Vec<NormalizedReport>;
    let normalized_reports = if is_english(i18n) {
        normalized_reports
    } else {
        let texts: std::collections::HashMap<String, String> = batch
            .reports
            .iter()
            .flat_map(|r| r.accessibility.wcag_results.localized_texts.clone())
            .collect();
        localized = normalized_reports
            .iter()
            .cloned()
            .map(|mut report| {
                super::helpers::localize_shared_texts(&mut report.findings, &texts);
                report
            })
            .collect();
        &localized
    };
    // One group per rule across all pages, in the order the batch JSON's
    // `top_actions` lists them (plan 64).
    let top_issues: Vec<FindingGroup> =
        crate::audit::normalized::aggregate_batch_findings(normalized_reports)
            .into_iter()
            .map(|batch| {
                finding_group_from_normalized(
                    i18n,
                    &NormalizedFindingAccumulator {
                        severity: batch.finding.severity,
                        count: batch.finding.occurrence_count,
                        finding: batch.finding,
                        urls: batch.urls,
                    },
                )
            })
            .collect();

    let issue_frequency: Vec<IssueFrequency> = top_issues
        .iter()
        .map(|g| IssueFrequency {
            problem: g.title.clone(),
            wcag: g.wcag_criterion.clone(),
            occurrences: g.occurrence_count,
            affected_urls: g.affected_urls.len(),
            priority: g.priority,
        })
        .collect();

    let interactive_summary = {
        use std::collections::HashMap as HMap;
        let total_pages_tested = normalized_reports
            .iter()
            .filter(|nr| nr.accessibility_journey.is_some() || !nr.interactive_findings.is_empty())
            .count();
        if total_pages_tested == 0 {
            None
        } else {
            let mut category_map: HMap<String, (usize, Severity)> = HMap::new();
            for nr in normalized_reports {
                let mut seen_in_page: std::collections::HashSet<String> = Default::default();
                for f in &nr.interactive_findings {
                    let entry = category_map
                        .entry(f.category.clone())
                        .or_insert((0, Severity::Low));
                    if seen_in_page.insert(f.category.clone()) {
                        entry.0 += 1;
                    }
                    if f.severity > entry.1 {
                        entry.1 = f.severity;
                    }
                }
            }
            let mut categories: Vec<crate::output::report_model::InteractiveCategoryRow> =
                category_map
                    .into_iter()
                    .map(|(category, (affected_urls, max_severity))| {
                        crate::output::report_model::InteractiveCategoryRow {
                            category,
                            affected_urls,
                            max_severity,
                        }
                    })
                    .collect();
            categories.sort_by(|a, b| {
                b.affected_urls
                    .cmp(&a.affected_urls)
                    .then_with(|| a.category.cmp(&b.category))
            });
            let pages_with_issues = normalized_reports
                .iter()
                .filter(|nr| !nr.interactive_findings.is_empty())
                .count();
            let has_critical = normalized_reports.iter().any(|nr| {
                nr.interactive_findings
                    .iter()
                    .any(|f| f.severity == Severity::Critical)
            });
            Some(crate::output::report_model::InteractiveJourneySummary {
                total_pages_tested,
                pages_with_issues,
                categories,
                has_critical,
            })
        }
    };

    let action_plan = derive_action_plan(i18n, &top_issues);

    let mut url_ranking: Vec<UrlSummary> = batch
        .reports
        .iter()
        .zip(normalized_reports.iter())
        .map(|(r, nr)| {
            let critical_count = nr.severity_counts.critical + nr.severity_counts.high;
            UrlSummary {
                url: r.url.clone(),
                score: nr.score as f32,
                overall_score: nr.overall_score,
                grade: nr.grade.clone(),
                critical_violations: critical_count,
                total_violations: nr.severity_counts.total,
                passed: nr.score >= 70 && nr.severity_counts.critical == 0,
                priority: score_to_priority(nr.score as f32),
            }
        })
        .collect();
    // Accessibility ascending — the report's main score (plan 83). Ties go
    // to more critical/high findings first, then the lower overall score,
    // then the URL, so the order never depends on audit completion order.
    url_ranking.sort_by(|a, b| {
        a.score
            .total_cmp(&b.score)
            .then_with(|| b.critical_violations.cmp(&a.critical_violations))
            .then_with(|| a.overall_score.cmp(&b.overall_score))
            .then_with(|| a.url.cmp(&b.url))
    });

    let url_details: Vec<CompactUrlSummary> = batch
        .reports
        .iter()
        .zip(normalized_reports.iter())
        .map(|(r, nr)| {
            // Already in the page's finding order (plan 58).
            let sorted = normalized_finding_groups(i18n, nr);
            let top_issue_titles: Vec<String> =
                sorted.iter().take(3).map(|g| g.title.clone()).collect();

            // Unmeasured modules carry no comparable value (plan 86).
            let module_scores = nr
                .module_scores
                .iter()
                .filter(|m| m.measurement_type != "not_measured")
                .map(|m| (m.name.clone(), m.score))
                .collect();

            let topic_terms = extract_page_topics(r);

            CompactUrlSummary {
                url: r.url.clone(),
                score: nr.score as f32,
                grade: nr.grade.clone(),
                critical_violations: nr.severity_counts.critical + nr.severity_counts.high,
                total_violations: nr.severity_counts.total,
                page_type: r
                    .discoverability
                    .seo
                    .as_ref()
                    .and_then(|seo| seo.content_profile.as_ref())
                    .map(|profile| {
                        profile
                            .page_classification
                            .primary_type
                            .label(is_english(i18n))
                            .to_string()
                    }),
                page_attributes: r
                    .discoverability
                    .seo
                    .as_ref()
                    .and_then(|seo| seo.content_profile.as_ref())
                    .map(|profile| profile.page_classification.attributes.clone())
                    .unwrap_or_default(),
                page_semantic_score: r
                    .discoverability
                    .seo
                    .as_ref()
                    .and_then(|seo| seo.content_profile.as_ref())
                    .map(|profile| average_page_semantic_score(&profile.page_classification)),
                biggest_lever: sorted
                    .first()
                    .map(|g| g.title.clone())
                    .or_else(|| {
                        r.discoverability.seo.as_ref().and_then(|seo| {
                            seo.content_profile
                                .as_ref()
                                .map(|cp| page_profile_optimization_note_text(cp, is_english(i18n)))
                        })
                    })
                    .unwrap_or_else(|| {
                        if is_english(i18n) {
                            "Maintain results".to_string()
                        } else {
                            "Ergebnisse stabil halten".to_string()
                        }
                    }),
                topic_terms,
                top_issues: top_issue_titles,
                module_scores,
            }
        })
        .collect();

    let mut sorted_by_score: Vec<_> = batch
        .reports
        .iter()
        .zip(normalized_reports.iter())
        .collect();
    sorted_by_score.sort_by(|a, b| {
        a.1.score
            .cmp(&b.1.score)
            .then_with(|| a.0.url.cmp(&b.0.url))
    });
    let worst_urls: Vec<(String, f32)> = sorted_by_score
        .iter()
        .take(3)
        .map(|(r, nr)| (truncate_url(&r.url, 60), nr.score as f32))
        .collect();
    let best_urls: Vec<(String, f32)> = sorted_by_score
        .iter()
        .rev()
        .take(3)
        .map(|(r, nr)| (truncate_url(&r.url, 60), nr.score as f32))
        .collect();

    let severity_distribution = {
        let critical = normalized_reports
            .iter()
            .map(|nr| nr.severity_counts.critical)
            .sum();
        let high = normalized_reports
            .iter()
            .map(|nr| nr.severity_counts.high)
            .sum();
        let medium = normalized_reports
            .iter()
            .map(|nr| nr.severity_counts.medium)
            .sum();
        let low = normalized_reports
            .iter()
            .map(|nr| nr.severity_counts.low)
            .sum();
        SeverityDistribution {
            critical,
            high,
            medium,
            low,
        }
    };
    let occurrence_distribution = {
        let critical = normalized_reports
            .iter()
            .map(|nr| nr.occurrence_counts.critical)
            .sum();
        let high = normalized_reports
            .iter()
            .map(|nr| nr.occurrence_counts.high)
            .sum();
        let medium = normalized_reports
            .iter()
            .map(|nr| nr.occurrence_counts.medium)
            .sum();
        let low = normalized_reports
            .iter()
            .map(|nr| nr.occurrence_counts.low)
            .sum();
        SeverityDistribution {
            critical,
            high,
            medium,
            low,
        }
    };

    let en = is_english(i18n);
    let mut page_type_counts: HashMap<String, usize> = HashMap::new();
    let mut page_semantic_scores: Vec<(String, String, u32)> = Vec::new();
    let mut thin_pages = 0usize;
    let mut editorial_pages = 0usize;
    let mut marketing_pages = 0usize;
    for report in &batch.reports {
        if let Some(profile) = report
            .discoverability
            .seo
            .as_ref()
            .and_then(|seo| seo.content_profile.as_ref())
        {
            let label = profile
                .page_classification
                .primary_type
                .label(en)
                .to_string();
            *page_type_counts.entry(label).or_default() += 1;
            let semantic_score = average_page_semantic_score(&profile.page_classification);
            page_semantic_scores.push((
                report.url.clone(),
                profile
                    .page_classification
                    .primary_type
                    .label(en)
                    .to_string(),
                semantic_score,
            ));
            match profile.page_classification.primary_type {
                PageType::ThinContent => thin_pages += 1,
                PageType::Editorial => editorial_pages += 1,
                PageType::MarketingLanding => marketing_pages += 1,
                _ => {}
            }
        }
    }
    let mut page_type_distribution: Vec<(String, usize, u32)> = page_type_counts
        .into_iter()
        .map(|(label, count)| {
            let pct = ((count as f64 / batch.summary.total_urls as f64) * 100.0).round() as u32;
            (label, count, pct)
        })
        .collect();
    page_type_distribution.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    let page_type_groups = build_page_type_groups(&url_details);

    let mut distribution_insights = Vec::new();
    if thin_pages > 0 && (thin_pages as f64 / batch.summary.total_urls as f64) >= 0.2 {
        distribution_insights.push(distribution_insight_text(
            DistributionInsightKind::HighThinContentShare,
            en,
        ));
    }
    if editorial_pages == 0 {
        distribution_insights.push(distribution_insight_text(
            DistributionInsightKind::NoEditorialContent,
            en,
        ));
    }
    if marketing_pages > 0 && (marketing_pages as f64 / batch.summary.total_urls as f64) >= 0.5 {
        distribution_insights.push(distribution_insight_text(
            DistributionInsightKind::MarketingDominated,
            en,
        ));
    }
    if distribution_insights.is_empty() && !page_type_distribution.is_empty() {
        distribution_insights.push(distribution_insight_text(
            DistributionInsightKind::Balanced,
            en,
        ));
    }

    page_semantic_scores.sort_by(|a, b| b.2.cmp(&a.2).then_with(|| a.0.cmp(&b.0)));
    let strongest_content_pages = page_semantic_scores.iter().take(5).cloned().collect();
    let weakest_content_pages = page_semantic_scores.iter().rev().take(5).cloned().collect();
    let top_topics = derive_domain_topics(&url_details);
    let overlap_pairs = derive_topic_overlap_pairs(&url_details);

    // Near-duplicate detection via SimHash on page text excerpts
    let dup_inputs: Vec<(String, String)> = batch
        .reports
        .iter()
        .filter_map(|r| {
            r.discoverability
                .seo
                .as_ref()
                .filter(|seo| !seo.technical.text_excerpt.is_empty())
                .map(|seo| (r.url.clone(), seo.technical.text_excerpt.clone()))
        })
        .collect();

    let near_duplicates: Vec<(String, String, u8)> = if dup_inputs.len() >= 2 {
        crate::audit::duplicate::detect_near_duplicates(&dup_inputs, 80, 80)
            .into_iter()
            .take(10)
            .map(|p| (p.url_a, p.url_b, p.similarity))
            .collect()
    } else {
        Vec::new()
    };

    // Cross-page duplicate content (identical title / meta description / H1 / og:image)
    let duplicate_content = build_duplicate_content(&batch.reports);

    // Cross-page missing-tag prevalence (meta description / canonical / og:image / og:title)
    let missing_tag_prevalence = build_missing_tag_prevalence(&batch.reports);

    // Per-page canonical conflicts (noindex / og:url mismatch / target
    // status from the sitemap check)
    let canonical_issues = build_canonical_issues(
        &batch.reports,
        batch
            .sitemap_diagnostics
            .as_ref()
            .map(|d| d.http_issues.as_slice())
            .unwrap_or_default(),
    );

    // Non-reciprocal hreflang relationships among audited pages
    let hreflang_issues = build_hreflang_issues(&batch.reports);

    // Cross-page minification consistency (#537)
    let minification_inconsistencies = build_minification_inconsistencies(&batch.reports);

    // Redirect-chain/loop detection across audited URLs (#546)
    let redirect_chain_issues = build_redirect_chain_issues(&batch.reports);

    let (
        sitemap_http_issues,
        orphan_sitemap_urls,
        linked_not_in_sitemap,
        robots_conflicts,
        crawl_depth_diagnostics,
        crawl_depths_unavailable,
        sitemap_status_counts,
    ) = batch
        .sitemap_diagnostics
        .as_ref()
        .map(|diagnostics| {
            (
                diagnostics.http_issues.clone(),
                diagnostics.orphan_sitemap_urls.clone(),
                diagnostics.linked_not_in_sitemap.clone(),
                diagnostics.robots_conflicts.clone(),
                diagnostics.crawl_depths.clone(),
                diagnostics.crawl_depths_unavailable.is_some(),
                diagnostics.status_counts.clone(),
            )
        })
        .unwrap_or_default();

    // Budget violations: aggregate across all pages
    let budget_summary: Vec<(String, String, usize, String)> = {
        use std::collections::HashMap;
        let mut map: HashMap<String, (String, usize, &str)> = HashMap::new();
        for r in &batch.reports {
            for v in &r.experience.budget_violations {
                let entry = map
                    .entry(v.metric.clone())
                    .or_insert_with(|| (v.budget_label.clone(), 0, "Warning"));
                entry.1 += 1;
                if v.severity == crate::audit::budget::BudgetSeverity::Error {
                    entry.2 = "Error";
                }
            }
        }
        let mut rows: Vec<_> = map
            .into_iter()
            .map(|(metric, (budget, count, sev))| (metric, budget, count, sev.to_string()))
            .collect();
        rows.sort_by(|a, b| b.2.cmp(&a.2).then(a.0.cmp(&b.0)));
        rows
    };

    // Render-blocking: aggregate across all pages
    let render_blocking_summary: Vec<(String, String)> = {
        let pages_with_data: Vec<_> = batch
            .reports
            .iter()
            .filter_map(|r| {
                r.performance
                    .as_ref()
                    .and_then(|p| p.render_blocking.as_ref())
            })
            .collect();
        if pages_with_data.is_empty() {
            Vec::new()
        } else {
            let n = pages_with_data.len() as f64;
            // The same file on twenty pages is one resource, not twenty
            // (plan 93): count distinct URLs, report loads alongside.
            let total_loads: usize = pages_with_data
                .iter()
                .map(|rb| rb.blocking_scripts.len() + rb.blocking_css.len())
                .sum();
            let distinct_blocking = pages_with_data
                .iter()
                .flat_map(|rb| rb.blocking_scripts.iter().chain(&rb.blocking_css))
                .map(|res| res.url.as_str())
                .collect::<std::collections::HashSet<_>>()
                .len();
            let total_third_party_bytes: u64 =
                pages_with_data.iter().map(|rb| rb.third_party_bytes).sum();
            let pages_with_blocking = pages_with_data
                .iter()
                .filter(|rb| rb.has_blocking())
                .count();
            let en = is_english(i18n);
            let (
                label_pages_analyzed,
                label_pages_blocking,
                label_blocking_total,
                label_third_party,
            ) = if en {
                (
                    "Pages analyzed",
                    "Pages with blocking",
                    "Distinct blocking resources",
                    "Third-party traffic total",
                )
            } else {
                (
                    "Seiten analysiert",
                    "Seiten mit Blocking",
                    "Verschiedene Blocking-Ressourcen",
                    "Third-Party-Traffic gesamt",
                )
            };
            let pages_count_value = if en {
                format!("{} of {}", pages_with_data.len(), batch.reports.len())
            } else {
                format!("{} von {}", pages_with_data.len(), batch.reports.len())
            };
            vec![
                (label_pages_analyzed.to_string(), pages_count_value),
                (
                    label_pages_blocking.to_string(),
                    format!(
                        "{} ({:.0}%)",
                        pages_with_blocking,
                        pages_with_blocking as f64 / n * 100.0
                    ),
                ),
                (
                    label_blocking_total.to_string(),
                    if en {
                        format!("{distinct_blocking} ({total_loads} loads across pages)")
                    } else {
                        format!("{distinct_blocking} ({total_loads} Einbindungen über alle Seiten)")
                    },
                ),
                (
                    label_third_party.to_string(),
                    format!("{:.1} KB", total_third_party_bytes as f64 / 1024.0),
                ),
            ]
        }
    };

    let crawl_links = batch
        .crawl_diagnostics
        .as_ref()
        .map(|crawl| CrawlLinkSummary {
            seed_url: crawl.seed_url.clone(),
            checked_internal_links: crawl.checked_internal_links,
            broken_internal_links: crawl
                .broken_internal_links
                .iter()
                .take(20)
                .map(|link| BrokenLinkRow {
                    source_url: link.source_url.clone(),
                    target_url: link.target_url.clone(),
                    status: match (link.status_code, link.error.as_deref()) {
                        (Some(code), _) => code.to_string(),
                        (None, Some(err)) => err.to_string(),
                        (None, None) => "Unbekannt".to_string(),
                    },
                    is_external: link.is_external,
                    severity: match link.severity {
                        BrokenLinkSeverity::High => "high".to_string(),
                        BrokenLinkSeverity::Medium => "medium".to_string(),
                        BrokenLinkSeverity::Low => "low".to_string(),
                    },
                    redirect_hops: link.redirect_hops,
                })
                .collect(),
            checked_external_links: crawl.checked_external_links,
            broken_external_links: crawl
                .broken_external_links
                .iter()
                .take(20)
                .map(|link| BrokenLinkRow {
                    source_url: link.source_url.clone(),
                    target_url: link.target_url.clone(),
                    status: match (link.status_code, link.error.as_deref()) {
                        (Some(code), _) => code.to_string(),
                        (None, Some(err)) => err.to_string(),
                        (None, None) => "Unbekannt".to_string(),
                    },
                    is_external: link.is_external,
                    severity: match link.severity {
                        BrokenLinkSeverity::High => "high".to_string(),
                        BrokenLinkSeverity::Medium => "medium".to_string(),
                        BrokenLinkSeverity::Low => "low".to_string(),
                    },
                    redirect_hops: link.redirect_hops,
                })
                .collect(),
            redirect_chains: crawl
                .redirect_chains
                .iter()
                .take(20)
                .map(|chain| RedirectChainRow {
                    source_url: chain.source_url.clone(),
                    target_url: chain.target_url.clone(),
                    final_url: chain.final_url.clone(),
                    hops: chain.hops,
                    is_external: chain.is_external,
                })
                .collect(),
        });

    // ── Aggregated module scores, overall score, risk ──────────────
    let average_overall_score = if normalized_reports.is_empty() {
        0
    } else {
        let sum: u32 = normalized_reports.iter().map(|n| n.overall_score).sum();
        (sum as f64 / normalized_reports.len() as f64).round() as u32
    };

    let verdict_text = build_batch_verdict(i18n, batch);

    // Aggregate module averages
    let module_averages = {
        let mut module_sums: HashMap<String, (u32, usize)> = HashMap::new();
        for nr in normalized_reports {
            for ms in nr
                .module_scores
                .iter()
                .filter(|m| m.measurement_type != "not_measured")
            {
                let entry = module_sums.entry(ms.name.clone()).or_insert((0, 0));
                entry.0 += ms.score;
                entry.1 += 1;
            }
        }
        let mut avgs: Vec<(String, u32)> = module_sums
            .into_iter()
            .map(|(name, (sum, count))| (name, (sum as f64 / count as f64).round() as u32))
            .collect();
        // Stable order: Accessibility first, then alphabetical
        avgs.sort_by(|a, b| {
            if a.0 == "Accessibility" {
                std::cmp::Ordering::Less
            } else if b.0 == "Accessibility" {
                std::cmp::Ordering::Greater
            } else {
                a.0.cmp(&b.0)
            }
        });
        avgs
    };

    // Active modules (from first report that has data)
    let active_modules: Vec<String> = module_averages.iter().map(|(n, _)| n.clone()).collect();

    // Schema type distribution across all pages
    let (schema_distribution, pages_without_schema) = {
        let mut type_counts: HashMap<String, usize> = HashMap::new();
        let mut without = 0usize;
        for report in &batch.reports {
            if let Some(seo) = &report.discoverability.seo {
                if seo.structured_data.types.is_empty() {
                    without += 1;
                } else {
                    for schema_type in &seo.structured_data.types {
                        *type_counts.entry(format!("{:?}", schema_type)).or_insert(0) += 1;
                    }
                }
            }
        }
        let mut dist: Vec<(String, usize)> = type_counts.into_iter().collect();
        dist.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        (dist, without)
    };

    // Batch risk level: escalate only when ≥20% of pages share that level.
    // This prevents a single outlier page from setting the domain-wide headline risk.
    // Critical is always surfaced regardless of page share.
    let (risk_level, risk_summary) = {
        use crate::audit::normalized::RiskLevel;
        let worst = crate::audit::compute_worst_risk(normalized_reports);
        let level_str = worst.label_localized(i18n);
        let en = is_english(i18n);
        let summary = match (worst, en) {
            (RiskLevel::Low, true) => "The audited pages overall show a low accessibility risk.",
            (RiskLevel::Low, false) => "Die geprüften Seiten weisen insgesamt ein geringes Barrierefreiheits-Risiko auf.",
            (RiskLevel::Medium, true) => "Some pages show medium risk. Targeted improvements recommended.",
            (RiskLevel::Medium, false) => "Einzelne Seiten weisen mittleres Risiko auf. Gezielte Verbesserungen empfohlen.",
            (RiskLevel::High, true) => "Several pages show high risk. Timely remediation recommended, especially for WCAG Level A violations.",
            (RiskLevel::High, false) => "Mehrere Seiten haben hohes Risiko. Zeitnahe Behebung empfohlen, besonders bei WCAG-Level-A-Verstößen.",
            (RiskLevel::Critical, true) => "Critical risk across multiple pages. WCAG Level A violations detected automatically — prioritised remediation recommended; whether legal obligations follow depends on the applicable framework (e.g. BFSG/EAA) and needs a manual review.",
            (RiskLevel::Critical, false) => "Kritisches Risiko über mehrere Seiten. WCAG-Level-A-Verstöße automatisiert erkannt — vorrangige Behebung empfohlen; ob daraus rechtliche Pflichten folgen, hängt vom anwendbaren Rahmen (z. B. BFSG/EAA) ab und braucht eine manuelle Prüfung.",
        };
        (level_str, summary.to_string())
    };

    // Domain from first URL
    let domain = batch
        .reports
        .first()
        .map(|r| {
            url::Url::parse(&r.url)
                .ok()
                .and_then(|u| u.host_str().map(|h| h.to_string()))
                .unwrap_or_else(|| r.url.clone())
        })
        .unwrap_or_default();

    // Same contract as the single report: grade and certificate describe the
    // subject of the report, which is accessibility (plan 29, D1). The
    // weighted `average_overall_score` stays visible as its own figure.
    //
    // They also go through the same two functions the single report uses.
    // The literal band tables that used to stand here were a third and fourth
    // cutoff set (95/90/85/80/70/60 for the grade), so the same score could
    // be graded differently depending on which report a reader was holding.
    let (grade, certificate) = batch.summary.grade_and_certificate();

    let en = is_english(i18n);
    // From the root causes (finding patterns on two or more pages), most
    // widespread first; open ones keep their own label (2.0 finding model).
    let template_clusters: Vec<TemplateClusterView> = batch
        .summary
        .root_causes
        .iter()
        .map(|cause| build_template_cluster_view(cause, batch.summary.total_urls, i18n))
        .collect();
    BatchPresentation {
        cover: CoverData {
            title: if en {
                "Web Accessibility Batch Audit Report".to_string()
            } else {
                "Barrierefreiheits-Batch-Audit-Report".to_string()
            },
            url: if en {
                format!("{} URLs audited", batch.summary.total_urls)
            } else {
                format!("{} URLs geprüft", batch.summary.total_urls)
            },
            date: if en {
                chrono::Utc::now().format("%Y-%m-%d").to_string()
            } else {
                chrono::Utc::now().format("%d.%m.%Y").to_string()
            },
            version: env!("CARGO_PKG_VERSION").to_string(),
        },
        portfolio_summary: PortfolioSummary {
            total_urls: batch.summary.total_urls,
            passed: batch.summary.passed,
            failed: batch.summary.failed,
            average_score: batch.summary.average_score.unwrap_or(0.0),
            average_overall_score,
            total_violations: batch.summary.total_violations,
            duration_ms: batch.total_duration_ms,
            verdict_text,
            worst_urls,
            best_urls,
            severity_distribution,
            occurrence_distribution,
            risk_level,
            risk_summary,
            module_averages,
            active_modules,
            domain,
            certificate,
            grade,
            page_type_distribution,
            page_type_groups,
            distribution_insights,
            strongest_content_pages,
            weakest_content_pages,
            top_topics,
            overlap_pairs,
            near_duplicates,
            crawl_links,
            budget_summary,
            render_blocking_summary,
            shared_blocking_resources: build_shared_blocking_resources(&batch.reports),
            schema_distribution,
            pages_without_schema,
            duplicate_content,
            missing_tag_prevalence,
            canonical_issues,
            security_header_consistency: build_security_header_consistency(&batch.reports),
            hreflang_issues,
            sitemap_http_issues,
            sitemap_status_counts,
            orphan_sitemap_urls,
            linked_not_in_sitemap,
            robots_conflicts,
            minification_inconsistencies,
            redirect_chain_issues,
            crawl_depth_diagnostics,
            crawl_depths_unavailable,
        },
        top_issues: top_issues.into_iter().take(10).collect(),
        issue_frequency,
        action_plan,
        url_ranking,
        url_details,
        url_matrix: build_url_matrix(batch),
        appendix: build_batch_appendix(i18n.locale(), batch, normalized_reports),
        interactive_summary,
        template_clusters,
    }
}

/// Build the localized presentation view for one verified template cluster.
///
/// The strong "resolves N pages" claim is only used for `"confirmed"`
/// clusters; `"likely"` clusters always render with explicit uncertainty
/// wording so a report never implies more certainty than the evidence
/// supports.
fn build_template_cluster_view(
    cause: &crate::audit::RootCause,
    total_pages: usize,
    i18n: &I18n,
) -> TemplateClusterView {
    use a11y_report::CauseConfidence;
    let n = cause.url_count.to_string();
    let total = total_pages.to_string();
    let args = || {
        [
            ("selector", cause.selector.clone()),
            ("n", n.clone()),
            ("total", total.clone()),
        ]
    };
    let (headline, decision_label, confidence) = match cause.confidence {
        CauseConfidence::Confirmed => (
            i18n.t_args("batch-template-confirmed-headline", &args()),
            i18n.t_args("batch-template-decision-confirmed", &[("n", n.clone())]),
            "confirmed",
        ),
        CauseConfidence::Likely => (
            i18n.t_args("batch-template-likely-headline", &args()),
            i18n.t_args("batch-template-decision-likely", &[("n", n.clone())]),
            "likely",
        ),
        // No template claim: not listed under template causes, only named
        // as recurring where the action table refers to it.
        CauseConfidence::Open => (
            String::new(),
            i18n.t_args("batch-template-decision-open", &[("n", n.clone())]),
            "open",
        ),
    };

    TemplateClusterView {
        id: cause.id.clone(),
        rule_id: cause.rule_id.clone(),
        selector: cause.selector.clone(),
        confidence: confidence.to_string(),
        affected_pages: cause.url_count,
        headline,
        decision_label,
    }
}

// ─── URL matrix ─────────────────────────────────────────────────────────────

fn build_url_matrix(batch: &BatchReport) -> Vec<UrlMatrixRow> {
    // Build inbound link map: path → count of pages that link here
    let mut inbound: HashMap<String, usize> = HashMap::new();
    for report in &batch.reports {
        if let Some(seo) = &report.discoverability.seo {
            for target in &seo.technical.internal_link_targets {
                *inbound.entry(target.clone()).or_insert(0) += 1;
            }
        }
    }

    batch
        .reports
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let title = r
                .discoverability
                .seo
                .as_ref()
                .and_then(|seo| seo.meta.title.clone());
            let word_count = r
                .discoverability
                .seo
                .as_ref()
                .map(|seo| seo.technical.word_count)
                .unwrap_or(0);
            let outbound = r
                .discoverability
                .seo
                .as_ref()
                .map(|seo| seo.technical.internal_links + seo.technical.external_links)
                .unwrap_or(0);
            let path = url_path(&r.url);
            let inbound_count = inbound.get(&path).copied().unwrap_or(0);

            UrlMatrixRow {
                rank: i + 1,
                url: r.url.clone(),
                title,
                inbound_links: inbound_count,
                outbound_links: outbound,
                word_count,
            }
        })
        .collect()
}

fn url_path(url: &str) -> String {
    url::Url::parse(url)
        .map(|u| u.path().to_string())
        .unwrap_or_else(|_| url.to_string())
}

/// Char-boundary-safe truncation with an ellipsis for display values.
fn truncate_chars(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let head: String = s.chars().take(max).collect();
        format!("{head}…")
    }
}

/// Detect pages that share an identical title, meta description, or H1 across
/// the audited set — a standard cross-page SEO signal (#423). Values are
/// normalized (trimmed, whitespace-collapsed, lowercased) for grouping but
/// stored verbatim (truncated) for display. Only groups with ≥2 pages are kept.
fn build_duplicate_content(reports: &[crate::audit::AuditReport]) -> Vec<DuplicateContentGroup> {
    fn norm(s: &str) -> String {
        s.split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase()
    }

    // (kind, normalized_value) → (verbatim display value, urls)
    let mut groups: HashMap<(&'static str, String), (String, Vec<String>)> = HashMap::new();
    for r in reports {
        let Some(seo) = &r.discoverability.seo else {
            continue;
        };
        let og_image = seo
            .social
            .open_graph
            .as_ref()
            .and_then(|og| og.image.as_deref());
        let candidates: [(&'static str, Option<&str>); 4] = [
            ("title", seo.meta.title.as_deref()),
            ("meta_description", seo.meta.description.as_deref()),
            ("h1", seo.headings.h1_text.as_deref()),
            ("og_image", og_image),
        ];
        for (kind, raw) in candidates {
            let Some(value) = raw.map(str::trim).filter(|s| !s.is_empty()) else {
                continue;
            };
            let entry = groups
                .entry((kind, norm(value)))
                .or_insert_with(|| (value.to_string(), Vec::new()));
            if !entry.1.contains(&r.url) {
                entry.1.push(r.url.clone());
            }
        }
    }

    let mut out: Vec<DuplicateContentGroup> = groups
        .into_iter()
        .filter(|(_, (_, urls))| urls.len() >= 2)
        .map(|((kind, _), (value, urls))| DuplicateContentGroup {
            kind: kind.to_string(),
            value: truncate_chars(&value, 80),
            urls,
        })
        .collect();

    // Deterministic order: most-duplicated first, then by kind, then value.
    out.sort_by(|a, b| {
        b.urls
            .len()
            .cmp(&a.urls.len())
            .then_with(|| a.kind.cmp(&b.kind))
            .then_with(|| a.value.cmp(&b.value))
            .then_with(|| a.urls.cmp(&b.urls))
    });
    out
}

/// Aggregate how often each SEO/social tag is missing across the audited set
/// (#536) — the systematic-gap counterpart to `build_duplicate_content`'s
/// duplicate-value detection. Only tags missing on ≥2 pages are surfaced
/// (same signal-vs-noise threshold as duplicate content): a single missing
/// tag is a page-specific issue already visible in that page's own SEO
/// findings, not a cross-page pattern worth a batch-level table.
fn build_missing_tag_prevalence(
    reports: &[crate::audit::AuditReport],
) -> Vec<MissingTagPrevalence> {
    let mut missing_counts: HashMap<&'static str, usize> = HashMap::new();
    let mut total = 0usize;

    for r in reports {
        let Some(seo) = &r.discoverability.seo else {
            continue;
        };
        total += 1;

        let og = seo.social.open_graph.as_ref();
        let candidates: [(&'static str, bool); 4] = [
            (
                "meta_description",
                seo.meta
                    .description
                    .as_deref()
                    .map(str::trim)
                    .unwrap_or_default()
                    .is_empty(),
            ),
            (
                "canonical",
                seo.technical
                    .canonical_url
                    .as_deref()
                    .map(str::trim)
                    .unwrap_or_default()
                    .is_empty(),
            ),
            (
                "og_image",
                og.and_then(|o| o.image.as_deref())
                    .map(str::trim)
                    .unwrap_or_default()
                    .is_empty(),
            ),
            (
                "og_title",
                og.and_then(|o| o.title.as_deref())
                    .map(str::trim)
                    .unwrap_or_default()
                    .is_empty(),
            ),
        ];
        for (kind, is_missing) in candidates {
            if is_missing {
                *missing_counts.entry(kind).or_insert(0) += 1;
            }
        }
    }

    if total == 0 {
        return Vec::new();
    }

    let mut out: Vec<MissingTagPrevalence> = missing_counts
        .into_iter()
        .filter(|(_, missing_count)| *missing_count >= 2)
        .map(|(kind, missing_count)| MissingTagPrevalence {
            kind: kind.to_string(),
            missing_count,
            total_count: total,
        })
        .collect();

    // Deterministic order: worst prevalence first, then by kind.
    out.sort_by(|a, b| {
        b.missing_count
            .cmp(&a.missing_count)
            .then_with(|| a.kind.cmp(&b.kind))
    });
    out
}

/// Pages grouped by detected page type (plan 93). The type is a heuristic
/// classification, not a verified template; a group of one page gets no
/// mean and no common finding.
fn build_page_type_groups(url_details: &[CompactUrlSummary]) -> Vec<PageTypeGroup> {
    let mut groups: std::collections::BTreeMap<&str, Vec<&CompactUrlSummary>> =
        std::collections::BTreeMap::new();
    for detail in url_details {
        if let Some(page_type) = detail.page_type.as_deref() {
            groups.entry(page_type).or_default().push(detail);
        }
    }
    let mut out: Vec<PageTypeGroup> = groups
        .into_iter()
        .map(|(page_type, pages)| {
            let comparable = pages.len() >= 2;
            let avg_accessibility = comparable.then(|| {
                (pages.iter().map(|p| p.score as f64).sum::<f64>() / pages.len() as f64).round()
                    as u32
            });
            let mut finding_pages: std::collections::BTreeMap<&str, usize> =
                std::collections::BTreeMap::new();
            for page in &pages {
                let titles: std::collections::BTreeSet<&str> =
                    page.top_issues.iter().map(String::as_str).collect();
                for title in titles {
                    *finding_pages.entry(title).or_default() += 1;
                }
            }
            let common_finding = finding_pages
                .into_iter()
                .filter(|(_, count)| comparable && *count >= 2)
                .max_by(|a, b| a.1.cmp(&b.1).then_with(|| b.0.cmp(a.0)))
                .map(|(title, count)| (title.to_string(), count));
            PageTypeGroup {
                page_type: page_type.to_string(),
                pages: pages.len(),
                avg_accessibility,
                common_finding,
            }
        })
        .collect();
    out.sort_by(|a, b| {
        b.pages
            .cmp(&a.pages)
            .then_with(|| a.page_type.cmp(&b.page_type))
    });
    out
}

/// Render-blocking resources loaded by two or more audited pages, by exact
/// URL — query strings and versions stay part of the identity (plan 93).
fn build_shared_blocking_resources(
    reports: &[crate::audit::AuditReport],
) -> Vec<SharedBlockingResource> {
    let mut by_url: std::collections::BTreeMap<
        &str,
        (&str, std::collections::BTreeSet<&str>, u64),
    > = std::collections::BTreeMap::new();
    for report in reports {
        let Some(rb) = report
            .performance
            .as_ref()
            .and_then(|p| p.render_blocking.as_ref())
        else {
            continue;
        };
        for res in rb.blocking_scripts.iter().chain(&rb.blocking_css) {
            let entry = by_url.entry(res.url.as_str()).or_insert((
                res.kind.as_str(),
                Default::default(),
                0,
            ));
            entry.1.insert(report.url.as_str());
            entry.2 = entry.2.max(res.transfer_bytes);
        }
    }
    let mut out: Vec<SharedBlockingResource> = by_url
        .into_iter()
        .filter(|(_, (_, pages, _))| pages.len() >= 2)
        .map(
            |(url, (kind, pages, transfer_bytes))| SharedBlockingResource {
                url: url.to_string(),
                kind: kind.to_string(),
                pages: pages.len(),
                transfer_bytes,
            },
        )
        .collect();
    out.sort_by(|a, b| b.pages.cmp(&a.pages).then_with(|| a.url.cmp(&b.url)));
    out.truncate(10);
    out
}

/// Each security header across the pages with a security measurement (plan
/// 93, step 5): how many pages send it and how many different values they
/// send. One page does not compare, so fewer than two measured pages yield
/// nothing. The values are compared as sent; their meaning is the single
/// page's security check.
fn build_security_header_consistency(
    reports: &[crate::audit::AuditReport],
) -> Vec<SecurityHeaderConsistency> {
    let measured: Vec<(&str, &crate::security::SecurityHeaders)> = reports
        .iter()
        .filter_map(|r| r.security.as_ref().map(|s| (r.url.as_str(), &s.headers)))
        .collect();
    if measured.len() < 2 {
        return Vec::new();
    }
    type Field = fn(&crate::security::SecurityHeaders) -> &Option<String>;
    let headers: [(&str, Field); 10] = [
        ("Content-Security-Policy", |h| &h.content_security_policy),
        ("Strict-Transport-Security", |h| {
            &h.strict_transport_security
        }),
        ("X-Content-Type-Options", |h| &h.x_content_type_options),
        ("X-Frame-Options", |h| &h.x_frame_options),
        ("Referrer-Policy", |h| &h.referrer_policy),
        ("Permissions-Policy", |h| &h.permissions_policy),
        ("Cross-Origin-Opener-Policy", |h| {
            &h.cross_origin_opener_policy
        }),
        ("Cross-Origin-Resource-Policy", |h| {
            &h.cross_origin_resource_policy
        }),
        ("Access-Control-Allow-Origin", |h| {
            &h.access_control_allow_origin
        }),
        ("Access-Control-Allow-Credentials", |h| {
            &h.access_control_allow_credentials
        }),
    ];
    headers
        .iter()
        .map(|(name, field)| {
            let values: Vec<Option<&str>> = measured
                .iter()
                .map(|(_, h)| field(h).as_deref().map(str::trim))
                .collect();
            let present: Vec<&str> = values.iter().flatten().copied().collect();
            let distinct = present
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len();
            let missing_on = if present.is_empty() {
                Vec::new()
            } else {
                measured
                    .iter()
                    .zip(&values)
                    .filter(|(_, v)| v.is_none())
                    .take(3)
                    .map(|((url, _), _)| url.to_string())
                    .collect()
            };
            SecurityHeaderConsistency {
                header: name.to_string(),
                pages_checked: measured.len(),
                pages_present: present.len(),
                distinct_values: distinct,
                missing_on,
            }
        })
        .collect()
}

/// Detect canonical-tag conflicts per page (#423): a canonical pointing away
/// while the page is `noindex` (mixed signals to crawlers), and a canonical
/// that disagrees with the page's `og:url`, and a canonical target the sitemap
/// check saw redirect or fail (plan 88). Targets outside the sitemap check
/// stay unchecked. Pure per-page checks aggregated across the batch; no extra
/// network requests.
fn build_canonical_issues(
    reports: &[crate::audit::AuditReport],
    sitemap_issues: &[crate::audit::SitemapHttpIssue],
) -> Vec<CanonicalIssue> {
    fn norm_url(u: &str) -> String {
        u.trim().trim_end_matches('/').to_string()
    }

    let mut out = Vec::new();
    for r in reports {
        let Some(seo) = &r.discoverability.seo else {
            continue;
        };
        let Some(canonical) = seo
            .technical
            .canonical_url
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
        else {
            continue;
        };

        // canonical present while the page is noindex → conflicting signal.
        let noindex = seo
            .technical
            .robots_meta
            .as_deref()
            .is_some_and(|m| m.to_lowercase().contains("noindex"));
        if noindex {
            out.push(CanonicalIssue {
                kind: "noindex_conflict".to_string(),
                url: r.url.clone(),
                detail: canonical.to_string(),
            });
        }

        // canonical vs og:url mismatch (ignoring trailing slash).
        if let Some(og_url) = seo
            .social
            .open_graph
            .as_ref()
            .and_then(|og| og.url.as_deref())
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            if norm_url(canonical) != norm_url(og_url) {
                out.push(CanonicalIssue {
                    kind: "og_url_mismatch".to_string(),
                    url: r.url.clone(),
                    detail: format!("{canonical} ≠ {og_url}"),
                });
            }
        }

        // canonical target the sitemap check saw redirect or fail.
        if let Some(target) = sitemap_issues
            .iter()
            .find(|i| norm_url(&i.url) == norm_url(canonical))
        {
            let issue = match target.kind.as_str() {
                "redirect" => Some(CanonicalIssue {
                    kind: "target_redirects".to_string(),
                    url: r.url.clone(),
                    detail: match &target.final_url {
                        Some(final_url) => format!("{canonical} → {final_url}"),
                        None => canonical.to_string(),
                    },
                }),
                "status" | "fetch_error" => Some(CanonicalIssue {
                    kind: "target_unreachable".to_string(),
                    url: r.url.clone(),
                    detail: match target.status_code {
                        Some(code) => format!("{canonical} (HTTP {code})"),
                        None => canonical.to_string(),
                    },
                }),
                _ => None,
            };
            out.extend(issue);
        }
    }
    out
}

/// Detect non-reciprocal hreflang relationships among the audited pages (#423):
/// page A points to B via hreflang, B is also in the set, but B does not point
/// back to A. Only verifiable pairs (both pages audited) are checked; targets
/// outside the set are skipped since their hreflang is unknown.
fn build_hreflang_issues(reports: &[crate::audit::AuditReport]) -> Vec<HreflangIssue> {
    fn norm_url(u: &str) -> String {
        u.trim().trim_end_matches('/').to_lowercase()
    }

    // normalized page url → set of normalized hreflang targets it declares.
    let mut declared: HashMap<String, std::collections::HashSet<String>> = HashMap::new();
    for r in reports {
        let Some(seo) = &r.discoverability.seo else {
            continue;
        };
        let entry = declared.entry(norm_url(&r.url)).or_default();
        for tag in &seo.technical.hreflang {
            let t = tag.url.trim();
            if !t.is_empty() {
                entry.insert(norm_url(t));
            }
        }
    }

    let mut out = Vec::new();
    for r in reports {
        let Some(seo) = &r.discoverability.seo else {
            continue;
        };
        let source = norm_url(&r.url);
        for tag in &seo.technical.hreflang {
            let target = norm_url(tag.url.trim());
            if target.is_empty() || target == source {
                continue;
            }
            // Only verify reciprocity for targets that are themselves audited.
            let Some(target_set) = declared.get(&target) else {
                continue;
            };
            if !target_set.contains(&source) {
                out.push(HreflangIssue {
                    source_url: r.url.clone(),
                    target_url: tag.url.trim().to_string(),
                    lang: tag.lang.clone(),
                });
            }
        }
    }
    out.sort_by(|a, b| {
        a.source_url
            .cmp(&b.source_url)
            .then_with(|| a.target_url.cmp(&b.target_url))
    });
    out
}

/// Cross-page minification consistency check (#537): the same asset URL
/// served minified on some pages and unminified on others points at a
/// build-config inconsistency (e.g. one template/content-collection route
/// not going through the minifier) rather than an isolated per-page issue.
/// This is purely a batch-level re-aggregation of `performance::minification`'s
/// existing per-page "flagged unminified" signal — no new detection, and
/// score-neutral (the per-page findings that already feed the performance
/// score are untouched; this just re-groups them across pages).
///
/// An asset flagged unminified on a page (`MinificationAnalysis.unminified_*`)
/// is presence-confirmed by construction — it was observed via
/// `PerformanceResourceTiming`. The harder half is "present but NOT flagged
/// unminified", i.e. proving the asset was actually loaded (and thus minified)
/// on a page rather than simply absent there — the pipeline does not store a
/// full per-page inventory of every loaded JS/CSS URL, so this reconstructs an
/// approximate one from the two existing data sources that do carry a URL
/// list: `CoverageAnalysis.unused_js.scripts` (near-complete script inventory
/// whenever JS coverage collection ran — it records every script the V8
/// profiler saw execute, not just render-blocking ones) and
/// `RenderBlockingAnalysis.blocking_scripts`/`.blocking_css` (render-blocking
/// resources only). For scripts this gives fairly reliable presence
/// confirmation; for CSS it only catches render-blocking stylesheets — a
/// lazily-loaded or media-gated stylesheet may be missed as "not present" even
/// when it was actually loaded and minified there, which would simply mean
/// that page is not counted rather than a wrong count. Full byte-content
/// comparison is deliberately out of scope (#551).
fn build_minification_inconsistencies(
    reports: &[crate::audit::AuditReport],
) -> Vec<MinificationInconsistency> {
    use std::collections::HashSet;

    #[derive(Default)]
    struct Counts {
        minified: usize,
        unminified: usize,
    }

    let mut counts: HashMap<(String, &'static str), Counts> = HashMap::new();

    for r in reports {
        let Some(perf) = &r.performance else {
            continue;
        };
        let Some(min) = &perf.minification else {
            continue;
        };

        let unminified_scripts: HashSet<&str> = min
            .unminified_scripts
            .iter()
            .map(|a| a.url.as_str())
            .collect();
        let unminified_styles: HashSet<&str> = min
            .unminified_styles
            .iter()
            .map(|a| a.url.as_str())
            .collect();

        for url in &unminified_scripts {
            counts
                .entry((url.to_string(), "script"))
                .or_default()
                .unminified += 1;
        }
        for url in &unminified_styles {
            counts
                .entry((url.to_string(), "css"))
                .or_default()
                .unminified += 1;
        }

        // Presence inventory for "loaded but NOT flagged unminified" — see
        // the doc comment above for the data-source limitation.
        let mut known_scripts: HashSet<&str> = HashSet::new();
        let mut known_styles: HashSet<&str> = HashSet::new();
        if let Some(coverage) = &perf.coverage {
            known_scripts.extend(coverage.unused_js.scripts.iter().map(|s| s.url.as_str()));
        }
        if let Some(rb) = &perf.render_blocking {
            known_scripts.extend(rb.blocking_scripts.iter().map(|res| res.url.as_str()));
            known_styles.extend(rb.blocking_css.iter().map(|res| res.url.as_str()));
        }

        for url in known_scripts.difference(&unminified_scripts) {
            counts
                .entry((url.to_string(), "script"))
                .or_default()
                .minified += 1;
        }
        for url in known_styles.difference(&unminified_styles) {
            counts.entry((url.to_string(), "css")).or_default().minified += 1;
        }
    }

    let mut out: Vec<MinificationInconsistency> = counts
        .into_iter()
        .filter(|(_, c)| c.minified > 0 && c.unminified > 0)
        .map(|((url, kind), c)| MinificationInconsistency {
            url,
            kind: kind.to_string(),
            minified_on_count: c.minified,
            unminified_on_count: c.unminified,
            total_pages_with_asset: c.minified + c.unminified,
        })
        .collect();

    // Deterministic order: worst inconsistency first, then by url.
    out.sort_by(|a, b| {
        b.unminified_on_count
            .cmp(&a.unminified_on_count)
            .then_with(|| a.url.cmp(&b.url))
            .then_with(|| a.kind.cmp(&b.kind))
    });
    out.truncate(30);
    out
}

/// Cross-page redirect-chain/loop detection (#546): flags audited URLs whose
/// own navigation passed through a long (≥3 hop) or cyclical HTTP redirect
/// chain before reaching a final page — wasted crawl budget/TTFB, not a
/// correctness issue. Purely a re-aggregation of the per-page redirect chain
/// already tracked by `seo::page_health::PageHealthAnalysis.redirect_chain`
/// (`follow_redirect_chain`, capped at 10 hops there) — no new redirect
/// detection, and score-neutral (the existing per-page `multiple_redirects`
/// SEO finding is untouched; this just re-groups the same underlying data
/// across pages).
fn build_redirect_chain_issues(reports: &[crate::audit::AuditReport]) -> Vec<RedirectChainIssue> {
    use std::collections::HashSet;

    let mut out = Vec::new();

    for r in reports {
        let Some(seo) = &r.discoverability.seo else {
            continue;
        };
        let Some(page_health) = &seo.page_health else {
            continue;
        };
        let hop_count = page_health.redirect_chain.len();
        if hop_count == 0 {
            continue;
        }

        let mut seen: HashSet<&str> = HashSet::new();
        let is_loop = page_health
            .redirect_chain
            .iter()
            .any(|hop| !seen.insert(hop.url.as_str()));

        if hop_count < 3 && !is_loop {
            continue;
        }

        let mut chain: Vec<String> = page_health
            .redirect_chain
            .iter()
            .map(|hop| hop.url.clone())
            .collect();
        if let Some(final_url) = &page_health.own_final_url {
            if chain.last().map(String::as_str) != Some(final_url.as_str()) {
                chain.push(final_url.clone());
            }
        }
        chain.truncate(10);

        out.push(RedirectChainIssue {
            url: r.url.clone(),
            hop_count,
            is_loop,
            chain,
        });
    }

    // Deterministic order: worst (most hops) first, then by url.
    out.sort_by(|a, b| {
        b.hop_count
            .cmp(&a.hop_count)
            .then_with(|| a.url.cmp(&b.url))
    });
    out.truncate(30);
    out
}

// ─── Internal batch helpers ──────────────────────────────────────────────────

#[derive(Clone)]
struct NormalizedFindingAccumulator {
    finding: NormalizedFinding,
    severity: Severity,
    count: usize,
    urls: Vec<String>,
}

fn normalized_finding_groups(i18n: &I18n, normalized: &NormalizedReport) -> Vec<FindingGroup> {
    normalized
        .findings
        .iter()
        .map(|finding| {
            finding_group_from_normalized(
                i18n,
                &NormalizedFindingAccumulator {
                    finding: finding.clone(),
                    severity: finding.severity,
                    count: finding.occurrence_count,
                    urls: vec![normalized.url.clone()],
                },
            )
        })
        .collect()
}

fn finding_group_from_normalized(i18n: &I18n, acc: &NormalizedFindingAccumulator) -> FindingGroup {
    let locale = i18n.locale();
    let finding = &acc.finding;
    // Shared axe_id -> rule_id -> wcag_criterion order; without the axe_id
    // step the batch report hits the same collapse as the single JSON path
    // did (plan 32).
    let explanation = resolve_explanation(
        finding.axe_id.as_deref(),
        &finding.rule_id,
        &finding.wcag_criterion,
    );
    let dimension_label = finding.dimension.as_str();
    let (
        title,
        customer_desc,
        user_impact_text,
        business_impact,
        typical_cause,
        recommendation,
        technical_note,
        role,
        effort,
        execution_priority,
    ) = if let Some(expl) = explanation {
        (
            // The rule's taxonomy title, as in the JSON, not the explanation's
            // criterion fallback (plans 59, 64).
            localized_finding_text(locale, finding).0,
            expl.customer_description_for(locale).to_string(),
            expl.user_impact_for(locale).to_string(),
            derive_business_impact(
                i18n,
                expl.user_impact_for(locale),
                dimension_label,
                acc.severity,
                Some(finding.subcategory_kind.label(false)),
                acc.count,
            ),
            expl.typical_cause_for(locale).to_string(),
            expl.recommendation_for(locale).to_string(),
            expl.technical_note_for(locale).to_string(),
            expl.responsible_role,
            expl.effort_estimate,
            derive_execution_priority(acc.severity, expl.effort_estimate, dimension_label),
        )
    } else {
        // JSON-stored title/user_impact/technical_impact are canonical English
        // (#406); re-derive the runtime-locale text from the taxonomy.
        let (title_loc, user_impact_loc, technical_loc) = localized_finding_text(locale, finding);
        (
            title_loc,
            finding.description.clone(),
            user_impact_loc.clone(),
            derive_business_impact(
                i18n,
                &user_impact_loc,
                dimension_label,
                acc.severity,
                Some(finding.subcategory_kind.label(false)),
                acc.count,
            ),
            String::new(),
            finding
                .occurrences
                .iter()
                .find_map(|o| o.fix_suggestion.clone())
                .unwrap_or_default(),
            technical_loc,
            Role::Development,
            Effort::Medium,
            derive_execution_priority(acc.severity, Effort::Medium, dimension_label),
        )
    };
    let examples = explanation.map(|e| e.examples()).unwrap_or_default();
    let location_hints = finding
        .occurrences
        .iter()
        .filter_map(|occ| {
            occ.selector
                .clone()
                .or_else(|| Some(format!("AX-Node {}", occ.node_id)))
        })
        .take(5)
        .collect();
    let representative_occurrences = finding
        .occurrences
        .iter()
        .take(3)
        .map(representative_occurrence_from_normalized)
        .collect();

    let narrative = build_narrative_arc(
        i18n,
        acc.count,
        acc.severity,
        dimension_label,
        &customer_desc,
        &user_impact_text,
        &business_impact,
        &typical_cause,
        &recommendation,
        effort,
        role,
    );

    FindingGroup {
        title,
        rule_id: finding.rule_id.clone(),
        wcag_criterion: finding.wcag_criterion.clone(),
        wcag_level: finding.wcag_level.clone(),
        help_url: finding.help_url.clone(),
        dimension: Some(finding.dimension.clone()),
        subcategory: Some(finding.subcategory.clone()),
        issue_class: Some(finding.issue_class.clone()),
        severity: acc.severity,
        priority: severity_to_priority(acc.severity),
        customer_description: customer_desc,
        user_impact: user_impact_text,
        business_impact,
        typical_cause,
        recommendation,
        technical_note,
        confidence: finding.confidence.clone(),
        false_positive_risk: finding.false_positive_risk.clone(),
        verification: finding.verification.clone(),
        complexity: finding.complexity.clone(),
        complexity_reason: finding.complexity_reason.clone(),
        complexity_kind: finding.complexity_kind,
        expected_impact: finding.expected_impact.clone(),
        expected_impact_kind: finding.expected_impact_kind.clone(),
        bfsg_relevance: finding.bfsg_relevance.clone(),
        remediation_priority: finding.remediation_priority.clone(),
        occurrence_count: acc.count,
        affected_urls: acc.urls.clone(),
        affected_elements: acc.count,
        additional_occurrences: acc.count,
        pattern_clusters: Vec::new(),
        location_hints,
        representative_occurrences,
        responsible_role: role,
        effort,
        execution_priority,
        examples,
        // plan/4-root-cause-confidence-wording.md: this threshold is a raw
        // occurrence count across pages, not `RootCause.confidence`
        // ("confirmed" vs "likely" HTML-snippet-shape evidence, see
        // `render_template_cluster_card` above) — a high count alone does
        // not confirm the occurrences share one component/template any
        // more than it does in the single-report case, so this must stay
        // equally hedged rather than asserting a "Root Cause".
        structural_cause: if acc.count >= 10 {
            if locale == "en" {
                Some(format!(
                    "{} occurrences follow the same pattern — this strongly suggests a \
                     shared component or template, though that is only confirmed with \
                     evidence from further pages. A fix at that one shared point would \
                     likely address these occurrences together.",
                    acc.count
                ))
            } else {
                Some(format!(
                    "{} Vorkommen folgen demselben Muster — das deutet stark auf eine \
                     gemeinsam genutzte Komponente oder ein Template hin, bestätigt ist \
                     das erst mit Belegen von weiteren Seiten. Eine Behebung an dieser \
                     einen Stelle würde diese Vorkommen voraussichtlich gemeinsam \
                     adressieren.",
                    acc.count
                ))
            }
        } else if acc.count >= 5 {
            if locale == "en" {
                Some(format!(
                    "This issue appears on {} elements — possibly a shared component or template.",
                    acc.count
                ))
            } else {
                Some(format!(
                    "Dieses Problem tritt bei {} Elementen auf — möglicherweise eine gemeinsam genutzte Komponente oder ein Template.",
                    acc.count
                ))
            }
        } else {
            None
        },
        is_component_issue: acc.count >= 10,
        criticality_tier: crate::output::report_model::classify_criticality_tier(
            &finding.category,
            &finding.wcag_level,
        ),
        narrative,
    }
}

fn representative_occurrence_from_normalized(
    occurrence: &crate::audit::normalized::OccurrenceDetail,
) -> RepresentativeOccurrence {
    RepresentativeOccurrence {
        selector: occurrence
            .selector
            .clone()
            .unwrap_or_else(|| format!("AX-Node {}", occurrence.node_id)),
        node_id: occurrence.node_id.clone(),
        message: occurrence.message.clone(),
        html_snippet: occurrence.html_snippet.clone(),
        suggested_code: occurrence.suggested_code.clone(),
        evidence: occurrence.evidence.clone(),
        // Batch reports never capture element-evidence screenshots
        // (`capture_element_evidence` is single-URL-only) — always None here.
        evidence_screenshot: occurrence.evidence_screenshot.clone(),
        evidence_viewport: occurrence.evidence_viewport,
    }
}

#[cfg(test)]
mod duplicate_content_tests {
    use super::*;
    use crate::audit::AuditReport;
    use crate::cli::WcagLevel;
    use crate::seo::{HeadingStructure, MetaTags, SeoAnalysis};
    use crate::wcag::WcagResults;

    fn report_with(url: &str, title: &str, h1: &str) -> AuditReport {
        let seo = SeoAnalysis {
            meta: MetaTags {
                title: Some(title.to_string()),
                ..Default::default()
            },
            headings: HeadingStructure {
                h1_text: Some(h1.to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        AuditReport::new(url.to_string(), WcagLevel::AA, WcagResults::new(), 100).with_seo(seo)
    }

    #[test]
    fn groups_identical_titles_and_h1s_ignoring_case_and_whitespace() {
        let reports = vec![
            report_with("https://x.test/a", "Welcome", "Hero"),
            // Same title (case + spacing differ) → one title group of 2 pages.
            report_with("https://x.test/b", "  welcome ", "Other"),
            report_with("https://x.test/c", "Unique", "Hero"),
        ];

        let groups = build_duplicate_content(&reports);

        let title_group = groups
            .iter()
            .find(|g| g.kind == "title")
            .expect("duplicate title group");
        assert_eq!(title_group.value, "Welcome"); // verbatim from first occurrence, trimmed
        assert_eq!(title_group.urls.len(), 2);

        // "Hero" H1 appears on a and c → one h1 group of 2.
        let h1_group = groups
            .iter()
            .find(|g| g.kind == "h1")
            .expect("duplicate h1 group");
        assert_eq!(h1_group.urls.len(), 2);

        // "Unique" title and "Other" H1 appear once → not grouped.
        assert!(!groups.iter().any(|g| g.value == "Unique"));
    }

    /// Plan 83: equal accessibility scores rank by URL, so the order is the
    /// same whatever order the audits finished in.
    #[test]
    fn url_ranking_does_not_depend_on_input_order() {
        let urls = ["https://x.test/c", "https://x.test/a", "https://x.test/b"];
        let ranking = |order: &[usize]| {
            let reports = order
                .iter()
                .map(|&i| report_with(urls[i], urls[i], "H"))
                .collect();
            build_batch_presentation(&crate::audit::BatchReport::from_reports(
                reports,
                vec![],
                100,
            ))
            .url_ranking
            .into_iter()
            .map(|u| u.url)
            .collect::<Vec<_>>()
        };
        let expected = vec!["https://x.test/a", "https://x.test/b", "https://x.test/c"];
        assert_eq!(ranking(&[0, 1, 2]), expected);
        assert_eq!(ranking(&[2, 0, 1]), expected);
    }

    #[test]
    fn no_groups_when_all_values_distinct() {
        let reports = vec![
            report_with("https://x.test/a", "One", "A"),
            report_with("https://x.test/b", "Two", "B"),
        ];
        assert!(build_duplicate_content(&reports).is_empty());
    }

    fn report_with_og_image(url: &str, og_image: Option<&str>) -> AuditReport {
        use crate::seo::{OpenGraph, SocialTags};
        let seo = SeoAnalysis {
            social: SocialTags {
                open_graph: og_image.map(|img| OpenGraph {
                    image: Some(img.to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        };
        AuditReport::new(url.to_string(), WcagLevel::AA, WcagResults::new(), 100).with_seo(seo)
    }

    #[test]
    fn groups_identical_og_images() {
        let reports = vec![
            report_with_og_image("https://x.test/a", Some("https://x.test/shared.jpg")),
            report_with_og_image("https://x.test/b", Some("https://x.test/shared.jpg")),
            report_with_og_image("https://x.test/c", Some("https://x.test/unique.jpg")),
        ];

        let groups = build_duplicate_content(&reports);

        let og_group = groups
            .iter()
            .find(|g| g.kind == "og_image")
            .expect("duplicate og_image group");
        assert_eq!(og_group.value, "https://x.test/shared.jpg");
        assert_eq!(og_group.urls.len(), 2);
    }

    fn report_with_tags(
        url: &str,
        description: Option<&str>,
        canonical: Option<&str>,
        og_image: Option<&str>,
        og_title: Option<&str>,
    ) -> AuditReport {
        use crate::seo::{MetaTags, OpenGraph, SocialTags, TechnicalSeo};
        let seo = SeoAnalysis {
            meta: MetaTags {
                description: description.map(str::to_string),
                ..Default::default()
            },
            technical: TechnicalSeo {
                canonical_url: canonical.map(str::to_string),
                ..Default::default()
            },
            social: SocialTags {
                open_graph: Some(OpenGraph {
                    image: og_image.map(str::to_string),
                    title: og_title.map(str::to_string),
                    ..Default::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        };
        AuditReport::new(url.to_string(), WcagLevel::AA, WcagResults::new(), 100).with_seo(seo)
    }

    #[test]
    fn flags_tags_missing_on_at_least_two_pages() {
        let reports = vec![
            report_with_tags(
                "https://x.test/a",
                None,
                Some("https://x.test/a"),
                None,
                Some("A"),
            ),
            report_with_tags(
                "https://x.test/b",
                None,
                Some("https://x.test/b"),
                None,
                Some("B"),
            ),
            report_with_tags(
                "https://x.test/c",
                Some("Has a description"),
                None,
                Some("https://x.test/c.jpg"),
                Some("C"),
            ),
        ];

        let prevalence = build_missing_tag_prevalence(&reports);

        let description_gap = prevalence
            .iter()
            .find(|p| p.kind == "meta_description")
            .expect("meta_description prevalence entry");
        assert_eq!(description_gap.missing_count, 2);
        assert_eq!(description_gap.total_count, 3);

        let og_image_gap = prevalence
            .iter()
            .find(|p| p.kind == "og_image")
            .expect("og_image prevalence entry");
        assert_eq!(og_image_gap.missing_count, 2);

        // canonical is missing on exactly one page → below the ≥2 signal
        // threshold, must not be surfaced.
        assert!(!prevalence.iter().any(|p| p.kind == "canonical"));
        // og_title is present on all three pages → no entry at all.
        assert!(!prevalence.iter().any(|p| p.kind == "og_title"));
    }

    #[test]
    fn no_prevalence_entries_when_all_tags_present() {
        let reports = vec![
            report_with_tags(
                "https://x.test/a",
                Some("Desc A"),
                Some("https://x.test/a"),
                Some("https://x.test/a.jpg"),
                Some("A"),
            ),
            report_with_tags(
                "https://x.test/b",
                Some("Desc B"),
                Some("https://x.test/b"),
                Some("https://x.test/b.jpg"),
                Some("B"),
            ),
        ];
        assert!(build_missing_tag_prevalence(&reports).is_empty());
    }

    fn report_with_canonical(
        url: &str,
        canonical: Option<&str>,
        robots: Option<&str>,
        og_url: Option<&str>,
    ) -> AuditReport {
        use crate::seo::{OpenGraph, SocialTags, TechnicalSeo};
        let seo = SeoAnalysis {
            technical: TechnicalSeo {
                canonical_url: canonical.map(str::to_string),
                robots_meta: robots.map(str::to_string),
                ..Default::default()
            },
            social: SocialTags {
                open_graph: og_url.map(|u| OpenGraph {
                    url: Some(u.to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        };
        AuditReport::new(url.to_string(), WcagLevel::AA, WcagResults::new(), 100).with_seo(seo)
    }

    #[test]
    fn flags_noindex_and_ogurl_conflicts_only() {
        let reports = vec![
            // noindex + canonical → conflict
            report_with_canonical(
                "https://x.test/a",
                Some("https://x.test/a"),
                Some("noindex, follow"),
                None,
            ),
            // canonical disagrees with og:url (trailing slash ignored elsewhere)
            report_with_canonical(
                "https://x.test/b",
                Some("https://x.test/canon"),
                None,
                Some("https://x.test/other"),
            ),
            // clean: canonical == og:url (trailing slash only), indexable → no issue
            report_with_canonical(
                "https://x.test/c",
                Some("https://x.test/c/"),
                Some("index, follow"),
                Some("https://x.test/c"),
            ),
        ];

        let issues = build_canonical_issues(&reports, &[]);
        assert_eq!(issues.len(), 2);
        assert!(issues
            .iter()
            .any(|i| i.kind == "noindex_conflict" && i.url == "https://x.test/a"));
        assert!(issues
            .iter()
            .any(|i| i.kind == "og_url_mismatch" && i.url == "https://x.test/b"));
        // page c must not be flagged (trailing-slash-only difference)
        assert!(!issues.iter().any(|i| i.url == "https://x.test/c"));
    }

    #[test]
    fn canonical_target_status_comes_from_sitemap_check() {
        let reports = vec![
            report_with_canonical("https://x.test/a", Some("https://x.test/old"), None, None),
            report_with_canonical("https://x.test/b", Some("https://x.test/gone/"), None, None),
            report_with_canonical(
                "https://x.test/c",
                Some("https://x.test/elsewhere"),
                None,
                None,
            ),
        ];
        let sitemap_issues = vec![
            crate::audit::SitemapHttpIssue {
                kind: "redirect".to_string(),
                url: "https://x.test/old".to_string(),
                status_code: Some(301),
                final_url: Some("https://x.test/new".to_string()),
                detail: String::new(),
            },
            crate::audit::SitemapHttpIssue {
                kind: "status".to_string(),
                url: "https://x.test/gone".to_string(),
                status_code: Some(404),
                final_url: None,
                detail: String::new(),
            },
        ];

        let issues = build_canonical_issues(&reports, &sitemap_issues);
        assert_eq!(issues.len(), 2);
        assert!(issues.iter().any(|i| i.kind == "target_redirects"
            && i.url == "https://x.test/a"
            && i.detail == "https://x.test/old → https://x.test/new"));
        assert!(issues.iter().any(|i| i.kind == "target_unreachable"
            && i.url == "https://x.test/b"
            && i.detail.ends_with("(HTTP 404)")));
        // a target outside the sitemap check stays unchecked, not flagged
        assert!(!issues.iter().any(|i| i.url == "https://x.test/c"));
    }

    fn report_with_hreflang(url: &str, targets: &[(&str, &str)]) -> AuditReport {
        use crate::seo::technical::HreflangTag;
        use crate::seo::TechnicalSeo;
        let seo = SeoAnalysis {
            technical: TechnicalSeo {
                has_hreflang: !targets.is_empty(),
                hreflang: targets
                    .iter()
                    .map(|(lang, u)| HreflangTag {
                        lang: lang.to_string(),
                        url: u.to_string(),
                    })
                    .collect(),
                ..Default::default()
            },
            ..Default::default()
        };
        AuditReport::new(url.to_string(), WcagLevel::AA, WcagResults::new(), 100).with_seo(seo)
    }

    #[test]
    fn flags_only_non_reciprocal_hreflang_between_audited_pages() {
        let reports = vec![
            // A points to B and to an external (non-audited) page.
            report_with_hreflang(
                "https://x.test/en",
                &[("de", "https://x.test/de"), ("fr", "https://ext.test/fr")],
            ),
            // B points back to A → A↔B reciprocal; B also points to C.
            report_with_hreflang(
                "https://x.test/de",
                &[("en", "https://x.test/en"), ("es", "https://x.test/es")],
            ),
            // C does NOT point back to B → B→C is non-reciprocal.
            report_with_hreflang("https://x.test/es", &[]),
        ];

        let issues = build_hreflang_issues(&reports);

        // Only B→C should be flagged. A→B is reciprocal; A→ext is unverifiable.
        assert_eq!(issues.len(), 1, "got: {issues:?}");
        assert_eq!(issues[0].source_url, "https://x.test/de");
        assert_eq!(issues[0].target_url, "https://x.test/es");
        assert_eq!(issues[0].lang, "es");
    }

    #[test]
    fn batch_presentation_exposes_aggregated_module_markers() {
        let batch = BatchReport::from_reports(
            vec![
                report_with("https://x.test/a", "One", "A"),
                report_with("https://x.test/b", "Two", "B"),
            ],
            vec![],
            200,
        );

        let pres = build_batch_presentation(&batch);

        assert!(
            pres.portfolio_summary
                .active_modules
                .iter()
                .any(|module| module == "SEO"),
            "Batch presentation must expose active module markers"
        );
        assert!(
            pres.portfolio_summary
                .module_averages
                .iter()
                .any(|(module, _)| module == "SEO"),
            "Batch presentation must expose module score aggregates"
        );
        assert_eq!(pres.portfolio_summary.total_urls, 2);
    }

    #[test]
    fn distribution_insight_text_en_has_no_german_umlauts() {
        for kind in [
            DistributionInsightKind::HighThinContentShare,
            DistributionInsightKind::NoEditorialContent,
            DistributionInsightKind::MarketingDominated,
            DistributionInsightKind::Balanced,
        ] {
            let text = distribution_insight_text(kind, true);
            assert!(
                !text.contains(['ä', 'ö', 'ü', 'Ä', 'Ö', 'Ü', 'ß']),
                "EN distribution insight must not contain German characters: {text}"
            );
        }
    }
}

#[cfg(test)]
mod minification_inconsistency_tests {
    use super::*;
    use crate::audit::AuditReport;
    use crate::cli::WcagLevel;
    use crate::performance::{
        CoverageAnalysis, MinificationAnalysis, PerformanceScore, RenderBlockingAnalysis,
        ScriptCoverageEntry, UnminifiedAsset, UnusedCssAnalysis, UnusedJsAnalysis, WebVitals,
    };
    use crate::wcag::WcagResults;

    fn perf_score() -> PerformanceScore {
        PerformanceScore {
            overall: 80,
            grade: crate::performance::PerformanceGrade::Gold,
            lcp_score: None,
            fcp_score: None,
            cls_score: None,
            interactivity_score: None,
            si_score: None,
            metrics_available: 0,
            size_penalty: None,
            js_penalty: None,
            request_penalty: None,
            dom_penalty: None,
            is_capped: None,
            partial: false,
        }
    }

    fn coverage_with_scripts(urls: &[&str]) -> CoverageAnalysis {
        CoverageAnalysis {
            unused_js: UnusedJsAnalysis {
                scripts: urls
                    .iter()
                    .map(|u| ScriptCoverageEntry {
                        url: u.to_string(),
                        total_bytes: 1000,
                        unused_bytes: 0,
                        used_pct: 100.0,
                    })
                    .collect(),
                total_bytes: 1000,
                unused_bytes: 0,
                used_pct: Some(100.0),
            },
            unused_css: UnusedCssAnalysis {
                total_rules: 0,
                used_rules: 0,
                used_pct: None,
                measurement: "not_available".to_string(),
            },
            measurement_warnings: vec![],
            duplicate_assets: vec![],
        }
    }

    fn minification(
        unminified_scripts: &[&str],
        unminified_styles: &[&str],
    ) -> MinificationAnalysis {
        let scripts: Vec<UnminifiedAsset> = unminified_scripts
            .iter()
            .map(|u| UnminifiedAsset {
                url: u.to_string(),
                kind: "script".to_string(),
                decoded_bytes: 100_000,
                transfer_bytes: 20_000,
                savings_bytes: 66_000,
            })
            .collect();
        let styles: Vec<UnminifiedAsset> = unminified_styles
            .iter()
            .map(|u| UnminifiedAsset {
                url: u.to_string(),
                kind: "css".to_string(),
                decoded_bytes: 100_000,
                transfer_bytes: 20_000,
                savings_bytes: 66_000,
            })
            .collect();
        let total_unminified_count = (scripts.len() + styles.len()) as u32;
        MinificationAnalysis {
            unminified_scripts: scripts,
            unminified_styles: styles,
            total_savings_bytes: 0,
            total_unminified_count,
            legacy_scripts: vec![],
            total_legacy_wasted_bytes: 0,
        }
    }

    fn report_with_perf(
        url: &str,
        unminified_scripts: &[&str],
        unminified_styles: &[&str],
        known_scripts: &[&str],
    ) -> AuditReport {
        AuditReport::new(url.to_string(), WcagLevel::AA, WcagResults::new(), 100).with_performance(
            crate::audit::PerformanceResults {
                vitals: WebVitals::default(),
                score: perf_score(),
                render_blocking: Some(RenderBlockingAnalysis {
                    blocking_scripts: vec![],
                    blocking_css: vec![],
                    blocking_transfer_bytes: 0,
                    first_party_bytes: 0,
                    third_party_bytes: 0,
                    third_party_origin_count: 0,
                    suggestions: vec![],
                }),
                content_weight: None,
                third_party: None,
                critical_chain: None,
                minification: Some(minification(unminified_scripts, unminified_styles)),
                animations: None,
                coverage: Some(coverage_with_scripts(known_scripts)),
                measurement_warnings: vec![],
            },
        )
    }

    fn report_with_blocking(url: &str, scripts: &[&str]) -> AuditReport {
        let mut report = report_with_perf(url, &[], &[], &[]);
        let rb = report
            .performance
            .as_mut()
            .and_then(|p| p.render_blocking.as_mut())
            .expect("fixture has render-blocking data");
        rb.blocking_scripts = scripts
            .iter()
            .map(|u| crate::performance::BlockingResource {
                url: u.to_string(),
                transfer_bytes: 2048,
                kind: "script".to_string(),
            })
            .collect();
        report
    }

    fn report_with_headers(url: &str, csp: Option<&str>, hsts: Option<&str>) -> AuditReport {
        AuditReport::new(url.to_string(), WcagLevel::AA, WcagResults::new(), 100).with_security(
            crate::security::SecurityAnalysis {
                score: 80,
                grade: "A".to_string(),
                headers: crate::security::SecurityHeaders {
                    content_security_policy: csp.map(str::to_string),
                    strict_transport_security: hsts.map(str::to_string),
                    ..Default::default()
                },
                ssl: Default::default(),
                issues: vec![],
                recommendations: vec![],
                protection: Default::default(),
                sourcemap_leaks: Default::default(),
                fetch: Default::default(),
            },
        )
    }

    /// Plan 93, step 5: a header missing on some pages, one with two values,
    /// and the uniform ones.
    #[test]
    fn security_headers_compared_across_pages() {
        let reports = vec![
            report_with_headers(
                "https://x.test/a",
                Some("default-src 'self'"),
                Some("max-age=1"),
            ),
            report_with_headers("https://x.test/b", Some("default-src *"), Some("max-age=1")),
            report_with_headers("https://x.test/c", None, Some("max-age=1")),
        ];
        let headers = build_security_header_consistency(&reports);
        let get = |name: &str| headers.iter().find(|h| h.header == name).unwrap();

        let csp = get("Content-Security-Policy");
        assert_eq!(
            (csp.pages_checked, csp.pages_present, csp.distinct_values),
            (3, 2, 2)
        );
        assert_eq!(csp.missing_on, vec!["https://x.test/c".to_string()]);

        let hsts = get("Strict-Transport-Security");
        assert_eq!((hsts.pages_present, hsts.distinct_values), (3, 1));
        assert!(hsts.missing_on.is_empty());

        // Sent nowhere: no page to name as "missing".
        assert!(get("Permissions-Policy").missing_on.is_empty());

        // One measured page compares nothing.
        assert!(build_security_header_consistency(&reports[..1]).is_empty());
    }

    /// Plan 93: a file is counted once per page, versions stay distinct,
    /// a file on one page only is not "shared".
    #[test]
    fn shared_blocking_resources_by_exact_url() {
        let reports = vec![
            report_with_blocking(
                "https://x.test/a",
                &["https://x.test/app.js?v=1", "https://x.test/a.js"],
            ),
            report_with_blocking(
                "https://x.test/b",
                &["https://x.test/app.js?v=1", "https://x.test/app.js?v=1"],
            ),
            report_with_blocking("https://x.test/c", &["https://x.test/app.js?v=2"]),
        ];
        let shared = build_shared_blocking_resources(&reports);
        assert_eq!(shared.len(), 1, "{shared:?}");
        assert_eq!(shared[0].url, "https://x.test/app.js?v=1");
        assert_eq!(shared[0].pages, 2);
        assert_eq!(shared[0].kind, "script");
    }

    fn compact(
        url: &str,
        page_type: Option<&str>,
        score: f32,
        issues: &[&str],
    ) -> CompactUrlSummary {
        CompactUrlSummary {
            url: url.to_string(),
            score,
            grade: String::new(),
            critical_violations: 0,
            total_violations: 0,
            page_type: page_type.map(str::to_string),
            page_attributes: vec![],
            page_semantic_score: None,
            biggest_lever: String::new(),
            topic_terms: vec![],
            top_issues: issues.iter().map(|i| i.to_string()).collect(),
            module_scores: vec![],
        }
    }

    /// Plan 93: group means need at least two pages; unknown types are left
    /// out; the common finding needs two pages of the group.
    #[test]
    fn page_type_groups_have_explicit_size() {
        let details = vec![
            compact("a", Some("Editorial"), 80.0, &["Kontrast", "Alt-Text"]),
            compact("b", Some("Editorial"), 61.0, &["Kontrast"]),
            compact("c", Some("Editorial"), 70.0, &["Labels"]),
            compact("d", Some("Hub"), 90.0, &["Kontrast"]),
            compact("e", None, 10.0, &["Kontrast"]),
        ];
        let groups = build_page_type_groups(&details);
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].page_type, "Editorial");
        assert_eq!(groups[0].pages, 3);
        assert_eq!(groups[0].avg_accessibility, Some(70));
        assert_eq!(groups[0].common_finding, Some(("Kontrast".to_string(), 2)));
        assert_eq!(groups[1].page_type, "Hub");
        assert_eq!(groups[1].avg_accessibility, None);
        assert_eq!(groups[1].common_finding, None);
    }

    #[test]
    fn flags_asset_unminified_on_one_page_and_minified_on_another() {
        let shared = "https://x.test/assets/app.js";
        let reports = vec![
            // Unminified on page A; coverage confirms it also loaded there.
            report_with_perf("https://x.test/a", &[shared], &[], &[shared]),
            // Same URL present (per coverage) but NOT flagged unminified on B.
            report_with_perf("https://x.test/b", &[], &[], &[shared]),
        ];

        let inconsistencies = build_minification_inconsistencies(&reports);

        let entry = inconsistencies
            .iter()
            .find(|i| i.url == shared)
            .expect("expected inconsistency entry for shared script");
        assert_eq!(entry.kind, "script");
        assert_eq!(entry.unminified_on_count, 1);
        assert_eq!(entry.minified_on_count, 1);
        assert_eq!(entry.total_pages_with_asset, 2);
    }

    #[test]
    fn does_not_flag_asset_unminified_on_every_page_it_appears_on() {
        let shared = "https://x.test/assets/legacy.js";
        let reports = vec![
            report_with_perf("https://x.test/a", &[shared], &[], &[shared]),
            report_with_perf("https://x.test/b", &[shared], &[], &[shared]),
        ];

        let inconsistencies = build_minification_inconsistencies(&reports);

        assert!(
            !inconsistencies.iter().any(|i| i.url == shared),
            "consistently-unminified asset must not be reported as an inconsistency"
        );
    }

    #[test]
    fn does_not_flag_asset_that_never_appears_unminified() {
        let shared = "https://x.test/assets/clean.js";
        let reports = vec![
            report_with_perf("https://x.test/a", &[], &[], &[shared]),
            report_with_perf("https://x.test/b", &[], &[], &[shared]),
        ];

        let inconsistencies = build_minification_inconsistencies(&reports);

        assert!(
            !inconsistencies.iter().any(|i| i.url == shared),
            "asset never flagged unminified must not be reported"
        );
    }

    #[test]
    fn empty_input_yields_empty_output() {
        assert!(build_minification_inconsistencies(&[]).is_empty());
    }
}

#[cfg(test)]
mod redirect_chain_issue_tests {
    use super::*;
    use crate::audit::AuditReport;
    use crate::cli::WcagLevel;
    use crate::seo::page_health::{PageHealthAnalysis, RedirectHop};
    use crate::seo::SeoAnalysis;
    use crate::wcag::WcagResults;

    fn report_with_chain(url: &str, hops: &[(u16, &str)], final_url: Option<&str>) -> AuditReport {
        let redirect_chain: Vec<RedirectHop> = hops
            .iter()
            .map(|(status, hop_url)| RedirectHop {
                status: *status,
                url: hop_url.to_string(),
            })
            .collect();
        let seo = SeoAnalysis {
            page_health: Some(PageHealthAnalysis {
                redirect_count: redirect_chain.len() as u32,
                redirect_chain,
                own_final_url: final_url.map(str::to_string),
                ..Default::default()
            }),
            ..Default::default()
        };
        AuditReport::new(url.to_string(), WcagLevel::AA, WcagResults::new(), 100).with_seo(seo)
    }

    #[test]
    fn four_hop_chain_is_flagged_as_long_chain_not_loop() {
        let reports = vec![report_with_chain(
            "https://x.test/a",
            &[
                (301, "https://x.test/a"),
                (301, "https://x.test/b"),
                (302, "https://x.test/c"),
                (301, "https://x.test/d"),
            ],
            Some("https://x.test/final"),
        )];

        let issues = build_redirect_chain_issues(&reports);

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].hop_count, 4);
        assert!(!issues[0].is_loop);
        assert_eq!(
            issues[0].chain,
            vec![
                "https://x.test/a",
                "https://x.test/b",
                "https://x.test/c",
                "https://x.test/d",
                "https://x.test/final",
            ]
        );
    }

    #[test]
    fn chain_revisiting_a_url_is_flagged_as_a_loop() {
        let reports = vec![report_with_chain(
            "https://x.test/a",
            &[
                (301, "https://x.test/a"),
                (302, "https://x.test/b"),
                (301, "https://x.test/a"),
            ],
            None,
        )];

        let issues = build_redirect_chain_issues(&reports);

        assert_eq!(issues.len(), 1);
        assert!(issues[0].is_loop);
    }

    #[test]
    fn two_hop_chain_below_threshold_is_not_flagged() {
        let reports = vec![report_with_chain(
            "https://x.test/a",
            &[(301, "https://x.test/a"), (302, "https://x.test/b")],
            Some("https://x.test/final"),
        )];

        assert!(build_redirect_chain_issues(&reports).is_empty());
    }

    #[test]
    fn no_redirects_is_not_flagged() {
        let reports = vec![report_with_chain("https://x.test/a", &[], None)];

        assert!(build_redirect_chain_issues(&reports).is_empty());
    }

    #[test]
    fn empty_batch_yields_empty_output() {
        assert!(build_redirect_chain_issues(&[]).is_empty());
    }
}

#[cfg(test)]
mod interactive_summary_tests {
    use super::*;
    use crate::audit::normalized::{InteractiveFinding, InteractiveFindingKind};
    use crate::audit::AuditReport;
    use crate::cli::WcagLevel;
    use crate::wcag::WcagResults;

    fn report_with_categories(url: &str, categories: &[&str]) -> AuditReport {
        let mut report = AuditReport::new(url.to_string(), WcagLevel::AA, WcagResults::new(), 100);
        for category in categories {
            report.interactive_findings.push(InteractiveFinding {
                category: category.to_string(),
                kind: InteractiveFindingKind::SkipLinkFocusNotMoved,
                maps_to_finding: None,
                severity: Severity::Medium,
                journey: "skip_link".to_string(),
                before_snapshot_label: None,
                after_snapshot_label: None,
                message: String::new(),
                fix_suggestion: None,
                values: Default::default(),
                uncertainty: None,
            });
        }
        report
    }

    #[test]
    fn categories_with_equal_counts_are_ordered_by_category() {
        let batch = BatchReport::from_reports(
            vec![
                report_with_categories("https://x.test/a", &["TabOrder", "FocusTrap", "SkipLink"]),
                report_with_categories("https://x.test/b", &["SkipLink", "MenuJourney"]),
                report_with_categories("https://x.test/c", &["FormError"]),
            ],
            vec![],
            0,
        );

        let summary = build_batch_presentation(&batch)
            .interactive_summary
            .expect("interactive summary");
        let order: Vec<&str> = summary
            .categories
            .iter()
            .map(|c| c.category.as_str())
            .collect();
        assert_eq!(
            order,
            [
                "SkipLink",
                "FocusTrap",
                "FormError",
                "MenuJourney",
                "TabOrder"
            ]
        );
    }
}
