/**
 * 🔐 Security Headers Analyzer
 * 
 * Comprehensive security headers analysis including:
 * - Content Security Policy (CSP) validation
 * - HTTP Strict Transport Security (HSTS)
 * - X-Frame-Options, X-Content-Type-Options
 * - Referrer Policy, Permissions Policy
 * - HTTPS configuration and certificate analysis
 * - Cookie security analysis
 */

import { Page, Response } from 'playwright';
import { 
  SecurityHeadersMetrics,
  SecurityHeader,
  CSPAnalysis
} from '../types/enhanced-metrics';
import {
  BaseAnalyzer,
  BaseAnalysisResult,
  BaseAnalysisOptions,
  BaseRecommendation,
  Grade,
  CertificateLevel,
  calculateGrade,
  calculateCertificateLevel,
  isSecureUrl
} from '../types/base-types';

// Security Headers specific result interface
interface SecurityHeadersAnalysisResult extends BaseAnalysisResult {
  securityHeaders: SecurityHeadersMetrics;
  recommendations: BaseRecommendation[];
}

// Security Headers specific options interface
interface SecurityHeadersAnalysisOptions extends BaseAnalysisOptions {
  /** Include certificate analysis */
  includeCertificateAnalysis?: boolean;
  /** Include cookie security analysis */
  includeCookieAnalysis?: boolean;
  /** Timeout for certificate analysis */
  certificateTimeout?: number;
}

export class SecurityHeadersAnalyzer implements BaseAnalyzer<SecurityHeadersAnalysisResult, SecurityHeadersAnalysisOptions> {
  constructor() {}

  // BaseAnalyzer interface implementations
  getName(): string {
    return 'SecurityHeadersAnalyzer';
  }

  getVersion(): string {
    return '1.0.0';
  }

  getScore(result: SecurityHeadersAnalysisResult): number {
    return result.overallScore;
  }

  getGrade(score: number): Grade {
    return calculateGrade(score);
  }

  getCertificateLevel(score: number): CertificateLevel {
    return calculateCertificateLevel(score);
  }

  getRecommendations(result: SecurityHeadersAnalysisResult): BaseRecommendation[] {
    return result.recommendations;
  }

  /**
   * Main analyze method implementing BaseAnalyzer interface
   */
  async analyze(page: Page, url: string | { loc: string }, options: SecurityHeadersAnalysisOptions = {}): Promise<SecurityHeadersAnalysisResult> {
    // Extract URL string from URL object if needed
    const urlString = (typeof url === 'object' && url.loc ? url.loc : url) as string;

    const startTime = Date.now();
    
    try {
      // Capture initial page response for headers analysis
      let response: Response | null = null;
      
      // Set up response listener before navigation
      page.once('response', (resp) => {
        if (resp.url() === urlString) {
          response = resp;
        }
      });

      // If page is already loaded, try to get response from navigation
      try {
        // Check if page is already at the target URL
        const currentUrl = page.url();
        if (currentUrl !== urlString) {
          response = await page.goto(urlString, { 
            waitUntil: 'networkidle',
            timeout: options.timeout || 30000 
          });
        }
      } catch (error) {
        console.warn('Failed to navigate for security headers analysis:', error);
        // Continue with analysis using current page state
      }

      // Analyze security headers
      const securityHeaders = await this.analyzeSecurityHeaders(page, urlString, response, options);
      
      const duration = Date.now() - startTime;
      
      // Calculate overall score
      const overallScore = this.calculateOverallScore(securityHeaders);
      const grade = calculateGrade(overallScore);
      const certificate = calculateCertificateLevel(overallScore);
      
      // Generate recommendations
      const recommendations = this.generateRecommendations(securityHeaders);

      return {
        overallScore,
        grade,
        certificate,
        analyzedAt: new Date().toISOString(),
        duration,
        status: 'completed' as const,
        securityHeaders,
        recommendations
      };

    } catch (error) {
      console.error('❌ Security headers analysis failed:', error);
      throw new Error(`Security headers analysis failed: ${error}`);
    }
  }

  /**
   * Analyze security headers comprehensively
   */
  private async analyzeSecurityHeaders(
    page: Page, 
    url: string, 
    response: Response | null, 
    options: SecurityHeadersAnalysisOptions
  ): Promise<SecurityHeadersMetrics> {
    
    const headers = response ? await response.headers() : {};
    
    // Analyze individual security headers
    const csp = this.analyzeCSP(headers);
    const hsts = this.analyzeHSTS(headers);
    const xFrameOptions = this.analyzeXFrameOptions(headers);
    const xContentTypeOptions = this.analyzeXContentTypeOptions(headers);
    const xXSSProtection = this.analyzeXXSSProtection(headers);
    const referrerPolicy = this.analyzeReferrerPolicy(headers);
    const permissionsPolicy = this.analyzePermissionsPolicy(headers);

    // Analyze HTTPS configuration
    const https = await this.analyzeHTTPS(url, page, options);

    // Analyze cookies security
    const cookies = options.includeCookieAnalysis 
      ? await this.analyzeCookieSecurity(page)
      : this.getDefaultCookieAnalysis();

    // Assess vulnerabilities
    const vulnerabilities = this.assessVulnerabilities({
      csp,
      hsts,
      xFrameOptions,
      xContentTypeOptions,
      https
    });

    // Calculate overall score
    const overallScore = this.calculateSecurityScore({
      csp,
      hsts,
      xFrameOptions,
      xContentTypeOptions,
      xXSSProtection,
      referrerPolicy,
      permissionsPolicy,
      https,
      cookies
    });

    const securityGrade = calculateGrade(overallScore);

    // Generate security-specific recommendations
    const recommendations = this.generateSecurityRecommendations({
      csp,
      hsts,
      xFrameOptions,
      xContentTypeOptions,
      xXSSProtection,
      referrerPolicy,
      permissionsPolicy,
      https,
      cookies,
      vulnerabilities
    });

    return {
      overallScore,
      securityGrade,
      headers: {
        csp,
        hsts,
        xFrameOptions,
        xContentTypeOptions,
        xXSSProtection,
        referrerPolicy,
        permissionsPolicy
      },
      https,
      cookies,
      recommendations,
      vulnerabilities
    };
  }

  /**
   * Analyze Content Security Policy
   */
  private analyzeCSP(headers: Record<string, string>): CSPAnalysis {
    const cspHeader = headers['content-security-policy'] || headers['content-security-policy-report-only'];
    
    if (!cspHeader) {
      return {
        present: false,
        directives: {},
        score: 0,
        issues: [{
          severity: 'critical',
          issue: 'Content Security Policy is missing',
          recommendation: 'Implement a CSP header to prevent XSS attacks and data injection'
        }],
        hasUnsafeDirectives: false,
        allowsInlineScripts: true, // No CSP means everything is allowed
        allowsEval: true
      };
    }

    // Parse CSP directives
    const directives: Record<string, string[]> = {};
    const directiveStrings = cspHeader.split(';').map(d => d.trim()).filter(d => d);
    
    for (const directive of directiveStrings) {
      const [name, ...values] = directive.split(/\s+/);
      if (name) {
        directives[name] = values;
      }
    }

    // Analyze security implications
    const issues: CSPAnalysis['issues'] = [];
    let score = 100;
    
    // Check for unsafe directives
    const hasUnsafeInline = this.hasUnsafeInlineDirectives(directives);
    const hasUnsafeEval = this.hasUnsafeEvalDirectives(directives);
    const allowsInlineScripts = hasUnsafeInline || !directives['script-src'];
    const allowsEval = hasUnsafeEval || !directives['script-src'];

    if (hasUnsafeInline) {
      score -= 30;
      issues.push({
        severity: 'high',
        directive: 'script-src or style-src',
        issue: "Unsafe 'unsafe-inline' directive found",
        recommendation: 'Remove unsafe-inline and use nonces or hashes for inline scripts/styles'
      });
    }

    if (hasUnsafeEval) {
      score -= 25;
      issues.push({
        severity: 'high',
        directive: 'script-src',
        issue: "Unsafe 'unsafe-eval' directive found",
        recommendation: 'Remove unsafe-eval to prevent code injection attacks'
      });
    }

    // Check for missing important directives
    if (!directives['default-src']) {
      score -= 15;
      issues.push({
        severity: 'medium',
        directive: 'default-src',
        issue: 'Missing default-src directive',
        recommendation: 'Add default-src directive as fallback for other resource types'
      });
    }

    if (!directives['script-src']) {
      score -= 20;
      issues.push({
        severity: 'medium',
        directive: 'script-src',
        issue: 'Missing script-src directive',
        recommendation: 'Add script-src directive to control JavaScript execution'
      });
    }

    if (!directives['object-src']) {
      score -= 10;
      issues.push({
        severity: 'low',
        directive: 'object-src',
        issue: 'Missing object-src directive',
        recommendation: "Add 'object-src none' to prevent plugin content"
      });
    }

    return {
      present: true,
      value: cspHeader,
      directives,
      score: Math.max(0, score),
      issues,
      hasUnsafeDirectives: hasUnsafeInline || hasUnsafeEval,
      allowsInlineScripts,
      allowsEval
    };
  }

  /**
   * Check for unsafe inline directives in CSP
   */
  private hasUnsafeInlineDirectives(directives: Record<string, string[]>): boolean {
    const relevantDirectives = ['script-src', 'style-src', 'default-src'];
    return relevantDirectives.some(directive => 
      directives[directive]?.includes("'unsafe-inline'")
    );
  }

  /**
   * Check for unsafe eval directives in CSP
   */
  private hasUnsafeEvalDirectives(directives: Record<string, string[]>): boolean {
    const relevantDirectives = ['script-src', 'default-src'];
    return relevantDirectives.some(directive => 
      directives[directive]?.includes("'unsafe-eval'")
    );
  }

  /**
   * Analyze HTTP Strict Transport Security
   */
  private analyzeHSTS(headers: Record<string, string>): SecurityHeader {
    const hstsHeader = headers['strict-transport-security'];
    
    if (!hstsHeader) {
      return {
        name: 'Strict-Transport-Security',
        present: false,
        valid: false,
        score: 0,
        issues: ['HSTS header is missing - HTTPS connections not enforced'],
        recommendations: ['Add Strict-Transport-Security header to enforce HTTPS']
      };
    }

    let score = 100;
    const issues: string[] = [];
    const recommendations: string[] = [];

    // Parse max-age
    const maxAgeMatch = hstsHeader.match(/max-age=(\d+)/);
    const maxAge = maxAgeMatch ? parseInt(maxAgeMatch[1]) : 0;

    if (maxAge < 31536000) { // Less than 1 year
      score -= 20;
      issues.push(`Max-age is too short (${maxAge} seconds)`);
      recommendations.push('Set max-age to at least 31536000 (1 year)');
    }

    // Check for includeSubDomains
    if (!hstsHeader.includes('includeSubDomains')) {
      score -= 10;
      issues.push('includeSubDomains directive is missing');
      recommendations.push('Add includeSubDomains to protect subdomains');
    }

    // Check for preload
    if (!hstsHeader.includes('preload')) {
      score -= 5;
      recommendations.push('Consider adding preload directive for HSTS preload list');
    }

    return {
      name: 'Strict-Transport-Security',
      value: hstsHeader,
      present: true,
      valid: maxAge > 0,
      score: Math.max(0, score),
      issues,
      recommendations
    };
  }

  /**
   * Analyze X-Frame-Options header
   */
  private analyzeXFrameOptions(headers: Record<string, string>): SecurityHeader {
    const xFrameHeader = headers['x-frame-options'];
    
    if (!xFrameHeader) {
      return {
        name: 'X-Frame-Options',
        present: false,
        valid: false,
        score: 0,
        issues: ['X-Frame-Options header is missing - vulnerable to clickjacking'],
        recommendations: ['Add X-Frame-Options: DENY or SAMEORIGIN to prevent clickjacking']
      };
    }

    const value = xFrameHeader.toLowerCase();
    const validValues = ['deny', 'sameorigin'];
    const isValid = validValues.includes(value) || value.startsWith('allow-from');

    let score = isValid ? 100 : 0;
    const issues: string[] = [];
    const recommendations: string[] = [];

    if (!isValid) {
      issues.push(`Invalid X-Frame-Options value: ${xFrameHeader}`);
      recommendations.push('Use DENY, SAMEORIGIN, or ALLOW-FROM with a specific origin');
    } else if (value === 'sameorigin') {
      score = 90; // DENY is more secure
      recommendations.push('Consider using DENY for maximum security if framing is not needed');
    }

    return {
      name: 'X-Frame-Options',
      value: xFrameHeader,
      present: true,
      valid: isValid,
      score,
      issues,
      recommendations
    };
  }

  /**
   * Analyze X-Content-Type-Options header
   */
  private analyzeXContentTypeOptions(headers: Record<string, string>): SecurityHeader {
    const xContentTypeHeader = headers['x-content-type-options'];
    
    if (!xContentTypeHeader) {
      return {
        name: 'X-Content-Type-Options',
        present: false,
        valid: false,
        score: 0,
        issues: ['X-Content-Type-Options header is missing - vulnerable to MIME-type sniffing'],
        recommendations: ['Add X-Content-Type-Options: nosniff to prevent MIME-type sniffing']
      };
    }

    const isValid = xContentTypeHeader.toLowerCase() === 'nosniff';
    
    return {
      name: 'X-Content-Type-Options',
      value: xContentTypeHeader,
      present: true,
      valid: isValid,
      score: isValid ? 100 : 0,
      issues: isValid ? [] : [`Invalid value: ${xContentTypeHeader}, should be 'nosniff'`],
      recommendations: isValid ? [] : ['Set X-Content-Type-Options to "nosniff"']
    };
  }

  /**
   * Analyze X-XSS-Protection header (deprecated but still relevant)
   */
  private analyzeXXSSProtection(headers: Record<string, string>): SecurityHeader {
    const xssHeader = headers['x-xss-protection'];
    
    if (!xssHeader) {
      return {
        name: 'X-XSS-Protection',
        present: false,
        valid: false,
        score: 50, // Not critical since it's deprecated
        issues: ['X-XSS-Protection header is missing'],
        recommendations: ['Add X-XSS-Protection: 1; mode=block (though CSP is preferred)']
      };
    }

    const value = xssHeader.toLowerCase();
    const isValid = value.includes('1') && value.includes('mode=block');
    
    return {
      name: 'X-XSS-Protection',
      value: xssHeader,
      present: true,
      valid: isValid,
      score: isValid ? 100 : 70,
      issues: isValid ? [] : ['X-XSS-Protection should be "1; mode=block"'],
      recommendations: isValid 
        ? ['Consider implementing CSP instead of relying on X-XSS-Protection']
        : ['Set X-XSS-Protection to "1; mode=block"']
    };
  }

  /**
   * Analyze Referrer-Policy header
   */
  private analyzeReferrerPolicy(headers: Record<string, string>): SecurityHeader {
    const referrerHeader = headers['referrer-policy'];
    
    if (!referrerHeader) {
      return {
        name: 'Referrer-Policy',
        present: false,
        valid: false,
        score: 70, // Not critical but recommended
        issues: ['Referrer-Policy header is missing'],
        recommendations: ['Add Referrer-Policy header to control referrer information']
      };
    }

    const validPolicies = [
      'no-referrer', 'no-referrer-when-downgrade', 'origin', 
      'origin-when-cross-origin', 'same-origin', 'strict-origin',
      'strict-origin-when-cross-origin', 'unsafe-url'
    ];

    const policy = referrerHeader.toLowerCase();
    const isValid = validPolicies.includes(policy);
    
    let score = isValid ? 100 : 0;
    const recommendations: string[] = [];

    // Rate security level of different policies
    if (policy === 'no-referrer' || policy === 'strict-origin-when-cross-origin') {
      score = 100;
    } else if (policy === 'unsafe-url') {
      score = 30;
      recommendations.push('Consider using a more restrictive policy like "strict-origin-when-cross-origin"');
    }

    return {
      name: 'Referrer-Policy',
      value: referrerHeader,
      present: true,
      valid: isValid,
      score,
      issues: isValid ? [] : [`Invalid referrer policy: ${referrerHeader}`],
      recommendations
    };
  }

  /**
   * Analyze Permissions-Policy / Feature-Policy header
   */
  private analyzePermissionsPolicy(headers: Record<string, string>): SecurityHeader {
    const permissionsHeader = headers['permissions-policy'] || headers['feature-policy'];
    
    if (!permissionsHeader) {
      return {
        name: 'Permissions-Policy',
        present: false,
        valid: false,
        score: 80, // Good to have but not critical
        issues: ['Permissions-Policy header is missing'],
        recommendations: ['Consider adding Permissions-Policy to control browser features']
      };
    }

    // Basic validation - permissions policy syntax is complex
    const hasValidSyntax = permissionsHeader.includes('=') || permissionsHeader.includes('()');
    
    return {
      name: 'Permissions-Policy',
      value: permissionsHeader,
      present: true,
      valid: hasValidSyntax,
      score: hasValidSyntax ? 100 : 50,
      issues: hasValidSyntax ? [] : ['Permissions-Policy syntax appears invalid'],
      recommendations: hasValidSyntax 
        ? ['Review permissions policy to ensure it matches your security requirements']
        : ['Fix Permissions-Policy syntax and test with browser developer tools']
    };
  }

  /**
   * Analyze HTTPS configuration
   */
  private async analyzeHTTPS(url: string, page: Page, options: SecurityHeadersAnalysisOptions): Promise<SecurityHeadersMetrics['https']> {
    const isHTTPS = isSecureUrl(url);
    
    let httpsRedirect = false;
    let mixedContent = false;
    
    if (isHTTPS) {
      // Check for mixed content by analyzing page resources
      try {
        mixedContent = await page.evaluate(() => {
          const resources = Array.from(document.querySelectorAll('script[src], link[href], img[src], iframe[src]'));
          return resources.some(element => {
            const src = element.getAttribute('src') || element.getAttribute('href');
            return src && src.startsWith('http://');
          });
        });
      } catch (error) {
        console.warn('Failed to check for mixed content:', error);
      }
    } else {
      // Test if HTTP redirects to HTTPS
      try {
        const httpUrl = url.replace('https://', 'http://');
        const response = await page.goto(httpUrl, { waitUntil: 'domcontentloaded', timeout: 5000 });
        httpsRedirect = response?.url().startsWith('https://') || false;
      } catch (error) {
        // Redirect test failed, assume no redirect
        httpsRedirect = false;
      }
    }

    // Certificate analysis (basic - would need more sophisticated analysis for production)
    const certificate = {
      valid: isHTTPS, // Simplified - actual cert validation would require additional checks
      issuer: undefined as string | undefined,
      expiresAt: undefined as string | undefined,
      daysUntilExpiry: undefined as number | undefined
    };

    return {
      enabled: isHTTPS,
      httpsRedirect,
      mixedContent,
      certificate
    };
  }

  /**
   * Analyze cookie security
   */
  private async analyzeCookieSecurity(page: Page): Promise<SecurityHeadersMetrics['cookies']> {
    try {
      const cookies = await page.context().cookies();
      
      const totalCookies = cookies.length;
      const secureCookies = cookies.filter(c => c.secure).length;
      const httpOnlyCookies = cookies.filter(c => c.httpOnly).length;
      const sameSiteCookies = cookies.filter(c => c.sameSite && c.sameSite !== 'None').length;
      
      const issues: string[] = [];
      
      if (totalCookies > 0) {
        if (secureCookies < totalCookies) {
          issues.push(`${totalCookies - secureCookies} cookies without Secure flag`);
        }
        if (httpOnlyCookies < totalCookies) {
          issues.push(`${totalCookies - httpOnlyCookies} cookies without HttpOnly flag`);
        }
        if (sameSiteCookies < totalCookies) {
          issues.push(`${totalCookies - sameSiteCookies} cookies without proper SameSite attribute`);
        }
      }

      return {
        totalCookies,
        secureCookies,
        httpOnlyCookies,
        sameSiteCookies,
        issues
      };
    } catch (error) {
      console.warn('Failed to analyze cookies:', error);
      return this.getDefaultCookieAnalysis();
    }
  }

  /**
   * Get default cookie analysis when analysis fails
   */
  private getDefaultCookieAnalysis(): SecurityHeadersMetrics['cookies'] {
    return {
      totalCookies: 0,
      secureCookies: 0,
      httpOnlyCookies: 0,
      sameSiteCookies: 0,
      issues: ['Could not analyze cookies']
    };
  }

  /**
   * Assess overall vulnerability status
   */
  private assessVulnerabilities(analysis: {
    csp: CSPAnalysis;
    hsts: SecurityHeader;
    xFrameOptions: SecurityHeader;
    xContentTypeOptions: SecurityHeader;
    https: SecurityHeadersMetrics['https'];
  }): SecurityHeadersMetrics['vulnerabilities'] {
    
    return {
      clickjacking: analysis.xFrameOptions.valid ? 'protected' : 'vulnerable',
      xss: analysis.csp.present && !analysis.csp.hasUnsafeDirectives ? 'protected' : 
           analysis.csp.present ? 'partially_protected' : 'vulnerable',
      contentTypeSniffing: analysis.xContentTypeOptions.valid ? 'protected' : 'vulnerable',
      referrerLeakage: 'partially_protected', // Depends on referrer policy
      mixedContent: analysis.https.enabled && !analysis.https.mixedContent ? 'protected' : 'vulnerable'
    };
  }

  /**
   * Calculate overall security score
   */
  private calculateSecurityScore(analysis: {
    csp: CSPAnalysis;
    hsts: SecurityHeader;
    xFrameOptions: SecurityHeader;
    xContentTypeOptions: SecurityHeader;
    xXSSProtection: SecurityHeader;
    referrerPolicy: SecurityHeader;
    permissionsPolicy: SecurityHeader;
    https: SecurityHeadersMetrics['https'];
    cookies: SecurityHeadersMetrics['cookies'];
  }): number {
    
    const weights = {
      csp: 0.25,           // 25% - Most important
      hsts: 0.20,          // 20% - Critical for HTTPS
      xFrameOptions: 0.15,  // 15% - Clickjacking protection
      xContentTypeOptions: 0.15, // 15% - MIME sniffing protection
      https: 0.15,         // 15% - HTTPS usage
      xXSSProtection: 0.05, // 5% - Deprecated but still counted
      referrerPolicy: 0.03, // 3% - Privacy protection
      permissionsPolicy: 0.02 // 2% - Feature control
    };

    let weightedScore = 0;
    let totalWeight = 0;

    // CSP score
    weightedScore += analysis.csp.score * weights.csp;
    totalWeight += weights.csp;

    // HSTS score
    weightedScore += analysis.hsts.score * weights.hsts;
    totalWeight += weights.hsts;

    // X-Frame-Options score
    weightedScore += analysis.xFrameOptions.score * weights.xFrameOptions;
    totalWeight += weights.xFrameOptions;

    // X-Content-Type-Options score
    weightedScore += analysis.xContentTypeOptions.score * weights.xContentTypeOptions;
    totalWeight += weights.xContentTypeOptions;

    // HTTPS score
    const httpsScore = analysis.https.enabled ? 100 : 0;
    weightedScore += httpsScore * weights.https;
    totalWeight += weights.https;

    // Other headers
    weightedScore += analysis.xXSSProtection.score * weights.xXSSProtection;
    totalWeight += weights.xXSSProtection;

    weightedScore += analysis.referrerPolicy.score * weights.referrerPolicy;
    totalWeight += weights.referrerPolicy;

    weightedScore += analysis.permissionsPolicy.score * weights.permissionsPolicy;
    totalWeight += weights.permissionsPolicy;

    return Math.round(weightedScore / totalWeight);
  }

  /**
   * Calculate overall score for BaseAnalyzer interface
   */
  private calculateOverallScore(securityHeaders: SecurityHeadersMetrics): number {
    return securityHeaders.overallScore;
  }

  /**
   * Generate security-specific recommendations
   */
  private generateSecurityRecommendations(analysis: any): SecurityHeadersMetrics['recommendations'] {
    const recommendations: SecurityHeadersMetrics['recommendations'] = [];

    // CSP recommendations
    if (!analysis.csp.present) {
      recommendations.push({
        priority: 'critical',
        category: 'Content Security Policy',
        issue: 'No Content Security Policy implemented',
        recommendation: 'Implement a CSP header to prevent XSS attacks and data injection',
        impact: 'Protects against cross-site scripting and data injection attacks'
      });
    } else if (analysis.csp.hasUnsafeDirectives) {
      recommendations.push({
        priority: 'high',
        category: 'Content Security Policy',
        issue: 'CSP contains unsafe directives',
        recommendation: 'Remove unsafe-inline and unsafe-eval directives, use nonces or hashes instead',
        impact: 'Eliminates major XSS attack vectors'
      });
    }

    // HSTS recommendations
    if (!analysis.hsts.present && analysis.https.enabled) {
      recommendations.push({
        priority: 'high',
        category: 'HTTPS Security',
        issue: 'HSTS header missing',
        recommendation: 'Add Strict-Transport-Security header with includeSubDomains',
        impact: 'Prevents HTTPS downgrade attacks and protects subdomains'
      });
    }

    // Clickjacking recommendations
    if (!analysis.xFrameOptions.present) {
      recommendations.push({
        priority: 'medium',
        category: 'Clickjacking Protection',
        issue: 'X-Frame-Options header missing',
        recommendation: 'Add X-Frame-Options: DENY or SAMEORIGIN header',
        impact: 'Prevents clickjacking attacks'
      });
    }

    // HTTPS recommendations
    if (!analysis.https.enabled) {
      recommendations.push({
        priority: 'critical',
        category: 'HTTPS',
        issue: 'Site not using HTTPS',
        recommendation: 'Implement HTTPS with valid SSL certificate',
        impact: 'Encrypts data transmission and improves SEO ranking'
      });
    }

    return recommendations;
  }

  /**
   * Generate general recommendations for BaseAnalyzer interface
   */
  private generateRecommendations(securityHeaders: SecurityHeadersMetrics): BaseRecommendation[] {
    return securityHeaders.recommendations.map(rec => ({
      id: `security-${rec.category.toLowerCase().replace(/\s+/g, '-')}`,
      priority: rec.priority as any,
      category: rec.category,
      issue: rec.issue,
      recommendation: rec.recommendation,
      impact: rec.impact,
      effort: this.estimateEffort(rec.priority),
      scoreImprovement: this.estimateScoreImprovement(rec.priority)
    }));
  }

  /**
   * Estimate implementation effort based on priority
   */
  private estimateEffort(priority: string): number {
    switch (priority) {
      case 'critical': return 8;
      case 'high': return 4;
      case 'medium': return 2;
      case 'low': return 1;
      default: return 2;
    }
  }

  /**
   * Estimate score improvement based on priority
   */
  private estimateScoreImprovement(priority: string): number {
    switch (priority) {
      case 'critical': return 25;
      case 'high': return 15;
      case 'medium': return 10;
      case 'low': return 5;
      default: return 8;
    }
  }
}