import { FullAuditResult } from '../types/audit-results';

/**
 * JSON Generator - generates structured JSON as the foundation data format
 * This is the core data structure that feeds HTML and Markdown generators
 */
export class JsonGenerator {
  /**
   * Generate structured JSON audit result
   */
  generateJson(auditData: FullAuditResult): string {
    return JSON.stringify(auditData, null, 2);
  }

  /**
   * Generate JSON for specific pages subset
   */
  generatePageSubset(auditData: FullAuditResult, pageUrls: string[]): string {
    const filteredData = {
      ...auditData,
      pages: auditData.pages.filter(page => pageUrls.includes(page.url))
    };
    return JSON.stringify(filteredData, null, 2);
  }

  /**
   * Generate JSON with only specific metrics (for API responses)
   */
  generateMetricsOnly(auditData: FullAuditResult, metrics: string[]): string {
    const metricsData = {
      metadata: auditData.metadata,
      summary: auditData.summary,
      pages: auditData.pages.map(page => {
        const filteredPage: any = {
          url: page.url,
          title: page.title,
          status: page.status
        };
        
        metrics.forEach(metric => {
          if (page[metric as keyof typeof page]) {
            filteredPage[metric] = page[metric as keyof typeof page];
          }
        });
        
        return filteredPage;
      })
    };
    
    return JSON.stringify(metricsData, null, 2);
  }
}
