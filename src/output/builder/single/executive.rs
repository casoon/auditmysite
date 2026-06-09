use crate::audit::normalized::{AuditContext, NormalizedReport};
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
    normalized: &AuditContext,
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
    // Single source of truth for "share of critical/high findings from the
    // dominant issue". Both the key-points line and the leverage text below must
    // use this value — never an independently recomputed percentage (#360).
    let dominant_share = audit_summary
        .dominant_issue
        .as_ref()
        .map(|d| d.share_pct.round() as usize);
    let key_points = build_single_key_points_text(
        i18n.locale(),
        severity,
        top_findings,
        normalized,
        dominant_share,
    );
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
            if normalized.risk.legal_flags > 0 {
                if en {
                    format!(
                        "Automated checks detected {} high or critical WCAG Level A findings — \
                         potentially relevant for BFSG/EAA. Additional manual review is required \
                         for a defensible legal classification.",
                        normalized.risk.legal_flags
                    )
                } else {
                    format!(
                        "Automatisiert wurden {} hohe oder kritische WCAG-Level-A-Befunde erkannt — \
                         potenziell relevant für BFSG/EAA. Für eine belastbare rechtliche \
                         Einordnung ist ergänzend manuelle Prüfung nötig.",
                        normalized.risk.legal_flags
                    )
                }
            } else if severity.high > 0 {
                if en {
                    "Automated checks found no high or critical Level A findings, but WCAG gaps \
                     exist. Additional manual review is required for a defensible BFSG/WCAG \
                     classification."
                        .to_string()
                } else {
                    "Automatisiert wurden keine hohen oder kritischen Level-A-Befunde erkannt, aber WCAG-Mängel \
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
        let share = dominant_share.unwrap_or_else(|| {
            (top.occurrence_count * 100)
                .checked_div(total_ch)
                .unwrap_or(0)
        });
        (
                audit_summary.dominant_issue_note.clone().unwrap_or_else(|| {
                    if en {
                        "The majority of critical/high findings originate from this single topic."
                            .to_string()
                    } else {
                        "Der Großteil der kritischen/hohen Befunde entsteht durch dieses eine Thema."
                            .to_string()
                    }
                }),
                sentence_preview(&top.user_impact).to_string(),
                sentence_preview(&top.recommendation).to_string(),
                (total_ch > 0).then(|| {
                    if en {
                        format!(
                            "Fixing the main issue removes about {}% of critical/high findings. Immediately tangible improvement in usability.",
                            share.min(99)
                        )
                    } else {
                        format!(
                            "Behebung des Hauptproblems reduziert ca. {}% der kritischen/hohen Befunde. Die Nutzbarkeit verbessert sich dadurch an vielen betroffenen Stellen.",
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
            "Improvements should be grouped by recurring patterns and user impact.".to_string(),
            None,
        )
    } else {
        (
            "Kein einzelnes Problem dominiert das Auditbild; die Befunde sind breiter verteilt."
                .to_string(),
            "Die Wirkung verteilt sich auf mehrere kleinere Barrieren.".to_string(),
            "Die Verbesserungen sollten nach wiederkehrenden Mustern und Nutzerwirkung gebündelt werden."
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
        risk_title: assessment,
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

/// Assessment label for the executive cover. The score band is the primary
/// signal (bands per CLAUDE.md: ≥90/≥75/≥60/≥40/<40); severity only refines the
/// wording *within* a band. A low score must never yield a reassuring label —
/// e.g. "Gute Basis" may only appear from score ≥ 60 upward.
fn build_single_assessment_text(locale: &str, score: u32, severity: &SeverityBlock) -> String {
    let en = locale == "en";
    let has_critical_a11y = severity.critical > 0;
    let has_high = severity.high > 0;

    if score < 40 {
        // Critical band — never a "good foundation" wording, regardless of severity mix.
        if en {
            "Critical barriers — not WCAG conformant".to_string()
        } else {
            "Kritische Barrieren — nicht WCAG-konform".to_string()
        }
    } else if score < 60 {
        // Inadequate band: relevant barriers, no reassurance.
        if has_critical_a11y {
            if en {
                "Serious barriers — not WCAG conformant".to_string()
            } else {
                "Gravierende Barrieren — nicht WCAG-konform".to_string()
            }
        } else if en {
            "Substantial accessibility gaps".to_string()
        } else {
            "Erhebliche Barrierefreiheitslücken".to_string()
        }
    } else if score < 75 {
        // Needs-improvement band.
        if has_critical_a11y {
            if en {
                "Usable, but legally risky".to_string()
            } else {
                "Nutzbar, aber rechtlich riskant".to_string()
            }
        } else if has_high {
            if en {
                "Usable foundation, but not yet accessible".to_string()
            } else {
                "Nutzbare Basis, aber noch nicht barrierefrei".to_string()
            }
        } else if en {
            "Needs improvement toward accessibility".to_string()
        } else {
            "Verbesserungswürdig auf dem Weg zur Barrierefreiheit".to_string()
        }
    } else if score < 90 {
        // Good band.
        if has_critical_a11y {
            if en {
                "Technically stable, but legally risky".to_string()
            } else {
                "Technisch stabil, aber rechtlich riskant".to_string()
            }
        } else if has_high {
            if en {
                "Good foundation, but not accessible".to_string()
            } else {
                "Gute Basis, aber nicht barrierefrei".to_string()
            }
        } else if en {
            "Largely accessible — fine-tuning".to_string()
        } else {
            "Weitgehend barrierefrei — Feinschliff".to_string()
        }
    } else {
        // Excellent band.
        if has_high {
            if en {
                "Largely accessible — close residual gaps".to_string()
            } else {
                "Weitgehend barrierefrei — Restlücken schließen".to_string()
            }
        } else if en {
            "Largely accessible — polish".to_string()
        } else {
            "Weitgehend barrierefrei — Feinschliff".to_string()
        }
    }
}

fn build_single_key_points_text(
    locale: &str,
    severity: &SeverityBlock,
    top_findings: &[FindingGroup],
    normalized: &NormalizedReport,
    dominant_share: Option<usize>,
) -> Vec<String> {
    let en = locale == "en";
    let mut points = Vec::with_capacity(3);
    if severity.critical > 0 {
        if en {
            points.push("Multiple critical WCAG barriers block assistive technology users — screen readers and keyboard navigation are affected.".to_string());
        } else {
            points.push("Mehrere kritische WCAG-Barrieren blockieren Nutzer von Hilfstechnologien — Screenreader und Tastaturnavigation sind betroffen.".to_string());
        }
    } else if severity.high > 0 {
        if en {
            points.push("Significant accessibility gaps reduce usability for users with disabilities — no complete access barriers, but relevant friction points.".to_string());
        } else {
            points.push("Deutliche Barrierefreiheitslücken erschweren die Nutzung für Menschen mit Behinderungen — keine vollständigen Zugangssperren, aber relevante Reibungspunkte.".to_string());
        }
    }

    if let Some(top) = top_findings.first() {
        let total_ch = (severity.critical + severity.high) as usize;
        // Prefer the canonical dominant-issue share so this line and the
        // leverage text never disagree on the same percentage (#360).
        let share = dominant_share.unwrap_or_else(|| {
            (top.occurrence_count * 100)
                .checked_div(total_ch)
                .unwrap_or(0)
        });
        if share >= 30 {
            if en {
                points.push(format!(
                    "Main issue: {} ({}% of all critical/high findings)",
                    top.title,
                    share.min(99)
                ));
            } else {
                points.push(format!(
                    "Hauptproblem: {} ({}% aller kritischen/hohen Befunde)",
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

    let score_driver_note = build_score_driver_note(locale, normalized);
    if !score_driver_note.is_empty() {
        points.push(score_driver_note);
    }

    if normalized.risk.legal_flags > 0 {
        if en {
            points.push(
                "High or critical WCAG Level A findings detected automatically — manual review needed for a defensible BFSG classification".to_string(),
            );
        } else {
            points.push(
                "Hohe oder kritische WCAG-Level-A-Befunde automatisiert erkannt — manuelle Prüfung für belastbare BFSG-Einordnung nötig".to_string(),
            );
        }
    } else if severity.high > 0 {
        if en {
            points.push("No Level A violations, but structural weaknesses".to_string());
        } else {
            points.push(
                "Keine Level-A-Verstöße, aber strukturelle Optimierungspotenziale".to_string(),
            );
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

    let (automated, total) = crate::wcag::coverage::coverage_stats();
    if en {
        points.push(format!(
            "WCAG scope: {automated} of about {total} WCAG 2.1 AA criteria are checked automatically; manual review remains required for context-dependent criteria."
        ));
    } else {
        points.push(format!(
            "WCAG-Prüfumfang: {automated} von ca. {total} WCAG-2.1-AA-Kriterien werden automatisch geprüft; kontextabhängige Kriterien bleiben manuell zu prüfen."
        ));
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

fn build_score_driver_note(locale: &str, normalized: &NormalizedReport) -> String {
    let en = locale == "en";
    let mut drivers: std::collections::BTreeMap<&'static str, u32> =
        std::collections::BTreeMap::new();
    for finding in &normalized.findings {
        let severity_weight = match finding.severity {
            crate::wcag::Severity::Critical => 20,
            crate::wcag::Severity::High => 14,
            crate::wcag::Severity::Medium => 8,
            crate::wcag::Severity::Low => 4,
        };
        *drivers
            .entry(score_area_for_key_point(finding))
            .or_default() += severity_weight * finding.occurrence_count as u32;
    }
    let mut drivers: Vec<_> = drivers.into_iter().filter(|(_, loss)| *loss > 0).collect();
    drivers.sort_by_key(|(_, loss)| std::cmp::Reverse(*loss));
    let labels: Vec<_> = drivers.into_iter().take(3).map(|(area, _)| area).collect();
    if labels.is_empty() {
        if en {
            "Score matrix: no negative driver detected in the weighted accessibility topics."
                .to_string()
        } else {
            "Score-Matrix: kein negativer Haupttreiber in den gewichteten Accessibility-Themen erkannt."
                .to_string()
        }
    } else if en {
        format!(
            "Score matrix: strongest negative drivers are {}; weighting includes semantics, forms, keyboard, focus, images, ARIA, headings and landmarks.",
            labels.join(", ")
        )
    } else {
        format!(
            "Score-Matrix: stärkste negative Treiber sind {}; gewichtet werden Semantik, Formulare, Tastatur, Fokus, Bilder, ARIA, Überschriften und Landmarks.",
            labels.join(", ")
        )
    }
}

fn score_area_for_key_point(finding: &crate::audit::normalized::NormalizedFinding) -> &'static str {
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
