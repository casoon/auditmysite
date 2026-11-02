//! Browser registry - platform-specific paths and search order
//!
//! Defines known browser locations for each platform in priority order. The
//! order is a contract: per browser kind (Chrome, Edge, Ungoogled Chromium,
//! Chromium) the system-wide install comes first, then the user-level
//! install, then the executables found on `PATH`.
//!
//! Everything here is a pure function of the [`Platform`] and an
//! [`EnvPaths`], so each platform's list can be tested on any host.

use std::path::{Path, PathBuf};

use super::platform::{Os, Platform};
use super::types::{BrowserKind, BrowserSource};

/// The places discovery looks in, taken from the process environment or
/// injected by a test.
#[derive(Debug, Clone, Default)]
pub(super) struct EnvPaths {
    /// Prefix for the absolute system locations (`/usr/bin/...`); `None` is
    /// the real file system root. Lets tests lay out a fake system.
    pub root: Option<PathBuf>,
    pub home: Option<PathBuf>,
    /// `%LOCALAPPDATA%` (Windows)
    pub local_app_data: Option<PathBuf>,
    /// `%ProgramFiles%` (Windows)
    pub program_files: Option<PathBuf>,
    /// `%ProgramFiles(x86)%` (Windows)
    pub program_files_x86: Option<PathBuf>,
    /// Directories of `PATH`, in order
    pub path_dirs: Vec<PathBuf>,
}

impl EnvPaths {
    pub(super) fn from_process() -> Self {
        let var = |name: &str| std::env::var_os(name).map(PathBuf::from);
        Self {
            root: None,
            home: dirs::home_dir(),
            local_app_data: var("LOCALAPPDATA"),
            program_files: var("ProgramFiles"),
            program_files_x86: var("ProgramFiles(x86)"),
            path_dirs: std::env::var_os("PATH")
                .map(|p| std::env::split_paths(&p).collect())
                .unwrap_or_default(),
        }
    }
}

impl EnvPaths {
    /// An absolute system location, re-rooted when `root` is set.
    pub(super) fn rooted(&self, absolute: &str) -> PathBuf {
        match &self.root {
            Some(root) => root.join(absolute.trim_start_matches('/')),
            None => PathBuf::from(absolute),
        }
    }
}

/// A location where a browser may be installed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RegistryEntry {
    pub kind: BrowserKind,
    pub path: PathBuf,
    pub source: BrowserSource,
}

/// Priority order of the browser kinds for automatic search.
pub(super) fn search_order(platform: Platform) -> &'static [BrowserKind] {
    if platform.is_edge_only() {
        return &[BrowserKind::Edge];
    }
    &[
        BrowserKind::Chrome,
        BrowserKind::Edge,
        BrowserKind::UngoogledChromium,
        BrowserKind::Chromium,
    ]
}

/// Executable names searched on `PATH` for a kind.
pub(super) fn path_names(platform: Platform, kind: BrowserKind) -> &'static [&'static str] {
    if platform.os == Os::Windows {
        return match kind {
            BrowserKind::Edge => &["msedge.exe"],
            _ => &[],
        };
    }
    match kind {
        BrowserKind::Chrome => &["google-chrome", "google-chrome-stable"],
        BrowserKind::Edge => &["microsoft-edge", "microsoft-edge-stable"],
        BrowserKind::UngoogledChromium => &["ungoogled-chromium"],
        BrowserKind::Chromium => &["chromium", "chromium-browser"],
        _ => &[],
    }
}

/// First existing file called `name` in `dirs` (what `which`/`where` finds).
pub(super) fn find_in_path(name: &str, dirs: &[PathBuf]) -> Option<PathBuf> {
    dirs.iter()
        .filter(|dir| !dir.as_os_str().is_empty())
        .map(|dir| dir.join(name))
        .find(|candidate| candidate.is_file())
}

/// Known install locations of `kind`, system-wide first, then user-level.
pub(super) fn install_locations(
    platform: Platform,
    kind: BrowserKind,
    env: &EnvPaths,
) -> Vec<RegistryEntry> {
    if !platform.allows_discovered(kind) {
        return Vec::new();
    }
    let system = |path: PathBuf| RegistryEntry {
        kind,
        path,
        source: BrowserSource::SystemPath,
    };
    let user = |path: PathBuf| RegistryEntry {
        kind,
        path,
        source: BrowserSource::UserInstall,
    };
    let mut entries = Vec::new();

    match platform.os {
        Os::MacOs => {
            let app = |root: &Path, bundle: &str, bin: &str| {
                root.join(format!("{bundle}.app"))
                    .join("Contents")
                    .join("MacOS")
                    .join(bin)
            };
            let (bundle, bin) = match kind {
                BrowserKind::Chrome => ("Google Chrome", "Google Chrome"),
                BrowserKind::Edge => ("Microsoft Edge", "Microsoft Edge"),
                BrowserKind::UngoogledChromium => ("Ungoogled Chromium", "Chromium"),
                _ => ("Chromium", "Chromium"),
            };
            entries.push(system(app(&env.rooted("/Applications"), bundle, bin)));
            if kind == BrowserKind::Chromium {
                entries.push(system(env.rooted("/opt/homebrew/bin/chromium")));
                entries.push(system(env.rooted("/usr/local/bin/chromium")));
            }
            if let Some(home) = &env.home {
                entries.push(user(app(&home.join("Applications"), bundle, bin)));
            }
        }
        Os::Linux => {
            let (system_paths, user_names): (&[&str], &[&str]) = match kind {
                BrowserKind::Chrome => (
                    &["/usr/bin/google-chrome", "/usr/bin/google-chrome-stable"],
                    &["google-chrome", "google-chrome-stable"],
                ),
                BrowserKind::Edge => (
                    &["/usr/bin/microsoft-edge", "/usr/bin/microsoft-edge-stable"],
                    &["microsoft-edge", "microsoft-edge-stable"],
                ),
                BrowserKind::UngoogledChromium => {
                    (&["/usr/bin/ungoogled-chromium"], &["ungoogled-chromium"])
                }
                _ => (
                    &[
                        "/usr/bin/chromium",
                        "/usr/bin/chromium-browser",
                        "/snap/bin/chromium",
                        "/var/lib/flatpak/exports/bin/org.chromium.Chromium",
                    ],
                    &["chromium", "chromium-browser"],
                ),
            };
            entries.extend(system_paths.iter().map(|p| system(env.rooted(p))));
            if let Some(home) = &env.home {
                entries.extend(
                    user_names
                        .iter()
                        .map(|n| user(home.join(".local/bin").join(n))),
                );
                if kind == BrowserKind::Chromium {
                    entries.push(user(
                        home.join(".local/share/flatpak/exports/bin/org.chromium.Chromium"),
                    ));
                }
            }
        }
        Os::Windows => {
            // Edge only (see `allows_discovered`).
            let edge = |root: PathBuf| {
                root.join("Microsoft")
                    .join("Edge")
                    .join("Application")
                    .join("msedge.exe")
            };
            let x86 = env
                .program_files_x86
                .clone()
                .unwrap_or_else(|| PathBuf::from(r"C:\Program Files (x86)"));
            let native = env
                .program_files
                .clone()
                .unwrap_or_else(|| PathBuf::from(r"C:\Program Files"));
            entries.push(system(edge(x86)));
            entries.push(system(edge(native)));
            if let Some(local) = &env.local_app_data {
                entries.push(user(edge(local.clone())));
            }
        }
        Os::Other => {}
    }

    entries
}

/// `~` and `~/...` in a user-given path expanded against `home`. Shells do
/// this for the command line, but not for environment variables or configs.
pub(super) fn expand_home(raw: &str, home: Option<&Path>) -> PathBuf {
    let Some(home) = home else {
        return PathBuf::from(raw);
    };
    if raw == "~" {
        return home.to_path_buf();
    }
    match raw.strip_prefix("~/").or_else(|| raw.strip_prefix("~\\")) {
        Some(rest) => home.join(rest),
        None => PathBuf::from(raw),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::browser::platform::Arch;

    const MAC_ARM: Platform = Platform::new(Os::MacOs, Arch::Aarch64);
    const LINUX_X64: Platform = Platform::new(Os::Linux, Arch::X86_64);
    const LINUX_ARM: Platform = Platform::new(Os::Linux, Arch::Aarch64);
    const WIN_X64: Platform = Platform::new(Os::Windows, Arch::X86_64);

    fn env() -> EnvPaths {
        EnvPaths {
            root: None,
            home: Some(PathBuf::from("/home/u")),
            local_app_data: Some(PathBuf::from("/appdata")),
            program_files: Some(PathBuf::from("/pf")),
            program_files_x86: Some(PathBuf::from("/pf86")),
            path_dirs: vec![],
        }
    }

    fn all(platform: Platform) -> Vec<RegistryEntry> {
        search_order(platform)
            .iter()
            .flat_map(|&k| install_locations(platform, k, &env()))
            .collect()
    }

    #[test]
    fn search_order_prefers_full_chrome_first_and_is_edge_only_on_windows() {
        assert_eq!(search_order(MAC_ARM)[0], BrowserKind::Chrome);
        assert_eq!(search_order(LINUX_X64).len(), 4);
        assert_eq!(search_order(WIN_X64), &[BrowserKind::Edge]);
    }

    #[test]
    fn macos_arm64_lists_system_then_user_applications() {
        let entries = install_locations(MAC_ARM, BrowserKind::Chrome, &env());
        assert_eq!(entries.len(), 2);
        assert_eq!(
            entries[0].path,
            PathBuf::from("/Applications/Google Chrome.app/Contents/MacOS/Google Chrome")
        );
        assert_eq!(entries[0].source, BrowserSource::SystemPath);
        assert_eq!(
            entries[1].path,
            PathBuf::from("/home/u/Applications/Google Chrome.app/Contents/MacOS/Google Chrome")
        );
        assert_eq!(entries[1].source, BrowserSource::UserInstall);
    }

    #[test]
    fn linux_x64_and_arm64_share_one_list_including_user_installs() {
        let x64 = all(LINUX_X64);
        assert_eq!(x64, all(LINUX_ARM));
        assert!(x64
            .iter()
            .any(|e| e.path == Path::new("/usr/bin/google-chrome")));
        assert!(x64.iter().any(|e| {
            e.path == Path::new("/home/u/.local/bin/chromium")
                && e.source == BrowserSource::UserInstall
        }));
        // Chrome precedes Edge precedes Chromium.
        let first = |kind| x64.iter().position(|e| e.kind == kind).unwrap();
        assert!(first(BrowserKind::Chrome) < first(BrowserKind::Edge));
        assert!(first(BrowserKind::Edge) < first(BrowserKind::Chromium));
    }

    #[test]
    fn windows_lists_edge_only_with_system_then_user_level_locations() {
        let entries = all(WIN_X64);
        assert!(entries.iter().all(|e| e.kind == BrowserKind::Edge));
        assert!(entries
            .iter()
            .all(|e| e.path.to_string_lossy().ends_with("msedge.exe")));
        let sources: Vec<_> = entries.iter().map(|e| e.source).collect();
        assert_eq!(
            sources,
            vec![
                BrowserSource::SystemPath,
                BrowserSource::SystemPath,
                BrowserSource::UserInstall
            ]
        );
        assert!(install_locations(WIN_X64, BrowserKind::Chrome, &env()).is_empty());
        assert!(path_names(WIN_X64, BrowserKind::Chrome).is_empty());
        assert_eq!(path_names(WIN_X64, BrowserKind::Edge), &["msedge.exe"]);
    }

    #[test]
    fn windows_without_program_files_variables_uses_the_default_locations() {
        let bare = EnvPaths::default();
        let entries = install_locations(WIN_X64, BrowserKind::Edge, &bare);
        assert!(entries[0]
            .path
            .to_string_lossy()
            .starts_with(r"C:\Program Files (x86)"));
        assert!(entries[1]
            .path
            .to_string_lossy()
            .starts_with(r"C:\Program Files"));
    }

    #[test]
    fn home_expansion_covers_tilde_forms_only() {
        let home = Path::new("/home/u");
        assert_eq!(expand_home("~", Some(home)), PathBuf::from("/home/u"));
        assert_eq!(
            expand_home("~/bin/chrome", Some(home)),
            PathBuf::from("/home/u/bin/chrome")
        );
        assert_eq!(
            expand_home("/abs/~/x", Some(home)),
            PathBuf::from("/abs/~/x")
        );
        assert_eq!(
            expand_home("~other/x", Some(home)),
            PathBuf::from("~other/x")
        );
        assert_eq!(expand_home("~/x", None), PathBuf::from("~/x"));
    }

    #[test]
    fn path_search_returns_the_first_directory_with_the_file() {
        let a = tempfile::tempdir().unwrap();
        let b = tempfile::tempdir().unwrap();
        std::fs::write(b.path().join("chromium"), "").unwrap();
        std::fs::write(a.path().join("other"), "").unwrap();
        let dirs = vec![PathBuf::new(), a.path().into(), b.path().into()];
        assert_eq!(
            find_in_path("chromium", &dirs),
            Some(b.path().join("chromium"))
        );
        assert_eq!(find_in_path("missing", &dirs), None);
    }
}
