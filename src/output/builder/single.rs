//! Single-report ViewModel builder.

use crate::audit::normalized::NormalizedReport;
use crate::audit::summary::analyze_with_locale;
use crate::cli::ReportLevel;
use crate::i18n::I18n;
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
    build_tracking_summary_text, build_vitals_list, derive_accessibility_card_context,
    derive_accessibility_context, derive_accessibility_lever, derive_mobile_card_context,
    derive_mobile_context, derive_mobile_lever, derive_performance_card_context,
    derive_performance_context, derive_performance_lever, derive_performance_recommendations,
    derive_security_card_context, derive_security_context, derive_security_lever,
    derive_security_recommendations, derive_seo_card_context, derive_seo_context, derive_seo_lever,
};
use super::seo::{
    build_seo_interpretation, page_profile_optimization_note, summarize_page_profile,
};

const NBSP: &str = "\u{00A0}";

/// Build a complete ViewModel from a normalized report (single source of truth for score/grade/certificate)
pub fn build_view_model(normalized: &NormalizedReport, config: &ReportConfig) -> ReportViewModel {
    let i18n = I18n::new(&config.locale)
        .or_else(|_| I18n::new("de"))
        .expect("default locale must always load");
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
        .map(|f| finding_group_from_normalized(&config.locale, f))
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
    let audit_summary = analyze_with_locale(normalized, &config.locale);
    let maturity_label = audit_summary.site_state.label_localized(&i18n);
    let problem_type = audit_summary.problem_type_label.clone();
    let mut technical_overview = build_technical_overview(&config.locale, normalized);
    for cross in &audit_summary.cross_impacts {
        technical_overview.push(format!(
            "Cross-Impact {}: {}",
            cross.dimensions, cross.description
        ));
    }
    let overall_impact = build_overall_impact(&config.locale, normalized);
    let date = if config.locale == "en" {
        normalized.timestamp.format("%Y-%m-%d").to_string()
    } else {
        normalized.timestamp.format("%d.%m.%Y").to_string()
    };
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
    let positive_aspects = derive_positive_aspects_from_normalized(&config.locale, normalized);
    let action_plan = derive_action_plan(&config.locale, &sorted_groups);

    let mut module_names: Vec<String> = vec!["Accessibility".into()];
    if normalized.raw_performance.is_some() {
        module_names.push("Performance".into());
    }
    if normalized.raw_seo.is_some() {
        module_names.push("SEO".into());
    }
    if normalized.raw_security.is_some() {
        module_names.push(if config.locale == "en" {
            "Security".into()
        } else {
            "Sicherheit".into()
        });
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

    let modules = build_modules_block_from_normalized(&config.locale, normalized);

    let quick_win_count = action_plan.quick_wins.len();
    let critical_count =
        (normalized.severity_counts.critical + normalized.severity_counts.high) as u32;
    let total_violations = normalized.severity_counts.total as u32;
    let nodes_analyzed = normalized.nodes_analyzed;
    let warning_count = normalized.raw_wcag.warnings.len() as u32;
    let not_testable_count = normalized.raw_wcag.not_testables.len() as u32;

    let actions = build_actions_block(
        &config.locale,
        &action_plan,
        score as f32,
        &audit_summary.site_state,
    );

    let module_details = build_module_details_from_normalized(&config.locale, normalized);
    let history = config
        .history_preview
        .as_ref()
        .map(|preview| build_history_trend_block(&config.locale, preview));
    let executive = build_executive_narrative(
        &i18n,
        normalized,
        &audit_summary,
        score,
        &severity,
        &top_findings,
        &action_plan,
    );

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
            desktop_score: normalized
                .viewport_scores
                .as_ref()
                .map(|vs| vs.desktop.accessibility),
            mobile_score: normalized
                .viewport_scores
                .as_ref()
                .map(|vs| vs.mobile.accessibility),
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
            verdict: build_verdict_text(&i18n, &normalized.url, score as f32),
            score_note: build_score_note(&i18n, normalized),
            metrics: {
                let en = config.locale == "en";
                let label_violations_total = if en {
                    format!("Total{NBSP}violations")
                } else {
                    format!("Verstöße{NBSP}gesamt")
                };
                let label_critical = if en {
                    "Critical".to_string()
                } else {
                    "Kritisch".to_string()
                };
                let label_overall = if en {
                    format!("Site{NBSP}overall{NBSP}score")
                } else {
                    format!("Gesamtscore{NBSP}Website")
                };
                let label_checked_nodes = if en {
                    format!("Checked{NBSP}nodes")
                } else {
                    format!("Geprüfte{NBSP}Knoten")
                };
                let label_quick_wins: String = "Quick Wins".into();
                let label_wcag_level = if en {
                    "WCAG level".to_string()
                } else {
                    "WCAG-Level".to_string()
                };
                let label_warnings = if en {
                    format!("Heuristic{NBSP}warnings")
                } else {
                    format!("Heuristische{NBSP}Warnungen")
                };
                let label_not_testable = if en {
                    format!("Manual{NBSP}testing{NBSP}required")
                } else {
                    format!("Manuell{NBSP}zu{NBSP}prüfen")
                };
                vec![
                    MetricItem {
                        title: label_violations_total,
                        value: total_violations.to_string(),
                        accent_color: Some("#f59e0b".into()),
                    },
                    MetricItem {
                        title: label_critical,
                        value: critical_count.to_string(),
                        accent_color: Some("#ef4444".into()),
                    },
                    MetricItem {
                        title: if has_quality_modules {
                            label_overall.clone()
                        } else {
                            label_checked_nodes.clone()
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
                            label_checked_nodes
                        } else {
                            label_quick_wins.clone()
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
                            label_quick_wins
                        } else {
                            label_wcag_level
                        },
                        value: if has_quality_modules {
                            quick_win_count.to_string()
                        } else {
                            normalized.wcag_level.to_string()
                        },
                        accent_color: Some("#7c3aed".into()),
                    },
                ]
                .into_iter()
                .chain(if warning_count > 0 {
                    Some(MetricItem {
                        title: label_warnings,
                        value: warning_count.to_string(),
                        accent_color: Some("#f97316".into()),
                    })
                } else {
                    None
                })
                .chain(if not_testable_count > 0 {
                    Some(MetricItem {
                        title: label_not_testable,
                        value: not_testable_count.to_string(),
                        accent_color: Some("#6b7280".into()),
                    })
                } else {
                    None
                })
                .collect()
            },
            top_actions: top_findings
                .iter()
                .take(3)
                .map(|f| humanize_action_text(&config.locale, &f.recommendation))
                .collect(),
            positive_aspects: positive_aspects
                .iter()
                .map(|a| format!("{}: {}", a.area, a.description))
                .collect(),
            overall_impact,
            technical_overview,
            benchmark_context: build_benchmark_context(&config.locale, score as f32),
            business_consequence: build_business_consequence(&i18n, normalized),
            consequence: build_consequence_text(&i18n, normalized),
            risk_level: normalized.risk.level.label_localized(&i18n),
            risk_summary: normalized.risk.summary_for(&config.locale),
        },
        executive,
        history,
        methodology: build_methodology(&config.locale, normalized),
        modules,
        severity,
        findings: {
            let clusters = build_thematic_clusters(&config.locale, &sorted_groups);
            let finding_summary = build_finding_summary(&config.locale, normalized, &audit_summary);
            FindingsBlock {
                summary: finding_summary,
                clusters,
                top_findings,
                all_findings: sorted_groups,
            }
        },
        diagnosis: build_diagnosis_block(&config.locale, normalized, &audit_summary),
        module_details,
        actions,
        appendix: build_appendix_block_from_normalized(normalized),
        positive_signals: build_positive_signals(&config.locale, normalized),
    }
}

/// Map recognized patterns from NormalizedReport into the ViewModel block.
/// Translates pattern names into localized titles; the message stays as the
/// description (already human-readable from the detector).
fn build_positive_signals(
    locale: &str,
    normalized: &NormalizedReport,
) -> crate::output::report_model::PositiveSignalsBlock {
    use crate::output::report_model::{PositiveSignal, PositiveSignalsBlock};

    let en = locale == "en";
    let mut items: Vec<PositiveSignal> = normalized
        .raw_patterns
        .as_ref()
        .map(|p| {
            p.recognized
                .iter()
                .map(|r| {
                    let title = match (en, r.pattern.as_str()) {
                        (true, "MainNavigation") => "Semantic main navigation",
                        (false, "MainNavigation") => "Semantische Hauptnavigation",
                        (true, "DisclosureMenu") => "Disclosure menu",
                        (false, "DisclosureMenu") => "Disclosure-Menü",
                        (true, "ModalDialog") => "Modal dialog",
                        (false, "ModalDialog") => "Modaler Dialog",
                        (true, "TabList") => "Tab list",
                        (false, "TabList") => "Tab-Liste",
                        (true, "SkipLink") => "Skip link",
                        (false, "SkipLink") => "Skip-Link",
                        (true, "Accordion") => "Accordion",
                        (false, "Accordion") => "Accordion",
                        (_, other) => other,
                    };
                    PositiveSignal {
                        title: title.to_string(),
                        description: r.message.clone(),
                        strong: matches!(r.confidence, crate::patterns::PatternConfidence::Strong),
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    // Append WCAG-rule-level positive signals (from FindingKind::Positive findings).
    for pos in &normalized.raw_wcag.positives {
        let title = if en { "WCAG signal" } else { "WCAG-Signal" };
        items.push(PositiveSignal {
            title: title.to_string(),
            description: pos.message.clone(),
            strong: false,
        });
    }

    PositiveSignalsBlock { items }
}

fn build_executive_narrative(
    i18n: &I18n,
    normalized: &NormalizedReport,
    audit_summary: &crate::audit::summary::AuditSummary,
    score: u32,
    severity: &SeverityBlock,
    top_findings: &[FindingGroup],
    action_plan: &ActionPlan,
) -> ExecutiveNarrativeBlock {
    let en = i18n.locale() == "en";
    let assessment = build_single_assessment_text(i18n.locale(), score, severity);
    let risk_action = match (normalized.risk.level, en) {
        (crate::audit::normalized::RiskLevel::Critical, true) => "Act immediately",
        (crate::audit::normalized::RiskLevel::Critical, false) => "Sofort handeln",
        (crate::audit::normalized::RiskLevel::High, true) => "Fix soon",
        (crate::audit::normalized::RiskLevel::High, false) => "Zeitnah beheben",
        (crate::audit::normalized::RiskLevel::Medium, true) => "Address with next optimization",
        (crate::audit::normalized::RiskLevel::Medium, false) => "Bei nächster Optimierung",
        (crate::audit::normalized::RiskLevel::Low, true) => "Optimization recommended",
        (crate::audit::normalized::RiskLevel::Low, false) => "Optimierung empfohlen",
    };

    let key_points =
        build_single_key_points_text(i18n.locale(), severity, top_findings, normalized);
    let (user_label, business_label, risk_label) = if en {
        ("User", "Business", "Risk")
    } else {
        ("Nutzer", "Business", "Risiko")
    };
    let impact_rows = vec![
        (
            user_label.to_string(),
            // Use the top finding's concrete user_impact for specificity.
            // Falls back to the generic rating if no finding or impact text is too short.
            top_findings
                .first()
                .filter(|f| f.user_impact.len() > 20)
                .map(|f| sentence_preview(&f.user_impact).to_string())
                .unwrap_or_else(|| {
                    build_overall_impact(i18n.locale(), normalized)
                        .into_iter()
                        .next()
                        .map(|(_, v)| v)
                        .unwrap_or_else(|| {
                            if en {
                                "Some users cannot use the content and functions.".to_string()
                            } else {
                                "Ein Teil der Nutzer kann Inhalte und Funktionen nicht nutzen."
                                    .to_string()
                            }
                        })
                }),
        ),
        (business_label.to_string(), {
            let consequence = build_business_consequence(i18n, normalized);
            if consequence.is_empty() {
                if en {
                    "Users abandon processes or fail to reach their goals.".to_string()
                } else {
                    "Nutzer brechen Prozesse ab oder erreichen Ziele nicht.".to_string()
                }
            } else {
                consequence
            }
        }),
        (
            risk_label.to_string(),
            if severity.critical > 0 {
                if en {
                    format!(
                        "Automated checks detected {} critical WCAG Level A violations — \
                         potentially relevant for BFSG/EAA. Additional manual review is required \
                         for a defensible legal classification.",
                        severity.critical
                    )
                } else {
                    format!(
                        "Automatisiert wurden {} kritische WCAG-Level-A-Verstöße erkannt — \
                         potenziell relevant für BFSG/EAA. Für eine belastbare rechtliche \
                         Einordnung ist ergänzend manuelle Prüfung nötig.",
                        severity.critical
                    )
                }
            } else if severity.high > 0 {
                if en {
                    "Automated checks found no critical Level A violations, but WCAG AA gaps \
                     exist. Additional manual review is required for a defensible BFSG/WCAG \
                     classification."
                        .to_string()
                } else {
                    "Automatisiert keine kritischen Level-A-Verstöße erkannt, aber WCAG-AA-Mängel \
                     vorhanden. Für eine belastbare BFSG-/WCAG-Einordnung ist ergänzend manuelle \
                     Prüfung nötig."
                        .to_string()
                }
            } else if en {
                "Automated checks found no critical violations. Additional manual review is \
                 required for a defensible BFSG/WCAG classification."
                    .to_string()
            } else {
                "Automatisiert wurden keine kritischen Verstöße erkannt. Für eine belastbare \
                 BFSG-/WCAG-Einordnung ist ergänzend manuelle Prüfung nötig."
                    .to_string()
            },
        ),
    ];

    let quick_actions = build_single_quick_actions_text(i18n.locale(), action_plan, top_findings);

    let total_ch = (severity.critical + severity.high) as usize;
    let (spotlight_body, spotlight_impact, spotlight_recommendation, leverage_text) = if let Some(
        top,
    ) =
        top_findings.first()
    {
        let share = (top.occurrence_count * 100)
            .checked_div(total_ch)
            .unwrap_or(0);
        (
            audit_summary.dominant_issue_note.clone().unwrap_or_else(|| {
                if en {
                    "The majority of critical problems originate from this single topic.".to_string()
                } else {
                    "Der Großteil der kritischen Probleme entsteht durch dieses eine Thema."
                        .to_string()
                }
            }),
            sentence_preview(&top.user_impact).to_string(),
            sentence_preview(&top.recommendation).to_string(),
            (total_ch > 0).then(|| {
                if en {
                    format!(
                        "Fixing the main issue removes about {}% of critical errors. Immediately tangible improvement in usability.",
                        share.min(99)
                    )
                } else {
                    format!(
                        "Behebung des Hauptproblems reduziert ca. {}% der kritischen Fehler. Sofort spürbare Verbesserung der Nutzbarkeit.",
                        share.min(99)
                    )
                }
            }),
        )
    } else if en {
        (
            "No single issue dominates the audit picture; findings are distributed more broadly."
                .to_string(),
            "The impact is spread across several smaller barriers.".to_string(),
            "Actions should be bundled and prioritized by impact.".to_string(),
            None,
        )
    } else {
        (
            "Kein einzelnes Problem dominiert das Auditbild; die Befunde sind breiter verteilt."
                .to_string(),
            "Die Wirkung verteilt sich auf mehrere kleinere Barrieren.".to_string(),
            "Die Maßnahmen sollten gebündelt und nach Hebel priorisiert umgesetzt werden."
                .to_string(),
            None,
        )
    };

    let findings_intro = if score >= 85 && top_findings.len() <= 2 {
        i18n.t("narrative-findings-intro-solid")
    } else {
        i18n.t("narrative-findings-intro-default")
    };

    ExecutiveNarrativeBlock {
        cover_eyebrow: i18n.t("narrative-cover-eyebrow"),
        cover_kicker: i18n.t("narrative-cover-kicker"),
        status_title: i18n.t("narrative-status-title"),
        risk_title: format!("{assessment}  —  {risk_action}"),
        metrics_title: i18n.t("narrative-metrics-title"),
        key_points_title: i18n.t("narrative-key-points-title"),
        key_points,
        impact_title: i18n.t("narrative-impact-title"),
        impact_rows,
        quick_actions_title: i18n.t("narrative-quick-actions-title"),
        quick_actions,
        spotlight_eyebrow: i18n.t("narrative-spotlight-eyebrow"),
        spotlight_body,
        spotlight_impact,
        spotlight_recommendation,
        leverage_title: i18n.t("narrative-leverage-title"),
        leverage_text,
        findings_title: i18n.t("narrative-findings-title"),
        findings_intro,
        action_plan_title: i18n.t("narrative-action-plan-title"),
        action_plan_intro: i18n.t("narrative-action-plan-intro"),
        action_plan_callout_title: i18n.t("narrative-action-plan-callout-title"),
        action_plan_callout_body: i18n.t("narrative-action-plan-callout-body"),
        technical_title: i18n.t("narrative-technical-title"),
        technical_intro: i18n.t("narrative-technical-intro"),
        next_steps_title: i18n.t("narrative-next-steps-title"),
        next_steps_intro: i18n.t("narrative-next-steps-intro"),
        next_steps_callout_title: i18n.t("narrative-next-steps-callout-title"),
        next_steps_callout_body: i18n.t("narrative-next-steps-callout-body"),
    }
}

fn build_single_assessment_text(locale: &str, score: u32, severity: &SeverityBlock) -> String {
    let en = locale == "en";
    let has_critical_a11y = severity.critical > 0;
    let has_high = severity.high > 0;

    if has_critical_a11y && score < 50 {
        if en {
            "Critical barriers — not WCAG conformant".to_string()
        } else {
            "Kritische Barrieren — nicht WCAG-konform".to_string()
        }
    } else if has_critical_a11y {
        if en {
            "Technically solid, but legally risky".to_string()
        } else {
            "Technisch solide, aber rechtlich riskant".to_string()
        }
    } else if has_high {
        if en {
            "Good foundation, but not accessible".to_string()
        } else {
            "Gute Basis, aber nicht barrierefrei".to_string()
        }
    } else if score >= 85 {
        if en {
            "Largely accessible — polish".to_string()
        } else {
            "Weitgehend barrierefrei — Feinschliff".to_string()
        }
    } else if en {
        "Solid foundation with room to optimize".to_string()
    } else {
        "Solide Grundlage mit Optimierungspotenzial".to_string()
    }
}

fn build_single_key_points_text(
    locale: &str,
    severity: &SeverityBlock,
    top_findings: &[FindingGroup],
    normalized: &NormalizedReport,
) -> Vec<String> {
    let en = locale == "en";
    let mut points = Vec::with_capacity(3);
    let ch = severity.critical + severity.high;
    if ch > 0 {
        if en {
            points.push(format!("{} critical/high WCAG violations on this page", ch));
        } else {
            points.push(format!(
                "{} kritische/hohe WCAG-Verstöße auf dieser Seite",
                ch
            ));
        }
    }

    if let Some(top) = top_findings.first() {
        let total_ch = (severity.critical + severity.high) as usize;
        let share = (top.occurrence_count * 100)
            .checked_div(total_ch)
            .unwrap_or(0);
        if share >= 30 {
            if en {
                points.push(format!(
                    "Main issue: {} ({}% of all critical errors)",
                    top.title,
                    share.min(99)
                ));
            } else {
                points.push(format!(
                    "Hauptproblem: {} ({}% aller kritischen Fehler)",
                    top.title,
                    share.min(99)
                ));
            }
        } else if en {
            points.push(format!("Most frequent issue: {}", top.title));
        } else {
            points.push(format!("Häufigstes Problem: {}", top.title));
        }
    }

    if severity.critical > 0 {
        if en {
            points.push(
                "WCAG Level A violations detected automatically — manual review needed for a defensible BFSG classification".to_string(),
            );
        } else {
            points.push(
                "WCAG-Level-A-Verstöße automatisiert erkannt — manuelle Prüfung für belastbare BFSG-Einordnung nötig".to_string(),
            );
        }
    } else if severity.high > 0 {
        if en {
            points.push("No Level A violations, but structural weaknesses".to_string());
        } else {
            points.push("Keine Level-A-Verstöße, aber strukturelle Schwächen".to_string());
        }
    } else if !normalized.audit_flags.is_empty() {
        if en {
            points.push(
                "Audit notes present — individual signals should be verified manually.".to_string(),
            );
        } else {
            points.push(
                "Audit-Hinweise vorhanden — einzelne Signale sollten fachlich gegengeprüft werden."
                    .to_string(),
            );
        }
    } else if en {
        points.push(
            "No automatically detectable critical barriers — manual review recommended."
                .to_string(),
        );
    } else {
        points.push(
            "Keine automatisiert erkennbaren kritischen Barrieren — manuelle Prüfung empfohlen."
                .to_string(),
        );
    }

    points
}

fn build_single_quick_actions_text(
    locale: &str,
    action_plan: &ActionPlan,
    top_findings: &[FindingGroup],
) -> Vec<(String, String)> {
    let en = locale == "en";
    let timeframe_label = |effort: Effort| -> &'static str {
        match (effort, en) {
            (Effort::Quick, true) => "1–2 days",
            (Effort::Quick, false) => "1–2 Tage",
            (Effort::Medium, true) => "3–5 days",
            (Effort::Medium, false) => "3–5 Tage",
            (Effort::Structural, true) => "1–2 weeks",
            (Effort::Structural, false) => "1–2 Wochen",
        }
    };

    let mut actions = Vec::new();

    for item in &action_plan.quick_wins {
        actions.push((
            item.action.clone(),
            timeframe_label(item.effort).to_string(),
        ));
    }

    if actions.is_empty() {
        for group in top_findings.iter().take(3) {
            actions.push((
                sentence_preview(&group.recommendation).to_string(),
                timeframe_label(group.effort).to_string(),
            ));
        }
    }

    actions.truncate(3);
    actions
}

fn sentence_preview(text: &str) -> &str {
    let mut search_from = 0;
    while let Some(rel) = text[search_from..].find(". ") {
        let pos = search_from + rel;
        // Skip single-letter abbreviations like "z. B.", "d. h.", "u. a."
        if pos >= 2 {
            let before = &text[pos - 2..pos];
            if before.starts_with(' ') && before.as_bytes()[1].is_ascii_alphabetic() {
                search_from = pos + 2;
                continue;
            }
        }
        return text[..pos + 1].trim_end();
    }
    text
}

fn build_history_trend_block(locale: &str, preview: &ReportHistoryPreview) -> HistoryTrendBlock {
    let trend_label = build_trend_label(
        locale,
        preview.delta_accessibility,
        preview.delta_total_issues,
    );
    let en = locale == "en";

    let trend_interpretation = if en {
        match trend_label.as_str() {
            "Significantly improved" => format!(
                "Accessibility has improved significantly versus the run on {} (+{} points, {} fewer issues).",
                preview.previous_date,
                preview.delta_accessibility,
                -preview.delta_total_issues
            ),
            "Improved" => format!(
                "Accessibility has improved versus the run on {}.",
                preview.previous_date
            ),
            "Stable" => format!(
                "Accessibility is unchanged compared with the run on {}.",
                preview.previous_date
            ),
            "Significantly regressed" => format!(
                "Accessibility has regressed significantly versus the run on {} ({} points, +{} issues). Action needed.",
                preview.previous_date,
                preview.delta_accessibility,
                preview.delta_total_issues
            ),
            _ => format!(
                "Accessibility has slightly regressed versus the run on {}.",
                preview.previous_date
            ),
        }
    } else {
        match trend_label.as_str() {
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
        }
    };

    let summary = if en {
        format!(
            "{} The history covers {} usable snapshots.",
            trend_interpretation, preview.timeline_entries
        )
    } else {
        format!(
            "{} Die Historie umfasst {} verwertbare Snapshots.",
            trend_interpretation, preview.timeline_entries
        )
    };

    let (acc_delta, total_delta, issue_delta, crit_delta, prev_acc, prev_total) = if en {
        (
            "Accessibility delta",
            "Overall delta",
            "Issue delta",
            "Critical+High delta",
            "Previous accessibility",
            "Previous overall",
        )
    } else {
        (
            "Accessibility-Delta",
            "Gesamt-Delta",
            "Issue-Delta",
            "Kritisch+Hoch-Delta",
            "Vorher Accessibility",
            "Vorher Gesamt",
        )
    };

    HistoryTrendBlock {
        previous_date: preview.previous_date.clone(),
        timeline_entries: preview.timeline_entries,
        trend_label,
        summary,
        metrics: vec![
            (
                acc_delta.to_string(),
                format!("{:+}", preview.delta_accessibility),
            ),
            (
                total_delta.to_string(),
                format!("{:+}", preview.delta_overall),
            ),
            (
                issue_delta.to_string(),
                format!("{:+}", preview.delta_total_issues),
            ),
            (
                crit_delta.to_string(),
                format!("{:+}", preview.delta_critical_issues),
            ),
            (
                prev_acc.to_string(),
                preview.previous_accessibility_score.to_string(),
            ),
            (
                prev_total.to_string(),
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

fn build_modules_block_from_normalized(
    locale: &str,
    normalized: &NormalizedReport,
) -> ModulesBlock {
    let en = locale == "en";
    let a11y_score = normalized.score as f32;
    let mut dashboard = vec![ModuleScore {
        name: if en {
            "Accessibility".into()
        } else {
            "Barrierefreiheit".into()
        },
        score: a11y_score.round() as u32,
        interpretation: interpret_score(
            a11y_score,
            if en {
                "accessibility"
            } else {
                "Barrierefreiheit"
            },
        ),
        card_context: derive_accessibility_card_context(locale, normalized),
        score_context: derive_accessibility_context(locale, normalized),
        key_lever: derive_accessibility_lever(locale, normalized),
        good_threshold: 75,
        warn_threshold: 50,
    }];

    if let Some(ref p) = normalized.raw_performance {
        let score = normalized_module_score(normalized, "Performance").unwrap_or(p.score.overall);
        dashboard.push(ModuleScore {
            name: "Performance".into(),
            score,
            interpretation: interpret_score(
                score as f32,
                if en { "performance" } else { "Performance" },
            ),
            card_context: derive_performance_card_context(locale, p),
            score_context: derive_performance_context(locale, p),
            key_lever: derive_performance_lever(locale, p),
            good_threshold: 75,
            warn_threshold: 50,
        });
    }
    if let Some(ref s) = normalized.raw_seo {
        let score = normalized_module_score(normalized, "SEO").unwrap_or(s.score);
        dashboard.push(ModuleScore {
            name: "SEO".into(),
            score,
            interpretation: build_seo_interpretation(locale, s),
            card_context: derive_seo_card_context(locale, s),
            score_context: derive_seo_context(locale, s),
            key_lever: derive_seo_lever(locale, s),
            good_threshold: 75,
            warn_threshold: 50,
        });
    }
    if let Some(ref s) = normalized.raw_security {
        let score = normalized_module_score(normalized, "Security").unwrap_or(s.score);
        dashboard.push(ModuleScore {
            name: if en {
                "Security".into()
            } else {
                "Sicherheit".into()
            },
            score,
            interpretation: interpret_score(
                score as f32,
                if en { "security" } else { "Sicherheit" },
            ),
            card_context: derive_security_card_context(locale, s),
            score_context: derive_security_context(locale, s),
            key_lever: derive_security_lever(locale, s),
            good_threshold: 75,
            warn_threshold: 50,
        });
    }
    if let Some(ref m) = normalized.raw_mobile {
        let score = normalized_module_score(normalized, "Mobile").unwrap_or(m.score);
        dashboard.push(ModuleScore {
            name: "Mobile".into(),
            score,
            interpretation: interpret_score(
                score as f32,
                if en {
                    "mobile usability"
                } else {
                    "mobile Nutzbarkeit"
                },
            ),
            card_context: derive_mobile_card_context(locale, m),
            score_context: derive_mobile_context(locale, m),
            key_lever: derive_mobile_lever(locale, m),
            good_threshold: 75,
            warn_threshold: 50,
        });
    }
    if let Some(ref u) = normalized.raw_ux {
        let ux_score = normalized_module_score(normalized, "UX").unwrap_or(u.score);
        let ux_context = format!(
            "CTA Clarity {}/100, Visual Hierarchy {}/100, Content Clarity {}/100, Trust Signals {}/100, Cognitive Load {}/100",
            u.cta_clarity.score, u.visual_hierarchy.score, u.content_clarity.score, u.trust_signals.score, u.cognitive_load.score
        );
        let ux_lever: String = if u.cta_clarity.score < 60 {
            if en {
                "Phrase CTA texts more clearly and specifically"
            } else {
                "CTA-Texte klarer und spezifischer formulieren"
            }
        } else if u.trust_signals.score < 60 {
            if en {
                "Add trust signals (contact, imprint)"
            } else {
                "Vertrauenssignale (Kontakt, Impressum) ergänzen"
            }
        } else if u.visual_hierarchy.score < 60 {
            if en {
                "Clean up heading structure (H1 → H2 → H3)"
            } else {
                "Heading-Struktur bereinigen (H1 → H2 → H3)"
            }
        } else if en {
            "Maintain UX quality at the current good level"
        } else {
            "UX-Qualität auf gutem Niveau halten"
        }
        .into();
        dashboard.push(ModuleScore {
            name: "UX".into(),
            score: ux_score,
            interpretation: interpret_score(ux_score as f32, "User Experience"),
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
        if en {
            "Weighted average of all active modules. Accessibility 40%, Performance 20%, \
             SEO 20%, UX 15%, Security 10%, Mobile 10%."
                .to_string()
        } else {
            "Gewichteter Durchschnitt aller aktiven Module. Accessibility 40%, Performance 20%, \
             SEO 20%, UX 15%, Sicherheit 10%, Mobile 10%."
                .to_string()
        }
    });

    ModulesBlock {
        dashboard,
        overall_score,
        overall_interpretation,
    }
}

fn build_actions_block(
    locale: &str,
    plan: &ActionPlan,
    score: f32,
    _site_state: &crate::audit::summary::SiteState,
) -> ActionsBlock {
    let is_good_site = score >= 85.0
        || (plan.quick_wins.is_empty() && plan.medium_term.len() + plan.structural.len() <= 3);
    let item_cap: usize = if is_good_site { 2 } else { usize::MAX };

    let en = locale == "en";

    // Collect all items from all effort buckets, then re-bucket by semantic priority
    let all_items: Vec<ActionItem> = plan
        .quick_wins
        .iter()
        .chain(plan.medium_term.iter())
        .chain(plan.structural.iter())
        .cloned()
        .collect();

    let mut blockers: Vec<ActionItem> = Vec::new();
    let mut high_prio: Vec<ActionItem> = Vec::new();
    let mut medium_prio: Vec<ActionItem> = Vec::new();
    let mut low_prio: Vec<ActionItem> = Vec::new();

    for item in all_items {
        match item.priority {
            Priority::Critical => blockers.push(item),
            Priority::High => high_prio.push(item),
            Priority::Medium => medium_prio.push(item),
            Priority::Low => low_prio.push(item),
        }
    }

    // Within each bucket sort by execution_priority descending
    let sort_bucket = |mut v: Vec<ActionItem>| -> Vec<ActionItem> {
        v.sort_by_key(|i| std::cmp::Reverse(i.execution_priority));
        v
    };
    let blockers = sort_bucket(blockers);
    let high_prio = sort_bucket(high_prio);
    let medium_prio = sort_bucket(medium_prio);
    let low_prio = sort_bucket(low_prio);

    let map_items = |items: &[ActionItem]| -> Vec<RoadmapItemData> {
        items
            .iter()
            .take(item_cap)
            .map(|i| {
                let user_effect = derive_user_effect_from_action(locale, &i.action, i.effort);
                let risk_effect = match (i.priority, en) {
                    (Priority::Critical, true) => {
                        "Directly reduces critical WCAG violation risk".to_string()
                    }
                    (Priority::Critical, false) => {
                        "Reduziert kritisches WCAG-Verstoßrisiko direkt".to_string()
                    }
                    (Priority::High, true) => "Reduces high accessibility risk".to_string(),
                    (Priority::High, false) => {
                        "Reduziert hohes Barrierefreiheitsrisiko".to_string()
                    }
                    (Priority::Medium, true) => "Lowers medium accessibility risk".to_string(),
                    (Priority::Medium, false) => {
                        "Verringert mittleres Barrierefreiheitsrisiko".to_string()
                    }
                    (Priority::Low, true) => "Improves WCAG conformance in detail".to_string(),
                    (Priority::Low, false) => "Verbessert WCAG-Konformität im Detail".to_string(),
                };
                let conversion_effect =
                    derive_conversion_effect_from_action(locale, &i.action, i.effort);
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

    // Bucket labels and colors
    let (blocker_label, blocker_desc) = if en {
        (
            "Blocker — fix immediately",
            "Acute barriers — highest risk, must be resolved before anything else",
        )
    } else {
        (
            "Blocker — Sofort beheben",
            "Akute Barrieren — höchstes Risiko, vor allen anderen Punkten beheben",
        )
    };
    let (high_label, high_desc) = if en {
        (
            "High priority",
            "Significant barriers with direct usability impact",
        )
    } else {
        (
            "Hohe Priorität",
            "Relevante Barrieren mit direktem Impact auf Nutzbarkeit",
        )
    };
    let (medium_label, medium_desc) = if en {
        (
            "Medium priority",
            "Quality improvements with moderate accessibility benefit",
        )
    } else {
        (
            "Mittlere Priorität",
            "Qualitätsverbesserungen mit moderatem Barrierefreiheits-Nutzen",
        )
    };
    let (low_label, low_desc) = if en {
        ("Low priority", "Fine-tuning and optional improvements")
    } else {
        (
            "Niedrige Priorität",
            "Feinschliff und optionale Verbesserungen",
        )
    };

    let mut phase_preview = Vec::new();
    let mut columns = Vec::new();

    let push_group = |items: &Vec<ActionItem>,
                      label: &str,
                      desc: &str,
                      color: &str,
                      preview: &mut Vec<PhasePreview>,
                      cols: &mut Vec<RoadmapColumnData>| {
        if !items.is_empty() {
            preview.push(PhasePreview {
                phase_label: label.into(),
                accent_color: color.into(),
                description: desc.into(),
                item_count: items.len(),
                top_items: items.iter().map(|i| i.action.clone()).collect(),
            });
            cols.push(RoadmapColumnData {
                title: label.into(),
                accent_color: color.into(),
                items: map_items(items),
            });
        }
    };

    push_group(
        &blockers,
        blocker_label,
        blocker_desc,
        "#dc2626",
        &mut phase_preview,
        &mut columns,
    );
    push_group(
        &high_prio,
        high_label,
        high_desc,
        "#f59e0b",
        &mut phase_preview,
        &mut columns,
    );
    push_group(
        &medium_prio,
        medium_label,
        medium_desc,
        "#2563eb",
        &mut phase_preview,
        &mut columns,
    );
    push_group(
        &low_prio,
        low_label,
        low_desc,
        "#6b7280",
        &mut phase_preview,
        &mut columns,
    );

    // Determine primary responsible role from the largest group
    let primary_role = {
        let mut role_counts: std::collections::HashMap<&str, usize> =
            std::collections::HashMap::new();
        for r in &plan.role_assignments {
            *role_counts.entry(r.role.label()).or_default() += r.responsibilities.len();
        }
        role_counts
            .into_iter()
            .max_by_key(|(_, c)| *c)
            .map(|(r, _)| r.to_string())
            .unwrap_or_default()
    };

    let task_summary = TaskSummary {
        blocker_count: blockers.len(),
        high_count: high_prio.len(),
        medium_count: medium_prio.len(),
        low_count: low_prio.len(),
        total_count: blockers.len() + high_prio.len() + medium_prio.len() + low_prio.len(),
        primary_role,
    };

    let block_title = if is_good_site {
        if en {
            "Last optimization steps".to_string()
        } else {
            "Letzte Optimierungsschritte".to_string()
        }
    } else if en {
        "Action plan by priority".to_string()
    } else {
        "Maßnahmenplan nach Priorität".to_string()
    };

    let intro_text = if is_good_site {
        if en {
            "The site is technically well positioned. The following are final optimization levers without structural pressure.".to_string()
        } else {
            "Die Seite ist technisch stark aufgestellt. Die folgenden Punkte sind letzte Optimierungshebel ohne strukturellen Druck.".to_string()
        }
    } else if en {
        "Blockers must be resolved first — they carry the highest risk. High and medium priority items follow. Low priority items are optional improvements.".to_string()
    } else {
        "Blocker zuerst beheben — sie tragen das höchste Risiko. Danach folgen hohe und mittlere Priorität. Niedrige Priorität sind optionale Verbesserungen.".to_string()
    };

    ActionsBlock {
        roadmap_columns: columns,
        role_assignments: plan.role_assignments.clone(),
        intro_text,
        phase_preview,
        block_title,
        task_summary,
    }
}

fn build_finding_summary(
    locale: &str,
    normalized: &NormalizedReport,
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
        total: normalized.severity_counts.total,
        critical: normalized.severity_counts.critical,
        high: normalized.severity_counts.high,
        medium: normalized.severity_counts.medium,
        low: normalized.severity_counts.low,
        verdict: audit_summary.verdict_intro.clone(),
        dominant_issue_note: audit_summary.dominant_issue_note.clone(),
        cross_impact_notes,
        issue_pattern_label,
    }
}

fn build_thematic_clusters(locale: &str, findings: &[FindingGroup]) -> Vec<FindingCluster> {
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

    // Sort by occurrence_total descending for display priority
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

fn build_diagnosis_block(
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

    // Category breakdown: (dimension, finding_count, worst_severity_label)
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

    // Build clusters using the same logic, but from NormalizedReport for completeness
    let clusters = {
        let mut by_dim: BTreeMap<String, Vec<(String, usize, crate::wcag::Severity)>> =
            BTreeMap::new();
        for f in &normalized.findings {
            by_dim.entry(f.dimension.clone()).or_default().push((
                f.title.clone(),
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
            Konfligierende Signale werden als Hinweis markiert, verändern den Score jedoch nicht nachträglich."
            .to_string(),
        has_violations,
    }
}

fn build_methodology(locale: &str, normalized: &NormalizedReport) -> MethodologyBlock {
    let en = locale == "en";
    let active_modules = normalized
        .module_scores
        .iter()
        .map(|m| m.name.clone())
        .collect::<Vec<_>>()
        .join(", ");

    let scope = if en {
        format!(
            "Automated audit of {} for accessibility per WCAG 2.1 (level {}). \
             Performance, SEO, security and mobile usability were also analyzed.",
            normalized.url, normalized.wcag_level
        )
    } else {
        format!(
            "Automatisierte Prüfung der Seite {} auf Barrierefreiheit nach WCAG 2.1 (Level {}). \
             Zusätzlich wurden Performance, SEO, Sicherheit und mobile Nutzbarkeit analysiert.",
            normalized.url, normalized.wcag_level
        )
    };
    let method = if en {
        "The audit was performed via the Chrome DevTools Protocol (CDP) and the browser's native \
         accessibility tree. 21 WCAG rules were checked automatically against the page content."
            .to_string()
    } else {
        "Die Prüfung erfolgte über den Chrome DevTools Protocol (CDP) und den \
         nativen Accessibility Tree des Browsers. 21 WCAG-Regeln wurden automatisiert \
         gegen den Seiteninhalt geprüft."
            .to_string()
    };
    let limitations = if en {
        "Automated tests can detect about 30–40% of all accessibility issues. Complex aspects \
         such as correct tab order, meaningful alt texts, or understandable language additionally \
         require manual review."
            .to_string()
    } else {
        "Automatisierte Tests können ca. 30–40% aller Barrierefreiheitsprobleme erkennen. \
         Komplexe Aspekte wie korrekte Tab-Reihenfolge, sinnvolle Alt-Texte oder \
         verständliche Sprache erfordern zusätzlich manuelle Prüfung."
            .to_string()
    };
    let disclaimer = if en {
        "This report represents an automated technical analysis. It does not replace a complete \
         WCAG 2.1 conformance assessment. A legally defensible accessibility statement requires a \
         comprehensive manual audit by experts."
            .to_string()
    } else {
        "Dieser Report stellt eine automatisierte technische Analyse dar. \
         Er ersetzt keine vollständige Konformitätsbewertung nach WCAG 2.1. \
         Für eine rechtsverbindliche Aussage zur Barrierefreiheit ist eine \
         umfassende manuelle Prüfung durch Experten erforderlich."
            .to_string()
    };

    let key = |de: &str, en_label: &str| -> String {
        if en {
            en_label.to_string()
        } else {
            de.to_string()
        }
    };

    let preview_value = if normalized.has_screenshots {
        if en {
            "Desktop and mobile captured".to_string()
        } else {
            "Desktop und Mobile erfasst".to_string()
        }
    } else if en {
        "Not captured".to_string()
    } else {
        "Nicht erfasst".to_string()
    };

    let total_score_value = {
        let total_raw: u32 = normalized.module_scores.iter().map(|m| m.weight_pct).sum();
        let weights: Vec<String> = normalized
            .module_scores
            .iter()
            .map(|m| {
                let pct = (m.weight_pct * 100 + total_raw / 2)
                    .checked_div(total_raw)
                    .unwrap_or(0);
                format!("{} {}%", m.name, pct)
            })
            .collect();
        let weights_label = if weights.is_empty() {
            "Accessibility 100%".to_string()
        } else {
            weights.join(", ")
        };
        if en {
            format!(
                "{} / 100 — weighting: {}",
                normalized.overall_score, weights_label
            )
        } else {
            format!(
                "{} / 100 — Gewichtung: {}",
                normalized.overall_score, weights_label
            )
        }
    };

    let runtime_unit = "s";

    MethodologyBlock {
        scope,
        method,
        limitations,
        disclaimer,
        audit_facts: vec![
            (
                key("Primärscore", "Primary score"),
                format!("Accessibility {} / 100", normalized.score),
            ),
            (key("Gesamtscore", "Overall score"), total_score_value),
            (
                key("WCAG-Level", "WCAG level"),
                normalized.wcag_level.to_string(),
            ),
            (
                key("Geprüfte Knoten", "Checked nodes"),
                normalized.nodes_analyzed.to_string(),
            ),
            (
                key("Laufzeit", "Runtime"),
                format!(
                    "{:.1} {}",
                    normalized.duration_ms as f64 / 1000.0,
                    runtime_unit
                ),
            ),
            (key("Aktive Module", "Active modules"), active_modules),
            (
                key("Audit-Hinweise", "Audit notes"),
                normalized.audit_flags.len().to_string(),
            ),
            (key("Vorschau", "Preview"), preview_value),
        ],
        confidence_summary: build_confidence_summary(locale, normalized),
        capabilities: build_capability_matrix(locale, normalized),
    }
}

fn build_confidence_summary(locale: &str, normalized: &NormalizedReport) -> Vec<(String, String)> {
    let en = locale == "en";
    let base_confidence = if normalized.nodes_analyzed >= 2_000 {
        if en {
            "High"
        } else {
            "Hoch"
        }
    } else if normalized.nodes_analyzed >= 500 {
        if en {
            "Solid"
        } else {
            "Solide"
        }
    } else if en {
        "Limited"
    } else {
        "Begrenzt"
    };
    let caveat_level = if normalized.audit_flags.is_empty() {
        if en {
            "No automatically detected conflict signals"
        } else {
            "Keine automatisiert erkannten Konfliktsignale"
        }
    } else if normalized.audit_flags.len() == 1 {
        if en {
            "1 caveat signal"
        } else {
            "1 Hinweissignal"
        }
    } else if en {
        "Multiple caveat signals"
    } else {
        "Mehrere Hinweissignale"
    };
    let module_coverage = if normalized.module_scores.len() >= 5 {
        if en {
            "Broad"
        } else {
            "Breit"
        }
    } else if normalized.module_scores.len() >= 3 {
        if en {
            "Extended"
        } else {
            "Erweitert"
        }
    } else if en {
        "Core checks"
    } else {
        "Kern-Checks"
    };

    let (label_trust, label_coverage, label_signals, label_manual, val_manual) = if en {
        (
            "Audit confidence",
            "Module coverage",
            "Conflict signals",
            "Manual review needed",
            "Yes, for semantic quality and usage context",
        )
    } else {
        (
            "Audit-Vertrauen",
            "Modul-Abdeckung",
            "Konfliktsignale",
            "Manuelle Prüfung nötig",
            "Ja, für semantische Qualität und Nutzungskontext",
        )
    };

    vec![
        (label_trust.to_string(), base_confidence.to_string()),
        (label_coverage.to_string(), module_coverage.to_string()),
        (label_signals.to_string(), caveat_level.to_string()),
        (label_manual.to_string(), val_manual.to_string()),
    ]
}

fn build_capability_matrix(locale: &str, normalized: &NormalizedReport) -> Vec<CapabilitySignal> {
    let en = locale == "en";
    let confidence_high = if en { "High" } else { "Hoch" };
    let confidence_solid = if en { "Solid" } else { "Solide" };
    let confidence_off = if en { "Not active" } else { "Nicht aktiv" };

    let mut capabilities = vec![
        CapabilitySignal {
            signal: if en {
                "WCAG rules & occurrences".into()
            } else {
                "WCAG-Regeln & Vorkommen".into()
            },
            source: if en {
                "Accessibility tree + rule engine".into()
            } else {
                "Accessibility Tree + Regelengine".into()
            },
            confidence: if normalized.nodes_analyzed >= 500 {
                confidence_high.to_string()
            } else {
                confidence_solid.to_string()
            },
            surfaces: vec!["CLI".into(), "JSON".into(), "PDF".into(), "Studio".into()],
            note: if en {
                "Primary audit truth for automatically detectable violations.".to_string()
            } else {
                "Primäre Audit-Wahrheit für automatisiert erkennbare Verstöße.".to_string()
            },
        },
        CapabilitySignal {
            signal: if en {
                "Web vitals & loading indicators".into()
            } else {
                "Web Vitals & Ladeindikatoren".into()
            },
            source: if en {
                "Performance module".into()
            } else {
                "Performance-Modul".into()
            },
            confidence: if normalized.raw_performance.is_some() {
                confidence_high.to_string()
            } else {
                confidence_off.to_string()
            },
            surfaces: vec!["CLI".into(), "JSON".into(), "PDF".into()],
            note: if en {
                "FCP, CLS and TTFB are reflected in facts and module sections.".to_string()
            } else {
                "FCP, CLS und TTFB werden in Facts und Modulkapiteln gespiegelt.".to_string()
            },
        },
        CapabilitySignal {
            signal: if en {
                "SEO structure & schema".into()
            } else {
                "SEO-Struktur & Schema".into()
            },
            source: if en {
                "SEO module".into()
            } else {
                "SEO-Modul".into()
            },
            confidence: if normalized.raw_seo.is_some() {
                confidence_solid.to_string()
            } else {
                confidence_off.to_string()
            },
            surfaces: vec!["CLI".into(), "JSON".into(), "PDF".into()],
            note: if en {
                "Meta, heading and schema signals are condensed into a report-ready form."
                    .to_string()
            } else {
                "Meta-, Heading- und Schema-Signale sind reportfähig verdichtet.".to_string()
            },
        },
        CapabilitySignal {
            signal: if en {
                "Security headers & HTTPS".into()
            } else {
                "Security Header & HTTPS".into()
            },
            source: if en {
                "Security module".into()
            } else {
                "Security-Modul".into()
            },
            confidence: if normalized.raw_security.is_some() {
                confidence_high.to_string()
            } else {
                confidence_off.to_string()
            },
            surfaces: vec!["CLI".into(), "JSON".into(), "PDF".into()],
            note: if en {
                "Header presence and HTTPS status remain visible as raw signal.".to_string()
            } else {
                "Header-Präsenz und HTTPS-Status bleiben als Rohsignal sichtbar.".to_string()
            },
        },
        CapabilitySignal {
            signal: if en {
                "Mobile, UX, journey".into()
            } else {
                "Mobile, UX, Journey".into()
            },
            source: if en {
                "Heuristic modules".into()
            } else {
                "Heuristik-Module".into()
            },
            confidence: if en {
                "Indicator-based".into()
            } else {
                "Hinweisbasiert".into()
            },
            surfaces: vec!["CLI".into(), "JSON".into(), "PDF".into(), "Studio".into()],
            note: if en {
                "Useful for prioritization, not the sole UX truth.".to_string()
            } else {
                "Zur Priorisierung geeignet, nicht als alleinige UX-Gesamtwahrheit.".to_string()
            },
        },
    ];

    if !normalized.audit_flags.is_empty() {
        capabilities.push(CapabilitySignal {
            signal: if en {
                "Audit conflict signals".into()
            } else {
                "Audit-Konfliktsignale".into()
            },
            source: if en {
                "Normalization / cross-checks".into()
            } else {
                "Normalisierung / Cross-Checks".into()
            },
            confidence: if en {
                "Explicitly flagged".into()
            } else {
                "Explizit markiert".into()
            },
            surfaces: vec!["JSON".into(), "PDF".into()],
            note: if en {
                format!(
                    "{} conflict signal(s) are surfaced openly rather than hidden in the score.",
                    normalized.audit_flags.len()
                )
            } else {
                format!(
                    "{} Konfliktsignal(e) werden offen ausgewiesen statt im Score versteckt.",
                    normalized.audit_flags.len()
                )
            },
        });
    }

    capabilities
}

fn build_serp_presentation(s: &crate::seo::SerpAnalysis) -> SerpPresentation {
    let signals = s
        .signals
        .iter()
        .map(|sig| {
            (
                sig.category.clone(),
                sig.label.clone(),
                sig.status.label().to_string(),
                sig.detail.clone(),
            )
        })
        .collect();
    SerpPresentation {
        score: s.score,
        pass_count: s.pass_count,
        warning_count: s.warning_count,
        fail_count: s.fail_count,
        signals,
        rich_result_types: s.rich_result_types.clone(),
    }
}

fn build_page_health_presentation(
    locale: &str,
    ph: &crate::seo::PageHealthAnalysis,
) -> PageHealthPresentation {
    let issues: Vec<(String, String, String)> = ph
        .issues
        .iter()
        .map(|i| (i.issue_type.clone(), i.message.clone(), i.severity.clone()))
        .collect();

    let mut url_info: Vec<(String, String)> = vec![
        (
            "URL-Länge".to_string(),
            format!("{} Zeichen", ph.url_length),
        ),
        ("Pfadtiefe".to_string(), ph.url_path_depth.to_string()),
        (
            "Query-Parameter".to_string(),
            yes_no(locale, ph.url_has_query_params),
        ),
        (
            "Eigene Weiterleitung".to_string(),
            yes_no(locale, ph.own_redirect_detected),
        ),
    ];
    if let Some(ref final_url) = ph.own_final_url {
        url_info.push(("Ziel-URL".to_string(), final_url.clone()));
    }

    let html_issues: Vec<(String, u32, String, String)> = ph
        .html_issues
        .iter()
        .map(|i| {
            (
                i.check.clone(),
                i.count,
                i.severity.clone(),
                i.detail.clone(),
            )
        })
        .collect();

    let html_validator = Some((
        match ph.html_validator_status.as_str() {
            "executed" => "Ausgeführt".to_string(),
            "failed" => "Fehlgeschlagen".to_string(),
            _ => "Übersprungen".to_string(),
        },
        ph.html_validator_detail
            .clone()
            .unwrap_or_else(|| "Keine Zusatzinformationen verfügbar".to_string()),
    ));

    let www_status = ph.www_consolidation.as_ref().map(|w| {
        let www_label = w
            .www_status
            .map(|s| s.to_string())
            .unwrap_or_else(|| "—".to_string());
        let non_www_label = w
            .non_www_status
            .map(|s| s.to_string())
            .unwrap_or_else(|| "—".to_string());
        (www_label, non_www_label, w.is_consolidated)
    });

    let soft_404 = ph.soft_404_status.map(|s| (s, ph.is_soft_404));

    let has_any_issue = !issues.is_empty() || !html_issues.is_empty();

    PageHealthPresentation {
        issues,
        url_info,
        html_issues,
        html_validator,
        www_status,
        soft_404,
        has_any_issue,
    }
}

fn build_module_details_from_normalized(
    locale: &str,
    normalized: &NormalizedReport,
) -> ModuleDetailsBlock {
    let performance = normalized.raw_performance.as_ref().map(|p| {
        let performance_score =
            normalized_module_score(normalized, "Performance").unwrap_or(p.score.overall);
        let performance_grade = normalized_module_grade(normalized, "Performance")
            .unwrap_or_else(|| p.score.grade.label().to_string());
        let vitals = build_vitals_list(p);
        let desktop_viewport =
            normalized
                .raw_performance_desktop
                .as_ref()
                .map(|d| PerformanceViewport {
                    score: d.score.overall,
                    grade: d.score.grade.label().to_string(),
                    vitals: build_vitals_list(d),
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

        let recommendations = derive_performance_recommendations(locale, p);

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
        let score_below_excellent = performance_score < 85;
        let perf_interpretation = if cwv_all_good && score_below_excellent {
            let mut reasons = Vec::new();
            if p.vitals.dom_nodes.is_some_and(|n| n > 1500) {
                reasons.push("DOM-Größe");
            }
            if has_render_blocking {
                reasons.push("Render-blockierende Ressourcen");
            }
            if p.vitals.tbt.as_ref().is_some_and(|v| v.rating != "good") {
                reasons.push("Total Blocking Time");
            }
            if reasons.is_empty() {
                interpret_score(performance_score as f32, "Performance")
            } else {
                format!(
                    "{} Score durch {} reduziert, obwohl Core Web Vitals im grünen Bereich liegen.",
                    interpret_score(performance_score as f32, "Performance"),
                    reasons.join(", ")
                )
            }
        } else {
            interpret_score(performance_score as f32, "Performance")
        };

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
        }
    });

    let seo = normalized.raw_seo.as_ref().map(|s| {
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
                page_profile_summary: summarize_page_profile(locale, cp),
                optimization_note: page_profile_optimization_note(locale, cp),
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
                maturity_level: cp.maturity.label().to_string(),
                maturity_description: cp.maturity.description().to_string(),
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
            tracking_summary_text: build_tracking_summary_text(locale, &s.technical),
            profile,
            page_health: s
                .page_health
                .as_ref()
                .map(|p| build_page_health_presentation(locale, p)),
            serp: s.serp.as_ref().map(build_serp_presentation),
            robots: s.robots.as_ref().map(|r| {
                use crate::seo::BotClass;
                let bot_rows: Vec<(String, String, usize, usize, bool)> = r
                    .groups
                    .iter()
                    .map(|g| {
                        let fully_blocked = g.disallows.iter().any(|d| d == "/");
                        (
                            g.user_agent.clone(),
                            g.bot_class.to_string(),
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
                }
            }),
        }
    });

    let security = normalized.raw_security.as_ref().map(|sec| {
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
            score: security_score,
            grade: normalized_module_grade(normalized, "Security")
                .unwrap_or_else(|| sec.grade.clone()),
            interpretation: interpret_score(security_score as f32, "Sicherheit"),
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
            recommendations: derive_security_recommendations(locale, sec),
            protection: sec
                .protection
                .services
                .iter()
                .map(|s| (s.name.clone(), s.kind.clone()))
                .collect(),
            has_waf: sec.protection.has_waf,
            has_cdn: sec.protection.has_cdn,
        }
    });

    let mobile = normalized.raw_mobile.as_ref().map(|m| {
        let mobile_score = normalized_module_score(normalized, "Mobile").unwrap_or(m.score);
        let small_targets = m.touch_targets.small_targets;
        let context_hint = if !m.touch_targets.small_by_context.is_empty() {
            let parts: Vec<String> = m
                .touch_targets
                .small_by_context
                .iter()
                .take(3)
                .map(|(ctx, count)| format!("{} im Bereich {}", count, ctx))
                .collect();
            format!(" ({})", parts.join(", "))
        } else {
            String::new()
        };
        let mobile_interpretation = if small_targets >= 10 {
            format!(
                "{} {} Touch-Targets kleiner als empfohlen (44×44 px){}.",
                interpret_score(mobile_score as f32, "mobile Nutzbarkeit"),
                small_targets,
                context_hint,
            )
        } else {
            interpret_score(mobile_score as f32, "mobile Nutzbarkeit")
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

    let ux = normalized.raw_ux.as_ref().map(|u| {
        let ux_score = normalized_module_score(normalized, "UX").unwrap_or(u.score);
        UxPresentation {
            score: ux_score,
            grade: normalized_module_grade(normalized, "UX").unwrap_or_else(|| u.grade.clone()),
            interpretation: interpret_score(ux_score as f32, "User Experience"),
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
    });

    let journey = normalized.raw_journey.as_ref().map(|j| {
        let journey_score = normalized_module_score(normalized, "Journey").unwrap_or(j.score);
        // Detect page type mismatch between SEO profile and Journey module
        let seo_type: Option<String> = normalized
            .raw_seo
            .as_ref()
            .and_then(|s| s.content_profile.as_ref())
            .map(|cp| cp.page_classification.primary_type.label().to_lowercase());
        let journey_type = j.page_intent.label().to_lowercase();
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
        let journey_interpretation = if type_note.is_empty() {
            interpret_score(journey_score as f32, "User Journey")
        } else {
            format!(
                "{}{}",
                interpret_score(journey_score as f32, "User Journey"),
                type_note
            )
        };
        JourneyPresentation {
            score: journey_score,
            grade: normalized_module_grade(normalized, "Journey")
                .unwrap_or_else(|| j.grade.clone()),
            page_intent: j.page_intent.label().to_string(),
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
    });

    let has_any = performance.is_some()
        || seo.is_some()
        || security.is_some()
        || mobile.is_some()
        || ux.is_some()
        || journey.is_some()
        || dark_mode.is_some();
    let source_quality = normalized.raw_source_quality.clone();
    let ai_visibility = normalized.raw_ai_visibility.clone();
    let tech_stack = normalized.raw_tech_stack.clone();

    let has_any =
        has_any || source_quality.is_some() || ai_visibility.is_some() || tech_stack.is_some();

    ModuleDetailsBlock {
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
        has_any,
    }
}

fn finding_group_from_normalized(
    locale: &str,
    f: &crate::audit::normalized::NormalizedFinding,
) -> FindingGroup {
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
            expl.customer_title_for(locale).to_string(),
            expl.customer_description_for(locale).to_string(),
            expl.user_impact_for(locale).to_string(),
            derive_business_impact(
                locale,
                expl.user_impact_for(locale),
                f.dimension.as_str(),
                f.severity,
                Some(f.subcategory.as_str()),
                f.occurrence_count,
            ),
            expl.typical_cause_for(locale).to_string(),
            expl.recommendation_for(locale).to_string(),
            expl.technical_note_for(locale).to_string(),
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
                locale,
                &f.user_impact,
                f.dimension.as_str(),
                f.severity,
                Some(f.subcategory.as_str()),
                f.occurrence_count,
            ),
            if locale == "en" {
                "Automatically detected issue."
            } else {
                "Automatisch erkanntes Problem."
            }
            .to_string(),
            f.occurrences
                .first()
                .and_then(|o| o.fix_suggestion.clone())
                .unwrap_or_else(|| {
                    if locale == "en" {
                        "Please review and fix.".to_string()
                    } else {
                        "Bitte prüfen und beheben.".to_string()
                    }
                }),
            String::new(),
            Role::Development,
            Effort::Medium,
            derive_execution_priority(f.severity, Effort::Medium, f.dimension.as_str()),
        )
    };

    let examples = explanation.map(|e| e.examples()).unwrap_or_default();
    let location_hints = build_location_hints(&f.occurrences);
    let representative_occurrences = build_representative_occurrences(&f.occurrences);
    let pattern_clusters = build_pattern_clusters(&f.occurrences);
    let additional_occurrences = f
        .occurrence_count
        .saturating_sub(representative_occurrences.len());

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
        additional_occurrences,
        pattern_clusters,
        location_hints,
        representative_occurrences,
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

fn build_representative_occurrences(
    occurrences: &[crate::audit::normalized::OccurrenceDetail],
) -> Vec<RepresentativeOccurrence> {
    let mut ranked: Vec<(usize, i32, &crate::audit::normalized::OccurrenceDetail)> = occurrences
        .iter()
        .enumerate()
        .map(|(index, occ)| (index, representative_occurrence_score(occ), occ))
        .collect();
    ranked.sort_by(|a, b| {
        b.1.cmp(&a.1)
            .then_with(|| a.0.cmp(&b.0))
            .then_with(|| b.2.message.len().cmp(&a.2.message.len()))
    });

    let mut items = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for (_, _, occ) in ranked {
        let selector = representative_selector(occ);

        if !seen.insert(selector.to_ascii_lowercase()) {
            continue;
        }

        items.push(RepresentativeOccurrence {
            selector,
            node_id: occ.node_id.clone(),
            message: occ.message.clone(),
            html_snippet: occ.html_snippet.clone(),
            suggested_code: occ.suggested_code.clone(),
        });

        if items.len() >= 3 {
            break;
        }
    }

    items
}

fn build_pattern_clusters(
    occurrences: &[crate::audit::normalized::OccurrenceDetail],
) -> Vec<FindingPatternCluster> {
    let mut clusters: std::collections::BTreeMap<String, (String, usize)> =
        std::collections::BTreeMap::new();

    for occ in occurrences {
        let selector = representative_selector(occ);
        let normalized = normalize_selector_cluster(&selector);
        let entry = clusters.entry(normalized).or_insert((selector.clone(), 0));
        entry.1 += 1;

        if selector.len() < entry.0.len() {
            entry.0 = selector;
        }
    }

    let mut items: Vec<FindingPatternCluster> = clusters
        .into_values()
        .map(|(label, occurrences)| FindingPatternCluster { label, occurrences })
        .collect();
    items.sort_by(|a, b| {
        b.occurrences
            .cmp(&a.occurrences)
            .then_with(|| a.label.len().cmp(&b.label.len()))
    });
    items.truncate(3);
    items
}

fn representative_selector(occ: &crate::audit::normalized::OccurrenceDetail) -> String {
    occ.selector
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(&occ.node_id)
        .to_string()
}

fn normalize_selector_cluster(selector: &str) -> String {
    let trimmed = selector.trim();
    if trimmed.is_empty() {
        return "unspecified".to_string();
    }

    let normalized: String = trimmed
        .chars()
        .map(|ch| {
            if ch.is_ascii_digit() {
                '#'
            } else if ch.is_whitespace() {
                ' '
            } else {
                ch.to_ascii_lowercase()
            }
        })
        .collect();

    let mut compact = String::new();
    let mut last_was_hash = false;
    let mut last_was_space = false;
    for ch in normalized.chars() {
        match ch {
            '#' => {
                if !last_was_hash {
                    compact.push(ch);
                }
                last_was_hash = true;
                last_was_space = false;
            }
            ' ' => {
                if !last_was_space {
                    compact.push(ch);
                }
                last_was_space = true;
                last_was_hash = false;
            }
            _ => {
                compact.push(ch);
                last_was_hash = false;
                last_was_space = false;
            }
        }
    }

    compact
}

fn representative_occurrence_score(occ: &crate::audit::normalized::OccurrenceDetail) -> i32 {
    let mut score = 0;

    let selector = occ
        .selector
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());

    if let Some(selector) = selector {
        score += 40;
        score += selector_quality_score(selector);
    } else {
        score -= 12;
    }

    if has_content(&occ.html_snippet) {
        score += 28;
    }
    if has_content(&occ.suggested_code) {
        score += 22;
    }
    if has_content(&occ.fix_suggestion) {
        score += 8;
    }

    let message = occ.message.trim();
    if !message.is_empty() {
        score += 6;
        score += (message.len().min(120) / 20) as i32;
        if message.contains(':') || message.contains('(') {
            score += 3;
        }
        if message.chars().any(|ch| ch.is_ascii_digit()) {
            score += 4;
        }
    }

    if !occ.node_id.trim().is_empty() {
        score += 2;
    }

    score
}

fn selector_quality_score(selector: &str) -> i32 {
    let mut score = 0;

    if selector.contains('#') {
        score += 16;
    }
    if selector.contains('[') {
        score += 10;
    }
    if selector.contains('.') {
        score += 8;
    }
    if selector.contains('>') {
        score += 6;
    }
    if selector.contains(' ') {
        score += 4;
    }
    if selector.starts_with("main")
        || selector.starts_with("header")
        || selector.starts_with("nav")
        || selector.starts_with("footer")
    {
        score += 4;
    }

    score
}

fn has_content(value: &Option<String>) -> bool {
    value.as_deref().is_some_and(|text| !text.trim().is_empty())
}

fn derive_positive_aspects_from_normalized(
    locale: &str,
    normalized: &NormalizedReport,
) -> Vec<PositiveAspect> {
    let en = locale == "en";
    let mut positives = Vec::new();
    let a11y_score = normalized.score as f32;

    let area_a11y: String = if en {
        "Accessibility".into()
    } else {
        "Barrierefreiheit".into()
    };

    if normalized.findings.is_empty() {
        positives.push(PositiveAspect {
            area: area_a11y.clone(),
            description: if en {
                "No automatically detectable violations found.".into()
            } else {
                "Keine automatisch erkennbaren Verstöße gefunden.".into()
            },
        });
    } else if a11y_score >= 80.0 {
        positives.push(PositiveAspect {
            area: area_a11y,
            description: if en {
                "Solid base quality with focused, prioritizable remaining items.".into()
            } else {
                "Solide Grundqualität mit gezielt priorisierbaren Restpunkten.".into()
            },
        });
    }

    if let Some(ref perf) = normalized.raw_performance {
        if perf.score.overall >= 80 {
            positives.push(PositiveAspect {
                area: "Performance".into(),
                description: if en {
                    "Stable load times and overall responsive page build-up.".into()
                } else {
                    "Stabile Ladezeiten und insgesamt reaktionsschneller Seitenaufbau.".into()
                },
            });
        }
    }
    if let Some(ref seo) = normalized.raw_seo {
        if seo.score >= 80 {
            positives.push(PositiveAspect {
                area: "SEO".into(),
                description: if en {
                    "Clean foundation for discoverability, structure and meta data.".into()
                } else {
                    "Saubere Basis für Auffindbarkeit, Struktur und Meta-Daten.".into()
                },
            });
        }
    }
    if let Some(ref sec) = normalized.raw_security {
        if sec.score >= 80 {
            positives.push(PositiveAspect {
                area: if en {
                    "Security".into()
                } else {
                    "Sicherheit".into()
                },
                description: if en {
                    "Key security mechanisms are fundamentally in place.".into()
                } else {
                    "Wichtige Sicherheitsmechanismen sind grundsätzlich vorhanden.".into()
                },
            });
        }
    }
    if let Some(ref mobile) = normalized.raw_mobile {
        if mobile.score >= 80 {
            positives.push(PositiveAspect {
                area: "Mobile".into(),
                description: if en {
                    "The site is usable and readable on small displays.".into()
                } else {
                    "Die Seite ist auf kleinen Displays gut bedienbar und lesbar.".into()
                },
            });
        }
    }

    if positives.is_empty() {
        positives.push(PositiveAspect {
            area: if en {
                "Base structure".into()
            } else {
                "Grundstruktur".into()
            },
            description: if en {
                "The site is fundamentally functional and reachable.".into()
            } else {
                "Die Seite ist grundsätzlich funktional und erreichbar.".into()
            },
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
            additional_occurrences: self.additional_occurrences,
            pattern_clusters: self
                .pattern_clusters
                .iter()
                .map(|cluster| FindingPatternCluster {
                    label: cluster.label.clone(),
                    occurrences: cluster.occurrences,
                })
                .collect(),
            location_hints: self.location_hints.clone(),
            representative_occurrences: self
                .representative_occurrences
                .iter()
                .map(|occ| RepresentativeOccurrence {
                    selector: occ.selector.clone(),
                    node_id: occ.node_id.clone(),
                    message: occ.message.clone(),
                    html_snippet: occ.html_snippet.clone(),
                    suggested_code: occ.suggested_code.clone(),
                })
                .collect(),
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

fn normalized_module_score(normalized: &NormalizedReport, module_name: &str) -> Option<u32> {
    normalized
        .module_scores
        .iter()
        .find(|m| m.name == module_name)
        .map(|m| m.score)
}

fn normalized_module_grade(normalized: &NormalizedReport, module_name: &str) -> Option<String> {
    normalized
        .module_scores
        .iter()
        .find(|m| m.name == module_name)
        .map(|m| m.grade.clone())
}

#[cfg(test)]
mod tests {
    use super::{
        build_pattern_clusters, build_representative_occurrences, build_view_model,
        normalize_selector_cluster,
    };
    use crate::audit::normalized::OccurrenceDetail;
    use crate::audit::{normalize, AuditReport};
    use crate::cli::WcagLevel;
    use crate::output::report_model::{ReportConfig, ReportHistoryPreview};
    use crate::wcag::{Severity, Violation, WcagResults};

    #[test]
    fn representative_occurrences_prefer_rich_and_actionable_examples() {
        let occurrences = vec![
            OccurrenceDetail {
                node_id: "node-1".into(),
                message: "Short".into(),
                selector: None,
                fix_suggestion: None,
                html_snippet: None,
                suggested_code: None,
            },
            OccurrenceDetail {
                node_id: "node-2".into(),
                message: "Contrast ratio 1.13:1 for hero headline.".into(),
                selector: Some("main .hero-title".into()),
                fix_suggestion: Some("Increase foreground/background contrast.".into()),
                html_snippet: Some("<h1 class=\"hero-title\">Insights</h1>".into()),
                suggested_code: None,
            },
            OccurrenceDetail {
                node_id: "node-3".into(),
                message: "Contrast ratio 1.00:1 for CTA button text.".into(),
                selector: Some("#cta-primary".into()),
                fix_suggestion: Some("Use darker text color.".into()),
                html_snippet: Some("<a id=\"cta-primary\">Kontakt</a>".into()),
                suggested_code: Some(
                    "<a id=\"cta-primary\" class=\"text-stone-900\">Kontakt</a>".into(),
                ),
            },
            OccurrenceDetail {
                node_id: "node-4".into(),
                message: "Link landmark is outside a region.".into(),
                selector: Some("a.skip-link".into()),
                fix_suggestion: None,
                html_snippet: None,
                suggested_code: None,
            },
        ];

        let selected = build_representative_occurrences(&occurrences);

        assert_eq!(selected.len(), 3);
        assert_eq!(selected[0].selector, "#cta-primary");
        assert_eq!(selected[1].selector, "main .hero-title");
        assert_eq!(selected[2].selector, "a.skip-link");
    }

    #[test]
    fn representative_occurrences_deduplicate_selector_variants() {
        let occurrences = vec![
            OccurrenceDetail {
                node_id: "node-1".into(),
                message: "First duplicate".into(),
                selector: Some("main .hero-title".into()),
                fix_suggestion: None,
                html_snippet: Some("<h1 class=\"hero-title\">One</h1>".into()),
                suggested_code: None,
            },
            OccurrenceDetail {
                node_id: "node-2".into(),
                message: "Second duplicate with richer text".into(),
                selector: Some("MAIN .HERO-TITLE".into()),
                fix_suggestion: Some("Adjust markup.".into()),
                html_snippet: Some("<h1 class=\"hero-title\">Two</h1>".into()),
                suggested_code: Some("<h1 lang=\"de\">Two</h1>".into()),
            },
            OccurrenceDetail {
                node_id: "node-3".into(),
                message: "Independent selector".into(),
                selector: Some("footer .meta a".into()),
                fix_suggestion: None,
                html_snippet: None,
                suggested_code: None,
            },
        ];

        let selected = build_representative_occurrences(&occurrences);

        assert_eq!(selected.len(), 2);
        assert_eq!(selected[0].selector, "MAIN .HERO-TITLE");
        assert_eq!(selected[1].selector, "footer .meta a");
    }

    #[test]
    fn pattern_clusters_group_similar_selector_variants() {
        let occurrences = vec![
            OccurrenceDetail {
                node_id: "node-1".into(),
                message: "One".into(),
                selector: Some("main .card-1 .cta".into()),
                fix_suggestion: None,
                html_snippet: None,
                suggested_code: None,
            },
            OccurrenceDetail {
                node_id: "node-2".into(),
                message: "Two".into(),
                selector: Some("main .card-2 .cta".into()),
                fix_suggestion: None,
                html_snippet: None,
                suggested_code: None,
            },
            OccurrenceDetail {
                node_id: "node-3".into(),
                message: "Three".into(),
                selector: Some("footer .meta a".into()),
                fix_suggestion: None,
                html_snippet: None,
                suggested_code: None,
            },
        ];

        let clusters = build_pattern_clusters(&occurrences);

        assert_eq!(
            normalize_selector_cluster("main .card-1 .cta"),
            "main .card-# .cta"
        );
        assert_eq!(clusters[0].occurrences, 2);
        assert_eq!(clusters[0].label, "main .card-1 .cta");
    }

    #[test]
    fn view_model_exposes_confidence_and_capabilities() {
        let mut results = WcagResults::new();
        results.add_violation(Violation::new(
            "1.1.1",
            "Non-text Content",
            WcagLevel::A,
            Severity::High,
            "Image missing alt attribute",
            "node-123",
        ));

        let report = AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            results,
            1500,
        );
        let normalized = normalize(&report);
        let vm = build_view_model(&normalized, &ReportConfig::default());

        assert!(vm
            .methodology
            .confidence_summary
            .iter()
            .any(|(label, _)| label == "Audit-Vertrauen"));
        assert!(vm
            .methodology
            .capabilities
            .iter()
            .any(|cap| cap.signal == "WCAG-Regeln & Vorkommen"));
    }

    #[test]
    fn view_model_exposes_history_delta_when_preview_exists() {
        let report = AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            WcagResults::new(),
            1500,
        );
        let normalized = normalize(&report);
        let vm = build_view_model(
            &normalized,
            &ReportConfig {
                history_preview: Some(ReportHistoryPreview {
                    previous_date: "01.04.2026".to_string(),
                    timeline_entries: 3,
                    previous_accessibility_score: 74,
                    previous_overall_score: 78,
                    delta_accessibility: 8,
                    delta_overall: 5,
                    delta_total_issues: -6,
                    delta_critical_issues: -2,
                    recent_entries: vec![
                        ("01.04.2026".to_string(), 74, 78, "C".to_string(), 12),
                        (
                            "06.04.2026".to_string(),
                            normalized.score,
                            normalized.overall_score,
                            normalized.grade.clone(),
                            normalized.severity_counts.total as u32,
                        ),
                    ],
                    new_findings: vec!["Link-Purpose".to_string()],
                    resolved_findings: vec!["Alt-Text".to_string()],
                }),
                ..ReportConfig::default()
            },
        );

        let history = vm.history.expect("history block should exist");
        assert_eq!(history.trend_label, "Deutlich verbessert");
        assert!(history.summary.contains("01.04.2026"));
        assert_eq!(history.timeline_rows.len(), 2);
        assert_eq!(history.resolved_findings.len(), 1);
    }

    #[test]
    fn english_view_model_excludes_top_level_german_labels() {
        use crate::audit::normalize;
        use crate::audit::AuditReport;
        use crate::wcag::{Severity, Violation, WcagResults};

        let mut results = WcagResults::new();
        results.add_violation(
            Violation::new(
                "1.1.1",
                "Non-text Content",
                WcagLevel::A,
                Severity::High,
                "Image missing alternative text",
                "node-hero-image",
            )
            .with_selector("img.hero")
            .with_html_snippet("<img class=\"hero\" src=\"hero.jpg\">")
            .with_fix("Add a meaningful alt attribute"),
        );
        let report = AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            results,
            1_500,
        );
        let normalized = normalize(&report);
        let vm = build_view_model(
            &normalized,
            &ReportConfig {
                locale: "en".to_string(),
                ..ReportConfig::default()
            },
        );

        // Top-level cover and executive narrative must not contain core German labels.
        let exec = &vm.executive;
        let candidates: Vec<&str> = vec![
            exec.cover_eyebrow.as_str(),
            exec.cover_kicker.as_str(),
            exec.status_title.as_str(),
            exec.metrics_title.as_str(),
            exec.key_points_title.as_str(),
            exec.impact_title.as_str(),
            exec.quick_actions_title.as_str(),
            exec.spotlight_eyebrow.as_str(),
            exec.leverage_title.as_str(),
            exec.findings_title.as_str(),
            exec.findings_intro.as_str(),
            exec.action_plan_title.as_str(),
            exec.action_plan_intro.as_str(),
            exec.action_plan_callout_title.as_str(),
            exec.action_plan_callout_body.as_str(),
            exec.technical_title.as_str(),
            exec.technical_intro.as_str(),
            exec.next_steps_title.as_str(),
            exec.next_steps_intro.as_str(),
            exec.next_steps_callout_title.as_str(),
            exec.next_steps_callout_body.as_str(),
            vm.summary.verdict.as_str(),
            vm.summary.executive_lead.as_str(),
            vm.summary.maturity_label.as_str(),
            vm.summary.problem_type.as_str(),
            vm.summary.business_consequence.as_str(),
            vm.summary.consequence.as_str(),
            vm.summary.risk_level.as_str(),
        ];

        // Marker words that should never appear in a localized English narrative.
        let forbidden = [
            "Automatisierter",
            "Empfohlen",
            "Maßnahmen",
            "Verbesserungshebel",
            "Solide Basis",
            "Kernaussagen",
            "Zertifikat ",
            "Auswirkungen",
            "Hauptproblem",
            "Stark",
            "Instabil",
            "Nutzbarkeit",
            "Barrierefreiheits",
            "Optimierungshebel",
            "Feinschliff",
            "Empfohlene",
            "Wirkung einer Behebung",
            "Erreicht",
            "Gering",
            "Hoch",
            "Mittel",
        ];

        for text in &candidates {
            for word in &forbidden {
                assert!(
                    !text.contains(word),
                    "English ViewModel still contains German marker '{}': {}",
                    word,
                    text
                );
            }
        }

        // Risk level must be a localized English label.
        assert!(
            ["Critical", "High", "Medium", "Low"].contains(&vm.summary.risk_level.as_str()),
            "Risk level should be an English label, got {}",
            vm.summary.risk_level
        );

        // Site state (maturity_label) must be one of the English variants.
        assert!(
            ["Strong", "Solid foundation", "Unstable", "Critical"]
                .contains(&vm.summary.maturity_label.as_str()),
            "Maturity label should be English, got {}",
            vm.summary.maturity_label
        );
    }
}
