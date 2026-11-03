//! Pre-consent findings from first-visit data (#871).
//!
//! [`analyze_privacy`] reads a [`FirstVisitCapture`] (#870), classifies hosts
//! and cookie / storage names with `web-checks` (barrierlab#91) and reports
//! what is technically observable — **not** whether a page complies with
//! GDPR / ePrivacy. There is no compliance score and no pass / fail verdict;
//! "unknown" is never a finding.
//!
//! * **Confidence class.** `evidence`: the request, cookie or storage key was
//!   observed and the classification names its purpose. `indication`: a
//!   heuristic (a tag manager that may fire tags, an embed that may be the
//!   content the visitor asked for, a missing consent trace, a missing link).
//!   `manual-check`: only a person can decide (reserved; no automatic rule
//!   emits it yet, the class exists so the report can carry it).
//! * **Phase A only.** One visit, no interaction: nothing is clicked, no
//!   consent is given or refused. What the page does after a choice is not
//!   measured (#874).
//! * **Not measured.** A capture that failed, or a part of it that is
//!   missing, never reads as a clean result (#777): every check reports an
//!   outcome of its own, and `not_measured` is one of them.
//! * **Localization (#406).** The analysis stores canonical English; the
//!   single text source is [`finding_text`], which the report layer calls
//!   with the run language.
//!
//! Item 6 (policy / imprint links) uses the anchors of the first-visit
//! document only. The link target is **not fetched**: reachability beyond
//! "the link has a usable target" is a manual check, stated in
//! `docs/privacy-findings.md`.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use web_checks::third_party::{
    classify_host, classify_storage_name, classify_url, Category, Jurisdiction, RULESET_VERSION,
};

use super::first_visit::{CapturedLink, FirstVisitCapture};
use crate::audit::collection::{Collected, CollectionState};

/// How sure a finding is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConfidenceClass {
    Evidence,
    Indication,
    ManualCheck,
}

/// The six checks of phase A.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrivacyCheck {
    TrackingRequests,
    NonEssentialStorage,
    VisitorIpTransfer,
    ThirdCountryProvider,
    ConsentPlatform,
    PolicyLinks,
}

impl PrivacyCheck {
    pub const ALL: [PrivacyCheck; 6] = [
        Self::TrackingRequests,
        Self::NonEssentialStorage,
        Self::VisitorIpTransfer,
        Self::ThirdCountryProvider,
        Self::ConsentPlatform,
        Self::PolicyLinks,
    ];
}

/// What one check came to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckOutcome {
    /// Measured, and there are findings.
    Found,
    /// Measured, and there is nothing to report.
    NotFound,
    /// The check has nothing to look at on this page.
    NotApplicable,
    /// The data the check needs is missing or incomplete: no statement.
    NotMeasured,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrivacyCheckResult {
    pub check: PrivacyCheck,
    pub outcome: CheckOutcome,
    /// Stable code explaining `not_applicable` / `not_measured`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// Kind of a finding; with [`finding_text`] the only source of its text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrivacyFindingKind {
    /// Analytics / advertising / tag-manager request before any consent.
    TrackingRequestBeforeConsent,
    /// Cookie or storage key of a non-essential provider before any consent.
    NonEssentialStorageBeforeConsent,
    /// Fonts / CDN / embed request that hands the visitor IP to a third party.
    VisitorIpTransfer,
    /// A contacted provider is seated outside the EU (hint only).
    ThirdCountryProvider,
    /// Trackers present, no consent platform trace.
    NoConsentPlatformDetected,
    /// No privacy policy / imprint link found.
    PolicyLinkMissing,
    /// A policy / imprint link exists but has no usable target.
    PolicyLinkUnusable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyDocument {
    Privacy,
    Imprint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceSource {
    Request,
    Cookie,
    StorageKey,
    Link,
    Signal,
    Provider,
}

/// One observation behind a finding. Never a cookie or storage value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Evidence {
    pub source: EvidenceSource,
    /// Request URL without query and fragment, cookie / key name, link
    /// target, signal id or host.
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderRef {
    pub id: String,
    pub name: String,
    /// `analytics`, `advertising`, `fonts`, ... (`web-checks` ids).
    pub category: String,
    /// `eu`, `us`, `other`, `unknown`: the provider's own statement of its
    /// seat, nothing about the actual data flow.
    pub jurisdiction: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrivacyFinding {
    pub kind: PrivacyFindingKind,
    pub confidence: ConfidenceClass,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<ProviderRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub document: Option<PolicyDocument>,
    pub evidence: Vec<Evidence>,
    /// Canonical English; the report layer renders [`finding_text`] with
    /// the run language instead.
    pub message: String,
}

/// A provider the first visit contacted (any category), for the report's
/// "who was contacted" table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThirdPartyContact {
    pub provider: ProviderRef,
    pub hosts: Vec<String>,
    pub request_count: usize,
}

/// How completely the capture was measured.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeasurementState {
    pub state: CollectionState,
    /// Named parts a partial capture lost.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub gaps: Vec<String>,
    /// Why nothing was measured.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// What was measured, so the report claims no more than that.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrivacyScope {
    /// `"A"`: first visit, no interaction.
    pub phase: String,
    pub interaction: bool,
    pub fresh_context: bool,
    pub privacy_signals_sent: bool,
    pub settle_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrivacyAnalysis {
    /// `web-checks` ruleset the classification used.
    pub ruleset_version: String,
    pub scope: PrivacyScope,
    pub measurement: MeasurementState,
    /// One entry per check, in fixed order.
    pub checks: Vec<PrivacyCheckResult>,
    pub findings: Vec<PrivacyFinding>,
    /// Classified providers contacted, sorted by provider id.
    pub contacts: Vec<ThirdPartyContact>,
    /// Third-party hosts no provider list knows: contacted, not classified.
    pub unclassified_hosts: Vec<String>,
}

// ── Text (single source, #406) ─────────────────────────────────────────

/// Text of a finding. `provider` is the provider name (or empty), `document`
/// the policy document for the link findings. `en` selects English; the
/// analysis itself always passes `true`.
pub fn finding_text(
    kind: PrivacyFindingKind,
    provider: &str,
    document: Option<PolicyDocument>,
    en: bool,
) -> String {
    use PrivacyFindingKind as K;
    let doc = match (document, en) {
        (Some(PolicyDocument::Imprint), true) => "imprint",
        (Some(PolicyDocument::Imprint), false) => "Impressum",
        (_, true) => "privacy policy",
        (_, false) => "Datenschutzerklärung",
    };
    match (kind, en) {
        (K::TrackingRequestBeforeConsent, true) => format!(
            "{provider} was contacted on the first visit, before any consent choice was possible."
        ),
        (K::TrackingRequestBeforeConsent, false) => format!(
            "{provider} wurde beim ersten Besuch kontaktiert, bevor eine Einwilligungsentscheidung möglich war."
        ),
        (K::NonEssentialStorageBeforeConsent, true) => format!(
            "Cookies or storage entries of {provider} were present after the first visit, before any consent choice."
        ),
        (K::NonEssentialStorageBeforeConsent, false) => format!(
            "Cookies oder Speichereinträge von {provider} waren nach dem ersten Besuch vorhanden, vor jeder Einwilligungsentscheidung."
        ),
        (K::VisitorIpTransfer, true) => format!(
            "{provider} was contacted on the first visit; the request transfers the visitor's IP address to a third party."
        ),
        (K::VisitorIpTransfer, false) => format!(
            "{provider} wurde beim ersten Besuch kontaktiert; die Anfrage übermittelt die IP-Adresse des Besuchers an einen Dritten."
        ),
        (K::ThirdCountryProvider, true) => {
            "Providers contacted on the first visit state a seat outside the EU. This is a hint about the provider, not about the actual data flow."
                .to_string()
        }
        (K::ThirdCountryProvider, false) => {
            "Beim ersten Besuch kontaktierte Anbieter geben einen Sitz außerhalb der EU an. Das ist ein Hinweis auf den Anbieter, nicht auf den tatsächlichen Datenfluss."
                .to_string()
        }
        (K::NoConsentPlatformDetected, true) => {
            "Tracking requests were observed, but no consent platform could be detected on the first visit (requests, cookies, storage, document)."
                .to_string()
        }
        (K::NoConsentPlatformDetected, false) => {
            "Es wurden Tracking-Anfragen beobachtet, aber beim ersten Besuch ließ sich keine Consent-Plattform erkennen (Anfragen, Cookies, Speicher, Dokument)."
                .to_string()
        }
        (K::PolicyLinkMissing, true) => format!("No link to the {doc} was found on the page."),
        (K::PolicyLinkMissing, false) => {
            format!("Auf der Seite wurde kein Link zur {doc} gefunden.")
        }
        (K::PolicyLinkUnusable, true) => {
            format!("The link to the {doc} has no usable target (empty, '#' or script).")
        }
        (K::PolicyLinkUnusable, false) => {
            format!("Der Link zur {doc} hat kein nutzbares Ziel (leer, '#' oder Skript).")
        }
    }
}

// ── Classification ─────────────────────────────────────────────────────

fn provider_ref(m: &web_checks::third_party::Match) -> ProviderRef {
    ProviderRef {
        id: m.provider.id.to_string(),
        name: m.provider.name.to_string(),
        category: m.category.as_str().to_string(),
        jurisdiction: m.provider.jurisdiction.as_str().to_string(),
    }
}

/// Request URL without query and fragment.
fn bare_url(url: &str) -> String {
    url.split(['?', '#']).next().unwrap_or(url).to_string()
}

fn tracking_class(category: Category) -> Option<ConfidenceClass> {
    match category {
        Category::Analytics | Category::Advertising => Some(ConfidenceClass::Evidence),
        Category::TagManager => Some(ConfidenceClass::Indication),
        _ => None,
    }
}

fn ip_transfer_class(category: Category) -> Option<ConfidenceClass> {
    match category {
        Category::Fonts | Category::Cdn => Some(ConfidenceClass::Evidence),
        Category::Maps | Category::VideoEmbed | Category::Social => {
            Some(ConfidenceClass::Indication)
        }
        _ => None,
    }
}

#[derive(Default)]
struct Group {
    provider: Option<ProviderRef>,
    class: Option<ConfidenceClass>,
    evidence: BTreeSet<(u8, String)>,
}

fn source_rank(source: EvidenceSource) -> u8 {
    match source {
        EvidenceSource::Request => 0,
        EvidenceSource::Cookie => 1,
        EvidenceSource::StorageKey => 2,
        EvidenceSource::Link => 3,
        EvidenceSource::Signal => 4,
        EvidenceSource::Provider => 5,
    }
}

fn source_of(rank: u8) -> EvidenceSource {
    match rank {
        0 => EvidenceSource::Request,
        1 => EvidenceSource::Cookie,
        2 => EvidenceSource::StorageKey,
        3 => EvidenceSource::Link,
        4 => EvidenceSource::Signal,
        _ => EvidenceSource::Provider,
    }
}

fn into_findings(kind: PrivacyFindingKind, groups: BTreeMap<String, Group>) -> Vec<PrivacyFinding> {
    groups
        .into_values()
        .map(|g| {
            let provider = g.provider.expect("group has a provider");
            PrivacyFinding {
                kind,
                confidence: g.class.expect("group has a class"),
                message: finding_text(kind, &provider.name, None, true),
                provider: Some(provider),
                document: None,
                evidence: g
                    .evidence
                    .into_iter()
                    .map(|(rank, value)| Evidence {
                        source: source_of(rank),
                        value,
                    })
                    .collect(),
            }
        })
        .collect()
}

const PRIVACY_WORDS: &[&str] = &[
    "datenschutz",
    "privacy",
    "privatsphäre",
    "privacidad",
    "confidentialité",
    "protection des données",
    "privacybeleid",
];
const IMPRINT_WORDS: &[&str] = &[
    "impressum",
    "imprint",
    "legal notice",
    "legal-notice",
    "mentions légales",
    "aviso legal",
];

fn link_is(link: &CapturedLink, words: &[&str]) -> bool {
    let haystack = format!("{} {}", link.text, link.href).to_lowercase();
    words.iter().any(|w| haystack.contains(w))
}

/// A target a visitor can follow: not empty, not a bare `#`, not script.
fn usable_target(link: &CapturedLink) -> bool {
    let href = link.href.trim().to_ascii_lowercase();
    !(href.is_empty() || href == "#" || href.starts_with("javascript:") || link.resolved.is_empty())
}

// ── Analysis ───────────────────────────────────────────────────────────

fn scope(capture: Option<&FirstVisitCapture>) -> PrivacyScope {
    PrivacyScope {
        phase: "A".into(),
        interaction: false,
        fresh_context: capture.map(|c| c.profile.fresh_context).unwrap_or(true),
        privacy_signals_sent: capture
            .map(|c| c.profile.privacy_signals_sent)
            .unwrap_or(false),
        settle_ms: capture.map(|c| c.profile.settle_ms).unwrap_or(0),
    }
}

/// Findings of a first-visit capture. A collection without a value yields an
/// analysis whose checks are all `not_measured`.
pub fn analyze_privacy(collected: &Collected<FirstVisitCapture>) -> PrivacyAnalysis {
    let (state, gaps, reason) = match collected {
        Collected::Partial { gaps, .. } => (
            collected.state(),
            gaps.iter().map(|g| g.to_string()).collect(),
            None,
        ),
        Collected::Failed(r) | Collected::Unsupported(r) | Collected::NotApplicable(r) => {
            (collected.state(), Vec::new(), Some(r.to_string()))
        }
        _ => (collected.state(), Vec::new(), None),
    };
    let measurement = MeasurementState {
        state,
        gaps,
        reason,
    };
    let Some(capture) = collected.value() else {
        let reason = measurement.reason.clone();
        return PrivacyAnalysis {
            ruleset_version: RULESET_VERSION.to_string(),
            scope: scope(None),
            checks: PrivacyCheck::ALL
                .iter()
                .map(|&check| PrivacyCheckResult {
                    check,
                    outcome: CheckOutcome::NotMeasured,
                    reason: reason.clone(),
                })
                .collect(),
            measurement,
            findings: Vec::new(),
            contacts: Vec::new(),
            unclassified_hosts: Vec::new(),
        };
    };
    let has_gap = |g: &str| measurement.gaps.iter().any(|x| x == g);
    let incomplete_requests = has_gap("network_not_idle");

    // Classify every third-party request once.
    struct Seen<'a> {
        url: &'a str,
        host: &'a str,
        m: web_checks::third_party::Match,
    }
    let mut seen: Vec<Seen> = Vec::new();
    let mut unclassified: BTreeSet<String> = BTreeSet::new();
    let mut third_party_requests = 0usize;
    for r in capture.requests.iter().filter(|r| r.third_party) {
        third_party_requests += 1;
        match classify_url(&r.url) {
            Some(m) => seen.push(Seen {
                url: &r.url,
                host: &r.host,
                m,
            }),
            None => {
                unclassified.insert(r.host.clone());
            }
        }
    }

    // 1. Tracking requests and 3. visitor IP transfer.
    let mut tracking: BTreeMap<String, Group> = BTreeMap::new();
    let mut ip: BTreeMap<String, Group> = BTreeMap::new();
    for s in &seen {
        for (groups, class) in [
            (&mut tracking, tracking_class(s.m.category)),
            (&mut ip, ip_transfer_class(s.m.category)),
        ] {
            let Some(class) = class else { continue };
            let g = groups.entry(s.m.provider.id.to_string()).or_default();
            g.provider = Some(provider_ref(&s.m));
            g.class = Some(g.class.map_or(class, |c| c.min(class)));
            g.evidence
                .insert((source_rank(EvidenceSource::Request), bare_url(s.url)));
        }
    }
    // 2. Non-essential cookies and storage keys.
    let mut storage: BTreeMap<String, Group> = BTreeMap::new();
    let mut note = |name: &str, source: EvidenceSource, value: String| {
        let Some(m) = classify_storage_name(name) else {
            return;
        };
        if m.category == Category::ConsentPlatform {
            return;
        }
        let g = storage.entry(m.provider.id.to_string()).or_default();
        g.provider = Some(provider_ref(&m));
        g.class = Some(ConfidenceClass::Evidence);
        g.evidence.insert((source_rank(source), value));
    };
    for c in &capture.cookies {
        note(&c.name, EvidenceSource::Cookie, c.name.clone());
    }
    for k in &capture.storage {
        note(&k.key, EvidenceSource::StorageKey, k.key.clone());
    }

    // Consent platform: request, cookie / storage name, or document trace.
    let consent_by_data = seen
        .iter()
        .any(|s| s.m.category == Category::ConsentPlatform)
        || capture
            .cookies
            .iter()
            .map(|c| c.name.as_str())
            .chain(capture.storage.iter().map(|k| k.key.as_str()))
            .any(|n| {
                classify_storage_name(n).is_some_and(|m| m.category == Category::ConsentPlatform)
            });
    let consent_by_document = !capture.signals.consent_signals.is_empty();

    // Contacts (every classified provider, consent platforms included).
    let mut contact_map: BTreeMap<String, (ProviderRef, BTreeSet<String>, usize)> = BTreeMap::new();
    for s in &seen {
        let e = contact_map
            .entry(s.m.provider.id.to_string())
            .or_insert_with(|| (provider_ref(&s.m), BTreeSet::new(), 0));
        e.1.insert(s.host.to_string());
        e.2 += 1;
    }
    let contacts: Vec<ThirdPartyContact> = contact_map
        .into_values()
        .map(|(provider, hosts, request_count)| ThirdPartyContact {
            provider,
            hosts: hosts.into_iter().collect(),
            request_count,
        })
        .collect();

    let mut findings = Vec::new();
    let mut results = Vec::new();
    let mut result = |check, outcome, reason: Option<&str>| {
        results.push(PrivacyCheckResult {
            check,
            outcome,
            reason: reason.map(str::to_string),
        });
    };

    // Outcome of a "looking for requests" check.
    let request_outcome = |found: bool| {
        if found {
            (CheckOutcome::Found, None)
        } else if third_party_requests == 0 && !incomplete_requests {
            (CheckOutcome::NotApplicable, Some("no_third_party_requests"))
        } else if incomplete_requests {
            (CheckOutcome::NotMeasured, Some("network_not_idle"))
        } else {
            (CheckOutcome::NotFound, None)
        }
    };

    let tracking_found = !tracking.is_empty();
    let (o, r) = request_outcome(tracking_found);
    result(PrivacyCheck::TrackingRequests, o, r);
    findings.extend(into_findings(
        PrivacyFindingKind::TrackingRequestBeforeConsent,
        tracking,
    ));

    let storage_found = !storage.is_empty();
    let storage_gap = has_gap("cookies") || has_gap("storage");
    let (o, r) = if storage_found {
        (CheckOutcome::Found, None)
    } else if storage_gap {
        (CheckOutcome::NotMeasured, Some("storage_unreadable"))
    } else if capture.cookies.is_empty() && capture.storage.is_empty() {
        (CheckOutcome::NotApplicable, Some("nothing_stored"))
    } else {
        (CheckOutcome::NotFound, None)
    };
    result(PrivacyCheck::NonEssentialStorage, o, r);
    findings.extend(into_findings(
        PrivacyFindingKind::NonEssentialStorageBeforeConsent,
        storage,
    ));

    let (o, r) = request_outcome(!ip.is_empty());
    result(PrivacyCheck::VisitorIpTransfer, o, r);
    findings.extend(into_findings(PrivacyFindingKind::VisitorIpTransfer, ip));

    // 4. Third-country hint over every classified provider contacted.
    let third_country: Vec<&ThirdPartyContact> = contacts
        .iter()
        .filter(|c| {
            c.provider.category != Category::ConsentPlatform.as_str()
                && [Jurisdiction::Us.as_str(), Jurisdiction::Other.as_str()]
                    .contains(&c.provider.jurisdiction.as_str())
        })
        .collect();
    if !third_country.is_empty() {
        result(
            PrivacyCheck::ThirdCountryProvider,
            CheckOutcome::Found,
            None,
        );
        findings.push(PrivacyFinding {
            kind: PrivacyFindingKind::ThirdCountryProvider,
            confidence: ConfidenceClass::Indication,
            provider: None,
            document: None,
            evidence: third_country
                .iter()
                .map(|c| Evidence {
                    source: EvidenceSource::Provider,
                    value: format!("{} ({})", c.provider.id, c.provider.jurisdiction),
                })
                .collect(),
            message: finding_text(PrivacyFindingKind::ThirdCountryProvider, "", None, true),
        });
    } else if incomplete_requests {
        result(
            PrivacyCheck::ThirdCountryProvider,
            CheckOutcome::NotMeasured,
            Some("network_not_idle"),
        );
    } else if contacts.is_empty() {
        result(
            PrivacyCheck::ThirdCountryProvider,
            CheckOutcome::NotApplicable,
            Some("no_classified_provider"),
        );
    } else {
        result(
            PrivacyCheck::ThirdCountryProvider,
            CheckOutcome::NotFound,
            None,
        );
    }

    // 5. Trackers without a consent platform.
    let trackers_present = findings
        .iter()
        .any(|f| f.kind == PrivacyFindingKind::TrackingRequestBeforeConsent);
    if !trackers_present {
        let (o, r) = if incomplete_requests {
            (CheckOutcome::NotMeasured, Some("network_not_idle"))
        } else {
            (CheckOutcome::NotApplicable, Some("no_tracking_requests"))
        };
        result(PrivacyCheck::ConsentPlatform, o, r);
    } else if consent_by_data || consent_by_document {
        result(PrivacyCheck::ConsentPlatform, CheckOutcome::NotFound, None);
    } else if has_gap("page_signals") || storage_gap {
        result(
            PrivacyCheck::ConsentPlatform,
            CheckOutcome::NotMeasured,
            Some("consent_traces_unreadable"),
        );
    } else {
        result(PrivacyCheck::ConsentPlatform, CheckOutcome::Found, None);
        findings.push(PrivacyFinding {
            kind: PrivacyFindingKind::NoConsentPlatformDetected,
            confidence: ConfidenceClass::Indication,
            provider: None,
            document: None,
            evidence: Vec::new(),
            message: finding_text(
                PrivacyFindingKind::NoConsentPlatformDetected,
                "",
                None,
                true,
            ),
        });
    }

    // 6. Policy and imprint links.
    if has_gap("page_signals") {
        result(
            PrivacyCheck::PolicyLinks,
            CheckOutcome::NotMeasured,
            Some("page_signals_unreadable"),
        );
    } else if capture.signals.links.is_empty() {
        result(
            PrivacyCheck::PolicyLinks,
            CheckOutcome::NotApplicable,
            Some("no_links_on_page"),
        );
    } else {
        let before = findings.len();
        for (document, words) in [
            (PolicyDocument::Privacy, PRIVACY_WORDS),
            (PolicyDocument::Imprint, IMPRINT_WORDS),
        ] {
            let candidates: Vec<&CapturedLink> = capture
                .signals
                .links
                .iter()
                .filter(|l| link_is(l, words))
                .collect();
            let (kind, evidence) = if candidates.is_empty() {
                (PrivacyFindingKind::PolicyLinkMissing, Vec::new())
            } else if candidates.iter().any(|l| usable_target(l)) {
                continue;
            } else {
                (
                    PrivacyFindingKind::PolicyLinkUnusable,
                    candidates
                        .iter()
                        .map(|l| Evidence {
                            source: EvidenceSource::Link,
                            value: l.href.clone(),
                        })
                        .collect(),
                )
            };
            findings.push(PrivacyFinding {
                kind,
                // An absent link may be named in a way the word list does not
                // know; a link seen with a dead target is observed.
                confidence: if kind == PrivacyFindingKind::PolicyLinkUnusable {
                    ConfidenceClass::Evidence
                } else {
                    ConfidenceClass::Indication
                },
                provider: None,
                document: Some(document),
                evidence,
                message: finding_text(kind, "", Some(document), true),
            });
        }
        result(
            PrivacyCheck::PolicyLinks,
            if findings.len() > before {
                CheckOutcome::Found
            } else {
                CheckOutcome::NotFound
            },
            None,
        );
    }

    PrivacyAnalysis {
        ruleset_version: RULESET_VERSION.to_string(),
        scope: scope(Some(capture)),
        measurement,
        checks: results,
        findings,
        contacts,
        unclassified_hosts: unclassified.into_iter().collect(),
    }
}

/// Used by the host-based classification tests of the corpus.
#[doc(hidden)]
pub fn host_category(host: &str) -> Option<&'static str> {
    classify_host(host).map(|m| m.category.as_str())
}
