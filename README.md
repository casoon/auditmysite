# auditmysite

> Accessibility audits for real rendered pages, built for CI and modern frontend stacks

[![CI](https://github.com/casoon/auditmysite/actions/workflows/ci.yml/badge.svg)](https://github.com/casoon/auditmysite/actions/workflows/ci.yml)
[![Release](https://github.com/casoon/auditmysite/actions/workflows/release.yml/badge.svg)](https://github.com/casoon/auditmysite/actions/workflows/release.yml)
[![Rust](https://img.shields.io/badge/rust-1.98.1-orange.svg)](rust-toolchain.toml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

## Overview

`auditmysite` is a Rust CLI that audits accessibility against fully rendered pages in Chrome. Instead of scanning raw HTML only, it uses Chrome DevTools Protocol (CDP) and the browser's native Accessibility Tree, so it can evaluate dynamic DOM, computed styles, and JavaScript-heavy applications more realistically.

It is designed for teams that want a fast local check, stable JSON for automation, and a single binary that can be dropped into CI.

## Why use it

- Real browser signals instead of static guesses
- Works for single pages, sitemaps, URL lists, and same-domain crawl discovery
- Outputs as terminal table, JSON, PDF, AI-optimized task list, or compact summary JSON for dashboards
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

By default, a single URL audit runs the full analysis set, prints a compact terminal summary, and writes report artifacts into the current working directory:

- `./example-com-YYYY-MM-DD-single-report.pdf`
- `./example-com-YYYY-MM-DD-single-report.json`
- `./example-com-YYYY-MM-DD-single-report-screen-reader-audit.json`

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

**Upgrading:** run the same command again. The installer detects where your current binary lives and replaces it in place — no PATH conflicts, no leftover old version.

> **Note:** If you previously installed via `cargo install auditmysite`, remove that binary first so the script installs to the right location:
> ```bash
> rm ~/.cargo/bin/auditmysite
> curl -fsSL https://raw.githubusercontent.com/casoon/auditmysite/main/install.sh | bash
> ```

Verify the installation:

```bash
auditmysite --version
auditmysite --help
auditmysite https://example.com
```

That default command writes report artifacts into the current directory, for example:

- `./example-com-YYYY-MM-DD-single-report.pdf`
- `./example-com-YYYY-MM-DD-single-report.json`
- `./example-com-YYYY-MM-DD-single-report-screen-reader-audit.json`

### cargo install (crates.io)

```bash
cargo install auditmysite
```

Builds and installs the binary from source. Tested with the pinned toolchain (see [Toolchain](docs/getting-started/build-from-source.md#toolchain)); older compilers are not supported.

Prebuilt binaries, building from source and the optional Cargo features: [Build from source](docs/getting-started/build-from-source.md).

## Requirements

- Chrome, Chromium, or a managed browser install (`auditmysite browser install`). auditmysite does **not** download a browser on its own; it reports an error and exits if none is found.
- macOS, Linux, or Windows for released binaries; Rust as pinned in `rust-toolchain.toml` for local builds.

Details, managed browsers and the macOS note on keyboard journeys: [Requirements](docs/getting-started/requirements.md).

## Quick Start

The fastest way to validate your setup:

```bash
auditmysite https://example.com
```

That creates the default report set in the current directory. For machine-readable output only:

```bash
auditmysite https://example.com -f json -o report.json
```

### Single page

```bash
# default: full audit + terminal summary + PDF/JSON in current directory
auditmysite https://example.com

# JSON
auditmysite https://example.com -f json -o report.json

# PDF with explicit path
auditmysite https://example.com -f pdf -o report.pdf

# stricter WCAG level
auditmysite https://example.com -l AAA
```

Common options:

- `--url-file <FILE>`: audit a list of URLs; `--sitemap <URL>`: audit a sitemap; `--crawl` discovers same-domain pages
- `--browser-path <PATH>`: use a specific Chrome or Chromium binary
- `-f json|table|pdf`, `-o <FILE>`: output format and target; `--lang de|en`: report language
- `--full`: add performance, SEO, security and mobile; `--level A|AA|AAA`: WCAG level

Sitemap, crawl and URL-file runs, per-page reports and browser selection: [Batch audits](docs/guides/batch-audits.md). All options: [CLI and output contract](docs/reference/cli-and-output.md).

## How the overall score is weighted

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/img/score-weights.dark.svg">
  <img src="docs/img/score-weights.svg" alt="Bar chart of the module weights in the overall score: Accessibility 40, Performance 20, SEO 15, Security 10, Mobile 10, HTML Conformance 5 percent." width="640">
</picture>

Only the weighted modules enter the overall score. The heuristic indicators (UX, Journey, Best Practices, Dark Mode, …) are shown next to it with weight 0. The charts in `docs/img/` are generated from the `.json` files next to them with [chartlet](https://github.com/casoon/chartlet) (`chartlet render <spec>.json --format svg --strict`, add `--theme dark` for the dark variant).

## What it checks

- **WCAG 2.2 Level A and AA** rules on the rendered page, with rule IDs that stay stable across releases.
- **Weighted modules:** Accessibility, Performance, SEO, Security, Mobile, HTML conformance.
- **Heuristic indicators** (UX, Journey, AI visibility, source quality, dark mode, tech stack, commerce) shown next to the score, with weight 0 and marked as unvalidated.
- **Accessibility journeys:** tab walk, skip link, modals, menus, SPA navigation, form errors; clicks that can change state are an opt-in (`--allow-state-changes`).
- **Display modes, structured data, risk assessment, rule configuration and exclusions.**

The full list with every module and option: [Feature scope](docs/reference/features.md).

## Architecture

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/img/audit-pipeline.dark.svg">
  <img src="docs/img/audit-pipeline.svg" alt="Flow chart: the CLI starts the browser manager, which drives Chrome over CDP; the accessibility tree feeds the WCAG engine and the analysis modules, and both write the report as table, JSON or PDF." width="560">
</picture>

Key layers:
- `browser/`: browser detection, resolution, explicit install (`browser install` command only — no auto-download), lifecycle, pooling
- `audit/`: pipeline, normalization, scoring, batch processing
- `wcag/`: rule engine and violations
- `output/`: CLI, JSON, PDF, AI, summary format
- `seo/`, `security/`, `performance/`, `mobile/`, `ux/`, `journey/`: optional analysis modules
- `tech_stack/`, `source_quality/`, `ai_visibility/`, `dark_mode/`: heuristic indicator modules

Parts of the checks live in [barrierlab](https://github.com/casoon/barrierlab) and come in as published crates (`a11y-rules`, `a11y-dom`, `a11y-report`, `accname`, `a11y-perception`, `web-checks`, `html-conform`); fixes there are released first and then taken over here by raising the version. Details and the boundary: [Shared libraries from barrierlab](docs/concepts/barrierlab.md). Current implementation: [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

## Documentation

| Topic | Where |
|---|---|
| First audit, CI use, dashboards, technician runs | [Typical workflows](docs/guides/workflows.md), [Quickstart](docs/getting-started/quickstart.md) |
| Batch audits, browser selection | [Batch audits](docs/guides/batch-audits.md), [Browser setup](docs/guides/browser-setup.md) |
| Options and output formats | [CLI and output contract](docs/reference/cli-and-output.md), [Output contract](docs/OUTPUT_CONTRACT.md) |
| Report types and levels | [Report modes](docs/guides/report-modes.md), [Reports](docs/guides/reports.mdx) |
| What is checked | [Feature scope](docs/reference/features.md) |
| How it differs from axe-core and Pa11y | [Compared to typical setups](docs/reference/comparison.md) |
| Heuristic signals and calibration | [Calibration](docs/calibration.md) |
| Building, tests, release checks, library API | [Development](docs/guides/development.md), [API reference](docs/reference/api.md) |
| Browser dependency | [Chrome dependency](docs/chrome-dependency.md) |

## Troubleshooting

- Browser not found: run `auditmysite browser detect` or install a managed browser with `auditmysite browser install`
- "Access to '…' was blocked": the site answered HTTP 401/403/407/429 or served a bot challenge (Cloudflare, Fastly, Akamai, DataDome, Imperva, HUMAN). That page is not the site's content, so the URL is not scored; in a batch it is listed under `errors`. Ask the site owner to allow the auditing machine
- Running in Docker or as root: use `--no-sandbox`
- Need raw output for scripts: prefer `-f json -o report.json`
- Unsure about the full CLI surface: run `auditmysite --help`

## Contributing

## License

`auditmysite` is licensed under the MIT License. Use it freely, including
commercially.

The shared rule engine, document model, accessible name computation, report
model and HTML conformance checker live in
[barrierlab](https://github.com/casoon/barrierlab) (see
[Shared libraries from barrierlab](docs/concepts/barrierlab.md)) and are
shared with [astro-post-audit](https://github.com/casoon/astro-post-audit) and
[liveaudit](https://github.com/casoon/liveaudit) — the same rule IDs across
build time, CI and the live page.

MIT applies from version 1.5.0 onward. Earlier releases remain under the
license that applied at the time: up to 0.25.x AGPL-3.0-or-later, 0.26.0
through 1.4.0 Business Source License 1.1. See [NOTICE](NOTICE).

## Credits

- Browser automation via [chromiumoxide](https://github.com/mattsse/chromiumoxide)
- PDF reports via [renderreport](https://github.com/casoon/renderreport)
- WCAG reference material from [W3C](https://www.w3.org/WAI/WCAG21/)
