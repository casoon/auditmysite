//! Findings from a first-visit capture (#871).
//!
//! Pure: takes a [`FirstVisitCapture`], returns what was observed and what that
//! indicates. No browser, no network, no verdict. A finding states a *technical
//! observation* ("a request to an analytics host left the browser before any
//! interaction"), never that the site violates a law.
//!
//! Classification (host, URL, cookie name → provider, category, seat) comes from
//! `web-checks::third_party` (casoon/barrierlab#91). A host that list does not
//! know is not a finding.
//!
//! # Confidence
//!
//! * [`Confidence::Evidence`] — a request or a stored name was observed and
//!   matched to a provider of that category.
//! * [`Confidence::Indication`] — follows from the observation but rests on an
//!   assumption (a tag manager's payload is invisible by host; a missing
//!   consent platform may be one the list does not know).
//!
//! Wording follows the localization architecture (#406): the stored struct
//! carries a canonical `kind` and the raw subjects; [`privacy_finding_text`] is
//! the only source of text and the analysis calls it with `en = true`.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use web_checks::third_party::{
    classify_host, classify_storage_name, classify_url, Category, Jurisdiction, Match,
    RULESET_VERSION,
};

use super::first_visit::{CaptureProfile, FirstVisitCapture, StorageArea};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
    Evidence,
    Indication,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrivacyFindingKind {
    /// Requests to analytics or advertising hosts without any interaction.
    TrackingRequestBeforeConsent,
    /// Cookies or storage keys of analytics or advertising providers.
    TrackingStorageBeforeConsent,
    /// Fonts, CDN, video, map or social hosts contacted: the visitor's IP
    /// address reaches a third party.
    ThirdPartyEmbedContacted,
    /// A tag manager loaded; what it fires is not visible by host.
    TagManagerLoaded,
    /// Contacted providers whose seat is outside the EU.
    ProviderOutsideEu,
    /// Tracking observed, no consent platform recognised.
    TrackingWithoutConsentPlatform,
}

impl PrivacyFindingKind {
    /// Stable snake_case code (the serialized form), for finding ids.
    pub fn code(self) -> &'static str {
        match self {
            Self::TrackingRequestBeforeConsent => "tracking_request_before_consent",
            Self::TrackingStorageBeforeConsent => "tracking_storage_before_consent",
            Self::ThirdPartyEmbedContacted => "third_party_embed_contacted",
            Self::TagManagerLoaded => "tag_manager_loaded",
            Self::ProviderOutsideEu => "provider_outside_eu",
            Self::TrackingWithoutConsentPlatform => "tracking_without_consent_platform",
        }
    }
}

/// A third-party provider the page contacted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderContact {
    pub provider_id: String,
    pub provider: String,
    pub category: String,
    pub jurisdiction: String,
    /// Hosts of this provider that were requested, sorted.
    pub hosts: Vec<String>,
    pub requests: usize,
}

/// A cookie or storage key that belongs to a known provider. Values are never kept.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderStorage {
    pub provider_id: String,
    pub provider: String,
    pub category: String,
    /// `cookie`, `local_storage` or `session_storage`.
    pub kind: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrivacyFinding {
    pub kind: PrivacyFindingKind,
    pub confidence: Confidence,
    /// Provider names (sorted), the subjects of the finding.
    pub subjects: Vec<String>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrivacyAnalysis {
    /// Version of the provider list the classification used.
    pub ruleset_version: String,
    /// How the page was visited; the report states it as the scope.
    pub profile: CaptureProfile,
    /// The capture is missing a part (network never idle, storage unreadable).
    pub partial: bool,
    pub contacts: Vec<ProviderContact>,
    pub stored: Vec<ProviderStorage>,
    /// Consent platforms recognised by request host or stored name.
    pub consent_platforms: Vec<String>,
    pub findings: Vec<PrivacyFinding>,
}

/// Categories that identify or profile a visitor on their own.
fn is_tracking(category: &str) -> bool {
    category == Category::Analytics.as_str() || category == Category::Advertising.as_str()
}

/// Categories that only transfer the IP address of the visitor.
fn is_embed(category: &str) -> bool {
    [
        Category::Fonts,
        Category::Cdn,
        Category::VideoEmbed,
        Category::Maps,
        Category::Social,
    ]
    .iter()
    .any(|c| c.as_str() == category)
}

/// `partial` is true when the capture reported a gap.
pub fn analyze_first_visit(capture: &FirstVisitCapture, partial: bool) -> PrivacyAnalysis {
    // The page is its own provider: youtube.com visiting youtube.com is no contact.
    let own_provider = classify_host(&capture.page_host).map(|m| m.provider.id);

    let mut by_provider: BTreeMap<&'static str, (Match, Vec<String>, usize)> = BTreeMap::new();
    for request in capture.requests.iter().filter(|r| r.third_party) {
        let Some(m) = classify_url(&request.url) else {
            continue;
        };
        if Some(m.provider.id) == own_provider {
            continue;
        }
        let entry = by_provider
            .entry(m.provider.id)
            .or_insert_with(|| (m, Vec::new(), 0));
        if !entry.1.contains(&request.host) {
            entry.1.push(request.host.clone());
        }
        entry.2 += 1;
    }
    let contacts: Vec<ProviderContact> = by_provider
        .into_values()
        .map(|(m, mut hosts, requests)| {
            hosts.sort();
            ProviderContact {
                provider_id: m.provider.id.into(),
                provider: m.provider.name.into(),
                category: m.category.as_str().into(),
                jurisdiction: m.provider.jurisdiction.as_str().into(),
                hosts,
                requests,
            }
        })
        .collect();

    let mut stored: Vec<ProviderStorage> = Vec::new();
    for cookie in &capture.cookies {
        if let Some(m) = classify_storage_name(&cookie.name) {
            stored.push(storage_entry(m, "cookie", &cookie.name));
        }
    }
    for key in &capture.storage {
        if let Some(m) = classify_storage_name(&key.key) {
            let kind = match key.area {
                StorageArea::Local => "local_storage",
                StorageArea::Session => "session_storage",
            };
            stored.push(storage_entry(m, kind, &key.key));
        }
    }
    stored.sort_by(|a, b| {
        (&a.provider_id, &a.kind, &a.name).cmp(&(&b.provider_id, &b.kind, &b.name))
    });
    stored.dedup();

    let consent = Category::ConsentPlatform.as_str();
    let mut consent_platforms: Vec<String> = contacts
        .iter()
        .filter(|c| c.category == consent)
        .map(|c| c.provider.clone())
        .chain(
            stored
                .iter()
                .filter(|s| s.category == consent)
                .map(|s| s.provider.clone()),
        )
        .collect();
    consent_platforms.sort();
    consent_platforms.dedup();

    let findings = findings(&contacts, &stored, &consent_platforms);

    PrivacyAnalysis {
        ruleset_version: RULESET_VERSION.into(),
        profile: capture.profile.clone(),
        partial,
        contacts,
        stored,
        consent_platforms,
        findings,
    }
}

fn storage_entry(m: Match, kind: &str, name: &str) -> ProviderStorage {
    ProviderStorage {
        provider_id: m.provider.id.into(),
        provider: m.provider.name.into(),
        category: m.category.as_str().into(),
        kind: kind.into(),
        name: name.into(),
    }
}

fn sorted(mut names: Vec<String>) -> Vec<String> {
    names.sort();
    names.dedup();
    names
}

fn findings(
    contacts: &[ProviderContact],
    stored: &[ProviderStorage],
    consent_platforms: &[String],
) -> Vec<PrivacyFinding> {
    use PrivacyFindingKind::*;

    let tracking_requests = sorted(
        contacts
            .iter()
            .filter(|c| is_tracking(&c.category))
            .map(|c| c.provider.clone())
            .collect(),
    );
    let tracking_storage = sorted(
        stored
            .iter()
            .filter(|s| is_tracking(&s.category))
            .map(|s| s.provider.clone())
            .collect(),
    );
    let embeds = sorted(
        contacts
            .iter()
            .filter(|c| is_embed(&c.category))
            .map(|c| c.provider.clone())
            .collect(),
    );
    let tag_managers = sorted(
        contacts
            .iter()
            .filter(|c| c.category == Category::TagManager.as_str())
            .map(|c| c.provider.clone())
            .collect(),
    );
    let outside_eu = sorted(
        contacts
            .iter()
            .filter(|c| {
                c.category != Category::ConsentPlatform.as_str()
                    && c.jurisdiction != Jurisdiction::Eu.as_str()
                    && c.jurisdiction != Jurisdiction::Unknown.as_str()
            })
            .map(|c| c.provider.clone())
            .collect(),
    );

    let mut tracking_all = tracking_requests.clone();
    tracking_all.extend(tracking_storage.clone());
    let tracking_all = sorted(tracking_all);

    let mut out = Vec::new();
    let mut push = |kind, confidence, subjects: Vec<String>| {
        out.push(PrivacyFinding {
            message: privacy_finding_text(kind, &subjects, true),
            kind,
            confidence,
            subjects,
        });
    };
    if !tracking_requests.is_empty() {
        push(
            TrackingRequestBeforeConsent,
            Confidence::Evidence,
            tracking_requests,
        );
    }
    if !tracking_storage.is_empty() {
        push(
            TrackingStorageBeforeConsent,
            Confidence::Evidence,
            tracking_storage,
        );
    }
    if !embeds.is_empty() {
        push(ThirdPartyEmbedContacted, Confidence::Evidence, embeds);
    }
    if !tag_managers.is_empty() {
        push(TagManagerLoaded, Confidence::Indication, tag_managers);
    }
    if !outside_eu.is_empty() {
        push(ProviderOutsideEu, Confidence::Indication, outside_eu);
    }
    if !tracking_all.is_empty() && consent_platforms.is_empty() {
        push(
            TrackingWithoutConsentPlatform,
            Confidence::Indication,
            tracking_all,
        );
    }
    out
}

/// Single source of truth for finding wording (#406). `subjects` are provider
/// names. Canonical English is `en = true`; the PDF layer passes the run
/// language.
pub fn privacy_finding_text(kind: PrivacyFindingKind, subjects: &[String], en: bool) -> String {
    use PrivacyFindingKind::*;
    let list = subjects.join(", ");
    match (kind, en) {
        (TrackingRequestBeforeConsent, true) => format!(
            "Requests to analytics or advertising providers were sent on first visit, before any interaction: {list}."
        ),
        (TrackingRequestBeforeConsent, false) => format!(
            "Beim ersten Besuch wurden vor jeder Interaktion Anfragen an Analyse- oder Werbeanbieter gesendet: {list}."
        ),
        (TrackingStorageBeforeConsent, true) => format!(
            "Cookies or storage entries of analytics or advertising providers exist after the first visit, before any interaction: {list}."
        ),
        (TrackingStorageBeforeConsent, false) => format!(
            "Nach dem ersten Besuch liegen vor jeder Interaktion Cookies oder Speichereinträge von Analyse- oder Werbeanbietern vor: {list}."
        ),
        (ThirdPartyEmbedContacted, true) => format!(
            "Third-party hosts for fonts, libraries, media, maps or social plugins were contacted; the visitor's IP address reaches them: {list}."
        ),
        (ThirdPartyEmbedContacted, false) => format!(
            "Fremde Hosts für Schriften, Bibliotheken, Medien, Karten oder Social-Plugins wurden kontaktiert; die IP-Adresse der Besucher gelangt dorthin: {list}."
        ),
        (TagManagerLoaded, true) => format!(
            "A tag manager was loaded on first visit ({list}); which tags it fires is not visible from the host alone."
        ),
        (TagManagerLoaded, false) => format!(
            "Beim ersten Besuch wurde ein Tag-Manager geladen ({list}); welche Tags er auslöst, ist am Host allein nicht erkennbar."
        ),
        (ProviderOutsideEu, true) => format!(
            "Contacted providers with a seat outside the EU, by the provider's own statement: {list}."
        ),
        (ProviderOutsideEu, false) => format!(
            "Kontaktierte Anbieter mit Sitz außerhalb der EU, nach eigener Angabe des Anbieters: {list}."
        ),
        (TrackingWithoutConsentPlatform, true) => {
            "Tracking was observed, but no known consent platform was recognised on the page."
                .into()
        }
        (TrackingWithoutConsentPlatform, false) => {
            "Es wurde Tracking beobachtet, aber keine bekannte Einwilligungsplattform auf der Seite erkannt."
                .into()
        }
    }
}

#[cfg(test)]
mod tests;
