use renderreport::components::advanced::{ChecklistPanel, ChecklistRow, SectionHeaderSplit};
use renderreport::components::{AuditTable, TableColumn};

use crate::i18n::I18n;
use crate::output::report_model::*;
use crate::util::truncate_url;

/// Render cross-page consistency analysis (issues #44/#45/#46).
/// Shows navigation, heading, and canonical consistency stats with any
/// findings as warning rows.
pub(super) fn render_batch_consistency(
    mut builder: renderreport::engine::ReportBuilder,
    consistency: &crate::audit::batch_consistency::BatchConsistencyAnalysis,
    pres: &BatchPresentation,
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
        let mut rows = Vec::new();
        if nav.pages_with_main_nav < nav.total_pages {
            let missing = nav.total_pages - nav.pages_with_main_nav;
            rows.push(
                ChecklistRow::new(
                    if en {
                        "Main navigation"
                    } else {
                        "Hauptnavigation"
                    },
                    if en {
                        format!("Missing on {missing} of {} audited pages.", nav.total_pages)
                    } else {
                        format!(
                            "Fehlt auf {missing} von {} geprüften Seiten.",
                            nav.total_pages
                        )
                    },
                )
                .with_status("warn"),
            );
        }
        if nav.pages_with_skip_link < nav.total_pages {
            let missing = nav.total_pages - nav.pages_with_skip_link;
            rows.push(
                ChecklistRow::new(
                    if en { "Skip link" } else { "Skip-Link" },
                    if en {
                        format!("Missing on {missing} of {} audited pages.", nav.total_pages)
                    } else {
                        format!(
                            "Fehlt auf {missing} von {} geprüften Seiten.",
                            nav.total_pages
                        )
                    },
                )
                .with_status("warn"),
            );
        }
        rows
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
        let mut rows = Vec::new();
        if h.pages_with_no_h1 > 0 {
            rows.push(
                ChecklistRow::new(
                    if en { "Missing H1" } else { "Fehlende H1" },
                    if en {
                        format!("{} audited pages have no H1.", h.pages_with_no_h1)
                    } else {
                        format!("{} geprüfte Seiten haben keine H1.", h.pages_with_no_h1)
                    },
                )
                .with_status("warn"),
            );
        }
        if h.pages_with_multiple_h1 > 0 {
            rows.push(
                ChecklistRow::new(
                    if en {
                        "Multiple H1 headings"
                    } else {
                        "Mehrere H1-Überschriften"
                    },
                    if en {
                        format!(
                            "{} audited pages have multiple H1 headings.",
                            h.pages_with_multiple_h1
                        )
                    } else {
                        format!(
                            "{} geprüfte Seiten haben mehrere H1-Überschriften.",
                            h.pages_with_multiple_h1
                        )
                    },
                )
                .with_status("warn"),
            );
        }
        rows
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
        let mut rows = Vec::new();
        if c.www_count > 0 && c.non_www_count > 0 {
            rows.push(
                ChecklistRow::new(
                    if en {
                        "Mixed host strategy"
                    } else {
                        "Gemischte Host-Strategie"
                    },
                    if en {
                        format!(
                            "{} pages use www and {} use non-www canonicals.",
                            c.www_count, c.non_www_count
                        )
                    } else {
                        format!(
                            "{} Seiten verwenden www- und {} Seiten nicht-www-Canonicals.",
                            c.www_count, c.non_www_count
                        )
                    },
                )
                .with_status("warn"),
            );
        }
        if c.missing_count > 0 {
            rows.push(
                ChecklistRow::new(
                    if en {
                        "Missing canonical"
                    } else {
                        "Fehlendes Canonical"
                    },
                    if en {
                        format!(
                            "Missing on {} of {} audited pages.",
                            c.missing_count, c.total_pages
                        )
                    } else {
                        format!(
                            "Fehlt auf {} von {} geprüften Seiten.",
                            c.missing_count, c.total_pages
                        )
                    },
                )
                .with_status("warn"),
            );
        }
        rows
    };
    builder = builder.add_component(ChecklistPanel::new(canon_rows).with_title(&canon_title));

    builder = render_consistency_wcag_cross_page(builder, consistency, en);

    builder = render_consistency_help_deviations(builder, consistency, en);

    builder = render_consistency_orphan_pages(builder, consistency, en);

    builder = render_consistency_schema_conflicts(builder, consistency, en);

    let structured = &consistency.structured_data;
    builder = render_consistency_structured_types(builder, structured, en);

    builder = render_consistency_structured_blockers(builder, structured, en);

    builder = render_consistency_structured_parity(builder, structured, en);

    builder = render_consistency_page_type_matrix(builder, structured, en);

    builder = render_consistency_identity_findings(builder, structured, en);

    builder = render_consistency_top_topics(builder, pres, en);

    builder = render_consistency_overlap_pairs(builder, pres, en);

    builder
}

fn render_consistency_wcag_cross_page(
    mut builder: renderreport::engine::ReportBuilder,
    consistency: &crate::audit::batch_consistency::BatchConsistencyAnalysis,
    en: bool,
) -> renderreport::engine::ReportBuilder {
    if !consistency.wcag_cross_page.is_empty() {
        let mut table = AuditTable::new(vec![
            TableColumn::new(if en { "Criterion" } else { "Kriterium" }).with_width("28%"),
            TableColumn::new("Status").with_width("20%"),
            TableColumn::new(if en {
                "Assessment basis"
            } else {
                "Prüfgrundlage"
            })
            .with_width("52%"),
        ])
        .with_title(if en {
            "Cross-page WCAG assessment"
        } else {
            "Seitenübergreifende WCAG-Prüfung"
        });
        for assessment in &consistency.wcag_cross_page {
            let status = match (assessment.status.as_str(), en) {
                ("no_inconsistency_detected", true) => "No inconsistency detected",
                ("no_inconsistency_detected", false) => "Keine Inkonsistenz erkannt",
                ("warning", true) => "Warning",
                ("warning", false) => "Warnung",
                (_, true) => "Manual review",
                (_, false) => "Manuell prüfen",
            };
            let basis = if assessment.criterion.starts_with("3.2.6") {
                crate::audit::batch_consistency::consistent_help_basis(&consistency.help, en)
            } else if en {
                assessment.basis.clone()
            } else {
                match assessment.criterion.as_str() {
                    value if value.starts_with("3.2.3") => "Hauptnavigation und Skip-Link-Vorkommen wurden über alle geprüften Seiten verglichen.",
                    value if value.starts_with("3.2.4") => "Gleichartige Bedienelemente benötigen einen seitenübergreifenden Vergleich ihrer zugänglichen Namen; die aktuelle Evidenz reicht nicht für eine Konformitätsaussage.",
                    _ => "Eingehende Links im geprüften Seitenset wurden ausgewertet; Suche, Sitemap und Ausnahmen für Prozessschritte müssen manuell bestätigt werden.",
                }.to_string()
            };
            table = table.add_row(vec![
                assessment.criterion.clone(),
                status.to_string(),
                basis,
            ]);
        }
        builder = builder.add_component(table);
    }

    builder
}

fn render_consistency_help_deviations(
    mut builder: renderreport::engine::ReportBuilder,
    consistency: &crate::audit::batch_consistency::BatchConsistencyAnalysis,
    en: bool,
) -> renderreport::engine::ReportBuilder {
    if !consistency.help.deviations.is_empty() {
        // Batch reports stay aggregated: the first rows illustrate the
        // pattern, the full list is in the JSON.
        const MAX_ROWS: usize = 10;
        let deviations = &consistency.help.deviations;
        let mut rows: Vec<ChecklistRow> = deviations
            .iter()
            .take(MAX_ROWS)
            .map(|d| {
                ChecklistRow::new(
                    crate::audit::batch_consistency::help_deviation_text(d, en),
                    d.url(),
                )
                .with_status("warn")
            })
            .collect();
        if deviations.len() > MAX_ROWS {
            let rest = deviations.len() - MAX_ROWS;
            rows.push(ChecklistRow::new(
                if en {
                    format!("{rest} further deviations")
                } else {
                    format!("{rest} weitere Abweichungen")
                },
                if en {
                    "see JSON report"
                } else {
                    "siehe JSON-Report"
                },
            ));
        }
        builder = builder.add_component(ChecklistPanel::new(rows).with_title(if en {
            "Consistent help (WCAG 3.2.6): deviations"
        } else {
            "Konsistente Hilfe (WCAG 3.2.6): Abweichungen"
        }));
    }

    builder
}

fn render_consistency_orphan_pages(
    mut builder: renderreport::engine::ReportBuilder,
    consistency: &crate::audit::batch_consistency::BatchConsistencyAnalysis,
    en: bool,
) -> renderreport::engine::ReportBuilder {
    if !consistency.orphan_pages.orphan_urls.is_empty() {
        let rows = consistency
            .orphan_pages
            .orphan_urls
            .iter()
            .map(|url| {
                ChecklistRow::new(
                    if en {
                        "Orphan candidate"
                    } else {
                        "Mögliche verwaiste Seite"
                    },
                    url,
                )
                .with_status("warn")
            })
            .collect();
        builder = builder.add_component(ChecklistPanel::new(rows).with_title(if en {
            "Pages without inbound links within the audited URL set"
        } else {
            "Seiten ohne eingehende Links innerhalb der geprüften URLs"
        }));
    }

    builder
}

fn render_consistency_schema_conflicts(
    mut builder: renderreport::engine::ReportBuilder,
    consistency: &crate::audit::batch_consistency::BatchConsistencyAnalysis,
    en: bool,
) -> renderreport::engine::ReportBuilder {
    if !consistency.schema_graph.conflicts.is_empty() {
        let rows = consistency
            .schema_graph
            .conflicts
            .iter()
            .map(|conflict| {
                ChecklistRow::new(&conflict.entity_id, conflict.conflicts.join("; "))
                    .with_status("warn")
            })
            .collect();
        builder = builder.add_component(ChecklistPanel::new(rows).with_title(if en {
            "Schema entity identity conflicts"
        } else {
            "Identitätskonflikte bei Schema-Entitäten"
        }));
    }

    builder
}

fn render_consistency_structured_types(
    mut builder: renderreport::engine::ReportBuilder,
    structured: &crate::audit::batch_consistency::StructuredDataConsistency,
    en: bool,
) -> renderreport::engine::ReportBuilder {
    if !structured.type_distribution.is_empty() {
        let mut table = AuditTable::new(vec![
            TableColumn::new(if en { "Schema type" } else { "Schema-Typ" }).with_width("68%"),
            TableColumn::new(if en {
                "Audited pages"
            } else {
                "Geprüfte Seiten"
            })
            .with_width("32%"),
        ])
        .with_title(if en {
            "Structured-data distribution"
        } else {
            "Verteilung strukturierter Daten"
        });
        for item in &structured.type_distribution {
            table = table.add_row(vec![item.schema_type.clone(), item.pages.to_string()]);
        }
        builder = builder.add_component(table);
    }

    builder
}

fn render_consistency_structured_blockers(
    mut builder: renderreport::engine::ReportBuilder,
    structured: &crate::audit::batch_consistency::StructuredDataConsistency,
    en: bool,
) -> renderreport::engine::ReportBuilder {
    if !structured.recurring_blockers.is_empty() {
        let rows = structured
            .recurring_blockers
            .iter()
            .map(|finding| {
                ChecklistRow::new(
                    &finding.key,
                    if en {
                        format!(
                            "Required condition missing on {} audited pages",
                            finding.affected_pages
                        )
                    } else {
                        format!(
                            "Pflichtbedingung fehlt auf {} geprüften Seiten",
                            finding.affected_pages
                        )
                    },
                )
                .with_status("warn")
            })
            .collect();
        builder = builder.add_component(ChecklistPanel::new(rows).with_title(if en {
            "Recurring structured-data blockers"
        } else {
            "Wiederkehrende Blocker bei strukturierten Daten"
        }));
    }

    builder
}

fn render_consistency_structured_parity(
    mut builder: renderreport::engine::ReportBuilder,
    structured: &crate::audit::batch_consistency::StructuredDataConsistency,
    en: bool,
) -> renderreport::engine::ReportBuilder {
    if !structured.parity_mismatches.is_empty() {
        let rows = structured
            .parity_mismatches
            .iter()
            .map(|finding| {
                ChecklistRow::new(
                    &finding.key,
                    if en {
                        format!(
                            "Visible content differs on {} audited pages",
                            finding.affected_pages
                        )
                    } else {
                        format!(
                            "Sichtbarer Inhalt weicht auf {} geprüften Seiten ab",
                            finding.affected_pages
                        )
                    },
                )
                .with_status("warn")
            })
            .collect();
        builder = builder.add_component(ChecklistPanel::new(rows).with_title(if en {
            "Recurring schema/content mismatches"
        } else {
            "Wiederkehrende Abweichungen zwischen Schema und Inhalt"
        }));
    }

    builder
}

fn render_consistency_page_type_matrix(
    mut builder: renderreport::engine::ReportBuilder,
    structured: &crate::audit::batch_consistency::StructuredDataConsistency,
    en: bool,
) -> renderreport::engine::ReportBuilder {
    if !structured.page_type_matrix.is_empty() {
        let mut table = AuditTable::new(vec![
            TableColumn::new("URL").with_width("31%"),
            TableColumn::new(if en { "Page type" } else { "Seitentyp" }).with_width("19%"),
            TableColumn::new("Status").with_width("16%"),
            TableColumn::new(if en {
                "Expected / detected"
            } else {
                "Erwartet / erkannt"
            })
            .with_width("34%"),
        ])
        .with_title(if en {
            "Page-type and schema-coverage matrix"
        } else {
            "Matrix aus Seitentyp und Schema-Abdeckung"
        });
        for row in &structured.page_type_matrix {
            table = table.add_row(vec![
                truncate_url(&row.url, 48),
                row.page_kind.clone(),
                row.coverage_status.clone(),
                format!(
                    "{} / {}",
                    if row.expected_types.is_empty() {
                        "-".to_string()
                    } else {
                        row.expected_types.join(", ")
                    },
                    if row.detected_types.is_empty() {
                        "-".to_string()
                    } else {
                        row.detected_types.join(", ")
                    }
                ),
            ]);
        }
        builder = builder.add_component(table);
    }

    builder
}

fn render_consistency_identity_findings(
    mut builder: renderreport::engine::ReportBuilder,
    structured: &crate::audit::batch_consistency::StructuredDataConsistency,
    en: bool,
) -> renderreport::engine::ReportBuilder {
    if !structured.identity_findings.is_empty() {
        let rows = structured
            .identity_findings
            .iter()
            .map(|finding| {
                let localized = if en {
                    finding.clone()
                } else if finding.contains("different @id values") {
                    "Die Organisationsidentität verwendet auf den geprüften Seiten unterschiedliche @id-Werte."
                        .to_string()
                } else {
                    "Mindestens ein Organisationsknoten besitzt keine stabile @id zur seitenübergreifenden Identitätsprüfung."
                        .to_string()
                };
                ChecklistRow::new(
                    if en { "Organization identity" } else { "Organisationsidentität" },
                    localized,
                )
                .with_status("warn")
            })
            .collect();
        builder = builder.add_component(ChecklistPanel::new(rows).with_title(if en {
            "Site-wide entity identity"
        } else {
            "Websiteweite Entitätsidentität"
        }));
    }

    builder
}

fn render_consistency_top_topics(
    mut builder: renderreport::engine::ReportBuilder,
    pres: &BatchPresentation,
    en: bool,
) -> renderreport::engine::ReportBuilder {
    if !pres.portfolio_summary.top_topics.is_empty() {
        let rows = pres
            .portfolio_summary
            .top_topics
            .iter()
            .take(10)
            .map(|(topic, pages)| {
                ChecklistRow::new(
                    topic,
                    if en {
                        format!("Present on {pages} audited pages")
                    } else {
                        format!("Auf {pages} geprüften Seiten vertreten")
                    },
                )
                .with_status("good")
            })
            .collect();
        builder = builder.add_component(ChecklistPanel::new(rows).with_title(if en {
            "Dominant content topics"
        } else {
            "Dominante Inhaltsthemen"
        }));
    }

    builder
}

fn render_consistency_overlap_pairs(
    mut builder: renderreport::engine::ReportBuilder,
    pres: &BatchPresentation,
    en: bool,
) -> renderreport::engine::ReportBuilder {
    if !pres.portfolio_summary.overlap_pairs.is_empty() {
        let rows = pres
            .portfolio_summary
            .overlap_pairs
            .iter()
            .take(10)
            .map(|(url_a, url_b, shared)| {
                ChecklistRow::new(
                    format!("{url_a} ↔ {url_b}"),
                    if en {
                        format!("{shared} shared topic terms")
                    } else {
                        format!("{shared} gemeinsame Themenbegriffe")
                    },
                )
                .with_status("warn")
            })
            .collect();
        builder = builder.add_component(ChecklistPanel::new(rows).with_title(if en {
            "Strong topic overlap between pages"
        } else {
            "Starke Themenüberschneidungen zwischen Seiten"
        }));
    }

    builder
}
