/**
 * 🧪 E2E TESTS: SEO Context Isolation
 * 
 * Tests that validate SEO analysis runs without fallbacks and that
 * page contexts are properly isolated to prevent interference.
 */

import { describe, it, expect, beforeAll, afterAll, beforeEach, jest } from '@jest/globals';
import { AccessibilityChecker } from '../../src/core/accessibility/accessibility-checker';
import { TestOptions } from '../../src/types';

describe('SEO Context Isolation E2E Tests', () => {
  let checker: AccessibilityChecker;
  let mockConsole: {
    warn: jest.MockedFunction<any>;
    log: jest.MockedFunction<any>;
    error: jest.MockedFunction<any>;
  };

  beforeAll(async () => {
    // Mock console methods to capture output
    mockConsole = {
      warn: jest.fn(),
      log: jest.fn(),
      error: jest.fn()
    };
    
    global.console.warn = mockConsole.warn;
    global.console.log = mockConsole.log;
    global.console.error = mockConsole.error;
  });

  beforeEach(async () => {
    // Create checker with comprehensive analysis enabled
    checker = new AccessibilityChecker({
      usePooling: true,
      enableComprehensiveAnalysis: true,
      qualityAnalysisOptions: {
        includeResourceAnalysis: true,
        includeTechnicalSEO: true,
        analysisTimeout: 15000
      }
    });

    await checker.initialize();
    
    // Clear mock calls before each test
    mockConsole.warn.mockClear();
    mockConsole.log.mockClear();
    mockConsole.error.mockClear();
  });

  afterEach(async () => {
    if (checker) {
      await checker.cleanup();
    }
  });

  describe('SEO Analysis without Fallbacks', () => {
    it('should run SEO analysis without triggering fallback mechanisms', async () => {
      const testUrl = 'https://example.com';
      const options: TestOptions = {
        timeout: 15000,
        verbose: false
      };

      // Run single page test
      const result = await checker.testPage(testUrl, options);

      // Check that SEO analysis was successful
      expect(result).toBeDefined();
      expect(result.url).toBe(testUrl);

      // Verify no fallback messages in console output
      const allOutput = [
        ...mockConsole.warn.mock.calls.flat(),
        ...mockConsole.log.mock.calls.flat(),
        ...mockConsole.error.mock.calls.flat()
      ].join(' ');

      // Should not contain any fallback indicators
      expect(allOutput).not.toMatch(/FALLBACK.*SEO/i);
      expect(allOutput).not.toMatch(/page evaluation failed/i);
      expect(allOutput).not.toMatch(/using minimal data/i);
      expect(allOutput).not.toMatch(/page context unavailable/i);
      expect(allOutput).not.toMatch(/page context destroyed/i);

      // Should contain success indicators
      expect(allOutput).toMatch(/SEO analysis.*completed successfully/i);
    }, 30000);

    it('should handle multiple concurrent SEO analyses without interference', async () => {
      const testUrls = [
        'https://example.com',
        'https://httpbin.org/html',
        'https://httpbin.org/json'
      ];

      const options: TestOptions = {
        timeout: 15000,
        verbose: false,
        maxConcurrent: 2
      };

      // Run parallel tests
      const results = await checker.testMultiplePagesParallel(testUrls, options);

      // Verify all tests completed
      expect(results).toHaveLength(3);
      expect(results.every(r => r.url)).toBe(true);

      // Check for no fallback messages across all tests
      const allOutput = [
        ...mockConsole.warn.mock.calls.flat(),
        ...mockConsole.log.mock.calls.flat(),
        ...mockConsole.error.mock.calls.flat()
      ].join(' ');

      expect(allOutput).not.toMatch(/FALLBACK.*SEO/i);
      expect(allOutput).not.toMatch(/page evaluation failed/i);
      expect(allOutput).not.toMatch(/page context.*destroyed/i);
    }, 45000);
  });

  describe('Browser Context Isolation', () => {
    it('should create isolated page contexts for SEO analysis', async () => {
      const testUrl = 'https://example.com';
      const options: TestOptions = {
        timeout: 15000,
        verbose: true // Enable verbose to see isolation messages
      };

      const result = await checker.testPage(testUrl, options);

      // Check verbose output for isolation indicators
      const verboseOutput = mockConsole.log.mock.calls.flat().join(' ');
      
      expect(verboseOutput).toMatch(/SEO analysis with isolated context/i);
      expect(verboseOutput).toMatch(/Loading page in isolated SEO context/i);
      expect(verboseOutput).toMatch(/SEO analysis completed successfully with isolated context/i);

      expect(result).toBeDefined();
      expect(result.passed).toBeDefined();
    }, 30000);

    it('should properly clean up isolated contexts after analysis', async () => {
      const testUrl = 'https://example.com';
      const options: TestOptions = {
        timeout: 15000,
        verbose: true
      };

      await checker.testPage(testUrl, options);

      // Check that no warnings about failed context cleanup appeared
      const warningOutput = mockConsole.warn.mock.calls.flat().join(' ');
      const errorOutput = mockConsole.error.mock.calls.flat().join(' ');
      
      expect(warningOutput).not.toMatch(/Failed to close isolated SEO page/i);
      expect(errorOutput).not.toMatch(/Failed to close isolated SEO page/i);
    }, 30000);
  });

  describe('CI/CD Deprecation Warning Suppression', () => {
    beforeEach(() => {
      // Reset environment variables
      delete process.env.CI;
    });

    afterEach(() => {
      // Clean up environment variables
      delete process.env.CI;
    });

    it('should show deprecation warnings by default', async () => {
      const testUrls = ['https://example.com'];
      const options: TestOptions = {
        timeout: 15000,
        verbose: false,
        eventCallbacks: {
          onUrlStarted: (url: string) => { /* test callback */ }
        }
      };

      await checker.testMultiplePagesParallel(testUrls, options);

      // Should show deprecation warnings
      const warningOutput = mockConsole.warn.mock.calls.flat().join(' ');
      expect(warningOutput).toMatch(/DEPRECATION WARNING.*TestOptions\.eventCallbacks/);
    }, 30000);

    it('should suppress deprecation warnings when CI environment is detected', async () => {
      // Set CI environment variable to suppress warnings
      process.env.CI = 'true';

      const testUrls = ['https://example.com'];
      const options: TestOptions = {
        timeout: 15000,
        verbose: false,
        eventCallbacks: {
          onUrlStarted: (url: string) => { /* test callback */ }
        }
      };

      await checker.testMultiplePagesParallel(testUrls, options);

      // Should NOT show deprecation warnings
      const warningOutput = mockConsole.warn.mock.calls.flat().join(' ');
      expect(warningOutput).not.toMatch(/DEPRECATION WARNING.*TestOptions\.eventCallbacks/);
    }, 30000);
  });

  describe('Enhanced SEO Data Collection', () => {
    it('should collect comprehensive SEO data without fallbacks', async () => {
      const testUrl = 'https://example.com';
      const options: TestOptions = {
        timeout: 15000,
        verbose: false
      };

      const result = await checker.testPage(testUrl, options);

      // Check that enhanced SEO data is present
      const enhancedResult = result as any;
      
      if (enhancedResult.enhancedSEO) {
        expect(enhancedResult.enhancedSEO).toBeDefined();
        expect(enhancedResult.enhancedSEO.seoScore).toBeGreaterThanOrEqual(0);
        expect(enhancedResult.enhancedSEO.seoScore).toBeLessThanOrEqual(100);
        expect(enhancedResult.enhancedSEO.metaData).toBeDefined();
        expect(enhancedResult.enhancedSEO.technicalSEO).toBeDefined();
      }

      // Verify no fallback data was used
      const allOutput = [
        ...mockConsole.warn.mock.calls.flat(),
        ...mockConsole.log.mock.calls.flat(),
        ...mockConsole.error.mock.calls.flat()
      ].join(' ');

      expect(allOutput).not.toMatch(/using minimal data/i);
      expect(allOutput).not.toMatch(/fallback.*seo/i);
    }, 30000);
  });

  describe('Error Recovery and Resilience', () => {
    it('should handle network timeouts gracefully without causing fallbacks', async () => {
      // Test with a URL that will timeout
      const testUrl = 'https://httpstat.us/200?sleep=20000'; // 20 second delay
      const options: TestOptions = {
        timeout: 5000, // 5 second timeout - will cause timeout
        verbose: false
      };

      const result = await checker.testPage(testUrl, options);

      // Should handle timeout gracefully
      expect(result).toBeDefined();
      expect(result.errors.length).toBeGreaterThan(0); // Should have errors due to timeout
      expect(result.passed).toBe(false);

      // Should not have triggered SEO fallbacks due to timeout
      const allOutput = [
        ...mockConsole.warn.mock.calls.flat(),
        ...mockConsole.log.mock.calls.flat(),
        ...mockConsole.error.mock.calls.flat()
      ].join(' ');

      // Timeout errors are acceptable, SEO fallbacks are not
      expect(allOutput).not.toMatch(/FALLBACK.*SEO.*page evaluation failed/i);
    }, 30000);

    it('should maintain context isolation even when individual analyses fail', async () => {
      const testUrls = [
        'https://example.com',
        'https://httpstat.us/500', // Will return 500 error
        'https://httpbin.org/html'
      ];

      const options: TestOptions = {
        timeout: 10000,
        verbose: true,
        maxConcurrent: 2
      };

      const results = await checker.testMultiplePagesParallel(testUrls, options);

      expect(results).toHaveLength(3);

      // Even with one failure, others should succeed
      const successfulResults = results.filter(r => r.passed || r.errors.length === 0);
      expect(successfulResults.length).toBeGreaterThanOrEqual(1);

      // Should maintain isolation messages for successful tests
      const verboseOutput = mockConsole.log.mock.calls.flat().join(' ');
      expect(verboseOutput).toMatch(/SEO analysis with isolated context/i);
    }, 45000);
  });
});
