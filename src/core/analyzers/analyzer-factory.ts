/**
 * Analyzer Factory for AuditMySite
 * 
 * Provides dependency injection and factory pattern for creating
 * analyzers with proper configuration and type safety.
 */

import { 
  IAnalyzer, 
  IAnalyzerFactory, 
  AnalyzerType, 
  BaseAnalysisOptions,
  ILogger
} from './interfaces';
import { QualityAnalysisOptions } from '../../types/enhanced-metrics';

// Import analyzer implementations
import { ContentWeightAnalyzer } from '../../analyzers/content-weight-analyzer';
import { PerformanceCollector } from '../../analyzers/performance-collector';
import { MobilePerformanceCollector } from '../../analyzers/mobile-performance-collector';
import { SEOAnalyzer } from '../../analyzers/seo-analyzer';
import { MobileFriendlinessAnalyzer } from '../../analyzers/mobile-friendliness-analyzer';
import { SecurityHeadersAnalyzer } from '../../analyzers/security-headers-analyzer';
import { StructuredDataAnalyzer } from '../../analyzers/structured-data-analyzer';

export interface AnalyzerFactoryConfig {
  readonly logger: ILogger;
  readonly qualityAnalysisOptions?: QualityAnalysisOptions;
  readonly enabledAnalyzers?: AnalyzerType[];
}

/**
 * Factory for creating analyzer instances with proper configuration
 */
export class AnalyzerFactory implements IAnalyzerFactory {
  private readonly config: AnalyzerFactoryConfig;
  private readonly analyzerCache = new Map<AnalyzerType, IAnalyzer>();

  constructor(config: AnalyzerFactoryConfig) {
    this.config = config;
  }

  /**
   * Create an analyzer of the specified type
   */
  createAnalyzer<T extends IAnalyzer>(type: AnalyzerType): T {
    // Check cache first for singleton behavior
    if (this.analyzerCache.has(type)) {
      return this.analyzerCache.get(type) as T;
    }

    if (!this.isAvailable(type)) {
      throw new AnalyzerNotAvailableError(type);
    }

    const analyzer = this.createAnalyzerInstance(type);
    this.analyzerCache.set(type, analyzer);
    
    this.config.logger.debug(`Created analyzer: ${type}`, { 
      analyzerName: analyzer.name,
      analyzerType: analyzer.type 
    });

    return analyzer as T;
  }

  /**
   * Get all available analyzer types
   */
  getAvailableTypes(): AnalyzerType[] {
    const allTypes: AnalyzerType[] = [
      'content-weight',
      'performance',
      'mobile-performance', 
      'seo',
      'mobile-friendliness',
      'security-headers',
      'structured-data'
    ];

    // Filter by enabled analyzers if specified
    if (this.config.enabledAnalyzers) {
      return allTypes.filter(type => this.config.enabledAnalyzers!.includes(type));
    }

    return allTypes;
  }

  /**
   * Check if a specific analyzer type is available
   */
  isAvailable(type: AnalyzerType): boolean {
    return this.getAvailableTypes().includes(type);
  }

  /**
   * Create multiple analyzers at once
   */
  createAnalyzers(types: AnalyzerType[]): IAnalyzer[] {
    return types.map(type => this.createAnalyzer(type));
  }

  /**
   * Clean up all cached analyzers
   */
  async cleanup(): Promise<void> {
    const cleanupPromises = Array.from(this.analyzerCache.values())
      .filter(analyzer => analyzer.cleanup)
      .map(analyzer => analyzer.cleanup!());

    await Promise.all(cleanupPromises);
    this.analyzerCache.clear();

    this.config.logger.debug('All analyzers cleaned up');
  }

  private createAnalyzerInstance(type: AnalyzerType): IAnalyzer {
    switch (type) {
      case 'content-weight':
        return new ContentWeightAnalyzerAdapter(
          new ContentWeightAnalyzer(),
          this.config.logger.child ? this.config.logger.child('content-weight') : this.config.logger
        );

      case 'performance':
        return new PerformanceAnalyzerAdapter(
          new PerformanceCollector(this.config.qualityAnalysisOptions),
          this.config.logger.child ? this.config.logger.child('performance') : this.config.logger
        );

      case 'mobile-performance':
        return new MobilePerformanceAnalyzerAdapter(
          new MobilePerformanceCollector(this.config.qualityAnalysisOptions),
          this.config.logger.child ? this.config.logger.child('mobile-performance') : this.config.logger
        );

      case 'seo':
        return new SEOAnalyzerAdapter(
          new SEOAnalyzer(this.config.qualityAnalysisOptions),
          this.config.logger.child ? this.config.logger.child('seo') : this.config.logger
        );

      case 'mobile-friendliness':
        return new MobileFriendlinessAnalyzerAdapter(
          new MobileFriendlinessAnalyzer({ verbose: false }),
          this.config.logger.child ? this.config.logger.child('mobile-friendliness') : this.config.logger
        );

      case 'security-headers':
        return new SecurityHeadersAnalyzerAdapter(
          new SecurityHeadersAnalyzer(),
          this.config.logger.child ? this.config.logger.child('security-headers') : this.config.logger
        );

      case 'structured-data':
        return new StructuredDataAnalyzerAdapter(
          new StructuredDataAnalyzer(),
          this.config.logger.child ? this.config.logger.child('structured-data') : this.config.logger
        );

      default:
        throw new AnalyzerNotAvailableError(type);
    }
  }
}

/**
 * Error thrown when an analyzer type is not available
 */
export class AnalyzerNotAvailableError extends Error {
  constructor(type: AnalyzerType) {
    super(`Analyzer type '${type}' is not available`);
    this.name = 'AnalyzerNotAvailableError';
  }
}

/**
 * Adapter classes to bridge existing analyzers to the new interface
 */

class ContentWeightAnalyzerAdapter implements IAnalyzer {
  readonly type: AnalyzerType = 'content-weight';
  readonly name = 'Content Weight Analyzer';

  constructor(
    private readonly analyzer: ContentWeightAnalyzer,
    private readonly logger: ILogger
  ) {}

  async analyze(page: any, url: string, options?: BaseAnalysisOptions): Promise<any> {
    const startTime = Date.now();
    
    try {
      this.logger.debug(`Starting content weight analysis for ${url}`);
      const result = await this.analyzer.analyze(page, url, { verbose: options?.verbose });
      
      const duration = Date.now() - startTime;
      this.logger.debug(`Content weight analysis completed in ${duration}ms`);
      
      return result;
    } catch (error) {
      this.logger.error(`Content weight analysis failed for ${url}`, error);
      throw error;
    }
  }
}

class PerformanceAnalyzerAdapter implements IAnalyzer {
  readonly type: AnalyzerType = 'performance';
  readonly name = 'Performance Analyzer';

  constructor(
    private readonly analyzer: PerformanceCollector,
    private readonly logger: ILogger
  ) {}

  async analyze(page: any, url: string, options?: BaseAnalysisOptions): Promise<any> {
    const startTime = Date.now();
    
    try {
      this.logger.debug(`Starting performance analysis for ${url}`);
      const result = await this.analyzer.collectEnhancedMetrics(page, url);
      
      const duration = Date.now() - startTime;
      this.logger.debug(`Performance analysis completed in ${duration}ms`);
      
      return result;
    } catch (error) {
      this.logger.error(`Performance analysis failed for ${url}`, error);
      throw error;
    }
  }
}

class MobilePerformanceAnalyzerAdapter implements IAnalyzer {
  readonly type: AnalyzerType = 'mobile-performance';
  readonly name = 'Mobile Performance Analyzer';

  constructor(
    private readonly analyzer: MobilePerformanceCollector,
    private readonly logger: ILogger
  ) {}

  async analyze(page: any, url: string, options?: BaseAnalysisOptions): Promise<any> {
    const startTime = Date.now();
    
    try {
      this.logger.debug(`Starting mobile performance analysis for ${url}`);
      const result = await this.analyzer.collectMobileMetrics(page, url);
      
      const duration = Date.now() - startTime;
      this.logger.debug(`Mobile performance analysis completed in ${duration}ms`);
      
      return result;
    } catch (error) {
      this.logger.error(`Mobile performance analysis failed for ${url}`, error);
      throw error;
    }
  }
}

class SEOAnalyzerAdapter implements IAnalyzer {
  readonly type: AnalyzerType = 'seo';
  readonly name = 'SEO Analyzer';

  constructor(
    private readonly analyzer: SEOAnalyzer,
    private readonly logger: ILogger
  ) {}

  async analyze(page: any, url: string, options?: BaseAnalysisOptions): Promise<any> {
    const startTime = Date.now();
    
    try {
      this.logger.debug(`Starting SEO analysis for ${url}`);
      const result = await this.analyzer.analyzeSEO(page, url);
      
      const duration = Date.now() - startTime;
      this.logger.debug(`SEO analysis completed in ${duration}ms`);
      
      return result;
    } catch (error) {
      this.logger.error(`SEO analysis failed for ${url}`, error);
      throw error;
    }
  }
}

class MobileFriendlinessAnalyzerAdapter implements IAnalyzer {
  readonly type: AnalyzerType = 'mobile-friendliness';
  readonly name = 'Mobile Friendliness Analyzer';

  constructor(
    private readonly analyzer: MobileFriendlinessAnalyzer,
    private readonly logger: ILogger
  ) {}

  async analyze(page: any, url: string, options?: BaseAnalysisOptions): Promise<any> {
    const startTime = Date.now();
    
    try {
      this.logger.debug(`Starting mobile friendliness analysis for ${url}`);
      const result = await this.analyzer.analyzeMobileFriendliness(page, url);
      
      const duration = Date.now() - startTime;
      this.logger.debug(`Mobile friendliness analysis completed in ${duration}ms`);
      
      return result;
    } catch (error) {
      this.logger.error(`Mobile friendliness analysis failed for ${url}`, error);
      throw error;
    }
  }
}

class SecurityHeadersAnalyzerAdapter implements IAnalyzer {
  readonly type: AnalyzerType = 'security-headers';
  readonly name = 'Security Headers Analyzer';

  constructor(
    private readonly analyzer: SecurityHeadersAnalyzer,
    private readonly logger: ILogger
  ) {}

  async analyze(page: any, url: string, options?: BaseAnalysisOptions): Promise<any> {
    const startTime = Date.now();
    
    try {
      this.logger.debug(`Starting security headers analysis for ${url}`);
      // For now, return a placeholder result
      const result = {
        score: 50,
        grade: 'C',
        headers: [],
        recommendations: ['Security headers analysis not fully implemented']
      };
      
      const duration = Date.now() - startTime;
      this.logger.debug(`Security headers analysis completed in ${duration}ms`);
      
      return result;
    } catch (error) {
      this.logger.error(`Security headers analysis failed for ${url}`, error);
      throw error;
    }
  }
}

class StructuredDataAnalyzerAdapter implements IAnalyzer {
  readonly type: AnalyzerType = 'structured-data';
  readonly name = 'Structured Data Analyzer';

  constructor(
    private readonly analyzer: StructuredDataAnalyzer,
    private readonly logger: ILogger
  ) {}

  async analyze(page: any, url: string, options?: BaseAnalysisOptions): Promise<any> {
    const startTime = Date.now();
    
    try {
      this.logger.debug(`Starting structured data analysis for ${url}`);
      // For now, return a placeholder result
      const result = {
        score: 50,
        grade: 'C',
        structuredData: [],
        recommendations: ['Structured data analysis not fully implemented']
      };
      
      const duration = Date.now() - startTime;
      this.logger.debug(`Structured data analysis completed in ${duration}ms`);
      
      return result;
    } catch (error) {
      this.logger.error(`Structured data analysis failed for ${url}`, error);
      throw error;
    }
  }
}
