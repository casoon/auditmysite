/**
 * Constants for Accessibility Testing
 * Centralized configuration values to avoid magic numbers
 */

/**
 * Timeout configurations (in milliseconds)
 */
export const TIMEOUTS = {
  /** Default page navigation timeout */
  DEFAULT_NAVIGATION: 30000,

  /** Pa11y test timeout */
  PA11Y_TEST: 15000,

  /** Minimal URL test timeout */
  MINIMAL_TEST: 5000,

  /** Wait time before running pa11y */
  PA11Y_WAIT: 1000,

  /** Queue item timeout */
  QUEUE_ITEM: 30000,

  /** Browser idle timeout */
  BROWSER_IDLE: 30000
} as const;

/**
 * Concurrency configurations
 */
export const CONCURRENCY = {
  /** Default maximum concurrent tests */
  DEFAULT_MAX: 3,

  /** Maximum browser pool size */
  MAX_POOL_SIZE: 5,

  /** Initial browser warmup count */
  WARMUP_COUNT: 1
} as const;

/**
 * Retry configurations
 */
export const RETRY = {
  /** Maximum retry attempts for failed tests */
  MAX_ATTEMPTS: 3,

  /** Delay between retries (in milliseconds) */
  DELAY_MS: 2000
} as const;

/**
 * Scoring configurations
 */
export const SCORING = {
  /** Points deducted per pa11y error */
  ERROR_PENALTY: 2.5,

  /** Points deducted per pa11y warning */
  WARNING_PENALTY: 1.0,

  /** Points deducted per image without alt */
  IMAGE_NO_ALT_PENALTY: 3,

  /** Points deducted per button without label */
  BUTTON_NO_LABEL_PENALTY: 5,

  /** Points deducted for no headings */
  NO_HEADINGS_PENALTY: 20,

  /** Fallback scoring penalties */
  FALLBACK: {
    ERROR_PENALTY: 15,
    WARNING_PENALTY: 5,
    IMAGE_NO_ALT: 3,
    BUTTON_NO_LABEL: 5,
    NO_HEADINGS: 20
  }
} as const;

/**
 * Viewport configurations
 */
export const VIEWPORT = {
  /** Default desktop viewport */
  DESKTOP: {
    width: 1920,
    height: 1080
  },

  /** Mobile viewport */
  MOBILE: {
    width: 375,
    height: 667
  },

  /** Tablet viewport */
  TABLET: {
    width: 768,
    height: 1024
  }
} as const;

/**
 * HTTP Status code ranges
 */
export const HTTP_STATUS = {
  /** Successful responses */
  SUCCESS_MIN: 200,
  SUCCESS_MAX: 299,

  /** Redirect responses */
  REDIRECT_MIN: 300,
  REDIRECT_MAX: 399,

  /** Client error responses */
  CLIENT_ERROR_MIN: 400,
  CLIENT_ERROR_MAX: 499,

  /** Server error responses */
  SERVER_ERROR_MIN: 500,
  SERVER_ERROR_MAX: 599,

  /** Common status codes */
  OK: 200,
  NOT_FOUND: 404,
  MOVED_PERMANENTLY: 301,
  FOUND: 302
} as const;

/**
 * User agent strings
 */
export const USER_AGENTS = {
  /** Default user agent for testing */
  DEFAULT: 'auditmysite/2.0.0 (+https://github.com/casoon/AuditMySite)',

  /** Desktop Chrome user agent */
  CHROME_DESKTOP: 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36',

  /** Mobile user agent */
  MOBILE: 'Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1'
} as const;

/**
 * Default hide elements for pa11y
 */
export const PA11Y_HIDE_ELEMENTS = 'iframe[src*="google-analytics"], iframe[src*="doubleclick"]';

/**
 * Progress update intervals
 */
export const PROGRESS = {
  /** Queue progress update interval (ms) */
  UPDATE_INTERVAL: 3000,

  /** Progress reporting threshold (%) */
  REPORT_THRESHOLD: 25
} as const;

/**
 * Memory configurations
 */
export const MEMORY = {
  /** Node max old space size (MB) */
  MAX_OLD_SPACE_SIZE: 2048,

  /** Browser args for memory optimization */
  BROWSER_ARGS: [
    '--no-sandbox',
    '--disable-setuid-sandbox',
    '--disable-dev-shm-usage',
    '--disable-gpu',
    '--memory-pressure-off',
    `--max_old_space_size=2048`
  ]
} as const;

/**
 * Helper function to check if status code is in range
 */
export function isHttpStatusInRange(status: number, min: number, max: number): boolean {
  return status >= min && status <= max;
}

/**
 * Helper function to check if status is success
 */
export function isHttpSuccess(status: number): boolean {
  return isHttpStatusInRange(status, HTTP_STATUS.SUCCESS_MIN, HTTP_STATUS.SUCCESS_MAX);
}

/**
 * Helper function to check if status is redirect
 */
export function isHttpRedirect(status: number): boolean {
  return isHttpStatusInRange(status, HTTP_STATUS.REDIRECT_MIN, HTTP_STATUS.REDIRECT_MAX);
}

/**
 * Helper function to check if status is error
 */
export function isHttpError(status: number): boolean {
  return status >= HTTP_STATUS.CLIENT_ERROR_MIN;
}
