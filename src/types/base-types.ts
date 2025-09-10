/**
 * 🏗️ Base Types for Modular Audit Architecture
 * 
 * Common interfaces, types, and utilities used across all analysis groups.
 * Provides consistent scoring, grading, and recommendation patterns.
 */

import { Page } from 'playwright';

// =============================================================================
// CORE TYPES
// =============================================================================

/** Grade levels used across all analysis groups */
export type Grade = 'A' | 'B' | 'C' | 'D' | 'F';

/** Certificate levels for visual achievement display */
export type CertificateLevel = 'PLATINUM' | 'GOLD' | 'SILVER' | 'BRONZE' | 'NEEDS_IMPROVEMENT';

/** Issue severity levels */
export type IssueSeverity = 'critical' | 'error' | 'warning' | 'notice' | 'info';

/** Recommendation priority levels */
export type RecommendationPriority = 'high' | 'medium' | 'low';

// =============================================================================
// BASE INTERFACES
// =============================================================================

/** Base interface for all analysis results */
export interface BaseAnalysisResult {
  /** Overall score for this analysis group (0-100) */
  overallScore: number;
  /** Letter grade based on score */
  grade: Grade;
  /** Certificate level for achievements */
  certificate: CertificateLevel;
  /** Timestamp when analysis was performed */
  analyzedAt: string;
  /** Analysis duration in milliseconds */
  duration: number;
  /** Analysis status */
  status: 'completed' | 'failed' | 'partial';
}

/** Base interface for issues found during analysis */
export interface BaseIssue {
  /** Unique identifier for this issue type */
  id: string;
  /** Issue severity level */
  severity: IssueSeverity;
  /** Human-readable issue description */
  message: string;
  /** Technical details about the issue */
  details?: string;
  /** CSS selector or location where issue was found */
  selector?: string;
  /** Code or context related to the issue */
  context?: string;
  /** Reference to documentation or standards */
  reference?: string;
}

/** Base interface for recommendations */
export interface BaseRecommendation {
  /** Unique identifier for this recommendation */
  id: string;
  /** Recommendation priority */
  priority: RecommendationPriority;
  /** Issue category this addresses */
  category: string;
  /** Short description of the issue */
  issue: string;
  /** Recommended action to take */
  recommendation: string;
  /** Expected impact of implementing the recommendation */
  impact: string;
  /** Estimated effort to implement (hours) */
  effort?: number;
  /** Potential improvement in score */
  scoreImprovement?: number;
}

/** Base analysis options for all analyzers */
export interface BaseAnalysisOptions {
  /** Maximum time to spend on analysis (ms) */
  timeout?: number;
  /** Whether to include detailed analysis */
  includeDetails?: boolean;
  /** Whether to generate recommendations */
  generateRecommendations?: boolean;
  /** Custom configuration for this analysis */
  config?: Record<string, unknown>;
}

// =============================================================================
// ANALYZER INTERFACE
// =============================================================================

/** Base interface that all analyzers must implement */
export interface BaseAnalyzer<TResult extends BaseAnalysisResult, TOptions extends BaseAnalysisOptions = BaseAnalysisOptions> {
  /** Perform the analysis and return results */
  analyze(page: Page, url: string, options?: TOptions): Promise<TResult>;
  
  /** Extract the overall score from analysis results */
  getScore(result: TResult): number;
  
  /** Calculate grade from score */
  getGrade(score: number): Grade;
  
  /** Calculate certificate level from score */
  getCertificateLevel(score: number): CertificateLevel;
  
  /** Extract recommendations from analysis results */
  getRecommendations(result: TResult): BaseRecommendation[];
  
  /** Get analyzer name for reporting */
  getName(): string;
  
  /** Get analyzer version for tracking */
  getVersion(): string;
}

// =============================================================================
// SCORING UTILITIES
// =============================================================================

/** Standard grade calculation based on score */
export function calculateGrade(score: number): Grade {
  if (score >= 90) return 'A';
  if (score >= 80) return 'B';
  if (score >= 70) return 'C';
  if (score >= 60) return 'D';
  return 'F';
}

/** Standard certificate level calculation based on score */
export function calculateCertificateLevel(score: number): CertificateLevel {
  if (score >= 95) return 'PLATINUM';
  if (score >= 85) return 'GOLD';
  if (score >= 70) return 'SILVER';
  if (score >= 60) return 'BRONZE';
  return 'NEEDS_IMPROVEMENT';
}

/** Calculate weighted average score */
export function calculateWeightedScore(scores: Array<{ score: number; weight: number }>): number {
  const totalWeight = scores.reduce((sum, item) => sum + item.weight, 0);
  if (totalWeight === 0) return 0;
  
  const weightedSum = scores.reduce((sum, item) => sum + (item.score * item.weight), 0);
  return Math.round(weightedSum / totalWeight);
}

/** Calculate overall score from multiple category scores */
export function calculateOverallScore(categoryScores: Record<string, number>, weights?: Record<string, number>): number {
  const categories = Object.keys(categoryScores);
  if (categories.length === 0) return 0;
  
  if (!weights) {
    // Equal weighting
    const totalScore = categories.reduce((sum, category) => sum + categoryScores[category], 0);
    return Math.round(totalScore / categories.length);
  }
  
  // Weighted scoring
  const weightedScores = categories.map(category => ({
    score: categoryScores[category],
    weight: weights[category] || 1
  }));
  
  return calculateWeightedScore(weightedScores);
}

// =============================================================================
// VALIDATION UTILITIES
// =============================================================================

/** Validate that a score is within the valid range */
export function validateScore(score: number, context?: string): number {
  if (typeof score !== 'number' || isNaN(score)) {
    console.warn(`Invalid score in ${context || 'unknown context'}: ${score}. Using 0.`);
    return 0;
  }
  
  if (score < 0) {
    console.warn(`Score below 0 in ${context || 'unknown context'}: ${score}. Using 0.`);
    return 0;
  }
  
  if (score > 100) {
    console.warn(`Score above 100 in ${context || 'unknown context'}: ${score}. Using 100.`);
    return 100;
  }
  
  return Math.round(score);
}

/** Create a base analysis result with common fields */
export function createBaseResult<T extends BaseAnalysisResult>(
  overallScore: number,
  partialResult: Omit<T, keyof BaseAnalysisResult>
): T {
  const validScore = validateScore(overallScore);
  
  const baseResult: BaseAnalysisResult = {
    overallScore: validScore,
    grade: calculateGrade(validScore),
    certificate: calculateCertificateLevel(validScore),
    analyzedAt: new Date().toISOString(),
    duration: 0, // Should be set by analyzer
    status: 'completed'
  };
  
  return { ...baseResult, ...partialResult } as T;
}

// =============================================================================
// COMMON CONFIGURATION
// =============================================================================

/** Global analysis configuration */
export interface GlobalAnalysisConfig {
  /** Enable verbose logging */
  verbose?: boolean;
  /** Maximum total analysis time (ms) */
  maxAnalysisTime?: number;
  /** Number of retries on failure */
  retries?: number;
  /** User agent to use for analysis */
  userAgent?: string;
  /** Viewport size for analysis */
  viewport?: {
    width: number;
    height: number;
  };
}

/** Default configuration values */
export const DEFAULT_CONFIG: Required<GlobalAnalysisConfig> = {
  verbose: false,
  maxAnalysisTime: 300000, // 5 minutes
  retries: 2,
  userAgent: 'AuditMySite/2.0 (Web Audit Tool)',
  viewport: {
    width: 1920,
    height: 1080
  }
};

// =============================================================================
// TIMING UTILITIES
// =============================================================================

/** Simple performance timer */
export class PerformanceTimer {
  private startTime: number;
  
  constructor() {
    this.startTime = performance.now();
  }
  
  /** Get elapsed time in milliseconds */
  getElapsed(): number {
    return Math.round(performance.now() - this.startTime);
  }
  
  /** Reset the timer */
  reset(): void {
    this.startTime = performance.now();
  }
}

/** Create a timer-enabled wrapper for async operations */
export async function withTiming<T>(
  operation: () => Promise<T>,
  context?: string
): Promise<{ result: T; duration: number }> {
  const timer = new PerformanceTimer();
  try {
    const result = await operation();
    const duration = timer.getElapsed();
    
    if (context) {
      console.log(`⏱️  ${context}: ${duration}ms`);
    }
    
    return { result, duration };
  } catch (error) {
    const duration = timer.getElapsed();
    if (context) {
      console.error(`❌ ${context} failed after ${duration}ms:`, error);
    }
    throw error;
  }
}

// =============================================================================
// URL UTILITIES
// =============================================================================

/** Validate URL format */
export function isValidUrl(urlString: string): boolean {
  try {
    new URL(urlString);
    return true;
  } catch {
    return false;
  }
}

/** Extract domain from URL */
export function extractDomain(url: string): string {
  try {
    return new URL(url).hostname;
  } catch {
    return 'unknown';
  }
}

/** Check if URL uses HTTPS */
export function isSecureUrl(url: string): boolean {
  try {
    return new URL(url).protocol === 'https:';
  } catch {
    return false;
  }
}
