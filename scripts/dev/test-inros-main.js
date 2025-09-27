#!/usr/bin/env node

/**
 * Direct test of www.inros-lackner.de main page
 * Testing redirect handling and comprehensive analysis
 */

const { AccessibilityChecker } = require('./dist/core/accessibility/accessibility-checker');
const fs = require('fs');
const path = require('path');

async function testInrosMainPage() {
  console.log('🏢 Testing INROS LACKNER Main Page');
  console.log('==================================\n');
  
  const testUrl = 'https://www.inros-lackner.de/';
  console.log(`🔍 Target: ${testUrl}`);
  
  // Create reports directory
  const reportsDir = './reports/inros-test';
  if (!fs.existsSync(reportsDir)) {
    fs.mkdirSync(reportsDir, { recursive: true });
  }
  
  try {
    // Use accessibility checker with comprehensive analysis
    const checker = new AccessibilityChecker({
      enableComprehensiveAnalysis: true
    });
    
    console.log('\n🚀 Initializing checker...\n');
    await checker.initialize();
    
    console.log('🚀 Starting comprehensive analysis...\n');
    
    const result = await checker.testPage(testUrl, {
      timeout: 15000, // Reduced timeout
      verbose: true,
      collectPerformanceMetrics: true,
      testKeyboardNavigation: true,
      testColorContrast: true,
      testFocusManagement: true,
      captureScreenshots: true
    });
    
    console.log('\n📊 Analysis Results:');
    console.log('==================');
    
    if (result) {
      const mainResult = result;
      
      console.log(`✅ Page analyzed: ${mainResult.url}`);
      console.log(`🔗 Page title: "${mainResult.title}"`);
      console.log(`📊 Status: ${mainResult.passed ? 'PASSED' : 'FAILED'}`);
      console.log(`⚠️  Warnings: ${mainResult.warnings?.length || 0}`);
      console.log(`❌ Errors: ${mainResult.errors?.length || 0}`);
      console.log(`⏱️  Duration: ${mainResult.duration}ms`);
      
      // Check for redirect information
      if (mainResult.redirectInfo) {
        console.log('\n🔀 Redirect Details:');
        console.log(`   📍 Type: ${mainResult.redirectInfo.type}`);
        console.log(`   🌐 Original: ${mainResult.redirectInfo.originalUrl}`);
        console.log(`   🎯 Final: ${mainResult.redirectInfo.finalUrl}`);
        if (mainResult.redirectInfo.status) {
          console.log(`   📊 HTTP Status: ${mainResult.redirectInfo.status}`);
        }
      } else {
        console.log('\n✅ No redirects detected');
      }
      
      // Show performance data if available
      if (mainResult.performanceMetrics) {
        console.log('\n⚡ Performance Metrics:');
        const perf = mainResult.performanceMetrics;
        console.log(`   🏆 Score: ${perf.performanceScore || 'N/A'} (${perf.performanceGrade || 'N/A'})`);
        console.log(`   🎨 FCP: ${perf.firstContentfulPaint}ms`);
        console.log(`   🖼️  LCP: ${perf.largestContentfulPaint}ms`);
        if (perf.cumulativeLayoutShift !== undefined) {
          console.log(`   📏 CLS: ${perf.cumulativeLayoutShift}`);
        }
      }
      
      // Show warnings
      if (mainResult.warnings && mainResult.warnings.length > 0) {
        console.log('\n⚠️  Warnings:');
        mainResult.warnings.forEach((warning, i) => {
          console.log(`   ${i + 1}. ${warning}`);
        });
      }
      
      // Show errors  
      if (mainResult.errors && mainResult.errors.length > 0) {
        console.log('\n❌ Errors:');
        mainResult.errors.forEach((error, i) => {
          console.log(`   ${i + 1}. ${error}`);
        });
      }
    } else {
      console.log('❌ No results returned from analysis');
    }
    
    console.log('\n📁 Reports saved to:', reportsDir);
    console.log('\n🎉 Test completed!');
    
    // Cleanup
    await checker.cleanup();
    
  } catch (error) {
    console.error(`❌ Test failed: ${error.message}`);
    console.error('Stack trace:', error.stack);
  }
}

// Run the test
if (require.main === module) {
  testInrosMainPage().catch(console.error);
}

module.exports = { testInrosMainPage };