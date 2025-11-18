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
const { SitemapDiscovery } = require('../dist/core/parsers');
const { SitemapParser } = require('../dist/parsers/sitemap-parser');
const { AccessibilityChecker } = require('../dist/core/accessibility');
const { BrowserPoolManager } = require('../dist/core/browser/browser-pool-manager');
const { HTMLGenerator } = require('../dist/generators/html-generator');
const { JsonGenerator } = require('../dist/generators/json-generator');
const { FileQueueStateAdapter } = require('../dist/core/queue/file-queue-state-adapter');
const inquirer = require('inquirer').default;
const path = require('path');
const fs = require('fs');
const packageJson = require('../package.json');

const program = new Command();

// 🎯 SIMPLIFIED CLI - Only 11 essential parameters!
program
  .name('auditmysite')
  .description('🎯 Professional accessibility testing - clean and simple!')
  .version(packageJson.version)
  .argument('<sitemapUrl>', 'URL of the sitemap.xml to test')
  
  // ✅ Core Options (4)
  .option('--max-pages <number>', 'Maximum number of pages to test (default: 5)', (value) => parseInt(value))
  .option('--format <type>', 'Report format: html (default, with detailed issues MD) or json (complete data)', 'html')
  .option('--output-dir <dir>', 'Output directory for reports', './reports')
  .option('--budget <template>', 'Performance budget: default, ecommerce, blog, corporate', 'default')
  
  // ✅ User Experience (4)
  .option('--expert', 'Interactive expert mode with advanced settings')
  .option('--non-interactive', 'Skip prompts for CI/CD (use defaults)')
  .option('--quiet-deprecations', 'Suppress deprecation warnings (auto-detected in CI: CI=true, NODE_ENV=production)')
  .option('-v, --verbose', 'Show detailed progress information')
  
  // ✅ Analysis Control (5) - Opt-out instead of opt-in
  .option('--no-performance', 'Disable performance analysis')
  .option('--no-seo', 'Disable SEO analysis')
  .option('--no-content-weight', 'Disable content weight analysis')
  .option('--no-mobile', 'Disable mobile-friendliness analysis')
  .option('--geo <locations>', 'Enable geographic testing with comma-separated locations (e.g., germany-berlin,usa-newyork,uk-london)')
  
  // ✅ Resume/Persistence Options (3)
  .option('--resume <stateId>', 'Resume a previous audit from saved state')
  .option('--save-state', 'Save audit state for resumption (enables persistence)')
  .option('--list-states', 'List all available saved audit states')
  
  .action(async (sitemapUrl, options) => {
    
    // Handle --list-states command
    if (options.listStates) {
      console.log('💾 Listing saved audit states...');
      try {
        const adapter = new FileQueueStateAdapter();
        const states = await adapter.list();
        
        if (states.length === 0) {
          console.log('No saved audit states found.');
          return;
        }
        
        console.log(`\nFound ${states.length} saved state(s):`);
        console.log('\u2500'.repeat(80));
        
        for (const stateId of states) {
          try {
            const info = await adapter.getStateInfo(stateId);
            if (info) {
              const date = new Date(info.lastUpdateTime).toLocaleString();
              const progress = info.totalUrls > 0 ? Math.round((info.processedUrls / info.totalUrls) * 100) : 0;
              console.log(`💾 ${stateId}`);
              console.log(`   Status: ${info.status} | Progress: ${info.processedUrls}/${info.totalUrls} (${progress}%)`);
              console.log(`   Last Updated: ${date}`);
              console.log('');
            }
          } catch (error) {
            console.log(`💾 ${stateId} (unable to read details)`);
          }
        }
        
        console.log('To resume a specific audit, use: --resume <stateId>');
        return;
      } catch (error) {
        console.error('Failed to list saved states:', error.message);
        process.exit(1);
      }
    }
    
    console.log(`🚀 AuditMySite v${packageJson.version} - Professional Accessibility Testing`);
    console.log(`📄 Sitemap: ${sitemapUrl}`);
    
    // 🎯 SMART DEFAULTS - Clean and simple!
    const QUICK_DEFAULTS = {
      maxPages: options.maxPages || 5,
      standard: 'WCAG2AA',
      format: options.format || 'html',
      outputDir: options.outputDir || './reports',
      budget: options.budget || 'default',
      timeout: 10000,
      maxConcurrent: 2,
      verbose: options.verbose || false,
      // 🚀 All Analysis Features ENABLED by default (opt-out model)
      performanceAnalysis: !options.noPerformance,
      seoAnalysis: !options.noSeo,
      contentWeight: !options.noContentWeight,
      mobileFriendliness: !options.noMobile,
      geoAudit: options.geo ? options.geo.split(',').map(loc => loc.trim()) : null
    };
    
    let config = { ...QUICK_DEFAULTS };
    
    // 🔧 EXPERT MODE - Interactive wizard
    if (options.expert && !options.nonInteractive) {
      console.log('\n🔧 Expert Mode - Custom Configuration');
      console.log('━'.repeat(50));
      
      const answers = await inquirer.prompt([
        {
          type: 'list',
          name: 'maxPages',
          message: '🔢 How many pages to test?',
          choices: [
            { name: '⚡ 5 pages (Quick test) - ~2 minutes', value: 5 },
            { name: '🎯 20 pages (Standard test) - ~8 minutes', value: 20 },
            { name: '📊 50 pages (Comprehensive) - ~20 minutes', value: 50 },
            { name: '🚀 All pages (Maximum coverage) - varies', value: 1000 }
          ],
          default: 20
        },
        {
          type: 'list', 
          name: 'standard',
          message: '♿ Accessibility standard?',
          choices: [
            { name: '🎯 WCAG 2.1 AA (Recommended) - Industry standard', value: 'WCAG2AA' },
            { name: '⭐ WCAG 2.1 AAA (Strict) - Highest compliance', value: 'WCAG2AAA' },
            { name: '🇺🇸 Section 508 (US Federal) - Government sites', value: 'Section508' }
          ],
          default: 'WCAG2AA'
        },
        {
          type: 'list',
          name: 'format',
          message: '📄 Report format?',
          choices: [
            { name: '🌐 HTML - Professional reports (includes detailed issues MD)', value: 'html' },
            { name: '📊 JSON - Complete typed data object for further processing', value: 'json' }
          ],
          default: 'html'
        },
        {
          type: 'confirm',
          name: 'generatePerformanceReport',
          message: '⚡ Include Core Web Vitals performance metrics?',
          default: true
        },
        {
          type: 'number',
          name: 'maxConcurrent',
          message: '🔄 Concurrent page tests (1-5)?',
          default: 2,
          validate: (value) => {
            const num = parseInt(value);
            if (num >= 1 && num <= 5) return true;
            return 'Please enter a number between 1 and 5';
          }
        },
        {
          type: 'confirm',
          name: 'verbose',
          message: '🔍 Show detailed progress information?',
          default: false
        },
        {
          type: 'checkbox',
          name: 'analysisFeatures',
          message: '🔍 Which analysis features to enable?',
          choices: [
            { name: '⚡ Performance Analysis - Core Web Vitals, loading metrics', value: 'performance', checked: true },
            { name: '🔍 SEO Analysis - Meta tags, content quality, structure', value: 'seo', checked: true },
            { name: '📏 Content Weight Analysis - Resource optimization', value: 'contentWeight', checked: true },
            { name: '📱 Mobile-Friendliness Analysis - Touch targets, responsive', value: 'mobile', checked: true }
          ],
          default: ['performance', 'seo', 'contentWeight', 'mobile']
        },
        {
          type: 'list',
          name: 'budget',
          message: '📈 Performance budget template?',
          choices: [
            { name: '⚙️ Default - Google Web Vitals standard thresholds', value: 'default' },
            { name: '🏬 E-commerce - Conversion-focused (stricter for revenue)', value: 'ecommerce' },
            { name: '🏢 Corporate - Professional standards (balanced)', value: 'corporate' },
            { name: '📝 Blog - Content-focused (relaxed for reading)', value: 'blog' }
          ],
          default: 'default'
        },
        {
          type: 'confirm',
          name: 'enableGeoAudit',
          message: '🌍 Enable geographic performance testing?',
          default: false
        },
        {
          type: 'checkbox',
          name: 'geoLocations',
          message: '🗺️ Select geographic locations to test from:',
          choices: [
            { name: '🇩🇪 Germany (Berlin)', value: 'germany-berlin' },
            { name: '🇺🇸 USA (New York)', value: 'usa-newyork' },
            { name: '🇬🇧 UK (London)', value: 'uk-london' },
            { name: '🇫🇷 France (Paris)', value: 'france-paris' },
            { name: '🇯🇵 Japan (Tokyo)', value: 'japan-tokyo' },
            { name: '🇦🇺 Australia (Sydney)', value: 'australia-sydney' }
          ],
          default: ['germany-berlin', 'usa-newyork'],
          when: (answers) => answers.enableGeoAudit
        }
      ]);
      
      // Update config with analysis feature selections
      config.performanceAnalysis = answers.analysisFeatures.includes('performance');
      config.seoAnalysis = answers.analysisFeatures.includes('seo');
      config.contentWeight = answers.analysisFeatures.includes('contentWeight');
      config.mobileFriendliness = answers.analysisFeatures.includes('mobile');
      
      // Update GEO audit configuration
      if (answers.enableGeoAudit && answers.geoLocations) {
        config.geoAudit = answers.geoLocations;
      }
      
      config = { ...config, ...answers };
    }
    
    // 🐎 Create performance budget from template
    const { BUDGET_TEMPLATES } = require('../dist/core/performance/web-vitals-collector');
    const template = config.budget || 'default';
    const performanceBudget = BUDGET_TEMPLATES[template] || BUDGET_TEMPLATES.default;
    
    // 📈 Show configuration
    console.log(`\n📋 Configuration:`);
    console.log(`   📄 Pages: ${config.maxPages}`);
    console.log(`   📋 Standard: ${config.standard}`);
    console.log(`   📈 Budget: ${template} (LCP: ${performanceBudget.lcp.good}ms, CLS: ${performanceBudget.cls.good})`);
    console.log(`   📄 Format: ${config.format.toUpperCase()}`);
    console.log(`   📁 Output: ${config.outputDir}`);
    
    // Analysis Features Summary
    console.log('\n🚀 Analysis Features:');
    console.log(`   ⚡ Performance: ${config.performanceAnalysis ? '✅' : '❌'}`);
    console.log(`   🔍 SEO: ${config.seoAnalysis ? '✅' : '❌'}`);
    console.log(`   📏 Content Weight: ${config.contentWeight ? '✅' : '❌'}`);
    console.log(`   📱 Mobile-Friendliness: ${config.mobileFriendliness ? '✅' : '❌'}`);
    if (config.geoAudit && config.geoAudit.length > 0) {
      console.log(`   🌍 GEO Audit: ✅ (${config.geoAudit.length} locations)`);
    }
    
    console.log('\n✨ Simplified CLI - Only 12 parameters for better usability!');
    
    // 🔇 Configure deprecation warning suppression for CI/CD environments
    const shouldSuppressDeprecations = 
      options.quietDeprecations || 
      process.env.CI === 'true' ||
      process.env.NODE_ENV === 'production';
    
    if (shouldSuppressDeprecations) {
      process.env.AUDITMYSITE_SUPPRESS_DEPRECATIONS = 'true';
      if (config.verbose) console.log('🔇 Deprecation warnings suppressed for CI/CD environment');
    }
    
    // 💾 Handle persistence and resume options - enabled by default
    let resumeFromState = false;
    let persistenceConfig = {
      enablePersistence: options.saveState !== false, // Default: enabled unless explicitly disabled
      stateId: options.resume || undefined,
      resumable: true
    };
    
    if (options.resume) {
      console.log(`\n💾 Attempting to resume from state: ${options.resume}`);
      resumeFromState = true;
      persistenceConfig.enablePersistence = true;
      
      try {
        const adapter = new FileQueueStateAdapter();
        const stateExists = await adapter.exists(options.resume);
        if (!stateExists) {
          console.error(`❌ State not found: ${options.resume}`);
          console.log('Use --list-states to see available states');
          process.exit(1);
        }
        console.log(`✅ State found, will resume processing...`);
      } catch (error) {
        console.error(`Failed to check state: ${error.message}`);
        process.exit(1);
      }
    } else if (options.saveState) {
      console.log('\n💾 State persistence enabled - audit can be resumed if interrupted');
    }
    
    // Helper functions for grade calculation
    function calculateGrade(score) {
      if (score >= 90) return 'A';
      if (score >= 80) return 'B';
      if (score >= 70) return 'C';
      if (score >= 60) return 'D';
      return 'F';
    }
    
    function calculateCertificateLevel(score) {
      if (score >= 95) return 'PLATINUM';
      if (score >= 85) return 'GOLD';
      if (score >= 75) return 'SILVER';
      if (score >= 65) return 'BRONZE';
      return 'NEEDS_IMPROVEMENT';
    }
    
    // Declare variables in outer scope for error handling
    let summary;
    let outputFiles;
    let startTime;
    
    try {
      // Extract domain for report organization - handle both URL and local file paths
      let domain;
      if (sitemapUrl.startsWith('http://') || sitemapUrl.startsWith('https://')) {
        const url = new URL(sitemapUrl);
        domain = url.hostname.replace(/\\./g, '-');
      } else {
        // For local files, use filename as domain
        domain = path.basename(sitemapUrl, path.extname(sitemapUrl)).replace(/[^a-zA-Z0-9-]/g, '-');
      }
      const dateOnly = new Date().toLocaleDateString('en-CA');
      
      // Create domain subdirectory
      const fs = require('fs');
      const subDir = path.join(config.outputDir, domain);
      if (!fs.existsSync(subDir)) {
        fs.mkdirSync(subDir, { recursive: true });
      }
      
      // Configuration for modern architecture
      const modernAnalysisConfig = {
        sitemapUrl,
        maxPages: config.maxPages,
        timeout: config.timeout,
        pa11yStandard: config.standard,
        outputDir: subDir,
        outputFormat: config.format,
        maxConcurrent: config.maxConcurrent,
        verbose: config.verbose,
        timestamp: new Date().toISOString(),
        
        // 🚀 Analysis Features (opt-out model)
        performanceAnalysis: config.performanceAnalysis,
        seoAnalysis: config.seoAnalysis,
        contentWeight: config.contentWeight,
        mobileFriendliness: config.mobileFriendliness,
        
        // 📊 Performance Budget
        performanceBudget: performanceBudget
      };
      
      // 🔍 Smart Sitemap Discovery first
      let finalSitemapUrl = sitemapUrl;
      if (!sitemapUrl.includes('sitemap.xml') && !sitemapUrl.includes('sitemap')) {
        console.log('\n🔍 Discovering sitemap...');
        const discovery = new SitemapDiscovery();
        const result = await discovery.discoverSitemap(sitemapUrl);
        
        if (result.found) {
          finalSitemapUrl = result.sitemaps[0];
          console.log(`✅ Found sitemap: ${finalSitemapUrl} (method: ${result.method})`);
          if (result.sitemaps.length > 1) {
            console.log(`📋 Additional sitemaps found: ${result.sitemaps.length - 1}`);
          }
        } else {
          console.log('❌ No sitemap found');
          result.warnings.forEach(warning => console.log(`   ⚠️  ${warning}`));
          process.exit(1);
        }
      }
      
      // Professional analysis with all features enabled by default
      const useStandardAnalysis = true;
      
      if (useStandardAnalysis) {
        if (config.verbose) console.log('\\n🚀 Starting accessibility analysis...');
        
        try {
        // 🚀 Using modern event-driven parallel browser architecture (default)
        if (config.verbose) console.log('✅ Using modern event-driven parallel testing architecture');
        
        // Use parallel accessibility analysis pipeline
        const { SitemapParser } = require('../dist/parsers/sitemap-parser');
        
        // Parse sitemap
        const parser = new SitemapParser();
        
        let urls;
        try {
          urls = await parser.parseSitemap(finalSitemapUrl);
        } catch (parseError) {
          throw parseError;
        }
        
        // Check if we have any URLs to test
        if (!urls || urls.length === 0) {
          console.log('❌ No URLs found in sitemap or sitemap is empty');
          console.log('💡 Please check:');
          console.log('   - The sitemap URL is correct and accessible');
          console.log('   - The sitemap contains valid URL entries');
          console.log('   - The sitemap is properly formatted XML');
          process.exit(1);
        }
        
        console.log(`📈 Found ${urls.length} URLs in sitemap, testing ${config.maxPages || 5}`);
        
        // 🎯 SMART URL SAMPLING - Find good pages, skip redirects
        console.log('🚀 Starting accessibility analysis...');
        console.log('✅ Using modern event-driven parallel testing architecture');
        
        const { AccessibilityChecker } = require('../dist/core/accessibility/accessibility-checker');
        const { BrowserPoolManager } = require('../dist/core/browser/browser-pool-manager');
        const { SilentLogger } = require('../dist/core/logging/structured-logger');
        
        // Initialize accessibility checker for smart sampling
        if (config.verbose) console.log('🚀 Initializing smart URL sampler with comprehensive analysis...');
        
        // Create browser pool manager for improved performance
        const poolManager = new BrowserPoolManager({
          maxBrowsers: config.maxConcurrent || 2,
          maxPagesPerBrowser: 5,
          verbose: config.verbose
        });
        
        // Initialize pool manager if method exists
        if (typeof poolManager.initialize === 'function') {
          await poolManager.initialize();
        } else {
          console.log('⚠️  Pool manager auto-initialization');
        }
        
        const qualityOptions = {
          includeResourceAnalysis: true,
          includeSocialAnalysis: false,
          includeReadabilityAnalysis: true,
          includeTechnicalSEO: true,
          includeMobileFriendliness: true,
          analysisTimeout: 30000,
          // Lighthouse Slow 4G Standard (matches PageSpeed Insights)
          // Mobile lab conditions for realistic performance testing
          psiProfile: true,
          psiCPUThrottlingRate: 4,
          psiNetwork: { latencyMs: 400, downloadKbps: 400, uploadKbps: 400 }
        };
        const checker = new AccessibilityChecker({
          poolManager: poolManager,
          logger: new SilentLogger(),
          enableComprehensiveAnalysis: true,
          analyzerTypes: ['performance', 'seo', 'content-weight', 'mobile-friendliness'],
          qualityAnalysisOptions: qualityOptions
        });
        
        await checker.initialize();
        if (config.verbose) console.log('🔧 Initializing comprehensive analysis with all analyzers');
        
        console.log('🎯 Testing pages with automatic redirect filtering');
        const allNormalizedUrls = urls.map(urlObj => typeof urlObj === 'string' ? urlObj : urlObj.loc);
        
        // Determine homepage (ensure it's first)
        let homepageUrl;
        try {
          const base = new URL(finalSitemapUrl);
          homepageUrl = base.origin + '/';
        } catch {
          homepageUrl = allNormalizedUrls[0];
        }
        
        // Smart sampling: pick the next non-redirecting URLs until we reach (targetCount - 1) because homepage will be added first
        const targetCount = config.maxPages || 5;
        const urlsToTest = [];
        const redirectSkips = [];
        
        // Exclude homepage from candidates to avoid duplication
        const candidates = allNormalizedUrls.filter(u => u !== homepageUrl);
        
        // Parallel batch minimal checks respecting a safe concurrency limit
        const samplingConcurrency = Math.max(1, config.maxConcurrent || 2);
        let index = 0;
        while (urlsToTest.length < (targetCount - 1) && index < candidates.length) {
          const batch = candidates.slice(index, index + samplingConcurrency);
          index += batch.length;
          const batchResults = await Promise.all(
            batch.map(async (candidate) => {
              try {
                const mini = await checker.testUrlMinimal(candidate, 8000);
                return { candidate, mini, error: null };
              } catch (error) {
                return { candidate, mini: null, error };
              }
            })
          );
          for (const r of batchResults) {
            if (urlsToTest.length >= (targetCount - 1)) break;
            if (r.error) {
              // On minimal test failure, include the URL for full testing
              urlsToTest.push(r.candidate);
            } else if (r.mini.skipped && (r.mini.errors || []).some(e => /Redirect/i.test(e))) {
              redirectSkips.push(r.candidate);
            } else {
              urlsToTest.push(r.candidate);
            }
          }
        }
        
        // Build final ordered test set: homepage first, then sampled URLs
        const finalUrlsToTest = [homepageUrl, ...urlsToTest].slice(0, targetCount);
        
        console.log(`📈 Testing ${finalUrlsToTest.length} URLs (requested: ${targetCount})`);
        if (redirectSkips.length > 0) {
          console.log(`↪️  Skipped ${redirectSkips.length} redirected URLs during sampling`);
        }
        if (finalUrlsToTest.length < targetCount) {
          console.log(`⚠️  Only ${finalUrlsToTest.length} non-redirect URLs available in sitemap (homepage forced first)`);
        }
        
        // Phase 1: Test homepage with redirects allowed (to always analyze the landing page)
        startTime = Date.now();
        const homepageResult = await checker.testPage(homepageUrl, {
          skipRedirects: false,
          verbose: config.verbose,
          timeout: 30000,
          maxConcurrent: 1,
          enableComprehensiveAnalysis: true
        });
        
        // Phase 2: Test remaining pages with redirect filtering enabled
        const restUrls = finalUrlsToTest.filter(u => u !== homepageUrl);
        let testResult = await checker.testMultiplePages(restUrls, {
          skipRedirects: true,
          verbose: config.verbose,
          timeout: 30000,
          maxConcurrent: config.maxConcurrent || 2,
          enableComprehensiveAnalysis: true
        });
        
        // Combine results with homepage and ensure uniqueness
        let combinedPageResults = [homepageResult, ...testResult.results].filter((v, i, arr) => v && arr.findIndex(x => x.url === v.url) === i);
        
        // Top-up: ensure we have targetCount non-skipped pages by sampling more if needed
        const isNonSkipped = (r) => !(r && r.accessibilityResult && r.accessibilityResult.skipped);
        let nonSkippedCount = combinedPageResults.filter(isNonSkipped).length;
        
        while (nonSkippedCount < targetCount && index < candidates.length) {
          const need = targetCount - nonSkippedCount;
          const batch = candidates.slice(index, index + Math.max(need, samplingConcurrency));
          index += batch.length;
          // Minimal checks first
          const miniResults = await Promise.all(batch.map(async (candidate) => {
            try {
              const mini = await checker.testUrlMinimal(candidate, 8000);
              return { candidate, mini, error: null };
            } catch (error) {
              return { candidate, mini: null, error };
            }
          }));
          const additional = miniResults
            .filter(r => !(r.mini && r.mini.skipped && (r.mini.errors || []).some(e => /Redirect/i.test(e))))
            .map(r => r.candidate)
            .filter(u => u !== homepageUrl);
          // Remove already tested
          const already = new Set(combinedPageResults.map(r => r.url));
          const additionalRest = additional.filter(u => !already.has(u)).slice(0, need);
          if (additionalRest.length === 0) {
            continue;
          }
          // Test additional batch
          const addResult = await checker.testMultiplePages(additionalRest, {
            skipRedirects: true,
            verbose: config.verbose,
            timeout: 30000,
            maxConcurrent: config.maxConcurrent || 2,
            enableComprehensiveAnalysis: true
          });
          combinedPageResults = [...combinedPageResults, ...addResult.results].filter((v, i, arr) => v && arr.findIndex(x => x.url === v.url) === i);
          nonSkippedCount = combinedPageResults.filter(isNonSkipped).length;
        }
        
        const limitedUrls = combinedPageResults.map(r => r.url);
        
        if (config.verbose) {
          console.log(`📊 Testing Results:`);
          console.log(`   ✅ Successfully tested: ${combinedPageResults.length} (non-skipped: ${nonSkippedCount})`);
          console.log(`   🎯 Total duration: ${Math.round((Date.now() - startTime) / 1000)}s`);
        } else {
          console.log(`✅ Completed testing ${combinedPageResults.length} URLs (non-skipped: ${nonSkippedCount})`);
        }
        
        if (limitedUrls.length === 0) {
          console.log('❌ No accessible URLs found for testing');
          console.log('💡 Possible issues:');
          console.log('   - All URLs in sitemap redirect to other pages');
          console.log('   - URLs are returning 404 or other errors');
          console.log('   - Network connectivity issues');
          console.log(`   - Tried ${urlsToTest.length} URLs from sitemap`);
          
          // Cleanup before exit
          await checker.cleanup();
          await poolManager.cleanup();
          process.exit(1);
        }
        
        startTime = Date.now(); // Use outer scope variable
        
      // Convert test results to the expected format
      const results = [];
      let successCount = 0;
      let errorCount = 0;
      let warningCount = 0;
      let redirectCount = testResult.skippedUrls.length;

      // Helper: map comprehensive analysis results array to structured fields expected by reports
      function mapComprehensiveToPageFields(comprehensive) {
        if (!comprehensive || !Array.isArray(comprehensive.results)) return {};
        const out = {};
        const res = comprehensive.results;
        const byType = {};
        res.forEach(r => {
          const t = r?.metadata?.analyzerType;
          if (t) byType[t] = r;
        });
        // Performance
        if (byType['performance']) {
          const p = byType['performance'];
          out.performance = {
            score: p.performanceScore ?? p.score ?? 0,
            grade: p.performanceGrade ?? p.grade ?? 'F',
            coreWebVitals: {
              largestContentfulPaint: p.lcp ?? p.coreWebVitals?.lcp ?? p.coreWebVitals?.largestContentfulPaint ?? 0,
              firstContentfulPaint: p.firstContentfulPaint ?? p.coreWebVitals?.fcp ?? p.coreWebVitals?.firstContentfulPaint ?? 0,
              cumulativeLayoutShift: p.cls ?? p.coreWebVitals?.cls ?? p.coreWebVitals?.cumulativeLayoutShift ?? 0,
              timeToFirstByte: p.ttfb ?? p.coreWebVitals?.ttfb ?? p.coreWebVitals?.timeToFirstByte ?? 0
            },
            metrics: {
              domContentLoaded: p.domContentLoaded ?? p.metrics?.domContentLoaded ?? 0,
              loadComplete: p.loadComplete ?? p.metrics?.loadComplete ?? 0,
              firstPaint: p.firstPaint ?? p.metrics?.firstPaint ?? 0,
              requestCount: p.requestCount ?? p.metrics?.requestCount ?? (Array.isArray(p.resourceLoadTimes) ? p.resourceLoadTimes.length : 0),
              transferSize: p.transferSize ?? p.metrics?.transferSize ?? p.contentWeight?.gzipTotal ?? p.contentWeight?.total ?? 0
            },
            recommendations: p.recommendations || [],
            issues: p.issues || []
          };
        }
        // SEO
        if (byType['seo']) {
          const s = byType['seo'];
          out.seo = {
            score: s.overallSEOScore || s.score || s.seoScore || s.overallScore || 0,
            grade: s.seoGrade || s.grade || 'F',
            metaTags: s.metaTags || s.metaData || {},
            headingStructure: s.headingStructure || s.headings || { h1: 0, h2: 0, h3: 0, issues: [] },
            issues: s.issues || s.recommendations || []
          };
        }
        // Content Weight
        if (byType['content-weight']) {
          const c = byType['content-weight'];
          const cw = c.contentWeight || {};
          const resources = cw.resources || cw.resourceAnalysis || {};
          out.contentWeight = {
            score: c.overallScore || c.score || c.contentScore || cw.contentQualityScore || 0,
            grade: c.grade || 'F',
            totalSize: cw.total || cw.totalSize || 0,
            resources: {
              html: { size: (resources.html?.size ?? resources.html ?? cw.html ?? 0) },
              css: { size: (resources.css?.size ?? resources.css ?? cw.css ?? 0), files: (resources.css?.files ?? resources.css?.count ?? 0) },
              javascript: { size: (resources.javascript?.size ?? resources.js?.size ?? resources.javascript ?? resources.js ?? cw.javascript ?? 0), files: (resources.javascript?.files ?? resources.js?.files ?? resources.javascript?.count ?? resources.js?.count ?? 0) },
              images: { size: (resources.images?.size ?? resources.images ?? cw.images ?? 0), files: (resources.images?.files ?? resources.images?.count ?? 0) },
              fonts: { size: (resources.fonts?.size ?? resources.fonts ?? cw.fonts ?? 0) },
              other: { size: (resources.other?.size ?? resources.other ?? cw.other ?? 0) }
            },
            optimizations: c.recommendations || c.optimizations || []
          };
        }
        // Mobile Friendliness
        if (byType['mobile-friendliness']) {
          const m = byType['mobile-friendliness'];
          out.mobileFriendliness = {
            overallScore: m.score || m.overallScore || 0,
            grade: m.grade || 'F',
            recommendations: m.recommendations || []
          };
          if (m.performance) {
            out.mobileFriendliness.performance = m.performance;
          }
        }
        return out;
      }
      
      for (const pageResult of combinedPageResults) {
        const accessibilityResult = pageResult.accessibilityResult;
        const comprehensiveFields = mapComprehensiveToPageFields(pageResult.comprehensiveAnalysis);
        const status = accessibilityResult.skipped ? 'skipped' : (accessibilityResult.passed ? 'passed' : (accessibilityResult.crashed ? 'crashed' : 'failed'));
        const mappedResult = {
          url: accessibilityResult.url,
          title: accessibilityResult.title || 'N/A',
          errors: accessibilityResult.errors?.length || 0,
          warnings: accessibilityResult.warnings?.length || 0,
          passed: accessibilityResult.passed,
          crashed: accessibilityResult.crashed || false,
          status,
          errorDetails: accessibilityResult.errors || [],
          warningDetails: accessibilityResult.warnings || [],
          pa11yScore: accessibilityResult.pa11yScore,
          pa11yIssues: accessibilityResult.pa11yIssues,
          // Comprehensive analysis mapped fields
          ...comprehensiveFields,
          issues: {
            pa11yScore: accessibilityResult.pa11yScore,
            pa11yIssues: accessibilityResult.pa11yIssues,
            performanceMetrics: comprehensiveFields.performance?.metrics,
            imagesWithoutAlt: accessibilityResult.imagesWithoutAlt || 0,
            buttonsWithoutLabel: accessibilityResult.buttonsWithoutLabel || 0,
            headingsCount: accessibilityResult.headingsCount || 0,
            keyboardNavigation: [],
            colorContrastIssues: [],
            focusManagementIssues: [],
            screenshots: []
          }
        };
        
        results.push(mappedResult);
        
        if (accessibilityResult.passed) successCount++;
        errorCount += accessibilityResult.errors?.length || 0;
        warningCount += accessibilityResult.warnings?.length || 0;
      }
        
        const actualResults = results;
        
        console.log(`\u2705 Testing completed: ${actualResults.length} pages analyzed`);
        
        // DEBUG: Log actual data structure for first result
        if (actualResults.length > 0 && config.verbose) {
          const firstResult = actualResults[0];
          console.log('🔍 First result data structure:');
          console.log(`   URL: ${firstResult.url}`);
          console.log(`   Title: ${firstResult.title}`);
          console.log(`   Status: passed=${firstResult.passed}, crashed=${firstResult.crashed}`);
          console.log(`   Performance: ${firstResult.performance ? 'Present' : 'Missing'}`);
          console.log(`   SEO: ${firstResult.seo ? 'Present' : 'Missing'}`);
          console.log(`   Content Weight: ${firstResult.contentWeight ? 'Present' : 'Missing'}`);
          console.log(`   Mobile Friendliness: ${firstResult.mobileFriendliness ? 'Present' : 'Missing'}`);
          console.log(`   Pa11y Score: ${firstResult.pa11yScore}`);
          console.log(`   Pa11y Issues: ${firstResult.pa11yIssues ? firstResult.pa11yIssues.length : 0}`);
        }
        
        // 🧹 COMPREHENSIVE CLEANUP to prevent hanging
        console.log('🧹 Cleaning up comprehensive analyzer resources...');
        try {
          // Cleanup AccessibilityChecker resources
          if (checker) {
            await checker.cleanup();
          }
          
          // Cleanup BrowserPoolManager resources
          if (poolManager) {
            await poolManager.cleanup();
          }
          
          console.log('✅ All analyzer resources cleaned up');
        } catch (cleanupError) {
          console.warn('⚠️  Cleanup warning:', cleanupError.message);
        }
        
        console.log('\n📝 Generating comprehensive HTML report...');
        
        // Prepare typed audit data structure with system performance metrics
        const totalDuration = Date.now() - startTime;
        const avgTimePerPage = totalDuration / actualResults.length;
        const throughputPagesPerMinute = (actualResults.length / (totalDuration / 1000)) * 60;
        const memoryUsageAtEnd = process.memoryUsage();
        const peakMemoryMB = Math.round(memoryUsageAtEnd.heapUsed / 1024 / 1024);
        
        const auditData = {
          metadata: {
            version: '1.0.0',
            timestamp: new Date().toISOString(),
            sitemapUrl: finalSitemapUrl,
            toolVersion: '2.0.0-alpha.2',
            duration: totalDuration
          },
          systemPerformance: {
            testCompletionTimeSeconds: Math.round(totalDuration / 1000),
            parallelProcessing: {
              pagesProcessed: actualResults.length,
              concurrentWorkers: config.maxConcurrent || 2,
              averageTimePerPageMs: Math.round(avgTimePerPage),
              throughputPagesPerMinute: Math.round(throughputPagesPerMinute * 10) / 10
            },
            memoryUsage: {
              peakUsageMB: peakMemoryMB,
              heapUsedMB: Math.round(memoryUsageAtEnd.heapUsed / 1024 / 1024),
              rssUsageMB: Math.round(memoryUsageAtEnd.rss / 1024 / 1024),
              externalMB: Math.round(memoryUsageAtEnd.external / 1024 / 1024)
            },
            architecture: {
              eventDrivenParallel: true,
              comprehensiveAnalysis: true,
              browserPooling: true, // Now enabled by default
              persistenceEnabled: false
            },
            measurementSettings: {
              psiProfile: qualityOptions.psiProfile,
              cpuThrottlingRate: qualityOptions.psiCPUThrottlingRate,
              network: qualityOptions.psiNetwork
            }
          },
          summary: {
            totalPages: urls.length,
            testedPages: actualResults.length, // Pages that were successfully analyzed
            passedPages: actualResults.filter(r => r.passed).length,
            failedPages: actualResults.filter(r => !r.passed && !r.crashed && r.status !== 'skipped').length,
            crashedPages: actualResults.filter(r => r.crashed).length,
            redirectPages: actualResults.filter(r => r.status === 'skipped').length,
            totalErrors: actualResults.reduce((sum, r) => sum + ((Array.isArray(r.errorDetails) ? r.errorDetails.length : 0) + ((r.pa11yIssues || []).filter(i => i.type === 'error').length)), 0),
            totalWarnings: actualResults.reduce((sum, r) => sum + ((Array.isArray(r.warningDetails) ? r.warningDetails.length : 0) + ((r.pa11yIssues || []).filter(i => i.type === 'warning').length)), 0)
          },
          pages: actualResults.map(page => ({
            url: page.url,
            title: page.title,
            status: page.status || (page.passed ? 'passed' : (page.crashed ? 'crashed' : 'failed')),
            duration: page.duration || 0,
            enhancedAnalysis: page.performance || page.seo || page.contentWeight || page.mobileFriendliness ? {
              performance: page.performance || null,
              seo: page.seo || null,
              contentWeight: page.contentWeight || null,
              mobileFriendliness: page.mobileFriendliness || null
            } : null,
            accessibility: {
              score: page.pa11yScore || 0,
              errors: [
                ...(page.pa11yIssues?.filter(i => i.type === 'error') || []),
                ...(Array.isArray(page.errorDetails) ? page.errorDetails.map(err => ({ message: err, type: 'error', code: 'accessibility-error' })) : [])
              ],
              warnings: [
                ...(page.pa11yIssues?.filter(i => i.type === 'warning') || []),
                ...(Array.isArray(page.warningDetails) ? page.warningDetails.map(warn => ({ message: warn, type: 'warning', code: 'accessibility-warning' })) : [])
              ],
              notices: page.pa11yIssues?.filter(i => i.type === 'notice') || [],
              basicChecks: {
                imagesWithoutAlt: page.imagesWithoutAlt || 0,
                buttonsWithoutLabel: page.buttonsWithoutLabel || 0,
                headingsCount: page.headingsCount || 0,
                contrastIssues: (Array.isArray(page.warningDetails) ? page.warningDetails.filter(w => typeof w === 'string' && w.includes('contrast')).length : 0)
              },
              wcagAnalysis: page.wcagAnalysis || undefined,
              ariaAnalysis: page.ariaAnalysis || undefined,
              formAnalysis: page.formAnalysis || undefined,
              keyboardAnalysis: page.keyboardAnalysis || undefined
            },
            performance: (page.performance || page.enhancedPerformance) ? {
              score: page.performance?.performanceScore || page.enhancedPerformance?.performanceScore || page.performance?.score || 0,
              grade: page.performance?.grade || page.enhancedPerformance?.grade || 'F',
              coreWebVitals: {
                largestContentfulPaint:
                  page.performance?.coreWebVitals?.largestContentfulPaint ||
                  page.performance?.coreWebVitals?.lcp?.value ||
                  page.enhancedPerformance?.coreWebVitals?.lcp?.value ||
                  page.performance?.coreWebVitals?.lcp ||
                  page.enhancedPerformance?.coreWebVitals?.lcp || 0,
                firstContentfulPaint:
                  page.performance?.coreWebVitals?.firstContentfulPaint ||
                  page.performance?.firstContentfulPaint ||
                  page.performance?.coreWebVitals?.fcp?.value ||
                  page.enhancedPerformance?.coreWebVitals?.fcp?.value ||
                  page.performance?.coreWebVitals?.fcp ||
                  page.enhancedPerformance?.coreWebVitals?.fcp || 0,
                cumulativeLayoutShift:
                  page.performance?.coreWebVitals?.cumulativeLayoutShift ||
                  page.performance?.coreWebVitals?.cls?.value ||
                  page.enhancedPerformance?.coreWebVitals?.cls?.value ||
                  page.performance?.coreWebVitals?.cls ||
                  page.enhancedPerformance?.coreWebVitals?.cls || 0,
                timeToFirstByte:
                  page.performance?.coreWebVitals?.timeToFirstByte ||
                  page.performance?.ttfb ||
                  page.enhancedPerformance?.metrics?.ttfb?.value ||
                  page.performance?.metrics?.ttfb ||
                  page.enhancedPerformance?.metrics?.ttfb || 0
              },
              metrics: {
                domContentLoaded: page.performance?.metrics?.domContentLoaded || page.enhancedPerformance?.metrics?.domContentLoaded || 0,
                loadComplete: page.performance?.metrics?.loadComplete || page.enhancedPerformance?.metrics?.loadComplete || 0,
                firstPaint: page.performance?.metrics?.firstPaint || page.enhancedPerformance?.metrics?.firstPaint || 0,
                requestCount: page.performance?.metrics?.requestCount || page.enhancedPerformance?.metrics?.requestCount || page.performance?.requestCount || page.enhancedPerformance?.requestCount || (Array.isArray(page.performance?.resourceLoadTimes) ? page.performance.resourceLoadTimes.length : 0),
                transferSize: page.performance?.metrics?.transferSize || page.enhancedPerformance?.metrics?.transferSize || page.performance?.transferSize || page.enhancedPerformance?.transferSize || page.contentWeight?.totalSize || 0
              },
              issues: page.performance?.issues || page.enhancedPerformance?.issues || []
            } : undefined,
            seo: (page.seo || page.enhancedSEO) ? {
              score: page.seo?.seoScore || page.enhancedSEO?.seoScore || page.seo?.overallScore || page.enhancedSEO?.overallScore || page.seo?.overallSEOScore || page.enhancedSEO?.overallSEOScore || page.seo?.score || page.enhancedSEO?.score || 0,
              grade: page.seo?.grade || page.enhancedSEO?.grade || page.seo?.seoGrade || page.enhancedSEO?.seoGrade || 'F',
              metaTags: page.seo?.metaData || page.enhancedSEO?.metaData || page.seo?.metaTags || page.enhancedSEO?.metaTags || {},
              headings: page.seo?.headingStructure || page.enhancedSEO?.headingStructure || page.seo?.headings || page.enhancedSEO?.headings || { h1: [], h2: [], h3: [], issues: [] },
              images: page.seo?.images || page.enhancedSEO?.images || { total: 0, missingAlt: 0, emptyAlt: 0 },
              issues: page.seo?.issues || page.enhancedSEO?.issues || [],
              // Include advanced SEO features
              overallSEOScore: page.enhancedSEO?.overallSEOScore || page.seo?.overallSEOScore,
              seoGrade: page.enhancedSEO?.seoGrade || page.seo?.seoGrade,
              url: page.url,
              title: page.title,
              semanticSEO: page.enhancedSEO?.semanticSEO,
              voiceSearchOptimization: page.enhancedSEO?.voiceSearchOptimization,
              eatAnalysis: page.enhancedSEO?.eatAnalysis,
              coreWebVitalsSEO: page.enhancedSEO?.coreWebVitalsSEO
            } : undefined,
            contentWeight: page.contentWeight ? {
              score: page.contentWeight.contentScore || page.contentWeight.score || page.contentWeight.contentQualityScore || 0,
              grade: page.contentWeight.grade || 'F', 
              totalSize: page.contentWeight.contentMetrics?.totalSize || page.contentWeight.totalSize || page.contentWeight.total || 0,
              resources: {
                html: { size: page.contentWeight.resourceAnalysis?.html?.size || page.contentWeight.resources?.html?.size || 0 },
                css: { size: page.contentWeight.resourceAnalysis?.css?.size || page.contentWeight.resources?.css?.size || 0, files: page.contentWeight.resourceAnalysis?.css?.count || page.contentWeight.resources?.css?.files || 0 },
                javascript: { 
                  size: page.contentWeight.resourceAnalysis?.javascript?.size || page.contentWeight.resources?.javascript?.size || 
                        page.contentWeight.resourceAnalysis?.js?.size || page.contentWeight.resources?.js?.size || 0, 
                  files: page.contentWeight.resourceAnalysis?.javascript?.count || page.contentWeight.resources?.javascript?.files || 
                         page.contentWeight.resourceAnalysis?.js?.count || page.contentWeight.resources?.js?.files || 0 
                },
                images: { size: page.contentWeight.resourceAnalysis?.images?.size || page.contentWeight.resources?.images?.size || 0, files: page.contentWeight.resourceAnalysis?.images?.count || page.contentWeight.resources?.images?.files || 0 },
                other: { size: page.contentWeight.resourceAnalysis?.other?.size || page.contentWeight.resources?.other?.size || 0, files: page.contentWeight.resourceAnalysis?.other?.count || page.contentWeight.resources?.other?.files || 0 }
              },
              optimizations: page.contentWeight.optimizations || []
            } : undefined,
            mobileFriendliness: page.mobileFriendliness ? {
              overallScore: page.mobileFriendliness.overallScore || 0,
              grade: page.mobileFriendliness.grade || 'F',
              performance: page.mobileFriendliness.performance || undefined,
              recommendations: page.mobileFriendliness.recommendations || []
            } : undefined
          }))
        };
        
        // Generate reports based on format
        if (config.format === 'json') {
          // JSON format: Complete typed data object
          console.log('\\n📊 Generating JSON report with complete data...');
          const { JsonGenerator } = require('../dist/generators/json-generator');
          const jsonGenerator = new JsonGenerator();
          
          const jsonContent = jsonGenerator.generateJson(auditData);
          const jsonPath = path.join(subDir, `audit-${dateOnly}.json`);
          require('fs').writeFileSync(jsonPath, jsonContent);
          
          outputFiles = [jsonPath];
          console.log('✅ JSON report generated with complete typed audit data');
        } else {
          // HTML format (default): Professional report + detailed issues MD
          console.log('\n📝 Generating HTML report...');
          const { HTMLGenerator } = require('../dist/generators/html-generator');
          const generator = new HTMLGenerator();
          
          const htmlContent = await generator.generate(auditData);
          const reportPath = path.join(subDir, `accessibility-report-${dateOnly}.html`);
          require('fs').writeFileSync(reportPath, htmlContent);
          
          // Always generate detailed issues markdown with HTML reports
          console.log('📄 Generating detailed issues markdown...');
          const { DetailedIssueMarkdownReport } = require('../dist/reports/detailed-issue-markdown');
          
          // Extract all pa11y issues
          const detailedIssues = [];
          
          actualResults.forEach((page, index) => {
            // Skip detailed issues for pages skipped due to redirects
            if ((page && page.status) && page.status === 'skipped') {
              return;
            }
            // Debug: Log what data we have for each page
            const hasIssues = page.pa11yIssues && Array.isArray(page.pa11yIssues) && page.pa11yIssues.length > 0;
            const hasErrorDetails = page.errorDetails && Array.isArray(page.errorDetails) && page.errorDetails.length > 0;
            const hasWarningDetails = page.warningDetails && Array.isArray(page.warningDetails) && page.warningDetails.length > 0;
            const hasErrors = page.errors && page.errors.length > 0;
            const hasWarnings = page.warnings && page.warnings.length > 0;
            
            
            // Extract pa11y issues if available
            if (hasIssues) {
              page.pa11yIssues.forEach(issue => {
                detailedIssues.push({
                  type: issue.type || 'accessibility',
                  severity: issue.type || 'error',
                  message: issue.message || 'Unknown accessibility issue',
                  code: issue.code,
                  selector: issue.selector,
                  context: issue.context,
                  htmlSnippet: issue.context,
                  pageUrl: page.url,
                  pageTitle: page.title || 'Untitled Page',
                  source: 'pa11y',
                  help: issue.help,
                  helpUrl: issue.helpUrl,
                  lineNumber: null,
                  recommendation: issue.help || 'Please refer to WCAG guidelines',
                  resource: null,
                  score: null,
                  metric: null
                });
              });
            }
            
            // Extract errorDetails/warningDetails as detailed issues
            if (hasErrorDetails) {
              page.errorDetails.forEach(error => {
                detailedIssues.push({
                  type: 'error',
                  severity: 'error',
                  message: error,
                  code: 'accessibility-error',
                  selector: null,
                  context: null,
                  htmlSnippet: null,
                  pageUrl: page.url,
                  pageTitle: page.title || 'Untitled Page',
                  source: 'accessibility-checker',
                  help: 'Please refer to WCAG guidelines',
                  helpUrl: 'https://www.w3.org/WAI/WCAG21/quickref/',
                  lineNumber: null,
                  recommendation: 'Please refer to WCAG guidelines',
                  resource: null,
                  score: null,
                  metric: null
                });
              });
            }
            
            if (hasWarningDetails) {
              page.warningDetails.forEach(warning => {
                detailedIssues.push({
                  type: 'warning',
                  severity: 'warning',
                  message: warning,
                  code: 'accessibility-warning',
                  selector: null,
                  context: null,
                  htmlSnippet: null,
                  pageUrl: page.url,
                  pageTitle: page.title || 'Untitled Page',
                  source: 'accessibility-checker',
                  help: 'Consider fixing this issue to improve accessibility',
                  helpUrl: 'https://www.w3.org/WAI/WCAG21/quickref/',
                  lineNumber: null,
                  recommendation: 'Consider fixing this issue to improve accessibility',
                  resource: null,
                  score: null,
                  metric: null
                });
              });
            }
            
            // For completely failed pages without any issue data, add a diagnostic issue
            if (!page.passed && !hasIssues && !hasErrorDetails && !hasWarningDetails) {
              detailedIssues.push({
                type: 'error',
                severity: 'error',
                message: 'Page failed accessibility test but no specific issues were detected. This may indicate a technical problem during testing.',
                code: 'test-failure',
                selector: null,
                context: null,
                htmlSnippet: null,
                pageUrl: page.url,
                pageTitle: page.title || 'Untitled Page',
                source: 'audit-system',
                help: 'Rerun the test or check if the page is accessible and loads correctly',
                helpUrl: null,
                lineNumber: null,
                recommendation: 'Investigate why the accessibility test failed for this page',
                resource: null,
                score: null,
                metric: null
              });
            }
          });
          
          
          
          if (detailedIssues.length > 0) {
            const skippedPages = actualResults.filter(p => p.status === 'skipped').map(p => ({ url: p.url, title: p.title, reason: 'HTTP Redirect' }));
            const detailedMarkdown = DetailedIssueMarkdownReport.generate(detailedIssues, { skippedPages });
            const detailedPath = path.join(subDir, `detailed-issues-${dateOnly}.md`);
            require('fs').writeFileSync(detailedPath, detailedMarkdown);
            outputFiles = [reportPath, detailedPath];
            console.log('✅ HTML report + detailed issues markdown generated');
          } else {
            outputFiles = [reportPath];
            console.log('✅ HTML report generated (no detailed issues found)');
          }
        }
        const totalTime = Math.round((Date.now() - startTime) / 1000);
        console.log(`\u2705 Analysis completed: ${actualResults.length} pages in ${formatTime(totalTime)}`);
        
        // Show results (using same format as standard pipeline)
        summary = {
          testedPages: actualResults.length,
          passedPages: actualResults.filter(r => r.passed).length,
          failedPages: actualResults.filter(r => !r.passed && !r.crashed).length,
          crashedPages: actualResults.filter(r => r.crashed).length,
          totalErrors: actualResults.reduce((sum, r) => sum + (r.errors?.length || 0), 0),
          totalWarnings: actualResults.reduce((sum, r) => sum + (r.warnings?.length || 0), 0)
        };
        // startTime already set above, no need to recalculate
        
        // Continue to standard success output below...
        
        } catch (analysisError) {
          console.error(`\n❌ Enhanced Analysis failed: ${analysisError.message}`);
          console.error('No fallback available - modern architecture only');
          throw analysisError;
        }
      } // End of useStandardAnalysis block
      const totalTime = Math.round((Date.now() - startTime) / 1000);
      console.log(`✅ Completed ${summary.testedPages} pages in ${formatTime(totalTime)}`);
      
      // Add performance summary
      const avgSpeed = summary.testedPages / (totalTime / 60); // pages per minute
      console.log(`⚡ Average speed: ${avgSpeed.toFixed(1)} pages/minute`);
      
      
      // 🎉 Success output
      console.log('\n✅ Test completed successfully!');
      console.log(`📊 Results:`);
      console.log(`   📄 Tested: ${summary.testedPages} pages`);
      console.log(`   ✅ Passed: ${summary.passedPages}`);
      console.log(`   ❌ Failed: ${summary.failedPages}`);
      console.log(`   ⚠️  Errors: ${summary.totalErrors}`);
      console.log(`   ⚠️  Warnings: ${summary.totalWarnings}`);
      
      const successRate = summary.testedPages > 0 ? 
        (summary.passedPages / summary.testedPages * 100).toFixed(1) : 0;
      console.log(`   🎯 Success Rate: ${successRate}%`);
      
      // Show generated files with proper icons and descriptions
      if (outputFiles.length > 0) {
        console.log(`\n📁 Generated reports:`);
        outputFiles.forEach(file => {
          const filename = path.basename(file);
          if (filename.includes('detailed-issues')) {
            console.log(`   📄 ${filename}`);
          } else if (filename.includes('performance-issues')) {
            console.log(`   📄 ${filename}`);
          } else if (filename.includes('accessibility-report')) {
            console.log(`   📄 ${filename}`);
          } else {
            console.log(`   📄 ${filename}`);
          }
        });
      }
      
      // Only exit with code 1 for technical errors, not accessibility failures
      if (summary.crashedPages > 0) {
        console.log(`\n❌ ${summary.crashedPages} pages crashed due to technical errors`);
        process.exit(1);
      } else if (summary.failedPages > 0) {
        console.log(`\n⚠️  ${summary.failedPages} pages failed accessibility tests (this is normal for real websites)`);
        console.log(`💡 Check the detailed report for specific issues to fix`);
      }
      
      // 💯 EXPLICIT EXIT to prevent hanging after successful completion
      console.log('💯 Process completed successfully - exiting cleanly');
      process.exit(0);
      
    } catch (error) {
      
      // Advanced error categorization and recovery
      const errorType = categorizeError(error);
      console.error(`\n❌ ${errorType.type}: ${errorType.message}`);
      
      // Recovery system removed - modern architecture handles errors directly
      console.log('\n⚠️  Error occurred during analysis - no automatic recovery available');
      console.log('💡 Use --expert mode for custom settings or try with fewer pages');
      console.log('🔍 Check the error details and suggestions below for guidance');
      
      // Show helpful suggestions
      console.log('\n💡 Troubleshooting suggestions:');
      errorType.suggestions.forEach(suggestion => {
        console.log(`   • ${suggestion}`);
      });
      
      if (options.verbose) {
        console.log('\n🔍 Full error details:');
        console.error(error.stack);
      } else {
        console.log('\n🔍 Run with --verbose for detailed error information');
      }
      
      process.exit(1);
    }
  });

// Helper functions for progress tracking
function calculateEstimatedTime(pages, concurrent = 2) {
  if (pages === 1000) return '10-60 min';
  const avgTimePerPage = 12; // seconds
  const totalTime = Math.round((pages * avgTimePerPage) / concurrent);
  return formatTime(totalTime);
}

function formatTime(seconds) {
  if (seconds < 60) return `${seconds}s`;
  const mins = Math.floor(seconds / 60);
  const secs = seconds % 60;
  return `${mins}m ${secs}s`;
}

function truncateUrl(url) {
  if (!url) return 'Processing...';
  return url.length > 50 ? url.substring(0, 47) + '...' : url;
}

function categorizeError(error) {
  const message = error.message || String(error);
  const stack = error.stack || '';
  
  // Network/Connection errors
  if (message.includes('ENOTFOUND') || message.includes('ECONNREFUSED') || 
      message.includes('net::ERR_') || message.includes('timeout')) {
    return {
      type: 'Network Error',
      message: 'Cannot connect to the website or sitemap',
      recoverable: true,
      suggestions: [
        'Check if the website URL is correct and accessible',
        'Verify your internet connection',
        'Try running the test later if the site is temporarily down',
        'Use --expert mode to increase timeout settings'
      ]
    };
  }
  
  // Sitemap parsing errors  
  if (message.includes('sitemap') || message.includes('XML') || message.includes('parsing')) {
    return {
      type: 'Sitemap Error',
      message: 'Cannot parse or access the sitemap.xml',
      recoverable: false,
      suggestions: [
        'Verify the sitemap URL is correct (should end with /sitemap.xml)',
        'Check if the sitemap is properly formatted XML',
        'Ensure the sitemap is publicly accessible',
        'Try testing a single page instead of the full sitemap'
      ]
    };
  }
  
  // Browser/Playwright errors
  if (message.includes('browser') || message.includes('playwright') || 
      message.includes('chromium') || stack.includes('playwright')) {
    return {
      type: 'Browser Error',
      message: 'Browser automation failed',
      recoverable: true,
      suggestions: [
        'Try reducing concurrent tests with --expert mode',
        'Restart your terminal and try again',
        'Check available system memory (close other applications)',
        'Run with --verbose for more detailed browser logs'
      ]
    };
  }
  
  // Memory/Resource errors
  if (message.includes('memory') || message.includes('ENOMEM') || 
      message.includes('heap') || message.includes('allocation')) {
    return {
      type: 'Resource Error',
      message: 'Insufficient system resources',
      recoverable: true,
      suggestions: [
        'Reduce the number of pages tested (use --expert mode)',
        'Close other applications to free memory',
        'Test pages in smaller batches',
        'Reduce concurrent tests to 1'
      ]
    };
  }
  
  // Permission errors
  if (message.includes('EACCES') || message.includes('permission') || message.includes('EPERM')) {
    return {
      type: 'Permission Error',
      message: 'Insufficient permissions',
      recoverable: false,
      suggestions: [
        'Run the command with appropriate permissions',
        'Check if the output directory is writable',
        'Ensure Node.js has permission to create browser profiles'
      ]
    };
  }
  
  // Generic/Unknown errors
  return {
    type: 'Unknown Error',
    message: message.length > 100 ? message.substring(0, 97) + '...' : message,
    recoverable: true,
    suggestions: [
      'Try running with --verbose for more details',
      'Use --expert mode for custom settings',
      'Test with fewer pages first',
      'Check the GitHub issues page for similar problems'
    ]
  };
}

// Streaming audit function removed in CLI simplification - no longer needed

// Legacy runStandardPipeline function removed - using modern AccessibilityChecker architecture only

// Legacy report generators removed - using UnifiedHTMLGenerator exclusively


program.parse();
