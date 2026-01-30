//! Error Handling Tests
//!
//! Tests for error paths and edge cases

use auditmysite::audit::read_url_file;
use auditmysite::error::AuditError;

#[test]
fn test_read_url_file_nonexistent() {
    let result = read_url_file("/nonexistent/path/to/file.txt");
    assert!(result.is_err());

    match result {
        Err(AuditError::FileError { path, reason }) => {
            assert!(path.to_string_lossy().contains("nonexistent"));
            assert!(!reason.is_empty());
        }
        _ => panic!("Expected FileError"),
    }
}

#[test]
fn test_read_url_file_filters_invalid_urls() {
    use std::io::Write;

    // Create a temp file with mixed content
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join("test_urls.txt");

    {
        let mut file = std::fs::File::create(&temp_file).unwrap();
        writeln!(file, "https://valid.com").unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, "").unwrap();
        writeln!(file, "not-a-url").unwrap();
        writeln!(file, "ftp://invalid-scheme.com").unwrap();
        writeln!(file, "http://also-valid.com").unwrap();
        writeln!(file, "  https://with-whitespace.com  ").unwrap();
    }

    let result = read_url_file(temp_file.to_str().unwrap());

    // Clean up
    std::fs::remove_file(&temp_file).ok();

    let urls = result.expect("Should parse file");

    // Only valid http/https URLs should be included
    assert_eq!(urls.len(), 3);
    assert!(urls.contains(&"https://valid.com".to_string()));
    assert!(urls.contains(&"http://also-valid.com".to_string()));
    assert!(urls.contains(&"https://with-whitespace.com".to_string()));

    // Invalid entries should be filtered out
    assert!(!urls.iter().any(|u| u.contains("comment")));
    assert!(!urls.iter().any(|u| u.contains("not-a-url")));
    assert!(!urls.iter().any(|u| u.contains("ftp://")));
}

#[test]
fn test_audit_error_display() {
    let error = AuditError::ConfigError("Test error message".to_string());
    let display = format!("{}", error);
    assert!(display.contains("Test error message"));

    let nav_error = AuditError::NavigationFailed {
        url: "https://example.com".to_string(),
        reason: "Connection refused".to_string(),
    };
    let nav_display = format!("{}", nav_error);
    assert!(nav_display.contains("example.com"));
    assert!(nav_display.contains("Connection refused"));
}

#[test]
fn test_sitemap_parsing_error_types() {
    // Test that SitemapParseFailed error contains URL
    let error = AuditError::SitemapParseFailed {
        url: "https://example.com/sitemap.xml".to_string(),
        reason: "Invalid XML".to_string(),
    };

    let display = format!("{}", error);
    assert!(display.contains("sitemap.xml") || display.contains("Invalid XML"));
}

#[test]
fn test_page_timeout_error() {
    let error = AuditError::PageLoadTimeout {
        url: "https://slow-site.com".to_string(),
        timeout_secs: 30,
    };

    let display = format!("{}", error);
    assert!(display.contains("slow-site.com") || display.contains("30"));
}
