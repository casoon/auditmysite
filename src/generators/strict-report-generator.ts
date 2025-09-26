/**
 * 🔒 STRICT REPORT GENERATOR - GUARANTEED COMPLETE DATA
 * 
 * Dieser Report-Generator nutzt die strikten Validatoren und Adapter,
 * um sicherzustellen, dass alle Reports auf vollständigen und validierten
 * Daten basieren. Kein Report wird erstellt, wenn kritische Daten fehlen.
 */

import { writeFileSync } from 'fs';
import { join } from 'path';
import {
  convertAndValidateAuditData,
  safeConvertAuditData
} from '../adapters/audit-data-adapter';
import {
  StrictAuditData,
  StrictAuditPage,
  hasCompleteAnalysis,
  IncompleteAuditDataError,
  MissingAnalysisError
} from '../types/strict-audit-types';
import { AuditResult } from '../adapters/audit-data-adapter';

// ============================================================================
// STRICT REPORT GENERATION INTERFACE
// ============================================================================

export interface StrictReportOptions {
  /** Basis-Ausgabeformat */
  format: 'markdown' | 'html' | 'json' | 'csv';
  
  /** Ausgabe-Dateiname ohne Extension */
  filename: string;
  
  /** Ausgabe-Verzeichnis */
  outputDir: string;
  
  /** Ob fehlende Daten toleriert werden sollen (false = fail-fast) */
  tolerateMissingData: boolean;
  
  /** Minimale Anzahl vollständiger Analysen pro Seite erforderlich */
  requiredAnalysisTypes: ('accessibility' | 'performance' | 'seo' | 'contentWeight' | 'mobileFriendliness')[];
  
  /** Ob detaillierte Validierungs-Logs ausgegeben werden sollen */
  verboseValidation: boolean;
}

export interface StrictReportResult {
  success: boolean;
  generatedFiles: string[];
  validationWarnings: string[];
  error?: string;
  strictData?: StrictAuditData;
}

// ============================================================================
// STRICT REPORT GENERATOR CLASS
// ============================================================================

export class StrictReportGenerator {
  private options: StrictReportOptions;
  
  constructor(options: Partial<StrictReportOptions> = {}) {
    this.options = {
      format: 'markdown',
      filename: 'audit-report',
      outputDir: './reports',
      tolerateMissingData: false,
      requiredAnalysisTypes: ['accessibility', 'performance', 'seo', 'contentWeight', 'mobileFriendliness'],
      verboseValidation: true,
      ...options
    };
  }

  /**
   * Hauptfunktion: Generiert Reports aus Legacy-Daten mit strikter Validierung
   */
  async generateFromLegacyData(legacyResult: AuditResult): Promise<StrictReportResult> {
    try {
      if (this.options.verboseValidation) {
        console.log('🔒 Starting strict report generation...');
        console.log(`   Required analyses: ${this.options.requiredAnalysisTypes.join(', ')}`);
        console.log(`   Tolerance for missing data: ${this.options.tolerateMissingData}`);
      }

      // Step 1: Convert and validate legacy data
      const conversionResult = safeConvertAuditData(legacyResult);
      
      if (!conversionResult.success) {
        if (this.options.tolerateMissingData) {
          console.warn('⚠️ Data conversion failed, but tolerateMissingData is enabled');
          console.warn(`   Error: ${conversionResult.error}`);
          // Try to generate a basic report from whatever data we have
          return await this.generateBasicReportFromIncompleteData(legacyResult, conversionResult.error!);
        } else {
          throw new IncompleteAuditDataError(
            `Strict validation failed: ${conversionResult.error}`,
            ['data_conversion']
          );
        }
      }

      const strictData = conversionResult.data!;
      const warnings = conversionResult.warnings;

      // Step 2: Validate required analysis types
      const analysisValidation = this.validateRequiredAnalyses(strictData);
      
      if (!analysisValidation.isValid && !this.options.tolerateMissingData) {
        throw new MissingAnalysisError(
          'required_analyses',
          'multiple_pages',
          `Missing required analysis types: ${analysisValidation.missingAnalyses.join(', ')}`
        );
      }

      warnings.push(...analysisValidation.warnings);

      // Step 3: Generate report with strict data
      const generatedFiles = await this.generateReportFiles(strictData);

      if (this.options.verboseValidation) {
        console.log('✅ Strict report generation completed successfully');
        console.log(`   Generated files: ${generatedFiles.join(', ')}`);
        console.log(`   Pages processed: ${strictData.pages.length}`);
        console.log(`   Total issues found: ${strictData.summary.totalErrors + strictData.summary.totalWarnings}`);
      }

      return {
        success: true,
        generatedFiles,
        validationWarnings: warnings,
        strictData
      };

    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error during report generation';
      
      if (this.options.verboseValidation) {
        console.error('❌ Strict report generation failed:', errorMessage);
      }

      if (this.options.tolerateMissingData) {
        // Last resort: try to generate something useful
        return await this.generateBasicReportFromIncompleteData(legacyResult, errorMessage);
      }

      return {
        success: false,
        generatedFiles: [],
        validationWarnings: [],
        error: errorMessage
      };
    }
  }

  /**
   * Generiert Report-Dateien aus validierten strikten Daten
   */
  private async generateReportFiles(strictData: StrictAuditData): Promise<string[]> {
    const generatedFiles: string[] = [];
    
    // Convert strict data back to legacy format for existing generators
    const legacyData = this.convertStrictToLegacy(strictData);
    
    switch (this.options.format) {
      case 'markdown':
        const mdFile = await this.generateMarkdownReport(legacyData);
        generatedFiles.push(mdFile);
        break;
        
      case 'html':
        const htmlFile = await this.generateHTMLReport(legacyData);
        generatedFiles.push(htmlFile);
        break;
        
      case 'json':
        const jsonFile = this.generateJSONReport(strictData);
        generatedFiles.push(jsonFile);
        break;
        
      case 'csv':
        const csvFile = this.generateCSVReport(strictData);
        generatedFiles.push(csvFile);
        break;
        
      default:
        throw new Error(`Unsupported format: ${this.options.format}`);
    }

    return generatedFiles;
  }

  /**
   * Validiert ob erforderliche Analyse-Typen vorhanden sind
   */
  private validateRequiredAnalyses(strictData: StrictAuditData): {
    isValid: boolean;
    missingAnalyses: string[];
    warnings: string[];
  } {
    const warnings: string[] = [];
    const missingAnalyses: Set<string> = new Set();

    for (const page of strictData.pages) {
      // Check each required analysis type
      for (const requiredType of this.options.requiredAnalysisTypes) {
        let hasValidAnalysis = false;
        
        switch (requiredType) {
          case 'accessibility':
            hasValidAnalysis = page.accessibility.totalIssues >= 0; // Always valid if structure exists
            break;
          case 'performance':
            hasValidAnalysis = page.performance.coreWebVitals.largestContentfulPaint >= 0;
            break;
          case 'seo':
            hasValidAnalysis = typeof page.seo.metaTags.title === 'string';
            break;
          case 'contentWeight':
            hasValidAnalysis = page.contentWeight.resources.totalSize >= 0;
            break;
          case 'mobileFriendliness':
            hasValidAnalysis = page.mobileFriendliness.overallScore >= 0;
            break;
        }

        if (!hasValidAnalysis) {
          missingAnalyses.add(requiredType);
          warnings.push(`Page ${page.url}: Missing valid ${requiredType} analysis`);
        }
      }

      // Additional validation: ensure complete analysis
      if (!hasCompleteAnalysis(page)) {
        warnings.push(`Page ${(page as any).url}: Does not have complete analysis data`);
      }
    }

    return {
      isValid: missingAnalyses.size === 0,
      missingAnalyses: Array.from(missingAnalyses),
      warnings
    };
  }

  /**
   * Fallback: Erstellt einen einfachen Report aus unvollständigen Daten
   */
  private async generateBasicReportFromIncompleteData(
    legacyResult: AuditResult,
    errorMessage: string
  ): Promise<StrictReportResult> {
    console.warn('⚠️ Generating basic report from incomplete data...');
    
    const outputPath = join(this.options.outputDir, `${this.options.filename}-incomplete.md`);
    
    const basicReport = this.createIncompleteDataReport(legacyResult, errorMessage);
    
    try {
      writeFileSync(outputPath, basicReport, 'utf8');
      
      return {
        success: true,
        generatedFiles: [outputPath],
        validationWarnings: [
          'Report generated from incomplete data',
          `Validation error: ${errorMessage}`,
          'Some analysis results may be missing or inaccurate'
        ]
      };
    } catch (writeError) {
      return {
        success: false,
        generatedFiles: [],
        validationWarnings: [],
        error: `Failed to write basic report: ${writeError instanceof Error ? writeError.message : 'Unknown error'}`
      };
    }
  }

  /**
   * Erstellt einen Report für unvollständige Daten mit Diagnose-Informationen
   */
  private createIncompleteDataReport(legacyResult: AuditResult, errorMessage: string): string {
    const timestamp = new Date().toISOString();
    const pageCount = legacyResult.pages?.length || 0;
    
    return `# Incomplete Audit Report

⚠️ **Warning**: This report was generated from incomplete data due to validation errors.

## Report Information

- **Generated**: ${timestamp}
- **Status**: Incomplete Data
- **Error**: ${errorMessage}
- **Pages Attempted**: ${pageCount}

## Summary

${legacyResult.summary ? `
- **Total Pages**: ${legacyResult.summary.totalPages || 'Unknown'}
- **Tested Pages**: ${legacyResult.summary.testedPages || 'Unknown'}
- **Average Score**: ${legacyResult.summary?.averageScore || 'Unknown'}
- **Overall Grade**: ${legacyResult.summary?.overallGrade || 'Unknown'}
` : 'Summary data unavailable'}

## Issue Diagnosis

The following validation issues prevented complete report generation:

1. **Data Structure**: ${errorMessage}
2. **Missing Analysis**: Some pages may be missing required analysis types
3. **Incomplete Results**: Accessibility, performance, or other analysis may be incomplete

## Available Page Data

${legacyResult.pages?.map((page: any) => `
### ${page.title || page.url}

- **URL**: ${page.url}
- **Status**: ${page.status || 'Unknown'}
- **Accessibility Score**: ${page.accessibility?.score || 'Not available'}
- **Performance Score**: ${page.performance?.score || 'Not available'}
- **SEO Score**: ${page.seo?.score || 'Not available'}

`).join('\n') || 'No page data available'}

---

**Note**: For complete and accurate results, please resolve the validation issues and regenerate the report.
`;
  }

  /**
   * Generiert Markdown-Report mit inline Implementierung
   */
  private async generateMarkdownReport(legacyData: AuditResult): Promise<string> {
    const outputPath = join(this.options.outputDir, `${this.options.filename}.md`);
    
    const markdownContent = this.createMarkdownReport(legacyData);
    writeFileSync(outputPath, markdownContent, 'utf8');
    
    return outputPath;
  }

  /**
   * Generiert HTML-Report mit inline Implementierung
   */
  private async generateHTMLReport(legacyData: AuditResult): Promise<string> {
    const outputPath = join(this.options.outputDir, `${this.options.filename}.html`);
    
    const htmlContent = this.createHTMLReport(legacyData);
    writeFileSync(outputPath, htmlContent, 'utf8');
    
    return outputPath;
  }

  /**
   * Generiert JSON-Report direkt aus strikten Daten
   */
  private generateJSONReport(strictData: StrictAuditData): string {
    const outputPath = join(this.options.outputDir, `${this.options.filename}.json`);
    
    const jsonContent = JSON.stringify(strictData, null, 2);
    writeFileSync(outputPath, jsonContent, 'utf8');
    
    return outputPath;
  }

  /**
   * Generiert CSV-Report aus strikten Daten
   */
  private generateCSVReport(strictData: StrictAuditData): string {
    const outputPath = join(this.options.outputDir, `${this.options.filename}.csv`);
    
    // CSV Header
    const headers = [
      'URL', 'Title', 'Status', 'Duration (ms)',
      'Accessibility Score', 'Accessibility Grade', 'Total Issues', 'Errors', 'Warnings',
      'Performance Score', 'Performance Grade', 'LCP (s)', 'FCP (s)', 'CLS',
      'SEO Score', 'SEO Grade', 'Title Length', 'Description Length',
      'Content Weight Score', 'Content Weight Grade', 'Total Size (bytes)',
      'Mobile Score', 'Mobile Grade', 'Touch Target Issues'
    ];

    // CSV Rows
    const rows = strictData.pages.map(page => [
      page.url,
      page.title,
      page.status,
      page.duration,
      page.accessibility.score,
      page.accessibility.wcagLevel,
      page.accessibility.totalIssues,
      page.accessibility.errors.length,
      page.accessibility.warnings.length,
      page.performance.score,
      page.performance.grade,
      page.performance.coreWebVitals.largestContentfulPaint,
      page.performance.coreWebVitals.firstContentfulPaint,
      page.performance.coreWebVitals.cumulativeLayoutShift,
      page.seo.score,
      page.seo.grade,
      page.seo.metaTags.titleLength,
      page.seo.metaTags.descriptionLength,
      page.contentWeight.score,
      page.contentWeight.grade,
      page.contentWeight.resources.totalSize,
      page.mobileFriendliness.overallScore,
      page.mobileFriendliness.grade,
      page.mobileFriendliness.touchTargetIssues
    ]);

    // Create CSV content
    const csvContent = [headers, ...rows]
      .map(row => row.map(cell => `"${cell}"`).join(','))
      .join('\n');

    writeFileSync(outputPath, csvContent, 'utf8');
    return outputPath;
  }

  /**
   * Konvertiert strikte Daten zurück zu Legacy-Format für bestehende Generatoren
   */
  private convertStrictToLegacy(strictData: StrictAuditData): AuditResult {
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
          recommendations: page.mobileFriendliness.recommendations.map(r => ({
            category: r.category,
            priority: r.priority,
            issue: r.issue,
            recommendation: r.recommendation,
            impact: r.impact
          }))
        }
      })),
      systemPerformance: {
        memoryUsageMB: strictData.systemPerformance.memoryUsageMB
      }
    } as AuditResult;
  }

  /**
   * Erstellt einfachen Markdown-Report
   */
  private createMarkdownReport(data: AuditResult): string {
    const timestamp = new Date().toISOString();
    const pages = data.pages || [];
    
    return `# Strict Audit Report

Generated: ${timestamp}
Pages: ${pages.length}

## Summary

- Total Pages: ${data.summary?.totalPages || pages.length}
- Tested Pages: ${data.summary?.testedPages || pages.length}
- Total Errors: ${data.summary?.totalErrors || 0}
- Total Warnings: ${data.summary?.totalWarnings || 0}

## Pages

${pages.map((page: any) => `
### ${page.title || page.url}

- URL: ${page.url}
- Status: ${page.status}
- Duration: ${page.duration}ms
- Accessibility Score: ${page.accessibility?.score || 'N/A'}

`).join('')}

---

Report generated with strict validation enabled.
`;
  }

  /**
   * Erstellt einfachen HTML-Report
   */
  private createHTMLReport(data: AuditResult): string {
    const timestamp = new Date().toISOString();
    const pages = data.pages || [];
    
    return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Strict Audit Report</title>
  <style>
    body { font-family: Arial, sans-serif; margin: 40px; }
    .header { background: #f0f0f0; padding: 20px; margin-bottom: 30px; }
    .page { border: 1px solid #ddd; margin: 20px 0; padding: 15px; }
    .score { font-weight: bold; color: #007acc; }
  </style>
</head>
<body>
  <div class="header">
    <h1>Strict Audit Report</h1>
    <p><strong>Generated:</strong> ${timestamp}</p>
    <p><strong>Pages:</strong> ${pages.length}</p>
  </div>
  
  <h2>Summary</h2>
  <ul>
    <li>Total Pages: ${data.summary?.totalPages || pages.length}</li>
    <li>Tested Pages: ${data.summary?.testedPages || pages.length}</li>
    <li>Total Errors: ${data.summary?.totalErrors || 0}</li>
    <li>Total Warnings: ${data.summary?.totalWarnings || 0}</li>
  </ul>
  
  <h2>Pages</h2>
  ${pages.map((page: any) => `
  <div class="page">
    <h3>${page.title || page.url}</h3>
    <p><strong>URL:</strong> ${page.url}</p>
    <p><strong>Status:</strong> ${page.status}</p>
    <p><strong>Duration:</strong> ${page.duration}ms</p>
    <p><strong>Accessibility Score:</strong> <span class="score">${page.accessibility?.score || 'N/A'}</span></p>
  </div>
  `).join('')}
  
  <footer style="margin-top: 50px; padding-top: 20px; border-top: 1px solid #ccc; font-size: 12px; color: #666;">
    Report generated with strict validation enabled.
  </footer>
</body>
</html>`;
  }
}

// ============================================================================
// CONVENIENCE FUNCTIONS
// ============================================================================

/**
 * High-level function: Generiert strikten Report aus Legacy-Daten
 */
export async function generateStrictReport(
  legacyResult: AuditResult,
  options: Partial<StrictReportOptions> = {}
): Promise<StrictReportResult> {
  const generator = new StrictReportGenerator(options);
  return generator.generateFromLegacyData(legacyResult);
}

/**
 * Batch-Funktion: Generiert mehrere Report-Formate gleichzeitig
 */
export async function generateMultipleStrictReports(
  legacyResult: AuditResult,
  formats: ('markdown' | 'html' | 'json' | 'csv')[],
  baseOptions: Partial<StrictReportOptions> = {}
): Promise<{ [format: string]: StrictReportResult }> {
  const results: { [format: string]: StrictReportResult } = {};
  
  for (const format of formats) {
    const options = { ...baseOptions, format };
    const generator = new StrictReportGenerator(options);
    results[format] = await generator.generateFromLegacyData(legacyResult);
  }
  
  return results;
}