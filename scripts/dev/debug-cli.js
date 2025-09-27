#!/usr/bin/env node

/**
 * Debug Script to trace why CLI shows 0 URLs but direct test shows 125
 */

console.log('🔍 Starting CLI debug trace...');

// Set up the same environment as CLI
const finalSitemapUrl = 'https://www.aib-bauplanung.de/wp-sitemap.xml';
const config = { maxPages: 3, verbose: true };

async function debugCLIPath() {
  try {
    console.log('Step 1: Testing direct SitemapParser...');
    const { SitemapParser } = require('./dist/parsers/sitemap-parser');
    const parser = new SitemapParser();
    
    console.log('Step 2: Calling parser.parseSitemap...');
    const urls = await parser.parseSitemap(finalSitemapUrl);
    console.log(`Step 3: Parser returned ${urls.length} URLs`);
    
    console.log('Step 4: Slicing URLs...');
    const limitedUrls = urls.slice(0, config.maxPages || 5);
    console.log(`Step 5: Limited URLs: ${limitedUrls.length}`);
    
    if (config.verbose) console.log(`📈 Found ${urls.length} URLs in sitemap, testing ${limitedUrls.length}`);
    
    // Check if we have any URLs to test
    if (limitedUrls.length === 0) {
      console.log('❌ No URLs found in sitemap or sitemap is empty');
      console.log('💡 Please check:');
      console.log('   - The sitemap URL is correct and accessible');
      console.log('   - The sitemap contains valid URL entries');
      console.log('   - The sitemap is properly formatted XML');
      process.exit(1);
    }
    
    console.log('✅ URLs found successfully');
    console.log('First 3 URLs:');
    limitedUrls.slice(0, 3).forEach((url, i) => {
      const finalUrl = typeof url === 'string' ? url : url.loc;
      console.log(`  ${i + 1}. ${finalUrl}`);
    });
    
  } catch (error) {
    console.error('❌ Error in debug trace:', error);
    console.error('Stack:', error.stack);
  }
}

debugCLIPath().catch(err => {
  console.error('❌ Unhandled error:', err);
});