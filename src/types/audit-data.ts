/**
 * 📊 FIXED AUDIT DATA STRUCTURE
 * 
 * Definitive data structure for all reports.
 * Any missing data must immediately throw an error.
 */

export interface AuditMetadata {
  version: string;
  timestamp: string;
  sitemapUrl: string;
  toolVersion: string;
  duration: number;
}

export interface AuditSummary {
  totalPages: number;
  testedPages: number;
  passedPages: number;
  failedPages: number;
  crashedPages: number;
  redirectPages?: number;
  totalErrors: number;
  totalWarnings: number;
  overallScore?: number;
  overallGrade?: string;
  certificateLevel?: string;
}

export interface AccessibilityIssue {
  code: string;
  type: 'error' | 'warning' | 'notice';
  message: string;
  context?: string;
  selector?: string;
  runner?: string;
}

export interface PageAccessibility {
  score: number;
  errors: AccessibilityIssue[];
  warnings: AccessibilityIssue[];
  notices: AccessibilityIssue[];
}

export interface PerformanceIssue {
  type: 'warning' | 'info';
  message: string;
  metric?: string;
  value?: number;
  threshold?: number;
}

export interface PagePerformance {
  score: number;
  grade: string;
  coreWebVitals: {
    largestContentfulPaint: number;
    firstContentfulPaint: number;
    cumulativeLayoutShift: number;
    timeToFirstByte: number;
  };
  metrics: {
    domContentLoaded: number;
    loadComplete: number;
    firstPaint: number;
  };
  issues?: PerformanceIssue[];
}

export interface SEOMetaTags {
  title?: string;
  description?: string;
  keywords?: string;
  ogTitle?: string;
  ogDescription?: string;
  canonical?: string;
  robots?: string;
}

export interface SEOHeadings {
  h1: string[];
  h2: string[];
  h3: string[];
  h4: string[];
  h5: string[];
  h6: string[];
}

export interface SEOImageAnalysis {
  total: number;
  withAlt: number;
  withoutAlt: number;
  missingAltImages?: string[];
}

export interface SEOIssue {
  type: 'error' | 'warning' | 'info';
  category: string;
  message: string;
  element?: string;
}

export interface PageSEO {
  score: number;
  grade: string;
  metaTags: SEOMetaTags;
  headings: SEOHeadings;
  images: SEOImageAnalysis;
  issues: SEOIssue[];
  url: string;
  title: string;
  // Enhanced SEO features
  overallSEOScore?: number;
  seoGrade?: string;
  semanticSEO?: Record<string, unknown>;
  voiceSearchOptimization?: Record<string, unknown>;
  eatAnalysis?: Record<string, unknown>;
  coreWebVitalsSEO?: Record<string, unknown>;
}

export interface ContentOptimization {
  type: string;
  message: string;
  potentialSavings?: number;
  severity?: 'low' | 'medium' | 'high';
}

export interface PageContentWeight {
  score: number;
  grade: string;
  totalSize: number;
  resources: {
    html: { size: number };
    css: { size: number; files: number };
    javascript: { size: number; files: number };
    images: { size: number; files: number };
    other: { size: number; files: number };
  };
  optimizations: ContentOptimization[];
}

export interface MobileRecommendation {
  category: string;
  message: string;
  priority?: 'low' | 'medium' | 'high';
  impact?: string;
}

export interface PageMobileFriendliness {
  overallScore: number;
  grade: string;
  recommendations: MobileRecommendation[];
}

export interface AuditPage {
  url: string;
  title: string;
  status: 'passed' | 'failed' | 'crashed';
  duration: number;
  accessibility: PageAccessibility;
  performance?: PagePerformance;
  seo?: PageSEO;
  contentWeight?: PageContentWeight;
  mobileFriendliness?: PageMobileFriendliness;
}

export interface SystemPerformance {
  testCompletionTimeSeconds: number;
  parallelProcessing: {
    pagesProcessed: number;
    concurrentWorkers: number;
    averageTimePerPageMs: number;
    throughputPagesPerMinute: number;
  };
  memoryUsage: {
    peakUsageMB: number;
    heapUsedMB: number;
    rssUsageMB: number;
    externalMB: number;
  };
  architecture: {
    eventDrivenParallel: boolean;
    comprehensiveAnalysis: boolean;
    browserPooling: boolean;
    persistenceEnabled: boolean;
  };
}

/**
 * MAIN AUDIT DATA STRUCTURE
 * This is the definitive format for all reports
 */
export interface AuditData {
  metadata: AuditMetadata;
  summary: AuditSummary;
  pages: AuditPage[];
  systemPerformance?: SystemPerformance;
}

/**
 * VALIDATION FUNCTIONS
 * Immediately throw errors if required data is missing
 */
export function validateAuditData(data: AuditData): void {
  // Validate metadata
  if (!data.metadata) throw new Error('Missing metadata in AuditData');
  if (!data.metadata.version) throw new Error('Missing metadata.version');
  if (!data.metadata.timestamp) throw new Error('Missing metadata.timestamp');
  if (!data.metadata.sitemapUrl) throw new Error('Missing metadata.sitemapUrl');
  if (!data.metadata.toolVersion) throw new Error('Missing metadata.toolVersion');
  if (typeof data.metadata.duration !== 'number') throw new Error('Missing metadata.duration');

  // Validate summary
  if (!data.summary) throw new Error('Missing summary in AuditData');
  if (typeof data.summary.totalPages !== 'number') throw new Error('Missing summary.totalPages');
  if (typeof data.summary.testedPages !== 'number') throw new Error('Missing summary.testedPages');
  if (typeof data.summary.passedPages !== 'number') throw new Error('Missing summary.passedPages');
  if (typeof data.summary.failedPages !== 'number') throw new Error('Missing summary.failedPages');
  if (typeof data.summary.crashedPages !== 'number') throw new Error('Missing summary.crashedPages');
  if (typeof data.summary.totalErrors !== 'number') throw new Error('Missing summary.totalErrors');
  if (typeof data.summary.totalWarnings !== 'number') throw new Error('Missing summary.totalWarnings');

  // Validate pages
  if (!Array.isArray(data.pages)) throw new Error('Missing or invalid pages array in AuditData');
  if (data.pages.length === 0) throw new Error('Empty pages array in AuditData');

  // Validate each page
  data.pages.forEach((page, index) => {
    if (!page.url) throw new Error(`Missing url in page ${index}`);
    if (!page.title) throw new Error(`Missing title in page ${index}`);
    if (!page.status) throw new Error(`Missing status in page ${index}`);
    if (typeof page.duration !== 'number') throw new Error(`Missing duration in page ${index}`);
    if (!page.accessibility) throw new Error(`Missing accessibility data in page ${index}`);
    if (typeof page.accessibility.score !== 'number') throw new Error(`Missing accessibility.score in page ${index}`);
    if (!Array.isArray(page.accessibility.errors)) throw new Error(`Missing accessibility.errors in page ${index}`);
    if (!Array.isArray(page.accessibility.warnings)) throw new Error(`Missing accessibility.warnings in page ${index}`);
  });

  console.log('✅ AuditData validation passed - all required data present');
}

/**
 * Check for comprehensive analysis data
 */
export function validateComprehensiveData(data: AuditData): void {
  const missingData: string[] = [];

  data.pages.forEach((page, index) => {
    if (!page.performance) {
      missingData.push(`Performance data missing in page ${index}: ${page.url}`);
    }
    if (!page.seo) {
      missingData.push(`SEO data missing in page ${index}: ${page.url}`);
    }
    if (!page.contentWeight) {
      missingData.push(`Content Weight data missing in page ${index}: ${page.url}`);
    }
    if (!page.mobileFriendliness) {
      missingData.push(`Mobile Friendliness data missing in page ${index}: ${page.url}`);
    }
  });

  if (missingData.length > 0) {
    console.error('❌ COMPREHENSIVE ANALYSIS DATA MISSING:');
    missingData.forEach(msg => console.error(`   - ${msg}`));
    throw new Error(`Comprehensive analysis failed: ${missingData.length} data points missing`);
  }

  console.log('✅ Comprehensive analysis data validation passed - all analysis types present');
}