/**
 * 🔧 Queue Factory
 * 
 * Factory for creating different queue adapter implementations.
 * Centralizes queue creation logic and provides type safety.
 */

import { QueueAdapter } from './queue-adapter';
import { QueueType, QueueConfig, QueueEventCallbacks } from './types';
import { SimpleQueueAdapter } from './adapters/simple-queue-adapter';
import { ParallelQueueAdapter } from './adapters/parallel-queue-adapter';
import { PersistentQueueAdapter } from './adapters/persistent-queue-adapter';

export class QueueFactory {
  /**
   * Create a queue adapter of the specified type
   */
  static create<T = any>(
    type: QueueType,
    config: QueueConfig = {},
    callbacks?: QueueEventCallbacks<T>
  ): QueueAdapter<T> {
    switch (type) {
      case 'simple':
        return new SimpleQueueAdapter<T>({ ...config, maxConcurrent: 1 }, callbacks);
        
      case 'parallel':
        return new ParallelQueueAdapter<T>(config, callbacks);
        
      case 'priority':
        // For now, use parallel adapter with priority patterns
        return new ParallelQueueAdapter<T>({
          ...config,
          priorityPatterns: config.priorityPatterns || [
            { pattern: '/home', priority: 10 },
            { pattern: '/', priority: 9 },
            { pattern: '/about', priority: 8 },
            { pattern: '/contact', priority: 7 },
            { pattern: '/blog', priority: 5 },
            { pattern: '/products', priority: 4 }
          ]
        }, callbacks);
        
      case 'persistent':
        // Use real PersistentQueueAdapter
        return new PersistentQueueAdapter<T>({
          ...config,
          enableAutoSave: true,
          autoSaveInterval: config.progressUpdateInterval || 30000
        }, callbacks);
        
      default:
        throw new Error(`Unsupported queue type: ${type}`);
    }
  }

  /**
   * Get default configuration for a queue type
   */
  static getDefaultConfig(type: QueueType): QueueConfig {
    const baseConfig: QueueConfig = {
      maxRetries: 3,
      retryDelay: 1000,
      timeout: 30000, // Increased from 10000ms to 30000ms for complex analysis
      enableEvents: true,
      enableProgressReporting: true,
      progressUpdateInterval: 1000
    };

    switch (type) {
      case 'simple':
        return {
          ...baseConfig,
          maxConcurrent: 1,
          enablePersistence: false
        };

      case 'parallel':
        return {
          ...baseConfig,
          maxConcurrent: 3,
          enablePersistence: false
        };

      case 'priority':
        return {
          ...baseConfig,
          maxConcurrent: 3,
          enablePersistence: false,
          priorityPatterns: [
            { pattern: '/home', priority: 10 },
            { pattern: '/', priority: 9 },
            { pattern: '/about', priority: 8 },
            { pattern: '/contact', priority: 7 },
            { pattern: '/blog', priority: 5 },
            { pattern: '/products', priority: 4 }
          ]
        };

      case 'persistent':
        return {
          ...baseConfig,
          maxConcurrent: 3,
          enablePersistence: true
        };

      default:
        return baseConfig;
    }
  }

  /**
   * Create queue adapter with optimal settings for accessibility testing
   */
  static createForAccessibilityTesting<T = any>(
    type: QueueType = 'parallel',
    customConfig: Partial<QueueConfig> = {},
    callbacks?: QueueEventCallbacks<T>
  ): QueueAdapter<T> {
    const accessibilityConfig: QueueConfig = {
      maxConcurrent: 2, // Conservative for browser testing
      maxRetries: 3,
      retryDelay: 2000, // Longer delay for browser recovery
      timeout: 90000, // Extended timeout for complex comprehensive accessibility scans
      enableEvents: true,
      enableProgressReporting: true,
      progressUpdateInterval: 2000,
      priorityPatterns: [
        { pattern: '/home', priority: 10 },
        { pattern: '/', priority: 10 },
        { pattern: '/about', priority: 8 },
        { pattern: '/contact', priority: 8 },
        { pattern: '/pricing', priority: 7 },
        { pattern: '/features', priority: 7 },
        { pattern: '/blog', priority: 5 },
        { pattern: '/docs', priority: 5 },
        { pattern: '/legal', priority: 3 },
        { pattern: '/privacy', priority: 3 }
      ],
      ...customConfig
    };

    return this.create<T>(type, accessibilityConfig, callbacks);
  }

  /**
   * Get supported queue types
   */
  static getSupportedTypes(): QueueType[] {
    return ['simple', 'parallel', 'priority', 'persistent'];
  }

  /**
   * Validate queue configuration
   */
  static validateConfig(config: QueueConfig): { valid: boolean; errors: string[] } {
    const errors: string[] = [];

    if (config.maxConcurrent !== undefined) {
      if (config.maxConcurrent < 1 || config.maxConcurrent > 10) {
        errors.push('maxConcurrent must be between 1 and 10');
      }
    }

    if (config.maxRetries !== undefined) {
      if (config.maxRetries < 0 || config.maxRetries > 10) {
        errors.push('maxRetries must be between 0 and 10');
      }
    }

    if (config.retryDelay !== undefined) {
      if (config.retryDelay < 0 || config.retryDelay > 60000) {
        errors.push('retryDelay must be between 0 and 60000ms');
      }
    }

    if (config.timeout !== undefined) {
      if (config.timeout < 1000 || config.timeout > 300000) {
        errors.push('timeout must be between 1000 and 300000ms');
      }
    }

    if (config.progressUpdateInterval !== undefined) {
      if (config.progressUpdateInterval < 100 || config.progressUpdateInterval > 10000) {
        errors.push('progressUpdateInterval must be between 100 and 10000ms');
      }
    }

    return {
      valid: errors.length === 0,
      errors
    };
  }
}
