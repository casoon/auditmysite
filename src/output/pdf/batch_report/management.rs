use renderreport::components::advanced::{ChecklistPanel, ChecklistRow, KeyValueList, List};
use renderreport::components::{AuditTable, TableColumn};
use renderreport::prelude::*;

use crate::audit::BatchReport;
use crate::i18n::I18n;
use crate::output::report_model::*;
use crate::util::truncate_url;

use super::super::helpers::{effort_label_i18n, severity_label_i18n};

// ─── Helper: Batch Report Assessment & Key Points ──────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct BatchAuditFlagSummary {
    pub(super) kind: String,
    pub(super) affected_pages: usize,
    example: String,
}

pub(super) fn aggregate_audit_flags(
    reports: &[crate::audit::AuditReport],
) -> Vec<BatchAuditFlagSummary> {
    let mut by_kind: std::collections::BTreeMap<String, (usize, String)> =
        std::collections::BTreeMap::new();

    for report in reports {
        let normalized = crate::audit::normalize(report);
        let mut seen_on_page = std::collections::BTreeSet::new();
        for flag in &normalized.normalized.audit_flags {
            if !seen_on_page.insert(flag.kind.clone()) {
                continue;
            }
            by_kind
                .entry(flag.kind.clone())
                .and_modify(|(count, _)| *count += 1)
                .or_insert((1, flag.message.clone()));
        }
    }

    let mut summaries: Vec<_> = by_kind
        .into_iter()
        .map(|(kind, (affected_pages, example))| BatchAuditFlagSummary {
            kind,
            affected_pages,
            example,
        })
        .collect();
    summaries.sort_by(|a, b| {
        b.affected_pages
            .cmp(&a.affected_pages)
            .then_with(|| a.kind.cmp(&b.kind))
    });
    summaries
}

pub(super) fn audit_flag_batch_title(kind: &str, en: bool) -> &'static str {
    match (kind, en) {
        ("consent_banner", true) => "Consent banner",
        ("consent_banner", false) => "Consent-Banner",
        ("bypass_blocks_untested", true) => "Skip link verification",
        ("bypass_blocks_untested", false) => "Skip-Link-Prüfung",
        ("conflicting_signal", true) => "Conflicting signal",
        ("conflicting_signal", false) => "Widersprüchliches Signal",
        ("viewport_gap", true) => "Desktop/mobile difference",
        ("viewport_gap", false) => "Desktop-/Mobile-Unterschied",
        ("incomplete_audit", true) => "Incomplete measurement scope",
        ("incomplete_audit", false) => "Unvollständiger Messumfang",
        ("consent_wall_artifact", true) => "Consent wall artifact",
        ("consent_wall_artifact", false) => "Consent-Wall-Artefakt",
        (_, true) => "Audit note",
        (_, false) => "Audit-Hinweis",
    }
}

pub(super) fn render_batch_audit_flags(
    builder: renderreport::engine::ReportBuilder,
    batch: &BatchReport,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let summaries = aggregate_audit_flags(&batch.reports);
    if summaries.is_empty() {
        return builder;
    }

    let en = i18n.locale() == "en";
    let mut rows = Vec::new();
    for summary in summaries {
        let pages = if en {
            format!(
                "{} affected page{}",
                summary.affected_pages,
                if summary.affected_pages == 1 { "" } else { "s" }
            )
        } else {
            format!(
                "{} betroffene Seite{}",
                summary.affected_pages,
                if summary.affected_pages == 1 { "" } else { "n" }
            )
        };
        rows.push(
            ChecklistRow::new(
                audit_flag_batch_title(&summary.kind, en),
                format!("{} — {}", pages, summary.example),
            )
            .with_status("warn"),
        );
    }

    builder.add_component(ChecklistPanel::new(rows).with_title(if en {
        "Recurring audit caveats"
    } else {
        "Wiederkehrende Audit-Hinweise"
    }))
}

/// Clear batch assessment — no score, just interpretation
pub(super) fn build_batch_assessment(
    summary: &crate::output::report_model::PortfolioSummary,
    dist: &SeverityDistribution,
    i18n: &I18n,
) -> String {
    let en = i18n.locale() == "en";
    let score = summary.average_score.round() as u32;
    // Score band is the primary signal; severity only refines wording within a
    // band. A low average must never yield a reassuring label (mirrors the
    // single-report logic in builder/single/executive.rs, #355).
    if score < 40 {
        if en {
            "Critical barriers — not WCAG conformant".to_string()
        } else {
            "Kritische Barrieren — nicht WCAG-konform".to_string()
        }
    } else if score < 60 {
        if dist.critical > 0 {
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
        if dist.critical > 0 {
            if en {
                "Usable, but legally risky".to_string()
            } else {
                "Nutzbar, aber rechtlich riskant".to_string()
            }
        } else if dist.high > 0 {
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
        if dist.critical > 0 {
            if en {
                "Technically stable, but legally risky".to_string()
            } else {
                "Technisch stabil, aber rechtlich riskant".to_string()
            }
        } else if dist.high > 0 {
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
    } else if en {
        "Largely accessible — polish".to_string()
    } else {
        "Weitgehend barrierefrei — Feinschliff".to_string()
    }
}

pub(super) fn render_batch_management_risks(
    builder: renderreport::engine::ReportBuilder,
    pres: &BatchPresentation,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = i18n.locale() == "en";
    let dist = &pres.portfolio_summary.severity_distribution;
    let avg = pres.portfolio_summary.average_score.round() as u32;
    let seo = pres
        .portfolio_summary
        .module_averages
        .iter()
        .find(|(name, _)| name == "SEO")
        .map(|(_, score)| *score);
    let component_count = pres
        .top_issues
        .iter()
        .filter(|issue| issue.is_component_issue || issue.affected_urls.len() > 1)
        .count();
    let legal_level = if dist.critical > 0 {
        if en {
            "High"
        } else {
            "Hoch"
        }
    } else if dist.high > 0 {
        if en {
            "Medium"
        } else {
            "Mittel"
        }
    } else if en {
        "Low"
    } else {
        "Niedrig"
    };
    let visibility_level = match seo {
        Some(score) if score < 60 => {
            if en {
                "High"
            } else {
                "Hoch"
            }
        }
        Some(score) if score < 80 => {
            if en {
                "Medium"
            } else {
                "Mittel"
            }
        }
        Some(_) => {
            if en {
                "Low"
            } else {
                "Niedrig"
            }
        }
        None => {
            if en {
                "Unknown"
            } else {
                "Unbekannt"
            }
        }
    };
    let project_level = if component_count >= 3 {
        if en {
            "High"
        } else {
            "Hoch"
        }
    } else if component_count > 0 {
        if en {
            "Medium"
        } else {
            "Mittel"
        }
    } else if en {
        "Low"
    } else {
        "Niedrig"
    };

    let mut kv = KeyValueList::new().with_title(if en {
        "Management risk view"
    } else {
        "Management-Risikoansicht"
    });
    kv = kv
        .add(
            if en {
                "Legal / BFSG-EAA"
            } else {
                "Recht / BFSG-EAA"
            },
            format!(
                "{} — {} critical/high findings across {} URLs",
                legal_level,
                dist.critical + dist.high,
                pres.portfolio_summary.total_urls
            ),
        )
        .add(
            if en {
                "Conversion / usability"
            } else {
                "Conversion / Nutzbarkeit"
            },
            format!("{} / 100 average accessibility score", avg),
        )
        .add(
            if en {
                "SEO / visibility"
            } else {
                "SEO / Sichtbarkeit"
            },
            seo.map(|score| format!("{} — SEO {} / 100", visibility_level, score))
                .unwrap_or_else(|| visibility_level.to_string()),
        )
        .add(
            if en { "Project risk" } else { "Projektrisiko" },
            format!(
                "{} — {} recurring component/template {}",
                project_level,
                component_count,
                if component_count == 1 {
                    "pattern"
                } else {
                    "patterns"
                }
            ),
        );
    builder.add_component(kv)
}

pub(super) fn render_batch_internal_comparison(
    mut builder: renderreport::engine::ReportBuilder,
    pres: &BatchPresentation,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = i18n.locale() == "en";

    // The internal comparison and outlier detection only carry meaning across a
    // real sample. With one or two URLs "best vs weakest" is identical or
    // trivial, so we show a caveat instead of a misleading table (#450).
    if pres.url_details.len() < 3 {
        let note = if en {
            format!(
                "Cross-page comparison needs at least 3 URLs for meaningful domain-wide averages; this report covers {}. See the per-URL detail instead.",
                pres.url_details.len()
            )
        } else {
            format!(
                "Der seitenübergreifende Vergleich benötigt mindestens 3 URLs für aussagekräftige domainweite Durchschnitte; dieser Report umfasst {}. Maßgeblich ist hier die Einzel-URL-Auswertung.",
                pres.url_details.len()
            )
        };
        return builder.add_component(Callout::info(note));
    }

    let mut rows = Vec::new();
    let mut module_names = std::collections::BTreeSet::new();
    for detail in &pres.url_details {
        for (module, _) in &detail.module_scores {
            module_names.insert(module.clone());
        }
    }
    for module in module_names {
        let mut scored: Vec<_> = pres
            .url_details
            .iter()
            .filter_map(|detail| {
                detail
                    .module_scores
                    .iter()
                    .find(|(name, _)| name == &module)
                    .map(|(_, score)| (detail.url.as_str(), *score))
            })
            .collect();
        if scored.is_empty() {
            continue;
        }
        scored.sort_by_key(|(_, score)| *score);
        let (worst_url, worst_score) = scored[0];
        let (best_url, best_score) = scored[scored.len() - 1];
        rows.push(vec![
            module,
            format!("{} ({}/100)", truncate_url(best_url, 34), best_score),
            format!("{} ({}/100)", truncate_url(worst_url, 34), worst_score),
        ]);
    }
    if !rows.is_empty() {
        let mut table = AuditTable::new(vec![
            TableColumn::new(if en { "Module" } else { "Modul" }).with_width("20%"),
            TableColumn::new(if en { "Best URL" } else { "Beste URL" }).with_width("40%"),
            TableColumn::new(if en { "Weakest URL" } else { "Schwächste URL" }).with_width("40%"),
        ])
        .with_title(if en {
            "Internal comparison by module"
        } else {
            "Interner Vergleich nach Modul"
        });
        for row in rows {
            table = table.add_row(row);
        }
        builder = builder.add_component(table);
    }

    let avg = pres.portfolio_summary.average_score.round() as i32;
    let outliers: Vec<_> = pres
        .url_ranking
        .iter()
        .filter_map(|url| {
            let delta = url.score.round() as i32 - avg;
            (delta <= -15).then(|| {
                if en {
                    format!(
                        "{}: {} / 100 ({} points below average)",
                        truncate_url(&url.url, 70),
                        url.score.round() as u32,
                        delta.abs()
                    )
                } else {
                    format!(
                        "{}: {} / 100 ({} Punkte unter Durchschnitt)",
                        truncate_url(&url.url, 70),
                        url.score.round() as u32,
                        delta.abs()
                    )
                }
            })
        })
        .take(6)
        .collect();
    if !outliers.is_empty() {
        let mut list = List::new().with_title(if en {
            "Outlier URLs"
        } else {
            "Ausreißer-URLs"
        });
        for item in outliers {
            list = list.add_item(item);
        }
        builder = builder.add_component(list);
    }

    builder
}

pub(super) fn render_batch_decision_actions(
    builder: renderreport::engine::ReportBuilder,
    pres: &BatchPresentation,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = i18n.locale() == "en";
    let mut table = AuditTable::new(vec![
        TableColumn::new(if en {
            "Action / root cause"
        } else {
            "Maßnahme / Root Cause"
        })
        .with_width("34%"),
        TableColumn::new(if en { "Risk" } else { "Risiko" }).with_width("12%"),
        TableColumn::new(if en { "Impact" } else { "Wirkung" }).with_width("24%"),
        TableColumn::new(if en { "Complexity" } else { "Komplexität" }).with_width("14%"),
        TableColumn::new(if en { "Reach" } else { "Reichweite" }).with_width("16%"),
    ])
    .with_title(if en {
        "Decision-oriented top actions"
    } else {
        "Entscheidungsorientierte Top-Maßnahmen"
    });

    let mut count = 0;
    for group in pres.top_issues.iter().take(8) {
        // A verified template cluster for this rule overrides the loose
        // occurrence-count heuristic with the evidence-backed claim (and its
        // confirmed/likely distinction) instead of the vague "likely shared
        // component/template" label.
        let cluster = pres
            .template_clusters
            .iter()
            .find(|c| c.rule_id == group.rule_id);
        let root: String = if let Some(cluster) = cluster {
            cluster.decision_label.clone()
        } else if group.is_component_issue || group.affected_urls.len() > 1 {
            if en {
                "likely shared component/template"
            } else {
                "vermutlich Komponente/Template"
            }
            .to_string()
        } else if en {
            "page-specific".to_string()
        } else {
            "seitenspezifisch".to_string()
        };
        table = table.add_row(vec![
            format!("{} — {}", group.title, root),
            severity_label_i18n(group.severity, i18n),
            crate::audit::normalized::expected_impact_text(&group.expected_impact_kind, en),
            effort_label_i18n(group.effort, i18n),
            if en {
                format!(
                    "{} occurrences / {} URLs",
                    group.occurrence_count,
                    group.affected_urls.len()
                )
            } else {
                format!(
                    "{} Vorkommen / {} URLs",
                    group.occurrence_count,
                    group.affected_urls.len()
                )
            },
        ]);
        count += 1;
    }
    if count == 0 {
        return builder;
    }
    builder.add_component(table)
}

/// 3 key takeaways for batch report
pub(super) fn build_batch_key_points(
    pres: &BatchPresentation,
    dist: &SeverityDistribution,
    i18n: &I18n,
) -> Vec<String> {
    let en = i18n.locale() == "en";
    let mut points = Vec::with_capacity(3);

    // Point 1: Critical/high count across all URLs
    let ch = dist.critical + dist.high;
    if ch > 0 {
        if en {
            points.push(format!(
                "{} critical/high violations across {} URLs",
                ch, pres.portfolio_summary.total_urls
            ));
        } else {
            points.push(format!(
                "{} kritische/hohe Verstöße über {} URLs hinweg",
                ch, pres.portfolio_summary.total_urls
            ));
        }
    }

    // Point 2: Dominant/recurring issue
    if let Some(top) = pres.top_issues.first() {
        if en {
            points.push(format!(
                "Main issue: {} ({} occurrences on {} URLs)",
                top.title,
                top.occurrence_count,
                top.affected_urls.len()
            ));
        } else {
            points.push(format!(
                "Hauptproblem: {} ({} Vorkommen auf {} URLs)",
                top.title,
                top.occurrence_count,
                top.affected_urls.len()
            ));
        }
    }

    // Point 3: Legal status
    if dist.critical > 0 {
        if en {
            points.push(
                "WCAG Level A violations detected automatically — manual review needed for a defensible BFSG classification".to_string(),
            );
        } else {
            points.push(
                "WCAG-Level-A-Verstöße automatisiert erkannt — manuelle Prüfung für belastbare BFSG-Einordnung nötig".to_string(),
            );
        }
    } else if dist.high > 0 {
        if en {
            points.push(
                "No Level A violations, but structural weaknesses on multiple pages".to_string(),
            );
        } else {
            points.push(
                "Keine Level-A-Verstöße, aber strukturelle Optimierungspotenziale auf mehreren Seiten"
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
