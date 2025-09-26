/**
 * 📊 Content Weight Analyzer
 * 
 * Analyzes the weight and composition of webpage content including:
 * - Resource sizes (HTML, CSS, JS, images, fonts)
 * - Content quality metrics
 * - Text-to-code ratios
 * - Performance impact analysis
 */

import { Page, Response } from 'playwright';
import { 
  ContentWeight, 
  ContentAnalysis, 
  ResourceTiming, 
  QualityAnalysisOptions 
} from '../types/enhanced-metrics';
import {
  BaseAnalyzer,
  BaseAnalysisResult,
  BaseAnalysisOptions,
  BaseRecommendation,
  Grade,
  CertificateLevel,
  calculateGrade,
  calculateCertificateLevel
} from '../types/base-types';

// Content Weight specific result interface
interface ContentWeightAnalysisResult extends BaseAnalysisResult {
  contentWeight: ContentWeight;
  contentAnalysis: ContentAnalysis;
  resourceTimings: ResourceTiming[];
  recommendations: BaseRecommendation[];
}

// Content Weight specific options interface
interface ContentWeightAnalysisOptions extends BaseAnalysisOptions {
  includeResourceAnalysis?: boolean;
  analysisTimeout?: number;
  verbose?: boolean; // Controls logging verbosity
}

export class ContentWeightAnalyzer implements BaseAnalyzer<ContentWeightAnalysisResult, ContentWeightAnalysisOptions> {
  private resourceTimings: ResourceTiming[] = [];
  private responses: Response[] = [];

  constructor() {}

  // BaseAnalyzer interface implementations
  getName(): string {
    return 'ContentWeightAnalyzer';
  }

  getVersion(): string {
    return '2.0.0';
  }

  getScore(result: ContentWeightAnalysisResult): number {
    return result.overallScore;
  }

  getGrade(score: number): Grade {
    return calculateGrade(score);
  }

  getCertificateLevel(score: number): CertificateLevel {
    return calculateCertificateLevel(score);
  }

  getRecommendations(result: ContentWeightAnalysisResult): BaseRecommendation[] {
    return result.recommendations;
  }

  /**
   * Main analyze method implementing BaseAnalyzer interface
   */
  async analyze(page: Page, url: string | { loc: string }, options: ContentWeightAnalysisOptions = {}): Promise<ContentWeightAnalysisResult> {
    return this.analyzeWithResponses(page, url, options, []);
  }

  /**
   * Enhanced analyze method that accepts pre-captured network responses
   * This fixes the issue where network monitoring starts after page load
   */
  async analyzeWithResponses(page: Page, url: string | { loc: string }, options: ContentWeightAnalysisOptions = {}, networkResponses: Response[] = []): Promise<ContentWeightAnalysisResult> {
    // Extract URL string from URL object if needed
    const urlString = (typeof url === 'object' && url.loc ? url.loc : url) as string;

    const startTime = Date.now();

    // Use a separate analysis page when we must reload to capture all responses
    let analysisPage: Page = page;
    let tempPage: Page | null = null;
    
    try {
      // 🔧 CRITICAL FIX: Use pre-captured network responses instead of setting up tracking
      // This ensures we capture all network requests from the beginning of page load
      if (networkResponses.length > 0) {
        this.responses = networkResponses;
        if (options.verbose) {
          console.log(`📊 Using ${this.responses.length} pre-captured network responses for analysis`);
        }
      } else {
        // Strict capture on an isolated page to avoid interfering with other analyzers
        const context = page.context();
        tempPage = await context.newPage();
        analysisPage = tempPage;

        this.setupResponseTracking(analysisPage);
        const tsParam = (urlString.includes('?') ? '&' : '?') + 'ams_nocache=' + Date.now();
        const freshUrl = urlString + tsParam;
        await analysisPage.goto(freshUrl, { waitUntil: 'networkidle', timeout: options.analysisTimeout || 30000 });
        if (options.verbose) {
          console.log(`📊 Captured ${this.responses.length} network responses for analysis`);
        }
      }
      

      // Collect resource data from the analysis page
      const contentWeight = await this.calculateContentWeight(analysisPage);
      const contentAnalysis = await this.analyzeContentComposition(analysisPage);
      const resourceTimings = options.includeResourceAnalysis ? await this.extractResourceTimings(analysisPage) : [];
      
      const duration = Date.now() - startTime;
      
      // Calculate overall score based on content weight metrics
      const overallScore = this.calculateOverallScore(contentWeight, contentAnalysis);
      const grade = calculateGrade(overallScore);
      const certificate = calculateCertificateLevel(overallScore);
      
      // Generate recommendations
      const recommendations = this.generateRecommendations(contentWeight, contentAnalysis);

      const result: ContentWeightAnalysisResult = {
        overallScore,
        grade,
        certificate,
        analyzedAt: new Date().toISOString(),
        duration,
        status: 'completed' as const,
        contentWeight,
        contentAnalysis,
        resourceTimings,
        recommendations
      };

      return result;

    } catch (error) {
      console.error('❌ Content weight analysis failed:', error);
      throw new Error(`Content weight analysis failed: ${error}`);
    } finally {
      if (tempPage) {
        try { await tempPage.close(); } catch {}
      }
    }
  }

  /**
   * Set up response tracking to capture all network requests
   */
  private setupResponseTracking(page: Page) {
    this.responses = [];
    this.resourceTimings = [];

    page.on('response', (response) => {
      this.responses.push(response);
    });
  }

  /**
   * Calculate the weight of different content types
   */
  private async calculateContentWeight(page: Page): Promise<ContentWeight> {
    const weights: ContentWeight = {
      html: 0,
      css: 0,
      javascript: 0,
      images: 0,
      fonts: 0,
      other: 0,
      total: 0,
      gzipTotal: 0,
      compressionRatio: 0
    };

    let totalTransferSize = 0;

    // Analyze all captured responses
    for (const response of this.responses) {
      try {
        const url = response.url();
        const headers = await response.headers();
        const size = await this.getResponseSize(response);
        const transferSize = this.getTransferSize(headers, size);
        
        totalTransferSize += transferSize;

        // Categorize by content type
        const contentType = headers['content-type'] || '';
        const category = this.categorizeResource(url, contentType);
        
        weights[category] += size;
        weights.total += size;

      } catch (error) {
        console.warn(`Failed to analyze response ${response.url()}:`, error);
      }
    }


    // Calculate compression metrics
    weights.gzipTotal = totalTransferSize || weights.gzipTotal || 0;
    weights.compressionRatio = weights.total > 0 ? (weights.gzipTotal / weights.total) : 0;

    return weights;
  }

  /**
   * Analyze content composition and quality metrics
   */
  private async analyzeContentComposition(page: Page): Promise<ContentAnalysis> {
    const analysis = await page.evaluate(() => {
      // Count text content
      const bodyText = document.body.innerText || '';
      const textContent = bodyText.length;
      const wordCount = bodyText.trim().split(/\s+/).filter(word => word.length > 0).length;
      
      // Count various elements
      const imageCount = document.querySelectorAll('img').length;
      const linkCount = document.querySelectorAll('a').length;
      const domElements = document.querySelectorAll('*').length;

      // Get HTML size for ratio calculation
      const htmlSize = new TextEncoder().encode(document.documentElement.outerHTML).length;
      
      return {
        textContent,
        wordCount,
        imageCount,
        linkCount,
        domElements,
        htmlSize
      };
    });

    // Calculate text-to-code ratio
    const textToCodeRatio = analysis.htmlSize > 0 
      ? analysis.textContent / analysis.htmlSize 
      : 0;

    // Calculate content quality score
    const contentQualityScore = this.calculateContentQualityScore({
      ...analysis,
      textToCodeRatio
    });

    return {
      textContent: analysis.textContent,
      imageCount: analysis.imageCount,
      linkCount: analysis.linkCount,
      domElements: analysis.domElements,
      textToCodeRatio,
      contentQualityScore,
      wordCount: analysis.wordCount
    };
  }

  /**
   * Extract detailed resource timing information
   */
  private async extractResourceTimings(page: Page): Promise<ResourceTiming[]> {

    const resourceTimings: ResourceTiming[] = [];

    // Get performance entries from the page
    const performanceEntries = await page.evaluate(() => {
      const entries = performance.getEntriesByType('resource');
      return entries.map(entry => ({
        name: entry.name,
        startTime: entry.startTime,
        duration: entry.duration,
        transferSize: (entry as any).transferSize || 0,
        encodedBodySize: (entry as any).encodedBodySize || 0,
        decodedBodySize: (entry as any).decodedBodySize || 0
      }));
    });

    // Combine with response data
    for (const entry of performanceEntries) {
      const matchingResponse = this.responses.find(r => r.url() === entry.name);
      
      resourceTimings.push({
        url: entry.name,
        type: this.getResourceType(entry.name),
        size: entry.decodedBodySize || entry.encodedBodySize,
        duration: entry.duration,
        transferSize: entry.transferSize,
        cached: entry.transferSize === 0 && entry.duration < 10
      });
    }

    return resourceTimings.sort((a, b) => b.size - a.size);
  }

  /**
   * Get the size of a response
   */
  private async getResponseSize(response: Response): Promise<number> {
    try {
      const buffer = await response.body();
      return buffer.length;
    } catch {
      // Fallback to content-length header
      const headers = await response.headers();
      return parseInt(headers['content-length'] || '0', 10);
    }
  }

  /**
   * Get transfer size from headers
   */
  private getTransferSize(headers: { [key: string]: string }, bodySize: number): number {
    // If gzipped, estimate compression
    const isCompressed = headers['content-encoding']?.includes('gzip') || 
                        headers['content-encoding']?.includes('br') ||
                        headers['content-encoding']?.includes('deflate');
    
    if (isCompressed && bodySize > 0) {
      // Typical compression ratios: text ~70%, images ~5%
      const contentType = headers['content-type'] || '';
      if (contentType.includes('text') || contentType.includes('javascript') || contentType.includes('css')) {
        return Math.round(bodySize * 0.3); // ~70% compression
      }
      return Math.round(bodySize * 0.95); // ~5% compression for images
    }
    
    return bodySize;
  }

  /**
   * Categorize resource by URL and content type
   */
  private categorizeResource(url: string, contentType: string): keyof ContentWeight {
    // Remove the computed properties from the type check
    if (contentType.includes('text/html')) return 'html';
    if (contentType.includes('text/css') || url.includes('.css')) return 'css';
    if (contentType.includes('javascript') || url.includes('.js') || url.includes('.mjs')) return 'javascript';
    if (contentType.includes('image/') || /\.(jpg|jpeg|png|gif|webp|svg|ico)(\?|$)/i.test(url)) return 'images';
    if (contentType.includes('font/') || /\.(woff|woff2|ttf|otf|eot)(\?|$)/i.test(url)) return 'fonts';
    return 'other';
  }

  /**
   * Get resource type for timing analysis
   */
  private getResourceType(url: string): string {
    if (/\.(css)(\?|$)/i.test(url)) return 'stylesheet';
    if (/\.(js|mjs)(\?|$)/i.test(url)) return 'script';
    if (/\.(jpg|jpeg|png|gif|webp|svg|ico)(\?|$)/i.test(url)) return 'image';
    if (/\.(woff|woff2|ttf|otf|eot)(\?|$)/i.test(url)) return 'font';
    if (/\.(mp4|mov|avi|webm)(\?|$)/i.test(url)) return 'video';
    if (/\.(mp3|wav|ogg)(\?|$)/i.test(url)) return 'audio';
    return 'other';
  }

  /**
   * Calculate content quality score based on various factors
   */
  private calculateContentQualityScore(analysis: {
    textContent: number;
    wordCount: number;
    imageCount: number;
    linkCount: number;
    domElements: number;
    textToCodeRatio: number;
  }): number {
    let score = 100;

    // Text content scoring (40% of total)
    if (analysis.wordCount < 200) score -= 20;
    else if (analysis.wordCount < 300) score -= 10;
    else if (analysis.wordCount > 2000) score += 10;

    // Text-to-code ratio scoring (30% of total)
    if (analysis.textToCodeRatio < 0.1) score -= 20;
    else if (analysis.textToCodeRatio < 0.2) score -= 10;
    else if (analysis.textToCodeRatio > 0.4) score += 15;

    // DOM complexity scoring (20% of total)
    if (analysis.domElements > 2000) score -= 15;
    else if (analysis.domElements > 1500) score -= 10;
    else if (analysis.domElements < 500) score += 5;

    // Media balance scoring (10% of total)
    const imageToTextRatio = analysis.wordCount > 0 ? analysis.imageCount / analysis.wordCount : 0;
    if (imageToTextRatio > 0.1) score -= 10; // Too many images relative to text
    else if (imageToTextRatio > 0.05) score -= 5;

    return Math.max(0, Math.min(100, score));
  }

  /**
   * Format bytes to human readable string
   */
  private formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  }

  /**
   * Calculate overall score from content weight and analysis data
   */
  private calculateOverallScore(contentWeight: ContentWeight, contentAnalysis: ContentAnalysis): number {
    let score = 100;

    // Size scoring (50% weight)
    const totalSizeMB = contentWeight.total / (1024 * 1024);
    if (totalSizeMB > 5) score -= 30;
    else if (totalSizeMB > 3) score -= 20;
    else if (totalSizeMB > 1.5) score -= 10;
    else if (totalSizeMB < 0.5) score += 10;

    // Compression scoring (20% weight)
    const compressionScore = (contentWeight.compressionRatio || 1) < 0.7 ? 15 : 0;
    score += compressionScore;

    // Content quality scoring (30% weight)
    score = Math.round(score * 0.7 + contentAnalysis.contentQualityScore * 0.3);

    return Math.max(0, Math.min(100, score));
  }

  /**
   * Generate optimization recommendations
   */
  private generateRecommendations(contentWeight: ContentWeight, contentAnalysis: ContentAnalysis): BaseRecommendation[] {
    const recommendations: BaseRecommendation[] = [];


    // Large image recommendations
    if (contentWeight.images > 1024 * 1024) { // > 1MB images
      recommendations.push({
        id: 'optimize-images',
        priority: 'high',
        category: 'Performance',
        issue: 'Large image files detected',
        recommendation: 'Optimize images by compressing, using modern formats (WebP/AVIF), and implementing responsive images',
        impact: 'Reduce page load time and bandwidth usage',
        effort: 4,
        scoreImprovement: 15
      });
    }

    // Large JavaScript bundles
    if (contentWeight.javascript > 500 * 1024) { // > 500KB JS
      recommendations.push({
        id: 'optimize-javascript',
        priority: 'medium',
        category: 'Performance',
        issue: 'Large JavaScript bundles detected',
        recommendation: 'Split JavaScript into smaller chunks, remove unused code, and implement code splitting',
        impact: 'Improve initial page load time and reduce bundle size',
        effort: 6,
        scoreImprovement: 10
      });
    }

    // Poor compression
    if ((contentWeight.compressionRatio || 1) > 0.8) {
      recommendations.push({
        id: 'enable-compression',
        priority: 'medium',
        category: 'Performance',
        issue: 'Poor or missing text compression',
        recommendation: 'Enable gzip/brotli compression on your web server for text resources',
        impact: 'Significantly reduce transferred file sizes',
        effort: 2,
        scoreImprovement: 8
      });
    }

    // Low text-to-code ratio
    if (contentAnalysis.textToCodeRatio < 0.2) {
      recommendations.push({
        id: 'improve-content-ratio',
        priority: 'low',
        category: 'Content Quality',
        issue: 'Low text-to-code ratio detected',
        recommendation: 'Increase meaningful text content or reduce excessive markup and scripts',
        impact: 'Improve content quality and user experience',
        effort: 3,
        scoreImprovement: 5
      });
    }


    return recommendations;
  }

  /**
   * Get performance recommendations based on content weight analysis
   */
  static generateContentRecommendations(
    contentWeight: ContentWeight,
    contentAnalysis: ContentAnalysis
  ): string[] {
    const recommendations: string[] = [];
    const totalMB = contentWeight.total / (1024 * 1024);

    // Size-based recommendations
    if (totalMB > 3) {
      recommendations.push(`📏 Page size is ${totalMB.toFixed(1)}MB - consider optimizing large resources`);
    }

    if (contentWeight.images > contentWeight.total * 0.6) {
      recommendations.push(`🖼️ Images comprise ${((contentWeight.images / contentWeight.total) * 100).toFixed(0)}% of page weight - optimize image sizes`);
    }

    if (contentWeight.javascript > 1024 * 1024) {
      recommendations.push(`📜 JavaScript bundle is ${(contentWeight.javascript / (1024 * 1024)).toFixed(1)}MB - consider code splitting`);
    }

    if (contentWeight.compressionRatio && contentWeight.compressionRatio > 0.8) {
      recommendations.push(`🗜️ Enable better compression - current ratio: ${(contentWeight.compressionRatio * 100).toFixed(0)}%`);
    }

    // Content quality recommendations
    if (contentAnalysis.textToCodeRatio < 0.15) {
      recommendations.push(`📝 Low text-to-code ratio (${(contentAnalysis.textToCodeRatio * 100).toFixed(0)}%) - add more meaningful content`);
    }

    if (contentAnalysis.domElements > 1500) {
      recommendations.push(`🏗️ High DOM complexity (${contentAnalysis.domElements} elements) - simplify page structure`);
    }

    if (contentAnalysis.wordCount < 300) {
      recommendations.push(`💬 Low word count (${contentAnalysis.wordCount} words) - add more content for better SEO`);
    }

    return recommendations;
  }
}
