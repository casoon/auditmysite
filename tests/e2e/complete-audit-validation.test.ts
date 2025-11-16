/**
 * E2E Test: Complete Audit Validation
 * Ensures that full audits produce complete and valid data
 */

import { AccessibilityChecker } from '../../src/core/accessibility/accessibility-checker';
import { ReportValidator } from '../../src/validators/report-validator';
import { DataCompletenessChecker } from '../../src/validators/data-completeness-checker';
import { TestOptions } from '../../src/types';

describe('Complete Audit Data Validation', () => {
  let checker: AccessibilityChecker;
  let reportValidator: ReportValidator;
  let completenessChecker: DataCompletenessChecker;

  beforeEach(() => {
    checker = new AccessibilityChecker({
      maxConcurrent: 2,
      headless: true,
      verbose: false
    });

    reportValidator = new ReportValidator();
    completenessChecker = new DataCompletenessChecker();
  });

  afterEach(async () => {
    await checker.cleanup();
  });

  describe('Single Page Audit', () => {
    it('should produce complete data for a single page', async () => {
      const options: TestOptions = {
        maxPages: 1,
        timeout: 30000,
        collectPerformanceMetrics: true,
        usePa11y: true
      };

      const summary = await checker.testUrls(
        ['https://example.com'],
        options
      );

      // Validate results exist
      expect(summary.results).toBeDefined();
      expect(summary.results.length).toBe(1);

      const result = summary.results[0];

      // Check completeness
      const completeness = completenessChecker.checkPageCompleteness(result);
      expect(completeness.isComplete).toBe(true);
      expect(completeness.score).toBeGreaterThanOrEqual(80);

      // Validate result structure
      const validation = reportValidator.validateAuditResults([result]);
      expect(validation.valid).toBe(true);
      expect(validation.errors.length).toBe(0);

      // Log results for manual verification
      completenessChecker.logCompletenessCheck(completeness);
      reportValidator.logValidation(validation);
    }, 60000);

    it('should have all critical fields populated', async () => {
      const options: TestOptions = {
        maxPages: 1,
        collectPerformanceMetrics: true
      };

      const summary = await checker.testUrls(
        ['https://example.com'],
        options
      );

      const result = summary.results[0];

      // Critical fields
      expect(result.url).toBeDefined();
      expect(result.title).toBeDefined();
      expect(result.duration).toBeGreaterThan(0);
      expect(Array.isArray(result.errors)).toBe(true);
      expect(Array.isArray(result.warnings)).toBe(true);
      expect(typeof result.passed).toBe('boolean');

      // Performance metrics
      expect(result.performanceMetrics).toBeDefined();
      expect(result.performanceMetrics?.loadTime).toBeGreaterThan(0);
      expect(result.performanceMetrics?.firstContentfulPaint).toBeGreaterThan(0);
      expect(result.performanceMetrics?.largestContentfulPaint).toBeGreaterThan(0);
    }, 60000);
  });

  describe('Multiple Pages Audit', () => {
    it('should produce complete data for multiple pages', async () => {
      const urls = [
        'https://example.com',
        'https://example.com/about',
        'https://example.com/contact'
      ];

      const options: TestOptions = {
        maxPages: 3,
        timeout: 30000,
        collectPerformanceMetrics: true,
        usePa11y: true
      };

      const summary = await checker.testUrls(urls, options);

      // Validate all results
      expect(summary.results.length).toBe(urls.length);

      // Check batch completeness
      const batchReport = completenessChecker.generateBatchReport(summary.results);
      expect(batchReport.overallScore).toBeGreaterThanOrEqual(80);
      expect(batchReport.completePages).toBeGreaterThan(0);

      // Validate all results
      const validation = reportValidator.validateAuditResults(summary.results);
      expect(validation.valid).toBe(true);
      expect(validation.stats.completenessScore).toBeGreaterThanOrEqual(80);

      // Log reports
      completenessChecker.logBatchReport(batchReport);
      reportValidator.logValidation(validation);
    }, 180000);

    it('should have correct aggregations in summary', async () => {
      const urls = [
        'https://example.com',
        'https://example.com/about'
      ];

      const options: TestOptions = {
        maxPages: 2,
        collectPerformanceMetrics: true
      };

      const summary = await checker.testUrls(urls, options);

      // Validate summary
      const validation = reportValidator.validateTestSummary(summary);
      expect(validation.valid).toBe(true);

      // Check aggregations
      const aggregationChecks = completenessChecker.verifyAggregations(summary.results);
      const incorrectAggregations = aggregationChecks.filter(c => !c.correct);
      expect(incorrectAggregations.length).toBe(0);

      completenessChecker.logAggregationChecks(aggregationChecks);
    }, 120000);
  });

  describe('Summary Validation', () => {
    it('should have consistent totals', async () => {
      const options: TestOptions = {
        maxPages: 2,
        collectPerformanceMetrics: true
      };

      const summary = await checker.testUrls(
        ['https://example.com', 'https://example.com/about'],
        options
      );

      // Validate summary consistency
      expect(summary.testedPages).toBe(summary.results.length);
      expect(summary.testedPages).toBe(
        summary.passedPages + summary.failedPages + (summary.crashedPages || 0)
      );

      // Validate error/warning counts
      const actualErrors = summary.results.reduce(
        (sum, r) => sum + r.errors.length,
        0
      );
      const actualWarnings = summary.results.reduce(
        (sum, r) => sum + r.warnings.length,
        0
      );

      expect(summary.totalErrors).toBe(actualErrors);
      expect(summary.totalWarnings).toBe(actualWarnings);

      // Validate duration
      const actualDuration = summary.results.reduce(
        (sum, r) => sum + r.duration,
        0
      );
      expect(summary.totalDuration).toBeGreaterThanOrEqual(actualDuration * 0.9);
      expect(summary.totalDuration).toBeLessThanOrEqual(actualDuration * 1.1);
    }, 120000);
  });

  describe('Data Quality Checks', () => {
    it('should not have any pages with zero duration', async () => {
      const summary = await checker.testUrls(
        ['https://example.com'],
        { maxPages: 1, collectPerformanceMetrics: true }
      );

      summary.results.forEach(result => {
        expect(result.duration).toBeGreaterThan(0);
      });
    }, 60000);

    it('should have valid performance scores', async () => {
      const summary = await checker.testUrls(
        ['https://example.com'],
        { maxPages: 1, collectPerformanceMetrics: true }
      );

      summary.results.forEach(result => {
        if (result.performanceMetrics?.performanceScore !== undefined) {
          expect(result.performanceMetrics.performanceScore).toBeGreaterThanOrEqual(0);
          expect(result.performanceMetrics.performanceScore).toBeLessThanOrEqual(100);
        }

        if (result.performanceMetrics?.performanceGrade) {
          expect(['A', 'B', 'C', 'D', 'F']).toContain(
            result.performanceMetrics.performanceGrade
          );
        }
      });
    }, 60000);

    it('should have valid pa11y scores when enabled', async () => {
      const summary = await checker.testUrls(
        ['https://example.com'],
        { maxPages: 1, usePa11y: true }
      );

      summary.results.forEach(result => {
        if (result.pa11yScore !== undefined) {
          expect(result.pa11yScore).toBeGreaterThanOrEqual(0);
          expect(result.pa11yScore).toBeLessThanOrEqual(100);
        }

        if (result.pa11yIssues) {
          expect(Array.isArray(result.pa11yIssues)).toBe(true);
          result.pa11yIssues.forEach(issue => {
            expect(issue.code).toBeDefined();
            expect(issue.message).toBeDefined();
            expect(['error', 'warning', 'notice']).toContain(issue.type);
          });
        }
      });
    }, 60000);
  });

  describe('Error Handling', () => {
    it('should handle invalid URLs gracefully', async () => {
      const summary = await checker.testUrls(
        ['https://this-domain-definitely-does-not-exist-12345.com'],
        { maxPages: 1, timeout: 10000 }
      );

      expect(summary.results.length).toBe(1);

      const result = summary.results[0];
      expect(result.crashed || !result.passed).toBe(true);

      // Should still have basic structure
      expect(result.url).toBeDefined();
      expect(result.duration).toBeGreaterThanOrEqual(0);
    }, 30000);

    it('should mark crashed pages correctly', async () => {
      const summary = await checker.testUrls(
        ['https://this-domain-definitely-does-not-exist-12345.com'],
        { maxPages: 1, timeout: 10000 }
      );

      const validation = reportValidator.validateTestSummary(summary);

      // Should have valid structure even with crashes
      expect(validation.valid).toBe(true);

      if (summary.crashedPages && summary.crashedPages > 0) {
        expect(summary.results.some(r => r.crashed)).toBe(true);
      }
    }, 30000);
  });
});
