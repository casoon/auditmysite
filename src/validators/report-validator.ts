/**
 * Report Data Validator
 * Ensures audit results are complete and properly aggregated
 */

import { AccessibilityResult, TestSummary } from '../types';
import { Logger } from '../core/logging/logger';

export interface ValidationResult {
  valid: boolean;
  errors: ValidationError[];
  warnings: ValidationWarning[];
  stats: ValidationStats;
}

export interface ValidationError {
  field: string;
  message: string;
  severity: 'critical' | 'error';
  context?: Record<string, unknown>;
}

export interface ValidationWarning {
  field: string;
  message: string;
  suggestion?: string;
}

export interface ValidationStats {
  totalPages: number;
  validPages: number;
  pagesWithErrors: number;
  pagesWithWarnings: number;
  missingFields: string[];
  completenessScore: number; // 0-100
}

export class ReportValidator {
  private logger: Logger;

  constructor() {
    this.logger = new Logger({ level: 'info' });
  }

  /**
   * Validate complete audit results
   */
  validateAuditResults(results: AccessibilityResult[]): ValidationResult {
    const errors: ValidationError[] = [];
    const warnings: ValidationWarning[] = [];
    let validPages = 0;
    let pagesWithErrors = 0;
    let pagesWithWarnings = 0;
    const missingFieldsSet = new Set<string>();

    // Validate each page result
    results.forEach((result, index) => {
      const pageValidation = this.validatePageResult(result, index);

      if (pageValidation.errors.length > 0) {
        errors.push(...pageValidation.errors);
        pagesWithErrors++;
      } else {
        validPages++;
      }

      if (pageValidation.warnings.length > 0) {
        warnings.push(...pageValidation.warnings);
        pagesWithWarnings++;
      }

      pageValidation.missingFields.forEach(field => missingFieldsSet.add(field));
    });

    const completenessScore = this.calculateCompletenessScore(results);

    const stats: ValidationStats = {
      totalPages: results.length,
      validPages,
      pagesWithErrors,
      pagesWithWarnings,
      missingFields: Array.from(missingFieldsSet),
      completenessScore
    };

    return {
      valid: errors.length === 0,
      errors,
      warnings,
      stats
    };
  }

  /**
   * Validate individual page result
   */
  private validatePageResult(result: AccessibilityResult, index: number): {
    errors: ValidationError[];
    warnings: ValidationWarning[];
    missingFields: string[];
  } {
    const errors: ValidationError[] = [];
    const warnings: ValidationWarning[] = [];
    const missingFields: string[] = [];

    // Required fields validation
    if (!result.url) {
      errors.push({
        field: `results[${index}].url`,
        message: 'URL is required',
        severity: 'critical',
        context: { index }
      });
    }

    if (!result.title) {
      warnings.push({
        field: `results[${index}].title`,
        message: 'Page title is missing',
        suggestion: 'Page might not have loaded correctly'
      });
      missingFields.push('title');
    }

    if (result.duration === undefined || result.duration < 0) {
      errors.push({
        field: `results[${index}].duration`,
        message: 'Invalid or missing duration',
        severity: 'error',
        context: { index, duration: result.duration }
      });
    }

    // Accessibility data validation
    if (!result.errors || !Array.isArray(result.errors)) {
      errors.push({
        field: `results[${index}].errors`,
        message: 'Errors array is missing or invalid',
        severity: 'critical'
      });
    }

    if (!result.warnings || !Array.isArray(result.warnings)) {
      errors.push({
        field: `results[${index}].warnings`,
        message: 'Warnings array is missing or invalid',
        severity: 'critical'
      });
    }

    // Pa11y data validation
    if (result.pa11yIssues && !Array.isArray(result.pa11yIssues)) {
      errors.push({
        field: `results[${index}].pa11yIssues`,
        message: 'pa11yIssues must be an array',
        severity: 'error'
      });
    }

    if (result.pa11yScore !== undefined) {
      if (typeof result.pa11yScore !== 'number' || result.pa11yScore < 0 || result.pa11yScore > 100) {
        errors.push({
          field: `results[${index}].pa11yScore`,
          message: 'pa11yScore must be a number between 0 and 100',
          severity: 'error',
          context: { score: result.pa11yScore }
        });
      }
    } else {
      missingFields.push('pa11yScore');
    }

    // Performance metrics validation
    if (result.performanceMetrics) {
      const perfErrors = this.validatePerformanceMetrics(result.performanceMetrics, index);
      errors.push(...perfErrors);
    } else {
      warnings.push({
        field: `results[${index}].performanceMetrics`,
        message: 'Performance metrics are missing',
        suggestion: 'Enable collectPerformanceMetrics option'
      });
      missingFields.push('performanceMetrics');
    }

    // Status validation
    if (result.crashed && result.passed) {
      errors.push({
        field: `results[${index}].status`,
        message: 'Page cannot be both crashed and passed',
        severity: 'error',
        context: { crashed: result.crashed, passed: result.passed }
      });
    }

    return { errors, warnings, missingFields };
  }

  /**
   * Validate performance metrics
   */
  private validatePerformanceMetrics(metrics: any, pageIndex: number): ValidationError[] {
    const errors: ValidationError[] = [];

    const requiredMetrics = [
      'loadTime',
      'domContentLoaded',
      'firstPaint',
      'renderTime',
      'firstContentfulPaint',
      'largestContentfulPaint'
    ];

    requiredMetrics.forEach(metric => {
      if (metrics[metric] === undefined) {
        errors.push({
          field: `results[${pageIndex}].performanceMetrics.${metric}`,
          message: `Performance metric '${metric}' is missing`,
          severity: 'error'
        });
      } else if (typeof metrics[metric] !== 'number' || metrics[metric] < 0) {
        errors.push({
          field: `results[${pageIndex}].performanceMetrics.${metric}`,
          message: `Performance metric '${metric}' must be a positive number`,
          severity: 'error',
          context: { value: metrics[metric] }
        });
      }
    });

    // Validate performance score
    if (metrics.performanceScore !== undefined) {
      if (typeof metrics.performanceScore !== 'number' ||
          metrics.performanceScore < 0 ||
          metrics.performanceScore > 100) {
        errors.push({
          field: `results[${pageIndex}].performanceMetrics.performanceScore`,
          message: 'performanceScore must be between 0 and 100',
          severity: 'error',
          context: { score: metrics.performanceScore }
        });
      }
    }

    // Validate performance grade
    if (metrics.performanceGrade !== undefined) {
      const validGrades = ['A', 'B', 'C', 'D', 'F'];
      if (!validGrades.includes(metrics.performanceGrade)) {
        errors.push({
          field: `results[${pageIndex}].performanceMetrics.performanceGrade`,
          message: 'performanceGrade must be A, B, C, D, or F',
          severity: 'error',
          context: { grade: metrics.performanceGrade }
        });
      }
    }

    return errors;
  }

  /**
   * Validate test summary
   */
  validateTestSummary(summary: TestSummary): ValidationResult {
    const errors: ValidationError[] = [];
    const warnings: ValidationWarning[] = [];

    // Check totals consistency
    const expectedTotal = summary.passedPages + summary.failedPages + (summary.crashedPages || 0);
    if (summary.testedPages !== expectedTotal) {
      errors.push({
        field: 'summary.testedPages',
        message: 'testedPages does not match sum of passed + failed + crashed',
        severity: 'critical',
        context: {
          testedPages: summary.testedPages,
          calculated: expectedTotal,
          passed: summary.passedPages,
          failed: summary.failedPages,
          crashed: summary.crashedPages
        }
      });
    }

    // Validate all pages have results
    if (summary.results.length !== summary.testedPages) {
      errors.push({
        field: 'summary.results',
        message: 'Number of results does not match testedPages count',
        severity: 'critical',
        context: {
          resultsCount: summary.results.length,
          testedPages: summary.testedPages
        }
      });
    }

    // Validate error/warning counts
    const actualErrors = summary.results.reduce((sum, r) => sum + (r.errors?.length || 0), 0);
    const actualWarnings = summary.results.reduce((sum, r) => sum + (r.warnings?.length || 0), 0);

    if (summary.totalErrors !== actualErrors) {
      warnings.push({
        field: 'summary.totalErrors',
        message: 'totalErrors count does not match aggregated errors from results',
        suggestion: `Expected ${actualErrors}, got ${summary.totalErrors}`
      });
    }

    if (summary.totalWarnings !== actualWarnings) {
      warnings.push({
        field: 'summary.totalWarnings',
        message: 'totalWarnings count does not match aggregated warnings from results',
        suggestion: `Expected ${actualWarnings}, got ${summary.totalWarnings}`
      });
    }

    const stats: ValidationStats = {
      totalPages: summary.testedPages,
      validPages: summary.passedPages,
      pagesWithErrors: summary.failedPages,
      pagesWithWarnings: 0,
      missingFields: [],
      completenessScore: this.calculateCompletenessScore(summary.results)
    };

    return {
      valid: errors.length === 0,
      errors,
      warnings,
      stats
    };
  }

  /**
   * Calculate completeness score (0-100)
   */
  private calculateCompletenessScore(results: AccessibilityResult[]): number {
    if (results.length === 0) return 0;

    const requiredFields = [
      'url',
      'title',
      'duration',
      'errors',
      'warnings',
      'passed',
      'pa11yScore',
      'performanceMetrics'
    ];

    let totalFields = 0;
    let presentFields = 0;

    results.forEach(result => {
      requiredFields.forEach(field => {
        totalFields++;
        if (result[field as keyof AccessibilityResult] !== undefined) {
          presentFields++;
        }
      });
    });

    return Math.round((presentFields / totalFields) * 100);
  }

  /**
   * Generate validation report
   */
  generateReport(validation: ValidationResult): string {
    const lines: string[] = [];

    lines.push('═══════════════════════════════════════════════════════');
    lines.push('  AUDIT DATA VALIDATION REPORT');
    lines.push('═══════════════════════════════════════════════════════');
    lines.push('');

    // Overall status
    lines.push(`Status: ${validation.valid ? '✅ VALID' : '❌ INVALID'}`);
    lines.push(`Completeness Score: ${validation.stats.completenessScore}%`);
    lines.push('');

    // Statistics
    lines.push('Statistics:');
    lines.push(`  Total Pages: ${validation.stats.totalPages}`);
    lines.push(`  Valid Pages: ${validation.stats.validPages}`);
    lines.push(`  Pages with Errors: ${validation.stats.pagesWithErrors}`);
    lines.push(`  Pages with Warnings: ${validation.stats.pagesWithWarnings}`);
    lines.push('');

    // Errors
    if (validation.errors.length > 0) {
      lines.push(`Errors (${validation.errors.length}):`);
      validation.errors.forEach((error, i) => {
        lines.push(`  ${i + 1}. [${error.severity.toUpperCase()}] ${error.field}`);
        lines.push(`     ${error.message}`);
        if (error.context) {
          lines.push(`     Context: ${JSON.stringify(error.context)}`);
        }
      });
      lines.push('');
    }

    // Warnings
    if (validation.warnings.length > 0) {
      lines.push(`Warnings (${validation.warnings.length}):`);
      validation.warnings.slice(0, 10).forEach((warning, i) => {
        lines.push(`  ${i + 1}. ${warning.field}: ${warning.message}`);
        if (warning.suggestion) {
          lines.push(`     Suggestion: ${warning.suggestion}`);
        }
      });
      if (validation.warnings.length > 10) {
        lines.push(`  ... and ${validation.warnings.length - 10} more warnings`);
      }
      lines.push('');
    }

    // Missing fields
    if (validation.stats.missingFields.length > 0) {
      lines.push('Commonly Missing Fields:');
      validation.stats.missingFields.forEach(field => {
        lines.push(`  - ${field}`);
      });
      lines.push('');
    }

    lines.push('═══════════════════════════════════════════════════════');

    return lines.join('\n');
  }

  /**
   * Log validation results
   */
  logValidation(validation: ValidationResult): void {
    if (validation.valid) {
      this.logger.success(`Audit data validation passed (${validation.stats.completenessScore}% complete)`);
    } else {
      this.logger.error(`Audit data validation failed with ${validation.errors.length} errors`);
    }

    if (validation.warnings.length > 0) {
      this.logger.warn(`Found ${validation.warnings.length} warnings`);
    }

    // Log detailed report in debug mode
    const report = this.generateReport(validation);
    this.logger.debug(report);
  }
}
