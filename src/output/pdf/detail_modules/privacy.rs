use super::*;

use crate::privacy::{privacy_finding_text, Confidence};

fn category_label(code: &str, en: bool) -> &'static str {
    match (code, en) {
        ("analytics", true) => "Analytics",
        ("analytics", false) => "Analyse",
        ("advertising", true) => "Advertising",
        ("advertising", false) => "Werbung",
        ("social", _) => "Social",
        ("tag_manager", true) => "Tag manager",
        ("tag_manager", false) => "Tag-Manager",
        ("fonts", true) => "Fonts",
        ("fonts", false) => "Schriften",
        ("cdn", _) => "CDN",
        ("video_embed", true) => "Video embed",
        ("video_embed", false) => "Video-Einbettung",
        ("maps", true) => "Maps",
        ("maps", false) => "Karten",
        ("consent_platform", true) => "Consent platform",
        ("consent_platform", false) => "Einwilligungsplattform",
        (_, true) => "Other",
        (_, false) => "Sonstige",
    }
}

fn jurisdiction_label(code: &str, en: bool) -> &'static str {
    match (code, en) {
        ("eu", _) => "EU",
        ("us", true) => "USA",
        ("us", false) => "USA",
        ("other", true) => "Outside EU",
        ("other", false) => "Außerhalb EU",
        (_, true) => "Not stated",
        (_, false) => "Nicht belegt",
    }
}

fn storage_kind_label(code: &str, en: bool) -> &'static str {
    match (code, en) {
        ("cookie", _) => "Cookie",
        ("local_storage", _) => "localStorage",
        ("session_storage", _) => "sessionStorage",
        (_, true) => "Storage",
        (_, false) => "Speicher",
    }
}

/// Renders the first-visit privacy indicators (#872): what a first visit
/// sent and stored before any interaction. A technical observation — no
/// score, no legal assessment, and the scope (clean profile, no interaction)
/// is stated first so the section never claims more than was measured.
pub(in crate::output::pdf) fn render_privacy(
    mut builder: renderreport::engine::ReportBuilder,
    privacy: &crate::privacy::PrivacyAnalysis,
    is_first: bool,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = is_english(i18n);
    let title = pick(i18n, "Datenschutz-Indikatoren", "Privacy Indicators");
    let takeaway = if privacy.findings.is_empty() {
        pick(
            i18n,
            "Beim ersten Besuch wurden keine bekannten Drittanbieter kontaktiert oder Kennungen gesetzt.",
            "The first visit contacted no known third-party provider and set no known identifier.",
        )
        .to_string()
    } else if en {
        format!(
            "{} technical indicator(s) on first visit, before any interaction.",
            privacy.findings.len()
        )
    } else {
        format!(
            "{} technische(r) Indikator(en) beim ersten Besuch, vor jeder Interaktion.",
            privacy.findings.len()
        )
    };
    builder = super::module_chapter_opener(builder, title, &takeaway, is_first);

    builder = builder.add_component(
        Label::new(pick(
            i18n,
            "Technische Beobachtung beim ersten Besuch in einem frischen Browserprofil, ohne \
             Klick und ohne Einwilligung — kein Bestandteil eines Werts und keine rechtliche \
             Bewertung. Nicht sichtbar sind serverseitige Verarbeitung und Anbieter, die die \
             Liste nicht kennt.",
            "Technical observation of a first visit in a clean browser profile, with no click \
             and no consent given — not part of any score and not a legal assessment. Server-side \
             processing and providers the list does not know are not visible.",
        ))
        .with_size("10.5pt")
        .with_color(crate::output::pdf::design::tokens::NEUTRAL),
    );

    if privacy.partial {
        builder = builder.add_component(
            Label::new(pick(
                i18n,
                "Die Erfassung ist unvollständig (z. B. Netzwerk nicht zur Ruhe gekommen); \
                 fehlende Einträge sind nicht als \"nichts gesetzt\" zu lesen.",
                "The capture is incomplete (e.g. the network never went quiet); missing \
                 entries are not to be read as \"nothing set\".",
            ))
            .with_size("10.5pt")
            .with_color(crate::output::pdf::design::tokens::NEUTRAL),
        );
    }

    if !privacy.findings.is_empty() {
        let mut table = AuditTable::new(vec![
            TableColumn::new(if en { "Indicator" } else { "Indikator" }).with_width("80%"),
            TableColumn::new(if en { "Basis" } else { "Grundlage" }).with_width("20%"),
        ])
        .with_title(if en { "Indicators" } else { "Indikatoren" });
        for f in &privacy.findings {
            let basis = match (f.confidence, en) {
                (Confidence::Evidence, true) => "Observed",
                (Confidence::Evidence, false) => "Beobachtet",
                (Confidence::Indication, true) => "Indication",
                (Confidence::Indication, false) => "Hinweis",
            };
            table = table.add_row(vec![
                privacy_finding_text(f.kind, &f.subjects, en),
                basis.to_string(),
            ]);
        }
        builder = builder.add_component(table);
    }

    if !privacy.contacts.is_empty() {
        let mut table = AuditTable::new(vec![
            TableColumn::new(if en { "Provider" } else { "Anbieter" }).with_width("26%"),
            TableColumn::new(if en { "Purpose" } else { "Zweck" }).with_width("18%"),
            TableColumn::new(if en { "Seat" } else { "Sitz" }).with_width("14%"),
            TableColumn::new("Hosts").with_width("32%"),
            TableColumn::new(if en { "Requests" } else { "Anfragen" }).with_width("10%"),
        ])
        .with_title(if en {
            "Third parties contacted"
        } else {
            "Kontaktierte Drittanbieter"
        });
        for c in &privacy.contacts {
            table = table.add_row(vec![
                c.provider.clone(),
                category_label(&c.category, en).to_string(),
                jurisdiction_label(&c.jurisdiction, en).to_string(),
                c.hosts.join(", "),
                c.requests.to_string(),
            ]);
        }
        builder = builder.add_component(table);
    }

    if !privacy.stored.is_empty() {
        let mut table = AuditTable::new(vec![
            TableColumn::new(if en { "Provider" } else { "Anbieter" }).with_width("34%"),
            TableColumn::new(if en { "Kind" } else { "Art" }).with_width("22%"),
            TableColumn::new("Name").with_width("44%"),
        ])
        .with_title(if en {
            "Identifiers stored (names only)"
        } else {
            "Gespeicherte Kennungen (nur Namen)"
        });
        for s in &privacy.stored {
            table = table.add_row(vec![
                s.provider.clone(),
                storage_kind_label(&s.kind, en).to_string(),
                s.name.clone(),
            ]);
        }
        builder = builder.add_component(table);
    }

    builder = builder.add_component(
        Label::new(if en {
            format!("Provider list version {}.", privacy.ruleset_version)
        } else {
            format!("Stand der Anbieterliste: {}.", privacy.ruleset_version)
        })
        .with_size("9pt")
        .with_color(crate::output::pdf::design::tokens::NEUTRAL),
    );

    if privacy.findings.is_empty() {
        builder = builder.add_component(clean_section_note(i18n));
    }

    builder
}
