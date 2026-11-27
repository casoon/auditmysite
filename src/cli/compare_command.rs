//! `auditmysite compare <OLD.json> <NEW.json>` (#1235): the command around
//! `compare` and `output::compare`. Offline: no browser, no network.

use std::path::Path;

use crate::audit::Verdict;
use crate::cli::OutputFormat;
use crate::compare::{compare, read_report};
use crate::error::{AuditError, Result};
use crate::output::compare::{format_compare_json, render_table, view};

fn usage(message: impl std::fmt::Display) -> AuditError {
    AuditError::ConfigError(format!("compare: {message}"))
}

/// The output format: `--format`, else the extension of `-o`, else a table.
fn resolve_format(format: Option<OutputFormat>, output: Option<&Path>) -> Result<OutputFormat> {
    let by_extension = || match output
        .and_then(|p| p.extension())
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("json") => OutputFormat::Json,
        Some("pdf") => OutputFormat::Pdf,
        _ => OutputFormat::Table,
    };
    match format.unwrap_or_else(by_extension) {
        f @ (OutputFormat::Table | OutputFormat::Json | OutputFormat::Pdf) => Ok(f),
        _ => Err(usage("--format is table, json or pdf")),
    }
}

/// To the file, or to stdout without one.
fn emit(output: Option<&Path>, bytes: &[u8]) -> Result<()> {
    use std::io::Write;
    match output {
        Some(path) => std::fs::write(path, bytes).map_err(|e| AuditError::FileError {
            path: path.to_path_buf(),
            reason: e.to_string(),
        }),
        None => std::io::stdout()
            .write_all(bytes)
            .map_err(|e| usage(format!("cannot write to stdout: {e}"))),
    }
}

fn file_name(path: &Path) -> String {
    path.file_name().map_or_else(
        || path.display().to_string(),
        |n| n.to_string_lossy().into_owned(),
    )
}

/// Compares the reports and writes the result. The verdict stands for the
/// exit code: `Fail` (2) when the new run has new findings or lost score,
/// `Pass` (0) otherwise; an unreadable or incompatible report is an error (3).
pub fn run_compare(
    old: &Path,
    new: &Path,
    format: Option<OutputFormat>,
    output: Option<&Path>,
    lang: &str,
) -> Result<Verdict> {
    let format = resolve_format(format, output)?;
    if format == OutputFormat::Pdf && output.is_none() {
        return Err(usage("--format pdf needs -o <FILE>"));
    }
    let comparison = compare(&read_report(old)?, &read_report(new)?);
    let en = lang != "de";

    match format {
        OutputFormat::Json => emit(output, format_compare_json(&comparison).as_bytes())?,
        OutputFormat::Table => {
            let table = render_table(&view(&comparison, &file_name(old), &file_name(new), en));
            emit(output, table.as_bytes())?;
        }
        OutputFormat::Pdf => {
            #[cfg(feature = "pdf")]
            {
                let v = view(&comparison, &file_name(old), &file_name(new), en);
                let bytes = crate::output::generate_compare_pdf(&v, lang)
                    .map_err(|e| usage(format!("cannot render the PDF: {e}")))?;
                emit(output, &bytes)?;
            }
            #[cfg(not(feature = "pdf"))]
            return Err(usage("this build has no PDF support (Cargo feature `pdf`)"));
        }
        _ => unreachable!("resolve_format admits table, json and pdf only"),
    }
    Ok(if comparison.exit_code() == 0 {
        Verdict::Pass
    } else {
        Verdict::Fail
    })
}
