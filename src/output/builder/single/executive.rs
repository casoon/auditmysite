use crate::audit::normalized::NormalizedReport;
use crate::i18n::I18n;
use crate::output::report_model::{
    ActionPlan, ExecutiveNarrativeBlock, FindingGroup, PositiveSignal, PositiveSignalsBlock,
    SeverityBlock,
};

use super::super::helpers::{build_business_consequence, build_overall_impact};

/// Translates pattern names into localized titles; the message stays as the
/// description (already human-readable from the detector).
pub(super) fn build_positive_signals(
    locale: &str,
    normalized: &NormalizedReport,
) -> PositiveSignalsBlock {
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

pub(super) fn build_executive_narrative(
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
                        "The majority of critical problems originate from this single topic."
                            .to_string()
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

    if let Some(flag) = normalized
        .audit_flags
        .iter()
        .find(|f| f.kind == "viewport_gap")
    {
        points.push(flag.message.clone());
    }

    points
}

fn build_single_quick_actions_text(
    _locale: &str,
    action_plan: &ActionPlan,
    top_findings: &[FindingGroup],
) -> Vec<String> {
    let mut actions: Vec<String> = Vec::new();

    for item in &action_plan.quick_wins {
        actions.push(item.action.clone());
    }

    if actions.is_empty() {
        for group in top_findings.iter().take(3) {
            actions.push(sentence_preview(&group.recommendation).to_string());
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
