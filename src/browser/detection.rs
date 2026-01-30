//! Chrome/Chromium binary detection across platforms
//!
//! Supports automatic detection on macOS, Linux, and Windows,
//! plus manual override via CLI flag or environment variable.

use std::path::PathBuf;
use std::process::Command;

use crate::error::{AuditError, Result};

/// Information about a detected Chrome installation
#[derive(Debug, Clone)]
pub struct ChromeInfo {
    /// Path to the Chrome binary
    pub path: PathBuf,
    /// Chrome version string (e.g., "122.0.6261.94")
    pub version: Option<String>,
    /// Detection method used
    pub detection_method: DetectionMethod,
}

/// How Chrome was detected
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetectionMethod {
    /// User provided via CLI --chrome-path
    ManualPath,
    /// Found via CHROME_PATH environment variable
    EnvironmentVariable,
    /// Found in standard system paths
    StandardPath,
    /// Found via `which` command
    WhichCommand,
    /// Auto-downloaded by chromiumoxide to ~/.cache/chromiumoxide/
    AutoDownload,
}

/// Standard Chrome/Chromium paths for each platform
fn get_standard_paths() -> Vec<&'static str> {
    if cfg!(target_os = "macos") {
        vec![
            "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
            "/Applications/Chromium.app/Contents/MacOS/Chromium",
            "/Applications/Google Chrome Canary.app/Contents/MacOS/Google Chrome Canary",
            "/opt/homebrew/bin/chromium",
            "/usr/local/bin/chromium",
        ]
    } else if cfg!(target_os = "linux") {
        vec![
            "/usr/bin/google-chrome",
            "/usr/bin/google-chrome-stable",
            "/usr/bin/chromium",
            "/usr/bin/chromium-browser",
            "/snap/bin/chromium",
            "/usr/bin/chrome",
            "/var/lib/flatpak/exports/bin/org.chromium.Chromium",
        ]
    } else if cfg!(target_os = "windows") {
        vec![
            r"C:\Program Files\Google\Chrome\Application\chrome.exe",
            r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
        ]
    } else {
        vec![]
    }
}

/// Detect Chrome in standard system paths
pub fn detect_chrome() -> Option<PathBuf> {
    get_standard_paths()
        .iter()
        .map(PathBuf::from)
        .find(|p| p.exists())
}

/// Detect Chrome using the `which` command (Unix-like systems)
fn detect_chrome_via_which() -> Option<PathBuf> {
    let names = [
        "google-chrome",
        "google-chrome-stable",
        "chromium",
        "chromium-browser",
        "chrome",
    ];

    for name in names {
        if let Ok(output) = Command::new("which").arg(name).output() {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path.is_empty() {
                    return Some(PathBuf::from(path));
                }
            }
        }
    }

    None
}

/// Get Chrome version from binary
fn get_chrome_version(path: &PathBuf) -> Option<String> {
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
                    .find(|s| {
                        s.chars()
                            .next()
                            .map(|c| c.is_ascii_digit())
                            .unwrap_or(false)
                    })
                    .map(|s| s.to_string())
            } else {
                None
            }
        })
}

/// Find Chrome using all available methods
///
/// Priority order:
/// 1. Manual path (if provided)
/// 2. CHROME_PATH environment variable
/// 3. Standard system paths
/// 4. `which` command
///
/// # Arguments
/// * `manual_path` - Optional path provided via CLI --chrome-path
///
/// # Returns
/// * `Ok(ChromeInfo)` with path and version if found
/// * `Err(AuditError::ChromeNotFound)` if not found
pub fn find_chrome(manual_path: Option<&str>) -> Result<ChromeInfo> {
    // 1. Check manual path first
    if let Some(path_str) = manual_path {
        let path = PathBuf::from(path_str);
        if path.exists() {
            let version = get_chrome_version(&path);
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
            let version = get_chrome_version(&path);
            return Ok(ChromeInfo {
                path,
                version,
                detection_method: DetectionMethod::EnvironmentVariable,
            });
        }
    }

    // 3. Check standard system paths
    if let Some(path) = detect_chrome() {
        let version = get_chrome_version(&path);
        return Ok(ChromeInfo {
            path,
            version,
            detection_method: DetectionMethod::StandardPath,
        });
    }

    // 4. Try `which` command
    if let Some(path) = detect_chrome_via_which() {
        let version = get_chrome_version(&path);
        return Ok(ChromeInfo {
            path,
            version,
            detection_method: DetectionMethod::WhichCommand,
        });
    }

    // Chrome not found
    Err(AuditError::ChromeNotFound)
}

/// Verify that the Chrome binary is executable
pub fn verify_chrome_executable(path: &PathBuf) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = std::fs::metadata(path).map_err(|e| AuditError::FileError {
            path: path.clone(),
            reason: e.to_string(),
        })?;
        let permissions = metadata.permissions();
        if permissions.mode() & 0o111 == 0 {
            return Err(AuditError::ChromeNotExecutable { path: path.clone() });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_standard_paths_not_empty() {
        let paths = get_standard_paths();
        // Should have paths for all platforms
        assert!(
            !paths.is_empty()
                || cfg!(not(any(
                    target_os = "macos",
                    target_os = "linux",
                    target_os = "windows"
                )))
        );
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
}
