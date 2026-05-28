use renderreport::components::advanced::{
    ChecklistPanel, ChecklistRow, KeyValueList, List, PageBreak, SectionHeaderSplit,
    TableOfContents,
};
use renderreport::components::text::{Label, TextBlock};
use renderreport::components::{AuditTable, BenchmarkRow, BenchmarkTable, TableColumn};
use renderreport::prelude::Image;
use renderreport::prelude::*;

use crate::audit::BatchReport;
use crate::cli::ReportLevel;
use crate::i18n::I18n;
use crate::output::builder::build_batch_presentation;
use crate::output::report_model::*;
use crate::util::truncate_url;

use super::batch::build_batch_overview_grid;
use super::cover::{batch_certificate_label, build_batch_cover_score_row, certificate_badge_path};
use super::findings::first_sentence;
use super::helpers::{priority_label_i18n, role_label_i18n, severity_label_i18n};

// ─── Helper: Batch Report Assessment & Key Points ──────────────────────────

/// Clear batch assessment — no score, just interpretation
pub(super) fn build_batch_assessment(
    summary: &crate::output::report_model::PortfolioSummary,
    dist: &SeverityDistribution,
    i18n: &I18n,
) -> String {
    let en = i18n.locale() == "en";
    let score = summary.average_score.round() as u32;
    if dist.critical > 0 && score < 50 {
        if en {
            "Critical barriers — not WCAG conformant".to_string()
        } else {
            "Kritische Barrieren — nicht WCAG-konform".to_string()
        }
    } else if dist.critical > 0 {
        if en {
            "Technically solid, but legally risky".to_string()
        } else {
            "Technisch solide, aber rechtlich riskant".to_string()
        }
    } else if dist.high > 0 {
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
                "Keine Level-A-Verstöße, aber strukturelle Schwächen auf mehreren Seiten"
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
                item.effort.label().to_string(),
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

fn translate_interactive_category(category: &str, en: bool) -> String {
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

fn render_batch_interactive_summary(
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

// ─── Batch Report ───────────────────────────────────────────────────────────

pub fn generate_batch_pdf(batch: &BatchReport, config: &ReportConfig) -> anyhow::Result<Vec<u8>> {
    let (engine, built_report) = build_batch_report(batch, config)?;
    Ok(engine.render_pdf(&built_report)?)
}

/// Render the intermediate Typst source for a batch report (hidden `--debug-typ`).
pub fn generate_batch_typ(batch: &BatchReport, config: &ReportConfig) -> anyhow::Result<String> {
    let (engine, built_report) = build_batch_report(batch, config)?;
    Ok(engine.render_typ(&built_report)?)
}

fn build_batch_report(
    batch: &BatchReport,
    config: &ReportConfig,
) -> anyhow::Result<(renderreport::Engine, renderreport::RenderRequest)> {
    let engine = super::helpers::create_engine()?;
    let i18n = I18n::new(&config.locale)?;
    let pres = build_batch_presentation(batch);

    let domain = &pres.portfolio_summary.domain;
    let score = pres.portfolio_summary.average_score.round() as u32;

    let mut builder = engine
        .report("wcag-batch-audit")
        .title(&pres.cover.title)
        .subtitle(&pres.cover.url)
        .metadata("date", &pres.cover.date)
        .metadata("version", &pres.cover.version)
        .metadata("author", domain)
        .metadata("footer_prefix", "Audit:")
        .metadata("footer_link_url", "")
        .metadata(
            "footer_tagline",
            "A technical auditing platform by casoon.de",
        );

    let cover_logo_asset = super::cover_logo_asset(config);
    builder = super::register_cover_logo_asset(builder, config, cover_logo_asset);

    // ── Cover Page with Audit-Rahmen ────────────────────────────────
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

    // Audit-Rahmen box (matching single report style)
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
            &i18n,
        )?)
        .add_component(
            TextBlock::new(&pres.portfolio_summary.verdict_text)
                .with_size("11pt")
                .with_line_height("1.4em")
                .with_max_width("100%"),
        )
        .add_component(super::output_scope_callout(&i18n))
        .add_component(PageBreak::new());
    if config.level != ReportLevel::Executive {
        builder = builder.add_component(TableOfContents::new().with_depth(1));
    }

    let dist = &pres.portfolio_summary.severity_distribution;

    // ── 1. Status der Website ───────────────────────────────────────
    {
        // Block 1: Bewertung — klare Einordnung, Risiko primär
        let assessment = build_batch_assessment(&pres.portfolio_summary, dist, &i18n);
        let risk_title = assessment;
        let callout = match pres.portfolio_summary.risk_level.as_str() {
            "Kritisch" | "Hoch" | "Critical" | "High" => {
                Callout::warning(&pres.portfolio_summary.risk_summary).with_title(&risk_title)
            }
            "Mittel" | "Medium" => {
                Callout::info(&pres.portfolio_summary.risk_summary).with_title(&risk_title)
            }
            _ => Callout::success(&pres.portfolio_summary.risk_summary).with_title(&risk_title),
        };
        builder = builder
            .add_component(Section::new(i18n.t("batch-section-status")).with_level(1))
            .add_component(callout);
    }

    // Score overview cards (sekundär)
    builder = builder.add_component(build_batch_overview_grid(
        pres.portfolio_summary.total_urls as u32,
        score,
        pres.portfolio_summary.total_violations as u32,
        (dist.critical + dist.high) as u32,
        pres.portfolio_summary.crawl_links.as_ref().map(|links| {
            (links.broken_internal_links.len() + links.broken_external_links.len()) as u32
        }),
    ));

    // Block 2: Kernaussagen (max 3 Punkte)
    {
        let key_points = build_batch_key_points(&pres, dist, &i18n);
        let mut kp_list = List::new().with_title(i18n.t("narrative-key-points-title"));
        for point in &key_points {
            kp_list = kp_list.add_item(point);
        }
        builder = builder.add_component(kp_list);
    }

    // Auswirkungen
    {
        let en = i18n.locale() == "en";
        let a11y_avg = pres.portfolio_summary.average_score.round() as u32;
        let user_impact = match (a11y_avg, en) {
            (s, true) if s < 50 => "Critical — core functions are unreachable for users with disabilities",
            (s, false) if s < 50 => "Kritisch — zentrale Funktionen sind für Nutzer mit Einschränkungen nicht erreichbar",
            (s, true) if s < 70 => "Limited — structural issues impede users with assistive technologies",
            (s, false) if s < 70 => "Eingeschränkt — strukturelle Probleme behindern Nutzer mit Hilfstechnologien",
            (s, true) if s < 85 => "Good — individual barriers for assistive technologies on several pages",
            (s, false) if s < 85 => "Gut — einzelne Barrieren für Hilfstechnologien auf mehreren Seiten",
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
    }

    // Block 3: Handlungsempfehlung
    {
        let actions = build_batch_quick_actions(&pres, &i18n);
        if !actions.is_empty() {
            let rows: Vec<ChecklistRow> = actions
                .iter()
                .map(|action| ChecklistRow::new(action, "").with_status("warn"))
                .collect();
            builder = builder.add_component(
                ChecklistPanel::new(rows).with_title(i18n.t("narrative-quick-actions-title")),
            );
        }
    }

    // Module overview
    if !pres.portfolio_summary.module_averages.is_empty() {
        let mut module_kv = KeyValueList::new().with_title(i18n.t("batch-panel-module-averages"));
        for (name, score) in &pres.portfolio_summary.module_averages {
            module_kv = module_kv.add(name, format!("{}/100", score));
        }
        builder = builder.add_component(module_kv);
    }

    // ── 2. URL-Ranking ──────────────────────────────────────────────
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

    // ── 2b. Interaktive Accessibility-Journey ──────────────────────────
    if let Some(ref interactive) = pres.interactive_summary {
        if interactive.total_pages_tested > 0 {
            builder = render_batch_interactive_summary(
                builder,
                interactive,
                pres.portfolio_summary.total_urls,
                &i18n,
            );
        }
    }

    // ── 3. Top-Probleme (vereinheitlicht) ─────────────────────────
    let top_intro = i18n.t("batch-top-issues-intro");
    builder = builder.add_component(
        SectionHeaderSplit::new(i18n.t("batch-section-most-frequent"), top_intro).with_level(1),
    );

    // Übersichtstabelle mit Aufwand
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
                super::helpers::priority_label_i18n(issue.priority, &i18n),
            ]);
        }
        builder = builder.add_component(freq_table);
    }

    // Unified problem blocks — 1 Problem = 1 kompakter Block
    // (keine doppelten Cards + Details mehr)
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
        let effort_label = group.effort.label();
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

    // ── 4. Maßnahmenplan (mit Aufwand + Scope) ─────────────────────
    let action_intro = i18n.t("batch-action-plan-intro");
    builder = builder.add_component(
        SectionHeaderSplit::new(i18n.t("batch-action-plan-title"), action_intro).with_level(1),
    );
    builder = render_batch_action_plan_enhanced(builder, &pres.action_plan, &i18n);

    // ── 5a. Render Blocking (Batch) ─────────────────────────────────
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

    // ── 5b. Performance Budgets (Batch) ─────────────────────────────
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

    // ── 5. Technische URL-Matrix ───────────────────────────────────
    builder = builder.add_component(
        SectionHeaderSplit::new(
            i18n.t("batch-section-tech-url-matrix"),
            i18n.t("batch-section-tech-url-matrix-intro"),
        )
        .with_level(1),
    );

    if let Some(ref crawl_links) = pres.portfolio_summary.crawl_links {
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
            .add_component(
                Section::new(i18n.t("batch-section-broken-links-internal")).with_level(1),
            )
            .add_component(TextBlock::new(internal_intro));

        if crawl_links.broken_internal_links.is_empty() {
            builder =
                builder.add_component(Callout::info(i18n.t("batch-crawl-no-broken-internal")));
        } else {
            let mut table = AuditTable::new(vec![
                TableColumn::new(i18n.t("batch-col-source")),
                TableColumn::new(target_col.clone()),
                TableColumn::new(i18n.t("batch-col-status-code")),
                TableColumn::new(type_col.clone()),
            ])
            .with_title(i18n.t("batch-table-broken-internal"));

            for row in &crawl_links.broken_internal_links {
                let severity_color = match row.severity.as_str() {
                    "high" => "#dc2626",
                    "medium" => "#ea580c",
                    _ => "#ca8a04",
                };
                let typ_label = if row.redirect_hops > 0 {
                    format!("→{} {}", row.redirect_hops, hops_label)
                } else {
                    direct_label.to_string()
                };
                table = table.add_row(vec![
                    truncate_url(&row.source_url, 30),
                    truncate_url(&row.target_url, 38),
                    format!("\x1b[{}m{}\x1b[0m", severity_color, row.status),
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
    }

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
            super::format_word_count(row.word_count),
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

    // ── 6. Content & SEO — integriert mit Business-Impact ─────────
    builder = builder.add_component(
        SectionHeaderSplit::new(
            i18n.t("batch-seo-potential-title"),
            i18n.t("batch-seo-potential-intro"),
        )
        .with_level(1),
    );

    // Schwache Seiten zuerst — mit Business-Impact
    if !pres.portfolio_summary.weakest_content_pages.is_empty() {
        let issues_title = i18n.t("batch-seo-issues-title");
        let mut issues_kv = KeyValueList::new().with_title(issues_title);
        for (url, page_type, score) in &pres.portfolio_summary.weakest_content_pages {
            let relevance = super::business_relevance(Some(page_type.as_str()), url, i18n.locale());
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

    // Content-Auffälligkeiten als Business-Relevanz
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

    // Near-duplicates mit Business-Kontext
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

    // Seitentyp-Verteilung (kompakt)
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

    // Schema-Typ-Verteilung
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

    // Stärkste Seiten (kurz)
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

    // ── Cross-page consistency (issues #44/#45/#46) ─────────────────
    if let Some(ref consistency) = batch.consistency {
        builder = render_batch_consistency(builder, consistency, &i18n);
    }

    // ── Empfohlene nächste Schritte ───────────────────────────────
    builder = render_next_steps_batch(builder, &pres, &i18n);

    // ── 7. Anhang ───────────────────────────────────────────────────
    if config.level == ReportLevel::Technical && !pres.appendix.per_url.is_empty() {
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

            builder = builder
                .add_component(Section::new(truncate_url(&url_appendix.url, 70)).with_level(2));

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
                    severity_label_i18n(v.severity, &i18n),
                    v.message.clone(),
                    elements,
                ]);
            }
            builder = builder.add_component(table);
        }
    }

    let built_report = builder.build();
    Ok((engine, built_report))
}
