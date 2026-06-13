use crate::audit::normalized::{NormalizedReport, SeverityCounts};
use crate::output::report_model::{
    CriticalityTier, DiagnosisBlock, FindingCluster, FindingCriticalityGroup, FindingGroup,
    FindingSeverityTier, FindingSummary,
};

pub(super) fn build_finding_summary(
    locale: &str,
    counts: &SeverityCounts,
    audit_summary: &crate::audit::summary::AuditSummary,
) -> FindingSummary {
    let en = locale == "en";
    let cross_impact_notes = audit_summary
        .cross_impacts
        .iter()
        .map(|c| format!("{}: {}", c.dimensions, c.description))
        .collect();
    let issue_pattern_label = match &audit_summary.issue_pattern {
        crate::audit::summary::IssuePattern::Minimal => {
            if en { "No findings" } else { "Keine Befunde" }.into()
        }
        crate::audit::summary::IssuePattern::SingleDominant => if en {
            "Single dominant issue"
        } else {
            "Einzelnes dominantes Problem"
        }
        .into(),
        crate::audit::summary::IssuePattern::Clustered => if en {
            "Clustered problems"
        } else {
            "Geclusterte Probleme"
        }
        .into(),
        crate::audit::summary::IssuePattern::Scattered => if en {
            "Scattered issues"
        } else {
            "Verteilte Einzelprobleme"
        }
        .into(),
    };
    FindingSummary {
        total: counts.total,
        critical: counts.critical,
        high: counts.high,
        medium: counts.medium,
        low: counts.low,
        verdict: audit_summary.verdict_intro.clone(),
        dominant_issue_note: audit_summary.dominant_issue_note.clone(),
        cross_impact_notes,
        issue_pattern_label,
    }
}

pub(super) fn build_severity_tiers(
    locale: &str,
    findings: &[FindingGroup],
) -> Vec<FindingSeverityTier> {
    use crate::wcag::Severity;

    let en = locale == "en";
    [
        (Severity::Critical, if en { "Critical" } else { "Kritisch" }),
        (Severity::High, if en { "High" } else { "Hoch" }),
        (Severity::Medium, if en { "Medium" } else { "Mittel" }),
        (Severity::Low, if en { "Low" } else { "Niedrig" }),
    ]
    .iter()
    .filter_map(|(sev, label)| {
        let tier_findings: Vec<FindingGroup> = findings
            .iter()
            .filter(|f| f.severity == *sev)
            .cloned()
            .collect();
        if tier_findings.is_empty() {
            return None;
        }
        let total_occurrences = tier_findings.iter().map(|f| f.occurrence_count).sum();
        Some(FindingSeverityTier {
            severity: *sev,
            label: (*label).to_string(),
            findings: tier_findings,
            total_occurrences,
        })
    })
    .collect()
}

pub(super) fn build_criticality_groups(
    locale: &str,
    findings: &[FindingGroup],
) -> Vec<FindingCriticalityGroup> {
    let en = locale == "en";

    [CriticalityTier::Mandatory, CriticalityTier::Optimization]
        .iter()
        .filter_map(|tier| {
            let tier_findings: Vec<FindingGroup> = findings
                .iter()
                .filter(|f| f.criticality_tier == *tier)
                .cloned()
                .collect();
            if tier_findings.is_empty() {
                return None;
            }
            let by_severity = build_severity_tiers(locale, &tier_findings);
            let total_findings = tier_findings.len();
            let total_occurrences = tier_findings.iter().map(|f| f.occurrence_count).sum();
            let (label, eyebrow, intro) = match tier {
                CriticalityTier::Mandatory => {
                    if en {
                        (
                            "Legally relevant barriers (WCAG A/AA)".to_string(),
                            "SECTION 1 · WCAG COMPLIANCE".to_string(),
                            "Accessibility issues under WCAG Level A/AA. Remediating these issues is recommended for compliance with legal standards (like BFSG).".to_string(),
                        )
                    } else {
                        (
                            "Gesetzlich relevante Barrieren (WCAG A/AA)".to_string(),
                            "BEREICH 1 · WCAG-KONFORMITÄT".to_string(),
                            "Barrierefreiheits-Verstöße nach WCAG Level A/AA. Diese Befunde beschreiben Barrieren, deren Behebung für die Erfüllung gesetzlicher Vorgaben (wie dem BFSG) empfohlen ist.".to_string(),
                        )
                    }
                }
                CriticalityTier::Optimization => {
                    if en {
                        (
                            "Additional quality & SEO recommendations".to_string(),
                            "SECTION 2 · OPTIMIZATION".to_string(),
                            "SEO, AI visibility, UX signals, dark mode and further quality benchmarks. These improve findability and user experience.".to_string(),
                        )
                    } else {
                        (
                            "Ergänzende Qualitäts- & SEO-Empfehlungen".to_string(),
                            "BEREICH 2 · OPTIMIERUNG".to_string(),
                            "Suchmaschinenoptimierung (SEO), KI-Sichtbarkeit, UX-Signale, Dark-Mode-Unterstützung und weitere Qualitätskriterien. Diese verbessern die Auffindbarkeit und das allgemeine Nutzererlebnis.".to_string(),
                        )
                    }
                }
            };
            Some(FindingCriticalityGroup {
                tier: *tier,
                label,
                eyebrow,
                intro,
                by_severity,
                total_findings,
                total_occurrences,
            })
        })
        .collect()
}

pub(super) fn build_thematic_clusters(
    locale: &str,
    findings: &[FindingGroup],
) -> Vec<FindingCluster> {
    use std::collections::BTreeMap;

    let en = locale == "en";
    let mut by_dimension: BTreeMap<String, Vec<&FindingGroup>> = BTreeMap::new();
    for f in findings {
        let dim = f.dimension.as_deref().unwrap_or("Accessibility");
        by_dimension.entry(dim.to_string()).or_default().push(f);
    }

    let mut clusters: Vec<FindingCluster> = by_dimension
        .into_iter()
        .filter(|(_, groups)| groups.len() >= 2)
        .map(|(dimension, groups)| {
            let worst_severity = groups
                .iter()
                .map(|g| g.severity)
                .max()
                .unwrap_or(crate::wcag::Severity::Low);
            let occurrence_total: usize = groups.iter().map(|g| g.occurrence_count).sum();
            let label = dimension_cluster_label(en, &dimension, groups.len());
            let severity_label = match worst_severity {
                crate::wcag::Severity::Critical => {
                    if en { "Critical" } else { "Kritisch" }.to_string()
                }
                crate::wcag::Severity::High => if en { "High" } else { "Hoch" }.to_string(),
                crate::wcag::Severity::Medium => if en { "Medium" } else { "Mittel" }.to_string(),
                crate::wcag::Severity::Low => if en { "Low" } else { "Niedrig" }.to_string(),
            };
            FindingCluster {
                label,
                dimension: dimension.clone(),
                finding_count: groups.len(),
                occurrence_total,
                severity_label,
                finding_titles: groups.iter().map(|g| g.title.clone()).collect(),
            }
        })
        .collect();

    clusters.sort_by_key(|c| std::cmp::Reverse(c.occurrence_total));
    clusters
}

fn dimension_cluster_label(en: bool, dimension: &str, count: usize) -> String {
    let count_label = if en {
        format!("{count} findings")
    } else {
        format!("{count} Befunde")
    };
    match dimension {
        "Accessibility" => {
            if en {
                format!("Accessibility barriers ({count_label})")
            } else {
                format!("Barrierefreiheits-Barrieren ({count_label})")
            }
        }
        "SEO" => {
            if en {
                format!("SEO issues ({count_label})")
            } else {
                format!("SEO-Probleme ({count_label})")
            }
        }
        "Performance" => {
            if en {
                format!("Performance issues ({count_label})")
            } else {
                format!("Performance-Probleme ({count_label})")
            }
        }
        "Security" => {
            if en {
                format!("Security gaps ({count_label})")
            } else {
                format!("Sicherheitslücken ({count_label})")
            }
        }
        "Mobile" => {
            if en {
                format!("Mobile issues ({count_label})")
            } else {
                format!("Mobile-Probleme ({count_label})")
            }
        }
        other => format!("{other} ({count_label})"),
    }
}

pub(super) fn build_diagnosis_block(
    locale: &str,
    normalized: &NormalizedReport,
    audit_summary: &crate::audit::summary::AuditSummary,
) -> DiagnosisBlock {
    use std::collections::BTreeMap;
    let en = locale == "en";

    let section_title = if en {
        "System diagnosis".to_string()
    } else {
        "Systemdiagnose".to_string()
    };

    let (pattern_label, pattern_description) = match &audit_summary.issue_pattern {
        crate::audit::summary::IssuePattern::Minimal => (
            if en { "No findings" } else { "Keine Befunde" }.to_string(),
            if en {
                "No measurable accessibility barriers detected."
            } else {
                "Keine messbaren Barrierefreiheits-Barrieren erkannt."
            }
            .to_string(),
        ),
        crate::audit::summary::IssuePattern::SingleDominant => (
            if en {
                "Single dominant problem"
            } else {
                "Einzelnes dominantes Problem"
            }
            .to_string(),
            if en {
                "One rule type accounts for the majority of all critical/high findings. \
                 Fixing this one root cause will have the largest single impact."
            } else {
                "Ein Regeltyp verursacht den Großteil aller kritischen/hohen Findings. \
                 Die Behebung dieser einen Ursache hat den größten Einzeleffekt."
            }
            .to_string(),
        ),
        crate::audit::summary::IssuePattern::Clustered => (
            if en { "Clustered problems" } else { "Geclusterte Probleme" }.to_string(),
            if en {
                "Issues are grouped in related problem areas. \
                 Addressing one cluster reduces several findings at once."
            } else {
                "Probleme konzentrieren sich in zusammenhängenden Bereichen. \
                 Die Behebung eines Clusters reduziert mehrere Findings gleichzeitig."
            }
            .to_string(),
        ),
        crate::audit::summary::IssuePattern::Scattered => (
            if en {
                "Distributed individual issues"
            } else {
                "Verteilte Einzelprobleme"
            }
            .to_string(),
            if en {
                "Issues are spread across many independent rules. \
                 No single root cause dominates — each finding requires individual attention."
            } else {
                "Probleme verteilen sich über viele unabhängige Regeln. \
                 Keine einzelne Ursache dominiert — jedes Finding erfordert individuelle Aufmerksamkeit."
            }
            .to_string(),
        ),
    };

    let mut dim_map: BTreeMap<String, (usize, crate::wcag::Severity)> = BTreeMap::new();
    for f in &normalized.findings {
        let dim = f.dimension.clone();
        let entry = dim_map
            .entry(dim)
            .or_insert((0, crate::wcag::Severity::Low));
        entry.0 += 1;
        if f.severity > entry.1 {
            entry.1 = f.severity;
        }
    }
    let mut category_breakdown: Vec<(String, usize, String)> = dim_map
        .into_iter()
        .map(|(dim, (count, sev))| {
            let sev_label = match sev {
                crate::wcag::Severity::Critical => {
                    if en { "Critical" } else { "Kritisch" }.to_string()
                }
                crate::wcag::Severity::High => if en { "High" } else { "Hoch" }.to_string(),
                crate::wcag::Severity::Medium => if en { "Medium" } else { "Mittel" }.to_string(),
                crate::wcag::Severity::Low => if en { "Low" } else { "Niedrig" }.to_string(),
            };
            (dim, count, sev_label)
        })
        .collect();
    category_breakdown.sort_by_key(|c| std::cmp::Reverse(c.1));

    let dominant_issue = audit_summary
        .dominant_issue
        .as_ref()
        .map(|d| d.title.clone());

    let clusters = {
        let mut by_dim: BTreeMap<String, Vec<(String, usize, crate::wcag::Severity)>> =
            BTreeMap::new();
        for f in &normalized.findings {
            // Stored title is canonical English (#406); re-derive the localized
            // title (taxonomy + SEO headings) for non-English reports.
            let title = crate::output::builder::actions::localized_finding_text(locale, f).0;
            by_dim.entry(f.dimension.clone()).or_default().push((
                title,
                f.occurrence_count,
                f.severity,
            ));
        }
        let mut result: Vec<FindingCluster> = by_dim
            .into_iter()
            .filter(|(_, items)| items.len() >= 2)
            .map(|(dim, items)| {
                let worst = items
                    .iter()
                    .map(|(_, _, s)| *s)
                    .max()
                    .unwrap_or(crate::wcag::Severity::Low);
                let total_occ: usize = items.iter().map(|(_, c, _)| c).sum();
                let sev_label = match worst {
                    crate::wcag::Severity::Critical => {
                        if en { "Critical" } else { "Kritisch" }.to_string()
                    }
                    crate::wcag::Severity::High => if en { "High" } else { "Hoch" }.to_string(),
                    crate::wcag::Severity::Medium => {
                        if en { "Medium" } else { "Mittel" }.to_string()
                    }
                    crate::wcag::Severity::Low => if en { "Low" } else { "Niedrig" }.to_string(),
                };
                FindingCluster {
                    label: dimension_cluster_label(en, &dim, items.len()),
                    dimension: dim.clone(),
                    finding_count: items.len(),
                    occurrence_total: total_occ,
                    severity_label: sev_label,
                    finding_titles: items.into_iter().map(|(t, _, _)| t).collect(),
                }
            })
            .collect();
        result.sort_by_key(|c| std::cmp::Reverse(c.occurrence_total));
        result
    };

    DiagnosisBlock {
        section_title,
        pattern_label,
        pattern_description,
        is_systematic: audit_summary.is_systematic,
        category_breakdown,
        dominant_issue,
        verdict_intro: audit_summary.verdict_intro.clone(),
        clusters,
    }
}
