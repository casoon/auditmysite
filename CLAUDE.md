# AuditMySite - Project Instructions

## Project Overview
Resource-efficient WCAG 2.1 Accessibility Checker written in Rust. Audits web pages using Chrome DevTools Protocol (CDP) and the browser's native Accessibility Tree. Supports single URL, sitemap batch, and URL file batch modes.

## Architecture
- **Language:** Rust (async with tokio)
- **Browser:** Chrome/Chromium via `chromiumoxide` (CDP)
- **CLI:** `clap` with derive macros
- **PDF:** `renderreport` (Typst-based, optional `pdf` feature) — lokales Repo unter `../renderreport`
- **Config:** Optional `auditmysite.toml` files

## Module Structure
```
src/
├── main.rs              # CLI entry point + test module
├── lib.rs               # Library exports
├── error.rs             # Centralized error types (AuditError)
├── util.rs              # Utility functions
│
├── cli/                 # CLI layer (args, config, orchestration)
│   ├── args.rs          # Clap args (Args, WcagLevel, OutputFormat)
│   ├── config.rs        # auditmysite.toml config file support
│   ├── commands.rs      # Subcommand handlers (browser, doctor, plan)
│   ├── runners.rs       # Mode runners (single, batch, compare)
│   ├── report_writers.rs# Output dispatch (single/batch/comparison)
│   ├── output_paths.rs  # File path generation for reports
│   ├── plan.rs          # Pre-audit plan/banner printing
│   └── sitemap_suggest.rs # Sitemap discovery + interactive prompt
│
├── audit/               # Pipeline, batch, scoring, normalization
├── browser/             # Chrome detection, launch, pooling
├── accessibility/       # AXTree extraction via CDP
├── wcag/                # WCAG rule engine + 50+ rule files
├── best_practices/      # Console errors and vulnerable JS library detection
│   ├── console_errors.rs # CDP-based console error/warning collection
│   └── vulnerable_libs.rs # Known-CVE JS library detection (jQuery, Bootstrap, …)
│
├── performance/         # Core Web Vitals, render-blocking, content weight
│   ├── animations.rs    # Non-composited animation detection
│   ├── coverage.rs      # Unused JS/CSS detection via CDP Coverage API
│   ├── critical_chain.rs # Critical request chain analysis
│   ├── minification.rs  # Unminified JS/CSS asset detection
│   └── third_party.rs   # Third-party resource attribution per origin
├── seo/                 # Meta, headings, schema, social, technical SEO
│   └── image_efficiency.rs # Image format and resolution analysis
├── security/            # Security header analysis
├── mobile/              # Mobile friendliness analysis
├── dark_mode/           # Dark mode support detection and contrast
├── ux/                  # UX analysis (5 dimensions, saturation curves)
├── journey/             # User journey analysis, page intent detection
├── screen_reader/       # Screen-reader reading-order primitives
├── ai_visibility/       # AI/LLM discoverability analysis
├── content_visibility/  # Cross-module signal aggregation (SEO+AI+Quality)
├── source_quality/      # Source quality signals (headers, schema, HTTPS)
├── tech_stack/          # CMS/framework detection from in-page signals
├── patterns/            # UI pattern detection (nav, accordion, modal, …)
├── assessment/          # Shared assessment types and evidence model
├── studio/              # Studio contract types (GUI data contract)
│
├── output/              # Formatters: table, json, pdf
├── taxonomy/            # Severity, Dimension, IssueClass enums
└── i18n/                # Project Fluent (.ftl), default language: German
```

## Key CLI Modes
- Single: `auditmysite <URL>`
- Sitemap: `auditmysite --sitemap <SITEMAP_URL>` (batch from XML sitemap)
- URL file: `auditmysite --url-file <FILE>` (batch from text file)
- Full audit: `--full` (enables performance, seo, security, mobile)
- Browser: `auditmysite browser {detect|install|remove|path}`, `auditmysite doctor`
- Output formats: `--format {json|table|pdf}`

## Report Intent
- **Single URL audit** is intentionally detailed and page-specific.
- Use it when one concrete page should be reviewed deeply, with findings, explanations, module detail, and implementation guidance for that page.
- **Sitemap / batch audit** is intentionally aggregated and domain-wide.
- Use it when multiple URLs should be compared, averaged, and prioritized across the site.
- Batch reports must focus on cross-page information such as:
  - average scores
  - strongest / weakest URLs
  - recurring issues
  - URL ranking and compact URL matrices
  - distribution patterns across the scanned set
- Batch reports must **not** devolve into a stack of single-page reports. Per-URL detail should stay compressed unless a dedicated technical appendix is explicitly intended.

## Reports Directory
- **All manually generated test reports MUST be saved to `reports/`**
- Use `--output reports/<filename>` when running audits
- The `reports/` directory is gitignored (except `reports/README.md`)
- Naming convention: `<domain>-audit.<format>` (e.g., `casoon-audit.html`)
- Batch reports: `<domain>-batch-audit.<format>`

## Build & Test
```bash
cargo build --release          # Build optimized binary
cargo check                    # Fast compile check
cargo check --all-features     # PFLICHT vor jedem Push — was CI prüft
cargo test                     # Run all tests
cargo test --lib               # Unit tests only
```

**Vor jedem Push `cargo check --all-features` ausführen.** CI prüft immer mit allen Features und Clippy.
Ein pre-push Hook ist unter `.git/hooks/pre-push` eingerichtet und läuft automatisch.

Häufige Falle: neue Felder in `NormalizedReport` brechen Struct-Initialisierer in
`src/audit/history.rs` und `src/audit/summary.rs`. Immer beide prüfen.

## Testing Against Live Sites
```bash
# 1. Single page audit (all modules) — tiefe Analyse einer konkreten Seite
./target/release/auditmysite https://example.com --full --format pdf --output reports/example-audit.pdf

# 2. Sample batch audit — 20 Seiten als repräsentativer Durchschnitt
# Ideal um template-weite Probleme (fehlendes ARIA, Struktur, SEO-Muster)
# von seitenspezifischen Fehlern zu trennen. Liefert stabile Durchschnittswerte.
./target/release/auditmysite --sitemap https://example.com/sitemap.xml --full --format pdf --output reports/example-sample-audit.pdf --max-pages 20

# 3. Full sitemap batch audit — alle Seiten, domain-weit
./target/release/auditmysite --sitemap https://example.com/sitemap.xml --full --format pdf --output reports/example-batch-audit.pdf

# Quick CLI check
./target/release/auditmysite https://example.com --format table
```

## renderreport-Workflow

`renderreport` ist eine eigene Typst-basierte PDF-Library unter `/Users/jseidel/GitHub/renderreport`.

**Dependency-Regel:** Immer als **crates.io-Dependency mit exakter Version** — niemals als `path`- oder `git`-Dep:
```toml
renderreport = { version = "0.2.19", optional = true }
```

**Neue Komponente oder Bugfix in renderreport:**
1. Änderungen in `/Users/jseidel/GitHub/renderreport` vornehmen
2. Version in `renderreport/Cargo.toml` bumpen (z. B. `0.2.19` → `0.2.20`)
3. In renderreport committen und pushen: `git push origin main`
4. Tag setzen und pushen: `git tag v0.2.20 && git push origin v0.2.20`
5. Auf crates.io veröffentlichen: `cargo publish --allow-dirty`
6. In `auditmysite/Cargo.toml` die Version aktualisieren
7. `cargo check --features pdf` zur Verifikation
8. `Cargo.lock` committen

**Komponenten** (Rust-Struct + Typst-Template + Registry-Eintrag):
- Rust-Struct: `src/components/standard.rs` oder `advanced.rs`
- Typst-Template: `templates/components/<name>.typ`
- Registry: `src/components/registry.rs` → `self.register(ComponentId::new("name"), include_str!(...))`
- Bei Verwendung in FlowGroup: Eintrag in `templates/components/flow_group.typ`
- Export über `pub use standard::*` in `src/components/mod.rs` — kein separater Re-export nötig

**Spacing-Tokens:** spacing-1=4pt, spacing-2=6pt, spacing-3=10pt, spacing-4=14pt, spacing-5=20pt
**Font-Tokens:** xs=8.5pt, sm=8.8pt, base=10.5pt, lg=13pt, xl=18pt, 2xl=24pt

## Report Format Rules
- **Always use PDF format** (`--format pdf`) when generating test reports
- Never use HTML export for reports
- PDF reports use the `renderreport` Typst engine with full module detail sections

## Report Wording Style
Gilt für alle Interpretations-/Erklärungstexte (`interpret_score`, `build_seo_interpretation`,
Overall-Erklärung, Modul-Dashboard). Quelle: `src/output/builder/helpers.rs` (`interpret_score`),
`src/output/builder/seo.rs`. Review-Surface: `reports/interpretations.json` (regenerieren mit
`cargo test --lib export_all_interpretations -- --ignored --nocapture`).

- **Lokalisierung ist Pflicht.** Bei `locale = "en"` echte englische Sätze ausgeben — nie deutschen
  Satzbau mit eingesetztem englischem Nomen ("die accessibility ist…"). Jeder Text existiert in `de` **und** `en`.
- **Module unterscheiden sich.** Keine geteilte Satzschablone über alle Module. Pro Modul eigene
  Betonung: Accessibility = rechtlich/Zugänglichkeit, Performance/UX/Journey = Nutzerwirkung,
  Security = vorsichtig/juristisch ("im geprüften Umfang", keine Sicherheitsgarantie),
  Mobile = "Nutzung auf Mobilgeräten" (nicht "mobile Nutzbarkeit").
- **Beschreibe Wirkung, nicht nur Zustand.** Gute Audit-Sätze decken Zustand + Auswirkung (+ ggf.
  Risiko/Priorität) ab — nicht nur "ist solide".
- **Aufwand nicht als Zeit ausdrücken.** Keine Zeitfenster oder Fristen für Aufwandsschätzungen
  nennen ("einige Tage", "1–4 Wochen", "mittelfristig", "innerhalb von Tagen"). Stattdessen nach
  Aufwand/Priorität formulieren: `geringer Aufwand`, `mittlerer Aufwand`, `strukturelle Änderung`.
- **Notenbänder (Label-Präfix):** `Sehr gut` (≥90) · `Gut` (≥75) · `Verbesserungswürdig` (≥60) ·
  `Ausbaufähig` (≥40) · `Kritisch` (<40). EN: `Excellent` · `Good` · `Needs improvement` ·
  `Inadequate` · `Critical`. **"Befriedigend" ist verboten** (klingt nach Schulnote).
- **Verbotene Füllphrasen:** "auf einem hohen Niveau", "einzelne Verbesserungen sind möglich",
  "weist (relevante/einzelne) Schwächen auf", "solide" als Allzweckwort. Bei SEO Endkunden-Jargon
  vermeiden ("Ranking-Signale" → "Sichtbarkeit in Suchmaschinen").
- **Bevorzugtes Vokabular:** beeinträchtigt, erschwert, stabil, konsistent, technisch sauber,
  zuverlässig, robust, eingeschränkt, fehlend, unvollständig, nachvollziehbar.

## Architecture Documentation
Whenever a new module is added, renamed, or removed, update the Module Structure section above **and** `ARCHITECTURE.md` in the same commit. Also update the `Current State` version and module list when bumping the version.

## Code Conventions
- Use `thiserror` for error types, `anyhow` for propagation
- WCAG rules go in `src/wcag/rules/` as individual files, register in `mod.rs`
- Output formatters go in `src/output/`, support both single and batch reports
- Keep async operations in audit pipeline and browser modules
- Use `tracing` for structured logging (INFO, WARN, ERROR)

## Current State (v0.45.0)
- **Catalog-Refactoring** (Phase A+B): `trait AuditModule` + `AuditCatalog` Registry mit Topo-Sort; alle 12 Module migriert; table-driven WCAG-Page-Rule-Catalog; `audit/interpretation.rs` (pre-computed DE/EN-Texte); `audit/summary.rs` (Aggregations-Logik); Builder ist reiner Mapper (#330–#338)
- Branch: `main`
- Cache: `--reuse-cache` validiert `CacheMeta.audit_signature` (Tool-Version + WCAG-Level + aktive Module + Consent) gegen die aktuelle Konfiguration; bei Mismatch Cache-Miss + Warnung, Legacy-Cache ohne Signatur wird nie wiederverwendet (#260)
- Crawler: parserbasierte Linkextraktion via html5ever inkl. `<base href>` (#263)
- Batch-JSON: optionaler `sample`-Block (source, total_discovered, audited, sample_limit, selection, is_sample) + PDF-Prüfumfang-Zeile (#261)
- Performance: `VitalMetric.measurement` (`lab_headless`/`estimated_lab`); INP/TTI/Speed Index als Lab-Schätzung markiert, Lab-Disclaimer im Report (#262)
- Kontrast: Bild-/Gradient-Hintergründe werden zu Manual-Review-Warnungen demoted statt als bestätigte Verstöße (#264, Pixel-Sampling offen)
- axe-Parität: `scripts/axe-compare.js` + `docs/AXE_PARITY.md` Kalibrierungs-Workflow (#265)
- **Accessibility Journey Layer** (`--interactive off|basic|full`): Tab-Walk, Skip-Link, Disclosure, Modal, TabList, Menu, Form-Error-Announcement, SPA-Navigation, Linktext-/Heading-/Landmark-Inventur (#297–#301). Ergebnisse in `interactive_findings` + `accessibility_journey` im JSON.
- **Semantic AI Evaluation** (`--semantic-eval`): Fastembed (lokal, multilingual) + optionaler Mistral-LLM-Check. Ergebnisse in `advisory_findings` (nie score-relevant). Feature-Flag: `semantic-eval` (#301).
- **Snapshot Export** (`--export-snapshot <path>`): AXTree + Journey-Traces als YAML für CI-Regression (#301).
- Linktext-Stopwords in i18n FTL (`locales/de|en/report.ftl`, Schlüssel `linktext-generic-stopwords`) — erweiterbar ohne Code-Änderung (#299).
- 95+ WCAG rules implemented (Level A, AA, full AAA coverage)
- 2 output formats (json, pdf); table for quick terminal checks
- Batch processing with configurable concurrency
- Pattern Detection: MainNavigation, SkipLink, Accordion, Dialog, DisclosureMenu, TabList, Form
- Modules: Performance, SEO, Security, Mobile, Dark Mode, UX, Journey, AI Visibility, Content Visibility, Source Quality, Tech Stack, Best Practices, Accessibility Journey Layer, Semantic Eval
- Consent: `--dismiss-consent` Flag; CMP-Cookie-Injection + Banner-Click; `consent_banner` audit_flag im JSON
- `audit_flags` kinds: `conflicting_signal` (3.1.1 vs. SEO lang), `viewport_gap` (Desktop/Mobile ≥20 Punkte), `consent_banner`, `consent_wall_artifact`, `bypass_blocks_untested` (Skip-Link vorhanden aber funktional kaputt — statischer Check hat PASS, Journey FAIL)
- JSON: **Unified Report Envelope v2.0** — einheitliches Schema für single + batch (`schema_version`, `report_type`, `summary`, `pages[]`, `pages[i].detail`). Breaking Change ggü. v0.17.
- Scoring: Depth-Saturation (Zwei-Phasen), Diversity-Faktor, Soft Floor + logarithmische Kompression für extreme Penalties (≥85 Punkte), WCAG-Prinzip-Coverage; `score_breakdown` (nur bei `score_calculation_method = "viewport_weighted"`, sonst absent)
- Findings: `category`-Feld auf `NormalizedFinding` (`"wcag"` / `"seo"`); `severity_counts` zählt **Findings** (eine Zeile pro Regel/Severity, **nur WCAG-Kategorie** — bleibt risiko-/rechts-relevant). Im JSON-Report decken `occurrence_counts`, `violation_count` und `violated_rule_count` **alle Kategorien (WCAG + SEO)** ab — konsistent mit `findings[]` und `detail.fix_guidance` (#254/#255). `top_recurring_rules` bleibt WCAG-only. Achtung: `NormalizedReport.occurrence_counts` ist weiterhin WCAG-only (speist `SiteState`/Risk), der JSON-PageEntry berechnet die All-Category-Variante separat. `risk.severity` = schwerste Violation über alle Findings (kein eigenes `severity_max`-Feld)
- Risk Level: Score-basierter Fallback (score ≤ 20 → mindestens Medium); `legal_flags > 0` oder `blocking_issues ≥ 1` heben das Level mindestens auf Medium. `legal_flags` zählt **distinct WCAG-Level-A-Regeln** mit High/Critical-Severity (nicht Occurrences).
- History: `schema_version: "1.0"`, `report_type: "history"` in History-JSON-Dateien
- PDF: Throttled-Performance-Tabelle, Indikator-Kennzeichnung konsistent, leere Seite nach ToC behoben; Accessibility-Journey-Section in Single- und Batch-Reports
- Performance-Score: Lighthouse-v10/v11-Gewichtung (FCP 10 %, LCP 25 %, TBT 30 %, CLS 25 %), log-normale Score-Kurven mit p10/p50-Kalibrierung; CLS > 0.5 hart auf 0 gecappt
- `tool_version` als Top-Level-Feld im JSON-Report (parallel zu `schema_version`/`report_type`)
- Sitemap-Summary enthält `violated_rule_count` (dedupliziert über alle Pages) und `top_recurring_rules` (max. 10 häufigste WCAG-Verstöße)
- Pass-Kriterium (`passed_url_count`): accessibility_score ≥ 80, keine Critical-Findings und keine WCAG-Level-A High/Critical-Findings (also `legal_flags == 0`)
- `detail.fix_guidance` ist immer im JSON präsent (leeres Array bei 0 Findings) — auch in Batch-/Sitemap-Reports; dort trägt jede Page ein kompaktes `detail` (nur `fix_guidance`, ohne Modul-Blob), siehe #256
