//! Single-report ViewModel builder.

use crate::audit::normalized::NormalizedReport;
use crate::audit::summary::analyze as analyze_report;
use crate::cli::ReportLevel;
use crate::output::explanations::get_explanation;
use crate::output::report_model::*;
use crate::util::truncate_url;
use crate::wcag::Severity;

use super::actions::{
    derive_action_plan, derive_business_impact, derive_conversion_effect_from_action,
    derive_execution_priority, derive_user_effect_from_action, humanize_action_text, impact_score,
    severity_to_priority,
};
use super::helpers::{
    build_benchmark_context, build_business_consequence, build_consequence_text,
    build_overall_impact, build_score_note, build_technical_overview, build_trend_label,
    build_verdict_text, extract_domain, interpret_score, localized_report_subtitle,
    localized_report_title, truncate_list, truncate_url_list, yes_no,
};
use super::modules::{
    build_tracking_summary_text, derive_accessibility_card_context, derive_accessibility_context,
    derive_accessibility_lever, derive_mobile_card_context, derive_mobile_context,
    derive_mobile_lever, derive_performance_card_context, derive_performance_context,
    derive_performance_lever, derive_performance_recommendations, derive_security_card_context,
    derive_security_context, derive_security_lever, derive_security_recommendations,
    derive_seo_card_context, derive_seo_context, derive_seo_lever,
};
use super::seo::{
    build_seo_interpretation, page_profile_optimization_note, summarize_page_profile,
};

const NBSP: &str = "\u{00A0}";

/// Build a complete ViewModel from a normalized report (single source of truth for score/grade/certificate)
pub fn build_view_model(normalized: &NormalizedReport, config: &ReportConfig) -> ReportViewModel {
    let priority_by_rule: std::collections::HashMap<&str, f32> = normalized
        .findings
        .iter()
        .map(|f| (f.rule_id.as_str(), f.priority_score))
        .collect();

    let mut sorted_groups: Vec<FindingGroup> = normalized
        .findings
        .iter()
        .filter(|f| match config.level {
            ReportLevel::Executive => f.report_visibility.executive,
            ReportLevel::Standard => f.report_visibility.standard,
            ReportLevel::Technical => f.report_visibility.technical,
        })
        .map(finding_group_from_normalized)
        .collect();
    sorted_groups.sort_by(|a, b| {
        let pa = priority_by_rule
            .get(a.rule_id.as_str())
            .copied()
            .unwrap_or(0.0);
        let pb = priority_by_rule
            .get(b.rule_id.as_str())
            .copied()
            .unwrap_or(0.0);
        pb.partial_cmp(&pa)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| impact_score(b).cmp(&impact_score(a)))
    });

    let score = normalized.score;
    let grade = normalized.grade.clone();
    let certificate = normalized.certificate.clone();
    let audit_summary = analyze_report(normalized);
    let maturity_label = audit_summary.site_state.label().to_string();
    let problem_type = audit_summary.problem_type_label.clone();
    let mut technical_overview = build_technical_overview(normalized);
    for cross in &audit_summary.cross_impacts {
        technical_overview.push(format!(
            "Cross-Impact {}: {}",
            cross.dimensions, cross.description
        ));
    }
    let overall_impact = build_overall_impact(normalized);
    let date = normalized.timestamp.format("%d.%m.%Y").to_string();
    let report_title = localized_report_title(&config.locale);
    let report_subtitle = localized_report_subtitle(&config.locale);
    let report_author = extract_domain(&normalized.url);
    let has_quality_modules = normalized.module_scores.len() > 1;

    let top_findings: Vec<FindingGroup> = {
        let mut urgent: Vec<FindingGroup> = sorted_groups
            .iter()
            .filter(|f| matches!(f.severity, Severity::Critical | Severity::High))
            .take(5)
            .cloned()
            .collect();
        if urgent.len() < 5 {
            let urgent_ids: std::collections::HashSet<String> =
                urgent.iter().map(|f| f.rule_id.clone()).collect();
            let remaining: Vec<FindingGroup> = sorted_groups
                .iter()
                .filter(|f| !urgent_ids.contains(&f.rule_id))
                .take(5 - urgent.len())
                .cloned()
                .collect();
            urgent.extend(remaining);
        }
        urgent
    };
    let positive_aspects = derive_positive_aspects_from_normalized(normalized);
    let action_plan = derive_action_plan(&sorted_groups);

    let mut module_names: Vec<String> = vec!["Accessibility".into()];
    if normalized.raw_performance.is_some() {
        module_names.push("Performance".into());
    }
    if normalized.raw_seo.is_some() {
        module_names.push("SEO".into());
    }
    if normalized.raw_security.is_some() {
        module_names.push("Sicherheit".into());
    }
    if normalized.raw_mobile.is_some() {
        module_names.push("Mobile".into());
    }
    if normalized.raw_ux.is_some() {
        module_names.push("UX".into());
    }
    if normalized.raw_journey.is_some() {
        module_names.push("Journey".into());
    }

    let severity = SeverityBlock {
        critical: normalized.severity_counts.critical as u32,
        high: normalized.severity_counts.high as u32,
        medium: normalized.severity_counts.medium as u32,
        low: normalized.severity_counts.low as u32,
        total: normalized.severity_counts.total as u32,
        has_issues: normalized.severity_counts.total > 0,
    };

    let modules = build_modules_block_from_normalized(normalized);

    let quick_win_count = action_plan.quick_wins.len();
    let critical_count =
        (normalized.severity_counts.critical + normalized.severity_counts.high) as u32;
    let total_violations = normalized.severity_counts.total as u32;
    let nodes_analyzed = normalized.nodes_analyzed;

    let actions = build_actions_block(&action_plan, score as f32, &audit_summary.site_state);

    let module_details = build_module_details_from_normalized(normalized);
    let history = config
        .history_preview
        .as_ref()
        .map(build_history_trend_block);

    ReportViewModel {
        meta: MetaBlock {
            title: report_title.clone(),
            subtitle: normalized.url.clone(),
            date: date.clone(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            author: report_author.clone(),
            report_level: config.level,
            score_label: format!("{}/100", score),
        },
        cover: CoverBlock {
            brand: report_author,
            title: report_title,
            domain: normalized.url.clone(),
            subtitle: report_subtitle.to_string(),
            date: date.clone(),
            score,
            grade: grade.clone(),
            certificate: certificate.clone(),
            maturity_label: maturity_label.clone(),
            total_issues: total_violations,
            critical_issues: critical_count,
            modules: module_names,
        },
        summary: SummaryBlock {
            score,
            grade: grade.clone(),
            certificate: certificate.clone(),
            maturity_label: maturity_label.clone(),
            problem_type: problem_type.clone(),
            domain: normalized.url.clone(),
            date: date.clone(),
            executive_lead: audit_summary.verdict_intro.clone(),
            dominant_issue_note: audit_summary.dominant_issue_note.clone(),
            verdict: build_verdict_text(&normalized.url, score as f32),
            score_note: build_score_note(normalized),
            metrics: vec![
                MetricItem {
                    title: format!("Verstöße{NBSP}gesamt"),
                    value: total_violations.to_string(),
                    accent_color: Some("#f59e0b".into()),
                },
                MetricItem {
                    title: "Kritisch".into(),
                    value: critical_count.to_string(),
                    accent_color: Some("#ef4444".into()),
                },
                MetricItem {
                    title: if has_quality_modules {
                        format!("Gesamtscore{NBSP}Website")
                    } else {
                        format!("Geprüfte{NBSP}Knoten")
                    },
                    value: if has_quality_modules {
                        format!("{}/100", normalized.overall_score)
                    } else {
                        nodes_analyzed.to_string()
                    },
                    accent_color: Some("#22c55e".into()),
                },
                MetricItem {
                    title: if has_quality_modules {
                        format!("Geprüfte{NBSP}Knoten")
                    } else {
                        "Quick Wins".into()
                    },
                    value: if has_quality_modules {
                        nodes_analyzed.to_string()
                    } else {
                        quick_win_count.to_string()
                    },
                    accent_color: Some("#2563eb".into()),
                },
                MetricItem {
                    title: if has_quality_modules {
                        "Quick Wins".into()
                    } else {
                        "WCAG-Level".into()
                    },
                    value: if has_quality_modules {
                        quick_win_count.to_string()
                    } else {
                        normalized.wcag_level.to_string()
                    },
                    accent_color: Some("#7c3aed".into()),
                },
            ],
            top_actions: top_findings
                .iter()
                .take(3)
                .map(|f| humanize_action_text(&f.recommendation))
                .collect(),
            positive_aspects: positive_aspects
                .iter()
                .map(|a| format!("{}: {}", a.area, a.description))
                .collect(),
            overall_impact,
            technical_overview,
            benchmark_context: build_benchmark_context(score as f32),
            business_consequence: build_business_consequence(normalized),
            consequence: build_consequence_text(normalized),
            risk_level: normalized.risk.level.to_string(),
            risk_summary: normalized.risk.summary.clone(),
        },
        history,
        methodology: build_methodology(normalized),
        modules,
        severity,
        findings: FindingsBlock {
            top_findings,
            all_findings: sorted_groups,
        },
        module_details,
        actions,
        appendix: build_appendix_block_from_normalized(normalized),
    }
}

fn build_history_trend_block(preview: &ReportHistoryPreview) -> HistoryTrendBlock {
    let trend_label = build_trend_label(preview.delta_accessibility, preview.delta_total_issues);

    let trend_interpretation = match trend_label.as_str() {
        "Deutlich verbessert" => format!(
            "Die Barrierefreiheit hat sich gegenüber dem letzten Lauf vom {} deutlich verbessert (+{} Punkte, {} Issues weniger).",
            preview.previous_date,
            preview.delta_accessibility,
            -preview.delta_total_issues
        ),
        "Verbessert" => format!(
            "Die Barrierefreiheit hat sich gegenüber dem letzten Lauf vom {} verbessert.",
            preview.previous_date
        ),
        "Stabil" => format!(
            "Die Barrierefreiheit ist gegenüber dem letzten Lauf vom {} unverändert stabil.",
            preview.previous_date
        ),
        "Deutlich verschlechtert" => format!(
            "Die Barrierefreiheit ist gegenüber dem letzten Lauf vom {} deutlich zurückgegangen ({} Punkte, +{} Issues). Handlungsbedarf.",
            preview.previous_date,
            preview.delta_accessibility,
            preview.delta_total_issues
        ),
        _ => format!(
            "Die Barrierefreiheit ist gegenüber dem letzten Lauf vom {} leicht zurückgegangen.",
            preview.previous_date
        ),
    };

    HistoryTrendBlock {
        previous_date: preview.previous_date.clone(),
        timeline_entries: preview.timeline_entries,
        trend_label,
        summary: format!(
            "{} Die Historie umfasst {} verwertbare Snapshots.",
            trend_interpretation, preview.timeline_entries
        ),
        metrics: vec![
            (
                "Accessibility-Delta".to_string(),
                format!("{:+}", preview.delta_accessibility),
            ),
            (
                "Gesamt-Delta".to_string(),
                format!("{:+}", preview.delta_overall),
            ),
            (
                "Issue-Delta".to_string(),
                format!("{:+}", preview.delta_total_issues),
            ),
            (
                "Kritisch+Hoch-Delta".to_string(),
                format!("{:+}", preview.delta_critical_issues),
            ),
            (
                "Vorher Accessibility".to_string(),
                preview.previous_accessibility_score.to_string(),
            ),
            (
                "Vorher Gesamt".to_string(),
                preview.previous_overall_score.to_string(),
            ),
        ],
        timeline_rows: preview
            .recent_entries
            .iter()
            .map(|entry| {
                (
                    entry.0.clone(),
                    entry.1.to_string(),
                    entry.2.to_string(),
                    entry.3.clone(),
                    entry.4.to_string(),
                )
            })
            .collect(),
        new_findings: preview.new_findings.clone(),
        resolved_findings: preview.resolved_findings.clone(),
    }
}

fn build_modules_block_from_normalized(normalized: &NormalizedReport) -> ModulesBlock {
    let a11y_score = normalized.score as f32;
    let mut dashboard = vec![ModuleScore {
        name: "Barrierefreiheit".into(),
        score: a11y_score.round() as u32,
        interpretation: interpret_score(a11y_score, "Barrierefreiheit"),
        card_context: derive_accessibility_card_context(normalized),
        score_context: derive_accessibility_context(normalized),
        key_lever: derive_accessibility_lever(normalized),
        good_threshold: 75,
        warn_threshold: 50,
    }];

    if let Some(ref p) = normalized.raw_performance {
        dashboard.push(ModuleScore {
            name: "Performance".into(),
            score: p.score.overall,
            interpretation: interpret_score(p.score.overall as f32, "Performance"),
            card_context: derive_performance_card_context(p),
            score_context: derive_performance_context(p),
            key_lever: derive_performance_lever(p),
            good_threshold: 75,
            warn_threshold: 50,
        });
    }
    if let Some(ref s) = normalized.raw_seo {
        dashboard.push(ModuleScore {
            name: "SEO".into(),
            score: s.score,
            interpretation: build_seo_interpretation(s),
            card_context: derive_seo_card_context(s),
            score_context: derive_seo_context(s),
            key_lever: derive_seo_lever(s),
            good_threshold: 75,
            warn_threshold: 50,
        });
    }
    if let Some(ref s) = normalized.raw_security {
        dashboard.push(ModuleScore {
            name: "Sicherheit".into(),
            score: s.score,
            interpretation: interpret_score(s.score as f32, "Sicherheit"),
            card_context: derive_security_card_context(s),
            score_context: derive_security_context(s),
            key_lever: derive_security_lever(s),
            good_threshold: 75,
            warn_threshold: 50,
        });
    }
    if let Some(ref m) = normalized.raw_mobile {
        dashboard.push(ModuleScore {
            name: "Mobile".into(),
            score: m.score,
            interpretation: interpret_score(m.score as f32, "mobile Nutzbarkeit"),
            card_context: derive_mobile_card_context(m),
            score_context: derive_mobile_context(m),
            key_lever: derive_mobile_lever(m),
            good_threshold: 75,
            warn_threshold: 50,
        });
    }
    if let Some(ref u) = normalized.raw_ux {
        let ux_context = format!(
            "CTA Clarity {}/100, Visual Hierarchy {}/100, Content Clarity {}/100, Trust Signals {}/100, Cognitive Load {}/100",
            u.cta_clarity.score, u.visual_hierarchy.score, u.content_clarity.score, u.trust_signals.score, u.cognitive_load.score
        );
        let ux_lever = if u.cta_clarity.score < 60 {
            "CTA-Texte klarer und spezifischer formulieren".into()
        } else if u.trust_signals.score < 60 {
            "Vertrauenssignale (Kontakt, Impressum) ergänzen".into()
        } else if u.visual_hierarchy.score < 60 {
            "Heading-Struktur bereinigen (H1 → H2 → H3)".into()
        } else {
            "UX-Qualität auf gutem Niveau halten".into()
        };
        dashboard.push(ModuleScore {
            name: "UX".into(),
            score: u.score,
            interpretation: interpret_score(u.score as f32, "User Experience"),
            card_context: ux_context.clone(),
            score_context: ux_context,
            key_lever: ux_lever,
            good_threshold: 80,
            warn_threshold: 55,
        });
    }

    let has_multiple = dashboard.len() > 1;
    let overall_score = if has_multiple {
        Some(normalized.overall_score)
    } else {
        None
    };
    let overall_interpretation = overall_score.map(|_| {
        "Gewichteter Durchschnitt aller aktiven Module. Accessibility 40%, Performance 20%, \
         SEO 20%, UX 15%, Sicherheit 10%, Mobile 10%."
            .to_string()
    });

    ModulesBlock {
        dashboard,
        overall_score,
        overall_interpretation,
    }
}

fn build_actions_block(
    plan: &ActionPlan,
    score: f32,
    site_state: &crate::audit::summary::SiteState,
) -> ActionsBlock {
    use crate::audit::summary::SiteState;
    let is_good_site = score >= 85.0
        || (plan.quick_wins.is_empty() && plan.medium_term.len() + plan.structural.len() <= 3);
    let item_cap: usize = if is_good_site { 2 } else { usize::MAX };

    let (phase1_label, phase1_desc) = match site_state {
        SiteState::Critical => (
            "Blocker — Sofort beheben",
            "Akute Barrieren — direkt beheben, keine weiteren Schritte vorher",
        ),
        SiteState::Weak => (
            "Phase 1 — Hohe Priorität",
            "Relevante Barrieren mit direktem Impact auf Nutzbarkeit",
        ),
        SiteState::NeedsWork => (
            "Phase 1 — Als Erstes",
            "Klarer Verbesserungshebel mit überschaubarem Aufwand",
        ),
        SiteState::Polished => (
            "Phase 1 — Optimieren",
            "Letzte Feinschliff-Maßnahmen ohne strukturellen Druck",
        ),
    };
    let (phase2_label, phase2_desc) = match site_state {
        SiteState::Critical | SiteState::Weak => (
            "Phase 2 — Struktur stabilisieren",
            "Semantik, Navigation und ARIA-Strukturprobleme",
        ),
        _ => (
            "Phase 2 — Struktur verbessern",
            "Semantik, Navigation und Barrierefreiheits-Struktur",
        ),
    };
    let phase3_label = "Phase 3 — Langfristig";
    let phase3_desc = "Langfristige Qualität, SEO und Performance";

    let map_items = |items: &[ActionItem]| -> Vec<RoadmapItemData> {
        items
            .iter()
            .map(|i| {
                let user_effect = derive_user_effect_from_action(&i.action, i.effort);
                let risk_effect = match i.priority {
                    Priority::Critical => {
                        "Reduziert kritisches WCAG-Verstoßrisiko direkt".to_string()
                    }
                    Priority::High => "Reduziert hohes Compliance-Risiko".to_string(),
                    Priority::Medium => "Verringert mittleres Barrierefreiheitsrisiko".to_string(),
                    Priority::Low => "Verbessert WCAG-Konformität im Detail".to_string(),
                };
                let conversion_effect = derive_conversion_effect_from_action(&i.action, i.effort);
                RoadmapItemData {
                    action: i.action.clone(),
                    role: i.role.label().to_string(),
                    priority: i.priority.label().to_string(),
                    execution_priority: i.execution_priority.label().to_string(),
                    effort: i.effort.label().to_string(),
                    benefit: i.benefit.clone(),
                    user_effect,
                    risk_effect,
                    conversion_effect,
                }
            })
            .collect()
    };

    let mut phase_preview = Vec::new();
    if !plan.quick_wins.is_empty() {
        phase_preview.push(PhasePreview {
            phase_label: phase1_label.into(),
            accent_color: "#dc2626".into(),
            description: phase1_desc.into(),
            item_count: plan.quick_wins.len(),
            top_items: plan.quick_wins.iter().map(|i| i.action.clone()).collect(),
        });
    }
    if !plan.medium_term.is_empty() {
        phase_preview.push(PhasePreview {
            phase_label: phase2_label.into(),
            accent_color: "#f59e0b".into(),
            description: phase2_desc.into(),
            item_count: plan.medium_term.len(),
            top_items: plan.medium_term.iter().map(|i| i.action.clone()).collect(),
        });
    }
    if !plan.structural.is_empty() {
        phase_preview.push(PhasePreview {
            phase_label: phase3_label.into(),
            accent_color: "#2563eb".into(),
            description: phase3_desc.into(),
            item_count: plan.structural.len(),
            top_items: plan.structural.iter().map(|i| i.action.clone()).collect(),
        });
    }

    let capped = |items: &[ActionItem]| -> Vec<ActionItem> {
        items.iter().take(item_cap).cloned().collect()
    };

    let mut columns = Vec::new();
    if !plan.quick_wins.is_empty() {
        columns.push(RoadmapColumnData {
            title: phase1_label.into(),
            accent_color: "#dc2626".into(),
            items: map_items(&capped(&plan.quick_wins)),
        });
    }
    if !plan.medium_term.is_empty() {
        columns.push(RoadmapColumnData {
            title: phase2_label.into(),
            accent_color: "#f59e0b".into(),
            items: map_items(&capped(&plan.medium_term)),
        });
    }
    if !plan.structural.is_empty() {
        columns.push(RoadmapColumnData {
            title: phase3_label.into(),
            accent_color: "#2563eb".into(),
            items: map_items(&capped(&plan.structural)),
        });
    }

    let block_title = if is_good_site {
        "Letzte Optimierungsschritte".to_string()
    } else {
        "Maßnahmenplan nach Phasen".to_string()
    };

    let intro_text = if is_good_site {
        "Die Seite ist technisch stark aufgestellt. Die folgenden Punkte sind letzte Optimierungshebel ohne strukturellen Druck.".to_string()
    } else {
        "Phase 1 enthält blockierende Issues mit sofortiger Wirkung. Phase 2 verbessert die Struktur. Phase 3 optimiert langfristig. Jede Phase baut auf der vorherigen auf.".to_string()
    };

    ActionsBlock {
        roadmap_columns: columns,
        role_assignments: plan.role_assignments.clone(),
        intro_text,
        phase_preview,
        block_title,
    }
}

fn build_appendix_block_from_normalized(normalized: &NormalizedReport) -> AppendixBlock {
    let violations: Vec<AppendixViolation> = normalized
        .findings
        .iter()
        .map(|f| {
            let affected_elements: Vec<AffectedElement> = f
                .occurrences
                .iter()
                .map(|occ| AffectedElement {
                    selector: occ.selector.clone().unwrap_or_else(|| occ.node_id.clone()),
                    node_id: occ.node_id.clone(),
                })
                .collect();

            AppendixViolation {
                rule: f.wcag_criterion.clone(),
                rule_name: f.title.clone(),
                severity: f.severity,
                message: f.description.clone(),
                fix_suggestion: f.occurrences.first().and_then(|o| o.fix_suggestion.clone()),
                affected_elements,
            }
        })
        .collect();

    let has_violations = !violations.is_empty();

    AppendixBlock {
        violations,
        score_methodology: "Score-Berechnung: Basis 100 Punkte. Abzug auf Basis der Taxonomie-Regel-Definitionen \
            mit regelspezifischen Penalties und logarithmischer Skalierung für wiederholte Verstöße. \
            Korrektur bei supprimierten Regeln (z.B. 3.1.1 bei vorhandener Sprachangabe).".to_string(),
        has_violations,
    }
}

fn build_methodology(normalized: &NormalizedReport) -> MethodologyBlock {
    let active_modules = normalized
        .module_scores
        .iter()
        .map(|m| m.name.clone())
        .collect::<Vec<_>>()
        .join(", ");

    MethodologyBlock {
        scope: format!(
            "Automatisierte Prüfung der Seite {} auf Barrierefreiheit nach WCAG 2.1 (Level {}). \
             Zusätzlich wurden Performance, SEO, Sicherheit und mobile Nutzbarkeit analysiert.",
            normalized.url, normalized.wcag_level
        ),
        method: "Die Prüfung erfolgte über den Chrome DevTools Protocol (CDP) und den \
                 nativen Accessibility Tree des Browsers. 21 WCAG-Regeln wurden automatisiert \
                 gegen den Seiteninhalt geprüft."
            .to_string(),
        limitations:
            "Automatisierte Tests können ca. 30–40% aller Barrierefreiheitsprobleme erkennen. \
                      Komplexe Aspekte wie korrekte Tab-Reihenfolge, sinnvolle Alt-Texte oder \
                      verständliche Sprache erfordern zusätzlich manuelle Prüfung."
                .to_string(),
        disclaimer: "Dieser Report stellt eine automatisierte technische Analyse dar. \
                     Er ersetzt keine vollständige Konformitätsbewertung nach WCAG 2.1. \
                     Für eine rechtsverbindliche Aussage zur Barrierefreiheit ist eine \
                     umfassende manuelle Prüfung durch Experten erforderlich."
            .to_string(),
        audit_facts: vec![
            (
                "Primärscore".to_string(),
                format!("Accessibility {} / 100", normalized.score),
            ),
            (
                "Gesamtscore".to_string(),
                format!(
                    "{} / 100 (gewichteter Mix aktiver Module)",
                    normalized.overall_score
                ),
            ),
            ("WCAG-Level".to_string(), normalized.wcag_level.to_string()),
            (
                "Geprüfte Knoten".to_string(),
                normalized.nodes_analyzed.to_string(),
            ),
            (
                "Laufzeit".to_string(),
                format!("{:.1} s", normalized.duration_ms as f64 / 1000.0),
            ),
            ("Aktive Module".to_string(), active_modules),
        ],
    }
}

fn build_module_details_from_normalized(normalized: &NormalizedReport) -> ModuleDetailsBlock {
    let performance = normalized.raw_performance.as_ref().map(|p| {
        let mut vitals = Vec::new();
        if let Some(ref lcp) = p.vitals.lcp {
            vitals.push((
                "Largest Contentful Paint (LCP)".to_string(),
                format!("{:.0}ms", lcp.value),
                lcp.rating.clone(),
            ));
        }
        if let Some(ref fcp) = p.vitals.fcp {
            vitals.push((
                "First Contentful Paint (FCP)".to_string(),
                format!("{:.0}ms", fcp.value),
                fcp.rating.clone(),
            ));
        }
        if let Some(ref cls) = p.vitals.cls {
            vitals.push((
                "Cumulative Layout Shift (CLS)".to_string(),
                format!("{:.3}", cls.value),
                cls.rating.clone(),
            ));
        }
        if let Some(ref ttfb) = p.vitals.ttfb {
            vitals.push((
                "Time to First Byte (TTFB)".to_string(),
                format!("{:.0}ms", ttfb.value),
                ttfb.rating.clone(),
            ));
        }
        if let Some(ref inp) = p.vitals.inp {
            vitals.push((
                "Interaction to Next Paint (INP)".to_string(),
                format!("{:.0}ms", inp.value),
                inp.rating.clone(),
            ));
        }
        if let Some(ref tbt) = p.vitals.tbt {
            vitals.push((
                "Total Blocking Time (TBT)".to_string(),
                format!("{:.0}ms", tbt.value),
                tbt.rating.clone(),
            ));
        }

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

        let recommendations = derive_performance_recommendations(p);

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

        PerformancePresentation {
            score: p.score.overall,
            grade: p.score.grade.label().to_string(),
            interpretation: interpret_score(p.score.overall as f32, "Performance"),
            vitals,
            additional_metrics: additional,
            recommendations,
            render_blocking_metrics,
            render_blocking_suggestions,
            has_render_blocking,
        }
    });

    let seo = normalized.raw_seo.as_ref().map(|s| {
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

        let meta_issues: Vec<(String, Severity, String)> = s
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
                page_type: cp.page_classification.primary_type.label().to_string(),
                page_attributes: cp.page_classification.attributes.clone(),
                content_depth_score: cp.page_classification.content_depth_score,
                structural_richness_score: cp.page_classification.structural_richness_score,
                media_text_balance_score: cp.page_classification.media_text_balance_score,
                intent_fit_score: cp.page_classification.intent_fit_score,
                page_profile_summary: summarize_page_profile(cp),
                optimization_note: page_profile_optimization_note(cp),
                page_profile_facts: vec![
                    (
                        "Seitentyp".to_string(),
                        cp.page_classification.primary_type.label().to_string(),
                    ),
                    (
                        "Merkmale".to_string(),
                        if cp.page_classification.attributes.is_empty() {
                            "Keine prägenden Merkmale erkannt".to_string()
                        } else {
                            format!("{}.", cp.page_classification.attributes.join(", "))
                        },
                    ),
                    ("Einordnung".to_string(), summarize_page_profile(cp)),
                    ("Empfehlung".to_string(), page_profile_optimization_note(cp)),
                ],
                schema_rows,
                schema_count: cp.schema_inventory.total_count,
                signal_rows,
                signal_overall_pct: cp.signal_strength.overall_pct,
                signal_details,
                maturity_level: cp.maturity.label().to_string(),
                maturity_description: cp.maturity.description().to_string(),
                maturity_techniques_used: cp.maturity_techniques,
                maturity_techniques_total: 13,
            }
        });

        SeoPresentation {
            score: s.score,
            interpretation: build_seo_interpretation(s),
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
                ("HTTPS".to_string(), yes_no(s.technical.https)),
                ("Canonical".to_string(), yes_no(s.technical.has_canonical)),
                ("Sprachangabe".to_string(), yes_no(s.technical.has_lang)),
                ("Wortanzahl".to_string(), s.technical.word_count.to_string()),
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
            tracking_summary_text: build_tracking_summary_text(&s.technical),
            profile,
        }
    });

    let security = normalized.raw_security.as_ref().map(|sec| {
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
            ("X-XSS-Protection", &sec.headers.x_xss_protection),
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
            score: sec.score,
            grade: sec.grade.clone(),
            interpretation: interpret_score(sec.score as f32, "Sicherheit"),
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
                ("HTTPS".to_string(), yes_no(sec.ssl.https)),
                (
                    "Gültiges Zertifikat".to_string(),
                    yes_no(sec.ssl.valid_certificate),
                ),
                ("HSTS".to_string(), yes_no(sec.ssl.has_hsts)),
                (
                    "HSTS Max-Age".to_string(),
                    sec.ssl
                        .hsts_max_age
                        .map(|v| format!("{}s", v))
                        .unwrap_or_else(|| "—".to_string()),
                ),
                (
                    "Subdomains".to_string(),
                    yes_no(sec.ssl.hsts_include_subdomains),
                ),
                ("Preload".to_string(), yes_no(sec.ssl.hsts_preload)),
            ],
            issues: sec
                .issues
                .iter()
                .map(|i| (i.header.clone(), i.severity, i.message.clone()))
                .collect(),
            recommendations: derive_security_recommendations(sec),
        }
    });

    let mobile = normalized.raw_mobile.as_ref().map(|m| MobilePresentation {
        score: m.score,
        interpretation: interpret_score(m.score as f32, "mobile Nutzbarkeit"),
        viewport: vec![
            ("Viewport-Tag".to_string(), yes_no(m.viewport.has_viewport)),
            (
                "device-width".to_string(),
                yes_no(m.viewport.uses_device_width),
            ),
            (
                "Initial Scale".to_string(),
                yes_no(m.viewport.has_initial_scale),
            ),
            ("Skalierbar".to_string(), yes_no(m.viewport.is_scalable)),
            (
                "Korrekt konfiguriert".to_string(),
                yes_no(m.viewport.is_properly_configured),
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
                yes_no(m.font_sizes.uses_relative_units),
            ),
        ],
        content_sizing: vec![
            (
                "Passt in Viewport".to_string(),
                yes_no(m.content_sizing.fits_viewport),
            ),
            (
                "Kein hor. Scrollen".to_string(),
                yes_no(!m.content_sizing.has_horizontal_scroll),
            ),
            (
                "Responsive Bilder".to_string(),
                yes_no(m.content_sizing.uses_responsive_images),
            ),
            (
                "Media Queries".to_string(),
                yes_no(m.content_sizing.uses_media_queries),
            ),
        ],
        issues: m
            .issues
            .iter()
            .map(|i| (i.category.clone(), i.severity, i.message.clone()))
            .collect(),
    });

    let dark_mode = normalized
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
        });

    let ux = normalized.raw_ux.as_ref().map(|u| UxPresentation {
        score: u.score,
        grade: u.grade.clone(),
        interpretation: interpret_score(u.score as f32, "User Experience"),
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
    });

    let journey = normalized
        .raw_journey
        .as_ref()
        .map(|j| JourneyPresentation {
            score: j.score,
            grade: j.grade.clone(),
            page_intent: j.page_intent.label().to_string(),
            interpretation: interpret_score(j.score as f32, "User Journey"),
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
        });

    let has_any = performance.is_some()
        || seo.is_some()
        || security.is_some()
        || mobile.is_some()
        || ux.is_some()
        || journey.is_some()
        || dark_mode.is_some();
    let source_quality = normalized.raw_source_quality.clone();

    let has_any = has_any || source_quality.is_some();

    ModuleDetailsBlock {
        performance,
        seo,
        security,
        mobile,
        ux,
        journey,
        dark_mode,
        source_quality,
        has_any,
    }
}

fn finding_group_from_normalized(f: &crate::audit::normalized::NormalizedFinding) -> FindingGroup {
    let explanation = get_explanation(&f.wcag_criterion);

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
                f.dimension.as_str(),
                f.severity,
                Some(f.subcategory.as_str()),
            ),
            expl.typical_cause.to_string(),
            expl.recommendation.to_string(),
            expl.technical_note.to_string(),
            expl.responsible_role,
            expl.effort_estimate,
            derive_execution_priority(f.severity, expl.effort_estimate, f.dimension.as_str()),
        )
    } else {
        (
            f.title.clone(),
            f.description.clone(),
            f.user_impact.clone(),
            derive_business_impact(
                &f.user_impact,
                f.dimension.as_str(),
                f.severity,
                Some(f.subcategory.as_str()),
            ),
            "Automatisch erkanntes Problem.".to_string(),
            f.occurrences
                .first()
                .and_then(|o| o.fix_suggestion.clone())
                .unwrap_or_else(|| "Bitte prüfen und beheben.".to_string()),
            String::new(),
            Role::Development,
            Effort::Medium,
            derive_execution_priority(f.severity, Effort::Medium, f.dimension.as_str()),
        )
    };

    let examples = explanation.map(|e| e.examples()).unwrap_or_default();
    let location_hints = build_location_hints(&f.occurrences);

    FindingGroup {
        title,
        rule_id: f.rule_id.clone(),
        wcag_criterion: f.wcag_criterion.clone(),
        wcag_level: f.wcag_level.clone(),
        dimension: Some(f.dimension.clone()),
        subcategory: Some(f.subcategory.clone()),
        issue_class: Some(f.issue_class.clone()),
        severity: f.severity,
        priority: severity_to_priority(f.severity),
        customer_description: customer_desc,
        user_impact: user_impact_text,
        business_impact,
        typical_cause,
        recommendation,
        technical_note,
        occurrence_count: f.occurrence_count,
        affected_urls: Vec::new(),
        affected_elements: f.occurrence_count,
        location_hints,
        responsible_role: role,
        effort,
        execution_priority,
        examples,
    }
}

fn build_location_hints(occurrences: &[crate::audit::normalized::OccurrenceDetail]) -> Vec<String> {
    let mut hints = Vec::new();
    for occ in occurrences {
        let hint = if let Some(selector) = &occ.selector {
            selector.trim().to_string()
        } else {
            format!("AX-Node {}", occ.node_id)
        };
        if !hint.is_empty() && !hints.contains(&hint) {
            hints.push(hint);
        }
        if hints.len() >= 5 {
            break;
        }
    }
    hints
}

fn derive_positive_aspects_from_normalized(normalized: &NormalizedReport) -> Vec<PositiveAspect> {
    let mut positives = Vec::new();
    let a11y_score = normalized.score as f32;

    if normalized.findings.is_empty() {
        positives.push(PositiveAspect {
            area: "Barrierefreiheit".into(),
            description: "Keine automatisch erkennbaren Verstöße gefunden.".into(),
        });
    } else if a11y_score >= 80.0 {
        positives.push(PositiveAspect {
            area: "Barrierefreiheit".into(),
            description: "Solide Grundqualität mit gezielt priorisierbaren Restpunkten.".into(),
        });
    }

    if let Some(ref perf) = normalized.raw_performance {
        if perf.score.overall >= 80 {
            positives.push(PositiveAspect {
                area: "Performance".into(),
                description: "Stabile Ladezeiten und insgesamt reaktionsschneller Seitenaufbau."
                    .into(),
            });
        }
    }
    if let Some(ref seo) = normalized.raw_seo {
        if seo.score >= 80 {
            positives.push(PositiveAspect {
                area: "SEO".into(),
                description: "Saubere Basis für Auffindbarkeit, Struktur und Meta-Daten.".into(),
            });
        }
    }
    if let Some(ref sec) = normalized.raw_security {
        if sec.score >= 80 {
            positives.push(PositiveAspect {
                area: "Sicherheit".into(),
                description: "Wichtige Sicherheitsmechanismen sind grundsätzlich vorhanden.".into(),
            });
        }
    }
    if let Some(ref mobile) = normalized.raw_mobile {
        if mobile.score >= 80 {
            positives.push(PositiveAspect {
                area: "Mobile".into(),
                description: "Die Seite ist auf kleinen Displays gut bedienbar und lesbar.".into(),
            });
        }
    }

    if positives.is_empty() {
        positives.push(PositiveAspect {
            area: "Grundstruktur".into(),
            description: "Die Seite ist grundsätzlich funktional und erreichbar.".into(),
        });
    }
    positives
}

// ─── Clone implementations ──────────────────────────────────────────────────

impl Clone for FindingGroup {
    fn clone(&self) -> Self {
        FindingGroup {
            title: self.title.clone(),
            rule_id: self.rule_id.clone(),
            wcag_criterion: self.wcag_criterion.clone(),
            wcag_level: self.wcag_level.clone(),
            dimension: self.dimension.clone(),
            subcategory: self.subcategory.clone(),
            issue_class: self.issue_class.clone(),
            severity: self.severity,
            priority: self.priority,
            customer_description: self.customer_description.clone(),
            user_impact: self.user_impact.clone(),
            business_impact: self.business_impact.clone(),
            typical_cause: self.typical_cause.clone(),
            recommendation: self.recommendation.clone(),
            technical_note: self.technical_note.clone(),
            occurrence_count: self.occurrence_count,
            affected_urls: self.affected_urls.clone(),
            affected_elements: self.affected_elements,
            location_hints: self.location_hints.clone(),
            responsible_role: self.responsible_role,
            effort: self.effort,
            execution_priority: self.execution_priority,
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

impl Clone for RoleAssignment {
    fn clone(&self) -> Self {
        RoleAssignment {
            role: self.role,
            responsibilities: self.responsibilities.clone(),
        }
    }
}
