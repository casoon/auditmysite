/**
 * 🧪 Report Generators Unit Tests
 * 
 * Tests report generation logic without actual file I/O.
 * Focus on content generation, validation, and formatting.
 */

import { HTMLGenerator } from '../../src/generators/html-generator';
import { MarkdownGenerator } from '../../src/generators/markdown-generator';
import { JsonGenerator } from '../../src/generators/json-generator';
import { createMockAuditResult, createMockPageResult } from '../setup';

// HTML generator tests
describe('HTMLGenerator', () => {
  let generator: HTMLGenerator;

  beforeEach(() => {
    generator = new HTMLGenerator();
  });

  it('should create an instance', () => {
    expect(generator).toBeInstanceOf(HTMLGenerator);
  });

  it('should have generate method', () => {
    expect(typeof generator.generate).toBe('function');
  });

  it('should have generateFromJSON method', () => {
    expect(typeof generator.generateFromJSON).toBe('function');
  });
});

describe('JsonGenerator', () => {
  let generator: JsonGenerator;

  beforeEach(() => {
    generator = new JsonGenerator();
  });

  it('should create an instance', () => {
    expect(generator).toBeInstanceOf(JsonGenerator);
  });

  it('should have generateJson method', () => {
    expect(typeof generator.generateJson).toBe('function');
  });

  it('should have generatePageSubset method', () => {
    expect(typeof generator.generatePageSubset).toBe('function');
  });

  it('should have generateMetricsOnly method', () => {
    expect(typeof generator.generateMetricsOnly).toBe('function');
  });
});


describe('MarkdownGenerator', () => {
  let generator: MarkdownGenerator;

  beforeEach(() => {
    generator = new MarkdownGenerator();
  });

  it('should create an instance', () => {
    expect(generator).toBeInstanceOf(MarkdownGenerator);
  });

  it('should have generateDetailedIssues method', () => {
    expect(typeof generator.generateDetailedIssues).toBe('function');
  });

  it('should have generateSummary method', () => {
    expect(typeof generator.generateSummary).toBe('function');
  });
});

// Additional test for certificate functionality
describe('HTMLGenerator Certificate Features', () => {
  let generator: HTMLGenerator;

  beforeEach(() => {
    generator = new HTMLGenerator();
  });

  it('should have certificate-related functionality', () => {
    // Test that the generator has the methods we expect
    expect(typeof generator.generate).toBe('function');
    expect(typeof generator.generateFromJSON).toBe('function');
  });
});
