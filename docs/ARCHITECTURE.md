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
│
├── cli/                 # Command-line interface
│   └── args.rs          # Clap argument parsing
│
├── browser/             # Chrome/Chromium management
│   ├── mod.rs
│   ├── detection.rs     # Find Chrome installation
│   ├── installer.rs     # Download Chromium if needed
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
│   └── rules/           # Individual WCAG rules
│       ├── text_alternatives.rs  # 1.1.1
│       ├── contrast.rs           # 1.4.3
│       ├── keyboard.rs           # 2.1.1
│       ├── bypass_blocks.rs      # 2.4.1
│       ├── link_purpose.rs       # 2.4.4
│       ├── headings.rs           # 2.4.6
│       ├── labels.rs             # 3.3.2
│       └── ...
│
├── audit/               # Audit orchestration
│   ├── mod.rs
│   ├── pipeline.rs      # Single page audit flow
│   ├── batch.rs         # Sitemap/batch processing
│   ├── report.rs        # AuditReport structure
│   └── scoring.rs       # Score calculation
│
├── output/              # Report generation
│   ├── mod.rs
│   ├── cli.rs           # Terminal table output
│   ├── json.rs          # JSON reports
│   ├── html.rs          # HTML reports
│   └── pdf.rs           # PDF reports (via renderreport)
│
├── security/            # Security analysis
│   └── mod.rs           # Headers, SSL, SSRF protection
│
├── seo/                 # SEO analysis
│   ├── mod.rs
│   ├── meta.rs          # Meta tags
│   ├── headings.rs      # Heading structure
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
7. Calculate score
   │
8. Generate report (JSON/HTML/PDF/Table)
   │
9. Close browser
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

### BrowserManager

Handles Chrome lifecycle:
- Auto-detects Chrome/Chromium installation
- Downloads Chromium if not found
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
                Severity::Serious,
                "Image missing alt text",
                &node.node_id,
            ));
        }
    }
    
    violations
}
```

## Security Considerations

### SSRF Protection

`validate_url()` blocks:
- Private IPs (10.x, 172.16-31.x, 192.168.x)
- Localhost (127.x, ::1)
- Link-local (169.254.x)
- Non-HTTP schemes

### Path Traversal

`read_url_file()` uses `canonicalize()` to prevent `../` attacks.

### XSS in Reports

`html_escape()` sanitizes all user-controlled content in HTML reports.

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
- `renderreport` - PDF generation
