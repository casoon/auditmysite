/**
 * AccessibilityChecker v2 - Clean Architecture
 * 
 * This is a complete rewrite of the AccessibilityChecker with proper
 * separation of concerns, dependency injection, and typed interfaces.
 */

import { Browser, Page, Response } from 'playwright';
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
import { RedirectDetector, createRedirectDetector } from './redirect-detector';
import { ResultFactory } from './result-factory';
import {
  TIMEOUTS,
  CONCURRENCY,
  RETRY,
  SCORING,
  VIEWPORT,
  USER_AGENTS,
  PA11Y_HIDE_ELEMENTS,
  PROGRESS,
  HTTP_STATUS,
  isHttpSuccess,
  isHttpRedirect,
  isHttpError
} from './constants';

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
        // Configure page settings
        await this.configurePage(page, options);

        // Set up redirect detection
        const redirectDetector = createRedirectDetector({
          skipRedirects: options.skipRedirects,
          logger: logger.child ? logger.child('redirect') : logger
        });

        const { getResult, cleanup } = redirectDetector.attachToPage(page);

        try {
          // Navigate to page
          const response = await page.goto(url, {
            waitUntil: 'domcontentloaded',
            timeout: options.timeout || TIMEOUTS.DEFAULT_NAVIGATION
          });

          // Check for HTTP errors
          if (!response || isHttpError(response.status())) {
            throw new Error(`HTTP ${response?.status() || 'unknown'} error`);
          }

          // Check for redirects
          const redirectInfo = getResult(response, url);

          if (options.skipRedirects !== false && redirectInfo.isRedirect) {
            const duration = Date.now() - startTime;
            logger.info(`Skipping redirected URL`, redirectInfo);
            return ResultFactory.createRedirectResult(redirectInfo, duration);
          }

          // Run basic accessibility analysis
          const accessibilityResult = await this.runBasicAccessibilityAnalysis(page, url, options);

          // Run comprehensive analysis if enabled
          let comprehensiveAnalysis: AnalysisResults | undefined;
          if (this.shouldRunComprehensiveAnalysis(options)) {
            comprehensiveAnalysis = await this.runComprehensiveAnalysis(page, url, options);
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
          cleanup();
        }

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

    logger.info(`Testing ${urls.length} pages`, {
      concurrency: options.maxConcurrent || CONCURRENCY.DEFAULT_MAX,
      comprehensive: this.shouldRunComprehensiveAnalysis(options),
      skipRedirects: options.skipRedirects !== false
    });

    // Pre-filter redirects if enabled
    const { urlsToProcess, skippedUrls } = await this.preFilterRedirects(urls, options, logger);

    // Create and configure queue
    const queue = this.createQueue(options, logger);

    try {
      // Process URLs with queue
      const queueResult = await queue.processWithProgress(urlsToProcess, async (url: string) => {
        return await this.testPage(url, options);
      }, {
        showProgress: !options.verbose,
        progressInterval: PROGRESS.UPDATE_INTERVAL
      });

      // Collect results
      const results = this.collectResults(queueResult);

      const totalDuration = Date.now() - startTime;

      logger.info(`Multi-page testing completed`, {
        total: urls.length,
        tested: results.length,
        skipped: skippedUrls.length,
        failed: queueResult.failed.length,
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
   * Pre-filter redirects from URL list
   */
  private async preFilterRedirects(
    urls: string[],
    options: PageTestOptions,
    logger: ILogger
  ): Promise<{ urlsToProcess: string[]; skippedUrls: string[] }> {
    const skipRedirects = options.skipRedirects !== false;

    if (!skipRedirects) {
      return { urlsToProcess: urls, skippedUrls: [] };
    }

    logger.debug('Pre-filtering redirects...');
    const filteredUrls: string[] = [];
    const skippedUrls: string[] = [];

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

    logger.info(`After redirect filtering: ${filteredUrls.length} URLs to test, ${skippedUrls.length} redirects skipped`);

    return { urlsToProcess: filteredUrls, skippedUrls };
  }

  /**
   * Create queue with callbacks
   */
  private createQueue(options: PageTestOptions, logger: ILogger): Queue<string, PageTestResult> {
    const callbacks: QueueEventCallbacks<string> = {
      onProgressUpdate: (stats) => {
        if (stats.progress > 0 && stats.progress % PROGRESS.REPORT_THRESHOLD === 0) {
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

    return Queue.forAccessibilityTesting<string>('parallel', {
      maxConcurrent: options.maxConcurrent || CONCURRENCY.DEFAULT_MAX,
      maxRetries: RETRY.MAX_ATTEMPTS,
      retryDelay: RETRY.DELAY_MS,
      timeout: options.timeout || TIMEOUTS.QUEUE_ITEM,
      enableProgressReporting: true,
      progressUpdateInterval: PROGRESS.UPDATE_INTERVAL
    }, callbacks);
  }

  /**
   * Collect results from queue execution
   */
  private collectResults(queueResult: any): PageTestResult[] {
    const results: PageTestResult[] = [];

    // Add successful results
    queueResult.completed.forEach((item: any) => {
      if (item.result) {
        results.push(item.result);
      }
    });

    // Add failed items as error results
    queueResult.failed.forEach((failedItem: any) => {
      const duration = failedItem.duration || 0;
      results.push(ResultFactory.createErrorResult(failedItem.data, failedItem.error, duration));
    });

    return results;
  }

  /**
   * Test URL for basic connectivity (used by SmartUrlSampler)
   */
  async testUrlMinimal(url: string, timeout: number = TIMEOUTS.MINIMAL_TEST): Promise<AccessibilityResult> {
    const startTime = Date.now();
    const logger = this.logger.child ? this.logger.child('minimal-test') : this.logger;

    logger.debug(`Minimal test: ${url}`);

    const { browser, context, release } = await this.config.poolManager.acquire();

    try {
      const page = await context.newPage();

      try {
        page.setDefaultTimeout(timeout);

        // Set up redirect detection
        const redirectDetector = createRedirectDetector({
          logger: logger.child ? logger.child('redirect') : logger
        });

        const { getResult, cleanup } = redirectDetector.attachToPage(page);

        try {
          const response = await page.goto(url, {
            waitUntil: 'domcontentloaded',
            timeout
          });

          const title = await page.title();
          const status = response?.status() || 0;
          const duration = Date.now() - startTime;

          // Get redirect information
          const redirectInfo = getResult(response, url);

          // Create base result
          let result = ResultFactory.createMinimalResult({
            url,
            title,
            duration
          });

          // Update based on HTTP status
          if (status === HTTP_STATUS.NOT_FOUND) {
            result = ResultFactory.create404Result(url, duration);
          } else if (isHttpError(status)) {
            result = ResultFactory.createHttpErrorResult(url, status, duration);
          } else if (redirectInfo.isRedirect) {
            result = ResultFactory.createHttpErrorResult(url, redirectInfo.statusCode, duration);
            result = ResultFactory.addRedirectInfo(result, redirectInfo);
          } else if (isHttpSuccess(status)) {
            result.passed = true;
            result.title = title;
          }

          return result;

        } finally {
          cleanup();
        }

      } finally {
        await page.close();
      }
    } catch (error) {
      logger.debug(`Minimal test failed: ${url}`, error);

      const duration = Date.now() - startTime;
      const result = ResultFactory.createMinimalResult({ url, duration });
      result.errors.push(`Navigation failed: ${error instanceof Error ? error.message : String(error)}`);
      result.passed = false;
      result.crashed = true;

      return result;
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

  /**
   * Check if comprehensive analysis should be run
   */
  private shouldRunComprehensiveAnalysis(options: PageTestOptions): boolean {
    return (
      (options.enableComprehensiveAnalysis ?? this.config.enableComprehensiveAnalysis) &&
      this.analysisOrchestrator !== undefined
    );
  }

  /**
   * Run comprehensive analysis on a page
   */
  private async runComprehensiveAnalysis(
    page: Page,
    url: string,
    options: PageTestOptions
  ): Promise<AnalysisResults | undefined> {
    if (!this.analysisOrchestrator) {
      return undefined;
    }

    try {
      return await this.analysisOrchestrator.runComprehensiveAnalysis(
        page,
        url,
        { timeout: options.timeout || TIMEOUTS.DEFAULT_NAVIGATION }
      );
    } catch (error) {
      this.logger.warn('Comprehensive analysis failed', { url, error });
      return undefined;
    }
  }

  /**
   * Configure page settings before navigation
   */
  private async configurePage(page: Page, options: PageTestOptions): Promise<void> {
    // Set viewport
    await page.setViewportSize(VIEWPORT.DESKTOP);

    // Set user agent
    await page.setExtraHTTPHeaders({
      'User-Agent': USER_AGENTS.DEFAULT
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

  /**
   * Run pa11y accessibility tests
   */
  private async runPa11yTests(
    result: AccessibilityResult,
    options: PageTestOptions,
    page?: Page
  ): Promise<void> {
    try {
      this.logger.debug('Running pa11y accessibility tests');

      const pa11yResult = await pa11y(result.url, {
        timeout: options.timeout || TIMEOUTS.PA11Y_TEST,
        wait: options.wait || TIMEOUTS.PA11Y_WAIT,
        standard: options.pa11yStandard || 'WCAG2AA',
        hideElements: options.hideElements || PA11Y_HIDE_ELEMENTS,
        includeNotices: options.includeNotices !== false,
        includeWarnings: options.includeWarnings !== false,
        runners: ['axe'] // Always use axe for pooled browsers
      });

      // Process pa11y issues
      if (pa11yResult.issues) {
        this.processPa11yIssues(pa11yResult.issues, result);
        result.pa11yScore = this.calculatePa11yScore(pa11yResult.issues);
      } else {
        result.pa11yScore = 100;
      }

    } catch (error) {
      this.logger.warn('pa11y test failed, using fallback scoring', error);
      result.pa11yScore = this.calculateFallbackScore(result);
    }
  }

  /**
   * Process pa11y issues and add them to result
   */
  private processPa11yIssues(issues: any[], result: AccessibilityResult): void {
    issues.forEach((issue: any) => {
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
  }

  /**
   * Calculate pa11y score based on issues
   */
  private calculatePa11yScore(issues: any[]): number {
    const errors = issues.filter((i: any) => i.type === 'error').length;
    const warnings = issues.filter((i: any) => i.type === 'warning').length;

    let score = 100;
    score -= errors * SCORING.ERROR_PENALTY;
    score -= warnings * SCORING.WARNING_PENALTY;

    return Math.max(0, Math.min(100, Math.round(score)));
  }

  /**
   * Calculate fallback score when pa11y fails
   */
  private calculateFallbackScore(result: AccessibilityResult): number {
    let score = 100;
    score -= result.errors.length * SCORING.FALLBACK.ERROR_PENALTY;
    score -= result.warnings.length * SCORING.FALLBACK.WARNING_PENALTY;
    score -= result.imagesWithoutAlt * SCORING.FALLBACK.IMAGE_NO_ALT;
    score -= result.buttonsWithoutLabel * SCORING.FALLBACK.BUTTON_NO_LABEL;

    if (result.headingsCount === 0) {
      score -= SCORING.FALLBACK.NO_HEADINGS;
    }

    return Math.max(0, score);
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