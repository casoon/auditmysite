//! Transactional install of a managed browser (#855).
//!
//! Everything happens in a staging directory below the browsers root; the
//! live installation is touched only by the final rename, and it is put back
//! if that rename fails:
//!
//! 1. exclusive lock on `<root>/.install.lock` (advisory, released by the OS
//!    when the process dies, so it can never be stale)
//! 2. recovery of what an earlier crashed run left behind
//! 3. manifest fetch (bounded) and selection of the one archive
//! 4. bounded download (size, total time, stall time, announced length)
//! 5. safe extraction (no path escape, no unsafe symlink, bounded entries and
//!    bytes)
//! 6. validation: binary present, version, CDP startup smoke test
//! 7. promotion: previous installation aside, staging in, previous deleted;
//!    on failure the previous one is renamed back
//!
//! There is a short window between "previous aside" and "staging in" in which
//! the managed directory does not exist; the lock excludes other installers,
//! not concurrent readers (an audit started in that window falls back to the
//! next browser or reports none).

use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use futures::future::BoxFuture;
use futures::StreamExt;
use tracing::{debug, info, warn};
use url::Url;

use super::install_plan::{
    select_download, validate_version, Artifact, InstallSource, Platform, ResolvedDownload,
};
use crate::error::AuditError;

const LOCK_FILE: &str = ".install.lock";
const STAGING_PREFIX: &str = ".staging-";
const PREVIOUS_PREFIX: &str = ".previous-";
const MAX_MANIFEST_BYTES: u64 = 32 * 1024 * 1024;

#[derive(Debug, thiserror::Error)]
pub enum InstallError {
    #[error("{0}")]
    UnsupportedPlatform(String),
    #[error("invalid version {0:?}: expected a Chrome for Testing version like 156.0.8078.80")]
    InvalidVersion(String),
    #[error("the publisher has no {platform} download for Chrome for Testing {version}: {reason}")]
    NotPublished {
        platform: String,
        version: String,
        reason: String,
    },
    #[error("setup needs network access and no previously validated managed browser exists (offline mode). Run `auditmysite browser install` with network access once")]
    OfflineNoInstall,
    #[error(
        "offline mode cannot fetch the requested version {requested}; installed is {installed}"
    )]
    OfflineVersionMismatch {
        requested: String,
        installed: String,
    },
    #[error("{what}: {reason}")]
    Network { what: String, reason: String },
    #[error("{what}: no response within the time limit")]
    Timeout { what: String },
    #[error("refusing untrusted URL {0}")]
    UntrustedUrl(String),
    #[error("publisher manifest unusable: {0}")]
    ManifestInvalid(String),
    #[error("download incomplete: received {received} of {expected} bytes")]
    DownloadIncomplete { received: u64, expected: u64 },
    #[error("download exceeds the limit of {limit} bytes")]
    TooLarge { limit: u64 },
    #[error("archive unusable: {0}")]
    ArchiveInvalid(String),
    #[error("archive rejected, unsafe entry: {0}")]
    UnsafeArchive(String),
    #[error("browser binary missing or not a regular file after extraction: {0}")]
    BinaryMissing(PathBuf),
    #[error("installed browser reports version {found:?}, expected {expected}")]
    VersionMismatch { expected: String, found: String },
    #[error("browser failed the startup check: {0}")]
    SmokeFailed(String),
    #[error("another install is running (lock {0}); try again when it has finished")]
    LockHeld(PathBuf),
    #[error("{context}: {reason}")]
    Io { context: String, reason: String },
    #[error("could not put the new installation in place ({reason}); previous installation {}", if *.restored { "restored" } else { "could NOT be restored" })]
    PromoteFailed { reason: String, restored: bool },
}

impl From<InstallError> for AuditError {
    fn from(e: InstallError) -> Self {
        AuditError::BrowserLaunchFailed {
            reason: e.to_string(),
        }
    }
}

fn io_err(context: impl Into<String>, e: impl std::fmt::Display) -> InstallError {
    InstallError::Io {
        context: context.into(),
        reason: e.to_string(),
    }
}

/// What a browser must prove before it is promoted.
pub trait BrowserProbe: Send + Sync {
    /// Output of `<binary> --version`.
    fn version<'a>(&'a self, binary: &'a Path) -> BoxFuture<'a, Result<String, String>>;
    /// Start the browser, connect over CDP, run one command. `scratch` is a
    /// writable directory for the throwaway profile.
    fn cdp_smoke<'a>(
        &'a self,
        binary: &'a Path,
        scratch: &'a Path,
    ) -> BoxFuture<'a, Result<(), String>>;
}

/// Starts the real binary.
pub struct SystemProbe {
    pub artifact: Artifact,
}

impl BrowserProbe for SystemProbe {
    fn version<'a>(&'a self, binary: &'a Path) -> BoxFuture<'a, Result<String, String>> {
        Box::pin(async move {
            let out = tokio::process::Command::new(binary)
                .arg("--version")
                .stdin(std::process::Stdio::null())
                .kill_on_drop(true)
                .output();
            let out = tokio::time::timeout(Duration::from_secs(20), out)
                .await
                .map_err(|_| "`--version` did not finish in 20 s".to_string())?
                .map_err(|e| format!("cannot run `--version`: {e}"))?;
            if !out.status.success() {
                return Err(format!("`--version` exited with {}", out.status));
            }
            Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
        })
    }

    fn cdp_smoke<'a>(
        &'a self,
        binary: &'a Path,
        scratch: &'a Path,
    ) -> BoxFuture<'a, Result<(), String>> {
        Box::pin(async move {
            use tokio::io::{AsyncBufReadExt, BufReader};

            let profile = scratch.join("smoke-profile");
            fs::create_dir_all(&profile).map_err(|e| format!("cannot create profile: {e}"))?;
            let mut cmd = tokio::process::Command::new(binary);
            if self.artifact == Artifact::Chrome {
                cmd.arg("--headless");
            }
            // The sandbox is off for this throwaway about:blank page only: the
            // check proves that the browser starts and speaks CDP, and must not
            // fail on hosts whose user namespaces forbid the sandbox. Audits
            // are unaffected (`--no-sandbox` stays opt-in there).
            cmd.args([
                "--remote-debugging-port=0",
                "--no-sandbox",
                "--disable-gpu",
                "--no-first-run",
                "--no-default-browser-check",
            ])
            .arg(format!("--user-data-dir={}", profile.display()))
            .arg("about:blank")
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true);
            let mut child = cmd.spawn().map_err(|e| format!("cannot start: {e}"))?;
            let stderr = child.stderr.take().ok_or("no stderr")?;
            let mut lines = BufReader::new(stderr).lines();

            let find_ws = async {
                while let Some(line) = lines
                    .next_line()
                    .await
                    .map_err(|e| format!("cannot read startup output: {e}"))?
                {
                    if let Some(rest) = line.split("DevTools listening on ").nth(1) {
                        return Ok(rest.trim().to_string());
                    }
                }
                Err("exited before announcing a DevTools endpoint".to_string())
            };
            let result = async {
                let ws = tokio::time::timeout(Duration::from_secs(30), find_ws)
                    .await
                    .map_err(|_| "no DevTools endpoint within 30 s".to_string())??;
                let (browser, mut handler) = tokio::time::timeout(
                    Duration::from_secs(15),
                    chromiumoxide::Browser::connect(ws),
                )
                .await
                .map_err(|_| "CDP connection timed out".to_string())?
                .map_err(|e| format!("CDP connection failed: {e}"))?;
                let pump = tokio::spawn(async move { while handler.next().await.is_some() {} });
                let version = tokio::time::timeout(Duration::from_secs(10), browser.version())
                    .await
                    .map_err(|_| "CDP command timed out".to_string())?
                    .map_err(|e| format!("CDP command failed: {e}"));
                pump.abort();
                version.map(|_| ())
            }
            .await;
            let _ = child.kill().await;
            result.map_err(|e| {
                if cfg!(target_os = "macos") {
                    format!(
                        "{e} (macOS: if Gatekeeper blocked the app, nothing was bypassed; \
                         install a browser you trust and pass --browser-path)"
                    )
                } else {
                    e
                }
            })
        })
    }
}

/// Download and extraction bounds.
#[derive(Debug, Clone)]
pub struct Limits {
    pub max_download_bytes: u64,
    pub connect_timeout: Duration,
    pub total_timeout: Duration,
    pub stall_timeout: Duration,
    pub retries: u32,
    pub retry_backoff: Duration,
    pub max_entries: usize,
    pub max_extracted_bytes: u64,
}

impl Limits {
    pub fn official() -> Self {
        Self {
            max_download_bytes: 600 * 1024 * 1024,
            connect_timeout: Duration::from_secs(15),
            total_timeout: Duration::from_secs(20 * 60),
            stall_timeout: Duration::from_secs(30),
            retries: 2,
            retry_backoff: Duration::from_secs(2),
            max_entries: 20_000,
            max_extracted_bytes: 2 * 1024 * 1024 * 1024,
        }
    }
}

#[derive(Debug, Clone)]
pub struct InstallRequest {
    pub artifact: Artifact,
    pub version: Option<String>,
    pub force: bool,
    /// Never touch the network: use a previously validated managed browser
    /// or fail.
    pub offline: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallOutcome {
    AlreadyInstalled {
        binary: PathBuf,
        version: String,
    },
    Installed {
        binary: PathBuf,
        version: String,
        replaced: Option<String>,
    },
}

impl InstallOutcome {
    pub fn binary(&self) -> &Path {
        match self {
            InstallOutcome::AlreadyInstalled { binary, .. }
            | InstallOutcome::Installed { binary, .. } => binary,
        }
    }
    pub fn version(&self) -> &str {
        match self {
            InstallOutcome::AlreadyInstalled { version, .. }
            | InstallOutcome::Installed { version, .. } => version,
        }
    }
}

/// What an install would fetch, for the consent prompt.
#[derive(Debug, Clone)]
pub struct InstallPreview {
    pub version: String,
    pub url: Url,
    pub size: Option<u64>,
    pub destination: PathBuf,
}

pub struct Installer {
    pub root: PathBuf,
    pub source: InstallSource,
    pub limits: Limits,
    pub platform: Platform,
    pub probe: Arc<dyn BrowserProbe>,
    pub lock_wait: Duration,
    pub show_progress: bool,
}

struct InstalledInfo {
    version: String,
    binary: PathBuf,
}

impl Installer {
    /// Production installer for this machine below `root`.
    pub fn official(root: PathBuf, artifact: Artifact) -> Result<Self, InstallError> {
        Ok(Self {
            root,
            source: InstallSource::official(),
            limits: Limits::official(),
            platform: Platform::detect()?,
            probe: Arc::new(SystemProbe { artifact }),
            lock_wait: Duration::from_secs(60),
            show_progress: true,
        })
    }

    pub fn target_dir(&self, artifact: Artifact) -> PathBuf {
        self.root.join(artifact.dir_name())
    }

    /// The installed browser, if it has a version marker and its binary.
    fn existing(&self, artifact: Artifact) -> Option<InstalledInfo> {
        let dir = self.target_dir(artifact);
        let version = fs::read_to_string(dir.join("version.txt")).ok()?;
        let version = version.trim().to_string();
        let binary = dir.join(self.platform.binary_rel_path(artifact));
        let is_file = fs::symlink_metadata(&binary)
            .map(|m| m.is_file())
            .unwrap_or(false);
        (!version.is_empty() && is_file).then_some(InstalledInfo { version, binary })
    }

    pub async fn install(&self, req: &InstallRequest) -> Result<InstallOutcome, InstallError> {
        if let Some(v) = &req.version {
            validate_version(v)?;
        }
        fs::create_dir_all(&self.root).map_err(|e| io_err("cannot create browsers dir", e))?;
        let _lock = self.acquire_lock().await?;
        self.recover()?;

        let existing = self.existing(req.artifact);
        if let Some(ex) = &existing {
            let same_version = req.version.as_deref().is_none_or(|v| v == ex.version);
            if (!req.force && same_version) || (req.offline && (req.force || same_version)) {
                return Ok(InstallOutcome::AlreadyInstalled {
                    binary: ex.binary.clone(),
                    version: ex.version.clone(),
                });
            }
            if req.offline {
                return Err(InstallError::OfflineVersionMismatch {
                    requested: req.version.clone().unwrap_or_default(),
                    installed: ex.version.clone(),
                });
            }
        } else if req.offline {
            return Err(InstallError::OfflineNoInstall);
        }

        let download = self.resolve(req).await?;
        let staging = self.root.join(format!(
            "{STAGING_PREFIX}{}-{}",
            req.artifact.dir_name(),
            unique_suffix()
        ));
        fs::create_dir_all(&staging).map_err(|e| io_err("cannot create staging dir", e))?;

        let built = self.build(&download, &staging).await;
        let payload = match built {
            Ok(p) => p,
            Err(e) => {
                let _ = fs::remove_dir_all(&staging);
                return Err(e);
            }
        };
        let target = self.target_dir(req.artifact);
        let promoted = promote(&self.root, &payload, &target, req.artifact.dir_name());
        let _ = fs::remove_dir_all(&staging);
        promoted?;

        let binary = target.join(self.platform.binary_rel_path(req.artifact));
        Ok(InstallOutcome::Installed {
            binary,
            version: download.version,
            replaced: existing.map(|e| e.version),
        })
    }

    /// Manifest lookup and archive size, for the consent prompt. No lock, no
    /// writes.
    pub async fn preview(&self, req: &InstallRequest) -> Result<InstallPreview, InstallError> {
        if let Some(v) = &req.version {
            validate_version(v)?;
        }
        let download = self.resolve(req).await?;
        let client = self.client()?;
        let size = client
            .head(download.url.clone())
            .timeout(self.limits.total_timeout.min(Duration::from_secs(30)))
            .send()
            .await
            .ok()
            .filter(|r| r.status().is_success())
            .and_then(|r| {
                r.headers()
                    .get(reqwest::header::CONTENT_LENGTH)?
                    .to_str()
                    .ok()?
                    .parse::<u64>()
                    .ok()
            });
        Ok(InstallPreview {
            version: download.version,
            url: download.url,
            size,
            destination: self.target_dir(req.artifact),
        })
    }

    async fn acquire_lock(&self) -> Result<fs::File, InstallError> {
        let path = self.root.join(LOCK_FILE);
        let file = fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(&path)
            .map_err(|e| io_err("cannot open install lock", e))?;
        let start = Instant::now();
        loop {
            match file.try_lock() {
                Ok(()) => return Ok(file),
                Err(fs::TryLockError::WouldBlock) => {
                    if start.elapsed() >= self.lock_wait {
                        return Err(InstallError::LockHeld(path));
                    }
                    tokio::time::sleep(Duration::from_millis(200)).await;
                }
                Err(fs::TryLockError::Error(e)) => return Err(io_err("cannot lock", e)),
            }
        }
    }

    /// Under the lock nothing else is installing: every staging directory is
    /// stale, and a `.previous-*` without its live directory is an install
    /// that died between the two renames of the promotion.
    fn recover(&self) -> Result<(), InstallError> {
        let entries =
            fs::read_dir(&self.root).map_err(|e| io_err("cannot read browsers dir", e))?;
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            let path = entry.path();
            if name.starts_with(STAGING_PREFIX) {
                debug!("removing stale staging {}", path.display());
                let _ = fs::remove_dir_all(&path);
            } else if let Some(rest) = name.strip_prefix(PREVIOUS_PREFIX) {
                let live = [Artifact::Chrome, Artifact::HeadlessShell]
                    .iter()
                    .map(|a| a.dir_name())
                    .find(|d| rest.starts_with(&format!("{d}-")));
                match live {
                    Some(d) if !self.root.join(d).exists() => {
                        warn!("restoring interrupted installation {d}");
                        fs::rename(&path, self.root.join(d))
                            .map_err(|e| io_err("cannot restore previous installation", e))?;
                    }
                    _ => {
                        let _ = fs::remove_dir_all(&path);
                    }
                }
            }
        }
        Ok(())
    }

    fn client(&self) -> Result<reqwest::Client, InstallError> {
        let policy = self.source.policy.clone();
        reqwest::Client::builder()
            .connect_timeout(self.limits.connect_timeout)
            .user_agent(concat!("auditmysite-installer/", env!("CARGO_PKG_VERSION")))
            .redirect(reqwest::redirect::Policy::custom(move |attempt| {
                if attempt.previous().len() >= 5 {
                    attempt.error("too many redirects")
                } else if policy.redirect_allowed(attempt.url()) {
                    attempt.follow()
                } else {
                    attempt.error("redirect to an untrusted location")
                }
            }))
            .build()
            .map_err(|e| io_err("cannot build HTTP client", e))
    }

    async fn resolve(&self, req: &InstallRequest) -> Result<ResolvedDownload, InstallError> {
        let manifest_url = match &req.version {
            None => &self.source.latest_manifest,
            Some(_) => &self.source.known_good_manifest,
        };
        let url = Url::parse(manifest_url)
            .map_err(|_| InstallError::UntrustedUrl(manifest_url.to_string()))?;
        self.source.policy.check_manifest_url(&url)?;
        let client = self.client()?;
        let mut body = Vec::new();
        self.fetch_into(
            &client,
            &url,
            "publisher manifest",
            MAX_MANIFEST_BYTES,
            &mut body,
            false,
        )
        .await?;
        let manifest: serde_json::Value = serde_json::from_slice(&body)
            .map_err(|e| InstallError::ManifestInvalid(e.to_string()))?;
        select_download(
            &manifest,
            req.version.as_deref(),
            self.platform,
            req.artifact,
            &self.source.policy,
        )
    }

    /// GET with retries on connection errors, timeouts and 5xx; the body is
    /// streamed into `out` under size, stall and announced-length checks.
    async fn fetch_into(
        &self,
        client: &reqwest::Client,
        url: &Url,
        what: &str,
        max_bytes: u64,
        out: &mut (dyn Write + Send),
        progress: bool,
    ) -> Result<u64, InstallError> {
        let mut last = None;
        for attempt in 0..=self.limits.retries {
            if attempt > 0 {
                let delay = self.limits.retry_backoff * 2u32.pow(attempt - 1);
                warn!(
                    "retrying {what} (attempt {}), backoff {delay:?}",
                    attempt + 1
                );
                tokio::time::sleep(delay).await;
            }
            match client
                .get(url.clone())
                .timeout(self.limits.total_timeout)
                .send()
                .await
            {
                Ok(resp) if resp.status().is_success() => {
                    return self.stream_body(resp, what, max_bytes, out, progress).await;
                }
                Ok(resp) if resp.status().is_server_error() => {
                    last = Some(InstallError::Network {
                        what: what.to_string(),
                        reason: format!("HTTP {}", resp.status()),
                    });
                }
                Ok(resp) => {
                    return Err(InstallError::Network {
                        what: what.to_string(),
                        reason: format!("HTTP {}", resp.status()),
                    });
                }
                Err(e) if e.is_timeout() => {
                    last = Some(InstallError::Timeout {
                        what: what.to_string(),
                    });
                }
                Err(e) if e.is_redirect() => {
                    return Err(InstallError::UntrustedUrl(format!("{url} ({e})")));
                }
                Err(e) => {
                    last = Some(InstallError::Network {
                        what: what.to_string(),
                        reason: e.to_string(),
                    });
                }
            }
        }
        Err(last.expect("loop ran at least once"))
    }

    async fn stream_body(
        &self,
        resp: reqwest::Response,
        what: &str,
        max_bytes: u64,
        out: &mut (dyn Write + Send),
        progress: bool,
    ) -> Result<u64, InstallError> {
        let expected = resp.content_length();
        if expected.is_some_and(|n| n > max_bytes) {
            return Err(InstallError::TooLarge { limit: max_bytes });
        }
        let mut stream = resp.bytes_stream();
        let mut received = 0u64;
        let mut next_report = 10u64;
        loop {
            let chunk = match tokio::time::timeout(self.limits.stall_timeout, stream.next()).await {
                Err(_) => {
                    return Err(InstallError::Timeout {
                        what: what.to_string(),
                    })
                }
                Ok(None) => break,
                Ok(Some(Err(e))) => {
                    return Err(match expected {
                        Some(total) => InstallError::DownloadIncomplete {
                            received,
                            expected: total,
                        },
                        None => InstallError::Network {
                            what: what.to_string(),
                            reason: e.to_string(),
                        },
                    })
                }
                Ok(Some(Ok(c))) => c,
            };
            received += chunk.len() as u64;
            if received > max_bytes {
                return Err(InstallError::TooLarge { limit: max_bytes });
            }
            out.write_all(&chunk)
                .map_err(|e| io_err("cannot write download", e))?;
            if let (true, true, Some(total)) = (progress, self.show_progress, expected) {
                let pct = received * 100 / total.max(1);
                if pct >= next_report {
                    eprintln!(
                        "  {pct}% ({} of {} MB)",
                        received / 1_000_000,
                        total / 1_000_000
                    );
                    next_report = (pct / 10 + 1) * 10;
                }
            }
        }
        if let Some(total) = expected {
            if received != total {
                return Err(InstallError::DownloadIncomplete {
                    received,
                    expected: total,
                });
            }
        }
        Ok(received)
    }

    /// Download, extract and validate inside `staging`; returns the payload
    /// directory that becomes the live installation.
    async fn build(
        &self,
        download: &ResolvedDownload,
        staging: &Path,
    ) -> Result<PathBuf, InstallError> {
        let part = staging.join("download.zip.part");
        let archive = staging.join("download.zip");
        let client = self.client()?;
        let bytes = {
            let mut file =
                fs::File::create(&part).map_err(|e| io_err("cannot create download file", e))?;
            let n = self
                .fetch_into(
                    &client,
                    &download.url,
                    "browser download",
                    self.limits.max_download_bytes,
                    &mut file,
                    true,
                )
                .await?;
            file.flush()
                .map_err(|e| io_err("cannot flush download", e))?;
            n
        };
        fs::rename(&part, &archive).map_err(|e| io_err("cannot finalize download", e))?;

        let payload = staging.join("payload");
        let (a, p, l) = (archive.clone(), payload.clone(), self.limits.clone());
        tokio::task::spawn_blocking(move || extract_zip_safely(&a, &p, &l))
            .await
            .map_err(|e| io_err("extraction task failed", e))??;
        let _ = fs::remove_file(&archive);

        let binary = payload.join(download.platform.binary_rel_path(download.artifact));
        let meta = fs::symlink_metadata(&binary)
            .map_err(|_| InstallError::BinaryMissing(binary.clone()))?;
        if !meta.is_file() {
            return Err(InstallError::BinaryMissing(binary));
        }
        set_executable(&binary)?;

        let found = self
            .probe
            .version(&binary)
            .await
            .map_err(InstallError::SmokeFailed)?;
        if !found.contains(&download.version) {
            return Err(InstallError::VersionMismatch {
                expected: download.version.clone(),
                found,
            });
        }
        self.probe
            .cdp_smoke(&binary, staging)
            .await
            .map_err(InstallError::SmokeFailed)?;

        fs::write(payload.join("version.txt"), &download.version)
            .map_err(|e| io_err("cannot write version.txt", e))?;
        let record = serde_json::json!({
            "schema": 1,
            "version": download.version,
            "platform": download.platform.manifest_id(),
            "source_url": download.url.as_str(),
            "archive_bytes": bytes,
            "installed_at_unix": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0),
            "verification": {
                // Upstream publishes no digest or signature (see install_plan).
                "publisher_checksum": "not published",
                "publisher_signature": "not published",
                "transport": "https, host allow-list, manifest entry for exact platform",
                "announced_size_matched": true,
                "binary_version": found,
                "cdp_smoke": "passed",
            },
        });
        fs::write(
            payload.join("install.json"),
            serde_json::to_vec_pretty(&record).expect("json"),
        )
        .map_err(|e| io_err("cannot write install.json", e))?;
        info!(
            "validated {} {}",
            download.artifact.dir_name(),
            download.version
        );
        Ok(payload)
    }
}

fn unique_suffix() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{}-{nanos}", std::process::id())
}

/// Swap `payload` in for `target`. The previous installation is renamed aside
/// first and renamed back if the second rename fails.
pub(crate) fn promote(
    root: &Path,
    payload: &Path,
    target: &Path,
    name: &str,
) -> Result<(), InstallError> {
    let previous = if target.exists() {
        let aside = root.join(format!("{PREVIOUS_PREFIX}{name}-{}", unique_suffix()));
        fs::rename(target, &aside).map_err(|e| InstallError::PromoteFailed {
            reason: format!("cannot move previous installation aside: {e}"),
            restored: true,
        })?;
        Some(aside)
    } else {
        None
    };
    match fs::rename(payload, target) {
        Ok(()) => {
            if let Some(aside) = previous {
                if let Err(e) = fs::remove_dir_all(&aside) {
                    warn!(
                        "cannot delete previous installation {}: {e}",
                        aside.display()
                    );
                }
            }
            Ok(())
        }
        Err(e) => {
            let restored = match previous {
                Some(aside) => fs::rename(&aside, target).is_ok(),
                None => true,
            };
            Err(InstallError::PromoteFailed {
                reason: e.to_string(),
                restored,
            })
        }
    }
}

fn set_executable(path: &Path) -> Result<(), InstallError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(path)
            .map_err(|e| io_err("cannot read binary permissions", e))?
            .permissions();
        perms.set_mode(perms.mode() | 0o755);
        fs::set_permissions(path, perms).map_err(|e| io_err("cannot make binary executable", e))?;
    }
    #[cfg(not(unix))]
    let _ = path;
    Ok(())
}

/// Extract `archive` into `dest` (created). Any entry that could write
/// outside `dest`, any link that leaves it, a duplicate path, or a bound
/// exceeded fails the whole extraction; nothing is skipped silently.
pub(crate) fn extract_zip_safely(
    archive: &Path,
    dest: &Path,
    limits: &Limits,
) -> Result<(), InstallError> {
    let file = fs::File::open(archive).map_err(|e| io_err("cannot open archive", e))?;
    let mut zip =
        zip::ZipArchive::new(file).map_err(|e| InstallError::ArchiveInvalid(e.to_string()))?;
    if zip.len() > limits.max_entries {
        return Err(InstallError::ArchiveInvalid(format!(
            "{} entries exceed the limit of {}",
            zip.len(),
            limits.max_entries
        )));
    }
    fs::create_dir_all(dest).map_err(|e| io_err("cannot create extraction dir", e))?;
    let root = dest
        .canonicalize()
        .map_err(|e| io_err("cannot resolve extraction dir", e))?;
    let mut total = 0u64;

    for i in 0..zip.len() {
        let mut entry = zip
            .by_index(i)
            .map_err(|e| InstallError::ArchiveInvalid(e.to_string()))?;
        let raw_name = String::from_utf8_lossy(entry.name_raw()).to_string();
        let rel = entry
            .enclosed_name()
            .ok_or_else(|| InstallError::UnsafeArchive(raw_name.clone()))?;
        let out = root.join(&rel);
        let mode = entry.unix_mode();
        let is_symlink = mode.is_some_and(|m| m & 0o170000 == 0o120000);

        if entry.is_dir() {
            fs::create_dir_all(&out).map_err(|e| io_err("cannot create directory", e))?;
            ensure_inside(&root, &out, &raw_name)?;
            continue;
        }
        let parent = out
            .parent()
            .ok_or_else(|| InstallError::UnsafeArchive(raw_name.clone()))?;
        fs::create_dir_all(parent).map_err(|e| io_err("cannot create directory", e))?;
        ensure_inside(&root, parent, &raw_name)?;

        if is_symlink {
            let mut target = String::new();
            entry
                .by_ref()
                .take(4097)
                .read_to_string(&mut target)
                .map_err(|e| {
                    InstallError::ArchiveInvalid(format!("bad symlink {raw_name}: {e}"))
                })?;
            check_symlink_target(&rel, &target, &raw_name)?;
            create_symlink(&target, &out, &raw_name)?;
            continue;
        }

        let remaining = limits.max_extracted_bytes.saturating_sub(total);
        let mut dst = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&out)
            .map_err(|e| {
                InstallError::ArchiveInvalid(format!("cannot create {raw_name} (duplicate?): {e}"))
            })?;
        let written = std::io::copy(&mut entry.by_ref().take(remaining + 1), &mut dst)
            .map_err(|e| InstallError::ArchiveInvalid(format!("cannot extract {raw_name}: {e}")))?;
        if written > remaining {
            return Err(InstallError::ArchiveInvalid(format!(
                "extracted size exceeds the limit of {} bytes",
                limits.max_extracted_bytes
            )));
        }
        total += written;
        drop(dst);
        #[cfg(unix)]
        if let Some(m) = mode {
            use std::os::unix::fs::PermissionsExt;
            // Keep rwx bits, never setuid/setgid/sticky.
            fs::set_permissions(&out, fs::Permissions::from_mode(m & 0o777))
                .map_err(|e| io_err("cannot set permissions", e))?;
        }
    }
    Ok(())
}

/// `path` (existing) must resolve to somewhere below `root`; catches writes
/// that would pass through a symlink created earlier in the same archive.
fn ensure_inside(root: &Path, path: &Path, name: &str) -> Result<(), InstallError> {
    let real = path
        .canonicalize()
        .map_err(|e| io_err("cannot resolve extracted path", e))?;
    if real.starts_with(root) {
        Ok(())
    } else {
        Err(InstallError::UnsafeArchive(name.to_string()))
    }
}

/// A link target must be relative and stay below the extraction root,
/// resolved from the link's own directory.
fn check_symlink_target(rel: &Path, target: &str, name: &str) -> Result<(), InstallError> {
    use std::path::Component;
    let bad = || InstallError::UnsafeArchive(format!("{name} -> {target}"));
    if target.is_empty() || target.len() > 4096 {
        return Err(bad());
    }
    let mut depth = rel.components().count().saturating_sub(1) as i64;
    for c in Path::new(target).components() {
        match c {
            Component::Normal(_) => depth += 1,
            Component::CurDir => {}
            Component::ParentDir => {
                depth -= 1;
                if depth < 0 {
                    return Err(bad());
                }
            }
            Component::RootDir | Component::Prefix(_) => return Err(bad()),
        }
    }
    Ok(())
}

fn create_symlink(target: &str, link: &Path, name: &str) -> Result<(), InstallError> {
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(target, link)
            .map_err(|e| InstallError::ArchiveInvalid(format!("cannot create link {name}: {e}")))
    }
    #[cfg(not(unix))]
    {
        let _ = (target, link);
        Err(InstallError::UnsafeArchive(format!(
            "{name}: symbolic links are not supported on this platform"
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::browser::install_testkit::*;
    use std::sync::atomic::Ordering;

    const V: &str = "156.0.8078.80";

    fn req() -> InstallRequest {
        InstallRequest {
            artifact: Artifact::Chrome,
            version: None,
            force: false,
            offline: false,
        }
    }

    fn live_binary(root: &Path) -> PathBuf {
        root.join("chrome-for-testing/chrome-linux64/chrome")
    }

    fn names(root: &Path) -> Vec<String> {
        let mut n: Vec<String> = fs::read_dir(root)
            .unwrap()
            .flatten()
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect();
        n.sort();
        n
    }

    async fn installed(root: &Path, server: &TestServer) -> Installer {
        server.publish_chrome(V, Platform::LinuxX64, good_linux_zip());
        let inst = test_installer(root, server, FakeProbe::new(V));
        inst.install(&req()).await.unwrap();
        inst
    }

    #[tokio::test]
    async fn fresh_install_is_validated_and_leaves_no_temp_files() {
        let (dir, server) = (tempfile::tempdir().unwrap(), TestServer::start().await);
        server.publish_chrome(V, Platform::LinuxX64, good_linux_zip());
        let probe = FakeProbe::new(V);
        let inst = test_installer(dir.path(), &server, probe.clone());

        let out = inst.install(&req()).await.unwrap();

        assert!(matches!(
            out,
            InstallOutcome::Installed { replaced: None, .. }
        ));
        assert_eq!(out.version(), V);
        assert_eq!(out.binary(), live_binary(dir.path()));
        assert_eq!(probe.smoke_calls.load(Ordering::SeqCst), 1);
        let record =
            fs::read_to_string(dir.path().join("chrome-for-testing/install.json")).unwrap();
        assert!(
            record.contains("\"publisher_signature\": \"not published\""),
            "{record}"
        );
        assert_eq!(names(dir.path()), [".install.lock", "chrome-for-testing"]);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(live_binary(dir.path()))
                .unwrap()
                .permissions()
                .mode();
            assert_eq!(mode & 0o111, 0o111, "binary must be executable");
        }
    }

    #[tokio::test]
    async fn repeated_install_is_idempotent_and_force_replaces() {
        let (dir, server) = (tempfile::tempdir().unwrap(), TestServer::start().await);
        let inst = installed(dir.path(), &server).await;
        let path = format!("/{V}/linux64/chrome-linux64.zip");
        assert_eq!(server.hits(&path), 1);

        let again = inst.install(&req()).await.unwrap();
        assert!(matches!(again, InstallOutcome::AlreadyInstalled { .. }));
        assert_eq!(server.hits(&path), 1, "no second download");
        assert_eq!(server.hits("/latest.json"), 1, "no manifest fetch either");

        let forced = inst
            .install(&InstallRequest {
                force: true,
                ..req()
            })
            .await
            .unwrap();
        assert!(matches!(
            forced,
            InstallOutcome::Installed {
                replaced: Some(_),
                ..
            }
        ));
        assert_eq!(server.hits(&path), 2);
        assert_eq!(names(dir.path()), [".install.lock", "chrome-for-testing"]);
    }

    #[tokio::test]
    async fn interrupted_download_fails_and_keeps_previous_installation() {
        let (dir, server) = (tempfile::tempdir().unwrap(), TestServer::start().await);
        let inst = installed(dir.path(), &server).await;
        let path = format!("/{V}/linux64/chrome-linux64.zip");
        server.route(
            &path,
            Route::Truncated {
                announce: 5_000,
                body: vec![0; 100],
            },
        );

        let err = inst
            .install(&InstallRequest {
                force: true,
                ..req()
            })
            .await
            .unwrap_err();

        assert!(
            matches!(err, InstallError::DownloadIncomplete { .. }),
            "{err}"
        );
        assert!(
            live_binary(dir.path()).is_file(),
            "previous install untouched"
        );
        assert_eq!(names(dir.path()), [".install.lock", "chrome-for-testing"]);
    }

    #[tokio::test]
    async fn unresponsive_endpoint_times_out() {
        let (dir, server) = (tempfile::tempdir().unwrap(), TestServer::start().await);
        server.route("/latest.json", Route::Hang);
        let mut inst = test_installer(dir.path(), &server, FakeProbe::new(V));
        inst.limits.total_timeout = Duration::from_millis(600);

        let err = inst.install(&req()).await.unwrap_err();

        assert!(matches!(err, InstallError::Timeout { .. }), "{err}");
        assert!(!dir.path().join("chrome-for-testing").exists());
    }

    #[tokio::test]
    async fn http_errors_and_oversize_downloads_are_refused() {
        let (dir, server) = (tempfile::tempdir().unwrap(), TestServer::start().await);
        let inst = test_installer(dir.path(), &server, FakeProbe::new(V));
        // No manifest served: 404.
        assert!(matches!(
            inst.install(&req()).await.unwrap_err(),
            InstallError::Network { .. }
        ));
        // Announced size above the bound: refused before any byte is stored.
        let path = server.publish_chrome(V, Platform::LinuxX64, good_linux_zip());
        server.route(
            &path,
            Route::Truncated {
                announce: 50 * 1024 * 1024,
                body: vec![],
            },
        );
        assert!(matches!(
            inst.install(&req()).await.unwrap_err(),
            InstallError::TooLarge { .. }
        ));
    }

    #[tokio::test]
    async fn wrong_platform_is_not_published_and_never_substituted() {
        let (dir, server) = (tempfile::tempdir().unwrap(), TestServer::start().await);
        server.publish_chrome(V, Platform::LinuxX64, good_linux_zip());
        let mut inst = test_installer(dir.path(), &server, FakeProbe::new(V));
        inst.platform = Platform::LinuxArm64;

        let err = inst.install(&req()).await.unwrap_err();

        assert!(matches!(err, InstallError::NotPublished { .. }), "{err}");
        assert_eq!(server.total_hits(), 1, "only the manifest was fetched");
    }

    #[tokio::test]
    async fn redirect_to_untrusted_host_is_refused() {
        let (dir, server) = (tempfile::tempdir().unwrap(), TestServer::start().await);
        let path = server.publish_chrome(V, Platform::LinuxX64, good_linux_zip());
        server.route(
            &path,
            Route::Redirect("http://evil.example/chrome.zip".to_string()),
        );
        let inst = test_installer(dir.path(), &server, FakeProbe::new(V));

        let err = inst.install(&req()).await.unwrap_err();

        assert!(matches!(err, InstallError::UntrustedUrl(_)), "{err}");
    }

    #[tokio::test]
    async fn broken_and_hostile_archives_are_rejected() {
        let (dir, server) = (tempfile::tempdir().unwrap(), TestServer::start().await);
        let inst = test_installer(dir.path(), &server, FakeProbe::new(V));
        let path = server.publish_chrome(V, Platform::LinuxX64, good_linux_zip());

        server.route(&path, Route::Ok(b"this is not a zip".to_vec()));
        assert!(matches!(
            inst.install(&req()).await.unwrap_err(),
            InstallError::ArchiveInvalid(_)
        ));

        let traversal = make_zip(&[
            ("chrome-linux64/chrome", b"x", Some(0o755)),
            ("../escape", b"x", None),
        ]);
        server.route(&path, Route::Ok(traversal));
        let err = inst.install(&req()).await.unwrap_err();
        assert!(matches!(err, InstallError::UnsafeArchive(_)), "{err}");
        assert!(!dir.path().join("escape").exists());
        assert!(!dir.path().parent().unwrap().join("escape").exists());
        assert!(
            !dir.path().join("chrome-for-testing").exists(),
            "nothing promoted"
        );
        assert_eq!(names(dir.path()), [".install.lock"], "staging cleaned");
    }

    #[cfg(unix)]
    #[test]
    fn symlinks_must_stay_inside_the_archive_tree() {
        let tmp = tempfile::tempdir().unwrap();
        let limits = test_limits();
        for (i, (target, ok)) in [
            ("/etc/passwd", false),
            ("../../../outside", false),
            ("../../../../x", false),
            ("../../inside", true),
            ("Versions/Current/Resources", true),
            ("../Resources", true),
        ]
        .into_iter()
        .enumerate()
        {
            let archive = tmp.path().join("a.zip");
            let mut w = zip::ZipWriter::new(fs::File::create(&archive).unwrap());
            let o = zip::write::SimpleFileOptions::default();
            w.start_file("top/app/file", o).unwrap();
            w.write_all(b"x").unwrap();
            w.add_symlink("top/app/link", target, o).unwrap();
            w.finish().unwrap();
            let dest = tmp.path().join(format!("out-{i}"));
            let res = extract_zip_safely(&archive, &dest, &limits);
            assert_eq!(res.is_ok(), ok, "{target}: {res:?}");
            if ok {
                assert!(fs::symlink_metadata(dest.join("top/app/link"))
                    .unwrap()
                    .is_symlink());
            }
        }
    }

    #[test]
    fn extraction_is_bounded_and_never_overwrites() {
        let tmp = tempfile::tempdir().unwrap();
        let archive = tmp.path().join("a.zip");
        fs::write(
            &archive,
            make_zip(&[("a/x", &[0u8; 2048], None), ("a/y", b"y", None)]),
        )
        .unwrap();
        let mut limits = test_limits();
        limits.max_extracted_bytes = 1024;
        assert!(matches!(
            extract_zip_safely(&archive, &tmp.path().join("o1"), &limits),
            Err(InstallError::ArchiveInvalid(_))
        ));
        let mut limits = test_limits();
        limits.max_entries = 1;
        assert!(matches!(
            extract_zip_safely(&archive, &tmp.path().join("o2"), &limits),
            Err(InstallError::ArchiveInvalid(_))
        ));
        // Extracting onto existing files must fail instead of overwriting them.
        let dest = tmp.path().join("o3");
        extract_zip_safely(&archive, &dest, &test_limits()).unwrap();
        assert!(matches!(
            extract_zip_safely(&archive, &dest, &test_limits()),
            Err(InstallError::ArchiveInvalid(_))
        ));
    }

    #[tokio::test]
    async fn missing_binary_fails_validation() {
        let (dir, server) = (tempfile::tempdir().unwrap(), TestServer::start().await);
        server.publish_chrome(
            V,
            Platform::LinuxX64,
            make_zip(&[("chrome-linux64/README", b"no browser here", None)]),
        );
        let probe = FakeProbe::new(V);
        let inst = test_installer(dir.path(), &server, probe.clone());

        let err = inst.install(&req()).await.unwrap_err();

        assert!(matches!(err, InstallError::BinaryMissing(_)), "{err}");
        assert_eq!(probe.smoke_calls.load(Ordering::SeqCst), 0);
        assert!(!dir.path().join("chrome-for-testing").exists());
    }

    #[tokio::test]
    async fn failed_cdp_smoke_keeps_the_previous_installation() {
        let (dir, server) = (tempfile::tempdir().unwrap(), TestServer::start().await);
        installed(dir.path(), &server).await;
        fs::write(dir.path().join("chrome-for-testing/marker"), "v1").unwrap();
        let probe = FakeProbe::new(V);
        probe.smoke_ok.store(false, Ordering::SeqCst);
        let broken = test_installer(dir.path(), &server, probe);

        let err = broken
            .install(&InstallRequest {
                force: true,
                ..req()
            })
            .await
            .unwrap_err();

        assert!(matches!(err, InstallError::SmokeFailed(_)), "{err}");
        assert_eq!(
            fs::read_to_string(dir.path().join("chrome-for-testing/marker")).unwrap(),
            "v1",
            "previous installation is the one still in place"
        );
        assert_eq!(names(dir.path()), [".install.lock", "chrome-for-testing"]);
    }

    #[tokio::test]
    async fn version_mismatch_fails_validation() {
        let (dir, server) = (tempfile::tempdir().unwrap(), TestServer::start().await);
        server.publish_chrome(V, Platform::LinuxX64, good_linux_zip());
        let probe = Arc::new(FakeProbe {
            version_output: "Chromium 99.0.1.2".to_string(),
            smoke_ok: true.into(),
            smoke_calls: 0.into(),
        });
        let inst = test_installer(dir.path(), &server, probe);

        let err = inst.install(&req()).await.unwrap_err();

        assert!(matches!(err, InstallError::VersionMismatch { .. }), "{err}");
    }

    #[tokio::test]
    async fn concurrent_install_is_refused_while_the_lock_is_held() {
        let (dir, server) = (tempfile::tempdir().unwrap(), TestServer::start().await);
        server.publish_chrome(V, Platform::LinuxX64, good_linux_zip());
        let mut inst = test_installer(dir.path(), &server, FakeProbe::new(V));
        inst.lock_wait = Duration::ZERO;
        let holder = fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(dir.path().join(LOCK_FILE))
            .unwrap();
        holder.lock().unwrap();

        let err = inst.install(&req()).await.unwrap_err();
        assert!(matches!(err, InstallError::LockHeld(_)), "{err}");
        assert_eq!(
            server.total_hits(),
            0,
            "a refused install does no network I/O"
        );

        drop(holder);
        inst.install(&req()).await.unwrap();
    }

    #[test]
    fn promote_failure_restores_the_previous_installation() {
        let tmp = tempfile::tempdir().unwrap();
        let target = tmp.path().join("chrome-for-testing");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("marker"), "old").unwrap();

        // Payload does not exist: the second rename fails after the first.
        let err = promote(
            tmp.path(),
            &tmp.path().join("missing"),
            &target,
            "chrome-for-testing",
        )
        .unwrap_err();

        assert!(
            matches!(err, InstallError::PromoteFailed { restored: true, .. }),
            "{err}"
        );
        assert_eq!(fs::read_to_string(target.join("marker")).unwrap(), "old");
        assert_eq!(names(tmp.path()), ["chrome-for-testing"]);
    }

    #[tokio::test]
    async fn offline_uses_the_validated_install_or_says_network_is_required() {
        let (dir, server) = (tempfile::tempdir().unwrap(), TestServer::start().await);
        let offline = InstallRequest {
            offline: true,
            ..req()
        };
        let inst = test_installer(dir.path(), &server, FakeProbe::new(V));

        let err = inst.install(&offline).await.unwrap_err();
        assert!(matches!(err, InstallError::OfflineNoInstall), "{err}");
        assert!(err.to_string().contains("network"));
        assert_eq!(server.total_hits(), 0);

        let inst = installed(dir.path(), &server).await;
        let hits = server.total_hits();
        let out = inst.install(&offline).await.unwrap();
        assert!(matches!(out, InstallOutcome::AlreadyInstalled { .. }));
        let out = inst
            .install(&InstallRequest {
                force: true,
                ..offline.clone()
            })
            .await
            .unwrap();
        assert!(matches!(out, InstallOutcome::AlreadyInstalled { .. }));
        let other = InstallRequest {
            version: Some("155.0.1.1".to_string()),
            ..offline
        };
        assert!(matches!(
            inst.install(&other).await.unwrap_err(),
            InstallError::OfflineVersionMismatch { .. }
        ));
        assert_eq!(
            server.total_hits(),
            hits,
            "offline never touches the network"
        );
    }

    #[tokio::test]
    async fn crashed_runs_are_cleaned_up_or_restored() {
        let (dir, server) = (tempfile::tempdir().unwrap(), TestServer::start().await);
        let root = dir.path();
        fs::create_dir_all(root.join(".staging-chrome-for-testing-1-2/junk")).unwrap();
        // Died between the two renames: only the aside copy is left.
        let aside = root.join(".previous-chrome-for-testing-1-2");
        fs::create_dir_all(aside.join("chrome-linux64")).unwrap();
        fs::write(aside.join("chrome-linux64/chrome"), "bin").unwrap();
        fs::write(aside.join("version.txt"), V).unwrap();
        let inst = test_installer(root, &server, FakeProbe::new(V));

        let out = inst
            .install(&InstallRequest {
                offline: true,
                ..req()
            })
            .await
            .unwrap();

        assert!(matches!(out, InstallOutcome::AlreadyInstalled { .. }));
        assert_eq!(names(root), [".install.lock", "chrome-for-testing"]);
    }

    #[tokio::test]
    async fn preview_reports_version_size_and_destination() {
        let (dir, server) = (tempfile::tempdir().unwrap(), TestServer::start().await);
        let zip = good_linux_zip();
        server.publish_chrome(V, Platform::LinuxX64, zip.clone());
        let inst = test_installer(dir.path(), &server, FakeProbe::new(V));

        let p = inst.preview(&req()).await.unwrap();

        assert_eq!(p.version, V);
        assert_eq!(p.size, Some(zip.len() as u64));
        assert_eq!(p.destination, dir.path().join("chrome-for-testing"));
        assert!(
            !dir.path().join("chrome-for-testing").exists(),
            "preview writes nothing"
        );
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn system_probe_reports_a_browser_that_dies_at_startup() {
        use std::os::unix::fs::PermissionsExt;
        let tmp = tempfile::tempdir().unwrap();
        let fake = tmp.path().join("chrome");
        fs::write(
            &fake,
            "#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then echo 'Google Chrome for Testing 1.2.3.4'; exit 0; fi\necho boom >&2\nexit 1\n",
        )
        .unwrap();
        fs::set_permissions(&fake, fs::Permissions::from_mode(0o755)).unwrap();
        let probe = SystemProbe {
            artifact: Artifact::Chrome,
        };

        assert!(probe.version(&fake).await.unwrap().contains("1.2.3.4"));
        let err = probe.cdp_smoke(&fake, tmp.path()).await.unwrap_err();
        assert!(err.contains("DevTools"), "{err}");
    }

    /// Real browser, real CDP. Run with
    /// `AUDITMYSITE_SMOKE_BROWSER=/path/to/chrome cargo test --lib real_probe -- --ignored`.
    #[tokio::test]
    #[ignore = "needs a real browser"]
    async fn real_probe_smoke_against_a_real_browser() {
        let path =
            std::env::var("AUDITMYSITE_SMOKE_BROWSER").expect("set AUDITMYSITE_SMOKE_BROWSER");
        let tmp = tempfile::tempdir().unwrap();
        let probe = SystemProbe {
            artifact: Artifact::Chrome,
        };
        let v = probe.version(Path::new(&path)).await.unwrap();
        eprintln!("version: {v}");
        probe.cdp_smoke(Path::new(&path), tmp.path()).await.unwrap();
    }
}
