//! C2PA manifest reading for images found on the audited page.
//!
//! Compiled only under the `ai-transparency` Cargo feature (pulls in the
//! `c2pa` crate's real crypto dependencies). Two things make this the first
//! fetch in this codebase that needs its own hardening beyond the same-origin
//! `reqwest` calls in `seo::robots`/`security`:
//!
//! 1. It fetches **arbitrary third-party origins** — image URLs come from the
//!    audited page's own markup, which may be attacker-controlled. Every
//!    fetch resolves DNS itself, rejects non-public IPs (private/loopback/
//!    link-local/multicast), and pins the connection to the exact address it
//!    validated (`reqwest::ClientBuilder::resolve`) so a second DNS lookup
//!    during the actual connect can't return a different, unvalidated
//!    address (DNS-rebinding TOCTOU).
//! 2. It reads a manifest structure embedded in untrusted bytes, so failures
//!    (bad format, oversized response, no manifest present) are all expected,
//!    non-fatal outcomes — never propagated as a page-audit failure.
//!
//! `c2pa` itself is configured (see `Cargo.toml`) with `default-features =
//! false, features = ["rust_native_crypto"]` — no `openssl`, no HTTP client
//! features — so `Reader::with_stream` cannot itself make a network call
//! (e.g. OCSP/trust-list/remote-manifest fetches); it only reads the bytes we
//! already fetched.

use std::net::{IpAddr, SocketAddr};
use std::time::Duration;

use chromiumoxide::Page;
use reqwest::Client;
use tracing::debug;

use c2pa::assertions::{labels, Actions, SoftwareAgent};
use c2pa::{format_from_path, Context, DigitalSourceType, Reader, ValidationState};

use crate::error::{AuditError, Result};

use super::{AiProvenanceKind, AiTransparencyAnalysis, ImageProvenanceFinding, ManifestValidation};

const RULE_ID: &str = "ai_transparency.c2pa_provenance";
/// Upper bound on distinct images fetched/parsed per page — this is a
/// per-page network+CPU cost, not a free heuristic over already-collected
/// data, so it stays intentionally small (mirrors the spirit of
/// `MAX_SAMPLE_TASKS`/`MAX_ELEMENT_CROPS`-style caps elsewhere).
const MAX_IMAGES_CHECKED: usize = 15;
/// Skip (and abort mid-download of) anything larger than this — a manifest
/// lives in the first few KB of a typical asset; there is no reason to pull
/// down a multi-hundred-MB file to look for one.
const MAX_IMAGE_FETCH_BYTES: u64 = 8_000_000;
const FETCH_TIMEOUT_SECS: u64 = 8;

const EXTRACT_JS: &str = include_str!("extract.js");

#[derive(Debug, Default, serde::Deserialize)]
struct RawImageUrls {
    #[serde(default, rename = "imageUrls")]
    image_urls: Vec<String>,
}

/// Run the ai-transparency analysis on the already-loaded page: collect
/// `<img>` source URLs, fetch each (bounded, hardened), and check for a C2PA
/// manifest asserting AI/algorithmic generation.
pub(super) async fn analyze_ai_transparency(page: &Page) -> Result<AiTransparencyAnalysis> {
    let js_code = format!("(() => {{ {EXTRACT_JS} }})();");
    let eval_result = page.evaluate(js_code.as_str()).await.map_err(|e| {
        AuditError::CdpError(format!("AI transparency image extraction failed: {e}"))
    })?;

    let raw: RawImageUrls = match eval_result.value() {
        Some(value) => serde_json::from_value(value.clone()).unwrap_or_default(),
        None => RawImageUrls::default(),
    };

    let mut findings = Vec::new();
    let mut images_checked = 0usize;

    for url in raw.image_urls.iter().take(MAX_IMAGES_CHECKED) {
        let Some((bytes, content_type)) = safe_fetch_image(url).await else {
            continue;
        };
        images_checked += 1;
        if let Some(finding) = inspect_image_manifest(url, &content_type, bytes) {
            findings.push(finding);
        }
    }

    Ok(AiTransparencyAnalysis {
        findings,
        images_checked,
    })
}

/// Fetch `url`'s bytes with SSRF hardening. Returns `None` on any failure
/// (bad scheme, unresolvable/private host, oversized body, network error) —
/// a single unreachable/malformed image must never fail the whole page audit.
async fn safe_fetch_image(url: &str) -> Option<(Vec<u8>, String)> {
    let parsed = url::Url::parse(url).ok()?;
    if parsed.scheme() != "http" && parsed.scheme() != "https" {
        return None;
    }
    let host = parsed.host_str()?.to_string();
    let port = parsed.port_or_known_default()?;

    let mut addrs = tokio::net::lookup_host((host.as_str(), port)).await.ok()?;
    let safe_addr: SocketAddr = addrs.find(|a| is_public_ip(&a.ip()))?;

    let client = Client::builder()
        .timeout(Duration::from_secs(FETCH_TIMEOUT_SECS))
        .user_agent("auditmysite/1.0")
        .resolve(&host, safe_addr)
        .build()
        .ok()?;

    let resp = client.get(url).send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    if let Some(len) = resp.content_length() {
        if len > MAX_IMAGE_FETCH_BYTES {
            debug!("Skipping oversized image (Content-Length {len}): {url}");
            return None;
        }
    }
    let content_type = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_string();

    let bytes = resp.bytes().await.ok()?;
    if bytes.len() as u64 > MAX_IMAGE_FETCH_BYTES {
        debug!(
            "Skipping oversized image (actual {} bytes): {url}",
            bytes.len()
        );
        return None;
    }
    Some((bytes.to_vec(), content_type))
}

/// Rejects loopback/private/link-local/multicast/unspecified addresses,
/// including the IPv4-mapped-into-IPv6 form. Explicit range checks rather
/// than newer `Ipv6Addr` helper methods, some of which stabilized recently
/// enough that this project's toolchain may not have them.
fn is_public_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            !(v4.is_private()
                || v4.is_loopback()
                || v4.is_link_local()
                || v4.is_multicast()
                || v4.is_broadcast()
                || v4.is_documentation()
                || v4.is_unspecified())
        }
        IpAddr::V6(v6) => {
            if v6.is_loopback() || v6.is_multicast() || v6.is_unspecified() {
                return false;
            }
            if let Some(v4) = v6.to_ipv4_mapped() {
                return is_public_ip(&IpAddr::V4(v4));
            }
            let seg0 = v6.segments()[0];
            let is_unique_local = seg0 & 0xfe00 == 0xfc00; // fc00::/7
            let is_link_local = seg0 & 0xffc0 == 0xfe80; // fe80::/10
            !(is_unique_local || is_link_local)
        }
    }
}

/// Parse `bytes` as a C2PA-bearing asset and, if the active manifest's
/// `c2pa.actions` assertion asserts AI/algorithmic generation, build a
/// finding. Returns `None` for any non-fatal outcome (no manifest, no
/// actions assertion, no AI-indicating source type, parse failure).
fn inspect_image_manifest(
    image_url: &str,
    content_type: &str,
    bytes: Vec<u8>,
) -> Option<ImageProvenanceFinding> {
    let format = if content_type.is_empty() {
        // `format_from_path` infers from the file extension, so use just the
        // URL's path component — a query string like `?w=800` would otherwise
        // get swallowed into a bogus "extension".
        url::Url::parse(image_url)
            .ok()
            .and_then(|u| format_from_path(u.path()))
            .unwrap_or_default()
    } else {
        content_type.to_string()
    };
    if format.is_empty() {
        return None;
    }

    let cursor = std::io::Cursor::new(bytes);
    let reader = Reader::from_context(Context::new())
        .with_stream(&format, cursor)
        .ok()?;

    let manifest = reader.active_manifest()?;
    let actions: Actions = manifest.find_assertion(labels::ACTIONS).ok()?;

    let (provenance, generator) = actions.actions().iter().find_map(|action| {
        let provenance = map_source_type(action.source_type()?)?;
        let generator = action
            .software_agent()
            .and_then(software_agent_name)
            .or_else(|| manifest.claim_generator().map(str::to_string));
        Some((provenance, generator))
    })?;

    let validation = map_validation(reader.validation_state());

    Some(ImageProvenanceFinding {
        rule_id: RULE_ID.to_string(),
        image_url: image_url.to_string(),
        provenance,
        validation,
        generator: generator.clone(),
        measurement_type: "c2pa_manifest".to_string(),
        evidence: format!(
            "C2PA manifest asserts digitalSourceType '{}'{}, manifest validation: {}.",
            provenance.as_str(),
            generator
                .as_ref()
                .map(|g| format!(" (generator: {g})"))
                .unwrap_or_default(),
            validation.as_str(),
        ),
        message: super::finding_message_text(RULE_ID, true),
    })
}

fn map_source_type(dst: &DigitalSourceType) -> Option<AiProvenanceKind> {
    match dst {
        DigitalSourceType::TrainedAlgorithmicMedia => {
            Some(AiProvenanceKind::TrainedAlgorithmicMedia)
        }
        DigitalSourceType::CompositeWithTrainedAlgorithmicMedia => {
            Some(AiProvenanceKind::CompositeWithTrainedAlgorithmicMedia)
        }
        DigitalSourceType::CompositeSynthetic => Some(AiProvenanceKind::CompositeSynthetic),
        DigitalSourceType::VirtualRecording => Some(AiProvenanceKind::VirtualRecording),
        DigitalSourceType::TrainedAlgorithmicData => Some(AiProvenanceKind::TrainedAlgorithmicData),
        _ => None,
    }
}

fn map_validation(state: ValidationState) -> ManifestValidation {
    match state {
        ValidationState::Invalid => ManifestValidation::Invalid,
        ValidationState::Valid => ManifestValidation::Valid,
        ValidationState::Trusted => ManifestValidation::Trusted,
    }
}

fn software_agent_name(agent: &SoftwareAgent) -> Option<String> {
    match agent {
        SoftwareAgent::String(s) => Some(s.clone()),
        SoftwareAgent::ClaimGeneratorInfo(info) => Some(match &info.version {
            Some(v) => format!("{} {}", info.name, v),
            None => info.name.clone(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Minimal valid 1x1 PNG (no C2PA manifest) — just enough format bytes
    /// for a blank pixel, not a copyrightable asset. Used as the base image
    /// `Builder::sign` embeds a manifest into, and as the "no manifest"
    /// negative-case fixture in its own right.
    const BLANK_PNG: &[u8] = &[
        0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x04, 0x00, 0x00, 0x00, 0xb5,
        0x1c, 0x0c, 0x02, 0x00, 0x00, 0x00, 0x0b, 0x49, 0x44, 0x41, 0x54, 0x78, 0xda, 0x63, 0x64,
        0xf8, 0x0f, 0x00, 0x01, 0x05, 0x01, 0x01, 0x27, 0x18, 0xe3, 0x66, 0x00, 0x00, 0x00, 0x00,
        0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
    ];

    /// Signs `BLANK_PNG` with a synthetic C2PA manifest asserting AI
    /// generation (`DigitalSourceType::TrainedAlgorithmicMedia`), using an
    /// ephemeral, non-trust-chained certificate (`EphemeralSigner` — test/
    /// local use only, never validates as `Trusted`, only `Valid`). No
    /// external test asset or vendored fixture file needed.
    fn ai_generated_png_bytes() -> Vec<u8> {
        use c2pa::{Builder, EphemeralSigner};

        let mut builder = Builder::default();
        builder
            .add_action(serde_json::json!({
                "action": "c2pa.created",
                "digitalSourceType": "http://cv.iptc.org/newscodes/digitalsourcetype/trainedAlgorithmicMedia",
                "softwareAgent": "Test Generative Tool 1.0",
            }))
            .expect("add_action");

        let signer = EphemeralSigner::new("ai-transparency-test.local").expect("ephemeral signer");
        let mut source = std::io::Cursor::new(BLANK_PNG.to_vec());
        let mut dest = std::io::Cursor::new(Vec::new());
        builder
            .sign(&signer, "image/png", &mut source, &mut dest)
            .expect("sign");
        dest.into_inner()
    }

    #[test]
    fn inspects_ai_generated_image_and_finds_provenance() {
        let bytes = ai_generated_png_bytes();
        let finding = inspect_image_manifest("https://example.com/hero.png", "image/png", bytes)
            .expect("finding expected for AI-generated image");

        assert_eq!(
            finding.provenance,
            AiProvenanceKind::TrainedAlgorithmicMedia
        );
        assert_eq!(
            finding.generator.as_deref(),
            Some("Test Generative Tool 1.0")
        );
        // EphemeralSigner's certs are self-signed/untrusted, never chaining to
        // a trust list — the manifest is well-formed and signed, so `Valid`,
        // never `Trusted`.
        assert_eq!(finding.validation, ManifestValidation::Valid);
        assert!(finding.evidence.contains("trained_algorithmic_media"));
    }

    #[test]
    fn plain_image_without_manifest_yields_no_finding() {
        let finding = inspect_image_manifest(
            "https://example.com/plain.png",
            "image/png",
            BLANK_PNG.to_vec(),
        );
        assert!(finding.is_none());
    }

    #[test]
    fn public_ip_v4_accepted() {
        assert!(is_public_ip(&"93.184.216.34".parse().unwrap()));
    }

    #[test]
    fn private_ip_v4_rejected() {
        for ip in [
            "10.0.0.1",
            "172.16.0.1",
            "192.168.1.1",
            "127.0.0.1",
            "169.254.1.1",
        ] {
            assert!(!is_public_ip(&ip.parse().unwrap()), "{ip} must be rejected");
        }
    }

    #[test]
    fn loopback_and_unique_local_ipv6_rejected() {
        for ip in ["::1", "fc00::1", "fd12:3456:789a::1", "fe80::1"] {
            assert!(!is_public_ip(&ip.parse().unwrap()), "{ip} must be rejected");
        }
    }

    #[test]
    fn ipv4_mapped_private_ipv6_rejected() {
        assert!(!is_public_ip(&"::ffff:127.0.0.1".parse().unwrap()));
    }

    #[test]
    fn public_ipv6_accepted() {
        assert!(is_public_ip(&"2606:4700:4700::1111".parse().unwrap()));
    }
}
