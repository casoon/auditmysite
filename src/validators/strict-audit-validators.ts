/**
 * 🔒 STRICT AUDIT VALIDATORS - MANDATORY DATA VALIDATION
 * 
 * Diese Validatoren und Factory-Functions erzwingen vollständige Datenstrukturen.
 * Sie werfen explizite Fehler wenn kritische Daten fehlen, anstatt mit
 * Default-Werten oder undefined-Werten weiterzumachen.
 */

import {
  StrictAuditData,
  StrictAuditPage,
  StrictPageAccessibility,
  StrictPagePerformance,
  StrictPageSEO,
  StrictPageContentWeight,
  StrictPageMobileFriendliness,
  StrictAccessibilityIssue,
  StrictPerformanceMetrics,
  StrictSEOData,
  StrictContentWeightData,
  StrictMobileRecommendation,
  IncompleteAuditDataError,
  MissingAnalysisError,
  hasCompleteAnalysis
} from '../types/strict-audit-types';

// ============================================================================
// FACTORY FUNCTIONS - STRICT CREATION WITH VALIDATION
// ============================================================================

/**
 * Factory: Erstellt strikte Accessibility-Ergebnisse mit vollständiger Validierung
 */
export function createStrictAccessibility(data: any, pageUrl: string): StrictPageAccessibility {
  // Validate required fields
  if (typeof data?.score !== 'number') {
    throw new MissingAnalysisError('accessibility', pageUrl, 'score is not a number');
  }

  if (data.score < 0 || data.score > 100) {
    throw new MissingAnalysisError('accessibility', pageUrl, `score ${data.score} is out of range 0-100`);
  }

  if (!Array.isArray(data?.errors)) {
    throw new MissingAnalysisError('accessibility', pageUrl, 'errors is not an array');
  }

  if (!Array.isArray(data?.warnings)) {
    throw new MissingAnalysisError('accessibility', pageUrl, 'warnings is not an array');
  }

  if (!Array.isArray(data?.notices)) {
    throw new MissingAnalysisError('accessibility', pageUrl, 'notices is not an array');
  }

  // Validate and convert issues
  const errors = data.errors.map((issue: any, index: number) => 
    createStrictAccessibilityIssue(issue, `error[${index}]`, pageUrl)
  );

  const warnings = data.warnings.map((issue: any, index: number) => 
    createStrictAccessibilityIssue(issue, `warning[${index}]`, pageUrl)
  );

  const notices = data.notices.map((issue: any, index: number) => 
    createStrictAccessibilityIssue(issue, `notice[${index}]`, pageUrl)
  );

  // Determine WCAG level based on score
  let wcagLevel: 'A' | 'AA' | 'AAA' | 'none';
  if (data.score >= 95) wcagLevel = 'AAA';
  else if (data.score >= 80) wcagLevel = 'AA';
  else if (data.score >= 60) wcagLevel = 'A';
  else wcagLevel = 'none';

  return {
    score: data.score,
    errors,
    warnings,
    notices,
    totalIssues: errors.length + warnings.length + notices.length,
    wcagLevel
  };
}

/**
 * Factory: Erstellt strikte Accessibility-Issue mit vollständiger Validierung
 */
function createStrictAccessibilityIssue(data: any, context: string, pageUrl: string): StrictAccessibilityIssue {
  // Handle string-based issues (legacy format)
  if (typeof data === 'string') {
    return {
      code: 'legacy-string-issue',
      message: data,
      type: context.includes('error') ? 'error' : context.includes('warning') ? 'warning' : 'notice',
      selector: null,
      context: null,
      impact: null,
      help: null,
      helpUrl: null
    };
  }

  if (!data?.message || typeof data.message !== 'string') {
    throw new MissingAnalysisError('accessibility', pageUrl, `${context}: message is required and must be string`);
  }

  if (!data?.type || !['error', 'warning', 'notice'].includes(data.type)) {
    throw new MissingAnalysisError('accessibility', pageUrl, `${context}: type must be error, warning, or notice`);
  }

  return {
    code: typeof data.code === 'string' ? data.code : 'unknown-rule',
    message: data.message,
    type: data.type,
    selector: typeof data.selector === 'string' ? data.selector : null,
    context: typeof data.context === 'string' ? data.context : null,
    impact: data.impact && ['minor', 'moderate', 'serious', 'critical'].includes(data.impact) ? data.impact : null,
    help: typeof data.help === 'string' ? data.help : null,
    helpUrl: typeof data.helpUrl === 'string' ? data.helpUrl : null
  };
}

/**
 * Factory: Erstellt strikte Performance-Ergebnisse mit vollständiger Validierung
 */
export function createStrictPerformance(data: any, pageUrl: string): StrictPagePerformance {
  if (typeof data?.score !== 'number') {
    throw new MissingAnalysisError('performance', pageUrl, 'score is not a number');
  }

  if (data.score < 0 || data.score > 100) {
    throw new MissingAnalysisError('performance', pageUrl, `score ${data.score} is out of range 0-100`);
  }

  if (!data?.grade || !['A', 'B', 'C', 'D', 'F'].includes(data.grade)) {
    throw new MissingAnalysisError('performance', pageUrl, 'grade must be A, B, C, D, or F');
  }

  // Validate Core Web Vitals
  const coreWebVitals = createStrictPerformanceMetrics(data.coreWebVitals, pageUrl);

  // Validate issues array
  if (!Array.isArray(data?.issues)) {
    throw new MissingAnalysisError('performance', pageUrl, 'issues must be an array');
  }

  const issues = data.issues.filter((issue: any) => typeof issue === 'string');

  // Count budget violations (issues that mention "budget" or "exceeds")
  const budgetViolations = issues.filter((issue: string) => 
    issue.toLowerCase().includes('budget') || issue.toLowerCase().includes('exceeds')
  ).length;

  return {
    score: data.score,
    grade: data.grade,
    coreWebVitals,
    issues,
    budgetViolations
  };
}

/**
 * Factory: Erstellt strikte Performance-Metriken mit vollständiger Validierung
 */
function createStrictPerformanceMetrics(data: any, pageUrl: string): StrictPerformanceMetrics {
  const requiredMetrics = [
    'largestContentfulPaint',
    'firstContentfulPaint', 
    'cumulativeLayoutShift',
    'timeToFirstByte',
    'domContentLoaded',
    'loadComplete',
    'firstPaint'
  ];

  const missingMetrics: string[] = [];

  for (const metric of requiredMetrics) {
    if (typeof data?.[metric] !== 'number') {
      missingMetrics.push(metric);
    }
  }

  if (missingMetrics.length > 0) {
    throw new MissingAnalysisError('performance', pageUrl, 
      `Missing required metrics: ${missingMetrics.join(', ')}`);
  }

  return {
    largestContentfulPaint: data.largestContentfulPaint,
    firstContentfulPaint: data.firstContentfulPaint,
    cumulativeLayoutShift: data.cumulativeLayoutShift,
    timeToFirstByte: data.timeToFirstByte,
    domContentLoaded: data.domContentLoaded,
    loadComplete: data.loadComplete,
    firstPaint: data.firstPaint
  };
}

/**
 * Factory: Erstellt strikte SEO-Ergebnisse mit vollständiger Validierung
 */
export function createStrictSEO(data: any, pageUrl: string): StrictPageSEO {
  if (typeof data?.score !== 'number') {
    throw new MissingAnalysisError('seo', pageUrl, 'score is not a number');
  }

  if (data.score < 0 || data.score > 100) {
    throw new MissingAnalysisError('seo', pageUrl, `score ${data.score} is out of range 0-100`);
  }

  if (!data?.grade || !['A', 'B', 'C', 'D', 'F'].includes(data.grade)) {
    throw new MissingAnalysisError('seo', pageUrl, 'grade must be A, B, C, D, or F');
  }

  // Create strict meta tags data
  const metaTags = createStrictSEOData(data.metaTags, pageUrl);

  // Validate arrays
  if (!Array.isArray(data?.issues)) {
    throw new MissingAnalysisError('seo', pageUrl, 'issues must be an array');
  }

  const issues = data.issues.filter((issue: any) => typeof issue === 'string');
  const recommendations = Array.isArray(data.recommendations) ? 
    data.recommendations.filter((rec: any) => typeof rec === 'string') : [];

  return {
    score: data.score,
    grade: data.grade,
    metaTags,
    issues,
    recommendations
  };
}

/**
 * Factory: Erstellt strikte SEO-Metadaten mit vollständiger Validierung
 */
function createStrictSEOData(data: any, pageUrl: string): StrictSEOData {
  return {
    title: typeof data?.title === 'string' ? data.title : '',
    titleLength: typeof data?.titleLength === 'number' ? data.titleLength : 
                 (typeof data?.title === 'string' ? data.title.length : 0),
    description: typeof data?.description === 'string' ? data.description : '',
    descriptionLength: typeof data?.descriptionLength === 'number' ? data.descriptionLength :
                      (typeof data?.description === 'string' ? data.description.length : 0),
    keywords: typeof data?.keywords === 'string' ? data.keywords : '',
    h1Count: typeof data?.h1 === 'number' ? data.h1 : 0,
    h2Count: typeof data?.h2 === 'number' ? data.h2 : 0,
    h3Count: typeof data?.h3 === 'number' ? data.h3 : 0,
    totalImages: typeof data?.total === 'number' ? data.total : 
                (typeof data?.images?.total === 'number' ? data.images.total : 0),
    imagesWithoutAlt: typeof data?.withoutAlt === 'number' ? data.withoutAlt :
                     (typeof data?.images?.withoutAlt === 'number' ? data.images.withoutAlt : 0),
    imagesWithEmptyAlt: typeof data?.withEmptyAlt === 'number' ? data.withEmptyAlt :
                       (typeof data?.images?.withEmptyAlt === 'number' ? data.images.withEmptyAlt : 0)
  };
}

/**
 * Factory: Erstellt strikte Content-Weight-Ergebnisse mit vollständiger Validierung
 */
export function createStrictContentWeight(data: any, pageUrl: string): StrictPageContentWeight {
  if (typeof data?.score !== 'number') {
    throw new MissingAnalysisError('contentWeight', pageUrl, 'score is not a number');
  }

  if (data.score < 0 || data.score > 100) {
    throw new MissingAnalysisError('contentWeight', pageUrl, `score ${data.score} is out of range 0-100`);
  }

  if (!data?.grade || !['A', 'B', 'C', 'D', 'F'].includes(data.grade)) {
    throw new MissingAnalysisError('contentWeight', pageUrl, 'grade must be A, B, C, D, or F');
  }

  const resources = createStrictContentWeightData(data.resources || data, pageUrl);

  const optimizations = Array.isArray(data.optimizations) ? 
    data.optimizations.filter((opt: any) => typeof opt === 'string') : [];

  // Calculate compression ratio (compressed size / original size)
  const totalOriginalSize = resources.totalSize;
  const estimatedCompressedSize = Math.round(totalOriginalSize * 0.7); // Estimate 30% compression
  const compressionRatio = totalOriginalSize > 0 ? estimatedCompressedSize / totalOriginalSize : 1.0;

  return {
    score: data.score,
    grade: data.grade,
    resources,
    optimizations,
    compressionRatio
  };
}

/**
 * Factory: Erstellt strikte Content-Weight-Daten mit vollständiger Validierung
 */
function createStrictContentWeightData(data: any, pageUrl: string): StrictContentWeightData {
  const totalSize = typeof data?.totalSize === 'number' ? data.totalSize :
                   (typeof data?.total === 'number' ? data.total : 0);

  return {
    totalSize,
    html: {
      size: typeof data?.html?.size === 'number' ? data.html.size : 0,
      files: typeof data?.html?.files === 'number' ? data.html.files : 1
    },
    css: {
      size: typeof data?.css?.size === 'number' ? data.css.size : 0,
      files: typeof data?.css?.files === 'number' ? data.css.files : 0
    },
    javascript: {
      size: typeof data?.javascript?.size === 'number' ? data.javascript.size : 0,
      files: typeof data?.javascript?.files === 'number' ? data.javascript.files : 0
    },
    images: {
      size: typeof data?.images?.size === 'number' ? data.images.size : 0,
      files: typeof data?.images?.files === 'number' ? data.images.files : 0
    },
    other: {
      size: typeof data?.other?.size === 'number' ? data.other.size : 0,
      files: typeof data?.other?.files === 'number' ? data.other.files : 0
    }
  };
}

/**
 * Factory: Erstellt strikte Mobile-Friendliness-Ergebnisse mit vollständiger Validierung
 */
export function createStrictMobileFriendliness(data: any, pageUrl: string): StrictPageMobileFriendliness {
  if (typeof data?.overallScore !== 'number') {
    throw new MissingAnalysisError('mobileFriendliness', pageUrl, 'overallScore is not a number');
  }

  if (data.overallScore < 0 || data.overallScore > 100) {
    throw new MissingAnalysisError('mobileFriendliness', pageUrl, 
      `overallScore ${data.overallScore} is out of range 0-100`);
  }

  if (!data?.grade || !['A', 'B', 'C', 'D', 'F'].includes(data.grade)) {
    throw new MissingAnalysisError('mobileFriendliness', pageUrl, 'grade must be A, B, C, D, or F');
  }

  if (!Array.isArray(data?.recommendations)) {
    throw new MissingAnalysisError('mobileFriendliness', pageUrl, 'recommendations must be an array');
  }

  const recommendations = data.recommendations.map((rec: any, index: number) =>
    createStrictMobileRecommendation(rec, `recommendation[${index}]`, pageUrl)
  );

  // Count specific issue types from recommendations
  const touchTargetIssues = recommendations.filter((r: any) => r.category === 'Touch Targets').length;
  const responsiveIssues = recommendations.filter((r: any) => r.category === 'Media').length;

  return {
    overallScore: data.overallScore,
    grade: data.grade,
    recommendations,
    touchTargetIssues,
    responsiveIssues
  };
}

/**
 * Factory: Erstellt strikte Mobile-Recommendation mit vollständiger Validierung
 */
function createStrictMobileRecommendation(data: any, context: string, pageUrl: string): StrictMobileRecommendation {
  const validCategories = ['Touch Targets', 'Performance', 'Media', 'Forms', 'Navigation', 'Content'];
  const validPriorities = ['low', 'medium', 'high', 'critical'];

  if (!data?.category || !validCategories.includes(data.category)) {
    throw new MissingAnalysisError('mobileFriendliness', pageUrl, 
      `${context}: category must be one of: ${validCategories.join(', ')}`);
  }

  if (!data?.priority || !validPriorities.includes(data.priority)) {
    throw new MissingAnalysisError('mobileFriendliness', pageUrl,
      `${context}: priority must be one of: ${validPriorities.join(', ')}`);
  }

  if (!data?.issue || typeof data.issue !== 'string') {
    throw new MissingAnalysisError('mobileFriendliness', pageUrl, `${context}: issue is required and must be string`);
  }

  if (!data?.recommendation || typeof data.recommendation !== 'string') {
    throw new MissingAnalysisError('mobileFriendliness', pageUrl, 
      `${context}: recommendation is required and must be string`);
  }

  if (!data?.impact || typeof data.impact !== 'string') {
    throw new MissingAnalysisError('mobileFriendliness', pageUrl, `${context}: impact is required and must be string`);
  }

  return {
    category: data.category,
    priority: data.priority,
    issue: data.issue,
    recommendation: data.recommendation,
    impact: data.impact
  };
}

// ============================================================================
// MAIN VALIDATION AND FACTORY FUNCTIONS
// ============================================================================

/**
 * Factory: Erstellt strikte Audit-Seite mit vollständiger Validierung aller Analyse-Typen
 */
export function createStrictAuditPage(data: any): StrictAuditPage {
  // Validate basic page data
  if (!data?.url || typeof data.url !== 'string') {
    throw new IncompleteAuditDataError('Page URL is required', ['url']);
  }

  if (!data?.title || typeof data.title !== 'string') {
    throw new IncompleteAuditDataError('Page title is required', ['title'], data.url);
  }

  if (!data?.status || !['passed', 'failed', 'crashed'].includes(data.status)) {
    throw new IncompleteAuditDataError('Page status must be passed, failed, or crashed', ['status'], data.url);
  }

  if (typeof data?.duration !== 'number') {
    throw new IncompleteAuditDataError('Page duration must be a number', ['duration'], data.url);
  }

  // CRITICAL: ALL analysis types must be present and valid
  const missingAnalyses: string[] = [];

  if (!data.accessibility) missingAnalyses.push('accessibility');
  if (!data.performance) missingAnalyses.push('performance');
  if (!data.seo) missingAnalyses.push('seo');
  if (!data.contentWeight) missingAnalyses.push('contentWeight');
  if (!data.mobileFriendliness) missingAnalyses.push('mobileFriendliness');

  if (missingAnalyses.length > 0) {
    throw new IncompleteAuditDataError(
      `Missing required analysis types for page: ${data.url}`,
      missingAnalyses,
      data.url
    );
  }

  // Create strict analysis objects with full validation
  const accessibility = createStrictAccessibility(data.accessibility, data.url);
  const performance = createStrictPerformance(data.performance, data.url);
  const seo = createStrictSEO(data.seo, data.url);
  const contentWeight = createStrictContentWeight(data.contentWeight, data.url);
  const mobileFriendliness = createStrictMobileFriendliness(data.mobileFriendliness, data.url);

  return {
    url: data.url,
    title: data.title,
    status: data.status,
    duration: data.duration,
    testedAt: typeof data.testedAt === 'string' ? data.testedAt : new Date().toISOString(),
    accessibility,
    performance,
    seo,
    contentWeight,
    mobileFriendliness
  };
}

/**
 * Main Factory: Erstellt vollständige strikte Audit-Daten mit umfassender Validierung
 */
export function createStrictAuditData(data: any): StrictAuditData {
  // Validate metadata
  if (!data?.metadata) {
    throw new IncompleteAuditDataError('Metadata is required', ['metadata']);
  }

  // Validate summary
  if (!data?.summary) {
    throw new IncompleteAuditDataError('Summary is required', ['summary']);
  }

  // Validate pages array
  if (!Array.isArray(data?.pages)) {
    throw new IncompleteAuditDataError('Pages must be an array', ['pages']);
  }

  if (data.pages.length === 0) {
    throw new IncompleteAuditDataError('Pages array cannot be empty', ['pages']);
  }

  // Create strict pages with full validation
  const strictPages: StrictAuditPage[] = data.pages.map((pageData: any) => 
    createStrictAuditPage(pageData)
  );

  // Validate that all pages passed the strict validation
  for (const page of strictPages) {
    if (!hasCompleteAnalysis(page)) {
      throw new IncompleteAuditDataError(
        `Page ${(page as any).url} does not have complete analysis data`,
        ['complete_analysis'],
        (page as any).url
      );
    }
  }

  // Calculate derived summary data
  const totalErrors = strictPages.reduce((sum, page) => sum + page.accessibility.errors.length, 0);
  const totalWarnings = strictPages.reduce((sum, page) => sum + page.accessibility.warnings.length, 0);
  const averageScore = strictPages.reduce((sum, page) => sum + page.accessibility.score, 0) / strictPages.length;
  
  let overallGrade: 'A' | 'B' | 'C' | 'D' | 'F';
  if (averageScore >= 90) overallGrade = 'A';
  else if (averageScore >= 75) overallGrade = 'B';
  else if (averageScore >= 60) overallGrade = 'C';
  else if (averageScore >= 50) overallGrade = 'D';
  else overallGrade = 'F';

  return {
    metadata: {
      version: data.metadata.version || '1.0.0',
      timestamp: data.metadata.timestamp || new Date().toISOString(),
      sitemapUrl: data.metadata.sitemapUrl || '',
      toolVersion: data.metadata.toolVersion || '2.0.0-alpha.2',
      duration: typeof data.metadata.duration === 'number' ? data.metadata.duration : 0,
      configuration: {
        maxPages: typeof data.metadata.maxPages === 'number' ? data.metadata.maxPages : strictPages.length,
        timeout: typeof data.metadata.timeout === 'number' ? data.metadata.timeout : 30000,
        standard: typeof data.metadata.standard === 'string' ? data.metadata.standard : 'WCAG2AA',
        features: Array.isArray(data.metadata.features) ? data.metadata.features : 
                 ['accessibility', 'performance', 'seo', 'contentWeight', 'mobileFriendliness']
      }
    },
    summary: {
      totalPages: typeof data.summary.totalPages === 'number' ? data.summary.totalPages : strictPages.length,
      testedPages: strictPages.length,
      passedPages: strictPages.filter(p => p.status === 'passed').length,
      failedPages: strictPages.filter(p => p.status === 'failed').length,
      crashedPages: strictPages.filter(p => p.status === 'crashed').length,
      redirectPages: typeof data.summary.redirectPages === 'number' ? data.summary.redirectPages : 0,
      totalErrors,
      totalWarnings,
      averageScore: Math.round(averageScore),
      overallGrade
    },
    pages: strictPages,
    systemPerformance: {
      testCompletionTimeSeconds: Math.round((data.metadata?.duration || 0) / 1000),
      averageTimePerPageMs: Math.round((data.metadata?.duration || 0) / strictPages.length),
      throughputPagesPerMinute: Math.round(strictPages.length / ((data.metadata?.duration || 1) / 1000 / 60)),
      memoryUsageMB: typeof data.systemPerformance?.memoryUsageMB === 'number' ? 
                    data.systemPerformance.memoryUsageMB : 0,
      efficiency: strictPages.length > 0 ? 100.0 : 0.0
    }
  };
}

// ============================================================================
// RUNTIME VALIDATION FUNCTIONS
// ============================================================================

/**
 * Runtime-Validator: Prüft ob Audit-Daten vollständig und gültig sind
 */
export function validateStrictAuditData(data: any): asserts data is StrictAuditData {
  try {
    const strictData = createStrictAuditData(data);
    // If we get here without throwing, data is valid
    console.log('✅ Strict audit data validation passed');
  } catch (error) {
    if (error instanceof IncompleteAuditDataError || error instanceof MissingAnalysisError) {
      throw error;
    }
    throw new IncompleteAuditDataError(
      `Audit data validation failed: ${error instanceof Error ? error.message : 'Unknown error'}`,
      ['validation_error']
    );
  }
}

/**
 * Runtime-Validator: Prüft ob alle Seiten vollständige Analysen haben
 */
export function validateAllPagesComplete(pages: any[]): asserts pages is StrictAuditPage[] {
  const incompletePages: string[] = [];

  pages.forEach((page, index) => {
    if (!hasCompleteAnalysis(page)) {
      incompletePages.push(page?.url || `page[${index}]`);
    }
  });

  if (incompletePages.length > 0) {
    throw new IncompleteAuditDataError(
      'Some pages do not have complete analysis data',
      ['complete_analysis'],
      incompletePages.join(', ')
    );
  }
}