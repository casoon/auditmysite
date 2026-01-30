//! Error types for AuditMySit
//!
//! Centralized error handling using thiserror for derive macros
//! and anyhow for error context propagation.

use std::path::PathBuf;
use thiserror::Error;

/// Main error type for the auditmysit application
#[derive(Debug, Error)]
pub enum AuditError {
    /// Chrome/Chromium browser not found on the system
    #[error("Chrome/Chromium not found!\n\nInstallation:\n  macOS:   brew install --cask google-chrome\n  Linux:   sudo apt install chromium-browser\n  Windows: Download from https://www.google.com/chrome/\n\nOr specify manually:\n  auditmysit --chrome-path /path/to/chrome <url>")]
    ChromeNotFound,

    /// Chrome binary exists but is not executable
    #[error("Chrome binary at '{path}' is not executable. Try: chmod +x {path}")]
    ChromeNotExecutable { path: PathBuf },

    /// Browser failed to launch
    #[error("Failed to launch browser: {reason}")]
    BrowserLaunchFailed { reason: String },

    /// Browser connection lost
    #[error("Lost connection to browser: {reason}")]
    BrowserConnectionLost { reason: String },

    /// Navigation to URL failed
    #[error("Failed to navigate to '{url}': {reason}")]
    NavigationFailed { url: String, reason: String },

    /// Page load timeout
    #[error("Page load timeout for '{url}' after {timeout_secs} seconds")]
    PageLoadTimeout { url: String, timeout_secs: u64 },

    /// Accessibility tree extraction failed
    #[error("Failed to extract accessibility tree: {reason}")]
    AXTreeExtractionFailed { reason: String },

    /// Invalid URL provided
    #[error("Invalid URL: {url} - {reason}")]
    InvalidUrl { url: String, reason: String },

    /// Sitemap parsing failed
    #[error("Failed to parse sitemap from '{url}': {reason}")]
    SitemapParseFailed { url: String, reason: String },

    /// File read/write error
    #[error("File operation failed for '{path}': {reason}")]
    FileError { path: PathBuf, reason: String },

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// CDP (Chrome DevTools Protocol) error
    #[error("CDP error: {0}")]
    CdpError(String),

    /// Report generation failed
    #[error("Failed to generate report: {reason}")]
    ReportGenerationFailed { reason: String },

    /// Output formatting/writing failed
    #[error("Output error: {reason}")]
    OutputError { reason: String },

    /// Generic IO error wrapper
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// URL parsing error
    #[error("URL parse error: {0}")]
    UrlParseError(#[from] url::ParseError),

    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// HTTP request error
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    /// Chromiumoxide error
    #[error("Browser automation error: {0}")]
    ChromiumError(String),

    /// Browser pool timeout
    #[error("Browser pool timeout: no page available after {timeout_secs} seconds")]
    PoolTimeout { timeout_secs: u64 },

    /// Browser pool closed
    #[error("Browser pool has been closed")]
    PoolClosed,

    /// Browser pool exhausted
    #[error("Browser pool exhausted: all pages are in use")]
    PoolExhausted,
}

/// Result type alias for AuditError
pub type Result<T> = std::result::Result<T, AuditError>;

impl From<chromiumoxide::error::CdpError> for AuditError {
    fn from(err: chromiumoxide::error::CdpError) -> Self {
        AuditError::CdpError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chrome_not_found_error_message() {
        let err = AuditError::ChromeNotFound;
        let msg = err.to_string();
        assert!(msg.contains("Chrome/Chromium not found"));
        assert!(msg.contains("brew install"));
        assert!(msg.contains("apt install"));
    }

    #[test]
    fn test_navigation_failed_error() {
        let err = AuditError::NavigationFailed {
            url: "https://example.com".to_string(),
            reason: "Connection refused".to_string(),
        };
        assert!(err.to_string().contains("example.com"));
        assert!(err.to_string().contains("Connection refused"));
    }
}
