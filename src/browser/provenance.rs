//! Which browser measured (#787).
//!
//! The record the measurement profile carries: product, version, source,
//! platform, headless mode and whether a failover happened. It holds
//! categories only -- never executable paths, user names or launch flags,
//! which are personal data and not reproducible. The JSON is canonical
//! English; the kinds are enums, not prose.

use serde::{Deserialize, Serialize};

use super::platform::{Arch, Os, Platform};
use super::types::{BrowserKind, BrowserSource};

/// What kind of browser measured.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BrowserProduct {
    Chrome,
    ChromeForTesting,
    HeadlessShell,
    Edge,
    Chromium,
    UngoogledChromium,
    /// An explicit executable that could not be classified.
    Unidentified,
}

impl From<BrowserKind> for BrowserProduct {
    fn from(kind: BrowserKind) -> Self {
        match kind {
            BrowserKind::Chrome => Self::Chrome,
            BrowserKind::Edge => Self::Edge,
            BrowserKind::UngoogledChromium => Self::UngoogledChromium,
            BrowserKind::Chromium => Self::Chromium,
            BrowserKind::ChromeForTesting => Self::ChromeForTesting,
            BrowserKind::HeadlessShell => Self::HeadlessShell,
            BrowserKind::Custom => Self::Unidentified,
        }
    }
}

impl BrowserProduct {
    /// Installed by `auditmysite browser install`.
    pub fn is_managed(self) -> bool {
        matches!(self, Self::ChromeForTesting | Self::HeadlessShell)
    }
}

/// Where the executable came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BrowserOrigin {
    /// A system-wide install location
    System,
    /// A user-level install location
    User,
    /// Found on `PATH`
    Path,
    /// Installed by `auditmysite browser install`
    Managed,
    /// Named by the caller (`--browser-path`, environment variable)
    Explicit,
}

impl From<BrowserSource> for BrowserOrigin {
    fn from(source: BrowserSource) -> Self {
        match source {
            BrowserSource::CliFlag | BrowserSource::EnvVar => Self::Explicit,
            BrowserSource::SystemPath => Self::System,
            BrowserSource::UserInstall => Self::User,
            BrowserSource::PathSearch => Self::Path,
            BrowserSource::ManagedInstall => Self::Managed,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlatformOs {
    Macos,
    Linux,
    Windows,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlatformArch {
    X86_64,
    Aarch64,
    Other,
}

/// How the browser ran.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HeadlessModeKind {
    /// `--headless=new`
    New,
    /// The old headless implementation
    Legacy,
    /// A visible window
    Headful,
}

impl HeadlessModeKind {
    pub fn is_headless(self) -> bool {
        !matches!(self, Self::Headful)
    }
}

/// The browser behind a measurement.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct BrowserProvenance {
    pub product: BrowserProduct,
    /// Full version string (`131.0.6778.85`); required for a live run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    pub origin: BrowserOrigin,
    pub os: PlatformOs,
    pub arch: PlatformArch,
    pub headless_mode: HeadlessModeKind,
    /// A candidate failed before this browser measured.
    pub failover: bool,
    /// Candidates tried or rejected ahead of the one that measured; paths
    /// and reasons stay out of the report. `failover` iff this is above 0.
    pub discarded_candidates: u32,
}

impl BrowserProvenance {
    pub fn new(
        kind: BrowserKind,
        source: BrowserSource,
        version: Option<String>,
        platform: Platform,
        headless_mode: HeadlessModeKind,
    ) -> Self {
        Self {
            product: kind.into(),
            version,
            origin: source.into(),
            os: match platform.os {
                Os::MacOs => PlatformOs::Macos,
                Os::Linux => PlatformOs::Linux,
                Os::Windows => PlatformOs::Windows,
                Os::Other => PlatformOs::Other,
            },
            arch: match platform.arch {
                Arch::X86_64 => PlatformArch::X86_64,
                Arch::Aarch64 => PlatformArch::Aarch64,
                Arch::Other => PlatformArch::Other,
            },
            headless_mode,
            failover: false,
            discarded_candidates: 0,
        }
    }

    /// A consistent record for tests that build reports without a browser.
    #[cfg(test)]
    pub(crate) fn for_tests() -> Self {
        Self::new(
            BrowserKind::Chrome,
            BrowserSource::SystemPath,
            Some("131.0.6778.85".into()),
            Platform::new(Os::MacOs, Arch::Aarch64),
            HeadlessModeKind::New,
        )
    }

    /// Counts candidates that were discarded ahead of the measuring one.
    pub fn record_discarded(&mut self, count: u32) {
        self.discarded_candidates += count;
        self.failover = self.discarded_candidates > 0;
    }

    pub fn platform(&self) -> Platform {
        Platform::new(
            match self.os {
                PlatformOs::Macos => Os::MacOs,
                PlatformOs::Linux => Os::Linux,
                PlatformOs::Windows => Os::Windows,
                PlatformOs::Other => Os::Other,
            },
            match self.arch {
                PlatformArch::X86_64 => Arch::X86_64,
                PlatformArch::Aarch64 => Arch::Aarch64,
                PlatformArch::Other => Arch::Other,
            },
        )
    }

    /// What a measurement depends on: product, version, platform and
    /// headless mode. Origin and failover say how the browser was found,
    /// not what it measured, so a replay does not compare them.
    pub fn identity(&self) -> String {
        format!(
            "{:?} {} {:?}/{:?} {:?}",
            self.product,
            self.version.as_deref().unwrap_or("unknown"),
            self.os,
            self.arch,
            self.headless_mode
        )
    }

    /// Why this record contradicts itself or the platform contract.
    pub fn inconsistencies(&self) -> Vec<String> {
        let mut out = Vec::new();
        if self.version.as_deref().is_none_or(|v| v.trim().is_empty()) {
            out.push("the browser version is missing".to_string());
        }
        if self.product.is_managed() != (self.origin == BrowserOrigin::Managed) {
            out.push(format!(
                "product {:?} and origin {:?} contradict each other: only the managed browsers \
                 come from a managed install",
                self.product, self.origin
            ));
        }
        if self.failover != (self.discarded_candidates > 0) {
            out.push(format!(
                "failover is {} but {} candidate(s) were discarded",
                self.failover, self.discarded_candidates
            ));
        }
        let platform = self.platform();
        if !platform.is_supported() {
            out.push(format!(
                "{} is outside the platform contract",
                platform.describe()
            ));
        } else if platform.is_edge_only() && self.product != BrowserProduct::Edge {
            out.push(format!(
                "{:?} on Windows: the platform contract allows Microsoft Edge only",
                self.product
            ));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> BrowserProvenance {
        BrowserProvenance::new(
            BrowserKind::Chrome,
            BrowserSource::SystemPath,
            Some("131.0.6778.85".into()),
            Platform::new(Os::MacOs, Arch::Aarch64),
            HeadlessModeKind::New,
        )
    }

    #[test]
    fn a_complete_record_is_consistent_and_carries_no_path() {
        let record = sample();
        assert!(record.inconsistencies().is_empty());
        let json = serde_json::to_value(&record).unwrap();
        assert_eq!(json["product"], "chrome");
        assert_eq!(json["origin"], "system");
        assert_eq!(json["os"], "macos");
        assert_eq!(json["arch"], "aarch64");
        assert_eq!(json["headless_mode"], "new");
        let text = json.to_string();
        assert!(!text.contains("Users") && !text.contains("Applications"));
    }

    #[test]
    fn contradictions_are_named() {
        let mut record = sample();
        record.version = None;
        record.origin = BrowserOrigin::Managed;
        record.failover = true;
        assert_eq!(record.inconsistencies().len(), 3);
        let mut windows = sample();
        windows.os = PlatformOs::Windows;
        windows.arch = PlatformArch::X86_64;
        assert_eq!(windows.inconsistencies().len(), 1);
        windows.product = BrowserProduct::Edge;
        assert!(windows.inconsistencies().is_empty());
        let mut intel = sample();
        intel.arch = PlatformArch::X86_64;
        assert_eq!(intel.inconsistencies().len(), 1);
    }

    #[test]
    fn discarding_candidates_sets_the_failover() {
        let mut record = sample();
        record.record_discarded(0);
        assert!(!record.failover);
        record.record_discarded(2);
        assert!(record.failover && record.discarded_candidates == 2);
    }

    #[test]
    fn identity_ignores_how_the_browser_was_found() {
        let mut other = sample();
        other.origin = BrowserOrigin::Path;
        other.record_discarded(1);
        assert_eq!(sample().identity(), other.identity());
    }
}
