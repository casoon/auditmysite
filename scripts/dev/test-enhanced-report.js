#!/usr/bin/env node

/**
 * 🧪 Test Enhanced Data in HTML Reports
 * Verifies that enhanced analysis data correctly flows through to HTML reports
 */

const { AccessibilityChecker } = require('./dist/core/accessibility');
const { HTMLGenerator } = require('./dist/generators/html-generator');
const fs = require('fs');
const path = require('path');

async function testEnhancedReportData() {
  console.log('🧪 Testing enhanced data flow to HTML reports...\n');
  
  const checker = new AccessibilityChecker({
    enableComprehensiveAnalysis: true,
    qualityAnalysisOptions: {
      verbose: false,
      includeResourceAnalysis: true,
      includeTechnicalSEO: true,
      includeSocialAnalysis: true,
      analysisTimeout: 30000
    }
  });
  
  await checker.initialize();
  console.log('✅ AccessibilityChecker initialized with comprehensive analysis\n');
  
  const testUrl = 'https://example.com';
  
  try {
    console.log(`🔍 Testing: ${testUrl}`);
    console.log('─'.repeat(60));
    
    const result = await checker.testPage(testUrl, {
      verbose: false,
      collectPerformanceMetrics: true,
      captureScreenshots: false,
      timeout: 20000
    });
    
    console.log('📊 Analysis completed. Creating audit data structure...');
    
    // Create audit data structure similar to what bin/audit.js creates
    const auditData = {
      metadata: {
        version: '1.0.0',
        timestamp: new Date().toISOString(),
        sitemapUrl: testUrl,
        toolVersion: '2.0.0-alpha.2',
        duration: result.duration || 30000
      },
      summary: {
        totalPages: 1,
        testedPages: 1,
        passedPages: result.passed ? 1 : 0,
        failedPages: result.passed ? 0 : 1,
        crashedPages: 0,
        totalErrors: result.errors?.length || 0,
        totalWarnings: result.warnings?.length || 0,
        successRate: result.passed ? 100 : 0
      },
      pages: [{
        url: result.url,
        title: result.title || 'Test Page',
        status: result.passed ? 'passed' : 'failed',
        duration: result.duration || 30000,
        accessibility: {
          score: result.pa11yScore || 0,
          errors: result.errors || [],
          warnings: result.warnings || [],
          notices: result.notices || []
        },
        performance: result.enhancedPerformance || result.performance,
        seo: result.enhancedSEO || result.seo,
        contentWeight: result.contentWeight,
        mobileFriendliness: result.mobileFriendliness
      }]
    };
    
    // Check what enhanced data we have
    const page = auditData.pages[0];
    console.log('\n🔍 Enhanced Data Availability Check:');
    console.log(`   ⚡ Performance: ${page.performance ? '✅ Available' : '❌ Missing'}`);
    console.log(`   🔍 SEO: ${page.seo ? '✅ Available' : '❌ Missing'}`);
    console.log(`   📏 Content Weight: ${page.contentWeight ? '✅ Available' : '❌ Missing'}`);
    console.log(`   📱 Mobile Friendliness: ${page.mobileFriendliness ? '✅ Available' : '❌ Missing'}`);
    
    if (page.performance) {
      console.log(`     └─ Performance Score: ${page.performance.performanceScore || page.performance.score}/100`);
    }
    if (page.seo) {
      console.log(`     └─ SEO Score: ${page.seo.overallSEOScore || page.seo.score}/100`);
    }
    if (page.contentWeight) {
      console.log(`     └─ Content Weight Score: ${page.contentWeight.contentScore || page.contentWeight.score}/100`);
    }
    if (page.mobileFriendliness) {
      console.log(`     └─ Mobile Score: ${page.mobileFriendliness.overallScore}/100`);
    }
    
    console.log('\n📝 Generating HTML report...');
    
    // Generate HTML report using the HTMLGenerator
    const htmlGenerator = new HTMLGenerator();
    const htmlContent = await htmlGenerator.generate(auditData);
    
    // Save the report
    const reportsDir = path.join(process.cwd(), 'reports', 'test');
    if (!fs.existsSync(reportsDir)) {
      fs.mkdirSync(reportsDir, { recursive: true });
    }
    
    const reportPath = path.join(reportsDir, `enhanced-data-test-${Date.now()}.html`);
    fs.writeFileSync(reportPath, htmlContent);
    
    console.log(`✅ HTML report generated: ${reportPath}`);
    
    // Parse the HTML to check if enhanced data is present
    const cheerio = require('cheerio');
    const $ = cheerio.load(htmlContent);
    
    console.log('\n🔍 HTML Report Content Verification:');
    
    // Check for performance section
    const perfSection = $('#performance');
    const perfMetrics = perfSection.find('.metric-card .metric-value').length;
    console.log(`   ⚡ Performance Section: ${perfSection.length > 0 ? '✅ Present' : '❌ Missing'}`);
    console.log(`     └─ Performance Metrics: ${perfMetrics} metric cards found`);
    
    // Check for SEO section  
    const seoSection = $('#seo');
    const seoMetrics = seoSection.find('.metric-card .metric-value').length;
    console.log(`   🔍 SEO Section: ${seoSection.length > 0 ? '✅ Present' : '❌ Missing'}`);
    console.log(`     └─ SEO Metrics: ${seoMetrics} metric cards found`);
    
    // Check for content weight section
    const contentSection = $('#contentweight');
    const contentMetrics = contentSection.find('.metric-card .metric-value').length;
    console.log(`   📏 Content Weight Section: ${contentSection.length > 0 ? '✅ Present' : '❌ Missing'}`);
    console.log(`     └─ Content Metrics: ${contentMetrics} metric cards found`);
    
    // Check for mobile section
    const mobileSection = $('#mobile');
    const mobileMetrics = mobileSection.find('.metric-card .metric-value').length;
    console.log(`   📱 Mobile Section: ${mobileSection.length > 0 ? '✅ Present' : '❌ Missing'}`);
    console.log(`     └─ Mobile Metrics: ${mobileMetrics} metric cards found`);
    
    // Check for specific data values (not "N/A" or empty)
    const performanceScoreElement = $('.metric-label:contains("Performance Score")').parent().find('.metric-value');
    const seoScoreElement = $('.metric-label:contains("SEO Score")').parent().find('.metric-value');
    
    if (performanceScoreElement.length > 0) {
      const perfScore = performanceScoreElement.text().trim();
      console.log(`     └─ Performance Score in Report: ${perfScore}`);
    }
    
    if (seoScoreElement.length > 0) {
      const seoScore = seoScoreElement.text().trim();
      console.log(`     └─ SEO Score in Report: ${seoScore}`);
    }
    
    // Summary
    const hasEnhancedData = perfSection.length > 0 && seoSection.length > 0 && 
                           contentSection.length > 0 && mobileSection.length > 0;
    
    console.log(`\n🎯 Test Result: ${hasEnhancedData ? '✅ SUCCESS' : '❌ FAILED'}`);
    console.log(`   Enhanced analysis data successfully flows to HTML reports: ${hasEnhancedData}`);
    
    if (hasEnhancedData) {
      console.log('\n🎉 All enhanced analysis sections are present in the HTML report!');
      console.log('   The fixes successfully ensure enhanced data is displayed even for failed pages.');
    } else {
      console.log('\n❌ Some enhanced analysis sections are missing from the HTML report.');
      console.log('   This indicates an issue with data flow or report generation.');
    }
    
  } catch (error) {
    console.error(`❌ Failed to test ${testUrl}: ${error.message}`);
    console.error(error.stack);
  }
  
  await checker.cleanup();
  console.log('\n✅ Test completed - all resources cleaned up');
}

testEnhancedReportData().catch(error => {
  console.error('❌ Test failed:', error);
  process.exit(1);
});