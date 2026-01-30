//! URL Validation Integration Tests
//!
//! Tests for SSRF protection and URL validation

use auditmysite::security::validate_url;

#[test]
fn test_valid_public_urls() {
    // Standard HTTPS URLs should pass
    assert!(validate_url("https://example.com").is_ok());
    assert!(validate_url("https://www.example.com").is_ok());
    assert!(validate_url("https://subdomain.example.com/path").is_ok());
    assert!(validate_url("https://example.com:8443/path?query=1").is_ok());

    // HTTP should also work (not recommended but valid)
    assert!(validate_url("http://example.com").is_ok());
}

#[test]
fn test_localhost_blocked() {
    // All localhost variants should be blocked
    assert!(validate_url("http://localhost").is_err());
    assert!(validate_url("http://localhost:3000").is_err());
    assert!(validate_url("http://localhost:8080/api").is_err());
    assert!(validate_url("https://localhost").is_err());

    // IPv4 loopback
    assert!(validate_url("http://127.0.0.1").is_err());
    assert!(validate_url("http://127.0.0.1:8080").is_err());
    assert!(validate_url("http://127.0.0.255").is_err());

    // IPv6 loopback
    assert!(validate_url("http://[::1]").is_err());
    assert!(validate_url("http://[::1]:8080").is_err());

    // 0.0.0.0 (binds to all interfaces)
    assert!(validate_url("http://0.0.0.0").is_err());
    assert!(validate_url("http://0.0.0.0:80").is_err());
}

#[test]
fn test_private_networks_blocked() {
    // 10.0.0.0/8 - Class A private
    assert!(validate_url("http://10.0.0.1").is_err());
    assert!(validate_url("http://10.255.255.255").is_err());
    assert!(validate_url("http://10.10.10.10:8080/path").is_err());

    // 172.16.0.0/12 - Class B private
    assert!(validate_url("http://172.16.0.1").is_err());
    assert!(validate_url("http://172.31.255.255").is_err());
    assert!(validate_url("http://172.20.0.1").is_err());

    // 172.32.x.x should be allowed (outside private range)
    assert!(validate_url("http://172.32.0.1").is_ok());

    // 192.168.0.0/16 - Class C private
    assert!(validate_url("http://192.168.0.1").is_err());
    assert!(validate_url("http://192.168.1.1").is_err());
    assert!(validate_url("http://192.168.255.255").is_err());

    // Link-local (169.254.0.0/16)
    assert!(validate_url("http://169.254.1.1").is_err());
    assert!(validate_url("http://169.254.169.254").is_err()); // AWS metadata endpoint
}

#[test]
fn test_invalid_schemes_blocked() {
    // Only http and https allowed
    assert!(validate_url("ftp://example.com").is_err());
    assert!(validate_url("file:///etc/passwd").is_err());
    assert!(validate_url("file:///C:/Windows/System32").is_err());
    assert!(validate_url("javascript:alert(1)").is_err());
    assert!(validate_url("data:text/html,<script>alert(1)</script>").is_err());
    assert!(validate_url("gopher://example.com").is_err());
    assert!(validate_url("dict://example.com").is_err());
}

#[test]
fn test_malformed_urls() {
    assert!(validate_url("").is_err());
    assert!(validate_url("not-a-url").is_err());
    assert!(validate_url("://missing-scheme.com").is_err());
    // Note: Some malformed URLs may still parse depending on the URL parser
    // The key is that truly invalid URLs are rejected
}

#[test]
fn test_url_with_credentials_allowed() {
    // URLs with embedded credentials should parse (though not recommended)
    // The security concern here is SSRF, not credential leakage
    assert!(validate_url("https://user:pass@example.com").is_ok());
}

#[test]
fn test_international_domains() {
    // Internationalized domain names should work
    assert!(validate_url("https://例え.jp").is_ok());
    assert!(validate_url("https://münchen.de").is_ok());
}
