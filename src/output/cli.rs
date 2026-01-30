//! CLI Table Output Formatter
//!
//! Generates human-readable terminal output with colored tables.

use colored::Colorize;
use prettytable::{format, Cell, Row, Table};

use crate::audit::AuditReport;
use crate::cli::WcagLevel;
use crate::wcag::{Severity, Violation};

/// Format and print the audit report to the terminal
pub fn print_report(report: &AuditReport, level: WcagLevel) {
    println!();
    print_header(report);
    print_summary(report, level);

    if !report.wcag_results.violations.is_empty() {
        print_violations_table(&report.wcag_results.violations);
    }

    print_footer(report);
}

/// Print the report header
fn print_header(report: &AuditReport) {
    println!("{}", "═".repeat(70).cyan());
    println!(
        "{} {}",
        "WCAG Accessibility Report".cyan().bold(),
        format!("({})", report.timestamp.format("%Y-%m-%d %H:%M:%S UTC")).dimmed()
    );
    println!("{}", "═".repeat(70).cyan());
    println!();
    println!("{} {}", "URL:".bold(), report.url);
    println!();
}

/// Print the summary section
fn print_summary(report: &AuditReport, level: WcagLevel) {
    let score = report.score;
    let score_color = if score >= 90.0 {
        format!("{:.1}", score).green()
    } else if score >= 70.0 {
        format!("{:.1}", score).yellow()
    } else if score >= 50.0 {
        format!("{:.1}", score).truecolor(255, 165, 0) // Orange
    } else {
        format!("{:.1}", score).red()
    };

    let grade_colored = match report.grade.as_str() {
        "A" => report.grade.green().bold(),
        "B" => report.grade.yellow().bold(),
        "C" => report.grade.truecolor(255, 165, 0).bold(),
        "D" | "F" => report.grade.red().bold(),
        _ => report.grade.white().bold(),
    };

    // Certificate color coding
    let certificate_colored = match report.certificate.as_str() {
        "PLATINUM" => report.certificate.truecolor(229, 228, 226).bold(), // Platinum color
        "GOLD" => report.certificate.truecolor(255, 215, 0).bold(),       // Gold
        "SILVER" => report.certificate.truecolor(192, 192, 192).bold(),   // Silver
        "BRONZE" => report.certificate.truecolor(205, 127, 50).bold(),    // Bronze
        _ => report.certificate.red().bold(),                             // NEEDS_IMPROVEMENT
    };

    println!("{}", "Summary".bold().underline());
    println!();
    println!(
        "  {} {} / 100  (Grade: {})",
        "Score:".bold(),
        score_color.bold(),
        grade_colored
    );
    println!("  {} {}", "Certificate:".bold(), certificate_colored);
    println!("  {} {}", "WCAG Level:".bold(), level.to_string().cyan());
    println!("  {} {}", "Nodes Analyzed:".bold(), report.nodes_analyzed);
    println!("  {} {}ms", "Duration:".bold(), report.duration_ms);
    println!();

    // Violation counts by severity
    let violations = &report.wcag_results.violations;
    let critical = violations
        .iter()
        .filter(|v| v.severity == Severity::Critical)
        .count();
    let serious = violations
        .iter()
        .filter(|v| v.severity == Severity::Serious)
        .count();
    let moderate = violations
        .iter()
        .filter(|v| v.severity == Severity::Moderate)
        .count();
    let minor = violations
        .iter()
        .filter(|v| v.severity == Severity::Minor)
        .count();

    println!("{}", "Violations by Severity".bold().underline());
    println!();
    println!(
        "  {} {}",
        "Critical:".red().bold(),
        if critical > 0 {
            critical.to_string().red().bold().to_string()
        } else {
            "0".green().to_string()
        }
    );
    println!(
        "  {} {}",
        "Serious: ".truecolor(255, 165, 0).bold(),
        if serious > 0 {
            serious.to_string().truecolor(255, 165, 0).to_string()
        } else {
            "0".green().to_string()
        }
    );
    println!(
        "  {} {}",
        "Moderate:".yellow().bold(),
        if moderate > 0 {
            moderate.to_string().yellow().to_string()
        } else {
            "0".green().to_string()
        }
    );
    println!("  {} {}", "Minor:   ".dimmed().bold(), minor);
    println!();
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
            Severity::Critical => Cell::new("CRITICAL").style_spec("Fr"),
            Severity::Serious => Cell::new("Serious").style_spec("Fy"),
            Severity::Moderate => Cell::new("Moderate").style_spec("Fw"),
            Severity::Minor => Cell::new("Minor").style_spec("Fd"),
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

/// Get letter grade from score
fn get_grade(score: u8) -> &'static str {
    match score {
        95..=100 => "A+",
        90..=94 => "A",
        85..=89 => "B+",
        80..=84 => "B",
        75..=79 => "C+",
        70..=74 => "C",
        60..=69 => "D",
        _ => "F",
    }
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
    fn test_get_grade() {
        assert_eq!(get_grade(100), "A+");
        assert_eq!(get_grade(95), "A+");
        assert_eq!(get_grade(90), "A");
        assert_eq!(get_grade(85), "B+");
        assert_eq!(get_grade(70), "C");
        assert_eq!(get_grade(50), "F");
    }

    #[test]
    fn test_format_violations_list() {
        let violations = vec![Violation::new(
            "1.1.1",
            "Non-text Content",
            WcagLevel::A,
            Severity::Serious,
            "Image missing alt text",
            "node-1",
        )
        .with_fix("Add alt attribute")];

        let output = format_violations_list(&violations);
        assert!(output.contains("1.1.1"));
        assert!(output.contains("Image missing"));
        assert!(output.contains("Add alt attribute"));
    }
}
