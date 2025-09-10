/**
 * 🔧 SDK Types & Interfaces
 * 
 * Comprehensive TypeScript definitions for the AuditMySite SDK.
 * Provides type safety and IntelliSense for all SDK operations.
 */

// Local ReportFormat definition
export type ReportFormat = 'html' | 'json' | 'markdown' | 'csv';
import { TestSummary } from '../core/types';

// =============================================================================
// SDK Configuration Types
// =============================================================================

export interface SDKConfig {
  /** Default timeout for operations in milliseconds */
  timeout?: number;
  /** Maximum number of concurrent page tests */
  maxConcurrency?: number;
  /** Default output directory for reports */
  defaultOutputDir?: string;
  /** Enable verbose logging */
  verbose?: boolean;
  /** Custom user agent string */
  userAgent?: string;
  /** Proxy configuration */
  proxy?: {
    host: string;
    port: number;
    auth?: {
      username: string;
      password: string;
    };
  };
  /** Rate limiting configuration */
  rateLimit?: {
    requestsPerSecond: number;
    burstLimit: number;
  };
}

// =============================================================================
// Audit Configuration Types
// =============================================================================

export interface AuditOptions {
  /** Maximum number of pages to test */
  maxPages?: number;
  /** Accessibility standard to test against */
  standard?: 'WCAG2AA' | 'WCAG2AAA' | 'Section508';
  /** Report formats to generate */
  formats?: ReportFormat[];
  /** Output directory for reports */
  outputDir?: string;

  // Legacy include flags (kept for backward compatibility)
  /** Include performance metrics */
  includePerformance?: boolean;
  /** Include SEO analysis */
  includeSeo?: boolean;
  /** Include security checks */
  includeSecurity?: boolean;

  // New unified feature flags (enhanced methods are default in Standard)
  /** Enable Accessibility analysis (enhanced methods included by default) */
  accessibility?: boolean;
  /** Enable Performance analysis (Core Web Vitals, timing) */
  performance?: boolean;
  /** Enable SEO analysis */
  seo?: boolean;
  /** Enable Content Weight analysis */
  contentWeight?: boolean;
  /** Use reduced mode to limit checks and speed up audits */
  reduced?: boolean;
  /** Include actionable recommendations in results */
  includeRecommendations?: boolean;

  /** Use Pa11y for accessibility testing */
  usePa11y?: boolean;
  /** Custom Pa11y options */
  pa11yOptions?: {
    ignore?: string[];
    rules?: string[];
    timeout?: number;
    wait?: number;
  };
  /** Performance budget configuration */
  performanceBudget?: {
    lcp?: { good: number; poor: number };
    cls?: { good: number; poor: number };
    fid?: { good: number; poor: number };
    inp?: { good: number; poor: number };
    ttfb?: { good: number; poor: number };
  };
  /** Custom viewport settings */
  viewport?: {
    width: number;
    height: number;
    deviceScaleFactor?: number;
    isMobile?: boolean;
  };
}

// =============================================================================
// Event System Types
// =============================================================================

export type AuditEventType = 
  | 'audit:start'
  | 'audit:progress'
  | 'audit:page:start'
  | 'audit:page:complete'
  | 'audit:page:error'
  | 'audit:complete'
  | 'audit:error'
  | 'report:start'
  | 'report:complete';

export interface AuditEvent<T = any> {
  type: AuditEventType;
  timestamp: Date;
  data: T;
  sessionId?: string;
}

export interface ProgressData {
  current: number;
  total: number;
  percentage: number;
  currentUrl?: string;
  estimatedTimeRemaining?: number;
}

export interface PageStartData {
  url: string;
  index: number;
  total: number;
}

export interface PageCompleteData {
  url: string;
  index: number;
  total: number;
  result: PageAuditResult;
  duration: number;
}

export interface PageErrorData {
  url: string;
  index: number;
  total: number;
  error: Error;
  duration: number;
}

// =============================================================================
// Result Types
// =============================================================================

export interface PageAuditResult {
  url: string;
  title: string;
  passed: boolean;
  crashed: boolean;
  errors: string[];
  warnings: string[];
  duration: number;
  timestamp: string;
  performanceMetrics?: {
    largestContentfulPaint?: number;
    cumulativeLayoutShift?: number;
    firstInputDelay?: number;
    timeToInteractive?: number;
    firstContentfulPaint?: number;
    speedIndex?: number;
  };
  pa11yIssues?: {
    type: 'error' | 'warning' | 'notice';
    message: string;
    selector?: string;
    context?: string;
    code?: string;
    runner?: string;
  }[];
  seoMetrics?: {
    title?: string;
    description?: string;
    h1Count?: number;
    imagesMissingAlt?: number;
    internalLinks?: number;
    externalLinks?: number;
  };
  securityIssues?: {
    type: string;
    severity: 'low' | 'medium' | 'high' | 'critical';
    message: string;
    recommendation?: string;
  }[];
}

export interface AuditResult {
  sessionId: string;
  sitemapUrl: string;
  startTime: Date;
  endTime: Date;
  duration: number;
  summary: TestSummary;
  results: PageAuditResult[];
  reports: GeneratedReport[];
  metadata: {
    version: string;
    environment: string;
    userAgent: string;
    configuration: AuditOptions;
  };
}

export interface GeneratedReport {
  format: ReportFormat;
  path: string;
  size: number;
  url?: string; // For web-accessible reports
  metadata: {
    generatedAt: Date;
    duration: number;
  };
}

// =============================================================================
// Callback Types
// =============================================================================

export type EventCallback<T = any> = (event: AuditEvent<T>) => void | Promise<void>;

export interface AuditCallbacks {
  onStart?: EventCallback;
  onProgress?: EventCallback<ProgressData>;
  onPageStart?: EventCallback<PageStartData>;
  onPageComplete?: EventCallback<PageCompleteData>;
  onPageError?: EventCallback<PageErrorData>;
  onComplete?: EventCallback<AuditResult>;
  onError?: EventCallback<Error>;
  onReportStart?: EventCallback<{ format: ReportFormat }>;
  onReportComplete?: EventCallback<GeneratedReport>;
}

// =============================================================================
// API Types
// =============================================================================

export interface APIConfig {
  /** Base URL for the API */
  baseUrl?: string;
  /** API key for authentication */
  apiKey?: string;
  /** Request timeout in milliseconds */
  timeout?: number;
  /** Retry configuration */
  retries?: {
    count: number;
    delay: number;
    backoff: number;
  };
}

export interface APIResponse<T = any> {
  success: boolean;
  data?: T;
  error?: {
    code: string;
    message: string;
    details?: any;
  };
  meta?: {
    timestamp: string;
    version: string;
    requestId: string;
  };
}

export interface AuditJobRequest {
  sitemapUrl: string;
  options?: AuditOptions;
  callbacks?: {
    webhookUrl?: string;
    events?: AuditEventType[];
  };
  priority?: 'low' | 'normal' | 'high';
}

export interface AuditJob {
  id: string;
  status: 'pending' | 'running' | 'completed' | 'failed' | 'cancelled';
  sitemapUrl: string;
  options: AuditOptions;
  createdAt: Date;
  startedAt?: Date;
  completedAt?: Date;
  progress?: ProgressData;
  result?: AuditResult;
  error?: string;
}

// =============================================================================
// Builder Pattern Types
// =============================================================================

export interface AuditBuilder {
  sitemap(url: string): AuditBuilder;
  maxPages(count: number): AuditBuilder;
  standard(standard: 'WCAG2AA' | 'WCAG2AAA' | 'Section508'): AuditBuilder;
  formats(formats: ReportFormat[]): AuditBuilder;
  outputDir(dir: string): AuditBuilder;
  includePerformance(include?: boolean): AuditBuilder;
  includeSeo(include?: boolean): AuditBuilder;
  includeSecurity(include?: boolean): AuditBuilder;
  viewport(width: number, height: number, mobile?: boolean): AuditBuilder;
  timeout(ms: number): AuditBuilder;
  on<T>(event: AuditEventType, callback: EventCallback<T>): AuditBuilder;
  run(): Promise<AuditResult>;
}

// =============================================================================
// Utility Types
// =============================================================================

export type DeepPartial<T> = {
  [P in keyof T]?: T[P] extends object ? DeepPartial<T[P]> : T[P];
};

export type RequiredKeys<T, K extends keyof T> = Omit<T, K> & Required<Pick<T, K>>;

export type OptionalKeys<T, K extends keyof T> = Omit<T, K> & Partial<Pick<T, K>>;

// =============================================================================
// Error Types
// =============================================================================

export class AuditSDKError extends Error {
  constructor(
    message: string,
    public code: string,
    public details?: any
  ) {
    super(message);
    this.name = 'AuditSDKError';
  }
}

export class AuditTimeoutError extends AuditSDKError {
  constructor(timeout: number) {
    super(`Audit timed out after ${timeout}ms`, 'TIMEOUT');
  }
}

export class InvalidSitemapError extends AuditSDKError {
  constructor(url: string, reason: string) {
    super(`Invalid sitemap URL: ${url} - ${reason}`, 'INVALID_SITEMAP', { url, reason });
  }
}

export class ConfigurationError extends AuditSDKError {
  constructor(message: string, invalidConfig?: any) {
    super(`Configuration error: ${message}`, 'INVALID_CONFIG', invalidConfig);
  }
}
