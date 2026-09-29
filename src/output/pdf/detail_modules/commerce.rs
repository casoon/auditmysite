use super::*;

/// Commerce/shop section (#8, 2026-09-01 product decision): only ever built
/// (see `build_commerce_details`) when there's substantive content — real
/// product-completeness data or at least one missing mandatory/trust-page
/// link — so a shop page with nothing to add stays silent instead of
/// padding the report with a near-empty chapter.
pub(in crate::output::pdf) fn render_commerce(
    mut builder: renderreport::engine::ReportBuilder,
    c: &CommercePresentation,
    is_first: bool,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = is_english(i18n);
    let title = i18n.t("section-commerce");
    let takeaway = if let Some(ref p) = c.product {
        if en {
            format!(
                "Product structured data is {}% complete on this page.",
                p.score
            )
        } else {
            format!(
                "Die Produkt-Strukturdaten sind auf dieser Seite zu {}% vollständig.",
                p.score
            )
        }
    } else {
        let missing = c.trust_pages.iter().filter(|(_, linked)| !linked).count();
        if en {
            format!("{missing} mandatory/trust-page link(s) not found on this page.")
        } else {
            format!("{missing} Pflichtseiten-Link(s) auf dieser Seite nicht gefunden.")
        }
    };
    builder = super::module_chapter_opener(builder, &title, &takeaway, is_first);

    // Explicit scope disclaimer: this is a single-page, derive-only signal,
    // not a checkout/payment/cart audit — this tool has no cross-page
    // session state (see crate::commerce::CommercePageKind's doc comment).
    // Stated up front so it can't be misread as broader shop coverage
    // (report-quality review, 2026-09-01).
    builder = builder.add_component(
        Label::new(i18n.t("pdf-commerce-scope-note"))
            .with_size("10.5pt")
            .with_color(crate::output::pdf::design::tokens::NEUTRAL),
    );

    let mut overview = KeyValueList::new().with_title(i18n.t("pdf-commerce-overview-title"));
    overview = overview.add(i18n.t("pdf-commerce-page-kind"), &c.page_kind_label);
    builder = builder.add_component(overview);

    if let Some(ref p) = c.product {
        builder = builder.add_component(
            MetricStrip::new(vec![MetricStripItem::new(
                i18n.t("pdf-commerce-completeness"),
                format!("{}/100", p.score),
            )
            .with_accent(crate::output::pdf::design::score_color(p.score as u8))])
            .compact(),
        );

        let mut product_kv = KeyValueList::new().with_title(i18n.t("pdf-commerce-product-title"));
        product_kv = product_kv.add(
            i18n.t("pdf-commerce-price"),
            p.price
                .clone()
                .unwrap_or_else(|| i18n.t("pdf-commerce-not-exposed")),
        );
        product_kv = product_kv.add(
            i18n.t("pdf-commerce-availability"),
            p.availability
                .clone()
                .unwrap_or_else(|| i18n.t("pdf-commerce-not-exposed")),
        );
        product_kv = product_kv.add(
            i18n.t("pdf-commerce-shipping"),
            if p.has_shipping_details {
                i18n.t("pdf-dm-yes")
            } else {
                i18n.t("pdf-commerce-not-exposed")
            },
        );
        product_kv = product_kv.add(
            i18n.t("pdf-commerce-returns"),
            if p.has_return_policy {
                i18n.t("pdf-dm-yes")
            } else {
                i18n.t("pdf-commerce-not-exposed")
            },
        );
        product_kv = product_kv.add(
            i18n.t("pdf-commerce-rating"),
            p.rating
                .clone()
                .unwrap_or_else(|| i18n.t("pdf-commerce-not-exposed")),
        );
        builder = builder.add_component(product_kv);
    }

    let trust_rows: Vec<ChecklistRow> = c
        .trust_pages
        .iter()
        .map(|(label, linked)| {
            let status_text = if *linked {
                i18n.t("pdf-commerce-linked")
            } else {
                i18n.t("pdf-commerce-not-linked")
            };
            ChecklistRow::new(label.clone(), status_text).with_status(if *linked {
                "good"
            } else {
                "warn"
            })
        })
        .collect();
    builder = builder.add_component(
        ChecklistPanel::new(trust_rows).with_title(i18n.t("pdf-commerce-trust-pages-title")),
    );

    if c.findings.is_empty() {
        builder = builder.add_component(clean_section_note(i18n));
    } else {
        let mut table = AuditTable::new(vec![
            TableColumn::new(i18n.t("label-severity")),
            TableColumn::new(i18n.t("label-issue")),
        ])
        .with_title(&title);
        for (severity, message) in &c.findings {
            table = table.add_row(vec![severity.as_str(), message.as_str()]);
        }
        builder = builder.add_component(table);
    }

    builder
}
