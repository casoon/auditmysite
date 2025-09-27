import * as fs from 'fs/promises';
import * as path from 'path';
import { calculateCertificateLevel, calculateGrade, CertificateLevel, Grade } from '../types/base-types';

// Define the required types for enhanced reporting
interface EnhancedAuditResult {
  metadata: {
    version: string;
    timestamp: string;
    sitemapUrl: string;
    totalPages: number;
    testedPages: number;
    duration: number;
    toolVersion?: string;
  };
  summary: {
    totalPages: number;
    testedPages: number;
    passedPages: number;
    failedPages: number;
    crashedPages: number;
    totalErrors: number;
    totalWarnings: number;
    successRate: number;
    overallScore?: number;
    overallGrade?: Grade;
    certificateLevel?: CertificateLevel;
  };
  pages: Array<{
    url: string;
    title: string;
    status: string;
    duration: number;
    accessibility?: any;
    performance?: any;
    seo?: any;
    contentWeight?: any;
    mobileFriendliness?: any;
  }>;
}

/**
 * HTML Report Generator with Certificate Badges and Comprehensive Reporting
 * - Includes certificate SVG badges based on overall score
 * - Sticky navigation with anchor links
 * - Comprehensive sections for all analysis types
 * - Modern, interactive design
 */
export class HTMLGenerator {
  private certificateSVGs: Record<CertificateLevel, string> = {
    'PLATINUM': '',
    'GOLD': '',
    'SILVER': '',
    'BRONZE': '',
    'NEEDS_IMPROVEMENT': ''
  };

  constructor() {
    this.loadCertificateSVGs();
  }

  private async loadCertificateSVGs(): Promise<void> {
    try {
      const certificatesPath = path.join(__dirname, '..', 'assets', 'certificates');
      
      for (const level of Object.keys(this.certificateSVGs) as CertificateLevel[]) {
        const svgPath = path.join(certificatesPath, `${level}.svg`);
        try {
          this.certificateSVGs[level] = await fs.readFile(svgPath, 'utf-8');
        } catch (error) {
          console.warn(`Could not load certificate SVG for ${level}:`, error);
          // Fallback SVG
          this.certificateSVGs[level] = this.createFallbackSVG(level);
        }
      }
    } catch (error) {
      console.warn('Error loading certificate SVGs:', error);
    }
  }

  private createFallbackSVG(level: CertificateLevel): string {
    const colors = {
      PLATINUM: '#e5e7eb',
      GOLD: '#fbbf24',
      SILVER: '#d1d5db',
      BRONZE: '#cd7c2f',
      NEEDS_IMPROVEMENT: '#ef4444'
    };

    return `<svg width="120" height="120" viewBox="0 0 120 120" xmlns="http://www.w3.org/2000/svg">
      <circle cx="60" cy="60" r="55" fill="${colors[level]}" stroke="#374151" stroke-width="2"/>
      <text x="60" y="50" text-anchor="middle" fill="#374151" font-family="Arial" font-size="12" font-weight="bold">${level.replace('_', ' ')}</text>
      <text x="60" y="70" text-anchor="middle" fill="#374151" font-family="Arial" font-size="10">CERTIFICATE</text>
    </svg>`;
  }

  async generateFromJSON(jsonPath: string): Promise<string> {
    const json = await fs.readFile(jsonPath, 'utf8');
    const data: EnhancedAuditResult = JSON.parse(json);
    return this.generate(data);
  }

  async generate(data: EnhancedAuditResult): Promise<string> {
    // Calculate overall metrics if not present
    if (!data.summary.overallScore) {
      data.summary.overallScore = this.calculateOverallScore(data);
    }
    if (!data.summary.overallGrade) {
      data.summary.overallGrade = calculateGrade(data.summary.overallScore);
    }
    if (!data.summary.certificateLevel) {
      data.summary.certificateLevel = calculateCertificateLevel(data.summary.overallScore);
    }

    const domain = this.extractDomain(data.metadata.sitemapUrl);
    const certificateLevel = data.summary.certificateLevel;
    const certificateSVG = this.certificateSVGs[certificateLevel] || this.createFallbackSVG(certificateLevel);

    const css = this.generateCSS();
    const header = this.renderHeader(data, domain, certificateSVG);
    const navigation = this.renderNavigation();
    const summary = this.renderSummary(data);
    const accessibility = this.renderAccessibilitySection(data);
    const performance = this.renderPerformanceSection(data);
    const seo = this.renderSEOSection(data);
    const contentWeight = this.renderContentWeightSection(data);
    const mobileFriendliness = this.renderMobileFriendlinessSection(data);
    const pages = this.renderPagesSection(data);
    const footer = this.renderFooter(data);

    return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>AuditMySite Report - ${this.escape(domain)}</title>
  <style>${css}</style>
</head>
<body>
  ${header}
  ${navigation}
  <div class="main-content">
    <div class="container">
      ${summary}
      ${accessibility}
      ${performance}
      ${seo}
      ${contentWeight}
      ${mobileFriendliness}
      ${pages}
      ${footer}
    </div>
  </div>
  <script>${this.generateJavaScript()}</script>
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
        --radius: 12px;
        --shadow: 0 4px 16px rgba(0,0,0,0.08);
        --shadow-lg: 0 8px 32px rgba(0,0,0,0.12);
      }

      * { margin: 0; padding: 0; box-sizing: border-box; }
      
      body {
        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
        background: var(--color-bg);
        color: var(--color-text);
        line-height: 1.6;
      }

      .header {
        background: linear-gradient(135deg, #0f172a 0%, #1e293b 100%);
        color: white;
        padding: 2rem 0;
        box-shadow: var(--shadow-lg);
        position: relative;
        z-index: 50;
      }

      .header-content {
        max-width: 1200px;
        margin: 0 auto;
        padding: 0 1rem;
        display: flex;
        justify-content: space-between;
        align-items: center;
        flex-wrap: wrap;
        gap: 2rem;
      }

      .header-info h1 {
        font-size: 2rem;
        font-weight: 700;
        margin-bottom: 0.5rem;
      }

      .header-meta {
        color: rgba(255, 255, 255, 0.8);
        font-size: 0.9rem;
      }

      .certificate-badge {
        text-align: center;
      }

      .certificate-badge svg {
        width: 120px;
        height: 120px;
        filter: drop-shadow(0 4px 8px rgba(0,0,0,0.3));
      }

      .certificate-grade {
        margin-top: 0.5rem;
        font-size: 1.25rem;
        font-weight: 600;
        color: rgba(255, 255, 255, 0.9);
      }

      .sticky-nav {
        position: sticky;
        top: 0;
        background: white;
        border-bottom: 1px solid #e5e7eb;
        box-shadow: 0 2px 8px rgba(0,0,0,0.1);
        z-index: 40;
      }

      .nav-container {
        max-width: 1200px;
        margin: 0 auto;
        padding: 0 1rem;
        display: flex;
        align-items: center;
        overflow-x: auto;
        white-space: nowrap;
      }

      .nav-link {
        display: inline-block;
        padding: 1rem 1.5rem;
        text-decoration: none;
        color: var(--color-subtle);
        font-weight: 500;
        border-bottom: 3px solid transparent;
        transition: all 0.2s ease;
      }

      .nav-link:hover,
      .nav-link.active {
        color: var(--primary);
        border-bottom-color: var(--primary);
      }

      .main-content {
        padding: 2rem 0;
      }

      .container {
        max-width: 1200px;
        margin: 0 auto;
        padding: 0 1rem;
      }

      .section {
        background: var(--color-card);
        border-radius: var(--radius);
        box-shadow: var(--shadow);
        margin-bottom: 2rem;
        overflow: hidden;
        scroll-margin-top: 120px;
      }

      .section-header {
        background: linear-gradient(135deg, #f8fafc 0%, #e2e8f0 100%);
        border-bottom: 1px solid #e5e7eb;
        padding: 1.5rem;
        display: flex;
        align-items: center;
        gap: 0.75rem;
      }

      .section-header h2 {
        font-size: 1.5rem;
        font-weight: 600;
        color: var(--color-text);
      }

      .section-content {
        padding: 1.5rem;
      }

      .metrics-grid {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
        gap: 1rem;
        margin-bottom: 2rem;
      }

      .metric-card {
        background: linear-gradient(135deg, #ffffff 0%, #f8fafc 100%);
        border: 1px solid #e5e7eb;
        border-radius: var(--radius);
        padding: 1.5rem;
        text-align: center;
        transition: transform 0.2s ease, box-shadow 0.2s ease;
      }

      .metric-card:hover {
        transform: translateY(-2px);
        box-shadow: var(--shadow-lg);
      }

      .metric-value {
        font-size: 2.5rem;
        font-weight: 700;
        display: block;
        margin-bottom: 0.5rem;
      }

      .metric-label {
        color: var(--color-subtle);
        font-size: 0.875rem;
        font-weight: 500;
        text-transform: uppercase;
        letter-spacing: 0.05em;
      }

      .grade-badge {
        display: inline-block;
        padding: 0.5rem 1rem;
        border-radius: 50px;
        font-weight: 700;
        font-size: 1.25rem;
        color: white;
        text-shadow: 0 1px 2px rgba(0,0,0,0.3);
      }

      .grade-A { background: linear-gradient(135deg, #10b981 0%, #059669 100%); }
      .grade-B { background: linear-gradient(135deg, #3b82f6 0%, #2563eb 100%); }
      .grade-C { background: linear-gradient(135deg, #f59e0b 0%, #d97706 100%); }
      .grade-D { background: linear-gradient(135deg, #f97316 0%, #ea580c 100%); }
      .grade-F { background: linear-gradient(135deg, #ef4444 0%, #dc2626 100%); }

      .success { color: var(--success); }
      .warning { color: var(--warning); }
      .error { color: var(--error); }
      .info { color: var(--primary); }

      .data-table {
        width: 100%;
        border-collapse: collapse;
        margin-top: 1rem;
        border-radius: var(--radius);
        overflow: hidden;
        box-shadow: 0 1px 3px rgba(0,0,0,0.1);
      }

      .data-table th,
      .data-table td {
        padding: 1rem;
        text-align: left;
        border-bottom: 1px solid #f3f4f6;
      }

      .data-table th {
        background: #f8fafc;
        font-weight: 600;
        color: var(--color-text);
        border-bottom: 2px solid #e5e7eb;
      }

      .data-table tr:hover {
        background: #f9fafb;
      }

      .data-table tr:last-child td {
        border-bottom: none;
      }

      .page-status {
        font-weight: 600;
        padding: 0.25rem 0.75rem;
        border-radius: 20px;
        font-size: 0.75rem;
        text-transform: uppercase;
      }

      .status-passed {
        background: #ecfdf5;
        color: #065f46;
      }

      .status-failed {
        background: #fef2f2;
        color: #991b1b;
      }

      .status-crashed {
        background: #fffbeb;
        color: #92400e;
      }

      .no-data {
        text-align: center;
        color: var(--color-subtle);
        font-style: italic;
        padding: 2rem;
      }

      .issue-list {
        list-style: none;
        padding: 0;
      }

      .issue-item {
        padding: 1rem;
        border-left: 4px solid #e5e7eb;
        margin-bottom: 1rem;
        background: #f9fafb;
        border-radius: 0 var(--radius) var(--radius) 0;
      }

      .issue-item.critical { border-left-color: #dc2626; }
      .issue-item.error { border-left-color: #ef4444; }
      .issue-item.warning { border-left-color: #f59e0b; }
      .issue-item.notice { border-left-color: #3b82f6; }
      .issue-item.info { border-left-color: #6b7280; }
      
      .scrollable-issues-container {
        max-height: 720px;
        overflow-y: auto;
        border: 1px solid #e5e7eb;
        border-radius: var(--radius);
        background: #ffffff;
        padding: 0.5rem;
      }
      
      .scrollable-issues-container::-webkit-scrollbar {
        width: 8px;
      }
      
      .scrollable-issues-container::-webkit-scrollbar-track {
        background: #f1f5f9;
        border-radius: 4px;
      }
      
      .scrollable-issues-container::-webkit-scrollbar-thumb {
        background: #cbd5e1;
        border-radius: 4px;
      }
      
      .scrollable-issues-container::-webkit-scrollbar-thumb:hover {
        background: #94a3b8;
      }
      
      /* Scrollable table container */
      .scrollable-table-container {
        max-height: 720px;
        overflow-y: auto;
        overflow-x: auto;
        border: 1px solid #e5e7eb;
        border-radius: var(--radius);
        box-shadow: var(--shadow);
        background: white;
      }
      
      .scrollable-table-container::-webkit-scrollbar {
        width: 8px;
      }
      
      .scrollable-table-container::-webkit-scrollbar-track {
        background: #f1f5f9;
        border-radius: 4px;
      }
      
      .scrollable-table-container::-webkit-scrollbar-thumb {
        background: #cbd5e1;
        border-radius: 4px;
      }
      
      .scrollable-table-container::-webkit-scrollbar-thumb:hover {
        background: #94a3b8;
      }
      
      /* Value color coding */
      .value-excellent { color: #10b981; font-weight: 600; }
      .value-good { color: #059669; font-weight: 600; }
      .value-warning { color: #f59e0b; font-weight: 600; }
      .value-poor { color: #ef4444; font-weight: 600; }
      .value-critical { color: #dc2626; font-weight: 700; background: #fef2f2; padding: 2px 6px; border-radius: 4px; }
      
      /* Score badges */
      .score-badge {
        display: inline-block;
        padding: 4px 8px;
        border-radius: 6px;
        font-weight: 600;
        font-size: 0.8rem;
        min-width: 45px;
        text-align: center;
      }
      .score-a { background: #dcfce7; color: #166534; }
      .score-b { background: #fef3c7; color: #92400e; }
      .score-c { background: #fed7aa; color: #9a3412; }
      .score-d { background: #fecaca; color: #991b1b; }
      .score-f { background: #fef2f2; color: #dc2626; }
      
      /* Performance value specific coloring */
      .perf-excellent { color: #10b981; font-weight: 600; } /* < 1000ms LCP */
      .perf-good { color: #059669; font-weight: 600; }      /* < 2500ms LCP */
      .perf-poor { color: #f59e0b; font-weight: 600; }      /* < 4000ms LCP */
      .perf-critical { color: #ef4444; font-weight: 600; }  /* > 4000ms LCP */
      
      /* Content weight specific coloring */
      .size-small { color: #10b981; font-weight: 600; }     /* < 500KB */
      .size-medium { color: #059669; font-weight: 600; }    /* < 1MB */
      .size-large { color: #f59e0b; font-weight: 600; }     /* < 2MB */
      .size-huge { color: #ef4444; font-weight: 600; }      /* > 2MB */
      
      /* N/A values styling */
      .perf-na { color: #9ca3af; font-style: italic; font-weight: 500; }

      .footer {
        text-align: center;
        color: var(--color-subtle);
        font-size: 0.875rem;
        padding: 2rem 0;
        border-top: 1px solid #e5e7eb;
        margin-top: 2rem;
      }

      .detailed-issues-container {
        max-height: 720px;
        overflow-y: auto;
        border: 1px solid #e5e7eb;
        border-radius: var(--radius);
        background: #f9fafb;
        font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
        font-size: 0.85rem;
        line-height: 1.5;
      }

      .copy-button {
        background: var(--primary);
        color: white;
        border: none;
        padding: 0.5rem 1rem;
        border-radius: var(--radius);
        font-size: 0.875rem;
        cursor: pointer;
        margin-bottom: 1rem;
        transition: background-color 0.2s ease;
      }

      .copy-button:hover {
        background: #1d4ed8;
      }

      .copy-button.copied {
        background: var(--success);
      }

      .scrollable-issues-container {
        max-height: 720px;
        overflow: auto;
        border: 1px solid #e5e7eb;
        background: var(--color-card);
        border-radius: 8px;
        padding: 0.5rem;
      }
      
      @media (max-width: 768px) {
        .header-content {
          flex-direction: column;
          text-align: center;
        }
        
        .metrics-grid {
          grid-template-columns: 1fr;
        }
        
        .nav-container {
          justify-content: flex-start;
        }
      }
    `;
  }

  private renderHeader(data: EnhancedAuditResult, domain: string, certificateSVG: string): string {
    const timestamp = new Date(data.metadata.timestamp).toLocaleString();
    const pages = data.pages || [];
    const analyzedCount = pages.filter(p => p.status !== 'skipped').length;
    const skippedCount = pages.filter(p => p.status === 'skipped').length;
    const skipInfo = skippedCount > 0 ? ` (skipped: ${skippedCount})` : '';

    return `
      <div class="header">
        <div class="header-content">
          <div class="header-info">
            <h1>Website Audit Report</h1>
            <div class="header-meta">
              <div><strong>Domain:</strong> ${this.escape(domain)}</div>
              <div><strong>Generated:</strong> ${timestamp}</div>
              <div><strong>Pages Analyzed:</strong> ${analyzedCount} of ${data.summary.totalPages}${skipInfo}</div>
            </div>
          </div>
        </div>
      </div>
    `;
  }

  private renderNavigation(): string {
    return `
      <nav class="sticky-nav">
        <div class="nav-container">
          <a href="#summary" class="nav-link">Summary</a>
          <a href="#accessibility" class="nav-link">Accessibility</a>
          <a href="#performance" class="nav-link">Performance</a>
          <a href="#seo" class="nav-link">SEO</a>
          <a href="#contentweight" class="nav-link">Content Weight</a>
          <a href="#mobile" class="nav-link">Mobile</a>
        </div>
      </nav>
    `;
  }

  private renderSummary(data: EnhancedAuditResult): string {
    const s = data.summary;
    const successRate = s.successRate || 0;
    const duration = Math.round(data.metadata.duration / 1000);
    const pagesList = data.pages || [];
    const analyzedCount = pagesList.filter(p => p.status !== 'skipped').length;
    const skippedCount = pagesList.filter(p => p.status === 'skipped').length;

    return `
      <section id="summary" class="section">
        <div class="section-header">
          <h2>Executive Summary</h2>
        </div>
        <div class="section-content">
          <!-- Overview -->
          <div class="metrics-grid">
            <div class="metric-card" style="border: 2px solid var(--primary); background: linear-gradient(135deg, #f8fafc 0%, #e0f2fe 100%);">
              <div class="metric-value info" style="font-size: 3rem; font-weight: 800;">${s.overallScore || 0}/100</div>
              <div class="metric-label" style="font-weight: 600; color: var(--primary);">Overall Score</div>
              <small style="color: #6b7280; font-size: 0.75rem; margin-top: 0.5rem; display: block;">Weighted average of all analyses</small>
            </div>
            <div class="metric-card">
              <div class="metric-value success">${successRate}%</div>
              <div class="metric-label">Success Rate</div>
            </div>
            <div class="metric-card">
              <div class="metric-value info">${analyzedCount}/${s.totalPages}</div>
              <div class="metric-label">Pages Analyzed</div>
              ${skippedCount > 0 ? `<small style="color: #6b7280;">Skipped: ${skippedCount}</small>` : ''}
            </div>
            <div class="metric-card">
              <div class="metric-value info">${duration}s</div>
              <div class="metric-label">Analysis Duration</div>
            </div>
          </div>

          <!-- Outcomes -->
          <div class="metrics-grid" style="margin-top: 1rem;">
            <div class="metric-card">
              <div class="metric-value success">${s.passedPages}</div>
              <div class="metric-label">Pages Passed</div>
            </div>
            <div class="metric-card">
              <div class="metric-value error">${s.failedPages}</div>
              <div class="metric-label">Pages Failed</div>
            </div>
            <div class="metric-card">
              <div class="metric-value error">${s.totalErrors}</div>
              <div class="metric-label">Total Errors</div>
            </div>
            <div class="metric-card">
              <div class="metric-value warning">${s.totalWarnings}</div>
              <div class="metric-label">Total Warnings</div>
            </div>
          </div>

          <!-- Key KPIs -->
          ${(() => {
            const cwPages = (data.pages || []).filter((p: any) => p.contentWeight);
            const perfPages = (data.pages || []).filter((p: any) => p.performance);

            const avg = (arr: number[]) => arr.length ? Math.round(arr.reduce((a,b)=>a+b,0)/arr.length) : 0;
            const anyPositive = (arr: number[]) => arr.some(v => typeof v === 'number' && v > 0);

            // Content sizes
            const jsSizes = cwPages.map((p: any) => p.contentWeight?.resources?.javascript?.size || p.contentWeight?.resources?.js?.size || p.contentWeight?.resources?.javascript || p.contentWeight?.resources?.js || p.contentWeight?.javascript || 0);
            const imgSizes = cwPages.map((p: any) => p.contentWeight?.resources?.images?.size || p.contentWeight?.resources?.images || p.contentWeight?.images || 0);
            const avgJs = avg(jsSizes);
            const avgImg = avg(imgSizes);
            const jsHeavy = jsSizes.filter((s: number) => s > 500000).length; // > 500KB JS
            const imgHeavy = imgSizes.filter((s: number) => s > 1000000).length; // > 1MB images

            // Performance KPIs
            const reqs = perfPages.map((p: any) => p.performance?.metrics?.requestCount || p.performance?.requestCount || 0);
            const xfer = perfPages.map((p: any) => p.performance?.metrics?.transferSize || p.performance?.transferSize || 0);
            const avgReq = avg(reqs);
            const avgXfer = avg(xfer);

            if (cwPages.length === 0 && perfPages.length === 0) return '';

            const reqClass = avgReq <= 60 ? 'success' : avgReq <= 100 ? 'warning' : 'error';
            const xferClass = avgXfer <= 1200000 ? 'success' : avgXfer <= 2500000 ? 'warning' : 'error';

            const fmtBytes = (val: number, hasData: boolean) => hasData ? this.formatBytes(val) : '<span class="perf-na">N/A</span>';
            const fmtCount = (val: number, hasData: boolean) => hasData ? String(val) : '<span class="perf-na">N/A</span>';

            const hasJsData = cwPages.length > 0;
            const hasImgData = cwPages.length > 0;
            const hasReqData = perfPages.length > 0;
            const hasXferData = perfPages.length > 0;

            return `
              <div style="margin-top: 1rem;">
                <h3 style="margin-bottom: 0.75rem;">Key KPIs</h3>
                <div class="metrics-grid" style="grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));">
                  <div class="metric-card">
                    <div class="metric-value ${hasJsData ? (avgJs <= 300000 ? 'success' : avgJs <= 600000 ? 'warning' : 'error') : 'info'}">${fmtBytes(avgJs, hasJsData)}</div>
                    <div class="metric-label">Avg JS Size/Page</div>
                  </div>
                  <div class="metric-card">
                    <div class="metric-value ${hasImgData ? (avgImg <= 800000 ? 'success' : avgImg <= 1600000 ? 'warning' : 'error') : 'info'}">${fmtBytes(avgImg, hasImgData)}</div>
                    <div class="metric-label">Avg Images Size/Page</div>
                  </div>
                  <div class="metric-card">
                    <div class="metric-value ${hasReqData ? reqClass : 'info'}">${fmtCount(avgReq, hasReqData)}</div>
                    <div class="metric-label">Avg Requests/Page</div>
                  </div>
                  <div class="metric-card">
                    <div class="metric-value ${hasXferData ? xferClass : 'info'}">${fmtBytes(avgXfer, hasXferData)}</div>
                    <div class="metric-label">Avg Transfer/Page</div>
                  </div>
                  ${cwPages.length > 0 ? `
                  <div class="metric-card">
                    <div class="metric-value ${jsHeavy > 0 ? 'warning' : 'success'}">${jsHeavy}</div>
                    <div class="metric-label">JS-heavy Pages (>500KB)</div>
                  </div>
                  <div class="metric-card">
                    <div class="metric-value ${imgHeavy > 0 ? 'warning' : 'success'}">${imgHeavy}</div>
                    <div class="metric-label">Image-heavy Pages (>1MB)</div>
                  </div>` : ''}
                </div>
              </div>
            `;
          })()}
          
          <!-- Overall Score Breakdown -->
          <div style="margin-top: 2rem;">
            <h3 style="margin-bottom: 1rem;">Overall Score Breakdown</h3>
            <p style="color: #6b7280; margin-bottom: 1rem; font-size: 0.9rem;">The Overall Score is calculated as a weighted average of all analyses. Each category contributes based on its importance for web quality:</p>
            ${this.renderScoreBreakdown(data)}
          </div>
        </div>
      </section>
    `;
  }

  private renderAccessibilitySection(data: EnhancedAuditResult): string {
    // Include pages with accessibility data, excluding skipped (redirected) pages from details
    const accessibilityPages = data.pages.filter(p => p.accessibility && p.status !== 'skipped' && (p.accessibility.score > 0 || (p.accessibility.errors && p.accessibility.errors.length > 0) || (p.accessibility.warnings && p.accessibility.warnings.length > 0)));
    const failedPages = data.pages.filter(p => p.status === 'skipped' && p.accessibility);
    
    if (accessibilityPages.length === 0 && failedPages.length === 0) {
      return `
        <section id="accessibility" class="section">
          <div class="section-header">
            <h2>Accessibility Analysis</h2>
          </div>
          <div class="section-content">
            <div class="no-data">No accessibility data available</div>
          </div>
        </section>
      `;
    }

    // Calculate comprehensive accessibility metrics
    const totalErrors = accessibilityPages.reduce((sum, p) => sum + (p.accessibility.errors?.length || 0), 0);
    const totalWarnings = accessibilityPages.reduce((sum, p) => sum + (p.accessibility.warnings?.length || 0), 0);
    const totalNotices = accessibilityPages.reduce((sum, p) => sum + (p.accessibility.notices?.length || 0), 0);
    const avgScore = accessibilityPages.length > 0 
      ? accessibilityPages.reduce((sum, p) => sum + (p.accessibility.score || 0), 0) / accessibilityPages.length 
      : 0;
    
    // Additional comprehensive metrics
    const totalImagesWithoutAlt = accessibilityPages.reduce((sum, p) => {
      return sum + (this.extractMetricFromIssues(p.accessibility, 'alt') || (p as any).imagesWithoutAlt || 0);
    }, 0);
    
    const totalButtonsWithoutLabel = accessibilityPages.reduce((sum, p) => {
      return sum + (this.extractMetricFromIssues(p.accessibility, 'button') || (p as any).buttonsWithoutLabel || 0);
    }, 0);
    
    const totalContrastIssues = accessibilityPages.reduce((sum, p) => {
      return sum + (this.extractMetricFromIssues(p.accessibility, 'contrast') || (p as any).colorContrastIssues?.length || 0);
    }, 0);
    
    // Analyze error/warning distribution
    const excellentPages = accessibilityPages.filter(p => (p.accessibility.score || 0) >= 90).length;
    const goodPages = accessibilityPages.filter(p => {
      const score = p.accessibility.score || 0;
      return score >= 75 && score < 90;
    }).length;
    const needsImprovementPages = accessibilityPages.filter(p => (p.accessibility.score || 0) < 75).length;
    const criticalPages = accessibilityPages.filter(p => (p.accessibility.score || 0) < 40).length;
    
    // Most common accessibility issues analysis
    const issuesByType: Record<string, { count: number; severity: string }> = {};
    accessibilityPages.forEach(page => {
      const allIssues = [
        ...(page.accessibility.errors || []),
        ...(page.accessibility.warnings || [])
      ];
      
      allIssues.forEach((issue: any) => {
        let issueType = 'Other';
        if (typeof issue === 'string') {
          // Try to categorize string-based issues
          if (issue.includes('alt') || issue.includes('image')) issueType = 'Images & Alt Text';
          else if (issue.includes('heading') || issue.includes('h1') || issue.includes('h2')) issueType = 'Headings';
          else if (issue.includes('contrast') || issue.includes('color')) issueType = 'Color Contrast';
          else if (issue.includes('form') || issue.includes('label')) issueType = 'Forms';
          else if (issue.includes('link') || issue.includes('anchor')) issueType = 'Links';
          else if (issue.includes('keyboard') || issue.includes('focus')) issueType = 'Keyboard Navigation';
        } else if (issue.code) {
          // Categorize based on pa11y/axe codes
          if (issue.code.includes('image') || issue.code.includes('alt')) issueType = 'Images & Alt Text';
          else if (issue.code.includes('heading')) issueType = 'Headings';
          else if (issue.code.includes('contrast') || issue.code.includes('color')) issueType = 'Color Contrast';
          else if (issue.code.includes('form') || issue.code.includes('label')) issueType = 'Forms';
          else if (issue.code.includes('link')) issueType = 'Links';
          else if (issue.code.includes('keyboard') || issue.code.includes('focus')) issueType = 'Keyboard Navigation';
        }
        
        if (!issuesByType[issueType]) {
          issuesByType[issueType] = {
            count: 0,
            severity: (typeof issue === 'string' || issue.type === 'error') ? 'error' : 'warning'
          };
        }
        issuesByType[issueType].count++;
      });
    });
    
    const topIssueTypes = Object.entries(issuesByType)
      .sort(([,a], [,b]) => b.count - a.count)
      .slice(0, 4);

    return `
      <section id="accessibility" class="section">
        <div class="section-header">
          <h2>Accessibility Analysis
            <a href="https://github.com/casoon/AuditMySite/blob/main/docs/accessibility.html" target="_blank" 
               style="margin-left: 0.5rem; color: #6b7280; text-decoration: none; font-size: 0.875rem;"
               title="What do these accessibility metrics mean?">
              <i style="font-style: normal; border: 1px solid; border-radius: 50%; padding: 2px 6px; font-size: 0.75rem;">i</i>
            </a>
          </h2>
        </div>
        <div class="section-content">
          <!-- Core Accessibility Metrics -->
          <div class="metrics-grid">
            <div class="metric-card">
              <div class="metric-value ${avgScore >= 90 ? 'success' : avgScore >= 75 ? 'warning' : 'error'}">${Math.round(avgScore)}/100</div>
              <div class="metric-label">Average A11y Score</div>
            </div>
            <div class="metric-card">
              <div class="metric-value ${totalErrors === 0 ? 'success' : totalErrors < 5 ? 'warning' : 'error'}">${totalErrors}</div>
              <div class="metric-label">Critical Issues</div>
            </div>
            <div class="metric-card">
              <div class="metric-value ${totalWarnings === 0 ? 'success' : totalWarnings < 10 ? 'warning' : 'error'}">${totalWarnings}</div>
              <div class="metric-label">Warnings</div>
            </div>
            <div class="metric-card">
              <div class="metric-value ${totalImagesWithoutAlt === 0 ? 'success' : totalImagesWithoutAlt < 5 ? 'warning' : 'error'}">${totalImagesWithoutAlt}</div>
              <div class="metric-label">Images Missing Alt</div>
            </div>
            <div class="metric-card">
              <div class="metric-value ${totalButtonsWithoutLabel === 0 ? 'success' : totalButtonsWithoutLabel < 3 ? 'warning' : 'error'}">${totalButtonsWithoutLabel}</div>
              <div class="metric-label">Unlabeled Buttons</div>
            </div>
            <div class="metric-card">
              <div class="metric-value ${totalContrastIssues === 0 ? 'success' : totalContrastIssues < 5 ? 'warning' : 'error'}">${totalContrastIssues}</div>
              <div class="metric-label">Contrast Issues</div>
            </div>
            <div class="metric-card">
              <div class="metric-value info">${totalNotices}</div>
              <div class="metric-label">Notices</div>
            </div>
            <div class="metric-card">
              <div class="metric-value info">${accessibilityPages.length}</div>
              <div class="metric-label">Pages Analyzed</div>
            </div>
          </div>
          
          <!-- Page Quality Distribution -->
          <div style="margin-top: 2rem;">
            <h3 style="margin-bottom: 1rem;">Accessibility Quality Distribution</h3>
            <div class="metrics-grid" style="grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 1rem;">
              <div class="metric-card">
                <div class="metric-value ${excellentPages === accessibilityPages.length ? 'success' : excellentPages > accessibilityPages.length * 0.5 ? 'warning' : 'error'}">${excellentPages}</div>
                <div class="metric-label">Excellent (90+)</div>
              </div>
              <div class="metric-card">
                <div class="metric-value ${goodPages === 0 && excellentPages === accessibilityPages.length ? 'success' : 'info'}">${goodPages}</div>
                <div class="metric-label">Good (75-89)</div>
              </div>
              <div class="metric-card">
                <div class="metric-value ${needsImprovementPages === 0 ? 'success' : needsImprovementPages < accessibilityPages.length * 0.2 ? 'warning' : 'error'}">${needsImprovementPages}</div>
                <div class="metric-label">Needs Work (40-74)</div>
              </div>
              <div class="metric-card">
                <div class="metric-value ${criticalPages === 0 ? 'success' : 'error'}">${criticalPages}</div>
                <div class="metric-label">Critical (<40)</div>
              </div>
              ${failedPages.length > 0 ? `
              <div class="metric-card">
                <div class="metric-value warning">${failedPages.length}</div>
                <div class="metric-label">Skipped (Redirects)</div>
              </div>` : ''}
            </div>
          </div>
          
          ${topIssueTypes.length > 0 ? `
            <div style="margin-top: 2rem;">
              <h3 style="margin-bottom: 1rem;">Most Common Accessibility Issues</h3>
              <div class="metrics-grid" style="grid-template-columns: repeat(auto-fit, minmax(220px, 1fr)); gap: 1rem;">
                ${topIssueTypes.map(([issueType, data]) => {
                  const severityClass = (data as any).severity === 'error' ? 'error' : 'warning';
                  return `
                    <div class="metric-card">
                      <div class="metric-value ${severityClass}">${(data as any).count}</div>
                      <div class="metric-label">${issueType}</div>
                      <small style="color: #6b7280; font-size: 0.8rem; margin-top: 0.5rem; display: block;">
                        ${(data as any).severity === 'error' ? 'Critical' : 'Needs attention'}
                      </small>
                    </div>
                  `;
                }).join('')}
              </div>
            </div>
          ` : ''}
          
          <div style="margin-top: 2rem;">
            <h3 style="margin-bottom: 1rem;">Per-Page Accessibility Details</h3>
            <div class="scrollable-table-container" style="max-height: 720px;">
              <table class="data-table">
                <thead>
                  <tr>
                    <th style="position: sticky; left: 0; background: #f8fafc; z-index: 11;">Page</th>
                    <th>A11y Score</th>
                    <th>Errors</th>
                    <th>Warnings</th>
                    <th>Notices</th>
                    <th>Images w/o Alt</th>
                    <th>Buttons w/o Label</th>
                    <th>Contrast Issues</th>
                    <th>WCAG Level</th>
                    <th>ARIA Issues</th>
                  </tr>
                </thead>
                <tbody>
                  ${accessibilityPages.map(p => {
                    const score = p.accessibility?.score || 0;
                    const errors = p.accessibility?.errors?.length || 0;
                    const warnings = p.accessibility?.warnings?.length || 0;
                    const notices = p.accessibility?.notices?.length || 0;
                    // Extract common accessibility metrics from issues or separate fields
                    const imagesWithoutAlt = this.extractMetricFromIssues(p.accessibility, 'alt') || (p as any).imagesWithoutAlt || 0;
                    const buttonsWithoutLabel = this.extractMetricFromIssues(p.accessibility, 'button') || (p as any).buttonsWithoutLabel || 0;
                    const contrastIssues = this.extractMetricFromIssues(p.accessibility, 'contrast') || (p as any).colorContrastIssues?.length || 0;
                    
                    // Professional WCAG analysis
                    const wcagLevel = this.calculateWCAGLevel(score, errors, warnings);
                    const ariaIssues = this.calculateARIAIssues(p.accessibility, buttonsWithoutLabel, imagesWithoutAlt);
                    
                    const getScoreClass = (score: number) => score >= 90 ? 'value-excellent' : score >= 75 ? 'value-good' : score >= 60 ? 'value-warning' : score >= 40 ? 'value-poor' : 'value-critical';
                    const getIssueClass = (count: number) => count === 0 ? 'value-excellent' : count < 3 ? 'value-warning' : 'value-critical';
                    const getWCAGClass = (level: string) => level === 'AAA' ? 'value-excellent' : level === 'AA' ? 'value-good' : level === 'A' ? 'value-warning' : 'value-critical';
                    
                    return `
                      <tr>
                        <td style="max-width: 200px;">
                          <strong>${this.escape(p.title || 'Untitled')}</strong><br/>
                          <small style="color: #6b7280; font-size: 0.7rem;">${this.escape(p.url.length > 40 ? '...' + p.url.slice(-37) : p.url)}</small>
                        </td>
                        <td class="${getScoreClass(score)}">${score}/100</td>
                        <td class="${getIssueClass(errors)}">${errors}</td>
                        <td class="${getIssueClass(warnings)}">${warnings}</td>
                        <td class="${notices === 0 ? 'value-excellent' : 'value-good'}">${notices}</td>
                        <td class="${getIssueClass(imagesWithoutAlt)}">${imagesWithoutAlt}</td>
                        <td class="${getIssueClass(buttonsWithoutLabel)}">${buttonsWithoutLabel}</td>
                        <td class="${getIssueClass(contrastIssues)}">${contrastIssues}</td>
                        <td class="${getWCAGClass(wcagLevel)}">${wcagLevel}</td>
                        <td class="${getIssueClass(ariaIssues)}">${ariaIssues}</td>
                      </tr>
                    `;
                  }).join('')}
                </tbody>
              </table>
            </div>
          </div>
          
          <!-- Professional WCAG 2.1 Compliance Analysis -->
          <div style="margin-top: 2rem;">
            <h3 style="margin-bottom: 1rem;">WCAG 2.1 Compliance Analysis</h3>
            <div class="metrics-grid" style="grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 1rem;">
              ${this.renderWCAGComplianceMetrics(accessibilityPages)}
            </div>
          </div>
          
          <!-- ARIA Implementation Analysis -->
          <div style="margin-top: 2rem;">
            <h3 style="margin-bottom: 1rem;">ARIA Implementation Analysis</h3>
            <div class="metrics-grid" style="grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 1rem;">
              ${this.renderARIAAnalysisMetrics(accessibilityPages)}
            </div>
          </div>
          
          <!-- Detailed Accessibility Issues -->
          <div style="margin-top: 2rem;">
            <h3 style="margin-bottom: 1rem;">Detailed Accessibility Issues & Recommendations</h3>
            ${this.renderAccessibilityDetailedIssues(data.pages)}
          </div>
          
          ${failedPages.length > 0 ? this.renderSkippedPagesInfo(failedPages) : ''}
        </div>
      </section>
    `;
  }

  private renderPerformanceSection(data: EnhancedAuditResult): string {
    // Include pages with any performance data, even if the overall test failed
    const performancePages = data.pages.filter(p => 
      p.performance || 
      (p as any).enhancedPerformance || 
      (p.status === 'failed' && p.performance)
    );
    
    if (performancePages.length === 0) {
      return `
        <section id="performance" class="section">
          <div class="section-header">
            <h2>Performance Analysis</h2>
          </div>
          <div class="section-content">
            <div class="no-data">No performance data available</div>
          </div>
        </section>
      `;
    }

    // Helper function to safely get numeric value
    const getNumericValue = (value: any): number => {
      if (value === null || value === undefined || isNaN(value)) return 0;
      return Number(value) || 0;
    };
    
    // Helper function to safely calculate average
    const safeAvg = (values: number[]): number => {
      const validValues = values.filter(v => !isNaN(v) && v !== null && v !== undefined);
      return validValues.length > 0 ? validValues.reduce((sum, v) => sum + v, 0) / validValues.length : 0;
    };
    
    const avgScore = safeAvg(performancePages.map(p => getNumericValue(p.performance.score)));
    const avgLCP = safeAvg(performancePages.map(p => getNumericValue(p.performance.coreWebVitals?.largestContentfulPaint)));
    const avgFCP = safeAvg(performancePages.map(p => getNumericValue(p.performance.coreWebVitals?.firstContentfulPaint)));
    const avgCLS = safeAvg(performancePages.map(p => getNumericValue(p.performance.coreWebVitals?.cumulativeLayoutShift)));
    const avgTTFB = safeAvg(performancePages.map(p => getNumericValue(p.performance.coreWebVitals?.timeToFirstByte)));
    const avgDomContentLoaded = safeAvg(performancePages.map(p => getNumericValue(p.performance.metrics?.domContentLoaded)));
    const avgLoadComplete = safeAvg(performancePages.map(p => getNumericValue(p.performance.metrics?.loadComplete)));

    // Synthesize performance issues based on heuristics if none provided
    performancePages.forEach(p => {
      if (!p.performance) return;
      if (!Array.isArray(p.performance.issues)) p.performance.issues = [];
      const issues = p.performance.issues;
      const LCP = getNumericValue(p.performance.coreWebVitals?.largestContentfulPaint);
      const FCP = getNumericValue(p.performance.coreWebVitals?.firstContentfulPaint);
      const CLS = getNumericValue(p.performance.coreWebVitals?.cumulativeLayoutShift);
      const TTFB = getNumericValue(p.performance.coreWebVitals?.timeToFirstByte);

      if (LCP > 4000) issues.push({ type: 'error', message: `High LCP: ${Math.round(LCP)}ms (target <= 2500ms)` });
      else if (LCP > 2500) issues.push({ type: 'warning', message: `LCP could be improved: ${Math.round(LCP)}ms (target <= 2500ms)` });

      if (CLS > 0.25) issues.push({ type: 'error', message: `High CLS: ${CLS.toFixed(3)} (target <= 0.10)` });
      else if (CLS > 0.10) issues.push({ type: 'warning', message: `CLS could be improved: ${CLS.toFixed(3)} (target <= 0.10)` });

      if (TTFB > 1000) issues.push({ type: 'error', message: `High TTFB: ${Math.round(TTFB)}ms (target <= 500ms)` });
      else if (TTFB > 500) issues.push({ type: 'warning', message: `TTFB could be improved: ${Math.round(TTFB)}ms (target <= 500ms)` });

      if (FCP > 3000) issues.push({ type: 'warning', message: `Slow FCP: ${Math.round(FCP)}ms (target <= 1800ms)` });
    });

    const totalPerformanceIssues = performancePages.reduce((sum, p) => sum + (p.performance.issues?.length || 0), 0);

    return `
      <section id="performance" class="section">
        <div class="section-header">
          <h2>Performance Analysis
            <a href="https://github.com/casoon/AuditMySite/blob/main/docs/performance.html" target="_blank" 
               style="margin-left: 0.5rem; color: #6b7280; text-decoration: none; font-size: 0.875rem;"
               title="Understanding Core Web Vitals and performance metrics">
              <i style="font-style: normal; border: 1px solid; border-radius: 50%; padding: 2px 6px; font-size: 0.75rem;">i</i>
            </a>
          </h2>
        </div>
        <div class="section-content">
          ${(() => {
            const ms = (data as any).systemPerformance?.measurementSettings;
            if (!ms) return '';
            const onOff = ms.psiProfile ? 'Enabled' : 'Disabled';
            const cpu = ms.cpuThrottlingRate ? `CPU×${ms.cpuThrottlingRate}` : 'CPU×1';
            const net = ms.network ? `${ms.network.latencyMs}ms, ${ms.network.downloadKbps}kbps down, ${ms.network.uploadKbps}kbps up` : 'Default network';
            return `
              <div style="margin-bottom: 1rem; color: #6b7280; font-size: 0.9rem;">
                <strong>Measurement Settings:</strong> PSI-like Profile ${onOff} · ${cpu} · ${net}
              </div>
            `;
          })()}
          <div class="metrics-grid">
            <div class="metric-card">
              <div class="metric-value info">${Math.round(avgScore)}/100</div>
              <div class="metric-label">Average Score</div>
            </div>
            <div class="metric-card">
              <div class="metric-value ${avgLCP < 2500 ? 'success' : avgLCP < 4000 ? 'warning' : 'error'}">${Math.round(avgLCP)}ms</div>
              <div class="metric-label">Avg LCP</div>
            </div>
            <div class="metric-card">
              <div class="metric-value ${avgFCP < 1800 ? 'success' : avgFCP < 3000 ? 'warning' : 'error'}">${Math.round(avgFCP)}ms</div>
              <div class="metric-label">Avg FCP</div>
            </div>
            <div class="metric-card">
              <div class="metric-value ${avgCLS < 0.1 ? 'success' : avgCLS < 0.25 ? 'warning' : 'error'}">${isNaN(avgCLS) || avgCLS === 0 ? '0.000' : avgCLS.toFixed(3)}</div>
              <div class="metric-label">Avg CLS</div>
            </div>
            <div class="metric-card">
              <div class="metric-value info">${Math.round(avgTTFB)}ms</div>
              <div class="metric-label">Avg TTFB</div>
            </div>
            <div class="metric-card">
              <div class="metric-value info">${Math.round(avgDomContentLoaded)}ms</div>
              <div class="metric-label">Avg DOM Loaded</div>
            </div>
            <div class="metric-card">
              <div class="metric-value info">${Math.round(avgLoadComplete)}ms</div>
              <div class="metric-label">Avg Load Complete</div>
            </div>
            <div class="metric-card">
              <div class="metric-value warning">${totalPerformanceIssues}</div>
              <div class="metric-label">Total Issues</div>
            </div>
            <div class="metric-card">
              <div class="metric-value info">${performancePages.length}</div>
              <div class="metric-label">Pages Analyzed</div>
            </div>
          </div>
          
          <div style="margin-top: 2rem;">
            <h3 style="margin-bottom: 1rem;">Per-Page Performance Details (Desktop & Mobile)</h3>
            <div class="scrollable-table-container" style="max-height: 720px;">
              <table class="data-table">
                <thead>
                  <tr>
                    <th style="position: sticky; left: 0; background: #f8fafc; z-index: 11; width: 200px;">Page</th>
                    <th style="position: sticky; left: 200px; background: #f8fafc; z-index: 10; width: 80px;">Device</th>
                    <th title="Overall Performance Score (0-100)">Score</th>
                    <th title="Largest Contentful Paint - Time when the largest content element becomes visible"><abbr title="Largest Contentful Paint">LCP</abbr></th>
                    <th title="First Contentful Paint - Time when first content appears"><abbr title="First Contentful Paint">FCP</abbr></th>
                    <th title="Cumulative Layout Shift - Measure of visual stability"><abbr title="Cumulative Layout Shift">CLS</abbr></th>
                    <th title="Time to First Byte - Server response time"><abbr title="Time to First Byte">TTFB</abbr></th>
                  </tr>
                </thead>
                <tbody>
                  ${performancePages.map(p => {
                    // Desktop performance data
                    const desktopScore = p.performance?.score || 0;
                    const desktopLCP = p.performance?.coreWebVitals?.largestContentfulPaint || 0;
                    const desktopFCP = p.performance?.coreWebVitals?.firstContentfulPaint || 0;
                    const desktopCLS = Number(p.performance?.coreWebVitals?.cumulativeLayoutShift?.value || p.performance?.coreWebVitals?.cumulativeLayoutShift || 0);
                    const desktopTTFB = p.performance?.coreWebVitals?.timeToFirstByte || 0;
                    
                    // Mobile performance data from mobile friendliness analysis (which includes mobile performance)
                    const mobilePerf = (p as any).mobilePerformance || (p as any).enhancedPerformance?.mobilePerformance || (p as any).mobileFriendliness?.performance;
                    const mobileScore = (p as any).mobileFriendliness?.performance?.score || this.calculateMobilePerformanceScore(mobilePerf) || 0;
                    const mobileLCP = mobilePerf?.coreWebVitals?.lcp || mobilePerf?.lcp || (p as any).mobileFriendliness?.performance?.lcp || 0;
                    const mobileFCP = mobilePerf?.coreWebVitals?.fcp || mobilePerf?.fcp || (p as any).mobileFriendliness?.performance?.fcp || 0;
                    const mobileCLS = mobilePerf?.coreWebVitals?.cls || mobilePerf?.cls || (p as any).mobileFriendliness?.performance?.cls || 0;
                    const mobileTTFB = mobilePerf?.coreWebVitals?.ttfb || mobilePerf?.ttfb || (p as any).mobileFriendliness?.performance?.ttfb || 0;
                    
                    const getScoreClass = (score: number) => score >= 90 ? 'value-excellent' : score >= 75 ? 'value-good' : score >= 60 ? 'value-warning' : score >= 40 ? 'value-poor' : 'value-critical';
                    const getLCPClass = (lcp: number) => lcp <= 1000 ? 'perf-excellent' : lcp <= 2500 ? 'perf-good' : lcp <= 4000 ? 'perf-poor' : 'perf-critical';
                    const getCLSClass = (cls: number) => cls <= 0.1 ? 'perf-excellent' : cls <= 0.25 ? 'perf-poor' : 'perf-critical';
                    const getTTFBClass = (ttfb: number) => ttfb <= 200 ? 'perf-excellent' : ttfb <= 500 ? 'perf-good' : ttfb <= 1000 ? 'perf-poor' : 'perf-critical';
                    
                    return `
                      <!-- Desktop Row -->
                      <tr>
                        <td style="max-width: 200px; position: sticky; left: 0; background: #f8fafc; border-right: 2px solid #e5e7eb; z-index: 9;" rowspan="2">
                          <strong>${this.escape(p.title || 'Untitled')}</strong><br/>
                          <small style="color: #6b7280; font-size: 0.7rem;">${this.escape(p.url.length > 30 ? '...' + p.url.slice(-27) : p.url)}</small>
                        </td>
                        <td style="position: sticky; left: 200px; background: #f8fafc; font-weight: 500; z-index: 8; border-right: 1px solid #e5e7eb;">
                          <div style="display: flex; align-items: center; gap: 0.5rem;">
                            <span style="font-size: 1.1rem;">🖥️</span>
                            <span>Desktop</span>
                          </div>
                        </td>
                        <td class="${getScoreClass(desktopScore)}">${desktopScore}/100</td>
                        <td class="${getLCPClass(desktopLCP)}">${(desktopLCP !== null && desktopLCP !== undefined && !isNaN(desktopLCP)) ? Math.round(desktopLCP) + 'ms' : 'N/A'}</td>
                        <td class="${getLCPClass(desktopFCP)}">${(desktopFCP !== null && desktopFCP !== undefined && !isNaN(desktopFCP)) ? Math.round(desktopFCP) + 'ms' : 'N/A'}</td>
                        <td class="${getCLSClass(desktopCLS)}">${(desktopCLS !== null && desktopCLS !== undefined && !isNaN(desktopCLS)) ? desktopCLS.toFixed(3) : 'N/A'}</td>
                        <td class="${getTTFBClass(desktopTTFB)}">${(desktopTTFB !== null && desktopTTFB !== undefined && !isNaN(desktopTTFB)) ? Math.round(desktopTTFB) + 'ms' : 'N/A'}</td>
                      </tr>
                      <!-- Mobile Row -->
                      <tr style="border-bottom: 2px solid #e5e7eb;">
                        <td style="position: sticky; left: 200px; background: #fef3c7; font-weight: 500; z-index: 8; border-right: 1px solid #e5e7eb;">
                          <div style="display: flex; align-items: center; gap: 0.5rem;">
                            <span style="font-size: 1.1rem;">📱</span>
                            <span>Mobile</span>
                          </div>
                        </td>
                        <td class="${getScoreClass(mobileScore)}">${(mobileScore !== null && mobileScore !== undefined && !isNaN(mobileScore)) ? mobileScore + '/100' : 'N/A'}</td>
                        <td class="${getLCPClass(mobileLCP)}">${(mobileLCP !== null && mobileLCP !== undefined && !isNaN(mobileLCP)) ? Math.round(mobileLCP) + 'ms' : 'N/A'}</td>
                        <td class="${getLCPClass(mobileFCP)}">${(mobileFCP !== null && mobileFCP !== undefined && !isNaN(mobileFCP)) ? Math.round(mobileFCP) + 'ms' : 'N/A'}</td>
                        <td class="${getCLSClass(mobileCLS)}">${(mobileCLS !== null && mobileCLS !== undefined && !isNaN(mobileCLS)) ? mobileCLS.toFixed(3) : 'N/A'}</td>
                        <td class="${getTTFBClass(mobileTTFB)}">${(mobileTTFB !== null && mobileTTFB !== undefined && !isNaN(mobileTTFB)) ? Math.round(mobileTTFB) + 'ms' : 'N/A'}</td>
                      </tr>
                    `;
                  }).join('')}
                </tbody>
              </table>
            </div>
          </div>
          
          <!-- Omitted issues-by-severity for performance to avoid duplication with table above -->

          <div style="margin-top: 1.5rem; color: #6b7280; font-size: 0.9rem;">
            <h3 style="margin-bottom: 0.5rem; color: var(--color-text);">Heuristics Used</h3>
            <ul style="margin-left: 1rem; list-style: disc;">
              <li><strong>LCP</strong> (Largest Contentful Paint): Warning > 2500ms, Error > 4000ms</li>
              <li><strong>TTFB</strong> (Time To First Byte): Warning > 500ms, Error > 1000ms</li>
              <li><strong>CLS</strong> (Cumulative Layout Shift): Warning > 0.10, Error > 0.25</li>
              <li><strong>FCP</strong> (First Contentful Paint): Warning > 3000ms</li>
              <li><strong>Requests/Page</strong>: Warning > 60, Error > 100</li>
              <li><strong>Transfer/Page</strong>: Warning > 1.2MB, Error > 2.5MB</li>
            </ul>
          </div>
        </div>
      </section>
    `;
  }

  /**
   * Calculate a mobile performance score based on available metrics
   * Now primarily used as fallback when direct score is not available
   */
  private calculateMobilePerformanceScore(mobilePerf: any): number {
    if (!mobilePerf) return 0;
    
    // If score is already calculated from MobilePerformanceCollector, use it
    if (mobilePerf.score && mobilePerf.score > 0) {
      return mobilePerf.score;
    }
    
    // Fallback calculation for legacy data structures
    let score = 100;
    
    // Handle both old structure (lcp, fcp, etc. directly) and new structure (coreWebVitals.lcp, etc.)
    const lcp = mobilePerf.coreWebVitals?.lcp || mobilePerf.lcp || 0;
    const fcp = mobilePerf.coreWebVitals?.fcp || mobilePerf.fcp || 0;
    const cls = mobilePerf.coreWebVitals?.cls || mobilePerf.cls || 0;
    const ttfb = mobilePerf.coreWebVitals?.ttfb || mobilePerf.ttfb || 0;
    
    // LCP scoring (35% weight - more critical for mobile)
    if (lcp > 0) {
      if (lcp > 4000) score -= 35;
      else if (lcp > 2500) score -= 25;
      else if (lcp > 2000) score -= 15;
      else if (lcp > 1800) score -= 5;
    }
    
    // FCP scoring (25% weight)
    if (fcp > 0) {
      if (fcp > 3000) score -= 25;
      else if (fcp > 2000) score -= 18;
      else if (fcp > 1500) score -= 10;
      else if (fcp > 1200) score -= 5;
    }
    
    // TTFB scoring (25% weight - important for mobile networks)
    if (ttfb > 0) {
      if (ttfb > 1000) score -= 25;
      else if (ttfb > 500) score -= 18;
      else if (ttfb > 300) score -= 10;
      else if (ttfb > 200) score -= 5;
    }
    
    // CLS scoring (15% weight)
    if (cls > 0) {
      if (cls > 0.25) score -= 15;
      else if (cls > 0.1) score -= 10;
      else if (cls > 0.05) score -= 5;
    }
    
    return Math.max(0, Math.round(score));
  }
  
  private renderSEOSection(data: EnhancedAuditResult): string {
    // Include pages with any SEO data, even if the overall test failed
    const seoPages = data.pages.filter(p => 
      p.seo || 
      (p as any).enhancedSEO ||
      (p.status === 'failed' && p.seo)
    );
    
    if (seoPages.length === 0) {
      return `
        <section id="seo" class="section">
          <div class="section-header">
            <h2>SEO Analysis</h2>
          </div>
          <div class="section-content">
            <div class="no-data">No SEO data available</div>
          </div>
        </section>
      `;
    }

    const avgScore = seoPages.reduce((sum, p) => sum + (p.seo.score || 0), 0) / seoPages.length;
    const totalIssues = seoPages.reduce((sum, p) => sum + (p.seo.issues?.length || 0), 0);
    
    // Calculate additional SEO metrics
    const totalImages = seoPages.reduce((sum, p) => sum + (p.seo.images?.total || 0), 0);
    const totalMissingAlt = seoPages.reduce((sum, p) => sum + (p.seo.images?.missingAlt || 0), 0);
    const totalEmptyAlt = seoPages.reduce((sum, p) => sum + (p.seo.images?.emptyAlt || 0), 0);
    const avgH1Count = seoPages.reduce((sum, p) => sum + (p.seo.headings?.h1?.length || 0), 0) / seoPages.length;
    const pagesWithOptimalTitle = seoPages.filter(p => p.seo.metaTags?.title?.optimal).length;
    const pagesWithOptimalDesc = seoPages.filter(p => p.seo.metaTags?.description?.optimal).length;

    return `
      <section id="seo" class="section">
        <div class="section-header">
          <h2>SEO Analysis
            <a href="https://github.com/casoon/AuditMySite/blob/main/docs/seo.html" target="_blank" 
               style="margin-left: 0.5rem; color: #6b7280; text-decoration: none; font-size: 0.875rem;"
               title="Search Engine Optimization metrics explained">
              <i style="font-style: normal; border: 1px solid; border-radius: 50%; padding: 2px 6px; font-size: 0.75rem;">i</i>
            </a>
          </h2>
        </div>
        <div class="section-content">
          <div class="metrics-grid">
            <div class="metric-card">
              <div class="metric-value info">${Math.round(avgScore)}/100</div>
              <div class="metric-label">Average Score</div>
            </div>
            <div class="metric-card">
              <div class="metric-value warning">${totalIssues}</div>
              <div class="metric-label">Total Issues</div>
            </div>
            <div class="metric-card">
              <div class="metric-value ${pagesWithOptimalTitle === seoPages.length ? 'success' : pagesWithOptimalTitle > seoPages.length * 0.5 ? 'warning' : 'error'}">${pagesWithOptimalTitle}/${seoPages.length}</div>
              <div class="metric-label">Optimal Titles</div>
            </div>
            <div class="metric-card">
              <div class="metric-value ${pagesWithOptimalDesc === seoPages.length ? 'success' : pagesWithOptimalDesc > seoPages.length * 0.5 ? 'warning' : 'error'}">${pagesWithOptimalDesc}/${seoPages.length}</div>
              <div class="metric-label">Optimal Descriptions</div>
            </div>
            <div class="metric-card">
              <div class="metric-value ${avgH1Count === 1 ? 'success' : avgH1Count > 1 ? 'warning' : 'error'}">${avgH1Count.toFixed(1)}</div>
              <div class="metric-label">Avg H1 Count</div>
            </div>
            <div class="metric-card">
              <div class="metric-value info">${totalImages}</div>
              <div class="metric-label">Total Images</div>
            </div>
            <div class="metric-card">
              <div class="metric-value ${totalMissingAlt === 0 ? 'success' : totalMissingAlt < totalImages * 0.1 ? 'warning' : 'error'}">${totalMissingAlt}</div>
              <div class="metric-label">Missing Alt Text</div>
            </div>
            <div class="metric-card">
              <div class="metric-value ${totalEmptyAlt === 0 ? 'success' : totalEmptyAlt < totalImages * 0.1 ? 'warning' : 'error'}">${totalEmptyAlt}</div>
              <div class="metric-label">Empty Alt Text</div>
            </div>
            <div class="metric-card">
              <div class="metric-value info">${seoPages.length}</div>
              <div class="metric-label">Pages Analyzed</div>
            </div>
          </div>
          
          <div style="margin-top: 2rem;">
            <h3 style="margin-bottom: 1rem;">Per-Page SEO Details</h3>
            <div class="scrollable-table-container" style="max-height: 720px;">
              <table class="data-table">
                <thead>
                  <tr>
                    <th style="position: sticky; left: 0; background: #f8fafc; z-index: 11;">Page</th>
                    <th>SEO Score</th>
                    <th>Title Length</th>
                    <th>Description Length</th>
                    <th>H1 Count</th>
                    <th>Images Missing Alt</th>
                  </tr>
                </thead>
                <tbody>
                  ${seoPages.map(p => {
                    const score = p.seo?.score || 0;
                    const titleLength = p.seo?.metaTags?.title?.length || p.seo?.metaTags?.titleLength || 0;
                    const descLength = p.seo?.metaTags?.description?.length || p.seo?.metaTags?.descriptionLength || 0;
                    const h1Count = p.seo?.headings?.h1?.length || p.seo?.headingStructure?.h1?.length || 0;
                    const missingAlt = p.seo?.images?.missingAlt || p.seo?.images?.withoutAlt || 0;
                    
                    const getScoreClass = (score: number) => score >= 90 ? 'value-excellent' : score >= 75 ? 'value-good' : score >= 60 ? 'value-warning' : score >= 40 ? 'value-poor' : 'value-critical';
                    const getTitleClass = (length: number) => length >= 30 && length <= 60 ? 'value-excellent' : length >= 20 && length <= 70 ? 'value-good' : 'value-warning';
                    const getDescClass = (length: number) => length >= 120 && length <= 160 ? 'value-excellent' : length >= 100 && length <= 180 ? 'value-good' : 'value-warning';
                    const getH1Class = (count: number) => count === 1 ? 'value-excellent' : count === 0 ? 'value-critical' : 'value-warning';
                    
                    return `
                      <tr>
                        <td style="max-width: 200px;">
                          <strong>${this.escape(p.title || 'Untitled')}</strong><br/>
                          <small style="color: #6b7280; font-size: 0.7rem;">${this.escape(p.url.length > 40 ? '...' + p.url.slice(-37) : p.url)}</small>
                        </td>
                        <td class="${getScoreClass(score)}">${score}/100</td>
                        <td class="${getTitleClass(titleLength)}">${titleLength} chars</td>
                        <td class="${getDescClass(descLength)}">${descLength} chars</td>
                        <td class="${getH1Class(h1Count)}">${h1Count}</td>
                        <td class="${missingAlt === 0 ? 'value-excellent' : missingAlt < 3 ? 'value-warning' : 'value-critical'}">${missingAlt}</td>
                      </tr>
                    `;
                  }).join('')}
                </tbody>
              </table>
            </div>
          </div>
          
          ${this.renderSEOAnalysis(seoPages)}
        </div>
      </section>
    `;
  }

  private renderContentWeightSection(data: EnhancedAuditResult): string {
    // Include pages with any content weight data, even if the overall test failed
    const contentWeightPages = data.pages.filter(p => 
      p.contentWeight ||
      (p.status === 'failed' && p.contentWeight)
    );
    
    if (contentWeightPages.length === 0) {
      return `
        <section id="contentweight" class="section">
          <div class="section-header">
            <h2>Content Weight Analysis</h2>
          </div>
          <div class="section-content">
            <div class="no-data">No content weight data available</div>
          </div>
        </section>
      `;
    }

    const avgScore = contentWeightPages.reduce((sum, p) => sum + (p.contentWeight.score || 0), 0) / contentWeightPages.length;
    const avgTotalSize = contentWeightPages.reduce((sum, p) => sum + (p.contentWeight.totalSize || 0), 0) / contentWeightPages.length;
    const totalOptimizations = contentWeightPages.reduce((sum, p) => sum + (p.contentWeight.optimizations?.length || 0), 0);
    
    // Calculate resource breakdown averages
    const avgResources = {
      html: contentWeightPages.reduce((sum, p) => sum + (p.contentWeight.resources?.html?.size || p.contentWeight.resources?.html || p.contentWeight.html || 0), 0) / contentWeightPages.length,
      css: contentWeightPages.reduce((sum, p) => sum + (p.contentWeight.resources?.css?.size || p.contentWeight.resources?.css || p.contentWeight.css || 0), 0) / contentWeightPages.length,
      javascript: contentWeightPages.reduce((sum, p) => sum + (p.contentWeight.resources?.javascript?.size || p.contentWeight.resources?.js?.size || p.contentWeight.resources?.javascript || p.contentWeight.resources?.js || p.contentWeight.javascript || 0), 0) / contentWeightPages.length,
      images: contentWeightPages.reduce((sum, p) => sum + (p.contentWeight.resources?.images?.size || p.contentWeight.resources?.images || p.contentWeight.images || 0), 0) / contentWeightPages.length,
      other: contentWeightPages.reduce((sum, p) => sum + (p.contentWeight.resources?.other?.size || p.contentWeight.resources?.other || p.contentWeight.other || 0), 0) / contentWeightPages.length
    };

    return `
      <section id="contentweight" class="section">
        <div class="section-header">
          <h2>Content Weight Analysis
            <a href="https://github.com/casoon/AuditMySite/blob/main/docs/content-weight.html" target="_blank" 
               style="margin-left: 0.5rem; color: #6b7280; text-decoration: none; font-size: 0.875rem;"
               title="Resource sizes and optimization metrics explained">
              <i style="font-style: normal; border: 1px solid; border-radius: 50%; padding: 2px 6px; font-size: 0.75rem;">i</i>
            </a>
          </h2>
        </div>
        <div class="section-content">
          <div class="metrics-grid">
            <div class="metric-card">
              <div class="metric-value info">${Math.round(avgScore)}/100</div>
              <div class="metric-label">Average Score</div>
            </div>
            <div class="metric-card">
              <div class="metric-value ${avgTotalSize < 500000 ? 'success' : avgTotalSize < 1000000 ? 'warning' : 'error'}">${this.formatBytes(avgTotalSize)}</div>
              <div class="metric-label">Avg Total Size</div>
            </div>
            <div class="metric-card">
              <div class="metric-value warning">${totalOptimizations}</div>
              <div class="metric-label">Total Optimizations</div>
            </div>
            <div class="metric-card">
              <div class="metric-value info">${contentWeightPages.length}</div>
              <div class="metric-label">Pages Analyzed</div>
            </div>
          </div>
          
          <div style="margin-top: 2rem;">
            <h3 style="margin-bottom: 1rem;">Resource Breakdown (Average)</h3>
            <div class="metrics-grid" style="grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 1rem;">
              <div class="metric-card">
                <div class="metric-value ${avgResources.html < 50000 ? 'success' : avgResources.html < 100000 ? 'warning' : 'error'}">${this.formatBytes(avgResources.html)}</div>
                <div class="metric-label">HTML</div>
              </div>
              <div class="metric-card">
                <div class="metric-value ${avgResources.css < 100000 ? 'success' : avgResources.css < 200000 ? 'warning' : 'error'}">${this.formatBytes(avgResources.css)}</div>
                <div class="metric-label">CSS</div>
              </div>
              <div class="metric-card">
                <div class="metric-value ${avgResources.javascript < 300000 ? 'success' : avgResources.javascript < 500000 ? 'warning' : 'error'}">${this.formatBytes(avgResources.javascript)}</div>
                <div class="metric-label">JavaScript</div>
              </div>
              <div class="metric-card">
                <div class="metric-value ${avgResources.images < 500000 ? 'success' : avgResources.images < 1000000 ? 'warning' : 'error'}">${this.formatBytes(avgResources.images)}</div>
                <div class="metric-label">Images</div>
              </div>
              <div class="metric-card">
                <div class="metric-value ${avgResources.other < 50000 ? 'success' : avgResources.other < 100000 ? 'warning' : 'error'}">${this.formatBytes(avgResources.other)}</div>
                <div class="metric-label">Other</div>
              </div>
            </div>
          </div>
          
          ${this.renderContentWeightOptimizations(contentWeightPages)}
        </div>
      </section>
    `;
  }
  
  private renderSEOAnalysis(seoPages: any[]): string {
    // Always compute synthetic issues (duplicates, lengths)
    const existingIssuesHtml = this.renderIssues(seoPages, 'seo');
    
    // Generate synthetic issues based on SEO metrics
    const seoIssues: any[] = [];
    const descriptionsForDupCheck: { url: string; content: string; length: number }[] = [];
    
    seoPages.forEach(page => {
      const seo = page.seo;
      
      // Title issues - handle both enhanced SEO data structure and legacy format
      const titleData = seo.metaTags?.title || seo.title;
      const titleContent = titleData?.content || titleData || null;
      const titlePresent = titleData?.present ?? !!titleContent;
      const titleOptimal = titleData?.optimal ?? false;
      
      if (!titlePresent || !titleContent) {
        seoIssues.push({
          severity: 'error',
          message: 'Missing page title',
          context: 'No <title> tag found',
          pageUrl: page.url,
          type: 'title'
        });
      } else {
        const titleLength = titleContent.length;
        if (!titleOptimal) {
          if (titleLength < 30) {
            seoIssues.push({
              severity: 'warning', 
              message: `Title too short (${titleLength} characters)`,
              context: titleContent,
              pageUrl: page.url,
              type: 'title'
            });
          } else if (titleLength > 60) {
            seoIssues.push({
              severity: 'warning',
              message: `Title too long (${titleLength} characters)`, 
              context: titleContent,
              pageUrl: page.url,
              type: 'title'
            });
          }
        }
        // Also surface analyzer-detected title issues (e.g., repetition "Brand - Brand")
        const titleIssuesArr = titleData?.issues || [];
        titleIssuesArr.forEach((msg: string) => {
          seoIssues.push({
            severity: 'notice',
            message: msg,
            context: titleContent,
            pageUrl: page.url,
            type: 'title'
          });
        });
      }
      
      // Description issues - handle both enhanced SEO data structure and legacy format
      const descriptionData = seo.metaTags?.description || seo.description;
      const descriptionContent = descriptionData?.content || descriptionData || null;
      const descriptionPresent = descriptionData?.present ?? !!descriptionContent;
      const descriptionOptimal = descriptionData?.optimal ?? false;
      
      if (!descriptionPresent || !descriptionContent) {
        seoIssues.push({
          severity: 'error',
          message: 'Missing meta description',
          context: 'No meta description tag found',
          pageUrl: page.url,
          type: 'description'
        });
      } else {
        const descriptionLength = descriptionContent.length;
        if (!descriptionOptimal) {
          if (descriptionLength < 120) {
            seoIssues.push({
              severity: 'warning',
              message: `Meta description too short (${descriptionLength} characters)`,
              context: descriptionContent,
              pageUrl: page.url,
              type: 'description'
            });
          } else if (descriptionLength > 160) {
            seoIssues.push({
              severity: 'warning',
              message: `Meta description too long (${descriptionLength} characters)`,
              context: descriptionContent,
              pageUrl: page.url,
              type: 'description'
            });
          }
        }
        // Collect for duplicate detection across pages
        descriptionsForDupCheck.push({ url: page.url, content: descriptionContent, length: descriptionLength });
      }
      
      // H1 issues - handle both enhanced SEO data structure and legacy format
      const headingStructure = seo.headingStructure || seo.headings;
      const h1Headings = headingStructure?.h1 || [];
      const h1Count = Array.isArray(h1Headings) ? h1Headings.length : 0;
      
      if (h1Count === 0) {
        seoIssues.push({
          severity: 'error',
          message: 'Missing H1 heading',
          context: 'No H1 heading found on page',
          pageUrl: page.url,
          type: 'heading'
        });
      } else if (h1Count > 1) {
        seoIssues.push({
          severity: 'warning',
          message: `Multiple H1 headings found (${h1Count})`,
          context: 'Page should have only one H1 heading',
          pageUrl: page.url,
          type: 'heading'
        });
      }
    });

    // Cross-page duplicate meta description detection
    if (descriptionsForDupCheck.length > 1) {
      const map: Record<string, { count: number; urls: string[]; length: number }> = {};
      descriptionsForDupCheck.forEach(d => {
        const key = (d.content || '').trim();
        if (!key) return;
        if (!map[key]) map[key] = { count: 0, urls: [], length: d.length };
        map[key].count++;
        if (map[key].urls.length < 5) map[key].urls.push(d.url);
      });
      Object.entries(map).forEach(([content, info]) => {
        if (info.count > 1) {
          const msg = `Duplicate meta description used on ${info.count} pages${info.length ? ` (length ${info.length})` : ''}`;
          seoIssues.push({
            severity: 'notice',
            message: msg,
            context: content.substring(0, 160),
            pageUrl: info.urls.join(', '),
            type: 'description-duplicate'
          });
        }
      });
    }
    
    // Group by severity 
    const issueGroups = seoIssues.reduce((groups, issue) => {
      const severity = issue.severity || 'info';
      if (!groups[severity]) groups[severity] = [];
      groups[severity].push(issue);
      return groups;
    }, {} as Record<string, any[]>);
    
    const severityOrder = ['error', 'warning', 'notice', 'info'];
    const sortedSeverities = severityOrder.filter(sev => issueGroups[sev]);
    
    const groupSections = sortedSeverities.map(severity => {
      const issues = issueGroups[severity];
      const issueItems = issues.map((issue: any) => {
        return `
          <li class="issue-item ${severity}">
            <strong>${this.escape(issue.message)}</strong>
            ${issue.context ? `<br/><small style="color: #6b7280;">Details: ${this.escape(issue.context)}</small>` : ''}
            ${issue.pageUrl ? `<br/><small style="color: #6b7280;">Page: ${this.escape(issue.pageUrl)}</small>` : ''}
          </li>
        `;
      }).join('');
      
      const severityLabel = severity.charAt(0).toUpperCase() + severity.slice(1) + 's';
      const totalCount = issues.length;
      
      return `
        <div style="margin-bottom: 2rem;">
          <h4 style="margin-bottom: 1rem; color: var(--color-text);">
            ${severityLabel} (${totalCount})
          </h4>
          <div>
            <ul class="issue-list">
              ${issueItems}
            </ul>
          </div>
        </div>
      `;
    }).join('');
    
    const syntheticBlock = groupSections
      ? `<div style="margin-top: 2rem;"><h3 style="margin-bottom: 1rem;">SEO Issues (Duplicates & Quality)</h3><div class="scrollable-issues-container">${groupSections}</div></div>`
      : '';
    
    // Combine existing analysis issues (avoid 'No issues found' when synthetic issues exist)
    const hasSynthetic = syntheticBlock !== '';
    const existingBlock = existingIssuesHtml && !/no-data/i.test(existingIssuesHtml) ? existingIssuesHtml : '';
    const combined = hasSynthetic
      ? `${existingBlock}${syntheticBlock}`
      : (existingBlock || '<div class="no-data">No issues found</div>');
    
    return combined;
  }

  private renderMobileFriendlinessSection(data: EnhancedAuditResult): string {
    // Include pages with any mobile friendliness data, even if the overall test failed
    const mobilePages = data.pages.filter(p => 
      p.mobileFriendliness ||
      (p.status === 'failed' && p.mobileFriendliness)
    );
    
    if (mobilePages.length === 0) {
      return `
        <section id="mobile" class="section">
          <div class="section-header">
            <h2>Mobile Friendliness</h2>
          </div>
          <div class="section-content">
            <div class="no-data">No mobile friendliness data available</div>
          </div>
        </section>
      `;
    }

    const avgScore = mobilePages.reduce((sum, p) => sum + (p.mobileFriendliness.overallScore || p.mobileFriendliness.score || 0), 0) / mobilePages.length;
    const totalRecommendations = mobilePages.reduce((sum, p) => sum + (p.mobileFriendliness.recommendations?.length || 0), 0);
    const totalIssues = mobilePages.reduce((sum, p) => sum + (p.mobileFriendliness.issues?.length || 0), 0);
    
    // Analyze common mobile issues
    const issuesByCategory: Record<string, { count: number; priority: string; examples: string[] }> = {};
    mobilePages.forEach(page => {
      const recommendations = page.mobileFriendliness?.recommendations || [];
      recommendations.forEach((rec: any) => {
        const category = rec.category || 'Other';
        if (!issuesByCategory[category]) {
          issuesByCategory[category] = {
            count: 0,
            priority: rec.priority || 'medium',
            examples: []
          };
        }
        issuesByCategory[category].count++;
        if (issuesByCategory[category].examples.length < 3) {
          issuesByCategory[category].examples.push(rec.issue || rec.recommendation);
        }
      });
    });
    
    const topIssueCategories = Object.entries(issuesByCategory)
      .sort(([,a], [,b]) => b.count - a.count)
      .slice(0, 4);
    
    // Calculate grade distribution
    const excellentPages = mobilePages.filter(p => (p.mobileFriendliness.overallScore || p.mobileFriendliness.score || 0) >= 90).length;
    const goodPages = mobilePages.filter(p => {
      const score = p.mobileFriendliness.overallScore || p.mobileFriendliness.score || 0;
      return score >= 75 && score < 90;
    }).length;
    const needsImprovementPages = mobilePages.filter(p => {
      const score = p.mobileFriendliness.overallScore || p.mobileFriendliness.score || 0;
      return score < 75;
    }).length;

    return `
      <section id="mobile" class="section">
        <div class="section-header">
          <h2>Mobile Friendliness
            <a href="https://github.com/casoon/AuditMySite/blob/main/docs/mobile-friendliness.html" target="_blank" 
               style="margin-left: 0.5rem; color: #6b7280; text-decoration: none; font-size: 0.875rem;"
               title="Mobile responsiveness and usability metrics explained">
              <i style="font-style: normal; border: 1px solid; border-radius: 50%; padding: 2px 6px; font-size: 0.75rem;">i</i>
            </a>
          </h2>
        </div>
        <div class="section-content">
          <div class="metrics-grid">
            <div class="metric-card">
              <div class="metric-value ${avgScore >= 90 ? 'success' : avgScore >= 75 ? 'warning' : 'error'}">${Math.round(avgScore)}/100</div>
              <div class="metric-label">Average Score</div>
            </div>
            <div class="metric-card">
              <div class="metric-value ${totalRecommendations === 0 ? 'success' : totalRecommendations < 10 ? 'warning' : 'error'}">${totalRecommendations}</div>
              <div class="metric-label">Total Recommendations</div>
            </div>
            <div class="metric-card">
              <div class="metric-value ${excellentPages === mobilePages.length ? 'success' : excellentPages > mobilePages.length * 0.5 ? 'warning' : 'error'}">${excellentPages}</div>
              <div class="metric-label">Excellent Pages</div>
            </div>
            <div class="metric-card">
              <div class="metric-value ${goodPages === 0 && excellentPages === mobilePages.length ? 'success' : 'info'}">${goodPages}</div>
              <div class="metric-label">Good Pages</div>
            </div>
            <div class="metric-card">
              <div class="metric-value ${needsImprovementPages === 0 ? 'success' : needsImprovementPages < mobilePages.length * 0.2 ? 'warning' : 'error'}">${needsImprovementPages}</div>
              <div class="metric-label">Needs Improvement</div>
            </div>
            <div class="metric-card">
              <div class="metric-value info">${Object.keys(issuesByCategory).length}</div>
              <div class="metric-label">Issue Categories</div>
            </div>
            <div class="metric-card">
              <div class="metric-value info">${mobilePages.length}</div>
              <div class="metric-label">Pages Analyzed</div>
            </div>
          </div>
          
          ${topIssueCategories.length > 0 ? `
            <div style="margin-top: 2rem;">
              <h3 style="margin-bottom: 1rem;">Most Common Mobile Issues</h3>
              <div class="metrics-grid" style="grid-template-columns: repeat(auto-fit, minmax(250px, 1fr)); gap: 1rem;">
                ${topIssueCategories.map(([category, data]) => {
                  const priorityClass = (data as any).priority === 'high' ? 'error' : (data as any).priority === 'medium' ? 'warning' : 'info';
                  return `
                    <div class="metric-card">
                      <div class="metric-value ${priorityClass}">${(data as any).count}</div>
                      <div class="metric-label">${category}</div>
                      <small style="color: #6b7280; font-size: 0.8rem; margin-top: 0.5rem; display: block;">
                        ${(data as any).examples[0] || 'Mobile optimization needed'}
                      </small>
                    </div>
                  `;
                }).join('')}
              </div>
            </div>
          ` : ''}
          
          <div style="margin-top: 2rem;">
            <h3 style="margin-bottom: 1rem;">Per-Page Mobile Friendliness Details</h3>
            <div class="scrollable-table-container" style="max-height: 720px;">
              <table class="data-table">
                <thead>
                  <tr>
                    <th style="position: sticky; left: 0; background: #f8fafc; z-index: 11;">Page</th>
                    <th>Mobile Score</th>
                    <th>Recommendations</th>
                    <th>Top Issues</th>
                  </tr>
                </thead>
                <tbody>
                  ${mobilePages.map(p => {
                    const score = p.mobileFriendliness?.overallScore || p.mobileFriendliness?.score || 0;
                    const recommendations = p.mobileFriendliness?.recommendations || [];
                    const topIssues = recommendations.slice(0, 2).map((rec: any) => rec.category || 'Mobile Issue').join(', ');
                    
                    const getScoreClass = (score: number) => score >= 90 ? 'value-excellent' : score >= 75 ? 'value-good' : score >= 60 ? 'value-warning' : score >= 40 ? 'value-poor' : 'value-critical';
                    
                    return `
                      <tr>
                        <td style="max-width: 200px;">
                          <strong>${this.escape(p.title || 'Untitled')}</strong><br/>
                          <small style="color: #6b7280; font-size: 0.7rem;">${this.escape(p.url.length > 40 ? '...' + p.url.slice(-37) : p.url)}</small>
                        </td>
                        <td class="${getScoreClass(score)}">${score}/100</td>
                        <td class="${recommendations.length === 0 ? 'value-excellent' : recommendations.length < 3 ? 'value-good' : recommendations.length < 5 ? 'value-warning' : 'value-critical'}">${recommendations.length}</td>
                        <td style="font-size: 0.8rem; color: #6b7280;">${topIssues || 'None'}</td>
                      </tr>
                    `;
                  }).join('')}
                </tbody>
              </table>
            </div>
          </div>
          
          ${this.renderIssues(mobilePages, 'mobileFriendliness')}
        </div>
      </section>
    `;
  }

  private renderPagesSection(data: EnhancedAuditResult): string {
    // Pages Overview removed - no audit value
    // Per-page details are now in their respective analysis sections
    return '';
  }

  private renderIssues(pages: any[], analysisType: string): string {
    const allIssues = pages.flatMap(p => {
      const analysis = p[analysisType];
      if (!analysis) return [];
      
      // Handle different issue structures - include recommendations for mobile friendliness
      // Properly combine all issue arrays (empty arrays should not stop the OR chain)
      const allPossibleIssues = [
        ...(analysis.issues || []),
        ...(analysis.errors || []),
        ...(analysis.warnings || []),
        ...(analysis.recommendations || [])
      ];
      
      if (allPossibleIssues.length === 0) return [];
      
      return allPossibleIssues.map(issue => {
        // Handle string issues (like SEO)
        if (typeof issue === 'string') {
          return {
            message: issue,
            severity: 'info',
            type: 'info',
            pageUrl: p.url
          };
        }
        // Handle object issues (add pageUrl for context)
        return {...issue, pageUrl: p.url};
      });
    });

    if (allIssues.length === 0) {
      return '<div class="no-data">No issues found</div>';
    }

    // Group by severity - handle recommendations format
    const issueGroups = allIssues.reduce((groups, issue) => {
      const severity = issue.severity || issue.priority || issue.type || 'info';
      if (!groups[severity]) groups[severity] = [];
      groups[severity].push(issue);
      return groups;
    }, {});

    // Sort by severity priority - include recommendation priorities
    const severityOrder = ['error', 'critical', 'high', 'warning', 'medium', 'notice', 'low', 'info'];
    const sortedSeverities = severityOrder.filter(sev => issueGroups[sev]);

    const groupSections = sortedSeverities.map(severity => {
      // Show ALL issues for all severities
      const issues = issueGroups[severity];
      const issueItems = issues.map((issue: any) => {
        // Better handling of different message field names
        let message = '';
        if (issue.message) {
          message = issue.message;
        } else if (issue.issue) {
          message = issue.issue;
        } else if (issue.recommendation) {
          message = issue.recommendation;
        } else if (issue.description) {
          message = issue.description;
        } else if (typeof issue === 'string') {
          message = issue;
        } else {
          message = 'Issue details not available';
        }
        
        // Enhanced context information
        let context = '';
        if (issue.context) {
          context = issue.context;
        } else if (issue.selector) {
          context = `Element: ${issue.selector}`;
        } else if (issue.impact && issue.issue) {
          // For mobile friendliness recommendations
          context = `Impact: ${issue.impact}`;
        } else if (issue.recommendation && issue.issue) {
          // Show both issue and recommendation for mobile friendliness
          context = `Recommendation: ${issue.recommendation}`;
        }
        
        const pageUrl = issue.pageUrl || '';
        
        return `
          <li class="issue-item ${severity}">
            <strong>${this.escape(message)}</strong>
            ${context ? `<br/><small style="color: #6b7280;">Details: ${this.escape(context)}</small>` : ''}
            ${pageUrl ? `<br/><small style="color: #6b7280;">Page: ${this.escape(pageUrl)}</small>` : ''}
          </li>
        `;
      }).join('');

      const severityLabel = severity.charAt(0).toUpperCase() + severity.slice(1) + 's';
      const totalCount = issueGroups[severity].length;
      
      // Add scrollable container for all severities with more than 5 issues
      const needsScrollContainer = totalCount > 5;
      const containerClass = needsScrollContainer ? 'scrollable-issues-container' : '';
      
      return `
        <div style="margin-bottom: 2rem;">
          <h4 style="margin-bottom: 1rem; color: var(--color-text);">
            ${severityLabel} (${totalCount})
          </h4>
          <div class="${containerClass}">
            <ul class="issue-list">
              ${issueItems}
            </ul>
          </div>
        </div>
      `;
    }).join('');

    return `
      <div style="margin-top: 2rem;">
        <h3 style="margin-bottom: 1rem;">Issues by Severity</h3>
        ${groupSections}
      </div>
    `;
  }

  private renderDetailedIssuesSection(data: EnhancedAuditResult): string {
    const allIssuesMarkdown = this.generateDetailedIssuesMarkdown(data);
    
    if (!allIssuesMarkdown || allIssuesMarkdown.trim() === '') {
      return `
        <section id="detailed-issues" class="section">
          <div class="section-header">
            <h2>Detailed Issues</h2>
          </div>
          <div class="section-content">
            <div class="no-data">No detailed issues available</div>
          </div>
        </section>
      `;
    }

    return `
      <section id="detailed-issues" class="section">
        <div class="section-header">
          <h2>Detailed Issues</h2>
        </div>
        <div class="section-content">
          <p style="margin-bottom: 1rem; color: var(--color-subtle);">
            This section contains all issues found during the audit, grouped by page. 
            You can copy this content for further analysis or AI assistance.
          </p>
          <button class="copy-button" onclick="copyDetailedIssues()">
            📋 Copy All Issues to Clipboard
          </button>
          <div class="detailed-issues-container" id="detailed-issues-content">
            <pre style="padding: 1rem; margin: 0; white-space: pre-wrap;">${this.escape(allIssuesMarkdown)}</pre>
          </div>
        </div>
      </section>
    `;
  }

  private generateAccessibilityIssuesMarkdown(data: EnhancedAuditResult): string {
    const lines: string[] = [];
    lines.push('# Accessibility Issues');
    lines.push('');
    data.pages.forEach((page: any, index) => {
      if (!page.accessibility) return;
      const errors = page.accessibility.errors || [];
      const warnings = page.accessibility.warnings || [];
      const notices = page.accessibility.notices || [];
      if (errors.length + warnings.length + notices.length === 0) return;
      lines.push(`## Page ${index + 1}: ${page.title || 'Untitled'}`);
      lines.push(`**URL:** ${page.url}`);
      lines.push('');
      if (errors.length) {
        lines.push('### Errors');
        errors.forEach((e: any, i: number) => {
          if (typeof e === 'string') lines.push(`${i + 1}. ${e}`); else this.formatIssueForMarkdown(lines, e, i + 1, 'accessibility');
        });
        lines.push('');
      }
      if (warnings.length) {
        lines.push('### Warnings');
        warnings.forEach((w: any, i: number) => {
          if (typeof w === 'string') lines.push(`${i + 1}. ${w}`); else this.formatIssueForMarkdown(lines, w, i + 1, 'accessibility');
        });
        lines.push('');
      }
      if (notices.length) {
        lines.push('### Notices');
        notices.forEach((n: any, i: number) => {
          if (typeof n === 'string') lines.push(`${i + 1}. ${n}`); else this.formatIssueForMarkdown(lines, n, i + 1, 'accessibility');
        });
        lines.push('');
      }
      lines.push('---');
      lines.push('');
    });
    const md = lines.join('\n');
    return md.trim().length ? md : '';
  }

  private generateDetailedIssuesMarkdown(data: EnhancedAuditResult): string {
    const lines: string[] = [];
    
    lines.push('# Detailed Issues Report');
    lines.push('');
    lines.push(`**Generated:** ${data.metadata.timestamp}`);
    lines.push(`**Tool Version:** ${(data.metadata as any).toolVersion || data.metadata.version}`);
    lines.push('');
    
    // Summary
    lines.push('## Summary');
    lines.push(`- **Tested Pages:** ${data.summary.testedPages}`);
    lines.push(`- **Failed Pages:** ${data.summary.failedPages}`);
    lines.push(`- **Total Errors:** ${data.summary.totalErrors}`);
    lines.push(`- **Total Warnings:** ${data.summary.totalWarnings}`);
    lines.push('');
    
    let hasAnyIssues = false;
    
    // Issues by page - include all types of issues
    data.pages.forEach((page: any, index) => {
      const pageIssues = [];
      
      // Collect accessibility issues (pa11y structured format)
      if (page.accessibility) {
        if (page.accessibility.errors && page.accessibility.errors.length > 0) {
          pageIssues.push({ type: 'Accessibility Error', issues: page.accessibility.errors });
        }
        if (page.accessibility.warnings && page.accessibility.warnings.length > 0) {
          pageIssues.push({ type: 'Accessibility Warning', issues: page.accessibility.warnings });
        }
        if (page.accessibility.notices && page.accessibility.notices.length > 0) {
          pageIssues.push({ type: 'Accessibility Notice', issues: page.accessibility.notices });
        }
      }
      
      // Collect performance issues
      if (page.performance && page.performance.issues && page.performance.issues.length > 0) {
        pageIssues.push({ type: 'Performance Issue', issues: page.performance.issues });
      }
      
      // Collect SEO issues
      if (page.seo && page.seo.issues && page.seo.issues.length > 0) {
        pageIssues.push({ type: 'SEO Issue', issues: page.seo.issues });
      }
      
      // Collect mobile friendliness recommendations
      if (page.mobileFriendliness && page.mobileFriendliness.recommendations && page.mobileFriendliness.recommendations.length > 0) {
        pageIssues.push({ type: 'Mobile Friendliness Recommendation', issues: page.mobileFriendliness.recommendations });
      }
      
      // Collect content weight optimizations
      if (page.contentWeight && page.contentWeight.optimizations && page.contentWeight.optimizations.length > 0) {
        pageIssues.push({ type: 'Content Weight Optimization', issues: page.contentWeight.optimizations });
      }
      
      // Check for general warnings/errors at page level
      const pageWarnings = [];
      if (Array.isArray(page.warnings)) {
        pageWarnings.push(...page.warnings);
      } else if (typeof page.warnings === 'string') {
        pageWarnings.push(page.warnings);
      }
      
      if (Array.isArray(page.errors)) {
        pageIssues.push({ type: 'General Error', issues: page.errors });
      }
      
      if (pageWarnings.length > 0) {
        pageIssues.push({ type: 'General Warning', issues: pageWarnings });
      }
      
      if (pageIssues.length > 0) {
        hasAnyIssues = true;
        lines.push(`## Page ${index + 1}: ${page.title || 'Untitled'}`);
        lines.push(`**URL:** ${page.url}`);
        lines.push(`**Status:** ${page.status.toUpperCase()}`);
        lines.push('');
        
        pageIssues.forEach(({ type, issues }) => {
          lines.push(`### ${this.getIssueTypeIcon(type)} ${type}`);
          
          issues.forEach((issue: any, issueIndex: number) => {
            if (typeof issue === 'string') {
              lines.push(`${issueIndex + 1}. ${issue}`);
            } else {
              this.formatIssueForMarkdown(lines, issue, issueIndex + 1, type.toLowerCase());
            }
          });
          lines.push('');
        });
        
        lines.push('---');
        lines.push('');
      }
    });
    
    if (!hasAnyIssues) {
      lines.push('## ✅ No Issues Found');
      lines.push('Congratulations! No issues were detected during this audit.');
      lines.push('');
    }
    
    return lines.join('\n');
  }
  
  private calculateWCAGLevel(score: number, errors: number, warnings: number): string {
    if (errors > 0) return 'Fail';
    if (score >= 95 && warnings === 0) return 'AAA';
    if (score >= 85 && warnings <= 2) return 'AA';
    if (score >= 70 && warnings <= 5) return 'A';
    return 'Partial';
  }
  
  private calculateARIAIssues(accessibility: any, buttonsWithoutLabel: number, imagesWithoutAlt: number): number {
    let ariaIssues = 0;
    ariaIssues += buttonsWithoutLabel;
    ariaIssues += imagesWithoutAlt;
    
    // Count ARIA-specific issues from pa11y results
    const allIssues = [
      ...(accessibility?.errors || []),
      ...(accessibility?.warnings || [])
    ];
    
    allIssues.forEach((issue: any) => {
      const message = issue.message || issue.description || '';
      const code = issue.code || '';
      if (message.toLowerCase().includes('aria') || 
          message.toLowerCase().includes('role') ||
          message.toLowerCase().includes('landmark') ||
          code.includes('aria')) {
        ariaIssues++;
      }
    });
    
    return ariaIssues;
  }
  
  private renderWCAGComplianceMetrics(accessibilityPages: any[]): string {
    const totalPages = accessibilityPages.length;
    if (totalPages === 0) return '<div class="metric-card"><div class="metric-value info">No data</div><div class="metric-label">WCAG Analysis</div></div>';
    
    // Calculate WCAG compliance distribution
    const wcagLevels = accessibilityPages.map(p => {
      const score = p.accessibility?.score || 0;
      const errors = p.accessibility?.errors?.length || 0;
      const warnings = p.accessibility?.warnings?.length || 0;
      return this.calculateWCAGLevel(score, errors, warnings);
    });
    
    const aaaPages = wcagLevels.filter(level => level === 'AAA').length;
    const aaPages = wcagLevels.filter(level => level === 'AA').length;
    const aPages = wcagLevels.filter(level => level === 'A').length;
    const partialPages = wcagLevels.filter(level => level === 'Partial').length;
    const failPages = wcagLevels.filter(level => level === 'Fail').length;
    
    return `
      <div class="metric-card">
        <div class="metric-value ${aaaPages === totalPages ? 'success' : aaaPages > totalPages * 0.7 ? 'warning' : 'error'}">${aaaPages}</div>
        <div class="metric-label">WCAG AAA</div>
      </div>
      <div class="metric-card">
        <div class="metric-value ${aaPages > totalPages * 0.7 ? 'success' : aaPages > totalPages * 0.5 ? 'warning' : 'error'}">${aaPages}</div>
        <div class="metric-label">WCAG AA</div>
      </div>
      <div class="metric-card">
        <div class="metric-value ${aPages === 0 ? 'success' : aPages < totalPages * 0.3 ? 'warning' : 'error'}">${aPages}</div>
        <div class="metric-label">WCAG A</div>
      </div>
      <div class="metric-card">
        <div class="metric-value ${failPages === 0 ? 'success' : 'error'}">${failPages}</div>
        <div class="metric-label">Failed</div>
      </div>
    `;
  }
  
  private renderARIAAnalysisMetrics(accessibilityPages: any[]): string {
    const totalPages = accessibilityPages.length;
    if (totalPages === 0) return '<div class="metric-card"><div class="metric-value info">No data</div><div class="metric-label">ARIA Analysis</div></div>';
    
    // Aggregate ARIA metrics
    let totalAriaIssues = 0;
    let totalLandmarks = 0;
    let totalMislabeledElements = 0;
    let totalFocusIssues = 0;
    
    accessibilityPages.forEach(p => {
      const accessibility = p.accessibility;
      const buttonsWithoutLabel = this.extractMetricFromIssues(accessibility, 'button') || 0;
      const imagesWithoutAlt = this.extractMetricFromIssues(accessibility, 'alt') || 0;
      
      totalAriaIssues += this.calculateARIAIssues(accessibility, buttonsWithoutLabel, imagesWithoutAlt);
      totalMislabeledElements += buttonsWithoutLabel + imagesWithoutAlt;
      
      // Detect landmark and focus issues from pa11y results
      const allIssues = [...(accessibility?.errors || []), ...(accessibility?.warnings || [])];
      allIssues.forEach((issue: any) => {
        const message = (issue.message || '').toLowerCase();
        if (message.includes('landmark') || message.includes('main') || message.includes('navigation')) {
          totalLandmarks++;
        }
        if (message.includes('focus') || message.includes('tabindex') || message.includes('keyboard')) {
          totalFocusIssues++;
        }
      });
    });
    
    return `
      <div class="metric-card">
        <div class="metric-value ${totalAriaIssues === 0 ? 'success' : totalAriaIssues < 5 ? 'warning' : 'error'}">${totalAriaIssues}</div>
        <div class="metric-label">ARIA Issues</div>
      </div>
      <div class="metric-card">
        <div class="metric-value ${totalMislabeledElements === 0 ? 'success' : totalMislabeledElements < 3 ? 'warning' : 'error'}">${totalMislabeledElements}</div>
        <div class="metric-label">Mislabeled Elements</div>
      </div>
      <div class="metric-card">
        <div class="metric-value ${totalLandmarks === 0 ? 'success' : totalLandmarks < 2 ? 'warning' : 'error'}">${totalLandmarks}</div>
        <div class="metric-label">Landmark Issues</div>
      </div>
      <div class="metric-card">
        <div class="metric-value ${totalFocusIssues === 0 ? 'success' : totalFocusIssues < 3 ? 'warning' : 'error'}">${totalFocusIssues}</div>
        <div class="metric-label">Focus Issues</div>
      </div>
    `;
  }
  
  private renderAccessibilityDetailedIssues(pages: any[]): string {
    // Include all non-skipped pages that have accessibility data (passed or failed)
    const accessibilityPages = pages.filter(p => p.accessibility && p.status !== 'skipped');
    
    if (accessibilityPages.length === 0) {
      return '<div class="no-data">No accessibility data available for detailed analysis</div>';
    }
    
    // Build Markdown-formatted output for copy/paste
    let markdownContent = '';
    let hasAnyIssues = false;
    
    accessibilityPages.forEach((page, index) => {
      const issues: string[] = [];
      
      // Collect accessibility errors from multiple sources
      // Map pa11yIssues from AccessibilityResult to accessibility.issues format
      const pa11yErrorIssues = ((page as any).pa11yIssues || []).filter((issue: any) => issue.type === 'error');
      const accessibilityErrorIssues = (page.accessibility.issues || []).filter((issue: any) => issue.type === 'error');
      
      const allErrors = [
        ...(page.accessibility.errors || []),
        ...pa11yErrorIssues,
        ...accessibilityErrorIssues
      ];
      
      if (allErrors.length > 0) {
        issues.push(`**Accessibility Errors (${allErrors.length}):**`);
        allErrors.forEach((issue: any, i: number) => {
          const message = issue.message || issue.description || String(issue);
          issues.push(`${i + 1}. ${message}`);
          if (issue.selector) issues.push(`   - Element: \`${issue.selector}\``);
          if (issue.context) {
            issues.push(`   - HTML Context: \`${issue.context.substring(0, 100)}...\``);
            // Extract line number from HTML context
            const lineInfo = this.extractLineNumber(issue.context, issue.selector);
            if (lineInfo.line > 0) {
              issues.push(`   - **Line ${lineInfo.line}** in HTML source`);
            }
          }
          if (issue.code) issues.push(`   - Code: \`${issue.code}\``);
          if (issue.impact) issues.push(`   - Impact: ${issue.impact}`);
          if (issue.help || issue.helpUrl) {
            const helpText = issue.help || 'More information';
            const helpLink = issue.helpUrl ? `[${helpText}](${issue.helpUrl})` : helpText;
            issues.push(`   - Help: ${helpLink}`);
          }
        });
        issues.push('');
      }
      
      // Collect accessibility warnings from multiple sources
      // Map pa11yIssues from AccessibilityResult to accessibility.issues format
      const pa11yWarningIssues = ((page as any).pa11yIssues || []).filter((issue: any) => issue.type === 'warning');
      const accessibilityWarningIssues = (page.accessibility.issues || []).filter((issue: any) => issue.type === 'warning');
      
      const allWarnings = [
        ...(page.accessibility.warnings || []),
        ...pa11yWarningIssues,
        ...accessibilityWarningIssues
      ];
      
      if (allWarnings.length > 0) {
        issues.push(`**Accessibility Warnings (${allWarnings.length}):**`);
        allWarnings.forEach((issue: any, i: number) => {
          const message = issue.message || issue.description || String(issue);
          issues.push(`${i + 1}. ${message}`);
          if (issue.selector) issues.push(`   - Element: \`${issue.selector}\``);
          if (issue.context) {
            issues.push(`   - HTML Context: \`${issue.context.substring(0, 100)}...\``);
            // Extract line number from HTML context
            const lineInfo = this.extractLineNumber(issue.context, issue.selector);
            if (lineInfo.line > 0) {
              issues.push(`   - **Line ${lineInfo.line}** in HTML source`);
            }
          }
          if (issue.code) issues.push(`   - Code: \`${issue.code}\``);
          if (issue.impact) issues.push(`   - Impact: ${issue.impact}`);
          if (issue.help || issue.helpUrl) {
            const helpText = issue.help || 'More information';
            const helpLink = issue.helpUrl ? `[${helpText}](${issue.helpUrl})` : helpText;
            issues.push(`   - Help: ${helpLink}`);
          }
        });
        issues.push('');
      }
      
      // Collect accessibility notices from multiple sources
      // Map pa11yIssues from AccessibilityResult to accessibility.issues format
      const pa11yNoticeIssues = ((page as any).pa11yIssues || []).filter((issue: any) => issue.type === 'notice');
      const accessibilityNoticeIssues = (page.accessibility.issues || []).filter((issue: any) => issue.type === 'notice');
      
      const allNotices = [
        ...(page.accessibility.notices || []),
        ...pa11yNoticeIssues,
        ...accessibilityNoticeIssues
      ];
      
      if (allNotices.length > 0) {
        issues.push(`**Accessibility Notices (${allNotices.length}):**`);
        allNotices.forEach((issue: any, i: number) => {
          const message = issue.message || issue.description || String(issue);
          issues.push(`${i + 1}. ${message}`);
          if (issue.selector) issues.push(`   - Element: \`${issue.selector}\``);
          if (issue.context) {
            issues.push(`   - HTML Context: \`${issue.context.substring(0, 100)}...\``);
            // Extract line number from HTML context
            const lineInfo = this.extractLineNumber(issue.context, issue.selector);
            if (lineInfo.line > 0) {
              issues.push(`   - **Line ${lineInfo.line}** in HTML source`);
            }
          }
          if (issue.code) issues.push(`   - Code: \`${issue.code}\``);
          if (issue.impact) issues.push(`   - Impact: ${issue.impact}`);
        });
        issues.push('');
      }
      
      // Always collect basic checks if they exist and have issues
      if (page.accessibility.basicChecks) {
        const basic = page.accessibility.basicChecks;
        if (basic.imagesWithoutAlt > 0 || basic.buttonsWithoutLabel > 0 || basic.contrastIssues > 0) {
          issues.push(`**Basic Accessibility Issues:**`);
          if (basic.imagesWithoutAlt > 0) issues.push(`- ${basic.imagesWithoutAlt} images missing alt text`);
          if (basic.buttonsWithoutLabel > 0) issues.push(`- ${basic.buttonsWithoutLabel} buttons without labels`);
          if (basic.contrastIssues > 0) issues.push(`- ${basic.contrastIssues} potential contrast issues`);
          issues.push('');
        }
      }
      
      if (issues.length > 0) {
        hasAnyIssues = true;
        markdownContent += `\n## Page ${index + 1}: ${page.title || 'Untitled'}\n\n`;
        markdownContent += `**URL:** ${page.url}\n`;
        markdownContent += `**Status:** ${page.status.toUpperCase()}\n`;
        markdownContent += `**Accessibility Score:** ${page.accessibility.score || 0}/100\n\n`;
        markdownContent += issues.join('\n') + '\n';
      }
    });
    
    if (!hasAnyIssues) {
      markdownContent = `## ✅ Excellent Accessibility!\n\nNo accessibility issues were detected in the analyzed pages.\nThis indicates strong WCAG compliance and inclusive design.`;
    }
    
    // Return as pre-formatted text block for easy copy/paste
    return `
      <div style="background: #f8fafc; border: 1px solid #e2e8f0; border-radius: 8px; padding: 1.5rem; margin-top: 1rem;">
        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem;">
          <h4 style="margin: 0; color: #1f2937;">📋 Detailed Issues (Markdown Format)</h4>
          <button onclick="copyToClipboard('detailed-issues-content')" 
                  style="background: #3b82f6; color: white; border: none; padding: 0.5rem 1rem; border-radius: 6px; cursor: pointer; font-size: 0.9rem;"
                  title="Copy to clipboard">
            📋 Copy
          </button>
        </div>
        <pre id="detailed-issues-content" style="background: white; border: 1px solid #d1d5db; border-radius: 6px; padding: 1rem; overflow: auto; white-space: pre-wrap; font-size: 0.9rem; line-height: 1.5; margin: 0; max-height: 720px;">${this.escape(markdownContent)}</pre>
        <script>
          function copyToClipboard(elementId) {
            const element = document.getElementById(elementId);
            const text = element.textContent || element.innerText;
            navigator.clipboard.writeText(text).then(function() {
              // Simple feedback
              const btn = event.target;
              const original = btn.textContent;
              btn.textContent = '✅ Copied!';
              setTimeout(() => btn.textContent = original, 2000);
            }).catch(function(err) {
              console.error('Could not copy text: ', err);
            });
          }
        </script>
      </div>
    `;
  }
  
  private getIssueTypeIcon(type: string): string {
    const icons: Record<string, string> = {
      'Accessibility Error': '❌',
      'Accessibility Warning': '⚠️',
      'Accessibility Notice': 'ℹ️',
      'Performance Issue': '⚡',
      'SEO Issue': '🔍',
      'Mobile Friendliness Recommendation': '📱',
      'Content Weight Optimization': '📦',
      'General Error': '🚫',
      'General Warning': '⚠️'
    };
    return icons[type] || '•';
  }
  
  /**
   * Extract line number information from HTML context and selector
   * This helps developers find the exact location of accessibility issues in their HTML source
   */
  private extractLineNumber(context: string, selector: string): { line: number; column: number } {
    if (!context || typeof context !== 'string') {
      return { line: 0, column: 0 };
    }
    
    try {
      // Try to find line number patterns in context
      // Pa11y often includes HTML context with the problematic element
      
      // Method 1: Look for HTML structure patterns to estimate line position
      const htmlLines = context.split('\n');
      if (htmlLines.length > 1) {
        // If context contains multiple lines, estimate from structure
        const targetLineIndex = htmlLines.findIndex(line => 
          selector && line.includes(selector.replace(/[\[\]]/g, ''))
        );
        
        if (targetLineIndex > -1) {
          return { line: targetLineIndex + 1, column: 0 };
        }
      }
      
      // Method 2: Look for specific element patterns
      if (selector) {
        const cleanSelector = selector.replace(/^.*>\s*/, ''); // Remove parent selectors
        const elementMatch = cleanSelector.match(/^([a-zA-Z]+)/); // Extract element type
        
        if (elementMatch) {
          const elementType = elementMatch[1];
          const elementRegex = new RegExp(`<${elementType}[^>]*>`, 'gi');
          const contextLines = context.split('\n');
          
          for (let i = 0; i < contextLines.length; i++) {
            if (elementRegex.test(contextLines[i])) {
              return { line: i + 1, column: contextLines[i].search(elementRegex) };
            }
          }
        }
      }
      
      // Method 3: Try to extract from error context patterns
      // Some accessibility tools include line:column information
      const linePattern = /line[:\s]*(\d+)/i;
      const columnPattern = /col(?:umn)?[:\s]*(\d+)/i;
      
      const lineMatch = context.match(linePattern);
      const columnMatch = context.match(columnPattern);
      
      if (lineMatch) {
        return {
          line: parseInt(lineMatch[1], 10),
          column: columnMatch ? parseInt(columnMatch[1], 10) : 0
        };
      }
      
      // Method 4: Estimate based on context position in document
      // Count opening tags to estimate approximate line position
      const tagCount = (context.match(/<[^>]+>/g) || []).length;
      if (tagCount > 0) {
        // Rough estimation: assume ~3-5 lines per tag on average
        const estimatedLine = Math.max(1, Math.floor(tagCount * 3.5));
        return { line: estimatedLine, column: 0 };
      }
      
    } catch (error) {
      console.warn('Error extracting line number from accessibility issue context:', error);
    }
    
    return { line: 0, column: 0 };
  }
  
  private extractMetricFromIssues(accessibility: any, metricType: string): number {
    if (!accessibility) return 0;
    
    // Use basicChecks data if available (preferred - more accurate)
    if (accessibility.basicChecks) {
      switch (metricType) {
        case 'alt':
          return accessibility.basicChecks.imagesWithoutAlt || 0;
        case 'button':
          return accessibility.basicChecks.buttonsWithoutLabel || 0;
        case 'contrast':
          return accessibility.basicChecks.contrastIssues || 0;
        default:
          break;
      }
    }
    
    // Fallback: parse from issues (legacy support)
    const allIssues = [
      ...(accessibility.errors || []),
      ...(accessibility.warnings || []),
      ...(accessibility.notices || [])
    ];
    
    let count = 0;
    
    allIssues.forEach((issue: any) => {
      const message = issue.message || issue.description || '';
      const code = issue.code || '';
      
      switch (metricType) {
        case 'alt':
          if (message.toLowerCase().includes('alt') || 
              message.toLowerCase().includes('alternative text') ||
              code.includes('image-alt')) {
            count++;
          }
          break;
        case 'button':
          if (message.toLowerCase().includes('button') || 
              message.toLowerCase().includes('label') ||
              message.toLowerCase().includes('aria-label') ||
              code.includes('button-name') || 
              code.includes('link-name')) {
            count++;
          }
          break;
        case 'contrast':
          if (message.toLowerCase().includes('contrast') || 
              message.toLowerCase().includes('color') ||
              code.includes('color-contrast')) {
            count++;
          }
          break;
      }
    });
    
    return count;
  }
  
  private formatIssueForMarkdown(lines: string[], issue: any, index: number, type: string): void {
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

  private renderFooter(data: EnhancedAuditResult): string {
    const durationSec = (() => {
      const d = Number(data.metadata.duration);
      if (Number.isFinite(d) && d > 0) return Math.round(d / 1000);
      const perfSec = (data as any).systemPerformance?.testCompletionTimeSeconds;
      if (Number.isFinite(perfSec) && perfSec > 0) return Math.round(perfSec);
      return 0;
    })();
    const timestamp = new Date(data.metadata.timestamp).toLocaleString();
    
    return `
      <footer class="footer">
        <div>
          Generated by <strong>AuditMySite v${data.metadata.toolVersion || data.metadata.version}</strong> •
          Analysis Duration: ${durationSec}s • 
          Generated: ${timestamp}
        </div>
      </footer>
    `;
  }

  private generateJavaScript(): string {
    return `
      // Navigation highlighting
      const sections = document.querySelectorAll('section[id]');
      const navLinks = document.querySelectorAll('.nav-link');
      
      function updateActiveNav() {
        let current = '';
        sections.forEach(section => {
          const sectionTop = section.offsetTop - 140;
          const sectionHeight = section.offsetHeight;
          if (window.pageYOffset >= sectionTop && window.pageYOffset < sectionTop + sectionHeight) {
            current = section.getAttribute('id');
          }
        });
        
        navLinks.forEach(link => {
          link.classList.remove('active');
          if (link.getAttribute('href') === '#' + current) {
            link.classList.add('active');
          }
        });
      }
      
      window.addEventListener('scroll', updateActiveNav);
      updateActiveNav();
      
      // Smooth scrolling for nav links
      navLinks.forEach(link => {
        link.addEventListener('click', (e) => {
          e.preventDefault();
          const targetId = link.getAttribute('href').substring(1);
          const targetSection = document.getElementById(targetId);
          if (targetSection) {
            window.scrollTo({
              top: targetSection.offsetTop - 120,
              behavior: 'smooth'
            });
          }
        });
      });
      
      // Copy detailed issues functionality
      function copyDetailedIssues() {
        const issuesContent = document.querySelector('#detailed-issues-content pre');
        const button = document.querySelector('.copy-button');
        
        if (issuesContent && button) {
          const textContent = issuesContent.textContent || issuesContent.innerText || '';
          
          if (navigator.clipboard) {
            navigator.clipboard.writeText(textContent).then(() => {
              button.textContent = '✓ Copied!';
              button.classList.add('copied');
              setTimeout(() => {
                button.textContent = '📋 Copy All Issues to Clipboard';
                button.classList.remove('copied');
              }, 2000);
            }).catch(err => {
              console.error('Failed to copy: ', err);
              fallbackCopyTextToClipboard(textContent, button);
            });
          } else {
            fallbackCopyTextToClipboard(textContent, button);
          }
        }
      }
      
      function fallbackCopyTextToClipboard(text, button) {
        const textArea = document.createElement('textarea');
        textArea.value = text;
        textArea.style.top = '0';
        textArea.style.left = '0';
        textArea.style.position = 'fixed';
        
        document.body.appendChild(textArea);
        textArea.focus();
        textArea.select();
        
        try {
          document.execCommand('copy');
          button.textContent = '✓ Copied!';
          button.classList.add('copied');
          setTimeout(() => {
            button.textContent = '📋 Copy All Issues to Clipboard';
            button.classList.remove('copied');
          }, 2000);
        } catch (err) {
          console.error('Fallback: Oops, unable to copy', err);
        }
        
        document.body.removeChild(textArea);
      }
      
      // Make function globally available
      window.copyDetailedIssues = copyDetailedIssues;
    `;
  }

  private renderScoreBreakdown(data: EnhancedAuditResult): string {
    if (data.pages.length === 0) return '<div class="no-data">No data available for breakdown</div>';
    
    // Calculate component scores (aligned with calculateOverallScore)
    let totalWeightedScore = 0;
    let totalWeight = 0;
    const componentScores: { [key: string]: { score: number; weight: number; count: number } } = {
      accessibility: { score: 0, weight: 0.35, count: 0 },
      performance: { score: 0, weight: 0.25, count: 0 },
      seo: { score: 0, weight: 0.2, count: 0 },
      contentWeight: { score: 0, weight: 0.1, count: 0 },
      mobileFriendliness: { score: 0, weight: 0.1, count: 0 }
    };
    
    data.pages.forEach(page => {
      // Presence detection (treat as analyzed even if score is 0)
      const hasA11y = (page as any).pa11yScore !== undefined || (page as any).accessibility !== undefined;
      const hasPerf = (page as any).enhancedPerformance !== undefined || (page as any).performance !== undefined;
      const hasSEO = (page as any).enhancedSEO !== undefined || (page as any).seo !== undefined;
      const hasCW = (page as any).contentWeight !== undefined;
      const hasMobile = (page as any).mobileFriendliness !== undefined;

      // Accessibility
      const a11yScore = (page as any).pa11yScore ?? (page as any).accessibility?.score ?? 0;
      if (hasA11y) {
        componentScores.accessibility.score += (a11yScore || 0);
        componentScores.accessibility.count++;
        totalWeightedScore += (a11yScore || 0) * componentScores.accessibility.weight;
        totalWeight += componentScores.accessibility.weight;
      }
      
      // Performance
      const perfScore = (page as any).enhancedPerformance?.performanceScore ?? (page as any).performance?.score ?? 0;
      if (hasPerf) {
        componentScores.performance.score += (perfScore || 0);
        componentScores.performance.count++;
        totalWeightedScore += (perfScore || 0) * componentScores.performance.weight;
        totalWeight += componentScores.performance.weight;
      }
      
      // SEO
      const seoScore = (page as any).enhancedSEO?.seoScore ?? (page as any).seo?.score ?? 0;
      if (hasSEO) {
        componentScores.seo.score += (seoScore || 0);
        componentScores.seo.count++;
        totalWeightedScore += (seoScore || 0) * componentScores.seo.weight;
        totalWeight += componentScores.seo.weight;
      }
      
      // Content Weight
      const contentScore = (page as any).contentWeight?.contentScore ?? (page as any).contentWeight?.score ?? 0;
      if (hasCW) {
        componentScores.contentWeight.score += (contentScore || 0);
        componentScores.contentWeight.count++;
        totalWeightedScore += (contentScore || 0) * componentScores.contentWeight.weight;
        totalWeight += componentScores.contentWeight.weight;
      }
      
      // Mobile Friendliness
      const mobileScore = (page as any).mobileFriendliness?.overallScore ?? (page as any).mobileFriendliness?.score ?? 0;
      if (hasMobile) {
        componentScores.mobileFriendliness.score += (mobileScore || 0);
        componentScores.mobileFriendliness.count++;
        totalWeightedScore += (mobileScore || 0) * componentScores.mobileFriendliness.weight;
        totalWeight += componentScores.mobileFriendliness.weight;
      }
    });
    
    // Calculate averages
    Object.keys(componentScores).forEach(key => {
      const component = componentScores[key];
      if (component.count > 0) {
        component.score = Math.round(component.score / component.count);
      }
    });
    
    const overallScore = totalWeight > 0 ? Math.round(totalWeightedScore / totalWeight) : 0;
    
    const componentLabels = {
      accessibility: 'Accessibility',
      performance: 'Performance', 
      seo: 'SEO',
      contentWeight: 'Content Weight',
      mobileFriendliness: 'Mobile Friendliness'
    };
    
    const componentDescriptions = {
      accessibility: 'WCAG compliance, screen reader support, keyboard navigation',
      performance: 'Core Web Vitals, loading speed, user experience metrics',
      seo: 'Meta tags, heading structure, content optimization', 
      contentWeight: 'Resource sizes, compression, optimization opportunities',
      mobileFriendliness: 'Responsive design, touch targets, mobile usability'
    };
    
    return `
      <div class="metrics-grid" style="grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); gap: 1rem;">
        ${Object.entries(componentScores).map(([key, component]) => {
          const isIncluded = component.count > 0;
          const score = component.score;
          const weight = Math.round(component.weight * 100);
          const contribution = isIncluded ? Math.round(score * component.weight) : 0;
          const label = componentLabels[key as keyof typeof componentLabels];
          const description = componentDescriptions[key as keyof typeof componentDescriptions];
          
          const getScoreClass = (score: number) => score >= 90 ? 'success' : score >= 75 ? 'warning' : score >= 50 ? 'info' : 'error';
          
          return `
            <div class="metric-card" style="${isIncluded ? 'border-left: 4px solid var(--primary);' : 'border-left: 4px solid #e5e7eb; opacity: 0.6;'}">
              <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 0.5rem;">
                <div class="metric-label" style="font-weight: 600; margin-bottom: 0;">${label}</div>
                <div style="font-size: 0.8rem; color: #6b7280; font-weight: 500;">${weight}% weight</div>
              </div>
              <div class="metric-value ${isIncluded ? getScoreClass(score) : 'info'}" style="font-size: 2rem; margin-bottom: 0.25rem;">
                ${isIncluded ? score + '/100' : 'N/A'}
              </div>
              <div style="font-size: 0.75rem; color: #6b7280; margin-bottom: 0.5rem;">
                ${description}
              </div>
              <div style="font-size: 0.8rem; color: var(--primary); font-weight: 600;">
                ${isIncluded ? `Contributes ${contribution} points` : 'Not analyzed'}
              </div>
            </div>
          `;
        }).join('')}
      </div>
      
      <div style="margin-top: 1.5rem; padding: 1rem; background: #f8fafc; border-radius: 8px; border-left: 4px solid var(--primary);">
        <div style="font-weight: 600; margin-bottom: 0.5rem; color: var(--color-text);">Calculation Formula:</div>
        <div style="color: #6b7280; font-size: 0.9rem; line-height: 1.5;">
          <strong>Overall Score = </strong>
          ${Object.entries(componentScores).map(([key, component]) => {
            const isIncluded = component.count > 0;
            const weight = Math.round(component.weight * 100);
            const label = componentLabels[key as keyof typeof componentLabels];
            return `${label} (${isIncluded ? component.score : 0} × ${weight}%)`;
          }).join(' + ')}
          <br><br>
          <strong>Result:</strong> ${overallScore}/100 (weighted average)
        </div>
      </div>
    `;
  }

  private calculateOverallScore(data: EnhancedAuditResult): number {
    if (data.pages.length === 0) return 0;
    
    let totalWeightedScore = 0;
    let totalWeight = 0;
    
    data.pages.forEach(page => {
      // Accessibility (35% weight)
      const hasA11y = (page as any).pa11yScore !== undefined || (page as any).accessibility !== undefined;
      const a11yScore = (page as any).pa11yScore ?? (page.accessibility?.score ?? 0);
      if (hasA11y) {
        totalWeightedScore += (a11yScore || 0) * 0.35;
        totalWeight += 0.35;
      }
      
      // Performance (25% weight)
      const hasPerf = (page as any).performance !== undefined || (page as any).enhancedPerformance !== undefined;
      const perfScore = (page as any).enhancedPerformance?.performanceScore ?? (page.performance?.score ?? 0);
      if (hasPerf) {
        totalWeightedScore += (perfScore || 0) * 0.25;
        totalWeight += 0.25;
      }
      
      // SEO (20% weight)
      const hasSEO = (page as any).seo !== undefined || (page as any).enhancedSEO !== undefined;
      const seoScore = (page as any).enhancedSEO?.seoScore ?? (page.seo?.score ?? 0);
      if (hasSEO) {
        totalWeightedScore += (seoScore || 0) * 0.2;
        totalWeight += 0.2;
      }
      
      // Content Weight (10% weight)
      const hasCW = (page as any).contentWeight !== undefined;
      const contentScore = (page as any).contentWeight?.contentScore ?? (page.contentWeight?.score ?? 0);
      if (hasCW) {
        totalWeightedScore += (contentScore || 0) * 0.1;
        totalWeight += 0.1;
      }
      
      // Mobile Friendliness (10% weight)
      const hasMobile = (page as any).mobileFriendliness !== undefined;
      const mobileScore = (page as any).mobileFriendliness?.overallScore ?? (page.mobileFriendliness?.score ?? 0);
      if (hasMobile) {
        totalWeightedScore += (mobileScore || 0) * 0.1;
        totalWeight += 0.1;
      }
    });
    
    return totalWeight > 0 ? Math.round(totalWeightedScore / totalWeight) : 0;
  }

  private extractDomain(url: string): string {
    try {
      return new URL(url).hostname;
    } catch {
      return url;
    }
  }

  private escape(str: string): string {
    return str
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;')
      .replace(/'/g, '&#039;');
  }

  private formatBytes(bytes: number): string {
    // Robust guard against invalid values that would produce "NaN undefined"
    if (typeof bytes !== 'number' || !isFinite(bytes) || bytes <= 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    let i = Math.floor(Math.log(bytes) / Math.log(k));
    if (!isFinite(i) || i < 0) i = 0;
    if (i >= sizes.length) i = sizes.length - 1;
    const value = Math.round((bytes / Math.pow(k, i)) * 100) / 100;
    if (!isFinite(value)) return '0 B';
    return value + ' ' + sizes[i];
  }

  private renderContentWeightOptimizations(pages: any[]): string {
    const allOptimizations = pages.flatMap(p => p.contentWeight?.recommendations || p.contentWeight?.optimizations || []);
    
    if (allOptimizations.length === 0) {
      // Create detailed analysis even without optimization suggestions
      return this.renderContentWeightDetailedAnalysis(pages);
    }

    // Group by optimization type and calculate total savings
    const optimizationGroups = allOptimizations.reduce((groups, opt) => {
      const type = opt.id || opt.type || 'other';
      if (!groups[type]) {
        groups[type] = {
          type,
          count: 0,
          totalSavings: 0,
          priority: opt.priority || 'medium',
          messages: [],
          affectedPages: []
        };
      }
      groups[type].count++;
      groups[type].totalSavings += opt.savings || opt.scoreImprovement || 0;
      const message = opt.message || opt.recommendation || opt.issue || 'Optimization available';
      if (message && !groups[type].messages.includes(message)) {
        groups[type].messages.push(message);
      }
      return groups;
    }, {});

    // Sort by priority first, then by total savings
    const sortedOptimizations = Object.values(optimizationGroups)
      .sort((a: any, b: any) => {
        const priorityWeights: Record<string, number> = { 'high': 3, 'medium': 2, 'low': 1 };
        const aPriority = priorityWeights[a.priority] || 1;
        const bPriority = priorityWeights[b.priority] || 1;
        if (aPriority !== bPriority) return bPriority - aPriority;
        return b.totalSavings - a.totalSavings;
      });

    const optimizationRows = sortedOptimizations.map((opt: any) => {
      const priorityClass = opt.priority === 'high' ? 'value-critical' : opt.priority === 'medium' ? 'value-warning' : 'value-good';
      const priorityBadge = opt.priority === 'high' ? '🔴 High' : opt.priority === 'medium' ? '🟡 Medium' : '🟢 Low';
      const impactEstimate = opt.totalSavings > 50 ? 'High Impact' : opt.totalSavings > 20 ? 'Medium Impact' : 'Low Impact';
      
      return `
        <tr>
          <td><strong>${this.formatOptimizationType(opt.type)}</strong></td>
          <td><span class="${priorityClass}">${priorityBadge}</span></td>
          <td>${opt.count}</td>
          <td>${opt.totalSavings > 0 ? opt.totalSavings + ' points' : 'Variable'}</td>
          <td style="font-size: 0.8rem; color: #6b7280;">${impactEstimate}</td>
          <td style="font-size: 0.8rem;">${opt.messages[0] || 'Optimization available'}</td>
        </tr>
      `;
    }).join('');

    // Create per-page optimization table
    const perPageOptimizations = this.renderPerPageOptimizations(pages);

    return `
      <div style="margin-top: 2rem;">
        <h3 style="margin-bottom: 1rem;">🔧 Optimization Recommendations</h3>
        
        <!-- Summary Table -->
        <div style="margin-bottom: 2rem;">
          <h4 style="margin-bottom: 0.5rem; color: #374151;">Summary by Optimization Type</h4>
          <div class="scrollable-table-container" style="max-height: 720px;">
            <table class="data-table">
              <thead>
                <tr>
                  <th>Optimization</th>
                  <th>Priority</th>
                  <th>Affected Pages</th>
                  <th>Potential Savings</th>
                  <th>Impact</th>
                  <th>Description</th>
                </tr>
              </thead>
              <tbody>
                ${optimizationRows}
              </tbody>
            </table>
          </div>
        </div>
        
        ${perPageOptimizations}
      </div>
    `;
  }
  
  private renderPerPageOptimizations(pages: any[]): string {
    const pagesWithOptimizations = pages.filter(p => {
      const opts = p.contentWeight?.recommendations || p.contentWeight?.optimizations || [];
      return opts.length > 0;
    });
    
    if (pagesWithOptimizations.length === 0) {
      return '<div style="margin-top: 1rem; color: #6b7280; font-style: italic;">No specific page optimizations available.</div>';
    }
    
    const pageRows = pagesWithOptimizations.map(page => {
      const optimizations = page.contentWeight?.recommendations || page.contentWeight?.optimizations || [];
      
      const totalSize = page.contentWeight?.totalSize || page.contentWeight?.total || 0;
      const jsSize = page.contentWeight?.resources?.javascript?.size || page.contentWeight?.resources?.js?.size || page.contentWeight?.javascript || 0;
      const cssSize = page.contentWeight?.resources?.css?.size || page.contentWeight?.css || 0;
      const imageSize = page.contentWeight?.resources?.images?.size || page.contentWeight?.images || 0;
      const contentScore = page.contentWeight?.score || page.contentWeight?.contentScore || 0;
      
      const getSizeClass = (size: number): string => {
        if (size <= 500000) return 'size-small';
        if (size <= 1000000) return 'size-medium';
        if (size <= 2000000) return 'size-large';
        return 'size-huge';
      };
      
      const getScoreClass = (score: number): string => {
        if (score >= 90) return 'value-excellent';
        if (score >= 75) return 'value-good';
        if (score >= 60) return 'value-warning';
        if (score >= 40) return 'value-poor';
        return 'value-critical';
      };
      
      const topOptimizations = optimizations.slice(0, 2).map((opt: any) => 
        `<span class="issue-item ${opt.priority === 'high' ? 'error' : opt.priority === 'medium' ? 'warning' : 'info'}" style="padding: 0.25rem 0.5rem; margin: 0.125rem; display: inline-block; border-radius: 4px; font-size: 0.7rem;">${this.formatOptimizationType(opt.id || opt.type || 'optimization')}</span>`
      ).join('');
      
      return `
        <tr>
          <td style="max-width: 200px;">
            <strong>${this.escape(page.title || 'Untitled')}</strong><br/>
            <small style="color: #6b7280; font-size: 0.7rem;">${this.escape(page.url.length > 40 ? '...' + page.url.slice(-37) : page.url)}</small>
          </td>
          <td class="${getScoreClass(contentScore)}">${contentScore}/100</td>
          <td class="${getSizeClass(totalSize)}">${this.formatBytes(totalSize)}</td>
          <td class="${getSizeClass(jsSize)}">${this.formatBytes(jsSize)}</td>
          <td class="${getSizeClass(cssSize)}">${this.formatBytes(cssSize)}</td>
          <td class="${getSizeClass(imageSize)}">${this.formatBytes(imageSize)}</td>
          <td style="font-size: 0.7rem;">${topOptimizations}</td>
        </tr>
      `;
    }).join('');
    
    return `
      <div style="margin-top: 1.5rem;">
        <h4 style="margin-bottom: 0.5rem; color: #374151;">Per-Page Optimization Details</h4>
        <div class="scrollable-table-container" style="max-height: 720px;">
          <table class="data-table">
            <thead>
              <tr>
                <th style="position: sticky; left: 0; background: #f8fafc; z-index: 11;">Page</th>
                <th>Content Score</th>
                <th>Total Size</th>
                <th>JS Size</th>
                <th>CSS Size</th>
                <th>Image Size</th>
                <th>Top Optimizations</th>
              </tr>
            </thead>
            <tbody>
              ${pageRows}
            </tbody>
          </table>
        </div>
      </div>
    `;
  }
  
  private renderContentWeightDetailedAnalysis(pages: any[]): string {
    // When no optimization suggestions are available, show detailed analysis
    const totalPages = pages.length;
    if (totalPages === 0) {
      return '<div class="no-data" style="margin-top: 2rem;">No content weight data available</div>';
    }
    
    // Analyze content weight distribution
    const sizeDistribution = pages.map(p => {
      const totalSize = p.contentWeight?.totalSize || 0;
      const jsSize = p.contentWeight?.resources?.javascript?.size || 0;
      const cssSize = p.contentWeight?.resources?.css?.size || 0;
      const imageSize = p.contentWeight?.resources?.images?.size || 0;
      const htmlSize = p.contentWeight?.resources?.html?.size || 0;
      
      return {
        url: p.url,
        title: p.title,
        total: totalSize,
        js: jsSize,
        css: cssSize,
        images: imageSize,
        html: htmlSize,
        score: p.contentWeight?.score || 0
      };
    });
    
    // Find outliers and potential optimization targets
    const avgTotalSize = sizeDistribution.reduce((sum, p) => sum + p.total, 0) / totalPages;
    const largePagesThreshold = avgTotalSize * 1.5;
    const largePages = sizeDistribution.filter(p => p.total > largePagesThreshold);
    
    const avgJsSize = sizeDistribution.reduce((sum, p) => sum + p.js, 0) / totalPages;
    const avgCssSize = sizeDistribution.reduce((sum, p) => sum + p.css, 0) / totalPages;
    const jsHeavyPages = sizeDistribution.filter(p => p.js > 500000); // > 500KB JS
    
    const lowScorePages = sizeDistribution.filter(p => p.score < 70);
    
    const analysisItems = [];
    
    if (largePages.length > 0) {
      analysisItems.push(`
        <div class="issue-item warning" style="margin-bottom: 1rem;">
          <strong>📊 Large Pages Detected</strong><br/>
          <small>${largePages.length} pages exceed the average size by 50% (${this.formatBytes(largePagesThreshold)})</small><br/>
          <small style="color: #6b7280;">Consider reducing total page size through image optimization, code splitting, or lazy loading.</small>
        </div>
      `);
    }
    
    if (jsHeavyPages.length > 0) {
      analysisItems.push(`
        <div class="issue-item error" style="margin-bottom: 1rem;">
          <strong>⚠️ JavaScript Bundle Size</strong><br/>
          <small>${jsHeavyPages.length} pages have JavaScript bundles larger than 500KB</small><br/>
          <small style="color: #6b7280;">Implement code splitting, tree shaking, and lazy loading to reduce initial bundle size.</small>
        </div>
      `);
    }
    
    if (avgCssSize > 200000) { // > 200KB CSS
      analysisItems.push(`
        <div class="issue-item warning" style="margin-bottom: 1rem;">
          <strong>🎨 CSS Bundle Size</strong><br/>
          <small>Average CSS size is ${this.formatBytes(avgCssSize)}</small><br/>
          <small style="color: #6b7280;">Consider CSS minification, unused CSS removal, and critical CSS extraction.</small>
        </div>
      `);
    }
    
    if (lowScorePages.length > 0) {
      analysisItems.push(`
        <div class="issue-item info" style="margin-bottom: 1rem;">
          <strong>🔧 Optimization Opportunities</strong><br/>
          <small>${lowScorePages.length} pages have content weight scores below 70</small><br/>
          <small style="color: #6b7280;">Focus on image compression, resource minification, and eliminating unused code.</small>
        </div>
      `);
    }
    
    if (analysisItems.length === 0) {
      analysisItems.push(`
        <div class="issue-item info" style="margin-bottom: 1rem;">
          <strong>✅ Content Weight Analysis</strong><br/>
          <small>All pages appear to be well-optimized for content weight.</small><br/>
          <small style="color: #6b7280;">Average page size: ${this.formatBytes(avgTotalSize)} | Continue monitoring for performance.</small>
        </div>
      `);
    }
    
    return `
      <div style="margin-top: 2rem;">
        <h3 style="margin-bottom: 1rem;">📋 Content Weight Analysis & Recommendations</h3>
        <div style="background: #f8fafc; padding: 1.5rem; border-radius: var(--radius); margin-bottom: 1rem;">
          <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(150px, 1fr)); gap: 1rem; margin-bottom: 1rem;">
            <div style="text-align: center;">
              <div style="font-size: 1.25rem; font-weight: 600; color: #374151;">${this.formatBytes(avgTotalSize)}</div>
              <div style="font-size: 0.8rem; color: #6b7280;">Avg Total Size</div>
            </div>
            <div style="text-align: center;">
              <div style="font-size: 1.25rem; font-weight: 600; color: #374151;">${this.formatBytes(avgJsSize)}</div>
              <div style="font-size: 0.8rem; color: #6b7280;">Avg JS Size</div>
            </div>
            <div style="text-align: center;">
              <div style="font-size: 1.25rem; font-weight: 600; color: #374151;">${this.formatBytes(avgCssSize)}</div>
              <div style="font-size: 0.8rem; color: #6b7280;">Avg CSS Size</div>
            </div>
            <div style="text-align: center;">
              <div style="font-size: 1.25rem; font-weight: 600; color: #374151;">${largePages.length}</div>
              <div style="font-size: 0.8rem; color: #6b7280;">Large Pages</div>
            </div>
          </div>
        </div>
        
        <div style="margin-top: 1rem;">
          ${analysisItems.join('')}
        </div>
        
        <div style="margin-top: 1.5rem; padding: 1rem; background: #f0f9ff; border-left: 4px solid #0ea5e9; border-radius: 0 var(--radius) var(--radius) 0;">
          <strong>💡 General Optimization Tips:</strong><br/>
          <small style="color: #374151; line-height: 1.6;">
            • Compress and optimize images (use WebP format when possible)<br/>
            • Implement lazy loading for images and non-critical resources<br/>
            • Use code splitting to reduce initial JavaScript bundle size<br/>
            • Enable GZIP/Brotli compression on your server<br/>
            • Remove unused CSS and JavaScript code<br/>
            • Consider using a Content Delivery Network (CDN)
          </small>
        </div>
      </div>
    `;
  }

  private formatOptimizationType(type: string): string {
    const typeNames = {
      'compress-images': 'Compress Images',
      'minify-css': 'Minify CSS',
      'minify-js': 'Minify JavaScript',
      'enable-gzip': 'Enable GZIP Compression',
      'reduce-requests': 'Reduce HTTP Requests',
      'optimize-javascript': 'Optimize JavaScript Bundles',
      'improve-content-ratio': 'Improve Content Ratio',
      'reduce-bundle-size': 'Reduce Bundle Size',
      'eliminate-unused-css': 'Remove Unused CSS',
      'implement-lazy-loading': 'Implement Lazy Loading',
      'enable-caching': 'Enable Browser Caching',
      'optimize-fonts': 'Optimize Web Fonts',
      'reduce-redirects': 'Reduce Redirects',
      'optimize-images': 'Optimize Images',
      'minify-html': 'Minify HTML',
      'use-webp': 'Use WebP Images',
      'critical-css': 'Implement Critical CSS'
    };
    return (typeNames as any)[type] || type.replace(/[-_]/g, ' ').replace(/\b\w/g, l => l.toUpperCase());
  }

  private renderSkippedPagesInfo(failedPages: any[]): string {
    if (failedPages.length === 0) return '';

    const skippedItems = failedPages.map(page => {
      const reasons = [];
      
      // Check for redirect errors
      const redirectError = page.accessibility?.errors?.find((e: any) => e.message?.includes('HTTP Redirect'));
      if (redirectError) {
        const match = redirectError.message.match(/redirects from (.+) to (.+)/);
        if (match) {
          reasons.push(`Redirects to ${match[2]}`);
        } else {
          reasons.push('Page redirects');
        }
      }
      
      // Add other failure reasons if needed
      if (reasons.length === 0) {
        reasons.push('Analysis failed');
      }
      
      return `
        <li class="issue-item info">
          <strong>Skipped: ${this.escape(page.url)}</strong>
          <br/><small style="color: #6b7280;">Reason: ${this.escape(reasons.join(', '))}</small>
          <br/><small style="color: #6b7280;">💡 Consider removing from sitemap or fixing redirects</small>
        </li>
      `;
    }).join('');

    return `
      <div style="margin-top: 2rem;">
        <h3 style="margin-bottom: 1rem;">Pages Skipped from Analysis</h3>
        <div class="scrollable-issues-container">
          <ul class="issue-list">
            ${skippedItems}
          </ul>
        </div>
        <p style="margin-top: 0.5rem; font-style: italic; color: #6b7280;">
          These pages were excluded from accessibility analysis due to technical issues.
        </p>
      </div>
    `;
  }
}
