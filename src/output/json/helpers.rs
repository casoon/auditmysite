use super::*;

pub(super) fn build_wcag_coverage_summary(normalized: &NormalizedReport) -> WcagCoverageSummary {
    build_wcag_coverage_for_level(&normalized.wcag_level.to_string())
}

pub(super) fn build_wcag_coverage_for_level(level: &str) -> WcagCoverageSummary {
    let (automated, total) = crate::wcag::coverage::coverage_stats();
    WcagCoverageSummary {
        level: format!("WCAG 2.1 {level}"),
        automated_criteria: automated,
        manual_review_criteria: crate::wcag::coverage::MANUAL_REVIEW_CRITERIA.len(),
        total_wcag_aa_criteria: total,
        note: "Automated score covers detectable criteria only; context-dependent WCAG criteria require manual review.".to_string(),
    }
}

pub(super) fn build_accessibility_score_breakdown(
    reports: &[NormalizedReport],
) -> Vec<AccessibilityScoreComponent> {
    const AREAS: [(&str, u32); 8] = [
        ("Semantics", 15),
        ("Forms", 15),
        ("Keyboard", 15),
        ("Focus management", 10),
        ("Images / alternative text", 15),
        ("ARIA", 15),
        ("Heading structure", 8),
        ("Landmarks / page structure", 7),
    ];

    AREAS
        .iter()
        .map(|(area, weight_pct)| {
            // Logarithmic occurrence penalty with a soft floor (#485). The old
            // linear `severity_weight * occurrence_count` saturated at 100 after a
            // handful of findings, collapsing whole areas to 0 on large, mostly
            // compliant pages (e.g. gov.uk Forms). Each additional occurrence now
            // contributes progressively less, and the per-area loss is capped below
            // 100 so areas stay diagnostic — 1 vs. 100 violations remain
            // distinguishable instead of all flatlining at 0.
            let mut penalty = 0f64;
            let mut driver: Option<(&str, usize)> = None;

            for finding in reports.iter().flat_map(|report| report.findings.iter()) {
                if score_area_for_finding(finding) != *area {
                    continue;
                }
                let severity_weight = match finding.severity {
                    crate::taxonomy::Severity::Critical => 20.0,
                    crate::taxonomy::Severity::High => 14.0,
                    crate::taxonomy::Severity::Medium => 8.0,
                    crate::taxonomy::Severity::Low => 4.0,
                };
                let occ = finding.occurrence_count.max(1) as f64;
                penalty += severity_weight * (1.0 + occ.ln());
                if driver
                    .map(|(_, count)| finding.occurrence_count > count)
                    .unwrap_or(true)
                {
                    driver = Some((&finding.title, finding.occurrence_count));
                }
            }

            // Soft cap at 90: the worst areas floor at a score of 10 rather than 0.
            let estimated_lost_points = (penalty.round() as u32).min(90);
            AccessibilityScoreComponent {
                area: (*area).to_string(),
                score: 100u32.saturating_sub(estimated_lost_points),
                weight_pct: *weight_pct,
                estimated_lost_points,
                main_driver: driver
                    .map(|(title, count)| format!("{title} ({count} occurrences)"))
                    .unwrap_or_else(|| "No detected driver".to_string()),
            }
        })
        .collect()
}

pub(super) fn score_area_for_finding(
    finding: &crate::audit::normalized::NormalizedFinding,
) -> &'static str {
    let key = format!(
        "{} {} {} {}",
        finding.rule_id.to_ascii_lowercase(),
        finding.subcategory.to_ascii_lowercase(),
        finding.title.to_ascii_lowercase(),
        finding.description.to_ascii_lowercase()
    );
    if key.contains("form") || key.contains("label") || key.contains("input") {
        "Forms"
    } else if key.contains("keyboard") || key.contains("tastatur") {
        "Keyboard"
    } else if key.contains("focus") || key.contains("fokus") {
        "Focus management"
    } else if key.contains("alt") || key.contains("image") || key.contains("bild") {
        "Images / alternative text"
    } else if key.contains("aria") || key.contains("role") {
        "ARIA"
    } else if key.contains("heading") || key.contains("überschrift") || key.contains("h1") {
        "Heading structure"
    } else if key.contains("landmark") || key.contains("main") || key.contains("navigation") {
        "Landmarks / page structure"
    } else {
        "Semantics"
    }
}

pub(super) fn build_management_risks(reports: &[NormalizedReport]) -> Vec<ManagementRisk> {
    let legal_flags: usize = reports.iter().map(|r| r.risk.legal_flags).sum();
    let critical: usize = reports.iter().map(|r| r.severity_counts.critical).sum();
    let high: usize = reports.iter().map(|r| r.severity_counts.high).sum();
    let avg = average_accessibility_score(reports);
    let seo = average_module_score_from_reports(reports, "SEO");
    let perf = average_module_score_from_reports(reports, "Performance");
    let mobile = average_module_score_from_reports(reports, "Mobile");
    let component_findings = reports
        .iter()
        .flat_map(|r| r.findings.iter())
        .filter(|f| f.occurrence_count >= 10 || f.complexity == "high")
        .count();

    vec![
        ManagementRisk {
            dimension: "Legal / BFSG-EAA".to_string(),
            level: if legal_flags > 0 || critical > 0 {
                "high"
            } else if high > 0 {
                "medium"
            } else {
                "low"
            }
            .to_string(),
            rationale: format!(
                "{legal_flags} legal flags, {critical} critical and {high} high WCAG findings detected automatically."
            ),
        },
        ManagementRisk {
            dimension: "Conversion / usability".to_string(),
            level: if avg < 60 || critical > 0 || perf.is_some_and(|s| s < 50) {
                "high"
            } else if avg < 80 || high > 0 || mobile.is_some_and(|s| s < 75) {
                "medium"
            } else {
                "low"
            }
            .to_string(),
            rationale: format!(
                "Average accessibility score is {avg}/100; performance {:?}, mobile {:?}.",
                perf, mobile
            ),
        },
        ManagementRisk {
            dimension: "SEO / visibility".to_string(),
            level: risk_level_from_optional_score(seo),
            rationale: seo
                .map(|score| format!("Average SEO score is {score}/100."))
                .unwrap_or_else(|| "SEO module was not run.".to_string()),
        },
        ManagementRisk {
            dimension: "Trust / brand".to_string(),
            level: if critical > 0 || avg < 50 {
                "high"
            } else if high > 0 || avg < 75 {
                "medium"
            } else {
                "low"
            }
            .to_string(),
            rationale: "Accessibility barriers can reduce perceived reliability and inclusiveness.".to_string(),
        },
        ManagementRisk {
            dimension: "Project risk".to_string(),
            level: if component_findings >= 3 {
                "high"
            } else if component_findings > 0 {
                "medium"
            } else {
                "low"
            }
            .to_string(),
            rationale: format!(
                "{component_findings} likely component or template {} coordinated remediation.",
                if component_findings == 1 {
                    "issue needs"
                } else {
                    "issues need"
                }
            ),
        },
    ]
}

pub(super) fn build_decision_actions(reports: &[NormalizedReport]) -> Vec<DecisionAction> {
    let mut findings: Vec<_> = reports.iter().flat_map(|r| r.findings.iter()).collect();
    findings.sort_by(|a, b| {
        b.priority_score
            .partial_cmp(&a.priority_score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| b.occurrence_count.cmp(&a.occurrence_count))
    });
    findings
        .into_iter()
        .take(8)
        .map(|finding| DecisionAction {
            title: finding.title.clone(),
            risk: format!("{:?}", finding.severity).to_lowercase(),
            priority: finding.remediation_priority.clone(),
            complexity: finding.complexity.clone(),
            occurrence_count: finding.occurrence_count,
            root_cause: if finding.occurrence_count >= 10 {
                "Likely shared component or template".to_string()
            } else {
                finding.subcategory.clone()
            },
            expected_impact: finding.expected_impact.clone(),
        })
        .collect()
}

pub(super) fn build_internal_comparison(reports: &[NormalizedReport]) -> InternalComparison {
    let module_names = {
        let mut names = std::collections::BTreeSet::new();
        for report in reports {
            for module in &report.module_scores {
                names.insert(module.name.clone());
            }
        }
        names
    };

    let module_extremes = module_names
        .into_iter()
        .filter_map(|module| {
            let mut scored: Vec<(&NormalizedReport, u32)> = reports
                .iter()
                .filter_map(|report| {
                    report
                        .module_scores
                        .iter()
                        .find(|m| m.name == module)
                        .map(|m| (report, m.score))
                })
                .collect();
            if scored.is_empty() {
                return None;
            }
            scored.sort_by_key(|(_, score)| *score);
            let (worst_report, worst_score) = scored.first().copied()?;
            let (best_report, best_score) = scored.last().copied()?;
            Some(ModuleExtreme {
                module,
                best_url: best_report.url.clone(),
                best_score,
                worst_url: worst_report.url.clone(),
                worst_score,
            })
        })
        .collect();

    let avg = average_accessibility_score(reports);
    let outlier_urls = reports
        .iter()
        .filter_map(|report| {
            let delta = report.score as i32 - avg as i32;
            (delta <= -15).then(|| UrlOutlier {
                url: report.url.clone(),
                accessibility_score: report.score,
                batch_average: avg,
                delta_points: delta,
                reason: "Accessibility score is at least 15 points below the batch average."
                    .to_string(),
            })
        })
        .collect();

    let mut root_map: std::collections::BTreeMap<
        String,
        (usize, std::collections::BTreeSet<String>),
    > = std::collections::BTreeMap::new();
    for report in reports {
        for finding in &report.findings {
            let entry = root_map
                .entry(finding.title.clone())
                .or_insert_with(|| (0, std::collections::BTreeSet::new()));
            entry.0 += finding.occurrence_count;
            entry.1.insert(report.url.clone());
        }
    }
    let mut root_causes: Vec<_> = root_map
        .into_iter()
        .map(|(title, (occurrence_count, urls))| RootCauseSummary {
            title,
            occurrence_count,
            affected_urls: urls.len(),
            classification: if urls.len() >= 2 || occurrence_count >= 10 {
                "likely_template_or_component".to_string()
            } else {
                "page_specific".to_string()
            },
        })
        .collect();
    root_causes.sort_by(|a, b| {
        b.affected_urls
            .cmp(&a.affected_urls)
            .then_with(|| b.occurrence_count.cmp(&a.occurrence_count))
    });
    root_causes.truncate(10);

    InternalComparison {
        module_extremes,
        outlier_urls,
        root_causes,
    }
}

pub(super) fn average_accessibility_score(reports: &[NormalizedReport]) -> u32 {
    if reports.is_empty() {
        0
    } else {
        reports.iter().map(|r| r.score).sum::<u32>() / reports.len() as u32
    }
}

pub(super) fn average_module_score_from_reports(
    reports: &[NormalizedReport],
    module_name: &str,
) -> Option<u32> {
    let scores: Vec<u32> = reports
        .iter()
        .filter_map(|report| {
            report
                .module_scores
                .iter()
                .find(|module| module.name == module_name)
                .map(|module| module.score)
        })
        .collect();
    (!scores.is_empty()).then(|| scores.iter().sum::<u32>() / scores.len() as u32)
}

pub(super) fn risk_level_from_optional_score(score: Option<u32>) -> String {
    match score {
        Some(score) if score < 60 => "high",
        Some(score) if score < 80 => "medium",
        Some(_) => "low",
        None => "unknown",
    }
    .to_string()
}

/// Number of distinct violated rules across all finding categories (issue #254).
pub(super) fn distinct_rule_count(
    findings: &[crate::audit::normalized::NormalizedFinding],
) -> usize {
    findings
        .iter()
        .map(|f| f.rule_id.as_str())
        .collect::<std::collections::HashSet<_>>()
        .len()
}

/// Occurrence counts across ALL finding categories (WCAG + SEO), by severity
/// (issue #255). Distinct from `NormalizedReport.occurrence_counts`, which stays
/// WCAG-only because it drives risk classification (`SiteState`).
pub(super) fn all_category_occurrence_counts(
    findings: &[crate::audit::normalized::NormalizedFinding],
) -> crate::audit::normalized::SeverityCounts {
    use crate::taxonomy::Severity;
    let occ = |sev: Severity| -> usize {
        findings
            .iter()
            .filter(|f| f.severity == sev)
            .map(|f| f.occurrence_count)
            .sum()
    };
    crate::audit::normalized::SeverityCounts {
        critical: occ(Severity::Critical),
        high: occ(Severity::High),
        medium: occ(Severity::Medium),
        low: occ(Severity::Low),
        total: findings.iter().map(|f| f.occurrence_count).sum(),
    }
}

pub(super) fn aggregate_severity(pages: &[PageEntry]) -> crate::audit::normalized::SeverityCounts {
    crate::audit::normalized::SeverityCounts {
        critical: pages.iter().map(|p| p.severity_counts.critical).sum(),
        high: pages.iter().map(|p| p.severity_counts.high).sum(),
        medium: pages.iter().map(|p| p.severity_counts.medium).sum(),
        low: pages.iter().map(|p| p.severity_counts.low).sum(),
        total: pages.iter().map(|p| p.severity_counts.total).sum(),
    }
}

pub(super) fn aggregate_occurrences(
    pages: &[PageEntry],
) -> crate::audit::normalized::SeverityCounts {
    crate::audit::normalized::SeverityCounts {
        critical: pages.iter().map(|p| p.occurrence_counts.critical).sum(),
        high: pages.iter().map(|p| p.occurrence_counts.high).sum(),
        medium: pages.iter().map(|p| p.occurrence_counts.medium).sum(),
        low: pages.iter().map(|p| p.occurrence_counts.low).sum(),
        total: pages.iter().map(|p| p.occurrence_counts.total).sum(),
    }
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

pub(super) fn avg_module_score(pages: &[PageEntry], name: &str) -> Option<u32> {
    let scores: Vec<u32> = pages
        .iter()
        .flat_map(|p| p.module_scores.iter())
        .filter(|m| m.name == name)
        .map(|m| m.score)
        .collect();
    if scores.is_empty() {
        None
    } else {
        // Round rather than truncate so this matches the PDF's module-average
        // computation (builder/batch.rs) — truncation could show a different
        // score band (e.g. 74 amber vs 75 green) for the same underlying value
        // (#QA-026).
        let sum: u32 = scores.iter().sum();
        Some((sum as f64 / scores.len() as f64).round() as u32)
    }
}

pub(super) fn with_normalized_score(
    mut value: serde_json::Value,
    normalized: &NormalizedReport,
    module_name: &str,
) -> serde_json::Value {
    let Some(entry) = normalized
        .module_scores
        .iter()
        .find(|m| m.name == module_name)
    else {
        return value;
    };

    if let Some(obj) = value.as_object_mut() {
        if module_name == "Performance" {
            if let Some(existing) = obj.remove("score") {
                obj.insert("score_details".to_string(), existing);
            }
        }
        obj.insert("score".to_string(), serde_json::json!(entry.score));
        obj.insert("grade".to_string(), serde_json::json!(entry.grade));
    }

    value
}

pub(super) fn inject_grade(mut value: serde_json::Value, score: u32) -> serde_json::Value {
    let grade = crate::audit::AccessibilityScorer::calculate_grade(score as f32);
    if let Some(obj) = value.as_object_mut() {
        obj.insert("grade".to_string(), serde_json::json!(grade));
    }
    value
}

pub(super) fn inject_unused_js_bytes(
    mut value: serde_json::Value,
    raw: &crate::audit::PerformanceResults,
) -> serde_json::Value {
    let Some(cov) = &raw.coverage else {
        return value;
    };
    if let Some(obj) = value.as_object_mut() {
        if let Some(cov_val) = obj.get_mut("coverage") {
            if let Some(cov_obj) = cov_val.as_object_mut() {
                cov_obj.insert(
                    "unused_js_bytes".to_string(),
                    serde_json::json!(cov.unused_js.unused_bytes),
                );
            }
        }
    }
    value
}

pub(super) fn with_measurement_type(
    mut value: serde_json::Value,
    measurement_type: &str,
) -> serde_json::Value {
    if let Some(obj) = value.as_object_mut() {
        obj.insert(
            "measurement_type".to_string(),
            serde_json::json!(measurement_type),
        );
    }
    value
}

pub(super) fn batch_report_timestamp(batch_report: &BatchReport) -> DateTime<Utc> {
    batch_report
        .reports
        .iter()
        .map(|report| report.timestamp)
        .max()
        .unwrap_or(DateTime::<Utc>::UNIX_EPOCH)
}
