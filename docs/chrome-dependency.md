# Chrome/Chromium Dependency in AuditMySite

## Why Chrome?

AuditMySite is fundamentally based on Chrome/Chromium as its rendering engine. The reason: **WCAG accessibility checks require a real browser rendering engine**, because:

1. **Accessibility Tree (AXTree):** Chrome internally builds a complete accessibility tree - a semantic representation of the page as screen readers see it. This tree can **only be obtained through a real browser**. There is no library that produces a correct AXTree from plain HTML.

2. **Computed Styles:** Contrast checks (WCAG 1.4.3) need the actually computed CSS values - after cascading, inheritance, media queries, CSS variables, etc. That only works with a real CSS engine.

3. **JavaScript Rendering:** Many modern websites (SPAs, React, Vue, etc.) render their content only via JavaScript. Without a real JS engine you only see an empty `<div id="root">`.

4. **Core Web Vitals:** Performance metrics such as LCP, FCP, CLS are measured by the browser itself and exposed through the Performance API.

---

## How Chrome Is Integrated

### Rust crate: `chromiumoxide`

AuditMySite uses the Rust library [`chromiumoxide`](https://github.com/nicholasgasior/chromiumoxide) to communicate with Chrome over the **Chrome DevTools Protocol (CDP)**. This protocol is the same one Chrome DevTools (F12) uses internally.

### Architecture

```
auditmysite (Rust process)
    │
    │ starts Chrome as a child process
    ▼
Chrome/Chromium (headless)
    │
    │ CDP connection over WebSocket (localhost)
    │ Port is assigned automatically
    ▼
Target website (loaded in the Chrome tab)
```

**Important:** The CDP connection runs over `localhost` (127.0.0.1) via WebSocket. Chrome opens a random local port for the remote debugging protocol.

---

## Chrome Lifecycle

### 1. Finding Chrome (`src/browser/resolver.rs`)

Search order:
1. `--browser-path` CLI argument (given by the user, alias: `--chrome-path`)
2. `AUDITMYSITE_BROWSER` environment variable
3. System Chrome at known paths:
   - macOS: `/Applications/Google Chrome.app/Contents/MacOS/Google Chrome`
   - Linux: `/usr/bin/google-chrome`, `/usr/bin/chromium-browser`, etc.
   - Windows: `Program Files/Google/Chrome/Application/chrome.exe`
4. Managed install under `~/.auditmysite/browsers/`

`CHROME_PATH` is still supported as a compatibility source, but the project's user interface primarily documents `--browser-path` and `AUDITMYSITE_BROWSER`.

### 2. Install Chrome (`src/browser/installer.rs`)

Nothing is downloaded without consent:

- `auditmysite browser install [--version V] [--force] [--offline] [--headless-shell]` is the explicit request.
- `auditmysite browser ensure --yes` installs only if no browser is found (idempotent); without `--yes` it asks in an interactive terminal and otherwise fails with a deterministic error. This is the opt-in for scripts and CI.
- An interactive audit that finds no browser asks once (source, destination, approximate size, decline path). Non-interactive runs (no TTY, `CI` set, `--quiet`) never ask and never download.

Platforms: macOS ARM64, Linux x86_64 and Linux ARM64 (Chrome for Testing `linux-arm64` exists from 153.0.8001.0; older versions fail with "not published"). Windows x86_64 uses an existing Microsoft Edge only and is never offered a download. macOS Intel and Windows ARM64 are not supported. No browser is bundled in the binary or release archive.

Source: the version and the exact per-platform URL come from the publisher's manifest (`googlechromelabs.github.io/chrome-for-testing/*.json`); only HTTPS to `googlechromelabs.github.io` (manifest) and `storage.googleapis.com/chrome-for-testing-public/` (archive) is accepted, including redirects. There is no hard-coded fallback version: offline, a previously validated managed browser is used, otherwise setup reports that it needs network access.

Transaction (`install_txn.rs`): exclusive file lock (a second install fails fast, the lock dies with the process) → recovery of leftovers → bounded download (600 MB, 20 min total, 30 s stall, announced length must match) into `~/.auditmysite/browsers/.staging-*` → safe extraction (no path escape, no link leaving the tree, no overwrite, entry and size bounds) → binary present, `--version` matches, headless start and one CDP command → rename into place, previous installation kept aside and renamed back if that fails. A failed install leaves the previous one in place. `install.json` records what was verified.

**Integrity limits (researched 2026-11-02).** The Chrome for Testing manifests list only `platform` and `url`: no checksum, no signature, no `.sha256`/`.sig` next to the archives. The bucket sends `x-goog-hash` (MD5/CRC32C) and an ETag, but from the same origin as the bytes, so they cannot authenticate the publisher; a hash computed from the download proves nothing either. auditmysite therefore does **not** claim publisher verification: trust rests on TLS to the two fixed hosts, the exact manifest entry, and the checks above. A compromise of those hosts is not detected. If upstream publishes a digest or signature, verification belongs in `install_plan.rs`. There is no downgrade path: a missing platform, version or asset is an error, never another platform or an older build.

**macOS.** The installer writes files itself, so they carry no `com.apple.quarantine` attribute and nothing is stripped (the old blanket `xattr -rd` is gone). If Gatekeeper still blocks the app the startup check fails, nothing is bypassed, and the install is rolled back.

### 3. Starting Chrome (`src/browser/manager.rs`)

Chrome is started as a **child process** with these flags:

```
--headless                       # No visible window
--no-first-run                   # No first-run setup dialogs
--no-default-browser-check       # No default browser check
--disable-extensions             # Do not load extensions
--disable-background-networking  # No background networking
--disable-sync                   # No Google sync
--disable-translate              # No Google Translate
--disable-features=TranslateUI   # No Translate UI
--metrics-recording-only         # Do not send telemetry
--mute-audio                     # No audio
--disable-infobars               # No infobars
--disable-popup-blocking         # No popup blocking
--disable-gpu                    # No GPU (headless)
--disable-software-rasterizer    # No software rendering
--window-size=1920,1080          # Consistent viewport
```

Optional flags:
```
--no-sandbox                     # For Docker/root (only with the --no-sandbox CLI flag)
--disable-dev-shm-usage          # For Docker (shared memory)
--blink-settings=imagesEnabled=false  # Disable images (--disable-images)
```

### 4. Processing Pages (`src/browser/pool.rs`)

- **BrowserPool:** A single Chrome process, multiple tabs (pages)
- Configurable parallelism: 1-10 concurrent tabs (default: 3)
- Pages are reset to `about:blank` after use and reused
- Semaphore-based access control

### 5. Shutting Down Chrome

- All pages are closed
- The Chrome process is terminated (drop handler)
- On unexpected termination: Chrome is killed in the background (warning in the log)

---

## What Is Communicated over CDP

### Accessibility Tree Extraction (`src/accessibility/extractor.rs`)

- **CDP command:** `Accessibility.getFullAXTree`
- **Data:** Complete AXTree with all nodes (roles, names, properties, parent-child relationships)
- **Direction:** Chrome → auditmysite (read-only)

### Computed Styles (`src/accessibility/dom_document.rs`)

- **CDP command:** `DOMSnapshot.captureSnapshot` with the required computed style values, plus a
  `Runtime.evaluate` pass (`elementFromPoint`) for covered text
- **Data:** Colors, effective background, font, borders, visually hidden text
- **Purpose:** the shared presentation rules, including contrast (WCAG 1.4.3, 1.4.6)

### Performance Metrics (`src/performance/vitals.rs`)

- **CDP command:** `Performance.getMetrics`
- **JS evaluation:** `performance.getEntriesByType('paint')`, `performance.getEntriesByType('largest-contentful-paint')`, `performance.getEntriesByType('layout-shift')`, `performance.getEntriesByType('navigation')`
- **Data:** FCP, LCP, CLS, TTFB, DOM nodes, JS heap size

### SEO/Security/Mobile Analysis

- **Method:** JavaScript evaluation over CDP
- **Data:** Meta tags, headings, Schema.org, social tags from the DOM
- **Security:** Own HTTP request (reqwest) for header analysis, not through Chrome

### Navigation

- **CDP:** `Page.navigate`, `Page.waitForNavigation`
- **Timeout:** Configurable (default: 30s), with 1 retry

---

## Network Connections

### Connections Chrome makes:

| Connection | Destination | Why | Control |
|---|---|---|---|
| **Target website** | URL being audited | Load and render the page | Requested by the user |
| **CDP WebSocket** | localhost (random port) | Communication auditmysite ↔ Chrome | Local only |
| **Google services** | *.google.com (potentially) | Chrome-internal requests (SafeBrowsing, updates, etc.) | Partly suppressed by flags |

### Connections auditmysite makes directly (without Chrome):

| Connection | Destination | Why |
|---|---|---|
| **Sitemap fetch** | Target domain | Download XML sitemap |
| **Security Headers** | Target domain | Analyze HTTP headers |
| **Browser Download** | storage.googleapis.com | Browser download (only on explicit `browser install`) |

---

## Positive Aspects

1. **Accuracy:** The native AXTree is the most accurate source for accessibility data - more accurate than any HTML parsing library
2. **Real-world rendering:** Pages are analyzed exactly as a real browser renders them (including JavaScript, CSS, lazy loading)
3. **Proven technology:** CDP is a stable, well-documented protocol that is also used by Playwright, Puppeteer and Lighthouse
4. **Headless:** No visible window, no GUI needed
5. **Cross-platform:** Works on macOS, Linux, Windows
6. **Managed install:** If no Chrome is installed, an isolated copy can be downloaded via `auditmysite browser install`
7. **Performance flags:** Many unnecessary Chrome features are disabled (extensions, sync, translate, GPU, audio)

---

## Negative Aspects / Problems

### 1. Firewall Alerts
- Chrome is started as a **separate process** and makes outgoing network connections
- The macOS firewall sees this as a new process that wants to reach the internet → **firewall dialog on every start**
- Even with `--disable-background-networking`, Chrome can make internal requests
- With a managed install: The downloaded Chromium binary is unsigned → additional Gatekeeper warning on macOS

### 2. Resource Consumption
- Chrome is a **heavy process** (~100-300MB RAM per instance)
- For batch audits with multiple tabs: memory consumption multiplies
- CPU load when rendering complex pages

### 3. Dependency on an External Process
- Chrome must be present on the system or explicitly installed
- Chrome versions can change and cause incompatibilities
- In CI/CD environments: Chrome must be installed separately or shipped along

### 4. Security Implications
- Chrome runs as a child process with the **privileges of the calling user**
- The CDP WebSocket port is theoretically reachable by other local processes
- `--no-sandbox` (for Docker) significantly reduces isolation
- The downloaded Chromium binary is not cryptographically verified (no hash check)

### 5. Portability
- Binary dependency: auditmysite alone is not sufficient, Chrome must be present as well
- On servers/containers Chrome must be installed explicitly
- On headless Linux systems: Often missing shared libraries (libX11, libatk, etc.)

### 6. Uncontrollable Chrome Background Activity
- Despite flags such as `--disable-background-networking` and `--metrics-recording-only`, Chrome can:
  - Perform DNS prefetching
  - Check certificate revocation lists
  - Fetch Safe Browsing data
  - Trigger the component updater
- These activities cause the firewall alerts

### 7. Process Management
- On crashes or timeouts, **zombie Chrome processes** sometimes remain
- The drop handler kills Chrome in the background, but not always reliably
- Multiple concurrent auditmysite instances → multiple Chrome instances

---

## Summary

Chrome/Chromium is currently a **technically necessary dependency**, because the native AXTree is the basis for correct WCAG checks. The alternative would be plain HTML parsing, which fails on JavaScript-heavy pages and complex CSS.

The main problems in practice are:
- **Firewall alerts** caused by Chrome as a separate process
- **Resource consumption** (RAM/CPU)
- **Deployment complexity** (Chrome must be present everywhere)
- **Uncontrollable network activity** of Chrome itself
