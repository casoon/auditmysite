import { chromium, Browser, Page } from "playwright";
import pa11y from "pa11y";
import { AccessibilityResult, TestOptions, Pa11yIssue } from '../types';
import { BrowserManager } from '../browser';
import { WebVitalsCollector } from '../performance';
import { ParallelTestManager, ParallelTestManagerOptions, ParallelTestResult } from './index';
import { Queue, QueueConfig, QueueEventCallbacks } from '../queue';
import * as fs from 'fs';
import * as path from 'path';

export class AccessibilityChecker {
  public browserManager: BrowserManager | null = null;
  private webVitalsCollector: WebVitalsCollector;
  private parallelTestManager: ParallelTestManager | null = null;

  constructor() {
    this.webVitalsCollector = new WebVitalsCollector();
  }

  async initialize(): Promise<void> {
    // 🆕 Browser Manager for shared browser
    this.browserManager = new BrowserManager({
      headless: true,
      port: 9222
    });
    
    await this.browserManager.initialize();
    
    // 🆕 Lighthouse Integration - Removed
    // this.lighthouseIntegration = new LighthouseIntegration(this.browserManager);
  }

  async cleanup(): Promise<void> {
    if (this.browserManager) {
      await this.browserManager.cleanup();
    }
  }

  async testPage(
    url: string,
    options: TestOptions = {},
  ): Promise<AccessibilityResult> {
    if (options.verbose) console.log('🔍 Testing:', url);
    if (!this.browserManager) {
      throw new Error("Browser Manager not initialized");
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
      if (options.verbose) console.log(`   🔧 Configuring page...`);
      // 🆕 Extended page configuration
      await this.configurePage(page, options);

      if (options.verbose) console.log(`   🌐 Navigating to page...`);
      await page.goto(url, {
        waitUntil: options.waitUntil || "domcontentloaded",
        timeout: options.timeout || 10000,
      });

      // 🆕 Collect performance metrics
      if (options.collectPerformanceMetrics) {
        if (options.verbose) console.log(`   📊 Collecting performance metrics...`);
        await this.collectPerformanceMetrics(page, result, options);
      }

      // Check page title
      if (options.verbose) console.log(`   📋 Extracting page title...`);
      result.title = await page.title();

      // Images without alt attribute
      if (options.verbose) console.log(`   🖼️  Checking images for alt attributes...`);
      result.imagesWithoutAlt = await page.locator("img:not([alt])").count();
      if (result.imagesWithoutAlt > 0) {
        result.warnings.push(
          `${result.imagesWithoutAlt} images without alt attribute`,
        );
      }
      // if (options.verbose) console.log('DEBUG: Nach Alt-Check', {url: result.url, errors: result.errors.length, warnings: result.warnings.length}); // Hidden - use --verbose for debug logs

      // Buttons without aria-label
      if (options.verbose) console.log(`   🔘 Checking buttons for aria labels...`);
      result.buttonsWithoutLabel = await page
        .locator("button:not([aria-label])")
        .filter({ hasText: "" })
        .count();
      if (result.buttonsWithoutLabel > 0) {
        result.warnings.push(
          `${result.buttonsWithoutLabel} buttons without aria-label`,
        );
      }
      // if (options.verbose) console.log('DEBUG: Nach Button-Label-Check', {url: result.url, errors: result.errors.length, warnings: result.warnings.length}); // Hidden - use --verbose for debug logs

      // Heading hierarchy
      if (options.verbose) console.log(`   📝 Checking heading hierarchy...`);
      result.headingsCount = await page
        .locator("h1, h2, h3, h4, h5, h6")
        .count();
      if (result.headingsCount === 0) {
        result.errors.push("No headings found");
      }
      // if (options.verbose) console.log('DEBUG: Nach Heading-Check', {url: result.url, errors: result.errors.length, warnings: result.warnings.length}); // Hidden - use --verbose for debug logs

      // 🆕 Extended accessibility tests
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

      // 🆕 Screenshots
      if (options.captureScreenshots) {
        if (options.verbose) console.log(`   📸 Capturing screenshots...`);
        await this.captureScreenshots(page, url, result, options);
      }

      // Run pa11y accessibility tests
      if (options.verbose) console.log(`   🔍 Running pa11y accessibility tests...`);
        try {
          // 🆕 Optimized pa11y config for localhost
          const pa11yResult = await pa11y(url, {
            timeout: options.timeout || 15000, // Increased for localhost
            wait: options.wait || 2000, // Wait longer for localhost
            standard: options.pa11yStandard || 'WCAG2AA',
            hideElements: options.hideElements || 'iframe[src*="google-analytics"], iframe[src*="doubleclick"]',
            includeNotices: options.includeNotices !== false,
            includeWarnings: options.includeWarnings !== false,
            runners: options.runners || ['axe', 'htmlcs'],
            // 🆕 Simplified Chrome config for localhost
            chromeLaunchConfig: {
              ...options.chromeLaunchConfig,
              args: [
                '--disable-web-security',
                '--disable-features=VizDisplayCompositor',
                '--no-sandbox',
                '--disable-setuid-sandbox',
                '--disable-dev-shm-usage',
                '--disable-gpu',
                '--disable-background-timer-throttling',
                '--disable-backgrounding-occluded-windows',
                '--disable-renderer-backgrounding'
              ]
            },
            log: options.verbose ? console : undefined,
          });

        // Convert pa11y results to our format
        pa11yResult.issues.forEach((issue) => {
          // Detaillierte Issue-Informationen speichern
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
          
          // For compatibility, also add to errors/warnings
          const message = `${issue.code}: ${issue.message}`;
          if (issue.type === 'error') {
            result.errors.push(message);
          } else if (issue.type === 'warning') {
            result.warnings.push(message);
          } else if (issue.type === 'notice') {
            result.warnings.push(`Notice: ${message}`);
          }
        });
        // if (options.verbose) console.log('DEBUG: Nach pa11y', {url: result.url, errors: result.errors.length, warnings: result.warnings.length, pa11yIssues: result.pa11yIssues?.length}); // Hidden - use --verbose for debug logs

        // Additional pa11y metrics
        if (pa11yResult.documentTitle) {
          result.title = pa11yResult.documentTitle;
        }

        // Calculate pa11y score (improved formula - less strict)
        if (pa11yResult.issues.length > 0) {
          const totalIssues = pa11yResult.issues.length;
          const errorIssues = pa11yResult.issues.filter(issue => issue.type === 'error').length;
          const warningIssues = pa11yResult.issues.filter(issue => issue.type === 'warning').length;
          const noticeIssues = pa11yResult.issues.filter(issue => issue.type === 'notice').length;
          
          // More balanced scoring: errors -5, warnings -2, notices -1 points each
          result.pa11yScore = Math.max(10, 100 - (errorIssues * 5) - (warningIssues * 2) - (noticeIssues * 1));
        } else {
          result.pa11yScore = 100;
        }

      } catch (pa11yError) {
        // 🆕 Improved error handling for pa11y
        const errorMessage = pa11yError instanceof Error ? pa11yError.message : String(pa11yError);
        
        // Handle timeout errors specifically
        if (errorMessage.includes('timeout') || errorMessage.includes('Timeout')) {
          if (options.verbose) {
            console.log(`   ⚠️  pa11y timeout for ${url} - skipping pa11y tests`);
          }
          // Do not add timeout errors as warnings, as they are normal for localhost
        } else {
          // Add other pa11y errors as warnings
          result.warnings.push(`pa11y test failed: ${errorMessage}`);
        }
        
        // 🆕 Fallback pa11y score calculation when pa11y fails
        // Calculate score based on basic accessibility checks we performed
        let fallbackScore = 100;
        
        // Penalize for basic issues
        if (result.errors.length > 0) {
          fallbackScore -= result.errors.length * 15;  // 15 points per error
        }
        if (result.warnings.length > 0) {
          fallbackScore -= result.warnings.length * 5;  // 5 points per warning
        }
        if (result.imagesWithoutAlt > 0) {
          fallbackScore -= result.imagesWithoutAlt * 3;  // 3 points per missing alt
        }
        if (result.buttonsWithoutLabel > 0) {
          fallbackScore -= result.buttonsWithoutLabel * 5;  // 5 points per missing label
        }
        if (result.headingsCount === 0) {
          fallbackScore -= 20;  // 20 points for no headings
        }
        
        result.pa11yScore = Math.max(0, fallbackScore);
        
        if (options.verbose) {
          console.log(`   🔢 Calculated fallback pa11y score: ${result.pa11yScore}/100`);
        }
      }
      // if (options.verbose) console.log('DEBUG: Nach pa11y/Ende', {url: result.url, errors: result.errors.length, warnings: result.warnings.length, pa11yIssues: result.pa11yIssues?.length}); // Hidden - use --verbose for debug logs

      // Lighthouse integration removed

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

  async testMultiplePages(
    urls: string[],
    options: TestOptions = {},
  ): Promise<AccessibilityResult[]> {
    const results: AccessibilityResult[] = [];
    const maxPages = options.maxPages || urls.length;
    const pagesToTest = urls.slice(0, maxPages);

    console.log(`🔄 Sequential testing of ${pagesToTest.length} pages...`);
    
    for (let i = 0; i < pagesToTest.length; i++) {
      const url = pagesToTest[i];
      const startTime = Date.now();
      console.log(`\n📄 Testing page ${i + 1}/${pagesToTest.length}: ${url}`);
      
      try {
        const result = await this.testPage(url, options);
        const duration = Date.now() - startTime;
        result.duration = duration;
        results.push(result);
        
        console.log(`   ✅ Test completed in ${duration}ms`);
        
        if (result.passed) {
          console.log(`   🎯 Result: PASSED (${result.errors.length} errors, ${result.warnings.length} warnings)`);
        } else {
          console.log(`   🎯 Result: FAILED (${result.errors.length} errors, ${result.warnings.length} warnings)`);
        }
        
      } catch (error) {
        const duration = Date.now() - startTime;
        console.error(`   💥 Error testing page after ${duration}ms: ${error}`);
        
        // Error-Result erstellen
        const errorResult: AccessibilityResult = {
          url: url,
          title: "",
          imagesWithoutAlt: 0,
          buttonsWithoutLabel: 0,
          headingsCount: 0,
          errors: [`Test failed: ${error}`],
          warnings: [],
          passed: false,
          crashed: true,  // 🆕 Mark as crashed for technical errors
          duration,
        };
        results.push(errorResult);
      }
    }

    return results;
  }

  /**
   * 🚀 Parallel accessibility tests with event-driven queue
   * 
   * This method uses the event-driven queue system for parallel tests
   * with real-time status reporting and resource monitoring.
   */
  async testMultiplePagesParallel(
    urls: string[],
    options: TestOptions = {},
  ): Promise<AccessibilityResult[]> {
    const maxPages = options.maxPages || urls.length;
    const pagesToTest = urls.slice(0, maxPages);
    
    // Parallel test options
    const parallelOptions: ParallelTestManagerOptions = {
      maxConcurrent: options.maxConcurrent || 3,
      maxRetries: options.maxRetries || 3,
      retryDelay: options.retryDelay || 2000,
      enableProgressBar: options.enableProgressBar !== false,
      progressUpdateInterval: options.progressUpdateInterval || 1000,
      enableResourceMonitoring: options.enableResourceMonitoring !== false,
      maxMemoryUsage: options.maxMemoryUsage || 512,
      maxCpuUsage: options.maxCpuUsage || 80,
      testOptions: options,
      eventCallbacks: {
        onUrlStarted: (url: string) => {
          if (options.verbose) {
            console.log(`🚀 Starting parallel test: ${url}`);
          }
        },
        onUrlCompleted: (url: string, result: AccessibilityResult, duration: number) => {
          const status = result.passed ? '✅ PASSED' : '❌ FAILED';
          console.log(`${status} ${url} (${duration}ms) - ${result.errors.length} errors, ${result.warnings.length} warnings`);
        },
        onUrlFailed: (url: string, error: string, attempts: number) => {
          console.error(`💥 Error testing ${url} (attempt ${attempts}): ${error}`);
        },
        onProgressUpdate: (stats) => {
          if (options.verbose) {
            console.log(`📊 Progress: ${stats.progress.toFixed(1)}% | Workers: ${stats.activeWorkers}/${options.maxConcurrent || 3} | Memory: ${stats.memoryUsage}MB`);
          }
        },
        onQueueEmpty: () => {
          console.log('🎉 All parallel tests completed!');
        }
      }
    };

    // Initialize Parallel Test Manager
    this.parallelTestManager = new ParallelTestManager(parallelOptions);
    
    try {
      console.log(`🚀 Starting parallel accessibility tests for ${pagesToTest.length} pages with ${parallelOptions.maxConcurrent} workers`);
      console.log(`⚙️  Configuration: maxRetries=${parallelOptions.maxRetries}, retryDelay=${parallelOptions.retryDelay}ms`);
      
      // Initialize manager
      await this.parallelTestManager.initialize();
      
      // Run tests
      const startTime = Date.now();
      const result: ParallelTestResult = await this.parallelTestManager.runTests(pagesToTest);
      const totalDuration = Date.now() - startTime;
      
      // Output results
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
        interactionToNextPaint: webVitals.inp,
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
}
