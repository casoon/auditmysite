/**
 * Real-World Test für www.casoon.de
 */

import { createStableAuditor, StableAuditConfig } from './src/interfaces/stable-audit-interface';
import { ReportValidator } from './src/validators/report-validator';
import { DataCompletenessChecker } from './src/validators/data-completeness-checker';

async function testCasoonDE() {
  console.log('\n🌐 Testing Real Website: www.casoon.de\n');

  const config: StableAuditConfig = {
    maxPages: 1,
    timeout: 30000,
    outputFormat: 'both',
    outputDir: './casoon-audit-results',
    standard: 'WCAG2AA',
    verbose: false
  };

  const auditor = createStableAuditor(config);
  const validator = new ReportValidator();
  const completenessChecker = new DataCompletenessChecker();

  try {
    const testUrl = 'https://www.casoon.de';

    console.log(`🔍 Auditing: ${testUrl}\n`);

    await auditor.initialize();

    const result = await auditor.auditWebsite(testUrl);

    console.log('✅ Audit Complete!\n');

    // Validate the result
    console.log('📊 VALIDATION PHASE\n');

    if (result && result.pages && result.pages.length > 0) {
      const page = result.pages[0];

      // Convert to AccessibilityResult format for validation
      const pageResult: any = {
        url: page.url,
        title: page.title,
        passed: page.passed,
        crashed: page.crashed,
        duration: page.duration,
        errors: page.issues?.errors || [],
        warnings: page.issues?.warnings || [],
        pa11yScore: page.scores?.accessibility,
        performanceMetrics: page.scores?.performance ? {
          performanceScore: page.scores.performance,
          largestContentfulPaint: 0,
          firstContentfulPaint: 0
        } : undefined
      };

      console.log('Page Details:');
      console.log(`  URL: ${pageResult.url}`);
      console.log(`  Title: ${pageResult.title || 'N/A'}`);
      console.log(`  Status: ${pageResult.passed ? '✅ Passed' : '❌ Failed'}`);
      console.log(`  Duration: ${Math.round(pageResult.duration / 1000)}s`);
      console.log(`  Errors: ${pageResult.errors?.length || 0}`);
      console.log(`  Warnings: ${pageResult.warnings?.length || 0}`);

      if (pageResult.pa11yScore !== undefined) {
        console.log(`  Pa11y Score: ${pageResult.pa11yScore}/100`);
      }

      if (pageResult.performanceMetrics) {
        console.log(`  Performance Score: ${pageResult.performanceMetrics.performanceScore || 'N/A'}/100`);
        console.log(`  LCP: ${pageResult.performanceMetrics.largestContentfulPaint || 'N/A'}ms`);
        console.log(`  FCP: ${pageResult.performanceMetrics.firstContentfulPaint || 'N/A'}ms`);
      }

      console.log('');

      // Check completeness
      const completeness = completenessChecker.checkPageCompleteness(pageResult);
      console.log(`Completeness Score: ${completeness.score}%`);

      if (!completeness.isComplete) {
        console.log('\n⚠️  Missing Fields:');
        completeness.missingFields.forEach(field => {
          console.log(`  - ${field}`);
        });
      }

      if (completeness.recommendations.length > 0) {
        console.log('\n💡 Recommendations:');
        completeness.recommendations.slice(0, 3).forEach(rec => {
          console.log(`  → ${rec}`);
        });
      }

      // Validate structure
      console.log('\n🔍 Structure Validation:');
      const validation = validator.validateAuditResults([pageResult]);

      if (validation.valid) {
        console.log('✅ Result structure is valid');
      } else {
        console.log(`❌ Validation failed with ${validation.errors.length} errors`);
        validation.errors.forEach(error => {
          console.log(`  - ${error.field}: ${error.message}`);
        });
      }

      // Final verdict
      console.log('\n═══════════════════════════════════════════════════════');
      console.log('🎯 FINAL VERDICT');
      console.log('═══════════════════════════════════════════════════════\n');

      const isGood = validation.valid && completeness.score >= 80;

      if (isGood) {
        console.log('🎉 SUCCESS!');
        console.log('✅ All data is complete and valid');
        console.log('✅ Audit results can be trusted\n');
        return true;
      } else {
        console.log('⚠️  ISSUES FOUND!');
        if (!validation.valid) {
          console.log('❌ Validation errors present');
        }
        if (completeness.score < 80) {
          console.log(`⚠️  Low completeness: ${completeness.score}%`);
        }
        console.log('');
        return false;
      }
    } else {
      console.log('❌ No results returned!');
      return false;
    }

  } catch (error) {
    console.error('\n❌ Test failed with error:');
    console.error(error);
    return false;
  } finally {
    console.log('\n🏁 Test completed\n');
  }
}

// Run
testCasoonDE().then(success => {
  process.exit(success ? 0 : 1);
}).catch(err => {
  console.error('Fatal error:', err);
  process.exit(1);
});
