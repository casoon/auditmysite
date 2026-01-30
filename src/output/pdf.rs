//! PDF Report Generator using renderreport/Typst
//!
//! Generates professional PDF reports for WCAG accessibility audits.

use renderreport::prelude::*;
use renderreport::Engine;

use crate::audit::{AuditReport, BatchReport};

/// Generate PDF report for a single audit
pub fn generate_pdf(report: &AuditReport) -> anyhow::Result<Vec<u8>> {
    let engine = Engine::new()?;

    // Build report using builder pattern
    let pdf_report = engine
        .report("wcag-audit")
        .title("WCAG 2.1 Accessibility Audit Report")
        .subtitle(&report.url)
        .metadata(
            "date",
            report.timestamp.format("%Y-%m-%d %H:%M UTC").to_string(),
        )
        .metadata("score", format!("{:.1}/100", report.score))
        .metadata("grade", &report.grade)
        .metadata("certificate", &report.certificate)
        // Executive Summary Section
        .add_component(Section::new("Executive Summary").with_level(1))
        .add_component(
            ScoreCard::new("Accessibility Score", report.score as u32)
                .with_description(format!(
                    "Grade: {} | Certificate: {} | {} violations found",
                    report.grade,
                    report.certificate,
                    report.wcag_results.violations.len()
                ))
                .with_thresholds(70, 50),
        )
        .add_component(
            SummaryBox::new("Audit Statistics")
                .add_item("Total Violations", report.statistics.total.to_string())
                .add_item("Errors", report.statistics.errors.to_string())
                .add_item("Warnings", report.statistics.warnings.to_string())
                .add_item("Notices", report.statistics.notices.to_string())
                .add_item("Nodes Analyzed", report.nodes_analyzed.to_string())
                .add_item("Duration", format!("{}ms", report.duration_ms)),
        );

    // Add violations section if any exist
    let pdf_report = if !report.wcag_results.violations.is_empty() {
        let mut builder = pdf_report.add_component(Section::new("Violations Found").with_level(1));

        // Add each violation as a Finding
        for violation in &report.wcag_results.violations {
            let severity = match violation.severity {
                crate::wcag::Severity::Critical => Severity::Critical,
                crate::wcag::Severity::Serious => Severity::High,
                crate::wcag::Severity::Moderate => Severity::Medium,
                crate::wcag::Severity::Minor => Severity::Low,
            };

            let mut finding = Finding::new(
                format!("{} - {}", violation.rule, violation.rule_name),
                severity,
                &violation.message,
            );

            if let Some(ref fix) = violation.fix_suggestion {
                finding = finding.with_recommendation(fix);
            }

            finding = finding.with_affected(&violation.node_id);

            builder = builder.add_component(finding);
        }

        builder
    } else {
        pdf_report.add_component(
            Callout::success(
                "No violations found! This page passed all WCAG 2.1 accessibility checks.",
            )
            .with_title("Excellent Accessibility"),
        )
    };

    // Add recommendations based on score
    let pdf_report = pdf_report.add_component(Section::new("Recommendations").with_level(1));

    let pdf_report = if report.score < 70.0 {
        pdf_report.add_component(
            Callout::warning(format!(
                "This page scored {:.1}/100 (Grade: {}), which indicates significant accessibility barriers. \
                Priority should be given to fixing errors first.",
                report.score, report.grade
            ))
            .with_title("Action Required")
        )
    } else if report.score < 90.0 {
        pdf_report.add_component(
            Callout::info(format!(
                "This page scored {:.1}/100 (Grade: {}). Consider addressing the remaining issues.",
                report.score, report.grade
            ))
            .with_title("Good Progress"),
        )
    } else {
        pdf_report.add_component(
            Callout::success(format!(
                "This page scored {:.1}/100 (Grade: {}), demonstrating excellent accessibility!",
                report.score, report.grade
            ))
            .with_title("Excellent Work"),
        )
    };

    // Build and render to PDF
    let built_report = pdf_report.build();
    let pdf_bytes = engine.render_pdf(&built_report)?;

    Ok(pdf_bytes)
}

/// Generate PDF report for batch audits
pub fn generate_batch_pdf(batch: &BatchReport) -> anyhow::Result<Vec<u8>> {
    let engine = Engine::new()?;

    let success_rate = if batch.summary.total_urls > 0 {
        (batch.summary.passed as f64 / batch.summary.total_urls as f64) * 100.0
    } else {
        0.0
    };

    let mut builder = engine
        .report("wcag-batch-audit")
        .title("WCAG 2.1 Batch Audit Report")
        .subtitle(format!("{} URLs Audited", batch.summary.total_urls))
        .metadata(
            "date",
            chrono::Utc::now().format("%Y-%m-%d %H:%M UTC").to_string(),
        )
        .metadata("total_urls", batch.summary.total_urls.to_string())
        .metadata("success_rate", format!("{:.1}%", success_rate))
        // Batch Summary
        .add_component(Section::new("Batch Summary").with_level(1))
        .add_component(
            SummaryBox::new("Overall Statistics")
                .add_item("Total URLs", batch.summary.total_urls.to_string())
                .add_item("Passed", batch.summary.passed.to_string())
                .add_item("Failed", batch.summary.failed.to_string())
                .add_item("Success Rate", format!("{:.1}%", success_rate))
                .add_item(
                    "Average Score",
                    format!("{:.1}/100", batch.summary.average_score),
                )
                .add_item(
                    "Total Violations",
                    batch.summary.total_violations.to_string(),
                )
                .add_item("Duration", format!("{}ms", batch.total_duration_ms)),
        );

    // Add individual results
    builder = builder.add_component(Section::new("Individual Results").with_level(1));

    for (idx, report) in batch.reports.iter().enumerate() {
        builder = builder
            .add_component(
                Section::new(format!("{}. {}", idx + 1, truncate_url(&report.url, 60)))
                    .with_level(2),
            )
            .add_component(
                ScoreCard::new("Score", report.score as u32)
                    .with_description(format!(
                        "Grade: {} | {} violations",
                        report.grade,
                        report.wcag_results.violations.len()
                    ))
                    .with_thresholds(70, 50),
            );

        // Show top 3 violations for each URL
        let top_violations: Vec<_> = report.wcag_results.violations.iter().take(3).collect();

        for violation in top_violations {
            let severity = match violation.severity {
                crate::wcag::Severity::Critical => Severity::Critical,
                crate::wcag::Severity::Serious => Severity::High,
                crate::wcag::Severity::Moderate => Severity::Medium,
                crate::wcag::Severity::Minor => Severity::Low,
            };

            builder = builder.add_component(Finding::new(
                format!("{} - {}", violation.rule, violation.rule_name),
                severity,
                &violation.message,
            ));
        }

        if report.wcag_results.violations.len() > 3 {
            builder = builder.add_component(Callout::info(format!(
                "...and {} more violations",
                report.wcag_results.violations.len() - 3
            )));
        }
    }

    // Build and render
    let built_report = builder.build();
    let pdf_bytes = engine.render_pdf(&built_report)?;

    Ok(pdf_bytes)
}

/// Truncate URL to max length with ellipsis
fn truncate_url(url: &str, max_len: usize) -> String {
    if url.len() <= max_len {
        url.to_string()
    } else {
        format!("{}...", &url[..max_len - 3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_url() {
        assert_eq!(
            truncate_url("https://example.com/very/long/path/that/exceeds/limit", 30),
            "https://example.com/very/lo..."
        );

        assert_eq!(
            truncate_url("https://example.com", 30),
            "https://example.com"
        );
    }
}
