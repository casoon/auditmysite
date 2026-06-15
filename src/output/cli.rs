//! CLI Output Formatter
//!
//! Generates human-readable terminal output for interactive use and batch tables.

use colored::{control::set_override, Colorize};
use comfy_table::{presets, Attribute, Cell, Color, ContentArrangement, Table};
use std::cmp::min;

use crate::audit::{AuditReport, BatchReport, BudgetSeverity, PerformanceResults};
use crate::cli::WcagLevel;
use crate::mobile::MobileFriendliness;
use crate::security::SecurityAnalysis;
use crate::seo::SeoAnalysis;
use crate::util::truncate_url;
use crate::wcag::{Severity, Violation};

/// Format and print the audit report to the terminal
pub fn print_report(report: &AuditReport, level: WcagLevel) {
    println!();
    print_dashboard(report, level);

    if !report.wcag_results.violations.is_empty() {
        print_violations_table(&report.wcag_results.violations);
    }

    // Optional module results
    if let Some(ref perf) = report.performance {
        print_performance_section(perf);
    }
    if let Some(ref seo) = report.seo {
        print_seo_section(seo);
    }
    if let Some(ref sec) = report.security {
        print_security_section(sec);
    }
    if let Some(ref mobile) = report.mobile {
        print_mobile_section(mobile);
    }

    if !report.budget_violations.is_empty() {
        print_budget_violations_section(&report.budget_violations);
    }
    if let Some(ref dm) = report.dark_mode {
        print_dark_mode_section(dm);
    }
    if let Some(ref cv) = report.content_visibility {
        print_content_visibility_section(cv);
    }

    print_footer(report);
}

fn print_dashboard(report: &AuditReport, level: WcagLevel) {
    println!("{}", "auditmysite".truecolor(140, 154, 181).bold());
    println!("{}", format!("$ auditmysite {}", report.url).white().bold());
    println!();

    for line in dashboard_rows(report) {
        println!("{line}");
    }

    println!();
    let viewport_info = if let Some(ref vs) = report.viewport_scores {
        format!(
            "  Desktop {}  Mobile {}  Weighted {}",
            colorize_score(vs.desktop.overall, &vs.desktop.overall.to_string()),
            colorize_score(vs.mobile.overall, &vs.mobile.overall.to_string()),
            colorize_score(vs.weighted_overall, &vs.weighted_overall.to_string()),
        )
    } else {
        String::new()
    };

    let normalized = crate::audit::normalize(report);
    println!(
        "{}",
        format!(
            "  WCAG {}  Nodes {}  Duration {:.1}s  Overall {}  Certificate {}",
            level,
            report.nodes_analyzed,
            report.duration_ms as f64 / 1000.0,
            normalized.normalized.overall_score,
            report.certificate
        )
        .dimmed()
    );
    if !viewport_info.is_empty() {
        println!("{}", viewport_info);
    }
    for flag in &normalized.normalized.audit_flags {
        if flag.kind == "viewport_gap" {
            println!(
                "  {}",
                format!("Note: {}", flag.message).truecolor(255, 165, 0)
            );
        }
    }

    // Risk level (computed from violations)
    let critical = report
        .wcag_results
        .violations
        .iter()
        .filter(|v| v.severity == Severity::Critical)
        .count();
    let high = report
        .wcag_results
        .violations
        .iter()
        .filter(|v| v.severity == Severity::High)
        .count();
    let risk_label = if critical >= 3 {
        "CRITICAL"
    } else if critical >= 1 && high >= 2 {
        "HIGH"
    } else if high >= 3 {
        "MEDIUM"
    } else {
        "LOW"
    };
    let risk_colored = match risk_label {
        "CRITICAL" => format!("Risk: {risk_label}").red().bold(),
        "HIGH" => format!("Risk: {risk_label}").truecolor(255, 165, 0).bold(),
        "MEDIUM" => format!("Risk: {risk_label}").yellow().bold(),
        _ => format!("Risk: {risk_label}").green().bold(),
    };
    println!("  {risk_colored}");
    println!();
}

fn dashboard_rows(report: &AuditReport) -> Vec<String> {
    let normalized = crate::audit::normalize(report);
    let mut rows = vec![render_dashboard_row(
        "Accessibility",
        normalized.normalized.score,
        &normalized.normalized.grade,
    )];

    if let Some(module) = normalized
        .normalized
        .module_scores
        .iter()
        .find(|m| m.name == "SEO")
    {
        rows.push(render_dashboard_row("SEO", module.score, &module.grade));
    }
    if let Some(module) = normalized
        .normalized
        .module_scores
        .iter()
        .find(|m| m.name == "Performance")
    {
        rows.push(render_dashboard_row(
            "Performance",
            module.score,
            &module.grade,
        ));
    }
    if let Some(module) = normalized
        .normalized
        .module_scores
        .iter()
        .find(|m| m.name == "Security")
    {
        rows.push(render_dashboard_row(
            "Security",
            module.score,
            &module.grade,
        ));
    }
    if let Some(module) = normalized
        .normalized
        .module_scores
        .iter()
        .find(|m| m.name == "Mobile")
    {
        rows.push(render_dashboard_row("Mobile", module.score, &module.grade));
    }
    if let Some(module) = normalized
        .normalized
        .module_scores
        .iter()
        .find(|m| m.name == "UX")
    {
        rows.push(render_dashboard_row("UX", module.score, &module.grade));
    }
    if let Some(module) = normalized
        .normalized
        .module_scores
        .iter()
        .find(|m| m.name == "Journey")
    {
        rows.push(render_dashboard_row("Journey", module.score, &module.grade));
    }

    rows.push(String::new());
    rows.push(format!(
        "  {}",
        render_issue_summary(
            report
                .wcag_results
                .violations
                .iter()
                .filter(|v| v.severity == Severity::Critical)
                .count(),
            report
                .wcag_results
                .violations
                .iter()
                .filter(|v| v.severity == Severity::High)
                .count(),
            report
                .wcag_results
                .violations
                .iter()
                .filter(|v| v.severity == Severity::Medium)
                .count(),
            report
                .wcag_results
                .violations
                .iter()
                .filter(|v| v.severity == Severity::Low)
                .count(),
        )
    ));
    rows
}

fn render_dashboard_row(label: &str, score: u32, grade: &str) -> String {
    let bar = render_score_bar(score);
    let score_text = colorize_score(score, &format!("{score}/100")).bold();
    let grade_text = colorize_grade(grade, grade).bold();
    format!("{label:<16}  {bar}  {score_text}  {grade_text}")
}

fn render_score_bar(score: u32) -> String {
    let slots = 18usize;
    let filled = min((score as usize * slots).div_ceil(100), slots);
    let filled_text = "█".repeat(filled);
    let empty_text = "░".repeat(slots - filled).truecolor(55, 68, 92);
    format!("{}{}", bar_color(score, &filled_text), empty_text)
}

fn render_issue_summary(critical: usize, high: usize, medium: usize, low: usize) -> String {
    let total = critical + high + medium + low;
    format!(
        "{}  {}  {}  {}  {}",
        format!("Issues {total}").white().bold(),
        format!("Critical {critical}").red().bold(),
        format!("High {high}").truecolor(255, 165, 0).bold(),
        format!("Medium {medium}").yellow().bold(),
        format!("Low {low}").dimmed(),
    )
}

fn colorize_score(score: u32, text: &str) -> colored::ColoredString {
    bar_color(score, text)
}

fn colorize_grade(grade: &str, text: &str) -> colored::ColoredString {
    match grade {
        "A+" | "A" => text.green(),
        "B" => text.yellow(),
        "C" => text.truecolor(255, 165, 0),
        "D" | "E" | "F" => text.red(),
        _ => text.white(),
    }
}

fn bar_color(score: u32, text: &str) -> colored::ColoredString {
    if score >= 90 {
        text.green()
    } else if score >= 80 {
        text.truecolor(120, 214, 75)
    } else if score >= 70 {
        text.yellow()
    } else if score >= 50 {
        text.truecolor(255, 165, 0)
    } else {
        text.red()
    }
}

/// Print the violations table
fn print_violations_table(violations: &[Violation]) {
    println!("{}", "Violations".bold().underline());
    println!();

    let mut table = Table::new();
    table
        .load_preset(presets::UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("Rule")
                .add_attribute(Attribute::Bold)
                .fg(Color::Cyan),
            Cell::new("Level")
                .add_attribute(Attribute::Bold)
                .fg(Color::Cyan),
            Cell::new("Severity")
                .add_attribute(Attribute::Bold)
                .fg(Color::Cyan),
            Cell::new("Description")
                .add_attribute(Attribute::Bold)
                .fg(Color::Cyan),
        ]);

    for violation in violations {
        let severity_cell = match violation.severity {
            Severity::Critical => Cell::new("Critical")
                .fg(Color::Red)
                .add_attribute(Attribute::Bold),
            Severity::High => Cell::new("High")
                .fg(Color::Yellow)
                .add_attribute(Attribute::Bold),
            Severity::Medium => Cell::new("Medium").fg(Color::White),
            Severity::Low => Cell::new("Low").fg(Color::DarkGrey),
        };

        let message = if violation.message.len() > 60 {
            let b = violation
                .message
                .char_indices()
                .take_while(|(i, _)| *i <= 57)
                .last()
                .map(|(i, _)| i)
                .unwrap_or(0);
            format!("{}…", &violation.message[..b])
        } else {
            violation.message.clone()
        };

        table.add_row(vec![
            Cell::new(&violation.rule),
            Cell::new(violation.level.to_string()),
            severity_cell,
            Cell::new(message),
        ]);
    }

    println!("{table}");
    println!();

    if !violations.is_empty() {
        print_fix_suggestions(violations);
    }
}

/// Print fix suggestions for violations
fn print_fix_suggestions(violations: &[Violation]) {
    println!("{}", "Suggested Fixes".bold().underline());
    println!();

    // Group by rule
    let mut seen_rules = std::collections::HashSet::new();

    for violation in violations {
        if seen_rules.contains(&violation.rule) {
            continue;
        }
        seen_rules.insert(violation.rule.clone());

        println!(
            "  {} {} - {}",
            "•".cyan(),
            violation.rule.bold(),
            violation.rule_name
        );

        if let Some(fix) = &violation.fix_suggestion {
            println!("    {}", fix.dimmed());
        }

        if let Some(url) = &violation.help_url {
            println!("    {} {}", "Learn more:".dimmed(), url.blue().underline());
        }

        println!();
    }
}

/// Print the report footer
fn print_footer(report: &AuditReport) {
    let normalized = crate::audit::normalize(report);
    let pass_fail = if normalized.normalized.score >= 70
        && normalized.normalized.severity_counts.critical == 0
    {
        "PASS".green().bold()
    } else {
        "NEEDS IMPROVEMENT".red().bold()
    };

    println!("{}", "═".repeat(70).cyan());
    println!("{} {}", "Overall:".bold(), pass_fail);
    println!("{}", "═".repeat(70).cyan());
    println!();
}

/// Print performance analysis section
fn print_performance_section(perf: &PerformanceResults) {
    println!("{}", "Performance".bold().underline());
    println!();
    println!(
        "  {} {}/100 ({})",
        "Score:".bold(),
        perf.score.overall,
        perf.score.grade.label()
    );

    if let Some(ref lcp) = perf.vitals.lcp {
        println!("  {} {:.0}ms ({})", "LCP:".bold(), lcp.value, lcp.rating);
    }
    if let Some(ref fcp) = perf.vitals.fcp {
        println!("  {} {:.0}ms ({})", "FCP:".bold(), fcp.value, fcp.rating);
    }
    if let Some(ref cls) = perf.vitals.cls {
        println!("  {} {:.3} ({})", "CLS:".bold(), cls.value, cls.rating);
    }
    if let Some(ref ttfb) = perf.vitals.ttfb {
        println!("  {} {:.0}ms ({})", "TTFB:".bold(), ttfb.value, ttfb.rating);
    }
    println!();
}

/// Print SEO analysis section
fn print_seo_section(seo: &SeoAnalysis) {
    println!("{}", "SEO".bold().underline());
    println!();
    println!("  {} {}/100", "Score:".bold(), seo.score);

    if let Some(ref title) = seo.meta.title {
        println!("  {} {}", "Title:".bold(), truncate_url(title, 60));
    }
    if seo.headings.h1_count == 0 {
        println!("  {} {}", "H1:".bold(), "Missing!".red());
    } else {
        println!("  {} {} found", "H1:".bold(), seo.headings.h1_count);
    }
    if !seo.meta_issues.is_empty() {
        println!("  {} {} issues", "Meta:".bold(), seo.meta_issues.len());
    }
    if seo.structured_data.has_structured_data {
        println!(
            "  {} {} schemas detected",
            "Schema.org:".bold(),
            seo.structured_data.types.len()
        );
    }
    println!();
}

/// Print security analysis section
fn print_security_section(sec: &SecurityAnalysis) {
    println!("{}", "Security".bold().underline());
    println!();
    println!("  {} {}/100 ({})", "Score:".bold(), sec.score, sec.grade);
    println!("  {} {}/9 present", "Headers:".bold(), sec.headers.count());
    if sec.ssl.https {
        println!("  {} Yes", "HTTPS:".bold());
    } else {
        println!("  {} {}", "HTTPS:".bold(), "No!".red());
    }
    if !sec.issues.is_empty() {
        println!("  {} {} issues", "Issues:".bold(), sec.issues.len());
    }
    println!();
}

/// Print mobile friendliness section
fn print_mobile_section(mobile: &MobileFriendliness) {
    println!("{}", "Mobile Friendliness".bold().underline());
    println!();
    println!("  {} {}/100", "Score:".bold(), mobile.score);
    println!(
        "  {} {}",
        "Viewport:".bold(),
        if mobile.viewport.has_viewport {
            "Configured".green().to_string()
        } else {
            "Missing!".red().to_string()
        }
    );
    if mobile.touch_targets.small_targets > 0 {
        println!(
            "  {} {} too small",
            "Touch Targets:".bold(),
            mobile.touch_targets.small_targets
        );
    }
    if !mobile.issues.is_empty() {
        println!("  {} {} issues", "Issues:".bold(), mobile.issues.len());
    }
    println!();
}

/// Print dark mode analysis section
fn print_dark_mode_section(dm: &crate::dark_mode::DarkModeAnalysis) {
    println!("{}", "Dark Mode".bold().underline());
    println!();

    let support_label = if dm.supported {
        "Supported".green().bold().to_string()
    } else {
        "Not supported".yellow().to_string()
    };
    println!("  {} {}", "Status:".bold(), support_label);
    println!("  {} {}/100", "Score:".bold(), dm.score);

    if !dm.detection_methods.is_empty() {
        println!(
            "  {} {}",
            "Methods:".bold(),
            dm.detection_methods.join(", ").dimmed()
        );
    }
    if dm.css_custom_properties > 0 {
        println!(
            "  {} {}",
            "CSS Custom Properties:".bold(),
            dm.css_custom_properties
        );
    }
    if dm.supported {
        if dm.dark_only_violations > 0 {
            println!(
                "  {} {} (dark mode only)",
                "Contrast issues:".bold(),
                dm.dark_only_violations.to_string().red()
            );
        } else if dm.dark_contrast_violations == 0 {
            println!("  {} {}", "Contrast (dark):".bold(), "No issues".green());
        }
        if dm.light_only_violations > 0 {
            println!(
                "  {} {} (light mode only)",
                "Fixed in dark:".bold(),
                dm.light_only_violations.to_string().cyan()
            );
        }
    }
    if !dm.issues.is_empty() {
        println!();
        for issue in &dm.issues {
            let icon = match issue.severity.as_str() {
                "high" => "✗".red().bold().to_string(),
                "medium" => "⚠".yellow().bold().to_string(),
                _ => "·".dimmed().to_string(),
            };
            println!("  {} {}", icon, issue.description.dimmed());
        }
    }
    println!();
}

/// Print content visibility & trust section
fn print_content_visibility_section(cv: &crate::content_visibility::ContentVisibilityAnalysis) {
    use crate::assessment::AssessmentLevel;

    println!("{}", "Content Visibility & Trust".bold().underline());
    println!();
    println!(
        "  {} signals analyzed, {} with optimization potential",
        cv.signal_count, cv.problem_count
    );
    println!();

    let areas: &[(&str, &[crate::assessment::ContentSignal])] = &[
        ("Organic Visibility", &cv.organic_visibility),
        ("Local Business & Trust Data", &cv.local_business),
        ("E-E-A-T Indicators", &cv.eeat),
        ("Content Depth & Localization", &cv.content_depth),
        ("Topical Authority (heuristic)", &cv.topical_authority),
    ];

    for (area_name, signals) in areas {
        let visible: Vec<_> = signals
            .iter()
            .filter(|s| s.level != AssessmentLevel::NotTestable)
            .collect();
        if visible.is_empty() {
            continue;
        }

        println!("  {}", area_name.bold());

        let mut table = Table::new();
        table
            .load_preset(presets::UTF8_BORDERS_ONLY)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec![
                Cell::new("Level").add_attribute(Attribute::Bold),
                Cell::new("Conf").add_attribute(Attribute::Bold),
                Cell::new("Signal").add_attribute(Attribute::Bold),
            ]);

        for signal in visible {
            let level_cell = match signal.level {
                AssessmentLevel::Pass => Cell::new("PASS").fg(Color::Green),
                AssessmentLevel::Positive => Cell::new("POSITIVE").fg(Color::Cyan),
                AssessmentLevel::Warning => Cell::new("WARN").fg(Color::Yellow),
                AssessmentLevel::Violation => Cell::new("VIOLATION").fg(Color::Red),
                AssessmentLevel::NotTestable => Cell::new("N/T").fg(Color::DarkGrey),
            };
            let conf_cell = match signal.confidence {
                crate::assessment::EvidenceConfidence::High => Cell::new("●"),
                crate::assessment::EvidenceConfidence::Medium => Cell::new("◐"),
                crate::assessment::EvidenceConfidence::Low => Cell::new("○"),
            };
            let detail_truncated = if signal.detail.chars().count() > 60 {
                let end = signal
                    .detail
                    .char_indices()
                    .nth(60)
                    .map(|(i, _)| i)
                    .unwrap_or(signal.detail.len());
                format!("{}…", &signal.detail[..end])
            } else {
                signal.detail.clone()
            };
            let text = format!("{} — {}", signal.title, detail_truncated);
            table.add_row(vec![level_cell, conf_cell, Cell::new(text)]);
        }

        println!("{table}");
        println!();
    }
}

/// Print budget violations section
fn print_budget_violations_section(violations: &[crate::audit::BudgetViolation]) {
    println!("{}", "Performance Budgets".bold().underline());
    println!();

    for v in violations {
        let (icon, label) = match v.severity {
            BudgetSeverity::Error => ("✗".red().bold(), v.severity.label().red().bold()),
            BudgetSeverity::Warning => ("⚠".yellow().bold(), v.severity.label().yellow().bold()),
        };
        println!(
            "  {} {} {}: {} (Budget: {}, +{:.0}%)",
            icon,
            label,
            v.metric.bold(),
            v.actual_label,
            v.budget_label,
            v.exceeded_by_pct,
        );
    }
    println!();
}

/// Print batch results as a table
pub fn print_batch_table(batch_report: &BatchReport, level: WcagLevel) {
    print!("{}", format_batch_table(batch_report, level, true));
}

/// Format batch results as a table string for terminal or file output
pub fn format_batch_table(batch_report: &BatchReport, level: WcagLevel, use_color: bool) -> String {
    set_override(use_color);
    let presentation = crate::output::builder::build_batch_presentation(batch_report);

    let mut output = String::new();
    output.push('\n');
    output.push_str(&format!(
        "{} WCAG {} Batch Audit Results\n\n",
        "═══".cyan(),
        level
    ));

    // Summary block
    output.push_str(&format!(
        "  {}  {}   {}  {:.1}/100   {}  {}/100   {}  {}   {}  {}ms\n\n",
        "URLs:".bold(),
        batch_report.summary.total_urls,
        "Average accessibility:".bold(),
        presentation.portfolio_summary.average_score,
        "Average overall:".bold(),
        presentation.portfolio_summary.average_overall_score,
        "Violations:".bold(),
        batch_report.summary.total_violations,
        "Duration:".bold(),
        batch_report.total_duration_ms,
    ));

    // Per-URL table
    let mut table = Table::new();
    table
        .load_preset(presets::UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("URL")
                .add_attribute(Attribute::Bold)
                .fg(Color::Cyan),
            Cell::new("Score")
                .add_attribute(Attribute::Bold)
                .fg(Color::Cyan),
            Cell::new("Overall")
                .add_attribute(Attribute::Bold)
                .fg(Color::Cyan),
            Cell::new("Violations")
                .add_attribute(Attribute::Bold)
                .fg(Color::Cyan),
            Cell::new("Status")
                .add_attribute(Attribute::Bold)
                .fg(Color::Cyan),
        ]);

    for report in &presentation.url_ranking {
        let (status_cell, score_color) = if report.passed {
            (
                Cell::new("PASS")
                    .fg(Color::Green)
                    .add_attribute(Attribute::Bold),
                Color::Green,
            )
        } else {
            (
                Cell::new("FAIL")
                    .fg(Color::Red)
                    .add_attribute(Attribute::Bold),
                if report.score >= 70.0 {
                    Color::Yellow
                } else {
                    Color::Red
                },
            )
        };

        table.add_row(vec![
            Cell::new(truncate_url(&report.url, 55)),
            Cell::new(format!("{:.0}/100", report.score)).fg(score_color),
            Cell::new(format!("{}/100", report.overall_score)),
            Cell::new(report.total_violations.to_string()),
            status_cell,
        ]);
    }

    output.push_str(&table.to_string());
    output.push('\n');

    if !batch_report.errors.is_empty() {
        output.push('\n');
        output.push_str(&format!("{}\n", "Errors:".red().bold()));
        for err in &batch_report.errors {
            output.push_str(&format!("  {} {}: {}\n", "✗".red(), err.url, err.error));
        }
    }

    set_override(false);
    output
}

/// Format violations as a simple list (for non-interactive output)
pub fn format_violations_list(violations: &[Violation]) -> String {
    let mut output = String::new();

    for (i, violation) in violations.iter().enumerate() {
        output.push_str(&format!(
            "{}. [{}] {} - {}\n",
            i + 1,
            violation.rule,
            violation.severity,
            violation.message
        ));

        if let Some(fix) = &violation.fix_suggestion {
            output.push_str(&format!("   Fix: {}\n", fix));
        }

        output.push('\n');
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_violations_list() {
        let violations = vec![Violation::new(
            "1.1.1",
            "Non-text Content",
            WcagLevel::A,
            Severity::High,
            "Image missing alt text",
            "node-1",
        )
        .with_fix("Add alt attribute")];

        let output = format_violations_list(&violations);
        assert!(output.contains("1.1.1"));
        assert!(output.contains("Image missing"));
        assert!(output.contains("Add alt attribute"));
    }

    #[test]
    fn test_render_dashboard_row_contains_score_and_grade() {
        let row = render_dashboard_row("Security", 98, "A+");
        assert!(row.contains("Security"));
        assert!(row.contains("98/100"));
        assert!(row.contains("A+"));
    }

    #[test]
    fn test_format_batch_table_contains_summary_and_urls() {
        let report = AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            crate::wcag::WcagResults::new(),
            42,
        );
        let batch = BatchReport::from_reports(vec![report], vec![], 1500);
        let text = format_batch_table(&batch, WcagLevel::AA, false);

        assert!(text.contains("WCAG AA Batch Audit Results"));
        assert!(text.contains("https://example.com"));
        assert!(text.contains("Violations:"));
    }
}
