/**
 * 🔍 Enhanced SEO Analyzer
 * 
 * Comprehensive SEO analysis including:
 * - Meta tags optimization
 * - Heading structure validation
 * - Social media meta tags
 * - Technical SEO factors
 * - Content quality analysis
 * - Readability scoring
 */

import { Page, Response } from 'playwright';
import { log } from '@core/logging';
import { 
  SEOMetrics,
  MetaTagAnalysis,
  HeadingStructure,
  SocialMetaTags,
  TechnicalSEO,
  QualityAnalysisOptions 
} from '../types/enhanced-metrics';

export class SEOAnalyzer {
  constructor(private options: QualityAnalysisOptions = {}) {}

  /**
   * Perform comprehensive SEO analysis of a webpage
   */
  async analyzeSEO(page: Page, url: string | { loc: string }): Promise<SEOMetrics> {
    // Extract URL string from URL object if needed
    const urlString = (typeof url === 'object' && url.loc ? url.loc : url) as string;
    
    const startTime = Date.now();

    try {
      // Use already loaded content - navigation is handled by main test flow
      // Skip navigation completely to preserve page context for comprehensive analysis

      // Collect all SEO metrics in parallel
      const [
        metaTags,
        headingStructure,
        socialTags,
        technicalSEO,
        contentMetrics
      ] = await Promise.all([
        this.analyzeMetaTags(page),
        this.analyzeHeadingStructure(page),
        this.options.includeSocialAnalysis ? this.analyzeSocialTags(page) : this.getDefaultSocialTags(),
        this.options.includeTechnicalSEO ? this.analyzeTechnicalSEO(page, urlString) : this.getDefaultTechnicalSEO(),
        this.analyzeContentQuality(page)
      ]);

      // Calculate SEO scores
      const overallSEOScore = this.calculateOverallSEOScore({
        metaTags,
        headingStructure,
        socialTags,
        technicalSEO,
        ...contentMetrics
      });

      const seoGrade = this.calculateSEOGrade(overallSEOScore);
      const recommendations = this.generateSEORecommendations({
        metaTags,
        headingStructure,
        socialTags,
        technicalSEO,
        ...contentMetrics
      });

      const searchVisibility = this.estimateSearchVisibility(overallSEOScore, contentMetrics.wordCount);
      const opportunityAreas = this.identifyOpportunityAreas({
        metaTags,
        headingStructure,
        socialTags,
        technicalSEO,
        ...contentMetrics
      });

      const seoMetrics: SEOMetrics = {
        metaTags,
        headingStructure,
        socialTags,
        technicalSEO,
        wordCount: contentMetrics.wordCount,
        readabilityScore: contentMetrics.readabilityScore,
        contentQuality: contentMetrics.contentQuality,
        contentUniqueness: contentMetrics.contentUniqueness,
        overallSEOScore,
        seoGrade,
        recommendations,
        searchVisibility,
        opportunityAreas
      };


      return seoMetrics;

    } catch (error) {
      console.error('❌ SEO analysis failed:', error);
      throw new Error(`SEO analysis failed: ${error}`);
    }
  }

  /**
   * Analyze meta tags
   */
  private async analyzeMetaTags(page: Page): Promise<MetaTagAnalysis> {
    const metaData = await page.evaluate(() => {
      const titleElement = document.querySelector('title');
      const descriptionElement = document.querySelector('meta[name="description"]');
      const keywordsElement = document.querySelector('meta[name="keywords"]');
      const robotsElement = document.querySelector('meta[name="robots"]');
      const canonicalElement = document.querySelector('link[rel="canonical"]');
      const viewportElement = document.querySelector('meta[name="viewport"]');

      return {
        title: titleElement ? titleElement.textContent : null,
        description: descriptionElement ? descriptionElement.getAttribute('content') : null,
        keywords: keywordsElement ? keywordsElement.getAttribute('content') : null,
        robots: robotsElement ? robotsElement.getAttribute('content') : null,
        canonical: canonicalElement ? canonicalElement.getAttribute('href') : null,
        viewport: viewportElement ? viewportElement.getAttribute('content') : null
      };
    });

    // Analyze title tag
    const titleAnalysis = {
      present: !!metaData.title,
      content: metaData.title || undefined,
      length: metaData.title ? metaData.title.length : 0,
      optimal: false,
      issues: [] as string[]
    };

    if (!metaData.title) {
      titleAnalysis.issues.push('Title tag is missing');
    } else {
      if (metaData.title.length < 30) {
        titleAnalysis.issues.push('Title is too short (< 30 characters)');
      } else if (metaData.title.length > 60) {
        titleAnalysis.issues.push('Title is too long (> 60 characters)');
      } else {
        titleAnalysis.optimal = true;
      }
    }

    // Analyze description tag
    const descriptionAnalysis = {
      present: !!metaData.description,
      content: metaData.description || undefined,
      length: metaData.description ? metaData.description.length : 0,
      optimal: false,
      issues: [] as string[]
    };

    if (!metaData.description) {
      descriptionAnalysis.issues.push('Meta description is missing');
    } else {
      if (metaData.description.length < 120) {
        descriptionAnalysis.issues.push('Meta description is too short (< 120 characters)');
      } else if (metaData.description.length > 160) {
        descriptionAnalysis.issues.push('Meta description is too long (> 160 characters)');
      } else {
        descriptionAnalysis.optimal = true;
      }
    }

    return {
      title: titleAnalysis,
      description: descriptionAnalysis,
      keywords: metaData.keywords ? {
        present: true,
        content: metaData.keywords,
        relevant: this.assessKeywordRelevance(metaData.keywords, metaData.title || '')
      } : { present: false, relevant: false },
      robots: metaData.robots ? {
        present: true,
        content: metaData.robots,
        indexable: !metaData.robots.includes('noindex')
      } : { present: false, indexable: true },
      canonical: metaData.canonical ? {
        present: true,
        url: metaData.canonical,
        valid: this.isValidUrl(metaData.canonical)
      } : { present: false, valid: false },
      viewport: metaData.viewport ? {
        present: true,
        mobileOptimized: metaData.viewport.includes('width=device-width')
      } : { present: false, mobileOptimized: false }
    };
  }

  /**
   * Analyze heading structure
   */
  private async analyzeHeadingStructure(page: Page): Promise<HeadingStructure> {
    const headingData = await page.evaluate(() => {
      const h1s = document.querySelectorAll('h1');
      const h2s = document.querySelectorAll('h2');
      const h3s = document.querySelectorAll('h3');
      const h4s = document.querySelectorAll('h4');
      const h5s = document.querySelectorAll('h5');
      const h6s = document.querySelectorAll('h6');

      // Get heading hierarchy
      const allHeadings = Array.from(document.querySelectorAll('h1, h2, h3, h4, h5, h6'));
      const headingLevels = allHeadings.map(h => parseInt(h.tagName.charAt(1)));

      return {
        h1Count: h1s.length,
        h2Count: h2s.length,
        h3Count: h3s.length,
        h4Count: h4s.length,
        h5Count: h5s.length,
        h6Count: h6s.length,
        headingLevels,
        h1Text: h1s.length > 0 ? Array.from(h1s).map(h => h.textContent || '').join(' | ') : ''
      };
    });

    // Validate heading structure
    const issues: string[] = [];
    let structureValid = true;

    if (headingData.h1Count === 0) {
      issues.push('No H1 tag found');
      structureValid = false;
    } else if (headingData.h1Count > 1) {
      issues.push(`Multiple H1 tags found (${headingData.h1Count})`);
      structureValid = false;
    }

    // Check hierarchy
    if (headingData.headingLevels.length > 1) {
      for (let i = 1; i < headingData.headingLevels.length; i++) {
        const current = headingData.headingLevels[i];
        const previous = headingData.headingLevels[i - 1];
        
        if (current > previous + 1) {
          issues.push(`Heading hierarchy skips levels (H${previous} followed by H${current})`);
          structureValid = false;
        }
      }
    }

    if (headingData.h2Count === 0 && headingData.h1Count > 0) {
      issues.push('H1 exists but no H2 tags found - consider adding subheadings');
    }

    return {
      h1Count: headingData.h1Count,
      h2Count: headingData.h2Count,
      h3Count: headingData.h3Count,
      h4Count: headingData.h4Count,
      h5Count: headingData.h5Count,
      h6Count: headingData.h6Count,
      structureValid,
      issues
    };
  }

  /**
   * Analyze social media meta tags
   */
  private async analyzeSocialTags(page: Page): Promise<SocialMetaTags> {
    const socialData = await page.evaluate(() => {
      // Open Graph tags
      const ogTitle = document.querySelector('meta[property="og:title"]');
      const ogDescription = document.querySelector('meta[property="og:description"]');
      const ogImage = document.querySelector('meta[property="og:image"]');
      const ogUrl = document.querySelector('meta[property="og:url"]');
      const ogType = document.querySelector('meta[property="og:type"]');
      const ogSiteName = document.querySelector('meta[property="og:site_name"]');
      const ogLocale = document.querySelector('meta[property="og:locale"]');

      // Twitter Card tags
      const twitterCard = document.querySelector('meta[name="twitter:card"]');
      const twitterTitle = document.querySelector('meta[name="twitter:title"]');
      const twitterDescription = document.querySelector('meta[name="twitter:description"]');
      const twitterImage = document.querySelector('meta[name="twitter:image"]');
      const twitterSite = document.querySelector('meta[name="twitter:site"]');
      const twitterCreator = document.querySelector('meta[name="twitter:creator"]');

      return {
        og: {
          title: ogTitle ? ogTitle.getAttribute('content') : undefined,
          description: ogDescription ? ogDescription.getAttribute('content') : undefined,
          image: ogImage ? ogImage.getAttribute('content') : undefined,
          url: ogUrl ? ogUrl.getAttribute('content') : undefined,
          type: ogType ? ogType.getAttribute('content') : undefined,
          siteName: ogSiteName ? ogSiteName.getAttribute('content') : undefined,
          locale: ogLocale ? ogLocale.getAttribute('content') : undefined
        },
        twitter: {
          card: twitterCard ? twitterCard.getAttribute('content') : undefined,
          title: twitterTitle ? twitterTitle.getAttribute('content') : undefined,
          description: twitterDescription ? twitterDescription.getAttribute('content') : undefined,
          image: twitterImage ? twitterImage.getAttribute('content') : undefined,
          site: twitterSite ? twitterSite.getAttribute('content') : undefined,
          creator: twitterCreator ? twitterCreator.getAttribute('content') : undefined
        }
      };
    });

    // Calculate completeness score
    const ogFields = Object.keys(socialData.og).filter(key => socialData.og[key as keyof typeof socialData.og]);
    const twitterFields = Object.keys(socialData.twitter).filter(key => socialData.twitter[key as keyof typeof socialData.twitter]);
    const totalFields = 13; // 7 OG + 6 Twitter
    const completenessScore = Math.round(((ogFields.length + twitterFields.length) / totalFields) * 100);

    return {
      openGraph: {
        title: socialData.og.title || undefined,
        description: socialData.og.description || undefined,
        image: socialData.og.image || undefined,
        url: socialData.og.url || undefined,
        type: socialData.og.type || undefined,
        siteName: socialData.og.siteName || undefined,
        locale: socialData.og.locale || undefined,
      },
      twitterCard: {
        card: socialData.twitter.card || undefined,
        title: socialData.twitter.title || undefined,
        description: socialData.twitter.description || undefined,
        image: socialData.twitter.image || undefined,
        site: socialData.twitter.site || undefined,
        creator: socialData.twitter.creator || undefined,
      },
      completenessScore
    };
  }

  /**
   * Analyze technical SEO factors
   */
  private async analyzeTechnicalSEO(page: Page, url: string): Promise<TechnicalSEO> {
    // Check if page context is still valid before proceeding
    try {
      await page.title(); // Quick test to see if context is still valid
    } catch (error) {
      // Always show this fallback - indicates page context issues that need investigation
      log.fallback('Technical SEO', 'page context unavailable', 'using minimal data');
      return this.getFallbackTechnicalSEO(url);
    }
    
    let technicalData;
    try {
      technicalData = await page.evaluate(() => {
        const links = Array.from(document.querySelectorAll('a[href]'));
        const internalLinks = links.filter(link => {
          const href = link.getAttribute('href');
          return href && (href.startsWith('/') || href.includes(window.location.hostname));
        });
        const externalLinks = links.filter(link => {
          const href = link.getAttribute('href');
          return href && !href.startsWith('/') && !href.includes(window.location.hostname) && href.startsWith('http');
        });

        // Check for schema markup
        const schemaScripts = Array.from(document.querySelectorAll('script[type="application/ld+json"]'));
        const schemaTypes = schemaScripts.map(script => {
          try {
            const data = JSON.parse(script.textContent || '');
            return data['@type'] || 'Unknown';
          } catch {
            return 'Invalid';
          }
        });

        return {
          internalLinkCount: internalLinks.length,
          externalLinkCount: externalLinks.length,
          schemaTypes,
          allLinks: links.map(link => link.getAttribute('href')).filter(Boolean)
        };
      });
    } catch (error) {
      // Always show this fallback - indicates technical SEO evaluation issues
      log.fallback('Technical SEO', 'page evaluation failed', 'using minimal data', error);
      return this.getFallbackTechnicalSEO(url);
    }

    // Check HTTPS
    const httpsEnabled = url.startsWith('https://');

    // Check for sitemap and robots.txt (simplified - would need actual HTTP requests)
    const domain = new URL(url).origin;
    
    // Mock technical SEO data (in real implementation, would make HTTP requests)
    const sitemapPresent = false; // Would check domain + '/sitemap.xml'
    const robotsTxtPresent = false; // Would check domain + '/robots.txt'
    
    // Simple page speed score (based on performance metrics if available)
    const pageSpeedScore = 75; // Would calculate from performance metrics

    // Mobile-friendly (basic check based on viewport)
    let mobileFriendly = false;
    try {
      // More robust check - first verify page context is still valid
      await page.title(); // Quick context check
      mobileFriendly = await page.evaluate(() => {
        const viewport = document.querySelector('meta[name="viewport"]');
        return !!(viewport && viewport.getAttribute('content')?.includes('width=device-width'));
      });
    } catch (error) {
      log.fallback('SEO Mobile Check', 'page context destroyed during check', 'assuming mobile-unfriendly', error);
      mobileFriendly = false;
    }

    // Check for broken links (simplified)
    const brokenLinks = 0; // In real implementation, would test each link

    return {
      httpsEnabled,
      sitemapPresent,
      robotsTxtPresent,
      schemaMarkup: technicalData.schemaTypes,
      pageSpeedScore,
      mobileFriendly,
      linkAnalysis: {
        internalLinks: technicalData.internalLinkCount,
        externalLinks: technicalData.externalLinkCount,
        brokenLinks
      }
    };
  }

  /**
   * Analyze content quality including images and alt text
   */
  private async analyzeContentQuality(page: Page): Promise<{
    wordCount: number;
    readabilityScore: number;
    contentQuality: 'poor' | 'fair' | 'good' | 'excellent';
    contentUniqueness: number;
    // NEW: Image analysis data
    imageAnalysis?: {
      totalImages: number;
      imagesWithAlt: number;
      imagesWithoutAlt: number;
      emptyAltImages: number;
      decorativeImages: number;
    };
  }> {
    const contentData = await page.evaluate(() => {
      const bodyText = document.body.innerText || '';
      const words = bodyText.trim().split(/\s+/).filter(word => word.length > 0);
      const sentences = bodyText.split(/[.!?]+/).filter(s => s.trim().length > 0);
      const paragraphs = bodyText.split(/\n\s*\n/).filter(p => p.trim().length > 0);

      // Calculate average words per sentence
      const avgWordsPerSentence = sentences.length > 0 ? words.length / sentences.length : 0;
      
      // Calculate average syllables per word (simplified)
      const avgSyllablesPerWord = 1.5; // Simplified: average English word has ~1.5 syllables
      
      // NEW: Analyze images and alt text
      const images = document.querySelectorAll('img');
      let imagesWithAlt = 0;
      let imagesWithoutAlt = 0;
      let emptyAltImages = 0;
      let decorativeImages = 0;
      
      images.forEach(img => {
        const alt = img.getAttribute('alt');
        if (alt === null) {
          // No alt attribute at all
          imagesWithoutAlt++;
        } else if (alt.trim() === '') {
          // Empty alt (decorative image)
          emptyAltImages++;
          decorativeImages++;
        } else {
          // Has meaningful alt text
          imagesWithAlt++;
        }
      });
      
      console.log(`🖼️ SEO Image Analysis: ${images.length} total images, ${imagesWithAlt} with alt, ${imagesWithoutAlt} missing alt, ${emptyAltImages} empty alt`);

      return {
        wordCount: words.length,
        sentenceCount: sentences.length,
        paragraphCount: paragraphs.length,
        avgWordsPerSentence,
        avgSyllablesPerWord,
        fullText: bodyText,
        // Image analysis results
        imageAnalysis: {
          totalImages: images.length,
          imagesWithAlt,
          imagesWithoutAlt,
          emptyAltImages,
          decorativeImages
        }
      };
    });

    // Calculate Flesch Reading Ease Score (simplified)
    const readabilityScore = this.calculateReadabilityScore(
      contentData.avgWordsPerSentence,
      contentData.avgSyllablesPerWord
    );

    // Determine content quality
    let contentQuality: 'poor' | 'fair' | 'good' | 'excellent';
    if (contentData.wordCount < 300) {
      contentQuality = 'poor';
    } else if (contentData.wordCount < 500) {
      contentQuality = 'fair';
    } else if (contentData.wordCount < 1000) {
      contentQuality = 'good';
    } else {
      contentQuality = 'excellent';
    }

    // Estimate content uniqueness (simplified - would need external API for real analysis)
    const contentUniqueness = Math.min(100, Math.max(60, 80 + Math.random() * 20));

    return {
      wordCount: contentData.wordCount,
      readabilityScore,
      contentQuality,
      contentUniqueness,
      imageAnalysis: contentData.imageAnalysis
    };
  }

  /**
   * Calculate overall SEO score
   */
  private calculateOverallSEOScore(seoData: {
    metaTags: MetaTagAnalysis;
    headingStructure: HeadingStructure;
    socialTags: SocialMetaTags;
    technicalSEO: TechnicalSEO;
    wordCount: number;
    readabilityScore: number;
    contentQuality: string;
  }): number {
    let score = 100;

    // Meta tags scoring (30%)
    if (!seoData.metaTags.title.present) score -= 15;
    else if (!seoData.metaTags.title.optimal) score -= 5;

    if (!seoData.metaTags.description.present) score -= 15;
    else if (!seoData.metaTags.description.optimal) score -= 5;

    // Heading structure scoring (20%)
    if (!seoData.headingStructure.structureValid) score -= 10;
    if (seoData.headingStructure.h1Count !== 1) score -= 10;

    // Technical SEO scoring (25%)
    if (!seoData.technicalSEO.httpsEnabled) score -= 5;
    if (!seoData.technicalSEO.mobileFriendly) score -= 10;
    if (seoData.technicalSEO.schemaMarkup.length === 0) score -= 5;
    if (seoData.technicalSEO.linkAnalysis.internalLinks === 0) score -= 5;

    // Content quality scoring (25%)
    if (seoData.wordCount < 300) score -= 15;
    else if (seoData.wordCount < 500) score -= 10;
    else if (seoData.wordCount < 800) score -= 5;

    if (seoData.readabilityScore < 30) score -= 10;
    else if (seoData.readabilityScore < 50) score -= 5;

    return Math.max(0, Math.min(100, score));
  }

  /**
   * Calculate SEO grade from score
   */
  private calculateSEOGrade(score: number): 'A' | 'B' | 'C' | 'D' | 'F' {
    if (score >= 90) return 'A';
    if (score >= 80) return 'B';
    if (score >= 70) return 'C';
    if (score >= 60) return 'D';
    return 'F';
  }

  /**
   * Generate SEO recommendations
   */
  private generateSEORecommendations(seoData: any): string[] {
    const recommendations: string[] = [];

    // Title recommendations
    if (!seoData.metaTags.title.present) {
      recommendations.push('📝 Add a title tag to your page');
    } else if (!seoData.metaTags.title.optimal) {
      recommendations.push(`📏 Optimize title length (current: ${seoData.metaTags.title.length} chars, optimal: 30-60)`);
    }

    // Description recommendations
    if (!seoData.metaTags.description.present) {
      recommendations.push('📄 Add a meta description to your page');
    } else if (!seoData.metaTags.description.optimal) {
      recommendations.push(`📝 Optimize meta description length (current: ${seoData.metaTags.description.length} chars, optimal: 120-160)`);
    }

    // Heading structure recommendations
    if (seoData.headingStructure.h1Count === 0) {
      recommendations.push('🏷️ Add an H1 tag for better content hierarchy');
    } else if (seoData.headingStructure.h1Count > 1) {
      recommendations.push('🏷️ Use only one H1 tag per page');
    }

    if (seoData.headingStructure.issues.length > 0) {
      recommendations.push(`🔧 Fix heading structure: ${seoData.headingStructure.issues.join(', ')}`);
    }

    // Technical SEO recommendations
    if (!seoData.technicalSEO.httpsEnabled) {
      recommendations.push('🔒 Enable HTTPS for better security and SEO');
    }

    if (!seoData.technicalSEO.mobileFriendly) {
      recommendations.push('📱 Make your site mobile-friendly with responsive design');
    }

    if (seoData.technicalSEO.schemaMarkup.length === 0) {
      recommendations.push('📋 Add structured data (schema.org) for better search results');
    }

    // Content recommendations
    if (seoData.wordCount < 300) {
      recommendations.push(`📖 Add more content (current: ${seoData.wordCount} words, recommended: 300+)`);
    }

    if (seoData.readabilityScore < 50) {
      recommendations.push('📚 Improve content readability with shorter sentences and simpler words');
    }

    // Social media recommendations
    if (seoData.socialTags.completenessScore < 50) {
      recommendations.push('📱 Add Open Graph and Twitter Card tags for better social media sharing');
    }

    return recommendations;
  }

  /**
   * Estimate search visibility based on SEO score
   */
  private estimateSearchVisibility(seoScore: number, wordCount: number): number {
    let visibility = seoScore * 0.7; // Base on SEO score
    
    // Adjust for content length
    if (wordCount > 1000) visibility += 10;
    else if (wordCount < 300) visibility -= 20;
    
    return Math.max(0, Math.min(100, visibility));
  }

  /**
   * Identify key opportunity areas for improvement
   */
  private identifyOpportunityAreas(seoData: any): string[] {
    const opportunities: string[] = [];
    
    if (!seoData.metaTags.title.optimal) opportunities.push('Title Optimization');
    if (!seoData.metaTags.description.optimal) opportunities.push('Meta Description');
    if (!seoData.headingStructure.structureValid) opportunities.push('Content Structure');
    if (seoData.socialTags.completenessScore < 70) opportunities.push('Social Media Tags');
    if (!seoData.technicalSEO.mobileFriendly) opportunities.push('Mobile Optimization');
    if (seoData.wordCount < 500) opportunities.push('Content Depth');
    if (seoData.readabilityScore < 50) opportunities.push('Content Readability');
    
    return opportunities;
  }

  /**
   * Helper methods
   */
  private assessKeywordRelevance(keywords: string, title: string): boolean {
    if (!keywords || !title) return false;
    const keywordList = keywords.toLowerCase().split(',').map(k => k.trim());
    const titleWords = title.toLowerCase().split(/\s+/);
    return keywordList.some(keyword => titleWords.some(word => word.includes(keyword)));
  }

  private isValidUrl(url: string): boolean {
    try {
      new URL(url);
      return true;
    } catch {
      return false;
    }
  }

  private calculateReadabilityScore(avgWordsPerSentence: number, avgSyllablesPerWord: number): number {
    // Simplified Flesch Reading Ease Score
    // Score = 206.835 - 1.015 × (average words per sentence) - 84.6 × (average syllables per word)
    const score = 206.835 - (1.015 * avgWordsPerSentence) - (84.6 * avgSyllablesPerWord);
    return Math.max(0, Math.min(100, score));
  }

  private getDefaultSocialTags(): SocialMetaTags {
    return {
      openGraph: {},
      twitterCard: {},
      completenessScore: 0
    };
  }

  private getFallbackTechnicalSEO(url: string): TechnicalSEO {
    return {
      httpsEnabled: url.startsWith('https://'),
      sitemapPresent: false,
      robotsTxtPresent: false,
      schemaMarkup: [],
      pageSpeedScore: 75, // Default score when can't measure
      mobileFriendly: true, // Assume modern sites are mobile-friendly
      linkAnalysis: {
        internalLinks: 0,
        externalLinks: 0,
        brokenLinks: 0
      }
    };
  }
  
  private getDefaultTechnicalSEO(): TechnicalSEO {
    return {
      httpsEnabled: false,
      sitemapPresent: false,
      robotsTxtPresent: false,
      schemaMarkup: [],
      pageSpeedScore: 0,
      mobileFriendly: false,
      linkAnalysis: {
        internalLinks: 0,
        externalLinks: 0,
        brokenLinks: 0
      }
    };
  }
}
