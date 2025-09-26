/**
 * 🔄 AUDIT DATA ADAPTER - LEGACY TO STRICT CONVERSION
 * 
 * Diese Adapter-Schicht transformiert die bestehenden flexiblen/lockeren 
 * Audit-Datenstrukturen in strikt validierte, vollständige Strukturen.
 * Sie fungiert als Bridge zwischen dem bestehenden System und dem neuen
 * strikten Validierungs-System.
 */

import {
  createStrictAuditData,
  createStrictAuditPage,
  validateStrictAuditData,
  validateAllPagesComplete
} from '../validators/strict-audit-validators';
import {
  StrictAuditData,
  StrictAuditPage,
  IncompleteAuditDataError,
  MissingAnalysisError,
  hasCompleteAnalysis
} from '../types/strict-audit-types';
// Legacy-compatible types for flexible validation
export interface LegacyAuditResult {
  metadata?: {
    version?: string;
    timestamp?: string;
    sitemapUrl?: string;
    toolVersion?: string;
    duration?: number;
    maxPages?: number;
    timeout?: number;
    standard?: string;
    features?: string[];
  };
  summary?: {
    totalPages?: number;
    testedPages?: number;
    passedPages?: number;
    failedPages?: number;
    crashedPages?: number;
    redirectPages?: number;
    totalErrors?: number;
    totalWarnings?: number;
    averageScore?: number;
    overallGrade?: 'A' | 'B' | 'C' | 'D' | 'F';
  };
  pages?: LegacyPageResult[];
  systemPerformance?: {
    memoryUsageMB?: number;
  };
}

export interface LegacyPageResult {
  url?: string;
  title?: string;
  status?: string;
  duration?: number;
  testedAt?: string;
  accessibility?: LegacyAccessibilityResult;
  performance?: LegacyPerformanceResult;
  seo?: LegacySEOResult;
  contentWeight?: LegacyContentWeightResult;
  mobileFriendliness?: LegacyMobileFriendlinessResult;
}

export interface LegacyAccessibilityResult {
  score?: number;
  errors?: any[];
  warnings?: any[];
  notices?: any[];
}

export interface LegacyPerformanceResult {
  score?: number;
  grade?: string;
  coreWebVitals?: any;
  issues?: any[];
}

export interface LegacySEOResult {
  score?: number;
  grade?: string;
  metaTags?: any;
  issues?: any[];
  recommendations?: any[];
}

export interface LegacyContentWeightResult {
  score?: number;
  grade?: string;
  resources?: any;
  optimizations?: any[];
}

export interface LegacyMobileFriendlinessResult {
  overallScore?: number;
  grade?: string;
  recommendations?: any[];
}

export type AuditResult = LegacyAuditResult;
export type PageResult = LegacyPageResult;
export type AccessibilityResult = LegacyAccessibilityResult;
export type PerformanceResult = LegacyPerformanceResult;
export type SEOResult = LegacySEOResult;
export type ContentWeightResult = LegacyContentWeightResult;
export type MobileFriendlinessResult = LegacyMobileFriendlinessResult;

// ============================================================================
// TYPE MAPPINGS - LEGACY TO STRICT
// ============================================================================

/**
 * Adapter: Konvertiert Legacy AuditResult zu StrictAuditData
 */
export class AuditDataAdapter {
  
  /**
   * Hauptkonvertierung: Legacy AuditResult → StrictAuditData
   */
  static convertToStrict(legacyResult: AuditResult): StrictAuditData {
    try {
      console.log('🔄 Converting legacy audit data to strict format...');
      
      // First, transform legacy structure to intermediate format
      const intermediateData = this.transformLegacyStructure(legacyResult);
      
      // Then apply strict validation and create strict object
      const strictData = createStrictAuditData(intermediateData);
      
      console.log('✅ Successfully converted to strict audit data');
      console.log(`   Pages: ${strictData.pages.length}`);
      console.log(`   Total Errors: ${strictData.summary.totalErrors}`);
      console.log(`   Total Warnings: ${strictData.summary.totalWarnings}`);
      
      return strictData;
      
    } catch (error) {
      if (error instanceof IncompleteAuditDataError || error instanceof MissingAnalysisError) {
        console.error('❌ Strict validation failed:', error.message);
        throw error;
      }
      
      throw new IncompleteAuditDataError(
        `Audit data conversion failed: ${error instanceof Error ? error.message : 'Unknown error'}`,
        ['conversion_error']
      );
    }
  }

  /**
   * Legacy-Struktur zu Intermediate-Format transformieren
   */
  private static transformLegacyStructure(legacyResult: AuditResult): any {
    // Transform metadata
    const metadata = {
      version: legacyResult.metadata?.version || '1.0.0',
      timestamp: legacyResult.metadata?.timestamp || new Date().toISOString(),
      sitemapUrl: legacyResult.metadata?.sitemapUrl || '',
      toolVersion: legacyResult.metadata?.toolVersion || '2.0.0-alpha.2',
      duration: typeof legacyResult.metadata?.duration === 'number' ? 
               legacyResult.metadata.duration : 0,
      maxPages: typeof legacyResult.metadata?.maxPages === 'number' ? 
               legacyResult.metadata.maxPages : (legacyResult.pages?.length || 0),
      timeout: typeof legacyResult.metadata?.timeout === 'number' ? 
              legacyResult.metadata.timeout : 30000,
      standard: typeof legacyResult.metadata?.standard === 'string' ? 
               legacyResult.metadata.standard : 'WCAG2AA',
      features: Array.isArray(legacyResult.metadata?.features) ? 
               legacyResult.metadata.features : 
               ['accessibility', 'performance', 'seo', 'contentWeight', 'mobileFriendliness']
    };

    // Transform summary with defensive fallbacks
    const summary = {
      totalPages: legacyResult.summary?.totalPages || (legacyResult.pages?.length || 0),
      testedPages: legacyResult.summary?.testedPages || (legacyResult.pages?.length || 0),
      passedPages: legacyResult.summary?.passedPages || 0,
      failedPages: legacyResult.summary?.failedPages || 0,
      crashedPages: legacyResult.summary?.crashedPages || 0,
      redirectPages: legacyResult.summary?.redirectPages || 0,
      totalErrors: legacyResult.summary?.totalErrors || 0,
      totalWarnings: legacyResult.summary?.totalWarnings || 0,
      averageScore: legacyResult.summary?.averageScore || 0,
      overallGrade: legacyResult.summary?.overallGrade || 'F'
    };

    // Transform pages with comprehensive data enhancement
    const pages = (legacyResult.pages || []).map((page: PageResult) => 
      this.transformLegacyPage(page)
    );

    // Transform system performance
    const systemPerformance = {
      testCompletionTimeSeconds: Math.round((metadata.duration || 0) / 1000),
      averageTimePerPageMs: Math.round((metadata.duration || 0) / Math.max(pages.length, 1)),
      throughputPagesPerMinute: Math.round(pages.length / Math.max((metadata.duration || 1) / 1000 / 60, 1)),
      memoryUsageMB: legacyResult.systemPerformance?.memoryUsageMB || 0,
      efficiency: pages.length > 0 ? 100.0 : 0.0
    };

    return {
      metadata,
      summary,
      pages,
      systemPerformance
    };
  }

  /**
   * Legacy PageResult zu Strict-kompatiblem Format transformieren
   */
  private static transformLegacyPage(legacyPage: PageResult): any {
    return {
      url: legacyPage.url || '',
      title: legacyPage.title || 'Untitled Page',
      status: this.normalizePageStatus(legacyPage.status),
      duration: typeof legacyPage.duration === 'number' ? legacyPage.duration : 0,
      testedAt: legacyPage.testedAt || new Date().toISOString(),
      
      // Transform each analysis type with comprehensive fallbacks
      accessibility: this.transformAccessibilityResult(legacyPage.accessibility, legacyPage.url || 'unknown-url'),
      performance: this.transformPerformanceResult(legacyPage.performance, legacyPage.url || 'unknown-url'),
      seo: this.transformSEOResult(legacyPage.seo, legacyPage.url || 'unknown-url'),
      contentWeight: this.transformContentWeightResult(legacyPage.contentWeight, legacyPage.url || 'unknown-url'),
      mobileFriendliness: this.transformMobileFriendlinessResult(legacyPage.mobileFriendliness, legacyPage.url || 'unknown-url')
    };
  }

  /**
   * Normalisiert Page Status zu erlaubten Werten
   */
  private static normalizePageStatus(status: string | undefined): 'passed' | 'failed' | 'crashed' {
    if (status === 'passed' || status === 'failed' || status === 'crashed') {
      return status;
    }
    
    // Try to infer from status content
    if (typeof status === 'string') {
      const lowerStatus = status.toLowerCase();
      if (lowerStatus.includes('pass')) return 'passed';
      if (lowerStatus.includes('crash') || lowerStatus.includes('error')) return 'crashed';
      if (lowerStatus.includes('fail')) return 'failed';
    }
    
    return 'crashed'; // Conservative fallback
  }

  /**
   * Legacy Accessibility zu Strict-Format transformieren
   */
  private static transformAccessibilityResult(
    legacy: AccessibilityResult | undefined,
    pageUrl: string
  ): any {
    if (!legacy) {
      console.warn(`⚠️ Missing accessibility data for ${pageUrl}, creating empty structure`);
      return {
        score: 0,
        errors: [],
        warnings: [],
        notices: []
      };
    }

    return {
      score: typeof legacy.score === 'number' ? legacy.score : 0,
      errors: this.transformAccessibilityIssues(legacy.errors, 'error'),
      warnings: this.transformAccessibilityIssues(legacy.warnings, 'warning'),
      notices: this.transformAccessibilityIssues(legacy.notices || [], 'notice')
    };
  }

  /**
   * Transformiert Accessibility-Issues zu Strict-Format
   */
  private static transformAccessibilityIssues(
    issues: any[] | undefined,
    type: 'error' | 'warning' | 'notice'
  ): any[] {
    if (!Array.isArray(issues)) {
      return [];
    }

    return issues.map((issue: any) => {
      // Handle both object and string formats
      if (typeof issue === 'string') {
        return {
          code: 'legacy-string-issue',
          message: issue,
          type: type,
          selector: null,
          context: null,
          impact: null,
          help: null,
          helpUrl: null
        };
      }

      return {
        code: issue?.code || 'unknown-rule',
        message: issue?.message || issue?.description || 'No description available',
        type: issue?.type || type,
        selector: issue?.selector || null,
        context: issue?.context || null,
        impact: issue?.impact && ['minor', 'moderate', 'serious', 'critical'].includes(issue.impact) ? 
               issue.impact : null,
        help: issue?.help || null,
        helpUrl: issue?.helpUrl || null
      };
    });
  }

  /**
   * Legacy Performance zu Strict-Format transformieren
   */
  private static transformPerformanceResult(
    legacy: PerformanceResult | undefined,
    pageUrl: string
  ): any {
    if (!legacy) {
      console.warn(`⚠️ Missing performance data for ${pageUrl}, creating minimal structure`);
      return {
        score: 0,
        grade: 'F',
        coreWebVitals: this.createDefaultCoreWebVitals(),
        issues: [`Performance analysis failed for ${pageUrl}`]
      };
    }

    return {
      score: typeof legacy.score === 'number' ? legacy.score : 0,
      grade: legacy.grade || this.scoreToGrade(legacy.score || 0),
      coreWebVitals: this.transformCoreWebVitals(legacy.coreWebVitals),
      issues: Array.isArray(legacy.issues) ? 
              legacy.issues.filter((i: any) => typeof i === 'string') : []
    };
  }

  /**
   * Core Web Vitals transformieren mit Fallback-Werten
   */
  private static transformCoreWebVitals(coreWebVitals: any): any {
    return {
      largestContentfulPaint: coreWebVitals?.largestContentfulPaint || 0,
      firstContentfulPaint: coreWebVitals?.firstContentfulPaint || 0,
      cumulativeLayoutShift: coreWebVitals?.cumulativeLayoutShift || 0,
      timeToFirstByte: coreWebVitals?.timeToFirstByte || 0,
      domContentLoaded: coreWebVitals?.domContentLoaded || 0,
      loadComplete: coreWebVitals?.loadComplete || 0,
      firstPaint: coreWebVitals?.firstPaint || 0
    };
  }

  /**
   * Standard Core Web Vitals für fehlende Performance-Daten
   */
  private static createDefaultCoreWebVitals(): any {
    return {
      largestContentfulPaint: 0,
      firstContentfulPaint: 0,
      cumulativeLayoutShift: 0,
      timeToFirstByte: 0,
      domContentLoaded: 0,
      loadComplete: 0,
      firstPaint: 0
    };
  }

  /**
   * Legacy SEO zu Strict-Format transformieren
   */
  private static transformSEOResult(
    legacy: SEOResult | undefined,
    pageUrl: string
  ): any {
    if (!legacy) {
      console.warn(`⚠️ Missing SEO data for ${pageUrl}, creating minimal structure`);
      return {
        score: 0,
        grade: 'F',
        metaTags: this.createDefaultSEOData(),
        issues: [`SEO analysis failed for ${pageUrl}`],
        recommendations: []
      };
    }

    return {
      score: typeof legacy.score === 'number' ? legacy.score : 0,
      grade: legacy.grade || this.scoreToGrade(legacy.score || 0),
      metaTags: this.transformSEOMetaTags(legacy.metaTags || legacy),
      issues: Array.isArray(legacy.issues) ? 
              legacy.issues.filter((i: any) => typeof i === 'string') : [],
      recommendations: Array.isArray(legacy.recommendations) ? 
                      legacy.recommendations.filter((r: any) => typeof r === 'string') : []
    };
  }

  /**
   * SEO Meta-Tags transformieren
   */
  private static transformSEOMetaTags(metaTags: any): any {
    return {
      title: metaTags?.title || '',
      titleLength: metaTags?.titleLength || (metaTags?.title?.length || 0),
      description: metaTags?.description || '',
      descriptionLength: metaTags?.descriptionLength || (metaTags?.description?.length || 0),
      keywords: metaTags?.keywords || '',
      h1Count: metaTags?.h1 || metaTags?.headings?.h1 || 0,
      h2Count: metaTags?.h2 || metaTags?.headings?.h2 || 0,
      h3Count: metaTags?.h3 || metaTags?.headings?.h3 || 0,
      totalImages: metaTags?.total || metaTags?.images?.total || 0,
      imagesWithoutAlt: metaTags?.withoutAlt || metaTags?.images?.withoutAlt || 0,
      imagesWithEmptyAlt: metaTags?.withEmptyAlt || metaTags?.images?.withEmptyAlt || 0
    };
  }

  /**
   * Standard SEO-Daten für fehlende SEO-Analyse
   */
  private static createDefaultSEOData(): any {
    return {
      title: '',
      titleLength: 0,
      description: '',
      descriptionLength: 0,
      keywords: '',
      h1Count: 0,
      h2Count: 0,
      h3Count: 0,
      totalImages: 0,
      imagesWithoutAlt: 0,
      imagesWithEmptyAlt: 0
    };
  }

  /**
   * Legacy Content Weight zu Strict-Format transformieren
   */
  private static transformContentWeightResult(
    legacy: ContentWeightResult | undefined,
    pageUrl: string
  ): any {
    if (!legacy) {
      console.warn(`⚠️ Missing content weight data for ${pageUrl}, creating minimal structure`);
      return {
        score: 0,
        grade: 'F',
        resources: this.createDefaultContentWeightData(),
        optimizations: [`Content weight analysis failed for ${pageUrl}`]
      };
    }

    return {
      score: typeof legacy.score === 'number' ? legacy.score : 0,
      grade: legacy.grade || this.scoreToGrade(legacy.score || 0),
      resources: this.transformContentWeightResources(legacy.resources || legacy),
      optimizations: Array.isArray(legacy.optimizations) ? 
                    legacy.optimizations.filter((o: any) => typeof o === 'string') : []
    };
  }

  /**
   * Content Weight Resources transformieren
   */
  private static transformContentWeightResources(resources: any): any {
    return {
      totalSize: resources?.totalSize || resources?.total || 0,
      html: {
        size: resources?.html?.size || 0,
        files: resources?.html?.files || 1
      },
      css: {
        size: resources?.css?.size || 0,
        files: resources?.css?.files || 0
      },
      javascript: {
        size: resources?.javascript?.size || 0,
        files: resources?.javascript?.files || 0
      },
      images: {
        size: resources?.images?.size || 0,
        files: resources?.images?.files || 0
      },
      other: {
        size: resources?.other?.size || 0,
        files: resources?.other?.files || 0
      }
    };
  }

  /**
   * Standard Content Weight Daten für fehlende Analyse
   */
  private static createDefaultContentWeightData(): any {
    return {
      totalSize: 0,
      html: { size: 0, files: 1 },
      css: { size: 0, files: 0 },
      javascript: { size: 0, files: 0 },
      images: { size: 0, files: 0 },
      other: { size: 0, files: 0 }
    };
  }

  /**
   * Legacy Mobile Friendliness zu Strict-Format transformieren
   */
  private static transformMobileFriendlinessResult(
    legacy: MobileFriendlinessResult | undefined,
    pageUrl: string
  ): any {
    if (!legacy) {
      console.warn(`⚠️ Missing mobile friendliness data for ${pageUrl}, creating minimal structure`);
      return {
        overallScore: 0,
        grade: 'F',
        recommendations: [{
          category: 'Performance',
          priority: 'high',
          issue: 'Mobile friendliness analysis failed',
          recommendation: 'Retry mobile analysis',
          impact: `Analysis could not complete for ${pageUrl}`
        }]
      };
    }

    return {
      overallScore: typeof legacy.overallScore === 'number' ? legacy.overallScore : 0,
      grade: legacy.grade || this.scoreToGrade(legacy.overallScore || 0),
      recommendations: this.transformMobileRecommendations(legacy.recommendations || [])
    };
  }

  /**
   * Mobile Recommendations transformieren
   */
  private static transformMobileRecommendations(recommendations: any[]): any[] {
    if (!Array.isArray(recommendations)) {
      return [];
    }

    const validCategories = ['Touch Targets', 'Performance', 'Media', 'Forms', 'Navigation', 'Content'];
    const validPriorities = ['low', 'medium', 'high', 'critical'];

    return recommendations.map((rec: any) => ({
      category: validCategories.includes(rec?.category) ? rec.category : 'Content',
      priority: validPriorities.includes(rec?.priority) ? rec.priority : 'medium',
      issue: rec?.issue || 'Mobile issue detected',
      recommendation: rec?.recommendation || 'Improve mobile compatibility',
      impact: rec?.impact || 'May affect mobile user experience'
    }));
  }

  /**
   * Score zu Grade konvertieren
   */
  private static scoreToGrade(score: number): 'A' | 'B' | 'C' | 'D' | 'F' {
    if (score >= 90) return 'A';
    if (score >= 75) return 'B';
    if (score >= 60) return 'C';
    if (score >= 50) return 'D';
    return 'F';
  }

  /**
   * Diagnose-Funktion: Analysiert Legacy-Daten auf Vollständigkeit
   */
  static diagnoseLegacyData(legacyResult: AuditResult): {
    isComplete: boolean;
    missingFields: string[];
    warnings: string[];
    pageAnalysis: { url: string; missingAnalyses: string[] }[];
  } {
    const missingFields: string[] = [];
    const warnings: string[] = [];
    const pageAnalysis: { url: string; missingAnalyses: string[] }[] = [];

    // Check metadata
    if (!legacyResult.metadata) missingFields.push('metadata');
    if (!legacyResult.summary) missingFields.push('summary');
    if (!Array.isArray(legacyResult.pages)) missingFields.push('pages');

    // Analyze each page
    (legacyResult.pages || []).forEach((page: PageResult) => {
      const missingAnalyses: string[] = [];
      
      if (!page.accessibility) missingAnalyses.push('accessibility');
      if (!page.performance) missingAnalyses.push('performance');
      if (!page.seo) missingAnalyses.push('seo');
      if (!page.contentWeight) missingAnalyses.push('contentWeight');
      if (!page.mobileFriendliness) missingAnalyses.push('mobileFriendliness');

      if (missingAnalyses.length > 0) {
        pageAnalysis.push({
          url: page.url || 'Unknown URL',
          missingAnalyses
        });
      }

      // Check for empty accessibility issues
      if (page.accessibility) {
        const hasIssues = (Array.isArray(page.accessibility.errors) && page.accessibility.errors.length > 0) ||
                         (Array.isArray(page.accessibility.warnings) && page.accessibility.warnings.length > 0) ||
                         (Array.isArray(page.accessibility.notices) && page.accessibility.notices.length > 0);
        
        if (!hasIssues && (page.accessibility.score || 0) < 100) {
          warnings.push(`Page ${page.url} has accessibility score ${page.accessibility.score} but no issues listed`);
        }
      }
    });

    return {
      isComplete: missingFields.length === 0 && pageAnalysis.length === 0,
      missingFields,
      warnings,
      pageAnalysis
    };
  }
}

// ============================================================================
// CONVENIENCE FUNCTIONS
// ============================================================================

/**
 * High-level Convenience-Function: Konvertiert und validiert Legacy-Daten
 */
export function convertAndValidateAuditData(legacyResult: AuditResult): StrictAuditData {
  console.log('🚀 Starting audit data conversion and validation...');
  
  // First, diagnose the legacy data
  const diagnosis = AuditDataAdapter.diagnoseLegacyData(legacyResult);
  
  if (!diagnosis.isComplete) {
    console.warn('⚠️ Legacy data is incomplete:');
    console.warn('   Missing fields:', diagnosis.missingFields);
    console.warn('   Incomplete pages:', diagnosis.pageAnalysis.length);
    
    if (diagnosis.warnings.length > 0) {
      console.warn('   Warnings:');
      diagnosis.warnings.forEach(warning => console.warn(`     - ${warning}`));
    }
  }
  
  // Convert with adapter (which handles missing data gracefully)
  const strictData = AuditDataAdapter.convertToStrict(legacyResult);
  
  // Final validation
  validateStrictAuditData(strictData);
  
  console.log('✅ Audit data conversion completed successfully');
  return strictData;
}

/**
 * Safe Conversion: Versucht Konvertierung, fängt Fehler ab
 */
export function safeConvertAuditData(legacyResult: AuditResult): {
  success: boolean;
  data?: StrictAuditData;
  error?: string;
  warnings: string[];
} {
  const warnings: string[] = [];
  
  try {
    const diagnosis = AuditDataAdapter.diagnoseLegacyData(legacyResult);
    warnings.push(...diagnosis.warnings);
    
    const strictData = AuditDataAdapter.convertToStrict(legacyResult);
    
    return {
      success: true,
      data: strictData,
      warnings
    };
    
  } catch (error) {
    return {
      success: false,
      error: error instanceof Error ? error.message : 'Unknown conversion error',
      warnings
    };
  }
}