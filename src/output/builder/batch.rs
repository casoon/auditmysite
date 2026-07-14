//! Batch-report presentation builder.

use std::cmp::Reverse;
use std::collections::HashMap;

use crate::audit::normalized::NormalizedFinding;
use crate::audit::{normalize, BatchReport, BrokenLinkSeverity, NormalizedReport};
use crate::i18n::I18n;
use crate::output::explanations::get_explanation;
use crate::output::report_model::*;
use crate::seo::profile::PageType;
use crate::util::truncate_url;
use crate::wcag::Severity;

use super::actions::{
    build_narrative_arc, derive_action_plan, derive_business_impact, impact_score,
    localized_finding_text,
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
    let collected = collect_batch_finding_groups(normalized_reports, i18n);
    // Deduplicate findings with the same title across rule sources; prefer
    // non-"unknown." rule_ids, merge occurrence counts.
    let mut seen_titles: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    let mut deduped: Vec<FindingGroup> = Vec::with_capacity(collected.len());
    for group in collected {
        let key = group.title.trim().to_lowercase();
        if let Some(&idx) = seen_titles.get(&key) {
            let existing = &mut deduped[idx];
            if existing.rule_id.starts_with("unknown.") && !group.rule_id.starts_with("unknown.") {
                let merged = existing.occurrence_count + group.occurrence_count;
                *existing = group;
                existing.occurrence_count = merged;
            } else {
                existing.occurrence_count += group.occurrence_count;
            }
        } else {
            seen_titles.insert(key, deduped.len());
            deduped.push(group);
        }
    }
    let mut top_issues = deduped;
    top_issues.sort_by_key(|b| std::cmp::Reverse(impact_score(b)));

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
            categories.sort_by_key(|c| Reverse(c.affected_urls));
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
    url_ranking.sort_by(|a, b| {
        a.score
            .partial_cmp(&b.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let url_details: Vec<CompactUrlSummary> = batch
        .reports
        .iter()
        .zip(normalized_reports.iter())
        .map(|(r, nr)| {
            let per_url_groups = normalized_finding_groups(i18n, nr);
            let mut sorted = per_url_groups;
            sorted.sort_by_key(|b| std::cmp::Reverse(impact_score(b)));
            let top_issue_titles: Vec<String> =
                sorted.iter().take(3).map(|g| g.title.clone()).collect();

            let module_scores = nr
                .module_scores
                .iter()
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
                            .label(i18n.locale() == "en")
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
                            seo.content_profile.as_ref().map(|cp| {
                                page_profile_optimization_note_text(cp, i18n.locale() == "en")
                            })
                        })
                    })
                    .unwrap_or_else(|| {
                        if i18n.locale() == "en" {
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
        (a.1.score as f32)
            .partial_cmp(&(b.1.score as f32))
            .unwrap_or(std::cmp::Ordering::Equal)
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

    let en = i18n.locale() == "en";
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

    // Cross-page duplicate content (identical title / meta description / H1)
    let duplicate_content = build_duplicate_content(&batch.reports);

    // Per-page canonical conflicts (noindex / og:url mismatch)
    let canonical_issues = build_canonical_issues(&batch.reports);

    // Non-reciprocal hreflang relationships among audited pages
    let hreflang_issues = build_hreflang_issues(&batch.reports);

    let (sitemap_http_issues, orphan_sitemap_urls, linked_not_in_sitemap) = batch
        .sitemap_diagnostics
        .as_ref()
        .map(|diagnostics| {
            (
                diagnostics.http_issues.clone(),
                diagnostics.orphan_sitemap_urls.clone(),
                diagnostics.linked_not_in_sitemap.clone(),
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
            let total_blocking: usize = pages_with_data
                .iter()
                .map(|rb| rb.blocking_scripts.len() + rb.blocking_css.len())
                .sum();
            let total_third_party_bytes: u64 =
                pages_with_data.iter().map(|rb| rb.third_party_bytes).sum();
            let pages_with_blocking = pages_with_data
                .iter()
                .filter(|rb| rb.has_blocking())
                .count();
            let en = i18n.locale() == "en";
            let (
                label_pages_analyzed,
                label_pages_blocking,
                label_blocking_total,
                label_third_party,
            ) = if en {
                (
                    "Pages analyzed",
                    "Pages with blocking",
                    "Blocking resources total",
                    "Third-party traffic total",
                )
            } else {
                (
                    "Seiten analysiert",
                    "Seiten mit Blocking",
                    "Blocking-Ressourcen gesamt",
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
                (label_blocking_total.to_string(), total_blocking.to_string()),
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

    let average_score = batch.summary.average_score.round() as u32;
    let verdict_text = build_batch_verdict(i18n, batch);

    // Aggregate module averages
    let module_averages = {
        let mut module_sums: HashMap<String, (u32, usize)> = HashMap::new();
        for nr in normalized_reports {
            for ms in &nr.module_scores {
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
        let en = i18n.locale() == "en";
        let summary = match (worst, en) {
            (RiskLevel::Low, true) => "The audited pages overall show a low accessibility risk.",
            (RiskLevel::Low, false) => "Die geprüften Seiten weisen insgesamt ein geringes Barrierefreiheits-Risiko auf.",
            (RiskLevel::Medium, true) => "Some pages show medium risk. Targeted improvements recommended.",
            (RiskLevel::Medium, false) => "Einzelne Seiten weisen mittleres Risiko auf. Gezielte Verbesserungen empfohlen.",
            (RiskLevel::High, true) => "Several pages show high risk. Timely remediation recommended, especially for WCAG Level A violations.",
            (RiskLevel::High, false) => "Mehrere Seiten haben hohes Risiko. Zeitnahe Behebung empfohlen, besonders bei WCAG-Level-A-Verstößen.",
            (RiskLevel::Critical, true) => "Critical risk across multiple pages. WCAG Level A violations detected automatically — immediate action recommended; manual review required for a defensible legal classification.",
            (RiskLevel::Critical, false) => "Kritisches Risiko über mehrere Seiten. WCAG-Level-A-Verstöße automatisiert erkannt — sofortige Maßnahmen empfohlen, manuelle Prüfung für belastbare rechtliche Einordnung nötig.",
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

    // Certificate and grade are based on the primary WCAG/accessibility score.
    // Thresholds mirror `AccessibilityScorer::calculate_certificate` (#449).
    let certificate = match average_score {
        90.. => "SEHR GUT",
        75.. => "GUT",
        60.. => "STABIL",
        40.. => "AUSBAUFÄHIG",
        _ => "UNGENÜGEND",
    }
    .to_string();

    let grade = match average_score {
        95.. => "A+",
        90.. => "A",
        85.. => "B+",
        80.. => "B",
        70.. => "C",
        60.. => "D",
        _ => "F",
    }
    .to_string();

    let en = i18n.locale() == "en";
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
            average_score: batch.summary.average_score,
            average_overall_score,
            total_violations: batch.summary.total_violations,
            duration_ms: batch.total_duration_ms,
            verdict_text,
            worst_urls,
            best_urls,
            severity_distribution,
            risk_level,
            risk_summary,
            module_averages,
            active_modules,
            domain,
            certificate,
            grade,
            page_type_distribution,
            distribution_insights,
            strongest_content_pages,
            weakest_content_pages,
            top_topics,
            overlap_pairs,
            near_duplicates,
            crawl_links,
            budget_summary,
            render_blocking_summary,
            schema_distribution,
            pages_without_schema,
            duplicate_content,
            canonical_issues,
            hreflang_issues,
            sitemap_http_issues,
            orphan_sitemap_urls,
            linked_not_in_sitemap,
        },
        top_issues: top_issues.into_iter().take(10).collect(),
        issue_frequency,
        action_plan,
        url_ranking,
        url_details,
        url_matrix: build_url_matrix(batch),
        appendix: build_batch_appendix(i18n.locale(), batch, normalized_reports),
        interactive_summary,
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
        let candidates: [(&'static str, Option<&str>); 3] = [
            ("title", seo.meta.title.as_deref()),
            ("meta_description", seo.meta.description.as_deref()),
            ("h1", seo.headings.h1_text.as_deref()),
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
    });
    out
}

/// Detect canonical-tag conflicts per page (#423): a canonical pointing away
/// while the page is `noindex` (mixed signals to crawlers), and a canonical
/// that disagrees with the page's `og:url`. Pure per-page checks aggregated
/// across the batch; no extra network requests.
fn build_canonical_issues(reports: &[crate::audit::AuditReport]) -> Vec<CanonicalIssue> {
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

// ─── Internal batch helpers ──────────────────────────────────────────────────

#[derive(Clone)]
struct NormalizedFindingAccumulator {
    finding: NormalizedFinding,
    severity: Severity,
    count: usize,
    urls: Vec<String>,
}

fn collect_batch_finding_groups(
    normalized_reports: &[NormalizedReport],
    i18n: &I18n,
) -> Vec<FindingGroup> {
    let mut groups: HashMap<String, NormalizedFindingAccumulator> = HashMap::new();
    for report in normalized_reports {
        for finding in &report.findings {
            let base_severity = crate::taxonomy::rules::RULES
                .iter()
                .find(|r| r.id == finding.rule_id)
                .map(|r| r.severity)
                .unwrap_or(finding.severity);
            let entry = groups
                .entry(finding.aggregation_key.clone())
                .or_insert_with(|| NormalizedFindingAccumulator {
                    finding: finding.clone(),
                    severity: base_severity,
                    count: 0,
                    urls: Vec::new(),
                });
            entry.count += finding.occurrence_count;
            if !entry.urls.contains(&report.url) {
                entry.urls.push(report.url.clone());
            }
        }
    }

    groups
        .values()
        .map(|acc| finding_group_from_normalized(i18n, acc))
        .collect()
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
    let explanation =
        get_explanation(&finding.rule_id).or_else(|| get_explanation(&finding.wcag_criterion));
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
            expl.customer_title_for(locale).to_string(),
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
        structural_cause: if acc.count >= 10 {
            if locale == "en" {
                Some(format!(
                    "Root cause: 1 component issue producing {} occurrences. \
                     This is likely a shared template or component — fixing it once \
                     eliminates all occurrences simultaneously.",
                    acc.count
                ))
            } else {
                Some(format!(
                    "Root Cause: 1 Komponentenproblem erzeugt {} Vorkommen. \
                     Wahrscheinlich ein gemeinsam genutztes Template oder eine Komponente — \
                     ein einmaliger Fix behebt alle Vorkommen gleichzeitig.",
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

    #[test]
    fn no_groups_when_all_values_distinct() {
        let reports = vec![
            report_with("https://x.test/a", "One", "A"),
            report_with("https://x.test/b", "Two", "B"),
        ];
        assert!(build_duplicate_content(&reports).is_empty());
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

        let issues = build_canonical_issues(&reports);
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
