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
    table
        .load_preset(presets::UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("Regel").add_attribute(Attribute::Bold).fg(Color::Cyan),
            Cell::new("Level").add_attribute(Attribute::Bold).fg(Color::Cyan),
            Cell::new("Severity").add_attribute(Attribute::Bold).fg(Color::Cyan),
            Cell::new("Beschreibung").add_attribute(Attribute::Bold).fg(Color::Cyan),
        ]);

    for violation in violations {
        let severity_cell = match violation.severity {
            Severity::Critical => Cell::new("Kritisch").fg(Color::Red).add_attribute(Attribute::Bold),
            Severity::High => Cell::new("Hoch").fg(Color::Yellow).add_attribute(Attribute::Bold),
            Severity::Medium => Cell::new("Mittel").fg(Color::White),
            Severity::Low => Cell::new("Niedrig").fg(Color::DarkGrey),
        };

        let message = if violation.message.len() > 60 {
            format!("{}…", &violation.message[..57])
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

/// Print dark mode analysis section
fn print_dark_mode_section(dm: &crate::dark_mode::DarkModeAnalysis) {
    println!("{}", "Dark Mode".bold().underline());
    println!();

    let support_label = if dm.supported {
        "Unterstützt".green().bold().to_string()
    } else {
        "Nicht unterstützt".yellow().to_string()
    };
    println!("  {} {}", "Status:".bold(), support_label);
    println!("  {} {}/100", "Score:".bold(), dm.score);

    if !dm.detection_methods.is_empty() {
        println!("  {} {}", "Methoden:".bold(), dm.detection_methods.join(", ").dimmed());
    }
    if dm.css_custom_properties > 0 {
        println!("  {} {}", "CSS Custom Properties:".bold(), dm.css_custom_properties);
    }
    if dm.supported {
        if dm.dark_only_violations > 0 {
            println!(
                "  {} {} (nur im Dark Mode)",
                "Kontrast-Probleme:".bold(),
                dm.dark_only_violations.to_string().red()
            );
        } else if dm.dark_contrast_violations == 0 {
            println!("  {} {}", "Kontrast Dark Mode:".bold(), "Keine Probleme".green());
        }
        if dm.light_only_violations > 0 {
            println!(
                "  {} {} (nur im Light Mode)",
                "Behoben in Dark:".bold(),
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

    let mut output = String::new();
    output.push('\n');
    output.push_str(&format!(
        "{} WCAG {} Batch Audit Results\n\n",
        "═══".cyan(),
        level
    ));

    // Summary block
    output.push_str(&format!(
        "  {}  {}   {}  {:.1}   {}  {}   {}  {}ms\n\n",
        "URLs:".bold(),
        batch_report.summary.total_urls,
        "Passed:".bold(),
        batch_report.summary.average_score,
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
            Cell::new("URL").add_attribute(Attribute::Bold).fg(Color::Cyan),
            Cell::new("Score").add_attribute(Attribute::Bold).fg(Color::Cyan),
            Cell::new("Violations").add_attribute(Attribute::Bold).fg(Color::Cyan),
            Cell::new("Status").add_attribute(Attribute::Bold).fg(Color::Cyan),
        ]);

    for report in &batch_report.reports {
        let (status_cell, score_color) = if report.passed() {
            (
                Cell::new("PASS").fg(Color::Green).add_attribute(Attribute::Bold),
                Color::Green,
            )
        } else {
            (
                Cell::new("FAIL").fg(Color::Red).add_attribute(Attribute::Bold),
                if report.score >= 70.0 { Color::Yellow } else { Color::Red },
            )
        };

        table.add_row(vec![
            Cell::new(truncate_url(&report.url, 55)),
            Cell::new(format!("{:.1}", report.score)).fg(score_color),
            Cell::new(report.violation_count().to_string()),
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
    fn test_score_grade_mapping() {
        assert_eq!(score_grade(98), "A+");
        assert_eq!(score_grade(92), "A");
        assert_eq!(score_grade(84), "B");
        assert_eq!(score_grade(74), "C");
        assert_eq!(score_grade(61), "D");
        assert_eq!(score_grade(49), "F");
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
