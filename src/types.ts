import { MobileFriendlinessMetrics } from './types/enhanced-metrics';

export interface Pa11yIssue {
  code: string;
  message: string;
  type: 'error' | 'warning' | 'notice';
  selector?: string;
  context?: string;
  impact?: string;
  help?: string;
  helpUrl?: string;
}

export interface AccessibilityResult {
  url: string;
  title: string;
  imagesWithoutAlt: number;
  buttonsWithoutLabel: number;
  headingsCount: number;
  errors: string[];
  warnings: string[];
  passed: boolean;
  crashed?: boolean;  // 🆕 Technical error (browser crash, network failure, etc.)
  skipped?: boolean;  // 🆕 Skipped (redirected, not analyzed)
  duration: number;
  pa11yScore?: number;
  pa11yIssues?: any[];
  performanceMetrics?: {
    loadTime: number;
    domContentLoaded: number;
    firstPaint: number;
    renderTime: number;
    firstContentfulPaint: number;
    largestContentfulPaint: number;
    cumulativeLayoutShift?: number;
    interactionToNextPaint?: number;
    timeToFirstByte?: number;
    
    // Quality metrics
    performanceScore?: number;
    performanceGrade?: 'A' | 'B' | 'C' | 'D' | 'F';
  };
  keyboardNavigation?: string[];
  colorContrastIssues?: string[];
  focusManagementIssues?: string[];
  screenshots?: {
    desktop?: string;
    mobile?: string;
  };
  consoleErrors?: string[];
  networkErrors?: string[];
  // 🆕 Lighthouse results
  lighthouseScores?: LighthouseScores;
  lighthouseMetrics?: LighthouseMetrics;
  // 🆕 Mobile-Friendliness results
  mobileFriendliness?: MobileFriendlinessMetrics;
  // 🆕 Redirect information (not treated as error)
  redirectInfo?: {
    status?: number;
    originalUrl: string;
    finalUrl: string;
    type: 'http_redirect' | 'automatic_redirect';
  };
}

export interface TestOptions {
  maxPages?: number;
  timeout?: number;
  waitUntil?: "domcontentloaded" | "load" | "networkidle";
  filterPatterns?: string[];
  includePatterns?: string[];
  verbose?: boolean;
  output?: "console" | "json" | "html";
  outputFile?: string;
  pa11yStandard?: 'WCAG2A' | 'WCAG2AA' | 'WCAG2AAA' | 'Section508';
  hideElements?: string;
  includeNotices?: boolean;
  includeWarnings?: boolean;
  includePasses?: boolean;
  runners?: string[];
  wait?: number;
  chromeLaunchConfig?: any;
  captureScreenshots?: boolean;
  testKeyboardNavigation?: boolean;
  testColorContrast?: boolean;
  testFocusManagement?: boolean;
  collectPerformanceMetrics?: boolean;
  blockImages?: boolean;
  blockCSS?: boolean;
  mobileEmulation?: boolean;
  viewportSize?: { width: number; height: number };
  userAgent?: string;

  // 🚀 Parallel test options
  maxConcurrent?: number;              // Number of parallel workers (default: 3)
  maxRetries?: number;                 // Max. retry attempts (default: 3)
  retryDelay?: number;                 // Retry delay in ms (default: 2000)
  enableProgressBar?: boolean;         // Enable progress bar (default: true)
  progressUpdateInterval?: number;     // Progress update interval in ms (default: 1000)
  enableResourceMonitoring?: boolean;  // Enable resource monitoring (default: true)
  maxMemoryUsage?: number;             // Max. memory usage in MB (default: 512)
  maxCpuUsage?: number;                // Max. CPU usage in % (default: 80)
  useParallelTesting?: boolean;        // Enable parallel tests (default: true)
  // 🆕 Output format option
  outputFormat?: 'markdown' | 'html' | 'pdf';
  // 🆕 pa11y options
  usePa11y?: boolean;
  // 🆕 Lighthouse options
  lighthouse?: boolean;
  
  // 🎯 Event Callbacks for real-time JSON population
  eventCallbacks?: {
    onUrlStarted?: (url: string) => void;
    onUrlCompleted?: (url: string, result: AccessibilityResult, duration: number) => void;
    onUrlFailed?: (url: string, error: string, attempts: number) => void;
    onProgressUpdate?: (stats: any) => void;
    onQueueEmpty?: () => void;
  };
}

export interface LighthouseScores {
  performance: number;
  accessibility: number;
  bestPractices: number;
  seo: number;
}

export interface LighthouseMetrics {
  firstContentfulPaint: number;
  largestContentfulPaint: number;
  firstInputDelay: number;
  cumulativeLayoutShift: number;
  totalBlockingTime: number;
  speedIndex: number;
}

export interface SitemapUrl {
  loc: string;
  lastmod?: string;
  changefreq?: string;
  priority?: number;
}

export interface TestSummary {
  totalPages: number;
  testedPages: number;
  passedPages: number;
  failedPages: number;
  crashedPages: number;  // 🆕 Technical errors/crashes (not accessibility failures)
  totalErrors: number;
  totalWarnings: number;
  totalDuration: number;
  results: AccessibilityResult[];
}

/**
 * Central, unified issue interface for all report types
 */
export interface AuditIssue {
  reportType: 'accessibility' | 'security' | 'seo' | 'performance';
  pageUrl: string;
  pageTitle?: string;
  type: string;
  severity: 'error' | 'warning' | 'info';
  message: string;
  code?: string;
  selector?: string;
  context?: string;
  htmlSnippet?: string;
  lineNumber?: number;
  source?: string;
  recommendation?: string;
  resource?: string;
  score?: number;
  metric?: string;
}

/**
 * @deprecated Please use AuditIssue!
 */
export interface DetailedIssue extends AuditIssue {}

export interface AuditConfig {
  sitemap?: string;
  maxPages?: number;
  timeout?: number;
  outputDir?: string;
  standards?: string[];
  performance?: {
    enabled?: boolean;
    lighthouse?: boolean;
    coreWebVitals?: boolean;
  };
  security?: {
    enabled?: boolean;
    scanHeaders?: boolean;
    httpsCheck?: boolean;
    cspCheck?: boolean;
  };
  accessibility?: {
    enabled?: boolean;
    pa11y?: boolean;
    wcag?: string;
  };
  seo?: {
    enabled?: boolean;
    metaCheck?: boolean;
    structuredData?: boolean;
  };
  mobile?: {
    enabled?: boolean;
    touchTargets?: boolean;
    pwa?: boolean;
  };
  parallel?: {
    maxConcurrent?: number;
    maxRetries?: number;
    retryDelay?: number;
  };
  output?: {
    format?: 'markdown' | 'html' | 'json' | 'csv';
    // includeCopyButtons?: boolean; // entfernt
    includeDetails?: boolean;
  };
  logging?: {
    level?: 'debug' | 'info' | 'warn' | 'error';
    file?: string;
    verbose?: boolean;
  };
}

export interface PresetConfig {
  name: string;
  description: string;
  config: Partial<AuditConfig>;
}

export interface SecurityScanResult {
  url: string;
  timestamp: string;
  overallScore: number;
  tests: {
    securityHeaders: any;
    https: any;
    csp: any;
    vulnerability: any;
  };
  summary: {
    totalIssues: number;
    totalWarnings: number;
    criticalIssues: number;
    highIssues: number;
    mediumIssues: number;
    lowIssues: number;
  };
  recommendations: string[];
}
