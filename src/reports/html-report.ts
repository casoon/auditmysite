import { HtmlGenerator } from '../generators/html-generator';

export function generateHtmlReport(data: any): string {
  const generator = new HtmlGenerator();
  const accessibilitySection = generator.generateAccessibilitySection(data);
  const detailedIssuesSection = generator.generateDetailedIssuesSection(data);
  const performanceSection = generator.generatePerformanceSection(data);
  const seoSection = generator.generateSeoSection(data);
  const securitySection = generator.generateSecuritySection(data);
  const mobileFriendlinessSection = generator.generateMobileFriendlinessSection(data);

  // Extract domain from first page URL if available
  const domain = data.pages && data.pages.length > 0 
    ? new URL(data.pages[0].url).hostname 
    : 'unknown';

  // Calculate metrics
  const successRate = data.summary && data.summary.testedPages > 0 
    ? Math.round((data.summary.passedPages / data.summary.testedPages) * 100) 
    : 0;
  
  const totalPages = data.summary?.totalPages || data.pages?.length || 0;
  const testedPages = data.summary?.testedPages || data.pages?.length || 0;
  
  // Fix: Calculate totalErrors correctly from pages data if summary is missing or 0
  let totalErrors = data.summary?.totalErrors || 0;
  let totalWarnings = data.summary?.totalWarnings || 0;
  
  // If totalErrors is 0 but we have pages with errors, recalculate
  if (totalErrors === 0 && data.pages && Array.isArray(data.pages)) {
    totalErrors = data.pages.reduce((sum: number, page: any) => {
      return sum + (page.errors || 0);
    }, 0);
    console.log(`Debug: Recalculated totalErrors from pages: ${totalErrors}`);
  }
  
  // If totalWarnings is 0 but we have pages with warnings, recalculate  
  if (totalWarnings === 0 && data.pages && Array.isArray(data.pages)) {
    totalWarnings = data.pages.reduce((sum: number, page: any) => {
      return sum + (page.warnings || 0);
    }, 0);
    console.log(`Debug: Recalculated totalWarnings from pages: ${totalWarnings}`);
  }
  
  const totalDuration = data.summary?.totalDuration || 0;

  // Format duration
  const formatDuration = (ms: number): string => {
    if (!ms || ms === 0) return '0ms';
    if (ms < 1000) return `${Math.round(ms)}ms`;
    if (ms < 60000) return `${Math.round(ms / 1000)}s`;
    return `${Math.round(ms / 60000)}min`;
  };
  
  // Fix: Calculate totalDuration correctly if it's 0
  let finalTotalDuration = totalDuration;
  if (finalTotalDuration === 0 && data.pages && Array.isArray(data.pages)) {
    finalTotalDuration = data.pages.reduce((sum: number, page: any) => {
      return sum + (page.loadTime || 0);
    }, 0);
    console.log(`Debug: Recalculated totalDuration from pages: ${finalTotalDuration}ms`);
  }

  // Use the enhanced template from HtmlGenerator
  const htmlTemplate = `<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Accessibility Report - {{domain}}</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
            background: #f8fafc;
            color: #1e293b;
            line-height: 1.6;
        }
        
        .header {
            background: linear-gradient(135deg, #0f172a 0%, #1e293b 100%);
            color: white;
            padding: 2rem 0;
            position: sticky;
            top: 0;
            z-index: 100;
            box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1);
        }
        
        .container {
            max-width: 1200px;
            margin: 0 auto;
            padding: 0 1rem;
        }
        
        .header-content {
            display: flex;
            justify-content: space-between;
            align-items: center;
            flex-wrap: wrap;
            gap: 1rem;
        }
        
        .header h1 {
            font-size: 1.8rem;
            font-weight: 700;
            margin: 0;
        }
        
        .main-content {
            padding: 2rem 0;
        }
        
        .section {
            background: white;
            border-radius: 0.75rem;
            box-shadow: 0 1px 3px 0 rgba(0, 0, 0, 0.1);
            margin-bottom: 2rem;
            overflow: hidden;
        }
        
        .section-header {
            background: #f8fafc;
            border-bottom: 1px solid #e2e8f0;
            padding: 1.5rem;
        }
        
        .section-header h2 {
            font-size: 1.25rem;
            font-weight: 600;
            color: #0f172a;
        }
        
        .section-content {
            padding: 1.5rem;
        }
        
        .stats-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 1rem;
            margin-bottom: 2rem;
        }
        
        .stat-card {
            background: white;
            border: 1px solid #e2e8f0;
            border-radius: 0.5rem;
            padding: 1.5rem;
            text-align: center;
        }
        
        .stat-value {
            font-size: 2rem;
            font-weight: bold;
            color: #0f172a;
            display: block;
        }
        
        .stat-label {
            color: #64748b;
            font-size: 0.875rem;
            margin-top: 0.25rem;
        }
        
        .success { color: #059669; }
        .warning { color: #d97706; }
        .error { color: #dc2626; }
        .info { color: #0284c7; }
        
        .data-table {
            width: 100%;
            border-collapse: collapse;
            margin-top: 1rem;
        }
        
        .data-table th,
        .data-table td {
            padding: 0.75rem;
            text-align: left;
            border-bottom: 1px solid #e2e8f0;
        }
        
        .data-table th {
            background: #f8fafc;
            font-weight: 600;
            color: #374151;
        }
        
        .data-table tr:hover {
            background: #f9fafb;
        }
    </style>
</head>
<body>
    <div class="header">
        <div class="container">
            <div class="header-content">
                <h1>🔍 Accessibility Report - {{domain}}</h1>
                <div>{{timestamp}}</div>
            </div>
        </div>
    </div>
    
    <div class="main-content">
        <div class="container">
            <div class="stats-grid">
                <div class="stat-card">
                    <div class="stat-value success">{{successRate}}%</div>
                    <div class="stat-label">Success Rate</div>
                </div>
                <div class="stat-card">
                    <div class="stat-value info">{{testedPages}}</div>
                    <div class="stat-label">Pages Tested</div>
                </div>
                <div class="stat-card">
                    <div class="stat-value error">{{totalErrors}}</div>
                    <div class="stat-label">Total Errors</div>
                </div>
                <div class="stat-card">
                    <div class="stat-value warning">{{totalWarnings}}</div>
                    <div class="stat-label">Total Warnings</div>
                </div>
                <div class="stat-card">
                    <div class="stat-value info">{{totalDuration}}</div>
                    <div class="stat-label">Duration</div>
                </div>
            </div>
            
            <div class="section">
                <div class="section-header">
                    <h2>🔍 Accessibility Analysis</h2>
                </div>
                <div class="section-content">
                    {{accessibility}}
                </div>
            </div>
            
            <div class="section">
                <div class="section-header">
                    <h2>⚡ Performance Analysis</h2>
                </div>
                <div class="section-content">
                    {{performance}}
                </div>
            </div>
            
            <div class="section">
                <div class="section-header">
                    <h2>🔍 SEO Analysis</h2>
                </div>
                <div class="section-content">
                    {{seo}}
                </div>
            </div>
            
            <div class="section">
                <div class="section-header">
                    <h2>📱 Mobile Friendliness</h2>
                </div>
                <div class="section-content">
                    {{mobileFriendliness}}
                </div>
            </div>
        </div>
    </div>
</body>
</html>`;

  let html = htmlTemplate
    // Replace sections
    .replace('{{accessibility}}', accessibilitySection)
    .replace('{{detailedIssues}}', detailedIssuesSection)
    .replace('{{performance}}', performanceSection)
    .replace('{{seo}}', seoSection)
    .replace('{{security}}', securitySection)
    .replace('{{mobileFriendliness}}', mobileFriendlinessSection)
    .replace('{{accessibility}}', accessibilitySection)
    .replace('{{issues}}', accessibilitySection)
    // Replace dashboard variables
    .replace(/{{domain}}/g, domain)
    .replace(/{{timestamp}}/g, data.metadata?.timestamp || new Date().toLocaleString())
    .replace(/{{successRate}}/g, successRate.toString())
    .replace(/{{totalPages}}/g, totalPages.toString())
    .replace(/{{testedPages}}/g, testedPages.toString())
    .replace(/{{totalErrors}}/g, totalErrors.toString())
    .replace(/{{totalWarnings}}/g, totalWarnings.toString())
    .replace(/{{totalDuration}}/g, formatDuration(finalTotalDuration));
  
  return html;
}
