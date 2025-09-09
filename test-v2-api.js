#!/usr/bin/env node

/**
 * Test script for AuditMySite v2.0 API
 * 
 * This script demonstrates the new modular API endpoints:
 * 1. GET /api/v2/sitemap/:domain (SitemapResult)
 * 2. POST /api/v2/page/accessibility (AccessibilityResult) 
 * 3. GET /api/v2/schema (Introspection)
 * 
 * Designed for Electron app integration with type-consistent responses.
 */

const { AuditAPIServer } = require('./dist/api/server');

async function testV2API() {
  console.log('🚀 Testing AuditMySite v2.0 API...\n');

  // Start API server
  const server = new AuditAPIServer({
    port: 3001,
    host: 'localhost',
    apiKeyRequired: false
  });

  try {
    await server.start();
    console.log('🌐 API Server started at http://localhost:3001');
    console.log('📚 Swagger UI available at http://localhost:3001/api-docs\n');

    // Test 1: API Schema Introspection
    console.log('📋 Test 1: API Schema Introspection');
    try {
      const response = await fetch('http://localhost:3001/api/v2/schema');
      const schemaData = await response.json();
      
      if (schemaData.success) {
        console.log('✅ Schema endpoint working');
        console.log(`   Version: ${schemaData.data.version}`);
        console.log(`   Endpoints: ${schemaData.data.endpoints.length}`);
        console.log(`   Types: ${schemaData.data.types.length} available`);
      } else {
        console.log('❌ Schema endpoint failed');
      }
    } catch (error) {
      console.log(`❌ Schema test failed: ${error.message}`);
    }

    // Test 2: Sitemap Parsing
    console.log('\n🗺️  Test 2: Sitemap Parsing');
    try {
      const response = await fetch('http://localhost:3001/api/v2/sitemap/example.com');
      const sitemapData = await response.json();
      
      if (sitemapData.success) {
        console.log('✅ Sitemap parsing working');
        console.log(`   URLs found: ${sitemapData.data.urls?.length || 0}`);
        console.log(`   Total URLs: ${sitemapData.data.totalUrls}`);
        console.log(`   Filtered: ${sitemapData.data.filteredUrls}`);
      } else {
        console.log(`⚠️  Sitemap parsing expected to fail for example.com: ${sitemapData.error}`);
      }
    } catch (error) {
      console.log(`❌ Sitemap test failed: ${error.message}`);
    }

    // Test 3: Accessibility Analysis
    console.log('\n🎯 Test 3: Accessibility Analysis');
    try {
      const response = await fetch('http://localhost:3001/api/v2/page/accessibility', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          url: 'https://example.com',
          options: { pa11yStandard: 'WCAG2AA', includeWarnings: true }
        })
      });
      
      const accessibilityData = await response.json();
      
      if (accessibilityData.success) {
        console.log('✅ Accessibility analysis working');
        console.log(`   Score: ${accessibilityData.data.score}/100`);
        console.log(`   WCAG Level: ${accessibilityData.data.wcagLevel}`);
        console.log(`   Errors: ${accessibilityData.data.errors.length}`);
        console.log(`   Warnings: ${accessibilityData.data.warnings.length}`);
      } else {
        console.log(`❌ Accessibility analysis failed: ${accessibilityData.error}`);
      }
    } catch (error) {
      console.log(`❌ Accessibility test failed: ${error.message}`);
    }

    // Test 4: Performance Analysis (should return 501 Not Implemented)
    console.log('\n⚡ Test 4: Performance Analysis (experimental)');
    try {
      const response = await fetch('http://localhost:3001/api/v2/page/performance', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ url: 'https://example.com' })
      });
      
      const performanceData = await response.json();
      
      if (response.status === 501) {
        console.log('✅ Performance endpoint correctly returns 501 (not implemented for single URLs)');
        console.log(`   Message: ${performanceData.error}`);
        console.log(`   Alternative: ${performanceData.available}`);
      } else if (performanceData.success) {
        console.log('✅ Performance analysis working (enhanced analyzer available!)');
        console.log(`   Score: ${performanceData.data.score}/100`);
        console.log(`   Grade: ${performanceData.data.grade}`);
      } else {
        console.log(`❌ Performance analysis failed: ${performanceData.error}`);
      }
    } catch (error) {
      console.log(`❌ Performance test failed: ${error.message}`);
    }

    console.log('\n🎉 v2.0 API test completed!');
    console.log('\n📊 API Summary:');
    console.log('   - Modular endpoints using shared TypeScript types');
    console.log('   - Type-consistent responses (SitemapResult, AccessibilityResult, etc.)');
    console.log('   - Self-documenting via Swagger UI and /schema endpoint');
    console.log('   - Designed for Electron app integration');
    console.log('   - Backward compatible (v1 API still available)');

  } catch (error) {
    console.error('❌ API test failed:', error.message);
  } finally {
    // Cleanup (Note: AuditAPIServer doesn't have a stop method in current implementation)
    process.exit(0);
  }
}

// Add fetch polyfill for Node.js < 18
if (!globalThis.fetch) {
  const fetch = require('node-fetch');
  globalThis.fetch = fetch;
}

// Run the test
if (require.main === module) {
  testV2API()
    .then(() => {
      console.log('\n✅ All API tests completed');
    })
    .catch(error => {
      console.error('Fatal error:', error);
      process.exit(1);
    });
}

module.exports = { testV2API };
