import { EventEmitter } from 'events';
import { AdaptiveBackpressureController, BackpressureConfig } from '../backpressure-controller';
import { ResourceMonitor, ResourceMonitorConfig } from '../resource-monitor';
import { QueueState, QueueStateAdapter, QueueStateOptions, ResumeOptions, QueueStateError } from '../../types/queue-state';
import { FileQueueStateAdapter } from '../queue/file-queue-state-adapter';

/**
 * @deprecated EventDrivenQueue is deprecated and will be removed in v3.0.0
 * Use PageAnalysisEmitter instead for unified event handling across all analyzers
 * 
 * Migration Guide:
 * - Replace EventDrivenQueue with PageAnalysisEmitter
 * - Use UnifiedEventCallbacks instead of EventDrivenQueueOptions.eventCallbacks
 * - Access via AccessibilityChecker.getUnifiedEmitter() for advanced usage
 */
export interface QueuedUrl {
  url: string;
  priority: number;
  status: 'pending' | 'in-progress' | 'completed' | 'failed' | 'retrying';
  attempts: number;
  result?: any;
  error?: string;
  startedAt?: Date;
  completedAt?: Date;
  duration?: number;
}

export interface QueueEvent {
  type: 'url-added' | 'url-started' | 'url-completed' | 'url-failed' | 'url-retrying' | 'queue-empty' | 'progress-update' | 'error';
  data: any;
  timestamp: Date;
}

export interface EventDrivenQueueOptions {
  maxRetries?: number;
  maxConcurrent?: number;
  priorityPatterns?: Array<{ pattern: string; priority: number }>;
  retryDelay?: number;
  enableEvents?: boolean;
  enableShortStatus?: boolean;
  statusUpdateInterval?: number;
  
  // Backpressure and resource management
  enableBackpressure?: boolean;
  backpressureConfig?: Partial<BackpressureConfig>;
  enableResourceMonitoring?: boolean;
  resourceMonitorConfig?: Partial<ResourceMonitorConfig>;
  hardTimeout?: number;
  globalTimeout?: number;
  
  // Persistence options
  enablePersistence?: boolean;
  stateAdapter?: QueueStateAdapter;
  autoSave?: boolean;
  autoSaveInterval?: number;
  stateId?: string;
  resumable?: boolean;
  
  eventCallbacks?: {
    onUrlAdded?: (url: string, priority: number) => void;
    onUrlStarted?: (url: string) => void;
    onUrlCompleted?: (url: string, result: any, duration: number) => void;
    onUrlFailed?: (url: string, error: string, attempts: number) => void;
    onUrlRetrying?: (url: string, attempts: number) => void;
    onQueueEmpty?: () => void;
    onProgressUpdate?: (stats: QueueStats) => void;
    onError?: (error: string) => void;
    onShortStatus?: (status: string) => void;
    
    // Enhanced callbacks
    onBackpressureActivated?: (reason: string) => void;
    onBackpressureDeactivated?: () => void;
    onResourceWarning?: (usage: number, limit: number) => void;
    onResourceCritical?: (usage: number, limit: number) => void;
    onGarbageCollection?: (beforeMB: number, afterMB?: number) => void;
  };
}

export interface QueueStats {
  total: number;
  pending: number;
  inProgress: number;
  completed: number;
  failed: number;
  retrying: number;
  progress: number;
  averageDuration: number;
  estimatedTimeRemaining: number;
  activeWorkers: number;
  memoryUsage: number;
  cpuUsage: number;
}

export interface ProcessOptions {
  processor: (url: string) => Promise<any>; // Function that processes a URL
  onResult?: (url: string, result: any) => void;
  onError?: (url: string, error: string) => void;
  onProgress?: (stats: QueueStats) => void;
  onShortStatus?: (status: string) => void;
}

/**
 * @deprecated EventDrivenQueue class is deprecated and will be removed in v3.0.0
 * 
 * Use PageAnalysisEmitter for unified event handling instead.
 * This class is maintained for backward compatibility only.
 * 
 * MIGRATION PATH:
 * ```typescript
 * // OLD (deprecated)
 * const queue = new EventDrivenQueue({ eventCallbacks: { ... } });
 * 
 * // NEW (recommended) 
 * const emitter = new PageAnalysisEmitter({ callbacks: { ... } });
 * // OR via AccessibilityChecker
 * const checker = new AccessibilityChecker({ enableUnifiedEvents: true });
 * checker.setUnifiedEventCallbacks({ ... });
 * ```
 */
export class EventDrivenQueue extends EventEmitter {
  private queue: QueuedUrl[] = [];
  private completed: QueuedUrl[] = [];
  private failed: QueuedUrl[] = [];
  private activeWorkers: Set<string> = new Set();
  private options: EventDrivenQueueOptions;
  private isProcessing = false;
  private startTime: Date | null = null;
  private lastProgressUpdate = 0;
  private progressUpdateInterval = 1000;
  private statusInterval: NodeJS.Timeout | null = null;
  
  // Backpressure and resource management
  private backpressureController?: AdaptiveBackpressureController;
  private resourceMonitor?: ResourceMonitor;
  private hardTimeoutTracker = new Map<string, NodeJS.Timeout>();
  private globalTimeoutTimer: NodeJS.Timeout | null = null;
  
  // Persistence properties
  private stateAdapter?: QueueStateAdapter;
  private autoSaveTimer: NodeJS.Timeout | null = null;
  private stateId: string = '';
  private currentState: QueueState | null = null;

  constructor(options: EventDrivenQueueOptions = {}) {
    super();
    this.options = {
      maxRetries: 3,
      maxConcurrent: 1,
      priorityPatterns: [
        { pattern: '/home', priority: 10 },
        { pattern: '/', priority: 9 },
        { pattern: '/about', priority: 8 },
        { pattern: '/contact', priority: 7 }
      ],
      retryDelay: 1000,
      enableEvents: true,
      enableShortStatus: true,
      statusUpdateInterval: 5000, // Reduced frequency - every 5 seconds instead of 2
      enableBackpressure: false,
      enableResourceMonitoring: false,
      hardTimeout: 30000,
      globalTimeout: 300000,
      enablePersistence: false,
      autoSave: true,
      autoSaveInterval: 10000,
      resumable: true,
      ...options
    };

    this.setupBackpressure();
    this.setupResourceMonitoring();
    this.setupPersistence();

    if (this.options.enableEvents) {
      this.setupEventListeners();
    }
  }

  /**
   * Setup backpressure controller
   */
  private setupBackpressure(): void {
    if (!this.options.enableBackpressure) return;
    
    const backpressureConfig: Partial<BackpressureConfig> = {
      enabled: true,
      maxQueueSize: 500,
      maxMemoryUsageMB: 2048,
      maxCpuUsagePercent: 80,
      minDelayMs: 50,
      maxDelayMs: 5000,
      activationThreshold: 0.8,
      deactivationThreshold: 0.6,
      ...this.options.backpressureConfig
    };
    
    this.backpressureController = new AdaptiveBackpressureController(backpressureConfig);
    
    this.backpressureController.on('backpressureActivated', (data) => {
      this.options.eventCallbacks?.onBackpressureActivated?.('Memory/CPU pressure detected');
    });
    
    this.backpressureController.on('backpressureDeactivated', (data) => {
      this.options.eventCallbacks?.onBackpressureDeactivated?.();
    });
    
    this.backpressureController.on('memoryWarning', (data) => {
      this.options.eventCallbacks?.onResourceWarning?.(data.current, data.threshold);
    });
    
    this.backpressureController.on('memoryCritical', (data) => {
      this.options.eventCallbacks?.onResourceCritical?.(data.current, data.max);
    });
    
    this.backpressureController.on('gcTriggered', (data) => {
      this.options.eventCallbacks?.onGarbageCollection?.(data.beforeMB);
    });
  }
  
  /**
   * Setup persistence functionality
   */
  private setupPersistence(): void {
    if (!this.options.enablePersistence) return;
    
    // Initialize state adapter
    this.stateAdapter = this.options.stateAdapter || new FileQueueStateAdapter();
    
    // Generate unique state ID if not provided
    this.stateId = this.options.stateId || this.generateStateId();
    
    // Setup auto-save if enabled
    if (this.options.autoSave && this.options.autoSaveInterval) {
      this.startAutoSave();
    }
  }
  
  /**
   * Generate unique state ID based on timestamp and random string
   */
  private generateStateId(): string {
    const timestamp = Date.now();
    const random = Math.random().toString(36).substring(2, 8);
    return `queue-${timestamp}-${random}`;
  }
  
  /**
   * Setup resource monitoring
   */
  private setupResourceMonitoring(): void {
    if (!this.options.enableResourceMonitoring) return;
    
    const resourceConfig: Partial<ResourceMonitorConfig> = {
      enabled: true,
      samplingIntervalMs: 2000,
      memoryWarningThresholdMB: 1536,
      memoryCriticalThresholdMB: 2048,
      ...this.options.resourceMonitorConfig
    };
    
    this.resourceMonitor = new ResourceMonitor(resourceConfig);
    
    this.resourceMonitor.on('resourceAlert', (alert) => {
      if (alert.level === 'warning') {
        this.options.eventCallbacks?.onResourceWarning?.(alert.current, alert.threshold);
      } else {
        this.options.eventCallbacks?.onResourceCritical?.(alert.current, alert.threshold);
      }
    });
    
    this.resourceMonitor.on('criticalAlert', (alert) => {
      // Force garbage collection on critical memory alerts
      if (alert.metric === 'rssMemory' || alert.metric === 'heapUsage') {
        const gcSuccess = this.resourceMonitor?.forceGC();
        if (gcSuccess) {
          this.options.eventCallbacks?.onGarbageCollection?.(alert.current);
        }
      }
    });
  }

  private setupEventListeners(): void {
    // Interne Event-Handler
    this.on('url-added', (url: string, priority: number) => {
      this.emit('queue:urlAdded', { url, priority, timestamp: new Date() });
      this.options.eventCallbacks?.onUrlAdded?.(url, priority);
    });

    this.on('url-started', (url: string) => {
      this.emit('queue:urlStarted', { url, timestamp: new Date() });
      this.options.eventCallbacks?.onUrlStarted?.(url);
    });

    this.on('url-completed', (url: string, result: any, duration: number) => {
      this.emit('queue:urlCompleted', { url, result, duration, timestamp: new Date() });
      this.options.eventCallbacks?.onUrlCompleted?.(url, result, duration);
    });

    this.on('url-failed', (url: string, error: string, attempts: number) => {
      this.emit('queue:urlFailed', { url, error, attempts, timestamp: new Date() });
      this.options.eventCallbacks?.onUrlFailed?.(url, error, attempts);
    });

    this.on('url-retrying', (url: string, attempts: number) => {
      this.emit('queue:urlRetrying', { url, attempts, timestamp: new Date() });
      this.options.eventCallbacks?.onUrlRetrying?.(url, attempts);
    });

    this.on('queue-empty', () => {
      this.emit('queue:empty', { timestamp: new Date() });
      this.options.eventCallbacks?.onQueueEmpty?.();
    });

    this.on('progress-update', (stats: QueueStats) => {
      this.emit('queue:progressUpdate', { stats, timestamp: new Date() });
      this.options.eventCallbacks?.onProgressUpdate?.(stats);
    });

    this.on('error', (error: string) => {
      this.emit('queue:error', { error, timestamp: new Date() });
      this.options.eventCallbacks?.onError?.(error);
    });
  }

  addUrls(urls: string[]): void {
    const newUrls = urls.filter(url => !this.queue.some(q => q.url === url));
    
    newUrls.forEach(url => {
      const priority = this.calculatePriority(url);
      const queuedUrl: QueuedUrl = {
        url,
        priority,
        status: 'pending',
        attempts: 0
      };
      
      this.queue.push(queuedUrl);
      this.emit('url-added', url, priority);
    });

    // Sort by priority (highest first)
    this.queue.sort((a, b) => b.priority - a.priority);
    
    this.updateProgress();
  }

  async getNextUrl(): Promise<QueuedUrl | null> {
    // Check backpressure delay
    const delay = this.backpressureController?.getCurrentDelay() || 0;
    if (delay > 0) {
      await new Promise(resolve => setTimeout(resolve, delay));
    }
    
    const pendingUrl = this.queue.find(q => q.status === 'pending');
    
    if (pendingUrl && this.activeWorkers.size < this.options.maxConcurrent!) {
      pendingUrl.status = 'in-progress';
      pendingUrl.startedAt = new Date();
      pendingUrl.attempts++;
      
      this.activeWorkers.add(pendingUrl.url);
      
      // Set hard timeout for this item
      if (this.options.hardTimeout) {
        const timeoutId = setTimeout(() => {
          this.markFailed(pendingUrl.url, `Hard timeout after ${this.options.hardTimeout}ms`);
        }, this.options.hardTimeout);
        
        this.hardTimeoutTracker.set(pendingUrl.url, timeoutId);
      }
      
      this.emit('url-started', pendingUrl.url);
      this.updateProgress();
      this.updateBackpressure();
      
      return pendingUrl;
    }
    
    return null;
  }

  markCompleted(url: string, result: any): void {
    const queuedUrl = this.queue.find(q => q.url === url);
    if (queuedUrl) {
      queuedUrl.status = 'completed';
      queuedUrl.result = result;
      queuedUrl.completedAt = new Date();
      queuedUrl.duration = queuedUrl.completedAt.getTime() - queuedUrl.startedAt!.getTime();
      
      this.completed.push(queuedUrl);
      this.queue = this.queue.filter(q => q.url !== url);
      this.activeWorkers.delete(url);
      
      // Clear hard timeout
      const timeoutId = this.hardTimeoutTracker.get(url);
      if (timeoutId) {
        clearTimeout(timeoutId);
        this.hardTimeoutTracker.delete(url);
      }
      
      this.emit('url-completed', url, result, queuedUrl.duration);
      this.updateProgress();
      this.updateBackpressure(false); // Success, no error
      
      this.checkQueueEmpty();
    }
  }

  markFailed(url: string, error: string): void {
    const queuedUrl = this.queue.find(q => q.url === url);
    if (queuedUrl) {
      // Clear hard timeout
      const timeoutId = this.hardTimeoutTracker.get(url);
      if (timeoutId) {
        clearTimeout(timeoutId);
        this.hardTimeoutTracker.delete(url);
      }
      
      if (queuedUrl.attempts < this.options.maxRetries!) {
        // Retry logic with backpressure consideration
        queuedUrl.status = 'retrying';
        this.emit('url-retrying', url, queuedUrl.attempts);
        
        // Calculate retry delay with backpressure
        const baseDelay = this.options.retryDelay || 1000;
        const backpressureDelay = this.backpressureController?.getCurrentDelay() || 0;
        const totalDelay = baseDelay + backpressureDelay;
        
        // Schedule retry
        setTimeout(() => {
          if (this.queue.find(q => q.url === url)) { // Still in queue
            queuedUrl.status = 'pending';
            this.updateProgress();
          }
        }, totalDelay);
      } else {
        // Max retries reached
        queuedUrl.status = 'failed';
        queuedUrl.error = error;
        queuedUrl.completedAt = new Date();
        queuedUrl.duration = queuedUrl.completedAt.getTime() - queuedUrl.startedAt!.getTime();
        
        this.failed.push(queuedUrl);
        this.queue = this.queue.filter(q => q.url !== url);
        this.activeWorkers.delete(url);
        
        this.emit('url-failed', url, error, queuedUrl.attempts);
        this.updateProgress();
        this.updateBackpressure(true); // Error occurred
        
        this.checkQueueEmpty();
      }
    }
  }

  /**
   * Update backpressure controller with current state
   */
  private updateBackpressure(hasError: boolean = false): void {
    if (!this.backpressureController) return;
    
    this.backpressureController.updateQueueState(
      this.queue.length,
      this.activeWorkers.size,
      hasError
    );
  }

  private checkQueueEmpty(): void {
    const pendingUrls = this.queue.filter(q => q.status === 'pending' || q.status === 'retrying');
    console.log(`🔍 checkQueueEmpty: pending=${pendingUrls.length}, activeWorkers=${this.activeWorkers.size}, totalQueue=${this.queue.length}`);
    
    if (pendingUrls.length === 0 && this.activeWorkers.size === 0) {
      console.log('📝 Queue is empty - processing complete');
      this.isProcessing = false;
      
      // Stop monitoring
      this.resourceMonitor?.stop();
      
      // Clear any remaining timeouts
      for (const timeoutId of this.hardTimeoutTracker.values()) {
        clearTimeout(timeoutId);
      }
      this.hardTimeoutTracker.clear();
      
      if (this.globalTimeoutTimer) {
        clearTimeout(this.globalTimeoutTimer);
        this.globalTimeoutTimer = null;
      }
      
      this.emit('queue-empty');
    }
  }

  private updateProgress(): void {
    const now = Date.now();
    if (now - this.lastProgressUpdate < this.progressUpdateInterval) {
      return;
    }
    
    this.lastProgressUpdate = now;
    const stats = this.getStats();
    this.emit('progress-update', stats);
  }

  getStats(): QueueStats {
    // Get unique URLs from queue, completed and failed to avoid double-counting
    const allUrls = new Set<string>([...this.queue.map(q => q.url), ...this.completed.map(q => q.url), ...this.failed.map(q => q.url)]);
    const total = allUrls.size;
    const pending = this.queue.filter(q => q.status === 'pending').length;
    const inProgress = this.queue.filter(q => q.status === 'in-progress').length;
    const completed = this.completed.length;
    const failed = this.failed.length;
    const retrying = this.queue.filter(q => q.status === 'retrying').length;
    const progress = total > 0 ? ((completed + failed) / total) * 100 : 0;
    
    // Debug progress calculation
    console.log(`📊 Queue stats: total=${total}, completed=${completed}, failed=${failed}, progress=${progress.toFixed(2)}%`);
    
    // Calculate average duration
    const completedWithDuration = this.completed.filter(q => q.duration);
    const averageDuration = completedWithDuration.length > 0 
      ? completedWithDuration.reduce((sum, q) => sum + q.duration!, 0) / completedWithDuration.length 
      : 0;
    
    // Estimate remaining time
    const remainingItems = pending + inProgress + retrying;
    const estimatedTimeRemaining = remainingItems > 0 && averageDuration > 0
      ? (remainingItems * averageDuration) / this.options.maxConcurrent!
      : 0;
    
    // System metrics (simplified)
    const memoryUsage = process.memoryUsage().heapUsed / 1024 / 1024; // MB
    const cpuUsage = process.cpuUsage().user / 1000000; // seconds
    
    return {
      total,
      pending,
      inProgress,
      completed,
      failed,
      retrying,
      progress: Math.round(progress * 100) / 100,
      averageDuration: Math.round(averageDuration),
      estimatedTimeRemaining: Math.round(estimatedTimeRemaining),
      activeWorkers: this.activeWorkers.size,
      memoryUsage: Math.round(memoryUsage * 100) / 100,
      cpuUsage: Math.round(cpuUsage * 100) / 100
    };
  }

  private calculatePriority(url: string): number {
    const pattern = this.options.priorityPatterns!.find(p => url.includes(p.pattern));
    return pattern ? pattern.priority : 1;
  }

  /**
   * Start auto-save timer
   */
  private startAutoSave(): void {
    if (!this.stateAdapter || !this.options.autoSaveInterval) return;
    
    this.autoSaveTimer = setInterval(async () => {
      try {
        await this.saveState();
      } catch (error) {
        console.warn('Failed to auto-save queue state:', error instanceof Error ? error.message : String(error));
      }
    }, this.options.autoSaveInterval);
  }
  
  /**
   * Stop auto-save timer
   */
  private stopAutoSave(): void {
    if (this.autoSaveTimer) {
      clearInterval(this.autoSaveTimer);
      this.autoSaveTimer = null;
    }
  }
  
  /**
   * Create current queue state snapshot
   */
  private createStateSnapshot(): QueueState {
    const allUrls = [...this.queue, ...this.completed, ...this.failed];
    
    return {
      id: this.stateId,
      urls: allUrls.map(q => q.url),
      processedUrls: this.completed.map(q => q.url),
      failedUrls: this.failed.map(q => q.url),
      currentIndex: this.completed.length + this.failed.length,
      totalUrls: allUrls.length,
      results: this.completed.map(q => q.result),
      startTime: this.startTime?.getTime() || Date.now(),
      lastUpdateTime: Date.now(),
      options: {
        concurrency: this.options.maxConcurrent || 1,
        retryLimit: this.options.maxRetries || 3,
        ...this.options
      },
      status: this.isProcessing ? 'processing' : 
              (allUrls.length === this.completed.length + this.failed.length ? 'completed' : 'paused'),
      metadata: {
        version: '1.0.0',
        ...this.options
      }
    };
  }
  
  /**
   * Save current state to adapter
   */
  async saveState(): Promise<void> {
    if (!this.stateAdapter || !this.options.enablePersistence) return;
    
    try {
      this.currentState = this.createStateSnapshot();
      await this.stateAdapter.save(this.currentState);
    } catch (error) {
      throw new QueueStateError('Failed to save queue state', error instanceof Error ? error : new Error(String(error)));
    }
  }
  
  /**
   * Load state from adapter and resume processing
   */
  async resumeFromState(options?: ResumeOptions): Promise<void> {
    if (!this.stateAdapter) {
      this.stateAdapter = options?.adapter || new FileQueueStateAdapter();
    }
    
    const stateId = options?.stateId || this.stateId;
    
    try {
      const state = await this.stateAdapter.load(stateId);
      if (!state) {
        throw new QueueStateError(`Queue state not found: ${stateId}`);
      }
      
      // Restore queue state
      this.stateId = state.id;
      this.currentState = state;
      this.startTime = new Date(state.startTime);
      
      // Rebuild queue from state
      const remainingUrls = options?.skipCompleted ? 
        state.urls.filter(url => !state.processedUrls.includes(url) && !state.failedUrls.includes(url)) :
        state.urls;
      
      this.queue = remainingUrls.map(url => ({
        url,
        priority: this.calculatePriority(url),
        status: 'pending' as const,
        attempts: 0
      }));
      
      // Restore completed results
      this.completed = state.processedUrls.map((url, index) => ({
        url,
        priority: this.calculatePriority(url),
        status: 'completed' as const,
        attempts: 1,
        result: state.results[index],
        startedAt: new Date(state.startTime),
        completedAt: new Date(state.lastUpdateTime),
        duration: 1000 // Approximate
      }));
      
      // Restore failed results
      this.failed = state.failedUrls.map(url => ({
        url,
        priority: this.calculatePriority(url),
        status: 'failed' as const,
        attempts: this.options.maxRetries || 3,
        error: 'Previously failed',
        startedAt: new Date(state.startTime),
        completedAt: new Date(state.lastUpdateTime),
        duration: 1000 // Approximate
      }));
      
      // Update options from state if needed
      this.options.maxConcurrent = state.options.concurrency;
      this.options.maxRetries = state.options.retryLimit;
      
      console.log(`✅ Resumed queue state: ${this.completed.length} completed, ${this.queue.length} remaining`);
      
    } catch (error) {
      throw new QueueStateError(`Failed to resume from state: ${error instanceof Error ? error.message : String(error)}`, error instanceof Error ? error : new Error(String(error)));
    }
  }
  
  /**
   * Delete saved state
   */
  async deleteState(stateId?: string): Promise<void> {
    if (!this.stateAdapter) return;
    
    try {
      await this.stateAdapter.delete(stateId || this.stateId);
    } catch (error) {
      throw new QueueStateError(`Failed to delete state: ${error instanceof Error ? error.message : String(error)}`, error instanceof Error ? error : new Error(String(error)));
    }
  }
  
  /**
   * List all available saved states
   */
  async listSavedStates(): Promise<string[]> {
    if (!this.stateAdapter) return [];
    
    try {
      return await this.stateAdapter.list();
    } catch (error) {
      console.warn('Failed to list saved states:', error instanceof Error ? error.message : String(error));
      return [];
    }
  }
  
  /**
   * Get state ID
   */
  getStateId(): string {
    return this.stateId;
  }
  
  /**
   * Check if persistence is enabled
   */
  isPersistenceEnabled(): boolean {
    return this.options.enablePersistence === true;
  }
  
  /**
   * 🚀 Integrated parallel queue processing
   * Processes all URLs in parallel with automatic status reporting
   */
  async processUrls(urls: string[], options: ProcessOptions): Promise<any[]> {
    console.log(`🚀 Starting queue processing for ${urls.length} URLs with ${this.options.maxConcurrent} workers`);
    
    this.addUrls(urls);
    this.startTime = new Date();
    this.isProcessing = true; // Fix: Enable processing so workers don't exit early
    
    // Save initial state if persistence is enabled
    if (this.options.enablePersistence) {
      try {
        await this.saveState();
        console.log(`💾 Initial state saved with ID: ${this.stateId}`);
      } catch (error) {
        console.warn('Failed to save initial state:', error instanceof Error ? error.message : String(error));
      }
    }
    
    // Starte Status-Updates
    if (this.options.enableShortStatus) {
      this.startStatusUpdates(options.onShortStatus);
    }

    const results: any[] = [];
    const promises: Promise<void>[] = [];

    // Erstelle Worker-Promises
    for (let i = 0; i < this.options.maxConcurrent!; i++) {
      promises.push(this.worker(i, options));
    }

    // Warte auf alle Worker
    await Promise.all(promises);

    // Stoppe Status-Updates
    this.stopStatusUpdates();

    // Collect all results
    results.push(...this.completed.map(q => q.result));
    
    // Save final state if persistence is enabled
    if (this.options.enablePersistence) {
      try {
        await this.saveState();
        console.log(`💾 Final state saved with ID: ${this.stateId}`);
      } catch (error) {
        console.warn('Failed to save final state:', error instanceof Error ? error.message : String(error));
      }
    }
    
    const duration = Date.now() - this.startTime!.getTime();
    console.log(`✅ Queue processing completed: ${this.completed.length}/${urls.length} URLs in ${duration}ms`);
    
    return results;
  }

  /**
   * 🔧 Worker function for parallel processing (optimized to prevent timeout accumulation)
   */
  private async worker(workerId: number, options: ProcessOptions): Promise<void> {
    let consecutiveIdleAttempts = 0;
    const maxIdleAttempts = 20; // Exit after 2 seconds of idle (20 * 100ms)
    
    console.log(`🚀 Worker ${workerId} starting`);
    
    while (this.isProcessing) {
      const pendingUrls = this.queue.filter(q => q.status === 'pending' || q.status === 'retrying');
      
      // Exit if no pending work remains
      if (pendingUrls.length === 0) {
        consecutiveIdleAttempts++;
        
        // Quick exit if no work is available
        if (consecutiveIdleAttempts > maxIdleAttempts) {
          console.log(`😴 Worker ${workerId} - no work available, exiting`);
          break;
        }
        
        // Short wait before checking again
        await new Promise(resolve => setTimeout(resolve, 100));
        continue;
      }
      
      const queuedUrl = await this.getNextUrl();
      
      if (!queuedUrl) {
        consecutiveIdleAttempts++;
        
        if (consecutiveIdleAttempts > maxIdleAttempts) {
          console.log(`😴 Worker ${workerId} idle timeout - exiting`);
          break;
        }
        
        // Short wait before checking again
        await new Promise(resolve => setTimeout(resolve, 100));
        continue;
      }
      
      // Reset idle counter when work is found
      consecutiveIdleAttempts = 0;

      try {
        const result = await options.processor(queuedUrl.url);
        this.markCompleted(queuedUrl.url, result);
        options.onResult?.(queuedUrl.url, result);
      } catch (error) {
        this.markFailed(queuedUrl.url, String(error));
        options.onError?.(queuedUrl.url, String(error));
      }
    }
    
    console.log(`🏁 Worker ${workerId} finished`);
  }

  /**
   * 📊 Starts short status updates
   */
  private startStatusUpdates(onShortStatus?: (status: string) => void): void {
    this.statusInterval = setInterval(() => {
      const stats = this.getStats();
      const status = this.generateShortStatus(stats);
      
      this.emit('short-status', status);
      onShortStatus?.(status);
      this.options.eventCallbacks?.onShortStatus?.(status);
    }, this.options.statusUpdateInterval);
  }

  /**
   * ⏹️ Stops status updates
   */
  private stopStatusUpdates(): void {
    if (this.statusInterval) {
      clearInterval(this.statusInterval);
      this.statusInterval = null;
    }
  }

  /**
   * 📝 Generates short status message with improved ETA
   */
  private generateShortStatus(stats: QueueStats): string {
    const progress = Math.round(stats.progress);
    const progressBar = this.createProgressBar(stats.progress, 20);
    
    // Improved ETA calculation
    let eta = '';
    if (stats.estimatedTimeRemaining > 0) {
      const seconds = Math.round(stats.estimatedTimeRemaining / 1000);
      if (seconds < 60) {
        eta = `ETA: ${seconds}s | `;
      } else if (seconds < 3600) {
        eta = `ETA: ${Math.round(seconds / 60)}m | `;
      } else {
        eta = `ETA: ${Math.round(seconds / 3600)}h | `;
      }
    }
    
    // Speed indicator (URLs per minute)
    const speed = stats.completed > 0 && this.startTime 
      ? Math.round((stats.completed / ((Date.now() - this.startTime.getTime()) / 60000)) * 10) / 10
      : 0;
    
    const elapsed = this.startTime ? Math.round((Date.now() - this.startTime.getTime()) / 1000) : 0;
    
    // Only return the status string - don't print here to avoid spam
    return `🚀 Testing pages... ${progressBar} ${progress}% (${stats.completed}/${stats.total})\n   ${eta}Speed: ${speed.toFixed(1)} pages/min | Elapsed: ${elapsed}s`;
  }
  
  /**
   * Create a simple progress bar
   */
  private createProgressBar(percentage: number, length: number = 20): string {
    const filled = Math.round((percentage / 100) * length);
    return '█'.repeat(filled) + '░'.repeat(length - filled);
  }

  // Public API für Event-Listener
  onUrlAdded(callback: (event: QueueEvent) => void): this {
    this.on('queue:urlAdded', callback);
    return this;
  }

  onUrlStarted(callback: (event: QueueEvent) => void): this {
    this.on('queue:urlStarted', callback);
    return this;
  }

  onUrlCompleted(callback: (event: QueueEvent) => void): this {
    this.on('queue:urlCompleted', callback);
    return this;
  }

  onUrlFailed(callback: (event: QueueEvent) => void): this {
    this.on('queue:urlFailed', callback);
    return this;
  }

  onUrlRetrying(callback: (event: QueueEvent) => void): this {
    this.on('queue:urlRetrying', callback);
    return this;
  }

  onQueueEmpty(callback: (event: QueueEvent) => void): this {
    this.on('queue:empty', callback);
    return this;
  }

  onProgressUpdate(callback: (event: QueueEvent) => void): this {
    this.on('queue:progressUpdate', callback);
    return this;
  }

  onError(callback: (event: QueueEvent) => void): this {
    this.on('queue:error', callback);
    return this;
  }
  
  /**
   * Cleanup resources and stop processing
   */
  destroy(): void {
    this.isProcessing = false;
    
    // Stop monitoring
    this.resourceMonitor?.stop();
    this.resourceMonitor?.destroy();
    this.backpressureController?.destroy();
    
    // Stop persistence
    this.stopAutoSave();
    
    // Clear timeouts
    for (const timeoutId of this.hardTimeoutTracker.values()) {
      clearTimeout(timeoutId);
    }
    this.hardTimeoutTracker.clear();
    
    if (this.globalTimeoutTimer) {
      clearTimeout(this.globalTimeoutTimer);
      this.globalTimeoutTimer = null;
    }
    
    this.stopStatusUpdates();
    
    // Clear active workers
    this.activeWorkers.clear();
    
    // Remove all listeners
    this.removeAllListeners();
  }
  
  /**
   * Force garbage collection if available
   */
  forceGarbageCollection(): boolean {
    const beforeMB = process.memoryUsage().heapUsed / 1024 / 1024;
    
    if (this.resourceMonitor?.forceGC()) {
      const afterMB = process.memoryUsage().heapUsed / 1024 / 1024;
      this.options.eventCallbacks?.onGarbageCollection?.(beforeMB, afterMB);
      return true;
    }
    
    return this.backpressureController?.triggerGarbageCollection() || false;
  }
  
  // Utility-Methoden
  getCompletedResults(): any[] {
    return this.completed.map(q => q.result);
  }

  getFailedResults(): any[] {
    return this.failed.map(q => ({ url: q.url, error: q.error, attempts: q.attempts }));
  }

  clear(): void {
    this.queue = [];
    this.completed = [];
    this.failed = [];
    this.activeWorkers.clear();
    this.isProcessing = false;
    this.startTime = null;
  }

  pause(): void {
    this.isProcessing = false;
  }

  resume(): void {
    this.isProcessing = true;
  }

  isPaused(): boolean {
    return !this.isProcessing;
  }

  /**
   * 🧯 Cleanup method to prevent memory leaks
   */
  cleanup(): void {
    // Stop any active status updates
    this.stopStatusUpdates();
    
    // Stop persistence
    this.stopAutoSave();
    
    // Clear all arrays and collections
    this.queue = [];
    this.completed = [];
    this.failed = [];
    this.activeWorkers.clear();
    
    // Reset state
    this.isProcessing = false;
    this.startTime = null;
    this.lastProgressUpdate = 0;
    this.currentState = null;
    
    // Remove all event listeners to prevent memory leaks
    this.removeAllListeners();
    
    console.log('🧯 EventDrivenQueue cleaned up - memory leaks prevented');
  }

  getQueueSize(): number {
    return this.queue.length;
  }

  getActiveWorkers(): number {
    return this.activeWorkers.size;
  }

  getMaxConcurrent(): number {
    return this.options.maxConcurrent!;
  }

  setMaxConcurrent(max: number): void {
    this.options.maxConcurrent = max;
  }
}
