/**
 * Mobile Performance Collector
 * 
 * Collects performance metrics specifically for mobile devices using mobile viewport
 * and mobile-specific thresholds. This runs parallel to desktop performance collection.
 */

import { Page } from 'playwright';
import { QualityAnalysisOptions } from '../types/enhanced-metrics';

export interface MobilePerformanceMetrics {
  score: number;
  grade: 'A' | 'B' | 'C' | 'D' | 'F';
  coreWebVitals: {
    lcp: number;
    fcp: number;
    cls: number;
    ttfb: number;
  };
  metrics: {
    domContentLoaded: number;
    loadComplete: number;
    renderTime: number;
  };
  recommendations: string[];
  isMobileOptimized: boolean;
}

export class MobilePerformanceCollector {
  constructor(private options: QualityAnalysisOptions = {}) {}

  /**
   * Collect mobile performance metrics using mobile viewport
   */
  async collectMobileMetrics(page: Page, url: string | { loc: string }): Promise<MobilePerformanceMetrics> {
    const urlString = (typeof url === 'object' && url.loc ? url.loc : url) as string;
    const startTime = Date.now();

    try {
      if (this.options.verbose) {
        console.log(`📱 Starting mobile performance analysis for: ${urlString}`);
      }

      // Set mobile viewport
      await page.setViewportSize({ width: 375, height: 812 }); // iPhone 12 Pro size
      
      // Simulate mobile device
      await page.setExtraHTTPHeaders({
        'User-Agent': 'Mozilla/5.0 (iPhone; CPU iPhone OS 16_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.0 Mobile/15E148 Safari/604.1'
      });

      // Navigate to page (if not already loaded)
      const currentUrl = page.url();
      if (currentUrl === 'about:blank' || currentUrl === '') {
        await page.goto(urlString, { 
          waitUntil: 'networkidle',
          timeout: this.options.analysisTimeout || 30000 
        });
      }

      // Wait for mobile-specific loading patterns
      await page.waitForTimeout(2000); // Mobile networks are slower

      // Optionally emulate PSI profile
      let cdpSession: any = null;
      try {
        if (this.options.psiProfile) {
          cdpSession = await (page as any)._client?.() || await (page.context() as any).newCDPSession(page);
          await cdpSession.send('Network.enable');
          // Lighthouse Slow 4G Standard (mobile, default)
          // Matches PageSpeed Insights lab conditions for mobile testing
          const net = this.options.psiNetwork || { latencyMs: 400, downloadKbps: 400, uploadKbps: 400 };
          await cdpSession.send('Network.emulateNetworkConditions', {
            offline: false,
            latency: net.latencyMs,
            downloadThroughput: Math.floor((net.downloadKbps * 1024) / 8),
            uploadThroughput: Math.floor((net.uploadKbps * 1024) / 8),
            connectionType: 'cellular4g' // Changed from 3g to 4g (Slow 4G)
          });
          const cpuRate = this.options.psiCPUThrottlingRate || 4;
          await cdpSession.send('Emulation.setCPUThrottlingRate', { rate: cpuRate });
        }
      } catch (e) {
        console.warn('PSI profile emulation (mobile) failed:', e);
      }

      // Collect Core Web Vitals with mobile focus
      const coreWebVitals = await this.collectMobileCoreWebVitals(page);
      
      // Collect additional metrics
      const additionalMetrics = await this.collectMobileTimingMetrics(page);

      // Reset emulation
      try {
        if (cdpSession) {
          await cdpSession.send('Emulation.setCPUThrottlingRate', { rate: 1 });
          await cdpSession.send('Network.emulateNetworkConditions', {
            offline: false,
            latency: 0,
            downloadThroughput: -1,
            uploadThroughput: -1
          });
        }
      } catch (e) {
        console.warn('Failed to reset mobile emulation:', e);
      }
      
      // Calculate mobile performance score
      const score = this.calculateMobilePerformanceScore({
        ...coreWebVitals,
        ...additionalMetrics
      });
      
      const grade = this.calculateGrade(score);
      const recommendations = this.generateMobileRecommendations(coreWebVitals, additionalMetrics);
      const isMobileOptimized = this.assessMobileOptimization(coreWebVitals);

      if (this.options.verbose) {
        console.log(`📱 Mobile performance analysis completed in ${Date.now() - startTime}ms`);
        console.log(`📊 Mobile Score: ${score}/100 (Grade: ${grade})`);
      }

      return {
        score,
        grade,
        coreWebVitals,
        metrics: additionalMetrics,
        recommendations,
        isMobileOptimized
      };

    } catch (error) {
      console.error('❌ Mobile performance collection failed:', error);
      return this.getFallbackMobileMetrics();
    }
  }

  /**
   * Collect Core Web Vitals with mobile-specific measurement
   */
  private async collectMobileCoreWebVitals(page: Page): Promise<{
    lcp: number;
    fcp: number;
    cls: number;
    ttfb: number;
  }> {
    // Inject mobile-optimized Web Vitals measurement
    await page.addScriptTag({
      content: `
        window.mobileWebVitalsData = {
          lcp: 0,
          fcp: 0,
          cls: 0,
          ttfb: 0
        };

        // Enhanced LCP Observer for mobile (buffered true for late observers)
        if (typeof PerformanceObserver !== 'undefined') {
          try {
            const lcpObserver = new PerformanceObserver((list) => {
              const entries = list.getEntries();
              const lastEntry = entries[entries.length - 1];
              window.mobileWebVitalsData.lcp = Math.round(lastEntry.startTime);
            });
            try { lcpObserver.observe({ type: 'largest-contentful-paint', buffered: true }); } catch(_) { lcpObserver.observe({ entryTypes: ['largest-contentful-paint'] }); }

            // CLS Observer with mobile-specific handling
            let clsValue = 0;
            const clsObserver = new PerformanceObserver((list) => {
              for (const entry of list.getEntries()) {
                if (!entry.hadRecentInput) {
                  clsValue += entry.value;
                }
              }
              window.mobileWebVitalsData.cls = Math.round(clsValue * 1000) / 1000;
            });
            clsObserver.observe({ entryTypes: ['layout-shift'] });

            // FCP Observer
            const paintObserver = new PerformanceObserver((list) => {
              for (const entry of list.getEntries()) {
                if (entry.name === 'first-contentful-paint') {
                  window.mobileWebVitalsData.fcp = Math.round(entry.startTime);
                }
              }
            });
            paintObserver.observe({ entryTypes: ['paint'] });

          } catch (e) {
            console.warn('Mobile Web Vitals observation failed:', e);
          }
        }
      `
    });

    // Wait longer so the final LCP can settle
    await page.waitForTimeout(5000);

    // Get collected metrics
    const webVitals = await page.evaluate(() => {
      const navigation = performance.getEntriesByType('navigation')[0] as PerformanceNavigationTiming;
      const mobileData = (window as any).mobileWebVitalsData || { lcp: 0, fcp: 0, cls: 0, ttfb: 0 };
      
      // Calculate TTFB
      const ttfb = navigation ? Math.round(navigation.responseStart - navigation.requestStart) : 0;
      
      // Fallback measurements if observers didn't work
      if (mobileData.lcp === 0) {
        const paintEntries = performance.getEntriesByType('paint');
        const fcp = paintEntries.find(entry => entry.name === 'first-contentful-paint')?.startTime || 0;
        mobileData.lcp = fcp > 0 ? fcp * 1.2 : navigation?.loadEventEnd * 0.8 || 0;
      }
      
      if (mobileData.fcp === 0) {
        const paintEntries = performance.getEntriesByType('paint');
        mobileData.fcp = paintEntries.find(entry => entry.name === 'first-contentful-paint')?.startTime || 0;
      }
      
      return {
        lcp: Math.round(mobileData.lcp),
        fcp: Math.round(mobileData.fcp),
        cls: mobileData.cls,
        ttfb: Math.round(ttfb)
      };
    });

    return webVitals;
  }

  /**
   * Collect mobile-specific timing metrics
   */
  private async collectMobileTimingMetrics(page: Page): Promise<{
    domContentLoaded: number;
    loadComplete: number;
    renderTime: number;
  }> {
    const timingData = await page.evaluate(() => {
      const navigation = performance.getEntriesByType('navigation')[0] as PerformanceNavigationTiming;
      
      return {
        domContentLoaded: navigation ? Math.round(navigation.domContentLoadedEventEnd - navigation.fetchStart) : 0,
        loadComplete: navigation ? Math.round(navigation.loadEventEnd - navigation.fetchStart) : 0,
        renderTime: navigation ? Math.round(navigation.domContentLoadedEventEnd - navigation.responseEnd) : 0
      };
    });

    return timingData;
  }

  /**
   * Calculate mobile performance score with mobile-specific thresholds
   */
  private calculateMobilePerformanceScore(metrics: any): number {
    let score = 100;
    
    // Mobile-specific thresholds (stricter than desktop)
    // LCP scoring (35% weight - most critical for mobile)
    if (metrics.lcp > 4000) score -= 35;
    else if (metrics.lcp > 2500) score -= 25;
    else if (metrics.lcp > 2000) score -= 15;
    else if (metrics.lcp > 1500) score -= 5;
    
    // FCP scoring (30% weight)
    if (metrics.fcp > 3000) score -= 30;
    else if (metrics.fcp > 2000) score -= 20;
    else if (metrics.fcp > 1500) score -= 10;
    else if (metrics.fcp > 1200) score -= 5;
    
    // TTFB scoring (25% weight - critical for mobile networks)
    if (metrics.ttfb > 1000) score -= 25;
    else if (metrics.ttfb > 600) score -= 15;
    else if (metrics.ttfb > 400) score -= 8;
    else if (metrics.ttfb > 200) score -= 3;
    
    // CLS scoring (10% weight)
    if (metrics.cls > 0.25) score -= 10;
    else if (metrics.cls > 0.1) score -= 5;
    else if (metrics.cls > 0.05) score -= 2;
    
    return Math.max(0, Math.round(score));
  }

  /**
   * Calculate performance grade
   */
  private calculateGrade(score: number): 'A' | 'B' | 'C' | 'D' | 'F' {
    if (score >= 90) return 'A';
    if (score >= 80) return 'B';
    if (score >= 70) return 'C';
    if (score >= 60) return 'D';
    return 'F';
  }

  /**
   * Generate mobile-specific performance recommendations
   */
  private generateMobileRecommendations(coreWebVitals: any, metrics: any): string[] {
    const recommendations: string[] = [];
    
    // LCP recommendations for mobile
    if (coreWebVitals.lcp > 2500) {
      recommendations.push(`🎯 Mobile LCP is ${coreWebVitals.lcp}ms - optimize for mobile networks with image compression, lazy loading, and CDN`);
    }
    
    // FCP recommendations for mobile
    if (coreWebVitals.fcp > 1800) {
      recommendations.push(`⚡ Mobile FCP is ${coreWebVitals.fcp}ms - minimize critical CSS, optimize fonts, reduce JavaScript for mobile`);
    }
    
    // TTFB recommendations for mobile
    if (coreWebVitals.ttfb > 600) {
      recommendations.push(`🚀 Mobile TTFB is ${coreWebVitals.ttfb}ms - optimize server response, use mobile-optimized CDN, enable aggressive caching`);
    }
    
    // CLS recommendations for mobile
    if (coreWebVitals.cls > 0.1) {
      recommendations.push(`📐 Mobile CLS is ${coreWebVitals.cls} - set explicit dimensions for mobile images, avoid dynamic content insertion`);
    }
    
    // Load time recommendations for mobile
    if (metrics.loadComplete > 5000) {
      recommendations.push(`📱 Mobile load time is ${metrics.loadComplete}ms - implement service worker, optimize for mobile networks`);
    }
    
    if (recommendations.length === 0) {
      recommendations.push('🎉 Excellent mobile performance! All metrics meet mobile optimization standards.');
    }
    
    return recommendations;
  }

  /**
   * Assess mobile optimization status
   */
  private assessMobileOptimization(coreWebVitals: any): boolean {
    return (
      coreWebVitals.lcp <= 2500 &&
      coreWebVitals.fcp <= 1800 &&
      coreWebVitals.ttfb <= 600 &&
      coreWebVitals.cls <= 0.1
    );
  }

  /**
   * Fallback metrics when collection fails
   */
  private getFallbackMobileMetrics(): MobilePerformanceMetrics {
    return {
      score: 0,
      grade: 'F',
      coreWebVitals: {
        lcp: 0,
        fcp: 0,
        cls: 0,
        ttfb: 0
      },
      metrics: {
        domContentLoaded: 0,
        loadComplete: 0,
        renderTime: 0
      },
      recommendations: ['Mobile performance analysis failed - unable to collect metrics'],
      isMobileOptimized: false
    };
  }
}