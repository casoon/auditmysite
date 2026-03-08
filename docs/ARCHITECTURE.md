# auditmysite Architecture

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
│   ├── args.rs          # Clap argument parsing
│   └── config.rs        # auditmysite.toml config support
│
├── browser/             # Chrome/Chromium management
│   ├── mod.rs
│   ├── detection.rs     # Find Chrome installation
│   ├── installer.rs     # Download browser via `browser install`
│   ├── resolver.rs      # Browser path resolution
│   ├── manager.rs       # Browser lifecycle (launch, navigate, close)
│   └── pool.rs          # Page pool for concurrent audits
│
├── accessibility/       # Accessibility Tree handling
│   ├── mod.rs
│   ├── extractor.rs     # CDP AXTree extraction
│   ├── tree.rs          # AXNode, AXTree structures
│   └── styles.rs        # Computed style extraction
│
├── wcag/                # WCAG rule engine
│   ├── mod.rs
│   ├── engine.rs        # Rule orchestration
│   ├── types.rs         # Violation, Severity, WcagResults
│   └── rules/           # Individual WCAG rules (22 rules)
│       ├── text_alternatives.rs  # 1.1.1
│       ├── contrast.rs           # 1.4.3
│       ├── keyboard.rs           # 2.1.1
│       ├── bypass_blocks.rs      # 2.4.1
│       ├── link_purpose.rs       # 2.4.4
│       ├── headings.rs           # 2.4.6
│       ├── labels.rs             # 3.3.2
│       └── ...
│
├── taxonomy/            # Rule taxonomy & classification
│   ├── mod.rs
│   ├── rules.rs         # Rule definitions with metadata
│   └── lookup.rs        # RuleLookup for WCAG ID → taxonomy mapping
│
├── audit/               # Audit orchestration
│   ├── mod.rs
│   ├── pipeline.rs      # Single page audit flow
│   ├── batch.rs         # Sitemap/batch processing
│   ├── report.rs        # AuditReport structure (raw data)
│   ├── normalized.rs    # NormalizedReport (enriched, score-corrected)
│   └── scoring.rs       # Score calculation
│
├── output/              # Report generation
│   ├── mod.rs
│   ├── cli.rs           # Terminal table output
│   ├── json.rs          # JSON reports (via NormalizedReport)
│   ├── pdf.rs           # PDF reports (via renderreport/Typst)
│   ├── report_model.rs  # ViewModel structs for PDF
│   └── report_builder.rs # AuditReport → ViewModel transformation
│
├── security/            # Security analysis
│   └── mod.rs           # Headers, SSL, SSRF protection
│
├── seo/                 # SEO analysis
│   ├── mod.rs
│   ├── meta.rs          # Meta tags
│   ├── headings.rs      # Heading structure
│   ├── profile.rs       # SEO content profile
│   └── ...
│
├── performance/         # Performance metrics
│   ├── mod.rs
│   ├── vitals.rs        # Core Web Vitals
│   └── ...
│
└── mobile/              # Mobile friendliness
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
7. Calculate raw score → AuditReport
   │
8. Normalize: AuditReport → NormalizedReport
   │  ├── Apply score corrections (e.g. 3.1.1 lang suppression)
   │  ├── Enrich findings with taxonomy fields
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
├── url_validation_tests.rs    # SSRF protection
├── output_format_tests.rs     # Report generation
└── error_handling_tests.rs    # Error paths

src/*/tests (inline)           # Unit tests per module
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
- `serde` - Serialization
- `tracing` - Logging
- `renderreport` - PDF generation (Typst-based)
