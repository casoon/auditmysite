/**
 * 🧪 Compiled CLI Integration Tests
 * 
 * Tests the compiled CLI using the actual bin/audit.js file
 * to catch import/export issues that Jest's in-memory compilation misses.
 */

import { spawn } from 'child_process';
import * as path from 'path';

const CLI_PATH = path.join(__dirname, '../../bin/audit.js');
const TIMEOUT = 5000; // 5 second timeout

/**
 * Runs CLI command and returns stdout/stderr with timeout
 */
function runCLI(args: string[] = [], timeoutMs = TIMEOUT): Promise<{
  stdout: string;
  stderr: string;
  code: number | null;
  timeout: boolean;
}> {
  return new Promise((resolve) => {
    const child = spawn('node', [CLI_PATH, ...args], {
      stdio: 'pipe',
      timeout: timeoutMs
    });

    let stdout = '';
    let stderr = '';
    let timeout = false;

    child.stdout?.on('data', (data) => {
      stdout += data.toString();
    });

    child.stderr?.on('data', (data) => {
      stderr += data.toString();
    });

    const timer = setTimeout(() => {
      timeout = true;
      child.kill('SIGTERM');
    }, timeoutMs);

    child.on('close', (code) => {
      clearTimeout(timer);
      resolve({ stdout, stderr, code, timeout });
    });

    child.on('error', (error) => {
      clearTimeout(timer);
      resolve({ 
        stdout, 
        stderr: stderr + error.message, 
        code: null, 
        timeout 
      });
    });
  });
}

describe('Compiled CLI Integration', () => {
  describe('CLI Executable', () => {
    it('should load without module import errors', async () => {
      // Test help command - should load all modules
      const result = await runCLI(['--help']);
      
      // Should not contain module loading errors
      expect(result.stderr).not.toMatch(/Cannot find module/);
      expect(result.stderr).not.toMatch(/Module not found/);
      expect(result.stderr).not.toMatch(/Error: Cannot resolve/);
      
      // Help should be displayed
      expect(result.stdout).toMatch(/Usage:/);
      expect(result.code).toBe(0);
    }, 10000);

    it('should show version without errors', async () => {
      const result = await runCLI(['--version']);
      
      expect(result.stderr).not.toMatch(/Cannot find module/);
      expect(result.stdout).toMatch(/2\.0\.[0-9a-z.-]+/); // Match version pattern
      expect(result.code).toBe(0);
    });

    it('should load modules when processing sitemap', async () => {
      // Test with invalid URL to avoid actual network calls, 
      // but should still load the modules
      const result = await runCLI([
        'https://invalid-test-url-that-should-fail-validation',
        '--max-pages', '1',
        '--non-interactive'
      ]);
      
      // Should fail due to invalid URL, not module loading
      expect(result.stderr).not.toMatch(/Cannot find module.*reports/);
      expect(result.stderr).not.toMatch(/UnifiedReportSystem/);
      
      // Should show URL validation error instead
      expect(result.stderr || result.stdout).toMatch(/Invalid.*URL|Validation failed|No sitemap found/);
    });

    it('should handle format options without import errors', async () => {
      const result = await runCLI([
        'https://invalid-test-url',
        '--format', 'markdown',
        '--format', 'json',
        '--max-pages', '1',
        '--non-interactive'
      ]);
      
      // Should not crash on module imports
      expect(result.stderr).not.toMatch(/Cannot find module/);
      expect(result.stderr).not.toMatch(/ModernMarkdownReportGenerator/);
      expect(result.stderr).not.toMatch(/JSONReportGenerator/);
    });
  });

  describe('Error Scenarios', () => {
    it('should handle missing sitemap URL gracefully', async () => {
      const result = await runCLI([]);
      
      expect(result.stderr).not.toMatch(/Cannot find module/);
      expect(result.stderr).toMatch(/missing required argument/);
      expect(result.code).toBe(1);
    });

    it('should validate arguments without module errors', async () => {
      const result = await runCLI([
        'invalid-url',
        '--max-pages', 'invalid',
      ]);
      
      expect(result.stderr).not.toMatch(/Cannot find module/);
      // Should show validation error for max-pages or URL (case insensitive)
      expect(result.stderr || result.stdout).toMatch(/Invalid|error|Configuration/i);
    });
  });

  describe('Module Loading Validation', () => {
    it('should import all critical report generators', async () => {
      // This test ensures CLI can load without crashing on imports
      const result = await runCLI(['--help']);
      
      // These are the modules that were causing issues
      expect(result.stderr).not.toMatch(/Cannot find module.*reports.*index/);
      expect(result.stderr).not.toMatch(/Cannot find module.*unified/);
      expect(result.stderr).not.toMatch(/UnifiedReportSystem.*not.*found/);
      
      expect(result.code).toBe(0);
    });

    it('should handle performance budget options', async () => {
      const result = await runCLI([
        'https://test.invalid',
        '--budget', 'ecommerce',
        '--non-interactive'
      ]);
      
      // Should not fail on module imports
      expect(result.stderr).not.toMatch(/Cannot find module/);
      
      // Will fail on URL validation, which is expected
      expect(result.code).not.toBe(0);
    });
  });

  describe('Advanced Features', () => {
    it('should handle verbose output without errors', async () => {
      const result = await runCLI([
        'https://test.invalid',
        '--verbose',
        '--non-interactive'
      ]);
      
      expect(result.stderr).not.toMatch(/Cannot find module/);
    });

    it('should handle API mode initialization', async () => {
      // Test API mode startup
      const result = await runCLI([
        '--api',
        '--port', '0',
        '--no-browser'
      ], 2000); // Short timeout since we just want to test initialization
      
      expect(result.stderr).not.toMatch(/Cannot find module/);
    });
  });
});

describe('Build Artifact Validation', () => {
  it('should have all required files in dist/', () => {
    const fs = require('fs');
    const requiredFiles = [
      'dist/cli/commands/audit-command.js',
      'dist/generators/html-generator.js',
      'dist/generators/markdown-generator.js',
    ];

    for (const file of requiredFiles) {
      const fullPath = path.join(__dirname, '../../', file);
      expect(fs.existsSync(fullPath)).toBe(true);
    }
  });

  it('should export HTMLGenerator', () => {
    const generatorsIndex = require('../../dist/generators/html-generator.js');
    expect(generatorsIndex.HTMLGenerator).toBeDefined();
    expect(typeof generatorsIndex.HTMLGenerator).toBe('function');
  });

  it('should export modern generators', () => {
    const htmlGenerator = require('../../dist/generators/html-generator.js');
    const markdownGenerator = require('../../dist/generators/markdown-generator.js');
    
    expect(htmlGenerator.HTMLGenerator).toBeDefined();
    expect(markdownGenerator.MarkdownGenerator).toBeDefined();
  });
});
