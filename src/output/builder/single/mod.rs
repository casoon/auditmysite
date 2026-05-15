//! Single-report ViewModel builder.

mod diagnosis;
mod executive;
mod findings;
mod methodology;
mod module_details;
mod positive;
mod serp;
use self::diagnosis::{build_diagnosis_block, build_finding_summary, build_thematic_clusters};
use self::executive::{build_executive_narrative, build_positive_signals};
use self::findings::finding_group_from_normalized;
use self::methodology::{build_appendix_block_from_normalized, build_methodology};
use self::module_details::{build_module_details_from_normalized, normalized_module_score};
use self::positive::derive_positive_aspects_from_normalized;

use crate::audit::normalized::{NormalizedReport, SeverityCounts};
use crate::audit::summary::analyze_with_locale;
use crate::cli::ReportLevel;
use crate::i18n::I18n;
use crate::output::report_model::*;
use crate::wcag::Severity;

use super::actions::{
    derive_action_plan, derive_conversion_effect_from_action, derive_user_effect_from_action,
    humanize_action_text, impact_score,
};
use super::helpers::{
    build_benchmark_context, build_business_consequence, build_consequence_text,
    build_overall_impact, build_score_note, build_technical_overview, build_trend_label,
    build_verdict_text, extract_domain, interpret_score, localized_report_subtitle,
    localized_report_title,
};
use super::modules::{
    derive_accessibility_card_context, derive_accessibility_context, derive_accessibility_lever,
    derive_mobile_card_context, derive_mobile_context, derive_mobile_lever,
    derive_performance_card_context, derive_performance_context, derive_performance_lever,
    derive_security_card_context, derive_security_context, derive_security_lever,
    derive_seo_card_context, derive_seo_context, derive_seo_lever,
};
use super::seo::build_seo_interpretation;

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
    let filtered_counts = {
        let critical = sorted_groups
            .iter()
            .filter(|f| f.severity == Severity::Critical)
            .map(|f| f.occurrence_count)
            .sum();
        let high = sorted_groups
            .iter()
            .filter(|f| f.severity == Severity::High)
            .map(|f| f.occurrence_count)
            .sum();
        let medium = sorted_groups
            .iter()
            .filter(|f| f.severity == Severity::Medium)
            .map(|f| f.occurrence_count)
            .sum();
        let low = sorted_groups
            .iter()
            .filter(|f| f.severity == Severity::Low)
            .map(|f| f.occurrence_count)
            .sum();
        SeverityCounts {
            critical,
            high,
            medium,
            low,
            total: critical + high + medium + low,
        }
    };

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
        critical: filtered_counts.critical as u32,
        high: filtered_counts.high as u32,
        medium: filtered_counts.medium as u32,
        low: filtered_counts.low as u32,
        total: filtered_counts.total as u32,
        has_issues: filtered_counts.total > 0,
    };

    let modules = build_modules_block_from_normalized(&config.locale, normalized);

    let quick_win_count = action_plan.quick_wins.len();
    let critical_count = (filtered_counts.critical + filtered_counts.high) as u32;
    let total_violations = filtered_counts.total as u32;
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
            let finding_summary =
                build_finding_summary(&config.locale, &filtered_counts, &audit_summary);
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
        measurement_type: "measured".into(),
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
            measurement_type: "measured".into(),
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
            measurement_type: "measured".into(),
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
            measurement_type: "measured".into(),
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
            measurement_type: "measured".into(),
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
            measurement_type: "heuristic".into(),
            interpretation: interpret_score(ux_score as f32, "User Experience"),
            card_context: ux_context.clone(),
            score_context: ux_context,
            key_lever: ux_lever,
            good_threshold: 80,
            warn_threshold: 55,
        });
    }
    if let Some(ref j) = normalized.raw_journey {
        let journey_score = normalized_module_score(normalized, "Journey").unwrap_or(j.score);
        let journey_context = format!(
            "{}: {} · Entry {}/100, Orientation {}/100, Navigation {}/100, Interaction {}/100, Conversion {}/100",
            if en { "Intent" } else { "Seitenabsicht" },
            j.page_intent.label(),
            j.entry_clarity.score,
            j.orientation.score,
            j.navigation.score,
            j.interaction.score,
            j.conversion.score
        );
        let journey_lever = j
            .friction_points
            .first()
            .map(|fp| fp.recommendation.clone())
            .unwrap_or_else(|| {
                if en {
                    "Maintain journey clarity at the current level".to_string()
                } else {
                    "Journey-Klarheit auf aktuellem Niveau halten".to_string()
                }
            });
        dashboard.push(ModuleScore {
            name: "Journey".into(),
            score: journey_score,
            measurement_type: "heuristic".into(),
            interpretation: interpret_score(journey_score as f32, "User Journey"),
            card_context: journey_context.clone(),
            score_context: journey_context,
            key_lever: journey_lever,
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
    let overall_interpretation =
        overall_score.map(|_| build_overall_score_explanation(locale, normalized));

    ModulesBlock {
        dashboard,
        overall_score,
        overall_interpretation,
    }
}

fn build_overall_score_explanation(locale: &str, normalized: &NormalizedReport) -> String {
    let en = locale == "en";
    let indicator_names: Vec<String> = normalized
        .module_scores
        .iter()
        .filter(|m| !m.contributes_to_overall || m.measurement_type == "heuristic")
        .map(|m| m.name.clone())
        .collect();
    let indicator_note = if indicator_names.is_empty() {
        String::new()
    } else if en {
        format!(
            " Indicator modules ({}) are shown separately and do not change the overall score.",
            indicator_names.join(", ")
        )
    } else {
        format!(
            " Indikator-Module ({}) werden separat ausgewiesen und verändern den Gesamtscore nicht.",
            indicator_names.join(", ")
        )
    };

    if normalized.viewport_scores.is_some() {
        if en {
            format!(
                "Overall score uses the dual-viewport result: 70% mobile and 30% desktop. \
                 Security contributes 10% when active.{indicator_note}"
            )
        } else {
            format!(
                "Der Gesamtscore nutzt das Dual-Viewport-Ergebnis: 70% Mobile und 30% Desktop. \
                 Sicherheit fließt mit 10% ein, wenn aktiv.{indicator_note}"
            )
        }
    } else {
        let contributing: Vec<String> = normalized
            .module_scores
            .iter()
            .filter(|m| m.contributes_to_overall)
            .map(|m| format!("{} {}%", m.name, m.weight_pct))
            .collect();
        let weights = if contributing.is_empty() {
            "Accessibility 100%".to_string()
        } else {
            contributing.join(", ")
        };
        if en {
            format!("Weighted average of contributing modules: {weights}.{indicator_note}")
        } else {
            format!("Gewichteter Durchschnitt der beitragenden Module: {weights}.{indicator_note}")
        }
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

#[cfg(test)]
mod tests {
    use super::build_view_model;
    use crate::audit::{normalize, AuditReport};
    use crate::cli::WcagLevel;
    use crate::output::report_model::{ReportConfig, ReportHistoryPreview};
    use crate::wcag::{Severity, Violation, WcagResults};

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
