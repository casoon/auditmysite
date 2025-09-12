/**
 * Comprehensive HTML Template for AuditMySite Reports (v1.8.4)
 * Features: Sticky navigation, KPIs, SEO/Performance sections, Interactive filters
 */

function getComprehensiveHtmlTemplate(data) {
  const { 
    domain, 
    testedPages = 0,
    totalPages = 0,
    totalErrors = 0,
    successRate = 0,
    totalDuration = '0s',
    groupedAccessibilityIssues = [],
    performanceResults = [],
    seoResults = []
  } = data;

  // Generate accessibility table
  const generateAccessibilityTable = () => {
    if (!groupedAccessibilityIssues || groupedAccessibilityIssues.length === 0) {
      return '<div class="no-issues"><h3>🎉 No accessibility issues found</h3><p>All tested pages passed accessibility checks.</p></div>';
    }

    let tableHTML = `
      <div class="table-container">
        <div class="table-header">
          <h3>Accessibility Issues</h3>
          <button class="copy-btn" onclick="copyToClipboard('accessibility-table')">📋 Copy Data</button>
        </div>
        <div class="table-wrapper">
          <table id="accessibility-table" class="data-table">
            <thead>
              <tr>
                <th>Type</th>
                <th>Page</th>
                <th>Element</th>
                <th>Message</th>
              </tr>
            </thead>
            <tbody>`;

    groupedAccessibilityIssues.forEach(group => {
      group.issues.forEach(issue => {
        const severity = issue.type || 'error';
        tableHTML += `
          <tr class="${severity}">
            <td><span class="${severity}">${severity.toUpperCase()}</span></td>
            <td>${issue.pageUrl || 'Unknown'}</td>
            <td><code>${issue.selector || 'N/A'}</code></td>
            <td>${issue.message || 'No message'}</td>
          </tr>`;
      });
    });

    tableHTML += '</tbody></table></div></div>';
    return tableHTML;
  };

  // Generate performance table
  const generatePerformanceTable = () => {
    if (!performanceResults || performanceResults.length === 0) {
      return '<div class="no-data"><h3>⏱️ No performance data available</h3><p>Performance metrics were not collected for this audit.</p></div>';
    }

    let tableHTML = `
      <div class="table-container">
        <div class="table-header">
          <h3>Performance Metrics</h3>
          <button class="copy-btn" onclick="copyToClipboard('performance-table')">📋 Copy Data</button>
        </div>
        <div class="table-wrapper">
          <table id="performance-table" class="data-table">
            <thead>
              <tr>
                <th>Page</th>
                <th>FCP</th>
                <th>LCP</th>
                <th>CLS</th>
                <th>Speed Index</th>
                <th>Grade</th>
              </tr>
            </thead>
            <tbody>`;

    performanceResults.forEach(result => {
      const grade = result.grade || 'N/A';
      tableHTML += `
        <tr>
          <td><strong>${result.url || 'Unknown'}</strong></td>
          <td>${result.fcp ? (result.fcp / 1000).toFixed(2) + 's' : 'N/A'}</td>
          <td>${result.lcp ? (result.lcp / 1000).toFixed(2) + 's' : 'N/A'}</td>
          <td>${result.cls ? result.cls.toFixed(3) : 'N/A'}</td>
          <td>${result.speedIndex ? Math.round(result.speedIndex) + 'ms' : 'N/A'}</td>
          <td><span class="grade grade-${grade}">${grade}</span></td>
        </tr>`;
    });

    tableHTML += '</tbody></table></div></div>';
    return tableHTML;
  };

  // Generate SEO table
  const generateSeoTable = () => {
    if (!seoResults || seoResults.length === 0) {
      return '<div class="no-data"><h3>🔍 No SEO data available</h3><p>SEO analysis was not performed for this audit.</p></div>';
    }

    let tableHTML = `
      <div class="table-container">
        <div class="table-header">
          <h3>SEO Analysis</h3>
          <button class="copy-btn" onclick="copyToClipboard('seo-table')">📋 Copy Data</button>
        </div>
        <div class="table-wrapper">
          <table id="seo-table" class="data-table">
            <thead>
              <tr>
                <th>Page & Title</th>
                <th>SEO Score</th>
                <th>Meta Description</th>
                <th>Headings</th>
                <th>Grade</th>
              </tr>
            </thead>
            <tbody>`;

    seoResults.forEach(result => {
      const grade = result.grade || result.seoGrade || 'N/A';
      const score = result.overallSEOScore || result.seoScore || 'N/A';
      const headingStructure = result.headings ? 
        `H1: ${result.headings.h1 || 0}, H2: ${result.headings.h2 || 0}` : 'N/A';
      
      tableHTML += `
        <tr>
          <td>
            <div class="page-info">
              <strong>${result.url || 'Unknown'}</strong>
              <div class="page-title">${result.title || 'No title'}</div>
            </div>
          </td>
          <td><strong>${score}${typeof score === 'number' ? '/100' : ''}</strong></td>
          <td>${result.metaDescription ? 
            (result.metaDescription.length > 50 ? 
              result.metaDescription.substring(0, 50) + '...' : 
              result.metaDescription) : 'Missing'}</td>
          <td>${headingStructure}</td>
          <td><span class="grade grade-${grade}">${grade}</span></td>
        </tr>`;
    });

    tableHTML += '</tbody></table></div>';
    
    // Add Advanced SEO Features Section
    tableHTML += '<div class="advanced-seo-section" style="margin-top: 2rem;">';
    tableHTML += '<h3 style="margin-bottom: 1rem;">🚀 Advanced SEO Analysis</h3>';
    
    seoResults.forEach(result => {
      if (result.semanticSEO || result.voiceSearchOptimization || result.eatAnalysis) {
        tableHTML += `<div class="advanced-seo-card" style="background: var(--surface-color); padding: 1.5rem; margin-bottom: 1rem; border-radius: 0.5rem; border: 1px solid var(--border-color);">`;
        tableHTML += `<h4 style="color: var(--text-primary); margin-bottom: 1rem;">🌐 ${result.url || 'Unknown'}</h4>`;
        
        // Semantic SEO
        if (result.semanticSEO) {
          tableHTML += `<div class="seo-metric" style="margin-bottom: 1rem;">`;
          tableHTML += `<h5 style="color: var(--text-secondary); margin-bottom: 0.5rem;">🧠 Semantic SEO</h5>`;
          tableHTML += `<div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 0.5rem; font-size: 0.875rem;">`;
          tableHTML += `<div><strong>Semantic Score:</strong> ${result.semanticSEO.semanticScore}/100</div>`;
          tableHTML += `<div><strong>Content Depth:</strong> ${result.semanticSEO.contentDepthScore}/100</div>`;
          tableHTML += `<div><strong>Topic Clusters:</strong> ${result.semanticSEO.topicClusters.slice(0, 3).join(', ')}</div>`;
          tableHTML += `<div><strong>LSI Keywords:</strong> ${result.semanticSEO.lsiKeywords.slice(0, 3).join(', ') || 'None'}</div>`;
          tableHTML += `</div></div>`;
        }
        
        // Voice Search Optimization
        if (result.voiceSearchOptimization) {
          tableHTML += `<div class="seo-metric" style="margin-bottom: 1rem;">`;
          tableHTML += `<h5 style="color: var(--text-secondary); margin-bottom: 0.5rem;">🎤 Voice Search Optimization</h5>`;
          tableHTML += `<div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 0.5rem; font-size: 0.875rem;">`;
          tableHTML += `<div><strong>Voice Score:</strong> ${result.voiceSearchOptimization.voiceSearchScore}/100</div>`;
          tableHTML += `<div><strong>Question Phrases:</strong> ${result.voiceSearchOptimization.questionPhrases}</div>`;
          tableHTML += `<div><strong>Conversational:</strong> ${result.voiceSearchOptimization.conversationalContent ? '✅ Yes' : '❌ No'}</div>`;
          tableHTML += `</div></div>`;
        }
        
        // E-A-T Analysis
        if (result.eatAnalysis) {
          tableHTML += `<div class="seo-metric" style="margin-bottom: 1rem;">`;
          tableHTML += `<h5 style="color: var(--text-secondary); margin-bottom: 0.5rem;">🏆 E-A-T Analysis</h5>`;
          tableHTML += `<div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 0.5rem; font-size: 0.875rem;">`;
          tableHTML += `<div><strong>E-A-T Score:</strong> ${result.eatAnalysis.eatScore}/100</div>`;
          tableHTML += `<div><strong>Author Present:</strong> ${result.eatAnalysis.authorPresence ? '✅ Yes' : '❌ No'}</div>`;
          tableHTML += `<div><strong>Trust Signals:</strong> ${result.eatAnalysis.trustSignals.length}</div>`;
          tableHTML += `<div><strong>Expertise Indicators:</strong> ${result.eatAnalysis.expertiseIndicators.length}</div>`;
          tableHTML += `</div></div>`;
        }
        
        tableHTML += '</div>';
      }
    });
    
    tableHTML += '</div></div>';
    return tableHTML;
  };

  return `<!DOCTYPE html>
<html lang="de">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Accessibility Test Report - ${domain}</title>
    <meta name="description" content="Comprehensive accessibility test report generated by auditmysite">
    
    <!-- Favicon -->
    <link rel="icon" type="image/svg+xml" href="data:image/svg+xml,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 100 100'><circle cx='50' cy='50' r='40' fill='%232563eb'/><text x='50' y='65' text-anchor='middle' fill='white' font-size='40' font-weight='bold'>A</text></svg>">
    
    <!-- CSS -->
    <style>
        /* CSS Variables für Theming */
        :root {
            --primary-color: #2563eb;
            --secondary-color: #64748b;
            --success-color: #10b981;
            --warning-color: #f59e0b;
            --error-color: #ef4444;
            --background-color: #ffffff;
            --surface-color: #f8fafc;
            --text-primary: #1e293b;
            --text-secondary: #64748b;
            --border-color: #e2e8f0;
            --shadow: 0 1px 3px 0 rgb(0 0 0 / 0.1), 0 1px 2px -1px rgb(0 0 0 / 0.1);
            --shadow-lg: 0 10px 15px -3px rgb(0 0 0 / 0.1), 0 4px 6px -4px rgb(0 0 0 / 0.1);
        }

        /* Dark Mode */
        @media (prefers-color-scheme: dark) {
            :root {
                --background-color: #0f172a;
                --surface-color: #1e293b;
                --text-primary: #f1f5f9;
                --text-secondary: #94a3b8;
                --border-color: #334155;
            }
        }

        /* Reset und Base Styles */
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }

        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
            line-height: 1.6;
            color: var(--text-primary);
            background-color: var(--background-color);
            transition: background-color 0.3s ease;
        }

        /* Header */
        .report-header {
            background: linear-gradient(135deg, var(--primary-color), #1d4ed8);
            color: white;
            padding: 1rem 0;
            box-shadow: var(--shadow-lg);
            position: sticky;
            top: 0;
            z-index: 100;
        }

        .report-nav {
            max-width: 1200px;
            margin: 0 auto;
            padding: 0 1rem;
            display: flex;
            justify-content: space-between;
            align-items: center;
        }

        .logo {
            font-size: 1.5rem;
            font-weight: bold;
            display: flex;
            align-items: center;
            gap: 0.5rem;
        }

        .logo::before {
            content: "🎯";
            font-size: 1.8rem;
        }

        .filter-badges {
            display: flex;
            list-style: none;
            gap: 1rem;
            flex-wrap: wrap;
        }

        .filter-badge {
            background: rgba(255, 255, 255, 0.15);
            color: white;
            border: 2px solid rgba(255, 255, 255, 0.3);
            padding: 0.5rem 1rem;
            border-radius: 2rem;
            cursor: pointer;
            transition: all 0.2s ease;
            font-size: 0.875rem;
            font-weight: 500;
            user-select: none;
        }

        .filter-badge:hover {
            background: rgba(255, 255, 255, 0.25);
            border-color: rgba(255, 255, 255, 0.5);
        }

        .filter-badge.active {
            background: rgba(255, 255, 255, 0.9);
            color: var(--primary-color);
            border-color: white;
        }

        .filter-badge.inactive {
            opacity: 0.6;
            background: rgba(255, 255, 255, 0.05);
        }

        /* Main Content */
        .main-content {
            max-width: 1200px;
            margin: 0 auto;
            padding: 2rem 1rem;
        }

        /* Dashboard */
        .dashboard {
            margin-bottom: 3rem;
        }

        .kpi-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
            gap: 1.5rem;
            margin-bottom: 2rem;
        }

        .kpi-card {
            background: var(--surface-color);
            padding: 1.5rem;
            border-radius: 0.75rem;
            box-shadow: var(--shadow);
            border: 1px solid var(--border-color);
            transition: transform 0.2s ease, box-shadow 0.2s ease;
        }

        .kpi-card:hover {
            transform: translateY(-2px);
            box-shadow: var(--shadow-lg);
        }

        .kpi-card h3 {
            color: var(--text-secondary);
            font-size: 0.875rem;
            font-weight: 600;
            text-transform: uppercase;
            letter-spacing: 0.05em;
            margin-bottom: 0.5rem;
        }

        .kpi-value {
            font-size: 2rem;
            font-weight: bold;
            margin-bottom: 0.5rem;
        }

        .kpi-trend {
            font-size: 0.875rem;
            font-weight: 500;
        }

        .kpi-trend.positive {
            color: var(--success-color);
        }

        .kpi-trend.negative {
            color: var(--error-color);
        }

        /* Table Styles */
        .table-container {
            background: var(--surface-color);
            border-radius: 0.75rem;
            box-shadow: var(--shadow);
            border: 1px solid var(--border-color);
            margin-bottom: 2rem;
            overflow: hidden;
        }

        .table-header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 1.5rem;
            border-bottom: 1px solid var(--border-color);
            background: var(--background-color);
        }

        .table-header h3 {
            font-size: 1.25rem;
            font-weight: 600;
            color: var(--text-primary);
        }

        .copy-btn {
            background: var(--primary-color);
            color: white;
            border: none;
            padding: 0.5rem 1rem;
            border-radius: 0.5rem;
            cursor: pointer;
            font-size: 0.875rem;
            font-weight: 500;
            transition: background-color 0.2s ease;
            display: flex;
            align-items: center;
            gap: 0.5rem;
        }

        .copy-btn:hover {
            background: #1d4ed8;
        }

        .copy-btn:active {
            transform: translateY(1px);
        }

        .table-wrapper {
            overflow-x: auto;
        }

        .data-table {
            width: 100%;
            border-collapse: collapse;
            font-size: 0.875rem;
        }

        .data-table th {
            background: var(--background-color);
            padding: 1rem;
            text-align: left;
            font-weight: 600;
            color: var(--text-primary);
            border-bottom: 2px solid var(--border-color);
            position: sticky;
            top: 0;
            z-index: 10;
        }

        .data-table td {
            padding: 1rem;
            border-bottom: 1px solid var(--border-color);
            color: var(--text-primary);
        }

        .data-table tr:hover {
            background: var(--background-color);
        }

        .data-table tr.error {
            background: rgba(239, 68, 68, 0.05);
        }

        .data-table tr.warning {
            background: rgba(245, 158, 11, 0.05);
        }

        .data-table tr.error:hover {
            background: rgba(239, 68, 68, 0.1);
        }

        .data-table tr.warning:hover {
            background: rgba(245, 158, 11, 0.1);
        }

        /* Section styling */
        .issues-section {
            background: var(--surface-color);
            border-radius: 0.75rem;
            margin-bottom: 2rem;
            overflow: hidden;
            border: 1px solid var(--border-color);
            transition: all 0.3s ease;
        }

        .issues-section.hidden {
            display: none;
        }

        .section-title {
            font-size: 1.5rem;
            font-weight: 700;
            color: var(--text-primary);
            margin-bottom: 0.5rem;
            display: flex;
            align-items: center;
            gap: 0.5rem;
        }

        .section-description {
            color: var(--text-secondary);
            font-size: 0.95rem;
            margin-bottom: 1.5rem;
            font-style: italic;
        }

        /* Page info styling for SEO section */
        .page-info {
            line-height: 1.4;
        }

        .page-info strong {
            color: var(--text-primary);
            font-size: 0.95rem;
        }

        .page-title {
            color: var(--text-secondary);
            font-size: 0.85rem;
            margin-top: 0.25rem;
        }

        .page-url {
            color: var(--text-secondary);
            font-size: 0.75rem;
            opacity: 0.7;
            margin-top: 0.125rem;
        }

        /* No Data States */
        .no-data, .no-issues {
            text-align: center;
            padding: 3rem 1rem;
            color: var(--text-secondary);
        }

        .no-data h3, .no-issues h3 {
            font-size: 1.5rem;
            margin-bottom: 1rem;
            color: var(--text-primary);
        }

        /* Toast Notification */
        .toast {
            position: fixed;
            top: 2rem;
            right: 2rem;
            background: var(--success-color);
            color: white;
            padding: 1rem 1.5rem;
            border-radius: 0.5rem;
            box-shadow: var(--shadow-lg);
            z-index: 1000;
            transform: translateX(100%);
            transition: transform 0.3s ease;
        }

        .toast.show {
            transform: translateX(0);
        }

        /* Grade Badges */
        .grade {
            padding: 0.25rem 0.5rem;
            border-radius: 0.25rem;
            font-weight: 600;
            font-size: 0.75rem;
        }

        .grade-A { background: #dcfce7; color: #166534; }
        .grade-B { background: #fef3c7; color: #92400e; }
        .grade-C { background: #fed7d7; color: #991b1b; }
        .grade-D, .grade-F { background: #fee2e2; color: #991b1b; }

        /* Severity colors */
        .error { color: var(--error-color); font-weight: 600; }
        .warning { color: var(--warning-color); font-weight: 600; }
        .notice { color: var(--secondary-color); font-weight: 600; }

        /* Responsive Design */
        @media (max-width: 768px) {
            .filter-badges {
                justify-content: center;
            }
            
            .table-header {
                flex-direction: column;
                gap: 1rem;
                align-items: flex-start;
            }
            
            .data-table {
                font-size: 0.75rem;
            }
            
            .data-table th,
            .data-table td {
                padding: 0.75rem 0.5rem;
            }
        }
    </style>
</head>
<body>
    <header class="report-header">
        <nav class="report-nav">
            <div class="logo">auditmysite</div>
            <div class="filter-badges">
                <span class="filter-badge active" data-section="summary">📊 Summary</span>
                <span class="filter-badge active" data-section="accessibility">♿ Accessibility</span>
                <span class="filter-badge" data-section="performance">⚡ Performance</span>
                <span class="filter-badge" data-section="seo">🔍 SEO</span>
            </div>
        </nav>
    </header>

    <main class="main-content">
        <!-- Dashboard Section -->
        <section id="summary" class="dashboard">
            <h2 class="section-title">Test Summary</h2>
            <div class="kpi-grid">
                <div class="kpi-card">
                    <h3>Success Rate</h3>
                    <div class="kpi-value">${successRate}%</div>
                    <div class="kpi-trend positive">Passed</div>
                </div>
                <div class="kpi-card">
                    <h3>Pages Tested</h3>
                    <div class="kpi-value">${testedPages}/${totalPages}</div>
                    <div class="kpi-trend">Total Pages</div>
                </div>
                <div class="kpi-card">
                    <h3>Total Errors</h3>
                    <div class="kpi-value">${totalErrors}</div>
                    <div class="kpi-trend negative">Issues Found</div>
                </div>
                <div class="kpi-card">
                    <h3>Test Duration</h3>
                    <div class="kpi-value">${totalDuration}</div>
                    <div class="kpi-trend">Time Taken</div>
                </div>
            </div>
        </section>

        <!-- Accessibility Section -->
        <section id="accessibility" class="issues-section">
            <div class="table-header">
                <h2 class="section-title">♿ Accessibility Issues</h2>
            </div>
            <div style="padding: 1.5rem;">
                <p class="section-description">Web accessibility compliance and WCAG violations analysis</p>
                ${generateAccessibilityTable()}
            </div>
        </section>

        <!-- Performance Section -->
        <section id="performance" class="issues-section">
            <div class="table-header">
                <h2 class="section-title">⚡ Performance Metrics</h2>
            </div>
            <div style="padding: 1.5rem;">
                <p class="section-description">Web page performance metrics and loading times</p>
                ${generatePerformanceTable()}
            </div>
        </section>

        <!-- SEO Section -->
        <section id="seo" class="issues-section">
            <div class="table-header">
                <h2 class="section-title">🔍 SEO Analysis</h2>
            </div>
            <div style="padding: 1.5rem;">
                <p class="section-description">Search engine optimization analysis and content structure</p>
                ${generateSeoTable()}
            </div>
        </section>
    </main>

    <!-- Toast Notification -->
    <div id="toast" class="toast">
        <span id="toast-message">Copied to clipboard!</span>
    </div>

    <!-- JavaScript für Interaktivität -->
    <script>
        // Copy to Clipboard Funktion
        function copyToClipboard(tableId) {
            const table = document.getElementById(tableId);
            if (!table) return;

            // Erstelle eine temporäre Textarea für das Kopieren
            const textarea = document.createElement('textarea');
            textarea.value = table.innerText;
            document.body.appendChild(textarea);
            textarea.select();
            
            try {
                document.execCommand('copy');
                showToast('Daten in Zwischenablage kopiert!');
            } catch (err) {
                showToast('Fehler beim Kopieren!');
            }
            
            document.body.removeChild(textarea);
        }

        // Toast Notification
        function showToast(message) {
            const toast = document.getElementById('toast');
            const toastMessage = document.getElementById('toast-message');
            
            toastMessage.textContent = message;
            toast.classList.add('show');
            
            setTimeout(() => {
                toast.classList.remove('show');
            }, 3000);
        }

        // Smooth Scrolling für Navigation
        function initSmoothScrolling() {
            document.querySelectorAll('a[href^="#"]').forEach(anchor => {
                anchor.addEventListener('click', function (e) {
                    e.preventDefault();
                    const target = document.querySelector(this.getAttribute('href'));
                    if (target) {
                        target.scrollIntoView({
                            behavior: 'smooth',
                            block: 'start'
                        });
                    }
                });
            });
        }

        // Dark Mode Toggle
        function initDarkMode() {
            const prefersDark = window.matchMedia('(prefers-color-scheme: dark)');
            
            function updateTheme() {
                document.documentElement.classList.toggle('dark', prefersDark.matches);
            }
            
            prefersDark.addEventListener('change', updateTheme);
            updateTheme();
        }

        // Filter Badge System
        function initFilterSystem() {
            const badges = document.querySelectorAll('.filter-badge');
            const sections = document.querySelectorAll('.issues-section, .dashboard');
            
            badges.forEach(badge => {
                badge.addEventListener('click', function() {
                    const targetSection = this.getAttribute('data-section');
                    
                    // Toggle badge state
                    this.classList.toggle('active');
                    
                    // Show/hide corresponding section
                    const section = document.getElementById(targetSection);
                    if (section) {
                        if (this.classList.contains('active')) {
                            section.classList.remove('hidden');
                            section.style.display = 'block';
                        } else {
                            section.classList.add('hidden');
                            section.style.display = 'none';
                        }
                    }
                    
                    // Update badge appearance based on active state
                    updateBadgeStates();
                });
            });
            
            // Initialize: Show summary and accessibility by default, hide others
            sections.forEach(function(section) {
                const sectionId = section.id;
                const badge = document.querySelector('[data-section="' + sectionId + '"]');
                
                if (sectionId === 'summary' || sectionId === 'accessibility') {
                    section.classList.remove('hidden');
                    section.style.display = 'block';
                    if (badge) badge.classList.add('active');
                } else {
                    section.classList.add('hidden');
                    section.style.display = 'none';
                    if (badge) badge.classList.remove('active');
                }
            });
        }
        
        function updateBadgeStates() {
            const badges = document.querySelectorAll('.filter-badge');
            const activeBadges = document.querySelectorAll('.filter-badge.active');
            
            badges.forEach(badge => {
                if (activeBadges.length === 0) {
                    badge.classList.remove('inactive');
                } else {
                    if (badge.classList.contains('active')) {
                        badge.classList.remove('inactive');
                    } else {
                        badge.classList.add('inactive');
                    }
                }
            });
        }

        // Initialize everything when DOM is loaded
        document.addEventListener('DOMContentLoaded', function() {
            initSmoothScrolling();
            initDarkMode();
            initFilterSystem();
        });
    </script>
</body>
</html>`;
}

module.exports = { getComprehensiveHtmlTemplate };
