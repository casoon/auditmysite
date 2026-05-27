//! Security headers analysis module
//!
//! Analyzes HTTP security headers and SSL/TLS configuration.

use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};
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

fn generate_recommendations(headers: &SecurityHeaders, https: bool) -> Vec<String> {
    let mut recommendations = Vec::new();

    if !https {
        recommendations.push("Enable HTTPS with a valid SSL certificate".to_string());
    }

    if headers.content_security_policy.is_none() {
        recommendations.push(
            "Add Content-Security-Policy header to prevent XSS and data injection".to_string(),
        );
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

    // Deduct for issues (includes missing HTTPS as a critical issue)
    for issue in issues {
        score = score.saturating_sub(match issue.severity {
            Severity::Critical => 25,
            Severity::High => 15,
            Severity::Medium => 10,
            Severity::Low => 5,
        });
    }

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
            content_security_policy: Some("default-src 'self'".to_string()),
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
            content_security_policy: Some("default-src 'self'".to_string()),
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
}
