# Quality Assurance Features

Comprehensive validation and debugging tools to ensure audit data completeness and correctness.

## 🎯 Overview

AuditMySite now includes a complete QA framework with:

- **ReportValidator** - Validates audit result structure and consistency
- **DataCompletenessChecker** - Monitors data completeness in real-time
- **AuditDebugger** - Performance and memory monitoring
- **E2E Validation Tests** - Automated validation test suite

## 🚀 Quick Start

### Run a Validated Audit

```bash
npm run example:validated-audit
```

This runs a complete audit with:
- ✅ Real-time completeness checking
- ✅ Automatic validation of all results
- ✅ Performance monitoring
- ✅ Debug data collection
- ✅ Detailed reports

### Validate Existing Results

```typescript
import { ReportValidator } from './src/validators/report-validator';

const validator = new ReportValidator();
const validation = validator.validateAuditResults(results);

console.log(validator.generateReport(validation));
```

### Run Validation Tests

```bash
# Run all E2E validation tests
npm run validate:audit

# Or use Jest directly
npm run test:e2e -- complete-audit-validation
```

## 📊 Features

### 1. Report Validation

Validates:
- Required fields presence
- Data type correctness
- Score ranges (0-100)
- Status consistency
- Aggregation accuracy

```typescript
const validation = validator.validateAuditResults(results);
// Returns: { valid, errors, warnings, stats }
```

### 2. Completeness Checking

Tracks:
- Critical fields (100% required)
- Recommended fields (pa11y, performance)
- Optional fields (screenshots, lighthouse)
- Completeness score (0-100)

```typescript
const check = checker.checkPageCompleteness(result);
// Returns: { isComplete, score, missingFields, recommendations }
```

### 3. Real-Time Monitoring

Monitor during execution:
- Progress tracking
- Memory usage
- Performance trends
- Time estimates

```typescript
debugger.startSession();
const snapshot = debugger.takeSnapshot(total, completed, failed);
debugger.logProgress(snapshot);
```

### 4. Aggregation Verification

Ensures correct totals:
- Passed vs failed count
- Error/warning totals
- Duration summation
- Crash/skip tracking

```typescript
const checks = checker.verifyAggregations(results);
// Returns: Array<{ field, expected, actual, correct }>
```

## 📝 Usage Examples

### Example 1: Basic Validation

```typescript
import { AccessibilityChecker } from './src/core/accessibility/accessibility-checker';
import { ReportValidator } from './src/validators/report-validator';

const checker = new AccessibilityChecker();
const validator = new ReportValidator();

// Run audit
const summary = await checker.testUrls(['https://example.com'], {
  collectPerformanceMetrics: true,
  usePa11y: true
});

// Validate
const validation = validator.validateTestSummary(summary);

if (!validation.valid) {
  console.error('Validation failed!');
  console.log(validator.generateReport(validation));
  process.exit(1);
}

console.log('✅ All data is valid and complete!');
```

### Example 2: Real-Time Monitoring

```typescript
import { DataCompletenessChecker } from './src/validators/data-completeness-checker';

const checker = new DataCompletenessChecker();

const summary = await auditor.testUrls(urls, {
  eventCallbacks: {
    onUrlCompleted: (url, result, duration) => {
      const check = checker.checkPageCompleteness(result);

      if (check.score < 80) {
        console.warn(`Low completeness: ${url} (${check.score}%)`);
        check.recommendations.forEach(r => console.log(`  → ${r}`));
      }
    }
  }
});
```

### Example 3: Performance Debugging

```typescript
import { AuditDebugger } from './src/utils/audit-debugger';

const debugger = new AuditDebugger({
  enableSnapshots: true,
  saveDebugData: true,
  logMemoryWarnings: true
});

debugger.startSession();

// Run audit...
const summary = await checker.testUrls(urls, {
  eventCallbacks: {
    onProgressUpdate: (stats) => {
      const snapshot = debugger.takeSnapshot(
        stats.totalPages,
        stats.completedPages,
        stats.failedPages
      );
      debugger.logProgress(snapshot);
    }
  }
});

debugger.saveAuditDebugData(summary);
console.log(debugger.generatePerformanceReport());
debugger.endSession();
```

## 🧪 Testing

### E2E Validation Tests

Located in `tests/e2e/complete-audit-validation.test.ts`:

```bash
npm run test:e2e
```

Tests include:
- ✅ Single page audit validation
- ✅ Multiple pages audit validation
- ✅ Summary consistency checks
- ✅ Aggregation verification
- ✅ Data quality checks
- ✅ Error handling

### Manual Testing

```bash
# Run example with your URLs
npx ts-node examples/validated-audit-example.ts

# Check debug output
ls -la debug-output/
cat debug-output/audit-debug.json
```

## 📈 Validation Reports

### Completeness Report

```
═══════════════════════════════════════════════════════
  DATA COMPLETENESS REPORT
═══════════════════════════════════════════════════════

Overall Score: 95%
Total Pages: 10
Complete Pages: 9 (90%)
Incomplete Pages: 1 (10%)

Common Missing Fields:
  - screenshots: 10 pages (100%)
  - lighthouseScores: 8 pages (80%)

Pages Needing Attention:
  https://example.com/contact (75%)
    → Recommended field missing: performanceMetrics
```

### Validation Report

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
```

### Performance Report

```
═══════════════════════════════════════════════════════
  AUDIT PERFORMANCE REPORT
═══════════════════════════════════════════════════════

Overall Statistics:
  Total Pages: 10
  Completed: 10
  Failed: 0
  Total Time: 125s
  Average Page Time: 12s

Memory Usage:
  Average: 345 MB
  Peak: 512 MB

Performance Trend:
  ✓ Consistent performance
```

## 🔧 Configuration

### ReportValidator Options

```typescript
const validator = new ReportValidator();
// No configuration needed - uses sensible defaults
```

### DataCompletenessChecker Options

```typescript
const checker = new DataCompletenessChecker();
// Configurable via constructor (currently uses defaults)
```

### AuditDebugger Options

```typescript
const debugger = new AuditDebugger({
  enableSnapshots: true,           // Enable periodic snapshots
  snapshotInterval: 10000,         // Snapshot every 10s
  saveDebugData: true,             // Save to files
  debugOutputDir: './debug-output', // Output directory
  logMemoryWarnings: true,         // Warn on high memory
  memoryWarningThreshold: 512      // Warning threshold (MB)
});
```

## 🎯 Best Practices

1. **Always validate in development**
   - Catch issues early
   - Ensure data completeness
   - Verify aggregations

2. **Use real-time monitoring**
   - Track progress
   - Monitor memory
   - Catch failures immediately

3. **Enable all recommended options**
   ```typescript
   {
     collectPerformanceMetrics: true,
     usePa11y: true,
     maxRetries: 2
   }
   ```

4. **Run E2E tests before releases**
   ```bash
   npm run validate:audit
   ```

5. **Save debug data for analysis**
   - Helps troubleshoot issues
   - Track performance trends
   - Verify completeness over time

## 📚 Documentation

- [Validation Guide](./VALIDATION_GUIDE.md) - Comprehensive guide
- [Example Script](../examples/validated-audit-example.ts) - Working example
- [E2E Tests](../tests/e2e/complete-audit-validation.test.ts) - Test suite

## 🐛 Troubleshooting

### Low Completeness Score

**Problem:** Score < 80%

**Solution:**
```typescript
// Enable all analysis options
{
  collectPerformanceMetrics: true,
  usePa11y: true,
  testKeyboardNavigation: true,
  testColorContrast: true
}
```

### Validation Errors

**Problem:** `validation.valid === false`

**Solution:**
```typescript
// Check the detailed report
console.log(validator.generateReport(validation));

// Fix issues based on error messages
// Most common: missing required fields
```

### High Memory Usage

**Problem:** Memory warnings during execution

**Solution:**
```typescript
// Reduce concurrent pages
const checker = new AccessibilityChecker({
  maxConcurrent: 2  // Instead of 3
});

// Or run with more memory
// node --max-old-space-size=4096 script.js
```

## 🚀 Getting Started

1. **Install dependencies** (already done)
   ```bash
   npm install
   ```

2. **Run the example**
   ```bash
   npm run example:validated-audit
   ```

3. **Customize for your needs**
   ```bash
   cp examples/validated-audit-example.ts my-audit.ts
   # Edit my-audit.ts with your URLs
   npx ts-node my-audit.ts
   ```

4. **Integrate into your workflow**
   - Add validation to CI/CD
   - Monitor production audits
   - Track completeness over time

---

**Questions?** See [VALIDATION_GUIDE.md](./VALIDATION_GUIDE.md) for more details.
