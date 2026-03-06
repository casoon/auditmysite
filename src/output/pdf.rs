//! PDF Report Generator using renderreport/Typst
//!
//! Generates professional PDF reports for WCAG accessibility audits.

use renderreport::prelude::*;
use renderreport::components::advanced::{Divider, KeyValueList, List};
use renderreport::Engine;

use crate::audit::{AuditReport, BatchReport};

/// Helper to map our severity to renderreport severity
fn map_severity(severity: &crate::wcag::Severity) -> Severity {
    match severity {
        crate::wcag::Severity::Critical => Severity::Critical,
        crate::wcag::Severity::Serious => Severity::High,
        crate::wcag::Severity::Moderate => Severity::Medium,
        crate::wcag::Severity::Minor => Severity::Low,
    }
}

/// Helper for yes/no display
fn yes_no(val: bool) -> &'static str {
    if val { "Yes" } else { "No" }
}

/// Generate PDF report for a single audit
pub fn generate_pdf(report: &AuditReport) -> anyhow::Result<Vec<u8>> {
    let engine = Engine::new()?;

    // Build report using builder pattern
    let mut builder = engine
        .report("wcag-audit")
        .title("Web Audit Report")
        .subtitle(&report.url)
        .metadata(
            "date",
            &report.timestamp.format("%Y-%m-%d %H:%M UTC").to_string(),
        )
        .metadata("score", &format!("{:.1}/100", report.score))
        .metadata("grade", &report.grade)
        .metadata("certificate", &report.certificate)
        // Executive Summary Section
        .add_component(Section::new("Executive Summary").with_level(1))
        .add_component(
            ScoreCard::new("Accessibility Score", report.score as u32)
                .with_description(&format!(
                    "Grade: {} | Certificate: {} | {} violations found",
                    report.grade,
                    report.certificate,
                    report.wcag_results.violations.len()
                ))
                .with_thresholds(70, 50),
        )
        .add_component(
            SummaryBox::new("Audit Statistics")
                .add_item("Total Violations", &report.statistics.total.to_string())
                .add_item("Errors", &report.statistics.errors.to_string())
                .add_item("Warnings", &report.statistics.warnings.to_string())
                .add_item("Notices", &report.statistics.notices.to_string())
                .add_item("Nodes Analyzed", &report.nodes_analyzed.to_string())
                .add_item("Duration", &format!("{}ms", report.duration_ms)),
        );

    // Add violations section if any exist
    builder = if !report.wcag_results.violations.is_empty() {
        let mut b = builder.add_component(Section::new("WCAG Violations").with_level(1));

        for violation in &report.wcag_results.violations {
            let mut finding = Finding::new(
                &format!("{} - {}", violation.rule, violation.rule_name),
                map_severity(&violation.severity),
                &violation.message,
            );

            if let Some(ref fix) = violation.fix_suggestion {
                finding = finding.with_recommendation(fix);
            }

            finding = finding.with_affected(&violation.node_id);

            if let Some(ref role) = violation.role {
                finding = finding.with_category(role);
            }

            b = b.add_component(finding);
        }

        b
    } else {
        builder.add_component(
            Callout::success(
                "No violations found! This page passed all WCAG 2.1 accessibility checks.",
            )
            .with_title("Excellent Accessibility"),
        )
    };

    // Recommendations based on score
    builder = builder.add_component(Section::new("Recommendations").with_level(1));

    builder = if report.score < 70.0 {
        builder.add_component(
            Callout::warning(&format!(
                "This page scored {:.1}/100 (Grade: {}), which indicates significant accessibility barriers. \
                Priority should be given to fixing critical and serious violations first.",
                report.score, report.grade
            ))
            .with_title("Action Required"),
        )
    } else if report.score < 90.0 {
        builder.add_component(
            Callout::info(&format!(
                "This page scored {:.1}/100 (Grade: {}). Consider addressing the remaining issues to improve accessibility.",
                report.score, report.grade
            ))
            .with_title("Good Progress"),
        )
    } else {
        builder.add_component(
            Callout::success(&format!(
                "This page scored {:.1}/100 (Grade: {}), demonstrating excellent accessibility!",
                report.score, report.grade
            ))
            .with_title("Excellent Work"),
        )
    };

    // ── Performance Section ──────────────────────────────────────────────
    if let Some(ref perf) = report.performance {
        builder = builder
            .add_component(Divider::new())
            .add_component(Section::new("Performance").with_level(1))
            .add_component(
                ScoreCard::new("Performance Score", perf.score.overall)
                    .with_description(&format!("Grade: {}", perf.score.grade.label()))
                    .with_thresholds(75, 50),
            );

        // Web Vitals
        let mut vitals_kv = KeyValueList::new().with_title("Core Web Vitals");

        if let Some(ref lcp) = perf.vitals.lcp {
            vitals_kv = vitals_kv.add(
                "Largest Contentful Paint (LCP)",
                &format!("{:.0}ms — {}", lcp.value, lcp.rating),
            );
        }
        if let Some(ref fcp) = perf.vitals.fcp {
            vitals_kv = vitals_kv.add(
                "First Contentful Paint (FCP)",
                &format!("{:.0}ms — {}", fcp.value, fcp.rating),
            );
        }
        if let Some(ref cls) = perf.vitals.cls {
            vitals_kv = vitals_kv.add(
                "Cumulative Layout Shift (CLS)",
                &format!("{:.3} — {}", cls.value, cls.rating),
            );
        }
        if let Some(ref ttfb) = perf.vitals.ttfb {
            vitals_kv = vitals_kv.add(
                "Time to First Byte (TTFB)",
                &format!("{:.0}ms — {}", ttfb.value, ttfb.rating),
            );
        }
        if let Some(ref inp) = perf.vitals.inp {
            vitals_kv = vitals_kv.add(
                "Interaction to Next Paint (INP)",
                &format!("{:.0}ms — {}", inp.value, inp.rating),
            );
        }
        if let Some(ref tbt) = perf.vitals.tbt {
            vitals_kv = vitals_kv.add(
                "Total Blocking Time (TBT)",
                &format!("{:.0}ms — {}", tbt.value, tbt.rating),
            );
        }

        builder = builder.add_component(vitals_kv);

        // Additional metrics
        let mut extra = SummaryBox::new("Additional Metrics");
        if let Some(dom_nodes) = perf.vitals.dom_nodes {
            extra = extra.add_item("DOM Nodes", &dom_nodes.to_string());
        }
        if let Some(heap) = perf.vitals.js_heap_size {
            extra = extra.add_item("JS Heap", &format!("{:.1} MB", heap as f64 / 1_048_576.0));
        }
        if let Some(load) = perf.vitals.load_time {
            extra = extra.add_item("Page Load", &format!("{:.0}ms", load));
        }
        if let Some(dcl) = perf.vitals.dom_content_loaded {
            extra = extra.add_item("DOM Content Loaded", &format!("{:.0}ms", dcl));
        }

        builder = builder.add_component(extra);
    }

    // ── SEO Section ──────────────────────────────────────────────────────
    if let Some(ref seo) = report.seo {
        builder = builder
            .add_component(Divider::new())
            .add_component(Section::new("SEO Analysis").with_level(1))
            .add_component(
                ScoreCard::new("SEO Score", seo.score)
                    .with_thresholds(80, 50),
            );

        // Meta Tags
        let mut meta_kv = KeyValueList::new().with_title("Meta Tags");
        if let Some(ref title) = seo.meta.title {
            meta_kv = meta_kv.add("Title", title);
        }
        if let Some(ref desc) = seo.meta.description {
            meta_kv = meta_kv.add("Description", desc);
        }
        if let Some(ref viewport) = seo.meta.viewport {
            meta_kv = meta_kv.add("Viewport", viewport);
        }
        builder = builder.add_component(meta_kv);

        // Meta Issues
        if !seo.meta_issues.is_empty() {
            let mut table = AuditTable::new(vec![
                TableColumn::new("Field"),
                TableColumn::new("Severity"),
                TableColumn::new("Message"),
            ]).with_title("Meta Tag Issues");

            for issue in &seo.meta_issues {
                table = table.add_row(vec![&issue.field, &issue.severity, &issue.message]);
            }
            builder = builder.add_component(table);
        }

        // Heading Structure
        builder = builder.add_component(
            SummaryBox::new("Heading Structure")
                .add_item("H1 Count", &seo.headings.h1_count.to_string())
                .add_item("Total Headings", &seo.headings.total_count.to_string())
                .add_item("Issues", &seo.headings.issues.len().to_string()),
        );

        if !seo.headings.issues.is_empty() {
            let mut heading_list = List::new().with_title("Heading Issues");
            for issue in &seo.headings.issues {
                heading_list = heading_list.add_item(&issue.message);
            }
            builder = builder.add_component(heading_list);
        }

        // Social Tags
        let mut social_kv = KeyValueList::new().with_title("Social Media Tags");
        social_kv = social_kv.add("Open Graph", if seo.social.open_graph.is_some() { "Present" } else { "Missing" });
        social_kv = social_kv.add("Twitter Card", if seo.social.twitter_card.is_some() { "Present" } else { "Missing" });
        social_kv = social_kv.add("Completeness", &format!("{}%", seo.social.completeness));
        builder = builder.add_component(social_kv);

        // Technical SEO
        builder = builder.add_component(
            SummaryBox::new("Technical SEO")
                .add_item("HTTPS", yes_no(seo.technical.https))
                .add_item("Canonical", yes_no(seo.technical.has_canonical))
                .add_item("Language", yes_no(seo.technical.has_lang))
                .add_item("Word Count", &seo.technical.word_count.to_string()),
        );

        // Structured Data
        if seo.structured_data.has_structured_data {
            let mut sd_list = List::new().with_title("Structured Data");
            for schema_type in &seo.structured_data.types {
                sd_list = sd_list.add_item(&format!("{:?}", schema_type));
            }
            builder = builder.add_component(sd_list);

            if !seo.structured_data.rich_snippets_potential.is_empty() {
                let mut snippets = List::new().with_title("Rich Snippet Opportunities");
                for snippet in &seo.structured_data.rich_snippets_potential {
                    snippets = snippets.add_item(snippet);
                }
                builder = builder.add_component(snippets);
            }
        }
    }

    // ── Security Section ─────────────────────────────────────────────────
    if let Some(ref sec) = report.security {
        builder = builder
            .add_component(Divider::new())
            .add_component(Section::new("Security").with_level(1))
            .add_component(
                ScoreCard::new("Security Score", sec.score)
                    .with_description(&format!("Grade: {}", sec.grade))
                    .with_thresholds(70, 50),
            );

        // Security Headers Checklist
        let header_checks: Vec<(&str, &Option<String>)> = vec![
            ("Content-Security-Policy", &sec.headers.content_security_policy),
            ("Strict-Transport-Security", &sec.headers.strict_transport_security),
            ("X-Content-Type-Options", &sec.headers.x_content_type_options),
            ("X-Frame-Options", &sec.headers.x_frame_options),
            ("X-XSS-Protection", &sec.headers.x_xss_protection),
            ("Referrer-Policy", &sec.headers.referrer_policy),
            ("Permissions-Policy", &sec.headers.permissions_policy),
            ("Cross-Origin-Opener-Policy", &sec.headers.cross_origin_opener_policy),
            ("Cross-Origin-Resource-Policy", &sec.headers.cross_origin_resource_policy),
        ];

        let mut headers_table = AuditTable::new(vec![
            TableColumn::new("Header"),
            TableColumn::new("Status"),
            TableColumn::new("Value"),
        ]).with_title("Security Headers");

        for (name, value) in &header_checks {
            let (status, val) = match value {
                Some(v) => ("Present", truncate_url(v, 50)),
                None => ("Missing", "—".to_string()),
            };
            headers_table = headers_table.add_row(vec![name.to_string(), status.to_string(), val]);
        }
        builder = builder.add_component(headers_table);

        // SSL/TLS Details
        builder = builder.add_component(
            KeyValueList::new()
                .with_title("SSL/TLS")
                .add("HTTPS", yes_no(sec.ssl.https))
                .add("Valid Certificate", yes_no(sec.ssl.valid_certificate))
                .add("HSTS Enabled", yes_no(sec.ssl.has_hsts))
                .add("HSTS Max-Age", &sec.ssl.hsts_max_age.map(|v| format!("{}s", v)).unwrap_or_else(|| "—".to_string()))
                .add("Include Subdomains", yes_no(sec.ssl.hsts_include_subdomains))
                .add("Preload", yes_no(sec.ssl.hsts_preload)),
        );

        // Security Issues
        for issue in &sec.issues {
            let severity = match issue.severity.as_str() {
                "critical" => Severity::Critical,
                "high" => Severity::High,
                "medium" => Severity::Medium,
                _ => Severity::Low,
            };
            builder = builder.add_component(
                Finding::new(&issue.header, severity, &issue.message)
                    .with_category(&issue.issue_type),
            );
        }

        // Recommendations
        if !sec.recommendations.is_empty() {
            let mut rec_list = List::new().with_title("Recommendations");
            for rec in &sec.recommendations {
                rec_list = rec_list.add_item(rec);
            }
            builder = builder.add_component(rec_list);
        }
    }

    // ── Mobile Section ───────────────────────────────────────────────────
    if let Some(ref mobile) = report.mobile {
        builder = builder
            .add_component(Divider::new())
            .add_component(Section::new("Mobile Friendliness").with_level(1))
            .add_component(
                ScoreCard::new("Mobile Score", mobile.score)
                    .with_thresholds(80, 50),
            );

        // Viewport Analysis
        builder = builder.add_component(
            KeyValueList::new()
                .with_title("Viewport Configuration")
                .add("Has Viewport Tag", yes_no(mobile.viewport.has_viewport))
                .add("Uses device-width", yes_no(mobile.viewport.uses_device_width))
                .add("Initial Scale", yes_no(mobile.viewport.has_initial_scale))
                .add("User Scalable", yes_no(mobile.viewport.is_scalable))
                .add("Properly Configured", yes_no(mobile.viewport.is_properly_configured)),
        );

        // Touch Targets
        builder = builder.add_component(
            SummaryBox::new("Touch Targets")
                .add_item("Total", &mobile.touch_targets.total_targets.to_string())
                .add_item("Adequate (≥44px)", &mobile.touch_targets.adequate_targets.to_string())
                .add_item("Too Small", &mobile.touch_targets.small_targets.to_string())
                .add_item("Crowded", &mobile.touch_targets.crowded_targets.to_string()),
        );

        // Font Analysis
        builder = builder.add_component(
            KeyValueList::new()
                .with_title("Font Analysis")
                .add("Base Font Size", &format!("{:.0}px", mobile.font_sizes.base_font_size))
                .add("Smallest Font", &format!("{:.0}px", mobile.font_sizes.smallest_font_size))
                .add("Legible Text", &format!("{:.0}%", mobile.font_sizes.legible_percentage))
                .add("Relative Units", yes_no(mobile.font_sizes.uses_relative_units)),
        );

        // Content Sizing
        builder = builder.add_component(
            SummaryBox::new("Content Sizing")
                .add_item("Fits Viewport", yes_no(mobile.content_sizing.fits_viewport))
                .add_item("No Horizontal Scroll", yes_no(!mobile.content_sizing.has_horizontal_scroll))
                .add_item("Responsive Images", yes_no(mobile.content_sizing.uses_responsive_images))
                .add_item("Media Queries", yes_no(mobile.content_sizing.uses_media_queries)),
        );

        // Mobile Issues
        for issue in &mobile.issues {
            let severity = match issue.severity.as_str() {
                "critical" => Severity::Critical,
                "high" => Severity::High,
                "medium" => Severity::Medium,
                _ => Severity::Low,
            };
            builder = builder.add_component(
                Finding::new(&issue.category, severity, &issue.message)
                    .with_category(&issue.impact),
            );
        }
    }

    // ── Overall Score ────────────────────────────────────────────────────
    if report.performance.is_some()
        || report.seo.is_some()
        || report.security.is_some()
        || report.mobile.is_some()
    {
        builder = builder
            .add_component(Divider::new())
            .add_component(Section::new("Overall Assessment").with_level(1))
            .add_component(
                ScoreCard::new("Overall Score", report.overall_score())
                    .with_description("Weighted average across all active modules")
                    .with_thresholds(70, 50),
            );

        // Module scores summary
        let mut module_summary = SummaryBox::new("Module Scores");
        module_summary = module_summary.add_item("WCAG Accessibility", &format!("{:.0}/100", report.score));
        if let Some(ref perf) = report.performance {
            module_summary = module_summary.add_item("Performance", &format!("{}/100", perf.score.overall));
        }
        if let Some(ref seo) = report.seo {
            module_summary = module_summary.add_item("SEO", &format!("{}/100", seo.score));
        }
        if let Some(ref sec) = report.security {
            module_summary = module_summary.add_item("Security", &format!("{}/100 ({})", sec.score, sec.grade));
        }
        if let Some(ref mobile) = report.mobile {
            module_summary = module_summary.add_item("Mobile", &format!("{}/100", mobile.score));
        }
        builder = builder.add_component(module_summary);
    }

    // Build and render to PDF
    let built_report = builder.build();
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
        .title("Web Batch Audit Report")
        .subtitle(&format!("{} URLs Audited", batch.summary.total_urls))
        .metadata(
            "date",
            &chrono::Utc::now().format("%Y-%m-%d %H:%M UTC").to_string(),
        )
        .metadata("total_urls", &batch.summary.total_urls.to_string())
        .metadata("success_rate", &format!("{:.1}%", success_rate))
        // Batch Summary
        .add_component(Section::new("Batch Summary").with_level(1))
        .add_component(
            SummaryBox::new("Overall Statistics")
                .add_item("Total URLs", &batch.summary.total_urls.to_string())
                .add_item("Passed", &batch.summary.passed.to_string())
                .add_item("Failed", &batch.summary.failed.to_string())
                .add_item("Success Rate", &format!("{:.1}%", success_rate))
                .add_item(
                    "Average Score",
                    &format!("{:.1}/100", batch.summary.average_score),
                )
                .add_item(
                    "Total Violations",
                    &batch.summary.total_violations.to_string(),
                )
                .add_item("Duration", &format!("{}ms", batch.total_duration_ms)),
        );

    // Add individual results
    builder = builder.add_component(Section::new("Individual Results").with_level(1));

    for (idx, report) in batch.reports.iter().enumerate() {
        builder = builder
            .add_component(
                Section::new(&format!("{}. {}", idx + 1, truncate_url(&report.url, 60)))
                    .with_level(2),
            )
            .add_component(
                ScoreCard::new("WCAG Score", report.score as u32)
                    .with_description(&format!(
                        "Grade: {} | {} violations",
                        report.grade,
                        report.wcag_results.violations.len()
                    ))
                    .with_thresholds(70, 50),
            );

        // Module scores summary per URL
        let has_modules = report.performance.is_some()
            || report.seo.is_some()
            || report.security.is_some()
            || report.mobile.is_some();

        if has_modules {
            let mut module_box = SummaryBox::new("Module Scores");
            if let Some(ref perf) = report.performance {
                module_box = module_box.add_item("Performance", &format!("{}/100", perf.score.overall));
            }
            if let Some(ref seo) = report.seo {
                module_box = module_box.add_item("SEO", &format!("{}/100", seo.score));
            }
            if let Some(ref sec) = report.security {
                module_box = module_box.add_item("Security", &format!("{}/100 ({})", sec.score, sec.grade));
            }
            if let Some(ref mobile) = report.mobile {
                module_box = module_box.add_item("Mobile", &format!("{}/100", mobile.score));
            }
            module_box = module_box.add_item("Overall", &format!("{}/100", report.overall_score()));
            builder = builder.add_component(module_box);
        }

        // All violations for each URL
        for violation in &report.wcag_results.violations {
            let mut finding = Finding::new(
                &format!("{} - {}", violation.rule, violation.rule_name),
                map_severity(&violation.severity),
                &violation.message,
            );
            if let Some(ref fix) = violation.fix_suggestion {
                finding = finding.with_recommendation(fix);
            }
            builder = builder.add_component(finding);
        }

        if report.wcag_results.violations.is_empty() {
            builder = builder.add_component(
                Callout::success("No violations found."),
            );
        }
    }

    // Build and render
    let built_report = builder.build();
    let pdf_bytes = engine.render_pdf(&built_report)?;

    Ok(pdf_bytes)
}

use crate::util::truncate_url;

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
