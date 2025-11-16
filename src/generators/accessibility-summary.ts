/**
 * 📊 Enhanced Accessibility Summary Generator
 * 
 * Generates detailed, actionable summaries of accessibility audits
 * with clear metrics, categorization, and severity levels.
 */

export interface AccessibilitySummary {
  overview: {
    totalPages: number;
    testedPages: number;
    overallScore: number;
    overallGrade: string;
    wcagLevel: string;
  };
  issues: {
    total: number;
    byType: {
      errors: number;
      warnings: number;
      notices: number;
    };
    bySeverity: {
      critical: number;
      serious: number;
      moderate: number;
      minor: number;
    };
    byCategory: Record<string, number>;
  };
  topIssues: Array<{
    code: string;
    count: number;
    severity: string;
    description: string;
    impact: string;
  }>;
  recommendations: string[];
  metrics: {
    averageIssuesPerPage: number;
    pagesWithErrors: number;
    pagesWithWarnings: number;
    successRate: number;
  };
}

/**
 * Generate enhanced accessibility summary
 */
export function generateAccessibilitySummary(
  pages: Array<{
    url: string;
    accessibility: {
      errors: any[];
      warnings: any[];
      notices?: any[];
      pa11yIssues?: any[];
      score?: number;
    };
  }>
): AccessibilitySummary {
  const totalPages = pages.length;
  const testedPages = pages.filter(p => p.accessibility).length;

  // Collect all issues
  const allIssues: any[] = [];
  const issuesByCode = new Map<string, number>();
  const issuesByCategory = new Map<string, number>();

  let totalErrors = 0;
  let totalWarnings = 0;
  let totalNotices = 0;

  for (const page of pages) {
    if (!page.accessibility) continue;

    const issues = page.accessibility.pa11yIssues || [];
    allIssues.push(...issues);

    totalErrors += page.accessibility.errors?.length || 0;
    totalWarnings += page.accessibility.warnings?.length || 0;
    totalNotices += page.accessibility.notices?.length || 0;

    // Categorize issues
    for (const issue of issues) {
      const code = issue.code || 'unknown';
      issuesByCode.set(code, (issuesByCode.get(code) || 0) + 1);

      const category = categorizeIssue(issue.code);
      issuesByCategory.set(category, (issuesByCategory.get(category) || 0) + 1);
    }
  }

  // Calculate severity distribution
  const bySeverity = {
    critical: allIssues.filter(i => i.impact === 'critical').length,
    serious: allIssues.filter(i => i.impact === 'serious').length,
    moderate: allIssues.filter(i => i.impact === 'moderate').length,
    minor: allIssues.filter(i => i.impact === 'minor' || !i.impact).length
  };

  // Find top issues
  const topIssues = Array.from(issuesByCode.entries())
    .sort((a, b) => b[1] - a[1])
    .slice(0, 5)
    .map(([code, count]) => ({
      code,
      count,
      severity: getSeverityForCode(code),
      description: getDescriptionForCode(code),
      impact: getImpactForCode(code)
    }));

  // Calculate metrics
  const pagesWithErrors = pages.filter(p => 
    p.accessibility && p.accessibility.errors && p.accessibility.errors.length > 0
  ).length;

  const pagesWithWarnings = pages.filter(p => 
    p.accessibility && p.accessibility.warnings && p.accessibility.warnings.length > 0
  ).length;

  const averageIssuesPerPage = totalPages > 0 ? allIssues.length / totalPages : 0;
  const successRate = totalPages > 0 ? ((totalPages - pagesWithErrors) / totalPages) * 100 : 0;

  // Calculate overall score (improved formula)
  const overallScore = calculateOverallScore(totalErrors, totalWarnings, totalNotices, totalPages);
  const overallGrade = getGradeFromScore(overallScore);
  const wcagLevel = determineWCAGLevel(totalErrors, totalWarnings, averageIssuesPerPage);

  // Generate recommendations
  const recommendations = generateRecommendations(topIssues, bySeverity, issuesByCategory);

  return {
    overview: {
      totalPages,
      testedPages,
      overallScore,
      overallGrade,
      wcagLevel
    },
    issues: {
      total: allIssues.length,
      byType: {
        errors: totalErrors,
        warnings: totalWarnings,
        notices: totalNotices
      },
      bySeverity,
      byCategory: Object.fromEntries(issuesByCategory)
    },
    topIssues,
    recommendations,
    metrics: {
      averageIssuesPerPage: Math.round(averageIssuesPerPage * 10) / 10,
      pagesWithErrors,
      pagesWithWarnings,
      successRate: Math.round(successRate * 10) / 10
    }
  };
}

/**
 * Improved overall score calculation
 */
function calculateOverallScore(
  errors: number,
  warnings: number,
  notices: number,
  pages: number
): number {
  if (pages === 0) return 0;

  // Start with perfect score
  let score = 100;

  // Deduct points per page (normalized)
  const errorsPerPage = errors / pages;
  const warningsPerPage = warnings / pages;
  const noticesPerPage = notices / pages;

  // Progressive penalties (diminishing returns for many issues)
  score -= Math.min(50, errorsPerPage * 5);     // Max 50 points for errors
  score -= Math.min(30, warningsPerPage * 2);   // Max 30 points for warnings
  score -= Math.min(10, noticesPerPage * 1);    // Max 10 points for notices

  // Additional penalty for widespread issues
  if (errorsPerPage > 20) score -= 10;
  if (errorsPerPage > 50) score -= 10;

  return Math.max(0, Math.round(score));
}

/**
 * Determine WCAG compliance level
 */
function determineWCAGLevel(errors: number, warnings: number, avgIssuesPerPage: number): string {
  if (errors === 0 && warnings === 0) return 'AAA';
  if (errors === 0 && avgIssuesPerPage < 5) return 'AA';
  if (avgIssuesPerPage < 10) return 'A';
  if (avgIssuesPerPage < 20) return 'Partial Compliance';
  return 'Non-Compliant';
}

/**
 * Get grade from score
 */
function getGradeFromScore(score: number): string {
  if (score >= 90) return 'A';
  if (score >= 80) return 'B';
  if (score >= 70) return 'C';
  if (score >= 60) return 'D';
  return 'F';
}

/**
 * Categorize issue by code
 */
function categorizeIssue(code: string): string {
  const categories: Record<string, string> = {
    'color-contrast': 'Visual Design',
    'button-name': 'Interactive Elements',
    'link-name': 'Navigation',
    'image-alt': 'Images & Media',
    'label': 'Forms',
    'aria-': 'ARIA & Semantics',
    'heading-': 'Document Structure',
    'landmark': 'Page Regions',
    'html-': 'HTML Validation'
  };

  for (const [prefix, category] of Object.entries(categories)) {
    if (code.includes(prefix)) return category;
  }

  return 'Other';
}

/**
 * Get severity for issue code
 */
function getSeverityForCode(code: string): string {
  const criticalCodes = ['color-contrast', 'button-name', 'link-name'];
  const seriousCodes = ['image-alt', 'label', 'aria-required'];
  
  if (criticalCodes.some(c => code.includes(c))) return 'Critical';
  if (seriousCodes.some(c => code.includes(c))) return 'Serious';
  return 'Moderate';
}

/**
 * Get human-readable description for code
 */
function getDescriptionForCode(code: string): string {
  const descriptions: Record<string, string> = {
    'color-contrast': 'Text does not have sufficient color contrast',
    'button-name': 'Buttons are missing accessible names',
    'link-name': 'Links are missing accessible names',
    'image-alt': 'Images are missing alternative text',
    'label': 'Form inputs are missing labels',
    'aria-required-attr': 'ARIA attributes are missing or incorrect'
  };

  return descriptions[code] || code;
}

/**
 * Get impact description for code
 */
function getImpactForCode(code: string): string {
  const impacts: Record<string, string> = {
    'color-contrast': 'Users with low vision cannot read text',
    'button-name': 'Screen reader users cannot identify button purpose',
    'link-name': 'Screen reader users cannot identify link destination',
    'image-alt': 'Screen reader users miss important image content',
    'label': 'Form fields are unusable for screen reader users'
  };

  return impacts[code] || 'Affects accessibility for some users';
}

/**
 * Generate actionable recommendations
 */
function generateRecommendations(
  topIssues: any[],
  bySeverity: any,
  byCategory: Map<string, number>
): string[] {
  const recommendations: string[] = [];

  // Priority 1: Critical issues
  if (bySeverity.critical > 0) {
    recommendations.push(
      `🔴 Address ${bySeverity.critical} critical issues immediately - these prevent users from accessing content`
    );
  }

  // Priority 2: Most common issues
  if (topIssues.length > 0) {
    const topIssue = topIssues[0];
    if (topIssue.code === 'color-contrast') {
      recommendations.push(
        `🎨 Fix color contrast issues (${topIssue.count} instances) - use darker colors on light backgrounds (e.g., #1F2937 instead of #374151)`
      );
    } else {
      recommendations.push(
        `🔧 Fix "${topIssue.description}" (${topIssue.count} instances) - this is your most common issue`
      );
    }
  }

  // Priority 3: Category-specific
  const topCategory = Array.from(byCategory.entries()).sort((a, b) => b[1] - a[1])[0];
  if (topCategory && topCategory[1] > 10) {
    recommendations.push(
      `📋 Focus on "${topCategory[0]}" category (${topCategory[1]} issues) - consider a comprehensive review`
    );
  }

  // Priority 4: Serious issues
  if (bySeverity.serious > 5) {
    recommendations.push(
      `⚠️  ${bySeverity.serious} serious issues need attention - these significantly impact user experience`
    );
  }

  // Priority 5: General improvements
  if (recommendations.length < 3) {
    recommendations.push(
      '✅ Consider automated accessibility testing in your CI/CD pipeline',
      '📚 Review WCAG 2.1 guidelines for comprehensive compliance'
    );
  }

  return recommendations.slice(0, 5); // Max 5 recommendations
}

/**
 * Format summary as markdown
 */
export function formatSummaryAsMarkdown(summary: AccessibilitySummary): string {
  return `
# 📊 Accessibility Audit Summary

## Overview
- **Pages Tested:** ${summary.overview.testedPages} / ${summary.overview.totalPages}
- **Overall Score:** ${summary.overview.overallScore}/100 (Grade: ${summary.overview.overallGrade})
- **WCAG Level:** ${summary.overview.wcagLevel}
- **Success Rate:** ${summary.metrics.successRate}%

## Issues Found

### By Type
- ❌ **Errors:** ${summary.issues.byType.errors}
- ⚠️  **Warnings:** ${summary.issues.byType.warnings}
- ℹ️  **Notices:** ${summary.issues.byType.notices}

### By Severity
- 🔴 **Critical:** ${summary.issues.bySeverity.critical}
- 🟠 **Serious:** ${summary.issues.bySeverity.serious}
- 🟡 **Moderate:** ${summary.issues.bySeverity.moderate}
- 🟢 **Minor:** ${summary.issues.bySeverity.minor}

## Top Issues

${summary.topIssues.map((issue, i) => `
${i + 1}. **${issue.description}**
   - Code: \`${issue.code}\`
   - Count: ${issue.count} instances
   - Severity: ${issue.severity}
   - Impact: ${issue.impact}
`).join('')}

## Recommendations

${summary.recommendations.map((rec, i) => `${i + 1}. ${rec}`).join('\n')}

## Metrics
- Average Issues per Page: ${summary.metrics.averageIssuesPerPage}
- Pages with Errors: ${summary.metrics.pagesWithErrors} / ${summary.overview.totalPages}
- Pages with Warnings: ${summary.metrics.pagesWithWarnings} / ${summary.overview.totalPages}
`;
}
