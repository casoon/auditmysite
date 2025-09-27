/**
 * 🏥 Health Monitoring Utility
 * 
 * Provides health monitoring and early warning system
 * for the audit infrastructure.
 */

export interface HealthMetrics {
  timestamp: number;
  memoryUsage: NodeJS.MemoryUsage;
  cpuUsage: NodeJS.CpuUsage;
  uptime: number;
  processId: number;
  nodeVersion: string;
  platform: string;
}

export interface HealthThresholds {
  maxMemoryMB: number;
  maxCpuPercent: number;
  maxUptimeHours: number;
  minFreeMemoryMB: number;
}

export interface HealthAlert {
  level: 'info' | 'warning' | 'critical';
  metric: string;
  message: string;
  value: number;
  threshold: number;
  timestamp: number;
}

export class HealthMonitor {
  private thresholds: HealthThresholds;
  private alerts: HealthAlert[] = [];
  private metrics: HealthMetrics[] = [];
  private monitoring = false;
  private interval?: NodeJS.Timeout;

  constructor(thresholds: Partial<HealthThresholds> = {}) {
    this.thresholds = {
      maxMemoryMB: 2048, // 2GB
      maxCpuPercent: 80,
      maxUptimeHours: 24, // Restart after 24h
      minFreeMemoryMB: 512, // 512MB minimum free
      ...thresholds
    };
  }

  /**
   * Start continuous health monitoring
   */
  start(intervalMs: number = 30000): void {
    if (this.monitoring) return;
    
    this.monitoring = true;
    console.log('🏥 Health monitoring started');
    
    this.interval = setInterval(() => {
      this.checkHealth();
    }, intervalMs);
    
    // Initial check
    this.checkHealth();
  }

  /**
   * Stop health monitoring
   */
  stop(): void {
    if (!this.monitoring) return;
    
    this.monitoring = false;
    if (this.interval) {
      clearInterval(this.interval);
      this.interval = undefined;
    }
    
    console.log('🏥 Health monitoring stopped');
  }

  /**
   * Perform immediate health check
   */
  checkHealth(): HealthMetrics {
    const metrics = this.collectMetrics();
    this.metrics.push(metrics);
    
    // Keep only last 100 metrics
    if (this.metrics.length > 100) {
      this.metrics = this.metrics.slice(-100);
    }
    
    // Check thresholds
    this.checkThresholds(metrics);
    
    return metrics;
  }

  /**
   * Get current health status
   */
  getHealthStatus(): {
    status: 'healthy' | 'warning' | 'critical';
    metrics: HealthMetrics;
    alerts: HealthAlert[];
    summary: string;
  } {
    const metrics = this.collectMetrics();
    const recentAlerts = this.alerts.filter(a => Date.now() - a.timestamp < 300000); // Last 5 minutes
    
    let status: 'healthy' | 'warning' | 'critical' = 'healthy';
    
    if (recentAlerts.some(a => a.level === 'critical')) {
      status = 'critical';
    } else if (recentAlerts.some(a => a.level === 'warning')) {
      status = 'warning';
    }
    
    const memoryMB = Math.round(metrics.memoryUsage.heapUsed / 1024 / 1024);
    const uptimeHours = (metrics.uptime / 3600).toFixed(1);
    
    const summary = `Memory: ${memoryMB}MB, Uptime: ${uptimeHours}h, Alerts: ${recentAlerts.length}`;
    
    return {
      status,
      metrics,
      alerts: recentAlerts,
      summary
    };
  }

  /**
   * Get performance trends
   */
  getTrends(): {
    memoryTrend: 'stable' | 'increasing' | 'decreasing';
    averageMemoryMB: number;
    peakMemoryMB: number;
    measurementCount: number;
  } {
    if (this.metrics.length < 3) {
      return {
        memoryTrend: 'stable',
        averageMemoryMB: 0,
        peakMemoryMB: 0,
        measurementCount: 0
      };
    }
    
    const recent = this.metrics.slice(-10); // Last 10 measurements
    const memoryValues = recent.map(m => m.memoryUsage.heapUsed / 1024 / 1024);
    
    const firstThird = memoryValues.slice(0, Math.floor(memoryValues.length / 3));
    const lastThird = memoryValues.slice(-Math.floor(memoryValues.length / 3));
    
    const firstAvg = firstThird.reduce((sum, val) => sum + val, 0) / firstThird.length;
    const lastAvg = lastThird.reduce((sum, val) => sum + val, 0) / lastThird.length;
    
    let memoryTrend: 'stable' | 'increasing' | 'decreasing' = 'stable';
    const changePercent = ((lastAvg - firstAvg) / firstAvg) * 100;
    
    if (changePercent > 10) {
      memoryTrend = 'increasing';
    } else if (changePercent < -10) {
      memoryTrend = 'decreasing';
    }
    
    return {
      memoryTrend,
      averageMemoryMB: Math.round(memoryValues.reduce((sum, val) => sum + val, 0) / memoryValues.length),
      peakMemoryMB: Math.round(Math.max(...memoryValues)),
      measurementCount: this.metrics.length
    };
  }

  /**
   * Export health data for analysis
   */
  exportHealthData(): {
    thresholds: HealthThresholds;
    alerts: HealthAlert[];
    metrics: HealthMetrics[];
    trends: {
      memoryTrend: 'stable' | 'increasing' | 'decreasing';
      averageMemoryMB: number;
      peakMemoryMB: number;
      measurementCount: number;
    };
    timestamp: number;
  } {
    return {
      thresholds: this.thresholds,
      alerts: this.alerts,
      metrics: this.metrics,
      trends: this.getTrends(),
      timestamp: Date.now()
    };
  }

  /**
   * Clear all stored data
   */
  clear(): void {
    this.alerts = [];
    this.metrics = [];
  }

  private collectMetrics(): HealthMetrics {
    return {
      timestamp: Date.now(),
      memoryUsage: process.memoryUsage(),
      cpuUsage: process.cpuUsage(),
      uptime: process.uptime(),
      processId: process.pid,
      nodeVersion: process.version,
      platform: process.platform
    };
  }

  private checkThresholds(metrics: HealthMetrics): void {
    const memoryMB = metrics.memoryUsage.heapUsed / 1024 / 1024;
    const uptimeHours = metrics.uptime / 3600;
    
    // Check memory usage
    if (memoryMB > this.thresholds.maxMemoryMB) {
      this.addAlert('critical', 'memory', 
        `High memory usage: ${Math.round(memoryMB)}MB exceeds ${this.thresholds.maxMemoryMB}MB threshold`,
        memoryMB, this.thresholds.maxMemoryMB);
    } else if (memoryMB > this.thresholds.maxMemoryMB * 0.8) {
      this.addAlert('warning', 'memory', 
        `Memory usage approaching limit: ${Math.round(memoryMB)}MB (${this.thresholds.maxMemoryMB}MB limit)`,
        memoryMB, this.thresholds.maxMemoryMB);
    }
    
    // Check uptime
    if (uptimeHours > this.thresholds.maxUptimeHours) {
      this.addAlert('warning', 'uptime', 
        `Long uptime detected: ${uptimeHours.toFixed(1)}h exceeds ${this.thresholds.maxUptimeHours}h recommendation`,
        uptimeHours, this.thresholds.maxUptimeHours);
    }
    
    // Check free memory
    const freeMB = (metrics.memoryUsage.heapTotal - metrics.memoryUsage.heapUsed) / 1024 / 1024;
    if (freeMB < this.thresholds.minFreeMemoryMB) {
      this.addAlert('critical', 'free_memory', 
        `Low free memory: ${Math.round(freeMB)}MB below ${this.thresholds.minFreeMemoryMB}MB minimum`,
        freeMB, this.thresholds.minFreeMemoryMB);
    }
  }

  private addAlert(level: HealthAlert['level'], metric: string, message: string, value: number, threshold: number): void {
    // Don't duplicate recent alerts for same metric
    const recentSimilar = this.alerts.filter(a => 
      a.metric === metric && 
      a.level === level && 
      Date.now() - a.timestamp < 60000 // Last minute
    );
    
    if (recentSimilar.length > 0) return;
    
    const alert: HealthAlert = {
      level,
      metric,
      message,
      value,
      threshold,
      timestamp: Date.now()
    };
    
    this.alerts.push(alert);
    
    // Keep only last 100 alerts
    if (this.alerts.length > 100) {
      this.alerts = this.alerts.slice(-100);
    }
    
    // Log the alert
    const emoji = level === 'critical' ? '🚨' : level === 'warning' ? '⚠️' : 'ℹ️';
    console.log(`${emoji} Health Alert (${level}): ${message}`);
  }
}

// Singleton instance for global health monitoring
let globalMonitor: HealthMonitor | null = null;

export function getGlobalHealthMonitor(thresholds?: Partial<HealthThresholds>): HealthMonitor {
  if (!globalMonitor) {
    globalMonitor = new HealthMonitor(thresholds);
  }
  return globalMonitor;
}

export function startGlobalHealthMonitoring(intervalMs: number = 30000): HealthMonitor {
  const monitor = getGlobalHealthMonitor();
  monitor.start(intervalMs);
  return monitor;
}

export function stopGlobalHealthMonitoring(): void {
  if (globalMonitor) {
    globalMonitor.stop();
  }
}