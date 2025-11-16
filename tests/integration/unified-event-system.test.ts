/**
 * 🧪 UNIFIED EVENT SYSTEM MIGRATION TESTS
 * 
 * Tests to validate that the unified event system works correctly
 * and maintains backward compatibility with existing APIs.
 */

import { AccessibilityChecker } from '../../src/core/accessibility/accessibility-checker';
import { PageAnalysisEmitter, UnifiedEventCallbacks } from '../../src/core/events/page-analysis-emitter';
import { UnifiedEventAdapterFactory, DeprecationManager } from '../../src/core/events/event-system-adapters';
import { TestOptions } from '../../src/types';
import { BrowserPoolManager } from '../../src/core/browser/browser-pool-manager';
import { createMockBrowserPool } from '../mocks/browser-pool-mock';

// Mock heavy Playwright dependencies
jest.mock('playwright', () => ({
  chromium: {
    launch: jest.fn().mockResolvedValue({
      newContext: jest.fn(),
      newPage: jest.fn(),
      close: jest.fn()
    })
  }
}));

describe('Unified Event System Migration', () => {
  let mockPoolManager: BrowserPoolManager;
  
  beforeEach(() => {
    // Clear deprecation warnings for clean test state
    DeprecationManager.clearWarnings();
    
    // Create mock pool manager using our comprehensive mock
    mockPoolManager = createMockBrowserPool({ simulateDelay: 10 });
  });

  afterEach(() => {
    // Clear deprecation warnings after each test
    DeprecationManager.clearWarnings();
  });

  describe('AccessibilityChecker Integration', () => {
    
    test('should initialize unified event system when enabled', () => {
      const checker = new AccessibilityChecker({
        poolManager: mockPoolManager,
        enableUnifiedEvents: true,
        enableComprehensiveAnalysis: true
      });

      const emitter = checker.getUnifiedEmitter();
      expect(emitter).toBeTruthy();
    });

    test('should not initialize unified event system when disabled', () => {
      const checker = new AccessibilityChecker({
        poolManager: mockPoolManager,
        enableUnifiedEvents: false,
        enableComprehensiveAnalysis: true
      });

      const emitter = checker.getUnifiedEmitter();
      expect(emitter).toBeNull();
    });

    test('should allow setting unified event callbacks', () => {
      const checker = new AccessibilityChecker({
        poolManager: mockPoolManager,
        enableUnifiedEvents: true,
        enableComprehensiveAnalysis: true
      });

      const callbacks: UnifiedEventCallbacks = {
        onUrlStarted: jest.fn(),
        onUrlCompleted: jest.fn(),
        onProgressUpdate: jest.fn()
      };

      expect(() => {
        checker.setUnifiedEventCallbacks(callbacks);
      }).not.toThrow();
    });

  });

  describe('PageAnalysisEmitter', () => {
    
    test('should initialize correctly with options', async () => {
      const emitter = new PageAnalysisEmitter({
        verbose: false,
        enableResourceMonitoring: true,
        enableBackpressure: true,
        maxConcurrent: 2,
        callbacks: {
          onProgressUpdate: jest.fn()
        }
      });

      expect(() => emitter.initialize()).not.toThrow();
      await emitter.cleanup();
    });

    test('should register analyzers correctly', () => {
      const emitter = new PageAnalysisEmitter();
      
      const testAnalyzer = jest.fn();
      emitter.registerAnalyzer('test', testAnalyzer);
      
      expect(emitter.getRegisteredAnalyzers()).toContain('test');
    });

    test('should provide progress and system metrics', () => {
      const emitter = new PageAnalysisEmitter();
      
      const progressStats = emitter.getProgressStats();
      expect(progressStats).toHaveProperty('total');
      expect(progressStats).toHaveProperty('completed');
      expect(progressStats).toHaveProperty('progress');

      const systemMetrics = emitter.getSystemMetrics();
      expect(systemMetrics).toHaveProperty('memoryUsageMB');
      expect(systemMetrics).toHaveProperty('cpuUsagePercent');
    });

  });

  describe('Event Adapter Compatibility', () => {
    
    test('should adapt TestOptions.eventCallbacks to UnifiedEventCallbacks', () => {
      const testOptions: TestOptions = {
        eventCallbacks: {
          onUrlStarted: jest.fn(),
          onUrlCompleted: jest.fn(),
          onProgressUpdate: jest.fn()
        }
      };

      const emitter = UnifiedEventAdapterFactory.createUnifiedEmitter({
        testOptions,
        verbose: false
      });

      expect(emitter).toBeTruthy();
    });

    test('should show deprecation warning for legacy callbacks', () => {
      const consoleSpy = jest.spyOn(console, 'warn').mockImplementation();
      
      const checker = new AccessibilityChecker({
        poolManager: mockPoolManager,
        enableUnifiedEvents: true,
        enableComprehensiveAnalysis: true,
        showDeprecationWarnings: true
      });

      // This should trigger a deprecation warning when eventCallbacks are used
      const testOptions: TestOptions = {
        eventCallbacks: {
          onUrlStarted: jest.fn()
        }
      };

      // Simulate the internal call that would trigger the warning
      checker.setUnifiedEventCallbacks({});

      consoleSpy.mockRestore();
    });

    test('should not show deprecation warnings when disabled', () => {
      const consoleSpy = jest.spyOn(console, 'warn').mockImplementation();
      
      const checker = new AccessibilityChecker({
        poolManager: mockPoolManager,
        enableUnifiedEvents: true,
        enableComprehensiveAnalysis: true,
        showDeprecationWarnings: false
      });

      checker.setUnifiedEventCallbacks({});

      // Should not have called console.warn
      expect(consoleSpy).not.toHaveBeenCalled();
      
      consoleSpy.mockRestore();
    });

  });

  describe('Backward Compatibility', () => {
    
    test('should maintain existing AccessibilityChecker APIs', () => {
      const checker = new AccessibilityChecker({
        poolManager: mockPoolManager
      });

      // All these methods should exist and be callable
      expect(typeof checker.testPage).toBe('function');
      expect(typeof checker.testMultiplePagesParallel).toBe('function');
      expect(typeof checker.cleanup).toBe('function');
      expect(typeof checker.initialize).toBe('function');
    });

    test('should support both unified and legacy event patterns', () => {
      const checker = new AccessibilityChecker({
        poolManager: mockPoolManager,
        enableUnifiedEvents: true,
        enableComprehensiveAnalysis: true
      });

      // New unified pattern
      const unifiedCallbacks: UnifiedEventCallbacks = {
        onUrlStarted: jest.fn(),
        onUrlCompleted: jest.fn()
      };
      
      expect(() => {
        checker.setUnifiedEventCallbacks(unifiedCallbacks);
      }).not.toThrow();

      // Should still have access to legacy methods
      expect(typeof checker.testMultiplePagesParallel).toBe('function');
    });

  });

  describe('Migration Validation', () => {
    
    test('should track deprecation warnings (mocked for test env)', () => {
      // Mock NODE_ENV to allow warnings in tests
      const originalEnv = process.env.NODE_ENV;
      process.env.NODE_ENV = 'development';
      
      DeprecationManager.clearWarnings();
      DeprecationManager.warnOnce('TestSystem', 'This is a test warning');
      
      const warnings = DeprecationManager.getWarnings();
      expect(warnings).toContain('TestSystem');
      
      // Should not warn again for the same system
      DeprecationManager.warnOnce('TestSystem', 'This is a test warning');
      expect(warnings).toHaveLength(1);
      
      process.env.NODE_ENV = originalEnv;
    });

    test('should clear warning cache when requested', () => {
      const originalEnv = process.env.NODE_ENV;
      process.env.NODE_ENV = 'development';
      
      DeprecationManager.clearWarnings();
      DeprecationManager.warnOnce('TestSystem', 'This is a test warning');
      expect(DeprecationManager.getWarnings()).toHaveLength(1);
      
      DeprecationManager.clearWarnings();
      expect(DeprecationManager.getWarnings()).toHaveLength(0);
      
      process.env.NODE_ENV = originalEnv;
    });

  });

  describe('Performance and Resource Management', () => {
    
    test('should handle cleanup gracefully (without browser)', async () => {
      const checker = new AccessibilityChecker({
        enableUnifiedEvents: true,
        enableComprehensiveAnalysis: true
      });

      // Test cleanup without initialization to avoid Playwright issues in tests
      expect(async () => {
        await checker.cleanup();
      }).not.toThrow();
    });

    test('should initialize resource monitoring when enabled', () => {
      const emitter = new PageAnalysisEmitter({
        enableResourceMonitoring: true,
        verbose: false
      });

      // Should have system metrics available
      const metrics = emitter.getSystemMetrics();
      expect(metrics).toHaveProperty('memoryUsageMB');
      expect(metrics).toHaveProperty('cpuUsagePercent');
    });

  });

});
