/**
 * 🔧 Queue Configuration Factory
 * 
 * Provides safe default configurations with environment variable overrides
 * and automatic tuning based on system capabilities.
 */

import { QueueConfig, QueueType } from './types';
import { BackpressureConfig } from '../backpressure-controller';
import { ResourceMonitorConfig } from '../resource-monitor';

export interface EnvironmentConfig {
  // Core queue settings
  QUEUE_MAX_CONCURRENT?: string;
  QUEUE_MAX_RETRIES?: string;
  QUEUE_RETRY_DELAY?: string;
  QUEUE_TIMEOUT?: string;
  QUEUE_MAX_SIZE?: string;
  
  // Backpressure settings
  QUEUE_ENABLE_BACKPRESSURE?: string;
  QUEUE_BACKPRESSURE_THRESHOLD?: string;
  QUEUE_MAX_MEMORY_MB?: string;
  QUEUE_MAX_CPU_PERCENT?: string;
  QUEUE_MIN_DELAY_MS?: string;
  QUEUE_MAX_DELAY_MS?: string;
  
  // Resource monitoring
  QUEUE_ENABLE_RESOURCE_MONITORING?: string;
  QUEUE_MEMORY_WARNING_MB?: string;
  QUEUE_MEMORY_CRITICAL_MB?: string;
  QUEUE_SAMPLING_INTERVAL_MS?: string;
  
  // Performance tuning
  QUEUE_ENABLE_ADAPTIVE_DELAY?: string;
  QUEUE_ENABLE_GC?: string;
  QUEUE_GC_INTERVAL?: string;
  QUEUE_PROGRESS_INTERVAL?: string;
  
  // CI/Test environment
  CI?: string;
  NODE_ENV?: string;
  JEST_WORKER_ID?: string;
}

export class QueueConfigFactory {
  /**
   * Create optimized configuration based on queue type and system capabilities
   */
  static createOptimizedConfig(
    type: QueueType,
    customConfig: Partial<QueueConfig> = {},
    envOverrides: EnvironmentConfig = process.env as EnvironmentConfig
  ): QueueConfig {
    const systemInfo = this.getSystemInfo();
    const isCI = this.isCIEnvironment(envOverrides);
    const baseConfig = this.getBaseConfigForType(type, systemInfo, isCI);
    const envConfig = this.parseEnvironmentConfig(envOverrides);
    
    // Merge configurations with priority: custom > env > base
    const finalConfig: QueueConfig = {
      ...baseConfig,
      ...envConfig,
      ...customConfig
    };
    
    // Validate and adjust configuration
    return this.validateAndAdjustConfig(finalConfig, systemInfo);
  }
  
  /**
   * Get base configuration for queue type
   */
  private static getBaseConfigForType(
    type: QueueType,
    systemInfo: SystemInfo,
    isCI: boolean
  ): QueueConfig {
    const baseConfig: QueueConfig = {
      // Core settings
      maxRetries: 3,
      retryDelay: 1000,
      timeout: 30000,
      enableEvents: true,
      enableProgressReporting: !isCI,
      progressUpdateInterval: isCI ? 5000 : 2000,
      
      // Backpressure and resource management (disabled by default in CI)
      enableBackpressure: !isCI,
      maxQueueSize: isCI ? 100 : 1000,
      backpressureThreshold: 0.8,
      adaptiveDelay: !isCI,
      maxMemoryUsage: Math.min(2048, Math.floor(systemInfo.totalMemoryMB * 0.6)),
      enableGarbageCollection: !isCI,
      gcInterval: 30000,
      
      // Resource monitoring
      enableResourceMonitoring: !isCI,
      enablePerformanceMetrics: !isCI,
      metricsCollectionInterval: 3000
    };
    
    // Type-specific configurations
    switch (type) {
      case 'simple':
        return {
          ...baseConfig,
          maxConcurrent: 1,
          enableBackpressure: false,
          enableResourceMonitoring: false
        };
      
      case 'priority':
        return {
          ...baseConfig,
          maxConcurrent: Math.min(3, systemInfo.cpuCount),
          priorityPatterns: [
            { pattern: '/home', priority: 10 },
            { pattern: '/', priority: 9 },
            { pattern: '/about', priority: 8 },
            { pattern: '/contact', priority: 7 }
          ]
        };
      
      case 'parallel':
        const optimalConcurrency = isCI 
          ? Math.min(2, systemInfo.cpuCount) 
          : Math.min(systemInfo.cpuCount * 2, Math.floor(systemInfo.totalMemoryMB / 256));
        
        return {
          ...baseConfig,
          maxConcurrent: Math.max(1, optimalConcurrency),
          enableBackpressure: !isCI && systemInfo.totalMemoryMB < 4096, // Enable on systems with <4GB RAM
          adaptiveDelay: !isCI
        };
      
      case 'persistent':
        return {
          ...baseConfig,
          maxConcurrent: Math.min(2, systemInfo.cpuCount),
          enablePersistence: true,
          enableBackpressure: !isCI,
          maxQueueSize: isCI ? 50 : 500, // Smaller queue for persistent type
          memoryCheckInterval: 5000
        };
      
      default:
        return baseConfig;
    }
  }
  
  /**
   * Parse environment variables into configuration
   */
  private static parseEnvironmentConfig(env: EnvironmentConfig): Partial<QueueConfig> {
    const config: Partial<QueueConfig> = {};
    
    // Core settings
    if (env.QUEUE_MAX_CONCURRENT) {
      config.maxConcurrent = parseInt(env.QUEUE_MAX_CONCURRENT, 10);
    }
    if (env.QUEUE_MAX_RETRIES) {
      config.maxRetries = parseInt(env.QUEUE_MAX_RETRIES, 10);
    }
    if (env.QUEUE_RETRY_DELAY) {
      config.retryDelay = parseInt(env.QUEUE_RETRY_DELAY, 10);
    }
    if (env.QUEUE_TIMEOUT) {
      config.timeout = parseInt(env.QUEUE_TIMEOUT, 10);
    }
    if (env.QUEUE_MAX_SIZE) {
      config.maxQueueSize = parseInt(env.QUEUE_MAX_SIZE, 10);
    }
    
    // Boolean settings
    if (env.QUEUE_ENABLE_BACKPRESSURE) {
      config.enableBackpressure = env.QUEUE_ENABLE_BACKPRESSURE.toLowerCase() === 'true';
    }
    if (env.QUEUE_ENABLE_RESOURCE_MONITORING) {
      config.enableResourceMonitoring = env.QUEUE_ENABLE_RESOURCE_MONITORING.toLowerCase() === 'true';
    }
    if (env.QUEUE_ENABLE_ADAPTIVE_DELAY) {
      config.adaptiveDelay = env.QUEUE_ENABLE_ADAPTIVE_DELAY.toLowerCase() === 'true';
    }
    if (env.QUEUE_ENABLE_GC) {
      config.enableGarbageCollection = env.QUEUE_ENABLE_GC.toLowerCase() === 'true';
    }
    
    // Backpressure settings
    if (env.QUEUE_BACKPRESSURE_THRESHOLD) {
      config.backpressureThreshold = parseFloat(env.QUEUE_BACKPRESSURE_THRESHOLD);
    }
    if (env.QUEUE_MAX_MEMORY_MB) {
      config.maxMemoryUsage = parseInt(env.QUEUE_MAX_MEMORY_MB, 10);
    }
    
    // Timing settings
    if (env.QUEUE_PROGRESS_INTERVAL) {
      config.progressUpdateInterval = parseInt(env.QUEUE_PROGRESS_INTERVAL, 10);
    }
    if (env.QUEUE_GC_INTERVAL) {
      config.gcInterval = parseInt(env.QUEUE_GC_INTERVAL, 10);
    }
    
    return config;
  }
  
  /**
   * Create backpressure configuration from queue config
   */
  static createBackpressureConfig(queueConfig: QueueConfig): BackpressureConfig {
    return {
      enabled: queueConfig.enableBackpressure || false,
      maxQueueSize: queueConfig.maxQueueSize || 1000,
      backpressureThreshold: queueConfig.backpressureThreshold || 0.8,
      maxMemoryUsageMB: queueConfig.maxMemoryUsage || 2048,
      maxCpuUsagePercent: 85,
      minDelayMs: 10,
      maxDelayMs: 5000,
      delayGrowthFactor: 1.5,
      activationThreshold: 0.85,
      deactivationThreshold: 0.65,
      resourceSamplingIntervalMs: 2000,
      maxErrorRatePercent: 15,
      errorRateWindowSize: 20
    };
  }
  
  /**
   * Create resource monitor configuration from queue config
   */
  static createResourceMonitorConfig(queueConfig: QueueConfig): ResourceMonitorConfig {
    const memoryThreshold = queueConfig.maxMemoryUsage || 2048;
    
    return {
      enabled: queueConfig.enableResourceMonitoring || false,
      samplingIntervalMs: queueConfig.metricsCollectionInterval || 3000,
      historySize: 100,
      memoryWarningThresholdMB: Math.floor(memoryThreshold * 0.75),
      memoryCriticalThresholdMB: memoryThreshold,
      cpuWarningThresholdPercent: 70,
      cpuCriticalThresholdPercent: 85,
      heapUsageWarningPercent: 75,
      heapUsageCriticalPercent: 90,
      eventLoopWarningDelayMs: 50,
      eventLoopCriticalDelayMs: 100,
      enableGCMonitoring: queueConfig.enableGarbageCollection || false,
      gcWarningFrequency: 60,
      disableInCI: true
    };
  }
  
  /**
   * Validate and adjust configuration for safety
   */
  private static validateAndAdjustConfig(
    config: QueueConfig,
    systemInfo: SystemInfo
  ): QueueConfig {
    const validated = { ...config };
    
    // Ensure safe concurrency limits
    if (validated.maxConcurrent) {
      validated.maxConcurrent = Math.max(1, Math.min(validated.maxConcurrent, systemInfo.cpuCount * 4));
    }
    
    // Ensure reasonable memory limits
    if (validated.maxMemoryUsage) {
      const maxSafeMemory = Math.floor(systemInfo.totalMemoryMB * 0.8); // Max 80% of system memory
      validated.maxMemoryUsage = Math.min(validated.maxMemoryUsage, maxSafeMemory);
    }
    
    // Ensure reasonable queue size
    if (validated.maxQueueSize) {
      const maxSafeQueueSize = Math.floor(systemInfo.totalMemoryMB / 2); // Rough heuristic
      validated.maxQueueSize = Math.min(validated.maxQueueSize, Math.max(100, maxSafeQueueSize));
    }
    
    // Ensure reasonable timeouts
    if (validated.timeout && validated.timeout < 1000) {
      validated.timeout = 1000; // Minimum 1 second timeout
    }
    if (validated.timeout && validated.timeout > 300000) {
      validated.timeout = 300000; // Maximum 5 minute timeout
    }
    
    // Ensure reasonable retry settings
    if (validated.maxRetries && validated.maxRetries > 10) {
      validated.maxRetries = 10; // Maximum 10 retries
    }
    if (validated.retryDelay && validated.retryDelay < 100) {
      validated.retryDelay = 100; // Minimum 100ms retry delay
    }
    
    // Adjust thresholds to safe ranges
    if (validated.backpressureThreshold) {
      validated.backpressureThreshold = Math.max(0.1, Math.min(1.0, validated.backpressureThreshold));
    }
    
    return validated;
  }
  
  /**
   * Get system information
   */
  private static getSystemInfo(): SystemInfo {
    const os = require('os');
    const totalMemoryMB = Math.floor(os.totalmem() / (1024 * 1024));
    const freeMemoryMB = Math.floor(os.freemem() / (1024 * 1024));
    const cpuCount = os.cpus().length;
    
    return {
      totalMemoryMB,
      freeMemoryMB,
      cpuCount,
      platform: os.platform(),
      arch: os.arch(),
      nodeVersion: process.version
    };
  }
  
  /**
   * Check if running in CI environment
   */
  private static isCIEnvironment(env: EnvironmentConfig): boolean {
    return env.CI === 'true' || 
           env.NODE_ENV === 'test' ||
           env.JEST_WORKER_ID !== undefined ||
           process.env.GITHUB_ACTIONS === 'true' ||
           process.env.TRAVIS === 'true' ||
           process.env.CIRCLECI === 'true';
  }
  
  /**
   * Create configuration for accessibility testing workload
   */
  static createAccessibilityTestingConfig(
    customConfig: Partial<QueueConfig> = {},
    envOverrides: EnvironmentConfig = process.env as EnvironmentConfig
  ): QueueConfig {
    const systemInfo = this.getSystemInfo();
    const isCI = this.isCIEnvironment(envOverrides);
    
    // Accessibility testing specific defaults
    const accessibilityDefaults: QueueConfig = {
      maxConcurrent: isCI ? 1 : Math.min(3, systemInfo.cpuCount), // Conservative concurrency for browser automation
      maxRetries: 2, // Fewer retries for accessibility tests
      retryDelay: 2000, // Longer delay between retries
      timeout: 60000, // 1 minute timeout for browser operations
      maxQueueSize: isCI ? 50 : 200,
      
      // Enhanced monitoring for accessibility workloads
      enableBackpressure: !isCI,
      enableResourceMonitoring: !isCI,
      enablePerformanceMetrics: !isCI,
      
      // Conservative memory limits for browser automation
      maxMemoryUsage: Math.min(1536, Math.floor(systemInfo.totalMemoryMB * 0.5)),
      backpressureThreshold: 0.7, // More aggressive backpressure
      
      // Frequent progress updates
      enableProgressReporting: true,
      progressUpdateInterval: isCI ? 10000 : 3000,
      
      // Priority patterns for common accessibility-critical pages
      priorityPatterns: [
        { pattern: '/accessibility', priority: 10 },
        { pattern: '/login', priority: 9 },
        { pattern: '/signup', priority: 9 },
        { pattern: '/home', priority: 8 },
        { pattern: '/', priority: 8 },
        { pattern: '/about', priority: 7 },
        { pattern: '/contact', priority: 7 },
        { pattern: '/help', priority: 6 }
      ],
      
      enableEvents: true,
      adaptiveDelay: !isCI,
      enableGarbageCollection: !isCI,
      gcInterval: 20000, // More frequent GC for browser automation
      metricsCollectionInterval: 5000
    };
    
    const envConfig = this.parseEnvironmentConfig(envOverrides);
    
    const finalConfig: QueueConfig = {
      ...accessibilityDefaults,
      ...envConfig,
      ...customConfig
    };
    
    return this.validateAndAdjustConfig(finalConfig, systemInfo);
  }
  
  /**
   * Get recommended configuration for production use
   */
  static createProductionConfig(
    type: QueueType = 'parallel',
    customConfig: Partial<QueueConfig> = {}
  ): QueueConfig {
    const systemInfo = this.getSystemInfo();
    
    const productionDefaults: QueueConfig = {
      // Conservative settings for production
      maxConcurrent: Math.min(systemInfo.cpuCount, 4),
      maxRetries: 3,
      retryDelay: 1500,
      timeout: 45000,
      maxQueueSize: 2000,
      
      // Enable all monitoring in production
      enableBackpressure: true,
      enableResourceMonitoring: true,
      enablePerformanceMetrics: true,
      enableEvents: true,
      enableProgressReporting: true,
      
      // Production memory management
      maxMemoryUsage: Math.floor(systemInfo.totalMemoryMB * 0.6),
      backpressureThreshold: 0.75,
      adaptiveDelay: true,
      enableGarbageCollection: true,
      gcInterval: 45000,
      
      // Balanced update intervals
      progressUpdateInterval: 5000,
      metricsCollectionInterval: 3000,
      memoryCheckInterval: 2000
    };
    
    const baseConfig = this.getBaseConfigForType(type, systemInfo, false);
    const finalConfig: QueueConfig = {
      ...baseConfig,
      ...productionDefaults,
      ...customConfig
    };
    
    return this.validateAndAdjustConfig(finalConfig, systemInfo);
  }
  
  /**
   * Get minimal configuration for testing
   */
  static createTestConfig(
    customConfig: Partial<QueueConfig> = {}
  ): QueueConfig {
    return {
      maxConcurrent: 1,
      maxRetries: 1,
      retryDelay: 100,
      timeout: 5000,
      maxQueueSize: 10,
      
      // Disable all monitoring in tests
      enableBackpressure: false,
      enableResourceMonitoring: false,
      enablePerformanceMetrics: false,
      enableEvents: false,
      enableProgressReporting: false,
      enableGarbageCollection: false,
      adaptiveDelay: false,
      
      progressUpdateInterval: 1000,
      metricsCollectionInterval: 1000,
      
      ...customConfig
    };
  }
}

interface SystemInfo {
  totalMemoryMB: number;
  freeMemoryMB: number;
  cpuCount: number;
  platform: string;
  arch: string;
  nodeVersion: string;
}
