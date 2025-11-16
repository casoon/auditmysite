/**
 * Central Logger for AuditMySite
 * 
 * Provides structured logging with different levels:
 * - ERROR: Critical errors that need immediate attention
 * - WARN: Warnings and fallbacks that indicate potential issues
 * - INFO: General information about the audit process
 * - SUCCESS: Successful completion of operations
 * - DEBUG: Detailed debugging information (only in verbose mode)
 * 
 * Features:
 * - Clean, consistent output formatting
 * - Configurable verbosity levels
 * - Progress tracking without progress bars
 * - Structured fallback reporting
 */

export enum LogLevel {
  ERROR = 0,
  WARN = 1,
  INFO = 2,
  SUCCESS = 3,
  DEBUG = 4
}

export interface LoggerOptions {
  level: LogLevel | 'error' | 'warn' | 'info' | 'success' | 'debug';
  verbose: boolean;
  prefix?: string;
  enableColors?: boolean;
}

export interface ProgressTracker {
  total: number;
  completed: number;
  failed: number;
  current?: string;
  startTime: Date;
}

export class Logger {
  private options: Required<Omit<LoggerOptions, 'prefix'>> & { prefix?: string };
  private progress: ProgressTracker | null = null;

  constructor(options: Partial<LoggerOptions> = {}) {
    // Convert string levels to LogLevel enum
    const normalizedLevel = this.normalizeLevel(options.level);

    this.options = {
      verbose: false,
      enableColors: true,
      ...options,
      level: normalizedLevel
    };
  }

  /**
   * Normalize log level from string to enum
   */
  private normalizeLevel(level?: LogLevel | 'error' | 'warn' | 'info' | 'success' | 'debug'): LogLevel {
    if (typeof level === 'number') {
      return level as LogLevel;
    }

    switch (level) {
      case 'error':
        return LogLevel.ERROR;
      case 'warn':
        return LogLevel.WARN;
      case 'info':
        return LogLevel.INFO;
      case 'success':
        return LogLevel.SUCCESS;
      case 'debug':
        return LogLevel.DEBUG;
      default:
        return LogLevel.INFO;
    }
  }

  /**
   * Set the verbosity level
   */
  setVerbose(verbose: boolean): void {
    this.options.verbose = verbose;
    this.options.level = verbose ? LogLevel.DEBUG : LogLevel.INFO;
  }
  
  /**
   * Log an error (always shown)
   */
  error(message: string, details?: any): void {
    this.log(LogLevel.ERROR, '❌', message, details);
  }
  
  /**
   * Log a warning (always shown)
   */
  warn(message: string, details?: any): void {
    this.log(LogLevel.WARN, '⚠️', message, details);
  }
  
  /**
   * Log a fallback warning (always shown, special formatting)
   */
  fallback(component: string, reason: string, alternative: string, details?: any): void {
    const message = `FALLBACK: ${component} - ${reason}, ${alternative}`;
    this.log(LogLevel.WARN, '⚠️', message, details);
  }
  
  /**
   * Log general information
   */
  info(message: string, details?: any): void {
    if (this.shouldLog(LogLevel.INFO)) {
      this.log(LogLevel.INFO, 'ℹ️', message, details);
    }
  }
  
  /**
   * Log success messages
   */
  success(message: string, details?: any): void {
    if (this.shouldLog(LogLevel.SUCCESS)) {
      this.log(LogLevel.SUCCESS, '✅', message, details);
    }
  }
  
  /**
   * Log debug information (only in verbose mode)
   */
  debug(message: string, details?: any): void {
    if (this.shouldLog(LogLevel.DEBUG)) {
      this.log(LogLevel.DEBUG, '🔧', message, details);
    }
  }
  
  /**
   * Start progress tracking
   */
  startProgress(total: number, description?: string): void {
    this.progress = {
      total,
      completed: 0,
      failed: 0,
      startTime: new Date()
    };
    
    if (description) {
      this.info(`Starting ${description} (${total} items)`);
    }
  }
  
  /**
   * Update progress (replaces progress bars)
   */
  updateProgress(completed: number, failed: number = 0, current?: string): void {
    if (!this.progress) return;
    
    this.progress.completed = completed;
    this.progress.failed = failed;
    this.progress.current = current;
    
    // Only show progress updates in verbose mode or at significant intervals
    const total = this.progress.total;
    const percentage = Math.round((completed / total) * 100);
    
    if (this.options.verbose || percentage % 25 === 0) {
      const status = failed > 0 ? ` (${failed} failed)` : '';
      this.info(`Progress: ${completed}/${total} (${percentage}%)${status}`);
    }
  }
  
  /**
   * Complete progress tracking
   */
  completeProgress(): void {
    if (!this.progress) return;
    
    const { completed, failed, total, startTime } = this.progress;
    const duration = Date.now() - startTime.getTime();
    const seconds = Math.round(duration / 1000);
    
    if (failed > 0) {
      this.warn(`Completed ${completed}/${total} items in ${seconds}s (${failed} failed)`);
    } else {
      this.success(`Completed all ${completed} items in ${seconds}s`);
    }
    
    this.progress = null;
  }
  
  /**
   * Log a section header
   */
  section(title: string): void {
    if (this.shouldLog(LogLevel.INFO)) {
      console.log(`\n🔍 ${title}`);
    }
  }
  
  /**
   * Log analysis results summary
   */
  results(stats: {
    tested: number;
    passed: number;
    failed: number;
    errors: number;
    warnings: number;
    successRate: number;
  }): void {
    console.log('\n📊 Results:');
    console.log(`   📄 Tested: ${stats.tested} pages`);
    console.log(`   ✅ Passed: ${stats.passed}`);
    console.log(`   ❌ Failed: ${stats.failed}`);
    console.log(`   ⚠️  Errors: ${stats.errors}`);
    console.log(`   ⚠️  Warnings: ${stats.warnings}`);
    console.log(`   🎯 Success Rate: ${stats.successRate.toFixed(1)}%`);
  }
  
  /**
   * Log configuration information
   */
  config(config: Record<string, any>): void {
    if (this.shouldLog(LogLevel.INFO)) {
      console.log('\n📋 Configuration:');
      Object.entries(config).forEach(([key, value]) => {
        console.log(`   📄 ${key}: ${value}`);
      });
    }
  }
  
  /**
   * Log generated files
   */
  files(files: string[]): void {
    console.log('\n📁 Generated reports:');
    files.forEach(file => {
      console.log(`   📄 ${file}`);
    });
  }
  
  /**
   * Core logging method
   */
  private log(level: LogLevel, icon: string, message: string, details?: any): void {
    const prefix = this.options.prefix ? `[${this.options.prefix}] ` : '';
    
    // Format message based on level
    const formattedMessage = `${icon} ${prefix}${message}`;
    
    console.log(formattedMessage);
    
    // Log details in verbose mode
    if (details && this.options.verbose) {
      console.log('   Details:', details);
    }
  }
  
  /**
   * Check if we should log at this level
   */
  private shouldLog(level: LogLevel): boolean {
    const currentLevel = typeof this.options.level === 'number'
      ? this.options.level
      : this.normalizeLevel(this.options.level);
    return level <= currentLevel;
  }
  
  /**
   * Create a child logger with a prefix
   */
  child(prefix: string): Logger {
    return new Logger({
      ...this.options,
      prefix: this.options.prefix ? `${this.options.prefix}:${prefix}` : prefix
    });
  }
}

// Global logger instance
export const logger = new Logger();

// Convenience functions for common use cases
export const log = {
  error: (message: string, details?: any) => logger.error(message, details),
  warn: (message: string, details?: any) => logger.warn(message, details),
  fallback: (component: string, reason: string, alternative: string, details?: any) => 
    logger.fallback(component, reason, alternative, details),
  info: (message: string, details?: any) => logger.info(message, details),
  success: (message: string, details?: any) => logger.success(message, details),
  debug: (message: string, details?: any) => logger.debug(message, details),
  section: (title: string) => logger.section(title),
  results: (stats: any) => logger.results(stats),
  config: (config: Record<string, any>) => logger.config(config),
  files: (files: string[]) => logger.files(files),
  setVerbose: (verbose: boolean) => logger.setVerbose(verbose),
  startProgress: (total: number, description?: string) => logger.startProgress(total, description),
  updateProgress: (completed: number, failed?: number, current?: string) => 
    logger.updateProgress(completed, failed, current),
  completeProgress: () => logger.completeProgress()
};
