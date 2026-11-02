//! Browser detection across platforms
//!
//! Scans system paths, user-level installs, PATH and managed installs for
//! Chromium-based browsers. Every file found is assessed (exists, executable,
//! answers `--version` in time) and reported with its verdict, in the
//! priority order defined by [`registry`]. Duplicates (symlinks, the same
//! binary reachable through PATH and a system path) are listed once.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::Duration;

use tracing::debug;

use super::platform::{Os, Platform};
use super::probe::{query_version, VersionOutcome, VERSION_TIMEOUT};
use super::registry::{self, EnvPaths, RegistryEntry};
use super::resolver::{resolve_browser, BrowserResolveOptions};
use super::types::{
    BrowserKind, BrowserSource, CandidateReport, CandidateVerdict, DetectedBrowser, InstallTarget,
    RejectionKind,
};
use crate::browser::installer::BrowserInstaller;
use crate::error::{AuditError, Result};

/// Candidates found by a scan.
pub(super) struct Discovery {
    pub candidates: Vec<CandidateReport>,
    /// How many locations and PATH names were looked at
    pub searched: usize,
}

/// Detect all usable browsers on the system
///
/// Returns the browsers that are not rejected by a static check (exists,
/// executable, answers `--version`), in priority order: per kind (Chrome,
/// Edge, Ungoogled Chromium, Chromium) the system install, the user-level
/// install, then PATH. Does NOT include managed installs (handled by the
/// resolver). Whether a browser really starts is checked by
/// [`super::probe::probe_cdp`].
pub fn detect_all_browsers() -> Vec<DetectedBrowser> {
    discover(
        Platform::current(),
        &EnvPaths::from_process(),
        VERSION_TIMEOUT,
    )
    .candidates
    .into_iter()
    .filter(|c| c.verdict.is_viable())
    .map(|c| c.browser)
    .collect()
}

/// Scan for browsers on `platform`. Pure in its inputs, so every platform's
/// scan can be tested on any host.
pub(super) fn discover(platform: Platform, env: &EnvPaths, version_timeout: Duration) -> Discovery {
    let mut candidates = Vec::new();
    let mut seen: HashSet<PathBuf> = HashSet::new();
    let mut searched = 0;

    for &kind in registry::search_order(platform) {
        let mut entries = registry::install_locations(platform, kind, env);
        searched += entries.len();
        for name in registry::path_names(platform, kind) {
            searched += 1;
            if let Some(path) = registry::find_in_path(name, &env.path_dirs) {
                entries.push(RegistryEntry {
                    kind,
                    path,
                    source: BrowserSource::PathSearch,
                });
            }
        }

        for entry in entries {
            if !entry.path.exists() {
                continue;
            }
            // The canonical path only serves as the de-duplication key: the
            // candidate keeps the path it was found under (a snap or flatpak
            // launcher resolves to a binary that must not be started directly).
            let key = std::fs::canonicalize(&entry.path).unwrap_or_else(|_| entry.path.clone());
            if !seen.insert(key) {
                debug!("{} is a duplicate, skipped", entry.path.display());
                continue;
            }
            candidates.push(assess_binary(
                &entry.path,
                entry.kind,
                entry.source,
                version_timeout,
            ));
        }
    }

    Discovery {
        candidates,
        searched,
    }
}

/// Managed installs under `~/.auditmysite/` in priority order: Chrome for
/// Testing, the headless shell, the legacy location. An install directory
/// whose binary is gone or unusable is reported as stale instead of being
/// treated as present.
pub(super) fn managed_candidates(
    platform: Platform,
    home: Option<&Path>,
    version_timeout: Duration,
) -> Vec<CandidateReport> {
    let Some(home) = home else {
        return Vec::new();
    };
    let base = home.join(".auditmysite");
    let installs = [
        (
            base.join("browsers").join("chrome-for-testing"),
            Some(InstallTarget::ChromeForTesting),
            BrowserKind::ChromeForTesting,
        ),
        (
            base.join("browsers").join("headless-shell"),
            Some(InstallTarget::HeadlessShell),
            BrowserKind::HeadlessShell,
        ),
        (base.join("chromium"), None, BrowserKind::ChromeForTesting),
    ];

    let mut reports = Vec::new();
    for (dir, target, kind) in installs {
        if !dir.exists() {
            continue;
        }
        let version_file = read_version_file(&dir);
        let reinstall = "run `auditmysite browser install --force` to repair it";

        let mut report = if let Some(reason) = platform.managed_unavailable_reason() {
            CandidateReport {
                browser: DetectedBrowser {
                    kind,
                    path: dir.clone(),
                    version: version_file.clone(),
                    source: BrowserSource::ManagedInstall,
                },
                verdict: CandidateVerdict::rejected(RejectionKind::Excluded, reason),
                cdp_product: None,
            }
        } else {
            let path = match target {
                Some(target) => BrowserInstaller::binary_path_for(platform, &dir, target),
                None => legacy_chromium_path(platform, &dir),
            }
            .unwrap_or_else(|| dir.clone());
            if path.exists() {
                assess_binary(&path, kind, BrowserSource::ManagedInstall, version_timeout)
            } else {
                CandidateReport {
                    browser: DetectedBrowser {
                        kind,
                        path,
                        version: None,
                        source: BrowserSource::ManagedInstall,
                    },
                    verdict: CandidateVerdict::rejected(
                        RejectionKind::StaleInstall,
                        format!(
                            "stale install: {} exists but its browser binary is missing; {}",
                            dir.display(),
                            reinstall
                        ),
                    ),
                    cdp_product: None,
                }
            }
        };
        if report.browser.version.is_none() {
            report.browser.version = version_file;
        }
        reports.push(report);
    }
    reports
}

/// Binary of the legacy install (`~/.auditmysite/chromium/`).
fn legacy_chromium_path(platform: Platform, dir: &Path) -> Option<PathBuf> {
    match platform.os {
        Os::MacOs => Some(
            dir.join("chrome-mac")
                .join("Chromium.app")
                .join("Contents")
                .join("MacOS")
                .join("Chromium"),
        ),
        Os::Linux => Some(dir.join("chrome-linux").join("chrome")),
        _ => None,
    }
}

/// Version from the `version.txt` an install leaves in its directory.
fn read_version_file(dir: &Path) -> Option<String> {
    std::fs::read_to_string(dir.join("version.txt"))
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Static check of one browser file: exists, is executable, answers
/// `--version` within `version_timeout`. Starts the binary but not a browser
/// session.
pub(super) fn assess_binary(
    path: &Path,
    kind: BrowserKind,
    source: BrowserSource,
    version_timeout: Duration,
) -> CandidateReport {
    let mut browser = DetectedBrowser {
        kind,
        path: path.to_path_buf(),
        version: None,
        source,
    };
    let verdict = if !path.exists() {
        CandidateVerdict::rejected(RejectionKind::NotFound, "no file at this path")
    } else if !path.is_file() {
        CandidateVerdict::rejected(RejectionKind::NotFound, "not a regular file")
    } else if !is_executable(path) {
        CandidateVerdict::rejected(
            RejectionKind::NotExecutable,
            format!("no execute permission (try: chmod +x {})", path.display()),
        )
    } else {
        match query_version(path, version_timeout) {
            VersionOutcome::Reported(version) => {
                browser.version = Some(version);
                CandidateVerdict::Viable
            }
            VersionOutcome::NoVersion => CandidateVerdict::Viable,
            VersionOutcome::TimedOut => CandidateVerdict::rejected(
                RejectionKind::Hangs,
                format!(
                    "`--version` gave no answer within {}s; the process was killed",
                    version_timeout.as_secs_f32().max(0.1)
                ),
            ),
            VersionOutcome::CannotStart(reason) => CandidateVerdict::rejected(
                RejectionKind::NotRunnable,
                format!("cannot be started: {reason}"),
            ),
        }
    };
    CandidateReport {
        browser,
        verdict,
        cdp_product: None,
    }
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(path).is_ok_and(|m| m.permissions().mode() & 0o111 != 0)
}

#[cfg(not(unix))]
fn is_executable(_path: &Path) -> bool {
    true
}

/// Verify that a binary is executable
pub(super) fn verify_executable(path: &Path) -> Result<()> {
    if is_executable(path) {
        Ok(())
    } else {
        Err(AuditError::ChromeNotExecutable {
            path: path.to_path_buf(),
        })
    }
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
                BrowserSource::SystemPath | BrowserSource::UserInstall => {
                    DetectionMethod::StandardPath
                }
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

/// Legacy: Find Chrome using all available methods (the resolver's rules)
pub fn find_chrome(manual_path: Option<&str>) -> Result<ChromeInfo> {
    let opts = BrowserResolveOptions {
        browser_path: manual_path.map(str::to_string),
        ..Default::default()
    };
    resolve_browser(&opts).map(|resolved| ChromeInfo::from(&resolved.browser))
}

#[cfg(all(test, unix))]
mod tests {
    use super::super::platform::Arch;
    use super::super::probe::test_support::script;
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    const LINUX_X64: Platform = Platform::new(Os::Linux, Arch::X86_64);
    const WIN_X64: Platform = Platform::new(Os::Windows, Arch::X86_64);
    const T: Duration = Duration::from_secs(5);

    fn chrome_script(dir: &Path, name: &str) -> PathBuf {
        script(dir, name, "echo 'Google Chrome 131.0.1.2'")
    }

    fn only_path(dir: &Path) -> EnvPaths {
        EnvPaths {
            root: Some(dir.join("sysroot")),
            path_dirs: vec![dir.to_path_buf()],
            ..Default::default()
        }
    }

    #[test]
    fn no_browser_anywhere_finds_nothing() {
        let empty = tempfile::tempdir().unwrap();
        let found = discover(LINUX_X64, &only_path(empty.path()), T);
        assert!(found.candidates.is_empty());
        assert!(found.searched > 0);
    }

    #[test]
    fn path_search_finds_a_browser_and_labels_its_source() {
        let bin = tempfile::tempdir().unwrap();
        chrome_script(bin.path(), "google-chrome");
        let found = discover(LINUX_X64, &only_path(bin.path()), T).candidates;
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].browser.kind, BrowserKind::Chrome);
        assert_eq!(found[0].browser.source, BrowserSource::PathSearch);
        assert_eq!(found[0].browser.version.as_deref(), Some("131.0.1.2"));
        assert!(found[0].verdict.is_viable());
    }

    #[test]
    fn user_level_install_is_found_under_home() {
        let home = tempfile::tempdir().unwrap();
        let local_bin = home.path().join(".local/bin");
        std::fs::create_dir_all(&local_bin).unwrap();
        chrome_script(&local_bin, "chromium");
        let env = EnvPaths {
            root: Some(home.path().join("sysroot")),
            home: Some(home.path().to_path_buf()),
            ..Default::default()
        };
        let found = discover(LINUX_X64, &env, T).candidates;
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].browser.source, BrowserSource::UserInstall);
        assert_eq!(found[0].browser.kind, BrowserKind::Chromium);
    }

    #[test]
    fn the_same_binary_through_symlink_and_path_is_listed_once() {
        let real = tempfile::tempdir().unwrap();
        let bin = tempfile::tempdir().unwrap();
        let target = chrome_script(real.path(), "chrome-real");
        std::os::unix::fs::symlink(&target, bin.path().join("google-chrome")).unwrap();
        std::os::unix::fs::symlink(&target, bin.path().join("google-chrome-stable")).unwrap();
        let found = discover(LINUX_X64, &only_path(bin.path()), T).candidates;
        assert_eq!(found.len(), 1, "{found:?}");
        // The candidate keeps the path it was found under, not the canonical one.
        assert_eq!(found[0].browser.path, bin.path().join("google-chrome"));
    }

    #[test]
    fn candidates_follow_the_kind_priority_not_the_path_order() {
        let first = tempfile::tempdir().unwrap();
        let second = tempfile::tempdir().unwrap();
        chrome_script(first.path(), "chromium");
        chrome_script(second.path(), "google-chrome");
        let env = EnvPaths {
            root: Some(first.path().join("sysroot")),
            path_dirs: vec![first.path().into(), second.path().into()],
            ..Default::default()
        };
        let kinds: Vec<_> = discover(LINUX_X64, &env, T)
            .candidates
            .iter()
            .map(|c| c.browser.kind)
            .collect();
        assert_eq!(kinds, vec![BrowserKind::Chrome, BrowserKind::Chromium]);
    }

    #[test]
    fn a_file_without_execute_permission_is_rejected_as_not_executable() {
        let bin = tempfile::tempdir().unwrap();
        let path = chrome_script(bin.path(), "google-chrome");
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
        let found = discover(LINUX_X64, &only_path(bin.path()), T).candidates;
        assert!(matches!(
            found[0].verdict,
            CandidateVerdict::Rejected {
                kind: RejectionKind::NotExecutable,
                ..
            }
        ));
    }

    #[test]
    fn a_binary_whose_version_call_hangs_is_rejected_within_the_bound() {
        let bin = tempfile::tempdir().unwrap();
        script(bin.path(), "google-chrome", "exec sleep 30");
        let started = std::time::Instant::now();
        let found = discover(
            LINUX_X64,
            &only_path(bin.path()),
            Duration::from_millis(300),
        )
        .candidates;
        assert!(started.elapsed() < Duration::from_secs(5));
        assert!(matches!(
            found[0].verdict,
            CandidateVerdict::Rejected {
                kind: RejectionKind::Hangs,
                ..
            }
        ));
    }

    #[test]
    fn a_binary_that_cannot_run_is_rejected_as_not_runnable() {
        let bin = tempfile::tempdir().unwrap();
        // Executable, but its interpreter does not exist (like a binary built for
        // another architecture, the OS refuses to start it).
        let path = bin.path().join("google-chrome");
        std::fs::write(&path, "#!/nonexistent/interpreter\n").unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        let found = discover(LINUX_X64, &only_path(bin.path()), T).candidates;
        assert!(matches!(
            found[0].verdict,
            CandidateVerdict::Rejected {
                kind: RejectionKind::NotRunnable,
                ..
            }
        ));
    }

    #[test]
    fn windows_scan_ignores_chrome_even_when_it_is_installed() {
        let pf = tempfile::tempdir().unwrap();
        let chrome_dir = pf.path().join("Google/Chrome/Application");
        let edge_dir = pf.path().join("Microsoft/Edge/Application");
        std::fs::create_dir_all(&chrome_dir).unwrap();
        std::fs::create_dir_all(&edge_dir).unwrap();
        chrome_script(&chrome_dir, "chrome.exe");
        chrome_script(&edge_dir, "msedge.exe");
        let env = EnvPaths {
            program_files: Some(pf.path().to_path_buf()),
            program_files_x86: Some(pf.path().join("none")),
            path_dirs: vec![chrome_dir.clone()],
            ..Default::default()
        };
        let found = discover(WIN_X64, &env, T).candidates;
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].browser.kind, BrowserKind::Edge);
        assert_eq!(found[0].browser.source, BrowserSource::SystemPath);
    }

    #[test]
    fn stale_managed_install_is_reported_not_treated_as_present() {
        let home = tempfile::tempdir().unwrap();
        let dir = home.path().join(".auditmysite/browsers/chrome-for-testing");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("version.txt"), "120.0.0.0\n").unwrap();
        let platform = Platform::new(Os::Linux, Arch::X86_64);
        let reports = managed_candidates(platform, Some(home.path()), T);
        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].browser.version.as_deref(), Some("120.0.0.0"));
        let CandidateVerdict::Rejected { kind, detail } = &reports[0].verdict else {
            panic!("stale install must be rejected");
        };
        assert_eq!(*kind, RejectionKind::StaleInstall);
        assert!(detail.contains("browser install --force"), "{detail}");
    }

    #[test]
    fn managed_install_with_a_working_binary_is_viable() {
        let home = tempfile::tempdir().unwrap();
        let dir = home.path().join(".auditmysite/browsers/chrome-for-testing");
        let platform = Platform::new(Os::Linux, Arch::X86_64);
        let binary =
            BrowserInstaller::binary_path_for(platform, &dir, InstallTarget::ChromeForTesting)
                .unwrap();
        std::fs::create_dir_all(binary.parent().unwrap()).unwrap();
        chrome_script(binary.parent().unwrap(), "chrome");
        let reports = managed_candidates(platform, Some(home.path()), T);
        assert!(reports[0].verdict.is_viable());
        assert_eq!(reports[0].browser.kind, BrowserKind::ChromeForTesting);
    }

    #[test]
    fn managed_installs_are_excluded_on_windows_and_linux_arm64() {
        let home = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(home.path().join(".auditmysite/browsers/headless-shell")).unwrap();
        for platform in [WIN_X64, Platform::new(Os::Linux, Arch::Aarch64)] {
            let reports = managed_candidates(platform, Some(home.path()), T);
            assert!(
                matches!(
                    reports[0].verdict,
                    CandidateVerdict::Rejected {
                        kind: RejectionKind::Excluded,
                        ..
                    }
                ),
                "{platform:?}"
            );
        }
    }

    #[test]
    fn test_find_chrome_with_invalid_manual_path() {
        assert!(find_chrome(Some("/nonexistent/path/to/chrome")).is_err());
    }

    #[test]
    fn test_chrome_info_from_detected_browser() {
        let browser = DetectedBrowser {
            kind: BrowserKind::Chrome,
            path: PathBuf::from("/usr/bin/chrome"),
            version: Some("131.0".to_string()),
            source: BrowserSource::UserInstall,
        };
        let info = ChromeInfo::from(&browser);
        assert_eq!(info.detection_method, DetectionMethod::StandardPath);
        assert_eq!(info.path, PathBuf::from("/usr/bin/chrome"));
    }
}
