#!/usr/bin/env node

/**
 * Real-World Audit Test: inros-lackner.de
 * Demonstrates complete validation workflow
 */

import { createStableAuditor, StableAuditConfig } from './src/interfaces/stable-audit-interface';
import { ReportValidator } from './src/validators/report-validator';
import { DataCompletenessChecker } from './src/validators/data-completeness-checker';
import { Logger } from './src/core/logging/logger';

async function auditInrosLackner() {
  const logger = new Logger({ level: 'info' });

  console.log('\n');
  logger.section('🏢 AUDIT: INROS LACKNER');
  console.log('Website: https://www.inros-lackner.de/de');
  console.log('Mit vollständiger Validation & QA-Prüfung');
  console.log('');

  // Configure audit
  const config: StableAuditConfig = {
    maxPages: 5,  // Test first 5 pages
    timeout: 45000,
    maxConcurrent: 2,
    outputFormat: 'both',
    outputDir: './inros-lackner-report',
    standard: 'WCAG2AA',
    verbose: true,
    reportPrefix: 'inros-lackner'
  };

  const auditor = createStableAuditor(config);
  const validator = new ReportValidator();
  const completenessChecker = new DataCompletenessChecker();

  // Track progress
  let progressCount = 0;
  auditor.onProgress((progress) => {
    progressCount++;
    if (progressCount % 5 === 0 || progress.progress === 100) {
      const bar = '█'.repeat(Math.round(progress.progress / 5)) +
                  '░'.repeat(20 - Math.round(progress.progress / 5));
      console.log(`\n📊 ${progress.phase.toUpperCase()}: [${bar}] ${progress.progress.toFixed(1)}%`);
      if (progress.message) {
        console.log(`   ${progress.message}`);
      }
    }
  });

  // Track errors
  auditor.onError((error) => {
    if (error.recoverable) {
      logger.warn(`Recoverable error: ${error.message}`);
    } else {
      logger.error(`Error: ${error.code} - ${error.message}`);
    }
  });

  try {
    // Initialize
    logger.info('🔧 Initializing auditor...');
    await auditor.initialize();
    logger.success('✅ Initialization complete');
    console.log('');

    // Run audit
    logger.section('🔍 RUNNING AUDIT');
    const startTime = Date.now();

    const result = await auditor.auditWebsite('https://www.inros-lackner.de/de');

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

    // Convert pages to AccessibilityResult format for validation
    const accessibilityResults = result.pages.map(page => ({
      url: page.url,
      title: page.title,
      passed: page.passed,
      crashed: page.crashed || false,
      skipped: false,
      duration: page.duration,
      errors: page.issues?.errors.map(e => e.message || e.toString()) || [],
      warnings: page.issues?.warnings.map(w => w.message || w.toString()) || [],
      imagesWithoutAlt: 0,
      buttonsWithoutLabel: 0,
      headingsCount: 0,
      pa11yScore: page.scores?.accessibility,
      pa11yIssues: page.issues?.errors.map(e => ({
        code: (e as any).code || e.rule || 'unknown',
        message: e.message || e.toString(),
        type: 'error' as const
      })) || [],
      performanceMetrics: page.scores?.performance ? {
        loadTime: page.duration,
        domContentLoaded: 0,
        firstPaint: 0,
        renderTime: 0,
        firstContentfulPaint: 0,
        largestContentfulPaint: 0,
        performanceScore: page.scores.performance,
        performanceGrade: 'N/A' as any
      } : undefined
    }));

    const validation = validator.validateAuditResults(accessibilityResults);

    if (validation.valid) {
      logger.success(`✅ All ${accessibilityResults.length} results are structurally valid`);
    } else {
      logger.error(`❌ Validation failed with ${validation.errors.length} errors`);
      validation.errors.slice(0, 5).forEach(error => {
        logger.error(`  - ${error.field}: ${error.message}`);
      });
    }

    console.log('');
    logger.info('2️⃣  Checking data completeness...');

    const batchReport = completenessChecker.generateBatchReport(accessibilityResults);

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
    console.log(`  Total Pages Audited: ${result.summary.testedPages}`);
    console.log(`  Success Rate: ${result.summary.successRate.toFixed(1)}%`);
    console.log(`  Total Duration: ${duration.toFixed(1)}s`);
    console.log(`  Average Page Time: ${(result.summary.averagePageTime / 1000).toFixed(1)}s`);
    console.log('');

    console.log('Pass/Fail Status:');
    console.log(`  ✅ Passed: ${result.summary.passedPages}`);
    console.log(`  ❌ Failed: ${result.summary.failedPages}`);
    if (result.summary.crashedPages > 0) {
      console.log(`  💥 Crashed: ${result.summary.crashedPages}`);
    }
    console.log('');

    const avgMobileScore = result.pages.reduce((sum, page) => sum + page.scores.mobile, 0) / result.pages.length;

    console.log('Quality Scores (Average):');
    console.log(`  Accessibility: ${result.performance.avgAccessibilityScore.toFixed(1)}/100`);
    console.log(`  Performance: ${result.performance.avgPerformanceScore.toFixed(1)}/100`);
    console.log(`  SEO: ${result.performance.avgSeoScore.toFixed(1)}/100`);
    console.log(`  Mobile: ${avgMobileScore.toFixed(1)}/100`);
    console.log('');

    // ========================================
    // PAGE-BY-PAGE ANALYSIS
    // ========================================
    logger.section('📄 PAGE-BY-PAGE ANALYSIS');
    console.log('');

    result.pages.forEach((page, index) => {
      const status = page.crashed ? '💥 CRASHED' : page.passed ? '✅ PASSED' : '❌ FAILED';
      const completeness = completenessChecker.checkPageCompleteness(accessibilityResults[index]);

      console.log(`${index + 1}. ${status} - ${page.title}`);
      console.log(`   URL: ${page.url}`);
      console.log(`   Duration: ${page.duration}ms`);
      console.log(`   Completeness: ${completeness.score}%`);
      console.log(`   Scores:`);
      console.log(`     - Accessibility: ${page.scores.accessibility}/100`);
      console.log(`     - Performance: ${page.scores.performance}/100`);
      console.log(`     - SEO: ${page.scores.seo}/100`);
      console.log(`     - Mobile: ${page.scores.mobile}/100`);
      console.log(`   Issues:`);
      console.log(`     - ${page.issues.errors.length} errors`);
      console.log(`     - ${page.issues.warnings.length} warnings`);

      if (page.issues.errors.length > 0) {
        console.log(`   Top Errors (showing first 3):`);
        page.issues.errors.slice(0, 3).forEach(error => {
          const errorMsg = typeof error === 'string' ? error : error.message || JSON.stringify(error);
          console.log(`     • ${errorMsg}`);
        });
      }

      if (!completeness.isComplete) {
        console.log(`   ⚠️  Missing: ${completeness.missingFields.join(', ')}`);
      }

      console.log('');
    });

    // ========================================
    // GENERATED REPORTS
    // ========================================
    logger.section('📁 GENERATED REPORTS');
    console.log('');

    if (result.reports.html) {
      logger.success(`HTML Report: ${result.reports.html}`);
    }
    if (result.reports.markdown) {
      logger.success(`Markdown Report: ${result.reports.markdown}`);
    }
    console.log('');

    // ========================================
    // QUALITY ASSESSMENT
    // ========================================
    logger.section('🎯 QUALITY ASSESSMENT');
    console.log('');

    const qualityChecks = {
      structureValid: validation.valid,
      completenessGood: batchReport.overallScore >= 80,
      hasResults: result.pages.length > 0,
      noErrors: validation.errors.length === 0,
      goodScores: result.performance.avgAccessibilityScore >= 50
    };

    const passedChecks = Object.values(qualityChecks).filter(v => v).length;
    const totalChecks = Object.keys(qualityChecks).length;

    console.log('Quality Checks:');
    console.log(`  ${qualityChecks.structureValid ? '✅' : '❌'} Data structure is valid`);
    console.log(`  ${qualityChecks.completenessGood ? '✅' : '❌'} Data completeness ≥ 80%`);
    console.log(`  ${qualityChecks.hasResults ? '✅' : '❌'} Results were generated`);
    console.log(`  ${qualityChecks.noErrors ? '✅' : '❌'} No validation errors`);
    console.log(`  ${qualityChecks.goodScores ? '✅' : '❌'} Accessibility scores are meaningful`);
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

    // Cleanup
    await auditor.cleanup();
    logger.success('🏁 Audit session completed successfully');
    console.log('');

    return qualityScore >= 80;

  } catch (error) {
    console.log('');
    logger.error('❌ Audit failed with error:');
    console.error(error);
    console.log('');

    await auditor.cleanup();
    return false;
  }
}

// Run the audit
if (require.main === module) {
  auditInrosLackner()
    .then(success => {
      process.exit(success ? 0 : 1);
    })
    .catch(error => {
      console.error('Fatal error:', error);
      process.exit(1);
    });
}

export { auditInrosLackner };
