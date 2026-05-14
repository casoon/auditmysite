//! Output path helpers and file writing utilities.
//!
//! Pure functions for determining output file paths and writing report
//! content to disk or stdout. Extracted from main.rs to keep that file
//! focused on bootstrap and dispatch.

use std::fs;
use std::path::{Path, PathBuf};

use chrono::Local;
use colored::Colorize;

use auditmysite::cli::{Args, OutputFormat, ReportLevel};
use auditmysite::error::{AuditError, Result};

/// Write text content to `path` (or stdout when `path` is None).
pub fn output_text(content: &str, path: &Option<PathBuf>, label: &str, quiet: bool) -> Result<()> {
    if let Some(path) = path {
        fs::create_dir_all(output_directory(path))?;
        fs::write(path, content).map_err(|e| AuditError::FileError {
            path: path.clone(),
            reason: e.to_string(),
        })?;
        if !quiet {
            println!(
                "{} {}-report saved to {}",
                "Done:".green().bold(),
                label,
                path.display()
            );
        }
    } else {
        println!("{}", content);
    }
    Ok(())
}

/// Write binary content to `path`.
#[cfg(feature = "pdf")]
pub fn output_bytes(content: &[u8], path: &PathBuf, label: &str, quiet: bool) -> Result<()> {
    fs::create_dir_all(output_directory(path))?;
    fs::write(path, content)?;
    if !quiet {
        println!(
            "{} {} report saved to {}",
            "Done:".green().bold(),
            label,
            path.display()
        );
    }
    Ok(())
}

/// Return the parent directory of `path`, defaulting to `"."`.
pub fn output_directory(path: &Path) -> &Path {
    match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent,
        _ => Path::new("."),
    }
}

/// Derive the companion `.json` path from a PDF output path.
pub fn default_single_json_output_path(pdf_path: &Path) -> PathBuf {
    pdf_path.with_extension("json")
}

/// Directory used for per-page batch output files.
pub fn per_page_output_directory(args: &Args) -> PathBuf {
    match args.output.as_ref() {
        Some(path) if path.extension().is_none() => path.clone(),
        Some(path) => output_directory(path).to_path_buf(),
        None => PathBuf::from("."),
    }
}

/// Concrete output path for one page inside a per-page batch run.
pub fn per_page_output_path(
    base_dir: &Path,
    url: &str,
    format: OutputFormat,
    report_level: ReportLevel,
) -> PathBuf {
    let date = Local::now().format("%Y-%m-%d");
    let subject = report_subject_from_url(url);
    let filename = match format {
        OutputFormat::Pdf => default_single_pdf_output_path(url, report_level),
        OutputFormat::Json => PathBuf::from(format!("{subject}-{date}-single-report.json")),
        OutputFormat::Table => PathBuf::from(format!("{subject}-{date}-single-report.txt")),
        OutputFormat::Ai => PathBuf::from(format!("{subject}-{date}-single-report-ai.json")),
        OutputFormat::Summary => PathBuf::from(format!("{subject}-{date}-summary.json")),
    };
    match filename.file_name() {
        Some(name) => base_dir.join(name),
        None => base_dir.join(filename),
    }
}

/// Default PDF output path for a single-URL audit.
pub fn default_single_pdf_output_path(url: &str, _report_level: ReportLevel) -> PathBuf {
    let date = Local::now().format("%Y-%m-%d");
    let subject = report_subject_from_url(url);
    PathBuf::from(format!("{subject}-{date}-single-report.pdf"))
}

/// Default PDF output path for a batch audit.
pub fn default_batch_pdf_output_path(args: &Args) -> PathBuf {
    let date = Local::now().format("%Y-%m-%d");
    let kind = if args.sitemap.is_some() {
        "sitemap"
    } else if args.crawl {
        "crawl"
    } else {
        "batch"
    };
    let source_url = args
        .sitemap
        .as_deref()
        .or(args.url.as_deref())
        .unwrap_or("");
    let subject = report_subject_from_url(source_url);
    PathBuf::from(format!("{subject}-{date}-{kind}-report.pdf"))
}

/// Derive a filename-safe domain slug from a URL (e.g. `"casoon.de"`).
pub fn report_subject_from_url(url: &str) -> String {
    let fallback = "audit-report".to_string();
    let Ok(parsed) = url::Url::parse(url) else {
        return fallback;
    };
    let Some(host) = parsed.host_str() else {
        return fallback;
    };
    let host = host.strip_prefix("www.").unwrap_or(host);
    let slug: String = host
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    let slug = slug.trim_matches('-').to_string();
    if slug.is_empty() {
        fallback
    } else {
        slug
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_subject_strips_www() {
        assert_eq!(
            report_subject_from_url("https://www.casoon.de/page"),
            "casoon-de"
        );
    }

    #[test]
    fn report_subject_replaces_dots_with_dashes() {
        assert_eq!(
            report_subject_from_url("https://sub.example.com/"),
            "sub-example-com"
        );
    }

    #[test]
    fn report_subject_fallback_on_invalid_url() {
        assert_eq!(report_subject_from_url("not-a-url"), "audit-report");
    }

    #[test]
    fn output_directory_returns_parent() {
        let path = Path::new("reports/foo.pdf");
        assert_eq!(output_directory(path), Path::new("reports"));
    }

    #[test]
    fn output_directory_falls_back_to_dot() {
        let path = Path::new("foo.pdf");
        assert_eq!(output_directory(path), Path::new("."));
    }

    #[test]
    fn default_single_json_output_path_replaces_extension() {
        let pdf = Path::new("reports/casoon-2026-01-01-single-report.pdf");
        assert_eq!(
            default_single_json_output_path(pdf),
            PathBuf::from("reports/casoon-2026-01-01-single-report.json")
        );
    }
}
