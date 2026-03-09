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
    /// Found in known system paths
    SystemPath,
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

/// Result of browser resolution
#[derive(Debug, Clone)]
pub struct ResolvedBrowser {
    pub browser: DetectedBrowser,
    pub mode: BrowserMode,
    /// All found candidates (for `browser detect`)
    pub all_candidates: Vec<DetectedBrowser>,
}

/// What to install via `auditmysite browser install`
#[derive(Debug, Clone, Copy)]
pub enum InstallTarget {
    /// Full Chrome for Testing
    ChromeForTesting,
    /// Minimal headless-shell (faster, smaller)
    HeadlessShell,
}
