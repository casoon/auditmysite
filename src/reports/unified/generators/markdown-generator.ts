/**
 * 🔧 Modern Markdown Report Generator
 * 
 * Generates comprehensive Markdown reports for developers.
 * Perfect for version control and documentation.
 */

import { 
  ReportGenerator, 
  ReportData, 
  ReportOptions, 
  GeneratedReport 
} from '../base-generator';
import * as fs from 'fs/promises';
import * as path from 'path';

export class ModernMarkdownReportGenerator extends ReportGenerator {
  constructor() {
    super('markdown');
  }

  getExtension(): string {
    return 'md';
  }

  getMimeType(): string {
    return 'text/markdown';
  }

  async generate(data: ReportData, options: ReportOptions): Promise<GeneratedReport> {
    const startTime = Date.now();

    // Validate data
    const validation = this.validateData(data);
    if (!validation.valid) {
      throw new Error(`Invalid report data: ${validation.errors.join(', ')}`);
    }

    // Generate Markdown content
    const markdownContent = this.generateMarkdown(data, options);

    // Create output directory (async)
    await fs.mkdir(options.outputDir, { recursive: true });

    // Write file (async)
    const filename = this.generateFilename(options, 'accessibility');
    const filePath = path.join(options.outputDir, filename);
    await fs.writeFile(filePath, markdownContent, 'utf8');

    const duration = Date.now() - startTime;

    return {
      path: filePath,
      format: this.format,
      size: this.calculateFileSize(markdownContent),
      metadata: {
        generatedAt: new Date(),
        duration
      }
    };
  }

  private generateMarkdown(data: ReportData, options: ReportOptions): string {
    const { summary, issues, metadata } = data;
    const successRate = this.calculateSuccessRate(summary);
    
    return [
      this.generateHeader(metadata, successRate),
      this.generateSummarySection(summary),
      this.generateMetricsSection(summary),
      !options.summaryOnly ? this.generateDetailsSection(summary, options) : '',
      this.generateFooter(metadata, options)
    ].filter(Boolean).join('\n\n');
  }

  private generateHeader(metadata: any, successRate: number): string {
    const statusEmoji = successRate >= 90 ? '✅' : successRate >= 70 ? '⚠️' : '❌';
    
    return `# ${statusEmoji} Accessibility Audit Report

**Generated:** ${new Date(metadata.timestamp).toLocaleString()}  
${metadata.sitemapUrl ? `**Site:** ${metadata.sitemapUrl}` : ''}  
${metadata.environment ? `**Environment:** ${metadata.environment}` : ''}  
**Duration:** ${this.formatDuration(metadata.duration)}

---`;
  }

  private generateSummarySection(summary: any): string {
    const successRate = this.calculateSuccessRate(summary);
    const rateEmoji = successRate >= 90 ? '🟢' : successRate >= 70 ? '🟡' : '🔴';
    
    return `## 📊 Summary

${rateEmoji} **Overall Success Rate: ${successRate}%**

${summary.passedPages} of ${summary.testedPages} pages passed accessibility tests.`;
  }

  private generateMetricsSection(summary: any): string {
    return `## 📈 Key Metrics

| Metric | Count | Percentage |
|--------|-------|------------|
| **Pages Tested** | ${summary.testedPages} | 100% |
| **✅ Passed** | ${summary.passedPages} | ${Math.round((summary.passedPages / summary.testedPages) * 100)}% |
| **❌ Failed** | ${summary.failedPages} | ${Math.round((summary.failedPages / summary.testedPages) * 100)}% |
| **💥 Crashed** | ${summary.crashedPages || 0} | ${Math.round(((summary.crashedPages || 0) / summary.testedPages) * 100)}% |
| **🐛 Total Errors** | ${summary.totalErrors} | - |
| **⚠️ Total Warnings** | ${summary.totalWarnings} | - |

**Average Test Duration:** ${Math.round(summary.totalDuration / summary.testedPages)}ms per page`;
  }

  private generateDetailsSection(summary: any, options: ReportOptions): string {
    if (options.summaryOnly || !summary.results?.length) {
      return '';
    }

    let markdown = `## 📋 Detailed Results\n\n`;

    // Group results by status
    const passed = summary.results.filter((r: any) => r.passed);
    const failed = summary.results.filter((r: any) => !r.passed && !r.crashed);
    const crashed = summary.results.filter((r: any) => r.crashed);

    if (failed.length > 0) {
      markdown += this.generateResultSection('❌ Failed Pages', failed, true);
    }

    if (crashed.length > 0) {
      markdown += this.generateResultSection('💥 Crashed Pages', crashed, true);
    }

    if (passed.length > 0) {
      markdown += this.generateResultSection('✅ Passed Pages', passed, false);
    }

    return markdown;
  }

  private generateResultSection(title: string, results: any[], showDetails: boolean): string {
    let section = `### ${title}\n\n`;

    results.forEach((result: any) => {
      section += `#### ${result.title || 'Untitled Page'}\n\n`;
      section += `**URL:** ${result.url}  \n`;
      section += `**Duration:** ${result.duration}ms  \n`;
      
      if (result.performanceMetrics) {
        section += `**LCP:** ${result.performanceMetrics.largestContentfulPaint}ms  \n`;
        section += `**CLS:** ${result.performanceMetrics.cumulativeLayoutShift?.toFixed(3) || 'N/A'}  \n`;
      }
      
      section += '\n';

      if (showDetails) {
        // Show errors
        if (result.errors?.length > 0) {
          section += `**🐛 Errors (${result.errors.length}):**\n\n`;
          result.errors.forEach((error: string) => {
            section += `- ${error}\n`;
          });
          section += '\n';
        }

        // Show warnings
        if (result.warnings?.length > 0) {
          section += `**⚠️ Warnings (${result.warnings.length}):**\n\n`;
          result.warnings.forEach((warning: string) => {
            section += `- ${warning}\n`;
          });
          section += '\n';
        }

        // Show pa11y issues if available
        if (result.pa11yIssues?.length > 0) {
          section += `**🔍 Pa11y Issues (${result.pa11yIssues.length}):**\n\n`;
          result.pa11yIssues.forEach((issue: any) => {
            section += `- **${issue.type.toUpperCase()}:** ${issue.message}\n`;
            if (issue.selector) {
              section += `  - *Selector:* \`${issue.selector}\`\n`;
            }
            if (issue.context) {
              section += `  - *Context:* ${issue.context}\n`;
            }
          });
          section += '\n';
        }
      }

      section += '---\n\n';
    });

    return section;
  }

  private generateFooter(metadata: any, options: ReportOptions): string {
    const company = options.branding?.company || 'AuditMySite';
    const footerText = options.branding?.footer || `Report generated by ${company}`;
    
    return `## 📄 Report Information

${footerText}

- **Version:** ${metadata.version}
- **Generated:** ${new Date(metadata.timestamp).toISOString()}
- **Report Duration:** ${this.formatDuration(metadata.duration)}
- **Format:** Markdown

### 🔗 Quick Links

- [WCAG Guidelines](https://www.w3.org/WAI/WCAG21/quickref/)
- [Pa11y Documentation](https://pa11y.org/)
- [Web Content Accessibility Guidelines](https://www.w3.org/WAI/WCAG21/Understanding/)

### 📊 How to Read This Report

- **✅ Passed:** Page meets accessibility standards
- **❌ Failed:** Page has accessibility issues that need fixing
- **💥 Crashed:** Technical error occurred during testing
- **🐛 Errors:** Critical accessibility violations
- **⚠️ Warnings:** Potential accessibility concerns

### 🚀 Next Steps

1. **Fix Critical Errors:** Focus on pages with the most errors first
2. **Address Warnings:** Review and resolve warning-level issues
3. **Test Manually:** Verify fixes with screen readers and keyboard navigation
4. **Re-run Tests:** Use the same configuration to verify improvements

---

*This report was generated automatically. For questions about specific issues, consult the WCAG guidelines or accessibility documentation.*`;
  }
}
