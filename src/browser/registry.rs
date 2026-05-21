//! Browser registry - platform-specific paths and search order
//!
//! Defines known browser locations for each platform in priority order.

use super::types::BrowserKind;

/// An entry in the browser registry
pub struct RegistryEntry {
    pub kind: BrowserKind,
    pub path: &'static str,
}

/// Known browser paths for the current platform, in priority order
pub fn system_browser_paths() -> Vec<RegistryEntry> {
    let mut entries = Vec::new();

    #[cfg(target_os = "macos")]
    {
        entries.extend([
            RegistryEntry {
                kind: BrowserKind::Chrome,
                path: "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
            },
            RegistryEntry {
                kind: BrowserKind::Edge,
                path: "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge",
            },
            RegistryEntry {
                kind: BrowserKind::UngoogledChromium,
                path: "/Applications/Ungoogled Chromium.app/Contents/MacOS/Chromium",
            },
            RegistryEntry {
                kind: BrowserKind::Chromium,
                path: "/Applications/Chromium.app/Contents/MacOS/Chromium",
            },
            RegistryEntry {
                kind: BrowserKind::Chromium,
                path: "/opt/homebrew/bin/chromium",
            },
            RegistryEntry {
                kind: BrowserKind::Chromium,
                path: "/usr/local/bin/chromium",
            },
        ]);
    }

    #[cfg(target_os = "linux")]
    {
        entries.extend([
            RegistryEntry {
                kind: BrowserKind::Chrome,
                path: "/usr/bin/google-chrome",
            },
            RegistryEntry {
                kind: BrowserKind::Chrome,
                path: "/usr/bin/google-chrome-stable",
            },
            RegistryEntry {
                kind: BrowserKind::Edge,
                path: "/usr/bin/microsoft-edge",
            },
            RegistryEntry {
                kind: BrowserKind::Edge,
                path: "/usr/bin/microsoft-edge-stable",
            },
            RegistryEntry {
                kind: BrowserKind::UngoogledChromium,
                path: "/usr/bin/ungoogled-chromium",
            },
            RegistryEntry {
                kind: BrowserKind::Chromium,
                path: "/usr/bin/chromium",
            },
            RegistryEntry {
                kind: BrowserKind::Chromium,
                path: "/usr/bin/chromium-browser",
            },
            RegistryEntry {
                kind: BrowserKind::Chromium,
                path: "/snap/bin/chromium",
            },
            RegistryEntry {
                kind: BrowserKind::Chromium,
                path: "/var/lib/flatpak/exports/bin/org.chromium.Chromium",
            },
        ]);
    }

    #[cfg(target_os = "windows")]
    {
        entries.extend([
            RegistryEntry {
                kind: BrowserKind::Chrome,
                path: r"C:\Program Files\Google\Chrome\Application\chrome.exe",
            },
            RegistryEntry {
                kind: BrowserKind::Chrome,
                path: r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
            },
            RegistryEntry {
                kind: BrowserKind::Edge,
                path: r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
            },
            RegistryEntry {
                kind: BrowserKind::Edge,
                path: r"C:\Program Files\Microsoft\Edge\Application\msedge.exe",
            },
        ]);
    }

    entries
}

/// Search terms for `which`/`where` per BrowserKind
pub fn which_names(kind: BrowserKind) -> &'static [&'static str] {
    match kind {
        BrowserKind::Chrome => &["google-chrome", "google-chrome-stable"],
        BrowserKind::Edge => &["microsoft-edge", "microsoft-edge-stable"],
        BrowserKind::UngoogledChromium => &["ungoogled-chromium"],
        BrowserKind::Chromium => &["chromium", "chromium-browser"],
        _ => &[],
    }
}

/// Priority order for automatic browser search
pub fn search_order() -> &'static [BrowserKind] {
    &[
        BrowserKind::Chrome,
        BrowserKind::Edge,
        BrowserKind::UngoogledChromium,
        BrowserKind::Chromium,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_browser_paths_not_empty() {
        let paths = system_browser_paths();
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
    fn test_search_order_starts_with_chrome() {
        let order = search_order();
        assert_eq!(order[0], BrowserKind::Chrome);
    }

    #[test]
    fn test_which_names_chrome() {
        let names = which_names(BrowserKind::Chrome);
        assert!(names.contains(&"google-chrome"));
    }
}
