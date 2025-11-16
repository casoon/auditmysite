/**
 * Custom Error Classes for AuditMySite
 * Provides type-safe error handling across the application
 */

export class AuditError extends Error {
  constructor(
    message: string,
    public readonly code: string,
    public readonly context?: Record<string, unknown>
  ) {
    super(message);
    this.name = 'AuditError';
    Error.captureStackTrace(this, this.constructor);
  }
}

export class NetworkError extends AuditError {
  constructor(message: string, context?: Record<string, unknown>) {
    super(message, 'NETWORK_ERROR', context);
    this.name = 'NetworkError';
  }
}

export class TimeoutError extends AuditError {
  constructor(message: string, context?: Record<string, unknown>) {
    super(message, 'TIMEOUT_ERROR', context);
    this.name = 'TimeoutError';
  }
}

export class ValidationError extends AuditError {
  constructor(message: string, context?: Record<string, unknown>) {
    super(message, 'VALIDATION_ERROR', context);
    this.name = 'ValidationError';
  }
}

export class BrowserError extends AuditError {
  constructor(message: string, context?: Record<string, unknown>) {
    super(message, 'BROWSER_ERROR', context);
    this.name = 'BrowserError';
  }
}

export class AnalysisError extends AuditError {
  constructor(message: string, context?: Record<string, unknown>) {
    super(message, 'ANALYSIS_ERROR', context);
    this.name = 'AnalysisError';
  }
}

/**
 * Type guard to check if an error is an AuditError
 */
export function isAuditError(error: unknown): error is AuditError {
  return error instanceof AuditError;
}

/**
 * Type guard to check if value is an Error
 */
export function isError(error: unknown): error is Error {
  return error instanceof Error;
}

/**
 * Safe error message extraction
 */
export function getErrorMessage(error: unknown): string {
  if (isError(error)) {
    return error.message;
  }
  if (typeof error === 'string') {
    return error;
  }
  if (error && typeof error === 'object' && 'message' in error) {
    return String(error.message);
  }
  return 'Unknown error';
}

/**
 * Safe error stack extraction
 */
export function getErrorStack(error: unknown): string | undefined {
  if (isError(error)) {
    return error.stack;
  }
  return undefined;
}

/**
 * Convert unknown error to AuditError
 */
export function toAuditError(error: unknown, code = 'UNKNOWN_ERROR', context?: Record<string, unknown>): AuditError {
  if (isAuditError(error)) {
    return error;
  }

  const message = getErrorMessage(error);
  const auditError = new AuditError(message, code, context);

  // Preserve original stack if available
  const stack = getErrorStack(error);
  if (stack) {
    auditError.stack = stack;
  }

  return auditError;
}
