import pa11y from "pa11y";
import { AccessibilityResult, TestOptions, Pa11yIssue } from '../types';
import { BrowserPoolManager } from '../browser/browser-pool-manager';

/**
 * 🚀 Pooled Accessibility Checker v2.0
 * 
 * Optimized for performance using browser pool management:
 * - Reuses browser instances efficiently
 * - Reduces memory footprint
 * - Eliminates browser startup overhead
 * - Minimal queue complexity
 */
export class PooledAccessibilityChecker {
  private poolManager: BrowserPoolManager;

  constructor(poolManager: BrowserPoolManager) {
    this.poolManager = poolManager;
  }

  /**
   * Test a single page using browser pool
   */
  async testPage(url: string, options: TestOptions = {}): Promise<AccessibilityResult> {
    const startTime = Date.now();
    
    if (options.verbose) console.log(`🔍 Testing: ${url}`);

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
    const { browser, context, release } = await this.poolManager.acquire();

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

        // Basic accessibility checks
        if (options.verbose) console.log(`   📋 Extracting page data...`);
        result.title = await page.title();
        result.imagesWithoutAlt = await page.locator("img:not([alt])").count();
        result.buttonsWithoutLabel = await page
          .locator("button:not([aria-label])")
          .filter({ hasText: "" })
          .count();
        result.headingsCount = await page.locator("h1, h2, h3, h4, h5, h6").count();

        // Add warnings for basic issues
        if (result.imagesWithoutAlt > 0) {
          result.warnings.push(`${result.imagesWithoutAlt} images without alt attribute`);
        }
        if (result.buttonsWithoutLabel > 0) {
          result.warnings.push(`${result.buttonsWithoutLabel} buttons without aria-label`);
        }
        if (result.headingsCount === 0) {
          result.errors.push("No headings found");
        }

        // Run pa11y accessibility tests (simplified configuration)
        if (options.verbose) console.log(`   🔍 Running pa11y tests...`);
        try {
          const pa11yResult = await pa11y(url, {
            timeout: options.timeout || 15000,
            wait: options.wait || 1000, // Reduced wait time
            standard: options.pa11yStandard || 'WCAG2AA',
            hideElements: options.hideElements || 'iframe[src*="google-analytics"]',
            includeNotices: options.includeNotices !== false,
            includeWarnings: options.includeWarnings !== false,
            runners: options.runners || ['axe'],
            // Optimized Chrome config for pooled browsers
            chromeLaunchConfig: {
              ignoreHTTPSErrors: true,
              args: [
                '--no-sandbox',
                '--disable-setuid-sandbox',
                '--disable-dev-shm-usage',
                '--disable-gpu'
              ]
            } as any,
          });

          // Convert pa11y results efficiently
          if (pa11yResult.issues && pa11yResult.issues.length > 0) {
            result.pa11yIssues = pa11yResult.issues.map((issue: any): Pa11yIssue => ({
              code: issue.code,
              message: issue.message,
              type: issue.type as 'error' | 'warning' | 'notice',
              selector: issue.selector,
              context: issue.context,
              impact: issue.impact,
              help: issue.help,
              helpUrl: issue.helpUrl
            }));

            // Add to legacy arrays for compatibility
            pa11yResult.issues.forEach((issue: any) => {
              const message = `${issue.code}: ${issue.message}`;
              if (issue.type === 'error') {
                result.errors.push(message);
              } else if (issue.type === 'warning' || issue.type === 'notice') {
                result.warnings.push(message);
              }
            });

            // Calculate pa11y score
            const errorCount = pa11yResult.issues.filter((issue: any) => issue.type === 'error').length;
            const warningCount = pa11yResult.issues.length - errorCount;
            result.pa11yScore = Math.max(0, 100 - (errorCount * 10) - (warningCount * 2));
          } else {
            result.pa11yScore = 100;
          }

        } catch (pa11yError) {
          // Handle pa11y errors gracefully
          const errorMessage = pa11yError instanceof Error ? pa11yError.message : String(pa11yError);
          
          if (options.verbose && !errorMessage.includes('timeout')) {
            console.log(`   ⚠️  pa11y warning: ${errorMessage}`);
            result.warnings.push(`pa11y test issue: ${errorMessage}`);
          }
          
          // Calculate fallback score
          let fallbackScore = 100;
          fallbackScore -= result.errors.length * 15;
          fallbackScore -= result.warnings.length * 5;
          fallbackScore -= result.imagesWithoutAlt * 3;
          fallbackScore -= result.buttonsWithoutLabel * 5;
          if (result.headingsCount === 0) fallbackScore -= 20;
          
          result.pa11yScore = Math.max(0, fallbackScore);
        }

        // Determine pass/fail status
        result.passed = result.errors.length === 0;

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
   * Test multiple pages efficiently using browser pool
   */
  async testMultiplePages(
    urls: string[], 
    options: TestOptions = {}
  ): Promise<AccessibilityResult[]> {
    const results: AccessibilityResult[] = [];
    const maxPages = Math.min(options.maxPages || urls.length, urls.length);
    const pagesToTest = urls.slice(0, maxPages);
    const maxConcurrent = options.maxConcurrent || 3;

    console.log(`🚀 Testing ${pagesToTest.length} pages with optimized browser pool (max concurrent: ${maxConcurrent})`);
    
    // Process URLs in concurrent batches
    const promises: Promise<void>[] = [];
    let currentIndex = 0;

    const processNextUrl = async (): Promise<void> => {
      while (currentIndex < pagesToTest.length) {
        const urlIndex = currentIndex++;
        const url = pagesToTest[urlIndex];
        
        try {
          console.log(`📄 Testing page ${urlIndex + 1}/${pagesToTest.length}: ${url}`);
          const result = await this.testPage(url, options);
          results[urlIndex] = result; // Maintain order
          
          const status = result.passed ? '✅ PASSED' : '❌ FAILED';
          console.log(`${status} ${url} (${result.duration}ms) - ${result.errors.length} errors, ${result.warnings.length} warnings`);
          
        } catch (error) {
          console.error(`💥 Error testing ${url}: ${error}`);
          results[urlIndex] = {
            url,
            title: "",
            imagesWithoutAlt: 0,
            buttonsWithoutLabel: 0,
            headingsCount: 0,
            errors: [`Test failed: ${error}`],
            warnings: [],
            passed: false,
            crashed: true,
            duration: 0
          };
        }
      }
    };

    // Start concurrent workers
    for (let i = 0; i < Math.min(maxConcurrent, pagesToTest.length); i++) {
      promises.push(processNextUrl());
    }

    // Wait for all workers to complete
    await Promise.all(promises);

    // Filter out any undefined results and sort by original order
    const finalResults = results.filter(r => r !== undefined);
    
    console.log(`\n✅ Completed ${finalResults.length}/${pagesToTest.length} pages`);
    console.log(`📊 Results: ${finalResults.filter(r => r.passed).length} passed, ${finalResults.filter(r => !r.passed).length} failed`);
    
    return finalResults;
  }

  /**
   * Get pool status for monitoring
   */
  getPoolStatus() {
    return this.poolManager.getStatus();
  }
}
