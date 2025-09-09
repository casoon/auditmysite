import * as fs from 'fs';
import * as path from 'path';
import { TestSummary, AccessibilityResult, Pa11yIssue } from '../types';
import { htmlReportTemplate } from './html-template';

export interface HtmlReportOptions {
  outputDir?: string;
  includeCharts?: boolean;
  theme?: 'light' | 'dark' | 'auto';
}

export class HtmlReportGenerator {
  private template: string;

  constructor() {
    this.template = htmlReportTemplate;
  }

  private loadTemplate(): string {
    return htmlReportTemplate;
  }

  async generateHtmlReport(summary: TestSummary, options: HtmlReportOptions = {}): Promise<string> {
    const outputDir = options.outputDir || './reports';
    const dateOnly = new Date().toISOString().split('T')[0];
    
    // Erstelle Output-Verzeichnis falls nicht vorhanden
    if (!fs.existsSync(outputDir)) {
      fs.mkdirSync(outputDir, { recursive: true });
    }

    // Extrahiere Domain aus der ersten URL
    const domain = summary.results.length > 0 
      ? new URL(summary.results[0].url).hostname 
      : 'unknown';

    const htmlContent = this.generateHtmlContent(summary, domain, dateOnly, options);
    const filename = `${domain}-accessibility-report-${dateOnly}.html`;
    const outputPath = path.join(outputDir, filename);

    fs.writeFileSync(outputPath, htmlContent, 'utf-8');
    console.log('HTML geschrieben:', outputPath);
    return outputPath;
  }

  private generateHtmlContent(summary: TestSummary, domain: string, date: string, options: HtmlReportOptions): string {
    const successRate = summary.testedPages > 0 ? (summary.passedPages / summary.testedPages * 100).toFixed(1) : '0';
    const errorRate = summary.testedPages > 0 ? (summary.failedPages / summary.testedPages * 100).toFixed(1) : '0';

    let html = this.template
      .replace(/{{domain}}/g, domain)
      .replace(/{{timestamp}}/g, new Date().toLocaleString('de-DE'))
      .replace(/{{successRate}}/g, successRate)
      .replace(/{{errorRate}}/g, errorRate)
      .replace(/{{totalPages}}/g, summary.totalPages.toString())
      .replace(/{{testedPages}}/g, summary.testedPages.toString())
      .replace(/{{passedPages}}/g, summary.passedPages.toString())
      .replace(/{{failedPages}}/g, summary.failedPages.toString())
      .replace(/{{totalErrors}}/g, summary.totalErrors.toString())
      .replace(/{{totalWarnings}}/g, summary.totalWarnings.toString())
      .replace(/{{totalDuration}}/g, this.formatDuration(summary.totalDuration));

    // Ersetze Issues-Placeholder
    html = html.replace('{{issues}}', this.generateIssuesTable(summary.results, options));
    
    // Ersetze Performance-Placeholder
    html = html.replace('{{performance}}', this.generatePerformanceTable(summary.results, options));
    
    // Ersetze SEO-Placeholder
    html = html.replace('{{seo}}', this.generateSeoTable(summary.results, options));
    
    // Ersetze Mobile-Friendliness-Placeholder
    html = html.replace('{{mobileFriendliness}}', this.generateMobileFriendlinessTable(summary.results, options));

    return html;
  }

  private generateIssuesTable(results: AccessibilityResult[], options: HtmlReportOptions): string {
    const allIssues = results.flatMap(result => 
      result.errors.map((error: string) => ({
        code: 'ERROR',
        message: error,
        selector: 'N/A',
        context: 'N/A',
        page: result.url,
        pageName: this.getPageName(result.url),
        type: 'error'
      })).concat(
        result.warnings.map((warning: string) => ({
          code: 'WARNING',
          message: warning,
          selector: 'N/A',
          context: 'N/A',
          page: result.url,
          pageName: this.getPageName(result.url),
          type: 'warning'
        }))
      )
    );

    if (allIssues.length === 0) {
      return `
        <div class="no-issues">
          <h3>🎉 Keine Probleme gefunden!</h3>
          <p>Alle getesteten Seiten erfüllen die Accessibility-Standards.</p>
        </div>
      `;
    }

    let tableRows = '';
    allIssues.forEach((issue, index) => {
      const severityClass = issue.type === 'error' ? 'error' : 'warning';
      const severityIcon = issue.type === 'error' ? '❌' : '⚠️';
      
      tableRows += `
        <tr class="${severityClass}">
          <td>${severityIcon} ${issue.code || 'N/A'}</td>
          <td>${this.escapeHtml(issue.message)}</td>
          <td>${this.escapeHtml(issue.selector || 'N/A')}</td>
          <td>${this.escapeHtml(issue.context || 'N/A')}</td>
          <td>${this.escapeHtml(issue.pageName)}</td>
        </tr>
      `;
    });

    return `
      <div class="table-container">
        <div class="table-header">
          <h3>🔍 Accessibility Issues (${allIssues.length})</h3>
        </div>
        <div class="table-wrapper">
          <table id="issues-table" class="data-table">
            <thead>
              <tr>
                <th>Code</th>
                <th>Message</th>
                <th>Selector</th>
                <th>Context</th>
                <th>Page</th>
              </tr>
            </thead>
            <tbody>
              ${tableRows}
            </tbody>
          </table>
        </div>
      </div>
    `;
  }

  private generatePerformanceTable(results: AccessibilityResult[], options: HtmlReportOptions): string {
    const performanceResults = results.filter(r => r.performanceMetrics);
    
    if (performanceResults.length === 0) {
      return `
        <div class="no-data">
          <h3>📊 Keine Performance-Daten verfügbar</h3>
          <p>Performance-Metriken wurden nicht gesammelt.</p>
        </div>
      `;
    }

    let tableRows = '';
    performanceResults.forEach(result => {
      const metrics = result.performanceMetrics!;
      tableRows += `
        <tr>
          <td>${this.escapeHtml(this.getPageName(result.url))}</td>
          <td>${this.formatDuration(metrics.loadTime)}</td>
          <td>${this.formatDuration(metrics.domContentLoaded)}</td>
          <td>${this.formatDuration(metrics.firstContentfulPaint)}</td>
          <td>${this.formatDuration(metrics.largestContentfulPaint)}</td>
          <td>N/A</td>
        </tr>
      `;
    });

    return `
      <div class="table-container">
        <div class="table-header">
          <h3>⚡ Performance Metrics (${performanceResults.length})</h3>
        </div>
        <div class="table-wrapper">
          <table id="performance-table" class="data-table">
            <thead>
              <tr>
                <th>Page</th>
                <th>Load Time</th>
                <th>DOM Ready</th>
                <th>FCP</th>
                <th>LCP</th>
                <th>Memory</th>
              </tr>
            </thead>
            <tbody>
              ${tableRows}
            </tbody>
          </table>
        </div>
      </div>
    `;
  }

  private generateSeoTable(results: AccessibilityResult[], options: HtmlReportOptions): string {
    // Da SEO-Analyse noch nicht implementiert ist, zeigen wir eine Nachricht
    return `
      <div class="no-data">
        <h3>🔍 SEO Analysis</h3>
        <p>SEO-Analyse wird in einer zukünftigen Version verfügbar sein.</p>
      </div>
    `;
  }

  private generateMobileFriendlinessTable(results: AccessibilityResult[], options: HtmlReportOptions): string {
    // Check for mobile-friendliness data in results
    const mobileResults = results.filter(r => r.mobileFriendliness && typeof r.mobileFriendliness === 'object');
    
    // Check for mobile-friendliness data in results
    
    if (mobileResults.length === 0) {
      return `
        <div class="no-data">
          <h3>📱 Keine Mobile-Friendliness-Daten verfügbar</h3>
          <p>Mobile-Friendliness-Analyse wurde nicht durchgeführt oder Daten sind nicht verfügbar.</p>
        </div>
      `;
    }

    let overviewHtml = this.generateMobileOverview(mobileResults);
    let detailsHtml = this.generateMobileDetails(mobileResults);
    
    return overviewHtml + detailsHtml;
  }

  private generateMobileOverview(results: AccessibilityResult[]): string {
    // Berechne Durchschnittswerte
    const totalScore = results.reduce((sum, r) => sum + (r.mobileFriendliness?.overallScore || 0), 0);
    const averageScore = results.length > 0 ? totalScore / results.length : 0;
    
    const gradeClass = this.getScoreClass(averageScore);
    const grade = this.getScoreGrade(averageScore);
    
    return `
      <div class="mobile-overview">
        <h3>📱 Mobile-Friendliness Overview</h3>
        <div class="metrics-grid">
          <div class="metric-card ${gradeClass}">
            <div class="metric-label">Overall Score</div>
            <div class="metric-value">${averageScore.toFixed(1)}/100</div>
            <div class="metric-grade ${grade}">${grade}</div>
          </div>
        </div>
      </div>
    `;
  }

  private generateMobileDetails(results: AccessibilityResult[]): string {
    let tableRows = '';
    
    results.forEach(result => {
      const mobile = result.mobileFriendliness!;
      const gradeClass = this.getScoreClass(mobile.overallScore);
      const grade = this.getScoreGrade(mobile.overallScore);
      
      tableRows += `
        <tr>
          <td>${this.escapeHtml(this.getPageName(result.url))}</td>
          <td class="score-cell ${gradeClass}">${mobile.overallScore.toFixed(1)}</td>
          <td class="score-cell">${mobile.viewport.score.toFixed(1)}</td>
          <td class="score-cell">${mobile.typography.score.toFixed(1)}</td>
          <td class="score-cell">${mobile.touchTargets.score.toFixed(1)}</td>
          <td class="score-cell">${mobile.navigation.score.toFixed(1)}</td>
        </tr>
      `;
    });

    return `
      <div class="table-container" style="margin-top: 2rem;">
        <div class="table-header">
          <h3>📊 Mobile-Friendliness Details (${results.length} pages)</h3>
        </div>
        <div class="table-wrapper">
          <table class="data-table">
            <thead>
              <tr>
                <th>Page</th>
                <th>Overall Score</th>
                <th>Viewport</th>
                <th>Typography</th>
                <th>Touch Targets</th>
                <th>Navigation</th>
              </tr>
            </thead>
            <tbody>
              ${tableRows}
            </tbody>
          </table>
        </div>
      </div>
    `;
  }

  private getScoreClass(score: number): string {
    if (score >= 90) return 'excellent';
    if (score >= 75) return 'good';
    if (score >= 50) return 'needs-improvement';
    return 'poor';
  }

  private getScoreGrade(score: number): string {
    if (score >= 90) return 'A';
    if (score >= 80) return 'B';
    if (score >= 70) return 'C';
    if (score >= 60) return 'D';
    return 'F';
  }

  private getPageName(url: string): string {
    try {
      const urlObj = new URL(url);
      const pathname = urlObj.pathname;
      return pathname === '/' ? 'Home' : pathname.split('/').pop() || pathname;
    } catch {
      return url;
    }
  }

  private escapeHtml(text: string): string {
    return text
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;')
      .replace(/'/g, '&#039;');
  }

  private formatDuration(ms: number): string {
    if (ms < 1000) return `${ms}ms`;
    if (ms < 60000) return `${(ms / 1000).toFixed(1)}s`;
    return `${(ms / 60000).toFixed(1)}min`;
  }
} 