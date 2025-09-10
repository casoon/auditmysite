export class HtmlGenerator {
  generateAccessibilitySection(data: any): string {
    return `
      <div class="table-wrapper">
        <table class="data-table">
        <thead><tr><th>Page</th><th>Errors</th><th>Warnings</th><th>Pa11y Score</th></tr></thead>
        <tbody>
          ${data.pages.map((page: any) => {
            // Debug: Check multiple possible locations for pa11yScore
            // Standard path: page.pa11yScore (from AccessibilityChecker)
            // Enhanced Analysis path: page.issues?.pa11yScore 
            // Alternative: Calculated from pa11yIssues if available
            let pa11yScore = page.pa11yScore ?? page.issues?.pa11yScore ?? 'N/A';
            
            // If we still don't have a score, try to calculate from pa11yIssues
            if (pa11yScore === 'N/A' && (page.pa11yIssues || page.issues?.pa11yIssues)) {
              const issues = page.pa11yIssues || page.issues?.pa11yIssues || [];
              if (issues.length === 0) {
                pa11yScore = 100;
              } else {
                const errorIssues = issues.filter((issue: any) => issue.type === 'error').length;
                const totalIssues = issues.length;
                pa11yScore = Math.max(0, 100 - (errorIssues * 10) - (totalIssues - errorIssues) * 2);
              }
            }
            
            const formattedScore = pa11yScore !== 'N/A' && typeof pa11yScore === 'number' ? 
              `${Math.round(pa11yScore)}/100` : pa11yScore;
            
            // Debug logging (can be removed in production)
            if (pa11yScore === 'N/A') {
              console.debug(`Pa11y score not found for ${page.url}. Available data:`, {
                hasDirectScore: page.pa11yScore !== undefined,
                hasIssuesScore: page.issues?.pa11yScore !== undefined,
                hasPa11yIssues: !!(page.pa11yIssues || page.issues?.pa11yIssues),
                pa11yIssuesCount: (page.pa11yIssues || page.issues?.pa11yIssues || []).length,
                pageKeys: Object.keys(page)
              });
            }
            
            return `<tr>
              <td>${page.url}</td>
              <td>${page.errors}</td>
              <td>${page.warnings}</td>
              <td>${formattedScore}</td>
            </tr>`;
          }).join('')}
        </tbody>
        </table>
      </div>`;
  }

  generatePerformanceSection(data: any): string {
    // Check if we have enhanced performance data
    const hasEnhancedData = data.pages.some((page: any) => page.enhancedPerformance || page.contentWeight);
    
    if (hasEnhancedData) {
      return this.generateEnhancedPerformanceSection(data);
    }
    
    // Fallback to basic performance section
    const formatMetric = (value: any): string => {
      if (value === null || value === undefined || value === 'N/A') return 'N/A';
      const numValue = parseFloat(value);
      return isNaN(numValue) ? 'N/A' : `${Math.round(numValue)}ms`;
    };

    return `<div class="table-container">
      <div class="table-header">
        <div>
          <!-- Description removed to avoid duplication with template -->
        </div>
      </div>
      <div class="table-wrapper">
        <table class="data-table">
        <thead><tr><th>Page</th><th>Load Time</th><th>FCP</th><th>LCP</th><th>DOM Loaded</th><th>First Paint</th></tr></thead>
        <tbody>
          ${data.pages.map((page: any) => {
            const perf = page.issues?.performanceMetrics || {};
            return `<tr>
              <td>${page.url}</td>
              <td>${formatMetric(perf.loadTime)}</td>
              <td>${formatMetric(perf.firstContentfulPaint)}</td>
              <td>${formatMetric(perf.largestContentfulPaint)}</td>
              <td>${formatMetric(perf.domContentLoaded)}</td>
              <td>${formatMetric(perf.firstPaint)}</td>
            </tr>`;
          }).join('')}
        </tbody>
        </table>
      </div>`;
  }

  generateSeoSection(data: any): string {
    // Check if we have enhanced SEO data (note: field is enhancedSEO not enhancedSeo)
    const hasEnhancedData = data.pages.some((page: any) => page.enhancedSEO || page.enhancedSeo);
    
    if (hasEnhancedData) {
      return this.generateEnhancedSeoSection(data);
    }
    
    // Fallback to basic SEO section
    return `<div class="table-container">
      <div class="table-header">
        <div>
          <!-- Description removed to avoid duplication with template -->
        </div>
      </div>
      <div class="table-wrapper">
        <table class="data-table">
        <thead><tr><th>Page & Title</th><th>Headings</th><th>Images w/o Alt</th><th>Buttons w/o Label</th></tr></thead>
        <tbody>
          ${data.pages.map((page: any) => {
            const pageTitle = page.title || 'No title';
            const pageName = this.getPageName(page.url);
            return `<tr>
              <td>
                <div class="page-info">
                  <strong>${pageName}</strong><br>
                  <small class="page-title">${pageTitle}</small><br>
                  <small class="page-url">${page.url}</small>
                </div>
              </td>
              <td>${page.issues?.headingsCount ?? 'N/A'}</td>
              <td>${page.issues?.imagesWithoutAlt ?? 'N/A'}</td>
              <td>${page.issues?.buttonsWithoutLabel ?? 'N/A'}</td>
            </tr>`;
          }).join('')}
        </tbody>
        </table>
      </div>`;
  }

  private getPageName(url: string): string {
    try {
      const urlObj = new URL(url);
      const pathname = urlObj.pathname;
      return pathname === '/' ? 'Home' : (pathname.split('/').pop() || pathname);
    } catch {
      return url;
    }
  }

  /**
   * Generate Enhanced Accessibility Report for EnhancedAccessibilityResult
   */
  generateEnhancedReport(results: any, url: string): string {
    const template = this.getEnhancedReportTemplate();
    
    // Process results to match expected format
    const processedData = this.processEnhancedResults(results);
    
    let html = template;
    
    // Replace placeholders with actual data
    html = html.replace('{{SITE_URL}}', url);
    html = html.replace('{{TIMESTAMP}}', new Date().toLocaleString());
    html = html.replace('{{TOTAL_PAGES}}', processedData.summary.testedPages.toString());
    html = html.replace('{{PASSED_PAGES}}', processedData.summary.passedPages.toString());
    html = html.replace('{{FAILED_PAGES}}', processedData.summary.failedPages.toString());
    html = html.replace('{{SUCCESS_RATE}}', Math.round((processedData.summary.passedPages / processedData.summary.testedPages) * 100).toString());
    
    // Generate sections
    html = html.replace('{{ACCESSIBILITY_SECTION}}', this.generateAccessibilitySection(processedData));
    html = html.replace('{{PERFORMANCE_SECTION}}', this.generateEnhancedPerformanceSection(processedData));
    html = html.replace('{{SEO_SECTION}}', this.generateEnhancedSeoSection(processedData));
    html = html.replace('{{CONTENT_WEIGHT_SECTION}}', this.generateContentWeightSection(processedData));
    html = html.replace('{{QUALITY_SCORE_SECTION}}', this.generateQualityScoreSection(processedData));
    
    return html;
  }

  private processEnhancedResults(results: any): any {
    // Convert EnhancedAccessibilityResult to format expected by existing generators
    const pages = [];
    
    if (results.enhancedResults) {
      // Results from CLI enhanced analysis
      pages.push(...results.enhancedResults.map((page: any) => ({
        url: page.url,
        title: page.title,
        issues: {
          errors: page.errors,
          warnings: page.warnings,
          passed: page.passed,
          pa11yScore: page.pa11yScore,
          pa11yIssues: page.pa11yIssues
        },
        // Also add pa11yScore directly to page level for backward compatibility
        pa11yScore: page.pa11yScore,
        pa11yIssues: page.pa11yIssues,
        enhancedPerformance: page.enhancedPerformance,
        enhancedSeo: page.enhancedSEO, 
        contentWeight: page.contentWeight,
        qualityScore: page.qualityScore
      })));
    } else {
      // Single result from direct enhanced analysis
      pages.push({
        url: results.url || 'Unknown',
        title: results.title || 'N/A',
        issues: {
          errors: results.errors?.length || 0,
          warnings: results.warnings?.length || 0,
          passed: results.passed,
          pa11yScore: results.pa11yScore,
          pa11yIssues: results.pa11yIssues
        },
        // Also add pa11yScore directly to page level for backward compatibility
        pa11yScore: results.pa11yScore,
        pa11yIssues: results.pa11yIssues,
        enhancedPerformance: results.enhancedPerformance,
        enhancedSeo: results.enhancedSEO,
        contentWeight: results.contentWeight,
        qualityScore: results.qualityScore
      });
    }
    
    return {
      pages,
      summary: results.summary || {
        testedPages: pages.length,
        passedPages: pages.filter(p => p.issues.passed).length,
        failedPages: pages.filter(p => !p.issues.passed).length,
        totalErrors: pages.reduce((sum, p) => sum + (p.issues.errors || 0), 0),
        totalWarnings: pages.reduce((sum, p) => sum + (p.issues.warnings || 0), 0)
      }
    };
  }

  private generateEnhancedPerformanceSection(data: any): string {
    // Check if we have enhanced performance data
    const hasEnhancedData = data.pages.some((page: any) => page.enhancedPerformance);
    
    if (hasEnhancedData) {
      return this.generateEnhancedPerformanceHtml(data);
    }
    
    // Fallback to basic performance section
    return this.generatePerformanceSection(data);
  }

  private generateEnhancedSeoSection(data: any): string {
    // Check if we have enhanced SEO data
    const hasEnhancedData = data.pages.some((page: any) => page.enhancedSEO || page.enhancedSeo);
    
    if (hasEnhancedData) {
      return this.generateEnhancedSeoHtml(data);
    }
    
    // Fallback to basic SEO section
    return this.generateSeoSection(data);
  }

  private generateContentWeightSection(data: any): string {
    const pagesWithContentWeight = data.pages.filter((page: any) => page.contentWeight);
    
    if (pagesWithContentWeight.length === 0) {
      return '<div class="content-weight-section"><p>No content weight data available.</p></div>';
    }

    let html = '<div class="content-weight-overview">';
    html += '<h3>Content Weight Analysis</h3>';
    
    // Content Weight metrics grid
    html += '<div class="metrics-grid">';
    
    const avgContentScore = pagesWithContentWeight.reduce((sum: number, page: any) => 
      sum + (page.contentWeight.contentScore || 0), 0) / pagesWithContentWeight.length;
    
    html += `<div class="metric-card ${this.getScoreClass(avgContentScore)}">`;
    html += '<div class="metric-label">Content Score</div>';
    html += `<div class="metric-value">${Math.round(avgContentScore)}</div>`;
    html += `<div class="metric-grade">${this.getGrade(avgContentScore)}</div>`;
    html += '</div>';
    
    // Average resource sizes
    const avgTotalSize = pagesWithContentWeight.reduce((sum: number, page: any) => 
      sum + (page.contentWeight.contentMetrics?.totalSize || 0), 0) / pagesWithContentWeight.length;
    
    html += '<div class="metric-card">';
    html += '<div class="metric-label">Avg Total Size</div>';
    html += `<div class="metric-value">${this.formatBytes(avgTotalSize)}</div>`;
    html += '</div>';
    
    const avgTextRatio = pagesWithContentWeight.reduce((sum: number, page: any) => 
      sum + (page.contentWeight.contentMetrics?.textToCodeRatio || 0), 0) / pagesWithContentWeight.length;
    
    html += '<div class="metric-card">';
    html += '<div class="metric-label">Text-to-Code Ratio</div>';
    html += `<div class="metric-value">${(avgTextRatio * 100).toFixed(1)}%</div>`;
    html += '</div>';
    
    html += '</div>'; // End metrics-grid
    
    // Content Weight Details Table
    html += '<div class="analysis-section">';
    html += '<table class="content-weight-table">';
    html += '<thead><tr><th>Page</th><th>HTML</th><th>CSS</th><th>JS</th><th>Images</th><th>Total Size</th><th>Score</th></tr></thead>';
    html += '<tbody>';
    
    data.pages.forEach((page: any) => {
      const cw = page.contentWeight;
      if (!cw) return;
      
      const pageName = this.getPageName(page.url);
      const resources = cw.resourceAnalysis || {};
      
      html += '<tr>';
      html += `<td>${pageName}</td>`;
      html += `<td>${this.formatBytes(resources.html?.size || 0)}</td>`;
      html += `<td>${this.formatBytes(resources.css?.size || 0)}</td>`;
      html += `<td>${this.formatBytes(resources.javascript?.size || 0)}</td>`;
      html += `<td>${this.formatBytes(resources.images?.size || 0)}</td>`;
      html += `<td>${this.formatBytes(cw.contentMetrics?.totalSize || 0)}</td>`;
      html += `<td class="score-cell score-${this.getScoreClass(cw.contentScore || 0)}">${Math.round(cw.contentScore || 0)}</td>`;
      html += '</tr>';
    });
    
    html += '</tbody></table>';
    html += '</div>'; // End analysis-section
    html += '</div>'; // End content-weight-overview
    
    return html;
  }

  private generateQualityScoreSection(data: any): string {
    const pagesWithQuality = data.pages.filter((page: any) => page.qualityScore);
    
    if (pagesWithQuality.length === 0) {
      return '<div class="quality-score-section"><p>No quality score data available.</p></div>';
    }

    let html = '<div class="quality-score-overview">';
    html += '<h3>Overall Quality Scores</h3>';
    
    // Quality Score Summary
    const avgQualityScore = pagesWithQuality.reduce((sum: number, page: any) => 
      sum + (page.qualityScore.score || 0), 0) / pagesWithQuality.length;
    
    html += '<div class="quality-summary">';
    html += `<div class="quality-card ${this.getScoreClass(avgQualityScore)}">`;
    html += '<div class="quality-label">Average Quality Score</div>';
    html += `<div class="quality-value">${Math.round(avgQualityScore)}/100</div>`;
    html += `<div class="quality-grade">${this.getGrade(avgQualityScore)}</div>`;
    html += '</div>';
    html += '</div>';
    
    // Breakdown by category
    const avgBreakdown = pagesWithQuality.reduce((acc: any, page: any) => {
      const breakdown = page.qualityScore.breakdown || {};
      acc.performance += breakdown.performance || 0;
      acc.seo += breakdown.seo || 0;
      acc.accessibility += breakdown.accessibility || 0;
      acc.content += breakdown.content || 0;
      return acc;
    }, { performance: 0, seo: 0, accessibility: 0, content: 0 });
    
    // Average the breakdown scores
    Object.keys(avgBreakdown).forEach(key => {
      avgBreakdown[key] = avgBreakdown[key] / pagesWithQuality.length;
    });
    
    html += '<div class="breakdown-grid">';
    html += `<div class="breakdown-card"><div class="breakdown-label">Performance</div><div class="breakdown-score">${Math.round(avgBreakdown.performance)}</div></div>`;
    html += `<div class="breakdown-card"><div class="breakdown-label">SEO</div><div class="breakdown-score">${Math.round(avgBreakdown.seo)}</div></div>`;
    html += `<div class="breakdown-card"><div class="breakdown-label">Accessibility</div><div class="breakdown-score">${Math.round(avgBreakdown.accessibility)}</div></div>`;
    html += `<div class="breakdown-card"><div class="breakdown-label">Content</div><div class="breakdown-score">${Math.round(avgBreakdown.content)}</div></div>`;
    html += '</div>';
    
    html += '</div>'; // End quality-score-overview
    
    return html;
  }

  private getScoreClass(score: number): string {
    if (score >= 90) return 'excellent';
    if (score >= 75) return 'good';
    if (score >= 50) return 'needs-improvement';
    return 'poor';
  }

  private getGrade(score: number): string {
    if (score >= 90) return 'A';
    if (score >= 80) return 'B';
    if (score >= 70) return 'C';
    if (score >= 60) return 'D';
    return 'F';
  }

  private formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  }

  private getEnhancedReportTemplate(): string {
    return `<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>🚀 Enhanced Accessibility Analysis Report</title>
    <style>
        :root {
            --color-primary: #2563eb;
            --color-secondary: #64748b;
            --color-success: #10b981;
            --color-warning: #f59e0b;
            --color-error: #ef4444;
            --color-background: #f8fafc;
            --color-card: #ffffff;
            --border-radius: 8px;
            --shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
        }
        
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            margin: 0;
            padding: 20px;
            background: var(--color-background);
            color: #1f2937;
        }
        
        .container {
            max-width: 1400px;
            margin: 0 auto;
            background: var(--color-card);
            padding: 30px;
            border-radius: var(--border-radius);
            box-shadow: var(--shadow);
        }
        
        .header {
            text-align: center;
            margin-bottom: 40px;
            padding-bottom: 20px;
            border-bottom: 2px solid var(--color-primary);
        }
        
        .header h1 {
            color: var(--color-primary);
            margin: 0;
            font-size: 2.5em;
            font-weight: 700;
        }
        
        .header .subtitle {
            color: var(--color-secondary);
            margin: 10px 0 0 0;
            font-size: 1.1em;
        }
        
        .summary {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 20px;
            margin: 30px 0;
        }
        
        .metric {
            background: var(--color-card);
            padding: 20px;
            border-radius: var(--border-radius);
            text-align: center;
            border: 1px solid #e2e8f0;
            transition: transform 0.2s;
        }
        
        .metric:hover {
            transform: translateY(-2px);
            box-shadow: var(--shadow);
        }
        
        .metric-value {
            font-size: 2.5em;
            font-weight: bold;
            color: var(--color-primary);
            margin: 0;
        }
        
        .metric-label {
            font-size: 0.9em;
            color: var(--color-secondary);
            margin-top: 8px;
            text-transform: uppercase;
            letter-spacing: 0.5px;
        }
        
        .section {
            margin: 40px 0;
            padding: 30px;
            background: var(--color-card);
            border-radius: var(--border-radius);
            border: 1px solid #e2e8f0;
        }
        
        .section h2 {
            color: var(--color-primary);
            margin: 0 0 20px 0;
            font-size: 1.8em;
            display: flex;
            align-items: center;
            gap: 10px;
        }
        
        .metrics-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
            gap: 20px;
            margin: 20px 0;
        }
        
        .metric-card {
            padding: 20px;
            border-radius: var(--border-radius);
            text-align: center;
            border: 2px solid #e2e8f0;
        }
        
        .metric-card.excellent { border-color: var(--color-success); background: #f0fdf4; }
        .metric-card.good { border-color: #3b82f6; background: #eff6ff; }
        .metric-card.needs-improvement { border-color: var(--color-warning); background: #fffbeb; }
        .metric-card.poor { border-color: var(--color-error); background: #fef2f2; }
        
        .data-table {
            width: 100%;
            border-collapse: collapse;
            margin: 20px 0;
            background: var(--color-card);
            border-radius: var(--border-radius);
            overflow: hidden;
            box-shadow: var(--shadow);
        }
        
        .data-table th,
        .data-table td {
            padding: 15px;
            text-align: left;
            border-bottom: 1px solid #e2e8f0;
        }
        
        .data-table th {
            background: #f1f5f9;
            font-weight: 600;
            color: var(--color-primary);
            text-transform: uppercase;
            font-size: 0.85em;
            letter-spacing: 0.5px;
        }
        
        .data-table tbody tr:hover {
            background: #f8fafc;
        }
        
        .score-cell {
            text-align: center;
            font-weight: bold;
            padding: 8px 12px;
            border-radius: 4px;
        }
        
        .score-excellent { background: var(--color-success); color: white; }
        .score-good { background: #3b82f6; color: white; }
        .score-needs-improvement { background: var(--color-warning); color: white; }
        .score-poor { background: var(--color-error); color: white; }
        
        .footer {
            margin-top: 50px;
            padding-top: 30px;
            border-top: 1px solid #e2e8f0;
            text-align: center;
            color: var(--color-secondary);
        }
        
        .footer p {
            margin: 5px 0;
        }
        
        @media (max-width: 768px) {
            .container {
                padding: 20px;
                margin: 10px;
            }
            
            .summary {
                grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
            }
            
            .header h1 {
                font-size: 2em;
            }
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🚀 Enhanced Accessibility Analysis</h1>
            <div class="subtitle">
                <strong>{{SITE_URL}}</strong> • Generated on {{TIMESTAMP}}
            </div>
        </div>
        
        <div class="summary">
            <div class="metric">
                <div class="metric-value">{{TOTAL_PAGES}}</div>
                <div class="metric-label">Pages Tested</div>
            </div>
            <div class="metric">
                <div class="metric-value">{{PASSED_PAGES}}</div>
                <div class="metric-label">Passed</div>
            </div>
            <div class="metric">
                <div class="metric-value">{{FAILED_PAGES}}</div>
                <div class="metric-label">Failed</div>
            </div>
            <div class="metric">
                <div class="metric-value">{{SUCCESS_RATE}}%</div>
                <div class="metric-label">Success Rate</div>
            </div>
        </div>
        
        <div class="section">
            <h2>🔍 Accessibility Analysis</h2>
            {{ACCESSIBILITY_SECTION}}
        </div>
        
        <div class="section">
            <h2>⚡ Performance Analysis</h2>
            {{PERFORMANCE_SECTION}}
        </div>
        
        <div class="section">
            <h2>📈 SEO Analysis</h2>
            {{SEO_SECTION}}
        </div>
        
        <div class="section">
            <h2>📏 Content Weight Analysis</h2>
            {{CONTENT_WEIGHT_SECTION}}
        </div>
        
        <div class="section">
            <h2>🏆 Quality Score</h2>
            {{QUALITY_SCORE_SECTION}}
        </div>
        
        <div class="footer">
            <p><strong>Generated by AuditMySite Enhanced Analysis</strong></p>
            <p>Professional Website Quality Assessment</p>
        </div>
    </div>
</body>
</html>`;
  }

  // Include the enhanced HTML generation methods from the enhanced generator
  generateEnhancedPerformanceHtml(data: any): string {
    // Performance Overview with KPIs
    const pagesWithPerformance = data.pages.filter((page: any) => page.enhancedPerformance);
    
    // Debug logging to see what data we have
    console.log('DEBUG: Enhanced Performance Data Check:', {
      totalPages: data.pages.length,
      pagesWithPerformance: pagesWithPerformance.length,
      firstPageKeys: data.pages.length > 0 ? Object.keys(data.pages[0]) : [],
      firstPagePerformance: data.pages.length > 0 ? data.pages[0].enhancedPerformance : null
    });
    
    if (pagesWithPerformance.length === 0) {
      console.log('DEBUG: No enhanced performance data found, falling back to basic');
      return this.generateBasicPerformanceSection(data);
    }

    const avgScore = pagesWithPerformance.reduce((sum: number, page: any) => sum + (page.enhancedPerformance.score || 0), 0) / pagesWithPerformance.length;
    const avgLCP = pagesWithPerformance.reduce((sum: number, page: any) => sum + (page.enhancedPerformance.coreWebVitals?.lcp?.value || 0), 0) / pagesWithPerformance.length;
    const avgCLS = pagesWithPerformance.reduce((sum: number, page: any) => sum + (page.enhancedPerformance.coreWebVitals?.cls?.value || 0), 0) / pagesWithPerformance.length;
    const avgTTFB = pagesWithPerformance.reduce((sum: number, page: any) => sum + (page.enhancedPerformance.metrics?.ttfb?.value || 0), 0) / pagesWithPerformance.length;
    
    const getScoreClass = (score: number) => {
      if (score >= 90) return 'excellent';
      if (score >= 75) return 'good';
      if (score >= 50) return 'needs-improvement';
      return 'poor';
    };
    
    const getGrade = (score: number) => {
      if (score >= 90) return 'A';
      if (score >= 80) return 'B';
      if (score >= 70) return 'C';
      if (score >= 60) return 'D';
      return 'F';
    };

    let html = '<div class="performance-overview">';
    
    // Performance metrics grid
    html += '<div class="metrics-grid">';
    html += `<div class="metric-card ${getScoreClass(avgScore)}">`;
    html += '<div class="metric-label">Performance Score</div>';
    html += `<div class="metric-value">${Math.round(avgScore)}</div>`;
    html += `<div class="metric-grade ${getGrade(avgScore)}">${getGrade(avgScore)}</div>`;
    html += '</div>';
    
    html += `<div class="metric-card ${avgLCP <= 2500 ? 'good' : (avgLCP <= 4000 ? 'needs-improvement' : 'poor')}">` ;
    html += '<div class="metric-label">LCP (Avg)</div>';
    html += `<div class="metric-value">${Math.round(avgLCP)}ms</div>`;
    html += `<div class="metric-grade">${avgLCP <= 2500 ? 'Good' : (avgLCP <= 4000 ? 'Needs Work' : 'Poor')}</div>`;
    html += '</div>';
    
    html += `<div class="metric-card ${avgCLS <= 0.1 ? 'good' : (avgCLS <= 0.25 ? 'needs-improvement' : 'poor')}">` ;
    html += '<div class="metric-label">CLS (Avg)</div>';
    html += `<div class="metric-value">${avgCLS.toFixed(3)}</div>`;
    html += `<div class="metric-grade">${avgCLS <= 0.1 ? 'Good' : (avgCLS <= 0.25 ? 'Needs Work' : 'Poor')}</div>`;
    html += '</div>';
    
    html += `<div class="metric-card ${avgTTFB <= 600 ? 'good' : (avgTTFB <= 1000 ? 'needs-improvement' : 'poor')}">` ;
    html += '<div class="metric-label">TTFB (Avg)</div>';
    html += `<div class="metric-value">${Math.round(avgTTFB)}ms</div>`;
    html += `<div class="metric-grade">${avgTTFB <= 600 ? 'Good' : (avgTTFB <= 1000 ? 'Needs Work' : 'Poor')}</div>`;
    html += '</div>';
    html += '</div>'; // End metrics-grid
    
    // Content Weight Breakdown (if available)
    const pagesWithContentWeight = data.pages.filter((page: any) => page.contentWeight);
    if (pagesWithContentWeight.length > 0) {
      const avgContentWeight = pagesWithContentWeight.reduce((acc: any, page: any) => {
        const cw = page.contentWeight;
        if (!cw) return acc;
        return {
          totalSize: acc.totalSize + (cw.totalSize || 0),
          htmlSize: acc.htmlSize + (cw.resourceSizes?.html || 0),
          cssSize: acc.cssSize + (cw.resourceSizes?.css || 0),
          jsSize: acc.jsSize + (cw.resourceSizes?.javascript || 0),
          imageSize: acc.imageSize + (cw.resourceSizes?.images || 0),
          count: acc.count + 1
        };
      }, { totalSize: 0, htmlSize: 0, cssSize: 0, jsSize: 0, imageSize: 0, count: 0 });
      
      if (avgContentWeight.count > 0) {
        html += '<div class="content-breakdown">';
        html += '<div class="breakdown-title">Content Weight Analysis</div>';
        html += '<div class="breakdown-grid">';
        
        const formatSize = (size: number) => size > 1024 ? `${(size/1024).toFixed(1)}KB` : `${size}B`;
        
        html += '<div class="breakdown-item">';
        html += '<div class="breakdown-type">Total</div>';
        html += `<div class="breakdown-size">${formatSize(avgContentWeight.totalSize / avgContentWeight.count)}</div>`;
        html += '<div class="breakdown-percentage">100%</div>';
        html += '</div>';
        
        if (avgContentWeight.htmlSize > 0) {
          html += '<div class="breakdown-item">';
          html += '<div class="breakdown-type">HTML</div>';
          html += `<div class="breakdown-size">${formatSize(avgContentWeight.htmlSize / avgContentWeight.count)}</div>`;
          html += `<div class="breakdown-percentage">${((avgContentWeight.htmlSize / avgContentWeight.totalSize) * 100).toFixed(1)}%</div>`;
          html += '</div>';
        }
        
        if (avgContentWeight.cssSize > 0) {
          html += '<div class="breakdown-item">';
          html += '<div class="breakdown-type">CSS</div>';
          html += `<div class="breakdown-size">${formatSize(avgContentWeight.cssSize / avgContentWeight.count)}</div>`;
          html += `<div class="breakdown-percentage">${((avgContentWeight.cssSize / avgContentWeight.totalSize) * 100).toFixed(1)}%</div>`;
          html += '</div>';
        }
        
        if (avgContentWeight.jsSize > 0) {
          html += '<div class="breakdown-item">';
          html += '<div class="breakdown-type">JS</div>';
          html += `<div class="breakdown-size">${formatSize(avgContentWeight.jsSize / avgContentWeight.count)}</div>`;
          html += `<div class="breakdown-percentage">${((avgContentWeight.jsSize / avgContentWeight.totalSize) * 100).toFixed(1)}%</div>`;
          html += '</div>';
        }
        
        if (avgContentWeight.imageSize > 0) {
          html += '<div class="breakdown-item">';
          html += '<div class="breakdown-type">Images</div>';
          html += `<div class="breakdown-size">${formatSize(avgContentWeight.imageSize / avgContentWeight.count)}</div>`;
          html += `<div class="breakdown-percentage">${((avgContentWeight.imageSize / avgContentWeight.totalSize) * 100).toFixed(1)}%</div>`;
          html += '</div>';
        }
        
        html += '</div></div>'; // End breakdown-grid and content-breakdown
      }
    }
    
    // Core Web Vitals Table
    html += '<div class="analysis-section">';
    html += '<div class="analysis-title">Core Web Vitals by Page</div>';
    html += '<table class="vitals-table">';
    html += '<thead><tr><th>Page</th><th>Score</th><th>Grade</th><th>LCP</th><th>CLS</th><th>INP</th><th>TTFB</th></tr></thead>';
    html += '<tbody>';
    
    data.pages.forEach((page: any) => {
      const perf = page.enhancedPerformance;
      if (!perf) return;
      
      const vitals = perf.coreWebVitals || {};
      const metrics = perf.metrics || {};
      const pageName = this.getPageName(page.url);
      
      html += '<tr>';
      html += `<td>${pageName}</td>`;
      html += `<td class="score-cell score-${getScoreClass(perf.score || 0)}">${Math.round(perf.score || 0)}</td>`;
      html += `<td><span class="grade-badge ${getGrade(perf.score || 0)}">${getGrade(perf.score || 0)}</span></td>`;
      html += `<td>${Math.round(vitals.lcp?.value || 0)}ms</td>`;
      html += `<td>${(vitals.cls?.value || 0).toFixed(3)}</td>`;
      html += `<td>${Math.round(vitals.inp?.value || 0)}ms</td>`;
      html += `<td>${Math.round(metrics.ttfb?.value || 0)}ms</td>`;
      html += '</tr>';
    });
    
    html += '</tbody></table>';
    html += '</div>'; // End analysis-section
    
    // Performance Recommendations
    const allRecommendations: any[] = [];
    data.pages.forEach((page: any) => {
      if (page.enhancedPerformance && page.enhancedPerformance.recommendations) {
        allRecommendations.push(...page.enhancedPerformance.recommendations);
      }
    });
    
    if (allRecommendations.length > 0) {
      const uniqueRecommendations = allRecommendations
        .filter((rec, index, self) => self.findIndex(r => r.description === rec.description) === index)
        .slice(0, 5); // Limit to top 5
      
      html += '<div class="recommendations-section">';
      html += '<div class="recommendations-title">Performance Recommendations</div>';
      html += '<ul class="recommendations-list">';
      
      uniqueRecommendations.forEach((rec: any) => {
        html += '<li class="recommendation-item">';
        html += `<div class="recommendation-priority ${rec.priority}">${rec.priority?.toUpperCase()}</div>`;
        html += `<div class="recommendation-text">${rec.description}</div>`;
        html += '</li>';
      });
      
      html += '</ul></div>'; // End recommendations-section
    }
    
    html += '</div>'; // End performance-overview
    
    return html;
  }

  generateEnhancedSeoHtml(data: any): string {
    const pagesWithSeo = data.pages.filter((page: any) => page.enhancedSEO || page.enhancedSeo);
    if (pagesWithSeo.length === 0) {
      return this.generateSeoSection(data);
    }

    const avgSeoScore = pagesWithSeo.reduce((sum: number, page: any) => {
      const seoData = page.enhancedSEO || page.enhancedSeo;
      return sum + (seoData.score || 0);
    }, 0) / pagesWithSeo.length;
    
    const getScoreClass = (score: number) => {
      if (score >= 90) return 'excellent';
      if (score >= 75) return 'good';
      if (score >= 50) return 'needs-improvement';
      return 'poor';
    };
    
    const getGrade = (score: number) => {
      if (score >= 90) return 'A';
      if (score >= 80) return 'B';
      if (score >= 70) return 'C';
      if (score >= 60) return 'D';
      return 'F';
    };

    let html = '<div class="seo-overview">';
    
    // SEO Overview metrics
    html += '<div class="metrics-grid">';
    html += `<div class="metric-card ${getScoreClass(avgSeoScore)}">`;
    html += '<div class="metric-label">SEO Score</div>';
    html += `<div class="metric-value">${Math.round(avgSeoScore)}</div>`;
    html += `<div class="metric-grade ${getGrade(avgSeoScore)}">${getGrade(avgSeoScore)}</div>`;
    html += '</div>';
    
    // Count pages with essential SEO elements  
    const pagesWithTitle = pagesWithSeo.filter((page: any) => {
      const seoData = page.enhancedSEO || page.enhancedSeo;
      return seoData.metaData?.title && seoData.metaData.title.length > 0;
    }).length;
    const pagesWithDescription = pagesWithSeo.filter((page: any) => {
      const seoData = page.enhancedSEO || page.enhancedSeo;
      return seoData.metaData?.description && seoData.metaData.description.length > 0;
    }).length;
    const pagesWithOg = pagesWithSeo.filter((page: any) => {
      const seoData = page.enhancedSEO || page.enhancedSeo;
      return seoData.socialTags?.openGraph && seoData.socialTags.openGraph > 0;
    }).length;
    const httpsPages = pagesWithSeo.filter((page: any) => {
      const seoData = page.enhancedSEO || page.enhancedSeo;
      return seoData.technicalSEO && seoData.technicalSEO.internalLinks >= 0;
    }).length;
    
    html += `<div class="metric-card ${pagesWithTitle === pagesWithSeo.length ? 'excellent' : 'needs-improvement'}">`;
    html += '<div class="metric-label">Pages with Title</div>';
    html += `<div class="metric-value">${pagesWithTitle}/${pagesWithSeo.length}</div>`;
    html += `<div class="metric-grade">${((pagesWithTitle / pagesWithSeo.length) * 100).toFixed(0)}%</div>`;
    html += '</div>';
    
    html += `<div class="metric-card ${pagesWithDescription === pagesWithSeo.length ? 'excellent' : 'needs-improvement'}">`;
    html += '<div class="metric-label">Pages with Description</div>';
    html += `<div class="metric-value">${pagesWithDescription}/${pagesWithSeo.length}</div>`;
    html += `<div class="metric-grade">${((pagesWithDescription / pagesWithSeo.length) * 100).toFixed(0)}%</div>`;
    html += '</div>';
    
    html += `<div class="metric-card ${httpsPages === pagesWithSeo.length ? 'excellent' : 'poor'}">`;
    html += '<div class="metric-label">HTTPS</div>';
    html += `<div class="metric-value">${httpsPages}/${pagesWithSeo.length}</div>`;
    html += `<div class="metric-grade">${httpsPages === pagesWithSeo.length ? 'Secure' : 'Insecure'}</div>`;
    html += '</div>';
    
    html += '</div>'; // End metrics-grid
    
    // Meta Tags Analysis Table
    html += '<div class="analysis-section">';
    html += '<div class="analysis-title">Meta Tags Analysis</div>';
    html += '<table class="meta-tags-table">';
    html += '<thead><tr><th>Page</th><th>Title</th><th>Description</th><th>Keywords</th><th>Open Graph</th><th>Twitter Card</th></tr></thead>';
    html += '<tbody>';
    
    data.pages.forEach((page: any) => {
      const seo = page.enhancedSEO || page.enhancedSeo;
      if (!seo) return;
      
      const pageName = this.getPageName(page.url);
      const meta = seo.metaData || {};
      const social = seo.socialTags || {};
      
      html += '<tr>';
      html += `<td>${pageName}</td>`;
      html += `<td>${meta.title ? '✅' : '❌'} ${meta.title ? (meta.title.length > 60 ? '⚠️' : '') : ''}</td>`;
      html += `<td>${meta.description ? '✅' : '❌'} ${meta.description ? (meta.description.length > 160 ? '⚠️' : '') : ''}</td>`;
      html += `<td>${meta.keywords ? '✅' : '❌'}</td>`;
      html += `<td>${social.openGraph && social.openGraph > 0 ? '✅' : '❌'}</td>`;
      html += `<td>${social.twitterCard && social.twitterCard > 0 ? '✅' : '❌'}</td>`;
      html += '</tr>';
    });
    
    html += '</tbody></table>';
    html += '</div>'; // End analysis-section
    
    // Content Analysis
    html += '<div class="analysis-section">';
    html += '<div class="analysis-title">Content Quality Analysis</div>';
    html += '<table class="content-analysis-table">';
    html += '<thead><tr><th>Page</th><th>Word Count</th><th>Readability</th><th>Heading Structure</th><th>Content Score</th></tr></thead>';
    html += '<tbody>';
    
    data.pages.forEach((page: any) => {
      const seo = page.enhancedSEO || page.enhancedSeo;
      if (!seo || !seo.contentAnalysis) return;
      
      const pageName = this.getPageName(page.url);
      const content = seo.contentAnalysis;
      const headingStructure = seo.headingStructure || {};
      
      html += '<tr>';
      html += `<td>${pageName}</td>`;
      html += `<td>${content.wordCount || 0}</td>`;
      html += `<td>${Math.round(content.readabilityScore || 0)}%</td>`;
      html += `<td>${headingStructure.h1 || 0}h1, ${(headingStructure.h1 + headingStructure.h2 + headingStructure.h3 + headingStructure.h4 + headingStructure.h5 + headingStructure.h6) || 0} total</td>`;
      html += `<td class="score-cell score-${getScoreClass(seo.seoScore || 0)}">${Math.round(seo.seoScore || 0)}</td>`;
      html += '</tr>';
    });
    
    html += '</tbody></table>';
    html += '</div>'; // End analysis-section
    
    // Technical SEO
    html += '<div class="analysis-section">';
    html += '<div class="analysis-title">Technical SEO</div>';
    html += '<table class="technical-seo-table">';
    html += '<thead><tr><th>Page</th><th>HTTPS</th><th>Mobile Friendly</th><th>Schema Markup</th><th>Canonical URL</th></tr></thead>';
    html += '<tbody>';
    
    data.pages.forEach((page: any) => {
      const seo = page.enhancedSEO || page.enhancedSeo;
      if (!seo) return;
      
      const pageName = this.getPageName(page.url);
      const tech = seo.technicalSEO || {};
      
      html += '<tr>';
      html += `<td>${pageName}</td>`;
      // HTTPS Status from technical SEO data
      html += `<td>${tech.httpsEnabled ? '✅ Yes' : '❌ No'}</td>`;
      // Mobile Friendly from technical SEO data  
      html += `<td>${tech.mobileFriendly ? '✅ Yes' : '❌ No'}</td>`;
      // Schema Markup from technical SEO data
      const schemaCount = tech.schemaMarkup?.length || 0;
      html += `<td>${schemaCount > 0 ? `✅ ${schemaCount} found` : '❌ None'}</td>`;
      // Canonical URL (check if canonical URL is present in meta tags)
      const canonical = seo.metaTags?.canonical || seo.metaData?.canonical;
      html += `<td>${canonical?.present ? '✅ Present' : '❌ Missing'}</td>`;
      html += '</tr>';
    });
    
    html += '</tbody></table>';
    html += '</div>'; // End analysis-section
    
    // SEO Recommendations
    const allSeoRecommendations: any[] = [];
    data.pages.forEach((page: any) => {
      if (page.enhancedSeo && page.enhancedSeo.recommendations) {
        allSeoRecommendations.push(...page.enhancedSeo.recommendations);
      }
    });
    
    if (allSeoRecommendations.length > 0) {
      const uniqueSeoRecommendations = allSeoRecommendations
        .filter((rec, index, self) => self.findIndex(r => r.description === rec.description) === index)
        .slice(0, 5); // Limit to top 5
      
      html += '<div class="recommendations-section">';
      html += '<div class="recommendations-title">SEO Recommendations</div>';
      html += '<ul class="recommendations-list">';
      
      uniqueSeoRecommendations.forEach((rec: any) => {
        html += '<li class="recommendation-item">';
        html += `<div class="recommendation-priority ${rec.priority || 'medium'}">${(rec.priority || 'medium').toUpperCase()}</div>`;
        html += `<div class="recommendation-text">${rec.description}</div>`;
        html += '</li>';
      });
      
      html += '</ul></div>'; // End recommendations-section
    }
    
    html += '</div>'; // End seo-overview
    
    return html;
  }

  private generateBasicPerformanceSection(data: any): string {
    const formatMetric = (value: any): string => {
      if (value === null || value === undefined || value === 'N/A') return 'N/A';
      const numValue = parseFloat(value);
      return isNaN(numValue) ? 'N/A' : `${Math.round(numValue)}ms`;
    };

    return `<div class="table-container">
      <div class="table-wrapper">
        <table class="data-table">
        <thead><tr><th>Page</th><th>Load Time</th><th>FCP</th><th>LCP</th><th>DOM Loaded</th><th>First Paint</th></tr></thead>
        <tbody>
          ${data.pages.map((page: any) => {
            const perf = page.issues?.performanceMetrics || {};
            return `<tr>
              <td>${page.url}</td>
              <td>${formatMetric(perf.loadTime)}</td>
              <td>${formatMetric(perf.firstContentfulPaint)}</td>
              <td>${formatMetric(perf.largestContentfulPaint)}</td>
              <td>${formatMetric(perf.domContentLoaded)}</td>
              <td>${formatMetric(perf.firstPaint)}</td>
            </tr>`;
          }).join('')}
        </tbody>
        </table>
      </div>
    </div>`;
  }

  private generateBasicSeoSection(data: any): string {
    return `<div class="table-container">
      <div class="table-wrapper">
        <table class="data-table">
        <thead><tr><th>Page & Title</th><th>Headings</th><th>Images w/o Alt</th><th>Buttons w/o Label</th></tr></thead>
        <tbody>
          ${data.pages.map((page: any) => {
            const pageTitle = page.title || 'No title';
            const pageName = this.getPageName(page.url);
            return `<tr>
              <td>
                <div class="page-info">
                  <strong>${pageName}</strong><br>
                  <small class="page-title">${pageTitle}</small><br>
                  <small class="page-url">${page.url}</small>
                </div>
              </td>
              <td>${page.issues?.headingsCount ?? 'N/A'}</td>
              <td>${page.issues?.imagesWithoutAlt ?? 'N/A'}</td>
              <td>${page.issues?.buttonsWithoutLabel ?? 'N/A'}</td>
            </tr>`;
          }).join('')}
        </tbody>
        </table>
      </div>
    </div>`;
  }



  generateDetailedIssuesSection(data: any): string {
    if (!data.pages || data.pages.length === 0) {
      return `<div class="no-data">
        <h3>No Detailed Issues Available</h3>
        <p>No detailed issues were found or collected during the audit.</p>
      </div>`;
    }

    // Collect all issues from all pages
    const allIssues: any[] = [];
    
    data.pages.forEach((page: any) => {
      // Add pa11y issues
      if (page.issues?.pa11yIssues && Array.isArray(page.issues.pa11yIssues)) {
        page.issues.pa11yIssues.forEach((issue: any) => {
          allIssues.push({
            ...issue,
            pageUrl: page.url,
            pageTitle: page.title,
            source: 'pa11y',
            category: this.getIssueCategory(issue.code || issue.type)
          });
        });
      }
      
      // Add error details as issues
      if (page.errorDetails && Array.isArray(page.errorDetails)) {
        page.errorDetails.forEach((error: string) => {
          allIssues.push({
            type: 'error',
            severity: 'error',
            message: error,
            pageUrl: page.url,
            pageTitle: page.title,
            source: 'playwright',
            category: 'General Accessibility'
          });
        });
      }
      
      // Add warning details as issues
      if (page.warningDetails && Array.isArray(page.warningDetails)) {
        page.warningDetails.forEach((warning: string) => {
          allIssues.push({
            type: 'warning',
            severity: 'warning', 
            message: warning,
            pageUrl: page.url,
            pageTitle: page.title,
            source: 'playwright',
            category: this.getWarningCategory(warning)
          });
        });
      }
    });

    if (allIssues.length === 0) {
      return `<div class="no-issues">
        <h3>🎉 Great! No Issues Found</h3>
        <p>Your website appears to be free of accessibility issues.</p>
      </div>`;
    }

    // Group issues by category
    const groupedIssues = this.groupIssuesByCategory(allIssues);
    
    let html = '<div class="detailed-issues-container">';
    
    // Summary stats
    html += this.generateIssuesSummary(allIssues);
    
    // Generate sections for each category
    Object.entries(groupedIssues).forEach(([category, issues]) => {
      html += this.generateCategorySection(category, issues as any[]);
    });
    
    html += '</div>';
    
    return html;
  }
  
  private generateIssuesSummary(issues: any[]): string {
    const errorCount = issues.filter(i => i.severity === 'error').length;
    const warningCount = issues.filter(i => i.severity === 'warning').length;
    const noticeCount = issues.filter(i => i.severity === 'notice' || i.type === 'notice').length;
    
    return `<div class="issues-summary">
      <h3>Issues Overview</h3>
      <div class="summary-grid">
        <div class="summary-card error">
          <div class="summary-number">${errorCount}</div>
          <div class="summary-label">Errors</div>
        </div>
        <div class="summary-card warning">
          <div class="summary-number">${warningCount}</div>
          <div class="summary-label">Warnings</div>
        </div>
        <div class="summary-card notice">
          <div class="summary-number">${noticeCount}</div>
          <div class="summary-label">Notices</div>
        </div>
        <div class="summary-card total">
          <div class="summary-number">${issues.length}</div>
          <div class="summary-label">Total Issues</div>
        </div>
      </div>
    </div>`;
  }
  
  private generateCategorySection(category: string, issues: any[]): string {
    const sectionId = category.toLowerCase().replace(/[^a-z0-9]/g, '-');
    
    let html = `<div class="category-section" id="category-${sectionId}">`;
    html += `<div class="category-header">`;
    html += `<h3 class="category-title">${category} <span class="issue-count">(${issues.length})</span></h3>`;
    html += `<button class="copy-category-btn" onclick="copyCategoryIssues('${sectionId}')">📋 Copy All Issues</button>`;
    html += `</div>`;
    
    // Group by page within category
    const pageGroups = this.groupIssuesByPage(issues);
    
    Object.entries(pageGroups).forEach(([pageUrl, pageIssues]) => {
      const pageTitle = (pageIssues as any[])[0]?.pageTitle || 'Unknown Page';
      html += this.generatePageIssuesSection(pageUrl, pageTitle, pageIssues as any[]);
    });
    
    html += '</div>';
    
    return html;
  }
  
  private generatePageIssuesSection(pageUrl: string, pageTitle: string, issues: any[]): string {
    let html = `<div class="page-issues-section">`;
    html += `<div class="page-header">`;
    html += `<h4 class="page-title">${this.getPageName(pageUrl)}</h4>`;
    html += `<div class="page-meta">`;
    html += `<span class="page-url">${pageUrl}</span>`;
    html += `<span class="page-issue-count">${issues.length} issues</span>`;
    html += `</div>`;
    html += `</div>`;
    
    html += `<div class="issues-list">`;
    
    issues.forEach((issue, index) => {
      html += this.generateIssueItem(issue, index);
    });
    
    html += `</div></div>`;
    
    return html;
  }
  
  private generateIssueItem(issue: any, index: number): string {
    const severityClass = issue.severity || 'info';
    const issueId = `issue-${Date.now()}-${index}`;
    
    let html = `<div class="issue-item ${severityClass}" id="${issueId}">`;
    html += `<div class="issue-header">`;
    html += `<div class="issue-type-badge ${severityClass}">${issue.severity?.toUpperCase() || 'INFO'}</div>`;
    html += `<div class="issue-source">${issue.source || 'unknown'}</div>`;
    html += `<button class="copy-issue-btn" onclick="copyIssue('${issueId}')">📋</button>`;
    html += `</div>`;
    
    html += `<div class="issue-content">`;
    html += `<div class="issue-message">${issue.message}</div>`;
    
    if (issue.code) {
      html += `<div class="issue-code"><strong>Code:</strong> ${issue.code}</div>`;
    }
    
    if (issue.selector) {
      html += `<div class="issue-selector"><strong>Element:</strong> <code>${issue.selector}</code></div>`;
    }
    
    if (issue.context) {
      html += `<div class="issue-context"><strong>Context:</strong> <code>${this.escapeHtml(issue.context)}</code></div>`;
    }
    
    if (issue.help) {
      html += `<div class="issue-help"><strong>Help:</strong> ${issue.help}</div>`;
    }
    
    if (issue.helpUrl) {
      html += `<div class="issue-help-url"><strong>More Info:</strong> <a href="${issue.helpUrl}" target="_blank">${issue.helpUrl}</a></div>`;
    }
    
    // AI-friendly format for copying
    html += `<div class="issue-ai-format" style="display: none;">`;
    html += `Issue: ${issue.message}\n`;
    if (issue.selector) html += `Element: ${issue.selector}\n`;
    if (issue.context) html += `HTML Context: ${issue.context}\n`;
    if (issue.code) html += `Rule: ${issue.code}\n`;
    html += `Page: ${issue.pageUrl}\n`;
    html += `Severity: ${issue.severity}\n`;
    if (issue.help) html += `Fix: ${issue.help}\n`;
    html += `---\n`;
    html += `</div>`;
    
    html += `</div></div>`;
    
    return html;
  }
  
  private groupIssuesByCategory(issues: any[]): Record<string, any[]> {
    const groups: Record<string, any[]> = {};
    
    issues.forEach(issue => {
      const category = issue.category || 'General';
      if (!groups[category]) {
        groups[category] = [];
      }
      groups[category].push(issue);
    });
    
    return groups;
  }
  
  private groupIssuesByPage(issues: any[]): Record<string, any[]> {
    const groups: Record<string, any[]> = {};
    
    issues.forEach(issue => {
      const pageUrl = issue.pageUrl || 'unknown';
      if (!groups[pageUrl]) {
        groups[pageUrl] = [];
      }
      groups[pageUrl].push(issue);
    });
    
    return groups;
  }
  
  private getIssueCategory(codeOrType?: string): string {
    if (!codeOrType) return 'General';
    
    const code = codeOrType.toLowerCase();
    
    if (code.includes('color') || code.includes('contrast')) return 'Color & Contrast';
    if (code.includes('aria') || code.includes('role')) return 'ARIA & Semantics';
    if (code.includes('form') || code.includes('label')) return 'Forms & Labels';
    if (code.includes('image') || code.includes('alt')) return 'Images & Media';
    if (code.includes('heading') || code.includes('structure')) return 'Document Structure';
    if (code.includes('keyboard') || code.includes('focus')) return 'Keyboard & Focus';
    if (code.includes('link') || code.includes('anchor')) return 'Links & Navigation';
    if (code.includes('table')) return 'Tables';
    if (code.includes('landmark')) return 'Page Landmarks';
    
    return 'General Accessibility';
  }
  
  private getWarningCategory(warning: string): string {
    const w = warning.toLowerCase();
    
    if (w.includes('image') && w.includes('alt')) return 'Images & Media';
    if (w.includes('button') && w.includes('label')) return 'Forms & Labels';
    if (w.includes('performance') || w.includes('load')) return 'Performance';
    if (w.includes('contrast')) return 'Color & Contrast';
    if (w.includes('keyboard')) return 'Keyboard & Focus';
    if (w.includes('focus')) return 'Keyboard & Focus';
    
    return 'General';
  }
  
  private escapeHtml(unsafe: string): string {
    return unsafe
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;")
      .replace(/'/g, "&#039;");
  }

  generateSecuritySection(data: any): string {
    return `<section id="security">
      <h2>Security Report</h2>
      <p>Security-Daten sind aktuell nicht integriert.</p>
    </section>`;
  }

  generateMobileFriendlinessSection(data: any): string {
    // Check if we have mobile-friendliness data
    const pagesWithMobile = data.pages.filter((page: any) => page.mobileFriendliness);
    
    if (pagesWithMobile.length === 0) {
      return `<div class="no-data">
        <h3>📱 Mobile-Friendliness Data Not Available</h3>
        <p>Mobile-friendliness analysis requires enhanced analysis mode. Run with enhanced features to get detailed mobile usability insights including touch targets, viewport configuration, and responsive design analysis.</p>
      </div>`;
    }

    const avgMobileScore = pagesWithMobile.reduce((sum: number, page: any) => {
      return sum + (page.mobileFriendliness.overallScore || page.mobileFriendliness.score || 0);
    }, 0) / pagesWithMobile.length;
    
    const getScoreClass = (score: number) => {
      if (score >= 90) return 'excellent';
      if (score >= 75) return 'good';
      if (score >= 50) return 'needs-improvement';
      return 'poor';
    };
    
    const getGrade = (score: number) => {
      if (score >= 90) return 'A';
      if (score >= 80) return 'B';
      if (score >= 70) return 'C';
      if (score >= 60) return 'D';
      return 'F';
    };

    let html = '<div class="mobile-friendliness-overview">';
    
    // Mobile-Friendliness Overview metrics
    html += '<div class="metrics-grid">';
    html += `<div class="metric-card ${getScoreClass(avgMobileScore)}">`;
    html += '<div class="metric-label">Mobile Score</div>';
    html += `<div class="metric-value">${Math.round(avgMobileScore)}</div>`;
    html += `<div class="metric-grade ${getGrade(avgMobileScore)}">${getGrade(avgMobileScore)}</div>`;
    html += '</div>';
    
    // Calculate mobile metrics averages if available
    if (pagesWithMobile.some((page: any) => page.mobileFriendliness.scores)) {
      const avgViewportScore = pagesWithMobile.reduce((sum: number, page: any) => {
        return sum + (page.mobileFriendliness.scores?.viewport || 0);
      }, 0) / pagesWithMobile.length;
      
      const avgTouchScore = pagesWithMobile.reduce((sum: number, page: any) => {
        return sum + (page.mobileFriendliness.scores?.touchTargets || 0);
      }, 0) / pagesWithMobile.length;
      
      const avgTypographyScore = pagesWithMobile.reduce((sum: number, page: any) => {
        return sum + (page.mobileFriendliness.scores?.typography || 0);
      }, 0) / pagesWithMobile.length;
      
      const avgNavigationScore = pagesWithMobile.reduce((sum: number, page: any) => {
        return sum + (page.mobileFriendliness.scores?.navigation || 0);
      }, 0) / pagesWithMobile.length;
      
      html += `<div class="metric-card ${getScoreClass(avgViewportScore)}">`;
      html += '<div class="metric-label">Viewport & Layout</div>';
      html += `<div class="metric-value">${Math.round(avgViewportScore)}</div>`;
      html += `<div class="metric-grade">${getGrade(avgViewportScore)}</div>`;
      html += '</div>';
      
      html += `<div class="metric-card ${getScoreClass(avgTouchScore)}">`;
      html += '<div class="metric-label">Touch Targets</div>';
      html += `<div class="metric-value">${Math.round(avgTouchScore)}</div>`;
      html += `<div class="metric-grade">${getGrade(avgTouchScore)}</div>`;
      html += '</div>';
      
      html += `<div class="metric-card ${getScoreClass(avgTypographyScore)}">`;
      html += '<div class="metric-label">Typography</div>';
      html += `<div class="metric-value">${Math.round(avgTypographyScore)}</div>`;
      html += `<div class="metric-grade">${getGrade(avgTypographyScore)}</div>`;
      html += '</div>';
    }
    
    html += '</div>'; // End metrics-grid
    
    // Mobile-Friendliness Analysis Table
    html += '<div class="analysis-section">';
    html += '<div class="analysis-title">Mobile Usability by Page</div>';
    html += '<table class="mobile-analysis-table">';
    html += '<thead><tr><th>Page</th><th>Overall Score</th><th>Grade</th><th>Issues</th></tr></thead>';
    html += '<tbody>';
    
    data.pages.forEach((page: any) => {
      const mobile = page.mobileFriendliness;
      if (!mobile) return;
      
      const pageName = this.getPageName(page.url);
      const score = mobile.overallScore || mobile.score || 0;
      const issues = mobile.issues?.length || 0;
      
      html += '<tr>';
      html += `<td>${pageName}</td>`;
      html += `<td class="score-cell score-${getScoreClass(score)}">${Math.round(score)}</td>`;
      html += `<td><span class="grade-badge ${getGrade(score)}">${getGrade(score)}</span></td>`;
      html += `<td>${issues}</td>`;
      html += '</tr>';
    });
    
    html += '</tbody></table>';
    html += '</div>'; // End analysis-section
    
    // Desktop vs Mobile Comparison (if available)
    const pagesWithComparison = data.pages.filter((page: any) => page.mobileFriendliness?.desktopComparison);
    if (pagesWithComparison.length > 0) {
      html += '<div class="comparison-section">';
      html += '<div class="comparison-title">🖥️📱 Desktop vs Mobile Comparison</div>';
      
      pagesWithComparison.forEach((page: any) => {
        const comparison = page.mobileFriendliness.desktopComparison;
        const pageName = this.getPageName(page.url);
        
        html += '<div class="comparison-analysis">';
        html += `<h4>${pageName}</h4>`;
        
        // Comparison metrics table
        html += '<table class="comparison-table">';
        html += '<thead><tr><th>Metric</th><th>🖥️ Desktop</th><th>📱 Mobile</th><th>Impact</th></tr></thead>';
        html += '<tbody>';
        
        // Usability Score
        html += '<tr>';
        html += '<td><strong>Usability Score</strong></td>';
        html += `<td class="score-cell score-${this.getScoreClass(comparison.desktop.usabilityScore)}">${comparison.desktop.usabilityScore}</td>`;
        html += `<td class="score-cell score-${this.getScoreClass(comparison.mobile.usabilityScore)}">${comparison.mobile.usabilityScore}</td>`;
        html += `<td>${Math.abs(comparison.differences.usabilityGap)} point difference</td>`;
        html += '</tr>';
        
        // Touch Targets
        html += '<tr>';
        html += '<td>Touch/Click Targets</td>';
        html += `<td>${comparison.desktop.touchTargets.averageSize.toFixed(1)}px avg</td>`;
        html += `<td>${comparison.mobile.touchTargets.averageSize.toFixed(1)}px avg</td>`;
        html += `<td>${comparison.mobile.touchTargets.averageSize < 48 ? '⚠️ Too small for mobile' : '✅ Adequate'}</td>`;
        html += '</tr>';
        
        // Typography
        html += '<tr>';
        html += '<td>Font Size</td>';
        html += `<td>${comparison.desktop.typography.baseFontSize}px</td>`;
        html += `<td>${comparison.mobile.typography.baseFontSize}px</td>`;
        html += `<td>${comparison.mobile.typography.baseFontSize < 16 ? '⚠️ Too small for mobile' : '✅ Mobile-friendly'}</td>`;
        html += '</tr>';
        
        // Performance
        html += '<tr>';
        html += '<td>LCP Performance</td>';
        html += `<td>${Math.round(comparison.desktop.performance.lcp)}ms</td>`;
        html += `<td>${Math.round(comparison.mobile.performance.lcp)}ms</td>`;
        html += `<td>${comparison.differences.performanceImpact > 500 ? '⚠️ Mobile slower' : '✅ Similar performance'}</td>`;
        html += '</tr>';
        
        html += '</tbody></table>';
        
        // Critical Issues
        if (comparison.differences.criticalIssues.length > 0) {
          html += '<div class="critical-issues">';
          html += '<h5>🚨 Critical Issues</h5>';
          html += '<ul>';
          comparison.differences.criticalIssues.forEach((issue: string) => {
            html += `<li class="critical-issue">${issue}</li>`;
          });
          html += '</ul>';
          html += '</div>';
        }
        
        // Comparison Recommendations
        if (comparison.recommendations && comparison.recommendations.length > 0) {
          html += '<div class="comparison-recommendations">';
          html += '<h5>💡 Desktop vs Mobile Recommendations</h5>';
          html += '<div class="recommendations-grid">';
          
          comparison.recommendations.slice(0, 3).forEach((rec: any) => {
            html += '<div class="comparison-rec-card">';
            html += `<div class="rec-priority priority-${rec.priority}">${rec.priority.toUpperCase()}</div>`;
            html += `<div class="rec-issue">${rec.issue}</div>`;
            html += `<div class="rec-mobile"><strong>📱 Mobile:</strong> ${rec.mobileRecommendation}</div>`;
            html += `<div class="rec-desktop"><strong>🖥️ Desktop:</strong> ${rec.desktopRecommendation}</div>`;
            html += `<div class="rec-impact"><em>${rec.impact}</em></div>`;
            html += '</div>';
          });
          
          html += '</div></div>'; // End comparison-recommendations
        }
        
        html += '</div>'; // End comparison-analysis
      });
      
      html += '</div>'; // End comparison-section
    }
    
    // Mobile-Friendliness Recommendations
    const allMobileRecommendations: any[] = [];
    data.pages.forEach((page: any) => {
      if (page.mobileFriendliness && page.mobileFriendliness.recommendations) {
        allMobileRecommendations.push(...page.mobileFriendliness.recommendations);
      }
    });
    
    if (allMobileRecommendations.length > 0) {
      const uniqueMobileRecommendations = allMobileRecommendations
        .filter((rec, index, self) => self.findIndex(r => (r.recommendation || r.description) === (rec.recommendation || rec.description)) === index)
        .slice(0, 5); // Limit to top 5
      
      html += '<div class="recommendations-section">';
      html += '<div class="recommendations-title">Mobile-Friendliness Recommendations</div>';
      html += '<ul class="recommendations-list">';
      
      uniqueMobileRecommendations.forEach((rec: any) => {
        html += '<li class="recommendation-item">';
        html += `<div class="recommendation-priority ${rec.priority || 'medium'}">${(rec.priority || 'medium').toUpperCase()}</div>`;
        html += `<div class="recommendation-text">${rec.recommendation || rec.description || 'Mobile-friendliness recommendation'}</div>`;
        html += '</li>';
      });
      
      html += '</ul></div>'; // End recommendations-section
    }
    
    html += '</div>'; // End mobile-friendliness-overview
    
    return html;
  }

}
