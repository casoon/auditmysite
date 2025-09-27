#!/usr/bin/env node

/**
 * Quick test to isolate the stack overflow issue
 */

async function testCore() {
  try {
    console.log('🧪 Testing Core Pipeline directly...');
    
    const { CoreAuditPipeline } = require('./dist/core/pipeline/core-audit-pipeline');
    
    const pipeline = new CoreAuditPipeline();
    
    const options = {
      sitemapUrl: 'https://inros-lackner.de',
      maxPages: 1,
      useEnhancedAnalysis: false, // Start with basic analysis
      pa11yStandard: 'WCAG2AA',
      captureScreenshots: false,
      includeWarnings: true,
      maxConcurrent: 1
    };
    
    console.log('📋 Testing with options:', options);
    
    const result = await pipeline.audit(options);
    
    console.log('✅ Core Pipeline Test Results:');
    console.log(`- Pages tested: ${result.pages.length}`);
    console.log(`- Total errors: ${result.summary.totalErrors}`);
    console.log(`- Total warnings: ${result.summary.totalWarnings}`);
    
    if (result.pages.length > 0) {
      console.log('✅ First page result:');
      const page = result.pages[0];
      console.log(`  - URL: ${page.url}`);
      console.log(`  - Status: ${page.status}`);
      console.log(`  - Errors: ${page.accessibility.errors.length}`);
      console.log(`  - Warnings: ${page.accessibility.warnings.length}`);
    }
    
  } catch (error) {
    console.error('❌ Core Pipeline Test failed:', error);
    console.error('Stack trace:', error.stack);
  }
}

testCore().then(() => {
  console.log('🏁 Test completed');
  process.exit(0);
}).catch(error => {
  console.error('💥 Unhandled error:', error);
  process.exit(1);
});
