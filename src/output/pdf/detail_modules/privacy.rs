use super::*;

use crate::audit::collection::CollectionState;
use crate::privacy::aggregate::Spread;
use crate::privacy::{
    finding_text, CheckOutcome, ConfidenceClass, Evidence, EvidenceSource, PolicyDocument,
    PrivacyAnalysis, PrivacyCheck, PrivacyFinding, PrivacyFindingKind,
};

/// Contacts shown before the table says "+N more".
const MAX_CONTACT_ROWS: usize = 12;
/// Evidence entries per finding row.
const MAX_EVIDENCE: usize = 4;
const MAX_EVIDENCE_CHARS: usize = 58;
const MAX_HOSTS_CHARS: usize = 44;

// ── Labels (localized at render time, #406) ────────────────────────────

/// `prefix-code` from the locale file; an unknown code shows as itself.
fn coded(i18n: &I18n, prefix: &str, code: &str) -> String {
    let key = format!("{prefix}-{code}");
    let text = i18n.t(&key);
    if text == key {
        code.to_string()
    } else {
        text
    }
}

fn check_id(check: PrivacyCheck) -> &'static str {
    match check {
        PrivacyCheck::TrackingRequests => "tracking_requests",
        PrivacyCheck::NonEssentialStorage => "non_essential_storage",
        PrivacyCheck::VisitorIpTransfer => "visitor_ip_transfer",
        PrivacyCheck::ThirdCountryProvider => "third_country_provider",
        PrivacyCheck::ConsentPlatform => "consent_platform",
        PrivacyCheck::PolicyLinks => "policy_links",
    }
}

fn outcome_id(outcome: CheckOutcome) -> &'static str {
    match outcome {
        CheckOutcome::Found => "found",
        CheckOutcome::NotFound => "not_found",
        CheckOutcome::NotApplicable => "not_applicable",
        CheckOutcome::NotMeasured => "not_measured",
    }
}

fn confidence_id(c: ConfidenceClass) -> &'static str {
    match c {
        ConfidenceClass::Evidence => "evidence",
        ConfidenceClass::Indication => "indication",
        ConfidenceClass::ManualCheck => "manual-check",
    }
}

fn source_id(s: EvidenceSource) -> &'static str {
    match s {
        EvidenceSource::Request => "request",
        EvidenceSource::Cookie => "cookie",
        EvidenceSource::StorageKey => "storage_key",
        EvidenceSource::Link => "link",
        EvidenceSource::Signal => "signal",
        EvidenceSource::Provider => "provider",
    }
}

pub(in crate::output::pdf) fn category_label(i18n: &I18n, category: &str) -> String {
    coded(i18n, "privacy-category", category)
}

pub(in crate::output::pdf) fn seat_label(i18n: &I18n, jurisdiction: &str) -> String {
    coded(i18n, "privacy-seat", jurisdiction)
}

pub(in crate::output::pdf) fn check_short_label(i18n: &I18n, check: PrivacyCheck) -> String {
    coded(i18n, "privacy-check-short", check_id(check))
}

pub(in crate::output::pdf) fn spread_label(i18n: &I18n, spread: Spread) -> String {
    i18n.t(match spread {
        Spread::AllPages => "privacy-batch-spread-all",
        Spread::SeveralPages => "privacy-batch-spread-several",
        Spread::SinglePage => "privacy-batch-spread-single",
    })
}

/// Provider-independent name of an indicator. The provider kinds have no
/// sentence without a provider, so they use a locale-file label; the page
/// level kinds use the one text source, `finding_text`.
pub(in crate::output::pdf) fn kind_label(
    i18n: &I18n,
    kind: PrivacyFindingKind,
    document: Option<PolicyDocument>,
) -> String {
    use PrivacyFindingKind as K;
    let id = match kind {
        K::TrackingRequestBeforeConsent => "tracking_request_before_consent",
        K::NonEssentialStorageBeforeConsent => "non_essential_storage_before_consent",
        K::VisitorIpTransfer => "visitor_ip_transfer",
        K::ThirdCountryProvider
        | K::NoConsentPlatformDetected
        | K::PolicyLinkMissing
        | K::PolicyLinkUnusable => {
            return finding_text(kind, "", document, is_english(i18n));
        }
    };
    coded(i18n, "privacy-kind", id)
}

// ── Rows (pure; asserted by tests) ─────────────────────────────────────

fn is_measured(a: &PrivacyAnalysis) -> bool {
    matches!(
        a.measurement.state,
        CollectionState::Measured | CollectionState::Empty | CollectionState::Partial
    )
}

fn unmeasured_checks(a: &PrivacyAnalysis) -> usize {
    a.checks
        .iter()
        .filter(|c| c.outcome == CheckOutcome::NotMeasured)
        .count()
}

fn check_rows(a: &PrivacyAnalysis, i18n: &I18n) -> Vec<Vec<String>> {
    a.checks
        .iter()
        .map(|c| {
            vec![
                coded(i18n, "privacy-check", check_id(c.check)),
                coded(i18n, "privacy-outcome", outcome_id(c.outcome)),
                c.reason
                    .as_deref()
                    .map(|r| coded(i18n, "privacy-reason", r))
                    .unwrap_or_default(),
            ]
        })
        .collect()
}

fn contact_rows(a: &PrivacyAnalysis, i18n: &I18n) -> Vec<Vec<String>> {
    let mut contacts: Vec<_> = a.contacts.iter().collect();
    contacts.sort_by(|x, y| {
        y.request_count
            .cmp(&x.request_count)
            .then(x.provider.id.cmp(&y.provider.id))
    });
    contacts
        .into_iter()
        .map(|c| {
            vec![
                c.provider.name.clone(),
                category_label(i18n, &c.provider.category),
                seat_label(i18n, &c.provider.jurisdiction),
                c.request_count.to_string(),
                truncate_with_ellipsis(&c.hosts.join(", "), MAX_HOSTS_CHARS),
            ]
        })
        .collect()
}

fn evidence_text(evidence: &[Evidence], i18n: &I18n) -> String {
    let mut parts: Vec<String> = evidence
        .iter()
        .take(MAX_EVIDENCE)
        .map(|e| {
            format!(
                "{}: {}",
                coded(i18n, "privacy-source", source_id(e.source)),
                truncate_with_ellipsis(&e.value, MAX_EVIDENCE_CHARS)
            )
        })
        .collect();
    if evidence.len() > MAX_EVIDENCE {
        parts.push(i18n.t_args(
            "privacy-more",
            &[("count", (evidence.len() - MAX_EVIDENCE) as i64)],
        ));
    }
    parts.join("\n")
}

fn finding_rows(findings: &[PrivacyFinding], i18n: &I18n) -> Vec<Vec<String>> {
    let en = is_english(i18n);
    findings
        .iter()
        .map(|f| {
            let provider = f.provider.as_ref().map(|p| p.name.as_str()).unwrap_or("");
            vec![
                finding_text(f.kind, provider, f.document, en),
                coded(i18n, "privacy-confidence", confidence_id(f.confidence)),
                evidence_text(&f.evidence, i18n),
            ]
        })
        .collect()
}

fn takeaway(a: &PrivacyAnalysis, i18n: &I18n) -> String {
    if !is_measured(a) {
        return i18n.t("privacy-takeaway-not-measured");
    }
    let mut text = if a.findings.is_empty() {
        i18n.t("privacy-takeaway-none")
    } else {
        i18n.t_args(
            "privacy-takeaway",
            &[
                ("providers", a.contacts.len() as i64),
                ("findings", a.findings.len() as i64),
            ],
        )
    };
    let unmeasured = unmeasured_checks(a);
    if unmeasured > 0 {
        text.push(' ');
        text.push_str(&i18n.t_args(
            "privacy-takeaway-unmeasured",
            &[("count", unmeasured as i64)],
        ));
    }
    text
}

fn measurement_note(a: &PrivacyAnalysis, i18n: &I18n) -> Option<String> {
    match a.measurement.state {
        CollectionState::Partial => {
            let gaps = a
                .measurement
                .gaps
                .iter()
                .map(|g| coded(i18n, "privacy-gap", g))
                .collect::<Vec<_>>()
                .join(", ");
            Some(i18n.t_args("privacy-measurement-partial", &[("gaps", gaps)]))
        }
        CollectionState::Failed | CollectionState::Unsupported | CollectionState::NotApplicable => {
            let reason = a
                .measurement
                .reason
                .as_deref()
                .map(|r| coded(i18n, "privacy-reason", r))
                .unwrap_or_default();
            Some(i18n.t_args("privacy-measurement-failed", &[("reason", reason)]))
        }
        _ => None,
    }
}

fn plain_table(title: String, headers: Vec<(String, &str)>, rows: Vec<Vec<String>>) -> AuditTable {
    let mut table = AuditTable::new(
        headers
            .into_iter()
            .map(|(h, w)| TableColumn::new(h).with_width(w))
            .collect(),
    )
    .with_title(title);
    for row in rows {
        table = table.add_row(row);
    }
    table
}

/// Single-page chapter (#872): what a first visit leaves behind, with the
/// scope stated and every unmeasured part shown as such. Score-neutral:
/// no score card, no verdict.
pub(in crate::output::pdf) fn render_privacy(
    mut builder: renderreport::engine::ReportBuilder,
    a: &PrivacyAnalysis,
    is_first: bool,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    builder = super::module_chapter_opener(
        builder,
        &i18n.t("privacy-title"),
        &takeaway(a, i18n),
        is_first,
    );
    builder = builder.add_component(
        Label::new(i18n.t("privacy-scope"))
            .with_size("10.5pt")
            .with_color(crate::output::pdf::design::tokens::NEUTRAL),
    );
    if let Some(note) = measurement_note(a, i18n) {
        builder = builder.add_component(
            Label::new(note)
                .with_size("10.5pt")
                .with_color(crate::output::pdf::design::tokens::WARN_DEEP),
        );
    }

    if is_measured(a) {
        builder = builder.add_component(
            MetricStrip::new(vec![
                MetricStripItem::new(
                    i18n.t("privacy-metric-contacts"),
                    a.contacts.len().to_string(),
                )
                .with_accent(crate::output::pdf::design::tokens::NEUTRAL),
                MetricStripItem::new(
                    i18n.t("privacy-metric-findings"),
                    a.findings.len().to_string(),
                )
                .with_accent(crate::output::pdf::design::tokens::NEUTRAL),
                MetricStripItem::new(
                    i18n.t("privacy-metric-unmeasured"),
                    unmeasured_checks(a).to_string(),
                )
                .with_accent(crate::output::pdf::design::tokens::NEUTRAL),
            ])
            .compact(),
        );
    }

    builder = builder.add_component(plain_table(
        i18n.t("privacy-table-checks"),
        vec![
            (i18n.t("privacy-col-check"), "46%"),
            (i18n.t("privacy-col-result"), "20%"),
            (i18n.t("privacy-col-note"), "34%"),
        ],
        check_rows(a, i18n),
    ));

    if !is_measured(a) {
        return builder;
    }

    if a.contacts.is_empty() {
        builder = builder.add_component(
            Label::new(i18n.t("privacy-no-contacts"))
                .with_size("10.5pt")
                .with_color(crate::output::pdf::design::tokens::NEUTRAL),
        );
    } else {
        let mut rows = contact_rows(a, i18n);
        let hidden = rows.len().saturating_sub(MAX_CONTACT_ROWS);
        rows.truncate(MAX_CONTACT_ROWS);
        builder = builder.add_component(plain_table(
            i18n.t("privacy-table-contacts"),
            vec![
                (i18n.t("privacy-col-provider"), "26%"),
                (i18n.t("privacy-col-category"), "18%"),
                (i18n.t("privacy-col-seat"), "16%"),
                (i18n.t("privacy-col-requests"), "10%"),
                (i18n.t("privacy-col-hosts"), "30%"),
            ],
            rows,
        ));
        if hidden > 0 {
            builder = builder.add_component(
                Label::new(i18n.t_args("privacy-more", &[("count", hidden as i64)]))
                    .with_size("9.5pt")
                    .with_color(crate::output::pdf::design::tokens::NEUTRAL),
            );
        }
    }

    if !a.findings.is_empty() {
        builder = builder.add_component(plain_table(
            i18n.t("privacy-table-findings"),
            vec![
                (i18n.t("privacy-col-indicator"), "42%"),
                (i18n.t("privacy-col-confidence"), "14%"),
                (i18n.t("privacy-col-evidence"), "44%"),
            ],
            finding_rows(&a.findings, i18n),
        ));
    }

    if !a.unclassified_hosts.is_empty() {
        let shown: Vec<&str> = a
            .unclassified_hosts
            .iter()
            .take(6)
            .map(String::as_str)
            .collect();
        let mut hosts = shown.join(", ");
        if a.unclassified_hosts.len() > shown.len() {
            hosts.push_str(", ");
            hosts.push_str(&i18n.t_args(
                "privacy-more",
                &[("count", (a.unclassified_hosts.len() - shown.len()) as i64)],
            ));
        }
        builder = builder.add_component(
            Label::new(i18n.t_args("privacy-unclassified", &[("hosts", hosts)]))
                .with_size("9.5pt")
                .with_color(crate::output::pdf::design::tokens::NEUTRAL),
        );
    }
    builder.add_component(
        Label::new(i18n.t_args("privacy-ruleset", &[("version", a.ruleset_version.clone())]))
            .with_size("9pt")
            .with_color(crate::output::pdf::design::tokens::NEUTRAL),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::privacy::fixtures;

    fn i18n(locale: &str) -> I18n {
        I18n::new(locale).expect("locale")
    }

    #[test]
    fn contact_rows_carry_provider_category_seat_and_request_count() {
        let rows = contact_rows(&fixtures::tracked(), &i18n("en"));
        // Most requests first.
        assert_eq!(rows[0][0], "Google Analytics");
        assert_eq!(rows[0][1], "Analytics");
        assert_eq!(rows[0][2], "USA");
        assert_eq!(rows[0][3], "3");
        assert_eq!(rows[1][0], "Google Fonts");
        assert_eq!(rows[1][3], "2");
    }

    #[test]
    fn finding_rows_show_confidence_class_and_cookie_names() {
        let a = fixtures::tracked();
        let rows = finding_rows(&a.findings, &i18n("en"));
        assert_eq!(rows.len(), a.findings.len());
        assert_eq!(rows[0][1], "Evidence");
        assert!(rows[1][2].contains("Cookie: _ga"), "{:?}", rows[1]);
        assert_eq!(rows[3][1], "Indication");
        // Message text comes from the single text source.
        assert_eq!(
            rows[0][0],
            finding_text(
                PrivacyFindingKind::TrackingRequestBeforeConsent,
                "Google Analytics",
                None,
                true
            )
        );
    }

    #[test]
    fn evidence_is_capped_with_a_remainder_count() {
        let evidence: Vec<Evidence> = (0..6)
            .map(|i| Evidence {
                source: EvidenceSource::Cookie,
                value: format!("c{i}"),
            })
            .collect();
        let text = evidence_text(&evidence, &i18n("en"));
        assert_eq!(text.lines().count(), MAX_EVIDENCE + 1);
        assert!(text.ends_with("+2 more"), "{text}");
    }

    #[test]
    fn unmeasured_checks_read_not_measured_never_nothing_found() {
        let rows = check_rows(&fixtures::partial(), &i18n("en"));
        assert_eq!(rows[0][1], "Not measured");
        assert_eq!(rows[1][1], "Nothing found");
        let rows = check_rows(&fixtures::not_measured(), &i18n("en"));
        assert!(rows.iter().all(|r| r[1] == "Not measured"));
    }

    #[test]
    fn takeaway_never_reads_clean_when_not_measured() {
        let en = i18n("en");
        assert_eq!(
            takeaway(&fixtures::not_measured(), &en),
            en.t("privacy-takeaway-not-measured")
        );
        let partial = takeaway(&fixtures::partial(), &en);
        assert!(partial.contains("Checks not measured: 4"), "{partial}");
    }

    #[test]
    fn takeaway_counts_providers_and_findings() {
        let t = takeaway(&fixtures::tracked(), &i18n("en"));
        assert!(t.contains("Providers contacted: 2"), "{t}");
        assert!(t.contains("technical indicators: 6"), "{t}");
    }

    #[test]
    fn failed_capture_states_the_reason() {
        let note = measurement_note(&fixtures::not_measured(), &i18n("en")).expect("note");
        assert!(note.contains("browser command failed"), "{note}");
        assert!(measurement_note(&fixtures::clean(), &i18n("en")).is_none());
    }

    #[test]
    fn german_labels_are_localized() {
        let rows = contact_rows(&fixtures::tracked(), &i18n("de"));
        assert_eq!(rows[0][1], "Analyse");
        assert_eq!(rows[0][2], "USA");
        let rows = check_rows(&fixtures::partial(), &i18n("de"));
        assert_eq!(rows[0][1], "Nicht gemessen");
    }

    /// Guard (#406): English output holds no German umlauts or ß.
    #[test]
    fn english_output_has_no_german_characters() {
        let en = i18n("en");
        let mut all = String::new();
        for a in [
            fixtures::tracked(),
            fixtures::clean(),
            fixtures::partial(),
            fixtures::not_measured(),
        ] {
            all.push_str(&takeaway(&a, &en));
            all.push_str(&measurement_note(&a, &en).unwrap_or_default());
            for row in check_rows(&a, &en)
                .into_iter()
                .chain(contact_rows(&a, &en))
                .chain(finding_rows(&a.findings, &en))
            {
                all.push_str(&row.join(" "));
            }
        }
        for key in [
            "privacy-title",
            "privacy-scope",
            "privacy-batch-intro",
            "privacy-batch-matrix-legend",
        ] {
            all.push_str(&en.t(key));
        }
        assert!(
            !all.chars().any(|c| "äöüÄÖÜß".contains(c)),
            "German characters in English output"
        );
    }

    /// No legal verdict wording in either language.
    #[test]
    fn wording_carries_no_verdict() {
        for locale in ["de", "en"] {
            let i = i18n(locale);
            let mut all = String::new();
            for a in [fixtures::tracked(), fixtures::clean(), fixtures::partial()] {
                all.push_str(&takeaway(&a, &i));
                for row in finding_rows(&a.findings, &i) {
                    all.push_str(&row.join(" "));
                }
            }
            all.push_str(&i.t("privacy-scope"));
            all.push_str(&i.t("privacy-batch-intro"));
            let lower = all.to_lowercase();
            for banned in [
                "violation",
                "unlawful",
                "illegal",
                "non-compliant",
                "verstoß",
                "rechtswidrig",
                "unzulässig",
                "dsgvo-konform",
            ] {
                assert!(!lower.contains(banned), "{locale}: {banned}");
            }
        }
    }
}
