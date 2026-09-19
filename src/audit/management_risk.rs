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
        critical: usize,
        high: usize,
        blocking_issues: usize,
    },
    AssistiveTechnology {
        critical: usize,
        high: usize,
        blocking_issues: usize,
    },
    Conversion {
        accessibility: u32,
        performance: Option<u32>,
        mobile: Option<u32>,
        blocking_issues: usize,
    },
    Seo {
        score: Option<u32>,
    },
    PerformanceMobile {
        performance: Option<u32>,
        mobile: Option<u32>,
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
                critical,
                high,
                blocking_issues,
            } => {
                if *legal_flags > 0 || *critical > 0 || *blocking_issues >= 5 {
                    RiskTier::High
                } else if *high > 0 || *blocking_issues > 0 {
                    RiskTier::Medium
                } else {
                    RiskTier::Low
                }
            }
            ManagementRiskKind::AssistiveTechnology {
                critical,
                high,
                blocking_issues,
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
            } => {
                // The weaker of the two drives the tier — a fast page that is
                // unusable on a phone is not a low risk.
                let worst = match (performance, mobile) {
                    (Some(p), Some(m)) => Some((*p).min(*m)),
                    (Some(p), None) => Some(*p),
                    (None, Some(m)) => Some(*m),
                    (None, None) => None,
                };
                RiskTier::from_optional_score(worst)
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
                    "Vertrauen & Marke"
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
                critical,
                high,
                blocking_issues,
            } => {
                let base = if en {
                    format!(
                        "{legal_flags} legal flags, {critical} critical and {high} high WCAG findings detected automatically."
                    )
                } else {
                    format!(
                        "{legal_flags} rechtlich relevante Marker, {critical} kritische und {high} hohe WCAG-Befunde automatisiert erkannt."
                    )
                };
                if *blocking_issues == 0 {
                    return base;
                }
                if en {
                    format!(
                        "{base} {blocking_issues} blocking interaction {} (missing accessible name/role) also affect BFSG/EAA operability requirements.",
                        if *blocking_issues == 1 { "issue" } else { "issues" }
                    )
                } else {
                    format!(
                        "{base} Zusätzlich {blocking_issues} blockierende{} Bedienelement{} ohne zugänglichen Namen oder Rolle — betrifft die BFSG/EAA-Anforderungen an die Bedienbarkeit.",
                        if *blocking_issues == 1 { "s" } else { "" },
                        if *blocking_issues == 1 { "" } else { "e" }
                    )
                }
            }
            ManagementRiskKind::AssistiveTechnology {
                critical,
                high,
                blocking_issues,
            } => {
                if *critical == 0 && *high == 0 && *blocking_issues == 0 {
                    return if en {
                        "No blocking barrier for screen reader or keyboard use detected within the automated scope.".to_string()
                    } else {
                        "Im automatisierten Prüfumfang keine blockierende Barriere für Screenreader- oder Tastaturnutzung erkannt.".to_string()
                    };
                }
                if en {
                    format!(
                        "{critical} critical and {high} high findings, {blocking_issues} of them controls without an accessible name or role — screen reader and keyboard use is obstructed at those points."
                    )
                } else {
                    format!(
                        "{critical} kritische und {high} hohe Befunde, davon {blocking_issues} Bedienelemente ohne zugänglichen Namen oder Rolle — Screenreader- und Tastaturnutzung ist an diesen Stellen behindert."
                    )
                }
            }
            ManagementRiskKind::Conversion {
                accessibility,
                performance,
                mobile,
                blocking_issues,
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
                if en {
                    format!(
                        "{base} {blocking_issues} interactive element{} cannot be operated with assistive technology, so any task routed through {} cannot be completed that way.",
                        if *blocking_issues == 1 { "" } else { "s" },
                        if *blocking_issues == 1 { "it" } else { "them" }
                    )
                } else {
                    format!(
                        "{base} {blocking_issues} Bedienelement{} lassen sich mit Hilfsmitteln nicht bedienen — Abläufe, die darüber führen, sind so nicht abschließbar.",
                        if *blocking_issues == 1 { "" } else { "e" }
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
                if en {
                    format!("Performance {perf}, mobile usability {mob} (lab measurement).")
                } else {
                    format!("Performance {perf}, mobile Nutzbarkeit {mob} (Labormessung).")
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
                            "{count} finding{} affect 10 or more elements on this page or need a structural fix. A single page cannot show whether a template or component is the cause — audit further pages to confirm.",
                            if *count == 1 { "" } else { "s" }
                        )
                    } else {
                        format!(
                            "{count} Befund{} betreffen 10 oder mehr Elemente dieser Seite oder erfordern eine strukturelle Änderung. Eine einzelne Seite zeigt nicht, ob Template oder Komponente die Ursache sind — dafür weitere Seiten prüfen.",
                            if *count == 1 { "" } else { "e" }
                        )
                    }
                }
            },
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
    let critical: usize = reports.iter().map(|r| r.severity_counts.critical).sum();
    let high: usize = reports.iter().map(|r| r.severity_counts.high).sum();
    // Controls without an accessible name (WCAG 4.1.2) can be "medium"
    // severity — so they never reach `legal_flags`, which needs High/Critical
    // — while still making a control fully inoperable. Regression
    // (satower-mosterei.de, 2026-08-31): overall risk "medium", verdict
    // "fail", 4 blocking issues, yet every dimension read "low" because this
    // field was never consulted.
    let blocking_issues: usize = reports.iter().map(|r| r.risk.blocking_issues).sum();

    let accessibility = if reports.is_empty() {
        0
    } else {
        reports.iter().map(|r| r.score).sum::<u32>() / reports.len() as u32
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
            Some(scores.iter().sum::<u32>() / scores.len() as u32)
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
            critical,
            high,
            blocking_issues,
        },
        ManagementRiskKind::AssistiveTechnology {
            critical,
            high,
            blocking_issues,
        },
        ManagementRiskKind::Conversion {
            accessibility,
            performance,
            mobile,
            blocking_issues,
        },
        ManagementRiskKind::Seo { score: seo },
        ManagementRiskKind::PerformanceMobile {
            performance,
            mobile,
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
