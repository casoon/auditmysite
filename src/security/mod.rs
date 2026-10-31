//! Security headers analysis module
//!
//! Analyzes HTTP security headers and SSL/TLS configuration.

pub mod module;
pub(crate) mod sourcemap;
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
    /// Where the analysed headers came from (#774).
    #[serde(default)]
    pub fetch: HeaderFetch,
}

/// Provenance of the response whose headers the security analysis reads.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeaderFetch {
    /// `http_probe`: a request outside the browser, not the browser's own
    /// document response.
    pub source: String,
    /// The method that produced the analysed response (`HEAD` or `GET`).
    pub method: String,
    pub status: u16,
    pub requested_url: String,
    /// The URL after redirects — its scheme is the transport the headers
    /// were served over (#775).
    pub final_url: String,
    /// Status of a rejected `HEAD` that made the probe fall back to `GET`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub head_status: Option<u16>,
}

/// The security headers of the audited URL, fetched outside the browser.
pub(crate) static HEADERS: crate::audit::collection::CollectorSpec =
    crate::audit::collection::CollectorSpec {
        id: "security.headers",
        module: "security",
        method: crate::audit::collection::CollectionMethod::Http,
        method_version: 2,
        scoring: true,
    };

/// Public source maps of the page's assets.
pub(crate) static SOURCE_MAPS: crate::audit::collection::CollectorSpec =
    crate::audit::collection::CollectorSpec {
        id: "security.source_maps",
        module: "security",
        method: crate::audit::collection::CollectionMethod::Http,
        method_version: 2,
        scoring: true,
    };

/// TLS certificate state as the browser saw it.
pub(crate) static CERTIFICATE: crate::audit::collection::CollectorSpec =
    crate::audit::collection::CollectorSpec {
        id: "security.certificate",
        module: "security",
        method: crate::audit::collection::CollectionMethod::Cdp,
        method_version: 1,
        scoring: false,
    };

/// Whether the probe's final document is the one the browser audited:
/// same scheme and host after redirects (#775).
pub(crate) static DOCUMENT_MATCH: crate::audit::collection::CollectorSpec =
    crate::audit::collection::CollectorSpec {
        id: "security.document_match",
        module: "security",
        method: crate::audit::collection::CollectionMethod::Http,
        method_version: 1,
        scoring: true,
    };

/// Fetch the response whose headers are analysed: `HEAD`, and `GET` when
/// `HEAD` fails or is rejected (405, 403, …). A request that does not
/// complete, times out or ends in an error status is no measurement — an
/// error page's headers are not the site's (#774).
pub(crate) async fn fetch_headers(
    client: &reqwest::Client,
    url: &str,
) -> crate::audit::collection::Collected<(HeaderMap, HeaderFetch)> {
    use crate::audit::collection::Collected;
    let head = client.head(url).send().await;
    let head_status = match &head {
        Ok(response) if response.status().is_success() => None,
        Ok(response) => Some(response.status().as_u16()),
        Err(_) => None,
    };
    let (method, response) = match head {
        Ok(response) if response.status().is_success() => ("HEAD", response),
        _ => match client.get(url).send().await {
            Ok(response) => ("GET", response),
            Err(error) if error.is_timeout() => return Collected::Failed("http_timeout"),
            Err(error) if error.is_connect() => return Collected::Failed("http_connect_failed"),
            Err(_) => {
                return Collected::Failed(crate::audit::collection::reason::HTTP_REQUEST_FAILED)
            }
        },
    };
    if !response.status().is_success() {
        return Collected::Failed("http_error_status");
    }
    let fetch = HeaderFetch {
        source: "http_probe".to_string(),
        method: method.to_string(),
        status: response.status().as_u16(),
        requested_url: url.to_string(),
        final_url: response.url().to_string(),
        head_status,
    };
    Collected::Measured((response.headers().clone(), fetch))
}

/// Same origin (scheme, host, port): the probe analysed the document the
/// browser loaded. The path stays out: a client-side route change does not
/// change which server answered.
pub(crate) fn document_match(
    fetch: &HeaderFetch,
    browser_final_url: Option<&str>,
) -> crate::audit::collection::Collected<()> {
    use crate::audit::collection::Collected;
    let Some(browser) = browser_final_url.and_then(|u| url::Url::parse(u).ok()) else {
        return Collected::Failed("browser_final_url_unknown");
    };
    let Ok(probe) = url::Url::parse(&fetch.final_url) else {
        return Collected::Failed("probe_final_url_invalid");
    };
    if probe.scheme() == browser.scheme()
        && probe.host_str() == browser.host_str()
        && probe.port_or_known_default() == browser.port_or_known_default()
    {
        Collected::Measured(())
    } else {
        Collected::Failed("probe_document_mismatch")
    }
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
    /// Canonical English, produced by [`security_issue_text`] (#406). The PDF
    /// re-derives the localized wording from `kind()` plus `values`; this field
    /// keeps the stored JSON language-neutral.
    pub message: String,
    pub severity: Severity,
    /// Classification tier for the underlying header (#578) — lets report
    /// consumers distinguish baseline hygiene from context-dependent
    /// findings without re-deriving it from `header`.
    pub tier: HeaderTier,
    /// Raw values interpolated into the message, so the presentation layer can
    /// rebuild the sentence in another language instead of parsing `message`.
    #[serde(default, skip_serializing_if = "SecurityIssueValues::is_empty")]
    pub values: SecurityIssueValues,
    /// Points this issue took off the module score — its entry in the one
    /// contribution list behind the module and category scores
    /// ([`issue_deductions`], plan 72). Absent in older reports.
    #[serde(default)]
    pub score_deduction: u32,
}

/// The canonical identity of a security finding — the single key
/// [`security_issue_text`] renders from.
///
/// Derived from the already-stored `header`/`issue_type` pair rather than
/// stored as a fourth field, so reports written by older builds (and cached
/// artifacts) keep working unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityIssueKind {
    MissingHttps,
    MissingCsp,
    MissingXContentTypeOptions,
    MissingXFrameOptions,
    MissingHsts,
    MissingReferrerPolicy,
    MissingPermissionsPolicy,
    MissingCoop,
    MissingCorp,
    CorsWildcardCredentials,
    HstsPreloadIneligible,
    PermissionsPolicyPermissive,
    PublicSourceMap,
    CspUnsafeInlineScript,
    CspUnsafeEvalScript,
    CspWildcardScriptSource,
    CspUnsafeInlineStyle,
    CspWildcardSource,
    /// `object-src`, `base-uri`, `frame-ancestors` — the directive name lives
    /// in [`SecurityIssueValues::csp_directive`].
    CspMissingDirective,
}

/// The CSP directives [`SecurityIssueKind::CspMissingDirective`] is raised for.
pub const CSP_REQUIRED_DIRECTIVES: [&str; 3] = ["object-src", "base-uri", "frame-ancestors"];

impl SecurityIssueKind {
    /// Every kind this build can emit. Add new variants here — the
    /// localization guard and the check inventory both iterate it.
    pub const ALL: [SecurityIssueKind; 19] = [
        SecurityIssueKind::MissingHttps,
        SecurityIssueKind::MissingCsp,
        SecurityIssueKind::MissingXContentTypeOptions,
        SecurityIssueKind::MissingXFrameOptions,
        SecurityIssueKind::MissingHsts,
        SecurityIssueKind::MissingReferrerPolicy,
        SecurityIssueKind::MissingPermissionsPolicy,
        SecurityIssueKind::MissingCoop,
        SecurityIssueKind::MissingCorp,
        SecurityIssueKind::CorsWildcardCredentials,
        SecurityIssueKind::HstsPreloadIneligible,
        SecurityIssueKind::PermissionsPolicyPermissive,
        SecurityIssueKind::PublicSourceMap,
        SecurityIssueKind::CspUnsafeInlineScript,
        SecurityIssueKind::CspUnsafeEvalScript,
        SecurityIssueKind::CspWildcardScriptSource,
        SecurityIssueKind::CspUnsafeInlineStyle,
        SecurityIssueKind::CspWildcardSource,
        SecurityIssueKind::CspMissingDirective,
    ];

    /// The response header a kind is reported against.
    pub fn header(self) -> &'static str {
        use SecurityIssueKind::*;
        match self {
            MissingHttps => "HTTPS",
            MissingCsp
            | CspUnsafeInlineScript
            | CspUnsafeEvalScript
            | CspWildcardScriptSource
            | CspUnsafeInlineStyle
            | CspWildcardSource
            | CspMissingDirective => "Content-Security-Policy",
            MissingXContentTypeOptions => "X-Content-Type-Options",
            MissingXFrameOptions => "X-Frame-Options",
            MissingHsts | HstsPreloadIneligible => "Strict-Transport-Security",
            MissingReferrerPolicy => "Referrer-Policy",
            MissingPermissionsPolicy | PermissionsPolicyPermissive => "Permissions-Policy",
            MissingCoop => "Cross-Origin-Opener-Policy",
            MissingCorp => "Cross-Origin-Resource-Policy",
            CorsWildcardCredentials => "Access-Control-Allow-Origin",
            PublicSourceMap => "Source Map",
        }
    }
}

/// Every `"{header}:{issue_type}"` id this build can emit — the canonical
/// inventory the non-WCAG detection corpus is checked against (#558).
///
/// Derived from [`SecurityIssueKind::ALL`], so it cannot drift from the code
/// the way the previous string-literal scan of this file could.
pub fn all_security_check_ids() -> Vec<String> {
    let mut ids = Vec::new();
    for kind in SecurityIssueKind::ALL {
        if kind == SecurityIssueKind::CspMissingDirective {
            for directive in CSP_REQUIRED_DIRECTIVES {
                let values = SecurityIssueValues {
                    csp_directive: Some(directive.to_string()),
                    ..Default::default()
                };
                ids.push(format!(
                    "{}:{}",
                    kind.header(),
                    issue_type_for(kind, &values)
                ));
            }
        } else {
            ids.push(format!(
                "{}:{}",
                kind.header(),
                issue_type_for(kind, &SecurityIssueValues::default())
            ));
        }
    }
    ids.sort();
    ids.dedup();
    ids
}

/// Values interpolated into a security message. Empty for the many findings
/// whose sentence carries no numbers.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecurityIssueValues {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hsts_max_age: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hsts_include_subdomains: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_map_count: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_map_example: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub csp_directive: Option<String>,
}

impl SecurityIssueValues {
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }
}

impl SecurityIssue {
    /// `None` for an `(header, issue_type)` pair this build does not know —
    /// callers then fall back to the stored canonical-English `message`.
    pub fn kind(&self) -> Option<SecurityIssueKind> {
        use SecurityIssueKind::*;
        let kind = match self.issue_type.as_str() {
            "missing_https" => MissingHttps,
            "missing_header" => match self.header.as_str() {
                "Content-Security-Policy" => MissingCsp,
                "X-Content-Type-Options" => MissingXContentTypeOptions,
                "X-Frame-Options" => MissingXFrameOptions,
                "Strict-Transport-Security" => MissingHsts,
                "Referrer-Policy" => MissingReferrerPolicy,
                "Permissions-Policy" => MissingPermissionsPolicy,
                "Cross-Origin-Opener-Policy" => MissingCoop,
                "Cross-Origin-Resource-Policy" => MissingCorp,
                _ => return None,
            },
            "cors_wildcard_credentials" => CorsWildcardCredentials,
            "hsts_preload_ineligible" => HstsPreloadIneligible,
            "permissions_policy_permissive" => PermissionsPolicyPermissive,
            "public_source_map" => PublicSourceMap,
            "unsafe_inline_script" => CspUnsafeInlineScript,
            "unsafe_eval_script" => CspUnsafeEvalScript,
            "wildcard_script_source" => CspWildcardScriptSource,
            "unsafe_inline_style" => CspUnsafeInlineStyle,
            "wildcard_source" => CspWildcardSource,
            other if other.starts_with("missing_") => CspMissingDirective,
            _ => return None,
        };
        Some(kind)
    }

    /// The finding's message in `locale`'s language. Falls back to the stored
    /// canonical English for kinds this build does not recognize.
    pub fn localized_message(&self, en: bool) -> String {
        match self.kind() {
            Some(kind) => security_issue_text(kind, &self.values, en),
            None => self.message.clone(),
        }
    }
}

/// The only source of security-finding wording (#406). The analysis layer calls
/// it with `en = true` to bake canonical English into the stored struct; the
/// PDF calls it with the run language.
pub fn security_issue_text(
    kind: SecurityIssueKind,
    values: &SecurityIssueValues,
    en: bool,
) -> String {
    use SecurityIssueKind::*;
    match kind {
        MissingHttps => if en {
            "Site is not served over HTTPS"
        } else {
            "Seite wird nicht über HTTPS ausgeliefert"
        }
        .to_string(),
        MissingCsp => if en {
            "Missing Content-Security-Policy header"
        } else {
            "Content-Security-Policy-Header fehlt"
        }
        .to_string(),
        MissingXContentTypeOptions => if en {
            "Missing X-Content-Type-Options header"
        } else {
            "X-Content-Type-Options-Header fehlt"
        }
        .to_string(),
        MissingXFrameOptions => if en {
            "Missing X-Frame-Options header (clickjacking protection)"
        } else {
            "X-Frame-Options-Header fehlt (Clickjacking-Schutz)"
        }
        .to_string(),
        MissingHsts => if en {
            "Missing HSTS header"
        } else {
            "HSTS-Header fehlt"
        }
        .to_string(),
        MissingReferrerPolicy => if en {
            "Missing Referrer-Policy header"
        } else {
            "Referrer-Policy-Header fehlt"
        }
        .to_string(),
        MissingPermissionsPolicy => if en {
            "Missing Permissions-Policy header"
        } else {
            "Permissions-Policy-Header fehlt"
        }
        .to_string(),
        MissingCoop => if en {
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
        }
        .to_string(),
        MissingCorp => if en {
            "Cross-Origin-Resource-Policy is not set. This header only matters if the page \
             serves fonts, scripts, or media that other origins should be prevented from \
             loading — if the site's resources are intentionally public, no action is needed \
             here. If cross-origin loading should be restricted, set it to same-origin or \
             same-site."
        } else {
            "Cross-Origin-Resource-Policy ist nicht gesetzt. Dieser Header ist nur relevant, \
             wenn die Seite Schriften, Skripte oder Medien ausliefert, die andere Origins nicht \
             laden können sollen — sind die Ressourcen bewusst öffentlich, ist hier keine \
             Maßnahme nötig. Soll Cross-Origin-Laden eingeschränkt werden: auf same-origin oder \
             same-site setzen."
        }
        .to_string(),
        CorsWildcardCredentials => if en {
            "CORS allows any origin while also allowing credentials"
        } else {
            "CORS erlaubt jede Origin und gleichzeitig Credentials"
        }
        .to_string(),
        HstsPreloadIneligible => {
            let max_age = values.hsts_max_age.as_deref().unwrap_or("unset");
            let subdomains = values.hsts_include_subdomains.unwrap_or(false);
            if en {
                format!(
                    "HSTS preload directive is set but requirements aren't met (max-age={max_age}, includeSubDomains={subdomains}) — the site won't qualify for the HSTS preload list"
                )
            } else {
                format!(
                    "HSTS-preload-Direktive ist gesetzt, die Voraussetzungen sind aber nicht erfüllt (max-age={max_age}, includeSubDomains={subdomains}) — die Seite wird nicht in die HSTS-Preload-Liste aufgenommen"
                )
            }
        }
        PermissionsPolicyPermissive => if en {
            "Permissions-Policy header is present but doesn't restrict any feature (empty or wildcard-only)"
        } else {
            "Permissions-Policy-Header ist vorhanden, schränkt aber kein Feature ein (leer oder nur Wildcards)"
        }
        .to_string(),
        PublicSourceMap => {
            let count = values.source_map_count.unwrap_or(1);
            let example = values
                .source_map_example
                .as_deref()
                .map(|url| format!(" (e.g. {url})"))
                .unwrap_or_default();
            if en {
                format!(
                    "{count} publicly reachable source map{} found{example} — exposes original source, comments, and internal file paths",
                    if count == 1 { "" } else { "s" }
                )
            } else {
                let example = example.replace(" (e.g. ", " (z. B. ");
                format!(
                    "{count} öffentlich erreichbare Source-Map{} gefunden{example} — legt Originalquellcode, Kommentare und interne Dateipfade offen",
                    if count == 1 { "" } else { "s" }
                )
            }
        }
        CspUnsafeInlineScript => if en {
            "CSP allows unsafe-inline scripts without nonce/hash protection"
        } else {
            "CSP erlaubt unsafe-inline-Skripte ohne Nonce-/Hash-Absicherung"
        }
        .to_string(),
        CspUnsafeEvalScript => if en {
            "CSP allows unsafe-eval in script sources"
        } else {
            "CSP erlaubt unsafe-eval in den Skript-Quellen"
        }
        .to_string(),
        CspWildcardScriptSource => if en {
            "CSP allows wildcard script sources"
        } else {
            "CSP erlaubt Wildcard-Skriptquellen"
        }
        .to_string(),
        CspUnsafeInlineStyle => if en {
            "CSP allows unsafe-inline styles without nonce/hash protection"
        } else {
            "CSP erlaubt unsafe-inline-Styles ohne Nonce-/Hash-Absicherung"
        }
        .to_string(),
        CspWildcardSource => if en {
            "CSP contains wildcard source expressions"
        } else {
            "CSP enthält Wildcard-Quellausdrücke"
        }
        .to_string(),
        CspMissingDirective => {
            let directive = values.csp_directive.as_deref().unwrap_or("object-src");
            if en {
                format!("CSP missing {directive} directive")
            } else {
                format!("CSP fehlt die {directive}-Direktive")
            }
        }
    }
}

/// The stored `issue_type` string for a kind. Kept as an explicit mapping so
/// the JSON contract stays byte-identical to what earlier builds wrote — see
/// `security_issue_types_match_the_stored_contract`.
fn issue_type_for(kind: SecurityIssueKind, values: &SecurityIssueValues) -> String {
    use SecurityIssueKind::*;
    match kind {
        MissingHttps => "missing_https".to_string(),
        MissingCsp
        | MissingXContentTypeOptions
        | MissingXFrameOptions
        | MissingHsts
        | MissingReferrerPolicy
        | MissingPermissionsPolicy
        | MissingCoop
        | MissingCorp => "missing_header".to_string(),
        CorsWildcardCredentials => "cors_wildcard_credentials".to_string(),
        HstsPreloadIneligible => "hsts_preload_ineligible".to_string(),
        PermissionsPolicyPermissive => "permissions_policy_permissive".to_string(),
        PublicSourceMap => "public_source_map".to_string(),
        CspUnsafeInlineScript => "unsafe_inline_script".to_string(),
        CspUnsafeEvalScript => "unsafe_eval_script".to_string(),
        CspWildcardScriptSource => "wildcard_script_source".to_string(),
        CspUnsafeInlineStyle => "unsafe_inline_style".to_string(),
        CspWildcardSource => "wildcard_source".to_string(),
        CspMissingDirective => format!(
            "missing_{}",
            values.csp_directive.as_deref().unwrap_or("object-src")
        ),
    }
}

/// Build a finding with its canonical-English message derived from `kind` —
/// the message is never written by hand at a call site (#406).
fn security_issue(
    header: &str,
    kind: SecurityIssueKind,
    values: SecurityIssueValues,
    severity: Severity,
    tier: HeaderTier,
) -> SecurityIssue {
    SecurityIssue {
        header: header.to_string(),
        issue_type: issue_type_for(kind, &values),
        message: security_issue_text(kind, &values, true),
        severity,
        tier,
        values,
        score_deduction: 0,
    }
}

/// Analyze security headers of a URL
pub async fn analyze_security(
    url: &str,
    log: &crate::audit::collection::CollectionLog,
) -> Result<SecurityAnalysis> {
    analyze_security_trusting(url, log, &[]).await
}

/// [`analyze_security`] with additional trust anchors for its HTTPS
/// requests. The roots are added to reqwest's built-in set; certificate,
/// chain and hostname validation stay fully on. Exists so the detection
/// corpus can serve a real HTTPS fixture signed by a corpus-only CA (#783);
/// the audit itself always calls [`analyze_security`], i.e. with none.
#[doc(hidden)]
pub async fn analyze_security_trusting(
    url: &str,
    log: &crate::audit::collection::CollectionLog,
    extra_roots: &[reqwest::Certificate],
) -> Result<SecurityAnalysis> {
    info!("Analyzing security headers for {}...", url);

    let mut builder = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .danger_accept_invalid_certs(false);
    for root in extra_roots {
        builder = builder.add_root_certificate(root.clone());
    }
    let client = builder.build().map_err(AuditError::HttpError)?;
    let (response_headers, fetch) = log
        .record(&HEADERS, fetch_headers(&client, url).await)
        .into_result(&HEADERS)?;
    // The transport the headers were served over, after redirects (#775).
    let https = fetch.final_url.starts_with("https://");
    let headers = extract_security_headers(&response_headers);
    let protection = detect_protection(&response_headers);

    let ssl = analyze_ssl(https, &headers);

    // Public source-map leak check (#538)
    let sourcemap_leaks = log
        .record(
            &SOURCE_MAPS,
            sourcemap::audit_source_maps_trusting(url, extra_roots).await,
        )
        .into_value()
        .unwrap_or_default();
    let Assessment {
        issues,
        recommendations,
        score,
        grade,
    } = assess(&headers, &ssl, &sourcemap_leaks);

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
        fetch,
    })
}

/// Everything the security score derives from the headers, the transport and
/// the source-map audit; pure, so the transport can be re-judged later.
struct Assessment {
    issues: Vec<SecurityIssue>,
    recommendations: Vec<String>,
    score: u32,
    grade: String,
}

fn assess(
    headers: &SecurityHeaders,
    ssl: &SslInfo,
    sourcemap_leaks: &SourceMapLeakAudit,
) -> Assessment {
    // Issues
    let mut issues = generate_security_issues(headers, ssl.https);

    // HSTS preload eligibility + Permissions-Policy quality (#535)
    if hsts_preload_ineligible(ssl) {
        issues.push(security_issue(
            "Strict-Transport-Security",
            SecurityIssueKind::HstsPreloadIneligible,
            SecurityIssueValues {
                hsts_max_age: Some(
                    ssl.hsts_max_age
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "unset".to_string()),
                ),
                hsts_include_subdomains: Some(ssl.hsts_include_subdomains),
                ..Default::default()
            },
            Severity::Low,
            HeaderTier::Baseline,
        ));
    }
    if permissions_policy_is_permissive(headers) {
        issues.push(security_issue(
            "Permissions-Policy",
            SecurityIssueKind::PermissionsPolicyPermissive,
            SecurityIssueValues::default(),
            Severity::Low,
            HeaderTier::ArchitectureDependent,
        ));
    }

    if !sourcemap_leaks.leaks.is_empty() {
        issues.push(security_issue(
            "Source Map",
            SecurityIssueKind::PublicSourceMap,
            SecurityIssueValues {
                source_map_count: Some(sourcemap_leaks.leaks.len()),
                source_map_example: Some(sourcemap_leaks.leaks[0].map_url.clone()),
                ..Default::default()
            },
            Severity::High,
            HeaderTier::Baseline,
        ));
    }

    // Generate recommendations
    let recommendations = generate_recommendations(headers, ssl.https);

    // Calculate score
    let score = calculate_security_score(headers, ssl, &issues);
    let deductions = issue_deductions(&issues);
    for (issue, points) in issues.iter_mut().zip(deductions) {
        issue.score_deduction = points;
    }
    let grade = calculate_grade(score);
    Assessment {
        issues,
        recommendations,
        score,
        grade,
    }
}

/// Judges the headers again under the transport the browser's document used
/// (#775). The probe fetches on its own and may end on another scheme than the
/// page the browser loaded (an `https://` entry that dropped to `http://` and
/// came back); the document's transport decides, not the probe's.
pub(crate) fn align_with_document_transport(analysis: &mut SecurityAnalysis, document_https: bool) {
    if analysis.ssl.https == document_https {
        return;
    }
    analysis.ssl.https = document_https;
    analysis.ssl.valid_certificate = document_https && analysis.ssl.certificate_error.is_none();
    let Assessment {
        issues,
        recommendations,
        score,
        grade,
    } = assess(&analysis.headers, &analysis.ssl, &analysis.sourcemap_leaks);
    analysis.issues = issues;
    analysis.recommendations = recommendations;
    analysis.score = score;
    analysis.grade = grade;
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
        // Every Content-Security-Policy header is enforced; several headers
        // are one comma-separated policy list (CSP3 §2.2), so join them
        // instead of reading only the first (#723).
        content_security_policy: {
            let policies: Vec<&str> = headers
                .get_all("content-security-policy")
                .iter()
                .filter_map(|v| v.to_str().ok())
                .collect();
            (!policies.is_empty()).then(|| policies.join(", "))
        },
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

    let (hsts_max_age, hsts_include_subdomains, hsts_preload) = match hsts {
        Some(hsts_value) => parse_hsts_directives(hsts_value),
        None => (None, false, false),
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

/// `max-age`, `includeSubDomains` and `preload` of an STS header value.
/// Directive names are case-insensitive and a value may be a quoted-string
/// (RFC 6797 §6.1), so `Max-Age="31536000"` is one year like `max-age=31536000`.
fn parse_hsts_directives(value: &str) -> (Option<u64>, bool, bool) {
    let mut max_age = None;
    let mut include_subdomains = false;
    let mut preload = false;
    for directive in value.split(';') {
        let (name, directive_value) = match directive.split_once('=') {
            Some((name, v)) => (name.trim(), Some(v.trim().trim_matches('"'))),
            None => (directive.trim(), None),
        };
        if name.eq_ignore_ascii_case("max-age") {
            max_age = directive_value.and_then(|v| v.parse().ok());
        } else if name.eq_ignore_ascii_case("includesubdomains") {
            include_subdomains = true;
        } else if name.eq_ignore_ascii_case("preload") {
            preload = true;
        }
    }
    (max_age, include_subdomains, preload)
}

pub(crate) fn generate_security_issues(
    headers: &SecurityHeaders,
    https: bool,
) -> Vec<SecurityIssue> {
    let mut issues = Vec::new();

    if !https {
        issues.push(security_issue(
            "HTTPS",
            SecurityIssueKind::MissingHttps,
            SecurityIssueValues::default(),
            Severity::Critical,
            HeaderTier::Baseline,
        ));
    }

    if headers.content_security_policy.is_none() {
        issues.push(security_issue(
            "Content-Security-Policy",
            SecurityIssueKind::MissingCsp,
            SecurityIssueValues::default(),
            Severity::High,
            HeaderTier::Baseline,
        ));
    } else if let Some(ref csp) = headers.content_security_policy {
        issues.extend(collect_csp_quality_issues(csp));
    }

    if headers.x_content_type_options.is_none() {
        issues.push(security_issue(
            "X-Content-Type-Options",
            SecurityIssueKind::MissingXContentTypeOptions,
            SecurityIssueValues::default(),
            Severity::Medium,
            HeaderTier::Baseline,
        ));
    }

    // A CSP `frame-ancestors` supersedes X-Frame-Options (CSP Level 2;
    // browsers ignore XFO when both are set) — missing XFO is no gap then
    // (plan 72).
    if headers.x_frame_options.is_none()
        && !headers
            .content_security_policy
            .as_deref()
            .is_some_and(csp_sets_frame_ancestors)
    {
        issues.push(security_issue(
            "X-Frame-Options",
            SecurityIssueKind::MissingXFrameOptions,
            SecurityIssueValues::default(),
            Severity::Medium,
            HeaderTier::Baseline,
        ));
    }

    if https && headers.strict_transport_security.is_none() {
        issues.push(security_issue(
            "Strict-Transport-Security",
            SecurityIssueKind::MissingHsts,
            SecurityIssueValues::default(),
            Severity::High,
            HeaderTier::Baseline,
        ));
    }

    if headers.referrer_policy.is_none() {
        issues.push(security_issue(
            "Referrer-Policy",
            SecurityIssueKind::MissingReferrerPolicy,
            SecurityIssueValues::default(),
            Severity::Low,
            HeaderTier::ArchitectureDependent,
        ));
    }

    if headers.permissions_policy.is_none() {
        issues.push(security_issue(
            "Permissions-Policy",
            SecurityIssueKind::MissingPermissionsPolicy,
            SecurityIssueValues::default(),
            Severity::Low,
            HeaderTier::ArchitectureDependent,
        ));
    }

    // COOP/CORP relevance depends on deployment context this tool cannot
    // observe from a single unauthenticated fetch — a marketing page almost
    // never needs them, but an authenticated app page using SharedArrayBuffer
    // or a cross-origin OAuth popup flow genuinely might. A static crawl has
    // no way to tell those two cases apart, so the message below asks the
    // verification question instead of asserting either "add this" or
    // "ignore this" (#578).
    if headers.cross_origin_opener_policy.is_none() {
        issues.push(security_issue(
            "Cross-Origin-Opener-Policy",
            SecurityIssueKind::MissingCoop,
            SecurityIssueValues::default(),
            Severity::Low,
            HeaderTier::ContextDependent,
        ));
    }

    if headers.cross_origin_resource_policy.is_none() {
        issues.push(security_issue(
            "Cross-Origin-Resource-Policy",
            SecurityIssueKind::MissingCorp,
            SecurityIssueValues::default(),
            Severity::Low,
            HeaderTier::ContextDependent,
        ));
    }

    if cors_allows_wildcard_credentials(headers) {
        issues.push(security_issue(
            "Access-Control-Allow-Origin",
            SecurityIssueKind::CorsWildcardCredentials,
            SecurityIssueValues::default(),
            Severity::High,
            HeaderTier::Baseline,
        ));
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
    // Over plain HTTP the header is ignored (RFC 6797 §8.1): nothing to judge.
    ssl.https
        && ssl.hsts_preload
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

/// CSP quality findings for a `Content-Security-Policy` value, parsed with
/// `csp-parse` (#723). The value may be a policy list (several headers or a
/// comma-separated value). Every policy in it is enforced and a resource
/// loads only if all of them allow it, so a weakness counts only when every
/// policy has it.
fn collect_csp_quality_issues(policy: &str) -> Vec<SecurityIssue> {
    let list = csp_parse::parse_policy_list(policy);
    let mut per_policy = list
        .policies
        .iter()
        .filter(|p| !p.directives.is_empty())
        .map(policy_quality_issues);
    let Some(mut issues) = per_policy.next() else {
        // Nothing but separators: as weak as an empty policy.
        return policy_quality_issues(&csp_parse::Policy::default());
    };
    for other in per_policy {
        issues.retain(|issue| other.iter().any(|o| o.issue_type == issue.issue_type));
    }
    issues
}

/// Whether some policy in the CSP value sets `frame-ancestors`.
fn csp_sets_frame_ancestors(policy: &str) -> bool {
    csp_parse::parse_policy_list(policy)
        .policies
        .iter()
        .any(|p| csp_directives(p).contains_key("frame-ancestors"))
}

fn policy_quality_issues(policy: &csp_parse::Policy) -> Vec<SecurityIssue> {
    let directives = csp_directives(policy);
    let mut issues = Vec::new();

    let effective_script = directives
        .get("script-src")
        .or_else(|| directives.get("default-src"));
    if effective_script.is_some_and(|list| {
        has_keyword(list, csp_parse::Keyword::UnsafeInline) && !has_nonce_or_hash(list)
    }) {
        issues.push(csp_issue(
            SecurityIssueKind::CspUnsafeInlineScript,
            SecurityIssueValues::default(),
            Severity::High,
        ));
    }
    if effective_script.is_some_and(|list| has_keyword(list, csp_parse::Keyword::UnsafeEval)) {
        issues.push(csp_issue(
            SecurityIssueKind::CspUnsafeEvalScript,
            SecurityIssueValues::default(),
            Severity::High,
        ));
    }
    if effective_script.is_some_and(has_wildcard_source) {
        issues.push(csp_issue(
            SecurityIssueKind::CspWildcardScriptSource,
            SecurityIssueValues::default(),
            Severity::High,
        ));
    }

    let effective_style = directives
        .get("style-src")
        .or_else(|| directives.get("default-src"));
    if effective_style.is_some_and(|list| {
        has_keyword(list, csp_parse::Keyword::UnsafeInline) && !has_nonce_or_hash(list)
    }) {
        issues.push(csp_issue(
            SecurityIssueKind::CspUnsafeInlineStyle,
            SecurityIssueValues::default(),
            Severity::Medium,
        ));
    }

    if directives.values().any(has_wildcard_source) {
        issues.push(csp_issue(
            SecurityIssueKind::CspWildcardSource,
            SecurityIssueValues::default(),
            Severity::Medium,
        ));
    }

    let declares = |name: &str| {
        policy
            .directives
            .iter()
            .any(|d| d.name.eq_ignore_ascii_case(name))
    };
    for directive in CSP_REQUIRED_DIRECTIVES {
        // `object-src` falls back to `default-src` (CSP3 §6.1.1); `base-uri`
        // and `frame-ancestors` have no fallback.
        let covered = declares(directive) || (directive == "object-src" && declares("default-src"));
        if !covered {
            issues.push(csp_issue(
                SecurityIssueKind::CspMissingDirective,
                SecurityIssueValues {
                    csp_directive: Some(directive.to_string()),
                    ..Default::default()
                },
                Severity::Medium,
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

fn csp_issue(
    kind: SecurityIssueKind,
    values: SecurityIssueValues,
    severity: Severity,
) -> SecurityIssue {
    security_issue(
        "Content-Security-Policy",
        kind,
        values,
        severity,
        HeaderTier::Baseline,
    )
}

/// The source-list directives of one policy, by lower-cased name. A
/// repeated directive is ignored after its first occurrence (CSP3 §2.2.1).
fn csp_directives(policy: &csp_parse::Policy) -> BTreeMap<String, csp_parse::SourceList> {
    let mut directives = BTreeMap::new();
    for directive in &policy.directives {
        let list = match directive.value() {
            csp_parse::DirectiveValue::SourceList(list)
            | csp_parse::DirectiveValue::AncestorSourceList(list) => list,
            _ => continue,
        };
        directives
            .entry(directive.name.to_ascii_lowercase())
            .or_insert(list);
    }
    directives
}

fn source_expressions(
    list: &csp_parse::SourceList,
) -> impl Iterator<Item = &csp_parse::SourceExpression> {
    let entries: &[csp_parse::SourceListEntry] = match list {
        csp_parse::SourceList::Sources(entries) => entries,
        _ => &[],
    };
    entries.iter().filter_map(|entry| entry.expression.as_ref())
}

fn has_keyword(list: &csp_parse::SourceList, keyword: csp_parse::Keyword) -> bool {
    source_expressions(list)
        .any(|e| matches!(e, csp_parse::SourceExpression::Keyword(k) if *k == keyword))
}

fn has_nonce_or_hash(list: &csp_parse::SourceList) -> bool {
    source_expressions(list).any(|e| {
        matches!(
            e,
            csp_parse::SourceExpression::Nonce(_) | csp_parse::SourceExpression::Hash(_)
        )
    })
}

/// `*`, `*.example.com` and `https://*` alike: any host source whose host
/// part is a wildcard.
fn has_wildcard_source(list: &csp_parse::SourceList) -> bool {
    source_expressions(list).any(|e| {
        matches!(
            e,
            csp_parse::SourceExpression::Host(csp_parse::HostSource {
                host: csp_parse::HostPart::AnyHost
                    | csp_parse::HostPart::Named {
                        wildcard_prefix: true,
                        ..
                    },
                ..
            })
        )
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

/// Points each issue takes off the security score, in `issues` order — the
/// one contribution list behind `calculate_security_score` and
/// [`calculate_security_category_scores`] (plan 72).
///
/// - Context-dependent and not-assessable headers cost nothing: whether
///   COOP/CORP or CORS matter depends on a deployment context a page fetch
///   cannot establish, so their absence is no confirmed gap.
/// - A *present* CSP (even a permissive one) provides partial protection, so
///   the CSP-quality issues together never cost more than a *missing* CSP
///   (High = 15) — otherwise an imperfect CSP would score worse than none.
pub fn issue_deductions(issues: &[SecurityIssue]) -> Vec<u32> {
    const MISSING_CSP_PENALTY: u32 = 15;
    let mut csp_quality_left = MISSING_CSP_PENALTY;
    issues
        .iter()
        .map(|issue| {
            if matches!(
                header_tier(&issue.header),
                HeaderTier::ContextDependent | HeaderTier::NotAssessable
            ) {
                return 0;
            }
            let points = match issue.severity {
                Severity::Critical => 25,
                Severity::High => 15,
                Severity::Medium => 10,
                Severity::Low => 5,
            };
            if issue.header == "Content-Security-Policy" && issue.issue_type != "missing_header" {
                let capped = points.min(csp_quality_left);
                csp_quality_left -= capped;
                capped
            } else {
                points
            }
        })
        .collect()
}

pub(crate) fn calculate_security_score(
    _headers: &SecurityHeaders,
    ssl: &SslInfo,
    issues: &[SecurityIssue],
) -> u32 {
    let deducted: u32 = issue_deductions(issues).iter().sum();
    let mut score = 100u32.saturating_sub(deducted);

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
    let deductions = issue_deductions(issues);
    SecurityCategory::all()
        .into_iter()
        .map(|category| {
            let lost: u32 = issues
                .iter()
                .zip(&deductions)
                .filter(|(issue, _)| SecurityCategory::for_header(&issue.header) == category)
                .map(|(_, points)| points)
                .sum();
            (category, 100u32.saturating_sub(lost))
        })
        .collect()
}

fn calculate_grade(score: u32) -> String {
    crate::registry::SECURITY_GRADE
        .label(score as f32, false)
        .to_string()
}

/// Counter-cases of the security collectors (#774, #775), against a local
/// server: refused connection, timeout, `HEAD` rejected with 405, 403 and a
/// redirect — none of them may turn into analysed headers except the
/// successful `GET` fallback and the redirect target.
#[cfg(test)]
pub(crate) fn counter_cases() {
    use crate::audit::collection::{Collected, CollectionState};
    use std::io::{Read, Write};

    /// Answers `(method, path)` with a raw response; `None` hangs.
    fn serve(routes: Vec<(&'static str, &'static str, Option<String>)>) -> String {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let base = format!("http://127.0.0.1:{}", listener.local_addr().unwrap().port());
        std::thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let routes = routes.clone();
                std::thread::spawn(move || {
                    let mut stream = stream;
                    let mut buf = [0u8; 2048];
                    let n = stream.read(&mut buf).unwrap_or(0);
                    let request = String::from_utf8_lossy(&buf[..n]).to_string();
                    let mut parts = request.split_whitespace();
                    let (method, path) = (parts.next().unwrap_or(""), parts.next().unwrap_or(""));
                    match routes
                        .iter()
                        .find(|(m, p, _)| *m == method && *p == path)
                        .map(|(_, _, r)| r.clone())
                    {
                        Some(Some(response)) => {
                            let _ = stream.write_all(response.as_bytes());
                        }
                        Some(None) => std::thread::sleep(std::time::Duration::from_secs(5)),
                        None => {
                            let _ = stream
                                .write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n");
                        }
                    }
                });
            }
        });
        base
    }
    fn status(code: u16, extra: &str) -> Option<String> {
        Some(format!(
            "HTTP/1.1 {code} X\r\nContent-Length: 0\r\nConnection: close\r\n{extra}\r\n"
        ))
    }
    /// A timeout long enough for a loaded test machine; the hanging server
    /// gets a short one of its own.
    fn fetch(url: &str) -> Collected<(HeaderMap, HeaderFetch)> {
        fetch_within(url, std::time::Duration::from_secs(5))
    }
    fn fetch_within(
        url: &str,
        timeout: std::time::Duration,
    ) -> Collected<(HeaderMap, HeaderFetch)> {
        let client = reqwest::Client::builder().timeout(timeout).build().unwrap();
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(fetch_headers(&client, url))
    }
    let reason = |c: &Collected<(HeaderMap, HeaderFetch)>| match c {
        Collected::Failed(r) => *r,
        other => panic!("expected a failure, got {:?}", other.state()),
    };

    // Refused connection and timeout: no headers, no score.
    let refused = fetch("http://127.0.0.1:9/");
    assert_eq!(reason(&refused), "http_connect_failed");
    let hanging = serve(vec![("HEAD", "/", None), ("GET", "/", None)]);
    assert_eq!(
        reason(&fetch_within(
            &format!("{hanging}/"),
            std::time::Duration::from_millis(300)
        )),
        "http_timeout"
    );

    // HEAD rejected with 405: the GET answer is analysed, and says so.
    let head_rejected = serve(vec![
        ("HEAD", "/", status(405, "")),
        ("GET", "/", status(200, "X-Frame-Options: DENY\r\n")),
    ]);
    let collected = fetch(&format!("{head_rejected}/"));
    assert_eq!(collected.state(), CollectionState::Measured);
    let (headers, provenance) = collected.into_value().unwrap();
    assert_eq!(headers.get("x-frame-options").unwrap(), "DENY");
    assert_eq!(provenance.method, "GET");
    assert_eq!(provenance.head_status, Some(405));
    assert_eq!(provenance.source, "http_probe");

    // 403 (a WAF page): its headers are not the site's.
    let forbidden = serve(vec![
        ("HEAD", "/", status(403, "")),
        ("GET", "/", status(403, "")),
    ]);
    assert_eq!(
        reason(&fetch(&format!("{forbidden}/"))),
        "http_error_status"
    );

    // Redirect: the final URL is recorded; its scheme is the transport.
    let redirect = serve(vec![
        ("HEAD", "/", status(301, "Location: /final\r\n")),
        ("HEAD", "/final", status(200, "")),
    ]);
    let (_, provenance) = fetch(&format!("{redirect}/")).into_value().unwrap();
    assert!(provenance.final_url.ends_with("/final"), "{provenance:?}");
    assert_eq!(provenance.method, "HEAD");

    // The probe's document vs. the browser's.
    let probe = HeaderFetch {
        final_url: "http://example.com/".into(),
        ..Default::default()
    };
    assert_eq!(
        document_match(&probe, Some("https://example.com/")).state(),
        CollectionState::Failed
    );
    assert_eq!(
        document_match(&probe, Some("http://example.com/other")).state(),
        CollectionState::Measured
    );
    assert_eq!(
        document_match(&probe, None).state(),
        CollectionState::Failed
    );
    // Another port is another server (#774): the headers are not the
    // browser's. The default port spelled out is the same origin.
    let local = HeaderFetch {
        final_url: "http://127.0.0.1:8080/".into(),
        ..Default::default()
    };
    assert_eq!(
        document_match(&local, Some("http://127.0.0.1:9090/")).state(),
        CollectionState::Failed
    );
    assert_eq!(
        document_match(&probe, Some("http://example.com:80/")).state(),
        CollectionState::Measured
    );

    // Source maps: unreachable assets make the check partial or failed.
    let audit = SourceMapLeakAudit {
        assets_checked: 2,
        leaks: vec![],
    };
    assert_eq!(
        sourcemap::sourcemap_outcome_for_tests(audit.clone(), 1).state(),
        CollectionState::Partial
    );
    assert_eq!(
        sourcemap::sourcemap_outcome_for_tests(SourceMapLeakAudit::default(), 3).state(),
        CollectionState::Failed
    );
    assert_eq!(
        sourcemap::sourcemap_outcome_for_tests(audit, 0).state(),
        CollectionState::Measured
    );
}

#[cfg(test)]
mod tests {
    fn analysis_over(https: bool) -> SecurityAnalysis {
        let headers = SecurityHeaders::default();
        let ssl = analyze_ssl(https, &headers);
        let leaks = SourceMapLeakAudit::default();
        let Assessment {
            issues,
            recommendations,
            score,
            grade,
        } = assess(&headers, &ssl, &leaks);
        SecurityAnalysis {
            score,
            grade,
            headers,
            ssl,
            issues,
            recommendations,
            protection: ProtectionDetection::default(),
            sourcemap_leaks: leaks,
            fetch: HeaderFetch::default(),
        }
    }

    /// The probe ended on `https://`, but the browser's document passed through
    /// cleartext: the analysis follows the document and says so (#775).
    #[test]
    fn the_document_transport_overrides_the_probe_transport() {
        let mut analysis = analysis_over(true);
        let without_https_issue = analysis.issues.iter().all(|i| i.header != "HTTPS");
        assert!(without_https_issue);
        let score_over_tls = analysis.score;

        align_with_document_transport(&mut analysis, false);

        assert!(!analysis.ssl.https && !analysis.ssl.valid_certificate);
        assert!(analysis.issues.iter().any(|i| i.header == "HTTPS"));
        assert!(
            analysis.score < score_over_tls,
            "{} !< {score_over_tls}",
            analysis.score
        );
        let fresh = analysis_over(false);
        assert_eq!(analysis.score, fresh.score);
        assert_eq!(analysis.issues.len(), fresh.issues.len());
    }

    #[test]
    fn a_matching_document_transport_changes_nothing() {
        let mut analysis = analysis_over(true);
        let before = (analysis.score, analysis.issues.len());
        align_with_document_transport(&mut analysis, true);
        assert_eq!((analysis.score, analysis.issues.len()), before);
    }

    use super::*;

    /// Every kind this build knows must render differently in the two
    /// languages, and the English side must stay free of German characters —
    /// the #406 guard every localized module carries.
    #[test]
    fn every_security_message_is_localized_and_english_stays_english() {
        let values = SecurityIssueValues {
            hsts_max_age: Some("100".to_string()),
            hsts_include_subdomains: Some(false),
            source_map_count: Some(3),
            source_map_example: Some("https://example.com/app.js.map".to_string()),
            csp_directive: Some("object-src".to_string()),
        };
        for kind in SecurityIssueKind::ALL {
            let en = security_issue_text(kind, &values, true);
            let de = security_issue_text(kind, &values, false);
            assert!(!en.is_empty(), "{kind:?}: empty English text");
            assert_ne!(en, de, "{kind:?}: German text must differ from English");
            assert!(
                !en.chars().any(|c| "äöüÄÖÜß".contains(c)),
                "{kind:?} English text leaks German: {en}"
            );
        }
    }

    /// The generated `issue_type` strings are part of the JSON contract and
    /// must not drift when the kind enum is refactored.
    #[test]
    fn security_issue_types_match_the_stored_contract() {
        use SecurityIssueKind::*;
        let empty = SecurityIssueValues::default();
        for (kind, expected) in [
            (MissingHttps, "missing_https"),
            (MissingCsp, "missing_header"),
            (MissingCoop, "missing_header"),
            (CorsWildcardCredentials, "cors_wildcard_credentials"),
            (HstsPreloadIneligible, "hsts_preload_ineligible"),
            (PermissionsPolicyPermissive, "permissions_policy_permissive"),
            (PublicSourceMap, "public_source_map"),
            (CspUnsafeInlineScript, "unsafe_inline_script"),
            (CspUnsafeEvalScript, "unsafe_eval_script"),
            (CspWildcardScriptSource, "wildcard_script_source"),
            (CspUnsafeInlineStyle, "unsafe_inline_style"),
            (CspWildcardSource, "wildcard_source"),
        ] {
            assert_eq!(issue_type_for(kind, &empty), expected, "{kind:?}");
        }
        assert_eq!(
            issue_type_for(
                CspMissingDirective,
                &SecurityIssueValues {
                    csp_directive: Some("frame-ancestors".to_string()),
                    ..Default::default()
                }
            ),
            "missing_frame-ancestors"
        );
    }

    /// `kind()` is derived from the stored `header`/`issue_type` pair, so a
    /// report written by an older build still localizes.
    #[test]
    fn kind_round_trips_through_the_stored_fields() {
        let issue = security_issue(
            "Cross-Origin-Opener-Policy",
            SecurityIssueKind::MissingCoop,
            SecurityIssueValues::default(),
            Severity::Low,
            HeaderTier::ContextDependent,
        );
        assert_eq!(issue.kind(), Some(SecurityIssueKind::MissingCoop));
        assert_eq!(issue.message, issue.localized_message(true));
        assert_ne!(issue.message, issue.localized_message(false));
    }

    /// An issue_type this build does not know falls back to the stored
    /// canonical-English message instead of inventing a sentence.
    #[test]
    fn unknown_issue_type_falls_back_to_the_stored_message() {
        let issue = SecurityIssue {
            header: "X-Frame-Options".to_string(),
            issue_type: "something_this_build_does_not_know".to_string(),
            message: "stored message".to_string(),
            severity: Severity::Low,
            tier: HeaderTier::Baseline,
            values: SecurityIssueValues::default(),
            score_deduction: 0,
        };
        assert_eq!(issue.kind(), None);
        assert_eq!(issue.localized_message(false), "stored message");
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
            values: Default::default(),
            score_deduction: 0,
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
            values: Default::default(),
            score_deduction: 0,
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
            https: true,
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
            https: true,
            hsts_preload: true,
            hsts_max_age: Some(31536000),
            hsts_include_subdomains: false,
            ..Default::default()
        };
        assert!(hsts_preload_ineligible(&ssl));
    }

    #[test]
    fn hsts_preload_over_plain_http_is_not_judged() {
        let ssl = SslInfo {
            https: false,
            hsts_preload: true,
            hsts_max_age: Some(3600),
            ..Default::default()
        };
        assert!(!hsts_preload_ineligible(&ssl));
    }

    #[test]
    fn hsts_directives_are_case_insensitive_and_may_be_quoted() {
        assert_eq!(
            parse_hsts_directives("Max-Age=\"31536000\" ; INCLUDESUBDOMAINS; Preload"),
            (Some(31536000), true, true)
        );
        assert_eq!(
            parse_hsts_directives("max-age=60"),
            (Some(60), false, false)
        );
    }

    #[test]
    fn hsts_preload_eligible_is_not_flagged() {
        let ssl = SslInfo {
            https: true,
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
            https: true,
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
        // object-src falls back to `default-src *`: not missing, the
        // wildcard is wildcard_source's finding.
        assert!(!issues
            .iter()
            .any(|issue| issue.issue_type == "missing_object-src"));
        assert!(issues
            .iter()
            .any(|issue| issue.issue_type == "wildcard_source"));
    }

    #[test]
    fn csp_object_src_falls_back_to_default_src() {
        let types = |policy: &str| -> Vec<String> {
            collect_csp_quality_issues(policy)
                .into_iter()
                .map(|i| i.issue_type)
                .collect()
        };
        let with_default = types("default-src 'self'; base-uri 'self'; frame-ancestors 'none'");
        assert!(!with_default.iter().any(|t| t == "missing_object-src"));
        let without = types("script-src 'self'; base-uri 'self'; frame-ancestors 'none'");
        assert!(without.iter().any(|t| t == "missing_object-src"));
    }

    #[test]
    fn test_csp_quality_accepts_nonce_hardened_policy() {
        let issues = collect_csp_quality_issues(
            "default-src 'self'; script-src 'self' 'nonce-abc123'; style-src 'self' 'sha256-abc'; object-src 'none'; base-uri 'self'; frame-ancestors 'none'",
        );

        assert!(issues.is_empty());
    }

    #[test]
    fn csp_policy_list_counts_a_weakness_only_when_every_policy_has_it() {
        // #723: two enforced policies; the second forbids inline script and
        // sets object-src, so neither weakness reaches the page.
        let issues = collect_csp_quality_issues(
            "script-src 'unsafe-inline' 'unsafe-eval'; base-uri 'self', \
             script-src 'self' 'unsafe-eval'; object-src 'none'",
        );
        let types: Vec<&str> = issues.iter().map(|i| i.issue_type.as_str()).collect();
        assert!(!types.contains(&"unsafe_inline_script"), "{types:?}");
        assert!(!types.contains(&"missing_object-src"), "{types:?}");
        assert!(!types.contains(&"missing_base-uri"), "{types:?}");
        assert!(types.contains(&"unsafe_eval_script"), "{types:?}");
        assert!(types.contains(&"missing_frame-ancestors"), "{types:?}");
    }

    #[test]
    fn csp_repeated_directive_keeps_its_first_value() {
        // CSP3: a repeated directive is ignored after its first occurrence.
        let issues = collect_csp_quality_issues(
            "script-src 'self'; SCRIPT-SRC 'unsafe-inline'; object-src 'none'; base-uri 'self'; frame-ancestors 'none'",
        );
        assert!(issues.is_empty(), "{issues:?}");
    }

    #[test]
    fn csp_headers_are_joined_into_one_policy_list() {
        let mut headers = HeaderMap::new();
        headers.append(
            "content-security-policy",
            "script-src 'unsafe-inline'".parse().unwrap(),
        );
        headers.append(
            "content-security-policy",
            "object-src 'none'".parse().unwrap(),
        );
        let csp = extract_security_headers(&headers).content_security_policy;
        assert_eq!(
            csp.as_deref(),
            Some("script-src 'unsafe-inline', object-src 'none'")
        );
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

    /// Plan 72, inros-lackner 2026-10-23: four baseline gaps, two
    /// architecture-dependent and two context-dependent headers missing. COOP
    /// and CORP now cost nothing (30 → 40); the category scores come from the
    /// same contribution list and lose exactly what the total loses.
    #[test]
    fn context_dependent_headers_cost_nothing_and_categories_add_up() {
        let ssl = SslInfo {
            https: true,
            valid_certificate: true,
            ..Default::default()
        };
        let headers = SecurityHeaders::default();
        let issues = generate_security_issues(&headers, true);
        let deductions = issue_deductions(&issues);
        for (issue, points) in issues.iter().zip(&deductions) {
            if matches!(
                issue.header.as_str(),
                "Cross-Origin-Opener-Policy" | "Cross-Origin-Resource-Policy"
            ) {
                assert_eq!(*points, 0, "{}", issue.header);
            }
        }
        let score = calculate_security_score(&headers, &ssl, &issues);
        assert_eq!(score, 40);
        let lost_in_categories: u32 = calculate_security_category_scores(&issues)
            .iter()
            .map(|(_, s)| 100 - s)
            .sum();
        assert_eq!(lost_in_categories, 100 - score);
    }

    /// A CSP with frame-ancestors supersedes X-Frame-Options: no second
    /// charge for the same protection.
    #[test]
    fn frame_ancestors_replaces_x_frame_options() {
        let with = SecurityHeaders {
            content_security_policy: Some("default-src 'self'; frame-ancestors 'self'".to_string()),
            ..Default::default()
        };
        let without = SecurityHeaders {
            content_security_policy: Some("default-src 'self'".to_string()),
            ..Default::default()
        };
        let xfo = |h: &SecurityHeaders| {
            generate_security_issues(h, true)
                .iter()
                .any(|i| i.header == "X-Frame-Options")
        };
        assert!(!xfo(&with));
        assert!(xfo(&without));
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
