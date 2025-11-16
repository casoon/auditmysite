#!/usr/bin/env node

/**
 * 🎯 Example: Using the Stable Audit Interface
 *
 * This example demonstrates how to use the StableAuditor interface
 * for reliable, production-ready website auditing.
 *
 * Run with: npx ts-node examples/stable-interface-example.ts
 */

import {
  createStableAuditor,
  StableAuditConfig,
  AuditResult,
  AuditProgress,
  AuditError,
} from '../src/interfaces/stable-audit-interface';

async function main() {
  console.log('🚀 Starting Website Audit with Stable Interface');
  console.log('================================================\n');

  // Configuration for the audit
  const config: StableAuditConfig = {
    maxPages: 3,
    timeout: 60000,
    maxConcurrent: 2,
    outputFormat: 'both',
    outputDir: './audit-reports',
    standard: 'WCAG2AA',
    verbose: false,
    reportPrefix: 'stable-audit',
  };

  // Create auditor instance
  const auditor = createStableAuditor(config);

  // Setup progress monitoring
  auditor.onProgress((progress: AuditProgress) => {
    const progressBar =
      '█'.repeat(Math.round(progress.progress / 5)) +
      '░'.repeat(20 - Math.round(progress.progress / 5));

    console.log(
      `📊 ${progress.phase.toUpperCase()}: [${progressBar}] ${progress.progress.toFixed(1)}% (${progress.completed}/${progress.total})`
    );

    if (progress.message) {
      console.log(`   ${progress.message}`);
    }
  });

  // Setup error monitoring
  auditor.onError((error: AuditError) => {
    if (error.recoverable) {
      console.log(`⚠️  Warning: ${error.message}`);
    } else {
      console.log(`🚨 Error: ${error.code} - ${error.message}`);
    }
  });

  try {
    console.log('🏥 Health Check: Initial Status');
    const initialHealth = auditor.getHealthStatus();
    console.log(`   Status: ${initialHealth.status}`);
    console.log(`   Initialized: ${initialHealth.details.initialized}`);
    console.log('');

    console.log('🚀 Initializing Auditor...');
    await auditor.initialize();

    const healthAfterInit = auditor.getHealthStatus();
    console.log(`✅ Initialization complete - Status: ${healthAfterInit.status}`);
    console.log(`   Browser Pool Size: ${healthAfterInit.details.browserPoolSize}`);
    console.log(
      `   Memory Usage: ${Math.round(healthAfterInit.details.memoryUsage.heapUsed / 1024 / 1024)}MB`
    );
    console.log('');

    console.log('🌐 Starting Website Audit...');
    const startTime = Date.now();

    const result: AuditResult = await auditor.auditWebsite('https://example.com/sitemap.xml');

    const duration = Date.now() - startTime;

    console.log('\n✅ Audit Completed Successfully!');
    console.log('================================\n');

    // Display results summary
    console.log('📊 AUDIT SUMMARY:');
    console.log(`   Domain: example.com`);
    console.log(`   Total Pages: ${result.summary.totalPages}`);
    console.log(`   Pages Tested: ${result.summary.testedPages}`);
    console.log(`   Pages Passed: ${result.summary.passedPages}`);
    console.log(`   Pages Failed: ${result.summary.failedPages}`);
    console.log(`   Pages Crashed: ${result.summary.crashedPages}`);
    console.log(`   Success Rate: ${result.summary.successRate.toFixed(1)}%`);
    console.log(`   Total Duration: ${(duration / 1000).toFixed(1)}s`);
    console.log(`   Average Page Time: ${(result.summary.averagePageTime / 1000).toFixed(1)}s`);
    console.log('');

    // Display performance metrics
    console.log('⚡ PERFORMANCE METRICS:');
    console.log(`   Avg Load Time: ${result.performance.avgLoadTime.toFixed(1)}ms`);
    console.log(
      `   Avg Accessibility Score: ${result.performance.avgAccessibilityScore.toFixed(1)}/100`
    );
    console.log(
      `   Avg Performance Score: ${result.performance.avgPerformanceScore.toFixed(1)}/100`
    );
    console.log(`   Avg SEO Score: ${result.performance.avgSeoScore.toFixed(1)}/100`);
    console.log('');

    // Display page results
    console.log('📄 PAGE RESULTS:');
    result.pages.forEach((page, index) => {
      const status = page.crashed ? '💥' : page.passed ? '✅' : '❌';
      console.log(`   ${index + 1}. ${status} ${page.title}`);
      console.log(`      URL: ${page.url}`);
      console.log(
        `      Scores: A11y:${page.scores.accessibility} Perf:${page.scores.performance} SEO:${page.scores.seo} Mobile:${page.scores.mobile}`
      );
      console.log(
        `      Issues: ${page.issues.errors.length} errors, ${page.issues.warnings.length} warnings`
      );
      console.log(`      Duration: ${page.duration}ms`);
    });
    console.log('');

    // Display generated reports
    console.log('📁 GENERATED REPORTS:');
    if (result.reports.html) {
      console.log(`   HTML: ${result.reports.html}`);
    }
    if (result.reports.markdown) {
      console.log(`   Markdown: ${result.reports.markdown}`);
    }
    console.log('');

    // Display system info
    console.log('🔧 SYSTEM INFO:');
    console.log(`   Node Version: ${result.metadata.systemInfo.nodeVersion}`);
    console.log(
      `   Memory Usage: ${Math.round(result.metadata.systemInfo.memoryUsage.heapUsed / 1024 / 1024)}MB`
    );
    console.log(`   Audit Date: ${new Date(result.metadata.auditDate).toLocaleString()}`);
    console.log('');

    // Final health check
    const finalHealth = auditor.getHealthStatus();
    console.log('🏥 Final Health Check:');
    console.log(`   Status: ${finalHealth.status}`);
    console.log(
      `   Memory: ${Math.round(finalHealth.details.memoryUsage.heapUsed / 1024 / 1024)}MB`
    );
    console.log(`   Uptime: ${finalHealth.details.uptime.toFixed(1)}s`);
  } catch (error) {
    console.error('🚨 Audit Failed:', error);

    // Show health status in case of failure
    const errorHealth = auditor.getHealthStatus();
    console.log(`\n🏥 Health Status after error: ${errorHealth.status}`);

    process.exit(1);
  } finally {
    console.log('\n🧹 Cleaning up resources...');
    await auditor.cleanup();
    console.log('✅ Cleanup complete');
  }
}

// Handle process signals for graceful shutdown
process.on('SIGINT', async () => {
  console.log('\n⚠️  Received SIGINT, shutting down gracefully...');
  process.exit(0);
});

process.on('SIGTERM', async () => {
  console.log('\n⚠️  Received SIGTERM, shutting down gracefully...');
  process.exit(0);
});

// Run the example
if (require.main === module) {
  main().catch((error) => {
    console.error('💥 Example failed:', error);
    process.exit(1);
  });
}

export { main };
