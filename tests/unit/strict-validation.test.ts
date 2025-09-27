/**
 * 🧪 STRICT VALIDATION TESTS
 * 
 * Tests für die strikten Validatoren und Adapter, die sicherstellen,
 * dass vollständige und konsistente Datenstrukturen erzwungen werden.
 */

import {
  createStrictAuditData,
  createStrictAuditPage,
  createStrictAccessibility,
  validateStrictAuditData
} from '../../src/validators/strict-audit-validators';
import {
  AuditDataAdapter,
  convertAndValidateAuditData,
  safeConvertAuditData
} from '../../src/adapters/audit-data-adapter';
import {
  StrictAuditData,
  IncompleteAuditDataError,
  MissingAnalysisError,
  hasCompleteAnalysis
} from '../../src/types/strict-audit-types';
import { AuditResult } from '../../src/types';

describe('Strict Validation System', () => {
  
  // ============================================================================
  // MINIMAL VALID DATA STRUCTURES FOR TESTING
  // ============================================================================

  const createMinimalValidAccessibility = () => ({
    score: 85,
    errors: [
      {
        code: 'color-contrast',
        message: 'Colors must have sufficient contrast',
        type: 'error',
        selector: '.button',
        context: '<button class="button">Click me</button>',
        impact: 'serious',
        help: 'Ensure all colors meet WCAG contrast requirements',
        helpUrl: 'https://dequeuniversity.com/rules/axe/4.10/color-contrast'
      }
    ],
    warnings: [
      {
        code: 'landmark-one-main',
        message: 'Page should have one main landmark',
        type: 'warning',
        selector: 'body',
        context: '<body>...</body>',
        impact: 'moderate',
        help: 'Add a main landmark to identify the primary content',
        helpUrl: 'https://dequeuniversity.com/rules/axe/4.10/landmark-one-main'
      }
    ],
    notices: []
  });

  const createMinimalValidPerformance = () => ({
    score: 78,
    grade: 'B',
    coreWebVitals: {
      largestContentfulPaint: 2.5,
      firstContentfulPaint: 1.2,
      cumulativeLayoutShift: 0.15,
      timeToFirstByte: 800,
      domContentLoaded: 1500,
      loadComplete: 3000,
      firstPaint: 1100
    },
    issues: ['LCP exceeds recommended threshold', 'CLS needs improvement']
  });

  const createMinimalValidSEO = () => ({
    score: 92,
    grade: 'A',
    metaTags: {
      title: 'Test Page Title',
      titleLength: 15,
      description: 'This is a test page description',
      descriptionLength: 33,
      keywords: 'test, page, seo',
      h1Count: 1,
      h2Count: 3,
      h3Count: 2,
      totalImages: 5,
      imagesWithoutAlt: 1,
      imagesWithEmptyAlt: 0
    },
    issues: ['One image missing alt attribute'],
    recommendations: ['Add alt attributes to all images']
  });

  const createMinimalValidContentWeight = () => ({
    score: 65,
    grade: 'C',
    resources: {
      totalSize: 2500000,
      html: { size: 45000, files: 1 },
      css: { size: 120000, files: 3 },
      javascript: { size: 800000, files: 8 },
      images: { size: 1500000, files: 12 },
      other: { size: 35000, files: 2 }
    },
    optimizations: ['Compress images', 'Minify JavaScript', 'Enable gzip compression']
  });

  const createMinimalValidMobileFriendliness = () => ({
    overallScore: 88,
    grade: 'B',
    recommendations: [
      {
        category: 'Touch Targets',
        priority: 'medium',
        issue: 'Some buttons are too small for touch',
        recommendation: 'Increase button size to at least 44px',
        impact: 'Users may have difficulty tapping small buttons'
      },
      {
        category: 'Media',
        priority: 'low',
        issue: 'Images could use responsive sizing',
        recommendation: 'Implement responsive image techniques',
        impact: 'Improved loading performance on mobile devices'
      }
    ]
  });

  const createMinimalValidPage = () => ({
    url: 'https://example.com/test',
    title: 'Test Page',
    status: 'passed' as const,
    duration: 5000,
    testedAt: '2024-01-15T10:30:00Z',
    accessibility: createMinimalValidAccessibility(),
    performance: createMinimalValidPerformance(),
    seo: createMinimalValidSEO(),
    contentWeight: createMinimalValidContentWeight(),
    mobileFriendliness: createMinimalValidMobileFriendliness()
  });

  const createMinimalValidAuditData = () => ({
    metadata: {
      version: '1.0.0',
      timestamp: '2024-01-15T10:00:00Z',
      sitemapUrl: 'https://example.com/sitemap.xml',
      toolVersion: '2.0.0-alpha.2',
      duration: 30000,
      configuration: {
        maxPages: 5,
        timeout: 30000,
        standard: 'WCAG2AA',
        features: ['accessibility', 'performance', 'seo', 'contentWeight', 'mobileFriendliness']
      }
    },
    summary: {
      totalPages: 3,
      testedPages: 3,
      passedPages: 2,
      failedPages: 1,
      crashedPages: 0,
      redirectPages: 0,
      totalErrors: 5,
      totalWarnings: 8,
      averageScore: 82,
      overallGrade: 'B' as const
    },
    pages: [createMinimalValidPage()],
    systemPerformance: {
      testCompletionTimeSeconds: 30,
      averageTimePerPageMs: 10000,
      throughputPagesPerMinute: 6,
      memoryUsageMB: 250,
      efficiency: 95.5
    }
  });

  // ============================================================================
  // POSITIVE TESTS - VALID DATA
  // ============================================================================

  describe('Valid Data Processing', () => {
    it('should create strict accessibility data from valid input', () => {
      const validData = createMinimalValidAccessibility();
      const result = createStrictAccessibility(validData, 'https://example.com');

      expect(result.score).toBe(85);
      expect(result.errors).toHaveLength(1);
      expect(result.warnings).toHaveLength(1);
      expect(result.notices).toHaveLength(0);
      expect(result.totalIssues).toBe(2);
      expect(result.wcagLevel).toBe('AA');
    });

    it('should create strict audit page from valid input', () => {
      const validPage = createMinimalValidPage();
      const result = createStrictAuditPage(validPage);

      expect(result.url).toBe('https://example.com/test');
      expect(result.title).toBe('Test Page');
      expect(result.status).toBe('passed');
      expect(result.accessibility.score).toBe(85);
      expect(result.performance.grade).toBe('B');
      expect(result.seo.metaTags.title).toBe('Test Page Title');
      expect(hasCompleteAnalysis(result)).toBe(true);
    });

    it('should create strict audit data from valid input', () => {
      const validData = createMinimalValidAuditData();
      const result = createStrictAuditData(validData);

      expect(result.metadata.version).toBe('1.0.0');
      expect(result.summary.totalPages).toBe(3);
      expect(result.pages).toHaveLength(1);
      expect(result.systemPerformance.efficiency).toBe(100.0);

      // Validate all pages have complete analysis
      result.pages.forEach(page => {
        expect(hasCompleteAnalysis(page)).toBe(true);
      });
    });

    it('should validate strict audit data without throwing', () => {
      const validData = createMinimalValidAuditData();
      const strictData = createStrictAuditData(validData);

      expect(() => validateStrictAuditData(strictData)).not.toThrow();
    });
  });

  // ============================================================================
  // NEGATIVE TESTS - INVALID DATA
  // ============================================================================

  describe('Invalid Data Rejection', () => {
    it('should reject accessibility data with missing score', () => {
      const invalidData = { ...createMinimalValidAccessibility() };
      delete invalidData.score;

      expect(() => createStrictAccessibility(invalidData, 'test-url'))
        .toThrow(MissingAnalysisError);
    });

    it('should reject accessibility data with invalid score range', () => {
      const invalidData = { ...createMinimalValidAccessibility(), score: 150 };

      expect(() => createStrictAccessibility(invalidData, 'test-url'))
        .toThrow(MissingAnalysisError);
    });

    it('should reject accessibility data with non-array errors', () => {
      const invalidData = { ...createMinimalValidAccessibility(), errors: 'not an array' };

      expect(() => createStrictAccessibility(invalidData, 'test-url'))
        .toThrow(MissingAnalysisError);
    });

    it('should reject page data with missing URL', () => {
      const invalidPage = { ...createMinimalValidPage() };
      delete invalidPage.url;

      expect(() => createStrictAuditPage(invalidPage))
        .toThrow(IncompleteAuditDataError);
    });

    it('should reject page data with missing analysis types', () => {
      const invalidPage = { ...createMinimalValidPage() };
      delete invalidPage.accessibility;

      expect(() => createStrictAuditPage(invalidPage))
        .toThrow(IncompleteAuditDataError);
    });

    it('should reject audit data with empty pages array', () => {
      const invalidData = { ...createMinimalValidAuditData(), pages: [] };

      expect(() => createStrictAuditData(invalidData))
        .toThrow(IncompleteAuditDataError);
    });

    it('should reject audit data with missing metadata', () => {
      const invalidData = { ...createMinimalValidAuditData() };
      delete invalidData.metadata;

      expect(() => createStrictAuditData(invalidData))
        .toThrow(IncompleteAuditDataError);
    });
  });

  // ============================================================================
  // ADAPTER TESTS - LEGACY TO STRICT CONVERSION
  // ============================================================================

  describe('Legacy Data Adapter', () => {
    const createLegacyAuditResult = (): AuditResult => ({
      metadata: {
        version: '1.0.0',
        timestamp: '2024-01-15T10:00:00Z',
        sitemapUrl: 'https://example.com/sitemap.xml',
        toolVersion: '2.0.0-alpha.2',
        duration: 25000,
        maxPages: 5,
        timeout: 30000,
        standard: 'WCAG2AA',
        features: ['accessibility', 'performance']
      },
      summary: {
        totalPages: 2,
        testedPages: 2,
        passedPages: 1,
        failedPages: 1,
        totalErrors: 3,
        totalWarnings: 5,
        averageScore: 75,
        overallGrade: 'B'
      },
      pages: [
        {
          url: 'https://example.com/page1',
          title: 'Legacy Test Page',
          status: 'passed',
          duration: 4000,
          accessibility: {
            score: 80,
            errors: ['Color contrast too low'],
            warnings: ['Missing alt text'],
            notices: []
          },
          performance: {
            score: 70,
            grade: 'B',
            coreWebVitals: {
              largestContentfulPaint: 3.0,
              firstContentfulPaint: 1.5,
              cumulativeLayoutShift: 0.2
            },
            issues: ['LCP needs improvement']
          },
          seo: {
            score: 85,
            metaTags: {
              title: 'Legacy Test Page',
              titleLength: 17,
              description: 'Legacy page description'
            },
            issues: ['Missing meta keywords']
          }
        }
      ]
    } as AuditResult);

    it('should diagnose legacy data completeness', () => {
      const legacyData = createLegacyAuditResult();
      const diagnosis = AuditDataAdapter.diagnoseLegacyData(legacyData);

      expect(diagnosis.isComplete).toBe(false);
      expect(diagnosis.pageAnalysis).toHaveLength(1);
      expect(diagnosis.pageAnalysis[0].missingAnalyses).toContain('contentWeight');
      expect(diagnosis.pageAnalysis[0].missingAnalyses).toContain('mobileFriendliness');
    });

    it('should convert legacy data to strict format', () => {
      const legacyData = createLegacyAuditResult();
      const strictData = AuditDataAdapter.convertToStrict(legacyData);

      expect(strictData.metadata.version).toBe('1.0.0');
      expect(strictData.pages).toHaveLength(1);
      expect(strictData.pages[0].accessibility.errors).toHaveLength(1);
      expect(strictData.pages[0].contentWeight).toBeDefined();
      expect(strictData.pages[0].mobileFriendliness).toBeDefined();
    });

    it('should handle missing analysis data gracefully', () => {
      const legacyData: AuditResult = {
        metadata: { version: '1.0.0', timestamp: '2024-01-15T10:00:00Z' } as any,
        summary: { totalPages: 1, testedPages: 1 } as any,
        pages: [
          {
            url: 'https://example.com/incomplete',
            title: 'Incomplete Page',
            status: 'crashed'
            // Missing all analysis data
          } as any
        ]
      } as AuditResult;

      const strictData = AuditDataAdapter.convertToStrict(legacyData);

      expect(strictData.pages[0].accessibility.score).toBe(0);
      expect(strictData.pages[0].performance.grade).toBe('F');
      expect(strictData.pages[0].seo.score).toBe(0);
      expect(strictData.pages[0].contentWeight.resources.totalSize).toBe(0);
      expect(strictData.pages[0].mobileFriendliness.overallScore).toBe(0);
    });

    it('should use safe conversion with error handling', () => {
      const validLegacyData = createLegacyAuditResult();
      const validResult = safeConvertAuditData(validLegacyData);

      expect(validResult.success).toBe(true);
      expect(validResult.data).toBeDefined();
      expect(validResult.error).toBeUndefined();

      // Test with invalid data that should cause conversion failure
      const invalidLegacyData = null as any;
      const invalidResult = safeConvertAuditData(invalidLegacyData);

      expect(invalidResult.success).toBe(false);
      expect(invalidResult.data).toBeUndefined();
      expect(invalidResult.error).toBeDefined();
    });

    it('should convert and validate in one step', () => {
      const legacyData = createLegacyAuditResult();

      expect(() => convertAndValidateAuditData(legacyData)).not.toThrow();

      const result = convertAndValidateAuditData(legacyData);
      expect(result.pages).toHaveLength(1);
      expect(hasCompleteAnalysis(result.pages[0])).toBe(true);
    });
  });

  // ============================================================================
  // EDGE CASE TESTS
  // ============================================================================

  describe('Edge Cases', () => {
    it('should handle string-based accessibility issues', () => {
      const dataWithStringIssues = {
        score: 60,
        errors: ['This is a string error'],
        warnings: ['This is a string warning'],
        notices: ['This is a string notice']
      };

      const result = createStrictAccessibility(dataWithStringIssues, 'test-url');

      expect(result.errors[0].message).toBe('This is a string error');
      expect(result.errors[0].code).toBe('legacy-string-issue');
      expect(result.errors[0].type).toBe('error');
    });

    it('should normalize page status values', () => {
      const testCases = [
        { input: 'success', expected: 'crashed' },
        { input: 'passed', expected: 'passed' },
        { input: 'error', expected: 'crashed' },
        { input: 'failed', expected: 'failed' },
        { input: 'crashed', expected: 'crashed' }
      ];

      testCases.forEach(({ input, expected }) => {
        // Use the adapter's conversion method which handles normalization
        const legacyPageData = {
          url: 'https://example.com/test',
          title: 'Test Page',
          status: input,
          duration: 5000,
          accessibility: { score: 85, errors: [], warnings: [], notices: [] },
          performance: { score: 78, grade: 'B', coreWebVitals: {}, issues: [] },
          seo: { score: 92, grade: 'A', metaTags: {}, issues: [] },
          contentWeight: { score: 65, grade: 'C', resources: {}, optimizations: [] },
          mobileFriendliness: { overallScore: 88, grade: 'B', recommendations: [] }
        };
        
        const auditData = { pages: [legacyPageData] };
        const result = AuditDataAdapter.convertToStrict(auditData);
        expect(result.pages[0].status).toBe(expected);
      });
    });

    it('should calculate WCAG levels correctly', () => {
      const testCases = [
        { score: 96, expectedLevel: 'AAA' },
        { score: 85, expectedLevel: 'AA' },
        { score: 65, expectedLevel: 'A' },
        { score: 45, expectedLevel: 'none' }
      ];

      testCases.forEach(({ score, expectedLevel }) => {
        const data = { ...createMinimalValidAccessibility(), score };
        const result = createStrictAccessibility(data, 'test-url');
        expect(result.wcagLevel).toBe(expectedLevel);
      });
    });

    it('should handle mixed object and string accessibility issues', () => {
      const mixedIssues = {
        score: 70,
        errors: [
          'String error',
          {
            code: 'color-contrast',
            message: 'Object error',
            type: 'error',
            selector: '.test',
            impact: 'serious'
          }
        ],
        warnings: [],
        notices: []
      };

      const result = createStrictAccessibility(mixedIssues, 'test-url');

      expect(result.errors).toHaveLength(2);
      expect(result.errors[0].code).toBe('legacy-string-issue');
      expect(result.errors[1].code).toBe('color-contrast');
      expect(result.totalIssues).toBe(2);
    });
  });

  // ============================================================================
  // PERFORMANCE TESTS
  // ============================================================================

  describe('Performance and Scalability', () => {
    it('should handle large datasets efficiently', () => {
      const largeAuditData = createMinimalValidAuditData();
      
      // Create 50 pages to test scalability
      largeAuditData.pages = Array.from({ length: 50 }, (_, i) => ({
        ...createMinimalValidPage(),
        url: `https://example.com/page-${i + 1}`,
        title: `Test Page ${i + 1}`
      }));

      largeAuditData.summary.testedPages = 50;
      largeAuditData.summary.totalPages = 50;

      const startTime = Date.now();
      const result = createStrictAuditData(largeAuditData);
      const processingTime = Date.now() - startTime;

      expect(result.pages).toHaveLength(50);
      expect(processingTime).toBeLessThan(1000); // Should complete within 1 second
      expect(result.pages.every(page => hasCompleteAnalysis(page))).toBe(true);
    });

    it('should validate large datasets without memory issues', () => {
      const largeValidData = createMinimalValidAuditData();
      largeValidData.pages = Array.from({ length: 100 }, (_, i) => 
        createMinimalValidPage()
      );

      const strictData = createStrictAuditData(largeValidData);

      expect(() => validateStrictAuditData(strictData)).not.toThrow();
      expect(strictData.pages).toHaveLength(100);
    });
  });
});