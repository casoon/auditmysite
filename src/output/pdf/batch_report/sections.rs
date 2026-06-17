use renderreport::components::advanced::{
    ChecklistPanel, ChecklistRow, Grid, KeyValueList, List, PageBreak, SectionHeaderSplit,
    TableOfContents,
};
use renderreport::components::charts::{Gauge, GaugeThreshold};
use renderreport::components::text::{Label, TextBlock};
use renderreport::components::{AuditTable, BenchmarkRow, BenchmarkTable, TableColumn};
use renderreport::prelude::Image;
use renderreport::prelude::*;

use crate::audit::BatchReport;
use crate::cli::ReportLevel;
use crate::i18n::I18n;
use crate::output::report_model::*;
use crate::util::truncate_url;

use super::super::batch::build_batch_overview_grid;
use super::super::cover::{
    batch_certificate_label, build_batch_cover_score_row, certificate_badge_path,
};
use super::super::design::tokens;
use super::super::findings::first_sentence;
use super::super::helpers::{
    effort_label_i18n, priority_label_i18n, role_label_i18n, severity_label_i18n,
};

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
                "{} — {} recurring component/template pattern(s)",
                project_level, component_count
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
        let root = if group.is_component_issue || group.affected_urls.len() > 1 {
            if en {
                "likely shared component/template"
            } else {
                "vermutlich Komponente/Template"
            }
        } else if en {
            "page-specific"
        } else {
            "seitenspezifisch"
        };
        table = table.add_row(vec![
            format!("{} — {}", group.title, root),
            severity_label_i18n(group.severity, i18n),
            group.expected_impact.clone(),
            effort_label_i18n(group.effort, i18n),
            format!(
                "{} occurrences / {} URLs",
                group.occurrence_count,
                group.affected_urls.len()
            ),
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

/// Concrete quick actions for batch report
pub(super) fn build_batch_quick_actions(pres: &BatchPresentation, _i18n: &I18n) -> Vec<String> {
    let mut actions: Vec<String> = Vec::new();

    for item in &pres.action_plan.quick_wins {
        if actions.len() >= 3 {
            break;
        }
        let action_lower = item.action.to_lowercase();
        let scope = if action_lower.contains("alle") || action_lower.contains("global") {
            " (global)"
        } else {
            ""
        };
        actions.push(format!("{}{}", item.action, scope));
    }

    // Fallback from top issues
    if actions.is_empty() {
        for group in pres.top_issues.iter().take(3) {
            let rec = first_sentence(&group.recommendation);
            if !rec.is_empty() {
                actions.push(rec.to_string());
            }
        }
    }

    actions
}

/// Enhanced action plan with effort + scope columns
pub(super) fn render_batch_action_plan_enhanced(
    mut builder: renderreport::engine::ReportBuilder,
    plan: &ActionPlan,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = i18n.locale() == "en";
    let effort_col = if en { "Effort" } else { "Aufwand" };
    let role_col = if en { "Role" } else { "Rolle" };
    let scope_global = "global";
    let scope_content = "Content";
    let scope_component = if en { "Component" } else { "Komponente" };

    let render_section = |mut b: renderreport::engine::ReportBuilder,
                          title: String,
                          items: &[crate::output::report_model::ActionItem],
                          i18n: &I18n|
     -> renderreport::engine::ReportBuilder {
        if items.is_empty() {
            return b;
        }
        b = b.add_component(Section::new(title).with_level(2));
        let mut table = AuditTable::new(vec![
            TableColumn::new(i18n.t("column-action")),
            TableColumn::new(effort_col),
            TableColumn::new(if en { "Scope" } else { "Reichweite" }),
            TableColumn::new(role_col),
            TableColumn::new(i18n.t("column-priority")),
        ]);
        for item in items {
            let action_lower = item.action.to_lowercase();
            let scope = if action_lower.contains("alle")
                || action_lower.contains("global")
                || action_lower.contains("designsystem")
                || action_lower.contains("design system")
                || action_lower.contains("seitenübergreifend")
                || action_lower.contains("site-wide")
            {
                scope_global
            } else if action_lower.contains("content")
                || action_lower.contains("text")
                || action_lower.contains("bild")
                || action_lower.contains("image")
            {
                scope_content
            } else {
                scope_component
            };
            table = table.add_row(vec![
                item.action.clone(),
                effort_label_i18n(item.effort, i18n),
                scope.to_string(),
                role_label_i18n(item.role, i18n),
                priority_label_i18n(item.priority, i18n),
            ]);
        }
        b.add_component(table)
    };

    builder = render_section(
        builder,
        i18n.t("section-quick-wins"),
        &plan.quick_wins,
        i18n,
    );
    builder = render_section(
        builder,
        i18n.t("section-medium-actions"),
        &plan.medium_term,
        i18n,
    );
    builder = render_section(
        builder,
        i18n.t("section-structural-actions"),
        &plan.structural,
        i18n,
    );
    builder
}

/// Render cross-page consistency analysis (issues #44/#45/#46).
/// Shows navigation, heading, and canonical consistency stats with any
/// findings as warning rows.
pub(super) fn render_batch_consistency(
    mut builder: renderreport::engine::ReportBuilder,
    consistency: &crate::audit::batch_consistency::BatchConsistencyAnalysis,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = i18n.locale() == "en";
    let title = if en {
        "Cross-page consistency"
    } else {
        "Seitenübergreifende Konsistenz"
    };
    let intro = if en {
        "Checks that shared structural elements (navigation, headings, canonical URLs) are consistent across all audited pages. WCAG 3.2.3 / 3.2.4."
    } else {
        "Prüft, ob geteilte strukturelle Elemente (Navigation, Überschriften, Canonical-URLs) auf allen geprüften Seiten konsistent sind. WCAG 3.2.3 / 3.2.4."
    };
    builder = builder.add_component(SectionHeaderSplit::new(title, intro).with_level(2));

    let nav = &consistency.navigation;
    let nav_title = if en {
        format!(
            "Navigation ({}/{} with main nav, {}/{} with skip link)",
            nav.pages_with_main_nav, nav.total_pages, nav.pages_with_skip_link, nav.total_pages
        )
    } else {
        format!(
            "Navigation ({}/{} mit Hauptnav, {}/{} mit Skip-Link)",
            nav.pages_with_main_nav, nav.total_pages, nav.pages_with_skip_link, nav.total_pages
        )
    };
    let nav_rows: Vec<ChecklistRow> = if nav.findings.is_empty() {
        vec![ChecklistRow::new(
            if en { "Consistent" } else { "Konsistent" },
            if en {
                "No navigation inconsistencies detected."
            } else {
                "Keine Navigation-Inkonsistenzen erkannt."
            },
        )
        .with_status("good")]
    } else {
        nav.findings
            .iter()
            .map(|f| {
                ChecklistRow::new(if en { "Issue" } else { "Befund" }, f.as_str())
                    .with_status("warn")
            })
            .collect()
    };
    builder = builder.add_component(ChecklistPanel::new(nav_rows).with_title(&nav_title));

    let h = &consistency.headings;
    let head_title = if en {
        format!(
            "Headings ({}/{} with single H1, {} missing, {} multiple)",
            h.pages_with_single_h1, h.total_pages, h.pages_with_no_h1, h.pages_with_multiple_h1
        )
    } else {
        format!(
            "Überschriften ({}/{} mit einem H1, {} ohne H1, {} mit mehreren)",
            h.pages_with_single_h1, h.total_pages, h.pages_with_no_h1, h.pages_with_multiple_h1
        )
    };
    let head_rows: Vec<ChecklistRow> = if h.findings.is_empty() {
        vec![ChecklistRow::new(
            if en { "Consistent" } else { "Konsistent" },
            if en {
                "Every page starts with exactly one H1."
            } else {
                "Jede Seite beginnt mit genau einem H1."
            },
        )
        .with_status("good")]
    } else {
        h.findings
            .iter()
            .map(|f| {
                ChecklistRow::new(if en { "Issue" } else { "Befund" }, f.as_str())
                    .with_status("warn")
            })
            .collect()
    };
    builder = builder.add_component(ChecklistPanel::new(head_rows).with_title(&head_title));

    let c = &consistency.canonical;
    let canon_title = if en {
        format!(
            "Canonical URLs (www: {}, non-www: {}, missing: {} of {})",
            c.www_count, c.non_www_count, c.missing_count, c.total_pages
        )
    } else {
        format!(
            "Canonical-URLs (www: {}, ohne www: {}, fehlend: {} von {})",
            c.www_count, c.non_www_count, c.missing_count, c.total_pages
        )
    };
    let canon_rows: Vec<ChecklistRow> = if c.findings.is_empty() {
        vec![ChecklistRow::new(
            if en { "Consistent" } else { "Konsistent" },
            if en {
                "All pages canonicalize to the same domain variant."
            } else {
                "Alle Seiten kanonisieren auf dieselbe Domain-Variante."
            },
        )
        .with_status("good")]
    } else {
        c.findings
            .iter()
            .map(|f| {
                ChecklistRow::new(if en { "Issue" } else { "Befund" }, f.as_str())
                    .with_status("warn")
            })
            .collect()
    };
    builder.add_component(ChecklistPanel::new(canon_rows).with_title(&canon_title))
}

/// Closing section for batch report
pub(super) fn render_next_steps_batch(
    mut builder: renderreport::engine::ReportBuilder,
    pres: &BatchPresentation,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = i18n.locale() == "en";
    let intro = if en {
        "Concrete recommendation for implementation."
    } else {
        "Konkrete Handlungsempfehlung für die Umsetzung."
    };
    builder = builder.add_component(
        SectionHeaderSplit::new(i18n.t("section-next-steps-recommended"), intro).with_level(1),
    );

    let mut steps: Vec<(String, &str)> = Vec::new();

    let scope_global = "global";
    let scope_component = if en {
        "component-based"
    } else {
        "komponentenbasiert"
    };

    // From quick wins
    for item in &pres.action_plan.quick_wins {
        if steps.len() >= 3 {
            break;
        }
        let action_lower = item.action.to_lowercase();
        let scope = if action_lower.contains("alle")
            || action_lower.contains("designsystem")
            || action_lower.contains("design system")
            || action_lower.contains("global")
        {
            scope_global
        } else {
            scope_component
        };
        steps.push((item.action.clone(), scope));
    }

    // Fallback from medium_term
    if steps.len() < 3 {
        for item in &pres.action_plan.medium_term {
            if steps.len() >= 3 {
                break;
            }
            steps.push((item.action.clone(), scope_component));
        }
    }

    if !steps.is_empty() {
        let mut table = AuditTable::new(vec![
            TableColumn::new(i18n.t("column-priority")),
            TableColumn::new(i18n.t("column-action")),
            TableColumn::new(if en { "Scope" } else { "Reichweite" }),
        ]);
        for (i, (action, scope)) in steps.iter().enumerate() {
            table = table.add_row(vec![
                format!("{}", i + 1),
                action.clone(),
                scope.to_string(),
            ]);
        }
        builder = builder.add_component(table);
    }

    let callout_body = if en {
        "For a complete WCAG conformance check we additionally recommend a manual audit with assistive technologies (screen reader, keyboard navigation). This automated audit covers about 30–40% of WCAG criteria."
    } else {
        "Für eine vollständige WCAG-Konformitätsprüfung empfehlen wir ergänzend einen manuellen Audit mit assistiven Technologien (Screenreader, Tastaturnavigation). Dieser automatisierte Audit deckt ca. 30–40% der WCAG-Kriterien ab."
    };
    builder = builder
        .add_component(Callout::info(callout_body).with_title(i18n.t("section-next-steps-block")));

    builder
}

// ─── Interactive Journey Summary ────────────────────────────────────────────

pub(super) fn translate_interactive_category(category: &str, en: bool) -> String {
    match (category, en) {
        ("TabOrder", true) => "Tab Order",
        ("TabOrder", false) => "Tab-Reihenfolge",
        ("FocusTrap", true) => "Focus Trap (Modal)",
        ("FocusTrap", false) => "Fokus-Falle (Modal)",
        ("StateTransition", true) => "State Transitions",
        ("StateTransition", false) => "Zustandswechsel",
        ("FocusRestoration", true) => "Focus Restoration",
        ("FocusRestoration", false) => "Fokus-Rückführung",
        ("FormError", true) => "Form Error Announcement",
        ("FormError", false) => "Formularfehler-Ankündigung",
        ("SpaNavigation", true) => "SPA Navigation",
        ("SpaNavigation", false) => "SPA-Navigation",
        ("HiddenFocusable", true) => "Hidden Focusable",
        ("HiddenFocusable", false) => "Verstecktes fokussierbares Element",
        ("SkipLink", true) => "Skip Link",
        ("SkipLink", false) => "Skip-Link",
        ("FocusIndicator", true) => "Focus Indicator",
        ("FocusIndicator", false) => "Fokus-Indikator",
        ("MenuJourney", true) => "Menu / Navigation",
        ("MenuJourney", false) => "Menü / Navigation",
        ("TabsJourney", true) => "Tab Widget",
        ("TabsJourney", false) => "Tab-Widget",
        _ => category,
    }
    .to_string()
}

pub(super) fn render_batch_interactive_summary(
    mut builder: renderreport::engine::ReportBuilder,
    summary: &crate::output::report_model::InteractiveJourneySummary,
    total_urls: usize,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = i18n.locale() == "en";

    let title = if en {
        "Keyboard Accessibility Journey"
    } else {
        "Tastatur-Accessibility-Journey"
    };
    let intro = if en {
        "Interactive tests verify whether focus management, state transitions, and keyboard navigation behave correctly — not just whether the initial page tree is well-structured."
    } else {
        "Interaktive Tests prüfen, ob Fokusführung, Zustandswechsel und Tastaturnavigation korrekt funktionieren — nicht nur ob der initiale Seitenbaum korrekt ausgezeichnet ist."
    };
    builder = builder.add_component(SectionHeaderSplit::new(title, intro).with_level(1));

    let callout_text = if summary.pages_with_issues == 0 {
        if en {
            format!(
                "No interactive issues detected on any of the {} tested pages.",
                summary.total_pages_tested
            )
        } else {
            format!(
                "Keine interaktiven Befunde auf den {} geprüften Seiten.",
                summary.total_pages_tested
            )
        }
    } else if en {
        format!(
            "{} of {} tested pages have interactive issues — {} pages without issues.",
            summary.pages_with_issues,
            summary.total_pages_tested,
            summary.total_pages_tested - summary.pages_with_issues
        )
    } else {
        format!(
            "{} von {} geprüften Seiten haben interaktive Befunde — {} Seiten ohne Befunde.",
            summary.pages_with_issues,
            summary.total_pages_tested,
            summary.total_pages_tested - summary.pages_with_issues
        )
    };
    builder = builder.add_component(if summary.has_critical {
        Callout::warning(&callout_text)
    } else if summary.pages_with_issues > 0 {
        Callout::info(&callout_text)
    } else {
        Callout::success(&callout_text)
    });

    if !summary.categories.is_empty() {
        let category_col = if en { "Category" } else { "Kategorie" };
        let affected_col = if en {
            "Affected URLs"
        } else {
            "Betroffene URLs"
        };
        let share_col = if en { "Share" } else { "Anteil" };
        let sev_col = if en { "Max. Severity" } else { "Max. Schwere" };
        let mut table = AuditTable::new(vec![
            TableColumn::new(category_col),
            TableColumn::new(affected_col),
            TableColumn::new(share_col),
            TableColumn::new(sev_col),
        ])
        .with_title(if en {
            "Issues by Category"
        } else {
            "Befunde nach Kategorie"
        });
        for row in &summary.categories {
            let pct = (row.affected_urls * 100)
                .checked_div(total_urls)
                .unwrap_or(0);
            let sev_label = match row.max_severity {
                crate::wcag::Severity::Critical => {
                    if en {
                        "Critical"
                    } else {
                        "Kritisch"
                    }
                }
                crate::wcag::Severity::High => {
                    if en {
                        "High"
                    } else {
                        "Hoch"
                    }
                }
                crate::wcag::Severity::Medium => {
                    if en {
                        "Medium"
                    } else {
                        "Mittel"
                    }
                }
                crate::wcag::Severity::Low => {
                    if en {
                        "Low"
                    } else {
                        "Niedrig"
                    }
                }
            };
            let category_label = translate_interactive_category(&row.category, en);
            table = table.add_row(vec![
                category_label,
                row.affected_urls.to_string(),
                format!("{pct}%"),
                sev_label.to_string(),
            ]);
        }
        builder = builder.add_component(table);
    }

    builder
}

// ─── Batch Report Sections ──────────────────────────────────────────────────

pub(super) fn render_batch_cover(
    mut builder: renderreport::engine::ReportBuilder,
    batch: &BatchReport,
    pres: &BatchPresentation,
    config: &ReportConfig,
    score: u32,
    i18n: &I18n,
) -> anyhow::Result<renderreport::engine::ReportBuilder> {
    let domain = &pres.portfolio_summary.domain;

    let cover_logo_asset = super::super::cover_logo_asset(config);
    builder = super::super::register_cover_logo_asset(builder, config, cover_logo_asset);

    builder = builder
        .add_component(Image::new(cover_logo_asset).with_width("120pt"))
        .add_component(
            Label::new(i18n.t("batch-cover-eyebrow"))
                .with_size("11pt")
                .bold()
                .with_color("#0f766e"),
        )
        .add_component(
            Label::new(i18n.t("batch-cover-title"))
                .with_size("28pt")
                .bold(),
        )
        .add_component(
            Label::new(i18n.t("batch-cover-kicker"))
                .with_size("12pt")
                .with_color("#475569"),
        );

    // Audit-Rahmen box
    {
        let modules_str = pres.portfolio_summary.active_modules.join(", ");
        let mut cover_meta = KeyValueList::new().with_title(i18n.t("batch-cover-frame-title"));
        cover_meta = cover_meta
            .add(i18n.t("batch-cover-frame-domain"), domain)
            .add(i18n.t("batch-cover-frame-date"), &pres.cover.date)
            .add(
                i18n.t("batch-cover-frame-urls"),
                format!("{}", pres.portfolio_summary.total_urls),
            )
            .add(
                i18n.t("batch-cover-frame-certificate"),
                &pres.portfolio_summary.certificate,
            )
            .add(i18n.t("batch-cover-frame-modules"), &modules_str)
            .add(
                i18n.t("batch-cover-frame-version"),
                format!("auditmysite v{}", pres.cover.version),
            );
        if let Some(sample) = &batch.sample {
            let source = i18n.t(&format!("batch-source-{}", sample.source));
            let scope = if sample.is_sample {
                i18n.t_args(
                    "batch-scope-sample",
                    &[
                        ("audited", sample.audited.to_string()),
                        ("total", sample.total_discovered.to_string()),
                        ("source", source),
                    ],
                )
            } else {
                i18n.t_args(
                    "batch-scope-full",
                    &[
                        ("total", sample.total_discovered.to_string()),
                        ("source", source),
                    ],
                )
            };
            cover_meta = cover_meta.add(i18n.t("batch-cover-frame-scope"), scope);
        }
        builder = builder.add_component(cover_meta);
    }

    let batch_badge_asset = "/certificate-badge-batch.svg";
    let batch_badge_enabled =
        if let Ok(path) = certificate_badge_path(batch_certificate_label(score)) {
            builder = builder.asset(batch_badge_asset, path);
            true
        } else {
            false
        };

    builder = builder
        .add_component(build_batch_cover_score_row(
            score,
            pres.portfolio_summary.total_urls as u32,
            pres.portfolio_summary.total_violations as u32,
            batch_badge_enabled.then_some(batch_badge_asset),
            i18n,
        )?)
        .add_component(
            TextBlock::new(&pres.portfolio_summary.verdict_text)
                .with_size("11pt")
                .with_line_height("1.4em")
                .with_max_width("100%"),
        )
        .add_component(super::super::output_scope_callout(i18n))
        .add_component(PageBreak::new());

    if config.level != ReportLevel::Executive {
        builder = builder.add_component(TableOfContents::new().with_depth(1));
    }

    Ok(builder)
}

pub(super) fn render_batch_status_section(
    mut builder: renderreport::engine::ReportBuilder,
    pres: &BatchPresentation,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let dist = &pres.portfolio_summary.severity_distribution;

    // Assessment callout
    let assessment = build_batch_assessment(&pres.portfolio_summary, dist, i18n);
    let callout = match pres.portfolio_summary.risk_level.as_str() {
        "Kritisch" | "Hoch" | "Critical" | "High" => {
            Callout::warning(&pres.portfolio_summary.risk_summary).with_title(&assessment)
        }
        "Mittel" | "Medium" => {
            Callout::info(&pres.portfolio_summary.risk_summary).with_title(&assessment)
        }
        _ => Callout::success(&pres.portfolio_summary.risk_summary).with_title(&assessment),
    };
    builder = builder
        .add_component(Section::new(i18n.t("batch-section-status")).with_level(1))
        .add_component(callout);

    // Score overview cards
    let score = pres.portfolio_summary.average_score.round() as u32;
    builder = builder.add_component(build_batch_overview_grid(
        pres.portfolio_summary.total_urls as u32,
        score,
        pres.portfolio_summary.total_violations as u32,
        (dist.critical + dist.high) as u32,
        pres.portfolio_summary.crawl_links.as_ref().map(|links| {
            (links.broken_internal_links.len() + links.broken_external_links.len()) as u32
        }),
        i18n.locale() == "en",
    ));

    // Key points
    let key_points = build_batch_key_points(pres, dist, i18n);
    let mut kp_list = List::new().with_title(i18n.t("narrative-key-points-title"));
    for point in &key_points {
        kp_list = kp_list.add_item(point);
    }
    builder = builder.add_component(kp_list);

    // Impact summary
    builder = render_batch_impact_summary(builder, pres, i18n);

    // Quick actions
    let actions = build_batch_quick_actions(pres, i18n);
    if !actions.is_empty() {
        let rows: Vec<ChecklistRow> = actions
            .iter()
            .map(|action| ChecklistRow::new(action, "").with_status("warn"))
            .collect();
        builder = builder.add_component(
            ChecklistPanel::new(rows).with_title(i18n.t("narrative-quick-actions-title")),
        );
    }

    builder = render_batch_management_risks(builder, pres, i18n);
    builder = render_batch_internal_comparison(builder, pres, i18n);

    builder
}

pub(super) fn render_batch_module_portfolio(
    mut builder: renderreport::engine::ReportBuilder,
    pres: &BatchPresentation,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    if pres.portfolio_summary.module_averages.is_empty() {
        return builder;
    }

    let en = i18n.locale() == "en";
    builder = builder.add_component(
        SectionHeaderSplit::new(
            i18n.t("batch-module-portfolio-title"),
            i18n.t("batch-module-portfolio-intro"),
        )
        .with_level(1),
    );

    let mut grid = Grid::new(4).with_item_min_height("118pt");
    for (name, score) in &pres.portfolio_summary.module_averages {
        let mut gauge = Gauge::new(localized_module_name(name, en), *score as f64);
        gauge.thresholds = vec![
            GaugeThreshold {
                value: 0.0,
                color: tokens::DANGER.to_string(),
            },
            GaugeThreshold {
                value: 50.0,
                color: tokens::WARN_DEEP.to_string(),
            },
            GaugeThreshold {
                value: 90.0,
                color: tokens::SUCCESS.to_string(),
            },
        ];
        grid = grid.add_item(serde_json::json!({
            "type": "gauge",
            "data": gauge.to_data()
        }));
    }
    builder = builder.add_component(grid);

    let total = pres.portfolio_summary.total_urls;
    let mut table = AuditTable::new(vec![
        TableColumn::new(i18n.t("batch-col-module")).with_width("20%"),
        TableColumn::new(i18n.t("batch-col-score")).with_width("14%"),
        TableColumn::new(i18n.t("batch-col-clean-pages")).with_width("18%"),
        TableColumn::new(i18n.t("batch-col-effect")).with_width("48%"),
    ])
    .with_title(i18n.t("batch-module-portfolio-table-title"));

    for (name, score) in &pres.portfolio_summary.module_averages {
        let clean_pages = clean_pages_for_module(pres, name);
        table = table.add_row(vec![
            localized_module_name(name, en).to_string(),
            format!("{score}/100"),
            i18n.t_args(
                "batch-module-clean-count",
                &[
                    ("clean", clean_pages.to_string()),
                    ("total", total.to_string()),
                ],
            ),
            module_effect_sentence(name, *score, clean_pages, total, en),
        ]);
    }
    builder = builder.add_component(table);

    let all_clean: Vec<String> = pres
        .portfolio_summary
        .module_averages
        .iter()
        .filter(|(name, _)| clean_pages_for_module(pres, name) == total && total > 0)
        .map(|(name, _)| localized_module_name(name, en).to_string())
        .collect();
    if !all_clean.is_empty() {
        builder = builder.add_component(
            Callout::success(
                i18n.t_args(
                    "batch-module-clean-confirmation",
                    &[
                        ("modules", all_clean.join(", ")),
                        ("total", total.to_string()),
                    ],
                )
                .as_str(),
            )
            .with_title(i18n.t("batch-module-clean-title")),
        );
    }

    builder
}

pub(super) fn clean_pages_for_module(pres: &BatchPresentation, module_name: &str) -> usize {
    pres.url_details
        .iter()
        .filter(|detail| {
            detail
                .module_scores
                .iter()
                .find(|(name, _)| name == module_name)
                .is_some_and(|(_, score)| *score >= 90)
        })
        .count()
}

pub(super) fn localized_module_name(name: &str, en: bool) -> &str {
    match (name, en) {
        ("Accessibility", false) => "Barrierefreiheit",
        ("Best Practices", false) => "Best Practices",
        ("Dark Mode", false) => "Dark Mode",
        ("Mobile", false) => "Mobile",
        ("Performance", false) => "Performance",
        ("Security", false) => "Security",
        ("SEO", false) => "SEO",
        ("UX", false) => "UX",
        ("Journey", false) => "Journey",
        ("AI Visibility", false) => "KI-Sichtbarkeit",
        ("Content Visibility", false) => "Content Visibility",
        ("Source Quality", false) => "Source Quality",
        ("Tech Stack", false) => "Tech-Stack",
        _ => name,
    }
}

pub(super) fn module_effect_sentence(
    module_name: &str,
    score: u32,
    clean_pages: usize,
    total: usize,
    en: bool,
) -> String {
    if clean_pages == total && total > 0 {
        return if en {
            "Checked across the audited URL set with no relevant anomalies in the green band."
                .to_string()
        } else {
            "Im geprüften URL-Set ohne relevante Auffälligkeiten im grünen Bereich.".to_string()
        };
    }

    let band = match score {
        90..=100 => 0,
        75..=89 => 1,
        60..=74 => 2,
        _ => 3,
    };

    match (module_name, band, en) {
        ("Accessibility", 0, true) => {
            "Accessibility is broadly stable; remaining findings are concentrated on individual pages."
        }
        ("Accessibility", 0, false) => {
            "Barrierefreiheit ist breit stabil; verbleibende Befunde konzentrieren sich auf einzelne Seiten."
        }
        ("Accessibility", _, true) => {
            "Accessibility barriers affect repeated user paths and should be prioritized by reach."
        }
        ("Accessibility", _, false) => {
            "Barrieren betreffen wiederkehrende Nutzerpfade und sollten nach Reichweite priorisiert werden."
        }
        ("Performance", 0, true) => {
            "Loading behavior is reliable across most audited pages."
        }
        ("Performance", 0, false) => {
            "Das Ladeverhalten ist über die meisten geprüften Seiten zuverlässig."
        }
        ("Performance", _, true) => {
            "Performance variance can slow important page groups and weaken completion rates."
        }
        ("Performance", _, false) => {
            "Performance-Streuung kann wichtige Seitengruppen verlangsamen und Abschlüsse erschweren."
        }
        ("SEO", 0, true) => {
            "Search visibility basics are consistent across the audited URL set."
        }
        ("SEO", 0, false) => {
            "Grundlagen für Sichtbarkeit in Suchmaschinen sind im geprüften URL-Set konsistent."
        }
        ("SEO", _, true) => {
            "SEO inconsistencies can dilute discoverability across page templates."
        }
        ("SEO", _, false) => {
            "SEO-Inkonsistenzen können die Auffindbarkeit über Seitentemplates hinweg schwächen."
        }
        ("Security", 0, true) => {
            "Security signals are clean in the automated checks across the audited pages."
        }
        ("Security", 0, false) => {
            "Security-Signale sind in den automatisierten Checks über die geprüften Seiten sauber."
        }
        ("Security", _, true) => {
            "Security findings should be resolved centrally because they often affect all templates."
        }
        ("Security", _, false) => {
            "Security-Befunde sollten zentral gelöst werden, da sie häufig alle Templates betreffen."
        }
        ("Mobile", 0, true) => "Mobile usage is stable across the audited pages.",
        ("Mobile", 0, false) => {
            "Die Nutzung auf Mobilgeräten ist über die geprüften Seiten stabil."
        }
        ("Mobile", _, true) => {
            "Mobile issues can reduce usability on smaller screens across page groups."
        }
        ("Mobile", _, false) => {
            "Mobile Probleme können die Nutzung auf kleineren Displays über Seitengruppen erschweren."
        }
        (_, 0, true) => "The module is stable across most audited pages.",
        (_, 0, false) => "Das Modul ist über die meisten geprüften Seiten stabil.",
        (_, 1, true) => "The module has a usable foundation with targeted cleanup potential.",
        (_, 1, false) => "Das Modul hat eine tragfähige Basis mit gezieltem Bereinigungsbedarf.",
        (_, 2, true) => "The module shows recurring weaknesses across the audited URL set.",
        (_, 2, false) => "Das Modul zeigt wiederkehrende Schwächen im geprüften URL-Set.",
        (_, _, true) => "The module needs structural attention across multiple pages.",
        (_, _, false) => "Das Modul benötigt strukturelle Aufmerksamkeit über mehrere Seiten.",
    }
    .to_string()
}

pub(super) fn render_batch_impact_summary(
    mut builder: renderreport::engine::ReportBuilder,
    pres: &BatchPresentation,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = i18n.locale() == "en";
    let dist = &pres.portfolio_summary.severity_distribution;
    let a11y_avg = pres.portfolio_summary.average_score.round() as u32;

    let user_impact = match (a11y_avg, en) {
        (s, true) if s < 50 => {
            "Critical — core functions are unreachable for users with disabilities"
        }
        (s, false) if s < 50 => {
            "Kritisch — zentrale Funktionen sind für Nutzer mit Einschränkungen nicht erreichbar"
        }
        (s, true) if s < 70 => {
            "Limited — structural issues impede users with assistive technologies"
        }
        (s, false) if s < 70 => {
            "Eingeschränkt — strukturelle Probleme behindern Nutzer mit Hilfstechnologien"
        }
        (s, true) if s < 85 => {
            "Good — individual barriers for assistive technologies on several pages"
        }
        (s, false) if s < 85 => {
            "Gut — einzelne Barrieren für Hilfstechnologien auf mehreren Seiten"
        }
        (_, true) => "Very good — assistive technologies are largely supported",
        (_, false) => "Sehr gut — Hilfstechnologien werden weitgehend unterstützt",
    };
    let business_impact = if dist.critical > 0 {
        if en {
            "Large parts of the website are unusable or barely usable for certain user groups."
        } else {
            "Weite Teile der Website sind für bestimmte Nutzergruppen nicht oder kaum nutzbar."
        }
    } else if dist.high > 0 {
        if en {
            "Individual functional areas are problematic for users with disabilities."
        } else {
            "Einzelne Funktionsbereiche sind für Nutzer mit Einschränkungen problematisch."
        }
    } else if en {
        "Low impact — users can fundamentally use the website."
    } else {
        "Geringe Auswirkung — Nutzer können die Website grundsätzlich verwenden."
    };
    let legal_impact = if dist.critical > 0 {
        if en {
            "WCAG Level A violations detected automatically — manual review required for a defensible BFSG classification."
        } else {
            "WCAG-Level-A-Verstöße automatisiert erkannt — für belastbare BFSG-Einordnung ist manuelle Prüfung nötig."
        }
    } else if en {
        "No critical violations detected automatically — manual review recommended for full classification."
    } else {
        "Automatisiert keine kritischen Verstöße erkannt — manuelle Prüfung für vollständige Einordnung empfohlen."
    };

    let (user_label, business_label, risk_label) = if en {
        ("User", "Business", "Risk")
    } else {
        ("Nutzer", "Business", "Risiko")
    };
    let mut impact_kv = KeyValueList::new().with_title(i18n.t("narrative-impact-title"));
    impact_kv = impact_kv
        .add(user_label, user_impact)
        .add(business_label, business_impact)
        .add(risk_label, legal_impact);
    builder = builder.add_component(impact_kv);

    builder
}

pub(super) fn render_batch_url_ranking(
    mut builder: renderreport::engine::ReportBuilder,
    pres: &BatchPresentation,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let rows: Vec<BenchmarkRow> = pres
        .url_ranking
        .iter()
        .enumerate()
        .map(|(i, u)| {
            let mut row = BenchmarkRow::new(
                (i + 1) as u32,
                &truncate_url(&u.url, 35),
                u.overall_score,
                u.score as u32,
                u.critical_violations as u32,
            );
            if let Some(detail) = pres.url_details.iter().find(|detail| detail.url == u.url) {
                if let Some((_, score)) = detail
                    .module_scores
                    .iter()
                    .find(|(module, _)| module == "SEO")
                {
                    row = row.with_seo(*score);
                }
                if let Some((_, score)) = detail
                    .module_scores
                    .iter()
                    .find(|(module, _)| module == "Performance")
                {
                    row = row.with_performance(*score);
                }
                if let Some((_, score)) = detail
                    .module_scores
                    .iter()
                    .find(|(module, _)| module == "Security")
                {
                    row = row.with_security(*score);
                }
            }
            row
        })
        .collect();

    builder = builder
        .add_component(
            SectionHeaderSplit::new(
                i18n.t("batch-url-ranking-title"),
                i18n.t("batch-url-ranking-intro"),
            )
            .with_level(1),
        )
        .add_component(BenchmarkTable::new(rows));

    builder
}

pub(super) fn render_batch_top_issues(
    mut builder: renderreport::engine::ReportBuilder,
    pres: &BatchPresentation,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let top_intro = i18n.t("batch-top-issues-intro");
    builder = builder.add_component(
        SectionHeaderSplit::new(i18n.t("batch-section-most-frequent"), top_intro).with_level(1),
    );

    // Frequency table
    if !pres.issue_frequency.is_empty() {
        let affected_col = i18n.t("batch-col-affected-urls");
        let mut freq_table = AuditTable::new(vec![
            TableColumn::new(i18n.t("batch-col-problem")),
            TableColumn::new("WCAG"),
            TableColumn::new(i18n.t("batch-col-occurrences")),
            TableColumn::new(affected_col),
            TableColumn::new(i18n.t("batch-col-priority")),
        ])
        .with_title(i18n.t("batch-section-most-frequent-violations"));

        for issue in &pres.issue_frequency {
            freq_table = freq_table.add_row(vec![
                issue.problem.clone(),
                issue.wcag.clone(),
                issue.occurrences.to_string(),
                issue.affected_urls.to_string(),
                super::super::helpers::priority_label_i18n(issue.priority, i18n),
            ]);
        }
        builder = builder.add_component(freq_table);
    }

    builder = render_batch_decision_actions(builder, pres, i18n);

    // Unified problem blocks
    let scope_global_word = i18n.t("batch-meta-global");
    let scope_individual = i18n.t("batch-meta-individual");
    let occurrences_word_top = i18n.t("batch-meta-occurrences");
    let affected_urls_word = i18n.t("batch-meta-affected-urls");
    let effort_word = i18n.t("batch-meta-effort");
    let scope_word = i18n.t("batch-meta-scope");
    let impact_user_label = i18n.t("batch-meta-impact-user");
    let impact_business_label = i18n.t("batch-meta-impact-business");
    let fix_label = i18n.t("batch-meta-fix");
    let meta_label = i18n.t("batch-meta-classification");

    for group in pres.top_issues.iter().take(5) {
        let scope = if group.affected_urls.len() >= pres.portfolio_summary.total_urls {
            &scope_global_word
        } else {
            &scope_individual
        };
        let effort_label = effort_label_i18n(group.effort, i18n);
        let meta_line = format!(
            "{} {} · {} {} · {}: {} · {}: {}",
            group.occurrence_count,
            occurrences_word_top,
            group.affected_urls.len(),
            affected_urls_word,
            effort_word,
            effort_label,
            scope_word,
            scope
        );

        let mut kv = KeyValueList::new()
            .with_title(&group.title)
            .add(
                i18n.t("findings-card-key-problem"),
                &group.customer_description,
            )
            .add(&impact_user_label, &group.user_impact)
            .add(&impact_business_label, &group.business_impact);
        if !group.typical_cause.is_empty() {
            kv = kv.add(i18n.t("findings-card-key-cause"), &group.typical_cause);
        }
        if !group.recommendation.is_empty() {
            kv = kv.add(&fix_label, &group.recommendation);
        }
        kv = kv.add(&meta_label, meta_line);
        builder = builder.add_component(kv);
    }

    builder
}

pub(super) fn render_batch_action_plan_section(
    mut builder: renderreport::engine::ReportBuilder,
    pres: &BatchPresentation,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let action_intro = i18n.t("batch-action-plan-intro");
    builder = builder.add_component(
        SectionHeaderSplit::new(i18n.t("batch-action-plan-title"), action_intro).with_level(1),
    );
    builder = render_batch_action_plan_enhanced(builder, &pres.action_plan, i18n);

    // Render blocking
    if !pres.portfolio_summary.render_blocking_summary.is_empty() {
        let mut kv = KeyValueList::new().with_title(i18n.t("batch-render-blocking-kv-title"));
        for (label, value) in &pres.portfolio_summary.render_blocking_summary {
            kv = kv.add(label, value);
        }
        builder = builder
            .add_component(Section::new(i18n.t("batch-render-blocking-section")).with_level(1))
            .add_component(TextBlock::new(i18n.t("batch-render-blocking-intro")))
            .add_component(kv);
    }

    // Performance budgets
    if !pres.portfolio_summary.budget_summary.is_empty() {
        let pages_col = i18n.t("batch-budget-pages-col");
        let budget_table_title = i18n.t("batch-budget-table-title");
        let mut table = AuditTable::new(vec![
            TableColumn::new(i18n.t("batch-col-metric")),
            TableColumn::new(i18n.t("batch-col-budget")),
            TableColumn::new(pages_col),
            TableColumn::new(i18n.t("batch-col-severity")),
        ])
        .with_title(budget_table_title);
        for (metric, budget, count, sev) in &pres.portfolio_summary.budget_summary {
            table = table.add_row(vec![
                metric.clone(),
                budget.clone(),
                count.to_string(),
                sev.clone(),
            ]);
        }
        let budgets_intro = i18n.t("batch-budget-intro");
        builder = builder
            .add_component(Section::new(i18n.t("batch-section-performance-budgets")).with_level(1))
            .add_component(TextBlock::new(budgets_intro))
            .add_component(table);
    }

    builder
}

pub(super) fn render_batch_tech_url_matrix(
    mut builder: renderreport::engine::ReportBuilder,
    pres: &BatchPresentation,
    config: &ReportConfig,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    builder = builder.add_component(
        SectionHeaderSplit::new(
            i18n.t("batch-section-tech-url-matrix"),
            i18n.t("batch-section-tech-url-matrix-intro"),
        )
        .with_level(1),
    );

    if let Some(ref crawl_links) = pres.portfolio_summary.crawl_links {
        builder = render_batch_crawl_links(builder, crawl_links, i18n);
    }

    // URL matrix table
    let page_col = i18n.t("batch-matrix-col-page");
    let title_col = i18n.t("batch-matrix-col-title");
    let mut matrix = AuditTable::new(vec![
        TableColumn::new("#").with_width("4%"),
        TableColumn::new(page_col).with_width("26%"),
        TableColumn::new(title_col).with_width("28%"),
        TableColumn::new(i18n.t("batch-col-links-to")).with_width("10%"),
        TableColumn::new(i18n.t("batch-col-links-from")).with_width("10%"),
        TableColumn::new(i18n.t("batch-col-words")).with_width("10%"),
        TableColumn::new("Score").with_width("12%"),
    ])
    .with_title(i18n.t("batch-table-pages-overview"));

    for row in &pres.url_matrix {
        let score_str = pres
            .url_details
            .iter()
            .find(|d| d.url == row.url)
            .map(|d| format!("{}/100", d.score.round() as u32))
            .unwrap_or_else(|| "—".to_string());
        matrix = matrix.add_row(vec![
            row.rank.to_string(),
            truncate_url(&row.url, 34),
            row.title
                .as_deref()
                .map(|t| truncate_url(t, 36))
                .unwrap_or_else(|| "—".to_string()),
            row.inbound_links.to_string(),
            row.outbound_links.to_string(),
            super::super::format_word_count(row.word_count),
            score_str,
        ]);
    }
    builder = builder.add_component(matrix);

    if config.level != ReportLevel::Executive {
        let mut focus_table = AuditTable::new(vec![
            TableColumn::new("URL"),
            TableColumn::new(i18n.t("batch-col-page-type")),
            TableColumn::new(i18n.t("batch-col-attributes")),
            TableColumn::new(i18n.t("batch-col-top-issues")),
        ])
        .with_title(i18n.t("batch-table-focus-pages"));

        for detail in pres.url_details.iter().take(10) {
            focus_table = focus_table.add_row(vec![
                truncate_url(&detail.url, 38),
                detail.page_type.clone().unwrap_or_else(|| "—".to_string()),
                if detail.page_attributes.is_empty() {
                    "—".to_string()
                } else {
                    truncate_url(&detail.page_attributes.join(", "), 40)
                },
                if detail.top_issues.is_empty() {
                    "—".to_string()
                } else {
                    truncate_url(&detail.top_issues.join(", "), 52)
                },
            ]);
        }
        builder = builder.add_component(focus_table);
    }

    builder
}

pub(super) fn render_batch_crawl_links(
    mut builder: renderreport::engine::ReportBuilder,
    crawl_links: &crate::output::report_model::CrawlLinkSummary,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let target_col = i18n.t("batch-crawl-col-target");
    let type_col = i18n.t("batch-crawl-col-type");
    let direct_label = i18n.t("batch-crawl-label-direct");
    let hops_label = i18n.t("batch-crawl-label-hops");
    let internal_intro = i18n.t_args(
        "batch-crawl-internal-intro",
        &[
            ("seed", crawl_links.seed_url.clone()),
            ("checked", crawl_links.checked_internal_links.to_string()),
            (
                "broken",
                crawl_links.broken_internal_links.len().to_string(),
            ),
        ],
    );
    builder = builder
        .add_component(Section::new(i18n.t("batch-section-broken-links-internal")).with_level(1))
        .add_component(TextBlock::new(internal_intro));

    if crawl_links.broken_internal_links.is_empty() {
        builder = builder.add_component(Callout::info(i18n.t("batch-crawl-no-broken-internal")));
    } else {
        let mut table = AuditTable::new(vec![
            TableColumn::new(i18n.t("batch-col-source")),
            TableColumn::new(target_col.clone()),
            TableColumn::new(i18n.t("batch-col-status-code")),
            TableColumn::new(type_col.clone()),
        ])
        .with_title(i18n.t("batch-table-broken-internal"));

        for row in &crawl_links.broken_internal_links {
            let typ_label = if row.redirect_hops > 0 {
                format!("→{} {}", row.redirect_hops, hops_label)
            } else {
                direct_label.to_string()
            };
            table = table.add_row(vec![
                truncate_url(&row.source_url, 30),
                truncate_url(&row.target_url, 38),
                row.status.clone(),
                typ_label,
            ]);
        }

        builder = builder.add_component(table);
    }

    // External broken links
    if !crawl_links.broken_external_links.is_empty() {
        let ext_intro = i18n.t_args(
            "batch-crawl-external-intro",
            &[
                ("checked", crawl_links.checked_external_links.to_string()),
                (
                    "broken",
                    crawl_links.broken_external_links.len().to_string(),
                ),
            ],
        );
        builder = builder
            .add_component(
                Section::new(i18n.t("batch-section-broken-links-external")).with_level(2),
            )
            .add_component(TextBlock::new(ext_intro));

        let mut ext_table = AuditTable::new(vec![
            TableColumn::new(i18n.t("batch-col-source")),
            TableColumn::new(target_col.clone()),
            TableColumn::new(i18n.t("batch-col-status-code")),
            TableColumn::new(type_col.clone()),
        ])
        .with_title(i18n.t("batch-table-broken-external"));

        for row in &crawl_links.broken_external_links {
            let typ_label = if row.redirect_hops > 0 {
                format!("→{} {}", row.redirect_hops, hops_label)
            } else {
                direct_label.to_string()
            };
            ext_table = ext_table.add_row(vec![
                truncate_url(&row.source_url, 30),
                truncate_url(&row.target_url, 38),
                row.status.clone(),
                typ_label,
            ]);
        }

        builder = builder.add_component(ext_table);
    } else if crawl_links.checked_external_links > 0 {
        let ext_clean_msg = i18n.t_args(
            "batch-crawl-external-clean",
            &[("checked", crawl_links.checked_external_links.to_string())],
        );
        builder = builder
            .add_component(Section::new(i18n.t("batch-section-external-links")).with_level(2))
            .add_component(Callout::info(ext_clean_msg));
    }

    // Redirect chains
    if !crawl_links.redirect_chains.is_empty() {
        let chain_intro = i18n.t_args(
            "batch-crawl-redirect-chains-intro",
            &[("count", crawl_links.redirect_chains.len().to_string())],
        );
        builder = builder
            .add_component(Section::new(i18n.t("batch-section-redirect-chains")).with_level(2))
            .add_component(TextBlock::new(chain_intro));

        let mut chain_table = AuditTable::new(vec![
            TableColumn::new(i18n.t("batch-col-source")),
            TableColumn::new(i18n.t("batch-col-target")),
            TableColumn::new("Hops"),
            TableColumn::new(i18n.t("batch-col-final-url")),
        ])
        .with_title(i18n.t("batch-redirect-chains-title"));

        for chain in &crawl_links.redirect_chains {
            chain_table = chain_table.add_row(vec![
                truncate_url(&chain.source_url, 28),
                truncate_url(&chain.target_url, 28),
                chain.hops.to_string(),
                truncate_url(&chain.final_url, 32),
            ]);
        }

        builder = builder.add_component(chain_table);
    }

    builder
}

pub(super) fn render_batch_seo_section(
    mut builder: renderreport::engine::ReportBuilder,
    pres: &BatchPresentation,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    builder = builder.add_component(
        SectionHeaderSplit::new(
            i18n.t("batch-seo-potential-title"),
            i18n.t("batch-seo-potential-intro"),
        )
        .with_level(1),
    );

    // Weakest pages
    if !pres.portfolio_summary.weakest_content_pages.is_empty() {
        let issues_title = i18n.t("batch-seo-issues-title");
        let mut issues_kv = KeyValueList::new().with_title(issues_title);
        for (url, page_type, score) in &pres.portfolio_summary.weakest_content_pages {
            let relevance =
                super::super::business_relevance(Some(page_type.as_str()), url, i18n.locale());
            let high_marker = if i18n.locale() == "en" {
                "high"
            } else {
                "hoch"
            };
            let impact = if relevance == high_marker {
                i18n.t("batch-seo-impact-ranking-loss")
            } else if *score < 30 {
                i18n.t("batch-seo-impact-weak-visibility")
            } else {
                i18n.t("batch-seo-impact-opt-potential")
            };
            let value = i18n.t_args(
                "batch-seo-recommendation-words",
                &[("page_type", page_type.clone()), ("impact", impact)],
            );
            let key_str = i18n.t_args(
                "batch-seo-profile-label",
                &[
                    ("url", truncate_url(url, 35)),
                    ("score", format!("{score}")),
                ],
            );
            issues_kv = issues_kv.add(key_str, value);
        }
        builder = builder.add_component(issues_kv);
    }

    // Distribution insights
    if !pres.portfolio_summary.distribution_insights.is_empty() {
        let action_label = i18n.t("batch-seo-action-needed");
        let panel_title = i18n.t("batch-seo-patterns-impact-title");
        let rows: Vec<ChecklistRow> = pres
            .portfolio_summary
            .distribution_insights
            .iter()
            .map(|insight| {
                let impact = if insight.contains("Thin") || insight.contains("dünn") {
                    i18n.t_args("batch-seo-impact-thin", &[("insight", insight.clone())])
                } else if insight.contains("Duplikat") || insight.contains("duplicate") {
                    i18n.t_args(
                        "batch-seo-impact-duplicate",
                        &[("insight", insight.clone())],
                    )
                } else {
                    insight.clone()
                };
                ChecklistRow::new(action_label.clone(), &impact).with_status("warn")
            })
            .collect();
        builder = builder.add_component(ChecklistPanel::new(rows).with_title(panel_title));
    }

    // Near-duplicates
    if !pres.portfolio_summary.near_duplicates.is_empty() {
        let near_dup_title = i18n.t("batch-seo-near-dup-title");
        let mut table = AuditTable::new(vec![
            TableColumn::new(i18n.t("batch-col-page-a")),
            TableColumn::new(i18n.t("batch-col-page-b")),
            TableColumn::new(i18n.t("batch-col-similarity")),
            TableColumn::new(i18n.t("batch-col-risk")),
        ])
        .with_title(near_dup_title);

        for (url_a, url_b, sim) in &pres.portfolio_summary.near_duplicates {
            let risk = if *sim >= 95 {
                i18n.t("batch-seo-risk-high")
            } else {
                i18n.t("batch-seo-risk-medium")
            };
            table = table.add_row(vec![
                truncate_url(url_a, 35),
                truncate_url(url_b, 35),
                format!("{sim}%"),
                risk.to_string(),
            ]);
        }
        builder = builder.add_component(table);
    }

    // Cross-page duplicate content (identical title / meta description / H1)
    if !pres.portfolio_summary.duplicate_content.is_empty() {
        let mut table = AuditTable::new(vec![
            TableColumn::new(i18n.t("batch-col-dup-type")),
            TableColumn::new(i18n.t("batch-col-dup-value")),
            TableColumn::new(i18n.t("batch-col-dup-count")),
            TableColumn::new(i18n.t("batch-col-pages-list")),
        ])
        .with_title(i18n.t("batch-seo-duplicate-title"));

        for group in &pres.portfolio_summary.duplicate_content {
            let kind_label = match group.kind.as_str() {
                "title" => i18n.t("batch-dup-kind-title"),
                "meta_description" => i18n.t("batch-dup-kind-description"),
                "h1" => i18n.t("batch-dup-kind-h1"),
                other => other.to_string(),
            };
            let examples = group
                .urls
                .iter()
                .take(3)
                .map(|u| truncate_url(u, 30))
                .collect::<Vec<_>>()
                .join(", ");
            table = table.add_row(vec![
                kind_label,
                group.value.clone(),
                group.urls.len().to_string(),
                examples,
            ]);
        }
        builder = builder.add_component(table);
    }

    // Canonical conflicts (noindex / og:url mismatch)
    if !pres.portfolio_summary.canonical_issues.is_empty() {
        let mut table = AuditTable::new(vec![
            TableColumn::new(i18n.t("batch-col-dup-type")),
            TableColumn::new(i18n.t("batch-col-page-a")),
            TableColumn::new(i18n.t("batch-col-dup-value")),
        ])
        .with_title(i18n.t("batch-seo-canonical-title"));

        for issue in &pres.portfolio_summary.canonical_issues {
            let kind_label = match issue.kind.as_str() {
                "noindex_conflict" => i18n.t("batch-canonical-noindex"),
                "og_url_mismatch" => i18n.t("batch-canonical-ogurl"),
                other => other.to_string(),
            };
            let detail = if issue.detail.chars().count() > 50 {
                format!("{}…", issue.detail.chars().take(50).collect::<String>())
            } else {
                issue.detail.clone()
            };
            table = table.add_row(vec![kind_label, truncate_url(&issue.url, 35), detail]);
        }
        builder = builder.add_component(table);
    }

    // Non-reciprocal hreflang relationships
    if !pres.portfolio_summary.hreflang_issues.is_empty() {
        let mut table = AuditTable::new(vec![
            TableColumn::new(i18n.t("batch-col-hreflang-source")),
            TableColumn::new(i18n.t("batch-col-hreflang-target")),
            TableColumn::new(i18n.t("batch-col-hreflang-lang")),
        ])
        .with_title(i18n.t("batch-seo-hreflang-title"));

        for issue in &pres.portfolio_summary.hreflang_issues {
            table = table.add_row(vec![
                truncate_url(&issue.source_url, 32),
                truncate_url(&issue.target_url, 32),
                issue.lang.clone(),
            ]);
        }
        builder = builder.add_component(table);
    }

    // Sitemap HTTP/indexability and orphan checks
    if !pres.portfolio_summary.sitemap_http_issues.is_empty() {
        let mut table = AuditTable::new(vec![
            TableColumn::new(i18n.t("batch-col-dup-type")),
            TableColumn::new(i18n.t("batch-col-page-a")),
            TableColumn::new(i18n.t("batch-col-status-code")),
            TableColumn::new(i18n.t("batch-col-dup-value")),
        ])
        .with_title(i18n.t("batch-seo-sitemap-http-title"));

        for issue in &pres.portfolio_summary.sitemap_http_issues {
            let kind_label = match issue.kind.as_str() {
                "status" => i18n.t("batch-sitemap-kind-status"),
                "redirect" => i18n.t("batch-sitemap-kind-redirect"),
                "noindex" => i18n.t("batch-sitemap-kind-noindex"),
                "fetch_error" => i18n.t("batch-sitemap-kind-fetch-error"),
                other => other.to_string(),
            };
            let detail = issue
                .final_url
                .as_ref()
                .map(|url| truncate_url(url, 35))
                .unwrap_or_else(|| truncate_url(&issue.detail, 45));
            table = table.add_row(vec![
                kind_label,
                truncate_url(&issue.url, 35),
                issue
                    .status_code
                    .map(|code| code.to_string())
                    .unwrap_or_else(|| "n/a".to_string()),
                detail,
            ]);
        }
        builder = builder.add_component(table);
    }

    if !pres.portfolio_summary.orphan_sitemap_urls.is_empty()
        || !pres.portfolio_summary.linked_not_in_sitemap.is_empty()
    {
        let mut table = AuditTable::new(vec![
            TableColumn::new(i18n.t("batch-col-dup-type")),
            TableColumn::new(i18n.t("batch-col-page-a")),
        ])
        .with_title(i18n.t("batch-seo-sitemap-orphan-title"));

        for url in pres.portfolio_summary.orphan_sitemap_urls.iter().take(20) {
            table = table.add_row(vec![
                i18n.t("batch-sitemap-kind-orphan"),
                truncate_url(url, 50),
            ]);
        }
        for url in pres.portfolio_summary.linked_not_in_sitemap.iter().take(20) {
            table = table.add_row(vec![
                i18n.t("batch-sitemap-kind-linked-missing"),
                truncate_url(url, 50),
            ]);
        }
        builder = builder.add_component(table);
    }

    // Page type distribution
    if !pres.portfolio_summary.page_type_distribution.is_empty() {
        let high_label = i18n.t("batch-relevance-high");
        let medium_label = i18n.t("batch-relevance-medium");
        let low_label = i18n.t("batch-relevance-low");
        let mut type_table = AuditTable::new(vec![
            TableColumn::new(i18n.t("batch-col-page-type")),
            TableColumn::new(i18n.t("batch-col-pages-list")),
            TableColumn::new(i18n.t("batch-col-share")),
            TableColumn::new(i18n.t("batch-col-relevance")),
        ])
        .with_title(i18n.t("batch-table-page-type-distribution"));

        for (label, count, pct) in &pres.portfolio_summary.page_type_distribution {
            let relevance = match label.as_str() {
                "Marketing / Landing Page"
                | "Transaktional / Utility"
                | "Transactional / Utility" => &high_label,
                "Editorial / Artikel"
                | "Editorial / Article"
                | "Strukturierter Wissensinhalt"
                | "Structured knowledge content" => &medium_label,
                "Thin / Minimal Content" => &low_label,
                _ => &medium_label,
            };
            type_table = type_table.add_row(vec![
                label.clone(),
                count.to_string(),
                format!("{pct}%"),
                relevance.to_string(),
            ]);
        }
        builder = builder.add_component(type_table);
    }

    // Schema distribution
    if !pres.portfolio_summary.schema_distribution.is_empty() {
        let total = pres.portfolio_summary.total_urls;
        let without = pres.portfolio_summary.pages_without_schema;
        let summary = if without == 0 {
            i18n.t_args("batch-schema-summary-all", &[("total", total.to_string())])
        } else {
            i18n.t_args(
                "batch-schema-summary-some",
                &[
                    ("without", without.to_string()),
                    ("total", total.to_string()),
                ],
            )
        };
        let schema_callout_title = i18n.t("batch-schema-callout-title");
        builder = builder.add_component(Callout::info(&summary).with_title(schema_callout_title));
        let mut schema_table = AuditTable::new(vec![
            TableColumn::new(i18n.t("batch-col-schema-type")).with_width("55%"),
            TableColumn::new(i18n.t("batch-col-pages-list")).with_width("20%"),
            TableColumn::new(i18n.t("batch-col-share")).with_width("25%"),
        ])
        .with_title(i18n.t("batch-table-schema-distribution"));
        for (schema_type, count) in &pres.portfolio_summary.schema_distribution {
            let pct = (*count * 100).checked_div(total).unwrap_or(0);
            schema_table = schema_table.add_row(vec![
                schema_type.clone(),
                count.to_string(),
                format!("{pct}%"),
            ]);
        }
        builder = builder.add_component(schema_table);
    }

    // Strongest pages
    if !pres.portfolio_summary.strongest_content_pages.is_empty() {
        let mut strengths = AuditTable::new(vec![
            TableColumn::new("URL"),
            TableColumn::new(i18n.t("batch-col-page-type")),
            TableColumn::new(i18n.t("batch-col-profile")),
        ])
        .with_title(i18n.t("batch-table-top-pages"));

        for (url, page_type, score) in &pres.portfolio_summary.strongest_content_pages {
            strengths = strengths.add_row(vec![
                truncate_url(url, 42),
                page_type.clone(),
                format!("{score}/100"),
            ]);
        }
        builder = builder.add_component(strengths);
    }

    builder
}

pub(super) fn render_batch_appendix(
    mut builder: renderreport::engine::ReportBuilder,
    pres: &BatchPresentation,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let appendix_intro = i18n.t("batch-appendix-intro");
    builder = builder.add_component(
        SectionHeaderSplit::new(i18n.t("section-appendix"), appendix_intro).with_level(1),
    );

    let rule_col = i18n.t("batch-appendix-col-rule");
    let elements_col = i18n.t("batch-appendix-col-elements");
    for url_appendix in &pres.appendix.per_url {
        if url_appendix.violations.is_empty() {
            continue;
        }

        builder =
            builder.add_component(Section::new(truncate_url(&url_appendix.url, 70)).with_level(2));

        let mut table = AuditTable::new(vec![
            TableColumn::new(rule_col.clone()),
            TableColumn::new(i18n.t("batch-col-severity")),
            TableColumn::new(i18n.t("batch-col-description")),
            TableColumn::new(elements_col.clone()),
        ]);

        for v in &url_appendix.violations {
            let elements = v
                .affected_elements
                .iter()
                .map(|e| e.selector.clone())
                .collect::<Vec<_>>()
                .join("; ");
            table = table.add_row(vec![
                format!(
                    "{} — {} ({}×)",
                    v.rule,
                    v.rule_name,
                    v.affected_elements.len()
                ),
                severity_label_i18n(v.severity, i18n),
                v.message.clone(),
                elements,
            ]);
        }
        builder = builder.add_component(table);
    }

    builder
}
