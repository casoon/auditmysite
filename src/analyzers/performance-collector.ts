/**
 * ⚡ Enhanced Performance Metrics Collector
 * 
 * Collects comprehensive performance metrics including:
 * - Core Web Vitals (LCP, INP, CLS)
 * - Advanced timing metrics (TTFB, FID, TBT)
 * - Resource timing analysis
 * - Network performance analysis
 * - Performance scoring and grading
 */

import { Page } from 'playwright';
import { 
  PerformanceMetrics,
  ContentWeight, 
  ContentAnalysis, 
  ResourceTiming,
  QualityAnalysisOptions 
} from '../types/enhanced-metrics';
import { ContentWeightAnalyzer } from './content-weight-analyzer';

export class PerformanceCollector {
  private contentAnalyzer: ContentWeightAnalyzer;

  constructor(private options: QualityAnalysisOptions = {}) {
    this.contentAnalyzer = new ContentWeightAnalyzer();
  }

  /**
   * Collect comprehensive performance metrics for a webpage
   */
  async collectEnhancedMetrics(page: Page, url: string | { loc: string }): Promise<PerformanceMetrics> {
    // Extract URL string from URL object if needed
    const urlString = (typeof url === 'object' && url.loc ? url.loc : url) as string;
    console.log(`⚡ Collecting enhanced performance metrics for: ${urlString}`);
    
    const startTime = Date.now();

    try {
      // Navigate to the page with performance timing (only if page is not already loaded)
      const currentUrl = page.url();
      const isDataUri = currentUrl.startsWith('data:');
      const isContentSet = currentUrl !== 'about:blank' && currentUrl !== '';
      
      // Only navigate if we don't already have content set
      if (!isContentSet && !isDataUri) {
        await page.goto(urlString, { 
          waitUntil: 'networkidle',
          timeout: this.options.analysisTimeout || 30000 
        });
      } else {
        console.log(`📄 Using pre-set page content for performance analysis (${currentUrl})`);
      }

      // Wait for potential lazy loading and interactions
      await page.waitForTimeout(3000);

      // Collect all performance metrics in parallel
      const [
        coreWebVitals,
        timingMetrics,
        { contentWeight, contentAnalysis, resourceTimings }
      ] = await Promise.all([
        this.collectCoreWebVitals(page),
        this.collectTimingMetrics(page),
        this.contentAnalyzer.analyze(page, urlString)
      ]);

      // Calculate derived metrics
      const performanceScore = this.calculatePerformanceScore({
        ...coreWebVitals,
        ...timingMetrics,
        contentWeight
      });

      const performanceGrade = this.calculatePerformanceGrade(performanceScore);
      const recommendations = this.generatePerformanceRecommendations(
        { ...coreWebVitals, ...timingMetrics }, 
        contentWeight, 
        contentAnalysis
      );

      const enhancedMetrics: PerformanceMetrics = {
        // Core Web Vitals
        lcp: coreWebVitals.lcp,
        inp: coreWebVitals.inp,
        cls: coreWebVitals.cls,
        
        // Additional Performance Metrics
        ttfb: timingMetrics.ttfb,
        fid: coreWebVitals.fid,
        tbt: timingMetrics.tbt,
        speedIndex: timingMetrics.speedIndex,
        
        // Timing Metrics
        domContentLoaded: timingMetrics.domContentLoaded,
        loadComplete: timingMetrics.loadComplete,
        firstPaint: timingMetrics.firstPaint,
        firstContentfulPaint: timingMetrics.firstContentfulPaint,
        
        // Network Analysis
        requestCount: resourceTimings.length,
        transferSize: contentWeight.gzipTotal || contentWeight.total,
        resourceLoadTimes: resourceTimings,
        
        // Performance Scores
        performanceScore,
        performanceGrade,
        recommendations,
        
        // Content Analysis
        contentWeight,
        contentAnalysis
      };

      console.log(`✅ Enhanced performance metrics collected in ${Date.now() - startTime}ms`);
      console.log(`📊 Performance Score: ${performanceScore}/100 (Grade: ${performanceGrade})`);
      console.log(`⚡ LCP: ${coreWebVitals.lcp}ms, CLS: ${coreWebVitals.cls}, INP: ${coreWebVitals.inp}ms`);

      return enhancedMetrics;

    } catch (error) {
      console.error('❌ Enhanced performance metrics collection failed:', error);
      throw new Error(`Enhanced performance metrics collection failed: ${error}`);
    }
  }

  /**
   * Collect Core Web Vitals metrics
   */
  private async collectCoreWebVitals(page: Page): Promise<{
    lcp: number;
    inp: number;
    cls: number;
    fid: number;
  }> {
    // Inject Web Vitals measurement script
    await page.addScriptTag({
      content: `
        window.webVitalsData = {
          lcp: 0,
          inp: 0,
          cls: 0,
          fid: 0
        };

        // LCP Observer
        if (typeof PerformanceObserver !== 'undefined') {
          try {
            const lcpObserver = new PerformanceObserver((list) => {
              const entries = list.getEntries();
              const lastEntry = entries[entries.length - 1];
              window.webVitalsData.lcp = Math.round(lastEntry.startTime);
            });
            lcpObserver.observe({ type: 'largest-contentful-paint', buffered: true });

            // CLS Observer
            let clsValue = 0;
            const clsObserver = new PerformanceObserver((list) => {
              for (const entry of list.getEntries()) {
                if (!entry.hadRecentInput) {
                  clsValue += entry.value;
                }
              }
              window.webVitalsData.cls = Math.round(clsValue * 1000) / 1000;
            });
            clsObserver.observe({ type: 'layout-shift', buffered: true });

            // FID Observer (for actual user input)
            const fidObserver = new PerformanceObserver((list) => {
              for (const entry of list.getEntries()) {
                window.webVitalsData.fid = Math.round(entry.processingStart - entry.startTime);
              }
            });
            fidObserver.observe({ type: 'first-input', buffered: true });

          } catch (e) {
            console.warn('Web Vitals observation failed:', e);
          }
        }
      `
    });

    // Simulate some user interactions for INP measurement
    await page.mouse.move(100, 100);
    await page.keyboard.press('Tab');
    await page.waitForTimeout(1000);

    // Measure INP through performance timeline
    const inp = await page.evaluate(() => {
      if (typeof PerformanceObserver !== 'undefined') {
        const interactionEntries = performance.getEntriesByType('event');
        if (interactionEntries.length > 0) {
          const maxDuration = Math.max(...interactionEntries.map((entry: any) => entry.duration || 0));
          return Math.round(maxDuration);
        }
      }
      return 0;
    });

    // Get the collected Web Vitals data
    const webVitals = await page.evaluate(() => (window as any).webVitalsData || {
      lcp: 0,
      inp: 0,
      cls: 0,
      fid: 0
    });

    // Fallback measurements if Web Vitals API is not available
    if (webVitals.lcp === 0) {
      webVitals.lcp = await this.fallbackLCPMeasurement(page);
    }

    return {
      lcp: webVitals.lcp,
      inp: inp || webVitals.inp,
      cls: webVitals.cls,
      fid: webVitals.fid
    };
  }

  /**
   * Collect additional timing metrics
   */
  private async collectTimingMetrics(page: Page): Promise<{
    ttfb: number;
    tbt: number;
    speedIndex: number;
    domContentLoaded: number;
    loadComplete: number;
    firstPaint: number;
    firstContentfulPaint: number;
  }> {
    const timingData = await page.evaluate(() => {
      const navigation = performance.getEntriesByType('navigation')[0] as PerformanceNavigationTiming;
      const paintEntries = performance.getEntriesByType('paint');
      
      const firstPaint = paintEntries.find(entry => entry.name === 'first-paint');
      const firstContentfulPaint = paintEntries.find(entry => entry.name === 'first-contentful-paint');
      
      return {
        ttfb: navigation ? Math.round(navigation.responseStart - navigation.requestStart) : 0,
        domContentLoaded: navigation ? Math.round(navigation.domContentLoadedEventEnd - navigation.fetchStart) : 0,
        loadComplete: navigation ? Math.round(navigation.loadEventEnd - navigation.fetchStart) : 0,
        firstPaint: firstPaint ? Math.round(firstPaint.startTime) : 0,
        firstContentfulPaint: firstContentfulPaint ? Math.round(firstContentfulPaint.startTime) : 0,
        responseEnd: navigation ? navigation.responseEnd : 0,
        domInteractive: navigation ? navigation.domInteractive : 0
      };
    });

    // Calculate Total Blocking Time (TBT)
    const tbt = await this.calculateTotalBlockingTime(page);
    
    // Calculate Speed Index (simplified version)
    const speedIndex = await this.calculateSpeedIndex(page, timingData.firstContentfulPaint);

    return {
      ttfb: timingData.ttfb,
      tbt,
      speedIndex,
      domContentLoaded: timingData.domContentLoaded,
      loadComplete: timingData.loadComplete,
      firstPaint: timingData.firstPaint,
      firstContentfulPaint: timingData.firstContentfulPaint
    };
  }

  /**
   * Fallback LCP measurement using largest image/text element
   */
  private async fallbackLCPMeasurement(page: Page): Promise<number> {
    return page.evaluate(() => {
      const images = Array.from(document.querySelectorAll('img'));
      const textElements = Array.from(document.querySelectorAll('h1, h2, h3, p, div'));
      
      let largestElement = null;
      let largestSize = 0;
      
      // Check images
      images.forEach(img => {
        const rect = img.getBoundingClientRect();
        const size = rect.width * rect.height;
        if (size > largestSize && rect.top < window.innerHeight) {
          largestSize = size;
          largestElement = img;
        }
      });
      
      // Check text elements
      textElements.forEach(el => {
        const rect = el.getBoundingClientRect();
        const size = rect.width * rect.height;
        if (size > largestSize && rect.top < window.innerHeight) {
          largestSize = size;
          largestElement = el;
        }
      });
      
      // Estimate LCP based on when largest element would be rendered
      return largestElement ? Math.round(performance.now()) : 0;
    });
  }

  /**
   * Calculate Total Blocking Time
   */
  private async calculateTotalBlockingTime(page: Page): Promise<number> {
    return page.evaluate(() => {
      const longTasks = performance.getEntriesByType('longtask');
      let tbt = 0;
      
      longTasks.forEach((task: any) => {
        if (task.duration > 50) {
          tbt += task.duration - 50;
        }
      });
      
      return Math.round(tbt);
    });
  }

  /**
   * Calculate Speed Index (simplified)
   */
  private async calculateSpeedIndex(page: Page, fcp: number): Promise<number> {
    // Simplified Speed Index calculation
    // In a real implementation, you'd measure visual completeness over time
    const visualCompleteTime = await page.evaluate(() => {
      const images = document.querySelectorAll('img');
      let loadedImages = 0;
      images.forEach(img => {
        if (img.complete && img.naturalHeight !== 0) {
          loadedImages++;
        }
      });
      
      const completionRatio = images.length > 0 ? loadedImages / images.length : 1;
      return completionRatio >= 0.85 ? performance.now() : performance.now() + 1000;
    });

    // Simplified Speed Index formula
    return Math.round((fcp + visualCompleteTime) / 2);
  }

  /**
   * Calculate overall performance score
   */
  private calculatePerformanceScore(metrics: {
    lcp: number;
    inp: number;
    cls: number;
    ttfb: number;
    fid: number;
    tbt: number;
    firstContentfulPaint: number;
    contentWeight: ContentWeight;
  }): number {
    let score = 100;
    
    // Core Web Vitals scoring (70% of total score)
    // LCP scoring (25%)
    if (metrics.lcp > 4000) score -= 25;
    else if (metrics.lcp > 2500) score -= 15;
    else if (metrics.lcp <= 1200) score += 5;
    
    // INP scoring (25%)
    if (metrics.inp > 500) score -= 25;
    else if (metrics.inp > 200) score -= 15;
    else if (metrics.inp <= 100) score += 5;
    
    // CLS scoring (20%)
    if (metrics.cls > 0.25) score -= 20;
    else if (metrics.cls > 0.1) score -= 10;
    else if (metrics.cls <= 0.05) score += 5;
    
    // Additional metrics (30% of total score)
    // TTFB scoring (10%)
    if (metrics.ttfb > 800) score -= 10;
    else if (metrics.ttfb > 600) score -= 5;
    
    // FCP scoring (10%)
    if (metrics.firstContentfulPaint > 3000) score -= 10;
    else if (metrics.firstContentfulPaint > 1800) score -= 5;
    
    // Page size scoring (10%)
    const totalMB = metrics.contentWeight.total / (1024 * 1024);
    if (totalMB > 5) score -= 10;
    else if (totalMB > 3) score -= 5;
    else if (totalMB <= 1) score += 5;
    
    return Math.max(0, Math.min(100, score));
  }

  /**
   * Calculate performance grade from score
   */
  private calculatePerformanceGrade(score: number): 'A' | 'B' | 'C' | 'D' | 'F' {
    if (score >= 90) return 'A';
    if (score >= 80) return 'B';
    if (score >= 70) return 'C';
    if (score >= 60) return 'D';
    return 'F';
  }

  /**
   * Generate performance recommendations
   */
  private generatePerformanceRecommendations(
    metrics: any,
    contentWeight: ContentWeight,
    contentAnalysis: ContentAnalysis
  ): string[] {
    const recommendations: string[] = [];
    
    // Core Web Vitals recommendations
    if (metrics.lcp > 2500) {
      recommendations.push(`🎯 LCP is ${metrics.lcp}ms - optimize largest content element loading (target: <2.5s)`);
    }
    
    if (metrics.inp > 200) {
      recommendations.push(`⚡ INP is ${metrics.inp}ms - optimize JavaScript execution and reduce main thread blocking (target: <200ms)`);
    }
    
    if (metrics.cls > 0.1) {
      recommendations.push(`📐 CLS is ${metrics.cls} - reserve space for images and avoid layout shifts (target: <0.1)`);
    }
    
    // TTFB recommendations
    if (metrics.ttfb > 600) {
      recommendations.push(`🚀 TTFB is ${metrics.ttfb}ms - optimize server response time and use CDN (target: <600ms)`);
    }
    
    // Resource-specific recommendations
    if (contentWeight.images > 2 * 1024 * 1024) {
      recommendations.push(`🖼️ Image size is ${(contentWeight.images / (1024 * 1024)).toFixed(1)}MB - compress and optimize images`);
    }
    
    if (contentWeight.javascript > 1024 * 1024) {
      recommendations.push(`📜 JavaScript bundle is ${(contentWeight.javascript / (1024 * 1024)).toFixed(1)}MB - implement code splitting`);
    }
    
    if (contentAnalysis.domElements > 1500) {
      recommendations.push(`🏗️ DOM is complex with ${contentAnalysis.domElements} elements - simplify HTML structure`);
    }
    
    // Add content weight recommendations
    const contentRecommendations = ContentWeightAnalyzer.generateContentRecommendations(
      contentWeight, 
      contentAnalysis
    );
    recommendations.push(...contentRecommendations);
    
    return recommendations;
  }
}
