//! Browser installer - explicit Chrome for Testing download
//!
//! Downloads to ~/.auditmysite/browsers/ — only when explicitly requested
//! via `auditmysite browser install`. No auto-download.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

use tracing::{info, warn};

use super::types::InstallTarget;
use crate::error::{AuditError, Result};

/// Chrome for Testing API endpoint
const CFT_API_URL: &str =
    "https://googlechromelabs.github.io/chrome-for-testing/last-known-good-versions-with-downloads.json";

/// Fallback version if API is unavailable
const FALLBACK_VERSION: &str = "131.0.6778.108";

/// Browser installer
pub struct BrowserInstaller;

impl BrowserInstaller {
    /// Install a browser to ~/.auditmysite/browsers/
    pub async fn install(
        target: InstallTarget,
        version: Option<&str>,
        force: bool,
    ) -> Result<PathBuf> {
        let base_dir = Self::browsers_dir()?;
        let (target_name, channel) = match target {
            InstallTarget::ChromeForTesting => ("chrome-for-testing", "chrome"),
            InstallTarget::HeadlessShell => ("headless-shell", "chrome-headless-shell"),
        };

        let target_dir = base_dir.join(target_name);

        // Check if already installed
        if target_dir.exists() && !force {
            let existing_version = fs::read_to_string(target_dir.join("version.txt"))
                .unwrap_or_default()
                .trim()
                .to_string();

            if !existing_version.is_empty() {
                println!("Already installed: {} v{}", target_name, existing_version);
                println!("Use --force to reinstall.");

                let binary_path = Self::binary_path(&target_dir, target);
                return Ok(binary_path);
            }
        }

        // Resolve version
        let resolved_version = if let Some(v) = version {
            v.to_string()
        } else {
            Self::fetch_latest_version().await.unwrap_or_else(|e| {
                warn!("Could not fetch latest version from API: {}", e);
                info!("Using fallback version: {}", FALLBACK_VERSION);
                FALLBACK_VERSION.to_string()
            })
        };

        println!("Installing {} v{}...", target_name, resolved_version);

        // Build download URL
        let download_url = Self::build_download_url(&resolved_version, channel);
        let archive_name = format!("{}.zip", target_name);

        // Prepare directory
        if target_dir.exists() {
            fs::remove_dir_all(&target_dir).map_err(|e| AuditError::BrowserLaunchFailed {
                reason: format!("Failed to clean existing install: {}", e),
            })?;
        }
        fs::create_dir_all(&target_dir).map_err(|e| AuditError::BrowserLaunchFailed {
            reason: format!("Failed to create install directory: {}", e),
        })?;

        let archive_path = target_dir.join(&archive_name);

        // Download
        println!("Downloading from: {}", download_url);
        Self::download_file(&download_url, &archive_path).await?;

        // Extract
        println!("Extracting...");
        Self::extract_archive(&archive_path, &target_dir)?;

        // Clean up archive
        fs::remove_file(&archive_path).ok();

        // Write version file
        fs::write(target_dir.join("version.txt"), &resolved_version).ok();

        // Make executable
        let binary_path = Self::binary_path(&target_dir, target);
        Self::set_executable(&binary_path)?;

        // macOS: remove quarantine attribute
        #[cfg(target_os = "macos")]
        Self::remove_quarantine(&target_dir);

        println!(
            "Installed {} v{} at {}",
            target_name,
            resolved_version,
            binary_path.display()
        );

        Ok(binary_path)
    }

    /// Remove a managed browser installation
    pub fn remove(target: InstallTarget) -> Result<()> {
        let base_dir = Self::browsers_dir()?;
        let target_name = match target {
            InstallTarget::ChromeForTesting => "chrome-for-testing",
            InstallTarget::HeadlessShell => "headless-shell",
        };
        let target_dir = base_dir.join(target_name);

        if target_dir.exists() {
            fs::remove_dir_all(&target_dir).map_err(|e| AuditError::BrowserLaunchFailed {
                reason: format!("Failed to remove {}: {}", target_name, e),
            })?;
            println!("Removed {}", target_name);
        } else {
            println!("{} is not installed", target_name);
        }

        Ok(())
    }

    /// Remove all managed browsers including legacy
    pub fn remove_all() -> Result<()> {
        Self::remove(InstallTarget::ChromeForTesting)?;
        Self::remove(InstallTarget::HeadlessShell)?;

        // Also clean up legacy location
        if let Ok(legacy_dir) = Self::legacy_chromium_dir() {
            if legacy_dir.exists() {
                fs::remove_dir_all(&legacy_dir).ok();
                println!("Removed legacy chromium install");
            }
        }

        Ok(())
    }

    /// Fetch latest stable version from Chrome for Testing API
    async fn fetch_latest_version() -> Result<String> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| AuditError::BrowserLaunchFailed {
                reason: format!("HTTP client error: {}", e),
            })?;

        let resp: serde_json::Value = client
            .get(CFT_API_URL)
            .send()
            .await
            .map_err(|e| AuditError::BrowserLaunchFailed {
                reason: format!("Failed to fetch versions: {}", e),
            })?
            .json()
            .await
            .map_err(|e| AuditError::BrowserLaunchFailed {
                reason: format!("Failed to parse versions: {}", e),
            })?;

        resp["channels"]["Stable"]["version"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| AuditError::BrowserLaunchFailed {
                reason: "Could not find stable version in API response".to_string(),
            })
    }

    fn build_download_url(version: &str, channel: &str) -> String {
        let platform = Self::platform_string();
        format!(
            "https://storage.googleapis.com/chrome-for-testing-public/{}/{}/{}-{}.zip",
            version, platform, channel, platform
        )
    }

    fn platform_string() -> &'static str {
        if cfg!(target_os = "macos") {
            if cfg!(target_arch = "aarch64") {
                "mac-arm64"
            } else {
                "mac-x64"
            }
        } else if cfg!(target_os = "linux") {
            "linux64"
        } else {
            "win64"
        }
    }

    fn binary_path(target_dir: &Path, target: InstallTarget) -> PathBuf {
        let platform = Self::platform_string();

        match target {
            InstallTarget::ChromeForTesting => {
                let inner_dir = format!("chrome-{}", platform);
                if cfg!(target_os = "macos") {
                    target_dir
                        .join(&inner_dir)
                        .join("Google Chrome for Testing.app")
                        .join("Contents")
                        .join("MacOS")
                        .join("Google Chrome for Testing")
                } else if cfg!(target_os = "windows") {
                    target_dir.join(&inner_dir).join("chrome.exe")
                } else {
                    target_dir.join(&inner_dir).join("chrome")
                }
            }
            InstallTarget::HeadlessShell => {
                let inner_dir = format!("chrome-headless-shell-{}", platform);
                if cfg!(target_os = "windows") {
                    target_dir
                        .join(&inner_dir)
                        .join("chrome-headless-shell.exe")
                } else {
                    target_dir.join(&inner_dir).join("chrome-headless-shell")
                }
            }
        }
    }

    fn browsers_dir() -> Result<PathBuf> {
        let dir = dirs::home_dir()
            .ok_or_else(|| AuditError::ConfigError("Could not find home directory".to_string()))?
            .join(".auditmysite")
            .join("browsers");
        Ok(dir)
    }

    fn legacy_chromium_dir() -> Result<PathBuf> {
        let dir = dirs::home_dir()
            .ok_or_else(|| AuditError::ConfigError("Could not find home directory".to_string()))?
            .join(".auditmysite")
            .join("chromium");
        Ok(dir)
    }

    async fn download_file(url: &str, dest: &Path) -> Result<()> {
        use futures::StreamExt;

        let max_retries = 2;
        let mut last_error = None;
        let mut response = None;

        for attempt in 0..=max_retries {
            if attempt > 0 {
                let delay = Duration::from_secs(2u64.pow(attempt as u32));
                warn!(
                    "Retrying download (attempt {}/{}, backoff {:?})",
                    attempt + 1,
                    max_retries + 1,
                    delay
                );
                tokio::time::sleep(delay).await;
            }

            match reqwest::get(url).await {
                Ok(resp) if resp.status().is_success() => {
                    response = Some(resp);
                    break;
                }
                Ok(resp) => {
                    last_error = Some(format!("HTTP {}", resp.status()));
                }
                Err(e) => {
                    last_error = Some(format!("Download failed: {}", e));
                }
            }
        }

        let response = response.ok_or_else(|| AuditError::BrowserLaunchFailed {
            reason: last_error.unwrap_or_else(|| "Download failed after retries".to_string()),
        })?;

        let total_size = response.content_length().unwrap_or(0);
        let mut downloaded = 0u64;

        let mut file = fs::File::create(dest).map_err(|e| AuditError::BrowserLaunchFailed {
            reason: format!("Failed to create file: {}", e),
        })?;

        let mut stream = response.bytes_stream();

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.map_err(|e| AuditError::BrowserLaunchFailed {
                reason: format!("Download chunk failed: {}", e),
            })?;

            file.write_all(&chunk)
                .map_err(|e| AuditError::BrowserLaunchFailed {
                    reason: format!("Write failed: {}", e),
                })?;

            downloaded += chunk.len() as u64;

            if total_size > 0 {
                let percent = (downloaded * 100) / total_size;
                if downloaded % (total_size / 10).max(1) < chunk.len() as u64 {
                    println!(
                        "  {}% ({}/{} MB)",
                        percent,
                        downloaded / 1_000_000,
                        total_size / 1_000_000
                    );
                }
            }
        }

        Ok(())
    }

    fn extract_archive(archive_path: &Path, dest_dir: &Path) -> Result<()> {
        let file = fs::File::open(archive_path).map_err(|e| AuditError::BrowserLaunchFailed {
            reason: format!("Failed to open archive: {}", e),
        })?;

        let mut archive =
            zip::ZipArchive::new(file).map_err(|e| AuditError::BrowserLaunchFailed {
                reason: format!("Failed to read zip: {}", e),
            })?;

        let total_files = archive.len();

        for i in 0..total_files {
            let mut file = archive
                .by_index(i)
                .map_err(|e| AuditError::BrowserLaunchFailed {
                    reason: format!("Failed to read file from archive: {}", e),
                })?;

            let outpath = match file.enclosed_name() {
                Some(path) => dest_dir.join(path),
                None => continue,
            };

            if file.name().ends_with('/') {
                fs::create_dir_all(&outpath).ok();
            } else {
                if let Some(p) = outpath.parent() {
                    fs::create_dir_all(p).ok();
                }
                let mut outfile =
                    fs::File::create(&outpath).map_err(|e| AuditError::BrowserLaunchFailed {
                        reason: format!("Failed to create extracted file: {}", e),
                    })?;
                std::io::copy(&mut file, &mut outfile).map_err(|e| {
                    AuditError::BrowserLaunchFailed {
                        reason: format!("Failed to extract file: {}", e),
                    }
                })?;
            }
        }

        Ok(())
    }

    fn set_executable(path: &Path) -> Result<()> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if path.exists() {
                let mut perms = fs::metadata(path)
                    .map_err(|e| AuditError::BrowserLaunchFailed {
                        reason: format!("Failed to get permissions: {}", e),
                    })?
                    .permissions();
                perms.set_mode(0o755);
                fs::set_permissions(path, perms).map_err(|e| AuditError::BrowserLaunchFailed {
                    reason: format!("Failed to set permissions: {}", e),
                })?;
            }
        }
        Ok(())
    }

    #[cfg(target_os = "macos")]
    fn remove_quarantine(dir: &Path) {
        use std::process::Command;
        // Remove macOS quarantine attribute recursively
        let _ = Command::new("xattr")
            .args(["-rd", "com.apple.quarantine"])
            .arg(dir)
            .output();
    }
}

// ── Legacy compatibility ──────────────────────────────────────

/// Legacy: Chromium installer (redirects to new installer)
pub struct ChromiumInstaller;

impl ChromiumInstaller {
    /// Legacy: Ensure Chromium is available
    /// Now uses the new BrowserInstaller but maintains the old API
    pub async fn ensure_chromium() -> Result<PathBuf> {
        // Check new location first
        let browsers_dir = dirs::home_dir()
            .ok_or_else(|| AuditError::ConfigError("Could not find home directory".to_string()))?
            .join(".auditmysite")
            .join("browsers")
            .join("chrome-for-testing");

        if browsers_dir.exists() {
            let binary =
                BrowserInstaller::binary_path(&browsers_dir, InstallTarget::ChromeForTesting);
            if binary.exists() {
                return Ok(binary);
            }
        }

        // Check legacy location
        let legacy_path = Self::local_chromium_path()?;
        if legacy_path.exists() {
            return Ok(legacy_path);
        }

        // Install to new location
        BrowserInstaller::install(InstallTarget::ChromeForTesting, None, false).await
    }

    fn local_chromium_path() -> Result<PathBuf> {
        let cache_dir = dirs::home_dir()
            .ok_or_else(|| AuditError::ConfigError("Could not find home directory".to_string()))?
            .join(".auditmysite")
            .join("chromium");

        Ok(if cfg!(target_os = "macos") {
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
}
