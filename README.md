# AuditMySite


**Version 2.1.0** - Professional Website Accessibility & Performance Testing Tool

AuditMySite is a comprehensive website analysis suite that provides professional accessibility testing, Core Web Vitals performance monitoring, SEO analysis, and content optimization insights. Built for developers, agencies, and businesses who need reliable, automated website quality assessments.

## 🚀 Quick Start

### Installation

```bash
npm install -g @casoon/auditmysite
```

### Basic Usage

```bash
# Test accessibility and performance for up to 5 pages
auditmysite https://example.com/sitemap.xml

# Generate JSON report with complete data
auditmysite https://example.com/sitemap.xml --format json

# Test 10 pages with detailed progress
auditmysite https://example.com/sitemap.xml --max-pages 10 --verbose
```

## 📊 What You Get

### Comprehensive Analysis Results

Each page tested provides detailed insights across **5 key areas**:

#### 🔍 **Accessibility Analysis**
- **WCAG 2.1 AA compliance testing** using pa11y and axe-core
- **Accessibility score** (0-100) with detailed issue breakdown
- **Error categorization**: Critical errors, warnings, and notices
- **Specific fixes**: Exact HTML elements and recommended solutions
- **Screen reader compatibility** and keyboard navigation issues
- **🆕 Glassmorphism false positive detection**: Automatically detects and filters backdrop-blur/transparent elements that cause color-contrast false positives
- **🆕 Intelligent error deduplication**: Eliminates duplicate error reporting for cleaner results
- **🆕 Filtering transparency**: See exactly how many issues were filtered and why (metadata included in reports)

#### ⚡ **Performance Analysis**
- **Core Web Vitals**: LCP, FCP, CLS, INP, TTFB with Google's official thresholds
- **Performance score** (0-100) and letter grade (A-F)
- **Performance budget compliance** with industry-specific templates
- **Mobile and desktop optimization recommendations**
- **Loading speed analysis** and resource optimization suggestions

#### 🔍 **SEO Analysis**
- **SEO score** (0-100) with actionable recommendations
- **Meta tags analysis**: Title, description, keywords optimization
- **Content quality**: Word count, readability score, text-to-code ratio
- **Heading structure**: H1-H6 hierarchy and semantic analysis
- **Social media tags**: Open Graph and Twitter Card validation
- **Technical SEO**: Internal/external links, image alt text coverage

#### 📏 **Content Weight Analysis**
- **Page weight breakdown**: HTML, CSS, JavaScript, images, fonts
- **Resource optimization score** with specific improvement suggestions
- **File compression** and delivery optimization recommendations
- **Content efficiency metrics** and performance impact analysis

#### 📱 **Mobile-Friendliness Analysis**
- **Mobile usability score** across 8 categories
- **Touch target analysis**: Size and spacing validation
- **Responsive design testing**: Viewport and layout optimization
- **Mobile performance**: Mobile-specific Core Web Vitals
- **User experience factors**: Navigation, forms, and interaction quality

### Report Formats

#### **HTML Report** (Default)
- **Professional dashboard** with executive summary
- **Interactive charts** and visual performance indicators  
- **Detailed issue breakdown** with priority rankings
- **Before/after recommendations** with implementation guidance
- **Exportable and shareable** for client presentations

#### **JSON Report** (--format json)
- **Complete structured data** for integration and automation
- **API-friendly format** for CI/CD pipelines
- **System performance metrics** including parallel processing stats
- **Machine-readable results** for custom reporting and analysis

## 🛠️ Command Line Options

### Required Arguments
```bash
auditmysite <sitemapUrl>    # URL of the sitemap.xml to analyze
```

### Core Options
```bash
--max-pages <number>        # Pages to test (default: 5)
--format <type>             # Report format: html or json (default: html)
--output-dir <dir>          # Output directory (default: ./reports)
--budget <template>         # Performance budget (default, ecommerce, blog, corporate)
```

### Analysis Control
```bash
--no-performance           # Disable performance analysis
--no-seo                   # Disable SEO analysis  
--no-content-weight        # Disable content weight analysis
--no-mobile                # Disable mobile-friendliness analysis
```

### Advanced Options
```bash
--expert                   # Interactive expert mode with advanced settings
--non-interactive          # Skip prompts for CI/CD (use defaults)
--verbose                  # Show detailed progress information
--save-state               # Save audit state for resumption
--resume <stateId>         # Resume a previous audit from saved state
--list-states              # List all available saved audit states
```

### Examples

```bash
# Complete analysis with performance budgets for e-commerce
auditmysite https://shop.example.com/sitemap.xml --budget ecommerce --max-pages 20

# JSON output for CI/CD integration
auditmysite https://example.com/sitemap.xml --format json --non-interactive --max-pages 50

# Quick accessibility-only check
auditmysite https://example.com/sitemap.xml --no-performance --no-seo --no-content-weight --no-mobile

# Expert mode with custom settings
auditmysite https://example.com/sitemap.xml --expert --verbose

# Resume interrupted audit
auditmysite --list-states
auditmysite --resume audit_20241211_143022_example.com
```

## 🏆 Performance Budgets

AuditMySite includes business-focused performance budget templates:

### **Default** (Google Web Vitals Standard)
- LCP: ≤2.5s, FCP: ≤1.8s, CLS: ≤0.1, INP: ≤200ms

### **E-commerce** (Conversion Optimized)
- LCP: ≤2.0s, FCP: ≤1.5s, CLS: ≤0.05, INP: ≤150ms
- Optimized for shopping experiences and conversion rates

### **Corporate** (Professional Standards)  
- LCP: ≤2.5s, FCP: ≤1.8s, CLS: ≤0.1, INP: ≤200ms
- Balanced for business websites and professional services

### **Blog** (Content Focused)
- LCP: ≤3.0s, FCP: ≤2.0s, CLS: ≤0.1, INP: ≤300ms
- Optimized for content consumption and reading experience

## 🏗️ Architecture Features

- **Event-driven parallel processing** with concurrent workers
- **Browser pooling** for optimal performance and resource usage
- **Persistent state management** with resume capability
- **Real-time progress reporting** with memory and CPU monitoring
- **Automatic sitemap discovery** from domain roots
- **Comprehensive error handling** with retry mechanisms

## 📈 Integration & Automation

### CI/CD Integration
```bash
# GitHub Actions / GitLab CI
auditmysite $SITEMAP_URL --format json --non-interactive --max-pages 10
```

### Node.js SDK Usage
```javascript
const { UnifiedAuditSDK } = require('@casoon/auditmysite');

const sdk = new UnifiedAuditSDK();
const results = await sdk.auditWebsite('https://example.com/sitemap.xml', {
  maxPages: 10,
  outputFormat: 'json'
});
```

### API Server Mode
```bash
# Start API server
auditmysite --api --port 3000 --api-key your-secret-key

# API endpoints available:
# POST /api/v1/audit/quick - Complete audit with all analysis types
# POST /api/v1/audit/performance - Performance-focused analysis  
# POST /api/v1/audit/seo - SEO-focused analysis
# POST /api/v1/audit/accessibility - Accessibility-focused analysis
# GET /api/v1/audit/{jobId} - Get audit job status
```

## 🎨 Advanced Features

### Glassmorphism False Positive Filtering

Modern web designs often use glassmorphism effects (backdrop-blur with transparent backgrounds). These cause axe-core to incorrectly report color-contrast violations because the tool analyzes static DOM, not final rendered pixels.

**How it works:**
- Automatically detects elements with `backdrop-filter` or semi-transparent backgrounds
- Filters color-contrast errors on detected glassmorphism elements
- Provides filtering metadata showing exactly what was filtered and why

**Transparency:**
```json
{
  "filteringMetadata": {
    "originalIssuesCount": 42,
    "deduplicatedIssuesCount": 21,
    "filteredIssuesCount": 0,
    "glassmorphismElementsDetected": 8,
    "whitelistedIssuesCount": 0
  }
}
```

### Whitelist System

For cases where automated detection isn't enough, use the whitelist system for URL-specific rules:

**Configuration:** `src/core/config/accessibility-whitelist.ts`

**Example:**
```typescript
{
  url: 'https://example.com',
  ignoreRules: {
    'color-contrast': {
      selectors: ['.glassmorphism-card'],
      reason: 'Manual verification: 12.5:1 contrast ratio (WCAG AAA compliant)',
      addedDate: '2025-11-16'
    }
  }
}
```

**Documentation:** See [docs/ACCESSIBILITY_MANUAL_VERIFICATION.md](docs/ACCESSIBILITY_MANUAL_VERIFICATION.md) for manual verification guidelines.

## 🎯 Use Cases

- **Development Teams**: Automated quality assurance in CI/CD pipelines
- **Agencies**: Client reporting and website optimization services  
- **E-commerce**: Conversion optimization through performance and UX analysis
- **Enterprise**: Large-scale website monitoring and compliance tracking
- **SEO Specialists**: Technical SEO audits with actionable insights

## 🔧 System Requirements

- **Node.js**: 18+ (required for pa11y v9 compatibility)
- **Memory**: 2GB minimum, 4GB recommended for large audits
- **Chrome/Chromium**: 120+ (automatically managed)
- **Network**: Required for testing external websites

## 📊 Exit Codes

- **0**: Successful completion (accessibility issues don't cause failure)
- **1**: Technical errors (network failures, parsing errors, system issues)

This ensures CI/CD pipelines can distinguish between technical failures and expected accessibility findings.

## 📦 Version

Current version: **2.0.0-alpha.2**

---

**AuditMySite** - Professional website quality analysis made simple.
