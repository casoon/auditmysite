//! Security headers analysis module
//!
//! Analyzes HTTP security headers and SSL/TLS configuration.
//! Also provides URL validation for SSRF protection.

use std::net::IpAddr;

use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use url::Url;

use crate::error::{AuditError, Result};

/// Validate a URL for safety (SSRF protection)
///
/// Blocks:
/// - Private IP ranges (10.x, 172.16-31.x, 192.168.x)
/// - Loopback addresses (127.x, ::1)
/// - Link-local addresses (169.254.x, fe80::)
/// - Non-HTTP(S) schemes
///
/// # Arguments
/// * `url_str` - The URL to validate
///
/// # Returns
/// * `Ok(Url)` - The parsed, validated URL
/// * `Err(AuditError)` - If the URL is invalid or blocked
pub fn validate_url(url_str: &str) -> Result<Url> {
    let url =
        Url::parse(url_str).map_err(|e| AuditError::ConfigError(format!("Invalid URL: {}", e)))?;

    // Only allow HTTP(S)
    match url.scheme() {
        "http" | "https" => {}
        scheme => {
            return Err(AuditError::ConfigError(format!(
                "URL scheme '{}' not allowed. Only http and https are permitted.",
                scheme
            )));
        }
    }

    // Check host
    let host = url
        .host_str()
        .ok_or_else(|| AuditError::ConfigError("URL must have a host".to_string()))?;

    // Block localhost variants
    if is_localhost(host) {
        warn!("Blocked localhost URL: {}", url_str);
        return Err(AuditError::ConfigError(
            "Localhost URLs are not allowed for security reasons".to_string(),
        ));
    }

    // Try to parse as IP and check for private ranges
    if let Ok(ip) = host.parse::<IpAddr>() {
        if is_private_ip(&ip) {
            warn!("Blocked private IP URL: {}", url_str);
            return Err(AuditError::ConfigError(
                "Private IP addresses are not allowed for security reasons".to_string(),
            ));
        }
    }

    Ok(url)
}

/// Check if a host string represents localhost
fn is_localhost(host: &str) -> bool {
    let host_lower = host.to_lowercase();
    host_lower == "localhost"
        || host_lower == "127.0.0.1"
        || host_lower == "::1"
        || host_lower == "[::1]"
        || host_lower.ends_with(".localhost")
        || host_lower == "0.0.0.0"
}

/// Check if an IP address is in a private range
fn is_private_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(ipv4) => {
            ipv4.is_loopback()           // 127.0.0.0/8
                || ipv4.is_private()     // 10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16
                || ipv4.is_link_local()  // 169.254.0.0/16
                || ipv4.is_unspecified() // 0.0.0.0
                || ipv4.is_broadcast()   // 255.255.255.255
                || ipv4.is_documentation() // 192.0.2.0/24, 198.51.100.0/24, 203.0.113.0/24
        }
        IpAddr::V6(ipv6) => {
            ipv6.is_loopback()           // ::1
                || ipv6.is_unspecified() // ::
                // Note: is_unique_local and is_unicast_link_local are unstable
                // Check manually for common private ranges
                || is_ipv6_private(ipv6)
        }
    }
}

/// Check for private IPv6 ranges (since some methods are unstable)
fn is_ipv6_private(ip: &std::net::Ipv6Addr) -> bool {
    let segments = ip.segments();
    // fc00::/7 - Unique Local Addresses
    (segments[0] & 0xfe00) == 0xfc00
        // fe80::/10 - Link-Local
        || (segments[0] & 0xffc0) == 0xfe80
        // ::ffff:0:0/96 - IPv4-mapped (check the embedded IPv4)
        || (segments[0] == 0 && segments[1] == 0 && segments[2] == 0
            && segments[3] == 0 && segments[4] == 0 && segments[5] == 0xffff)
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
    /// X-XSS-Protection
    pub x_xss_protection: Option<String>,
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
            self.x_xss_protection.is_some(),
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
    pub severity: String,
}

/// Analyze security headers of a URL
pub async fn analyze_security(url: &str) -> Result<SecurityAnalysis> {
    info!("Analyzing security headers for {}...", url);

    let https = url.starts_with("https://");

    // Make a HEAD request to get headers
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(false)
        .build()
        .map_err(|e| AuditError::HttpError(e))?;

    let response = client
        .head(url)
        .send()
        .await
        .map_err(|e| AuditError::HttpError(e))?;

    let header_map = response.headers();

    // Extract security headers
    let headers = extract_security_headers(header_map);

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
        "Security analysis: score={}, grade={}, headers={}",
        score,
        grade,
        headers.count()
    );

    Ok(SecurityAnalysis {
        score,
        grade,
        headers,
        ssl,
        issues,
        recommendations,
    })
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
        x_xss_protection: headers
            .get("x-xss-protection")
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

fn generate_security_issues(headers: &SecurityHeaders, https: bool) -> Vec<SecurityIssue> {
    let mut issues = Vec::new();

    if !https {
        issues.push(SecurityIssue {
            header: "HTTPS".to_string(),
            issue_type: "missing_https".to_string(),
            message: "Site is not served over HTTPS".to_string(),
            severity: "critical".to_string(),
        });
    }

    if headers.content_security_policy.is_none() {
        issues.push(SecurityIssue {
            header: "Content-Security-Policy".to_string(),
            issue_type: "missing_header".to_string(),
            message: "Missing Content-Security-Policy header".to_string(),
            severity: "high".to_string(),
        });
    }

    if headers.x_content_type_options.is_none() {
        issues.push(SecurityIssue {
            header: "X-Content-Type-Options".to_string(),
            issue_type: "missing_header".to_string(),
            message: "Missing X-Content-Type-Options header".to_string(),
            severity: "medium".to_string(),
        });
    }

    if headers.x_frame_options.is_none() {
        issues.push(SecurityIssue {
            header: "X-Frame-Options".to_string(),
            issue_type: "missing_header".to_string(),
            message: "Missing X-Frame-Options header (clickjacking protection)".to_string(),
            severity: "medium".to_string(),
        });
    }

    if https && headers.strict_transport_security.is_none() {
        issues.push(SecurityIssue {
            header: "Strict-Transport-Security".to_string(),
            issue_type: "missing_header".to_string(),
            message: "Missing HSTS header".to_string(),
            severity: "high".to_string(),
        });
    }

    if headers.referrer_policy.is_none() {
        issues.push(SecurityIssue {
            header: "Referrer-Policy".to_string(),
            issue_type: "missing_header".to_string(),
            message: "Missing Referrer-Policy header".to_string(),
            severity: "low".to_string(),
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

    recommendations
}

fn calculate_security_score(
    _headers: &SecurityHeaders,
    ssl: &SslInfo,
    issues: &[SecurityIssue],
) -> u32 {
    let mut score = 100u32;

    // Deduct for HTTPS
    if !ssl.https {
        score = score.saturating_sub(30);
    }

    // Deduct for missing headers
    for issue in issues {
        score = score.saturating_sub(match issue.severity.as_str() {
            "critical" => 25,
            "high" => 15,
            "medium" => 10,
            "low" => 5,
            _ => 5,
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
    fn test_calculate_grade() {
        assert_eq!(calculate_grade(95), "A+");
        assert_eq!(calculate_grade(85), "A");
        assert_eq!(calculate_grade(75), "B");
        assert_eq!(calculate_grade(65), "C");
        assert_eq!(calculate_grade(55), "D");
        assert_eq!(calculate_grade(40), "F");
    }

    #[test]
    fn test_validate_url_valid() {
        assert!(validate_url("https://example.com").is_ok());
        assert!(validate_url("https://example.com/path?query=1").is_ok());
        assert!(validate_url("http://example.com").is_ok());
    }

    #[test]
    fn test_validate_url_localhost_blocked() {
        assert!(validate_url("http://localhost").is_err());
        assert!(validate_url("http://localhost:8080").is_err());
        assert!(validate_url("http://127.0.0.1").is_err());
        assert!(validate_url("http://127.0.0.1:3000").is_err());
        assert!(validate_url("http://0.0.0.0").is_err());
        assert!(validate_url("http://[::1]").is_err());
    }

    #[test]
    fn test_validate_url_private_ip_blocked() {
        assert!(validate_url("http://10.0.0.1").is_err());
        assert!(validate_url("http://10.255.255.255").is_err());
        assert!(validate_url("http://172.16.0.1").is_err());
        assert!(validate_url("http://172.31.255.255").is_err());
        assert!(validate_url("http://192.168.0.1").is_err());
        assert!(validate_url("http://192.168.1.100").is_err());
        assert!(validate_url("http://169.254.1.1").is_err());
    }

    #[test]
    fn test_validate_url_invalid_scheme() {
        assert!(validate_url("ftp://example.com").is_err());
        assert!(validate_url("file:///etc/passwd").is_err());
        assert!(validate_url("javascript:alert(1)").is_err());
    }

    #[test]
    fn test_validate_url_invalid_format() {
        assert!(validate_url("not a url").is_err());
        assert!(validate_url("").is_err());
    }

    #[test]
    fn test_is_localhost() {
        assert!(is_localhost("localhost"));
        assert!(is_localhost("LOCALHOST"));
        assert!(is_localhost("127.0.0.1"));
        assert!(is_localhost("::1"));
        assert!(is_localhost("[::1]"));
        assert!(is_localhost("0.0.0.0"));
        assert!(is_localhost("foo.localhost"));
        assert!(!is_localhost("example.com"));
        assert!(!is_localhost("localhost.example.com"));
    }
}
