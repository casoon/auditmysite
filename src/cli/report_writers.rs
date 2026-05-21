//! Report output dispatchers.
//!
//! Converts AuditReport / BatchReport / ComparisonReport into the format
//! the user requested and writes to a file or stdout. Extracted from main.rs.

use colored::Colorize;

use auditmysite::audit::history::preview_report_history;
use auditmysite::audit::normalize;
use auditmysite::cli::{Args, OutputFormat};
use auditmysite::error::{AuditError, Result};
#[cfg(feature = "pdf")]
use auditmysite::output::report_model::ReportConfig;
use auditmysite::output::{
    format_ai_json, format_batch_table, format_json_batch, format_summary, print_batch_table,
    print_report, UnifiedReport,
};
#[cfg(feature = "pdf")]
use auditmysite::output::{generate_batch_pdf, generate_batch_typ, generate_pdf, generate_typ};

#[cfg(feature = "pdf")]
use crate::output_paths::output_bytes;
#[cfg(feature = "pdf")]
use crate::output_paths::{
    default_batch_pdf_output_path, default_single_json_output_path, default_single_pdf_output_path,
};
use crate::output_paths::{
    output_directory, output_text, per_page_output_directory, per_page_output_path,
};

pub fn output_single_report(report: &auditmysite::AuditReport, args: &Args) -> Result<()> {
    match args.effective_format() {
        OutputFormat::Json => {
            let normalized = normalize(report);
            let mut unified = UnifiedReport::single(&normalized, report);
            if let Some(path) = args.output.as_ref() {
                if let Ok(Some(preview)) =
                    preview_report_history(output_directory(path), path, &normalized)
                {
                    if let Ok(history) = serde_json::to_value(&preview) {
                        unified.set_history(history);
                    }
                }
            }
            let output = unified.to_json(true)?;
            output_text(&output, &args.output, "JSON", args.quiet)?;
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
                let raw_history =
                    preview_report_history(output_directory(&path), &path, &normalized)
                        .ok()
                        .flatten();
                let history_preview = raw_history.as_ref().map(|preview| {
                    auditmysite::output::report_model::ReportHistoryPreview {
                        previous_date: preview.previous_date.clone(),
                        timeline_entries: preview.timeline_entries,
                        previous_accessibility_score: preview.previous_accessibility_score,
                        previous_overall_score: preview.previous_overall_score,
                        delta_accessibility: preview.delta.accessibility_score_delta,
                        delta_overall: preview.delta.overall_score_delta,
                        delta_total_issues: preview.delta.total_issues_delta,
                        delta_critical_issues: preview.delta.critical_issues_delta,
                        recent_entries: preview
                            .recent_entries
                            .iter()
                            .map(|entry| {
                                (
                                    entry.timestamp.format("%d.%m.%Y").to_string(),
                                    entry.accessibility_score,
                                    entry.overall_score,
                                    entry.grade.clone(),
                                    entry.severity_counts.total as u32,
                                )
                            })
                            .collect(),
                        new_findings: preview.delta.new_findings.clone(),
                        resolved_findings: preview.delta.resolved_findings.clone(),
                    }
                });
                let config = ReportConfig {
                    level: args.report_level,
                    logo_path: args.logo.clone(),
                    locale: args.lang.clone(),
                    history_preview,
                };
                let pdf_bytes = generate_pdf(report, &config).map_err(|e| {
                    AuditError::ReportGenerationFailed {
                        reason: e.to_string(),
                    }
                })?;
                if let Some(json_path) = auto_json_path.as_ref() {
                    let mut unified = UnifiedReport::single(&normalized, report);
                    if let Some(ref preview) = raw_history {
                        if let Ok(history) = serde_json::to_value(preview) {
                            unified.set_history(history);
                        }
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
            let output = format_summary(&normalized).map_err(|e| AuditError::OutputError {
                reason: e.to_string(),
            })?;
            output_text(&output, &args.output, "summary JSON", args.quiet)?;
        }
    }
    Ok(())
}

pub fn output_batch_report(
    batch_report: &auditmysite::audit::BatchReport,
    args: &Args,
) -> Result<()> {
    match args.effective_format() {
        OutputFormat::Json => {
            let output = format_json_batch(batch_report, true)?;
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
                    history_preview: None,
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
                let json_output = format_json_batch(batch_report, true)?;
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
                    format_summary(&normalized).ok()
                })
                .collect();
            let combined = format!("[\n{}\n]", summaries.join(",\n"));
            output_text(&combined, &args.output, "summary JSON batch", args.quiet)?;
        }
    }
    Ok(())
}

pub fn output_comparison_report(
    comparison: &auditmysite::audit::ComparisonReport,
    args: &Args,
) -> Result<()> {
    match args.effective_format() {
        OutputFormat::Json => {
            let output =
                serde_json::to_string_pretty(comparison).map_err(|e| AuditError::OutputError {
                    reason: e.to_string(),
                })?;
            output_text(&output, &args.output, "JSON comparison", args.quiet)?;
        }
        OutputFormat::Table => {
            if let Some(path) = &args.output {
                let mut lines = vec![
                    "Rank,Domain,Overall,Accessibility,OverallGrade,Violations,Critical"
                        .to_string(),
                ];
                for (i, e) in comparison.entries.iter().enumerate() {
                    lines.push(format!(
                        "{},{},{},{},{},{},{}",
                        i + 1,
                        e.domain,
                        e.overall_score,
                        e.accessibility_score,
                        e.grade,
                        e.total_violations,
                        e.critical_violations,
                    ));
                }
                output_text(
                    &lines.join("\n"),
                    &Some(path.clone()),
                    "CSV comparison",
                    args.quiet,
                )?;
            }
        }
        OutputFormat::Pdf => {
            #[cfg(feature = "pdf")]
            {
                use auditmysite::output::generate_comparison_pdf;
                use auditmysite::output::report_model::ReportConfig;
                let config = ReportConfig {
                    level: args.report_level,
                    logo_path: args.logo.clone(),
                    locale: args.lang.clone(),
                    history_preview: None,
                };
                let pdf_bytes = generate_comparison_pdf(comparison, &config).map_err(|e| {
                    AuditError::ReportGenerationFailed {
                        reason: e.to_string(),
                    }
                })?;
                let path = args
                    .output
                    .clone()
                    .unwrap_or_else(|| std::path::PathBuf::from("comparison-report.pdf"));
                output_bytes(&pdf_bytes, &path, "PDF comparison", args.quiet)?;
            }
            #[cfg(not(feature = "pdf"))]
            {
                return Err(AuditError::ConfigError(
                    "PDF output requires the 'pdf' feature.".to_string(),
                ));
            }
        }
        OutputFormat::Ai => {
            let output =
                serde_json::to_string_pretty(comparison).map_err(|e| AuditError::OutputError {
                    reason: e.to_string(),
                })?;
            output_text(&output, &args.output, "AI JSON comparison", args.quiet)?;
        }
        OutputFormat::Summary => {
            let output =
                serde_json::to_string_pretty(comparison).map_err(|e| AuditError::OutputError {
                    reason: e.to_string(),
                })?;
            output_text(&output, &args.output, "summary JSON comparison", args.quiet)?;
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
        output_single_report(report, &single_args)?;
    }

    Ok(())
}
