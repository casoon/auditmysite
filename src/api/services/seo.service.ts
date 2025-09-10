import { SEOResult, SEOIssue } from '../../types/audit-results';

/**
 * SEOService - Returns SEOResult (same type used in PageAuditResult)
 * Used by API endpoint: POST /api/v2/page/seo
 */
export class SEOService {
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
          '--disable-gpu'
        ]
      }
    });
    
    await this.poolManager.warmUp(1);
  }
  
  async analyzeUrl(url: string, options: {
    timeout?: number;
  } = {}): Promise<SEOResult> {
    if (!this.poolManager) {
      await this.initializePool();
    }
    
    try {
      // Use pooled browser for SEO analysis
      const { PooledAccessibilityChecker } = require('../../core/accessibility/pooled-accessibility-checker');
      const checker = new PooledAccessibilityChecker(this.poolManager);
      
      const results = await checker.testMultiplePages([url], {
        enhancedSeoAnalysis: true,
        timeout: options.timeout || 10000,
        maxPages: 1
      });
      
      if (results[0]?.enhancedSEO) {
        const seoData = results[0].enhancedSEO;
        const score = this.calculateSEOScore(seoData);
        
        return {
          score,
          grade: this.scoreToGrade(score),
          metaTags: {
            title: seoData.title ? {
              content: seoData.title,
              length: seoData.title.length,
              optimal: seoData.title.length >= 10 && seoData.title.length <= 60
            } : undefined,
            description: seoData.description ? {
              content: seoData.description,
              length: seoData.description.length,
              optimal: seoData.description.length >= 120 && seoData.description.length <= 160
            } : undefined,
            openGraph: seoData.openGraph || {},
            twitterCard: seoData.twitterCard || {}
          },
          headings: {
            h1: seoData.headings?.h1 || [],
            h2: seoData.headings?.h2 || [],
            h3: seoData.headings?.h3 || [],
            issues: this.identifyHeadingIssues(seoData.headings)
          },
          images: {
            total: seoData.images?.total || 0,
            missingAlt: seoData.images?.missingAlt || 0,
            emptyAlt: seoData.images?.emptyAlt || 0
          },
          issues: this.identifySEOIssues(seoData)
        };
      }
    } catch (error) {
      console.warn('SEO analysis failed:', (error as Error).message);
    }
    
    // Fallback: Return minimal SEO data
    return {
      score: 0,
      grade: 'F',
      metaTags: {
        title: undefined,
        description: undefined,
        openGraph: {},
        twitterCard: {}
      },
      headings: {
        h1: [],
        h2: [],
        h3: [],
        issues: []
      },
      images: {
        total: 0,
        missingAlt: 0,
        emptyAlt: 0
      },
      issues: [{
        type: 'title-missing',
        message: 'SEO analysis failed - unable to collect data',
        severity: 'error'
      }]
    };
  }
  
  private calculateSEOScore(seoData: any): number {
    let score = 100;
    
    // Title evaluation
    if (!seoData.title) score -= 20;
    else if (seoData.title.length < 10 || seoData.title.length > 60) score -= 10;
    
    // Description evaluation
    if (!seoData.description) score -= 15;
    else if (seoData.description.length < 120 || seoData.description.length > 160) score -= 8;
    
    // Heading structure
    if (!seoData.headings?.h1 || seoData.headings.h1.length === 0) score -= 15;
    else if (seoData.headings.h1.length > 1) score -= 10;
    
    // Images with alt text
    if (seoData.images?.total > 0) {
      const altMissingRatio = seoData.images.missingAlt / seoData.images.total;
      if (altMissingRatio > 0.5) score -= 20;
      else if (altMissingRatio > 0.2) score -= 10;
    }
    
    return Math.max(0, Math.round(score));
  }
  
  private scoreToGrade(score: number): 'A' | 'B' | 'C' | 'D' | 'F' {
    if (score >= 90) return 'A';
    if (score >= 80) return 'B';
    if (score >= 70) return 'C';
    if (score >= 60) return 'D';
    return 'F';
  }
  
  private identifyHeadingIssues(headings: any): string[] {
    const issues: string[] = [];
    
    if (!headings?.h1 || headings.h1.length === 0) {
      issues.push('Missing H1 tag - pages should have exactly one H1');
    } else if (headings.h1.length > 1) {
      issues.push(`Multiple H1 tags found (${headings.h1.length}) - pages should have only one H1`);
    }
    
    return issues;
  }
  
  private identifySEOIssues(seoData: any): SEOIssue[] {
    const issues: SEOIssue[] = [];
    
    if (!seoData.title) {
      issues.push({
        type: 'title-missing',
        message: 'Missing title tag',
        severity: 'error'
      });
    } else if (seoData.title.length < 10) {
      issues.push({
        type: 'title-long',
        message: 'Title too short (minimum 10 characters recommended)',
        severity: 'warning'
      });
    } else if (seoData.title.length > 60) {
      issues.push({
        type: 'title-long',
        message: 'Title too long (maximum 60 characters recommended)',
        severity: 'warning'
      });
    }
    
    if (!seoData.description) {
      issues.push({
        type: 'description-missing',
        message: 'Missing meta description',
        severity: 'error'
      });
    }
    
    if (seoData.images?.total > 0 && seoData.images.missingAlt > 0) {
      issues.push({
        type: 'image-alt-missing',
        message: `${seoData.images.missingAlt} of ${seoData.images.total} images missing alt text`,
        severity: 'warning'
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
