# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2026-01-30

### Changed
- **License**: Changed from MIT to LGPL-3.0-or-later

### Added
- **Security Hardening**
  - SSRF Protection: Block private IPs (10.x, 172.16-31.x, 192.168.x), localhost, link-local
  - URL validation for all audit targets (single and batch mode)
  - Path traversal protection in URL file reading
  - Pinned Chromium version and trusted CDN URL

- **Performance Optimizations**
  - Parallel extraction of AXTree and computed styles via `tokio::join!`
  - Style caching: `check_with_styles()` method eliminates redundant CDP calls
  - ~100-200ms faster audits for AA/AAA level checks

- **Testing**
  - 19 new integration tests (170 total)
  - `tests/url_validation_tests.rs` - SSRF protection tests
  - `tests/output_format_tests.rs` - Report generation tests
  - `tests/error_handling_tests.rs` - Error path tests

- **Documentation**
  - `docs/ARCHITECTURE.md` - System design and data flow
  - `docs/CONTRIBUTING.md` - Development setup and PR process
  - `docs/TROUBLESHOOTING.md` - Common issues and solutions

### Fixed
- JSON parsing now logs warnings instead of failing silently
- All `.expect()` calls replaced with proper error handling
- Browser pool reset timeout (5 seconds) prevents hanging
- WebSocket error on browser close eliminated

### Removed
- Outdated documentation files (MIGRATION.md, FEATURE_PARITY.md, etc.)

## [0.2.1] - 2026-01-30

### Changed
- Default output format changed to PDF
- Auto-generated output path: `reports/<domain>_<date>.pdf`

### Fixed
- WebSocket connection error on browser close
- Build warnings cleaned up

## [0.2.0] - 2026-01-30

### Changed
- Renamed binary from `audit` to `auditmysite` to avoid macOS conflict

### Added
- PDF report generation via renderreport/Typst
- Homebrew formula for easy installation

## [0.1.0] - 2026-01-29

### Added
- Initial release
- Chrome/Chromium auto-detection (macOS, Linux, Windows)
- Headless browser management via chromiumoxide
- CDP (Chrome DevTools Protocol) integration
- Accessibility Tree (AXTree) extraction
- 12 WCAG 2.1 rules (Level A, AA, AAA)
- Contrast checking with color calculation
- JSON, HTML, Table, Markdown output formats
- Sitemap XML parsing for batch processing
- Browser pool for concurrent audits
- Progress bars with ETA

---

**Repository:** https://github.com/casoon/auditmysite
