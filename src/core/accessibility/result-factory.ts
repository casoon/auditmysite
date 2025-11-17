/**
 * Result Factory Module
 * Creates consistent AccessibilityResult and PageTestResult objects
 */

import { AccessibilityResult } from '../types';
import { PageTestResult } from './accessibility-checker';
import { RedirectDetectionResult } from './redirect-detector';

/**
 * Configuration for result creation
 */
export interface ResultFactoryConfig {
  readonly url: string;
  readonly title?: string;
  readonly duration?: number;
}

/**
 * ResultFactory - Centralized result object creation
 *
 * Responsibilities:
 * - Create consistent AccessibilityResult objects
 * - Create PageTestResult objects
 * - Handle error scenarios uniformly
 * - Reduce code duplication
 */
export class ResultFactory {
  /**
   * Create a skipped result for redirected URLs
   */
  static createRedirectResult(
    redirectInfo: RedirectDetectionResult,
    duration: number
  ): PageTestResult {
    const { originalUrl, finalUrl, statusCode } = redirectInfo;

    return {
      url: originalUrl,
      title: 'Redirected',
      accessibilityResult: {
        url: originalUrl,
        title: 'Redirected',
        imagesWithoutAlt: 0,
        buttonsWithoutLabel: 0,
        headingsCount: 0,
        errors: [
          `HTTP Redirect detected: ${originalUrl} → ${finalUrl} (${statusCode || 'unknown'})`
        ],
        warnings: [],
        passed: false,
        crashed: false,
        skipped: true,
        duration
      },
      comprehensiveAnalysis: undefined,
      duration,
      timestamp: new Date()
    };
  }

  /**
   * Create an error result for failed tests
   */
  static createErrorResult(
    url: string,
    error: string | Error,
    duration: number,
    crashed: boolean = true
  ): PageTestResult {
    const errorMessage = error instanceof Error ? error.message : error;

    return {
      url,
      title: 'Error',
      accessibilityResult: {
        url,
        title: 'Error',
        imagesWithoutAlt: 0,
        buttonsWithoutLabel: 0,
        headingsCount: 0,
        errors: [`Test failed: ${errorMessage}`],
        warnings: [],
        passed: false,
        crashed,
        duration
      },
      duration,
      timestamp: new Date()
    };
  }

  /**
   * Create a minimal accessibility result (for URL checks)
   */
  static createMinimalResult(config: ResultFactoryConfig): AccessibilityResult {
    return {
      url: config.url,
      title: config.title || 'Untitled',
      imagesWithoutAlt: 0,
      buttonsWithoutLabel: 0,
      headingsCount: 0,
      errors: [],
      warnings: [],
      passed: true,
      crashed: false,
      duration: config.duration || 0
    };
  }

  /**
   * Create a base accessibility result with common defaults
   */
  static createBaseAccessibilityResult(
    url: string,
    title: string
  ): AccessibilityResult {
    return {
      url,
      title,
      imagesWithoutAlt: 0,
      buttonsWithoutLabel: 0,
      headingsCount: 0,
      errors: [],
      warnings: [],
      passed: true,
      duration: 0
    };
  }

  /**
   * Create a 404 result
   */
  static create404Result(url: string, duration: number): AccessibilityResult {
    return {
      url,
      title: 'Not Found',
      imagesWithoutAlt: 0,
      buttonsWithoutLabel: 0,
      headingsCount: 0,
      errors: ['HTTP 404 Not Found'],
      warnings: [],
      passed: false,
      crashed: false,
      skipped: true,
      duration
    };
  }

  /**
   * Create an HTTP error result
   */
  static createHttpErrorResult(
    url: string,
    statusCode: number,
    duration: number
  ): AccessibilityResult {
    const result = this.createMinimalResult({ url, duration });
    result.passed = false;
    result.skipped = true;

    if (statusCode === 404) {
      result.errors.push('HTTP 404 Not Found');
    } else if (statusCode >= 300 && statusCode < 400) {
      result.errors.push(`HTTP ${statusCode} Redirect`);
    } else if (statusCode >= 400) {
      result.errors.push(`HTTP ${statusCode} Error`);
    }

    return result;
  }

  /**
   * Add redirect metadata to result
   */
  static addRedirectInfo(
    result: AccessibilityResult,
    redirectInfo: RedirectDetectionResult
  ): AccessibilityResult {
    if (redirectInfo.isRedirect) {
      (result as any).redirectInfo = {
        status: redirectInfo.statusCode,
        originalUrl: redirectInfo.originalUrl,
        finalUrl: redirectInfo.finalUrl,
        type: redirectInfo.redirectType || 'http_redirect'
      };
    }
    return result;
  }
}
