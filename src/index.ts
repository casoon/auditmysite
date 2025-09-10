#!/usr/bin/env node

import { Command } from "commander";
import chalk from "chalk";
import ora from "ora";
import { SitemapParser } from "./parsers";
import { AccessibilityChecker } from "./core";
import { JsonGenerator } from "./generators";
import { TestOptions, TestSummary } from "./types";

const program = new Command();

program
  .name("a11y-test")
  .description(
    "CLI tool for automated accessibility testing based on sitemap",
  )
  .version("1.0.0");

program
  .argument("<sitemap-url>", "URL to sitemap.xml")
  .option(
    "-m, --max-pages <number>",
    "Maximum number of pages to test",
    "20",
  )
  .option("-t, --timeout <number>", "Timeout in milliseconds", "10000")
  .option(
    "-w, --wait-until <string>",
    "Wait until (domcontentloaded|load|networkidle)",
    "domcontentloaded",
  )
  .option(
    "-f, --filter <patterns>",
    "URL patterns to exclude (comma-separated)",
    "[...slug],[category],/demo/",
  )
  .option(
    "-i, --include <patterns>",
    "URL patterns to include (comma-separated)",
  )
  .option("-v, --verbose", "Detailed output")

  .option(
    "--standard <standard>",
    "Accessibility Standard (WCAG2A|WCAG2AA|WCAG2AAA|Section508)",
    "WCAG2AA"
  )
  .option("--include-details", "Detailed information in output file")
  .option("--include-pa11y", "Include pa11y issues in output file")
  .option("--summary-only", "Summary only without page details")
  .action(async (sitemapUrl: string, options: any) => {
    const spinner = ora("Initializing accessibility tests...").start();

    try {
      // Initialize parser
      const parser = new SitemapParser();
      spinner.text = "Loading sitemap...";

      // Parse sitemap
      const urls = await parser.parseSitemap(sitemapUrl);
      spinner.text = `Sitemap loaded: ${urls.length} URLs found`;

      // Filter URLs
      const filterPatterns = options.filter
        ? options.filter.split(",")
        : ["[...slug]", "[category]", "/demo/"];
      const includePatterns = options.include
        ? options.include.split(",")
        : undefined;

      const filteredUrls = parser.filterUrls(urls, {
        filterPatterns,
        includePatterns,
      });
      spinner.text = `URLs filtered: ${filteredUrls.length} URLs to test`;

      // Convert URLs to local URLs (if needed)
      const baseUrl = new URL(sitemapUrl).origin;
      const localUrls = parser.convertToLocalUrls(filteredUrls, baseUrl);

      // Initialize accessibility checker
      const checker = new AccessibilityChecker();
      await checker.initialize();

      spinner.text = "Running accessibility tests...";

      // Run tests
      const testOptions: TestOptions = {
        maxPages: parseInt(options.maxPages),
        timeout: parseInt(options.timeout),
        waitUntil: options.waitUntil,
        verbose: options.verbose,
        pa11yStandard: options.standard,
      };

      const results = await checker.testMultiplePages(
        localUrls.map((url) => url.loc),
        testOptions,
      );

      // Create summary
      const summary: TestSummary = {
        totalPages: localUrls.length,
        testedPages: results.length,
        passedPages: results.filter((r: any) => r.passed).length,
        failedPages: results.filter((r: any) => !r.passed && !r.crashed).length,
        crashedPages: results.filter((r: any) => r.crashed === true).length,
        totalErrors: results.reduce((sum: number, r: any) => sum + r.errors.length, 0),
        totalWarnings: results.reduce((sum: number, r: any) => sum + r.warnings.length, 0),
        totalDuration: results.reduce((sum: number, r: any) => sum + r.duration, 0),
        results,
      };

      await checker.cleanup();

      // Display results
      spinner.succeed("Tests completed!");
      
      // Generate JSON output if requested
      if (options.output && options.output !== 'console') {
        spinner.text = 'Generating JSON output...';
        const jsonGenerator = new JsonGenerator();
        
        try {
          const jsonData = {
            metadata: { timestamp: new Date().toISOString(), toolVersion: '2.0.0-alpha.1' },
            summary,
            pages: summary.results
          };
          const jsonContent = jsonGenerator.generateJson(jsonData as any);
          const outputPath = options.outputFile || `audit-${Date.now()}.json`;
          require('fs').writeFileSync(outputPath, jsonContent);
          spinner.succeed(`JSON output created: ${outputPath}`);
        } catch (error) {
          spinner.warn(`Error creating JSON output: ${error}`);
        }
      }
      
      displayResults(summary, options);
    } catch (error) {
      spinner.fail(`Error: ${error}`);
      process.exit(1);
    }
  });

function displayResults(summary: TestSummary, options: any): void {
  console.log("\n" + chalk.bold.blue("🎯 Accessibility Test Summary"));
  console.log(chalk.gray("─".repeat(50)));

  console.log(`📄 Total pages: ${summary.totalPages}`);
  console.log(`🧪 Tested pages: ${summary.testedPages}`);
  console.log(`✅ Passed: ${chalk.green(summary.passedPages)}`);
  console.log(`❌ Failed: ${chalk.red(summary.failedPages)}`);
  console.log(`⚠️  Warnings: ${chalk.yellow(summary.totalWarnings)}`);
  console.log(`⏱️  Total duration: ${summary.totalDuration}ms`);

  if (options.verbose) {
    console.log("\n" + chalk.bold("📋 Detailed results:"));
    summary.results.forEach((result) => {
      const status = result.passed ? chalk.green("✅") : chalk.red("❌");
      console.log(`${status} ${result.url}`);
      console.log(`   Title: ${result.title}`);
      console.log(`   Duration: ${result.duration}ms`);

      if (result.warnings.length > 0) {
        result.warnings.forEach((warning) => {
          console.log(`   ⚠️  ${warning}`);
        });
      }

      if (result.errors.length > 0) {
        result.errors.forEach((error) => {
          console.log(`   ❌ ${error}`);
        });
      }
    });
  }

  // Only exit with code 1 for technical crashes, not accessibility failures
  if (summary.crashedPages > 0) {
    console.log(`\n❌ ${summary.crashedPages} pages crashed due to technical errors`);
    process.exit(1);
  } else if (summary.failedPages > 0) {
    console.log(`\n⚠️  Note: ${summary.failedPages} pages have accessibility issues (this is normal)`);
    console.log(`💡 Check the detailed report for specific issues to fix`);
  }
}

program.parse();
