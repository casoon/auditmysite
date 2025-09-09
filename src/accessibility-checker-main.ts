import { AccessibilityChecker } from './core/accessibility/accessibility-checker';
import { ContentWeightAnalyzer } from './analyzers/content-weight-analyzer';
import { EnhancedPerformanceCollector } from './analyzers/enhanced-performance-collector';
import { EnhancedSEOAnalyzer } from './analyzers/enhanced-seo-analyzer';
import { MobileFriendlinessAnalyzer } from './analyzers/mobile-friendliness-analyzer';
import { 
    AccessibilityResult
} from './types';
import {
    EnhancedPerformanceMetrics,
    EnhancedSEOMetrics,
    MobileFriendlinessMetrics,
    ContentWeight,
    ContentAnalysis,
    QualityAnalysisOptions
} from './types/enhanced-metrics';
import { chromium, Browser, Page } from 'playwright';

/**
 * Complete result interface combining all analysis types
 */
export interface AccessibilityAnalysisResult extends AccessibilityResult {
    contentWeight?: {
        contentScore: number;
        grade: string;
        resourceAnalysis: {
            html: { size: number; count: number };
            css: { size: number; count: number };
            javascript: { size: number; count: number };
            images: { size: number; count: number };
            fonts: { size: number; count: number };
        };
        contentMetrics: {
            textToCodeRatio: number;
            totalSize: number;
            contentSize: number;
        };
    };
    enhancedPerformance?: {
        performanceScore: number;
        grade: string;
        coreWebVitals: {
            fcp: { value: number; rating: string };
            lcp: { value: number; rating: string };
            cls: { value: number; rating: string };
            inp: { value: number; rating: string };
        };
        metrics: {
            ttfb: { value: number; rating: string };
            fid: { value: number; rating: string };
            tbt: { value: number; rating: string };
            si: { value: number; rating: string };
        };
    };
    enhancedSEO?: {
        seoScore: number;
        grade: string;
        metaData: {
            title: string;
            titleLength: number;
            description: string;
            descriptionLength: number;
            keywords: string;
        };
        headingStructure: {
            h1: number;
            h2: number;
            h3: number;
            h4: number;
            h5: number;
            h6: number;
        };
        contentAnalysis: {
            wordCount: number;
            readabilityScore: number;
            textToCodeRatio: number;
        };
        socialTags: {
            openGraph: number;
            twitterCard: number;
        };
        technicalSEO: {
            internalLinks: number;
            externalLinks: number;
            altTextCoverage: number;
        };
    };
    mobileFriendliness?: MobileFriendlinessMetrics;
    qualityScore?: {
        score: number;
        grade: string;
        breakdown: {
            performance: number;
            seo: number;
            accessibility: number;
            content: number;
            mobile: number;
        };
    };
}

/**
 * Main Accessibility Checker that combines accessibility analysis
 * with performance, SEO, and content weight analysis
 */
export class MainAccessibilityChecker {
    private accessibilityChecker: AccessibilityChecker;
    private contentWeightAnalyzer: ContentWeightAnalyzer;
    private performanceCollector: EnhancedPerformanceCollector;
    private seoAnalyzer: EnhancedSEOAnalyzer;
    private mobileFriendlinessAnalyzer: MobileFriendlinessAnalyzer;
    private browser: Browser | null = null;

    constructor(options: QualityAnalysisOptions = {}) {
        this.accessibilityChecker = new AccessibilityChecker();
        this.contentWeightAnalyzer = new ContentWeightAnalyzer(options);
        this.performanceCollector = new EnhancedPerformanceCollector(options);
        this.seoAnalyzer = new EnhancedSEOAnalyzer(options);
        this.mobileFriendlinessAnalyzer = new MobileFriendlinessAnalyzer();
    }

    /**
     * Initialize all analyzers and launch browser if needed
     */
    async initialize(): Promise<void> {
        try {
            // Launch browser for analysis
            this.browser = await chromium.launch({ 
                headless: true,
                args: ['--no-sandbox', '--disable-setuid-sandbox']
            });

            // Initialize accessibility checker
            await this.accessibilityChecker.initialize();

        } catch (error) {
            console.error('Error initializing accessibility checker:', error);
            throw error;
        }
    }

    /**
     * Run comprehensive analysis including accessibility, performance, SEO, and content weight
     */
    async analyze(html: string, url: string | any): Promise<AccessibilityAnalysisResult> {
        if (!this.browser) {
            throw new Error('Accessibility checker not initialized. Call initialize() first.');
        }

        try {
            // Extract URL string from object if needed
            const urlString = typeof url === 'string' ? url : (url?.loc || url?.url || String(url));
            console.log(`🔍 Testing: ${JSON.stringify(url)}`);
            
            console.log('Running accessibility analysis...');

            // For HTML content analysis, we'll use a data URI
            const dataUri = `data:text/html;charset=utf-8,${encodeURIComponent(html)}`;

            // Run standard accessibility analysis using the extracted URL string
            const accessibilityResults = await this.accessibilityChecker.testPage(urlString, {
                verbose: true,
                collectPerformanceMetrics: false,
                usePa11y: true
            });

            // Create a page for analysis
            const page = await this.browser.newPage();

            try {
                // Set content and wait for page to be ready
                await page.setContent(html, { waitUntil: 'domcontentloaded' });

                // Run analyses using the extracted URL string
                console.log('Running content weight analysis...');
                const contentWeight = await this.analyzeContentWeight(page, urlString);

                console.log('Running performance analysis...');
                const enhancedPerformance = await this.analyzePerformance(page, urlString);

                console.log('Running SEO analysis...');
                const enhancedSEO = await this.analyzeSEO(page, urlString);

                console.log('Running mobile-friendliness analysis...');
                const mobileFriendliness = await this.analyzeMobileFriendliness(page, urlString);

                console.log('Calculating quality score...');
                const qualityScore = this.calculateQualityScore(
                    contentWeight,
                    enhancedPerformance,
                    enhancedSEO,
                    mobileFriendliness,
                    accessibilityResults
                );

                return {
                    ...accessibilityResults,
                    contentWeight,
                    enhancedPerformance,
                    enhancedSEO,
                    mobileFriendliness: mobileFriendliness || undefined,
                    qualityScore
                };

            } finally {
                await page.close();
            }

        } catch (error) {
            console.error('Error during analysis:', error);
            throw error;
        }
    }

    /**
     * Analyze content weight using the ContentWeightAnalyzer
     */
    private async analyzeContentWeight(page: Page, url: string): Promise<AccessibilityAnalysisResult['contentWeight']> {
        try {
            const { contentWeight, contentAnalysis } = await this.contentWeightAnalyzer.analyzeContentWeight(page, url);
            
            const score = this.calculateContentScore(contentWeight, contentAnalysis);
            const grade = this.calculateGrade(score);

            return {
                contentScore: score,
                grade,
                resourceAnalysis: {
                    html: { size: contentWeight.html, count: 1 },
                    css: { size: contentWeight.css, count: 0 },
                    javascript: { size: contentWeight.javascript, count: 0 },
                    images: { size: contentWeight.images, count: contentAnalysis.imageCount },
                    fonts: { size: contentWeight.fonts, count: 0 }
                },
                contentMetrics: {
                    textToCodeRatio: contentAnalysis.textToCodeRatio,
                    totalSize: contentWeight.total,
                    contentSize: contentAnalysis.textContent
                }
            };
        } catch (error) {
            console.warn('Content weight analysis failed:', error);
            return this.getDefaultContentWeightResult();
        }
    }

    /**
     * Analyze performance using the PerformanceCollector
     */
    private async analyzePerformance(page: Page, url: string): Promise<AccessibilityAnalysisResult['enhancedPerformance']> {
        try {
            const metrics = await this.performanceCollector.collectEnhancedMetrics(page, url);
            
            return {
                performanceScore: metrics.performanceScore,
                grade: metrics.performanceGrade,
                coreWebVitals: {
                    fcp: { value: metrics.firstContentfulPaint, rating: this.rateMetric(metrics.firstContentfulPaint, 'fcp') },
                    lcp: { value: metrics.lcp, rating: this.rateMetric(metrics.lcp, 'lcp') },
                    cls: { value: metrics.cls, rating: this.rateMetric(metrics.cls, 'cls') },
                    inp: { value: metrics.inp, rating: this.rateMetric(metrics.inp, 'inp') }
                },
                metrics: {
                    ttfb: { value: metrics.ttfb, rating: this.rateMetric(metrics.ttfb, 'ttfb') },
                    fid: { value: metrics.fid, rating: this.rateMetric(metrics.fid, 'fid') },
                    tbt: { value: metrics.tbt, rating: this.rateMetric(metrics.tbt, 'tbt') },
                    si: { value: metrics.speedIndex, rating: this.rateMetric(metrics.speedIndex, 'si') }
                }
            };
        } catch (error) {
            console.warn('Performance analysis failed:', error);
            return this.getDefaultPerformanceResult();
        }
    }

    /**
     * Analyze SEO using the EnhancedSEOAnalyzer
     */
    private async analyzeSEO(page: Page, url: string): Promise<AccessibilityAnalysisResult['enhancedSEO']> {
        try {
            const seoMetrics = await this.seoAnalyzer.analyzeSEO(page, url);
            
            return {
                seoScore: seoMetrics.overallSEOScore,
                grade: seoMetrics.seoGrade,
                metaData: {
                    title: seoMetrics.metaTags?.title?.content || '',
                    titleLength: seoMetrics.metaTags?.title?.length || 0,
                    description: seoMetrics.metaTags?.description?.content || '',
                    descriptionLength: seoMetrics.metaTags?.description?.length || 0,
                    keywords: seoMetrics.metaTags?.keywords?.content || ''
                },
                headingStructure: {
                    h1: seoMetrics.headingStructure.h1Count,
                    h2: seoMetrics.headingStructure.h2Count,
                    h3: seoMetrics.headingStructure.h3Count,
                    h4: seoMetrics.headingStructure.h4Count,
                    h5: seoMetrics.headingStructure.h5Count,
                    h6: seoMetrics.headingStructure.h6Count
                },
                contentAnalysis: {
                    wordCount: seoMetrics.wordCount,
                    readabilityScore: seoMetrics.readabilityScore,
                    textToCodeRatio: 0 // Will be filled from content analysis
                },
                socialTags: {
                    openGraph: Object.keys(seoMetrics.socialTags?.openGraph || {}).length,
                    twitterCard: Object.keys(seoMetrics.socialTags?.twitterCard || {}).length
                },
                technicalSEO: {
                    internalLinks: seoMetrics.technicalSEO?.linkAnalysis?.internalLinks || 0,
                    externalLinks: seoMetrics.technicalSEO?.linkAnalysis?.externalLinks || 0,
                    altTextCoverage: 0 // Calculate based on image analysis
                }
            };
        } catch (error) {
            console.warn('Enhanced SEO analysis failed:', error);
            return this.getDefaultSEOResult();
        }
    }

    /**
     * Analyze mobile-friendliness using the MobileFriendlinessAnalyzer
     */
    private async analyzeMobileFriendliness(page: Page, url: string): Promise<MobileFriendlinessMetrics | null> {
        try {
            return await this.mobileFriendlinessAnalyzer.analyzeMobileFriendliness(page, url);
        } catch (error) {
            console.warn('Mobile-friendliness analysis failed:', error);
            return null;
        }
    }

    /**
     * Calculate overall quality score based on all analysis results
     */
    private calculateQualityScore(
        contentWeight: AccessibilityAnalysisResult['contentWeight'],
        performance: AccessibilityAnalysisResult['enhancedPerformance'],
        seo: AccessibilityAnalysisResult['enhancedSEO'],
        mobileFriendliness: MobileFriendlinessMetrics | null,
        accessibility: AccessibilityResult
    ): AccessibilityAnalysisResult['qualityScore'] {
        // Calculate individual scores (0-100)
        const contentScore = contentWeight?.contentScore || 0;
        const performanceScore = performance?.performanceScore || 0;
        const seoScore = seo?.seoScore || 0;
        const mobileScore = mobileFriendliness?.overallScore || 0;
        
        // Calculate accessibility score from errors
        const accessibilityIssues = accessibility.errors?.length || 0;
        const accessibilityScore = Math.max(0, 100 - (accessibilityIssues * 10));

        // Weighted average calculation (including mobile-friendliness)
        const weights = {
            performance: 0.25,
            seo: 0.2,
            accessibility: 0.25,
            content: 0.15,
            mobile: 0.15
        };

        const combinedScore = Math.round(
            performanceScore * weights.performance +
            seoScore * weights.seo +
            accessibilityScore * weights.accessibility +
            contentScore * weights.content +
            mobileScore * weights.mobile
        );

        // Determine grade
        let grade = 'F';
        if (combinedScore >= 90) grade = 'A';
        else if (combinedScore >= 80) grade = 'B';
        else if (combinedScore >= 70) grade = 'C';
        else if (combinedScore >= 60) grade = 'D';

        return {
            score: combinedScore,
            grade,
            breakdown: {
                performance: performanceScore,
                seo: seoScore,
                accessibility: accessibilityScore,
                content: contentScore,
                mobile: mobileScore
            }
        };
    }

    /**
     * Helper methods
     */
    private calculateContentScore(weight: ContentWeight, analysis: ContentAnalysis): number {
        // Simple scoring based on text-to-code ratio and content size
        const ratioScore = Math.min(analysis.textToCodeRatio * 100, 50);
        const sizeScore = weight.total < 1000000 ? 50 : Math.max(0, 50 - (weight.total - 1000000) / 100000);
        return Math.round(ratioScore + sizeScore);
    }

    private calculateGrade(score: number): string {
        if (score >= 90) return 'A';
        if (score >= 80) return 'B';
        if (score >= 70) return 'C';
        if (score >= 60) return 'D';
        return 'F';
    }

    private rateMetric(value: number, metricType: string): string {
        // Simplified rating system - you can make this more sophisticated
        const thresholds: { [key: string]: { good: number; poor: number } } = {
            fcp: { good: 1800, poor: 3000 },
            lcp: { good: 2500, poor: 4000 },
            cls: { good: 0.1, poor: 0.25 },
            inp: { good: 200, poor: 500 },
            ttfb: { good: 800, poor: 1800 },
            fid: { good: 100, poor: 300 },
            tbt: { good: 200, poor: 600 },
            si: { good: 3400, poor: 5800 }
        };

        const threshold = thresholds[metricType];
        if (!threshold) return 'unknown';

        if (metricType === 'cls') {
            return value <= threshold.good ? 'good' : (value <= threshold.poor ? 'needs-improvement' : 'poor');
        } else {
            return value <= threshold.good ? 'good' : (value <= threshold.poor ? 'needs-improvement' : 'poor');
        }
    }

    /**
     * Get default content weight result for fallback
     */
    private getDefaultContentWeightResult(): AccessibilityAnalysisResult['contentWeight'] {
        return {
            contentScore: 0,
            grade: 'N/A',
            resourceAnalysis: {
                html: { size: 0, count: 1 },
                css: { size: 0, count: 0 },
                javascript: { size: 0, count: 0 },
                images: { size: 0, count: 0 },
                fonts: { size: 0, count: 0 }
            },
            contentMetrics: {
                textToCodeRatio: 0,
                totalSize: 0,
                contentSize: 0
            }
        };
    }

    /**
     * Get default performance result for fallback
     */
    private getDefaultPerformanceResult(): AccessibilityAnalysisResult['enhancedPerformance'] {
        return {
            performanceScore: 0,
            grade: 'N/A',
            coreWebVitals: {
                fcp: { value: 0, rating: 'poor' },
                lcp: { value: 0, rating: 'poor' },
                cls: { value: 0, rating: 'poor' },
                inp: { value: 0, rating: 'poor' }
            },
            metrics: {
                ttfb: { value: 0, rating: 'poor' },
                fid: { value: 0, rating: 'poor' },
                tbt: { value: 0, rating: 'poor' },
                si: { value: 0, rating: 'poor' }
            }
        };
    }

    /**
     * Get default SEO result for fallback
     */
    private getDefaultSEOResult(): AccessibilityAnalysisResult['enhancedSEO'] {
        return {
            seoScore: 0,
            grade: 'N/A',
            metaData: {
                title: '',
                titleLength: 0,
                description: '',
                descriptionLength: 0,
                keywords: ''
            },
            headingStructure: {
                h1: 0,
                h2: 0,
                h3: 0,
                h4: 0,
                h5: 0,
                h6: 0
            },
            contentAnalysis: {
                wordCount: 0,
                readabilityScore: 0,
                textToCodeRatio: 0
            },
            socialTags: {
                openGraph: 0,
                twitterCard: 0
            },
            technicalSEO: {
                internalLinks: 0,
                externalLinks: 0,
                altTextCoverage: 0
            }
        };
    }

    /**
     * Clean up resources
     */
    async cleanup(): Promise<void> {
        try {
            // Cleanup accessibility checker
            await this.accessibilityChecker.cleanup();

            // Close browser
            if (this.browser) {
                await this.browser.close();
                this.browser = null;
            }
        } catch (error) {
            console.error('Error during cleanup:', error);
        }
    }
}
