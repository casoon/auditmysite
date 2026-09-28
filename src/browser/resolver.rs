//! Browser resolver - finds the best available browser
//!
//! Priority order:
//! 1. --browser-path (explicit CLI flag)
//! 2. AUDITMYSITE_BROWSER env var
//! 3. CHROME_PATH env var (deprecated, backwards compat)
//! 4. Managed headless-shell (`browser install --headless-shell`), unless a
//!    browser kind was requested or `--strict` is set
//! 5. System scan (Chrome → Edge → Ungoogled Chromium → Chromium)
//! 6. Managed Chrome for Testing (~/.auditmysite/browsers/)
//! 7. Error with installation hints
//!
//! Why headless-shell goes first: a full Chrome in `--headless=new` still runs
//! the macOS event loop, and keyboard events the page does not consume are
//! routed through AppKit and the WindowServer on the browser's single main
//! thread. Under batch concurrency that stalled the whole browser — every
//! page's journeys timed out in the same second, twice it never recovered
//! (plan 65, reference batch: 4 of 4 runs affected; headless-shell 3 of 3
//! clean, 38 s, identical accessibility findings).

use std::path::PathBuf;

use tracing::{debug, info, warn};

use super::detection::{
    detect_all_browsers, get_browser_version, validate_browser, verify_executable,
};
use super::installer::BrowserInstaller;
use super::types::*;
use crate::error::{AuditError, Result};

/// Options that affect browser resolution
#[derive(Default)]
pub struct BrowserResolveOptions {
    /// --browser-path /explicit/path
    pub browser_path: Option<String>,
    /// --browser chrome|edge|chromium|auto
    pub browser_preference: Option<String>,
    /// --strict (system browser only, no managed fallback)
    pub strict: bool,
}

/// Find the best available browser
pub fn resolve_browser(opts: &BrowserResolveOptions) -> Result<ResolvedBrowser> {
    let mode = if opts.strict {
        BrowserMode::Strict
    } else {
        BrowserMode::Standard
    };

    // 1. Explicit path (highest priority)
    if let Some(ref path_str) = opts.browser_path {
        let path = PathBuf::from(path_str);
        let browser = validate_browser(&path, BrowserKind::Custom, BrowserSource::CliFlag)?;
        info!(
            "Using specified browser: {} v{}",
            browser.kind.display_name(),
            browser.version.as_deref().unwrap_or("unknown")
        );
        return Ok(ResolvedBrowser {
            browser,
            mode,
            all_candidates: vec![],
        });
    }

    // 2. AUDITMYSITE_BROWSER env var
    if let Ok(path_str) = std::env::var("AUDITMYSITE_BROWSER") {
        let path = PathBuf::from(&path_str);
        if path.exists() {
            let browser = validate_browser(&path, BrowserKind::Custom, BrowserSource::EnvVar)?;
            info!("Using browser from AUDITMYSITE_BROWSER: {}", path.display());
            return Ok(ResolvedBrowser {
                browser,
                mode,
                all_candidates: vec![],
            });
        }
        warn!(
            "AUDITMYSITE_BROWSER points to non-existent path: {}",
            path_str
        );
    }

    // 3. Legacy: CHROME_PATH env var
    if let Ok(path_str) = std::env::var("CHROME_PATH") {
        let path = PathBuf::from(&path_str);
        if path.exists() {
            warn!("CHROME_PATH is deprecated, use AUDITMYSITE_BROWSER or --browser-path instead");
            let browser = validate_browser(&path, BrowserKind::Chrome, BrowserSource::EnvVar)?;
            return Ok(ResolvedBrowser {
                browser,
                mode,
                all_candidates: vec![],
            });
        }
    }

    // 4. Parse browser preference filter
    let filter_kind: Option<BrowserKind> = opts.browser_preference.as_deref().and_then(|s| match s
        .to_lowercase()
        .as_str()
    {
        "chrome" => Some(BrowserKind::Chrome),
        "edge" => Some(BrowserKind::Edge),
        "chromium" => Some(BrowserKind::Chromium),
        "auto" | "" => None,
        other => {
            warn!("Unknown browser kind '{}', using auto-detection", other);
            None
        }
    });

    // 5. Managed headless-shell: installed on purpose for this tool, and the
    //    only build without the macOS event loop (see module docs).
    if mode != BrowserMode::Strict && filter_kind.is_none() {
        if let Some(shell) = check_managed_headless_shell() {
            info!("Using managed headless-shell: {}", shell.path.display());
            return Ok(ResolvedBrowser {
                browser: shell,
                mode,
                all_candidates: vec![],
            });
        }
    }

    // 6. System scan
    let all_candidates = detect_all_browsers();
    debug!("Found {} browser candidates", all_candidates.len());

    let best = if let Some(kind) = filter_kind {
        all_candidates.iter().find(|b| b.kind == kind)
    } else {
        all_candidates.first()
    };

    if let Some(browser) = best {
        info!(
            "Selected browser: {} v{} ({})",
            browser.kind.display_name(),
            browser.version.as_deref().unwrap_or("unknown"),
            browser.path.display()
        );
        return Ok(ResolvedBrowser {
            browser: browser.clone(),
            mode,
            all_candidates,
        });
    }

    // 7. Managed install check (not in strict mode)
    if mode != BrowserMode::Strict {
        if let Some(managed) = check_managed_install() {
            info!("Using managed browser: {}", managed.path.display());
            return Ok(ResolvedBrowser {
                browser: managed,
                mode,
                all_candidates,
            });
        }
    }

    // 8. Nothing found
    Err(AuditError::ChromeNotFound)
}

/// Whether `browser` can stall under keyboard-driven journeys: any full
/// browser on macOS (see module docs). `doctor` and the audit start warn.
/// A headless-shell passed via `--browser-path` is recognized by file name.
pub fn stalls_on_keyboard_journeys(browser: &DetectedBrowser) -> bool {
    let is_shell = browser.kind == BrowserKind::HeadlessShell
        || browser
            .path
            .file_name()
            .is_some_and(|name| name.to_string_lossy().starts_with("chrome-headless-shell"));
    cfg!(target_os = "macos") && !is_shell
}

/// Managed headless-shell under ~/.auditmysite/browsers/headless-shell/
fn check_managed_headless_shell() -> Option<DetectedBrowser> {
    let dir = dirs::home_dir()?
        .join(".auditmysite")
        .join("browsers")
        .join("headless-shell");
    let path = BrowserInstaller::binary_path(&dir, InstallTarget::HeadlessShell);
    if !path.exists() || verify_executable(&path).is_err() {
        return None;
    }
    Some(DetectedBrowser {
        kind: BrowserKind::HeadlessShell,
        path,
        version: read_version_file(&dir),
        source: BrowserSource::ManagedInstall,
    })
}

/// Check for managed browser installs under ~/.auditmysite/browsers/
fn check_managed_install() -> Option<DetectedBrowser> {
    let base = dirs::home_dir()?.join(".auditmysite").join("browsers");

    // Check Chrome for Testing
    let cft_path = managed_binary_path(&base, "chrome-for-testing");
    if cft_path.exists() && verify_executable(&cft_path).is_ok() {
        return Some(DetectedBrowser {
            kind: BrowserKind::ChromeForTesting,
            path: cft_path,
            version: read_version_file(&base.join("chrome-for-testing")),
            source: BrowserSource::ManagedInstall,
        });
    }

    // Check legacy location (~/.auditmysite/chromium/)
    let legacy_path = legacy_chromium_path();
    if let Some(ref path) = legacy_path {
        if path.exists() && verify_executable(path).is_ok() {
            let version = get_browser_version(path);
            return Some(DetectedBrowser {
                kind: BrowserKind::ChromeForTesting,
                path: path.clone(),
                version,
                source: BrowserSource::ManagedInstall,
            });
        }
    }

    None
}

/// Get the binary path for a managed install
fn managed_binary_path(base: &std::path::Path, name: &str) -> PathBuf {
    let dir = base.join(name);
    if cfg!(target_os = "macos") {
        if cfg!(target_arch = "aarch64") {
            dir.join("chrome-mac-arm64")
                .join("Google Chrome for Testing.app")
                .join("Contents")
                .join("MacOS")
                .join("Google Chrome for Testing")
        } else {
            dir.join("chrome-mac-x64")
                .join("Google Chrome for Testing.app")
                .join("Contents")
                .join("MacOS")
                .join("Google Chrome for Testing")
        }
    } else if cfg!(target_os = "linux") {
        dir.join("chrome-linux64").join("chrome")
    } else {
        dir.join("chrome-win64").join("chrome.exe")
    }
}

/// Get legacy Chromium binary path (~/.auditmysite/chromium/)
fn legacy_chromium_path() -> Option<PathBuf> {
    let cache_dir = dirs::home_dir()?.join(".auditmysite").join("chromium");
    Some(if cfg!(target_os = "macos") {
        cache_dir
            .join("chrome-mac")
            .join("Chromium.app")
            .join("Contents")
            .join("MacOS")
            .join("Chromium")
    } else if cfg!(target_os = "linux") {
        cache_dir.join("chrome-linux").join("chrome")
    } else {
        cache_dir.join("chrome-win").join("chrome.exe")
    })
}

/// Read version from a version.txt file in a managed install directory
fn read_version_file(dir: &std::path::Path) -> Option<String> {
    std::fs::read_to_string(dir.join("version.txt"))
        .ok()
        .map(|s| s.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_with_invalid_explicit_path() {
        let opts = BrowserResolveOptions {
            browser_path: Some("/nonexistent/browser".to_string()),
            ..Default::default()
        };
        assert!(resolve_browser(&opts).is_err());
    }

    #[test]
    fn test_managed_binary_path_format() {
        let base = PathBuf::from("/home/user/.auditmysite/browsers");
        let path = managed_binary_path(&base, "chrome-for-testing");
        assert!(path.to_string_lossy().contains("chrome-for-testing"));
    }

    fn browser(kind: BrowserKind, path: &str) -> DetectedBrowser {
        DetectedBrowser {
            kind,
            path: PathBuf::from(path),
            version: None,
            source: BrowserSource::SystemPath,
        }
    }

    #[test]
    fn headless_shell_never_counts_as_stalling() {
        let managed = browser(BrowserKind::HeadlessShell, "/x/chrome-headless-shell");
        let explicit = browser(BrowserKind::Custom, "/cache/chrome-headless-shell");
        assert!(!stalls_on_keyboard_journeys(&managed));
        assert!(!stalls_on_keyboard_journeys(&explicit));
    }

    #[test]
    fn full_chrome_stalls_only_on_macos() {
        let chrome = browser(
            BrowserKind::Chrome,
            "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
        );
        assert_eq!(
            stalls_on_keyboard_journeys(&chrome),
            cfg!(target_os = "macos")
        );
    }
}
