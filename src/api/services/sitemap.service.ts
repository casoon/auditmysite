import { SitemapParser } from '../../core/parsers';
import { SitemapResult } from '../../types/audit-results';

/**
 * SitemapService - Returns SitemapResult (same type used in FullAuditResult)
 * Used by API endpoint: GET /api/v2/sitemap/:domain
 */
export class SitemapService {
  private parser = new SitemapParser();

  async getUrls(sitemapUrl: string): Promise<SitemapResult> {
    try {
      // Parse sitemap
      const rawUrls = await this.parser.parseSitemap(sitemapUrl);
      const allUrls = rawUrls.map((url: any) => url.loc || url);
      
      // Apply basic filters (same as pipeline)
      const filterPatterns = ['[...slug]', '[category]', '/demo/', '/test/'];
      const filteredUrls = this.parser.filterUrls(
        allUrls.map(url => ({ loc: url })), 
        { filterPatterns }
      );
      
      const finalUrls = filteredUrls.map((url: any) => url.loc);
      
      return {
        sourceUrl: sitemapUrl,
        urls: finalUrls,
        parsedAt: new Date().toISOString(),
        totalUrls: allUrls.length,
        filteredUrls: allUrls.length - filteredUrls.length,
        filterPatterns
      };
    } catch (error) {
      throw new Error(`Failed to parse sitemap: ${(error as Error).message}`);
    }
  }

  /**
   * Get domain from sitemap URL for convenience
   */
  async getUrlsFromDomain(domain: string): Promise<SitemapResult> {
    // Try common sitemap paths
    const sitemapPaths = [
      `https://${domain}/sitemap.xml`,
      `http://${domain}/sitemap.xml`,
      `https://${domain}/sitemap_index.xml`
    ];

    for (const sitemapUrl of sitemapPaths) {
      try {
        return await this.getUrls(sitemapUrl);
      } catch (error) {
        // Continue to next path
      }
    }
    
    throw new Error(`No sitemap found for domain: ${domain}`);
  }
}
