/**
 * Analysis Orchestrator for AuditMySite
 * 
 * Coordinates multiple analyzers in a clean, typed way with proper
 * error handling and resource management.
 */

import { Page } from 'playwright';
import { 
  IAnalysisOrchestrator, 
  AnalyzerType, 
  BaseAnalysisOptions, 
  BaseAnalysisResult,
  ILogger,
  IAnalyzerFactory
} from '../analyzers/interfaces';

export interface OrchestratorConfig {
  readonly analyzerFactory: IAnalyzerFactory;
  readonly logger: ILogger;
  readonly defaultTimeout: number;
  readonly failFast: boolean;
}

export interface AnalysisOptions extends BaseAnalysisOptions {
  readonly analyzerTypes?: AnalyzerType[];
  readonly failFast?: boolean;
  readonly concurrency?: boolean;
}

export interface AnalysisResults {
  readonly url: string;
  readonly timestamp: Date;
  readonly totalDuration: number;
  readonly results: BaseAnalysisResult[];
  readonly errors: AnalysisError[];
  readonly successful: BaseAnalysisResult[];
  readonly failed: BaseAnalysisResult[];
}

export interface AnalysisError {
  readonly analyzerType: AnalyzerType;
  readonly error: string;
  readonly duration: number;
}

/**
 * Orchestrates analysis across multiple analyzers with proper resource management
 */
export class AnalysisOrchestrator implements IAnalysisOrchestrator {
  private readonly config: OrchestratorConfig;

  constructor(config: OrchestratorConfig) {
    this.config = config;
  }

  /**
   * Run analysis for a specific page
   */
  async runAnalysis(
    page: Page,
    url: string,
    analyzerTypes: AnalyzerType[],
    options: AnalysisOptions = {}
  ): Promise<BaseAnalysisResult[]> {
    const startTime = Date.now();
    const logger = this.config.logger.child ? this.config.logger.child('orchestrator') : this.config.logger;

    logger.info(`Starting analysis for ${url}`, { 
      analyzerTypes, 
      analyzerCount: analyzerTypes.length 
    });

    const results: BaseAnalysisResult[] = [];
    const errors: AnalysisError[] = [];

    try {
      if (options.concurrency && analyzerTypes.length > 1) {
        // Run analyzers in parallel
        const analysisPromises = analyzerTypes.map(type => 
          this.runSingleAnalysis(page, url, type, options)
        );

        const settledResults = await Promise.allSettled(analysisPromises);
        
        settledResults.forEach((result, index) => {
          if (result.status === 'fulfilled') {
            results.push(result.value);
          } else {
            errors.push({
              analyzerType: analyzerTypes[index],
              error: result.reason?.message || String(result.reason),
              duration: 0
            });
          }
        });
      } else {
        // Run analyzers sequentially
        for (const analyzerType of analyzerTypes) {
          try {
            const result = await this.runSingleAnalysis(page, url, analyzerType, options);
            results.push(result);

            // Break on first failure if failFast is enabled
            if (options.failFast && !result.success) {
              logger.warn(`Stopping analysis due to failure in ${analyzerType}`, { error: result.error });
              break;
            }
          } catch (error) {
            const analysisError: AnalysisError = {
              analyzerType,
              error: error instanceof Error ? error.message : String(error),
              duration: 0
            };
            
            errors.push(analysisError);
            
            if (options.failFast) {
              logger.error(`Aborting analysis due to error in ${analyzerType}`, error);
              break;
            }
          }
        }
      }

      const totalDuration = Date.now() - startTime;
      
      logger.info(`Analysis completed for ${url}`, {
        duration: totalDuration,
        successful: results.filter(r => r.success).length,
        failed: results.filter(r => !r.success).length + errors.length
      });

      return results;

    } catch (error) {
      logger.error(`Analysis orchestration failed for ${url}`, error);
      throw new AnalysisOrchestrationError(url, error instanceof Error ? error.message : String(error));
    }
  }

  /**
   * Get available analyzers from the factory
   */
  getAvailableAnalyzers(): AnalyzerType[] {
    return this.config.analyzerFactory.getAvailableTypes();
  }

  /**
   * Run a comprehensive analysis with all available analyzers
   */
  async runComprehensiveAnalysis(
    page: Page,
    url: string,
    options: AnalysisOptions = {}
  ): Promise<AnalysisResults> {
    const startTime = Date.now();
    const availableAnalyzers = options.analyzerTypes || this.getAvailableAnalyzers();

    this.config.logger.info(`Running comprehensive analysis for ${url}`, {
      analyzers: availableAnalyzers
    });

    const results = await this.runAnalysis(page, url, availableAnalyzers, options);
    const totalDuration = Date.now() - startTime;

    return {
      url,
      timestamp: new Date(),
      totalDuration,
      results,
      errors: [],
      successful: results.filter(r => r.success),
      failed: results.filter(r => !r.success)
    };
  }

  /**
   * Validate that all required analyzers are available
   */
  validateAnalyzers(analyzerTypes: AnalyzerType[]): void {
    const availableTypes = this.getAvailableAnalyzers();
    const unavailable = analyzerTypes.filter(type => !availableTypes.includes(type));

    if (unavailable.length > 0) {
      throw new AnalyzerValidationError(unavailable);
    }
  }

  private async runSingleAnalysis(
    page: Page,
    url: string,
    analyzerType: AnalyzerType,
    options: BaseAnalysisOptions
  ): Promise<BaseAnalysisResult> {
    const startTime = Date.now();
    const logger = this.config.logger.child ? this.config.logger.child(analyzerType) : this.config.logger;

    try {
      // Get analyzer from factory
      const analyzer = this.config.analyzerFactory.createAnalyzer(analyzerType);
      
      // Initialize if needed
      if (analyzer.initialize) {
        await analyzer.initialize();
      }

      logger.debug(`Running ${analyzer.name} analysis`);

      // Run the analysis with timeout
      const analysisPromise = analyzer.analyze(page, url, options);
      const timeoutPromise = this.createTimeoutPromise(options.timeout || this.config.defaultTimeout);

      const result = await Promise.race([analysisPromise, timeoutPromise]);
      const duration = Date.now() - startTime;

      // Create standardized result
      const analysisResult: BaseAnalysisResult = {
        metadata: {
          analyzerType,
          analyzerName: analyzer.name,
          analysisTime: duration,
          timestamp: new Date(),
          url
        },
        success: true,
        ...result
      };

      logger.debug(`${analyzer.name} completed successfully`, { duration });
      return analysisResult;

    } catch (error) {
      const duration = Date.now() - startTime;
      const errorMessage = error instanceof Error ? error.message : String(error);
      
      logger.error(`${analyzerType} analysis failed`, { error: errorMessage, duration });

      // Return failed result instead of throwing
      return {
        metadata: {
          analyzerType,
          analyzerName: `${analyzerType} Analyzer`,
          analysisTime: duration,
          timestamp: new Date(),
          url
        },
        success: false,
        error: errorMessage
      };
    }
  }

  private createTimeoutPromise<T>(timeout: number): Promise<T> {
    return new Promise((_, reject) => {
      setTimeout(() => {
        reject(new Error(`Analysis timed out after ${timeout}ms`));
      }, timeout);
    });
  }
}

/**
 * Error classes for analysis orchestration
 */
export class AnalysisOrchestrationError extends Error {
  constructor(url: string, message: string) {
    super(`Analysis orchestration failed for ${url}: ${message}`);
    this.name = 'AnalysisOrchestrationError';
  }
}

export class AnalyzerValidationError extends Error {
  constructor(unavailableAnalyzers: AnalyzerType[]) {
    super(`The following analyzers are not available: ${unavailableAnalyzers.join(', ')}`);
    this.name = 'AnalyzerValidationError';
  }
}