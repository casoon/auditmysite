#!/usr/bin/env node

/**
 * 🧪 Test New Analyzers - Direct test of Security Headers and Structured Data
 */

const { AccessibilityChecker } = require('./dist/core/accessibility');

async function testNewAnalyzers() {
  console.log('🧪 Testing new Security Headers and Structured Data analyzers...\n');
  
  const checker = new AccessibilityChecker({
    enableComprehensiveAnalysis: true,
    qualityAnalysisOptions: {
      verbose: true,
      includeResourceAnalysis: true,
      includeTechnicalSEO: true,
      includeSocialAnalysis: true,
      analysisTimeout: 30000
    }
  });
  
  await checker.initialize();
  console.log('✅ AccessibilityChecker initialized with comprehensive analysis\n');
  
  // Test URLs that should have different security headers and structured data
  const testUrls = [
    'https://example.com',
    'https://httpbin.org/html'
  ];
  
  for (const url of testUrls) {
    try {
      console.log(`🔍 Testing: ${url}`);
      console.log('─'.repeat(60));
      
      const result = await checker.testPage(url, {
        verbose: true,
        collectPerformanceMetrics: true,
        captureScreenshots: false, // Disable screenshots for faster testing
        timeout: 20000
      });
      
      console.log(`📊 Results for ${url}:`);
      console.log(`   Title: ${result.title}`);
      console.log(`   Passed: ${result.passed ? '✅' : '❌'}`);
      console.log(`   Duration: ${result.duration}ms`);
      console.log(`   Errors: ${result.errors?.length || 0}`);
      console.log(`   Warnings: ${result.warnings?.length || 0}`);
      
      // Check if new analyzers ran
      const hasSecurityHeaders = result.securityHeaders;
      const hasStructuredData = result.structuredData;
      const hasPerformance = result.enhancedPerformance;
      const hasSEO = result.enhancedSEO;
      const hasContentWeight = result.contentWeight;
      const hasMobileFriendliness = result.mobileFriendliness;
      
      console.log(`\n🔍 Analysis Results:`);
      console.log(`   🔐 Security Headers: ${hasSecurityHeaders ? '✅ Present' : '❌ Missing'}`);
      console.log(`   📊 Structured Data: ${hasStructuredData ? '✅ Present' : '❌ Missing'}`);
      console.log(`   ⚡ Performance: ${hasPerformance ? '✅ Present' : '❌ Missing'}`);
      console.log(`   🔍 SEO: ${hasSEO ? '✅ Present' : '❌ Missing'}`);
      console.log(`   📏 Content Weight: ${hasContentWeight ? '✅ Present' : '❌ Missing'}`);
      console.log(`   📱 Mobile Friendliness: ${hasMobileFriendliness ? '✅ Present' : '❌ Missing'}`);
      
      // Show detailed results for new analyzers if available
      if (hasSecurityHeaders) {
        console.log(`\n🔐 Security Headers Details:`);
        console.log(`   Overall Score: ${hasSecurityHeaders.overallScore}/100 (${hasSecurityHeaders.grade})`);
        console.log(`   HTTPS Enabled: ${hasSecurityHeaders.https?.enabled ? '✅' : '❌'}`);
        console.log(`   CSP Present: ${hasSecurityHeaders.headers?.csp?.present ? '✅' : '❌'}`);
        console.log(`   HSTS Present: ${hasSecurityHeaders.headers?.hsts?.present ? '✅' : '❌'}`);
        console.log(`   X-Frame-Options: ${hasSecurityHeaders.headers?.xFrameOptions?.present ? '✅' : '❌'}`);
        console.log(`   Vulnerabilities: ${JSON.stringify(hasSecurityHeaders.vulnerabilities, null, 2)}`);
      }
      
      if (hasStructuredData) {
        console.log(`\n📊 Structured Data Details:`);
        console.log(`   Overall Score: ${hasStructuredData.overallScore}/100 (${hasStructuredData.grade})`);
        console.log(`   Total Items: ${hasStructuredData.summary?.totalItems || 0}`);
        console.log(`   Valid Items: ${hasStructuredData.summary?.validItems || 0}`);
        console.log(`   JSON-LD Count: ${hasStructuredData.summary?.jsonLdCount || 0}`);
        console.log(`   Rich Snippets Eligible: ${hasStructuredData.richSnippets?.eligible ? '✅' : '❌'}`);
        console.log(`   Knowledge Graph Score: ${hasStructuredData.knowledgeGraph?.readinessScore || 0}/100`);
      }
      
    } catch (error) {
      console.error(`❌ Failed to test ${url}: ${error.message}`);
    }
    
    console.log('\n' + '='.repeat(80) + '\n');
  }
  
  await checker.cleanup();
  console.log('✅ Test completed - all resources cleaned up');
}

testNewAnalyzers().catch(error => {
  console.error('❌ Test failed:', error);
  process.exit(1);
});