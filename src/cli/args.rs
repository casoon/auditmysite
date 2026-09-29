//! CLI argument parsing using clap
//!
//! Defines all command-line arguments and their validation.

use clap::{Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// AuditMySite - Resource-efficient WCAG 2.2 AA Accessibility Checker
///
/// Analyzes web pages for WCAG accessibility violations using
/// Chrome DevTools Protocol and the Accessibility Tree.
// The doc comments on these fields are clap's `--help` text before they are
// rustdoc, and a bare example URL is what a terminal should show. Wrapping one
// in angle brackets or backticks to satisfy `bare_urls` would put that markup
// in front of every user running `--help`.
#[allow(rustdoc::bare_urls)]
#[derive(Parser, Debug, Clone)]
#[command(
    name = "auditmysite",
    version,
    author,
    about = "Resource-efficient WCAG 2.2 AA Accessibility Checker in Rust",
    long_about = "AuditMySite analyzes web pages for WCAG 2.2 AA accessibility violations.\n\n\
It uses Chrome's Accessibility Tree via CDP for accurate detection of:\n\
- Missing alt text on images (1.1.1)\n\
- Heading hierarchy issues (2.4.6)\n\
- Unlabeled form controls (4.1.2)\n\
- Insufficient color contrast (1.4.3)\n\n\
Supported output formats: json, table, pdf, ai.\n\
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
    #[arg(short = 's', long, value_name = "SITEMAP_URL", global = true)]
    pub sitemap: Option<String>,

    /// File containing URLs to audit (one per line)
    ///
    /// Example: --url-file urls.txt
    #[arg(short = 'u', long, value_name = "FILE", global = true)]
    pub url_file: Option<PathBuf>,

    /// Crawl a site from the given base URL and discover same-domain pages automatically.
    ///
    /// NOTE: Crawl mode sends additional HTTP requests beyond the browser-based audit —
    /// one HEAD/GET per discovered link (including external links up to a cap of 100 unique targets).
    /// This generates observable traffic to third-party domains and increases audit time.
    /// Use --crawl deliberately, not in every CI run, to avoid repeated external traffic.
    #[arg(long, global = true)]
    pub crawl: bool,

    /// WCAG conformance level to check.
    ///
    /// `A` checks only level A.
    /// `AA` checks level A and AA.
    /// `AAA` checks levels A, AA, and AAA.
    #[arg(short = 'l', long, default_value = "aa", value_enum)]
    pub level: WcagLevel,

    /// Output format: `json`, `table`, `pdf`, or `ai`.
    ///
    /// Default for a single URL without `-f`: `pdf`.
    /// Default for batch inputs without `-f`: `table`.
    #[arg(short = 'f', long, value_enum, global = true)]
    pub format: Option<OutputFormat>,

    /// Output file path.
    ///
    /// Single URL + default PDF mode writes to `./<domain>-<date>-<report-level>.pdf`.
    /// JSON without `-o` prints to stdout.
    /// With `--per-page-reports`, `-o` is treated as the target directory.
    #[arg(short = 'o', long, value_name = "FILE", global = true)]
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
    #[arg(
        short = 'm',
        long,
        default_value = "0",
        value_name = "NUM",
        global = true
    )]
    pub max_pages: usize,

    /// Maximum crawl depth for `--crawl` (BFS levels from seed URL).
    ///
    /// Depth 1 = only links on the seed page. Depth 2 = seed + one level deeper.
    /// Higher values multiply the number of HTTP link-check requests.
    #[arg(long, default_value = "2", value_name = "NUM")]
    pub crawl_depth: usize,

    /// Number of concurrent browser tabs [default: 3]
    #[arg(short = 'c', long, value_name = "NUM")]
    pub concurrency: Option<usize>,

    /// Page load timeout in seconds [default: 30]
    #[arg(short = 't', long, value_name = "SECS")]
    pub timeout: Option<u64>,

    /// Maximum wait for late hydration and DOM stabilization after navigation.
    #[arg(long, default_value = "1500", value_name = "MS", global = true)]
    pub stability_budget_ms: u64,

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
    #[arg(short = 'q', long, global = true)]
    pub quiet: bool,

    /// Terminal color policy for table output and batch progress (#529)
    #[arg(long, default_value = "auto")]
    pub color: ColorPolicy,

    /// Batch lifecycle progress policy, independent of --quiet (#530)
    #[arg(long, default_value = "auto")]
    pub progress: ProgressPolicy,

    /// Detect Chrome and print path (then exit)
    #[arg(long)]
    pub detect_chrome: bool,

    /// Run all checks (Performance + SEO + Security + Mobile)
    ///
    /// This is already the default for standard single-page audits.
    #[arg(long, global = true)]
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

    /// Enable the design-quality module (opt-in UX/readability heuristics;
    /// does not affect score, grade, or certificate). Not part of --full yet.
    #[arg(long)]
    pub design_quality: bool,

    /// Enable the C2PA image-provenance check (EU AI Act Art. 50
    /// transparency duties; opt-in, single-URL mode only, score-neutral).
    /// Requires the binary to be built with `--features ai-transparency`;
    /// otherwise the audit fails with a rebuild instruction before starting.
    #[arg(long)]
    pub ai_transparency: bool,

    /// Enable the DNS-configuration check (CAA, DNSSEC, SPF/MX; opt-in,
    /// score-neutral, #545). Runs once per unique host, not once per page.
    #[arg(long)]
    pub dns_check: bool,

    /// Measure the isolated main-thread impact of individual third-party
    /// scripts (tag managers, chat widgets, tracking pixels) via CDP
    /// request-blocking (#531).
    ///
    /// For each of the top 5 third-party origins found on the page, reloads
    /// the page once with that origin's requests blocked and diffs the
    /// resulting Total Blocking Time against the normal audit pass. Opt-in
    /// and costly (up to 5 extra full page reloads per audited page) —
    /// requires performance checking (`--full` or `--performance`) to also
    /// be active, and only runs in single-URL mode.
    #[arg(long)]
    pub isolate_third_party_impact: bool,

    /// Detect SSR/hydration content gaps by reloading the page a second
    /// time with JavaScript execution disabled (CDP
    /// `Emulation.setScriptExecutionDisabled`) and comparing visible
    /// content length against the normal, JavaScript-enabled load (#534).
    ///
    /// Relevant for hybrid/partial-hydration frameworks (e.g. Astro
    /// Islands) where essential content may only appear client-side —
    /// search crawlers and assistive tools that do not execute JavaScript
    /// would see substantially less content. Opt-in and costly (one extra
    /// full page reload per audited page) — requires SEO checking
    /// (`--full` or `--seo`) to also be active, and only runs in
    /// single-URL mode.
    #[arg(long)]
    pub check_ssr_content: bool,

    /// Attempt to dismiss cookie consent banners before auditing.
    ///
    /// Injects known CMP consent cookies before navigation and clicks
    /// accept buttons if a banner is still detected after load.
    /// Without this flag, detected banners are reported as audit_flags.
    #[arg(long)]
    pub dismiss_consent: bool,

    /// Exclude the subtree matched by a CSS selector from the findings
    /// (repeatable), e.g. intentionally broken teaching specimens.
    ///
    /// The page is still audited; only findings whose element lies inside a
    /// matched subtree are dropped. The report lists every selector, how many
    /// elements it matched (also zero) and how many findings it removed.
    /// `[data-audit-exclude]` is always honoured without this flag.
    #[arg(long = "exclude-selector", value_name = "CSS")]
    pub exclude_selector: Vec<String>,

    /// Run the Accessibility-Journey-Layer for interactive checks
    /// (tab walk, modal focus trap, skip-link verification, …).
    ///
    /// `off`   — no interactive phase, fastest.
    /// `basic` — tab walk + skip link + obvious pattern journeys.
    /// `full`  (default) — adds SPA navigation, form-error announcement,
    ///           link-text inventory and outline export.
    ///
    /// Adds a few seconds per URL; recommended for single-URL audits.
    #[arg(long, value_enum, default_value = "full")]
    pub interactive: InteractiveMode,

    /// Enable tech stack detection and stack-specific audits (WordPress, Next.js, Drupal, …)
    #[arg(long)]
    pub stack: bool,

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

    /// For sitemap or URL-file inputs, generate one single-page report per URL
    ///
    /// Batch inputs normally create one aggregated domain report. With this flag,
    /// auditmysite still scans all URLs but writes individual reports per page.
    #[arg(long)]
    pub per_page_reports: bool,

    /// Only audit batch URLs whose path matches this glob (repeatable).
    ///
    /// Applies to --sitemap, --url-file and --crawl, before -m/--max-pages.
    /// Matched against the whole, percent-decoded URL path (no host, no
    /// query): `*` and `?` stay within one path segment, `**` crosses
    /// segments, `/**/` also matches a single `/`. Example: `/blog/**`
    /// selects everything below /blog/ (but not /blog itself). With several
    /// patterns a URL is kept when it matches any of them.
    #[arg(long = "include-path", value_name = "GLOB")]
    pub include_path: Vec<String>,

    /// Skip batch URLs whose path matches this glob (repeatable).
    ///
    /// Same glob rules as --include-path; exclusion wins over inclusion.
    /// Example: `--exclude-path '/tag/**' --exclude-path '/**/page/*'`.
    #[arg(long = "exclude-path", value_name = "GLOB")]
    pub exclude_path: Vec<String>,

    /// Do not write the per-page screen-reader sidecar JSON
    /// (`*-screen-reader-audit.json`).
    ///
    /// The screen-reader summary inside the main report stays.
    #[arg(long)]
    pub no_screen_reader_report: bool,

    /// Enable the HTML-conformance module without --full.
    #[arg(long)]
    pub html_conform: bool,

    /// Technician mode: per-page JSON for fixing, not a report to read.
    ///
    /// Batch inputs only. Shorthand for `--per-page-reports -f json
    /// --no-screen-reader-report --seo --html-conform`: one JSON file per
    /// page plus `index.json` (every attempted URL with status and counts)
    /// and `findings.jsonl` (one line per finding occurrence) in the -o
    /// directory. Runs only modules that produce fixable findings —
    /// accessibility (incl. keyboard journeys), HTML conformance and SEO —
    /// and no throttled performance passes; every file states that partial
    /// scope in `execution.scope`/`module_runs`. Flags given explicitly win:
    /// `-f` replaces the format, `--full`, `--performance`, `--mobile`,
    /// `--security` or `--interactive` add or change modules as usual.
    #[arg(long)]
    pub technician: bool,

    /// PDF detail level: `executive`, `standard`, or `technical`.
    #[arg(long, default_value = "standard", value_enum, global = true)]
    pub report_level: ReportLevel,

    /// Report language (PDF text i18n)
    #[arg(long, default_value = "de", value_parser = ["de", "en"], global = true)]
    pub lang: String,

    /// Also write a JSON report alongside the primary output format.
    ///
    /// When combined with `--format pdf`, writes a `.json` file next to the PDF
    /// without running a second audit. Useful for both single and batch modes.
    #[arg(long, global = true)]
    pub also_json: bool,

    /// Logo image path for PDF cover page
    #[arg(long, value_name = "PATH")]
    pub logo: Option<PathBuf>,

    /// Hidden: with `--format pdf`, also write the intermediate Typst source as a
    /// `.typ` sidecar next to the PDF (single + batch). For template/wording review.
    #[arg(long, hide = true, global = true)]
    pub debug_typ: bool,

    /// Export AXTree snapshots and journey traces as YAML for developer debugging.
    ///
    /// The YAML file contains the page URL, document title, and all journey traces
    /// (action sequences with focus snapshots). Compatible with Playwright's ARIA
    /// snapshot format for CI regression testing.
    ///
    /// Example: --export-snapshot reports/casoon-snapshot.yaml
    #[arg(long, value_name = "PATH")]
    pub export_snapshot: Option<PathBuf>,

    /// Include an additional regulatory/editorial appendix section in the PDF report.
    ///
    /// Opt-in only — this section is not part of the default report ("Zusatz").
    /// `en301549` adds an EN 301 549 (chapter 9, "Web") clause mapping: which
    /// clauses have automatically detected violations, which were checked
    /// automatically with no violations found, and which require manual
    /// review, plus a disclaimer and the chapters outside this tool's scope.
    /// `bik` adds a chapter mapping onto the "BIK für Alle" editorial
    /// accessibility guide (Bilder/Alt-Text, Linktext, Struktur, Leichte
    /// Sprache, PDFs, Videos) — a re-grouping of already-computed findings,
    /// no new checks. The underlying `en301549_annex`/`bik_guide` JSON data
    /// is always included regardless of this flag — it only gates the
    /// optional PDF section.
    #[arg(long, value_enum)]
    pub annex: Option<AnnexKind>,

    /// How the browser should identify itself when making requests.
    ///
    /// In interactive mode this is prompted automatically.
    /// Use `--request-mode bot` to skip the prompt and identify as an audit bot.
    #[arg(long, default_value = "browser", value_enum)]
    pub request_mode: RequestMode,

    /// Exit 0 whenever the requested report artifacts were written successfully,
    /// regardless of the computed verdict (PASS/WARN/FAIL).
    ///
    /// Without this flag, the process exit code encodes the verdict (0/1/2),
    /// which is useful for CI gates but forces report-generation and batch-ranking
    /// consumers to special-case non-zero exits. Technical/runtime failures
    /// (browser errors, timeouts, invalid arguments, write failures) still exit
    /// non-zero either way. The verdict itself is unaffected and still appears
    /// in CLI, JSON and PDF output.
    #[arg(long, global = true)]
    pub report_mode: bool,
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
    /// Show what an audit would do (mode, scope, modules, output paths) without
    /// running it.
    Plan {
        /// URL to plan an audit for (required unless --sitemap or --url-file is set)
        url: Option<String>,
    },
    /// Compare the own `accname` computation against Chrome's native
    /// accessibility tree — a differential test without a screen reader.
    AccnameDiff {
        /// URL(s) to load and compare; more than one URL (together with
        /// --url-file) produces a corpus aggregate instead of a single result
        #[arg(value_name = "URL", required_unless_present = "url_file")]
        urls: Vec<String>,
        /// Read further URLs from a file (one per line, `#` comments allowed)
        #[arg(long)]
        url_file: Option<PathBuf>,
        /// Write the full JSON result here (terminal summary is always printed)
        #[arg(long)]
        output: Option<PathBuf>,
        /// Maximum stored divergence samples per kind; counts stay complete
        #[arg(long, default_value_t = auditmysite_default_max_samples())]
        max_samples: usize,
    },
    /// Run deterministic report-lint checks (#507) against a JSON report file.
    ReportLint {
        /// Path to a JSON report file (single or batch envelope)
        input: PathBuf,
        /// Minimum finding severity that causes a non-zero exit code (default: high)
        #[arg(long, value_enum)]
        fail_on: Option<ReportLintFailOn>,
        /// Path to the `--debug-typ` Typst source for the same report — when
        /// given, additionally checks the certificate token is traceable to it
        #[arg(long)]
        typst_source: Option<PathBuf>,
    },
}

/// `--fail-on` threshold for the `report-lint` subcommand.
///
/// A separate CLI-facing enum (rather than deriving `clap::ValueEnum` on
/// `taxonomy::Severity` directly) so the domain type stays free of CLI
/// concerns, matching how `WcagLevel` is kept distinct from any internal
/// WCAG-level representation.
#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
#[value(rename_all = "lowercase")]
pub enum ReportLintFailOn {
    Low,
    Medium,
    High,
    Critical,
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
///
/// Variant order is significant: derives `PartialOrd`/`Ord` so a higher
/// configured level is `>=` every lower required level. This lets rule
/// catalogs filter with `cfg.wcag_level >= rule.min_level` instead of
/// pattern-matching the gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Serialize, Deserialize)]
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

/// Depth of the Accessibility-Journey-Layer.
#[derive(ValueEnum, Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InteractiveMode {
    /// No interactive phase.
    #[value(name = "off")]
    Off,
    /// Tab walk, skip link, obvious pattern journeys.
    #[value(name = "basic")]
    Basic,
    /// Adds SPA navigation, form-error announcement, link/outline export.
    #[default]
    #[value(name = "full")]
    Full,
}

impl InteractiveMode {
    pub fn is_enabled(self) -> bool {
        !matches!(self, InteractiveMode::Off)
    }
}

/// Optional regulatory/editorial PDF appendix sections (opt-in, see `--annex`).
#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnnexKind {
    /// EN 301 549 (chapter 9, "Web") clause mapping annex.
    #[value(name = "en301549")]
    En301549,
    /// "BIK für Alle" editorial accessibility guide chapter mapping.
    #[value(name = "bik")]
    Bik,
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
    /// AI/LLM-optimised JSON output (task-oriented, impact-sorted)
    #[value(name = "ai")]
    Ai,
    /// Compact summary JSON for ranking dashboards (lastAudit-compatible)
    #[value(name = "summary")]
    Summary,
    /// SARIF 2.1.0 output for GitHub Code Scanning and other SARIF consumers
    #[value(name = "sarif")]
    Sarif,
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

/// Terminal color policy for `--format table` and batch progress output (#529).
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Default)]
pub enum ColorPolicy {
    /// Color only for an interactive terminal, honouring `NO_COLOR` (default)
    #[default]
    #[value(name = "auto")]
    Auto,
    /// Always emit ANSI color
    #[value(name = "always")]
    Always,
    /// Never emit ANSI color
    #[value(name = "never")]
    Never,
}

/// Batch lifecycle progress policy, independent of `--quiet` (#530).
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Default)]
pub enum ProgressPolicy {
    /// Interactive bar for a terminal, bounded lifecycle lines otherwise (default)
    #[default]
    #[value(name = "auto")]
    Auto,
    /// Force the interactive bar even when not a terminal
    #[value(name = "always")]
    Always,
    /// No progress/lifecycle output at all
    #[value(name = "never")]
    Never,
}

/// How the browser should identify itself when making requests
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RequestMode {
    /// Simulate a real browser (avoids bot detection)
    #[default]
    Browser,
    /// Identify as an audit bot (transparent, RFC 9309)
    Bot,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::Json => write!(f, "json"),
            OutputFormat::Table => write!(f, "table"),
            OutputFormat::Pdf => write!(f, "pdf"),
            OutputFormat::Ai => write!(f, "ai"),
            OutputFormat::Summary => write!(f, "summary"),
            OutputFormat::Sarif => write!(f, "sarif"),
        }
    }
}

impl Args {
    /// Returns the effective timeout in seconds (CLI value or default 30).
    pub fn effective_timeout(&self) -> u64 {
        self.timeout.unwrap_or(30)
    }

    /// Returns the effective concurrency: the CLI value, else 3 — or 1 when
    /// journeys run on a browser that stalls under parallel keyboard
    /// journeys (full browser on macOS, plan 65).
    pub fn effective_concurrency(&self) -> usize {
        self.concurrency.unwrap_or_else(|| {
            if self.interactive.is_enabled()
                && crate::browser::resolver::parallel_journeys_stall(self.chrome_path.as_deref())
            {
                1
            } else {
                3
            }
        })
    }

    pub fn effective_format(&self) -> OutputFormat {
        match self.format {
            Some(format) => format,
            None if self.per_page_reports => OutputFormat::Pdf,
            None if self.url.is_some() => OutputFormat::Pdf,
            None => OutputFormat::Table,
        }
    }

    /// Expand `--technician` into the flags it stands for.
    ///
    /// Only fills in what the user left open: an explicit `-f` stays, and
    /// module flags are additive, so `--full`, `--performance`, `--mobile` or
    /// `--security` given alongside still take effect.
    pub fn apply_technician_preset(&mut self) {
        if !self.technician {
            return;
        }
        self.per_page_reports = true;
        if self.format.is_none() {
            self.format = Some(OutputFormat::Json);
        }
        self.no_screen_reader_report = true;
        self.seo = true;
        self.html_conform = true;
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

        if self.crawl && self.url.is_none() {
            return Err("--crawl requires a base URL input".to_string());
        }

        if self.crawl && (self.sitemap.is_some() || self.url_file.is_some()) {
            return Err("--crawl can only be combined with a single base URL".to_string());
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
        let concurrency = self.effective_concurrency();
        if concurrency == 0 {
            return Err("Concurrency must be at least 1".to_string());
        }
        if concurrency > 10 {
            return Err("Concurrency cannot exceed 10".to_string());
        }

        let is_batch = self.sitemap.is_some() || self.url_file.is_some() || self.crawl;
        if self.technician && !is_batch {
            return Err(
                "--technician needs a batch input: --sitemap, --url-file or --crawl".to_string(),
            );
        }
        if !is_batch && !(self.include_path.is_empty() && self.exclude_path.is_empty()) {
            return Err(
                "--include-path/--exclude-path need a batch input: --sitemap, --url-file or --crawl"
                    .to_string(),
            );
        }
        for pattern in self.include_path.iter().chain(&self.exclude_path) {
            if !(pattern.starts_with('/') || pattern.starts_with('*')) {
                return Err(format!(
                    "Path pattern '{pattern}' must start with '/' or '*': it is matched \
                     against the URL path, e.g. '/{pattern}'"
                ));
            }
        }

        if self.crawl_depth == 0 {
            return Err("Crawl depth must be at least 1".to_string());
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

/// Default for `--max-samples`, kept in sync with the module's own constant
/// so the CLI help and the library never drift apart.
fn auditmysite_default_max_samples() -> usize {
    crate::accessibility::DEFAULT_MAX_SAMPLES
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
    fn test_interactive_defaults_to_full() {
        let args = Args::parse_from(["auditmysite", "https://example.com"]);
        assert_eq!(args.interactive, InteractiveMode::Full);
    }

    #[test]
    fn test_interactive_can_be_disabled_explicitly() {
        let args = Args::parse_from(["auditmysite", "https://example.com", "--interactive", "off"]);
        assert_eq!(args.interactive, InteractiveMode::Off);
    }

    #[test]
    fn test_report_mode_defaults_to_false_and_can_be_enabled() {
        let args = Args::parse_from(["auditmysite", "https://example.com"]);
        assert!(!args.report_mode);

        let args = Args::parse_from(["auditmysite", "https://example.com", "--report-mode"]);
        assert!(args.report_mode);
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
        // `--html-conform` is a module flag, not the removed html format.
        assert!(!help.replace("--html-conform", "").contains("html"));
        assert!(!help.contains("markdown"));
        assert!(!help.contains("--urls"));
    }

    fn test_args(url: Option<&str>) -> Args {
        Args {
            command: None,
            url: url.map(|s| s.to_string()),
            sitemap: None,
            url_file: None,
            crawl: false,
            level: WcagLevel::AA,
            format: None,
            output: None,
            chrome_path: None,
            remote_debugging_port: None,
            max_pages: 0,
            crawl_depth: 2,
            concurrency: None,
            timeout: None,
            stability_budget_ms: 1500,
            no_sandbox: false,
            disable_images: false,
            verbose: false,
            quiet: false,
            color: ColorPolicy::Auto,
            progress: ProgressPolicy::Auto,
            detect_chrome: false,
            full: false,
            performance: false,
            skip_performance: false,
            seo: false,
            security: false,
            mobile: false,
            skip_mobile: false,
            design_quality: false,
            ai_transparency: false,
            dns_check: false,
            isolate_third_party_impact: false,
            check_ssr_content: false,
            stack: false,
            reuse_cache: false,
            force_refresh: false,
            no_sitemap_suggest: false,
            prefer_sitemap: false,
            per_page_reports: false,
            include_path: Vec::new(),
            exclude_path: Vec::new(),
            no_screen_reader_report: false,
            html_conform: false,
            technician: false,
            dismiss_consent: false,
            exclude_selector: Vec::new(),
            interactive: InteractiveMode::Off,
            report_level: ReportLevel::Standard,
            lang: "de".to_string(),
            also_json: false,
            logo: None,
            debug_typ: false,
            export_snapshot: None,
            annex: None,
            request_mode: RequestMode::Browser,
            report_mode: false,
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
    fn test_validate_crawl_requires_url() {
        let mut args = test_args(None);
        args.crawl = true;
        assert!(args.validate().is_err());
    }

    #[test]
    fn test_validate_crawl_conflicts_with_sitemap() {
        let mut args = test_args(Some("https://example.com"));
        args.crawl = true;
        args.sitemap = Some("https://example.com/sitemap.xml".to_string());
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
    fn test_effective_format_defaults_to_pdf_for_per_page_batch_reports() {
        let mut args = test_args(None);
        args.sitemap = Some("https://example.com/sitemap.xml".to_string());
        args.per_page_reports = true;
        assert_eq!(args.effective_format(), OutputFormat::Pdf);
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

    fn technician_args(extra: &[&str]) -> Args {
        let mut argv = vec![
            "auditmysite",
            "--sitemap",
            "https://example.com/sitemap.xml",
            "--technician",
        ];
        argv.extend_from_slice(extra);
        let mut args = Args::parse_from(argv);
        args.apply_technician_preset();
        args
    }

    #[test]
    fn technician_preset_expands_to_per_page_json_without_sidecar() {
        let args = technician_args(&[]);
        assert!(args.per_page_reports);
        assert_eq!(args.effective_format(), OutputFormat::Json);
        assert!(args.no_screen_reader_report);
        assert!(args.seo);
        assert!(args.html_conform);
        assert!(!args.full_audit_enabled(), "a module selection, not --full");
        assert!(!args.performance && !args.mobile && !args.security);
        assert!(args.validate().is_ok());
    }

    #[test]
    fn technician_preset_keeps_an_explicit_format() {
        let args = technician_args(&["-f", "sarif"]);
        assert_eq!(args.effective_format(), OutputFormat::Sarif);
        assert!(args.per_page_reports);
    }

    #[test]
    fn technician_preset_leaves_explicit_module_flags_in_place() {
        let args = technician_args(&["--performance", "--full"]);
        assert!(args.performance);
        assert!(args.full_audit_enabled());
    }

    #[test]
    fn without_technician_the_preset_changes_nothing() {
        let mut args = Args::parse_from(["auditmysite", "--sitemap", "https://example.com/s.xml"]);
        args.apply_technician_preset();
        assert!(!args.per_page_reports && !args.seo && !args.html_conform);
        assert!(!args.no_screen_reader_report);
        assert!(args.format.is_none());
    }

    #[test]
    fn technician_and_path_filters_need_a_batch_input() {
        let mut args = test_args(Some("https://example.com"));
        args.technician = true;
        assert!(args.validate().unwrap_err().contains("--technician"));

        let mut args = test_args(Some("https://example.com"));
        args.include_path = vec!["/blog/**".to_string()];
        assert!(args.validate().unwrap_err().contains("--include-path"));

        let mut args = test_args(Some("https://example.com"));
        args.crawl = true;
        args.exclude_path = vec!["/tag/**".to_string()];
        assert!(args.validate().is_ok());
    }

    #[test]
    fn path_patterns_must_be_anchored() {
        let mut args = test_args(None);
        args.sitemap = Some("https://example.com/sitemap.xml".to_string());
        args.include_path = vec!["blog/*".to_string()];
        let err = args.validate().unwrap_err();
        assert!(err.contains("'/blog/*'"), "{err}");

        args.include_path = vec!["**/feed".to_string(), "/blog/**".to_string()];
        assert!(args.validate().is_ok());
    }

    #[test]
    fn path_flags_are_repeatable() {
        let args = Args::parse_from([
            "auditmysite",
            "--sitemap",
            "https://example.com/sitemap.xml",
            "--include-path",
            "/a/**",
            "--include-path",
            "/b/**",
            "--exclude-path",
            "/a/tag/**",
        ]);
        assert_eq!(args.include_path, vec!["/a/**", "/b/**"]);
        assert_eq!(args.exclude_path, vec!["/a/tag/**"]);
    }

    #[test]
    fn accname_diff_nimmt_mehrere_urls_oder_url_datei() {
        let args = Args::try_parse_from([
            "auditmysite",
            "accname-diff",
            "https://a.example",
            "https://b.example",
        ])
        .unwrap();
        match args.command {
            Some(Command::AccnameDiff { urls, url_file, .. }) => {
                assert_eq!(urls.len(), 2);
                assert!(url_file.is_none());
            }
            _ => panic!("accname-diff erwartet"),
        }

        let args = Args::try_parse_from(["auditmysite", "accname-diff", "--url-file", "urls.txt"])
            .unwrap();
        assert!(matches!(
            args.command,
            Some(Command::AccnameDiff { ref urls, url_file: Some(_), .. }) if urls.is_empty()
        ));

        assert!(Args::try_parse_from(["auditmysite", "accname-diff"]).is_err());
    }
}
