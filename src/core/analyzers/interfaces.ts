/**
 * Core analyzer interfaces for AuditMySite
 * 
 * This file defines the contracts that all analyzers must implement,
 * ensuring consistent behavior and enabling proper dependency injection.
 */

import { Page } from 'playwright';

/**
 * Base interface for all analyzers
 */
export interface IAnalyzer<TResult = any, TOptions = any> {
  /**
   * Unique identifier for this analyzer type
   */
  readonly type: AnalyzerType;

  /**
   * Human-readable name for this analyzer
   */
  readonly name: string;

  /**
   * Initialize the analyzer (if needed)
   */
  initialize?(): Promise<void>;

  /**
   * Perform analysis on a page
   * 
   * @param page - Playwright page instance
   * @param url - URL being analyzed
   * @param options - Analyzer-specific options
   */
  analyze(page: Page, url: string, options?: TOptions): Promise<TResult>;

  /**
   * Cleanup resources (if needed)
   */
  cleanup?(): Promise<void>;
}

/**
 * Analyzer types for better type safety
 */
export type AnalyzerType = 
  | 'accessibility'
  | 'performance' 
  | 'mobile-performance'
  | 'seo'
  | 'content-weight'
  | 'mobile-friendliness'
  | 'security-headers'
  | 'structured-data';

/**
 * Analysis result metadata
 */
export interface AnalysisResultMetadata {
  analyzerType: AnalyzerType;
  analyzerName: string;
  analysisTime: number;
  timestamp: Date;
  url: string;
}

/**
 * Base analysis result
 */
export interface BaseAnalysisResult {
  readonly metadata: AnalysisResultMetadata;
  readonly success: boolean;
  readonly error?: string;
}

/**
 * Accessibility analysis result
 */
export interface AccessibilityAnalysisResult extends BaseAnalysisResult {
  readonly metadata: AnalysisResultMetadata & { analyzerType: 'accessibility' };
  readonly score: number;
  readonly grade: string;
  readonly issues: AccessibilityIssue[];
  readonly recommendations: string[];
}

/**
 * Performance analysis result  
 */
export interface PerformanceAnalysisResult extends BaseAnalysisResult {
  readonly metadata: AnalysisResultMetadata & { analyzerType: 'performance' };
  readonly score: number;
  readonly grade: string;
  readonly coreWebVitals: {
    lcp: number;
    fcp: number; 
    cls: number;
    ttfb: number;
  };
  readonly recommendations: string[];
}

/**
 * Content weight analysis result
 */
export interface ContentWeightAnalysisResult extends BaseAnalysisResult {
  readonly metadata: AnalysisResultMetadata & { analyzerType: 'content-weight' };
  readonly score: number;
  readonly grade: string;
  readonly contentWeight: {
    total: number;
    html: number;
    css: number;
    javascript: number;
    images: number;
    fonts: number;
    other: number;
  };
  readonly recommendations: string[];
}

/**
 * SEO analysis result
 */
export interface SEOAnalysisResult extends BaseAnalysisResult {
  readonly metadata: AnalysisResultMetadata & { analyzerType: 'seo' };
  readonly score: number;
  readonly grade: string;
  readonly metaTags: {
    title: { content: string; length: number };
    description: { content: string; length: number };
    keywords?: { content: string };
  };
  readonly headingStructure: Record<string, number>;
  readonly recommendations: string[];
}

/**
 * Accessibility issue details
 */
export interface AccessibilityIssue {
  readonly code: string;
  readonly message: string;
  readonly type: 'error' | 'warning' | 'notice';
  readonly selector?: string;
  readonly context?: string;
  readonly impact?: string;
  readonly help?: string;
  readonly helpUrl?: string;
}

/**
 * Analysis options base interface
 */
export interface BaseAnalysisOptions {
  readonly timeout?: number;
  readonly verbose?: boolean;
  readonly maxConcurrent?: number;
}

/**
 * Factory interface for creating analyzers
 */
export interface IAnalyzerFactory {
  /**
   * Create an analyzer of the specified type
   */
  createAnalyzer<T extends IAnalyzer>(type: AnalyzerType): T;
  
  /**
   * Get all available analyzer types
   */
  getAvailableTypes(): AnalyzerType[];
  
  /**
   * Check if a specific analyzer type is available
   */
  isAvailable(type: AnalyzerType): boolean;
}

/**
 * Logger interface for structured logging
 */
export interface ILogger {
  debug(message: string, data?: any): void;
  info(message: string, data?: any): void;
  warn(message: string, data?: any): void;
  error(message: string, error?: Error | any): void;
  success(message: string, data?: any): void;
  child?(prefix: string): ILogger;
}

/**
 * Analysis orchestrator interface
 */
export interface IAnalysisOrchestrator {
  /**
   * Run multiple analyzers on a page
   */
  runAnalysis(
    page: Page, 
    url: string, 
    analyzerTypes: AnalyzerType[], 
    options?: BaseAnalysisOptions
  ): Promise<BaseAnalysisResult[]>;
  
  /**
   * Check which analyzers are available
   */
  getAvailableAnalyzers(): AnalyzerType[];
}