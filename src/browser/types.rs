//! Browser type definitions
//!
//! Core types for the browser management architecture.

use std::fmt;
use std::path::PathBuf;

/// Which kind of Chromium-based browser
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BrowserKind {
    Chrome,
    Edge,
    UngoogledChromium,
    Chromium,
    /// Managed install via `auditmysite browser install`
    ChromeForTesting,
    /// Managed install, headless-shell only (fast mode)
    HeadlessShell,
    /// User-provided path via --browser-path
    Custom,
}

impl BrowserKind {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Chrome => "Google Chrome",
            Self::Edge => "Microsoft Edge",
            Self::UngoogledChromium => "Ungoogled Chromium",
            Self::Chromium => "Chromium",
            Self::ChromeForTesting => "Chrome for Testing",
            Self::HeadlessShell => "Chrome Headless Shell",
            Self::Custom => "Custom Browser",
        }
    }

    /// Whether this is a managed (self-installed) browser
    pub fn is_managed(&self) -> bool {
        matches!(self, Self::ChromeForTesting | Self::HeadlessShell)
    }
}

impl fmt::Display for BrowserKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// How the browser was found
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserSource {
    /// Explicit via --browser-path
    CliFlag,
    /// Via AUDITMYSITE_BROWSER or CHROME_PATH env var
    EnvVar,
    /// Found in known system-wide install locations
    SystemPath,
    /// Found in a user-level install location (home directory, `%LOCALAPPDATA%`)
    UserInstall,
    /// Found via `which`/`where` in PATH
    PathSearch,
    /// Self-installed under ~/.auditmysite/browsers/
    ManagedInstall,
}

impl fmt::Display for BrowserSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CliFlag => write!(f, "CLI flag"),
            Self::EnvVar => write!(f, "environment variable"),
            Self::SystemPath => write!(f, "system path"),
            Self::UserInstall => write!(f, "user-level install"),
            Self::PathSearch => write!(f, "PATH search"),
            Self::ManagedInstall => write!(f, "managed install"),
        }
    }
}

/// Browser run mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserMode {
    /// Normal headless Chrome with all features
    Standard,
    /// System browser only, no fallback to managed install
    Strict,
    /// Prefer headless-shell for speed
    Fast,
}

/// A detected browser on the system
#[derive(Debug, Clone)]
pub struct DetectedBrowser {
    pub kind: BrowserKind,
    pub path: PathBuf,
    pub version: Option<String>,
    pub source: BrowserSource,
}

/// Why a candidate cannot be used (#854).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RejectionKind {
    /// No file at the path (or not a regular file)
    NotFound,
    /// The file exists but has no execute permission
    NotExecutable,
    /// The file cannot be started (wrong architecture, broken binary, ...)
    NotRunnable,
    /// `--version` did not answer within the time bound
    Hangs,
    /// Started headless, but the DevTools connection could not be made
    CdpFailed,
    /// Started headless, but did not become reachable within the time bound
    CdpTimedOut,
    /// Excluded by the platform contract, strict mode or `--browser`
    Excluded,
    /// A managed install whose files are incomplete or outdated
    StaleInstall,
}

impl RejectionKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::NotFound => "not_found",
            Self::NotExecutable => "not_executable",
            Self::NotRunnable => "not_runnable",
            Self::Hangs => "hangs",
            Self::CdpFailed => "cdp_failed",
            Self::CdpTimedOut => "cdp_timed_out",
            Self::Excluded => "excluded",
            Self::StaleInstall => "stale_install",
        }
    }
}

/// Whether a candidate can be used, and if not, why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CandidateVerdict {
    Viable,
    Rejected { kind: RejectionKind, detail: String },
}

impl CandidateVerdict {
    pub fn is_viable(&self) -> bool {
        matches!(self, Self::Viable)
    }

    pub fn rejected(kind: RejectionKind, detail: impl Into<String>) -> Self {
        Self::Rejected {
            kind,
            detail: detail.into(),
        }
    }
}

/// One candidate in priority order with the reason it is or is not viable.
#[derive(Debug, Clone)]
pub struct CandidateReport {
    pub browser: DetectedBrowser,
    pub verdict: CandidateVerdict,
    /// Product string reported over CDP (`Chrome/131...`) when a headless
    /// launch was probed and succeeded; `None` when no probe ran.
    pub cdp_product: Option<String>,
}

/// Result of browser resolution
#[derive(Debug, Clone)]
pub struct ResolvedBrowser {
    pub browser: DetectedBrowser,
    pub mode: BrowserMode,
    /// All found candidates (for `browser detect`)
    pub all_candidates: Vec<DetectedBrowser>,
    /// Every candidate in priority order with its verdict. For an explicit
    /// path this is that one path (plus the automatic candidates only when
    /// fallback was requested).
    pub candidates: Vec<CandidateReport>,
    /// The selected browser came from an explicit path (flag or env var)
    pub explicit: bool,
}

impl ResolvedBrowser {
    /// Browsers to try in order when launching: the viable candidates,
    /// selected one first. An explicit path without requested fallback is a
    /// chain of one, so a failing launch never switches silently.
    /// Candidates ranked ahead of the launched one that were rejected on
    /// their merits (not found, not runnable, CDP failed ...). Those the
    /// platform contract or the options exclude never competed (#787).
    pub fn discarded_before_launch(&self) -> u32 {
        self.candidates
            .iter()
            .take_while(|c| !c.verdict.is_viable())
            .filter(|c| {
                !matches!(
                    c.verdict,
                    CandidateVerdict::Rejected {
                        kind: RejectionKind::Excluded,
                        ..
                    }
                )
            })
            .count() as u32
    }

    pub fn launch_chain(&self) -> Vec<DetectedBrowser> {
        self.candidates
            .iter()
            .filter(|c| c.verdict.is_viable())
            .map(|c| c.browser.clone())
            .collect()
    }
}

/// What to install via `auditmysite browser install`
#[derive(Debug, Clone, Copy)]
pub enum InstallTarget {
    /// Full Chrome for Testing
    ChromeForTesting,
    /// Minimal headless-shell (faster, smaller)
    HeadlessShell,
}
