/**
 * Audit Debugger
 * Debugging and monitoring utilities for audit execution
 */

import fs from 'fs';
import path from 'path';
import { AccessibilityResult, TestSummary } from '../types';
import { Logger } from '../core/logging/logger';

export interface DebugSnapshot {
  timestamp: string;
  totalPages: number;
  completedPages: number;
  failedPages: number;
  currentPage?: string;
  memoryUsage: NodeJS.MemoryUsage;
  elapsedTime: number;
  averagePageTime: number;
  estimatedTimeRemaining: number;
}

export interface AuditDebugOptions {
  enableSnapshots?: boolean;
  snapshotInterval?: number; // milliseconds
  saveDebugData?: boolean;
  debugOutputDir?: string;
  logMemoryWarnings?: boolean;
  memoryWarningThreshold?: number; // MB
}

export class AuditDebugger {
  private logger: Logger;
  private options: Required<AuditDebugOptions>;
  private snapshots: DebugSnapshot[] = [];
  private startTime: number = 0;
  private snapshotTimer?: NodeJS.Timeout;

  constructor(options: AuditDebugOptions = {}) {
    this.logger = new Logger({ level: 'debug' });
    this.options = {
      enableSnapshots: options.enableSnapshots ?? true,
      snapshotInterval: options.snapshotInterval ?? 5000,
      saveDebugData: options.saveDebugData ?? true,
      debugOutputDir: options.debugOutputDir ?? './debug-output',
      logMemoryWarnings: options.logMemoryWarnings ?? true,
      memoryWarningThreshold: options.memoryWarningThreshold ?? 512
    };

    if (this.options.saveDebugData) {
      this.ensureDebugDir();
    }
  }

  /**
   * Start debugging session
   */
  startSession(): void {
    this.startTime = Date.now();
    this.snapshots = [];

    this.logger.info('🔍 Debug session started');

    if (this.options.enableSnapshots) {
      this.startSnapshotTimer();
    }
  }

  /**
   * End debugging session
   */
  endSession(): void {
    if (this.snapshotTimer) {
      clearInterval(this.snapshotTimer);
    }

    const duration = Date.now() - this.startTime;
    this.logger.info(`🔍 Debug session ended (Duration: ${Math.round(duration / 1000)}s)`);

    if (this.options.saveDebugData && this.snapshots.length > 0) {
      this.saveSnapshotsToFile();
    }
  }

  /**
   * Take a debug snapshot
   */
  takeSnapshot(
    totalPages: number,
    completedPages: number,
    failedPages: number,
    currentPage?: string
  ): DebugSnapshot {
    const now = Date.now();
    const elapsedTime = now - this.startTime;
    const averagePageTime = completedPages > 0 ? elapsedTime / completedPages : 0;
    const remainingPages = totalPages - completedPages - failedPages;
    const estimatedTimeRemaining = remainingPages * averagePageTime;

    const snapshot: DebugSnapshot = {
      timestamp: new Date().toISOString(),
      totalPages,
      completedPages,
      failedPages,
      currentPage,
      memoryUsage: process.memoryUsage(),
      elapsedTime,
      averagePageTime,
      estimatedTimeRemaining
    };

    this.snapshots.push(snapshot);

    // Check memory usage
    if (this.options.logMemoryWarnings) {
      this.checkMemoryUsage(snapshot.memoryUsage);
    }

    return snapshot;
  }

  /**
   * Log current progress
   */
  logProgress(snapshot: DebugSnapshot): void {
    const progress = (snapshot.completedPages / snapshot.totalPages) * 100;

    this.logger.debug('');
    this.logger.debug('─────────────────────────────────────');
    this.logger.debug(`📊 Progress: ${snapshot.completedPages}/${snapshot.totalPages} (${Math.round(progress)}%)`);
    this.logger.debug(`⏱️  Elapsed: ${Math.round(snapshot.elapsedTime / 1000)}s`);
    this.logger.debug(`⏳ Estimated remaining: ${Math.round(snapshot.estimatedTimeRemaining / 1000)}s`);
    this.logger.debug(`⚡ Average page time: ${Math.round(snapshot.averagePageTime / 1000)}s`);

    if (snapshot.currentPage) {
      this.logger.debug(`🔄 Current: ${snapshot.currentPage}`);
    }

    if (snapshot.failedPages > 0) {
      this.logger.debug(`❌ Failed: ${snapshot.failedPages}`);
    }

    const memoryMB = Math.round(snapshot.memoryUsage.heapUsed / 1024 / 1024);
    this.logger.debug(`💾 Memory: ${memoryMB} MB`);
    this.logger.debug('─────────────────────────────────────');
    this.logger.debug('');
  }

  /**
   * Check memory usage and warn if high
   */
  private checkMemoryUsage(memoryUsage: NodeJS.MemoryUsage): void {
    const heapUsedMB = memoryUsage.heapUsed / 1024 / 1024;

    if (heapUsedMB > this.options.memoryWarningThreshold) {
      this.logger.warn(`⚠️  High memory usage: ${Math.round(heapUsedMB)} MB`);

      // Suggest garbage collection
      if (global.gc) {
        this.logger.debug('Running garbage collection...');
        global.gc();
      } else {
        this.logger.debug('Tip: Run with --expose-gc flag to enable manual garbage collection');
      }
    }
  }

  /**
   * Start automatic snapshot timer
   */
  private startSnapshotTimer(): void {
    this.snapshotTimer = setInterval(() => {
      if (this.snapshots.length > 0) {
        const lastSnapshot = this.snapshots[this.snapshots.length - 1];
        this.logProgress(lastSnapshot);
      }
    }, this.options.snapshotInterval);
  }

  /**
   * Save debug snapshots to file
   */
  private saveSnapshotsToFile(): void {
    try {
      const filename = `debug-snapshots-${Date.now()}.json`;
      const filepath = path.join(this.options.debugOutputDir, filename);

      fs.writeFileSync(filepath, JSON.stringify(this.snapshots, null, 2));

      this.logger.info(`💾 Debug snapshots saved to: ${filepath}`);
    } catch (error) {
      this.logger.error('Failed to save debug snapshots', error);
    }
  }

  /**
   * Save detailed audit results for debugging
   */
  saveAuditDebugData(summary: TestSummary, filename: string = 'audit-debug.json'): void {
    if (!this.options.saveDebugData) return;

    try {
      const filepath = path.join(this.options.debugOutputDir, filename);

      const debugData = {
        timestamp: new Date().toISOString(),
        summary: {
          totalPages: summary.totalPages,
          testedPages: summary.testedPages,
          passedPages: summary.passedPages,
          failedPages: summary.failedPages,
          crashedPages: summary.crashedPages,
          totalErrors: summary.totalErrors,
          totalWarnings: summary.totalWarnings,
          totalDuration: summary.totalDuration
        },
        results: summary.results.map(r => ({
          url: r.url,
          title: r.title,
          passed: r.passed,
          crashed: r.crashed,
          skipped: r.skipped,
          duration: r.duration,
          errorCount: r.errors.length,
          warningCount: r.warnings.length,
          pa11yScore: r.pa11yScore,
          performanceScore: r.performanceMetrics?.performanceScore,
          performanceGrade: r.performanceMetrics?.performanceGrade,
          hasPerformanceMetrics: !!r.performanceMetrics,
          hasPa11yIssues: !!r.pa11yIssues && r.pa11yIssues.length > 0,
          hasScreenshots: !!r.screenshots
        })),
        snapshots: this.snapshots
      };

      fs.writeFileSync(filepath, JSON.stringify(debugData, null, 2));

      this.logger.info(`💾 Audit debug data saved to: ${filepath}`);
    } catch (error) {
      this.logger.error('Failed to save audit debug data', error);
    }
  }

  /**
   * Generate performance report
   */
  generatePerformanceReport(): string {
    if (this.snapshots.length === 0) {
      return 'No snapshots available';
    }

    const lines: string[] = [];

    lines.push('');
    lines.push('═══════════════════════════════════════════════════════');
    lines.push('  AUDIT PERFORMANCE REPORT');
    lines.push('═══════════════════════════════════════════════════════');
    lines.push('');

    const lastSnapshot = this.snapshots[this.snapshots.length - 1];

    // Overall stats
    lines.push('Overall Statistics:');
    lines.push(`  Total Pages: ${lastSnapshot.totalPages}`);
    lines.push(`  Completed: ${lastSnapshot.completedPages}`);
    lines.push(`  Failed: ${lastSnapshot.failedPages}`);
    lines.push(`  Total Time: ${Math.round(lastSnapshot.elapsedTime / 1000)}s`);
    lines.push(`  Average Page Time: ${Math.round(lastSnapshot.averagePageTime / 1000)}s`);
    lines.push('');

    // Memory stats
    const avgMemory = this.snapshots.reduce((sum, s) =>
      sum + s.memoryUsage.heapUsed, 0
    ) / this.snapshots.length;
    const maxMemory = Math.max(...this.snapshots.map(s => s.memoryUsage.heapUsed));

    lines.push('Memory Usage:');
    lines.push(`  Average: ${Math.round(avgMemory / 1024 / 1024)} MB`);
    lines.push(`  Peak: ${Math.round(maxMemory / 1024 / 1024)} MB`);
    lines.push('');

    // Performance trends
    if (this.snapshots.length > 2) {
      const firstHalf = this.snapshots.slice(0, Math.floor(this.snapshots.length / 2));
      const secondHalf = this.snapshots.slice(Math.floor(this.snapshots.length / 2));

      const firstHalfAvg = firstHalf.reduce((sum, s) =>
        sum + s.averagePageTime, 0
      ) / firstHalf.length;
      const secondHalfAvg = secondHalf.reduce((sum, s) =>
        sum + s.averagePageTime, 0
      ) / secondHalf.length;

      const trend = ((secondHalfAvg - firstHalfAvg) / firstHalfAvg) * 100;

      lines.push('Performance Trend:');
      if (trend > 10) {
        lines.push(`  ⚠️  Slowing down (${Math.round(trend)}% slower)`);
      } else if (trend < -10) {
        lines.push(`  ✓ Speeding up (${Math.abs(Math.round(trend))}% faster)`);
      } else {
        lines.push(`  ✓ Consistent performance`);
      }
      lines.push('');
    }

    lines.push('═══════════════════════════════════════════════════════');
    lines.push('');

    return lines.join('\n');
  }

  /**
   * Ensure debug directory exists
   */
  private ensureDebugDir(): void {
    if (!fs.existsSync(this.options.debugOutputDir)) {
      fs.mkdirSync(this.options.debugOutputDir, { recursive: true });
    }
  }
}
