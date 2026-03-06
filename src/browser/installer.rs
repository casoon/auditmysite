//! Chromium installer - downloads isolated Chromium binary
//!
//! Downloads Chromium to ~/.auditmysite/chromium/ without affecting system Chrome.
//! Uses Chrome for Testing stable builds.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

use tracing::{info, warn};

use crate::error::{AuditError, Result};

/// Chromium installation manager
pub struct ChromiumInstaller;

impl ChromiumInstaller {
    /// Ensure Chromium is available (check cache, ask user, download if needed)
    pub async fn ensure_chromium() -> Result<PathBuf> {
        // 1. Check if already downloaded
        let local_path = Self::local_chromium_path()?;
        if local_path.exists() {
            info!("Found cached Chromium at: {}", local_path.display());
            return Ok(local_path);
        }

        // 2. Chromium not found - inform user and auto-download
        Self::log_download_info();

        // 3. Download
        Self::download_chromium().await
    }

    /// Get path to local Chromium installation
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
            // Windows
            cache_dir.join("chrome-win").join("chrome.exe")
        })
    }

    /// Log info about Chromium auto-download
    fn log_download_info() {
        info!("Chrome/Chromium not found on system, auto-downloading to ~/.auditmysite/");
        info!("To use an existing Chrome installation, pass --chrome-path or set CHROME_PATH");
    }

    /// Download Chromium binary
    async fn download_chromium() -> Result<PathBuf> {
        use futures::StreamExt;

        info!("Downloading Chromium...");

        // Chrome for Testing URLs (stable builds)
        let (download_url, archive_name) = if cfg!(target_os = "macos") {
            // Check if Apple Silicon or Intel
            let is_arm = cfg!(target_arch = "aarch64");
            if is_arm {
                (
                    "https://storage.googleapis.com/chrome-for-testing-public/131.0.6778.108/mac-arm64/chrome-mac-arm64.zip",
                    "chrome-mac-arm64.zip"
                )
            } else {
                (
                    "https://storage.googleapis.com/chrome-for-testing-public/131.0.6778.108/mac-x64/chrome-mac-x64.zip",
                    "chrome-mac-x64.zip"
                )
            }
        } else if cfg!(target_os = "linux") {
            (
                "https://storage.googleapis.com/chrome-for-testing-public/131.0.6778.108/linux64/chrome-linux64.zip",
                "chrome-linux64.zip"
            )
        } else {
            // Windows
            (
                "https://storage.googleapis.com/chrome-for-testing-public/131.0.6778.108/win64/chrome-win64.zip",
                "chrome-win64.zip"
            )
        };

        let cache_dir = dirs::home_dir()
            .ok_or_else(|| AuditError::ConfigError("Could not find home directory".to_string()))?
            .join(".auditmysite")
            .join("chromium");

        fs::create_dir_all(&cache_dir).map_err(|e| AuditError::BrowserLaunchFailed {
            reason: format!("Failed to create cache directory: {}", e),
        })?;

        let archive_path = cache_dir.join(archive_name);

        // Download with progress and retry
        info!("Downloading from: {}", download_url);
        info!("Destination: {}", archive_path.display());

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

            match reqwest::get(download_url).await {
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

        let mut file =
            fs::File::create(&archive_path).map_err(|e| AuditError::BrowserLaunchFailed {
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

            if total_size > 0 && downloaded % (total_size / 10).max(1) < chunk.len() as u64 {
                let percent = (downloaded * 100) / total_size;
                info!(
                    "Download progress: {}% ({}/{} MB)",
                    percent,
                    downloaded / 1_000_000,
                    total_size / 1_000_000
                );
            }
        }

        info!("Download complete");

        // Extract archive
        info!("Extracting archive...");
        Self::extract_archive(&archive_path, &cache_dir)?;

        // Clean up archive
        fs::remove_file(&archive_path).ok();

        let chromium_path = Self::local_chromium_path()?;

        if !chromium_path.exists() {
            return Err(AuditError::BrowserLaunchFailed {
                reason: format!(
                    "Chromium binary not found after extraction: {}",
                    chromium_path.display()
                ),
            });
        }

        // Make executable (Unix)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&chromium_path)
                .map_err(|e| AuditError::BrowserLaunchFailed {
                    reason: format!("Failed to get permissions: {}", e),
                })?
                .permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&chromium_path, perms).map_err(|e| {
                AuditError::BrowserLaunchFailed {
                    reason: format!("Failed to set permissions: {}", e),
                }
            })?;
        }

        info!(
            "Chromium installed successfully at: {}",
            chromium_path.display()
        );

        Ok(chromium_path)
    }

    /// Extract zip archive
    fn extract_archive(archive_path: &Path, dest_dir: &Path) -> Result<()> {
        let file = fs::File::open(archive_path).map_err(|e| AuditError::BrowserLaunchFailed {
            reason: format!("Failed to open archive: {}", e),
        })?;

        let mut archive =
            zip::ZipArchive::new(file).map_err(|e| AuditError::BrowserLaunchFailed {
                reason: format!("Failed to read zip: {}", e),
            })?;

        let total_files = archive.len();
        info!("Extracting {} files...", total_files);

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

            if i > 0 && i % 500 == 0 {
                info!("Extracting: {}/{}", i, total_files);
            }
        }

        info!("Extraction complete");
        Ok(())
    }
}
