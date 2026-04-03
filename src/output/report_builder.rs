//! Report builder — transforms raw audit data into ViewModels
//!
//! This module takes raw AuditReport / BatchReport data and produces
//! structured ViewModels with grouped findings, aggregated statistics,
//! and pre-computed presentation data. The renderer does zero data transformation.

use std::collections::{HashMap, HashSet};

use crate::audit::normalized::NormalizedReport;
use crate::audit::{BatchReport, BrokenLinkSeverity};
use crate::cli::ReportLevel;
use crate::output::explanations::get_explanation;
use crate::output::report_model::*;
use crate::seo::profile::PageType;
use crate::taxonomy::RuleLookup;
use crate::util::truncate_url;
use crate::wcag::Severity;

const NBSP: &str = "\u{00A0}";

// ─── Single Report ViewModel ────────────────────────────────────────────────

/// Build a complete ViewModel from a normalized report (single source of truth for score/grade/certificate)
pub fn build_view_model(normalized: &NormalizedReport, config: &ReportConfig) -> ReportViewModel {
    let priority_by_rule: HashMap<&str, f32> = normalized
        .findings
        .iter()
        .map(|f| (f.rule_id.as_str(), f.priority_score))
        .collect();

    // Convert NormalizedFindings → FindingGroups, filtering by report visibility
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
    let maturity_label = build_maturity_label(
        score,
        normalized.severity_counts.critical,
        normalized.severity_counts.high,
    );
    let problem_type = build_problem_type(normalized);
    let technical_overview = build_technical_overview(normalized);
    let overall_impact = build_overall_impact(normalized);
    let date = normalized.timestamp.format("%d.%m.%Y").to_string();
    let report_title = localized_report_title(&config.locale);
    let report_subtitle = localized_report_subtitle(&config.locale);
    let report_author = extract_domain(&normalized.url);
    let has_quality_modules = normalized.module_scores.len() > 1;

    // Top findings: Critical/High severity first (max 5), then fill with next-highest priority
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

    // Build module list
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

    // Build severity block from normalized counts
    let severity = SeverityBlock {
        critical: normalized.severity_counts.critical as u32,
        high: normalized.severity_counts.high as u32,
        medium: normalized.severity_counts.medium as u32,
        low: normalized.severity_counts.low as u32,
        total: normalized.severity_counts.total as u32,
        has_issues: normalized.severity_counts.total > 0,
    };

    // Build modules block
    let modules = build_modules_block_from_normalized(normalized);

    // Build summary metrics
    let quick_win_count = action_plan.quick_wins.len();
    let critical_count =
        (normalized.severity_counts.critical + normalized.severity_counts.high) as u32;
    let total_violations = normalized.severity_counts.total as u32;
    let nodes_analyzed = normalized.nodes_analyzed;

    // Build actions block (pre-mapped for ActionRoadmap component)
    let actions = build_actions_block(&action_plan, score as f32);

    // Build module details from raw data
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
            executive_lead: build_executive_lead(normalized),
            verdict: build_verdict_text(&normalized.url, score as f32),
            score_note: build_score_note(normalized),
            metrics: vec![
                MetricItem {
                    title: format!("Verstöße{NBSP}gesamt"),
                    value: total_violations.to_string(),
                    accent_color: None,
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

// ─── Block Builders ─────────────────────────────────────────────────────────

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

    let has_multiple = dashboard.len() > 1;
    let overall_score = if has_multiple {
        Some(normalized.overall_score)
    } else {
        None
    };
    let overall_interpretation = overall_score.map(|_| {
        "Gewichteter Durchschnitt aller aktiven Module. Accessibility fließt mit 40% ein, \
         Performance und SEO mit je 20%, Sicherheit und Mobile mit je 10%."
            .to_string()
    });

    ModulesBlock {
        dashboard,
        overall_score,
        overall_interpretation,
    }
}

fn build_actions_block(plan: &ActionPlan, score: f32) -> ActionsBlock {
    // Good site: high score or very few remaining actions
    let is_good_site = score >= 85.0
        || (plan.quick_wins.is_empty() && plan.medium_term.len() + plan.structural.len() <= 3);
    // Cap items per column for good sites to keep the section compact
    let item_cap: usize = if is_good_site { 2 } else { usize::MAX };

    let map_items = |items: &[ActionItem]| -> Vec<RoadmapItemData> {
        items
            .iter()
            .map(|i| {
                // Build effects from the finding group context (approximated from ActionItem)
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

    // Build phase preview
    let mut phase_preview = Vec::new();
    if !plan.quick_wins.is_empty() {
        phase_preview.push(PhasePreview {
            phase_label: "Phase 1 – Sofort beheben".into(),
            accent_color: "#dc2626".into(),
            description: "Blockierende Issues — direkter Impact auf Nutzbarkeit".into(),
            item_count: plan.quick_wins.len(),
            top_items: plan
                .quick_wins
                .iter()
                .take(3)
                .map(|i| i.action.clone())
                .collect(),
        });
    }
    if !plan.medium_term.is_empty() {
        phase_preview.push(PhasePreview {
            phase_label: "Phase 2 – Struktur verbessern".into(),
            accent_color: "#f59e0b".into(),
            description: "Semantik, Navigation und Barrierefreiheits-Struktur".into(),
            item_count: plan.medium_term.len(),
            top_items: plan
                .medium_term
                .iter()
                .take(3)
                .map(|i| i.action.clone())
                .collect(),
        });
    }
    if !plan.structural.is_empty() {
        phase_preview.push(PhasePreview {
            phase_label: "Phase 3 – Optimierung".into(),
            accent_color: "#2563eb".into(),
            description: "Langfristige Qualität, SEO und Performance".into(),
            item_count: plan.structural.len(),
            top_items: plan
                .structural
                .iter()
                .take(3)
                .map(|i| i.action.clone())
                .collect(),
        });
    }

    let capped = |items: &[ActionItem]| -> Vec<ActionItem> {
        items.iter().take(item_cap).cloned().collect()
    };

    let mut columns = Vec::new();
    if !plan.quick_wins.is_empty() {
        columns.push(RoadmapColumnData {
            title: "Phase 1 – Sofort".into(),
            accent_color: "#dc2626".into(),
            items: map_items(&capped(&plan.quick_wins)),
        });
    }
    if !plan.medium_term.is_empty() {
        columns.push(RoadmapColumnData {
            title: "Phase 2 – Als Nächstes".into(),
            accent_color: "#f59e0b".into(),
            items: map_items(&capped(&plan.medium_term)),
        });
    }
    if !plan.structural.is_empty() {
        columns.push(RoadmapColumnData {
            title: "Phase 3 – Langfristig".into(),
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
    // Build appendix from normalized findings (already suppressed/filtered)
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

        // Render-blocking data
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

    let has_any = performance.is_some()
        || seo.is_some()
        || security.is_some()
        || mobile.is_some()
        || dark_mode.is_some();
    ModuleDetailsBlock {
        performance,
        seo,
        security,
        mobile,
        dark_mode,
        has_any,
    }
}

/// Convert a NormalizedFinding into a FindingGroup (with explanation enrichment)
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

// ─── Batch Report Builder (unchanged) ───────────────────────────────────────

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
    url_ranking.sort_by(|a, b| a.score.partial_cmp(&b.score).unwrap());

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

    // ── Budget violations: aggregate across all pages ──────────────────────
    // Count how many URLs violated each budget metric and pick the worst severity.
    let budget_summary: Vec<(String, String, usize, String)> = {
        use std::collections::HashMap;
        // key: metric label → (budget_label, url_count, worst_severity)
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

    // ── Render-blocking: aggregate across all pages ─────────────────────────
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

// ─── Internal helpers ───────────────────────────────────────────────────────

/// Look up taxonomy metadata for a rule by its WCAG ID
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
            let (dimension, subcategory, issue_class, mapped_rule_id) = taxonomy_fields(&rule_id);
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

fn severity_to_priority(severity: Severity) -> Priority {
    match severity {
        Severity::Critical => Priority::Critical,
        Severity::High => Priority::High,
        Severity::Medium => Priority::Medium,
        Severity::Low => Priority::Low,
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
        Severity::High => 3,
        Severity::Medium => 2,
        Severity::Low => 1,
    };
    severity_weight * group.occurrence_count as u32
}

fn build_verdict_text(url: &str, score: f32) -> String {
    if score >= 90.0 {
        format!(
            "{url} erreicht {score:.0}/100 im Accessibility-Audit. \
             Die verbleibenden Findings sind letzte Optimierungshebel — kein strukturelles Problem, sondern Feinschliff.",
        )
    } else if score >= 70.0 {
        format!(
            "{url} erreicht {score:.0}/100 im Accessibility-Audit. \
             Die Basis ist solide — klarer Verbesserungshebel mit überschaubarem Aufwand.",
        )
    } else if score >= 50.0 {
        format!(
            "{url} erreicht {score:.0}/100 im Accessibility-Audit. \
             Es bestehen deutliche Barrieren — nicht nur Detailprobleme, sondern struktureller Nachholbedarf.",
        )
    } else {
        format!(
            "{url} erreicht nur {score:.0}/100 im Accessibility-Audit. \
             Akuter Handlungsbedarf: Wesentliche Inhalte und Funktionen sind für einen Teil der Nutzer nicht zugänglich.",
        )
    }
}

fn build_executive_lead(normalized: &NormalizedReport) -> String {
    let score = normalized.score;
    let critical = normalized.severity_counts.critical;
    let high = normalized.severity_counts.high;
    let urgent = critical + high;
    let total = normalized.severity_counts.total;

    // Gather cross-module signals for richer context
    let has_security_issues = normalized
        .raw_security
        .as_ref()
        .map(|s| s.score < 60)
        .unwrap_or(false);
    let has_mobile_issues = normalized
        .raw_mobile
        .as_ref()
        .map(|m| m.score < 60)
        .unwrap_or(false);
    let high_dom = normalized.nodes_analyzed > 1500;

    // Detect structural overlap: SEO issues that are simultaneously accessibility issues
    let has_weak_seo = normalized
        .raw_seo
        .as_ref()
        .map(|s| s.score < 65)
        .unwrap_or(false);
    let has_heading_issues = normalized.findings.iter().any(|f| {
        f.rule_id.to_lowercase().contains("heading")
            || f.title.to_lowercase().contains("überschrift")
            || f.title.to_lowercase().contains("h1")
    });
    let seo_a11y_overlap = has_weak_seo && has_heading_issues;

    // Build a cross-module context hint when multiple weak areas align
    let context_hint = if seo_a11y_overlap {
        " Fehlende Struktur (Überschriften, Inhalt) betrifft gleichzeitig SEO-Sichtbarkeit und Zugänglichkeit."
    } else {
        match (has_security_issues, has_mobile_issues, high_dom) {
            (true, true, _) => " Auch Security und Mobile weisen Optimierungsbedarf auf — die Schwächen ziehen sich durch mehrere Bereiche.",
            (true, false, _) => " Zusätzlich bestehen offene Security-Themen.",
            (false, true, _) => " Mobile-Nutzbarkeit ist ebenfalls verbesserungswürdig.",
            (false, false, true) => " Die hohe DOM-Komplexität erschwert automatisierte Prüfungen.",
            _ => "",
        }
    };

    // Systematic scale: 40+ issues signals a process problem, not individual failures
    let is_systematic = total > 40 || (critical >= 5 && total > 25);

    if is_systematic {
        format!(
            "Kein Einzelproblem — {total} Verstöße über {critical} kritische und {high} hohe Themen sind ein systematisches Muster. Betrifft große Teile der Seite, nicht einzelne Stellen.{context_hint}"
        )
    } else if score >= 90 && urgent == 0 {
        format!("Sehr gutes Ergebnis — keine dringenden Barrieren.{context_hint} Das Niveau halten und regelmäßig nachprüfen.")
    } else if score >= 90 {
        format!(
            "Technisch stark aufgestellt. {urgent} kritische{} Thema{} {} — jetzt gezielt beheben, bevor sie sich häufen.{context_hint}",
            if urgent == 1 { "s" } else { "" },
            if urgent == 1 { "" } else { "n" },
            if urgent == 1 { "wartet" } else { "warten" },
        )
    } else if score >= 75 && urgent == 0 {
        format!("Solide Basis ohne akute Risiken. {total} Verbesserungen möglich — gut priorisierbar und umzusetzen.{context_hint}")
    } else if score >= 75 {
        format!(
            "Gute Basis, aber {urgent} priorisierte{} Thema{} braucht{} sofortige Aufmerksamkeit.{context_hint}",
            if urgent == 1 { "s" } else { "" },
            if urgent == 1 { "" } else { "n" },
            if urgent == 1 { "" } else { "n" }
        )
    } else if score >= 50 {
        format!(
            "Relevante Barrieren vorhanden — {urgent} davon kritisch oder hoch. Jetzt strukturiert priorisieren und Phase 1 starten.{context_hint}"
        )
    } else {
        format!(
            "Akuter Handlungsbedarf: {critical} kritische, {high} hohe Issues. Die Seite ist für einen Teil der Nutzer schwer nutzbar — sofort Phase 1 starten.{context_hint}"
        )
    }
}

fn build_score_note(normalized: &NormalizedReport) -> Option<String> {
    let critical_topics = normalized.severity_counts.critical + normalized.severity_counts.high;
    if normalized.score >= 90 && critical_topics > 0 {
        Some(
            "Der Score berücksichtigt Gewichtung und Häufigkeit. Einzelne kritische Themen können trotz hoher Gesamtbewertung bestehen."
                .to_string(),
        )
    } else {
        None
    }
}

/// 4-level maturity classification based on score and severity profile
fn build_maturity_label(score: u32, critical: usize, high: usize) -> String {
    let urgent = critical + high;
    if score < 50 || critical >= 3 {
        "Kritisch".to_string()
    } else if score < 70 || (critical >= 1 && urgent >= 3) {
        "Instabil".to_string()
    } else if score < 88 || urgent > 0 {
        "Solide Basis".to_string()
    } else {
        "Stark".to_string()
    }
}

/// Overall impact assessment for the summary — user experience, risk, conversion effect
fn build_overall_impact(normalized: &NormalizedReport) -> Vec<(String, String)> {
    let score = normalized.score;
    let critical = normalized.severity_counts.critical;
    let high = normalized.severity_counts.high;
    let urgent = critical + high;

    let user_rating = if score >= 90 && urgent == 0 {
        "Sehr gut — keine relevanten Barrieren"
    } else if score >= 75 {
        "Gut — einzelne Barrieren für Hilfstechnologien"
    } else if score >= 50 {
        "Eingeschränkt — spürbare Barrieren für Screenreader-Nutzer"
    } else {
        "Stark eingeschränkt — wesentliche Inhalte nicht zugänglich"
    };

    let risk_level = if critical >= 2 {
        "Hoch — BITV/WCAG-Verstoßrisiko akut"
    } else if critical >= 1 || urgent >= 3 {
        "Mittel — kritische Themen vorhanden"
    } else if score < 70 {
        "Mittel — kumulierter Nachholbedarf"
    } else {
        "Niedrig"
    };

    let conversion = if score < 50 {
        "Hoch wahrscheinlich negativ"
    } else if score < 75 {
        "Möglicherweise negativ (Navigation, Formulare)"
    } else {
        "Gering — gute Nutzbarkeit"
    };

    vec![
        ("Nutzererlebnis".to_string(), user_rating.to_string()),
        ("Risiko-Level".to_string(), risk_level.to_string()),
        ("Conversion-Effekt".to_string(), conversion.to_string()),
    ]
}

/// Magnitude-based trend label for history deltas
fn build_trend_label(delta_accessibility: i32, delta_issues: i32) -> String {
    if delta_accessibility >= 10 || (delta_accessibility >= 5 && delta_issues <= -5) {
        "Deutlich verbessert".to_string()
    } else if delta_accessibility > 0 || delta_issues < 0 {
        "Verbessert".to_string()
    } else if delta_accessibility == 0 && delta_issues == 0 {
        "Stabil".to_string()
    } else if delta_accessibility >= -5 && delta_issues <= 5 {
        "Leicht zurückgegangen".to_string()
    } else {
        "Deutlich verschlechtert".to_string()
    }
}

/// Problem distribution type based on issue count, severity spread, and rule diversity
fn build_problem_type(normalized: &NormalizedReport) -> String {
    let total = normalized.severity_counts.total;
    let critical = normalized.severity_counts.critical;
    let high = normalized.severity_counts.high;
    let rule_count = normalized
        .findings
        .iter()
        .map(|f| f.wcag_criterion.as_str())
        .collect::<std::collections::HashSet<_>>()
        .len();

    // "Structural" requires flächendeckend: many issues AND broad rule spread
    // Concentrated critical problems (e.g. 2 critical, 9 high but only 18 total) are "Einzelprobleme"
    let is_structural = total > 30
        || (critical >= 5 && total > 20)
        || (rule_count >= 7 && total > 25 && (critical + high) >= 15);

    if is_structural {
        "Strukturelle Defizite — flächendeckende Barrieren in mehreren Bereichen".to_string()
    } else if total > 8 || critical >= 2 || (critical + high) >= 5 {
        "Mehrere kritische Einzelprobleme — konzentriert und gezielt behebbar".to_string()
    } else if total > 0 {
        "Feinschliff — keine strukturellen Defizite, letzte Optimierungshebel".to_string()
    } else {
        "Keine Verstöße gefunden — volle Konformität im geprüften Umfang".to_string()
    }
}

/// Cross-module technical overview: always exactly 4 categories for consistent framework feel
fn build_technical_overview(normalized: &NormalizedReport) -> Vec<String> {
    let mut insights = Vec::new();

    let critical = normalized.severity_counts.critical;
    let high = normalized.severity_counts.high;
    let total = normalized.severity_counts.total;
    let rule_count = normalized.findings.len();

    // 1. Accessibility-Systematik — always present
    let a11y = if total == 0 {
        "Accessibility-Systematik: Keine Verstöße — Basis vollständig konform".to_string()
    } else if critical >= 5 && total > 30 {
        format!("Accessibility-Systematik: Systematische Muster ({rule_count} Regeltypen, {total} Instanzen) — Prozess-Problem, kein Einzelfall")
    } else if critical >= 3 || (critical >= 2 && rule_count >= 5) {
        format!("Accessibility-Systematik: Mehrere kritische Blockaden ({critical} kritisch, {high} hoch) — direkte Screenreader-Barrieren")
    } else if total > 10 {
        format!("Accessibility-Systematik: Verteilt über {rule_count} Regeltypen — kein Muster, einzeln behebbar")
    } else {
        format!("Accessibility-Systematik: {total} Verstöße in {rule_count} Bereichen — konzentriert und gezielt behebbar")
    };
    insights.push(a11y);

    // 2. SEO-Level — always present
    let seo = if let Some(ref s) = normalized.raw_seo {
        if s.score >= 85 {
            format!(
                "SEO-Level: {} Pkt — technische Ranking-Voraussetzungen erfüllt",
                s.score
            )
        } else if s.score >= 65 {
            format!(
                "SEO-Level: {} Pkt — Basis vorhanden, gezielte Optimierungen möglich",
                s.score
            )
        } else if s.score >= 45 {
            format!(
                "SEO-Level: {} Pkt — relevante Signale fehlen, Sichtbarkeit eingeschränkt",
                s.score
            )
        } else {
            format!(
                "SEO-Level: {} Pkt — strukturelle Basis fehlt, Ranking praktisch ausgeschlossen",
                s.score
            )
        }
    } else {
        "SEO-Level: Nicht geprüft (--full für vollständige Analyse)".to_string()
    };
    insights.push(seo);

    // 3. Security-Level — always present
    let sec = if let Some(ref s) = normalized.raw_security {
        if s.score >= 80 {
            format!(
                "Security-Level: {} Pkt — HTTP-Security-Header vollständig gesetzt",
                s.score
            )
        } else if s.score >= 55 {
            format!(
                "Security-Level: {} Pkt — Grundschutz vorhanden, einzelne Header fehlen",
                s.score
            )
        } else if s.score >= 30 {
            format!(
                "Security-Level: {} Pkt — mehrere kritische Security-Header fehlen",
                s.score
            )
        } else {
            format!("Security-Level: {} Pkt — Security-Header fehlen fast vollständig — hohes Risiko, schnell behebbar", s.score)
        }
    } else {
        "Security-Level: Nicht geprüft (--full für vollständige Analyse)".to_string()
    };
    insights.push(sec);

    // 4. Tech-Komplexität — DOM + performance combined
    let dom = normalized.nodes_analyzed;
    let perf_score = normalized.raw_performance.as_ref().map(|p| p.score.overall);
    let tech = match (dom, perf_score) {
        (d, Some(p)) if d > 2000 && p < 60 => format!(
            "Tech-Komplexität: Hoch — {d} DOM-Knoten, Performance {p} Pkt — Refactoring empfohlen"
        ),
        (d, Some(p)) if d > 2000 => format!(
            "Tech-Komplexität: Mittel-hoch — {d} DOM-Knoten (Performance {p} Pkt stabil)"
        ),
        (d, Some(p)) if p < 60 => format!(
            "Tech-Komplexität: Performance kritisch ({p} Pkt) — {d} DOM-Knoten analysiert"
        ),
        (d, Some(p)) if p < 80 => format!(
            "Tech-Komplexität: Gering — {d} DOM-Knoten, Performance optimierbar ({p} Pkt)"
        ),
        (d, Some(p)) => format!(
            "Tech-Komplexität: Gering — {d} DOM-Knoten, Performance {p} Pkt — technische Basis stabil"
        ),
        (d, None) if d > 2000 => format!(
            "Tech-Komplexität: Hoch — {d} DOM-Knoten (Performance nicht geprüft)"
        ),
        (d, None) => format!(
            "Tech-Komplexität: {d} DOM-Knoten analysiert (Performance nicht geprüft, --full)"
        ),
    };
    insights.push(tech);

    insights
}

/// Generate executive-oriented effects for an action item based on its finding context
/// SEO interpretation: contextual reading of the SEO score
/// Score-range benchmark: contextualizes the score against a typical distribution
/// Based on internal reference data from audited sites (approximate percentile buckets).
fn build_benchmark_context(score: f32) -> String {
    if score >= 95.0 {
        "Top 5% — Ausnahmeniveau. Kein struktureller Handlungsdruck.".to_string()
    } else if score >= 90.0 {
        "Top 15% — Deutlich besser als die Mehrheit. Feinschliff genügt.".to_string()
    } else if score >= 80.0 {
        "Oberes Drittel — Guter Stand, einzelne Optimierungen lohnen sich.".to_string()
    } else if score >= 70.0 {
        "Mittleres Feld — Verbesserungspotenzial vorhanden, kein akuter Notfall.".to_string()
    } else if score >= 55.0 {
        "Unteres Mittelfeld — Deutlicher Rückstand gegenüber vergleichbaren Websites.".to_string()
    } else if score >= 40.0 {
        "Unteres Drittel — Erheblicher Rückstand, strukturelle Defizite häufig.".to_string()
    } else {
        "Kritisch — Zu den schwächsten geprüften Seiten. Sofortiger Handlungsbedarf.".to_string()
    }
}

/// Forward-looking consequence: what happens without action, tailored to problem profile
/// Concrete current-state business consequence for the KV list ("Konsequenz" row)
fn build_business_consequence(normalized: &NormalizedReport) -> String {
    let critical = normalized.severity_counts.critical;
    let total = normalized.severity_counts.total;
    let score = normalized.score;

    if total == 0 {
        return "Keine bekannten Barrieren — gutes Fundament für alle Nutzergruppen.".to_string();
    }

    let has_weak_seo = normalized.raw_seo.as_ref().is_some_and(|s| s.score < 65);
    let has_heading_issues = normalized.findings.iter().any(|f| {
        f.rule_id.to_lowercase().contains("heading")
            || f.title.to_lowercase().contains("überschrift")
    });

    if score < 50 || (critical >= 5 && total > 30) {
        "Weite Teile der Seite sind für bestimmte Nutzergruppen nicht oder kaum nutzbar."
            .to_string()
    } else if has_weak_seo && has_heading_issues {
        "Seite wird schlechter gefunden und ist für Teile der Nutzer strukturell nicht zugänglich."
            .to_string()
    } else if critical >= 2 {
        "Einzelne Kernfunktionen sind für Screenreader-Nutzer blockiert oder fehleranfällig."
            .to_string()
    } else {
        "Nutzbarkeit ist gegeben — gezielte Verbesserungen erhöhen Qualität und Reichweite."
            .to_string()
    }
}

/// Forward-looking consequence: what happens without action, concretely framed
fn build_consequence_text(normalized: &NormalizedReport) -> String {
    let critical = normalized.severity_counts.critical;
    let total = normalized.severity_counts.total;
    let score = normalized.score;

    if total == 0 {
        return String::new();
    }

    let weak_module_count = [
        normalized
            .raw_security
            .as_ref()
            .is_some_and(|s| s.score < 60),
        normalized.raw_seo.as_ref().is_some_and(|s| s.score < 60),
        normalized
            .raw_performance
            .as_ref()
            .is_some_and(|p| p.score.overall < 70),
        normalized.raw_mobile.as_ref().is_some_and(|m| m.score < 65),
    ]
    .iter()
    .filter(|&&v| v)
    .count();

    if score < 50 || (critical >= 5 && total > 30) {
        "Neue Inhalte und Funktionen erben die bestehenden Fehler — Korrekturaufwand wächst mit jeder Erweiterung.".to_string()
    } else if critical >= 3 || weak_module_count >= 3 {
        "Aufwand für spätere Korrekturen steigt deutlich — besonders bei Relaunch oder größerem Content-Ausbau.".to_string()
    } else if score >= 85 {
        "Kein akuter Handlungsdruck. Regelmäßige Checks sichern das Niveau nach Updates und Erweiterungen.".to_string()
    } else {
        "Ohne Korrektur bleibt die Seite hinter erreichbarem Standard — Verbesserungspotenzial wird nicht genutzt.".to_string()
    }
}

fn build_seo_interpretation(seo: &crate::seo::SeoAnalysis) -> String {
    if seo.score >= 90 {
        "Sehr gute SEO-Basis — technische Voraussetzungen für gutes Ranking erfüllt.".to_string()
    } else if seo.score >= 70 {
        "Solide SEO-Basis mit gezieltem Optimierungspotenzial.".to_string()
    } else if seo.score >= 55 {
        "SEO-Basis lückenhaft — relevante Ranking-Signale fehlen, Sichtbarkeit deutlich eingeschränkt.".to_string()
    } else if seo.score >= 35 {
        "SEO unzureichend — wesentliche Grundlagen fehlen. Ranking in kompetitiven Bereichen quasi unmöglich.".to_string()
    } else {
        "SEO kritisch — Seite ist für Suchmaschinen kaum indexierbar. Nicht wettbewerbsfähig."
            .to_string()
    }
}

fn localized_report_title(locale: &str) -> String {
    match locale {
        "en" => "Accessibility Audit Report".to_string(),
        _ => "Barrierefreiheits-Prüfbericht".to_string(),
    }
}

fn localized_report_subtitle(locale: &str) -> &'static str {
    match locale {
        "en" => "Automated accessibility audit with optional website quality modules.",
        _ => "Automatisierter Accessibility-Report mit ergänzenden Qualitätsmodulen.",
    }
}

#[cfg(test)]
#[allow(
    clippy::items_after_test_module,
    clippy::too_many_arguments,
    clippy::field_reassign_with_default
)]
mod tests {
    use super::*;
    use crate::audit::{normalize, AuditReport, BatchReport};
    use crate::cli::WcagLevel;
    use crate::output::report_model::ReportConfig;
    use crate::performance::{PerformanceGrade, PerformanceScore, WebVitals};
    use crate::seo::technical::TechnicalSeo;
    use crate::seo::SeoAnalysis;
    use crate::seo::{HeadingStructure, MetaTags};
    use crate::wcag::{Violation, WcagResults};

    #[test]
    fn test_view_model_uses_accessibility_score_as_primary_score() {
        let mut wcag = WcagResults::new();
        wcag.add_violation(Violation::new(
            "1.1.1",
            "Non-text Content",
            WcagLevel::A,
            crate::taxonomy::Severity::High,
            "Missing alt text",
            "n1",
        ));

        let report = AuditReport::new("https://example.com".into(), WcagLevel::AA, wcag, 1500)
            .with_performance(crate::audit::PerformanceResults {
                vitals: WebVitals::default(),
                score: PerformanceScore {
                    overall: 60,
                    grade: PerformanceGrade::NeedsImprovement,
                    lcp_score: 15,
                    fcp_score: 15,
                    cls_score: 15,
                    interactivity_score: 15,
                },
                render_blocking: None,
                content_weight: None,
            })
            .with_seo(SeoAnalysis::default());

        let normalized = normalize(&report);
        let vm = build_view_model(&normalized, &ReportConfig::default());

        assert_eq!(vm.summary.score, normalized.score);
        assert_eq!(vm.summary.grade, normalized.grade);
        assert_eq!(vm.summary.certificate, normalized.certificate);
        assert!(vm
            .summary
            .metrics
            .iter()
            .any(|m| m.title == format!("Gesamtscore{NBSP}Website")));
    }

    #[test]
    fn test_batch_presentation_includes_topics_and_overlap() {
        let reports = vec![
            make_topic_report(
                "https://example.com/cloud-entwicklung/",
                "Container Deployment Plattform Architektur",
                "Container Deployment fuer Plattformen und Kubernetes Betrieb.",
                &["Container Deployment", "Plattform Architektur"],
                "Container Deployment Kubernetes Plattform Architektur Betrieb",
                72.0,
            ),
            make_topic_report(
                "https://example.com/cloud-migration/",
                "Container Deployment Migration Plattform",
                "Container Deployment fuer Migration und Plattform Betrieb.",
                &["Container Deployment", "Migration Plattform"],
                "Container Deployment Migration Plattform Betrieb",
                68.0,
            ),
        ];

        let batch = BatchReport::from_reports(reports, vec![], 1200);
        let pres = build_batch_presentation(&batch);

        assert!(!pres.portfolio_summary.top_topics.is_empty());
        assert!(pres
            .portfolio_summary
            .top_topics
            .iter()
            .any(|(topic, _)| topic == "container" || topic == "deployment"));
        assert!(!pres.portfolio_summary.overlap_pairs.is_empty());
        assert!(pres
            .url_details
            .iter()
            .all(|detail| !detail.topic_terms.is_empty()));
    }

    #[test]
    fn test_batch_presentation_filters_generic_topic_tokens() {
        let report = make_topic_report(
            "https://example.com/arbeitsweise/",
            "Klare Arbeitsweise fuer digitale Projekte",
            "Willkommen. Drei Schritte fuer transparente Zusammenarbeit.",
            &["Klare Arbeitsweise", "Drei Schritte"],
            "Willkommen transparente Zusammenarbeit drei Schritte fuer Projekte",
            71.0,
        );
        let batch = BatchReport::from_reports(vec![report], vec![], 800);
        let pres = build_batch_presentation(&batch);
        let terms = &pres.url_details[0].topic_terms;

        assert!(!terms.iter().any(|term| term == "fuer" || term == "drei"));
    }

    #[test]
    fn test_batch_presentation_populates_ranking_and_matrix_inputs() {
        let first = make_topic_report_with_modules(
            "https://example.com/arbeitsweise/",
            "Container Deployment Plattform Architektur",
            "Container Deployment fuer Plattformen und Kubernetes Betrieb.",
            &["Container Deployment", "Plattform Architektur"],
            "Container Deployment Kubernetes Plattform Architektur Betrieb",
            72.0,
            91,
            63,
            95,
        );

        let second = make_topic_report_with_modules(
            "https://example.com/datenschutz/",
            "Datenschutz und DSGVO Grundlagen",
            "Datenschutz Hinweise fuer Website und DSGVO Prozesse.",
            &["Datenschutz", "DSGVO Grundlagen"],
            "Datenschutz DSGVO Website Prozesse Hinweise Rechtsgrundlagen",
            68.0,
            88,
            57,
            93,
        );

        let batch = BatchReport::from_reports(vec![first, second], vec![], 1400);
        let pres = build_batch_presentation(&batch);

        assert_eq!(pres.url_details.len(), 2);
        assert!(pres
            .url_details
            .iter()
            .all(|detail| !detail.topic_terms.is_empty()));
        assert!(pres.url_details.iter().all(|detail| detail
            .module_scores
            .iter()
            .any(|(module, _)| module == "SEO")));
        assert!(pres.url_details.iter().all(|detail| detail
            .module_scores
            .iter()
            .any(|(module, _)| module == "Performance")));
        assert!(pres.url_details.iter().all(|detail| detail
            .module_scores
            .iter()
            .any(|(module, _)| module == "Security")));
        assert!(pres
            .portfolio_summary
            .top_topics
            .iter()
            .any(|(topic, _)| topic == "container" || topic == "datenschutz"));
    }

    fn make_topic_report_with_modules(
        url: &str,
        title: &str,
        description: &str,
        headings: &[&str],
        text_excerpt: &str,
        score: f32,
        seo_score: u32,
        performance_score: u32,
        security_score: u32,
    ) -> AuditReport {
        make_topic_report(url, title, description, headings, text_excerpt, score)
            .with_performance(crate::audit::PerformanceResults {
                vitals: WebVitals::default(),
                score: PerformanceScore {
                    overall: performance_score,
                    grade: PerformanceGrade::NeedsImprovement,
                    lcp_score: 15,
                    fcp_score: 15,
                    cls_score: 15,
                    interactivity_score: 15,
                },
                render_blocking: None,
                content_weight: None,
            })
            .with_security(crate::security::SecurityAnalysis {
                score: security_score,
                grade: "A".to_string(),
                headers: crate::security::SecurityHeaders {
                    content_security_policy: Some("default-src 'self'".to_string()),
                    x_frame_options: Some("DENY".to_string()),
                    x_content_type_options: Some("nosniff".to_string()),
                    x_xss_protection: Some("1; mode=block".to_string()),
                    referrer_policy: Some("strict-origin-when-cross-origin".to_string()),
                    permissions_policy: None,
                    strict_transport_security: Some(
                        "max-age=31536000; includeSubDomains".to_string(),
                    ),
                    cross_origin_opener_policy: None,
                    cross_origin_resource_policy: None,
                },
                ssl: crate::security::SslInfo {
                    https: true,
                    valid_certificate: true,
                    has_hsts: true,
                    hsts_max_age: Some(31536000),
                    hsts_include_subdomains: true,
                    hsts_preload: false,
                },
                issues: vec![],
                recommendations: vec![],
            })
            .with_seo({
                let mut seo = SeoAnalysis::default();
                seo.meta = MetaTags {
                    title: Some(title.to_string()),
                    description: Some(description.to_string()),
                    keywords: None,
                    robots: None,
                    author: None,
                    viewport: Some("width=device-width, initial-scale=1".to_string()),
                    charset: Some("utf-8".to_string()),
                    canonical: Some(url.to_string()),
                    lang: Some("de".to_string()),
                };
                let mut heading_structure = HeadingStructure::default();
                heading_structure.h1_count = 1;
                heading_structure.h1_text = headings.first().map(|value| (*value).to_string());
                heading_structure.total_count = headings.len();
                seo.headings = heading_structure;
                seo.technical = TechnicalSeo {
                    https: true,
                    has_canonical: true,
                    canonical_url: Some(url.to_string()),
                    has_lang: true,
                    lang: Some("de".to_string()),
                    has_robots_meta: false,
                    robots_meta: None,
                    has_hreflang: false,
                    hreflang: vec![],
                    word_count: 650,
                    internal_links: 12,
                    external_links: 1,
                    broken_links: vec![],
                    text_excerpt: text_excerpt.to_string(),
                    uses_remote_google_fonts: false,
                    google_fonts_sources: vec![],
                    tracking_cookies: vec![],
                    tracking_signals: vec![],
                    zaraz: crate::seo::technical::ZarazDetection::default(),
                    issues: vec![],
                };
                seo.content_profile = Some(crate::seo::build_content_profile(&seo));
                seo.score = seo_score;
                seo
            })
    }

    fn make_topic_report(
        url: &str,
        title: &str,
        description: &str,
        headings: &[&str],
        text_excerpt: &str,
        score: f32,
    ) -> AuditReport {
        let mut seo = SeoAnalysis::default();
        seo.meta = MetaTags {
            title: Some(title.to_string()),
            description: Some(description.to_string()),
            keywords: None,
            robots: None,
            author: None,
            viewport: Some("width=device-width, initial-scale=1".to_string()),
            charset: Some("utf-8".to_string()),
            canonical: None,
            lang: Some("de".to_string()),
        };
        let mut heading_structure = HeadingStructure::default();
        heading_structure.h1_count = 1;
        heading_structure.h1_text = headings.first().map(|value| (*value).to_string());
        heading_structure.total_count = headings.len();
        seo.headings = heading_structure;
        seo.technical = TechnicalSeo {
            https: true,
            has_canonical: true,
            canonical_url: Some(url.to_string()),
            has_lang: true,
            lang: Some("de".to_string()),
            has_robots_meta: false,
            robots_meta: None,
            has_hreflang: false,
            hreflang: vec![],
            word_count: 650,
            internal_links: 12,
            external_links: 1,
            broken_links: vec![],
            text_excerpt: text_excerpt.to_string(),
            uses_remote_google_fonts: false,
            google_fonts_sources: vec![],
            tracking_cookies: vec![],
            tracking_signals: vec![],
            zaraz: crate::seo::technical::ZarazDetection::default(),
            issues: vec![],
        };
        seo.content_profile = Some(crate::seo::build_content_profile(&seo));
        seo.score = 92;

        let mut report = AuditReport::new(url.to_string(), WcagLevel::AA, WcagResults::new(), 1500)
            .with_seo(seo);
        report.score = score;
        report.grade = grade_label(score.round() as u32).to_string();
        report
    }
}

fn derive_user_effect_from_action(action: &str, effort: Effort) -> String {
    let a = action.to_lowercase();
    // Match against humanized action texts for maximum specificity
    if a.contains("buttons") || a.contains("schaltflächen") {
        "Nutzer verstehen Schaltflächen sofort — weniger Fehlklicks".to_string()
    } else if a.contains("links verständlich") || a.contains("links eindeutig") {
        "Navigation klarer — Nutzer finden Ziele schneller".to_string()
    } else if a.contains("interaktive elemente") || a.contains("aria-label") {
        "Alle Bedienelemente klar benannt — Screenreader-Nutzung ohne Ratespiel".to_string()
    } else if a.contains("bilder") || a.contains("alternativtext") || a.contains("alt-text") {
        "Bilder verständlich für Nutzer ohne Sehvermögen".to_string()
    } else if a.contains("kontrast") || a.contains("farbkontrast") {
        "Text für alle Nutzer gut lesbar — auch bei schlechten Lichtverhältnissen".to_string()
    } else if a.contains("formular") && a.contains("beschrift") {
        "Formulare ausfüllbar ohne Verwirrung — weniger Abbrüche".to_string()
    } else if a.contains("überschrift") || a.contains("heading") {
        "Inhaltsstruktur sofort erfassbar — schnellere Orientierung".to_string()
    } else if a.contains("sprunglink") || a.contains("skip") {
        "Tastaturnutzer gelangen direkt zum Hauptinhalt".to_string()
    } else if a.contains("tastatur") || a.contains("keyboard") || a.contains("fokus") {
        "Vollständige Bedienbarkeit ohne Maus".to_string()
    } else if a.contains("sprache") || a.contains("lang-attribut") {
        "Screenreader liest Inhalte in korrekter Sprache und Betonung".to_string()
    } else if a.contains("seitentitel") || a.contains("title") {
        "Seite klar identifizierbar in Browser-Tab und Suche".to_string()
    } else if a.contains("landmark") || a.contains("orientierungspunkt") {
        "Screenreader-Nutzer navigieren strukturiert durch die Seite".to_string()
    } else {
        match effort {
            Effort::Quick => "Direkte, spürbare Verbesserung der Nutzererfahrung".to_string(),
            Effort::Medium => "Merkliche Verbesserung für betroffene Nutzergruppen".to_string(),
            Effort::Structural => "Langfristig inklusivere Nutzererfahrung für alle".to_string(),
        }
    }
}

fn derive_conversion_effect_from_action(action: &str, effort: Effort) -> String {
    let action_lower = action.to_lowercase();
    if action_lower.contains("link") || action_lower.contains("navigation") {
        "Klarere Navigation → weniger Absprünge".to_string()
    } else if action_lower.contains("kontrast") || action_lower.contains("contrast") {
        "Bessere Lesbarkeit → höhere Verweildauer".to_string()
    } else if action_lower.contains("heading") || action_lower.contains("h1") {
        "Strukturklarheit → schnellere Orientierung".to_string()
    } else if action_lower.contains("lang") {
        "Korrekte Sprachausgabe → keine Abbrüche durch Vorlesefehler".to_string()
    } else {
        match effort {
            Effort::Quick => "Schnell wirksam — messbar innerhalb von Tagen".to_string(),
            Effort::Medium => "Mittelfristig messbare UX-Verbesserung".to_string(),
            Effort::Structural => "Solide technische Basis für weiteres Wachstum".to_string(),
        }
    }
}

/// Translate technical WCAG recommendation text into executive-friendly language.
/// Keeps specificity where the original is already clear; only replaces recognizable
/// technical shorthand that non-developers won't understand.
fn humanize_action_text(action: &str) -> String {
    let lower = action.to_lowercase();
    // aria-label / accessible name
    if lower.contains("aria-label") || lower.contains("aria_label") {
        return "Interaktive Elemente (Buttons, Links) verständlich benennen".to_string();
    }
    // alt text / alternative text
    if (lower.contains("alt-text") || lower.contains("alt text") || lower.contains("alt-attribut"))
        && !lower.contains("kein")
    {
        return "Bilder mit beschreibendem Alternativtext versehen".to_string();
    }
    // color contrast
    if lower.contains("kontrast") || lower.contains("contrast") {
        return "Farbkontraste für Text und UI-Elemente verbessern".to_string();
    }
    // form labels
    if (lower.contains("label") || lower.contains("beschriftung"))
        && (lower.contains("formular") || lower.contains("input") || lower.contains("feld"))
    {
        return "Formularfelder eindeutig beschriften".to_string();
    }
    // heading structure
    if lower.contains("überschrift") || (lower.contains("heading") && lower.contains("struktur")) {
        return "Überschriften-Hierarchie logisch strukturieren".to_string();
    }
    // keyboard / focus
    if lower.contains("tastatur")
        || lower.contains("keyboard")
        || lower.contains("fokus-reihenfolge")
    {
        return "Tastaturnavigation und Fokus-Reihenfolge sicherstellen".to_string();
    }
    // skip links
    if lower.contains("sprunglink") || lower.contains("skip link") || lower.contains("skip-link") {
        return "Sprunglinks für Screenreader-Nutzer ergänzen".to_string();
    }
    // language attribute
    if lower.contains("lang-attribut") || (lower.contains("sprache") && lower.contains("attribut"))
    {
        return "Seitensprache korrekt im HTML auszeichnen".to_string();
    }
    // page title
    if lower.contains("seitentitel") || (lower.contains("title") && lower.contains("tag")) {
        return "Aussagekräftigen Seitentitel vergeben".to_string();
    }
    // link text
    if lower.contains("linktext") || (lower.contains("link") && lower.contains("beschrift")) {
        return "Links verständlich und eindeutig beschriften".to_string();
    }
    // ARIA roles / landmarks
    if lower.contains("landmark") || (lower.contains("aria") && lower.contains("role")) {
        return "Seitenstruktur mit Orientierungspunkten auszeichnen".to_string();
    }
    // No match — return original
    action.to_string()
}

fn derive_action_plan(finding_groups: &[FindingGroup]) -> ActionPlan {
    let mut quick_wins = Vec::new();
    let mut medium_term = Vec::new();
    let mut structural = Vec::new();

    for group in finding_groups {
        let item = ActionItem {
            action: humanize_action_text(&group.recommendation),
            benefit: group.business_impact.clone(),
            role: group.responsible_role,
            priority: group.priority,
            execution_priority: group.execution_priority,
            effort: group.effort,
        };
        match group.effort {
            Effort::Quick => quick_wins.push(item),
            Effort::Medium => medium_term.push(item),
            Effort::Structural => structural.push(item),
        }
    }

    quick_wins.sort_by(|a, b| b.execution_priority.cmp(&a.execution_priority));
    medium_term.sort_by(|a, b| b.execution_priority.cmp(&a.execution_priority));
    structural.sort_by(|a, b| b.execution_priority.cmp(&a.execution_priority));

    // Deduplicate by action text across ALL phases (keep first occurrence = highest phase/priority)
    let mut seen_actions: std::collections::HashSet<String> = std::collections::HashSet::new();
    let dedup =
        |items: Vec<ActionItem>, seen: &mut std::collections::HashSet<String>| -> Vec<ActionItem> {
            items
                .into_iter()
                .filter(|i| seen.insert(i.action.clone()))
                .collect()
        };
    let quick_wins = dedup(quick_wins, &mut seen_actions);
    let medium_term = dedup(medium_term, &mut seen_actions);
    let structural = dedup(structural, &mut seen_actions);

    let mut role_map: HashMap<Role, Vec<String>> = HashMap::new();
    for group in finding_groups {
        role_map
            .entry(group.responsible_role)
            .or_default()
            .push(group.title.clone());
    }
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

fn derive_accessibility_lever(normalized: &NormalizedReport) -> String {
    if let Some(finding) = normalized
        .findings
        .iter()
        .max_by_key(|f| f.occurrence_count)
    {
        format!("Größter Hebel: {}", finding.title)
    } else {
        "Größter Hebel: Ergebnisse stabil halten und manuell nachprüfen".to_string()
    }
}

fn derive_accessibility_context(normalized: &NormalizedReport) -> String {
    let high = normalized
        .findings
        .iter()
        .filter(|f| matches!(f.severity, Severity::High | Severity::Critical))
        .count();
    let total = normalized.findings.len();
    if total == 0 {
        return "Keine automatisch erkannten Barrieren im aktuellen Lauf.".to_string();
    }
    format!(
        "{} erkannte Problemgruppe(n), davon {} mit hoher Priorität.",
        total, high
    )
}

fn derive_accessibility_card_context(normalized: &NormalizedReport) -> String {
    let high = normalized
        .findings
        .iter()
        .filter(|f| matches!(f.severity, Severity::High | Severity::Critical))
        .count();
    if high == 0 {
        "Keine High-Priority-Funde".to_string()
    } else {
        format!("{high} Problemgruppe(n) mit hoher Priorität")
    }
}

fn derive_performance_lever(perf: &crate::audit::PerformanceResults) -> String {
    if let Some(dom_nodes) = perf.vitals.dom_nodes {
        if dom_nodes > 1500 {
            return format!("Größter Hebel: DOM-Größe reduzieren ({dom_nodes} Knoten)");
        }
    }
    if let Some(load) = perf.vitals.load_time {
        if load > 2_500.0 {
            return format!("Größter Hebel: Ladezeit senken ({load:.0} ms)");
        }
    }
    "Größter Hebel: Render-Pfad und Asset-Größe weiter optimieren".to_string()
}

fn derive_performance_context(perf: &crate::audit::PerformanceResults) -> String {
    let fcp = perf
        .vitals
        .fcp
        .as_ref()
        .map(|v| format!("FCP {:.0} ms", v.value))
        .unwrap_or_else(|| "FCP n/a".to_string());
    let ttfb = perf
        .vitals
        .ttfb
        .as_ref()
        .map(|v| format!("TTFB {:.0} ms", v.value))
        .unwrap_or_else(|| "TTFB n/a".to_string());
    let dom = perf
        .vitals
        .dom_nodes
        .map(|n| format!("{n} DOM-Knoten"))
        .unwrap_or_else(|| "DOM-Knoten n/a".to_string());
    format!("{fcp}, {ttfb}, {dom}.")
}

fn derive_performance_card_context(perf: &crate::audit::PerformanceResults) -> String {
    if let Some(dom_nodes) = perf.vitals.dom_nodes {
        return format!("{dom_nodes} DOM-Knoten");
    }
    if let Some(load) = perf.vitals.load_time {
        return format!("Ladezeit {:.0} ms", load);
    }
    "Render-Pfad weiter optimieren".to_string()
}

fn derive_performance_recommendations(perf: &crate::audit::PerformanceResults) -> Vec<String> {
    let mut recommendations = Vec::new();

    if let Some(lcp) = &perf.vitals.lcp {
        if lcp.value > 2500.0 {
            recommendations.push(
                "Größtes sichtbares Element schneller laden: Hero-Bilder optimieren, priorisieren und kritische Styles früher ausliefern."
                    .to_string(),
            );
        }
    }

    if let Some(fcp) = &perf.vitals.fcp {
        if fcp.value > 1800.0 {
            recommendations.push(
                "Ersten sichtbaren Inhalt früher ausliefern: render-blockierende CSS- und JavaScript-Dateien reduzieren."
                    .to_string(),
            );
        }
    }

    if let Some(interactivity) = perf.vitals.inp.as_ref().or(perf.vitals.tbt.as_ref()) {
        if interactivity.value > 200.0 {
            recommendations.push(
                "Haupt-Thread entlasten: große JavaScript-Aufgaben aufteilen und nicht benötigte Skripte später laden."
                    .to_string(),
            );
        }
    }

    if let Some(cls) = &perf.vitals.cls {
        if cls.value > 0.1 {
            recommendations.push(
                "Layout-Verschiebungen vermeiden: Medien, Banner und dynamische Inhalte mit festen Platzhaltern reservieren."
                    .to_string(),
            );
        }
    }

    if let Some(dom_nodes) = perf.vitals.dom_nodes {
        if dom_nodes > 1200 {
            recommendations.push(
                "DOM-Struktur verschlanken: große Komponenten, tiefe Container-Hierarchien und wiederholte Markup-Blöcke reduzieren."
                    .to_string(),
            );
        }
    }

    if let Some(load_time) = perf.vitals.load_time {
        if load_time > 3000.0 {
            recommendations.push(
                "Gesamte Ladezeit senken: große Assets komprimieren, Caching schärfen und Drittanbieter-Skripte prüfen."
                    .to_string(),
            );
        }
    }

    if recommendations.is_empty() {
        recommendations.push(
            "Die Kernmetriken sind stabil. Nächster Hebel: Seitengröße und Drittanbieter-Skripte regelmäßig überwachen, damit das Niveau gehalten wird."
                .to_string(),
        );
    }

    recommendations.truncate(3);
    recommendations
}

fn derive_seo_lever(seo: &crate::seo::SeoAnalysis) -> String {
    if !seo.meta_issues.is_empty() {
        return format!(
            "Größter Hebel: Meta-Daten bereinigen ({} offene Punkte)",
            seo.meta_issues.len()
        );
    }
    if seo.social.completeness < 80 {
        return "Größter Hebel: Social-Meta-Daten vervollständigen".to_string();
    }
    "Größter Hebel: Struktur- und Inhalts-Signale weiter schärfen".to_string()
}

fn derive_seo_context(seo: &crate::seo::SeoAnalysis) -> String {
    let meta_issues = seo.meta_issues.len();
    let schema_count = seo.structured_data.json_ld.len();
    let h1 = seo.headings.h1_count;
    format!(
        "{} Meta-Probleme, {} H1, {} strukturierte Daten erkannt.",
        meta_issues, h1, schema_count
    )
}

fn derive_seo_card_context(seo: &crate::seo::SeoAnalysis) -> String {
    if !seo.meta_issues.is_empty() {
        return format!("{} Meta-Probleme offen", seo.meta_issues.len());
    }
    format!(
        "{} strukturierte Daten erkannt",
        seo.structured_data.json_ld.len()
    )
}

fn derive_security_lever(sec: &crate::security::SecurityAnalysis) -> String {
    let missing_headers = sec.headers.content_security_policy.is_none() as usize
        + sec.headers.strict_transport_security.is_none() as usize
        + sec.headers.permissions_policy.is_none() as usize
        + sec.headers.referrer_policy.is_none() as usize;
    if missing_headers > 0 {
        return format!(
            "Größter Hebel: fehlende Security-Header ergänzen ({missing_headers} Kernheader)"
        );
    }
    "Größter Hebel: Header-Regeln und TLS-Setup weiter härten".to_string()
}

fn derive_security_context(sec: &crate::security::SecurityAnalysis) -> String {
    let present_headers = [
        sec.headers.content_security_policy.is_some(),
        sec.headers.strict_transport_security.is_some(),
        sec.headers.x_content_type_options.is_some(),
        sec.headers.x_frame_options.is_some(),
        sec.headers.x_xss_protection.is_some(),
        sec.headers.referrer_policy.is_some(),
        sec.headers.permissions_policy.is_some(),
        sec.headers.cross_origin_opener_policy.is_some(),
        sec.headers.cross_origin_resource_policy.is_some(),
    ]
    .into_iter()
    .filter(|p| *p)
    .count();
    format!(
        "{} von 9 Kern-Headern vorhanden, HTTPS {}.",
        present_headers,
        if sec.ssl.https { "aktiv" } else { "fehlt" }
    )
}

fn derive_security_card_context(sec: &crate::security::SecurityAnalysis) -> String {
    let present_headers = [
        sec.headers.content_security_policy.is_some(),
        sec.headers.strict_transport_security.is_some(),
        sec.headers.x_content_type_options.is_some(),
        sec.headers.x_frame_options.is_some(),
        sec.headers.x_xss_protection.is_some(),
        sec.headers.referrer_policy.is_some(),
        sec.headers.permissions_policy.is_some(),
        sec.headers.cross_origin_opener_policy.is_some(),
        sec.headers.cross_origin_resource_policy.is_some(),
    ]
    .into_iter()
    .filter(|p| *p)
    .count();
    format!("{present_headers} von 9 Kern-Headern vorhanden")
}

fn build_tracking_summary_text(technical: &crate::seo::technical::TechnicalSeo) -> String {
    if technical.zaraz.detected {
        if technical.tracking_cookies.is_empty() && technical.tracking_signals.is_empty() {
            return "Zaraz ist auf der Seite erkennbar. Zusätzlich wurden im Lauf keine weiteren Tracking-Cookies oder externen Tracking-Signale festgestellt.".to_string();
        }
        return "Auf der Seite sind Tracking- oder Consent-nahe Signale erkennbar. Prüfen Sie insbesondere externe Einbindungen, Cookie-Setzung und den tatsächlichen Auslösezeitpunkt nach Einwilligung.".to_string();
    }

    if technical.uses_remote_google_fonts {
        return "Es werden extern gehostete Google Fonts geladen. Das ist datenschutz- und performance-relevant und sollte bewusst geprüft werden.".to_string();
    }

    if !technical.tracking_cookies.is_empty() || !technical.tracking_signals.is_empty() {
        return "Es wurden Tracking-Signale erkannt. Prüfen Sie Einwilligung, Auslösezeitpunkt und die Herkunft der eingebundenen Dienste.".to_string();
    }

    "Im aktuellen Lauf wurden keine externen Google Fonts, keine Tracking-Cookies und keine weiteren Tracking-Signale erkannt.".to_string()
}

fn derive_security_recommendations(sec: &crate::security::SecurityAnalysis) -> Vec<String> {
    let mut recommendations = Vec::new();

    if !sec.ssl.https {
        recommendations.push(
            "HTTPS durchgängig erzwingen und ein gültiges TLS-Zertifikat für alle Varianten der Domain sicherstellen."
                .to_string(),
        );
    }

    if sec.headers.content_security_policy.is_none() {
        recommendations.push(
            "Content-Security-Policy ergänzen und nur die tatsächlich benötigten Skript-, Style- und Medienquellen erlauben."
                .to_string(),
        );
    }

    if sec.headers.strict_transport_security.is_none() && sec.ssl.https {
        recommendations.push(
            "HSTS ergänzen, damit Browser die Seite dauerhaft nur noch per HTTPS laden."
                .to_string(),
        );
    }

    if sec.headers.cross_origin_opener_policy.is_none() {
        recommendations.push(
            "Cross-Origin-Opener-Policy prüfen und setzen, um die Isolation des Browser-Kontexts für moderne Webfunktionen zu stärken."
                .to_string(),
        );
    }

    if sec.headers.cross_origin_resource_policy.is_none() {
        recommendations.push(
            "Cross-Origin-Resource-Policy ergänzen, damit eingebundene Ressourcen nicht unnötig von fremden Origins mitgenutzt werden können."
                .to_string(),
        );
    }

    if sec.headers.permissions_policy.is_none() {
        recommendations.push(
            "Permissions-Policy definieren und nur die Browser-Funktionen freigeben, die auf der Seite wirklich benötigt werden."
                .to_string(),
        );
    }

    if sec.headers.referrer_policy.is_none() {
        recommendations.push(
            "Referrer-Policy setzen, damit bei Weiterleitungen und externen Aufrufen nicht mehr Informationen als nötig übergeben werden."
                .to_string(),
        );
    }

    if recommendations.is_empty() {
        recommendations.push(
            "Die grundlegenden Security-Header sind sauber gesetzt. Nächster Schritt: Richtlinien regelmäßig prüfen und an neue Skript- oder Integrationsquellen anpassen."
                .to_string(),
        );
    }

    recommendations.truncate(4);
    recommendations
}

fn derive_mobile_lever(mobile: &crate::mobile::MobileFriendliness) -> String {
    if mobile.touch_targets.small_targets > 0 {
        return format!(
            "Größter Hebel: Touch Targets vergrößern ({} zu klein)",
            mobile.touch_targets.small_targets
        );
    }
    if mobile.touch_targets.crowded_targets > 0 {
        return format!(
            "Größter Hebel: Abstände mobiler Bedienelemente erhöhen ({})",
            mobile.touch_targets.crowded_targets
        );
    }
    "Größter Hebel: mobile Lesbarkeit und Touch-Flows weiter optimieren".to_string()
}

fn derive_mobile_context(mobile: &crate::mobile::MobileFriendliness) -> String {
    format!(
        "Viewport {}, {} zu kleine Touch Targets, {} zu enge Abstände.",
        if mobile.viewport.is_properly_configured {
            "korrekt gesetzt"
        } else {
            "nicht sauber konfiguriert"
        },
        mobile.touch_targets.small_targets,
        mobile.touch_targets.crowded_targets
    )
}

fn derive_mobile_card_context(mobile: &crate::mobile::MobileFriendliness) -> String {
    if mobile.touch_targets.small_targets > 0 {
        format!(
            "{} zu kleine Touch Targets",
            mobile.touch_targets.small_targets
        )
    } else if mobile.touch_targets.crowded_targets > 0 {
        format!("{} zu enge Abstände", mobile.touch_targets.crowded_targets)
    } else if mobile.viewport.is_properly_configured {
        "Viewport korrekt gesetzt".to_string()
    } else {
        "Viewport prüfen".to_string()
    }
}

fn derive_business_impact(
    user_impact: &str,
    dimension: &str,
    severity: Severity,
    subcategory: Option<&str>,
) -> String {
    match dimension {
        "SEO" => "Kann Sichtbarkeit in Suchmaschinen reduzieren und organischen Traffic senken."
            .to_string(),
        "Security" => "Erhöht Angriffsfläche und Risiko für Datenverlust.".to_string(),
        "Performance" => {
            "Verschlechtert Ladezeit und Nutzererlebnis, erhöht Absprungrate.".to_string()
        }
        "Mobile" => "Beeinträchtigt mobile Nutzbarkeit für die Mehrheit der Nutzer.".to_string(),
        "Accessibility" => {
            // Differentiate by subcategory or content of user_impact
            if subcategory == Some("Visuelle Darstellung")
                || user_impact.contains("Kontrast")
                || user_impact.contains("Lesbarkeit")
            {
                "Beeinträchtigt Lesbarkeit für Nutzer mit Sehschwäche.".to_string()
            } else {
                match severity {
                    Severity::Critical | Severity::High => {
                        "Kann Nutzer ausschließen und rechtliches Risiko erhöhen.".to_string()
                    }
                    _ if user_impact.contains("Sprachsteuerung") => {
                        "Kann Nutzungshürden erhöhen und Interaktionen mit zentralen Elementen verhindern."
                            .to_string()
                    }
                    _ => "Beeinträchtigt Qualität und Nutzererlebnis der Website.".to_string(),
                }
            }
        }
        _ => match severity {
            Severity::Critical | Severity::High => {
                "Kann Nutzer ausschließen und rechtliches Risiko erhöhen.".to_string()
            }
            _ if user_impact.contains("Sprachsteuerung") => {
                "Kann Nutzungshürden erhöhen und Interaktionen mit zentralen Elementen verhindern."
                    .to_string()
            }
            _ => "Beeinträchtigt Qualität und Nutzererlebnis der Website.".to_string(),
        },
    }
}

fn derive_execution_priority(
    severity: Severity,
    effort: Effort,
    dimension: &str,
) -> ExecutionPriority {
    match (severity, effort, dimension) {
        (Severity::Critical, _, _) => ExecutionPriority::Immediate,
        (Severity::High, _, "Accessibility") => ExecutionPriority::Immediate,
        (Severity::High, Effort::Quick, _) => ExecutionPriority::Important,
        (Severity::High, _, _) => ExecutionPriority::Important,
        (Severity::Medium, Effort::Quick, _) => ExecutionPriority::Important,
        _ => ExecutionPriority::Optional,
    }
}

fn average_page_semantic_score(classification: &crate::seo::profile::PageClassification) -> u32 {
    let total = classification.content_depth_score
        + classification.structural_richness_score
        + classification.media_text_balance_score
        + classification.intent_fit_score;
    total / 4
}

fn summarize_page_profile(profile: &crate::seo::profile::SeoContentProfile) -> String {
    let classification = &profile.page_classification;
    let avg = average_page_semantic_score(classification);
    let quality = match avg {
        85..=100 => "sehr stimmig aufgebaut",
        70..=84 => "inhaltlich solide aufgestellt",
        50..=69 => "nur teilweise klar strukturiert",
        _ => "aktuell inhaltlich und strukturell schwach ausgeprägt",
    };

    let mut traits = classification.attributes.clone();
    if traits.is_empty() {
        traits.push("ohne klare Zusatzmerkmale".to_string());
    }

    format!(
        "Die Seite wirkt wie „{}“ und ist {}. Auffällig sind {}.",
        classification.primary_type.label(),
        quality,
        traits.join(", ")
    )
}

fn page_profile_optimization_note(profile: &crate::seo::profile::SeoContentProfile) -> String {
    let classification = &profile.page_classification;
    if classification.content_depth_score < 45 {
        return "Mehr inhaltliche Tiefe und klar gegliederte Abschnitte würden den Nutzwert erhöhen."
            .to_string();
    }
    if classification.structural_richness_score < 55 {
        return "Mehr Zwischenüberschriften und eine klarere Inhaltsstruktur würden die Seite besser scannbar machen."
            .to_string();
    }
    if classification.media_text_balance_score < 55 {
        return "Die Seite wirkt stark visuell. Mehr erklärender Text und klarer Kontext würden Nutzen und Orientierung verbessern."
            .to_string();
    }
    if classification.intent_fit_score < 65 {
        return "Die Seite bedient ihren Seitentyp noch nicht sauber; Aufbau und Inhalte sollten stärker auf das eigentliche Nutzerziel einzahlen."
            .to_string();
    }
    "Die Seite passt insgesamt gut zu ihrem Seitentyp. Der größte Hebel liegt in weiterer inhaltlicher Schärfung statt in Grundsatzumbauten."
        .to_string()
}

fn extract_page_topics(report: &crate::audit::AuditReport) -> Vec<String> {
    let mut weighted_segments: Vec<(String, usize)> = Vec::new();
    if let Some(ref seo) = report.seo {
        if let Some(ref title) = seo.meta.title {
            weighted_segments.push((title.clone(), 4));
        }
        if let Some(ref description) = seo.meta.description {
            weighted_segments.push((description.clone(), 2));
        }
        for heading in &seo.headings.headings {
            weighted_segments.push((heading.text.clone(), if heading.level <= 2 { 3 } else { 2 }));
        }
        weighted_segments.push((seo.technical.text_excerpt.clone(), 1));
    }

    top_terms_from_segments(&weighted_segments, 5)
}

fn derive_domain_topics(url_details: &[CompactUrlSummary]) -> Vec<(String, usize)> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for detail in url_details {
        for term in &detail.topic_terms {
            *counts.entry(term.clone()).or_default() += 1;
        }
    }

    let mut topics: Vec<(String, usize)> = counts.into_iter().collect();
    topics.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    topics.into_iter().take(8).collect()
}

fn derive_topic_overlap_pairs(url_details: &[CompactUrlSummary]) -> Vec<(String, String, u32)> {
    let mut pairs = Vec::new();
    for (idx, left) in url_details.iter().enumerate() {
        let left_terms: HashSet<&str> = left.topic_terms.iter().map(String::as_str).collect();
        if left_terms.len() < 3 {
            continue;
        }

        for right in url_details.iter().skip(idx + 1) {
            let right_terms: HashSet<&str> = right.topic_terms.iter().map(String::as_str).collect();
            if right_terms.len() < 3 {
                continue;
            }

            let intersection = left_terms.intersection(&right_terms).count();
            if intersection < 2 {
                continue;
            }

            let overlap_ratio =
                intersection as f64 / left_terms.len().min(right_terms.len()) as f64;
            let union = left_terms.union(&right_terms).count();
            let jaccard = intersection as f64 / union as f64;
            let similarity = ((jaccard * 0.55 + overlap_ratio * 0.45) * 100.0).round() as u32;
            if similarity >= 45 {
                pairs.push((left.url.clone(), right.url.clone(), similarity));
            }
        }
    }

    pairs.sort_by(|a, b| {
        b.2.cmp(&a.2)
            .then_with(|| a.0.cmp(&b.0))
            .then_with(|| a.1.cmp(&b.1))
    });
    pairs.into_iter().take(6).collect()
}

fn top_terms_from_segments(segments: &[(String, usize)], limit: usize) -> Vec<String> {
    let stopwords = german_stopwords();
    let mut counts: HashMap<String, usize> = HashMap::new();

    for (segment, weight) in segments {
        for token in segment
            .split(|c: char| !c.is_alphanumeric() && c != '-' && c != '_')
            .filter(|token| !token.is_empty())
        {
            let normalized = normalize_topic_token(token);
            if normalized.len() < 4
                || normalized.chars().all(|ch| ch.is_ascii_digit())
                || stopwords.contains(normalized.as_str())
            {
                continue;
            }
            *counts.entry(normalized).or_default() += *weight;
        }
    }

    let mut terms: Vec<(String, usize)> = counts.into_iter().collect();
    terms.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    terms
        .into_iter()
        .take(limit)
        .map(|(term, _)| term)
        .collect()
}

fn normalize_topic_token(token: &str) -> String {
    token
        .trim_matches(|c: char| !c.is_alphanumeric())
        .to_lowercase()
        .replace("ä", "ae")
        .replace("ö", "oe")
        .replace("ü", "ue")
        .replace("ß", "ss")
}

fn german_stopwords() -> HashSet<&'static str> {
    [
        "2026",
        "aber",
        "allem",
        "alle",
        "auch",
        "auf",
        "aus",
        "autor",
        "bei",
        "bereits",
        "bietet",
        "bild",
        "bilder",
        "casoon",
        "checker",
        "cloud",
        "content",
        "damit",
        "dass",
        "deine",
        "diese",
        "dieser",
        "drei",
        "durch",
        "eine",
        "einem",
        "einen",
        "einer",
        "eines",
        "einfach",
        "entwickelt",
        "entwicklung",
        "erfahren",
        "fuer",
        "für",
        "gmbh",
        "heute",
        "hier",
        "ihre",
        "ihren",
        "ihrer",
        "ihres",
        "inklusive",
        "inhalt",
        "jetzt",
        "keine",
        "kunden",
        "launch",
        "lesen",
        "mehr",
        "moderne",
        "klare",
        "oder",
        "page",
        "pages",
        "projekt",
        "projekten",
        "recht",
        "rund",
        "seite",
        "seiten",
        "seine",
        "seiner",
        "sich",
        "sind",
        "site",
        "statt",
        "systeme",
        "technik",
        "themen",
        "thema",
        "über",
        "und",
        "unsere",
        "unserer",
        "unsers",
        "unter",
        "transparent",
        "viele",
        "vom",
        "von",
        "web",
        "websites",
        "webentwicklung",
        "website",
        "weiter",
        "werden",
        "wird",
        "wenig",
        "willkommen",
        "zeigen",
        "ziel",
        "with",
        "your",
        "about",
        "into",
        "that",
        "this",
        "from",
        "haben",
        "sowie",
        "digitale",
    ]
    .into_iter()
    .collect()
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
        "Befriedigend" => format!("{} — die {} weist einzelne Schwächen auf.", grade, area),
        "Ausbaufähig" => format!("{} — die {} weist relevante Schwächen auf.", grade, area),
        _ => format!(
            "{} — die {} hat erhebliche Mängel, die behoben werden sollten.",
            grade, area
        ),
    }
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

fn build_batch_appendix(batch: &BatchReport) -> BatchAppendixData {
    BatchAppendixData {
        per_url: batch
            .reports
            .iter()
            .map(|r| {
                // Aggregate violations by rule for each URL
                let mut rule_map: std::collections::HashMap<String, AppendixViolation> =
                    std::collections::HashMap::new();
                let mut rule_order: Vec<String> = Vec::new();

                for v in &r.wcag_results.violations {
                    let element = AffectedElement {
                        selector: v.selector.clone().unwrap_or_else(|| v.node_id.clone()),
                        node_id: v.node_id.clone(),
                    };

                    if let Some(existing) = rule_map.get_mut(&v.rule) {
                        existing.affected_elements.push(element);
                    } else {
                        rule_order.push(v.rule.clone());
                        rule_map.insert(
                            v.rule.clone(),
                            AppendixViolation {
                                rule: v.rule.clone(),
                                rule_name: v.rule_name.clone(),
                                severity: v.severity,
                                message: v.message.clone(),
                                fix_suggestion: v.fix_suggestion.clone(),
                                affected_elements: vec![element],
                            },
                        );
                    }
                }

                UrlAppendix {
                    url: r.url.clone(),
                    violations: rule_order
                        .into_iter()
                        .filter_map(|rule| rule_map.remove(&rule))
                        .collect(),
                }
            })
            .collect(),
    }
}

fn yes_no(val: bool) -> String {
    if val {
        "Ja".to_string()
    } else {
        "Nein".to_string()
    }
}

fn truncate_list(items: &[String], limit: usize) -> String {
    let mut values: Vec<String> = items
        .iter()
        .filter(|item| !item.trim().is_empty())
        .cloned()
        .collect();
    values.sort();
    values.dedup();

    let shown: Vec<String> = values.iter().take(limit).cloned().collect();
    if values.len() > limit {
        format!("{} +{}", shown.join(", "), values.len() - limit)
    } else {
        shown.join(", ")
    }
}

fn truncate_url_list(items: &[String], limit: usize, max_len: usize) -> String {
    let shortened: Vec<String> = items
        .iter()
        .map(|item| truncate_url(item, max_len))
        .collect();
    truncate_list(&shortened, limit)
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

/// Extract the bare domain from a URL (strips scheme, www, trailing slash).
fn extract_domain(url: &str) -> String {
    let without_scheme = url
        .trim_start_matches("https://")
        .trim_start_matches("http://");
    let host = without_scheme.split('/').next().unwrap_or(without_scheme);
    host.trim_start_matches("www.").to_string()
}
