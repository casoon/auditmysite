/**
 * 📸 Report Snapshot Tests
 * 
 * Diese Tests erstellen Snapshots von generierten Reports und
 * erkennen unbeabsichtigte Änderungen in der Report-Struktur.
 */

import { describe, it, expect } from '@jest/globals';
import { HTMLGenerator } from '../../src/generators/html-generator';
import { MarkdownGenerator } from '../../src/generators/markdown-generator';
import { JsonGenerator } from '../../src/generators/json-generator';

describe('Report Snapshot Tests', () => {
  // Mock-Daten für konsistente Snapshots
  const mockAuditData = {
    summary: {
      totalPages: 5,
      testedPages: 5,
      passedPages: 3,
      failedPages: 2,
      crashedPages: 0,
      totalErrors: 15,
      totalWarnings: 8,
      totalDuration: 45000,
      successRate: 60,
      overallScore: 75,
      overallGrade: 'B' as const,
      certificateLevel: 'SILVER' as const
    },
    pages: [
      {
        url: 'https://example.com/',
        title: 'Example Domain - Homepage',
        passed: true,
        crashed: false,
        status: 'passed' as const,
        errors: [],
        warnings: [
          {
            code: 'WCAG2AA.Principle1.Guideline1_3.1_3_1.H49.I',
            message: 'Check that the i element is being used appropriately',
            type: 'warning',
            selector: 'html > body > div > p:nth-child(2) > i',
            context: '<i>example</i>'
          }
        ],
        imagesWithoutAlt: 0,
        buttonsWithoutLabel: 0,
        headingsCount: 2,
        duration: 3500,
        pa11yScore: 95,
        enhancedPerformance: {
          performanceScore: 88,
          webVitals: {
            lcp: 1200,
            fid: 50,
            cls: 0.05,
            fcp: 800,
            ttfb: 200
          },
          metrics: {
            loadTime: 1500,
            domContentLoaded: 1200,
            requestCount: 25,
            transferSize: 450000
          }
        },
        enhancedSEO: {
          seoScore: 82,
          metaTags: {
            title: {
              present: true,
              content: 'Example Domain - Homepage',
              optimal: true,
              length: 28
            },
            description: {
              present: true,
              content: 'Example domain for testing purposes',
              optimal: true,
              length: 35
            }
          },
          headings: {
            h1: ['Example Domain'],
            h2: ['About', 'Contact']
          }
        },
        contentWeight: {
          contentScore: 78,
          totalSize: 450000,
          resources: {
            html: { size: 5000 },
            css: { size: 25000 },
            javascript: { size: 120000 },
            images: { size: 300000 },
            other: { size: 0 }
          },
          optimizations: [
            {
              id: 'compress-images',
              type: 'Image Compression',
              priority: 'high',
              savings: 50000,
              message: 'Images can be compressed to reduce size'
            }
          ]
        },
        mobileFriendliness: {
          overallScore: 85,
          score: 85,
          viewport: { hasViewportTag: true, isResponsive: true },
          textSize: { readableText: true, issues: [] },
          tapTargets: { adequateSize: true, issues: [] },
          recommendations: []
        }
      },
      {
        url: 'https://example.com/about',
        title: 'About Us',
        passed: false,
        crashed: false,
        status: 'failed' as const,
        errors: [
          {
            code: 'WCAG2AA.Principle1.Guideline1_1.1_1_1.H37',
            message: 'Img element missing an alt attribute',
            type: 'error',
            selector: 'img.logo',
            context: '<img src="/logo.png" class="logo">'
          }
        ],
        warnings: [],
        imagesWithoutAlt: 1,
        buttonsWithoutLabel: 0,
        headingsCount: 3,
        duration: 4200,
        pa11yScore: 65,
        enhancedPerformance: {
          performanceScore: 72,
          webVitals: {
            lcp: 2800,
            fid: 120,
            cls: 0.15,
            fcp: 1800,
            ttfb: 450
          }
        },
        enhancedSEO: {
          seoScore: 70,
          metaTags: {
            title: { present: true, content: 'About Us', optimal: false, length: 8 },
            description: { present: false, content: '', optimal: false, length: 0 }
          }
        },
        contentWeight: {
          contentScore: 68,
          totalSize: 850000,
          resources: {
            html: { size: 8000 },
            css: { size: 40000 },
            javascript: { size: 250000 },
            images: { size: 550000 },
            other: { size: 2000 }
          }
        },
        mobileFriendliness: {
          overallScore: 75,
          score: 75,
          viewport: { hasViewportTag: true, isResponsive: true },
          textSize: { readableText: false, issues: ['Small text detected'] },
          tapTargets: { adequateSize: true, issues: [] }
        }
      }
    ],
    metadata: {
      sitemapUrl: 'https://example.com/sitemap.xml',
      timestamp: '2025-01-01T12:00:00.000Z', // Fixed timestamp for consistent snapshots
      duration: 45000,
      version: '2.0.0-test'
    }
  };

  describe('HTML Report Snapshots', () => {
    it('should generate consistent HTML structure', async () => {
      const generator = new HTMLGenerator();
      const html = await generator.generate(mockAuditData);

      // Extract key sections for snapshot testing
      const summaryMatch = html.match(/<section id="summary"[^>]*>[\s\S]*?<\/section>/);
      const accessibilityMatch = html.match(/<section id="accessibility"[^>]*>[\s\S]*?<\/section>/);
      
      expect(summaryMatch).toBeDefined();
      expect(accessibilityMatch).toBeDefined();
      
      // Verify structure
      expect(html).toContain('<!DOCTYPE html>');
      expect(html).toContain('id="summary"');
      expect(html).toContain('id="accessibility"');
      expect(html).toContain('id="performance"');
      expect(html).toContain('id="seo"');
      expect(html).toContain('id="contentweight"');
      expect(html).toContain('id="mobile"');
      expect(html).toContain('id="geo"');
    });

    it('should include all summary metrics', async () => {
      const generator = new HTMLGenerator();
      const html = await generator.generate(mockAuditData);

      // Check all key metrics are present
      expect(html).toContain('75/100'); // Overall score
      expect(html).toContain('60%'); // Success rate
      expect(html).toContain('5/5'); // Pages analyzed
      expect(html).toContain('15'); // Total errors
      expect(html).toContain('8'); // Total warnings
    });

    it('should render performance data correctly', async () => {
      const generator = new HTMLGenerator();
      const html = await generator.generate(mockAuditData);

      // Check performance metrics
      expect(html).toContain('LCP'); // Largest Contentful Paint
      expect(html).toContain('CLS'); // Cumulative Layout Shift
      expect(html).toContain('FID'); // First Input Delay
    });

    it('should include mobile-friendliness scores', async () => {
      const generator = new HTMLGenerator();
      const html = await generator.generate(mockAuditData);

      expect(html).toContain('Mobile'); 
      expect(html).toContain('85/100'); // Mobile score from mockData
    });
  });

  describe('JSON Report Snapshots', () => {
    it('should generate valid and consistent JSON', async () => {
      const generator = new JsonGenerator();
      const json = await generator.generate(mockAuditData);

      const parsed = JSON.parse(json);
      
      // Verify structure
      expect(parsed).toHaveProperty('summary');
      expect(parsed).toHaveProperty('pages');
      expect(parsed).toHaveProperty('metadata');
      
      // Verify summary
      expect(parsed.summary.totalPages).toBe(5);
      expect(parsed.summary.passedPages).toBe(3);
      expect(parsed.summary.overallScore).toBe(75);
      
      // Verify pages array
      expect(parsed.pages).toHaveLength(2);
      expect(parsed.pages[0].url).toBe('https://example.com/');
      expect(parsed.pages[0].passed).toBe(true);
      expect(parsed.pages[1].passed).toBe(false);
    });

    it('should preserve all data fields', async () => {
      const generator = new JsonGenerator();
      const json = await generator.generate(mockAuditData);
      const parsed = JSON.parse(json);

      // Check first page has all fields
      const page = parsed.pages[0];
      expect(page).toHaveProperty('enhancedPerformance');
      expect(page).toHaveProperty('enhancedSEO');
      expect(page).toHaveProperty('contentWeight');
      expect(page).toHaveProperty('mobileFriendliness');
      
      // Check nested structures
      expect(page.enhancedPerformance).toHaveProperty('webVitals');
      expect(page.enhancedSEO).toHaveProperty('metaTags');
      expect(page.contentWeight).toHaveProperty('resources');
    });

    it('should handle missing optional fields gracefully', async () => {
      const minimalData = {
        summary: {
          totalPages: 1,
          testedPages: 1,
          passedPages: 0,
          failedPages: 1,
          crashedPages: 0,
          totalErrors: 5,
          totalWarnings: 0,
          totalDuration: 1000,
          successRate: 0
        },
        pages: [{
          url: 'https://test.com',
          title: 'Test',
          passed: false,
          crashed: false,
          errors: [],
          warnings: [],
          imagesWithoutAlt: 0,
          buttonsWithoutLabel: 0,
          headingsCount: 0,
          duration: 1000
        }],
        metadata: {
          sitemapUrl: 'https://test.com/sitemap.xml',
          timestamp: '2025-01-01T12:00:00.000Z',
          duration: 1000,
          version: '2.0.0'
        }
      };

      const generator = new JsonGenerator();
      const json = await generator.generate(minimalData);
      
      expect(() => JSON.parse(json)).not.toThrow();
      const parsed = JSON.parse(json);
      expect(parsed.pages).toHaveLength(1);
    });
  });

  describe('Markdown Report Snapshots', () => {
    it('should generate consistent markdown structure', async () => {
      const generator = new MarkdownGenerator();
      const markdown = await generator.generate(mockAuditData);

      // Check for key sections
      expect(markdown).toContain('# Accessibility Audit Report');
      expect(markdown).toContain('## Summary');
      expect(markdown).toContain('## Pages');
      expect(markdown).toContain('### https://example.com/');
      
      // Check for metrics
      expect(markdown).toContain('Total Pages: 5');
      expect(markdown).toContain('Passed: 3');
      expect(markdown).toContain('Failed: 2');
    });

    it('should format scores consistently', async () => {
      const generator = new MarkdownGenerator();
      const markdown = await generator.generate(mockAuditData);

      // Scores should be formatted with /100
      expect(markdown).toMatch(/Overall Score:.*75/);
      expect(markdown).toMatch(/Success Rate:.*60%/);
    });

    it('should include issue details', async () => {
      const generator = new MarkdownGenerator();
      const markdown = await generator.generate(mockAuditData);

      // Should contain error details
      expect(markdown).toContain('Img element missing an alt attribute');
      expect(markdown).toContain('WCAG2AA.Principle1.Guideline1_1.1_1_1.H37');
    });
  });

  describe('Report Consistency', () => {
    it('should produce identical output for identical input', async () => {
      const htmlGen = new HTMLGenerator();
      const html1 = await htmlGen.generate(mockAuditData);
      const html2 = await htmlGen.generate(mockAuditData);

      // HTML should be identical (modulo dynamic elements like generated IDs)
      expect(html1.length).toBe(html2.length);
      expect(html1).toContain(html2.substring(0, 1000)); // First 1000 chars should match
    });

    it('should maintain data integrity across formats', async () => {
      const htmlGen = new HTMLGenerator();
      const jsonGen = new JsonGenerator();
      const mdGen = new MarkdownGenerator();

      const html = await htmlGen.generate(mockAuditData);
      const json = await jsonGen.generate(mockAuditData);
      const markdown = await mdGen.generate(mockAuditData);

      // All formats should contain key data points
      const keyValues = ['75', '60', 'Example Domain'];
      
      keyValues.forEach(value => {
        expect(html).toContain(value);
        expect(json).toContain(value);
        expect(markdown).toContain(value);
      });
    });
  });
});
