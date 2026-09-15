//! Security headers analysis module
//!
//! Analyzes HTTP security headers and SSL/TLS configuration.

pub mod module;
mod sourcemap;
pub use module::SecurityModule;
pub use sourcemap::{audit_source_maps, SourceMapLeak, SourceMapLeakAudit};

use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tracing::info;

use crate::error::{AuditError, Result};
use crate::taxonomy::Severity;

/// A CDN, WAF, or hosting service detected from response headers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedProtection {
    pub name: String,
    /// Human-readable category, e.g. "CDN + WAF"
    pub kind: String,
    pub is_waf: bool,
    pub is_cdn: bool,
}

/// CDN/WAF protection fingerprinted from response headers
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProtectionDetection {
    pub services: Vec<DetectedProtection>,
    pub has_waf: bool,
    pub has_cdn: bool,
}

/// Security analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAnalysis {
    /// Overall security score (0-100)
    pub score: u32,
    /// Security grade (A+ to F)
    pub grade: String,
    /// Security headers present
    pub headers: SecurityHeaders,
    /// SSL/TLS information
    pub ssl: SslInfo,
    /// Issues found
    pub issues: Vec<SecurityIssue>,
    /// Recommendations
    pub recommendations: Vec<String>,
    /// Detected CDN/WAF/hosting protection
    pub protection: ProtectionDetection,
    /// Publicly reachable source maps for loaded scripts/stylesheets (#538)
    #[serde(default)]
    pub sourcemap_leaks: SourceMapLeakAudit,
}

/// Security headers status
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SecurityHeaders {
    /// Content-Security-Policy
    pub content_security_policy: Option<String>,
    /// X-Content-Type-Options
    pub x_content_type_options: Option<String>,
    /// X-Frame-Options
    pub x_frame_options: Option<String>,
    /// Referrer-Policy
    pub referrer_policy: Option<String>,
    /// Permissions-Policy
    pub permissions_policy: Option<String>,
    /// Strict-Transport-Security (HSTS)
    pub strict_transport_security: Option<String>,
    /// Cross-Origin-Opener-Policy
    pub cross_origin_opener_policy: Option<String>,
    /// Cross-Origin-Resource-Policy
    pub cross_origin_resource_policy: Option<String>,
    /// Access-Control-Allow-Origin
    pub access_control_allow_origin: Option<String>,
    /// Access-Control-Allow-Credentials
    pub access_control_allow_credentials: Option<String>,
}

impl SecurityHeaders {
    /// Count how many security headers are present
    pub fn count(&self) -> usize {
        [
            self.content_security_policy.is_some(),
            self.x_content_type_options.is_some(),
            self.x_frame_options.is_some(),
            self.referrer_policy.is_some(),
            self.permissions_policy.is_some(),
            self.strict_transport_security.is_some(),
            self.cross_origin_opener_policy.is_some(),
            self.cross_origin_resource_policy.is_some(),
            self.access_control_allow_origin.is_some(),
            self.access_control_allow_credentials.is_some(),
        ]
        .iter()
        .filter(|&&x| x)
        .count()
    }
}

/// SSL/TLS information
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SslInfo {
    /// Uses HTTPS
    pub https: bool,
    /// Has valid certificate (basic check)
    pub valid_certificate: bool,
    /// Has HSTS
    pub has_hsts: bool,
    /// HSTS max-age value
    pub hsts_max_age: Option<u64>,
    /// HSTS includes subdomains
    pub hsts_include_subdomains: bool,
    /// HSTS preload
    pub hsts_preload: bool,
    /// TLS protocol reported by the browser (e.g. "TLS 1.3").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol: Option<String>,
    /// TLS cipher reported by the browser.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cipher: Option<String>,
    /// Certificate subject name reported by the browser.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub certificate_subject: Option<String>,
    /// Certificate issuer reported by the browser.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub certificate_issuer: Option<String>,
    /// Certificate expiration timestamp (seconds since Unix epoch).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub certificate_valid_to: Option<i64>,
    /// Days until certificate expiration, based on collection time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub certificate_expires_in_days: Option<i64>,
    /// Number of certificates in the chain exposed by CDP.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub certificate_chain_length: Option<usize>,
    /// Browser-reported certificate network error, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub certificate_error: Option<String>,
    /// Whether the browser reported weak certificate signatures.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub certificate_has_weak_signature: Option<bool>,
    /// Whether the browser reported SHA-1 signatures in the chain.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub certificate_has_sha1_signature: Option<bool>,
}

/// Browser-observed certificate details from the CDP Security domain.
#[derive(Debug, Clone, Default)]
pub struct BrowserCertificateDetails {
    pub protocol: Option<String>,
    pub cipher: Option<String>,
    pub subject: Option<String>,
    pub issuer: Option<String>,
    pub valid_to: Option<i64>,
    pub expires_in_days: Option<i64>,
    pub chain_length: Option<usize>,
    pub error: Option<String>,
    pub has_weak_signature: Option<bool>,
    pub has_sha1_signature: Option<bool>,
}

impl SslInfo {
    pub fn apply_certificate_details(&mut self, details: BrowserCertificateDetails) {
        self.protocol = details.protocol;
        self.cipher = details.cipher;
        self.certificate_subject = details.subject;
        self.certificate_issuer = details.issuer;
        self.certificate_valid_to = details.valid_to;
        self.certificate_expires_in_days = details.expires_in_days;
        self.certificate_chain_length = details.chain_length;
        self.certificate_error = details.error;
        self.certificate_has_weak_signature = details.has_weak_signature;
        self.certificate_has_sha1_signature = details.has_sha1_signature;
        if self.certificate_error.is_some() {
            self.valid_certificate = false;
        }
    }
}

/// Classification tier for a security header (#578).
///
/// A raw "N headers missing" count treats every header as equally urgent,
/// which isn't true: some are baseline hygiene for virtually every site,
/// some only matter for a specific architecture, some depend on a
/// deployment context this tool cannot observe from a single automated page
/// fetch, and some can't be meaningfully judged as present/absent at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HeaderTier {
    /// Applies to virtually every site; missing it is a real gap regardless
    /// of architecture (CSP, HSTS, X-Content-Type-Options, X-Frame-Options,
    /// HTTPS itself).
    Baseline,
    /// Good practice, but whether it's expected depends on the site's
    /// architecture (Referrer-Policy, Permissions-Policy).
    ArchitectureDependent,
    /// Relevance depends on a deployment context (e.g. SharedArrayBuffer
    /// usage, cross-origin isolation) that can't be determined from a
    /// single automated page fetch (Cross-Origin-Opener/Resource-Policy).
    ContextDependent,
    /// Presence or absence alone isn't a meaningful signal — a static crawl
    /// can't tell whether cross-origin sharing is intentional (CORS
    /// headers).
    NotAssessable,
}

/// Localized presentation text for the COOP/CORP context-verification
/// messages introduced in #578 (#406 kind-enum pattern: `SecurityIssue.message`
/// stays canonical English for JSON; the PDF layer derives the run-locale
/// text via this function instead of rendering `.message` directly — those
/// two messages became long explanatory paragraphs in #578, so leaving them
/// unlocalized would leak substantial English prose into German reports).
/// Returns `None` for any other `(header, issue_type)` pair — callers should
/// fall back to `SecurityIssue.message` for those, matching this codebase's
/// existing (documented, out-of-scope-to-fully-fix-here) behavior for the
/// rest of the security issue catalog.
pub fn coop_corp_verification_text(
    header: &str,
    issue_type: &str,
    en: bool,
) -> Option<&'static str> {
    if issue_type != "missing_header" {
        return None;
    }
    match header {
        "Cross-Origin-Opener-Policy" => Some(if en {
            "Cross-Origin-Opener-Policy is not set. This header only matters if the page uses \
             SharedArrayBuffer, high-resolution timers, or needs to isolate itself from \
             cross-origin popups — if none of that applies, no action is needed here. If it \
             does apply, set it to same-origin and verify popup/window interactions still work \
             as expected."
        } else {
            "Cross-Origin-Opener-Policy ist nicht gesetzt. Dieser Header ist nur relevant, wenn \
             die Seite SharedArrayBuffer, hochauflösende Timer verwendet oder sich von \
             Cross-Origin-Popups isolieren muss — trifft das nicht zu, ist hier keine Maßnahme \
             nötig. Trifft es zu: auf same-origin setzen und prüfen, ob Popup-/Fenster-\
             Interaktionen weiterhin wie erwartet funktionieren."
        }),
        "Cross-Origin-Resource-Policy" => Some(if en {
            "Cross-Origin-Resource-Policy is not set. This header only matters if the page \
             serves fonts, scripts, or media that other origins should be prevented from \
             loading — if the site's resources are intentionally public, no action is needed \
             here. If cross-origin loading should be restricted, set it to same-origin or \
             same-site."
        } else {
            "Cross-Origin-Resource-Policy ist nicht gesetzt. Dieser Header ist nur relevant, \
             wenn die Seite Schriften, Skripte oder Medien ausliefert, die andere Origins nicht \
             laden können sollen — sind die Ressourcen bewusst öffentlich zugänglich, ist hier \
             keine Maßnahme nötig. Soll das Laden von anderen Origins eingeschränkt werden: auf \
             same-origin oder same-site setzen."
        }),
        _ => None,
    }
}

/// Classifies a security header by how universally applicable it is (#578).
/// Falls back to `Baseline` for anything outside the fixed table, which
/// covers confirmed misconfigurations (e.g. CSP quality issues, public
/// source maps) — those are never ambiguous by nature, so they read as
/// baseline findings rather than context-dependent ones.
pub fn header_tier(header: &str) -> HeaderTier {
    match header {
        "Content-Security-Policy"
        | "Strict-Transport-Security"
        | "X-Content-Type-Options"
        | "X-Frame-Options"
        | "HTTPS" => HeaderTier::Baseline,
        "Referrer-Policy" | "Permissions-Policy" => HeaderTier::ArchitectureDependent,
        "Cross-Origin-Opener-Policy" | "Cross-Origin-Resource-Policy" => {
            HeaderTier::ContextDependent
        }
        "Access-Control-Allow-Origin" | "Access-Control-Allow-Credentials" => {
            HeaderTier::NotAssessable
        }
        _ => HeaderTier::Baseline,
    }
}

/// Security issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityIssue {
    pub header: String,
    pub issue_type: String,
    pub message: String,
    pub severity: Severity,
    /// Classification tier for the underlying header (#578) — lets report
    /// consumers distinguish baseline hygiene from context-dependent
    /// findings without re-deriving it from `header`.
    pub tier: HeaderTier,
}

/// Analyze security headers of a URL
pub async fn analyze_security(url: &str) -> Result<SecurityAnalysis> {
    info!("Analyzing security headers for {}...", url);

    let https = url.starts_with("https://");

    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(false)
        .build()
        .map_err(AuditError::HttpError)?;

    let response = match client.head(url).send().await {
        Ok(response) => Ok(response),
        Err(_) => client.get(url).send().await,
    };

    let (headers, protection) = match response {
        Ok(response) => {
            let raw = response.headers();
            let h = extract_security_headers(raw);
            let p = detect_protection(raw);
            (h, p)
        }
        Err(err) => {
            info!(
                "Security header request failed for {}; continuing with URL-only security analysis: {}",
                url, err
            );
            (SecurityHeaders::default(), ProtectionDetection::default())
        }
    };

    // Analyze SSL
    let ssl = analyze_ssl(https, &headers);

    // Generate issues
    let mut issues = generate_security_issues(&headers, https);

    // HSTS preload eligibility + Permissions-Policy quality (#535)
    if hsts_preload_ineligible(&ssl) {
        issues.push(SecurityIssue {
            header: "Strict-Transport-Security".to_string(),
            issue_type: "hsts_preload_ineligible".to_string(),
            message: format!(
                "HSTS preload directive is set but requirements aren't met (max-age={}, includeSubDomains={}) — the site won't qualify for the HSTS preload list",
                ssl.hsts_max_age
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "unset".to_string()),
                ssl.hsts_include_subdomains
            ),
            severity: Severity::Low,
            tier: HeaderTier::Baseline,
        });
    }
    if permissions_policy_is_permissive(&headers) {
        issues.push(SecurityIssue {
            header: "Permissions-Policy".to_string(),
            issue_type: "permissions_policy_permissive".to_string(),
            message: "Permissions-Policy header is present but doesn't restrict any feature (empty or wildcard-only)".to_string(),
            severity: Severity::Low,
            tier: HeaderTier::ArchitectureDependent,
        });
    }

    // Public source-map leak check (#538)
    let sourcemap_leaks = audit_source_maps(url).await;
    if !sourcemap_leaks.leaks.is_empty() {
        issues.push(SecurityIssue {
            header: "Source Map".to_string(),
            issue_type: "public_source_map".to_string(),
            message: format!(
                "{} publicly reachable source map{} found (e.g. {}) — exposes original source, comments, and internal file paths",
                sourcemap_leaks.leaks.len(),
                if sourcemap_leaks.leaks.len() == 1 { "" } else { "s" },
                sourcemap_leaks.leaks[0].map_url
            ),
            severity: Severity::High,
            tier: HeaderTier::Baseline,
        });
    }

    // Generate recommendations
    let recommendations = generate_recommendations(&headers, https);

    // Calculate score
    let score = calculate_security_score(&headers, &ssl, &issues);
    let grade = calculate_grade(score);

    info!(
        "Security analysis: score={}, grade={}, headers={}, protection={:?}",
        score,
        grade,
        headers.count(),
        protection
            .services
            .iter()
            .map(|s| s.name.as_str())
            .collect::<Vec<_>>(),
    );

    Ok(SecurityAnalysis {
        score,
        grade,
        headers,
        ssl,
        issues,
        recommendations,
        protection,
        sourcemap_leaks,
    })
}

fn detect_protection(headers: &HeaderMap) -> ProtectionDetection {
    let hdr = |name: &str| {
        headers
            .get(name)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_lowercase()
    };
    let has = |name: &str| headers.contains_key(name);

    let mut services: Vec<DetectedProtection> = Vec::new();

    macro_rules! push {
        ($name:expr, $kind:expr, $waf:expr, $cdn:expr) => {
            services.push(DetectedProtection {
                name: $name.to_string(),
                kind: $kind.to_string(),
                is_waf: $waf,
                is_cdn: $cdn,
            });
        };
    }

    if has("cf-ray") || hdr("server").contains("cloudflare") {
        push!("Cloudflare", "CDN + WAF", true, true);
    }
    if has("x-amz-cf-id") || hdr("via").contains("cloudfront") {
        push!("AWS CloudFront", "CDN", false, true);
    }
    if has("x-akamai-request-id")
        || has("x-check-cacheable")
        || has("akamai-origin-hop")
        || hdr("server").contains("akamaighost")
    {
        push!("Akamai", "CDN + WAF", true, true);
    }
    if has("x-fastly-request-id") || has("fastly-restarts") || hdr("x-served-by").contains("cache-")
    {
        push!("Fastly", "CDN", false, true);
    }
    if has("x-sucuri-id") || has("x-sucuri-cache") || hdr("server").contains("sucuri") {
        push!("Sucuri", "WAF + CDN", true, true);
    }
    if has("x-iinfo") || hdr("x-cdn").contains("imperva") || hdr("x-cdn").contains("incapsula") {
        push!("Imperva", "WAF + CDN", true, true);
    }
    if has("x-vercel-id") {
        push!("Vercel", "Hosting + CDN", false, true);
    }
    if has("x-nf-request-id") || hdr("server").contains("netlify") {
        push!("Netlify", "Hosting + CDN", false, true);
    }
    if has("cdn-pullzone") || has("bunny-request-id") || has("cdn-requestid") {
        push!("BunnyCDN", "CDN", false, true);
    }
    if hdr("server").contains("keycdn-engine") || (has("x-edge-location") && !has("x-amz-cf-id")) {
        push!("KeyCDN", "CDN", false, true);
    }
    if has("x-varnish") || hdr("via").contains("varnish") {
        push!("Varnish", "Cache", false, true);
    }

    let has_waf = services.iter().any(|s| s.is_waf);
    let has_cdn = services.iter().any(|s| s.is_cdn);

    ProtectionDetection {
        services,
        has_waf,
        has_cdn,
    }
}

fn extract_security_headers(headers: &HeaderMap) -> SecurityHeaders {
    SecurityHeaders {
        content_security_policy: headers
            .get("content-security-policy")
            .and_then(|v| v.to_str().ok())
            .map(String::from),
        x_content_type_options: headers
            .get("x-content-type-options")
            .and_then(|v| v.to_str().ok())
            .map(String::from),
        x_frame_options: headers
            .get("x-frame-options")
            .and_then(|v| v.to_str().ok())
            .map(String::from),
        referrer_policy: headers
            .get("referrer-policy")
            .and_then(|v| v.to_str().ok())
            .map(String::from),
        permissions_policy: headers
            .get("permissions-policy")
            .and_then(|v| v.to_str().ok())
            .map(String::from),
        strict_transport_security: headers
            .get("strict-transport-security")
            .and_then(|v| v.to_str().ok())
            .map(String::from),
        cross_origin_opener_policy: headers
            .get("cross-origin-opener-policy")
            .and_then(|v| v.to_str().ok())
            .map(String::from),
        cross_origin_resource_policy: headers
            .get("cross-origin-resource-policy")
            .and_then(|v| v.to_str().ok())
            .map(String::from),
        access_control_allow_origin: headers
            .get("access-control-allow-origin")
            .and_then(|v| v.to_str().ok())
            .map(String::from),
        access_control_allow_credentials: headers
            .get("access-control-allow-credentials")
            .and_then(|v| v.to_str().ok())
            .map(String::from),
    }
}

fn analyze_ssl(https: bool, headers: &SecurityHeaders) -> SslInfo {
    let hsts = headers.strict_transport_security.as_ref();

    let (hsts_max_age, hsts_include_subdomains, hsts_preload) = if let Some(hsts_value) = hsts {
        let max_age = hsts_value
            .split(';')
            .find(|s| s.trim().starts_with("max-age"))
            .and_then(|s| s.split('=').nth(1))
            .and_then(|s| s.trim().parse().ok());

        let include_subdomains = hsts_value.to_lowercase().contains("includesubdomains");
        let preload = hsts_value.to_lowercase().contains("preload");

        (max_age, include_subdomains, preload)
    } else {
        (None, false, false)
    };

    SslInfo {
        https,
        valid_certificate: https, // Basic assumption
        has_hsts: hsts.is_some(),
        hsts_max_age,
        hsts_include_subdomains,
        hsts_preload,
        ..Default::default()
    }
}

pub(crate) fn generate_security_issues(
    headers: &SecurityHeaders,
    https: bool,
) -> Vec<SecurityIssue> {
    let mut issues = Vec::new();

    if !https {
        issues.push(SecurityIssue {
            header: "HTTPS".to_string(),
            issue_type: "missing_https".to_string(),
            message: "Site is not served over HTTPS".to_string(),
            severity: Severity::Critical,
            tier: HeaderTier::Baseline,
        });
    }

    if headers.content_security_policy.is_none() {
        issues.push(SecurityIssue {
            header: "Content-Security-Policy".to_string(),
            issue_type: "missing_header".to_string(),
            message: "Missing Content-Security-Policy header".to_string(),
            severity: Severity::High,
            tier: HeaderTier::Baseline,
        });
    } else if let Some(ref csp) = headers.content_security_policy {
        issues.extend(collect_csp_quality_issues(csp));
    }

    if headers.x_content_type_options.is_none() {
        issues.push(SecurityIssue {
            header: "X-Content-Type-Options".to_string(),
            issue_type: "missing_header".to_string(),
            message: "Missing X-Content-Type-Options header".to_string(),
            severity: Severity::Medium,
            tier: HeaderTier::Baseline,
        });
    }

    if headers.x_frame_options.is_none() {
        issues.push(SecurityIssue {
            header: "X-Frame-Options".to_string(),
            issue_type: "missing_header".to_string(),
            message: "Missing X-Frame-Options header (clickjacking protection)".to_string(),
            severity: Severity::Medium,
            tier: HeaderTier::Baseline,
        });
    }

    if https && headers.strict_transport_security.is_none() {
        issues.push(SecurityIssue {
            header: "Strict-Transport-Security".to_string(),
            issue_type: "missing_header".to_string(),
            message: "Missing HSTS header".to_string(),
            severity: Severity::High,
            tier: HeaderTier::Baseline,
        });
    }

    if headers.referrer_policy.is_none() {
        issues.push(SecurityIssue {
            header: "Referrer-Policy".to_string(),
            issue_type: "missing_header".to_string(),
            message: "Missing Referrer-Policy header".to_string(),
            severity: Severity::Low,
            tier: HeaderTier::ArchitectureDependent,
        });
    }

    if headers.permissions_policy.is_none() {
        issues.push(SecurityIssue {
            header: "Permissions-Policy".to_string(),
            issue_type: "missing_header".to_string(),
            message: "Missing Permissions-Policy header".to_string(),
            severity: Severity::Low,
            tier: HeaderTier::ArchitectureDependent,
        });
    }

    // COOP/CORP relevance depends on deployment context this tool cannot
    // observe from a single unauthenticated fetch — a marketing page almost
    // never needs them, but an authenticated app page using SharedArrayBuffer
    // or a cross-origin OAuth popup flow genuinely might. A static crawl has
    // no way to tell those two cases apart, so the message below asks the
    // verification question instead of asserting either "add this" or
    // "ignore this" (#578).
    if headers.cross_origin_opener_policy.is_none() {
        issues.push(SecurityIssue {
            header: "Cross-Origin-Opener-Policy".to_string(),
            issue_type: "missing_header".to_string(),
            message: "Cross-Origin-Opener-Policy is not set. This header only matters if the \
                page uses SharedArrayBuffer, high-resolution timers, or needs to isolate itself \
                from cross-origin popups — if none of that applies, no action is needed here. If \
                it does apply, set it to same-origin and verify popup/window interactions still \
                work as expected."
                .to_string(),
            severity: Severity::Low,
            tier: HeaderTier::ContextDependent,
        });
    }

    if headers.cross_origin_resource_policy.is_none() {
        issues.push(SecurityIssue {
            header: "Cross-Origin-Resource-Policy".to_string(),
            issue_type: "missing_header".to_string(),
            message: "Cross-Origin-Resource-Policy is not set. This header only matters if the \
                page serves fonts, scripts, or media that other origins should be prevented from \
                loading — if the site's resources are intentionally public, no action is needed \
                here. If cross-origin loading should be restricted, set it to same-origin or \
                same-site."
                .to_string(),
            severity: Severity::Low,
            tier: HeaderTier::ContextDependent,
        });
    }

    if cors_allows_wildcard_credentials(headers) {
        issues.push(SecurityIssue {
            header: "Access-Control-Allow-Origin".to_string(),
            issue_type: "cors_wildcard_credentials".to_string(),
            message: "CORS allows any origin while also allowing credentials".to_string(),
            severity: Severity::High,
            tier: HeaderTier::Baseline,
        });
    }

    issues
}

fn cors_allows_wildcard_credentials(headers: &SecurityHeaders) -> bool {
    headers
        .access_control_allow_origin
        .as_deref()
        .is_some_and(|origin| origin.trim() == "*")
        && headers
            .access_control_allow_credentials
            .as_deref()
            .is_some_and(|credentials| credentials.trim().eq_ignore_ascii_case("true"))
}

/// True when the HSTS header's `preload` directive is present but the site
/// doesn't actually meet the HSTS preload list requirements (max-age of at
/// least one year AND `includeSubDomains`) — claiming preload readiness
/// without meeting it is a misleading/incomplete configuration (#535).
fn hsts_preload_ineligible(ssl: &SslInfo) -> bool {
    const ONE_YEAR_SECONDS: u64 = 31_536_000;
    ssl.hsts_preload
        && (ssl.hsts_max_age.unwrap_or(0) < ONE_YEAR_SECONDS || !ssl.hsts_include_subdomains)
}

/// True when Permissions-Policy is present but restricts nothing: an empty
/// value, or every listed feature left wide open with `*` (#535).
fn permissions_policy_is_permissive(headers: &SecurityHeaders) -> bool {
    let Some(value) = headers.permissions_policy.as_deref() else {
        return false;
    };
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return true;
    }
    trimmed.split(',').all(|directive| {
        directive
            .split('=')
            .nth(1)
            .map(|allowlist| allowlist.trim() == "*")
            .unwrap_or(false)
    })
}

fn collect_csp_quality_issues(policy: &str) -> Vec<SecurityIssue> {
    let directives = parse_csp_directives(policy);
    let mut issues = Vec::new();

    let effective_script = directive_values(&directives, "script-src")
        .or_else(|| directive_values(&directives, "default-src"))
        .unwrap_or_default();
    if effective_script.contains(&"'unsafe-inline'") && !has_nonce_or_hash(effective_script) {
        issues.push(csp_issue(
            "unsafe_inline_script",
            "CSP allows unsafe-inline scripts without nonce/hash protection",
            Severity::High,
        ));
    }
    if effective_script.contains(&"'unsafe-eval'") {
        issues.push(csp_issue(
            "unsafe_eval_script",
            "CSP allows unsafe-eval in script sources",
            Severity::High,
        ));
    }
    if has_wildcard_source(effective_script) {
        issues.push(csp_issue(
            "wildcard_script_source",
            "CSP allows wildcard script sources",
            Severity::High,
        ));
    }

    let effective_style = directive_values(&directives, "style-src")
        .or_else(|| directive_values(&directives, "default-src"))
        .unwrap_or_default();
    if effective_style.contains(&"'unsafe-inline'") && !has_nonce_or_hash(effective_style) {
        issues.push(csp_issue(
            "unsafe_inline_style",
            "CSP allows unsafe-inline styles without nonce/hash protection",
            Severity::Medium,
        ));
    }

    if has_wildcard_source(
        directives
            .values()
            .flatten()
            .copied()
            .collect::<Vec<_>>()
            .as_slice(),
    ) {
        issues.push(csp_issue(
            "wildcard_source",
            "CSP contains wildcard source expressions",
            Severity::Medium,
        ));
    }

    for (directive, severity) in [
        ("object-src", Severity::Medium),
        ("base-uri", Severity::Medium),
        ("frame-ancestors", Severity::Medium),
    ] {
        if !directives.contains_key(directive) {
            issues.push(csp_issue(
                &format!("missing_{directive}"),
                &format!("CSP missing {directive} directive"),
                severity,
            ));
        }
    }

    issues
}

fn generate_csp_recommendations(policy: &str) -> Vec<String> {
    let issue_types: std::collections::BTreeSet<_> = collect_csp_quality_issues(policy)
        .into_iter()
        .map(|issue| issue.issue_type)
        .collect();
    let mut recommendations = Vec::new();

    if issue_types.contains("unsafe_inline_script") || issue_types.contains("unsafe_inline_style") {
        recommendations.push(
            "Remove unsafe-inline from CSP or replace it with nonces/hashes for approved inline code"
                .to_string(),
        );
    }
    if issue_types.contains("unsafe_eval_script") {
        recommendations.push(
            "Remove unsafe-eval from script-src; avoid runtime string evaluation in production"
                .to_string(),
        );
    }
    if issue_types.contains("wildcard_script_source") || issue_types.contains("wildcard_source") {
        recommendations
            .push("Replace wildcard CSP sources with explicit trusted origins".to_string());
    }
    if issue_types
        .iter()
        .any(|issue| issue.starts_with("missing_"))
    {
        recommendations.push(
            "Harden CSP with object-src 'none', base-uri 'self', and frame-ancestors 'none' or a trusted origin"
                .to_string(),
        );
    }

    recommendations
}

fn csp_issue(issue_type: &str, message: &str, severity: Severity) -> SecurityIssue {
    SecurityIssue {
        header: "Content-Security-Policy".to_string(),
        issue_type: issue_type.to_string(),
        message: message.to_string(),
        severity,
        tier: HeaderTier::Baseline,
    }
}

fn parse_csp_directives(policy: &str) -> BTreeMap<String, Vec<&str>> {
    let mut directives = BTreeMap::new();
    for directive in policy.split(';') {
        let mut parts = directive.split_whitespace();
        let Some(name) = parts.next() else {
            continue;
        };
        directives.insert(name.to_ascii_lowercase(), parts.collect());
    }
    directives
}

fn directive_values<'a>(
    directives: &'a BTreeMap<String, Vec<&'a str>>,
    name: &str,
) -> Option<&'a [&'a str]> {
    directives.get(name).map(Vec::as_slice)
}

fn has_nonce_or_hash(values: &[&str]) -> bool {
    values.iter().any(|value| {
        value.starts_with("'nonce-")
            || value.starts_with("'sha256-")
            || value.starts_with("'sha384-")
            || value.starts_with("'sha512-")
    })
}

fn has_wildcard_source(values: &[&str]) -> bool {
    values.iter().any(|value| {
        *value == "*"
            || value.starts_with("*.")
            || value.starts_with("https://*")
            || value.starts_with("http://*")
    })
}

fn generate_recommendations(headers: &SecurityHeaders, https: bool) -> Vec<String> {
    let mut recommendations = Vec::new();

    if !https {
        recommendations.push("Enable HTTPS with a valid SSL certificate".to_string());
    }

    if headers.content_security_policy.is_none() {
        recommendations.push(
            "Add Content-Security-Policy header to prevent XSS and data injection".to_string(),
        );
    } else if let Some(ref csp) = headers.content_security_policy {
        recommendations.extend(generate_csp_recommendations(csp));
    }

    if headers.x_content_type_options.is_none() {
        recommendations
            .push("Add X-Content-Type-Options: nosniff to prevent MIME-type sniffing".to_string());
    }

    if headers.x_frame_options.is_none() {
        recommendations
            .push("Add X-Frame-Options: DENY or SAMEORIGIN to prevent clickjacking".to_string());
    }

    if https && headers.strict_transport_security.is_none() {
        recommendations.push(
            "Add Strict-Transport-Security header with max-age of at least 1 year".to_string(),
        );
    }

    if headers.permissions_policy.is_none() {
        recommendations
            .push("Add Permissions-Policy header to control browser features".to_string());
    }

    if headers.cross_origin_opener_policy.is_none() {
        recommendations.push(
            "Consider Cross-Origin-Opener-Policy (same-origin) if the site uses SharedArrayBuffer, \
             high-resolution timers, or cross-origin popups — not required for standard sites."
                .to_string(),
        );
    }

    if headers.cross_origin_resource_policy.is_none() {
        recommendations.push(
            "Consider Cross-Origin-Resource-Policy if the site serves fonts, scripts, or media \
             that other origins should not be able to load — not required if resources are intentionally public."
                .to_string(),
        );
    }

    if cors_allows_wildcard_credentials(headers) {
        recommendations.push(
            "Do not combine Access-Control-Allow-Origin: * with credentialed CORS; allowlist trusted origins instead."
                .to_string(),
        );
    }

    recommendations
}

pub(crate) fn calculate_security_score(
    _headers: &SecurityHeaders,
    ssl: &SslInfo,
    issues: &[SecurityIssue],
) -> u32 {
    let mut score = 100u32;

    // A *present* CSP (even a permissive one) provides partial protection, so its
    // cumulative quality penalty must never exceed the penalty for a *missing*
    // CSP — otherwise a site with an imperfect CSP scores worse than one with no
    // CSP at all, which is backwards. We therefore bucket CSP-quality issues and
    // cap their combined deduction at the missing-CSP penalty (High = 15).
    const MISSING_CSP_PENALTY: u32 = 15;
    let deduction = |sev: &Severity| match sev {
        Severity::Critical => 25,
        Severity::High => 15,
        Severity::Medium => 10,
        Severity::Low => 5,
    };
    let mut csp_quality_penalty = 0u32;
    for issue in issues {
        let is_csp_quality =
            issue.header == "Content-Security-Policy" && issue.issue_type != "missing_header";
        if is_csp_quality {
            csp_quality_penalty += deduction(&issue.severity);
        } else {
            // Deduct for issues (includes missing HTTPS as a critical issue)
            score = score.saturating_sub(deduction(&issue.severity));
        }
    }
    score = score.saturating_sub(csp_quality_penalty.min(MISSING_CSP_PENALTY));

    // Bonus for HSTS
    if ssl.has_hsts {
        score = score.saturating_add(5).min(100);
        if ssl.hsts_include_subdomains {
            score = score.saturating_add(3).min(100);
        }
        if ssl.hsts_preload {
            score = score.saturating_add(2).min(100);
        }
    }

    score
}

/// Coarse security sub-category (plan/5-module-accessibility-security-
/// driver-detail.md): Security has no per-check taxonomy registry like
/// WCAG's `taxonomy::Subcategory`, so this groups `SecurityIssue.header`
/// values into the handful of categories a non-technical reader would
/// recognize, mirroring the review's own example wording ("fehlende
/// Security-Header, unsichere externe Ressourcen, Cookie-/Transport-
/// Konfiguration").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SecurityCategory {
    /// CSP, X-Content-Type-Options, X-Frame-Options — baseline response
    /// headers that harden the page itself.
    #[default]
    ResponseHeaders,
    /// HTTPS/TLS transport security (HSTS).
    TransportSecurity,
    /// Cross-origin/embedding access policies (Referrer-Policy,
    /// Permissions-Policy, Cross-Origin-Opener/Resource-Policy, CORS).
    AccessPolicies,
    /// Publicly reachable source maps and other third-party/asset exposure.
    ThirdPartyExposure,
}

impl SecurityCategory {
    pub fn label(&self, en: bool) -> &'static str {
        if en {
            match self {
                Self::ResponseHeaders => "Response headers",
                Self::TransportSecurity => "Transport security",
                Self::AccessPolicies => "Access policies",
                Self::ThirdPartyExposure => "Third-party exposure",
            }
        } else {
            match self {
                Self::ResponseHeaders => "Response-Header",
                Self::TransportSecurity => "Transportsicherheit",
                Self::AccessPolicies => "Zugriffs-Richtlinien",
                Self::ThirdPartyExposure => "Drittanbieter-Exposition",
            }
        }
    }

    /// All categories, in a fixed display order.
    pub fn all() -> [SecurityCategory; 4] {
        [
            Self::ResponseHeaders,
            Self::TransportSecurity,
            Self::AccessPolicies,
            Self::ThirdPartyExposure,
        ]
    }

    fn for_header(header: &str) -> SecurityCategory {
        match header {
            "HTTPS" | "Strict-Transport-Security" => Self::TransportSecurity,
            "Content-Security-Policy" | "X-Content-Type-Options" | "X-Frame-Options" => {
                Self::ResponseHeaders
            }
            "Referrer-Policy"
            | "Permissions-Policy"
            | "Cross-Origin-Opener-Policy"
            | "Cross-Origin-Resource-Policy"
            | "Access-Control-Allow-Origin" => Self::AccessPolicies,
            "Source Map" => Self::ThirdPartyExposure,
            // Unrecognized/future header names default to the general
            // response-headers bucket rather than being silently dropped.
            _ => Self::ResponseHeaders,
        }
    }
}

/// Per-category security score (0-100), reusing the exact severity→penalty
/// mapping `calculate_security_score` uses for the overall score, just
/// scoped to the issues in each category — so a category with e.g. one
/// missing baseline header scores the same way that single issue would
/// deduct from the overall score, not a newly invented scale. A category
/// with no issues scores 100 (plan/5-module-accessibility-security-driver-
/// detail.md).
pub fn calculate_security_category_scores(
    issues: &[SecurityIssue],
) -> Vec<(SecurityCategory, u32)> {
    let deduction = |sev: &Severity| match sev {
        Severity::Critical => 25,
        Severity::High => 15,
        Severity::Medium => 10,
        Severity::Low => 5,
    };
    SecurityCategory::all()
        .into_iter()
        .map(|category| {
            let mut score = 100u32;
            for issue in issues {
                if SecurityCategory::for_header(&issue.header) == category {
                    score = score.saturating_sub(deduction(&issue.severity));
                }
            }
            (category, score)
        })
        .collect()
}

fn calculate_grade(score: u32) -> String {
    crate::registry::SECURITY_GRADE
        .label(score as f32, false)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coop_corp_verification_text_is_localized_and_distinct_from_english() {
        for header in ["Cross-Origin-Opener-Policy", "Cross-Origin-Resource-Policy"] {
            let en = coop_corp_verification_text(header, "missing_header", true)
                .expect("expected English text");
            let de = coop_corp_verification_text(header, "missing_header", false)
                .expect("expected German text");
            assert_ne!(en, de, "{header}: DE text must differ from EN");
            let has_umlaut = |s: &str| s.chars().any(|c| "äöüÄÖÜß".contains(c));
            assert!(!has_umlaut(en), "{header} EN text leaks German: {en}");
        }
    }

    #[test]
    fn coop_corp_verification_text_none_for_other_issue_types() {
        assert!(coop_corp_verification_text(
            "Cross-Origin-Opener-Policy",
            "some_other_issue_type",
            true
        )
        .is_none());
        assert!(coop_corp_verification_text("X-Frame-Options", "missing_header", true).is_none());
    }

    #[test]
    fn test_security_headers_count() {
        let headers = SecurityHeaders {
            content_security_policy: Some(strong_csp()),
            x_content_type_options: Some("nosniff".to_string()),
            x_frame_options: Some("DENY".to_string()),
            ..Default::default()
        };

        assert_eq!(headers.count(), 3);
    }

    #[test]
    fn test_generate_recommendations_coop_corp_generate_low_issues() {
        let headers = SecurityHeaders {
            content_security_policy: Some("default-src 'self'".to_string()),
            x_content_type_options: Some("nosniff".to_string()),
            x_frame_options: Some("DENY".to_string()),
            strict_transport_security: Some("max-age=31536000".to_string()),
            referrer_policy: Some("strict-origin".to_string()),
            permissions_policy: Some("camera=()".to_string()),
            ..Default::default()
        };
        let recs = generate_recommendations(&headers, true);
        // COOP and CORP absent → informational recommendations are still generated
        assert!(recs
            .iter()
            .any(|r| r.contains("Cross-Origin-Opener-Policy")));
        assert!(recs
            .iter()
            .any(|r| r.contains("Cross-Origin-Resource-Policy")));
        // COOP and CORP now generate Low-severity issues, tagged context-dependent
        let issues = generate_security_issues(&headers, true);
        assert!(issues.iter().any(|i| i.header.contains("Cross-Origin")
            && i.severity == Severity::Low
            && i.tier == HeaderTier::ContextDependent));
    }

    /// Header classification table (#578): Baseline headers apply to
    /// virtually every site, architecture-dependent ones are good practice
    /// but not universal, context-dependent relevance can't be determined
    /// from a single automated fetch, and CORS presence/absence alone isn't
    /// assessable at all.
    #[test]
    fn test_header_tier_classification() {
        assert_eq!(header_tier("Content-Security-Policy"), HeaderTier::Baseline);
        assert_eq!(
            header_tier("Strict-Transport-Security"),
            HeaderTier::Baseline
        );
        assert_eq!(header_tier("X-Content-Type-Options"), HeaderTier::Baseline);
        assert_eq!(header_tier("X-Frame-Options"), HeaderTier::Baseline);
        assert_eq!(header_tier("HTTPS"), HeaderTier::Baseline);

        assert_eq!(
            header_tier("Referrer-Policy"),
            HeaderTier::ArchitectureDependent
        );
        assert_eq!(
            header_tier("Permissions-Policy"),
            HeaderTier::ArchitectureDependent
        );

        assert_eq!(
            header_tier("Cross-Origin-Opener-Policy"),
            HeaderTier::ContextDependent
        );
        assert_eq!(
            header_tier("Cross-Origin-Resource-Policy"),
            HeaderTier::ContextDependent
        );

        assert_eq!(
            header_tier("Access-Control-Allow-Origin"),
            HeaderTier::NotAssessable
        );
        assert_eq!(
            header_tier("Access-Control-Allow-Credentials"),
            HeaderTier::NotAssessable
        );
    }

    /// "Standardmarketingseite" scenario (#578 acceptance criteria): a page
    /// with only baseline headers set and COOP/CORP missing should read as a
    /// verification question, not an unconditional "add this header"
    /// instruction — the tool cannot know whether this deployment needs
    /// cross-origin isolation at all.
    #[test]
    fn test_coop_missing_message_is_a_verification_question_not_a_flat_instruction() {
        let headers = SecurityHeaders::default();
        let issues = generate_security_issues(&headers, true);
        let coop = issues
            .iter()
            .find(|i| i.header == "Cross-Origin-Opener-Policy")
            .expect("COOP issue expected when header is absent");

        assert_eq!(coop.severity, Severity::Low);
        assert_eq!(coop.tier, HeaderTier::ContextDependent);
        assert!(
            coop.message.contains("SharedArrayBuffer"),
            "message should name the concrete scenario that makes COOP relevant: {}",
            coop.message
        );
        assert!(
            coop.message.contains("no action is needed"),
            "message should tell the reader when COOP does NOT apply: {}",
            coop.message
        );
        assert!(
            !coop.message.starts_with("Missing"),
            "message should not read as a flat missing-header instruction: {}",
            coop.message
        );
    }

    /// "Cross-Origin-Ressourcen-Szenario" (#578 acceptance criteria): CORP's
    /// guidance must name the concrete scenario (other origins loading
    /// fonts/scripts/media) rather than a blanket "add this header" line.
    #[test]
    fn test_corp_missing_message_covers_cross_origin_resource_scenario() {
        let headers = SecurityHeaders::default();
        let issues = generate_security_issues(&headers, true);
        let corp = issues
            .iter()
            .find(|i| i.header == "Cross-Origin-Resource-Policy")
            .expect("CORP issue expected when header is absent");

        assert_eq!(corp.severity, Severity::Low);
        assert_eq!(corp.tier, HeaderTier::ContextDependent);
        assert!(
            corp.message.contains("other origins"),
            "message should name the cross-origin resource-loading scenario: {}",
            corp.message
        );
        assert!(
            corp.message.contains("intentionally public"),
            "message should tell the reader when CORP does NOT apply: {}",
            corp.message
        );
    }

    #[test]
    fn permissive_csp_scores_at_least_as_high_as_missing_csp() {
        let ssl = SslInfo::default();
        // Same baseline issue set; one variant has no CSP, the other has a
        // present-but-permissive CSP that fires several quality issues.
        let missing_csp = vec![SecurityIssue {
            header: "Content-Security-Policy".into(),
            issue_type: "missing_header".into(),
            message: "Missing Content-Security-Policy header".into(),
            severity: Severity::High,
            tier: HeaderTier::Baseline,
        }];
        let permissive_csp: Vec<SecurityIssue> = [
            "unsafe_inline_script",
            "unsafe_eval_script",
            "wildcard_script_source",
            "unsafe_inline_style",
            "wildcard_source",
        ]
        .iter()
        .map(|t| SecurityIssue {
            header: "Content-Security-Policy".into(),
            issue_type: (*t).into(),
            message: "csp quality".into(),
            severity: Severity::High,
            tier: HeaderTier::Baseline,
        })
        .collect();
        let s_missing = calculate_security_score(&SecurityHeaders::default(), &ssl, &missing_csp);
        let s_permissive =
            calculate_security_score(&SecurityHeaders::default(), &ssl, &permissive_csp);
        assert!(
            s_permissive >= s_missing,
            "a present (permissive) CSP must not score worse than no CSP: permissive={s_permissive} missing={s_missing}"
        );
    }

    #[test]
    fn test_calculate_grade() {
        assert_eq!(calculate_grade(95), "A+");
        assert_eq!(calculate_grade(85), "A");
        assert_eq!(calculate_grade(75), "B");
        assert_eq!(calculate_grade(65), "C");
        assert_eq!(calculate_grade(55), "D");
        assert_eq!(calculate_grade(40), "F");
    }

    #[test]
    fn test_analyze_ssl_with_hsts() {
        let headers = SecurityHeaders {
            strict_transport_security: Some(
                "max-age=31536000; includeSubDomains; preload".to_string(),
            ),
            ..Default::default()
        };

        let ssl = analyze_ssl(true, &headers);
        assert!(ssl.https);
        assert!(ssl.has_hsts);
        assert_eq!(ssl.hsts_max_age, Some(31536000));
        assert!(ssl.hsts_include_subdomains);
        assert!(ssl.hsts_preload);
    }

    #[test]
    fn test_analyze_ssl_without_hsts() {
        let headers = SecurityHeaders::default();
        let ssl = analyze_ssl(false, &headers);
        assert!(!ssl.https);
        assert!(!ssl.has_hsts);
        assert_eq!(ssl.hsts_max_age, None);
    }

    #[test]
    fn test_generate_security_issues_no_https() {
        let headers = SecurityHeaders::default();
        let issues = generate_security_issues(&headers, false);

        assert!(issues.iter().any(|i| i.header == "HTTPS"));
        assert!(issues.iter().any(|i| i.header == "Content-Security-Policy"));
        assert!(issues.iter().any(|i| i.header == "X-Content-Type-Options"));
        // HSTS issue should NOT appear for non-HTTPS sites
        assert!(!issues
            .iter()
            .any(|i| i.header == "Strict-Transport-Security"));
    }

    #[test]
    fn test_generate_security_issues_https_no_hsts() {
        let headers = SecurityHeaders::default();
        let issues = generate_security_issues(&headers, true);

        // No HTTPS issue
        assert!(!issues.iter().any(|i| i.header == "HTTPS"));
        // But HSTS should be flagged
        assert!(issues
            .iter()
            .any(|i| i.header == "Strict-Transport-Security"));
    }

    #[test]
    fn test_generate_security_issues_all_headers_present() {
        let headers = SecurityHeaders {
            content_security_policy: Some(strong_csp()),
            x_content_type_options: Some("nosniff".to_string()),
            x_frame_options: Some("DENY".to_string()),
            strict_transport_security: Some("max-age=31536000".to_string()),
            referrer_policy: Some("strict-origin".to_string()),
            permissions_policy: Some("camera=()".to_string()),
            cross_origin_opener_policy: Some("same-origin".to_string()),
            cross_origin_resource_policy: Some("same-origin".to_string()),
            ..Default::default()
        };
        let issues = generate_security_issues(&headers, true);
        assert!(issues.is_empty());
    }

    #[test]
    fn test_cors_wildcard_credentials_generates_high_issue() {
        let headers = SecurityHeaders {
            access_control_allow_origin: Some("*".to_string()),
            access_control_allow_credentials: Some("true".to_string()),
            ..Default::default()
        };

        let issues = generate_security_issues(&headers, true);
        assert!(issues.iter().any(|issue| {
            issue.issue_type == "cors_wildcard_credentials" && issue.severity == Severity::High
        }));
    }

    #[test]
    fn test_cors_wildcard_without_credentials_is_not_flagged() {
        let headers = SecurityHeaders {
            access_control_allow_origin: Some("*".to_string()),
            ..Default::default()
        };

        let issues = generate_security_issues(&headers, true);
        assert!(!issues
            .iter()
            .any(|issue| issue.issue_type == "cors_wildcard_credentials"));
    }

    #[test]
    fn hsts_preload_ineligible_flags_short_max_age() {
        let ssl = SslInfo {
            hsts_preload: true,
            hsts_max_age: Some(3600),
            hsts_include_subdomains: true,
            ..Default::default()
        };
        assert!(hsts_preload_ineligible(&ssl));
    }

    #[test]
    fn hsts_preload_ineligible_flags_missing_subdomains() {
        let ssl = SslInfo {
            hsts_preload: true,
            hsts_max_age: Some(31536000),
            hsts_include_subdomains: false,
            ..Default::default()
        };
        assert!(hsts_preload_ineligible(&ssl));
    }

    #[test]
    fn hsts_preload_eligible_is_not_flagged() {
        let ssl = SslInfo {
            hsts_preload: true,
            hsts_max_age: Some(31536000),
            hsts_include_subdomains: true,
            ..Default::default()
        };
        assert!(!hsts_preload_ineligible(&ssl));
    }

    #[test]
    fn hsts_preload_not_set_is_not_flagged_regardless_of_max_age() {
        let ssl = SslInfo {
            hsts_preload: false,
            hsts_max_age: Some(60),
            hsts_include_subdomains: false,
            ..Default::default()
        };
        assert!(!hsts_preload_ineligible(&ssl));
    }

    #[test]
    fn permissions_policy_empty_value_is_permissive() {
        let headers = SecurityHeaders {
            permissions_policy: Some("".to_string()),
            ..Default::default()
        };
        assert!(permissions_policy_is_permissive(&headers));
    }

    #[test]
    fn permissions_policy_wildcard_only_is_permissive() {
        let headers = SecurityHeaders {
            permissions_policy: Some("camera=*, microphone=*".to_string()),
            ..Default::default()
        };
        assert!(permissions_policy_is_permissive(&headers));
    }

    #[test]
    fn permissions_policy_restrictive_is_not_flagged() {
        let headers = SecurityHeaders {
            permissions_policy: Some("camera=(), microphone=(self)".to_string()),
            ..Default::default()
        };
        assert!(!permissions_policy_is_permissive(&headers));
    }

    #[test]
    fn permissions_policy_absent_is_not_flagged_as_permissive() {
        let headers = SecurityHeaders::default();
        assert!(!permissions_policy_is_permissive(&headers));
    }

    #[test]
    fn test_csp_quality_flags_unsafe_and_wildcards() {
        let issues = collect_csp_quality_issues(
            "default-src *; script-src 'self' 'unsafe-inline' 'unsafe-eval' https://*",
        );

        assert!(issues
            .iter()
            .any(|issue| issue.issue_type == "unsafe_inline_script"));
        assert!(issues
            .iter()
            .any(|issue| issue.issue_type == "unsafe_eval_script"));
        assert!(issues
            .iter()
            .any(|issue| issue.issue_type == "wildcard_script_source"));
        assert!(issues
            .iter()
            .any(|issue| issue.issue_type == "missing_object-src"));
    }

    #[test]
    fn test_csp_quality_accepts_nonce_hardened_policy() {
        let issues = collect_csp_quality_issues(
            "default-src 'self'; script-src 'self' 'nonce-abc123'; style-src 'self' 'sha256-abc'; object-src 'none'; base-uri 'self'; frame-ancestors 'none'",
        );

        assert!(issues.is_empty());
    }

    #[test]
    fn test_csp_recommendations_explain_quality_findings() {
        let recommendations =
            generate_csp_recommendations("default-src *; script-src 'unsafe-inline' 'unsafe-eval'");

        assert!(recommendations
            .iter()
            .any(|recommendation| recommendation.contains("unsafe-inline")));
        assert!(recommendations
            .iter()
            .any(|recommendation| recommendation.contains("unsafe-eval")));
        assert!(recommendations
            .iter()
            .any(|recommendation| recommendation.contains("wildcard")));
    }

    #[test]
    fn test_generate_recommendations() {
        let headers = SecurityHeaders::default();
        let recs = generate_recommendations(&headers, true);

        assert!(recs.iter().any(|r| r.contains("Content-Security-Policy")));
        assert!(recs.iter().any(|r| r.contains("X-Content-Type-Options")));
        assert!(recs.iter().any(|r| r.contains("Strict-Transport-Security")));
    }

    #[test]
    fn test_calculate_security_score_perfect() {
        let headers = SecurityHeaders::default();
        let ssl = SslInfo {
            https: true,
            valid_certificate: true,
            has_hsts: true,
            hsts_max_age: Some(31536000),
            hsts_include_subdomains: true,
            hsts_preload: true,
            ..Default::default()
        };
        let issues = vec![];
        let score = calculate_security_score(&headers, &ssl, &issues);
        // 100 base + 5 (hsts) + 3 (subdomains) + 2 (preload) = 110, capped at 100
        assert_eq!(score, 100);
    }

    #[test]
    fn test_calculate_security_score_no_https() {
        let headers = SecurityHeaders::default();
        let ssl = SslInfo::default();
        let issues = generate_security_issues(&headers, false);
        let score = calculate_security_score(&headers, &ssl, &issues);
        // Should lose points for critical (HTTPS) + high (CSP) + medium (XCT, XFO) + low (referrer)
        assert!(score < 50);
    }

    fn strong_csp() -> String {
        "default-src 'self'; script-src 'self'; style-src 'self'; object-src 'none'; base-uri 'self'; frame-ancestors 'none'".to_string()
    }
}
