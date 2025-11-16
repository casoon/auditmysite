/**
 * 🚨 Standardized Error Handling
 * 
 * Common error types and handling utilities for the audit system.
 * Provides consistent error categorization and recovery strategies.
 */

/**
 * Base class for all audit-related errors
 */
export class AuditError extends Error {
  public readonly code: string;
  public readonly category: ErrorCategory;
  public readonly recoverable: boolean;
  public readonly context?: Record<string, unknown>;

  constructor(
    message: string,
    code: string,
    category: ErrorCategory,
    recoverable = false,
    context?: Record<string, unknown>
  ) {
    super(message);
    this.name = this.constructor.name;
    this.code = code;
    this.category = category;
    this.recoverable = recoverable;
    this.context = context;

    // Maintains proper stack trace for where error was thrown
    Error.captureStackTrace(this, this.constructor);
  }
}

/**
 * Error categories for better error handling
 */
export enum ErrorCategory {
  NETWORK = 'NETWORK',
  BROWSER = 'BROWSER',
  PARSING = 'PARSING',
  VALIDATION = 'VALIDATION',
  RESOURCE = 'RESOURCE',
  TIMEOUT = 'TIMEOUT',
  PERMISSION = 'PERMISSION',
  CONFIGURATION = 'CONFIGURATION',
  UNKNOWN = 'UNKNOWN'
}

/**
 * Network-related errors (connection, DNS, HTTP)
 */
export class NetworkError extends AuditError {
  constructor(message: string, url?: string, statusCode?: number) {
    super(
      message,
      'ERR_NETWORK',
      ErrorCategory.NETWORK,
      true, // Retryable
      { url, statusCode }
    );
  }
}

/**
 * Browser/Playwright-related errors
 */
export class BrowserError extends AuditError {
  constructor(message: string, browserType?: string, recoverable = false) {
    super(
      message,
      'ERR_BROWSER',
      ErrorCategory.BROWSER,
      recoverable,
      { browserType }
    );
  }
}

/**
 * Sitemap/XML parsing errors
 */
export class ParsingError extends AuditError {
  constructor(message: string, source?: string) {
    super(
      message,
      'ERR_PARSING',
      ErrorCategory.PARSING,
      false,
      { source }
    );
  }
}

/**
 * Data validation errors
 */
export class ValidationError extends AuditError {
  constructor(message: string, field?: string, value?: unknown) {
    super(
      message,
      'ERR_VALIDATION',
      ErrorCategory.VALIDATION,
      false,
      { field, value }
    );
  }
}

/**
 * System resource errors (memory, CPU, disk)
 */
export class ResourceError extends AuditError {
  constructor(message: string, resourceType?: string, limit?: number) {
    super(
      message,
      'ERR_RESOURCE',
      ErrorCategory.RESOURCE,
      false,
      { resourceType, limit }
    );
  }
}

/**
 * Timeout errors
 */
export class TimeoutError extends AuditError {
  constructor(message: string, operation?: string, timeoutMs?: number) {
    super(
      message,
      'ERR_TIMEOUT',
      ErrorCategory.TIMEOUT,
      true, // Retryable
      { operation, timeoutMs }
    );
  }
}

/**
 * Permission/access errors
 */
export class PermissionError extends AuditError {
  constructor(message: string, resource?: string) {
    super(
      message,
      'ERR_PERMISSION',
      ErrorCategory.PERMISSION,
      false,
      { resource }
    );
  }
}

/**
 * Configuration errors
 */
export class ConfigurationError extends AuditError {
  constructor(message: string, configKey?: string, providedValue?: unknown) {
    super(
      message,
      'ERR_CONFIGURATION',
      ErrorCategory.CONFIGURATION,
      false,
      { configKey, providedValue }
    );
  }
}

/**
 * Error handler utility
 */
export class ErrorHandler {
  /**
   * Categorize unknown errors into known types
   */
  static categorize(error: Error | unknown): AuditError {
    if (error instanceof AuditError) {
      return error;
    }

    const errorMessage = error instanceof Error ? error.message : String(error);
    const lowerMessage = errorMessage.toLowerCase();

    // Network errors
    if (
      lowerMessage.includes('network') ||
      lowerMessage.includes('econnrefused') ||
      lowerMessage.includes('enotfound') ||
      lowerMessage.includes('timeout') ||
      lowerMessage.includes('fetch failed')
    ) {
      return new NetworkError(errorMessage);
    }

    // Browser errors
    if (
      lowerMessage.includes('browser') ||
      lowerMessage.includes('playwright') ||
      lowerMessage.includes('page closed') ||
      lowerMessage.includes('context') ||
      lowerMessage.includes('target closed')
    ) {
      return new BrowserError(errorMessage);
    }

    // Parsing errors
    if (
      lowerMessage.includes('parse') ||
      lowerMessage.includes('xml') ||
      lowerMessage.includes('json') ||
      lowerMessage.includes('sitemap')
    ) {
      return new ParsingError(errorMessage);
    }

    // Validation errors
    if (
      lowerMessage.includes('invalid') ||
      lowerMessage.includes('validation') ||
      lowerMessage.includes('required')
    ) {
      return new ValidationError(errorMessage);
    }

    // Resource errors
    if (
      lowerMessage.includes('memory') ||
      lowerMessage.includes('heap') ||
      lowerMessage.includes('out of') ||
      lowerMessage.includes('resource')
    ) {
      return new ResourceError(errorMessage);
    }

    // Timeout errors
    if (lowerMessage.includes('timeout')) {
      return new TimeoutError(errorMessage);
    }

    // Permission errors
    if (
      lowerMessage.includes('permission') ||
      lowerMessage.includes('eacces') ||
      lowerMessage.includes('forbidden')
    ) {
      return new PermissionError(errorMessage);
    }

    // Unknown error
    return new AuditError(
      errorMessage,
      'ERR_UNKNOWN',
      ErrorCategory.UNKNOWN,
      false
    );
  }

  /**
   * Get user-friendly error message with recovery suggestions
   */
  static getUserMessage(error: AuditError): string {
    const messages: Record<ErrorCategory, string> = {
      [ErrorCategory.NETWORK]: `Network error: ${error.message}. Please check your internet connection and try again.`,
      [ErrorCategory.BROWSER]: `Browser error: ${error.message}. The browser may have crashed or closed unexpectedly.`,
      [ErrorCategory.PARSING]: `Parsing error: ${error.message}. The sitemap or data format may be invalid.`,
      [ErrorCategory.VALIDATION]: `Validation error: ${error.message}. Please check your input parameters.`,
      [ErrorCategory.RESOURCE]: `Resource error: ${error.message}. The system may be running out of memory or resources.`,
      [ErrorCategory.TIMEOUT]: `Timeout error: ${error.message}. The operation took too long. Try increasing timeout or reducing load.`,
      [ErrorCategory.PERMISSION]: `Permission error: ${error.message}. Check file/folder permissions.`,
      [ErrorCategory.CONFIGURATION]: `Configuration error: ${error.message}. Please review your configuration settings.`,
      [ErrorCategory.UNKNOWN]: `Unexpected error: ${error.message}`
    };

    return messages[error.category] || error.message;
  }

  /**
   * Determine if error should be retried
   */
  static shouldRetry(error: AuditError, attemptNumber: number, maxRetries: number): boolean {
    if (attemptNumber >= maxRetries) {
      return false;
    }

    return error.recoverable;
  }

  /**
   * Calculate backoff delay for retries
   */
  static getRetryDelay(attemptNumber: number, baseDelay = 1000): number {
    // Exponential backoff: 1s, 2s, 4s, 8s, ...
    return Math.min(baseDelay * Math.pow(2, attemptNumber - 1), 30000); // Max 30s
  }
}

/**
 * Async wrapper with automatic error categorization and retry logic
 */
export async function withErrorHandling<T>(
  operation: () => Promise<T>,
  options: {
    maxRetries?: number;
    baseDelay?: number;
    onRetry?: (error: AuditError, attempt: number) => void;
  } = {}
): Promise<T> {
  const { maxRetries = 3, baseDelay = 1000, onRetry } = options;
  let lastError: AuditError;

  for (let attempt = 1; attempt <= maxRetries; attempt++) {
    try {
      return await operation();
    } catch (error) {
      lastError = ErrorHandler.categorize(error);

      if (!ErrorHandler.shouldRetry(lastError, attempt, maxRetries)) {
        throw lastError;
      }

      if (onRetry) {
        onRetry(lastError, attempt);
      }

      const delay = ErrorHandler.getRetryDelay(attempt, baseDelay);
      await new Promise(resolve => setTimeout(resolve, delay));
    }
  }

  throw lastError!;
}
