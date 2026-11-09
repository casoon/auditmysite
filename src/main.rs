//! AuditMySit CLI Entry Point
//!
//! Bootstrap and argument parsing. Orchestration logic lives in the sibling
//! modules declared below, shared with the `auditmysite-dev` binary.

#![warn(unreachable_pub)]

#[path = "cli/app.rs"]
mod app;
#[path = "cli/batch_lifecycle.rs"]
mod batch_lifecycle;
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

use clap::{CommandFactory, FromArgMatches};

use auditmysite::cli::Args;

#[tokio::main]
async fn main() {
    let matches = Args::command().get_matches();
    let interactive_from_cli =
        matches.value_source("interactive") == Some(clap::parser::ValueSource::CommandLine);
    let mut args = Args::from_arg_matches(&matches).unwrap_or_else(|e| e.exit());
    app::setup_logging(&args);

    // Load config file and apply defaults (CLI args take precedence)
    let config = auditmysite::cli::Config::load();
    if let Some(ref cfg) = config {
        cfg.apply_to_args_with_sources(&mut args, interactive_from_cli);
    }

    let report_mode = args.report_mode;
    app::run_and_exit(app::run(args, &config), report_mode).await
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "pdf")]
    use crate::output_paths::{default_batch_pdf_output_path, default_single_json_output_path};
    use crate::output_paths::{
        default_single_pdf_output_path, output_directory, per_page_output_directory,
        per_page_output_path, report_subject_from_url,
    };
    use crate::plan::{active_modules_label, planned_batch_outputs, planned_single_outputs};
    use crate::runners::suggested_sitemap_batch_args;
    use crate::sitemap_suggest::{looks_like_base_url, sitemap_candidates};
    use auditmysite::cli::{OutputFormat, ReportLevel};
    use clap::Parser;
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
        assert!(rendered.contains("in-punkto-com-leistungen-"));
    }

    #[test]
    fn test_per_page_output_paths_differ_per_page() {
        let path = |url: &str| {
            per_page_output_path(
                std::path::Path::new("reports"),
                url,
                OutputFormat::Json,
                ReportLevel::Standard,
            )
        };
        let pages = [
            "https://www.casoon.de/",
            "https://www.casoon.de/arbeitsweise/",
            "https://www.casoon.de/blog/a11y-check",
            "https://www.casoon.de/search?q=a",
            "https://www.casoon.de/search?q=b",
        ];
        let paths: std::collections::HashSet<_> = pages.iter().map(|u| path(u)).collect();
        assert_eq!(paths.len(), pages.len(), "{paths:?}");
        assert!(path("https://www.casoon.de/")
            .display()
            .to_string()
            .contains("reports/casoon-de-2"));
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
    fn test_suggested_sitemap_batch_args_keeps_the_default_format() {
        let args = Args::parse_from(["auditmysite", "https://example.com"]);
        let batch_args =
            suggested_sitemap_batch_args(&args, "https://example.com/sitemap.xml".into());

        assert!(batch_args.url.is_none());
        assert_eq!(
            batch_args.sitemap.as_deref(),
            Some("https://example.com/sitemap.xml")
        );
        assert_eq!(batch_args.format, None);
        assert_eq!(batch_args.effective_format(), OutputFormat::Table);
    }

    #[test]
    fn test_suggested_sitemap_batch_args_preserves_explicit_format() {
        let args = Args::parse_from(["auditmysite", "https://example.com", "-f", "json"]);
        let batch_args =
            suggested_sitemap_batch_args(&args, "https://example.com/sitemap.xml".into());

        assert_eq!(batch_args.format, Some(OutputFormat::Json));
    }

    #[test]
    fn test_planned_single_outputs_write_nothing_by_default() {
        let args = Args::parse_from(["auditmysite", "https://example.com"]);
        let outputs = planned_single_outputs(&args, "https://example.com");

        assert_eq!(outputs, vec!["stdout"]);
    }

    #[test]
    fn test_planned_single_outputs_include_auto_json_for_pdf_without_output_file() {
        let args = Args::parse_from(["auditmysite", "https://example.com", "-f", "pdf"]);
        let outputs = planned_single_outputs(&args, "https://example.com");

        assert_eq!(outputs.len(), 2);
        assert!(outputs[0].ends_with("-single-report.pdf"));
        assert!(outputs[1].ends_with("-single-report.json"));
    }

    #[test]
    fn test_planned_single_outputs_use_stdout_without_output_file() {
        let args = Args::parse_from(["auditmysite", "https://example.com", "-f", "json"]);
        let outputs = planned_single_outputs(&args, "https://example.com");

        assert_eq!(outputs, vec!["stdout"]);
    }

    #[test]
    fn test_planned_single_outputs_add_the_sidecar_only_on_request() {
        let args = Args::parse_from([
            "auditmysite",
            "https://example.com",
            "-o",
            "report.json",
            "--screen-reader-report",
        ]);
        let outputs = planned_single_outputs(&args, "https://example.com");

        assert_eq!(
            outputs,
            vec!["report.json", "report-screen-reader-audit.json"]
        );
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
    fn test_active_modules_label_respects_the_module_selection() {
        let args = Args::parse_from(["auditmysite", "https://example.com", "--modules", "a11y"]);
        let label = active_modules_label(&args);

        let expected =
            "Accessibility, Accessibility Journey, AI Visibility, Source Quality".to_string();
        assert_eq!(label, expected);
    }
}
