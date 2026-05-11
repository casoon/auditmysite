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
├── main.rs              # CLI entry point
├── lib.rs               # Library exports
├── error.rs             # Centralized error types (AuditError)
├── util.rs              # Utility functions
├── accessibility/       # AXTree extraction via CDP
├── audit/               # Pipeline, batch processing, scoring, reports
├── browser/             # Chrome detection, management, pooling
├── cli/                 # Args (clap), config file support
├── output/              # Formatters: cli, json, pdf
├── performance/         # Core Web Vitals, content weight
├── seo/                 # Meta, headings, schema, social, technical
├── security/            # Security header analysis
├── mobile/              # Mobile friendliness analysis
└── wcag/                # WCAG rule engine + 22 individual rule files
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

**Dependency-Regel:** Immer als **git-Dependency mit Tag** — niemals als `path`-Dep (bricht CI):
```toml
renderreport = { git = "https://github.com/casoon/renderreport.git", tag = "v0.2.9", optional = true }
```

**Neue Komponente oder Bugfix in renderreport:**
1. Änderungen in `/Users/jseidel/GitHub/renderreport` vornehmen
2. Version in `renderreport/Cargo.toml` bumpen (z. B. `0.2.9` → `0.2.10`)
3. In renderreport committen und pushen: `git push origin main`
4. Tag setzen und pushen: `git tag v0.2.10 && git push origin v0.2.10`
5. In `auditmysite/Cargo.toml` den Tag aktualisieren
6. `cargo check --features pdf` zur Verifikation
7. `Cargo.lock` committen

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

## Code Conventions
- Use `thiserror` for error types, `anyhow` for propagation
- WCAG rules go in `src/wcag/rules/` as individual files, register in `mod.rs`
- Output formatters go in `src/output/`, support both single and batch reports
- Keep async operations in audit pipeline and browser modules
- Use `tracing` for structured logging (INFO, WARN, ERROR)

## Current State (v0.10.0)
- Branch: `main`
- 33+ WCAG rules implemented (Level A, AA, some AAA)
- 3 output formats (json, table, pdf)
- Batch processing with configurable concurrency
- Pattern Detection: MainNavigation, SkipLink, Accordion, Dialog, DisclosureMenu, TabList
- Performance, SEO, Security, Mobile, Source Quality, AI Visibility modules
