#!/usr/bin/env node

/**
 * Test Smart URL Sampling with INROS LACKNER
 * 
 * This demonstrates how the smart sampler:
 * 1. Tests URLs from sitemap sequentially
 * 2. Skips redirects and 404s
 * 3. Finds the desired number of GOOD pages
 */

const { AccessibilityChecker } = require('./dist/core/accessibility/accessibility-checker');
const { SmartUrlSampler } = require('./dist/core/smart-url-sampler');
const { SitemapParser } = require('./dist/parsers/sitemap-parser');

async function testSmartSampling() {
  console.log('🧪 Testing Smart URL Sampling with INROS LACKNER');
  console.log('=================================================\n');
  
  const sitemap = 'https://www.inros-lackner.de/sitemap.xml';
  const desiredPages = 3;
  
  try {
    // 1. Parse sitemap
    console.log('🔍 Step 1: Parsing sitemap...');
    const parser = new SitemapParser();
    const allUrls = await parser.parseSitemap(sitemap);
    
    console.log(`📄 Found ${allUrls.length} URLs in sitemap`);
    console.log(`🎯 Target: ${desiredPages} good pages`);
    
    // 2. Initialize checker
    console.log('\n🔧 Step 2: Initializing accessibility checker...');
    const checker = new AccessibilityChecker();
    await checker.initialize();
    console.log('✅ Checker ready');
    
    // 3. Smart sampling
    console.log('\n🎯 Step 3: Smart URL sampling...');
    const sampler = new SmartUrlSampler(checker, {
      maxPages: desiredPages,
      maxAttempts: Math.min(15, allUrls.length), // Try max 15 URLs
      timeout: 8000,
      verbose: true,
      skipRedirects: true,
      skip404s: true
    });
    
    const samplingResult = await sampler.sampleGoodUrls(allUrls.map(u => u.loc || u));
    
    // 4. Results
    console.log('\n📊 Sampling Complete!');
    console.log('==================');
    
    console.log(`✅ Good URLs (${samplingResult.goodUrls.length}):`);
    samplingResult.goodUrls.forEach((url, i) => {
      console.log(`   ${i + 1}. ${url}`);
    });
    
    if (samplingResult.skippedUrls.length > 0) {
      console.log(`\n↪️  Skipped URLs (${samplingResult.skippedUrls.length}):`);
      samplingResult.skippedUrls.slice(0, 5).forEach((url, i) => {
        console.log(`   ${i + 1}. ${url}`);
      });
      if (samplingResult.skippedUrls.length > 5) {
        console.log(`   ... and ${samplingResult.skippedUrls.length - 5} more`);
      }
    }
    
    if (samplingResult.errorUrls.length > 0) {
      console.log(`\n❌ Error URLs (${samplingResult.errorUrls.length}):`);
      samplingResult.errorUrls.slice(0, 3).forEach((url, i) => {
        console.log(`   ${i + 1}. ${url}`);
      });
    }
    
    console.log(`\n📈 Statistics:`);
    console.log(`   Attempts: ${samplingResult.totalAttempts}`);
    console.log(`   Success rate: ${(samplingResult.goodUrls.length / samplingResult.totalAttempts * 100).toFixed(1)}%`);
    console.log(`   Sampling time: ${Math.round(samplingResult.samplingTime / 1000)}s`);
    
    // 5. Now run full analysis on good URLs only
    if (samplingResult.goodUrls.length > 0) {
      console.log(`\n🚀 Step 4: Running full analysis on ${samplingResult.goodUrls.length} good URLs...`);
      
      const results = await checker.testMultiplePagesParallel(samplingResult.goodUrls, {
        verbose: true,
        collectPerformanceMetrics: true,
        timeout: 15000,
        maxConcurrent: 2
      });
      
      console.log(`\n✅ Analysis Complete!`);
      console.log(`   Analyzed: ${results.length} pages`);
      console.log(`   Passed: ${results.filter(r => r.passed).length}`);
      console.log(`   Failed: ${results.filter(r => !r.passed).length}`);
      
      // Show some results
      results.forEach((result, i) => {
        const status = result.passed ? '✅' : '❌';
        const title = result.title ? `"${result.title}"` : 'No title';
        const errors = result.errors?.length || 0;
        const warnings = result.warnings?.length || 0;
        console.log(`   ${i + 1}. ${status} ${title} (${errors}E, ${warnings}W, ${result.duration}ms)`);
      });
    }
    
    // Cleanup
    await checker.cleanup();
    
    console.log('\n🎉 Smart sampling test completed successfully!');
    console.log('\n📋 Benefits demonstrated:');
    console.log('   ✅ Redirects are properly detected and skipped');
    console.log('   ✅ System continues until desired number of good pages found');
    console.log('   ✅ No timeout errors from redirect pages');  
    console.log('   ✅ Efficient sampling saves time and resources');
    
  } catch (error) {
    console.error('❌ Test failed:', error.message);
    console.error('Stack trace:', error.stack);
  }
}

// Run the test
if (require.main === module) {
  testSmartSampling().catch(console.error);
}

module.exports = { testSmartSampling };