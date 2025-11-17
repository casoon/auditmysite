/**
 * Redirect Detection Module
 * Handles HTTP redirect detection and validation for accessibility testing
 */

import { Page, Response } from 'playwright';
import { ILogger } from '../analyzers/interfaces';

/**
 * Redirect detection result
 */
export interface RedirectDetectionResult {
  readonly isRedirect: boolean;
  readonly statusCode: number;
  readonly originalUrl: string;
  readonly finalUrl: string;
  readonly urlChanged: boolean;
  readonly hasRedirectChain: boolean;
  readonly redirectType?: 'http' | 'meta' | 'javascript';
}

/**
 * Configuration for redirect detection
 */
export interface RedirectDetectorConfig {
  readonly skipRedirects?: boolean;
  readonly followRedirects?: boolean;
  readonly logger?: ILogger;
}

/**
 * Default configuration values
 */
const DEFAULT_CONFIG: Required<RedirectDetectorConfig> = {
  skipRedirects: true,
  followRedirects: false,
  logger: console as ILogger
};

/**
 * RedirectDetector - Centralized redirect detection logic
 *
 * Responsibilities:
 * - Detect HTTP redirects (3xx status codes)
 * - Track redirect chains
 * - Validate URL changes
 * - Provide consistent redirect information
 */
export class RedirectDetector {
  private readonly config: Required<RedirectDetectorConfig>;

  constructor(config: RedirectDetectorConfig = {}) {
    this.config = {
      ...DEFAULT_CONFIG,
      ...config
    };
  }

  /**
   * Attach redirect detection listeners to a page
   * Returns a cleanup function to remove listeners
   */
  attachToPage(page: Page): {
    getResult: (response: Response | null, requestedUrl: string) => RedirectDetectionResult;
    cleanup: () => void;
  } {
    let wasRedirectNav = false;
    let redirectStatusCode = 0;

    const onResponse = (res: Response) => {
      try {
        const req = res.request();
        const isNav = req.isNavigationRequest();
        const status = res.status();

        if (isNav && status >= 300 && status < 400) {
          wasRedirectNav = true;
          redirectStatusCode = status;

          this.config.logger.debug('Redirect detected during navigation', {
            url: res.url(),
            status,
            isNavigationRequest: isNav
          });
        }
      } catch (error) {
        // Silently ignore response check errors
        this.config.logger.debug('Error checking response for redirect', error);
      }
    };

    page.on('response', onResponse);

    const cleanup = () => {
      page.off('response', onResponse);
    };

    const getResult = (response: Response | null, requestedUrl: string): RedirectDetectionResult => {
      if (!response) {
        return this.createNonRedirectResult(requestedUrl, requestedUrl);
      }

      const finalUrl = response.url();
      const urlChanged = finalUrl !== requestedUrl;
      const hasRedirectChain = this.hasRedirectChain(response);
      const status = response.status();

      // Only consider it a redirect if:
      // 1. We detected a 3xx status during navigation, OR
      // 2. The response has a redirect chain, OR
      // 3. The final status is 3xx
      const detectedRedirect = wasRedirectNav || hasRedirectChain || (status >= 300 && status < 400);

      // But it's only a REAL redirect if the URL actually changed
      const isRealRedirect = detectedRedirect && urlChanged;

      if (isRealRedirect) {
        this.config.logger.info('Real redirect detected', {
          originalUrl: requestedUrl,
          finalUrl,
          statusCode: redirectStatusCode || status,
          hasRedirectChain
        });
      } else if (detectedRedirect && !urlChanged) {
        this.config.logger.debug('Redirect signals detected but URL unchanged', {
          url: requestedUrl,
          wasRedirectNav,
          hasRedirectChain,
          statusCode: redirectStatusCode || status
        });
      }

      return {
        isRedirect: isRealRedirect,
        statusCode: redirectStatusCode || status,
        originalUrl: requestedUrl,
        finalUrl,
        urlChanged,
        hasRedirectChain,
        redirectType: this.determineRedirectType(status, hasRedirectChain)
      };
    };

    return { getResult, cleanup };
  }

  /**
   * Quick check if a URL redirects (without full page test)
   */
  async checkUrlForRedirect(
    page: Page,
    url: string,
    timeout: number = 5000
  ): Promise<RedirectDetectionResult> {
    const { getResult, cleanup } = this.attachToPage(page);

    try {
      const response = await page.goto(url, {
        waitUntil: 'domcontentloaded',
        timeout
      });

      return getResult(response, url);
    } finally {
      cleanup();
    }
  }

  /**
   * Check if response has a redirect chain
   */
  private hasRedirectChain(response: Response): boolean {
    try {
      const request = response.request();
      // Playwright's Request has a redirectedFrom() method that returns
      // the previous request in the redirect chain, or null if none
      if (typeof (request as any).redirectedFrom === 'function') {
        return !!(request as any).redirectedFrom();
      }
    } catch (error) {
      // Ignore errors checking redirect chain
    }
    return false;
  }

  /**
   * Determine the type of redirect
   */
  private determineRedirectType(
    statusCode: number,
    hasChain: boolean
  ): 'http' | 'meta' | 'javascript' | undefined {
    if (statusCode >= 300 && statusCode < 400) {
      return 'http';
    }
    if (hasChain) {
      // Could be meta refresh or JavaScript redirect
      // We can't easily distinguish without analyzing the page
      return 'meta';
    }
    return undefined;
  }

  /**
   * Create a non-redirect result
   */
  private createNonRedirectResult(url: string, finalUrl: string): RedirectDetectionResult {
    return {
      isRedirect: false,
      statusCode: 200,
      originalUrl: url,
      finalUrl,
      urlChanged: false,
      hasRedirectChain: false
    };
  }

  /**
   * Check if redirects should be skipped based on config
   */
  shouldSkipRedirects(): boolean {
    return this.config.skipRedirects;
  }
}

/**
 * Factory function for convenient creation
 */
export function createRedirectDetector(config?: RedirectDetectorConfig): RedirectDetector {
  return new RedirectDetector(config);
}
