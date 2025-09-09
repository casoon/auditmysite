#!/usr/bin/env node

/**
 * Test script for CoreAuditPipeline v2.0
 * 
 * This script demonstrates the new simplified pipeline:
 * 1. Clean JSON export
 * 2. Simplified options
 * 3. Optional HTML generation
 */

const { CoreAuditPipeline } = require('./dist/core/pipeline');

async function testCorePipeline() {
  console.log('🚀 Testing CoreAuditPipeline v2.0...\n');
  
  const pipeline = new CoreAuditPipeline();
  
  // Test with a small sitemap
  const options = {
    sitemapUrl: 'https://example.com/sitemap.xml',
    maxPages: 2,
    outputDir: './test-reports',
    useEnhancedAnalysis: false, // Start with standard mode
    generateHTML: true, // Optional HTML report
    collectPerformanceMetrics: true,
    includeWarnings: false
  };
  
  try {
    const result = await pipeline.run(options);
    
    console.log('\n✅ Core Pipeline Test Results:');
    console.log('═══════════════════════════════════');
    console.log(`📊 Summary:`);
    console.log(`   Total pages: ${result.summary.totalPages}`);
    console.log(`   Passed: ${result.summary.passedPages}`);
    console.log(`   Failed: ${result.summary.failedPages}`);
    console.log(`   Duration: ${Math.round(result.metadata.duration / 1000)}s`);
    console.log(`\n📄 Exports:`);
    console.log(`   JSON: ./test-reports/audit-result-${new Date().toISOString().split('T')[0]}.json`);
    console.log(`   HTML: ./test-reports/audit-report-${new Date().toISOString().split('T')[0]}.html`);
    console.log(`\n🔍 Issues found: ${result.issues.length}`);
    
    if (result.issues.length > 0) {
      console.log(`\n📝 Sample Issues:`);
      result.issues.slice(0, 3).forEach((issue, i) => {
        console.log(`   ${i + 1}. [${issue.severity.toUpperCase()}] ${issue.message}`);
      });
    }
    
    console.log('\n🎯 CoreAuditPipeline v2.0 test completed successfully!');
    return true;
    
  } catch (error) {
    console.error('❌ CoreAuditPipeline test failed:', error.message);
    if (error.stack) {
      console.error(error.stack);
    }
    return false;
  }
}

// Run the test
if (require.main === module) {
  testCorePipeline()
    .then(success => {
      process.exit(success ? 0 : 1);
    })
    .catch(error => {
      console.error('Fatal error:', error);
      process.exit(1);
    });
}

module.exports = { testCorePipeline };
