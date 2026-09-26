# AuditMySite - Project Instructions

## Project Overview
Resource-efficient WCAG 2.2 AA Accessibility Checker written in Rust. Audits web pages using Chrome DevTools Protocol (CDP) and the browser's native Accessibility Tree. Supports single URL, sitemap batch, and URL file batch modes.

## Architecture
- **Language:** Rust (async with tokio)
- **Browser:** Chrome/Chromium via `chromiumoxide` (CDP)
- **CLI:** `clap` with derive macros
- **PDF:** `renderreport` (Typst-based, optional `pdf` feature) — lokales Repo unter `../renderreport`
- **Config:** Optional `auditmysite.toml` files

## Key CLI Modes
- Single: `auditmysite <URL>`
- Sitemap: `auditmysite --sitemap <SITEMAP_URL>` (batch from XML sitemap)
- URL file: `auditmysite --url-file <FILE>` (batch from text file)
- Full audit: `--full` (enables performance, seo, security, mobile)
- Browser: `auditmysite browser {detect|install|remove|path}`, `auditmysite doctor`
- Report lint: `auditmysite report-lint <JSON_FILE> [--fail-on low|medium|high|critical]` (#507, deterministic checks, no network/Chrome; default fail-on: high; exit code 3 if breached)
- accname-Differential: `auditmysite accname-diff <URL> [--output FILE] [--max-samples N]` (eigenes `accname` gegen Chromes nativen AX-Tree; misst, urteilt nicht — Exit-Code immer 0; siehe `docs/accname-differential.md`)
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
`src/audit/normalized.rs` und `src/audit/summary.rs`. Immer beide prüfen.

## Release & `cargo publish` (PFLICHT — #588)
Eine auf crates.io veröffentlichte Version lässt sich **nie** ersetzen, nur zurückziehen. Und
docs.rs baut jede Version genau einmal: Was dort kaputt ankommt, bleibt dauerhaft kaputt. Deshalb
in dieser Reihenfolge:

0. **Vor dem Tag: Referenzlauf.** `cargo test --test reference_sites_test -- --ignored` auditiert
   die manuell bewerteten Live-Seiten aus `tests/fixtures/reference_sites.json` und meldet jede,
   die ihr Band verlässt (Plan 47). Blockiert nicht automatisch: Live-Seiten ändern sich, also
   erst die Seite neu bewerten, dann das Werkzeug verdächtigen — aber jede Abweichung vor dem Tag
   klären.
1. **Tag `vX.Y.Z` setzen und pushen.** Das startet `release.yml`: Versionsabgleich, volle CI,
   Binaries, GitHub-Release.
2. **Auf den Job `docs.rs Build (sandbox parity)` warten.** Er läuft **nur auf Tags** und baut die
   Doku im echten docs.rs-Sandbox-Image — deren Nightly, deren Speicher- und Zeitlimits, und auf
   dem, was `cargo package` veröffentlichen würde. Nichts sonst in der CI beantwortet das: Der
   Doku-Schritt in `check-all-features` baut auf dem Runner, mit Stable und ohne Speicherdeckel.
3. **Erst dann `cargo publish`.**

Der Job ist `continue-on-error`, solange die Action Beta ist — er blockiert also nicht, er
**informiert**. Ist er rot, nicht publizieren, sondern erst die Ursache klären.

Zur Größenordnung: 1.5.1 brauchte dort 4,75 GB von 6,4 GB verfügbarem RAM. Seit
`all-features = true` dokumentiert docs.rs auch `ai-transparency`, was den Abhängigkeitsgraphen von
454 auf 565 Crates hebt. Der Abstand zur Decke ist real, aber nicht groß.

Lokal vor dem Tag, falls `cargo-docs-rs` installiert ist: `cargo docs-rs` baut mit derselben
`[package.metadata.docs.rs]`-Konfiguration, allerdings ohne Sandbox und ohne Limits.

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

## Report Format Rules
- **Always use PDF format** (`--format pdf`) when generating test reports
- Never use HTML export for reports
- PDF reports use the `renderreport` Typst engine with full module detail sections

## Lokalisierungs-Architektur (#406 — PFLICHT bei neuen report-sichtbaren Texten)
**JSON ist kanonisch Englisch, nur das PDF ist mehrsprachig.** Regel:
- Die **Analyse-/Derive-Schicht backt kanonisches Englisch** in die gespeicherten Structs
  (`AuditReport`/`NormalizedReport` → JSON). Niemals die Lauf-Sprache in die Analyse-Ergebnisse
  backen — das JSON muss sprach-unabhängig englisch bleiben.
- Die **PDF-Präsentationsschicht leitet lokalisierte Texte zur Laufzeit ab** (`i18n.locale()`),
  über reine Funktionen/`build_*_presentation`/`render_*`.
- **Muster für message-baked Structs:** das Struct trägt ein kanonisches `kind`-Enum (+ Rohwerte
  in einem `#[serde(skip)]`/`skip_serializing_if`-Feld); eine reine `pub fn *_text(kind, .., en)`
  ist die EINZIGE Textquelle — die Analyse ruft sie mit `en=true`, der PDF-Builder mit der
  Lauf-Sprache. Beispiele: `source_quality`, `content_visibility`, `ai_visibility`, `journey`, `ux`,
  `seo/page_health` (`collect_issues`), `screen_reader`.
- **Erkennungs-Sprache ≠ Message-Sprache:** sprachabhängige DETEKTION (z. B. Stopword-Matching für
  generische Linktexte) folgt der **Seiten-Sprache**, nicht der Ausgabe-Sprache. Beide getrennt
  durchreichen (siehe `screen_reader::analyze_reading_sequence(detect_locale, message_en)`).
- Guard-Test pro lokalisiertem Modul: EN-Ausgabe enthält keine deutschen Umlaute/ß.

## Architecture Documentation
Whenever a new module is added, renamed, or removed, update `docs/ARCHITECTURE.md` in the same commit. Also update the `Current State` version when bumping the version.

## Code Conventions
- Use `thiserror` for error types, `anyhow` for propagation
- WCAG rules go in `src/wcag/rules/` as individual files, register in `mod.rs`
- Output formatters go in `src/output/`, support both single and batch reports
- Keep async operations in audit pipeline and browser modules
- Use `tracing` for structured logging (INFO, WARN, ERROR)

## Current State (v1.5.1)
See `CHANGELOG.md` for the detailed, chronological development history (findings, fixes,
verification) — extracted here (plan/11-claude-md-version-drift.md) to keep this file focused on
working rules. Update the version number above whenever `Cargo.toml`'s version bumps, and add new
entries to `CHANGELOG.md`, not here.
