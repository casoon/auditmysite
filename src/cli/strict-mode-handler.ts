/**
 * 🚀 STRICT MODE CLI HANDLER - ENHANCED AUDIT VALIDATION
 * 
 * Diese CLI-Integration aktiviert das neue strikte Validierungssystem,
 * das vollständige Datenerfassung und konsistente Reports garantiert.
 * 
 * Flags:
 * --strict-validation: Aktiviert strikte Validierung (fail-fast bei fehlenden Daten)
 * --strict-reports: Generiert Reports nur aus vollständig validierten Daten
 * --validation-level: Setzt das Validierungslevel (basic|standard|strict)
 */

import { Command } from 'commander';
import { 
  generateStrictReport,
  generateMultipleStrictReports,
  StrictReportOptions,
  StrictReportResult
} from '../generators/strict-report-generator';
import {
  convertAndValidateAuditData,
  safeConvertAuditData,
  AuditDataAdapter
} from '../adapters/audit-data-adapter';
import {
  StrictAuditData,
  IncompleteAuditDataError,
  MissingAnalysisError
} from '../types/strict-audit-types';
import { AuditResult } from '../adapters/audit-data-adapter';
import chalk from 'chalk';

// ============================================================================
// STRICT MODE CONFIGURATION
// ============================================================================

export interface StrictModeConfig {
  /** Aktiviert strikte Validierung */
  enabled: boolean;
  
  /** Validierungslevel */
  level: 'basic' | 'standard' | 'strict';
  
  /** Ob Reports bei unvollständigen Daten generiert werden sollen */
  tolerateMissingData: boolean;
  
  /** Erforderliche Analyse-Typen */
  requiredAnalyses: string[];
  
  /** Ausgabe-Formate für strikte Reports */
  outputFormats: string[];
  
  /** Diagnose-Modus (detaillierte Validierungsausgabe) */
  diagnosticMode: boolean;
  
  /** Exit-Code bei Validierungsfehlern */
  failOnValidationErrors: boolean;
}

export const DEFAULT_STRICT_CONFIG: StrictModeConfig = {
  enabled: false,
  level: 'standard',
  tolerateMissingData: true,
  requiredAnalyses: ['accessibility', 'performance'],
  outputFormats: ['markdown'],
  diagnosticMode: false,
  failOnValidationErrors: false
};

// ============================================================================
// CLI OPTION DEFINITIONS
// ============================================================================

export function addStrictModeOptions(program: Command): void {
  program
    .option(
      '--strict-validation',
      'Enable strict data validation (fail-fast on missing data)',
      false
    )
    .option(
      '--strict-reports',
      'Generate reports only from fully validated data',
      false
    )
    .option(
      '--validation-level <level>',
      'Set validation strictness level (basic|standard|strict)',
      'standard'
    )
    .option(
      '--required-analyses <analyses>',
      'Comma-separated list of required analysis types',
      'accessibility,performance'
    )
    .option(
      '--strict-formats <formats>',
      'Comma-separated list of output formats for strict reports',
      'markdown'
    )
    .option(
      '--diagnostic-validation',
      'Enable detailed validation diagnostics',
      false
    )
    .option(
      '--fail-on-validation-errors',
      'Exit with error code if validation fails',
      false
    )
    .option(
      '--validate-only',
      'Only validate data without generating reports',
      false
    );
}

// ============================================================================
// STRICT MODE HANDLER CLASS
// ============================================================================

export class StrictModeHandler {
  private config: StrictModeConfig;
  
  constructor(options: any) {
    this.config = this.parseStrictOptions(options);
  }

  /**
   * Parse CLI options to strict mode configuration
   */
  private parseStrictOptions(options: any): StrictModeConfig {
    const config: StrictModeConfig = { ...DEFAULT_STRICT_CONFIG };

    // Enable strict mode if any strict flags are set
    config.enabled = !!(
      options.strictValidation ||
      options.strictReports ||
      options.validateOnly ||
      options.diagnosticValidation
    );

    // Set validation level
    if (options.validationLevel) {
      if (['basic', 'standard', 'strict'].includes(options.validationLevel)) {
        config.level = options.validationLevel;
      } else {
        console.warn(chalk.yellow(`Invalid validation level: ${options.validationLevel}, using 'standard'`));
      }
    }

    // Configure tolerance for missing data
    config.tolerateMissingData = !options.strictValidation && !options.failOnValidationErrors;

    // Parse required analyses
    if (options.requiredAnalyses) {
      const analyses = options.requiredAnalyses.split(',').map((a: string) => a.trim());
      const validAnalyses = ['accessibility', 'performance', 'seo', 'contentWeight', 'mobileFriendliness'];
      config.requiredAnalyses = analyses.filter((a: string) => validAnalyses.includes(a));
      
      if (config.requiredAnalyses.length === 0) {
        console.warn(chalk.yellow('No valid required analyses specified, defaulting to accessibility,performance'));
        config.requiredAnalyses = ['accessibility', 'performance'];
      }
    }

    // Parse output formats
    if (options.strictFormats) {
      const formats = options.strictFormats.split(',').map((f: string) => f.trim());
      const validFormats = ['markdown', 'html', 'json', 'csv'];
      config.outputFormats = formats.filter((f: string) => validFormats.includes(f));
      
      if (config.outputFormats.length === 0) {
        console.warn(chalk.yellow('No valid output formats specified, defaulting to markdown'));
        config.outputFormats = ['markdown'];
      }
    }

    // Set diagnostic and failure modes
    config.diagnosticMode = !!options.diagnosticValidation;
    config.failOnValidationErrors = !!options.failOnValidationErrors;

    return config;
  }

  /**
   * Main handler: Process audit results with strict validation
   */
  async handleStrictProcessing(
    auditResult: AuditResult,
    outputPath: string = './reports'
  ): Promise<{
    success: boolean;
    strictData?: StrictAuditData;
    generatedFiles: string[];
    diagnostics: string[];
    exitCode: number;
  }> {
    if (!this.config.enabled) {
      return {
        success: true,
        generatedFiles: [],
        diagnostics: ['Strict mode not enabled'],
        exitCode: 0
      };
    }

    const diagnostics: string[] = [];
    let exitCode = 0;

    try {
      // Step 1: Display configuration
      if (this.config.diagnosticMode) {
        this.displayStrictConfiguration();
      }

      // Step 2: Diagnose data completeness
      const diagnosis = this.diagnoseAuditData(auditResult);
      diagnostics.push(...diagnosis.messages);

      if (this.config.diagnosticMode) {
        this.displayDiagnostics(diagnosis);
      }

      // Step 3: Validate and convert data
      console.log(chalk.blue('🔒 Starting strict data validation...'));
      
      const conversionResult = safeConvertAuditData(auditResult);
      
      if (!conversionResult.success) {
        diagnostics.push(`Data conversion failed: ${conversionResult.error}`);
        
        if (this.config.failOnValidationErrors) {
          console.error(chalk.red(`❌ Strict validation failed: ${conversionResult.error}`));
          return {
            success: false,
            generatedFiles: [],
            diagnostics,
            exitCode: 1
          };
        }
        
        if (!this.config.tolerateMissingData) {
          throw new IncompleteAuditDataError(
            `Strict validation failed: ${conversionResult.error}`,
            ['validation_failed']
          );
        }
      }

      // Step 4: Generate reports if validation succeeded or tolerance is enabled
      const generatedFiles: string[] = [];
      let strictData: StrictAuditData | undefined;

      if (conversionResult.success) {
        strictData = conversionResult.data!;
        console.log(chalk.green('✅ Data validation successful'));
        
        // Generate reports for each requested format
        const reportPromises = this.config.outputFormats.map(format =>
          this.generateStrictReport(strictData!, format, outputPath)
        );
        
        const reportResults = await Promise.allSettled(reportPromises);
        
        reportResults.forEach((result, index) => {
          const format = this.config.outputFormats[index];
          if (result.status === 'fulfilled') {
            generatedFiles.push(...result.value.generatedFiles);
            diagnostics.push(`${format} report generated successfully`);
          } else {
            diagnostics.push(`${format} report generation failed: ${result.reason}`);
            if (this.config.failOnValidationErrors) {
              exitCode = 1;
            }
          }
        });
      } else {
        console.warn(chalk.yellow('⚠️ Proceeding with incomplete data (tolerance enabled)'));
        diagnostics.push('Report generation with incomplete data');
      }

      // Step 5: Display results
      this.displayResults(generatedFiles, diagnostics);

      return {
        success: exitCode === 0,
        strictData,
        generatedFiles,
        diagnostics,
        exitCode
      };

    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown strict processing error';
      console.error(chalk.red(`❌ Strict processing failed: ${errorMessage}`));
      
      diagnostics.push(`Strict processing error: ${errorMessage}`);
      
      return {
        success: false,
        generatedFiles: [],
        diagnostics,
        exitCode: this.config.failOnValidationErrors ? 1 : 0
      };
    }
  }

  /**
   * Validate audit data and return diagnostics
   */
  private diagnoseAuditData(auditResult: AuditResult): {
    isComplete: boolean;
    messages: string[];
    warnings: string[];
    missingAnalyses: string[];
  } {
    const diagnosis = AuditDataAdapter.diagnoseLegacyData(auditResult);
    const messages: string[] = [];

    messages.push(`📊 Data Diagnosis:`);
    messages.push(`   Complete: ${diagnosis.isComplete ? 'Yes' : 'No'}`);
    messages.push(`   Missing fields: ${diagnosis.missingFields.length}`);
    messages.push(`   Incomplete pages: ${diagnosis.pageAnalysis.length}`);
    messages.push(`   Warnings: ${diagnosis.warnings.length}`);

    if (diagnosis.missingFields.length > 0) {
      messages.push(`   Missing: ${diagnosis.missingFields.join(', ')}`);
    }

    const missingAnalyses = new Set<string>();
    diagnosis.pageAnalysis.forEach(page => {
      page.missingAnalyses.forEach(analysis => missingAnalyses.add(analysis));
    });

    return {
      isComplete: diagnosis.isComplete,
      messages,
      warnings: diagnosis.warnings,
      missingAnalyses: Array.from(missingAnalyses)
    };
  }

  /**
   * Generate strict report in specified format
   */
  private async generateStrictReport(
    strictData: StrictAuditData,
    format: string,
    outputPath: string
  ): Promise<StrictReportResult> {
    const options: Partial<StrictReportOptions> = {
      format: format as any,
      outputDir: outputPath,
      tolerateMissingData: this.config.tolerateMissingData,
      requiredAnalysisTypes: this.config.requiredAnalyses as any,
      verboseValidation: this.config.diagnosticMode,
      filename: `audit-report-strict-${format}`
    };

    // Convert strict data back to legacy format for report generation
    const legacyData = this.convertStrictDataToLegacy(strictData);
    
    return generateStrictReport(legacyData, options);
  }

  /**
   * Convert strict data back to legacy format
   */
  private convertStrictDataToLegacy(strictData: StrictAuditData): AuditResult {
    return {
      metadata: {
        version: strictData.metadata.version,
        timestamp: strictData.metadata.timestamp,
        sitemapUrl: strictData.metadata.sitemapUrl,
        toolVersion: strictData.metadata.toolVersion,
        duration: strictData.metadata.duration,
        maxPages: strictData.metadata.configuration.maxPages,
        timeout: strictData.metadata.configuration.timeout,
        standard: strictData.metadata.configuration.standard,
        features: strictData.metadata.configuration.features as any
      },
      summary: {
        totalPages: strictData.summary.totalPages,
        testedPages: strictData.summary.testedPages,
        passedPages: strictData.summary.passedPages,
        failedPages: strictData.summary.failedPages,
        crashedPages: strictData.summary.crashedPages,
        redirectPages: 0,
        totalErrors: strictData.summary.totalErrors,
        totalWarnings: strictData.summary.totalWarnings,
        averageScore: strictData.summary.averageScore,
        overallGrade: strictData.summary.overallGrade
      },
      pages: strictData.pages.map(page => ({
        url: page.url,
        title: page.title,
        status: page.status,
        duration: page.duration,
        testedAt: page.testedAt,
        accessibility: {
          score: page.accessibility.score,
          errors: page.accessibility.errors.map(e => e.message),
          warnings: page.accessibility.warnings.map(w => w.message),
          notices: page.accessibility.notices.map(n => n.message)
        },
        performance: {
          score: page.performance.score,
          grade: page.performance.grade,
          coreWebVitals: page.performance.coreWebVitals,
          issues: page.performance.issues
        },
        seo: {
          score: page.seo.score,
          grade: page.seo.grade,
          metaTags: page.seo.metaTags,
          issues: page.seo.issues,
          recommendations: page.seo.recommendations
        },
        contentWeight: {
          score: page.contentWeight.score,
          grade: page.contentWeight.grade,
          resources: page.contentWeight.resources,
          optimizations: page.contentWeight.optimizations
        },
        mobileFriendliness: {
          overallScore: page.mobileFriendliness.overallScore,
          grade: page.mobileFriendliness.grade,
          recommendations: page.mobileFriendliness.recommendations
        }
      })),
      systemPerformance: {
        memoryUsageMB: strictData.systemPerformance.memoryUsageMB
      }
    } as AuditResult;
  }

  /**
   * Display strict mode configuration
   */
  private displayStrictConfiguration(): void {
    console.log(chalk.blue('\n🔒 Strict Mode Configuration:'));
    console.log(`   Level: ${chalk.cyan(this.config.level)}`);
    console.log(`   Tolerate missing data: ${chalk.cyan(this.config.tolerateMissingData)}`);
    console.log(`   Required analyses: ${chalk.cyan(this.config.requiredAnalyses.join(', '))}`);
    console.log(`   Output formats: ${chalk.cyan(this.config.outputFormats.join(', '))}`);
    console.log(`   Fail on errors: ${chalk.cyan(this.config.failOnValidationErrors)}`);
    console.log();
  }

  /**
   * Display detailed diagnostics
   */
  private displayDiagnostics(diagnosis: any): void {
    console.log(chalk.blue('\n🔍 Validation Diagnostics:'));
    diagnosis.messages.forEach((msg: string) => console.log(`   ${msg}`));
    
    if (diagnosis.warnings.length > 0) {
      console.log(chalk.yellow('\n   Warnings:'));
      diagnosis.warnings.forEach((warning: string) => 
        console.log(chalk.yellow(`     - ${warning}`))
      );
    }
    
    if (diagnosis.missingAnalyses.length > 0) {
      console.log(chalk.yellow('\n   Missing Analysis Types:'));
      diagnosis.missingAnalyses.forEach((analysis: string) => 
        console.log(chalk.yellow(`     - ${analysis}`))
      );
    }
    console.log();
  }

  /**
   * Display final results
   */
  private displayResults(generatedFiles: string[], diagnostics: string[]): void {
    console.log(chalk.green('\n✅ Strict Processing Complete'));
    
    if (generatedFiles.length > 0) {
      console.log(chalk.green('\n📄 Generated Files:'));
      generatedFiles.forEach(file => console.log(chalk.green(`   - ${file}`)));
    }
    
    if (this.config.diagnosticMode && diagnostics.length > 0) {
      console.log(chalk.blue('\n📋 Processing Diagnostics:'));
      diagnostics.forEach(diagnostic => console.log(chalk.blue(`   - ${diagnostic}`)));
    }
    
    console.log();
  }
}

// ============================================================================
// CONVENIENCE FUNCTIONS
// ============================================================================

/**
 * High-level function to handle strict mode processing
 */
export async function handleStrictMode(
  auditResult: AuditResult,
  cliOptions: any,
  outputPath: string = './reports'
): Promise<number> {
  const handler = new StrictModeHandler(cliOptions);
  const result = await handler.handleStrictProcessing(auditResult, outputPath);
  
  return result.exitCode;
}

/**
 * Check if strict mode is enabled based on CLI options
 */
export function isStrictModeEnabled(options: any): boolean {
  return !!(
    options.strictValidation ||
    options.strictReports ||
    options.validateOnly ||
    options.diagnosticValidation ||
    options.failOnValidationErrors
  );
}

/**
 * Validate-only mode: Just validate data without generating reports
 */
export async function validateOnlyMode(auditResult: AuditResult, options: any): Promise<number> {
  console.log(chalk.blue('🔍 Validate-only mode: Checking data completeness...\n'));
  
  try {
    const diagnosis = AuditDataAdapter.diagnoseLegacyData(auditResult);
    const conversionResult = safeConvertAuditData(auditResult);
    
    // Display diagnosis
    console.log(chalk.blue('📊 Data Completeness:'));
    console.log(`   Complete: ${diagnosis.isComplete ? chalk.green('Yes') : chalk.red('No')}`);
    console.log(`   Missing fields: ${diagnosis.missingFields.length}`);
    console.log(`   Incomplete pages: ${diagnosis.pageAnalysis.length}`);
    console.log(`   Warnings: ${diagnosis.warnings.length}`);
    
    if (diagnosis.missingFields.length > 0) {
      console.log(chalk.yellow('\n   Missing Fields:'));
      diagnosis.missingFields.forEach(field => 
        console.log(chalk.yellow(`     - ${field}`))
      );
    }
    
    if (diagnosis.pageAnalysis.length > 0) {
      console.log(chalk.yellow('\n   Pages with Missing Analyses:'));
      diagnosis.pageAnalysis.forEach(page => 
        console.log(chalk.yellow(`     - ${page.url}: ${page.missingAnalyses.join(', ')}`))
      );
    }
    
    // Display conversion result
    console.log(chalk.blue('\n🔒 Strict Validation:'));
    if (conversionResult.success) {
      console.log(chalk.green('   ✅ Passed - Data can be converted to strict format'));
      console.log(`   Pages: ${conversionResult.data!.pages.length}`);
      console.log(`   Total Issues: ${conversionResult.data!.summary.totalErrors + conversionResult.data!.summary.totalWarnings}`);
    } else {
      console.log(chalk.red('   ❌ Failed - Data cannot be converted to strict format'));
      console.log(chalk.red(`   Error: ${conversionResult.error}`));
    }
    
    if (conversionResult.warnings.length > 0) {
      console.log(chalk.yellow('\n   Validation Warnings:'));
      conversionResult.warnings.forEach(warning => 
        console.log(chalk.yellow(`     - ${warning}`))
      );
    }
    
    const shouldFail = options.failOnValidationErrors && !conversionResult.success;
    const exitCode = shouldFail ? 1 : 0;
    
    console.log(chalk.blue(`\n📋 Validation complete. Exit code: ${exitCode}`));
    
    return exitCode;
    
  } catch (error) {
    console.error(chalk.red(`❌ Validation failed: ${error instanceof Error ? error.message : 'Unknown error'}`));
    return options.failOnValidationErrors ? 1 : 0;
  }
}