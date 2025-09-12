/**
 * Complete TypeScript definitions for AuditMySite Tauri Desktop App Integration
 * 
 * This file contains all type definitions needed for seamless integration between
 * AuditMySite CLI engine and the Tauri desktop application.
 */

import { Html5ElementsAnalysis } from '../core/accessibility/html5-elements-checker';
import { AriaAnalysisResults } from '../core/accessibility/aria-rules-analyzer';
import { PerformanceOptimizationResults } from '../core/performance/chrome135-optimizer';
import { StreamEvent, StreamingConfiguration } from '../core/reporting/streaming-reporter';

// Simple Enhanced Report Summary interface (replaces deleted enhanced-report-generator)
interface EnhancedReportSummary {
  testedPages: number;
  passedPages: number;
  failedPages: number;
  totalErrors: number;
  totalWarnings: number;
  avgAccessibilityScore: number;
  avgPerformanceScore: number;
}

// Re-export streaming types
export * from '../core/reporting/streaming-reporter';
import type { InitEvent, PageResultEvent, SummaryEvent, CompleteEvent, ErrorEvent, ProgressEvent } from '../core/reporting/streaming-reporter';

/**
 * Complete configuration for AuditMySite testing
 */
export interface AuditConfiguration {
  /** Target sitemap URL */
  sitemapUrl: string;
  
  /** Basic settings */
  maxPages: number;
  timeout: number;
  wait: number;
  format: 'html' | 'markdown' | 'json';
  outputDir: string;
  
  /** v1.3 Enhanced Features */
  enhancedFeatures: {
    modernHtml5: boolean;
    ariaEnhanced: boolean;
    chrome135Features: boolean;
    semanticAnalysis: boolean;
  };
  
  /** Advanced options */
  advanced: {
    concurrency: number;
    standard: 'WCAG2AA' | 'WCAG2AAA' | 'Section508';
    includePerformance: boolean;
    verbose: boolean;
  };
  
  /** Output preferences */
  streaming: StreamingConfiguration;
}

/**
 * Real-time audit results with streaming support
 */
export interface StreamingAuditResult {
  /** Unique audit session ID */
  sessionId: string;
  
  /** Current state */
  state: 'initializing' | 'parsing' | 'testing' | 'reporting' | 'complete' | 'error';
  
  /** Progress tracking */
  progress: {
    current: number;
    total: number;
    percentage: number;
    currentUrl?: string;
    stage: string;
    timeElapsed: number;
    estimatedRemaining?: number;
  };
  
  /** Accumulated results */
  results: {
    completed: EnhancedAccessibilityResult[];
    summary?: EnhancedReportSummary;
    errors: AuditError[];
  };
  
  /** Real-time metrics */
  realTimeMetrics: {
    pagesPerSecond: number;
    memoryUsage: number;
    cpuUsage?: number;
  };
}

/**
 * Enhanced accessibility result with streaming metadata
 */
export interface EnhancedAccessibilityResult {
  /** Basic result properties */
  url: string;
  title: string;
  passed: boolean;
  duration: number;
  
  /** Error and warning counts */
  errors: string[];
  warnings: string[];
  
  /** Traditional accessibility metrics */
  imagesWithoutAlt: number;
  buttonsWithoutLabel: number;
  headingsCount: number;
  
  /** v1.3 Enhanced Analysis */
  html5Analysis?: Html5ElementsAnalysis;
  ariaAnalysis?: AriaAnalysisResults;
  chrome135Optimizations?: PerformanceOptimizationResults;
  semanticScore?: number;
  
  /** Performance results */
  performanceResults?: {
    score: number;
    grade: string;
    metrics: {
      fcp?: number;
      lcp?: number;
      cls?: number;
      inp?: number;
      ttfb?: number;
    };
  };
  
  /** pa11y integration */
  pa11yIssues?: Pa11yIssue[];
  pa11yScore?: number;
  
  /** Compliance assessment */
  complianceLevel?: 'basic' | 'enhanced' | 'comprehensive';
  futureReadiness?: number;
  modernFeaturesDetected?: string[];
  enhancedRecommendations?: string[];
  
  /** Streaming metadata */
  streamingMeta?: {
    processedAt: string;
    processingTime: number;
    chunkId: string;
    sequenceNumber: number;
  };
}

/**
 * Pa11y issue structure
 */
export interface Pa11yIssue {
  code: string;
  message: string;
  type: 'error' | 'warning' | 'notice';
  selector: string;
  context?: string;
  impact?: string;
  help?: string;
  helpUrl?: string;
}

/**
 * Audit error information
 */
export interface AuditError {
  message: string;
  url?: string;
  stage: string;
  timestamp: string;
  recoverable: boolean;
  stack?: string;
}

/**
 * Sitemap validation result
 */
export interface SitemapValidation {
  valid: boolean;
  accessible: boolean;
  urlCount: number;
  errors: string[];
  warnings: string[];
  estimatedTime?: number;
}

/**
 * Export format options
 */
export type ExportFormat = 'html' | 'markdown' | 'json' | 'pdf' | 'csv';

/**
 * Pagination options for result retrieval
 */
export interface PaginationOptions {
  page: number;
  limit: number;
  sortBy?: 'url' | 'score' | 'errors' | 'timestamp';
  sortOrder?: 'asc' | 'desc';
  filter?: {
    passed?: boolean;
    complianceLevel?: 'basic' | 'enhanced' | 'comprehensive';
    minScore?: number;
    maxScore?: number;
    hasErrors?: boolean;
  };
}

/**
 * Audit session information
 */
export interface AuditSession {
  id: string;
  config: AuditConfiguration;
  state: StreamingAuditResult['state'];
  startTime: string;
  endTime?: string;
  progress: StreamingAuditResult['progress'];
  results: EnhancedAccessibilityResult[];
  summary?: EnhancedReportSummary;
  errors: AuditError[];
}

/**
 * Tauri command definitions for Rust backend
 */
export interface TauriCommands {
  // Audit management
  start_audit(config: AuditConfiguration): Promise<string>;
  stop_audit(sessionId: string): Promise<void>;
  pause_audit(sessionId: string): Promise<void>;
  resume_audit(sessionId: string): Promise<void>;
  get_audit_status(sessionId: string): Promise<StreamingAuditResult>;
  
  // Data access
  get_page_results(sessionId: string, options?: PaginationOptions): Promise<{
    results: EnhancedAccessibilityResult[];
    total: number;
    page: number;
    hasMore: boolean;
  }>;
  get_audit_summary(sessionId: string): Promise<EnhancedReportSummary>;
  get_session_info(sessionId: string): Promise<AuditSession>;
  list_sessions(): Promise<AuditSession[]>;
  
  // Export functionality
  export_results(sessionId: string, format: ExportFormat, path: string): Promise<string>;
  export_individual_result(sessionId: string, url: string, format: ExportFormat, path: string): Promise<string>;
  
  // Configuration management
  validate_sitemap(url: string): Promise<SitemapValidation>;
  get_default_config(): Promise<AuditConfiguration>;
  save_config(config: Partial<AuditConfiguration>): Promise<void>;
  load_config(): Promise<AuditConfiguration>;
  
  // System utilities
  get_system_info(): Promise<SystemInfo>;
  cleanup_sessions(): Promise<number>;
  get_app_version(): Promise<string>;
}

/**
 * System information
 */
export interface SystemInfo {
  platform: string;
  arch: string;
  nodeVersion: string;
  chromeVersion?: string;
  memory: {
    total: number;
    free: number;
    used: number;
  };
  cpu: {
    cores: number;
    model: string;
  };
}

/**
 * Real-time event subscriptions for Tauri frontend
 */
export interface TauriEvents {
  // Progress events
  'audit:init': InitEvent;
  'audit:progress': ProgressEvent;
  'audit:page_complete': PageResultEvent;
  'audit:summary': SummaryEvent;
  'audit:error': ErrorEvent;
  'audit:complete': CompleteEvent;
  
  // System events
  'audit:memory_warning': MemoryWarningEvent;
  'audit:performance_update': PerformanceUpdateEvent;
  'audit:session_update': SessionUpdateEvent;
}

/**
 * Memory warning event
 */
export interface MemoryWarningEvent {
  type: 'memory_warning';
  sessionId: string;
  timestamp: string;
  data: {
    currentUsage: number;
    maxUsage: number;
    warningLevel: 'low' | 'medium' | 'high';
    recommendation: string;
  };
}

/**
 * Performance update event
 */
export interface PerformanceUpdateEvent {
  type: 'performance_update';
  sessionId: string;
  timestamp: string;
  data: {
    pagesPerSecond: number;
    memoryUsage: number;
    cpuUsage?: number;
    networkLatency?: number;
  };
}

/**
 * Session update event
 */
export interface SessionUpdateEvent {
  type: 'session_update';
  sessionId: string;
  timestamp: string;
  data: {
    sessionId: string;
    state: StreamingAuditResult['state'];
    message: string;
  };
}

/**
 * Application configuration
 */
export interface AppConfiguration {
  /** Default audit settings */
  defaultAudit: Partial<AuditConfiguration>;
  
  /** UI preferences */
  ui: {
    theme: 'light' | 'dark' | 'system';
    language: string;
    showAdvancedOptions: boolean;
    autoSave: boolean;
  };
  
  /** Performance settings */
  performance: {
    maxConcurrentSessions: number;
    memoryLimitMB: number;
    cacheResults: boolean;
    cleanupInterval: number; // hours
  };
  
  /** Export settings */
  export: {
    defaultFormat: ExportFormat;
    defaultPath: string;
    includeScreenshots: boolean;
    compressReports: boolean;
  };
}

/**
 * Batch audit configuration for multiple sites
 */
export interface BatchAuditConfiguration {
  name: string;
  sitemaps: Array<{
    name: string;
    url: string;
    config?: Partial<AuditConfiguration>;
  }>;
  globalConfig: AuditConfiguration;
  outputDir: string;
  schedule?: {
    enabled: boolean;
    cron: string;
    timezone: string;
  };
}

/**
 * Historical audit data for comparison
 */
export interface AuditHistory {
  sessionId: string;
  timestamp: string;
  config: AuditConfiguration;
  summary: EnhancedReportSummary;
  duration: number;
  version: string;
}

/**
 * Comparison result between two audits
 */
export interface AuditComparison {
  baseline: AuditHistory;
  current: AuditHistory;
  changes: {
    accessibility: {
      scoreDiff: number;
      newIssues: number;
      resolvedIssues: number;
    };
    html5: {
      scoreDiff: number;
      newElements: string[];
      improvedElements: string[];
    };
    aria: {
      scoreDiff: number;
      impactChanges: {
        critical: number;
        serious: number;
        moderate: number;
        minor: number;
      };
    };
    performance: {
      scoreDiff: number;
      metricsChanges: Record<string, number>;
    };
  };
  summary: string;
  recommendations: string[];
}

/**
 * Main export interface combining all types for external consumption
 */
export interface TauriIntegrationTypes {
  // Configuration
  AuditConfiguration: AuditConfiguration;
  AppConfiguration: AppConfiguration;
  BatchAuditConfiguration: BatchAuditConfiguration;
  StreamingConfiguration: StreamingConfiguration;
  
  // Results
  StreamingAuditResult: StreamingAuditResult;
  EnhancedAccessibilityResult: EnhancedAccessibilityResult;
  EnhancedReportSummary: EnhancedReportSummary;
  AuditSession: AuditSession;
  
  // Events
  StreamEvent: StreamEvent;
  TauriEvents: TauriEvents;
  
  // Commands
  TauriCommands: TauriCommands;
  
  // Utilities
  SitemapValidation: SitemapValidation;
  PaginationOptions: PaginationOptions;
  SystemInfo: SystemInfo;
  AuditHistory: AuditHistory;
  AuditComparison: AuditComparison;
}

/**
 * Default configurations for common use cases
 */
export const DEFAULT_CONFIGURATIONS = {
  /** Quick test configuration (default) */
  QUICK: {
    sitemapUrl: '',
    outputDir: './reports',
    maxPages: 5,
    timeout: 15000,
    wait: 1000,
    format: 'html' as const,
    enhancedFeatures: {
      modernHtml5: true,
      ariaEnhanced: true,
      chrome135Features: true,
      semanticAnalysis: true,
    },
    advanced: {
      concurrency: 2,
      standard: 'WCAG2AA' as const,
      includePerformance: true,
      verbose: false,
    },
    streaming: {
      enabled: true,
      chunkSize: 10,
      bufferTimeout: 1000,
      includeDetailedResults: true,
      compressResults: false,
    },
  },
  
  /** Comprehensive test configuration */
  COMPREHENSIVE: {
    sitemapUrl: '',
    outputDir: './reports',
    maxPages: 50,
    timeout: 30000,
    wait: 3000,
    format: 'html' as const,
    enhancedFeatures: {
      modernHtml5: true,
      ariaEnhanced: true,
      chrome135Features: true,
      semanticAnalysis: true,
    },
    advanced: {
      concurrency: 3,
      standard: 'WCAG2AA' as const,
      includePerformance: true,
      verbose: true,
    },
    streaming: {
      enabled: true,
      chunkSize: 10,
      bufferTimeout: 1000,
      includeDetailedResults: true,
      compressResults: false,
    },
  },
  
  /** CI/CD configuration */
  CI_CD: {
    sitemapUrl: '',
    outputDir: './reports',
    maxPages: 20,
    timeout: 20000,
    wait: 2000,
    format: 'markdown' as const,
    enhancedFeatures: {
      modernHtml5: true,
      ariaEnhanced: true,
      chrome135Features: false, // Might not be available in CI
      semanticAnalysis: true,
    },
    advanced: {
      concurrency: 1,
      standard: 'WCAG2AA' as const,
      includePerformance: true,
      verbose: false,
    },
    streaming: {
      enabled: true,
      chunkSize: 10,
      bufferTimeout: 1000,
      includeDetailedResults: true,
      compressResults: false,
    },
  },
} as const;

/**
 * Helper functions for type guards and validation
 */
export namespace TauriIntegrationUtils {
  export function isStreamEvent(obj: unknown): obj is StreamEvent {
    return (
      typeof obj === 'object' &&
      obj !== null &&
      'type' in obj &&
      'sessionId' in obj &&
      'timestamp' in obj &&
      'data' in obj
    );
  }
  
  export function isProgressEvent(event: StreamEvent): event is import('../core/reporting/streaming-reporter').ProgressEvent {
    return event.type === 'progress';
  }
  
  export function isPageResultEvent(event: StreamEvent): event is import('../core/reporting/streaming-reporter').PageResultEvent {
    return event.type === 'page_result';
  }
  
  export function isCompleteEvent(event: StreamEvent): event is import('../core/reporting/streaming-reporter').CompleteEvent {
    return event.type === 'complete';
  }
  
  export function isErrorEvent(event: StreamEvent): event is import('../core/reporting/streaming-reporter').ErrorEvent {
    return event.type === 'error';
  }
  
  export function validateAuditConfiguration(config: Partial<AuditConfiguration>): config is AuditConfiguration {
    return !!(
      config.sitemapUrl &&
      config.maxPages &&
      config.timeout &&
      config.enhancedFeatures &&
      config.advanced
    );
  }
  
  export function createDefaultConfig(baseUrl: string): AuditConfiguration {
    return {
      ...DEFAULT_CONFIGURATIONS.QUICK,
      sitemapUrl: baseUrl,
      outputDir: './reports',
    };
  }
}
