# Browser Architecture: Technical Elaboration

> Status note:
> Besides the current state, this document also contains architecture ideas and earlier proposals.
> Not every option described here is currently implemented in the CLI.
> For the actual user-facing state, `auditmysite --help`, the [README](../README.md) and [ARCHITECTURE.md](ARCHITECTURE.md) are authoritative.

## Status quo (current state)

- Browser resolution via `resolver` with `--browser-path`, `AUDITMYSITE_BROWSER`, system browsers and managed install
- Managed browser under `~/.auditmysite/browsers/`
- No silent auto-download; installation happens explicitly via `auditmysite browser install`
- `browser` subcommands and `doctor` are implemented
- `--chrome-path` is now only an alias for `--browser-path`
- Some flags described further below, such as `--browser`, `--fast` and `--strict`, are currently proposals, not productive CLI options

---

## 1. Target architecture

```
┌─────────────────────────────────────────────────────┐
│                    CLI Layer                         │
│  audit <url>  │  browser detect/install  │  doctor  │
└────────┬──────┴────────────┬─────────────┴──────────┘
         │                   │
┌────────▼───────────────────▼────────────────────────┐
│                Browser Resolution                    │
│  1. --browser-path / AUDITMYSITE_BROWSER            │
│  2. Registry: Chrome → Edge → Ungoogled → Chromium  │
│  3. Managed install (~/.auditmysite/browsers/)      │
│  4. Error with installation hints                   │
└────────┬────────────────────────────────────────────┘
         │
┌────────▼────────────────────────────────────────────┐
│                Browser Launcher                      │
│  Modes: Standard │ Strict │ Fast (headless-shell)   │
│  Flags, timeouts, port handling, cleanup            │
└────────┬────────────────────────────────────────────┘
         │
┌────────▼────────────────────────────────────────────┐
│              chromiumoxide / CDP                     │
│  AXTree, Computed Styles, Web Vitals, Navigation    │
└─────────────────────────────────────────────────────┘
```

**Core principles:**

1. **No silent download.** If no browser is found → error with clear instructions.
2. **Prefer the system browser.** Does the user have Chrome or Edge installed? Use it.
3. **Explicit installer.** `auditmysite browser install` as a deliberate action.
4. **Homebrew stays lightweight.** No browser dependency in the formula.

**Responsibilities:**

| Component | Task |
|---|---|
| `browser::registry` | Knows all supported browser types and their paths per platform |
| `browser::detection` | Finds installed browsers, checks executable + version |
| `browser::resolver` | Performs the prioritized selection (CLI → Env → Registry) |
| `browser::installer` | Downloads Chrome for Testing / headless-shell |
| `browser::launcher` | Starts the browser with the right flags for the mode |
| `cli::browser` | Subcommands: `detect`, `install`, `remove` |
| `cli::doctor` | System diagnostics: browser, network, permissions |

---

## 2. Browser strategy

### Supported browsers

| Browser | BrowserKind | CDP-compatible | Priority |
|---|---|---|---|
| Google Chrome | `Chrome` | Yes | 1 (default) |
| Microsoft Edge | `Edge` | Yes (same engine) | 2 |
| Ungoogled Chromium | `UngoogledChromium` | Yes | 3 |
| Chromium (system) | `Chromium` | Yes | 4 |
| Chrome for Testing (managed) | `ChromeForTesting` | Yes | 5 (explicitly installed) |
| chrome-headless-shell (managed) | `HeadlessShell` | Yes (limited) | — (only with `--fast`) |

**Assumption:** Brave, Opera and Vivaldi are technically possible (Chromium-based), but the support effort is not worth it. Anyone who uses Brave can set `--browser-path`.

### Detection order (default mode)

```
1. --browser-path /explicit/path          → Use exactly this browser
2. AUDITMYSITE_BROWSER=/env/path          → Env variable
3. CHROME_PATH=/legacy/path               → Backward compatibility
4. Google Chrome (system)                 → Preferred, because widespread and signed
5. Microsoft Edge (system)                → Often preinstalled on macOS/Windows
6. Ungoogled Chromium (system/Brew)       → Privacy-conscious alternative
7. Chromium (system/Brew/Snap)            → Generic Chromium
8. Chrome for Testing (~/.auditmysite/)   → Only if installed via `browser install`
9. → Error: ChromeNotFound                → With concrete installation hints
```

### Modes

| Mode | Flag | Behavior |
|---|---|---|
| **Default** | (none) | Search system browser → managed browser → error |
| **Strict** | `--strict` | System browser only, no fallback to managed |
| **Fast** | `--fast` | Prefers `chrome-headless-shell`, then the standard chain |

**Strict mode:** For CI/CD pipelines where the browser should be controlled explicitly. No automatic fallback to downloads.

**Fast mode:** `chrome-headless-shell` is smaller (~50 MB vs ~300 MB) and starts faster. Limitation: no complete AXTree — check whether that is sufficient for the use case. **Assumption:** If the AXTree is not available with headless-shell, a fallback to standard Chrome must happen, with a warning.

### CLI flags and env variables

| Flag/Env | Type | Default | Description |
|---|---|---|---|
| `--browser-path <PATH>` | CLI | — | Explicit path to the browser binary |
| `--browser <KIND>` | CLI | `auto` | Force browser type: `chrome`, `edge`, `chromium`, `auto` |
| `--fast` | CLI | false | Prefers headless-shell for speed |
| `--strict` | CLI | false | System browser only, no managed fallback |
| `AUDITMYSITE_BROWSER` | Env | — | Path to the browser binary |
| `CHROME_PATH` | Env | — | Legacy compatibility (being deprecated in favor of `AUDITMYSITE_BROWSER`) |

---

## 3. CLI design

### Subcommand structure

```
auditmysite <URL>                              # Single audit (main command stays)
auditmysite --sitemap <URL>                    # Batch via sitemap
auditmysite --url-file <FILE>                  # Batch via file

auditmysite browser detect                     # Shows all found browsers
auditmysite browser install                    # Installs Chrome for Testing
auditmysite browser install --headless-shell   # Installs chrome-headless-shell
auditmysite browser remove                     # Removes managed browser
auditmysite browser path                       # Prints path of the active browser

auditmysite doctor                             # System diagnostics
```

### Example invocations

```bash
# Default: use system Chrome
auditmysite https://example.com --full --format pdf --output reports/audit.pdf

# Explicit browser
auditmysite https://example.com --browser-path /usr/bin/chromium-browser

# Force Edge
auditmysite https://example.com --browser edge

# Fast mode (headless-shell, if installed)
auditmysite https://example.com --fast

# Strict mode (system browser only, no fallback)
auditmysite https://example.com --strict

# Detect browsers (diagnostics)
auditmysite browser detect
# Output:
#   ✓ Google Chrome 131.0.6778.108  /Applications/Google Chrome.app/.../Google Chrome
#   ✓ Microsoft Edge 131.0.2903.86  /Applications/Microsoft Edge.app/.../Microsoft Edge
#   ✗ Ungoogled Chromium            not found
#   ✗ Chromium                      not found
#   ✗ Chrome for Testing            not installed (~/.auditmysite/browsers/)
#   Active: Google Chrome (auto-detected)

# Install browser
auditmysite browser install
# Output:
#   Downloading Chrome for Testing v131.0.6778.108...
#   Platform: macOS ARM64
#   Destination: ~/.auditmysite/browsers/chrome-for-testing/
#   Progress: [████████████████████] 100% (142 MB)
#   ✓ Installed successfully

# System diagnostics
auditmysite doctor
# Output:
#   Browser:     ✓ Google Chrome 131.0.6778.108
#   Permissions: ✓ Executable
#   Network:     ✓ Can reach example.com
#   Firewall:    ✓ Loopback connections allowed
#   Disk:        ✓ 2.1 GB free in ~/.auditmysite/
#   Config:      ✓ auditmysite.toml found
```

### Error case: no browser found

```
Error: No compatible browser found.

auditmysite requires a Chromium-based browser (Chrome, Edge, or Chromium).

Options:
  1. Install Google Chrome:
       macOS:   brew install --cask google-chrome
       Linux:   sudo apt install google-chrome-stable
       Windows: https://www.google.com/chrome/

  2. Install a managed browser for auditmysite:
       auditmysite browser install

  3. Specify an existing browser:
       auditmysite --browser-path /path/to/chrome <url>
       AUDITMYSITE_BROWSER=/path/to/chrome auditmysite <url>

Run 'auditmysite doctor' for full system diagnostics.
```

---

## 4. Rust module structure

```
src/browser/
├── mod.rs              # Public exports
├── detection.rs        # Finds browsers on the system (paths, which, validation)
├── registry.rs         # Defines BrowserKind, paths per platform, search order
├── resolver.rs         # Orchestrates: CLI → Env → Registry → Managed → Error
├── installer.rs        # Downloads: Chrome for Testing, headless-shell
├── launcher.rs         # Starts browser with mode-specific flags
├── manager.rs          # Browser lifecycle (existing, uses resolver + launcher)
├── pool.rs             # Concurrent page management (existing)
└── types.rs            # Shared types (BrowserKind, DetectedBrowser, LaunchConfig, etc.)

src/cli/
├── mod.rs
├── args.rs             # Main args + new flags (--browser, --browser-path, --fast, --strict)
├── browser.rs          # NEW: subcommands detect/install/remove/path
├── doctor.rs           # NEW: system diagnostics
└── config.rs           # Config file support

src/
├── doctor.rs           # NEW: doctor logic (browser, network, permissions)
└── ...
```

**Responsibility per module:**

| Module | Responsibility |
|---|---|
| `types.rs` | All enums and structs: `BrowserKind`, `DetectedBrowser`, `LaunchConfig`, `BrowserMode` |
| `registry.rs` | Static data: which browsers exist, which paths per OS, search order |
| `detection.rs` | Runtime detection: check paths, call `which`, read version, check capabilities |
| `resolver.rs` | Decision logic: takes CLI options, env vars and detection results, returns a `ResolvedBrowser` |
| `installer.rs` | Download + extraction of Chrome for Testing / headless-shell to `~/.auditmysite/browsers/` |
| `launcher.rs` | Builds `BrowserConfig`, picks flags depending on the mode, starts via chromiumoxide |
| `manager.rs` | High-level API: `BrowserManager::new()` uses resolver → launcher, manages the lifecycle |
| `pool.rs` | Concurrent page management (unchanged) |

---

## 5. State model / data structures

```rust
// ── types.rs ──────────────────────────────────────────────────

/// Which browser type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BrowserKind {
    Chrome,
    Edge,
    UngoogledChromium,
    Chromium,
    ChromeForTesting,  // Managed install
    HeadlessShell,     // Managed install, fast-mode only
    Custom,            // --browser-path
}

impl BrowserKind {
    /// Human-readable name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Chrome => "Google Chrome",
            Self::Edge => "Microsoft Edge",
            Self::UngoogledChromium => "Ungoogled Chromium",
            Self::Chromium => "Chromium",
            Self::ChromeForTesting => "Chrome for Testing",
            Self::HeadlessShell => "Chrome Headless Shell",
            Self::Custom => "Custom Browser",
        }
    }

    /// Whether this is a managed (self-installed) browser
    pub fn is_managed(&self) -> bool {
        matches!(self, Self::ChromeForTesting | Self::HeadlessShell)
    }
}

/// How the browser was found
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserSource {
    /// Explicitly via --browser-path
    CliFlag,
    /// Via AUDITMYSITE_BROWSER or CHROME_PATH
    EnvVar,
    /// Found in known system paths
    SystemPath,
    /// Found in PATH via `which`/`where`
    PathSearch,
    /// Self-installed under ~/.auditmysite/browsers/
    ManagedInstall,
}

/// Run mode for the browser
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserMode {
    /// Normal headless Chrome with all features
    Standard,
    /// System browser only, no fallback
    Strict,
    /// Prefers headless-shell for speed
    Fast,
}

/// A found browser
#[derive(Debug, Clone)]
pub struct DetectedBrowser {
    pub kind: BrowserKind,
    pub path: PathBuf,
    pub version: Option<String>,
    pub source: BrowserSource,
}

/// Result of the browser resolution
#[derive(Debug, Clone)]
pub struct ResolvedBrowser {
    /// The selected browser
    pub browser: DetectedBrowser,
    /// Which mode is active
    pub mode: BrowserMode,
    /// All found candidates (for `browser detect`)
    pub all_candidates: Vec<DetectedBrowser>,
}

/// Configuration for the browser launch
#[derive(Debug, Clone)]
pub struct LaunchConfig {
    pub browser: DetectedBrowser,
    pub mode: BrowserMode,
    pub headless: bool,
    pub no_sandbox: bool,
    pub disable_gpu: bool,
    pub disable_images: bool,
    pub window_size: (u32, u32),
    pub timeout_secs: u64,
    pub extra_args: Vec<String>,
}

/// What should be installed
#[derive(Debug, Clone, Copy)]
pub enum InstallTarget {
    /// Chrome for Testing (complete)
    ChromeForTesting,
    /// chrome-headless-shell (minimal, fast)
    HeadlessShell,
}
```

---

## 6. Detection logic

### Step by step

```
resolve_browser(cli_args) → Result<ResolvedBrowser>
│
├─ 1. CLI flag: --browser-path set?
│     → Validate path, check version, BrowserKind::Custom
│     → Error if path does not exist or is not executable
│
├─ 2. Env var: AUDITMYSITE_BROWSER set?
│     → Validate path, check version
│     → Only log an error if path is invalid, continue with next step
│
├─ 3. Env var: CHROME_PATH set? (legacy)
│     → Like 2., but with deprecation warning
│
├─ 4. --browser flag set? (e.g. --browser edge)
│     → Search only the specific BrowserKind
│     → Error if this type is not found
│
├─ 5. System scan (registry order):
│     │
│     ├─ Chrome: known paths → which google-chrome
│     ├─ Edge: known paths → which microsoft-edge
│     ├─ Ungoogled Chromium: Brew paths → which
│     └─ Chromium: known paths → Brew → Snap → which
│     │
│     → Take the first hit (unless --strict)
│
├─ 6. Check managed install (unless --strict):
│     → ~/.auditmysite/browsers/chrome-for-testing/ exists?
│     → ~/.auditmysite/browsers/headless-shell/ exists? (only with --fast)
│
└─ 7. Nothing found → ChromeNotFound error
```

### Platform-specific paths

#### macOS

```rust
fn macos_paths() -> Vec<(BrowserKind, &'static str)> {
    vec![
        // Chrome
        (BrowserKind::Chrome,
         "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"),
        // Edge
        (BrowserKind::Edge,
         "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge"),
        // Ungoogled Chromium (Homebrew cask: eloston-chromium)
        (BrowserKind::UngoogledChromium,
         "/Applications/Chromium.app/Contents/MacOS/Chromium"),  // Caution: collides with regular Chromium
        // Chromium (Homebrew)
        (BrowserKind::Chromium,
         "/opt/homebrew/bin/chromium"),
        (BrowserKind::Chromium,
         "/usr/local/bin/chromium"),
        // Chrome Canary (low priority, unstable)
        (BrowserKind::Chrome,
         "/Applications/Google Chrome Canary.app/Contents/MacOS/Google Chrome Canary"),
    ]
}
```

**Problem: Ungoogled Chromium vs. Chromium on macOS**

Both install as `/Applications/Chromium.app`. They can be told apart via:
- `Chromium --version` → "Chromium 131.0.6778.108 Ungoogled" vs. "Chromium 131.0.6778.108"
- Checking `brew list --cask`: `eloston-chromium` vs. `chromium`
- In practice: both work identically with CDP, the distinction is cosmetic.

**Recommendation:** Treat both as `BrowserKind::Chromium`, unless the user has explicitly set `--browser ungoogled`. On macOS, detecting `/Applications/Chromium.app` is enough.

#### Linux

```rust
fn linux_paths() -> Vec<(BrowserKind, &'static str)> {
    vec![
        // Chrome
        (BrowserKind::Chrome, "/usr/bin/google-chrome"),
        (BrowserKind::Chrome, "/usr/bin/google-chrome-stable"),
        // Edge
        (BrowserKind::Edge, "/usr/bin/microsoft-edge"),
        (BrowserKind::Edge, "/usr/bin/microsoft-edge-stable"),
        // Chromium
        (BrowserKind::Chromium, "/usr/bin/chromium"),
        (BrowserKind::Chromium, "/usr/bin/chromium-browser"),
        (BrowserKind::Chromium, "/snap/bin/chromium"),
        (BrowserKind::Chromium, "/var/lib/flatpak/exports/bin/org.chromium.Chromium"),
    ]
}
```

#### Windows

```rust
fn windows_paths() -> Vec<(BrowserKind, &'static str)> {
    vec![
        // Chrome
        (BrowserKind::Chrome,
         r"C:\Program Files\Google\Chrome\Application\chrome.exe"),
        (BrowserKind::Chrome,
         r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe"),
        // Edge (always present on Windows 10+)
        (BrowserKind::Edge,
         r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe"),
        (BrowserKind::Edge,
         r"C:\Program Files\Microsoft\Edge\Application\msedge.exe"),
    ]
}
```

### Validating a found binary

```rust
fn validate_browser(path: &Path) -> Result<DetectedBrowser> {
    // 1. Does the file exist?
    if !path.exists() {
        return Err(NotFound);
    }

    // 2. Is it executable? (Unix)
    #[cfg(unix)]
    check_executable_permission(path)?;

    // 3. Read version
    let version = run_version_check(path)?;  // --version

    // 4. Minimum version check
    //    chromiumoxide 0.8 needs Chrome >= 64 (CDP protocol)
    //    In practice: Chrome >= 90 recommended for a stable AXTree API
    if let Some(ref v) = version {
        let major = parse_major_version(v);
        if major < 90 {
            warn!("Chrome {} is very old, recommend >= 90", v);
        }
    }

    // 5. Optional: CDP capability check
    //    Quick test: `chrome --headless --dump-dom about:blank`
    //    If that works, CDP is available.
    //    → Only in `doctor`, not on every start.

    Ok(DetectedBrowser { ... })
}
```

### which search terms per BrowserKind

```rust
fn which_names(kind: BrowserKind) -> &'static [&'static str] {
    match kind {
        BrowserKind::Chrome => &["google-chrome", "google-chrome-stable", "chrome"],
        BrowserKind::Edge => &["microsoft-edge", "microsoft-edge-stable"],
        BrowserKind::Chromium => &["chromium", "chromium-browser"],
        _ => &[],
    }
}
```

---

## 7. Installation strategy

### `auditmysite browser install`

```
~/.auditmysite/
├── browsers/
│   ├── chrome-for-testing/          # Full Chrome
│   │   ├── version.txt              # "131.0.6778.108"
│   │   └── chrome-mac-arm64/        # Platform-specific
│   │       └── Google Chrome for Testing.app/...
│   └── headless-shell/              # Minimal (only with --headless-shell)
│       ├── version.txt
│       └── chrome-headless-shell
└── chromium/                         # LEGACY (delete in phase 5)
```

### Download sources

Chrome for Testing has a stable JSON API:

```
https://googlechromelabs.github.io/chrome-for-testing/known-good-versions-with-downloads.json
```

**Recommended approach:**

1. Fetch the latest stable version from the API (do not hardcode)
2. Assemble the download URL for platform + architecture
3. Download, extract, store the version in `version.txt`

```rust
/// Installer logic
impl BrowserInstaller {
    /// Install Chrome for Testing
    pub async fn install(target: InstallTarget) -> Result<InstalledBrowser> {
        // 1. Determine the current stable version
        let version = Self::fetch_latest_stable_version().await?;

        // 2. Check whether already installed
        let install_dir = Self::install_dir(target);
        if let Some(existing) = Self::check_existing(&install_dir, &version) {
            info!("Already installed: {} v{}", target.name(), version);
            return Ok(existing);
        }

        // 3. Assemble the download URL
        let url = Self::download_url(target, &version)?;

        // 4. Download with progress
        info!("Downloading {} v{}...", target.name(), version);
        let archive = Self::download_with_progress(&url).await?;

        // 5. Extract
        Self::extract(&archive, &install_dir)?;

        // 6. Write version file
        fs::write(install_dir.join("version.txt"), &version)?;

        // 7. Return binary path
        let binary_path = Self::binary_path(target, &install_dir);
        Ok(InstalledBrowser { path: binary_path, version })
    }

    /// Fetch stable version from the API
    async fn fetch_latest_stable_version() -> Result<String> {
        // Primary: Chrome for Testing API
        let url = "https://googlechromelabs.github.io/chrome-for-testing/last-known-good-versions.json";
        let resp: serde_json::Value = reqwest::get(url).await?.json().await?;
        let version = resp["channels"]["Stable"]["version"]
            .as_str()
            .ok_or_else(|| anyhow!("Could not parse stable version"))?;
        Ok(version.to_string())
    }
}
```

### Version pinning

**Recommendation:** No pinning to an exact version. Instead:

- `browser install` always fetches the current stable
- The installed version is stored in `version.txt`
- `browser install --update` updates to the latest stable
- Anyone who needs a specific version: `browser install --version 131.0.6778.108`
- **Compatibility guarantee:** Chrome for Testing Stable is backward compatible with CDP

### Integrity check

Chrome for Testing offers no official checksums. Pragmatic approach:

1. Download over HTTPS (TLS integrity)
2. After extraction: check that the binary exists
3. Call `--version` and parse it
4. If everything works → valid

### Error cases

| Error | Reaction |
|---|---|
| No internet | Clear message, hint to `--browser-path` |
| Download error | 2 retries with backoff, then error |
| Extraction failed | Delete cache dir, report error |
| No space | Check disk size, warn at < 500 MB |
| Binary not runnable | `--version` check fails → error + hint |

---

## 8. Launch Strategy

### Chrome flags by mode

```rust
fn build_launch_args(config: &LaunchConfig) -> Vec<String> {
    let mut args = vec![
        // Always
        "--headless=new".into(),          // New headless mode (Chrome >= 112)
        "--no-first-run".into(),
        "--no-default-browser-check".into(),
        "--disable-extensions".into(),
        "--disable-background-networking".into(),
        "--disable-sync".into(),
        "--disable-translate".into(),
        "--disable-features=TranslateUI".into(),
        "--metrics-recording-only".into(),
        "--mute-audio".into(),
        "--disable-infobars".into(),
        "--disable-popup-blocking".into(),
        "--disable-background-timer-throttling".into(),
        "--disable-backgrounding-occluded-windows".into(),
        "--disable-renderer-backgrounding".into(),
        format!("--window-size={},{}", config.window_size.0, config.window_size.1),
    ];

    // macOS: avoid the firewall problem
    if cfg!(target_os = "macos") {
        // Loopback connection instead of network
        args.push("--remote-debugging-address=127.0.0.1".into());
        // No mDNS/Bonjour → no firewall prompt
        args.push("--disable-features=NetworkService".into());
    }

    // Mode-specific
    match config.mode {
        BrowserMode::Standard => {
            args.push("--disable-gpu".into());
        }
        BrowserMode::Strict => {
            args.push("--disable-gpu".into());
            // No extra features
        }
        BrowserMode::Fast => {
            args.push("--disable-gpu".into());
            args.push("--blink-settings=imagesEnabled=false".into());
            args.push("--disable-javascript".into());  // Only if JS is not needed
            // CAUTION: JS is needed for Web Vitals!
            // → disable JS only if --no-vitals is set
        }
    }

    // Sandbox
    if config.no_sandbox {
        args.push("--no-sandbox".into());
        args.push("--disable-setuid-sandbox".into());
        args.push("--disable-dev-shm-usage".into());
    }

    args
}
```

**Note on `--headless=new` vs `--headless`:**
- `--headless` (old): Chrome < 112, outdated headless mode
- `--headless=new`: Chrome >= 112, uses the real browser renderer
- **Recommendation:** `--headless=new` for Chrome >= 112, fall back to `--headless` for older versions. Version check in detection.

### Port handling

```rust
/// Find a free port for CDP
fn find_free_port() -> u16 {
    // Briefly bind to port 0, the OS assigns a free port
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.local_addr().unwrap().port()
}
```

chromiumoxide does this internally, but if we want to control the port ourselves (e.g. for `--remote-debugging-port`), we need this.

### Process lifecycle and zombie avoidance

```rust
impl BrowserManager {
    pub async fn close(self) -> Result<()> {
        // 1. Close all pages
        if let Ok(pages) = self.browser.pages().await {
            for page in pages {
                page.close().await.ok();
            }
        }

        // 2. Shut down the browser gracefully
        //    chromiumoxide sends Browser.close over CDP
        drop(self.browser);

        // 3. Timeout: if the browser is still running after 5s, SIGTERM
        //    chromiumoxide handles this via Drop, but we
        //    should make sure no zombie is left behind

        // 4. Abort the handler task
        if let Some(handle) = self.handler.lock().await.take() {
            handle.abort();
        }

        Ok(())
    }
}

// Additionally: Ctrl+C handler in main.rs
// Makes sure the browser is cleaned up on SIGINT as well
```

### Logging

```rust
// At startup
info!("Browser: {} v{} ({})",
    resolved.browser.kind.display_name(),
    resolved.browser.version.as_deref().unwrap_or("unknown"),
    resolved.browser.source);
info!("Mode: {:?}", resolved.mode);
debug!("Launch args: {:?}", launch_args);
debug!("CDP port: {}", port);

// On problems
warn!("Browser process exited unexpectedly");
error!("CDP connection lost: {}", err);
```

---

## 9. Homebrew Strategy

### Formula structure

```ruby
class Auditmysite < Formula
  desc "WCAG 2.1 Accessibility Checker with full audit reporting"
  homepage "https://github.com/casoon/auditmysite"
  url "https://github.com/casoon/auditmysite/archive/refs/tags/v0.4.5.tar.gz"
  sha256 "..."
  license "LGPL-3.0-or-later"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  def caveats
    <<~EOS
      auditmysite requires a Chromium-based browser (Chrome, Edge, or Chromium).

      If you don't have one installed:
        brew install --cask google-chrome

      Or install a managed browser:
        auditmysite browser install

      Run 'auditmysite doctor' to verify your setup.
    EOS
  end

  test do
    # Only check that the binary runs, not the browser
    assert_match "auditmysite", shell_output("#{bin}/auditmysite --version")
  end
end
```

### Why the browser is NOT a hard dependency

1. **Many users already have Chrome.** `depends_on "google-chrome"` would install it again or cause conflicts.
2. **Cask dependencies are fragile.** `depends_on cask: "google-chrome"` does not work reliably in Homebrew.
3. **Edge is often preinstalled on macOS.** Why force Chrome when Edge is already there?
4. **CI/CD systems** often have their own Chrome installations.
5. **The user should be in control** of which browser they use.

### What should happen on `brew install auditmysite`

1. The Rust compiler builds the binary
2. The binary is installed to `/opt/homebrew/bin/auditmysite`
3. `caveats` shows the note about the browser requirement
4. **No browser download, no auto-install, no network access at runtime**

### What should NOT happen

- No automatic Chrome download on the first `brew install`
- No Homebrew dependency on a browser cask
- No post-install script that makes network accesses
- No aborting the installation when no browser is present

### Error message after installation (first run without a browser)

When the user runs `auditmysite https://example.com` and there is no browser:

```
Error: No compatible browser found.

You installed auditmysite via Homebrew — great!
Now you need a Chromium-based browser. Options:

  brew install --cask google-chrome     # Recommended
  auditmysite browser install           # Self-contained download

Run 'auditmysite doctor' to check your setup.
```

---

## 10. macOS-Specific Problems

### Problem 1: Firewall popups

**Cause:** Chrome opens a listening socket for CDP. The macOS Application Firewall asks "Do you want to allow incoming connections?"

**Solution:**
```
--remote-debugging-address=127.0.0.1
```
Binds to loopback only. The firewall only prompts for non-loopback connections. chromiumoxide uses this by default, but we set it explicitly.

**Additionally:**
```
--disable-features=MediaRouter
```
Prevents mDNS discovery (Chromecast etc.), which can also trigger firewall popups.

### Problem 2: Gatekeeper warnings

**Cause:** Binaries downloaded by ourselves have no code signing and get the `com.apple.quarantine` extended attribute.

**Solution for `browser install`:**
```rust
#[cfg(target_os = "macos")]
fn remove_quarantine(path: &Path) -> Result<()> {
    // Remove the quarantine attribute
    Command::new("xattr")
        .args(["-dr", "com.apple.quarantine"])
        .arg(path)
        .output()
        .ok();
    Ok(())
}
```

After extraction, remove the quarantine attribute on the entire app bundle. This is safe because we downloaded the file ourselves from a known Google URL.

### Problem 3: Unsigned browser binaries

**Chrome for Testing** is signed by Google (it has a valid code-signing certificate). This is not a problem.

**Self-compiled Chromium** (e.g. via `brew install chromium`) is NOT signed and can cause Gatekeeper problems.

**Recommendation:**
- System Chrome and system Edge are always signed → prefer them
- Chrome for Testing is signed → safe as a managed install
- Unsigned Chromium → warn, but allow

### Problem 4: Differences between installed Chrome and a download

| Aspect | System Chrome | Chrome for Testing |
|---|---|---|
| Code signing | Yes (Apple notarized) | Yes (signed by Google) |
| Firewall | Already allowed | New rule needed |
| Updates | Automatic | Manual (`browser install --update`) |
| Path | `/Applications/Google Chrome.app/...` | `~/.auditmysite/browsers/...` |
| Gatekeeper | No problem | Quarantine removal needed |
| User profile | Own profile | No profile (headless) |

**Recommendation:** System Chrome is **always** the better choice on macOS. The managed install is a fallback for systems without Chrome.

### Problem 5: SingletonLock

**Cause:** Chrome creates a `SingletonLock` in the profile directory. If the process is terminated uncleanly, the lock stays behind and prevents the next start.

**Current solution:** The user has to run `rm` manually.

**Better solution:**
```rust
fn cleanup_stale_locks(profile_dir: &Path) {
    let lock_file = profile_dir.join("SingletonLock");
    if lock_file.exists() {
        // Check whether the process is still running
        // SingletonLock contains hostname-PID
        if let Ok(content) = fs::read_to_string(&lock_file) {
            // Format: "hostname-PID"
            if let Some(pid_str) = content.split('-').last() {
                if let Ok(pid) = pid_str.parse::<u32>() {
                    // Check whether the PID is still alive
                    if !process_exists(pid) {
                        warn!("Removing stale SingletonLock (PID {} no longer running)", pid);
                        fs::remove_file(&lock_file).ok();
                    }
                }
            }
        }
    }
}
```

Even better: **a separate temporary profile directory** per audit run:
```rust
let temp_profile = tempfile::tempdir()?;
args.push(format!("--user-data-dir={}", temp_profile.path().display()));
```

This avoids the SingletonLock problem entirely and prevents conflicts with the user's running Chrome instances.

**Assumption:** chromiumoxide already sets a temporary profile directory. Check whether that is the case — if so, the SingletonLock problem is a bug in chromiumoxide. The current error message suggests that the fixed directory `/tmp/chromiumoxide-runner/` is used, which collides when there are several parallel runs.

---

## 11. Migration Plan

### Phase 1: Detection refactor

**Goal:** New browser detection with registry pattern, Edge support, better error messages.

**Changes:**
- Create `browser/types.rs` (BrowserKind, DetectedBrowser, BrowserSource, etc.)
- Create `browser/registry.rs` (paths per platform and BrowserKind)
- Rewrite `browser/detection.rs` (uses the registry, finds Edge + Ungoogled Chromium)
- Create `browser/resolver.rs` (prioritised selection)
- `cli/args.rs`: `--browser-path` as an alias for `--chrome-path` (accept both)
- `cli/args.rs`: `--browser <kind>` flag
- Introduce env var `AUDITMYSITE_BROWSER`, keep accepting `CHROME_PATH`
- Improve the error message for ChromeNotFound

**Risks:**
- Detection might find different browsers than before on some systems → `--browser chrome` as an escape hatch
- Edge compatibility with chromiumoxide: should work (same CDP API), but test it

**Tests:**
- Unit tests for the registry (paths per platform)
- Unit tests for the resolver (priority order)
- Integration test: `auditmysite browser detect` on CI
- Manual: test on macOS with Chrome + Edge

### Phase 2: Explicit installer

**Goal:** `auditmysite browser install` as an explicit command. No more silent download.

**Changes:**
- Create `cli/browser.rs` (subcommands: detect, install, remove, path)
- Rewrite `browser/installer.rs`:
  - Version from API instead of hardcoded
  - Target directory: `~/.auditmysite/browsers/chrome-for-testing/`
  - Progress bar with `indicatif`
  - Quarantine handling on macOS
- `browser/manager.rs`: remove the auto-download fallback
- `browser/manager.rs`: if no browser → `ChromeNotFound` error (no download)

**Risks:**
- Breaking change: users who rely on the auto-download now get an error
- Mitigation: clear error message with a hint to `auditmysite browser install`

**Tests:**
- Test `browser install` in CI (download + extraction + runnability)
- Test `browser remove`
- Test: audit without a browser → error with the correct message
- Test: audit after `browser install` → works

### Phase 3: CLI doctor

**Goal:** `auditmysite doctor` for system diagnosis.

**Changes:**
- Create `cli/doctor.rs`
- Create `doctor.rs` in the root (logic)
- Checks: browser present, executable, version, CDP-capable, network, disk space

**Risks:** Low. Purely additive feature.

**Tests:**
- Doctor on CI: all checks green with Chrome installed
- Doctor without a browser: browser check red, rest green

### Phase 4: Headless-shell mode

**Goal:** `--fast` mode with `chrome-headless-shell`.

**Changes:**
- `browser/installer.rs`: `InstallTarget::HeadlessShell` support
- `browser/launcher.rs`: special flags for headless-shell
- `cli/args.rs`: `--fast` flag
- Check: is the AXTree available via headless-shell?
  - If yes: fast mode is fully fledged
  - If no: fast mode only for performance/SEO/security, no WCAG audit

**Risks:**
- AXTree may not be available → feature usable only to a limited extent
- Has to be tested empirically

**Tests:**
- Test `browser install --headless-shell`
- Audit with `--fast` against a known page
- Comparison: standard vs. fast results

### Phase 5: Cleanup legacy

**Goal:** Clean up the old auto-download code and `~/.auditmysite/chromium/`.

**Changes:**
- Mark `--chrome-path` as deprecated (→ `--browser-path`)
- Mark `CHROME_PATH` as deprecated (→ `AUDITMYSITE_BROWSER`)
- `~/.auditmysite/chromium/` migration: check at startup and show a notice
- Remove the old installer code
- Remove `DetectionMethod::AutoDownload`

**Risks:**
- Users with the old setup have to run `browser install` or use a system browser
- Mitigation: clear migration message

**Tests:**
- Test: old `chromium/` folder exists → migration message
- Test: `--chrome-path` still works (deprecation warning)

---

## 12. Concrete Code Sketches

### Browser Registry

```rust
// browser/registry.rs

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
        BrowserKind::Chromium => &["chromium", "chromium-browser"],
        _ => &[],
    }
}

/// Priority order of the BrowserKinds for the automatic search
pub fn search_order() -> &'static [BrowserKind] {
    &[
        BrowserKind::Chrome,
        BrowserKind::Edge,
        BrowserKind::Chromium,
    ]
}
```

### Browser Resolver

```rust
// browser/resolver.rs

use std::path::PathBuf;
use tracing::{info, warn, debug};

use super::detection::{detect_all_browsers, validate_browser};
use super::registry;
use super::types::*;

/// CLI options that affect the browser
pub struct BrowserCliOptions {
    /// --browser-path /explicit/path
    pub browser_path: Option<String>,
    /// --browser chrome|edge|chromium|auto
    pub browser_kind: Option<String>,
    /// --fast
    pub fast: bool,
    /// --strict
    pub strict: bool,
}

/// Finds the best available browser
pub fn resolve_browser(opts: &BrowserCliOptions) -> Result<ResolvedBrowser, BrowserResolveError> {
    let mode = if opts.strict {
        BrowserMode::Strict
    } else if opts.fast {
        BrowserMode::Fast
    } else {
        BrowserMode::Standard
    };

    let mut all_candidates = Vec::new();

    // 1. Explicit path (highest priority)
    if let Some(ref path_str) = opts.browser_path {
        let path = PathBuf::from(path_str);
        let browser = validate_browser(&path, BrowserKind::Custom, BrowserSource::CliFlag)?;
        info!("Using specified browser: {} v{}",
            browser.kind.display_name(),
            browser.version.as_deref().unwrap_or("unknown"));
        return Ok(ResolvedBrowser { browser, mode, all_candidates: vec![] });
    }

    // 2. Environment variable
    if let Ok(path_str) = std::env::var("AUDITMYSITE_BROWSER") {
        let path = PathBuf::from(&path_str);
        if path.exists() {
            let browser = validate_browser(&path, BrowserKind::Custom, BrowserSource::EnvVar)?;
            info!("Using browser from AUDITMYSITE_BROWSER: {}", path.display());
            return Ok(ResolvedBrowser { browser, mode, all_candidates: vec![] });
        }
        warn!("AUDITMYSITE_BROWSER points to non-existent path: {}", path_str);
    }

    // 2b. Legacy: CHROME_PATH
    if let Ok(path_str) = std::env::var("CHROME_PATH") {
        let path = PathBuf::from(&path_str);
        if path.exists() {
            warn!("CHROME_PATH is deprecated, use AUDITMYSITE_BROWSER instead");
            let browser = validate_browser(&path, BrowserKind::Chrome, BrowserSource::EnvVar)?;
            return Ok(ResolvedBrowser { browser, mode, all_candidates: vec![] });
        }
    }

    // 3. Specific BrowserKind requested?
    let filter_kind: Option<BrowserKind> = opts.browser_kind.as_deref().and_then(|s| match s {
        "chrome" => Some(BrowserKind::Chrome),
        "edge" => Some(BrowserKind::Edge),
        "chromium" => Some(BrowserKind::Chromium),
        "auto" | "" => None,
        _ => {
            warn!("Unknown browser kind '{}', using auto-detection", s);
            None
        }
    });

    // 4. System scan
    all_candidates = detect_all_browsers();
    debug!("Found {} browser candidates", all_candidates.len());

    // Filter by the requested kind
    let candidates: Vec<&DetectedBrowser> = if let Some(kind) = filter_kind {
        all_candidates.iter().filter(|b| b.kind == kind).collect()
    } else {
        all_candidates.iter().collect()
    };

    // Pick the best candidate (first in priority order)
    if let Some(browser) = candidates.first() {
        info!("Selected browser: {} v{} ({})",
            browser.kind.display_name(),
            browser.version.as_deref().unwrap_or("unknown"),
            browser.path.display());
        return Ok(ResolvedBrowser {
            browser: (*browser).clone(),
            mode,
            all_candidates,
        });
    }

    // 5. Check managed install (not in strict mode)
    if mode != BrowserMode::Strict {
        if let Some(managed) = check_managed_install(mode == BrowserMode::Fast) {
            info!("Using managed browser: {}", managed.path.display());
            return Ok(ResolvedBrowser {
                browser: managed,
                mode,
                all_candidates,
            });
        }
    }

    // 6. Nothing found
    Err(BrowserResolveError::NoBrowserFound { all_candidates })
}

fn check_managed_install(prefer_headless_shell: bool) -> Option<DetectedBrowser> {
    let base = dirs::home_dir()?.join(".auditmysite").join("browsers");

    if prefer_headless_shell {
        let shell_path = managed_binary_path(&base, "headless-shell");
        if shell_path.exists() {
            return Some(DetectedBrowser {
                kind: BrowserKind::HeadlessShell,
                path: shell_path,
                version: read_version_file(&base.join("headless-shell")),
                source: BrowserSource::ManagedInstall,
            });
        }
    }

    let cft_path = managed_binary_path(&base, "chrome-for-testing");
    if cft_path.exists() {
        return Some(DetectedBrowser {
            kind: BrowserKind::ChromeForTesting,
            path: cft_path,
            version: read_version_file(&base.join("chrome-for-testing")),
            source: BrowserSource::ManagedInstall,
        });
    }

    None
}
```

### Detection (scan all browsers)

```rust
// browser/detection.rs

use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::debug;

use super::registry::{self, RegistryEntry};
use super::types::*;

/// Finds all browsers installed on the system
pub fn detect_all_browsers() -> Vec<DetectedBrowser> {
    let mut found = Vec::new();

    // 1. Scan known paths
    for entry in registry::system_browser_paths() {
        let path = PathBuf::from(entry.path);
        if path.exists() {
            if let Ok(browser) = validate_browser(&path, entry.kind, BrowserSource::SystemPath) {
                // Avoid duplicates (same path)
                if !found.iter().any(|b: &DetectedBrowser| b.path == browser.path) {
                    debug!("Found {} at {}", entry.kind.display_name(), entry.path);
                    found.push(browser);
                }
            }
        }
    }

    // 2. which/where for each BrowserKind
    for kind in registry::search_order() {
        for name in registry::which_names(*kind) {
            if let Some(path) = which_binary(name) {
                if !found.iter().any(|b| b.path == path) {
                    if let Ok(browser) = validate_browser(&path, *kind, BrowserSource::PathSearch) {
                        debug!("Found {} via which: {}", kind.display_name(), path.display());
                        found.push(browser);
                    }
                }
            }
        }
    }

    // Sort by priority (Chrome > Edge > Chromium)
    let order = registry::search_order();
    found.sort_by_key(|b| {
        order.iter().position(|k| *k == b.kind).unwrap_or(usize::MAX)
    });

    found
}

/// Validates a browser binary
pub fn validate_browser(
    path: &Path,
    kind: BrowserKind,
    source: BrowserSource,
) -> Result<DetectedBrowser, DetectionError> {
    // Existence
    if !path.exists() {
        return Err(DetectionError::NotFound(path.to_path_buf()));
    }

    // Executable permission (Unix)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let meta = std::fs::metadata(path)
            .map_err(|_| DetectionError::NotExecutable(path.to_path_buf()))?;
        if meta.permissions().mode() & 0o111 == 0 {
            return Err(DetectionError::NotExecutable(path.to_path_buf()));
        }
    }

    // Version
    let version = get_browser_version(path);

    Ok(DetectedBrowser {
        kind,
        path: path.to_path_buf(),
        version,
        source,
    })
}

fn get_browser_version(path: &Path) -> Option<String> {
    Command::new(path)
        .arg("--version")
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout);
                // Extract version number: "Google Chrome 131.0.6778.108" → "131.0.6778.108"
                text.split_whitespace()
                    .find(|s| s.starts_with(|c: char| c.is_ascii_digit()))
                    .map(|s| s.trim().to_string())
            } else {
                None
            }
        })
}

fn which_binary(name: &str) -> Option<PathBuf> {
    let cmd = if cfg!(target_os = "windows") { "where" } else { "which" };
    Command::new(cmd)
        .arg(name)
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .next()?
                    .trim()
                    .to_string();
                if !path.is_empty() {
                    Some(PathBuf::from(path))
                } else {
                    None
                }
            } else {
                None
            }
        })
}
```

### CLI Browser Subcommand

```rust
// cli/browser.rs

use clap::Subcommand;

#[derive(Subcommand)]
pub enum BrowserCommand {
    /// Detect all installed browsers
    Detect,
    /// Install Chrome for Testing
    Install {
        /// Install headless-shell instead (smaller, faster)
        #[arg(long)]
        headless_shell: bool,
        /// Install specific version
        #[arg(long)]
        version: Option<String>,
        /// Force reinstall even if already present
        #[arg(long)]
        force: bool,
    },
    /// Remove managed browser installation
    Remove {
        /// Remove headless-shell
        #[arg(long)]
        headless_shell: bool,
        /// Remove all managed browsers
        #[arg(long)]
        all: bool,
    },
    /// Print path of the active browser
    Path,
}

pub async fn handle_browser_command(cmd: &BrowserCommand) -> anyhow::Result<()> {
    match cmd {
        BrowserCommand::Detect => {
            let browsers = crate::browser::detection::detect_all_browsers();
            if browsers.is_empty() {
                println!("No browsers found.");
                println!();
                println!("Install one:");
                println!("  brew install --cask google-chrome");
                println!("  auditmysite browser install");
            } else {
                for browser in &browsers {
                    println!("  ✓ {:<25} {:<20} {}",
                        browser.kind.display_name(),
                        browser.version.as_deref().unwrap_or("unknown"),
                        browser.path.display());
                }
            }

            // Check managed installs
            let managed_dir = dirs::home_dir()
                .map(|h| h.join(".auditmysite").join("browsers"));
            if let Some(dir) = managed_dir {
                let cft = dir.join("chrome-for-testing");
                let hs = dir.join("headless-shell");
                if cft.exists() {
                    println!("  ✓ {:<25} {:<20} {}",
                        "Chrome for Testing",
                        read_version(&cft),
                        cft.display());
                } else {
                    println!("  ✗ Chrome for Testing      not installed");
                }
                if hs.exists() {
                    println!("  ✓ {:<25} {:<20} {}",
                        "Headless Shell",
                        read_version(&hs),
                        hs.display());
                } else {
                    println!("  ✗ Headless Shell          not installed");
                }
            }

            Ok(())
        }

        BrowserCommand::Install { headless_shell, version, force } => {
            let target = if *headless_shell {
                crate::browser::types::InstallTarget::HeadlessShell
            } else {
                crate::browser::types::InstallTarget::ChromeForTesting
            };
            crate::browser::installer::BrowserInstaller::install(target, version.as_deref(), *force).await?;
            Ok(())
        }

        BrowserCommand::Remove { headless_shell, all } => {
            // Implementation: delete directories
            todo!()
        }

        BrowserCommand::Path => {
            let opts = crate::browser::resolver::BrowserCliOptions {
                browser_path: None,
                browser_kind: None,
                fast: false,
                strict: false,
            };
            match crate::browser::resolver::resolve_browser(&opts) {
                Ok(resolved) => println!("{}", resolved.browser.path.display()),
                Err(_) => {
                    eprintln!("No browser found");
                    std::process::exit(1);
                }
            }
            Ok(())
        }
    }
}
```

### Doctor

```rust
// doctor.rs

pub struct DoctorReport {
    pub checks: Vec<DoctorCheck>,
}

pub struct DoctorCheck {
    pub name: String,
    pub status: CheckStatus,
    pub detail: String,
}

pub enum CheckStatus {
    Ok,
    Warning,
    Error,
}

pub fn run_doctor() -> DoctorReport {
    let mut checks = Vec::new();

    // 1. Browser check
    let browsers = crate::browser::detection::detect_all_browsers();
    if browsers.is_empty() {
        checks.push(DoctorCheck {
            name: "Browser".into(),
            status: CheckStatus::Error,
            detail: "No compatible browser found".into(),
        });
    } else {
        let best = &browsers[0];
        checks.push(DoctorCheck {
            name: "Browser".into(),
            status: CheckStatus::Ok,
            detail: format!("{} v{}",
                best.kind.display_name(),
                best.version.as_deref().unwrap_or("unknown")),
        });
    }

    // 2. Permissions
    if let Some(browser) = browsers.first() {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let executable = std::fs::metadata(&browser.path)
                .map(|m| m.permissions().mode() & 0o111 != 0)
                .unwrap_or(false);
            checks.push(DoctorCheck {
                name: "Permissions".into(),
                status: if executable { CheckStatus::Ok } else { CheckStatus::Error },
                detail: if executable { "Executable".into() } else { "Not executable".into() },
            });
        }
    }

    // 3. Disk space
    let home = dirs::home_dir();
    if let Some(ref home) = home {
        let cache_dir = home.join(".auditmysite");
        // Simple check: 500 MB free?
        checks.push(DoctorCheck {
            name: "Disk".into(),
            status: CheckStatus::Ok,
            detail: format!("Cache dir: {}", cache_dir.display()),
        });
    }

    // 4. Config
    let config_exists = std::path::Path::new("auditmysite.toml").exists();
    checks.push(DoctorCheck {
        name: "Config".into(),
        status: if config_exists { CheckStatus::Ok } else { CheckStatus::Warning },
        detail: if config_exists {
            "auditmysite.toml found".into()
        } else {
            "No config file (using defaults)".into()
        },
    });

    DoctorReport { checks }
}

pub fn print_doctor_report(report: &DoctorReport) {
    for check in &report.checks {
        let icon = match check.status {
            CheckStatus::Ok => "✓",
            CheckStatus::Warning => "⚠",
            CheckStatus::Error => "✗",
        };
        println!("  {} {:<15} {}", icon, check.name, check.detail);
    }
}
```

---

## 13. Recommended Decision

### What should the default be?

**Detect and use the system browser automatically.** Chrome > Edge > Chromium. No download, no installation, no surprises. Most developers and CI systems have Chrome or Edge installed. It works out of the box.

### What should be optional?

- `auditmysite browser install` for users without a system browser (rare on macOS/Windows, more common on minimal Linux servers)
- `--fast` mode with headless-shell (after validating that the AXTree works with it)
- `--browser edge` for users who prefer Edge over Chrome
- `--strict` for CI pipelines that want no fallbacks

### What should I avoid?

1. **No silent auto-download.** The current behavior (silently downloading Chromium) is the core problem. It surprises users, causes firewall problems, and is hard to debug.
2. **Do not bundle our own browser.** Not in the formula, not in the release archive, nowhere. The browser is a runtime dependency, not a build dependency.
3. **No hard browser dependency in Homebrew.** `depends_on cask: "google-chrome"` creates more problems than it solves.
4. **No version pinning in the installer.** The current hardcoded version (131.0.6778.108) will become outdated quickly. The Chrome for Testing API provides the current Stable.

### Trade-offs

| Decision | Advantage | Disadvantage |
|---|---|---|
| No auto-download | No surprises, no firewall problems | A user without Chrome has to run `browser install` |
| Prefer system Chrome | Signed, no Gatekeeper problems, always up to date | Version not under our control |
| Edge as fallback | Almost always present on Windows | Minimal additional testing effort |
| Managed install optional | User has explicit control | One additional step for users without a system browser |
| No version pinning | Always the current version | In theory, a Chrome version could have a bug |

### Recommended implementation order

1. **Phase 1 (Detection Refactor)** → biggest impact, makes everything more robust
2. **Phase 2 (Explicit Installer)** → removes the silent download
3. **Phase 5 (Cleanup)** → can happen right after Phase 2
4. **Phase 3 (Doctor)** → nice-to-have, reduces support requests
5. **Phase 4 (Headless-Shell)** → only if the AXTree works with it, otherwise low priority
