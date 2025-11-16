#!/usr/bin/env node

/**
 * Real-World Audit Test: inros-lackner.de
 * Using compiled code to avoid module resolution issues
 */

const { AccessibilityChecker } = require('./dist/core/accessibility/accessibility-checker');
const { BrowserPoolManager } = require('./dist/core/browser/browser-pool-manager');
const { ReportValidator } = require('./dist/validators/report-validator');
const { DataCompletenessChecker } = require('./dist/validators/data-completeness-checker');
const { Logger } = require('./dist/core/logging/logger');

async function auditInrosLackner() {
  const logger = new Logger({ level: 'info' });

  console.log('\n');
  logger.section('🏢 AUDIT: INROS LACKNER');
  console.log('Website: https://www.inros-lackner.de/de');
  console.log('Mit vollständiger Validation & QA-Prüfung');
  console.log('');

  const browserPool = new BrowserPoolManager({ maxConcurrent: 2 });
  const checker = new AccessibilityChecker({
    poolManager: browserPool,
    enableComprehensiveAnalysis: false
  });
  const validator = new ReportValidator();
  const completenessChecker = new DataCompletenessChecker();

  try {
    // Test first 5 pages starting from homepage
    const urls = ['https://www.inros-lackner.de/de'];

    logger.info('🔧 Starting audit...');
    const startTime = Date.now();

    const result = await checker.testMultiplePages(urls, {
      collectPerformanceMetrics: true,
      usePa11y: true,
      timeout: 45000,
      verbose: true,
      maxRetries: 2,
      eventCallbacks: {
        onProgressUpdate: (stats) => {
          if (stats.completedPages % 1 === 0 || stats.completedPages === stats.totalPages) {
            const progress = stats.totalPages > 0
              ? Math.round((stats.completedPages / stats.totalPages) * 100)
              : 0;
            const bar = '█'.repeat(Math.round(progress / 5)) + '░'.repeat(20 - Math.round(progress / 5));
            console.log(`\n📊 PROGRESS: [${bar}] ${progress}%`);
            console.log(`   Completed: ${stats.completedPages}/${stats.totalPages}`);
          }
        },
        onUrlCompleted: (url, result, duration) => {
          console.log(`\n✅ Completed: ${url}`);
          console.log(`   Duration: ${Math.round(duration / 1000)}s`);

          // Check completeness
          const check = completenessChecker.checkPageCompleteness(result);
          console.log(`   Completeness: ${check.score}%`);

          if (!check.isComplete) {
            console.log(`   ⚠️  Missing fields: ${check.missingFields.join(', ')}`);
          }
        },
        onUrlFailed: (url, error, attempts) => {
          logger.error(`❌ Failed: ${url} (attempt ${attempts})`);
          logger.error(`   Error: ${error}`);
        }
      }
    });

    const duration = (Date.now() - startTime) / 1000;

    console.log('');
    logger.section('✅ AUDIT COMPLETE');
    logger.success(`Completed in ${duration.toFixed(1)}s`);
    console.log('');

    // ========================================
    // VALIDATION PHASE
    // ========================================
    logger.section('📊 VALIDATION & QUALITY CHECK');
    console.log('');

    logger.info('1️⃣  Validating result structure...');
    const validation = validator.validateTestSummary(summary);

    if (validation.valid) {
      logger.success(`✅ All ${summary.results.length} results are structurally valid`);
    } else {
      logger.error(`❌ Validation failed with ${validation.errors.length} errors`);
      validation.errors.slice(0, 5).forEach(error => {
        logger.error(`  - ${error.field}: ${error.message}`);
      });
    }

    console.log('');
    logger.info('2️⃣  Checking data completeness...');

    const batchReport = completenessChecker.generateBatchReport(summary.results);

    logger.info(`Overall Completeness: ${batchReport.overallScore}%`);
    logger.info(`Complete Pages: ${batchReport.completePages}/${batchReport.totalPages}`);

    if (batchReport.overallScore >= 90) {
      logger.success('✅ Excellent data completeness!');
    } else if (batchReport.overallScore >= 80) {
      logger.success('✅ Good data completeness');
    } else {
      logger.warn(`⚠️  Low completeness score: ${batchReport.overallScore}%`);
    }

    if (batchReport.commonMissingFields.size > 0) {
      console.log('');
      logger.info('Common missing fields:');
      Array.from(batchReport.commonMissingFields.entries())
        .slice(0, 5)
        .forEach(([field, count]) => {
          console.log(`  - ${field}: ${count} pages`);
        });
    }

    // ========================================
    // DETAILED RESULTS
    // ========================================
    console.log('');
    logger.section('📋 AUDIT RESULTS SUMMARY');
    console.log('');

    console.log('Website Overview:');
    console.log(`  Domain: inros-lackner.de`);
    console.log(`  Total Pages Audited: ${summary.testedPages}`);
    console.log(`  Success Rate: ${summary.successRate.toFixed(1)}%`);
    console.log(`  Total Duration: ${duration.toFixed(1)}s`);
    console.log('');

    console.log('Pass/Fail Status:');
    console.log(`  ✅ Passed: ${summary.passedPages}`);
    console.log(`  ❌ Failed: ${summary.failedPages}`);
    if (summary.crashedPages > 0) {
      console.log(`  💥 Crashed: ${summary.crashedPages}`);
    }
    console.log('');

    // ========================================
    // PAGE-BY-PAGE ANALYSIS
    // ========================================
    logger.section('📄 PAGE-BY-PAGE ANALYSIS');
    console.log('');

    summary.results.forEach((page, index) => {
      const status = page.crashed ? '💥 CRASHED' : page.passed ? '✅ PASSED' : '❌ FAILED';
      const completeness = completenessChecker.checkPageCompleteness(page);

      console.log(`${index + 1}. ${status} - ${page.title || 'Untitled'}`);
      console.log(`   URL: ${page.url}`);
      console.log(`   Duration: ${page.duration}ms`);
      console.log(`   Completeness: ${completeness.score}%`);

      if (page.pa11yScore !== undefined) {
        console.log(`   Pa11y Score: ${page.pa11yScore}/100`);
      }

      if (page.performanceMetrics) {
        console.log(`   Performance Score: ${page.performanceMetrics.performanceScore || 'N/A'}/100`);
      }

      console.log(`   Issues:`);
      console.log(`     - ${page.errors.length} errors`);
      console.log(`     - ${page.warnings.length} warnings`);

      if (page.errors.length > 0) {
        console.log(`   Top Errors (showing first 3):`);
        page.errors.slice(0, 3).forEach(error => {
          console.log(`     • ${error}`);
        });
      }

      if (!completeness.isComplete) {
        console.log(`   ⚠️  Missing: ${completeness.missingFields.join(', ')}`);
      }

      console.log('');
    });

    // ========================================
    // QUALITY ASSESSMENT
    // ========================================
    logger.section('🎯 QUALITY ASSESSMENT');
    console.log('');

    const qualityChecks = {
      structureValid: validation.valid,
      completenessGood: batchReport.overallScore >= 80,
      hasResults: summary.results.length > 0,
      noErrors: validation.errors.length === 0,
      passedTests: summary.passedPages > 0
    };

    const passedChecks = Object.values(qualityChecks).filter(v => v).length;
    const totalChecks = Object.keys(qualityChecks).length;

    console.log('Quality Checks:');
    console.log(`  ${qualityChecks.structureValid ? '✅' : '❌'} Data structure is valid`);
    console.log(`  ${qualityChecks.completenessGood ? '✅' : '❌'} Data completeness ≥ 80%`);
    console.log(`  ${qualityChecks.hasResults ? '✅' : '❌'} Results were generated`);
    console.log(`  ${qualityChecks.noErrors ? '✅' : '❌'} No validation errors`);
    console.log(`  ${qualityChecks.passedTests ? '✅' : '❌'} At least one page passed`);
    console.log('');

    const qualityScore = (passedChecks / totalChecks) * 100;
    console.log(`Overall Quality Score: ${qualityScore.toFixed(0)}%`);
    console.log('');

    // ========================================
    // FINAL VERDICT
    // ========================================
    logger.section('🏆 FINAL VERDICT');
    console.log('');

    if (qualityScore >= 80) {
      logger.success('🎉 EXCELLENT - Results are trustworthy and complete!');
      console.log('');
      console.log('✅ The audit results are:');
      console.log('  • Structurally valid');
      console.log('  • Complete and comprehensive');
      console.log('  • Ready for production use');
      console.log('  • Suitable for decision-making');
    } else if (qualityScore >= 60) {
      logger.info('⚠️  GOOD - Results are usable with minor limitations');
      console.log('');
      console.log('The audit results are generally good, but consider:');
      if (!qualityChecks.completenessGood) {
        console.log('  • Enabling more analysis options for completeness');
      }
      if (!qualityChecks.noErrors) {
        console.log('  • Reviewing validation errors');
      }
    } else {
      logger.warn('❌ NEEDS IMPROVEMENT - Results have quality issues');
      console.log('');
      console.log('Please review the validation errors above.');
    }

    console.log('');
    console.log('═══════════════════════════════════════════════════════');
    console.log('');

    logger.success('🏁 Audit session completed successfully');
    console.log('');

    // Cleanup
    await browserPool.cleanup();

    return qualityScore >= 80;

  } catch (error) {
    console.log('');
    logger.error('❌ Audit failed with error:');
    console.error(error);
    console.log('');

    // Cleanup on error
    try {
      await browserPool.cleanup();
    } catch (cleanupError) {
      // Ignore cleanup errors
    }

    return false;
  }
}

// Run the audit
auditInrosLackner()
  .then(success => {
    process.exit(success ? 0 : 1);
  })
  .catch(error => {
    console.error('Fatal error:', error);
    process.exit(1);
  });
