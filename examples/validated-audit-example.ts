/**
 * Validated Audit Example
 *
 * Demonstrates how to run an audit with complete validation and debugging
 * This ensures all data is properly collected and aggregated
 */

import { AccessibilityChecker } from '../src/core/accessibility/accessibility-checker';
import { ReportValidator } from '../src/validators/report-validator';
import { DataCompletenessChecker } from '../src/validators/data-completeness-checker';
import { AuditDebugger } from '../src/utils/audit-debugger';
import { TestOptions, AccessibilityResult } from '../src/types';
import { Logger } from '../src/core/logging/logger';

async function runValidatedAudit() {
  const logger = new Logger({ level: 'info' });

  logger.section('🚀 Starting Validated Audit');

  // Initialize tools
  const checker = new AccessibilityChecker({
    maxConcurrent: 3,
    headless: true,
    verbose: false
  });

  const reportValidator = new ReportValidator();
  const completenessChecker = new DataCompletenessChecker();
  const debugger = new AuditDebugger({
    enableSnapshots: true,
    snapshotInterval: 10000, // Every 10 seconds
    saveDebugData: true,
    debugOutputDir: './debug-output',
    logMemoryWarnings: true,
    memoryWarningThreshold: 512 // Warn if memory > 512 MB
  });

  // Start debug session
  debugger.startSession();

  try {
    // Define URLs to audit
    const urls = [
      'https://example.com',
      'https://example.com/about',
      'https://example.com/contact'
    ];

    logger.info(`📋 Auditing ${urls.length} URLs`);
    logger.info('');

    // Configure audit options
    const options: TestOptions = {
      maxPages: urls.length,
      timeout: 30000,
      collectPerformanceMetrics: true,
      usePa11y: true,
      maxRetries: 2,

      // Real-time callbacks for monitoring
      eventCallbacks: {
        onUrlStarted: (url: string) => {
          logger.info(`🔄 Starting: ${url}`);
        },

        onUrlCompleted: (url: string, result: AccessibilityResult, duration: number) => {
          logger.success(`✅ Completed: ${url} (${Math.round(duration / 1000)}s)`);

          // Check completeness immediately
          const completeness = completenessChecker.checkPageCompleteness(result);

          if (!completeness.isComplete) {
            logger.warn(`⚠️  Incomplete data for ${url}`);
            completeness.recommendations.forEach(rec => {
              logger.debug(`   → ${rec}`);
            });
          }
        },

        onUrlFailed: (url: string, error: string, attempts: number) => {
          logger.error(`❌ Failed: ${url} (Attempt ${attempts})`);
          logger.debug(`   Error: ${error}`);
        },

        onProgressUpdate: (stats) => {
          // Take debug snapshot
          const snapshot = debugger.takeSnapshot(
            stats.totalPages,
            stats.completedPages,
            stats.failedPages
          );

          // Log progress periodically
          if (stats.completedPages % 5 === 0) {
            debugger.logProgress(snapshot);
          }
        }
      }
    };

    logger.info('⚙️  Options:');
    logger.info(`   - Performance Metrics: ${options.collectPerformanceMetrics}`);
    logger.info(`   - Pa11y Analysis: ${options.usePa11y}`);
    logger.info(`   - Max Retries: ${options.maxRetries}`);
    logger.info('');

    // Run the audit
    logger.section('🔍 Running Audit');
    const summary = await checker.testUrls(urls, options);

    logger.info('');
    logger.section('✅ Audit Complete');

    // Validate results
    logger.section('📊 Validating Results');

    // 1. Validate individual results
    logger.info('');
    logger.info('1️⃣  Validating individual page results...');
    const resultValidation = reportValidator.validateAuditResults(summary.results);
    reportValidator.logValidation(resultValidation);

    if (!resultValidation.valid) {
      logger.error('❌ Result validation failed!');
      const report = reportValidator.generateReport(resultValidation);
      console.log(report);
    } else {
      logger.success(`✅ All ${summary.results.length} results are valid`);
    }

    // 2. Validate test summary
    logger.info('');
    logger.info('2️⃣  Validating test summary...');
    const summaryValidation = reportValidator.validateTestSummary(summary);
    reportValidator.logValidation(summaryValidation);

    if (!summaryValidation.valid) {
      logger.error('❌ Summary validation failed!');
      const report = reportValidator.generateReport(summaryValidation);
      console.log(report);
    } else {
      logger.success('✅ Summary is valid and consistent');
    }

    // 3. Check data completeness
    logger.info('');
    logger.info('3️⃣  Checking data completeness...');
    const batchReport = completenessChecker.generateBatchReport(summary.results);
    completenessChecker.logBatchReport(batchReport);

    if (batchReport.overallScore < 80) {
      logger.warn(`⚠️  Low completeness score: ${batchReport.overallScore}%`);
    } else {
      logger.success(`✅ Good completeness score: ${batchReport.overallScore}%`);
    }

    // 4. Verify aggregations
    logger.info('');
    logger.info('4️⃣  Verifying aggregations...');
    const aggregationChecks = completenessChecker.verifyAggregations(summary.results);
    completenessChecker.logAggregationChecks(aggregationChecks);

    // 5. Generate performance report
    logger.info('');
    logger.section('⚡ Performance Report');
    const perfReport = debugger.generatePerformanceReport();
    console.log(perfReport);

    // Save debug data
    debugger.saveAuditDebugData(summary);

    // Final summary
    logger.section('📋 Final Summary');
    logger.info(`Total Pages: ${summary.totalPages}`);
    logger.info(`Tested Pages: ${summary.testedPages}`);
    logger.info(`Passed: ${summary.passedPages} (${Math.round(summary.passedPages / summary.testedPages * 100)}%)`);
    logger.info(`Failed: ${summary.failedPages}`);
    if (summary.crashedPages) {
      logger.info(`Crashed: ${summary.crashedPages}`);
    }
    logger.info(`Total Errors: ${summary.totalErrors}`);
    logger.info(`Total Warnings: ${summary.totalWarnings}`);
    logger.info(`Duration: ${Math.round(summary.totalDuration / 1000)}s`);
    logger.info('');

    // Overall validation status
    const allValid = resultValidation.valid &&
                     summaryValidation.valid &&
                     batchReport.overallScore >= 80;

    if (allValid) {
      logger.success('🎉 All validations passed! Audit data is complete and correct.');
    } else {
      logger.warn('⚠️  Some validations failed. Review the reports above.');
    }

  } catch (error) {
    logger.error('❌ Audit failed with error:', error);
    throw error;
  } finally {
    // Cleanup
    debugger.endSession();
    await checker.cleanup();
    logger.info('');
    logger.info('🏁 Session ended');
  }
}

// Run the validated audit
if (require.main === module) {
  runValidatedAudit()
    .then(() => {
      process.exit(0);
    })
    .catch(error => {
      console.error('Fatal error:', error);
      process.exit(1);
    });
}

export { runValidatedAudit };
