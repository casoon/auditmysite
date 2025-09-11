/**
 * 📱 Mobile-Friendliness Analyzer
 * 
 * Comprehensive analysis of mobile usability including:
 * - Viewport & Layout (responsive design, no horizontal scrolling, safe areas)
 * - Typography & Touch Targets (font sizes, click areas, spacing)
 * - Navigation & Interactions (touch-friendly UI, focus management)
 * - Media & Images (responsive images, lazy loading, video handling)
 * - Performance (mobile-specific Core Web Vitals)
 * - Forms & Input (mobile-optimized form controls)
 * - Mobile UX (no intrusive popups, proper error handling)
 */

import { Page } from 'playwright';
import { QualityAnalysisOptions } from '../types/enhanced-metrics';

export interface MobileFriendlinessMetrics {
    overallScore: number;
    grade: string;
    viewport: ViewportAnalysis;
    typography: TypographyAnalysis;
    touchTargets: TouchTargetAnalysis;
    navigation: NavigationAnalysis;
    media: MediaAnalysis;
    performance: MobilePerformanceAnalysis;
    forms: FormAnalysis;
    ux: UserExperienceAnalysis;
    recommendations: MobileRecommendation[];
    // NEW: Desktop vs Mobile Comparison
    desktopComparison?: DesktopMobileComparison;
}

export interface DesktopMobileComparison {
    desktop: DeviceAnalysis;
    mobile: DeviceAnalysis;
    differences: ComparisonDifferences;
    recommendations: ComparisonRecommendation[];
}

export interface DeviceAnalysis {
    viewport: { width: number; height: number };
    touchTargets: { averageSize: number; compliantTargets: number; totalTargets: number };
    typography: { baseFontSize: number; lineHeight: number };
    navigation: { stickyHeaderHeight: number; hasVisibleFocusIndicators: boolean };
    performance: { lcp: number; ttfb: number; cls: number };
    usabilityScore: number;
}

export interface ComparisonDifferences {
    touchTargetSizeDifference: number;
    fontSizeImprovement: number;
    performanceImpact: number;
    usabilityGap: number;
    criticalIssues: string[];
}

export interface ComparisonRecommendation {
    category: 'viewport' | 'touch-targets' | 'typography' | 'performance' | 'navigation';
    priority: 'critical' | 'high' | 'medium' | 'low';
    issue: string;
    mobileRecommendation: string;
    desktopRecommendation: string;
    impact: string;
    difficulty: 'easy' | 'medium' | 'complex';
}

export interface ViewportAnalysis {
    hasViewportTag: boolean;
    viewportContent: string;
    isResponsive: boolean;
    hasHorizontalScroll: boolean;
    breakpointCount: number;
    hasSafeAreaInsets: boolean;
    score: number;
}

export interface TypographyAnalysis {
    baseFontSize: number;
    lineHeight: number;
    maxLineLength: number;
    isAccessibleFontSize: boolean;
    contrastScore: number;
    score: number;
}

export interface TouchTargetAnalysis {
    compliantTargets: number;
    totalTargets: number;
    averageTargetSize: number;
    minimumSpacing: number;
    violations: TouchTargetViolation[];
    score: number;
}

export interface TouchTargetViolation {
    selector: string;
    currentSize: number;
    requiredSize: number;
    spacing: number;
    recommendation: string;
}

export interface NavigationAnalysis {
    hasStickyHeader: boolean;
    stickyHeaderHeight: number;
    hasAccessibleNavigation: boolean;
    supportsKeyboardNavigation: boolean;
    hasVisibleFocusIndicators: boolean;
    score: number;
}

export interface MediaAnalysis {
    hasResponsiveImages: boolean;
    usesModernImageFormats: boolean;
    hasLazyLoading: boolean;
    videoOptimizations: VideoOptimizations;
    score: number;
}

export interface VideoOptimizations {
    hasPlaysinline: boolean;
    hasPosterImage: boolean;
    hasSubtitles: boolean;
    noAutoplayAudio: boolean;
}

export interface MobilePerformanceAnalysis {
    lcp: number;
    inp: number;
    cls: number;
    ttfb: number;
    isMobileOptimized: boolean;
    score: number;
}

export interface FormAnalysis {
    hasProperInputTypes: boolean;
    hasAutocomplete: boolean;
    labelsAboveFields: boolean;
    keyboardFriendly: boolean;
    score: number;
}

export interface UserExperienceAnalysis {
    hasIntrusiveInterstitials: boolean;
    hasProperErrorHandling: boolean;
    isOfflineFriendly: boolean;
    hasCumulativeLayoutShift: boolean;
    score: number;
}

export interface MobileRecommendation {
    category: string;
    priority: 'high' | 'medium' | 'low';
    issue: string;
    recommendation: string;
    impact: string;
}

export class MobileFriendlinessAnalyzer {
    constructor(private options: QualityAnalysisOptions = {}) {}

    /**
     * Analyze desktop vs mobile differences for comprehensive comparison
     */
    async analyzeDesktopMobileComparison(page: Page, url: string | { loc: string }): Promise<DesktopMobileComparison> {
        // Extract URL string from URL object if needed
        const urlString = (typeof url === 'object' && url.loc ? url.loc : url) as string;
        console.log(`🖥️📱 Running desktop vs mobile comparison analysis for: ${urlString}`);
        
        const startTime = Date.now();

        try {
            // Check if we need to navigate or use pre-set content
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
                console.log(`📄 Using pre-set page content for comparison analysis (${currentUrl})`);
            }

            // Step 1: Analyze in Desktop mode
            console.log('🖥️ Analyzing desktop experience...');
            await page.setViewportSize({ width: 1920, height: 1080 }); // Full HD Desktop
            await page.waitForTimeout(1000);
            const desktopAnalysis = await this.performDeviceAnalysis(page, 'desktop');

            // Step 2: Analyze in Mobile mode
            console.log('📱 Analyzing mobile experience...');
            await page.setViewportSize({ width: 375, height: 812 }); // iPhone 12 Pro
            await page.waitForTimeout(1000);
            const mobileAnalysis = await this.performDeviceAnalysis(page, 'mobile');

            // Step 3: Calculate differences and generate recommendations
            const differences = this.calculateDifferences(desktopAnalysis, mobileAnalysis);
            const recommendations = this.generateComparisonRecommendations(desktopAnalysis, mobileAnalysis, differences);

            console.log(`✅ Desktop vs Mobile comparison completed in ${Date.now() - startTime}ms`);
            console.log(`📊 Desktop Usability: ${desktopAnalysis.usabilityScore}/100, Mobile Usability: ${mobileAnalysis.usabilityScore}/100`);

            return {
                desktop: desktopAnalysis,
                mobile: mobileAnalysis,
                differences,
                recommendations
            };

        } catch (error) {
            console.error('❌ Desktop vs Mobile comparison analysis failed:', error);
            throw new Error(`Desktop vs Mobile comparison analysis failed: ${error}`);
        }
    }

    async analyzeMobileFriendliness(page: Page, url: string | { loc: string }, includeDesktopComparison: boolean = false): Promise<MobileFriendlinessMetrics> {
        // Extract URL string from URL object if needed
        const urlString = (typeof url === 'object' && url.loc ? url.loc : url) as string;
        console.log(`📱 Analyzing mobile-friendliness for: ${urlString}${includeDesktopComparison ? ' (with desktop comparison)' : ''}`);
        
        const startTime = Date.now();

        try {
            // Check if we need to navigate or use pre-set content
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
                console.log(`📄 Using pre-set page content for mobile analysis (${currentUrl})`);
            }

            // Set mobile viewport for testing
            await page.setViewportSize({ width: 375, height: 812 }); // iPhone 12 Pro size
            await page.waitForTimeout(1000);

            // Run parallel analysis
            const [
                viewport,
                typography,
                touchTargets,
                navigation,
                media,
                performance,
                forms,
                ux
            ] = await Promise.all([
                this.analyzeViewport(page),
                this.analyzeTypography(page),
                this.analyzeTouchTargets(page),
                this.analyzeNavigation(page),
                this.analyzeMedia(page),
                this.analyzeMobilePerformance(page),
                this.analyzeForms(page),
                this.analyzeUserExperience(page)
            ]);

            // Debug: Log individual component scores
            console.log('📱 Mobile component scores:', {
                viewport: viewport.score,
                typography: typography.score, 
                touchTargets: touchTargets.score,
                navigation: navigation.score,
                media: media.score,
                performance: performance.score,
                forms: forms.score,
                ux: ux.score
            });
            
            // Calculate overall score
            const overallScore = this.calculateOverallScore({
                viewport,
                typography,
                touchTargets,
                navigation,
                media,
                performance,
                forms,
                ux
            });

            const grade = this.calculateGrade(overallScore);
            const recommendations = this.generateRecommendations({
                viewport,
                typography,
                touchTargets,
                navigation,
                media,
                performance,
                forms,
                ux
            });

            // NEW: Optional desktop comparison analysis
            let desktopComparison: DesktopMobileComparison | undefined;
            if (includeDesktopComparison) {
                try {
                    console.log('\uD83D\uDDA5\uFE0F Running additional desktop comparison analysis...');
                    desktopComparison = await this.analyzeDesktopMobileComparison(page, urlString);
                } catch (error) {
                    console.warn('\u26A0\uFE0F Desktop comparison analysis failed:', error);
                    // Continue without desktop comparison rather than failing entirely
                }
            }

            console.log(`\u2705 Mobile-friendliness analysis completed in ${Date.now() - startTime}ms`);
            console.log(`\ud83d\udcf1 Mobile Score: ${overallScore}/100 (Grade: ${grade})`);
            if (desktopComparison) {
                console.log(`\ud83d\udcca Usability Gap: ${Math.abs(desktopComparison.differences.usabilityGap)} points`);
            }

            return {
                overallScore,
                grade,
                viewport,
                typography,
                touchTargets,
                navigation,
                media,
                performance,
                forms,
                ux,
                recommendations,
                desktopComparison
            };

        } catch (error) {
            console.error('❌ Mobile-friendliness analysis failed:', error);
            throw new Error(`Mobile-friendliness analysis failed: ${error}`);
        }
    }

    private async analyzeViewport(page: Page): Promise<ViewportAnalysis> {
        const viewportData = await page.evaluate(() => {
            const viewport = document.querySelector('meta[name="viewport"]');
            const viewportContent = viewport?.getAttribute('content') || '';
            
            // Check if responsive
            const isResponsive = viewportContent.includes('width=device-width');
            
            // Check for horizontal scroll
            const hasHorizontalScroll = document.documentElement.scrollWidth > window.innerWidth;
            
            // Count breakpoints (simplified - looks for common responsive patterns)
            const stylesheets = Array.from(document.styleSheets);
            let breakpointCount = 0;
            
            try {
                stylesheets.forEach(sheet => {
                    if (sheet.href && !sheet.href.includes(window.location.origin)) return;
                    const rules = Array.from(sheet.cssRules || []);
                    breakpointCount += rules.filter(rule => 
                        rule.type === CSSRule.MEDIA_RULE
                    ).length;
                });
            } catch (e) {
                // Cross-origin stylesheets or other errors
            }
            
            // Check for safe area insets
            const computedStyle = window.getComputedStyle(document.documentElement);
            const hasSafeAreaInsets = computedStyle.paddingTop.includes('env(safe-area-inset') || 
                                    computedStyle.paddingBottom.includes('env(safe-area-inset');
            
            return {
                hasViewportTag: !!viewport,
                viewportContent,
                isResponsive,
                hasHorizontalScroll,
                breakpointCount: Math.min(breakpointCount, 10), // Cap at 10 for scoring
                hasSafeAreaInsets
            };
        });

        // Calculate viewport score
        let score = 100;
        if (!viewportData.hasViewportTag) score -= 30;
        else if (!viewportData.isResponsive) score -= 25;
        if (viewportData.hasHorizontalScroll) score -= 20;
        if (viewportData.breakpointCount < 2) score -= 10;
        if (!viewportData.hasSafeAreaInsets) score -= 5;

        return {
            ...viewportData,
            score: Math.max(0, score)
        };
    }

    private async analyzeTypography(page: Page): Promise<TypographyAnalysis> {
        const typographyData = await page.evaluate(() => {
            const bodyStyle = window.getComputedStyle(document.body);
            const baseFontSize = parseFloat(bodyStyle.fontSize);
            const lineHeight = parseFloat(bodyStyle.lineHeight) || baseFontSize * 1.2;
            
            // Check line length (characters per line)
            const textElements = document.querySelectorAll('p, div, span');
            let maxLineLength = 0;
            
            textElements.forEach(element => {
                const text = element.textContent || '';
                const lines = text.split('\n');
                lines.forEach(line => {
                    if (line.length > maxLineLength) {
                        maxLineLength = line.length;
                    }
                });
            });

            // Basic contrast check (simplified)
            const color = bodyStyle.color;
            const backgroundColor = bodyStyle.backgroundColor;
            
            return {
                baseFontSize,
                lineHeight: lineHeight / baseFontSize, // Ratio
                maxLineLength,
                color,
                backgroundColor
            };
        });

        // Calculate scores
        const isAccessibleFontSize = typographyData.baseFontSize >= 16;
        const contrastScore = 85; // Simplified - would need actual contrast calculation
        
        let score = 100;
        if (!isAccessibleFontSize) score -= 20;
        if (typographyData.lineHeight < 1.4 || typographyData.lineHeight > 1.6) score -= 10;
        if (typographyData.maxLineLength > 75) score -= 10;
        if (contrastScore < 70) score -= 15;

        return {
            baseFontSize: typographyData.baseFontSize,
            lineHeight: typographyData.lineHeight,
            maxLineLength: typographyData.maxLineLength,
            isAccessibleFontSize,
            contrastScore,
            score: Math.max(0, score)
        };
    }

    private async analyzeTouchTargets(page: Page): Promise<TouchTargetAnalysis> {
        const touchTargetData = await page.evaluate(() => {
            const interactiveSelectors = [
                'button', 'a[href]', 'input[type="button"]', 'input[type="submit"]',
                '[role="button"]', '[role="link"]', '[tabindex]:not([tabindex="-1"])'
            ];
            
            const elements = document.querySelectorAll(interactiveSelectors.join(','));
            const targets: any[] = [];
            const violations: any[] = [];
            const minSize = 48; // 48px minimum touch target
            const minSpacing = 8; // 8px minimum spacing
            
            elements.forEach((element, index) => {
                const rect = element.getBoundingClientRect();
                const computedStyle = window.getComputedStyle(element);
                
                if (rect.width === 0 || rect.height === 0 || 
                    computedStyle.display === 'none' || 
                    computedStyle.visibility === 'hidden') {
                    return;
                }
                
                const size = Math.min(rect.width, rect.height);
                const isCompliant = size >= minSize;
                
                targets.push({
                    index,
                    width: rect.width,
                    height: rect.height,
                    size,
                    isCompliant,
                    selector: element.tagName.toLowerCase() + (element.id ? `#${element.id}` : '')
                });
                
                if (!isCompliant) {
                    violations.push({
                        selector: element.tagName.toLowerCase() + (element.id ? `#${element.id}` : ''),
                        currentSize: size,
                        requiredSize: minSize,
                        spacing: minSpacing, // Simplified
                        recommendation: `Increase touch target size to at least ${minSize}px`
                    });
                }
            });
            
            const compliantTargets = targets.filter(t => t.isCompliant).length;
            const averageSize = targets.length > 0 
                ? targets.reduce((sum, t) => sum + t.size, 0) / targets.length 
                : 0;
                
            return {
                compliantTargets,
                totalTargets: targets.length,
                averageTargetSize: averageSize,
                minimumSpacing: minSpacing,
                violations
            };
        });

        // Calculate score
        const complianceRate = touchTargetData.totalTargets > 0 
            ? touchTargetData.compliantTargets / touchTargetData.totalTargets 
            : 1;
        const score = Math.round(complianceRate * 100);

        return {
            ...touchTargetData,
            score
        };
    }

    private async analyzeNavigation(page: Page): Promise<NavigationAnalysis> {
        const navData = await page.evaluate(() => {
            // Check for sticky header
            const headers = document.querySelectorAll('header, nav, [role="banner"], [role="navigation"]');
            let hasStickyHeader = false;
            let stickyHeaderHeight = 0;
            
            headers.forEach(header => {
                const style = window.getComputedStyle(header);
                if (style.position === 'fixed' || style.position === 'sticky') {
                    hasStickyHeader = true;
                    stickyHeaderHeight = Math.max(stickyHeaderHeight, header.getBoundingClientRect().height);
                }
            });
            
            // Check navigation accessibility
            const navElements = document.querySelectorAll('nav, [role="navigation"]');
            const hasAccessibleNavigation = navElements.length > 0;
            
            // Check keyboard navigation support
            const focusableElements = document.querySelectorAll(
                'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
            );
            const supportsKeyboardNavigation = focusableElements.length > 0;
            
            // Check focus indicators
            let hasVisibleFocusIndicators = false;
            focusableElements.forEach(element => {
                const style = window.getComputedStyle(element, ':focus');
                if (style.outline !== 'none' && style.outline !== '0px') {
                    hasVisibleFocusIndicators = true;
                }
            });
            
            return {
                hasStickyHeader,
                stickyHeaderHeight,
                hasAccessibleNavigation,
                supportsKeyboardNavigation,
                hasVisibleFocusIndicators
            };
        });

        // Calculate score
        let score = 100;
        if (navData.stickyHeaderHeight > 812 * 0.3) score -= 15; // More than 30% of iPhone 12 Pro screen
        if (!navData.hasAccessibleNavigation) score -= 10;
        if (!navData.supportsKeyboardNavigation) score -= 15;
        if (!navData.hasVisibleFocusIndicators) score -= 10;

        return {
            ...navData,
            score: Math.max(0, score)
        };
    }

    private async analyzeMedia(page: Page): Promise<MediaAnalysis> {
        const mediaData = await page.evaluate(() => {
            // Check responsive images
            const images = document.querySelectorAll('img');
            const imagesWithSrcset = document.querySelectorAll('img[srcset]');
            const hasResponsiveImages = images.length > 0 && imagesWithSrcset.length > 0;
            
            // Check modern image formats
            const modernFormats = Array.from(images).some(img => {
                const src = img.src || img.getAttribute('data-src') || '';
                return src.includes('.webp') || src.includes('.avif');
            });
            
            // Check lazy loading
            const lazyImages = document.querySelectorAll('img[loading="lazy"]');
            const hasLazyLoading = lazyImages.length > 0;
            
            // Check video optimizations
            const videos = document.querySelectorAll('video');
            const videoOptimizations = {
                hasPlaysinline: Array.from(videos).some(v => v.hasAttribute('playsinline')),
                hasPosterImage: Array.from(videos).some(v => v.hasAttribute('poster')),
                hasSubtitles: Array.from(videos).some(v => v.querySelector('track')),
                noAutoplayAudio: Array.from(videos).every(v => !v.hasAttribute('autoplay') || v.muted)
            };
            
            return {
                hasResponsiveImages,
                usesModernImageFormats: modernFormats,
                hasLazyLoading,
                videoOptimizations,
                imageCount: images.length,
                videoCount: videos.length
            };
        });

        // Calculate score
        let score = 100;
        if (!mediaData.hasResponsiveImages && mediaData.imageCount > 0) score -= 20;
        if (!mediaData.usesModernImageFormats && mediaData.imageCount > 0) score -= 10;
        if (!mediaData.hasLazyLoading && mediaData.imageCount > 5) score -= 10;
        
        // Video scoring
        if (mediaData.videoCount > 0) {
            if (!mediaData.videoOptimizations.hasPlaysinline) score -= 5;
            if (!mediaData.videoOptimizations.hasPosterImage) score -= 5;
            if (!mediaData.videoOptimizations.noAutoplayAudio) score -= 15;
        }

        return {
            hasResponsiveImages: mediaData.hasResponsiveImages,
            usesModernImageFormats: mediaData.usesModernImageFormats,
            hasLazyLoading: mediaData.hasLazyLoading,
            videoOptimizations: mediaData.videoOptimizations,
            score: Math.max(0, score)
        };
    }

    private async analyzeMobilePerformance(page: Page): Promise<MobilePerformanceAnalysis> {
        // Get performance metrics (simplified - would integrate with existing performance collector)
        const performanceMetrics = await page.evaluate(() => {
            const navigation = performance.getEntriesByType('navigation')[0] as PerformanceNavigationTiming;
            const paintEntries = performance.getEntriesByType('paint');
            
            return {
                lcp: paintEntries.find(entry => entry.name === 'largest-contentful-paint')?.startTime || 0,
                ttfb: navigation?.responseStart - navigation?.requestStart || 0,
                domContentLoaded: navigation?.domContentLoadedEventEnd - (navigation as any)?.navigationStart || 0
            };
        });

        // Mobile-specific thresholds (stricter)
        const mobileThresholds = {
            lcp: 2000,  // 2s for mobile (vs 2.5s for desktop)
            inp: 150,   // 150ms for mobile (vs 200ms for desktop)
            cls: 0.1,   // Same as desktop
            ttfb: 300   // 300ms for mobile (vs 400ms for desktop)
        };

        const isMobileOptimized = 
            performanceMetrics.lcp <= mobileThresholds.lcp &&
            performanceMetrics.ttfb <= mobileThresholds.ttfb;

        // Calculate score based on mobile thresholds
        let score = 100;
        if (performanceMetrics.lcp > mobileThresholds.lcp) {
            score -= Math.min(30, (performanceMetrics.lcp - mobileThresholds.lcp) / 100);
        }
        if (performanceMetrics.ttfb > mobileThresholds.ttfb) {
            score -= Math.min(20, (performanceMetrics.ttfb - mobileThresholds.ttfb) / 50);
        }

        return {
            lcp: performanceMetrics.lcp,
            inp: 0, // Would need more sophisticated measurement
            cls: 0, // Would need more sophisticated measurement
            ttfb: performanceMetrics.ttfb,
            isMobileOptimized,
            score: Math.max(0, Math.round(score))
        };
    }

    private async analyzeForms(page: Page): Promise<FormAnalysis> {
        const formData = await page.evaluate(() => {
            const inputs = document.querySelectorAll('input, select, textarea');
            const forms = document.querySelectorAll('form');
            
            // Check input types
            const hasProperInputTypes = Array.from(inputs).some(input => {
                const type = input.getAttribute('type');
                return ['email', 'tel', 'number', 'url', 'date', 'time'].includes(type || '');
            });
            
            // Check autocomplete
            const hasAutocomplete = Array.from(inputs).some(input => {
                return input.hasAttribute('autocomplete');
            });
            
            // Check label positioning (simplified)
            const labels = document.querySelectorAll('label');
            const labelsAboveFields = labels.length > 0; // Simplified check
            
            // Check keyboard accessibility
            const keyboardFriendly = Array.from(inputs).every(input => {
                return !input.hasAttribute('readonly') || input.getAttribute('tabindex') !== '-1';
            });
            
            return {
                hasProperInputTypes,
                hasAutocomplete,
                labelsAboveFields,
                keyboardFriendly,
                inputCount: inputs.length,
                formCount: forms.length
            };
        });

        // Calculate score
        let score = 100;
        if (formData.inputCount > 0) {
            if (!formData.hasProperInputTypes) score -= 15;
            if (!formData.hasAutocomplete) score -= 10;
            if (!formData.labelsAboveFields) score -= 10;
            if (!formData.keyboardFriendly) score -= 15;
        }

        return {
            hasProperInputTypes: formData.hasProperInputTypes,
            hasAutocomplete: formData.hasAutocomplete,
            labelsAboveFields: formData.labelsAboveFields,
            keyboardFriendly: formData.keyboardFriendly,
            score: Math.max(0, score)
        };
    }

    private async analyzeUserExperience(page: Page): Promise<UserExperienceAnalysis> {
        const uxData = await page.evaluate(() => {
            // Check for intrusive interstitials/popups
            const possiblePopups = document.querySelectorAll(
                '[class*="popup"], [class*="modal"], [class*="overlay"], [id*="popup"], [id*="modal"]'
            );
            const hasIntrusiveInterstitials = possiblePopups.length > 0;
            
            // Check error handling (simplified)
            const errorElements = document.querySelectorAll('[class*="error"], [id*="error"]');
            const hasProperErrorHandling = errorElements.length > 0;
            
            // Check offline capability
            const isOfflineFriendly = 'serviceWorker' in navigator;
            
            // Check for layout shift indicators (simplified)
            const hasCumulativeLayoutShift = document.querySelectorAll('[style*="height: 0"], [style*="width: 0"]').length === 0;
            
            return {
                hasIntrusiveInterstitials,
                hasProperErrorHandling,
                isOfflineFriendly,
                hasCumulativeLayoutShift
            };
        });

        // Calculate score
        let score = 100;
        if (uxData.hasIntrusiveInterstitials) score -= 20;
        if (!uxData.hasProperErrorHandling) score -= 10;
        if (!uxData.isOfflineFriendly) score -= 5;
        if (!uxData.hasCumulativeLayoutShift) score -= 15;

        return {
            ...uxData,
            score: Math.max(0, score)
        };
    }

    private calculateOverallScore(analyses: {
        viewport: ViewportAnalysis;
        typography: TypographyAnalysis;
        touchTargets: TouchTargetAnalysis;
        navigation: NavigationAnalysis;
        media: MediaAnalysis;
        performance: MobilePerformanceAnalysis;
        forms: FormAnalysis;
        ux: UserExperienceAnalysis;
    }): number {
        // Debug: Log individual scores to identify NaN sources
        console.log('📊 Mobile component scores for weighted calculation:', {
            viewport: analyses.viewport.score,
            typography: analyses.typography.score,
            touchTargets: analyses.touchTargets.score,
            navigation: analyses.navigation.score,
            media: analyses.media.score,
            performance: analyses.performance.score,
            forms: analyses.forms.score,
            ux: analyses.ux.score
        });
        
        // Validate all scores are numbers
        const scores = [
            analyses.viewport.score,
            analyses.typography.score,
            analyses.touchTargets.score,
            analyses.navigation.score,
            analyses.media.score,
            analyses.performance.score,
            analyses.forms.score,
            analyses.ux.score
        ];
        
        // Check for NaN values and replace with 0
        const validatedScores = {
            viewport: isNaN(analyses.viewport.score) ? 0 : analyses.viewport.score,
            typography: isNaN(analyses.typography.score) ? 0 : analyses.typography.score,
            touchTargets: isNaN(analyses.touchTargets.score) ? 0 : analyses.touchTargets.score,
            navigation: isNaN(analyses.navigation.score) ? 0 : analyses.navigation.score,
            media: isNaN(analyses.media.score) ? 0 : analyses.media.score,
            performance: isNaN(analyses.performance.score) ? 0 : analyses.performance.score,
            forms: isNaN(analyses.forms.score) ? 0 : analyses.forms.score,
            ux: isNaN(analyses.ux.score) ? 0 : analyses.ux.score
        };
        
        if (scores.some(score => isNaN(score))) {
            console.warn('⚠️ Found NaN values in mobile analysis scores:', scores.map((score, i) => ({
                component: ['viewport', 'typography', 'touchTargets', 'navigation', 'media', 'performance', 'forms', 'ux'][i],
                score,
                isNaN: isNaN(score)
            })));
        }
        
        // Weighted scoring
        const weights = {
            viewport: 0.20,      // 20% - Critical for mobile
            typography: 0.10,    // 10% - Important but not critical
            touchTargets: 0.15,  // 15% - Very important for mobile
            navigation: 0.10,    // 10% - Important for usability
            media: 0.10,         // 10% - Important for performance
            performance: 0.20,   // 20% - Critical for mobile
            forms: 0.10,         // 10% - Important if forms exist
            ux: 0.05            // 5% - General UX considerations
        };

        const weightedScore = 
            validatedScores.viewport * weights.viewport +
            validatedScores.typography * weights.typography +
            validatedScores.touchTargets * weights.touchTargets +
            validatedScores.navigation * weights.navigation +
            validatedScores.media * weights.media +
            validatedScores.performance * weights.performance +
            validatedScores.forms * weights.forms +
            validatedScores.ux * weights.ux;
            
        console.log(`🧮 Mobile weighted calculation: ${weightedScore} (rounded: ${Math.round(weightedScore)})`);
        
        // Ensure we return a valid number between 0 and 100
        const finalScore = Math.max(0, Math.min(100, Math.round(weightedScore)));
        
        if (isNaN(finalScore)) {
            console.error('❌ Final mobile score is NaN - returning 0');
            return 0;
        }
        
        return finalScore;
    }

    private calculateGrade(score: number): string {
        if (score >= 90) return 'A';
        if (score >= 80) return 'B';
        if (score >= 70) return 'C';
        if (score >= 60) return 'D';
        return 'F';
    }

    private generateRecommendations(analyses: {
        viewport: ViewportAnalysis;
        typography: TypographyAnalysis;
        touchTargets: TouchTargetAnalysis;
        navigation: NavigationAnalysis;
        media: MediaAnalysis;
        performance: MobilePerformanceAnalysis;
        forms: FormAnalysis;
        ux: UserExperienceAnalysis;
    }): MobileRecommendation[] {
        const recommendations: MobileRecommendation[] = [];

        // Viewport recommendations
        if (!analyses.viewport.hasViewportTag) {
            recommendations.push({
                category: 'Viewport',
                priority: 'high',
                issue: 'Missing viewport meta tag',
                recommendation: 'Add <meta name="viewport" content="width=device-width, initial-scale=1">',
                impact: 'Critical for mobile responsiveness'
            });
        }

        if (analyses.viewport.hasHorizontalScroll) {
            recommendations.push({
                category: 'Viewport',
                priority: 'high',
                issue: 'Horizontal scrolling detected',
                recommendation: 'Ensure no elements are wider than the viewport',
                impact: 'Poor mobile user experience'
            });
        }

        // Typography recommendations
        if (!analyses.typography.isAccessibleFontSize) {
            recommendations.push({
                category: 'Typography',
                priority: 'medium',
                issue: 'Font size below 16px',
                recommendation: 'Use minimum 16px base font size for mobile readability',
                impact: 'Improves text legibility on mobile devices'
            });
        }

        // Touch target recommendations
        if (analyses.touchTargets.violations.length > 0) {
            recommendations.push({
                category: 'Touch Targets',
                priority: 'high',
                issue: `${analyses.touchTargets.violations.length} touch targets below 48px`,
                recommendation: 'Increase touch target size to minimum 48x48px or add padding',
                impact: 'Essential for mobile usability and accessibility'
            });
        }

        // Performance recommendations
        if (!analyses.performance.isMobileOptimized) {
            recommendations.push({
                category: 'Performance',
                priority: 'high',
                issue: 'Mobile performance thresholds not met',
                recommendation: 'Optimize for mobile-specific performance budgets (LCP < 2s, TTFB < 300ms)',
                impact: 'Critical for mobile user experience and SEO'
            });
        }

        // Media recommendations
        if (!analyses.media.hasResponsiveImages) {
            recommendations.push({
                category: 'Media',
                priority: 'medium',
                issue: 'Images not responsive',
                recommendation: 'Use srcset and sizes attributes for responsive images',
                impact: 'Improves performance and visual quality across devices'
            });
        }

        // Form recommendations
        if (!analyses.forms.hasProperInputTypes) {
            recommendations.push({
                category: 'Forms',
                priority: 'medium',
                issue: 'Input types not optimized for mobile',
                recommendation: 'Use appropriate input types (email, tel, number, etc.)',
                impact: 'Better mobile keyboard experience'
            });
        }

        // UX recommendations
        if (analyses.ux.hasIntrusiveInterstitials) {
            recommendations.push({
                category: 'User Experience',
                priority: 'high',
                issue: 'Intrusive interstitials detected',
                recommendation: 'Remove or delay popups that block content on mobile',
                impact: 'Google penalty avoidance and better UX'
            });
        }

        return recommendations;
    }

    /**
     * Perform device-specific analysis for desktop or mobile
     */
    private async performDeviceAnalysis(page: Page, device: 'desktop' | 'mobile'): Promise<DeviceAnalysis> {
        // Get viewport information
        const viewportInfo = await page.evaluate(() => ({
            width: window.innerWidth,
            height: window.innerHeight
        }));

        // Analyze touch targets/click targets
        const targetAnalysis = await page.evaluate((device) => {
            const interactiveSelectors = [
                'button', 'a[href]', 'input[type="button"]', 'input[type="submit"]',
                '[role="button"]', '[role="link"]', '[tabindex]:not([tabindex="-1"])'
            ];
            
            const elements = document.querySelectorAll(interactiveSelectors.join(','));
            const targets: any[] = [];
            let compliantCount = 0;
            const minSize = device === 'mobile' ? 48 : 24; // Mobile needs larger targets
            
            elements.forEach(element => {
                const rect = element.getBoundingClientRect();
                const computedStyle = window.getComputedStyle(element);
                
                if (rect.width === 0 || rect.height === 0 || 
                    computedStyle.display === 'none' || 
                    computedStyle.visibility === 'hidden') {
                    return;
                }
                
                const size = Math.min(rect.width, rect.height);
                if (size >= minSize) compliantCount++;
                
                targets.push({ size });
            });
            
            const averageSize = targets.length > 0 
                ? targets.reduce((sum, t) => sum + t.size, 0) / targets.length 
                : 0;
                
            return {
                averageSize,
                compliantTargets: compliantCount,
                totalTargets: targets.length
            };
        }, device);

        // Analyze typography
        const typographyAnalysis = await page.evaluate(() => {
            const bodyStyle = window.getComputedStyle(document.body);
            const baseFontSize = parseFloat(bodyStyle.fontSize);
            const lineHeight = parseFloat(bodyStyle.lineHeight) || baseFontSize * 1.2;
            
            return {
                baseFontSize,
                lineHeight: lineHeight / baseFontSize // Ratio
            };
        });

        // Analyze navigation
        const navigationAnalysis = await page.evaluate(() => {
            const headers = document.querySelectorAll('header, nav, [role="banner"], [role="navigation"]');
            let stickyHeaderHeight = 0;
            
            headers.forEach(header => {
                const style = window.getComputedStyle(header);
                if (style.position === 'fixed' || style.position === 'sticky') {
                    stickyHeaderHeight = Math.max(stickyHeaderHeight, header.getBoundingClientRect().height);
                }
            });
            
            // Check focus indicators
            const focusableElements = document.querySelectorAll(
                'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
            );
            let hasVisibleFocusIndicators = false;
            focusableElements.forEach(element => {
                const style = window.getComputedStyle(element, ':focus');
                if (style.outline !== 'none' && style.outline !== '0px') {
                    hasVisibleFocusIndicators = true;
                }
            });
            
            return {
                stickyHeaderHeight,
                hasVisibleFocusIndicators
            };
        });

        // Basic performance metrics
        const performanceMetrics = await page.evaluate(() => {
            const navigation = performance.getEntriesByType('navigation')[0] as PerformanceNavigationTiming;
            const paintEntries = performance.getEntriesByType('paint');
            
            return {
                lcp: paintEntries.find(entry => entry.name === 'largest-contentful-paint')?.startTime || 0,
                ttfb: navigation?.responseStart - navigation?.requestStart || 0,
                cls: 0 // Simplified - would need more sophisticated measurement
            };
        });

        // Calculate device-specific usability score
        const usabilityScore = this.calculateDeviceUsabilityScore({
            device,
            touchTargets: targetAnalysis,
            typography: typographyAnalysis,
            navigation: navigationAnalysis,
            performance: performanceMetrics
        });

        return {
            viewport: viewportInfo,
            touchTargets: targetAnalysis,
            typography: typographyAnalysis,
            navigation: navigationAnalysis,
            performance: performanceMetrics,
            usabilityScore
        };
    }

    /**
     * Calculate device-specific usability score
     */
    private calculateDeviceUsabilityScore(data: {
        device: 'desktop' | 'mobile';
        touchTargets: any;
        typography: any;
        navigation: any;
        performance: any;
    }): number {
        let score = 100;
        const { device, touchTargets, typography, navigation, performance } = data;

        // Touch target scoring (more critical for mobile)
        const targetCompliance = touchTargets.totalTargets > 0 
            ? touchTargets.compliantTargets / touchTargets.totalTargets 
            : 1;
        
        if (device === 'mobile') {
            score -= (1 - targetCompliance) * 30; // 30 points penalty for mobile
        } else {
            score -= (1 - targetCompliance) * 15; // 15 points penalty for desktop
        }

        // Typography scoring
        const minFontSize = device === 'mobile' ? 16 : 14;
        if (typography.baseFontSize < minFontSize) {
            score -= device === 'mobile' ? 20 : 10;
        }

        // Navigation scoring
        if (device === 'mobile' && navigation.stickyHeaderHeight > 100) {
            score -= 15; // Sticky header takes too much mobile space
        }
        if (!navigation.hasVisibleFocusIndicators) {
            score -= device === 'desktop' ? 15 : 10; // More important for desktop keyboard users
        }

        // Performance scoring (more critical for mobile)
        if (performance.lcp > (device === 'mobile' ? 2000 : 2500)) {
            score -= device === 'mobile' ? 20 : 15;
        }
        if (performance.ttfb > (device === 'mobile' ? 300 : 400)) {
            score -= device === 'mobile' ? 15 : 10;
        }

        return Math.max(0, Math.round(score));
    }

    /**
     * Calculate differences between desktop and mobile analysis
     */
    private calculateDifferences(desktop: DeviceAnalysis, mobile: DeviceAnalysis): ComparisonDifferences {
        const touchTargetSizeDifference = desktop.touchTargets.averageSize - mobile.touchTargets.averageSize;
        const fontSizeImprovement = mobile.typography.baseFontSize - desktop.typography.baseFontSize;
        const performanceImpact = mobile.performance.lcp - desktop.performance.lcp;
        const usabilityGap = desktop.usabilityScore - mobile.usabilityScore;
        
        const criticalIssues: string[] = [];
        
        // Identify critical issues
        if (Math.abs(usabilityGap) > 20) {
            criticalIssues.push(`Significant usability gap: ${Math.abs(usabilityGap)} points difference`);
        }
        
        if (touchTargetSizeDifference > 20 && mobile.touchTargets.averageSize < 48) {
            criticalIssues.push('Touch targets too small for mobile despite being adequate for desktop');
        }
        
        if (performanceImpact > 1000) {
            criticalIssues.push('Mobile performance significantly worse than desktop');
        }
        
        if (mobile.typography.baseFontSize < 16 && desktop.typography.baseFontSize >= 14) {
            criticalIssues.push('Font size acceptable for desktop but too small for mobile');
        }

        return {
            touchTargetSizeDifference,
            fontSizeImprovement,
            performanceImpact,
            usabilityGap,
            criticalIssues
        };
    }

    /**
     * Generate comparison-specific recommendations
     */
    private generateComparisonRecommendations(
        desktop: DeviceAnalysis, 
        mobile: DeviceAnalysis, 
        differences: ComparisonDifferences
    ): ComparisonRecommendation[] {
        const recommendations: ComparisonRecommendation[] = [];

        // Touch target recommendations
        if (differences.touchTargetSizeDifference > 10 && mobile.touchTargets.averageSize < 48) {
            recommendations.push({
                category: 'touch-targets',
                priority: 'critical',
                issue: 'Touch targets work on desktop but are too small for mobile',
                mobileRecommendation: 'Increase touch target size to minimum 48x48px with 8px spacing',
                desktopRecommendation: 'Current desktop interaction targets are adequate',
                impact: 'Critical for mobile usability and accessibility compliance',
                difficulty: 'medium'
            });
        }

        // Typography recommendations
        if (mobile.typography.baseFontSize < 16 && desktop.typography.baseFontSize >= 14) {
            recommendations.push({
                category: 'typography',
                priority: 'high',
                issue: 'Font size readable on desktop but too small for mobile',
                mobileRecommendation: 'Increase base font size to 16px minimum for mobile',
                desktopRecommendation: 'Consider increasing to 16px for better accessibility',
                impact: 'Improves readability and reduces eye strain on mobile devices',
                difficulty: 'easy'
            });
        }

        // Performance recommendations
        if (differences.performanceImpact > 500) {
            recommendations.push({
                category: 'performance',
                priority: 'high',
                issue: 'Mobile performance significantly worse than desktop',
                mobileRecommendation: 'Optimize images, reduce JavaScript, implement lazy loading',
                desktopRecommendation: 'Performance is acceptable but mobile optimization will help desktop too',
                impact: 'Critical for mobile user experience and SEO rankings',
                difficulty: 'complex'
            });
        }

        // Navigation recommendations
        if (mobile.navigation.stickyHeaderHeight > desktop.viewport.height * 0.15) {
            recommendations.push({
                category: 'navigation',
                priority: 'medium',
                issue: 'Sticky header takes up too much mobile screen space',
                mobileRecommendation: 'Reduce sticky header height or make it collapsible on mobile',
                desktopRecommendation: 'Desktop header size is appropriate',
                impact: 'Increases available content area on mobile devices',
                difficulty: 'medium'
            });
        }

        // Viewport recommendations
        if (Math.abs(differences.usabilityGap) > 15) {
            recommendations.push({
                category: 'viewport',
                priority: 'high',
                issue: `${differences.usabilityGap > 0 ? 'Mobile' : 'Desktop'} experience significantly worse`,
                mobileRecommendation: 'Implement responsive design patterns and mobile-first approach',
                desktopRecommendation: 'Ensure desktop layout adapts well to different screen sizes',
                impact: 'Provides consistent user experience across all devices',
                difficulty: 'complex'
            });
        }

        return recommendations;
    }
}
