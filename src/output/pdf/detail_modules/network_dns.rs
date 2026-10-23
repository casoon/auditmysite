use super::*;

/// Renders the DNS-configuration module (#545) — opt-in checks for CAA,
/// DNSSEC, and SPF/MX records. Deliberately has no `ScoreCard`, same as
/// `design_quality`: this module never contributes a score to the report.
pub(in crate::output::pdf) fn render_network_dns(
    mut builder: renderreport::engine::ReportBuilder,
    dns: &crate::network::dns::NetworkDnsAnalysis,
    is_first: bool,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = is_english(i18n);
    let title = pick(i18n, "DNS-Konfiguration", "DNS Configuration");
    let is_clean = dns.findings.is_empty();
    let takeaway = if is_clean {
        pick(
            i18n,
            "Keine Hinweise zur DNS-Konfiguration gefunden.",
            "No DNS-configuration findings.",
        )
        .to_string()
    } else if en {
        format!(
            "{} DNS-configuration finding(s) for {}.",
            dns.findings.len(),
            dns.host
        )
    } else {
        format!(
            "{} Hinweis(e) zur DNS-Konfiguration für {}.",
            dns.findings.len(),
            dns.host
        )
    };
    builder = super::module_chapter_opener(builder, title, &takeaway, is_first);

    builder = builder.add_component(
        Label::new(pick(
            i18n,
            "Direkte DNS-Abfragen (CAA, DNSSEC, SPF/MX) — kein Bestandteil des \
             Barrierefreiheitswerts oder des kombinierten technischen Werts, keine rechtliche Bewertung.",
            "Direct DNS queries (CAA, DNSSEC, SPF/MX) — not part of the accessibility \
             or overall score, not a legal assessment.",
        ))
        .with_size("10.5pt")
        .with_color(crate::output::pdf::design::tokens::NEUTRAL),
    );

    let ok = pick(i18n, "Vorhanden", "Present").to_string();
    let missing = pick(i18n, "Fehlt", "Missing").to_string();

    let mut table = AuditTable::new(vec![
        TableColumn::new(if en { "Check" } else { "Prüfung" }).with_width("20%"),
        TableColumn::new("Status").with_width("15%"),
        TableColumn::new(if en { "Finding" } else { "Befund" }).with_width("65%"),
    ])
    .with_title(if en { "DNS Records" } else { "DNS-Einträge" });

    table = table.add_row(vec![
        "CAA".to_string(),
        if dns.caa_present {
            ok.clone()
        } else {
            missing.clone()
        },
        if dns.caa_present {
            String::new()
        } else {
            crate::network::dns::finding_message_text("dns.caa_missing", en)
        },
    ]);

    table = table.add_row(vec![
        "DNSSEC".to_string(),
        if dns.dnssec_detected {
            ok.clone()
        } else {
            missing.clone()
        },
        if dns.dnssec_detected {
            String::new()
        } else {
            crate::network::dns::finding_message_text("dns.dnssec_not_detected", en)
        },
    ]);

    if let Some(spf_present) = dns.spf_present {
        table = table.add_row(vec![
            "SPF".to_string(),
            if spf_present { ok } else { missing },
            if spf_present {
                String::new()
            } else {
                crate::network::dns::finding_message_text("dns.spf_missing", en)
            },
        ]);
    }

    builder = builder.add_component(table);

    if is_clean {
        builder = builder.add_component(clean_section_note(i18n));
    }

    builder
}
