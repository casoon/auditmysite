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
    const detailedIssues = this.renderDetailedIssuesSection(data);
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
      ${detailedIssues}
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
        max-height: 400px;
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

      .footer {
        text-align: center;
        color: var(--color-subtle);
        font-size: 0.875rem;
        padding: 2rem 0;
        border-top: 1px solid #e5e7eb;
        margin-top: 2rem;
      }

      .detailed-issues-container {
        max-height: 600px;
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
    const grade = data.summary.overallGrade || 'F';
    const score = data.summary.overallScore || 0;

    return `
      <div class="header">
        <div class="header-content">
          <div class="header-info">
            <h1>Website Audit Report</h1>
            <div class="header-meta">
              <div><strong>Domain:</strong> ${this.escape(domain)}</div>
              <div><strong>Generated:</strong> ${timestamp}</div>
              <div><strong>Pages Analyzed:</strong> ${data.summary.testedPages} of ${data.summary.totalPages}</div>
            </div>
          </div>
          <div class="certificate-badge">
            ${certificateSVG}
            <div class="certificate-grade">
              <span class="grade-badge grade-${grade}">Grade ${grade}</span>
              <div style="margin-top: 0.5rem; font-size: 0.9rem;">Overall Score: ${score}/100</div>
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
          <a href="#pages" class="nav-link">Pages</a>
          <a href="#detailed-issues" class="nav-link">Detailed Issues</a>
        </div>
      </nav>
    `;
  }

  private renderSummary(data: EnhancedAuditResult): string {
    const s = data.summary;
    const successRate = s.successRate || 0;
    const duration = Math.round(data.metadata.duration / 1000);

    return `
      <section id="summary" class="section">
        <div class="section-header">
          <h2>Executive Summary</h2>
        </div>
        <div class="section-content">
          <div class="metrics-grid">
            <div class="metric-card">
              <div class="metric-value success">${successRate}%</div>
              <div class="metric-label">Success Rate</div>
            </div>
            <div class="metric-card">
              <div class="metric-value info">${s.testedPages}/${s.totalPages}</div>
              <div class="metric-label">Pages Tested</div>
            </div>
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
            <div class="metric-card">
              <div class="metric-value info">${duration}s</div>
              <div class="metric-label">Analysis Duration</div>
            </div>
            <div class="metric-card">
              <div class="metric-value info">${s.overallScore || 0}/100</div>
              <div class="metric-label">Overall Score</div>
            </div>
          </div>
        </div>
      </section>
    `;
  }

  private renderAccessibilitySection(data: EnhancedAuditResult): string {
    const accessibilityPages = data.pages.filter(p => p.accessibility);
    
    if (accessibilityPages.length === 0) {
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

    const totalErrors = accessibilityPages.reduce((sum, p) => sum + (p.accessibility.errors?.length || 0), 0);
    const totalWarnings = accessibilityPages.reduce((sum, p) => sum + (p.accessibility.warnings?.length || 0), 0);
    const avgScore = accessibilityPages.reduce((sum, p) => sum + (p.accessibility.score || 0), 0) / accessibilityPages.length;

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
          <div class="metrics-grid">
            <div class="metric-card">
              <div class="metric-value info">${Math.round(avgScore)}/100</div>
              <div class="metric-label">Average Score</div>
            </div>
            <div class="metric-card">
              <div class="metric-value error">${totalErrors}</div>
              <div class="metric-label">Total Errors</div>
            </div>
            <div class="metric-card">
              <div class="metric-value warning">${totalWarnings}</div>
              <div class="metric-label">Total Warnings</div>
            </div>
            <div class="metric-card">
              <div class="metric-value info">${accessibilityPages.length}</div>
              <div class="metric-label">Pages Analyzed</div>
            </div>
          </div>
          ${this.renderIssues(accessibilityPages, 'accessibility')}
        </div>
      </section>
    `;
  }

  private renderPerformanceSection(data: EnhancedAuditResult): string {
    const performancePages = data.pages.filter(p => p.performance);
    
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
          ${this.renderIssues(performancePages, 'performance')}
        </div>
      </section>
    `;
  }

  private renderSEOSection(data: EnhancedAuditResult): string {
    const seoPages = data.pages.filter(p => p.seo);
    
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
          ${this.renderSEOAnalysis(seoPages)}
        </div>
      </section>
    `;
  }

  private renderContentWeightSection(data: EnhancedAuditResult): string {
    const contentWeightPages = data.pages.filter(p => p.contentWeight);
    
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
      html: contentWeightPages.reduce((sum, p) => sum + (p.contentWeight.resources?.html?.size || 0), 0) / contentWeightPages.length,
      css: contentWeightPages.reduce((sum, p) => sum + (p.contentWeight.resources?.css?.size || 0), 0) / contentWeightPages.length,
      javascript: contentWeightPages.reduce((sum, p) => sum + (p.contentWeight.resources?.javascript?.size || 0), 0) / contentWeightPages.length,
      images: contentWeightPages.reduce((sum, p) => sum + (p.contentWeight.resources?.images?.size || 0), 0) / contentWeightPages.length,
      other: contentWeightPages.reduce((sum, p) => sum + (p.contentWeight.resources?.other?.size || 0), 0) / contentWeightPages.length
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
                <div class="metric-value info">${this.formatBytes(avgResources.html)}</div>
                <div class="metric-label">HTML</div>
              </div>
              <div class="metric-card">
                <div class="metric-value info">${this.formatBytes(avgResources.css)}</div>
                <div class="metric-label">CSS</div>
              </div>
              <div class="metric-card">
                <div class="metric-value info">${this.formatBytes(avgResources.javascript)}</div>
                <div class="metric-label">JavaScript</div>
              </div>
              <div class="metric-card">
                <div class="metric-value info">${this.formatBytes(avgResources.images)}</div>
                <div class="metric-label">Images</div>
              </div>
              <div class="metric-card">
                <div class="metric-value info">${this.formatBytes(avgResources.other)}</div>
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
    // First try to render actual issues if they exist
    const hasActualIssues = seoPages.some(page => 
      page.seo.issues && page.seo.issues.length > 0
    );
    
    if (hasActualIssues) {
      return this.renderIssues(seoPages, 'seo');
    }
    
    // Generate synthetic issues based on SEO metrics
    const seoIssues: any[] = [];
    
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
      } else if (!titleOptimal) {
        const titleLength = titleContent.length;
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
      } else if (!descriptionOptimal) {
        const descriptionLength = descriptionContent.length;
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
    
    if (seoIssues.length === 0) {
      return '<div class="no-data">No significant SEO issues found</div>';
    }
    
    // Group by severity 
    const issueGroups = seoIssues.reduce((groups, issue) => {
      const severity = issue.severity || 'info';
      if (!groups[severity]) groups[severity] = [];
      groups[severity].push(issue);
      return groups;
    }, {});
    
    const severityOrder = ['error', 'warning', 'info'];
    const sortedSeverities = severityOrder.filter(sev => issueGroups[sev]);
    
    const groupSections = sortedSeverities.map(severity => {
      const issues = issueGroups[severity];
      const issueItems = issues.map((issue: any) => {
        return `
          <li class="issue-item ${severity}">
            <strong>${this.escape(issue.message)}</strong>
            ${issue.context ? `<br/><small style="color: #6b7280;">Details: ${this.escape(issue.context)}</small>` : ''}
            <br/><small style="color: #6b7280;">Page: ${this.escape(issue.pageUrl)}</small>
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
    
    return `
      <div style="margin-top: 2rem;">
        <h3 style="margin-bottom: 1rem;">SEO Issues Analysis</h3>
        <div class="scrollable-issues-container">
          ${groupSections}
        </div>
      </div>
    `;
  }

  private renderMobileFriendlinessSection(data: EnhancedAuditResult): string {
    const mobilePages = data.pages.filter(p => p.mobileFriendliness);
    
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

    const avgScore = mobilePages.reduce((sum, p) => sum + (p.mobileFriendliness.score || 0), 0) / mobilePages.length;
    const totalIssues = mobilePages.reduce((sum, p) => sum + (p.mobileFriendliness.issues?.length || 0), 0);

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
              <div class="metric-value info">${Math.round(avgScore)}/100</div>
              <div class="metric-label">Average Score</div>
            </div>
            <div class="metric-card">
              <div class="metric-value warning">${totalIssues}</div>
              <div class="metric-label">Total Issues</div>
            </div>
            <div class="metric-card">
              <div class="metric-value info">${mobilePages.length}</div>
              <div class="metric-label">Pages Analyzed</div>
            </div>
          </div>
          ${this.renderIssues(mobilePages, 'mobileFriendliness')}
        </div>
      </section>
    `;
  }

  private renderPagesSection(data: EnhancedAuditResult): string {
    const rows = data.pages.map(p => {
      const accessibilityScore = p.accessibility?.score || 0;
      const performanceLCP = p.performance?.coreWebVitals?.largestContentfulPaint || 0;
      const seoScore = p.seo?.score || 0;
      const contentWeightScore = p.contentWeight?.score || 0;
      const mobileScore = p.mobileFriendliness?.score || 0;

      return `
        <tr>
          <td>
            <strong>${this.escape(p.title || p.url)}</strong><br/>
            <small style="color: #6b7280;">${this.escape(p.url)}</small>
          </td>
          <td>
            <span class="page-status status-${p.status}">${p.status.toUpperCase()}</span>
          </td>
          <td>${accessibilityScore}/100</td>
          <td>${performanceLCP ? Math.round(performanceLCP) + 'ms' : '—'}</td>
          <td>${seoScore}/100</td>
          <td>${contentWeightScore}/100</td>
          <td>${mobileScore}/100</td>
          <td>${Math.round(p.duration)}ms</td>
        </tr>
      `;
    }).join('');

    return `
      <section id="pages" class="section">
        <div class="section-header">
          <h2>Individual Pages</h2>
        </div>
        <div class="section-content">
          <div style="overflow-x: auto;">
            <table class="data-table">
              <thead>
                <tr>
                  <th>Page</th>
                  <th>Status</th>
                  <th>A11y Score</th>
                  <th>LCP</th>
                  <th>SEO Score</th>
                  <th>Content Score</th>
                  <th>Mobile Score</th>
                  <th>Duration</th>
                </tr>
              </thead>
              <tbody>
                ${rows}
              </tbody>
            </table>
          </div>
        </div>
      </section>
    `;
  }

  private renderIssues(pages: any[], analysisType: string): string {
    const allIssues = pages.flatMap(p => {
      const analysis = p[analysisType];
      if (!analysis) return [];
      
      // Handle different issue structures - include recommendations for mobile friendliness
      const issues = analysis.issues || analysis.errors || analysis.warnings || analysis.recommendations || [];
      return Array.isArray(issues) ? issues.map(issue => ({...issue, pageUrl: p.url})) : [];
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
      // Show ALL issues for errors, limit others to 5
      const maxIssues = severity === 'error' ? issueGroups[severity].length : 5;
      const issues = issueGroups[severity].slice(0, maxIssues);
      const issueItems = issues.map((issue: any) => {
        const message = issue.message || issue.issue || issue.recommendation || issue.description || 'No description available';
        const context = issue.context || issue.selector || issue.impact || '';
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
      
      // Add scrollable container for errors
      const containerClass = severity === 'error' && totalCount > 10 ? 'scrollable-issues-container' : '';
      const maxHeight = severity === 'error' && totalCount > 10 ? 'style="max-height: 400px; overflow-y: auto;"' : '';
      
      return `
        <div style="margin-bottom: 2rem;">
          <h4 style="margin-bottom: 1rem; color: var(--color-text);">
            ${severityLabel} (${totalCount})
          </h4>
          <div class="${containerClass}" ${maxHeight}>
            <ul class="issue-list">
              ${issueItems}
            </ul>
          </div>
          ${(severity !== 'error' && totalCount > 5) ? `<p style="margin-top: 0.5rem;"><em>... and ${totalCount - 5} more ${severity} issues</em></p>` : ''}
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

  private generateDetailedIssuesMarkdown(data: EnhancedAuditResult): string {
    const lines: string[] = [];
    
    lines.push('# Detailed Accessibility Issues Report');
    lines.push('');
    lines.push(`**Generated:** ${data.metadata.timestamp}`);
    lines.push(`**Tool Version:** ${data.metadata.version}`);
    lines.push('');
    
    // Summary
    lines.push('## Summary');
    lines.push(`- **Tested Pages:** ${data.summary.testedPages}`);
    lines.push(`- **Failed Pages:** ${data.summary.failedPages}`);
    lines.push(`- **Total Errors:** ${data.summary.totalErrors}`);
    lines.push(`- **Total Warnings:** ${data.summary.totalWarnings}`);
    lines.push('');
    
    // Issues by page
    data.pages.forEach((page, index) => {
      const hasAccessibilityIssues = page.accessibility && (
        (page.accessibility.errors && page.accessibility.errors.length > 0) ||
        (page.accessibility.warnings && page.accessibility.warnings.length > 0) ||
        (page.accessibility.notices && page.accessibility.notices.length > 0)
      );
      
      if (hasAccessibilityIssues) {
        lines.push(`## Page ${index + 1}: ${page.title || 'Untitled'}`);
        lines.push(`**URL:** ${page.url}`);
        lines.push(`**Status:** ${page.status.toUpperCase()}`);
        lines.push('');
        
        // Group issues by type
        const errorIssues = page.accessibility.errors || [];
        const warningIssues = page.accessibility.warnings || [];
        const noticeIssues = page.accessibility.notices || [];
        
        if (errorIssues.length > 0) {
          lines.push('### ❌ Errors');
          errorIssues.forEach((issue: any, issueIndex: number) => {
            this.formatIssueForMarkdown(lines, issue, issueIndex + 1, 'error');
          });
          lines.push('');
        }
        
        if (warningIssues.length > 0) {
          lines.push('### ⚠️ Warnings');
          warningIssues.forEach((issue: any, issueIndex: number) => {
            this.formatIssueForMarkdown(lines, issue, issueIndex + 1, 'warning');
          });
          lines.push('');
        }
        
        if (noticeIssues && noticeIssues.length > 0) {
          lines.push('### ℹ️ Notices');
          noticeIssues.forEach((issue: any, issueIndex: number) => {
            this.formatIssueForMarkdown(lines, issue, issueIndex + 1, 'notice');
          });
          lines.push('');
        }
        
        lines.push('---');
        lines.push('');
      }
    });
    
    return lines.join('\n');
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
    const duration = Math.round(data.metadata.duration / 1000);
    const timestamp = new Date(data.metadata.timestamp).toLocaleString();
    
    return `
      <footer class="footer">
        <div>
          Generated by <strong>AuditMySite v${data.metadata.version}</strong> • 
          Analysis Duration: ${duration}s • 
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

  private calculateOverallScore(data: EnhancedAuditResult): number {
    if (data.pages.length === 0) return 0;
    
    let totalWeightedScore = 0;
    let totalWeight = 0;
    
    data.pages.forEach(page => {
      // Accessibility (30% weight) - use pa11yScore from comprehensive analysis
      const a11yScore = (page as any).pa11yScore || page.accessibility?.score || 0;
      if (a11yScore > 0) {
        totalWeightedScore += a11yScore * 0.3;
        totalWeight += 0.3;
      }
      
      // Performance (25% weight) - use enhancedPerformance or performance
      const perfScore = (page as any).enhancedPerformance?.performanceScore || page.performance?.score || 0;
      if (perfScore > 0) {
        totalWeightedScore += perfScore * 0.25;
        totalWeight += 0.25;
      }
      
      // SEO (25% weight) - use enhancedSEO or seo
      const seoScore = (page as any).enhancedSEO?.seoScore || page.seo?.score || 0;
      if (seoScore > 0) {
        totalWeightedScore += seoScore * 0.25;
        totalWeight += 0.25;
      }
      
      // Content Weight (10% weight)
      const contentScore = (page as any).contentWeight?.contentScore || page.contentWeight?.score || 0;
      if (contentScore > 0) {
        totalWeightedScore += contentScore * 0.1;
        totalWeight += 0.1;
      }
      
      // Mobile Friendliness (10% weight)
      const mobileScore = (page as any).mobileFriendliness?.overallScore || page.mobileFriendliness?.score || 0;
      if (mobileScore > 0) {
        totalWeightedScore += mobileScore * 0.1;
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
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + ' ' + sizes[i];
  }

  private renderContentWeightOptimizations(pages: any[]): string {
    const allOptimizations = pages.flatMap(p => p.contentWeight?.recommendations || p.contentWeight?.optimizations || []);
    
    if (allOptimizations.length === 0) {
      return '<div class="no-data" style="margin-top: 2rem;">No optimization suggestions available</div>';
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
          messages: []
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

    const sortedOptimizations = Object.values(optimizationGroups)
      .sort((a: any, b: any) => b.totalSavings - a.totalSavings);

    const optimizationItems = sortedOptimizations.map((opt: any) => {
      const priorityClass = opt.priority === 'high' ? 'error' : opt.priority === 'medium' ? 'warning' : 'info';
      return `
        <li class="issue-item ${priorityClass}">
          <strong>${this.formatOptimizationType(opt.type)} (${opt.count} pages)</strong><br/>
          <em>Potential savings: ${this.formatBytes(opt.totalSavings)}</em><br/>
          <small style="color: #6b7280;">${opt.messages[0] || 'Optimization available'}</small>
        </li>
      `;
    }).join('');

    return `
      <div style="margin-top: 2rem;">
        <h3 style="margin-bottom: 1rem;">Optimization Recommendations</h3>
        <ul class="issue-list">
          ${optimizationItems}
        </ul>
      </div>
    `;
  }

  private formatOptimizationType(type: string): string {
    const typeNames = {
      'compress-images': 'Compress Images',
      'minify-css': 'Minify CSS',
      'minify-js': 'Minify JavaScript',
      'enable-gzip': 'Enable GZIP Compression',
      'reduce-requests': 'Reduce HTTP Requests'
    };
    return (typeNames as any)[type] || type.replace('-', ' ').replace(/\b\w/g, l => l.toUpperCase());
  }
}
