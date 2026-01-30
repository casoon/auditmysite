# Feature Parity Analysis: auditmysit_rust vs TypeScript Projects

**Document Version:** 1.0  
**Date:** 2026-01-30  
**Scope:** CLI-only features for WCAG accessibility testing  
**Exclusions:** Desktop UI (auditmysite_studio), REST API endpoints, obsolete CLI

---

## Executive Summary

This document compares the Rust CLI tool (`auditmysit_rust`) with the TypeScript-based `auditmysite` project to establish feature parity for command-line WCAG testing. The goal is to identify which features should be ported to Rust while explicitly excluding Desktop UI components (Flutter/Tauri) and REST API endpoints.

**Key Findings:**
- ✅ **Core WCAG Testing:** Full parity achievable for WCAG 2.1 A/AA/AAA compliance checking
- ✅ **Batch Processing:** Sitemap parsing, queue management, and resume capabilities can be ported
- ✅ **Report Generation:** Transition from HTML/JSON to **Typst PDF output** via `renderreport`
- ❌ **Excluded:** 3,716 lines of Flutter UI code, REST API server, WebSocket endpoints, SDK library
- 📊 **Priority:** Focus on pa11y-equivalent rules + renderreport integration for MVP

---

## 1. WCAG Rules & Compliance Testing

### 1.1 TypeScript Implementation (auditmysite)

#### Primary Analyzers
| Analyzer | Version | Purpose | CLI-Relevant? |
|----------|---------|---------|---------------|
| **pa11y** | 9.0.0 | Core WCAG 2.1 A/AA/AAA checker | ✅ YES |
| **axe-core** | 4.10.0 | Secondary accessibility engine | ✅ YES |

#### WCAG Standards Supported
- ✅ **WCAG 2.1 Level A** (`WCAG2A`)
- ✅ **WCAG 2.1 Level AA** (`WCAG2AA`) - Default/Recommended
- ✅ **WCAG 2.1 Level AAA** (`WCAG2AAA`)
- ✅ **Section 508** (US Federal requirement)

#### Custom WCAG 2.1 Checks (9 test suites)

| Test Suite | WCAG Principle | Key Checks | Implementation Status |
|------------|----------------|------------|----------------------|
| `FormLabelTest` | Perceivable/Operable | Form labels, ARIA labels, input associations | 🟡 Partial (4.1.2 implemented) |
| `KeyboardNavigationTest` | Operable | Tab order, focus indicators, ARIA navigation | ⚪ Not started |
| `AriaLandmarksTest` | Perceivable | Navigation landmarks, ARIA roles | ⚪ Not started |
| `SemanticHtmlTest` | Robust | HTML5 semantic elements, proper nesting | ⚪ Not started |
| `MediaAccessibilityTest` | Perceivable | Captions, transcripts, audio descriptions | ⚪ Not started |
| `PerformanceLoadingTest` | Operable | Page load timeouts, progressive rendering | ⚪ Not started |
| `ValidationErrorHandlingTest` | Understandable | Error messages, ARIA alerts | ⚪ Not started |
| `LanguageI18nTest` | Understandable | `lang` attributes, i18n support | ⚪ Not started |
| `LanguageTextDirectionTest` | Understandable | RTL/LTR support via `dir` attribute | ⚪ Not started |

#### Custom WCAG Checks by Principle

**Perceivable:**
- ✅ Color contrast analysis (1.4.3) - **IMPLEMENTED in Rust**
- ✅ Text alternatives (1.1.1 - images without alt) - **IMPLEMENTED in Rust**
- ⚪ Captions for media
- ⚪ Adaptable content (multiple modalities)

**Operable:**
- ⚪ Keyboard navigation testing
- ⚪ ARIA landmarks verification
- ⚪ Focus management
- ⚪ Touch target analysis (mobile)

**Understandable:**
- ⚪ Language/i18n testing (`lang`, `dir` attributes)
- ✅ Form labels (4.1.2 - labels/buttons) - **IMPLEMENTED in Rust**
- ⚪ Error handling patterns

**Robust:**
- ⚪ HTML5 semantic elements validation
- ✅ ARIA rules compliance (basic checks) - **PARTIAL in Rust**
- ⚪ Mobile touch target analysis (8-category evaluation)

#### Glassmorphism False-Positive Detection
- **Feature:** Automatic detection of `backdrop-filter` and transparent backgrounds
- **Impact:** Filters incorrect color-contrast errors caused by blurred/glass backgrounds
- **Metadata:** `filteringMetadata` with transparency info
- **Status:** ⚪ Not implemented in Rust (consider for enhancement phase)

### 1.2 Rust Implementation Status (auditmysit_rust)

#### Current WCAG Rules (MVP - 3 rules)
| Rule Code | WCAG Principle | Description | File |
|-----------|----------------|-------------|------|
| **1.1.1** | Perceivable | Non-text Content (images missing alt) | `src/wcag/rules/text_alternatives.rs` |
| **2.4.6** | Operable | Headings and Labels | `src/wcag/rules/labels.rs` |
| **4.1.2** | Robust | Name, Role, Value (ARIA) | `src/wcag/rules/labels.rs` |

#### Planned Enhancement Rules (Phase 2)
| Rule Code | Description | Priority | Estimated Effort |
|-----------|-------------|----------|------------------|
| **1.4.3** | Contrast (Minimum) | HIGH | 2-3 days (color parsing + math) |
| **2.1.1** | Keyboard navigation | MEDIUM | 3-5 days (interaction simulation) |
| **3.1.1** | Language of Page (`lang` attribute) | HIGH | 1 day |
| **4.1.1** | Parsing (HTML validation) | LOW | 2-3 days |
| **1.3.1** | Info and Relationships (semantic HTML) | MEDIUM | 2-3 days |

### 1.3 Gap Analysis: WCAG Rules

| Feature | TypeScript (pa11y) | Rust (Current) | Action Required |
|---------|-------------------|----------------|-----------------|
| Image alt text (1.1.1) | ✅ | ✅ | None - complete |
| Form labels (4.1.2) | ✅ | ✅ | None - complete |
| Heading structure (2.4.6) | ✅ | ✅ | None - complete |
| Contrast checking (1.4.3) | ✅ | ❌ | **Implement via CDP CSS.getComputedStyleForNode** |
| Language attribute (3.1.1) | ✅ | ❌ | **Extract from AXTree or DOM** |
| Keyboard navigation (2.1.1) | ✅ | ❌ | **Simulate via CDP Input.dispatchKeyEvent** |
| ARIA landmarks (2.4.1) | ✅ | ❌ | **Parse AXTree for landmark roles** |
| Semantic HTML (1.3.1) | ✅ | ❌ | **Validate HTML5 elements via DOM** |
| Glassmorphism detection | ✅ | ❌ | Low priority - enhancement phase |

**Recommendation:** Focus on contrast (1.4.3), language (3.1.1), and ARIA landmarks for Phase 2 to achieve 80% parity with pa11y core checks.

---

## 2. CLI Features & Architecture

### 2.1 TypeScript CLI Capabilities

#### Command Structure
```bash
auditmysite <sitemapUrl> [options]
```

#### Core CLI Options (12 parameters)

| Category | Option | Default | Description | Rust Status |
|----------|--------|---------|-------------|-------------|
| **Input** | `<sitemapUrl>` | Required | Sitemap URL or local file | ✅ Implemented |
| **Batch** | `--max-pages <N>` | 5 | Number of pages to test | ❌ Not implemented |
| **Output** | `--format <type>` | `html` | `html`, `json`, `csv` | ✅ JSON/table (HTML ❌) |
| **Output** | `--output-dir <dir>` | `./reports` | Output directory | ✅ `reports/` |
| **Perf** | `--budget <template>` | `default` | Performance budget preset | ❌ Not implemented |
| **UX** | `--expert` | Off | Interactive expert mode | ❌ Not implemented |
| **UX** | `--non-interactive` | Off | CI/CD mode (skip prompts) | ✅ Default behavior |
| **UX** | `--verbose` | Off | Detailed progress | ❌ Not implemented |
| **Opt-Out** | `--no-performance` | Off | Disable performance analysis | ❌ N/A (not implemented) |
| **Opt-Out** | `--no-seo` | Off | Disable SEO analysis | ❌ N/A (not implemented) |
| **Opt-Out** | `--no-mobile` | Off | Disable mobile checks | ❌ N/A (not implemented) |
| **Resume** | `--resume <stateId>` | - | Resume from saved state | ❌ Not implemented |

#### Advanced Features

**Sitemap Discovery (Auto-Discovery):**
1. Direct URL parsing (if `sitemap.xml` provided)
2. `robots.txt` parsing for sitemap location
3. Common paths (`/sitemap.xml`, `/wp-sitemap.xml`, etc.)
4. Sitemap index support (WordPress multi-sitemap)
5. Recursion limits (max depth 5, max 10 sub-sitemaps)

**Status:** ❌ Not implemented in Rust (manual sitemap URL required)

**Smart URL Sampling:**
- Homepage-first testing with redirect handling
- Filters 301/302 redirects automatically
- Parallel minimal checks for redirect detection

**Status:** ❌ Not implemented in Rust

**Queue System (Batch Processing):**
- Multiple adapters: `simple`, `parallel`, `priority`, `persistent`
- Configurable concurrency (1-5 workers, default: 2)
- Retry mechanism (max 3 attempts, 2s delay)
- Progress reporting with ETA
- Queue statistics (throughput, error rates, CPU/memory)

**Status:** ❌ Not implemented in Rust (single-URL only)

**Resume/Persistence:**
- Save state to filesystem (`.auditmysite-states/`)
- List all saved states with progress info
- Resume from specific state ID
- Tracks: status, processed URLs, total URLs, last update

**Status:** ❌ Not implemented in Rust

### 2.2 Rust CLI Current State

#### Implemented Features
```bash
auditmysit <URL> [options]
```

| Option | Description | Example |
|--------|-------------|---------|
| `<URL>` | Single URL to audit | `https://example.com` |
| `-l, --level <LEVEL>` | WCAG level (A/AA/AAA) | `--level AA` |
| `-f, --format <FORMAT>` | Output format (json/table/html) | `--format json` |
| `-o, --output <FILE>` | Output file path | `-o reports/audit.json` |
| `--chrome-path <PATH>` | Chrome executable path | `--chrome-path /usr/bin/chromium` |
| `--remote-debugging-port <PORT>` | Use existing Chrome instance | `--remote-debugging-port 9222` |

#### Current Limitations
- ❌ Single URL only (no batch processing)
- ❌ No sitemap parsing
- ❌ No resume/persistence
- ❌ No performance/SEO analysis
- ❌ No expert/interactive mode
- ❌ Limited output formats (no HTML report generator)

### 2.3 Gap Analysis: CLI Features

| Feature | TypeScript | Rust | Priority | Estimated Effort |
|---------|------------|------|----------|------------------|
| Sitemap parsing | ✅ | ❌ | **HIGH** | 3-5 days |
| Batch processing (queue) | ✅ | ❌ | **HIGH** | 5-7 days |
| Resume/persistence | ✅ | ❌ | MEDIUM | 3-4 days |
| Performance analysis | ✅ | ❌ | LOW | 7-10 days |
| SEO analysis | ✅ | ❌ | LOW | 5-7 days |
| Expert mode | ✅ | ❌ | LOW | 2-3 days |
| Verbose logging | ✅ | ❌ | MEDIUM | 1-2 days |
| --max-pages flag | ✅ | ❌ | HIGH | 1 day |

**Recommendation:** Prioritize sitemap parsing + batch processing for Phase 2 to enable multi-page audits.

---

## 3. Report Generation & Output Formats

### 3.1 TypeScript Report Formats

#### HTML Report (Professional Dashboard)
- **Components:**
  - Professional header with certificate SVG badge
  - Sticky navigation with anchor links
  - Executive summary with graphs (Chart.js)
  - Detailed sections: Accessibility, Performance, SEO, Content Weight, Mobile
  - Individual page results with filtering
  - Modern CSS with dark gradient theme
  - Responsive design
  
- **Status:** ❌ Not implemented in Rust (replaced by Typst PDF)

#### JSON Report (Structured Data)
```typescript
{
  metadata: {
    timestamp: string,
    sitemapUrl: string,
    totalPages: number,
    testedPages: number,
    duration: number,
    wcagLevel: 'A' | 'AA' | 'AAA'
  },
  summary: {
    overallScore: number,  // 0-100
    overallGrade: 'A'-'F',
    certificateLevel: 'PLATINUM' | 'GOLD' | 'SILVER' | 'BRONZE' | 'NEEDS_IMPROVEMENT',
    totalErrors: number,
    totalWarnings: number
  },
  pages: [{
    url: string,
    title: string,
    accessibility: {
      score: number,
      wcagLevel: 'A' | 'AA' | 'AAA' | 'none',
      errors: AccessibilityIssue[],
      warnings: AccessibilityIssue[],
      filteringMetadata?: { glassmorphismDetected: boolean }
    },
    performance?: { ... },
    seo?: { ... }
  }]
}
```

**Status:** ✅ Implemented in Rust (`src/output/json.rs`)

#### CLI Table Output (Terminal Display)
- **Library:** `prettytable-rs`
- **Columns:** Rule, Severity, Message, Element
- **Color coding:** Red (errors), Yellow (warnings)

**Status:** ✅ Implemented in Rust (`src/output/cli.rs`)

### 3.2 Rust Report Generation (Current + Planned)

#### Current Formats
| Format | File | Status | Library |
|--------|------|--------|---------|
| JSON | `src/output/json.rs` | ✅ Implemented | `serde_json` |
| CLI Table | `src/output/cli.rs` | ✅ Implemented | `prettytable-rs` |
| HTML | `src/output/html.rs` | ❌ Basic stub only | - |

#### Planned: Typst PDF Output (via renderreport)

**Integration Strategy:**

1. **Add renderreport dependency** to `Cargo.toml`:
   ```toml
   renderreport = "0.1.0-alpha.1"  # When published
   ```

2. **Create WCAG-specific pack** (`src/output/typst/wcag_pack.rs`):
   ```rust
   use renderreport::{Pack, Component, Theme};
   
   pub struct WCAGPack;
   
   impl Pack for WCAGPack {
       fn components(&self) -> Vec<Box<dyn Component>> {
           vec![
               Box::new(WCAGFinding::default()),
               Box::new(WCAGScoreCard::default()),
               Box::new(WCAGViolationTable::default()),
               Box::new(WCAGSummaryChart::default()),
           ]
       }
       
       fn theme(&self) -> Theme {
           Theme::builder()
               .primary_color("#2563eb")  // WCAG blue
               .danger_color("#dc2626")   // Error red
               .warning_color("#f59e0b")  // Warning amber
               .build()
       }
   }
   ```

3. **Map audit data to components** (`src/output/typst.rs`):
   ```rust
   use renderreport::{Engine, Report, ScoreCard, Finding, AuditTable, Severity};
   use crate::audit::report::AuditReport;
   
   pub struct TypstReportGenerator {
       engine: Engine,
   }
   
   impl TypstReportGenerator {
       pub fn new() -> Result<Self> {
           let engine = Engine::builder()
               .pack(WCAGPack)
               .build()?;
           Ok(Self { engine })
       }
       
       pub fn generate(&self, audit_report: &AuditReport) -> Result<Vec<u8>> {
           let report = self.engine
               .report("wcag-audit")
               .title(&format!("WCAG 2.1 Audit Report - {}", audit_report.url))
               .add_component(
                   ScoreCard::new("Overall Score", audit_report.score as i32)
                       .grade(calculate_grade(audit_report.score))
               )
               .add_component(
                   Finding::new(
                       &format!("WCAG {} Compliance", audit_report.wcag_level),
                       if audit_report.passed { Severity::Info } else { Severity::High },
                       &format!("{} violations found", audit_report.violations.len())
                   )
               );
           
           // Add violations table
           let mut table = AuditTable::new(vec!["Rule", "Severity", "Message", "Element"]);
           for violation in &audit_report.violations {
               table.add_row(vec![
                   &violation.rule,
                   &format!("{:?}", violation.severity),
                   &violation.message,
                   &violation.node_id
               ]);
           }
           report.add_component(table);
           
           // Render PDF
           let pdf_bytes = self.engine.render_pdf(&report.build())?;
           Ok(pdf_bytes)
       }
   }
   
   fn calculate_grade(score: f32) -> &'static str {
       match score as i32 {
           90..=100 => "A",
           80..=89 => "B",
           70..=79 => "C",
           60..=69 => "D",
           _ => "F"
       }
   }
   ```

4. **Component mapping strategy:**

| Audit Data | renderreport Component | Purpose |
|------------|------------------------|---------|
| `AuditReport.score` | `ScoreCard` | Overall score display (0-100) |
| `AuditReport.violations[]` | `Finding` | Individual WCAG violations |
| `AuditReport.violations[]` | `AuditTable` | Tabular violation listing |
| `AuditReport.summary` | `Chart` (bar chart) | Violations by severity |
| `AuditReport.wcag_level` | `StatusIndicator` | Compliance level badge |
| `AuditReport.passed_rules[]` | `InfoBox` | Passed rules summary |

5. **Update CLI to support Typst output:**
   ```rust
   // src/cli/args.rs
   #[derive(Parser)]
   pub struct Cli {
       #[arg(short = 'f', long, default_value = "table")]
       pub format: String,  // table, json, html, pdf
       
       #[arg(short = 'o', long)]
       pub output: Option<PathBuf>,
   }
   
   // src/main.rs
   match args.format.as_str() {
       "json" => { /* JSON output */ },
       "table" => { /* CLI table */ },
       "pdf" => {
           let generator = TypstReportGenerator::new()?;
           let pdf_bytes = generator.generate(&report)?;
           std::fs::write(output_path, pdf_bytes)?;
           println!("PDF report saved to {}", output_path.display());
       },
       _ => return Err("Invalid format".into()),
   }
   ```

### 3.3 Gap Analysis: Report Generation

| Feature | TypeScript | Rust Current | Rust Planned (Typst) |
|---------|------------|--------------|----------------------|
| JSON export | ✅ | ✅ | ✅ |
| CLI table | ✅ | ✅ | ✅ |
| HTML report | ✅ | ❌ | ❌ (replaced by PDF) |
| **PDF report** | ❌ | ❌ | ✅ **via renderreport** |
| Certificate badge | ✅ (SVG in HTML) | ❌ | ✅ (ScoreCard component) |
| Violation charts | ✅ (Chart.js) | ❌ | ✅ (Chart component) |
| Severity color coding | ✅ | ✅ (CLI only) | ✅ (Theme system) |

**Recommendation:** Implement Typst PDF output as primary report format. Skip HTML generation since PDF provides professional output suitable for compliance documentation.

---

## 4. Scoring & Grading System

### 4.1 TypeScript Scoring

#### Accessibility Score Calculation
```typescript
let score = 100.0;

// Deduction penalties
score -= errors.length * 2.5;    // 2.5 pts per error
score -= warnings.length * 1.0;   // 1.0 pt per warning

// Specific penalties
if (imagesWithoutAlt > 0) score -= 3;
if (buttonsWithoutLabel > 0) score -= 5;
if (headingsCount === 0) score -= 20;

return Math.max(0, score);
```

#### Grade Calculation
| Score Range | Grade |
|-------------|-------|
| 90-100% | A |
| 80-89% | B |
| 70-79% | C |
| 60-69% | D |
| 0-59% | F |

#### Certificate Levels
| Score Range | Certificate |
|-------------|-------------|
| ≥95% | PLATINUM |
| ≥85% | GOLD |
| ≥75% | SILVER |
| ≥65% | BRONZE |
| <65% | NEEDS_IMPROVEMENT |

### 4.2 Rust Scoring (To Implement)

**Current Status:** ❌ No scoring system implemented

**Implementation Plan:**
```rust
// src/audit/scoring.rs
pub struct AccessibilityScorer;

impl AccessibilityScorer {
    pub fn calculate_score(violations: &[Violation]) -> f32 {
        let errors = violations.iter().filter(|v| matches!(v.severity, Severity::Error)).count();
        let warnings = violations.iter().filter(|v| matches!(v.severity, Severity::Warning)).count();
        
        let mut score = 100.0;
        score -= errors as f32 * 2.5;
        score -= warnings as f32 * 1.0;
        
        // Specific penalties
        if violations.iter().any(|v| v.rule == "1.1.1") {
            score -= 3.0;  // Images without alt
        }
        if violations.iter().any(|v| v.rule == "4.1.2") {
            score -= 5.0;  // Buttons without label
        }
        if violations.iter().any(|v| v.rule == "2.4.6") {
            score -= 20.0;  // No headings
        }
        
        score.max(0.0)
    }
    
    pub fn calculate_grade(score: f32) -> &'static str {
        match score as u32 {
            90..=100 => "A",
            80..=89 => "B",
            70..=79 => "C",
            60..=69 => "D",
            _ => "F"
        }
    }
    
    pub fn calculate_certificate(score: f32) -> &'static str {
        match score as u32 {
            95..=100 => "PLATINUM",
            85..=94 => "GOLD",
            75..=84 => "SILVER",
            65..=74 => "BRONZE",
            _ => "NEEDS_IMPROVEMENT"
        }
    }
}
```

**Priority:** MEDIUM (needed for renderreport ScoreCard component)

---

## 5. Secondary Analyzers (Out of Scope for CLI MVP)

### Implemented in TypeScript (Not Porting to Rust CLI)

| Analyzer | Purpose | Lines of Code | Reason for Exclusion |
|----------|---------|---------------|---------------------|
| **PerformanceCollector** | Core Web Vitals (LCP, FCP, CLS, TTFB) | ~400 | Scope creep - focus on WCAG |
| **MobilePerformanceCollector** | Mobile-specific performance | ~350 | Not core accessibility |
| **SEOAnalyzer** | Meta tags, headings, content quality | ~500 | Different domain (SEO vs a11y) |
| **ContentWeightAnalyzer** | Resource size analysis | ~300 | Performance, not accessibility |
| **MobileFriendlinessAnalyzer** | Touch targets, responsive design | ~450 | Partial overlap with WCAG 2.5.5 |
| **SecurityHeadersAnalyzer** | Security headers validation | ~200 | Different domain (security) |
| **StructuredDataAnalyzer** | Schema.org/JSON-LD | ~300 | SEO-focused |

**Total LOC Excluded:** ~2,500 lines

**Rationale:** The Rust CLI tool should focus exclusively on WCAG 2.1 compliance testing. Performance, SEO, and security analysis are valuable but belong in separate tools or future extensions.

**Partial Exception:** Mobile touch target analysis (from MobileFriendlinessAnalyzer) overlaps with WCAG 2.5.5 (Target Size). Consider implementing this specific rule in Phase 3.

---

## 6. Configuration & Advanced Options

### 6.1 TypeScript Configuration System

#### Performance Budget Templates

| Template | LCP (good) | FCP (good) | CLS (good) | TTFB (good) |
|----------|-----------|-----------|-----------|-------------|
| **Default** | 2500ms | 1800ms | 0.1 | 400ms |
| **E-commerce** | 2000ms | 1500ms | 0.05 | 350ms |
| **Corporate** | 2200ms | 1600ms | 0.08 | 350ms |
| **Blog** | 3000ms | 2000ms | 0.1 | 400ms |

**Status:** ❌ Not applicable for Rust CLI (performance analysis excluded)

#### Configuration Priority Chain
1. CLI arguments (highest)
2. Environment variables
3. Config file (`auditmysite.config.js/json/ts`)
4. `package.json` (`auditmysite` field)
5. Preset configurations
6. Built-in defaults (lowest)

**Status:** ⚪ Partial (CLI args only, no config file support)

#### Viewport & Network Settings

| Setting | Desktop | Mobile | Tablet |
|---------|---------|--------|--------|
| **Viewport** | 1920x1080 | 375x667 | 768x1024 |
| **User Agent** | Chrome Desktop | iPhone | iPad |
| **Network** | WiFi | 4G | WiFi |

**Status:** ⚪ Not configurable in Rust (uses default viewport)

#### Wait Conditions
- `domcontentloaded` (default)
- `load` (full page load)
- `networkidle0` / `networkidle2`
- Custom selector visibility

**Status:** ⚪ Hardcoded in Rust (uses `load` event)

### 6.2 Rust Configuration (Current)

**Implemented:**
- ✅ WCAG level selection (`--level A/AA/AAA`)
- ✅ Output format (`--format json/table/html`)
- ✅ Chrome executable path (`--chrome-path`)
- ✅ Remote debugging port (`--remote-debugging-port`)

**Missing:**
- ❌ Configuration file support
- ❌ Environment variable configuration
- ❌ Viewport customization
- ❌ Network throttling
- ❌ Wait condition customization
- ❌ User agent customization

**Priority:** LOW (CLI args sufficient for MVP)

---

## 7. Exclusions Summary

### 7.1 Desktop UI Components (auditmysite_studio - Flutter)

**Total Excluded:** 3,716 lines of Flutter UI code

| Component Category | Files | Reason |
|-------------------|-------|--------|
| Screen Components | 7 screens (Audit, Results, Progress, Settings, Error, Splash, RunSetup) | CLI tool has no GUI |
| Widget Library | ModernCard, GlassCard, StatCard, Toast, etc. | Terminal-only output |
| Theme System | Light/dark mode, Material Design 3 | Not applicable |
| State Management | Riverpod providers, StateNotifiers | CLI is stateless |
| Services | EngineClient (WebSocket), SettingsService, ResultsManager | Server-side features |
| Platform Code | macOS/Windows native integration | CLI is cross-platform |

### 7.2 REST API & Server Components

**Excluded Endpoints:**

| Method | Path | Purpose | Reason |
|--------|------|---------|--------|
| GET | `/health` | Health check | Server-only |
| GET | `/status` | Service status | Server-only |
| POST | `/audit` | Start async audit | Server-only |
| WebSocket | `/ws` | Event streaming | Server-only |

**Excluded Architecture:**
- `shelf` HTTP server framework
- CORS middleware
- WebSocket connection management
- Async job management with run IDs
- API request/response schemas

### 7.3 Obsolete CLI (auditmysite_cli - Dart)

**Status:** Marked as obsolete in original project

**Excluded Features:**
- HTML report builder (replaced by Typst)
- CSV export (not needed)
- Template system (replaced by renderreport components)

---

## 8. Implementation Roadmap

### Phase 1: MVP Enhancement (Weeks 1-2)
**Goal:** Achieve core parity with pa11y for WCAG 2.1 AA testing

| Task | Estimated Effort | Priority |
|------|------------------|----------|
| Implement scoring system | 1 day | HIGH |
| Add contrast checking (1.4.3) | 2-3 days | HIGH |
| Add language attribute check (3.1.1) | 1 day | HIGH |
| Implement ARIA landmarks (2.4.1) | 2 days | MEDIUM |
| Add `--verbose` logging | 1 day | MEDIUM |
| **Total** | **7-9 days** | |

### Phase 2: Batch Processing (Weeks 3-4)
**Goal:** Enable multi-page audits via sitemap parsing

| Task | Estimated Effort | Priority |
|------|------------------|----------|
| Sitemap XML parsing | 2 days | HIGH |
| Sitemap auto-discovery (robots.txt, common paths) | 2-3 days | HIGH |
| Implement queue system | 3-4 days | HIGH |
| Add `--max-pages` flag | 1 day | HIGH |
| Redirect filtering | 1-2 days | MEDIUM |
| **Total** | **9-12 days** | |

### Phase 3: Typst PDF Output (Week 5)
**Goal:** Replace HTML reports with professional PDF via renderreport

| Task | Estimated Effort | Priority |
|------|------------------|----------|
| Create WCAG pack for renderreport | 2-3 days | HIGH |
| Implement TypstReportGenerator | 2 days | HIGH |
| Map audit data to components | 1 day | HIGH |
| Add `--format pdf` CLI option | 0.5 days | HIGH |
| Design WCAG theme (colors, fonts) | 1 day | MEDIUM |
| **Total** | **6.5-7.5 days** | |

### Phase 4: Resume & Persistence (Week 6)
**Goal:** Enable resuming interrupted audits

| Task | Estimated Effort | Priority |
|------|------------------|----------|
| Design state serialization schema | 1 day | MEDIUM |
| Implement state saving/loading | 2-3 days | MEDIUM |
| Add `--resume <stateId>` flag | 1 day | MEDIUM |
| Add `--list-states` command | 0.5 days | LOW |
| **Total** | **4.5-5.5 days** | |

### Phase 5: Advanced WCAG Rules (Weeks 7-8)
**Goal:** Expand beyond MVP rules to 15+ WCAG checks

| Task | Estimated Effort | Priority |
|------|------------------|----------|
| Keyboard navigation simulation (2.1.1) | 3-4 days | MEDIUM |
| Semantic HTML validation (1.3.1) | 2-3 days | MEDIUM |
| Error handling patterns (3.3.1) | 2 days | LOW |
| Touch target analysis (2.5.5) | 2-3 days | LOW |
| Media accessibility (1.2.x) | 3-4 days | LOW |
| **Total** | **12-16 days** | |

### Total Timeline: 8 weeks (39-50 days of development)

---

## 9. Success Metrics

### Feature Parity Targets

| Metric | Target | Current | Gap |
|--------|--------|---------|-----|
| **WCAG Rules Implemented** | 15+ rules | 3 rules | +12 rules |
| **Batch Processing** | Sitemap + queue | Single URL | Not started |
| **Output Formats** | JSON + PDF | JSON + table | Add PDF |
| **Scoring System** | Score + grade + certificate | None | Not started |
| **Resume Capability** | Yes | No | Not started |
| **CLI Options** | 12+ flags | 6 flags | +6 flags |

### Quality Targets

| Metric | Target |
|--------|--------|
| **Test Coverage** | ≥80% for rule engine |
| **Binary Size** | <15 MB (release build) |
| **Memory Usage** | <300 MB per instance |
| **Audit Speed** | <3s per page (typical site) |
| **WCAG Compliance** | Detect 90%+ of pa11y violations |

---

## 10. renderreport Integration Details

### 10.1 Component Mapping Strategy

#### ScoreCard Component
**Purpose:** Display overall accessibility score prominently

```rust
ScoreCard::new("WCAG 2.1 AA Compliance", audit_report.score as i32)
    .grade(calculate_grade(audit_report.score))
    .subtitle(&format!("{} pages tested", audit_report.tested_pages))
    .color(match audit_report.score as i32 {
        90..=100 => "#10b981",  // Green
        70..=89 => "#f59e0b",   // Amber
        _ => "#ef4444"          // Red
    })
```

#### Finding Component
**Purpose:** Display individual WCAG violations

```rust
for violation in &audit_report.violations {
    Finding::new(
        &format!("WCAG {} - {}", violation.rule, get_rule_name(&violation.rule)),
        match violation.severity {
            Severity::Error => renderreport::Severity::High,
            Severity::Warning => renderreport::Severity::Medium,
            Severity::Notice => renderreport::Severity::Low,
        },
        &violation.message
    )
    .context(&format!("Element: {}", violation.node_id))
    .recommendation(get_fix_recommendation(&violation.rule))
}
```

#### AuditTable Component
**Purpose:** Tabular listing of all violations

```rust
let mut table = AuditTable::new(vec!["Rule", "Severity", "Element", "Message"]);
table.set_striped(true);
table.set_header_style(TableHeaderStyle::Bold);

for violation in &audit_report.violations {
    table.add_row(vec![
        &violation.rule,
        &format!("{:?}", violation.severity),
        &violation.node_id,
        &violation.message
    ]);
}
```

#### Chart Component
**Purpose:** Visualize violations by severity or rule category

```rust
let error_count = violations.iter().filter(|v| v.severity == Severity::Error).count();
let warning_count = violations.iter().filter(|v| v.severity == Severity::Warning).count();

Chart::bar()
    .title("Violations by Severity")
    .data(vec![
        ("Errors", error_count as f32),
        ("Warnings", warning_count as f32),
    ])
    .color_scheme(vec!["#ef4444", "#f59e0b"])
```

### 10.2 WCAG-Specific Pack Structure

```rust
// src/output/typst/wcag_pack.rs
pub struct WCAGPack;

impl Pack for WCAGPack {
    fn name(&self) -> &str {
        "wcag-audit"
    }
    
    fn components(&self) -> Vec<Box<dyn Component>> {
        vec![
            Box::new(WCAGHeader::default()),      // Custom header with WCAG logo
            Box::new(ComplianceBadge::default()), // Certificate level badge
            Box::new(ViolationSummary::default()), // Executive summary
            Box::new(RuleBreakdown::default()),   // Breakdown by WCAG principle
        ]
    }
    
    fn theme(&self) -> Theme {
        Theme::builder()
            .primary_color("#2563eb")      // WCAG blue
            .secondary_color("#8b5cf6")    // Purple accent
            .success_color("#10b981")      // Green (passed)
            .warning_color("#f59e0b")      // Amber (warnings)
            .danger_color("#ef4444")       // Red (errors)
            .font_family("Inter")          // Modern sans-serif
            .heading_font("Inter")
            .mono_font("JetBrains Mono")   // Code/selectors
            .build()
    }
    
    fn templates(&self) -> Vec<Template> {
        vec![
            Template::new("wcag-audit", include_str!("templates/wcag_audit.typ")),
            Template::new("executive-summary", include_str!("templates/summary.typ")),
        ]
    }
}
```

### 10.3 Custom WCAG Components

#### ComplianceBadge Component
```rust
pub struct ComplianceBadge {
    level: String,        // "PLATINUM", "GOLD", etc.
    score: i32,
    wcag_level: String,   // "A", "AA", "AAA"
}

impl Component for ComplianceBadge {
    fn render(&self, context: &RenderContext) -> String {
        format!(
            r#"
            #box(
              width: 100%,
              height: 80pt,
              fill: {},
              radius: 8pt,
              inset: 16pt,
              [
                #align(center)[
                  #text(size: 24pt, weight: "bold")[{}]
                  #v(4pt)
                  #text(size: 14pt)[Score: {}/100]
                  #v(2pt)
                  #text(size: 12pt)[WCAG 2.1 Level {}]
                ]
              ]
            )
            "#,
            self.get_badge_color(),
            self.level,
            self.score,
            self.wcag_level
        )
    }
}
```

#### ViolationSummary Component
```rust
pub struct ViolationSummary {
    total_errors: usize,
    total_warnings: usize,
    total_notices: usize,
    pages_tested: usize,
    passed_pages: usize,
}

impl Component for ViolationSummary {
    fn render(&self, context: &RenderContext) -> String {
        // Typst grid layout with summary stats
        format!(
            r#"
            #grid(
              columns: (1fr, 1fr, 1fr),
              gutter: 16pt,
              [
                #box(fill: red.lighten(80%), inset: 12pt, radius: 4pt)[
                  #text(size: 32pt, fill: red)[{}]
                  #v(4pt)
                  #text(size: 10pt)[Errors]
                ]
              ],
              [
                #box(fill: orange.lighten(80%), inset: 12pt, radius: 4pt)[
                  #text(size: 32pt, fill: orange)[{}]
                  #v(4pt)
                  #text(size: 10pt)[Warnings]
                ]
              ],
              [
                #box(fill: green.lighten(80%), inset: 12pt, radius: 4pt)[
                  #text(size: 32pt, fill: green)[{}]
                  #v(4pt)
                  #text(size: 10pt)[Passed Pages]
                ]
              ]
            )
            "#,
            self.total_errors,
            self.total_warnings,
            self.passed_pages
        )
    }
}
```

### 10.4 Typst Template Example

```typst
// src/output/typst/templates/wcag_audit.typ
#import "@preview/renderreport:0.1.0": *

#set page(
  paper: "a4",
  margin: (x: 2cm, y: 2.5cm),
  header: [
    #image("wcag_logo.svg", width: 30pt)
    #h(1fr)
    #text(size: 10pt, fill: gray)[WCAG 2.1 Accessibility Audit]
  ],
  footer: [
    #text(size: 9pt, fill: gray)[
      Generated by auditmysit on #datetime.today().display()
    ]
    #h(1fr)
    #counter(page).display("1 of 1", both: true)
  ]
)

#set text(font: "Inter", size: 11pt)
#set heading(numbering: "1.1")

// Title
#align(center)[
  #text(size: 24pt, weight: "bold")[
    WCAG 2.1 Accessibility Audit Report
  ]
  #v(8pt)
  #text(size: 14pt, fill: gray)[
    {{url}}
  ]
  #v(16pt)
]

// Executive Summary
#heading(level: 1)[Executive Summary]
{{compliance_badge}}
#v(12pt)
{{violation_summary}}

// Detailed Findings
#pagebreak()
#heading(level: 1)[Detailed Findings]
{{findings_list}}

// Violations Table
#pagebreak()
#heading(level: 1)[All Violations]
{{violations_table}}

// Appendix
#pagebreak()
#heading(level: 1)[Appendix: WCAG 2.1 Guidelines]
{{wcag_reference}}
```

### 10.5 Integration Timeline

| Week | Task | Deliverable |
|------|------|-------------|
| Week 1 | Design component mapping | Component spec document |
| Week 2 | Implement WCAGPack | Basic PDF generation working |
| Week 3 | Create custom components (ComplianceBadge, ViolationSummary) | Professional-looking PDF |
| Week 4 | Design Typst template | Final template with branding |
| Week 5 | Integration testing | Full audit → PDF workflow |

---

## 11. Conclusion

### Summary of Key Findings

1. **WCAG Rules:** Rust CLI currently implements 3/15 target rules. Priority: contrast checking (1.4.3), language attributes (3.1.1), and ARIA landmarks (2.4.1).

2. **Batch Processing:** TypeScript has comprehensive sitemap parsing + queue system. Rust needs this for multi-page audits (HIGH priority).

3. **Report Generation:** Transition from HTML to **Typst PDF** via renderreport is strategic decision. Professional compliance documentation > web-based reports.

4. **Exclusions:** Successfully identified 3,716 lines of Flutter UI code and complete REST API server to exclude. Clean separation of concerns.

5. **Timeline:** 8-week roadmap to achieve 80% feature parity with TypeScript CLI while delivering superior PDF output.

### Strategic Decisions

| Decision | Rationale |
|----------|-----------|
| **Skip HTML reports** | Typst PDF provides better compliance documentation |
| **Skip performance/SEO analyzers** | Focus on WCAG accessibility only |
| **Prioritize batch processing** | Essential for real-world usage (multi-page sites) |
| **Implement scoring system** | Needed for ScoreCard component in PDF |
| **Defer glassmorphism detection** | Edge case, low ROI for MVP |

### Next Steps

1. ✅ **Complete this documentation** (DONE)
2. ⏳ **Implement scoring system** (1 day - enables PDF reports)
3. ⏳ **Add contrast checking** (2-3 days - closes major WCAG gap)
4. ⏳ **Integrate renderreport** (1 week - when crate published)
5. ⏳ **Sitemap parsing** (1 week - enables batch processing)

---

**Document Status:** Complete  
**Last Updated:** 2026-01-30  
**Authors:** AI Assistant (Claude Code)  
**Review Status:** Pending user approval
