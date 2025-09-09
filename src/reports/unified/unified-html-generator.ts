import * as fs from 'fs/promises';
import * as path from 'path';
import { FullAuditResult } from '../../types/audit-results';

/**
 * Unified HTML Generator
 * - Reads JSON (FullAuditResult)
 * - Renders minimal, modern HTML using strict types
 * - Section-based architecture (Header, Summary, Pages, Footer)
 */
export class UnifiedHTMLGenerator {
  async generateFromJSON(jsonPath: string): Promise<string> {
    const json = await fs.readFile(jsonPath, 'utf8');
    const data: FullAuditResult = JSON.parse(json);
    return this.generate(data);
  }

  async generate(data: FullAuditResult): Promise<string> {
    const css = this.generateCSS();
    const header = this.renderHeader(data);
    const summary = this.renderSummary(data);
    const pages = this.renderPages(data);
    const footer = this.renderFooter(data);

    return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>AuditMySite Report</title>
  <style>${css}</style>
</head>
<body>
  <div class="container">
    ${header}
    ${summary}
    ${pages}
    ${footer}
  </div>
</body>
</html>`;
  }

  private generateCSS(): string {
    return `
      :root {
        --color-bg: #f8fafc;
        --color-card: #ffffff;
        --color-text: #1f2937;
        --color-subtle: #6b7280;
        --primary: #2563eb;
        --success: #10b981;
        --warning: #f59e0b;
        --error: #ef4444;
        --radius: 10px;
        --shadow: 0 2px 12px rgba(0,0,0,0.08);
      }
      body { font-family: -apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif; background: var(--color-bg); color: var(--color-text); margin: 0; }
      .container { max-width: 1200px; margin: 0 auto; padding: 24px; }
      .card { background: var(--color-card); border-radius: var(--radius); box-shadow: var(--shadow); padding: 20px; margin-bottom: 20px; }
      .header h1 { margin: 0 0 8px; color: var(--primary); }
      .subtitle { color: var(--color-subtle); }
      .grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(220px, 1fr)); gap: 16px; }
      .metric { border: 1px solid #e5e7eb; border-radius: var(--radius); padding: 16px; text-align: center; }
      .metric .value { font-size: 28px; font-weight: 700; }
      .metric .label { color: var(--color-subtle); font-size: 12px; text-transform: uppercase; letter-spacing: .06em; }
      .badge { display: inline-block; padding: 4px 10px; border-radius: 20px; font-size: 12px; font-weight: 600; }
      .badge.success { background: #ecfdf5; color: #065f46; }
      .badge.warning { background: #fffbeb; color: #92400e; }
      .badge.error { background: #fef2f2; color: #991b1b; }
      table { width: 100%; border-collapse: collapse; }
      th, td { padding: 12px; border-bottom: 1px solid #e5e7eb; text-align: left; }
      th { background: #f3f4f6; font-weight: 600; color: #374151; }
      .page-status { font-weight: 700; }
      .status-passed { color: var(--success); }
      .status-failed { color: var(--error); }
      .status-crashed { color: var(--warning); }
    `;
  }

  private renderHeader(data: FullAuditResult): string {
    return `
      <div class="card header">
        <h1>AuditMySite Report</h1>
        <div class="subtitle">
          <div>Sitemap: ${this.escape(data.metadata.sitemapUrl)}</div>
          <div>Generated: ${new Date(data.metadata.timestamp).toLocaleString()}</div>
          <div>Version: ${this.escape(data.metadata.toolVersion)} • Format v${this.escape(data.metadata.version)}</div>
        </div>
      </div>
    `;
  }

  private renderSummary(data: FullAuditResult): string {
    const s = data.summary;
    return `
      <div class="card">
        <div class="grid">
          <div class="metric">
            <div class="value">${s.testedPages}/${s.totalPages}</div>
            <div class="label">Pages Tested</div>
          </div>
          <div class="metric">
            <div class="value">${s.passedPages}</div>
            <div class="label">Passed</div>
          </div>
          <div class="metric">
            <div class="value">${s.failedPages}</div>
            <div class="label">Failed</div>
          </div>
          <div class="metric">
            <div class="value">${s.crashedPages}</div>
            <div class="label">Crashed</div>
          </div>
          <div class="metric">
            <div class="value">${s.totalErrors}</div>
            <div class="label">Total Errors</div>
          </div>
          <div class="metric">
            <div class="value">${s.totalWarnings}</div>
            <div class="label">Total Warnings</div>
          </div>
        </div>
      </div>
    `;
  }

  private renderPages(data: FullAuditResult): string {
    const rows = data.pages.map(p => {
      const statusClass = p.status === 'passed' ? 'status-passed' : (p.status === 'failed' ? 'status-failed' : 'status-crashed');
      const statusText = p.status.toUpperCase();
      return `
        <tr>
          <td><div><strong>${this.escape(p.title || p.url)}</strong><br/><small>${this.escape(p.url)}</small></div></td>
          <td class="page-status ${statusClass}">${statusText}</td>
          <td>${p.accessibility.errors.length}</td>
          <td>${p.accessibility.warnings.length}</td>
          <td>${p.accessibility.score}</td>
          <td>${p.performance ? p.performance.coreWebVitals.largestContentfulPaint : '—'}</td>
          <td>${Math.round(p.duration)}ms</td>
        </tr>
      `;
    }).join('');

    return `
      <div class="card">
        <h2>Pages</h2>
        <div style="overflow-x:auto">
          <table>
            <thead>
              <tr>
                <th>Page</th>
                <th>Status</th>
                <th>Errors</th>
                <th>Warnings</th>
                <th>A11y Score</th>
                <th>LCP</th>
                <th>Duration</th>
              </tr>
            </thead>
            <tbody>
              ${rows}
            </tbody>
          </table>
        </div>
      </div>
    `;
  }

  private renderFooter(data: FullAuditResult): string {
    return `
      <div class="card" style="text-align:center; color:#6b7280;">
        Generated by AuditMySite • Duration: ${Math.round(data.metadata.duration/1000)}s
      </div>
    `;
  }

  private escape(s: string): string {
    return s
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;')
      .replace(/'/g, '&#039;');
  }
}

