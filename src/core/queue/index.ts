/**
 * 🔧 Queue System - Main Exports
 * 
 * Centralized exports for the queue system.
 * Provides clean API for consumers.
 */

// Main queue class
export { Queue } from './queue';

// Factory for creating queues
export { QueueFactory } from './queue-factory';

// Base adapter class
export { QueueAdapter } from './queue-adapter';

// Concrete adapters
export { SimpleQueueAdapter } from './adapters/simple-queue-adapter';
export { ParallelQueueAdapter } from './adapters/parallel-queue-adapter';

// Types and interfaces
export * from './types';

// Re-export for convenience
export type {
  QueueItem,
  QueueConfig,
  QueueStatistics,
  QueueProcessor,
  QueueResult,
  QueueEventCallbacks,
  QueueType,
  QueueAdapterOptions,
  PriorityPattern
} from './types';
