#!/usr/bin/env node

/**
 * Test-Skript zur Analyse von Redirects auf casoon.de
 * Testet alle URLs aus der Sitemap und dokumentiert die Redirects
 */

const https = require('https');
const http = require('http');

const urls = [
  'https://www.casoon.de/',
  'https://www.casoon.de/arbeitsweise',
  'https://www.casoon.de/cloud-entwicklung',
  'https://www.casoon.de/datenschutz',
  'https://www.casoon.de/e-commerce',
  'https://www.casoon.de/impressum',
  'https://www.casoon.de/kollaboration',
  'https://www.casoon.de/kontakt',
  'https://www.casoon.de/leistungskatalog',
  'https://www.casoon.de/plattform-apps',
  'https://www.casoon.de/projekte',
  'https://www.casoon.de/seo-marketing',
  'https://www.casoon.de/technologien',
  'https://www.casoon.de/usp',
  'https://www.casoon.de/webentwicklung'
];

/**
 * Testet eine URL auf Redirects
 */
async function testUrl(url) {
  return new Promise((resolve) => {
    const urlObj = new URL(url);
    const protocol = urlObj.protocol === 'https:' ? https : http;

    const options = {
      method: 'HEAD',
      headers: {
        'User-Agent': 'AuditMySite-RedirectChecker/1.0'
      },
      // Don't follow redirects automatically
      followRedirect: false,
      timeout: 10000
    };

    const req = protocol.request(url, options, (res) => {
      const statusCode = res.statusCode;
      const location = res.headers.location;

      const result = {
        url: url,
        statusCode: statusCode,
        isRedirect: statusCode >= 300 && statusCode < 400,
        redirectTo: location || null,
        finalUrl: location ? (location.startsWith('http') ? location : new URL(location, url).href) : url
      };

      resolve(result);
    });

    req.on('error', (error) => {
      resolve({
        url: url,
        statusCode: 0,
        isRedirect: false,
        redirectTo: null,
        finalUrl: url,
        error: error.message
      });
    });

    req.on('timeout', () => {
      req.destroy();
      resolve({
        url: url,
        statusCode: 0,
        isRedirect: false,
        redirectTo: null,
        finalUrl: url,
        error: 'Timeout'
      });
    });

    req.end();
  });
}

/**
 * Hauptfunktion
 */
async function main() {
  console.log('🔍 CASOON.DE Redirect Analysis');
  console.log('━'.repeat(80));
  console.log('');

  const results = [];

  for (const url of urls) {
    process.stdout.write(`Testing: ${url} ... `);
    const result = await testUrl(url);
    results.push(result);

    if (result.error) {
      console.log(`❌ ERROR: ${result.error}`);
    } else if (result.isRedirect) {
      console.log(`🔀 REDIRECT (${result.statusCode}) → ${result.finalUrl}`);
    } else if (result.statusCode === 200) {
      console.log(`✅ OK (${result.statusCode})`);
    } else {
      console.log(`⚠️  ${result.statusCode}`);
    }

    // Small delay to avoid rate limiting
    await new Promise(resolve => setTimeout(resolve, 500));
  }

  console.log('');
  console.log('━'.repeat(80));
  console.log('📊 SUMMARY');
  console.log('━'.repeat(80));
  console.log('');

  const okUrls = results.filter(r => r.statusCode === 200);
  const redirectUrls = results.filter(r => r.isRedirect);
  const errorUrls = results.filter(r => r.error || (r.statusCode !== 200 && !r.isRedirect));

  console.log(`Total URLs tested: ${results.length}`);
  console.log(`✅ OK (200): ${okUrls.length}`);
  console.log(`🔀 Redirects (3xx): ${redirectUrls.length}`);
  console.log(`❌ Errors/Other: ${errorUrls.length}`);
  console.log('');

  if (redirectUrls.length > 0) {
    console.log('━'.repeat(80));
    console.log('🔀 REDIRECT DETAILS');
    console.log('━'.repeat(80));
    console.log('');

    redirectUrls.forEach(r => {
      console.log(`Source: ${r.url}`);
      console.log(`Status: ${r.statusCode}`);
      console.log(`Target: ${r.finalUrl}`);
      console.log('');
    });
  }

  if (okUrls.length > 0) {
    console.log('━'.repeat(80));
    console.log('✅ WORKING URLs (No Redirects)');
    console.log('━'.repeat(80));
    console.log('');
    okUrls.forEach(r => {
      console.log(`- ${r.url}`);
    });
    console.log('');
  }

  // Generate recommended sitemap
  console.log('━'.repeat(80));
  console.log('💡 RECOMMENDATIONS');
  console.log('━'.repeat(80));
  console.log('');

  if (redirectUrls.length === 0) {
    console.log('✅ No redirects found! Sitemap is already correct.');
  } else {
    console.log('📝 Recommended sitemap updates:');
    console.log('');
    console.log('OPTION 1: Use only working URLs (remove redirecting URLs)');
    console.log('─'.repeat(80));
    okUrls.forEach(r => {
      console.log(`  <url><loc>${r.url}</loc></url>`);
    });
    console.log('');

    console.log('OPTION 2: Replace redirecting URLs with their targets');
    console.log('─'.repeat(80));
    const uniqueTargets = new Set();
    okUrls.forEach(r => uniqueTargets.add(r.url));
    redirectUrls.forEach(r => uniqueTargets.add(r.finalUrl));

    Array.from(uniqueTargets).sort().forEach(url => {
      console.log(`  <url><loc>${url}</loc></url>`);
    });
  }

  console.log('');
  console.log('━'.repeat(80));
  console.log('✅ Analysis complete!');
  console.log('━'.repeat(80));
}

// Run the analysis
main().catch(console.error);
