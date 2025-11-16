/**
 * 🧪 Mock Implementation: AccessibilityChecker
 * 
 * Provides a comprehensive mock for AccessibilityChecker with realistic behavior
 * for unit and integration tests.
 */

import { jest } from '@jest/globals';
import { AccessibilityResult } from '../../src/types';

export interface MockAccessibilityCheckerConfig {
  simulateDelay?: number;
  failureRate?: number;
  defaultPassRate?: number;
}

export class MockAccessibilityChecker {
  private config: MockAccessibilityCheckerConfig;
  private initialized: boolean = false;
  
  constructor(config: MockAccessibilityCheckerConfig = {}) {
    this.config = {
      simulateDelay: 100,
      failureRate: 0,
      defaultPassRate: 0.8,
      ...config
    };
  }

  async initialize() {
    if (this.config.simulateDelay) {
      await new Promise(resolve => setTimeout(resolve, this.config.simulateDelay));
    }
    this.initialized = true;
  }

  async testPage(url: string, options?: any): Promise<AccessibilityResult> {
    if (!this.initialized) {
      throw new Error('Checker not initialized');
    }

    // Simulate processing delay
    if (this.config.simulateDelay) {
      await new Promise(resolve => setTimeout(resolve, this.config.simulateDelay));
    }

    // Simulate random failures
    if (Math.random() < (this.config.failureRate || 0)) {
      return this.createFailedResult(url);
    }

    // Create realistic mock result
    const passed = Math.random() < (this.config.defaultPassRate || 0.8);
    return this.createMockResult(url, passed);
  }

  async testMultiplePagesParallel(urls: string[], options?: any): Promise<AccessibilityResult[]> {
    if (!this.initialized) {
      throw new Error('Checker not initialized');
    }

    const results: AccessibilityResult[] = [];
    
    for (const url of urls) {
      const result = await this.testPage(url, options);
      results.push(result);
    }

    return results;
  }

  async cleanup() {
    this.initialized = false;
    return Promise.resolve();
  }

  getUnifiedEmitter() {
    return {
      onUrlStarted: jest.fn(),
      onUrlCompleted: jest.fn(),
      onProgressUpdate: jest.fn(),
      onError: jest.fn(),
      onQueueEmpty: jest.fn(),
      getProgressStats: jest.fn().mockReturnValue({ total: 0, completed: 0, progress: 0 }),
      getSystemMetrics: jest.fn().mockReturnValue({ memoryUsageMB: 100, cpuUsagePercent: 10 }),
      registerAnalyzer: jest.fn(),
      getRegisteredAnalyzers: jest.fn().mockReturnValue([]),
      initialize: jest.fn().mockResolvedValue(undefined),
      cleanup: jest.fn().mockResolvedValue(undefined)
    };
  }

  setUnifiedEventCallbacks(callbacks: any) {
    // Store callbacks for potential use
    this.unifiedCallbacks = callbacks;
  }

  getHealthStatus() {
    return {
      status: this.initialized ? 'healthy' : 'uninitialized',
      details: {
        initialized: this.initialized,
        browserPoolSize: 1,
        memoryUsageMB: 100,
        timestamp: new Date().toISOString()
      }
    };
  }

  private unifiedCallbacks: any = {};

  private createMockResult(url: string, passed: boolean): AccessibilityResult {
    const errorCount = passed ? Math.floor(Math.random() * 3) : Math.floor(Math.random() * 10) + 5;
    const warningCount = Math.floor(Math.random() * 5);

    return {
      url,
      title: `Mock Page - ${url}`,
      passed,
      errors: this.generateMockIssues('error', errorCount),
      warnings: this.generateMockIssues('warning', warningCount),
      imagesWithoutAlt: Math.floor(Math.random() * 5),
      buttonsWithoutLabel: Math.floor(Math.random() * 3),
      headingsCount: Math.floor(Math.random() * 10) + 1,
      duration: this.config.simulateDelay || 100,
      pa11yScore: passed ? Math.floor(Math.random() * 20) + 80 : Math.floor(Math.random() * 40) + 20,
      crashed: false
    };
  }

  private createFailedResult(url: string): AccessibilityResult {
    return {
      url,
      title: `Failed - ${url}`,
      passed: false,
      errors: [{ message: 'Page failed to load', code: 'PAGE_LOAD_FAILED', severity: 'error' }],
      warnings: [],
      imagesWithoutAlt: 0,
      buttonsWithoutLabel: 0,
      headingsCount: 0,
      duration: this.config.simulateDelay || 100,
      crashed: true
    };
  }

  private generateMockIssues(severity: string, count: number) {
    const issues = [];
    const types = ['color-contrast', 'missing-alt', 'missing-label', 'invalid-aria', 'heading-order'];
    
    for (let i = 0; i < count; i++) {
      issues.push({
        message: `Mock ${severity} ${i + 1}`,
        code: types[Math.floor(Math.random() * types.length)],
        severity,
        selector: `div:nth-child(${i + 1})`,
        context: '<div>Mock context</div>'
      });
    }
    
    return issues;
  }
}

/**
 * Create a mock AccessibilityChecker instance
 */
export function createMockAccessibilityChecker(config?: MockAccessibilityCheckerConfig): any {
  return new MockAccessibilityChecker(config);
}

/**
 * Jest mock factory for AccessibilityChecker
 */
export function mockAccessibilityChecker(config?: MockAccessibilityCheckerConfig) {
  const mock = createMockAccessibilityChecker(config);
  
  return {
    AccessibilityChecker: jest.fn().mockImplementation(() => mock)
  };
}
