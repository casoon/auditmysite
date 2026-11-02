//! Bounded checks of a browser binary (#854).
//!
//! - [`query_version`]: runs `--version` with a time limit, so a binary that
//!   hangs cannot stall discovery.
//! - [`probe_cdp`]: a genuine headless launch, DevTools connection and
//!   cleanup. A candidate is only called usable after this has worked; the
//!   launch goes through the same code as a real audit run.

use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use tracing::debug;

use super::manager::{BrowserManager, BrowserOptions};

/// Time a `--version` call may take.
pub const VERSION_TIMEOUT: Duration = Duration::from_secs(5);

/// Time a headless launch plus DevTools connection may take.
pub const CDP_PROBE_TIMEOUT: Duration = Duration::from_secs(20);

/// What running `<binary> --version` came to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VersionOutcome {
    /// The version number, e.g. `122.0.6261.94`
    Reported(String),
    /// The binary ran but printed no version (Windows GUI-subsystem browsers)
    NoVersion,
    /// No answer within the time limit; the process was killed
    TimedOut,
    /// The process could not be started (missing, wrong architecture, ...)
    CannotStart(String),
}

impl VersionOutcome {
    pub fn version(&self) -> Option<String> {
        match self {
            Self::Reported(v) => Some(v.clone()),
            _ => None,
        }
    }
}

/// Extract the version from output like `Google Chrome 122.0.6261.94`.
pub(super) fn parse_version(stdout: &str) -> Option<String> {
    stdout
        .split_whitespace()
        .find(|s| s.chars().next().is_some_and(|c| c.is_ascii_digit()))
        .map(|s| s.to_string())
}

/// Run `<path> --version`, waiting at most `timeout`.
pub fn query_version(path: &Path, timeout: Duration) -> VersionOutcome {
    let mut child = match Command::new(path)
        .arg("--version")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(child) => child,
        Err(e) => return VersionOutcome::CannotStart(e.to_string()),
    };

    // Read on a thread: a grandchild keeping the pipe open must not block us.
    let mut stdout = child.stdout.take();
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let mut text = String::new();
        if let Some(out) = stdout.as_mut() {
            let _ = out.take(64 * 1024).read_to_string(&mut text);
        }
        let _ = tx.send(text);
    });

    let deadline = Instant::now() + timeout;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                return VersionOutcome::TimedOut;
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(10)),
            Err(e) => return VersionOutcome::CannotStart(e.to_string()),
        }
    };

    if !status.success() {
        return VersionOutcome::NoVersion;
    }
    let text = rx
        .recv_timeout(Duration::from_millis(500))
        .unwrap_or_default();
    match parse_version(&text) {
        Some(v) => VersionOutcome::Reported(v),
        None => VersionOutcome::NoVersion,
    }
}

/// Why a headless probe failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProbeFailure {
    /// The process did not start or exited before offering DevTools
    LaunchFailed(String),
    /// It started but neither exited nor answered within the time limit
    TimedOut,
    /// DevTools was offered but the connection or first command failed
    ConnectFailed(String),
}

impl std::fmt::Display for ProbeFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LaunchFailed(e) => write!(f, "headless launch failed: {e}"),
            Self::TimedOut => write!(f, "no DevTools connection within the time limit"),
            Self::ConnectFailed(e) => write!(f, "DevTools connection failed: {e}"),
        }
    }
}

/// Start `path` headless, connect over CDP, ask for its version, close it.
///
/// Returns the product string the browser reports over CDP (`Chrome/131...`,
/// `Edg/131...`). The temporary profile is removed and the process ended on
/// every path, including the timeout.
pub async fn probe_cdp(
    path: &Path,
    no_sandbox: bool,
    timeout: Duration,
) -> Result<String, ProbeFailure> {
    let user_data_dir = BrowserManager::create_user_data_dir()
        .map_err(|e| ProbeFailure::LaunchFailed(e.to_string()))?;
    let options = BrowserOptions {
        no_sandbox,
        ..BrowserOptions::default()
    };

    let attempt = async {
        let (mut browser, mut handler, plan) =
            BrowserManager::launch_one(path, None, &user_data_dir, &options, timeout)
                .await
                .map_err(|failure| {
                    if failure.timed_out {
                        ProbeFailure::TimedOut
                    } else {
                        ProbeFailure::LaunchFailed(failure.reason)
                    }
                })?;
        debug!("Probe of {} launched with '{}'", path.display(), plan.label);

        let events = tokio::spawn(async move {
            use futures::StreamExt;
            while handler.next().await.is_some() {}
        });

        let product = browser
            .version()
            .await
            .map(|v| v.product)
            .map_err(|e| ProbeFailure::ConnectFailed(e.to_string()));

        let _ = browser.close().await;
        let _ = browser.wait().await;
        events.abort();
        product
    };

    // `Browser` kills its child when dropped, so the timeout also ends the process.
    let result = match tokio::time::timeout(timeout + Duration::from_secs(5), attempt).await {
        Ok(result) => result,
        Err(_) => Err(ProbeFailure::TimedOut),
    };
    let _ = std::fs::remove_dir_all(&user_data_dir);
    result
}

#[cfg(all(test, unix))]
pub(super) mod test_support {
    use std::os::unix::fs::PermissionsExt;
    use std::path::{Path, PathBuf};

    /// Write an executable shell script into `dir`.
    pub(crate) fn script(dir: &Path, name: &str, body: &str) -> PathBuf {
        let path = dir.join(name);
        std::fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        path
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::test_support::script;
    use super::*;

    #[test]
    fn version_is_parsed_from_typical_output() {
        assert_eq!(
            parse_version("Google Chrome 122.0.6261.94 \n"),
            Some("122.0.6261.94".to_string())
        );
        assert_eq!(parse_version("Chromium"), None);
    }

    #[test]
    fn version_query_reports_the_number() {
        let dir = tempfile::tempdir().unwrap();
        let bin = script(dir.path(), "chrome", "echo 'Google Chrome 131.0.1.2'");
        assert_eq!(
            query_version(&bin, Duration::from_secs(5)),
            VersionOutcome::Reported("131.0.1.2".to_string())
        );
    }

    #[test]
    fn a_binary_that_hangs_is_cut_off_at_the_time_limit() {
        let dir = tempfile::tempdir().unwrap();
        let bin = script(dir.path(), "chrome", "exec sleep 30");
        let started = Instant::now();
        let outcome = query_version(&bin, Duration::from_millis(300));
        assert_eq!(outcome, VersionOutcome::TimedOut);
        assert!(started.elapsed() < Duration::from_secs(5));
    }

    #[test]
    fn a_binary_that_cannot_start_is_reported_with_the_os_error() {
        let outcome = query_version(Path::new("/definitely/missing/browser"), VERSION_TIMEOUT);
        assert!(matches!(outcome, VersionOutcome::CannotStart(_)));
    }

    #[test]
    fn a_failing_version_call_is_no_version_not_a_start_failure() {
        let dir = tempfile::tempdir().unwrap();
        let bin = script(dir.path(), "chrome", "exit 3");
        assert_eq!(
            query_version(&bin, VERSION_TIMEOUT),
            VersionOutcome::NoVersion
        );
    }

    #[tokio::test]
    async fn probe_reports_a_process_that_exits_at_once_as_launch_failure() {
        let dir = tempfile::tempdir().unwrap();
        let bin = script(dir.path(), "chrome", "echo boom >&2; exit 1");
        let failure = probe_cdp(&bin, false, Duration::from_secs(5))
            .await
            .unwrap_err();
        assert!(
            matches!(failure, ProbeFailure::LaunchFailed(_)),
            "{failure:?}"
        );
    }

    #[tokio::test]
    async fn probe_cuts_off_a_process_that_never_offers_devtools() {
        let dir = tempfile::tempdir().unwrap();
        let bin = script(dir.path(), "chrome", "exec sleep 30");
        let started = Instant::now();
        let failure = probe_cdp(&bin, false, Duration::from_secs(1))
            .await
            .unwrap_err();
        assert_eq!(failure, ProbeFailure::TimedOut);
        assert!(started.elapsed() < Duration::from_secs(15));
    }

    #[tokio::test]
    async fn probe_reports_an_unreachable_devtools_endpoint_as_cdp_failure() {
        let dir = tempfile::tempdir().unwrap();
        // Announces a DevTools address nothing listens on.
        let bin = script(
            dir.path(),
            "chrome",
            "echo 'DevTools listening on ws://127.0.0.1:9/devtools/browser/x' >&2; exec sleep 30",
        );
        let failure = probe_cdp(&bin, false, Duration::from_secs(3))
            .await
            .unwrap_err();
        assert!(
            matches!(
                failure,
                ProbeFailure::LaunchFailed(_) | ProbeFailure::ConnectFailed(_)
            ),
            "{failure:?}"
        );
    }
}
