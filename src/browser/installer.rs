//! Browser installer - explicit Chrome for Testing download
//!
//! Downloads to ~/.auditmysite/browsers/ — only on explicit request
//! (`auditmysite browser install`, `browser ensure --yes`) or after the user
//! consented to the prompt of an interactive audit (see `bootstrap`).
//!
//! The work is transactional and lives in `install_txn`; which archive is
//! trusted is decided in `install_plan`. This file is the stable entry point.

use std::fs;
use std::path::{Path, PathBuf};

use tracing::warn;

use super::install_plan::{Artifact, Platform};
use super::install_txn::{InstallError, InstallOutcome, InstallPreview, InstallRequest, Installer};
use super::types::InstallTarget;
use crate::error::{AuditError, Result};

/// Browser installer
pub struct BrowserInstaller;

impl BrowserInstaller {
    /// Install a browser to ~/.auditmysite/browsers/ and print the result.
    pub async fn install(
        target: InstallTarget,
        version: Option<&str>,
        force: bool,
    ) -> Result<PathBuf> {
        let request = InstallRequest {
            artifact: target.into(),
            version: version.map(str::to_string),
            force,
            offline: false,
        };
        let outcome = Self::run(&request).await?;
        match &outcome {
            InstallOutcome::AlreadyInstalled { version, .. } => {
                println!(
                    "Already installed: {} v{}",
                    request.artifact.dir_name(),
                    version
                );
                println!("Use --force to reinstall.");
            }
            InstallOutcome::Installed {
                binary, version, ..
            } => println!(
                "Installed {} v{} at {}",
                request.artifact.dir_name(),
                version,
                binary.display()
            ),
        }
        Ok(outcome.binary().to_path_buf())
    }

    /// Run one install transaction; prints nothing but download progress.
    pub async fn run(
        request: &InstallRequest,
    ) -> std::result::Result<InstallOutcome, InstallError> {
        Installer::official(Self::install_root()?, request.artifact)?
            .install(request)
            .await
    }

    /// What an install would download (version, source, size, destination).
    pub async fn preview(
        request: &InstallRequest,
    ) -> std::result::Result<InstallPreview, InstallError> {
        Installer::official(Self::install_root()?, request.artifact)?
            .preview(request)
            .await
    }

    /// Remove a managed browser installation
    pub fn remove(target: InstallTarget) -> Result<()> {
        let base_dir = Self::browsers_dir()?;
        let artifact = Artifact::from(target);
        let target_name = artifact.dir_name();
        let target_dir = base_dir.join(target_name);

        // Not while an install runs.
        let _lock = if base_dir.exists() {
            let file = fs::OpenOptions::new()
                .create(true)
                .truncate(false)
                .write(true)
                .open(base_dir.join(".install.lock"))
                .map_err(|e| AuditError::BrowserLaunchFailed {
                    reason: format!("Failed to open install lock: {}", e),
                })?;
            file.try_lock()
                .map_err(|_| AuditError::BrowserLaunchFailed {
                    reason: InstallError::LockHeld(base_dir.join(".install.lock")).to_string(),
                })?;
            Some(file)
        } else {
            None
        };

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
                match fs::remove_dir_all(&legacy_dir) {
                    Ok(()) => println!("Removed legacy chromium install"),
                    Err(e) => warn!("Failed to remove legacy chromium install: {}", e),
                }
            }
        }

        Ok(())
    }

    /// Binary of a managed install below `target_dir`. On platforms without
    /// a managed browser (Windows, macOS Intel) this is the path the old
    /// layout would have had, which never exists.
    pub(crate) fn binary_path(target_dir: &Path, target: InstallTarget) -> PathBuf {
        let artifact = Artifact::from(target);
        match Platform::detect() {
            Ok(platform) => target_dir.join(platform.binary_rel_path(artifact)),
            Err(_) => target_dir
                .join("unsupported-platform")
                .join(artifact.dir_name()),
        }
    }

    fn install_root() -> std::result::Result<PathBuf, InstallError> {
        Self::browsers_dir().map_err(|e| InstallError::Io {
            context: "home directory".to_string(),
            reason: e.to_string(),
        })
    }

    pub(crate) fn browsers_dir() -> std::result::Result<PathBuf, AuditError> {
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
}
