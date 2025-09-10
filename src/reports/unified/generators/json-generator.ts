/**
 * 🔧 JSON Report Generator
 * 
 * Generates structured JSON reports for machine consumption and API integrations.
 * Perfect for CI/CD pipelines and automated processing.
 */

import { 
  ReportGenerator, 
  ReportData, 
  ReportOptions, 
  GeneratedReport 
} from '../base-generator';
import * as fs from 'fs/promises';
import * as path from 'path';

export class JSONReportGenerator extends ReportGenerator {
  constructor() {
    super('json');
  }

  getExtension(): string {
    return 'json';
  }

  getMimeType(): string {
    return 'application/json';
  }

  async generate(data: ReportData, options: ReportOptions): Promise<GeneratedReport> {
    const startTime = Date.now();

    // Validate data
    const validation = this.validateData(data);
    if (!validation.valid) {
      throw new Error(`Invalid report data: ${validation.errors.join(', ')}`);
    }

    // Generate JSON report
    const jsonReport = this.generateJSONReport(data, options);

    // Create output directory (async)
    await fs.mkdir(options.outputDir, { recursive: true });

    // Write file (async)
    const filename = this.generateFilename(options, 'accessibility');
    const filePath = path.join(options.outputDir, filename);
    const jsonContent = JSON.stringify(jsonReport, null, options.prettyPrint ? 2 : undefined);
    await fs.writeFile(filePath, jsonContent, 'utf8');

    const duration = Date.now() - startTime;

    return {
      path: filePath,
      format: this.format,
      size: this.calculateFileSize(jsonContent),
      metadata: {
        generatedAt: new Date(),
        duration
      }
    };
  }

  private generateJSONReport(data: ReportData, options: ReportOptions): any {
    const { summary, issues, metadata } = data;
    const successRate = this.calculateSuccessRate(summary);

    const report: any = {
      metadata: {
        version: metadata.version,
        timestamp: metadata.timestamp,
        generatedAt: new Date().toISOString(),
        duration: metadata.duration,
        format: 'json',
        generator: 'AuditMySite JSON Reporter',
        environment: metadata.environment || null,
        sitemapUrl: metadata.sitemapUrl || null,
        reportOptions: {
          summaryOnly: options.summaryOnly,
          includePa11yIssues: options.includePa11yIssues,
          outputDir: options.outputDir,
          prettyPrint: options.prettyPrint
        }
      },
      summary: {
        overallSuccessRate: successRate,
        status: this.getOverallStatus(successRate),
        testedPages: summary.testedPages,
        passedPages: summary.passedPages,
        failedPages: summary.failedPages,
        crashedPages: summary.crashedPages || 0,
        totalErrors: summary.totalErrors,
        totalWarnings: summary.totalWarnings,
        averageTestDuration: Math.round(summary.totalDuration / summary.testedPages),
        totalDuration: summary.totalDuration,
        breakdown: {
          passed: {
            count: summary.passedPages,
            percentage: Math.round((summary.passedPages / summary.testedPages) * 100)
          },
          failed: {
            count: summary.failedPages,
            percentage: Math.round((summary.failedPages / summary.testedPages) * 100)
          },
          crashed: {
            count: summary.crashedPages || 0,
            percentage: Math.round(((summary.crashedPages || 0) / summary.testedPages) * 100)
          }
        }
      },
      statistics: {
        errorDistribution: this.calculateErrorDistribution(summary.results || []),
        performanceMetrics: this.calculatePerformanceStats(summary.results || []),
        pageTypeBreakdown: this.calculatePageTypeBreakdown(summary.results || [])
      }
    };

    // Add detailed results if not summary-only
    if (!options.summaryOnly && summary.results?.length > 0) {
      report.results = this.processResults(summary.results, options);
    }

    // Add issues if available
    if (issues?.length > 0) {
      report.issues = this.processIssues(issues);
    }

    // Add branding if provided
    if (options.branding) {
      report.branding = {
        company: options.branding.company,
        logo: options.branding.logo,
        footer: options.branding.footer
      };
    }

    return report;
  }

  private getOverallStatus(successRate: number): string {
    if (successRate >= 90) return 'excellent';
    if (successRate >= 70) return 'good';
    if (successRate >= 50) return 'fair';
    return 'poor';
  }

  private calculateErrorDistribution(results: any[]): any {
    const distribution: { [key: string]: number } = {};
    let totalErrors = 0;

    results.forEach(result => {
      const errorCount = result.errors?.length || 0;
      totalErrors += errorCount;
      
      if (errorCount === 0) {
        distribution['0'] = (distribution['0'] || 0) + 1;
      } else if (errorCount <= 5) {
        distribution['1-5'] = (distribution['1-5'] || 0) + 1;
      } else if (errorCount <= 10) {
        distribution['6-10'] = (distribution['6-10'] || 0) + 1;
      } else if (errorCount <= 20) {
        distribution['11-20'] = (distribution['11-20'] || 0) + 1;
      } else {
        distribution['20+'] = (distribution['20+'] || 0) + 1;
      }
    });

    return {
      byRange: distribution,
      totalErrors,
      averageErrorsPerPage: results.length > 0 ? Math.round(totalErrors / results.length) : 0
    };
  }

  private calculatePerformanceStats(results: any[]): any {
    const durations = results.map(r => r.duration).filter(d => typeof d === 'number');
    const lcpValues = results.map(r => r.performanceMetrics?.largestContentfulPaint).filter(lcp => typeof lcp === 'number');
    const clsValues = results.map(r => r.performanceMetrics?.cumulativeLayoutShift).filter(cls => typeof cls === 'number');

    return {
      testDuration: {
        min: durations.length > 0 ? Math.min(...durations) : 0,
        max: durations.length > 0 ? Math.max(...durations) : 0,
        average: durations.length > 0 ? Math.round(durations.reduce((a, b) => a + b, 0) / durations.length) : 0,
        median: durations.length > 0 ? this.calculateMedian(durations) : 0
      },
      largestContentfulPaint: {
        min: lcpValues.length > 0 ? Math.min(...lcpValues) : null,
        max: lcpValues.length > 0 ? Math.max(...lcpValues) : null,
        average: lcpValues.length > 0 ? Math.round(lcpValues.reduce((a, b) => a + b, 0) / lcpValues.length) : null,
        median: lcpValues.length > 0 ? this.calculateMedian(lcpValues) : null
      },
      cumulativeLayoutShift: {
        min: clsValues.length > 0 ? Math.min(...clsValues) : null,
        max: clsValues.length > 0 ? Math.max(...clsValues) : null,
        average: clsValues.length > 0 ? +(clsValues.reduce((a, b) => a + b, 0) / clsValues.length).toFixed(3) : null,
        median: clsValues.length > 0 ? +this.calculateMedian(clsValues).toFixed(3) : null
      }
    };
  }

  private calculatePageTypeBreakdown(results: any[]): any {
    const types: { [key: string]: { count: number; passed: number; failed: number } } = {};

    results.forEach(result => {
      const type = this.determinePageType(result.url);
      
      if (!types[type]) {
        types[type] = { count: 0, passed: 0, failed: 0 };
      }
      
      types[type].count++;
      if (result.passed) {
        types[type].passed++;
      } else {
        types[type].failed++;
      }
    });

    // Convert to array format with percentages
    return Object.entries(types).map(([type, stats]) => ({
      type,
      count: stats.count,
      passed: stats.passed,
      failed: stats.failed,
      successRate: Math.round((stats.passed / stats.count) * 100)
    })).sort((a, b) => b.count - a.count);
  }

  private determinePageType(url: string): string {
    const pathname = new URL(url).pathname.toLowerCase();
    
    if (pathname === '/' || pathname === '/index' || pathname === '') return 'home';
    if (pathname.includes('/blog') || pathname.includes('/news') || pathname.includes('/article')) return 'blog';
    if (pathname.includes('/product') || pathname.includes('/shop') || pathname.includes('/store')) return 'product';
    if (pathname.includes('/about') || pathname.includes('/team') || pathname.includes('/company')) return 'about';
    if (pathname.includes('/contact') || pathname.includes('/support')) return 'contact';
    if (pathname.includes('/category') || pathname.includes('/archive')) return 'category';
    if (pathname.includes('/search') || pathname.includes('/results')) return 'search';
    
    return 'other';
  }

  private calculateMedian(values: number[]): number {
    const sorted = [...values].sort((a, b) => a - b);
    const mid = Math.floor(sorted.length / 2);
    
    if (sorted.length % 2 === 0) {
      return (sorted[mid - 1] + sorted[mid]) / 2;
    }
    return sorted[mid];
  }

  private processResults(results: any[], options: ReportOptions): any[] {
    return results.map(result => {
      const processedResult: any = {
        url: result.url,
        title: result.title || null,
        status: result.passed ? 'passed' : (result.crashed ? 'crashed' : 'failed'),
        passed: result.passed,
        crashed: result.crashed || false,
        duration: result.duration,
        timestamp: result.timestamp || new Date().toISOString(),
        errorCount: result.errors?.length || 0,
        warningCount: result.warnings?.length || 0,
        errors: result.errors || [],
        warnings: result.warnings || []
      };

      // Add performance metrics if available
      if (result.performanceMetrics) {
        processedResult.performanceMetrics = {
          largestContentfulPaint: result.performanceMetrics.largestContentfulPaint || null,
          cumulativeLayoutShift: result.performanceMetrics.cumulativeLayoutShift || null,
          firstInputDelay: result.performanceMetrics.firstInputDelay || null,
          timeToInteractive: result.performanceMetrics.timeToInteractive || null
        };
      }

      // Add pa11y issues if requested and available
      if (options.includePa11yIssues && result.pa11yIssues?.length > 0) {
        processedResult.pa11yIssues = result.pa11yIssues.map((issue: any) => ({
          type: issue.type,
          message: issue.message,
          selector: issue.selector || null,
          context: issue.context || null,
          code: issue.code || null,
          runner: issue.runner || null
        }));
      }

      return processedResult;
    });
  }

  private processIssues(issues: any[]): any[] {
    return issues.map(issue => ({
      type: issue.type || 'unknown',
      severity: issue.severity || 'error',
      message: issue.message,
      count: issue.count || 1,
      affectedPages: issue.affectedPages || [],
      recommendation: issue.recommendation || null
    }));
  }
}
