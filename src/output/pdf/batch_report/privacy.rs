use renderreport::components::advanced::{PageBreak, SectionHeaderSplit};
use renderreport::components::text::Label;
use renderreport::components::{AuditTable, TableColumn};

use crate::audit::BatchReport;
use crate::i18n::I18n;
use crate::privacy::aggregate::{MatrixRow, PrivacyBatchAggregate};
use crate::privacy::{aggregate_privacy, CheckOutcome, PrivacyCheck};
use crate::util::short_url;

use super::super::design::tokens;
use super::super::detail_modules::{
    category_label, check_short_label, kind_label, seat_label, spread_label,
};

const MAX_PROVIDER_ROWS: usize = 10;
const MAX_MATRIX_ROWS: usize = 20;

fn pages_of(i18n: &I18n, pages: usize, measured: usize) -> String {
    i18n.t_args(
        "privacy-batch-pages-of",
        &[("pages", pages as i64), ("measured", measured as i64)],
    )
}

fn table(title: String, headers: Vec<(String, &str)>, rows: Vec<Vec<String>>) -> AuditTable {
    let mut t = AuditTable::new(
        headers
            .into_iter()
            .map(|(h, w)| TableColumn::new(h).with_width(w))
            .collect(),
    )
    .with_title(title);
    for row in rows {
        t = t.add_row(row);
    }
    t
}

fn note(text: String, size: &str) -> Label {
    Label::new(text).with_size(size).with_color(tokens::NEUTRAL)
}

pub(super) fn kind_rows(agg: &PrivacyBatchAggregate, i18n: &I18n) -> Vec<Vec<String>> {
    agg.kinds
        .iter()
        .map(|k| {
            vec![
                kind_label(i18n, k.kind, k.document),
                pages_of(i18n, k.pages, agg.pages_measured),
                spread_label(i18n, k.spread),
            ]
        })
        .collect()
}

/// Providers on two or more pages; with fewer measured pages every provider
/// (a single page has no "recurring"). Returns the rows and how many
/// providers were left out as single-page.
pub(super) fn provider_rows(agg: &PrivacyBatchAggregate, i18n: &I18n) -> (Vec<Vec<String>>, usize) {
    let recurring_only = agg.pages_measured >= 2;
    let (shown, left_out): (Vec<_>, Vec<_>) = agg
        .providers
        .iter()
        .partition(|p| !recurring_only || p.pages >= 2);
    let rows = shown
        .into_iter()
        .take(MAX_PROVIDER_ROWS)
        .map(|p| {
            vec![
                p.provider.name.clone(),
                category_label(i18n, &p.provider.category),
                seat_label(i18n, &p.provider.jurisdiction),
                pages_of(i18n, p.pages, agg.pages_measured),
                p.requests.to_string(),
                spread_label(i18n, p.spread),
            ]
        })
        .collect();
    (rows, left_out.len())
}

pub(super) fn check_rows(agg: &PrivacyBatchAggregate, i18n: &I18n) -> Vec<Vec<String>> {
    agg.checks
        .iter()
        .map(|c| {
            vec![
                check_short_label(i18n, c.check),
                c.found.to_string(),
                c.not_found.to_string(),
                c.not_applicable.to_string(),
                c.not_measured.to_string(),
            ]
        })
        .collect()
}

fn cell_text(row: &MatrixRow, check_index: usize, i18n: &I18n) -> String {
    let cell = row.cells[check_index];
    match cell.outcome {
        CheckOutcome::Found => cell.findings.to_string(),
        CheckOutcome::NotFound => "0".to_string(),
        CheckOutcome::NotApplicable => "–".to_string(),
        CheckOutcome::NotMeasured => i18n.t("privacy-batch-matrix-nm"),
    }
}

pub(super) fn matrix_rows(agg: &PrivacyBatchAggregate, i18n: &I18n) -> Vec<Vec<String>> {
    agg.matrix
        .iter()
        .take(MAX_MATRIX_ROWS)
        .map(|row| {
            let mut cells = vec![short_url(&row.url, 44)];
            cells.extend((0..PrivacyCheck::ALL.len()).map(|i| cell_text(row, i, i18n)));
            cells
        })
        .collect()
}

/// Batch section (#872): cross-page aggregate only. One compact matrix row
/// per page, no per-page detail. Rendered only when a page carries privacy
/// data (`--privacy-check`).
pub(super) fn render_batch_privacy(
    mut builder: renderreport::engine::ReportBuilder,
    batch: &BatchReport,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let Some(agg) = aggregate_privacy(
        batch
            .reports
            .iter()
            .filter_map(|r| r.experience.privacy.as_ref().map(|p| (r.url.as_str(), p))),
    ) else {
        return builder;
    };

    builder = builder.add_component(PageBreak::new()).add_component(
        SectionHeaderSplit::new(i18n.t("privacy-batch-title"), i18n.t("privacy-batch-intro"))
            .with_level(2),
    );
    builder = builder.add_component(note(
        i18n.t_args(
            "privacy-batch-coverage",
            &[
                ("measured", agg.pages_measured as i64),
                ("total", batch.reports.len() as i64),
                ("failed", (batch.reports.len() - agg.pages_measured) as i64),
            ],
        ),
        "10.5pt",
    ));

    if agg.pages_measured == 0 {
        return builder.add_component(note(i18n.t("privacy-batch-nothing-measured"), "10.5pt"));
    }

    builder = builder.add_component(note(
        if agg.pages_with_findings == 0 {
            i18n.t("privacy-batch-none")
        } else {
            i18n.t_args(
                "privacy-batch-affected",
                &[
                    ("pages", agg.pages_with_findings as i64),
                    ("measured", agg.pages_measured as i64),
                ],
            )
        },
        "10.5pt",
    ));

    if !agg.kinds.is_empty() {
        builder = builder.add_component(table(
            i18n.t("privacy-batch-table-kinds"),
            vec![
                (i18n.t("privacy-col-indicator"), "56%"),
                (i18n.t("privacy-batch-col-pages"), "16%"),
                (i18n.t("privacy-batch-col-spread"), "28%"),
            ],
            kind_rows(&agg, i18n),
        ));
    }

    let (providers, single_only) = provider_rows(&agg, i18n);
    if !providers.is_empty() {
        builder = builder.add_component(table(
            i18n.t("privacy-batch-table-providers"),
            vec![
                (i18n.t("privacy-col-provider"), "22%"),
                (i18n.t("privacy-col-category"), "16%"),
                (i18n.t("privacy-col-seat"), "14%"),
                (i18n.t("privacy-batch-col-pages"), "14%"),
                (i18n.t("privacy-col-requests"), "10%"),
                (i18n.t("privacy-batch-col-spread"), "24%"),
            ],
            providers,
        ));
    }
    if single_only > 0 {
        builder = builder.add_component(note(
            i18n.t_args(
                "privacy-batch-providers-single",
                &[("count", single_only as i64)],
            ),
            "9.5pt",
        ));
    }

    builder = builder.add_component(table(
        i18n.t("privacy-batch-table-checks"),
        vec![
            (i18n.t("privacy-col-check"), "36%"),
            (i18n.t("privacy-batch-col-found"), "16%"),
            (i18n.t("privacy-batch-col-not_found"), "16%"),
            (i18n.t("privacy-batch-col-not_applicable"), "16%"),
            (i18n.t("privacy-batch-col-not_measured"), "16%"),
        ],
        check_rows(&agg, i18n),
    ));

    let mut headers = vec![("URL".to_string(), "34%")];
    headers.extend(
        PrivacyCheck::ALL
            .iter()
            .map(|&c| (check_short_label(i18n, c), "11%")),
    );
    builder = builder.add_component(table(
        i18n.t("privacy-batch-table-matrix"),
        headers,
        matrix_rows(&agg, i18n),
    ));
    builder = builder.add_component(note(i18n.t("privacy-batch-matrix-legend"), "9.5pt"));
    if agg.matrix.len() > MAX_MATRIX_ROWS {
        builder = builder.add_component(note(
            i18n.t_args(
                "privacy-batch-matrix-more",
                &[
                    ("shown", MAX_MATRIX_ROWS as i64),
                    ("total", agg.matrix.len() as i64),
                ],
            ),
            "9.5pt",
        ));
    }
    builder
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::privacy::fixtures;

    fn agg() -> PrivacyBatchAggregate {
        let tracked = fixtures::tracked();
        let clean = fixtures::clean();
        let failed = fixtures::not_measured();
        aggregate_privacy([
            ("https://a.example/", &tracked),
            ("https://a.example/news", &tracked),
            ("https://a.example/about", &clean),
            ("https://a.example/broken", &failed),
        ])
        .expect("aggregate")
    }

    #[test]
    fn kind_rows_count_pages_of_measured() {
        let rows = kind_rows(&agg(), &I18n::new("en").unwrap());
        // Tracking: 2 of the 3 measured pages (the failed page is outside).
        assert_eq!(rows[0][1], "2 of 3");
        assert_eq!(rows[0][2], "on several pages");
    }

    #[test]
    fn recurring_providers_exclude_single_page_ones() {
        let tracked = fixtures::tracked();
        let clean = fixtures::clean();
        let a = aggregate_privacy([
            ("https://a.example/", &tracked),
            ("https://a.example/b", &clean),
        ])
        .unwrap();
        let (rows, left_out) = provider_rows(&a, &I18n::new("en").unwrap());
        assert!(rows.is_empty());
        assert_eq!(left_out, 2);
        let (rows, _) = provider_rows(&agg(), &I18n::new("en").unwrap());
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0][0], "Google Analytics");
        assert_eq!(rows[0][4], "6");
    }

    #[test]
    fn matrix_cells_separate_zero_not_applicable_and_not_measured() {
        let en = I18n::new("en").unwrap();
        let rows = matrix_rows(&agg(), &en);
        let by = |suffix: &str| {
            rows.iter()
                .find(|r| r[0].ends_with(suffix))
                .unwrap()
                .clone()
        };
        // Clean page: storage not applicable, policy links nothing found.
        let clean = by("about");
        assert_eq!(clean[2], "–");
        assert_eq!(clean[6], "0");
        // Failed page: every cell is "not measured", never 0.
        let failed = by("broken");
        assert!(failed[1..].iter().all(|c| c == "n/m"), "{failed:?}");
        // Tracked page: one tracking finding.
        assert_eq!(by("news")[1], "1");
        let de = matrix_rows(&agg(), &I18n::new("de").unwrap());
        assert!(de.iter().any(|r| r.iter().any(|c| c == "n. g.")));
    }

    fn page(url: &str, privacy: crate::privacy::PrivacyAnalysis) -> crate::audit::AuditReport {
        let mut r = crate::audit::AuditReport::new(
            url.to_string(),
            crate::model::WcagLevel::AA,
            crate::wcag::WcagResults::new(),
            100,
        );
        r.experience.privacy = Some(privacy);
        r
    }

    /// Writes the `.typ` sources for review when `PRIVACY_TYP_DIR` is set.
    fn dump(name: &str, typ: &str) {
        if let Ok(dir) = std::env::var("PRIVACY_TYP_DIR") {
            std::fs::write(format!("{dir}/{name}.typ"), typ).expect("write typ");
        }
    }

    /// The batch report carries the aggregate and no per-page sections: a
    /// provider's per-page sentence and its evidence URL never appear.
    #[test]
    fn batch_typ_holds_the_aggregate_and_no_per_page_detail() {
        let batch = crate::audit::BatchReport::from_reports(
            vec![
                page("https://a.example/", fixtures::tracked()),
                page("https://a.example/news", fixtures::tracked()),
                page("https://a.example/about", fixtures::clean()),
                page("https://a.example/broken", fixtures::not_measured()),
            ],
            vec![],
            10,
        );
        for locale in ["en", "de"] {
            let config = crate::output::report_model::ReportConfig {
                locale: locale.to_string(),
                ..Default::default()
            };
            let typ = crate::output::pdf::generate_batch_typ(&batch, &config).expect("typ");
            dump(&format!("batch-{locale}"), &typ);
            let i18n = I18n::new(locale).unwrap();
            assert!(typ.contains(&i18n.t("privacy-batch-title")), "{locale}");
            assert!(
                typ.contains(&i18n.t("privacy-batch-table-matrix")),
                "{locale}"
            );
            // Per-page detail stays out.
            assert!(!typ.contains("google-analytics.com/g/collect"), "{locale}");
            assert!(!typ.contains("_ga_ABC123"), "{locale}");
            assert!(!typ.contains(&i18n.t("privacy-table-findings")), "{locale}");
        }
    }

    #[test]
    fn single_typ_has_the_evidence_tables_and_the_scope() {
        for (locale, analysis) in [
            ("en", fixtures::tracked()),
            ("de", fixtures::tracked()),
            ("en", fixtures::not_measured()),
            ("de", fixtures::partial()),
        ] {
            let measured = !matches!(
                analysis.measurement.state,
                crate::audit::collection::CollectionState::Failed
            );
            let tag = format!("single-{locale}-{}", if measured { "m" } else { "x" });
            let report = page("https://a.example/", analysis);
            let config = crate::output::report_model::ReportConfig {
                locale: locale.to_string(),
                ..Default::default()
            };
            let typ = crate::output::pdf::generate_typ(&report, &config).expect("typ");
            dump(&tag, &typ);
            let i18n = I18n::new(locale).unwrap();
            assert!(typ.contains(&i18n.t("privacy-title")), "{tag}");
            assert!(typ.contains(&i18n.t("privacy-table-checks")), "{tag}");
            let has_contacts = report
                .experience
                .privacy
                .as_ref()
                .is_some_and(|p| !p.contacts.is_empty());
            assert_eq!(
                typ.contains(&i18n.t("privacy-table-contacts")),
                has_contacts,
                "{tag}"
            );
        }
    }

    #[test]
    fn check_distribution_leaves_failed_page_out() {
        let rows = check_rows(&agg(), &I18n::new("en").unwrap());
        // Tracking: 2 found, 0 nothing found, 1 not applicable, 0 not measured.
        assert_eq!(rows[0][1..], ["2", "0", "1", "0"]);
    }
}
