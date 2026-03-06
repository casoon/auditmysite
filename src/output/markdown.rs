//! Markdown output formatter
//!
//! Generates Markdown reports for single and batch audits.

use crate::audit::{AuditReport, BatchReport};

/// Format a single audit report as Markdown
pub fn format_markdown(report: &AuditReport) -> String {
    let mut output = String::new();

    output.push_str("# WCAG Accessibility Report\n\n");
    output.push_str(&format!("**URL:** {}\n\n", report.url));
    output.push_str(&format!(
        "**Date:** {}\n\n",
        report.timestamp.format("%Y-%m-%d %H:%M:%S UTC")
    ));
    output.push_str(&format!("**Score:** {}/100\n\n", report.score));
    output.push_str(&format!(
        "**Violations:** {}\n\n",
        report.wcag_results.violations.len()
    ));

    if !report.wcag_results.violations.is_empty() {
        output.push_str("## Violations\n\n");

        for violation in &report.wcag_results.violations {
            output.push_str(&format!(
                "### {} - {}\n\n",
                violation.rule, violation.rule_name
            ));
            output.push_str(&format!("- **Level:** {}\n", violation.level));
            output.push_str(&format!("- **Severity:** {}\n", violation.severity));
            output.push_str(&format!("- **Message:** {}\n", violation.message));

            if let Some(fix) = &violation.fix_suggestion {
                output.push_str(&format!("- **Suggested Fix:** {}\n", fix));
            }

            if let Some(url) = &violation.help_url {
                output.push_str(&format!(
                    "- **Learn More:** [WCAG Documentation]({})\n",
                    url
                ));
            }

            output.push('\n');
        }
    }

    // Performance section
    if let Some(ref perf) = report.performance {
        output.push_str(&format!(
            "## Performance — {}/100 ({})\n\n",
            perf.score.overall,
            perf.score.grade.label()
        ));

        // Web Vitals table
        output.push_str("### Core Web Vitals\n\n");
        output.push_str("| Metric | Value | Target | Rating |\n");
        output.push_str("|--------|-------|--------|--------|\n");

        let vitals: Vec<(&str, Option<&crate::performance::VitalMetric>, bool)> = vec![
            ("LCP (Largest Contentful Paint)", perf.vitals.lcp.as_ref(), true),
            ("FCP (First Contentful Paint)", perf.vitals.fcp.as_ref(), true),
            ("CLS (Cumulative Layout Shift)", perf.vitals.cls.as_ref(), false),
            ("TTFB (Time to First Byte)", perf.vitals.ttfb.as_ref(), true),
            ("INP (Interaction to Next Paint)", perf.vitals.inp.as_ref(), true),
            ("TBT (Total Blocking Time)", perf.vitals.tbt.as_ref(), true),
        ];

        for (name, metric, is_ms) in &vitals {
            if let Some(m) = metric {
                let val = if *is_ms {
                    format!("{:.0} ms", m.value)
                } else {
                    format!("{:.3}", m.value)
                };
                let target = if *is_ms {
                    format!("≤ {:.0} ms", m.target)
                } else {
                    format!("≤ {:.2}", m.target)
                };
                let rating_icon = match m.rating.as_str() {
                    "good" => "🟢 Good",
                    "needs-improvement" => "🟡 Needs Work",
                    _ => "🔴 Poor",
                };
                output.push_str(&format!("| {} | {} | {} | {} |\n", name, val, target, rating_icon));
            }
        }
        output.push('\n');

        // Additional metrics
        let mut extras = Vec::new();
        if let Some(nodes) = perf.vitals.dom_nodes {
            extras.push(format!("- **DOM Nodes:** {}", nodes));
        }
        if let Some(heap) = perf.vitals.js_heap_size {
            extras.push(format!("- **JS Heap Size:** {:.1} MB", heap as f64 / 1_048_576.0));
        }
        if let Some(load) = perf.vitals.load_time {
            extras.push(format!("- **Page Load Time:** {:.0} ms", load));
        }
        if let Some(dcl) = perf.vitals.dom_content_loaded {
            extras.push(format!("- **DOM Content Loaded:** {:.0} ms", dcl));
        }
        if !extras.is_empty() {
            output.push_str("### Additional Metrics\n\n");
            output.push_str(&extras.join("\n"));
            output.push_str("\n\n");
        }
    }

    // SEO section
    if let Some(ref seo) = report.seo {
        output.push_str(&format!("## SEO — {}/100\n\n", seo.score));

        // Meta tags
        output.push_str("### Meta Tags\n\n");
        output.push_str("| Tag | Value |\n");
        output.push_str("|-----|-------|\n");
        let meta_rows: Vec<(&str, &str)> = vec![
            ("Title", seo.meta.title.as_deref().unwrap_or("—")),
            ("Description", seo.meta.description.as_deref().unwrap_or("—")),
            ("Viewport", seo.meta.viewport.as_deref().unwrap_or("—")),
            ("Charset", seo.meta.charset.as_deref().unwrap_or("—")),
            ("Language", seo.meta.lang.as_deref().unwrap_or("—")),
            ("Canonical", seo.meta.canonical.as_deref().unwrap_or("—")),
        ];
        for (tag, val) in &meta_rows {
            output.push_str(&format!("| {} | {} |\n", tag, val));
        }
        output.push('\n');

        // Meta issues
        if !seo.meta_issues.is_empty() {
            output.push_str("### Meta Issues\n\n");
            output.push_str("| Field | Severity | Message |\n");
            output.push_str("|-------|----------|--------|\n");
            for issue in &seo.meta_issues {
                output.push_str(&format!(
                    "| {} | {} | {} |\n",
                    issue.field, issue.severity, issue.message
                ));
            }
            output.push('\n');
        }

        // Heading structure
        output.push_str("### Heading Structure\n\n");
        output.push_str(&format!("- **H1 Tags:** {}\n", seo.headings.h1_count));
        output.push_str(&format!("- **Total Headings:** {}\n", seo.headings.total_count));
        if !seo.headings.issues.is_empty() {
            output.push_str(&format!("- **Issues:** {}\n", seo.headings.issues.len()));
        }
        output.push('\n');

        // Social tags
        output.push_str("### Social Tags\n\n");
        output.push_str(&format!(
            "- **Open Graph:** {}\n",
            if seo.social.open_graph.is_some() { "✓ Present" } else { "✗ Missing" }
        ));
        output.push_str(&format!(
            "- **Twitter Card:** {}\n",
            if seo.social.twitter_card.is_some() { "✓ Present" } else { "✗ Missing" }
        ));
        output.push_str(&format!("- **Completeness:** {}%\n\n", seo.social.completeness));

        // Technical SEO
        output.push_str("### Technical SEO\n\n");
        let check = |b: bool| if b { "✓" } else { "✗" };
        output.push_str(&format!("- **HTTPS:** {}\n", check(seo.technical.https)));
        output.push_str(&format!("- **Canonical Tag:** {}\n", check(seo.technical.has_canonical)));
        output.push_str(&format!("- **Language Attribute:** {}\n", check(seo.technical.has_lang)));
        output.push_str(&format!("- **Word Count:** {}\n\n", seo.technical.word_count));

        // Structured data
        if seo.structured_data.has_structured_data {
            output.push_str("### Structured Data\n\n");
            output.push_str(&format!(
                "- **JSON-LD Schemas:** {}\n",
                seo.structured_data.json_ld.len()
            ));
            if !seo.structured_data.types.is_empty() {
                let types: Vec<String> = seo
                    .structured_data
                    .types
                    .iter()
                    .map(|t| format!("{:?}", t))
                    .collect();
                output.push_str(&format!("- **Schema Types:** {}\n", types.join(", ")));
            }
            if !seo.structured_data.rich_snippets_potential.is_empty() {
                output.push_str(&format!(
                    "- **Rich Snippet Opportunities:** {}\n",
                    seo.structured_data.rich_snippets_potential.join(", ")
                ));
            }
            output.push('\n');
        }
    }

    // Security section
    if let Some(ref sec) = report.security {
        output.push_str(&format!(
            "## Security — {}/100 (Grade {})\n\n",
            sec.score, sec.grade
        ));

        // Headers table
        output.push_str("### Security Headers\n\n");
        output.push_str("| Header | Status | Value |\n");
        output.push_str("|--------|--------|-------|\n");

        let headers: Vec<(&str, &Option<String>)> = vec![
            ("Content-Security-Policy", &sec.headers.content_security_policy),
            ("X-Content-Type-Options", &sec.headers.x_content_type_options),
            ("X-Frame-Options", &sec.headers.x_frame_options),
            ("X-XSS-Protection", &sec.headers.x_xss_protection),
            ("Referrer-Policy", &sec.headers.referrer_policy),
            ("Permissions-Policy", &sec.headers.permissions_policy),
            ("Strict-Transport-Security", &sec.headers.strict_transport_security),
            ("Cross-Origin-Opener-Policy", &sec.headers.cross_origin_opener_policy),
            ("Cross-Origin-Resource-Policy", &sec.headers.cross_origin_resource_policy),
        ];

        for (name, value) in &headers {
            let (status, val) = match value {
                Some(v) => ("✓", v.as_str()),
                None => ("✗", "Not set"),
            };
            output.push_str(&format!("| {} | {} | {} |\n", name, status, val));
        }
        output.push('\n');

        // SSL/TLS
        output.push_str("### SSL/TLS\n\n");
        let check = |b: bool| if b { "✓" } else { "✗" };
        output.push_str(&format!("- **HTTPS:** {}\n", check(sec.ssl.https)));
        output.push_str(&format!("- **Valid Certificate:** {}\n", check(sec.ssl.valid_certificate)));
        output.push_str(&format!("- **HSTS:** {}\n", check(sec.ssl.has_hsts)));
        if sec.ssl.has_hsts {
            if let Some(max_age) = sec.ssl.hsts_max_age {
                output.push_str(&format!("- **HSTS Max-Age:** {}\n", max_age));
            }
            output.push_str(&format!(
                "- **Include Subdomains:** {}\n",
                check(sec.ssl.hsts_include_subdomains)
            ));
            output.push_str(&format!("- **Preload:** {}\n", check(sec.ssl.hsts_preload)));
        }
        output.push('\n');

        // Issues
        if !sec.issues.is_empty() {
            output.push_str("### Security Issues\n\n");
            output.push_str("| Header | Severity | Message |\n");
            output.push_str("|--------|----------|---------|\n");
            for issue in &sec.issues {
                output.push_str(&format!(
                    "| {} | {} | {} |\n",
                    issue.header, issue.severity, issue.message
                ));
            }
            output.push('\n');
        }

        // Recommendations
        if !sec.recommendations.is_empty() {
            output.push_str("### Recommendations\n\n");
            for rec in &sec.recommendations {
                output.push_str(&format!("- {}\n", rec));
            }
            output.push('\n');
        }
    }

    // Mobile section
    if let Some(ref mobile) = report.mobile {
        output.push_str(&format!("## Mobile Friendliness — {}/100\n\n", mobile.score));

        // Viewport
        let check = |b: bool| if b { "✓" } else { "✗" };
        output.push_str("### Viewport Configuration\n\n");
        output.push_str(&format!("- **Has Viewport:** {}\n", check(mobile.viewport.has_viewport)));
        output.push_str(&format!("- **Device Width:** {}\n", check(mobile.viewport.uses_device_width)));
        output.push_str(&format!("- **Initial Scale:** {}\n", check(mobile.viewport.has_initial_scale)));
        output.push_str(&format!("- **Scalable:** {}\n", check(mobile.viewport.is_scalable)));
        output.push_str(&format!(
            "- **Properly Configured:** {}\n\n",
            check(mobile.viewport.is_properly_configured)
        ));

        // Touch targets
        output.push_str("### Touch Targets\n\n");
        output.push_str(&format!("- **Total Interactive Elements:** {}\n", mobile.touch_targets.total_targets));
        output.push_str(&format!("- **Adequate Size (≥44x44px):** {}\n", mobile.touch_targets.adequate_targets));
        output.push_str(&format!("- **Too Small:** {}\n", mobile.touch_targets.small_targets));
        output.push_str(&format!("- **Too Close Together:** {}\n\n", mobile.touch_targets.crowded_targets));

        // Font analysis
        output.push_str("### Font Analysis\n\n");
        output.push_str(&format!("- **Base Font Size:** {:.0}px\n", mobile.font_sizes.base_font_size));
        output.push_str(&format!("- **Smallest Font:** {:.0}px\n", mobile.font_sizes.smallest_font_size));
        output.push_str(&format!("- **Legible Text:** {:.0}%\n", mobile.font_sizes.legible_percentage));
        output.push_str(&format!("- **Relative Units:** {}\n\n", check(mobile.font_sizes.uses_relative_units)));

        // Content sizing
        output.push_str("### Content Sizing\n\n");
        output.push_str(&format!("- **Fits Viewport:** {}\n", check(mobile.content_sizing.fits_viewport)));
        output.push_str(&format!(
            "- **Horizontal Scroll:** {}\n",
            if mobile.content_sizing.has_horizontal_scroll { "✗ Yes" } else { "✓ No" }
        ));
        output.push_str(&format!(
            "- **Responsive Images:** {}\n",
            check(mobile.content_sizing.uses_responsive_images)
        ));
        output.push_str(&format!(
            "- **Media Queries:** {}\n\n",
            check(mobile.content_sizing.uses_media_queries)
        ));

        // Issues
        if !mobile.issues.is_empty() {
            output.push_str("### Mobile Issues\n\n");
            output.push_str("| Category | Type | Severity | Message |\n");
            output.push_str("|----------|------|----------|--------|\n");
            for issue in &mobile.issues {
                output.push_str(&format!(
                    "| {} | {} | {} | {} |\n",
                    issue.category, issue.issue_type, issue.severity, issue.message
                ));
            }
            output.push('\n');
        }
    }

    // Overall score if multiple modules
    if report.performance.is_some()
        || report.seo.is_some()
        || report.security.is_some()
        || report.mobile.is_some()
    {
        output.push_str(&format!(
            "## Overall Score: {}/100\n\n",
            report.overall_score()
        ));
        output.push_str("| Module | Score |\n");
        output.push_str("|--------|-------|\n");
        output.push_str(&format!("| WCAG | {:.0} |\n", report.score));
        if let Some(ref p) = report.performance {
            output.push_str(&format!("| Performance | {} |\n", p.score.overall));
        }
        if let Some(ref s) = report.seo {
            output.push_str(&format!("| SEO | {} |\n", s.score));
        }
        if let Some(ref s) = report.security {
            output.push_str(&format!("| Security | {} |\n", s.score));
        }
        if let Some(ref m) = report.mobile {
            output.push_str(&format!("| Mobile | {} |\n", m.score));
        }
        output.push('\n');
    }

    output.push_str("---\n\n");
    output.push_str(&format!(
        "*Generated by AuditMySit v{} in {}ms*\n",
        env!("CARGO_PKG_VERSION"),
        report.duration_ms
    ));

    output
}

/// Format a batch audit report as Markdown
pub fn format_batch_markdown(batch_report: &BatchReport) -> String {
    let mut output = String::new();

    output.push_str("# WCAG Batch Audit Report\n\n");
    output.push_str("## Summary\n\n");
    output.push_str(&format!(
        "- **Total URLs:** {}\n",
        batch_report.summary.total_urls
    ));
    output.push_str(&format!("- **Passed:** {}\n", batch_report.summary.passed));
    output.push_str(&format!("- **Failed:** {}\n", batch_report.summary.failed));
    output.push_str(&format!(
        "- **Average Score:** {:.1}\n",
        batch_report.summary.average_score
    ));
    output.push_str(&format!(
        "- **Total Violations:** {}\n",
        batch_report.summary.total_violations
    ));
    output.push_str(&format!(
        "- **Duration:** {}ms\n\n",
        batch_report.total_duration_ms
    ));

    output.push_str("## Results by URL\n\n");

    // Check which module columns we need
    let has_perf = batch_report.reports.iter().any(|r| r.performance.is_some());
    let has_seo = batch_report.reports.iter().any(|r| r.seo.is_some());
    let has_sec = batch_report.reports.iter().any(|r| r.security.is_some());
    let has_mobile = batch_report.reports.iter().any(|r| r.mobile.is_some());

    // Build header
    let mut header = "| URL | WCAG | Violations".to_string();
    let mut separator = "|-----|------|----------".to_string();
    if has_perf {
        header.push_str(" | Perf");
        separator.push_str("|-----");
    }
    if has_seo {
        header.push_str(" | SEO");
        separator.push_str("|----");
    }
    if has_sec {
        header.push_str(" | Security");
        separator.push_str("|---------");
    }
    if has_mobile {
        header.push_str(" | Mobile");
        separator.push_str("|-------");
    }
    header.push_str(" | Status |\n");
    separator.push_str("|--------|\n");

    output.push_str(&header);
    output.push_str(&separator);

    for report in &batch_report.reports {
        let status = if report.passed() { "Pass" } else { "Fail" };
        let mut row = format!(
            "| {} | {} | {}",
            report.url,
            report.score,
            report.violation_count(),
        );
        if has_perf {
            row.push_str(&format!(
                " | {}",
                report.performance.as_ref().map(|p| format!("{}", p.score.overall)).unwrap_or_else(|| "—".to_string())
            ));
        }
        if has_seo {
            row.push_str(&format!(
                " | {}",
                report.seo.as_ref().map(|s| format!("{}", s.score)).unwrap_or_else(|| "—".to_string())
            ));
        }
        if has_sec {
            row.push_str(&format!(
                " | {}",
                report.security.as_ref().map(|s| format!("{}", s.score)).unwrap_or_else(|| "—".to_string())
            ));
        }
        if has_mobile {
            row.push_str(&format!(
                " | {}",
                report.mobile.as_ref().map(|m| format!("{}", m.score)).unwrap_or_else(|| "—".to_string())
            ));
        }
        row.push_str(&format!(" | {} |\n", status));
        output.push_str(&row);
    }

    if !batch_report.errors.is_empty() {
        output.push_str("\n## Errors\n\n");
        for err in &batch_report.errors {
            output.push_str(&format!("- **{}**: {}\n", err.url, err.error));
        }
    }

    output.push_str("\n---\n\n");
    output.push_str(&format!(
        "*Generated by AuditMySit v{}*\n",
        env!("CARGO_PKG_VERSION")
    ));

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::AuditReport;
    use crate::cli::WcagLevel;
    use crate::wcag::{Severity, Violation, WcagResults};

    #[test]
    fn test_format_markdown_basic() {
        let report = AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            WcagResults::new(),
            500,
        );
        let md = format_markdown(&report);

        assert!(md.contains("# WCAG Accessibility Report"));
        assert!(md.contains("https://example.com"));
        assert!(md.contains("**Score:** 100/100"));
        assert!(md.contains("**Violations:** 0"));
    }

    #[test]
    fn test_format_markdown_with_violations() {
        let mut results = WcagResults::new();
        results.add_violation(
            Violation::new(
                "1.1.1",
                "Non-text Content",
                WcagLevel::A,
                Severity::Serious,
                "Image missing alt text",
                "node-1",
            )
            .with_fix("Add alt attribute"),
        );
        let report = AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            results,
            500,
        );
        let md = format_markdown(&report);

        assert!(md.contains("## Violations"));
        assert!(md.contains("1.1.1"));
        assert!(md.contains("Non-text Content"));
        assert!(md.contains("Add alt attribute"));
    }

    #[test]
    fn test_format_markdown_with_security_module() {
        let mut report = AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            WcagResults::new(),
            500,
        );
        report.security = Some(crate::security::SecurityAnalysis {
            score: 85,
            grade: "A".to_string(),
            headers: Default::default(),
            ssl: Default::default(),
            issues: vec![],
            recommendations: vec![],
        });
        let md = format_markdown(&report);

        assert!(md.contains("## Security"));
        assert!(md.contains("85/100"));
        assert!(md.contains("## Overall Score:"));
    }

    #[test]
    fn test_format_batch_markdown() {
        let reports = vec![
            AuditReport::new(
                "https://a.com".to_string(),
                WcagLevel::AA,
                WcagResults::new(),
                100,
            ),
            AuditReport::new(
                "https://b.com".to_string(),
                WcagLevel::AA,
                WcagResults::new(),
                200,
            ),
        ];
        let batch = crate::audit::BatchReport::from_reports(reports, vec![], 300);
        let md = format_batch_markdown(&batch);

        assert!(md.contains("# WCAG Batch Audit Report"));
        assert!(md.contains("**Total URLs:** 2"));
        assert!(md.contains("https://a.com"));
        assert!(md.contains("https://b.com"));
        assert!(md.contains("| Pass |"));
    }

    #[test]
    fn test_format_batch_markdown_with_errors() {
        let batch = crate::audit::BatchReport::from_reports(
            vec![],
            vec![crate::audit::BatchError {
                url: "https://fail.com".to_string(),
                error: "Connection refused".to_string(),
            }],
            100,
        );
        let md = format_batch_markdown(&batch);

        assert!(md.contains("## Errors"));
        assert!(md.contains("https://fail.com"));
        assert!(md.contains("Connection refused"));
    }
}
