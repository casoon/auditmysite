/**
 * 🎯 Stable Audit Interface
 * 
 * This interface provides a stable, reliable API for website auditing.
 * It abstracts internal complexity and ensures consistent behavior.
 * 
 * Key Features:
 * - Simple, consistent API
 * - Built-in error handling and validation
 * - Progress monitoring
 * - Health checks
 * - Comprehensive results
 * 
 * Usage:
 * ```typescript
 * const auditor = new StableAuditor({
 *   maxPages: 5,
 *   timeout: 60000,
 *   outputFormat: 'markdown'
 * });
 * 
 * const results = await auditor.auditWebsite('https://example.com/sitemap.xml');
 * ```
 */

import { AccessibilityChecker } from '@core/accessibility';
import { SitemapParser } from '@core/parsers';
import { BrowserPoolManager } from '@core/browser/browser-pool-manager';
import { HTMLGenerator } from '../generators/html-generator';
import * as path from 'path';
import * as fs from 'fs';

// Core types for the stable interface
export interface StableAuditConfig {
  /** Maximum number of pages to audit */
  maxPages: number;
  
  /** Timeout per page in milliseconds (default: 90000) */
  timeout?: number;
  
  /** Number of concurrent workers (default: 3) */
  maxConcurrent?: number;
  
  /** Output format (default: 'html') */
  outputFormat?: 'html' | 'markdown' | 'both';
  
  /** Output directory (default: './reports') */
  outputDir?: string;
  
  /** WCAG standard to test against (default: 'WCAG2AA') */
  standard?: 'WCAG2A' | 'WCAG2AA' | 'WCAG2AAA';
  
  /** Enable detailed logging (default: false) */
  verbose?: boolean;
  
  /** Custom report filename prefix */
  reportPrefix?: string;
}

export interface AuditResult {
  /** Overall audit summary */
  summary: {
    totalPages: number;
    testedPages: number;
    passedPages: number;
    failedPages: number;
    crashedPages: number;
    successRate: number;
    totalDuration: number;
    averagePageTime: number;
  };
  
  /** Per-page results */
  pages: PageAuditResult[];
  
  /** Generated report files */
  reports: {
    html?: string;
    markdown?: string;
  };
  
  /** Performance metrics */
  performance: {
    avgLoadTime: number;
    avgAccessibilityScore: number;
    avgPerformanceScore: number;
    avgSeoScore: number;
  };
  
  /** Audit metadata */
  metadata: {
    auditDate: string;
    version: string;
    config: StableAuditConfig;
    systemInfo: {
      nodeVersion: string;
      memoryUsage: NodeJS.MemoryUsage;
    };
  };
}

export interface PageAuditResult {
  url: string;
  title: string;
  passed: boolean;
  crashed: boolean;
  duration: number;
  
  scores: {
    accessibility: number;
    performance: number;
    seo: number;
    mobile: number;
  };
  
  issues: {
    errors: AuditIssue[];
    warnings: AuditIssue[];
    notices: AuditIssue[];
  };
  
  metrics: {
    loadTime: number;
    contentSize: number;
    resourceCount: number;
  };
}

export interface AuditIssue {
  type: 'accessibility' | 'performance' | 'seo' | 'mobile';
  severity: 'error' | 'warning' | 'notice';
  rule: string;
  message: string;
  element?: string;
  selector?: string;
}

export interface AuditProgress {
  phase: 'initializing' | 'parsing' | 'testing' | 'generating' | 'complete';
  progress: number; // 0-100
  completed: number;
  total: number;
  currentUrl?: string;
  eta?: number;
  message?: string;
}

export type ProgressCallback = (progress: AuditProgress) => void;
export type ErrorCallback = (error: AuditError) => void;

export interface AuditError {
  code: 'SITEMAP_ERROR' | 'BROWSER_ERROR' | 'TIMEOUT_ERROR' | 'VALIDATION_ERROR' | 'SYSTEM_ERROR';
  message: string;
  url?: string;
  details?: any;
  recoverable: boolean;
}

/**
 * Stable Auditor Class - The main interface for website auditing
 */
export class StableAuditor {
  private config: Required<StableAuditConfig>;
  private accessibilityChecker?: AccessibilityChecker;
  private browserPoolManager?: BrowserPoolManager;
  private progressCallback?: ProgressCallback;
  private errorCallback?: ErrorCallback;
  private isInitialized = false;
  private healthStatus: 'healthy' | 'degraded' | 'unhealthy' = 'healthy';

  constructor(config: StableAuditConfig) {
    // Apply defaults and validate config
    this.config = this.validateAndNormalizeConfig(config);
  }

  /**
   * Set progress callback to monitor audit progress
   */
  onProgress(callback: ProgressCallback): this {
    this.progressCallback = callback;
    return this;
  }

  /**
   * Set error callback to handle recoverable errors
   */
  onError(callback: ErrorCallback): this {
    this.errorCallback = callback;
    return this;
  }

  /**
   * Initialize the auditor (must be called before auditing)
   */
  async initialize(): Promise<void> {
    try {
      this.reportProgress('initializing', 0, 0, 0, 'Initializing auditor...');
      
      // Initialize browser pool
      this.browserPoolManager = new BrowserPoolManager({
        maxConcurrent: this.config.maxConcurrent,
        maxIdleTime: 30000,
        enableResourceOptimization: true
      });
      
      await this.browserPoolManager.warmUp(1);
      
      // Initialize accessibility checker
      this.accessibilityChecker = new AccessibilityChecker({
        poolManager: this.browserPoolManager,
        enableComprehensiveAnalysis: true,
        qualityAnalysisOptions: {
          verbose: this.config.verbose,
          analysisTimeout: this.config.timeout
        }
      });
      
      await this.accessibilityChecker.initialize();
      
      this.isInitialized = true;
      this.healthStatus = 'healthy';
      
      this.reportProgress('initializing', 100, 0, 0, 'Auditor initialized successfully');
      
    } catch (error) {
      this.healthStatus = 'unhealthy';
      this.reportError('SYSTEM_ERROR', 'Failed to initialize auditor', undefined, error, false);
      throw error;
    }
  }

  /**
   * Perform a complete website audit
   */
  async auditWebsite(sitemapUrl: string): Promise<AuditResult> {
    if (!this.isInitialized) {
      throw new Error('Auditor not initialized. Call initialize() first.');
    }

    const startTime = Date.now();
    
    try {
      // Health check
      await this.performHealthCheck();
      
      // Parse sitemap
      this.reportProgress('parsing', 0, 0, 0, 'Parsing sitemap...');
      const urls = await this.parseSitemap(sitemapUrl);
      
      // Audit pages
      this.reportProgress('testing', 0, 0, urls.length, 'Starting page audits...');
      const pageResults = await this.auditPages(urls);
      
      // Generate reports
      this.reportProgress('generating', 0, pageResults.length, pageResults.length, 'Generating reports...');
      const reports = await this.generateReports(pageResults, sitemapUrl);
      
      // Calculate summary
      const summary = this.calculateSummary(pageResults, startTime);
      const performance = this.calculatePerformanceMetrics(pageResults);
      
      const result: AuditResult = {
        summary,
        pages: pageResults,
        reports,
        performance,
        metadata: {
          auditDate: new Date().toISOString(),
          version: '2.0.0-alpha.2',
          config: this.config,
          systemInfo: {
            nodeVersion: process.version,
            memoryUsage: process.memoryUsage()
          }
        }
      };
      
      this.reportProgress('complete', 100, pageResults.length, pageResults.length, 'Audit completed successfully');
      
      return result;
      
    } catch (error) {
      this.healthStatus = 'unhealthy';
      this.reportError('SYSTEM_ERROR', 'Audit failed', sitemapUrl, error, false);
      throw error;
    }
  }

  /**
   * Cleanup resources (call when done)
   */
  async cleanup(): Promise<void> {
    try {
      if (this.accessibilityChecker) {
        await this.accessibilityChecker.cleanup();
      }
      
      if (this.browserPoolManager) {
        await this.browserPoolManager.shutdown();
      }
      
      this.isInitialized = false;
      this.healthStatus = 'healthy';
      
    } catch (error) {
      console.warn('Warning during cleanup:', error);
    }
  }

  /**
   * Get current health status of the auditor
   */
  getHealthStatus(): { status: string; details: any } {
    return {
      status: this.healthStatus,
      details: {
        initialized: this.isInitialized,
        browserPoolSize: this.browserPoolManager?.getMetrics()?.poolSize || 0,
        memoryUsage: process.memoryUsage(),
        uptime: process.uptime()
      }
    };
  }

  // Private helper methods

  private validateAndNormalizeConfig(config: StableAuditConfig): Required<StableAuditConfig> {
    if (!config.maxPages || config.maxPages < 1) {
      throw new Error('maxPages must be a positive number');
    }
    
    if (config.maxPages > 100) {
      console.warn('⚠️  Warning: maxPages > 100 may cause performance issues');
    }

    return {
      maxPages: config.maxPages,
      timeout: config.timeout || 90000,
      maxConcurrent: Math.min(config.maxConcurrent || 3, 5), // Cap at 5 for stability
      outputFormat: config.outputFormat || 'html',
      outputDir: config.outputDir || './reports',
      standard: config.standard || 'WCAG2AA',
      verbose: config.verbose || false,
      reportPrefix: config.reportPrefix || 'audit'
    };
  }

  private async performHealthCheck(): Promise<void> {
    // Check memory usage
    const memUsage = process.memoryUsage().heapUsed / 1024 / 1024;
    if (memUsage > 2048) { // 2GB threshold
      this.healthStatus = 'degraded';
      this.reportError('SYSTEM_ERROR', 'High memory usage detected', undefined, { memoryMB: memUsage }, true);
    }
    
    // Check browser pool
    if (this.browserPoolManager) {
      const metrics = this.browserPoolManager.getMetrics();
      if (metrics.poolSize === 0) {
        this.healthStatus = 'degraded';
        this.reportError('BROWSER_ERROR', 'No browsers available in pool', undefined, metrics, true);
      }
    }
  }

  private async parseSitemap(sitemapUrl: string): Promise<string[]> {
    try {
      const parser = new SitemapParser();
      const urls = await parser.parseSitemap(sitemapUrl);
      
      if (urls.length === 0) {
        throw new Error('No URLs found in sitemap');
      }
      
      // Filter and limit URLs
      const filteredUrls = parser.filterUrls(urls, {
        filterPatterns: ['[...slug]', '[category]', '/demo/', '/test/']
      });
      
      const limitedUrls = filteredUrls.slice(0, this.config.maxPages);
      
      if (this.config.verbose) {
        console.log(`📄 Sitemap parsed: ${urls.length} total, ${filteredUrls.length} after filtering, ${limitedUrls.length} selected`);
      }
      
      return limitedUrls.map(url => url.loc);
      
    } catch (error) {
      this.reportError('SITEMAP_ERROR', 'Failed to parse sitemap', sitemapUrl, error, false);
      throw error;
    }
  }

  private async auditPages(urls: string[]): Promise<PageAuditResult[]> {
    if (!this.accessibilityChecker) {
      throw new Error('AccessibilityChecker not initialized');
    }

    try {
      const multi = await this.accessibilityChecker.testMultiplePages(urls, {
        timeout: this.config.timeout,
        maxConcurrent: this.config.maxConcurrent,
        pa11yStandard: this.config.standard,
        verbose: this.config.verbose
      });

      return multi.results.map(result => this.transformToPageAuditResult(result.accessibilityResult));
      
    } catch (error) {
      this.reportError('BROWSER_ERROR', 'Failed to audit pages', undefined, error, false);
      throw error;
    }
  }

  private transformToPageAuditResult(result: any): PageAuditResult {
    // Extract scores safely
    const accessibilityScore = result.pa11yScore || 0;
    const performanceScore = result.enhancedPerformance?.score || result.performance?.score || 0;
    const seoScore = result.enhancedSEO?.score || 0;
    const mobileScore = result.mobileFriendliness?.score || 0;

    // Transform issues
    const issues = {
      errors: (result.errors || []).map((issue: any) => ({
        type: 'accessibility' as const,
        severity: 'error' as const,
        rule: issue.code || 'unknown',
        message: issue.message || 'Unknown error',
        element: issue.element,
        selector: issue.selector
      })),
      warnings: (result.warnings || []).map((warning: any) => ({
        type: 'accessibility' as const,
        severity: 'warning' as const,
        rule: warning.code || 'unknown',
        message: warning.message || 'Unknown warning',
        element: warning.element,
        selector: warning.selector
      })),
      notices: (result.notices || []).map((notice: any) => ({
        type: 'accessibility' as const,
        severity: 'notice' as const,
        rule: notice.code || 'unknown',
        message: notice.message || 'Unknown notice',
        element: notice.element,
        selector: notice.selector
      }))
    };

    return {
      url: result.url,
      title: result.title || 'Unknown Page',
      passed: result.passed || false,
      crashed: result.crashed || false,
      duration: result.duration || 0,
      scores: {
        accessibility: accessibilityScore,
        performance: performanceScore,
        seo: seoScore,
        mobile: mobileScore
      },
      issues,
      metrics: {
        loadTime: result.loadTime || 0,
        contentSize: result.contentWeight?.totalSize || 0,
        resourceCount: result.contentWeight?.totalResources || 0
      }
    };
  }

  private async generateReports(results: PageAuditResult[], sitemapUrl: string): Promise<{ html?: string; markdown?: string }> {
    const dateString = new Date().toISOString().split('T')[0];
    const domain = new URL(sitemapUrl).hostname.replace(/^www\./, '');
    
    const reports: { html?: string; markdown?: string } = {};
    
    try {
      // Ensure output directory exists
      if (!fs.existsSync(this.config.outputDir)) {
        fs.mkdirSync(this.config.outputDir, { recursive: true });
      }
      
      if (this.config.outputFormat === 'html' || this.config.outputFormat === 'both') {
        const htmlFilename = `${this.config.reportPrefix}-${domain}-${dateString}.html`;
        const htmlPath = path.join(this.config.outputDir, htmlFilename);
        
        // Generate HTML report using existing generator
        const htmlGenerator = new HTMLGenerator();
        const summary = this.calculateSummary(results, Date.now());
        const auditResult = {
          summary,
          pages: results as any,
          metadata: {
            version: '2.0.0-alpha.2',
            timestamp: new Date().toISOString(),
            sitemapUrl: sitemapUrl,
            totalPages: summary.totalPages,
            testedPages: summary.testedPages,
            duration: summary.totalDuration
          }
        };
        
        const htmlContent = await htmlGenerator.generate(auditResult);
        fs.writeFileSync(htmlPath, htmlContent, 'utf8');
        
        reports.html = htmlPath;
      }
      
      if (this.config.outputFormat === 'markdown' || this.config.outputFormat === 'both') {
        const mdFilename = `${this.config.reportPrefix}-${domain}-${dateString}.md`;
        const mdPath = path.join(this.config.outputDir, mdFilename);
        
        // Generate Markdown report
        const markdown = this.generateMarkdownReport(results, sitemapUrl);
        fs.writeFileSync(mdPath, markdown, 'utf8');
        
        reports.markdown = mdPath;
      }
      
      return reports;
      
    } catch (error) {
      this.reportError('SYSTEM_ERROR', 'Failed to generate reports', undefined, error, true);
      return {};
    }
  }

  private generateMarkdownReport(results: PageAuditResult[], sitemapUrl: string): string {
    const summary = this.calculateSummary(results, Date.now());
    const domain = new URL(sitemapUrl).hostname;
    
    let markdown = `# Website Audit Report\n\n`;
    markdown += `**Domain:** ${domain}  \n`;
    markdown += `**Date:** ${new Date().toLocaleDateString()}  \n`;
    markdown += `**Pages Tested:** ${summary.testedPages}  \n`;
    markdown += `**Success Rate:** ${summary.successRate.toFixed(1)}%  \n\n`;
    
    markdown += `## Summary\n\n`;
    markdown += `| Metric | Value |\n`;
    markdown += `|--------|-------|\n`;
    markdown += `| Total Pages | ${summary.totalPages} |\n`;
    markdown += `| Tested Pages | ${summary.testedPages} |\n`;
    markdown += `| Passed Pages | ${summary.passedPages} |\n`;
    markdown += `| Failed Pages | ${summary.failedPages} |\n`;
    markdown += `| Crashed Pages | ${summary.crashedPages} |\n`;
    markdown += `| Total Duration | ${(summary.totalDuration / 1000).toFixed(1)}s |\n\n`;
    
    markdown += `## Page Results\n\n`;
    
    for (const result of results) {
      const status = result.crashed ? '💥 CRASHED' : result.passed ? '✅ PASSED' : '❌ FAILED';
      markdown += `### ${result.title}\n`;
      markdown += `**URL:** ${result.url}  \n`;
      markdown += `**Status:** ${status}  \n`;
      markdown += `**Duration:** ${result.duration}ms  \n\n`;
      
      markdown += `**Scores:**\n`;
      markdown += `- Accessibility: ${result.scores.accessibility}/100\n`;
      markdown += `- Performance: ${result.scores.performance}/100\n`;
      markdown += `- SEO: ${result.scores.seo}/100\n`;
      markdown += `- Mobile: ${result.scores.mobile}/100\n\n`;
      
      if (result.issues.errors.length > 0) {
        markdown += `**Errors (${result.issues.errors.length}):**\n`;
        result.issues.errors.slice(0, 5).forEach(error => {
          markdown += `- ${error.message}\n`;
        });
        if (result.issues.errors.length > 5) {
          markdown += `- ... and ${result.issues.errors.length - 5} more\n`;
        }
        markdown += '\n';
      }
    }
    
    return markdown;
  }

  private calculateSummary(results: PageAuditResult[], startTime: number): {
    totalPages: number;
    testedPages: number;
    passedPages: number;
    failedPages: number;
    crashedPages: number;
    successRate: number;
    totalDuration: number;
    averagePageTime: number;
    totalErrors: number;
    totalWarnings: number;
  } {
    const totalDuration = Date.now() - startTime;
    const testedPages = results.length;
    const passedPages = results.filter(r => r.passed).length;
    const failedPages = results.filter(r => !r.passed && !r.crashed).length;
    const crashedPages = results.filter(r => r.crashed).length;
    const totalErrors = results.reduce((sum, r) => sum + r.issues.errors.length, 0);
    const totalWarnings = results.reduce((sum, r) => sum + r.issues.warnings.length, 0);
    
    return {
      totalPages: this.config.maxPages,
      testedPages,
      passedPages,
      failedPages,
      crashedPages,
      successRate: testedPages > 0 ? (passedPages / testedPages) * 100 : 0,
      totalDuration,
      averagePageTime: testedPages > 0 ? totalDuration / testedPages : 0,
      totalErrors,
      totalWarnings
    };
  }

  private calculatePerformanceMetrics(results: PageAuditResult[]): AuditResult['performance'] {
    if (results.length === 0) {
      return {
        avgLoadTime: 0,
        avgAccessibilityScore: 0,
        avgPerformanceScore: 0,
        avgSeoScore: 0
      };
    }

    const totals = results.reduce((acc, result) => ({
      loadTime: acc.loadTime + result.metrics.loadTime,
      accessibility: acc.accessibility + result.scores.accessibility,
      performance: acc.performance + result.scores.performance,
      seo: acc.seo + result.scores.seo
    }), { loadTime: 0, accessibility: 0, performance: 0, seo: 0 });

    return {
      avgLoadTime: totals.loadTime / results.length,
      avgAccessibilityScore: totals.accessibility / results.length,
      avgPerformanceScore: totals.performance / results.length,
      avgSeoScore: totals.seo / results.length
    };
  }

  private reportProgress(phase: AuditProgress['phase'], progress: number, completed: number, total: number, message?: string): void {
    if (this.progressCallback) {
      this.progressCallback({
        phase,
        progress: Math.min(100, Math.max(0, progress)),
        completed,
        total,
        message
      });
    }
  }

  private reportError(code: AuditError['code'], message: string, url?: string, details?: any, recoverable: boolean = false): void {
    const error: AuditError = {
      code,
      message,
      url,
      details,
      recoverable
    };
    
    if (this.errorCallback) {
      this.errorCallback(error);
    } else if (!recoverable) {
      console.error(`🚨 ${code}: ${message}`, details);
    } else {
      console.warn(`⚠️ ${code}: ${message}`, details);
    }
  }
}

// Export default factory function for convenience
export function createStableAuditor(config: StableAuditConfig): StableAuditor {
  return new StableAuditor(config);
}