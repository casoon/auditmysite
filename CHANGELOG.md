# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.25.0] - 2026-05-21

### Added
- **`tool_version`** als Top-Level-Feld im JSON-Report — parallel zu `schema_version`/`report_type` (#247)
- **`occurrence_counts`** auf Page- und Summary-Ebene: Element-Occurrences je Severity, getrennt von den Finding-Counts in `severity_counts` (#249)
- **Sitemap-Aggregat**: `violated_rule_count` (dedupliziert über alle Pages) und `top_recurring_rules` (max. 10 häufigste WCAG-Verstöße) im Batch-Summary (#253)
- **`Gesamtscore Website` / `Overall score`** als eigener Metrik-Eintrag im Single-Report-Summary (#251)

### Changed
- **Performance-Score-Kalibrierung (#236)**: Der gemeldete Performance-Score basiert jetzt auf dem `LhMobile`-Profil (Lighthouse Mobile Preset mit Netzwerk- und CPU-Throttling) statt auf der unthrottled Desktop/Mobile-Pass-Messung. Damit produziert die Bewertung kontinuierliche Verteilungen statt pauschal 100 für schnelle Dev-Netz-Messungen. Strukturelle Performance-Daten (render_blocking, content_weight, third_party, …) bleiben aus dem unthrottled Pass. Fällt die LhMobile-LCP-Messung aus, bleibt der unthrottled Score erhalten.
- **Performance-Score-Gewichtung (#248)**: Lighthouse-v10/v11-Gewichte (FCP 10 %, LCP 25 %, TBT 30 %, CLS 25 %) mit log-normalen Score-Kurven (p10/p50-Kalibrierung); CLS > 0.5 wird hart auf 0 gecappt. Profile mit LCP > 6–7 s landen jetzt korrekt unter 50 statt in den 70ern.
- **`severity_counts` zählt Findings** (eine Zeile pro Regel/Severity) statt Element-Occurrences; die alte Occurrence-Summierung steht weiterhin in `occurrence_counts` (#249). SiteState-, Risk- und UX/Journey-Schwellen nutzen weiterhin Occurrence-Mengen.
- **Finding-`title`** nutzt den kundenorientierten Taxonomie-Titel statt des rohen WCAG-Engine-Namens — JSON und PDF nennen dasselbe Label (#252)
- **PDF-„Gesamtscore"** zeigt `overall_score` (modulgewichtet) statt `accessibility_score` (#251)
- **Risk-Level**: `legal_flags > 0` oder `blocking_issues ≥ 1` heben das Level mindestens auf Medium; Medium-Summary nennt BFSG-Relevanz und Blocker-Anzahl konsistent (#250)
- **Pass-Kriterium** (`passed_url_count`): accessibility_score ≥ 80, keine Critical-Findings und keine WCAG-Level-A High/Critical-Findings (#253)

### Fixed
- **`detail.fix_guidance`** ist jetzt immer im JSON präsent (leeres Array bei 0 Findings) statt zu fehlen (#253)

## [0.21.0] - 2026-05-18

### Removed
- `ChromiumInstaller` struct entfernt — war interner Wrapper für den Auto-Download-Fallback
- Auto-Download-Fallback in `BrowserManager::with_options()` entfernt: kein stilles Herunterladen von Chrome mehr, wenn kein System-Browser gefunden wird; `resolve_browser()` schlägt jetzt direkt mit `ChromeNotFound` fehl
- `pub use installer::ChromiumInstaller` aus `browser/mod.rs` entfernt

`BrowserInstaller` und `auditmysite browser install` bleiben unverändert — explizites Opt-in für Nutzer ohne System-Chrome.

---

## [0.19.0] - 2026-05-18

### Added
- **Unified Report Envelope v2.0**: einheitliches JSON-Schema für single + batch (`schema_version`, `report_type`, `summary`, `pages[]`, `pages[i].detail`) — Breaking Change gegenüber v0.17
- **WCAG-Prinzip-Coverage** als sekundärer Indikator: prozentualer Abdeckungsgrad nach WCAG-Prinzip (Wahrnehmbar, Bedienbar, Verständlich, Robust)
- **Depth-Saturation Scoring**: Zwei-Phasen-Sättigungskurve, Diversity-Faktor, Soft Floor — verhindert Score-Inflation bei wenig geprüften Regeln

### Fixed
- Visually-hidden-Erkennung, echte Fokus-Erreichbarkeit, lokalisierbare Selektoren in WCAG-Regeln

---

## [0.17.0] - 2026-05-17

### Added
- **17 neue WCAG AAA Regeln** (Issues #84–#89): vollständige AAA-Abdeckung (70+ Regeln gesamt)

---

## [0.16.0] - 2026-05-16

### Added
- 20 Unit-Tests für AI-Visibility-Module (knowledge_graph, readability, citation, chunks)

### Changed
- **PDF-Modularisierung**: `pdf/mod.rs` von 1976 auf 460 LOC reduziert — 6 separate Module extrahiert

### Removed
- 14 tote `dead_code`-Suppressionen — unbenutzte Felder, Funktionen und Konstanten entfernt

---

## [0.15.0] - 2026-05-16

### Added
- **axe_id-Feld** in `Rule` und `NormalizedFinding` (JSON) für Pa11y/axe-core-Vergleich
- **Topical Authority Signals** im SEO-Modul
- **Browser Integration Tests** (E2E mit Chrome)
- Snapshot-Tests als Regressionsnetz vor weiteren Refactorings

### Fixed
- Kontrast-Erkennung, aria-hidden-focus-Regel, pa11y-Vergleich

---

## [0.14.0] - 2026-05-14

### Added
- **Throttled Performance PDF-Sektion**: Desktop- und Mobile-Vitals getrennt dargestellt
- Heuristische Modul-Markierungen in JSON und PDF

### Fixed
- JSON-Vollständigkeit, History-Datei-Bug, leere Seite nach Inhaltsverzeichnis im PDF

---

## [0.13.0] - 2026-05-14

### Fixed
- Security-Header-Bereinigung, Occurrence-Deduplizierung, Content-Weight-Berechnung, Dark-Mode-Konsistenz

---

## [0.12.3] - 2026-05-14

### Fixed
- Sitemap-Prompt-Sichtbarkeit und Terminal-Detection für interaktiven Select-Dialog

---

## [0.12.2] - 2026-05-14

### Changed
- **CLI-Orchestrierung** aus `main.rs` in `src/cli/`-Unterverzeichnis ausgelagert (runners, commands, report_writers, output_paths, plan, sitemap_suggest)
- Schema-Vertrag und Modul-Gewichtungs-Semantik formalisiert

---

## [0.12.0] - 2026-05-14

### Added
- **Content-Visibility-Kapitel** im PDF-Report
- Manual-Review TagCloud für WCAG-Kriterien

### Fixed
- False Positives in ARIA-Rollen, Landmark-Regeln und aria-controls-Erkennung reduziert

---

## [0.11.9] - 2026-05-14

### Added
- **Unified Assessment Types** (`src/assessment/`): Issues #51, #52, #54 — einheitliches Typ-System, Evidence-Model, Content-Visibility-Builder
- Testabdeckung: ViolationEvidence, content_visibility, From-Konversionen

---

## [0.11.4] - 2026-05-13

### Fixed
- PDF-Severity-Counts nach Visibility-Filter (Zählung stimmte nicht mit angezeigten Findings überein)

---

## [0.11.3] - 2026-05-13

### Added
- `--format summary` für Ranking-Dashboard-Export (kompakter Score-Überblick aller URLs)

---

## [0.11.2] - 2026-05-13

### Fixed
- Unbekannte Taxonomy-Rule-IDs für WCAG 1.4.1, 1.4.13, 2.3.3

---

## [0.11.1] - 2026-05-13

### Changed
- Batch-Builder, CLI-Output, PDF und Tests überarbeitet und konsistent gemacht

---

## [0.11.0] - 2026-05-13

### Added
- **CDN/WAF-Erkennung** aus Response-Headern (Security-Modul)

### Fixed
- TLS-Resets von Cloudflare toleriert; `Sec-CH-UA`-Header entfernt (Bot-Detection-Reduzierung)

---

## [0.10.9–0.10.4] - 2026-05-11/12

### Added
- **Tech Stack im PDF-Report**: CMS/Framework-Erkennung in eigenem PDF-Abschnitt
- **URL-Matrix-Tabelle** in Batch- und Single-Page-Reports

### Fixed
- PDF-Layout (Seitenumbrüche, Orphan-Schutz via renderreport 0.2.13–0.2.15)
- `www`-Subdomains-Guard bei Domain-Checks
- hreflang-Behandlung für einsprachige Sites

---

## [0.10.1] - 2026-05-11

### Added
- **Desktop/Mobile Performance-Split**: getrennte Vitals-Sektionen für beide Viewports
- **Sample-Scan** (20 Seiten) als dritte Sitemap-Option
- **WCAG-Coverage TagCloud**: automatisch geprüfte Kriterien visuell dargestellt

---

## [0.10.0] - 2026-05-11

### Added
- **WCAG-Regelexpansion**: zahlreiche neue Regeln, `NotTestable`-Findings für manuelle Kriterien (#48), `FindingKind`-Konfidenz-Level (#36)
- **Pattern Detection**: MainNavigation, SkipLink, Accordion, Dialog, DisclosureMenu, TabList
- **axe-core Vergleichsskript** (`scripts/axe-compare`) für Cross-Tool-Vergleich (#50)
- **SERP-Pass**: Aggregationsschicht über SEO-Signale
- **Page Health**: W3C Nu HTML Validator + 9 weitere Signale
- Schema-Typ-Verteilung im Batch-Report

---

## [0.9.22–0.9.29] - 2026-05-08/10

### Added
- **Desktop/Mobile-Screenshot-Preview** auf der PDF-Titelseite
- **Dual-Viewport Dark-Mode-Scoring**: Desktop + Mobile mit 70/30-Gewichtung
- `ReportModule`-Trait für JSON/PDF-Parität

### Changed
- **i18n-Vollständigkeit**: alle PDF-Texte auf Deutsch, mehrsprachige generische Link-Phrasen (14 Sprachen)
- Executive-Abschnitt im PDF gestrafft, CLI-Sprache auf Englisch vereinheitlicht

### Fixed
- Dark-Mode-Kontraststrafe begrenzt (verhindert Score-Kollaps auf 0)
- Performance-Metriken: Lighthouse-style pre-navigation, LCP-Observer-Wartezeit, INP durch TBT ersetzt

---

## [0.9.12] - 2026-04-22

### Added
- **PDF-Report-Redesign**: neue Modulabschnitte, Section-Akzente, kompaktere Struktur

---

## [0.9.6–0.9.11] - 2026-04-17/22

### Added
- **SERP-Aggregationsschicht** über SEO-Signale
- **W3C Nu HTML Validator** im Page-Health-Modul

### Fixed
- ARIA-Rollen und Landmark-Regeln: False Positives reduziert
- CLS-Messung via PerformanceObserver (erfasst Post-Load-Shifts)

---

## [0.9.0–0.9.5] - 2026-04-09/16

### Added
- **AI-Visibility-Modul** (`src/ai_visibility/`): Chunks, Knowledge Graph, Readability, Citations
- Gewichtungsanzeige auf 100% normalisiert (war 125% bei 7 Modulen)

### Fixed
- Score-Gewichtung, Seitentyp-Konflikt, robots.txt-KI-Training-Block kein Fehler mehr

---

## [0.8.3–0.8.9] - 2026-04-06/09

### Added
- **Source Quality-Modul** (`src/source_quality/`): Header-Signale, Schema, HTTPS
- **Executive Narrative**, Confidence Matrix, AuditFlags, Batch-Normalisierung

### Fixed
- Batch-Report-Dateiname mit Domain und Datum
- UTF-8-Truncate-Panics bei Multibyte-Zeichen

---

## [0.8.0–0.8.2] - 2026-04-05

### Added
- **Studio-Vertrag** (`src/studio/`): Shared Types und JSON-Schema für GUI-Anbindung
- **PDF-Report-Redesign**: 6-Seiten-Struktur nach 9-Punkte-Review

### Changed
- renderreport auf crates.io 0.2.3 umgestellt (zuvor lokale Path-Dep)

---

## [0.7.0] - 2026-04-05

### Added
- **UX-Modul** (`src/ux/`): 5 Dimensionen, Sättigungskurven, PDF/JSON-Integration
- **Journey-Modul** (`src/journey/`): Seitentyp-Erkennung, Friction-Points, 5 Dimensionen
- **Score ≠ Risk**: unabhängige Risikobewertung neben dem Accessibility-Score
- 21 neue WCAG-Regeln

### Changed
- renderreport 0.2.2: neue PDF-Komponenten, Section-Akzente, Orphan-Schutz

---

## [0.6.1–0.6.3] - 2026-04-02/03

### Added
- `--format ai` Output-Format für AI-lesbare Ausgabe (0.6.3)

### Fixed
- Binary-Umbenennung in Release-Workflow, Clippy/fmt-Fehler (0.6.1/0.6.2)

---

## [0.6.0] - 2026-04-02

### Added
- **Crawler + URL-Discovery** (`--crawl`, `--crawl-depth`): BFS-Crawler entdeckt intern verlinkte Seiten einer Domain automatisch
- **Broken Links**: Erkennt interne/externe 4xx/5xx und Redirect-Ketten (bis 6 Hops); HEAD→GET-Fallback; Severity-Stufen; PDF-Sektion
- **Duplicate / Near-Duplicate Content**: SimHash-64 mit 2-Wort-Shingles; Boilerplate-Filter; Duplikat (≥ 95 %) vs. Near-Duplicate (80–94 %); Batch-PDF-Sektion
- **Render Blocking & Asset-Größen** (`src/performance/render_blocking.rs`): erkennt blocking `<script>`/CSS in `<head>`, First-/Third-Party-Aufteilung; PDF-Sektion
- **Performance Budgets** (`[budgets]` in `auditmysite.toml`): 10 konfigurierbare Limits (LCP, FCP, CLS, TBT, JS-KB, CSS-KB, Größe, Requests, Blocking-Scripts, Third-Party-KB); Severity Error/Warning; JSON, CLI und PDF
- **Wettbewerbsvergleich** (`--compare`): 2–10 Domains in einem Lauf vergleichen; Domain-Ranking, Modul-Vergleich, Top-Findings; PDF, JSON, CSV
- **Dark Mode Detection** (`src/dark_mode/`): erkennt `@media (prefers-color-scheme: dark)`, `color-scheme`-Property, Meta-Tags, CSS Custom Properties; CDP-Emulation für Kontrastvergleich; Score + PDF-Sektion
- **Fundstellen-Enrichment** (`src/accessibility/enrichment.rs`): ersetzt „AX-Node 103" durch echte DOM-Selektoren via CDP `DOM.describeNode` + `DOM.resolveNode`; zeigt `img.hero [src: …]`, `a.nav-link > svg` etc.
- **Batch PDF**: Render-Blocking-Aggregation und Budget-Violations-Aggregation über alle URLs
- **comfy-table** ersetzt prettytable-rs für UTF-8-Box-Drawing-Tabellen mit farbigen Severity-Zellen
- **dialoguer** ersetzt manuelles stdin-Parsing für interaktive Prompts (Domain-Input, Sitemap-Auswahl)
- `--crawl` erklärt nun im CLI-Help warum externes Traffic erzeugt wird

### Changed
- CLI-Tabellen nutzen jetzt `comfy_table` mit `UTF8_FULL`-Preset und dynamischer Spaltenbreite
- Domain-Eingabe und Sitemap-Auswahl im interaktiven Modus über `dialoguer::Input` / `Select`

## [0.3.2] - 2026-01-30

### Fixed
- Fixed GitHub Actions release workflow for renamed binary (audit → auditmysite)
- Updated artifact names and packaging commands

### Added
- Installation policy documentation in DEPLOYMENT.md

## [0.3.1] - 2026-01-30

### Fixed
- Fixed all Clippy warnings for cleaner codebase
- Renamed `SchemaType::from_str` to `parse` to avoid std trait conflict
- Added `ProgressCallback` type alias for better readability
- Simplified `map_or` patterns to `is_some_and`

### Added
- v0.3.x roadmap for stabilization and PDF improvements
- Release checklist in ROADMAP.md

## [0.3.0] - 2026-01-30

### Changed
- **License**: Changed from MIT to LGPL-3.0-or-later

### Added
- **Security Hardening**
  - SSRF Protection: Block private IPs (10.x, 172.16-31.x, 192.168.x), localhost, link-local
  - URL validation for all audit targets (single and batch mode)
  - Path traversal protection in URL file reading
  - Pinned Chromium version and trusted CDN URL

- **Performance Optimizations**
  - Parallel extraction of AXTree and computed styles via `tokio::join!`
  - Style caching: `check_with_styles()` method eliminates redundant CDP calls
  - ~100-200ms faster audits for AA/AAA level checks

- **Testing**
  - 19 new integration tests (170 total)
  - `tests/url_validation_tests.rs` - SSRF protection tests
  - `tests/output_format_tests.rs` - Report generation tests
  - `tests/error_handling_tests.rs` - Error path tests

- **Documentation**
  - `docs/ARCHITECTURE.md` - System design and data flow
  - `docs/CONTRIBUTING.md` - Development setup and PR process
  - `docs/TROUBLESHOOTING.md` - Common issues and solutions

### Fixed
- JSON parsing now logs warnings instead of failing silently
- All `.expect()` calls replaced with proper error handling
- Browser pool reset timeout (5 seconds) prevents hanging
- WebSocket error on browser close eliminated

### Removed
- Outdated documentation files (MIGRATION.md, FEATURE_PARITY.md, etc.)

## [0.2.1] - 2026-01-30

### Changed
- Default output format changed to PDF
- Auto-generated output path: `reports/<domain>_<date>.pdf`

### Fixed
- WebSocket connection error on browser close
- Build warnings cleaned up

## [0.2.0] - 2026-01-30

### Changed
- Renamed binary from `audit` to `auditmysite` to avoid macOS conflict

### Added
- PDF report generation via renderreport/Typst
- Homebrew formula for easy installation

## [0.1.0] - 2026-01-29

### Added
- Initial release
- Chrome/Chromium auto-detection (macOS, Linux, Windows)
- Headless browser management via chromiumoxide
- CDP (Chrome DevTools Protocol) integration
- Accessibility Tree (AXTree) extraction
- 12 WCAG 2.1 rules (Level A, AA, AAA)
- Contrast checking with color calculation
- JSON, HTML, Table, Markdown output formats
- Sitemap XML parsing for batch processing
- Browser pool for concurrent audits
- Progress bars with ETA

---

**Repository:** https://github.com/casoon/auditmysite
