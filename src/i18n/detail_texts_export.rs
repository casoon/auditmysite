//! Text export of the SEO detail-table texts and the evidence labels of
//! slice 5 of #936: every text of `seo::schema_fit::SchemaPageKind::label`,
//! `seo::schema_fit::SchemaFitAssessment::fit_text`,
//! `seo::schema_parity::ContentParityAssessment::status_text` and
//! `::evidence_text`, `seo::page_health::html_validation_check_text`,
//! `html_validation_detail_text`, `html_validator_delegated_text` and
//! `html_validator_executed_text`, `assessment::EvidenceSource::label` and
//! `assessment::evidence_field_label`, across an enumerated input space in
//! both languages, as records in `reports/detail-texts.json` (gitignored; the
//! path can be set with `DETAIL_TEXT_EXPORT`). Run it before and after a
//! change of how the texts are stored and compare the two files byte for byte.

use crate::assessment::{evidence_field_label, EvidenceSource};
use crate::seo::page_health::{
    html_validation_check_text, html_validation_detail_text, html_validator_delegated_text,
    html_validator_executed_text, HtmlValidationKind,
};
use crate::seo::schema_fit::{
    SchemaCoverageStatus, SchemaFitAssessment, SchemaFitStatus, SchemaPageKind,
};
use crate::seo::schema_parity::{ContentParityAssessment, ContentParityStatus};
use serde_json::{json, Value};

pub(crate) const SCHEMA_PAGE_KINDS: [SchemaPageKind; 18] = [
    SchemaPageKind::MerchantProductDetail,
    SchemaPageKind::ProductCollection,
    SchemaPageKind::ServiceDetail,
    SchemaPageKind::SoftwareDetail,
    SchemaPageKind::JobDetail,
    SchemaPageKind::JobListing,
    SchemaPageKind::EventDetail,
    SchemaPageKind::EventListing,
    SchemaPageKind::FaqPage,
    SchemaPageKind::PersonProfile,
    SchemaPageKind::LocationDetail,
    SchemaPageKind::EditorialProductReview,
    SchemaPageKind::EditorialContent,
    SchemaPageKind::MarketingLandingPage,
    SchemaPageKind::CorporatePage,
    SchemaPageKind::HubPage,
    SchemaPageKind::LeadGenerationPage,
    SchemaPageKind::Indeterminate,
];

pub(crate) const SCHEMA_FIT_STATUSES: [SchemaFitStatus; 4] = [
    SchemaFitStatus::Matched,
    SchemaFitStatus::Plausible,
    SchemaFitStatus::Mismatch,
    SchemaFitStatus::Indeterminate,
];

pub(crate) const CONTENT_PARITY_STATUSES: [ContentParityStatus; 3] = [
    ContentParityStatus::Match,
    ContentParityStatus::Mismatch,
    ContentParityStatus::NotEvaluated,
];

pub(crate) const HTML_VALIDATION_KINDS: [HtmlValidationKind; 6] = [
    HtmlValidationKind::DuplicateIds,
    HtmlValidationKind::ImagesWithoutAlt,
    HtmlValidationKind::TablesWithoutHeaders,
    HtmlValidationKind::EmptyHeadings,
    HtmlValidationKind::NestedInteractive,
    HtmlValidationKind::ParseErrors,
];

pub(crate) fn evidence_sources() -> [EvidenceSource; 9] {
    [
        EvidenceSource::Meta,
        EvidenceSource::JsonLd,
        EvidenceSource::VisibleText,
        EvidenceSource::Link,
        EvidenceSource::HttpHeader,
        EvidenceSource::AxTree,
        EvidenceSource::DomAttribute,
        EvidenceSource::CssProperty,
        EvidenceSource::Computed,
    ]
}

/// Every path `evidence_field_label` names, and paths it shows as they are or
/// drops.
pub(crate) const EVIDENCE_FIELD_PATHS: [&str; 22] = [
    "heading.h1",
    "meta.title",
    "meta.description",
    "ssr_content_gap",
    "canonical",
    "patterns.recognized",
    "structured_data.type",
    "structured_data.rich_snippets",
    "structured_data",
    "headings.h2_plus",
    "internal_links",
    "word_count",
    "",
    "some_new_internal_key",
    "meta.title.extra",
    "Heading.H1",
    "FAQPage",
    "html[lang]",
    "LocalBusiness.geo.latitude",
    "{braces} and {$var}",
    "äöüß — „quotes“ \"x\" <b>; semi",
    "https://example.com/a-very-long-path/that-keeps-going/and-going/and-going/and-going/file.min.js?v=1234567890",
];

/// Counts of an HTML validation finding: 0, 1, 2, 5, one that Fluent could
/// group and the largest stored value.
pub(crate) const COUNTS: [u32; 7] = [0, 1, 2, 5, 1_500, 1_234_567, u32::MAX];

pub(crate) const TEXTS: [&str; 6] = [
    "",
    "https://example.com/a",
    " padded value ",
    "{braces} and {$var}",
    "äöüß — „quotes“ \"x\" <b>; semi",
    "https://example.com/a-very-long-path/that-keeps-going/and-going/and-going/and-going/and-going/and-going/and-going/and-going/file.min.js?v=1234567890",
];

/// Sample lists: none, one, two and many (every text of [`TEXTS`]).
pub(crate) fn sample_lists() -> Vec<Vec<String>> {
    let mut lists = vec![
        vec![],
        vec!["hero-swiper".to_string()],
        vec!["hero-swiper".to_string(), "webshop-widget".to_string()],
        TEXTS.iter().map(|t| t.to_string()).collect(),
    ];
    lists.extend(TEXTS.iter().map(|t| vec![t.to_string()]));
    lists
}

pub(crate) fn schema_fit(status: SchemaFitStatus) -> SchemaFitAssessment {
    SchemaFitAssessment {
        page_kind: SchemaPageKind::Indeterminate,
        confidence: 0,
        fit_status: status,
        coverage_status: SchemaCoverageStatus::ManualReview,
        expected_primary_types: vec![],
        detected_primary_types: vec![],
        evidence: vec![],
    }
}

/// Every parity status with no, empty and other schema and visible values.
pub(crate) fn content_parities() -> Vec<ContentParityAssessment> {
    let values: Vec<Option<String>> = std::iter::once(None)
        .chain(TEXTS.iter().map(|t| Some(t.to_string())))
        .collect();
    let mut out = Vec::new();
    for status in CONTENT_PARITY_STATUSES {
        for schema_value in &values {
            for visible_value in &values {
                out.push(ContentParityAssessment {
                    node_index: 0,
                    schema_type: "Product".to_string(),
                    property: "offers.price".to_string(),
                    status,
                    schema_value: schema_value.clone(),
                    visible_value: visible_value.clone(),
                    evidence: String::new(),
                });
            }
        }
    }
    out
}

const LANGS: [(&str, bool); 2] = [("en", true), ("de", false)];

#[test]
#[ignore = "writes reports/detail-texts.json; run before and after a change of the text storage"]
fn export_detail_texts() {
    let mut records: Vec<Value> = Vec::new();
    for kind in SCHEMA_PAGE_KINDS {
        for (lang, en) in LANGS {
            records.push(json!([
                "SchemaPageKind::label",
                format!("{kind:?}"),
                lang,
                kind.label(en)
            ]));
        }
    }
    for status in SCHEMA_FIT_STATUSES {
        let fit = schema_fit(status);
        for (lang, en) in LANGS {
            records.push(json!([
                "SchemaFitAssessment::fit_text",
                format!("{status:?}"),
                lang,
                fit.fit_text(en)
            ]));
        }
    }
    for parity in content_parities() {
        let input = format!(
            "{:?} schema={:?} visible={:?}",
            parity.status, parity.schema_value, parity.visible_value
        );
        for (lang, en) in LANGS {
            records.push(json!([
                "ContentParityAssessment::status_text",
                input,
                lang,
                parity.status_text(en)
            ]));
            records.push(json!([
                "ContentParityAssessment::evidence_text",
                input,
                lang,
                parity.evidence_text(en)
            ]));
        }
    }
    for kind in HTML_VALIDATION_KINDS {
        for (lang, en) in LANGS {
            records.push(json!([
                "html_validation_check_text",
                format!("{kind:?}"),
                lang,
                html_validation_check_text(kind, en)
            ]));
        }
        for count in COUNTS {
            for samples in sample_lists() {
                for (lang, en) in LANGS {
                    records.push(json!([
                        "html_validation_detail_text",
                        format!("{kind:?} count={count} samples={samples:?}"),
                        lang,
                        html_validation_detail_text(kind, count, &samples, en)
                    ]));
                }
            }
        }
    }
    for (lang, en) in LANGS {
        records.push(json!([
            "html_validator_delegated_text",
            "",
            lang,
            html_validator_delegated_text(en)
        ]));
        records.push(json!([
            "html_validator_executed_text",
            "",
            lang,
            html_validator_executed_text(en)
        ]));
    }
    for source in evidence_sources() {
        for (lang, en) in LANGS {
            records.push(json!([
                "EvidenceSource::label",
                format!("{source:?}"),
                lang,
                source.label(en)
            ]));
        }
    }
    for path in EVIDENCE_FIELD_PATHS {
        for (lang, en) in LANGS {
            records.push(json!([
                "evidence_field_label",
                path,
                lang,
                evidence_field_label(path, en)
            ]));
        }
    }
    let path = std::env::var("DETAIL_TEXT_EXPORT")
        .unwrap_or_else(|_| format!("{}/reports/detail-texts.json", env!("CARGO_MANIFEST_DIR")));
    std::fs::create_dir_all(std::path::Path::new(&path).parent().unwrap()).unwrap();
    std::fs::write(&path, serde_json::to_string_pretty(&records).unwrap()).unwrap();
    println!("{} detail text records written to {path}", records.len());
}
