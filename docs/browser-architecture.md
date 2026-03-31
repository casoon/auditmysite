# Browser-Architektur: Technische Ausarbeitung

> Statushinweis:
> Dieses Dokument enthält neben dem Ist-Zustand auch Architekturideen und frühere Vorschläge.
> Nicht jede hier beschriebene Option ist aktuell in der CLI implementiert.
> Für den tatsächlichen Benutzerstand sind `auditmysite --help`, die [README](/Users/jseidel/GitHub/auditmysite/README.md) und [ARCHITECTURE.md](/Users/jseidel/GitHub/auditmysite/docs/ARCHITECTURE.md) maßgeblich.

## Status quo (Ist-Zustand)

- Browser-Auflösung über `resolver` mit `--browser-path`, `AUDITMYSITE_BROWSER`, System-Browsern und Managed Install
- Managed Browser unter `~/.auditmysite/browsers/`
- Kein stiller Auto-Download; Installation erfolgt explizit über `auditmysite browser install`
- `browser`-Subcommands und `doctor` sind implementiert
- `--chrome-path` ist nur noch Alias für `--browser-path`
- Einige weiter unten beschriebene Flags wie `--browser`, `--fast` und `--strict` sind derzeit Vorschläge, nicht produktive CLI-Optionen

---

## 1. Zielarchitektur

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
│  4. Error mit Installationshinweisen                │
└────────┬────────────────────────────────────────────┘
         │
┌────────▼────────────────────────────────────────────┐
│                Browser Launcher                      │
│  Modes: Standard │ Strict │ Fast (headless-shell)   │
│  Flags, Timeouts, Port-Handling, Cleanup            │
└────────┬────────────────────────────────────────────┘
         │
┌────────▼────────────────────────────────────────────┐
│              chromiumoxide / CDP                     │
│  AXTree, Computed Styles, Web Vitals, Navigation    │
└─────────────────────────────────────────────────────┘
```

**Kernprinzipien:**

1. **Kein stiller Download.** Wenn kein Browser gefunden wird → Fehler mit klarer Anleitung.
2. **System-Browser bevorzugen.** Der User hat Chrome oder Edge installiert? Nutzen.
3. **Expliziter Installer.** `auditmysite browser install` als bewusste Aktion.
4. **Homebrew bleibt leichtgewichtig.** Keine Browser-Dependency in der Formula.

**Verantwortlichkeiten:**

| Komponente | Aufgabe |
|---|---|
| `browser::registry` | Kennt alle unterstützten Browser-Typen und ihre Pfade pro Plattform |
| `browser::detection` | Findet installierte Browser, prüft Executable + Version |
| `browser::resolver` | Führt die priorisierte Auswahl durch (CLI → Env → Registry) |
| `browser::installer` | Lädt Chrome for Testing / headless-shell herunter |
| `browser::launcher` | Startet den Browser mit den richtigen Flags für den Mode |
| `cli::browser` | Subcommands: `detect`, `install`, `remove` |
| `cli::doctor` | Systemdiagnose: Browser, Netzwerk, Berechtigungen |

---

## 2. Browser-Strategie

### Unterstützte Browser

| Browser | BrowserKind | CDP-kompatibel | Priorität |
|---|---|---|---|
| Google Chrome | `Chrome` | Ja | 1 (Standard) |
| Microsoft Edge | `Edge` | Ja (gleiche Engine) | 2 |
| Ungoogled Chromium | `UngoogledChromium` | Ja | 3 |
| Chromium (System) | `Chromium` | Ja | 4 |
| Chrome for Testing (managed) | `ChromeForTesting` | Ja | 5 (explizit installiert) |
| chrome-headless-shell (managed) | `HeadlessShell` | Ja (eingeschränkt) | — (nur mit `--fast`) |

**Annahme:** Brave, Opera, Vivaldi sind technisch möglich (Chromium-basiert), aber der Support-Aufwand lohnt sich nicht. Wer Brave nutzt, kann `--browser-path` setzen.

### Erkennungsreihenfolge (Default-Mode)

```
1. --browser-path /explicit/path          → Genau diesen Browser nutzen
2. AUDITMYSITE_BROWSER=/env/path          → Env-Variable
3. CHROME_PATH=/legacy/path               → Rückwärtskompatibilität
4. Google Chrome (System)                 → Bevorzugt, weil verbreitet und signiert
5. Microsoft Edge (System)                → Auf macOS/Windows oft vorinstalliert
6. Ungoogled Chromium (System/Brew)       → Privacy-bewusste Alternative
7. Chromium (System/Brew/Snap)            → Generisches Chromium
8. Chrome for Testing (~/.auditmysite/)   → Nur wenn per `browser install` installiert
9. → Fehler: ChromeNotFound              → Mit konkreten Installationshinweisen
```

### Modi

| Modus | Flag | Verhalten |
|---|---|---|
| **Default** | (keiner) | System-Browser suchen → managed Browser → Fehler |
| **Strict** | `--strict` | Nur System-Browser, kein Fallback auf managed |
| **Fast** | `--fast` | Bevorzugt `chrome-headless-shell`, dann Standard-Kette |

**Strict-Mode:** Für CI/CD-Pipelines, wo der Browser explizit kontrolliert werden soll. Kein Auto-Fallback auf Downloads.

**Fast-Mode:** `chrome-headless-shell` ist kleiner (~50 MB vs ~300 MB) und startet schneller. Einschränkung: Kein vollständiger AXTree — prüfen ob das für den Use-Case reicht. **Annahme:** Falls der AXTree mit headless-shell nicht verfügbar ist, muss ein Fallback auf Standard-Chrome passieren, mit Warnung.

### CLI-Flags und Env-Variablen

| Flag/Env | Typ | Default | Beschreibung |
|---|---|---|---|
| `--browser-path <PATH>` | CLI | — | Expliziter Pfad zur Browser-Binary |
| `--browser <KIND>` | CLI | `auto` | Browser-Typ erzwingen: `chrome`, `edge`, `chromium`, `auto` |
| `--fast` | CLI | false | Bevorzugt headless-shell für Geschwindigkeit |
| `--strict` | CLI | false | Nur System-Browser, kein managed Fallback |
| `AUDITMYSITE_BROWSER` | Env | — | Pfad zur Browser-Binary |
| `CHROME_PATH` | Env | — | Legacy-Kompatibilität (wird zu Gunsten von `AUDITMYSITE_BROWSER` deprecated) |

---

## 3. CLI-Design

### Subcommand-Struktur

```
auditmysite <URL>                              # Single audit (Hauptbefehl bleibt)
auditmysite --sitemap <URL>                    # Batch via Sitemap
auditmysite --url-file <FILE>                  # Batch via Datei

auditmysite browser detect                     # Zeigt alle gefundenen Browser
auditmysite browser install                    # Installiert Chrome for Testing
auditmysite browser install --headless-shell   # Installiert chrome-headless-shell
auditmysite browser remove                     # Entfernt managed Browser
auditmysite browser path                       # Gibt Pfad des aktiven Browsers aus

auditmysite doctor                             # Systemdiagnose
```

### Beispielaufrufe

```bash
# Standard: System-Chrome nutzen
auditmysite https://example.com --full --format pdf --output reports/audit.pdf

# Expliziter Browser
auditmysite https://example.com --browser-path /usr/bin/chromium-browser

# Edge erzwingen
auditmysite https://example.com --browser edge

# Fast-Mode (headless-shell, falls installiert)
auditmysite https://example.com --fast

# Strict-Mode (nur System-Browser, kein Fallback)
auditmysite https://example.com --strict

# Browser erkennen (Diagnostik)
auditmysite browser detect
# Ausgabe:
#   ✓ Google Chrome 131.0.6778.108  /Applications/Google Chrome.app/.../Google Chrome
#   ✓ Microsoft Edge 131.0.2903.86  /Applications/Microsoft Edge.app/.../Microsoft Edge
#   ✗ Ungoogled Chromium            not found
#   ✗ Chromium                      not found
#   ✗ Chrome for Testing            not installed (~/.auditmysite/browsers/)
#   Active: Google Chrome (auto-detected)

# Browser installieren
auditmysite browser install
# Ausgabe:
#   Downloading Chrome for Testing v131.0.6778.108...
#   Platform: macOS ARM64
#   Destination: ~/.auditmysite/browsers/chrome-for-testing/
#   Progress: [████████████████████] 100% (142 MB)
#   ✓ Installed successfully

# Systemdiagnose
auditmysite doctor
# Ausgabe:
#   Browser:     ✓ Google Chrome 131.0.6778.108
#   Permissions: ✓ Executable
#   Network:     ✓ Can reach example.com
#   Firewall:    ✓ Loopback connections allowed
#   Disk:        ✓ 2.1 GB free in ~/.auditmysite/
#   Config:      ✓ auditmysite.toml found
```

### Fehlerfall: Kein Browser gefunden

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

## 4. Rust-Modulstruktur

```
src/browser/
├── mod.rs              # Public exports
├── detection.rs        # Findet Browser auf dem System (Pfade, which, Validierung)
├── registry.rs         # Definiert BrowserKind, Pfade pro Plattform, Suchreihenfolge
├── resolver.rs         # Orchestriert: CLI → Env → Registry → Managed → Error
├── installer.rs        # Downloads: Chrome for Testing, headless-shell
├── launcher.rs         # Startet Browser mit Mode-spezifischen Flags
├── manager.rs          # Browser-Lifecycle (bestehend, nutzt resolver + launcher)
├── pool.rs             # Concurrent page management (bestehend)
└── types.rs            # Shared types (BrowserKind, DetectedBrowser, LaunchConfig, etc.)

src/cli/
├── mod.rs
├── args.rs             # Hauptargs + neue Flags (--browser, --browser-path, --fast, --strict)
├── browser.rs          # NEU: Subcommands detect/install/remove/path
├── doctor.rs           # NEU: Systemdiagnose
└── config.rs           # Config file support

src/
├── doctor.rs           # NEU: Doctor-Logik (Browser, Netzwerk, Berechtigungen)
└── ...
```

**Verantwortung pro Modul:**

| Modul | Verantwortung |
|---|---|
| `types.rs` | Alle Enums und Structs: `BrowserKind`, `DetectedBrowser`, `LaunchConfig`, `BrowserMode` |
| `registry.rs` | Statische Daten: Welche Browser existieren, welche Pfade pro OS, Suchreihenfolge |
| `detection.rs` | Laufzeit-Erkennung: Pfade prüfen, `which` aufrufen, Version auslesen, Capabilities checken |
| `resolver.rs` | Entscheidungslogik: Nimmt CLI-Optionen, Env-Vars und Detection-Ergebnisse, gibt einen `ResolvedBrowser` zurück |
| `installer.rs` | Download + Extraktion von Chrome for Testing / headless-shell nach `~/.auditmysite/browsers/` |
| `launcher.rs` | Baut `BrowserConfig` auf, wählt Flags je nach Mode, startet via chromiumoxide |
| `manager.rs` | High-Level-API: `BrowserManager::new()` nutzt resolver → launcher, verwaltet Lifecycle |
| `pool.rs` | Concurrent page management (unverändert) |

---

## 5. Zustandsmodell / Datenstrukturen

```rust
// ── types.rs ──────────────────────────────────────────────────

/// Welcher Browser-Typ
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
    /// Menschenlesbarer Name
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

    /// Ob das ein managed (selbst installierter) Browser ist
    pub fn is_managed(&self) -> bool {
        matches!(self, Self::ChromeForTesting | Self::HeadlessShell)
    }
}

/// Wie der Browser gefunden wurde
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserSource {
    /// Explizit per --browser-path
    CliFlag,
    /// Per AUDITMYSITE_BROWSER oder CHROME_PATH
    EnvVar,
    /// In bekannten System-Pfaden gefunden
    SystemPath,
    /// Per `which`/`where` im PATH gefunden
    PathSearch,
    /// Selbst installiert unter ~/.auditmysite/browsers/
    ManagedInstall,
}

/// Laufmodus für den Browser
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserMode {
    /// Normaler Headless-Chrome mit allen Features
    Standard,
    /// Nur System-Browser, kein Fallback
    Strict,
    /// Bevorzugt headless-shell für Geschwindigkeit
    Fast,
}

/// Ein gefundener Browser
#[derive(Debug, Clone)]
pub struct DetectedBrowser {
    pub kind: BrowserKind,
    pub path: PathBuf,
    pub version: Option<String>,
    pub source: BrowserSource,
}

/// Ergebnis der Browser-Resolution
#[derive(Debug, Clone)]
pub struct ResolvedBrowser {
    /// Der ausgewählte Browser
    pub browser: DetectedBrowser,
    /// Welcher Mode aktiv ist
    pub mode: BrowserMode,
    /// Alle gefundenen Kandidaten (für `browser detect`)
    pub all_candidates: Vec<DetectedBrowser>,
}

/// Konfiguration für den Browser-Start
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

/// Was installiert werden soll
#[derive(Debug, Clone, Copy)]
pub enum InstallTarget {
    /// Chrome for Testing (vollständig)
    ChromeForTesting,
    /// chrome-headless-shell (minimal, schnell)
    HeadlessShell,
}
```

---

## 6. Erkennungslogik

### Schritt für Schritt

```
resolve_browser(cli_args) → Result<ResolvedBrowser>
│
├─ 1. CLI-Flag: --browser-path gesetzt?
│     → Pfad validieren, Version prüfen, BrowserKind::Custom
│     → Fehler wenn Pfad nicht existiert oder nicht executable
│
├─ 2. Env-Var: AUDITMYSITE_BROWSER gesetzt?
│     → Pfad validieren, Version prüfen
│     → Fehler nur loggen wenn Pfad ungültig, weiter mit nächstem Schritt
│
├─ 3. Env-Var: CHROME_PATH gesetzt? (Legacy)
│     → Wie 2., aber mit Deprecation-Warnung
│
├─ 4. --browser Flag gesetzt? (z.B. --browser edge)
│     → Nur den spezifischen BrowserKind suchen
│     → Fehler wenn dieser Typ nicht gefunden
│
├─ 5. System-Scan (Registry-Reihenfolge):
│     │
│     ├─ Chrome: Bekannte Pfade → which google-chrome
│     ├─ Edge: Bekannte Pfade → which microsoft-edge
│     ├─ Ungoogled Chromium: Brew-Pfade → which
│     └─ Chromium: Bekannte Pfade → Brew → Snap → which
│     │
│     → Ersten Treffer nehmen (wenn nicht --strict)
│
├─ 6. Managed Install prüfen (wenn nicht --strict):
│     → ~/.auditmysite/browsers/chrome-for-testing/ existiert?
│     → ~/.auditmysite/browsers/headless-shell/ existiert? (nur bei --fast)
│
└─ 7. Nichts gefunden → ChromeNotFound Error
```

### Plattform-spezifische Pfade

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
        // Ungoogled Chromium (Homebrew Cask: eloston-chromium)
        (BrowserKind::UngoogledChromium,
         "/Applications/Chromium.app/Contents/MacOS/Chromium"),  // Achtung: kollidiert mit normalem Chromium
        // Chromium (Homebrew)
        (BrowserKind::Chromium,
         "/opt/homebrew/bin/chromium"),
        (BrowserKind::Chromium,
         "/usr/local/bin/chromium"),
        // Chrome Canary (niedrige Prio, instabil)
        (BrowserKind::Chrome,
         "/Applications/Google Chrome Canary.app/Contents/MacOS/Google Chrome Canary"),
    ]
}
```

**Problem: Ungoogled Chromium vs. Chromium auf macOS**

Beide installieren sich als `/Applications/Chromium.app`. Unterscheidung möglich über:
- `Chromium --version` → "Chromium 131.0.6778.108 Ungoogled" vs. "Chromium 131.0.6778.108"
- `brew list --cask` prüfen: `eloston-chromium` vs. `chromium`
- In der Praxis: Beide funktionieren identisch mit CDP, die Unterscheidung ist kosmetisch.

**Empfehlung:** Beide als `BrowserKind::Chromium` behandeln, es sei denn der User hat `--browser ungoogled` explizit gesetzt. Auf macOS reicht die `/Applications/Chromium.app`-Erkennung.

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
        // Edge (immer vorhanden auf Windows 10+)
        (BrowserKind::Edge,
         r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe"),
        (BrowserKind::Edge,
         r"C:\Program Files\Microsoft\Edge\Application\msedge.exe"),
    ]
}
```

### Validierung einer gefundenen Binary

```rust
fn validate_browser(path: &Path) -> Result<DetectedBrowser> {
    // 1. Existiert die Datei?
    if !path.exists() {
        return Err(NotFound);
    }

    // 2. Ist sie executable? (Unix)
    #[cfg(unix)]
    check_executable_permission(path)?;

    // 3. Version auslesen
    let version = run_version_check(path)?;  // --version

    // 4. Minimale Versionsprüfung
    //    chromiumoxide 0.8 braucht Chrome >= 64 (CDP-Protokoll)
    //    Praktisch: Chrome >= 90 empfohlen für stabile AXTree-API
    if let Some(ref v) = version {
        let major = parse_major_version(v);
        if major < 90 {
            warn!("Chrome {} is very old, recommend >= 90", v);
        }
    }

    // 5. Optional: CDP-Capability-Check
    //    Schnelltest: `chrome --headless --dump-dom about:blank`
    //    Wenn das funktioniert, ist CDP verfügbar.
    //    → Nur bei `doctor`, nicht bei jedem Start.

    Ok(DetectedBrowser { ... })
}
```

### which-Suchbegriffe pro BrowserKind

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

## 7. Installationsstrategie

### `auditmysite browser install`

```
~/.auditmysite/
├── browsers/
│   ├── chrome-for-testing/          # Voller Chrome
│   │   ├── version.txt              # "131.0.6778.108"
│   │   └── chrome-mac-arm64/        # Plattform-spezifisch
│   │       └── Google Chrome for Testing.app/...
│   └── headless-shell/              # Minimal (nur mit --headless-shell)
│       ├── version.txt
│       └── chrome-headless-shell
└── chromium/                         # LEGACY (Phase 5 löschen)
```

### Download-Quellen

Chrome for Testing hat eine stabile JSON-API:

```
https://googlechromelabs.github.io/chrome-for-testing/known-good-versions-with-downloads.json
```

**Empfohlenes Vorgehen:**

1. Latest Stable Version von der API holen (nicht hardcoden)
2. Download-URL für Plattform + Architektur zusammenbauen
3. Herunterladen, extrahieren, Version in `version.txt` speichern

```rust
/// Installer-Logik
impl BrowserInstaller {
    /// Chrome for Testing installieren
    pub async fn install(target: InstallTarget) -> Result<InstalledBrowser> {
        // 1. Aktuelle stabile Version ermitteln
        let version = Self::fetch_latest_stable_version().await?;

        // 2. Prüfen ob schon installiert
        let install_dir = Self::install_dir(target);
        if let Some(existing) = Self::check_existing(&install_dir, &version) {
            info!("Already installed: {} v{}", target.name(), version);
            return Ok(existing);
        }

        // 3. Download-URL zusammenbauen
        let url = Self::download_url(target, &version)?;

        // 4. Herunterladen mit Progress
        info!("Downloading {} v{}...", target.name(), version);
        let archive = Self::download_with_progress(&url).await?;

        // 5. Extrahieren
        Self::extract(&archive, &install_dir)?;

        // 6. Version-Datei schreiben
        fs::write(install_dir.join("version.txt"), &version)?;

        // 7. Binary-Pfad zurückgeben
        let binary_path = Self::binary_path(target, &install_dir);
        Ok(InstalledBrowser { path: binary_path, version })
    }

    /// Stabile Version von API holen
    async fn fetch_latest_stable_version() -> Result<String> {
        // Primär: Chrome for Testing API
        let url = "https://googlechromelabs.github.io/chrome-for-testing/last-known-good-versions.json";
        let resp: serde_json::Value = reqwest::get(url).await?.json().await?;
        let version = resp["channels"]["Stable"]["version"]
            .as_str()
            .ok_or_else(|| anyhow!("Could not parse stable version"))?;
        Ok(version.to_string())
    }
}
```

### Version-Pinning

**Empfehlung:** Kein Pinning auf eine exakte Version. Stattdessen:

- `browser install` holt immer die aktuelle Stable
- Die installierte Version wird in `version.txt` gespeichert
- `browser install --update` aktualisiert auf neueste Stable
- Wer eine bestimmte Version braucht: `browser install --version 131.0.6778.108`
- **Kompatibilitätsgarantie:** Chrome for Testing Stable ist abwärtskompatibel mit CDP

### Integritätsprüfung

Chrome for Testing bietet keine offiziellen Checksums. Pragmatischer Ansatz:

1. Download über HTTPS (TLS-Integrität)
2. Nach Extraktion: Binary-Existenz prüfen
3. `--version` aufrufen und parsen
4. Wenn alles klappt → gültig

### Fehlerfälle

| Fehler | Reaktion |
|---|---|
| Kein Internet | Klare Meldung, Hinweis auf `--browser-path` |
| Download-Fehler | 2 Retries mit Backoff, dann Fehler |
| Extraktion fehlgeschlagen | Cache-Dir löschen, Fehler melden |
| Kein Platz | Plattengröße prüfen, warnen bei < 500 MB |
| Binary nicht lauffähig | `--version`-Check schlägt fehl → Fehler + Hinweis |

---

## 8. Launch-Strategie

### Chrome-Flags nach Mode

```rust
fn build_launch_args(config: &LaunchConfig) -> Vec<String> {
    let mut args = vec![
        // Immer
        "--headless=new".into(),          // Neuer Headless-Mode (Chrome >= 112)
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

    // macOS: Firewall-Problem vermeiden
    if cfg!(target_os = "macos") {
        // Loopback-Verbindung statt Netzwerk
        args.push("--remote-debugging-address=127.0.0.1".into());
        // Kein mDNS/Bonjour → keine Firewall-Nachfrage
        args.push("--disable-features=NetworkService".into());
    }

    // Mode-spezifisch
    match config.mode {
        BrowserMode::Standard => {
            args.push("--disable-gpu".into());
        }
        BrowserMode::Strict => {
            args.push("--disable-gpu".into());
            // Keine Extra-Features
        }
        BrowserMode::Fast => {
            args.push("--disable-gpu".into());
            args.push("--blink-settings=imagesEnabled=false".into());
            args.push("--disable-javascript".into());  // Nur wenn JS nicht gebraucht
            // ACHTUNG: JS wird für Web Vitals gebraucht!
            // → JS-disable nur wenn --no-vitals gesetzt
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

**Anmerkung zu `--headless=new` vs `--headless`:**
- `--headless` (alt): Chrome < 112, veralteter Headless-Mode
- `--headless=new`: Chrome >= 112, nutzt den echten Browser-Renderer
- **Empfehlung:** `--headless=new` für Chrome >= 112, Fallback auf `--headless` für ältere Versionen. Versionsprüfung in Detection.

### Port-Handling

```rust
/// Freien Port finden für CDP
fn find_free_port() -> u16 {
    // Binde kurz an Port 0, OS vergibt freien Port
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.local_addr().unwrap().port()
}
```

chromiumoxide macht das intern, aber wenn wir den Port selbst kontrollieren wollen (z.B. für `--remote-debugging-port`), brauchen wir das.

### Prozess-Lifecycle und Zombie-Vermeidung

```rust
impl BrowserManager {
    pub async fn close(self) -> Result<()> {
        // 1. Alle Pages schließen
        if let Ok(pages) = self.browser.pages().await {
            for page in pages {
                page.close().await.ok();
            }
        }

        // 2. Browser graceful beenden
        //    chromiumoxide sendet Browser.close über CDP
        drop(self.browser);

        // 3. Timeout: Wenn Browser nach 5s noch läuft, SIGTERM
        //    chromiumoxide handled das über Drop, aber wir
        //    sollten sicherstellen, dass kein Zombie bleibt

        // 4. Handler-Task abbrechen
        if let Some(handle) = self.handler.lock().await.take() {
            handle.abort();
        }

        Ok(())
    }
}

// Zusätzlich: Ctrl+C Handler im main.rs
// Stellt sicher, dass Browser auch bei SIGINT aufgeräumt wird
```

### Logging

```rust
// Beim Start
info!("Browser: {} v{} ({})",
    resolved.browser.kind.display_name(),
    resolved.browser.version.as_deref().unwrap_or("unknown"),
    resolved.browser.source);
info!("Mode: {:?}", resolved.mode);
debug!("Launch args: {:?}", launch_args);
debug!("CDP port: {}", port);

// Bei Problemen
warn!("Browser process exited unexpectedly");
error!("CDP connection lost: {}", err);
```

---

## 9. Homebrew-Strategie

### Formula-Struktur

```ruby
class Auditmysite < Formula
  desc "WCAG 2.1 Accessibility Checker with full audit reporting"
  homepage "https://github.com/casoon/auditmysite"
  url "https://github.com/casoon/auditmysite/archive/refs/tags/v0.4.3.tar.gz"
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
    # Prüfe nur dass die Binary läuft, nicht den Browser
    assert_match "auditmysite", shell_output("#{bin}/auditmysite --version")
  end
end
```

### Warum Browser NICHT als harte Dependency

1. **Viele User haben Chrome schon.** `depends_on "google-chrome"` würde es nochmal installieren oder Konflikte erzeugen.
2. **Cask-Dependencies sind fragil.** `depends_on cask: "google-chrome"` funktioniert in Homebrew nicht zuverlässig.
3. **Edge ist auf macOS oft vorinstalliert.** Warum Chrome erzwingen, wenn Edge schon da ist?
4. **CI/CD-Systeme** haben oft eigene Chrome-Installationen.
5. **Der User soll die Kontrolle haben**, welchen Browser er nutzt.

### Was bei `brew install auditmysite` passieren soll

1. Rust-Compiler baut die Binary
2. Binary wird nach `/opt/homebrew/bin/auditmysite` installiert
3. `caveats` zeigt den Hinweis auf Browser-Requirement
4. **Kein Browser-Download, kein Auto-Install, kein Netzwerkzugriff zur Laufzeit**

### Was NICHT passieren soll

- Kein automatischer Chrome-Download beim ersten `brew install`
- Keine Homebrew-Dependency auf einen Browser-Cask
- Kein Post-Install-Script das Netzwerkzugriffe macht
- Kein Abbruch der Installation wenn kein Browser vorhanden ist

### Fehlermeldung nach Installation (erster Run ohne Browser)

Wenn der User `auditmysite https://example.com` ausführt und kein Browser da ist:

```
Error: No compatible browser found.

You installed auditmysite via Homebrew — great!
Now you need a Chromium-based browser. Options:

  brew install --cask google-chrome     # Recommended
  auditmysite browser install           # Self-contained download

Run 'auditmysite doctor' to check your setup.
```

---

## 10. macOS-spezifische Probleme

### Problem 1: Firewall-Popups

**Ursache:** Chrome öffnet einen Listening-Socket für CDP. macOS Application Firewall fragt "Möchten Sie eingehende Verbindungen erlauben?"

**Lösung:**
```
--remote-debugging-address=127.0.0.1
```
Bindet nur an Loopback. Die Firewall fragt nur bei Nicht-Loopback-Verbindungen. chromiumoxide nutzt das standardmäßig, aber wir setzen es explizit.

**Zusätzlich:**
```
--disable-features=MediaRouter
```
Verhindert mDNS-Discovery (Chromecast etc.), das ebenfalls Firewall-Popups auslösen kann.

### Problem 2: Gatekeeper-Warnungen

**Ursache:** Selbst heruntergeladene Binaries haben kein Code-Signing und bekommen das `com.apple.quarantine`-Extended-Attribute.

**Lösung bei `browser install`:**
```rust
#[cfg(target_os = "macos")]
fn remove_quarantine(path: &Path) -> Result<()> {
    // Quarantine-Attribut entfernen
    Command::new("xattr")
        .args(["-dr", "com.apple.quarantine"])
        .arg(path)
        .output()
        .ok();
    Ok(())
}
```

Nach der Extraktion das Quarantine-Attribut auf dem gesamten App-Bundle entfernen. Das ist sicher, weil wir die Datei selbst von einer bekannten Google-URL heruntergeladen haben.

### Problem 3: Unsignierte Browser-Binaries

**Chrome for Testing** ist von Google signiert (hat ein gültiges Code-Signing-Zertifikat). Das ist kein Problem.

**Selbst kompiliertes Chromium** (z.B. via `brew install chromium`) ist NICHT signiert und kann Gatekeeper-Probleme verursachen.

**Empfehlung:**
- System-Chrome und System-Edge sind immer signiert → bevorzugen
- Chrome for Testing ist signiert → sicher als Managed Install
- Unsigniertes Chromium → warnen, aber erlauben

### Problem 4: Unterschiede zwischen installiertem Chrome und Download

| Aspekt | System Chrome | Chrome for Testing |
|---|---|---|
| Code-Signing | Ja (Apple Notarized) | Ja (Google signiert) |
| Firewall | Bereits erlaubt | Neue Regel nötig |
| Updates | Automatisch | Manuell (`browser install --update`) |
| Pfad | `/Applications/Google Chrome.app/...` | `~/.auditmysite/browsers/...` |
| Gatekeeper | Kein Problem | Quarantine entfernen nötig |
| User-Profil | Eigenes Profil | Kein Profil (headless) |

**Empfehlung:** System-Chrome ist **immer** die bessere Wahl auf macOS. Der Managed Install ist ein Fallback für Systeme ohne Chrome.

### Problem 5: SingletonLock

**Ursache:** Chrome erstellt ein `SingletonLock` im Profil-Verzeichnis. Wenn der Prozess unsauber beendet wird, bleibt das Lock liegen und verhindert den nächsten Start.

**Aktuelle Lösung:** User muss manuell `rm` aufrufen.

**Bessere Lösung:**
```rust
fn cleanup_stale_locks(profile_dir: &Path) {
    let lock_file = profile_dir.join("SingletonLock");
    if lock_file.exists() {
        // Prüfe ob der Prozess noch läuft
        // SingletonLock enthält hostname-PID
        if let Ok(content) = fs::read_to_string(&lock_file) {
            // Format: "hostname-PID"
            if let Some(pid_str) = content.split('-').last() {
                if let Ok(pid) = pid_str.parse::<u32>() {
                    // Prüfe ob PID noch lebt
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

Noch besser: **Eigenes temporäres Profil-Verzeichnis** pro Audit-Run:
```rust
let temp_profile = tempfile::tempdir()?;
args.push(format!("--user-data-dir={}", temp_profile.path().display()));
```

Das vermeidet das SingletonLock-Problem komplett und verhindert Konflikte mit laufenden Chrome-Instanzen des Users.

**Annahme:** chromiumoxide setzt bereits ein temporäres Profil-Verzeichnis. Prüfen ob das der Fall ist — wenn ja, ist das SingletonLock-Problem ein Bug in chromiumoxide. Die aktuelle Fehlermeldung deutet darauf hin, dass das feste Verzeichnis `/tmp/chromiumoxide-runner/` verwendet wird, was bei mehreren parallelen Runs kollidiert.

---

## 11. Migrationsplan

### Phase 1: Detection Refactor

**Ziel:** Neue Browser-Erkennung mit Registry-Pattern, Edge-Support, bessere Fehlermeldungen.

**Änderungen:**
- `browser/types.rs` anlegen (BrowserKind, DetectedBrowser, BrowserSource, etc.)
- `browser/registry.rs` anlegen (Pfade pro Plattform und BrowserKind)
- `browser/detection.rs` umschreiben (nutzt Registry, findet Edge + Ungoogled Chromium)
- `browser/resolver.rs` anlegen (priorisierte Auswahl)
- `cli/args.rs`: `--browser-path` als Alias für `--chrome-path` (beide akzeptieren)
- `cli/args.rs`: `--browser <kind>` Flag
- Env-Var `AUDITMYSITE_BROWSER` einführen, `CHROME_PATH` weiter akzeptieren
- Fehlermeldung bei ChromeNotFound verbessern

**Risiken:**
- Detection könnte auf manchen Systemen andere Browser finden als vorher → `--browser chrome` als Escape-Hatch
- Edge-Kompatibilität mit chromiumoxide: Sollte funktionieren (gleiche CDP-API), aber testen

**Tests:**
- Unit-Tests für Registry (Pfade pro Plattform)
- Unit-Tests für Resolver (Prioritätsreihenfolge)
- Integrationstest: `auditmysite browser detect` auf CI
- Manuell: Auf macOS mit Chrome + Edge testen

### Phase 2: Explicit Installer

**Ziel:** `auditmysite browser install` als expliziter Befehl. Kein stiller Download mehr.

**Änderungen:**
- `cli/browser.rs` anlegen (Subcommands: detect, install, remove, path)
- `browser/installer.rs` umschreiben:
  - Version von API statt hardcoded
  - Zielverzeichnis: `~/.auditmysite/browsers/chrome-for-testing/`
  - Progress-Bar mit `indicatif`
  - Quarantine-Handling auf macOS
- `browser/manager.rs`: Auto-Download-Fallback entfernen
- `browser/manager.rs`: Wenn kein Browser → `ChromeNotFound` Error (kein Download)

**Risiken:**
- Breaking Change: User die sich auf Auto-Download verlassen, bekommen jetzt einen Fehler
- Mitigation: Klare Fehlermeldung mit `auditmysite browser install` Hinweis

**Tests:**
- `browser install` in CI testen (Download + Extraktion + Lauffähigkeit)
- `browser remove` testen
- Test: Audit ohne Browser → Fehler mit korrekter Meldung
- Test: Audit nach `browser install` → funktioniert

### Phase 3: CLI Doctor

**Ziel:** `auditmysite doctor` für Systemdiagnose.

**Änderungen:**
- `cli/doctor.rs` anlegen
- `doctor.rs` im Root anlegen (Logik)
- Checks: Browser vorhanden, executable, Version, CDP-fähig, Netzwerk, Plattenplatz

**Risiken:** Gering. Rein additives Feature.

**Tests:**
- Doctor auf CI: Alle Checks grün mit installiertem Chrome
- Doctor ohne Browser: Browser-Check rot, Rest grün

### Phase 4: Headless-Shell Mode

**Ziel:** `--fast` Mode mit `chrome-headless-shell`.

**Änderungen:**
- `browser/installer.rs`: `InstallTarget::HeadlessShell` Support
- `browser/launcher.rs`: Spezielle Flags für headless-shell
- `cli/args.rs`: `--fast` Flag
- Prüfen: Ist AXTree über headless-shell verfügbar?
  - Wenn ja: Fast-Mode ist vollwertig
  - Wenn nein: Fast-Mode nur für Performance/SEO/Security, kein WCAG-Audit

**Risiken:**
- AXTree möglicherweise nicht verfügbar → Feature eingeschränkt nutzbar
- Muss empirisch getestet werden

**Tests:**
- `browser install --headless-shell` testen
- Audit mit `--fast` gegen bekannte Seite
- Vergleich: Standard vs. Fast Ergebnisse

### Phase 5: Cleanup Legacy

**Ziel:** Alten Auto-Download-Code und `~/.auditmysite/chromium/` aufräumen.

**Änderungen:**
- `--chrome-path` als Deprecated markieren (→ `--browser-path`)
- `CHROME_PATH` als Deprecated markieren (→ `AUDITMYSITE_BROWSER`)
- `~/.auditmysite/chromium/` Migration: Beim Start prüfen und Hinweis geben
- Alten Installer-Code entfernen
- `DetectionMethod::AutoDownload` entfernen

**Risiken:**
- User mit altem Setup müssen `browser install` ausführen oder System-Browser nutzen
- Mitigation: Klare Migrationsmeldung

**Tests:**
- Test: Alter `chromium/`-Ordner existiert → Migrationsmeldung
- Test: `--chrome-path` funktioniert noch (Deprecated-Warnung)

---

## 12. Konkrete Code-Skizzen

### Browser Registry

```rust
// browser/registry.rs

use super::types::BrowserKind;

/// Ein Eintrag im Browser-Registry
pub struct RegistryEntry {
    pub kind: BrowserKind,
    pub path: &'static str,
}

/// Bekannte Browser-Pfade für die aktuelle Plattform, in Prioritätsreihenfolge
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

/// Suchbegriffe für `which`/`where` pro BrowserKind
pub fn which_names(kind: BrowserKind) -> &'static [&'static str] {
    match kind {
        BrowserKind::Chrome => &["google-chrome", "google-chrome-stable"],
        BrowserKind::Edge => &["microsoft-edge", "microsoft-edge-stable"],
        BrowserKind::Chromium => &["chromium", "chromium-browser"],
        _ => &[],
    }
}

/// Prioritätsreihenfolge der BrowserKinds für die automatische Suche
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

/// CLI-Optionen die den Browser betreffen
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

/// Findet den besten verfügbaren Browser
pub fn resolve_browser(opts: &BrowserCliOptions) -> Result<ResolvedBrowser, BrowserResolveError> {
    let mode = if opts.strict {
        BrowserMode::Strict
    } else if opts.fast {
        BrowserMode::Fast
    } else {
        BrowserMode::Standard
    };

    let mut all_candidates = Vec::new();

    // 1. Expliziter Pfad (höchste Prio)
    if let Some(ref path_str) = opts.browser_path {
        let path = PathBuf::from(path_str);
        let browser = validate_browser(&path, BrowserKind::Custom, BrowserSource::CliFlag)?;
        info!("Using specified browser: {} v{}",
            browser.kind.display_name(),
            browser.version.as_deref().unwrap_or("unknown"));
        return Ok(ResolvedBrowser { browser, mode, all_candidates: vec![] });
    }

    // 2. Environment-Variable
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

    // 3. Spezifischer BrowserKind angefordert?
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

    // 4. System-Scan
    all_candidates = detect_all_browsers();
    debug!("Found {} browser candidates", all_candidates.len());

    // Filter nach gewünschtem Kind
    let candidates: Vec<&DetectedBrowser> = if let Some(kind) = filter_kind {
        all_candidates.iter().filter(|b| b.kind == kind).collect()
    } else {
        all_candidates.iter().collect()
    };

    // Besten Kandidaten wählen (erster in Prioritätsreihenfolge)
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

    // 5. Managed Install prüfen (nicht im Strict-Mode)
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

    // 6. Nichts gefunden
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

### Detection (alle Browser scannen)

```rust
// browser/detection.rs

use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::debug;

use super::registry::{self, RegistryEntry};
use super::types::*;

/// Findet alle installierten Browser auf dem System
pub fn detect_all_browsers() -> Vec<DetectedBrowser> {
    let mut found = Vec::new();

    // 1. Bekannte Pfade scannen
    for entry in registry::system_browser_paths() {
        let path = PathBuf::from(entry.path);
        if path.exists() {
            if let Ok(browser) = validate_browser(&path, entry.kind, BrowserSource::SystemPath) {
                // Duplikate vermeiden (gleicher Pfad)
                if !found.iter().any(|b: &DetectedBrowser| b.path == browser.path) {
                    debug!("Found {} at {}", entry.kind.display_name(), entry.path);
                    found.push(browser);
                }
            }
        }
    }

    // 2. which/where für jeden BrowserKind
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

    // Sortieren nach Priorität (Chrome > Edge > Chromium)
    let order = registry::search_order();
    found.sort_by_key(|b| {
        order.iter().position(|k| *k == b.kind).unwrap_or(usize::MAX)
    });

    found
}

/// Validiert eine Browser-Binary
pub fn validate_browser(
    path: &Path,
    kind: BrowserKind,
    source: BrowserSource,
) -> Result<DetectedBrowser, DetectionError> {
    // Existenz
    if !path.exists() {
        return Err(DetectionError::NotFound(path.to_path_buf()));
    }

    // Executable-Permission (Unix)
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
                // Extrahiere Versionsnummer: "Google Chrome 131.0.6778.108" → "131.0.6778.108"
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

            // Managed installs prüfen
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
            // Implementierung: Verzeichnisse löschen
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

    // 1. Browser-Check
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
        // Einfacher Check: 500 MB frei?
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

## 13. Entscheidungsempfehlung

### Was sollte der Default sein?

**System-Browser automatisch erkennen und verwenden.** Chrome > Edge > Chromium. Kein Download, kein Installieren, keine Überraschungen. Die meisten Entwickler und CI-Systeme haben Chrome oder Edge installiert. Das funktioniert sofort.

### Was sollte optional sein?

- `auditmysite browser install` für User ohne System-Browser (selten auf macOS/Windows, häufiger auf minimalen Linux-Servern)
- `--fast` Mode mit headless-shell (nach Validierung dass AXTree damit funktioniert)
- `--browser edge` für User die Edge statt Chrome bevorzugen
- `--strict` für CI-Pipelines die keine Fallbacks wollen

### Was sollte ich vermeiden?

1. **Keinen stillen Auto-Download.** Das aktuelle Verhalten (Chromium still herunterladen) ist das Kernproblem. Es überrascht User, verursacht Firewall-Probleme, und ist schwer zu debuggen.
2. **Keinen eigenen Browser bundlen.** In der Formula, im Release-Archiv, nirgendwo. Der Browser ist eine Laufzeit-Dependency, keine Build-Dependency.
3. **Keine harte Browser-Dependency in Homebrew.** `depends_on cask: "google-chrome"` erzeugt mehr Probleme als es löst.
4. **Kein Version-Pinning im Installer.** Die aktuelle hardcoded Version (131.0.6778.108) wird schnell veralten. Die Chrome for Testing API liefert die aktuelle Stable.

### Kompromisse

| Entscheidung | Vorteil | Nachteil |
|---|---|---|
| Kein Auto-Download | Keine Überraschungen, keine Firewall-Probleme | User ohne Chrome muss `browser install` ausführen |
| System-Chrome bevorzugen | Signiert, keine Gatekeeper-Probleme, immer aktuell | Nicht unter unserer Kontrolle (Version) |
| Edge als Fallback | Auf Windows fast immer vorhanden | Minimaler Zusatz-Testaufwand |
| Managed Install optional | User hat explizite Kontrolle | Ein zusätzlicher Schritt für User ohne System-Browser |
| Kein Version-Pinning | Immer aktuelle Version | Theoretisch könnte eine Chrome-Version einen Bug haben |

### Empfohlene Umsetzungsreihenfolge

1. **Phase 1 (Detection Refactor)** → größter Impact, macht alles robuster
2. **Phase 2 (Explicit Installer)** → entfernt den stillen Download
3. **Phase 5 (Cleanup)** → kann direkt nach Phase 2 passieren
4. **Phase 3 (Doctor)** → nice-to-have, reduziert Support-Anfragen
5. **Phase 4 (Headless-Shell)** → nur wenn AXTree damit funktioniert, sonst niedrige Prio
