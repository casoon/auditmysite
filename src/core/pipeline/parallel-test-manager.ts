import { EventDrivenQueue, EventDrivenQueueOptions, QueueStats } from './event-driven-queue';
import { AccessibilityChecker } from '@core/accessibility';
import { TestOptions, AccessibilityResult } from '../types';
import { log } from '@core/logging';

/**
 * @deprecated ParallelTestManagerOptions is deprecated and will be removed in v3.0.0
 * Use AccessibilityChecker with unified event system instead
 */
export interface ParallelTestManagerOptions extends EventDrivenQueueOptions {
  // Queue-specific options
  maxConcurrent?: number;
  maxRetries?: number;
  retryDelay?: number;
  
  // Test-specific options
  testOptions?: TestOptions;
  
  // AccessibilityChecker instance (for comprehensive analysis support)
  accessibilityChecker?: AccessibilityChecker;
  
  // Logging options  
  verbose?: boolean; // when true, emit detailed logs; default false keeps CLI quiet
  
  // Resource Management
  enableResourceMonitoring?: boolean;
  maxMemoryUsage?: number; // MB
  maxCpuUsage?: number; // Percent
  
  // Persistence options (inherited from EventDrivenQueueOptions)
  enablePersistence?: boolean;
  stateAdapter?: any;
  autoSave?: boolean;
  autoSaveInterval?: number;
  stateId?: string;
  resumable?: boolean;
  
  // Event Callbacks
  onTestStart?: (url: string) => void;
  onTestComplete?: (url: string, result: AccessibilityResult) => void;
  onTestError?: (url: string, error: string) => void;
  onProgressUpdate?: (stats: QueueStats) => void;
  onQueueEmpty?: () => void;
}

export interface ParallelTestResult {
  results: AccessibilityResult[];
  stats: QueueStats;
  duration: number;
  errors: Array<{ url: string; error: string; attempts: number }>;
}

/**
 * @deprecated ParallelTestManager is deprecated and will be removed in v3.0.0
 * 
 * This class is replaced by the unified PageAnalysisEmitter system integrated into AccessibilityChecker.
 * The new system provides better performance, consistent event handling, and integrated resource monitoring.
 * 
 * MIGRATION GUIDE:
 * ```typescript
 * // OLD (deprecated)
 * const manager = new ParallelTestManager({ 
 *   maxConcurrent: 3,
 *   onTestComplete: (url, result) => { ... }
 * });
 * await manager.runTests(urls);
 * 
 * // NEW (recommended)
 * const checker = new AccessibilityChecker({ 
 *   enableUnifiedEvents: true,
 *   enableComprehensiveAnalysis: true 
 * });
 * checker.setUnifiedEventCallbacks({ onUrlCompleted: (url, result) => { ... } });
 * await checker.testMultiplePagesParallel(urls, { maxConcurrent: 3 });
 * ```
 */
export class ParallelTestManager {
  private queue: EventDrivenQueue;
  private accessibilityChecker: AccessibilityChecker;
  private options: ParallelTestManagerOptions;
  private isRunning = false;
  private startTime: Date | null = null;
  private activeTests: Map<string, Promise<AccessibilityResult>> = new Map();

  constructor(options: ParallelTestManagerOptions = {}) {
    this.options = {
      maxConcurrent: 3,
      maxRetries: 3,
      retryDelay: 1000,
      // default to quiet unless explicitly verbose
      verbose: false,
      enableResourceMonitoring: true,
      maxMemoryUsage: 512, // 512 MB
      maxCpuUsage: 80, // 80%
      ...options
    };
    
    // Configure logger
    log.setVerbose(this.options.verbose || false);

    // Initialize Event-Driven Queue with persistence support
    this.queue = new EventDrivenQueue({
      maxRetries: this.options.maxRetries,
      maxConcurrent: this.options.maxConcurrent,
      retryDelay: this.options.retryDelay,
      enableEvents: true,
      // Pass persistence options through
      enablePersistence: this.options.enablePersistence,
      stateAdapter: this.options.stateAdapter,
      autoSave: this.options.autoSave,
      autoSaveInterval: this.options.autoSaveInterval,
      stateId: this.options.stateId,
      resumable: this.options.resumable,
      eventCallbacks: {
        onUrlAdded: this.handleUrlAdded.bind(this),
        onUrlStarted: this.handleUrlStarted.bind(this),
        onUrlCompleted: this.handleUrlCompleted.bind(this),
        onUrlFailed: this.handleUrlFailed.bind(this),
        onUrlRetrying: this.handleUrlRetrying.bind(this),
        onQueueEmpty: this.handleQueueEmpty.bind(this),
        onProgressUpdate: this.handleProgressUpdate.bind(this),
        onError: this.handleError.bind(this)
      }
    });

    // Initialize Accessibility Checker - use provided instance or create new one
    this.accessibilityChecker = options.accessibilityChecker || new AccessibilityChecker();
  }

  async initialize(): Promise<void> {
    await this.accessibilityChecker.initialize();
    log.debug(`Parallel Test Manager initialized with ${this.options.maxConcurrent} concurrent workers`);
  }
  
  /**
   * Resume tests from saved state
   */
  async resumeFromState(stateId?: string): Promise<void> {
    if (!this.queue.isPersistenceEnabled()) {
      throw new Error('Persistence is not enabled, cannot resume from state');
    }
    
    try {
      await this.queue.resumeFromState({ 
        stateId: stateId || this.options.stateId!,
        skipCompleted: true 
      });
      log.success(`Resumed from saved state: ${stateId || this.options.stateId}`);
    } catch (error) {
      throw new Error(`Failed to resume from state: ${error instanceof Error ? error.message : String(error)}`);
    }
  }
  
  /**
   * Get the current state ID
   */
  getStateId(): string {
    return this.queue.getStateId();
  }
  
  /**
   * Save current state
   */
  async saveState(): Promise<void> {
    if (this.queue.isPersistenceEnabled()) {
      await this.queue.saveState();
    }
  }

  async runTests(urls: string[]): Promise<ParallelTestResult> {
    if (this.isRunning) {
      throw new Error('Test manager is already running');
    }

    this.isRunning = true;
    this.startTime = new Date();
    this.activeTests.clear();

    log.info(`Analyzing ${urls.length} pages with ${this.options.maxConcurrent} parallel workers`);
    log.startProgress(urls.length, 'accessibility analysis');

    // Add URLs to queue
    this.queue.addUrls(urls);

    // Event listeners for queue events
    this.setupEventListeners();

    // Start parallel test execution
    await this.processQueue();

    const duration = this.startTime ? Date.now() - this.startTime.getTime() : 0;
    const stats = this.queue.getStats();
    const results = this.queue.getCompletedResults();
    const errors = this.queue.getFailedResults();

    this.isRunning = false;

    return {
      results,
      stats,
      duration,
      errors
    };
  }

  private async processQueue(): Promise<void> {
    return new Promise((resolve, reject) => {
      // Event-Listener für Queue-Ende
      this.queue.onQueueEmpty(() => {
        resolve();
      });

      // Event-Listener für Fehler
      this.queue.onError((event) => {
        reject(new Error(event.data.error));
      });

      // Starte Worker-Prozesse
      this.startWorkers();
    });
  }

  private startWorkers(): void {
    // Starte initiale Worker bis zur maxConcurrent-Grenze
    for (let i = 0; i < this.options.maxConcurrent!; i++) {
      this.processNextUrl();
    }
  }

  private async processNextUrl(): Promise<void> {
    if (!this.isRunning) return;

    const queuedUrl = await this.queue.getNextUrl();
    if (!queuedUrl) return;

    try {
      // Resource-Monitoring
      if (this.options.enableResourceMonitoring) {
        this.checkResourceLimits();
      }

      // Test ausführen
      const testPromise = this.accessibilityChecker.testPage(queuedUrl.url, this.options.testOptions);
      this.activeTests.set(queuedUrl.url, testPromise);

      const result = await testPromise;
      this.queue.markCompleted(queuedUrl.url, result);

    } catch (error) {
      this.queue.markFailed(queuedUrl.url, String(error));
    } finally {
      this.activeTests.delete(queuedUrl.url);
      
      // Starte nächsten Worker
      this.processNextUrl();
    }
  }

  private checkResourceLimits(): void {
    const memoryUsage = process.memoryUsage().heapUsed / 1024 / 1024; // MB
    const cpuUsage = process.cpuUsage().user / 1000000; // Sekunden

    if (memoryUsage > this.options.maxMemoryUsage!) {
      log.warn(`High memory usage: ${memoryUsage.toFixed(2)} MB`);
      // Optional: Queue pausieren oder Worker reduzieren
    }

    if (cpuUsage > this.options.maxCpuUsage!) {
      log.warn(`High CPU usage: ${cpuUsage.toFixed(2)}s`);
      // Optional: Queue pausieren oder Worker reduzieren
    }
  }

  // Event Handler - Reduced logging for cleaner output
  private handleUrlAdded(url: string, priority: number): void {
    // Silent - no logging needed
  }

  private handleUrlStarted(url: string): void {
    this.options.onTestStart?.(url);
  }

  private handleUrlCompleted(url: string, result: AccessibilityResult, duration: number): void {
    this.options.onTestComplete?.(url, result);
  }

  private handleUrlFailed(url: string, error: string, attempts: number): void {
    this.options.onTestError?.(url, error);
  }

  private handleUrlRetrying(url: string, attempts: number): void {
    // Silent retry - no logging needed for cleaner output
  }

  private handleQueueEmpty(): void {
    log.completeProgress();
    this.options.onQueueEmpty?.();
  }

  private handleProgressUpdate(stats: QueueStats): void {
    log.updateProgress(stats.completed, stats.failed);
    this.options.onProgressUpdate?.(stats);
  }

  private handleError(error: string): void {
    log.error(`Queue error: ${error}`);
  }

  private setupEventListeners(): void {
    // Progress-Update-Interval
    setInterval(() => {
      const stats = this.queue.getStats();
      this.handleProgressUpdate(stats);
    }, 5000); // Update every 5 seconds instead of every second
  }

  // Public API
  pause(): void {
    this.queue.pause();
    log.info('Tests paused');
  }

  resume(): void {
    this.queue.resume();
    log.info('Tests resumed');
    this.startWorkers();
  }

  stop(): void {
    this.isRunning = false;
    this.queue.clear();
    log.info('Tests stopped');
  }

  getStats(): QueueStats {
    return this.queue.getStats();
  }

  getActiveTests(): number {
    return this.activeTests.size;
  }

  getQueueSize(): number {
    return this.queue.getQueueSize();
  }

  setMaxConcurrent(max: number): void {
    this.queue.setMaxConcurrent(max);
    log.debug(`Max concurrent workers set to ${max}`);
  }

  // Resource Management
  getMemoryUsage(): number {
    return process.memoryUsage().heapUsed / 1024 / 1024;
  }

  getCpuUsage(): number {
    return process.cpuUsage().user / 1000000;
  }

  // Cleanup
  async cleanup(): Promise<void> {
    this.stop();
    await this.accessibilityChecker.cleanup();
  }
} 