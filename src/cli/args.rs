//! CLI argument parsing using clap
//!
//! Defines all command-line arguments and their validation.

use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// auditmysite - Resource-efficient WCAG 2.1 Accessibility Checker
///
/// Analyzes web pages for WCAG accessibility violations using
/// Chrome DevTools Protocol and the Accessibility Tree.
#[derive(Parser, Debug)]
#[command(
    name = "auditmysite",
    version,
    author,
    about = "Resource-efficient WCAG 2.1 Accessibility Checker in Rust",
    long_about = "auditmysite analyzes web pages for WCAG 2.1 accessibility violations.\n\n\
                  It uses Chrome's Accessibility Tree via CDP for accurate detection of:\n\
                  - Missing alt text on images (1.1.1)\n\
                  - Heading hierarchy issues (2.4.6)\n\
                  - Unlabeled form controls (4.1.2)\n\
                  - Insufficient color contrast (1.4.3)\n\n\
                  Supports single URLs, sitemaps, and URL list files."
)]
pub struct Args {
    /// URL to audit (single page)
    ///
    /// Example: https://example.com
    #[arg(value_name = "URL")]
    pub url: Option<String>,

    /// Sitemap URL to audit all pages
    ///
    /// Example: --sitemap https://example.com/sitemap.xml
    #[arg(short = 's', long, value_name = "SITEMAP_URL")]
    pub sitemap: Option<String>,

    /// File containing URLs to audit (one per line)
    ///
    /// Example: --url-file urls.txt
    #[arg(short = 'u', long, value_name = "FILE")]
    pub url_file: Option<PathBuf>,

    /// WCAG conformance level to check
    ///
    /// A: Level A only (minimum)
    /// AA: Level A + AA (recommended)
    /// AAA: Level A + AA + AAA (maximum)
    #[arg(short = 'l', long, default_value = "aa", value_enum)]
    pub level: WcagLevel,

    /// Output format
    ///
    /// json: Machine-readable JSON
    /// table: Human-readable CLI table
    /// html: Interactive HTML report
    #[arg(short = 'f', long, default_value = "table", value_enum)]
    pub format: OutputFormat,

    /// Output file path (stdout if not specified)
    #[arg(short = 'o', long, value_name = "FILE")]
    pub output: Option<PathBuf>,

    /// Custom Chrome/Chromium binary path
    ///
    /// Overrides auto-detection. Can also be set via CHROME_PATH env var.
    #[arg(long, value_name = "PATH", env = "CHROME_PATH")]
    pub chrome_path: Option<String>,

    /// Remote debugging port for existing Chrome instance
    ///
    /// Connect to Chrome started with: --remote-debugging-port=9222
    #[arg(long, value_name = "PORT")]
    pub remote_debugging_port: Option<u16>,

    /// Maximum number of pages to audit (0 = unlimited)
    #[arg(short = 'm', long, default_value = "0", value_name = "NUM")]
    pub max_pages: usize,

    /// Number of concurrent browser tabs
    #[arg(short = 'c', long, default_value = "3", value_name = "NUM")]
    pub concurrency: usize,

    /// Page load timeout in seconds
    #[arg(short = 't', long, default_value = "30", value_name = "SECS")]
    pub timeout: u64,

    /// Disable sandbox mode (required for Docker/root)
    ///
    /// WARNING: Reduces security. Only use in containerized environments.
    #[arg(long)]
    pub no_sandbox: bool,

    /// Disable loading images (faster but no contrast check)
    #[arg(long)]
    pub disable_images: bool,

    /// Verbose output (show progress and debug info)
    #[arg(short = 'v', long)]
    pub verbose: bool,

    /// Quiet mode (only show errors)
    #[arg(short = 'q', long)]
    pub quiet: bool,

    /// Detect Chrome and print path (then exit)
    #[arg(long)]
    pub detect_chrome: bool,
}

/// WCAG conformance levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum WcagLevel {
    /// Level A - Minimum conformance
    #[value(name = "a", alias = "A")]
    A,
    /// Level AA - Recommended conformance (default)
    #[value(name = "aa", alias = "AA")]
    AA,
    /// Level AAA - Maximum conformance
    #[value(name = "aaa", alias = "AAA")]
    AAA,
}

impl std::fmt::Display for WcagLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WcagLevel::A => write!(f, "A"),
            WcagLevel::AA => write!(f, "AA"),
            WcagLevel::AAA => write!(f, "AAA"),
        }
    }
}

/// Output format options
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    /// JSON output (machine-readable)
    #[value(name = "json")]
    Json,
    /// CLI table output (human-readable)
    #[value(name = "table")]
    Table,
    /// HTML report output
    #[value(name = "html")]
    Html,
    /// Markdown output
    #[value(name = "markdown", alias = "md")]
    /// PDF report output (via Typst)
    #[value(name = "pdf")]
    Pdf,

    Markdown,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::Json => write!(f, "json"),
            OutputFormat::Table => write!(f, "table"),
            OutputFormat::Html => write!(f, "html"),
            OutputFormat::Markdown => write!(f, "markdown"),
            OutputFormat::Pdf => write!(f, "pdf"),
        }
    }
}

impl Args {
    /// Validate arguments
    pub fn validate(&self) -> Result<(), String> {
        // At least one input source required (unless --detect-chrome)
        if !self.detect_chrome
            && self.url.is_none()
            && self.sitemap.is_none()
            && self.url_file.is_none()
        {
            return Err("No input specified. Provide a URL, --sitemap, or --url-file.".to_string());
        }

        // Cannot specify multiple input sources
        let input_count = [
            self.url.is_some(),
            self.sitemap.is_some(),
            self.url_file.is_some(),
        ]
        .iter()
        .filter(|&&x| x)
        .count();

        if input_count > 1 {
            return Err(
                "Only one input source allowed. Use URL, --sitemap, OR --url-file.".to_string(),
            );
        }

        // Validate URL format if provided
        if let Some(ref url) = self.url {
            url::Url::parse(url).map_err(|e| format!("Invalid URL '{}': {}", url, e))?;
        }

        // Validate sitemap URL format if provided
        if let Some(ref sitemap) = self.sitemap {
            url::Url::parse(sitemap)
                .map_err(|e| format!("Invalid sitemap URL '{}': {}", sitemap, e))?;
        }

        // Validate URL file exists
        if let Some(ref file) = self.url_file {
            if !file.exists() {
                return Err(format!("URL file not found: {:?}", file));
            }
        }

        // Validate concurrency
        if self.concurrency == 0 {
            return Err("Concurrency must be at least 1".to_string());
        }
        if self.concurrency > 10 {
            return Err("Concurrency cannot exceed 10".to_string());
        }

        // Cannot be both verbose and quiet
        if self.verbose && self.quiet {
            return Err("Cannot use --verbose and --quiet together".to_string());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wcag_level_display() {
        assert_eq!(WcagLevel::A.to_string(), "A");
        assert_eq!(WcagLevel::AA.to_string(), "AA");
        assert_eq!(WcagLevel::AAA.to_string(), "AAA");
    }

    #[test]
    fn test_output_format_display() {
        assert_eq!(OutputFormat::Json.to_string(), "json");
        assert_eq!(OutputFormat::Table.to_string(), "table");
        assert_eq!(OutputFormat::Html.to_string(), "html");
    }

    #[test]
    fn test_validate_no_input() {
        let args = Args {
            url: None,
            sitemap: None,
            url_file: None,
            level: WcagLevel::AA,
            format: OutputFormat::Table,
            output: None,
            chrome_path: None,
            remote_debugging_port: None,
            max_pages: 0,
            concurrency: 3,
            timeout: 30,
            no_sandbox: false,
            disable_images: false,
            verbose: false,
            quiet: false,
            detect_chrome: false,
        };
        assert!(args.validate().is_err());
    }

    #[test]
    fn test_validate_with_url() {
        let args = Args {
            url: Some("https://example.com".to_string()),
            sitemap: None,
            url_file: None,
            level: WcagLevel::AA,
            format: OutputFormat::Table,
            output: None,
            chrome_path: None,
            remote_debugging_port: None,
            max_pages: 0,
            concurrency: 3,
            timeout: 30,
            no_sandbox: false,
            disable_images: false,
            verbose: false,
            quiet: false,
            detect_chrome: false,
        };
        assert!(args.validate().is_ok());
    }

    #[test]
    fn test_validate_invalid_url() {
        let args = Args {
            url: Some("not-a-valid-url".to_string()),
            sitemap: None,
            url_file: None,
            level: WcagLevel::AA,
            format: OutputFormat::Table,
            output: None,
            chrome_path: None,
            remote_debugging_port: None,
            max_pages: 0,
            concurrency: 3,
            timeout: 30,
            no_sandbox: false,
            disable_images: false,
            verbose: false,
            quiet: false,
            detect_chrome: false,
        };
        assert!(args.validate().is_err());
    }

    #[test]
    fn test_validate_verbose_and_quiet() {
        let args = Args {
            url: Some("https://example.com".to_string()),
            sitemap: None,
            url_file: None,
            level: WcagLevel::AA,
            format: OutputFormat::Table,
            output: None,
            chrome_path: None,
            remote_debugging_port: None,
            max_pages: 0,
            concurrency: 3,
            timeout: 30,
            no_sandbox: false,
            disable_images: false,
            verbose: true,
            quiet: true,
            detect_chrome: false,
        };
        assert!(args.validate().is_err());
    }
}
