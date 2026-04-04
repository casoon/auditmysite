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
- Works for single pages, sitemaps, URL lists, and same-domain crawl discovery
- Outputs as terminal table, JSON, PDF, or AI-optimized task list
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
- `./example-com-history.json`

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

That default command writes report artifacts into the current directory, for example:

- `./in-punkto-com-YYYY-MM-DD-single-report.pdf`
- `./in-punkto-com-YYYY-MM-DD-single-report.json`
- `./in-punkto-com-history.json`

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

That creates the default report set in the current directory. For machine-readable output only:

```bash
auditmysite https://example.com -f json -o report.json
```

### Single page

```bash
# default: full audit + terminal summary + PDF/JSON/history in current directory
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
# explicit sitemap
auditmysite --sitemap https://example.com/sitemap.xml

# crawl from a base URL and discover same-domain pages automatically
auditmysite https://example.com --crawl --crawl-depth 2

# base URL: probe robots.txt / common sitemap locations first
auditmysite https://example.com

# prefer sitemap automatically if one is found
auditmysite https://example.com --prefer-sitemap

# suppress sitemap suggestion and stay on the single page
auditmysite https://example.com --no-sitemap-suggest

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
- `auditmysite <url>`: run a full single-page audit and write PDF/JSON/history into the current directory
- `auditmysite --sitemap <url>`: audit sitemap URLs
- `auditmysite --url-file <file>`: audit URLs from file
- `auditmysite <url> --crawl`: discover same-domain pages from a seed URL and audit them as a batch
- `auditmysite browser detect`: show available browsers
- `auditmysite browser install`: install managed Chrome for Testing
- `auditmysite doctor`: run local diagnostics

Useful flags:
- `--prefer-sitemap`: if a sitemap is detected for a base URL, switch directly into batch mode
- `--no-sitemap-suggest`: suppress sitemap probing/suggestion and keep the run on the single URL
- `--crawl-depth <n>`: limit same-domain crawl discovery depth when using `--crawl`

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

### WCAG rules (Level A and AA)

Core rules:
- Non-text content (1.1.1)
- Keyboard access (2.1.1)
- Bypass blocks (2.4.1)
- Language of page (3.1.1)
- Name, role, value / form labeling (4.1.2)
- Contrast minimum (1.4.3) and non-text contrast (1.4.11)
- Headings and labels (2.4.6)
- Labels or instructions (3.3.2)
- Focus order (2.4.3) and focus visible (2.4.7)
- Label in name (2.5.3)

ARIA and semantics:
- ARIA role validation — invalid roles, required owned elements, required context
- ARIA attribute checks — allowed attributes per role, required attributes, prohibited attributes
- Accessible name checks — icon-only controls, empty aria-labelledby/describedby, name/description conflicts, naming by role type (command, input, meter, progressbar, toggle, dialog, treeitem)
- ARIA relationship checks — aria-controls, aria-owns, aria-activedescendant, duplicate IDs
- Landmark structure — main, navigation, banner, contentinfo (presence, uniqueness, top-level nesting, no-duplicate for banner/contentinfo/main, required parent for landmarks)
- Content in landmarks — region rule ensuring body content lives inside landmark regions
- Table rules — caption/name, header cells, presentational tables, cell placement
- Form rules — fieldset/legend for grouped controls, required field indication, error description, label-title-only detection
- List structure — listitem context, empty lists, definition list integrity
- Dialog rules — accessible name, aria-modal, alert region labeling
- Widget rules — tab/tabpanel pairing, selected state, combobox options, slider value, tree context, summary element naming
- Media rules — application and image-role elements without accessible names
- SVG rules — SVG image accessible names
- Server-side image maps — detection and flagging
- Meta viewport — large maximum-scale restrictions

77 rules with stable `rule_id`, `tags` (e.g. `wcag2a`, `wcag412`, `cat.aria`), and an `impact` field (`critical` / `serious` / `moderate` / `minor`).

AAA is not fully implemented yet.

### Additional modules

- Performance: Core Web Vitals and score interpretation
- SEO: meta tags, headings, structured data, content profile, tracking/external services signals
- Security: HTTPS and header checks
- Mobile: viewport, touch-target, readability checks, UX heuristics (cookie-banner, modal/overlay, CTA detection)

### Rule configuration

Rules can be selectively disabled or filtered via `auditmysite.toml`:

```toml
[rules]
disabled = ["heading-order", "landmark-one-main"]
# enabled_only = ["image-alt", "label"]  # run only these rules
```

### AI / LLM output format

Export findings as a task-oriented JSON list for direct LLM processing:

```bash
auditmysite https://example.com -f ai -o findings.json
```

Each entry is a task object with `task_id`, `rule_id`, `impact`, `wcag`, `tags`, `title`, `issue`, `fix`, `selector`, `node_id`, and `help_url` — sorted by impact severity. Suitable for direct use as context in AI-assisted code remediation.

### Baseline and CI diff

Save a baseline snapshot and compare future runs against it:

```bash
# Save baseline
auditmysite https://example.com -f json -o baseline.json

# Future CI runs can diff against the baseline programmatically via the Rust API
```

The `Baseline` type in the `audit` module supports `from_violations`, `diff`, `load`, and `save`.

## Report Modes

Single-page reports and sitemap/batch reports are intentionally different.

**Single-page report** is structured in two layers:
- Top (decision layer): hero block with score, top 3 problems, next 3 steps, overall assessment (UX/Accessibility, Technik/Sicherheit, SEO), trend
- Bottom (implementation layer): task block ("Was jetzt tun?" with role, effort, impact, priority), module overview, key findings, technical implementation details, detailed metrics

**Sitemap/batch report** is aggregated and domain-wide: averages, ranking, recurring issues, URL matrix, near-duplicate content, broken links, crawl diagnostics.

Batch reports are not a stack of single-page reports.

## Compared to typical setups

- Better fit for JavaScript-heavy sites than static HTML-only checks
- Easier to distribute than a multi-package browser toolchain
- More automation-friendly than ad hoc console output because the JSON contract is explicit and tested
- Broader reporting surface than a pure accessibility-only checker when you also want performance, SEO, security, and mobile signals
- Violations carry stable `rule_id`, `tags`, and `impact` — easier to integrate with existing tooling or dashboards

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

### Batch audit from crawl discovery

```bash
auditmysite https://example.com --crawl --crawl-depth 2 --max-pages 50 -f json -o crawl-report.json
```

### Base URL with sitemap suggestion

```bash
# ask first if a sitemap is found
auditmysite https://example.com

# switch directly to sitemap mode
auditmysite https://example.com --prefer-sitemap
```

## Architecture

```text
CLI -> Browser Manager -> Chrome/CDP -> Accessibility Tree -> WCAG Engine -> Output
```

Key layers:
- `browser/`: browser detection, resolution, install, lifecycle, pooling
- `audit/`: pipeline, normalization, scoring, batch processing
- `wcag/`: rule engine and violations
- `output/`: CLI, JSON, PDF, AI format
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

### Pre-commit checks

This repository uses Git hooks with a fast local `pre-commit` gate and a full `pre-push` gate.

`pre-commit` runs:

- `nosecrets` on staged changes
- `cargo fmt -- --check`
- `cargo clippy --lib --bins --all-features -- -D warnings`

`pre-push` runs:

- `scripts/check-version-match.sh` for pushed `v*` tags
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test`

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

Skip the Rust checks only when you intentionally need to bypass them:

```bash
SKIP_RUST_CHECKS=1 git commit -m "..."
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
