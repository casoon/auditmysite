//! Browser detection across platforms
//!
//! Scans system paths, PATH, and managed installs for Chromium-based browsers.
//! Returns all found browsers in priority order.

use std::path::PathBuf;
use std::process::Command;

use tracing::{debug, warn};

use super::registry;
use super::types::{BrowserKind, BrowserSource, DetectedBrowser};
use crate::error::{AuditError, Result};

/// Detect all available browsers on the system
///
/// Returns a list of detected browsers in priority order:
/// 1. System paths (Chrome → Edge → Ungoogled Chromium → Chromium)
/// 2. PATH search via `which`
///
/// Does NOT include managed installs (handled by resolver).
pub fn detect_all_browsers() -> Vec<DetectedBrowser> {
    let mut found = Vec::new();
    let mut seen_paths = std::collections::HashSet::new();

    // 1. Scan known system paths
    for entry in registry::system_browser_paths() {
        let path = PathBuf::from(entry.path);
        if path.exists() && seen_paths.insert(path.clone()) {
            let version = get_browser_version(&path);
            debug!(
                "Found {} at {} (v{})",
                entry.kind.display_name(),
                path.display(),
                version.as_deref().unwrap_or("unknown")
            );
            found.push(DetectedBrowser {
                kind: entry.kind,
                path,
                version,
                source: BrowserSource::SystemPath,
            });
        }
    }

    // 2. Search via `which` for each browser kind in priority order
    for &kind in registry::search_order() {
        for &name in registry::which_names(kind) {
            if let Some(path) = which_binary(name) {
                if seen_paths.insert(path.clone()) {
                    let version = get_browser_version(&path);
                    debug!(
                        "Found {} via which: {} (v{})",
                        kind.display_name(),
                        path.display(),
                        version.as_deref().unwrap_or("unknown")
                    );
                    found.push(DetectedBrowser {
                        kind,
                        path,
                        version,
                        source: BrowserSource::PathSearch,
                    });
                }
            }
        }
    }

    found
}

/// Validate a browser binary at a given path
pub fn validate_browser(
    path: &PathBuf,
    kind: BrowserKind,
    source: BrowserSource,
) -> Result<DetectedBrowser> {
    if !path.exists() {
        return Err(AuditError::FileError {
            path: path.clone(),
            reason: "Browser binary not found at specified path".to_string(),
        });
    }

    verify_executable(path)?;

    let version = get_browser_version(path);

    Ok(DetectedBrowser {
        kind,
        path: path.clone(),
        version,
        source,
    })
}

/// Verify that a binary is executable
pub fn verify_executable(path: &PathBuf) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = std::fs::metadata(path).map_err(|e| AuditError::FileError {
            path: path.clone(),
            reason: e.to_string(),
        })?;
        if metadata.permissions().mode() & 0o111 == 0 {
            return Err(AuditError::ChromeNotExecutable { path: path.clone() });
        }
    }
    Ok(())
}

/// Get browser version by running `--version`
pub fn get_browser_version(path: &PathBuf) -> Option<String> {
    Command::new(path)
        .arg("--version")
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                let version_str = String::from_utf8_lossy(&output.stdout);
                // Extract version number from strings like "Google Chrome 122.0.6261.94"
                version_str
                    .split_whitespace()
                    .find(|s| s.chars().next().is_some_and(|c| c.is_ascii_digit()))
                    .map(|s| s.to_string())
            } else {
                None
            }
        })
}

/// Find a binary via `which` (Unix) or `where` (Windows)
fn which_binary(name: &str) -> Option<PathBuf> {
    let cmd = if cfg!(target_os = "windows") {
        "where"
    } else {
        "which"
    };

    Command::new(cmd)
        .arg(name)
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if path.is_empty() {
                    None
                } else {
                    // `where` on Windows may return multiple lines
                    Some(PathBuf::from(path.lines().next().unwrap_or(&path)))
                }
            } else {
                None
            }
        })
}

// ── Legacy compatibility ──────────────────────────────────────

/// Legacy: Information about a detected Chrome installation
#[derive(Debug, Clone)]
pub struct ChromeInfo {
    pub path: PathBuf,
    pub version: Option<String>,
    pub detection_method: DetectionMethod,
}

/// Legacy: How Chrome was detected
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetectionMethod {
    ManualPath,
    EnvironmentVariable,
    StandardPath,
    WhichCommand,
    AutoDownload,
}

impl From<&DetectedBrowser> for ChromeInfo {
    fn from(browser: &DetectedBrowser) -> Self {
        ChromeInfo {
            path: browser.path.clone(),
            version: browser.version.clone(),
            detection_method: match browser.source {
                BrowserSource::CliFlag => DetectionMethod::ManualPath,
                BrowserSource::EnvVar => DetectionMethod::EnvironmentVariable,
                BrowserSource::SystemPath => DetectionMethod::StandardPath,
                BrowserSource::PathSearch => DetectionMethod::WhichCommand,
                BrowserSource::ManagedInstall => DetectionMethod::AutoDownload,
            },
        }
    }
}

/// Legacy: Detect Chrome in standard system paths
pub fn detect_chrome() -> Option<PathBuf> {
    detect_all_browsers().into_iter().next().map(|b| b.path)
}

/// Legacy: Find Chrome using all available methods
pub fn find_chrome(manual_path: Option<&str>) -> Result<ChromeInfo> {
    // 1. Check manual path first
    if let Some(path_str) = manual_path {
        let path = PathBuf::from(path_str);
        if path.exists() {
            let version = get_browser_version(&path);
            return Ok(ChromeInfo {
                path,
                version,
                detection_method: DetectionMethod::ManualPath,
            });
        } else {
            return Err(AuditError::FileError {
                path,
                reason: "Chrome binary not found at specified path".to_string(),
            });
        }
    }

    // 2. Check CHROME_PATH environment variable
    if let Ok(path_str) = std::env::var("CHROME_PATH") {
        let path = PathBuf::from(&path_str);
        if path.exists() {
            warn!("CHROME_PATH is deprecated, use AUDITMYSITE_BROWSER instead");
            let version = get_browser_version(&path);
            return Ok(ChromeInfo {
                path,
                version,
                detection_method: DetectionMethod::EnvironmentVariable,
            });
        }
    }

    // 3. Use new detection
    let browsers = detect_all_browsers();
    if let Some(browser) = browsers.into_iter().next() {
        return Ok(ChromeInfo::from(&browser));
    }

    Err(AuditError::ChromeNotFound)
}

/// Legacy: Verify Chrome is executable (kept for backward compatibility)
#[allow(dead_code)]
pub fn verify_chrome_executable(path: &PathBuf) -> Result<()> {
    verify_executable(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_all_browsers_runs() {
        // Should not panic
        let _ = detect_all_browsers();
    }

    #[test]
    fn test_find_chrome_with_invalid_manual_path() {
        let result = find_chrome(Some("/nonexistent/path/to/chrome"));
        assert!(result.is_err());
    }

    #[test]
    fn test_detection_method_display() {
        assert_eq!(format!("{:?}", DetectionMethod::ManualPath), "ManualPath");
    }

    #[test]
    fn test_validate_browser_nonexistent() {
        let result = validate_browser(
            &PathBuf::from("/nonexistent"),
            BrowserKind::Chrome,
            BrowserSource::CliFlag,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_chrome_info_from_detected_browser() {
        let browser = DetectedBrowser {
            kind: BrowserKind::Chrome,
            path: PathBuf::from("/usr/bin/chrome"),
            version: Some("131.0".to_string()),
            source: BrowserSource::SystemPath,
        };
        let info = ChromeInfo::from(&browser);
        assert_eq!(info.detection_method, DetectionMethod::StandardPath);
        assert_eq!(info.path, PathBuf::from("/usr/bin/chrome"));
    }
}
