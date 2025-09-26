#!/usr/bin/env node

/**
 * 🚀 AuditMySite CLI - Clean & Straightforward
 * 
 * 100% Event-driven workflow:
 * CLI → Config → Sitemap-Discovery → AccessibilityChecker → Reports → Done
 * 
 * No fallbacks, no "what-if" logic, no workarounds.
 */

const { Command } = require('commander');
const { SitemapParser } = require('../dist/parsers/sitemap-parser');
const { AccessibilityChecker } = require('../dist/core/accessibility');
const { HTMLGenerator } = require('../dist/generators/html-generator');
const { JsonGenerator } = require('../dist/generators/json-generator');
const path = require('path');
const fs = require('fs');
const packageJson = require('../package.json');

const program = new Command();

// CLI Configuration - MINIMAL OPTIONS
program
  .name('auditmysite')
  .description('🎯 Professional accessibility testing - clean and simple!')
  .version(packageJson.version)
  .argument('<sitemapUrl>', 'URL of the sitemap.xml to test')
  .option('--max-pages <number>', 'Maximum number of pages to test (default: 5)', (value) => parseInt(value))
  .option('--format <type>', 'Report format: html (default) or json', 'html')
  .option('--output-dir <dir>', 'Output directory for reports', './reports')
  .option('--non-interactive', 'Skip prompts for CI/CD')
  .option('-v, --verbose', 'Show detailed progress information')
  .action(async (sitemapUrl, options) => {
    
    try {
      console.log(`🚀 AuditMySite v${packageJson.version} - Clean & Straightforward`);
      console.log(`📄 Sitemap: ${sitemapUrl}`);
      
      // STEP 1: Configuration
      const config = {
        maxPages: options.maxPages || 5,
        format: options.format || 'html',
        outputDir: options.outputDir || './reports',
        verbose: options.verbose || false,
        maxConcurrent: 2
      };
      
      console.log(`\n📋 Configuration: ${config.maxPages} pages, ${config.format.toUpperCase()} format`);
      console.log(`🚀 All analysis features enabled by default`);
      
      // STEP 2: Parse sitemap
      console.log('\n🔍 Parsing sitemap...');
      const parser = new SitemapParser();
      const urls = await parser.parseSitemap(sitemapUrl);
      
      if (urls.length === 0) {
        throw new Error('Sitemap is empty or contains no valid URLs');
      }
      
      // STEP 2.5: Intelligent URL sampling with 301 redirect handling
      console.log(`📈 Found ${urls.length} URLs in sitemap`);
      console.log(`🎯 Sampling ${config.maxPages} working URLs (will skip 301 redirects)`);
      
      const normalizedUrls = await sampleWorkingUrls(urls, config.maxPages, config.verbose);
      
      if (normalizedUrls.length === 0) {
        throw new Error('No working URLs found in sitemap (all URLs redirect or are inaccessible)');
      }
      
      console.log(`✅ Selected ${normalizedUrls.length} working URLs for testing`);
      
      // STEP 3: Create accessibility checker with comprehensive analysis
      console.log('\n🚀 Initializing comprehensive analysis...');
      const checker = new AccessibilityChecker({
        enableComprehensiveAnalysis: true,
        qualityAnalysisOptions: {
          includeResourceAnalysis: true,
          includeSocialAnalysis: false,
          includeReadabilityAnalysis: true,
          includeTechnicalSEO: true,
          includeMobileFriendliness: true,
          analysisTimeout: 30000,
          verbose: config.verbose
        },
        enableUnifiedEvents: true,
        showDeprecationWarnings: false
      });
      
      await checker.initialize();
      console.log('✅ Event-driven checker with comprehensive analysis ready');
      
      // STEP 4: Run analysis
      const startTime = Date.now();
      const results = [];
      
      // Event callbacks for result collection
      const eventCallbacks = {
        onUrlCompleted: (url, result, duration) => {
          const shortUrl = url.split('/').pop() || url;
          if (config.verbose) {
            const status = result.passed ? '✅' : '⚠️';
            const errors = result.errors?.length || 0;
            const warnings = result.warnings?.length || 0;
            console.log(`${status} ${shortUrl} (${duration}ms) - ${errors} errors, ${warnings} warnings`);
          }
          results.push(result);
        },
        
        onUrlFailed: (url, error, attempts) => {
          const shortUrl = url.split('/').pop() || url;
          console.error(`❌ ${shortUrl}: Failed - ${error}`);
          results.push({
            url: url,
            title: 'Error',
            errors: [error],
            warnings: [],
            passed: false,
            crashed: true,
            duration: 0,
            pa11yScore: 0,
            pa11yIssues: []
          });
        },
        
        onProgressUpdate: (stats) => {
          if (!config.verbose && stats.progress % 25 === 0 && stats.progress > 0) {
            process.stdout.write(`\r🔍 Progress: ${Math.round(stats.progress)}% (${stats.completed}/${stats.total})`);
            if (stats.progress >= 100) process.stdout.write('\n');
          }
        }
      };
      
      checker.setUnifiedEventCallbacks(eventCallbacks);
      
      console.log(`\n🚀 Starting analysis of ${normalizedUrls.length} pages...`);
      await checker.testMultiplePagesWithQueue(normalizedUrls, {
        verbose: config.verbose,
        collectPerformanceMetrics: true,
        timeout: 30000,
        wait: 3000,
        includeWarnings: true,
        includeNotices: true,
        pa11yStandard: 'WCAG2AA',
        maxConcurrent: config.maxConcurrent,
        maxRetries: 3,
        retryDelay: 2000
      });
      
      console.log(`✅ Analysis completed: ${results.length} pages processed`);
      
      // STEP 5: Build AuditData with validation
      const totalDuration = Date.now() - startTime;
      const auditData = {
        metadata: {
          version: '1.0.0',
          timestamp: new Date().toISOString(),
          sitemapUrl: sitemapUrl,
          toolVersion: packageJson.version,
          duration: totalDuration
        },
        summary: {
          totalPages: results.length,
          testedPages: results.length,
          passedPages: results.filter(r => r.passed).length,
          failedPages: results.filter(r => !r.passed && !r.crashed).length,
          crashedPages: results.filter(r => r.crashed).length,
          totalErrors: results.reduce((sum, r) => sum + (r.errors?.length || 0), 0),
          totalWarnings: results.reduce((sum, r) => sum + (r.warnings?.length || 0), 0)
        },
        pages: results.map(result => ({
          url: result.url,
          title: result.title || 'Untitled',
          status: result.passed ? 'passed' : (result.crashed ? 'crashed' : 'failed'),
          duration: result.duration || 0,
          accessibility: {
            score: result.pa11yScore || 0,
            errors: result.pa11yIssues?.filter(i => i.type === 'error') || [],
            warnings: result.pa11yIssues?.filter(i => i.type === 'warning') || [],
            notices: result.pa11yIssues?.filter(i => i.type === 'notice') || []
          },
          performance: result.enhancedPerformance || result.performance ? {
            score: result.enhancedPerformance?.performanceScore || result.performance?.performanceScore || 0,
            grade: result.enhancedPerformance?.grade || result.performance?.grade || 'F',
            coreWebVitals: {
              largestContentfulPaint: result.enhancedPerformance?.coreWebVitals?.lcp?.value || result.performance?.coreWebVitals?.lcp || 0,
              firstContentfulPaint: result.enhancedPerformance?.coreWebVitals?.fcp?.value || result.performance?.coreWebVitals?.fcp || 0,
              cumulativeLayoutShift: result.enhancedPerformance?.coreWebVitals?.cls?.value || result.performance?.coreWebVitals?.cls || 0,
              timeToFirstByte: result.enhancedPerformance?.metrics?.ttfb?.value || result.performance?.metrics?.ttfb || 0
            }
          } : undefined,
          seo: result.enhancedSEO || result.seo ? {
            score: result.enhancedSEO?.seoScore || result.seo?.seoScore || 0,
            grade: result.enhancedSEO?.grade || result.seo?.grade || 'F',
            metaTags: result.enhancedSEO?.metaData || result.seo?.metaData || {},
            headings: result.enhancedSEO?.headingStructure || result.seo?.headingStructure || {},
            images: result.enhancedSEO?.images || result.seo?.images || {},
            issues: result.enhancedSEO?.issues || result.seo?.issues || [],
            url: result.url,
            title: result.title || 'Untitled'
          } : undefined,
          contentWeight: result.contentWeight ? {
            score: result.contentWeight.contentScore || result.contentWeight.score || 0,
            grade: result.contentWeight.grade || 'F',
            totalSize: result.contentWeight.contentMetrics?.totalSize || result.contentWeight.total || 0,
            resources: {
              html: { size: result.contentWeight.resourceAnalysis?.html?.size || 0 },
              css: { size: result.contentWeight.resourceAnalysis?.css?.size || 0, files: result.contentWeight.resourceAnalysis?.css?.count || 0 },
              javascript: { size: result.contentWeight.resourceAnalysis?.javascript?.size || 0, files: result.contentWeight.resourceAnalysis?.javascript?.count || 0 },
              images: { size: result.contentWeight.resourceAnalysis?.images?.size || 0, files: result.contentWeight.resourceAnalysis?.images?.count || 0 },
              other: { size: result.contentWeight.resourceAnalysis?.other?.size || 0, files: result.contentWeight.resourceAnalysis?.other?.count || 0 }
            },
            optimizations: result.contentWeight.optimizations || []
          } : undefined,
          mobileFriendliness: result.mobileFriendliness ? {
            overallScore: result.mobileFriendliness.overallScore || 0,
            grade: result.mobileFriendliness.grade || 'F',
            recommendations: result.mobileFriendliness.recommendations || []
          } : undefined
        }))
      };
      
      console.log('📊 AuditData structured successfully');
      
      // VALIDATE COMPREHENSIVE ANALYSIS DATA - throw error if missing
      const missingData = [];
      auditData.pages.forEach((page, index) => {
        if (!page.performance) missingData.push(`Performance data missing in page ${index}: ${page.url}`);
        if (!page.seo) missingData.push(`SEO data missing in page ${index}: ${page.url}`);
        if (!page.contentWeight) missingData.push(`Content Weight data missing in page ${index}: ${page.url}`);
        if (!page.mobileFriendliness) missingData.push(`Mobile Friendliness data missing in page ${index}: ${page.url}`);
      });
      
      if (missingData.length > 0) {
        console.error('❌ COMPREHENSIVE ANALYSIS DATA MISSING:');
        missingData.forEach(msg => console.error(`   - ${msg}`));
        throw new Error(`Comprehensive analysis failed: ${missingData.length} data points missing`);
      }
      
      console.log('✅ All comprehensive analysis data present and validated');
      
      // STEP 6: Generate reports
      const url = new URL(sitemapUrl);
      const domain = url.hostname.replace(/\./g, '-');
      const dateOnly = new Date().toLocaleDateString('en-CA');
      const outputDir = path.join(config.outputDir, domain);
      
      if (!fs.existsSync(outputDir)) {
        fs.mkdirSync(outputDir, { recursive: true });
      }
      
      const outputFiles = [];
      
      if (config.format === 'json') {
        console.log('\n📊 Generating JSON report...');
        const jsonGenerator = new JsonGenerator();
        const jsonContent = jsonGenerator.generateJson(auditData);
        const jsonPath = path.join(outputDir, `audit-${dateOnly}.json`);
        fs.writeFileSync(jsonPath, jsonContent);
        outputFiles.push(jsonPath);
        
      } else {
        console.log('\n📝 Generating HTML report...');
        const htmlGenerator = new HTMLGenerator();
        const htmlContent = await htmlGenerator.generate(auditData);
        const htmlPath = path.join(outputDir, `accessibility-report-${dateOnly}.html`);
        fs.writeFileSync(htmlPath, htmlContent);
        outputFiles.push(htmlPath);
      }
      
      // STEP 7: Cleanup
      console.log('\n🧹 Cleaning up resources...');
      try {
        if (checker) await checker.cleanup();
        console.log('✅ Resources cleaned up');
      } catch (cleanupError) {
        console.warn('⚠️  Cleanup warning:', cleanupError.message);
      }
      
      // STEP 8: Success output
      const totalTime = Math.round(totalDuration / 1000);
      const successRate = auditData.summary.testedPages > 0 ? 
        (auditData.summary.passedPages / auditData.summary.testedPages * 100).toFixed(1) : 0;
      
      console.log('\n✅ Test completed successfully!');
      console.log(`📊 Results:`);
      console.log(`   📄 Tested: ${auditData.summary.testedPages} pages in ${totalTime}s`);
      console.log(`   ✅ Passed: ${auditData.summary.passedPages}`);
      console.log(`   ❌ Failed: ${auditData.summary.failedPages}`);
      console.log(`   🎯 Success Rate: ${successRate}%`);
      
      if (auditData.summary.totalErrors > 0 || auditData.summary.totalWarnings > 0) {
        console.log(`   ⚠️  Issues: ${auditData.summary.totalErrors} errors, ${auditData.summary.totalWarnings} warnings`);
      }
      
      console.log(`\n📁 Generated reports:`);
      outputFiles.forEach(file => {
        const filename = path.basename(file);
        console.log(`   📄 ${filename}`);
      });
      
      process.exit(0);
      
    } catch (error) {
      console.error(`\n❌ Error: ${error.message}`);
      
      if (error.message.includes('sitemap') || error.message.includes('XML')) {
        console.error('\n💡 Sitemap issues:');
        console.error('   • Verify the sitemap URL is correct');
        console.error('   • Check if sitemap is publicly accessible');
        console.error('   • Ensure sitemap is properly formatted XML');
      } else if (error.message.includes('ENOTFOUND') || error.message.includes('timeout')) {
        console.error('\n💡 Network issues:');
        console.error('   • Check your internet connection');
        console.error('   • Verify the website is accessible');
      }
      
      if (options.verbose) {
        console.error('\n🔍 Full error details:');
        console.error(error.stack);
      }
      
      process.exit(1);
    }
  });

/**
 * Intelligent URL sampling with 301 redirect detection
 * 
 * Takes URLs from sitemap one by one, tests for redirects, and only includes working URLs
 * Reports 301 redirects but doesn't count them towards the target
 */
async function sampleWorkingUrls(urls, targetCount, verbose) {
  const workingUrls = [];
  let testedCount = 0;
  let redirectCount = 0;
  let errorCount = 0;
  const maxAttempts = Math.min(urls.length, Math.max(targetCount * 5, 20)); // Search up to 5x target or minimum 20 URLs
  
  console.log(`\n🔍 Sampling URLs from sitemap...`);
  
  for (let i = 0; i < urls.length && workingUrls.length < targetCount && testedCount < maxAttempts; i++) {
    const urlObj = urls[i];
    const url = typeof urlObj === 'string' ? urlObj : urlObj.loc;
    testedCount++;
    
    if (verbose) {
      process.stdout.write(`\r📍 Testing URL ${testedCount}/${Math.min(maxAttempts, urls.length)}: ${url.split('/').pop()}`);
    }
    
    try {
      // Quick HEAD request to check status without downloading content
      const https = require('https');
      const http = require('http');
      const urlParsed = new URL(url);
      const client = urlParsed.protocol === 'https:' ? https : http;
      
      const statusCode = await new Promise((resolve, reject) => {
        const req = client.request({
          hostname: urlParsed.hostname,
          port: urlParsed.port,
          path: urlParsed.pathname + urlParsed.search,
          method: 'HEAD',
          timeout: 5000,
          headers: {
            'User-Agent': 'AuditMySite/2.0 URL Sampler'
          }
        }, (res) => {
          resolve(res.statusCode);
        });
        
        req.on('error', (err) => reject(err));
        req.on('timeout', () => {
          req.destroy();
          reject(new Error('Timeout'));
        });
        
        req.end();
      });
      
      if (statusCode === 301 || statusCode === 302) {
        redirectCount++;
        console.log(`\n↪️  HTTP ${statusCode} Redirect: ${url}`);
        console.log(`   📋 Skipping redirect (${redirectCount} redirects found so far)`);
        
      } else if (statusCode >= 200 && statusCode < 300) {
        // Working URL
        workingUrls.push(url);
        if (verbose) {
          console.log(`\n✅ HTTP ${statusCode}: ${url}`);
        }
        
      } else if (statusCode >= 400) {
        errorCount++;
        console.log(`\n❌ HTTP ${statusCode} Error: ${url}`);
        console.log(`   📋 Skipping error page (${errorCount} errors found so far)`);
      }
      
    } catch (error) {
      errorCount++;
      console.log(`\n⚠️  Network Error: ${url} - ${error.message}`);
      console.log(`   📋 Skipping inaccessible URL (${errorCount} network errors so far)`);
    }
  }
  
  if (verbose || redirectCount > 0 || errorCount > 0) {
    console.log(`\n\n📊 URL Sampling Results:`);
    console.log(`   🎯 Target: ${targetCount} working URLs`);
    console.log(`   ✅ Found: ${workingUrls.length} working URLs`);
    console.log(`   ↪️  Redirects: ${redirectCount} URLs (skipped)`);
    console.log(`   ❌ Errors: ${errorCount} URLs (skipped)`);
    console.log(`   📍 Total tested: ${testedCount} URLs`);
  }
  
  return workingUrls;
}

program.parse();
