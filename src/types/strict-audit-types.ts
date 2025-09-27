/**
 * 🔒 STRICT AUDIT TYPES - MANDATORY DATA STRUCTURES
 * 
 * Diese Interfaces erzwingen vollständige Datenstrukturen und verbieten
 * undefined/null-Werte für kritische Felder.
 * 
 * Wenn Daten fehlen, soll das System explizit fehlschlagen,
 * nicht mit leeren/Standard-Werten weiterarbeiten.
 */

// ============================================================================
// CORE DATA TYPES (non-optional, strictly typed)
// ============================================================================

/**
 * Strikte Accessibility Issue - alle Felder verpflichtend
 */
export interface StrictAccessibilityIssue {
  readonly code: string;              // WCAG rule code (required)
  readonly message: string;           // Human-readable message (required)  
  readonly type: 'error' | 'warning' | 'notice';  // Issue severity (required)
  readonly selector: string | null;   // CSS selector (null if not available, but never undefined)
  readonly context: string | null;    // HTML context (null if not available, but never undefined)
  readonly impact: 'minor' | 'moderate' | 'serious' | 'critical' | null; // Impact level
  readonly help: string | null;       // Help text (null if not available)
  readonly helpUrl: string | null;    // Help URL (null if not available)
}

/**
 * Strikte Performance-Metriken - alle Core Web Vitals verpflichtend
 */
export interface StrictPerformanceMetrics {
  readonly largestContentfulPaint: number;     // LCP in ms (required)
  readonly firstContentfulPaint: number;       // FCP in ms (required)
  readonly cumulativeLayoutShift: number;      // CLS score (required)
  readonly timeToFirstByte: number;            // TTFB in ms (required)
  readonly domContentLoaded: number;           // DOM loaded in ms (required)
  readonly loadComplete: number;               // Load complete in ms (required)
  readonly firstPaint: number;                 // First paint in ms (required)
}

/**
 * Strikte SEO-Daten - alle wichtigen Felder verpflichtend
 */
export interface StrictSEOData {
  readonly title: string;                      // Page title (required)
  readonly titleLength: number;               // Title length (required)
  readonly description: string;               // Meta description (required, can be empty string)
  readonly descriptionLength: number;         // Description length (required)
  readonly keywords: string;                  // Keywords (required, can be empty string)
  readonly h1Count: number;                   // H1 count (required)
  readonly h2Count: number;                   // H2 count (required)
  readonly h3Count: number;                   // H3 count (required)
  readonly totalImages: number;               // Total images (required)
  readonly imagesWithoutAlt: number;          // Images without alt (required)
  readonly imagesWithEmptyAlt: number;        // Images with empty alt (required)
}

/**
 * Strikte Content-Weight-Daten - alle Resource-Typen verpflichtend
 */
export interface StrictContentWeightData {
  readonly totalSize: number;                 // Total page size in bytes (required)
  readonly html: { size: number; files: number };     // HTML resources (required)
  readonly css: { size: number; files: number };      // CSS resources (required)  
  readonly javascript: { size: number; files: number }; // JS resources (required)
  readonly images: { size: number; files: number };   // Image resources (required)
  readonly other: { size: number; files: number };    // Other resources (required)
}

/**
 * Strikte Mobile-Friendliness-Empfehlung - alle Felder verpflichtend  
 */
export interface StrictMobileRecommendation {
  readonly category: 'Touch Targets' | 'Performance' | 'Media' | 'Forms' | 'Navigation' | 'Content';
  readonly priority: 'low' | 'medium' | 'high' | 'critical';
  readonly issue: string;              // Issue description (required)
  readonly recommendation: string;     // Recommendation (required)
  readonly impact: string;            // Impact description (required)
}

// ============================================================================
// PAGE-LEVEL STRICT INTERFACES 
// ============================================================================

/**
 * Strikte Accessibility-Ergebnisse - keine optionalen Felder
 */
export interface StrictPageAccessibility {
  readonly score: number;                              // 0-100 score (required)
  readonly errors: readonly StrictAccessibilityIssue[];        // Error issues (required array, can be empty)
  readonly warnings: readonly StrictAccessibilityIssue[];      // Warning issues (required array, can be empty)
  readonly notices: readonly StrictAccessibilityIssue[];       // Notice issues (required array, can be empty)
  readonly totalIssues: number;                        // Total count (required)
  readonly wcagLevel: 'A' | 'AA' | 'AAA' | 'none';    // WCAG compliance level (required)
}

/**
 * Strikte Performance-Ergebnisse - alle Metriken verpflichtend
 */
export interface StrictPagePerformance {
  readonly score: number;                              // 0-100 score (required)
  readonly grade: 'A' | 'B' | 'C' | 'D' | 'F';       // Letter grade (required)
  readonly coreWebVitals: StrictPerformanceMetrics;   // All Core Web Vitals (required)
  readonly issues: readonly string[];                  // Performance issues (required array, can be empty)
  readonly budgetViolations: number;                   // Budget violations count (required)
}

/**
 * Strikte SEO-Ergebnisse - alle wichtigen Daten verpflichtend
 */
export interface StrictPageSEO {
  readonly score: number;                              // 0-100 score (required)
  readonly grade: 'A' | 'B' | 'C' | 'D' | 'F';       // Letter grade (required)
  readonly metaTags: StrictSEOData;                   // Meta tag data (required)
  readonly issues: readonly string[];                 // SEO issues (required array, can be empty)
  readonly recommendations: readonly string[];        // SEO recommendations (required array, can be empty)
}

/**
 * Strikte Content-Weight-Ergebnisse - alle Resource-Daten verpflichtend
 */
export interface StrictPageContentWeight {
  readonly score: number;                              // 0-100 score (required)
  readonly grade: 'A' | 'B' | 'C' | 'D' | 'F';       // Letter grade (required)
  readonly resources: StrictContentWeightData;        // Resource breakdown (required)
  readonly optimizations: readonly string[];          // Optimization suggestions (required array, can be empty)
  readonly compressionRatio: number;                  // Compression ratio (required)
}

/**
 * Strikte Mobile-Friendliness-Ergebnisse - alle Empfehlungen verpflichtend
 */  
export interface StrictPageMobileFriendliness {
  readonly overallScore: number;                       // 0-100 score (required)
  readonly grade: 'A' | 'B' | 'C' | 'D' | 'F';       // Letter grade (required)
  readonly recommendations: readonly StrictMobileRecommendation[]; // Recommendations (required array, can be empty)
  readonly touchTargetIssues: number;                 // Touch target issues count (required)
  readonly responsiveIssues: number;                  // Responsive issues count (required)
}

// ============================================================================
// MAIN AUDIT RESULT INTERFACES
// ============================================================================

/**
 * Strikte Seiten-Ergebnisse - ALLE Analyse-Typen sind verpflichtend
 * 
 * Das System darf keine Seite ohne vollständige Analyse-Ergebnisse akzeptieren.
 * Wenn eine Analyse fehlschlägt, muss das explizit als Fehler behandelt werden.
 */
export interface StrictAuditPage {
  readonly url: string;                                // Page URL (required)
  readonly title: string;                              // Page title (required)
  readonly status: 'passed' | 'failed' | 'crashed';   // Test status (required)
  readonly duration: number;                           // Test duration in ms (required)
  readonly testedAt: string;                          // ISO timestamp (required)
  
  // ALLE Analyse-Typen sind verpflichtend - keine optionalen Felder
  readonly accessibility: StrictPageAccessibility;     // Accessibility results (REQUIRED)
  readonly performance: StrictPagePerformance;         // Performance results (REQUIRED)
  readonly seo: StrictPageSEO;                        // SEO results (REQUIRED)
  readonly contentWeight: StrictPageContentWeight;     // Content weight results (REQUIRED)
  readonly mobileFriendliness: StrictPageMobileFriendliness; // Mobile results (REQUIRED)
}

/**
 * Strikte Audit-Zusammenfassung - alle Zählungen verpflichtend
 */
export interface StrictAuditSummary {
  readonly totalPages: number;                         // Total discovered pages (required)
  readonly testedPages: number;                        // Actually tested pages (required)
  readonly passedPages: number;                        // Passed pages (required)
  readonly failedPages: number;                        // Failed pages (required)
  readonly crashedPages: number;                       // Crashed pages (required)
  readonly redirectPages: number;                      // Redirect pages (required)
  readonly totalErrors: number;                        // Total errors across all pages (required)
  readonly totalWarnings: number;                      // Total warnings across all pages (required)
  readonly averageScore: number;                       // Average accessibility score (required)
  readonly overallGrade: 'A' | 'B' | 'C' | 'D' | 'F'; // Overall grade (required)
}

/**
 * Strikte Audit-Metadaten - alle Felder verpflichtend
 */
export interface StrictAuditMetadata {
  readonly version: string;                            // Data format version (required)
  readonly timestamp: string;                          // ISO timestamp (required)
  readonly sitemapUrl: string;                        // Source sitemap URL (required)
  readonly toolVersion: string;                       // AuditMySite version (required)
  readonly duration: number;                          // Total audit duration in ms (required)
  readonly configuration: {                           // Test configuration (required)
    readonly maxPages: number;
    readonly timeout: number;
    readonly standard: string;
    readonly features: readonly string[];
  };
}

/**
 * HAUPT-INTERFACE: Strikte Audit-Daten
 * 
 * Dieses Interface erzwingt vollständige Daten für alle Bereiche.
 * Keine optionalen Felder, keine undefined-Werte.
 */
export interface StrictAuditData {
  readonly metadata: StrictAuditMetadata;              // Metadata (REQUIRED)
  readonly summary: StrictAuditSummary;               // Summary (REQUIRED) 
  readonly pages: readonly StrictAuditPage[];         // Pages array (REQUIRED, min length 1)
  readonly systemPerformance: {                       // System performance (REQUIRED)
    readonly testCompletionTimeSeconds: number;
    readonly averageTimePerPageMs: number;
    readonly throughputPagesPerMinute: number;
    readonly memoryUsageMB: number;
    readonly efficiency: number;
  };
}

// ============================================================================
// TYPE GUARDS AND VALIDATION UTILITIES
// ============================================================================

/**
 * Type Guard: Prüft ob ein Objekt StrictAuditData ist
 */
export function isStrictAuditData(data: any): data is StrictAuditData {
  return (
    data &&
    typeof data === 'object' &&
    data.metadata &&
    data.summary &&
    Array.isArray(data.pages) &&
    data.pages.length > 0 &&
    data.systemPerformance
  );
}

/**
 * Type Guard: Prüft ob eine Seite alle verpflichtenden Analysen hat
 */
export function hasCompleteAnalysis(page: any): page is StrictAuditPage {
  return (
    page &&
    page.accessibility &&
    page.performance &&
    page.seo &&
    page.contentWeight &&
    page.mobileFriendliness &&
    typeof page.accessibility.score === 'number' &&
    typeof page.performance.score === 'number' &&
    typeof page.seo.score === 'number' &&
    typeof page.contentWeight.score === 'number' &&
    typeof page.mobileFriendliness.overallScore === 'number'
  );
}

// ============================================================================
// ERROR CLASSES FOR STRICT VALIDATION
// ============================================================================

/**
 * Fehler für unvollständige Audit-Daten
 */
export class IncompleteAuditDataError extends Error {
  constructor(
    message: string,
    public readonly missingFields: string[],
    public readonly pageUrl?: string
  ) {
    super(`Incomplete audit data: ${message}. Missing fields: ${missingFields.join(', ')}`);
    this.name = 'IncompleteAuditDataError';
  }
}

/**
 * Fehler für fehlende Analyse-Ergebnisse
 */
export class MissingAnalysisError extends Error {
  constructor(
    public readonly analysisType: string,
    public readonly pageUrl: string,
    public readonly reason?: string
  ) {
    super(`Missing ${analysisType} analysis for page: ${pageUrl}${reason ? `. Reason: ${reason}` : ''}`);
    this.name = 'MissingAnalysisError';
  }
}