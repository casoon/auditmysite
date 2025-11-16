/**
 * Real-World Test: CASOON.de
 * Test des QA-Frameworks mit echter Website
 */

import { AccessibilityChecker } from '../../src/core/accessibility/accessibility-checker';
import { ReportValidator } from '../../src/validators/report-validator';
import { DataCompletenessChecker } from '../../src/validators/data-completeness-checker';
import { AuditDebugger } from '../../src/utils/audit-debugger';
import { TestOptions, AccessibilityResult } from '../../src/types';
import { Logger } from '../../src/core/logging/logger';
import * as fs from 'fs';

async function testCasoonDE() {
  const logger = new Logger({ level: 'info' });

  logger.section('🌐 Testing Real Website: www.casoon.de');

  // Initialize validation tools
  const checker = new AccessibilityChecker({
    maxConcurrent: 2,
    headless: true,
    verbose: false
  });

  const reportValidator = new ReportValidator();
  const completenessChecker = new DataCompletenessChecker();
  const auditDebugger = new AuditDebugger({
    enableSnapshots: true,
    snapshotInterval: 5000,
    saveDebugData: true,
    debugOutputDir: './casoon-audit-results',
    logMemoryWarnings: true
  });

  auditDebugger.startSession();

  try {
    const baseUrl = 'https://www.casoon.de';

    // Test a few key pages
    const urls = [
      baseUrl,
      `${baseUrl}/leistungen`,
      `${baseUrl}/kontakt`
    ];

    logger.info(`📋 Testing ${urls.length} pages from casoon.de`);
    logger.info('');

    const options: TestOptions = {
      maxPages: urls.length,
      timeout: 30000,
      collectPerformanceMetrics: true,
      usePa11y: true,
      maxRetries: 2,
      waitUntil: 'networkidle',

      eventCallbacks: {
        onUrlStarted: (url: string) => {
          logger.info(`🔄 Starting: ${url}`);
        },

        onUrlCompleted: (url: string, result: AccessibilityResult, duration: number) => {
          logger.success(`✅ Completed: ${url} (${Math.round(duration / 1000)}s)`);

          // Real-time completeness check
          const completeness = completenessChecker.checkPageCompleteness(result);
          logger.info(`   Completeness: ${completeness.score}%`);

          if (!completeness.isComplete) {
            logger.warn(`   ⚠️  Missing fields: ${completeness.missingFields.join(', ')}`);
          }

          // Show key metrics
          if (result.performanceMetrics) {
            logger.info(`   Performance Score: ${result.performanceMetrics.performanceScore || 'N/A'}`);
            logger.info(`   LCP: ${result.performanceMetrics.largestContentfulPaint || 'N/A'}ms`);
          }

          if (result.pa11yScore !== undefined) {
            logger.info(`   Accessibility Score: ${result.pa11yScore}/100`);
          }

          logger.info(`   Errors: ${result.errors.length}, Warnings: ${result.warnings.length}`);
          logger.info('');
        },

        onUrlFailed: (url: string, error: string, attempts: number) => {
          logger.error(`❌ Failed: ${url} (Attempt ${attempts})`);
          logger.debug(`   Error: ${error}`);
        },

        onProgressUpdate: (stats) => {
          const snapshot = auditDebugger.takeSnapshot(
            stats.totalPages,
            stats.completedPages,
            stats.failedPages
          );

          if (stats.completedPages % 2 === 0 && stats.completedPages > 0) {
            auditDebugger.logProgress(snapshot);
          }
        }
      }
    };

    logger.section('🔍 Running Audit');
    const startTime = Date.now();
    const summary = await checker.testUrls(urls, options);
    const totalTime = Date.now() - startTime;

    logger.info('');
    logger.section('✅ Audit Complete');
    logger.info(`Total time: ${Math.round(totalTime / 1000)}s`);
    logger.info('');

    // VALIDATION PHASE
    logger.section('📊 VALIDATION PHASE');

    // 1. Validate individual results
    logger.info('');
    logger.info('1️⃣  Validating individual page results...');
    const resultValidation = reportValidator.validateAuditResults(summary.results);

    if (!resultValidation.valid) {
      logger.error(`❌ VALIDATION FAILED with ${resultValidation.errors.length} errors!`);
      logger.error('');
      const report = reportValidator.generateReport(resultValidation);
      console.log(report);
    } else {
      logger.success(`✅ All ${summary.results.length} results are structurally valid`);
    }

    // 2. Validate summary
    logger.info('');
    logger.info('2️⃣  Validating test summary...');
    const summaryValidation = reportValidator.validateTestSummary(summary);

    if (!summaryValidation.valid) {
      logger.error('❌ SUMMARY VALIDATION FAILED!');
      const report = reportValidator.generateReport(summaryValidation);
      console.log(report);
    } else {
      logger.success('✅ Summary is valid and consistent');
    }

    // 3. Check completeness
    logger.info('');
    logger.info('3️⃣  Checking data completeness...');
    const batchReport = completenessChecker.generateBatchReport(summary.results);

    logger.info(`Overall Completeness Score: ${batchReport.overallScore}%`);
    logger.info(`Complete Pages: ${batchReport.completePages}/${batchReport.totalPages}`);

    if (batchReport.overallScore < 80) {
      logger.warn(`⚠️  Low completeness score!`);
    } else if (batchReport.overallScore >= 90) {
      logger.success('✅ Excellent completeness!');
    } else {
      logger.success('✅ Good completeness');
    }

    // 4. Verify aggregations
    logger.info('');
    logger.info('4️⃣  Verifying aggregations...');
    const aggregationChecks = completenessChecker.verifyAggregations(summary.results);
    const failedChecks = aggregationChecks.filter(c => !c.correct);

    if (failedChecks.length > 0) {
      logger.error(`❌ ${failedChecks.length} aggregation errors found!`);
      failedChecks.forEach(check => {
        logger.error(`   ${check.field}: expected ${check.expected}, got ${check.actual}`);
      });
    } else {
      logger.success('✅ All aggregations are correct');
    }

    // DETAILED RESULTS
    logger.info('');
    logger.section('📋 DETAILED RESULTS');

    summary.results.forEach((result, index) => {
      logger.info('');
      logger.info(`Page ${index + 1}: ${result.url}`);
      logger.info(`  Title: ${result.title || 'N/A'}`);
      logger.info(`  Status: ${result.passed ? '✅ Passed' : '❌ Failed'}`);
      logger.info(`  Duration: ${Math.round(result.duration / 1000)}s`);
      logger.info(`  Errors: ${result.errors.length}`);
      logger.info(`  Warnings: ${result.warnings.length}`);

      if (result.pa11yScore !== undefined) {
        logger.info(`  Pa11y Score: ${result.pa11yScore}/100`);
      }

      if (result.performanceMetrics) {
        logger.info(`  Performance:`);
        logger.info(`    - Score: ${result.performanceMetrics.performanceScore || 'N/A'}/100`);
        logger.info(`    - Grade: ${result.performanceMetrics.performanceGrade || 'N/A'}`);
        logger.info(`    - LCP: ${result.performanceMetrics.largestContentfulPaint || 'N/A'}ms`);
        logger.info(`    - FCP: ${result.performanceMetrics.firstContentfulPaint || 'N/A'}ms`);
        logger.info(`    - CLS: ${result.performanceMetrics.cumulativeLayoutShift || 'N/A'}`);
      }

      // Show sample errors
      if (result.errors.length > 0) {
        logger.info(`  Sample Errors (showing first 3):`);
        result.errors.slice(0, 3).forEach(err => {
          logger.info(`    - ${err}`);
        });
      }
    });

    // SUMMARY
    logger.info('');
    logger.section('📊 FINAL SUMMARY');
    logger.info(`Website: ${baseUrl}`);
    logger.info(`Total Pages Tested: ${summary.testedPages}`);
    logger.info(`Passed: ${summary.passedPages} (${Math.round(summary.passedPages / summary.testedPages * 100)}%)`);
    logger.info(`Failed: ${summary.failedPages}`);
    if (summary.crashedPages) {
      logger.info(`Crashed: ${summary.crashedPages}`);
    }
    logger.info(`Total Errors: ${summary.totalErrors}`);
    logger.info(`Total Warnings: ${summary.totalWarnings}`);
    logger.info(`Total Duration: ${Math.round(summary.totalDuration / 1000)}s`);
    logger.info(`Average Page Time: ${Math.round(summary.totalDuration / summary.testedPages / 1000)}s`);
    logger.info('');

    // Performance report
    logger.section('⚡ PERFORMANCE REPORT');
    const perfReport = auditDebugger.generatePerformanceReport();
    console.log(perfReport);

    // Completeness report
    logger.section('📊 COMPLETENESS REPORT');
    completenessChecker.logBatchReport(batchReport);

    // Save results
    auditDebugger.saveAuditDebugData(summary, 'casoon-de-audit.json');

    // Save full results as JSON
    const resultsDir = './casoon-audit-results';
    if (!fs.existsSync(resultsDir)) {
      fs.mkdirSync(resultsDir, { recursive: true });
    }

    fs.writeFileSync(
      `${resultsDir}/full-results.json`,
      JSON.stringify(summary, null, 2)
    );

    logger.info(`💾 Full results saved to: ${resultsDir}/full-results.json`);
    logger.info('');

    // FINAL VERDICT
    logger.section('🎯 FINAL VERDICT');

    const allValid = resultValidation.valid &&
                     summaryValidation.valid &&
                     batchReport.overallScore >= 80 &&
                     failedChecks.length === 0;

    if (allValid) {
      logger.success('🎉 ALL VALIDATIONS PASSED!');
      logger.success('✅ Results are complete and correct');
      logger.success('✅ All aggregations match');
      logger.success('✅ Data quality is excellent');
      logger.info('');
      logger.info('➡️  You can trust these results!');
    } else {
      logger.warn('⚠️  SOME VALIDATIONS FAILED');
      logger.info('');
      logger.info('Issues found:');
      if (!resultValidation.valid) {
        logger.error(`  - ${resultValidation.errors.length} validation errors`);
      }
      if (!summaryValidation.valid) {
        logger.error(`  - Summary validation failed`);
      }
      if (batchReport.overallScore < 80) {
        logger.warn(`  - Low completeness score: ${batchReport.overallScore}%`);
      }
      if (failedChecks.length > 0) {
        logger.error(`  - ${failedChecks.length} aggregation errors`);
      }
      logger.info('');
      logger.info('➡️  Review the reports above for details');
    }

    logger.info('');
    logger.info('═══════════════════════════════════════════════════════');

    return {
      passed: allValid,
      summary,
      validations: {
        results: resultValidation,
        summary: summaryValidation,
        completeness: batchReport,
        aggregations: aggregationChecks
      }
    };

  } catch (error) {
    logger.error('❌ Audit failed with error:', error);
    throw error;
  } finally {
    auditDebugger.endSession();
    await checker.cleanup();
    logger.info('');
    logger.info('🏁 Test completed');
  }
}

// Run the test
if (require.main === module) {
  testCasoonDE()
    .then((result) => {
      if (result.passed) {
        console.log('\n✅ SUCCESS: All validations passed!');
        process.exit(0);
      } else {
        console.log('\n⚠️  WARNING: Some validations failed!');
        process.exit(1);
      }
    })
    .catch(error => {
      console.error('\n❌ FATAL ERROR:', error);
      process.exit(1);
    });
}

export { testCasoonDE };
