import { EventEmitter } from 'events';

export interface BackpressureConfig {
  enabled: boolean;
  maxQueueSize: number;
  backpressureThreshold: number;
  
  // Resource thresholds
  maxMemoryUsageMB: number;
  maxCpuUsagePercent: number;
  
  // Adaptive delay configuration
  minDelayMs: number;
  maxDelayMs: number;
  delayGrowthFactor: number;
  
  // Hysteresis to prevent oscillation
  activationThreshold: number;
  deactivationThreshold: number;
  
  // Monitoring intervals
  resourceSamplingIntervalMs: number;
  
  // Error rate thresholds
  maxErrorRatePercent: number;
  errorRateWindowSize: number;
}

export interface BackpressureMetrics {
  isActive: boolean;
  currentDelay: number;
  memoryUsageMB: number;
  cpuUsagePercent: number;
  queueLength: number;
  concurrency: number;
  errorRate: number;
  activationCount: number;
  totalDelayTime: number;
  peakMemoryMB: number;
  gcCount: number;
}

export interface ResourceSample {
  timestamp: number;
  memoryUsageMB: number;
  cpuUsagePercent: number;
  heapUsedMB: number;
  heapTotalMB: number;
}

export class AdaptiveBackpressureController extends EventEmitter {
  private config: BackpressureConfig;
  private metrics: BackpressureMetrics;
  private resourceSamples: ResourceSample[] = [];
  private errorWindow: boolean[] = [];
  private samplingTimer?: NodeJS.Timeout;
  private isActive = false;
  private currentDelay = 0;
  
  constructor(config: Partial<BackpressureConfig> = {}) {
    super();
    
    this.config = {
      enabled: false,
      maxQueueSize: 1000,
      backpressureThreshold: 0.8,
      maxMemoryUsageMB: 2048,
      maxCpuUsagePercent: 85,
      minDelayMs: 10,
      maxDelayMs: 5000,
      delayGrowthFactor: 1.5,
      activationThreshold: 0.9,
      deactivationThreshold: 0.7,
      resourceSamplingIntervalMs: 1000,
      maxErrorRatePercent: 15,
      errorRateWindowSize: 20,
      ...config
    };
    
    this.metrics = {
      isActive: false,
      currentDelay: 0,
      memoryUsageMB: 0,
      cpuUsagePercent: 0,
      queueLength: 0,
      concurrency: 0,
      errorRate: 0,
      activationCount: 0,
      totalDelayTime: 0,
      peakMemoryMB: 0,
      gcCount: 0
    };
    
    if (this.config.enabled) {
      this.startResourceMonitoring();
    }
  }
  
  /**
   * Update queue state and assess backpressure needs
   */
  updateQueueState(queueLength: number, concurrency: number, hasError: boolean = false): void {
    if (!this.config.enabled) return;
    
    this.metrics.queueLength = queueLength;
    this.metrics.concurrency = concurrency;
    
    // Track errors for error rate calculation
    if (this.errorWindow.length >= this.config.errorRateWindowSize) {
      this.errorWindow.shift();
    }
    this.errorWindow.push(hasError);
    
    this.metrics.errorRate = (this.errorWindow.filter(Boolean).length / this.errorWindow.length) * 100;
    
    this.assessBackpressure();
  }
  
  /**
   * Get current adaptive delay in milliseconds
   */
  getCurrentDelay(): number {
    return this.config.enabled ? this.currentDelay : 0;
  }
  
  /**
   * Get current metrics snapshot
   */
  getMetrics(): BackpressureMetrics {
    return { ...this.metrics };
  }
  
  /**
   * Check if backpressure is currently active
   */
  isBackpressureActive(): boolean {
    return this.isActive;
  }
  
  /**
   * Start resource monitoring
   */
  private startResourceMonitoring(): void {
    if (this.samplingTimer) return;
    
    this.samplingTimer = setInterval(() => {
      this.sampleResources();
    }, this.config.resourceSamplingIntervalMs);
  }
  
  /**
   * Stop resource monitoring
   */
  private stopResourceMonitoring(): void {
    if (this.samplingTimer) {
      clearInterval(this.samplingTimer);
      this.samplingTimer = undefined;
    }
  }
  
  /**
   * Sample current resource usage
   */
  private sampleResources(): void {
    const memUsage = process.memoryUsage();
    const memoryUsageMB = memUsage.rss / (1024 * 1024);
    const heapUsedMB = memUsage.heapUsed / (1024 * 1024);
    const heapTotalMB = memUsage.heapTotal / (1024 * 1024);
    
    // CPU usage calculation (approximation based on event loop delay)
    const start = process.hrtime.bigint();
    setImmediate(() => {
      const delta = Number(process.hrtime.bigint() - start) / 1000000; // Convert to ms
      const cpuUsagePercent = Math.min(100, delta * 10); // Rough approximation
      
      const sample: ResourceSample = {
        timestamp: Date.now(),
        memoryUsageMB,
        cpuUsagePercent,
        heapUsedMB,
        heapTotalMB
      };
      
      // Keep only recent samples (last 60 seconds)
      const cutoff = Date.now() - 60000;
      this.resourceSamples = this.resourceSamples.filter(s => s.timestamp > cutoff);
      this.resourceSamples.push(sample);
      
      // Update current metrics
      this.metrics.memoryUsageMB = memoryUsageMB;
      this.metrics.cpuUsagePercent = cpuUsagePercent;
      this.metrics.peakMemoryMB = Math.max(this.metrics.peakMemoryMB, memoryUsageMB);
      
      // Emit resource warnings
      this.checkResourceWarnings(sample);
    });
  }
  
  /**
   * Check for resource warnings and emit events
   */
  private checkResourceWarnings(sample: ResourceSample): void {
    const memoryWarningThreshold = this.config.maxMemoryUsageMB * 0.8;
    const cpuWarningThreshold = this.config.maxCpuUsagePercent * 0.8;
    
    if (sample.memoryUsageMB > memoryWarningThreshold) {
      this.emit('memoryWarning', {
        current: sample.memoryUsageMB,
        threshold: memoryWarningThreshold,
        max: this.config.maxMemoryUsageMB
      });
    }
    
    if (sample.memoryUsageMB > this.config.maxMemoryUsageMB) {
      this.emit('memoryCritical', {
        current: sample.memoryUsageMB,
        max: this.config.maxMemoryUsageMB
      });
    }
    
    if (sample.cpuUsagePercent > cpuWarningThreshold) {
      this.emit('cpuWarning', {
        current: sample.cpuUsagePercent,
        threshold: cpuWarningThreshold,
        max: this.config.maxCpuUsagePercent
      });
    }
  }
  
  /**
   * Assess whether backpressure should be activated/deactivated
   */
  private assessBackpressure(): void {
    const pressureFactors = this.calculatePressureFactors();
    const overallPressure = Math.max(...Object.values(pressureFactors));
    
    if (!this.isActive && overallPressure >= this.config.activationThreshold) {
      this.activateBackpressure(pressureFactors);
    } else if (this.isActive && overallPressure <= this.config.deactivationThreshold) {
      this.deactivateBackpressure();
    } else if (this.isActive) {
      this.adjustDelay(overallPressure);
    }
  }
  
  /**
   * Calculate pressure factors from various metrics
   */
  private calculatePressureFactors(): Record<string, number> {
    const factors: Record<string, number> = {
      queue: 0,
      memory: 0,
      cpu: 0,
      error: 0
    };
    
    // Queue length pressure
    factors.queue = this.metrics.queueLength / this.config.maxQueueSize;
    
    // Memory pressure
    factors.memory = this.metrics.memoryUsageMB / this.config.maxMemoryUsageMB;
    
    // CPU pressure
    factors.cpu = this.metrics.cpuUsagePercent / this.config.maxCpuUsagePercent;
    
    // Error rate pressure
    factors.error = this.metrics.errorRate / this.config.maxErrorRatePercent;
    
    return factors;
  }
  
  /**
   * Activate backpressure with initial delay calculation
   */
  private activateBackpressure(factors: Record<string, number>): void {
    this.isActive = true;
    this.metrics.isActive = true;
    this.metrics.activationCount++;
    
    const maxFactor = Math.max(...Object.values(factors));
    this.currentDelay = Math.min(
      this.config.maxDelayMs,
      this.config.minDelayMs * Math.pow(this.config.delayGrowthFactor, maxFactor * 10)
    );
    
    this.metrics.currentDelay = this.currentDelay;
    
    this.emit('backpressureActivated', {
      factors,
      initialDelay: this.currentDelay,
      metrics: this.getMetrics()
    });
  }
  
  /**
   * Deactivate backpressure
   */
  private deactivateBackpressure(): void {
    this.isActive = false;
    this.metrics.isActive = false;
    this.currentDelay = 0;
    this.metrics.currentDelay = 0;
    
    this.emit('backpressureDeactivated', {
      metrics: this.getMetrics()
    });
  }
  
  /**
   * Adjust delay based on current pressure
   */
  private adjustDelay(pressure: number): void {
    const targetDelay = Math.min(
      this.config.maxDelayMs,
      this.config.minDelayMs * Math.pow(this.config.delayGrowthFactor, pressure * 10)
    );
    
    // Smooth adjustment
    this.currentDelay = Math.round((this.currentDelay * 0.7) + (targetDelay * 0.3));
    this.metrics.currentDelay = this.currentDelay;
    this.metrics.totalDelayTime += this.currentDelay;
  }
  
  /**
   * Force garbage collection if available and conditions are met
   */
  triggerGarbageCollection(): boolean {
    if (global.gc && this.metrics.memoryUsageMB > this.config.maxMemoryUsageMB * 0.7) {
      try {
        global.gc();
        this.metrics.gcCount++;
        this.emit('gcTriggered', {
          beforeMB: this.metrics.memoryUsageMB,
          gcCount: this.metrics.gcCount
        });
        return true;
      } catch (error) {
        this.emit('gcError', error);
      }
    }
    return false;
  }
  
  /**
   * Cleanup and stop monitoring
   */
  destroy(): void {
    this.stopResourceMonitoring();
    this.removeAllListeners();
  }
}
