//! Browser resolver - finds the best available browser
//!
//! Priority order:
//! 1. --browser-path (explicit CLI flag)
//! 2. AUDITMYSITE_BROWSER env var
//! 3. CHROME_PATH env var (deprecated, backwards compat; ignored when it
//!    points nowhere, because other tools set it too)
//! 4. System scan, per kind (Chrome → Edge → Ungoogled Chromium → Chromium)
//!    system install, user-level install, PATH (see `registry`)
//! 5. Managed install (~/.auditmysite/browsers/: Chrome for Testing, then
//!    the headless shell)
//! 6. Error with installation hints
//!
//! An explicit path (1–3) is never replaced silently: when it is unusable the
//! resolution fails with the reason, unless the caller asked for
//! `fallback_to_auto`. Windows runs Microsoft Edge only (also for explicit
//! paths); managed installs do not exist there.
//!
//! A full browser stays first on purpose. The headless shell never stalls
//! (see [`parallel_journeys_stall`]), but sites with bot protection serve it
//! a challenge page: www.hornbach.de gave it 355 nodes instead of 12,942, and
//! the audit scored the challenge page (plan 65). Existing working browsers
//! also come before a managed install.
//!
//! Resolution here is static (file, permission, bounded `--version`). That a
//! browser really starts is shown by [`diagnose_browsers`] (headless CDP
//! probe) and, for an audit run, by the launch itself, which hands over to the
//! next candidate when an automatically selected browser fails
//! ([`ResolvedBrowser::launch_chain`]).

use std::time::Duration;

use tracing::{debug, info, warn};

use super::detection::{assess_binary, discover, managed_candidates};
use super::platform::{ExplicitPolicy, Platform};
use super::probe::{probe_cdp, ProbeFailure, CDP_PROBE_TIMEOUT, VERSION_TIMEOUT};
use super::registry::{self, EnvPaths};
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
    /// Use the automatic candidates when an explicit path is unusable. Off by
    /// default: an explicit path that fails must fail, not switch browsers.
    pub fallback_to_auto: bool,
}

/// Everything resolution reads from the outside world.
pub(super) struct ResolveContext {
    pub platform: Platform,
    pub paths: EnvPaths,
    /// `AUDITMYSITE_BROWSER`
    pub env_browser: Option<String>,
    /// `CHROME_PATH` (deprecated)
    pub env_chrome_path: Option<String>,
    pub version_timeout: Duration,
}

impl ResolveContext {
    pub(super) fn from_process() -> Self {
        let var = |name: &str| std::env::var(name).ok().filter(|v| !v.is_empty());
        Self {
            platform: Platform::current(),
            paths: EnvPaths::from_process(),
            env_browser: var("AUDITMYSITE_BROWSER"),
            env_chrome_path: var("CHROME_PATH"),
            version_timeout: VERSION_TIMEOUT,
        }
    }
}

/// Candidates collected for one resolution, before one is selected.
pub(super) struct Collected {
    pub candidates: Vec<CandidateReport>,
    pub searched: usize,
    /// The first candidate is an explicit path
    pub explicit: bool,
    pub mode: BrowserMode,
}

/// Find the best available browser
pub fn resolve_browser(opts: &BrowserResolveOptions) -> Result<ResolvedBrowser> {
    resolve_in(&ResolveContext::from_process(), opts)
}

pub(super) fn resolve_in(
    ctx: &ResolveContext,
    opts: &BrowserResolveOptions,
) -> Result<ResolvedBrowser> {
    let collected = collect_in(ctx, opts)?;
    let Collected {
        candidates,
        explicit,
        mode,
        ..
    } = collected;

    // An explicit path that is unusable fails with its own reason.
    if explicit && !opts.fallback_to_auto {
        if let Some(first) = candidates.first() {
            if !first.verdict.is_viable() {
                return Err(explicit_error(first));
            }
        }
    }

    let selected = candidates.iter().find(|c| c.verdict.is_viable());
    let Some(selected) = selected else {
        let only_excluded = candidates.iter().all(|c| {
            matches!(
                c.verdict,
                CandidateVerdict::Rejected {
                    kind: RejectionKind::Excluded,
                    ..
                }
            )
        });
        if only_excluded {
            return Err(AuditError::ChromeNotFound);
        }
        return Err(AuditError::BrowserLaunchFailed {
            reason: summarize(&candidates),
        });
    };

    info!(
        "Selected browser: {} v{} ({}, {})",
        selected.browser.kind.display_name(),
        selected.browser.version.as_deref().unwrap_or("unknown"),
        selected.browser.path.display(),
        selected.browser.source
    );
    Ok(ResolvedBrowser {
        browser: selected.browser.clone(),
        mode,
        all_candidates: candidates.iter().map(|c| c.browser.clone()).collect(),
        explicit: explicit && candidates[0].verdict.is_viable(),
        candidates,
    })
}

/// Gather the candidates in priority order with their static verdicts.
///
/// `Err` only for a request the platform contract rules out; an unusable
/// candidate is a verdict, not an error (so diagnostics can show it).
pub(super) fn collect_in(ctx: &ResolveContext, opts: &BrowserResolveOptions) -> Result<Collected> {
    let platform = ctx.platform;
    if let Some(reason) = platform.unsupported_reason() {
        return Err(AuditError::UnsupportedBrowser { reason });
    }
    let mode = if opts.strict {
        BrowserMode::Strict
    } else {
        BrowserMode::Standard
    };

    let mut candidates = Vec::new();
    let mut explicit = false;

    // 1.-3. Explicit paths: the first one that is set decides.
    let explicit_source = if let Some(path) = &opts.browser_path {
        Some((path.clone(), BrowserSource::CliFlag, BrowserKind::Custom))
    } else if let Some(path) = &ctx.env_browser {
        Some((path.clone(), BrowserSource::EnvVar, BrowserKind::Custom))
    } else {
        ctx.env_chrome_path.as_ref().and_then(|raw| {
            let expanded = registry::expand_home(raw, ctx.paths.home.as_deref());
            if expanded.exists() {
                warn!(
                    "CHROME_PATH is deprecated, use AUDITMYSITE_BROWSER or --browser-path instead"
                );
                Some((raw.clone(), BrowserSource::EnvVar, BrowserKind::Chrome))
            } else {
                debug!("CHROME_PATH points to a non-existent path, ignored: {raw}");
                None
            }
        })
    };
    if let Some((raw, source, kind)) = explicit_source {
        explicit = true;
        candidates.push(assess_explicit(ctx, &raw, source, kind));
        if !opts.fallback_to_auto {
            return Ok(Collected {
                candidates,
                searched: 1,
                explicit,
                mode,
            });
        }
    }

    // 4. Preference filter
    let filter_kind: Option<BrowserKind> = match opts
        .browser_preference
        .as_deref()
        .map(str::to_lowercase)
        .as_deref()
    {
        Some("chrome") => Some(BrowserKind::Chrome),
        Some("edge") => Some(BrowserKind::Edge),
        Some("chromium") => Some(BrowserKind::Chromium),
        Some("auto") | Some("") | None => None,
        Some(other) => {
            warn!("Unknown browser kind '{}', using auto-detection", other);
            None
        }
    };
    if let Some(kind) = filter_kind {
        if !platform.allows_discovered(kind) {
            return Err(AuditError::UnsupportedBrowser {
                reason: format!(
                    "--browser {} is not available on {}: only Microsoft Edge is supported on Windows",
                    kind.display_name().to_lowercase(),
                    platform.describe()
                ),
            });
        }
    }

    // 4. System scan
    let discovery = discover(platform, &ctx.paths, ctx.version_timeout);
    debug!("Found {} browser candidates", discovery.candidates.len());
    candidates.extend(discovery.candidates);

    // 5. Managed installs, after every existing browser
    candidates.extend(managed_candidates(
        platform,
        ctx.paths.home.as_deref(),
        ctx.version_timeout,
    ));

    // Exclusions: strict mode and the preference filter.
    for candidate in candidates.iter_mut().skip(usize::from(explicit)) {
        if !candidate.verdict.is_viable() {
            continue;
        }
        if candidate.browser.source == BrowserSource::ManagedInstall {
            if mode == BrowserMode::Strict {
                candidate.verdict = CandidateVerdict::rejected(
                    RejectionKind::Excluded,
                    "strict mode: managed installs are not used",
                );
            } else if let Some(kind) = filter_kind {
                candidate.verdict = CandidateVerdict::rejected(
                    RejectionKind::Excluded,
                    format!(
                        "excluded by --browser {}",
                        kind.display_name().to_lowercase()
                    ),
                );
            }
        } else if filter_kind.is_some_and(|kind| kind != candidate.browser.kind) {
            candidate.verdict = CandidateVerdict::rejected(
                RejectionKind::Excluded,
                format!(
                    "excluded by --browser {}",
                    filter_kind.unwrap().display_name().to_lowercase()
                ),
            );
        }
    }

    Ok(Collected {
        candidates,
        searched: discovery.searched,
        explicit,
        mode,
    })
}

/// Assess an explicitly given path: platform policy first, then the file.
fn assess_explicit(
    ctx: &ResolveContext,
    raw: &str,
    source: BrowserSource,
    kind: BrowserKind,
) -> CandidateReport {
    let path = registry::expand_home(raw, ctx.paths.home.as_deref());
    if let ExplicitPolicy::Rejected(reason) = ctx.platform.check_explicit(&path.to_string_lossy()) {
        return CandidateReport {
            browser: DetectedBrowser {
                kind,
                path,
                version: None,
                source,
            },
            verdict: CandidateVerdict::rejected(RejectionKind::Excluded, reason),
            cdp_product: None,
        };
    }
    assess_binary(&path, kind, source, ctx.version_timeout)
}

/// The error for an unusable explicit path.
fn explicit_error(report: &CandidateReport) -> AuditError {
    let path = report.browser.path.clone();
    let CandidateVerdict::Rejected { kind, detail } = &report.verdict else {
        unreachable!("only called for rejected candidates")
    };
    match kind {
        RejectionKind::NotFound if detail == "no file at this path" => AuditError::FileError {
            path,
            reason: "Browser binary not found at specified path".to_string(),
        },
        RejectionKind::NotExecutable => AuditError::ChromeNotExecutable { path },
        RejectionKind::Excluded => AuditError::UnsupportedBrowser {
            reason: detail.clone(),
        },
        _ => AuditError::BrowserLaunchFailed {
            reason: format!("{} ({}): {}", path.display(), report.browser.source, detail),
        },
    }
}

/// One line per rejected candidate.
fn summarize(candidates: &[CandidateReport]) -> String {
    candidates
        .iter()
        .map(|c| match &c.verdict {
            CandidateVerdict::Viable => {
                format!("{}: usable", c.browser.path.display())
            }
            CandidateVerdict::Rejected { kind, detail } => {
                format!(
                    "{}: {} ({})",
                    c.browser.path.display(),
                    detail,
                    kind.as_str()
                )
            }
        })
        .collect::<Vec<_>>()
        .join(" || ")
}

/// Selection, alternatives and the reason for each candidate (#854).
#[derive(Debug, Clone)]
pub struct BrowserDiagnosis {
    pub platform: Platform,
    /// Why nothing can be resolved on this platform or with these options
    pub problem: Option<String>,
    /// Every candidate in priority order with its verdict
    pub candidates: Vec<CandidateReport>,
    /// Locations and PATH names looked at
    pub searched: usize,
    /// The first candidate is an explicit path
    pub explicit: bool,
    /// Whether a headless CDP probe ran for each viable candidate
    pub probed: bool,
}

impl BrowserDiagnosis {
    /// The candidate an audit run would start with.
    pub fn selected(&self) -> Option<&CandidateReport> {
        self.candidates.iter().find(|c| c.verdict.is_viable())
    }

    fn failed(platform: Platform, problem: String) -> Self {
        Self {
            platform,
            problem: Some(problem),
            candidates: Vec::new(),
            searched: 0,
            explicit: false,
            probed: false,
        }
    }
}

/// Static diagnosis: files, permissions and bounded `--version` calls.
pub fn diagnose_browsers_static(opts: &BrowserResolveOptions) -> BrowserDiagnosis {
    diagnose_static_in(&ResolveContext::from_process(), opts)
}

pub(super) fn diagnose_static_in(
    ctx: &ResolveContext,
    opts: &BrowserResolveOptions,
) -> BrowserDiagnosis {
    match collect_in(ctx, opts) {
        Ok(c) => BrowserDiagnosis {
            platform: ctx.platform,
            problem: None,
            candidates: c.candidates,
            searched: c.searched,
            explicit: c.explicit,
            probed: false,
        },
        Err(e) => BrowserDiagnosis::failed(ctx.platform, e.to_string()),
    }
}

/// Full diagnosis: the static one plus a bounded headless CDP launch,
/// connection and cleanup for every statically viable candidate. A candidate
/// is only viable after that probe has worked.
pub async fn diagnose_browsers(opts: &BrowserResolveOptions, no_sandbox: bool) -> BrowserDiagnosis {
    let mut diagnosis = diagnose_browsers_static(opts);
    probe_candidates(&mut diagnosis.candidates, no_sandbox, CDP_PROBE_TIMEOUT).await;
    diagnosis.probed = true;
    diagnosis
}

/// Run the CDP probe for each viable candidate and record the outcome.
pub(super) async fn probe_candidates(
    candidates: &mut [CandidateReport],
    no_sandbox: bool,
    timeout: Duration,
) {
    for candidate in candidates.iter_mut().filter(|c| c.verdict.is_viable()) {
        match probe_cdp(&candidate.browser.path, no_sandbox, timeout).await {
            Ok(product) => candidate.cdp_product = Some(product),
            Err(failure) => {
                let kind = match failure {
                    ProbeFailure::TimedOut => RejectionKind::CdpTimedOut,
                    _ => RejectionKind::CdpFailed,
                };
                candidate.verdict = CandidateVerdict::rejected(kind, failure.to_string());
            }
        }
    }
}

/// Whether keyboard journeys on parallel pages can stall `browser`.
///
/// A full Chrome in `--headless=new` still runs the macOS event loop. Keyboard
/// events a page does not consume go through AppKit on the browser's single
/// main thread and wait on the WindowServer. With several pages audited at
/// once that stalled the whole browser: every open page's journeys timed out
/// in the same second, twice it never recovered (plan 65, reference batch
/// with three workers: 4 of 4 runs affected). One page at a time: 2 of 2 clean,
/// as were 6 of 6 single-URL runs. Serializing only the journeys did not help
/// (1–10 timeouts). The headless shell has no event loop and is not affected.
/// A headless shell passed via `--browser-path` is recognized by file name.
pub fn stalls_on_keyboard_journeys(browser: &DetectedBrowser) -> bool {
    let is_shell = browser.kind == BrowserKind::HeadlessShell
        || browser
            .path
            .file_name()
            .is_some_and(|name| name.to_string_lossy().starts_with("chrome-headless-shell"));
    cfg!(target_os = "macos") && !is_shell
}

/// [`stalls_on_keyboard_journeys`] for the browser this run would use.
/// Resolved once per process; `false` when no browser can be resolved (the
/// run fails later with its own error).
pub fn parallel_journeys_stall(browser_path: Option<&str>) -> bool {
    static STALLS: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *STALLS.get_or_init(|| {
        let opts = BrowserResolveOptions {
            browser_path: browser_path.map(str::to_string),
            ..Default::default()
        };
        resolve_browser(&opts)
            .map(|resolved| stalls_on_keyboard_journeys(&resolved.browser))
            .unwrap_or(false)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::browser::installer::BrowserInstaller;
    use crate::browser::platform::{Arch, Os};
    use std::path::PathBuf;

    fn browser(kind: BrowserKind, path: &str) -> DetectedBrowser {
        DetectedBrowser {
            kind,
            path: PathBuf::from(path),
            version: None,
            source: BrowserSource::SystemPath,
        }
    }

    #[test]
    fn headless_shell_never_counts_as_stalling() {
        let managed = browser(BrowserKind::HeadlessShell, "/x/chrome-headless-shell");
        let explicit = browser(BrowserKind::Custom, "/cache/chrome-headless-shell");
        assert!(!stalls_on_keyboard_journeys(&managed));
        assert!(!stalls_on_keyboard_journeys(&explicit));
    }

    #[test]
    fn full_chrome_stalls_only_on_macos() {
        let chrome = browser(
            BrowserKind::Chrome,
            "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
        );
        assert_eq!(
            stalls_on_keyboard_journeys(&chrome),
            cfg!(target_os = "macos")
        );
    }

    #[test]
    fn test_resolve_with_invalid_explicit_path() {
        let opts = BrowserResolveOptions {
            browser_path: Some("/nonexistent/browser".to_string()),
            ..Default::default()
        };
        assert!(resolve_browser(&opts).is_err());
    }

    #[test]
    fn managed_binary_path_uses_the_download_layout_of_the_platform() {
        let base = PathBuf::from("/home/user/.auditmysite/browsers/chrome-for-testing");
        let linux = BrowserInstaller::binary_path_for(
            Platform::new(Os::Linux, Arch::X86_64),
            &base,
            InstallTarget::ChromeForTesting,
        )
        .unwrap();
        assert!(linux.ends_with("chrome-linux64/chrome"));
        let mac = BrowserInstaller::binary_path_for(
            Platform::new(Os::MacOs, Arch::Aarch64),
            &base,
            InstallTarget::ChromeForTesting,
        )
        .unwrap();
        assert!(mac.to_string_lossy().contains("chrome-mac-arm64"));
        assert!(BrowserInstaller::binary_path_for(
            Platform::new(Os::Windows, Arch::X86_64),
            &base,
            InstallTarget::ChromeForTesting
        )
        .is_none());
    }
}

#[cfg(all(test, unix))]
mod resolution_tests {
    use super::*;
    use crate::browser::installer::BrowserInstaller;
    use crate::browser::platform::{Arch, Os};
    use crate::browser::probe::test_support::script;
    use std::os::unix::fs::PermissionsExt;
    use std::path::{Path, PathBuf};

    const MAC_ARM: Platform = Platform::new(Os::MacOs, Arch::Aarch64);
    const MAC_INTEL: Platform = Platform::new(Os::MacOs, Arch::X86_64);
    const LINUX_X64: Platform = Platform::new(Os::Linux, Arch::X86_64);
    const LINUX_ARM: Platform = Platform::new(Os::Linux, Arch::Aarch64);
    const WIN_X64: Platform = Platform::new(Os::Windows, Arch::X86_64);

    /// A fake machine: system root, home and one PATH directory in a tempdir.
    struct Machine {
        dir: tempfile::TempDir,
        ctx: ResolveContext,
    }

    impl Machine {
        fn new(platform: Platform) -> Self {
            let dir = tempfile::tempdir().unwrap();
            let root = dir.path().join("sys");
            let home = dir.path().join("home");
            let bin = dir.path().join("bin");
            for d in [&root, &home, &bin] {
                std::fs::create_dir_all(d).unwrap();
            }
            let ctx = ResolveContext {
                platform,
                paths: EnvPaths {
                    root: Some(root),
                    home: Some(home),
                    local_app_data: None,
                    program_files: Some(dir.path().join("pf")),
                    program_files_x86: Some(dir.path().join("pf86")),
                    path_dirs: vec![bin],
                },
                env_browser: None,
                env_chrome_path: None,
                version_timeout: Duration::from_secs(5),
            };
            Self { dir, ctx }
        }

        fn root(&self) -> PathBuf {
            self.dir.path().join("sys")
        }

        fn home(&self) -> PathBuf {
            self.dir.path().join("home")
        }

        fn bin(&self) -> PathBuf {
            self.dir.path().join("bin")
        }

        /// A working fake browser at `<root>/<absolute>`.
        fn install(&self, absolute: &str, product: &str) -> PathBuf {
            let path = self.root().join(absolute.trim_start_matches('/'));
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            let dir = path.parent().unwrap().to_path_buf();
            let made = script(
                &dir,
                path.file_name().unwrap().to_str().unwrap(),
                &format!("echo '{product} 131.0.1.2'"),
            );
            assert_eq!(made, path);
            path
        }

        fn install_managed(&self, target: InstallTarget, dir_name: &str) -> PathBuf {
            let dir = self.home().join(".auditmysite/browsers").join(dir_name);
            let binary =
                BrowserInstaller::binary_path_for(self.ctx.platform, &dir, target).unwrap();
            let parent = binary.parent().unwrap();
            std::fs::create_dir_all(parent).unwrap();
            script(
                parent,
                binary.file_name().unwrap().to_str().unwrap(),
                "echo 'Chrome 120.0.0.1'",
            );
            binary
        }

        fn resolve(&self, opts: &BrowserResolveOptions) -> Result<ResolvedBrowser> {
            resolve_in(&self.ctx, opts)
        }
    }

    fn auto() -> BrowserResolveOptions {
        BrowserResolveOptions::default()
    }

    fn explicit(path: &Path) -> BrowserResolveOptions {
        BrowserResolveOptions {
            browser_path: Some(path.to_string_lossy().into_owned()),
            ..Default::default()
        }
    }

    #[test]
    fn no_browser_is_chrome_not_found() {
        let m = Machine::new(LINUX_X64);
        assert!(matches!(
            m.resolve(&auto()),
            Err(AuditError::ChromeNotFound)
        ));
    }

    #[test]
    fn linux_x64_prefers_chrome_over_chromium_and_lists_both() {
        let m = Machine::new(LINUX_X64);
        m.install("/usr/bin/chromium", "Chromium");
        m.install("/usr/bin/google-chrome", "Google Chrome");
        let r = m.resolve(&auto()).unwrap();
        assert_eq!(r.browser.kind, BrowserKind::Chrome);
        assert_eq!(r.browser.source, BrowserSource::SystemPath);
        assert_eq!(r.candidates.len(), 2);
        assert!(!r.explicit);
        assert_eq!(r.launch_chain().len(), 2);
    }

    #[test]
    fn linux_arm64_resolves_a_distro_chromium() {
        let m = Machine::new(LINUX_ARM);
        m.install("/usr/bin/chromium", "Chromium");
        let r = m.resolve(&auto()).unwrap();
        assert_eq!(r.browser.kind, BrowserKind::Chromium);
    }

    #[test]
    fn macos_arm64_resolves_an_application_bundle() {
        let m = Machine::new(MAC_ARM);
        m.install(
            "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
            "Google Chrome",
        );
        let r = m.resolve(&auto()).unwrap();
        assert_eq!(r.browser.kind, BrowserKind::Chrome);
    }

    #[test]
    fn macos_intel_is_outside_the_platform_contract() {
        let m = Machine::new(MAC_INTEL);
        m.install(
            "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
            "Google Chrome",
        );
        let err = m.resolve(&auto()).unwrap_err();
        assert!(
            matches!(err, AuditError::UnsupportedBrowser { ref reason } if reason.contains("macOS on x86_64")),
            "{err}"
        );
    }

    #[test]
    fn windows_auto_selects_edge_and_ignores_an_installed_chrome() {
        let m = Machine::new(WIN_X64);
        let edge = m.dir.path().join("pf86/Microsoft/Edge/Application");
        let chrome = m.dir.path().join("pf/Google/Chrome/Application");
        std::fs::create_dir_all(&edge).unwrap();
        std::fs::create_dir_all(&chrome).unwrap();
        script(&edge, "msedge.exe", "echo 'Microsoft Edge 131.0.1.2'");
        script(&chrome, "chrome.exe", "echo 'Google Chrome 131.0.1.2'");
        let r = m.resolve(&auto()).unwrap();
        assert_eq!(r.browser.kind, BrowserKind::Edge);
        assert!(r
            .candidates
            .iter()
            .all(|c| c.browser.kind == BrowserKind::Edge));
    }

    #[test]
    fn windows_with_only_chrome_installed_finds_no_browser() {
        let m = Machine::new(WIN_X64);
        let chrome = m.dir.path().join("pf/Google/Chrome/Application");
        std::fs::create_dir_all(&chrome).unwrap();
        script(&chrome, "chrome.exe", "echo 'Google Chrome 131.0.1.2'");
        assert!(matches!(
            m.resolve(&auto()),
            Err(AuditError::ChromeNotFound)
        ));
    }

    #[test]
    fn windows_rejects_an_explicit_chrome_with_an_actionable_error() {
        let m = Machine::new(WIN_X64);
        let chrome = m.dir.path().join("pf/Google/Chrome/Application");
        std::fs::create_dir_all(&chrome).unwrap();
        let path = script(&chrome, "chrome.exe", "echo 'Google Chrome 131.0.1.2'");
        let err = m.resolve(&explicit(&path)).unwrap_err();
        let AuditError::UnsupportedBrowser { reason } = err else {
            panic!("expected UnsupportedBrowser, got {err}");
        };
        assert!(reason.contains("only Microsoft Edge"), "{reason}");
        assert!(reason.contains("msedge.exe"), "{reason}");
    }

    #[test]
    fn windows_rejects_chrome_from_the_env_var_and_the_preference_flag() {
        let mut m = Machine::new(WIN_X64);
        m.ctx.env_browser = Some(r"C:\Program Files\Google\Chrome\Application\chrome.exe".into());
        assert!(matches!(
            m.resolve(&auto()),
            Err(AuditError::UnsupportedBrowser { .. })
        ));
        m.ctx.env_browser = None;
        let opts = BrowserResolveOptions {
            browser_preference: Some("chrome".into()),
            ..Default::default()
        };
        assert!(matches!(
            m.resolve(&opts),
            Err(AuditError::UnsupportedBrowser { .. })
        ));
    }

    #[test]
    fn windows_accepts_an_explicit_edge() {
        let m = Machine::new(WIN_X64);
        let dir = m.dir.path().join("custom");
        std::fs::create_dir_all(&dir).unwrap();
        let path = script(&dir, "msedge.exe", "echo 'Microsoft Edge 131.0.1.2'");
        let r = m.resolve(&explicit(&path)).unwrap();
        assert!(r.explicit);
        assert_eq!(r.browser.source, BrowserSource::CliFlag);
    }

    #[test]
    fn an_explicit_path_wins_and_is_the_only_candidate() {
        let m = Machine::new(LINUX_X64);
        m.install("/usr/bin/google-chrome", "Google Chrome");
        let dir = m.dir.path().join("custom");
        std::fs::create_dir_all(&dir).unwrap();
        let path = script(&dir, "my-browser", "echo 'Chromium 130.0.0.1'");
        let r = m.resolve(&explicit(&path)).unwrap();
        assert_eq!(r.browser.path, path);
        assert!(r.explicit);
        assert_eq!(r.launch_chain().len(), 1, "no silent alternatives");
    }

    #[test]
    fn a_missing_explicit_path_fails_even_though_a_browser_is_installed() {
        let m = Machine::new(LINUX_X64);
        m.install("/usr/bin/google-chrome", "Google Chrome");
        let err = m
            .resolve(&explicit(Path::new("/nowhere/chrome")))
            .unwrap_err();
        assert!(matches!(err, AuditError::FileError { .. }), "{err}");
    }

    #[test]
    fn a_non_executable_explicit_path_fails_with_chmod_advice() {
        let m = Machine::new(LINUX_X64);
        m.install("/usr/bin/google-chrome", "Google Chrome");
        let path = m.dir.path().join("plain");
        std::fs::write(&path, "x").unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
        let err = m.resolve(&explicit(&path)).unwrap_err();
        assert!(
            matches!(err, AuditError::ChromeNotExecutable { .. }),
            "{err}"
        );
        assert!(err.to_string().contains("chmod +x"));
    }

    #[test]
    fn a_hanging_explicit_path_fails_within_the_bound() {
        let mut m = Machine::new(LINUX_X64);
        m.ctx.version_timeout = Duration::from_millis(300);
        let path = script(m.dir.path(), "hang", "exec sleep 30");
        let started = std::time::Instant::now();
        let err = m.resolve(&explicit(&path)).unwrap_err();
        assert!(started.elapsed() < Duration::from_secs(5));
        assert!(
            matches!(err, AuditError::BrowserLaunchFailed { ref reason } if reason.contains("no answer")),
            "{err}"
        );
    }

    #[test]
    fn explicit_fallback_is_only_taken_when_requested() {
        let m = Machine::new(LINUX_X64);
        m.install("/usr/bin/chromium", "Chromium");
        let missing = BrowserResolveOptions {
            browser_path: Some("/nowhere/chrome".into()),
            fallback_to_auto: true,
            ..Default::default()
        };
        let r = m.resolve(&missing).unwrap();
        assert_eq!(r.browser.kind, BrowserKind::Chromium);
        assert!(!r.explicit);
        // The failed explicit attempt stays in the diagnostics.
        assert!(!r.candidates[0].verdict.is_viable());
        assert_eq!(r.candidates[0].browser.source, BrowserSource::CliFlag);
    }

    #[test]
    fn an_unusable_auditmysite_browser_env_var_fails_instead_of_being_ignored() {
        let mut m = Machine::new(LINUX_X64);
        m.install("/usr/bin/google-chrome", "Google Chrome");
        m.ctx.env_browser = Some("/nowhere/chrome".into());
        assert!(matches!(
            m.resolve(&auto()),
            Err(AuditError::FileError { .. })
        ));
    }

    #[test]
    fn a_stale_legacy_chrome_path_is_ignored() {
        let mut m = Machine::new(LINUX_X64);
        m.install("/usr/bin/google-chrome", "Google Chrome");
        m.ctx.env_chrome_path = Some("/nowhere/chrome".into());
        let r = m.resolve(&auto()).unwrap();
        assert!(!r.explicit);
    }

    #[test]
    fn a_leading_tilde_in_an_explicit_path_is_expanded() {
        let m = Machine::new(LINUX_X64);
        std::fs::create_dir_all(m.home().join("bin")).unwrap();
        script(
            &m.home().join("bin"),
            "chrome",
            "echo 'Google Chrome 131.0.1.2'",
        );
        let opts = BrowserResolveOptions {
            browser_path: Some("~/bin/chrome".into()),
            ..Default::default()
        };
        let r = m.resolve(&opts).unwrap();
        assert_eq!(r.browser.path, m.home().join("bin/chrome"));
    }

    #[test]
    fn first_candidate_unusable_second_is_selected_and_the_first_is_documented() {
        let m = Machine::new(LINUX_X64);
        let chrome = m.install("/usr/bin/google-chrome", "Google Chrome");
        std::fs::set_permissions(&chrome, std::fs::Permissions::from_mode(0o644)).unwrap();
        m.install("/usr/bin/chromium", "Chromium");
        let r = m.resolve(&auto()).unwrap();
        assert_eq!(r.browser.kind, BrowserKind::Chromium);
        assert_eq!(r.launch_chain().len(), 1);
        let first = &r.candidates[0];
        assert_eq!(first.browser.kind, BrowserKind::Chrome);
        assert!(matches!(
            first.verdict,
            CandidateVerdict::Rejected {
                kind: RejectionKind::NotExecutable,
                ..
            }
        ));
    }

    #[test]
    fn when_every_candidate_is_unusable_the_error_names_each_one() {
        let m = Machine::new(LINUX_X64);
        let chrome = m.install("/usr/bin/google-chrome", "Google Chrome");
        let chromium = m.install("/usr/bin/chromium", "Chromium");
        for p in [&chrome, &chromium] {
            std::fs::set_permissions(p, std::fs::Permissions::from_mode(0o644)).unwrap();
        }
        let AuditError::BrowserLaunchFailed { reason } = m.resolve(&auto()).unwrap_err() else {
            panic!("expected BrowserLaunchFailed");
        };
        assert!(
            reason.contains("google-chrome") && reason.contains("chromium"),
            "{reason}"
        );
    }

    #[test]
    fn an_existing_browser_beats_a_managed_install() {
        let m = Machine::new(LINUX_X64);
        m.install_managed(InstallTarget::ChromeForTesting, "chrome-for-testing");
        m.install("/usr/bin/chromium", "Chromium");
        let r = m.resolve(&auto()).unwrap();
        assert_eq!(r.browser.source, BrowserSource::SystemPath);
        // The managed install stays visible as a later alternative.
        assert_eq!(
            r.candidates.last().unwrap().browser.source,
            BrowserSource::ManagedInstall
        );
        assert_eq!(r.launch_chain().len(), 2);
    }

    #[test]
    fn full_chrome_for_testing_comes_before_the_headless_shell() {
        let m = Machine::new(LINUX_X64);
        m.install_managed(InstallTarget::HeadlessShell, "headless-shell");
        m.install_managed(InstallTarget::ChromeForTesting, "chrome-for-testing");
        let r = m.resolve(&auto()).unwrap();
        assert_eq!(r.browser.kind, BrowserKind::ChromeForTesting);
        assert_eq!(r.candidates[1].browser.kind, BrowserKind::HeadlessShell);
    }

    #[test]
    fn the_headless_shell_is_used_only_when_no_full_browser_exists() {
        let m = Machine::new(LINUX_X64);
        m.install_managed(InstallTarget::HeadlessShell, "headless-shell");
        let r = m.resolve(&auto()).unwrap();
        assert_eq!(r.browser.kind, BrowserKind::HeadlessShell);
    }

    #[test]
    fn a_stale_managed_install_is_skipped_for_a_working_system_browser() {
        let m = Machine::new(LINUX_X64);
        let stale = m.home().join(".auditmysite/browsers/chrome-for-testing");
        std::fs::create_dir_all(&stale).unwrap();
        std::fs::write(stale.join("version.txt"), "120.0.0.0").unwrap();
        m.install("/usr/bin/chromium", "Chromium");
        let r = m.resolve(&auto()).unwrap();
        assert_eq!(r.browser.kind, BrowserKind::Chromium);
        let managed = r
            .candidates
            .iter()
            .find(|c| c.browser.source == BrowserSource::ManagedInstall)
            .unwrap();
        assert!(matches!(
            managed.verdict,
            CandidateVerdict::Rejected {
                kind: RejectionKind::StaleInstall,
                ..
            }
        ));
    }

    #[test]
    fn a_stale_managed_install_alone_fails_with_reinstall_advice() {
        let m = Machine::new(LINUX_X64);
        let stale = m.home().join(".auditmysite/browsers/chrome-for-testing");
        std::fs::create_dir_all(&stale).unwrap();
        let err = m.resolve(&auto()).unwrap_err();
        assert!(err.to_string().contains("browser install --force"), "{err}");
    }

    #[test]
    fn strict_mode_excludes_managed_installs() {
        let m = Machine::new(LINUX_X64);
        m.install_managed(InstallTarget::ChromeForTesting, "chrome-for-testing");
        let opts = BrowserResolveOptions {
            strict: true,
            ..Default::default()
        };
        assert!(matches!(m.resolve(&opts), Err(AuditError::ChromeNotFound)));
    }

    #[test]
    fn linux_arm64_never_offers_a_managed_install() {
        let m = Machine::new(LINUX_ARM);
        let dir = m.home().join(".auditmysite/browsers/chrome-for-testing");
        std::fs::create_dir_all(&dir).unwrap();
        m.install("/usr/bin/chromium", "Chromium");
        let r = m.resolve(&auto()).unwrap();
        assert_eq!(r.browser.kind, BrowserKind::Chromium);
        assert!(matches!(
            r.candidates.last().unwrap().verdict,
            CandidateVerdict::Rejected {
                kind: RejectionKind::Excluded,
                ..
            }
        ));
    }

    #[test]
    fn the_browser_preference_excludes_other_kinds_with_a_reason() {
        let m = Machine::new(LINUX_X64);
        m.install("/usr/bin/google-chrome", "Google Chrome");
        m.install("/usr/bin/chromium", "Chromium");
        let opts = BrowserResolveOptions {
            browser_preference: Some("chromium".into()),
            ..Default::default()
        };
        let r = m.resolve(&opts).unwrap();
        assert_eq!(r.browser.kind, BrowserKind::Chromium);
        assert!(matches!(
            r.candidates[0].verdict,
            CandidateVerdict::Rejected {
                kind: RejectionKind::Excluded,
                ..
            }
        ));
    }

    #[test]
    fn a_path_installed_user_browser_is_found_and_labelled() {
        let m = Machine::new(LINUX_X64);
        script(&m.bin(), "chromium", "echo 'Chromium 131.0.1.2'");
        let r = m.resolve(&auto()).unwrap();
        assert_eq!(r.browser.source, BrowserSource::PathSearch);
    }

    #[test]
    fn diagnosis_shows_selection_alternatives_and_reasons_without_failing() {
        let m = Machine::new(LINUX_X64);
        let chrome = m.install("/usr/bin/google-chrome", "Google Chrome");
        std::fs::set_permissions(&chrome, std::fs::Permissions::from_mode(0o644)).unwrap();
        m.install("/usr/bin/chromium", "Chromium");
        let d = diagnose_static_in(&m.ctx, &auto());
        assert_eq!(d.candidates.len(), 2);
        assert_eq!(d.selected().unwrap().browser.kind, BrowserKind::Chromium);
        assert!(d.problem.is_none());
        // Nothing resolvable is still a diagnosis, not an error.
        let empty = Machine::new(LINUX_X64);
        let d = diagnose_static_in(&empty.ctx, &auto());
        assert!(d.selected().is_none());
        // And so is an unsupported platform.
        let intel = Machine::new(MAC_INTEL);
        assert!(diagnose_static_in(&intel.ctx, &auto()).problem.is_some());
    }

    #[tokio::test]
    async fn probing_marks_a_browser_that_starts_but_never_serves_cdp_as_unusable() {
        let m = Machine::new(LINUX_X64);
        m.install("/usr/bin/google-chrome", "Google Chrome"); // answers --version, no DevTools
        let mut d = diagnose_static_in(&m.ctx, &auto());
        assert!(d.selected().is_some());
        probe_candidates(&mut d.candidates, false, Duration::from_secs(2)).await;
        assert!(d.selected().is_none());
        assert!(matches!(
            d.candidates[0].verdict,
            CandidateVerdict::Rejected {
                kind: RejectionKind::CdpFailed | RejectionKind::CdpTimedOut,
                ..
            }
        ));
    }
}
