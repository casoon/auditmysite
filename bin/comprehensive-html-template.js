/**
 * Comprehensive HTML Template for AuditMySite Reports
 */

function getComprehensiveHtmlTemplate(data) {
  const { 
    domain, 
    summary, 
    results, 
    allDetailedIssues, 
    groupedIssues, 
    timestamp, 
    successRate, 
    errorCount, 
    warningCount, 
    noticeCount 
  } = data;
  
  return `<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Accessibility Report - ${domain}</title>
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
        
        .nav-filters {
            display: flex;
            gap: 0.5rem;
            flex-wrap: wrap;
        }
        
        .filter-badge {
            background: rgba(255, 255, 255, 0.1);
            color: white;
            padding: 0.5rem 1rem;
            border-radius: 0.5rem;
            text-decoration: none;
            font-size: 0.875rem;
            font-weight: 500;
            border: 1px solid rgba(255, 255, 255, 0.2);
            transition: all 0.2s;
            cursor: pointer;
        }
        
        .filter-badge:hover,
        .filter-badge.active {
            background: rgba(255, 255, 255, 0.2);
            border-color: rgba(255, 255, 255, 0.4);
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
        
        .results-table {
            width: 100%;
            border-collapse: collapse;
            margin-top: 1rem;
        }
        
        .results-table th,
        .results-table td {
            padding: 0.75rem;
            text-align: left;
            border-bottom: 1px solid #e2e8f0;
        }
        
        .results-table th {
            background: #f8fafc;
            font-weight: 600;
            color: #374151;
        }
        
        .results-table tr:hover {
            background: #f9fafb;
        }
        
        .grade {
            padding: 0.25rem 0.5rem;
            border-radius: 0.25rem;
            font-weight: 600;
            font-size: 0.875rem;
        }
        
        .grade-A { background: #dcfce7; color: #166534; }
        .grade-B { background: #fef3c7; color: #92400e; }
        .grade-C { background: #fed7d7; color: #991b1b; }
        .grade-D, .grade-F { background: #fee2e2; color: #991b1b; }
        
        .issues-group {
            margin-bottom: 2rem;
        }
        
        .issues-group h3 {
            background: #f1f5f9;
            padding: 1rem;
            margin: 0 -1.5rem 1rem -1.5rem;
            border-left: 4px solid #3b82f6;
            font-size: 1rem;
            font-weight: 600;
        }
        
        .issue-item {
            border: 1px solid #e2e8f0;
            border-radius: 0.5rem;
            margin-bottom: 1rem;
            overflow: hidden;
        }
        
        .issue-header {
            background: #f8fafc;
            padding: 0.75rem 1rem;
            cursor: pointer;
            display: flex;
            justify-content: space-between;
            align-items: center;
        }
        
        .issue-severity {
            padding: 0.25rem 0.5rem;
            border-radius: 0.25rem;
            font-size: 0.75rem;
            font-weight: 500;
        }
        
        .issue-content {
            padding: 1rem;
            display: none;
        }
        
        .issue-meta {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 1rem;
            margin-bottom: 1rem;
            padding: 1rem;
            background: #f9fafb;
            border-radius: 0.5rem;
        }
        
        .copy-btn {
            background: #3b82f6;
            color: white;
            border: none;
            padding: 0.5rem 1rem;
            border-radius: 0.25rem;
            cursor: pointer;
            font-size: 0.875rem;
            transition: background 0.2s;
        }
        
        .copy-btn:hover {
            background: #2563eb;
        }
        
        .hidden { display: none; }
        
        @media (max-width: 768px) {
            .header-content {
                flex-direction: column;
                align-items: stretch;
            }
            
            .nav-filters {
                justify-content: center;
            }
            
            .stats-grid {
                grid-template-columns: repeat(2, 1fr);
            }
            
            .results-table {
                font-size: 0.875rem;
            }
            
            .results-table th,
            .results-table td {
                padding: 0.5rem;
            }
        }
    </style>
</head>
<body>
    <header class="header">
        <div class="container">
            <div class="header-content">
                <h1>🎯 Accessibility Report - ${domain}</h1>
                <nav class="nav-filters">
                    <a href="#summary" class="filter-badge active" data-section="summary">Summary</a>
                    <a href="#accessibility" class="filter-badge" data-section="accessibility">Accessibility</a>
                    <a href="#detailed-issues" class="filter-badge" data-section="detailed-issues">Detailed Issues</a>
                </nav>
            </div>
        </div>
    </header>

    <main class="main-content">
        <div class="container">
            <!-- Summary Section -->
            <section id="summary" class="section">
                <div class="section-header">
                    <h2>📊 Test Summary</h2>
                </div>
                <div class="section-content">
                    <div class="stats-grid">
                        <div class="stat-card">
                            <span class="stat-value success">${summary.testedPages}</span>
                            <div class="stat-label">Pages Tested</div>
                        </div>
                        <div class="stat-card">
                            <span class="stat-value success">${summary.passedPages}</span>
                            <div class="stat-label">Pages Passed</div>
                        </div>
                        <div class="stat-card">
                            <span class="stat-value error">${summary.totalErrors || 0}</span>
                            <div class="stat-label">Total Errors</div>
                        </div>
                        <div class="stat-card">
                            <span class="stat-value warning">${summary.totalWarnings || 0}</span>
                            <div class="stat-label">Total Warnings</div>
                        </div>
                        <div class="stat-card">
                            <span class="stat-value info">${successRate.toFixed(1)}%</span>
                            <div class="stat-label">Success Rate</div>
                        </div>
                        <div class="stat-card">
                            <span class="stat-value info">${allDetailedIssues.length}</span>
                            <div class="stat-label">Total Issues</div>
                        </div>
                    </div>
                </div>
            </section>

            <!-- Accessibility Results Section -->
            <section id="accessibility" class="section">
                <div class="section-header">
                    <h2>♿ Accessibility Results</h2>
                </div>
                <div class="section-content">
                    <table class="results-table">
                        <thead>
                            <tr>
                                <th>Page</th>
                                <th>Status</th>
                                <th>Issues Found</th>
                                <th>Quality Score</th>
                            </tr>
                        </thead>
                        <tbody>
                            ${results.map(page => `
                                <tr>
                                    <td>
                                        <strong>${getPageName(page.url)}</strong>
                                        <br><small style="color: #64748b;">${page.url}</small>
                                    </td>
                                    <td>
                                        ${page.passed ? '✅ Passed' : '❌ Failed'}
                                        ${page.errors ? `<br><small class="error">${page.errors} errors</small>` : ''}
                                        ${page.warnings ? `<br><small class="warning">${page.warnings} warnings</small>` : ''}
                                    </td>
                                    <td>
                                        ${page.pa11yIssues && Array.isArray(page.pa11yIssues) ? 
                                            `<strong>${page.pa11yIssues.length} issues</strong><br><small>Score: ${page.pa11yScore}/100</small>` : 
                                            'No data'
                                        }
                                    </td>
                                    <td>
                                        ${page.qualityScore ? 
                                            `<span class="grade grade-${page.qualityScore.grade}">${page.qualityScore.score}/100 (${page.qualityScore.grade})</span>` : 'N/A'
                                        }
                                    </td>
                                </tr>
                            `).join('')}
                        </tbody>
                    </table>
                </div>
            </section>

            <!-- Detailed Issues Section -->
            <section id="detailed-issues" class="section">
                <div class="section-header">
                    <h2>🔍 Detailed Issues (${allDetailedIssues.length})</h2>
                </div>
                <div class="section-content">
                    ${Object.entries(groupedIssues).map(([category, issues]) => `
                        <div class="issues-group">
                            <h3>${category} (${issues.length} issues)</h3>
                            ${issues.map((issue, index) => `
                                <div class="issue-item">
                                    <div class="issue-header" onclick="toggleIssue('issue-${category}-${index}')">
                                        <span>
                                            <span class="issue-severity ${issue.severity || 'error'}">${(issue.severity || 'error').toUpperCase()}</span>
                                            ${issue.message}
                                        </span>
                                        <span style="font-size: 0.75rem; color: #64748b;">Click to expand</span>
                                    </div>
                                    <div id="issue-${category}-${index}" class="issue-content">
                                        <div class="issue-meta">
                                            <div>
                                                <strong>Page:</strong><br>
                                                <a href="${issue.pageUrl}" target="_blank">${getPageName(issue.pageUrl)}</a>
                                            </div>
                                            <div>
                                                <strong>Code:</strong><br>
                                                <code>${issue.code || 'N/A'}</code>
                                            </div>
                                            <div>
                                                <strong>Selector:</strong><br>
                                                <code>${issue.selector || 'N/A'}</code>
                                            </div>
                                            <div>
                                                <strong>Source:</strong><br>
                                                ${issue.source || 'pa11y'}
                                            </div>
                                        </div>
                                        
                                        ${issue.context ? `
                                            <div style="margin-bottom: 1rem;">
                                                <strong>HTML Context:</strong>
                                                <pre style="background: #f1f5f9; padding: 0.75rem; border-radius: 0.25rem; overflow-x: auto; font-size: 0.875rem;"><code>${escapeHtml(issue.context)}</code></pre>
                                            </div>
                                        ` : ''}
                                        
                                        ${issue.recommendation ? `
                                            <div style="margin-bottom: 1rem;">
                                                <strong>How to Fix:</strong>
                                                <p style="margin-top: 0.5rem; color: #374151;">${issue.recommendation}</p>
                                            </div>
                                        ` : ''}
                                        
                                        <button class="copy-btn" onclick="copyIssueToClipboard('issue-${category}-${index}')">
                                            📋 Copy Issue Details
                                        </button>
                                    </div>
                                </div>
                            `).join('')}
                        </div>
                    `).join('')}
                </div>
            </section>
        </div>
    </main>

    <footer style="background: #f8fafc; border-top: 1px solid #e2e8f0; padding: 2rem 0; margin-top: 4rem; text-align: center; color: #64748b;">
        <div class="container">
            <p>Generated by AuditMySite v2.0 - ${timestamp}</p>
            <p style="margin-top: 0.5rem;">✨ Simplified CLI with only 11 parameters for better usability</p>
        </div>
    </footer>

    <script>
        // Navigation and filtering
        document.querySelectorAll('.filter-badge').forEach(badge => {
            badge.addEventListener('click', function(e) {
                e.preventDefault();
                
                // Update active badge
                document.querySelectorAll('.filter-badge').forEach(b => b.classList.remove('active'));
                this.classList.add('active');
                
                // Show/hide sections
                const targetSection = this.getAttribute('data-section');
                document.querySelectorAll('.section').forEach(section => {
                    section.style.display = section.id === targetSection ? 'block' : 'none';
                });
            });
        });
        
        // Issue expansion
        function toggleIssue(issueId) {
            const content = document.getElementById(issueId);
            content.style.display = content.style.display === 'none' || !content.style.display ? 'block' : 'none';
        }
        
        // Copy to clipboard
        async function copyIssueToClipboard(issueId) {
            const issueElement = document.getElementById(issueId);
            const text = issueElement.innerText;
            
            try {
                await navigator.clipboard.writeText(text);
                
                // Show feedback
                const btn = issueElement.querySelector('.copy-btn');
                const originalText = btn.textContent;
                btn.textContent = '✅ Copied!';
                setTimeout(() => {
                    btn.textContent = originalText;
                }, 2000);
            } catch (err) {
                console.error('Failed to copy text: ', err);
            }
        }
        
        // Initialize - show summary by default
        document.querySelectorAll('.section').forEach(section => {
            section.style.display = section.id === 'summary' ? 'block' : 'none';
        });
    </script>
</body>
</html>`;
}

function getPageName(url) {
  try {
    const urlObj = new URL(url);
    const pathname = urlObj.pathname;
    return pathname === '/' ? 'Home' : pathname.split('/').pop() || pathname;
  } catch {
    return url;
  }
}

function escapeHtml(text) {
  return (text || '')
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#039;');
}

module.exports = {
  getComprehensiveHtmlTemplate
};
