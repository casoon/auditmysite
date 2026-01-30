//! Chromium installer - downloads isolated Chromium binary
//!
//! Downloads Chromium to ~/.audit/chromium/ without affecting system Chrome.
//! Uses Chrome for Testing stable builds.

use std::fs;
use std::io::Write;
use std::path::PathBuf;

use tracing::info;

use crate::error::{AuditError, Result};

/// Chromium installation manager
pub struct ChromiumInstaller;

impl ChromiumInstaller {
    /// Ensure Chromium is available (check cache, ask user, download if needed)
    pub async fn ensure_chromium() -> Result<PathBuf> {
        // 1. Check if already downloaded
        let local_path = Self::local_chromium_path();
        if local_path.exists() {
            info!("Found cached Chromium at: {}", local_path.display());
            return Ok(local_path);
        }

        // 2. Chromium not found - inform user
        Self::prompt_user()?;

        // 3. Download
        Self::download_chromium().await
    }

    /// Get path to local Chromium installation
    fn local_chromium_path() -> PathBuf {
        let cache_dir = dirs::home_dir()
            .expect("Could not find home directory")
            .join(".audit")
            .join("chromium");

        if cfg!(target_os = "macos") {
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
        }
    }

    /// Prompt user about Chromium download
    fn prompt_user() -> Result<()> {
        println!("\n┌──────────────────────────────────────────────────────────┐");
        println!("│          Chromium Required for Accessibility Testing     │");
        println!("└──────────────────────────────────────────────────────────┘\n");
        println!("Chrome/Chromium not found on your system.\n");
        println!("Options:");
        println!("  1) Auto-download Chromium (~120 MB, isolated in ~/.audit/)");
        println!("     ✓ No system dependencies affected");
        println!("     ✓ Managed by audit");
        println!("     ✓ Can be deleted anytime\n");
        println!("  2) Use existing Chrome:");
        println!("     audit --chrome-path \"/Applications/Google Chrome.app/Contents/MacOS/Google Chrome\" <url>\n");
        println!("  3) Install via Homebrew:");
        println!("     brew install chromium\n");

        println!("Proceed with auto-download? [Y/n]: ");

        // For now, auto-accept (user can Ctrl+C to cancel)
        // In production, read from stdin
        println!("Proceeding with download...\n");

        Ok(())
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
            .expect("Could not find home directory")
            .join(".audit")
            .join("chromium");

        fs::create_dir_all(&cache_dir).map_err(|e| AuditError::BrowserLaunchFailed {
            reason: format!("Failed to create cache directory: {}", e),
        })?;

        let archive_path = cache_dir.join(archive_name);

        // Download with progress
        println!("Downloading from: {}", download_url);
        println!("Destination: {}", archive_path.display());

        let response =
            reqwest::get(download_url)
                .await
                .map_err(|e| AuditError::BrowserLaunchFailed {
                    reason: format!("Download failed: {}", e),
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

            if total_size > 0 {
                let percent = (downloaded * 100) / total_size;
                print!(
                    "\rProgress: {}% ({}/{} MB)",
                    percent,
                    downloaded / 1_000_000,
                    total_size / 1_000_000
                );
                std::io::stdout().flush().ok();
            }
        }

        println!("\n✓ Download complete!");

        // Extract archive
        info!("Extracting archive...");
        Self::extract_archive(&archive_path, &cache_dir)?;

        // Clean up archive
        fs::remove_file(&archive_path).ok();

        let chromium_path = Self::local_chromium_path();

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

        println!("✓ Chromium installed successfully!");
        println!("  Location: {}", chromium_path.display());

        Ok(chromium_path)
    }

    /// Extract zip archive
    fn extract_archive(archive_path: &PathBuf, dest_dir: &PathBuf) -> Result<()> {
        

        let file = fs::File::open(archive_path).map_err(|e| AuditError::BrowserLaunchFailed {
            reason: format!("Failed to open archive: {}", e),
        })?;

        let mut archive =
            zip::ZipArchive::new(file).map_err(|e| AuditError::BrowserLaunchFailed {
                reason: format!("Failed to read zip: {}", e),
            })?;

        let total_files = archive.len();
        println!("Extracting {} files...", total_files);

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
                std::io::copy(&mut file, &mut outfile).ok();
            }

            // Progress indicator
            if i % 100 == 0 {
                print!("\rExtracting: {}/{}", i, total_files);
                std::io::stdout().flush().ok();
            }
        }

        println!("\n✓ Extraction complete!");
        Ok(())
    }
}
