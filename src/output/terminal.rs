//! Terminal presentation boundary built on `runemark` (#529).
//!
//! AuditMySite continues to own all audit/WCAG/batch/JSON/PDF models; this
//! module only maps the existing presentation layer (`ReportViewModel`/
//! `BatchPresentation`, already shared with JSON/PDF) into Runemark's
//! decision-oriented `Report` primitives. No AuditMySite domain type moves
//! into Runemark, and Runemark never inspects the terminal itself — width and
//! color policy are host-provided.

use runemark::report::RenderOptions;
use runemark::{
    ColorMode, Console, Finding, FindingGroup as RmFindingGroup, Metric, NextStep, Report, Tone,
    Verdict,
};

use crate::audit::{normalize, AuditReport, BatchReport};
use crate::cli::{Args, ColorPolicy};
use crate::output::builder::{build_batch_presentation, build_view_model};
use crate::output::report_model::ReportConfig;
use crate::taxonomy::Severity;

impl From<ColorPolicy> for ColorMode {
    fn from(policy: ColorPolicy) -> Self {
        match policy {
            ColorPolicy::Auto => ColorMode::Auto,
            ColorPolicy::Always => ColorMode::Always,
            ColorPolicy::Never => ColorMode::Never,
        }
    }
}

fn severity_tone(severity: Severity) -> Tone {
    match severity {
        Severity::Critical | Severity::High => Tone::Error,
        Severity::Medium => Tone::Warning,
        Severity::Low => Tone::Info,
    }
}

/// Detected stdout width, or `None` when it cannot be determined (e.g.
/// redirected output) — `RenderOptions` then stays unconstrained.
fn detected_width() -> Option<usize> {
    console::Term::stdout()
        .size_checked()
        .map(|(_rows, cols)| cols as usize)
}

fn render_options() -> RenderOptions {
    match detected_width() {
        Some(width) => RenderOptions::new().with_width(width),
        None => RenderOptions::new(),
    }
}

fn report_config(args: &Args) -> ReportConfig {
    ReportConfig {
        level: args.report_level,
        logo_path: None,
        locale: args.lang.clone(),
        annex: None,
    }
}

/// Resolves the `Console` for a table render.
///
/// `Console::stdout(Auto)` checks the process's real stdout TTY-ness, which is
/// unrelated to whether *this particular* rendered string ends up written to
/// a file. Saved files must not carry ANSI bytes, so callers that write the
/// result to a path (`--output`) pass `writing_to_file: true` to force
/// `Never`, unless the user explicitly asked for `--color always`. A pure
/// terminal preview (e.g. the pre-PDF table preview) passes `false`.
fn console_for(args: &Args, writing_to_file: bool) -> Console {
    let mode = if writing_to_file && args.color != ColorPolicy::Always {
        ColorMode::Never
    } else {
        ColorMode::from(args.color)
    };
    Console::stdout(mode)
}

/// Renders a single-page audit as a decision-oriented Runemark report.
pub fn render_single_report(report: &AuditReport, args: &Args) -> String {
    render_single_report_for(report, args, args.output.is_some())
}

/// Renders a single-page audit, with explicit control over whether the
/// result will be written to a file (see `console_for`) — used by the
/// pre-PDF terminal preview, which always targets the terminal regardless of
/// `--output` (that flag controls the PDF's path, not this preview's).
pub fn render_single_report_for(
    report: &AuditReport,
    args: &Args,
    writing_to_file: bool,
) -> String {
    let normalized = normalize(report);
    let vm = build_view_model(&normalized, &report_config(args));

    let verdict = if report.passed() {
        Verdict::Passed
    } else if vm.severity.critical > 0 {
        Verdict::Failed
    } else {
        Verdict::Warning
    };

    let mut rm_report = Report::new(&vm.meta.title, verdict)
        // Accessibility first — it is what grade and certificate below
        // classify (plan 29, D1); the combined technical value follows, named.
        .add_metric(Metric::new("Accessibility", vm.summary.score.to_string()))
        .add_metric(Metric::new(
            "Combined technical",
            vm.summary.overall_score.to_string(),
        ))
        .add_metric(Metric::new("Grade", &vm.summary.grade))
        .add_metric(Metric::new("Certificate", &vm.summary.certificate))
        .add_metric(Metric::new("Violations", vm.severity.total.to_string()))
        .add_metric(Metric::new("Duration", format!("{}ms", report.duration_ms)));

    for tier in &vm.findings.by_severity {
        let mut group = RmFindingGroup::new(&tier.label).with_total_count(tier.total_occurrences);
        for f in tier.findings.iter().take(3) {
            let mut finding = Finding::new(severity_tone(f.severity), &f.customer_description)
                .with_rule_id(&f.rule_id)
                .with_remedy(&f.recommendation);
            if !f.title.is_empty() {
                finding =
                    finding.with_location(runemark::location::Location::Selector(f.title.clone()));
            }
            group = group.add_finding(finding);
        }
        rm_report = rm_report.add_group(group);
    }

    for item in vm
        .actions
        .roadmap_columns
        .iter()
        .flat_map(|c| c.items.iter())
        .take(5)
    {
        rm_report = rm_report.add_next_step(NextStep::new(&item.action));
    }

    let console = console_for(args, writing_to_file);
    rm_report.render_with_options(console, render_options())
}

/// Renders a batch audit as a domain-wide summary — recurring rules across
/// the batch as the primary issue groups, not a stack of per-page reports.
pub fn render_batch_report(batch_report: &BatchReport, args: &Args) -> String {
    let presentation = build_batch_presentation(batch_report);
    let p = &presentation.portfolio_summary;

    let verdict = if p.failed == 0 {
        Verdict::Passed
    } else if p.passed == 0 {
        Verdict::Failed
    } else {
        Verdict::Warning
    };

    let mut rm_report = Report::new(format!("Batch Audit — {}", p.domain), verdict)
        .add_metric(Metric::new("URLs", p.total_urls.to_string()))
        .add_metric(Metric::new("Passed", p.passed.to_string()))
        .add_metric(Metric::new("Failed", p.failed.to_string()))
        .add_metric(Metric::new(
            "Avg. score",
            format!("{:.0}/100", p.average_score),
        ))
        .add_metric(Metric::new("Violations", p.total_violations.to_string()))
        .add_metric(Metric::new("Duration", format!("{}ms", p.duration_ms)));

    if let Some(sample) = &batch_report.sample {
        rm_report = rm_report.add_scope_note(runemark::report::ScopeNote::new(
            "Scope",
            vec![format!(
                "{} of {} discovered URLs ({})",
                sample.audited, sample.total_discovered, sample.source
            )],
        ));
    }

    let mut group =
        RmFindingGroup::new("Top recurring rules").with_total_count(presentation.top_issues.len());
    for issue in presentation.top_issues.iter().take(5) {
        group = group.add_finding(
            Finding::new(severity_tone(issue.severity), &issue.customer_description)
                .with_rule_id(&issue.rule_id)
                .with_remedy(&issue.recommendation),
        );
    }
    if !group.findings.is_empty() {
        rm_report = rm_report.add_group(group);
    }

    for err in batch_report.errors.iter().take(5) {
        rm_report = rm_report.add_next_step(NextStep::new(format!(
            "Fix audit error for {}: {}",
            err.url, err.error
        )));
    }

    let console = console_for(args, args.output.is_some());
    rm_report.render_with_options(console, render_options())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::BatchError;
    use crate::cli::Args;
    use crate::wcag::WcagResults;
    use clap::Parser;

    fn test_args() -> Args {
        let mut args = Args::parse_from(["auditmysite", "https://example.com"]);
        args.color = ColorPolicy::Never;
        args
    }

    fn fixture_report() -> AuditReport {
        AuditReport::new(
            "https://example.com".to_string(),
            crate::cli::WcagLevel::AA,
            WcagResults::new(),
            42,
        )
    }

    fn fixture_batch() -> BatchReport {
        BatchReport::from_reports(
            vec![fixture_report()],
            vec![BatchError {
                url: "https://example.com/broken".to_string(),
                error: "timeout".to_string(),
            }],
            1500,
        )
    }

    fn has_ansi(s: &str) -> bool {
        s.contains('\x1b')
    }

    #[test]
    fn single_report_renders_deterministically_at_width_40_and_80() {
        let report = fixture_report();
        let args = test_args();
        let vm = build_view_model(&normalize(&report), &report_config(&args));
        let rm_report = Report::new(&vm.meta.title, Verdict::Passed);
        let console = Console::stdout(ColorMode::Never);

        let at_40 = rm_report.render_with_options(console, RenderOptions::new().with_width(40));
        let at_80 = rm_report.render_with_options(console, RenderOptions::new().with_width(80));

        assert!(!at_40.is_empty());
        assert!(!at_80.is_empty());
        assert!(!has_ansi(&at_40));
        assert!(!has_ansi(&at_80));
    }

    #[test]
    fn single_report_never_emits_ansi_under_color_never() {
        let report = fixture_report();
        let args = test_args();
        let output = render_single_report(&report, &args);
        assert!(!has_ansi(&output));
        assert!(output.contains(&vm_title(&report, &args)));
    }

    fn vm_title(report: &AuditReport, args: &Args) -> String {
        let vm = build_view_model(&normalize(report), &report_config(args));
        vm.meta.title
    }

    #[test]
    fn single_report_emits_ansi_under_color_always() {
        let report = fixture_report();
        let mut args = test_args();
        args.color = ColorPolicy::Always;
        let output = render_single_report(&report, &args);
        assert!(has_ansi(&output));
    }

    #[test]
    fn writing_to_file_forces_no_color_even_under_auto() {
        let report = fixture_report();
        let mut args = test_args();
        args.color = ColorPolicy::Auto;
        let with_file = render_single_report_for(&report, &args, true);
        assert!(!has_ansi(&with_file));
    }

    #[test]
    fn writing_to_file_does_not_override_explicit_color_always() {
        let report = fixture_report();
        let mut args = test_args();
        args.color = ColorPolicy::Always;
        let with_file = render_single_report_for(&report, &args, true);
        assert!(has_ansi(&with_file));
    }

    #[test]
    fn batch_report_never_emits_ansi_under_color_never() {
        let batch = fixture_batch();
        let args = test_args();
        let output = render_batch_report(&batch, &args);
        assert!(!has_ansi(&output));
        assert!(output.contains("example.com"));
    }

    #[test]
    fn batch_report_lists_errors_as_next_steps() {
        let batch = fixture_batch();
        let args = test_args();
        let output = render_batch_report(&batch, &args);
        assert!(output.contains("broken"));
    }

    #[test]
    fn ascii_theme_is_used_when_color_is_never() {
        // #529: ColorMode::Never automatically falls back to the ASCII symbol
        // theme (no Unicode glyphs), confirmed by runemark's own Console::new.
        let console = Console::stdout(ColorMode::Never);
        assert_eq!(console.symbol_theme(), runemark::SymbolTheme::Ascii);
    }
}
