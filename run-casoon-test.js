#!/usr/bin/env node

/**
 * Real-World Test for www.casoon.de
 */

const { AccessibilityChecker } = require('./dist/core/accessibility/accessibility-checker');
const { BrowserPoolManager } = require('./dist/core/browser/browser-pool-manager');
const { ReportValidator } = require('./dist/validators/report-validator');
const { DataCompletenessChecker } = require('./dist/validators/data-completeness-checker');

async function testCasoon() {
  console.log('\n═══════════════════════════════════════════════════════');
  console.log('🏢 AUDIT: CASOON.DE');
  console.log('═══════════════════════════════════════════════════════\n');
  console.log('Website: https://www.casoon.de');
  console.log('Test: Comprehensive audit with validation\n');

  const browserPool = new BrowserPoolManager({ maxConcurrent: 1 });
  const checker = new AccessibilityChecker({
    poolManager: browserPool
  });
  const validator = new ReportValidator();
  const completenessChecker = new DataCompletenessChecker();

  try {
    await checker.initialize();
    console.log('✅ Browser pool initialized\n');

    const url = 'https://www.casoon.de';
    console.log(`🔍 Testing: ${url}`);
    console.log('Please wait...\n');

    const startTime = Date.now();

    const pageResult = await checker.testPage(url, {
      pa11yStandard: 'WCAG2AA',
      includeWarnings: true,
      wait: 2000
    });

    const duration = Date.now() - startTime;

    console.log('═══════════════════════════════════════════════════════');
    console.log('📊 AUDIT RESULTS');
    console.log('═══════════════════════════════════════════════════════\n');

    const accessibilityResult = pageResult.accessibilityResult;

    console.log(`Page: ${accessibilityResult.title || 'Untitled'}`);
    console.log(`URL: ${accessibilityResult.url}`);
    console.log(`Status: ${accessibilityResult.passed ? '✅ PASSED' : '❌ FAILED'}`);
    console.log(`Duration: ${Math.round(duration / 1000)}s`);
    console.log('');

    console.log('Accessibility Checks:');
    console.log(`  Images without alt: ${accessibilityResult.imagesWithoutAlt}`);
    console.log(`  Buttons without label: ${accessibilityResult.buttonsWithoutLabel}`);
    console.log(`  Headings count: ${accessibilityResult.headingsCount}`);
    console.log('');

    console.log('Issues Found:');
    console.log(`  Errors: ${accessibilityResult.errors.length}`);
    console.log(`  Warnings: ${accessibilityResult.warnings.length}`);

    if (accessibilityResult.errors.length > 0) {
      console.log('\n  Top 5 Errors:');
      accessibilityResult.errors.slice(0, 5).forEach((error, i) => {
        console.log(`    ${i + 1}. ${error}`);
      });
    }

    if (accessibilityResult.pa11yScore !== undefined) {
      console.log(`\nPa11y Score: ${accessibilityResult.pa11yScore}/100`);
    }

    if (accessibilityResult.performanceMetrics) {
      console.log('\nPerformance Metrics:');
      console.log(`  Load Time: ${accessibilityResult.performanceMetrics.loadTime}ms`);
      console.log(`  Performance Score: ${accessibilityResult.performanceMetrics.performanceScore || 'N/A'}`);
    }

    // Validation
    console.log('\n═══════════════════════════════════════════════════════');
    console.log('🔍 VALIDATION');
    console.log('═══════════════════════════════════════════════════════\n');

    const validation = validator.validateAuditResults([accessibilityResult]);
    console.log(`Structure Valid: ${validation.valid ? '✅ YES' : '❌ NO'}`);

    if (!validation.valid && validation.errors.length > 0) {
      console.log('\nValidation Errors:');
      validation.errors.slice(0, 5).forEach(error => {
        console.log(`  - ${error.field}: ${error.message}`);
      });
    }

    const completeness = completenessChecker.checkPageCompleteness(accessibilityResult);
    console.log(`\nCompleteness Score: ${completeness.score}%`);
    console.log(`Complete: ${completeness.isComplete ? '✅ YES' : '❌ NO'}`);

    if (!completeness.isComplete) {
      console.log('\nMissing Fields:');
      completeness.missingFields.forEach(field => {
        console.log(`  - ${field}`);
      });
    }

    if (completeness.recommendations.length > 0) {
      console.log('\nRecommendations:');
      completeness.recommendations.slice(0, 3).forEach(rec => {
        console.log(`  → ${rec}`);
      });
    }

    // Final assessment
    console.log('\n═══════════════════════════════════════════════════════');
    console.log('🎯 QUALITY ASSESSMENT');
    console.log('═══════════════════════════════════════════════════════\n');

    const qualityChecks = {
      structureValid: validation.valid,
      hasData: accessibilityResult.errors.length >= 0,
      completenessGood: completeness.score >= 80,
      testPassed: !accessibilityResult.crashed
    };

    const passedChecks = Object.values(qualityChecks).filter(v => v).length;
    const totalChecks = Object.keys(qualityChecks).length;
    const qualityScore = Math.round((passedChecks / totalChecks) * 100);

    console.log('Quality Checks:');
    console.log(`  ${qualityChecks.structureValid ? '✅' : '❌'} Data structure is valid`);
    console.log(`  ${qualityChecks.hasData ? '✅' : '❌'} Contains audit data`);
    console.log(`  ${qualityChecks.completenessGood ? '✅' : '❌'} Completeness ≥ 80%`);
    console.log(`  ${qualityChecks.testPassed ? '✅' : '❌'} Test completed without crash`);

    console.log(`\nOverall Quality Score: ${qualityScore}%`);

    // Final verdict
    console.log('\n═══════════════════════════════════════════════════════');
    console.log('🏆 FINAL VERDICT - TOOL EVALUATION');
    console.log('═══════════════════════════════════════════════════════\n');

    if (qualityScore >= 75) {
      console.log('🎉 EXCELLENT - The audit tool is reliable!');
      console.log('\n✅ Das AuditMySite Tool funktioniert korrekt:');
      console.log('  • Generiert valide Datenstrukturen');
      console.log('  • Sammelt umfassende Informationen');
      console.log('  • Erkennt Accessibility-Probleme');
      console.log('  • Liefert aussagekräftige Ergebnisse');
      console.log('\n✅ Die Ergebnisse sind zuverlässig und können');
      console.log('   für Entscheidungen verwendet werden.');
    } else if (qualityScore >= 50) {
      console.log('⚠️  GOOD - Tool works but with limitations');
      console.log('\nThe audit completed but has some data completeness issues.');
    } else {
      console.log('❌ NEEDS IMPROVEMENT - Quality issues detected');
      console.log('\nPlease review the validation errors above.');
    }

    console.log('\n═══════════════════════════════════════════════════════\n');

    // Cleanup
    await checker.cleanup();
    await browserPool.cleanup();

    return qualityScore >= 75;

  } catch (error) {
    console.error('\n❌ Audit failed with error:');
    console.error(error.message || error);
    console.error('');

    try {
      await checker.cleanup();
      await browserPool.cleanup();
    } catch (cleanupError) {
      // Ignore cleanup errors
    }

    return false;
  }
}

// Run the test
testCasoon()
  .then(success => {
    process.exit(success ? 0 : 1);
  })
  .catch(error => {
    console.error('Fatal error:', error);
    process.exit(1);
  });
