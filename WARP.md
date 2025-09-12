# WARP.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

## Project Overview

AuditMySite is a comprehensive website analysis suite that provides professional accessibility testing, Core Web Vitals performance monitoring, SEO analysis, and content optimization insights. The project is built in TypeScript and offers three main interfaces: CLI tool, REST API server, and JavaScript SDK.

## Essential Commands

### Development and Building
```bash
# Build the project (with type checking)
npm run build

# Build and validate (includes build validation tests)
npm run build:validate

# Development mode with watch
npm run dev

# Type checking only
npm run type-check
```

### Testing
```bash
# Run all tests (includes type checking)
npm test

# Watch mode for development
npm run test:watch

# Coverage reports
npm run test:coverage

# Specific test categories
npm run test:unit          # Unit tests only
npm run test:integration   # Integration tests
npm run test:api          # API endpoint tests
npm run test:cli          # CLI command tests
npm run test:e2e          # End-to-end tests

# CI/CD optimized
npm run test:ci           # Non-interactive with coverage

# Verbose test output
npm run test:verbose      # Detailed test logs
```

### CLI Usage Examples
```bash
# Basic audit (tests 5 pages by default)
auditmysite https://example.com/sitemap.xml

# Test specific number of pages
auditmysite https://example.com/sitemap.xml --max-pages 10

# Expert mode with interactive configuration
auditmysite https://example.com/sitemap.xml --expert

# API server mode
auditmysite --api --port 3000 --api-key your-secret-key

# Performance budget templates
auditmysite https://example.com/sitemap.xml --budget ecommerce
auditmysite https://example.com/sitemap.xml --budget corporate

# CI/CD friendly mode
auditmysite https://example.com/sitemap.xml --non-interactive --format markdown
```

### Data Analysis & Validation
```bash
# Analyze data structure (for debugging report generation)
npm run analyze-data

# Consolidate data structure
npm run consolidate-data

# Validate report data structures
npm run validate-reports
```

## Architecture Overview

### Core Components

**Main Entry Points:**
- `src/index.ts` - Legacy accessibility checker entry point
- `bin/audit.js` - Main CLI entry point with full feature set
- `src/accessibility-checker-main.ts` - Main accessibility analysis engine

**Core Analysis Pipeline:**
- `MainAccessibilityChecker` - Orchestrates all analysis types
- `AccessibilityChecker` - Core accessibility testing with pa11y integration
- `ContentWeightAnalyzer` - Resource analysis and content optimization
- `EnhancedPerformanceCollector` - Core Web Vitals and performance metrics
- `EnhancedSEOAnalyzer` - SEO analysis with meta tags and content quality
- `MobileFriendlinessAnalyzer` - Mobile responsiveness and touch target analysis

**Shared Infrastructure:**
- `src/core/` - Core components (pipeline, browser management, queue systems)
- `src/types/` - TypeScript type definitions and interfaces
- `src/generators/` - Report generation (HTML, Markdown, JSON, CSV)
- `src/parsers/` - Sitemap parsing and URL discovery

### Key Architectural Patterns

1. **Analyzer Pattern**: Each analysis type (accessibility, performance, SEO, etc.) implements a consistent analyzer interface
2. **Pipeline Architecture**: `StandardPipeline` coordinates the entire analysis workflow
3. **Browser Management**: Playwright-based browser automation with resource monitoring
4. **Queue Systems**: Both legacy and unified queue systems for managing concurrent analysis
5. **Report Generation**: Multi-format report generation with consistent data structures

### Analysis Flow

1. **Sitemap Discovery**: Automatic sitemap detection or parsing from provided URL
2. **URL Filtering**: Apply include/exclude patterns and page limits
3. **Browser Initialization**: Launch Chromium with appropriate configuration
4. **Concurrent Analysis**: Run multiple analysis types per page (accessibility, performance, SEO, content weight, mobile-friendliness)
5. **Quality Scoring**: Calculate composite quality scores with letter grades
6. **Report Generation**: Generate HTML/Markdown reports with detailed findings and recommendations

## Technology Stack

### Core Dependencies
- **Playwright**: Browser automation and testing
- **pa11y v9**: Accessibility testing with axe-core v4.10
- **TypeScript**: Type-safe development
- **Commander**: CLI argument parsing
- **Express**: REST API server
- **Jest**: Testing framework with comprehensive coverage

### Analysis Libraries
- **Google Web Vitals**: Official Core Web Vitals collection
- **Fast XML Parser**: Sitemap parsing
- **Cheerio**: HTML parsing for content analysis
- **Chalk**: Terminal styling and colors

## Enhanced Analysis Features

The project includes "Enhanced Analysis" which is the current standard approach (enabled by default):

- **Robust Accessibility Testing**: ARIA validation, focus management, color contrast
- **Core Web Vitals**: Performance monitoring with retry mechanisms and isolated browser contexts
- **SEO Analysis**: Meta tags, heading structure, content optimization
- **Content Weight Assessment**: Resource analysis with optimization recommendations
- **Mobile-Friendliness**: Responsive design and touch target validation

## Performance Budgets

The system includes business-focused performance budget templates:

- `ecommerce`: Strict thresholds for conversion optimization
- `corporate`: Professional standards for business sites
- `blog`: Content-focused thresholds
- `default`: Google Web Vitals standard thresholds
- `custom`: User-defined individual metric thresholds

## API Endpoints

When running in API mode (`--api` flag), the following endpoints are available:

- `POST /api/v1/audit/quick` - Quick audit with enhanced analysis (default)
- `POST /api/v1/audit/performance` - Performance-focused analysis
- `POST /api/v1/audit/seo` - SEO-focused analysis
- `POST /api/v1/audit/content-weight` - Content weight analysis
- `POST /api/v1/audit/accessibility` - Accessibility-focused analysis
- `GET /api/v1/audit/{jobId}` - Get audit job status
- `GET /health` - Server health check

## Development Guidelines

### Code Organization
- Use TypeScript interfaces from `src/types.ts` for type safety
- Follow the analyzer pattern for new analysis types
- Maintain backward compatibility in CLI options
- Use the existing error categorization system in `bin/audit.js`

### Testing Strategy
- Unit tests for individual components in `tests/unit/`
- Integration tests for component interactions in `tests/integration/`
- API tests for HTTP endpoints in `tests/api/`
- CLI tests for command validation in `tests/cli/`
- E2E tests for critical user journeys in `tests/e2e/`

### Report Generation
- All reports use consistent data structures defined in `types.ts`
- HTML reports include interactive elements and modern styling
- Markdown reports are optimized for developer workflows and CI/CD
- Support for accessibility compliance levels (Basic, Enhanced, Comprehensive)

### Browser Management
- Use Playwright's Chromium for consistent results
- Implement proper cleanup to prevent memory leaks
- Use isolated browser contexts for measurement stability
- Include retry mechanisms for reliability

## Common Development Tasks

### Adding New Analysis Types
1. Create analyzer class implementing the standard interface
2. Add type definitions in `src/types/enhanced-metrics.ts`
3. Integrate into `MainAccessibilityChecker`
4. Update report generators to handle new data structure
5. Add corresponding tests

### Modifying CLI Options
1. Update `bin/audit.js` options parsing
2. Add validation and help text
3. Update expert mode prompts if needed
4. Add tests in `tests/cli/`
5. Update README.md documentation

### Adding New Report Formats
1. Create generator in `src/generators/`
2. Update `OutputGenerator` to handle new format
3. Add format validation in CLI
4. Test with various data structures

## Error Handling

The project includes sophisticated error categorization:
- **Network Errors**: Connection and timeout issues
- **Sitemap Errors**: XML parsing and format issues
- **Browser Errors**: Playwright and automation failures
- **Resource Errors**: Memory and system resource issues
- **Permission Errors**: File system and access issues

Each error type includes recovery strategies and user-friendly suggestions.

## System Requirements

- **Node.js**: 18+ (required for pa11y v9)
- **Memory**: 3GB minimum, 4GB recommended for enhanced analysis
- **Chrome/Chromium**: 120+ (Chrome 135+ recommended for full optimization)
- **Network**: Required for testing external sites

## Exit Codes

- `0`: Successful completion (accessibility failures are normal and don't cause non-zero exit)
- `1`: Technical errors (browser crashes, network failures, parsing errors)

This ensures CI/CD pipelines can distinguish between technical failures and expected accessibility issues.
