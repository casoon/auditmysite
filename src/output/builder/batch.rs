//! Batch-report presentation builder.

use std::collections::HashMap;

use crate::audit::{BatchReport, BrokenLinkSeverity};
use crate::output::explanations::get_explanation;
use crate::output::report_model::*;
use crate::seo::profile::PageType;
use crate::taxonomy::RuleLookup;
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
    let all_violations: Vec<_> = batch
        .reports
        .iter()
        .flat_map(|r| r.wcag_results.violations.iter().map(move |v| (v, &r.url)))
        .collect();

    let mut rule_groups: HashMap<String, GroupAccumulator> = HashMap::new();
    for (violation, url) in &all_violations {
        let entry = rule_groups
            .entry(violation.rule.clone())
            .or_insert_with(|| GroupAccumulator {
                rule: violation.rule.clone(),
                rule_name: violation.rule_name.clone(),
                severity: violation.severity,
                count: 0,
                urls: Vec::new(),
            });
        entry.count += 1;
        if !entry.urls.contains(url) {
            entry.urls.push((*url).clone());
        }
        if violation.severity > entry.severity {
            entry.severity = violation.severity;
        }
    }

    let mut top_issues: Vec<FindingGroup> = rule_groups
        .values()
        .map(build_finding_group_from_accumulator)
        .collect();
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

    let action_plan = derive_action_plan(&top_issues);

    let mut url_ranking: Vec<UrlSummary> = batch
        .reports
        .iter()
        .map(|r| {
            let critical_count = r
                .wcag_results
                .violations
                .iter()
                .filter(|v| matches!(v.severity, Severity::Critical | Severity::High))
                .count();
            UrlSummary {
                url: r.url.clone(),
                score: r.score,
                grade: r.grade.clone(),
                critical_violations: critical_count,
                total_violations: r.wcag_results.violations.len(),
                passed: r.passed(),
                priority: score_to_priority(r.score),
            }
        })
        .collect();
    url_ranking.sort_by(|a, b| a.score.partial_cmp(&b.score).unwrap_or(std::cmp::Ordering::Equal));

    let url_details: Vec<CompactUrlSummary> = batch
        .reports
        .iter()
        .map(|r| {
            let per_url_groups = group_violations(&r.wcag_results.violations, &[]);
            let mut sorted = per_url_groups;
            sorted.sort_by_key(|b| std::cmp::Reverse(impact_score(b)));
            let top_issue_titles: Vec<String> =
                sorted.iter().take(3).map(|g| g.title.clone()).collect();

            let mut module_scores = vec![("Accessibility".to_string(), r.score.round() as u32)];
            if let Some(ref p) = r.performance {
                module_scores.push(("Performance".to_string(), p.score.overall));
            }
            if let Some(ref s) = r.seo {
                module_scores.push(("SEO".to_string(), s.score));
            }
            if let Some(ref s) = r.security {
                module_scores.push(("Security".to_string(), s.score));
            }
            if let Some(ref m) = r.mobile {
                module_scores.push(("Mobile".to_string(), m.score));
            }

            let topic_terms = extract_page_topics(r);

            CompactUrlSummary {
                url: r.url.clone(),
                score: r.score,
                grade: r.grade.clone(),
                critical_violations: r
                    .wcag_results
                    .violations
                    .iter()
                    .filter(|v| matches!(v.severity, Severity::Critical | Severity::High))
                    .count(),
                total_violations: r.wcag_results.violations.len(),
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
                                .map(page_profile_optimization_note)
                        })
                    })
                    .unwrap_or_else(|| "Ergebnisse stabil halten".to_string()),
                topic_terms,
                top_issues: top_issue_titles,
                module_scores,
            }
        })
        .collect();

    let mut sorted_by_score: Vec<_> = batch.reports.iter().collect();
    sorted_by_score.sort_by(|a, b| a.score.partial_cmp(&b.score).unwrap_or(std::cmp::Ordering::Equal));
    let worst_urls: Vec<(String, f32)> = sorted_by_score
        .iter()
        .take(3)
        .map(|r| (truncate_url(&r.url, 60), r.score))
        .collect();
    let best_urls: Vec<(String, f32)> = sorted_by_score
        .iter()
        .rev()
        .take(3)
        .map(|r| (truncate_url(&r.url, 60), r.score))
        .collect();

    let verdict_text = build_batch_verdict(batch);

    let severity_distribution = {
        let (mut critical, mut high, mut medium, mut low) = (0usize, 0usize, 0usize, 0usize);
        for (violation, _) in &all_violations {
            match violation.severity {
                Severity::Critical => critical += 1,
                Severity::High => high += 1,
                Severity::Medium => medium += 1,
                Severity::Low => low += 1,
            }
        }
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
            vec![
                (
                    "Seiten analysiert".to_string(),
                    format!("{} von {}", pages_with_data.len(), batch.reports.len()),
                ),
                (
                    "Seiten mit Blocking".to_string(),
                    format!(
                        "{} ({:.0}%)",
                        pages_with_blocking,
                        pages_with_blocking as f64 / n * 100.0
                    ),
                ),
                (
                    "Blocking-Ressourcen gesamt".to_string(),
                    total_blocking.to_string(),
                ),
                (
                    "Third-Party-Traffic gesamt".to_string(),
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

    BatchPresentation {
        cover: CoverData {
            title: "Web Accessibility Batch Audit Report".to_string(),
            url: format!("{} URLs geprüft", batch.summary.total_urls),
            date: chrono::Utc::now().format("%d.%m.%Y").to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        },
        portfolio_summary: PortfolioSummary {
            total_urls: batch.summary.total_urls,
            passed: batch.summary.passed,
            failed: batch.summary.failed,
            average_score: batch.summary.average_score,
            total_violations: batch.summary.total_violations,
            duration_ms: batch.total_duration_ms,
            verdict_text,
            worst_urls,
            best_urls,
            severity_distribution,
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
        },
        top_issues: top_issues.into_iter().take(10).collect(),
        issue_frequency,
        action_plan,
        url_ranking,
        url_details,
        appendix: build_batch_appendix(batch),
    }
}

// ─── Internal batch helpers ──────────────────────────────────────────────────

fn taxonomy_fields(wcag_id: &str) -> (Option<String>, Option<String>, Option<String>, String) {
    if let Some(rule) = RuleLookup::by_legacy_wcag_id(wcag_id) {
        (
            Some(rule.dimension.label().to_string()),
            Some(rule.subcategory.label().to_string()),
            Some(rule.issue_class.label().to_string()),
            rule.id.to_string(),
        )
    } else {
        (None, None, None, wcag_id.to_string())
    }
}

struct GroupAccumulator {
    rule: String,
    rule_name: String,
    severity: Severity,
    count: usize,
    urls: Vec<String>,
}

fn group_violations(
    violations: &[crate::wcag::Violation],
    _url_context: &[&str],
) -> Vec<FindingGroup> {
    let mut groups: HashMap<String, (Vec<&crate::wcag::Violation>, usize)> = HashMap::new();
    for v in violations {
        let entry = groups
            .entry(v.rule.clone())
            .or_insert_with(|| (Vec::new(), 0));
        entry.0.push(v);
        entry.1 += 1;
    }

    groups
        .into_iter()
        .map(|(rule_id, (violations, count))| {
            let first = violations[0];
            let explanation = get_explanation(&rule_id);
            let (dimension, subcategory, issue_class, mapped_rule_id) =
                taxonomy_fields(&rule_id);
            let dimension_label = dimension.as_deref().unwrap_or("Accessibility");

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
                    expl.customer_title.to_string(),
                    expl.customer_description.to_string(),
                    expl.user_impact.to_string(),
                    derive_business_impact(
                        expl.user_impact,
                        dimension_label,
                        first.severity,
                        subcategory.as_deref(),
                    ),
                    expl.typical_cause.to_string(),
                    expl.recommendation.to_string(),
                    expl.technical_note.to_string(),
                    expl.responsible_role,
                    expl.effort_estimate,
                    derive_execution_priority(
                        first.severity,
                        expl.effort_estimate,
                        dimension_label,
                    ),
                )
            } else {
                (
                    format!("{} — {}", first.rule, first.rule_name),
                    first.message.clone(),
                    "Nutzer mit Einschränkungen können betroffen sein.".to_string(),
                    derive_business_impact(
                        "Nutzer mit Einschränkungen können betroffen sein.",
                        dimension_label,
                        first.severity,
                        subcategory.as_deref(),
                    ),
                    "Automatisch erkanntes Problem.".to_string(),
                    first
                        .fix_suggestion
                        .clone()
                        .unwrap_or_else(|| "Bitte prüfen und beheben.".to_string()),
                    first.fix_suggestion.clone().unwrap_or_default(),
                    Role::Development,
                    Effort::Medium,
                    derive_execution_priority(first.severity, Effort::Medium, dimension_label),
                )
            };

            let examples = explanation.map(|e| e.examples()).unwrap_or_default();
            let location_hints = violations
                .iter()
                .filter_map(|occ| {
                    occ.selector
                        .clone()
                        .or_else(|| Some(format!("AX-Node {}", occ.node_id)))
                })
                .take(5)
                .collect();

            FindingGroup {
                title,
                rule_id: mapped_rule_id,
                wcag_criterion: rule_id,
                wcag_level: format!("{:?}", first.level),
                dimension,
                subcategory,
                issue_class,
                severity: first.severity,
                priority: severity_to_priority(first.severity),
                customer_description: customer_desc,
                user_impact: user_impact_text,
                business_impact,
                typical_cause,
                recommendation,
                technical_note,
                occurrence_count: count,
                affected_urls: Vec::new(),
                affected_elements: count,
                location_hints,
                responsible_role: role,
                effort,
                execution_priority,
                examples,
            }
        })
        .collect()
}

fn build_finding_group_from_accumulator(acc: &GroupAccumulator) -> FindingGroup {
    let explanation = get_explanation(&acc.rule);
    let (dimension, subcategory, issue_class, mapped_rule_id) = taxonomy_fields(&acc.rule);
    let dimension_label = dimension.as_deref().unwrap_or("Accessibility");
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
            expl.customer_title.to_string(),
            expl.customer_description.to_string(),
            expl.user_impact.to_string(),
            derive_business_impact(expl.user_impact, dimension_label, acc.severity, None),
            expl.typical_cause.to_string(),
            expl.recommendation.to_string(),
            expl.technical_note.to_string(),
            expl.responsible_role,
            expl.effort_estimate,
            derive_execution_priority(acc.severity, expl.effort_estimate, dimension_label),
        )
    } else {
        (
            format!("{} — {}", acc.rule, acc.rule_name),
            String::new(),
            String::new(),
            derive_business_impact("", dimension_label, acc.severity, None),
            String::new(),
            String::new(),
            String::new(),
            Role::Development,
            Effort::Medium,
            derive_execution_priority(acc.severity, Effort::Medium, dimension_label),
        )
    };
    let examples = explanation.map(|e| e.examples()).unwrap_or_default();
    let location_hints = Vec::new();

    FindingGroup {
        title,
        rule_id: mapped_rule_id,
        wcag_criterion: acc.rule.clone(),
        wcag_level: String::new(),
        dimension,
        subcategory,
        issue_class,
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
        location_hints,
        responsible_role: role,
        effort,
        execution_priority,
        examples,
    }
}
