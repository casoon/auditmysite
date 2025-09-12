/**
 * 🔄 EVENT SYSTEM ADAPTERS
 * 
 * Provides backward compatibility for existing event systems while
 * internally using the unified PageAnalysisEmitter system.
 * 
 * 🎯 CONSOLIDATES:
 * - TestOptions.eventCallbacks -> UnifiedEventCallbacks
 * - EventDrivenQueueOptions.eventCallbacks -> UnifiedEventCallbacks
 * - ParallelTestManager callbacks -> UnifiedEventCallbacks
 * - Direct callback patterns -> UnifiedEventCallbacks
 * 
 * 🚨 DEPRECATION STRATEGY:
 * - Mark old systems as @deprecated
 * - Provide migration guides in comments
 * - Log deprecation warnings when old systems are used
 * - Maintain full functionality during transition period
 */

import { TestOptions } from '../../types';
import { PageAnalysisEmitter, UnifiedEventCallbacks, ProgressStats } from './page-analysis-emitter';

/**
 * 🔄 TestOptions Event Callbacks Adapter
 * 
 * Converts existing TestOptions.eventCallbacks to UnifiedEventCallbacks
 * 
 * @deprecated This adapter maintains compatibility but will be removed in v3.0.0
 * Use UnifiedEventCallbacks directly instead.
 */
export class TestOptionsEventAdapter {
  
  /**
   * Convert TestOptions.eventCallbacks to UnifiedEventCallbacks
   */
  static adaptTestOptionsCallbacks(options: TestOptions): UnifiedEventCallbacks {
    if (!options.eventCallbacks) {
      return {};
    }

    // Log deprecation warning (unless suppressed for CI/CD environments)
    const suppressWarnings = process.env.NODE_ENV === 'test' || 
                             process.env.CI === 'true' ||
                             process.env.NODE_ENV === 'production' ||
                             process.env.AUDITMYSITE_SUPPRESS_DEPRECATIONS === 'true';
    
    if (!suppressWarnings) {
      console.warn(`
⚠️  DEPRECATION WARNING: TestOptions.eventCallbacks is deprecated
📋 Current usage detected in your code
🚀 Migration Guide:
   Instead of: testOptions.eventCallbacks = { onUrlStarted: ... }
   Use: unifiedEmitter.setEventCallbacks({ onUrlStarted: ... })

🗓️  This will be removed in AuditMySite v3.0.0
      `);
    }

    const unified: UnifiedEventCallbacks = {};

    // Map existing callbacks to unified interface
    if (options.eventCallbacks.onUrlStarted) {
      unified.onUrlStarted = options.eventCallbacks.onUrlStarted;
    }

    if (options.eventCallbacks.onUrlCompleted) {
      unified.onUrlCompleted = options.eventCallbacks.onUrlCompleted;
    }

    if (options.eventCallbacks.onUrlFailed) {
      unified.onUrlFailed = options.eventCallbacks.onUrlFailed;
    }

    if (options.eventCallbacks.onProgressUpdate) {
      unified.onProgressUpdate = (stats: ProgressStats) => {
        // Adapt ProgressStats to the format expected by TestOptions
        const adaptedStats = {
          total: stats.total,
          completed: stats.completed,
          failed: stats.failed,
          progress: stats.progress,
          memoryUsage: stats.memoryUsage,
          activeWorkers: stats.activeWorkers
        };
        options.eventCallbacks!.onProgressUpdate!(adaptedStats);
      };
    }

    if (options.eventCallbacks.onQueueEmpty) {
      unified.onQueueEmpty = options.eventCallbacks.onQueueEmpty;
    }

    return unified;
  }
}

/**
 * 🔄 EventDrivenQueue Callbacks Adapter
 * 
 * @deprecated Use UnifiedEventCallbacks directly
 */
export interface LegacyEventDrivenQueueCallbacks {
  onUrlAdded?: (url: string, priority: number) => void;
  onUrlStarted?: (url: string) => void;
  onUrlCompleted?: (url: string, result: any, duration: number) => void;
  onUrlFailed?: (url: string, error: string, attempts: number) => void;
  onUrlRetrying?: (url: string, attempts: number) => void;
  onQueueEmpty?: () => void;
  onProgressUpdate?: (stats: any) => void;
  onError?: (error: string) => void;
  onShortStatus?: (status: string) => void;
  onBackpressureActivated?: (reason: string) => void;
  onBackpressureDeactivated?: () => void;
  onResourceWarning?: (usage: number, limit: number) => void;
  onResourceCritical?: (usage: number, limit: number) => void;
  onGarbageCollection?: (beforeMB: number, afterMB?: number) => void;
}

export class EventDrivenQueueAdapter {
  
  /**
   * Convert EventDrivenQueue callbacks to UnifiedEventCallbacks
   * 
   * @deprecated This adapter will be removed in v3.0.0
   */
  static adaptEventDrivenQueueCallbacks(callbacks: LegacyEventDrivenQueueCallbacks): UnifiedEventCallbacks {
    // Log deprecation warning (unless suppressed for CI/CD environments)
    const suppressWarnings = process.env.NODE_ENV === 'test' || 
                             process.env.CI === 'true' ||
                             process.env.NODE_ENV === 'production' ||
                             process.env.AUDITMYSITE_SUPPRESS_DEPRECATIONS === 'true';
    
    if (!suppressWarnings) {
      console.warn(`
⚠️  DEPRECATION WARNING: EventDrivenQueue callback pattern is deprecated
📋 Legacy EventDrivenQueueOptions.eventCallbacks detected
🚀 Migration Guide:
   Replace EventDrivenQueue with PageAnalysisEmitter
   Use unified callback interface instead of separate queue system

🗓️  EventDrivenQueue will be removed in AuditMySite v3.0.0
      `);
    }

    const unified: UnifiedEventCallbacks = {};

    // Direct mappings (these interfaces are mostly compatible)
    if (callbacks.onUrlAdded) {
      unified.onUrlAdded = (url: string, priority?: number) => {
        callbacks.onUrlAdded!(url, priority || 0);
      };
    }
    if (callbacks.onUrlStarted) unified.onUrlStarted = callbacks.onUrlStarted;
    if (callbacks.onUrlCompleted) unified.onUrlCompleted = callbacks.onUrlCompleted;
    if (callbacks.onUrlFailed) unified.onUrlFailed = callbacks.onUrlFailed;
    if (callbacks.onUrlRetrying) unified.onUrlRetrying = callbacks.onUrlRetrying;
    if (callbacks.onQueueEmpty) unified.onQueueEmpty = callbacks.onQueueEmpty;
    if (callbacks.onProgressUpdate) {
      unified.onProgressUpdate = callbacks.onProgressUpdate;
    }
    if (callbacks.onError) unified.onError = callbacks.onError;
    if (callbacks.onShortStatus) unified.onShortStatus = callbacks.onShortStatus;
    if (callbacks.onBackpressureActivated) unified.onBackpressureActivated = callbacks.onBackpressureActivated;
    if (callbacks.onBackpressureDeactivated) unified.onBackpressureDeactivated = callbacks.onBackpressureDeactivated;
    if (callbacks.onResourceWarning) {
      unified.onResourceWarning = (usage: number, limit: number, type: 'memory' | 'cpu') => {
        callbacks.onResourceWarning!(usage, limit);
      };
    }
    if (callbacks.onResourceCritical) {
      unified.onResourceCritical = (usage: number, limit: number, type: 'memory' | 'cpu') => {
        callbacks.onResourceCritical!(usage, limit);
      };
    }
    if (callbacks.onGarbageCollection) unified.onGarbageCollection = callbacks.onGarbageCollection;

    return unified;
  }
}

/**
 * 🔄 ParallelTestManager Callbacks Adapter
 * 
 * @deprecated Use UnifiedEventCallbacks directly
 */
export interface LegacyParallelTestManagerCallbacks {
  onTestStart?: (url: string) => void;
  onTestComplete?: (url: string, result: any) => void;
  onTestError?: (url: string, error: string) => void;
  onProgressUpdate?: (stats: any) => void;
  onQueueEmpty?: () => void;
}

export class ParallelTestManagerAdapter {
  
  /**
   * Convert ParallelTestManager callbacks to UnifiedEventCallbacks
   * 
   * @deprecated This adapter will be removed in v3.0.0
   */
  static adaptParallelTestManagerCallbacks(callbacks: LegacyParallelTestManagerCallbacks): UnifiedEventCallbacks {
    // Log deprecation warning (unless suppressed for CI/CD environments)
    const suppressWarnings = process.env.NODE_ENV === 'test' || 
                             process.env.CI === 'true' ||
                             process.env.NODE_ENV === 'production' ||
                             process.env.AUDITMYSITE_SUPPRESS_DEPRECATIONS === 'true';
    
    if (!suppressWarnings) {
      console.warn(`
⚠️  DEPRECATION WARNING: ParallelTestManager callback pattern is deprecated
📋 Legacy ParallelTestManager callbacks detected
🚀 Migration Guide:
   Replace ParallelTestManager with PageAnalysisEmitter
   Use unified event system for better performance and consistency

🗓️  ParallelTestManager will be removed in AuditMySite v3.0.0
      `);
    }

    const unified: UnifiedEventCallbacks = {};

    if (callbacks.onTestStart) unified.onUrlStarted = callbacks.onTestStart;
    if (callbacks.onTestComplete) {
      unified.onUrlCompleted = (url: string, result: any, duration: number) => {
        callbacks.onTestComplete!(url, result);
      };
    }
    if (callbacks.onTestError) {
      unified.onUrlFailed = (url: string, error: string, attempts: number) => {
        callbacks.onTestError!(url, error);
      };
    }
    if (callbacks.onProgressUpdate) unified.onProgressUpdate = callbacks.onProgressUpdate;
    if (callbacks.onQueueEmpty) unified.onQueueEmpty = callbacks.onQueueEmpty;

    return unified;
  }
}

/**
 * 🎯 UNIFIED ADAPTER FACTORY
 * 
 * Central factory for creating unified event callbacks from any legacy system
 */
export class UnifiedEventAdapterFactory {
  
  /**
   * Create unified callbacks from various legacy sources
   * 
   * BACKWARD COMPATIBLE: Supports all existing callback patterns
   */
  static createUnifiedCallbacks(sources: {
    testOptions?: TestOptions;
    eventDrivenQueue?: LegacyEventDrivenQueueCallbacks;
    parallelTestManager?: LegacyParallelTestManagerCallbacks;
    direct?: UnifiedEventCallbacks;
  }): UnifiedEventCallbacks {
    
    let unified: UnifiedEventCallbacks = {};

    // Merge callbacks from all sources (later sources override earlier ones)
    if (sources.testOptions) {
      const testOptionsCallbacks = TestOptionsEventAdapter.adaptTestOptionsCallbacks(sources.testOptions);
      unified = { ...unified, ...testOptionsCallbacks };
    }

    if (sources.eventDrivenQueue) {
      const queueCallbacks = EventDrivenQueueAdapter.adaptEventDrivenQueueCallbacks(sources.eventDrivenQueue);
      unified = { ...unified, ...queueCallbacks };
    }

    if (sources.parallelTestManager) {
      const managerCallbacks = ParallelTestManagerAdapter.adaptParallelTestManagerCallbacks(sources.parallelTestManager);
      unified = { ...unified, ...managerCallbacks };
    }

    if (sources.direct) {
      unified = { ...unified, ...sources.direct };
    }

    return unified;
  }

  /**
   * Create unified emitter with legacy compatibility
   * 
   * This is the main factory method that should be used throughout the codebase
   */
  static createUnifiedEmitter(options: {
    testOptions?: TestOptions;
    verbose?: boolean;
    enableResourceMonitoring?: boolean;
    enableBackpressure?: boolean;
    maxConcurrent?: number;
    maxRetries?: number;
  } = {}): PageAnalysisEmitter {
    
    // Create unified callbacks from test options
    const callbacks = options.testOptions ? 
      TestOptionsEventAdapter.adaptTestOptionsCallbacks(options.testOptions) : 
      {};

    // Create emitter with unified configuration
    const emitter = new PageAnalysisEmitter({
      verbose: options.verbose || options.testOptions?.verbose || false,
      enableResourceMonitoring: options.enableResourceMonitoring ?? true,
      enableBackpressure: options.enableBackpressure ?? true,
      maxConcurrent: options.maxConcurrent || options.testOptions?.maxConcurrent || 3,
      maxRetries: options.maxRetries || options.testOptions?.maxRetries || 3,
      callbacks
    });

    return emitter;
  }
}

/**
 * 🚨 DEPRECATION UTILITIES
 * 
 * Utilities for managing deprecation warnings and migration guides
 */
export class DeprecationManager {
  private static warnedSystems = new Set<string>();

  /**
   * Show deprecation warning once per system per session
   */
  static warnOnce(systemName: string, message: string): void {
    // Check if deprecation warnings should be suppressed
    const suppressWarnings = process.env.NODE_ENV === 'test' || 
                             process.env.CI === 'true' ||
                             process.env.NODE_ENV === 'production' ||
                             process.env.AUDITMYSITE_SUPPRESS_DEPRECATIONS === 'true';
    
    if (!this.warnedSystems.has(systemName) && !suppressWarnings) {
      console.warn(`
🚨 DEPRECATION WARNING: ${systemName}
${message}

📚 Documentation: https://auditmysite.com/docs/v2-migration
🚫  Migration Guide: https://auditmysite.com/docs/unified-events
      `);
      this.warnedSystems.add(systemName);
    }
  }

  /**
   * Get list of systems that have shown deprecation warnings
   */
  static getWarnings(): string[] {
    return Array.from(this.warnedSystems);
  }

  /**
   * Clear warning cache (useful for tests)
   */
  static clearWarnings(): void {
    this.warnedSystems.clear();
  }
}
