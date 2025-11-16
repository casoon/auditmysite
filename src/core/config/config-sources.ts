/**
 * 🔧 Configuration Sources
 * 
 * Different sources for configuration data with priority handling.
 */

import { AuditConfig, ConfigSource } from './types';
import * as fs from 'fs';
import * as path from 'path';

export abstract class BaseConfigSource {
  abstract load(basePath?: string): Promise<ConfigSource | null>;
  abstract isAvailable(basePath?: string): Promise<boolean>;
}

/**
 * CLI Configuration Source - Highest priority
 */
export class CLIConfigSource extends BaseConfigSource {
  constructor(private cliArgs: Record<string, any>) {
    super();
  }

  async isAvailable(): Promise<boolean> {
    return Object.keys(this.cliArgs).length > 0;
  }

  async load(): Promise<ConfigSource> {
    const config: Partial<AuditConfig> = {};

    // Map CLI arguments to config structure
    if (this.cliArgs.maxPages) config.maxPages = this.cliArgs.maxPages;
    if (this.cliArgs.sitemapUrl) config.sitemapUrl = this.cliArgs.sitemapUrl;

    if (this.cliArgs.format || this.cliArgs.outputDir) {
      config.output = {
        format: this.cliArgs.format || 'html',
        outputDir: this.cliArgs.outputDir
      };
    }

    if (this.cliArgs.budget || this.cliArgs.lcpBudget || this.cliArgs.clsBudget) {
      config.performance = {
        budgets: this.cliArgs.budget || 'default',
        customBudgets: {}
      };

      if (this.cliArgs.lcpBudget) {
        config.performance.customBudgets!.lcp = {
          good: this.cliArgs.lcpBudget,
          poor: this.cliArgs.lcpBudget * 1.6
        };
      }
      if (this.cliArgs.clsBudget) {
        config.performance.customBudgets!.cls = {
          good: this.cliArgs.clsBudget,
          poor: this.cliArgs.clsBudget * 2.5
        };
      }
      if (this.cliArgs.fcpBudget) {
        config.performance.customBudgets!.fcp = {
          good: this.cliArgs.fcpBudget,
          poor: this.cliArgs.fcpBudget * 1.7
        };
      }
      if (this.cliArgs.inpBudget) {
        config.performance.customBudgets!.inp = {
          good: this.cliArgs.inpBudget,
          poor: this.cliArgs.inpBudget * 2.5
        };
      }
      if (this.cliArgs.ttfbBudget) {
        config.performance.customBudgets!.ttfb = {
          good: this.cliArgs.ttfbBudget,
          poor: this.cliArgs.ttfbBudget * 2
        };
      }
    }

    if (this.cliArgs.unifiedQueue || this.cliArgs.maxConcurrent) {
      config.testing = {
        queueType: this.cliArgs.unifiedQueue ? 'parallel' : 'simple',
        parallel: {
          enabled: true,
          maxConcurrent: this.cliArgs.maxConcurrent || 2
        }
      };
    }

    return {
      type: 'cli',
      priority: 100, // Highest priority
      data: config
    };
  }
}

/**
 * JavaScript Configuration File Source
 */
export class JSConfigSource extends BaseConfigSource {
  constructor(private filename: string = 'audit.config.js') {
    super();
  }

  async isAvailable(basePath: string = process.cwd()): Promise<boolean> {
    const configPath = path.join(basePath, this.filename);
    return fs.existsSync(configPath);
  }

  async load(basePath: string = process.cwd()): Promise<ConfigSource | null> {
    const configPath = path.join(basePath, this.filename);
    
    if (!await this.isAvailable(basePath)) {
      return null;
    }

    try {
      // Clear require cache to allow hot reloading
       
      delete require.cache[require.resolve(configPath)];
      // eslint-disable-next-line @typescript-eslint/no-require-imports -- Dynamic config loading
      const configModule = require(configPath);
      const config = configModule.default || configModule;

      return {
        type: 'file',
        path: configPath,
        priority: 90,
        data: config
      };
    } catch (error) {
      throw new Error(`Failed to load ${this.filename}: ${error}`);
    }
  }
}

/**
 * JSON Configuration File Source
 */
export class JSONConfigSource extends BaseConfigSource {
  constructor(private filename: string = 'audit.config.json') {
    super();
  }

  async isAvailable(basePath: string = process.cwd()): Promise<boolean> {
    const configPath = path.join(basePath, this.filename);
    return fs.existsSync(configPath);
  }

  async load(basePath: string = process.cwd()): Promise<ConfigSource | null> {
    const configPath = path.join(basePath, this.filename);
    
    if (!await this.isAvailable(basePath)) {
      return null;
    }

    try {
      const configContent = fs.readFileSync(configPath, 'utf8');
      const config = JSON.parse(configContent);

      return {
        type: 'file',
        path: configPath,
        priority: 85,
        data: config
      };
    } catch (error) {
      throw new Error(`Failed to load ${this.filename}: ${error}`);
    }
  }
}

/**
 * .auditrc Configuration File Source
 */
export class AuditRCSource extends BaseConfigSource {
  constructor(private filename: string = '.auditrc') {
    super();
  }

  async isAvailable(basePath: string = process.cwd()): Promise<boolean> {
    const configPath = path.join(basePath, this.filename);
    return fs.existsSync(configPath);
  }

  async load(basePath: string = process.cwd()): Promise<ConfigSource | null> {
    const configPath = path.join(basePath, this.filename);
    
    if (!await this.isAvailable(basePath)) {
      return null;
    }

    try {
      const configContent = fs.readFileSync(configPath, 'utf8');
      
      // Try JSON first, then YAML
      let config: any;
      try {
        config = JSON.parse(configContent);
      } catch {
        // Try YAML parsing if available
        try {
          // eslint-disable-next-line @typescript-eslint/no-require-imports -- Optional YAML support
          const yaml = require('js-yaml');
          config = yaml.load(configContent);
        } catch {
          throw new Error('Config file must be valid JSON or YAML');
        }
      }

      return {
        type: 'file',
        path: configPath,
        priority: 80,
        data: config
      };
    } catch (error) {
      throw new Error(`Failed to load ${this.filename}: ${error}`);
    }
  }
}

/**
 * Package.json Configuration Source
 */
export class PackageJsonConfigSource extends BaseConfigSource {
  async isAvailable(basePath: string = process.cwd()): Promise<boolean> {
    const packagePath = path.join(basePath, 'package.json');
    if (!fs.existsSync(packagePath)) return false;

    try {
      const packageContent = JSON.parse(fs.readFileSync(packagePath, 'utf8'));
      return packageContent.auditConfig !== undefined;
    } catch {
      return false;
    }
  }

  async load(basePath: string = process.cwd()): Promise<ConfigSource | null> {
    const packagePath = path.join(basePath, 'package.json');
    
    if (!await this.isAvailable(basePath)) {
      return null;
    }

    try {
      const packageContent = JSON.parse(fs.readFileSync(packagePath, 'utf8'));
      const config = packageContent.auditConfig;

      return {
        type: 'package.json',
        path: packagePath,
        priority: 70,
        data: config
      };
    } catch (error) {
      throw new Error(`Failed to load package.json config: ${error}`);
    }
  }
}

/**
 * Environment Variables Configuration Source
 */
export class EnvironmentConfigSource extends BaseConfigSource {
  async isAvailable(): Promise<boolean> {
    // Check for common environment variables
    return !!(
      process.env.AUDIT_SITEMAP_URL ||
      process.env.AUDIT_MAX_PAGES ||
      process.env.AUDIT_OUTPUT_FORMAT ||
      process.env.AUDIT_BUDGET
    );
  }

  async load(): Promise<ConfigSource | null> {
    if (!await this.isAvailable()) {
      return null;
    }

    const config: Partial<AuditConfig> = {};

    if (process.env.AUDIT_SITEMAP_URL) {
      config.sitemapUrl = process.env.AUDIT_SITEMAP_URL;
    }

    if (process.env.AUDIT_MAX_PAGES) {
      config.maxPages = parseInt(process.env.AUDIT_MAX_PAGES, 10);
    }

    if (process.env.AUDIT_OUTPUT_FORMAT || process.env.AUDIT_OUTPUT_DIR) {
      config.output = {
        format: (process.env.AUDIT_OUTPUT_FORMAT as any) || 'html',
        outputDir: process.env.AUDIT_OUTPUT_DIR
      };
    }

    if (process.env.AUDIT_BUDGET) {
      config.performance = {
        budgets: process.env.AUDIT_BUDGET as any
      };
    }

    return {
      type: 'environment',
      priority: 60,
      data: config
    };
  }
}

/**
 * Default Configuration Source - Lowest priority
 */
export class DefaultConfigSource extends BaseConfigSource {
  async isAvailable(): Promise<boolean> {
    return true; // Always available
  }

  async load(): Promise<ConfigSource> {
    const config: AuditConfig = {
      maxPages: 5,
      standards: {
        wcag: 'WCAG2AA',
        strictMode: false,
        failOnWarnings: false
      },
      performance: {
        budgets: 'default',
        collectMetrics: ['LCP', 'FCP', 'CLS', 'INP', 'TTFB'],
        ignoreThresholds: false
      },
      output: {
        format: 'html',
        outputDir: './reports',
        interactive: true,
        detailedFixes: true
      },
      testing: {
        parallel: {
          enabled: true,
          maxConcurrent: 2,
          retries: 3
        },
        screenshots: {
          enabled: false,
          onErrors: true,
          responsive: ['desktop']
        },
        coverage: {
          accessibility: true,
          performance: true,
          seo: false,
          security: false
        },
        queueType: 'parallel'
      },
      reporting: {
        outputDir: './reports',
        formats: ['html']
      },
      framework: {
        type: 'static'
      },
      advanced: {
        browser: {
          viewport: { width: 1920, height: 1080 },
          userAgent: 'auditmysite/1.6.0 (+https://github.com/casoon/AuditMySite)'
        },
        waitConditions: ['domcontentloaded']
      }
    };

    return {
      type: 'default',
      priority: 10, // Lowest priority
      data: config
    };
  }
}
