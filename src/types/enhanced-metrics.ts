/**
 * 🚀 Enhanced Performance & SEO Metrics Types
 * 
 * Extended metrics for comprehensive website analysis including:
 * - Content weight analysis
 * - Enhanced performance metrics  
 * - Comprehensive SEO analysis
 * - Resource timing and network analysis
 */

export interface ContentWeight {
  /** HTML content size in bytes */
  html: number;
  /** CSS content size in bytes */
  css: number;
  /** JavaScript content size in bytes */
  javascript: number;
  /** Images total size in bytes */
  images: number;
  /** Fonts total size in bytes */
  fonts: number;
  /** Other assets size in bytes */
  other: number;
  /** Total uncompressed size in bytes */
  total: number;
  /** Total compressed/transferred size in bytes */
  gzipTotal?: number;
  /** Compression ratio (0-1) */
  compressionRatio?: number;
}

export interface ContentAnalysis {
  /** Text content length in characters */
  textContent: number;
  /** Number of images on the page */
  imageCount: number;
  /** Number of links on the page */
  linkCount: number;
  /** Total DOM elements count */
  domElements: number;
  /** Text to code ratio (higher is better for SEO) */
  textToCodeRatio: number;
  /** Content quality score (0-100) */
  contentQualityScore: number;
  /** Word count for readability analysis */
  wordCount: number;
}

export interface ResourceTiming {
  /** Resource URL */
  url: string;
  /** Resource type (script, stylesheet, image, etc.) */
  type: string;
  /** Size in bytes */
  size: number;
  /** Load duration in milliseconds */
  duration: number;
  /** Transfer size (compressed) */
  transferSize: number;
  /** Whether resource was cached */
  cached: boolean;
}

export interface PerformanceMetrics {
  // Core Web Vitals
  /** Largest Contentful Paint in ms */
  lcp: number;
  /** Interaction to Next Paint in ms */
  inp: number;
  /** Cumulative Layout Shift score */
  cls: number;
  
  // Additional Performance Metrics
  /** Time to First Byte in ms */
  ttfb: number;
  /** First Input Delay in ms */
  fid: number;
  /** Total Blocking Time in ms */
  tbt: number;
  /** Speed Index */
  speedIndex: number;
  
  // Timing Metrics
  /** DOM Content Loaded event in ms */
  domContentLoaded: number;
  /** Load event complete in ms */
  loadComplete: number;
  /** First Paint in ms */
  firstPaint: number;
  /** First Contentful Paint in ms */
  firstContentfulPaint: number;
  
  // Network Analysis
  /** Total number of HTTP requests */
  requestCount: number;
  /** Total transfer size in bytes */
  transferSize: number;
  /** Individual resource timings */
  resourceLoadTimes: ResourceTiming[];
  
  // Performance Scores & Analysis
  /** Overall performance score (0-100) */
  performanceScore: number;
  /** Performance grade (A-F) */
  performanceGrade: 'A' | 'B' | 'C' | 'D' | 'F';
  /** Specific recommendations for improvement */
  recommendations: string[];
  
  // Content Weight Analysis
  contentWeight: ContentWeight;
  contentAnalysis: ContentAnalysis;
}

export interface HeadingStructure {
  /** Number of H1 tags */
  h1Count: number;
  /** Number of H2 tags */
  h2Count: number;
  /** Number of H3 tags */
  h3Count: number;
  /** Number of H4 tags */
  h4Count: number;
  /** Number of H5 tags */
  h5Count: number;
  /** Number of H6 tags */
  h6Count: number;
  /** Whether heading structure follows hierarchy */
  structureValid: boolean;
  /** Heading structure issues */
  issues: string[];
}

export interface MetaTagAnalysis {
  title: {
    present: boolean;
    content?: string;
    length: number;
    optimal: boolean;
    issues: string[];
  };
  description: {
    present: boolean;
    content?: string;
    length: number;
    optimal: boolean;
    issues: string[];
  };
  keywords?: {
    present: boolean;
    content?: string;
    relevant: boolean;
  };
  robots?: {
    present: boolean;
    content?: string;
    indexable: boolean;
  };
  canonical?: {
    present: boolean;
    url?: string;
    valid: boolean;
  };
  viewport?: {
    present: boolean;
    mobileOptimized: boolean;
  };
}

export interface SocialMetaTags {
  openGraph: {
    title?: string;
    description?: string;
    image?: string;
    url?: string;
    type?: string;
    siteName?: string;
    locale?: string;
  };
  twitterCard: {
    card?: string;
    title?: string;
    description?: string;
    image?: string;
    site?: string;
    creator?: string;
  };
  /** Social tags completeness score (0-100) */
  completenessScore: number;
}

export interface TechnicalSEO {
  /** SSL certificate present */
  httpsEnabled: boolean;
  /** Sitemap.xml accessible */
  sitemapPresent: boolean;
  /** Robots.txt accessible */
  robotsTxtPresent: boolean;
  /** Schema markup present */
  schemaMarkup: string[];
  /** Page load speed impact on SEO */
  pageSpeedScore: number;
  /** Mobile-friendly test result */
  mobileFriendly: boolean;
  /** Internal/external link analysis */
  linkAnalysis: {
    internalLinks: number;
    externalLinks: number;
    brokenLinks: number;
  };
}

export interface SEOMetrics {
  /** Meta tag analysis */
  metaTags: MetaTagAnalysis;
  /** Heading structure analysis */
  headingStructure: HeadingStructure;
  /** Social media meta tags */
  socialTags: SocialMetaTags;
  /** Technical SEO factors */
  technicalSEO: TechnicalSEO;
  
  // Content Quality Analysis
  /** Total word count */
  wordCount: number;
  /** Readability score (Flesch-Kincaid) */
  readabilityScore: number;
  /** Content quality rating */
  contentQuality: 'poor' | 'fair' | 'good' | 'excellent';
  /** Content uniqueness score */
  contentUniqueness: number;
  
  // SEO Scores
  /** Overall SEO score (0-100) */
  overallSEOScore: number;
  /** SEO grade (A-F) */
  seoGrade: 'A' | 'B' | 'C' | 'D' | 'F';
  /** Specific SEO recommendations */
  recommendations: string[];
  
  // Competitive Analysis
  /** Estimated search visibility */
  searchVisibility: number;
  /** Key improvement opportunities */
  opportunityAreas: string[];
}

export interface PageQualityMetrics {
  /** URL being analyzed */
  url: string;
  /** Page title */
  title: string;
  /** Performance metrics */
  performance: PerformanceMetrics;
  /** SEO metrics */
  seo: SEOMetrics;
  /** Mobile-friendliness metrics */
  mobileFriendliness?: MobileFriendlinessMetrics;
  /** Overall quality score combining all metrics */
  overallQualityScore: number;
  /** Quality grade (A-F) */
  qualityGrade: 'A' | 'B' | 'C' | 'D' | 'F';
  /** Timestamp of analysis */
  analyzedAt: string;
}

export interface QualityAnalysisOptions {
  /** Include detailed resource analysis */
  includeResourceAnalysis?: boolean;
  /** Include social media tag analysis */
  includeSocialAnalysis?: boolean;
  /** Include readability analysis */
  includeReadabilityAnalysis?: boolean;
  /** Include technical SEO checks */
  includeTechnicalSEO?: boolean;
  /** Include mobile-friendliness analysis */
  includeMobileFriendliness?: boolean;
  /** Timeout for analysis in milliseconds */
  analysisTimeout?: number;
  /** Enable verbose logging for debugging */
  verbose?: boolean;
}

// Mobile-Friendliness Analysis Types
export interface MobileFriendlinessMetrics {
  overallScore: number;
  grade: string;
  viewport: {
    hasViewportTag: boolean;
    viewportContent: string;
    isResponsive: boolean;
    hasHorizontalScroll: boolean;
    breakpointCount: number;
    hasSafeAreaInsets: boolean;
    score: number;
  };
  typography: {
    baseFontSize: number;
    lineHeight: number;
    maxLineLength: number;
    isAccessibleFontSize: boolean;
    contrastScore: number;
    score: number;
  };
  touchTargets: {
    compliantTargets: number;
    totalTargets: number;
    averageTargetSize: number;
    minimumSpacing: number;
    violations: {
      selector: string;
      currentSize: number;
      requiredSize: number;
      spacing: number;
      recommendation: string;
    }[];
    score: number;
  };
  navigation: {
    hasStickyHeader: boolean;
    stickyHeaderHeight: number;
    hasAccessibleNavigation: boolean;
    supportsKeyboardNavigation: boolean;
    hasVisibleFocusIndicators: boolean;
    score: number;
  };
  media: {
    hasResponsiveImages: boolean;
    usesModernImageFormats: boolean;
    hasLazyLoading: boolean;
    videoOptimizations: {
      hasPlaysinline: boolean;
      hasPosterImage: boolean;
      hasSubtitles: boolean;
      noAutoplayAudio: boolean;
    };
    score: number;
  };
  performance: {
    lcp: number;
    inp: number;
    cls: number;
    ttfb: number;
    isMobileOptimized: boolean;
    score: number;
  };
  forms: {
    hasProperInputTypes: boolean;
    hasAutocomplete: boolean;
    labelsAboveFields: boolean;
    keyboardFriendly: boolean;
    score: number;
  };
  ux: {
    hasIntrusiveInterstitials: boolean;
    hasProperErrorHandling: boolean;
    isOfflineFriendly: boolean;
    hasCumulativeLayoutShift: boolean;
    score: number;
  };
  recommendations: {
    category: string;
    priority: 'high' | 'medium' | 'low';
    issue: string;
    recommendation: string;
    impact: string;
  }[];
}

export interface QualityBudgets {
  /** Performance budget thresholds */
  performance: {
    lcp: number;        // Max LCP in ms
    fid: number;        // Max FID in ms
    cls: number;        // Max CLS score
    totalSize: number;  // Max total size in MB
  };
  /** SEO quality thresholds */
  seo: {
    titleLength: { min: number; max: number };
    descriptionLength: { min: number; max: number };
    minWordCount: number;
    minReadabilityScore: number;
  };
  /** Content quality thresholds */
  content: {
    minTextToCodeRatio: number;
    maxImageCount: number;
    maxDomElements: number;
  };
}
