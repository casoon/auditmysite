# auditmysite

> Accessibility audits for real rendered pages, built for CI and modern frontend stacks

[![CI](https://github.com/casoon/auditmysite/actions/workflows/ci.yml/badge.svg)](https://github.com/casoon/auditmysite/actions/workflows/ci.yml)
[![Release](https://github.com/casoon/auditmysite/actions/workflows/release.yml/badge.svg)](https://github.com/casoon/auditmysite/actions/workflows/release.yml)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-LGPL--3.0-blue.svg)](LICENSE)

## Overview

`auditmysite` is a Rust CLI that audits accessibility against fully rendered pages in Chrome. Instead of scanning raw HTML only, it uses Chrome DevTools Protocol (CDP) and the browser's native Accessibility Tree, so it can evaluate dynamic DOM, computed styles, and JavaScript-heavy applications more realistically.

It is designed for teams that want a fast local check, stable JSON for automation, and a single binary that can be dropped into CI.

## Why use it

- Real browser signals instead of static guesses
- Works for single pages, sitemaps, and URL lists
- Outputs as terminal table, JSON, or PDF
- JSON output is schema-backed and tested for release stability
- Ships as a Rust binary instead of a Node-based toolchain

## Why this approach

Most accessibility CLIs either depend on static parsing or require a heavier runtime stack around browser automation. `auditmysite` is opinionated in a different direction:

- Chrome-native accessibility data first
- CLI-first workflow for local use and CI
- Small operational surface: install a binary, point it at a URL, get a report
- Optional modules for performance, SEO, security, and mobile without changing tools

## Quick Example

```bash
auditmysite https://example.com
```

By default, a single URL audit runs the full analysis set and writes a PDF report into the current working directory, for example `./example-com-2026-03-31-standard.pdf`.

For CI or machine-readable output:

```bash
auditmysite https://example.com -f json -o report.json --quiet
```

## Install

### curl installer (macOS/Linux)

```bash
curl -fsSL https://raw.githubusercontent.com/casoon/auditmysite/main/install.sh | bash
```

The installer downloads the latest GitHub Release asset for your platform and verifies it against the published `.sha256` checksum before installing it.

Verify the installation:

```bash
auditmysite --version
auditmysite --help
auditmysite https://www.in-punkto.com
```

That default command writes a PDF report into the current directory, for example `./in-punkto-com-2026-03-31-standard.pdf`.

### Prebuilt binaries

Download from [Releases](https://github.com/casoon/auditmysite/releases).

- macOS/Linux: `.tar.gz`
- Windows: `.zip`

### Build from source

```bash
git clone https://github.com/casoon/auditmysite.git
cd auditmysite
cargo build --release
./target/release/auditmysite --version
```

## Requirements

- Rust 1.75+ for local builds
- Chrome/Chromium or a managed browser install
- macOS, Linux, or Windows for released binaries

If no compatible browser is installed:

```bash
auditmysite browser detect
auditmysite browser install
```

## Quick Start

The fastest way to validate your setup:

```bash
auditmysite https://example.com
```

That creates a PDF report in the current directory. For machine-readable output:

```bash
auditmysite https://example.com -f json -o report.json
```

### Single page

```bash
# default: full audit + PDF in current directory
auditmysite https://example.com

# JSON
auditmysite https://example.com -f json -o report.json

# PDF with explicit path
auditmysite https://example.com -f pdf -o report.pdf

# stricter WCAG level
auditmysite https://example.com -l AAA
```

### Batch audits

```bash
# sitemap
auditmysite --sitemap https://example.com/sitemap.xml

# URL file
auditmysite --url-file urls.txt
```

### Browser selection

```bash
auditmysite --browser-path /path/to/chrome https://example.com
```

## CLI

```text
auditmysite [OPTIONS] [URL] [COMMAND]
```

Primary commands:
- `auditmysite <url>`: run a full single-page audit and write a PDF into the current directory
- `auditmysite --sitemap <url>`: audit sitemap URLs
- `auditmysite --url-file <file>`: audit URLs from file
- `auditmysite browser detect`: show available browsers
- `auditmysite browser install`: install managed Chrome for Testing
- `auditmysite doctor`: run local diagnostics

For the full current interface, use:

```bash
auditmysite --help
auditmysite browser --help
```

## Output Contract

JSON output is treated as an automation contract.

- Contract documentation: [docs/OUTPUT_CONTRACT.md](/Users/jseidel/GitHub/auditmysite/docs/OUTPUT_CONTRACT.md)
- Single report schema: [docs/json-report.schema.json](/Users/jseidel/GitHub/auditmysite/docs/json-report.schema.json)
- Batch report schema: [docs/json-batch-report.schema.json](/Users/jseidel/GitHub/auditmysite/docs/json-batch-report.schema.json)

The repository validates these contracts in automated tests.

## Feature Scope

WCAG coverage currently includes key rules across level A and AA, including:
- Non-text content
- Keyboard access
- Bypass blocks
- Language of page
- Name, role, value / form labeling
- Contrast (minimum)
- Headings and labels
- Labels or instructions

AAA is not fully implemented yet.

Additional modules:
- Performance: Core Web Vitals and score interpretation
- SEO: meta tags, headings, structured data, content profile
- Security: HTTPS and header checks
- Mobile: viewport, touch-target, and readability checks

## Compared to typical setups

- Better fit for JavaScript-heavy sites than static HTML-only checks
- Easier to distribute than a multi-package browser toolchain
- More automation-friendly than ad hoc console output because the JSON contract is explicit and tested
- Broader reporting surface than a pure accessibility-only checker when you also want performance, SEO, security, and mobile signals

## Typical Workflows

### Local audit while developing

```bash
auditmysite https://localhost:3000 --browser-path /Applications/Google\\ Chrome.app/Contents/MacOS/Google\\ Chrome
```

### JSON report for CI

```bash
auditmysite https://example.com -f json -o report.json --quiet
```

### Batch audit from sitemap

```bash
auditmysite --sitemap https://example.com/sitemap.xml -f json -o sitemap-report.json
```

## Architecture

```text
CLI -> Browser Manager -> Chrome/CDP -> Accessibility Tree -> WCAG Engine -> Output
```

Key layers:
- `browser/`: browser detection, resolution, install, lifecycle, pooling
- `audit/`: pipeline, normalization, scoring, batch processing
- `wcag/`: rule engine and violations
- `output/`: CLI, JSON, PDF
- `seo/`, `security/`, `performance/`, `mobile/`: optional analysis modules

More detail:
- Current implementation: [docs/ARCHITECTURE.md](/Users/jseidel/GitHub/auditmysite/docs/ARCHITECTURE.md)
- Browser dependency details: [docs/chrome-dependency.md](/Users/jseidel/GitHub/auditmysite/docs/chrome-dependency.md)
- Troubleshooting: [docs/TROUBLESHOOTING.md](/Users/jseidel/GitHub/auditmysite/docs/TROUBLESHOOTING.md)

## Development

### Setup

```bash
git clone https://github.com/casoon/auditmysite.git
cd auditmysite
cargo test
cargo build --release
./target/release/auditmysite https://example.com
```

### Pre-commit secret scan

This repository uses `nosecrets` in the Git `pre-commit` hook.

Enable the repo hook path:

```bash
git config core.hooksPath .githooks
```

Install `nosecrets` as a real binary first:

```bash
npm install -g @casoon/nosecrets
# or
cargo install nosecrets-cli
```

The hook expects `nosecrets` to be available in `PATH`.

### Release checks

Run the local release gate with:

```bash
./scripts/release-check.sh
```

It validates:
- `cargo test`
- ignored browser integration tests
- builds with and without PDF
- current `--help` output
- JSON contract tests
- installer/release artifact consistency
- stale docs references

## Troubleshooting

- Browser not found: run `auditmysite browser detect` or install a managed browser with `auditmysite browser install`
- Running in Docker or as root: use `--no-sandbox`
- Need raw output for scripts: prefer `-f json -o report.json`
- Unsure about the full CLI surface: run `auditmysite --help`

## Contributing

## Library / Development

For library development or local work from the repository:

```bash
cargo build
cargo test
```

If you want the current local repository state as an installed binary while developing:

```bash
cargo install --path . --force
```

Contributions are welcome. At minimum before opening a PR:

```bash
cargo test
./scripts/release-check.sh
```

## License

LGPL-3.0-or-later. See [LICENSE](LICENSE).

## Credits

- Browser automation via [chromiumoxide](https://github.com/mattsse/chromiumoxide)
- PDF reports via [renderreport](https://github.com/casoon/renderreport)
- WCAG reference material from [W3C](https://www.w3.org/WAI/WCAG21/)
