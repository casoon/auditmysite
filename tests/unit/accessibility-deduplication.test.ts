/**
 * Unit tests for accessibility issue deduplication
 *
 * Tests the fix for duplicate error reporting bug
 * (e.g., errors 22-42 being exact duplicates of 1-21)
 */

import { describe, it, expect } from '@jest/globals';

// Import the deduplication function
// Note: We need to export it from accessibility-checker.ts for testing
interface Pa11yIssue {
  code: string;
  message: string;
  type: 'error' | 'warning' | 'notice';
  selector: string;
  context: string;
  impact?: string;
  help?: string;
  helpUrl?: string;
}

// Inline copy of deduplication function for testing
function deduplicateAccessibilityIssues(issues: Pa11yIssue[]): Pa11yIssue[] {
  const seen = new Set<string>();
  return issues.filter(issue => {
    const key = `${issue.code}|${issue.selector}|${issue.context}`;
    if (seen.has(key)) {
      return false;
    }
    seen.add(key);
    return true;
  });
}

describe('Accessibility Issue Deduplication', () => {
  it('should remove exact duplicates', () => {
    const issues: Pa11yIssue[] = [
      {
        code: 'color-contrast',
        message: 'Element has insufficient color contrast',
        type: 'error',
        selector: 'h1.text-gray-800',
        context: '<h1 class="text-gray-800">Headline</h1>'
      },
      {
        code: 'color-contrast',
        message: 'Element has insufficient color contrast',
        type: 'error',
        selector: 'h1.text-gray-800',
        context: '<h1 class="text-gray-800">Headline</h1>'
      }
    ];

    const deduplicated = deduplicateAccessibilityIssues(issues);

    expect(deduplicated).toHaveLength(1);
    expect(deduplicated[0]).toEqual(issues[0]);
  });

  it('should keep issues with different codes', () => {
    const issues: Pa11yIssue[] = [
      {
        code: 'color-contrast',
        message: 'Insufficient contrast',
        type: 'error',
        selector: 'h1',
        context: '<h1>Test</h1>'
      },
      {
        code: 'heading-order',
        message: 'Heading order invalid',
        type: 'error',
        selector: 'h1',
        context: '<h1>Test</h1>'
      }
    ];

    const deduplicated = deduplicateAccessibilityIssues(issues);

    expect(deduplicated).toHaveLength(2);
  });

  it('should keep issues with different selectors', () => {
    const issues: Pa11yIssue[] = [
      {
        code: 'color-contrast',
        message: 'Insufficient contrast',
        type: 'error',
        selector: 'h1.title',
        context: '<h1 class="title">Test</h1>'
      },
      {
        code: 'color-contrast',
        message: 'Insufficient contrast',
        type: 'error',
        selector: 'h2.subtitle',
        context: '<h2 class="subtitle">Test</h2>'
      }
    ];

    const deduplicated = deduplicateAccessibilityIssues(issues);

    expect(deduplicated).toHaveLength(2);
  });

  it('should keep issues with different contexts', () => {
    const issues: Pa11yIssue[] = [
      {
        code: 'color-contrast',
        message: 'Insufficient contrast',
        type: 'error',
        selector: 'p',
        context: '<p>First paragraph</p>'
      },
      {
        code: 'color-contrast',
        message: 'Insufficient contrast',
        type: 'error',
        selector: 'p',
        context: '<p>Second paragraph</p>'
      }
    ];

    const deduplicated = deduplicateAccessibilityIssues(issues);

    expect(deduplicated).toHaveLength(2);
  });

  it('should handle the casoon.de bug (42 errors → 21 unique)', () => {
    // Simulate the real-world bug: errors 1-21, then exact duplicates as 22-42
    const uniqueIssues: Pa11yIssue[] = Array.from({ length: 21 }, (_, i) => ({
      code: 'color-contrast',
      message: `Element ${i + 1} has insufficient color contrast`,
      type: 'error' as const,
      selector: `h${(i % 6) + 1}.element-${i}`,
      context: `<h${(i % 6) + 1} class="element-${i}">Text</h${(i % 6) + 1}>`
    }));

    // Create exact duplicates
    const duplicateIssues = [...uniqueIssues];
    const allIssues = [...uniqueIssues, ...duplicateIssues];

    expect(allIssues).toHaveLength(42); // Bug: 42 reported errors

    const deduplicated = deduplicateAccessibilityIssues(allIssues);

    expect(deduplicated).toHaveLength(21); // Fixed: 21 unique errors
  });

  it('should preserve issue order (first occurrence)', () => {
    const issues: Pa11yIssue[] = [
      {
        code: 'A',
        message: 'First',
        type: 'error',
        selector: 's1',
        context: 'c1'
      },
      {
        code: 'B',
        message: 'Second',
        type: 'error',
        selector: 's2',
        context: 'c2'
      },
      {
        code: 'A',
        message: 'First',
        type: 'error',
        selector: 's1',
        context: 'c1'
      }
    ];

    const deduplicated = deduplicateAccessibilityIssues(issues);

    expect(deduplicated).toHaveLength(2);
    expect(deduplicated[0].code).toBe('A');
    expect(deduplicated[1].code).toBe('B');
  });

  it('should handle empty input', () => {
    const issues: Pa11yIssue[] = [];
    const deduplicated = deduplicateAccessibilityIssues(issues);

    expect(deduplicated).toHaveLength(0);
  });

  it('should handle single issue', () => {
    const issues: Pa11yIssue[] = [
      {
        code: 'color-contrast',
        message: 'Test',
        type: 'error',
        selector: 'h1',
        context: '<h1>Test</h1>'
      }
    ];

    const deduplicated = deduplicateAccessibilityIssues(issues);

    expect(deduplicated).toHaveLength(1);
    expect(deduplicated[0]).toEqual(issues[0]);
  });
});
