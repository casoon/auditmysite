import { chromium, Browser, Page } from "playwright";
import pa11y from "pa11y";
import { AccessibilityResult, TestOptions, Pa11yIssue } from '../types';
import { log } from '@core/logging';
import { BrowserManager } from '../browser';
import { BrowserPoolManager } from '../browser/browser-pool-manager';
import { WebVitalsCollector } from '../performance';
import { ParallelTestManager, ParallelTestManagerOptions, ParallelTestResult } from './index';
import { Queue, QueueConfig, QueueEventCallbacks } from '../queue';
import { ContentWeightAnalyzer } from '../../analyzers/content-weight-analyzer';
import { PerformanceCollector } from '../../analyzers/performance-collector';
import { SEOAnalyzer } from '../../analyzers/seo-analyzer';
import { MobileFriendlinessAnalyzer } from '../../analyzers/mobile-friendliness-analyzer';
import { QualityAnalysisOptions } from '../../types/enhanced-metrics';
import * as fs from 'fs';
import * as path from 'path';

export interface AccessibilityCheckerOptions {
  /** Use browser pooling for better performance with multiple pages */
  usePooling?: boolean;
  /** Pool manager to use (only if usePooling is true) */
  poolManager?: BrowserPoolManager;
  /** Browser manager options */
  browserOptions?: {
    headless?: boolean;
    port?: number;
  };
  /** Enable comprehensive analysis (performance, SEO, content weight, mobile) */
  enableComprehensiveAnalysis?: boolean;
  /** Quality analysis options for comprehensive analysis */
  qualityAnalysisOptions?: QualityAnalysisOptions;
}

export class AccessibilityChecker {
  public browserManager: BrowserManager | null = null;
  private poolManager: BrowserPoolManager | null = null;
  private webVitalsCollector: WebVitalsCollector;
  private parallelTestManager: ParallelTestManager | null = null;
  private usePooling: boolean;
  
  // Comprehensive analysis components
  private contentWeightAnalyzer: ContentWeightAnalyzer | null = null;
  private performanceCollector: PerformanceCollector | null = null;
  private seoAnalyzer: SEOAnalyzer | null = null;
  private mobileFriendlinessAnalyzer: MobileFriendlinessAnalyzer | null = null;
  private enableComprehensiveAnalysis: boolean;

  constructor(options: AccessibilityCheckerOptions = {}) {
    this.webVitalsCollector = new WebVitalsCollector(undefined, { verbose: false }); // Default to quiet
    this.usePooling = options.usePooling || false;
    this.poolManager = options.poolManager || null;
    this.enableComprehensiveAnalysis = options.enableComprehensiveAnalysis || false;
    
    // Initialize comprehensive analyzers if enabled
    if (this.enableComprehensiveAnalysis) {
      console.log('🔧 Initializing comprehensive analysis with all analyzers');
      this.contentWeightAnalyzer = new ContentWeightAnalyzer();
      this.performanceCollector = new PerformanceCollector(options.qualityAnalysisOptions);
      this.seoAnalyzer = new SEOAnalyzer(options.qualityAnalysisOptions);
      this.mobileFriendlinessAnalyzer = new MobileFriendlinessAnalyzer({ verbose: false }); // Default to quiet
    }
  }

  async initialize(options: AccessibilityCheckerOptions = {}): Promise<void> {
    // Merge with constructor options
    const finalOptions = { ...options, usePooling: options.usePooling ?? this.usePooling };
    
    if (finalOptions.usePooling && (finalOptions.poolManager || this.poolManager)) {
      // Use existing or provided pool manager
      this.poolManager = finalOptions.poolManager || this.poolManager;
      this.usePooling = true;
      console.log('🏊 AccessibilityChecker initialized with browser pooling');
    } else {
      // Use standard browser manager
      this.browserManager = new BrowserManager({
        headless: finalOptions.browserOptions?.headless ?? true,
        port: finalOptions.browserOptions?.port ?? 9222,
        verbose: false // Default to quiet for cleaner CLI output
      });
      
      await this.browserManager.initialize();
      console.log('🔧 AccessibilityChecker initialized with standard browser manager');
    }
  }

  async cleanup(): Promise<void> {
    if (this.browserManager) {
      await this.browserManager.cleanup();
    }
  }

  /**
   * Test a single page with comprehensive analysis
   * This method is used by ParallelTestManager and other components
   */
  async testPage(
    url: string,
    options: TestOptions = {},
  ): Promise<AccessibilityResult> {
    if (options.verbose) console.log('🔍 Testing:', url);
    
    if (this.usePooling && this.poolManager) {
      return this.testPageWithPool(url, options);
    }
    
    if (!this.browserManager) {
      throw new Error("AccessibilityChecker not initialized - call initialize() first");
    }

    const startTime = Date.now();
    const page = await this.browserManager.getPage();
    const result: AccessibilityResult = {
      url,
      title: "",
      imagesWithoutAlt: 0,
      buttonsWithoutLabel: 0,
      headingsCount: 0,
      errors: [],
      warnings: [],
      passed: true,
      duration: 0,
    };

    try {
      if (options.verbose) console.log(`   🌐 Navigating to page...`);
      await page.goto(url, {
        waitUntil: options.waitUntil || "domcontentloaded",
        timeout: options.timeout || 10000,
      });

      // Use shared test logic which includes comprehensive analysis
      await this.runPageTests(page, result, options);

      // Check for critical errors
      if (result.errors.length > 0) {
        result.passed = false;
      }
    } catch (error) {
      result.errors.push(`Navigation error: ${error}`);
      result.passed = false;
    } finally {
      await page.close();
      result.duration = Date.now() - startTime;
    }

    return result;
  }


  /**
   * Normalize URL from various formats (fixes sitemap parser URL object issue)
   */
  private normalizeUrl(url: string | { loc: string } | any): string {
    if (typeof url === 'string') {
      return url;
    }
    
    if (typeof url === 'object' && url !== null) {
      // Handle sitemap parser objects
      if ('loc' in url && typeof url.loc === 'string') {
        return url.loc;
      }
      
      // Handle other URL-like objects
      if ('url' in url && typeof url.url === 'string') {
        return url.url;
      }
      
      // Handle href property
      if ('href' in url && typeof url.href === 'string') {
        return url.href;
      }
      
      // Try to convert to string
      const urlStr = String(url);
      if (urlStr && urlStr !== '[object Object]') {
        return urlStr;
      }
    }
    
    throw new Error(`Cannot normalize URL: ${JSON.stringify(url)}`);
  }

  /**
   * 🚀 Parallel accessibility tests with event-driven queue
   * 
   * This method uses the event-driven queue system for parallel tests
   * with real-time status reporting and resource monitoring.
   */
  async testMultiplePagesParallel(
    urls: (string | { loc: string } | any)[],
    options: TestOptions = {},
  ): Promise<AccessibilityResult[]> {
    // Normalize all URLs to strings first
    const normalizedUrls = urls.map(url => {
      try {
        return this.normalizeUrl(url);
      } catch (error) {
        console.error(`❌ Failed to normalize URL ${JSON.stringify(url)}: ${error}`);
        return null;
      }
    }).filter((url): url is string => url !== null);
    
    const maxPages = options.maxPages || normalizedUrls.length;
    const pagesToTest = normalizedUrls.slice(0, maxPages);
    
    // Parallel test options with event callback support
    const parallelOptions: ParallelTestManagerOptions = {
      maxConcurrent: options.maxConcurrent || 3,
      maxRetries: options.maxRetries || 3,
      retryDelay: options.retryDelay || 2000,
      verbose: options.verbose || false, // Pass through verbose flag from test options
      enableResourceMonitoring: options.enableResourceMonitoring !== false,
      maxMemoryUsage: options.maxMemoryUsage || 512,
      maxCpuUsage: options.maxCpuUsage || 80,
      testOptions: options,
      // 🎯 Pass this AccessibilityChecker instance (with comprehensive analysis)
      accessibilityChecker: this,
      // Map event callbacks to ParallelTestManager's expected interface
      onTestStart: (url: string) => {
        // 🎯 Use custom callback from options or default
        if (options.eventCallbacks?.onUrlStarted) {
          options.eventCallbacks.onUrlStarted(url);
        } else if (options.verbose) {
          console.log(`🚀 Starting parallel test: ${url}`);
        }
      },
      onTestComplete: (url: string, result: AccessibilityResult) => {
        // Calculate duration from result object
        const duration = result.duration || 0;
        
        // 🎯 Use custom callback from options or default
        if (options.eventCallbacks?.onUrlCompleted) {
          options.eventCallbacks.onUrlCompleted(url, result, duration);
        } else if (options.verbose) {
          const status = result.passed ? '✅ PASSED' : '❌ FAILED';
          console.log(`${status} ${url} (${duration}ms) - ${result.errors.length} errors, ${result.warnings.length} warnings`);
        }
      },
      onTestError: (url: string, error: string) => {
        // 🎯 Use custom callback from options or default
        if (options.eventCallbacks?.onUrlFailed) {
          // ParallelTestManager doesn't provide attempts directly, so we use 1 as default
          options.eventCallbacks.onUrlFailed(url, error, 1);
        } else if (options.verbose) {
          console.error(`💥 Error testing ${url}: ${error}`);
        }
      },
      onProgressUpdate: (stats) => {
        // 🎯 Use custom callback from options or default
        if (options.eventCallbacks?.onProgressUpdate) {
          options.eventCallbacks.onProgressUpdate(stats);
        } else if (options.verbose) {
          console.log(`📆 Progress: ${stats.progress.toFixed(1)}% | Workers: ${stats.activeWorkers}/${options.maxConcurrent || 3} | Memory: ${stats.memoryUsage}MB`);
        }
      },
      onQueueEmpty: () => {
        // 🎯 Use custom callback from options or default
        if (options.eventCallbacks?.onQueueEmpty) {
          options.eventCallbacks.onQueueEmpty();
        } else if (options.verbose) {
          console.log('🎉 All parallel tests completed!');
        }
      }
    };

    // Initialize Parallel Test Manager
    this.parallelTestManager = new ParallelTestManager(parallelOptions);
    
    try {
      if (options.verbose) {
        console.log(`🚀 Starting parallel accessibility tests for ${pagesToTest.length} pages with ${parallelOptions.maxConcurrent} workers`);
        console.log(`⚙️  Configuration: maxRetries=${parallelOptions.maxRetries}, retryDelay=${parallelOptions.retryDelay}ms`);
      }
      
      // Initialize manager
      await this.parallelTestManager.initialize();
      
      // Run tests
      const startTime = Date.now();
      const result: ParallelTestResult = await this.parallelTestManager.runTests(pagesToTest);
      const totalDuration = Date.now() - startTime;
      
      // Output results (only in verbose mode)
      if (options.verbose) {
        console.log('\n📋 Parallel Test Results Summary:');
        console.log('==================================');
        console.log(`⏱️  Total Duration: ${totalDuration}ms`);
        console.log(`📄 URLs Tested: ${result.results.length}`);
        console.log(`✅ Successful: ${result.results.filter(r => r.passed).length}`);
        console.log(`❌ Failed: ${result.results.filter(r => !r.passed).length}`);
        console.log(`💥 Errors: ${result.errors.length}`);
        
        // Performance metrics
        const avgTimePerUrl = totalDuration / pagesToTest.length;
        const speedup = avgTimePerUrl > 0 ? (avgTimePerUrl * pagesToTest.length) / totalDuration : 0;
        
        console.log('\n🚀 Performance Metrics:');
        console.log('======================');
        console.log(`Average time per URL: ${avgTimePerUrl.toFixed(0)}ms`);
        console.log(`Speedup factor: ${speedup.toFixed(1)}x`);
        console.log(`Throughput: ${(pagesToTest.length / (totalDuration / 1000)).toFixed(1)} URLs/second`);
        
        // Detailed statistics
        console.log('\n📊 Queue Statistics:');
        console.log('===================');
        console.log(`Total: ${result.stats.total}`);
        console.log(`Completed: ${result.stats.completed}`);
        console.log(`Failed: ${result.stats.failed}`);
        console.log(`Retrying: ${result.stats.retrying}`);
        console.log(`Progress: ${result.stats.progress.toFixed(1)}%`);
        console.log(`Average Duration: ${result.stats.averageDuration}ms`);
        console.log(`Memory Usage: ${result.stats.memoryUsage}MB`);
        console.log(`CPU Usage: ${result.stats.cpuUsage}s`);
        
        // Error details
        if (result.errors.length > 0) {
          console.log('\n❌ Failed URLs:');
          console.log('===============');
          result.errors.forEach((error, index) => {
            console.log(`${index + 1}. ${error.url} (${error.attempts} attempts): ${error.error}`);
          });
        }
      }
      
      return result.results;
      
    } catch (error) {
      console.error('❌ Parallel test execution failed:', error);
      throw error;
    } finally {
      // Cleanup
      if (this.parallelTestManager) {
        await this.parallelTestManager.cleanup();
        this.parallelTestManager = null;
      }
    }
  }

  // Legacy EventDrivenQueue method removed - using modern Queue system only

  /**
   * 🔧 Test multiple pages with the modern Queue System
   * This is the recommended approach for concurrent testing
   */
  async testMultiplePagesWithQueue(
    urls: string[],
    options: TestOptions = {},
  ): Promise<AccessibilityResult[]> {
    console.log(`🔧 Starting Queue processing for ${urls.length} URLs`);
    
    // Initialize browser
    if (!this.browserManager) {
      this.browserManager = new BrowserManager({
        headless: true,
        port: 9222
      });
      await this.browserManager.initialize();
    }

    // Configure queue callbacks for progress reporting
    const callbacks: QueueEventCallbacks<string> = {
      onProgressUpdate: (stats) => {
        if (stats.progress > 0 && stats.progress % 20 === 0) {
          process.stdout.write(`\r🚀 Testing: ${stats.progress.toFixed(1)}% (${stats.completed}/${stats.total}) | Workers: ${stats.activeWorkers}`);
        }
      },
      onItemCompleted: (item, result) => {
        const shortUrl = item.data.split('/').pop() || item.data;
        if (options.verbose) {
          console.log(`\n✅ ${shortUrl} (${item.duration}ms)`);
        }
      },
      onItemFailed: (item, error) => {
        const shortUrl = item.data.split('/').pop() || item.data;
        console.log(`\n❌ ${shortUrl}: ${error}`);
      },
      onQueueEmpty: () => {
        console.log('\n🎉 All tests completed!');
      }
    };

    // Create queue optimized for accessibility testing
    const queue = Queue.forAccessibilityTesting<string>('parallel', {
      maxConcurrent: options.maxConcurrent || 2,
      maxRetries: options.maxRetries || 3,
      retryDelay: options.retryDelay || 2000,
      timeout: options.timeout || 30000,
      enableProgressReporting: true,
      progressUpdateInterval: 2000
    }, callbacks);

    try {
      // Process all URLs with queue
      const result = await queue.processWithProgress(urls, async (url: string) => {
        return await this.testPage(url, options);
      }, {
        showProgress: !options.verbose, // Show progress bar unless verbose
        progressInterval: 3000
      });
      
      // Extract results from queue items
      const results: AccessibilityResult[] = result.completed.map(item => item.result);
      
      // Add failed items as crashed results
      result.failed.forEach(failedItem => {
        results.push({
          url: failedItem.data,
          title: '',
          imagesWithoutAlt: 0,
          buttonsWithoutLabel: 0,
          headingsCount: 0,
          errors: [`Test failed: ${failedItem.error}`],
          warnings: [],
          passed: false,
          crashed: true,
          duration: failedItem.duration || 0
        });
      });

      console.log(`📊 Queue Results: ${result.completed.length} completed, ${result.failed.length} failed`);
      const metrics = queue.getPerformanceMetrics();
      console.log(`📈 Performance: ${metrics.efficiency.toFixed(1)}% efficiency, ${metrics.throughput.toFixed(2)} pages/sec`);
      
      return results;
    } finally {
      // Queue cleanup is automatic
    }
  }

  // 🆕 Extended page configuration
  private async configurePage(page: Page, options: TestOptions): Promise<void> {
    // Viewport configuration
    const viewportSize = options.viewportSize || { width: 1920, height: 1080 };
    await page.setViewportSize(viewportSize);

    // Set user agent (default: auditmysite)
    const userAgent = options.userAgent || 'auditmysite/1.0 (+https://github.com/casoon/AuditMySite)';
    await page.setExtraHTTPHeaders({
      'User-Agent': userAgent
    });

    // Network interception for performance
    if (options.blockImages) {
      await page.route('**/*.{png,jpg,jpeg,gif,svg,webp}', route => {
        route.abort();
      });
    }

    if (options.blockCSS) {
      await page.route('**/*.css', route => {
        route.abort();
      });
    }

    // Console logging
    page.on('console', msg => {
      if (options.verbose) {
        console.log(`Browser Console: ${msg.text()}`);
      }
    });

    // Error handling
    page.on('pageerror', error => {
      if (options.verbose) {
        console.log(`JavaScript Error: ${error.message}`);
      }
    });
  }

  // 🆕 Collect performance metrics using Google's Web Vitals
  private async collectPerformanceMetrics(page: Page, result: AccessibilityResult, options: TestOptions): Promise<void> {
    try {
      if (options.verbose) console.log(`   📊 Collecting Core Web Vitals...`);
      
      // Use the official WebVitalsCollector for accurate metrics
      const webVitals = await this.webVitalsCollector.collectMetrics(page);
      
      // Store Web Vitals in result
      result.performanceMetrics = {
        // Navigation timing
        loadTime: webVitals.loadTime,
        domContentLoaded: webVitals.domContentLoaded,
        firstPaint: 0, // Not available in Web Vitals, set to 0
        renderTime: webVitals.renderTime,
        
        // Core Web Vitals
        firstContentfulPaint: webVitals.fcp,
        largestContentfulPaint: webVitals.lcp,
        cumulativeLayoutShift: webVitals.cls,
        timeToFirstByte: webVitals.ttfb,
        
        // Quality score
        performanceScore: webVitals.score,
        performanceGrade: webVitals.grade
      };
      
      // Add performance-based warnings using Web Vitals thresholds
      webVitals.recommendations.forEach(rec => {
        if (rec !== 'Excellent performance! All Core Web Vitals are within good thresholds.') {
          result.warnings.push(rec);
        }
      });
      
      if (options.verbose) {
        console.log(`   🏆 Performance Score: ${webVitals.score} (${webVitals.grade})`);
        console.log(`   📈 LCP: ${webVitals.lcp}ms, FCP: ${webVitals.fcp}ms, CLS: ${webVitals.cls}`);
      }
      
    } catch (error) {
      if (options.verbose) {
        console.log(`Web Vitals collection failed: ${error}`);
      }
      // Fallback to simple metrics if Web Vitals fail
      await this.collectFallbackMetrics(page, result, options);
    }
  }
  
  private async collectFallbackMetrics(page: Page, result: AccessibilityResult, options: TestOptions): Promise<void> {
    try {
      const metrics = await page.evaluate(() => {
        const navigation = performance.getEntriesByType('navigation')[0] as PerformanceNavigationTiming;
        return {
          loadTime: navigation.loadEventEnd - navigation.loadEventStart,
          domContentLoaded: navigation.domContentLoadedEventEnd - navigation.domContentLoadedEventStart,
          firstPaint: performance.getEntriesByName('first-paint')[0]?.startTime || 0,
          renderTime: navigation.loadEventEnd - navigation.fetchStart || 0,
          firstContentfulPaint: performance.getEntriesByName('first-contentful-paint')[0]?.startTime || 0,
          largestContentfulPaint: performance.getEntriesByName('largest-contentful-paint')[0]?.startTime || 0,
          cumulativeLayoutShift: 0,  // Not available in fallback
          interactionToNextPaint: 0, // Not available in fallback
          timeToFirstByte: navigation.responseStart - navigation.fetchStart || 0,
          performanceScore: 50,      // Default fallback score
          performanceGrade: 'C' as 'C' // Default fallback grade
        };
      });

      result.performanceMetrics = metrics;

      if (metrics.loadTime > 3000) {
        result.warnings.push(`Slow page load: ${Math.round(metrics.loadTime)}ms`);
      }
    } catch (error) {
      if (options.verbose) {
        console.log(`Fallback metrics collection failed: ${error}`);
      }
    }
  }

  // 🆕 Keyboard Navigation Test
  private async testKeyboardNavigation(page: Page, result: AccessibilityResult, options: TestOptions): Promise<void> {
    try {
      const keyboardNavigation = await page.evaluate(() => {
        const focusableElements = document.querySelectorAll('button, input, select, textarea, a[href], [tabindex]:not([tabindex="-1"])');
        const navigation: string[] = [];
        
        // Simulate tab navigation for the first 10 elements
        for (let i = 0; i < Math.min(focusableElements.length, 10); i++) {
          const element = focusableElements[i] as HTMLElement;
          navigation.push(`${element.tagName.toLowerCase()}: ${element.textContent?.trim().substring(0, 50) || element.outerHTML}`);
        }
        
        return navigation;
      });

      result.keyboardNavigation = keyboardNavigation;
    } catch (error) {
      if (options.verbose) {
        console.log(`Keyboard navigation test failed: ${error}`);
      }
    }
  }

  // 🆕 Color Contrast Test (simplified)
  private async testColorContrast(page: Page, result: AccessibilityResult, options: TestOptions): Promise<void> {
    try {
      const contrastIssues = await page.evaluate(() => {
        const elements = document.querySelectorAll('p, span, div, h1, h2, h3, h4, h5, h6, a, button, input, label');
        const issues: string[] = [];
        
        elements.forEach(el => {
          const style = window.getComputedStyle(el);
          const color = style.color;
          const backgroundColor = style.backgroundColor;
          
          // Simple contrast check (simplified)
          if (color && backgroundColor && 
              color !== backgroundColor && 
              color !== 'rgba(0, 0, 0, 0)' && 
              backgroundColor !== 'rgba(0, 0, 0, 0)') {
            issues.push(`${el.tagName}: ${color} on ${backgroundColor}`);
          }
        });
        
        return issues.slice(0, 10); // Limitiere auf 10 Issues
      });

      if (contrastIssues.length > 0) {
        result.colorContrastIssues = contrastIssues;
        result.warnings.push(`${contrastIssues.length} potential color contrast issues found`);
      }
    } catch (error) {
      if (options.verbose) {
        console.log(`Color contrast test failed: ${error}`);
      }
    }
  }

  // 🆕 Focus Management Test
  private async testFocusManagement(page: Page, result: AccessibilityResult, options: TestOptions): Promise<void> {
    try {
      const focusIssues = await page.evaluate(() => {
        const issues: string[] = [];
        
        // Check for focus-visible
        const focusableElements = document.querySelectorAll('button, input, select, textarea, a[href]');
        focusableElements.forEach(el => {
          const style = window.getComputedStyle(el);
          if (style.outline === 'none' && 
              style.border === 'none' && 
              !el.classList.contains('focus-visible') &&
              !el.classList.contains('focus')) {
            issues.push(`Element without focus indicator: ${el.tagName} - ${el.textContent?.trim().substring(0, 30) || 'no text'}`);
          }
        });
        
        return issues.slice(0, 10); // Limitiere auf 10 Issues
      });

      if (focusIssues.length > 0) {
        result.focusManagementIssues = focusIssues;
        result.warnings.push(`${focusIssues.length} focus management issues found`);
      }
    } catch (error) {
      if (options.verbose) {
        console.log(`Focus management test failed: ${error}`);
      }
    }
  }

  // 🆕 Screenshot functionality
  private async captureScreenshots(page: Page, url: string, result: AccessibilityResult, options: TestOptions): Promise<void> {
    try {
      // Screenshots-Ordner erstellen
      const screenshotsDir = './screenshots';
      if (!fs.existsSync(screenshotsDir)) {
        fs.mkdirSync(screenshotsDir, { recursive: true });
      }

      const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
      const domain = new URL(url).hostname.replace(/\./g, '-');
      
      // Desktop Screenshot
      const desktopPath = path.join(screenshotsDir, `${domain}-desktop-${timestamp}.png`);
      await page.screenshot({
        path: desktopPath,
        fullPage: true
      });
      result.screenshots = { desktop: desktopPath };

      // Mobile Screenshot
      await page.setViewportSize({ width: 375, height: 667 });
      const mobilePath = path.join(screenshotsDir, `${domain}-mobile-${timestamp}.png`);
      await page.screenshot({
        path: mobilePath,
        fullPage: true
      });
      result.screenshots.mobile = mobilePath;
      
      // Reset viewport
      await page.setViewportSize({ width: 1920, height: 1080 });
    } catch (error) {
      if (options.verbose) {
        console.log(`Screenshot capture failed: ${error}`);
      }
    }
  }

  /**
   * Test page using browser pool for better performance
   */
  private async testPageWithPool(url: string, options: TestOptions = {}): Promise<AccessibilityResult> {
    const startTime = Date.now();
    
    const result: AccessibilityResult = {
      url,
      title: "",
      imagesWithoutAlt: 0,
      buttonsWithoutLabel: 0,
      headingsCount: 0,
      errors: [],
      warnings: [],
      passed: true,
      duration: 0,
    };

    // Acquire browser from pool
    const { browser, context, release } = await this.poolManager!.acquire();

    try {
      const page = await context.newPage();

      try {
        // Configure page with minimal setup
        await page.setDefaultTimeout(options.timeout || 10000);

        if (options.verbose) console.log(`   🌐 Navigating...`);
        await page.goto(url, {
          waitUntil: options.waitUntil || "domcontentloaded",
          timeout: options.timeout || 10000,
        });

        // Use same test logic as standard testPage
        await this.runPageTests(page, result, options);

      } finally {
        await page.close();
      }
    } catch (error) {
      result.errors.push(`Navigation error: ${error}`);
      result.passed = false;
    } finally {
      // Always release browser back to pool
      await release();
      result.duration = Date.now() - startTime;
    }

    return result;
  }

  /**
   * Shared test logic for both standard and pooled testing
   */
  private async runPageTests(page: Page, result: AccessibilityResult, options: TestOptions): Promise<void> {
    // Extract the core testing logic from testPage
    if (options.verbose) console.log(`   🔧 Configuring page...`);
    await this.configurePage(page, options);

    // Collect performance metrics
    if (options.collectPerformanceMetrics) {
      if (options.verbose) console.log(`   📊 Collecting performance metrics...`);
      await this.collectPerformanceMetrics(page, result, options);
    }

    // Check page title
    if (options.verbose) console.log(`   📋 Extracting page title...`);
    result.title = await page.title();

    // Basic accessibility checks
    if (options.verbose) console.log(`   🖼️  Checking images for alt attributes...`);
    result.imagesWithoutAlt = await page.locator("img:not([alt])").count();
    if (result.imagesWithoutAlt > 0) {
      result.warnings.push(`${result.imagesWithoutAlt} images without alt attribute`);
    }

    if (options.verbose) console.log(`   🔘 Checking buttons for aria labels...`);
    result.buttonsWithoutLabel = await page
      .locator("button:not([aria-label])")
      .filter({ hasText: "" })
      .count();
    if (result.buttonsWithoutLabel > 0) {
      result.warnings.push(`${result.buttonsWithoutLabel} buttons without aria-label`);
    }

    if (options.verbose) console.log(`   📝 Checking heading hierarchy...`);
    result.headingsCount = await page.locator("h1, h2, h3, h4, h5, h6").count();
    if (result.headingsCount === 0) {
      result.errors.push("No headings found");
    }

    // Extended accessibility tests
    if (options.testKeyboardNavigation) {
      if (options.verbose) console.log(`   ⌨️  Testing keyboard navigation...`);
      await this.testKeyboardNavigation(page, result, options);
    }

    if (options.testColorContrast) {
      if (options.verbose) console.log(`   🎨 Testing color contrast...`);
      await this.testColorContrast(page, result, options);
    }

    if (options.testFocusManagement) {
      if (options.verbose) console.log(`   🎯 Testing focus management...`);
      await this.testFocusManagement(page, result, options);
    }

    // Screenshots
    if (options.captureScreenshots) {
      if (options.verbose) console.log(`   📸 Capturing screenshots...`);
      await this.captureScreenshots(page, result.url, result, options);
    }

    // Run pa11y accessibility tests FIRST to avoid context destruction issues
    if (options.verbose) console.log(`   🔍 Running pa11y accessibility tests...`);
    await this.runPa11yTests(result, options, page);
    
    // Run comprehensive analysis if enabled (AFTER pa11y to preserve page context)
    if (this.enableComprehensiveAnalysis) {
      if (options.verbose) console.log(`   📈 Running comprehensive analysis...`);
      await this.runComprehensiveAnalysis(page, result, options);
    }

    // Determine pass/fail status
    result.passed = result.errors.length === 0;
  }

  /**
   * Run comprehensive analysis using existing analyzers
   */
  private async runComprehensiveAnalysis(page: Page, result: AccessibilityResult, options: TestOptions): Promise<void> {
    try {
      if (options.verbose) console.log(`   📊 Running comprehensive analysis...`);
      
      const url = result.url;
      const analyses: Promise<void>[] = [];
      
      // Content Weight Analysis
      if (this.contentWeightAnalyzer) {
        analyses.push(this.runContentWeightAnalysis(page, url, result, options));
      }
      
      // Enhanced Performance Analysis
      if (this.performanceCollector) {
        analyses.push(this.runEnhancedPerformanceAnalysis(page, url, result, options));
      }
      
      // SEO Analysis
      if (this.seoAnalyzer) {
        analyses.push(this.runSEOAnalysis(page, url, result, options));
      }
      
      // Mobile Friendliness Analysis
      if (this.mobileFriendlinessAnalyzer) {
        analyses.push(this.runMobileFriendlinessAnalysis(page, url, result, options));
      }
      
      // Run all analyses in parallel
      await Promise.allSettled(analyses);
      
      // Calculate overall quality score
      this.calculateQualityScore(result);
      
      if (options.verbose) console.log(`   ✅ Comprehensive analysis completed`);
    } catch (error) {
      console.error(`❌ Comprehensive analysis failed: ${error}`);
      // Don't fail the entire test, just log the error
    }
  }
  
  private async runContentWeightAnalysis(page: Page, url: string, result: AccessibilityResult, options: TestOptions): Promise<void> {
    try {
      if (options.verbose) console.log(`     📊 Content weight analysis...`);
      const contentWeightResult = await this.contentWeightAnalyzer!.analyze(page, url, { verbose: options.verbose });
      
      // Add content weight data to result
      (result as any).contentWeight = {
        contentScore: contentWeightResult.overallScore,
        grade: contentWeightResult.grade,
        resourceAnalysis: {
          html: { size: contentWeightResult.contentWeight.html, count: 1 },
          css: { size: contentWeightResult.contentWeight.css, count: 1 },
          javascript: { size: contentWeightResult.contentWeight.javascript, count: 1 },
          images: { size: contentWeightResult.contentWeight.images, count: 1 },
          fonts: { size: contentWeightResult.contentWeight.fonts, count: 1 },
          other: { size: contentWeightResult.contentWeight.other, count: 1 }
        },
        contentMetrics: {
          textToCodeRatio: contentWeightResult.contentAnalysis.textToCodeRatio,
          totalSize: contentWeightResult.contentWeight.total,
          contentSize: contentWeightResult.contentAnalysis.textContent
        }
      };
    } catch (error) {
      if (options.verbose) console.log(`     ❌ Content weight analysis failed: ${error}`);
    }
  }
  
  private async runEnhancedPerformanceAnalysis(page: Page, url: string, result: AccessibilityResult, options: TestOptions): Promise<void> {
    try {
      if (options.verbose) console.log(`     ⚡ Enhanced performance analysis...`);
      const performanceResult = await this.performanceCollector!.collectEnhancedMetrics(page, url);
      
      // Add enhanced performance data to result
      (result as any).enhancedPerformance = {
        performanceScore: performanceResult.performanceScore,
        grade: performanceResult.performanceGrade,
        coreWebVitals: {
          fcp: { value: performanceResult.firstContentfulPaint, rating: this.getRating(performanceResult.firstContentfulPaint, [1800, 3000]) },
          lcp: { value: performanceResult.lcp, rating: this.getRating(performanceResult.lcp, [2500, 4000]) },
          cls: { value: performanceResult.cls, rating: this.getRating(performanceResult.cls, [0.1, 0.25]) },
          inp: { value: performanceResult.inp, rating: this.getRating(performanceResult.inp, [200, 500]) }
        },
        metrics: {
          ttfb: { value: performanceResult.ttfb, rating: this.getRating(performanceResult.ttfb, [800, 1800]) },
          fid: { value: performanceResult.fid, rating: this.getRating(performanceResult.fid, [100, 300]) },
          tbt: { value: performanceResult.tbt, rating: this.getRating(performanceResult.tbt, [200, 600]) },
          si: { value: performanceResult.speedIndex, rating: this.getRating(performanceResult.speedIndex, [3400, 5800]) }
        }
      };
    } catch (error) {
      if (options.verbose) console.log(`     ❌ Enhanced performance analysis failed: ${error}`);
    }
  }
  
  private async runSEOAnalysis(page: Page, url: string, result: AccessibilityResult, options: TestOptions): Promise<void> {
    try {
      if (options.verbose) console.log(`     🔍 SEO analysis...`);
      const seoResult = await this.seoAnalyzer!.analyzeSEO(page, url);
      
      // Add SEO data to result
      (result as any).enhancedSEO = {
        seoScore: seoResult.overallSEOScore,
        grade: seoResult.seoGrade,
        metaData: {
          title: seoResult.metaTags.title.content || '',
          titleLength: seoResult.metaTags.title.length,
          description: seoResult.metaTags.description.content || '',
          descriptionLength: seoResult.metaTags.description.length,
          keywords: seoResult.metaTags.keywords?.content || ''
        },
        headingStructure: {
          h1: seoResult.headingStructure.h1Count,
          h2: seoResult.headingStructure.h2Count,
          h3: seoResult.headingStructure.h3Count,
          h4: seoResult.headingStructure.h4Count,
          h5: seoResult.headingStructure.h5Count,
          h6: seoResult.headingStructure.h6Count
        },
        contentAnalysis: {
          wordCount: seoResult.wordCount,
          readabilityScore: seoResult.readabilityScore,
          textToCodeRatio: 0 // Will be calculated by content weight analyzer
        },
        socialTags: {
          openGraph: Object.keys(seoResult.socialTags.openGraph).length,
          twitterCard: Object.keys(seoResult.socialTags.twitterCard).length
        },
        technicalSEO: {
          internalLinks: seoResult.technicalSEO.linkAnalysis.internalLinks,
          externalLinks: seoResult.technicalSEO.linkAnalysis.externalLinks,
          altTextCoverage: 100 // Will be calculated based on image analysis
        }
      };
    } catch (error) {
      if (options.verbose) console.log(`     ❌ SEO analysis failed: ${error}`);
    }
  }
  
  private async runMobileFriendlinessAnalysis(page: Page, url: string, result: AccessibilityResult, options: TestOptions): Promise<void> {
    try {
      if (options.verbose) console.log(`     📱 Mobile friendliness analysis...`);
      const mobileResult = await this.mobileFriendlinessAnalyzer!.analyzeMobileFriendliness(page, url, false);
      
      // Add mobile friendliness data to result
      (result as any).mobileFriendliness = mobileResult;
    } catch (error) {
      if (options.verbose) console.log(`     ❌ Mobile friendliness analysis failed: ${error}`);
    }
  }
  
  private calculateQualityScore(result: AccessibilityResult): void {
    try {
      const scores: { [key: string]: number } = {};
      let totalWeight = 0;
      
      // Accessibility score (25% weight)
      if (result.pa11yScore !== undefined) {
        scores.accessibility = result.pa11yScore;
        totalWeight += 25;
      }
      
      // Performance score (25% weight)
      if ((result as any).enhancedPerformance?.performanceScore) {
        scores.performance = (result as any).enhancedPerformance.performanceScore;
        totalWeight += 25;
      }
      
      // SEO score (25% weight)
      if ((result as any).enhancedSEO?.seoScore) {
        scores.seo = (result as any).enhancedSEO.seoScore;
        totalWeight += 25;
      }
      
      // Content score (15% weight)
      if ((result as any).contentWeight?.contentScore) {
        scores.content = (result as any).contentWeight.contentScore;
        totalWeight += 15;
      }
      
      // Mobile score (10% weight)
      if ((result as any).mobileFriendliness?.overallScore) {
        scores.mobile = (result as any).mobileFriendliness.overallScore;
        totalWeight += 10;
      }
      
      if (totalWeight > 0) {
        // Calculate weighted average
        const weightedSum = Object.entries(scores).reduce((sum, [key, score]) => {
          const weight = key === 'accessibility' || key === 'performance' || key === 'seo' ? 25 :
                        key === 'content' ? 15 : 10;
          return sum + (score * weight / 100);
        }, 0);
        
        const overallScore = Math.round((weightedSum / totalWeight) * 100);
        const grade = this.calculateGrade(overallScore);
        
        (result as any).qualityScore = {
          score: overallScore,
          grade,
          breakdown: scores
        };
      }
    } catch (error) {
      console.error(`Quality score calculation failed: ${error}`);
    }
  }
  
  private calculateGrade(score: number): string {
    if (score >= 90) return 'A';
    if (score >= 80) return 'B';
    if (score >= 70) return 'C';
    if (score >= 60) return 'D';
    return 'F';
  }
  
  private getRating(value: number, thresholds: [number, number]): string {
    if (value <= thresholds[0]) return 'good';
    if (value <= thresholds[1]) return 'needs-improvement';
    return 'poor';
  }

  /**
   * Extract pa11y test logic for reuse
   * Modified to use shared browser instance to prevent duplicate launches
   */
  private async runPa11yTests(result: AccessibilityResult, options: TestOptions, sharedPage?: Page): Promise<void> {
    try {
      // If we have a shared page, use axe-core directly instead of launching pa11y
      // This prevents execution context destruction that affects other analyzers
      if (sharedPage) {
        await this.runAxeTests(sharedPage, result, options);
        return;
      }
      
      const pa11yResult = await pa11y(result.url, {
        timeout: options.timeout || 15000,
        wait: options.wait || (this.usePooling ? 1000 : 2000), // Shorter wait for pooled
        standard: options.pa11yStandard || 'WCAG2AA',
        hideElements: options.hideElements || 'iframe[src*="google-analytics"], iframe[src*="doubleclick"]',
        includeNotices: options.includeNotices !== false,
        includeWarnings: options.includeWarnings !== false,
        runners: options.runners || (this.usePooling ? ['axe'] : ['axe', 'htmlcs']),
        chromeLaunchConfig: {
          ...options.chromeLaunchConfig,
          args: [
            '--disable-web-security',
            '--disable-features=VizDisplayCompositor',
            '--no-sandbox',
            '--disable-setuid-sandbox',
            '--disable-dev-shm-usage',
            '--disable-gpu',
            ...(this.usePooling ? [] : [
              '--disable-background-timer-throttling',
              '--disable-backgrounding-occluded-windows',
              '--disable-renderer-backgrounding'
            ])
          ]
        },
        log: options.verbose ? console : undefined,
      });

      // Convert pa11y results
      pa11yResult.issues.forEach((issue) => {
        const detailedIssue: Pa11yIssue = {
          code: issue.code,
          message: issue.message,
          type: issue.type as 'error' | 'warning' | 'notice',
          selector: issue.selector,
          context: issue.context,
          impact: (issue as any).impact,
          help: (issue as any).help,
          helpUrl: (issue as any).helpUrl
        };
        
        result.pa11yIssues = result.pa11yIssues || [];
        result.pa11yIssues.push(detailedIssue);
        
        // For compatibility
        const message = `${issue.code}: ${issue.message}`;
        if (issue.type === 'error') {
          result.errors.push(message);
        } else if (issue.type === 'warning') {
          result.warnings.push(message);
        } else if (issue.type === 'notice') {
          result.warnings.push(`Notice: ${message}`);
        }
      });

      // Calculate pa11y score - better algorithm that considers severity and quantity
      if (pa11yResult.issues && pa11yResult.issues.length > 0) {
        let totalDeductions = 0;
        
        pa11yResult.issues.forEach((issue: any) => {
          if (issue.type === 'error') {
            // Critical errors: -5 points each, but cap at 20 to avoid too harsh penalty
            totalDeductions += Math.min(5, 20 / Math.max(1, pa11yResult.issues.filter((i: any) => i.type === 'error').length));
          } else if (issue.type === 'warning') {
            // Warnings: -1 point each, but cap at 10 total
            totalDeductions += Math.min(1, 10 / Math.max(1, pa11yResult.issues.filter((i: any) => i.type === 'warning').length));
          } else if (issue.type === 'notice') {
            // Notices: -0.5 points each, cap at 5 total
            totalDeductions += Math.min(0.5, 5 / Math.max(1, pa11yResult.issues.filter((i: any) => i.type === 'notice').length));
          }
        });
        
        result.pa11yScore = Math.max(0, Math.round(100 - totalDeductions));
      } else {
        result.pa11yScore = 100;
      }

      // Additional pa11y metrics
      if (pa11yResult.documentTitle) {
        result.title = pa11yResult.documentTitle;
      }

    } catch (pa11yError) {
      // Fallback score calculation
      const errorMessage = pa11yError instanceof Error ? pa11yError.message : String(pa11yError);
      
      if (options.verbose && !errorMessage.includes('timeout')) {
        console.log(`   ⚠️  pa11y warning: ${errorMessage}`);
        result.warnings.push(`pa11y test issue: ${errorMessage}`);
      }
      
      let fallbackScore = 100;
      fallbackScore -= result.errors.length * 15;
      fallbackScore -= result.warnings.length * 5;
      fallbackScore -= result.imagesWithoutAlt * 3;
      fallbackScore -= result.buttonsWithoutLabel * 5;
      if (result.headingsCount === 0) fallbackScore -= 20;
      
      result.pa11yScore = Math.max(0, fallbackScore);
      
      if (options.verbose) {
        console.log(`   🔢 Calculated fallback pa11y score: ${result.pa11yScore}/100`);
      }
    }
  }

  /**
   * Run axe-core tests directly on shared page to avoid launching new browsers
   */
  private async runAxeTests(page: Page, result: AccessibilityResult, options: TestOptions): Promise<void> {
    try {
      // Inject axe-core if not already present
      await page.addScriptTag({
        url: 'https://unpkg.com/axe-core@latest/axe.min.js'
      });
      
      // Run axe tests
      const axeResults = await page.evaluate(() => {
        return new Promise((resolve) => {
          // @ts-ignore
          window.axe.run({
            tags: ['wcag2a', 'wcag2aa', 'wcag21aa'],
            rules: {
              'bypass': { enabled: true },
              'color-contrast': { enabled: true },
              'focus-order': { enabled: true },
              'keyboard': { enabled: true },
              'label': { enabled: true },
              'link-name': { enabled: true },
              'page-has-heading-one': { enabled: true },
              'region': { enabled: true }
            }
          }, (err: any, results: any) => {
            if (err) {
              resolve({ violations: [], passes: [], incomplete: [], inapplicable: [] });
            } else {
              resolve(results);
            }
          });
        });
      });
      
      // Convert axe results to pa11y format
      if (axeResults && typeof axeResults === 'object' && 'violations' in axeResults) {
        const violations = (axeResults as any).violations || [];
        
        if (violations.length === 0) {
          // Axe found no violations - don't run pa11y backup to preserve page context
          // Instead, calculate a good accessibility score based on basic checks
          let baseScore = 100;
          
          // Deduct based on basic accessibility issues found earlier
          baseScore -= result.errors.length * 10;  // Each error: -10 points
          baseScore -= result.warnings.length * 3; // Each warning: -3 points
          baseScore -= result.imagesWithoutAlt * 2; // Each missing alt: -2 points
          baseScore -= result.buttonsWithoutLabel * 3; // Each unlabeled button: -3 points
          if (result.headingsCount === 0) baseScore -= 15; // No headings: -15 points
          
          result.pa11yScore = Math.max(0, Math.min(100, baseScore));
          
          if (options.verbose) {
            log.debug(`Axe found no violations, using calculated score: ${result.pa11yScore}/100 (preserved page context)`);
          }
          
          return;
        }
        
        violations.forEach((violation: any) => {
          violation.nodes.forEach((node: any) => {
            const detailedIssue: Pa11yIssue = {
              code: violation.id,
              message: violation.description,
              type: violation.impact === 'critical' || violation.impact === 'serious' ? 'error' : 'warning',
              selector: node.target.join(', '),
              context: node.html || '',
              impact: violation.impact,
              help: violation.help,
              helpUrl: violation.helpUrl
            };
            
            result.pa11yIssues = result.pa11yIssues || [];
            result.pa11yIssues.push(detailedIssue);
            
            const message = `${violation.id}: ${violation.description}`;
            if (detailedIssue.type === 'error') {
              result.errors.push(message);
            } else {
              result.warnings.push(message);
            }
          });
        });
        
        // Calculate score based on violations - matching pa11y algorithm
        let totalDeductions = 0;
        
        violations.forEach((violation: any) => {
          violation.nodes.forEach((node: any) => {
            if (violation.impact === 'critical' || violation.impact === 'serious') {
              // Critical/serious: -5 points each, cap at 20
              totalDeductions += Math.min(5, 20 / Math.max(1, violations.filter((v: any) => v.impact === 'critical' || v.impact === 'serious').reduce((sum: number, v: any) => sum + v.nodes.length, 0)));
            } else if (violation.impact === 'moderate') {
              // Moderate: -1 point each, cap at 10
              totalDeductions += Math.min(1, 10 / Math.max(1, violations.filter((v: any) => v.impact === 'moderate').reduce((sum: number, v: any) => sum + v.nodes.length, 0)));
            } else if (violation.impact === 'minor') {
              // Minor: -0.5 points each, cap at 5
              totalDeductions += Math.min(0.5, 5 / Math.max(1, violations.filter((v: any) => v.impact === 'minor').reduce((sum: number, v: any) => sum + v.nodes.length, 0)));
            }
          });
        });
        
        result.pa11yScore = Math.max(0, Math.round(100 - totalDeductions));
      } else {
        // Axe results don't have expected format
        console.warn('⚠️ Axe results have unexpected format, using default score');
        result.pa11yScore = 100;
      }
      
      if (options.verbose) {
        console.log(`   🔢 Axe-core accessibility score: ${result.pa11yScore}/100`);
      }
      
    } catch (error) {
      // Always show this fallback - indicates accessibility testing issues
      log.fallback('Axe-core Test', 'test failed', 'using basic checks', error);
      
      let fallbackScore = 100;
      fallbackScore -= result.errors.length * 15;
      fallbackScore -= result.warnings.length * 5;
      fallbackScore -= result.imagesWithoutAlt * 3;
      fallbackScore -= result.buttonsWithoutLabel * 5;
      if (result.headingsCount === 0) fallbackScore -= 20;
      
      result.pa11yScore = Math.max(0, fallbackScore);
      
      if (options.verbose) {
        console.log(`   🔢 Calculated fallback accessibility score: ${result.pa11yScore}/100`);
      }
    }
  }
  
  /**
   * Fallback to original pa11y when axe finds no issues
   */
  private async runOriginalPa11yAsBackup(result: AccessibilityResult, options: TestOptions): Promise<void> {
    try {
      if (options.verbose) {
        console.log('   🔄 Running pa11y as backup accessibility test...');
      }
      const pa11y = require('pa11y');
      
      const pa11yResult = await pa11y(result.url, {
        timeout: options.timeout || 15000,
        wait: 2000,
        standard: options.pa11yStandard || 'WCAG2AA',
        hideElements: options.hideElements || 'iframe[src*="google-analytics"], iframe[src*="doubleclick"]',
        includeNotices: options.includeNotices !== false,
        includeWarnings: options.includeWarnings !== false,
        runners: ['axe', 'htmlcs'],
        chromeLaunchConfig: {
          ...options.chromeLaunchConfig,
          args: [
            '--disable-web-security',
            '--disable-features=VizDisplayCompositor',
            '--no-sandbox',
            '--disable-setuid-sandbox',
            '--disable-dev-shm-usage',
            '--disable-gpu'
          ]
        }
      });
      
      // Convert pa11y results to our format
      pa11yResult.issues.forEach((issue: any) => {
        const detailedIssue: Pa11yIssue = {
          code: issue.code,
          message: issue.message,
          type: issue.type as 'error' | 'warning' | 'notice',
          selector: issue.selector,
          context: issue.context,
          impact: (issue as any).impact,
          help: (issue as any).help,
          helpUrl: (issue as any).helpUrl
        };
        
        result.pa11yIssues = result.pa11yIssues || [];
        result.pa11yIssues.push(detailedIssue);
        
        // For compatibility
        const message = `${issue.code}: ${issue.message}`;
        if (issue.type === 'error') {
          result.errors.push(message);
        } else if (issue.type === 'warning') {
          result.warnings.push(message);
        } else if (issue.type === 'notice') {
          result.warnings.push(`Notice: ${message}`);
        }
      });
      
      // Calculate pa11y score
      if (pa11yResult.issues && pa11yResult.issues.length > 0) {
        let totalDeductions = 0;
        pa11yResult.issues.forEach((issue: any) => {
          if (issue.type === 'error') {
            totalDeductions += Math.min(5, 20 / Math.max(1, pa11yResult.issues.filter((i: any) => i.type === 'error').length));
          } else if (issue.type === 'warning') {
            totalDeductions += Math.min(1, 10 / Math.max(1, pa11yResult.issues.filter((i: any) => i.type === 'warning').length));
          } else if (issue.type === 'notice') {
            totalDeductions += Math.min(0.5, 5 / Math.max(1, pa11yResult.issues.filter((i: any) => i.type === 'notice').length));
          }
        });
        result.pa11yScore = Math.max(0, Math.round(100 - totalDeductions));
        if (options.verbose) {
          console.log(`   🔢 Pa11y backup found ${pa11yResult.issues.length} issues, score: ${result.pa11yScore}/100`);
        }
      } else {
        result.pa11yScore = 100;
        if (options.verbose) {
          console.log('   ✅ Pa11y backup also found no issues');
        }
      }
      
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      // Always show this critical fallback - both accessibility test engines failed
      log.fallback('Pa11y Backup Test', 'backup test also failed', 'using default score', errorMessage);
      result.pa11yScore = 100; // Default score when both axe and pa11y fail
    }
  }
}
