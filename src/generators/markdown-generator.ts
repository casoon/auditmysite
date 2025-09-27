import { FullAuditResult } from '../types/audit-results';

/**
 * Markdown Generator - generates detailed issues in Markdown format
 * Primary use: detailed accessibility issues for developer workflows
 */
export class MarkdownGenerator {
  /**
   * Generate detailed issues markdown report
   */
  generateDetailedIssues(auditData: FullAuditResult): string {
    const lines: string[] = [];
    
    // Header
    lines.push('# Detailed Accessibility Issues Report');
    lines.push('');
    lines.push(`**Generated:** ${auditData.metadata.timestamp}`);
    lines.push(`**Tool Version:** ${auditData.metadata.toolVersion}`);
    lines.push('');
    
    // Summary
    lines.push('## Summary');
    lines.push(`- **Tested Pages:** ${auditData.summary.testedPages}`);
    lines.push(`- **Failed Pages:** ${auditData.summary.failedPages}`);
    lines.push(`- **Total Errors:** ${auditData.summary.totalErrors}`);
    lines.push(`- **Total Warnings:** ${auditData.summary.totalWarnings}`);
    lines.push('');
    
    // Issues by page (exclude skipped redirect pages)
    const pagesForIssues = auditData.pages.filter(p => ((p as any).status !== 'skipped'));
    pagesForIssues.forEach((page, index) => {
      const notices = page.accessibility.notices || [];
      const accessibilityIssues = [...page.accessibility.errors, ...page.accessibility.warnings, ...notices];
      
      if (accessibilityIssues.length > 0) {
        lines.push(`## Page ${index + 1}: ${page.title}`);
        lines.push(`**URL:** ${page.url}`);
        lines.push(`**Status:** ${page.status.toUpperCase()}`);
        lines.push(`**Issues Found:** ${accessibilityIssues.length}`);
        lines.push('');
        
        // Group issues by type
        const errorIssues = page.accessibility.errors;
        const warningIssues = page.accessibility.warnings;
        const noticeIssues = page.accessibility.notices || [];
        
        if (errorIssues.length > 0) {
          lines.push('### ❌ Errors');
          errorIssues.forEach((issue, issueIndex) => {
            this.formatIssue(lines, issue, issueIndex + 1, 'error');
          });
          lines.push('');
        }
        
        if (warningIssues.length > 0) {
          lines.push('### ⚠️ Warnings');
          warningIssues.forEach((issue, issueIndex) => {
            this.formatIssue(lines, issue, issueIndex + 1, 'warning');
          });
          lines.push('');
        }
        
        if (noticeIssues && noticeIssues.length > 0) {
          lines.push('### ℹ️ Notices');
          noticeIssues.forEach((issue, issueIndex) => {
            this.formatIssue(lines, issue, issueIndex + 1, 'notice');
          });
          lines.push('');
        }
        
        lines.push('---');
        lines.push('');
      }
    });
    
    return lines.join('\n');
  }
  
  /**
   * Generate summary markdown report
   */
  generateSummary(auditData: FullAuditResult): string {
    const lines: string[] = [];
    
    lines.push('# Accessibility Audit Summary');
    lines.push('');
    lines.push(`**Generated:** ${auditData.metadata.timestamp}`);
    lines.push(`**Duration:** ${auditData.metadata.duration}ms`);
    lines.push('');
    
    // Overall results
    lines.push('## Overall Results');
    lines.push(`- **Total Pages:** ${auditData.summary.totalPages}`);
    lines.push(`- **Tested Pages:** ${auditData.summary.testedPages}`);
    lines.push(`- **Passed Pages:** ${auditData.summary.passedPages}`);
    lines.push(`- **Failed Pages:** ${auditData.summary.failedPages}`);
    lines.push(`- **Success Rate:** ${((auditData.summary.passedPages / auditData.summary.testedPages) * 100).toFixed(1)}%`);
    lines.push('');
    
    // Page-by-page summary
    lines.push('## Page Summary');
    lines.push('');
    lines.push('| Page | Status | Score | Errors | Warnings |');
    lines.push('|------|--------|-------|--------|----------|');
    
    const pagesForSummary = auditData.pages.filter(p => ((p as any).status !== 'skipped'));
    pagesForSummary.forEach(page => {
      const statusIcon = page.status === 'passed' ? '✅' : page.status === 'failed' ? '❌' : (page.status === 'crashed' ? '💥' : '⚠️');
      lines.push(`| ${page.title} | ${statusIcon} ${page.status} | ${page.accessibility.score}/100 | ${page.accessibility.errors.length} | ${page.accessibility.warnings.length} |`);
    });
    
    return lines.join('\n');
  }
  
  private formatIssue(lines: string[], issue: any, index: number, type: string): void {
    lines.push(`#### Issue ${index}`);
    lines.push(`- **Severity:** ${type}`);
    lines.push(`- **Code:** ${issue.code || 'N/A'}`);
    lines.push(`- **Message:** ${issue.message}`);
    
    if (issue.selector) {
      lines.push(`- **Element:** \`${issue.selector}\``);
    }
    
    if (issue.context) {
      lines.push(`- **Context:** \`${issue.context.substring(0, 100)}...\``);
    }
    
    if (issue.help) {
      lines.push(`- **Help:** ${issue.help}`);
    }
    
    if (issue.helpUrl) {
      lines.push(`- **More Info:** [${issue.helpUrl}](${issue.helpUrl})`);
    }
    
    lines.push('');
  }
}
