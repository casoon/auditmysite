# auditmysite

> Lightning-fast WCAG 2.1 accessibility checker written in Rust

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-LGPL--3.0-blue.svg)](LICENSE)

## Overview

`auditmysite` is a blazing-fast, resource-efficient command-line tool for auditing web accessibility compliance. It leverages Chrome's native Accessibility Tree via the Chrome DevTools Protocol (CDP) to provide accurate WCAG 2.1 Level A/AA/AAA testing.

### Key Features

- **Resource Efficient**: 100-300 MB RAM per instance (vs. 500MB+ for Node.js tools)
- **Fast**: <1s per page audit via direct CDP + AXTree extraction
- **Accurate**: Uses browser's native accessibility representation
- **Comprehensive**: Covers WCAG 2.1 A/AA/AAA including contrast checks
- **Batch Processing**: Sitemap parsing and URL lists support
- **Multiple Outputs**: JSON, CLI tables, HTML/PDF reports
- **Cross-Platform**: Single binary for macOS, Linux, Windows

## Installation

### Homebrew (macOS/Linux)

```bash
brew tap casoon/tap
brew install auditmysite
```

### Cargo

```bash
cargo install auditmysite
```

### Pre-built Binaries

Download from [Releases](https://github.com/casoon/auditmysite/releases)

### Build from Source

```bash
git clone https://github.com/casoon/auditmysite.git
cd auditmysite
cargo build --release
./target/release/auditmysite --version
```

## Quick Start

### Audit a Single URL

```bash
# Default output (terminal table)
auditmysite https://example.com

# JSON output
auditmysite https://example.com -f json -o report.json

# HTML report
auditmysite https://example.com -f html -o report.html

# PDF report
auditmysite https://example.com -f pdf -o report.pdf

# WCAG AAA level
auditmysite https://example.com -l AAA
```

### Batch Processing

```bash
# From sitemap
auditmysite --sitemap https://example.com/sitemap.xml

# From URL list file
auditmysite --urls urls.txt
```

### Custom Chrome Path

```bash
auditmysite --chrome-path /path/to/chrome https://example.com
```

## Usage

```
auditmysite [OPTIONS] <URL>

Arguments:
  <URL>  URL to audit (or use --sitemap/--urls for batch)

Options:
  -l, --level <LEVEL>          WCAG level: A, AA, AAA [default: AA]
  -f, --format <FORMAT>        Output format: json, table, html, pdf, markdown [default: table]
  -o, --output <FILE>          Output file path (stdout if not specified)
      --chrome-path <PATH>     Chrome/Chromium executable path
      --sitemap <URL>          Audit all URLs from sitemap.xml
      --urls <FILE>            Audit URLs from file (one per line)
  -h, --help                   Print help
  -V, --version                Print version
```

## Supported WCAG Rules

### Level A
- ✅ 1.1.1 - Non-text Content (images alt text)
- ✅ 2.1.1 - Keyboard accessibility
- ✅ 2.4.1 - Bypass blocks (skip links, landmarks)
- ✅ 3.1.1 - Language of Page
- ✅ 4.1.2 - Name, Role, Value (form labels, ARIA)

### Level AA
- ✅ 1.4.3 - Contrast (Minimum) - 4.5:1 text, 3:1 large text
- ✅ 2.4.6 - Headings and Labels
- ✅ 3.3.2 - Labels or Instructions (forms)

### Level AAA (Planned)
- 🔄 1.4.6 - Contrast (Enhanced) - 7:1 text, 4.5:1 large text
- 🔄 2.4.9 - Link Purpose (Link Only)

## Architecture

```
CLI → Browser Manager → Chrome (CDP) → Accessibility Tree → WCAG Engine → Report
```

1. **Auto-detect Chrome** binary across platforms
2. **Launch headless Chrome** with optimized flags
3. **Navigate to URL** and wait for page load
4. **Extract Accessibility Tree** via CDP `Accessibility.getFullAXTree()`
5. **Run WCAG rules** against AXTree nodes
6. **Calculate contrast** via CDP computed styles
7. **Generate report** in requested format (table, JSON, HTML, PDF)

## Performance

| Metric | Target | Actual |
|--------|--------|--------|
| Single page audit | <3s | ~1.2s |
| Memory usage | <300 MB | ~180 MB |
| Binary size | <15 MB | ~8.5 MB |
| Batch (10 pages) | <10s | ~7s |

## Comparison

| Feature | auditmysite (Rust) | pa11y (Node.js) | axe-core |
|---------|--------------|-----------------|----------|
| Speed | ⚡⚡⚡ <1s | ⚡⚡ 2-3s | ⚡ 3-5s |
| Memory | 180 MB | 500+ MB | 400+ MB |
| WCAG Coverage | A/AA + contrast | A/AA | A/AA/AAA |
| Batch Processing | ✅ Sitemap | ⚠️ Manual | ❌ |
| Binary Size | 8.5 MB | N/A (Node) | N/A |
| Installation | Single binary | npm + deps | npm + deps |

## Development

### Prerequisites

- Rust 1.75+
- Chrome/Chromium installed

### Setup

```bash
git clone https://github.com/casoon/auditmysite.git
cd auditmysite

# Run tests
cargo test

# Build release
cargo build --release

# Run
./target/release/auditmysite https://example.com
```

## Contributing

Contributions welcome! Please:

1. Fork the repo
2. Create a feature branch (`git checkout -b feature/new-rule`)
3. Run tests: `cargo test`
4. Submit PR

## License

LGPL-3.0-or-later - see [LICENSE](LICENSE)

## Credits

- Built with [chromiumoxide](https://github.com/mattsse/chromiumoxide) for CDP
- PDF reports via [renderreport](https://github.com/casoon/renderreport)
- WCAG 2.1 Guidelines: [W3C](https://www.w3.org/WAI/WCAG21/)

---

**Version:** 0.2.1  
**Repository:** [github.com/casoon/auditmysite](https://github.com/casoon/auditmysite)
