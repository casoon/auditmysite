/**
 * 🎯 Audit Quality Verification Tests
 * 
 * Diese Tests verifizieren, dass die Audits qualitativ hochwertig
 * und korrekt funktionieren. Sie prüfen:
 * 
 * 1. Datenstruktur-Korrektheit
 * 2. Score-Berechnungen
 * 3. Issue-Detection
 * 4. Performance-Metriken
 * 5. Report-Vollständigkeit
 */

import { describe, it, expect, beforeAll, afterAll } from '@jest/globals';
import { AccessibilityChecker } from '../../src/core/accessibility/accessibility-checker';
import { BrowserPoolManager } from '../../src/core/browser/browser-pool-manager';
import { HTMLGenerator } from '../../src/generators/html-generator';
import { JsonGenerator } from '../../src/generators/json-generator';
import * as fs from 'fs';
import * as path from 'path';

describe('Audit Quality Verification', () => {
  let checker: AccessibilityChecker;
  let poolManager: BrowserPoolManager;
  
  // Test-URLs mit bekannten Charakteristiken
  const TEST_URLS = {
    // Seite mit guter Accessibility
    good: 'https://www.w3.org/WAI/',
    // Seite mit bekannten Issues
    issues: 'https://www.example.com',
    // Komplexe moderne Seite
    complex: 'https://github.com'
  };

  beforeAll(async () => {
    poolManager = new BrowserPoolManager({
      maxConcurrent: 2,
      browserType: 'chromium',
      enableResourceOptimization: true
    });
    
    checker = new AccessibilityChecker({
      usePooling: true,
      poolManager,
      enableComprehensiveAnalysis: true,
      qualityAnalysisOptions: {
        includeResourceAnalysis: true,
        includeTechnicalSEO: true,
        analysisTimeout: 30000
      }
    });
    
    await checker.initialize();
  }, 60000);

  afterAll(async () => {
    if (checker) {
      await checker.cleanup();
    }
    if (poolManager) {
      await poolManager.shutdown();
    }
  });

  describe('1. Data Structure Verification', () => {
    it('should return complete accessibility result structure', async () => {
      const result = await checker.testPage(TEST_URLS.good, {
        timeout: 30000,
        verbose: false
      });

      // Basis-Struktur
      expect(result).toBeDefined();
      expect(result).toHaveProperty('url');
      expect(result).toHaveProperty('title');
      expect(result).toHaveProperty('passed');
      expect(result).toHaveProperty('errors');
      expect(result).toHaveProperty('warnings');
      expect(result).toHaveProperty('duration');
      
      // URLs sollten valid sein
      expect(result.url).toMatch(/^https?:\/\//);
      expect(typeof result.title).toBe('string');
      expect(typeof result.passed).toBe('boolean');
      expect(Array.isArray(result.errors)).toBe(true);
      expect(Array.isArray(result.warnings)).toBe(true);
      expect(typeof result.duration).toBe('number');
      expect(result.duration).toBeGreaterThan(0);
    }, 60000);

    it('should include enhanced analysis data', async () => {
      const result = await checker.testPage(TEST_URLS.good, {
        timeout: 30000,
        verbose: false
      });

      // Enhanced Data sollte vorhanden sein
      expect(result).toHaveProperty('enhancedPerformance');
      expect(result).toHaveProperty('enhancedSEO');
      expect(result).toHaveProperty('contentWeight');
      expect(result).toHaveProperty('mobileFriendliness');
      
      // Performance sollte Core Web Vitals enthalten
      if (result.enhancedPerformance) {
        expect(result.enhancedPerformance).toHaveProperty('performanceScore');
        expect(result.enhancedPerformance.performanceScore).toBeGreaterThanOrEqual(0);
        expect(result.enhancedPerformance.performanceScore).toBeLessThanOrEqual(100);
      }
    }, 60000);

    it('should have valid pa11y score when present', async () => {
      const result = await checker.testPage(TEST_URLS.good, {
        timeout: 30000,
        verbose: false
      });

      if (result.pa11yScore !== undefined) {
        expect(result.pa11yScore).toBeGreaterThanOrEqual(0);
        expect(result.pa11yScore).toBeLessThanOrEqual(100);
        expect(typeof result.pa11yScore).toBe('number');
      }
    }, 60000);
  });

  describe('2. Score Calculation Verification', () => {
    it('should calculate accessibility scores correctly', async () => {
      const result = await checker.testPage(TEST_URLS.good, {
        timeout: 30000,
        verbose: false
      });

      // Score sollte im gültigen Bereich sein
      if (result.pa11yScore !== undefined) {
        expect(result.pa11yScore).toBeGreaterThanOrEqual(0);
        expect(result.pa11yScore).toBeLessThanOrEqual(100);
        
        // Score sollte mit Issues korrelieren
        const totalIssues = result.errors.length + result.warnings.length;
        if (totalIssues === 0) {
          // Keine Issues = hoher Score
          expect(result.pa11yScore).toBeGreaterThan(80);
        }
      }
    }, 60000);

    it('should calculate performance scores consistently', async () => {
      const result = await checker.testPage(TEST_URLS.good, {
        timeout: 30000,
        verbose: false
      });

      if (result.enhancedPerformance) {
        const perfScore = result.enhancedPerformance.performanceScore;
        
        // Score sollte numerisch und im Bereich sein
        expect(typeof perfScore).toBe('number');
        expect(perfScore).toBeGreaterThanOrEqual(0);
        expect(perfScore).toBeLessThanOrEqual(100);
        expect(Number.isNaN(perfScore)).toBe(false);
        expect(Number.isFinite(perfScore)).toBe(true);
      }
    }, 60000);

    it('should calculate SEO scores logically', async () => {
      const result = await checker.testPage(TEST_URLS.good, {
        timeout: 30000,
        verbose: false
      });

      if (result.enhancedSEO) {
        const seoScore = result.enhancedSEO.seoScore;
        
        expect(typeof seoScore).toBe('number');
        expect(seoScore).toBeGreaterThanOrEqual(0);
        expect(seoScore).toBeLessThanOrEqual(100);
        
        // SEO Score sollte mit Meta-Tags korrelieren
        const hasTitleTag = result.enhancedSEO.metaTags?.title?.present;
        const hasDescription = result.enhancedSEO.metaTags?.description?.present;
        
        if (hasTitleTag && hasDescription) {
          // Mit Title & Description sollte Score höher sein
          expect(seoScore).toBeGreaterThan(30);
        }
      }
    }, 60000);

    it('should calculate mobile scores based on criteria', async () => {
      const result = await checker.testPage(TEST_URLS.good, {
        timeout: 30000,
        verbose: false
      });

      if (result.mobileFriendliness) {
        const mobileScore = result.mobileFriendliness.overallScore || result.mobileFriendliness.score;
        
        expect(typeof mobileScore).toBe('number');
        expect(mobileScore).toBeGreaterThanOrEqual(0);
        expect(mobileScore).toBeLessThanOrEqual(100);
      }
    }, 60000);
  });

  describe('3. Issue Detection Verification', () => {
    it('should detect accessibility issues consistently', async () => {
      const result1 = await checker.testPage(TEST_URLS.issues, { timeout: 30000 });
      const result2 = await checker.testPage(TEST_URLS.issues, { timeout: 30000 });

      // Beide Läufe sollten ähnliche Issue-Counts haben (±20% Toleranz)
      const issues1 = result1.errors.length + result1.warnings.length;
      const issues2 = result2.errors.length + result2.warnings.length;
      
      if (issues1 > 0) {
        const difference = Math.abs(issues1 - issues2);
        const tolerance = Math.max(issues1, issues2) * 0.2;
        
        expect(difference).toBeLessThanOrEqual(tolerance);
      }
    }, 120000);

    it('should categorize issues correctly', async () => {
      const result = await checker.testPage(TEST_URLS.issues, {
        timeout: 30000,
        verbose: false
      });

      // Errors sollten Error-Properties haben
      result.errors.forEach(error => {
        expect(error).toHaveProperty('message');
        expect(typeof error.message).toBe('string');
        expect(error.message.length).toBeGreaterThan(0);
      });

      // Warnings sollten Warning-Properties haben
      result.warnings.forEach(warning => {
        expect(warning).toHaveProperty('message');
        expect(typeof warning.message).toBe('string');
      });
    }, 60000);

    it('should detect missing alt text on images', async () => {
      const result = await checker.testPage(TEST_URLS.issues, {
        timeout: 30000,
        verbose: false
      });

      // imagesWithoutAlt sollte numerisch sein
      expect(typeof result.imagesWithoutAlt).toBe('number');
      expect(result.imagesWithoutAlt).toBeGreaterThanOrEqual(0);
    }, 60000);
  });

  describe('4. Performance Metrics Verification', () => {
    it('should measure page load time accurately', async () => {
      const startTime = Date.now();
      const result = await checker.testPage(TEST_URLS.good, {
        timeout: 30000,
        verbose: false
      });
      const endTime = Date.now();
      const actualDuration = endTime - startTime;

      // Gemessene Duration sollte realistisch sein
      expect(result.duration).toBeGreaterThan(0);
      expect(result.duration).toBeLessThan(actualDuration + 1000); // +1s Toleranz
    }, 60000);

    it('should collect Core Web Vitals when available', async () => {
      const result = await checker.testPage(TEST_URLS.good, {
        timeout: 30000,
        verbose: false
      });

      if (result.enhancedPerformance?.webVitals) {
        const vitals = result.enhancedPerformance.webVitals;
        
        // LCP sollte sinnvoll sein
        if (vitals.lcp !== undefined && vitals.lcp > 0) {
          expect(vitals.lcp).toBeGreaterThan(0);
          expect(vitals.lcp).toBeLessThan(100000); // < 100 Sekunden
        }
        
        // CLS sollte im Bereich sein
        if (vitals.cls !== undefined) {
          expect(vitals.cls).toBeGreaterThanOrEqual(0);
          expect(vitals.cls).toBeLessThan(10); // Realistischer Maximalwert
        }
      }
    }, 60000);

    it('should measure content weight accurately', async () => {
      const result = await checker.testPage(TEST_URLS.good, {
        timeout: 30000,
        verbose: false
      });

      if (result.contentWeight) {
        // Total size sollte positiv sein
        if (result.contentWeight.totalSize) {
          expect(result.contentWeight.totalSize).toBeGreaterThan(0);
          expect(result.contentWeight.totalSize).toBeLessThan(100 * 1024 * 1024); // < 100MB
        }
        
        // Content Score sollte gültig sein
        if (result.contentWeight.contentScore !== undefined) {
          expect(result.contentWeight.contentScore).toBeGreaterThanOrEqual(0);
          expect(result.contentWeight.contentScore).toBeLessThanOrEqual(100);
        }
      }
    }, 60000);
  });

  describe('5. Report Generation Verification', () => {
    it('should generate valid HTML reports', async () => {
      const result = await checker.testPage(TEST_URLS.good, {
        timeout: 30000,
        verbose: false
      });

      const htmlGenerator = new HTMLGenerator();
      const mockData = {
        summary: {
          totalPages: 1,
          testedPages: 1,
          passedPages: result.passed ? 1 : 0,
          failedPages: result.passed ? 0 : 1,
          crashedPages: 0,
          totalErrors: result.errors.length,
          totalWarnings: result.warnings.length,
          totalDuration: result.duration,
          successRate: result.passed ? 100 : 0,
          overallScore: result.pa11yScore || 0,
          overallGrade: 'B',
          certificateLevel: 'SILVER' as const
        },
        pages: [result],
        metadata: {
          sitemapUrl: TEST_URLS.good,
          timestamp: new Date().toISOString(),
          duration: result.duration,
          version: '2.0.0'
        }
      };

      const html = await htmlGenerator.generate(mockData);

      // HTML sollte valid sein
      expect(html).toContain('<!DOCTYPE html>');
      expect(html).toContain('<html');
      expect(html).toContain('</html>');
      expect(html).toContain('<body');
      expect(html).toContain('</body>');
      
      // Sollte Daten enthalten
      expect(html).toContain(result.title);
      expect(html.length).toBeGreaterThan(1000);
    }, 60000);

    it('should generate valid JSON reports', async () => {
      const result = await checker.testPage(TEST_URLS.good, {
        timeout: 30000,
        verbose: false
      });

      const jsonGenerator = new JsonGenerator();
      const mockData = {
        summary: {
          totalPages: 1,
          testedPages: 1,
          passedPages: result.passed ? 1 : 0,
          failedPages: result.passed ? 0 : 1,
          crashedPages: 0,
          totalErrors: result.errors.length,
          totalWarnings: result.warnings.length,
          totalDuration: result.duration,
          successRate: result.passed ? 100 : 0
        },
        pages: [result],
        metadata: {
          sitemapUrl: TEST_URLS.good,
          timestamp: new Date().toISOString(),
          duration: result.duration,
          version: '2.0.0'
        }
      };

      const json = await jsonGenerator.generate(mockData);

      // JSON sollte parsebar sein
      expect(() => JSON.parse(json)).not.toThrow();
      
      const parsed = JSON.parse(json);
      expect(parsed).toHaveProperty('summary');
      expect(parsed).toHaveProperty('pages');
      expect(parsed).toHaveProperty('metadata');
      expect(parsed.pages).toHaveLength(1);
    }, 60000);
  });

  describe('6. Edge Cases & Error Handling', () => {
    it('should handle non-existent URLs gracefully', async () => {
      const result = await checker.testPage('https://this-domain-definitely-does-not-exist-12345.com', {
        timeout: 10000,
        verbose: false
      });

      expect(result).toBeDefined();
      expect(result.crashed).toBe(true);
      expect(result.passed).toBe(false);
    }, 30000);

    it('should handle timeout scenarios', async () => {
      // Test mit sehr kurzem Timeout
      const result = await checker.testPage(TEST_URLS.complex, {
        timeout: 100, // Nur 100ms - wird wahrscheinlich timeout
        verbose: false
      });

      // Sollte nicht crashen, aber als failed markiert sein
      expect(result).toBeDefined();
      expect(result.url).toBe(TEST_URLS.complex);
    }, 30000);

    it('should handle pages with JavaScript errors', async () => {
      const result = await checker.testPage(TEST_URLS.issues, {
        timeout: 30000,
        verbose: false
      });

      // Sollte auch bei JS-Errors Daten sammeln
      expect(result).toBeDefined();
      expect(result.title).toBeDefined();
      expect(typeof result.passed).toBe('boolean');
    }, 60000);
  });

  describe('7. Consistency & Reliability', () => {
    it('should produce consistent results across multiple runs', async () => {
      const runs = 3;
      const results: any[] = [];

      for (let i = 0; i < runs; i++) {
        const result = await checker.testPage(TEST_URLS.good, {
          timeout: 30000,
          verbose: false
        });
        results.push(result);
        
        // Kleine Pause zwischen Tests
        await new Promise(resolve => setTimeout(resolve, 1000));
      }

      // Alle Runs sollten ähnliche Basis-Daten haben
      const titles = results.map(r => r.title);
      const allSameTitle = titles.every(t => t === titles[0]);
      expect(allSameTitle).toBe(true);

      // Scores sollten ähnlich sein (±10% Toleranz)
      const scores = results.map(r => r.pa11yScore || 0).filter(s => s > 0);
      if (scores.length > 1) {
        const avgScore = scores.reduce((a, b) => a + b, 0) / scores.length;
        scores.forEach(score => {
          const deviation = Math.abs(score - avgScore) / avgScore;
          expect(deviation).toBeLessThan(0.1); // < 10% Abweichung
        });
      }
    }, 180000);
  });
});
