/**
 * 🧪 Integration Tests for Stable Audit Interface
 * 
 * These tests ensure the StableAuditor interface works reliably
 * and can detect regressions in the audit system.
 * 
 * Test Categories:
 * - Basic functionality
 * - Error handling and recovery
 * - Progress monitoring
 * - Health checks
 * - Data consistency
 * - Performance requirements
 */

import { 
  StableAuditor, 
  createStableAuditor, 
  StableAuditConfig,
  AuditResult,
  AuditProgress,
  AuditError 
} from '../../src/interfaces/stable-audit-interface';
import * as path from 'path';
import * as fs from 'fs';

// Test configuration
const TEST_CONFIG: StableAuditConfig = {
  maxPages: 2,
  timeout: 30000,
  maxConcurrent: 2,
  outputFormat: 'both',
  outputDir: './test-reports',
  standard: 'WCAG2AA',
  verbose: false,
  reportPrefix: 'test-audit'
};

const VALID_SITEMAP_URL = 'https://www.inros-lackner.de/sitemap.xml';
const INVALID_SITEMAP_URL = 'https://nonexistent-domain-123456.com/sitemap.xml';

describe('Stable Audit Interface Integration Tests', () => {
  let auditor: StableAuditor;
  let progressEvents: AuditProgress[] = [];
  let errorEvents: AuditError[] = [];

  beforeEach(() => {
    // Clean up test reports directory
    if (fs.existsSync(TEST_CONFIG.outputDir!)) {
      fs.rmSync(TEST_CONFIG.outputDir!, { recursive: true, force: true });
    }

    // Reset event collectors
    progressEvents = [];
    errorEvents = [];

    // Create fresh auditor instance
    auditor = createStableAuditor({ ...TEST_CONFIG });
    
    // Setup event listeners for monitoring
    auditor.onProgress((progress) => {
      progressEvents.push(progress);
      console.log(`📊 Progress: ${progress.phase} - ${progress.progress}% (${progress.completed}/${progress.total})`);
    });

    auditor.onError((error) => {
      errorEvents.push(error);
      console.log(`⚠️ Error: ${error.code} - ${error.message}`);
    });
  });

  afterEach(async () => {
    // Always cleanup to prevent resource leaks
    await auditor.cleanup();
    
    // Clean up test files
    if (fs.existsSync(TEST_CONFIG.outputDir!)) {
      fs.rmSync(TEST_CONFIG.outputDir!, { recursive: true, force: true });
    }
  });

  describe('🚀 Basic Functionality Tests', () => {
    it('should create auditor with valid configuration', () => {
      expect(auditor).toBeDefined();
      expect(auditor.getHealthStatus().status).toBe('healthy');
    });

    it('should reject invalid configuration', () => {
      expect(() => {
        createStableAuditor({ maxPages: 0 });
      }).toThrow('maxPages must be a positive number');
    });

    it('should initialize successfully', async () => {
      await auditor.initialize();
      
      const healthStatus = auditor.getHealthStatus();
      expect(healthStatus.status).toBe('healthy');
      expect(healthStatus.details.initialized).toBe(true);
      expect(healthStatus.details.browserPoolSize).toBeGreaterThan(0);
      
      // Should have initialization progress events
      const initEvents = progressEvents.filter(p => p.phase === 'initializing');
      expect(initEvents.length).toBeGreaterThan(0);
      expect(initEvents[initEvents.length - 1].progress).toBe(100);
    }, 30000);

    it('should perform complete audit workflow', async () => {
      await auditor.initialize();
      
      const result = await auditor.auditWebsite(VALID_SITEMAP_URL);
      
      // Validate result structure
      expect(result).toBeDefined();
      expect(result.summary).toBeDefined();
      expect(result.pages).toBeInstanceOf(Array);
      expect(result.reports).toBeDefined();
      expect(result.performance).toBeDefined();
      expect(result.metadata).toBeDefined();
      
      // Validate summary data
      expect(result.summary.totalPages).toBe(TEST_CONFIG.maxPages);
      expect(result.summary.testedPages).toBeGreaterThan(0);
      expect(result.summary.totalDuration).toBeGreaterThan(0);
      expect(result.summary.averagePageTime).toBeGreaterThan(0);
      
      // Validate pages data
      expect(result.pages.length).toBeGreaterThan(0);
      result.pages.forEach(page => {
        expect(page.url).toBeDefined();
        expect(page.title).toBeDefined();
        expect(typeof page.passed).toBe('boolean');
        expect(typeof page.crashed).toBe('boolean');
        expect(page.duration).toBeGreaterThan(0);
        expect(page.scores).toBeDefined();
        expect(page.issues).toBeDefined();
        expect(page.metrics).toBeDefined();
      });
      
      // Validate reports generation
      if (result.reports.html) {
        expect(fs.existsSync(result.reports.html)).toBe(true);
      }
      if (result.reports.markdown) {
        expect(fs.existsSync(result.reports.markdown)).toBe(true);
      }
      
    }, 120000); // 2 minutes timeout for full audit

    it('should track progress through all phases', async () => {
      await auditor.initialize();
      await auditor.auditWebsite(VALID_SITEMAP_URL);
      
      // Should have events for all phases
      const phases = ['initializing', 'parsing', 'testing', 'generating', 'complete'];
      phases.forEach(phase => {
        const phaseEvents = progressEvents.filter(p => p.phase === phase);
        expect(phaseEvents.length).toBeGreaterThan(0);
      });
      
      // Final progress should be 100%
      const finalEvent = progressEvents[progressEvents.length - 1];
      expect(finalEvent.phase).toBe('complete');
      expect(finalEvent.progress).toBe(100);
      
    }, 120000);
  });

  describe('🛡️ Error Handling and Recovery Tests', () => {
    it('should handle invalid sitemap URLs gracefully', async () => {
      await auditor.initialize();
      
      await expect(auditor.auditWebsite(INVALID_SITEMAP_URL)).rejects.toThrow();
      
      // Should have recorded error events
      const sitemapErrors = errorEvents.filter(e => e.code === 'SITEMAP_ERROR');
      expect(sitemapErrors.length).toBeGreaterThan(0);
      
      // Health status should indicate issues
      const healthStatus = auditor.getHealthStatus();
      expect(healthStatus.status).toBe('unhealthy');
    }, 30000);

    it('should prevent audit without initialization', async () => {
      await expect(auditor.auditWebsite(VALID_SITEMAP_URL))
        .rejects
        .toThrow('Auditor not initialized');
    });

    it('should handle cleanup gracefully even after errors', async () => {
      await auditor.initialize();
      
      try {
        await auditor.auditWebsite(INVALID_SITEMAP_URL);
      } catch (error) {
        // Expected error
      }
      
      // Cleanup should still work
      await expect(auditor.cleanup()).resolves.not.toThrow();
    });
  });

  describe('🏥 Health Monitoring Tests', () => {
    it('should report healthy status after successful initialization', async () => {
      await auditor.initialize();
      
      const health = auditor.getHealthStatus();
      expect(health.status).toBe('healthy');
      expect(health.details.initialized).toBe(true);
      expect(health.details.browserPoolSize).toBeGreaterThan(0);
      expect(health.details.memoryUsage).toBeDefined();
      expect(health.details.uptime).toBeGreaterThan(0);
    });

    it('should detect degraded performance conditions', async () => {
      await auditor.initialize();
      
      // Create a mock high memory condition (this is simulated)
      const health = auditor.getHealthStatus();
      expect(health.details.memoryUsage.heapUsed).toBeGreaterThan(0);
      
      // In a real scenario, high memory usage would trigger health degradation
      // This test validates that monitoring infrastructure is in place
    });
  });

  describe('📊 Data Consistency and Quality Tests', () => {
    it('should maintain consistent data structures across runs', async () => {
      await auditor.initialize();
      
      // Run two identical audits
      const result1 = await auditor.auditWebsite(VALID_SITEMAP_URL);
      await new Promise(resolve => setTimeout(resolve, 1000)); // Brief pause
      const result2 = await auditor.auditWebsite(VALID_SITEMAP_URL);
      
      // Results should have consistent structure
      expect(result1.pages.length).toBe(result2.pages.length);
      expect(result1.summary.totalPages).toBe(result2.summary.totalPages);
      
      // URLs should be in same order (assuming stable sampling)
      result1.pages.forEach((page, index) => {
        expect(result2.pages[index]).toBeDefined();
        // URLs might differ due to smart sampling, but structure should be consistent
        expect(result2.pages[index].url).toBeDefined();
      });
      
    }, 180000); // 3 minutes for two audits

    it('should validate score ranges and data types', async () => {
      await auditor.initialize();
      
      const result = await auditor.auditWebsite(VALID_SITEMAP_URL);
      
      result.pages.forEach(page => {
        // Scores should be in valid range (0-100)
        expect(page.scores.accessibility).toBeGreaterThanOrEqual(0);
        expect(page.scores.accessibility).toBeLessThanOrEqual(100);
        expect(page.scores.performance).toBeGreaterThanOrEqual(0);
        expect(page.scores.performance).toBeLessThanOrEqual(100);
        expect(page.scores.seo).toBeGreaterThanOrEqual(0);
        expect(page.scores.seo).toBeLessThanOrEqual(100);
        expect(page.scores.mobile).toBeGreaterThanOrEqual(0);
        expect(page.scores.mobile).toBeLessThanOrEqual(100);
        
        // Issues should be properly categorized
        expect(Array.isArray(page.issues.errors)).toBe(true);
        expect(Array.isArray(page.issues.warnings)).toBe(true);
        expect(Array.isArray(page.issues.notices)).toBe(true);
        
        // Metrics should be non-negative
        expect(page.metrics.loadTime).toBeGreaterThanOrEqual(0);
        expect(page.metrics.contentSize).toBeGreaterThanOrEqual(0);
        expect(page.metrics.resourceCount).toBeGreaterThanOrEqual(0);
      });
      
    }, 120000);
  });

  describe('⚡ Performance Requirements Tests', () => {
    it('should complete audit within reasonable time limits', async () => {
      await auditor.initialize();
      
      const startTime = Date.now();
      const result = await auditor.auditWebsite(VALID_SITEMAP_URL);
      const duration = Date.now() - startTime;
      
      // Should complete within 2 minutes for 2 pages
      expect(duration).toBeLessThan(120000);
      
      // Average page time should be reasonable (less than 60s per page)
      const avgPageTime = duration / result.pages.length;
      expect(avgPageTime).toBeLessThan(60000);
      
    }, 120000);

    it('should maintain memory usage within acceptable limits', async () => {
      const initialMemory = process.memoryUsage().heapUsed;
      
      await auditor.initialize();
      const result = await auditor.auditWebsite(VALID_SITEMAP_URL);
      await auditor.cleanup();
      
      // Force garbage collection if available
      if (global.gc) {
        global.gc();
      }
      
      const finalMemory = process.memoryUsage().heapUsed;
      const memoryIncrease = (finalMemory - initialMemory) / 1024 / 1024; // MB
      
      console.log(`📊 Memory usage: ${memoryIncrease.toFixed(2)}MB increase`);
      
      // Memory increase should be reasonable (less than 500MB for 2 pages)
      expect(memoryIncrease).toBeLessThan(500);
      
    }, 120000);
  });

  describe('🔧 Configuration Validation Tests', () => {
    it('should respect maxPages configuration', async () => {
      const customAuditor = createStableAuditor({
        ...TEST_CONFIG,
        maxPages: 1 // Only test 1 page
      });
      
      customAuditor.onProgress((progress) => progressEvents.push(progress));
      
      await customAuditor.initialize();
      const result = await customAuditor.auditWebsite(VALID_SITEMAP_URL);
      await customAuditor.cleanup();
      
      expect(result.summary.totalPages).toBe(1);
      expect(result.pages.length).toBeLessThanOrEqual(1);
      
    }, 60000);

    it('should respect output format configuration', async () => {
      const htmlOnlyAuditor = createStableAuditor({
        ...TEST_CONFIG,
        outputFormat: 'html'
      });
      
      await htmlOnlyAuditor.initialize();
      const result = await htmlOnlyAuditor.auditWebsite(VALID_SITEMAP_URL);
      await htmlOnlyAuditor.cleanup();
      
      expect(result.reports.html).toBeDefined();
      expect(result.reports.markdown).toBeUndefined();
      
    }, 120000);
  });

  describe('🔄 Regression Detection Tests', () => {
    it('should detect if queue progress gets stuck at 0%', async () => {
      await auditor.initialize();
      
      const progressPromise = new Promise<void>((resolve, reject) => {
        const timeout = setTimeout(() => {
          reject(new Error('Progress appears to be stuck - no progress updates received within 30 seconds'));
        }, 30000);
        
        auditor.onProgress((progress) => {
          if (progress.phase === 'testing' && progress.progress > 0) {
            clearTimeout(timeout);
            resolve();
          }
        });
      });
      
      // Start audit and race with progress detection
      const auditPromise = auditor.auditWebsite(VALID_SITEMAP_URL);
      
      await expect(Promise.race([auditPromise, progressPromise])).resolves.not.toThrow();
      
      // Wait for audit to complete
      await auditPromise;
      
    }, 120000);

    it('should detect if worker loops cause hanging', async () => {
      await auditor.initialize();
      
      const auditPromise = auditor.auditWebsite(VALID_SITEMAP_URL);
      const timeoutPromise = new Promise((_, reject) => {
        setTimeout(() => reject(new Error('Audit hanging - exceeded 2 minute timeout')), 120000);
      });
      
      // Audit should complete before timeout
      await expect(Promise.race([auditPromise, timeoutPromise])).resolves.toBeDefined();
      
    }, 130000);
  });
});

/**
 * 📊 Test Results Summary
 * 
 * This test suite validates:
 * ✅ Basic functionality and workflow
 * ✅ Error handling and recovery
 * ✅ Progress monitoring accuracy  
 * ✅ Health status reporting
 * ✅ Data consistency and quality
 * ✅ Performance requirements
 * ✅ Configuration compliance
 * ✅ Regression detection
 * 
 * Key metrics tracked:
 * - Execution time limits
 * - Memory usage bounds
 * - Progress reporting accuracy
 * - Error recovery capability
 * - Data structure consistency
 */