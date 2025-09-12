/**
 * 🔧 Unified Queue Types
 * 
 * Common interfaces and types for the unified queue system.
 * This consolidates all queue functionality into a consistent API.
 */

export interface QueueItem<T = any> {
  id: string;
  data: T;
  priority: number;
  status: 'pending' | 'processing' | 'completed' | 'failed' | 'retrying';
  attempts: number;
  maxAttempts: number;
  timestamp: Date;
  startedAt?: Date;
  completedAt?: Date;
  duration?: number;
  error?: string;
  result?: any;
}

export interface QueueConfig {
  maxConcurrent?: number;
  maxRetries?: number;
  retryDelay?: number;
  timeout?: number;
  priorityPatterns?: PriorityPattern[];
  enablePersistence?: boolean;
  enableEvents?: boolean;
  enableProgressReporting?: boolean;
  progressUpdateInterval?: number;
  
  // 🚀 Backpressure and Resource Management
  enableBackpressure?: boolean;    // Enable backpressure system
  maxQueueSize?: number;           // Maximum items in queue before backpressure kicks in
  backpressureThreshold?: number;  // Memory usage threshold (MB) to trigger backpressure
  adaptiveDelay?: boolean;         // Enable adaptive delays based on system load
  memoryCheckInterval?: number;    // Interval for memory checks (ms)
  maxMemoryUsage?: number;         // Hard memory limit (MB) before stopping
  enableGarbageCollection?: boolean; // Force garbage collection periodically
  gcInterval?: number;             // GC interval (ms)
  
  // 🔍 Performance Monitoring
  enablePerformanceMetrics?: boolean;
  metricsCollectionInterval?: number;
  enableResourceMonitoring?: boolean;
}

export interface PriorityPattern {
  pattern: string;
  priority: number;
}

export interface QueueStatistics {
  total: number;
  pending: number;
  processing: number;
  completed: number;
  failed: number;
  retrying: number;
  progress: number;
  averageDuration: number;
  estimatedTimeRemaining: number;
  activeWorkers: number;
  memoryUsage: number;
  cpuUsage: number;
  throughput: number; // items/second
  startTime?: Date;
  endTime?: Date;
  
  // 🚀 Enhanced Performance Metrics
  peakMemoryUsage: number;        // Peak memory usage (MB)
  averageMemoryUsage: number;     // Average memory usage (MB)
  gcCount: number;                // Number of garbage collections performed
  backpressureEvents: number;     // Number of backpressure events
  adaptiveDelayMs: number;        // Current adaptive delay in ms
  queueSizeLimit: number;         // Current queue size limit
  resourceHealthScore: number;    // Overall resource health (0-100)
  
  // 📊 Advanced Queue Metrics
  queueUtilization: number;       // Current queue usage as percentage of limit
  workerEfficiency: number;       // Average worker success rate (0-100)
  systemLoadScore: number;        // Combined CPU/Memory load score (0-100)
  errorBurstDetected: boolean;    // True if error burst pattern detected
  adaptiveScalingActive: boolean; // True if dynamic worker scaling is active
  
  // 📊 Detailed Timing
  medianDuration: number;
  p95Duration: number;
  p99Duration: number;
  minDuration: number;
  maxDuration: number;
}

export interface QueueProcessor<T, R = any> {
  (item: T): Promise<R>;
}

export interface QueueResult<T> {
  completed: QueueItem<T>[];
  failed: QueueItem<T>[];
  statistics: QueueStatistics;
  duration: number;
}

export interface QueueEventCallbacks<T = any> {
  onItemAdded?: (item: QueueItem<T>) => void;
  onItemStarted?: (item: QueueItem<T>) => void;
  onItemCompleted?: (item: QueueItem<T>, result: any) => void;
  onItemFailed?: (item: QueueItem<T>, error: string) => void;
  onItemRetrying?: (item: QueueItem<T>) => void;
  onProgressUpdate?: (statistics: QueueStatistics) => void;
  onQueueEmpty?: () => void;
  onError?: (error: string) => void;
  
  // 🚀 Backpressure and Resource Events
  onBackpressureActivated?: (reason: string, stats: QueueStatistics) => void;
  onBackpressureDeactivated?: (stats: QueueStatistics) => void;
  onMemoryWarning?: (usage: number, limit: number) => void;
  onMemoryCritical?: (usage: number, limit: number) => void;
  onGarbageCollection?: (beforeMB: number, afterMB: number) => void;
  onResourceHealthCheck?: (score: number, details: any) => void;
  onAdaptiveDelayChanged?: (oldDelay: number, newDelay: number) => void;
}

export type QueueType = 'simple' | 'priority' | 'persistent' | 'parallel';

export interface QueueAdapterOptions {
  config: QueueConfig;
  callbacks?: QueueEventCallbacks;
}
