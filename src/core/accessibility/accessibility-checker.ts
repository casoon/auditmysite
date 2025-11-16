/**
 * AccessibilityChecker v2 - Clean Architecture
 * 
 * This is a complete rewrite of the AccessibilityChecker with proper
 * separation of concerns, dependency injection, and typed interfaces.
 */

import { Browser, Page } from 'playwright';
import pa11y from 'pa11y';
import { AccessibilityResult, TestOptions, Pa11yIssue } from '../types';
import { BrowserPoolManager } from '../browser/browser-pool-manager';
import { Queue, QueueEventCallbacks } from '../queue';
import { 
  ILogger, 
  AnalyzerType, 
  BaseAnalysisOptions,
  BaseAnalysisResult 
} from '../analyzers/interfaces';
import { AnalyzerFactory, AnalyzerFactoryConfig } from '../analyzers/analyzer-factory';
import { AnalysisOrchestrator, OrchestratorConfig, AnalysisResults } from '../analysis/analysis-orchestrator';
import { createLogger } from '../logging/structured-logger';

/**
 * Configuration for AccessibilityChecker
 */
export interface AccessibilityCheckerConfig {
  readonly poolManager: BrowserPoolManager;
  readonly logger?: ILogger;
  readonly enableComprehensiveAnalysis?: boolean;
  readonly analyzerTypes?: AnalyzerType[];
  readonly qualityAnalysisOptions?: any; // TODO: Type this properly
}

/**
 * Options for testing pages
 */
export interface PageTestOptions extends BaseAnalysisOptions {
  readonly pa11yStandard?: 'WCAG2A' | 'WCAG2AA' | 'WCAG2AAA' | 'Section508';
  readonly includeNotices?: boolean;
  readonly includeWarnings?: boolean;
  readonly wait?: number;
  readonly hideElements?: string;
  readonly enableComprehensiveAnalysis?: boolean;
  readonly skipRedirects?: boolean;         // Skip 301/302 redirects (default: true)
}

/**
 * Result from testing a single page
 */
export interface PageTestResult {
  readonly url: string;
  readonly title: string;
  readonly accessibilityResult: AccessibilityResult;
  readonly comprehensiveAnalysis?: AnalysisResults;
  readonly duration: number;
  readonly timestamp: Date;
}

/**
 * Result from testing multiple pages
 */
export interface MultiPageTestResult {
  readonly results: PageTestResult[];
  readonly skippedUrls: string[];           // URLs skipped due to redirects/errors
  readonly totalDuration: number;
  readonly timestamp: Date;
}

/**
 * AccessibilityChecker v2 - Clean, focused implementation
 * 
 * Responsibilities:
 * - Core accessibility testing via pa11y
 * - Basic page analysis (images, buttons, headings)
 * - Orchestration of comprehensive analysis (via AnalysisOrchestrator)
 * - Browser pool management coordination
 */
export class AccessibilityChecker {
  private readonly config: Required<AccessibilityCheckerConfig>;
  private readonly logger: ILogger;
  private readonly analyzerFactory?: AnalyzerFactory;
  private readonly analysisOrchestrator?: AnalysisOrchestrator;
  
  constructor(config: AccessibilityCheckerConfig) {
    // Validate required dependencies
    if (!config.poolManager) {
      throw new Error('BrowserPoolManager is required');
    }

    // Set up configuration with defaults
    this.config = {
      poolManager: config.poolManager,
      logger: config.logger || createLogger('accessibility-checker'),
      enableComprehensiveAnalysis: config.enableComprehensiveAnalysis || false,
      analyzerTypes: config.analyzerTypes || [],
      qualityAnalysisOptions: config.qualityAnalysisOptions || {}
    };

    this.logger = this.config.logger;
    
    // Initialize analyzer factory if comprehensive analysis is enabled
    if (this.config.enableComprehensiveAnalysis) {
      try {
        const factoryConfig: AnalyzerFactoryConfig = {
          logger: this.logger.child ? this.logger.child('factory') : this.logger,
          qualityAnalysisOptions: this.config.qualityAnalysisOptions,
          enabledAnalyzers: this.config.analyzerTypes.length > 0 ? this.config.analyzerTypes : undefined
        };

        this.analyzerFactory = new AnalyzerFactory(factoryConfig);

        // Initialize analysis orchestrator
        const orchestratorConfig: OrchestratorConfig = {
          analyzerFactory: this.analyzerFactory,
          logger: this.logger.child ? this.logger.child('orchestrator') : this.logger,
          defaultTimeout: 30000,
          failFast: false
        };

        this.analysisOrchestrator = new AnalysisOrchestrator(orchestratorConfig);
      } catch (error) {
        this.logger.warn('Failed to initialize comprehensive analysis - continuing with basic accessibility testing only', error);
        // Cannot modify readonly config - comprehensive analysis will remain disabled for this session
      }
    }
  }

  /**
   * Initialize the accessibility checker
   */
  async initialize(): Promise<void> {
    this.logger.info('AccessibilityChecker initialized with browser pooling');
  }

  /**
   * Cleanup resources
   */
  async cleanup(): Promise<void> {
    if (this.analyzerFactory) {
      await this.analyzerFactory.cleanup();
    }
    this.logger.info('AccessibilityChecker cleaned up');
  }

  /**
   * Test a single page for accessibility
   */
  async testPage(url: string, options: PageTestOptions = {}): Promise<PageTestResult> {
    const startTime = Date.now();
    const logger = this.logger.child ? this.logger.child('test-page') : this.logger;

    logger.info(`Testing page: ${url}`);

    const { browser, context, release } = await this.config.poolManager.acquire();
    
    try {
      const page = await context.newPage();
      
      try {
        // Navigate to page
        await this.configurePage(page, options);

        // Observe navigation-level redirects
        let wasRedirectNav = false;
        const onResponse = (res: any) => {
          try {
            const req = res.request();
            const isNav = typeof (req as any).isNavigationRequest === 'function' ? req.isNavigationRequest() : false;
            if (isNav && res.status() >= 300 && res.status() < 400) {
              wasRedirectNav = true;
            }
          } catch { /* Ignore response check errors */ }
        };
        page.on('response', onResponse);

        const response = await page.goto(url, {
          waitUntil: 'domcontentloaded',
          timeout: options.timeout || 30000
        });

        // Stop observing responses
        page.off('response', onResponse);

        // Check for errors and redirects
        if (!response || response.status() >= 400) {
          throw new Error(`HTTP ${response?.status() || 'unknown'} error`);
        }

        // If this navigation was the result of an HTTP redirect and skipping is enabled, short-circuit
        const skipRedirects = options.skipRedirects !== false;
        try {
          const lastRequest = response.request();
          if (typeof (lastRequest as any).redirectedFrom === 'function') {
            wasRedirectNav = wasRedirectNav || !!lastRequest.redirectedFrom();
          }
        } catch { /* Ignore redirect check errors */ }
        
        if (skipRedirects && wasRedirectNav) {
          const duration = Date.now() - startTime;
          const titleNow = await page.title();
          const minimal: PageTestResult = {
            url,
            title: titleNow || 'Redirected',
            accessibilityResult: {
              url,
              title: titleNow || 'Redirected',
              imagesWithoutAlt: 0,
              buttonsWithoutLabel: 0,
              headingsCount: 0,
              errors: [`HTTP Redirect detected (skipped)`],
              warnings: [],
              passed: false,
              crashed: false,
              skipped: true,
              duration
            },
            comprehensiveAnalysis: undefined,
            duration,
            timestamp: new Date()
          };
          return minimal;
        }

        // Run basic accessibility analysis
        const accessibilityResult = await this.runBasicAccessibilityAnalysis(page, url, options);
        
        // Run comprehensive analysis if enabled
        let comprehensiveAnalysis: AnalysisResults | undefined;
        if ((options.enableComprehensiveAnalysis ?? this.config.enableComprehensiveAnalysis) && this.analysisOrchestrator) {
          comprehensiveAnalysis = await this.analysisOrchestrator.runComprehensiveAnalysis(
            page, 
            url, 
            { timeout: options.timeout }
          );
        }

        const duration = Date.now() - startTime;
        
        const result: PageTestResult = {
          url,
          title: accessibilityResult.title,
          accessibilityResult,
          comprehensiveAnalysis,
          duration,
          timestamp: new Date()
        };

        logger.info(`Page testing completed`, { 
          url, 
          duration, 
          passed: accessibilityResult.passed 
        });

        return result;

      } finally {
        await page.close();
      }
    } finally {
      await release();
    }
  }

  /**
   * Test multiple pages in parallel with redirect filtering
   */
  async testMultiplePages(
    urls: string[],
    options: PageTestOptions = {}
  ): Promise<MultiPageTestResult> {
    const startTime = Date.now();
    const logger = this.logger.child ? this.logger.child('multi-test') : this.logger;
    const skipRedirects = options.skipRedirects !== false;
    
    logger.info(`Testing ${urls.length} pages`, { 
      concurrency: options.maxConcurrent || 3,
      comprehensive: options.enableComprehensiveAnalysis ?? this.config.enableComprehensiveAnalysis,
      skipRedirects
    });

    const results: PageTestResult[] = [];
    const skippedUrls: string[] = [];
    let urlsToProcess = urls;

    // Pre-filter redirects if enabled
    if (skipRedirects) {
      logger.debug('Pre-filtering redirects...');
      const filteredUrls: string[] = [];
      
      for (const url of urls) {
        try {
          const minimalResult = await this.testUrlMinimal(url, 8000);
          if (minimalResult.skipped && minimalResult.errors.some(e => e.includes('Redirect'))) {
            skippedUrls.push(url);
            logger.debug(`Skipped redirect: ${url}`);
          } else {
            filteredUrls.push(url);
          }
        } catch (error) {
          // If minimal test fails, include URL for full testing
          filteredUrls.push(url);
        }
      }
      
      urlsToProcess = filteredUrls;
      logger.info(`After redirect filtering: ${urlsToProcess.length} URLs to test, ${skippedUrls.length} redirects skipped`);
    }

    // Configure queue callbacks
    const callbacks: QueueEventCallbacks<string> = {
      onProgressUpdate: (stats) => {
        if (stats.progress > 0 && stats.progress % 25 === 0) {
          logger.info(`Progress: ${stats.progress.toFixed(1)}%`, {
            completed: stats.completed,
            total: stats.total,
            workers: stats.activeWorkers
          });
        }
      },
      onItemCompleted: (item, result) => {
        logger.debug(`Completed: ${item.data}`, { duration: item.duration });
      },
      onItemFailed: (item, error) => {
        logger.warn(`Failed: ${item.data}`, { error, attempts: item.attempts });
      },
      onQueueEmpty: () => {
        logger.info('All page tests completed');
      }
    };

    // Create optimized queue for accessibility testing
    const queue = Queue.forAccessibilityTesting<string>('parallel', {
      maxConcurrent: options.maxConcurrent || 3,
      maxRetries: 3,
      retryDelay: 2000,
      timeout: options.timeout || 30000,
      enableProgressReporting: true,
      progressUpdateInterval: 3000
    }, callbacks);

    try {
      // Process filtered URLs with queue
      const result = await queue.processWithProgress(urlsToProcess, async (url: string) => {
        return await this.testPage(url, options);
      }, {
        showProgress: !options.verbose,
        progressInterval: 3000
      });

      // Extract successful results
      result.completed.forEach(item => {
        if (item.result) {
          results.push(item.result);
        }
      });

      // Add failed items as error results
      result.failed.forEach(failedItem => {
        results.push({
          url: failedItem.data,
          title: 'Error',
          accessibilityResult: {
            url: failedItem.data,
            title: 'Error',
            imagesWithoutAlt: 0,
            buttonsWithoutLabel: 0,
            headingsCount: 0,
            errors: [`Test failed: ${failedItem.error}`],
            warnings: [],
            passed: false,
            crashed: true,
            duration: failedItem.duration || 0
          },
          duration: failedItem.duration || 0,
          timestamp: new Date()
        });
      });

      const totalDuration = Date.now() - startTime;
      
      logger.info(`Multi-page testing completed`, {
        total: urls.length,
        tested: results.length,
        skipped: skippedUrls.length,
        failed: result.failed.length,
        totalDuration
      });

      return {
        results,
        skippedUrls,
        totalDuration,
        timestamp: new Date()
      };

    } catch (error) {
      logger.error('Multi-page testing failed', error);
      throw error;
    }
  }

  /**
   * Test URL for basic connectivity (used by SmartUrlSampler)
   */
  async testUrlMinimal(url: string, timeout: number = 5000): Promise<AccessibilityResult> {
    const startTime = Date.now();
    const logger = this.logger.child ? this.logger.child('minimal-test') : this.logger;

    logger.debug(`Minimal test: ${url}`);

    const { browser, context, release } = await this.config.poolManager.acquire();
    
    try {
      const page = await context.newPage();
      
      try {
        page.setDefaultTimeout(timeout);

        // Observe navigation-level redirects
        let wasRedirect = false;
        const onResponse = (res: any) => {
          try {
            const req = res.request();
            const isNav = typeof (req as any).isNavigationRequest === 'function' ? req.isNavigationRequest() : false;
            if (isNav && res.status() >= 300 && res.status() < 400) {
              wasRedirect = true;
            }
          } catch { /* Ignore redirect detection errors */ }
        };
        page.on('response', onResponse);
        
        const response = await page.goto(url, {
          waitUntil: 'domcontentloaded',
          timeout
        });

        // Stop observing responses
        page.off('response', onResponse);

        const finalUrl = page.url();
        const title = await page.title();
        const status = response?.status() || 0;
        
        // Detect redirects even when Playwright follows them to a final 200
        try {
          const lastRequest = response?.request();
          if (lastRequest) {
            // If the last request was created via a redirect chain, redirectedFrom() will be non-null
            if (typeof (lastRequest as any).redirectedFrom === 'function') {
              wasRedirect = wasRedirect || !!lastRequest.redirectedFrom();
            }
          }
        } catch { /* Ignore redirect chain check errors */ }
        
        // Only treat real HTTP redirects as redirects, not arbitrary URL changes
        const isHttpRedirect = wasRedirect || (status >= 300 && status < 400);
        const urlChanged = finalUrl !== url;
        
        const result: AccessibilityResult = {
          url,
          title: title || 'Untitled',
          imagesWithoutAlt: 0,
          buttonsWithoutLabel: 0,
          headingsCount: 0,
          errors: [],
          warnings: [],
          passed: status >= 200 && status < 300, // Only 2xx are successful
          crashed: false,
          skipped: isHttpRedirect || status === 404,
          duration: Date.now() - startTime
        };
        
        // Add status-specific errors
        if (status === 404) {
          result.errors.push('HTTP 404 Not Found');
        } else if (status >= 300 && status < 400) {
          result.errors.push(`HTTP ${status} Redirect`);
        } else if (status >= 400) {
          result.errors.push(`HTTP ${status} Error`);
        }
        
        // Add redirect info if needed
        if (isHttpRedirect) {
          (result as any).redirectInfo = {
            status: status >= 300 && status < 400 ? status : 0,
            originalUrl: url,
            finalUrl,
            type: 'http_redirect'
          };
        }
        
        return result;

      } finally {
        await page.close();
      }
    } catch (error) {
      logger.debug(`Minimal test failed: ${url}`, error);
      
      return {
        url,
        title: 'Error',
        imagesWithoutAlt: 0,
        buttonsWithoutLabel: 0,
        headingsCount: 0,
        errors: [`Navigation failed: ${error}`],
        warnings: [],
        passed: false,
        crashed: true,
        duration: Date.now() - startTime
      };
    } finally {
      await release();
    }
  }

  /**
   * Get available analysis types
   */
  getAvailableAnalysisTypes(): AnalyzerType[] {
    if (!this.analysisOrchestrator) {
      return [];
    }
    return this.analysisOrchestrator.getAvailableAnalyzers();
  }

  // Private helper methods

  private async configurePage(page: Page, options: PageTestOptions): Promise<void> {
    // Set viewport
    await page.setViewportSize({ width: 1920, height: 1080 });

    // Set user agent
    await page.setExtraHTTPHeaders({
      'User-Agent': 'auditmysite/2.0.0 (+https://github.com/casoon/AuditMySite)'
    });

    // Configure console/error logging based on verbose setting
    if (options.verbose) {
      page.on('console', msg => this.logger.debug(`Browser: ${msg.text()}`));
      page.on('pageerror', error => this.logger.warn(`JS Error: ${error.message}`));
    }
  }

  private async runBasicAccessibilityAnalysis(
    page: Page, 
    url: string, 
    options: PageTestOptions
  ): Promise<AccessibilityResult> {
    const startTime = Date.now();
    
    const result: AccessibilityResult = {
      url,
      title: await page.title(),
      imagesWithoutAlt: 0,
      buttonsWithoutLabel: 0,
      headingsCount: 0,
      errors: [],
      warnings: [],
      passed: true,
      duration: 0
    };

    // Basic element checks
    result.imagesWithoutAlt = await page.locator('img:not([alt])').count();
    if (result.imagesWithoutAlt > 0) {
      result.warnings.push(`${result.imagesWithoutAlt} images without alt attribute`);
    }

    result.buttonsWithoutLabel = await page
      .locator('button:not([aria-label])')
      .filter({ hasText: '' })
      .count();
    if (result.buttonsWithoutLabel > 0) {
      result.warnings.push(`${result.buttonsWithoutLabel} buttons without aria-label`);
    }

    result.headingsCount = await page.locator('h1, h2, h3, h4, h5, h6').count();
    if (result.headingsCount === 0) {
      result.errors.push('No headings found');
    }

    // Run pa11y tests
    await this.runPa11yTests(result, options, page);

    result.duration = Date.now() - startTime;
    result.passed = result.errors.length === 0;

    return result;
  }

  private async runPa11yTests(
    result: AccessibilityResult, 
    options: PageTestOptions,
    page?: Page
  ): Promise<void> {
    try {
      this.logger.debug('Running pa11y accessibility tests');

      const pa11yResult = await pa11y(result.url, {
        timeout: options.timeout || 15000,
        wait: options.wait || 1000,
        standard: options.pa11yStandard || 'WCAG2AA',
        hideElements: options.hideElements || 'iframe[src*="google-analytics"], iframe[src*="doubleclick"]',
        includeNotices: options.includeNotices !== false,
        includeWarnings: options.includeWarnings !== false,
        runners: ['axe'] // Always use axe for pooled browsers
      });

      // Process pa11y issues
      if (pa11yResult.issues) {
        pa11yResult.issues.forEach((issue: any) => {
          const detailedIssue: Pa11yIssue = {
            code: issue.code,
            message: issue.message,
            type: issue.type as 'error' | 'warning' | 'notice',
            selector: issue.selector,
            context: issue.context,
            impact: issue.impact,
            help: issue.help,
            helpUrl: issue.helpUrl
          };

          result.pa11yIssues = result.pa11yIssues || [];
          result.pa11yIssues.push(detailedIssue);

          // Add to appropriate array
          const message = `${issue.code}: ${issue.message}`;
          if (issue.type === 'error') {
            result.errors.push(message);
          } else if (issue.type === 'warning') {
            result.warnings.push(message);
          } else if (issue.type === 'notice') {
            result.warnings.push(`Notice: ${message}`);
          }
        });

        // Calculate pa11y score with balanced penalties
        const totalIssues = pa11yResult.issues.length;
        const errors = pa11yResult.issues.filter((i: any) => i.type === 'error').length;
        const warnings = pa11yResult.issues.filter((i: any) => i.type === 'warning').length;
        
        let score = 100;
        // More balanced scoring: ~5 errors = ~75 score, ~20 errors = ~50 score
        score -= errors * 2.5; // 2.5 points per error (was 10)
        score -= warnings * 1; // 1 point per warning (was 2)
        
        result.pa11yScore = Math.max(0, Math.min(100, Math.round(score)));
      } else {
        result.pa11yScore = 100;
      }

    } catch (error) {
      this.logger.warn('pa11y test failed, using fallback scoring', error);
      
      // Fallback score calculation
      let score = 100;
      score -= result.errors.length * 15;
      score -= result.warnings.length * 5;
      score -= result.imagesWithoutAlt * 3;
      score -= result.buttonsWithoutLabel * 5;
      if (result.headingsCount === 0) score -= 20;
      
      result.pa11yScore = Math.max(0, score);
    }
  }
}

/**
 * Error classes
 */
export class AccessibilityTestError extends Error {
  constructor(url: string, message: string) {
    super(`Accessibility test failed for ${url}: ${message}`);
    this.name = 'AccessibilityTestError';
  }
}