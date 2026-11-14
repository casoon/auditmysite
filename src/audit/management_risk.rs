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
use crate::i18n::{count_text, text, text_args};
use fluent_bundle::FluentValue;

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
        text(
            match self {
                ManagementRiskKind::Legal { .. } => "audit-risk-dimension-legal",
                ManagementRiskKind::AssistiveTechnology { .. } => "audit-risk-dimension-assistive",
                ManagementRiskKind::Conversion { .. } => "audit-risk-dimension-conversion",
                ManagementRiskKind::Seo { .. } => "audit-risk-dimension-seo",
                ManagementRiskKind::PerformanceMobile { .. } => {
                    "audit-risk-dimension-performance-mobile"
                }
                ManagementRiskKind::Trust { .. } => "audit-risk-dimension-trust",
                ManagementRiskKind::Project { .. } => "audit-risk-dimension-project",
            },
            en,
        )
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
                let legal_flags = (*bfsg_relevant).max(*legal_flags);
                let base = text_args(
                    "audit-risk-legal-base",
                    &[
                        ("count", number(legal_flags)),
                        ("critical", number(*critical)),
                        ("high", number(*high)),
                    ],
                    en,
                );
                if *blocking_issues == 0 {
                    return base;
                }
                let controls = blocking_controls_text(*blocking_issues, *keyboard_blocking, en);
                text_args(
                    "audit-risk-legal-controls",
                    &[
                        ("base", FluentValue::from(base)),
                        ("controls", FluentValue::from(controls)),
                    ],
                    en,
                )
            }
            ManagementRiskKind::AssistiveTechnology {
                critical,
                high,
                blocking_issues,
                keyboard_blocking,
            } => {
                if *critical == 0 && *high == 0 && *blocking_issues == 0 {
                    return text("audit-risk-assistive-none", en);
                }
                // Blockers are counted per occurrence and can be medium
                // findings, so they are no subset of the critical/high count
                // ("0 critical and 0 high findings, 3 of them …" on gov.uk).
                if *blocking_issues == 0 {
                    return text_args(
                        "audit-risk-assistive-findings",
                        &[("critical", number(*critical)), ("high", number(*high))],
                        en,
                    );
                }
                let controls = blocking_controls_text(*blocking_issues, *keyboard_blocking, en);
                let consequence =
                    blocking_consequence_text(*blocking_issues, *keyboard_blocking, en);
                text_args(
                    "audit-risk-assistive-blocking",
                    &[
                        ("critical", number(*critical)),
                        ("high", number(*high)),
                        ("controls", FluentValue::from(controls)),
                        ("consequence", FluentValue::from(consequence)),
                    ],
                    en,
                )
            }
            ManagementRiskKind::Conversion {
                accessibility,
                performance,
                mobile,
                blocking_issues,
                keyboard_blocking,
            } => {
                let base = text_args(
                    "audit-risk-conversion-base",
                    &[
                        ("accessibility", number(*accessibility)),
                        (
                            "performance",
                            FluentValue::from(score_or_not_measured(*performance, en)),
                        ),
                        (
                            "mobile",
                            FluentValue::from(score_or_not_measured(*mobile, en)),
                        ),
                    ],
                    en,
                );
                if *blocking_issues == 0 {
                    return base;
                }
                let controls = blocking_controls_text(*blocking_issues, *keyboard_blocking, en);
                // A nameless control still works; only a keyboard blocker can
                // stop a task, and no task was run to completion (plan 99).
                text_args(
                    "audit-risk-conversion-blocking",
                    &[
                        ("base", FluentValue::from(base)),
                        ("controls", FluentValue::from(controls)),
                    ],
                    en,
                )
            }
            ManagementRiskKind::Seo { score } => match score {
                Some(score) => text_args("audit-risk-seo-score", &[("score", number(*score))], en),
                None => text("audit-risk-seo-not-run", en),
            },
            ManagementRiskKind::PerformanceMobile {
                performance,
                mobile,
                high_findings,
            } => {
                if performance.is_none() && mobile.is_none() {
                    return text("audit-risk-performance-mobile-none", en);
                }
                let base = text_args(
                    "audit-risk-performance-mobile-base",
                    &[
                        (
                            "performance",
                            FluentValue::from(score_or_not_measured(*performance, en)),
                        ),
                        (
                            "mobile",
                            FluentValue::from(score_or_not_measured(*mobile, en)),
                        ),
                    ],
                    en,
                );
                match *high_findings {
                    0 => base,
                    n => text_args(
                        "audit-risk-performance-mobile-high",
                        &[("base", FluentValue::from(base)), ("count", number(n))],
                        en,
                    ),
                }
            }
            ManagementRiskKind::Trust {
                accessibility,
                critical,
                high,
            } => text_args(
                "audit-risk-trust",
                &[
                    ("accessibility", number(*accessibility)),
                    ("critical", number(*critical)),
                    ("high", number(*high)),
                ],
                en,
            ),
            ManagementRiskKind::Project { count, pages } => match pages {
                Some(pages) => text_args(
                    "audit-risk-project-recurring",
                    &[("count", number(*count)), ("pages", number(*pages))],
                    en,
                ),
                None => text_args(
                    "audit-risk-project-single",
                    &[("count", number(*count))],
                    en,
                ),
            },
        }
    }
}

/// A count or score as a Fluent number, so a plural selector can use it.
fn number(value: impl TryInto<i64>) -> FluentValue<'static> {
    FluentValue::from(value.try_into().unwrap_or(i64::MAX))
}

/// Die blockierenden Bedienelemente, nach Ursache getrennt: ohne Namen oder
/// Rolle (bleibt bedienbar, Zweck für Screenreader unklar) und per Tastatur
/// nicht bedienbar (2.1.1). `keyboard` ist eine Teilmenge von `total`.
pub(crate) fn blocking_controls_text(total: usize, keyboard: usize, en: bool) -> String {
    let unnamed = total.saturating_sub(keyboard);
    let mut parts = Vec::with_capacity(2);
    if unnamed > 0 {
        parts.push(count_text("audit-controls-unnamed", unnamed, en));
    }
    if keyboard > 0 {
        parts.push(count_text("audit-controls-keyboard", keyboard, en));
    }
    parts.join(&text("audit-join-and", en))
}

/// Was die blockierenden Bedienelemente für Hilfsmittel bedeuten — je Ursache
/// nur die Folge, die sie tatsächlich hat.
fn blocking_consequence_text(total: usize, keyboard: usize, en: bool) -> String {
    let unnamed = total.saturating_sub(keyboard);
    text(
        match (unnamed > 0, keyboard > 0) {
            (true, false) => "audit-controls-consequence-unnamed",
            (false, _) => "audit-controls-consequence-keyboard",
            (true, true) => "audit-controls-consequence-both",
        },
        en,
    )
}

fn score_or_not_measured(score: Option<u32>, en: bool) -> String {
    match score {
        Some(score) => format!("{score}/100"),
        None => text("audit-not-measured", en),
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
