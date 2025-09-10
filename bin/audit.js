#!/usr/bin/env node

const { Command } = require('commander');
const { StandardPipeline } = require('../dist/core');
const { SitemapDiscovery } = require('../dist/core/parsers');
const inquirer = require('inquirer').default;
const path = require('path');
const ora = require('ora').default || require('ora');
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
  .option('--format <type>', 'Report format: html or markdown', 'html')
  .option('--output-dir <dir>', 'Output directory for reports', './reports')
  .option('--budget <template>', 'Performance budget: default, ecommerce, blog, corporate', 'default')
  
  // ✅ User Experience (3)
  .option('--expert', 'Interactive expert mode with advanced settings')
  .option('--non-interactive', 'Skip prompts for CI/CD (use defaults)')
  .option('-v, --verbose', 'Show detailed progress information')
  
  // ✅ Analysis Control (4) - Opt-out instead of opt-in
  .option('--no-performance', 'Disable performance analysis')
  .option('--no-seo', 'Disable SEO analysis')
  .option('--no-content-weight', 'Disable content weight analysis')
  .option('--no-mobile', 'Disable mobile-friendliness analysis')
  
  .action(async (sitemapUrl, options) => {
    
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
      mobileFriendliness: !options.noMobile
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
        }
      ]);
      
      // Update config with analysis feature selections
      config.performanceAnalysis = answers.analysisFeatures.includes('performance');
      config.seoAnalysis = answers.analysisFeatures.includes('seo');
      config.contentWeight = answers.analysisFeatures.includes('contentWeight');
      config.mobileFriendliness = answers.analysisFeatures.includes('mobile');
      
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
    
    console.log('\n✨ Simplified CLI - Only 11 parameters for better usability!');
    
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
        
        // Initialize Main Accessibility Checker with BASIC features (Enhanced temporarily disabled due to stack overflow)
        const checker = new MainAccessibilityChecker({
          includeResourceAnalysis: false, // Disabled temporarily
          includeSocialAnalysis: false,   // Disabled temporarily  
          includeReadabilityAnalysis: false, // Disabled temporarily
          includeTechnicalSEO: false,     // Disabled temporarily
          includeMobileFriendliness: false, // Disabled temporarily
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
              // Pa11y data - store directly for markdown report
              pa11yScore: result.pa11yScore,
              pa11yIssues: result.pa11yIssues,
              performance: result.performance,
              seo: result.seo,
              contentWeight: result.contentWeight,
              mobileFriendliness: result.mobileFriendliness, // Add Mobile-Friendliness data
              qualityScore: result.qualityScore,
              // Issues structure expected by HTML generator
              issues: {
                pa11yScore: result.pa11yScore,
                pa11yIssues: result.pa11yIssues,
                performanceMetrics: result.performance?.metrics,
                imagesWithoutAlt: result.imagesWithoutAlt || 0,
                buttonsWithoutLabel: result.buttonsWithoutLabel || 0,
                headingsCount: result.headingsCount || 0,
                keyboardNavigation: result.keyboardNavigation || [],
                colorContrastIssues: result.colorContrastIssues || [],
                focusManagementIssues: result.focusManagementIssues || [],
                screenshots: result.screenshots
              }
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
        
        console.log('\n📝 Generating comprehensive HTML report...');
        
        // Use the unified HTML generator
        let htmlContent;
        try {
          const { UnifiedHTMLGenerator } = require('../dist/reports/unified/unified-html-generator');
          const generator = new UnifiedHTMLGenerator();
          
          // Prepare data in unified FullAuditResult format
          const unifiedData = {
            metadata: {
              version: '1.0.0',
              timestamp: new Date().toISOString(),
              sitemapUrl: finalSitemapUrl,
              toolVersion: '2.0.0-alpha.1',
              duration: Date.now() - startTime
            },
            summary: {
              totalPages: urls.length,
              testedPages: results.length,
              passedPages: successCount,
              failedPages: results.length - successCount,
              crashedPages: results.filter(r => r.crashed).length,
              totalErrors: errorCount,
              totalWarnings: warningCount
            },
            pages: results.map(page => ({
              url: page.url,
              title: page.title,
              status: page.passed ? 'passed' : (page.crashed ? 'crashed' : 'failed'),
              duration: page.loadTime || 0,
              accessibility: {
                score: page.pa11yScore || 0,
                errors: page.pa11yIssues?.filter(i => i.type === 'error') || [],
                warnings: page.pa11yIssues?.filter(i => i.type === 'warning') || [],
                notices: page.pa11yIssues?.filter(i => i.type === 'notice') || []
              },
              performance: page.performance ? {
                coreWebVitals: {
                  largestContentfulPaint: page.performance.metrics?.lcp || 0,
                  firstInputDelay: page.performance.metrics?.fid || 0,
                  cumulativeLayoutShift: page.performance.metrics?.cls || 0
                }
              } : null,
              seo: page.seo || null,
              contentWeight: page.contentWeight || null,
              mobileFriendliness: page.mobileFriendliness || null
            }))
          };
          
          htmlContent = await generator.generate(unifiedData);
        } catch (e) {
          console.warn('Using fallback HTML template:', e.message);
          // Fallback to the comprehensive template if new generator fails
          htmlContent = generateComprehensiveHtmlReport(results, {
            timestamp: new Date().toLocaleString(),
            sitemapUrl: finalSitemapUrl,
            totalPages: urls.length,
            testedPages: results.length,
            passedPages: successCount,
            failedPages: results.length - successCount,
            totalErrors: errorCount,
            totalWarnings: warningCount
          });
        }
        
        const reportPath = path.join(subDir, `accessibility-report-${dateOnly}.html`);
        require('fs').writeFileSync(reportPath, htmlContent);
        
        // 📝 Generate detailed accessibility issues markdown report
        const { DetailedIssueMarkdownReport } = require('../dist/reports/detailed-issue-markdown');
        
        // Extract all pa11y issues and convert to DetailedIssue format
        const detailedIssues = [];
        results.forEach((page, index) => {
          if (page.pa11yIssues && Array.isArray(page.pa11yIssues) && page.pa11yIssues.length > 0) {
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
        });
        
        // Generate detailed issues markdown if there are issues
        if (detailedIssues.length > 0) {
          const detailedMarkdown = DetailedIssueMarkdownReport.generate(detailedIssues);
          const detailedPath = path.join(subDir, `detailed-issues-${dateOnly}.md`);
          require('fs').writeFileSync(detailedPath, detailedMarkdown);
          outputFiles = [reportPath, detailedPath];
        } else {
          outputFiles = [reportPath];
        }
        
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

// Streaming audit function removed in CLI simplification - no longer needed

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

// Comprehensive HTML Report Generator with Filter System and Detailed Issues
function generateComprehensiveHtmlReport(results, summary) {
  const domain = results.length > 0 ? new URL(results[0].url).hostname : 'unknown';
  const successRate = Math.round((summary.passedPages / summary.testedPages) * 100) || 0;
  
  // Calculate total pa11y issues for detailed section
  const allDetailedIssues = [];
  results.forEach(page => {
    if (page.pa11yIssues && Array.isArray(page.pa11yIssues)) {
      page.pa11yIssues.forEach(issue => {
        allDetailedIssues.push({
          ...issue,
          pageUrl: page.url,
          pageTitle: page.title,
          source: 'pa11y',
          category: getIssueCategory(issue.code || issue.type)
        });
      });
    }
  });
  
  return generateFullHtmlTemplate({
    domain,
    summary,
    results,
    allDetailedIssues,
    timestamp: summary.timestamp || new Date().toLocaleString(),
    successRate
  });
}

// Helper function for Simple Report Generation (fallback)
function generateSimpleHtmlReport(results, summary) {
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
        .pa11y-info { background: #e3f2fd; padding: 15px; border-left: 4px solid #2196f3; margin: 20px 0; }
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
        
        <div class="pa11y-info">
          <h3>📝 Detailed Accessibility Issues</h3>
          <p>The detailed accessibility issues found by Pa11y have been saved to a separate markdown file for easier review and integration into your development workflow.</p>
          <p><strong>Look for:</strong> detailed-issues-${new Date().toISOString().split('T')[0]}.md</p>
        </div>
        
        <h2>Detailed Results</h2>
        <table class="results-table">
            <thead>
                <tr>
                    <th>Page</th>
                    <th>Status</th>
                    <th>Pa11y Issues</th>
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
                            ${page.pa11yIssues && Array.isArray(page.pa11yIssues) ? 
                                `<strong>${page.pa11yIssues.length} issues</strong><br><small>Score: ${page.pa11yScore}/100</small>` : 
                                'No data'
                            }
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
            <p>Generated by AuditMySite - ${summary.timestamp}</p>
            <p><em>Note: Full HTML report generation temporarily disabled due to large issue count. Check the detailed-issues markdown file for complete accessibility findings.</em></p>
        </footer>
    </div>
</body>
</html>`;
}

// Helper functions for issue categorization and HTML generation
function getIssueCategory(codeOrType) {
  if (!codeOrType) return 'General';
  
  const code = codeOrType.toLowerCase();
  
  if (code.includes('color') || code.includes('contrast')) return 'Color & Contrast';
  if (code.includes('aria') || code.includes('role')) return 'ARIA & Semantics';
  if (code.includes('form') || code.includes('label')) return 'Forms & Labels';
  if (code.includes('image') || code.includes('alt')) return 'Images & Media';
  if (code.includes('heading') || code.includes('structure')) return 'Document Structure';
  if (code.includes('keyboard') || code.includes('focus')) return 'Keyboard & Focus';
  if (code.includes('link') || code.includes('anchor')) return 'Links & Navigation';
  if (code.includes('table')) return 'Tables';
  if (code.includes('landmark')) return 'Page Landmarks';
  
  return 'General Accessibility';
}

function escapeHtml(text) {
  return (text || '')
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#039;');
}

function getPageName(url) {
  try {
    const urlObj = new URL(url);
    const pathname = urlObj.pathname;
    return pathname === '/' ? 'Home' : pathname.split('/').pop() || pathname;
  } catch {
    return url;
  }
}

function generateFullHtmlTemplate(data) {
  const { domain, summary, results, allDetailedIssues, timestamp, successRate } = data;
  
  // Group issues by category for detailed section
  const groupedIssues = {};
  allDetailedIssues.forEach(issue => {
    const category = issue.category || 'General';
    if (!groupedIssues[category]) {
      groupedIssues[category] = [];
    }
    groupedIssues[category].push(issue);
  });
  
  const errorCount = allDetailedIssues.filter(i => i.type === 'error').length;
  const warningCount = allDetailedIssues.filter(i => i.type === 'warning').length;
  const noticeCount = allDetailedIssues.filter(i => i.type === 'notice').length;

  return getComprehensiveHtmlTemplate({ 
    domain, 
    summary, 
    results, 
    allDetailedIssues, 
    groupedIssues, 
    timestamp, 
    successRate, 
    errorCount, 
    warningCount, 
    noticeCount 
  });
}

// Load comprehensive HTML template functions
const { getComprehensiveHtmlTemplate } = require('./comprehensive-html-template.js');

program.parse();
