/**
 * 🔧 AuditMySite SDK
 * 
 * Main SDK class providing a fluent, chainable API for programmatic 
 * accessibility testing. Designed for easy integration into any Node.js application.
 */

import { EventEmitter } from 'events';
import { v4 as uuidv4 } from 'uuid';
import {
  SDKConfig,
  AuditOptions,
  AuditResult,
  AuditBuilder,
  AuditCallbacks,
  AuditEvent,
  AuditEventType,
  EventCallback,
  ProgressData,
  PageStartData,
  PageCompleteData,
  PageErrorData,
  ReportFormat,
  AuditSDKError,
  InvalidSitemapError,
  ConfigurationError,
  GeneratedReport
} from './types';
import { StandardPipeline } from '../core/pipeline/standard-pipeline';
// Report system imports removed
import { ConfigManager } from '../core/config/config-manager';
import * as path from 'path';

export class AuditSDK extends EventEmitter {
  private config: SDKConfig;
  private configManager: ConfigManager;
  // Report system removed - using direct generation
  
  constructor(config: SDKConfig = {}) {
    super();
    this.config = this.mergeConfig(config);
    this.configManager = new ConfigManager();
    // Report system initialization removed
    
    // Set default max listeners for heavy event usage
    this.setMaxListeners(50);
  }

  /**
   * Create a new audit builder for fluent configuration
   */
  audit(): AuditBuilder {
    return new FluentAuditBuilder(this);
  }

  /**
   * Quick audit method for simple use cases
   */
  async quickAudit(
    sitemapUrl: string, 
    options: AuditOptions = {},
    callbacks?: AuditCallbacks
  ): Promise<AuditResult> {
    const builder = this.audit()
      .sitemap(sitemapUrl);
    
    // Apply options
    if (options.maxPages) builder.maxPages(options.maxPages);
    if (options.standard) builder.standard(options.standard);
    if (options.formats) builder.formats(options.formats);
    if (options.outputDir) builder.outputDir(options.outputDir);
    if (options.includePerformance) builder.includePerformance();
    if (options.includeSeo) builder.includeSeo();
    if (options.includeSecurity) builder.includeSecurity();
    if (options.viewport) {
      builder.viewport(
        options.viewport.width, 
        options.viewport.height, 
        options.viewport.isMobile
      );
    }

    // Apply callbacks
    if (callbacks) {
      if (callbacks.onStart) builder.on('audit:start', callbacks.onStart);
      if (callbacks.onProgress) builder.on('audit:progress', callbacks.onProgress);
      if (callbacks.onPageStart) builder.on('audit:page:start', callbacks.onPageStart);
      if (callbacks.onPageComplete) builder.on('audit:page:complete', callbacks.onPageComplete);
      if (callbacks.onPageError) builder.on('audit:page:error', callbacks.onPageError);
      if (callbacks.onComplete) builder.on('audit:complete', callbacks.onComplete);
      if (callbacks.onError) builder.on('audit:error', callbacks.onError);
      if (callbacks.onReportStart) builder.on('report:start', callbacks.onReportStart);
      if (callbacks.onReportComplete) builder.on('report:complete', callbacks.onReportComplete);
    }

    return builder.run();
  }

  /**
   * Update SDK configuration
   */
  configure(config: Partial<SDKConfig>): this {
    this.config = this.mergeConfig(config);
    return this;
  }

  /**
   * Get current SDK configuration
   */
  getConfig(): Readonly<SDKConfig> {
    return { ...this.config };
  }

  /**
   * Get SDK version
   */
  getVersion(): string {
    return require('../../package.json').version;
  }

  /**
   * Test connection to sitemap
   */
  async testConnection(sitemapUrl: string): Promise<{ success: boolean; error?: string }> {
    try {
      const { SitemapDiscovery } = await import('../core/parsers/sitemap-discovery');
      const discovery = new SitemapDiscovery();
      const result = await discovery.discoverSitemap(sitemapUrl);
      
      return { success: result.found };
    } catch (error) {
      return { 
        success: false, 
        error: error instanceof Error ? error.message : String(error) 
      };
    }
  }

  private mergeConfig(config: Partial<SDKConfig>): SDKConfig {
    return {
      timeout: 30000,
      maxConcurrency: 3,
      defaultOutputDir: './audit-results',
      verbose: false,
      userAgent: `AuditMySite-SDK/${this.getVersion()}`,
      ...config
    };
  }

  private emitEvent<T>(type: AuditEventType, data: T, sessionId?: string): void {
    const event: AuditEvent<T> = {
      type,
      timestamp: new Date(),
      data,
      sessionId
    };
    
    this.emit(type, event);
    this.emit('event', event);
  }
}

/**
 * Fluent builder implementation for chainable audit configuration
 */
class FluentAuditBuilder implements AuditBuilder {
  private options: AuditOptions = {};
  private sitemapUrl: string = '';
  private callbacks = new Map<AuditEventType, EventCallback[]>();
  
  constructor(private sdk: AuditSDK) {}

  sitemap(url: string): AuditBuilder {
    this.validateSitemapUrl(url);
    this.sitemapUrl = url;
    return this;
  }

  maxPages(count: number): AuditBuilder {
    if (count < 1 || count > 10000) {
      throw new ConfigurationError('maxPages must be between 1 and 10000');
    }
    this.options.maxPages = count;
    return this;
  }

  standard(standard: 'WCAG2AA' | 'WCAG2AAA' | 'Section508'): AuditBuilder {
    this.options.standard = standard;
    return this;
  }

  formats(formats: ReportFormat[]): AuditBuilder {
    if (!Array.isArray(formats) || formats.length === 0) {
      throw new ConfigurationError('formats must be a non-empty array');
    }
    this.options.formats = formats;
    return this;
  }

  outputDir(dir: string): AuditBuilder {
    this.options.outputDir = dir;
    return this;
  }

  includePerformance(include: boolean = true): AuditBuilder {
    this.options.includePerformance = include;
    return this;
  }

  includeSeo(include: boolean = true): AuditBuilder {
    this.options.includeSeo = include;
    return this;
  }

  includeSecurity(include: boolean = true): AuditBuilder {
    this.options.includeSecurity = include;
    return this;
  }

  viewport(width: number, height: number, mobile?: boolean): AuditBuilder {
    this.options.viewport = {
      width,
      height,
      isMobile: mobile || false
    };
    return this;
  }

  timeout(ms: number): AuditBuilder {
    if (ms < 1000 || ms > 300000) {
      throw new ConfigurationError('timeout must be between 1000ms and 300000ms');
    }
    this.sdk.configure({ timeout: ms });
    return this;
  }

  on<T>(event: AuditEventType, callback: EventCallback<T>): AuditBuilder {
    if (!this.callbacks.has(event)) {
      this.callbacks.set(event, []);
    }
    this.callbacks.get(event)!.push(callback as EventCallback);
    return this;
  }

  async run(): Promise<AuditResult> {
    if (!this.sitemapUrl) {
      throw new ConfigurationError('sitemap URL is required');
    }

    const sessionId = uuidv4();
    const startTime = new Date();

    // Register callbacks
    this.callbacks.forEach((callbacks, event) => {
      callbacks.forEach(callback => {
        this.sdk.on(event, callback);
      });
    });

    try {
      // Emit start event
      this.sdk.emit('audit:start', {
        type: 'audit:start',
        timestamp: startTime,
        data: { sessionId, sitemapUrl: this.sitemapUrl, options: this.options },
        sessionId
      });

      // Merge options with defaults
      const finalOptions = this.mergeWithDefaults();

      // Run the audit
      const pipeline = new StandardPipeline();
      const pipelineResult = await pipeline.run({
        sitemapUrl: this.sitemapUrl,
        maxPages: finalOptions.maxPages,
        pa11yStandard: finalOptions.standard,
        outputDir: finalOptions.outputDir,
        generatePerformanceReport: finalOptions.includePerformance,
        // generateSeoReport: finalOptions.includeSeo, // Removed for compatibility
        // generateSecurityReport: finalOptions.includeSecurity, // Removed - not supported in StandardPipelineOptions
        // usePa11y: finalOptions.usePa11y, // Removed - not in StandardPipelineOptions
        timeout: this.sdk.getConfig().timeout,
        maxConcurrent: this.sdk.getConfig().maxConcurrency,
        // verbose: this.sdk.getConfig().verbose, // Not in StandardPipelineOptions
        // Modern queue system is default
        // timestamp: startTime.toISOString(), // Not in StandardPipelineOptions
        // onProgress callback removed - not supported in StandardPipelineOptions
      });

      // Generate reports if formats specified
      const reports = await this.generateReports(pipelineResult, finalOptions, sessionId);

      const endTime = new Date();
      const result: AuditResult = {
        sessionId,
        sitemapUrl: this.sitemapUrl,
        startTime,
        endTime,
        duration: endTime.getTime() - startTime.getTime(),
        summary: pipelineResult.summary,
        results: (pipelineResult.summary.results || []).map((result: any) => ({
          ...result,
          timestamp: result.timestamp || new Date().toISOString()
        })),
        reports,
        metadata: {
          version: this.sdk.getVersion(),
          environment: process.env.NODE_ENV || 'development',
          userAgent: this.sdk.getConfig().userAgent || '',
          configuration: finalOptions
        }
      };

      // Emit complete event
      this.sdk.emit('audit:complete', {
        type: 'audit:complete',
        timestamp: endTime,
        data: result,
        sessionId
      });

      return result;

    } catch (error) {
      // Emit error event
      this.sdk.emit('audit:error', {
        type: 'audit:error',
        timestamp: new Date(),
        data: error,
        sessionId
      });

      throw error instanceof AuditSDKError ? error : new AuditSDKError(
        error instanceof Error ? error.message : String(error),
        'AUDIT_FAILED',
        error
      );
    } finally {
      // Clean up callbacks
      this.callbacks.forEach((callbacks, event) => {
        callbacks.forEach(callback => {
          this.sdk.removeListener(event, callback);
        });
      });
    }
  }

  private validateSitemapUrl(url: string): void {
    try {
      new URL(url);
    } catch {
      throw new InvalidSitemapError(url, 'Invalid URL format');
    }

    if (!url.startsWith('http://') && !url.startsWith('https://')) {
      throw new InvalidSitemapError(url, 'URL must use HTTP or HTTPS protocol');
    }
  }

  private mergeWithDefaults(): Required<AuditOptions> {
    const config = this.sdk.getConfig();
    return {
      // Basic options
      maxPages: this.options.maxPages || 10,
      standard: this.options.standard || 'WCAG2AA',
      formats: this.options.formats || ['html'],
      outputDir: this.options.outputDir || config.defaultOutputDir || './audit-results',
      
      // Legacy compatibility flags (backward compatibility)
      includePerformance: this.options.includePerformance ?? (this.options.performance ?? true),
      includeSeo: this.options.includeSeo ?? (this.options.seo ?? true),
      includeSecurity: this.options.includeSecurity ?? false,
      
      // New unified feature flags (enhanced methods are default in Standard)
      accessibility: this.options.accessibility ?? true,
      performance: this.options.performance ?? true,
      seo: this.options.seo ?? true,
      contentWeight: this.options.contentWeight ?? true,
      reduced: this.options.reduced ?? false,
      includeRecommendations: this.options.includeRecommendations ?? true,
      
      // Technical options
      usePa11y: this.options.usePa11y ?? true,
      pa11yOptions: this.options.pa11yOptions || {},
      performanceBudget: this.options.performanceBudget || {},
      viewport: this.options.viewport || {
        width: 1920,
        height: 1080,
        deviceScaleFactor: 1,
        isMobile: false
      }
    };
  }

  private calculateETA(current: number, total: number, startTime: Date): number {
    if (current === 0) return 0;
    
    const elapsed = Date.now() - startTime.getTime();
    const avgTimePerPage = elapsed / current;
    const remaining = total - current;
    
    return Math.round(remaining * avgTimePerPage);
  }

  private async generateReports(
    pipelineResult: any, 
    options: Required<AuditOptions>, 
    sessionId: string
  ): Promise<GeneratedReport[]> {
    if (!options.formats || options.formats.length === 0) {
      return [];
    }

    // Report generation simplified - using pipeline output directly

    // Return pipeline output files as reports
    const reports: GeneratedReport[] = (pipelineResult.outputFiles || []).map((filePath: string) => ({
      format: path.extname(filePath).slice(1) as any,
      path: filePath,
      size: 0, // Size calculation can be added later
      metadata: {}
    }));

    return reports;
  }
}
