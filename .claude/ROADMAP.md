# auditmysite - Improvement Roadmap

> Code Review durchgeführt am 2026-01-30
> Gesamtbewertung: B+ (Gut mit Verbesserungspotenzial)

## Übersicht

Dieses Dokument beschreibt die priorisierten Verbesserungen basierend auf dem Code-Review.

---

## Phase 1: Kritische Fixes (Sofort)

**Ziel:** Production-Ready Stabilität

### 1.1 `.expect()` durch Result ersetzen
- [ ] `main.rs:62` - `set_global_default(subscriber).expect(...)`
- [ ] `main.rs:94` - `args.url.as_ref().expect("URL required...")`
- [ ] `main.rs:210` - `.expect("Invalid template")`
- [ ] `browser/installer.rs` - `.expect("Could not find home directory")`

### 1.2 Pool Reset Timeout
- [ ] `browser/pool.rs` - Timeout für `page.goto("about:blank")` hinzufügen
- [ ] Graceful degradation bei Reset-Fehlern

### 1.3 Duplicate Enum Variant fixen
- [ ] `cli/args.rs` - Doppeltes `Markdown` Variant entfernen

**Geschätzter Aufwand:** 2-3 Stunden

---

## Phase 2: Error Handling (Kurzfristig)

**Ziel:** Keine stillen Fehler mehr

### 2.1 `.unwrap_or()` Audit
Ersetzen durch proper error handling in:
- [ ] `seo/schema.rs` - JSON parsing
- [ ] `seo/meta.rs` - Meta tag extraction
- [ ] `seo/headings.rs` - Heading analysis
- [ ] `seo/social.rs` - Social tags
- [ ] `seo/technical.rs` - Technical SEO
- [ ] `performance/vitals.rs` - Web Vitals
- [ ] `performance/content_weight.rs` - Content analysis
- [ ] `accessibility/extractor.rs` - AXTree extraction
- [ ] `wcag/contrast.rs` - Color parsing

### 2.2 Error Logging verbessern
- [ ] Warn-Level für recoverable errors
- [ ] Error-Level für critical failures
- [ ] Debug-Level für development info

**Geschätzter Aufwand:** 3-4 Stunden

---

## Phase 3: Security Hardening (Kurzfristig)

**Ziel:** Production-sichere Anwendung

### 3.1 SSRF Protection
- [ ] Optional `--allowed-domains` Flag hinzufügen
- [ ] URL Validation vor Sitemap-Fetch
- [ ] Private IP Ranges blockieren (10.x, 192.168.x, localhost)

### 3.2 Chromium Download Verification
- [ ] SHA256 Checksum nach Download prüfen
- [ ] Trusted CDN URLs hardcoden

### 3.3 HTML Report Escaping
- [ ] User-Input in HTML Reports escapen
- [ ] XSS Prevention in `output/html.rs`

### 3.4 File Path Validation
- [ ] Path Traversal Protection in `read_url_file()`
- [ ] Relative Paths validieren

**Geschätzter Aufwand:** 3-4 Stunden

---

## Phase 4: Testing (Mittelfristig)

**Ziel:** Confidence bei Releases

### 4.1 Integration Tests
- [ ] End-to-End Test mit echtem Browser
- [ ] Batch Processing Pipeline Test
- [ ] Alle Output-Formate testen (JSON, HTML, PDF, Markdown)

### 4.2 Error Path Tests
- [ ] Navigation Failure Handling
- [ ] Timeout Scenarios
- [ ] Pool Exhaustion

### 4.3 Performance Tests
- [ ] Benchmark: Single Audit < 3 Sekunden
- [ ] Memory Usage Monitoring

**Geschätzter Aufwand:** 6-8 Stunden

---

## Phase 5: Documentation (Mittelfristig)

**Ziel:** Onboarding und Community

### 5.1 Architektur-Dokumentation
- [ ] `docs/ARCHITECTURE.md` - System Design und Data Flow
- [ ] Modul-Diagramme

### 5.2 Contributor Guide
- [ ] `docs/CONTRIBUTING.md` - Wie man beiträgt
- [ ] `docs/ADDING_RULES.md` - Neue WCAG Regeln hinzufügen

### 5.3 Troubleshooting
- [ ] `docs/TROUBLESHOOTING.md` - Häufige Probleme und Lösungen
- [ ] Chrome nicht gefunden
- [ ] Timeout Issues
- [ ] Pool Exhaustion

### 5.4 API Documentation
- [ ] `examples/library_usage.rs` - Programmatische Nutzung
- [ ] Library vs CLI Dokumentation

**Geschätzter Aufwand:** 4-6 Stunden

---

## Phase 6: Features (Backlog)

### 6.1 API Server Feature
- [ ] `--serve` Flag für REST API
- [ ] OpenAPI Specification
- [ ] Rate Limiting

### 6.2 Desktop App Feature
- [ ] Tauri Integration
- [ ] Native GUI

### 6.3 Performance Optimizations
- [ ] CDP Calls batchen
- [ ] Style Caching zwischen Checks
- [ ] Request Cancellation in Batch Processing

**Geschätzter Aufwand:** 20+ Stunden

---

## Dependency Cleanup

### Sofort
- [ ] `renderreport` Git-Dependency auf Commit-Hash pinnen

### Später
- [ ] `anyhow` entfernen, nur `thiserror` nutzen
- [ ] `Cargo.lock` committen für reproducible builds

---

## Metriken

| Kategorie | Aktuell | Ziel |
|-----------|---------|------|
| Build Warnings | 0 | 0 |
| `.expect()` calls | 4 | 0 |
| `.unwrap_or()` calls | 50+ | <10 |
| Test Coverage | ~80% | >90% |
| Integration Tests | 0 | 10+ |
| Documentation | Basic | Comprehensive |

---

## Changelog

### 2026-01-30
- Initial Roadmap erstellt nach Code Review
- Phase 1 gestartet
