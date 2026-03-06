# AuditMySite - Project Instructions

## Project Overview
Resource-efficient WCAG 2.1 Accessibility Checker written in Rust. Audits web pages using Chrome DevTools Protocol (CDP) and the browser's native Accessibility Tree. Supports single URL, sitemap batch, and URL file batch modes.

## Architecture
- **Language:** Rust (async with tokio)
- **Browser:** Chrome/Chromium via `chromiumoxide` (CDP)
- **CLI:** `clap` with derive macros
- **PDF:** `renderreport` (Typst-based, optional `pdf` feature, local path `../renderreport`)
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
cargo test                     # Run all tests
cargo test --lib               # Unit tests only
```

## Testing Against Live Sites
```bash
# Single page audit (all modules) — always use PDF format for reports
./target/release/auditmysite https://example.com --full --format pdf --output reports/example-audit.pdf

# Sitemap batch audit
./target/release/auditmysite --sitemap https://example.com/sitemap.xml --full --format pdf --output reports/example-batch-audit.pdf --max-pages 5

# Quick CLI check
./target/release/auditmysite https://example.com --format table
```

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

## Current State (v0.4.0)
- Branch: `feat/march-improvements`
- 22 WCAG rules implemented (Level A, AA, some AAA)
- 3 output formats (json, table, pdf)
- Batch processing with configurable concurrency
- Performance, SEO, Security, Mobile analysis modules
