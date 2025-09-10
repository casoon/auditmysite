/**
 * 🚀 Pooled Accessibility Checker (Full Analysis)
 * 
 * Comprehensive accessibility checker that uses browser pool management
 * for better performance and resource utilization while providing full analysis.
 */

import { AccessibilityResult } from '../../types';
import { PerformanceMetrics, SEOMetrics } from '../../types/enhanced-metrics';

export interface PooledFullTestOptions {
  maxPages?: number;
  timeout?: number;
  pa11yStandard?: 'WCAG2A' | 'WCAG2AA' | 'WCAG2AAA';
  enhancedAnalysis?: boolean;
  contentWeightAnalysis?: boolean;
  enhancedPerformanceAnalysis?: boolean;
  enhancedSeoAnalysis?: boolean;
  semanticAnalysis?: boolean;
  maxConcurrent?: number;
  captureScreenshots?: boolean;
  includeWarnings?: boolean;
}

export class PooledAccessibilityCheckerFull {
  private poolManager: any;
  private accessibilityChecker: any;
  
  constructor(poolManager: any) {
    this.poolManager = poolManager;
  }
  
  /**
   * Initialize accessibility analysis components
   */
  async initialize(): Promise<void> {
    // Load accessibility analyzer components - using existing AccessibilityChecker
    const { AccessibilityChecker } = require('./accessibility-checker');
    this.accessibilityChecker = new AccessibilityChecker();
    
    console.log('🔧 PooledAccessibilityCheckerFull initialized with browser pool');
  }
  
  /**
   * Test multiple pages with full analysis using pooled browsers
   */
  async testMultiplePagesWithFullAnalysis(
    urls: string[], 
    options: PooledFullTestOptions = {}
  ): Promise<AccessibilityResult[]> {
    if (!this.accessibilityChecker) {
      await this.initialize();
    }
    
    const {
      maxPages = 20,
      timeout = 10000,
      pa11yStandard = 'WCAG2AA',
      enhancedAnalysis = true,
      contentWeightAnalysis = false,
      enhancedPerformanceAnalysis = false,
      enhancedSeoAnalysis = false,
      semanticAnalysis = false,
      maxConcurrent = 3,
      captureScreenshots = false,
      includeWarnings = true
    } = options;
    
    const limitedUrls = urls.slice(0, maxPages);
    console.log(`🔍 Running enhanced analysis on ${limitedUrls.length} pages with pooled browsers...`);
    
    // Process URLs in concurrent batches using browser pool
    const batchSize = Math.min(maxConcurrent, this.poolManager.getPoolSize());
    const results: AccessibilityResult[] = [];
    
    for (let i = 0; i < limitedUrls.length; i += batchSize) {
      const batch = limitedUrls.slice(i, i + batchSize);
      const batchPromises = batch.map(url => this.analyzePageWithPool(url, {
        timeout,
        pa11yStandard,
        enhancedAnalysis,
        contentWeightAnalysis,
        enhancedPerformanceAnalysis,
        enhancedSeoAnalysis,
        semanticAnalysis,
        captureScreenshots,
        includeWarnings
      }));
      
      try {
        const batchResults = await Promise.allSettled(batchPromises);
        
        for (const result of batchResults) {
          if (result.status === 'fulfilled') {
            results.push(result.value);
          } else {
            console.warn('Enhanced analysis failed for URL:', result.reason);
            // Add error result
            results.push({
              url: 'unknown',
              title: 'Analysis Failed',
              imagesWithoutAlt: 0,
              buttonsWithoutLabel: 0,
              headingsCount: 0,
              errors: ['Enhanced analysis failed: ' + result.reason.message],
              warnings: [],
              passed: false,
              crashed: true,
              duration: 0
            });
          }
        }
        
        console.log(`✅ Completed batch ${Math.floor(i/batchSize) + 1}/${Math.ceil(limitedUrls.length/batchSize)}`);
      } catch (error) {
        console.error('Batch processing failed:', error);
      }
    }
    
        console.log(`🎯 Full pooled analysis completed: ${results.length} results`);
    return results;
  }
  
  /**
   * Analyze single page using pooled browser with enhanced features
   */
  private async analyzePageWithPool(url: string, options: any): Promise<AccessibilityResult> {
    const browser = await this.poolManager.getBrowser();
    const context = await browser.newContext({
      viewport: { width: 1366, height: 768 },
      userAgent: 'Mozilla/5.0 (compatible; AuditMySite/2.0; +https://github.com/jseidel/AuditMySite)'
    });
    
    try {
      const page = await context.newPage();
      
      // Set timeout and navigate
      page.setDefaultTimeout(options.timeout);
      await page.goto(url, { 
        waitUntil: 'domcontentloaded',
        timeout: options.timeout 
      });
      
      // Initialize result
      const result: AccessibilityResult = {
        url,
        title: '',
        imagesWithoutAlt: 0,
        buttonsWithoutLabel: 0,
        headingsCount: 0,
        errors: [],
        warnings: [],
        passed: false,
        duration: 0
      };
      
      // Run pa11y accessibility analysis
      await this.runAccessibilityAnalysis(page, result, options);
      
      // Run enhanced analyses if requested and store in performanceMetrics
      if (options.enhancedPerformanceAnalysis) {
        const enhancedPerf = await this.runPerformanceAnalysis(page);
        result.performanceMetrics = {
          loadTime: enhancedPerf.loadComplete || 0,
          domContentLoaded: enhancedPerf.domContentLoaded || 0,
          firstPaint: enhancedPerf.firstPaint || 0,
          renderTime: enhancedPerf.loadComplete || 0,
          firstContentfulPaint: enhancedPerf.firstContentfulPaint || 0,
          largestContentfulPaint: enhancedPerf.largestContentfulPaint || 0,
          cumulativeLayoutShift: enhancedPerf.cumulativeLayoutShift,
          timeToFirstByte: enhancedPerf.timeToFirstByte
        };
      }
      
      return result;
    } finally {
      await context.close();
      this.poolManager.releaseBrowser(browser);
    }
  }
  
  /**
   * Run accessibility analysis with pa11y
   */
  private async runAccessibilityAnalysis(page: any, result: AccessibilityResult, options: any): Promise<void> {
    try {
      // Use pa11y to analyze the page
      const pa11y = require('pa11y');
      const pa11yResult = await pa11y(result.url, {
        standard: options.pa11yStandard,
        timeout: options.timeout,
        wait: 1000,
        chromeLaunchConfig: {
          args: ['--no-sandbox', '--disable-web-security']
        },
        includeNotices: false,
        includeWarnings: options.includeWarnings
      });
      
      result.errors = pa11yResult.issues
        .filter((issue: any) => issue.type === 'error')
        .map((issue: any) => issue.message);
      
      result.warnings = pa11yResult.issues
        .filter((issue: any) => issue.type === 'warning')
        .map((issue: any) => issue.message);
        
      // Set basic accessibility metrics
      result.title = pa11yResult.pageUrl?.split('/').pop() || 'Page';
      result.passed = result.errors.length === 0;
      
    } catch (error) {
      result.errors.push('Accessibility analysis failed: ' + (error as Error).message);
      result.crashed = true;
    }
  }
  
  /**
   * Run performance analysis
   */
  private async runPerformanceAnalysis(page: any): Promise<any> {
    try {
      // Collect Web Vitals and performance metrics
      const performanceMetrics = await page.evaluate(() => {
        return new Promise((resolve) => {
          const metrics: any = {};
          
          try {
            // Add navigation timing
            const navigation = performance.getEntriesByType('navigation')[0] as PerformanceNavigationTiming;
            if (navigation) {
              metrics.domContentLoaded = navigation.domContentLoadedEventEnd - navigation.domContentLoadedEventStart;
              metrics.loadComplete = navigation.loadEventEnd - navigation.loadEventStart;
              metrics.timeToFirstByte = navigation.responseStart - navigation.requestStart;
            }
            
            // Try to get paint timing
            const paintEntries = performance.getEntriesByType('paint');
            paintEntries.forEach((entry) => {
              if (entry.name === 'first-paint') {
                metrics.firstPaint = entry.startTime;
              }
              if (entry.name === 'first-contentful-paint') {
                metrics.firstContentfulPaint = entry.startTime;
              }
            });
            
            // Set defaults for missing metrics
            metrics.largestContentfulPaint = metrics.firstContentfulPaint || 0;
            metrics.cumulativeLayoutShift = 0;
          } catch (e) {
            console.warn('Performance metrics collection failed:', e);
          }
          
          resolve(metrics);
        });
      });
      
      return performanceMetrics;
    } catch (error) {
      console.warn('Performance analysis failed:', error);
      return {
        domContentLoaded: 0,
        loadComplete: 0,
        firstPaint: 0,
        firstContentfulPaint: 0,
        largestContentfulPaint: 0,
        timeToFirstByte: 0,
        cumulativeLayoutShift: 0
      };
    }
  }
  
  /**
   * Run SEO analysis
   */
  private async runSEOAnalysis(page: any): Promise<any> {
    try {
      const seoMetrics = await page.evaluate(() => {
        const title = document.querySelector('title')?.textContent || '';
        const description = document.querySelector('meta[name="description"]')?.getAttribute('content') || '';
        
        // Collect headings
        const h1 = Array.from(document.querySelectorAll('h1')).map(h => h.textContent?.trim() || '');
        const h2 = Array.from(document.querySelectorAll('h2')).map(h => h.textContent?.trim() || '');
        const h3 = Array.from(document.querySelectorAll('h3')).map(h => h.textContent?.trim() || '');
        
        // Collect image data
        const images = Array.from(document.querySelectorAll('img'));
        const imageStats = {
          total: images.length,
          missingAlt: images.filter(img => !img.getAttribute('alt')).length,
          emptyAlt: images.filter(img => img.getAttribute('alt') === '').length
        };
        
        // Collect Open Graph data
        const ogTitle = document.querySelector('meta[property="og:title"]')?.getAttribute('content');
        const ogDescription = document.querySelector('meta[property="og:description"]')?.getAttribute('content');
        const ogImage = document.querySelector('meta[property="og:image"]')?.getAttribute('content');
        
        return {
          title,
          description,
          headings: { h1, h2, h3 },
          images: imageStats,
          openGraph: {
            title: ogTitle,
            description: ogDescription,
            image: ogImage
          }
        };
      });
      
      return seoMetrics;
    } catch (error) {
      console.warn('SEO analysis failed:', error);
      return {
        title: '',
        description: '',
        headings: { h1: [], h2: [], h3: [] },
        images: { total: 0, missingAlt: 0, emptyAlt: 0 },
        openGraph: {}
      };
    }
  }
  
  /**
   * Run content weight analysis
   */
  private async runContentWeightAnalysis(page: any): Promise<any> {
    try {
      const contentWeight = await page.evaluate(() => {
        const images = document.querySelectorAll('img').length;
        const scripts = document.querySelectorAll('script').length;
        const stylesheets = document.querySelectorAll('link[rel="stylesheet"]').length;
        
        return {
          images,
          scripts,
          stylesheets,
          totalResources: images + scripts + stylesheets
        };
      });
      
      return contentWeight;
    } catch (error) {
      console.warn('Content weight analysis failed:', error);
      return {
        images: 0,
        scripts: 0,
        stylesheets: 0,
        totalResources: 0
      };
    }
  }
  
  /**
   * Run semantic analysis
   */
  private async runSemanticAnalysis(page: any): Promise<any> {
    try {
      const semantics = await page.evaluate(() => {
        const landmarks = document.querySelectorAll('[role], main, nav, header, footer, aside, section').length;
        const headingsCount = document.querySelectorAll('h1, h2, h3, h4, h5, h6').length;
        const listsCount = document.querySelectorAll('ul, ol').length;
        
        return {
          landmarks,
          headingsCount,
          listsCount,
          hasMainLandmark: !!document.querySelector('main, [role="main"]'),
          hasNavigation: !!document.querySelector('nav, [role="navigation"]')
        };
      });
      
      return semantics;
    } catch (error) {
      console.warn('Semantic analysis failed:', error);
      return {
        landmarks: 0,
        headingsCount: 0,
        listsCount: 0,
        hasMainLandmark: false,
        hasNavigation: false
      };
    }
  }
  
  /**
   * Get pool status for monitoring
   */
  getPoolStatus(): any {
    return this.poolManager?.getStatus() || { 
      metrics: { efficiency: 0 } 
    };
  }
  
  /**
   * Cleanup resources
   */
  async cleanup(): Promise<void> {
    if (this.accessibilityChecker?.cleanup) {
      await this.accessibilityChecker.cleanup();
    }
    console.log('🧹 PooledAccessibilityCheckerFull cleaned up');
  }
}
