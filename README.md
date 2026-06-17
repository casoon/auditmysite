# auditmysite

> Accessibility audits for real rendered pages, built for CI and modern frontend stacks

[![CI](https://github.com/casoon/auditmysite/actions/workflows/ci.yml/badge.svg)](https://github.com/casoon/auditmysite/actions/workflows/ci.yml)
[![Release](https://github.com/casoon/auditmysite/actions/workflows/release.yml/badge.svg)](https://github.com/casoon/auditmysite/actions/workflows/release.yml)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-BSL--1.1-blue.svg)](LICENSE)

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

Requires Rust 1.75+. Builds and installs the binary from source.

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

**Optional Cargo features:**

| Feature | What it adds | Build command |
|---------|-------------|---------------|
| `pdf` | PDF report generation via the `renderreport`/Typst engine | `cargo build --release --features pdf` |
| `pdf_test` | PDF rendering integration tests | `cargo test --features pdf_test` |

## Requirements

- Rust 1.75+ for local builds
- Chrome, Chromium, or a managed browser install (`auditmysite browser install`)
- macOS, Linux, or Windows for released binaries

`auditmysite` requires a browser to be present at run time. It does **not** download or install one automatically — it reports an error and exits if none is found. To install a managed Chrome for Testing into `~/.auditmysite/browsers/`:

```bash
auditmysite browser detect   # show what's found
auditmysite browser install  # download Chrome for Testing (opt-in)
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
# default: full audit + terminal summary + PDF/JSON in current directory
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

# per-page reports: scan a list/sitemap but write one PDF per URL instead of an aggregated batch report
auditmysite --url-file urls.txt --per-page-reports --output reports/per-page/
auditmysite --sitemap https://example.com/sitemap.xml --per-page-reports --output reports/per-page/
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
- `auditmysite <url>`: run a full single-page audit and write PDF/JSON into the current directory
- `auditmysite --sitemap <url>`: audit sitemap URLs
- `auditmysite --url-file <file>`: audit URLs from file
- `auditmysite <url> --crawl`: discover same-domain pages from a seed URL and audit them as a batch
- `auditmysite browser detect`: show available browsers
- `auditmysite browser install`: download and install Chrome for Testing into `~/.auditmysite/browsers/` (opt-in, never automatic)
- `auditmysite doctor`: run local diagnostics

Useful flags:
- `--prefer-sitemap`: if a sitemap is detected for a base URL, switch directly into batch mode
- `--no-sitemap-suggest`: suppress sitemap probing/suggestion and keep the run on the single URL
- `--crawl-depth <n>`: limit same-domain crawl discovery depth when using `--crawl`
- `--per-page-reports`: scan a URL list or sitemap but write one individual report per URL instead of an aggregated batch report; `-o` is treated as a target directory
- `--lang <de|en>`: set the language for PDF reports (default: `de`)
- `--stack`: enable tech stack detection and stack-specific security probes (included automatically with `--full`)
- `--interactive <off|basic|full>`: control the Accessibility Journey Layer for interactive checks — tab walk, skip-link, modal focus trap, SPA navigation, form-error announcement, link-text inventory (default: `full`; use `off` for fastest runs)

For the full current interface, use:

```bash
auditmysite --help
auditmysite browser --help
```

## Output Contract

JSON output is treated as an automation contract.

- Contract documentation: [docs/OUTPUT_CONTRACT.md](docs/OUTPUT_CONTRACT.md)
- Single report schema: [docs/json-report.schema.json](docs/json-report.schema.json)
- Batch report schema: [docs/json-batch-report.schema.json](docs/json-batch-report.schema.json)

Key top-level fields in a single-page report:
- `findings` — static WCAG violations and SEO findings
- `interactive_findings` — journey-phase results (link texts, landmarks, heading outline, focus order, modal traps …); present when `--interactive basic|full` was used
- `accessibility_journey` — structured trace of each journey (steps, snapshots, durations); present when `--interactive basic|full` was used

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
- Frame and iframe rules — accessible names on all frames (`frame-title`), manual-review notices for cross-origin frames (`frame-tested`), and a full WCAG content scan inside same-origin iframes: image-alt (1.1.1), button-name (4.1.2), link-name (2.4.4), form labels (1.3.1), duplicate IDs (4.1.1), html lang (3.1.1)
- SVG rules — SVG image accessible names
- Server-side image maps — detection and flagging
- Meta viewport — large maximum-scale restrictions

100+ rules with stable `rule_id`, `tags` (e.g. `wcag2a`, `wcag412`, `cat.aria`), and an `impact` field (`critical` / `serious` / `moderate` / `minor`).

Methodology numbers are frozen in `docs/PARITY_CONTRACT.jsonc` and guarded by `tests/parity_contract.rs`: WCAG 2.1 AA has 50 A/AA criteria, 27 are covered by automated AuditMySite checks, and 16 are listed as manual-review criteria.

Some criteria (keyboard trap behavior, timed content, captions) cannot be reliably verified by automated means. These are flagged as `not_testable` in the JSON output and listed in the report's audit scope section as requiring manual review.

AAA is not fully implemented yet.

### Additional modules

Modules are classified as **measured** (based on real browser data) or **heuristic** (structural-signal estimates, marked with `~` in reports).

Measured:
- Performance: Core Web Vitals (FCP, LCP, TBT, CLS) and technical complexity (DOM size, render blocking, resource loading)
- SEO: meta tags, headings, structured data, content profile, tracking/external services signals
- Security: HTTPS, header checks, and CDN/WAF protection detection
- Mobile: viewport, touch-target, readability checks, UX heuristics (cookie-banner, modal/overlay, CTA detection)

Heuristic (indicator scores — tendency, not measurements):
- UX: 5-dimension analysis (CTA clarity, visual hierarchy, content clarity, trust signals, cognitive load) with saturation curve scoring
- Journey: user-flow analysis (entry clarity, orientation, navigation, interaction, conversion) with page-intent-aware weighting
- AI Visibility: structural readiness for LLM indexing and citation (readability, citability, structured data, AI policy, chunk quality)
- Source Quality: code hygiene signals (inline styles, deprecated elements, semantic structure, asset hygiene)
- Dark Mode: detects dark mode support via `prefers-color-scheme` media queries and CSS custom properties
- Tech Stack: detects CMS and frameworks (WordPress, Drupal, Joomla, Next.js, Astro, React, Vue, etc.) via in-page signals and runs stack-specific security probes (admin panel exposure, user enumeration, version disclosure)

### Accessibility Journey Layer

Interactive checks run a real browser session after the static AXTree phase. They run in `full` mode by default and can be reduced via `--interactive <off|basic|full>` or `mode` in `auditmysite.toml`.

| Mode | What runs |
|------|-----------|
| `off` | No interactive phase — fastest, no browser interaction after initial load |
| `basic` | Tab-walk (focus order, reverse jumps), skip-link verification, disclosure/accordion, modal focus trap, tab-list, menu journey |
| `full` (default) | Everything in `basic`, plus: SPA-navigation detection, form-error announcement, link-text inventory (generic/duplicate texts, heading outline, landmark structure) |

Results appear in `interactive_findings` and `accessibility_journey` in the JSON output. They do not affect the accessibility score or `legal_flags`; critical interactive findings can raise the risk level.

**`auditmysite.toml` configuration:**

```toml
[interactive]
mode = "full"             # off | basic | full
journey_budget_ms = 8000  # wall-clock budget per URL in milliseconds (default: 6000)
```

### Risk assessment

Risk level is computed independently from the score. A page scoring 81 can still carry "Critical" risk if it has Level A violations relevant under BFSG/EAA. Risk levels: Low, Medium, High, Critical — based on critical/high violations, legal flags, and blocking issues (4.1.2/2.1.1).

### Configuration file

`auditmysite.toml` is an optional project-level config file placed in the working directory. It supports `[audit]`, `[rules]`, `[interactive]`, `[thresholds]`, and `[budget]` sections.

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
- Top (decision layer): hero block with score + risk level, top 3 problems, next 3 steps, overall assessment (UX/Accessibility, Technik/Sicherheit, SEO)
- Bottom (implementation layer): task block ("Was jetzt tun?" with role, effort, impact, priority), module overview, key findings, technical implementation details, detailed metrics

**Sitemap/batch report** is aggregated and domain-wide: averages, ranking, recurring issues, URL matrix, near-duplicate content, broken links, crawl diagnostics.

Batch reports are not a stack of single-page reports.

## Compared to typical setups

- Better fit for JavaScript-heavy sites than static HTML-only checks
- Easier to distribute than a multi-package browser toolchain
- More automation-friendly than ad hoc console output because the JSON contract is explicit and tested
- Broader reporting surface than a pure accessibility-only checker when you also want performance, SEO, security, and mobile signals
- Violations carry stable `rule_id`, `tags`, and `impact` — easier to integrate with existing tooling or dashboards

### Screen Reader Audit vs. axe-core / Pa11y

Standard accessibility checkers verify individual rules in isolation. `auditmysite` additionally simulates the sequential experience of a screen reader user navigating the page — detecting problems that only emerge in context.

| Capability | axe-core | Pa11y | auditmysite |
|---|---|---|---|
| Rule-based WCAG checks | ✓ | ✓ | ✓ |
| Reading sequence simulation | — | — | ✓ |
| Out-of-context link text analysis (duplicate "Read more" × 8) | — | — | ✓ |
| Accessible name quality score (not just present/absent) | — | — | ✓ |
| Landmark navigation strategy (can a SR user reach main content?) | — | — | ✓ |
| BFSG / EN 301 549 legal mapping per finding | — | — | ✓ |

When the screen reader module runs, a JSON sidecar is written automatically next to the primary report:

```
example-com-YYYY-MM-DD-single-report.pdf
example-com-YYYY-MM-DD-single-report-screen-reader-audit.json  ← automatic sidecar
```

The sidecar shows exactly what a screen reader would announce, node by node, including which announcements are ambiguous or missing — suitable as a developer reference and as evidence for BFSG compliance audits. No extra flag is required; the file is created whenever screen reader data is available in the audit result.

## Typical Workflows

Examples grouped by audience and goal.

### Customer-facing report (PDF)

Single-URL audit with full module coverage and a custom logo on the cover.

```bash
# default: writes a PDF + JSON sidecar to the current directory
auditmysite https://example.com --full

# explicit branding and output path
auditmysite https://example.com --full --logo ./assets/customer-logo.svg --output reports/customer.pdf

# pick a report depth: executive (management), standard (default), technical (developers)
auditmysite https://example.com --full --report-level executive --output reports/exec.pdf

# PDF language (default: de)
auditmysite https://example.com --full --lang en --output reports/report-en.pdf
```

### CI / automation (JSON)

Quiet, machine-readable output for pipelines.

```bash
# exit code follows score thresholds; JSON report for downstream tooling
auditmysite https://example.com -f json -o report.json --quiet

# batch CI run on a sitemap
auditmysite --sitemap https://example.com/sitemap.xml -f json -o sitemap-report.json --quiet
```

### AI fix list

Compact, agent-friendly output that focuses on actionable fixes.

```bash
auditmysite https://example.com -f ai -o fixes.json
```

### Dashboard / ranking feed

Compact summary JSON with score, grade, medal, issue counts, and top 10 findings — matches the `lastAudit` schema used by dashboard tools.

```bash
auditmysite https://example.com -f summary -o summary.json
```

### Sitemap / batch

Domain-wide audits with cross-page aggregation.

```bash
# explicit sitemap
auditmysite --sitemap https://example.com/sitemap.xml --full

# crawl from a base URL
auditmysite https://example.com --crawl --crawl-depth 2 --max-pages 50 --full

# URL list from file
auditmysite --url-file urls.txt --full

# one PDF per URL instead of an aggregated batch report
auditmysite --sitemap https://example.com/sitemap.xml --per-page-reports --output reports/per-page/
```

### Local development

```bash
# audit a local dev server with a system Chrome
auditmysite https://localhost:3000 --browser-path /Applications/Google\ Chrome.app/Contents/MacOS/Google\ Chrome

# quick CLI summary without writing files
auditmysite https://example.com --format table
```

### Base URL with sitemap suggestion

```bash
# interactive: ask first if a sitemap is found
auditmysite https://example.com

# non-interactive: switch directly to sitemap mode
auditmysite https://example.com --prefer-sitemap

# stay on the single URL even when a sitemap exists
auditmysite https://example.com --no-sitemap-suggest
```

## Architecture

```text
CLI -> Browser Manager -> Chrome/CDP -> Accessibility Tree -> WCAG Engine -> Output
```

Key layers:
- `browser/`: browser detection, resolution, explicit install (`browser install` command only — no auto-download), lifecycle, pooling
- `audit/`: pipeline, normalization, scoring, batch processing
- `wcag/`: rule engine and violations
- `output/`: CLI, JSON, PDF, AI, summary format
- `seo/`, `security/`, `performance/`, `mobile/`, `ux/`, `journey/`: optional analysis modules
- `tech_stack/`, `source_quality/`, `ai_visibility/`, `dark_mode/`: heuristic indicator modules

More detail:
- Current implementation: [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)
- Browser dependency details: [docs/chrome-dependency.md](docs/chrome-dependency.md)
- Troubleshooting: [docs/TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md)

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

### Debugging report content (hidden `--debug-typ`)

PDF reports are rendered through the `renderreport`/Typst engine. To review report
**completeness and wording** without opening the binary PDF, use the hidden
`--debug-typ` flag together with `--format pdf`. It writes the intermediate Typst
source as a `.typ` sidecar next to the PDF, for both single and batch reports:

```bash
# Single report → reports/example-audit.pdf + reports/example-audit.typ
./target/release/auditmysite https://example.com --full --format pdf \
  --output reports/example-audit.pdf --debug-typ

# Batch report → reports/example-batch.pdf + reports/example-batch.typ
./target/release/auditmysite --sitemap https://example.com/sitemap.xml --full \
  --format pdf --output reports/example-batch.pdf --debug-typ
```

The `.typ` file is plain text and diff-friendly — useful for checking which audits
land in the report and reviewing the exact wording of every section. The flag is
intentionally hidden from `--help` (developer/debug use only).

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

`auditmysite` is source available under the Business Source License 1.1
(`BUSL-1.1`). You may view, copy, modify, redistribute, and make non-production
use of the source code. Limited internal production use is permitted under the
Additional Use Grant in [LICENSE](LICENSE).

Commercial SaaS rehosting, paid third-party audit services based on the
Licensed Work, and use for AI training, embeddings, dataset generation,
automated code ingestion, or other machine learning usage require a separate
commercial license from Casoon.

Each version automatically converts to the Apache License, Version 2.0 on its
Change Date. For version 0.26.0, the Change Date is 2030-05-21. Older versions
released under AGPL-3.0-or-later remain under their original license terms.

## Credits

- Browser automation via [chromiumoxide](https://github.com/mattsse/chromiumoxide)
- PDF reports via [renderreport](https://github.com/casoon/renderreport)
- WCAG reference material from [W3C](https://www.w3.org/WAI/WCAG21/)
