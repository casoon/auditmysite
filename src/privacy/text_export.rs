//! Text export of `privacy::findings` (#936): `finding_text` in both languages
//! over every finding kind, every policy-document variant (none, privacy
//! policy, imprint) and provider names (empty, plain, with non-ASCII and
//! placeholder-like characters), as records in `reports/privacy-texts.json`
//! (gitignored; the path can be set with `PRIVACY_TEXT_EXPORT`). Run it before
//! and after a change of how the texts are stored and compare the two files
//! byte for byte.

use super::findings::{finding_text, PolicyDocument, PrivacyFindingKind};
use serde_json::{json, Value};

pub(crate) const KINDS: [PrivacyFindingKind; 7] = [
    PrivacyFindingKind::TrackingRequestBeforeConsent,
    PrivacyFindingKind::NonEssentialStorageBeforeConsent,
    PrivacyFindingKind::VisitorIpTransfer,
    PrivacyFindingKind::ThirdCountryProvider,
    PrivacyFindingKind::NoConsentPlatformDetected,
    PrivacyFindingKind::PolicyLinkMissing,
    PrivacyFindingKind::PolicyLinkUnusable,
];

pub(crate) const DOCUMENTS: [Option<PolicyDocument>; 3] = [
    None,
    Some(PolicyDocument::Privacy),
    Some(PolicyDocument::Imprint),
];

pub(crate) const PROVIDERS: [&str; 4] = ["", "Google Fonts", "Ünï {$x} Cörp", "Mapbox, Inc."];

/// Every text of `privacy::findings` in one language.
pub(crate) fn privacy_texts(en: bool) -> Vec<Value> {
    let mut out = Vec::new();
    for kind in KINDS {
        for document in DOCUMENTS {
            for provider in PROVIDERS {
                out.push(json!([
                    "finding_text",
                    format!("{kind:?}/{document:?}/{provider}"),
                    finding_text(kind, provider, document, en)
                ]));
            }
        }
    }
    out
}

#[test]
#[ignore = "writes reports/privacy-texts.json; run before and after a change of the text storage"]
fn export_privacy_texts() {
    let mut records = Vec::new();
    for lang in ["en", "de"] {
        for text in privacy_texts(lang == "en") {
            records.push(json!({ "lang": lang, "text": text }));
        }
    }
    let path = std::env::var("PRIVACY_TEXT_EXPORT")
        .unwrap_or_else(|_| format!("{}/reports/privacy-texts.json", env!("CARGO_MANIFEST_DIR")));
    std::fs::create_dir_all(std::path::Path::new(&path).parent().unwrap()).unwrap();
    std::fs::write(&path, serde_json::to_string_pretty(&records).unwrap()).unwrap();
    println!("{} privacy text records written to {path}", records.len());
}
