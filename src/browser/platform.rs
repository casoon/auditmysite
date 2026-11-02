//! Platform contract for browser selection (#854).
//!
//! Supported targets: macOS on ARM64, Linux on x86_64 and ARM64, Windows on
//! x86_64 with Microsoft Edge only. No browser is bundled. The platform is a
//! plain value so every rule below is unit-testable for a platform other than
//! the one the tests run on.

use super::types::BrowserKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Os {
    MacOs,
    Linux,
    Windows,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arch {
    X86_64,
    Aarch64,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Platform {
    pub os: Os,
    pub arch: Arch,
}

/// How an explicitly given executable fares under the platform policy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExplicitPolicy {
    Allowed,
    /// Not allowed on this platform; the message says what to do instead.
    Rejected(String),
}

impl Platform {
    pub const fn new(os: Os, arch: Arch) -> Self {
        Self { os, arch }
    }

    pub const fn current() -> Self {
        let os = if cfg!(target_os = "macos") {
            Os::MacOs
        } else if cfg!(target_os = "linux") {
            Os::Linux
        } else if cfg!(target_os = "windows") {
            Os::Windows
        } else {
            Os::Other
        };
        let arch = if cfg!(target_arch = "x86_64") {
            Arch::X86_64
        } else if cfg!(target_arch = "aarch64") {
            Arch::Aarch64
        } else {
            Arch::Other
        };
        Self { os, arch }
    }

    pub fn is_supported(&self) -> bool {
        matches!(
            (self.os, self.arch),
            (Os::MacOs, Arch::Aarch64)
                | (Os::Linux, Arch::X86_64)
                | (Os::Linux, Arch::Aarch64)
                | (Os::Windows, Arch::X86_64)
        )
    }

    /// Why this platform is outside the contract (`None` when supported).
    pub fn unsupported_reason(&self) -> Option<String> {
        if self.is_supported() {
            return None;
        }
        Some(format!(
            "{} is not a supported platform. Supported: macOS on ARM64, Linux on x86_64 and \
             ARM64, Windows on x86_64 (Microsoft Edge only)",
            self.describe()
        ))
    }

    pub fn describe(&self) -> String {
        let os = match self.os {
            Os::MacOs => "macOS",
            Os::Linux => "Linux",
            Os::Windows => "Windows",
            Os::Other => "this operating system",
        };
        let arch = match self.arch {
            Arch::X86_64 => "x86_64",
            Arch::Aarch64 => "ARM64",
            Arch::Other => "an unknown architecture",
        };
        format!("{os} on {arch}")
    }

    /// Windows runs Microsoft Edge only.
    pub fn is_edge_only(&self) -> bool {
        self.os == Os::Windows
    }

    /// Whether automatic discovery may offer a browser of this kind.
    pub fn allows_discovered(&self, kind: BrowserKind) -> bool {
        if self.is_edge_only() {
            return kind == BrowserKind::Edge;
        }
        !matches!(kind, BrowserKind::Custom)
    }

    /// Platform string of the Chrome for Testing downloads, `None` where no
    /// managed install exists: Windows (Edge only). Linux ARM64 has an
    /// official build from Chrome for Testing 153.0.8001.0 on.
    pub fn managed_platform_string(&self) -> Option<&'static str> {
        match (self.os, self.arch) {
            (Os::MacOs, Arch::Aarch64) => Some("mac-arm64"),
            (Os::Linux, Arch::X86_64) => Some("linux64"),
            (Os::Linux, Arch::Aarch64) => Some("linux-arm64"),
            _ => None,
        }
    }

    /// Why no managed browser can be installed or used here (`None` when one can).
    pub fn managed_unavailable_reason(&self) -> Option<String> {
        if let Some(reason) = self.unsupported_reason() {
            return Some(reason);
        }
        if self.is_edge_only() {
            return Some(
                "managed browsers are not available on Windows: only Microsoft Edge is supported. \
                 Install Microsoft Edge or pass its msedge.exe with --browser-path"
                    .to_string(),
            );
        }
        if self.managed_platform_string().is_none() {
            return Some(format!(
                "no managed browser exists for {}: Google publishes no Chrome for Testing build \
                 for it. Install Chromium with the system package manager or pass --browser-path",
                self.describe()
            ));
        }
        None
    }

    /// Policy for an explicitly given executable (`--browser-path`, env vars).
    pub fn check_explicit(&self, path: &str) -> ExplicitPolicy {
        if !self.is_edge_only() {
            return ExplicitPolicy::Allowed;
        }
        match classify_path(path) {
            Some(BrowserKind::Edge) => ExplicitPolicy::Allowed,
            Some(other) => ExplicitPolicy::Rejected(format!(
                "{} is not supported on Windows: only Microsoft Edge is. Install Microsoft Edge \
                 or pass the path of msedge.exe (got '{}')",
                other.display_name(),
                path
            )),
            None => ExplicitPolicy::Rejected(format!(
                "'{}' cannot be identified as Microsoft Edge. On Windows only Microsoft Edge is \
                 supported; pass the path of msedge.exe",
                path
            )),
        }
    }
}

/// Kind of browser a path points at, judged by its file name (`/` and `\`
/// both separate, so a Windows path classifies the same on any host).
pub fn classify_path(path: &str) -> Option<BrowserKind> {
    let name = path.rsplit(['/', '\\']).next().unwrap_or(path);
    let lower = name.to_lowercase();
    let stem = lower.strip_suffix(".exe").unwrap_or(&lower);
    if stem.contains("headless-shell") {
        Some(BrowserKind::HeadlessShell)
    } else if stem == "msedge" || stem.starts_with("microsoft-edge") || stem == "microsoft edge" {
        Some(BrowserKind::Edge)
    } else if stem.contains("for testing") || stem.contains("for-testing") {
        Some(BrowserKind::ChromeForTesting)
    } else if stem.contains("ungoogled") {
        Some(BrowserKind::UngoogledChromium)
    } else if stem.starts_with("chromium") || stem == "org.chromium.chromium" {
        Some(BrowserKind::Chromium)
    } else if stem == "chrome" || stem.starts_with("google-chrome") || stem == "google chrome" {
        Some(BrowserKind::Chrome)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MAC_ARM: Platform = Platform::new(Os::MacOs, Arch::Aarch64);
    const MAC_INTEL: Platform = Platform::new(Os::MacOs, Arch::X86_64);
    const LINUX_X64: Platform = Platform::new(Os::Linux, Arch::X86_64);
    const LINUX_ARM: Platform = Platform::new(Os::Linux, Arch::Aarch64);
    const WIN_X64: Platform = Platform::new(Os::Windows, Arch::X86_64);
    const WIN_ARM: Platform = Platform::new(Os::Windows, Arch::Aarch64);

    #[test]
    fn support_matrix_follows_the_platform_contract() {
        assert!(MAC_ARM.is_supported());
        assert!(LINUX_X64.is_supported());
        assert!(LINUX_ARM.is_supported());
        assert!(WIN_X64.is_supported());
        assert!(!MAC_INTEL.is_supported());
        assert!(!WIN_ARM.is_supported());
        assert!(MAC_INTEL
            .unsupported_reason()
            .unwrap()
            .contains("macOS on x86_64"));
    }

    #[test]
    fn windows_discovers_edge_only() {
        assert!(WIN_X64.allows_discovered(BrowserKind::Edge));
        for kind in [
            BrowserKind::Chrome,
            BrowserKind::Chromium,
            BrowserKind::UngoogledChromium,
        ] {
            assert!(!WIN_X64.allows_discovered(kind));
            assert!(MAC_ARM.allows_discovered(kind));
            assert!(LINUX_X64.allows_discovered(kind));
        }
    }

    #[test]
    fn managed_installs_exist_on_mac_arm_and_linux_but_not_windows() {
        assert_eq!(MAC_ARM.managed_platform_string(), Some("mac-arm64"));
        assert_eq!(LINUX_X64.managed_platform_string(), Some("linux64"));
        assert_eq!(LINUX_ARM.managed_platform_string(), Some("linux-arm64"));
        assert!(MAC_ARM.managed_unavailable_reason().is_none());
        assert!(LINUX_ARM.managed_unavailable_reason().is_none());
        assert!(WIN_X64
            .managed_unavailable_reason()
            .unwrap()
            .contains("Microsoft Edge"));
    }

    #[test]
    fn windows_explicit_chrome_is_rejected_with_an_actionable_message() {
        let ExplicitPolicy::Rejected(msg) =
            WIN_X64.check_explicit(r"C:\Program Files\Google\Chrome\Application\chrome.exe")
        else {
            panic!("chrome.exe must be rejected on Windows");
        };
        assert!(
            msg.contains("Google Chrome is not supported on Windows"),
            "{msg}"
        );
        assert!(msg.contains("msedge.exe"), "{msg}");
    }

    #[test]
    fn windows_explicit_edge_is_allowed_and_unknown_names_are_not() {
        assert_eq!(
            WIN_X64.check_explicit(r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe"),
            ExplicitPolicy::Allowed
        );
        assert!(matches!(
            WIN_X64.check_explicit(r"D:\tools\browser.exe"),
            ExplicitPolicy::Rejected(_)
        ));
        assert_eq!(
            MAC_ARM.check_explicit("/anything/browser"),
            ExplicitPolicy::Allowed
        );
    }

    #[test]
    fn path_classification_separates_chrome_headless_shell_and_edge() {
        assert_eq!(
            classify_path("/usr/bin/google-chrome-stable"),
            Some(BrowserKind::Chrome)
        );
        assert_eq!(
            classify_path("/x/chrome-headless-shell"),
            Some(BrowserKind::HeadlessShell)
        );
        assert_eq!(classify_path(r"C:\x\msedge.exe"), Some(BrowserKind::Edge));
        assert_eq!(
            classify_path("/usr/bin/chromium-browser"),
            Some(BrowserKind::Chromium)
        );
        assert_eq!(
            classify_path("/x/Google Chrome for Testing"),
            Some(BrowserKind::ChromeForTesting)
        );
        assert_eq!(classify_path("/x/unknown"), None);
    }
}
