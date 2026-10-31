//! DNS configuration audit (#545): opt-in, host-scoped checks for CAA,
//! DNSSEC, and SPF/MX records via direct DNS queries. Pure network/DNS —
//! no CDP/browser page is involved, and no `AuditReport`/score/certificate
//! field is ever touched by these findings (same category as
//! `design_quality` #528 and `ai_transparency`: opt-in and score-neutral).
//!
//! Runs once per host, not once per page — see [`NetworkDnsModule`]'s
//! per-host memoization.
//!
//! Scope (#776):
//! - **Answers**: every lookup is a positive answer, a proven absence
//!   (NXDOMAIN/NODATA) or unknown (timeout, SERVFAIL, refused). Unknown is
//!   never read as absent: it produces no finding and is recorded as a
//!   failed collection.
//! - **CAA**: walks from the host up to its parent domains (RFC 8659 tree
//!   climbing) and stops at the first name with CAA records. Without a
//!   public-suffix list the walk ends at the last two labels.
//! - **DNSSEC**: "a DNSKEY is published for the host or a parent domain" —
//!   the same walk, since the key sits at the zone apex. It is not
//!   chain-of-trust validation: no DS, no RRSIG check.
//! - **SPF**: applies when the host has an MX record. MX unknown leaves SPF
//!   unknown, not "not applicable".

pub mod module;
pub use module::NetworkDnsModule;

use hickory_resolver::proto::rr::{RData, RecordType};
use hickory_resolver::TokioResolver;
use serde::{Deserialize, Serialize};
use tracing::warn;

/// One resolver answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordLookup {
    /// Records of the type exist; their presentation form.
    Found(Vec<String>),
    /// NXDOMAIN or NODATA: proven absence.
    Absent,
    /// Timeout, SERVFAIL or another failure: nothing is known.
    Unknown(&'static str),
}

/// What a DNS check established for one host.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DnsAnswer {
    Present,
    Absent,
    /// The check does not apply (SPF without MX).
    NotApplicable,
    Unknown(&'static str),
}

impl DnsAnswer {
    /// `Some(true/false)` for an answer, `None` when unknown or not applicable.
    pub fn as_bool(self) -> Option<bool> {
        match self {
            Self::Present => Some(true),
            Self::Absent => Some(false),
            Self::NotApplicable | Self::Unknown(_) => None,
        }
    }

    /// The collection state of this check (#777).
    pub fn collected(self) -> crate::audit::collection::Collected<()> {
        use crate::audit::collection::Collected;
        match self {
            Self::Present => Collected::Measured(()),
            Self::Absent => Collected::Empty(()),
            Self::NotApplicable => Collected::NotApplicable("no_mx_record"),
            Self::Unknown(reason) => Collected::Failed(reason),
        }
    }
}

/// Raw DNS query results for one host — pure data, no message text baked in.
/// Kept separate from finding construction ([`build_analysis`]) so the
/// finding-mapping logic can be unit-tested without a real DNS query.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DnsCheckResult {
    pub caa: DnsAnswer,
    pub dnssec: DnsAnswer,
    pub mx: DnsAnswer,
    pub spf: DnsAnswer,
}

/// The resolver behind the checks; injectable so tests can answer with
/// NODATA, NXDOMAIN, SERVFAIL, timeouts and delegations.
#[async_trait::async_trait]
pub trait DnsLookup: Send + Sync {
    async fn lookup(&self, name: &str, record_type: RecordType) -> RecordLookup;
}

struct HickoryLookup(TokioResolver);

#[async_trait::async_trait]
impl DnsLookup for HickoryLookup {
    async fn lookup(&self, name: &str, record_type: RecordType) -> RecordLookup {
        match self.0.lookup(name, record_type).await {
            Ok(lookup) => {
                let answers: Vec<String> = lookup
                    .answers()
                    .iter()
                    .filter(|record| record.record_type() == record_type)
                    .map(|record| match &record.data {
                        RData::TXT(txt) => txt.to_string(),
                        other => other.to_string(),
                    })
                    .collect();
                if answers.is_empty() {
                    RecordLookup::Absent
                } else {
                    RecordLookup::Found(answers)
                }
            }
            Err(error) => lookup_error(&error),
        }
    }
}

/// What a resolver error says about the record: NODATA and NXDOMAIN prove
/// absence; a timeout or any other failure proves nothing (#776).
fn lookup_error(error: &hickory_resolver::net::NetError) -> RecordLookup {
    if error.is_no_records_found() || error.is_nx_domain() {
        RecordLookup::Absent
    } else if matches!(error, hickory_resolver::net::NetError::Timeout) {
        RecordLookup::Unknown("dns_timeout")
    } else {
        RecordLookup::Unknown("dns_lookup_failed")
    }
}

/// A single DNS-configuration finding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsFinding {
    /// Stable identifier, e.g. `"dns.caa_missing"`.
    pub rule_id: String,
    /// Canonical English message (#406 — the PDF re-derives via
    /// `finding_message_text` at the run locale).
    pub message: String,
}

/// Results of the DNS-configuration analysis for one host.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NetworkDnsAnalysis {
    pub host: String,
    /// CAA records for the host or the closest parent; `None` when unknown.
    pub caa_present: Option<bool>,
    /// A DNSKEY is published for the host or a parent (no chain
    /// validation); `None` when unknown.
    pub dnssec_detected: Option<bool>,
    /// `None` when unknown.
    pub mx_present: Option<bool>,
    /// `None` when there is no MX record — the SPF check is skipped
    /// entirely per #545 ("only when mail is routed through this domain"),
    /// not reported as pass or fail — or when MX or TXT are unknown; the
    /// collection record tells which.
    pub spf_present: Option<bool>,
    pub findings: Vec<DnsFinding>,
}

/// Localized presentation text for a DNS finding, keyed by its canonical
/// `rule_id` (#406 message-baked-struct pattern: [`build_analysis`] bakes
/// `DnsFinding.message` by calling this with `en=true` — JSON stays
/// canonical English — and the PDF builder calls it again with the run
/// locale).
pub fn finding_message_text(rule_id: &str, en: bool) -> String {
    match rule_id {
        "dns.caa_missing" => if en {
            "No CAA record was found for this domain. Without one, any certificate authority \
             is allowed to issue TLS certificates for it."
        } else {
            "Für diese Domain wurde kein CAA-Eintrag gefunden. Ohne CAA-Eintrag darf jede \
             Zertifizierungsstelle TLS-Zertifikate für sie ausstellen."
        }
        .to_string(),
        "dns.dnssec_not_detected" => if en {
            "No DNSKEY record was found at this domain's apex — DNSSEC signing does not \
             appear to be active."
        } else {
            "Am Apex dieser Domain wurde kein DNSKEY-Eintrag gefunden — DNSSEC-Signierung \
             scheint nicht aktiv zu sein."
        }
        .to_string(),
        "dns.spf_missing" => if en {
            "This domain has an MX record (mail is routed here) but no SPF TXT record, \
             making it easier for others to spoof its sender addresses."
        } else {
            "Diese Domain hat einen MX-Eintrag (Mail wird hierhin geroutet), aber keinen \
             SPF-TXT-Eintrag, wodurch sich ihre Absenderadressen leichter fälschen lassen."
        }
        .to_string(),
        _ => String::new(),
    }
}

/// Maps a raw [`DnsCheckResult`] into findings — pure, no network access, so
/// this is fully unit-testable without a real DNS query.
pub fn build_analysis(host: &str, result: &DnsCheckResult) -> NetworkDnsAnalysis {
    let mut findings = Vec::new();
    let mut finding = |rule_id: &str| {
        findings.push(DnsFinding {
            rule_id: rule_id.to_string(),
            message: finding_message_text(rule_id, true),
        })
    };
    // Only a proven absence is a finding; unknown answers stay silent.
    if result.caa == DnsAnswer::Absent {
        finding("dns.caa_missing");
    }
    if result.dnssec == DnsAnswer::Absent {
        finding("dns.dnssec_not_detected");
    }
    // SPF is only evaluated when mail is actually routed through this domain
    // (an MX record exists) — see module doc comment / #545 scope.
    if result.spf == DnsAnswer::Absent {
        finding("dns.spf_missing");
    }

    NetworkDnsAnalysis {
        host: host.to_string(),
        caa_present: result.caa.as_bool(),
        dnssec_detected: result.dnssec.as_bool(),
        mx_present: result.mx.as_bool(),
        spf_present: result.spf.as_bool(),
        findings,
    }
}

/// The host and its parent domains, nearest first, down to two labels.
fn names_up(host: &str) -> Vec<String> {
    let host = host.trim_end_matches('.');
    let labels: Vec<&str> = host.split('.').collect();
    (0..labels.len().saturating_sub(1))
        .map(|start| format!("{}.", labels[start..].join(".")))
        .collect()
}

/// The first name on the way up that has `record_type`. A name whose answer
/// is unknown stops the walk: what lies above cannot settle it.
async fn climb(lookup: &dyn DnsLookup, host: &str, record_type: RecordType) -> DnsAnswer {
    for name in names_up(host) {
        match lookup.lookup(&name, record_type).await {
            RecordLookup::Found(_) => return DnsAnswer::Present,
            RecordLookup::Absent => continue,
            RecordLookup::Unknown(reason) => return DnsAnswer::Unknown(reason),
        }
    }
    DnsAnswer::Absent
}

/// The DNS checks for one host against `lookup`.
pub async fn check_dns(lookup: &dyn DnsLookup, host: &str) -> DnsCheckResult {
    let fqdn = format!("{}.", host.trim_end_matches('.'));
    let caa = climb(lookup, host, RecordType::CAA).await;
    let dnssec = climb(lookup, host, RecordType::DNSKEY).await;
    let mx = match lookup.lookup(&fqdn, RecordType::MX).await {
        RecordLookup::Found(_) => DnsAnswer::Present,
        RecordLookup::Absent => DnsAnswer::Absent,
        RecordLookup::Unknown(reason) => DnsAnswer::Unknown(reason),
    };
    let spf = match mx {
        DnsAnswer::Present => match lookup.lookup(&fqdn, RecordType::TXT).await {
            RecordLookup::Found(texts) if texts.iter().any(|t| t.starts_with("v=spf1")) => {
                DnsAnswer::Present
            }
            RecordLookup::Found(_) | RecordLookup::Absent => DnsAnswer::Absent,
            RecordLookup::Unknown(reason) => DnsAnswer::Unknown(reason),
        },
        DnsAnswer::Absent => DnsAnswer::NotApplicable,
        // Without knowing whether mail is routed here, SPF is unknown too.
        DnsAnswer::NotApplicable | DnsAnswer::Unknown(_) => DnsAnswer::Unknown("mx_unknown"),
    };
    DnsCheckResult {
        caa,
        dnssec,
        mx,
        spf,
    }
}

/// What a host's DNS check holds when the resolver could not be constructed (a
/// local configuration problem): nothing was asked, so every answer is
/// unknown. Recorded as failed collections, never as absent records (#776).
pub(crate) fn resolver_unavailable() -> DnsCheckResult {
    const REASON: &str = "dns_resolver_unavailable";
    DnsCheckResult {
        caa: DnsAnswer::Unknown(REASON),
        dnssec: DnsAnswer::Unknown(REASON),
        mx: DnsAnswer::Unknown(REASON),
        spf: DnsAnswer::Unknown(REASON),
    }
}

/// Runs the real DNS queries for one host. A resolver that cannot be built
/// yields `resolver_unavailable`.
pub async fn query_dns(host: &str) -> DnsCheckResult {
    let resolver: TokioResolver = match TokioResolver::builder_tokio() {
        Ok(builder) => match builder.build() {
            Ok(resolver) => resolver,
            Err(e) => {
                warn!("Failed to build DNS resolver for {host}: {e}");
                return resolver_unavailable();
            }
        },
        Err(e) => {
            warn!("Failed to configure DNS resolver for {host}: {e}");
            return resolver_unavailable();
        }
    };
    check_dns(&HickoryLookup(resolver), host).await
}

/// Counter-cases of the DNS collectors (#776): an unknown answer is a failed
/// collection, a proven absence an empty one, SPF without MX not applicable.
#[cfg(test)]
pub(crate) fn counter_cases() {
    use crate::audit::collection::CollectionState;
    assert_eq!(
        DnsAnswer::Present.collected().state(),
        CollectionState::Measured
    );
    assert_eq!(
        DnsAnswer::Absent.collected().state(),
        CollectionState::Empty
    );
    assert_eq!(
        DnsAnswer::NotApplicable.collected().state(),
        CollectionState::NotApplicable
    );
    assert_eq!(
        DnsAnswer::Unknown("dns_timeout").collected().state(),
        CollectionState::Failed
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    use DnsAnswer::{Absent, NotApplicable, Present, Unknown};

    fn result(caa: DnsAnswer, dnssec: DnsAnswer, mx: DnsAnswer, spf: DnsAnswer) -> DnsCheckResult {
        DnsCheckResult {
            caa,
            dnssec,
            mx,
            spf,
        }
    }

    fn rule_ids(result: &DnsCheckResult) -> Vec<String> {
        build_analysis("example.com", result)
            .findings
            .into_iter()
            .map(|f| f.rule_id)
            .collect()
    }

    #[test]
    fn proven_absence_produces_findings() {
        assert!(rule_ids(&result(Present, Present, Present, Present)).is_empty());
        assert_eq!(
            rule_ids(&result(Absent, Absent, Present, Absent)),
            vec![
                "dns.caa_missing",
                "dns.dnssec_not_detected",
                "dns.spf_missing"
            ]
        );
        let no_mx = build_analysis(
            "example.com",
            &result(Present, Present, Absent, NotApplicable),
        );
        assert!(no_mx.findings.is_empty());
        assert_eq!(no_mx.spf_present, None);
    }

    #[test]
    fn unknown_answers_produce_no_findings() {
        // #776: a timeout or SERVFAIL is no missing record.
        let unknown = result(
            Unknown("dns_timeout"),
            Unknown("dns_lookup_failed"),
            Unknown("dns_timeout"),
            Unknown("mx_unknown"),
        );
        let analysis = build_analysis("example.com", &unknown);
        assert!(analysis.findings.is_empty());
        assert_eq!(analysis.caa_present, None);
        assert_eq!(analysis.dnssec_detected, None);
        assert_eq!(analysis.mx_present, None);
        assert_eq!(analysis.spf_present, None);
    }

    /// The mapping of the real resolver's errors (#776): only a proven
    /// negative answer is an absence; everything else stays unknown.
    #[test]
    fn an_unavailable_resolver_is_a_failed_collection_without_findings() {
        use crate::audit::collection::CollectionState;
        let unavailable = resolver_unavailable();
        for answer in [
            unavailable.caa,
            unavailable.dnssec,
            unavailable.mx,
            unavailable.spf,
        ] {
            assert_eq!(answer.collected().state(), CollectionState::Failed);
            assert_eq!(answer, Unknown("dns_resolver_unavailable"));
        }
        let analysis = build_analysis("example.com", &unavailable);
        assert!(analysis.findings.is_empty(), "unknown is not absence");
        assert_eq!(analysis.caa_present, None);
    }

    #[test]
    fn resolver_errors_map_to_absent_or_unknown() {
        use hickory_resolver::net::{DnsError, NetError, NoRecords};
        use hickory_resolver::proto::op::{Query, ResponseCode};
        let no_records = |code| {
            NetError::Dns(DnsError::NoRecordsFound(NoRecords::new(
                Query::default(),
                code,
            )))
        };
        assert!(matches!(
            lookup_error(&no_records(ResponseCode::NoError)),
            RecordLookup::Absent
        ));
        assert!(matches!(
            lookup_error(&no_records(ResponseCode::NXDomain)),
            RecordLookup::Absent
        ));
        assert!(matches!(
            lookup_error(&NetError::Timeout),
            RecordLookup::Unknown("dns_timeout")
        ));
        for failure in [
            NetError::Dns(DnsError::ResponseCode(ResponseCode::ServFail)),
            NetError::NoConnections,
            NetError::Message("resolver unavailable"),
        ] {
            assert!(
                matches!(
                    lookup_error(&failure),
                    RecordLookup::Unknown("dns_lookup_failed")
                ),
                "{failure}"
            );
        }
    }

    /// A scripted resolver: answers by (name, type), everything else NODATA.
    struct FakeDns(Vec<(&'static str, RecordType, RecordLookup)>);

    #[async_trait::async_trait]
    impl DnsLookup for FakeDns {
        async fn lookup(&self, name: &str, record_type: RecordType) -> RecordLookup {
            self.0
                .iter()
                .find(|(n, t, _)| *n == name && *t == record_type)
                .map(|(_, _, answer)| answer.clone())
                .unwrap_or(RecordLookup::Absent)
        }
    }

    fn check(fake: FakeDns, host: &str) -> DnsCheckResult {
        tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
            .block_on(check_dns(&fake, host))
    }

    fn found() -> RecordLookup {
        RecordLookup::Found(vec!["record".into()])
    }

    #[test]
    fn caa_and_dnskey_are_inherited_from_the_parent() {
        // Delegation: www has no records of its own, example.com has them.
        let result = check(
            FakeDns(vec![
                ("example.com.", RecordType::CAA, found()),
                ("example.com.", RecordType::DNSKEY, found()),
            ]),
            "www.example.com",
        );
        assert_eq!(result.caa, Present);
        assert_eq!(result.dnssec, Present);
    }

    #[test]
    fn nodata_and_nxdomain_are_proven_absence() {
        let result = check(
            FakeDns(vec![(
                "shop.example.com.",
                RecordType::MX,
                RecordLookup::Absent,
            )]),
            "shop.example.com",
        );
        assert_eq!(result.caa, Absent);
        assert_eq!(result.dnssec, Absent);
        assert_eq!(result.mx, Absent);
        assert_eq!(result.spf, NotApplicable);
    }

    #[test]
    fn servfail_and_timeout_stay_unknown() {
        let result = check(
            FakeDns(vec![
                // The host's own CAA answer is unknown: the parent cannot settle it.
                (
                    "www.example.com.",
                    RecordType::CAA,
                    RecordLookup::Unknown("dns_lookup_failed"),
                ),
                ("example.com.", RecordType::CAA, found()),
                (
                    "www.example.com.",
                    RecordType::DNSKEY,
                    RecordLookup::Unknown("dns_timeout"),
                ),
                (
                    "www.example.com.",
                    RecordType::MX,
                    RecordLookup::Unknown("dns_timeout"),
                ),
            ]),
            "www.example.com",
        );
        assert_eq!(result.caa, Unknown("dns_lookup_failed"));
        assert_eq!(result.dnssec, Unknown("dns_timeout"));
        assert_eq!(result.mx, Unknown("dns_timeout"));
        // MX unknown: whether SPF applies is unknown too, not "no mail".
        assert_eq!(result.spf, Unknown("mx_unknown"));
    }

    #[test]
    fn spf_needs_a_v_spf1_text_record() {
        let mx_and_txt = |txt: RecordLookup| {
            check(
                FakeDns(vec![
                    ("example.com.", RecordType::MX, found()),
                    ("example.com.", RecordType::TXT, txt),
                ]),
                "example.com",
            )
            .spf
        };
        assert_eq!(
            mx_and_txt(RecordLookup::Found(vec!["v=spf1 -all".into()])),
            Present
        );
        assert_eq!(
            mx_and_txt(RecordLookup::Found(vec![
                "google-site-verification=x".into()
            ])),
            Absent
        );
        assert_eq!(
            mx_and_txt(RecordLookup::Unknown("dns_timeout")),
            Unknown("dns_timeout")
        );
    }

    #[test]
    fn english_message_text_has_no_known_german_leaks() {
        // #406 guard: canonical (en=true) message text must be real English,
        // never a German sentence with the umlaut/ß left in.
        let has_umlaut = |s: &str| s.chars().any(|c| "äöüÄÖÜß".contains(c));
        for rule_id in [
            "dns.caa_missing",
            "dns.dnssec_not_detected",
            "dns.spf_missing",
        ] {
            let msg = finding_message_text(rule_id, true);
            assert!(!msg.is_empty(), "{rule_id} must produce a message");
            assert!(
                !has_umlaut(&msg),
                "{rule_id} EN message leaks German: {msg}"
            );
        }
    }

    #[test]
    fn german_message_text_differs_from_english() {
        for rule_id in [
            "dns.caa_missing",
            "dns.dnssec_not_detected",
            "dns.spf_missing",
        ] {
            let en = finding_message_text(rule_id, true);
            let de = finding_message_text(rule_id, false);
            assert_ne!(en, de, "{rule_id} must have a distinct German translation");
        }
    }
}
