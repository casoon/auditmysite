#!/usr/bin/env node

const { Command } = require('commander');
const { StandardPipeline } = require('../dist/core');
const { SitemapDiscovery } = require('../dist/core/parsers');
const inquirer = require('inquirer').default;
const path = require('path');
const ora = require('ora').default || require('ora');
const packageJson = require('../package.json');

const program = new Command();

// 🎯 SIMPLIFIED CLI - Only 6 essential options!
program
  .name('auditmysite')
  .description('🎯 Simple accessibility testing - just works!')
  .version(packageJson.version)
  .argument('<sitemapUrl>', 'URL of the sitemap.xml to test')
  
  // ✅ Only these 7 ESSENTIAL options:
  .option('--full', 'Test all pages instead of just 5 (default: 5 pages)')
  .option('--max-pages <number>', 'Maximum number of pages to test (overrides --full)', (value) => parseInt(value))
  .option('--expert', 'Interactive expert mode with custom settings')
  .option('--format <type>', 'Report format: html or markdown', 'html')
  .option('--output-dir <dir>', 'Output directory for reports', './reports')
  .option('--non-interactive', 'Skip prompts for CI/CD (use defaults)')
  .option('-v, --verbose', 'Show detailed progress information')
  
  // 📊 Performance Budget Options
  .option('--budget <template>', 'Performance budget template: ecommerce, blog, corporate, or default', 'default')
  .option('--lcp-budget <ms>', 'Custom LCP budget in milliseconds (good threshold)')
  .option('--cls-budget <score>', 'Custom CLS budget score (good threshold)')
  .option('--fcp-budget <ms>', 'Custom FCP budget in milliseconds (good threshold)')
  .option('--inp-budget <ms>', 'Custom INP budget in milliseconds (good threshold)')
  .option('--ttfb-budget <ms>', 'Custom TTFB budget in milliseconds (good threshold)')
  
// 🚀 Tauri Integration Options
  .option('--stream', 'Enable streaming output for desktop integration')
  .option('--session-id <id>', 'Session ID for tracking (required with --stream)')
  .option('--chunk-size <size>', 'Chunk size for large reports', '1000')
  
  // 🔧 NEW: Unified Queue System Options
  .option('--unified-queue', 'Use the new unified queue system (EXPERIMENTAL)')
  
  // 🎯 Analysis Features - all included by default
  .option('--no-performance', 'Disable performance analysis')
  .option('--no-seo', 'Disable SEO analysis')
  .option('--no-content-weight', 'Disable content weight analysis')
  .option('--no-mobile', 'Disable mobile-friendliness analysis')
  
  .action(async (sitemapUrl, options) => {
    // 🚀 Tauri Integration: Streaming Mode
    if (options.stream) {
      if (!options.sessionId) {
        console.error('Error: --session-id is required when using --stream mode');
        process.exit(1);
      }
      await runStreamingAudit(sitemapUrl, options);
      return;
    }
    
    console.log(`🚀 AuditMySite v${packageJson.version} - Professional Accessibility Testing`);
    console.log(`📄 Sitemap: ${sitemapUrl}`);
    
    // 🎯 SMART DEFAULTS - All analysis features are standard!
    const QUICK_DEFAULTS = {
      maxPages: options.maxPages || (options.full ? 1000 : 5),
      standard: 'WCAG2AA',
      format: options.format || 'html',
      outputDir: options.outputDir || './reports',
      timeout: 10000,
      maxConcurrent: 2,
      generateDetailedReport: true,
      generatePerformanceReport: true,
      generateSeoReport: false,        // ❌ Removed 
      generateSecurityReport: false,   // ❌ Removed
      usePa11y: true,
      lighthouse: false,               // ❌ Removed Lighthouse
      captureScreenshots: false,      // ❌ Removed
      verbose: options.verbose || false,
      // 🚀 All Analysis Features are STANDARD
      performanceAnalysis: true,       // ✅ Performance metrics standard
      seoAnalysis: true,               // ✅ SEO analysis standard
      contentWeight: true,             // ✅ Content weight standard
      mobileFriendliness: true         // ✅ Mobile-friendliness standard
    };
    
    let config = { ...QUICK_DEFAULTS };
    
    // 🚀 Override Analysis Features from CLI arguments
    if (options.noPerformance) {
      config.performanceAnalysis = false;
    }
    if (options.noSeo) {
      config.seoAnalysis = false;
    }
    if (options.noContentWeight) {
      config.contentWeight = false;
    }
    if (options.noMobile) {
      config.mobileFriendliness = false;
    }
    // Note: All features enabled by default unless explicitly disabled
    
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
            { name: '🌐 HTML - Professional reports for stakeholders', value: 'html' },
            { name: '📝 Markdown - Developer-friendly, version control', value: 'markdown' }
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
          type: 'confirm',
          name: 'modernHtml5',
          message: '🔥 Enable modern HTML5 elements testing (details, dialog, semantic)?',
          default: true
        },
        {
          type: 'confirm',
          name: 'ariaAdvanced',
          message: '⚡ Enable advanced ARIA analysis with impact scoring?',
          default: true
        },
        {
          type: 'confirm',
          name: 'chrome135Features',
          message: '🚀 Enable Chrome 135 specific features and optimizations?',
          default: true
        },
        {
          type: 'confirm',
          name: 'semanticAnalysis',
          message: '📊 Enable semantic structure analysis and recommendations?',
          default: true
        },
        {
          type: 'confirm',
          name: 'captureScreenshots',
          message: '📸 Capture desktop and mobile screenshots of pages?',
          default: false
        },
        {
          type: 'confirm',
          name: 'testKeyboardNavigation',
          message: '⌨️  Test keyboard navigation and focusable elements?',
          default: false
        },
        {
          type: 'confirm',
          name: 'testColorContrast',
          message: '🎨 Test color contrast ratios (basic analysis)?',
          default: false
        },
        {
          type: 'confirm',
          name: 'testFocusManagement',
          message: '🎯 Test focus management and indicators?',
          default: false
        },
        {
          type: 'confirm',
          name: 'allFeatures',
          message: '🚀 Keep all analysis features enabled? (Performance, SEO, Content Weight)',
          default: true
        },
        {
          type: 'checkbox',
          name: 'analysisComponents',
          message: '🔍 Which analysis components? (select multiple)',
          choices: [
            { name: '⚡ Performance - Core Web Vitals, advanced metrics', value: 'performance' },
            { name: '🔍 SEO - Meta tags, content quality, readability', value: 'seo' },
            { name: '📏 Content Weight - Resource analysis, text-to-code ratio', value: 'contentWeight' }
          ],
          when: (answers) => answers.allFeatures,
          validate: (answer) => {
            if (answer.length === 0) {
              return 'Please select at least one analysis component';
            }
            return true;
          }
        },
        {
          type: 'list',
          name: 'budgetTemplate',
          message: '📈 Performance budget template?',
          choices: [
            { name: '🏢 Corporate - Professional standards (stricter thresholds)', value: 'corporate' },
            { name: '🏬 E-commerce - Conversion-focused (very strict for revenue)', value: 'ecommerce' },
            { name: '📝 Blog - Content-focused (standard Google thresholds)', value: 'blog' },
            { name: '⚙️ Default - Google Web Vitals standard thresholds', value: 'default' },
            { name: '🔧 Custom - Set individual thresholds manually', value: 'custom' }
          ],
          default: 'default'
        }
      ]);
      
      // If custom budget selected, ask for individual thresholds
      if (answers.budgetTemplate === 'custom') {
        const customBudget = await inquirer.prompt([
          {
            type: 'number',
            name: 'lcpBudget',
            message: '📈 LCP (Largest Contentful Paint) good threshold in ms?',
            default: 2500,
            validate: (value) => value > 0 && value < 10000 ? true : 'Please enter a value between 0 and 10000ms'
          },
          {
            type: 'input',
            name: 'clsBudget',
            message: '📈 CLS (Cumulative Layout Shift) good threshold (e.g. 0.1)?',
            default: '0.1',
            validate: (value) => {
              const num = parseFloat(value);
              return num >= 0 && num <= 1 ? true : 'Please enter a value between 0 and 1';
            }
          },
          {
            type: 'number',
            name: 'fcpBudget',
            message: '📈 FCP (First Contentful Paint) good threshold in ms?',
            default: 1800,
            validate: (value) => value > 0 && value < 10000 ? true : 'Please enter a value between 0 and 10000ms'
          },
          {
            type: 'number',
            name: 'inpBudget',
            message: '📈 INP (Interaction to Next Paint) good threshold in ms?',
            default: 200,
            validate: (value) => value >= 0 && value < 5000 ? true : 'Please enter a value between 0 and 5000ms'
          },
          {
            type: 'number',
            name: 'ttfbBudget',
            message: '📈 TTFB (Time to First Byte) good threshold in ms?',
            default: 400,
            validate: (value) => value > 0 && value < 5000 ? true : 'Please enter a value between 0 and 5000ms'
          }
        ]);
        
        answers.customBudgetValues = {
          lcp: customBudget.lcpBudget,
          cls: parseFloat(customBudget.clsBudget),
          fcp: customBudget.fcpBudget,
          inp: customBudget.inpBudget,
          ttfb: customBudget.ttfbBudget
        };
      }
      
      config = { ...config, ...answers };
    }
    
    // 🐎 Create performance budget
    const { BUDGET_TEMPLATES } = require('../dist/core/performance/web-vitals-collector');
    let performanceBudget;
    
    // Priority: CLI options > Expert mode > Default template
    if (options.lcpBudget || options.clsBudget || options.fcpBudget || options.inpBudget || options.ttfbBudget) {
      // Custom CLI budget
      const defaultBudget = BUDGET_TEMPLATES[options.budget || 'default'];
      performanceBudget = {
        lcp: { 
          good: parseInt(options.lcpBudget) || defaultBudget.lcp.good, 
          poor: (parseInt(options.lcpBudget) || defaultBudget.lcp.good) * 1.6 
        },
        cls: { 
          good: parseFloat(options.clsBudget) || defaultBudget.cls.good, 
          poor: (parseFloat(options.clsBudget) || defaultBudget.cls.good) * 2.5 
        },
        fcp: { 
          good: parseInt(options.fcpBudget) || defaultBudget.fcp.good, 
          poor: (parseInt(options.fcpBudget) || defaultBudget.fcp.good) * 1.7 
        },
        inp: { 
          good: parseInt(options.inpBudget) || defaultBudget.inp.good, 
          poor: (parseInt(options.inpBudget) || defaultBudget.inp.good) * 2.5 
        },
        ttfb: { 
          good: parseInt(options.ttfbBudget) || defaultBudget.ttfb.good, 
          poor: (parseInt(options.ttfbBudget) || defaultBudget.ttfb.good) * 2 
        }
      };
    } else if (config.budgetTemplate === 'custom' && config.customBudgetValues) {
      // Expert mode custom budget
      const custom = config.customBudgetValues;
      performanceBudget = {
        lcp: { good: custom.lcp, poor: custom.lcp * 1.6 },
        cls: { good: custom.cls, poor: custom.cls * 2.5 },
        fcp: { good: custom.fcp, poor: custom.fcp * 1.7 },
        inp: { good: custom.inp, poor: custom.inp * 2.5 },
        ttfb: { good: custom.ttfb, poor: custom.ttfb * 2 }
      };
    } else {
      // Template budget
      const template = config.budgetTemplate || options.budget || 'default';
      performanceBudget = BUDGET_TEMPLATES[template] || BUDGET_TEMPLATES.default;
    }
    
    // 📈 Show configuration
    console.log(`\\n📋 Configuration:`);
    console.log(`   📄 Pages: ${config.maxPages === 1000 ? 'All' : config.maxPages}`);
    console.log(`   📋 Standard: ${config.standard}`);
    console.log(`   📈 Basic Performance: ${config.generatePerformanceReport ? 'Yes' : 'No'}`);
    console.log(`   📈 Budget: ${config.budgetTemplate || options.budget || 'default'} (LCP: ${performanceBudget.lcp.good}ms, CLS: ${performanceBudget.cls.good})`);
    console.log(`   📄 Format: ${config.format.toUpperCase()}`);
    console.log(`   📁 Output: ${config.outputDir}`);
    
    // Analysis Features Summary
    console.log('\\n🚀 Analysis Features:');
    if (config.performanceAnalysis) {
      console.log('   ⚡ Performance: ✅ Core Web Vitals, loading metrics');
    }
    if (config.seoAnalysis) {
      console.log('   🔍 SEO: ✅ Meta tags, content quality, readability');
    }
    if (config.contentWeight) {
      console.log('   📏 Content Weight: ✅ Resource analysis, optimization');
    }
    if (config.mobileFriendliness) {
      console.log('   📱 Mobile-Friendliness: ✅ Responsive design, touch targets');
    }
    
    // Declare variables in outer scope for error handling
    let pipelineOptions;
    let pipeline;
    let summary;
    let outputFiles;
    let startTime;
    
    try {
      // Extract domain for report organization
      const url = new URL(sitemapUrl);
      const domain = url.hostname.replace(/\\./g, '-');
      const dateOnly = new Date().toLocaleDateString('en-CA');
      
      // Create domain subdirectory
      const fs = require('fs');
      const subDir = path.join(config.outputDir, domain);
      if (!fs.existsSync(subDir)) {
        fs.mkdirSync(subDir, { recursive: true });
      }
      
      // 🚀 Run the pipeline with simplified options
      pipeline = new StandardPipeline();
      
      pipelineOptions = {
        sitemapUrl,
        maxPages: config.maxPages,
        timeout: config.timeout,
        pa11yStandard: config.standard,
        outputDir: subDir,
        generateDetailedReport: config.generateDetailedReport,
        generatePerformanceReport: config.generatePerformanceReport,
        generateSeoReport: false,           // ❌ Always false now
        generateSecurityReport: false,      // ❌ Always false now
        outputFormat: config.format,
        maxConcurrent: config.maxConcurrent,
        verbose: config.verbose,
        timestamp: new Date().toISOString(),
        // 🆕 Performance-Metriken aktivieren
        collectPerformanceMetrics: true,    // ✅ Web Vitals immer aktiviert
        captureScreenshots: config.captureScreenshots || false,
        testKeyboardNavigation: config.testKeyboardNavigation || false,
        testColorContrast: config.testColorContrast || false,
        testFocusManagement: config.testFocusManagement || false,
        
        // 🔥 Advanced v2.0 Features  
        modernHtml5: config.modernHtml5 !== undefined ? config.modernHtml5 : true,
        ariaAdvanced: config.ariaAdvanced !== undefined ? config.ariaAdvanced : true,
        chrome135Features: config.chrome135Features !== undefined ? config.chrome135Features : true,
        semanticAnalysis: config.semanticAnalysis !== undefined ? config.semanticAnalysis : true,
        
        // 🔧 NEW: Unified Queue System
        useUnifiedQueue: options.unifiedQueue || false,
        
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
        console.log('\\n🚀 Starting accessibility analysis...');
        
        try {
          // Use main accessibility analysis pipeline
          const { MainAccessibilityChecker } = require('../dist/accessibility-checker-main');
          const { SitemapParser } = require('../dist/parsers/sitemap-parser');
        
        // Parse sitemap
        const parser = new SitemapParser();
        const urls = await parser.parseSitemap(finalSitemapUrl);
        const limitedUrls = urls.slice(0, config.maxPages || 5);
        
        console.log(`📈 Found ${urls.length} URLs in sitemap, testing ${limitedUrls.length}`);
        
        // Initialize Main Accessibility Checker with all features
        const checker = new MainAccessibilityChecker({
          includeResourceAnalysis: (config.analysisComponents && config.analysisComponents.includes('contentWeight')) || config.contentWeight,
          includeSocialAnalysis: (config.analysisComponents && config.analysisComponents.includes('seo')) || config.seoAnalysis,
          includeReadabilityAnalysis: (config.analysisComponents && config.analysisComponents.includes('seo')) || config.seoAnalysis,
          includeTechnicalSEO: (config.analysisComponents && config.analysisComponents.includes('seo')) || config.seoAnalysis,
          includeMobileFriendliness: config.mobileFriendliness,
          analysisTimeout: 30000
        });
        
        await checker.initialize();
        console.log('✨ Accessibility analyzer initialized');
        
        const results = [];
        let successCount = 0;
        let errorCount = 0;
        let warningCount = 0;
        startTime = Date.now(); // Use outer scope variable
        
        // Process each URL
        for (let i = 0; i < limitedUrls.length; i++) {
          const urlObj = limitedUrls[i];
          const url = typeof urlObj === 'string' ? urlObj : urlObj.loc;
          const spinner = ora(`[${i + 1}/${limitedUrls.length}] Analyzing ${url}`).start();
          
          try {
            const result = await checker.analyze('', url);
            
            results.push({
              url: url,
              title: result.title || 'N/A',
              errors: result.errors?.length || 0,
              warnings: result.warnings?.length || 0,
              passed: result.passed,
              // Store actual error/warning arrays for detailed issues
              errorDetails: result.errors || [],
              warningDetails: result.warnings || [],
              performance: result.performance,
              seo: result.seo,
              contentWeight: result.contentWeight,
              mobileFriendliness: result.mobileFriendliness, // Add Mobile-Friendliness data
              qualityScore: result.qualityScore
            });
            
            if (result.passed) successCount++;
            errorCount += result.errors?.length || 0;
            warningCount += result.warnings?.length || 0;
            
            // Show analysis metrics for this page
            let statusText = result.passed ? '✅ Passed' : '❌ Failed';
            if (result.qualityScore) {
              statusText += ` (Quality: ${result.qualityScore.score}/100 ${result.qualityScore.grade})`;
            }
            spinner.succeed(statusText);
            
          } catch (error) {
            spinner.fail(`Failed: ${error.message}`);
            results.push({
              url: url,
              title: 'Error',
              errors: 1,
              warnings: 0,
              passed: false,
              crashed: true
            });
            errorCount++;
          }
        }
        
        // Cleanup
        await checker.cleanup();
        
        // Generate standard HTML report using the normal report generator
        const { generateHtmlReport } = require('../dist/reports/html-report');
        
        const reportData = {
          summary: {
            totalPages: urls.length,
            testedPages: results.length,
            passedPages: successCount,
            failedPages: results.length - successCount,
            crashedPages: results.filter(r => r.crashed).length,
            totalErrors: errorCount,
            totalWarnings: warningCount,
            totalDuration: Date.now() - startTime
          },
          pages: results,
          metadata: {
            timestamp: new Date().toLocaleString(),
            sitemapUrl: finalSitemapUrl
          }
        };
        
        const htmlContent = generateHtmlReport(reportData);
        
        const reportPath = path.join(subDir, `accessibility-report-${dateOnly}.html`);
        require('fs').writeFileSync(reportPath, htmlContent);
        
        const totalTime = Math.round((Date.now() - startTime) / 1000);
        console.log(`✅ Analysis completed: ${results.length} pages in ${formatTime(totalTime)}`);
        
        // Show results (using same format as standard pipeline)
        summary = {
          testedPages: results.length,
          passedPages: successCount,
          failedPages: results.length - successCount,
          crashedPages: results.filter(r => r.crashed).length,
          totalErrors: errorCount,
          totalWarnings: warningCount
        };
        outputFiles = [reportPath];
        // startTime already set above, no need to recalculate
        
        // Continue to standard success output below...
        
        } catch (analysisError) {
          console.error(`\\n⚠️  Analysis failed: ${analysisError.message}`);
          console.log('🔄 Falling back to basic accessibility analysis...');
          
          // Fallback to standard pipeline
          return await runStandardPipeline(finalSitemapUrl, config, pipelineOptions, pipeline);
        }
        
      } else {
        // Use standard pipeline
        const standardResult = await runStandardPipeline(finalSitemapUrl, config, pipelineOptions, pipeline);
        summary = standardResult.summary;
        outputFiles = standardResult.outputFiles;
        startTime = Date.now() - (standardResult.totalTime * 1000); // Reconstruct startTime
      }
      
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
        // Exit with 0 for accessibility failures - this is expected behavior
      }
      
    } catch (error) {
      
      // Advanced error categorization and recovery
      const errorType = categorizeError(error);
      console.error(`\n❌ ${errorType.type}: ${errorType.message}`);
      
      if (errorType.recoverable && !options.nonInteractive) {
        console.log('\n🔄 Attempting automatic recovery...');
        
        try {
          // Try with safer options
          console.log('🔄 Retrying with conservative settings...');
          
          const saferOptions = {
            ...(pipelineOptions || {}),
            maxConcurrent: 1,
            timeout: 20000,
            collectPerformanceMetrics: false,
            maxPages: Math.min((pipelineOptions?.maxPages || 10), 3)
          };
          
          // Ensure pipeline is initialized
          if (!pipeline) {
            pipeline = new StandardPipeline();
          }
          
          const { summary, outputFiles } = await pipeline.run(saferOptions);
          
          console.log('✅ Recovery successful with limited scope');
          console.log('⚠️  Note: Test completed with reduced scope due to initial error');
          
          // Continue with success output but warn user
          const successRate = summary.testedPages > 0 ? 
            (summary.passedPages / summary.testedPages * 100).toFixed(1) : 0;
          
          console.log(`\n📊 Partial Results:`);
          console.log(`   📄 Tested: ${summary.testedPages} pages (reduced from ${pipelineOptions?.maxPages || 'unknown'})`);
          console.log(`   ✅ Passed: ${summary.passedPages}`);
          console.log(`   ❌ Failed: ${summary.failedPages}`);
          console.log(`   ⚠️  Success Rate: ${successRate}%`);
          
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
          
          console.log('\n💡 Recommendation: Try running with --expert mode for more control');
          // Only exit with code 1 for technical crashes, not accessibility failures
          process.exit(summary.crashedPages > 0 ? 1 : 0);
          
        } catch (recoveryError) {
          console.error('❌ Recovery attempt failed:', categorizeError(recoveryError).message);
        }
      }
      
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

// 🚀 Streaming Audit Function for Tauri Integration
async function runStreamingAudit(sitemapUrl, options) {
  const { StreamingReporter } = require('../dist/core/reporting/streaming-reporter');
  const { StandardPipeline } = require('../dist/core');
  
  const streamingReporter = StreamingReporter.create(
    options.sessionId,
    process.stdout,
    {
      enabled: true,
      chunkSize: parseInt(options.chunkSize) || 10,
      bufferTimeout: 1000,
      includeDetailedResults: true,
      compressResults: false
    }
  );
  
  try {
    // Initialize streaming session
    streamingReporter.init(options.full ? 1000 : 5, {});
    
    // Report initial progress
    streamingReporter.reportProgress({
      current: 0,
      total: options.full ? 1000 : 5,
      currentUrl: sitemapUrl,
      stage: 'parsing_sitemap'
    });
    
    // Configure pipeline options
    const config = {
      maxPages: options.full ? 1000 : 5,
      standard: 'WCAG2AA',
      format: options.format || 'html',
      outputDir: options.outputDir || './reports',
      timeout: 30000,
      generateDetailedReport: true,
      generatePerformanceReport: true,
      generateSeoReport: false,
      generateSecurityReport: false,
      outputFormat: options.format || 'html',
      maxConcurrent: 2,
      verbose: options.verbose || false,
      timestamp: new Date().toISOString(),
      collectPerformanceMetrics: true,
      
      // 🔥 Advanced v2.0 Features (all enabled by default for streaming)
      modernHtml5: true,
      ariaAdvanced: true,
      chrome135Features: true,
      semanticAnalysis: true
    };
    
    const pipeline = new StandardPipeline();
    
    // Override pipeline progress reporting for streaming
    const originalProgressCallback = config.progressCallback;
    config.progressCallback = (current, total, currentUrl) => {
      streamingReporter.reportProgress({
        current,
        total,
        currentUrl: currentUrl || 'Processing...',
        stage: 'testing_pages'
      });
      
      if (originalProgressCallback) {
        originalProgressCallback(current, total, currentUrl);
      }
    };
    
    const { summary, outputFiles } = await pipeline.run({
      sitemapUrl,
      ...config
    });
    
    // Report completion
    streamingReporter.complete(summary, summary.testedPages, summary.passedPages);
    
    // Clean exit for streaming mode
    // Only exit with code 1 for technical crashes, not accessibility failures
    process.exit(summary.crashedPages > 0 ? 1 : 0);
    
  } catch (error) {
    streamingReporter.reportError(
      error.message || String(error),
      sitemapUrl,
      'streaming_audit',
      false
    );
    
    process.exit(1);
  } finally {
    streamingReporter.cleanup();
  }
}

// Helper function to run standard pipeline (used as fallback)
async function runStandardPipeline(finalSitemapUrl, config, pipelineOptions, pipeline) {
  console.log('\\n🎯 Starting standard accessibility test...');
  
  // Get actual page count from sitemap
  let actualPageCount = config.maxPages;
  try {
    const { SitemapParser } = require('../dist/parsers/sitemap-parser');
    const parser = new SitemapParser();
    const urls = await parser.parseSitemap(finalSitemapUrl);
    actualPageCount = config.maxPages === 1000 ? urls.length : Math.min(urls.length, config.maxPages);
    console.log(`📈 Found ${urls.length} URLs in sitemap, testing ${actualPageCount}`);
  } catch (error) {
    console.log('⚙️  Could not parse sitemap, using default page count');
  }
  
  const startTime = Date.now();
  
  // Update pipeline options with discovered sitemap URL
  pipelineOptions.sitemapUrl = finalSitemapUrl;
  
  console.log(`✨ ${actualPageCount === 1 ? '1 page' : actualPageCount + ' pages'} will be tested...`);
  
  const { summary, outputFiles } = await pipeline.run(pipelineOptions);
  
  const totalTime = Math.round((Date.now() - startTime) / 1000);
  console.log(`✅ Completed ${summary.testedPages} pages in ${formatTime(totalTime)}`);
  
  return { summary, outputFiles, totalTime };
}

// Helper function for Report Generation
function generateAccessibilityReport(result) {
  const { summary, results } = result;
  
  return `<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Accessibility Analysis Report</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 0; padding: 20px; background: #f5f5f5; }
        .container { max-width: 1200px; margin: 0 auto; background: white; padding: 30px; border-radius: 8px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }
        h1 { color: #2563eb; border-bottom: 2px solid #2563eb; padding-bottom: 10px; }
        .summary { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 20px; margin: 20px 0; }
        .metric { background: #f8fafc; padding: 15px; border-radius: 8px; text-align: center; }
        .metric-value { font-size: 2em; font-weight: bold; color: #2563eb; }
        .metric-label { font-size: 0.9em; color: #64748b; margin-top: 5px; }
        .results-table { width: 100%; border-collapse: collapse; margin: 20px 0; }
        .results-table th, .results-table td { padding: 12px; text-align: left; border-bottom: 1px solid #e2e8f0; }
        .results-table th { background: #f1f5f9; font-weight: 600; }
        .grade { padding: 4px 8px; border-radius: 4px; color: white; font-weight: bold; }
        .grade-A { background: #10b981; }
        .grade-B { background: #3b82f6; }
        .grade-C { background: #f59e0b; }
        .grade-D { background: #ef4444; }
        .grade-F { background: #991b1b; }
        .analysis-metrics { display: grid; grid-template-columns: 1fr 1fr 1fr; gap: 10px; font-size: 0.9em; }
        .analysis-metric { background: #f8fafc; padding: 8px; border-radius: 4px; text-align: center; }
    </style>
</head>
<body>
    <div class="container">
        <h1>🚀 Accessibility Analysis Report</h1>
        
        <div class="summary">
            <div class="metric">
                <div class="metric-value">${summary.testedPages}</div>
                <div class="metric-label">Pages Tested</div>
            </div>
            <div class="metric">
                <div class="metric-value">${summary.passedPages}</div>
                <div class="metric-label">Passed</div>
            </div>
            <div class="metric">
                <div class="metric-value">${summary.failedPages}</div>
                <div class="metric-label">Failed</div>
            </div>
            <div class="metric">
                <div class="metric-value">${Math.round((summary.passedPages / summary.testedPages) * 100)}%</div>
                <div class="metric-label">Success Rate</div>
            </div>
        </div>
        
        <h2>Detailed Results</h2>
        <table class="results-table">
            <thead>
                <tr>
                    <th>Page</th>
                    <th>Status</th>
                    <th>Analysis Metrics</th>
                    <th>Quality Score</th>
                </tr>
            </thead>
            <tbody>
                ${results.map((page) => `
                    <tr>
                        <td>
                            <strong>${page.title}</strong><br>
                            <small style="color: #64748b;">${page.url}</small>
                        </td>
                        <td>
                            ${page.passed ? '✅ Passed' : '❌ Failed'}
                            ${page.errors ? `<br><small>${page.errors} errors</small>` : ''}
                            ${page.warnings ? `<br><small>${page.warnings} warnings</small>` : ''}
                        </td>
                        <td>
                            <div class="analysis-metrics">
                                ${page.seo ? `<div class="analysis-metric">SEO: ${page.seo.seoScore}/100</div>` : ''}
                                ${page.contentWeight ? `<div class="analysis-metric">Content: ${page.contentWeight.contentScore}/100</div>` : ''}
                                ${page.performance ? `<div class="analysis-metric">Performance: ${page.performance.performanceScore || 'N/A'}</div>` : ''}
                            </div>
                        </td>
                        <td>
                            ${page.qualityScore ? 
                                `<span class="grade grade-${page.qualityScore.grade}">${page.qualityScore.score}/100 (${page.qualityScore.grade})</span>` : 'N/A'
                            }
                        </td>
                    </tr>
                `).join('')}
            </tbody>
        </table>
        
        <footer style="margin-top: 40px; padding-top: 20px; border-top: 1px solid #e2e8f0; text-align: center; color: #64748b;">
            <p>Generated by AuditMySite - ${new Date().toLocaleString()}</p>
        </footer>
    </div>
</body>
</html>`;
}

program.parse();
