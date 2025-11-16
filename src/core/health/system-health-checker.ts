/**
 * System Health Checker
 * Provides comprehensive health monitoring for AuditMySite
 */

import os from 'os';
import { Logger } from '../logging/logger';

export interface HealthStatus {
  status: 'healthy' | 'degraded' | 'unhealthy';
  timestamp: string;
  uptime: number;
  checks: {
    memory: HealthCheck;
    cpu: HealthCheck;
    browser: HealthCheck;
    filesystem: HealthCheck;
  };
  metrics: {
    totalMemoryMB: number;
    usedMemoryMB: number;
    freeMemoryMB: number;
    memoryUsagePercent: number;
    cpuLoadAverage: number[];
    processUptime: number;
  };
}

export interface HealthCheck {
  status: 'pass' | 'warn' | 'fail';
  message: string;
  details?: Record<string, unknown>;
}

export class SystemHealthChecker {
  private logger: Logger;
  private startTime: number;
  private readonly MEMORY_WARNING_THRESHOLD = 0.8; // 80%
  private readonly MEMORY_CRITICAL_THRESHOLD = 0.9; // 90%

  constructor() {
    this.logger = new Logger({ level: 'info' });
    this.startTime = Date.now();
  }

  /**
   * Get comprehensive system health status
   */
  async getHealthStatus(): Promise<HealthStatus> {
    const memoryCheck = this.checkMemory();
    const cpuCheck = this.checkCPU();
    const browserCheck = await this.checkBrowser();
    const filesystemCheck = this.checkFilesystem();

    // Determine overall status
    const checks = [memoryCheck, cpuCheck, browserCheck, filesystemCheck];
    const hasFailure = checks.some(check => check.status === 'fail');
    const hasWarning = checks.some(check => check.status === 'warn');

    let overallStatus: 'healthy' | 'degraded' | 'unhealthy';
    if (hasFailure) {
      overallStatus = 'unhealthy';
    } else if (hasWarning) {
      overallStatus = 'degraded';
    } else {
      overallStatus = 'healthy';
    }

    const totalMemory = os.totalmem();
    const freeMemory = os.freemem();
    const usedMemory = totalMemory - freeMemory;

    return {
      status: overallStatus,
      timestamp: new Date().toISOString(),
      uptime: Date.now() - this.startTime,
      checks: {
        memory: memoryCheck,
        cpu: cpuCheck,
        browser: browserCheck,
        filesystem: filesystemCheck,
      },
      metrics: {
        totalMemoryMB: Math.round(totalMemory / 1024 / 1024),
        usedMemoryMB: Math.round(usedMemory / 1024 / 1024),
        freeMemoryMB: Math.round(freeMemory / 1024 / 1024),
        memoryUsagePercent: (usedMemory / totalMemory) * 100,
        cpuLoadAverage: os.loadavg(),
        processUptime: process.uptime(),
      },
    };
  }

  /**
   * Check memory health
   */
  private checkMemory(): HealthCheck {
    const totalMemory = os.totalmem();
    const freeMemory = os.freemem();
    const usedMemory = totalMemory - freeMemory;
    const usagePercent = usedMemory / totalMemory;

    if (usagePercent >= this.MEMORY_CRITICAL_THRESHOLD) {
      return {
        status: 'fail',
        message: 'Critical memory usage',
        details: {
          usagePercent: Math.round(usagePercent * 100),
          threshold: this.MEMORY_CRITICAL_THRESHOLD * 100,
        },
      };
    }

    if (usagePercent >= this.MEMORY_WARNING_THRESHOLD) {
      return {
        status: 'warn',
        message: 'High memory usage',
        details: {
          usagePercent: Math.round(usagePercent * 100),
          threshold: this.MEMORY_WARNING_THRESHOLD * 100,
        },
      };
    }

    return {
      status: 'pass',
      message: 'Memory usage is healthy',
      details: {
        usagePercent: Math.round(usagePercent * 100),
      },
    };
  }

  /**
   * Check CPU health
   */
  private checkCPU(): HealthCheck {
    const loadAvg = os.loadavg();
    const cpuCount = os.cpus().length;
    const load1min = loadAvg[0];
    const loadPerCpu = load1min / cpuCount;

    // Load average threshold: 0.7 per CPU core
    if (loadPerCpu > 0.9) {
      return {
        status: 'fail',
        message: 'Critical CPU load',
        details: {
          loadAverage: loadAvg,
          cpuCount,
          loadPerCpu: Math.round(loadPerCpu * 100) / 100,
        },
      };
    }

    if (loadPerCpu > 0.7) {
      return {
        status: 'warn',
        message: 'High CPU load',
        details: {
          loadAverage: loadAvg,
          cpuCount,
          loadPerCpu: Math.round(loadPerCpu * 100) / 100,
        },
      };
    }

    return {
      status: 'pass',
      message: 'CPU load is healthy',
      details: {
        loadAverage: loadAvg,
        cpuCount,
      },
    };
  }

  /**
   * Check browser availability
   */
  private async checkBrowser(): Promise<HealthCheck> {
    try {
      // Check if Playwright browsers are installed
      const { execSync } = await import('child_process');

      try {
        // Check if chromium is available
        execSync('npx playwright --version', { stdio: 'ignore' });

        return {
          status: 'pass',
          message: 'Playwright browser is available',
        };
      } catch {
        return {
          status: 'fail',
          message: 'Playwright browser is not installed',
          details: {
            suggestion: 'Run: npx playwright install chromium',
          },
        };
      }
    } catch (error) {
      return {
        status: 'warn',
        message: 'Unable to verify browser installation',
        details: {
          error: error instanceof Error ? error.message : String(error),
        },
      };
    }
  }

  /**
   * Check filesystem health
   */
  private checkFilesystem(): HealthCheck {
    try {
      const tmpdir = os.tmpdir();

      // Try to write to temp directory
      const { writeFileSync, unlinkSync } = require('fs');
      const testFile = `${tmpdir}/auditmysite-health-check-${Date.now()}.tmp`;

      writeFileSync(testFile, 'health check', 'utf-8');
      unlinkSync(testFile);

      return {
        status: 'pass',
        message: 'Filesystem is writable',
        details: {
          tmpdir,
        },
      };
    } catch (error) {
      return {
        status: 'fail',
        message: 'Filesystem write failed',
        details: {
          error: error instanceof Error ? error.message : String(error),
        },
      };
    }
  }

  /**
   * Get simplified health check (for quick checks)
   */
  async isHealthy(): Promise<boolean> {
    const status = await this.getHealthStatus();
    return status.status === 'healthy';
  }

  /**
   * Log current health status
   */
  async logHealthStatus(): Promise<void> {
    const status = await this.getHealthStatus();

    if (status.status === 'healthy') {
      this.logger.info('System health check: HEALTHY');
    } else if (status.status === 'degraded') {
      this.logger.warn('System health check: DEGRADED', status.checks);
    } else {
      this.logger.error('System health check: UNHEALTHY', status.checks);
    }
  }
}
