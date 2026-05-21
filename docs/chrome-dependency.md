# Chrome/Chromium Abhängigkeit in AuditMySite

## Warum Chrome?

AuditMySite basiert fundamental auf Chrome/Chromium als Rendering-Engine. Der Grund: **WCAG-Accessibility-Prüfungen erfordern eine echte Browser-Rendering-Engine**, weil:

1. **Accessibility Tree (AXTree):** Chrome baut intern einen vollständigen Accessibility Tree auf - eine semantische Repräsentation der Seite, wie sie von Screenreadern gesehen wird. Diesen Baum kann man **nur über einen echten Browser** erhalten. Es gibt keine Bibliothek, die aus reinem HTML einen korrekten AXTree erzeugt.

2. **Computed Styles:** Für Kontrast-Prüfungen (WCAG 1.4.3) müssen die tatsächlich berechneten CSS-Werte bekannt sein - nach Kaskadierung, Vererbung, Media Queries, CSS-Variablen, etc. Das geht nur mit einer echten CSS-Engine.

3. **JavaScript-Rendering:** Viele moderne Websites (SPAs, React, Vue, etc.) rendern ihren Inhalt erst via JavaScript. Ohne echte JS-Engine sieht man nur ein leeres `<div id="root">`.

4. **Core Web Vitals:** Performance-Metriken wie LCP, FCP, CLS werden vom Browser selbst gemessen und über die Performance API bereitgestellt.

---

## Wie Chrome eingebunden wird

### Rust-Crate: `chromiumoxide`

AuditMySite verwendet die Rust-Library [`chromiumoxide`](https://github.com/nicholasgasior/chromiumoxide) zur Kommunikation mit Chrome über das **Chrome DevTools Protocol (CDP)**. Dieses Protokoll ist dasselbe, das Chrome DevTools (F12) intern verwendet.

### Architektur

```
auditmysite (Rust-Prozess)
    │
    │ startet Chrome als Kindprozess
    ▼
Chrome/Chromium (headless)
    │
    │ CDP-Verbindung über WebSocket (localhost)
    │ Port wird automatisch zugewiesen
    ▼
Zielwebsite (wird im Chrome-Tab geladen)
```

**Wichtig:** Die CDP-Verbindung läuft über `localhost` (127.0.0.1) via WebSocket. Chrome öffnet dafür einen zufälligen lokalen Port für das Remote-Debugging-Protokoll.

---

## Chrome-Lebenszyklus

### 1. Chrome finden (`src/browser/resolver.rs`)

Suchreihenfolge:
1. `--browser-path` CLI-Argument (vom Benutzer angegeben, Alias: `--chrome-path`)
2. `AUDITMYSITE_BROWSER` Umgebungsvariable
3. System-Chrome an bekannten Pfaden:
   - macOS: `/Applications/Google Chrome.app/Contents/MacOS/Google Chrome`
   - Linux: `/usr/bin/google-chrome`, `/usr/bin/chromium-browser`, etc.
   - Windows: `Program Files/Google/Chrome/Application/chrome.exe`
4. Managed Install unter `~/.auditmysite/browsers/`

`CHROME_PATH` wird weiterhin als Kompatibilitätsquelle unterstützt, die Benutzeroberfläche des Projekts dokumentiert aber primär `--browser-path` und `AUDITMYSITE_BROWSER`.

### 2. Chrome installieren (`src/browser/installer.rs`)

Falls kein Chrome gefunden wird, muss der Benutzer explizit installieren:
```bash
auditmysite browser install
```

- Download von "Chrome for Testing" (stabile Builds von Google)
- Download-URL: `storage.googleapis.com/chrome-for-testing-public/...`
- Zielverzeichnis: `~/.auditmysite/browsers/`
- Plattformspezifisch (macOS arm64/x64, Linux x64, Windows x64)
- Einmaliger Download (~150-200MB), danach aus Cache

**Hinweis:** Es gibt keinen automatischen Download mehr. Der Benutzer entscheidet selbst, wann ein Browser installiert wird.

### 3. Chrome starten (`src/browser/manager.rs`)

Chrome wird als **Kindprozess** gestartet mit diesen Flags:

```
--headless                       # Kein sichtbares Fenster
--no-first-run                   # Keine Ersteinrichtungs-Dialoge
--no-default-browser-check       # Keine Default-Browser-Prüfung
--disable-extensions             # Keine Extensions laden
--disable-background-networking  # Kein Hintergrund-Netzwerk
--disable-sync                   # Kein Google-Sync
--disable-translate              # Kein Google Translate
--disable-features=TranslateUI   # Kein Translate-UI
--metrics-recording-only         # Keine Telemetrie senden
--mute-audio                     # Kein Audio
--disable-infobars               # Keine Infobars
--disable-popup-blocking         # Keine Popup-Blockierung
--disable-gpu                    # Kein GPU (headless)
--disable-software-rasterizer    # Kein Software-Rendering
--window-size=1920,1080          # Konsistenter Viewport
```

Optionale Flags:
```
--no-sandbox                     # Für Docker/Root (nur mit --no-sandbox CLI-Flag)
--disable-dev-shm-usage          # Für Docker (shared memory)
--blink-settings=imagesEnabled=false  # Bilder deaktivieren (--disable-images)
```

### 4. Seiten verarbeiten (`src/browser/pool.rs`)

- **BrowserPool:** Ein einzelner Chrome-Prozess, mehrere Tabs (Pages)
- Konfigurierbare Parallelität: 1-10 gleichzeitige Tabs (default: 3)
- Pages werden nach Gebrauch auf `about:blank` zurückgesetzt und wiederverwendet
- Semaphore-basierte Zugriffskontrolle

### 5. Chrome beenden

- Alle Pages werden geschlossen
- Chrome-Prozess wird beendet (Drop-Handler)
- Bei unvorhergesehenem Abbruch: Chrome wird im Hintergrund gekillt (Warnung im Log)

---

## Was über CDP kommuniziert wird

### Accessibility Tree Extraktion (`src/accessibility/extractor.rs`)

- **CDP-Befehl:** `Accessibility.getFullAXTree`
- **Daten:** Kompletter AXTree mit allen Nodes (Rollen, Namen, Properties, Eltern-Kind-Beziehungen)
- **Richtung:** Chrome → auditmysite (nur Lesen)

### Computed Styles (`src/accessibility/styles.rs`)

- **Methode:** JavaScript-Evaluation über CDP (`Runtime.evaluate`)
- **Daten:** `window.getComputedStyle()` für Textelemente (color, background-color, font-size, font-weight)
- **Zweck:** Kontrast-Prüfung (WCAG 1.4.3)

### Performance Metriken (`src/performance/vitals.rs`)

- **CDP-Befehl:** `Performance.getMetrics`
- **JS-Evaluation:** `performance.getEntriesByType('paint')`, `performance.getEntriesByType('largest-contentful-paint')`, `performance.getEntriesByType('layout-shift')`, `performance.getEntriesByType('navigation')`
- **Daten:** FCP, LCP, CLS, TTFB, DOM-Nodes, JS-Heap-Size

### SEO/Security/Mobile Analyse

- **Methode:** JavaScript-Evaluation über CDP
- **Daten:** Meta-Tags, Headings, Schema.org, Social-Tags aus dem DOM
- **Security:** Eigener HTTP-Request (reqwest) für Header-Analyse, nicht über Chrome

### Navigation

- **CDP:** `Page.navigate`, `Page.waitForNavigation`
- **Timeout:** Konfigurierbar (default: 30s), mit 1 Retry

---

## Netzwerkverbindungen

### Verbindungen die Chrome macht:

| Verbindung | Wohin | Warum | Kontrolle |
|---|---|---|---|
| **Ziel-Website** | URL die auditiert wird | Seite laden und rendern | Vom Benutzer gewünscht |
| **CDP WebSocket** | localhost (zufälliger Port) | Kommunikation auditmysite ↔ Chrome | Nur lokal |
| **Google-Services** | *.google.com (potenziell) | Chrome-interne Requests (SafeBrowsing, Updates, etc.) | Teilweise durch Flags unterdrückt |

### Verbindungen die auditmysite direkt macht (ohne Chrome):

| Verbindung | Wohin | Warum |
|---|---|---|
| **Sitemap fetch** | Ziel-Domain | XML-Sitemap herunterladen |
| **Security Headers** | Ziel-Domain | HTTP-Header analysieren |
| **Browser Download** | storage.googleapis.com | Browser-Download (nur bei explizitem `browser install`) |

---

## Positive Aspekte

1. **Genauigkeit:** Der native AXTree ist die genaueste Quelle für Accessibility-Daten - genauer als jede HTML-Parsing-Bibliothek
2. **Real-World-Rendering:** Seiten werden exakt so analysiert, wie ein echter Browser sie rendert (inkl. JavaScript, CSS, Lazy Loading)
3. **Bewährte Technologie:** CDP ist ein stabiles, gut dokumentiertes Protokoll, das auch von Playwright, Puppeteer und Lighthouse verwendet wird
4. **Headless:** Kein sichtbares Fenster, kein GUI nötig
5. **Plattformübergreifend:** Funktioniert auf macOS, Linux, Windows
6. **Managed Install:** Falls kein Chrome installiert ist, kann über `auditmysite browser install` eine isolierte Kopie heruntergeladen werden
7. **Performance-Flags:** Viele unnötige Chrome-Features werden deaktiviert (Extensions, Sync, Translate, GPU, Audio)

---

## Negative Aspekte / Probleme

### 1. Firewall-Alerts
- Chrome wird als **separater Prozess** gestartet und macht ausgehende Netzwerkverbindungen
- macOS-Firewall erkennt das als neuen Prozess der ins Internet will → **Firewall-Dialog bei jedem Start**
- Selbst mit `--disable-background-networking` kann Chrome interne Requests machen
- Bei managed Install: Die heruntergeladene Chromium-Binary ist unsigniert → zusätzliche Gatekeeper-Warnung auf macOS

### 2. Ressourcenverbrauch
- Chrome ist ein **schwerer Prozess** (~100-300MB RAM pro Instanz)
- Bei Batch-Audits mit mehreren Tabs: Speicherverbrauch multipliziert sich
- CPU-Last beim Rendern komplexer Seiten

### 3. Abhängigkeit von einem externen Prozess
- Chrome muss auf dem System vorhanden sein oder explizit installiert werden
- Chrome-Versionen können sich ändern und Inkompatibilitäten verursachen
- In CI/CD-Umgebungen: Chrome muss extra installiert oder mitgeliefert werden

### 4. Security-Implikationen
- Chrome läuft als Kindprozess mit den **Rechten des aufrufenden Benutzers**
- Der CDP-WebSocket-Port ist theoretisch von anderen lokalen Prozessen erreichbar
- `--no-sandbox` (für Docker) reduziert die Isolation erheblich
- Die heruntergeladene Chromium-Binary wird nicht kryptographisch verifiziert (kein Hash-Check)

### 5. Portabilität
- Binäre Abhängigkeit: auditmysite allein ist nicht ausreichend, Chrome muss zusätzlich da sein
- Auf Servern/Containern muss Chrome explizit installiert werden
- Auf headless Linux-Systemen: Oft fehlende Shared Libraries (libX11, libatk, etc.)

### 6. Unkontrollierbare Chrome-Hintergrundaktivitäten
- Trotz Flags wie `--disable-background-networking` und `--metrics-recording-only` kann Chrome:
  - DNS-Prefetch machen
  - Certificate Revocation Lists prüfen
  - Safe Browsing Daten abrufen
  - Component Updater triggern
- Diese Aktivitäten erzeugen die Firewall-Alerts

### 7. Prozessmanagement
- Bei Crashes oder Timeouts bleiben manchmal **Zombie-Chrome-Prozesse** zurück
- Der Drop-Handler killt Chrome im Hintergrund, aber nicht immer zuverlässig
- Mehrere gleichzeitige auditmysite-Instanzen → mehrere Chrome-Instanzen

---

## Zusammenfassung

Chrome/Chromium ist aktuell eine **technisch notwendige Abhängigkeit**, weil der native AXTree die Grundlage für korrekte WCAG-Prüfungen ist. Die Alternative wäre reines HTML-Parsing, was aber bei JavaScript-lastigen Seiten und komplexem CSS versagt.

Die Hauptprobleme in der Praxis sind:
- **Firewall-Alerts** durch Chrome als separater Prozess
- **Ressourcenverbrauch** (RAM/CPU)
- **Deployment-Komplexität** (Chrome muss überall vorhanden sein)
- **Unkontrollierbare Netzwerkaktivitäten** von Chrome selbst
