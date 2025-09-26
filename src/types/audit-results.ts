/**
 * 🎯 Shared TypeScript Types for AuditMySite v2.0
 * 
 * These types are the single source of truth for:
 * 1. CLI JSON export (FullAuditResult)
 * 2. HTML report generation (reads JSON, uses same types)
 * 3. API responses (returns sub-types like SitemapResult, AccessibilityResult)
 * 
 * Architecture: Strict typing ensures consistency across all outputs
 */

/**
 * Complete audit result - exported as JSON by CLI tool
 */
export interface FullAuditResult {
  /** Audit metadata and configuration */
  metadata: AuditMetadata;
  /** Sitemap parsing results */
  sitemap: SitemapResult;
  /** Individual page audit results */
  pages: PageAuditResult[];
  /** Summary statistics across all pages */
  summary: AuditSummary;
}

/**
 * Metadata about the audit execution
 */
export interface AuditMetadata {
  /** ISO timestamp when audit was started */
  timestamp: string;
  /** Audit result format version */
  version: string;
  /** Original sitemap URL that was audited */
  sitemapUrl: string;
  /** Total audit duration in milliseconds */
  duration: number;
  /** AuditMySite tool version */
  toolVersion: string;
  /** Audit configuration used */
  config: AuditConfig;
}

/**
 * Audit configuration options
 */
export interface AuditConfig {
  /** Maximum number of pages audited */
  maxPages: number;
  /** Whether full analysis was used */
  fullAnalysis: boolean;
  /** WCAG standard applied */
  pa11yStandard: 'WCAG2A' | 'WCAG2AA' | 'WCAG2AAA' | 'Section508';
  /** Analysis types enabled */
  analysisTypes: {
    accessibility: boolean;
    performance: boolean;
    seo: boolean;
    contentWeight: boolean;
  };
}

/**
 * Sitemap parsing results (API endpoint: GET /api/sitemap/:domain)
 */
export interface SitemapResult {
  /** Original sitemap URL */
  sourceUrl: string;
  /** Successfully parsed URLs */
  urls: string[];
  /** When the sitemap was parsed */
  parsedAt: string;
  /** Number of URLs found */
  totalUrls: number;
  /** Number of URLs filtered out */
  filteredUrls: number;
  /** Applied filter patterns */
  filterPatterns: string[];
}

/**
 * Complete audit result for a single page
 */
export interface PageAuditResult {
  /** Page URL */
  url: string;
  /** Page title (if available) */
  title?: string;
  /** Overall page status */
  status: 'passed' | 'failed' | 'crashed';
  /** Page load duration in milliseconds */
  duration: number;
  /** When this page was audited */
  auditedAt: string;
  
  /** Accessibility audit results */
  accessibility: AccessibilityResult;
  /** Performance audit results (optional) */
  performance?: PerformanceResult;
  /** SEO audit results (optional) */
  seo?: SEOResult;
  /** Content weight analysis (optional) */
  contentWeight?: ContentWeightResult;
  /** Mobile friendliness analysis (optional) */
  mobileFriendliness?: MobileFriendlinessResult;
}

/**
 * Accessibility audit result (API endpoint: POST /api/page/accessibility)
 */
export interface AccessibilityResult {
  /** Overall accessibility status */
  passed: boolean;
  /** WCAG compliance level achieved */
  wcagLevel: 'A' | 'AA' | 'AAA' | 'none';
  /** Accessibility score (0-100) */
  score: number;
  /** Critical accessibility errors */
  errors: AccessibilityIssue[];
  /** Accessibility warnings */
  warnings: AccessibilityIssue[];
  /** Accessibility notices (optional) */
  notices?: AccessibilityIssue[];
  /** Pa11y-specific results */
  pa11yResults: {
    /** Total issues found by pa11y */
    totalIssues: number;
    /** Pa11y runner version */
    runner: string;
  };
  /** Basic accessibility checks with counts */
  basicChecks?: {
    /** Number of images without alt attribute */
    imagesWithoutAlt: number;
    /** Number of buttons without aria-label */
    buttonsWithoutLabel: number;
    /** Total number of headings found */
    headingsCount: number;
    /** Number of potential color contrast issues */
    contrastIssues?: number;
  };
  /** Comprehensive WCAG 2.1 analysis results */
  wcagAnalysis?: {
    /** Perceivable principle compliance */
    perceivable: {
      colorContrast: { violations: number; score: number };
      textAlternatives: { violations: number; score: number };
      captions: { violations: number; score: number };
      adaptable: { violations: number; score: number };
    };
    /** Operable principle compliance */
    operable: {
      keyboardAccessible: { violations: number; score: number };
      seizures: { violations: number; score: number };
      navigable: { violations: number; score: number };
      inputModalities: { violations: number; score: number };
    };
    /** Understandable principle compliance */
    understandable: {
      readable: { violations: number; score: number };
      predictable: { violations: number; score: number };
      inputAssistance: { violations: number; score: number };
    };
    /** Robust principle compliance */
    robust: {
      compatible: { violations: number; score: number };
      parsing: { violations: number; score: number };
    };
  };
  /** ARIA implementation analysis */
  ariaAnalysis?: {
    /** Total ARIA violations */
    totalViolations: number;
    /** ARIA landmarks usage */
    landmarks: { present: string[]; missing: string[]; score: number };
    /** ARIA roles implementation */
    roles: { correct: number; incorrect: number; missing: number; score: number };
    /** ARIA properties and states */
    properties: { correct: number; incorrect: number; missing: number; score: number };
    /** Live regions implementation */
    liveRegions: { present: number; appropriate: number; score: number };
  };
  /** Form accessibility analysis */
  formAnalysis?: {
    /** Total form elements analyzed */
    totalElements: number;
    /** Form labeling issues */
    labeling: { proper: number; missing: number; inadequate: number; score: number };
    /** Form validation and error handling */
    validation: { accessible: number; inaccessible: number; score: number };
    /** Form focus management */
    focusManagement: { proper: number; issues: number; score: number };
  };
  /** Keyboard navigation analysis */
  keyboardAnalysis?: {
    /** Focus indicators */
    focusIndicators: { visible: number; missing: number; score: number };
    /** Tab order analysis */
    tabOrder: { logical: number; problematic: number; score: number };
    /** Keyboard traps */
    keyboardTraps: { detected: number; score: number };
  };
}

/**
 * Individual accessibility issue
 */
export interface AccessibilityIssue {
  /** Issue severity */
  severity: 'error' | 'warning' | 'notice';
  /** Issue message */
  message: string;
  /** WCAG rule code */
  code?: string;
  /** CSS selector where issue was found */
  selector?: string;
  /** Context/element content */
  context?: string;
  /** WCAG guideline reference */
  guideline?: string;
}

/**
 * Performance audit result (API endpoint: POST /api/page/performance)
 */
export interface PerformanceResult {
  /** Overall performance score (0-100) */
  score: number;
  /** Performance grade (A-F) */
  grade: 'A' | 'B' | 'C' | 'D' | 'F';
  /** Core Web Vitals */
  coreWebVitals: {
    /** Largest Contentful Paint in ms */
    largestContentfulPaint: number;
    /** First Contentful Paint in ms */
    firstContentfulPaint: number;
    /** Cumulative Layout Shift */
    cumulativeLayoutShift: number;
    /** Interaction to Next Paint in ms (if available) */
    interactionToNextPaint?: number;
    /** Time to First Byte in ms */
    timeToFirstByte: number;
  };
  /** Additional performance metrics */
  metrics: {
    /** DOM content loaded time in ms */
    domContentLoaded: number;
    /** Page load complete time in ms */
    loadComplete: number;
    /** First paint time in ms */
    firstPaint?: number;
  };
  /** Performance issues identified */
  issues: PerformanceIssue[];
}

/**
 * Performance issue identified
 */
export interface PerformanceIssue {
  /** Issue type */
  type: 'lcp-slow' | 'fcp-slow' | 'cls-high' | 'ttfb-slow';
  /** Issue severity */
  severity: 'error' | 'warning';
  /** Issue description */
  message: string;
  /** Measured value */
  value: number;
  /** Recommended threshold */
  threshold: number;
}

/**
 * SEO audit result (API endpoint: POST /api/page/seo)
 */
export interface SEOResult {
  /** Overall SEO score (0-100) */
  score: number;
  /** SEO grade (A-F) */
  grade: 'A' | 'B' | 'C' | 'D' | 'F';
  /** Meta tag analysis */
  metaTags: {
    /** Page title */
    title?: {
      content: string;
      length: number;
      optimal: boolean;
    };
    /** Meta description */
    description?: {
      content: string;
      length: number;
      optimal: boolean;
    };
    /** Canonical URL */
    canonical?: string;
    /** Open Graph tags */
    openGraph: Record<string, string>;
    /** Twitter Card tags */
    twitterCard: Record<string, string>;
  };
  /** Heading structure analysis */
  headings: {
    /** H1 tags */
    h1: string[];
    /** H2 tags */
    h2: string[];
    /** H3 tags */
    h3: string[];
    /** Heading structure issues */
    issues: string[];
  };
  /** Image analysis */
  images: {
    /** Total images found */
    total: number;
    /** Images missing alt text */
    missingAlt: number;
    /** Images with empty alt text */
    emptyAlt: number;
  };
  /** SEO issues found */
  issues: SEOIssue[];
}

/**
 * SEO issue identified
 */
export interface SEOIssue {
  /** Issue type */
  type: 'title-missing' | 'title-long' | 'description-missing' | 'description-long' | 'h1-missing' | 'image-alt-missing';
  /** Issue severity */
  severity: 'error' | 'warning';
  /** Issue message */
  message: string;
  /** Element selector (if applicable) */
  selector?: string;
}

/**
 * Content weight analysis result (API endpoint: POST /api/page/content-weight)
 */
export interface ContentWeightResult {
  /** Overall content weight score (0-100) */
  score: number;
  /** Content weight grade (A-F) */
  grade: 'A' | 'B' | 'C' | 'D' | 'F';
  /** Total page size in bytes */
  totalSize: number;
  /** Resource breakdown */
  resources: {
    /** HTML size */
    html: { size: number; gzipped?: number };
    /** CSS size */
    css: { size: number; gzipped?: number; files: number };
    /** JavaScript size */
    javascript: { size: number; gzipped?: number; files: number };
    /** Images size */
    images: { size: number; files: number };
    /** Other resources */
    other: { size: number; files: number };
  };
  /** Content optimization recommendations */
  optimizations: ContentOptimization[];
}

/**
 * Content optimization recommendation
 */
export interface ContentOptimization {
  /** Optimization type */
  type: 'compress-images' | 'minify-css' | 'minify-js' | 'enable-gzip' | 'reduce-requests';
  /** Potential savings in bytes */
  savings: number;
  /** Recommendation description */
  message: string;
  /** Priority level */
  priority: 'high' | 'medium' | 'low';
}

/**
 * Summary statistics across all audited pages
 */
export interface AuditSummary {
  /** Total number of pages discovered in sitemap */
  totalPages: number;
  /** Number of pages actually tested */
  testedPages: number;
  /** Number of pages that passed all tests */
  passedPages: number;
  /** Number of pages that failed tests */
  failedPages: number;
  /** Number of pages that crashed during testing */
  crashedPages: number;
  /** Total accessibility errors across all pages */
  totalErrors: number;
  /** Total accessibility warnings across all pages */
  totalWarnings: number;
  /** Average scores across all pages */
  averageScores: {
    accessibility: number;
    performance?: number;
    seo?: number;
    contentWeight?: number;
  };
  /** Overall quality grades */
  overallGrades: {
    accessibility: 'A' | 'B' | 'C' | 'D' | 'F';
    performance?: 'A' | 'B' | 'C' | 'D' | 'F';
    seo?: 'A' | 'B' | 'C' | 'D' | 'F';
    contentWeight?: 'A' | 'B' | 'C' | 'D' | 'F';
  };
}

/**
 * Structured issue for easy processing (used in issues array of FullAuditResult)
 */
export interface StructuredIssue {
  /** Issue type category */
  type: 'accessibility' | 'performance' | 'seo' | 'content-weight';
  /** Issue severity */
  severity: 'error' | 'warning' | 'notice';
  /** Issue message */
  message: string;
  /** URL where issue was found */
  url: string;
  /** CSS selector (if applicable) */
  selector?: string;
  /** Rule/check code */
  code?: string;
  /** Additional context */
  context?: string;
}

/**
 * Helper type for API responses - all individual audit result types
 */
export type AuditResultTypes = SitemapResult | AccessibilityResult | PerformanceResult | SEOResult | ContentWeightResult;

/**
 * Helper type for grading scores
 */
export type Grade = 'A' | 'B' | 'C' | 'D' | 'F';

/**
 * Mobile friendliness audit result
 */
export interface MobileFriendlinessResult {
  /** Overall mobile friendliness score (0-100) */
  overallScore: number;
  /** Mobile friendliness grade (A-F) */
  grade: 'A' | 'B' | 'C' | 'D' | 'F';
  /** Mobile optimization recommendations */
  recommendations: MobileFriendlinessRecommendation[];
}

/**
 * Mobile friendliness recommendation
 */
export interface MobileFriendlinessRecommendation {
  /** Recommendation category */
  category: 'viewport' | 'typography' | 'touchTargets' | 'navigation' | 'media' | 'performance' | 'forms' | 'ux';
  /** Recommendation priority */
  priority: 'high' | 'medium' | 'low';
  /** Issue description */
  issue: string;
  /** Recommended action */
  recommendation: string;
  /** Expected impact */
  impact: string;
}

/**
 * Helper function to calculate grade from score
 */
export function calculateGrade(score: number): Grade {
  if (score >= 90) return 'A';
  if (score >= 80) return 'B';
  if (score >= 70) return 'C';
  if (score >= 60) return 'D';
  return 'F';
}

/**
 * Helper function to calculate overall score from individual scores
 */
export function calculateOverallScore(scores: Record<string, number>): number {
  const values = Object.values(scores).filter(score => score !== undefined);
  return values.length > 0 ? Math.round(values.reduce((a, b) => a + b, 0) / values.length) : 0;
}
