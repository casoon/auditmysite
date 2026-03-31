//! CLI Output Formatter
//!
//! Generates human-readable terminal output for interactive use and batch tables.

use colored::Colorize;
use prettytable::{format, Cell, Row, Table};
use std::cmp::min;

use crate::audit::{AuditReport, BatchReport, PerformanceResults};
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
    println!(
        "{}",
        format!(
            "  WCAG {}  Nodes {}  Duration {:.1}s  Overall {}  Certificate {}",
            level,
            report.nodes_analyzed,
            report.duration_ms as f64 / 1000.0,
            report.overall_score(),
            report.certificate
        )
        .dimmed()
    );
    println!();
}

fn dashboard_rows(report: &AuditReport) -> Vec<String> {
    let mut rows = vec![render_dashboard_row(
        "Accessibility",
        report.score.round() as u32,
        &report.grade,
    )];

    if let Some(ref seo) = report.seo {
        rows.push(render_dashboard_row(
            "SEO",
            seo.score,
            score_grade(seo.score),
        ));
    }
    if let Some(ref perf) = report.performance {
        rows.push(render_dashboard_row(
            "Performance",
            perf.score.overall,
            score_grade(perf.score.overall),
        ));
    }
    if let Some(ref sec) = report.security {
        rows.push(render_dashboard_row("Security", sec.score, &sec.grade));
    }
    if let Some(ref mobile) = report.mobile {
        rows.push(render_dashboard_row(
            "Mobile",
            mobile.score,
            score_grade(mobile.score),
        ));
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
    let filled = min((score as usize * slots + 99) / 100, slots);
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

fn colorize_score<'a>(score: u32, text: &'a str) -> colored::ColoredString {
    bar_color(score, text)
}

fn colorize_grade<'a>(grade: &str, text: &'a str) -> colored::ColoredString {
    match grade {
        "A+" | "A" => text.green(),
        "B" => text.yellow(),
        "C" => text.truecolor(255, 165, 0),
        "D" | "E" | "F" => text.red(),
        _ => text.white(),
    }
}

fn bar_color<'a>(score: u32, text: &'a str) -> colored::ColoredString {
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

fn score_grade(score: u32) -> &'static str {
    match score {
        97..=100 => "A+",
        90..=96 => "A",
        80..=89 => "B",
        70..=79 => "C",
        60..=69 => "D",
        _ => "F",
    }
}

/// Print the violations table
fn print_violations_table(violations: &[Violation]) {
    println!("{}", "Violations".bold().underline());
    println!();

    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_BOX_CHARS);

    // Header row
    table.add_row(Row::new(vec![
        Cell::new("Rule").style_spec("bFc"),
        Cell::new("Level").style_spec("bFc"),
        Cell::new("Severity").style_spec("bFc"),
        Cell::new("Message").style_spec("bFc"),
    ]));

    // Add violations
    for violation in violations {
        let severity_cell = match violation.severity {
            Severity::Critical => Cell::new("Kritisch").style_spec("Fr"),
            Severity::High => Cell::new("Hoch").style_spec("Fy"),
            Severity::Medium => Cell::new("Mittel").style_spec("Fw"),
            Severity::Low => Cell::new("Niedrig").style_spec("Fd"),
        };

        let level_cell = match violation.level {
            WcagLevel::A => Cell::new("A"),
            WcagLevel::AA => Cell::new("AA"),
            WcagLevel::AAA => Cell::new("AAA"),
        };

        // Truncate message if too long
        let message = if violation.message.len() > 50 {
            format!("{}...", &violation.message[..47])
        } else {
            violation.message.clone()
        };

        table.add_row(Row::new(vec![
            Cell::new(&violation.rule),
            level_cell,
            severity_cell,
            Cell::new(&message),
        ]));
    }

    table.printstd();
    println!();

    // Print detailed fix suggestions
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
    let pass_fail = if report.score >= 70.0
        && report
            .wcag_results
            .violations
            .iter()
            .all(|v| v.severity != Severity::Critical)
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

/// Print batch results as a table
pub fn print_batch_table(batch_report: &BatchReport, level: WcagLevel) {
    println!();
    println!("{} WCAG {} Batch Audit Results", "═══".cyan(), level);
    println!();

    // Summary
    println!(
        "  {} {} URLs audited",
        "Total:".bold(),
        batch_report.summary.total_urls
    );
    println!(
        "  {} {} passed, {} failed",
        "Status:".bold(),
        batch_report.summary.passed.to_string().green(),
        batch_report.summary.failed.to_string().red()
    );
    println!(
        "  {} {:.1}",
        "Avg Score:".bold(),
        batch_report.summary.average_score
    );
    println!(
        "  {} {}",
        "Total Violations:".bold(),
        batch_report.summary.total_violations
    );
    println!(
        "  {} {}ms",
        "Duration:".bold(),
        batch_report.total_duration_ms
    );
    println!();

    // Individual results
    println!("{}", "─".repeat(80));
    println!(
        "{:<50} {:>8} {:>10} {:>8}",
        "URL".bold(),
        "Score".bold(),
        "Violations".bold(),
        "Status".bold()
    );
    println!("{}", "─".repeat(80));

    for report in &batch_report.reports {
        let status = if report.passed() {
            "PASS".green()
        } else {
            "FAIL".red()
        };

        let score_color = if report.score >= 90.0 {
            format!("{:.1}", report.score).green()
        } else if report.score >= 70.0 {
            format!("{:.1}", report.score).yellow()
        } else {
            format!("{:.1}", report.score).red()
        };

        println!(
            "{:<50} {:>8} {:>10} {:>8}",
            truncate_url(&report.url, 48),
            score_color,
            report.violation_count(),
            status
        );
    }

    // Show errors if any
    if !batch_report.errors.is_empty() {
        println!();
        println!("{}", "Errors:".red().bold());
        for err in &batch_report.errors {
            println!("  {} {}: {}", "✗".red(), err.url, err.error);
        }
    }

    println!("{}", "─".repeat(80));
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
    fn test_score_grade_mapping() {
        assert_eq!(score_grade(98), "A+");
        assert_eq!(score_grade(92), "A");
        assert_eq!(score_grade(84), "B");
        assert_eq!(score_grade(74), "C");
        assert_eq!(score_grade(61), "D");
        assert_eq!(score_grade(49), "F");
    }
}
