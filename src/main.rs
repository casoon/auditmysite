//! AuditMySit CLI Entry Point
//!
//! Bootstrap, logging setup, and top-level command dispatch.
//! Orchestration logic lives in the sibling modules declared below.

#[path = "cli/commands.rs"]
mod commands;
#[path = "cli/output_paths.rs"]
mod output_paths;
#[path = "cli/plan.rs"]
mod plan;
#[path = "cli/report_writers.rs"]
mod report_writers;
#[path = "cli/runners.rs"]
mod runners;
#[path = "cli/sitemap_suggest.rs"]
mod sitemap_suggest;

use commands::{detect_chrome_command, handle_command};
use plan::print_banner;
use runners::{run_batch_mode, run_compare_mode, run_single_mode};

use std::io::{self, IsTerminal};

use clap::{CommandFactory, FromArgMatches};
use colored::Colorize;
use dialoguer::{Input, Select};
use tracing::error;
use tracing_subscriber::EnvFilter;

use auditmysite::cli::Args;
use auditmysite::error::{AuditError, Result};

#[tokio::main]
async fn main() {
    let matches = Args::command().get_matches();
    let interactive_from_cli =
        matches.value_source("interactive") == Some(clap::parser::ValueSource::CommandLine);
    let mut args = Args::from_arg_matches(&matches).unwrap_or_else(|e| e.exit());
    setup_logging(&args);

    // Load config file and apply defaults (CLI args take precedence)
    let config = auditmysite::cli::Config::load();
    if let Some(ref cfg) = config {
        cfg.apply_to_args_with_sources(&mut args, interactive_from_cli);
    }

    let exit_code = match run(args, &config).await {
        Ok(score) => {
            // Check score threshold from config
            if let Some(min_score) = auditmysite::cli::config::get_min_score_threshold(&config) {
                if score < min_score {
                    eprintln!(
                        "{} Score {:.1} is below threshold {:.1}",
                        "FAIL:".red().bold(),
                        score,
                        min_score
                    );
                    1
                } else {
                    0
                }
            } else {
                0
            }
        }
        Err(e) => {
            error!("{}", e);
            eprintln!("{} {}", "Error:".red().bold(), e);
            2
        }
    };

    std::process::exit(exit_code);
}

/// Setup tracing/logging based on CLI flags
fn setup_logging(args: &Args) {
    let filter = if args.quiet {
        "error,chromiumoxide=off,tungstenite=off".to_string()
    } else if args.verbose {
        "debug,chromiumoxide=error,tungstenite=error,auditmysite=debug".to_string()
    } else {
        "warn,chromiumoxide=off,tungstenite=off,auditmysite=warn".to_string()
    };

    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(filter))
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .compact()
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");
}

/// Main application logic
/// Returns the audit score (or 0.0 for non-scoring operations).
async fn run(mut args: Args, _config: &Option<auditmysite::cli::Config>) -> Result<f64> {
    // Handle subcommands first
    if let Some(ref command) = args.command {
        return handle_command(command, &args).await;
    }

    if args.detect_chrome {
        return detect_chrome_command(&args);
    }

    // --compare mode: audits multiple domains for side-by-side comparison
    if !args.compare.is_empty() {
        if let Err(e) = args.validate() {
            return Err(AuditError::ConfigError(e));
        }
        if !args.quiet {
            print_banner();
        }
        return run_compare_mode(&args).await;
    }

    // If no input source specified interactively ask for a domain (terminal only)
    let no_input = args.url.is_none()
        && args.sitemap.is_none()
        && args.url_file.is_none()
        && !args.crawl
        && !args.detect_chrome;

    if no_input && !args.quiet && io::stdin().is_terminal() && io::stdout().is_terminal() {
        print_banner();
        let input: String = Input::new()
            .with_prompt("  Domain or URL (e.g., example.com)")
            .validate_with(|s: &String| {
                if s.trim().is_empty() {
                    Err("Please enter a URL.")
                } else {
                    Ok(())
                }
            })
            .interact_text()
            .map_err(|e| AuditError::ConfigError(e.to_string()))?;
        let url = if input.contains("://") {
            input
        } else {
            format!("https://{}", input)
        };
        args.url = Some(url);

        let mode_idx = Select::new()
            .with_prompt("  Request mode")
            .items(&[
                "Simulate browser  (default, avoids bot detection)",
                "Identify as bot   (--request-mode bot, transparent)",
            ])
            .default(0)
            .interact()
            .map_err(|e| AuditError::ConfigError(e.to_string()))?;
        if mode_idx == 1 {
            args.request_mode = auditmysite::cli::RequestMode::Bot;
        }
    }

    if let Err(e) = args.validate() {
        return Err(AuditError::ConfigError(e));
    }

    if !args.quiet {
        // Only print banner if we haven't already (interactive prompt already printed it)
        if !no_input {
            print_banner();
        }
    }

    let is_batch = args.sitemap.is_some() || args.url_file.is_some() || args.crawl;

    if is_batch {
        run_batch_mode(&args).await
    } else {
        run_single_mode(&args, _config).await
    }
}

#[cfg(test)]
#[allow(clippy::items_after_test_module)]
mod tests {
    use super::*;
    #[cfg(feature = "pdf")]
    use crate::output_paths::{default_batch_pdf_output_path, default_single_json_output_path};
    use crate::output_paths::{
        default_single_pdf_output_path, output_directory, per_page_output_directory,
        per_page_output_path, report_subject_from_url,
    };
    use crate::plan::{
        active_modules_label, planned_batch_outputs, planned_comparison_outputs,
        planned_single_outputs,
    };
    use crate::runners::suggested_sitemap_batch_args;
    use crate::sitemap_suggest::{looks_like_base_url, sitemap_candidates};
    use auditmysite::cli::{OutputFormat, ReportLevel};
    use std::path::PathBuf;

    #[test]
    fn test_report_subject_from_url_strips_www_and_normalizes() {
        assert_eq!(
            report_subject_from_url("https://www.in-punkto.com"),
            "in-punkto-com"
        );
    }

    #[test]
    fn test_default_single_pdf_output_path_uses_current_directory_filename() {
        let path =
            default_single_pdf_output_path("https://www.in-punkto.com", ReportLevel::Standard);
        let rendered = path.display().to_string();
        assert!(rendered.ends_with("-single-report.pdf"));
        assert!(rendered.contains("in-punkto-com-"));
        assert!(!rendered.contains('/'));
    }

    #[test]
    fn test_output_directory_defaults_to_current_directory_for_bare_filename() {
        let path = PathBuf::from("casoon-de-2026-03-31-single-report.pdf");
        assert_eq!(output_directory(&path), std::path::Path::new("."));
    }

    #[cfg(feature = "pdf")]
    #[test]
    fn test_default_single_json_output_path_matches_pdf_basename() {
        let pdf_path = PathBuf::from("casoon-de-2026-03-31-single-report.pdf");
        assert_eq!(
            default_single_json_output_path(&pdf_path),
            PathBuf::from("casoon-de-2026-03-31-single-report.json")
        );
    }

    #[test]
    fn test_per_page_output_directory_uses_output_directory_for_file_path() {
        let mut args = Args::parse_from(["auditmysite", "https://example.com"]);
        args.per_page_reports = true;
        args.output = Some(PathBuf::from("reports/custom-name.pdf"));
        assert_eq!(per_page_output_directory(&args), PathBuf::from("reports"));
    }

    #[test]
    fn test_per_page_output_directory_uses_path_directly_for_directory_like_output() {
        let mut args = Args::parse_from(["auditmysite", "https://example.com"]);
        args.per_page_reports = true;
        args.output = Some(PathBuf::from("reports/per-page"));
        assert_eq!(
            per_page_output_directory(&args),
            PathBuf::from("reports/per-page")
        );
    }

    #[test]
    fn test_per_page_output_path_uses_url_slug_and_extension() {
        let path = per_page_output_path(
            std::path::Path::new("reports"),
            "https://www.in-punkto.com/leistungen/",
            OutputFormat::Pdf,
            ReportLevel::Standard,
        );
        let rendered = path.display().to_string();
        assert!(rendered.starts_with("reports/"));
        assert!(rendered.ends_with("-single-report.pdf"));
        assert!(rendered.contains("in-punkto-com-"));
    }

    #[cfg(feature = "pdf")]
    #[test]
    fn test_default_batch_pdf_output_path_uses_sitemap_kind() {
        let args = Args::parse_from([
            "auditmysite",
            "--sitemap",
            "https://example.com/sitemap.xml",
        ]);
        let rendered = default_batch_pdf_output_path(&args).display().to_string();
        assert!(rendered.contains("example-com-"));
        assert!(rendered.contains("-sitemap-report.pdf"));
        assert!(rendered.ends_with(".pdf"));
        assert!(!rendered.contains("reports/"));
    }

    #[cfg(feature = "pdf")]
    #[test]
    fn test_default_batch_pdf_output_path_uses_crawl_kind() {
        let mut args = Args::parse_from(["auditmysite", "https://example.com"]);
        args.crawl = true;
        let rendered = default_batch_pdf_output_path(&args).display().to_string();
        assert!(rendered.contains("example-com-"));
        assert!(rendered.contains("-crawl-report.pdf"));
    }

    #[test]
    fn test_per_page_output_path_uses_single_report_name_for_json() {
        let path = per_page_output_path(
            std::path::Path::new("reports"),
            "https://www.in-punkto.com/leistungen/",
            OutputFormat::Json,
            ReportLevel::Standard,
        );
        let rendered = path.display().to_string();
        assert!(rendered.ends_with("-single-report.json"));
    }

    #[test]
    fn test_looks_like_base_url_accepts_root_url() {
        assert!(looks_like_base_url("https://www.casoon.de"));
        assert!(looks_like_base_url("https://www.casoon.de/"));
        assert!(!looks_like_base_url("https://www.casoon.de/blog"));
        assert!(!looks_like_base_url("https://www.casoon.de/?p=1"));
    }

    #[test]
    fn test_sitemap_candidates_include_usual_suspects() {
        let candidates = sitemap_candidates("https://www.casoon.de").unwrap();
        assert!(candidates.contains(&"https://www.casoon.de/sitemap.xml".to_string()));
        assert!(candidates.contains(&"https://www.casoon.de/sitemap_index.xml".to_string()));
        assert!(candidates.contains(&"https://www.casoon.de/page-sitemap.xml".to_string()));
    }

    #[test]
    fn test_suggested_sitemap_batch_args_defaults_to_pdf() {
        let args = Args::parse_from(["auditmysite", "https://example.com"]);
        let batch_args =
            suggested_sitemap_batch_args(&args, "https://example.com/sitemap.xml".into());

        assert!(batch_args.url.is_none());
        assert_eq!(
            batch_args.sitemap.as_deref(),
            Some("https://example.com/sitemap.xml")
        );
        assert_eq!(batch_args.format, Some(OutputFormat::Pdf));
    }

    #[test]
    fn test_suggested_sitemap_batch_args_preserves_explicit_format() {
        let args = Args::parse_from(["auditmysite", "https://example.com", "-f", "json"]);
        let batch_args =
            suggested_sitemap_batch_args(&args, "https://example.com/sitemap.xml".into());

        assert_eq!(batch_args.format, Some(OutputFormat::Json));
    }

    #[test]
    fn test_planned_single_outputs_include_auto_json_for_default_pdf() {
        let args = Args::parse_from(["auditmysite", "https://example.com"]);
        let outputs = planned_single_outputs(&args, "https://example.com");

        assert_eq!(outputs.len(), 2);
        assert!(outputs[0].ends_with("-single-report.pdf"));
        assert!(outputs[1].ends_with("-single-report.json"));
    }

    #[test]
    fn test_planned_single_outputs_use_stdout_without_output_file() {
        let args = Args::parse_from(["auditmysite", "https://example.com", "-f", "json"]);
        let outputs = planned_single_outputs(&args, "https://example.com");

        assert_eq!(outputs, vec!["stdout".to_string()]);
    }

    #[test]
    fn test_planned_batch_outputs_include_json_sidecar_for_pdf() {
        let args = Args::parse_from([
            "auditmysite",
            "--sitemap",
            "https://example.com/sitemap.xml",
            "-f",
            "pdf",
        ]);
        let outputs = planned_batch_outputs(&args);

        assert_eq!(outputs.len(), 2);
        assert!(outputs[0].ends_with("-sitemap-report.pdf"));
        assert!(outputs[1].ends_with("-sitemap-report.json"));
    }

    #[test]
    fn test_planned_comparison_outputs_default_to_pdf_file() {
        let args = Args::parse_from([
            "auditmysite",
            "--compare",
            "https://alpha.example.com",
            "https://beta.example.com",
        ]);
        let outputs = planned_comparison_outputs(&args);

        assert_eq!(outputs, vec!["comparison-report.pdf".to_string()]);
    }

    #[test]
    fn test_planned_comparison_outputs_use_stdout_for_json_without_output() {
        let args = Args::parse_from([
            "auditmysite",
            "--compare",
            "https://alpha.example.com",
            "https://beta.example.com",
            "-f",
            "json",
        ]);
        let outputs = planned_comparison_outputs(&args);

        assert_eq!(outputs, vec!["stdout".to_string()]);
    }

    #[test]
    fn test_active_modules_label_respects_skip_flags() {
        let args = Args::parse_from([
            "auditmysite",
            "https://example.com",
            "--skip-performance",
            "--skip-mobile",
        ]);
        let label = active_modules_label(&args);

        assert_eq!(label, "Accessibility");
    }
}
