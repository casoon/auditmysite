//! Management-Risiko-Dimensionen — eine Ableitung für JSON und PDF.
//!
//! Vorher existierten zwei unabhängige Modelle: `build_management_risks` für
//! das JSON und `compute_dimension_rows` in der PDF-Schicht. Sie führten
//! unterschiedliche Dimensionen, stuften unterschiedlich ein, und die
//! belegkräftigen Begründungen des JSON erreichten den PDF-Leser nie
//! (Plan 35).
//!
//! Lokalisierung folgt dem #406-Muster: die Analyse backt kanonisches Englisch
//! in das gespeicherte JSON, die PDF-Präsentationsschicht leitet den Text zur
//! Laufzeit ab. [`ManagementRiskKind`] trägt die Rohwerte, und
//! `dimension`/`rationale` sind die EINZIGE Textquelle — das JSON ruft sie mit
//! `en = true`, der PDF-Builder mit der Lauf-Sprache.

use super::normalized::NormalizedReport;

/// Einstufung einer Risiko-Dimension.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskTier {
    High,
    Medium,
    Low,
    /// Modul wurde nicht ausgeführt — keine Aussage möglich.
    Unknown,
}

impl RiskTier {
    /// Kanonischer Wert im JSON.
    pub fn as_str(self) -> &'static str {
        match self {
            RiskTier::High => "high",
            RiskTier::Medium => "medium",
            RiskTier::Low => "low",
            RiskTier::Unknown => "unknown",
        }
    }

    /// Status-Token der Checklist-Komponente im PDF.
    pub fn status(self) -> &'static str {
        match self {
            RiskTier::High => "bad",
            RiskTier::Medium => "warn",
            RiskTier::Low => "good",
            RiskTier::Unknown => "info",
        }
    }

    fn from_optional_score(score: Option<u32>) -> Self {
        match score {
            Some(score) if score < 60 => RiskTier::High,
            Some(score) if score < 80 => RiskTier::Medium,
            Some(_) => RiskTier::Low,
            None => RiskTier::Unknown,
        }
    }
}

/// Eine Risiko-Dimension mit genau den Rohwerten, aus denen ihr Text entsteht.
#[derive(Debug, Clone)]
pub enum ManagementRiskKind {
    Legal {
        legal_flags: usize,
        /// Open findings with BFSG/EAA relevance high or medium — the one
        /// definition shared with the screen-reader callout and the finding
        /// rows (#952-2). A superset of `legal_flags`.
        bfsg_relevant: usize,
        critical: usize,
        high: usize,
        blocking_issues: usize,
        /// Teilmenge von `blocking_issues`, die per Tastatur nicht bedienbar
        /// ist (2.1.1). Der Rest hat keinen Namen und bleibt bedienbar.
        keyboard_blocking: usize,
    },
    AssistiveTechnology {
        critical: usize,
        high: usize,
        blocking_issues: usize,
        keyboard_blocking: usize,
    },
    Conversion {
        accessibility: u32,
        performance: Option<u32>,
        mobile: Option<u32>,
        blocking_issues: usize,
        keyboard_blocking: usize,
    },
    Seo {
        score: Option<u32>,
    },
    PerformanceMobile {
        performance: Option<u32>,
        mobile: Option<u32>,
        /// Open findings of High or Critical severity in the performance and
        /// mobile modules. A good module score does not hide them (#952-3).
        high_findings: usize,
    },
    Trust {
        accessibility: u32,
        critical: usize,
        high: usize,
    },
    Project {
        count: usize,
        /// `Some(n)` bei einem Mehrseiten-Lauf über `n` Seiten, `None` bei
        /// einer einzelnen URL. Nur im ersten Fall ist eine Aussage über
        /// Template oder Komponente belegt (Plan 40).
        pages: Option<usize>,
    },
}

impl ManagementRiskKind {
    pub fn tier(&self) -> RiskTier {
        match self {
            ManagementRiskKind::Legal {
                legal_flags,
                bfsg_relevant,
                critical,
                high,
                blocking_issues,
                ..
            } => {
                if *legal_flags > 0 || *critical > 0 || *blocking_issues >= 5 {
                    RiskTier::High
                } else if *bfsg_relevant > 0 || *high > 0 || *blocking_issues > 0 {
                    RiskTier::Medium
                } else {
                    RiskTier::Low
                }
            }
            ManagementRiskKind::AssistiveTechnology {
                critical,
                high,
                blocking_issues,
                ..
            } => {
                if *critical > 0 || *blocking_issues >= 5 {
                    RiskTier::High
                } else if *high > 0 || *blocking_issues > 0 {
                    RiskTier::Medium
                } else {
                    RiskTier::Low
                }
            }
            ManagementRiskKind::Conversion {
                accessibility,
                performance,
                mobile,
                blocking_issues,
                ..
            } => {
                if *accessibility < 60
                    || performance.is_some_and(|s| s < 50)
                    || *blocking_issues >= 5
                {
                    RiskTier::High
                } else if *accessibility < 80
                    || mobile.is_some_and(|s| s < 75)
                    || *blocking_issues > 0
                {
                    RiskTier::Medium
                } else {
                    RiskTier::Low
                }
            }
            ManagementRiskKind::Seo { score } => RiskTier::from_optional_score(*score),
            ManagementRiskKind::PerformanceMobile {
                performance,
                mobile,
                high_findings,
            } => {
                // The weaker of the two drives the tier — a fast page that is
                // unusable on a phone is not a low risk.
                let worst = match (performance, mobile) {
                    (Some(p), Some(m)) => Some((*p).min(*m)),
                    (Some(p), None) => Some(*p),
                    (None, Some(m)) => Some(*m),
                    (None, None) => None,
                };
                match RiskTier::from_optional_score(worst) {
                    RiskTier::Low if *high_findings > 0 => RiskTier::Medium,
                    tier => tier,
                }
            }
            ManagementRiskKind::Trust {
                accessibility,
                critical,
                high,
            } => {
                if *critical > 0 || *accessibility < 50 {
                    RiskTier::High
                } else if *high > 0 || *accessibility < 75 {
                    RiskTier::Medium
                } else {
                    RiskTier::Low
                }
            }
            ManagementRiskKind::Project { count, .. } => {
                if *count >= 3 {
                    RiskTier::High
                } else if *count > 0 {
                    RiskTier::Medium
                } else {
                    RiskTier::Low
                }
            }
        }
    }

    /// Name der Dimension. Einzige Quelle — JSON ruft mit `en = true`.
    pub fn dimension(&self, en: bool) -> String {
        match self {
            ManagementRiskKind::Legal { .. } => {
                if en {
                    "Legal / BFSG-EAA"
                } else {
                    "Rechtliches / BFSG-EAA"
                }
            }
            ManagementRiskKind::AssistiveTechnology { .. } => {
                if en {
                    "Assistive technology use"
                } else {
                    "Nutzung mit Hilfsmitteln"
                }
            }
            ManagementRiskKind::Conversion { .. } => {
                if en {
                    "Conversion / usability"
                } else {
                    "Conversion & Nutzbarkeit"
                }
            }
            ManagementRiskKind::Seo { .. } => {
                if en {
                    "SEO / visibility"
                } else {
                    "SEO & Sichtbarkeit"
                }
            }
            ManagementRiskKind::PerformanceMobile { .. } => {
                if en {
                    "Performance & mobile"
                } else {
                    "Ladezeit & Mobilfreundlichkeit"
                }
            }
            ManagementRiskKind::Trust { .. } => {
                if en {
                    "Trust / brand"
                } else {
                    "Markenwirkung (aus Barrierefreiheit abgeleitet)"
                }
            }
            ManagementRiskKind::Project { .. } => {
                if en {
                    "Project risk"
                } else {
                    "Projektrisiko"
                }
            }
        }
        .to_string()
    }

    /// Begründung. Einzige Quelle — nennt immer die Werte, die die Einstufung
    /// gesetzt haben, nie eine Wirkung ohne Messgrundlage (Plan 40).
    pub fn rationale(&self, en: bool) -> String {
        match self {
            ManagementRiskKind::Legal {
                legal_flags,
                bfsg_relevant,
                critical,
                high,
                blocking_issues,
                keyboard_blocking,
            } => {
                // One count for "possible legal relevance": the open findings
                // the finding rows mark Medium/High (never fewer than the
                // legal flags, which also include screen-reader blockers).
                let legal_flags = &(*bfsg_relevant).max(*legal_flags);
                let base = if en {
                    format!(
                        "{legal_flags} {} with possible legal relevance, {critical} critical and {high} high WCAG findings detected automatically.",
                        if *legal_flags == 1 { "finding" } else { "findings" }
                    )
                } else {
                    format!(
                        "{legal_flags} {} mit möglicher Rechtsrelevanz, {critical} kritische und {high} hohe WCAG-Befunde automatisiert erkannt.",
                        if *legal_flags == 1 { "Befund" } else { "Befunde" }
                    )
                };
                if *blocking_issues == 0 {
                    return base;
                }
                let controls = blocking_controls_text(*blocking_issues, *keyboard_blocking, en);
                if en {
                    format!("{base} In addition: {controls} (WCAG 4.1.2/2.1.1, also referenced by BFSG/EAA).")
                } else {
                    format!("{base} Zusätzlich: {controls} (WCAG 4.1.2/2.1.1, auf die auch BFSG/EAA verweisen).")
                }
            }
            ManagementRiskKind::AssistiveTechnology {
                critical,
                high,
                blocking_issues,
                keyboard_blocking,
            } => {
                if *critical == 0 && *high == 0 && *blocking_issues == 0 {
                    return if en {
                        "No blocking barrier for screen reader or keyboard use detected within the automated scope.".to_string()
                    } else {
                        "Im automatisierten Prüfumfang keine blockierende Barriere für Screenreader- oder Tastaturnutzung erkannt.".to_string()
                    };
                }
                // Blockers are counted per occurrence and can be medium
                // findings, so they are no subset of the critical/high count
                // ("0 critical and 0 high findings, 3 of them …" on gov.uk).
                if *blocking_issues == 0 {
                    return if en {
                        format!("{critical} critical and {high} high findings.")
                    } else {
                        format!("{critical} kritische und {high} hohe Befunde.")
                    };
                }
                let controls = blocking_controls_text(*blocking_issues, *keyboard_blocking, en);
                let consequence =
                    blocking_consequence_text(*blocking_issues, *keyboard_blocking, en);
                if en {
                    format!(
                        "{critical} critical and {high} high findings; {controls}. {consequence}"
                    )
                } else {
                    format!(
                        "{critical} kritische und {high} hohe Befunde; {controls}. {consequence}"
                    )
                }
            }
            ManagementRiskKind::Conversion {
                accessibility,
                performance,
                mobile,
                blocking_issues,
                keyboard_blocking,
            } => {
                let perf = score_or_not_measured(*performance, en);
                let mob = score_or_not_measured(*mobile, en);
                let base = if en {
                    format!("Accessibility {accessibility}/100, performance {perf}, mobile {mob}.")
                } else {
                    format!(
                        "Barrierefreiheit {accessibility}/100, Performance {perf}, Mobile {mob}."
                    )
                };
                if *blocking_issues == 0 {
                    return base;
                }
                let controls = blocking_controls_text(*blocking_issues, *keyboard_blocking, en);
                // A nameless control still works; only a keyboard blocker can
                // stop a task, and no task was run to completion (plan 99).
                if en {
                    format!(
                        "{base} {controls}: tasks routed through them are harder to complete with assistive technology. Whether a task is fully blocked was not tested."
                    )
                } else {
                    format!(
                        "{base} {controls}: Abläufe, die darüber führen, sind mit Hilfsmitteln erschwert. Ob ein Ablauf vollständig blockiert ist, wurde nicht geprüft."
                    )
                }
            }
            ManagementRiskKind::Seo { score } => match score {
                Some(score) => {
                    if en {
                        format!("SEO score {score}/100.")
                    } else {
                        format!("SEO-Score {score}/100.")
                    }
                }
                None => {
                    if en {
                        "SEO module was not run.".to_string()
                    } else {
                        "Das SEO-Modul wurde nicht ausgeführt.".to_string()
                    }
                }
            },
            ManagementRiskKind::PerformanceMobile {
                performance,
                mobile,
                high_findings,
            } => {
                if performance.is_none() && mobile.is_none() {
                    return if en {
                        "Neither the performance nor the mobile module was run.".to_string()
                    } else {
                        "Weder das Performance- noch das Mobile-Modul wurde ausgeführt.".to_string()
                    };
                }
                let perf = score_or_not_measured(*performance, en);
                let mob = score_or_not_measured(*mobile, en);
                let base = if en {
                    format!("Performance {perf}, mobile usability {mob} (lab measurement).")
                } else {
                    format!("Performance {perf}, mobile Nutzbarkeit {mob} (Labormessung).")
                };
                match (*high_findings, en) {
                    (0, _) => base,
                    (n, true) => format!(
                        "{base} {n} high-severity finding{} in these modules.",
                        if n == 1 { "" } else { "s" }
                    ),
                    (n, false) => format!(
                        "{base} {n} {} mit hohem Schweregrad in diesen Modulen.",
                        if n == 1 { "Befund" } else { "Befunde" }
                    ),
                }
            }
            ManagementRiskKind::Trust {
                accessibility,
                critical,
                high,
            } => {
                if en {
                    format!(
                        "Inferred from accessibility only: score {accessibility}/100 with {critical} critical and {high} high findings. No trust or brand signal is measured directly."
                    )
                } else {
                    format!(
                        "Nur aus der Barrierefreiheit abgeleitet: Score {accessibility}/100 mit {critical} kritischen und {high} hohen Befunden. Ein Vertrauens- oder Markensignal wird nicht direkt gemessen."
                    )
                }
            }
            ManagementRiskKind::Project { count, pages } => match pages {
                Some(pages) => {
                    if en {
                        format!(
                            "{count} rule{} recur on two or more of the {pages} audited pages — likely component or template issues needing coordinated remediation.",
                            if *count == 1 { "" } else { "s" }
                        )
                    } else {
                        format!(
                            "{count} Regel{} treten auf zwei oder mehr der {pages} geprüften Seiten auf — vermutlich Komponenten- oder Template-Fehler, die gebündelt behoben werden sollten.",
                            if *count == 1 { "" } else { "n" }
                        )
                    }
                }
                None => {
                    if en {
                        format!(
                            "{count} finding{} {} 10 or more elements on this page or {} a structural fix. A single page cannot show whether a template or component is the cause — audit further pages to confirm.",
                            if *count == 1 { "" } else { "s" },
                            if *count == 1 { "affects" } else { "affect" },
                            if *count == 1 { "needs" } else { "need" }
                        )
                    } else {
                        format!(
                            "{count} Befund{} {} 10 oder mehr Elemente dieser Seite oder {} eine strukturelle Änderung. Eine einzelne Seite zeigt nicht, ob Template oder Komponente die Ursache sind — dafür weitere Seiten prüfen.",
                            if *count == 1 { "" } else { "e" },
                            if *count == 1 { "betrifft" } else { "betreffen" },
                            if *count == 1 { "erfordert" } else { "erfordern" }
                        )
                    }
                }
            },
        }
    }
}

/// Die blockierenden Bedienelemente, nach Ursache getrennt: ohne Namen oder
/// Rolle (bleibt bedienbar, Zweck für Screenreader unklar) und per Tastatur
/// nicht bedienbar (2.1.1). `keyboard` ist eine Teilmenge von `total`.
pub(crate) fn blocking_controls_text(total: usize, keyboard: usize, en: bool) -> String {
    let unnamed = total.saturating_sub(keyboard);
    let mut parts = Vec::with_capacity(2);
    if unnamed > 0 {
        parts.push(if en {
            format!(
                "{unnamed} {} without an accessible name or role",
                if unnamed == 1 { "control" } else { "controls" }
            )
        } else {
            format!(
                "{unnamed} {} ohne zugänglichen Namen oder Rolle",
                if unnamed == 1 {
                    "Bedienelement"
                } else {
                    "Bedienelemente"
                }
            )
        });
    }
    if keyboard > 0 {
        parts.push(if en {
            format!(
                "{keyboard} {} not operable by keyboard",
                if keyboard == 1 { "control" } else { "controls" }
            )
        } else {
            format!(
                "{keyboard} {} nicht per Tastatur bedienbar",
                if keyboard == 1 {
                    "Bedienelement"
                } else {
                    "Bedienelemente"
                }
            )
        });
    }
    parts.join(if en { " and " } else { " und " })
}

/// Was die blockierenden Bedienelemente für Hilfsmittel bedeuten — je Ursache
/// nur die Folge, die sie tatsächlich hat.
fn blocking_consequence_text(total: usize, keyboard: usize, en: bool) -> &'static str {
    let unnamed = total.saturating_sub(keyboard);
    match (unnamed > 0, keyboard > 0, en) {
        (true, false, true) => {
            "Screen readers cannot convey their purpose; no keyboard blocker was detected at those points."
        }
        (true, false, false) => {
            "Screenreader können deren Zweck nicht vermitteln; eine Tastaturblockade wurde an diesen Stellen nicht erkannt."
        }
        (false, _, true) => "Keyboard users cannot reach or trigger them.",
        (false, _, false) => "Tastaturnutzende können sie nicht erreichen oder auslösen.",
        (true, true, true) => {
            "Screen readers cannot convey the purpose of the unnamed ones; keyboard users cannot reach or trigger the others."
        }
        (true, true, false) => {
            "Screenreader können den Zweck der namenlosen nicht vermitteln; die übrigen sind per Tastatur nicht erreichbar oder auslösbar."
        }
    }
}

fn score_or_not_measured(score: Option<u32>, en: bool) -> String {
    match score {
        Some(score) => format!("{score}/100"),
        None if en => "not measured".to_string(),
        None => "nicht gemessen".to_string(),
    }
}

/// Alle Risiko-Dimensionen für einen Lauf.
pub fn build_management_risk_kinds(reports: &[NormalizedReport]) -> Vec<ManagementRiskKind> {
    let legal_flags: usize = reports.iter().map(|r| r.risk.legal_flags).sum();
    let bfsg_relevant: usize = reports
        .iter()
        .map(|r| r.findings.iter().filter(|f| f.is_bfsg_relevant()).count())
        .sum();
    let critical: usize = reports.iter().map(|r| r.severity_counts.critical).sum();
    let high: usize = reports.iter().map(|r| r.severity_counts.high).sum();
    // Controls without an accessible name (WCAG 4.1.2) can be "medium"
    // severity — so they never reach `legal_flags`, which needs High/Critical
    // — while still making a control fully inoperable. Regression
    // (satower-mosterei.de, 2026-08-31): overall risk "medium", verdict
    // "fail", 4 blocking issues, yet every dimension read "low" because this
    // field was never consulted.
    let blocking_issues: usize = reports.iter().map(|r| r.risk.blocking_issues).sum();
    let keyboard_blocking: usize = reports
        .iter()
        .map(|r| r.risk.keyboard_blocking_issues)
        .sum();

    // Averages are rounded, not truncated, like every other batch average
    // (`summary.average_score`, the module averages): a truncated 95 next to
    // a rounded 96 reads as two values (#1039).
    let average =
        |scores: &[u32]| (scores.iter().sum::<u32>() as f64 / scores.len() as f64).round() as u32;
    let accessibility = if reports.is_empty() {
        0
    } else {
        average(&reports.iter().map(|r| r.score).collect::<Vec<_>>())
    };
    let module = |name: &str| -> Option<u32> {
        let scores: Vec<u32> = reports
            .iter()
            .filter_map(|report| {
                report
                    .module_scores
                    .iter()
                    .find(|module| module.name == name)
                    .map(|module| module.score)
            })
            .collect();
        if scores.is_empty() {
            None
        } else {
            Some(average(&scores))
        }
    };
    let performance = module("Performance");
    let mobile = module("Mobile");
    let seo = module("SEO");

    // A template or component defect can only be inferred from recurrence
    // across pages; within one page, repetition is one page repeating itself
    // (plan 40).
    let (project_count, pages) = if reports.len() > 1 {
        let mut pages_per_rule: std::collections::HashMap<&str, usize> =
            std::collections::HashMap::new();
        for report in reports {
            let mut seen = std::collections::HashSet::new();
            for finding in &report.findings {
                if seen.insert(finding.rule_id.as_str()) {
                    *pages_per_rule.entry(finding.rule_id.as_str()).or_default() += 1;
                }
            }
        }
        (
            pages_per_rule.values().filter(|p| **p >= 2).count(),
            Some(reports.len()),
        )
    } else {
        (
            reports
                .iter()
                .flat_map(|r| r.findings.iter())
                .filter(|f| f.occurrence_count >= 10 || f.complexity == "high")
                .count(),
            None,
        )
    };

    vec![
        ManagementRiskKind::Legal {
            legal_flags,
            bfsg_relevant,
            critical,
            high,
            blocking_issues,
            keyboard_blocking,
        },
        ManagementRiskKind::AssistiveTechnology {
            critical,
            high,
            blocking_issues,
            keyboard_blocking,
        },
        ManagementRiskKind::Conversion {
            accessibility,
            performance,
            mobile,
            blocking_issues,
            keyboard_blocking,
        },
        ManagementRiskKind::Seo { score: seo },
        ManagementRiskKind::PerformanceMobile {
            performance,
            mobile,
            high_findings: reports
                .iter()
                .flat_map(|r| r.findings.iter())
                .filter(|f| {
                    matches!(
                        f.contract.domain,
                        crate::audit::finding_contract::FindingDomain::Mobile
                            | crate::audit::finding_contract::FindingDomain::Performance
                    ) && f.severity >= crate::taxonomy::Severity::High
                })
                .count(),
        },
        ManagementRiskKind::Trust {
            accessibility,
            critical,
            high,
        },
        ManagementRiskKind::Project {
            count: project_count,
            pages,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// #952-4: the trust risk is derived from accessibility alone, so its
    /// German name must not read like the measured "Vertrauenssignale" score.
    #[test]
    fn trust_risk_is_not_named_like_the_trust_signal_score() {
        let trust = ManagementRiskKind::Trust {
            accessibility: 93,
            critical: 0,
            high: 1,
        };
        assert!(!trust.dimension(false).contains("Vertrauen"));
        assert_eq!(trust.dimension(true), "Trust / brand");
    }

    /// #952-2/#952-3: an open finding the finding rows mark BFSG-relevant, or a
    /// high finding in the performance/mobile modules, is never a "low" card
    /// next to a good module score.
    #[test]
    fn open_findings_lift_a_card_that_scores_alone_would_leave_low() {
        let legal = |bfsg_relevant| ManagementRiskKind::Legal {
            legal_flags: 0,
            bfsg_relevant,
            critical: 0,
            high: 0,
            blocking_issues: 0,
            keyboard_blocking: 0,
        };
        assert_eq!(legal(0).tier(), RiskTier::Low);
        assert_eq!(legal(1).tier(), RiskTier::Medium);
        assert!(legal(1)
            .rationale(false)
            .starts_with("1 Befund mit möglicher"));
        assert!(legal(2)
            .rationale(true)
            .starts_with("2 findings with possible"));

        let perf = |high_findings| ManagementRiskKind::PerformanceMobile {
            performance: Some(82),
            mobile: Some(80),
            high_findings,
        };
        assert_eq!(perf(0).tier(), RiskTier::Low);
        assert_eq!(perf(1).tier(), RiskTier::Medium);
        assert!(perf(1)
            .rationale(false)
            .contains("1 Befund mit hohem Schweregrad"));
    }

    /// Plan 99: seven nameless links are not "inoperable". The texts name the
    /// cause that was measured and claim no blocked task.
    #[test]
    fn nameless_controls_are_not_called_inoperable() {
        let kinds = [
            ManagementRiskKind::Legal {
                legal_flags: 2,
                bfsg_relevant: 2,
                critical: 1,
                high: 2,
                blocking_issues: 7,
                keyboard_blocking: 0,
            },
            ManagementRiskKind::AssistiveTechnology {
                critical: 1,
                high: 2,
                blocking_issues: 7,
                keyboard_blocking: 0,
            },
            ManagementRiskKind::Conversion {
                accessibility: 49,
                performance: Some(76),
                mobile: Some(80),
                blocking_issues: 7,
                keyboard_blocking: 0,
            },
        ];
        for kind in &kinds {
            let en = kind.rationale(true);
            let de = kind.rationale(false);
            assert!(
                en.contains("7 controls without an accessible name or role"),
                "{en}"
            );
            assert!(
                de.contains("7 Bedienelemente ohne zugänglichen Namen oder Rolle"),
                "{de}"
            );
            for text in [&en, &de] {
                for banned in [
                    "cannot be operated",
                    "cannot be completed",
                    "not operable by keyboard",
                    "nicht bedienen",
                    "nicht abschließbar",
                    "nicht per Tastatur",
                ] {
                    assert!(!text.contains(banned), "{banned:?} in {text}");
                }
            }
            assert!(!en.chars().any(|c| "äöüÄÖÜß".contains(c)), "{en}");
        }
        assert!(
            kinds[2].rationale(true).contains("was not tested"),
            "conversion must say a blocked task was not tested"
        );
    }

    #[test]
    fn keyboard_blockers_are_named_separately() {
        let kind = ManagementRiskKind::AssistiveTechnology {
            critical: 0,
            high: 1,
            blocking_issues: 3,
            keyboard_blocking: 1,
        };
        assert_eq!(
            kind.rationale(true),
            "0 critical and 1 high findings; 2 controls without an accessible name or role and 1 control not operable by keyboard. Screen readers cannot convey the purpose of the unnamed ones; keyboard users cannot reach or trigger the others."
        );
        let only_keyboard = ManagementRiskKind::Conversion {
            accessibility: 70,
            performance: None,
            mobile: None,
            blocking_issues: 2,
            keyboard_blocking: 2,
        };
        let de = only_keyboard.rationale(false);
        assert!(
            de.contains("2 Bedienelemente nicht per Tastatur bedienbar"),
            "{de}"
        );
        assert!(!de.contains("ohne zugänglichen Namen"), "{de}");
    }

    /// Batch averages are rounded: pages at 95 and 96 average 95.5, which
    /// is 96 as every other batch average shows it, not a truncated 95 (#1039).
    #[test]
    fn conversion_averages_are_rounded_not_truncated() {
        let page = |score: u32, mobile: u32| {
            let report = crate::audit::AuditReport::new(
                "https://example.com".into(),
                crate::model::WcagLevel::AA,
                crate::wcag::WcagResults::new(),
                100,
            );
            let mut n = crate::audit::normalized::normalize(&report).normalized;
            n.score = score;
            n.module_scores = vec![crate::audit::normalized::ModuleScoreEntry::new(
                "Mobile", mobile, "measured", true,
            )];
            n
        };
        let kinds = build_management_risk_kinds(&[page(95, 80), page(96, 81)]);
        let conversion = kinds
            .iter()
            .find_map(|kind| match kind {
                ManagementRiskKind::Conversion {
                    accessibility,
                    mobile,
                    ..
                } => Some((*accessibility, *mobile)),
                _ => None,
            })
            .unwrap();
        assert_eq!(conversion, (96, Some(81)));
    }
}
