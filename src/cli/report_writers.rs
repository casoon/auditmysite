//! Report output dispatchers.
//!
//! Converts AuditReport / BatchReport into the format the user requested and
//! writes to a file or stdout. Extracted from main.rs.

use colored::Colorize;

use auditmysite::audit::normalize;
use auditmysite::audit::VerdictResult;
use auditmysite::cli::{Args, OutputFormat};
use auditmysite::error::{AuditError, Result};
#[cfg(feature = "pdf")]
use auditmysite::output::report_model::ReportConfig;
use auditmysite::output::{
    export_snapshot_yaml, export_sr_audit, format_ai_json, format_batch_table, format_sarif,
    format_summary, print_batch_table, print_report, UnifiedReport,
};
#[cfg(feature = "pdf")]
use auditmysite::output::{generate_batch_pdf, generate_batch_typ, generate_pdf, generate_typ};

#[cfg(feature = "pdf")]
use crate::output_paths::output_bytes;
#[cfg(feature = "pdf")]
use crate::output_paths::{default_batch_pdf_output_path, default_single_json_output_path};
use crate::output_paths::{
    default_screen_reader_json_output_path, default_single_pdf_output_path, output_text,
    per_page_output_directory, per_page_output_path,
};

pub fn output_single_report(
    report: &auditmysite::AuditReport,
    args: &Args,
    verdict: Option<&VerdictResult>,
) -> Result<()> {
    match args.effective_format() {
        OutputFormat::Json => {
            let normalized = normalize(report);
            let mut unified = UnifiedReport::single(&normalized, report);
            if let Some(vr) = verdict {
                unified = unified.with_verdict(vr);
            }
            let output = unified.to_json(true)?;
            output_text(&output, &args.output, "JSON", args.quiet)?;
            if let Some(ref snap_path) = args.export_snapshot {
                export_snapshot_yaml(report, snap_path).map_err(|e| AuditError::OutputError {
                    reason: format!("snapshot export failed: {e}"),
                })?;
                if !args.quiet {
                    println!("Snapshot YAML written to {}", snap_path.display());
                }
            }
        }
        OutputFormat::Table => {
            print_report(report, args.level);
        }
        OutputFormat::Pdf => {
            #[cfg(feature = "pdf")]
            {
                if !args.quiet {
                    print_report(report, args.level);
                }
                let normalized = normalize(report);
                let path = args.output.clone().unwrap_or_else(|| {
                    default_single_pdf_output_path(report.url.as_str(), args.report_level)
                });
                let auto_json_path = if args.also_json || args.output.is_none() {
                    Some(default_single_json_output_path(&path))
                } else {
                    None
                };
                let config = ReportConfig {
                    level: args.report_level,
                    logo_path: args.logo.clone(),
                    locale: args.lang.clone(),
                    annex: args.annex,
                };
                let pdf_bytes = generate_pdf(report, &config).map_err(|e| {
                    AuditError::ReportGenerationFailed {
                        reason: e.to_string(),
                    }
                })?;
                if let Some(json_path) = auto_json_path.as_ref() {
                    let mut unified = UnifiedReport::single(&normalized, report);
                    if let Some(vr) = verdict {
                        unified = unified.with_verdict(vr);
                    }
                    let json_output = unified.to_json(true)?;
                    output_text(&json_output, &Some(json_path.clone()), "JSON", args.quiet)?;
                }
                output_bytes(&pdf_bytes, &path, "PDF", args.quiet)?;
                if args.debug_typ {
                    let typ = generate_typ(report, &config).map_err(|e| {
                        AuditError::ReportGenerationFailed {
                            reason: e.to_string(),
                        }
                    })?;
                    let typ_path = path.with_extension("typ");
                    output_text(&typ, &Some(typ_path), "Typst source", args.quiet)?;
                }
                if let Some(ref snap_path) = args.export_snapshot {
                    export_snapshot_yaml(report, snap_path).map_err(|e| {
                        AuditError::OutputError {
                            reason: format!("snapshot export failed: {e}"),
                        }
                    })?;
                    if !args.quiet {
                        println!("Snapshot YAML written to {}", snap_path.display());
                    }
                }
            }
            #[cfg(not(feature = "pdf"))]
            {
                return Err(AuditError::ConfigError(
                    "PDF output requires the 'pdf' feature. Rebuild with: cargo build --features pdf".to_string(),
                ));
            }
        }
        OutputFormat::Ai => {
            let output = format_ai_json(report);
            output_text(&output, &args.output, "AI JSON", args.quiet)?;
        }
        OutputFormat::Summary => {
            let normalized = normalize(report);
            let output =
                format_summary(&normalized.normalized).map_err(|e| AuditError::OutputError {
                    reason: e.to_string(),
                })?;
            output_text(&output, &args.output, "summary JSON", args.quiet)?;
        }
        OutputFormat::Sarif => {
            let normalized = normalize(report);
            let output =
                format_sarif(&[&normalized.normalized]).map_err(|e| AuditError::OutputError {
                    reason: e.to_string(),
                })?;
            output_text(&output, &args.output, "SARIF", args.quiet)?;
        }
    }
    output_screen_reader_sidecar(report, args)?;
    Ok(())
}

pub(crate) fn output_screen_reader_sidecar(
    report: &auditmysite::AuditReport,
    args: &Args,
) -> Result<()> {
    let Some(sr_audit) = report.screen_reader_audit.as_ref() else {
        return Ok(());
    };

    let primary_output_path = args
        .output
        .clone()
        .unwrap_or_else(|| default_single_pdf_output_path(report.url.as_str(), args.report_level));
    let path = default_screen_reader_json_output_path(&primary_output_path);
    export_sr_audit(sr_audit, &path)?;
    if !args.quiet {
        println!(
            "{} Screen-reader JSON report saved to {}",
            "Done:".green().bold(),
            path.display()
        );
    }
    Ok(())
}

pub fn output_batch_report(
    batch_report: &auditmysite::audit::BatchReport,
    args: &Args,
    verdict: Option<&VerdictResult>,
) -> Result<()> {
    match args.effective_format() {
        OutputFormat::Json => {
            let mut unified = auditmysite::output::UnifiedReport::batch(batch_report);
            if let Some(vr) = verdict {
                unified = unified.with_verdict(vr);
            }
            let output = unified.to_json(true)?;
            output_text(&output, &args.output, "JSON batch", args.quiet)?;
        }
        OutputFormat::Table => {
            if let Some(path) = &args.output {
                let output = format_batch_table(batch_report, args.level, false);
                output_text(&output, &Some(path.clone()), "Table batch", args.quiet)?;
            } else {
                print_batch_table(batch_report, args.level);
            }
        }
        OutputFormat::Pdf => {
            #[cfg(feature = "pdf")]
            {
                let config = ReportConfig {
                    level: args.report_level,
                    logo_path: args.logo.clone(),
                    locale: args.lang.clone(),
                    annex: args.annex,
                };
                let pdf_bytes = generate_batch_pdf(batch_report, &config).map_err(|e| {
                    AuditError::ReportGenerationFailed {
                        reason: e.to_string(),
                    }
                })?;
                let path = args
                    .output
                    .clone()
                    .unwrap_or_else(|| default_batch_pdf_output_path(args));
                output_bytes(&pdf_bytes, &path, "PDF batch", args.quiet)?;

                // Auto-generate JSON alongside batch PDF
                let json_path = path.with_extension("json");
                let mut unified = UnifiedReport::batch(batch_report);
                if let Some(vr) = verdict {
                    unified = unified.with_verdict(vr);
                }
                let json_output = unified.to_json(true)?;
                output_text(&json_output, &Some(json_path), "JSON batch", args.quiet)?;

                if args.debug_typ {
                    let typ = generate_batch_typ(batch_report, &config).map_err(|e| {
                        AuditError::ReportGenerationFailed {
                            reason: e.to_string(),
                        }
                    })?;
                    let typ_path = path.with_extension("typ");
                    output_text(&typ, &Some(typ_path), "Typst source batch", args.quiet)?;
                }
            }
            #[cfg(not(feature = "pdf"))]
            {
                return Err(AuditError::ConfigError(
                    "PDF output requires the 'pdf' feature. Rebuild with: cargo build --features pdf".to_string(),
                ));
            }
        }
        OutputFormat::Ai => {
            // For batch mode, emit one AI JSON document per URL.
            let outputs: Vec<String> = batch_report.reports.iter().map(format_ai_json).collect();
            let combined = format!("[\n{}\n]", outputs.join(",\n"));
            output_text(&combined, &args.output, "AI JSON batch", args.quiet)?;
        }
        OutputFormat::Summary => {
            let summaries: Vec<String> = batch_report
                .reports
                .iter()
                .filter_map(|r| {
                    let normalized = normalize(r);
                    format_summary(&normalized.normalized).ok()
                })
                .collect();
            let combined = format!("[\n{}\n]", summaries.join(",\n"));
            output_text(&combined, &args.output, "summary JSON batch", args.quiet)?;
        }
        OutputFormat::Sarif => {
            let normalized_reports: Vec<_> = batch_report.reports.iter().map(normalize).collect();
            let refs: Vec<_> = normalized_reports.iter().map(|n| &n.normalized).collect();
            let output = format_sarif(&refs).map_err(|e| AuditError::OutputError {
                reason: e.to_string(),
            })?;
            output_text(&output, &args.output, "SARIF batch", args.quiet)?;
        }
    }
    Ok(())
}

pub fn output_batch_as_single_reports(
    batch_report: &auditmysite::audit::BatchReport,
    args: &Args,
) -> Result<()> {
    let base_dir = per_page_output_directory(args);

    if !args.quiet {
        println!(
            "{} {} individual reports to {}",
            "Info:".cyan().bold(),
            batch_report.reports.len(),
            base_dir.display()
        );
    }

    for report in &batch_report.reports {
        let mut single_args = args.clone();
        single_args.url = Some(report.url.clone());
        single_args.sitemap = None;
        single_args.url_file = None;
        single_args.output = Some(per_page_output_path(
            &base_dir,
            &report.url,
            single_args.effective_format(),
            single_args.report_level,
        ));
        output_single_report(report, &single_args, None)?;
    }

    Ok(())
}
