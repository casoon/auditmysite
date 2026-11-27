//! Management-Risiko-Dimensionen — eine Ableitung für JSON und PDF.
//!
//! Vorher existierten zwei unabhängige Modelle: `build_management_risks` für
//! das JSON und `compute_dimension_rows` in der PDF-Schicht. Sie führten
//! unterschiedliche Dimensionen, stuften unterschiedlich ein, und die
//! belegkräftigen Begründungen des JSON erreichten den PDF-Leser nie.
//!
//! Lokalisierung folgt dem #406-Muster: die Analyse backt kanonisches Englisch
//! in das gespeicherte JSON, die PDF-Präsentationsschicht leitet den Text zur
//! Laufzeit ab. [`ManagementRiskKind`] trägt die Rohwerte, und
//! `dimension`/`rationale` sind die EINZIGE Textquelle — das JSON ruft sie mit
//! `en = true`, der PDF-Builder mit der Lauf-Sprache.

use super::normalized::{NormalizedFinding, NormalizedReport};
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
        /// Findings with BFSG/EAA relevance high or medium
        /// (`is_bfsg_relevant_open`) — the one definition shared with
        /// `risk.legal_flags`, the verdict reason, the screen-reader callout
        /// and the finding rows (#952-2). The count the card prints.
        legal_flags: usize,
        /// The subset at High/Critical severity (`risk.severe_legal_flags`):
        /// the only part that lifts the tier to High.
        severe_legal_flags: usize,
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
        /// Open SEO findings of Medium severity or higher. The row exists
        /// only because of them; the score alone is not a cause (#1139).
        findings: usize,
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
        /// Reach: findings of Medium severity or higher that are structural
        /// (single report: 10+ elements or a structural fix) or recur (batch:
        /// a rule on two or more pages).
        count: usize,
        /// Severity x reach (#1228): the part of `count` that is of High or
        /// Critical severity *and* wide (single report: the same reach as
        /// `count`; batch: on at least half of the audited pages). Only this
        /// lifts the tier to High; recurrence alone never does.
        severe_wide: usize,
        /// `Some(n)` bei einem Mehrseiten-Lauf über `n` Seiten, `None` bei
        /// einer einzelnen URL. Nur im ersten Fall ist eine Aussage über
        /// Template oder Komponente belegt.
        pages: Option<usize>,
    },
}

impl ManagementRiskKind {
    pub fn tier(&self) -> RiskTier {
        match self {
            ManagementRiskKind::Legal {
                legal_flags,
                severe_legal_flags,
                critical,
                high,
                blocking_issues,
                ..
            } => {
                if *severe_legal_flags > 0 || *critical > 0 || *blocking_issues >= 5 {
                    RiskTier::High
                } else if *legal_flags > 0 || *high > 0 || *blocking_issues > 0 {
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
            // The row only exists with a Medium-or-higher SEO finding, so it
            // is never a strength, whatever the score says.
            ManagementRiskKind::Seo { score, .. } => match RiskTier::from_optional_score(*score) {
                RiskTier::Low | RiskTier::Unknown => RiskTier::Medium,
                tier => tier,
            },
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
            ManagementRiskKind::Project {
                count, severe_wide, ..
            } => {
                if *severe_wide > 0 {
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
    /// gesetzt haben, nie eine Wirkung ohne Messgrundlage.
    pub fn rationale(&self, en: bool) -> String {
        match self {
            ManagementRiskKind::Legal {
                legal_flags,
                critical,
                high,
                blocking_issues,
                keyboard_blocking,
                ..
            } => {
                let base = text_args(
                    "audit-risk-legal-base",
                    &[
                        ("count", number(*legal_flags)),
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
                blocking_issues,
                keyboard_blocking,
                ..
            } => {
                let controls = blocking_controls_text(*blocking_issues, *keyboard_blocking, en);
                // A nameless control still works; only a keyboard blocker can
                // stop a task, and no task was run to completion.
                text_args(
                    "audit-risk-conversion-blocking",
                    &[("controls", FluentValue::from(controls))],
                    en,
                )
            }
            ManagementRiskKind::Seo { score, findings } => text_args(
                "audit-risk-seo-findings",
                &[
                    ("count", number(*findings)),
                    (
                        "score",
                        FluentValue::from(score_or_not_measured(*score, en)),
                    ),
                ],
                en,
            ),
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
            ManagementRiskKind::Project {
                count,
                severe_wide,
                pages,
            } => match pages {
                Some(pages) => text_args(
                    "audit-risk-project-recurring",
                    &[
                        ("count", number(*count)),
                        ("pages", number(*pages)),
                        ("severe", number(*severe_wide)),
                    ],
                    en,
                ),
                None => text_args(
                    "audit-risk-project-single",
                    &[("count", number(*count)), ("severe", number(*severe_wide))],
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

/// Whether a finding can make a project risk: Medium severity or higher.
/// Single report and batch use this one predicate (#1039).
fn counts_for_project_risk(finding: &NormalizedFinding) -> bool {
    finding.severity >= crate::taxonomy::Severity::Medium
}

/// Alle Risiko-Dimensionen für einen Lauf.
pub fn build_management_risk_kinds(reports: &[NormalizedReport]) -> Vec<ManagementRiskKind> {
    let legal_flags: usize = reports.iter().map(|r| r.risk.legal_flags).sum();
    let severe_legal_flags: usize = reports.iter().map(|r| r.risk.severe_legal_flags).sum();
    let critical: usize = reports.iter().map(|r| r.severity_counts.critical).sum();
    let high: usize = reports.iter().map(|r| r.severity_counts.high).sum();
    // Controls without an accessible name (WCAG 4.1.2) can be "medium"
    // severity — so they need not reach `legal_flags` — while still making a
    // control fully inoperable. Regression
    // (a shop site, 2026-08-31): overall risk "medium", verdict
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
    let seo_findings = reports
        .iter()
        .flat_map(|r| r.findings.iter())
        .filter(|f| {
            f.contract.domain == crate::audit::finding_contract::FindingDomain::Seo
                && f.severity >= crate::taxonomy::Severity::Medium
        })
        .count();

    // A template or component defect can only be inferred from recurrence
    // across pages; within one page, repetition is one page repeating itself.
    // Only findings of Medium severity or higher count: a Low finding that
    // recurs on every page (fonts not preloaded, static caching) is polish and
    // no reason for "High" project risk (#1039). One predicate for both paths.
    let (project_count, severe_wide, pages) = if reports.len() > 1 {
        // Per rule: on how many pages it occurs and its highest severity.
        let mut per_rule: std::collections::HashMap<&str, (usize, crate::taxonomy::Severity)> =
            std::collections::HashMap::new();
        for report in reports {
            let mut seen = std::collections::HashSet::new();
            for finding in report
                .findings
                .iter()
                .filter(|f| counts_for_project_risk(f))
            {
                let entry = per_rule
                    .entry(finding.rule_id.as_str())
                    .or_insert((0, finding.severity));
                entry.1 = entry.1.max(finding.severity);
                if seen.insert(finding.rule_id.as_str()) {
                    entry.0 += 1;
                }
            }
        }
        let total_pages = reports.len();
        (
            per_rule.values().filter(|(p, _)| *p >= 2).count(),
            per_rule
                .values()
                .filter(|(p, severity)| {
                    *p >= 2 && p * 2 >= total_pages && *severity >= crate::taxonomy::Severity::High
                })
                .count(),
            Some(total_pages),
        )
    } else {
        let structural = || {
            reports
                .iter()
                .flat_map(|r| r.findings.iter())
                .filter(|f| counts_for_project_risk(f))
                .filter(|f| f.occurrence_count >= 10 || f.complexity == "high")
        };
        (
            structural().count(),
            structural()
                .filter(|f| f.severity >= crate::taxonomy::Severity::High)
                .count(),
            None,
        )
    };

    // A dimension that would only restate a module score is not a risk row:
    // the score is on the cover already. Conversion, SEO and trust appear only
    // with the findings that justify them (#1139).
    let mut kinds = vec![
        ManagementRiskKind::Legal {
            legal_flags,
            severe_legal_flags,
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
    ];
    if blocking_issues > 0 {
        kinds.push(ManagementRiskKind::Conversion {
            accessibility,
            performance,
            mobile,
            blocking_issues,
            keyboard_blocking,
        });
    }
    if seo_findings > 0 {
        kinds.push(ManagementRiskKind::Seo {
            score: seo,
            findings: seo_findings,
        });
    }
    kinds.push(ManagementRiskKind::PerformanceMobile {
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
    });
    if critical > 0 || high > 0 {
        kinds.push(ManagementRiskKind::Trust {
            accessibility,
            critical,
            high,
        });
    }
    kinds.push(ManagementRiskKind::Project {
        count: project_count,
        severe_wide,
        pages,
    });
    kinds
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A page with the given findings, cloned from one real finding so the
    /// contract fields are filled: `(rule id, severity, occurrences)`.
    fn page(findings: &[(&str, crate::taxonomy::Severity, usize)]) -> NormalizedReport {
        use crate::audit::AuditReport;
        use crate::model::WcagLevel;
        use crate::wcag::{Severity, Violation, WcagResults};
        let mut results = WcagResults::new();
        results.add_violation(
            Violation::new(
                "1.4.3",
                "Contrast (Minimum)",
                WcagLevel::AA,
                Severity::High,
                "Insufficient contrast",
                "node-1",
            )
            .with_rule_id("contrast/text-insufficient"),
        );
        let report = AuditReport::new(
            "https://example.com/".to_string(),
            WcagLevel::AA,
            results,
            1,
        );
        let mut normalized = crate::audit::normalized::normalize(&report).normalized;
        let template = normalized.findings[0].clone();
        normalized.findings = findings
            .iter()
            .map(|(rule_id, severity, occurrences)| {
                let mut finding = template.clone();
                finding.rule_id = rule_id.to_string();
                finding.severity = *severity;
                finding.occurrence_count = *occurrences;
                finding.complexity = "low".to_string();
                finding
            })
            .collect();
        normalized
    }

    fn project(reports: &[NormalizedReport]) -> (usize, Option<usize>, RiskTier) {
        let (count, pages, tier, _) = project_full(reports);
        (count, pages, tier)
    }

    fn project_full(reports: &[NormalizedReport]) -> (usize, Option<usize>, RiskTier, usize) {
        let kind = build_management_risk_kinds(reports)
            .into_iter()
            .find(|k| matches!(k, ManagementRiskKind::Project { .. }))
            .expect("the project risk is in every run");
        let ManagementRiskKind::Project {
            count,
            severe_wide,
            pages,
        } = &kind
        else {
            unreachable!()
        };
        (*count, *pages, kind.tier(), *severe_wide)
    }

    /// #1039: ten Low rules that recur on every page were "High" project
    /// risk next to a clean report. Only rules of Medium severity or higher
    /// count, in the batch (recurrence) and in the single report (reach).
    #[test]
    fn project_risk_counts_only_medium_and_higher_findings() {
        use crate::taxonomy::Severity::{High, Low, Medium};
        let low_everywhere = [("l/a", Low, 1), ("l/b", Low, 1), ("l/c", Low, 1)];

        // Low rules recurring on all three pages: no project risk.
        let reports = vec![
            page(&low_everywhere),
            page(&low_everywhere),
            page(&low_everywhere),
        ];
        assert_eq!(project(&reports), (0, Some(3), RiskTier::Low));

        // Two rules recur on two of five pages, a third on one page only,
        // next to the Low ones: two recurring rules; the High one reaches
        // two of five pages only, so not wide: tier Medium.
        let reports = vec![
            page(&[
                ("m/a", Medium, 1),
                ("m/b", High, 1),
                ("m/c", Medium, 1),
                ("l/a", Low, 1),
            ]),
            page(&[("m/a", Medium, 1), ("m/b", High, 1), ("l/a", Low, 1)]),
            page(&[("l/a", Low, 1)]),
            page(&[("l/a", Low, 1)]),
            page(&[("l/a", Low, 1)]),
        ];
        assert_eq!(project_full(&reports), (2, Some(5), RiskTier::Medium, 0));

        // #1228: recurrence alone is not severity. Three Medium rules on every
        // page stay Medium, the old count of three made them High.
        let every_page = [("m/a", Medium, 1), ("m/b", Medium, 1), ("m/c", Medium, 1)];
        let reports = vec![page(&every_page), page(&every_page)];
        assert_eq!(project_full(&reports), (3, Some(2), RiskTier::Medium, 0));

        // A High rule on at least half of the pages is High, whatever else recurs.
        let reports = vec![
            page(&[("m/a", Medium, 1), ("m/b", High, 1), ("m/c", Medium, 1)]),
            page(&[("m/a", Medium, 1), ("m/b", High, 1), ("m/c", Medium, 1)]),
        ];
        assert_eq!(project_full(&reports), (3, Some(2), RiskTier::High, 1));
        let reports = vec![
            page(&[("m/b", High, 1)]),
            page(&[("m/b", High, 1)]),
            page(&[("l/a", Low, 1)]),
            page(&[("l/a", Low, 1)]),
        ];
        assert_eq!(project_full(&reports), (1, Some(4), RiskTier::High, 1));

        // Single report: reach of ten or more elements counts only from Medium.
        let single = vec![page(&[
            ("l/a", Low, 12),
            ("m/a", Medium, 12),
            ("m/b", Medium, 3),
        ])];
        assert_eq!(project(&single), (1, None, RiskTier::Medium));
        let single = vec![page(&[("l/a", Low, 12), ("l/b", Low, 40)])];
        assert_eq!(project(&single), (0, None, RiskTier::Low));

        // #1228: single report, three wide Medium findings stay Medium; one
        // wide High finding is High.
        let single = vec![page(&[
            ("m/a", Medium, 12),
            ("m/b", Medium, 20),
            ("m/c", Medium, 30),
        ])];
        assert_eq!(project_full(&single), (3, None, RiskTier::Medium, 0));
        let single = vec![page(&[("m/a", High, 12)])];
        assert_eq!(project_full(&single), (1, None, RiskTier::High, 1));
    }

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

    /// #952-2/#952-3: a legal flag (a finding the finding rows mark
    /// BFSG-relevant), or a high finding in the performance/mobile modules, is
    /// never a "low" card next to a good module score.
    #[test]
    fn open_findings_lift_a_card_that_scores_alone_would_leave_low() {
        let legal = |legal_flags, severe_legal_flags| ManagementRiskKind::Legal {
            legal_flags,
            severe_legal_flags,
            critical: 0,
            high: 0,
            blocking_issues: 0,
            keyboard_blocking: 0,
        };
        assert_eq!(legal(0, 0).tier(), RiskTier::Low);
        // Relevant, but not severe: shown and counted, tier Medium.
        assert_eq!(legal(1, 0).tier(), RiskTier::Medium);
        assert_eq!(legal(1, 1).tier(), RiskTier::High);
        assert!(legal(1, 0)
            .rationale(false)
            .starts_with("1 Befund mit möglicher"));
        assert!(legal(2, 0)
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

    /// Seven nameless links are not "inoperable". The texts name the
    /// cause that was measured and claim no blocked task.
    #[test]
    fn nameless_controls_are_not_called_inoperable() {
        let kinds = [
            ManagementRiskKind::Legal {
                legal_flags: 2,
                severe_legal_flags: 2,
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
            n.risk.blocking_issues = 1;
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
