# auditmysite Architecture

This document describes the current implementation. For older browser-design explorations and proposals that are not fully implemented, see [browser-architecture.md](browser-architecture.md).

## Overview

auditmysite is a WCAG 2.1 accessibility checker written in Rust. It uses Chrome DevTools Protocol (CDP) to extract the browser's native Accessibility Tree and analyze it for violations.

```
┌─────────────────────────────────────────────────────────────────┐
│                         CLI (main.rs)                           │
├─────────────────────────────────────────────────────────────────┤
│                      Audit Pipeline                             │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐        │
│  │ Browser  │→ │ AXTree   │→ │  WCAG    │→ │  Report  │        │
│  │ Manager  │  │ Extract  │  │  Engine  │  │  Output  │        │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘        │
└─────────────────────────────────────────────────────────────────┘
```

## Module Structure

```
src/
├── main.rs              # CLI entry point
├── lib.rs               # Library exports
├── error.rs             # Error types (AuditError)
├── util.rs              # Utility functions
│
├── cli/                 # Command-line interface
│   ├── args.rs          # Clap argument parsing (Args, WcagLevel, OutputFormat, AnnexKind)
│   ├── config.rs        # auditmysite.toml config support
│   ├── commands.rs      # Subcommand handlers (browser, doctor, plan)
│   ├── runners.rs       # Mode runners (single, batch, compare)
│   ├── report_writers.rs # Output dispatch (single/batch/comparison)
│   ├── output_paths.rs  # File path generation for reports
│   ├── plan.rs          # Pre-audit plan/banner printing
│   ├── doctor.rs        # `auditmysite doctor` diagnostics
│   └── sitemap_suggest.rs # Sitemap discovery + interactive prompt
│
├── browser/             # Chrome/Chromium management
│   ├── mod.rs
│   ├── detection.rs     # Find Chrome installation
│   ├── installer.rs     # Download browser via `browser install`
│   ├── resolver.rs      # Browser path resolution
│   ├── manager.rs       # Browser lifecycle (launch, navigate, close)
│   ├── pool.rs          # Page pool for concurrent audits
│   ├── consent.rs       # CMP cookie injection + consent-banner dismissal
│   ├── throttle.rs      # CPU/network throttling profiles
│   ├── registry.rs      # Known-browser registry
│   └── mock.rs          # Test-only mock browser
│
├── accessibility/       # Accessibility Tree handling
│   ├── mod.rs
│   ├── extractor.rs     # CDP AXTree extraction
│   ├── tree.rs          # AXNode, AXTree structures
│   ├── styles.rs        # Computed style extraction
│   ├── enrichment.rs    # AXNode enrichment (roles, computed properties)
│   ├── code_gen.rs      # Selector/snippet generation for findings
│   ├── element_capture.rs # CDP screenshot capture + element highlighting for evidence
│   ├── snapshot.rs      # Cached snapshot (de)serialization
│   └── diff.rs          # Snapshot diffing for regression comparisons
│
├── wcag/                # WCAG rule engine
│   ├── mod.rs
│   ├── engine.rs        # Rule orchestration
│   ├── types.rs         # Violation, Severity, WcagResults
│   ├── coverage.rs      # WCAG success-criterion coverage accounting
│   ├── en301549.rs      # EN 301 549 (chapter 9, Web) clause mapping + annex derivation
│   └── rules/           # Individual WCAG rules (85+ files, Level A/AA/AAA)
│       ├── text_alternatives.rs  # 1.1.1
│       ├── contrast.rs           # 1.4.3
│       ├── keyboard.rs           # 2.1.1
│       ├── bypass_blocks.rs      # 2.4.1
│       ├── link_purpose.rs       # 2.4.4
│       ├── headings.rs           # 2.4.6
│       ├── language_of_parts.rs  # Conservative language-change check for 3.1.2
│       ├── labels.rs             # 3.3.2
│       ├── target_size_minimum.rs # 2.5.8
│       ├── text_spacing.rs       # 1.4.12
│       └── ... (registered in rules/mod.rs)
│
├── taxonomy/            # Rule taxonomy & classification
│   ├── mod.rs
│   ├── rules.rs         # Canonical rule list (RULES) with metadata
│   ├── criteria.rs      # WCAG success-criterion definitions
│   ├── dimensions.rs    # Dimension enum (Barrierefreiheit, Usability, ...)
│   ├── issue_class.rs   # IssueClass enum (Fehlend, Falsch, Unvollständig)
│   ├── score.rs         # Taxonomy-based score impact helpers
│   └── severity.rs      # Severity mapping helpers
│
├── audit/               # Audit orchestration
│   ├── mod.rs
│   ├── pipeline.rs      # Single page audit flow
│   ├── batch.rs         # Sitemap/batch processing
│   ├── module.rs        # AuditModule trait + AuditCatalog registry (topo-sorted)
│   ├── catalog.rs       # Module registration/wiring
│   ├── report.rs        # AuditReport structure (raw data)
│   ├── normalized.rs    # NormalizedReport (enriched, score-corrected)
│   ├── scoring.rs       # Score calculation
│   ├── interpretation.rs # Pre-computed DE/EN interpretation texts
│   ├── summary.rs       # Cross-page aggregation logic
│   ├── template_dedup.rs # Template-level root-cause deduplication (batch)
│   ├── occurrence_analysis.rs # Occurrence counting across categories
│   ├── prioritization.rs # Fix-guidance prioritization
│   ├── verdict.rs       # Pass/fail verdict derivation
│   ├── baseline.rs      # Baseline/compare-mode support
│   ├── batch_consistency.rs # Cross-page consistency checks
│   ├── budget.rs        # Concurrency/rate budgeting
│   ├── crawl.rs         # Link crawling (html5ever-based)
│   ├── duplicate.rs     # Duplicate-page detection
│   ├── artifacts.rs     # Cache artifact persistence (--reuse-cache)
│   └── performance_interpretation.rs # Performance-specific interpretation texts
│
├── output/              # Report generation
│   ├── mod.rs
│   ├── cli.rs           # Terminal table output
│   ├── json.rs          # JSON reports (via NormalizedReport)
│   ├── json/            # JSON detail/helper builders
│   ├── sr_audit_json.rs # Standalone screen-reader audit JSON sidecar
│   ├── sarif.rs         # SARIF output format
│   ├── builder/         # AuditReport → PDF ViewModel transformation
│   │   ├── mod.rs, actions.rs, modules.rs, helpers.rs, batch.rs
│   │   └── single/      # Single-report builder (findings, etc.)
│   ├── pdf/             # PDF reports (via renderreport/Typst)
│   │   ├── mod.rs, single_report.rs, batch_report.rs, batch_report/
│   │   ├── cover.rs, findings.rs, wcag_coverage.rs, en301549.rs
│   │   ├── diagnosis.rs, appendix.rs, design.rs (4-color law)
│   │   └── detail_modules/ # Per-module chapter renderers
│   ├── report_model.rs  # ViewModel structs for PDF
│   ├── explanations.rs  # interpret_score() and shared explanation text
│   ├── search_experience.rs # Search/AI-visibility presentation helpers
│   ├── localized.rs     # Locale-aware text helpers
│   ├── renderer.rs      # Top-level format dispatch
│   └── snapshot_export.rs # AXTree + journey traces → YAML (--export-snapshot)
│
├── a11y_journey/        # Accessibility Journey Layer (--interactive)
│   ├── mod.rs           # Orchestrator: run() single entry point, commerce gating
│   ├── tab_walk.rs      # Tab-order recording + focus evaluation
│   ├── skip_link.rs     # Skip-link activation journey
│   ├── disclosure_journey.rs # Accordion/disclosure toggle
│   ├── modal_journey.rs # Modal focus-trap verification
│   ├── tabs_journey.rs  # TabList navigation
│   ├── menu_journey.rs  # DisclosureMenu journey
│   ├── form_error.rs    # Form-error announcement (per-form clustering, multi-form pages)
│   ├── spa_navigation.rs # SPA single-page navigation detection
│   ├── link_inventory.rs # Linktext/heading/landmark inventory (pure AXTree)
│   ├── add_to_cart.rs   # Add-to-cart status/feedback journey (commerce-gated, PDP only)
│   ├── quantity_stepper.rs # Quantity-stepper keyboard operability (commerce-gated, PDP only)
│   └── evaluate.rs      # Tab-walk finding evaluation
│
├── screen_reader/       # Screen-reader reading-order primitives
│   ├── mod.rs
│   ├── types.rs         # ReadingItem and ignored-node diagnostics
│   ├── announcer.rs     # Localized screen-reader announcement strings
│   ├── navigator.rs     # Virtual SR navigation lists
│   ├── analyzer.rs      # SR-specific issue detection
│   ├── bfsg.rs          # Thin wrapper over wcag::en301549 for legacy call sites
│   └── linearizer.rs    # AXTree DFS reading-order linearization
│
├── best_practices/      # Console errors and vulnerable JS library detection
│   ├── mod.rs, module.rs
│   ├── console_errors.rs # CDP-based console error/warning collection
│   └── vulnerable_libs.rs # Known-CVE JS library detection (jQuery, Bootstrap, ...)
│
├── performance/         # Core Web Vitals, render-blocking, content weight
│   ├── mod.rs, vitals.rs
│   ├── animations.rs    # Non-composited animation detection
│   ├── coverage.rs      # Unused JS/CSS detection via CDP Coverage API
│   ├── critical_chain.rs # Critical request chain analysis
│   ├── minification.rs  # Unminified JS/CSS asset detection
│   └── third_party.rs   # Third-party resource attribution per origin
│
├── seo/                 # Meta, headings, schema, social, technical SEO
│   ├── mod.rs, module.rs
│   ├── meta.rs          # Meta tags
│   ├── headings.rs      # Heading structure
│   ├── profile.rs       # SEO content profile
│   ├── schema.rs        # Structured data (JSON-LD/schema.org)
│   ├── schema_rules.rs  # Feature-specific required/recommended property rules
│   ├── schema_fit.rs    # Page-intent and URL based primary-schema fit
│   ├── schema_parity.rs # Conservative visible-content vs. JSON-LD comparison
│   ├── social.rs        # Open Graph / Twitter Card
│   ├── technical.rs     # robots/canonical/hreflang
│   ├── page_health.rs   # Aggregated issue collection (collect_issues)
│   ├── image_efficiency.rs # Image format and resolution analysis
│   └── ...
│
├── security/            # Security header analysis
│   └── mod.rs, module.rs
│
├── mobile/              # Mobile friendliness analysis
│   ├── mod.rs, module.rs
│   └── ux_heuristics.rs
│
├── dark_mode/           # Dark mode support detection and contrast
│   └── mod.rs, module.rs
│
├── ux/                  # UX analysis (5 dimensions, saturation curves)
│   ├── mod.rs, module.rs
│   ├── analysis.rs
│   └── scoring.rs
│
├── journey/             # User journey analysis, page intent detection
│   ├── mod.rs, module.rs
│   ├── analysis.rs
│   ├── page_intent.rs
│   └── scoring.rs
│
├── ai_visibility/       # AI/LLM discoverability analysis
│   ├── mod.rs, module.rs
│   ├── chunks.rs, citation.rs, knowledge_graph.rs, readability.rs
│
├── content_visibility/  # Cross-module signal aggregation (SEO+AI+Quality)
│   └── mod.rs, module.rs
│
├── commerce/            # Shop audit (derive-only, shop-gated): product schema-completeness,
│   └── mod.rs, module.rs # mandatory/trust-page links, page-kind (PDP/Category only), batch roll-up
│
├── source_quality/      # Source quality signals (headers, schema, HTTPS)
│   └── mod.rs, module.rs
│
├── tech_stack/          # CMS/framework detection from in-page signals
│   └── mod.rs, module.rs
│
├── patterns/            # UI pattern detection (nav, accordion, modal, ...)
│   ├── mod.rs           # PatternKind/JourneyKind registry, analyze()
│   ├── main_navigation.rs, accordion.rs, modal_dialog.rs
│   ├── disclosure_menu.rs, tab_list.rs, skip_link.rs
│   ├── form.rs          # Per-form clustering (ancestor_chain_from_root)
│   ├── add_to_cart.rs   # Add-to-cart button/feedback detection
│   └── quantity_stepper.rs # Quantity-stepper control detection
│
├── interaction/         # Cross-cutting interaction analysis
│   └── mod.rs, focus.rs, keyboard.rs, pointer.rs, stability.rs # Bounded DOM/app-ready settling
│
├── assessment/          # Shared assessment types and evidence model
│   └── mod.rs
│
├── studio/              # Studio contract types (GUI data contract)
│   └── mod.rs
│
└── i18n/                # Project Fluent (.ftl) loader, default language: German
    └── mod.rs
```

## Data Flow

### Single URL Audit

```
1. CLI parses arguments
   │
2. validate_url() checks for SSRF
   │
3. BrowserManager launches Chrome
   │
4. Navigate to URL, wait for load
   │
5. Extract AXTree via CDP
   │  └── GetFullAXTreeParams
   │
6. Run WCAG rules against AXTree
   │  ├── Level A rules
   │  ├── Level AA rules (if requested)
   │  └── Level AAA rules (if requested)
   │
6b. Run optional modules (`--full`, or individually enabled): performance, seo,
    security, mobile, dark-mode, ux, journey, ai-visibility, content-visibility,
    source-quality, tech-stack, best-practices, commerce (shop-gated)
   │
6c. Run Accessibility Journey Layer (`--interactive off|basic|full`): tab-walk,
    skip-link, disclosure/modal/tablist/menu journeys, form-error announcement,
    SPA navigation, commerce-gated add-to-cart/quantity-stepper journeys
   │
7. Calculate raw score → AuditReport
   │  └── Preserve requested scope, module/subcheck runs, rule outcomes,
   │      navigation, consent state, environment and audit quality
   │
8. Normalize: AuditReport → NormalizedReport
   │  ├── Apply score corrections (e.g. 3.1.1 lang suppression)
   │  ├── Enrich findings with taxonomy fields
   │  ├── Normalize warnings/manual-review items separately from violations
   │  ├── Retain Journey execution coverage and compact focus evidence
   │  └── Compute grade + certificate from corrected score
   │
9. Generate report (JSON/PDF/Table)
   │  └── All formats use NormalizedReport as single source of truth
   │
10. Close browser
```

### Batch Audit (Sitemap)

```
1. Fetch and parse sitemap.xml
   │
2. Create BrowserPool with N pages
   │
3. For each URL (concurrent):
   │  ├── Acquire page from pool
   │  ├── Run single audit
   │  ├── Return page to pool
   │  └── Report progress
   │
4. Aggregate results
   │
5. Generate batch report
```

## Key Components

### NormalizedReport

Central data layer between raw `AuditReport` and output formats:
- Applies score corrections (e.g. 3.1.1 language suppression)
- Enriches violations with taxonomy metadata (dimension, subcategory, issue_class, etc.)
- Carries explicit audit quality, module/rule execution status, accessibility assessments, and Journey coverage
- Computes grade and certificate from corrected score
- Single source of truth for JSON, PDF, and CLI output — ensures consistent scores

### BrowserManager

Handles Chrome lifecycle:
- Auto-detects Chrome/Chromium installation
- Explicit download via `auditmysite browser install`
- Configures headless mode with security flags
- Manages page navigation with timeouts

### BrowserPool

For concurrent audits:
- Pre-creates N pages
- Reuses pages between audits (reset to about:blank)
- Handles page failures gracefully
- Timeout protection on page reset

### AXTree Extractor

Uses CDP `Accessibility.getFullAXTree`:
- Converts CDP response to internal AXNode structure
- Preserves parent/child relationships
- Extracts properties (role, name, focusable, etc.)

### WCAG Engine

Orchestrates rule checking:
- Filters rules by WCAG level (A, AA, AAA)
- Runs each rule against relevant nodes
- Collects violations with severity
- Counts passes for scoring

### Rule Structure

Each rule implements checking logic:

```rust
pub fn check(tree: &AXTree, _styles: &[NodeStyle], level: WcagLevel) -> Vec<Violation> {
    let mut violations = Vec::new();

    for node in tree.iter() {
        if should_check(node) && has_violation(node) {
            violations.push(Violation::new(
                "1.1.1",
                "Non-text Content",
                level,
                Severity::High,
                "Image missing alt text",
                &node.node_id,
            ));
        }
    }

    violations
}
```

### Taxonomy

Maps WCAG rule IDs to rich metadata:
- **Dimension**: Barrierefreiheit, Usability, etc.
- **Subcategory**: Inhalte & Alternativen, Navigation, etc.
- **IssueClass**: Fehlend, Falsch, Unvollständig
- **ScoreImpact**: Base penalty, max penalty, scaling curve
- **ReportVisibility**: Which report levels (Executive/Standard/Technical) include each finding

## Security Considerations

### SSRF Protection

`validate_url()` blocks:
- Private IPs (10.x, 172.16-31.x, 192.168.x)
- Localhost (127.x, ::1)
- Link-local (169.254.x)
- Non-HTTP schemes

### Path Traversal

`read_url_file()` uses `canonicalize()` to prevent `../` attacks.

### Chromium Downloads

- Pinned version constant
- Trusted CDN (storage.googleapis.com)
- HTTPS only

## Testing Strategy

```
tests/
├── output_format_tests.rs        # Report generation
├── error_handling_tests.rs       # Error paths
├── parity_contract.rs            # Frozen-count guard vs. docs/PARITY_CONTRACT.jsonc
├── wcag_coverage.rs               # Level A/AA/AAA rule-set coverage checks
├── wcag_unit_tests.rs             # Per-rule unit tests (fixtures in wcag_fixtures/)
├── integration_test.rs / integration_mocked.rs # End-to-end pipeline tests
├── report_consistency_tests.rs   # Score/finding consistency across formats
├── batch_consistency (see src/audit/batch_consistency.rs)
├── release_contract_tests.rs     # Release-gate invariants
└── snapshot_tests.rs              # insta snapshot tests (snapshots/)

src/*/tests (inline)               # Unit tests per module
```

Run all tests:
```bash
cargo test
```

## Performance

- Single audit: ~2-5 seconds (depends on page complexity)
- Batch audit: Linear scaling with concurrency
- Memory: ~100MB per Chrome page

## Dependencies

Key crates:
- `chromiumoxide` - CDP client
- `tokio` - Async runtime
- `clap` - CLI parsing
- `serde` / `serde_json` - Serialization
- `tracing` - Logging
- `html5ever` / `markup5ever_rcdom` - Local HTML5 parsing (link crawling, no remote validator)
- `fluent-bundle` / `unic-langid` - i18n (Project Fluent, default language German)
- `reqwest` (rustls) - Sitemap fetching, Chromium download
- `renderreport` (optional, `pdf` feature) - PDF generation (Typst-based)
