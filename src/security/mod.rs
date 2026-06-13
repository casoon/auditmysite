//! Security headers analysis module
//!
//! Analyzes HTTP security headers and SSL/TLS configuration.

pub mod module;
pub use module::SecurityModule;

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
}

/// Security issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityIssue {
    pub header: String,
    pub issue_type: String,
    pub message: String,
    pub severity: Severity,
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
    let issues = generate_security_issues(&headers, https);

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
        });
    }

    if headers.content_security_policy.is_none() {
        issues.push(SecurityIssue {
            header: "Content-Security-Policy".to_string(),
            issue_type: "missing_header".to_string(),
            message: "Missing Content-Security-Policy header".to_string(),
            severity: Severity::High,
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
        });
    }

    if headers.x_frame_options.is_none() {
        issues.push(SecurityIssue {
            header: "X-Frame-Options".to_string(),
            issue_type: "missing_header".to_string(),
            message: "Missing X-Frame-Options header (clickjacking protection)".to_string(),
            severity: Severity::Medium,
        });
    }

    if https && headers.strict_transport_security.is_none() {
        issues.push(SecurityIssue {
            header: "Strict-Transport-Security".to_string(),
            issue_type: "missing_header".to_string(),
            message: "Missing HSTS header".to_string(),
            severity: Severity::High,
        });
    }

    if headers.referrer_policy.is_none() {
        issues.push(SecurityIssue {
            header: "Referrer-Policy".to_string(),
            issue_type: "missing_header".to_string(),
            message: "Missing Referrer-Policy header".to_string(),
            severity: Severity::Low,
        });
    }

    if headers.permissions_policy.is_none() {
        issues.push(SecurityIssue {
            header: "Permissions-Policy".to_string(),
            issue_type: "missing_header".to_string(),
            message: "Missing Permissions-Policy header".to_string(),
            severity: Severity::Low,
        });
    }

    if headers.cross_origin_opener_policy.is_none() {
        issues.push(SecurityIssue {
            header: "Cross-Origin-Opener-Policy".to_string(),
            issue_type: "missing_header".to_string(),
            message: "Missing Cross-Origin-Opener-Policy header".to_string(),
            severity: Severity::Low,
        });
    }

    if headers.cross_origin_resource_policy.is_none() {
        issues.push(SecurityIssue {
            header: "Cross-Origin-Resource-Policy".to_string(),
            issue_type: "missing_header".to_string(),
            message: "Missing Cross-Origin-Resource-Policy header".to_string(),
            severity: Severity::Low,
        });
    }

    issues
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

fn calculate_grade(score: u32) -> String {
    match score {
        90..=100 => "A+".to_string(),
        80..=89 => "A".to_string(),
        70..=79 => "B".to_string(),
        60..=69 => "C".to_string(),
        50..=59 => "D".to_string(),
        _ => "F".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        // COOP and CORP now generate Low-severity issues
        let issues = generate_security_issues(&headers, true);
        assert!(issues
            .iter()
            .any(|i| i.header.contains("Cross-Origin") && i.severity == Severity::Low));
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
        };
        let issues = generate_security_issues(&headers, true);
        assert!(issues.is_empty());
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
