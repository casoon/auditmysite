/**
 * 🔧 Unified Queue System
 * 
 * Main class that provides a unified interface for all queue types.
 * Uses the Adapter Pattern to provide consistent API regardless of implementation.
 */

import { QueueAdapter } from './queue-adapter';
import { QueueFactory } from './queue-factory';
import { QueueType, QueueConfig, QueueStatistics, QueueProcessor, QueueResult, QueueEventCallbacks } from './types';

export class UnifiedQueue<T = any> {
  private adapter: QueueAdapter<T>;
  private type: QueueType;
  private config: QueueConfig;

  constructor(
    type: QueueType = 'parallel',
    config: QueueConfig = {},
    callbacks?: QueueEventCallbacks<T>
  ) {
    // Validate configuration
    const validation = QueueFactory.validateConfig(config);
    if (!validation.valid) {
      throw new Error(`Invalid queue configuration: ${validation.errors.join(', ')}`);
    }

    this.type = type;
    this.config = { ...QueueFactory.getDefaultConfig(type), ...config };
    this.adapter = QueueFactory.create<T>(type, this.config, callbacks);
  }

  /**
   * Create queue optimized for accessibility testing
   */
  static forAccessibilityTesting<T = any>(
    type: QueueType = 'parallel',
    customConfig: Partial<QueueConfig> = {},
    callbacks?: QueueEventCallbacks<T>
  ): UnifiedQueue<T> {
    const adapter = QueueFactory.createForAccessibilityTesting<T>(type, customConfig, callbacks);
    const instance = Object.create(UnifiedQueue.prototype);
    instance.adapter = adapter;
    instance.type = type;
    instance.config = adapter.getConfiguration();
    return instance;
  }

  /**
   * Add items to the queue
   */
  enqueue(data: T[], options?: { priority?: number }): string[] {
    return this.adapter.enqueue(data, options);
  }

  /**
   * Add a single item to the queue
   */
  enqueueOne(data: T, priority?: number): string {
    const ids = this.adapter.enqueue([data], { priority });
    return ids[0];
  }

  /**
   * Process all items in the queue
   */
  async process(processor: QueueProcessor<T>): Promise<QueueResult<T>> {
    return this.adapter.process(processor);
  }

  /**
   * Get queue statistics
   */
  getStatistics(): QueueStatistics {
    return this.adapter.getStatistics();
  }

  /**
   * Pause queue processing
   */
  pause(): void {
    this.adapter.pause();
  }

  /**
   * Resume queue processing
   */
  resume(): void {
    this.adapter.resume();
  }

  /**
   * Clear all items from the queue
   */
  clear(): void {
    this.adapter.clear();
  }

  /**
   * Update queue configuration
   */
  configure(config: Partial<QueueConfig>): void {
    // Validate new configuration
    const mergedConfig = { ...this.config, ...config };
    const validation = QueueFactory.validateConfig(mergedConfig);
    if (!validation.valid) {
      throw new Error(`Invalid queue configuration: ${validation.errors.join(', ')}`);
    }

    this.config = mergedConfig;
    this.adapter.configure(config);
  }

  /**
   * Get current configuration
   */
  getConfiguration(): QueueConfig {
    return this.adapter.getConfiguration();
  }

  /**
   * Get queue type
   */
  getType(): QueueType {
    return this.type;
  }

  /**
   * Check if queue is currently processing
   */
  isActive(): boolean {
    return this.adapter.isActive();
  }

  /**
   * Get all queue items
   */
  getItems() {
    return this.adapter.getItems();
  }

  /**
   * Get items by status
   */
  getItemsByStatus(status: 'pending' | 'processing' | 'completed' | 'failed' | 'retrying') {
    return this.adapter.getItemsByStatus(status);
  }

  /**
   * Get queue size
   */
  size(): number {
    return this.getItems().length;
  }

  /**
   * Check if queue is empty
   */
  isEmpty(): boolean {
    return this.size() === 0;
  }

  /**
   * Get performance metrics
   */
  getPerformanceMetrics() {
    const stats = this.getStatistics();
    return {
      throughput: stats.throughput,
      averageDuration: stats.averageDuration,
      efficiency: stats.total > 0 ? (stats.completed / stats.total) * 100 : 0,
      errorRate: stats.total > 0 ? (stats.failed / stats.total) * 100 : 0,
      memoryUsage: stats.memoryUsage,
      cpuUsage: stats.cpuUsage
    };
  }

  /**
   * Export queue state for debugging
   */
  exportState() {
    const stats = this.getStatistics();
    const items = this.getItems();

    return {
      type: this.type,
      config: this.config,
      statistics: stats,
      items: items.map(item => ({
        id: item.id,
        status: item.status,
        priority: item.priority,
        attempts: item.attempts,
        duration: item.duration,
        error: item.error
      })),
      timestamp: new Date().toISOString()
    };
  }

  /**
   * Clean up queue resources
   */
  async cleanup(): Promise<void> {
    // Clear all items
    this.clear();
    
    // If the adapter has a cleanup method, call it
    if (typeof (this.adapter as any).cleanup === 'function') {
      await (this.adapter as any).cleanup();
    }
  }

  /**
   * Create progress reporter that updates at regular intervals
   */
  createProgressReporter(interval: number = 2000): () => void {
    const timer = setInterval(() => {
      const stats = this.getStatistics();
      console.log(`🚀 Queue Progress: ${stats.progress.toFixed(1)}% (${stats.completed}/${stats.total}) | Workers: ${stats.activeWorkers} | ETA: ${Math.round(stats.estimatedTimeRemaining / 1000)}s`);
    }, interval);

    return () => clearInterval(timer);
  }

  /**
   * Wait for queue to complete processing
   */
  async waitForCompletion(checkInterval: number = 500): Promise<void> {
    return new Promise((resolve) => {
      const check = () => {
        if (!this.isActive() && this.getItemsByStatus('pending').length === 0) {
          resolve();
        } else {
          setTimeout(check, checkInterval);
        }
      };
      check();
    });
  }

  /**
   * Process items with automatic retry and progress reporting
   */
  async processWithProgress<R = any>(
    items: T[],
    processor: QueueProcessor<T, R>,
    options?: {
      showProgress?: boolean;
      progressInterval?: number;
    }
  ): Promise<QueueResult<T>> {
    // Add items to queue
    this.enqueue(items);

    // Setup progress reporter
    let stopProgress: (() => void) | undefined;
    if (options?.showProgress !== false) {
      stopProgress = this.createProgressReporter(options?.progressInterval);
    }

    try {
      // Process queue
      const result = await this.process(processor);
      
      // Final progress report
      if (options?.showProgress !== false) {
        const stats = result.statistics;
        console.log(`✅ Queue completed: ${stats.completed}/${stats.total} items | ${stats.failed} failed | Duration: ${Math.round(result.duration / 1000)}s`);
      }

      return result;
    } finally {
      stopProgress?.();
    }
  }
}
