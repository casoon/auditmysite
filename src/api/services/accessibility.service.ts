import { AccessibilityChecker } from '../../core/accessibility';
import { AccessibilityResult } from '../../types/audit-results';

/**
 * AccessibilityService - Returns AccessibilityResult (same type used in PageAuditResult)
 * Used by API endpoint: POST /api/v2/page/accessibility
 */
export class AccessibilityService {
  private checker: AccessibilityChecker | null = null;

  async initialize(): Promise<void> {
    if (!this.checker) {
      this.checker = new AccessibilityChecker();
      await this.checker.initialize();
    }
  }

  async analyzeUrl(url: string, options: {
    pa11yStandard?: 'WCAG2A' | 'WCAG2AA' | 'WCAG2AAA' | 'Section508';
    includeWarnings?: boolean;
  } = {}): Promise<AccessibilityResult> {
    await this.initialize();

    try {
      // Run single URL accessibility test
      const result = await this.checker!.testPage(url, {
        timeout: 10000,
        waitUntil: 'domcontentloaded',
        pa11yStandard: options.pa11yStandard || 'WCAG2AA',
        includeWarnings: options.includeWarnings || false
      });

      // Convert to typed AccessibilityResult
      return {
        passed: result.passed,
        wcagLevel: this.getWcagLevel(options.pa11yStandard || 'WCAG2AA'),
        score: this.calculateScore(result),
        errors: (result.errors || []).map(error => ({
          severity: 'error' as const,
          message: error,
          code: 'a11y-error'
        })),
        warnings: (result.warnings || []).map(warning => ({
          severity: 'warning' as const,
          message: warning,
          code: 'a11y-warning'
        })),
        pa11yResults: {
          totalIssues: (result.errors?.length || 0) + (result.warnings?.length || 0),
          runner: 'pa11y@9.0.0'
        }
      };
    } catch (error) {
      throw new Error(`Failed to analyze accessibility: ${(error as Error).message}`);
    }
  }

  private getWcagLevel(standard: string): 'A' | 'AA' | 'AAA' | 'none' {
    switch (standard) {
      case 'WCAG2A': return 'A';
      case 'WCAG2AA': return 'AA';
      case 'WCAG2AAA': return 'AAA';
      default: return 'AA';
    }
  }

  private calculateScore(result: any): number {
    const errors = result.errors?.length || 0;
    const warnings = result.warnings?.length || 0;
    
    if (errors === 0 && warnings === 0) return 100;
    
    // Deduct points for issues (errors weighted more heavily)
    const score = Math.max(0, 100 - (errors * 10) - (warnings * 2));
    return Math.round(score);
  }

  async cleanup(): Promise<void> {
    if (this.checker) {
      await this.checker.cleanup();
      this.checker = null;
    }
  }
}
