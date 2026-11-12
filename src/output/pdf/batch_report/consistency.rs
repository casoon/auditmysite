use crate::output::localized::is_english;
use fluent_bundle::FluentValue;
use renderreport::components::advanced::{ChecklistPanel, ChecklistRow, SectionHeaderSplit};
use renderreport::components::text::TextBlock;
use renderreport::components::{AuditTable, Callout, TableColumn};

use crate::i18n::I18n;
use crate::output::report_model::*;
use crate::util::short_url;

/// Render cross-page consistency analysis (issues #44/#45/#46).
/// Shows navigation, heading, and canonical consistency stats with any
/// findings as warning rows.
pub(super) fn render_batch_consistency(
    mut builder: renderreport::engine::ReportBuilder,
    consistency: &crate::audit::batch_consistency::BatchConsistencyAnalysis,
    pres: &BatchPresentation,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    // The heading check counts H1 per page; it measures no WCAG criterion.
    // 3.2.4 (Consistent Identification) is not measured at all and stands as
    // a manual review in the criteria table below (#1039).
    builder = builder.add_component(
        SectionHeaderSplit::new(
            i18n.t("batch-consistency-title"),
            i18n.t("batch-consistency-intro"),
        )
        .with_level(2),
    );
    let consistent = |detail_key: &str| {
        vec![
            ChecklistRow::new(i18n.t("batch-consistency-consistent"), i18n.t(detail_key))
                .with_status("good"),
        ]
    };
    let missing_on = |missing: usize, total: usize| {
        i18n.t_args(
            "batch-consistency-missing-on-pages",
            &[
                ("missing", FluentValue::from(missing as i64)),
                ("total", FluentValue::from(total as i64)),
            ],
        )
    };

    let nav = &consistency.navigation;
    let nav_title = i18n.t_args(
        "batch-consistency-nav-title",
        &[
            ("main", FluentValue::from(nav.pages_with_main_nav as i64)),
            ("skip", FluentValue::from(nav.pages_with_skip_link as i64)),
            ("total", FluentValue::from(nav.total_pages as i64)),
        ],
    );
    let nav_rows: Vec<ChecklistRow> = if nav.findings.is_empty() {
        consistent("batch-consistency-nav-consistent")
    } else {
        let mut rows = Vec::new();
        if nav.pages_with_main_nav < nav.total_pages {
            let missing = nav.total_pages - nav.pages_with_main_nav;
            rows.push(
                ChecklistRow::new(
                    i18n.t("batch-consistency-main-nav"),
                    missing_on(missing, nav.total_pages),
                )
                .with_status("warn"),
            );
        }
        if nav.pages_with_skip_link < nav.total_pages {
            let missing = nav.total_pages - nav.pages_with_skip_link;
            rows.push(
                ChecklistRow::new(
                    i18n.t("batch-consistency-skip-link"),
                    missing_on(missing, nav.total_pages),
                )
                .with_status("warn"),
            );
        }
        rows
    };
    builder = builder.add_component(ChecklistPanel::new(nav_rows).with_title(&nav_title));

    let h = &consistency.headings;
    let head_title = i18n.t_args(
        "batch-consistency-headings-title",
        &[
            ("single", FluentValue::from(h.pages_with_single_h1 as i64)),
            ("total", FluentValue::from(h.total_pages as i64)),
            ("none", FluentValue::from(h.pages_with_no_h1 as i64)),
            (
                "multiple",
                FluentValue::from(h.pages_with_multiple_h1 as i64),
            ),
        ],
    );
    let head_rows: Vec<ChecklistRow> = if h.findings.is_empty() {
        consistent("batch-consistency-headings-consistent")
    } else {
        let mut rows = Vec::new();
        if h.pages_with_no_h1 > 0 {
            rows.push(
                ChecklistRow::new(
                    i18n.t("batch-consistency-missing-h1"),
                    i18n.t_args(
                        "batch-consistency-missing-h1-detail",
                        &[("count", FluentValue::from(h.pages_with_no_h1 as i64))],
                    ),
                )
                .with_status("warn"),
            );
        }
        if h.pages_with_multiple_h1 > 0 {
            rows.push(
                ChecklistRow::new(
                    i18n.t("batch-consistency-multiple-h1"),
                    i18n.t_args(
                        "batch-consistency-multiple-h1-detail",
                        &[("count", FluentValue::from(h.pages_with_multiple_h1 as i64))],
                    ),
                )
                .with_status("warn"),
            );
        }
        rows
    };
    builder = builder.add_component(ChecklistPanel::new(head_rows).with_title(&head_title));

    let c = &consistency.canonical;
    let canon_title = i18n.t_args(
        "batch-consistency-canonical-title",
        &[
            ("www", FluentValue::from(c.www_count as i64)),
            ("non_www", FluentValue::from(c.non_www_count as i64)),
            ("missing", FluentValue::from(c.missing_count as i64)),
            ("total", FluentValue::from(c.total_pages as i64)),
        ],
    );
    let canon_rows: Vec<ChecklistRow> = if c.findings.is_empty() {
        consistent("batch-consistency-canonical-consistent")
    } else {
        let mut rows = Vec::new();
        if c.www_count > 0 && c.non_www_count > 0 {
            rows.push(
                ChecklistRow::new(
                    i18n.t("batch-consistency-mixed-hosts"),
                    i18n.t_args(
                        "batch-consistency-mixed-hosts-detail",
                        &[
                            ("www", FluentValue::from(c.www_count as i64)),
                            ("non_www", FluentValue::from(c.non_www_count as i64)),
                        ],
                    ),
                )
                .with_status("warn"),
            );
        }
        if c.missing_count > 0 {
            rows.push(
                ChecklistRow::new(
                    i18n.t("batch-consistency-missing-canonical"),
                    // Optional tag: without it search engines pick the
                    // canonical URL themselves (plan 88).
                    i18n.t_args(
                        "batch-consistency-missing-canonical-detail",
                        &[
                            ("missing", FluentValue::from(c.missing_count as i64)),
                            ("total", FluentValue::from(c.total_pages as i64)),
                        ],
                    ),
                )
                .with_status("info"),
            );
        }
        rows
    };
    builder = builder.add_component(ChecklistPanel::new(canon_rows).with_title(&canon_title));

    builder = render_security_header_consistency(builder, pres, i18n);

    builder = render_consistency_wcag_cross_page(builder, consistency, i18n);

    builder = render_consistency_help_deviations(builder, consistency, i18n);

    builder = render_consistency_orphan_pages(builder, consistency, i18n);

    builder = render_consistency_schema_conflicts(builder, consistency, i18n);

    let structured = &consistency.structured_data;
    // The SEO section already shows the pages per schema type, with their
    // share; the same table a second time under another title is left out
    // (#1039).
    if pres.portfolio_summary.schema_distribution.is_empty() {
        builder = render_consistency_structured_types(builder, structured, i18n);
    }

    builder = render_consistency_structured_blockers(builder, structured, i18n);

    builder = render_consistency_structured_parity(builder, structured, i18n);

    builder = render_consistency_page_type_matrix(builder, structured, i18n);

    builder = render_consistency_identity_findings(builder, structured, i18n);

    builder = render_consistency_top_topics(builder, pres, i18n);

    builder = render_consistency_overlap_pairs(builder, pres, i18n);

    builder
}

fn render_consistency_wcag_cross_page(
    mut builder: renderreport::engine::ReportBuilder,
    consistency: &crate::audit::batch_consistency::BatchConsistencyAnalysis,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    if !consistency.wcag_cross_page.is_empty() {
        let en = is_english(i18n);
        let mut table = AuditTable::new(vec![
            TableColumn::new(i18n.t("batch-consistency-col-criterion")).with_width("28%"),
            TableColumn::new("Status").with_width("20%"),
            TableColumn::new(i18n.t("batch-consistency-col-basis")).with_width("52%"),
        ])
        .with_title(i18n.t("batch-consistency-wcag-title"));
        for assessment in &consistency.wcag_cross_page {
            let status = i18n.t(match assessment.status.as_str() {
                "no_inconsistency_detected" => "batch-consistency-status-no-inconsistency",
                "warning" => "batch-consistency-status-warning",
                _ => "batch-consistency-status-manual-review",
            });
            let basis = if assessment.criterion.starts_with("3.2.6") {
                crate::audit::batch_consistency::consistent_help_basis(&consistency.help, en)
            } else {
                // The English message is the canonical basis itself; the
                // German one states the same, it used to drop the candidates
                // and unaudited-links part of 2.4.5 (#1039).
                let key = match assessment.criterion.as_str() {
                    value if value.starts_with("3.2.3") => "batch-consistency-basis-navigation",
                    value if value.starts_with("3.2.4") => "batch-consistency-basis-identification",
                    _ => "batch-consistency-basis-multiple-ways",
                };
                i18n.t_args(
                    key,
                    &[("basis", FluentValue::from(assessment.basis.as_str()))],
                )
            };
            table = table.add_row(vec![
                criterion_label(&assessment.criterion, en),
                status,
                basis,
            ]);
        }
        builder = builder.add_component(table);
    }

    builder
}

/// "3.2.3 Consistent Navigation" in the run language. The JSON keeps the
/// canonical English name; the German one comes from the WCAG criterion
/// table the EN 301 549 mapping already translates (#572), so there is no
/// second translation to drift (#1039).
pub(super) fn criterion_label(criterion: &str, en: bool) -> String {
    if en {
        return criterion.to_string();
    }
    let number = criterion.split_whitespace().next().unwrap_or(criterion);
    crate::wcag::en301549::EN301549_WEB_CLAUSES
        .iter()
        .find(|clause| clause.wcag == number)
        .map_or_else(
            || criterion.to_string(),
            |clause| format!("{number} {}", clause.title_de),
        )
}

fn render_consistency_help_deviations(
    mut builder: renderreport::engine::ReportBuilder,
    consistency: &crate::audit::batch_consistency::BatchConsistencyAnalysis,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    if !consistency.help.deviations.is_empty() {
        let en = is_english(i18n);
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
                i18n.t_args(
                    "batch-consistency-help-more",
                    &[("count", FluentValue::from(rest as i64))],
                ),
                i18n.t("batch-consistency-see-json"),
            ));
        }
        builder = builder.add_component(
            ChecklistPanel::new(rows).with_title(i18n.t("batch-consistency-help-title")),
        );
    }

    builder
}

fn render_consistency_orphan_pages(
    mut builder: renderreport::engine::ReportBuilder,
    consistency: &crate::audit::batch_consistency::BatchConsistencyAnalysis,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    if !consistency.orphan_pages.orphan_urls.is_empty() {
        let rows = consistency
            .orphan_pages
            .orphan_urls
            .iter()
            .map(|url| {
                ChecklistRow::new(i18n.t("batch-consistency-orphan-row"), url).with_status("warn")
            })
            .collect();
        builder = builder
            .add_component(
                ChecklistPanel::new(rows).with_title(i18n.t("batch-consistency-orphan-title")),
            )
            // Counted from the links between the audited pages only
            // (`inbound_link_counts`); unlike the sitemap link graph, the
            // fetched homepage's links are not in it (#1039).
            .add_component(TextBlock::new(i18n.t("batch-consistency-orphan-note")));
    }

    builder
}

fn render_consistency_schema_conflicts(
    mut builder: renderreport::engine::ReportBuilder,
    consistency: &crate::audit::batch_consistency::BatchConsistencyAnalysis,
    i18n: &I18n,
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
        builder = builder.add_component(
            ChecklistPanel::new(rows)
                .with_title(i18n.t("batch-consistency-schema-conflicts-title")),
        );
    }

    builder
}

fn render_consistency_structured_types(
    mut builder: renderreport::engine::ReportBuilder,
    structured: &crate::audit::batch_consistency::StructuredDataConsistency,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    if !structured.type_distribution.is_empty() {
        let mut table = AuditTable::new(vec![
            TableColumn::new(i18n.t("batch-col-schema-type")).with_width("68%"),
            TableColumn::new(i18n.t("batch-consistency-col-audited-pages")).with_width("32%"),
        ])
        .with_title(i18n.t("batch-consistency-structured-types-title"));
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
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    if !structured.recurring_blockers.is_empty() {
        let rows = structured
            .recurring_blockers
            .iter()
            .map(|finding| {
                ChecklistRow::new(
                    &finding.key,
                    i18n.t_args(
                        "batch-consistency-blocker-detail",
                        &[("count", FluentValue::from(finding.affected_pages as i64))],
                    ),
                )
                .with_status("warn")
            })
            .collect();
        builder = builder.add_component(
            ChecklistPanel::new(rows).with_title(i18n.t("batch-consistency-blockers-title")),
        );
    }

    builder
}

fn render_consistency_structured_parity(
    mut builder: renderreport::engine::ReportBuilder,
    structured: &crate::audit::batch_consistency::StructuredDataConsistency,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    if !structured.parity_mismatches.is_empty() {
        let rows = structured
            .parity_mismatches
            .iter()
            .map(|finding| {
                ChecklistRow::new(
                    &finding.key,
                    i18n.t_args(
                        "batch-consistency-parity-detail",
                        &[("count", FluentValue::from(finding.affected_pages as i64))],
                    ),
                )
                .with_status("warn")
            })
            .collect();
        builder = builder.add_component(
            ChecklistPanel::new(rows).with_title(i18n.t("batch-consistency-parity-title")),
        );
    }

    builder
}

/// The schema check classifies a page on its own (page intent and URL) to
/// pick the schema.org types that fit; that kind is not the content page type
/// of the other sections and is labelled as its own column, so one URL does
/// not read as two page types (#1039).
fn render_consistency_page_type_matrix(
    mut builder: renderreport::engine::ReportBuilder,
    structured: &crate::audit::batch_consistency::StructuredDataConsistency,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = is_english(i18n);
    if !structured.page_type_matrix.is_empty() {
        builder = builder.add_component(Callout::info(i18n.t("batch-schema-matrix-note")));
        let mut table = AuditTable::new(vec![
            TableColumn::new("URL").with_width("31%"),
            TableColumn::new(i18n.t("batch-schema-matrix-col-kind")).with_width("19%"),
            TableColumn::new("Status").with_width("16%"),
            TableColumn::new(i18n.t("batch-consistency-matrix-col-types")).with_width("34%"),
        ])
        .with_title(i18n.t("batch-schema-matrix-title"));
        for row in &structured.page_type_matrix {
            table = table.add_row(vec![
                short_url(&row.url, 48),
                schema_page_kind_label(&row.page_kind, en),
                schema_coverage_label(&row.coverage_status, i18n),
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

/// Security headers across the audited pages (plan 93, step 5): deviations
/// one row each, uniform headers in one summary row.
fn render_security_header_consistency(
    builder: renderreport::engine::ReportBuilder,
    pres: &BatchPresentation,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let headers = &pres.portfolio_summary.security_header_consistency;
    let Some(checked) = headers.first().map(|h| h.pages_checked) else {
        return builder;
    };
    let mut rows = Vec::new();
    let mut uniform_set = Vec::new();
    let mut uniform_missing = Vec::new();
    for h in headers {
        let partial = h.pages_present > 0 && h.pages_present < h.pages_checked;
        if partial {
            let examples = h
                .missing_on
                .iter()
                .map(|u| short_url(u, 40))
                .collect::<Vec<_>>()
                .join(", ");
            rows.push(
                ChecklistRow::new(
                    &h.header,
                    i18n.t_args(
                        "batch-consistency-header-partial",
                        &[
                            ("present", FluentValue::from(h.pages_present as i64)),
                            ("checked", FluentValue::from(h.pages_checked as i64)),
                            ("examples", FluentValue::from(examples)),
                        ],
                    ),
                )
                .with_status("warn"),
            );
        } else if h.distinct_values > 1 {
            rows.push(
                ChecklistRow::new(
                    &h.header,
                    i18n.t_args(
                        "batch-consistency-header-values",
                        &[
                            ("values", FluentValue::from(h.distinct_values as i64)),
                            ("present", FluentValue::from(h.pages_present as i64)),
                        ],
                    ),
                )
                .with_status("warn"),
            );
        } else if h.pages_present == h.pages_checked {
            uniform_set.push(h.header.as_str());
        } else {
            uniform_missing.push(h.header.as_str());
        }
    }
    if !uniform_set.is_empty() || !uniform_missing.is_empty() {
        let mut parts = Vec::new();
        if !uniform_set.is_empty() {
            parts.push(i18n.t_args(
                "batch-consistency-header-uniform-set",
                &[("headers", FluentValue::from(uniform_set.join(", ")))],
            ));
        }
        if !uniform_missing.is_empty() {
            parts.push(i18n.t_args(
                "batch-consistency-header-uniform-missing",
                &[("headers", FluentValue::from(uniform_missing.join(", ")))],
            ));
        }
        rows.push(
            ChecklistRow::new(i18n.t("batch-consistency-header-uniform"), parts.join("; "))
                .with_status("info"),
        );
    }
    builder.add_component(ChecklistPanel::new(rows).with_title(i18n.t_args(
        "batch-consistency-headers-title",
        &[("checked", FluentValue::from(checked as i64))],
    )))
}

/// Page-type key of the schema matrix in the run language; the JSON keeps
/// the canonical key (plan 89).
fn schema_page_kind_label(key: &str, en: bool) -> String {
    serde_json::from_value::<crate::seo::schema_fit::SchemaPageKind>(serde_json::Value::String(
        key.to_string(),
    ))
    .map(|kind| kind.label(en).to_string())
    .unwrap_or_else(|_| key.to_string())
}

/// Coverage-status key of the schema matrix in the run language.
fn schema_coverage_label(key: &str, i18n: &I18n) -> String {
    use crate::seo::schema_fit::SchemaCoverageStatus;
    let status =
        serde_json::from_value::<SchemaCoverageStatus>(serde_json::Value::String(key.to_string()));
    i18n.t(match status {
        Ok(SchemaCoverageStatus::Complete) => "batch-consistency-coverage-complete",
        Ok(SchemaCoverageStatus::Opportunity) => "batch-consistency-coverage-opportunity",
        Ok(SchemaCoverageStatus::ManualReview) => "batch-consistency-coverage-manual-review",
        Ok(SchemaCoverageStatus::NotApplicable) => "batch-consistency-coverage-not-applicable",
        Err(_) => return key.to_string(),
    })
}

fn render_consistency_identity_findings(
    mut builder: renderreport::engine::ReportBuilder,
    structured: &crate::audit::batch_consistency::StructuredDataConsistency,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    if !structured.identity_findings.is_empty() {
        let rows = structured
            .identity_findings
            .iter()
            .map(|finding| {
                // The English message is the stored finding itself.
                let key = if finding.contains("different @id values") {
                    "batch-consistency-identity-ids-differ"
                } else {
                    "batch-consistency-identity-id-missing"
                };
                ChecklistRow::new(
                    i18n.t("batch-consistency-identity-row"),
                    i18n.t_args(key, &[("finding", FluentValue::from(finding.as_str()))]),
                )
                .with_status("warn")
            })
            .collect();
        builder = builder.add_component(
            ChecklistPanel::new(rows).with_title(i18n.t("batch-consistency-identity-title")),
        );
    }

    builder
}

/// The topics found on at least two audited pages. A topic of one page is
/// that page's subject, not a topic across the site (#1039).
pub(super) fn topics_on_several_pages(
    topics: &[(String, usize)],
) -> impl Iterator<Item = &(String, usize)> {
    topics.iter().filter(|(_, pages)| *pages >= 2)
}

fn render_consistency_top_topics(
    mut builder: renderreport::engine::ReportBuilder,
    pres: &BatchPresentation,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let rows: Vec<ChecklistRow> = topics_on_several_pages(&pres.portfolio_summary.top_topics)
        .take(10)
        .map(|(topic, pages)| {
            ChecklistRow::new(topic, i18n.count("batch-topic-pages", *pages)).with_status("good")
        })
        .collect();
    if !rows.is_empty() {
        builder = builder
            .add_component(ChecklistPanel::new(rows).with_title(i18n.t("batch-topics-title")));
    }

    builder
}

fn render_consistency_overlap_pairs(
    mut builder: renderreport::engine::ReportBuilder,
    pres: &BatchPresentation,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    if !pres.portfolio_summary.overlap_pairs.is_empty() {
        let rows = pres
            .portfolio_summary
            .overlap_pairs
            .iter()
            .take(10)
            .map(|pair| {
                let terms = pair.shared_terms.join(", ");
                ChecklistRow::new(
                    format!(
                        "{} ↔ {}",
                        short_url(&pair.url_a, 40),
                        short_url(&pair.url_b, 40)
                    ),
                    i18n.t_args(
                        "batch-consistency-overlap-detail",
                        &[
                            ("percent", FluentValue::from(pair.similarity_percent as i64)),
                            ("terms", FluentValue::from(terms)),
                        ],
                    ),
                )
                .with_status("warn")
            })
            .collect();
        builder = builder.add_component(
            ChecklistPanel::new(rows).with_title(i18n.t("batch-consistency-overlap-title")),
        );
    }

    builder
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_matrix_keys_are_localized() {
        let de = I18n::new("de").unwrap();
        let en = I18n::new("en").unwrap();
        assert_eq!(
            schema_coverage_label("manual_review", &de),
            "Manuell prüfen"
        );
        assert_eq!(
            schema_coverage_label("opportunity", &en),
            "Optional addition"
        );
        assert_ne!(
            schema_page_kind_label("corporate_page", false),
            "corporate_page"
        );
        assert_ne!(
            schema_page_kind_label("corporate_page", false),
            schema_page_kind_label("corporate_page", true)
        );
        // unknown keys pass through rather than vanish
        assert_eq!(schema_coverage_label("something_new", &de), "something_new");
    }
}
