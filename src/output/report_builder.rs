//! Report builder — transforms raw audit data into presentation models
//!
//! This module is the core of the report overhaul: it takes raw AuditReport / BatchReport
//! data and produces a structured, customer-facing ReportPresentation / BatchPresentation
//! with grouped findings, aggregated statistics, and explanatory text.

use std::collections::HashMap;

use crate::audit::{AuditReport, BatchReport};
use crate::output::explanations::get_explanation;
use crate::output::report_model::*;
use crate::util::truncate_url;
use crate::wcag::Severity;

fn truncate_text(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        text.to_string()
    } else {
        let truncated: String = text.chars().take(max_chars).collect();
        format!("{}…", truncated.trim_end())
    }
}

fn capitalize_severity(severity: &Severity) -> String {
    match severity {
        Severity::Critical => "Critical".to_string(),
        Severity::Serious => "Serious".to_string(),
        Severity::Moderate => "Moderate".to_string(),
        Severity::Minor => "Minor".to_string(),
    }
}

// ─── Single Report ──────────────────────────────────────────────────────────

/// Build a complete presentation model from a single audit report
pub fn build_single_presentation(report: &AuditReport) -> ReportPresentation {
    let finding_groups = group_violations(&report.wcag_results.violations, &[]);
    let mut sorted_groups = finding_groups;
    sorted_groups.sort_by(|a, b| impact_score(b).cmp(&impact_score(a)));

    // Cross-reference: SEO says lang exists but WCAG 3.1.1 fires → add note
    if report.seo.as_ref().map_or(false, |s| s.technical.has_lang) {
        for group in &mut sorted_groups {
            if group.wcag_criterion == "3.1.1" {
                group.technical_note.push_str(
                    "\n\nHinweis: Die SEO-Analyse hat ein lang-Attribut erkannt. \
                     Der WCAG-Check prüft über den Accessibility Tree und kann \
                     unter bestimmten Umständen abweichen. Bitte manuell verifizieren.",
                );
            }
        }
    }

    let top_findings: Vec<FindingGroup> = sorted_groups.iter().take(5).cloned().collect();
    let positive_aspects = derive_positive_aspects(report);
    let action_plan = derive_action_plan(&sorted_groups);

    ReportPresentation {
        cover: CoverData {
            title: "Web Accessibility Audit Report".to_string(),
            url: report.url.clone(),
            date: report.timestamp.format("%d.%m.%Y").to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        },
        brief_verdict: build_brief_verdict(report, &top_findings),
        methodology: build_methodology(&report.url),
        executive_summary: build_executive_summary(report, &top_findings, &positive_aspects),
        top_findings: top_findings.clone(),
        score_breakdown: build_score_breakdown(report),
        accessibility_details: sorted_groups.clone(),
        module_details: build_module_details(report),
        action_plan,
        positive_aspects,
        appendix: build_appendix(report),
    }
}

// ─── Batch Report ───────────────────────────────────────────────────────────

/// Build a complete presentation model from a batch audit report
pub fn build_batch_presentation(batch: &BatchReport) -> BatchPresentation {
    // Aggregate all violations across all URLs
    let all_violations: Vec<_> = batch
        .reports
        .iter()
        .flat_map(|r| {
            r.wcag_results.violations.iter().map(move |v| (v, &r.url))
        })
        .collect();

    // Group violations with URL tracking
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
        // Use the highest severity seen
        if violation.severity > entry.severity {
            entry.severity = violation.severity;
        }
    }

    let mut top_issues: Vec<FindingGroup> = rule_groups
        .values()
        .map(|acc| build_finding_group_from_accumulator(acc))
        .collect();
    top_issues.sort_by(|a, b| impact_score(b).cmp(&impact_score(a)));

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

    // URL ranking sorted by score (worst first)
    let mut url_ranking: Vec<UrlSummary> = batch
        .reports
        .iter()
        .map(|r| {
            let critical_count = r
                .wcag_results
                .violations
                .iter()
                .filter(|v| matches!(v.severity, Severity::Critical | Severity::Serious))
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
    url_ranking.sort_by(|a, b| a.score.partial_cmp(&b.score).unwrap());

    // Compact per-URL summaries
    let url_details: Vec<CompactUrlSummary> = batch
        .reports
        .iter()
        .map(|r| {
            let per_url_groups = group_violations(&r.wcag_results.violations, &[]);
            let mut sorted = per_url_groups;
            sorted.sort_by(|a, b| impact_score(b).cmp(&impact_score(a)));
            let top_issue_titles: Vec<String> = sorted.iter().take(3).map(|g| g.title.clone()).collect();

            let mut module_scores = Vec::new();
            module_scores.push(("Accessibility".to_string(), r.score.round() as u32));
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

            CompactUrlSummary {
                url: r.url.clone(),
                score: r.score,
                grade: r.grade.clone(),
                top_issues: top_issue_titles,
                module_scores,
            }
        })
        .collect();

    // Worst/best URLs for portfolio summary
    let mut sorted_by_score: Vec<_> = batch.reports.iter().collect();
    sorted_by_score.sort_by(|a, b| a.score.partial_cmp(&b.score).unwrap());
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

    // Calculate severity distribution across all violations
    let severity_distribution = {
        let mut critical = 0usize;
        let mut serious = 0usize;
        let mut moderate = 0usize;
        let mut minor = 0usize;
        for (violation, _) in &all_violations {
            match violation.severity {
                Severity::Critical => critical += 1,
                Severity::Serious => serious += 1,
                Severity::Moderate => moderate += 1,
                Severity::Minor => minor += 1,
            }
        }
        SeverityDistribution { critical, serious, moderate, minor }
    };

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
        },
        top_issues: top_issues.into_iter().take(10).collect(),
        issue_frequency,
        action_plan,
        url_ranking,
        url_details,
        appendix: build_batch_appendix(batch),
    }
}

// ─── Internal helpers ───────────────────────────────────────────────────────

struct GroupAccumulator {
    rule: String,
    rule_name: String,
    severity: Severity,
    count: usize,
    urls: Vec<String>,
}

/// Group violations by rule ID and enrich with explanations
fn group_violations(
    violations: &[crate::wcag::Violation],
    _url_context: &[&str],
) -> Vec<FindingGroup> {
    let mut groups: HashMap<String, (Vec<&crate::wcag::Violation>, usize)> = HashMap::new();

    for v in violations {
        let entry = groups.entry(v.rule.clone()).or_insert_with(|| (Vec::new(), 0));
        entry.0.push(v);
        entry.1 += 1;
    }

    groups
        .into_iter()
        .map(|(rule_id, (violations, count))| {
            let first = violations[0];
            let explanation = get_explanation(&rule_id);

            let (title, customer_desc, user_impact_text, typical_cause, recommendation, technical_note, role, effort) =
                if let Some(expl) = explanation {
                    (
                        expl.customer_title.to_string(),
                        expl.customer_description.to_string(),
                        expl.user_impact.to_string(),
                        expl.typical_cause.to_string(),
                        expl.recommendation.to_string(),
                        expl.technical_note.to_string(),
                        expl.responsible_role,
                        expl.effort_estimate,
                    )
                } else {
                    // Fallback for rules without explanation
                    (
                        format!("{} — {}", first.rule, first.rule_name),
                        first.message.clone(),
                        "Nutzer mit Einschränkungen können betroffen sein.".to_string(),
                        "Automatisch erkanntes Problem.".to_string(),
                        first.fix_suggestion.clone().unwrap_or_else(|| "Bitte prüfen und beheben.".to_string()),
                        first.fix_suggestion.clone().unwrap_or_default(),
                        Role::Development,
                        Effort::Medium,
                    )
                };

            let examples = explanation
                .map(|e| e.examples())
                .unwrap_or_default();

            FindingGroup {
                title,
                wcag_criterion: rule_id,
                wcag_level: format!("{:?}", first.level),
                severity: first.severity,
                priority: severity_to_priority(first.severity),
                customer_description: customer_desc,
                user_impact: user_impact_text,
                typical_cause,
                recommendation,
                technical_note,
                occurrence_count: count,
                affected_urls: Vec::new(),
                affected_elements: count,
                responsible_role: role,
                effort: effort,
                examples,
            }
        })
        .collect()
}

fn build_finding_group_from_accumulator(acc: &GroupAccumulator) -> FindingGroup {
    let explanation = get_explanation(&acc.rule);

    let (title, customer_desc, user_impact_text, typical_cause, recommendation, technical_note, role, effort) =
        if let Some(expl) = explanation {
            (
                expl.customer_title.to_string(),
                expl.customer_description.to_string(),
                expl.user_impact.to_string(),
                expl.typical_cause.to_string(),
                expl.recommendation.to_string(),
                expl.technical_note.to_string(),
                expl.responsible_role,
                expl.effort_estimate,
            )
        } else {
            (
                format!("{} — {}", acc.rule, acc.rule_name),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                Role::Development,
                Effort::Medium,
            )
        };

    let examples = explanation.map(|e| e.examples()).unwrap_or_default();

    FindingGroup {
        title,
        wcag_criterion: acc.rule.clone(),
        wcag_level: String::new(), // Not available in accumulator
        severity: acc.severity,
        priority: severity_to_priority(acc.severity),
        customer_description: customer_desc,
        user_impact: user_impact_text,
        typical_cause,
        recommendation,
        technical_note,
        occurrence_count: acc.count,
        affected_urls: acc.urls.clone(),
        affected_elements: acc.count,
        responsible_role: role,
        effort,
        examples,
    }
}

fn severity_to_priority(severity: Severity) -> Priority {
    match severity {
        Severity::Critical => Priority::Critical,
        Severity::Serious => Priority::High,
        Severity::Moderate => Priority::Medium,
        Severity::Minor => Priority::Low,
    }
}

fn score_to_priority(score: f32) -> Priority {
    if score < 50.0 {
        Priority::Critical
    } else if score < 70.0 {
        Priority::High
    } else if score < 85.0 {
        Priority::Medium
    } else {
        Priority::Low
    }
}

fn impact_score(group: &FindingGroup) -> u32 {
    let severity_weight = match group.severity {
        Severity::Critical => 4,
        Severity::Serious => 3,
        Severity::Moderate => 2,
        Severity::Minor => 1,
    };
    severity_weight * group.occurrence_count as u32
}

fn build_brief_verdict(report: &AuditReport, top_findings: &[FindingGroup]) -> BriefVerdict {
    let critical_count = report
        .wcag_results
        .violations
        .iter()
        .filter(|v| matches!(v.severity, Severity::Critical | Severity::Serious))
        .count();

    let verdict_text = if report.score >= 90.0 {
        format!(
            "Die Website {} erreicht mit {:.0}/100 Punkten ein sehr gutes Ergebnis. \
             Die Barrierefreiheit ist weitgehend gewährleistet.",
            report.url, report.score
        )
    } else if report.score >= 70.0 {
        format!(
            "Die Website {} erreicht {:.0}/100 Punkte — eine solide Basis, \
             aber mit relevanten Barrieren, die behoben werden sollten.",
            report.url, report.score
        )
    } else if report.score >= 50.0 {
        format!(
            "Die Website {} erreicht nur {:.0}/100 Punkte. \
             Es bestehen erhebliche Barrierefreiheitsprobleme, die zeitnah behoben werden müssen.",
            report.url, report.score
        )
    } else {
        format!(
            "Die Website {} erreicht nur {:.0}/100 Punkte. \
             Die Barrierefreiheit ist stark eingeschränkt — dringender Handlungsbedarf.",
            report.url, report.score
        )
    };

    let top_actions: Vec<String> = top_findings
        .iter()
        .take(3)
        .map(|f| f.recommendation.clone())
        .collect();

    BriefVerdict {
        score: report.score,
        grade: report.grade.clone(),
        verdict_text,
        critical_count,
        total_violations: report.wcag_results.violations.len(),
        top_actions,
    }
}

fn build_methodology(url: &str) -> MethodologySection {
    MethodologySection {
        scope: format!(
            "Automatisierte Prüfung der Seite {} auf Barrierefreiheit nach WCAG 2.1 (Level AA). \
             Zusätzlich wurden Performance, SEO, Sicherheit und mobile Nutzbarkeit analysiert.",
            url
        ),
        method: "Die Prüfung erfolgte über den Chrome DevTools Protocol (CDP) und den \
                 nativen Accessibility Tree des Browsers. 21 WCAG-Regeln wurden automatisiert \
                 gegen den Seiteninhalt geprüft."
            .to_string(),
        limitations: "Automatisierte Tests können ca. 30–40% aller Barrierefreiheitsprobleme erkennen. \
                      Komplexe Aspekte wie korrekte Tab-Reihenfolge, sinnvolle Alt-Texte oder \
                      verständliche Sprache erfordern zusätzlich manuelle Prüfung."
            .to_string(),
        disclaimer: "Dieser Report stellt eine automatisierte technische Analyse dar. \
                     Er ersetzt keine vollständige Konformitätsbewertung nach WCAG 2.1. \
                     Für eine rechtsverbindliche Aussage zur Barrierefreiheit ist eine \
                     umfassende manuelle Prüfung durch Experten erforderlich."
            .to_string(),
    }
}

fn build_executive_summary(
    report: &AuditReport,
    top_findings: &[FindingGroup],
    positive_aspects: &[PositiveAspect],
) -> ExecutiveSummary {
    let overall = if report.score >= 90.0 {
        "Die Website zeigt insgesamt eine gute Barrierefreiheit mit nur wenigen Auffälligkeiten."
    } else if report.score >= 70.0 {
        "Die Website hat eine solide Basis, weist aber relevante Barrieren auf, die behoben werden sollten."
    } else if report.score >= 50.0 {
        "Die Website weist erhebliche Barrierefreiheitsprobleme auf. Eine zeitnahe Behebung wird empfohlen."
    } else {
        "Die Website hat schwerwiegende Barrierefreiheitsmängel. Dringender Handlungsbedarf besteht."
    };

    let key_risks: Vec<String> = top_findings
        .iter()
        .take(3)
        .map(|f| {
            format!(
                "{} (WCAG {}, {}): {}",
                f.title,
                f.wcag_criterion,
                f.priority.label(),
                f.user_impact
            )
        })
        .collect();

    let positive_highlights: Vec<String> = positive_aspects
        .iter()
        .map(|p| format!("{}: {}", p.area, p.description))
        .collect();

    let priorities: Vec<String> = top_findings
        .iter()
        .take(5)
        .map(|f| format!("{} — {}", f.title, f.effort.label()))
        .collect();

    ExecutiveSummary {
        overall_assessment: overall.to_string(),
        key_risks,
        positive_highlights,
        priorities,
    }
}

fn derive_positive_aspects(report: &AuditReport) -> Vec<PositiveAspect> {
    let mut positives = Vec::new();

    if report.wcag_results.violations.is_empty() {
        positives.push(PositiveAspect {
            area: "Barrierefreiheit".to_string(),
            description: "Keine automatisch erkennbaren Verstöße gefunden.".to_string(),
        });
    } else if report.score >= 80.0 {
        positives.push(PositiveAspect {
            area: "Barrierefreiheit".to_string(),
            description: format!("Guter Score von {:.0}/100 — die Basis stimmt.", report.score),
        });
    }

    if let Some(ref perf) = report.performance {
        if perf.score.overall >= 80 {
            positives.push(PositiveAspect {
                area: "Performance".to_string(),
                description: format!("Gute Ladezeiten mit {}/100 Punkten.", perf.score.overall),
            });
        }
    }

    if let Some(ref seo) = report.seo {
        if seo.score >= 80 {
            positives.push(PositiveAspect {
                area: "SEO".to_string(),
                description: format!("Solide SEO-Basis mit {}/100 Punkten.", seo.score),
            });
        }
    }

    if let Some(ref sec) = report.security {
        if sec.score >= 80 {
            positives.push(PositiveAspect {
                area: "Sicherheit".to_string(),
                description: format!(
                    "Gute Security-Konfiguration mit {}/100 Punkten (Grade {}).",
                    sec.score, sec.grade
                ),
            });
        }
    }

    if let Some(ref mobile) = report.mobile {
        if mobile.score >= 80 {
            positives.push(PositiveAspect {
                area: "Mobile".to_string(),
                description: format!("Gute mobile Nutzbarkeit mit {}/100 Punkten.", mobile.score),
            });
        }
    }

    // If nothing positive found, add a generic one
    if positives.is_empty() {
        positives.push(PositiveAspect {
            area: "Grundstruktur".to_string(),
            description: "Die Seite ist grundsätzlich funktional und erreichbar.".to_string(),
        });
    }

    positives
}

fn derive_action_plan(finding_groups: &[FindingGroup]) -> ActionPlan {
    let mut quick_wins = Vec::new();
    let mut medium_term = Vec::new();
    let mut structural = Vec::new();

    for group in finding_groups {
        let item = ActionItem {
            action: truncate_text(&group.recommendation, 80),
            benefit: truncate_text(&group.user_impact, 50),
            role: group.responsible_role,
            priority: group.priority,
        };

        match group.effort {
            Effort::Quick => quick_wins.push(item),
            Effort::Medium => medium_term.push(item),
            Effort::Structural => structural.push(item),
        }
    }

    // Sort by priority within each category
    quick_wins.sort_by(|a, b| b.priority.cmp(&a.priority));
    medium_term.sort_by(|a, b| b.priority.cmp(&a.priority));
    structural.sort_by(|a, b| b.priority.cmp(&a.priority));

    // Build role assignments
    let mut role_map: HashMap<Role, Vec<String>> = HashMap::new();
    for group in finding_groups {
        role_map
            .entry(group.responsible_role)
            .or_default()
            .push(group.title.clone());
    }

    // Add standard structural recommendations
    role_map
        .entry(Role::ProjectManagement)
        .or_default()
        .extend([
            "Priorisierung der Maßnahmen".to_string(),
            "Qualitätssicherung und Testing".to_string(),
            "Verantwortlichkeiten festlegen".to_string(),
        ]);

    let role_assignments: Vec<RoleAssignment> = role_map
        .into_iter()
        .map(|(role, mut responsibilities)| {
            responsibilities.dedup();
            RoleAssignment {
                role,
                responsibilities,
            }
        })
        .collect();

    ActionPlan {
        quick_wins,
        medium_term,
        structural,
        role_assignments,
    }
}

fn build_score_breakdown(report: &AuditReport) -> ScoreBreakdown {
    let accessibility = ScoreDetail {
        score: report.score.round() as u32,
        label: "Barrierefreiheit (WCAG 2.1)".to_string(),
        interpretation: interpret_score(report.score, "Barrierefreiheit"),
    };

    let performance = report.performance.as_ref().map(|p| ScoreDetail {
        score: p.score.overall,
        label: "Performance".to_string(),
        interpretation: interpret_score(p.score.overall as f32, "Performance"),
    });

    let seo = report.seo.as_ref().map(|s| ScoreDetail {
        score: s.score,
        label: "SEO".to_string(),
        interpretation: interpret_score(s.score as f32, "SEO"),
    });

    let security = report.security.as_ref().map(|s| ScoreDetail {
        score: s.score,
        label: "Sicherheit".to_string(),
        interpretation: interpret_score(s.score as f32, "Sicherheit"),
    });

    let mobile = report.mobile.as_ref().map(|m| ScoreDetail {
        score: m.score,
        label: "Mobile".to_string(),
        interpretation: interpret_score(m.score as f32, "mobile Nutzbarkeit"),
    });

    let overall = if performance.is_some() || seo.is_some() || security.is_some() || mobile.is_some() {
        Some(ScoreDetail {
            score: report.overall_score(),
            label: "Gesamtbewertung".to_string(),
            interpretation: format!(
                "Gewichteter Durchschnitt aller aktiven Module. Accessibility fließt mit 40% ein, \
                 Performance und SEO mit je 20%, Sicherheit und Mobile mit je 10%."
            ),
        })
    } else {
        None
    };

    ScoreBreakdown {
        accessibility,
        performance,
        seo,
        security,
        mobile,
        overall,
    }
}

fn grade_label(score: u32) -> &'static str {
    match score {
        90..=100 => "Sehr gut",
        75..=89 => "Gut",
        60..=74 => "Befriedigend",
        40..=59 => "Ausbaufähig",
        _ => "Kritisch",
    }
}

fn interpret_score(score: f32, area: &str) -> String {
    let grade = grade_label(score.round() as u32);
    match grade {
        "Sehr gut" => format!("{} — die {} ist auf einem hohen Niveau.", grade, area),
        "Gut" => format!(
            "{} — die {} ist solide, einzelne Verbesserungen sind möglich.",
            grade, area
        ),
        "Befriedigend" => format!(
            "{} — die {} weist einzelne Schwächen auf.",
            grade, area
        ),
        "Ausbaufähig" => format!(
            "{} — die {} weist relevante Schwächen auf.",
            grade, area
        ),
        _ => format!(
            "{} — die {} hat erhebliche Mängel, die behoben werden sollten.",
            grade, area
        ),
    }
}

fn build_module_details(report: &AuditReport) -> ModuleDetails {
    let performance = report.performance.as_ref().map(|p| {
        let mut vitals = Vec::new();
        if let Some(ref lcp) = p.vitals.lcp {
            vitals.push(("Largest Contentful Paint (LCP)".to_string(), format!("{:.0}ms", lcp.value), lcp.rating.clone()));
        }
        if let Some(ref fcp) = p.vitals.fcp {
            vitals.push(("First Contentful Paint (FCP)".to_string(), format!("{:.0}ms", fcp.value), fcp.rating.clone()));
        }
        if let Some(ref cls) = p.vitals.cls {
            vitals.push(("Cumulative Layout Shift (CLS)".to_string(), format!("{:.3}", cls.value), cls.rating.clone()));
        }
        if let Some(ref ttfb) = p.vitals.ttfb {
            vitals.push(("Time to First Byte (TTFB)".to_string(), format!("{:.0}ms", ttfb.value), ttfb.rating.clone()));
        }
        if let Some(ref inp) = p.vitals.inp {
            vitals.push(("Interaction to Next Paint (INP)".to_string(), format!("{:.0}ms", inp.value), inp.rating.clone()));
        }
        if let Some(ref tbt) = p.vitals.tbt {
            vitals.push(("Total Blocking Time (TBT)".to_string(), format!("{:.0}ms", tbt.value), tbt.rating.clone()));
        }

        let mut additional = Vec::new();
        if let Some(nodes) = p.vitals.dom_nodes {
            additional.push(("DOM-Knoten".to_string(), nodes.to_string()));
        }
        if let Some(heap) = p.vitals.js_heap_size {
            additional.push(("JS Heap".to_string(), format!("{:.1} MB", heap as f64 / 1_048_576.0)));
        }
        if let Some(load) = p.vitals.load_time {
            additional.push(("Ladezeit".to_string(), format!("{:.0}ms", load)));
        }
        if let Some(dcl) = p.vitals.dom_content_loaded {
            additional.push(("DOM Content Loaded".to_string(), format!("{:.0}ms", dcl)));
        }

        PerformancePresentation {
            score: p.score.overall,
            grade: p.score.grade.label().to_string(),
            interpretation: interpret_score(p.score.overall as f32, "Performance"),
            vitals,
            additional_metrics: additional,
        }
    });

    let seo = report.seo.as_ref().map(|s| {
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

        let meta_issues: Vec<(String, String, String)> = s
            .meta_issues
            .iter()
            .map(|i| (i.field.clone(), i.severity.clone(), i.message.clone()))
            .collect();

        let heading_summary = format!(
            "{} H1-Überschrift(en), {} Überschriften gesamt, {} Probleme",
            s.headings.h1_count,
            s.headings.total_count,
            s.headings.issues.len()
        );

        let social_summary = format!(
            "Open Graph: {}, Twitter Card: {}, Vollständigkeit: {}%",
            if s.social.open_graph.is_some() { "vorhanden" } else { "fehlt" },
            if s.social.twitter_card.is_some() { "vorhanden" } else { "fehlt" },
            s.social.completeness
        );

        let mut technical = Vec::new();
        technical.push(("HTTPS".to_string(), yes_no(s.technical.https)));
        technical.push(("Canonical".to_string(), yes_no(s.technical.has_canonical)));
        technical.push(("Sprachangabe".to_string(), yes_no(s.technical.has_lang)));
        technical.push(("Wortanzahl".to_string(), s.technical.word_count.to_string()));

        SeoPresentation {
            score: s.score,
            interpretation: interpret_score(s.score as f32, "SEO"),
            meta_tags,
            meta_issues,
            heading_summary,
            social_summary,
            technical_summary: technical,
        }
    });

    let security = report.security.as_ref().map(|sec| {
        let header_checks: Vec<(&str, &Option<String>)> = vec![
            ("Content-Security-Policy", &sec.headers.content_security_policy),
            ("Strict-Transport-Security", &sec.headers.strict_transport_security),
            ("X-Content-Type-Options", &sec.headers.x_content_type_options),
            ("X-Frame-Options", &sec.headers.x_frame_options),
            ("X-XSS-Protection", &sec.headers.x_xss_protection),
            ("Referrer-Policy", &sec.headers.referrer_policy),
            ("Permissions-Policy", &sec.headers.permissions_policy),
            ("Cross-Origin-Opener-Policy", &sec.headers.cross_origin_opener_policy),
            ("Cross-Origin-Resource-Policy", &sec.headers.cross_origin_resource_policy),
        ];

        let headers: Vec<(String, String, String)> = header_checks
            .iter()
            .map(|(name, value)| {
                let (status, val) = match value {
                    Some(v) => ("Vorhanden".to_string(), truncate_url(v, 50)),
                    None => ("Fehlt".to_string(), "—".to_string()),
                };
                (name.to_string(), status, val)
            })
            .collect();

        let ssl_info = vec![
            ("HTTPS".to_string(), yes_no(sec.ssl.https)),
            ("Gültiges Zertifikat".to_string(), yes_no(sec.ssl.valid_certificate)),
            ("HSTS".to_string(), yes_no(sec.ssl.has_hsts)),
            ("HSTS Max-Age".to_string(), sec.ssl.hsts_max_age.map(|v| format!("{}s", v)).unwrap_or_else(|| "—".to_string())),
            ("Subdomains".to_string(), yes_no(sec.ssl.hsts_include_subdomains)),
            ("Preload".to_string(), yes_no(sec.ssl.hsts_preload)),
        ];

        let issues: Vec<(String, String, String)> = sec
            .issues
            .iter()
            .map(|i| (i.header.clone(), i.severity.clone(), i.message.clone()))
            .collect();

        SecurityPresentation {
            score: sec.score,
            grade: sec.grade.clone(),
            interpretation: interpret_score(sec.score as f32, "Sicherheit"),
            headers,
            ssl_info,
            issues,
            recommendations: sec.recommendations.clone(),
        }
    });

    let mobile = report.mobile.as_ref().map(|m| {
        let viewport = vec![
            ("Viewport-Tag".to_string(), yes_no(m.viewport.has_viewport)),
            ("device-width".to_string(), yes_no(m.viewport.uses_device_width)),
            ("Initial Scale".to_string(), yes_no(m.viewport.has_initial_scale)),
            ("Skalierbar".to_string(), yes_no(m.viewport.is_scalable)),
            ("Korrekt konfiguriert".to_string(), yes_no(m.viewport.is_properly_configured)),
        ];

        let touch_targets = vec![
            ("Gesamt".to_string(), m.touch_targets.total_targets.to_string()),
            ("Ausreichend (≥44px)".to_string(), m.touch_targets.adequate_targets.to_string()),
            ("Zu klein".to_string(), m.touch_targets.small_targets.to_string()),
            ("Zu eng beieinander".to_string(), m.touch_targets.crowded_targets.to_string()),
        ];

        let font_analysis = vec![
            ("Basis-Schriftgröße".to_string(), format!("{:.0}px", m.font_sizes.base_font_size)),
            ("Kleinste Schrift".to_string(), format!("{:.0}px", m.font_sizes.smallest_font_size)),
            ("Lesbarer Text".to_string(), format!("{:.0}%", m.font_sizes.legible_percentage)),
            ("Relative Einheiten".to_string(), yes_no(m.font_sizes.uses_relative_units)),
        ];

        let content_sizing = vec![
            ("Passt in Viewport".to_string(), yes_no(m.content_sizing.fits_viewport)),
            ("Kein hor. Scrollen".to_string(), yes_no(!m.content_sizing.has_horizontal_scroll)),
            ("Responsive Bilder".to_string(), yes_no(m.content_sizing.uses_responsive_images)),
            ("Media Queries".to_string(), yes_no(m.content_sizing.uses_media_queries)),
        ];

        let issues: Vec<(String, String, String)> = m
            .issues
            .iter()
            .map(|i| (i.category.clone(), i.severity.clone(), i.message.clone()))
            .collect();

        MobilePresentation {
            score: m.score,
            interpretation: interpret_score(m.score as f32, "mobile Nutzbarkeit"),
            viewport,
            touch_targets,
            font_analysis,
            content_sizing,
            issues,
        }
    });

    ModuleDetails {
        performance,
        seo,
        security,
        mobile,
    }
}

fn build_appendix(report: &AuditReport) -> AppendixData {
    let violations: Vec<AppendixViolation> = report
        .wcag_results
        .violations
        .iter()
        .map(|v| AppendixViolation {
            rule: v.rule.clone(),
            rule_name: v.rule_name.clone(),
            severity: capitalize_severity(&v.severity),
            message: v.message.clone(),
            node_id: v.node_id.clone(),
            selector: v.selector.clone(),
            fix_suggestion: v.fix_suggestion.clone(),
        })
        .collect();

    AppendixData {
        violations,
        score_methodology: "Score-Berechnung: Basis 100 Punkte. Abzug von 2,5 Punkten pro \
            kritischem/schwerem Verstoß und 1 Punkt pro mäßigem Verstoß. \
            Zusätzliche Abzüge für besonders impactstarke Regeln (z. B. fehlende Sprache: -10, \
            fehlende Überschriften: -20, fehlende Alt-Texte: -3, fehlende Labels: -5, \
            Kontrastprobleme: -5)."
            .to_string(),
    }
}

fn build_batch_appendix(batch: &BatchReport) -> BatchAppendixData {
    let per_url: Vec<UrlAppendix> = batch
        .reports
        .iter()
        .map(|r| UrlAppendix {
            url: r.url.clone(),
            violations: r
                .wcag_results
                .violations
                .iter()
                .map(|v| AppendixViolation {
                    rule: v.rule.clone(),
                    rule_name: v.rule_name.clone(),
                    severity: capitalize_severity(&v.severity),
                    message: v.message.clone(),
                    node_id: v.node_id.clone(),
                    selector: v.selector.clone(),
                    fix_suggestion: v.fix_suggestion.clone(),
                })
                .collect(),
        })
        .collect();

    BatchAppendixData { per_url }
}

fn build_batch_verdict(batch: &BatchReport) -> String {
    let avg = batch.summary.average_score;
    if avg >= 90.0 {
        format!(
            "Über {} geprüfte URLs hinweg erreicht die Website einen durchschnittlichen \
             Accessibility-Score von {:.0}/100 — ein sehr gutes Ergebnis.",
            batch.summary.total_urls, avg
        )
    } else if avg >= 70.0 {
        format!(
            "Im Durchschnitt erreichen die {} geprüften URLs {:.0}/100 Punkte. \
             Die Basis ist solide, es bestehen aber wiederkehrende Barrieren.",
            batch.summary.total_urls, avg
        )
    } else if avg >= 50.0 {
        format!(
            "Die {} geprüften URLs erreichen im Schnitt nur {:.0}/100 Punkte. \
             Es bestehen erhebliche systematische Barrierefreiheitsprobleme.",
            batch.summary.total_urls, avg
        )
    } else {
        format!(
            "Die {} geprüften URLs erreichen im Schnitt nur {:.0}/100 Punkte. \
             Die Barrierefreiheit ist stark eingeschränkt — dringender Handlungsbedarf.",
            batch.summary.total_urls, avg
        )
    }
}

fn yes_no(val: bool) -> String {
    if val { "Ja".to_string() } else { "Nein".to_string() }
}

// ─── Clone implementations for types that need it ───────────────────────────

impl Clone for FindingGroup {
    fn clone(&self) -> Self {
        FindingGroup {
            title: self.title.clone(),
            wcag_criterion: self.wcag_criterion.clone(),
            wcag_level: self.wcag_level.clone(),
            severity: self.severity,
            priority: self.priority,
            customer_description: self.customer_description.clone(),
            user_impact: self.user_impact.clone(),
            typical_cause: self.typical_cause.clone(),
            recommendation: self.recommendation.clone(),
            technical_note: self.technical_note.clone(),
            occurrence_count: self.occurrence_count,
            affected_urls: self.affected_urls.clone(),
            affected_elements: self.affected_elements,
            responsible_role: self.responsible_role,
            effort: self.effort,
            examples: self.examples.clone(),
        }
    }
}

impl Clone for ExampleBlock {
    fn clone(&self) -> Self {
        ExampleBlock {
            bad: self.bad.clone(),
            good: self.good.clone(),
            decorative: self.decorative.clone(),
        }
    }
}
