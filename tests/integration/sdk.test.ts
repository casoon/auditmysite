/**
 * 🧪 SDK Integration Tests
 * 
 * Tests SDK fluent API, event system, and error handling.
 * Uses mocks to avoid expensive I/O operations.
 */

import { AuditSDK } from '../../src/sdk/audit-sdk';
import { AuditOptions, AuditResult, SDKConfig, AuditSDKError, InvalidSitemapError } from '../../src/sdk/types';
import { createMockAuditResult, createMockPageResult } from '../setup';

// Mock the StandardPipeline to avoid real audits
jest.mock('../../src/core/pipeline/standard-pipeline', () => ({
  StandardPipeline: jest.fn().mockImplementation(() => ({
    run: jest.fn().mockResolvedValue({
      summary: {
        testedPages: 3,
        passedPages: 2,
        failedPages: 1,
        crashedPages: 0,
        totalErrors: 5,
        totalWarnings: 2,
        totalDuration: 15000,
        results: [
          createMockPageResult({ passed: true }),
          createMockPageResult({ passed: true, url: 'https://example.com/page2' }),
          createMockPageResult({ passed: false, errors: ['Missing alt text'], url: 'https://example.com/page3' })
        ]
      },
      issues: [],
      outputFiles: []
    })
  }))
}));

// Mock SitemapDiscovery for connection testing
const mockDiscoverSitemap = jest.fn().mockResolvedValue({
  found: true,
  sitemaps: ['https://example.com/sitemap.xml'],
  method: 'direct'
});

jest.mock('../../src/core/parsers/sitemap-discovery', () => ({
  SitemapDiscovery: jest.fn().mockImplementation(() => ({
    discoverSitemap: mockDiscoverSitemap
  }))
}));

describe('AuditSDK', () => {
  let sdk: AuditSDK;

  beforeEach(() => {
    sdk = new AuditSDK();
  });

  describe('SDK Initialization', () => {
    it('should initialize with default configuration', () => {
      const config = sdk.getConfig();
      
      expect(config.timeout).toBe(30000);
      expect(config.maxConcurrency).toBe(3);
      expect(config.defaultOutputDir).toBe('./audit-results');
      expect(config.verbose).toBe(false);
      expect(config.userAgent).toContain('AuditMySite-SDK');
    });

    it('should initialize with custom configuration', () => {
      const customConfig: SDKConfig = {
        timeout: 60000,
        maxConcurrency: 5,
        defaultOutputDir: './custom-reports',
        verbose: true,
        userAgent: 'Custom-Bot/1.0'
      };

      const customSdk = new AuditSDK(customConfig);
      const config = customSdk.getConfig();
      
      expect(config.timeout).toBe(60000);
      expect(config.maxConcurrency).toBe(5);
      expect(config.defaultOutputDir).toBe('./custom-reports');
      expect(config.verbose).toBe(true);
      expect(config.userAgent).toBe('Custom-Bot/1.0');
    });

    it('should update configuration after initialization', () => {
      sdk.configure({ timeout: 45000, verbose: true });
      const config = sdk.getConfig();
      
      expect(config.timeout).toBe(45000);
      expect(config.verbose).toBe(true);
      // Should keep other defaults
      expect(config.maxConcurrency).toBe(3);
    });

    it('should return correct SDK version', () => {
      const version = sdk.getVersion();
      expect(version).toMatch(/^\d+\.\d+\.\d+/); // Semantic version pattern
    });
  });

  describe('Connection Testing', () => {
    it('should test sitemap connection successfully', async () => {
      const result = await sdk.testConnection('https://example.com/sitemap.xml');
      
      expect(result.success).toBe(true);
      expect(result.error).toBeUndefined();
    });

    it('should handle connection failures', async () => {
      // Mock failure case
      mockDiscoverSitemap.mockResolvedValueOnce({ found: false });

      const result = await sdk.testConnection('https://broken-site.com/sitemap.xml');
      
      expect(result.success).toBe(false);
      expect(result.error).toBeDefined();
    });
  });

  describe('Quick Audit', () => {
    it('should run quick audit with minimal options', async () => {
      const result = await sdk.quickAudit('https://example.com/sitemap.xml');
      
      expect(result.sessionId).toBeDefined();
      expect(result.sitemapUrl).toBe('https://example.com/sitemap.xml');
      expect(result.summary.testedPages).toBe(3);
      expect(result.summary.passedPages).toBe(2);
      expect(result.summary.failedPages).toBe(1);
      expect(result.startTime).toBeInstanceOf(Date);
      expect(result.endTime).toBeInstanceOf(Date);
      expect(result.duration).toBeGreaterThan(0);
    });

    it('should run quick audit with custom options', async () => {
      const options: AuditOptions = {
        maxPages: 10,
        standard: 'WCAG2AAA',
        formats: ['html', 'json'],
        includePerformance: true,
        viewport: { width: 1920, height: 1080, isMobile: false }
      };

      const result = await sdk.quickAudit('https://example.com/sitemap.xml', options);
      
      expect(result.metadata.configuration).toMatchObject({
        maxPages: 10,
        standard: 'WCAG2AAA',
        formats: ['html', 'json'],
        includePerformance: true
      });
    });

    it('should run quick audit with callbacks', async () => {
      const callbacks = {
        onStart: jest.fn(),
        onProgress: jest.fn(),
        onComplete: jest.fn()
      };

      const result = await sdk.quickAudit('https://example.com/sitemap.xml', {}, callbacks);
      
      expect(callbacks.onStart).toHaveBeenCalledWith(
        expect.objectContaining({ type: 'audit:start' })
      );
      expect(callbacks.onComplete).toHaveBeenCalledWith(
        expect.objectContaining({ type: 'audit:complete', data: result })
      );
    });
  });

  describe('Fluent API', () => {
    it('should build audit configuration using fluent API', async () => {
      const mockPipeline = require('../../src/core/pipeline/standard-pipeline').StandardPipeline.mock.instances[0];
      
      await sdk.audit()
        .sitemap('https://example.com/sitemap.xml')
        .maxPages(20)
        .standard('WCAG2AAA')
        .formats(['html', 'json', 'csv'])
        .includePerformance(true)
        .includeSeo(true)
        .viewport(1920, 1080, false)
        .timeout(45000)
        .run();

      expect(mockPipeline.run).toHaveBeenCalledWith(
        expect.objectContaining({
          sitemapUrl: 'https://example.com/sitemap.xml',
          maxPages: 20,
          pa11yStandard: 'WCAG2AAA',
          generatePerformanceReport: true,
          generateSeoReport: true,
          timeout: 45000
        })
      );
    });

    it('should validate sitemap URL in fluent API', () => {
      expect(() => {
        sdk.audit().sitemap('invalid-url').run();
      }).rejects.toThrow(InvalidSitemapError);

      expect(() => {
        sdk.audit().sitemap('ftp://example.com/sitemap.xml').run();
      }).rejects.toThrow(InvalidSitemapError);
    });

    it('should validate configuration parameters', () => {
      expect(() => {
        sdk.audit().maxPages(0);
      }).toThrow('maxPages must be between 1 and 10000');

      expect(() => {
        sdk.audit().maxPages(15000);
      }).toThrow('maxPages must be between 1 and 10000');

      expect(() => {
        sdk.audit().timeout(500);
      }).toThrow('timeout must be between 1000ms and 300000ms');

      expect(() => {
        sdk.audit().formats([]);
      }).toThrow('formats must be a non-empty array');
    });

    it('should require sitemap URL before running', async () => {
      await expect(
        sdk.audit().maxPages(10).run()
      ).rejects.toThrow('sitemap URL is required');
    });
  });

  describe('Event System', () => {
    it('should emit events during audit execution', async () => {
      const events: any[] = [];
      const eventHandler = (event: any) => events.push(event);

      await sdk.audit()
        .sitemap('https://example.com/sitemap.xml')
        .maxPages(5)
        .on('audit:start', eventHandler)
        .on('audit:progress', eventHandler)
        .on('audit:complete', eventHandler)
        .run();

      // Should have received start and complete events
      expect(events.filter(e => e.type === 'audit:start')).toHaveLength(1);
      expect(events.filter(e => e.type === 'audit:complete')).toHaveLength(1);

      // All events should have timestamp and sessionId
      events.forEach(event => {
        expect(event.timestamp).toBeInstanceOf(Date);
        expect(event.sessionId).toBeDefined();
      });
    });

    it('should handle event callback errors gracefully', async () => {
      const errorCallback = jest.fn().mockImplementation(() => {
        throw new Error('Callback error');
      });

      // Should not crash the audit even if event callback throws
      await expect(
        sdk.audit()
          .sitemap('https://example.com/sitemap.xml')
          .on('audit:start', errorCallback)
          .run()
      ).resolves.toBeDefined();

      expect(errorCallback).toHaveBeenCalled();
    });

    it('should remove event listeners after audit completion', async () => {
      const eventHandler = jest.fn();

      const result1 = await sdk.audit()
        .sitemap('https://example.com/sitemap.xml')
        .on('audit:start', eventHandler)
        .run();

      const result2 = await sdk.audit()
        .sitemap('https://example.com/sitemap.xml')
        .run();

      // Event handler should only be called for the first audit
      expect(eventHandler).toHaveBeenCalledTimes(1);
    });
  });

  describe('Error Handling', () => {
    it('should handle pipeline execution errors', async () => {
      const mockPipeline = require('../../src/core/pipeline/standard-pipeline').StandardPipeline.mock.instances[0];
      mockPipeline.run.mockRejectedValueOnce(new Error('Pipeline failed'));

      await expect(
        sdk.quickAudit('https://example.com/sitemap.xml')
      ).rejects.toThrow(AuditSDKError);
    });

    it('should emit error events on failures', async () => {
      const mockPipeline = require('../../src/core/pipeline/standard-pipeline').StandardPipeline.mock.instances[0];
      mockPipeline.run.mockRejectedValueOnce(new Error('Pipeline failed'));

      const errorHandler = jest.fn();

      await expect(
        sdk.audit()
          .sitemap('https://example.com/sitemap.xml')
          .on('audit:error', errorHandler)
          .run()
      ).rejects.toThrow();

      expect(errorHandler).toHaveBeenCalledWith(
        expect.objectContaining({
          type: 'audit:error',
          data: expect.any(Error)
        })
      );
    });

    it('should wrap non-SDK errors in AuditSDKError', async () => {
      const mockPipeline = require('../../src/core/pipeline/standard-pipeline').StandardPipeline.mock.instances[0];
      mockPipeline.run.mockRejectedValueOnce(new Error('Generic error'));

      try {
        await sdk.quickAudit('https://example.com/sitemap.xml');
      } catch (error) {
        expect(error).toBeInstanceOf(AuditSDKError);
        expect(error.code).toBe('AUDIT_FAILED');
        expect(error.message).toBe('Generic error');
      }
    });
  });

  describe('Progress Tracking', () => {
    it('should calculate ETA during progress updates', async () => {
      const progressEvents: any[] = [];

      // Mock pipeline to simulate progress
      const mockPipeline = require('../../src/core/pipeline/standard-pipeline').StandardPipeline.mock.instances[0];
      mockPipeline.run.mockImplementation(async (options) => {
        // Simulate progress callbacks
        if (options.onProgress) {
          setTimeout(() => options.onProgress(1, 3, 'https://example.com/page1'), 10);
          setTimeout(() => options.onProgress(2, 3, 'https://example.com/page2'), 20);
          setTimeout(() => options.onProgress(3, 3, 'https://example.com/page3'), 30);
        }
        
        return {
          summary: { testedPages: 3, passedPages: 2, failedPages: 1, totalDuration: 30 },
          issues: [],
          outputFiles: []
        };
      });

      await sdk.audit()
        .sitemap('https://example.com/sitemap.xml')
        .on('audit:progress', (event) => progressEvents.push(event.data))
        .run();

      expect(progressEvents).toHaveLength(3);
      expect(progressEvents[0]).toMatchObject({
        current: 1,
        total: 3,
        percentage: 33,
        currentUrl: 'https://example.com/page1'
      });
      expect(progressEvents[2]).toMatchObject({
        current: 3,
        total: 3,
        percentage: 100,
        currentUrl: 'https://example.com/page3'
      });
    });
  });

  describe('Configuration Merging', () => {
    it('should merge SDK config with audit options correctly', async () => {
      const sdkWithConfig = new AuditSDK({
        timeout: 45000,
        maxConcurrency: 4,
        defaultOutputDir: './custom-reports'
      });

      const mockPipeline = require('../../src/core/pipeline/standard-pipeline').StandardPipeline.mock.instances[0];

      await sdkWithConfig.quickAudit('https://example.com/sitemap.xml', {
        maxPages: 15,
        formats: ['json']
      });

      expect(mockPipeline.run).toHaveBeenCalledWith(
        expect.objectContaining({
          timeout: 45000,
          maxConcurrent: 4,
          outputDir: './custom-reports',
          maxPages: 15
        })
      );
    });
  });

  describe('Report Integration', () => {
    it('should generate reports when formats are specified', async () => {
      const result = await sdk.quickAudit('https://example.com/sitemap.xml', {
        formats: ['html', 'json']
      });

      expect(result.reports).toHaveLength(2);
      expect(result.reports.map(r => r.format)).toEqual(expect.arrayContaining(['html', 'json']));
      expect(result.reports.every(r => r.size > 0)).toBe(true);
    });
  });
});
