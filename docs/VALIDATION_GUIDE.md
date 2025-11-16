# Validation Guide: Ensuring Complete and Correct Audit Data

This guide explains how to ensure that your audits produce complete and correctly aggregated data.

## Table of Contents

- [Quick Start](#quick-start)
- [Validation Tools](#validation-tools)
- [Running Validated Audits](#running-validated-audits)
- [Common Issues](#common-issues)
- [Best Practices](#best-practices)

---

## Quick Start

Run a complete audit with full validation:

```bash
# Run the example script
npx ts-node examples/validated-audit-example.ts

# Or run the E2E tests
npm run test:e2e -- complete-audit-validation
```

---

## Validation Tools

### 1. ReportValidator

Validates the structure and consistency of audit results.

```typescript
import { ReportValidator } from './src/validators/report-validator';

const validator = new ReportValidator();

// Validate individual results
const validation = validator.validateAuditResults(summary.results);

if (!validation.valid) {
  console.log(validator.generateReport(validation));
}
```

**What it checks:**
- ✅ Required fields are present (url, title, duration, errors, warnings)
- ✅ Pa11y scores are valid (0-100)
- ✅ Performance metrics are valid
- ✅ Performance scores/grades are in valid ranges
- ✅ Status consistency (not both crashed and passed)

### 2. DataCompletenessChecker

Monitors data completeness in real-time during audit execution.

```typescript
import { DataCompletenessChecker } from './src/validators/data-completeness-checker';

const checker = new DataCompletenessChecker();

// Check single page
const pageCheck = checker.checkPageCompleteness(result);
console.log(`Completeness: ${pageCheck.score}%`);

// Check batch
const batchReport = checker.generateBatchReport(results);
checker.logBatchReport(batchReport);
```

**What it checks:**
- ✅ Critical fields (100% required)
- ✅ Recommended fields (pa11y, performance metrics)
- ✅ Optional fields (screenshots, lighthouse)
- ✅ Provides recommendations for missing data

### 3. AuditDebugger

Monitors performance and memory during audit execution.

```typescript
import { AuditDebugger } from './src/utils/audit-debugger';

const debugger = new AuditDebugger({
  enableSnapshots: true,
  snapshotInterval: 10000, // 10 seconds
  saveDebugData: true
});

debugger.startSession();

// During audit...
const snapshot = debugger.takeSnapshot(total, completed, failed);
debugger.logProgress(snapshot);

// After audit...
debugger.saveAuditDebugData(summary);
debugger.endSession();
```

**What it monitors:**
- 📊 Progress and timing
- 💾 Memory usage
- ⚡ Performance trends
- 🔍 Detailed snapshots

---

## Running Validated Audits

### Method 1: Using the SDK with Callbacks

```typescript
import { AccessibilityChecker } from './src/core/accessibility/accessibility-checker';
import { DataCompletenessChecker } from './src/validators/data-completeness-checker';

const checker = new AccessibilityChecker({ maxConcurrent: 3 });
const completenessChecker = new DataCompletenessChecker();

const summary = await checker.testUrls(urls, {
  collectPerformanceMetrics: true,
  usePa11y: true,

  eventCallbacks: {
    onUrlCompleted: (url, result, duration) => {
      // Validate each page as it completes
      const check = completenessChecker.checkPageCompleteness(result);

      if (!check.isComplete) {
        console.warn(`Incomplete: ${url} (${check.score}%)`);
        check.recommendations.forEach(rec => console.log(`  → ${rec}`));
      }
    }
  }
});

// Validate final results
const validator = new ReportValidator();
const validation = validator.validateAuditResults(summary.results);

if (!validation.valid) {
  console.error('Validation failed!');
  console.log(validator.generateReport(validation));
}
```

### Method 2: Using the Validated Audit Example

```bash
# Copy and customize the example
cp examples/validated-audit-example.ts my-audit.ts

# Edit URLs and options
vim my-audit.ts

# Run
npx ts-node my-audit.ts
```

### Method 3: CLI with Post-Validation

```bash
# Run audit
auditmysite scan https://example.com --output audit-results.json

# Validate results programmatically
npx ts-node -e "
const { ReportValidator } = require('./dist/validators/report-validator');
const results = require('./audit-results.json');
const validator = new ReportValidator();
const validation = validator.validateTestSummary(results);
console.log(validator.generateReport(validation));
"
```

---

## Common Issues

### Issue 1: Missing Performance Metrics

**Symptom:**
```
⚠️  Incomplete data (60% complete)
→ Recommended field missing: performanceMetrics
```

**Solution:**
```typescript
const options = {
  collectPerformanceMetrics: true  // ← Enable this!
};
```

### Issue 2: Low Pa11y Scores or Missing Issues

**Symptom:**
```
⚠️  Pa11y issues missing but errors present
```

**Solution:**
```typescript
const options = {
  usePa11y: true,  // ← Enable pa11y
  pa11yStandard: 'WCAG2AA'
};
```

### Issue 3: Inconsistent Totals

**Symptom:**
```
❌ testedPages does not match sum of passed + failed + crashed
```

**Solution:**
This usually indicates a bug. Check:
- Are crashed pages being counted correctly?
- Are skipped pages being excluded from totals?

Run validation during development:
```bash
npm run test:e2e
```

### Issue 4: Very Short Durations

**Symptom:**
```
⚠️  Very short duration detected (50ms)
→ Page might not have loaded properly
```

**Solution:**
```typescript
const options = {
  timeout: 30000,  // Increase timeout
  waitUntil: 'networkidle'  // Wait for network to be idle
};
```

### Issue 5: High Memory Usage

**Symptom:**
```
⚠️  High memory usage: 612 MB
```

**Solution:**
```typescript
// Reduce concurrent pages
const checker = new AccessibilityChecker({
  maxConcurrent: 2  // Reduce from 3 to 2
});

// Or run with garbage collection
// node --expose-gc --max-old-space-size=4096 your-script.js
```

---

## Best Practices

### 1. Always Validate in Development

Add validation to your development workflow:

```typescript
// In your audit script
if (process.env.NODE_ENV === 'development') {
  const validator = new ReportValidator();
  const validation = validator.validateAuditResults(summary.results);

  if (!validation.valid) {
    throw new Error('Validation failed:\n' + validator.generateReport(validation));
  }
}
```

### 2. Use Real-Time Monitoring

Monitor progress with callbacks:

```typescript
eventCallbacks: {
  onProgressUpdate: (stats) => {
    console.log(`Progress: ${stats.completedPages}/${stats.totalPages}`);

    // Take debug snapshot every 10 pages
    if (stats.completedPages % 10 === 0) {
      const snapshot = debugger.takeSnapshot(
        stats.totalPages,
        stats.completedPages,
        stats.failedPages
      );
      debugger.logProgress(snapshot);
    }
  }
}
```

### 3. Enable All Recommended Options

For complete data, enable:

```typescript
const options = {
  // Core options
  collectPerformanceMetrics: true,
  usePa11y: true,

  // Additional analysis
  testKeyboardNavigation: true,
  testColorContrast: true,
  testFocusManagement: true,

  // Reliability
  maxRetries: 2,
  timeout: 30000,
  waitUntil: 'networkidle'
};
```

### 4. Run E2E Tests Before Production

```bash
# Run full validation test suite
npm run test:e2e -- complete-audit-validation

# Check for regressions
npm run test:regression
```

### 5. Save Debug Data for Analysis

```typescript
const debugger = new AuditDebugger({
  saveDebugData: true,
  debugOutputDir: './audit-logs'
});

// This creates:
// - debug-snapshots-{timestamp}.json
// - audit-debug.json
```

### 6. Check Completeness Scores

Aim for:
- **Critical fields**: 100%
- **Overall score**: ≥ 80%
- **Recommended fields**: ≥ 90%

```typescript
const batchReport = completenessChecker.generateBatchReport(results);

if (batchReport.overallScore < 80) {
  console.warn('Low completeness score!');
  console.log('Missing fields:', Array.from(batchReport.commonMissingFields.entries()));
}
```

---

## Validation Checklist

Before deploying or releasing:

- [ ] Run `npm run test:e2e`
- [ ] All validation tests pass
- [ ] Completeness score ≥ 80%
- [ ] No critical validation errors
- [ ] Aggregations are correct
- [ ] Performance metrics present (if enabled)
- [ ] Pa11y data present (if enabled)
- [ ] Memory usage acceptable
- [ ] No crashed pages (unless expected)

---

## Example Output

When everything is correct, you'll see:

```
═══════════════════════════════════════════════════════
  AUDIT DATA VALIDATION REPORT
═══════════════════════════════════════════════════════

Status: ✅ VALID
Completeness Score: 95%

Statistics:
  Total Pages: 10
  Valid Pages: 10
  Pages with Errors: 0
  Pages with Warnings: 0

✅ All 10 results are valid
✅ Summary is valid and consistent
✅ Good completeness score: 95%
✅ All aggregations are correct

🎉 All validations passed! Audit data is complete and correct.
```

---

## Getting Help

If validation fails:

1. Check the detailed validation report
2. Review [Common Issues](#common-issues)
3. Run with debug logging: `DEBUG=* npm run test`
4. Open an issue with the validation report

---

**Next Steps:**
- See `examples/validated-audit-example.ts` for a complete working example
- Run `npm run test:e2e` to see all validations in action
- Check `debug-output/` directory for detailed debug data
