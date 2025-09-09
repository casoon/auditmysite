import { Page, Browser, BrowserContext } from 'playwright';
import { AccessibilityResult, TestOptions } from '@core/types';
import { Chrome135Optimizer, PerformanceOptimizationResults } from '../performance/chrome135-optimizer';
import { Html5ElementsChecker, Html5ElementsAnalysis } from './html5-elements-checker';
import { AriaRulesAnalyzer, AriaAnalysisResults } from './aria-rules-analyzer';
import { WebVitalsCollector } from '@core/performance';
import { ContentWeightAnalyzer } from '../../analyzers/content-weight-analyzer';
import { EnhancedPerformanceCollector } from '../../analyzers/enhanced-performance-collector';
import { EnhancedSEOAnalyzer } from '../../analyzers/enhanced-seo-analyzer';
import { MobileFriendlinessAnalyzer } from '../../analyzers/mobile-friendliness-analyzer';
import { ContentWeight, ContentAnalysis, EnhancedPerformanceMetrics, EnhancedSEOMetrics, MobileFriendlinessMetrics } from '../../types/enhanced-metrics';
import { BrowserManager } from '../browser';

/**
 * Enhanced Test Options for v1.3
 */
export interface EnhancedTestOptions extends TestOptions {
  // New v1.3 features
  modernHtml5?: boolean;
  ariaEnhanced?: boolean;
  chrome135Features?: boolean;
  semanticAnalysis?: boolean;
  
  // Performance optimizations
  enableChrome135Optimizations?: boolean;
  optimizeAccessibilityTree?: boolean;
  enhancedDialogSupport?: boolean;
  
  // NEW: Content weight and enhanced analysis options
  contentWeightAnalysis?: boolean;
  enhancedPerformanceAnalysis?: boolean;
  enhancedSeoAnalysis?: boolean;
  mobileFriendlinessAnalysis?: boolean;
  enhancedAnalysis?: boolean; // Enable all enhanced features
  
  // NEW: Desktop vs Mobile Comparison
  includeDesktopMobileComparison?: boolean;
}

/**
 * Enhanced Accessibility Results for v1.3
 */
export interface EnhancedAccessibilityResult extends AccessibilityResult {
  // New v1.3 analysis results
  html5Analysis?: Html5ElementsAnalysis;
  ariaAnalysis?: AriaAnalysisResults;
  semanticScore?: number;
  
  // Performance optimization results
  chrome135Optimizations?: PerformanceOptimizationResults;
  performanceResults?: any; // Add performanceResults property
  
  // NEW: Enhanced analyzer results
  contentWeight?: ContentWeight;
  contentAnalysis?: ContentAnalysis;
  enhancedPerformance?: EnhancedPerformanceMetrics;
  enhancedSeo?: EnhancedSEOMetrics;
  mobileFriendliness?: MobileFriendlinessMetrics;
  
  // Enhanced recommendations
  enhancedRecommendations?: string[];
  modernFeaturesDetected?: string[];
  
  // Compliance levels
  complianceLevel?: 'basic' | 'enhanced' | 'comprehensive';
  futureReadiness?: number; // 0-100 score for modern web standards
}

/**
 * Enhanced Accessibility Checker with v1.3 Features
 * Integrates HTML5, ARIA, Chrome 135 optimizations, content weight, performance, and SEO analysis
 */
export class EnhancedAccessibilityChecker {
  private chrome135Optimizer: Chrome135Optimizer;
  private html5Checker: Html5ElementsChecker;
  private ariaAnalyzer: AriaRulesAnalyzer;
  private webVitalsCollector: WebVitalsCollector;
  
  // NEW: Enhanced analyzers
  private contentWeightAnalyzer: ContentWeightAnalyzer;
  private enhancedPerformanceCollector: EnhancedPerformanceCollector;
  private enhancedSeoAnalyzer: EnhancedSEOAnalyzer;
  private mobileFriendlinessAnalyzer: MobileFriendlinessAnalyzer;
  private browserManager?: BrowserManager;

  constructor() {
    this.chrome135Optimizer = new Chrome135Optimizer();
    this.html5Checker = new Html5ElementsChecker();
    this.ariaAnalyzer = new AriaRulesAnalyzer();
    this.webVitalsCollector = new WebVitalsCollector();
    
    // Initialize new analyzers
    this.contentWeightAnalyzer = new ContentWeightAnalyzer();
    this.enhancedPerformanceCollector = new EnhancedPerformanceCollector();
    this.enhancedSeoAnalyzer = new EnhancedSEOAnalyzer();
    this.mobileFriendlinessAnalyzer = new MobileFriendlinessAnalyzer();
  }

  /**
   * Initialize enhanced analyzers with browser manager
   */
  async initialize(browserManager: BrowserManager): Promise<void> {
    this.browserManager = browserManager;
    
    // Store browser manager reference for analyzers
    // Note: These analyzers don't have initialize methods, they work directly with pages
  }

  /**
   * Cleanup enhanced analyzers
   */
  async cleanup(): Promise<void> {
    // These analyzers don't require cleanup
    // They work directly with pages provided to them
  }

  /**
   * Enhanced page testing with v1.3 features
   */
  async testPageEnhanced(
    page: Page,
    url: string,
    options: EnhancedTestOptions = {}
  ): Promise<EnhancedAccessibilityResult> {
    const startTime = Date.now();
    const result: EnhancedAccessibilityResult = {
      url,
      title: "",
      imagesWithoutAlt: 0,
      buttonsWithoutLabel: 0,
      headingsCount: 0,
      errors: [],
      warnings: [],
      passed: true,
      duration: 0,
      enhancedRecommendations: [],
      modernFeaturesDetected: [],
      complianceLevel: 'basic',
      futureReadiness: 0
    };

    try {
      // Apply Chrome 135 optimizations if enabled
      if (options.chrome135Features && options.enableChrome135Optimizations) {
        await this.chrome135Optimizer.optimizePage(page);
        result.chrome135Optimizations = await this.chrome135Optimizer.generateOptimizationReport(page);
      }

      // Navigate to page
      await page.goto(url, {
        waitUntil: options.waitUntil || 'domcontentloaded',
        timeout: options.timeout || 30000,
      });

      // Wait for dynamic content
      if (options.wait) {
        await page.waitForTimeout(options.wait);
      }

      // Basic accessibility checks
      result.title = await page.title();
      result.imagesWithoutAlt = await page.locator('img:not([alt])').count();
      result.buttonsWithoutLabel = await page.locator('button:not([aria-label])').filter({ hasText: '' }).count();
      result.headingsCount = await page.locator('h1, h2, h3, h4, h5, h6').count();

      // Enhanced HTML5 Analysis
      if (options.modernHtml5) {
        try {
          result.html5Analysis = await this.html5Checker.analyzeHtml5Elements(page);
          
          if (result.html5Analysis.modernHtml5Usage) {
            result.modernFeaturesDetected?.push('Modern HTML5 Elements');
          }
          
          // Add HTML5 specific errors/warnings
          if (result.html5Analysis.summaryWithoutName > 0) {
            result.errors.push(`${result.html5Analysis.summaryWithoutName} <summary> elements lack accessible names`);
          }
          
          result.html5Analysis.dialogAccessibilityIssues.forEach(issue => {
            result.warnings.push(issue);
          });
          
        } catch (error) {
          result.warnings.push('HTML5 analysis failed: ' + String(error));
        }
      }

      // Enhanced ARIA Analysis
      if (options.ariaEnhanced) {
        try {
          result.ariaAnalysis = await this.ariaAnalyzer.analyzeAriaUsage(page);
          
          // Add ARIA-specific errors/warnings based on impact
          if (result.ariaAnalysis.impactBreakdown.critical > 0) {
            result.errors.push(`${result.ariaAnalysis.impactBreakdown.critical} critical ARIA issues found`);
          }
          
          if (result.ariaAnalysis.impactBreakdown.serious > 0) {
            result.errors.push(`${result.ariaAnalysis.impactBreakdown.serious} serious ARIA issues found`);
          }
          
          if (result.ariaAnalysis.impactBreakdown.moderate > 0) {
            result.warnings.push(`${result.ariaAnalysis.impactBreakdown.moderate} moderate ARIA issues found`);
          }
          
          if (result.ariaAnalysis.enhancedFeatures.modernAriaSupport) {
            result.modernFeaturesDetected?.push('Modern ARIA Features');
          }
          
        } catch (error) {
          result.warnings.push('ARIA analysis failed: ' + String(error));
        }
      }

      // Semantic Analysis and Scoring
      if (options.semanticAnalysis) {
        result.semanticScore = await this.calculateSemanticScore(result);
        result.complianceLevel = this.determineComplianceLevel(result);
        result.futureReadiness = this.calculateFutureReadiness(result);
      }

      // Performance Metrics Collection
      if (options.collectPerformanceMetrics) {
        try {
          const performanceResults = await this.webVitalsCollector.collectMetrics(page);
          result.performanceResults = performanceResults;
          
          if (performanceResults.score >= 75) {
            result.modernFeaturesDetected?.push('Good Performance Metrics');
          }
        } catch (error) {
          result.warnings.push('Performance metrics collection failed: ' + String(error));
        }
      }

      // NEW: Content Weight Analysis
      if (options.contentWeightAnalysis || options.enhancedAnalysis) {
        try {
          const contentWeightResult = await this.contentWeightAnalyzer.analyzeContentWeight(page, url);
          result.contentWeight = contentWeightResult.contentWeight;
          result.contentAnalysis = contentWeightResult.contentAnalysis;
          
          // Add content quality warnings
          if (result.contentAnalysis.textToCodeRatio < 0.1) {
            result.warnings.push('Low content-to-code ratio detected');
          }
          if (result.contentWeight.total > 1024 * 1024) { // > 1MB
            result.warnings.push('Large page size may impact accessibility');
          }
        } catch (error) {
          result.warnings.push('Content weight analysis failed: ' + String(error));
        }
      }

      // NEW: Enhanced Performance Analysis
      if (options.enhancedPerformanceAnalysis || options.enhancedAnalysis) {
        try {
          const enhancedPerformanceData = await this.enhancedPerformanceCollector.collectEnhancedMetrics(page, url);
          result.enhancedPerformance = enhancedPerformanceData;
          
          // Add performance-based warnings
          if (enhancedPerformanceData.performanceScore < 75) {
            result.warnings.push(`Performance score (${enhancedPerformanceData.performanceScore}) below recommended threshold`);
          }
          if (enhancedPerformanceData.lcp > 2500) {
            result.warnings.push('Large Contentful Paint exceeds recommended time');
          }
          if (enhancedPerformanceData.cls > 0.1) {
            result.warnings.push('Cumulative Layout Shift exceeds recommended threshold');
          }
        } catch (error) {
          result.warnings.push('Enhanced performance analysis failed: ' + String(error));
        }
      }

      // NEW: Enhanced SEO Analysis
      if (options.enhancedSeoAnalysis || options.enhancedAnalysis) {
        try {
          const enhancedSeoData = await this.enhancedSeoAnalyzer.analyzeSEO(page, url);
          result.enhancedSeo = enhancedSeoData;
          
          // Add SEO-based warnings based on scores
          if (enhancedSeoData.overallSEOScore < 50) {
            result.warnings.push(`SEO score (${enhancedSeoData.overallSEOScore}) is below recommended threshold`);
          }
          
          // Add specific SEO issues
          if (!enhancedSeoData.metaTags.title.present) {
            result.errors.push('SEO: Missing page title');
          }
          if (!enhancedSeoData.metaTags.description.present) {
            result.warnings.push('SEO: Missing meta description');
          }
          
          // Check for accessibility-related SEO issues
          if (!enhancedSeoData.metaTags.title.present || !enhancedSeoData.metaTags.title.content) {
            result.errors.push('Missing page title affects both SEO and accessibility');
          }
          if (!enhancedSeoData.metaTags.description.present || !enhancedSeoData.metaTags.description.content) {
            result.warnings.push('Missing meta description affects discoverability');
          }
          
        } catch (error) {
          result.warnings.push('Enhanced SEO analysis failed: ' + String(error));
        }
      }

      // NEW: Mobile-Friendliness Analysis
      if (options.mobileFriendlinessAnalysis || options.enhancedAnalysis) {
        try {
          // Check if desktop comparison is requested
          const includeDesktopComparison = options.includeDesktopMobileComparison || false;
          const mobileFriendlinessData = await this.mobileFriendlinessAnalyzer.analyzeMobileFriendliness(page, url, includeDesktopComparison);
          result.mobileFriendliness = mobileFriendlinessData;
          
          // Add mobile-friendliness-based warnings
          if (mobileFriendlinessData.overallScore < 70) {
            result.warnings.push(`Mobile-friendliness score (${mobileFriendlinessData.overallScore}) below recommended threshold`);
          }
          
          // Add specific mobile issues
          if (!mobileFriendlinessData.viewport.hasViewportTag) {
            result.errors.push('Mobile: Missing viewport meta tag');
          }
          
          if (mobileFriendlinessData.viewport.hasHorizontalScroll) {
            result.warnings.push('Mobile: Horizontal scrolling detected');
          }
          
          if (mobileFriendlinessData.touchTargets.violations.length > 0) {
            result.warnings.push(`Mobile: ${mobileFriendlinessData.touchTargets.violations.length} touch targets too small`);
          }
          
          if (!mobileFriendlinessData.typography.isAccessibleFontSize) {
            result.warnings.push('Mobile: Font size too small for mobile devices');
          }
          
          // Add mobile-friendliness as a modern feature if score is good
          if (mobileFriendlinessData.overallScore >= 80) {
            result.modernFeaturesDetected?.push('Mobile-Optimized Design');
          }
          
        } catch (error) {
          result.warnings.push('Mobile-friendliness analysis failed: ' + String(error));
        }
      }

      // Generate Enhanced Recommendations
      result.enhancedRecommendations = this.generateEnhancedRecommendations(result, options);

      // Basic error checks for overall pass/fail
      if (result.errors.length > 0) {
        result.passed = false;
      }

      if (result.headingsCount === 0) {
        result.errors.push('No headings found - page lacks proper structure');
        result.passed = false;
      }

    } catch (error) {
      result.errors.push(`Enhanced accessibility test failed: ${String(error)}`);
      result.passed = false;
    } finally {
      result.duration = Date.now() - startTime;
    }

    return result;
  }

  /**
   * Calculate semantic score based on all analysis results
   */
  private async calculateSemanticScore(result: EnhancedAccessibilityResult): Promise<number> {
    let score = 0;
    let maxScore = 0;

    // Base semantic score from structure
    maxScore += 25;
    if (result.headingsCount > 0) score += 15;
    if (result.headingsCount >= 3) score += 10; // Good heading hierarchy

    // HTML5 semantic contribution
    if (result.html5Analysis) {
      maxScore += 25;
      score += (result.html5Analysis.semanticStructureScore / 100) * 25;
    }

    // ARIA contribution
    if (result.ariaAnalysis) {
      maxScore += 25;
      score += (result.ariaAnalysis.ariaScore / 100) * 25;
    }

    // Modern features bonus
    if (result.modernFeaturesDetected && result.modernFeaturesDetected.length > 0) {
      maxScore += 15;
      score += Math.min(result.modernFeaturesDetected.length * 5, 15);
    }

    // Performance contribution
    if (result.performanceResults) {
      maxScore += 10;
      score += (result.performanceResults.score / 100) * 10;
    }

    // NEW: Enhanced performance contribution
    if (result.enhancedPerformance) {
      maxScore += 15;
      score += (result.enhancedPerformance.performanceScore / 100) * 15;
    }

    // NEW: Content quality contribution
    if (result.contentAnalysis) {
      maxScore += 10;
      // Good content-to-code ratio indicates semantic quality
      score += Math.min(result.contentAnalysis.textToCodeRatio * 100, 10);
    }

    // NEW: SEO semantic contribution
    if (result.enhancedSeo) {
      maxScore += 15;
      score += (result.enhancedSeo.overallSEOScore / 100) * 15;
      // Bonus for good content structure
      if (result.enhancedSeo.readabilityScore > 70) {
        maxScore += 5;
        score += 5;
      }
    }

    // NEW: Mobile-friendliness contribution
    if (result.mobileFriendliness) {
      maxScore += 10;
      score += (result.mobileFriendliness.overallScore / 100) * 10;
    }

    return maxScore > 0 ? Math.round((score / maxScore) * 100) : 0;
  }

  /**
   * Determine compliance level based on analysis
   */
  private determineComplianceLevel(result: EnhancedAccessibilityResult): 'basic' | 'enhanced' | 'comprehensive' {
    const hasModernFeatures = (result.modernFeaturesDetected?.length || 0) > 0;
    const hasGoodSemantics = (result.semanticScore || 0) >= 70;
    const hasLowErrorCount = result.errors.length <= 2;
    const hasAriaOptimization = result.ariaAnalysis && result.ariaAnalysis.ariaScore >= 80;
    const hasHtml5Optimization = result.html5Analysis && result.html5Analysis.semanticStructureScore >= 70;

    if (hasModernFeatures && hasGoodSemantics && hasLowErrorCount && hasAriaOptimization && hasHtml5Optimization) {
      return 'comprehensive';
    } else if (hasGoodSemantics && hasLowErrorCount && (hasAriaOptimization || hasHtml5Optimization)) {
      return 'enhanced';
    } else {
      return 'basic';
    }
  }

  /**
   * Calculate future readiness score
   */
  private calculateFutureReadiness(result: EnhancedAccessibilityResult): number {
    let score = 0;

    // Modern HTML5 usage
    if (result.html5Analysis?.modernHtml5Usage) score += 25;
    if (result.html5Analysis && result.html5Analysis.semanticStructureScore >= 80) score += 15;

    // ARIA modern features
    if (result.ariaAnalysis?.enhancedFeatures.modernAriaSupport) score += 20;
    if (result.ariaAnalysis?.enhancedFeatures.descendantLabeling) score += 10;

    // Chrome 135 compatibility
    if (result.chrome135Optimizations?.chrome135Features.enhancedAccessibilityTree) score += 15;
    if (result.chrome135Optimizations?.chrome135Features.improvedDialogSupport) score += 10;

    // Performance readiness
    if (result.performanceResults && result.performanceResults.score >= 75) score += 5;

    return Math.min(score, 100);
  }

  /**
   * Generate enhanced recommendations based on all analysis
   */
  private generateEnhancedRecommendations(
    result: EnhancedAccessibilityResult,
    options: EnhancedTestOptions
  ): string[] {
    const recommendations: string[] = [];

    // Basic accessibility recommendations
    if (result.imagesWithoutAlt > 0) {
      recommendations.push(`Add alt attributes to ${result.imagesWithoutAlt} images`);
    }

    if (result.buttonsWithoutLabel > 0) {
      recommendations.push(`Add aria-label to ${result.buttonsWithoutLabel} buttons without text`);
    }

    if (result.headingsCount === 0) {
      recommendations.push('Add heading structure (h1-h6) for better document outline');
    }

    // HTML5 recommendations
    if (result.html5Analysis) {
      recommendations.push(...result.html5Analysis.recommendations);
    }

    // ARIA recommendations
    if (result.ariaAnalysis) {
      recommendations.push(...result.ariaAnalysis.recommendations);
    }

    // Chrome 135 optimization recommendations
    if (result.chrome135Optimizations) {
      recommendations.push(...result.chrome135Optimizations.recommendations);
    }

    // NEW: Content weight recommendations (Note: ContentWeight doesn't have recommendations directly)
    if (result.contentWeight && result.contentWeight.total > 2048 * 1024) { // > 2MB
      recommendations.push('Content: Consider optimizing page size for better performance');
    }

    // NEW: Enhanced performance recommendations
    if (result.enhancedPerformance) {
      result.enhancedPerformance.recommendations.slice(0, 3).forEach((rec: any) => {
        if (rec.priority === 'high' || rec.priority === 'medium') {
          recommendations.push(`Performance: ${rec.description}`);
        }
      });
    }

    // NEW: SEO recommendations
    if (result.enhancedSeo) {
      result.enhancedSeo.recommendations.slice(0, 3).forEach((rec: any) => {
        recommendations.push(`SEO: ${rec.description}`);
      });
    }

    // NEW: Mobile-friendliness recommendations
    if (result.mobileFriendliness) {
      result.mobileFriendliness.recommendations.slice(0, 3).forEach((rec) => {
        if (rec.priority === 'high' || rec.priority === 'medium') {
          recommendations.push(`Mobile: ${rec.recommendation}`);
        }
      });
    }

    // Future readiness recommendations
    if ((result.futureReadiness || 0) < 70) {
      recommendations.push('Consider upgrading to modern HTML5 and ARIA patterns for better future compatibility');
    }

    // Performance recommendations
    if (result.performanceResults && result.performanceResults.score < 75) {
      recommendations.push('Improve Core Web Vitals for better user experience and accessibility');
    }

    // Compliance level recommendations
    if (result.complianceLevel === 'basic') {
      recommendations.push('Implement enhanced accessibility features to reach higher compliance levels');
    }

    return recommendations.slice(0, 10); // Limit to top 10 recommendations
  }

  /**
   * Apply browser context optimizations
   */
  async optimizeBrowserContext(context: BrowserContext, options: EnhancedTestOptions): Promise<void> {
    if (options.chrome135Features && options.enableChrome135Optimizations) {
      await this.chrome135Optimizer.optimizeBrowserContext(context);
    }
  }

  /**
   * Check Chrome 135 compatibility
   */
  async isChrome135Compatible(browser: Browser): Promise<boolean> {
    return this.chrome135Optimizer.isChrome135Compatible(browser);
  }

  /**
   * Get optimization summary
   */
  getOptimizationsSummary(): string[] {
    return this.chrome135Optimizer.getOptimizationsSummary();
  }

  /**
   * Generate comprehensive accessibility report
   */
  generateComprehensiveReport(result: EnhancedAccessibilityResult): string {
    const sections = [];
    
    sections.push(`🎯 Enhanced Accessibility Report for ${result.url}`);
    sections.push(`📊 Overall Status: ${result.passed ? '✅ PASSED' : '❌ FAILED'}`);
    sections.push(`⭐ Compliance Level: ${result.complianceLevel?.toUpperCase()}`);
    sections.push(`🚀 Future Readiness: ${result.futureReadiness || 0}%`);
    
    if (result.semanticScore !== undefined) {
      sections.push(`📋 Semantic Score: ${result.semanticScore}%`);
    }
    
    if (result.modernFeaturesDetected?.length) {
      sections.push(`🔥 Modern Features: ${result.modernFeaturesDetected.join(', ')}`);
    }
    
    if (result.errors.length > 0) {
      sections.push(`\n❌ Errors (${result.errors.length}):`);
      result.errors.forEach(error => sections.push(`  • ${error}`));
    }
    
    if (result.warnings.length > 0) {
      sections.push(`\n⚠️  Warnings (${result.warnings.length}):`);
      result.warnings.slice(0, 5).forEach(warning => sections.push(`  • ${warning}`));
      if (result.warnings.length > 5) {
        sections.push(`  • ... and ${result.warnings.length - 5} more warnings`);
      }
    }
    
    if (result.enhancedRecommendations?.length) {
      sections.push(`\n💡 Priority Recommendations:`);
      result.enhancedRecommendations.slice(0, 5).forEach(rec => sections.push(`  • ${rec}`));
    }
    
    return sections.join('\n');
  }

  /**
   * Test multiple pages with enhanced analysis enabled
   */
  async testMultiplePagesWithEnhancedAnalysis(
    urls: string[],
    options: EnhancedTestOptions = {}
  ): Promise<EnhancedAccessibilityResult[]> {
    if (!this.browserManager) {
      throw new Error('Enhanced accessibility checker not initialized with browser manager');
    }

    const results: EnhancedAccessibilityResult[] = [];
    const enhancedOptions: EnhancedTestOptions = {
      ...options,
      enhancedAnalysis: true, // Enable all enhanced features by default
      contentWeightAnalysis: options.contentWeightAnalysis !== false,
      enhancedPerformanceAnalysis: options.enhancedPerformanceAnalysis !== false,
      enhancedSeoAnalysis: options.enhancedSeoAnalysis !== false,
      mobileFriendlinessAnalysis: options.mobileFriendlinessAnalysis !== false,
      semanticAnalysis: true
    };

    console.log(`🚀 Enhanced accessibility testing for ${urls.length} pages`);
    console.log('📊 Enhanced features enabled:');
    console.log(`   📦 Content Weight Analysis: ${enhancedOptions.contentWeightAnalysis ? 'Yes' : 'No'}`);
    console.log(`   ⚡ Enhanced Performance: ${enhancedOptions.enhancedPerformanceAnalysis ? 'Yes' : 'No'}`);
    console.log(`   🔍 Enhanced SEO: ${enhancedOptions.enhancedSeoAnalysis ? 'Yes' : 'No'}`);
    console.log(`   📱 Mobile-Friendliness: ${enhancedOptions.mobileFriendlinessAnalysis ? 'Yes' : 'No'}`);
    console.log(`   🧠 Semantic Analysis: ${enhancedOptions.semanticAnalysis ? 'Yes' : 'No'}`);

    for (let i = 0; i < urls.length; i++) {
      const url = urls[i];
      console.log(`\n📄 Testing ${i + 1}/${urls.length}: ${url}`);
      
      try {
        const page = await this.browserManager.getPage();
        const result = await this.testPageEnhanced(page, url, enhancedOptions);
        results.push(result);
        
        console.log(`   ${result.passed ? '✅ PASSED' : '❌ FAILED'} - Semantic: ${result.semanticScore || 0}% | Compliance: ${result.complianceLevel?.toUpperCase()}`);
        
      } catch (error) {
        console.error(`   💥 Error testing ${url}: ${error}`);
        // Add error result
        const errorResult: EnhancedAccessibilityResult = {
          url,
          title: '',
          imagesWithoutAlt: 0,
          buttonsWithoutLabel: 0,
          headingsCount: 0,
          errors: [`Enhanced test failed: ${error}`],
          warnings: [],
          passed: false,
          duration: 0,
          crashed: true,
          enhancedRecommendations: [],
          modernFeaturesDetected: [],
          complianceLevel: 'basic',
          futureReadiness: 0
        };
        results.push(errorResult);
      }
    }

    // Generate summary
    const passedCount = results.filter(r => r.passed).length;
    const avgSemanticScore = results.reduce((sum, r) => sum + (r.semanticScore || 0), 0) / results.length;
    const avgFutureReadiness = results.reduce((sum, r) => sum + (r.futureReadiness || 0), 0) / results.length;
    
    console.log('\n📊 Enhanced Analysis Summary:');
    console.log('==============================');
    console.log(`✅ Passed: ${passedCount}/${results.length} (${((passedCount / results.length) * 100).toFixed(1)}%)`);
    console.log(`🧠 Average Semantic Score: ${avgSemanticScore.toFixed(1)}%`);
    console.log(`🚀 Average Future Readiness: ${avgFutureReadiness.toFixed(1)}%`);
    
    const complianceLevels = results.reduce((acc, r) => {
      acc[r.complianceLevel || 'basic'] = (acc[r.complianceLevel || 'basic'] || 0) + 1;
      return acc;
    }, {} as Record<string, number>);
    
    console.log('📋 Compliance Distribution:');
    Object.entries(complianceLevels).forEach(([level, count]) => {
      console.log(`   ${level.toUpperCase()}: ${count} pages`);
    });

    return results;
  }

  /**
   * Reset optimizer state
   */
  reset(): void {
    this.chrome135Optimizer.reset();
  }
}
