import fs from "fs";
import path from "path";
import { XMLParser } from "fast-xml-parser";
import { SitemapUrl } from "../types";
import { ValidationError } from "../utils/errors";
import { Logger } from "../core/logging/logger";

interface SitemapUrlEntry {
  loc: string;
  lastmod?: string;
  changefreq?: string;
  priority?: number;
}

export class SitemapParser {
  private parser: XMLParser;
  private logger: Logger;
  private readonly MAX_RECURSION_DEPTH = 5;
  private readonly MAX_SUBSITEMAPS = 10;
  private visitedUrls: Set<string> = new Set();

  constructor() {
    this.parser = new XMLParser({
      ignoreAttributes: false,
      attributeNamePrefix: "@_",
    });
    this.logger = new Logger({ level: 'info' });
  }

  async parseSitemap(sitemapUrl: string, depth = 0): Promise<SitemapUrl[]> {
    // Validate recursion depth
    if (depth > this.MAX_RECURSION_DEPTH) {
      this.logger.warn(`Maximum recursion depth (${this.MAX_RECURSION_DEPTH}) reached for sitemap: ${sitemapUrl}`);
      return [];
    }

    // Prevent circular dependencies
    if (this.visitedUrls.has(sitemapUrl)) {
      this.logger.debug(`Skipping already visited sitemap: ${sitemapUrl}`);
      return [];
    }
    this.visitedUrls.add(sitemapUrl);

    // Validate URL format
    if (!this.isValidSitemapUrl(sitemapUrl)) {
      throw new ValidationError('Invalid sitemap URL format', { url: sitemapUrl });
    }
    let xml: string;

    // Lade XML von URL oder Datei
    if (sitemapUrl.startsWith("http")) {
      const response = await fetch(sitemapUrl);
      if (!response.ok) {
        throw new Error(`Failed to fetch sitemap: ${response.statusText}`);
      }
      xml = await response.text();
    } else {
      xml = fs.readFileSync(path.resolve(sitemapUrl), "utf-8");
    }

    const parsed = this.parser.parse(xml);
    const urls: SitemapUrl[] = [];

    // Fall 1: Sitemap Index (WordPress/multi-sitemap structure)
    if (parsed.sitemapindex && parsed.sitemapindex.sitemap) {
      const sitemapCount = Array.isArray(parsed.sitemapindex.sitemap) ? parsed.sitemapindex.sitemap.length : 1;
      this.logger.info(`Found sitemap index with ${sitemapCount} sub-sitemaps`);

      const sitemaps = Array.isArray(parsed.sitemapindex.sitemap)
        ? parsed.sitemapindex.sitemap
        : [parsed.sitemapindex.sitemap];

      // Fetch URLs from each sub-sitemap (limit to MAX_SUBSITEMAPS for performance)
      const sitemapsToProcess = sitemaps.slice(0, this.MAX_SUBSITEMAPS);

      for (const sitemap of sitemapsToProcess) {
        try {
          const subSitemapUrl = sitemap.loc;
          if (subSitemapUrl) {
            this.logger.debug(`Processing sub-sitemap: ${subSitemapUrl}`);
            const subUrls = await this.parseSitemap(subSitemapUrl, depth + 1); // Recursive call
            urls.push(...subUrls);
          }
        } catch (error) {
          this.logger.warn(`Failed to process sub-sitemap ${sitemap.loc}`, error);
          // Continue with other sitemaps even if one fails
        }
      }

      if (sitemaps.length > this.MAX_SUBSITEMAPS) {
        this.logger.info(`Limited processing to first ${this.MAX_SUBSITEMAPS} of ${sitemaps.length} sub-sitemaps for performance`);
      }
      
      return urls;
    }

    // Fall 2: Standard sitemap.xml Struktur
    if (parsed.urlset && parsed.urlset.url) {
      if (Array.isArray(parsed.urlset.url)) {
        urls.push(
          ...parsed.urlset.url.map((u: SitemapUrlEntry) => ({
            loc: u.loc,
            lastmod: u.lastmod,
            changefreq: u.changefreq,
            priority: u.priority,
          })),
        );
      } else {
        urls.push({
          loc: parsed.urlset.url.loc,
          lastmod: parsed.urlset.url.lastmod,
          changefreq: parsed.urlset.url.changefreq,
          priority: parsed.urlset.url.priority,
        });
      }
    }

    // Fall 3: Falls die URLs im #text Feld sind (wie bei Astro)
    if (urls.length === 0 && parsed.urlset && parsed.urlset["#text"]) {
      const textContent = parsed.urlset["#text"];
      const urlMatches = textContent.match(/<loc>(.*?)<\/loc>/g);
      if (urlMatches) {
        urls.push(
          ...urlMatches.map((match: string) => ({
            loc: match.replace(/<\/?loc>/g, ""),
          })),
        );
      }
    }

    return urls;
  }

  filterUrls(
    urls: SitemapUrl[],
    options: {
      filterPatterns?: string[];
      includePatterns?: string[];
    },
  ): SitemapUrl[] {
    let filtered = urls;

    // Filtere nach Ausschluss-Mustern
    if (options.filterPatterns) {
      filtered = filtered.filter(
        (url) =>
          !options.filterPatterns!.some((pattern) => url.loc.includes(pattern)),
      );
    }

    // Filtere nach Einschluss-Mustern
    if (options.includePatterns) {
      filtered = filtered.filter((url) =>
        options.includePatterns!.some((pattern) => url.loc.includes(pattern)),
      );
    }

    return filtered;
  }

  convertToLocalUrls(urls: SitemapUrl[], baseUrl: string): SitemapUrl[] {
    return urls.map((url) => ({
      ...url,
      loc: this.convertUrlToLocal(url.loc, baseUrl),
    }));
  }

  private convertUrlToLocal(url: string, baseUrl: string): string {
    // Extrahiere Domain aus der URL
    const urlObj = new URL(url);
    const domain = urlObj.origin;

    // Ersetze Domain durch baseUrl
    return url.replace(domain, baseUrl);
  }

  private isValidSitemapUrl(url: string): boolean {
    if (!url || typeof url !== 'string') {
      return false;
    }

    // Allow http(s) URLs and file paths
    if (url.startsWith('http://') || url.startsWith('https://')) {
      try {
        new URL(url);
        return true;
      } catch {
        return false;
      }
    }

    // Allow file paths
    return url.length > 0;
  }
}
