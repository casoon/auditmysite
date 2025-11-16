/**
 * 🔧 Configuration Manager
 * 
 * Simplified configuration management for AuditMySite
 * Handles loading, merging, and validation of configurations
 */

import { 
  AuditConfig, 
  ServerConfig,
  StandardsConfig,
  PerformanceConfig,
  OutputConfig
} from './types';

export interface ValidationResult {
  isValid: boolean;
  errors: string[];
  warnings: string[];
  suggestions: string[];
}

export interface PresetConfig {
  name: string;
  description: string;
  config: Partial<AuditConfig>;
}

export class ConfigManager {
  private presets: Map<string, PresetConfig> = new Map();

  constructor() {
    this.initializePresets();
  }

  /**
   * Get default configuration
   */
  getDefaults(): AuditConfig {
    return {
      maxPages: 5,
      standards: {
        wcag: 'WCAG2AA',
        strictMode: false
      },
      performance: {
        budgets: 'default',
        collectMetrics: ['LCP', 'CLS', 'FCP'],
        ignoreThresholds: false
      },
      output: {
        format: 'html',
        outputDir: './reports',
        interactive: false
      }
    };
  }

  /**
   * Load configuration from CLI arguments
   */
  loadFromCLI(args: any): Partial<AuditConfig> {
    const config: Partial<AuditConfig> = {};

    if (args.maxPages !== undefined) {
      config.maxPages = args.maxPages;
    }

    if (args.format) {
      // Take the first format for the main format field, store others in extended config
      const format = Array.isArray(args.format) ? args.format[0] : args.format;
      config.output = { 
        format: format as 'html' | 'markdown' | 'json' | 'junit' | 'terminal',
        outputDir: args.outputDir || './reports'
      };
    }

    return config;
  }

  /**
   * Load configuration from environment variables
   */
  loadFromEnvironment(): Partial<AuditConfig> {
    const config: Partial<AuditConfig> = {};

    if (process.env.AUDIT_MAX_PAGES) {
      const maxPages = parseInt(process.env.AUDIT_MAX_PAGES);
      if (!isNaN(maxPages)) {
        config.maxPages = maxPages;
      }
    }

    return config;
  }

  /**
   * Merge multiple configurations
   */
  mergeConfigs(configs: Array<Partial<AuditConfig>>): AuditConfig {
    const defaults = this.getDefaults();
    const result = { ...defaults };

    for (const config of configs) {
      // Merge top-level properties
      if (config.maxPages !== undefined) {
        result.maxPages = config.maxPages;
      }
      if (config.standards) {
        result.standards = { ...result.standards, ...config.standards };
      }
      if (config.performance) {
        result.performance = { ...result.performance, ...config.performance };
      }
      if (config.output) {
        result.output = { ...result.output, ...config.output };
      }
    }

    return result;
  }

  /**
   * Validate configuration
   */
  validate(config: Partial<AuditConfig>): ValidationResult {
    const errors: string[] = [];
    const warnings: string[] = [];
    const suggestions: string[] = [];

    // Validate maxPages
    if (config.maxPages !== undefined && (config.maxPages < 1 || config.maxPages > 1000)) {
      errors.push('maxPages must be between 1 and 1000');
    }

    // Validate output config
    if (config.output) {
      const validFormats = ['html', 'markdown', 'json', 'junit', 'terminal'];
      if (config.output.format && !validFormats.includes(config.output.format)) {
        errors.push(`Invalid output format. Must be one of: ${validFormats.join(', ')}`);
      }
    }

    // Generate warnings
    if (config.maxPages && config.maxPages > 50) {
      warnings.push('Testing more than 50 pages may result in high resource usage and long processing times');
    }

    return {
      isValid: errors.length === 0,
      errors,
      warnings,
      suggestions
    };
  }

  /**
   * Load preset configuration
   */
  loadPreset(presetName: string): Partial<AuditConfig> {
    switch (presetName) {
      case 'react':
        return {
          maxPages: 10,
          standards: { wcag: 'WCAG2AA', strictMode: false },
          performance: { budgets: 'development', collectMetrics: ['LCP', 'CLS', 'FCP'] },
          framework: { type: 'react', router: 'react-router' }
        };

      case 'vue':
        return {
          maxPages: 8,
          standards: { wcag: 'WCAG2AA', strictMode: false },
          framework: { type: 'vue', router: 'vue-router' }
        };

      case 'angular':
        return {
          maxPages: 10,
          standards: { wcag: 'WCAG2AA', strictMode: false },
          framework: { type: 'angular' }
        };

      case 'ecommerce':
        return {
          maxPages: 25,
          standards: { wcag: 'WCAG2AA', strictMode: true },
          performance: { budgets: 'ecommerce', enforceThresholds: true }
        };

      case 'corporate':
        return {
          maxPages: 15,
          standards: { wcag: 'WCAG2AAA', strictMode: true },
          performance: { budgets: 'corporate' }
        };

      case 'blog':
        return {
          maxPages: 20,
          standards: { wcag: 'WCAG2AA', strictMode: false },
          performance: { budgets: 'blog' }
        };

      default:
        return {};
    }
  }

  /**
   * Initialize built-in presets
   */
  private initializePresets(): void {
    const presets: PresetConfig[] = [
      {
        name: 'react',
        description: 'Optimized for React applications',
        config: this.loadPreset('react')
      },
      {
        name: 'vue',
        description: 'Optimized for Vue.js applications', 
        config: this.loadPreset('vue')
      },
      {
        name: 'angular',
        description: 'Optimized for Angular applications',
        config: this.loadPreset('angular')
      },
      {
        name: 'ecommerce',
        description: 'Strict performance budgets for e-commerce',
        config: this.loadPreset('ecommerce')
      },
      {
        name: 'corporate',
        description: 'Professional standards for corporate sites',
        config: this.loadPreset('corporate')
      },
      {
        name: 'blog',
        description: 'Balanced settings for blogs and content sites',
        config: this.loadPreset('blog')
      }
    ];

    for (const preset of presets) {
      this.presets.set(preset.name, preset);
    }
  }

  /**
   * Get available presets
   */
  getAvailablePresets(): PresetConfig[] {
    return Array.from(this.presets.values());
  }
}
