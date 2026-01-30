# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Planning Phase
- Comprehensive 35-week project plan created
- Architecture documentation completed
- WCAG rules catalog defined
- Development workflows established
- Skills and templates created for efficient development

### Infrastructure
- Git repository initialized
- Private GitHub repository created: https://github.com/casoon/auditmysit_rust
- `.claude/` configuration directory with:
  - Project setup documentation
  - Architecture documentation
  - WCAG rules catalog
  - Chrome installation strategies
  - Custom skills (audit, test-wcag, chrome-detect)
  - Code templates for WCAG rules
  - Development workflows

## [0.1.0] - TBD (MVP Target: Week 4)

### Planned Features
- Chrome/Chromium auto-detection (macOS, Linux, Windows)
- Headless browser management via chromiumoxide
- CDP (Chrome DevTools Protocol) integration
- Accessibility Tree (AXTree) extraction
- WCAG 2.1 Level AA compliance testing:
  - 1.1.1 - Non-text Content (image alt text)
  - 4.1.2 - Name, Role, Value (form labels, ARIA)
  - 2.4.6 - Headings and Labels
  - 1.4.3 - Contrast (Minimum) with color calculation
- JSON output format
- CLI table output format
- Sitemap XML parsing
- Batch URL processing
- Single binary distribution

### Technical Highlights
- Rust 1.75+ required
- Async/await with Tokio runtime
- Resource-efficient (100-300 MB RAM)
- Fast execution (<1s per page)
- Cross-platform support

---

## Future Releases

### [0.2.0] - Performance & SEO Analysis
- Core Web Vitals collection (LCP, FCP, CLS, INP, TTFB)
- Performance scoring system (0-100)
- Certificate levels (Platinum, Gold, Silver, Bronze)
- Meta tags validation
- Heading structure analysis
- Social meta tags (OpenGraph, Twitter Cards)
- Technical SEO (HTTPS, sitemap, robots.txt, schema.org)
- HTML report generation with interactive dashboard

### [0.3.0] - Advanced Features
- Mobile friendliness analysis (8-category scoring)
- Security headers analysis
- Advanced WCAG rules (keyboard, bypass blocks, link purpose)
- Glassmorphism false positive filtering
- Error deduplication
- PDF report generation

### [0.4.0] - Scaling & Enterprise
- Browser pool management
- Queue and pipeline system
- Network throttling (Slow 4G, Fast 3G)
- Request rate limiting
- Performance budgets (default, ecommerce, blog, corporate)
- Configuration system
- Geo-audit feature (multi-location testing)

### [0.5.0] - API & Integration
- REST API server (Axum/Actix)
- WebSocket support for real-time progress
- Job management and queuing
- Rust SDK with fluent API
- Tauri desktop app (cross-platform)

### [1.0.0] - Production Release
- State persistence and resume functionality
- Advanced CLI features (expert mode, non-interactive)
- System health monitoring
- Audit debugger
- Comprehensive test coverage
- CI/CD integration examples
- Docker image
- Homebrew formula

---

**Repository:** https://github.com/casoon/auditmysit_rust  
**Documentation:** See `.claude/COMPREHENSIVE_PROJECT_PLAN.md` for full roadmap
