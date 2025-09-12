/**
 * 🔧 Queue Adapter Interface
 * 
 * Base adapter interface for the unified queue system.
 * Implements the Adapter Pattern to provide a consistent API
 * for different queue implementations.
 */

import { QueueItem, QueueConfig, QueueStatistics, QueueProcessor, QueueResult, QueueEventCallbacks } from './types';

export abstract class QueueAdapter<T = any> {
  protected config: QueueConfig;
  protected callbacks?: QueueEventCallbacks<T>;
  protected items: Map<string, QueueItem<T>> = new Map();
  protected isProcessing = false;
  protected startTime?: Date;
  protected endTime?: Date;

  constructor(config: QueueConfig, callbacks?: QueueEventCallbacks<T>) {
    this.config = {
      maxConcurrent: 3,
      maxRetries: 3,
      retryDelay: 1000,
      timeout: 10000,
      priorityPatterns: [],
      enablePersistence: false,
      enableEvents: true,
      enableProgressReporting: true,
      progressUpdateInterval: 1000,
      ...config
    };
    this.callbacks = callbacks;
  }

  /**
   * Add items to the queue
   */
  abstract enqueue(data: T[], options?: { priority?: number }): string[];

  /**
   * Process all items in the queue
   */
  abstract process(processor: QueueProcessor<T>): Promise<QueueResult<T>>;

  /**
   * Get queue statistics
   */
  abstract getStatistics(): QueueStatistics;

  /**
   * Pause queue processing
   */
  abstract pause(): void;

  /**
   * Resume queue processing
   */
  abstract resume(): void;

  /**
   * Clear all items from the queue
   */
  abstract clear(): void;

  /**
   * Update queue configuration
   */
  configure(config: Partial<QueueConfig>): void {
    this.config = { ...this.config, ...config };
  }

  /**
   * Get current configuration
   */
  getConfiguration(): QueueConfig {
    return { ...this.config };
  }

  /**
   * Check if queue is processing
   */
  isActive(): boolean {
    return this.isProcessing;
  }

  /**
   * Get all items
   */
  getItems(): QueueItem<T>[] {
    return Array.from(this.items.values());
  }

  /**
   * Get items by status
   */
  getItemsByStatus(status: QueueItem<T>['status']): QueueItem<T>[] {
    return this.getItems().filter(item => item.status === status);
  }

  /**
   * Calculate priority for an item based on patterns
   */
  protected calculatePriority(data: T, defaultPriority = 5): number {
    if (!this.config.priorityPatterns?.length) {
      return defaultPriority;
    }

    const dataString = typeof data === 'string' ? data : JSON.stringify(data);
    
    for (const pattern of this.config.priorityPatterns) {
      if (dataString.includes(pattern.pattern)) {
        return pattern.priority;
      }
    }

    return defaultPriority;
  }

  /**
   * Generate unique ID for queue item
   */
  protected generateId(): string {
    return `queue_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  /**
   * Create a new queue item
   */
  protected createQueueItem(data: T, priority?: number): QueueItem<T> {
    const id = this.generateId();
    const calculatedPriority = priority ?? this.calculatePriority(data);

    const item: QueueItem<T> = {
      id,
      data,
      priority: calculatedPriority,
      status: 'pending',
      attempts: 0,
      maxAttempts: this.config.maxRetries || 3,
      timestamp: new Date()
    };

    this.items.set(id, item);
    
    // Trigger callback
    this.callbacks?.onItemAdded?.(item);

    return item;
  }

  /**
   * Update item status
   */
  protected updateItemStatus(id: string, status: QueueItem<T>['status'], data?: Partial<QueueItem<T>>): boolean {
    const item = this.items.get(id);
    if (!item) return false;

    item.status = status;
    
    if (data) {
      Object.assign(item, data);
    }

    // Update timestamps
    if (status === 'processing') {
      item.startedAt = new Date();
      this.callbacks?.onItemStarted?.(item);
    } else if (status === 'completed') {
      item.completedAt = new Date();
      if (item.startedAt) {
        item.duration = item.completedAt.getTime() - item.startedAt.getTime();
      }
      this.callbacks?.onItemCompleted?.(item, item.result);
    } else if (status === 'failed') {
      item.completedAt = new Date();
      if (item.startedAt) {
        item.duration = item.completedAt.getTime() - item.startedAt.getTime();
      }
      this.callbacks?.onItemFailed?.(item, item.error || 'Unknown error');
    } else if (status === 'retrying') {
      this.callbacks?.onItemRetrying?.(item);
    }

    return true;
  }

  /**
   * Get basic statistics
   */
  protected getBaseStatistics(): QueueStatistics {
    const items = this.getItems();
    const total = items.length;
    const pending = items.filter(item => item.status === 'pending').length;
    const processing = items.filter(item => item.status === 'processing').length;
    const completed = items.filter(item => item.status === 'completed').length;
    const failed = items.filter(item => item.status === 'failed').length;
    const retrying = items.filter(item => item.status === 'retrying').length;

    const progress = total > 0 ? ((completed + failed) / total) * 100 : 0;

    // Calculate average duration
    const completedItems = items.filter(item => item.status === 'completed' && item.duration);
    const averageDuration = completedItems.length > 0 
      ? completedItems.reduce((sum, item) => sum + (item.duration || 0), 0) / completedItems.length
      : 0;

    // Estimate remaining time
    const remainingItems = pending + processing + retrying;
    const estimatedTimeRemaining = remainingItems > 0 && averageDuration > 0
      ? (remainingItems * averageDuration) / (this.config.maxConcurrent || 1)
      : 0;

    // System metrics
    const memoryUsage = process.memoryUsage().heapUsed / 1024 / 1024;
    const cpuUsage = process.cpuUsage().user / 1000000;

    // Throughput calculation
    let throughput = 0;
    if (this.startTime && completed > 0) {
      const elapsedSeconds = (Date.now() - this.startTime.getTime()) / 1000;
      throughput = completed / elapsedSeconds;
    }

    // Calculate duration metrics
    const durations = completedItems.map(item => item.duration || 0).filter(d => d > 0).sort((a, b) => a - b);
    const medianDuration = durations.length > 0 ? durations[Math.floor(durations.length / 2)] : 0;
    const p95Duration = durations.length > 0 ? durations[Math.floor(durations.length * 0.95)] : 0;
    const p99Duration = durations.length > 0 ? durations[Math.floor(durations.length * 0.99)] : 0;
    const minDuration = durations.length > 0 ? Math.min(...durations) : 0;
    const maxDuration = durations.length > 0 ? Math.max(...durations) : 0;

    return {
      total,
      pending,
      processing,
      completed,
      failed,
      retrying,
      progress: Math.round(progress * 100) / 100,
      averageDuration: Math.round(averageDuration),
      estimatedTimeRemaining: Math.round(estimatedTimeRemaining),
      activeWorkers: processing,
      memoryUsage: Math.round(memoryUsage * 100) / 100,
      cpuUsage: Math.round(cpuUsage * 100) / 100,
      throughput: Math.round(throughput * 100) / 100,
      startTime: this.startTime,
      endTime: this.endTime,
      
      // Enhanced Performance Metrics
      peakMemoryUsage: memoryUsage, // Current implementation uses current memory as peak
      averageMemoryUsage: memoryUsage,
      gcCount: 0,
      backpressureEvents: 0,
      adaptiveDelayMs: 0,
      queueSizeLimit: this.config.maxQueueSize || 1000,
      resourceHealthScore: 85, // Default good score
      
      // Advanced Queue Metrics
      queueUtilization: Math.min(100, (total / (this.config.maxQueueSize || 1000)) * 100),
      workerEfficiency: completed > 0 ? (completed / (completed + failed)) * 100 : 100,
      systemLoadScore: Math.min(100, memoryUsage / 10 + cpuUsage), // Simple load calculation
      errorBurstDetected: false,
      adaptiveScalingActive: false,
      
      // Detailed Timing
      medianDuration,
      p95Duration,
      p99Duration,
      minDuration,
      maxDuration
    };
  }
}
