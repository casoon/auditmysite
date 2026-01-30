# Migration Guide: TypeScript → Rust

This guide helps you migrate from the TypeScript/Node.js version of AuditMySite to the new Rust implementation.

## Quick Comparison

| Feature | TypeScript (Legacy) | Rust (New) |
|---------|---------------------|------------|
| **Performance** | 2-3s per page | <1s per page |
| **Memory Usage** | 500-600MB | 200-300MB |
| **WCAG Rules** | 9 implemented | 12 implemented |
| **Output Formats** | 3 (JSON, HTML, CSV) | 5 (JSON, HTML, PDF, Markdown, Table) |
| **PDF Reports** | ❌ | ✅ (Typst-based) |
| **Progress Bars** | ❌ | ✅ (with ETA) |
| **Browser Pooling** | ❌ | ✅ |
| **Contrast Detection** | Basic | Enhanced (150+ elements) |
| **Binary Size** | ~50MB (node_modules) | ~10MB (single binary) |

## Installation

### Old (TypeScript/npm)
```bash
npm install -g @casoon/auditmysite
```

### New (Rust/cargo)
```bash
# Via Cargo
cargo install auditmysit

# Via Homebrew (coming soon)
brew install auditmysit

# Or download binary from releases
wget https://github.com/casoon/auditmysite/releases/latest/download/auditmysit-macos
chmod +x auditmysit-macos
```

## CLI Changes

### Basic Audit

**Old:**
```bash
auditmysite https://example.com/sitemap.xml
```

**New:**
```bash
# Single URL
auditmysit https://example.com

# Sitemap
auditmysit --sitemap https://example.com/sitemap.xml
```

### Output Formats

**Old:**
```bash
auditmysite https://example.com/sitemap.xml --format html
auditmysite https://example.com/sitemap.xml --format json
```

**New:**
```bash
auditmysit https://example.com -f html
auditmysit https://example.com -f json
auditmysit https://example.com -f pdf   # NEW!
auditmysit https://example.com -f markdown
```

### Batch Processing

**Old:**
```bash
auditmysite https://example.com/sitemap.xml --max-pages 10
```

**New:**
```bash
auditmysit --sitemap https://example.com/sitemap.xml --max-pages 10
```

### WCAG Level

**Old:**
```bash
# Level was auto-detected
```

**New:**
```bash
auditmysit https://example.com -l AA   # Level AA (default)
auditmysit https://example.com -l AAA  # Level AAA
auditmysit https://example.com -l A    # Level A
```

## Breaking Changes

### 1. CLI Syntax

| Change | Old | New |
|--------|-----|-----|
| Positional argument | `<sitemapUrl>` | `<url>` or `--sitemap <url>` |
| Format flag | `--format` | `-f` or `--format` |
| Output directory | `--output-dir` | `-o` or `--output` |
| Verbose mode | `--verbose` | `-v` or `--verbose` |
| Quiet mode | Not available | `-q` or `--quiet` |

### 2. Output JSON Structure

The JSON output structure has been streamlined:

**Old:**
```json
{
  "url": "https://example.com",
  "accessibility": {
    "score": 85,
    "violations": [...]
  },
  "performance": {...},
  "seo": {...}
}
```

**New:**
```json
{
  "metadata": {
    "version": "0.1.0",
    "timestamp": "2026-01-30T12:00:00Z"
  },
  "report": {
    "url": "https://example.com",
    "score": 85.0,
    "grade": "B",
    "certificate": "GOLD",
    "wcag_results": {
      "violations": [...]
    }
  }
}
```

### 3. Configuration Format

**Old:** JSON-based config file
```json
{
  "maxPages": 5,
  "format": "html"
}
```

**New:** No config file yet (coming soon), use CLI flags
```bash
auditmysit --max-pages 5 -f html
```

### 4. Exit Codes

| Scenario | Old | New |
|----------|-----|-----|
| Success | 0 | 0 |
| Violations found | 0 | 0 (configurable with `--strict`) |
| Error | 1 | 1 |

## New Features Not in TypeScript Version

### 1. PDF Reports
```bash
auditmysit https://example.com -f pdf -o report.pdf
```

### 2. Progress Bars
Batch audits now show visual progress with ETA:
```
⠁ [00:00:15] [####################>---] 3/5 (00:00:10) example.com
```

### 3. Enhanced Contrast Checking
- Analyzes 150+ elements per page (was ~20)
- JavaScript-based extraction (more reliable)
- Detects large text correctly

### 4. Browser Pooling
Concurrent processing for faster batch audits.

### 5. AAA Level Support
```bash
auditmysit https://example.com -l AAA
```

## Missing Features (Planned)

The following TypeScript features are not yet in the Rust version:

- ❌ REST API server
- ❌ SEO analysis (partial)
- ❌ Performance analysis (Core Web Vitals)
- ❌ Mobile-specific tests
- ❌ Security headers analysis
- ❌ Expert mode / Interactive CLI

These will be added in future releases.

## Troubleshooting

### Chrome Not Found

**Old:** Used bundled Puppeteer Chromium
**New:** Auto-downloads Chromium to `~/.cache/chromiumoxide/`

If you have Chrome installed:
```bash
auditmysit --chrome-path /path/to/chrome https://example.com
```

### Different Results

The Rust version may report slightly different violations because:
- Enhanced contrast detection finds more issues
- More WCAG rules implemented
- Different scoring algorithm

### Performance Slower Than Expected

Make sure you're using a release build:
```bash
cargo build --release
./target/release/auditmysit https://example.com
```

Debug builds are ~3x slower.

## Code Migration

### TypeScript SDK → Rust Library

**Old:**
```typescript
import { AuditSDK } from '@casoon/auditmysite';

const sdk = new AuditSDK();
const result = await sdk.auditUrl('https://example.com');
```

**New:** Library usage coming soon. For now, use CLI:
```bash
auditmysit https://example.com -f json -o result.json
# Then parse JSON in your code
```

## Support

- **Legacy TypeScript Version:** See branch `archive/typescript-legacy`
- **Rust Version Issues:** https://github.com/casoon/auditmysite/issues
- **Migration Questions:** Open a discussion on GitHub

## Timeline

- **TypeScript version:** Archived as of 2026-01-30
- **Rust version:** 0.1.0 released 2026-01-30
- **Next release (0.2.0):** Planned for Q1 2026 (API server, SEO)
