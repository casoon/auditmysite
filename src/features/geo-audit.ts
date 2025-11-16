/**
 * 🌍 Geo-Audit Feature
 * 
 * Performs website audits from different geographic locations
 * to test regional performance, content variations, and accessibility.
 */

import { Page, Browser } from 'playwright';

export interface GeoLocation {
  name: string;
  locale: string;
  timezone: string;
  latitude: number;
  longitude: number;
  acceptLanguage?: string;
}

export interface GeoAuditOptions {
  locations: GeoLocation[];
  url: string;
  viewport?: { width: number; height: number };
  timeout?: number;
  collectMetrics?: boolean;
}

export interface GeoAuditResult {
  location: GeoLocation;
  url: string;
  timestamp: string;
  performance: {
    loadTime: number;
    firstContentfulPaint?: number;
    largestContentfulPaint?: number;
    timeToInteractive?: number;
  };
  content: {
    language?: string;
    currency?: string;
    pricing?: string;
    contentDifferences?: string[];
  };
  accessibility: {
    errors: number;
    warnings: number;
    locationSpecificIssues?: string[];
  };
  seo: {
    hreflang?: string[];
    localizedMeta?: boolean;
    geoTargeting?: string;
  };
}

/**
 * Pre-defined geographic locations for testing
 */
export const COMMON_LOCATIONS: Record<string, GeoLocation> = {
  'germany-berlin': {
    name: 'Germany (Berlin)',
    locale: 'de-DE',
    timezone: 'Europe/Berlin',
    latitude: 52.52,
    longitude: 13.405,
    acceptLanguage: 'de-DE,de;q=0.9,en;q=0.8'
  },
  'usa-newyork': {
    name: 'USA (New York)',
    locale: 'en-US',
    timezone: 'America/New_York',
    latitude: 40.7128,
    longitude: -74.006,
    acceptLanguage: 'en-US,en;q=0.9'
  },
  'uk-london': {
    name: 'UK (London)',
    locale: 'en-GB',
    timezone: 'Europe/London',
    latitude: 51.5074,
    longitude: -0.1278,
    acceptLanguage: 'en-GB,en;q=0.9'
  },
  'france-paris': {
    name: 'France (Paris)',
    locale: 'fr-FR',
    timezone: 'Europe/Paris',
    latitude: 48.8566,
    longitude: 2.3522,
    acceptLanguage: 'fr-FR,fr;q=0.9,en;q=0.8'
  },
  'japan-tokyo': {
    name: 'Japan (Tokyo)',
    locale: 'ja-JP',
    timezone: 'Asia/Tokyo',
    latitude: 35.6762,
    longitude: 139.6503,
    acceptLanguage: 'ja-JP,ja;q=0.9,en;q=0.8'
  },
  'australia-sydney': {
    name: 'Australia (Sydney)',
    locale: 'en-AU',
    timezone: 'Australia/Sydney',
    latitude: -33.8688,
    longitude: 151.2093,
    acceptLanguage: 'en-AU,en;q=0.9'
  }
};

/**
 * Perform geo-located audit
 */
export async function performGeoAudit(
  browser: Browser,
  options: GeoAuditOptions
): Promise<GeoAuditResult[]> {
  const results: GeoAuditResult[] = [];

  for (const location of options.locations) {
    const result = await auditFromLocation(browser, options.url, location, options);
    results.push(result);
  }

  return results;
}

/**
 * Audit from a specific geographic location
 */
async function auditFromLocation(
  browser: Browser,
  url: string,
  location: GeoLocation,
  options: GeoAuditOptions
): Promise<GeoAuditResult> {
  const context = await browser.newContext({
    locale: location.locale,
    timezoneId: location.timezone,
    geolocation: {
      latitude: location.latitude,
      longitude: location.longitude
    },
    permissions: ['geolocation'],
    viewport: options.viewport || { width: 1920, height: 1080 },
    extraHTTPHeaders: location.acceptLanguage ? {
      'Accept-Language': location.acceptLanguage
    } : undefined
  });

  const page = await context.newPage();
  const startTime = Date.now();

  try {
    // Navigate to URL
    await page.goto(url, {
      timeout: options.timeout || 30000,
      waitUntil: 'load'
    });

    // Collect performance metrics
    const performance = await collectPerformanceMetrics(page, startTime);

    // Analyze content variations
    const content = await analyzeContent(page);

    // Basic accessibility check
    const accessibility = await checkAccessibility(page);

    // SEO geo-targeting check
    const seo = await checkGeoSEO(page);

    await context.close();

    return {
      location,
      url,
      timestamp: new Date().toISOString(),
      performance,
      content,
      accessibility,
      seo
    };
  } catch (error) {
    await context.close();
    throw new Error(`Geo-audit failed for ${location.name}: ${error}`);
  }
}

/**
 * Collect performance metrics
 */
async function collectPerformanceMetrics(
  page: Page,
  startTime: number
): Promise<GeoAuditResult['performance']> {
  const loadTime = Date.now() - startTime;

  try {
    const metrics = await page.evaluate(() => {
      const perfData = performance.getEntriesByType('navigation')[0] as PerformanceNavigationTiming;
      const paintData = performance.getEntriesByType('paint');

      return {
        firstContentfulPaint: paintData.find(p => p.name === 'first-contentful-paint')?.startTime,
        largestContentfulPaint: 0, // Would need observer
        timeToInteractive: perfData?.domInteractive || 0
      };
    });

    return {
      loadTime,
      ...metrics
    };
  } catch {
    return { loadTime };
  }
}

/**
 * Analyze content variations by location
 */
async function analyzeContent(page: Page): Promise<GeoAuditResult['content']> {
  try {
    const content = await page.evaluate(() => {
      const html = document.documentElement.lang;
      const currency = document.body.textContent?.match(/[$€£¥]/)?.[0];
      
      return {
        language: html || document.querySelector('html')?.getAttribute('lang') || undefined,
        currency,
        pricing: document.body.textContent?.match(/\d+[.,]\d{2}\s*[$€£¥]/)?.[0]
      };
    });

    return content;
  } catch {
    return {};
  }
}

/**
 * Basic accessibility check
 */
async function checkAccessibility(page: Page): Promise<GeoAuditResult['accessibility']> {
  try {
    const issues = await page.evaluate(() => {
      const errors: string[] = [];
      const warnings: string[] = [];

      // Check for missing alt text
      const imagesWithoutAlt = document.querySelectorAll('img:not([alt])').length;
      if (imagesWithoutAlt > 0) {
        warnings.push(`${imagesWithoutAlt} images without alt text`);
      }

      // Check for language attribute
      if (!document.documentElement.lang) {
        errors.push('Missing language attribute on html element');
      }

      return { errors: errors.length, warnings: warnings.length };
    });

    return issues;
  } catch {
    return { errors: 0, warnings: 0 };
  }
}

/**
 * Check SEO geo-targeting
 */
async function checkGeoSEO(page: Page): Promise<GeoAuditResult['seo']> {
  try {
    const seo = await page.evaluate(() => {
      const hreflang = Array.from(document.querySelectorAll('link[hreflang]'))
        .map(link => link.getAttribute('hreflang'))
        .filter(Boolean) as string[];

      const hasLocalizedMeta = !!document.querySelector('meta[name="geo.region"]') ||
                              !!document.querySelector('meta[name="geo.placename"]');

      const geoTargeting = document.querySelector('meta[name="geo.region"]')?.getAttribute('content') || undefined;

      return {
        hreflang: hreflang.length > 0 ? hreflang : undefined,
        localizedMeta: hasLocalizedMeta,
        geoTargeting
      };
    });

    return seo;
  } catch {
    return {};
  }
}

/**
 * Compare results across locations
 */
export function compareGeoResults(results: GeoAuditResult[]): {
  performanceVariance: number;
  contentDifferences: string[];
  recommendations: string[];
} {
  if (results.length < 2) {
    return {
      performanceVariance: 0,
      contentDifferences: [],
      recommendations: []
    };
  }

  // Calculate performance variance
  const loadTimes = results.map(r => r.performance.loadTime);
  const avgLoadTime = loadTimes.reduce((a, b) => a + b, 0) / loadTimes.length;
  const variance = Math.max(...loadTimes) - Math.min(...loadTimes);
  const performanceVariance = (variance / avgLoadTime) * 100;

  // Find content differences
  const languages = new Set(results.map(r => r.content.language).filter(Boolean));
  const currencies = new Set(results.map(r => r.content.currency).filter(Boolean));
  
  const contentDifferences: string[] = [];
  if (languages.size > 1) {
    contentDifferences.push(`Multiple languages detected: ${Array.from(languages).join(', ')}`);
  }
  if (currencies.size > 1) {
    contentDifferences.push(`Multiple currencies detected: ${Array.from(currencies).join(', ')}`);
  }

  // Generate recommendations
  const recommendations: string[] = [];
  
  if (performanceVariance > 50) {
    recommendations.push('Consider using a CDN to reduce geographic performance variance');
  }

  const hasHreflang = results.some(r => r.seo.hreflang && r.seo.hreflang.length > 0);
  if (languages.size > 1 && !hasHreflang) {
    recommendations.push('Add hreflang tags for multi-language support');
  }

  const hasGeoTargeting = results.some(r => r.seo.geoTargeting);
  if (!hasGeoTargeting && results.length > 1) {
    recommendations.push('Consider adding geo-targeting meta tags for regional content');
  }

  return {
    performanceVariance,
    contentDifferences,
    recommendations
  };
}
