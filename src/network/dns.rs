//! DNS configuration audit (#545): opt-in, host-scoped checks for CAA,
//! DNSSEC, and SPF/MX records via direct DNS queries. Pure network/DNS —
//! no CDP/browser page is involved, and no `AuditReport`/score/certificate
//! field is ever touched by these findings (same category as
//! `design_quality` #528 and `ai_transparency`: opt-in and score-neutral).
//!
//! Runs once per host, not once per page — see [`NetworkDnsModule`]'s
//! per-host memoization.
//!
//! Known limitations (documented rather than silently assumed away, per
//! #545's explicit scope):
//! - **CAA**: queries the exact hostname only. RFC 6844 requires walking up
//!   to parent domains until a CAA record (or the zone apex) is found; a
//!   subdomain with no CAA record of its own but one on a parent is reported
//!   here as "missing" even though CAA policy does apply via the ancestor.
//!   Acceptable for a first cut.
//! - **DNSSEC**: a best-effort "does a DNSKEY record exist at this name"
//!   check, not full chain-of-trust validation — there is no verification
//!   that the DNSKEY is signed by a trusted parent zone, no RRSIG check. A
//!   "detected" result means DNSSEC signing looks present, not that a
//!   resolver has cryptographically validated the chain.
//! - **Query failures**: NXDOMAIN/NODATA establish absence; transport or
//!   resolver failures leave the module unmeasured instead of inventing a
//!   missing-record finding.

pub mod module;
pub use module::NetworkDnsModule;

use hickory_resolver::proto::rr::{RData, RecordType};
use hickory_resolver::TokioResolver;
use serde::{Deserialize, Serialize};
use tracing::warn;

/// Raw DNS query results for one host — pure data, no message text baked in.
/// Kept separate from finding construction ([`build_analysis`]) so the
/// finding-mapping logic can be unit-tested without a real DNS query.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DnsCheckResult {
    pub caa_present: bool,
    pub dnssec_detected: bool,
    pub mx_present: bool,
    /// Only meaningful when `mx_present` is true — see [`build_analysis`].
    pub spf_present: bool,
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
    pub caa_present: bool,
    pub dnssec_detected: bool,
    pub mx_present: bool,
    /// `None` when there is no MX record — the SPF check is skipped
    /// entirely per #545 ("only when mail is routed through this domain"),
    /// not reported as pass or fail.
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

    if !result.caa_present {
        findings.push(DnsFinding {
            rule_id: "dns.caa_missing".to_string(),
            message: finding_message_text("dns.caa_missing", true),
        });
    }

    if !result.dnssec_detected {
        findings.push(DnsFinding {
            rule_id: "dns.dnssec_not_detected".to_string(),
            message: finding_message_text("dns.dnssec_not_detected", true),
        });
    }

    // SPF is only evaluated when mail is actually routed through this domain
    // (an MX record exists) — see module doc comment / #545 scope.
    let spf_present = if result.mx_present {
        if !result.spf_present {
            findings.push(DnsFinding {
                rule_id: "dns.spf_missing".to_string(),
                message: finding_message_text("dns.spf_missing", true),
            });
        }
        Some(result.spf_present)
    } else {
        None
    };

    NetworkDnsAnalysis {
        host: host.to_string(),
        caa_present: result.caa_present,
        dnssec_detected: result.dnssec_detected,
        mx_present: result.mx_present,
        spf_present,
        findings,
    }
}

/// Runs the real DNS queries for one host. Returns `None` when a lookup
/// fails technically or the resolver could not be constructed (a local configuration/
/// environment problem) — an individual record lookup failing (including
/// "no records found") is treated as "absent", see module doc comment.
pub async fn query_dns(host: &str) -> Option<DnsCheckResult> {
    let resolver: TokioResolver = match TokioResolver::builder_tokio() {
        Ok(builder) => match builder.build() {
            Ok(resolver) => resolver,
            Err(e) => {
                warn!("Failed to build DNS resolver for {host}: {e}");
                return None;
            }
        },
        Err(e) => {
            warn!("Failed to configure DNS resolver for {host}: {e}");
            return None;
        }
    };

    // Fully-qualified (trailing dot) avoids the resolver silently applying
    // search-domain suffixes configured on the host machine.
    let fqdn = if host.ends_with('.') {
        host.to_string()
    } else {
        format!("{host}.")
    };

    let caa_present = resolver
        .lookup(fqdn.as_str(), RecordType::CAA)
        .await
        .map(|l| !l.answers().is_empty())
        .or_else(|error| {
            if error.is_no_records_found() {
                Ok(false)
            } else {
                Err(error)
            }
        })
        .ok()?;

    let dnssec_detected = resolver
        .lookup(fqdn.as_str(), RecordType::DNSKEY)
        .await
        .map(|l| !l.answers().is_empty())
        .or_else(|error| {
            if error.is_no_records_found() {
                Ok(false)
            } else {
                Err(error)
            }
        })
        .ok()?;

    let mx_present = resolver
        .mx_lookup(fqdn.as_str())
        .await
        .map(|l| !l.answers().is_empty())
        .or_else(|error| {
            if error.is_no_records_found() {
                Ok(false)
            } else {
                Err(error)
            }
        })
        .ok()?;

    let spf_present = if mx_present {
        resolver
            .txt_lookup(fqdn.as_str())
            .await
            .map(|l| {
                l.answers().iter().any(|record| {
                    matches!(&record.data, RData::TXT(txt) if txt.to_string().starts_with("v=spf1"))
                })
            })
            .or_else(|error| {
                if error.is_no_records_found() {
                    Ok(false)
                } else {
                    Err(error)
                }
            })
            .ok()?
    } else {
        false
    };

    Some(DnsCheckResult {
        caa_present,
        dnssec_detected,
        mx_present,
        spf_present,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_present_produces_no_findings() {
        let result = DnsCheckResult {
            caa_present: true,
            dnssec_detected: true,
            mx_present: true,
            spf_present: true,
        };
        let analysis = build_analysis("example.com", &result);
        assert!(analysis.findings.is_empty());
        assert_eq!(analysis.spf_present, Some(true));
    }

    #[test]
    fn caa_missing_produces_finding() {
        let result = DnsCheckResult {
            caa_present: false,
            dnssec_detected: true,
            mx_present: false,
            spf_present: false,
        };
        let analysis = build_analysis("example.com", &result);
        assert_eq!(analysis.findings.len(), 1);
        assert_eq!(analysis.findings[0].rule_id, "dns.caa_missing");
    }

    #[test]
    fn dnssec_not_detected_produces_finding() {
        let result = DnsCheckResult {
            caa_present: true,
            dnssec_detected: false,
            mx_present: false,
            spf_present: false,
        };
        let analysis = build_analysis("example.com", &result);
        assert_eq!(analysis.findings.len(), 1);
        assert_eq!(analysis.findings[0].rule_id, "dns.dnssec_not_detected");
    }

    #[test]
    fn spf_missing_only_fires_when_mx_present() {
        let no_mx = DnsCheckResult {
            caa_present: true,
            dnssec_detected: true,
            mx_present: false,
            spf_present: false,
        };
        let analysis = build_analysis("example.com", &no_mx);
        assert!(analysis.findings.is_empty());
        assert_eq!(analysis.spf_present, None);

        let mx_without_spf = DnsCheckResult {
            caa_present: true,
            dnssec_detected: true,
            mx_present: true,
            spf_present: false,
        };
        let analysis = build_analysis("example.com", &mx_without_spf);
        assert_eq!(analysis.findings.len(), 1);
        assert_eq!(analysis.findings[0].rule_id, "dns.spf_missing");
        assert_eq!(analysis.spf_present, Some(false));
    }

    #[test]
    fn mx_with_spf_produces_no_spf_finding() {
        let result = DnsCheckResult {
            caa_present: true,
            dnssec_detected: true,
            mx_present: true,
            spf_present: true,
        };
        let analysis = build_analysis("example.com", &result);
        assert!(analysis.findings.is_empty());
        assert_eq!(analysis.spf_present, Some(true));
    }

    #[test]
    fn all_missing_produces_three_findings() {
        let result = DnsCheckResult {
            caa_present: false,
            dnssec_detected: false,
            mx_present: true,
            spf_present: false,
        };
        let analysis = build_analysis("example.com", &result);
        assert_eq!(analysis.findings.len(), 3);
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
