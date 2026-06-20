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

pub fn print_banner() {
    println!(
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
    println!(
        "  {} v{} - WCAG 2.1 Accessibility Checker\n",
        "AuditMySite".bold(),
        env!("CARGO_PKG_VERSION")
    );
}

// ─── Plan display ─────────────────────────────────────────────────────────────

pub fn print_single_audit_plan(args: &Args, url: &str) {
    if args.quiet {
        return;
    }
    println!("{}", "Audit plan".cyan().bold());
    println!("  {} Single URL", "Mode:".dimmed());
    println!("  {} {}", "Format:".dimmed(), args.effective_format());
    println!("  {} {}", "Report level:".dimmed(), args.report_level);
    println!("  {} {}", "Modules:".dimmed(), active_modules_label(args));
    let outputs = planned_single_outputs(args, url);
    if !outputs.is_empty() {
        println!("  {} {}", "Output:".dimmed(), outputs.join(", "));
    }
    println!();
}

pub fn print_batch_audit_plan(args: &Args, total_urls: usize) {
    if args.quiet {
        return;
    }
    println!("{}", "Audit plan".cyan().bold());
    println!(
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
    println!("  {} {} URLs", "Scope:".dimmed(), total_urls);
    println!("  {} {}", "Format:".dimmed(), args.effective_format());
    println!("  {} {}", "Report level:".dimmed(), args.report_level);
    println!("  {} {}", "Modules:".dimmed(), active_modules_label(args));
    let outputs = planned_batch_outputs(args);
    if !outputs.is_empty() {
        println!("  {} {}", "Output:".dimmed(), outputs.join(", "));
    }
    println!();
}

// ─── Planned output path lists ────────────────────────────────────────────────

pub fn planned_single_outputs(args: &Args, url: &str) -> Vec<String> {
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
            outputs.push(
                default_screen_reader_json_output_path(&path)
                    .display()
                    .to_string(),
            );
            outputs
        }
        OutputFormat::Json | OutputFormat::Ai | OutputFormat::Table | OutputFormat::Summary => {
            let mut outputs = match args.output.as_ref() {
                Some(path) => vec![path.display().to_string()],
                None => vec!["stdout".to_string()],
            };
            let primary_path = args
                .output
                .clone()
                .unwrap_or_else(|| default_single_pdf_output_path(url, args.report_level));
            outputs.push(
                default_screen_reader_json_output_path(&primary_path)
                    .display()
                    .to_string(),
            );
            outputs
        }
    }
}

pub fn planned_batch_outputs(args: &Args) -> Vec<String> {
    if args.per_page_reports {
        return vec![format!("{}/*", per_page_output_directory(args).display())];
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
        OutputFormat::Json | OutputFormat::Ai | OutputFormat::Table | OutputFormat::Summary => {
            match args.output.as_ref() {
                Some(path) => vec![path.display().to_string()],
                None => vec!["stdout".to_string()],
            }
        }
    }
}

// ─── Module label ─────────────────────────────────────────────────────────────

pub fn active_modules_label(args: &Args) -> String {
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
        let expected = "Accessibility, Accessibility Journey, Best Practices, Dark Mode, Journey, Mobile, Performance, Security, SEO, AI Visibility, Commerce, Tech Stack, UX, Source Quality, Content Visibility".to_string();
        assert_eq!(active_modules_label(&args), expected);
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
