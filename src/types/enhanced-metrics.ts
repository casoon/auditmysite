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
  
  // Advanced SEO Features
  /** Semantic SEO analysis */
  semanticSEO?: {
    semanticScore: number;
    topicClusters: string[];
    contentDepthScore: number;
    lsiKeywords: string[];
    recommendations: string[];
  };
  /** Voice search optimization analysis */
  voiceSearchOptimization?: {
    voiceSearchScore: number;
    questionPhrases: number;
    conversationalContent: boolean;
    recommendations: string[];
  };
  /** E-A-T (Expertise, Authoritativeness, Trustworthiness) analysis */
  eatAnalysis?: {
    eatScore: number;
    authorPresence: boolean;
    expertiseIndicators: string[];
    trustSignals: string[];
    recommendations: string[];
  };
  /** Core Web Vitals SEO impact analysis */
  coreWebVitalsSEO?: {
    seoImpactScore: number;
    vitalsCritical: string[];
    seoRecommendations: string[];
  };
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
  /** Time to wait for metrics to settle (e.g., LCP) in milliseconds. Default: 2000ms. Set to 0 to disable. */
  metricsSettleTime?: number;
  /** Enable verbose logging for debugging */
  verbose?: boolean;
  /** Enable PSI-like throttling profile */
  psiProfile?: boolean;
  /** CPU throttling rate (e.g., 4 = 4x slower) */
  psiCPUThrottlingRate?: number;
  /** Network emulation settings in kbps/ms */
  psiNetwork?: {
    latencyMs: number;
    downloadKbps: number;
    uploadKbps: number;
  };
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

// =============================================================================
// SECURITY HEADERS ANALYSIS TYPES
// =============================================================================

/** Individual security header analysis */
export interface SecurityHeader {
  /** Header name */
  name: string;
  /** Header value if present */
  value?: string;
  /** Whether the header is present */
  present: boolean;
  /** Whether the header value is valid/secure */
  valid: boolean;
  /** Security score for this header (0-100) */
  score: number;
  /** Issues found with this header */
  issues: string[];
  /** Recommendations for improvement */
  recommendations: string[];
}

/** Content Security Policy analysis */
export interface CSPAnalysis {
  /** Whether CSP header is present */
  present: boolean;
  /** CSP header value */
  value?: string;
  /** Parsed CSP directives */
  directives: Record<string, string[]>;
  /** CSP security score (0-100) */
  score: number;
  /** Security issues found */
  issues: {
    severity: 'critical' | 'high' | 'medium' | 'low';
    directive?: string;
    issue: string;
    recommendation: string;
  }[];
  /** Whether unsafe directives are used */
  hasUnsafeDirectives: boolean;
  /** Whether inline scripts/styles are allowed */
  allowsInlineScripts: boolean;
  /** Whether eval() is allowed */
  allowsEval: boolean;
}

/** Security headers analysis results */
export interface SecurityHeadersMetrics {
  /** Overall security score (0-100) */
  overallScore: number;
  /** Security grade (A-F) */
  securityGrade: 'A' | 'B' | 'C' | 'D' | 'F';
  
  /** Individual header analyses */
  headers: {
    /** Content Security Policy */
    csp: CSPAnalysis;
    /** HTTP Strict Transport Security */
    hsts: SecurityHeader;
    /** X-Frame-Options */
    xFrameOptions: SecurityHeader;
    /** X-Content-Type-Options */
    xContentTypeOptions: SecurityHeader;
    /** X-XSS-Protection */
    xXSSProtection: SecurityHeader;
    /** Referrer-Policy */
    referrerPolicy: SecurityHeader;
    /** Permissions-Policy / Feature-Policy */
    permissionsPolicy: SecurityHeader;
  };
  
  /** HTTPS configuration analysis */
  https: {
    enabled: boolean;
    httpsRedirect: boolean;
    mixedContent: boolean;
    certificate: {
      valid: boolean;
      issuer?: string;
      expiresAt?: string;
      daysUntilExpiry?: number;
    };
  };
  
  /** Cookie security analysis */
  cookies: {
    totalCookies: number;
    secureCookies: number;
    httpOnlyCookies: number;
    sameSiteCookies: number;
    issues: string[];
  };
  
  /** Security recommendations */
  recommendations: {
    priority: 'critical' | 'high' | 'medium' | 'low';
    category: string;
    issue: string;
    recommendation: string;
    impact: string;
  }[];
  
  /** Vulnerability assessment */
  vulnerabilities: {
    clickjacking: 'protected' | 'vulnerable' | 'partially_protected';
    xss: 'protected' | 'vulnerable' | 'partially_protected';
    contentTypeSniffing: 'protected' | 'vulnerable';
    referrerLeakage: 'protected' | 'vulnerable' | 'partially_protected';
    mixedContent: 'protected' | 'vulnerable';
  };
}

// =============================================================================
// STRUCTURED DATA VALIDATION TYPES
// =============================================================================

/** Individual structured data item */
export interface StructuredDataItem {
  /** Data format (JSON-LD, Microdata, RDFa) */
  format: 'JSON-LD' | 'Microdata' | 'RDFa';
  /** Schema.org type */
  type: string;
  /** Location in the page */
  location: 'head' | 'body';
  /** CSS selector where found */
  selector?: string;
  /** Raw data content */
  data: any;
  /** Whether the structure is valid */
  valid: boolean;
  /** Validation errors */
  errors: string[];
  /** Validation warnings */
  warnings: string[];
  /** Schema.org compliance score (0-100) */
  complianceScore: number;
}

/** Schema.org analysis by type */
export interface SchemaTypeAnalysis {
  /** Schema type name (e.g., 'Organization', 'Article') */
  type: string;
  /** Number of instances found */
  count: number;
  /** Required properties analysis */
  requiredProperties: {
    property: string;
    present: boolean;
    valid: boolean;
    value?: string;
  }[];
  /** Recommended properties analysis */
  recommendedProperties: {
    property: string;
    present: boolean;
    benefit: string;
  }[];
  /** Completeness score (0-100) */
  completenessScore: number;
  /** Issues with this schema type */
  issues: string[];
}

/** Rich snippets potential analysis */
export interface RichSnippetsAnalysis {
  /** Eligible for rich snippets */
  eligible: boolean;
  /** Supported snippet types found */
  supportedTypes: string[];
  /** Potential snippet types that could be added */
  potentialTypes: string[];
  /** Rich snippets score (0-100) */
  richSnippetsScore: number;
  /** Recommendations for rich snippet optimization */
  recommendations: string[];
}

/** Knowledge Graph potential analysis */
export interface KnowledgeGraphAnalysis {
  /** Organization information completeness */
  organization: {
    present: boolean;
    completeness: number;
    missingProperties: string[];
  };
  /** Local business information completeness */
  localBusiness: {
    present: boolean;
    completeness: number;
    missingProperties: string[];
  };
  /** Article/content information completeness */
  content: {
    present: boolean;
    completeness: number;
    missingProperties: string[];
  };
  /** Knowledge Graph readiness score */
  readinessScore: number;
}

/** Structured data validation results */
export interface StructuredDataMetrics {
  /** Overall structured data score (0-100) */
  overallScore: number;
  /** Structured data grade (A-F) */
  structuredDataGrade: 'A' | 'B' | 'C' | 'D' | 'F';
  
  /** Summary statistics */
  summary: {
    totalItems: number;
    validItems: number;
    invalidItems: number;
    jsonLdCount: number;
    microdataCount: number;
    rdfaCount: number;
    uniqueTypes: string[];
  };
  
  /** Individual structured data items */
  items: StructuredDataItem[];
  
  /** Analysis by schema type */
  schemaTypes: SchemaTypeAnalysis[];
  
  /** Rich snippets analysis */
  richSnippets: RichSnippetsAnalysis;
  
  /** Knowledge Graph analysis */
  knowledgeGraph: KnowledgeGraphAnalysis;
  
  /** SEO impact analysis */
  seoImpact: {
    searchVisibilityBoost: number;
    clickThroughRateImpact: number;
    rankingFactorScore: number;
  };
  
  /** Validation issues */
  issues: {
    severity: 'error' | 'warning' | 'info';
    type: string;
    location: string;
    message: string;
    recommendation: string;
  }[];
  
  /** Improvement recommendations */
  recommendations: {
    priority: 'high' | 'medium' | 'low';
    category: string;
    issue: string;
    recommendation: string;
    impact: string;
    implementation: string;
  }[];
  
  /** Testing and validation URLs */
  testingUrls: {
    googleRichResultsTest: string;
    googleStructuredDataTest: string;
    schemaMarkupValidator: string;
  };
}
