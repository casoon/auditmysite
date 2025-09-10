/**
 * 🔧 CSV Report Generator
 * 
 * Generates CSV reports for data analysis and spreadsheet integration.
 * Perfect for tracking metrics over time and business reporting.
 */

import { 
  ReportGenerator, 
  ReportData, 
  ReportOptions, 
  GeneratedReport 
} from '../base-generator';
import * as fs from 'fs/promises';
import * as path from 'path';

export class CSVReportGenerator extends ReportGenerator {
  constructor() {
    super('csv');
  }

  getExtension(): string {
    return 'csv';
  }

  getMimeType(): string {
    return 'text/csv';
  }

  async generate(data: ReportData, options: ReportOptions): Promise<GeneratedReport> {
    const startTime = Date.now();

    // Validate data
    const validation = this.validateData(data);
    if (!validation.valid) {
      throw new Error(`Invalid report data: ${validation.errors.join(', ')}`);
    }

    // Generate CSV content
    const csvContent = this.generateCSV(data, options);

    // Create output directory (async)
    await fs.mkdir(options.outputDir, { recursive: true });

    // Write file (async)
    const filename = this.generateFilename(options, 'accessibility');
    const filePath = path.join(options.outputDir, filename);
    await fs.writeFile(filePath, csvContent, 'utf8');

    const duration = Date.now() - startTime;

    return {
      path: filePath,
      format: this.format,
      size: this.calculateFileSize(csvContent),
      metadata: {
        generatedAt: new Date(),
        duration
      }
    };
  }

  private generateCSV(data: ReportData, options: ReportOptions): string {
    const { summary, metadata } = data;
    
    if (options.summaryOnly || !summary.results?.length) {
      return this.generateSummaryCSV(summary, metadata);
    }
    
    return this.generateDetailedCSV(summary, metadata, options);
  }

  private generateSummaryCSV(summary: any, metadata: any): string {
    const successRate = this.calculateSuccessRate(summary);
    
    const headers = [
      'Report Date',
      'Report Time', 
      'Total Pages',
      'Passed Pages',
      'Failed Pages',
      'Crashed Pages',
      'Success Rate %',
      'Total Errors',
      'Total Warnings',
      'Average Duration (ms)',
      'Total Duration (ms)'
    ];

    const date = new Date(metadata.timestamp);
    const values = [
      date.toISOString().split('T')[0], // Date only
      date.toISOString().split('T')[1].split('.')[0], // Time only
      summary.testedPages,
      summary.passedPages,
      summary.failedPages,
      summary.crashedPages || 0,
      successRate,
      summary.totalErrors,
      summary.totalWarnings,
      Math.round(summary.totalDuration / summary.testedPages),
      summary.totalDuration
    ];

    return [
      headers.join(','),
      values.map(v => this.escapeCsvValue(v)).join(',')
    ].join('\n');
  }

  private generateDetailedCSV(summary: any, metadata: any, options: ReportOptions): string {
    const headers = [
      'URL',
      'Page Title',
      'Status',
      'Passed',
      'Crashed',
      'Error Count',
      'Warning Count',
      'Duration (ms)',
      'Timestamp',
      'Page Type'
    ];

    // Add performance headers if available
    const hasPerformanceMetrics = summary.results.some((r: any) => r.performanceMetrics);
    if (hasPerformanceMetrics) {
      headers.push(
        'LCP (ms)',
        'CLS',
        'FID (ms)',
        'TTI (ms)'
      );
    }

    // Add detailed error/warning columns if requested
    if (!options.summaryOnly) {
      headers.push('Errors', 'Warnings');
    }

    // Add pa11y columns if requested
    if (options.includePa11yIssues) {
      headers.push('Pa11y Issues', 'Pa11y Issue Types');
    }

    const rows = [headers.join(',')];

    summary.results.forEach((result: any) => {
      const pageType = this.determinePageType(result.url);
      
      const values = [
        result.url,
        result.title || 'Untitled',
        result.passed ? 'Passed' : (result.crashed ? 'Crashed' : 'Failed'),
        result.passed ? 'TRUE' : 'FALSE',
        result.crashed ? 'TRUE' : 'FALSE',
        result.errors?.length || 0,
        result.warnings?.length || 0,
        result.duration,
        result.timestamp || new Date().toISOString(),
        pageType
      ];

      // Add performance metrics if available
      if (hasPerformanceMetrics) {
        values.push(
          result.performanceMetrics?.largestContentfulPaint || '',
          result.performanceMetrics?.cumulativeLayoutShift || '',
          result.performanceMetrics?.firstInputDelay || '',
          result.performanceMetrics?.timeToInteractive || ''
        );
      }

      // Add detailed errors/warnings
      if (!options.summaryOnly) {
        values.push(
          result.errors?.join('; ') || '',
          result.warnings?.join('; ') || ''
        );
      }

      // Add pa11y data
      if (options.includePa11yIssues) {
        const pa11yIssues = result.pa11yIssues || [];
        const issueMessages = pa11yIssues.map((issue: any) => issue.message).join('; ');
        const issueTypes = [...new Set(pa11yIssues.map((issue: any) => issue.type))].join('; ');
        
        values.push(issueMessages, issueTypes);
      }

      rows.push(values.map(v => this.escapeCsvValue(v)).join(','));
    });

    return rows.join('\n');
  }

  private determinePageType(url: string): string {
    try {
      const pathname = new URL(url).pathname.toLowerCase();
      
      if (pathname === '/' || pathname === '/index' || pathname === '') return 'Home';
      if (pathname.includes('/blog') || pathname.includes('/news') || pathname.includes('/article')) return 'Blog';
      if (pathname.includes('/product') || pathname.includes('/shop') || pathname.includes('/store')) return 'Product';
      if (pathname.includes('/about') || pathname.includes('/team') || pathname.includes('/company')) return 'About';
      if (pathname.includes('/contact') || pathname.includes('/support')) return 'Contact';
      if (pathname.includes('/category') || pathname.includes('/archive')) return 'Category';
      if (pathname.includes('/search') || pathname.includes('/results')) return 'Search';
      
      return 'Other';
    } catch {
      return 'Unknown';
    }
  }

  private escapeCsvValue(value: any): string {
    if (value === null || value === undefined) {
      return '';
    }
    
    const stringValue = String(value);
    
    // If the value contains commas, quotes, or newlines, wrap it in quotes
    if (stringValue.includes(',') || stringValue.includes('"') || stringValue.includes('\n') || stringValue.includes('\r')) {
      // Escape existing quotes by doubling them
      const escapedValue = stringValue.replace(/"/g, '""');
      return `"${escapedValue}"`;
    }
    
    return stringValue;
  }

  /**
   * Generate a separate CSV for issues/recommendations
   */
  generateIssuesCSV(data: ReportData, options: ReportOptions): string {
    const { issues } = data;
    
    if (!issues?.length) {
      return 'Type,Severity,Message,Count,Affected Pages,Recommendation\n';
    }

    const headers = ['Type', 'Severity', 'Message', 'Count', 'Affected Pages', 'Recommendation'];
    const rows = [headers.join(',')];

    issues.forEach(issue => {
      const values = [
        issue.type || 'Unknown',
        issue.severity || 'Error',
        issue.message,
        1, // Default count
        [], // Default affected pages
        issue.recommendation || ''
      ];
      
      rows.push(values.map(v => this.escapeCsvValue(v)).join(','));
    });

    return rows.join('\n');
  }

  /**
   * Generate a separate CSV for performance metrics
   */
  generatePerformanceCSV(data: ReportData): string {
    const { summary } = data;
    
    if (!summary.results?.length) {
      return 'URL,Page Title,Duration (ms),LCP (ms),CLS,FID (ms),TTI (ms)\n';
    }

    const headers = ['URL', 'Page Title', 'Duration (ms)', 'LCP (ms)', 'CLS', 'FID (ms)', 'TTI (ms)'];
    const rows = [headers.join(',')];

    summary.results.forEach((result: any) => {
      if (!result.performanceMetrics) return;
      
      const values = [
        result.url,
        result.title || 'Untitled',
        result.duration,
        result.performanceMetrics.largestContentfulPaint || '',
        result.performanceMetrics.cumulativeLayoutShift || '',
        result.performanceMetrics.firstInputDelay || '',
        result.performanceMetrics.timeToInteractive || ''
      ];
      
      rows.push(values.map(v => this.escapeCsvValue(v)).join(','));
    });

    return rows.join('\n');
  }
}
