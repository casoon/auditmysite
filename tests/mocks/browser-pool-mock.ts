/**
 * 🧪 Mock Implementation: BrowserPoolManager
 * 
 * Provides a comprehensive mock for BrowserPoolManager with realistic behavior
 * for unit and integration tests.
 */

import { jest } from '@jest/globals';

export interface MockBrowserPoolConfig {
  simulateDelay?: number;
  simulateFailure?: boolean;
  failureRate?: number;
}

export class MockBrowserPoolManager {
  private config: MockBrowserPoolConfig;
  private activeBrowsers: number = 0;
  private totalAcquired: number = 0;
  private totalReleased: number = 0;
  
  constructor(config: MockBrowserPoolConfig = {}) {
    this.config = {
      simulateDelay: 10,
      simulateFailure: false,
      failureRate: 0,
      ...config
    };
  }

  async acquire() {
    // Simulate acquisition delay
    if (this.config.simulateDelay) {
      await new Promise(resolve => setTimeout(resolve, this.config.simulateDelay));
    }

    // Simulate random failures
    if (this.config.simulateFailure || Math.random() < (this.config.failureRate || 0)) {
      throw new Error('Failed to acquire browser from pool');
    }

    this.activeBrowsers++;
    this.totalAcquired++;

    const mockPage = {
      goto: jest.fn().mockResolvedValue(undefined),
      evaluate: jest.fn().mockResolvedValue({}),
      content: jest.fn().mockResolvedValue('<html><body>Mock Page</body></html>'),
      title: jest.fn().mockResolvedValue('Mock Page Title'),
      url: jest.fn().mockReturnValue('https://example.com'),
      close: jest.fn().mockResolvedValue(undefined),
      waitForLoadState: jest.fn().mockResolvedValue(undefined),
      screenshot: jest.fn().mockResolvedValue(Buffer.from('mock-screenshot'))
    };

    const mockContext = {
      newPage: jest.fn().mockResolvedValue(mockPage),
      close: jest.fn().mockResolvedValue(undefined)
    };

    const mockBrowser = {
      newContext: jest.fn().mockResolvedValue(mockContext),
      newPage: jest.fn().mockResolvedValue(mockPage),
      close: jest.fn().mockResolvedValue(undefined),
      isConnected: jest.fn().mockReturnValue(true)
    };

    const release = async () => {
      this.activeBrowsers--;
      this.totalReleased++;
      await mockContext.close();
    };

    return {
      browser: mockBrowser,
      context: mockContext,
      release
    };
  }

  async warmUp(count: number) {
    // Simulate warmup
    if (this.config.simulateDelay) {
      await new Promise(resolve => setTimeout(resolve, this.config.simulateDelay * count));
    }
  }

  async shutdown() {
    this.activeBrowsers = 0;
    return Promise.resolve();
  }

  getMetrics() {
    return {
      active: this.activeBrowsers,
      totalRequests: this.totalAcquired,
      created: this.totalAcquired,
      reused: Math.max(0, this.totalAcquired - 1),
      efficiency: this.totalAcquired > 1 ? ((this.totalAcquired - 1) / this.totalAcquired) * 100 : 0
    };
  }

  getStats() {
    return {
      active: this.activeBrowsers,
      idle: 0,
      total: this.activeBrowsers
    };
  }

  async cleanup() {
    return this.shutdown();
  }
}

/**
 * Create a mock BrowserPoolManager instance
 */
export function createMockBrowserPool(config?: MockBrowserPoolConfig): any {
  return new MockBrowserPoolManager(config);
}

/**
 * Jest mock factory for BrowserPoolManager
 */
export function mockBrowserPoolManager(config?: MockBrowserPoolConfig) {
  const mock = createMockBrowserPool(config);
  
  return {
    BrowserPoolManager: jest.fn().mockImplementation(() => mock)
  };
}
