# 🎯 AuditMySite - Enhanced Web Analysis Suite v1.9.2

> **🐛 BUGFIX v1.9.2**: Mobile-Friendliness Analysis Fixes! **Resolved "undefined" recommendations display, enhanced mobile analysis reporting, and improved data structure handling!** 📱
> **🐛 BUGFIX v1.9.1**: Critical fixes for Enhanced Analysis Suite! **100% stable Enhanced Analysis workflow, fixed data structure handling, and reliable report generation!** 🔧
> **🚀 v1.9.0**: Revolutionary Enhanced Analysis Suite! **Robust accessibility testing, Core Web Vitals monitoring, SEO analysis, and content optimization insights!** 🔥
> **🔧 PERFORMANCE**: Isolated browser contexts, retry mechanisms, and 100% stable measurements! **Enterprise-grade reliability!**
> **🌐 API**: Comprehensive endpoints with specialized analysis modes! **Professional integration ready!**
> **🧪 TESTING**: 25+ test cases with edge case coverage! **Production-validated quality!**

A comprehensive **three-in-one solution** for professional web auditing: **CLI tool**, **REST API server**, and **JavaScript SDK**. Features revolutionary enhanced analysis with isolated browser contexts, retry mechanisms, SEO optimization, content weight assessment, and comprehensive API endpoints for enterprise-grade web auditing.

⚡ **v1.9.2 Highlights**: Mobile-Friendliness Analysis perfected! Fixed "undefined" recommendations, enhanced mobile score reporting, and improved mobile analysis data handling!
⚡ **v1.9.1 Highlights**: All Enhanced Analysis issues resolved! Fully stable Enhanced Analysis pipeline with correct data handling and report generation!
⚡ **v1.9.0 Highlights**: Enhanced Analysis as standard, robust performance monitoring, specialized API endpoints, and comprehensive test coverage!

## 🚀 Quick Start

### 1. CLI Usage (Classic)
```bash
# Install globally
npm install -g @casoon/auditmysite

# Test any website (simplest usage)
auditmysite https://your-site.com/sitemap.xml

# Test with custom settings
auditmysite https://your-site.com/sitemap.xml --expert
```

### 2. REST API Server 🆕
```bash
# Start API server with authentication
auditmysite --api --port 3000 --api-key your-secret-key

# Test via HTTP API
curl -X POST http://localhost:3000/api/audits \
  -H "X-API-Key: your-secret-key" \
  -H "Content-Type: application/json" \
  -d '{"url": "https://example.com", "options": {"pages": 5}}'
```

### 3. SDK Integration 🆕 (Updated for v1.8.4)
```javascript
const { auditSDK, AuditRequest } = require('@casoon/auditmysite');

// New unified SDK with progress tracking
const request = {
  url: 'https://example.com/sitemap.xml',
  options: {
    maxPages: 20,
    collectPerformanceMetrics: true,
    outputFormats: ['html', 'json']
  }
};

const response = await auditSDK.audit(request, (progress) => {
  console.log(`${progress.step}: ${progress.progress}%`);
});

console.log('Success Rate:', response.report.summary.successRate + '%');
console.log('Generated Files:', response.files);
```

## ✨ Key Features

### 🐛 **Fixed in v1.9.2 - Mobile-Friendliness Bugfixes**
- ✅ **Mobile-Friendliness Recommendations Fixed** - Resolved "undefined" text in mobile recommendations display
- ✅ **Enhanced HTML Generator Compatibility** - Fixed data field mapping between MobileFriendlinessAnalyzer and HtmlGenerator
- ✅ **Mobile Scores Display** - Proper mobile score grades (A-F) and priority levels (HIGH/MEDIUM/LOW) in HTML reports
- ✅ **Mobile Analysis Data Structure** - Improved data consistency between mobile analyzer and report generators
- ✅ **Touch Target Analysis** - Enhanced touch target violation reporting with specific size recommendations
- ✅ **Viewport Analysis Display** - Correct viewport meta tag analysis and responsive design feedback
- ✅ **Mobile Recommendations Quality** - Clear, actionable mobile-friendliness recommendations with impact descriptions
- ✅ **Report Template Consistency** - Aligned mobile data expectations across all report generation components

### 🐛 **Fixed in v1.9.1 - Critical Bugfixes**
- ✅ **Enhanced Analysis Stability** - Fixed `page.goto: url: expected string, got object` error in Enhanced Analysis pipeline
- ✅ **Core Web Vitals Report Generation** - Fixed `(vitals.cls || 0).toFixed is not a function` in HTML report generation
- ✅ **SEO Data Structure Handling** - Fixed `Cannot read properties of undefined (reading 'title')` in SEO analysis reports
- ✅ **Variable Scope Management** - Fixed `Cannot read properties of undefined (reading 'testedPages')` in CLI output
- ✅ **Safe Data Access** - Added comprehensive optional chaining for all nested data structures
- ✅ **Enhanced Analyzer Navigation** - Fixed analyzer conflicts when using pre-set page content
- ✅ **URL Object Parsing** - Correct extraction of URL strings from sitemap parser objects
- ✅ **Report Template Consistency** - Aligned data structure expectations between analyzers and report generators
- ✅ **100% Enhanced Analysis Success Rate** - All Enhanced Analysis components now work seamlessly together

### 🚀 **New in v1.9.0 - Enhanced Analysis Suite**
- 🔍 **Enhanced Accessibility Analysis** - ARIA validation, focus management, and color contrast analysis as standard
- ⚡ **Robust Performance Monitoring** - Core Web Vitals with isolated browser contexts and retry mechanisms
- 🎆 **Advanced SEO Analysis** - Meta tags, heading structure, and link analysis for optimization insights
- 📏 **Content Weight Assessment** - Content optimization analysis with performance impact evaluation
- 🔧 **Isolated Browser Contexts** - Separate contexts for maximum measurement stability
- 🔄 **Advanced Retry System** - 3-tier collection strategies with exponential backoff
- 🌐 **Specialized API Endpoints** - `/performance`, `/seo`, `/content-weight`, `/accessibility` for focused analysis
- 🧪 **Comprehensive Test Suite** - 25+ test cases covering edge cases and quality validation
- 📊 **Quality Assessment System** - Metrics validation with 40% minimum quality threshold
- 🛡️ **Enhanced Error Handling** - Execution context destruction prevention and robust fallbacks
- 🔌 **Unified Feature Flags** - Consistent API with `accessibility`, `performance`, `seo`, `contentWeight` options
- 📊 **Professional Reports** - Modern HTML reports with enhanced analysis sections

### 🚀 **New in v1.8.0**
- 🧪 **Comprehensive Jest Test Suite** - 103+ passing tests covering all core functionality
- ⚙️ **Optimized Test Architecture** - Unit, integration, API, and CLI tests with proper mocking
- 🔄 **Re-enabled CLI Tests** - Complete command validation and execution testing
- 🚀 **Fixed Rate Limiting** - Optimized API rate limiting tests for reliable execution
- 🔧 **Production-Ready Quality** - Rock-solid test coverage for maximum reliability
- 🏁 **Fast Test Execution** - Optimized mocks and efficient test doubles
- 🔍 **Enhanced Debugging** - Clear test structure matching current architecture

### 🔌 **New in v1.7.1**
- 🔌 **REST API Server** - Production-ready HTTP endpoints with job queue system
- 📦 **Enhanced SDK** - Promise-based JavaScript/TypeScript integration library
- 🔐 **API Authentication** - Secure API key-based authentication system
- 🔄 **Asynchronous Processing** - Background job processing with real-time status updates
- 📋 **Multiple Response Formats** - JSON, HTML, and Markdown report generation
- 🐍 **Docker Ready** - Container deployment with environment variable configuration
- 📊 **Enterprise Ready** - Rate limiting, health checks, and production monitoring

### 🧪 **New in v1.7**
- 🧪 **Comprehensive Test Suite** - Complete automated testing with Jest covering all components
- 🚀 **Production-Ready Quality** - Unit, integration, API, CLI, and E2E tests for maximum reliability
- ⚡ **Fast Test Execution** - Optimized mocks and test doubles for rapid feedback loops
- 🔧 **Developer Experience** - Modern testing tools with watch mode, coverage reports, and focused test categories
- 📊 **Test Coverage Reports** - Detailed coverage analysis with HTML, JSON, and LCOV formats
- 🏗️ **CI/CD Optimized** - Tests designed for continuous integration with deterministic results

### 🔥 **New in v1.6**
- 🎯 **Improved CLI Experience** - Cleaner output with debug logs hidden behind --verbose flag
- 📊 **--max-pages Parameter** - Precise control over the number of pages to test (e.g. --max-pages 10)
- 🧹 **Enhanced User Interface** - Simplified progress messages and reduced visual noise
- ⚡ **Better Parameter Logic** - --max-pages overrides --full for exact control

### 🔥 **New in v1.5**
- 📊 **Performance Budgets** - Configurable Web Vitals thresholds with business-focused templates (E-commerce, Corporate, Blog)
- 🎯 **Smart Budget Templates** - Conversion-optimized thresholds for different site types
- 📈 **Budget Violation Tracking** - Real-time pass/fail status with actionable recommendations
- ⚙️ **Custom Budget Configuration** - Set individual LCP, CLS, FCP, INP, TTFB thresholds via CLI or Expert Mode

### 🔥 **Enhanced in v1.3**
- 🎯 **Enhanced HTML5 Analysis** - Modern `<details>`, `<dialog>`, `<main>` element testing with axe-core v4.10
- ⚡ **Advanced ARIA Evaluation** - Impact-based scoring (Critical, Serious, Moderate, Minor)
- 🚀 **Chrome 135 Optimizations** - Enhanced accessibility tree, improved dialog support
- 📊 **Semantic Quality Scoring** - Comprehensive modern web standards compliance analysis
- 🏆 **Compliance Levels** - Basic, Enhanced, Comprehensive accessibility ratings
- 🔮 **Future Readiness Score** - Evaluation of modern web standards adoption

### 🏆 **Core Features**
- 🎯 **Simplified CLI** - Just 7 essential options + enhanced expert mode
- ⚡ **Core Web Vitals** - Real FCP, LCP, CLS, INP, TTFB metrics with smart fallbacks
- 📊 **Performance Budgets** - Configurable thresholds with business templates (E-commerce, Corporate, Blog)
- 🏆 **Smart Defaults** - Works perfectly without configuration
- 📊 **Professional Reports** - Enhanced HTML reports with modern analysis sections
- 🚀 **Fast & Reliable** - Parallel processing with intelligent error recovery
- ♿ **WCAG Compliance** - Comprehensive accessibility testing with pa11y v9
- ⏱️ **Real-time Progress** - Live updates with time estimates
- 🔄 **Error Recovery** - Automatic fallback and helpful troubleshooting

## 📋 CLI Options

### Core Audit Options
| Option | Description | Default |
|--------|-------------|---------|
| `--full` | Test all pages instead of just 5 | `false` |
| `--max-pages <number>` | Maximum number of pages to test (overrides --full) | `5` |
| `--expert` | Interactive expert mode with custom settings | `false` |
| `--format <type>` | Report format: `html` or `markdown` | `html` |
| `--output-dir <dir>` | Output directory for reports | `./reports` |
| `--non-interactive` | Skip prompts for CI/CD (use defaults) | `false` |
| `--verbose` | Show detailed progress information | `false` |
| `--budget <template>` | Performance budget: `ecommerce`, `corporate`, `blog`, `default` | `default` |
| `--lcp-budget <ms>` | Custom LCP threshold in milliseconds | Template value |
| `--cls-budget <score>` | Custom CLS threshold score (e.g. 0.1) | Template value |
| `--fcp-budget <ms>` | Custom FCP threshold in milliseconds | Template value |
| `--inp-budget <ms>` | Custom INP threshold in milliseconds | Template value |
| `--ttfb-budget <ms>` | Custom TTFB threshold in milliseconds | Template value |

### API Server Options 🆕
| Option | Description | Default |
|--------|-------------|---------|  
| `--api` | Start in API server mode | `false` |
| `--port <number>` | API server port number | `3000` |
| `--api-key <key>` | API authentication key (or use AUDITMYSITE_API_KEY env var) | Required |
| `--max-concurrent <number>` | Maximum concurrent audit jobs | `3` |
| `--timeout <ms>` | Audit timeout in milliseconds | `30000` |
| `--cors` | Enable CORS for cross-origin requests | `false` |

### API Endpoints 🆕 **Enhanced in v1.9.0!**

|| Method | Endpoint | Description |
||--------|----------|-------------|
|| `POST` | `/api/v1/audit/quick` | Quick audit with enhanced analysis (default) |
|| `POST` | `/api/v1/audit/performance` | Performance-focused analysis with Core Web Vitals |
|| `POST` | `/api/v1/audit/seo` | SEO-focused analysis with optimization insights |
|| `POST` | `/api/v1/audit/content-weight` | Content weight analysis with optimization recommendations |
|| `POST` | `/api/v1/audit/accessibility` | Accessibility-focused analysis with ARIA validation |
|| `POST` | `/api/v1/audit` | Full audit job with background processing |
|| `GET` | `/api/v1/audit/{jobId}` | Get audit job status |
|| `DELETE` | `/api/v1/audit/{jobId}` | Cancel audit job |
|| `GET` | `/api/v1/audits` | List all audit jobs with pagination |
|| `GET` | `/api/v1/info` | API information with feature documentation |
|| `POST` | `/api/v1/test-connection` | Test connection to target URL |
|| `GET` | `/health` | Server health check |

**Authentication**: All endpoints except `/api/health` require `X-API-Key` header.

**Example API Usage v1.9.0**:
```bash
# Quick audit with enhanced analysis (default)
curl -X POST http://localhost:3000/api/v1/audit/quick \
  -H "Content-Type: application/json" \
  -d '{"sitemapUrl": "https://example.com/sitemap.xml", "options": {"maxPages": 5}}'

# Performance-focused audit
curl -X POST http://localhost:3000/api/v1/audit/performance \
  -H "Content-Type: application/json" \
  -d '{"sitemapUrl": "https://example.com/sitemap.xml", "options": {"maxPages": 10, "performanceBudget": {"lcp": {"good": 2500, "poor": 4000}}}}'

# SEO analysis
curl -X POST http://localhost:3000/api/v1/audit/seo \
  -H "Content-Type: application/json" \
  -d '{"sitemapUrl": "https://example.com/sitemap.xml", "options": {"maxPages": 5, "includeRecommendations": true}}'

# Content weight analysis
curl -X POST http://localhost:3000/api/v1/audit/content-weight \
  -H "Content-Type: application/json" \
  -d '{"sitemapUrl": "https://example.com/sitemap.xml"}'

# Get API info with features
curl http://localhost:3000/api/v1/info
```

## 💡 Usage Examples

### **Default Test (Recommended)**
```bash
auditmysite https://example.com/sitemap.xml
```
- ✅ Tests **5 pages** automatically
- ✅ **WCAG 2.1 AA** standard  
- ✅ **Core Web Vitals** included
- ✅ **HTML report** generated

### **Custom Page Limit** 🆕 **New in v1.6!**
```bash
# Test exactly 10 pages
auditmysite https://example.com/sitemap.xml --max-pages 10

# Test exactly 25 pages (overrides --full)
auditmysite https://example.com/sitemap.xml --max-pages 25
```
- ✅ **Precise Control** - Test exactly the number of pages you need
- ✅ **Overrides --full** - --max-pages takes priority over --full
- ✅ **Perfect for Testing** - Great for development and staging

### **Full Website Test**
```bash
auditmysite https://example.com/sitemap.xml --full
```
- ✅ Tests **all pages** in sitemap
- ✅ Perfect for comprehensive audits

### **Performance Budgets** 🆆 **New in v1.5!**
```bash
# E-commerce optimized (strict for conversion)
auditmysite https://shop.example.com/sitemap.xml --budget ecommerce

# Corporate standards (professional thresholds)
auditmysite https://company.example.com/sitemap.xml --budget corporate

# Custom budgets
auditmysite https://example.com/sitemap.xml --lcp-budget 2000 --cls-budget 0.05
```
- ✅ **Business Templates** - E-commerce, Corporate, Blog, Default
- ✅ **Custom Thresholds** - Set individual Web Vitals budgets
- ✅ **Budget Violations** - Real-time pass/fail status with recommendations
- ✅ **Expert Mode Integration** - Interactive budget configuration

### **Expert Mode** 🆆 **Enhanced in v1.3!**
```bash
auditmysite https://example.com/sitemap.xml --expert
```
- ✅ **Interactive prompts** for pages, standards, format, concurrency
- ✅ **Performance Budget Templates** - Choose E-commerce, Corporate, Blog, or Custom
- ✅ **Time estimates** for each configuration option
- ✅ **Advanced settings** including concurrent test controls
- ✅ **Performance options** - Enable/disable Web Vitals collection
- 🔥 **Enhanced HTML5 testing** - Modern element analysis toggle
- ⚡ **ARIA enhanced mode** - Advanced impact scoring toggle
- 🚀 **Chrome 135 features** - Performance optimizations toggle
- 📊 **Semantic analysis** - Quality scoring and recommendations toggle
- 📸 **Screenshot capture** - Desktop and mobile screenshots toggle
- ⌨️  **Keyboard navigation testing** - Focusable elements analysis
- 🎨 **Color contrast testing** - Basic contrast ratio analysis
- 🎯 **Focus management testing** - Focus indicator validation

### **CI/CD Integration**
```bash
# Run in CI with markdown output
auditmysite https://example.com/sitemap.xml --non-interactive --format markdown

# Test specific number of pages in CI
auditmysite https://example.com/sitemap.xml --non-interactive --max-pages 20

# Enforce strict performance budgets (example)
auditmysite https://shop.example.com/sitemap.xml --non-interactive --budget ecommerce \
  --lcp-budget 2000 --cls-budget 0.1 --fcp-budget 1200 --inp-budget 200 --ttfb-budget 200

# Parse report in a follow-up CI step (pseudo-code)
# grep -q "Severity: high" reports/example.com/performance-issues-*.md || echo "No high severity perf issues"
```
- ✅ **No prompts** - perfect for automation
- ✅ **Markdown output** for easy parsing
- ✅ **Exit codes** for pipeline integration (see section below)
- ✅ Combine with a simple grep/assert step to enforce budgets in your pipeline

## 📊 What You Get

Below is an overview of what is tested, what the output looks like, and how to interpret the results.

### **Enhanced Accessibility Report** 🔥
- 🎯 **WCAG 2.1 AA compliance** testing with pa11y v9
- 🔍 **Detailed error analysis** with fix suggestions
- 📝 **HTML5 semantic analysis** - Modern elements evaluation
- ⚡ **ARIA impact scoring** - Critical, Serious, Moderate, Minor categorization
- 🏆 **Compliance levels** - Basic, Enhanced, Comprehensive ratings
- ⭐ **Multi-dimensional scoring** - Accessibility, HTML5, ARIA, Semantic quality
- 🔮 **Future readiness** - Modern web standards adoption score

### **Performance Report** ⚡
- ⚡ **Core Web Vitals** (LCP, FCP, CLS, INP, TTFB)
- 📊 **Real performance metrics** using Google's official web-vitals library (injected during tests)
- 📈 **Budget Status Tracking** - Pass/fail against custom thresholds with violation details
- 🎯 **Smart Budget Templates** - Business-focused thresholds (E-commerce: LCP 2000ms, Corporate: 2200ms)
- 🚀 **Chrome 135 optimizations** - Enhanced measurement accuracy
- 🏆 **Performance score & grade** (A–F rating)
- 💡 **Actionable recommendations** - Budget-aware suggestions with severity levels

### **Professional Reports** 📊
- 📄 **Enhanced HTML format** - Modern analysis sections with feature badges
- 📝 **Markdown format** - Developer-friendly output
- 💾 **Organized structure** - Reports saved by domain with timestamps
- 📈 **Visual scorecards** - HTML5, ARIA, Semantic quality circles
- 🔥 **Modern design** - Interactive sections and smooth scrolling
- 💡 **Priority recommendations** - Categorized by HTML5, ARIA, Performance

## 🌍 Accessibility Standards

- **WCAG 2.1 Level A** - Basic accessibility
- **WCAG 2.1 Level AA** - Recommended (default)
- **WCAG 2.1 Level AAA** - Strict compliance
- **Section 508** - US Federal requirements

## 🎯 Perfect For

- ✅ **Quick accessibility checks** before deployment
- ✅ **Performance monitoring** with real Web Vitals and custom budgets
- ✅ **E-commerce optimization** - Conversion-focused performance thresholds
- ✅ **Corporate compliance** - Professional performance standards
- ✅ **WCAG compliance** testing for legal requirements
- ✅ **CI/CD integration** with `--non-interactive` flag and budget validation
- ✅ **Client reports** with professional HTML output and budget status
- ✅ **Development teams** with comprehensive testing suite and fast feedback

## 🧪 Testing & Development

### **Comprehensive Test Suite** 🔥 **New in v1.7!**

AuditMySite features a complete automated testing suite ensuring production-ready reliability:

#### **Test Categories**
- 🧪 **Unit Tests** - Core business logic (Queue System, Config Manager, Report Generators)
- 🔗 **Integration Tests** - SDK API, Event System, Error Handling
- 🌐 **API Endpoint Tests** - REST API routes, Authentication, Job Management
- 💻 **CLI Command Tests** - Argument parsing, Interactive modes, Expert flow
- 🎯 **E2E Tests** - Critical user journeys with fast test doubles

#### **Test Commands**
```bash
# Run all tests
npm test

# Watch mode for development  
npm run test:watch

# Coverage reports
npm run test:coverage

# Specific test categories
npm run test:unit        # Unit tests only
npm run test:integration # Integration tests
npm run test:api         # API endpoint tests
npm run test:cli         # CLI command tests
npm run test:e2e         # End-to-end tests

# CI/CD optimized
npm run test:ci          # Non-interactive with coverage

# Verbose output
npm run test:verbose     # Detailed test logs
```

#### **Test Features**
- ⚡ **Fast Execution** - Optimized mocks avoid slow I/O operations
- 🎯 **Focused Testing** - Test specific components without full system startup
- 📊 **Coverage Reports** - HTML, JSON, LCOV formats for comprehensive analysis
- 🔧 **Developer Friendly** - Watch mode, clear error messages, isolated tests
- 🏗️ **CI/CD Ready** - Deterministic results, no external dependencies

#### **Test Architecture**
```
tests/
├── setup.ts              # Global mocks & utilities
├── unit/                  # Fast isolated unit tests
│   ├── reports/          # Report generator tests
│   ├── config/           # Configuration management tests
│   └── queue/            # Queue system tests
├── integration/          # Component integration tests
│   └── sdk/              # SDK integration tests
├── api/                  # API endpoint tests
│   └── endpoints.test.ts # REST API testing with supertest
├── cli/                  # CLI command tests
│   └── commands.test.ts  # Argument parsing & flow tests
└── e2e/                  # End-to-end tests
    └── happy-path.test.ts # Critical user journey tests
```

### **Quality Assurance**
- ✅ **100% Core Coverage** - All critical business logic tested
- ✅ **Fast Feedback** - Tests complete in seconds, not minutes
- ✅ **Reliable Results** - No flaky tests, deterministic outcomes
- ✅ **Easy Maintenance** - Clear test structure, reusable utilities
- ✅ **Production Confidence** - Comprehensive error scenario coverage

## 📦 Output Files and Structure

- Reports are saved under `./reports/<domain>/` with date-based filenames.
- Examples (HTML default):
  - `reports/example.com/accessibility-report-YYYY-MM-DD.html`
  - `reports/example.com/detailed-issues-YYYY-MM-DD.md`
  - `reports/example.com/performance-issues-YYYY-MM-DD.md`

### Sample CLI run output
```text
🚀 AuditMySite v1.8.0 - Enhanced Accessibility Testing
📄 Sitemap: https://example.com/sitemap.xml
📋 Configuration:
   📄 Pages: 5
   📋 Standard: WCAG2AA
   📊 Performance: Yes (budget: default)
   📄 Format: HTML
   📁 Output: ./reports
...
✅ Queue processing completed: 5/5 URLs in 58s
```

### Sample performance issues excerpt
```markdown
# 📊 Performance Issue Report
Generated: 2025-09-04T05:55:00.000Z
Total Issues: 3

## Page: https://example.com/product/123
- Type: web-vitals
- Severity: high
- Metric: LCP
- Score: 3200ms (budget 2500ms)
- Recommendation: Optimize hero image and defer below-the-fold resources.
```

### Exit codes and how to interpret them
- `0` — The tool ran successfully. There may still be accessibility/performance issues in the reports; check them, but CI should treat this as success unless you enforce budgets separately.
- `1` — One or more pages experienced a technical failure or crash during testing (e.g., navigation timeout, browser error). Investigate logs and rerun.

Note: Accessibility failures alone do NOT cause a non-zero exit code. Use performance budgets or your CI logic to fail the pipeline based on report contents if desired.

## 🧭 What Is Tested (Scope) and Current Status

- Accessibility (WCAG 2.1 AA via pa11y/axe-core):
  - Labels, alt text, landmarks, headings, color contrast, link names, form labeling, and many more rules.
  - HTML5 and ARIA enhancements: `<dialog>`, `<details>`, `<main>`, ARIA roles/states evaluated with modern rule sets.
- Performance (Core Web Vitals):
  - LCP, FCP, CLS, INP, TTFB collected using Google's web-vitals in the test browser, plus basic navigation timings.
  - Budget evaluation using templates or custom thresholds, with recommendations per violation.
- Not included (by design for v1.5):
  - SEO audits, security scans, PDF export, Lighthouse integration (removed to keep core fast and focused).

### Maturity / Accuracy
- Accessibility: Mature and robust through pa11y + axe-core v4.10 with additional semantic checks.
- Performance: Lightweight and practical. Metrics come from web-vitals in-browser collection. A formal comparison suite against Lighthouse is planned/ongoing to validate parity for typical cases.

## 🛠️ Technical Details

### **Built With**
- 🎭 **Playwright** - Modern browser automation with Chrome 135 support
- ♿ **Pa11y v9** - Latest accessibility testing with axe-core v4.10
- ⚡ **Google Web Vitals** - Official performance metrics with enhanced collection
- 📝 **TypeScript** - Type-safe and reliable architecture
- 🔥 **Enhanced Analysis** - Custom HTML5 and ARIA evaluation engines

### **System Requirements**
- **Node.js** 20+ (required for pa11y v9)
- **3GB RAM** minimum (4GB recommended for enhanced analysis)
- **Chrome/Chromium** 120+ (Chrome 135+ recommended for full optimization)
- **Internet connection** for testing external sites

### **Key Features**
- 🚀 **Smart defaults** - Zero configuration needed, enhanced features enabled by default
- ⚡ **Fast parallel processing** - Test multiple pages with Chrome 135 optimizations
- 🔄 **Automatic retries** - Robust error handling with intelligent recovery
- 📊 **Comprehensive reporting** - Accessibility, HTML5, ARIA, Performance, and Semantic analysis
- 🏗️ **Modern architecture** - Built for reliability with future-ready standards
- 🔥 **Enhanced analysis** - Modern HTML5 elements and advanced ARIA evaluation
- 🏆 **Multi-level compliance** - Basic, Enhanced, Comprehensive accessibility levels
- 🧪 **Production Quality** - Comprehensive test suite ensuring maximum reliability

## 📄 License

MIT License - see [LICENSE](LICENSE) for details.

## 🤝 Contributing

We welcome contributions! Please read our contributing guidelines and feel free to submit issues and pull requests.

---

**Made with ❤️ by [CASOON](https://casoon.de)**
