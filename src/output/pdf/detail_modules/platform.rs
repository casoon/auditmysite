use super::*;

pub(in crate::output::pdf) fn render_security(
    mut builder: renderreport::engine::ReportBuilder,
    sec: &SecurityPresentation,
    is_first: bool,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let security_title = i18n.t("section-security");

    builder = super::module_chapter_opener(builder, &security_title, is_first);

    builder = builder
        .add_component(
            ScoreCard::new(super::module_score_caption(i18n), sec.score)
                .with_description(super::score_band_label(sec.score, i18n))
                .with_thresholds(75, 40),
        )
        .add_component(
            Label::new(sec.interpretation.as_str())
                .with_size("10.5pt")
                .with_color(crate::output::pdf::design::tokens::NEUTRAL),
        )
        .add_component(module_customer_context(
            i18n,
            "security",
            sec.score,
            &sec.interpretation,
        ));

    let header_count = sec
        .headers
        .iter()
        .filter(|(_, status, _)| status.to_lowercase().contains("vorhanden") || status == "✓")
        .count();
    builder = builder.add_component(
        MetricStrip::new(vec![
            MetricStripItem::new("Header", format!("{}/9", header_count)).with_accent("#0f766e"),
            MetricStripItem::new(
                "HTTPS",
                if sec
                    .ssl_info
                    .iter()
                    .any(|(k, v)| k.contains("HTTPS") && v == "Ja")
                {
                    i18n.t("pdf-sec-https-yes")
                } else {
                    i18n.t("pdf-sec-https-unclear")
                },
            )
            .with_accent("#2563eb"),
            MetricStripItem::new("Issues", sec.issues.len().to_string()).with_accent("#dc2626"),
        ])
        .compact(),
    );

    if !sec.headers.is_empty() {
        let mut table = AuditTable::new(vec![
            TableColumn::new("Header"),
            TableColumn::new("Status"),
            TableColumn::new(i18n.t("pdf-seo-value")),
        ])
        .with_title("Security Headers");
        for (name, status, val) in &sec.headers {
            table = table.add_row(vec![name.as_str(), status.as_str(), val.as_str()]);
        }
        builder = builder.add_component(table);
    }

    if !sec.ssl_info.is_empty() {
        let mut kv = KeyValueList::new().with_title("SSL/TLS");
        for (k, v) in &sec.ssl_info {
            kv = kv.add(k, v);
        }
        builder = builder.add_component(kv);
    }

    if !sec.protection.is_empty() {
        let title = i18n.t("pdf-sec-protection-title");
        let mut kv = KeyValueList::new().with_title(&title);
        for (name, kind) in &sec.protection {
            kv = kv.add(name, kind);
        }
        builder = builder.add_component(kv);
    }

    // A strong security score means no significant findings — state it, so a
    // clean section reads differently from an unchecked one. Mirrors the
    // score>=80 confirmation the Source Quality / AI Visibility modules use, and
    // fires for real-but-good sites (which always carry a few minor header
    // notes and so never hit "zero findings") (#446 re-scope).
    if sec.score >= 80 {
        builder = builder.add_component(clean_section_note(i18n));
    }
    for (title, sev, msg) in &sec.issues {
        builder = builder.add_component(Finding::new(title, map_severity(sev), msg));
    }

    if !sec.recommendations.is_empty() {
        let mut rec_list = List::new().with_title(i18n.t("label-improvement-suggestions"));
        for rec in &sec.recommendations {
            rec_list = rec_list.add_item(rec);
        }
        builder = builder.add_component(rec_list);
    }
    builder
}

pub(in crate::output::pdf) fn render_mobile(
    mut builder: renderreport::engine::ReportBuilder,
    mobile: &MobilePresentation,
    is_first: bool,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let mobile_title = i18n.t("section-mobile-usability");
    builder = super::module_chapter_opener(builder, &mobile_title, is_first);
    builder = builder
        .add_component(
            ScoreCard::new(super::module_score_caption(i18n), mobile.score)
                .with_description(super::score_band_label(mobile.score, i18n))
                .with_thresholds(75, 40),
        )
        .add_component(
            Label::new(mobile.interpretation.as_str())
                .with_size("10.5pt")
                .with_color(crate::output::pdf::design::tokens::NEUTRAL),
        )
        .add_component(module_customer_context(
            i18n,
            "mobile",
            mobile.score,
            &mobile.interpretation,
        ));

    let configured_label = i18n.t("mobile-configured");
    let viewport_status = mobile
        .viewport
        .iter()
        .find(|(k, _)| k.to_lowercase().contains("viewport"))
        .map(|(_, v)| v.as_str())
        .unwrap_or(&configured_label);
    let touch_targets = mobile
        .touch_targets
        .iter()
        .find(|(k, _)| k.to_lowercase().contains("zu klein") || k.to_lowercase().contains("small"))
        .map(|(_, v)| v.as_str())
        .unwrap_or("n/a");
    builder = builder.add_component(
        MetricStrip::new(vec![
            MetricStripItem::new("Viewport", viewport_status).with_accent("#0f766e"),
            MetricStripItem::new(i18n.t("mobile-touch-targets"), touch_targets)
                .with_accent("#d97706"),
            MetricStripItem::new("Issues", mobile.issues.len().to_string()).with_accent("#dc2626"),
        ])
        .compact(),
    );

    if !mobile.viewport.is_empty() {
        let mut kv = KeyValueList::new().with_title(i18n.t("mobile-viewport-config"));
        for (k, v) in &mobile.viewport {
            kv = kv.add(k, v);
        }
        builder = builder.add_component(kv);
    }
    if !mobile.touch_targets.is_empty() {
        let mut kv = KeyValueList::new().with_title(i18n.t("mobile-touch-targets"));
        for (k, v) in &mobile.touch_targets {
            kv = kv.add(k, v);
        }
        builder = builder.add_component(kv);
    }
    if !mobile.font_analysis.is_empty() {
        let mut kv = KeyValueList::new().with_title(i18n.t("mobile-font-analysis"));
        for (k, v) in &mobile.font_analysis {
            kv = kv.add(k, v);
        }
        builder = builder.add_component(kv);
    }
    if !mobile.content_sizing.is_empty() {
        let mut kv = KeyValueList::new().with_title(i18n.t("mobile-content-sizing"));
        for (k, v) in &mobile.content_sizing {
            kv = kv.add(k, v);
        }
        builder = builder.add_component(kv);
    }

    for (cat, sev, msg) in &mobile.issues {
        let label = mobile_category_label(cat, i18n);
        builder = builder.add_component(Finding::new(&label, map_severity(sev), msg));
    }
    builder
}

/// Maps a raw mobile issue category (e.g. "touch_targets") to a localized,
/// human-readable label. Unknown categories fall back to a title-cased form of
/// the identifier so a snake_case key never reaches the report verbatim (#358).
fn mobile_category_label(category: &str, i18n: &I18n) -> String {
    let key = format!("mobile-cat-{category}");
    let translated = i18n.t(&key);
    if translated != key {
        return translated;
    }
    category
        .split(['_', '-'])
        .filter(|w| !w.is_empty())
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
