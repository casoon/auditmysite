/**
 * Report Data Validation Tests
 * 
 * Ensures that data from CLI analysis is correctly displayed in HTML reports
 * This prevents manual testing and catches data mapping issues automatically
 */

const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');
const cheerio = require('cheerio');

describe('Report Data Validation', () => {
  let testResults;
  let reportHtml;
  let reportPath;
  
  beforeAll(async () => {
    // Run analysis on test URL
    const testUrl = 'https://example.com/sitemap.xml';
    const maxPages = 1;
    
    console.log('Running analysis for data validation test...');
    try {
      const output = execSync(
        `node bin/audit.js ${testUrl} --max-pages ${maxPages} --non-interactive`,
        { encoding: 'utf-8', cwd: process.cwd() }
      );
      
      // Extract metrics from CLI output
      testResults = extractMetricsFromCliOutput(output);
      
      // Find the generated report
      reportPath = findLatestReport();
      if (fs.existsSync(reportPath)) {
        reportHtml = fs.readFileSync(reportPath, 'utf-8');
      }
      
    } catch (error) {
      console.error('Failed to run analysis:', error);
      throw error;
    }
  }, 60000); // 60 second timeout

  describe('Performance Metrics Validation', () => {
    test('Performance score should match CLI output', () => {
      const reportScore = extractPerformanceScoreFromHtml(reportHtml);
      const cliScore = testResults.performanceScore;
      
      expect(reportScore).toBeDefined();
      expect(cliScore).toBeDefined();
      expect(reportScore).toBe(cliScore);
    });

    test('LCP values should match CLI output', () => {
      const reportLCP = extractLCPFromHtml(reportHtml);
      const cliLCP = testResults.lcp;
      
      expect(reportLCP).toBeDefined();
      expect(cliLCP).toBeDefined();
      expect(Math.abs(reportLCP - cliLCP)).toBeLessThan(50); // Allow 50ms tolerance
    });

    test('Content weight should match CLI output', () => {
      const reportWeight = extractContentWeightFromHtml(reportHtml);
      const cliWeight = testResults.contentWeight;
      
      expect(reportWeight).toBeDefined();
      expect(cliWeight).toBeDefined();
      
      // Convert to same unit for comparison
      const reportMB = convertToMB(reportWeight);
      const cliMB = convertToMB(cliWeight);
      
      expect(Math.abs(reportMB - cliMB)).toBeLessThan(1); // Allow 1MB tolerance
    });
  });

  describe('SEO Metrics Validation', () => {
    test('SEO score should match CLI output', () => {
      const reportScore = extractSeoScoreFromHtml(reportHtml);
      const cliScore = testResults.seoScore;
      
      expect(reportScore).toBeDefined();
      expect(cliScore).toBeDefined();
      expect(reportScore).toBe(cliScore);
    });

    test('SEO metrics should not show N/A when data exists', () => {
      const $ = cheerio.load(reportHtml);
      const seoSection = $('#seo');
      
      // Check if we have enhanced SEO data in CLI but showing N/A in report
      if (testResults.seoScore && testResults.seoScore > 0) {
        const naValues = seoSection.find('td:contains("N/A")').length;
        expect(naValues).toBeLessThan(3); // Allow some N/A but not all fields
      }
    });
  });

  describe('Mobile-Friendliness Validation', () => {
    test('Mobile score should match CLI output', () => {
      const reportScore = extractMobileScoreFromHtml(reportHtml);
      const cliScore = testResults.mobileScore;
      
      if (cliScore && cliScore > 0) {
        expect(reportScore).toBeDefined();
        expect(reportScore).toBe(cliScore);
      }
    });
  });

  describe('Accessibility Data Validation', () => {
    test('Error count should match CLI output', () => {
      const reportErrors = extractErrorCountFromHtml(reportHtml);
      const cliErrors = testResults.errors;
      
      expect(reportErrors).toBeDefined();
      expect(cliErrors).toBeDefined();
      expect(reportErrors).toBe(cliErrors);
    });

    test('Warning count should match CLI output', () => {
      const reportWarnings = extractWarningCountFromHtml(reportHtml);
      const cliWarnings = testResults.warnings;
      
      expect(reportWarnings).toBeDefined();
      expect(cliWarnings).toBeDefined();
      expect(reportWarnings).toBe(cliWarnings);
    });

    test('Pa11y score should not be N/A if data exists', () => {
      const pa11yScore = extractPa11yScoreFromHtml(reportHtml);
      
      // If we have accessibility data, Pa11y score should be calculated
      if (testResults.errors || testResults.warnings) {
        expect(pa11yScore).not.toBe('N/A');
        expect(pa11yScore).toMatch(/\d+\/100/);
      }
    });
  });

  describe('Data Completeness Validation', () => {
    test('All major sections should have data when CLI shows data', () => {
      const $ = cheerio.load(reportHtml);
      
      // Check that sections aren't empty when we have CLI data
      const sections = ['accessibility', 'performance', 'seo', 'mobile-friendliness'];
      
      sections.forEach(section => {
        const sectionElement = $(`#${section}`);
        expect(sectionElement.length).toBe(1);
        
        const tables = sectionElement.find('table');
        if (tables.length > 0) {
          const rows = sectionElement.find('tbody tr');
          expect(rows.length).toBeGreaterThan(0);
        }
      });
    });

    test('Detailed issues section should contain actual issues', () => {
      const $ = cheerio.load(reportHtml);
      const detailedSection = $('#detailed-issues');
      
      if (testResults.errors > 0 || testResults.warnings > 0) {
        const issueItems = detailedSection.find('.issue-item');
        expect(issueItems.length).toBeGreaterThan(0);
        
        const totalIssues = testResults.errors + testResults.warnings;
        expect(issueItems.length).toBe(totalIssues);
      }
    });
  });

  describe('Data Structure Consistency', () => {
    test('Enhanced data should be properly mapped', () => {
      // This test ensures enhanced data structures are consistently used
      const dataStructureReport = analyzeDataStructure(reportPath);
      
      expect(dataStructureReport.hasEnhancedPerformance).toBe(true);
      expect(dataStructureReport.hasEnhancedSEO).toBe(true);
      expect(dataStructureReport.hasContentWeight).toBe(true);
      expect(dataStructureReport.hasMobileFriendliness).toBe(true);
    });
  });
});

// Helper Functions
function extractMetricsFromCliOutput(output) {
  const metrics = {};
  
  // Extract performance score
  const perfMatch = output.match(/Performance Score: (\d+)\/100/);
  if (perfMatch) metrics.performanceScore = parseInt(perfMatch[1]);
  
  // Extract LCP
  const lcpMatch = output.match(/LCP: (\d+)ms/);
  if (lcpMatch) metrics.lcp = parseInt(lcpMatch[1]);
  
  // Extract content weight
  const weightMatch = output.match(/Total page weight: ([\d.]+)\s*(MB|GB|KB|B)/);
  if (weightMatch) metrics.contentWeight = `${weightMatch[1]} ${weightMatch[2]}`;
  
  // Extract SEO score
  const seoMatch = output.match(/SEO Score: (\d+)\/100/);
  if (seoMatch) metrics.seoScore = parseInt(seoMatch[1]);
  
  // Extract mobile score
  const mobileMatch = output.match(/Mobile Score: (\d+)\/100/);
  if (mobileMatch) metrics.mobileScore = parseInt(mobileMatch[1]);
  
  // Extract error/warning counts
  const errorMatch = output.match(/⚠️\s+Errors: (\d+)/);
  if (errorMatch) metrics.errors = parseInt(errorMatch[1]);
  
  const warningMatch = output.match(/⚠️\s+Warnings: (\d+)/);
  if (warningMatch) metrics.warnings = parseInt(warningMatch[1]);
  
  return metrics;
}

function extractPerformanceScoreFromHtml(html) {
  const $ = cheerio.load(html);
  const scoreElement = $('.metric-label:contains("Performance Score")').parent().find('.metric-value');
  return scoreElement.length > 0 ? parseInt(scoreElement.text()) : null;
}

function extractLCPFromHtml(html) {
  const $ = cheerio.load(html);
  const lcpElement = $('.metric-label:contains("LCP")').parent().find('.metric-value');
  if (lcpElement.length > 0) {
    const lcpText = lcpElement.text();
    const match = lcpText.match(/(\d+)ms/);
    return match ? parseInt(match[1]) : null;
  }
  return null;
}

function extractContentWeightFromHtml(html) {
  const $ = cheerio.load(html);
  const weightElement = $('.breakdown-type:contains("Total")').parent().find('.breakdown-size');
  return weightElement.length > 0 ? weightElement.text().trim() : null;
}

function extractSeoScoreFromHtml(html) {
  const $ = cheerio.load(html);
  const seoElement = $('.metric-label:contains("SEO Score")').parent().find('.metric-value');
  return seoElement.length > 0 ? parseInt(seoElement.text()) : null;
}

function extractMobileScoreFromHtml(html) {
  const $ = cheerio.load(html);
  const mobileTable = $('#mobile-friendliness table');
  if (mobileTable.length > 0) {
    const scoreCell = mobileTable.find('tbody tr td:nth-child(2)');
    if (scoreCell.length > 0) {
      const scoreText = scoreCell.text().trim();
      const match = scoreText.match(/(\d+)\/100/);
      return match ? parseInt(match[1]) : null;
    }
  }
  return null;
}

function extractErrorCountFromHtml(html) {
  const $ = cheerio.load(html);
  const errorElement = $('.kpi-card h3:contains("Total Errors")').parent().find('.kpi-value');
  return errorElement.length > 0 ? parseInt(errorElement.text()) : null;
}

function extractWarningCountFromHtml(html) {
  const $ = cheerio.load(html);
  const accessibilityTable = $('#accessibility table tbody tr td:nth-child(3)');
  return accessibilityTable.length > 0 ? parseInt(accessibilityTable.text()) : null;
}

function extractPa11yScoreFromHtml(html) {
  const $ = cheerio.load(html);
  const pa11yCell = $('#accessibility table tbody tr td:nth-child(4)');
  return pa11yCell.length > 0 ? pa11yCell.text().trim() : null;
}

function findLatestReport() {
  const reportsDir = path.join(process.cwd(), 'reports');
  const todayDate = new Date().toISOString().split('T')[0];
  
  // Look for today's reports
  const subdirs = fs.readdirSync(reportsDir, { withFileTypes: true })
    .filter(dirent => dirent.isDirectory())
    .map(dirent => dirent.name);
  
  for (const subdir of subdirs) {
    const subdirPath = path.join(reportsDir, subdir);
    const files = fs.readdirSync(subdirPath);
    
    const htmlFiles = files
      .filter(file => file.includes('accessibility-report') && file.includes(todayDate) && file.endsWith('.html'))
      .sort((a, b) => b.localeCompare(a));
    
    if (htmlFiles.length > 0) {
      return path.join(subdirPath, htmlFiles[0]);
    }
  }
  
  throw new Error('No recent HTML report found');
}

function convertToMB(sizeString) {
  if (!sizeString) return 0;
  
  const match = sizeString.match(/([\d.]+)\s*(MB|GB|KB|B)/i);
  if (!match) return 0;
  
  const value = parseFloat(match[1]);
  const unit = match[2].toUpperCase();
  
  switch (unit) {
    case 'GB': return value * 1024;
    case 'MB': return value;
    case 'KB': return value / 1024;
    case 'B': return value / (1024 * 1024);
    default: return 0;
  }
}

function analyzeDataStructure(reportPath) {
  // This would analyze the generated report to ensure data structures are consistent
  const html = fs.readFileSync(reportPath, 'utf-8');
  const $ = cheerio.load(html);
  
  return {
    hasEnhancedPerformance: $('.metric-card .metric-label:contains("Performance Score")').length > 0,
    hasEnhancedSEO: $('.metric-card .metric-label:contains("SEO Score")').length > 0,
    hasContentWeight: $('.breakdown-type:contains("Total")').length > 0,
    hasMobileFriendliness: $('#mobile-friendliness table').length > 0
  };
}
