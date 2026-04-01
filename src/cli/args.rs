//! CLI argument parsing using clap
//!
//! Defines all command-line arguments and their validation.

use clap::{Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// AuditMySit - Resource-efficient WCAG 2.1 Accessibility Checker
///
/// Analyzes web pages for WCAG accessibility violations using
/// Chrome DevTools Protocol and the Accessibility Tree.
#[derive(Parser, Debug, Clone)]
#[command(
    name = "auditmysite",
    version,
    author,
    about = "Resource-efficient WCAG 2.1 Accessibility Checker in Rust",
    long_about = "AuditMySit analyzes web pages for WCAG 2.1 accessibility violations.\n\n\
It uses Chrome's Accessibility Tree via CDP for accurate detection of:\n\
- Missing alt text on images (1.1.1)\n\
- Heading hierarchy issues (2.4.6)\n\
- Unlabeled form controls (4.1.2)\n\
- Insufficient color contrast (1.4.3)\n\n\
Supported output formats: json, table, pdf.\n\
Supported inputs: a single URL, --sitemap, or --url-file.\n\n\
Default single-URL behavior: generate a PDF report in the current directory."
)]
pub struct Args {
    /// Subcommand
    #[command(subcommand)]
    pub command: Option<Command>,

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

    /// WCAG conformance level to check.
    ///
    /// `A` checks only level A.
    /// `AA` checks level A and AA.
    /// `AAA` checks levels A, AA, and AAA.
    #[arg(short = 'l', long, default_value = "aa", value_enum)]
    pub level: WcagLevel,

    /// Output format: `json`, `table`, or `pdf`.
    ///
    /// Default for a single URL without `-f`: `pdf`.
    /// Default for batch inputs without `-f`: `table`.
    #[arg(short = 'f', long, value_enum)]
    pub format: Option<OutputFormat>,

    /// Output file path.
    ///
    /// Single URL + default PDF mode writes to `./<domain>-<date>-<report-level>.pdf`.
    /// JSON without `-o` prints to stdout.
    #[arg(short = 'o', long, value_name = "FILE")]
    pub output: Option<PathBuf>,

    /// Custom browser binary path (overrides auto-detection)
    ///
    /// Can also be set via AUDITMYSITE_BROWSER or CHROME_PATH env var.
    #[arg(
        long = "browser-path",
        alias = "chrome-path",
        value_name = "PATH",
        env = "AUDITMYSITE_BROWSER"
    )]
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

    /// Run all checks (Performance + SEO + Security + Mobile)
    ///
    /// This is already the default for standard single-page audits.
    #[arg(long)]
    pub full: bool,

    /// Enable performance analysis (Core Web Vitals)
    #[arg(long)]
    pub performance: bool,

    /// Skip performance analysis even when --full is used
    #[arg(long)]
    pub skip_performance: bool,

    /// Enable SEO analysis (meta tags, headings, schema.org)
    #[arg(long)]
    pub seo: bool,

    /// Enable security header analysis
    #[arg(long)]
    pub security: bool,

    /// Enable mobile friendliness check
    #[arg(long)]
    pub mobile: bool,

    /// Skip mobile analysis even when --full is used
    #[arg(long)]
    pub skip_mobile: bool,

    /// Reuse cached artifacts from previous runs when available
    #[arg(long)]
    pub reuse_cache: bool,

    /// Ignore cache and force a fresh crawl
    #[arg(long)]
    pub force_refresh: bool,

    /// Do not suggest scanning a discovered sitemap for base URLs
    #[arg(long)]
    pub no_sitemap_suggest: bool,

    /// If a populated sitemap is discovered for a base URL, scan it directly
    #[arg(long)]
    pub prefer_sitemap: bool,

    /// PDF detail level: `executive`, `standard`, or `technical`.
    #[arg(long, default_value = "standard", value_enum)]
    pub report_level: ReportLevel,

    /// Report language (PDF text i18n)
    #[arg(long, default_value = "de", value_parser = ["de", "en"])]
    pub lang: String,

    /// Company name for report branding (appears in footer)
    #[arg(long, value_name = "NAME")]
    pub company_name: Option<String>,

    /// Logo image path for PDF cover page
    #[arg(long, value_name = "PATH")]
    pub logo: Option<PathBuf>,
}

/// Subcommands
#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    /// Manage browser detection and installation
    Browser {
        #[command(subcommand)]
        action: BrowserAction,
    },
    /// Run diagnostics and check system health
    Doctor,
}

/// Browser management actions
#[derive(Subcommand, Debug, Clone)]
pub enum BrowserAction {
    /// Detect all installed browsers
    Detect,
    /// Install Chrome for Testing
    Install {
        /// Install headless-shell instead (smaller, faster)
        #[arg(long)]
        headless_shell: bool,
        /// Install specific version (default: latest stable)
        #[arg(long)]
        version: Option<String>,
        /// Force reinstall even if already present
        #[arg(long)]
        force: bool,
    },
    /// Remove managed browser installation
    Remove {
        /// Remove all managed browsers
        #[arg(long)]
        all: bool,
    },
    /// Print path of the active browser
    Path,
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
    /// PDF report output (via Typst)
    #[value(name = "pdf")]
    Pdf,
}

/// Report detail level for PDF reports
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Default)]
pub enum ReportLevel {
    /// Executive summary — compact overview for management
    #[value(name = "executive")]
    Executive,
    /// Standard report — all chapters (default)
    #[default]
    #[value(name = "standard")]
    Standard,
    /// Technical report — extended appendix with full details
    #[value(name = "technical")]
    Technical,
}

impl std::fmt::Display for ReportLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReportLevel::Executive => write!(f, "executive"),
            ReportLevel::Standard => write!(f, "standard"),
            ReportLevel::Technical => write!(f, "technical"),
        }
    }
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::Json => write!(f, "json"),
            OutputFormat::Table => write!(f, "table"),
            OutputFormat::Pdf => write!(f, "pdf"),
        }
    }
}

impl Args {
    pub fn effective_format(&self) -> OutputFormat {
        match self.format {
            Some(format) => format,
            None if self.url.is_some() => OutputFormat::Pdf,
            None => OutputFormat::Table,
        }
    }

    pub fn full_audit_enabled(&self) -> bool {
        self.full
            || (!self.performance
                && !self.seo
                && !self.security
                && !self.mobile
                && !self.skip_performance
                && !self.skip_mobile)
    }

    /// Validate arguments
    pub fn validate(&self) -> Result<(), String> {
        // Subcommands don't need URL validation
        if self.command.is_some() {
            return Ok(());
        }

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

        if self.reuse_cache && self.force_refresh {
            return Err("Cannot use --reuse-cache and --force-refresh together".to_string());
        }

        if self.no_sitemap_suggest && self.prefer_sitemap {
            return Err(
                "Cannot use --no-sitemap-suggest and --prefer-sitemap together".to_string(),
            );
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

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
        assert_eq!(OutputFormat::Pdf.to_string(), "pdf");
    }

    #[test]
    fn test_help_does_not_reference_removed_formats_or_flags() {
        let mut command = Args::command();
        let help = command.render_long_help().to_string();

        assert!(help.contains("--browser-path"));
        assert!(help.contains("--url-file"));
        assert!(help.contains("json"));
        assert!(help.contains("table"));
        assert!(help.contains("pdf"));
        assert!(help.contains(
            "Default single-URL behavior: generate a PDF report in the current directory."
        ));
        assert!(!help.contains("html"));
        assert!(!help.contains("markdown"));
        assert!(!help.contains("--urls"));
    }

    fn test_args(url: Option<&str>) -> Args {
        Args {
            command: None,
            url: url.map(|s| s.to_string()),
            sitemap: None,
            url_file: None,
            level: WcagLevel::AA,
            format: None,
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
            full: false,
            performance: false,
            skip_performance: false,
            seo: false,
            security: false,
            mobile: false,
            skip_mobile: false,
            reuse_cache: false,
            force_refresh: false,
            no_sitemap_suggest: false,
            prefer_sitemap: false,
            report_level: ReportLevel::Standard,
            lang: "de".to_string(),
            company_name: None,
            logo: None,
        }
    }

    #[test]
    fn test_validate_no_input() {
        assert!(test_args(None).validate().is_err());
    }

    #[test]
    fn test_validate_with_url() {
        assert!(test_args(Some("https://example.com")).validate().is_ok());
    }

    #[test]
    fn test_validate_invalid_url() {
        assert!(test_args(Some("not-a-valid-url")).validate().is_err());
    }

    #[test]
    fn test_validate_verbose_and_quiet() {
        let mut args = test_args(Some("https://example.com"));
        args.verbose = true;
        args.quiet = true;
        assert!(args.validate().is_err());
    }

    #[test]
    fn test_validate_reuse_and_force_refresh_conflict() {
        let mut args = test_args(Some("https://example.com"));
        args.reuse_cache = true;
        args.force_refresh = true;
        assert!(args.validate().is_err());
    }

    #[test]
    fn test_validate_sitemap_suggest_flags_conflict() {
        let mut args = test_args(Some("https://example.com"));
        args.no_sitemap_suggest = true;
        args.prefer_sitemap = true;
        assert!(args.validate().is_err());
    }

    #[test]
    fn test_effective_format_defaults_to_pdf_for_single_url() {
        let args = test_args(Some("https://example.com"));
        assert_eq!(args.effective_format(), OutputFormat::Pdf);
    }

    #[test]
    fn test_effective_format_defaults_to_table_for_batch() {
        let mut args = test_args(None);
        args.sitemap = Some("https://example.com/sitemap.xml".to_string());
        assert_eq!(args.effective_format(), OutputFormat::Table);
    }

    #[test]
    fn test_full_audit_enabled_by_default() {
        let args = test_args(Some("https://example.com"));
        assert!(args.full_audit_enabled());
    }

    #[test]
    fn test_full_audit_disabled_when_only_skip_flags_are_used() {
        let mut args = test_args(Some("https://example.com"));
        args.skip_mobile = true;
        assert!(!args.full_audit_enabled());
    }
}
