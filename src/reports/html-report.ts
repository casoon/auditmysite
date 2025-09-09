import { htmlReportTemplate } from './html-template';
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

  let html = htmlReportTemplate
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
