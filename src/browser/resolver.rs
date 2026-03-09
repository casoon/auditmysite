//! Browser resolver - finds the best available browser
//!
//! Priority order:
//! 1. --browser-path (explicit CLI flag)
//! 2. AUDITMYSITE_BROWSER env var
//! 3. CHROME_PATH env var (deprecated, backwards compat)
//! 4. System scan (Chrome → Edge → Ungoogled Chromium → Chromium)
//! 5. Managed install (~/.auditmysite/browsers/)
//! 6. Error with installation hints

use std::path::PathBuf;

use tracing::{debug, info, warn};

use super::detection::{
    detect_all_browsers, get_browser_version, validate_browser, verify_executable,
};
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

    // 5. System scan
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

    // 6. Managed install check (not in strict mode)
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

    // 7. Nothing found
    Err(AuditError::ChromeNotFound)
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
}
