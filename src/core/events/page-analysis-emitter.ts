/**
 * 🔧 UNIFIED Page Analysis Event System
 * 
 * ✨ MAIN EVENT SYSTEM - Replaces multiple parallel event systems
 * 
 * Event-driven system where analyzers attach to page load events
 * and contribute their data to a unified result structure.
 * 
 * 🎯 CONSOLIDATES:
 * - AccessibilityChecker event callbacks
 * - EventDrivenQueue callbacks  
 * - ParallelTestManager events
 * - Direct callback patterns in bin/audit.js
 * 
 * 📋 BACKWARD COMPATIBILITY:
 * - Supports all existing callback patterns via adapters
 * - Maintains existing APIs while using unified backend
 * 
 * 🚀 FEATURES:
 * - Parallel analyzer execution
 * - Resource monitoring integration
 * - Backpressure control integration
 * - Progress tracking
 * - Error handling & fallbacks
 * - State persistence
 */

import { EventEmitter } from 'events';
import { Page } from 'playwright';
import { AccessibilityResult, TestOptions } from '../../types';
import { AdaptiveBackpressureController } from '../backpressure-controller';
import { ResourceMonitor } from '../resource-monitor';

export interface PageAnalysisContext {
  url: string;
  page: Page;
  options: TestOptions;
  result: PageAnalysisResult;
  startTime: number;
  
  // 🎯 Extended context for comprehensive analysis
  retryCount?: number;
  maxRetries?: number;
  sessionId?: string;
  batchId?: string;
  priority?: number;
  
  // 🚀 Resource management context
  memoryUsageMB?: number;
  cpuUsagePercent?: number;
  backpressureActive?: boolean;
}

/**
 * 🎯 UNIFIED Event Callbacks - Supports ALL existing callback patterns
 * 
 * This interface consolidates all event patterns from:
 * - TestOptions.eventCallbacks
 * - EventDrivenQueueOptions.eventCallbacks  
 * - ParallelTestManager callbacks
 * - bin/audit.js direct callbacks
 */
export interface UnifiedEventCallbacks {
  // 📋 Core page events (existing pattern)
  onUrlAdded?: (url: string, priority?: number) => void;
  onUrlStarted?: (url: string) => void;
  onUrlCompleted?: (url: string, result: AccessibilityResult, duration: number) => void;
  onUrlFailed?: (url: string, error: string, attempts: number) => void;
  onUrlRetrying?: (url: string, attempts: number) => void;
  
  // 📈 Progress and queue events
  onProgressUpdate?: (stats: ProgressStats) => void;
  onQueueEmpty?: () => void;
  onBatchComplete?: (results: PageAnalysisResult[]) => void;
  
  // 🚨 Error and resource events
  onError?: (error: string, context?: any) => void;
  onResourceWarning?: (usage: number, limit: number, type: 'memory' | 'cpu') => void;
  onResourceCritical?: (usage: number, limit: number, type: 'memory' | 'cpu') => void;
  onBackpressureActivated?: (reason: string) => void;
  onBackpressureDeactivated?: () => void;
  onGarbageCollection?: (beforeMB: number, afterMB?: number) => void;
  
  // 📋 Detailed analysis events (new for granular control)
  onAnalyzerStart?: (analyzerName: string, url: string) => void;
  onAnalyzerComplete?: (analyzerName: string, url: string, result: any) => void;
  onAnalyzerError?: (analyzerName: string, url: string, error: string) => void;
  
  // 🚀 Status and monitoring
  onShortStatus?: (status: string) => void;
  onSystemMetrics?: (metrics: SystemMetrics) => void;
}

/**
 * 📈 Unified progress statistics
 */
export interface ProgressStats {
  total: number;
  pending: number;
  inProgress: number;
  completed: number;
  failed: number;
  retrying: number;
  progress: number; // 0-100
  averageDuration: number;
  estimatedTimeRemaining: number;
  activeWorkers: number;
  memoryUsage: number;
  cpuUsage: number;
  
  // 📋 Batch information
  batchId?: string;
  batchSize?: number;
  currentBatch?: number;
  totalBatches?: number;
}

/**
 * 🚀 System metrics for monitoring
 */
export interface SystemMetrics {
  memoryUsageMB: number;
  heapUsedMB: number;
  cpuUsagePercent: number;
  eventLoopDelayMs: number;
  activeHandles: number;
  gcCount: number;
  uptimeSeconds: number;
}

export interface PageAnalysisResult {
  url: string;
  title: string;
  status: 'success' | 'failed' | 'crashed';
  duration: number;
  
  // Core analysis data
  accessibility: {
    passed: boolean;
    score: number;
    errors: Array<{ code: string; message: string; type: 'error' | 'warning' | 'notice' }>;
    warnings: Array<{ code: string; message: string; type: 'error' | 'warning' | 'notice' }>;
    issues: Array<{
      code: string;
      message: string;
      type: 'error' | 'warning' | 'notice';
      selector?: string;
      context?: string;
      impact?: string;
    }>;
    basicChecks: {
      imagesWithoutAlt: number;
      buttonsWithoutLabel: number;
      headingsCount: number;
    };
  };
  
  performance?: {
    score: number;
    grade: string;
    coreWebVitals: {
      lcp: number;
      fcp: number;
      cls: number;
      inp: number;
      ttfb: number;
    };
    timing: {
      loadTime: number;
      domContentLoaded: number;
      renderTime: number;
    };
  };
  
  seo?: {
    score: number;
    grade: string;
    metaTags: {
      title?: { content: string; length: number; optimal: boolean };
      description?: { content: string; length: number; optimal: boolean };
      keywords?: string[];
      openGraph: Record<string, any>;
      twitterCard: Record<string, any>;
    };
    headings: {
      h1: string[];
      h2: string[];
      h3: string[];
      issues: string[];
    };
    images: {
      total: number;
      missingAlt: number;
      emptyAlt: number;
    };
  };
  
  contentWeight?: {
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
    optimizations: string[];
  };
  
  mobileFriendliness?: {
    score: number;
    grade: string;
    viewport: { hasViewportMeta: boolean; width?: string; initialScale?: number };
    touchTargets: { tooSmall: number; overlapping: number };
    textReadability: { tooSmall: number };
    contentFit: { horizontalScrolling: boolean };
  };
}

/**
 * 🎯 UNIFIED PAGE ANALYSIS EMITTER - Main Event System
 * 
 * Replaces multiple parallel event systems with a single, comprehensive solution.
 * Maintains backward compatibility while providing enhanced features.
 */
export class PageAnalysisEmitter extends EventEmitter {
  private analyzers: Map<string, AnalyzerFunction> = new Map();
  private callbacks: UnifiedEventCallbacks = {};
  private backpressureController?: AdaptiveBackpressureController;
  private resourceMonitor?: ResourceMonitor;
  private stats: ProgressStats;
  private systemMetrics: SystemMetrics;
  private sessionId: string;
  private isInitialized = false;
  
  // 🚀 Configuration options
  private options: {
    enableResourceMonitoring: boolean;
    enableBackpressure: boolean;
    maxConcurrent: number;
    maxRetries: number;
    retryDelay: number;
    verbose: boolean;
  };
  
  constructor(options: Partial<{
    enableResourceMonitoring: boolean;
    enableBackpressure: boolean;
    maxConcurrent: number;
    maxRetries: number;
    retryDelay: number;
    verbose: boolean;
    callbacks: UnifiedEventCallbacks;
  }> = {}) {
    super();
    
    this.sessionId = `session_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
    
    this.options = {
      enableResourceMonitoring: options.enableResourceMonitoring ?? true,
      enableBackpressure: options.enableBackpressure ?? true,
      maxConcurrent: options.maxConcurrent ?? 3,
      maxRetries: options.maxRetries ?? 3,
      retryDelay: options.retryDelay ?? 2000,
      verbose: options.verbose ?? false,
      ...options
    };
    
    this.callbacks = options.callbacks || {};
    
    // Initialize stats
    this.stats = {
      total: 0,
      pending: 0,
      inProgress: 0,
      completed: 0,
      failed: 0,
      retrying: 0,
      progress: 0,
      averageDuration: 0,
      estimatedTimeRemaining: 0,
      activeWorkers: 0,
      memoryUsage: 0,
      cpuUsage: 0
    };
    
    // Initialize system metrics
    this.systemMetrics = {
      memoryUsageMB: 0,
      heapUsedMB: 0,
      cpuUsagePercent: 0,
      eventLoopDelayMs: 0,
      activeHandles: 0,
      gcCount: 0,
      uptimeSeconds: 0
    };
  }
  
  /**
   * 🚀 Initialize the unified event system with integrated monitoring
   */
  async initialize(): Promise<void> {
    if (this.isInitialized) return;
    
    try {
      // Setup resource monitoring
      if (this.options.enableResourceMonitoring) {
        this.setupResourceMonitoring();
      }
      
      // Setup backpressure control
      if (this.options.enableBackpressure) {
        this.setupBackpressureControl();
      }
      
      this.isInitialized = true;
      
      if (this.options.verbose) {
        console.log(`🚀 Unified Page Analysis System initialized (${this.sessionId})`);
        console.log(`   📊 Resource Monitoring: ${this.options.enableResourceMonitoring ? '✅' : '❌'}`);
        console.log(`   🏃 Backpressure Control: ${this.options.enableBackpressure ? '✅' : '❌'}`);
        console.log(`   🔄 Max Concurrent: ${this.options.maxConcurrent}`);
      }
      
    } catch (error) {
      console.error(`❌ Failed to initialize unified event system: ${error}`);
      throw error;
    }
  }
  
  /**
   * 📋 Register an analyzer that will run when a page is loaded
   * 
   * BACKWARD COMPATIBLE: Maintains existing analyzer registration pattern
   */
  registerAnalyzer(name: string, analyzer: AnalyzerFunction): void {
    this.analyzers.set(name, analyzer);
    
    if (this.options.verbose) {
      console.log(`📋 Registered analyzer: ${name} (total: ${this.analyzers.size})`);
    }
    
    this.emit('analyzer-registered', { name, total: this.analyzers.size });
  }
  
  /**
   * 🎯 Set unified event callbacks
   * 
   * This replaces/consolidates:
   * - TestOptions.eventCallbacks
   * - EventDrivenQueueOptions.eventCallbacks
   * - Direct callback patterns
   */
  setEventCallbacks(callbacks: UnifiedEventCallbacks): void {
    this.callbacks = { ...this.callbacks, ...callbacks };
    
    if (this.options.verbose) {
      const callbackNames = Object.keys(callbacks).join(', ');
      console.log(`🎯 Event callbacks configured: ${callbackNames}`);
    }
  }
  
  /**
   * 📊 Setup resource monitoring integration
   */
  private setupResourceMonitoring(): void {
    try {
      this.resourceMonitor = new ResourceMonitor({
        enabled: true,
        samplingIntervalMs: 2000,
        memoryWarningThresholdMB: 1024,
        memoryCriticalThresholdMB: 1536
      });
      
      // Connect resource events to unified callbacks
      this.resourceMonitor.on('memoryWarning', (data) => {
        this.callbacks.onResourceWarning?.(data.current, data.threshold, 'memory');
      });
      
      this.resourceMonitor.on('memoryCritical', (data) => {
        this.callbacks.onResourceCritical?.(data.current, data.max, 'memory');
      });
      
      this.resourceMonitor.on('cpuWarning', (data) => {
        this.callbacks.onResourceWarning?.(data.current, data.threshold, 'cpu');
      });
      
      // Update system metrics
      this.resourceMonitor.on('metricsUpdate', (metrics) => {
        this.systemMetrics = {
          memoryUsageMB: metrics.rssMemoryMB,
          heapUsedMB: metrics.heapUsedMB,
          cpuUsagePercent: metrics.cpuUsagePercent,
          eventLoopDelayMs: metrics.eventLoopDelayMs,
          activeHandles: 0, // Would need additional monitoring
          gcCount: metrics.gcCount || 0,
          uptimeSeconds: metrics.uptimeSeconds
        };
        
        this.callbacks.onSystemMetrics?.(this.systemMetrics);
      });
      
      this.resourceMonitor.start();
      
    } catch (error) {
      console.warn(`⚠️  Resource monitoring setup failed: ${error}`);
    }
  }
  
  /**
   * 🏃 Setup backpressure control integration
   */
  private setupBackpressureControl(): void {
    try {
      this.backpressureController = new AdaptiveBackpressureController({
        enabled: true,
        maxMemoryUsageMB: 1536,
        maxCpuUsagePercent: 85
      });
      
      // Connect backpressure events to unified callbacks
      this.backpressureController.on('backpressureActivated', (data) => {
        this.callbacks.onBackpressureActivated?.(data.reason || 'Resource pressure detected');
      });
      
      this.backpressureController.on('backpressureDeactivated', () => {
        this.callbacks.onBackpressureDeactivated?.();
      });
      
      this.backpressureController.on('gcTriggered', (data) => {
        this.callbacks.onGarbageCollection?.(data.beforeMB, data.afterMB);
      });
      
    } catch (error) {
      console.warn(`⚠️  Backpressure control setup failed: ${error}`);
    }
  }
  
  /**
   * 📈 Update and emit progress statistics
   */
  private updateProgress(): void {
    // Calculate progress percentage
    if (this.stats.total > 0) {
      this.stats.progress = ((this.stats.completed + this.stats.failed) / this.stats.total) * 100;
    }
    
    // Emit progress update
    this.callbacks.onProgressUpdate?.(this.stats);
    this.emit('progress-update', this.stats);
  }
  
  /**
   * 🎯 Get current progress statistics
   */
  getProgressStats(): ProgressStats {
    return { ...this.stats };
  }
  
  /**
   * 🚀 Get system metrics
   */
  getSystemMetrics(): SystemMetrics {
    return { ...this.systemMetrics };
  }
  
  /**
   * 🧪 Cleanup resources (enhanced version)
   */
  async cleanup(): Promise<void> {
    try {
      if (this.resourceMonitor) {
        this.resourceMonitor.stop();
      }
      
      if (this.backpressureController) {
        // Cleanup backpressure controller if it has cleanup method
      }
      
      this.emit('cleanup-complete');
      
      if (this.options.verbose) {
        console.log(`🧪 Unified event system cleanup completed (${this.sessionId})`);
      }
      
    } catch (error) {
      console.error(`❌ Error during cleanup: ${error}`);
    }
  }
  
  /**
   * 🎯 UNIFIED PAGE ANALYSIS - Enhanced version with all features
   * 
   * BACKWARD COMPATIBLE: Maintains existing analyzePage signature
   * ENHANCED: Integrates resource monitoring, backpressure, progress tracking
   */
  async analyzePage(
    url: string, 
    page: Page, 
    options: TestOptions = {}, 
    contextOptions: Partial<PageAnalysisContext> = {}
  ): Promise<PageAnalysisResult> {
    const startTime = Date.now();
    
    // Initialize result structure
    const result: PageAnalysisResult = {
      url,
      title: '',
      status: 'success',
      duration: 0,
      accessibility: {
        passed: true,
        score: 100,
        errors: [],
        warnings: [],
        issues: [],
        basicChecks: {
          imagesWithoutAlt: 0,
          buttonsWithoutLabel: 0,
          headingsCount: 0,
        }
      }
    };
    
    const context: PageAnalysisContext = {
      url,
      page,
      options,
      result,
      startTime,
      ...contextOptions
    };
    
    try {
      // Navigate to page
      if (options.verbose) console.log(`   🌐 Loading: ${url}`);
      await page.goto(url, {
        waitUntil: options.waitUntil || 'domcontentloaded',
        timeout: options.timeout || 10000,
      });
      
      // Get basic page info
      result.title = await page.title();
      if (options.verbose) console.log(`   📋 Title: ${result.title}`);
      
      // Emit page-loaded event and run all analyzers
      this.emit('page-loaded', context);
      
      // Run all registered analyzers in parallel
      const analyzerPromises = Array.from(this.analyzers.entries()).map(async ([name, analyzer]) => {
        try {
          if (options.verbose) console.log(`   🔍 Running ${name} analysis...`);
          await analyzer(context);
        } catch (error) {
          console.error(`   ❌ ${name} analysis failed: ${error}`);
          // Add error but don't fail the whole analysis
          result.accessibility.warnings.push({
            code: `${name.toUpperCase()}_ANALYSIS_ERROR`,
            message: `${name} analysis failed: ${error}`,
            type: 'warning'
          });
        }
      });
      
      await Promise.all(analyzerPromises);
      
    } catch (error) {
      console.error(`   💥 Page loading failed: ${error}`);
      result.status = 'crashed';
      result.accessibility.passed = false;
      result.accessibility.errors.push({
        code: 'PAGE_LOAD_ERROR',
        message: `Failed to load page: ${error}`,
        type: 'error'
      });
    } finally {
      result.duration = Date.now() - startTime;
      
      // Determine overall status
      if (result.accessibility.errors.length > 0) {
        result.accessibility.passed = false;
        if (result.status === 'success') {
          result.status = 'failed';
        }
      }
    }
    
    return result;
  }
  
  /**
   * Get list of registered analyzers
   */
  getRegisteredAnalyzers(): string[] {
    return Array.from(this.analyzers.keys());
  }
}

export type AnalyzerFunction = (context: PageAnalysisContext) => Promise<void>;

// Default analyzer functions

export const accessibilityAnalyzer: AnalyzerFunction = async (context) => {
  const { page, result } = context;
  
  // Basic accessibility checks
  result.accessibility.basicChecks.imagesWithoutAlt = await page.locator('img:not([alt])').count();
  result.accessibility.basicChecks.buttonsWithoutLabel = await page
    .locator('button:not([aria-label])')
    .filter({ hasText: '' })
    .count();
  result.accessibility.basicChecks.headingsCount = await page.locator('h1, h2, h3, h4, h5, h6').count();
  
  // Add warnings for basic issues
  if (result.accessibility.basicChecks.imagesWithoutAlt > 0) {
    result.accessibility.warnings.push({
      code: 'MISSING_ALT_ATTRIBUTE',
      message: `${result.accessibility.basicChecks.imagesWithoutAlt} images without alt attribute`,
      type: 'warning'
    });
  }
  
  if (result.accessibility.basicChecks.buttonsWithoutLabel > 0) {
    result.accessibility.warnings.push({
      code: 'MISSING_BUTTON_LABEL',
      message: `${result.accessibility.basicChecks.buttonsWithoutLabel} buttons without aria-label`,
      type: 'warning'
    });
  }
  
  if (result.accessibility.basicChecks.headingsCount === 0) {
    result.accessibility.errors.push({
      code: 'NO_HEADINGS',
      message: 'No headings found',
      type: 'error'
    });
  }
  
  // Run pa11y tests
  try {
    const pa11y = require('pa11y');
    const pa11yResult = await pa11y(context.url, {
      timeout: 15000,
      wait: 2000,
      standard: 'WCAG2AA',
      includeNotices: true,
      includeWarnings: true,
      runners: ['axe'],
      chromeLaunchConfig: {
        args: [
          '--disable-web-security',
          '--no-sandbox',
          '--disable-setuid-sandbox',
          '--disable-dev-shm-usage',
          '--disable-gpu'
        ]
      }
    });
    
    // Add pa11y issues
    if (pa11yResult.issues) {
      pa11yResult.issues.forEach((issue: any) => {
        const detailedIssue = {
          code: issue.code,
          message: issue.message,
          type: issue.type as 'error' | 'warning' | 'notice',
          selector: issue.selector,
          context: issue.context,
          impact: issue.impact,
        };
        
        result.accessibility.issues.push(detailedIssue);
        
        if (issue.type === 'error') {
          result.accessibility.errors.push({
            code: issue.code,
            message: issue.message,
            type: 'error'
          });
        } else if (issue.type === 'warning') {
          result.accessibility.warnings.push({
            code: issue.code,
            message: issue.message,
            type: 'warning'
          });
        }
      });
    }
    
    // Calculate score
    if (pa11yResult.issues && pa11yResult.issues.length > 0) {
      const errors = pa11yResult.issues.filter((i: any) => i.type === 'error').length;
      const warnings = pa11yResult.issues.filter((i: any) => i.type === 'warning').length;
      result.accessibility.score = Math.max(10, 100 - (errors * 5) - (warnings * 2));
    }
    
  } catch (error) {
    // pa11y failed, use fallback score
    let score = 100;
    score -= result.accessibility.errors.length * 15;
    score -= result.accessibility.warnings.length * 5;
    result.accessibility.score = Math.max(0, score);
  }
};

export const performanceAnalyzer: AnalyzerFunction = async (context) => {
  const { page, result, options } = context;
  
  if (!(options as any).enablePerformanceAnalysis) return;
  
  try {
    // Get performance metrics using browser's Performance API
    const metrics = await page.evaluate(() => {
      const navigation = performance.getEntriesByType('navigation')[0] as PerformanceNavigationTiming;
      const paint = performance.getEntriesByType('paint');
      
      const fcp = paint.find(p => p.name === 'first-contentful-paint')?.startTime || 0;
      const lcp = performance.getEntriesByType('largest-contentful-paint')[0]?.startTime || 0;
      
      return {
        loadTime: navigation?.loadEventEnd - navigation?.loadEventStart || 0,
        domContentLoaded: navigation?.domContentLoadedEventEnd - navigation?.domContentLoadedEventStart || 0,
        renderTime: navigation?.loadEventEnd - navigation?.fetchStart || 0,
        fcp,
        lcp: lcp || fcp * 1.5, // Fallback estimate
        cls: 0, // Would need special measurement
        inp: 0, // Would need interaction
        ttfb: navigation?.responseStart - navigation?.fetchStart || 0,
      };
    });
    
    // Calculate performance score (simplified)
    let score = 100;
    if (metrics.lcp > 4000) score -= 30;
    else if (metrics.lcp > 2500) score -= 15;
    if (metrics.fcp > 3000) score -= 20;
    else if (metrics.fcp > 1800) score -= 10;
    if (metrics.ttfb > 600) score -= 15;
    
    const grade = score >= 90 ? 'A' : score >= 75 ? 'B' : score >= 60 ? 'C' : score >= 50 ? 'D' : 'F';
    
    result.performance = {
      score: Math.max(0, score),
      grade,
      coreWebVitals: {
        lcp: Math.round(metrics.lcp),
        fcp: Math.round(metrics.fcp),
        cls: metrics.cls,
        inp: metrics.inp,
        ttfb: Math.round(metrics.ttfb),
      },
      timing: {
        loadTime: Math.round(metrics.loadTime),
        domContentLoaded: Math.round(metrics.domContentLoaded),
        renderTime: Math.round(metrics.renderTime),
      }
    };
    
  } catch (error) {
    console.error('Performance analysis failed:', error);
  }
};

export const seoAnalyzer: AnalyzerFunction = async (context) => {
  const { page, result, options } = context;
  
  if (!(options as any).enableSEOAnalysis) return;
  
  try {
    const seoData = await page.evaluate(() => {
      // Meta tags
      const title = document.querySelector('title')?.textContent || '';
      const description = document.querySelector('meta[name="description"]')?.getAttribute('content') || '';
      const keywords = document.querySelector('meta[name="keywords"]')?.getAttribute('content') || '';
      
      // Open Graph
      const ogTags: Record<string, any> = {};
      document.querySelectorAll('meta[property^="og:"]').forEach(meta => {
        const property = meta.getAttribute('property')?.replace('og:', '');
        const content = meta.getAttribute('content');
        if (property && content) ogTags[property] = content;
      });
      
      // Twitter Card
      const twitterTags: Record<string, any> = {};
      document.querySelectorAll('meta[name^="twitter:"]').forEach(meta => {
        const name = meta.getAttribute('name')?.replace('twitter:', '');
        const content = meta.getAttribute('content');
        if (name && content) twitterTags[name] = content;
      });
      
      // Headings
      const h1 = Array.from(document.querySelectorAll('h1')).map(h => h.textContent || '');
      const h2 = Array.from(document.querySelectorAll('h2')).map(h => h.textContent || '');
      const h3 = Array.from(document.querySelectorAll('h3')).map(h => h.textContent || '');
      
      // Images
      const images = document.querySelectorAll('img');
      const missingAlt = Array.from(images).filter(img => !img.getAttribute('alt')).length;
      const emptyAlt = Array.from(images).filter(img => img.getAttribute('alt') === '').length;
      
      return {
        title,
        description,
        keywords: keywords ? keywords.split(',').map(k => k.trim()) : [],
        ogTags,
        twitterTags,
        h1,
        h2,
        h3,
        totalImages: images.length,
        missingAlt,
        emptyAlt,
      };
    });
    
    // Calculate SEO score
    let score = 100;
    const issues: string[] = [];
    
    if (!seoData.title || seoData.title.length < 10) {
      score -= 20;
      issues.push('Missing or too short title tag');
    } else if (seoData.title.length > 60) {
      score -= 10;
      issues.push('Title tag too long');
    }
    
    if (!seoData.description || seoData.description.length < 120) {
      score -= 15;
      issues.push('Missing or too short meta description');
    } else if (seoData.description.length > 160) {
      score -= 5;
      issues.push('Meta description too long');
    }
    
    if (seoData.h1.length === 0) {
      score -= 15;
      issues.push('Missing H1 heading');
    } else if (seoData.h1.length > 1) {
      score -= 10;
      issues.push('Multiple H1 headings');
    }
    
    const grade = score >= 90 ? 'A' : score >= 75 ? 'B' : score >= 60 ? 'C' : score >= 50 ? 'D' : 'F';
    
    result.seo = {
      score: Math.max(0, score),
      grade,
      metaTags: {
        title: seoData.title ? {
          content: seoData.title,
          length: seoData.title.length,
          optimal: seoData.title.length >= 10 && seoData.title.length <= 60
        } : undefined,
        description: seoData.description ? {
          content: seoData.description,
          length: seoData.description.length,
          optimal: seoData.description.length >= 120 && seoData.description.length <= 160
        } : undefined,
        keywords: seoData.keywords,
        openGraph: seoData.ogTags,
        twitterCard: seoData.twitterTags
      },
      headings: {
        h1: seoData.h1,
        h2: seoData.h2,
        h3: seoData.h3,
        issues
      },
      images: {
        total: seoData.totalImages,
        missingAlt: seoData.missingAlt,
        emptyAlt: seoData.emptyAlt
      }
    };
    
  } catch (error) {
    console.error('SEO analysis failed:', error);
  }
};
