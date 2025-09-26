/**
 * Structured logging implementation for AuditMySite
 * 
 * Provides a clean, consistent logging interface that can be easily
 * configured and replaced without affecting business logic.
 */

import chalk from 'chalk';
import { ILogger } from '../analyzers/interfaces';

export interface LoggerConfig {
  readonly level: LogLevel;
  readonly prefix?: string;
  readonly enableTimestamps?: boolean;
  readonly enableColors?: boolean;
}

export type LogLevel = 'debug' | 'info' | 'warn' | 'error' | 'success';

const LOG_LEVELS: Record<LogLevel, number> = {
  debug: 0,
  info: 1,
  warn: 2,
  error: 3,
  success: 1
};

export class StructuredLogger implements ILogger {
  private readonly config: Required<LoggerConfig>;

  constructor(config: Partial<LoggerConfig> = {}) {
    this.config = {
      level: config.level || 'info',
      prefix: config.prefix || '',
      enableTimestamps: config.enableTimestamps ?? true,
      enableColors: config.enableColors ?? true
    };
  }

  debug(message: string, data?: any): void {
    if (this.shouldLog('debug')) {
      this.log('debug', message, data);
    }
  }

  info(message: string, data?: any): void {
    if (this.shouldLog('info')) {
      this.log('info', message, data);
    }
  }

  warn(message: string, data?: any): void {
    if (this.shouldLog('warn')) {
      this.log('warn', message, data);
    }
  }

  error(message: string, error?: Error | any): void {
    if (this.shouldLog('error')) {
      this.log('error', message, error);
    }
  }

  success(message: string, data?: any): void {
    if (this.shouldLog('success')) {
      this.log('success', message, data);
    }
  }

  /**
   * Create a child logger with additional context
   */
  child(prefix: string): StructuredLogger {
    return new StructuredLogger({
      ...this.config,
      prefix: this.config.prefix ? `${this.config.prefix}:${prefix}` : prefix
    });
  }

  private shouldLog(level: LogLevel): boolean {
    return LOG_LEVELS[level] >= LOG_LEVELS[this.config.level];
  }

  private log(level: LogLevel, message: string, data?: any): void {
    const timestamp = this.config.enableTimestamps 
      ? `[${new Date().toISOString()}] ` 
      : '';
    
    const prefix = this.config.prefix 
      ? `[${this.config.prefix}] ` 
      : '';
    
    const levelLabel = `[${level.toUpperCase()}]`;
    
    let formattedMessage = `${timestamp}${prefix}${levelLabel} ${message}`;

    if (this.config.enableColors) {
      formattedMessage = this.colorize(level, formattedMessage);
    }

    // Output to appropriate stream
    if (level === 'error') {
      console.error(formattedMessage);
      if (data && data instanceof Error) {
        console.error(data.stack);
      } else if (data) {
        console.error('Data:', data);
      }
    } else {
      console.log(formattedMessage);
      if (data && typeof data === 'object') {
        console.log('Data:', JSON.stringify(data, null, 2));
      } else if (data) {
        console.log('Data:', data);
      }
    }
  }

  private colorize(level: LogLevel, message: string): string {
    switch (level) {
      case 'debug':
        return chalk.gray(message);
      case 'info':
        return chalk.blue(message);
      case 'warn':
        return chalk.yellow(message);
      case 'error':
        return chalk.red(message);
      case 'success':
        return chalk.green(message);
      default:
        return message;
    }
  }
}

/**
 * Silent logger for testing or when logging is not needed
 */
export class SilentLogger implements ILogger {
  debug(): void {}
  info(): void {}
  warn(): void {}
  error(): void {}
  success(): void {}
}

/**
 * Default logger instance for convenience
 */
export const defaultLogger = new StructuredLogger({
  level: 'info',
  enableColors: true,
  enableTimestamps: false
});

/**
 * Create a logger for a specific component
 */
export function createLogger(component: string, level: LogLevel = 'info'): ILogger {
  return new StructuredLogger({
    level,
    prefix: component,
    enableColors: true,
    enableTimestamps: false
  });
}