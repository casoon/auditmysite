//! Report output dispatchers.
//!
//! Converts AuditReport / BatchReport into the format the user requested and
//! writes to a file or stdout. Extracted from main.rs.

use std::collections::HashMap;

use colored::Colorize;

use auditmysite::audit::normalize;
use auditmysite::audit::VerdictResult;
use auditmysite::cli::{Args, OutputFormat};
use auditmysite::error::{AuditError, Result};
#[cfg(feature = "pdf")]
use auditmysite::output::report_model::ReportConfig;
use auditmysite::output::technician;
use auditmysite::output::{
    export_snapshot_yaml, export_sr_audit, format_ai_json, format_sarif, format_summary,
    UnifiedReport,
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

pub(crate) fn output_single_report(
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
            let output = auditmysite::output::terminal::render_single_report(report, args);
            output_text(&output, &args.output, "Table", args.quiet)?;
        }
        OutputFormat::Pdf => {
            #[cfg(feature = "pdf")]
            {
                if !args.quiet {
                    println!(
                        "{}",
                        auditmysite::output::terminal::render_single_report_for(
                            report, args, false
                        )
                    );
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
    if args.no_screen_reader_report {
        return Ok(());
    }
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

pub(crate) fn output_batch_report(
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
            let output = auditmysite::output::terminal::render_batch_report(batch_report, args);
            output_text(&output, &args.output, "Table batch", args.quiet)?;
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

pub(crate) fn output_batch_as_single_reports(
    batch_report: &auditmysite::audit::BatchReport,
    args: &Args,
    attempted_urls: &[String],
) -> Result<()> {
    let base_dir = per_page_output_directory(args);
    // JSON per-page reports get the technician aggregates next to them:
    // index.json and findings.jsonl (plan 67).
    let with_aggregates = args.effective_format() == OutputFormat::Json;

    if !args.quiet {
        println!(
            "{} {} individual reports to {}",
            "Info:".cyan().bold(),
            batch_report.reports.len(),
            base_dir.display()
        );
    }

    let mut index_entries = HashMap::new();
    let mut rows = Vec::new();
    for report in &batch_report.reports {
        let mut single_args = args.clone();
        single_args.url = Some(report.url.clone());
        single_args.sitemap = None;
        single_args.url_file = None;
        let path = per_page_output_path(
            &base_dir,
            &report.url,
            single_args.effective_format(),
            single_args.report_level,
        );
        single_args.output = Some(path.clone());
        output_single_report(report, &single_args, None)?;

        if with_aggregates {
            let normalized = normalize(report).normalized;
            let page_rows = technician::finding_rows(report, &normalized);
            let file = path
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_default();
            index_entries.insert(
                report.url.clone(),
                technician::ok_entry(report, &normalized, &page_rows, file),
            );
            rows.extend(page_rows);
        }
    }

    if with_aggregates {
        let index = technician::build_index(attempted_urls, batch_report, index_entries);
        let index_json =
            serde_json::to_string_pretty(&index).map_err(|e| AuditError::OutputError {
                reason: format!("index.json serialization failed: {e}"),
            })?;
        output_text(
            &index_json,
            &Some(base_dir.join("index.json")),
            "Index JSON",
            args.quiet,
        )?;
        let jsonl = technician::to_jsonl(&rows).map_err(|e| AuditError::OutputError {
            reason: format!("findings.jsonl serialization failed: {e}"),
        })?;
        output_text(
            &jsonl,
            &Some(base_dir.join("findings.jsonl")),
            "Findings JSONL",
            args.quiet,
        )?;
    }

    Ok(())
}
