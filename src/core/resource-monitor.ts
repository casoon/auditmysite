/* eslint-disable @typescript-eslint/no-require-imports */
import { EventEmitter } from 'events';
import { performance } from 'perf_hooks';

export interface ResourceMonitorConfig {
  enabled: boolean;
  samplingIntervalMs: number;
  historySize: number;
  
  // Warning thresholds
  memoryWarningThresholdMB: number;
  memoryCriticalThresholdMB: number;
  cpuWarningThresholdPercent: number;
  cpuCriticalThresholdPercent: number;
  heapUsageWarningPercent: number;
  heapUsageCriticalPercent: number;
  
  // Event loop monitoring
  eventLoopWarningDelayMs: number;
  eventLoopCriticalDelayMs: number;
  
  // GC monitoring
  enableGCMonitoring: boolean;
  gcWarningFrequency: number; // GCs per minute
  
  // Disable in CI/test environments by default
  disableInCI: boolean;
}

export interface ResourceSnapshot {
  timestamp: number;
  
  // Memory metrics
  rssMemoryMB: number;
  heapUsedMB: number;
  heapTotalMB: number;
  heapUsagePercent: number;
  externalMemoryMB: number;
  arrayBuffersMB: number;
  
  // Performance metrics
  cpuUsagePercent: number;
  eventLoopDelayMs: number;
  uptimeSeconds: number;
  
  // GC metrics (if available)
  gcCount?: number;
  gcDurationMs?: number;
  gcType?: string;
  
  // Process metrics
  pid: number;
  ppid: number;
  platform: string;
  nodeVersion: string;
}

export interface ResourceTrend {
  metric: keyof ResourceSnapshot;
  trend: 'stable' | 'increasing' | 'decreasing' | 'volatile';
  changePercent: number;
  samples: number;
}

export interface ResourceAlert {
  level: 'warning' | 'critical';
  metric: string;
  current: number;
  threshold: number;
  message: string;
  timestamp: number;
  snapshot: ResourceSnapshot;
}

/**
 * Advanced resource monitoring with trending analysis and alerting
 */
export class ResourceMonitor extends EventEmitter {
  private config: ResourceMonitorConfig;
  private snapshots: ResourceSnapshot[] = [];
  private monitoringTimer?: NodeJS.Timeout;
  private eventLoopTimer?: NodeJS.Timeout;
  private lastEventLoopTime: number = performance.now();
  private eventLoopDelays: number[] = [];
  private gcEvents: Array<{ timestamp: number; type: string; duration: number }> = [];
  private isMonitoring = false;
  
  constructor(config: Partial<ResourceMonitorConfig> = {}) {
    super();
    
    // Check if running in CI environment
    const isCI = process.env.CI === 'true' || 
                 process.env.NODE_ENV === 'test' ||
                 process.env.JEST_WORKER_ID !== undefined;
    
    this.config = {
      enabled: true,
      samplingIntervalMs: 2000,
      historySize: 100,
      memoryWarningThresholdMB: 1536,
      memoryCriticalThresholdMB: 2048,
      cpuWarningThresholdPercent: 70,
      cpuCriticalThresholdPercent: 85,
      heapUsageWarningPercent: 75,
      heapUsageCriticalPercent: 90,
      eventLoopWarningDelayMs: 50,
      eventLoopCriticalDelayMs: 100,
      enableGCMonitoring: false,
      gcWarningFrequency: 60, // 60 GCs per minute is concerning
      disableInCI: true,
      ...config
    };
    
    // Disable in CI if configured to do so
    if (this.config.disableInCI && isCI) {
      this.config.enabled = false;
    }
    
    if (this.config.enabled) {
      this.setupGCMonitoring();
    }
  }
  
  /**
   * Start resource monitoring
   */
  start(): void {
    if (!this.config.enabled || this.isMonitoring) return;
    
    this.isMonitoring = true;
    
    // Take initial snapshot
    this.takeSnapshot();
    
    // Start periodic monitoring
    this.monitoringTimer = setInterval(() => {
      this.takeSnapshot();
      this.analyzeResourceTrends();
    }, this.config.samplingIntervalMs);
    
    // Start event loop monitoring
    this.startEventLoopMonitoring();
    
    this.emit('monitoringStarted', { 
      config: this.config,
      initialSnapshot: this.getCurrentSnapshot()
    });
  }
  
  /**
   * Stop resource monitoring
   */
  stop(): void {
    if (!this.isMonitoring) return;
    
    this.isMonitoring = false;
    
    if (this.monitoringTimer) {
      clearInterval(this.monitoringTimer);
      this.monitoringTimer = undefined;
    }
    
    if (this.eventLoopTimer) {
      clearInterval(this.eventLoopTimer);
      this.eventLoopTimer = undefined;
    }
    
    this.emit('monitoringStopped', {
      totalSnapshots: this.snapshots.length,
      monitoringDurationMs: this.snapshots.length * this.config.samplingIntervalMs
    });
  }
  
  /**
   * Get current resource snapshot
   */
  getCurrentSnapshot(): ResourceSnapshot | null {
    if (!this.config.enabled) return null;
    return this.snapshots[this.snapshots.length - 1] || null;
  }
  
  /**
   * Get resource history
   */
  getHistory(maxSamples?: number): ResourceSnapshot[] {
    const limit = maxSamples || this.snapshots.length;
    return this.snapshots.slice(-limit);
  }
  
  /**
   * Get resource trends analysis
   */
  getResourceTrends(): ResourceTrend[] {
    if (this.snapshots.length < 5) return [];
    
    const recentSnapshots = this.snapshots.slice(-10);
    const trends: ResourceTrend[] = [];
    
    const metricsToAnalyze: Array<keyof ResourceSnapshot> = [
      'rssMemoryMB', 
      'heapUsedMB', 
      'cpuUsagePercent', 
      'eventLoopDelayMs'
    ];
    
    for (const metric of metricsToAnalyze) {
      const values = recentSnapshots.map(s => s[metric] as number).filter(v => v !== undefined);
      if (values.length < 3) continue;
      
      const trend = this.calculateTrend(values);
      trends.push({
        metric,
        trend: trend.direction,
        changePercent: trend.changePercent,
        samples: values.length
      });
    }
    
    return trends;
  }
  
  /**
   * Force garbage collection if available
   */
  forceGC(): boolean {
    if (global.gc) {
      try {
        global.gc();
        return true;
      } catch (error) {
        this.emit('error', { 
          message: 'Failed to trigger garbage collection',
          error 
        });
      }
    }
    return false;
  }
  
  /**
   * Get memory pressure level
   */
  getMemoryPressure(): 'normal' | 'warning' | 'critical' {
    const snapshot = this.getCurrentSnapshot();
    if (!snapshot) return 'normal';
    
    if (snapshot.rssMemoryMB >= this.config.memoryCriticalThresholdMB ||
        snapshot.heapUsagePercent >= this.config.heapUsageCriticalPercent) {
      return 'critical';
    }
    
    if (snapshot.rssMemoryMB >= this.config.memoryWarningThresholdMB ||
        snapshot.heapUsagePercent >= this.config.heapUsageWarningPercent) {
      return 'warning';
    }
    
    return 'normal';
  }
  
  /**
   * Take a resource snapshot
   */
  private takeSnapshot(): void {
    const memUsage = process.memoryUsage();
    const uptime = process.uptime();
    
    // Calculate average event loop delay
    const avgEventLoopDelay = this.eventLoopDelays.length > 0
      ? this.eventLoopDelays.reduce((a, b) => a + b, 0) / this.eventLoopDelays.length
      : 0;
    this.eventLoopDelays = []; // Reset for next interval
    
    const snapshot: ResourceSnapshot = {
      timestamp: Date.now(),
      rssMemoryMB: memUsage.rss / (1024 * 1024),
      heapUsedMB: memUsage.heapUsed / (1024 * 1024),
      heapTotalMB: memUsage.heapTotal / (1024 * 1024),
      heapUsagePercent: (memUsage.heapUsed / memUsage.heapTotal) * 100,
      externalMemoryMB: memUsage.external / (1024 * 1024),
      arrayBuffersMB: (memUsage as any).arrayBuffers ? (memUsage as any).arrayBuffers / (1024 * 1024) : 0,
      cpuUsagePercent: this.calculateCPUUsage(),
      eventLoopDelayMs: avgEventLoopDelay,
      uptimeSeconds: uptime,
      pid: process.pid,
      ppid: process.ppid || 0,
      platform: process.platform,
      nodeVersion: process.version
    };
    
    // Add GC metrics if available
    const recentGC = this.getRecentGCActivity();
    if (recentGC) {
      snapshot.gcCount = recentGC.count;
      snapshot.gcDurationMs = recentGC.avgDuration;
      snapshot.gcType = recentGC.dominantType;
    }
    
    // Add to history
    this.snapshots.push(snapshot);
    
    // Maintain history size limit
    if (this.snapshots.length > this.config.historySize) {
      this.snapshots.shift();
    }
    
    // Check for alerts
    this.checkForAlerts(snapshot);
    
    this.emit('snapshot', snapshot);
  }
  
  /**
   * Calculate CPU usage approximation
   */
  private calculateCPUUsage(): number {
    // This is a rough approximation based on event loop delay
    // More sophisticated CPU monitoring would require native modules
    const avgDelay = this.eventLoopDelays.length > 0
      ? this.eventLoopDelays.reduce((a, b) => a + b, 0) / this.eventLoopDelays.length
      : 0;
    
    // Convert delay to approximate CPU usage percentage
    return Math.min(100, avgDelay * 2);
  }
  
  /**
   * Start event loop delay monitoring
   */
  private startEventLoopMonitoring(): void {
    const measureEventLoop = () => {
      const start = performance.now();
      setImmediate(() => {
        const delay = performance.now() - start;
        this.eventLoopDelays.push(delay);
        
        // Keep only recent measurements
        if (this.eventLoopDelays.length > 50) {
          this.eventLoopDelays.shift();
        }
      });
    };
    
    // Measure event loop delay every 100ms
    this.eventLoopTimer = setInterval(measureEventLoop, 100);
  }
  
  /**
   * Setup garbage collection monitoring if available
   */
  private setupGCMonitoring(): void {
    if (!this.config.enableGCMonitoring) return;
    
    try {
      const v8 = require('v8');
      if (v8.getHeapStatistics) {
        // Monitor GC events
        process.on('exit', () => {
          this.logGCSummary();
        });
      }
    } catch (error) {
      // v8 module not available
    }
  }
  
  /**
   * Get recent GC activity summary
   */
  private getRecentGCActivity() {
    const cutoff = Date.now() - 60000; // Last minute
    const recentEvents = this.gcEvents.filter(event => event.timestamp > cutoff);
    
    if (recentEvents.length === 0) return null;
    
    const avgDuration = recentEvents.reduce((sum, event) => sum + event.duration, 0) / recentEvents.length;
    const typeCounts = recentEvents.reduce((counts, event) => {
      counts[event.type] = (counts[event.type] || 0) + 1;
      return counts;
    }, {} as Record<string, number>);
    
    const dominantType = Object.entries(typeCounts).reduce((a, b) => a[1] > b[1] ? a : b)[0];
    
    return {
      count: recentEvents.length,
      avgDuration,
      dominantType
    };
  }
  
  /**
   * Log GC summary on exit
   */
  private logGCSummary(): void {
    if (this.gcEvents.length > 0) {
      const totalDuration = this.gcEvents.reduce((sum, event) => sum + event.duration, 0);
      this.emit('gcSummary', {
        totalEvents: this.gcEvents.length,
        totalDurationMs: totalDuration,
        avgDurationMs: totalDuration / this.gcEvents.length
      });
    }
  }
  
  /**
   * Check for resource alerts
   */
  private checkForAlerts(snapshot: ResourceSnapshot): void {
    const alerts: ResourceAlert[] = [];
    
    // Memory alerts
    if (snapshot.rssMemoryMB >= this.config.memoryCriticalThresholdMB) {
      alerts.push({
        level: 'critical',
        metric: 'rssMemory',
        current: snapshot.rssMemoryMB,
        threshold: this.config.memoryCriticalThresholdMB,
        message: `RSS memory usage critical: ${snapshot.rssMemoryMB.toFixed(1)}MB`,
        timestamp: snapshot.timestamp,
        snapshot
      });
    } else if (snapshot.rssMemoryMB >= this.config.memoryWarningThresholdMB) {
      alerts.push({
        level: 'warning',
        metric: 'rssMemory',
        current: snapshot.rssMemoryMB,
        threshold: this.config.memoryWarningThresholdMB,
        message: `RSS memory usage high: ${snapshot.rssMemoryMB.toFixed(1)}MB`,
        timestamp: snapshot.timestamp,
        snapshot
      });
    }
    
    // Heap usage alerts
    if (snapshot.heapUsagePercent >= this.config.heapUsageCriticalPercent) {
      alerts.push({
        level: 'critical',
        metric: 'heapUsage',
        current: snapshot.heapUsagePercent,
        threshold: this.config.heapUsageCriticalPercent,
        message: `Heap usage critical: ${snapshot.heapUsagePercent.toFixed(1)}%`,
        timestamp: snapshot.timestamp,
        snapshot
      });
    } else if (snapshot.heapUsagePercent >= this.config.heapUsageWarningPercent) {
      alerts.push({
        level: 'warning',
        metric: 'heapUsage',
        current: snapshot.heapUsagePercent,
        threshold: this.config.heapUsageWarningPercent,
        message: `Heap usage high: ${snapshot.heapUsagePercent.toFixed(1)}%`,
        timestamp: snapshot.timestamp,
        snapshot
      });
    }
    
    // CPU alerts
    if (snapshot.cpuUsagePercent >= this.config.cpuCriticalThresholdPercent) {
      alerts.push({
        level: 'critical',
        metric: 'cpuUsage',
        current: snapshot.cpuUsagePercent,
        threshold: this.config.cpuCriticalThresholdPercent,
        message: `CPU usage critical: ${snapshot.cpuUsagePercent.toFixed(1)}%`,
        timestamp: snapshot.timestamp,
        snapshot
      });
    } else if (snapshot.cpuUsagePercent >= this.config.cpuWarningThresholdPercent) {
      alerts.push({
        level: 'warning',
        metric: 'cpuUsage',
        current: snapshot.cpuUsagePercent,
        threshold: this.config.cpuWarningThresholdPercent,
        message: `CPU usage high: ${snapshot.cpuUsagePercent.toFixed(1)}%`,
        timestamp: snapshot.timestamp,
        snapshot
      });
    }
    
    // Event loop delay alerts
    if (snapshot.eventLoopDelayMs >= this.config.eventLoopCriticalDelayMs) {
      alerts.push({
        level: 'critical',
        metric: 'eventLoopDelay',
        current: snapshot.eventLoopDelayMs,
        threshold: this.config.eventLoopCriticalDelayMs,
        message: `Event loop delay critical: ${snapshot.eventLoopDelayMs.toFixed(1)}ms`,
        timestamp: snapshot.timestamp,
        snapshot
      });
    } else if (snapshot.eventLoopDelayMs >= this.config.eventLoopWarningDelayMs) {
      alerts.push({
        level: 'warning',
        metric: 'eventLoopDelay',
        current: snapshot.eventLoopDelayMs,
        threshold: this.config.eventLoopWarningDelayMs,
        message: `Event loop delay high: ${snapshot.eventLoopDelayMs.toFixed(1)}ms`,
        timestamp: snapshot.timestamp,
        snapshot
      });
    }
    
    // Emit alerts
    alerts.forEach(alert => {
      this.emit('resourceAlert', alert);
      if (alert.level === 'critical') {
        this.emit('criticalAlert', alert);
      }
    });
  }
  
  /**
   * Calculate trend from a series of values
   */
  private calculateTrend(values: number[]): { direction: ResourceTrend['trend']; changePercent: number } {
    if (values.length < 3) {
      return { direction: 'stable', changePercent: 0 };
    }
    
    const first = values[0];
    const last = values[values.length - 1];
    const changePercent = ((last - first) / first) * 100;
    
    // Calculate volatility
    const avg = values.reduce((sum, val) => sum + val, 0) / values.length;
    const variance = values.reduce((sum, val) => sum + Math.pow(val - avg, 2), 0) / values.length;
    const coefficientOfVariation = Math.sqrt(variance) / avg;
    
    if (coefficientOfVariation > 0.2) {
      return { direction: 'volatile', changePercent };
    }
    
    if (Math.abs(changePercent) < 5) {
      return { direction: 'stable', changePercent };
    }
    
    return {
      direction: changePercent > 0 ? 'increasing' : 'decreasing',
      changePercent: Math.abs(changePercent)
    };
  }
  
  /**
   * Analyze resource trends and emit trend updates
   */
  private analyzeResourceTrends(): void {
    const trends = this.getResourceTrends();
    
    if (trends.length > 0) {
      this.emit('trendsUpdate', trends);
      
      // Check for concerning trends
      const concerningTrends = trends.filter(trend => 
        trend.trend === 'increasing' && 
        ['rssMemoryMB', 'heapUsedMB', 'cpuUsagePercent'].includes(trend.metric) &&
        trend.changePercent > 20
      );
      
      if (concerningTrends.length > 0) {
        this.emit('concerningTrends', concerningTrends);
      }
    }
  }
  
  /**
   * Clean up resources
   */
  destroy(): void {
    this.stop();
    this.removeAllListeners();
    this.snapshots = [];
    this.eventLoopDelays = [];
    this.gcEvents = [];
  }
}
