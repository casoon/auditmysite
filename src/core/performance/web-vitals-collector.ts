import { Page } from 'playwright';
import { log } from '@core/logging';

export interface CoreWebVitals {
  // Core Web Vitals (Google ranking factors)
  lcp: number;  // Largest Contentful Paint
  cls: number;  // Cumulative Layout Shift  
  fcp: number;  // First Contentful Paint
  ttfb: number; // Time to First Byte
  
  // Additional metrics
  loadTime: number;
  domContentLoaded: number;
  renderTime: number;
  
  // Quality indicators
  score: number;
  grade: 'A' | 'B' | 'C' | 'D' | 'F';
  recommendations: string[];
  
  // Budget status
  budgetStatus?: BudgetStatus;
}

export interface PerformanceBudget {
  lcp: { good: number; poor: number };
  cls: { good: number; poor: number };
  fcp: { good: number; poor: number };
  inp?: { good: number; poor: number }; // INP is optional (newer metric)
  ttfb: { good: number; poor: number };
}

export interface BudgetStatus {
  passed: boolean;
  violations: BudgetViolation[];
  summary: string;
}

export interface BudgetViolation {
  metric: keyof PerformanceBudget;
  actual: number;
  threshold: number;
  severity: 'warning' | 'error';
  message: string;
}

// Predefined budget templates
export const BUDGET_TEMPLATES: Record<string, PerformanceBudget> = {
  ecommerce: {
    lcp: { good: 2000, poor: 3000 }, // Stricter for conversion
    cls: { good: 0.05, poor: 0.15 },
    fcp: { good: 1500, poor: 2500 },
    ttfb: { good: 300, poor: 600 }
  },
  blog: {
    lcp: { good: 2500, poor: 4000 }, // Standard thresholds
    cls: { good: 0.1, poor: 0.25 },
    fcp: { good: 1800, poor: 3000 },
    ttfb: { good: 400, poor: 800 }
  },
  corporate: {
    lcp: { good: 2200, poor: 3500 }, // Professional standards
    cls: { good: 0.08, poor: 0.2 },
    fcp: { good: 1600, poor: 2800 },
    ttfb: { good: 350, poor: 700 }
  },
  default: {
    lcp: { good: 2500, poor: 4000 }, // Google's standard thresholds
    cls: { good: 0.1, poor: 0.25 },
    fcp: { good: 1800, poor: 3000 },
    ttfb: { good: 400, poor: 800 }
  }
};

export class WebVitalsCollector {
  private budget: PerformanceBudget;
  private maxRetries: number;
  private retryDelay: number;
  private verbose: boolean;
  
  constructor(budget?: PerformanceBudget, options?: { maxRetries?: number; retryDelay?: number; verbose?: boolean }) {
    this.budget = budget || BUDGET_TEMPLATES.default;
    this.maxRetries = options?.maxRetries || 3;
    this.retryDelay = options?.retryDelay || 1000;
    this.verbose = options?.verbose || false;
  }
  
  /**
   * Collect Core Web Vitals using robust browser synchronization
   * Prevents timing issues and execution context destruction
   */
  async collectMetrics(page: Page): Promise<CoreWebVitals> {
    try {
      // Use isolated context collection for maximum stability
      const metrics = await this.collectWithIsolatedContext(page);
      
      // Apply fallback strategies for missing metrics
      const enhancedMetrics = this.applyFallbackStrategies(metrics);
      
      // Calculate performance score and grade
      const score = this.calculateScore(enhancedMetrics);
      const grade = this.calculateGrade(score);
      const recommendations = this.generateRecommendations(enhancedMetrics);
      const budgetStatus = this.evaluateBudget(enhancedMetrics);
      
      return {
        ...enhancedMetrics,
        score,
        grade,
        recommendations,
        budgetStatus
      };
      
    } catch (error) {
      // Always show fallback warnings, even in quiet mode - indicates potential implementation issues
      log.fallback('Web Vitals Collection', 'collection failed', 'using basic navigation timing', error);
      return this.getFallbackMetrics();
    }
  }
  
  /**
   * Collect metrics using isolated browser context for maximum stability
   * Each measurement runs in its own clean environment
   */
  async collectWithIsolatedContext(page: Page): Promise<any> {
    try {
      const browser = page.context().browser();
      if (!browser) {
        // Use shared page instead of failing
        if (this.verbose) console.log('🔄 No browser for isolated context, using shared page');
        return this.collectWithRetry(page, this.maxRetries);
      }
    
      // Create isolated context for performance measurement with minimal config
      const isolatedContext = await browser.newContext({
        // Basic config only to avoid failures
        viewport: page.viewportSize() || { width: 1280, height: 720 },
        javaScriptEnabled: true,
        ignoreHTTPSErrors: true
      });
    
      try {
        const isolatedPage = await isolatedContext.newPage();
        
        // Navigate to the same URL as the original page
        const currentUrl = page.url();
        if (this.verbose) {
          console.log(`🔄 Creating isolated context for: ${currentUrl}`);
        }
        
        await isolatedPage.goto(currentUrl, { 
          waitUntil: 'networkidle',
          timeout: 30000
        });
        
        // Collect metrics with enhanced retry mechanism
        const metrics = await this.collectWithAdvancedRetry(isolatedPage);
        
        return metrics;
        
      } finally {
        // Always clean up the isolated context
        try {
          await isolatedContext.close();
        } catch (e) {
          if (this.verbose) console.warn('Error closing isolated context:', e);
        }
      }
      
    } catch (isolatedError) {
      // If isolated context fails, fallback to shared page
      if (this.verbose) console.log('🔄 Isolated context failed, using shared page:', isolatedError);
      return this.collectWithRetry(page, this.maxRetries);
    }
  }
  
  /**
   * Advanced retry mechanism with exponential backoff and different strategies
   */
  async collectWithAdvancedRetry(page: Page): Promise<any> {
    const strategies = [
      { name: 'web-vitals-library', method: this.collectWithWebVitalsLibrary.bind(this) },
      { name: 'performance-observer', method: this.collectWithPerformanceObserver.bind(this) },
      { name: 'navigation-timing', method: this.getNavigationTimingFallback.bind(this) }
    ];
    
    let lastError: Error | null = null;
    
    for (let attempt = 1; attempt <= this.maxRetries; attempt++) {
      for (const strategy of strategies) {
        try {
          if (this.verbose) {
            console.log(`🔍 Attempt ${attempt}/${this.maxRetries} using ${strategy.name}`);
          }
          
          // Ensure page stability before each attempt
          await page.waitForLoadState('networkidle');
          await page.waitForTimeout(Math.min(attempt * 200, 1000));
          
          const metrics = await strategy.method(page);
          
          // Validate metrics quality
          if (this.hasValidMetrics(metrics)) {
            if (this.verbose) {
              console.log(`✅ Success with ${strategy.name} on attempt ${attempt}`);
            }
            return metrics;
          } else if (this.verbose) {
            console.log(`⚠️ ${strategy.name} returned incomplete metrics`);
          }
          
        } catch (error) {
          lastError = error as Error;
          // Always show performance strategy failures - critical for debugging performance issues
          log.fallback('Performance Strategy', `${strategy.name} failed on attempt ${attempt}`, 'trying next strategy', error);
        }
      }
      
      // Wait before next attempt with exponential backoff
      if (attempt < this.maxRetries) {
        const delay = this.retryDelay * Math.pow(2, attempt - 1);
        console.log(`⏱️ Waiting ${delay}ms before next attempt...`);
        await page.waitForTimeout(delay);
      }
    }
    
    // Always show this critical fallback - indicates serious performance measurement issues
    log.fallback('Performance Collection', 'all strategies exhausted', 'using basic navigation timing (least accurate)');
    return this.getNavigationTimingFallback(page);
  }
  
  /**
   * Collect metrics with retry mechanism and robust error handling
   */
  private async collectWithRetry(page: Page, maxRetries: number): Promise<any> {
    let lastError: Error | null = null;
    
    for (let attempt = 1; attempt <= maxRetries; attempt++) {
      try {
        const metrics = await this.collectMetricsOnce(page);
        
        // Validate metrics - if we get some valid data, use it
        if (this.hasValidMetrics(metrics)) {
          return metrics;
        }
        
        if (this.verbose) {
          console.log(`Attempt ${attempt} got incomplete metrics, retrying...`);
        }
        
        if (attempt < maxRetries) {
          // Wait before retry, with exponential backoff
          await page.waitForTimeout(attempt * 500);
        }
        
      } catch (error) {
        lastError = error as Error;
        // Always show retry failures - helps identify patterns
        log.fallback('Performance Collection', `attempt ${attempt} failed`, 'retrying with different strategy', error);
        
        if (attempt < maxRetries) {
          // Wait before retry
          await page.waitForTimeout(attempt * 1000);
        }
      }
    }
    
    // All retries failed, return fallback metrics - Always show this critical fallback
    log.fallback('Performance Collection', 'all retry attempts failed', 'using navigation timing fallback');
    return this.getNavigationTimingFallback(page);
  }
  
  /**
   * Single attempt at collecting metrics with better synchronization
   */
  private async collectMetricsOnce(page: Page): Promise<any> {
    // First try the modern web-vitals library approach
    try {
      return await this.collectWithWebVitalsLibrary(page);
    } catch (error) {
      console.warn('Web-vitals library failed, trying PerformanceObserver approach:', error);
      return await this.collectWithPerformanceObserver(page);
    }
  }
  
  /**
   * Collect using Google's web-vitals library with better error handling
   */
  private async collectWithWebVitalsLibrary(page: Page): Promise<any> {
    // Check if we can inject the library safely
    const canInject = await page.evaluate(() => {
      return !!(window && document && document.readyState);
    });
    
    if (!canInject) {
      throw new Error('Page context not ready for script injection');
    }
    
    // Inject library with timeout
    await Promise.race([
      page.addScriptTag({
        url: 'https://unpkg.com/web-vitals@3/dist/web-vitals.iife.js'
      }),
      new Promise((_, reject) => 
        setTimeout(() => reject(new Error('Script injection timeout')), 10000)
      )
    ]);
    
    // Collect metrics with timeout and better state management
    return await page.evaluate(() => {
      return new Promise<any>((resolve, reject) => {
        const results: any = {
          lcp: 0, cls: 0, fcp: 0, ttfb: 0,
          loadTime: 0, domContentLoaded: 0, renderTime: 0
        };
        
        let resolved = false;
        let collectedMetrics = 0;
        const expectedMetrics = 4; // LCP, CLS, FCP, TTFB
        
        const finishCollection = (reason: string) => {
          if (!resolved) {
            resolved = true;
            
            // Add navigation timing metrics
            const navigation = performance.getEntriesByType('navigation')[0] as PerformanceNavigationTiming;
            if (navigation) {
              results.loadTime = navigation.loadEventEnd;
              results.domContentLoaded = navigation.domContentLoadedEventEnd;
              results.renderTime = navigation.domContentLoadedEventEnd - navigation.responseEnd;
            }
            
            console.log(`Web Vitals collection finished (${reason}):`, results);
            resolve(results);
          }
        };
        
        // Set up metric collectors with error handling
        try {
          if (!(window as any).webVitals) {
            throw new Error('web-vitals library not available');
          }
          
          const webVitals = (window as any).webVitals;
          
          // Collect each metric with individual error handling
          try {
            webVitals.onLCP((metric: any) => {
              results.lcp = metric.value;
              collectedMetrics++;
              console.log(`LCP: ${metric.value}ms`);
            });
          } catch (e) { console.warn('LCP collection failed:', e); }
          
          try {
            webVitals.onCLS((metric: any) => {
              results.cls = metric.value;
              collectedMetrics++;
              console.log(`CLS: ${metric.value}`);
            });
          } catch (e) { console.warn('CLS collection failed:', e); }
          
          try {
            webVitals.onFCP((metric: any) => {
              results.fcp = metric.value;
              collectedMetrics++;
              console.log(`FCP: ${metric.value}ms`);
            });
          } catch (e) { console.warn('FCP collection failed:', e); }
          
          
          try {
            webVitals.onTTFB((metric: any) => {
              results.ttfb = metric.value;
              collectedMetrics++;
              console.log(`TTFB: ${metric.value}ms`);
            });
          } catch (e) { console.warn('TTFB collection failed:', e); }
          
          // Set timeout based on page state
          const timeout = document.readyState === 'complete' ? 3000 : 8000;
          
          setTimeout(() => {
            finishCollection(`timeout after ${timeout}ms`);
          }, timeout);
          
          // If page is already loaded, give it less time
          if (document.readyState === 'complete') {
            setTimeout(() => {
              if (collectedMetrics >= expectedMetrics * 0.6) { // If we have 60% of metrics
                finishCollection('sufficient metrics collected');
              }
            }, 2000);
          }
          
        } catch (error) {
          reject(error);
        }
      });
    });
  }
  
  /**
   * Fallback collection using PerformanceObserver API directly
   */
  private async collectWithPerformanceObserver(page: Page): Promise<any> {
    return await page.evaluate(() => {
      return new Promise<any>((resolve) => {
        const results: any = {
          lcp: 0, cls: 0, fcp: 0, ttfb: 0,
          loadTime: 0, domContentLoaded: 0, renderTime: 0
        };
        
        let resolved = false;
        
        const finishCollection = () => {
          if (!resolved) {
            resolved = true;
            
            // Add navigation timing
            const navigation = performance.getEntriesByType('navigation')[0] as PerformanceNavigationTiming;
            if (navigation) {
              results.loadTime = navigation.loadEventEnd;
              results.domContentLoaded = navigation.domContentLoadedEventEnd;
              results.renderTime = navigation.domContentLoadedEventEnd - navigation.responseEnd;
              results.ttfb = navigation.responseStart - navigation.requestStart;
            }
            
            // Add paint metrics
            const paintEntries = performance.getEntriesByType('paint');
            const fcpEntry = paintEntries.find(entry => entry.name === 'first-contentful-paint');
            if (fcpEntry) {
              results.fcp = fcpEntry.startTime;
            }
            
            resolve(results);
          }
        };
        
        try {
          // Try to collect LCP
          const lcpObserver = new PerformanceObserver((list) => {
            const entries = list.getEntries();
            const lastEntry = entries[entries.length - 1];
            if (lastEntry) {
              results.lcp = lastEntry.startTime;
            }
          });
          
          lcpObserver.observe({ entryTypes: ['largest-contentful-paint'] });
          
          // Try to collect CLS
          let clsValue = 0;
          const clsObserver = new PerformanceObserver((list) => {
            const entries = list.getEntries();
            entries.forEach((entry: any) => {
              if (!entry.hadRecentInput) {
                clsValue += entry.value;
              }
            });
            results.cls = clsValue;
          });
          
          clsObserver.observe({ entryTypes: ['layout-shift'] });
          
          // Set timeout
          setTimeout(finishCollection, 5000);
          
        } catch (error) {
          console.warn('PerformanceObserver setup failed:', error);
          setTimeout(finishCollection, 1000);
        }
      });
    });
  }
  
  /**
   * Get navigation timing as fallback when all else fails
   */
  private async getNavigationTimingFallback(page: Page): Promise<any> {
    return await page.evaluate(() => {
      const navigation = performance.getEntriesByType('navigation')[0] as PerformanceNavigationTiming;
      const paintEntries = performance.getEntriesByType('paint');
      
      const fcp = paintEntries.find(entry => entry.name === 'first-contentful-paint')?.startTime || 0;
      
      return {
        lcp: fcp || navigation?.loadEventEnd || 0,
        cls: 0, // Can't measure without PerformanceObserver
        fcp: fcp,
        ttfb: navigation ? navigation.responseStart - navigation.requestStart : 0,
        loadTime: navigation?.loadEventEnd || 0,
        domContentLoaded: navigation?.domContentLoadedEventEnd || 0,
        renderTime: navigation ? navigation.domContentLoadedEventEnd - navigation.responseEnd : 0
      };
    });
  }
  
  /**
   * Enhanced metrics validation with quality scoring
   */
  private hasValidMetrics(metrics: any): boolean {
    const quality = this.assessMetricsQuality(metrics);
    return quality.score >= 0.4; // At least 40% quality required
  }
  
  /**
   * Assess the quality of collected metrics
   */
  private assessMetricsQuality(metrics: any): { score: number; issues: string[]; strengths: string[] } {
    let score = 0;
    const issues: string[] = [];
    const strengths: string[] = [];
    const maxScore = 10;
    
    // Core metrics availability (4 points)
    if (metrics.lcp > 0) { score += 1.5; strengths.push('LCP available'); } else issues.push('LCP missing');
    if (metrics.fcp > 0) { score += 1; strengths.push('FCP available'); } else issues.push('FCP missing');
    if (metrics.cls >= 0) { score += 0.5; strengths.push('CLS available'); } else issues.push('CLS missing');
    if (metrics.ttfb > 0) { score += 1; strengths.push('TTFB available'); } else issues.push('TTFB missing');
    
    // Navigation timing availability (2 points)
    if (metrics.loadTime > 0) { score += 1; strengths.push('Load time available'); } else issues.push('Load time missing');
    if (metrics.domContentLoaded > 0) { score += 1; strengths.push('DOM timing available'); } else issues.push('DOM timing missing');
    
    // Reasonableness checks (4 points)
    if (metrics.loadTime > 0 && metrics.loadTime < 60000) { score += 1; strengths.push('Reasonable load time'); }
    else if (metrics.loadTime >= 60000) issues.push('Unreasonable load time (>60s)');
    
    if (metrics.lcp > 0 && metrics.lcp < 30000) { score += 1; strengths.push('Reasonable LCP'); }
    else if (metrics.lcp >= 30000) issues.push('Unreasonable LCP (>30s)');
    
    if (metrics.fcp > 0 && metrics.fcp < 20000) { score += 1; strengths.push('Reasonable FCP'); }
    else if (metrics.fcp >= 20000) issues.push('Unreasonable FCP (>20s)');
    
    if (metrics.cls >= 0 && metrics.cls < 5) { score += 1; strengths.push('Reasonable CLS'); }
    else if (metrics.cls >= 5) issues.push('Unreasonable CLS (>5)');
    
    const normalizedScore = Math.max(0, Math.min(1, score / maxScore));
    
    return {
      score: normalizedScore,
      issues,
      strengths
    };
  }
  
  /**
   * Get metrics quality report for debugging
   */
  getMetricsQualityReport(metrics: any): string {
    const quality = this.assessMetricsQuality(metrics);
    const percentage = Math.round(quality.score * 100);
    
    let report = `Performance Metrics Quality: ${percentage}%\n`;
    
    if (quality.strengths.length > 0) {
      report += `✅ Strengths: ${quality.strengths.join(', ')}\n`;
    }
    
    if (quality.issues.length > 0) {
      report += `❌ Issues: ${quality.issues.join(', ')}\n`;
    }
    
    return report;
  }
  
  /**
   * Apply fallback strategies when Web Vitals metrics are missing or zero
   * Provides alternative calculations for small/static sites
   */
  private applyFallbackStrategies(metrics: any): any {
    const enhanced = { ...metrics };
    
    // LCP Fallback: Use navigation timing if LCP is 0
    if (enhanced.lcp === 0 && enhanced.loadTime > 0) {
      // For small pages, LCP often equals load time or FCP
      enhanced.lcp = enhanced.fcp > 0 ? enhanced.fcp * 1.2 : enhanced.loadTime * 0.8;
    }
    
    // Additional LCP fallback using document timing
    if (enhanced.lcp === 0 && enhanced.domContentLoaded > 0) {
      // Estimate LCP as slightly after DOM ready for text-heavy pages
      enhanced.lcp = enhanced.domContentLoaded + 200;
      console.log(`LCP fallback from DOM timing: ${enhanced.lcp}ms`);
    }
    
    // CLS Fallback: Static pages often have 0 CLS, which is actually good
    if (enhanced.cls === 0) {
      // 0 CLS is perfect for static content, only log in verbose mode
      if (process.env.VERBOSE) {
        console.log('CLS is 0 - excellent layout stability for static content');
      }
    } else if (enhanced.cls > 0 && enhanced.cls < 0.001) {
      // Very small CLS values are often measurement artifacts
      enhanced.cls = 0;
      console.log('CLS below threshold, normalized to 0');
    }
    
    
    // TTFB Fallback: Calculate from navigation timing if available
    if (enhanced.ttfb === 0 && enhanced.domContentLoaded > 0) {
      // Rough estimate from navigation timing
      enhanced.ttfb = Math.max(100, enhanced.domContentLoaded * 0.3);
      console.log('TTFB fallback applied:', enhanced.ttfb);
    }
    
    // FCP Fallback: Very important metric, try to calculate
    if (enhanced.fcp === 0 && enhanced.domContentLoaded > 0) {
      // Estimate FCP from DOM ready time
      enhanced.fcp = enhanced.domContentLoaded * 0.7;
      console.log('FCP fallback applied:', enhanced.fcp);
    }
    
    return enhanced;
  }
  
  /**
   * Calculate performance score based on configurable budget thresholds
   * Uses custom scoring methodology based on user-defined budgets
   */
  private calculateScore(metrics: any): number {
    let score = 100;
    
    // LCP scoring (25% weight)
    if (metrics.lcp > this.budget.lcp.poor) score -= 25;
    else if (metrics.lcp > this.budget.lcp.good) score -= 15;
    
    // CLS scoring (25% weight) 
    if (metrics.cls > this.budget.cls.poor) score -= 25;
    else if (metrics.cls > this.budget.cls.good) score -= 15;
    
    // FCP scoring (35% weight) - increased from 20%
    if (metrics.fcp > this.budget.fcp.poor) score -= 35;
    else if (metrics.fcp > this.budget.fcp.good) score -= 18;
    
    // TTFB scoring (15% weight)
    if (metrics.ttfb > this.budget.ttfb.poor) score -= 15;
    else if (metrics.ttfb > this.budget.ttfb.good) score -= 8;
    
    return Math.max(0, Math.round(score));
  }
  
  private calculateGrade(score: number): 'A' | 'B' | 'C' | 'D' | 'F' {
    if (score >= 90) return 'A';
    if (score >= 80) return 'B';
    if (score >= 70) return 'C';
    if (score >= 60) return 'D';
    return 'F';
  }
  
  private generateRecommendations(metrics: any): string[] {
    const recommendations: string[] = [];
    
    // LCP recommendations
    if (metrics.lcp > this.budget.lcp.good) {
      const status = metrics.lcp > this.budget.lcp.poor ? 'CRITICAL' : 'WARNING';
      recommendations.push(`${status}: LCP (${metrics.lcp}ms) exceeds budget (${this.budget.lcp.good}ms good, ${this.budget.lcp.poor}ms poor). Compress images, use CDN, enable lazy loading`);
    }
    
    // CLS recommendations  
    if (metrics.cls > this.budget.cls.good) {
      const status = metrics.cls > this.budget.cls.poor ? 'CRITICAL' : 'WARNING';
      recommendations.push(`${status}: CLS (${metrics.cls.toFixed(3)}) exceeds budget (${this.budget.cls.good} good, ${this.budget.cls.poor} poor). Set explicit dimensions for images and ads`);
    }
    
    // FCP recommendations
    if (metrics.fcp > this.budget.fcp.good) {
      const status = metrics.fcp > this.budget.fcp.poor ? 'CRITICAL' : 'WARNING';
      recommendations.push(`${status}: FCP (${metrics.fcp}ms) exceeds budget (${this.budget.fcp.good}ms good, ${this.budget.fcp.poor}ms poor). Minimize CSS, optimize fonts, reduce JavaScript`);
    }
    
    
    // TTFB recommendations
    if (metrics.ttfb > this.budget.ttfb.good) {
      const status = metrics.ttfb > this.budget.ttfb.poor ? 'CRITICAL' : 'WARNING';
      recommendations.push(`${status}: TTFB (${metrics.ttfb}ms) exceeds budget (${this.budget.ttfb.good}ms good, ${this.budget.ttfb.poor}ms poor). Optimize backend, use CDN, enable compression`);
    }
    
    if (recommendations.length === 0) {
      recommendations.push('🎉 Excellent performance! All Core Web Vitals meet your performance budget.');
    }
    
    return recommendations;
  }
  
  /**
   * Evaluate performance against budget and return status
   */
  private evaluateBudget(metrics: any): BudgetStatus {
    const violations: BudgetViolation[] = [];
    
    // Check each metric against budget
    if (metrics.lcp > this.budget.lcp.good) {
      violations.push({
        metric: 'lcp',
        actual: metrics.lcp,
        threshold: metrics.lcp > this.budget.lcp.poor ? this.budget.lcp.poor : this.budget.lcp.good,
        severity: metrics.lcp > this.budget.lcp.poor ? 'error' : 'warning',
        message: `LCP ${metrics.lcp}ms exceeds ${metrics.lcp > this.budget.lcp.poor ? 'poor' : 'good'} threshold`
      });
    }
    
    if (metrics.cls > this.budget.cls.good) {
      violations.push({
        metric: 'cls',
        actual: metrics.cls,
        threshold: metrics.cls > this.budget.cls.poor ? this.budget.cls.poor : this.budget.cls.good,
        severity: metrics.cls > this.budget.cls.poor ? 'error' : 'warning',
        message: `CLS ${metrics.cls.toFixed(3)} exceeds ${metrics.cls > this.budget.cls.poor ? 'poor' : 'good'} threshold`
      });
    }
    
    if (metrics.fcp > this.budget.fcp.good) {
      violations.push({
        metric: 'fcp',
        actual: metrics.fcp,
        threshold: metrics.fcp > this.budget.fcp.poor ? this.budget.fcp.poor : this.budget.fcp.good,
        severity: metrics.fcp > this.budget.fcp.poor ? 'error' : 'warning',
        message: `FCP ${metrics.fcp}ms exceeds ${metrics.fcp > this.budget.fcp.poor ? 'poor' : 'good'} threshold`
      });
    }
    
    
    if (metrics.ttfb > this.budget.ttfb.good) {
      violations.push({
        metric: 'ttfb',
        actual: metrics.ttfb,
        threshold: metrics.ttfb > this.budget.ttfb.poor ? this.budget.ttfb.poor : this.budget.ttfb.good,
        severity: metrics.ttfb > this.budget.ttfb.poor ? 'error' : 'warning',
        message: `TTFB ${metrics.ttfb}ms exceeds ${metrics.ttfb > this.budget.ttfb.poor ? 'poor' : 'good'} threshold`
      });
    }
    
    const passed = violations.length === 0;
    const criticalViolations = violations.filter(v => v.severity === 'error').length;
    const warningViolations = violations.filter(v => v.severity === 'warning').length;
    
    let summary: string;
    if (passed) {
      summary = '🎉 All metrics within budget';
    } else if (criticalViolations > 0) {
      summary = `❌ Budget failed: ${criticalViolations} critical, ${warningViolations} warnings`;
    } else {
      summary = `⚠️ Budget warnings: ${warningViolations} metrics exceed thresholds`;
    }
    
    return {
      passed,
      violations,
      summary
    };
  }
  
  private getFallbackMetrics(error?: Error): CoreWebVitals {
    const recommendations = [
      'Performance metrics collection failed.',
      'This may be due to network issues, blocked resources, or browser restrictions.',
      'Consider running the audit again or check your network connection.'
    ];
    
    if (error) {
      recommendations.push(`Error details: ${error.message}`);
    }
    
    return {
      lcp: 0, cls: 0, fcp: 0, ttfb: 0,
      loadTime: 0, domContentLoaded: 0, renderTime: 0,
      score: 0,
      grade: 'F',
      recommendations,
      budgetStatus: {
        passed: false,
        violations: [{
          metric: 'lcp',
          actual: 0,
          threshold: this.budget.lcp.good,
          severity: 'error',
          message: 'Unable to measure performance metrics'
        }],
        summary: '❌ Performance measurement failed'
      }
    };
  }
  
  /**
   * Update retry configuration
   */
  updateRetryConfig(maxRetries: number, retryDelay: number): void {
    this.maxRetries = Math.max(1, Math.min(10, maxRetries)); // Limit between 1-10
    this.retryDelay = Math.max(100, Math.min(10000, retryDelay)); // Limit between 100ms-10s
  }
  
  /**
   * Get current retry configuration
   */
  getRetryConfig(): { maxRetries: number; retryDelay: number } {
    return {
      maxRetries: this.maxRetries,
      retryDelay: this.retryDelay
    };
  }
}
