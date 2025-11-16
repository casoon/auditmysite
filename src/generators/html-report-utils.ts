/**
 * 📊 HTML Report Utilities
 * 
 * Shared utility functions for HTML report generation.
 * Extracted from html-generator.ts for better maintainability.
 */

export type CertificateLevel = 'basic' | 'enhanced' | 'comprehensive';

/**
 * Calculate WCAG compliance level based on metrics
 */
export function calculateWCAGLevel(score: number, errors: number, warnings: number): string {
  if (errors === 0 && warnings === 0) return 'AAA';
  if (errors === 0 && warnings <= 5) return 'AA';
  if (errors <= 3 && warnings <= 10) return 'A';
  return 'Needs Improvement';
}

/**
 * Calculate ARIA issues count
 */
export function calculateARIAIssues(
  accessibility: any,
  buttonsWithoutLabel: number,
  imagesWithoutAlt: number
): number {
  const ariaErrors = accessibility?.errors?.filter((e: any) => 
    e.code?.includes('aria') || e.message?.toLowerCase().includes('aria')
  ).length || 0;
  
  const ariaWarnings = accessibility?.warnings?.filter((w: any) => 
    w.code?.includes('aria') || w.message?.toLowerCase().includes('aria')
  ).length || 0;

  return ariaErrors + ariaWarnings + buttonsWithoutLabel + imagesWithoutAlt;
}

/**
 * Calculate mobile performance score
 */
export function calculateMobilePerformanceScore(mobilePerf: any): number {
  if (!mobilePerf) return 0;
  
  const metrics = mobilePerf.metrics || mobilePerf;
  const lcp = metrics.largestContentfulPaint || 0;
  const fid = metrics.firstInputDelay || metrics.interactionToNextPaint || 0;
  const cls = metrics.cumulativeLayoutShift || 0;

  let score = 100;
  
  // LCP scoring (50% weight)
  if (lcp > 4000) score -= 50;
  else if (lcp > 2500) score -= 25;
  
  // FID/INP scoring (30% weight)
  if (fid > 300) score -= 30;
  else if (fid > 100) score -= 15;
  
  // CLS scoring (20% weight)
  if (cls > 0.25) score -= 20;
  else if (cls > 0.1) score -= 10;

  return Math.max(0, score);
}

/**
 * Extract specific metric from issues
 */
export function extractMetricFromIssues(accessibility: any, metricType: string): number {
  if (!accessibility) return 0;
  
  const allIssues = [
    ...(accessibility.errors || []),
    ...(accessibility.warnings || [])
  ];

  const metricPatterns: Record<string, RegExp[]> = {
    'color-contrast': [/color.*contrast/i, /insufficient.*contrast/i],
    'keyboard': [/keyboard/i, /focus/i, /tab.*index/i],
    'screen-reader': [/screen.*reader/i, /aria.*label/i, /alt.*text/i],
    'form': [/form.*label/i, /input.*label/i, /form.*field/i]
  };

  const patterns = metricPatterns[metricType] || [];
  return allIssues.filter((issue: any) => 
    patterns.some(pattern => 
      pattern.test(issue.message) || pattern.test(issue.code || '')
    )
  ).length;
}

/**
 * Get icon for issue type
 */
export function getIssueTypeIcon(type: string): string {
  const icons: Record<string, string> = {
    error: '❌',
    warning: '⚠️',
    notice: 'ℹ️',
    info: 'ℹ️'
  };
  return icons[type.toLowerCase()] || '•';
}

/**
 * Extract line number from context/selector
 */
export function extractLineNumber(
  context: string,
  selector: string
): { line: number; column: number } {
  // Try to extract from context if it contains line info
  const lineMatch = context?.match(/line[:\s]+(\d+)/i);
  const colMatch = context?.match(/col(?:umn)?[:\s]+(\d+)/i);

  return {
    line: lineMatch ? parseInt(lineMatch[1], 10) : 0,
    column: colMatch ? parseInt(colMatch[1], 10) : 0
  };
}

/**
 * Format number with commas
 */
export function formatNumber(num: number): string {
  return num.toLocaleString();
}

/**
 * Format bytes to human-readable format
 */
export function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + ' ' + sizes[i];
}

/**
 * Get grade from score
 */
export function getGrade(score: number): string {
  if (score >= 90) return 'A';
  if (score >= 80) return 'B';
  if (score >= 70) return 'C';
  if (score >= 60) return 'D';
  return 'F';
}

/**
 * Get color for grade
 */
export function getGradeColor(grade: string): string {
  const colors: Record<string, string> = {
    'A': '#10b981',
    'B': '#84cc16',
    'C': '#f59e0b',
    'D': '#f97316',
    'F': '#ef4444'
  };
  return colors[grade] || '#6b7280';
}
