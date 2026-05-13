//! Batch-report presentation builder.

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
    derive_action_plan, derive_business_impact, derive_execution_priority, impact_score,
    score_to_priority, severity_to_priority,
};
use super::helpers::{build_batch_appendix, build_batch_verdict};
use super::seo::{
    average_page_semantic_score, derive_domain_topics, derive_topic_overlap_pairs,
    extract_page_topics, page_profile_optimization_note,
};

/// Build a complete presentation model from a batch audit report
pub fn build_batch_presentation(batch: &BatchReport) -> BatchPresentation {
    let i18n = I18n::new("de").expect("default locale must always load");
    build_batch_presentation_with_locale(batch, &i18n)
}

/// Locale-aware variant of [`build_batch_presentation`].
pub fn build_batch_presentation_with_locale(batch: &BatchReport, i18n: &I18n) -> BatchPresentation {
    // Normalize all reports early — needed for overall scores, risk, module averages
    let normalized_reports: Vec<_> = batch.reports.iter().map(normalize).collect();
    let mut top_issues = collect_batch_finding_groups(&normalized_reports, i18n.locale());
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

    let action_plan = derive_action_plan(i18n.locale(), &top_issues);

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
            let per_url_groups = normalized_finding_groups(i18n.locale(), nr);
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
                    .seo
                    .as_ref()
                    .and_then(|seo| seo.content_profile.as_ref())
                    .map(|profile| profile.page_classification.primary_type.label().to_string()),
                page_attributes: r
                    .seo
                    .as_ref()
                    .and_then(|seo| seo.content_profile.as_ref())
                    .map(|profile| profile.page_classification.attributes.clone())
                    .unwrap_or_default(),
                page_semantic_score: r
                    .seo
                    .as_ref()
                    .and_then(|seo| seo.content_profile.as_ref())
                    .map(|profile| average_page_semantic_score(&profile.page_classification)),
                biggest_lever: sorted
                    .first()
                    .map(|g| g.title.clone())
                    .or_else(|| {
                        r.seo.as_ref().and_then(|seo| {
                            seo.content_profile
                                .as_ref()
                                .map(|cp| page_profile_optimization_note(i18n.locale(), cp))
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

    let mut page_type_counts: HashMap<String, usize> = HashMap::new();
    let mut page_semantic_scores: Vec<(String, String, u32)> = Vec::new();
    let mut thin_pages = 0usize;
    let mut editorial_pages = 0usize;
    let mut marketing_pages = 0usize;
    for report in &batch.reports {
        if let Some(profile) = report
            .seo
            .as_ref()
            .and_then(|seo| seo.content_profile.as_ref())
        {
            let label = profile.page_classification.primary_type.label().to_string();
            *page_type_counts.entry(label).or_default() += 1;
            let semantic_score = average_page_semantic_score(&profile.page_classification);
            page_semantic_scores.push((
                report.url.clone(),
                profile.page_classification.primary_type.label().to_string(),
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
        distribution_insights.push(
            "Hoher Anteil an Thin-Content-Seiten: Das kann Informationswert und SEO-Potenzial begrenzen."
                .to_string(),
        );
    }
    if editorial_pages == 0 {
        distribution_insights.push(
            "Editoriale Inhaltsseiten fehlen: Wissensaufbau und Suchintentionen werden kaum bedient."
                .to_string(),
        );
    }
    if marketing_pages > 0 && (marketing_pages as f64 / batch.summary.total_urls as f64) >= 0.5 {
        distribution_insights.push(
            "Marketing- und Landingpages dominieren: Mehr strukturierter Tiefeninhalt würde die Domain ausbalancieren."
                .to_string(),
        );
    }
    if distribution_insights.is_empty() && !page_type_distribution.is_empty() {
        distribution_insights.push(
            "Die Seitentypen sind insgesamt ausgewogen verteilt, ohne klar dominierende Schwachmuster."
                .to_string(),
        );
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
            r.seo
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

    // Budget violations: aggregate across all pages
    let budget_summary: Vec<(String, String, usize, String)> = {
        use std::collections::HashMap;
        let mut map: HashMap<String, (String, usize, &str)> = HashMap::new();
        for r in &batch.reports {
            for v in &r.budget_violations {
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
    let verdict_text = build_batch_verdict(i18n, batch.summary.total_urls, average_score);

    // Aggregate module averages
    let module_averages = {
        let mut module_sums: HashMap<String, (u32, usize)> = HashMap::new();
        for nr in &normalized_reports {
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
            if let Some(seo) = &report.seo {
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

    // Worst-case risk level across all URLs
    let (risk_level, risk_summary) = {
        use crate::audit::normalized::RiskLevel;
        let worst = normalized_reports
            .iter()
            .map(|n| n.risk.level)
            .max()
            .unwrap_or(RiskLevel::Low);
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
    let certificate = match average_score {
        95.. => "SEHR GUT",
        85.. => "GUT",
        70.. => "SOLIDE",
        50.. => "AUSBAUFÄHIG",
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
        },
        top_issues: top_issues.into_iter().take(10).collect(),
        issue_frequency,
        action_plan,
        url_ranking,
        url_details,
        url_matrix: build_url_matrix(batch),
        appendix: build_batch_appendix(batch),
    }
}

// ─── URL matrix ─────────────────────────────────────────────────────────────

fn build_url_matrix(batch: &BatchReport) -> Vec<UrlMatrixRow> {
    // Build inbound link map: path → count of pages that link here
    let mut inbound: HashMap<String, usize> = HashMap::new();
    for report in &batch.reports {
        if let Some(seo) = &report.seo {
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
            let title = r.seo.as_ref().and_then(|seo| seo.meta.title.clone());
            let word_count = r
                .seo
                .as_ref()
                .map(|seo| seo.technical.word_count)
                .unwrap_or(0);
            let outbound = r
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
    locale: &str,
) -> Vec<FindingGroup> {
    let mut groups: HashMap<String, NormalizedFindingAccumulator> = HashMap::new();
    for report in normalized_reports {
        for finding in &report.findings {
            let entry = groups
                .entry(finding.aggregation_key.clone())
                .or_insert_with(|| NormalizedFindingAccumulator {
                    finding: finding.clone(),
                    severity: finding.severity,
                    count: 0,
                    urls: Vec::new(),
                });
            entry.count += finding.occurrence_count;
            if finding.severity > entry.severity {
                entry.severity = finding.severity;
            }
            if !entry.urls.contains(&report.url) {
                entry.urls.push(report.url.clone());
            }
        }
    }

    groups
        .values()
        .map(|acc| finding_group_from_normalized(locale, acc))
        .collect()
}

fn normalized_finding_groups(locale: &str, normalized: &NormalizedReport) -> Vec<FindingGroup> {
    normalized
        .findings
        .iter()
        .map(|finding| {
            finding_group_from_normalized(
                locale,
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

fn finding_group_from_normalized(locale: &str, acc: &NormalizedFindingAccumulator) -> FindingGroup {
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
                locale,
                expl.user_impact_for(locale),
                dimension_label,
                acc.severity,
                Some(finding.subcategory.as_str()),
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
        let auto_detected = if locale == "en" {
            "Automatically detected issue.".to_string()
        } else {
            "Automatisch erkanntes Problem.".to_string()
        };
        (
            finding.title.clone(),
            finding.description.clone(),
            finding.user_impact.clone(),
            derive_business_impact(
                locale,
                &finding.user_impact,
                dimension_label,
                acc.severity,
                Some(finding.subcategory.as_str()),
                acc.count,
            ),
            auto_detected,
            finding
                .occurrences
                .iter()
                .find_map(|o| o.fix_suggestion.clone())
                .unwrap_or_else(|| {
                    if locale == "en" {
                        "Review and remediate the affected implementation.".to_string()
                    } else {
                        "Betroffene Umsetzung prüfen und beheben.".to_string()
                    }
                }),
            finding.technical_impact.clone(),
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

    FindingGroup {
        title,
        rule_id: finding.rule_id.clone(),
        wcag_criterion: finding.wcag_criterion.clone(),
        wcag_level: finding.wcag_level.clone(),
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
