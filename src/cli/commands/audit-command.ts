/**
 * 🔧 Audit Command
 * 
 * Main command for running accessibility audits.
 * Replaces the monolithic CLI logic with clean command structure.
 */

import { BaseCommand, CommandArgs, CommandResult } from './base-command';
import { StandardPipeline, StandardPipelineOptions } from '../../core/pipeline/standard-pipeline';
import { SitemapDiscovery } from '../../core/parsers/sitemap-discovery';
import { UnifiedReportSystem, ReportData, ReportOptions, ReportFormat } from '../../reports/unified';
import inquirer from 'inquirer';
import * as path from 'path';
import * as fs from 'fs';

export interface AuditCommandArgs extends CommandArgs {
  sitemapUrl: string;
  full?: boolean;
  maxPages?: number;
  expert?: boolean;
  format?: ReportFormat[];
  outputDir?: string;
  nonInteractive?: boolean;
  verbose?: boolean;
  budget?: string;
  lcpBudget?: number;
  clsBudget?: number;
  fcpBudget?: number;
  inpBudget?: number;
  ttfbBudget?: number;
  unifiedQueue?: boolean;
  // 🆕 Analysis Options - all enabled by default
  noPerformance?: boolean;         // Disable performance analysis
  noSeo?: boolean;                 // Disable SEO analysis
  noContentWeight?: boolean;       // Disable content weight analysis
  noMobile?: boolean;              // Disable mobile-friendliness analysis
}

export class AuditCommand extends BaseCommand {
  constructor() {
    super('audit', 'Run accessibility audit on sitemap URLs');
  }

  protected validate(args: AuditCommandArgs): { valid: boolean; errors: string[] } {
    const errors: string[] = [];

    // Validate sitemap URL
    const urlValidation = this.validateSitemapUrl(args.sitemapUrl);
    if (!urlValidation.valid) {
      errors.push(`Invalid sitemap URL: ${urlValidation.error}`);
    }

    // Validate maxPages if provided
    if (args.maxPages !== undefined) {
      if (args.maxPages < 1 || args.maxPages > 1000) {
        errors.push('maxPages must be between 1 and 1000');
      }
    }

    // Validate formats
    if (args.format && args.format.length > 0) {
      const validFormats: ReportFormat[] = ['html', 'markdown', 'json', 'csv'];
      const invalidFormats = args.format.filter(f => !validFormats.includes(f));
      if (invalidFormats.length > 0) {
        errors.push(`Invalid formats: ${invalidFormats.join(', ')}. Valid formats: ${validFormats.join(', ')}`);
      }
    }

    // Validate budget template
    if (args.budget && !['default', 'ecommerce', 'corporate', 'blog'].includes(args.budget)) {
      errors.push('budget must be one of: default, ecommerce, corporate, blog');
    }

    return {
      valid: errors.length === 0,
      errors
    };
  }

  async execute(args: AuditCommandArgs): Promise<CommandResult> {
    try {
      // Validate arguments
      const validation = this.validate(args);
      if (!validation.valid) {
        return this.error(`Validation failed: ${validation.errors.join(', ')}`);
      }

      // Show header
      const packageJson = require('../../../package.json');
      this.logProgress(`AuditMySite v${packageJson.version} - Professional Accessibility Testing`);
      this.logProgress(`Sitemap: ${args.sitemapUrl}`);

      // Determine configuration
      const config = await this.buildConfiguration(args);
      
      // Show configuration summary
      this.showConfigurationSummary(config, args);

      // Discover sitemap if needed
      const finalSitemapUrl = await this.discoverSitemap(args.sitemapUrl);

      // Create output directory
      const outputInfo = this.setupOutputDirectory(finalSitemapUrl, args.outputDir);

      // Run the audit
      const result = await this.runAudit(finalSitemapUrl, config, outputInfo);

      // Show results
      this.showResults(result);

      return this.success('Audit completed successfully', result);

    } catch (error) {
      const errorMessage = this.formatError(error as Error);
      this.logError(`Audit failed: ${errorMessage}`);
      
      if (args.verbose) {
        console.error('Full error details:', error);
      } else {
        this.logProgress('Run with --verbose for detailed error information');
      }

      return this.error(errorMessage);
    }
  }

  private async buildConfiguration(args: AuditCommandArgs): Promise<StandardPipelineOptions> {
    // Smart defaults - sitemapUrl will be set later in runAudit
    const baseConfig: StandardPipelineOptions = {
      sitemapUrl: args.sitemapUrl, // Required property
      maxPages: args.maxPages || (args.full ? 1000 : 5),
      timeout: 10000,
      pa11yStandard: 'WCAG2AA',
      outputDir: args.outputDir || './reports',
      outputFormat: Array.isArray(args.format) ? 
        (args.format[0] === 'json' ? 'html' : 
         args.format[0] === 'markdown' ? 'markdown' : 'html') : 
        (args.format === 'json' ? 'html' : 
         args.format === 'markdown' ? 'markdown' : 'html'),
      maxConcurrent: 2,
      generateDetailedReport: true,
      generatePerformanceReport: true,
      // generateSeoReport: false, // Not in StandardPipelineOptions
      // generateSecurityReport: false, // Not in StandardPipelineOptions
      // usePa11y: true, // Not in StandardPipelineOptions
      collectPerformanceMetrics: true,
      useUnifiedQueue: args.unifiedQueue || false, // NEW: Use unified queue system
      // 🚀 NEW: Enhanced analysis enabled (BrowserPoolManager fixed)
      useEnhancedAnalysis: true,
      contentWeightAnalysis: true,
      enhancedPerformanceAnalysis: true,
      enhancedSeoAnalysis: true
    };
    
    // Store all formats for unified report system
    if (Array.isArray(args.format) && args.format.length > 0) {
      (baseConfig as any).outputFormats = args.format;
    }

    // Analysis configuration from CLI args - disable specific features
    if (args.noPerformance) {
      (baseConfig as any).enhancedPerformanceAnalysis = false;
    }
    if (args.noSeo) {
      (baseConfig as any).enhancedSeoAnalysis = false;
    }
    if (args.noContentWeight) {
      (baseConfig as any).contentWeightAnalysis = false;
    }

    // Expert mode - interactive configuration
    if (args.expert && !args.nonInteractive) {
      return await this.runExpertMode(baseConfig);
    }

    // Build performance budget
    // Performance budget will be handled by the unified report system

    return baseConfig;
  }

  private async runExpertMode(baseConfig: StandardPipelineOptions): Promise<StandardPipelineOptions> {
    this.logProgress('Expert Mode - Custom Configuration');
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
        default: baseConfig.maxPages
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
        default: baseConfig.pa11yStandard
      },
      {
        type: 'checkbox',
        name: 'formats',
        message: '📄 Report formats? (select multiple)',
        choices: [
          { name: '🌐 HTML - Professional reports for stakeholders', value: 'html', checked: true },
          { name: '📝 Markdown - Developer-friendly, version control', value: 'markdown' },
          { name: '📊 JSON - Machine-readable for CI/CD pipelines', value: 'json' },
          { name: '📈 CSV - Data analysis and spreadsheet integration', value: 'csv' }
        ],
        validate: (answer) => {
          if (answer.length === 0) {
            return 'Please select at least one format';
          }
          return true;
        }
      },
      {
        type: 'confirm',
        name: 'enhanced',
        message: '🚀 Enable Enhanced Analysis? (Performance, SEO, Content Weight)',
        default: false
      },
      {
        type: 'checkbox',
        name: 'enhancedComponents',
        message: '🔍 Which enhanced components? (select multiple)',
        choices: [
          { name: '⚡ Enhanced Performance - Core Web Vitals, advanced metrics', value: 'performance' },
          { name: '🔍 Enhanced SEO - Meta tags, content quality, readability', value: 'seo' },
          { name: '📏 Content Weight - Resource analysis, text-to-code ratio', value: 'contentWeight' }
        ],
        when: (answers) => answers.enhanced,
        validate: (answer) => {
          if (answer.length === 0) {
            return 'Please select at least one enhanced component';
          }
          return true;
        }
      },
      {
        type: 'confirm',
        name: 'useUnifiedQueue',
        message: '🔧 Use the NEW Unified Queue System? (Recommended)',
        default: true
      },
      {
        type: 'number',
        name: 'maxConcurrent',
        message: '🔄 Concurrent page tests (1-5)?',
        default: baseConfig.maxConcurrent,
        validate: (value) => {
          const num = parseInt(value?.toString() || '0');
          if (num >= 1 && num <= 5) return true;
          return 'Please enter a number between 1 and 5';
        }
      }
    ]);

    // Store additional formats in extended config for the unified system
    const result: StandardPipelineOptions = {
      ...baseConfig,
      maxPages: answers.maxPages,
      pa11yStandard: answers.standard,
      outputFormat: (answers.formats[0] === 'markdown' ? 'markdown' : 'html') as 'markdown' | 'html',
      useUnifiedQueue: answers.useUnifiedQueue,
      maxConcurrent: answers.maxConcurrent
    };
    
    // Store all formats for unified system (not in StandardPipelineOptions interface)
    (result as any).outputFormats = answers.formats;
    
    // Store enhanced analysis settings
    if (answers.enhanced && answers.enhancedComponents) {
      (result as any).enhanced = true;
      (result as any).enhancedPerformance = answers.enhancedComponents.includes('performance');
      (result as any).enhancedSeo = answers.enhancedComponents.includes('seo');
      (result as any).contentWeight = answers.enhancedComponents.includes('contentWeight');
    }
    
    return result;
  }

  private buildPerformanceBudget(args: AuditCommandArgs): any {
    const { BUDGET_TEMPLATES } = require('../../core/performance/web-vitals-collector');

    // Custom budget from CLI options
    if (args.lcpBudget || args.clsBudget || args.fcpBudget || args.inpBudget || args.ttfbBudget) {
      const defaultBudget = BUDGET_TEMPLATES[args.budget || 'default'];
      return {
        lcp: { 
          good: args.lcpBudget || defaultBudget.lcp.good, 
          poor: (args.lcpBudget || defaultBudget.lcp.good) * 1.6 
        },
        cls: { 
          good: args.clsBudget || defaultBudget.cls.good, 
          poor: (args.clsBudget || defaultBudget.cls.good) * 2.5 
        },
        fcp: { 
          good: args.fcpBudget || defaultBudget.fcp.good, 
          poor: (args.fcpBudget || defaultBudget.fcp.good) * 1.7 
        },
        inp: { 
          good: args.inpBudget || defaultBudget.inp.good, 
          poor: (args.inpBudget || defaultBudget.inp.good) * 2.5 
        },
        ttfb: { 
          good: args.ttfbBudget || defaultBudget.ttfb.good, 
          poor: (args.ttfbBudget || defaultBudget.ttfb.good) * 2 
        }
      };
    }

    // Template budget
    const template = args.budget || 'default';
    return BUDGET_TEMPLATES[template] || BUDGET_TEMPLATES.default;
  }

  private showConfigurationSummary(config: StandardPipelineOptions, args: AuditCommandArgs): void {
    console.log('\\n📋 Configuration:');
    console.log(`   📄 Pages: ${config.maxPages === 1000 ? 'All' : config.maxPages}`);
    console.log(`   📋 Standard: ${config.pa11yStandard}`);
    console.log(`   📈 Performance: ${config.generatePerformanceReport ? 'Yes' : 'No'}`);
    console.log(`   🔧 Queue System: ${config.useUnifiedQueue ? 'Unified (NEW)' : 'Legacy'}`);
    console.log(`   📄 Format: ${config.outputFormat?.toUpperCase()}`);
    console.log(`   📁 Output: ${config.outputDir}`);
    
    // Enhanced Analysis Summary
    const enhancedConfig = config as any;
    if (enhancedConfig.enhanced || args.enhanced) {
      console.log('\\n🚀 Enhanced Analysis:');
      if (enhancedConfig.enhancedPerformance || args.enhancedPerformance) {
        console.log('   ⚡ Enhanced Performance: Yes - Core Web Vitals, advanced metrics');
      }
      if (enhancedConfig.enhancedSeo || args.enhancedSeo) {
        console.log('   🔍 Enhanced SEO: Yes - Meta analysis, content quality, readability');
      }
      if (enhancedConfig.contentWeight || args.contentWeight) {
        console.log('   📏 Content Weight: Yes - Resource analysis, text-to-code ratios');
      }
    }
  }

  private async discoverSitemap(sitemapUrl: string): Promise<string> {
    if (sitemapUrl.includes('sitemap.xml') || sitemapUrl.includes('sitemap')) {
      return sitemapUrl;
    }

    this.logProgress('Discovering sitemap...');
    const discovery = new SitemapDiscovery();
    const result = await discovery.discoverSitemap(sitemapUrl);

    if (result.found) {
      const finalUrl = result.sitemaps[0];
      this.logSuccess(`Found sitemap: ${finalUrl} (method: ${result.method})`);
      if (result.sitemaps.length > 1) {
        this.logProgress(`Additional sitemaps found: ${result.sitemaps.length - 1}`);
      }
      return finalUrl;
    } else {
      result.warnings.forEach(warning => this.logWarning(warning));
      throw new Error('No sitemap found');
    }
  }

  private setupOutputDirectory(sitemapUrl: string, outputDir: string = './reports'): { dir: string; domain: string } {
    const domain = this.extractDomain(sitemapUrl);
    const subDir = path.join(outputDir, domain);
    
    if (!fs.existsSync(subDir)) {
      fs.mkdirSync(subDir, { recursive: true });
    }

    return { dir: subDir, domain };
  }

  private async runAudit(sitemapUrl: string, config: StandardPipelineOptions, outputInfo: any): Promise<any> {
    const enhancedConfig = config as any;
    const startTime = Date.now();
    
    // Check if Enhanced Analysis is enabled
    if (enhancedConfig.enhanced) {
      this.logProgress('Starting enhanced accessibility analysis...');
      return await this.runEnhancedAudit(sitemapUrl, config, outputInfo);
    }
    
    // Standard audit pipeline
    this.logProgress('Starting accessibility test...');
    const pipeline = new StandardPipeline();

    const result = await pipeline.run({
      ...config,
      sitemapUrl,
      outputDir: outputInfo.dir
    });

    // Generate reports using the Unified Report System
    if (config.outputFormat) {
      await this.generateUnifiedReports(result, config, outputInfo);
    }

    const totalTime = Date.now() - startTime;
    const avgSpeed = result.summary.testedPages / (totalTime / 60000); // pages per minute

    this.logSuccess(`Completed ${result.summary.testedPages} pages in ${this.formatDuration(totalTime)}`);
    this.logProgress(`Average speed: ${avgSpeed.toFixed(1)} pages/minute`);

    return result;
  }

  private async runEnhancedAudit(sitemapUrl: string, config: StandardPipelineOptions, outputInfo: any): Promise<any> {
    const { EnhancedAccessibilityChecker } = require('../../enhanced-accessibility-checker');
    const { SitemapParser } = require('../../core/parsers/sitemap-parser');
    
    const enhancedConfig = config as any;
    const startTime = Date.now();
    
    try {
      // Parse sitemap to get URLs
      this.logProgress('Parsing sitemap...');
      const parser = new SitemapParser();
      const urls = await parser.parseFromUrl(sitemapUrl);
      const limitedUrls = urls.slice(0, config.maxPages || 5);
      
      this.logProgress(`Found ${urls.length} URLs, testing ${limitedUrls.length}`);
      
      // Initialize Enhanced Accessibility Checker
      const checker = new EnhancedAccessibilityChecker({
        includeResourceAnalysis: enhancedConfig.contentWeight,
        includeSocialAnalysis: enhancedConfig.enhancedSeo,
        includeReadabilityAnalysis: enhancedConfig.enhancedSeo,
        includeTechnicalSEO: enhancedConfig.enhancedSeo,
        analysisTimeout: 30000
      });
      
      await checker.initialize();
      this.logProgress('Enhanced accessibility checker initialized');
      
      const results = [];
      let successCount = 0;
      let errorCount = 0;
      let warningCount = 0;
      
      // Process each URL
      for (let i = 0; i < limitedUrls.length; i++) {
        const url = limitedUrls[i];
        this.logProgress(`[${i + 1}/${limitedUrls.length}] Analyzing ${url}`);
        
        try {
          // For enhanced analysis, we'll use the URL directly
          // Note: This is a simplified approach - in production you'd want to fetch HTML first
          const result = await checker.analyze('', url); // Empty HTML means it will fetch the page
          
          results.push({
            url,
            title: result.title || 'N/A',
            errors: result.errors?.length || 0,
            warnings: result.warnings?.length || 0,
            passed: result.passed,
            enhancedPerformance: result.enhancedPerformance,
            enhancedSEO: result.enhancedSEO,
            contentWeight: result.contentWeight,
            qualityScore: result.qualityScore
          });
          
          if (result.passed) successCount++;
          errorCount += result.errors?.length || 0;
          warningCount += result.warnings?.length || 0;
          
          // Show enhanced metrics for this page
          if (result.qualityScore) {
            this.logProgress(`   Quality Score: ${result.qualityScore.score}/100 (${result.qualityScore.grade})`);
          }
          if (result.enhancedSEO) {
            this.logProgress(`   SEO Score: ${result.enhancedSEO.seoScore}/100`);
          }
          if (result.contentWeight) {
            this.logProgress(`   Content Score: ${result.contentWeight.contentScore}/100`);
          }
          
        } catch (error) {
          this.logError(`Failed to analyze ${url}: ${error}`);
          results.push({
            url,
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
      
      const totalTime = Date.now() - startTime;
      const avgSpeed = results.length / (totalTime / 60000);
      
      // Build result summary
      const summary = {
        totalPages: urls.length,
        testedPages: results.length,
        passedPages: successCount,
        failedPages: results.length - successCount,
        crashedPages: results.filter(r => r.crashed).length,
        totalErrors: errorCount,
        totalWarnings: warningCount,
        totalDuration: totalTime,
        results
      };
      
      const finalResult = {
        summary,
        issues: [],
        sitemapUrl,
        outputFiles: [],
        enhancedResults: results // Store enhanced results
      };
      
      // Generate enhanced reports if needed
      if (config.outputFormat) {
        await this.generateEnhancedReports(finalResult, config, outputInfo);
      }
      
      this.logSuccess(`Enhanced analysis completed: ${results.length} pages in ${this.formatDuration(totalTime)}`);
      this.logProgress(`Average speed: ${avgSpeed.toFixed(1)} pages/minute`);
      
      return finalResult;
      
    } catch (error) {
      this.logError(`Enhanced audit failed: ${error}`);
      throw error;
    }
  }

  private async generateEnhancedReports(result: any, config: StandardPipelineOptions, outputInfo: any): Promise<void> {
    try {
      this.logProgress('Generating enhanced analysis reports...');
      
      // For now, create a simple enhanced report
      // In the future, this should use an enhanced report generator
      const reportPath = path.join(outputInfo.dir, 'enhanced-audit-report.html');
      
      const htmlContent = this.generateSimpleEnhancedReport(result, config);
      fs.writeFileSync(reportPath, htmlContent);
      
      const sizeKB = Math.round(fs.statSync(reportPath).size / 1024);
      this.logSuccess(`Generated enhanced HTML report: enhanced-audit-report.html (${sizeKB}KB)`);
      
      result.outputFiles = [reportPath];
      
    } catch (error) {
      this.logWarning(`Enhanced report generation failed: ${error}`);
      this.logProgress('Enhanced analysis results are still available, only report generation failed');
    }
  }

  private generateSimpleEnhancedReport(result: any, config: StandardPipelineOptions): string {
    const { summary, enhancedResults } = result;
    
    return `<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Enhanced Accessibility Analysis Report</title>
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
        .enhanced-metrics { display: grid; grid-template-columns: 1fr 1fr 1fr; gap: 10px; font-size: 0.9em; }
        .enhanced-metric { background: #f8fafc; padding: 8px; border-radius: 4px; text-align: center; }
    </style>
</head>
<body>
    <div class="container">
        <h1>🚀 Enhanced Accessibility Analysis Report</h1>
        
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
                    <th>Enhanced Metrics</th>
                    <th>Quality Score</th>
                </tr>
            </thead>
            <tbody>
                ${enhancedResults.map((page: any) => `
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
                            <div class="enhanced-metrics">
                                ${page.enhancedSEO ? `<div class="enhanced-metric">SEO: ${page.enhancedSEO.seoScore}/100</div>` : ''}
                                ${page.contentWeight ? `<div class="enhanced-metric">Content: ${page.contentWeight.contentScore}/100</div>` : ''}
                                ${page.enhancedPerformance ? `<div class="enhanced-metric">Performance: ${page.enhancedPerformance.performanceScore || 'N/A'}</div>` : ''}
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
            <p>Generated by AuditMySite Enhanced Analysis - ${new Date().toLocaleString()}</p>
        </footer>
    </div>
</body>
</html>`;
  }

  private async generateUnifiedReports(result: any, config: StandardPipelineOptions, outputInfo: any): Promise<void> {
    try {
      const reportSystem = new UnifiedReportSystem();
      
      // Prepare report data
      const reportData: ReportData = {
        summary: result.summary,
        issues: result.issues || [],
        metadata: {
          timestamp: new Date().toISOString(),
          version: require('../../../package.json').version,
          duration: result.summary.totalDuration || 0,
          sitemapUrl: result.sitemapUrl,
          environment: process.env.NODE_ENV || 'development'
        },
        config: config
      };

      // Prepare report options
      const reportOptions: ReportOptions = {
        outputDir: outputInfo.dir,
        includePa11yIssues: true,
        summaryOnly: false,
        prettyPrint: true,
        branding: {
          company: 'AuditMySite',
          footer: 'Generated by AuditMySite - Professional Accessibility Testing'
        }
      };

      // Determine formats to generate (from args or expert mode)
      const formats: ReportFormat[] = (config as any).outputFormats || 
        (config.outputFormat ? [config.outputFormat as ReportFormat] : ['html']);
      
      this.logProgress(`Generating reports in ${formats.join(', ')} format${formats.length > 1 ? 's' : ''}...`);
      
      // Generate all requested reports
      const generatedReports = await reportSystem.generateMultipleReports(formats, reportData, reportOptions);
      
      // Log generated reports
      generatedReports.forEach(report => {
        const sizeKB = Math.round(report.size / 1024);
        this.logSuccess(`Generated ${report.format.toUpperCase()} report: ${path.basename(report.path)} (${sizeKB}KB)`);
      });
      
      // Add paths to result for legacy compatibility
      result.outputFiles = result.outputFiles || [];
      result.outputFiles.push(...generatedReports.map(r => r.path));
      
    } catch (error) {
      this.logWarning(`Report generation failed: ${error}`);
      this.logProgress('Audit results are still available, only report generation failed');
    }
  }

  private showResults(result: any): void {
    const { summary, outputFiles } = result;

    console.log('\\n📊 Results:');
    console.log(`   📄 Tested: ${summary.testedPages} pages`);
    console.log(`   ✅ Passed: ${summary.passedPages}`);
    console.log(`   ❌ Failed: ${summary.failedPages}`);
    console.log(`   ⚠️  Errors: ${summary.totalErrors}`);
    console.log(`   ⚠️  Warnings: ${summary.totalWarnings}`);

    const successRate = summary.testedPages > 0 ? 
      (summary.passedPages / summary.testedPages * 100).toFixed(1) : 0;
    console.log(`   🎯 Success Rate: ${successRate}%`);

    if (outputFiles.length > 0) {
      console.log('\\n📁 Generated reports:');
      outputFiles.forEach((file: string) => {
        console.log(`   📄 ${path.basename(file)}`);
      });
    }

    // Status summary
    if (summary.crashedPages > 0) {
      this.logError(`${summary.crashedPages} pages crashed due to technical errors`);
    } else if (summary.failedPages > 0) {
      this.logWarning(`${summary.failedPages} pages failed accessibility tests (this is normal for real websites)`);
      this.logProgress('Check the detailed report for specific issues to fix');
    }
  }
}
