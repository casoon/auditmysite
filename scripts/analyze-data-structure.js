#!/usr/bin/env node

/**
 * Data Structure Analysis Tool
 * 
 * Analyzes the current data flow from CLI to HTML reports
 * Identifies missing data mappings and inconsistencies
 * Provides recommendations for consolidation
 */

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

class DataStructureAnalyzer {
  constructor() {
    this.results = {
      cliData: {},
      htmlData: {},
      missingMappings: [],
      inconsistencies: [],
      recommendations: []
    };
  }

  async analyzeFullDataFlow() {
    console.log('🔍 Starting comprehensive data structure analysis...\n');

    // Step 1: Run analysis and capture CLI output
    await this.captureCliData();
    
    // Step 2: Analyze generated HTML report
    await this.analyzeHtmlReport();
    
    // Step 3: Compare and identify gaps
    this.identifyDataGaps();
    
    // Step 4: Analyze code structure
    this.analyzeCodeStructure();
    
    // Step 5: Generate report
    this.generateReport();
  }

  async captureCliData() {
    console.log('📊 Capturing CLI analysis data...');
    
    try {
      const output = execSync(
        'node bin/audit.js https://example.com/sitemap.xml --max-pages 1 --non-interactive',
        { encoding: 'utf-8', cwd: process.cwd() }
      );
      
      this.results.cliData = this.parseCliOutput(output);
      console.log('✅ CLI data captured');
      
    } catch (error) {
      console.error('❌ Failed to capture CLI data:', error.message);
      this.results.cliData = {};
    }
  }

  parseCliOutput(output) {
    const data = {
      metrics: {},
      features: {},
      rawOutput: output
    };

    // Extract all numeric metrics
    const patterns = {
      performanceScore: /Performance Score: (\d+)\/100/,
      seoScore: /SEO Score: (\d+)\/100/,
      mobileScore: /Mobile Score: (\d+)\/100/,
      qualityScore: /Quality: (\d+)\/100/,
      lcp: /LCP: (\d+)ms/,
      cls: /CLS: ([\d.]+)/,
      inp: /INP: (\d+)ms/,
      ttfb: /TTFB: (\d+)ms/,
      contentWeight: /Total page weight: ([\d.]+)\s*(MB|GB|KB|B)/,
      textToCodeRatio: /Text-to-code ratio: ([\d.]+)%/,
      wordCount: /Word Count: (\d+)/,
      readability: /Readability: ([\d.]+)/,
      errors: /⚠️\s+Errors: (\d+)/,
      warnings: /⚠️\s+Warnings: (\d+)/,
      testedPages: /📄 Tested: (\d+) pages/,
      passedPages: /✅ Passed: (\d+)/,
      failedPages: /❌ Failed: (\d+)/,
      successRate: /🎯 Success Rate: ([\d.]+)%/,
      duration: /⚡ Average speed: ([\d.]+) pages\/minute/
    };

    Object.entries(patterns).forEach(([key, pattern]) => {
      const match = output.match(pattern);
      if (match) {
        if (key === 'contentWeight') {
          data.metrics[key] = { value: parseFloat(match[1]), unit: match[2] };
        } else {
          data.metrics[key] = parseFloat(match[1]);
        }
      }
    });

    // Extract feature flags
    const featurePatterns = {
      performance: /⚡ Performance: ✅/,
      seo: /🔍 SEO: ✅/,
      contentWeight: /📏 Content Weight: ✅/,
      mobileFriendliness: /📱 Mobile-Friendliness: ✅/
    };

    Object.entries(featurePatterns).forEach(([key, pattern]) => {
      data.features[key] = pattern.test(output);
    });

    // Extract available data fields
    const dataFieldsMatch = output.match(/pageKeys: \[(.*?)\]/);
    if (dataFieldsMatch) {
      data.availableFields = dataFieldsMatch[1]
        .split(',')
        .map(field => field.trim().replace(/['"]/g, ''));
    }

    return data;
  }

  async analyzeHtmlReport() {
    console.log('📝 Analyzing HTML report data...');
    
    try {
      const reportPath = this.findLatestReport();
      if (!reportPath) {
        console.warn('⚠️ No HTML report found');
        return;
      }

      const html = fs.readFileSync(reportPath, 'utf-8');
      this.results.htmlData = this.parseHtmlReport(html);
      this.results.htmlData.filePath = reportPath;
      
      console.log('✅ HTML data analyzed');
      
    } catch (error) {
      console.error('❌ Failed to analyze HTML report:', error.message);
    }
  }

  parseHtmlReport(html) {
    const data = {
      sections: {},
      metrics: {},
      tables: {},
      missingValues: []
    };

    // Use basic regex parsing instead of cheerio for this analysis
    const sections = ['accessibility', 'performance', 'seo', 'mobile-friendliness', 'detailed-issues'];
    
    sections.forEach(section => {
      const sectionRegex = new RegExp(`<section id="${section}"[^>]*>(.*?)</section>`, 's');
      const match = html.match(sectionRegex);
      
      data.sections[section] = {
        present: !!match,
        hasTable: match ? /<table/.test(match[1]) : false,
        hasMetricCards: match ? /metric-card/.test(match[1]) : false,
        naCount: match ? (match[1].match(/N\/A/g) || []).length : 0
      };
    });

    // Extract displayed metrics
    const metricPatterns = {
      performanceScore: /<div class="metric-label">Performance Score<\/div>\s*<div class="metric-value">(\d+)</,
      seoScore: /<div class="metric-label">SEO Score<\/div>\s*<div class="metric-value">(\d+)</,
      contentWeight: /<div class="breakdown-size">([^<]+)</,
      lcp: /<div class="metric-label">LCP[^<]*<\/div>\s*<div class="metric-value">(\d+)ms</,
      errors: /<div class="kpi-value">(\d+)<\/div>/,
      pa11yScore: /<td>([^<]*\/100|N\/A)<\/td>/
    };

    Object.entries(metricPatterns).forEach(([key, pattern]) => {
      const matches = html.match(pattern);
      if (matches) {
        data.metrics[key] = matches[1];
      }
    });

    // Count N/A values in different sections
    data.naAnalysis = {
      accessibility: (html.match(/<section id="accessibility"[^>]*>.*?<\/section>/s)?.[0]?.match(/N\/A/g) || []).length,
      performance: (html.match(/<section id="performance"[^>]*>.*?<\/section>/s)?.[0]?.match(/N\/A/g) || []).length,
      seo: (html.match(/<section id="seo"[^>]*>.*?<\/section>/s)?.[0]?.match(/N\/A/g) || []).length,
      mobile: (html.match(/<section id="mobile-friendliness"[^>]*>.*?<\/section>/s)?.[0]?.match(/N\/A/g) || []).length
    };

    return data;
  }

  identifyDataGaps() {
    console.log('🔍 Identifying data gaps and inconsistencies...');

    const cliMetrics = this.results.cliData.metrics || {};
    const htmlMetrics = this.results.htmlData.metrics || {};

    // Check for missing mappings
    Object.keys(cliMetrics).forEach(metric => {
      if (!htmlMetrics[metric] || htmlMetrics[metric] === 'N/A' || htmlMetrics[metric] === '0') {
        this.results.missingMappings.push({
          metric,
          cliValue: cliMetrics[metric],
          htmlValue: htmlMetrics[metric] || 'missing',
          severity: this.calculateSeverity(metric, cliMetrics[metric])
        });
      }
    });

    // Check for inconsistencies
    Object.keys(htmlMetrics).forEach(metric => {
      if (cliMetrics[metric] && htmlMetrics[metric]) {
        const cliVal = parseFloat(cliMetrics[metric]);
        const htmlVal = parseFloat(htmlMetrics[metric]);
        
        if (!isNaN(cliVal) && !isNaN(htmlVal) && Math.abs(cliVal - htmlVal) > 0.1) {
          this.results.inconsistencies.push({
            metric,
            cliValue: cliVal,
            htmlValue: htmlVal,
            difference: Math.abs(cliVal - htmlVal)
          });
        }
      }
    });

    console.log(`📊 Found ${this.results.missingMappings.length} missing mappings`);
    console.log(`⚠️ Found ${this.results.inconsistencies.length} inconsistencies`);
  }

  calculateSeverity(metric, value) {
    // High severity for metrics that have actual values but show as missing
    const highPriorityMetrics = ['performanceScore', 'seoScore', 'contentWeight', 'mobileScore'];
    if (highPriorityMetrics.includes(metric) && value > 0) {
      return 'HIGH';
    }
    return 'MEDIUM';
  }

  analyzeCodeStructure() {
    console.log('🔧 Analyzing code structure...');

    const generatorPath = 'src/generators/html-generator.ts';
    const reportPath = 'src/reports/html-report.ts';

    try {
      const generatorCode = fs.readFileSync(generatorPath, 'utf-8');
      const reportCode = fs.readFileSync(reportPath, 'utf-8');

      // Analyze method patterns
      const methods = {
        basic: generatorCode.match(/generate\w+Section/g) || [],
        enhanced: generatorCode.match(/generateEnhanced\w+/g) || [],
        fallback: generatorCode.match(/generateBasic\w+/g) || []
      };

      // Check for inconsistent field access patterns
      const fieldPatterns = [
        /page\.enhancedPerformance/g,
        /page\.enhancedSEO/g,
        /page\.enhancedSeo/g,
        /page\.contentWeight/g,
        /page\.mobileFriendliness/g,
        /page\.issues\?/g
      ];

      const fieldUsage = {};
      fieldPatterns.forEach((pattern, index) => {
        const matches = generatorCode.match(pattern) || [];
        fieldUsage[pattern.source] = matches.length;
      });

      this.results.codeStructure = {
        methods,
        fieldUsage,
        inconsistentFieldAccess: this.findInconsistentFieldAccess(generatorCode)
      };

    } catch (error) {
      console.error('❌ Failed to analyze code structure:', error.message);
    }
  }

  findInconsistentFieldAccess(code) {
    const issues = [];
    
    // Check for mixed field naming (enhancedSEO vs enhancedSeo)
    if (code.includes('enhancedSEO') && code.includes('enhancedSeo')) {
      issues.push('Mixed case in SEO field names (enhancedSEO vs enhancedSeo)');
    }

    // Check for redundant method calls
    const methodCalls = code.match(/this\.generate\w+\(/g) || [];
    const uniqueCalls = [...new Set(methodCalls)];
    if (methodCalls.length > uniqueCalls.length) {
      issues.push('Potential redundant method calls detected');
    }

    return issues;
  }

  generateReport() {
    console.log('\n📋 Generating comprehensive analysis report...\n');

    // Summary
    console.log('='.repeat(60));
    console.log('📊 DATA STRUCTURE ANALYSIS REPORT');
    console.log('='.repeat(60));

    console.log('\n🎯 EXECUTIVE SUMMARY:');
    console.log(`   • Missing Mappings: ${this.results.missingMappings.length}`);
    console.log(`   • Data Inconsistencies: ${this.results.inconsistencies.length}`);
    console.log(`   • Code Structure Issues: ${this.results.codeStructure?.inconsistentFieldAccess?.length || 0}`);

    // Detailed findings
    console.log('\n🚨 HIGH PRIORITY ISSUES:');
    const highPriorityIssues = this.results.missingMappings.filter(m => m.severity === 'HIGH');
    if (highPriorityIssues.length === 0) {
      console.log('   ✅ No high priority issues found');
    } else {
      highPriorityIssues.forEach(issue => {
        console.log(`   • ${issue.metric}: CLI shows ${issue.cliValue}, HTML shows ${issue.htmlValue}`);
      });
    }

    console.log('\n⚠️ DATA MAPPING ISSUES:');
    this.results.missingMappings.forEach(issue => {
      console.log(`   • ${issue.metric}: ${issue.cliValue} -> ${issue.htmlValue} (${issue.severity})`);
    });

    console.log('\n📈 INCONSISTENCIES:');
    this.results.inconsistencies.forEach(issue => {
      console.log(`   • ${issue.metric}: CLI=${issue.cliValue}, HTML=${issue.htmlValue} (diff: ${issue.difference})`);
    });

    console.log('\n🔧 CODE STRUCTURE:');
    if (this.results.codeStructure) {
      console.log(`   • Basic methods: ${this.results.codeStructure.methods.basic.length}`);
      console.log(`   • Enhanced methods: ${this.results.codeStructure.methods.enhanced.length}`);
      console.log(`   • Field access issues: ${this.results.codeStructure.inconsistentFieldAccess.length}`);
      
      this.results.codeStructure.inconsistentFieldAccess.forEach(issue => {
        console.log(`     - ${issue}`);
      });
    }

    console.log('\n🛠️ RECOMMENDATIONS:');
    this.generateRecommendations();
    
    this.recommendations.forEach((rec, index) => {
      console.log(`   ${index + 1}. ${rec}`);
    });

    console.log('\n' + '='.repeat(60));
    console.log('📁 Report saved to: analysis-report.json');
    
    // Save detailed report
    fs.writeFileSync(
      'analysis-report.json',
      JSON.stringify(this.results, null, 2)
    );
  }

  generateRecommendations() {
    this.recommendations = [];

    // Based on missing mappings
    if (this.results.missingMappings.length > 0) {
      this.recommendations.push('Standardize enhanced data field access patterns in html-generator.ts');
      this.recommendations.push('Add fallback logic for missing enhanced data');
      this.recommendations.push('Create unified data transformation layer between CLI and HTML generation');
    }

    // Based on code structure
    if (this.results.codeStructure?.inconsistentFieldAccess?.length > 0) {
      this.recommendations.push('Consolidate field naming conventions (use enhancedSEO consistently)');
      this.recommendations.push('Refactor generator methods to use consistent data access patterns');
    }

    // Based on N/A values
    if (this.results.htmlData?.naAnalysis) {
      const totalNAs = Object.values(this.results.htmlData.naAnalysis).reduce((a, b) => a + b, 0);
      if (totalNAs > 5) {
        this.recommendations.push('Implement pa11y score calculation algorithm');
        this.recommendations.push('Add data validation layer to ensure enhanced data is properly structured');
      }
    }

    // General recommendations
    this.recommendations.push('Add automated tests to validate CLI-to-HTML data mapping');
    this.recommendations.push('Create data structure documentation');
    this.recommendations.push('Implement debug logging for data transformation pipeline');
  }

  findLatestReport() {
    try {
      const reportsDir = 'reports';
      const todayDate = new Date().toISOString().split('T')[0];
      
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
    } catch (error) {
      console.warn('Could not find latest report:', error.message);
    }
    
    return null;
  }
}

// Run the analyzer if called directly
if (require.main === module) {
  const analyzer = new DataStructureAnalyzer();
  analyzer.analyzeFullDataFlow().catch(console.error);
}

module.exports = DataStructureAnalyzer;
