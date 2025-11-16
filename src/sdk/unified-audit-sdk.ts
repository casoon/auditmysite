/* eslint-disable @typescript-eslint/no-require-imports */
/**
 * 🎯 AuditMySite SDK - Single Source of Truth Interface
 * 
 * This SDK uses ONLY the unified export types and serves as the
 * single entry point for all programmatic access to AuditMySite.
 * 
 * Used by:
 * - Node.js SDK
 * - CLI tool
 * - REST API server
 * - External integrations
 */

import { 
  AuditRequest,
  AuditResponse,
  AuditOptions,
  AuditProgress,
  UnifiedReportExport,
  GeneratedFiles,
  AuditError,
  VersionInfo,
  ReportExportValidator
} from '../reports/types/report-export';
import { UnifiedReportExporter } from '../reports/exporters/unified-export';
import { TestSummary } from '../types';

export class AuditMySiteSDK {
  private version: string;
  private exporter: UnifiedReportExporter;
  
  constructor(version: string = '1.8.8') {
    this.version = version;
    this.exporter = new UnifiedReportExporter(version);
  }

  /**
   * Get version information
   */
  getVersion(): VersionInfo {
    return {
      version: this.version,
      nodeVersion: process.version,
      apiVersion: '1.0',
      features: [
        'accessibility-testing',
        'performance-metrics',
        'screenshot-capture',
        'keyboard-navigation',
        'color-contrast',
        'focus-management',
        'multiple-formats'
      ]
    };
  }

  /**
   * Validate audit request
   */
  validateRequest(request: any): { valid: boolean; error?: string } {
    if (!ReportExportValidator.validateAuditRequest(request)) {
      return { valid: false, error: 'Invalid audit request: missing or invalid URL' };
    }

    if (request.options && !ReportExportValidator.validateAuditOptions(request.options)) {
      return { valid: false, error: 'Invalid audit options' };
    }

    return { valid: true };
  }

  /**
   * Execute audit (main SDK function)
   */
  async audit(
    request: AuditRequest, 
    progressCallback?: (progress: AuditProgress) => void
  ): Promise<AuditResponse> {
    try {
      // Validate request
      const validation = this.validateRequest(request);
      if (!validation.valid) {
        const error: AuditError = {
          code: 'INVALID_REQUEST',
          message: validation.error!
        };
        
        return {
          status: 'error',
          report: ReportExportValidator.createErrorReport(error, request.url),
          error
        };
      }

      // Progress callback setup
      if (progressCallback) {
        progressCallback({
          step: 'discovering',
          progress: 10,
          message: 'Discovering pages...',
          pagesCompleted: 0,
          totalPages: 0
        });
      }

      // Execute the actual audit using existing pipeline
      const testResult = await this.executeAudit(request, progressCallback);
      
      if (progressCallback) {
        progressCallback({
          step: 'generating',
          progress: 90,
          message: 'Generating reports...',
          pagesCompleted: testResult.summary.testedPages,
          totalPages: testResult.summary.totalPages
        });
      }

      // Convert to unified format
      const unifiedReport = this.exporter.exportUnified(
        testResult.summary,
        { 
          timestamp: new Date().toISOString(),
          sitemapUrl: request.url,
          ...request.metadata
        },
        request.options
      );

      // Generate files if requested
      let files: GeneratedFiles | undefined;
      if (request.options?.outputFormats?.length) {
        files = await this.generateFiles(unifiedReport, testResult.summary, request.options);
      }

      if (progressCallback) {
        progressCallback({
          step: 'completed',
          progress: 100,
          message: 'Audit completed successfully',
          pagesCompleted: testResult.summary.testedPages,
          totalPages: testResult.summary.totalPages
        });
      }

      return {
        status: 'success',
        report: unifiedReport,
        files
      };

    } catch (error: any) {
      const auditError: AuditError = {
        code: error.code || 'AUDIT_FAILED',
        message: error.message || 'Audit execution failed',
        details: error.details,
        stack: error.stack
      };

      return {
        status: 'error',
        report: ReportExportValidator.createErrorReport(auditError, request.url),
        error: auditError
      };
    }
  }

  /**
   * Execute audit using existing pipeline
   */
  private async executeAudit(
    request: AuditRequest,
    progressCallback?: (progress: AuditProgress) => void
  ): Promise<{ summary: TestSummary; outputFiles: string[] }> {
    // Import the existing pipeline
    const { StandardPipeline } = require('../core/pipeline/standard-pipeline');
    
    // Convert SDK options to pipeline options
    const pipelineOptions = this.convertToPipelineOptions(request.options);
    
    const pipeline = new StandardPipeline();
    
    if (progressCallback) {
      progressCallback({
        step: 'testing',
        progress: 30,
        message: 'Running accessibility tests...',
        pagesCompleted: 0,
        totalPages: 0
      });
    }
    
    return await pipeline.execute(request.url, pipelineOptions);
  }

  /**
   * Convert SDK options to pipeline options
   */
  private convertToPipelineOptions(options?: AuditOptions): any {
    if (!options) return {};
    
    return {
      maxPages: options.maxPages || 50,
      timeout: options.timeout || 10000,
      pa11yStandard: options.pa11yStandard || 'WCAG2AA',
      collectPerformanceMetrics: options.collectPerformanceMetrics || false,
      captureScreenshots: options.captureScreenshots || false,
      testKeyboardNavigation: options.testKeyboardNavigation || false,
      testColorContrast: options.testColorContrast || false,
      testFocusManagement: options.testFocusManagement || false,
      outputFormat: options.outputFormats?.includes('html') ? 'html' : 'markdown'
    };
  }

  /**
   * Generate output files
   */
  private async generateFiles(
    report: UnifiedReportExport, 
    testSummary: TestSummary,
    options: AuditOptions
  ): Promise<GeneratedFiles> {
    const files: GeneratedFiles = {};
    const outputDir = options.outputDir || './reports';
    const domain = report.metadata.domain;
    const dateStr = new Date().toISOString().split('T')[0];

    // Ensure output directory exists
    const fs = await import('fs/promises');
    const path = await import('path');
    await fs.mkdir(outputDir, { recursive: true });

    // Generate JSON (always available)
    if (options.outputFormats?.includes('json')) {
      const jsonPath = path.join(outputDir, `${domain}-audit-${dateStr}.json`);
      await this.exporter.saveJsonExport(testSummary, jsonPath, {
        timestamp: report.metadata.timestamp,
        sitemapUrl: report.metadata.sourceUrl
      }, options);
      files.json = jsonPath;
    }

    // Generate HTML
    if (options.outputFormats?.includes('html')) {
      const htmlPath = path.join(outputDir, `${domain}-audit-${dateStr}.html`);
      await this.generateHtmlReport(report, htmlPath);
      files.html = htmlPath;
    }

    // Generate Markdown
    if (options.outputFormats?.includes('markdown')) {
      const mdPath = path.join(outputDir, `${domain}-audit-${dateStr}.md`);
      await this.generateMarkdownReport(report, mdPath);
      files.markdown = mdPath;
    }

    // Generate CSV
    if (options.outputFormats?.includes('csv')) {
      const csvPath = path.join(outputDir, `${domain}-audit-${dateStr}.csv`);
      await this.generateCsvReport(report, csvPath);
      files.csv = csvPath;
    }

    return files;
  }

  /**
   * Generate HTML report using unified data
   */
  private async generateHtmlReport(report: UnifiedReportExport, outputPath: string): Promise<void> {
    // Use HTMLGenerator (current standard)
    const { HTMLGenerator } = require('../generators/html-generator');
    const generator = new HTMLGenerator();
    // Note: May need adapter for unified report format
    await generator.generate(report as any);
  }

  /**
   * Generate Markdown report using unified data
   */
  private async generateMarkdownReport(report: UnifiedReportExport, outputPath: string): Promise<void> {
    const { ModernMarkdownReportGenerator } = require('../reports/generators/modern-markdown-generator');
    const generator = new ModernMarkdownReportGenerator();
    await generator.generateFromUnified(report, outputPath);
  }

  /**
   * Generate CSV report using unified data
   */
  private async generateCsvReport(report: UnifiedReportExport, outputPath: string): Promise<void> {
    const { ModernCsvReportGenerator } = require('../reports/generators/modern-csv-generator');
    const generator = new ModernCsvReportGenerator();
    await generator.generateFromUnified(report, outputPath);
  }

  /**
   * Get report summary without full audit (for quick checks)
   */
  async getReportSummary(request: AuditRequest): Promise<{
    domain: string;
    estimatedPages: number;
    estimatedDuration: number;
  }> {
    const { SitemapParser } = require('../core/utils/sitemap-parser');
    const parser = new SitemapParser();
    
    try {
      const urls = await parser.parseSitemap(request.url, request.options?.maxPages || 50);
      const domain = new URL(request.url).hostname;
      
      return {
        domain,
        estimatedPages: urls.length,
        estimatedDuration: urls.length * (request.options?.timeout || 10000)
      };
    } catch {
      return {
        domain: 'unknown',
        estimatedPages: 1,
        estimatedDuration: request.options?.timeout || 10000
      };
    }
  }

  /**
   * Export report data in different formats
   */
  exportData(report: UnifiedReportExport, format: 'json' | 'minimal'): string | object {
    switch (format) {
      case 'json':
        return JSON.stringify(report, null, 2);
      
      case 'minimal':
        return {
          domain: report.metadata.domain,
          timestamp: report.metadata.timestamp,
          summary: {
            successRate: report.summary.successRate,
            totalPages: report.summary.totalPages,
            totalErrors: report.summary.totalErrors,
            avgScore: report.summary.avgAccessibilityScore
          },
          recommendations: report.recommendations.length
        };
      
      default:
        return report;
    }
  }
}

// Export singleton instance for convenience
export const auditSDK = new AuditMySiteSDK();

// Export all types for external use
export * from '../reports/types/report-export';
