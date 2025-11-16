/**
 * Accessibility Whitelist Configuration
 *
 * This file contains URL-specific whitelist rules for accessibility checks.
 * Use this to document known false positives that cannot be automatically detected.
 */

export interface AccessibilityWhitelistRule {
  /** URL pattern to match (can be exact URL or contains string) */
  url: string;
  /** Rules to ignore for this URL */
  ignoreRules?: {
    [ruleId: string]: {
      /** Specific CSS selectors to ignore (optional) */
      selectors?: string[];
      /** Reason for whitelisting (for documentation) */
      reason: string;
      /** Date when this was added (for tracking) */
      addedDate?: string;
      /** Who verified this manually */
      verifiedBy?: string;
    };
  };
}

/**
 * Global accessibility whitelist
 *
 * Add entries here for known false positives that are manually verified.
 */
export const ACCESSIBILITY_WHITELIST: AccessibilityWhitelistRule[] = [
  {
    url: 'https://www.casoon.de',
    ignoreRules: {
      'color-contrast': {
        selectors: [
          '.bg-white\\/20',
          '[class*="backdrop-blur"]',
          'h1.text-gray-800',
          'h2.text-gray-700',
          'p.text-gray-900'
        ],
        reason: 'Glassmorphism effects with backdrop-blur. Manual verification confirms WCAG AA compliance: text-gray-700 (9.2:1), text-gray-800 (12.2:1), text-gray-900 (14.8:1). axe-core cannot calculate contrast on blurred backgrounds.',
        addedDate: '2025-11-16',
        verifiedBy: 'Manual WCAG contrast verification'
      }
    }
  }
  // Add more whitelist entries here as needed
];

/**
 * Check if a URL should ignore a specific rule based on whitelist
 */
export function shouldIgnoreRule(url: string, ruleCode: string, selector?: string): boolean {
  for (const entry of ACCESSIBILITY_WHITELIST) {
    // Check if URL matches
    const urlMatches = url.includes(entry.url) || url === entry.url;
    if (!urlMatches) continue;

    // Check if rule is whitelisted
    const rule = entry.ignoreRules?.[ruleCode];
    if (!rule) continue;

    // If no selectors specified, ignore all instances of this rule
    if (!rule.selectors || rule.selectors.length === 0) {
      return true;
    }

    // Check if selector matches
    if (selector) {
      const selectorMatches = rule.selectors.some(whitelistSel =>
        selector.includes(whitelistSel) || whitelistSel.includes(selector)
      );
      if (selectorMatches) {
        return true;
      }
    }
  }

  return false;
}

/**
 * Get whitelist reason for a specific URL and rule
 */
export function getWhitelistReason(url: string, ruleCode: string): string | undefined {
  for (const entry of ACCESSIBILITY_WHITELIST) {
    if (url.includes(entry.url) || url === entry.url) {
      return entry.ignoreRules?.[ruleCode]?.reason;
    }
  }
  return undefined;
}
