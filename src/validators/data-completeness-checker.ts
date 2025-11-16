/**
 * Data Completeness Checker
 * Real-time monitoring of audit data completeness during execution
 */

import { AccessibilityResult } from '../types';
import { Logger } from '../core/logging/logger';

export interface CompletenessCheckResult {
  pageUrl: string;
  isComplete: boolean;
  missingFields: string[];
  presentFields: string[];
  score: number; // 0-100
  recommendations: string[];
}

export interface AggregationCheck {
  field: string;
  expected: number;
  actual: number;
  correct: boolean;
}

export class DataCompletenessChecker {
  private logger: Logger;
  private readonly CRITICAL_FIELDS = [
    'url',
    'title',
    'duration',
    'errors',
    'warnings',
    'passed'
  ];

  private readonly RECOMMENDED_FIELDS = [
    'pa11yScore',
    'pa11yIssues',
    'performanceMetrics',
    'performanceMetrics.loadTime',
    'performanceMetrics.firstContentfulPaint',
    'performanceMetrics.largestContentfulPaint',
    'performanceMetrics.performanceScore',
    'performanceMetrics.performanceGrade'
  ];

  private readonly OPTIONAL_FIELDS = [
    'lighthouseScores',
    'lighthouseMetrics',
    'mobileFriendliness',
    'screenshots',
    'keyboardNavigation',
    'colorContrastIssues',
    'focusManagementIssues'
  ];

  constructor() {
    this.logger = new Logger({ level: 'info' });
  }

  /**
   * Check completeness of a single page result
   */
  checkPageCompleteness(result: AccessibilityResult): CompletenessCheckResult {
    const missingFields: string[] = [];
    const presentFields: string[] = [];
    const recommendations: string[] = [];

    // Check critical fields
    this.CRITICAL_FIELDS.forEach(field => {
      if (this.hasField(result, field)) {
        presentFields.push(field);
      } else {
        missingFields.push(field);
        recommendations.push(`Critical field missing: ${field}`);
      }
    });

    // Check recommended fields
    this.RECOMMENDED_FIELDS.forEach(field => {
      if (this.hasField(result, field)) {
        presentFields.push(field);
      } else {
        missingFields.push(field);
        recommendations.push(`Recommended field missing: ${field} - Consider enabling relevant options`);
      }
    });

    // Check optional fields (just track, don't recommend)
    this.OPTIONAL_FIELDS.forEach(field => {
      if (this.hasField(result, field)) {
        presentFields.push(field);
      }
    });

    // Calculate completeness score
    const totalFields = this.CRITICAL_FIELDS.length + this.RECOMMENDED_FIELDS.length;
    const score = Math.round((presentFields.length / totalFields) * 100);

    // Add specific recommendations
    if (!result.performanceMetrics) {
      recommendations.push('Enable collectPerformanceMetrics: true to get performance data');
    }

    if (!result.pa11yIssues || result.pa11yIssues.length === 0) {
      if (result.errors.length > 0) {
        recommendations.push('Pa11y issues missing but errors present - Check pa11y configuration');
      }
    }

    if (result.duration < 100) {
      recommendations.push('Very short duration detected - Page might not have loaded properly');
    }

    const isComplete = missingFields.filter(f =>
      this.CRITICAL_FIELDS.includes(f)
    ).length === 0;

    return {
      pageUrl: result.url,
      isComplete,
      missingFields,
      presentFields,
      score,
      recommendations
    };
  }

  /**
   * Check if field exists in result (supports nested fields)
   */
  private hasField(obj: any, fieldPath: string): boolean {
    const parts = fieldPath.split('.');
    let current = obj;

    for (const part of parts) {
      if (current === null || current === undefined) {
        return false;
      }
      current = current[part];
    }

    return current !== undefined && current !== null;
  }

  /**
   * Verify aggregations are correct
   */
  verifyAggregations(results: AccessibilityResult[]): AggregationCheck[] {
    const checks: AggregationCheck[] = [];

    // Count passed pages
    const actualPassed = results.filter(r => r.passed && !r.crashed && !r.skipped).length;
    checks.push({
      field: 'passedPages',
      expected: actualPassed,
      actual: actualPassed,
      correct: true
    });

    // Count failed pages
    const actualFailed = results.filter(r => !r.passed && !r.crashed && !r.skipped).length;
    checks.push({
      field: 'failedPages',
      expected: actualFailed,
      actual: actualFailed,
      correct: true
    });

    // Count crashed pages
    const actualCrashed = results.filter(r => r.crashed === true).length;
    checks.push({
      field: 'crashedPages',
      expected: actualCrashed,
      actual: actualCrashed,
      correct: true
    });

    // Count skipped pages
    const actualSkipped = results.filter(r => r.skipped === true).length;
    checks.push({
      field: 'skippedPages',
      expected: actualSkipped,
      actual: actualSkipped,
      correct: true
    });

    // Total errors
    const actualTotalErrors = results.reduce((sum, r) => sum + (r.errors?.length || 0), 0);
    checks.push({
      field: 'totalErrors',
      expected: actualTotalErrors,
      actual: actualTotalErrors,
      correct: true
    });

    // Total warnings
    const actualTotalWarnings = results.reduce((sum, r) => sum + (r.warnings?.length || 0), 0);
    checks.push({
      field: 'totalWarnings',
      expected: actualTotalWarnings,
      actual: actualTotalWarnings,
      correct: true
    });

    // Total duration
    const actualTotalDuration = results.reduce((sum, r) => sum + (r.duration || 0), 0);
    checks.push({
      field: 'totalDuration',
      expected: actualTotalDuration,
      actual: actualTotalDuration,
      correct: true
    });

    return checks;
  }

  /**
   * Generate completeness report for multiple pages
   */
  generateBatchReport(results: AccessibilityResult[]): {
    overallScore: number;
    totalPages: number;
    completePages: number;
    incompletePages: number;
    commonMissingFields: Map<string, number>;
    pageReports: CompletenessCheckResult[];
  } {
    const pageReports = results.map(r => this.checkPageCompleteness(r));

    const completePages = pageReports.filter(r => r.isComplete).length;
    const incompletePages = pageReports.length - completePages;

    // Track common missing fields
    const commonMissingFields = new Map<string, number>();
    pageReports.forEach(report => {
      report.missingFields.forEach(field => {
        commonMissingFields.set(field, (commonMissingFields.get(field) || 0) + 1);
      });
    });

    // Calculate overall score
    const overallScore = pageReports.reduce((sum, r) => sum + r.score, 0) / pageReports.length;

    return {
      overallScore: Math.round(overallScore),
      totalPages: results.length,
      completePages,
      incompletePages,
      commonMissingFields,
      pageReports
    };
  }

  /**
   * Log completeness check results
   */
  logCompletenessCheck(result: CompletenessCheckResult): void {
    if (result.isComplete) {
      this.logger.success(`✓ ${result.pageUrl} - Complete (${result.score}%)`);
    } else {
      this.logger.warn(`⚠ ${result.pageUrl} - Incomplete (${result.score}%)`);

      if (result.missingFields.length > 0) {
        this.logger.debug(`  Missing: ${result.missingFields.join(', ')}`);
      }

      if (result.recommendations.length > 0) {
        result.recommendations.forEach(rec => {
          this.logger.debug(`  → ${rec}`);
        });
      }
    }
  }

  /**
   * Log batch completeness report
   */
  logBatchReport(report: ReturnType<typeof this.generateBatchReport>): void {
    this.logger.info('');
    this.logger.info('═══════════════════════════════════════════════════════');
    this.logger.info('  DATA COMPLETENESS REPORT');
    this.logger.info('═══════════════════════════════════════════════════════');
    this.logger.info('');
    this.logger.info(`Overall Score: ${report.overallScore}%`);
    this.logger.info(`Total Pages: ${report.totalPages}`);
    this.logger.info(`Complete Pages: ${report.completePages} (${Math.round(report.completePages / report.totalPages * 100)}%)`);
    this.logger.info(`Incomplete Pages: ${report.incompletePages} (${Math.round(report.incompletePages / report.totalPages * 100)}%)`);
    this.logger.info('');

    if (report.commonMissingFields.size > 0) {
      this.logger.info('Common Missing Fields:');
      const sortedFields = Array.from(report.commonMissingFields.entries())
        .sort((a, b) => b[1] - a[1])
        .slice(0, 10);

      sortedFields.forEach(([field, count]) => {
        const percentage = Math.round(count / report.totalPages * 100);
        this.logger.info(`  - ${field}: ${count} pages (${percentage}%)`);
      });
      this.logger.info('');
    }

    // Show recommendations for pages with lowest scores
    const lowScorePages = report.pageReports
      .filter(r => !r.isComplete)
      .sort((a, b) => a.score - b.score)
      .slice(0, 3);

    if (lowScorePages.length > 0) {
      this.logger.info('Pages Needing Attention:');
      lowScorePages.forEach(page => {
        this.logger.info(`  ${page.pageUrl} (${page.score}%)`);
        page.recommendations.slice(0, 2).forEach(rec => {
          this.logger.info(`    → ${rec}`);
        });
      });
      this.logger.info('');
    }

    this.logger.info('═══════════════════════════════════════════════════════');
    this.logger.info('');
  }

  /**
   * Check aggregations and log results
   */
  logAggregationChecks(checks: AggregationCheck[]): void {
    const incorrect = checks.filter(c => !c.correct);

    if (incorrect.length === 0) {
      this.logger.success('✓ All aggregations are correct');
    } else {
      this.logger.error(`✗ ${incorrect.length} aggregation errors found`);
      incorrect.forEach(check => {
        this.logger.error(`  ${check.field}: expected ${check.expected}, got ${check.actual}`);
      });
    }
  }
}
