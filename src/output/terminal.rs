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

use crate::audit::{
    normalize_checked, validate_batch, AuditReport, BatchReport, ReportContractError,
};
use crate::i18n::I18n;
use crate::model::{ColorPolicy, ReportLevel};
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

/// What the terminal renderers need to know about a run.
///
/// [`TerminalOptions::default`] is the plain `auditmysite <URL>` run. Start
/// from it and assign what differs:
///
/// ```
/// use auditmysite::model::ColorPolicy;
/// use auditmysite::output::terminal::TerminalOptions;
///
/// let mut options = TerminalOptions::default();
/// options.color = ColorPolicy::Never;
/// options.lang = "de".to_string();
/// ```
///
/// Non-exhaustive: a new option is a new field, so code outside this crate
/// cannot write the struct literal.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct TerminalOptions {
    /// Report level of the view model the table is built from
    /// (`--report-level`).
    pub report_level: ReportLevel,
    /// Report language, `"en"` or `"de"` (`--lang`).
    pub lang: String,
    /// Colour policy (`--color`).
    pub color: ColorPolicy,
    /// Whether the rendered text is written to a file (`--output`). A file
    /// carries no ANSI bytes unless `color` is [`ColorPolicy::Always`].
    pub writing_to_file: bool,
}

impl Default for TerminalOptions {
    /// The plain `auditmysite <URL>` run: standard report level, English,
    /// colour by terminal detection, printed to the terminal.
    fn default() -> Self {
        Self {
            report_level: ReportLevel::Standard,
            lang: "en".to_string(),
            color: ColorPolicy::Auto,
            writing_to_file: false,
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

/// The table's own texts in the run language: any language but "de" prints
/// English, as the baseline note does.
fn terminal_i18n(options: &TerminalOptions) -> I18n {
    I18n::new(if options.lang == "de" { "de" } else { "en" }).expect("the bundled locales load")
}

/// Runemark's default overflow hint names a `--details` flag this CLI does
/// not have. The table is a sample by design; the complete findings are in
/// the JSON and PDF outputs.
fn details_hint(i18n: &I18n) -> String {
    i18n.t("report-builder-terminal-details-hint")
}

fn render_options() -> RenderOptions {
    match detected_width() {
        Some(width) => RenderOptions::new().with_width(width),
        None => RenderOptions::new(),
    }
}

fn report_config(options: &TerminalOptions) -> ReportConfig {
    ReportConfig {
        level: options.report_level,
        logo_path: None,
        locale: options.lang.clone(),
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
fn console_for(options: &TerminalOptions, writing_to_file: bool) -> Console {
    let mode = if writing_to_file && options.color != ColorPolicy::Always {
        ColorMode::Never
    } else {
        ColorMode::from(options.color)
    };
    Console::stdout(mode)
}

/// Renders a single-page audit as a decision-oriented Runemark report.
pub fn render_single_report(
    report: &AuditReport,
    options: &TerminalOptions,
) -> Result<String, ReportContractError> {
    render_single_report_for(report, options, options.writing_to_file)
}

/// Renders a single-page audit, with explicit control over whether the
/// result will be written to a file (see `console_for`) — used by the
/// pre-PDF terminal preview, which always targets the terminal regardless of
/// `--output` (that flag controls the PDF's path, not this preview's).
/// `writing_to_file` replaces [`TerminalOptions::writing_to_file`].
pub fn render_single_report_for(
    report: &AuditReport,
    options: &TerminalOptions,
    writing_to_file: bool,
) -> Result<String, ReportContractError> {
    let normalized = normalize_checked(report)?;
    let vm = build_view_model(&normalized, &report_config(options));

    let verdict = terminal_verdict(normalized.normalized.decision.verdict);
    let i18n = terminal_i18n(options);

    let mut rm_report = Report::new(&vm.meta.title, verdict)
        .with_details_hint(Some(details_hint(&i18n)))
        // Accessibility first — it is what the grade below
        // classifies; the site-quality score follows, named.
        .add_metric(Metric::new(
            i18n.t("report-builder-terminal-metric-accessibility"),
            vm.summary.score.to_string(),
        ))
        .add_metric(Metric::new(
            i18n.t("report-builder-terminal-metric-site-quality"),
            vm.summary.site_quality_score.to_string(),
        ))
        .add_metric(Metric::new(
            i18n.t("report-builder-terminal-metric-grade"),
            &vm.summary.grade,
        ))
        .add_metric(Metric::new(
            i18n.t("report-builder-terminal-metric-violations"),
            vm.severity.total.to_string(),
        ))
        .add_metric(Metric::new(
            i18n.t("report-builder-terminal-metric-duration"),
            format!("{}ms", report.duration_ms),
        ));

    for tier in &vm.findings.by_severity {
        let mut group = RmFindingGroup::new(&tier.label).with_total_count(tier.total_occurrences);
        for f in tier.findings.iter().take(3) {
            let mut finding =
                Finding::new(severity_tone(f.finding.severity), &f.customer_description)
                    .with_rule_id(&f.finding.rule_id)
                    .with_remedy(&f.recommendation);
            // Where the finding sits on the page; the rule title is not a location.
            if let Some(hint) = f.location_hints.first() {
                finding =
                    finding.with_location(runemark::location::Location::Selector(hint.clone()));
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

    if let Some(baseline) = &normalized.normalized.baseline {
        rm_report = rm_report.add_scope_note(runemark::report::ScopeNote::new(
            i18n.t("report-builder-terminal-baseline-title"),
            vec![crate::audit::baseline::summary_text(
                baseline.known,
                baseline.new,
                baseline.fixed,
                None,
                options.lang != "de",
            )],
        ));
    }

    #[cfg(feature = "experimental")]
    {
        let lines = experimental_lines(&report.experimental, &i18n);
        if !lines.is_empty() {
            rm_report = rm_report.add_scope_note(runemark::report::ScopeNote::new(
                i18n.t("report-builder-terminal-experimental-title"),
                lines,
            ));
        }
    }

    let console = console_for(options, writing_to_file);
    Ok(rm_report.render_with_options(console, render_options()))
}

/// One line per experimental indicator that ran (`--experimental`, #916); the
/// table lists them only when they were asked for. The only text source of
/// these lines: `i18n` selects the language, the analysis stays English.
#[cfg(feature = "experimental")]
fn experimental_lines(
    experimental: &crate::audit::ExperimentalIndicators,
    i18n: &I18n,
) -> Vec<String> {
    use crate::audit::ExperimentalIndicator as I;
    use crate::output::module::ReportModule;

    let score_of = |value: serde_json::Value| value.get("score").and_then(|s| s.as_u64());
    let scored = |label: &str, score: Option<u64>| match score {
        Some(score) => format!("{label}: {score}/100"),
        None => i18n.t_args(
            "report-builder-terminal-no-score",
            &[("label", label.to_string())],
        ),
    };
    let mut lines = Vec::new();
    for indicator in experimental.present() {
        let line = match indicator {
            I::Ux => scored(
                "UX",
                experimental.ux.as_ref().and_then(|m| score_of(m.to_json())),
            ),
            I::Journey => scored(
                "Journey",
                experimental
                    .journey
                    .as_ref()
                    .and_then(|m| score_of(m.to_json())),
            ),
            I::AiVisibility => scored(
                "AI Visibility",
                experimental
                    .ai_visibility
                    .as_ref()
                    .and_then(|m| score_of(m.to_json())),
            ),
            I::SourceQuality => scored(
                "Source Quality",
                experimental
                    .source_quality
                    .as_ref()
                    .and_then(|m| score_of(m.to_json())),
            ),
            I::ContentVisibility => {
                let (signals, problems) = experimental
                    .content_visibility
                    .as_ref()
                    .map_or((0, 0), |m| (m.signal_count, m.problem_count));
                i18n.t_args(
                    "report-builder-terminal-content-visibility",
                    &[
                        ("problems", problems.to_string()),
                        ("signals", signals.to_string()),
                    ],
                )
            }
            I::Commerce => {
                let findings = experimental
                    .commerce
                    .as_ref()
                    .map_or(0, |m| m.findings.len());
                i18n.t_args(
                    "report-builder-terminal-commerce",
                    &[("findings", findings as i64)],
                )
            }
        };
        lines.push(line);
        if indicator == I::Ux {
            lines.extend(ux_heuristic_lines(experimental, i18n));
        }
    }
    lines
}

/// The advisory UX hints of the `ux` indicator (#1035), one line each in the
/// run language; the analysis stored the kind and the raw value.
#[cfg(feature = "experimental")]
fn ux_heuristic_lines(
    experimental: &crate::audit::ExperimentalIndicators,
    i18n: &I18n,
) -> Vec<String> {
    let Some(heuristics) = experimental.ux_heuristics.as_ref() else {
        return Vec::new();
    };
    if heuristics.findings.is_empty() {
        return vec![i18n.t("ux-heuristic-terminal-none")];
    }
    let en = i18n.locale() != "de";
    heuristics
        .findings
        .iter()
        .map(|finding| {
            i18n.t_args(
                "ux-heuristic-terminal-line",
                &[(
                    "text",
                    crate::ux::ux_heuristic_text(finding.kind, finding.count, en),
                )],
            )
        })
        .collect()
}

/// The stored decision in the terminal's vocabulary — a projection, never a
/// rule of its own (#781).
fn terminal_verdict(verdict: crate::audit::Verdict) -> Verdict {
    match verdict {
        crate::audit::Verdict::Pass => Verdict::Passed,
        crate::audit::Verdict::Warn => Verdict::Warning,
        crate::audit::Verdict::Fail => Verdict::Failed,
    }
}

/// Refuses a batch whose data contradicts itself (#790).
///
/// Renders a batch audit as a domain-wide summary — recurring rules across
/// the batch as the primary issue groups, not a stack of per-page reports.
pub fn render_batch_report(
    batch_report: &BatchReport,
    options: &TerminalOptions,
) -> Result<String, ReportContractError> {
    validate_batch(batch_report)?;
    let presentation = build_batch_presentation(batch_report);
    let p = &presentation.portfolio_summary;

    // The stored batch decision (#781), not a rule of its own.
    let verdict = terminal_verdict(batch_report.summary.decision.verdict);

    let i18n = terminal_i18n(options);
    let mut rm_report = Report::new(
        i18n.t_args(
            "report-builder-terminal-batch-title",
            &[("domain", p.domain.to_string())],
        ),
        verdict,
    )
    .with_details_hint(Some(details_hint(&i18n)))
    .add_metric(Metric::new(
        i18n.t("report-builder-terminal-metric-urls"),
        p.total_urls.to_string(),
    ))
    .add_metric(Metric::new(
        i18n.t("report-builder-terminal-metric-passed"),
        p.passed.to_string(),
    ))
    .add_metric(Metric::new(
        i18n.t("report-builder-terminal-metric-failed"),
        p.fail_urls.to_string(),
    ))
    .add_metric(Metric::new(
        i18n.t("report-builder-terminal-metric-warned"),
        p.warn_urls.to_string(),
    ))
    .add_metric(Metric::new(
        i18n.t("report-builder-terminal-metric-avg-score"),
        batch_report
            .summary
            .average_score
            .map_or("–".to_string(), |average| {
                format!(
                    "{}/100",
                    crate::audit::average_score_text(average, i18n.locale() == "en")
                )
            }),
    ))
    .add_metric(Metric::new(
        i18n.t("report-builder-terminal-metric-violations"),
        p.total_violations.to_string(),
    ))
    .add_metric(Metric::new(
        i18n.t("report-builder-terminal-metric-duration"),
        format!("{}ms", p.duration_ms),
    ));

    if let Some(sample) = &batch_report.sample {
        rm_report = rm_report.add_scope_note(runemark::report::ScopeNote::new(
            i18n.t("report-builder-terminal-scope-title"),
            vec![i18n.t_args(
                "report-builder-terminal-scope-sample",
                &[
                    ("audited", sample.audited.to_string()),
                    ("total", sample.total_discovered.to_string()),
                    ("source", sample.source.to_string()),
                ],
            )],
        ));
    }
    if let Some(baseline) = &batch_report.summary.baseline {
        rm_report = rm_report.add_scope_note(runemark::report::ScopeNote::new(
            i18n.t("report-builder-terminal-baseline-title"),
            vec![crate::audit::baseline::summary_text(
                baseline.known,
                baseline.new,
                baseline.fixed,
                Some(baseline.unmatched_pages),
                options.lang != "de",
            )],
        ));
    }
    if !batch_report.errors.is_empty() {
        rm_report = rm_report.add_scope_note(runemark::report::ScopeNote::new(
            i18n.t("report-builder-terminal-score-basis-title"),
            vec![i18n.t_args(
                "report-builder-terminal-score-basis",
                &[
                    ("audited", p.total_urls.to_string()),
                    (
                        "total",
                        (p.total_urls + batch_report.errors.len()).to_string(),
                    ),
                    ("failed", batch_report.errors.len().to_string()),
                ],
            )],
        ));
    }

    let mut group = RmFindingGroup::new(i18n.t("report-builder-terminal-top-rules"))
        .with_total_count(presentation.top_issues.len());
    for issue in presentation.top_issues.iter().take(5) {
        group = group.add_finding(
            Finding::new(
                severity_tone(issue.finding.severity),
                &issue.customer_description,
            )
            .with_rule_id(&issue.finding.rule_id)
            .with_remedy(&issue.recommendation),
        );
    }
    if !group.findings.is_empty() {
        rm_report = rm_report.add_group(group);
    }

    for err in batch_report.errors.iter().take(5) {
        rm_report = rm_report.add_next_step(NextStep::new(i18n.t_args(
            "report-builder-terminal-fix-audit-error",
            &[
                ("url", err.url.to_string()),
                ("error", err.error.to_string()),
            ],
        )));
    }

    let console = console_for(options, options.writing_to_file);
    Ok(rm_report.render_with_options(console, render_options()))
}

#[cfg(test)]
#[cfg(feature = "experimental")]
pub(crate) mod text_export;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::normalized::normalize;
    use crate::audit::BatchError;
    use crate::wcag::WcagResults;

    fn test_options() -> TerminalOptions {
        TerminalOptions {
            color: ColorPolicy::Never,
            ..Default::default()
        }
    }

    fn fixture_report() -> AuditReport {
        AuditReport::new(
            "https://example.com".to_string(),
            crate::model::WcagLevel::AA,
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
                blocked_reason: None,
            }],
            1500,
        )
    }

    fn has_ansi(s: &str) -> bool {
        s.contains('\x1b')
    }

    /// #916: the table lists experimental indicators only when they ran.
    #[test]
    #[cfg(feature = "experimental")]
    fn table_lists_experimental_indicators_only_when_they_ran() {
        let options = test_options();
        let plain = render_single_report(&fixture_report(), &options).unwrap();
        assert!(!plain.contains("Experimental"), "{plain}");

        let mut report = fixture_report();
        report.experimental.ux = Some(crate::analyze_ux(&crate::AXTree::new()));
        let table = render_single_report(&report, &options).unwrap();
        assert!(table.contains("Experimental indicators"), "{table}");
        assert!(table.contains("UX: "), "{table}");

        let mut german = options.clone();
        german.lang = "de".to_string();
        let table = render_single_report(&report, &german).unwrap();
        assert!(table.contains("Experimentelle Indikatoren"), "{table}");
    }

    /// Localization guard (#406): the English lines hold no German.
    #[test]
    #[cfg(feature = "experimental")]
    fn english_experimental_lines_hold_no_german() {
        let mut report = fixture_report();
        report.experimental.ux = Some(crate::analyze_ux(&crate::AXTree::new()));
        report.experimental.content_visibility = Some(
            crate::content_visibility::analyze_content_visibility(&fixture_report()),
        );
        let lines = experimental_lines(&report.experimental, &I18n::new("en").unwrap());
        assert_eq!(lines.len(), 2);
        for line in &lines {
            assert!(
                !line.chars().any(|c| "äöüÄÖÜß".contains(c)),
                "German in an English line: {line}"
            );
            assert!(!line.contains("von ") && !line.contains("kein "), "{line}");
        }
        let german = experimental_lines(&report.experimental, &I18n::new("de").unwrap());
        assert!(german.iter().any(|l| l.contains("Signalen")), "{german:?}");
    }

    /// #1035: the advisory UX hints follow the UX line, in the run language;
    /// the English lines hold no German.
    #[test]
    #[cfg(feature = "experimental")]
    fn ux_hints_follow_the_ux_line_in_the_run_language() {
        use crate::ux::{UxHeuristicFinding, UxHeuristicKind, UxHeuristics};
        let mut report = fixture_report();
        report.experimental.ux = Some(crate::analyze_ux(&crate::AXTree::new()));
        report.experimental.ux_heuristics = Some(UxHeuristics { findings: vec![] });
        let en = I18n::new("en").unwrap();
        let de = I18n::new("de").unwrap();
        let none = experimental_lines(&report.experimental, &en);
        assert_eq!(none.len(), 2);
        assert_eq!(none[1], "UX hints: none");
        assert_eq!(
            experimental_lines(&report.experimental, &de)[1],
            "UX-Hinweise: keine"
        );

        report.experimental.ux_heuristics = Some(UxHeuristics {
            findings: vec![
                UxHeuristicFinding {
                    kind: UxHeuristicKind::ManyCtas,
                    severity: crate::taxonomy::Severity::Low,
                    count: Some(9),
                    message: String::new(),
                },
                UxHeuristicFinding {
                    kind: UxHeuristicKind::NoCta,
                    severity: crate::taxonomy::Severity::Low,
                    count: None,
                    message: String::new(),
                },
            ],
        });
        let english = experimental_lines(&report.experimental, &en);
        assert_eq!(english.len(), 3);
        assert!(english[0].starts_with("UX: "), "{english:?}");
        assert!(
            english[1].starts_with("UX hint: 9 buttons or links"),
            "{english:?}"
        );
        for line in &english {
            assert!(!line.chars().any(|c| "äöüÄÖÜß".contains(c)), "{line}");
        }
        let german = experimental_lines(&report.experimental, &de);
        assert!(
            german[1].starts_with("UX-Hinweis: 9 Schaltflächen"),
            "{german:?}"
        );
    }

    #[test]
    fn single_report_renders_deterministically_at_width_40_and_80() {
        let report = fixture_report();
        let options = test_options();
        let vm = build_view_model(&normalize(&report), &report_config(&options));
        let rm_report = Report::new(&vm.meta.title, Verdict::Passed);
        let console = Console::stdout(ColorMode::Never);

        let at_40 = rm_report.render_with_options(console, RenderOptions::new().with_width(40));
        let at_80 = rm_report.render_with_options(console, RenderOptions::new().with_width(80));

        assert!(!at_40.is_empty());
        assert!(!at_80.is_empty());
        assert!(!has_ansi(&at_40));
        assert!(!has_ansi(&at_80));
    }

    /// #781: the terminal prints the stored decision. A page with a High
    /// Level A barrier fails there exactly as in the JSON and the exit code;
    /// a clean page does not.
    #[test]
    fn terminal_prints_the_stored_decision() {
        use crate::wcag::Violation;
        let clean = fixture_report();
        let mut results = WcagResults::new();
        results.add_violation(Violation::new(
            "1.1.1",
            "Non-text Content",
            crate::model::WcagLevel::A,
            crate::taxonomy::Severity::High,
            "Image without alternative text",
            "img-1",
        ));
        let broken = AuditReport::new(
            "https://example.com".to_string(),
            crate::model::WcagLevel::AA,
            results,
            42,
        );
        let options = test_options();
        assert_eq!(
            normalize(&broken).normalized.decision.verdict,
            crate::audit::Verdict::Fail
        );
        let clean_out = render_single_report(&clean, &options).unwrap();
        let broken_out = render_single_report(&broken, &options).unwrap();
        assert!(clean_out.starts_with("[OK]"), "{clean_out}");
        assert!(broken_out.starts_with("[FAIL]"), "{broken_out}");
    }

    /// The overflow line must not point at a `--details` flag that does not exist.
    #[test]
    fn overflow_hint_names_real_output_formats() {
        use crate::wcag::Violation;
        let mut results = WcagResults::new();
        for i in 0..5 {
            results.add_violation(Violation::new(
                "1.1.1",
                "Non-text Content",
                crate::model::WcagLevel::A,
                crate::taxonomy::Severity::High,
                format!("Image {i} without alternative text"),
                format!("img-{i}"),
            ));
        }
        let report = AuditReport::new(
            "https://example.com".to_string(),
            crate::model::WcagLevel::AA,
            results,
            42,
        );
        let output = render_single_report(&report, &test_options()).unwrap();
        assert!(output.contains("more finding(s)"), "{output}");
        assert!(
            output.contains(&details_hint(&terminal_i18n(&test_options()))),
            "{output}"
        );
        assert!(!output.contains("--details"), "{output}");
    }

    #[test]
    fn single_report_never_emits_ansi_under_color_never() {
        let report = fixture_report();
        let options = test_options();
        let output = render_single_report(&report, &options).unwrap();
        assert!(!has_ansi(&output));
        assert!(output.contains(&vm_title(&report, &options)));
    }

    fn vm_title(report: &AuditReport, options: &TerminalOptions) -> String {
        let vm = build_view_model(&normalize(report), &report_config(options));
        vm.meta.title
    }

    #[test]
    fn single_report_emits_ansi_under_color_always() {
        let report = fixture_report();
        let mut options = test_options();
        options.color = ColorPolicy::Always;
        let output = render_single_report(&report, &options).unwrap();
        assert!(has_ansi(&output));
    }

    #[test]
    fn color_policy_maps_to_the_console_mode() {
        assert_eq!(ColorMode::from(ColorPolicy::Auto), ColorMode::Auto);
        assert_eq!(ColorMode::from(ColorPolicy::Always), ColorMode::Always);
        assert_eq!(ColorMode::from(ColorPolicy::Never), ColorMode::Never);
    }

    #[test]
    fn writing_to_file_forces_no_color_even_under_auto() {
        let report = fixture_report();
        let mut options = test_options();
        options.color = ColorPolicy::Auto;
        let with_file = render_single_report_for(&report, &options, true).unwrap();
        assert!(!has_ansi(&with_file));
    }

    #[test]
    fn writing_to_file_does_not_override_explicit_color_always() {
        let report = fixture_report();
        let mut options = test_options();
        options.color = ColorPolicy::Always;
        let with_file = render_single_report_for(&report, &options, true).unwrap();
        assert!(has_ansi(&with_file));
    }

    #[test]
    fn batch_report_never_emits_ansi_under_color_never() {
        let batch = fixture_batch();
        let options = test_options();
        let output = render_batch_report(&batch, &options).unwrap();
        assert!(!has_ansi(&output));
        assert!(output.contains("example.com"));
    }

    #[test]
    fn batch_report_states_score_coverage_when_urls_failed() {
        let batch = fixture_batch();
        let options = test_options();
        let output = render_batch_report(&batch, &options).unwrap();
        assert!(output.contains("Scores cover 1 of 2 URLs; 1 could not be audited"));
    }

    #[test]
    fn batch_without_a_measured_url_prints_a_dash_not_a_score() {
        let batch = BatchReport::from_reports(
            vec![],
            vec![BatchError {
                url: "https://example.com/broken".to_string(),
                error: "timeout".to_string(),
                blocked_reason: None,
            }],
            1500,
        );
        assert!(batch.summary.average_score.is_none());
        assert_eq!(
            crate::output::builder::build_batch_presentation(&batch)
                .portfolio_summary
                .average_score,
            None,
            "the presentation model carries no stand-in zero"
        );
        let output = render_batch_report(&batch, &test_options()).unwrap();
        let line = output
            .lines()
            .find(|line| line.contains("Avg. score"))
            .expect("average score metric is rendered");
        assert!(
            line.contains('–'),
            "unmeasured average prints a dash: {line}"
        );
        assert!(!line.contains("0/100"), "no stand-in zero: {line}");
    }

    #[test]
    fn batch_report_lists_errors_as_next_steps() {
        let batch = fixture_batch();
        let options = test_options();
        let output = render_batch_report(&batch, &options).unwrap();
        assert!(output.contains("broken"));
    }

    /// `--lang de` prints the table's own labels in German; the English run
    /// keeps English and no umlauts.
    #[test]
    fn table_labels_follow_the_run_language() {
        let mut de = test_options();
        de.lang = "de".to_string();
        let en = test_options();
        let single_de = render_single_report(&fixture_report(), &de).unwrap();
        let batch_de = render_batch_report(&fixture_batch(), &de).unwrap();
        for out in [&single_de, &batch_de] {
            assert!(out.contains("Verstöße") && out.contains("Dauer"), "{out}");
            for english in ["Violations", "Duration", "Accessibility:", "Avg. score"] {
                assert!(!out.contains(english), "{english} in German output:\n{out}");
            }
        }
        assert!(batch_de.contains("Bestanden"), "{batch_de}");
        assert!(
            batch_de.contains("Die Scores decken 1 von 2 URLs ab"),
            "{batch_de}"
        );
        let single_en = render_single_report(&fixture_report(), &en).unwrap();
        let batch_en = render_batch_report(&fixture_batch(), &en).unwrap();
        for out in [&single_en, &batch_en] {
            assert!(
                out.contains("Violations") && out.contains("Duration"),
                "{out}"
            );
            assert!(!out.chars().any(|c| "äöüÄÖÜß".contains(c)), "{out}");
        }
    }

    /// The location line shows where a finding sits, never the rule title.
    #[test]
    fn finding_location_is_not_the_rule_title() {
        use crate::wcag::Violation;
        let mut results = WcagResults::new();
        results.add_violation(Violation::new(
            "1.1.1",
            "Non-text Content",
            crate::model::WcagLevel::A,
            crate::taxonomy::Severity::High,
            "Image without alternative text".to_string(),
            "img-1".to_string(),
        ));
        let report = AuditReport::new(
            "https://example.com".to_string(),
            crate::model::WcagLevel::AA,
            results,
            42,
        );
        let vm = build_view_model(&normalize(&report), &report_config(&test_options()));
        let title = &vm.findings.by_severity[0].findings[0].title;
        let output = render_single_report(&report, &test_options()).unwrap();
        assert!(
            !output.contains(&format!("at `{title}`")),
            "the title is printed as a location:\n{output}"
        );
    }

    #[test]
    fn ascii_theme_is_used_when_color_is_never() {
        // #529: ColorMode::Never automatically falls back to the ASCII symbol
        // theme (no Unicode glyphs), confirmed by runemark's own Console::new.
        let console = Console::stdout(ColorMode::Never);
        assert_eq!(console.symbol_theme(), runemark::SymbolTheme::Ascii);
    }
}
