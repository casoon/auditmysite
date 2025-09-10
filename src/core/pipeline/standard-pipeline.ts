import { SitemapParser } from '@core/parsers';
import { AccessibilityChecker } from '@core/accessibility';
import { TestOptions, TestSummary, AccessibilityResult, AuditIssue } from '@core/types';
import * as path from 'path';
import * as fs from 'fs';

export interface StandardPipelineOptions {
  sitemapUrl: string;
  maxPages?: number;
  timeout?: number;
  pa11yStandard?: 'WCAG2A' | 'WCAG2AA' | 'WCAG2AAA' | 'Section508';
  outputDir?: string;
  includeDetails?: boolean;
  includePa11yIssues?: boolean;
  generateDetailedReport?: boolean;
  generatePerformanceReport?: boolean;
  hideElements?: string;
  includeNotices?: boolean;
  includeWarnings?: boolean;
  wait?: number;
  // 🆕 New Playwright options
  collectPerformanceMetrics?: boolean;
  captureScreenshots?: boolean;
  testKeyboardNavigation?: boolean;
  testColorContrast?: boolean;
  testFocusManagement?: boolean;
  blockImages?: boolean;
  blockCSS?: boolean;
  mobileEmulation?: boolean;
  viewportSize?: { width: number; height: number };
  userAgent?: string;
  // 🚀 Parallel test options (Queue is now default)
  maxConcurrent?: number;
  maxRetries?: number;
  retryDelay?: number;
  enableProgressBar?: boolean;
  progressUpdateInterval?: number;
  enableResourceMonitoring?: boolean;
  maxMemoryUsage?: number;
  maxCpuUsage?: number;
  // 🆕 Legacy option for sequential tests (for compatibility only)
  useSequentialTesting?: boolean;
  // 🆕 Output format option
  outputFormat?: 'markdown' | 'html';
  // Queue system is now the default and only option
  // 🆕 NEW: Enhanced analysis options
  useEnhancedAnalysis?: boolean;
  contentWeightAnalysis?: boolean;
  enhancedPerformanceAnalysis?: boolean;
  enhancedSeoAnalysis?: boolean;
}

export class StandardPipeline {
  
  /**
   * Runs the standard pipeline and creates AI-friendly output files
   */
  async run(options: StandardPipelineOptions): Promise<{
    summary: TestSummary;
    outputFiles: string[];
  }> {
    const outputDir = options.outputDir || './reports';
    const dateOnly = new Date().toISOString().split('T')[0]; // Date only, no timestamp
    
    // Initialize parser
    const parser = new SitemapParser();
    
    // Parse sitemap
    const urls = await parser.parseSitemap(options.sitemapUrl);
    console.log(`📄 Sitemap loaded: ${urls.length} URLs found`);
    
    // Filter URLs
    const filterPatterns = ['[...slug]', '[category]', '/demo/'];
    const filteredUrls = parser.filterUrls(urls, { filterPatterns });
    console.log(`🔍 URLs filtered: ${filteredUrls.length} URLs to test`);
    
    // Convert URLs to local URLs
    const baseUrl = new URL(options.sitemapUrl).origin;
    const localUrls = parser.convertToLocalUrls(filteredUrls, baseUrl);
    
    // IMPORTANT: Limit URLs to maxPages
    const maxPages = options.maxPages || 20;
    const limitedUrls = localUrls.slice(0, maxPages);
    console.log(`📋 URLs limited to ${maxPages}: ${limitedUrls.length} URLs will be tested`);
    
    // Initialize Accessibility Checker (with enhanced analysis if requested)
    let checker: AccessibilityChecker;
    let enhancedChecker: any = null;
    
    if (options.useEnhancedAnalysis) {
      console.log('🆕 Initializing Enhanced Accessibility Checker...');
      const { EnhancedAccessibilityChecker } = require('../accessibility/enhanced-accessibility-checker');
      const { BrowserManager } = require('../browser');
      
      enhancedChecker = new EnhancedAccessibilityChecker();
      const browserManager = new BrowserManager({ headless: true, port: 9222 });
      await browserManager.initialize();
      await enhancedChecker.initialize(browserManager);
      checker = enhancedChecker; // Use enhanced checker as the main checker
    } else {
      checker = new AccessibilityChecker();
      await checker.initialize();
    }
    
    console.log('🤪 Running accessibility tests...');
    console.log('⚙️  Configuration:');
    
    if (options.useEnhancedAnalysis) {
      console.log('   🆕 Enhanced Analysis Mode:');
      console.log(`     📦 Content Weight Analysis: ${options.contentWeightAnalysis !== false ? 'Yes' : 'No'}`);
      console.log(`     ⚡ Enhanced Performance: ${options.enhancedPerformanceAnalysis !== false ? 'Yes' : 'No'}`);
      console.log(`     🔍 Enhanced SEO: ${options.enhancedSeoAnalysis !== false ? 'Yes' : 'No'}`);
      console.log(`     🧠 Semantic Analysis: Yes`);
    } else {
      console.log('   Default mode:');
      console.log('     📊 Collect performance metrics');
      console.log('     🧪 Run accessibility tests (pa11y)');
      console.log('     🚀 Parallel processing');
    }
    
    console.log('   Expert mode (use --expert):');
    console.log(`     📸 Capture screenshots: ${options.captureScreenshots ? 'Yes' : 'No'} (--screenshots)`);
    console.log(`     ⌨️  Test keyboard navigation: ${options.testKeyboardNavigation ? 'Yes' : 'No'} (--keyboard)`);
    console.log(`     🎨 Test color contrast: ${options.testColorContrast ? 'Yes' : 'No'} (--contrast)`);
    console.log(`     🎯 Test focus management: ${options.testFocusManagement ? 'Yes' : 'No'} (--focus)`);
    if (options.useSequentialTesting) {
      console.log(`   📋 Sequential mode: Yes (--sequential)`);
    } else {
      console.log(`   🔧 Workers: ${options.maxConcurrent || 3} | Retries: ${options.maxRetries || 3} | Delay: ${options.retryDelay || 2000}ms`);
    }
    
    // Execute tests
    const testOptions: TestOptions = {
      maxPages: maxPages,
      timeout: options.timeout || 10000,
      waitUntil: 'domcontentloaded',
      pa11yStandard: options.pa11yStandard || 'WCAG2AA',
      hideElements: options.hideElements,
      includeNotices: options.includeNotices,
      includeWarnings: options.includeWarnings,
      wait: options.wait,
      // 🆕 New Playwright options
      collectPerformanceMetrics: options.collectPerformanceMetrics,
      captureScreenshots: options.captureScreenshots,
      testKeyboardNavigation: options.testKeyboardNavigation,
      testColorContrast: options.testColorContrast,
      testFocusManagement: options.testFocusManagement,
      blockImages: options.blockImages,
      blockCSS: options.blockCSS,
      mobileEmulation: options.mobileEmulation,
      viewportSize: options.viewportSize,
      userAgent: options.userAgent,
      // 🚀 Parallel test options
      useParallelTesting: !options.useSequentialTesting, // Keep this for compatibility, but it's now the default
      maxConcurrent: options.maxConcurrent,
      maxRetries: options.maxRetries,
      retryDelay: options.retryDelay,
      enableProgressBar: options.enableProgressBar,
      progressUpdateInterval: options.progressUpdateInterval,
      enableResourceMonitoring: options.enableResourceMonitoring,
      maxMemoryUsage: options.maxMemoryUsage,
      maxCpuUsage: options.maxCpuUsage
    };
    
    // Choose between enhanced analysis and regular analysis
    let results: AccessibilityResult[];
    
    if (options.useEnhancedAnalysis && enhancedChecker) {
      console.log('🆕 Running Enhanced Analysis Tests...');
      const enhancedTestOptions = {
        ...testOptions,
        enhancedAnalysis: true,
        contentWeightAnalysis: options.contentWeightAnalysis !== false,
        enhancedPerformanceAnalysis: options.enhancedPerformanceAnalysis !== false,
        enhancedSeoAnalysis: options.enhancedSeoAnalysis !== false,
        semanticAnalysis: true
      };
      
      results = await enhancedChecker.testMultiplePagesWithEnhancedAnalysis(
        limitedUrls.map((url: any) => url.loc),
        enhancedTestOptions
      );
    } else {
      // Regular accessibility tests with modern queue system
      if (options.useSequentialTesting) {
        console.log('📋 Use sequential tests (Legacy mode)...');
        results = await checker.testMultiplePages(
          limitedUrls.map((url: any) => url.loc),
          testOptions
        );
      } else {
        console.log('🔧 Use Queue System (Recommended)...');
        results = await checker.testMultiplePagesWithQueue(
          limitedUrls.map((url: any) => url.loc),
          testOptions
        );
      }
    }
    
    console.log('\n📋 Creating test summary...');
    
    // Create summary
    const summary: TestSummary = {
      totalPages: limitedUrls.length, // 🆕 Use limitedUrls instead of localUrls
      testedPages: results.length,
      passedPages: results.filter(r => r.passed).length,
      failedPages: results.filter(r => !r.passed && !r.crashed).length, // Only accessibility failures
      crashedPages: results.filter(r => r.crashed === true).length, // 🆕 Technical crashes
      totalErrors: results.reduce((sum, r) => sum + r.errors.length, 0),
      totalWarnings: results.reduce((sum, r) => sum + r.warnings.length, 0),
      totalDuration: results.reduce((sum, r) => sum + r.duration, 0),
      results
    };

    // 🆕 Create DetailedIssue array and generate Markdown report
    const { DetailedIssueCollector } = require('@core/accessibility');
    const { DetailedIssueMarkdownReport } = require('../../reports/index.js');
    const detailedIssues = (DetailedIssueCollector.collectAll(results) || []);
    const detailedMd = DetailedIssueMarkdownReport.generate(detailedIssues || []);
    const detailedMdPath = path.join(outputDir, `detailed-issues-${dateOnly}.md`);
    fs.writeFileSync(detailedMdPath, detailedMd, 'utf8');
    
    // Cleanup checkers
    if (enhancedChecker) {
      await enhancedChecker.cleanup();
    }
    await checker.cleanup();
    
    console.log('📄 Generating output files...');
    // Generate output files
    const outputFiles: string[] = [];
    
    // Fix: Always add detailed-issues.md to output files list first
    outputFiles.push(detailedMdPath);
    
    // Collect issues for reports
    const { PerformanceIssueCollector } = require('@core/performance');
    let allIssues: any[] = [];
    allIssues = allIssues.concat(DetailedIssueCollector.collectAll(results) || []);
    allIssues = allIssues.concat(PerformanceIssueCollector.collectAll(summary) || []);

    // Choose between Markdown and HTML output
    if (options.outputFormat === 'html') {
      console.log('   🌐 Generating HTML report with modern HTMLGenerator...');
      const { HTMLGenerator } = require('../../reports/unified/generators/html-generator');
      const generator = new HTMLGenerator();
      
      // Prepare audit data structure (similar to CLI format)
      const auditData = {
        metadata: {
          version: '1.0.0',
          timestamp: new Date().toISOString(),
          sitemapUrl: options.sitemapUrl,
          toolVersion: '2.0.0-alpha.1',
          duration: summary.totalDuration
        },
        summary: {
          totalPages: summary.totalPages,
          testedPages: summary.testedPages,
          passedPages: summary.passedPages,
          failedPages: summary.failedPages,
          crashedPages: summary.crashedPages,
          totalErrors: summary.totalErrors,
          totalWarnings: summary.totalWarnings
        },
        pages: summary.results.map(result => ({
          url: result.url,
          title: result.title,
          status: result.passed ? 'passed' : (result.crashed ? 'crashed' : 'failed'),
          duration: result.duration || 0,
          accessibility: {
            score: result.pa11yScore || 0,
            errors: result.errors || [],
            warnings: result.warnings || [],
            notices: result.pa11yIssues?.filter(issue => issue.type === 'notice') || []
          }
        }))
      };
      
      const htmlContent = await generator.generate(auditData);
      const htmlPath = path.join(outputDir, `accessibility-report-${dateOnly}.html`);
      fs.writeFileSync(htmlPath, htmlContent, 'utf8');
      outputFiles.push(htmlPath);
    } else {
      // Generate Markdown reports (default)
      const { OutputGenerator } = require('@generators');
      const { generateMarkdownReport } = require('@generators');
      const outputGenerator = new OutputGenerator();

      // Main accessibility report
      const timestamp = new Date().toISOString();
      const { prepareOutputData } = require('@generators/output-generator');
      const mdData = prepareOutputData(summary, timestamp, { includeDetails: true, summaryOnly: false });
      const qualityMd = generateMarkdownReport(mdData);
      const qualityMdPath = path.join(outputDir, `accessibility-quality-report-${dateOnly}.md`);
      fs.writeFileSync(qualityMdPath, qualityMd, 'utf8');
      outputFiles.push(qualityMdPath);
    }
    
    // Performance data is now integrated into HTML reports
    // No separate markdown files needed
    
    
    return {
      summary,
      outputFiles
    };
  }
}