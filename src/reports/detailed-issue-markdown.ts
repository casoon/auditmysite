import { AuditIssue } from '@core/types';
import { groupByPage, sortBySeverity, sortByType } from './report-utils';

export class DetailedIssueMarkdownReport {
  static generate(issues: AuditIssue[], options?: { verbose?: boolean }): string {
    if (!Array.isArray(issues)) issues = [];
    
    if (options?.verbose) {
      console.log('DEBUG detailed-issue-markdown: processing', issues.length, 'issues');
    }
    
    const lines: string[] = [];
    lines.push('# Detailed Accessibility Error Report');
    lines.push(`Generated: ${new Date().toISOString()}`);
    lines.push(`Total Issues: ${issues.length}`);
    lines.push('');

    // Gruppiere nach Seite
    const issuesByPage = groupByPage(issues) || {};
    
    if (options?.verbose) {
      console.log('DEBUG detailed-issue-markdown: grouped by page', Object.keys(issuesByPage).length, 'pages');
    }
    for (const [pageUrl, pageIssues] of Object.entries(issuesByPage)) {
      lines.push(`## Page: ${pageUrl}`);
      if (pageIssues[0]?.pageTitle) {
        lines.push(`**Title:** ${pageIssues[0].pageTitle}`);
      }
      lines.push('');
      // Sortiere innerhalb der Seite
      const sortedIssues = sortByType(sortBySeverity(pageIssues));
      sortedIssues.forEach((issue, idx) => {
        lines.push(`### Issue ${idx + 1}`);
        lines.push(`- **Category:** ${issue.type}`);
        lines.push(`- **Severity:** ${issue.severity}`);
        if (issue.source) lines.push(`- **Source:** ${issue.source}`);
        lines.push(`- **Message:** ${issue.message}`);
        if (issue.code) lines.push(`- **Code:** ${issue.code}`);
        if (issue.selector) lines.push(`- **Selector:** \`${issue.selector}\``);
        if (issue.context) lines.push(`- **Context:** \`${issue.context}\``);
        if (issue.lineNumber) lines.push(`- **Line:** ${issue.lineNumber}`);
        if (issue.htmlSnippet) {
          lines.push('- **HTML Snippet:**');
          lines.push('```html');
          lines.push(issue.htmlSnippet);
          lines.push('```');
        }
        if (issue.recommendation) lines.push(`- **Recommendation:** ${issue.recommendation}`);
        if (issue.resource) lines.push(`- **Resource:** ${issue.resource}`);
        if (issue.score !== undefined) lines.push(`- **Score:** ${issue.score}`);
        if (issue.metric) lines.push(`- **Metric:** ${issue.metric}`);
        lines.push('');
      });
    }

    return lines.join('\n');
  }
}