/**
 * 🧪 Mock Implementation: SitemapDiscovery
 * 
 * Provides a mock for SitemapDiscovery to test sitemap-related functionality
 * without making actual HTTP requests.
 */

import { jest } from '@jest/globals';

export interface MockSitemapDiscoveryConfig {
  simulateDelay?: number;
  failureRate?: number;
  defaultUrls?: string[];
}

export class MockSitemapDiscovery {
  private config: MockSitemapDiscoveryConfig;
  
  constructor(config: MockSitemapDiscoveryConfig = {}) {
    this.config = {
      simulateDelay: 50,
      failureRate: 0,
      defaultUrls: [
        'https://example.com',
        'https://example.com/about',
        'https://example.com/contact'
      ],
      ...config
    };
  }

  async discoverSitemap(url: string) {
    // Simulate delay
    if (this.config.simulateDelay) {
      await new Promise(resolve => setTimeout(resolve, this.config.simulateDelay));
    }

    // Simulate failures
    if (Math.random() < (this.config.failureRate || 0)) {
      return {
        found: false,
        sitemaps: [],
        method: 'none'
      };
    }

    // Return mock sitemap discovery result
    return {
      found: true,
      sitemaps: [url],
      method: 'direct',
      urls: this.config.defaultUrls
    };
  }

  async fetchSitemap(url: string) {
    if (this.config.simulateDelay) {
      await new Promise(resolve => setTimeout(resolve, this.config.simulateDelay));
    }

    if (Math.random() < (this.config.failureRate || 0)) {
      throw new Error(`Failed to fetch sitemap: ${url}`);
    }

    return {
      urls: this.config.defaultUrls,
      totalUrls: this.config.defaultUrls?.length || 0
    };
  }
}

/**
 * Create a mock SitemapDiscovery instance
 */
export function createMockSitemapDiscovery(config?: MockSitemapDiscoveryConfig): any {
  return new MockSitemapDiscovery(config);
}

/**
 * Jest mock factory for SitemapDiscovery
 */
export function mockSitemapDiscovery(config?: MockSitemapDiscoveryConfig) {
  const mock = createMockSitemapDiscovery(config);
  
  return {
    SitemapDiscovery: jest.fn().mockImplementation(() => mock)
  };
}
