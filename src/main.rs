//! auditmysite CLI Entry Point
//!
//! Resource-efficient WCAG 2.1 Accessibility Checker in Rust

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use clap::Parser;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

use auditmysite::audit::{
    parse_sitemap, read_url_file, run_concurrent_batch, run_single_audit, BatchConfig,
    PipelineConfig,
};
use auditmysite::browser::{find_chrome, BrowserManager, BrowserOptions};
use auditmysite::cli::{Args, OutputFormat};
use auditmysite::error::{AuditError, Result};
use auditmysite::output::{
    format_batch_html, format_html, generate_batch_pdf, generate_pdf, print_report, JsonReport,
};

#[tokio::main]
async fn main() {
    // Parse CLI arguments
    let args = Args::parse();

    // Setup logging
    setup_logging(&args);

    // Run the main logic
    if let Err(e) = run(args).await {
        error!("{}", e);
        eprintln!("{} {}", "Error:".red().bold(), e);
        std::process::exit(1);
    }
}

/// Setup tracing/logging based on CLI flags
fn setup_logging(args: &Args) {
    let level = if args.quiet {
        Level::ERROR
    } else if args.verbose {
        Level::DEBUG
    } else {
        Level::INFO
    };

    let subscriber = FmtSubscriber::builder()
        .with_max_level(level)
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .compact()
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");
}

/// Main application logic
async fn run(args: Args) -> Result<()> {
    // Handle --detect-chrome flag
    if args.detect_chrome {
        return detect_chrome_command(&args);
    }

    // Validate arguments
    if let Err(e) = args.validate() {
        return Err(auditmysite::error::AuditError::ConfigError(e));
    }

    // Print banner
    if !args.quiet {
        print_banner();
    }

    // Determine if this is a batch operation
    let is_batch = args.sitemap.is_some() || args.url_file.is_some();

    if is_batch {
        run_batch_mode(&args).await
    } else {
        run_single_mode(&args).await
    }
}

/// Run single URL audit mode
async fn run_single_mode(args: &Args) -> Result<()> {
    let url = args.url.as_ref().expect("URL required after validation");

    info!("Starting audit for: {}", url);

    // Build browser options from CLI args
    let browser_options = BrowserOptions {
        chrome_path: args.chrome_path.clone(),
        headless: true,
        disable_gpu: true,
        no_sandbox: args.no_sandbox,
        disable_images: args.disable_images,
        window_size: (1920, 1080),
        timeout_secs: args.timeout,
        verbose: args.verbose,
    };

    // Launch browser
    if !args.quiet {
        println!("{}", "Launching browser...".dimmed());
    }
    let browser = BrowserManager::with_options(browser_options).await?;

    if !args.quiet {
        println!(
            "{} Chrome {} at {}",
            "Found:".green().bold(),
            browser.chrome_version().unwrap_or("unknown version"),
            browser.chrome_path().display()
        );
    }

    // Build pipeline config
    let config = PipelineConfig::from(args);

    // Run the audit
    if !args.quiet {
        println!("{} {}", "Auditing:".cyan().bold(), url);
    }

    let report = run_single_audit(url, &browser, &config).await?;

    // Close browser
    browser.close().await?;
    info!("Browser closed");

    // Output results
    output_single_report(&report, args)?;

    // Exit with non-zero code if critical violations found
    if report
        .wcag_results
        .violations
        .iter()
        .any(|v| v.severity == auditmysite::Severity::Critical)
    {
        std::process::exit(1);
    }

    Ok(())
}

/// Run batch audit mode (sitemap or URL file)
async fn run_batch_mode(args: &Args) -> Result<()> {
    // Collect URLs from source
    let urls = if let Some(ref sitemap_url) = args.sitemap {
        if !args.quiet {
            println!("{} {}", "Fetching sitemap:".cyan().bold(), sitemap_url);
        }
        parse_sitemap(sitemap_url).await?
    } else if let Some(ref url_file) = args.url_file {
        if !args.quiet {
            println!(
                "{} {}",
                "Reading URL file:".cyan().bold(),
                url_file.display()
            );
        }
        read_url_file(url_file.to_str().unwrap_or(""))?
    } else {
        return Err(auditmysite::error::AuditError::ConfigError(
            "No batch source specified".to_string(),
        ));
    };

    if urls.is_empty() {
        if !args.quiet {
            println!("{} No URLs found to audit.", "Warning:".yellow().bold());
        }
        return Ok(());
    }

    let total_urls = if args.max_pages > 0 {
        args.max_pages.min(urls.len())
    } else {
        urls.len()
    };

    if !args.quiet {
        println!(
            "{} {} URLs with {} concurrent workers",
            "Auditing:".cyan().bold(),
            total_urls,
            args.concurrency
        );
        println!();
    }

    // Build batch config
    let batch_config = BatchConfig::from(args);

    // Progress callback with progress bar
    let quiet = args.quiet;
    let progress_bar = if !quiet {
        let pb = ProgressBar::new(urls.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")
                .expect("Invalid template")
                .progress_chars("#>-")
        );
        Some(pb)
    } else {
        None
    };

    let progress: Option<Arc<dyn Fn(usize, usize, &str) + Send + Sync>> =
        if let Some(ref pb) = progress_bar {
            let pb_clone = pb.clone();
            Some(Arc::new(move |current, _total, url| {
                pb_clone.set_position(current as u64);
                pb_clone.set_message(truncate_url(url, 50));
            }))
        } else {
            None
        };

    // Run batch audit with concurrent processing
    let batch_report = run_concurrent_batch(urls, &batch_config, progress).await?;

    // Finish progress bar
    if let Some(pb) = progress_bar {
        pb.finish_with_message("Complete");
    }

    if !args.quiet {
        println!();
        println!(
            "{} {}/{} passed, {} violations found in {}ms",
            "Results:".green().bold(),
            batch_report.summary.passed,
            batch_report.summary.total_urls,
            batch_report.summary.total_violations,
            batch_report.total_duration_ms
        );
    }

    // Output batch results
    output_batch_report(&batch_report, args)?;

    // Exit with non-zero code if any failures
    if batch_report.summary.failed > 0 {
        std::process::exit(1);
    }

    Ok(())
}

/// Output a single audit report in the requested format
fn output_single_report(report: &auditmysite::AuditReport, args: &Args) -> Result<()> {
    match args.format {
        OutputFormat::Json => {
            let json_report =
                JsonReport::new(report.clone(), &args.level.to_string(), report.duration_ms);
            let output = json_report.to_json(true)?;

            if let Some(path) = &args.output {
                write_output(&output, path)?;
                if !args.quiet {
                    println!(
                        "{} JSON report saved to {}",
                        "Success:".green().bold(),
                        path.display()
                    );
                }
            } else {
                println!("{}", output);
            }
        }
        OutputFormat::Table => {
            print_report(report, args.level);
        }
        OutputFormat::Html => {
            let output = format_html(report, &args.level.to_string())?;

            if let Some(path) = &args.output {
                write_output(&output, path)?;
                if !args.quiet {
                    println!(
                        "{} HTML report saved to {}",
                        "Success:".green().bold(),
                        path.display()
                    );
                }
            } else {
                // For HTML without output file, save to default
                let default_path = PathBuf::from("audit-report.html");
                write_output(&output, &default_path)?;
                if !args.quiet {
                    println!(
                        "{} HTML report saved to {}",
                        "Success:".green().bold(),
                        default_path.display()
                    );
                }
            }
        }
        OutputFormat::Pdf => {
            let pdf_bytes =
                generate_pdf(report).map_err(|e| AuditError::ReportGenerationFailed {
                    reason: e.to_string(),
                })?;

            let output_path = if let Some(path) = &args.output {
                path.clone()
            } else {
                PathBuf::from("reports/audit-report.pdf")
            };

            if let Some(parent) = output_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            std::fs::write(&output_path, pdf_bytes)?;

            if !args.quiet {
                println!(
                    "{} PDF report saved to {}",
                    "Success:".green().bold(),
                    output_path.display()
                );
            }
        }

        OutputFormat::Markdown => {
            let output = format_markdown_report(report);
            if let Some(path) = &args.output {
                write_output(&output, path)?;
                if !args.quiet {
                    println!(
                        "{} Markdown report saved to {}",
                        "Success:".green().bold(),
                        path.display()
                    );
                }
            } else {
                println!("{}", output);
            }
        }
    }

    Ok(())
}

/// Output batch audit results in the requested format
fn output_batch_report(batch_report: &auditmysite::audit::BatchReport, args: &Args) -> Result<()> {
    match args.format {
        OutputFormat::Json => {
            let output = serde_json::to_string_pretty(batch_report).map_err(|e| {
                auditmysite::error::AuditError::OutputError {
                    reason: e.to_string(),
                }
            })?;

            if let Some(path) = &args.output {
                write_output(&output, path)?;
                if !args.quiet {
                    println!(
                        "{} JSON batch report saved to {}",
                        "Success:".green().bold(),
                        path.display()
                    );
                }
            } else {
                println!("{}", output);
            }
        }
        OutputFormat::Table => {
            print_batch_table(batch_report, args);
        }
        OutputFormat::Html => {
            let output = format_batch_html(&batch_report.reports, &args.level.to_string())?;

            if let Some(path) = &args.output {
                write_output(&output, path)?;
                if !args.quiet {
                    println!(
                        "{} HTML batch report saved to {}",
                        "Success:".green().bold(),
                        path.display()
                    );
                }
            } else {
                let default_path = PathBuf::from("batch-audit-report.html");
                write_output(&output, &default_path)?;
                if !args.quiet {
                    println!(
                        "{} HTML batch report saved to {}",
                        "Success:".green().bold(),
                        default_path.display()
                    );
                }
            }
        }
        OutputFormat::Pdf => {
            let pdf_bytes = generate_batch_pdf(&batch_report).map_err(|e| {
                AuditError::ReportGenerationFailed {
                    reason: e.to_string(),
                }
            })?;

            let output_path = if let Some(path) = &args.output {
                path.clone()
            } else {
                PathBuf::from("reports/batch-audit-report.pdf")
            };

            if let Some(parent) = output_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            std::fs::write(&output_path, pdf_bytes)?;

            if !args.quiet {
                println!(
                    "{} PDF batch report saved to {}",
                    "Success:".green().bold(),
                    output_path.display()
                );
            }
        }

        OutputFormat::Markdown => {
            let output = format_batch_markdown(batch_report);
            if let Some(path) = &args.output {
                write_output(&output, path)?;
                if !args.quiet {
                    println!(
                        "{} Markdown batch report saved to {}",
                        "Success:".green().bold(),
                        path.display()
                    );
                }
            } else {
                println!("{}", output);
            }
        }
    }

    Ok(())
}

/// Print batch results as a table
fn print_batch_table(batch_report: &auditmysite::audit::BatchReport, args: &Args) {
    println!();
    println!("{} WCAG {} Batch Audit Results", "═══".cyan(), args.level);
    println!();

    // Summary
    println!(
        "  {} {} URLs audited",
        "Total:".bold(),
        batch_report.summary.total_urls
    );
    println!(
        "  {} {} passed, {} failed",
        "Status:".bold(),
        batch_report.summary.passed.to_string().green(),
        batch_report.summary.failed.to_string().red()
    );
    println!(
        "  {} {:.1}",
        "Avg Score:".bold(),
        batch_report.summary.average_score
    );
    println!(
        "  {} {}",
        "Total Violations:".bold(),
        batch_report.summary.total_violations
    );
    println!(
        "  {} {}ms",
        "Duration:".bold(),
        batch_report.total_duration_ms
    );
    println!();

    // Individual results
    println!("{}", "─".repeat(80));
    println!(
        "{:<50} {:>8} {:>10} {:>8}",
        "URL".bold(),
        "Score".bold(),
        "Violations".bold(),
        "Status".bold()
    );
    println!("{}", "─".repeat(80));

    for report in &batch_report.reports {
        let status = if report.passed() {
            "PASS".green()
        } else {
            "FAIL".red()
        };

        let score_color = if report.score >= 90.0 {
            format!("{:.1}", report.score).green()
        } else if report.score >= 70.0 {
            format!("{:.1}", report.score).yellow()
        } else {
            format!("{:.1}", report.score).red()
        };

        println!(
            "{:<50} {:>8} {:>10} {:>8}",
            truncate_url(&report.url, 48),
            score_color,
            report.violation_count(),
            status
        );
    }

    println!("{}", "─".repeat(80));
}

/// Format batch results as markdown
fn format_batch_markdown(batch_report: &auditmysite::audit::BatchReport) -> String {
    let mut output = String::new();

    output.push_str("# WCAG Batch Audit Report\n\n");
    output.push_str("## Summary\n\n");
    output.push_str(&format!(
        "- **Total URLs:** {}\n",
        batch_report.summary.total_urls
    ));
    output.push_str(&format!("- **Passed:** {}\n", batch_report.summary.passed));
    output.push_str(&format!("- **Failed:** {}\n", batch_report.summary.failed));
    output.push_str(&format!(
        "- **Average Score:** {:.1}\n",
        batch_report.summary.average_score
    ));
    output.push_str(&format!(
        "- **Total Violations:** {}\n",
        batch_report.summary.total_violations
    ));
    output.push_str(&format!(
        "- **Duration:** {}ms\n\n",
        batch_report.total_duration_ms
    ));

    output.push_str("## Results by URL\n\n");
    output.push_str("| URL | Score | Violations | Status |\n");
    output.push_str("|-----|-------|------------|--------|\n");

    for report in &batch_report.reports {
        let status = if report.passed() { "Pass" } else { "Fail" };
        output.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            report.url,
            report.score,
            report.violation_count(),
            status
        ));
    }

    output.push_str("\n---\n\n");
    output.push_str(&format!(
        "*Generated by auditmysite v{}*\n",
        env!("CARGO_PKG_VERSION")
    ));

    output
}

/// Handle --detect-chrome command
fn detect_chrome_command(args: &Args) -> Result<()> {
    println!("{}", "Detecting Chrome/Chromium...".cyan().bold());
    println!();

    match find_chrome(args.chrome_path.as_deref()) {
        Ok(info) => {
            println!("{} Chrome found!", "Success:".green().bold());
            println!();
            println!("  Path:    {}", info.path.display());
            println!(
                "  Version: {}",
                info.version.as_deref().unwrap_or("unknown")
            );
            println!("  Method:  {:?}", info.detection_method);
            Ok(())
        }
        Err(e) => {
            println!("{}", e);
            Err(e)
        }
    }
}

/// Write output to file
fn write_output(content: &str, path: &PathBuf) -> Result<()> {
    fs::write(path, content).map_err(|e| auditmysite::error::AuditError::FileError {
        path: path.clone(),
        reason: e.to_string(),
    })
}

/// Format a simple markdown report for single URL
fn format_markdown_report(report: &auditmysite::AuditReport) -> String {
    let mut output = String::new();

    output.push_str("# WCAG Accessibility Report\n\n");
    output.push_str(&format!("**URL:** {}\n\n", report.url));
    output.push_str(&format!(
        "**Date:** {}\n\n",
        report.timestamp.format("%Y-%m-%d %H:%M:%S UTC")
    ));
    output.push_str(&format!("**Score:** {}/100\n\n", report.score));
    output.push_str(&format!(
        "**Violations:** {}\n\n",
        report.wcag_results.violations.len()
    ));

    if !report.wcag_results.violations.is_empty() {
        output.push_str("## Violations\n\n");

        for violation in &report.wcag_results.violations {
            output.push_str(&format!(
                "### {} - {}\n\n",
                violation.rule, violation.rule_name
            ));
            output.push_str(&format!("- **Level:** {}\n", violation.level));
            output.push_str(&format!("- **Severity:** {}\n", violation.severity));
            output.push_str(&format!("- **Message:** {}\n", violation.message));

            if let Some(fix) = &violation.fix_suggestion {
                output.push_str(&format!("- **Suggested Fix:** {}\n", fix));
            }

            if let Some(url) = &violation.help_url {
                output.push_str(&format!(
                    "- **Learn More:** [WCAG Documentation]({})\n",
                    url
                ));
            }

            output.push('\n');
        }
    }

    output.push_str("---\n\n");
    output.push_str(&format!(
        "*Generated by auditmysite v{} in {}ms*\n",
        env!("CARGO_PKG_VERSION"),
        report.duration_ms
    ));

    output
}

/// Truncate URL for display
fn truncate_url(url: &str, max_len: usize) -> String {
    if url.len() <= max_len {
        url.to_string()
    } else {
        format!("{}...", &url[..max_len - 3])
    }
}

/// Print application banner
fn print_banner() {
    println!();
    println!(
        "{}",
        r#"
                 _ _ _
   __ _ _   _  __| (_) |_
  / _` | | | |/ _` | | __|
 | (_| | |_| | (_| | | |_
  \__,_|\__,_|\__,_|_|\__|
"#
        .cyan()
    );
    println!(
        "  {} v{} - WCAG 2.1 Accessibility Checker",
        "auditmysite".bold(),
        env!("CARGO_PKG_VERSION")
    );
    println!();
}
