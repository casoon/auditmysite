import { EventDrivenQueue, EventDrivenQueueOptions, QueueStats } from './event-driven-queue';
import { AccessibilityChecker } from '@core/accessibility';
import { TestOptions, AccessibilityResult } from '../types';

export interface ParallelTestManagerOptions extends EventDrivenQueueOptions {
  // Queue-specific options
  maxConcurrent?: number;
  maxRetries?: number;
  retryDelay?: number;
  
  // Test-specific options
  testOptions?: TestOptions;
  
  // Progress Reporting
  enableProgressBar?: boolean;
  progressUpdateInterval?: number;
  
  // Resource Management
  enableResourceMonitoring?: boolean;
  maxMemoryUsage?: number; // MB
  maxCpuUsage?: number; // Percent
  
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
      enableProgressBar: true,
      progressUpdateInterval: 1000,
      enableResourceMonitoring: true,
      maxMemoryUsage: 512, // 512 MB
      maxCpuUsage: 80, // 80%
      ...options
    };

    // Initialize Event-Driven Queue
    this.queue = new EventDrivenQueue({
      maxRetries: this.options.maxRetries,
      maxConcurrent: this.options.maxConcurrent,
      retryDelay: this.options.retryDelay,
      enableEvents: true,
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

    // Initialize Accessibility Checker
    this.accessibilityChecker = new AccessibilityChecker();
  }

  async initialize(): Promise<void> {
    await this.accessibilityChecker.initialize();
    console.log(`🚀 Parallel Test Manager initialized with ${this.options.maxConcurrent} concurrent workers`);
  }

  async runTests(urls: string[]): Promise<ParallelTestResult> {
    if (this.isRunning) {
      throw new Error('Test manager is already running');
    }

    this.isRunning = true;
    this.startTime = new Date();
    this.activeTests.clear();

    console.log(`🧪 Starting parallel tests for ${urls.length} URLs with ${this.options.maxConcurrent} workers`);

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
      console.warn(`⚠️  High memory usage: ${memoryUsage.toFixed(2)} MB`);
      // Optional: Queue pausieren oder Worker reduzieren
    }

    if (cpuUsage > this.options.maxCpuUsage!) {
      console.warn(`⚠️  High CPU usage: ${cpuUsage.toFixed(2)}s`);
      // Optional: Queue pausieren oder Worker reduzieren
    }
  }

  // Event Handler
  private handleUrlAdded(url: string, priority: number): void {
    console.log(`📋 Added URL to queue: ${url} (priority: ${priority})`);
  }

  private handleUrlStarted(url: string): void {
    console.log(`🚀 Started testing: ${url}`);
    this.options.onTestStart?.(url);
  }

  private handleUrlCompleted(url: string, result: AccessibilityResult, duration: number): void {
    const status = result.passed ? '✅ PASSED' : '❌ FAILED';
    console.log(`${status} ${url} (${duration}ms) - ${result.errors.length} errors, ${result.warnings.length} warnings`);
    this.options.onTestComplete?.(url, result);
  }

  private handleUrlFailed(url: string, error: string, attempts: number): void {
    console.log(`💥 Failed ${url} (attempt ${attempts}): ${error}`);
    this.options.onTestError?.(url, error);
  }

  private handleUrlRetrying(url: string, attempts: number): void {
    console.log(`🔄 Retrying ${url} (attempt ${attempts + 1}/${this.options.maxRetries})`);
  }

  private handleQueueEmpty(): void {
    console.log('🎉 All tests completed!');
    this.options.onQueueEmpty?.();
  }

  private handleProgressUpdate(stats: QueueStats): void {
    if (this.options.enableProgressBar) {
      this.displayProgressBar(stats);
    }
    this.options.onProgressUpdate?.(stats);
  }

  private handleError(error: string): void {
    console.error(`❌ Queue error: ${error}`);
  }

  private setupEventListeners(): void {
    // Progress-Update-Interval
    if (this.options.enableProgressBar) {
      setInterval(() => {
        const stats = this.queue.getStats();
        this.handleProgressUpdate(stats);
      }, this.options.progressUpdateInterval);
    }
  }

  private displayProgressBar(stats: QueueStats): void {
    const progress = Math.round(stats.progress);
    const barLength = 30;
    const filledLength = Math.round((progress / 100) * barLength);
    const bar = '█'.repeat(filledLength) + '░'.repeat(barLength - filledLength);
    
    const eta = stats.estimatedTimeRemaining > 0 
      ? `ETA: ${Math.round(stats.estimatedTimeRemaining / 1000)}s`
      : '';
    
    const memory = `Memory: ${stats.memoryUsage}MB`;
    const workers = `Workers: ${stats.activeWorkers}/${this.options.maxConcurrent}`;
    
    process.stdout.write(`\r🧪 Progress: [${bar}] ${progress}% | ${stats.completed}/${stats.total} | ${workers} | ${memory} | ${eta}`);
    
    if (stats.progress >= 100) {
      process.stdout.write('\n');
    }
  }

  // Public API
  pause(): void {
    this.queue.pause();
    console.log('⏸️  Tests paused');
  }

  resume(): void {
    this.queue.resume();
    console.log('▶️  Tests resumed');
    this.startWorkers();
  }

  stop(): void {
    this.isRunning = false;
    this.queue.clear();
    console.log('⏹️  Tests stopped');
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
    console.log(`🔧 Max concurrent workers set to ${max}`);
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