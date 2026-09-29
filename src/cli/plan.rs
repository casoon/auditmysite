//! Audit plan display and banner output.
//!
//! Pure display functions that print what a planned audit will do —
//! format, scope, modules, output paths. No async, no I/O side effects
//! beyond printing to stdout. Extracted from main.rs.

use colored::Colorize;

use auditmysite::audit::PipelineConfig;
use auditmysite::cli::{Args, OutputFormat};

use crate::output_paths::{
    default_batch_pdf_output_path, default_screen_reader_json_output_path,
    default_single_json_output_path, default_single_pdf_output_path, per_page_output_directory,
};

// ─── Banner ──────────────────────────────────────────────────────────────────

pub(crate) fn print_banner() {
    eprintln!(
        "{}",
        r#"
    _             _ _ _   __  __       ____  _ _
   / \  _   _  __| (_) |_|  \/  |_   _/ ___|(_) |_ ___
  / _ \| | | |/ _` | | __| |\/| | | | \___ \| | __/ _ \
 / ___ \ |_| | (_| | | |_| |  | | |_| |___) | | ||  __/
/_/   \_\__,_|\__,_|_|\__|_|  |_|\__, |____/|_|\__\___|
                                 |___/
"#
        .cyan()
    );
    eprintln!(
        "  {} v{} - WCAG 2.2 AA Accessibility Checker\n",
        "AuditMySite".bold(),
        env!("CARGO_PKG_VERSION")
    );
}

// ─── Plan display ─────────────────────────────────────────────────────────────

pub(crate) fn print_single_audit_plan(args: &Args, url: &str) {
    if args.quiet {
        return;
    }
    eprintln!("{}", "Audit plan".cyan().bold());
    eprintln!("  {} Single URL", "Mode:".dimmed());
    eprintln!("  {} {}", "Format:".dimmed(), args.effective_format());
    eprintln!("  {} {}", "Report level:".dimmed(), args.report_level);
    eprintln!("  {} {}", "Modules:".dimmed(), active_modules_label(args));
    let outputs = planned_single_outputs(args, url);
    if !outputs.is_empty() {
        eprintln!("  {} {}", "Output:".dimmed(), outputs.join(", "));
    }
    eprintln!();
}

pub(crate) fn print_batch_audit_plan(args: &Args, total_urls: usize) {
    if args.quiet {
        return;
    }
    eprintln!("{}", "Audit plan".cyan().bold());
    eprintln!(
        "  {} {}",
        "Mode:".dimmed(),
        if args.per_page_reports {
            "Individual reports from batch"
        } else if args.crawl {
            "Crawl"
        } else if args.sitemap.is_some() {
            "Sitemap"
        } else {
            "URL file"
        }
    );
    eprintln!("  {} {} URLs", "Scope:".dimmed(), total_urls);
    eprintln!("  {} {}", "Format:".dimmed(), args.effective_format());
    eprintln!("  {} {}", "Report level:".dimmed(), args.report_level);
    eprintln!("  {} {}", "Modules:".dimmed(), active_modules_label(args));
    let outputs = planned_batch_outputs(args);
    if !outputs.is_empty() {
        eprintln!("  {} {}", "Output:".dimmed(), outputs.join(", "));
    }
    eprintln!();
}

// ─── Planned output path lists ────────────────────────────────────────────────

pub(crate) fn planned_single_outputs(args: &Args, url: &str) -> Vec<String> {
    match args.effective_format() {
        OutputFormat::Pdf => {
            let path = args
                .output
                .clone()
                .unwrap_or_else(|| default_single_pdf_output_path(url, args.report_level));
            let mut outputs = vec![path.display().to_string()];
            if args.also_json || args.output.is_none() {
                outputs.push(default_single_json_output_path(&path).display().to_string());
            }
            if !args.no_screen_reader_report {
                outputs.push(
                    default_screen_reader_json_output_path(&path)
                        .display()
                        .to_string(),
                );
            }
            outputs
        }
        OutputFormat::Json
        | OutputFormat::Ai
        | OutputFormat::Table
        | OutputFormat::Summary
        | OutputFormat::Sarif => {
            let mut outputs = match args.output.as_ref() {
                Some(path) => vec![path.display().to_string()],
                None => vec!["stdout".to_string()],
            };
            let primary_path = args
                .output
                .clone()
                .unwrap_or_else(|| default_single_pdf_output_path(url, args.report_level));
            if !args.no_screen_reader_report {
                outputs.push(
                    default_screen_reader_json_output_path(&primary_path)
                        .display()
                        .to_string(),
                );
            }
            outputs
        }
    }
}

pub(crate) fn planned_batch_outputs(args: &Args) -> Vec<String> {
    if args.per_page_reports {
        let dir = per_page_output_directory(args);
        let mut outputs = vec![dir.join("*").display().to_string()];
        if args.effective_format() == OutputFormat::Json {
            outputs.push(dir.join("index.json").display().to_string());
            outputs.push(dir.join("findings.jsonl").display().to_string());
        }
        return outputs;
    }
    match args.effective_format() {
        OutputFormat::Pdf => {
            let path = args
                .output
                .clone()
                .unwrap_or_else(|| default_batch_pdf_output_path(args));
            vec![
                path.display().to_string(),
                path.with_extension("json").display().to_string(),
            ]
        }
        OutputFormat::Json
        | OutputFormat::Ai
        | OutputFormat::Table
        | OutputFormat::Summary
        | OutputFormat::Sarif => match args.output.as_ref() {
            Some(path) => vec![path.display().to_string()],
            None => vec!["stdout".to_string()],
        },
    }
}

// ─── Module label ─────────────────────────────────────────────────────────────

pub(crate) fn active_modules_label(args: &Args) -> String {
    PipelineConfig::from(args).active_module_labels().join(", ")
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::*;

    #[test]
    fn active_modules_label_skip_performance_disables_full_mode() {
        // --skip-performance makes full_audit_enabled() return false,
        // so no optional full modules are active unless explicitly requested.
        let args = Args::parse_from(["auditmysite", "https://example.com", "--skip-performance"]);
        let expected =
            "Accessibility, Accessibility Journey, Dark Mode, AI Visibility, Source Quality"
                .to_string();
        assert_eq!(active_modules_label(&args), expected);
    }

    #[test]
    fn active_modules_label_default_matches_standard_pipeline() {
        let args = Args::parse_from(["auditmysite", "https://example.com"]);
        let expected = "Accessibility, Accessibility Journey, Best Practices, Dark Mode, HTML Conformance, Journey, Mobile, Performance, Security, SEO, AI Visibility, Tech Stack, Commerce, UX, Source Quality, Content Visibility".to_string();
        assert_eq!(active_modules_label(&args), expected);
    }

    #[test]
    fn planned_batch_outputs_per_page_json_lists_the_aggregates() {
        let mut args = Args::parse_from([
            "auditmysite",
            "--url-file",
            "urls.txt",
            "--technician",
            "-o",
            "out/tech",
        ]);
        args.apply_technician_preset();
        assert_eq!(
            planned_batch_outputs(&args),
            vec![
                "out/tech/*".to_string(),
                "out/tech/index.json".to_string(),
                "out/tech/findings.jsonl".to_string(),
            ]
        );
    }

    #[test]
    fn planned_batch_outputs_trailing_slash_gives_no_double_slash() {
        let mut args = Args::parse_from([
            "auditmysite",
            "--url-file",
            "urls.txt",
            "--technician",
            "-o",
            "out/tech/",
        ]);
        args.apply_technician_preset();
        assert_eq!(planned_batch_outputs(&args)[0], "out/tech/*");
    }

    #[test]
    fn planned_single_outputs_skip_the_sidecar_when_suppressed() {
        let args = Args::parse_from([
            "auditmysite",
            "https://example.com",
            "-f",
            "json",
            "--no-screen-reader-report",
        ]);
        assert_eq!(
            planned_single_outputs(&args, "https://example.com"),
            vec!["stdout"]
        );
    }

    #[test]
    fn planned_single_outputs_json_returns_stdout_without_output_flag() {
        let args = Args::parse_from(["auditmysite", "https://example.com", "-f", "json"]);
        let outputs = planned_single_outputs(&args, "https://example.com");
        assert_eq!(outputs.len(), 2);
        assert_eq!(outputs[0], "stdout");
        assert!(outputs[1].ends_with("-single-report-screen-reader-audit.json"));
    }

    #[test]
    fn planned_single_outputs_json_returns_path_when_set() {
        let args = Args::parse_from([
            "auditmysite",
            "https://example.com",
            "-f",
            "json",
            "-o",
            "out.json",
        ]);
        let outputs = planned_single_outputs(&args, "https://example.com");
        assert_eq!(
            outputs,
            vec![
                "out.json".to_string(),
                "out-screen-reader-audit.json".to_string()
            ]
        );
    }

    #[test]
    fn planned_batch_outputs_json_returns_stdout_without_output_flag() {
        let args = Args::parse_from([
            "auditmysite",
            "-f",
            "json",
            "--sitemap",
            "https://example.com/sitemap.xml",
        ]);
        let outputs = planned_batch_outputs(&args);
        assert_eq!(outputs, vec!["stdout"]);
    }
}
