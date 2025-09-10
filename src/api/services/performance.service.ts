import { PerformanceResult, PerformanceIssue } from '../../types/audit-results';

/**
 * PerformanceService - Returns PerformanceResult (same type used in PageAuditResult)
 * Used by API endpoint: POST /api/v2/page/performance
 */
export class PerformanceService {
  private poolManager: any;
  
  constructor(testMode = false) {
    if (!testMode) {
      this.initializePool();
    }
  }
  
  private async initializePool() {
    const { BrowserPoolManager } = require('../../core/browser/browser-pool-manager');
    this.poolManager = new BrowserPoolManager({
      maxConcurrent: 2, // Conservative for API service
      maxIdleTime: 30000,
      browserType: 'chromium',
      enableResourceOptimization: true,
      launchOptions: {
        headless: true,
        args: [
          '--no-sandbox',
          '--disable-setuid-sandbox',
          '--disable-dev-shm-usage',
          '--disable-gpu',
          '--memory-pressure-off'
        ]
      }
    });
    
    await this.poolManager.warmUp(1);
  }
  
  async analyzeUrl(url: string, options: {
    timeout?: number;
  } = {}): Promise<PerformanceResult> {
    if (!this.poolManager) {
      await this.initializePool();
    }
    
    try {
      // Use pooled browser for performance analysis
      const { PooledAccessibilityChecker } = require('../../core/accessibility/pooled-accessibility-checker');
      const checker = new PooledAccessibilityChecker(this.poolManager);
      
      const results = await checker.testMultiplePages([url], {
        enhancedPerformanceAnalysis: true,
        timeout: options.timeout || 10000,
        maxPages: 1
      });
      
      if (results[0]?.performanceMetrics) {
        const metrics = results[0].performanceMetrics;
        const score = this.calculatePerformanceScore(metrics);
        
        return {
          score,
          grade: this.scoreToGrade(score),
          coreWebVitals: {
            largestContentfulPaint: metrics.largestContentfulPaint || 0,
            firstContentfulPaint: metrics.firstContentfulPaint || 0,
            cumulativeLayoutShift: metrics.cumulativeLayoutShift || 0,
            timeToFirstByte: metrics.timeToFirstByte || 0
          },
          metrics: {
            domContentLoaded: metrics.domContentLoaded || 0,
            loadComplete: metrics.loadTime || 0,
            firstPaint: metrics.firstPaint
          },
          issues: this.identifyPerformanceIssues(metrics)
        };
      }
    } catch (error) {
      console.warn('Performance analysis failed:', (error as Error).message);
    }
    
    // Fallback: Return minimal performance data
    return {
      score: 0,
      grade: 'F',
      coreWebVitals: {
        largestContentfulPaint: 0,
        firstContentfulPaint: 0,
        cumulativeLayoutShift: 0,
        timeToFirstByte: 0
      },
      metrics: {
        domContentLoaded: 0,
        loadComplete: 0
      },
      issues: [{
        type: 'lcp-slow',
        message: 'Performance analysis failed - unable to collect metrics',
        severity: 'error',
        value: 0,
        threshold: 0
      }]
    };
  }
  
  private calculatePerformanceScore(metrics: any): number {
    let score = 100;
    
    // Penalize slow LCP (target: 2.5s)
    if (metrics.largestContentfulPaint > 4000) score -= 30;
    else if (metrics.largestContentfulPaint > 2500) score -= 15;
    
    // Penalize slow FCP (target: 1.8s)
    if (metrics.firstContentfulPaint > 3000) score -= 20;
    else if (metrics.firstContentfulPaint > 1800) score -= 10;
    
    // Penalize high CLS (target: 0.1)
    if (metrics.cumulativeLayoutShift > 0.25) score -= 25;
    else if (metrics.cumulativeLayoutShift > 0.1) score -= 10;
    
    // Penalize slow TTFB (target: 600ms)
    if (metrics.timeToFirstByte > 1500) score -= 15;
    else if (metrics.timeToFirstByte > 600) score -= 8;
    
    return Math.max(0, Math.round(score));
  }
  
  private scoreToGrade(score: number): 'A' | 'B' | 'C' | 'D' | 'F' {
    if (score >= 90) return 'A';
    if (score >= 80) return 'B';
    if (score >= 70) return 'C';
    if (score >= 60) return 'D';
    return 'F';
  }
  
  private identifyPerformanceIssues(metrics: any): PerformanceIssue[] {
    const issues: PerformanceIssue[] = [];
    
    if (metrics.largestContentfulPaint > 4000) {
      issues.push({
        type: 'lcp-slow',
        message: `Largest Contentful Paint is very slow (${Math.round(metrics.largestContentfulPaint)}ms). Target: <2.5s`,
        severity: 'error',
        value: metrics.largestContentfulPaint,
        threshold: 2500
      });
    }
    
    if (metrics.cumulativeLayoutShift > 0.25) {
      issues.push({
        type: 'cls-high',
        message: `High Cumulative Layout Shift (${metrics.cumulativeLayoutShift.toFixed(3)}). Target: <0.1`,
        severity: 'error',
        value: metrics.cumulativeLayoutShift,
        threshold: 0.1
      });
    }
    
    if (metrics.timeToFirstByte > 1500) {
      issues.push({
        type: 'ttfb-slow',
        message: `Slow server response time (${Math.round(metrics.timeToFirstByte)}ms). Target: <600ms`,
        severity: 'warning',
        value: metrics.timeToFirstByte,
        threshold: 600
      });
    }
    
    if (metrics.firstContentfulPaint > 3000) {
      issues.push({
        type: 'fcp-slow',
        message: `First Contentful Paint is slow (${Math.round(metrics.firstContentfulPaint)}ms). Target: <1.8s`,
        severity: 'warning',
        value: metrics.firstContentfulPaint,
        threshold: 1800
      });
    }
    
    return issues;
  }
  
  async cleanup() {
    if (this.poolManager) {
      await this.poolManager.shutdown();
    }
  }
}
