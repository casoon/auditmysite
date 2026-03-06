//! HTML Report Generator
//!
//! Generates rich HTML reports with interactive dashboards.

use crate::audit::AuditReport;
use crate::error::Result;
use crate::wcag::{Severity, Violation};

/// Generate a complete HTML report from an audit
pub fn format_html(report: &AuditReport, wcag_level: &str) -> Result<String> {
    let html = HtmlReport::new(report, wcag_level);
    Ok(html.render())
}

/// HTML Report builder
struct HtmlReport<'a> {
    report: &'a AuditReport,
    wcag_level: &'a str,
}

impl<'a> HtmlReport<'a> {
    fn new(report: &'a AuditReport, wcag_level: &'a str) -> Self {
        Self { report, wcag_level }
    }

    fn render(&self) -> String {
        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Accessibility Audit Report - {url}</title>
    {styles}
</head>
<body>
    <div class="container">
        {header}
        {score_card}
        {summary_cards}
        {violations_by_severity}
        {violations_by_rule}
        {violations_list}
        {module_sections}
        {footer}
    </div>
    {scripts}
</body>
</html>"#,
            url = self.report.url,
            styles = self.render_styles(),
            header = self.render_header(),
            score_card = self.render_score_card(),
            summary_cards = self.render_summary_cards(),
            violations_by_severity = self.render_violations_by_severity(),
            violations_by_rule = self.render_violations_by_rule(),
            violations_list = self.render_violations_list(),
            module_sections = self.render_module_sections(),
            footer = self.render_footer(),
            scripts = self.render_scripts(),
        )
    }

    fn render_styles(&self) -> String {
        r#"<style>
:root {
    --color-critical: #dc2626;
    --color-serious: #ea580c;
    --color-moderate: #ca8a04;
    --color-minor: #2563eb;
    --color-pass: #16a34a;
    --color-bg: #f8fafc;
    --color-card: #ffffff;
    --color-border: #e2e8f0;
    --color-text: #1e293b;
    --color-text-muted: #64748b;
}

* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
    background: var(--color-bg);
    color: var(--color-text);
    line-height: 1.6;
}

.container {
    max-width: 1200px;
    margin: 0 auto;
    padding: 2rem;
}

header {
    text-align: center;
    margin-bottom: 2rem;
    padding-bottom: 1rem;
    border-bottom: 1px solid var(--color-border);
}

header h1 {
    font-size: 1.75rem;
    font-weight: 600;
    margin-bottom: 0.5rem;
}

header .url {
    color: var(--color-text-muted);
    font-size: 1rem;
    word-break: break-all;
}

header .meta {
    display: flex;
    justify-content: center;
    gap: 2rem;
    margin-top: 1rem;
    font-size: 0.875rem;
    color: var(--color-text-muted);
}

.score-card {
    background: var(--color-card);
    border-radius: 1rem;
    padding: 2rem;
    margin-bottom: 2rem;
    box-shadow: 0 1px 3px rgba(0,0,0,0.1);
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 3rem;
}

.score-gauge {
    position: relative;
    width: 160px;
    height: 160px;
}

.score-gauge svg {
    transform: rotate(-90deg);
}

.score-gauge circle {
    fill: none;
    stroke-width: 12;
}

.score-gauge .bg {
    stroke: var(--color-border);
}

.score-gauge .progress {
    stroke-linecap: round;
    transition: stroke-dashoffset 1s ease-out;
}

.score-gauge .score-text {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    font-size: 2.5rem;
    font-weight: 700;
}

.score-gauge .score-label {
    position: absolute;
    top: 70%;
    left: 50%;
    transform: translateX(-50%);
    font-size: 0.875rem;
    color: var(--color-text-muted);
}

.score-details h2 {
    font-size: 1.5rem;
    margin-bottom: 1rem;
}

.score-status {
    display: inline-block;
    padding: 0.5rem 1rem;
    border-radius: 0.5rem;
    font-weight: 600;
    font-size: 0.875rem;
}

.score-status.pass {
    background: #dcfce7;
    color: var(--color-pass);
}

.score-status.fail {
    background: #fee2e2;
    color: var(--color-critical);
}

.summary-cards {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 1rem;
    margin-bottom: 2rem;
}

.summary-card {
    background: var(--color-card);
    border-radius: 0.75rem;
    padding: 1.25rem;
    box-shadow: 0 1px 3px rgba(0,0,0,0.1);
}

.summary-card .label {
    font-size: 0.75rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--color-text-muted);
    margin-bottom: 0.5rem;
}

.summary-card .value {
    font-size: 1.75rem;
    font-weight: 700;
}

.summary-card.critical .value { color: var(--color-critical); }
.summary-card.serious .value { color: var(--color-serious); }
.summary-card.moderate .value { color: var(--color-moderate); }
.summary-card.minor .value { color: var(--color-minor); }
.summary-card.pass .value { color: var(--color-pass); }

.section {
    background: var(--color-card);
    border-radius: 0.75rem;
    padding: 1.5rem;
    margin-bottom: 1.5rem;
    box-shadow: 0 1px 3px rgba(0,0,0,0.1);
}

.section h2 {
    font-size: 1.125rem;
    font-weight: 600;
    margin-bottom: 1rem;
    padding-bottom: 0.75rem;
    border-bottom: 1px solid var(--color-border);
}

.chart-container {
    display: flex;
    gap: 2rem;
    flex-wrap: wrap;
}

.chart {
    flex: 1;
    min-width: 250px;
}

.bar-chart {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
}

.bar-item {
    display: flex;
    align-items: center;
    gap: 0.75rem;
}

.bar-label {
    width: 100px;
    font-size: 0.875rem;
    text-align: right;
}

.bar-track {
    flex: 1;
    height: 24px;
    background: var(--color-bg);
    border-radius: 4px;
    overflow: hidden;
}

.bar-fill {
    height: 100%;
    border-radius: 4px;
    transition: width 0.5s ease-out;
    display: flex;
    align-items: center;
    padding-left: 0.5rem;
}

.bar-fill span {
    color: white;
    font-size: 0.75rem;
    font-weight: 600;
}

.bar-fill.critical { background: var(--color-critical); }
.bar-fill.serious { background: var(--color-serious); }
.bar-fill.moderate { background: var(--color-moderate); }
.bar-fill.minor { background: var(--color-minor); }

.violations-list {
    display: flex;
    flex-direction: column;
    gap: 1rem;
}

.violation {
    border: 1px solid var(--color-border);
    border-radius: 0.5rem;
    overflow: hidden;
}

.violation-header {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 1rem;
    background: var(--color-bg);
    cursor: pointer;
}

.violation-header:hover {
    background: #f1f5f9;
}

.severity-badge {
    padding: 0.25rem 0.5rem;
    border-radius: 0.25rem;
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
}

.severity-badge.critical { background: #fee2e2; color: var(--color-critical); }
.severity-badge.serious { background: #ffedd5; color: var(--color-serious); }
.severity-badge.moderate { background: #fef9c3; color: var(--color-moderate); }
.severity-badge.minor { background: #dbeafe; color: var(--color-minor); }

.violation-rule {
    font-weight: 600;
    flex: 1;
}

.violation-rule code {
    background: var(--color-bg);
    padding: 0.125rem 0.375rem;
    border-radius: 0.25rem;
    font-size: 0.875rem;
    margin-right: 0.5rem;
}

.violation-details {
    padding: 1rem;
    display: none;
}

.violation.open .violation-details {
    display: block;
}

.violation-details dl {
    display: grid;
    grid-template-columns: 120px 1fr;
    gap: 0.5rem;
}

.violation-details dt {
    font-weight: 600;
    color: var(--color-text-muted);
    font-size: 0.875rem;
}

.violation-details dd {
    font-size: 0.875rem;
}

.violation-details .fix {
    margin-top: 1rem;
    padding: 0.75rem;
    background: #ecfdf5;
    border-radius: 0.375rem;
    border-left: 3px solid var(--color-pass);
}

.violation-details .fix strong {
    display: block;
    margin-bottom: 0.25rem;
    color: var(--color-pass);
}

footer {
    text-align: center;
    padding-top: 2rem;
    margin-top: 2rem;
    border-top: 1px solid var(--color-border);
    color: var(--color-text-muted);
    font-size: 0.875rem;
}

footer a {
    color: #2563eb;
    text-decoration: none;
}

footer a:hover {
    text-decoration: underline;
}

/* Module detail sections */
.module-section {
    background: var(--color-card);
    border-radius: 0.75rem;
    padding: 1.5rem;
    margin-bottom: 1.5rem;
    box-shadow: 0 1px 3px rgba(0,0,0,0.1);
}

.module-section h2 {
    font-size: 1.125rem;
    font-weight: 600;
    margin-bottom: 1rem;
    padding-bottom: 0.75rem;
    border-bottom: 1px solid var(--color-border);
    display: flex;
    align-items: center;
    gap: 0.75rem;
}

.module-score-badge {
    display: inline-block;
    padding: 0.25rem 0.75rem;
    border-radius: 1rem;
    font-size: 0.875rem;
    font-weight: 700;
    color: white;
}

.module-subsection {
    margin-top: 1.25rem;
}

.module-subsection h3 {
    font-size: 0.9375rem;
    font-weight: 600;
    margin-bottom: 0.75rem;
    color: var(--color-text);
}

.detail-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
    gap: 1rem;
}

.detail-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.875rem;
    margin-bottom: 0.5rem;
}

.detail-table th,
.detail-table td {
    padding: 0.625rem 0.75rem;
    text-align: left;
    border-bottom: 1px solid var(--color-border);
}

.detail-table th {
    background: var(--color-bg);
    font-weight: 600;
    color: var(--color-text-muted);
    font-size: 0.75rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
}

.detail-table tr:last-child td {
    border-bottom: none;
}

.kv-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
}

.kv-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.5rem 0;
    border-bottom: 1px solid var(--color-border);
    font-size: 0.875rem;
}

.kv-item:last-child {
    border-bottom: none;
}

.kv-key {
    color: var(--color-text-muted);
    font-weight: 500;
}

.kv-value {
    font-weight: 600;
    text-align: right;
    max-width: 60%;
    word-break: break-word;
}

.rating-badge {
    display: inline-block;
    padding: 0.125rem 0.5rem;
    border-radius: 0.25rem;
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
}

.rating-good { background: #dcfce7; color: #15803d; }
.rating-needs-improvement { background: #fef9c3; color: #a16207; }
.rating-poor { background: #fee2e2; color: #dc2626; }

.check-icon { color: #16a34a; font-weight: 700; }
.cross-icon { color: #dc2626; font-weight: 700; }

.tag-list {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    margin-top: 0.5rem;
}

.tag {
    display: inline-block;
    padding: 0.25rem 0.625rem;
    background: #eff6ff;
    color: #1d4ed8;
    border-radius: 0.25rem;
    font-size: 0.8125rem;
    font-weight: 500;
}

.issue-item {
    padding: 0.75rem;
    border: 1px solid var(--color-border);
    border-radius: 0.375rem;
    margin-bottom: 0.5rem;
    font-size: 0.875rem;
}

.issue-item .issue-header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-bottom: 0.25rem;
}

.issue-severity {
    padding: 0.125rem 0.375rem;
    border-radius: 0.25rem;
    font-size: 0.6875rem;
    font-weight: 600;
    text-transform: uppercase;
}

.issue-severity.high, .issue-severity.error { background: #fee2e2; color: #dc2626; }
.issue-severity.medium, .issue-severity.warning { background: #fef9c3; color: #a16207; }
.issue-severity.low, .issue-severity.info { background: #dbeafe; color: #2563eb; }

.recommendation-list {
    list-style: none;
    padding: 0;
}

.recommendation-list li {
    padding: 0.5rem 0 0.5rem 1.5rem;
    position: relative;
    font-size: 0.875rem;
    border-bottom: 1px solid var(--color-border);
}

.recommendation-list li:last-child {
    border-bottom: none;
}

.recommendation-list li::before {
    content: "→";
    position: absolute;
    left: 0;
    color: #2563eb;
    font-weight: 700;
}

.overall-scores {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
    gap: 1rem;
    margin-top: 1rem;
}

.overall-score-item {
    text-align: center;
    padding: 1rem;
    background: var(--color-bg);
    border-radius: 0.5rem;
}

.overall-score-item .score-value {
    font-size: 1.75rem;
    font-weight: 700;
}

.overall-score-item .score-name {
    font-size: 0.75rem;
    color: var(--color-text-muted);
    text-transform: uppercase;
    margin-top: 0.25rem;
}

@media (max-width: 768px) {
    .container {
        padding: 1rem;
    }

    .score-card {
        flex-direction: column;
        text-align: center;
    }

    .chart-container {
        flex-direction: column;
    }

    .bar-label {
        width: 80px;
        font-size: 0.75rem;
    }

    .detail-grid {
        grid-template-columns: 1fr;
    }
}
</style>"#.to_string()
    }

    fn render_header(&self) -> String {
        format!(
            r#"<header>
    <h1>Accessibility Audit Report</h1>
    <p class="url">{url}</p>
    <div class="meta">
        <span>WCAG {level}</span>
        <span>{timestamp}</span>
        <span>{duration}ms</span>
    </div>
</header>"#,
            url = self.report.url,
            level = self.wcag_level,
            timestamp = self.report.timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
            duration = self.report.duration_ms,
        )
    }

    fn render_score_card(&self) -> String {
        let score = self.report.score;
        let color = get_score_color(score);
        let circumference = 2.0 * std::f64::consts::PI * 65.0;
        let offset = circumference * (1.0 - score as f64 / 100.0);
        let passed = self.report.passed();

        format!(
            r#"<div class="score-card">
    <div class="score-gauge">
        <svg width="160" height="160" viewBox="0 0 160 160">
            <circle class="bg" cx="80" cy="80" r="65"></circle>
            <circle class="progress" cx="80" cy="80" r="65"
                stroke="{color}"
                stroke-dasharray="{circumference}"
                stroke-dashoffset="{offset}"></circle>
        </svg>
        <div class="score-text" style="color: {color}">{score}</div>
        <div class="score-label">Score</div>
    </div>
    <div class="score-details">
        <h2>WCAG {level} Compliance</h2>
        <span class="score-status {status_class}">{status_text}</span>
        <div style="margin-top: 1rem; display: flex; gap: 1.5rem; align-items: center;">
            <div>
                <span style="font-size: 0.75rem; color: var(--color-text-muted); text-transform: uppercase;">Grade</span>
                <div style="font-size: 2rem; font-weight: 700; color: {grade_color}">{grade}</div>
            </div>
            <div>
                <span style="font-size: 0.75rem; color: var(--color-text-muted); text-transform: uppercase;">Certificate</span>
                <div style="font-size: 1rem; font-weight: 600; color: {cert_color}">{certificate}</div>
            </div>
        </div>
        <p style="margin-top: 0.75rem; color: var(--color-text-muted);">
            {nodes} nodes analyzed &middot; {violations} violations found
        </p>
    </div>
</div>"#,
            color = color,
            circumference = circumference,
            offset = offset,
            score = score,
            level = self.wcag_level,
            status_class = if passed { "pass" } else { "fail" },
            status_text = if passed {
                "Passed"
            } else {
                "Needs Improvement"
            },
            grade = self.report.grade,
            grade_color = match self.report.grade.as_str() {
                "A" => "#16a34a",
                "B" => "#ca8a04",
                "C" => "#ea580c",
                _ => "#dc2626",
            },
            certificate = self.report.certificate,
            cert_color = match self.report.certificate.as_str() {
                "PLATINUM" => "#94a3b8",
                "GOLD" => "#ca8a04",
                "SILVER" => "#64748b",
                "BRONZE" => "#b45309",
                _ => "#dc2626",
            },
            nodes = self.report.nodes_analyzed,
            violations = self.report.violation_count(),
        )
    }

    fn render_summary_cards(&self) -> String {
        let violations = &self.report.wcag_results.violations;
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
        let passes = self.report.wcag_results.passes;

        format!(
            r#"<div class="summary-cards">
    <div class="summary-card critical">
        <div class="label">Critical</div>
        <div class="value">{critical}</div>
    </div>
    <div class="summary-card serious">
        <div class="label">Serious</div>
        <div class="value">{serious}</div>
    </div>
    <div class="summary-card moderate">
        <div class="label">Moderate</div>
        <div class="value">{moderate}</div>
    </div>
    <div class="summary-card minor">
        <div class="label">Minor</div>
        <div class="value">{minor}</div>
    </div>
    <div class="summary-card pass">
        <div class="label">Passes</div>
        <div class="value">{passes}</div>
    </div>
</div>"#,
            critical = critical,
            serious = serious,
            moderate = moderate,
            minor = minor,
            passes = passes,
        )
    }

    fn render_violations_by_severity(&self) -> String {
        let violations = &self.report.wcag_results.violations;
        let total = violations.len().max(1) as f64;

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

        format!(
            r#"<section class="section">
    <h2>Violations by Severity</h2>
    <div class="bar-chart">
        <div class="bar-item">
            <span class="bar-label">Critical</span>
            <div class="bar-track">
                <div class="bar-fill critical" style="width: {critical_pct}%"><span>{critical}</span></div>
            </div>
        </div>
        <div class="bar-item">
            <span class="bar-label">Serious</span>
            <div class="bar-track">
                <div class="bar-fill serious" style="width: {serious_pct}%"><span>{serious}</span></div>
            </div>
        </div>
        <div class="bar-item">
            <span class="bar-label">Moderate</span>
            <div class="bar-track">
                <div class="bar-fill moderate" style="width: {moderate_pct}%"><span>{moderate}</span></div>
            </div>
        </div>
        <div class="bar-item">
            <span class="bar-label">Minor</span>
            <div class="bar-track">
                <div class="bar-fill minor" style="width: {minor_pct}%"><span>{minor}</span></div>
            </div>
        </div>
    </div>
</section>"#,
            critical = critical,
            critical_pct = (critical as f64 / total * 100.0).round(),
            serious = serious,
            serious_pct = (serious as f64 / total * 100.0).round(),
            moderate = moderate,
            moderate_pct = (moderate as f64 / total * 100.0).round(),
            minor = minor,
            minor_pct = (minor as f64 / total * 100.0).round(),
        )
    }

    fn render_violations_by_rule(&self) -> String {
        use std::collections::HashMap;

        let violations = &self.report.wcag_results.violations;
        let mut by_rule: HashMap<String, usize> = HashMap::new();

        for v in violations {
            *by_rule
                .entry(format!("{} - {}", v.rule, v.rule_name))
                .or_insert(0) += 1;
        }

        let mut rules: Vec<_> = by_rule.into_iter().collect();
        rules.sort_by(|a, b| b.1.cmp(&a.1));

        let max_count = rules.first().map(|(_, c)| *c).unwrap_or(1).max(1) as f64;

        let bars: String = rules
            .iter()
            .take(8)
            .map(|(rule, count)| {
                let pct = (*count as f64 / max_count * 100.0).round();
                format!(
                    r#"<div class="bar-item">
            <span class="bar-label" title="{rule}">{short_rule}</span>
            <div class="bar-track">
                <div class="bar-fill moderate" style="width: {pct}%"><span>{count}</span></div>
            </div>
        </div>"#,
                    rule = rule,
                    short_rule = if rule.len() > 15 { &rule[..15] } else { rule },
                    pct = pct,
                    count = count,
                )
            })
            .collect();

        format!(
            r#"<section class="section">
    <h2>Violations by WCAG Rule</h2>
    <div class="bar-chart">
        {bars}
    </div>
</section>"#,
            bars = bars,
        )
    }

    fn render_violations_list(&self) -> String {
        let violations = &self.report.wcag_results.violations;

        if violations.is_empty() {
            return r#"<section class="section">
    <h2>All Checks Passed!</h2>
    <p>No accessibility violations were found.</p>
</section>"#
                .to_string();
        }

        let items: String = violations.iter().map(render_violation).collect();

        format!(
            r#"<section class="section">
    <h2>Violation Details ({count})</h2>
    <div class="violations-list">
        {items}
    </div>
</section>"#,
            count = violations.len(),
            items = items,
        )
    }

    fn render_module_sections(&self) -> String {
        let mut html = String::new();

        html.push_str(&self.render_performance_section());
        html.push_str(&self.render_seo_section());
        html.push_str(&self.render_security_section());
        html.push_str(&self.render_mobile_section());
        html.push_str(&self.render_overall_section());

        html
    }

    fn render_performance_section(&self) -> String {
        let perf = match self.report.performance {
            Some(ref p) => p,
            None => return String::new(),
        };

        let score_color = get_score_color(perf.score.overall as f32);

        // Web Vitals table rows
        let vitals: Vec<(&str, Option<&crate::performance::VitalMetric>, bool)> = vec![
            ("LCP (Largest Contentful Paint)", perf.vitals.lcp.as_ref(), true),
            ("FCP (First Contentful Paint)", perf.vitals.fcp.as_ref(), true),
            ("CLS (Cumulative Layout Shift)", perf.vitals.cls.as_ref(), false),
            ("TTFB (Time to First Byte)", perf.vitals.ttfb.as_ref(), true),
            ("INP (Interaction to Next Paint)", perf.vitals.inp.as_ref(), true),
            ("TBT (Total Blocking Time)", perf.vitals.tbt.as_ref(), true),
        ];

        let vital_rows: String = vitals
            .iter()
            .filter_map(|(name, metric, is_ms)| {
                metric.map(|m| {
                    let value_str = if *is_ms {
                        format!("{:.0} ms", m.value)
                    } else {
                        format!("{:.3}", m.value)
                    };
                    let rating_class = match m.rating.as_str() {
                        "good" => "rating-good",
                        "needs-improvement" => "rating-needs-improvement",
                        _ => "rating-poor",
                    };
                    let rating_label = match m.rating.as_str() {
                        "good" => "Good",
                        "needs-improvement" => "Needs Work",
                        _ => "Poor",
                    };
                    let target_str = if *is_ms {
                        format!("≤ {:.0} ms", m.target)
                    } else {
                        format!("≤ {:.2}", m.target)
                    };
                    format!(
                        "<tr><td>{}</td><td><strong>{}</strong></td><td>{}</td><td><span class=\"rating-badge {}\">{}</span></td></tr>",
                        name, value_str, target_str, rating_class, rating_label
                    )
                })
            })
            .collect();

        // Additional metrics
        let mut extra_items = Vec::new();
        if let Some(nodes) = perf.vitals.dom_nodes {
            extra_items.push(format!(
                r#"<div class="kv-item"><span class="kv-key">DOM Nodes</span><span class="kv-value">{}</span></div>"#,
                nodes
            ));
        }
        if let Some(heap) = perf.vitals.js_heap_size {
            extra_items.push(format!(
                r#"<div class="kv-item"><span class="kv-key">JS Heap Size</span><span class="kv-value">{:.1} MB</span></div>"#,
                heap as f64 / 1_048_576.0
            ));
        }
        if let Some(load) = perf.vitals.load_time {
            extra_items.push(format!(
                r#"<div class="kv-item"><span class="kv-key">Page Load Time</span><span class="kv-value">{:.0} ms</span></div>"#,
                load
            ));
        }
        if let Some(dcl) = perf.vitals.dom_content_loaded {
            extra_items.push(format!(
                r#"<div class="kv-item"><span class="kv-key">DOM Content Loaded</span><span class="kv-value">{:.0} ms</span></div>"#,
                dcl
            ));
        }

        let extra_html = if extra_items.is_empty() {
            String::new()
        } else {
            format!(
                r#"<div class="module-subsection">
        <h3>Additional Metrics</h3>
        <div class="kv-list">{}</div>
    </div>"#,
                extra_items.join("\n            ")
            )
        };

        format!(
            r#"<div class="module-section">
    <h2><span class="module-score-badge" style="background:{score_color}">{score}/100</span> Performance ({grade})</h2>
    <div class="module-subsection">
        <h3>Core Web Vitals</h3>
        <table class="detail-table">
            <thead><tr><th>Metric</th><th>Value</th><th>Target</th><th>Rating</th></tr></thead>
            <tbody>{vital_rows}</tbody>
        </table>
    </div>
    {extra_html}
</div>"#,
            score_color = score_color,
            score = perf.score.overall,
            grade = perf.score.grade.label(),
            vital_rows = vital_rows,
            extra_html = extra_html,
        )
    }

    fn render_seo_section(&self) -> String {
        let seo = match self.report.seo {
            Some(ref s) => s,
            None => return String::new(),
        };

        let score_color = get_score_color(seo.score as f32);

        // Meta tags
        let meta_items = vec![
            ("Title", seo.meta.title.as_deref().unwrap_or("—")),
            ("Description", seo.meta.description.as_deref().unwrap_or("—")),
            ("Viewport", seo.meta.viewport.as_deref().unwrap_or("—")),
            ("Charset", seo.meta.charset.as_deref().unwrap_or("—")),
            ("Language", seo.meta.lang.as_deref().unwrap_or("—")),
            ("Canonical", seo.meta.canonical.as_deref().unwrap_or("—")),
        ];

        let meta_kv: String = meta_items
            .iter()
            .map(|(k, v)| {
                format!(
                    r#"<div class="kv-item"><span class="kv-key">{}</span><span class="kv-value">{}</span></div>"#,
                    k,
                    html_escape(v)
                )
            })
            .collect();

        // Meta issues
        let meta_issues_html = if seo.meta_issues.is_empty() {
            String::new()
        } else {
            let rows: String = seo
                .meta_issues
                .iter()
                .map(|issue| {
                    let sev_class = match issue.severity.as_str() {
                        "error" => "error",
                        "warning" => "warning",
                        _ => "info",
                    };
                    format!(
                        r#"<div class="issue-item">
                <div class="issue-header">
                    <span class="issue-severity {sev_class}">{severity}</span>
                    <strong>{field}</strong>
                </div>
                <p>{message}</p>
            </div>"#,
                        sev_class = sev_class,
                        severity = html_escape(&issue.severity),
                        field = html_escape(&issue.field),
                        message = html_escape(&issue.message),
                    )
                })
                .collect();
            format!(
                r#"<div class="module-subsection">
        <h3>Meta Issues ({count})</h3>
        {rows}
    </div>"#,
                count = seo.meta_issues.len(),
                rows = rows,
            )
        };

        // Heading structure
        let heading_html = format!(
            r#"<div class="module-subsection">
        <h3>Heading Structure</h3>
        <div class="kv-list">
            <div class="kv-item"><span class="kv-key">H1 Tags</span><span class="kv-value">{h1}</span></div>
            <div class="kv-item"><span class="kv-key">Total Headings</span><span class="kv-value">{total}</span></div>
            <div class="kv-item"><span class="kv-key">Issues</span><span class="kv-value">{issues}</span></div>
        </div>
    </div>"#,
            h1 = seo.headings.h1_count,
            total = seo.headings.total_count,
            issues = seo.headings.issues.len(),
        );

        // Social tags
        let og_status = if seo.social.open_graph.is_some() {
            r#"<span class="check-icon">✓</span> Present"#
        } else {
            r#"<span class="cross-icon">✗</span> Missing"#
        };
        let tw_status = if seo.social.twitter_card.is_some() {
            r#"<span class="check-icon">✓</span> Present"#
        } else {
            r#"<span class="cross-icon">✗</span> Missing"#
        };
        let social_html = format!(
            r#"<div class="module-subsection">
        <h3>Social Tags</h3>
        <div class="kv-list">
            <div class="kv-item"><span class="kv-key">Open Graph</span><span class="kv-value">{og}</span></div>
            <div class="kv-item"><span class="kv-key">Twitter Card</span><span class="kv-value">{tw}</span></div>
            <div class="kv-item"><span class="kv-key">Completeness</span><span class="kv-value">{comp}%</span></div>
        </div>
    </div>"#,
            og = og_status,
            tw = tw_status,
            comp = seo.social.completeness,
        );

        // Technical SEO
        let check = |b: bool| {
            if b {
                r#"<span class="check-icon">✓</span>"#
            } else {
                r#"<span class="cross-icon">✗</span>"#
            }
        };
        let tech_html = format!(
            r#"<div class="module-subsection">
        <h3>Technical SEO</h3>
        <div class="kv-list">
            <div class="kv-item"><span class="kv-key">HTTPS</span><span class="kv-value">{https}</span></div>
            <div class="kv-item"><span class="kv-key">Canonical Tag</span><span class="kv-value">{canonical}</span></div>
            <div class="kv-item"><span class="kv-key">Language Attribute</span><span class="kv-value">{lang}</span></div>
            <div class="kv-item"><span class="kv-key">Word Count</span><span class="kv-value">{words}</span></div>
        </div>
    </div>"#,
            https = check(seo.technical.https),
            canonical = check(seo.technical.has_canonical),
            lang = check(seo.technical.has_lang),
            words = seo.technical.word_count,
        );

        // Structured data
        let structured_html = if seo.structured_data.has_structured_data {
            let types: String = seo
                .structured_data
                .types
                .iter()
                .map(|t| format!(r#"<span class="tag">{:?}</span>"#, t))
                .collect();
            let snippets: String = seo
                .structured_data
                .rich_snippets_potential
                .iter()
                .map(|s| format!(r#"<span class="tag">{}</span>"#, html_escape(s)))
                .collect();
            format!(
                r#"<div class="module-subsection">
        <h3>Structured Data</h3>
        <div class="kv-list">
            <div class="kv-item"><span class="kv-key">JSON-LD Schemas</span><span class="kv-value">{count}</span></div>
        </div>
        <p style="margin-top:0.5rem;font-size:0.8125rem;color:var(--color-text-muted)">Schema Types:</p>
        <div class="tag-list">{types}</div>
        {snippets_section}
    </div>"#,
                count = seo.structured_data.json_ld.len(),
                types = types,
                snippets_section = if snippets.is_empty() {
                    String::new()
                } else {
                    format!(
                        r#"<p style="margin-top:0.75rem;font-size:0.8125rem;color:var(--color-text-muted)">Rich Snippet Opportunities:</p>
        <div class="tag-list">{}</div>"#,
                        snippets
                    )
                },
            )
        } else {
            r#"<div class="module-subsection">
        <h3>Structured Data</h3>
        <p style="font-size:0.875rem;color:var(--color-text-muted)">No structured data detected.</p>
    </div>"#
                .to_string()
        };

        format!(
            r#"<div class="module-section">
    <h2><span class="module-score-badge" style="background:{score_color}">{score}/100</span> SEO</h2>
    <div class="module-subsection">
        <h3>Meta Tags</h3>
        <div class="kv-list">{meta_kv}</div>
    </div>
    {meta_issues_html}
    {heading_html}
    {social_html}
    {tech_html}
    {structured_html}
</div>"#,
            score_color = score_color,
            score = seo.score,
            meta_kv = meta_kv,
            meta_issues_html = meta_issues_html,
            heading_html = heading_html,
            social_html = social_html,
            tech_html = tech_html,
            structured_html = structured_html,
        )
    }

    fn render_security_section(&self) -> String {
        let sec = match self.report.security {
            Some(ref s) => s,
            None => return String::new(),
        };

        let score_color = get_score_color(sec.score as f32);

        // Security headers table
        let headers_data: Vec<(&str, &Option<String>)> = vec![
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

        let header_rows: String = headers_data
            .iter()
            .map(|(name, value)| {
                let (status, val_display) = match value {
                    Some(v) => (
                        r#"<span class="check-icon">✓</span>"#,
                        html_escape(v),
                    ),
                    None => (
                        r#"<span class="cross-icon">✗</span>"#,
                        "Not set".to_string(),
                    ),
                };
                format!(
                    "<tr><td>{}</td><td>{}</td><td style=\"font-size:0.8125rem;max-width:300px;word-break:break-all\">{}</td></tr>",
                    name, status, val_display
                )
            })
            .collect();

        // SSL/TLS details
        let check = |b: bool| {
            if b {
                r#"<span class="check-icon">✓</span>"#
            } else {
                r#"<span class="cross-icon">✗</span>"#
            }
        };
        let ssl_html = format!(
            r#"<div class="module-subsection">
        <h3>SSL/TLS</h3>
        <div class="kv-list">
            <div class="kv-item"><span class="kv-key">HTTPS</span><span class="kv-value">{https}</span></div>
            <div class="kv-item"><span class="kv-key">Valid Certificate</span><span class="kv-value">{cert}</span></div>
            <div class="kv-item"><span class="kv-key">HSTS</span><span class="kv-value">{hsts}</span></div>
            {hsts_details}
        </div>
    </div>"#,
            https = check(sec.ssl.https),
            cert = check(sec.ssl.valid_certificate),
            hsts = check(sec.ssl.has_hsts),
            hsts_details = if sec.ssl.has_hsts {
                format!(
                    r#"<div class="kv-item"><span class="kv-key">HSTS Max-Age</span><span class="kv-value">{}</span></div>
            <div class="kv-item"><span class="kv-key">Include Subdomains</span><span class="kv-value">{}</span></div>
            <div class="kv-item"><span class="kv-key">Preload</span><span class="kv-value">{}</span></div>"#,
                    sec.ssl.hsts_max_age.map(|v| format!("{}", v)).unwrap_or_else(|| "—".to_string()),
                    check(sec.ssl.hsts_include_subdomains),
                    check(sec.ssl.hsts_preload),
                )
            } else {
                String::new()
            },
        );

        // Security issues
        let issues_html = if sec.issues.is_empty() {
            String::new()
        } else {
            let items: String = sec
                .issues
                .iter()
                .map(|issue| {
                    let sev_class = match issue.severity.as_str() {
                        "high" => "high",
                        "medium" => "medium",
                        _ => "low",
                    };
                    format!(
                        r#"<div class="issue-item">
                <div class="issue-header">
                    <span class="issue-severity {sev_class}">{severity}</span>
                    <strong>{header}</strong>
                </div>
                <p>{message}</p>
            </div>"#,
                        sev_class = sev_class,
                        severity = html_escape(&issue.severity),
                        header = html_escape(&issue.header),
                        message = html_escape(&issue.message),
                    )
                })
                .collect();
            format!(
                r#"<div class="module-subsection">
        <h3>Issues ({count})</h3>
        {items}
    </div>"#,
                count = sec.issues.len(),
                items = items,
            )
        };

        // Recommendations
        let rec_html = if sec.recommendations.is_empty() {
            String::new()
        } else {
            let items: String = sec
                .recommendations
                .iter()
                .map(|r| format!("<li>{}</li>", html_escape(r)))
                .collect();
            format!(
                r#"<div class="module-subsection">
        <h3>Recommendations</h3>
        <ul class="recommendation-list">{items}</ul>
    </div>"#,
                items = items,
            )
        };

        format!(
            r#"<div class="module-section">
    <h2><span class="module-score-badge" style="background:{score_color}">{score}/100</span> Security (Grade {grade})</h2>
    <div class="module-subsection">
        <h3>Security Headers ({present}/9)</h3>
        <table class="detail-table">
            <thead><tr><th>Header</th><th>Status</th><th>Value</th></tr></thead>
            <tbody>{header_rows}</tbody>
        </table>
    </div>
    {ssl_html}
    {issues_html}
    {rec_html}
</div>"#,
            score_color = score_color,
            score = sec.score,
            grade = html_escape(&sec.grade),
            present = sec.headers.count(),
            header_rows = header_rows,
            ssl_html = ssl_html,
            issues_html = issues_html,
            rec_html = rec_html,
        )
    }

    fn render_mobile_section(&self) -> String {
        let mobile = match self.report.mobile {
            Some(ref m) => m,
            None => return String::new(),
        };

        let score_color = get_score_color(mobile.score as f32);
        let check = |b: bool| {
            if b {
                r#"<span class="check-icon">✓</span>"#
            } else {
                r#"<span class="cross-icon">✗</span>"#
            }
        };

        // Viewport
        let viewport_html = format!(
            r#"<div class="module-subsection">
        <h3>Viewport Configuration</h3>
        <div class="kv-list">
            <div class="kv-item"><span class="kv-key">Has Viewport</span><span class="kv-value">{has}</span></div>
            <div class="kv-item"><span class="kv-key">Device Width</span><span class="kv-value">{dw}</span></div>
            <div class="kv-item"><span class="kv-key">Initial Scale</span><span class="kv-value">{is}</span></div>
            <div class="kv-item"><span class="kv-key">Scalable</span><span class="kv-value">{sc}</span></div>
            <div class="kv-item"><span class="kv-key">Properly Configured</span><span class="kv-value">{pc}</span></div>
        </div>
    </div>"#,
            has = check(mobile.viewport.has_viewport),
            dw = check(mobile.viewport.uses_device_width),
            is = check(mobile.viewport.has_initial_scale),
            sc = check(mobile.viewport.is_scalable),
            pc = check(mobile.viewport.is_properly_configured),
        );

        // Touch targets
        let touch_html = format!(
            r#"<div class="module-subsection">
        <h3>Touch Targets</h3>
        <div class="kv-list">
            <div class="kv-item"><span class="kv-key">Total Interactive Elements</span><span class="kv-value">{total}</span></div>
            <div class="kv-item"><span class="kv-key">Adequate Size (≥44x44px)</span><span class="kv-value">{adequate}</span></div>
            <div class="kv-item"><span class="kv-key">Too Small</span><span class="kv-value" style="color:{small_color}">{small}</span></div>
            <div class="kv-item"><span class="kv-key">Too Close Together</span><span class="kv-value" style="color:{crowded_color}">{crowded}</span></div>
        </div>
    </div>"#,
            total = mobile.touch_targets.total_targets,
            adequate = mobile.touch_targets.adequate_targets,
            small = mobile.touch_targets.small_targets,
            small_color = if mobile.touch_targets.small_targets > 0 { "var(--color-serious)" } else { "var(--color-pass)" },
            crowded = mobile.touch_targets.crowded_targets,
            crowded_color = if mobile.touch_targets.crowded_targets > 0 { "var(--color-moderate)" } else { "var(--color-pass)" },
        );

        // Font analysis
        let font_html = format!(
            r#"<div class="module-subsection">
        <h3>Font Analysis</h3>
        <div class="kv-list">
            <div class="kv-item"><span class="kv-key">Base Font Size</span><span class="kv-value">{base:.0}px</span></div>
            <div class="kv-item"><span class="kv-key">Smallest Font</span><span class="kv-value" style="color:{smallest_color}">{smallest:.0}px</span></div>
            <div class="kv-item"><span class="kv-key">Legible Text</span><span class="kv-value">{legible:.0}%</span></div>
            <div class="kv-item"><span class="kv-key">Relative Units</span><span class="kv-value">{relative}</span></div>
        </div>
    </div>"#,
            base = mobile.font_sizes.base_font_size,
            smallest = mobile.font_sizes.smallest_font_size,
            smallest_color = if mobile.font_sizes.smallest_font_size < 12.0 { "var(--color-serious)" } else { "var(--color-pass)" },
            legible = mobile.font_sizes.legible_percentage,
            relative = check(mobile.font_sizes.uses_relative_units),
        );

        // Content sizing
        let content_html = format!(
            r#"<div class="module-subsection">
        <h3>Content Sizing</h3>
        <div class="kv-list">
            <div class="kv-item"><span class="kv-key">Fits Viewport</span><span class="kv-value">{fits}</span></div>
            <div class="kv-item"><span class="kv-key">Horizontal Scroll</span><span class="kv-value">{scroll}</span></div>
            <div class="kv-item"><span class="kv-key">Responsive Images</span><span class="kv-value">{images}</span></div>
            <div class="kv-item"><span class="kv-key">Media Queries</span><span class="kv-value">{media}</span></div>
        </div>
    </div>"#,
            fits = check(mobile.content_sizing.fits_viewport),
            scroll = if mobile.content_sizing.has_horizontal_scroll {
                r#"<span class="cross-icon">✗ Yes</span>"#
            } else {
                r#"<span class="check-icon">✓ No</span>"#
            },
            images = check(mobile.content_sizing.uses_responsive_images),
            media = check(mobile.content_sizing.uses_media_queries),
        );

        // Mobile issues
        let issues_html = if mobile.issues.is_empty() {
            String::new()
        } else {
            let items: String = mobile
                .issues
                .iter()
                .map(|issue| {
                    let sev_class = match issue.severity.as_str() {
                        "high" => "high",
                        "medium" => "medium",
                        _ => "low",
                    };
                    format!(
                        r#"<div class="issue-item">
                <div class="issue-header">
                    <span class="issue-severity {sev_class}">{severity}</span>
                    <strong>[{category}] {issue_type}</strong>
                </div>
                <p>{message}</p>
            </div>"#,
                        sev_class = sev_class,
                        severity = html_escape(&issue.severity),
                        category = html_escape(&issue.category),
                        issue_type = html_escape(&issue.issue_type),
                        message = html_escape(&issue.message),
                    )
                })
                .collect();
            format!(
                r#"<div class="module-subsection">
        <h3>Issues ({count})</h3>
        {items}
    </div>"#,
                count = mobile.issues.len(),
                items = items,
            )
        };

        format!(
            r#"<div class="module-section">
    <h2><span class="module-score-badge" style="background:{score_color}">{score}/100</span> Mobile Friendliness</h2>
    {viewport_html}
    {touch_html}
    {font_html}
    {content_html}
    {issues_html}
</div>"#,
            score_color = score_color,
            score = mobile.score,
            viewport_html = viewport_html,
            touch_html = touch_html,
            font_html = font_html,
            content_html = content_html,
            issues_html = issues_html,
        )
    }

    fn render_overall_section(&self) -> String {
        let has_modules = self.report.performance.is_some()
            || self.report.seo.is_some()
            || self.report.security.is_some()
            || self.report.mobile.is_some();

        if !has_modules {
            return String::new();
        }

        let overall = self.report.overall_score();
        let overall_color = get_score_color(overall as f32);

        let mut score_items = Vec::new();
        score_items.push(format!(
            r#"<div class="overall-score-item">
            <div class="score-value" style="color:{}">{:.0}</div>
            <div class="score-name">WCAG</div>
        </div>"#,
            get_score_color(self.report.score),
            self.report.score,
        ));
        if let Some(ref p) = self.report.performance {
            score_items.push(format!(
                r#"<div class="overall-score-item">
            <div class="score-value" style="color:{}">{}</div>
            <div class="score-name">Performance</div>
        </div>"#,
                get_score_color(p.score.overall as f32),
                p.score.overall,
            ));
        }
        if let Some(ref s) = self.report.seo {
            score_items.push(format!(
                r#"<div class="overall-score-item">
            <div class="score-value" style="color:{}">{}</div>
            <div class="score-name">SEO</div>
        </div>"#,
                get_score_color(s.score as f32),
                s.score,
            ));
        }
        if let Some(ref s) = self.report.security {
            score_items.push(format!(
                r#"<div class="overall-score-item">
            <div class="score-value" style="color:{}">{}</div>
            <div class="score-name">Security</div>
        </div>"#,
                get_score_color(s.score as f32),
                s.score,
            ));
        }
        if let Some(ref m) = self.report.mobile {
            score_items.push(format!(
                r#"<div class="overall-score-item">
            <div class="score-value" style="color:{}">{}</div>
            <div class="score-name">Mobile</div>
        </div>"#,
                get_score_color(m.score as f32),
                m.score,
            ));
        }

        format!(
            r#"<div class="module-section">
    <h2><span class="module-score-badge" style="background:{overall_color}">{overall}/100</span> Overall Assessment</h2>
    <div class="overall-scores">
        {items}
    </div>
</div>"#,
            overall_color = overall_color,
            overall = overall,
            items = score_items.join("\n        "),
        )
    }

    fn render_footer(&self) -> String {
        format!(
            r#"<footer>
    <p>Generated by <strong>AuditMySit</strong> v{version}</p>
    <p>
        <a href="https://www.w3.org/WAI/WCAG21/quickref/" target="_blank">WCAG 2.1 Quick Reference</a> &middot;
        <a href="https://www.w3.org/WAI/standards-guidelines/wcag/" target="_blank">About WCAG</a>
    </p>
</footer>"#,
            version = env!("CARGO_PKG_VERSION"),
        )
    }

    fn render_scripts(&self) -> String {
        r#"<script>
document.querySelectorAll('.violation-header').forEach(header => {
    header.addEventListener('click', () => {
        header.parentElement.classList.toggle('open');
    });
});
</script>"#
            .to_string()
    }
}

fn render_violation(v: &Violation) -> String {
    let severity_class = match v.severity {
        Severity::Critical => "critical",
        Severity::Serious => "serious",
        Severity::Moderate => "moderate",
        Severity::Minor => "minor",
    };

    let fix_html = v
        .fix_suggestion
        .as_ref()
        .map(|fix| {
            format!(
                r#"<div class="fix"><strong>How to fix:</strong> {}</div>"#,
                html_escape(fix)
            )
        })
        .unwrap_or_default();

    let help_html = v
        .help_url
        .as_ref()
        .map(|url| {
            format!(
                r#"<dd><a href="{}" target="_blank">Learn more</a></dd>"#,
                url
            )
        })
        .unwrap_or_default();

    format!(
        r#"<div class="violation">
    <div class="violation-header">
        <span class="severity-badge {severity_class}">{severity}</span>
        <span class="violation-rule"><code>{rule}</code> {rule_name}</span>
    </div>
    <div class="violation-details">
        <dl>
            <dt>Message</dt>
            <dd>{message}</dd>
            <dt>Node ID</dt>
            <dd><code>{node_id}</code></dd>
            {role_row}
            {name_row}
            <dt>Help</dt>
            {help_html}
        </dl>
        {fix_html}
    </div>
</div>"#,
        severity_class = severity_class,
        severity = format_args!("{:?}", v.severity),
        rule = v.rule,
        rule_name = html_escape(&v.rule_name),
        message = html_escape(&v.message),
        node_id = html_escape(&v.node_id),
        role_row = v
            .role
            .as_ref()
            .map(|r| format!("<dt>Role</dt><dd>{}</dd>", html_escape(r)))
            .unwrap_or_default(),
        name_row = v
            .name
            .as_ref()
            .map(|n| format!("<dt>Name</dt><dd>{}</dd>", html_escape(n)))
            .unwrap_or_default(),
        help_html = help_html,
        fix_html = fix_html,
    )
}

fn get_score_color(score: f32) -> &'static str {
    match score as u32 {
        90..=100 => "#16a34a", // green
        70..=89 => "#ca8a04",  // yellow
        50..=69 => "#ea580c",  // orange
        _ => "#dc2626",        // red
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// Generate HTML report for a batch of audits
pub fn format_batch_html(reports: &[AuditReport], wcag_level: &str) -> Result<String> {
    let total = reports.len();
    let passed = reports.iter().filter(|r| r.passed()).count();
    let avg_score: f64 = reports.iter().map(|r| r.score as f64).sum::<f64>() / total.max(1) as f64;
    let total_violations: usize = reports.iter().map(|r| r.violation_count()).sum();

    // Check if any report has module data
    let has_perf = reports.iter().any(|r| r.performance.is_some());
    let has_seo = reports.iter().any(|r| r.seo.is_some());
    let has_sec = reports.iter().any(|r| r.security.is_some());
    let has_mobile = reports.iter().any(|r| r.mobile.is_some());

    let extra_headers = format!(
        "{}{}{}{}",
        if has_perf { "<th>Perf</th>" } else { "" },
        if has_seo { "<th>SEO</th>" } else { "" },
        if has_sec { "<th>Security</th>" } else { "" },
        if has_mobile { "<th>Mobile</th>" } else { "" },
    );

    let rows: String = reports.iter().map(|r| {
        let status = if r.passed() { "pass" } else { "fail" };
        let url_id = url_to_id(&r.url);
        let url = html_escape(&r.url);
        let status_text = if r.passed() { "Pass" } else { "Fail" };

        let extra_cols = format!(
            "{}{}{}{}",
            if has_perf {
                format!("<td>{}</td>", r.performance.as_ref().map(|p| format!("{}", p.score.overall)).unwrap_or_else(|| "—".to_string()))
            } else { String::new() },
            if has_seo {
                format!("<td>{}</td>", r.seo.as_ref().map(|s| format!("{}", s.score)).unwrap_or_else(|| "—".to_string()))
            } else { String::new() },
            if has_sec {
                format!("<td>{}</td>", r.security.as_ref().map(|s| format!("{}", s.score)).unwrap_or_else(|| "—".to_string()))
            } else { String::new() },
            if has_mobile {
                format!("<td>{}</td>", r.mobile.as_ref().map(|m| format!("{}", m.score)).unwrap_or_else(|| "—".to_string()))
            } else { String::new() },
        );

        format!(
            "<tr class=\"{status}\">\n    <td><a href=\"#{url_id}\">{url}</a></td>\n    <td>{score}</td>\n    <td>{violations}</td>\n    {extra_cols}\n    <td class=\"status-{status}\">{status_text}</td>\n</tr>",
            status = status,
            url_id = url_id,
            url = url,
            score = r.score,
            violations = r.violation_count(),
            extra_cols = extra_cols,
            status_text = status_text,
        )
    }).collect();

    let individual_reports: String = reports
        .iter()
        .map(|r| {
            let html_report = HtmlReport::new(r, wcag_level);
            let violations = html_report.render_violations_list();
            let modules = html_report.render_module_sections();
            format!(
                r#"<div id="{url_id}" class="individual-report">
    <h2>{url}</h2>
    <div class="module-scores-row">{module_scores}</div>
    {modules}
    {violations}
</div>"#,
                url_id = url_to_id(&r.url),
                url = html_escape(&r.url),
                module_scores = format_batch_module_scores(r),
                modules = modules,
                violations = violations,
            )
        })
        .collect();

    Ok(format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Batch Accessibility Audit Report</title>
    <style>
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #f8fafc; color: #1e293b; margin: 0; padding: 2rem; line-height: 1.6; }}
        .container {{ max-width: 1400px; margin: 0 auto; }}
        h1 {{ margin-bottom: 2rem; }}
        .summary {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 1rem; margin-bottom: 2rem; }}
        .summary-item {{ background: white; padding: 1.5rem; border-radius: 0.5rem; box-shadow: 0 1px 3px rgba(0,0,0,0.1); }}
        .summary-item .value {{ font-size: 2rem; font-weight: 700; }}
        .summary-item .label {{ color: #64748b; font-size: 0.875rem; }}
        table {{ width: 100%; border-collapse: collapse; background: white; border-radius: 0.5rem; overflow: hidden; box-shadow: 0 1px 3px rgba(0,0,0,0.1); margin-bottom: 2rem; }}
        th, td {{ padding: 0.75rem 1rem; text-align: left; border-bottom: 1px solid #e2e8f0; font-size: 0.875rem; }}
        th {{ background: #f1f5f9; font-weight: 600; }}
        .status-pass {{ color: #16a34a; font-weight: 600; }}
        .status-fail {{ color: #dc2626; font-weight: 600; }}
        .individual-report {{ background: white; padding: 1.5rem; border-radius: 0.5rem; margin-bottom: 1.5rem; box-shadow: 0 1px 3px rgba(0,0,0,0.1); }}
        .individual-report h2 {{ font-size: 1.125rem; margin-bottom: 1rem; padding-bottom: 0.75rem; border-bottom: 1px solid #e2e8f0; word-break: break-all; }}
        .module-scores-row {{ display: flex; gap: 1rem; flex-wrap: wrap; margin-bottom: 1rem; }}
        .module-score-chip {{ display: inline-flex; align-items: center; gap: 0.375rem; padding: 0.375rem 0.75rem; background: #f1f5f9; border-radius: 0.375rem; font-size: 0.8125rem; }}
        .module-score-chip .chip-score {{ font-weight: 700; }}
        .module-section {{ margin-bottom: 1rem; padding: 1rem; background: #f8fafc; border-radius: 0.5rem; }}
        .module-section h2 {{ font-size: 1rem; margin-bottom: 0.75rem; padding-bottom: 0.5rem; border-bottom: 1px solid #e2e8f0; display: flex; align-items: center; gap: 0.5rem; }}
        .module-score-badge {{ display: inline-block; padding: 0.125rem 0.5rem; border-radius: 0.75rem; font-size: 0.75rem; font-weight: 700; color: white; }}
        .module-subsection {{ margin-top: 0.75rem; }}
        .module-subsection h3 {{ font-size: 0.8125rem; font-weight: 600; margin-bottom: 0.5rem; }}
        .kv-list {{ display: flex; flex-direction: column; gap: 0.25rem; }}
        .kv-item {{ display: flex; justify-content: space-between; align-items: center; padding: 0.375rem 0; border-bottom: 1px solid #e2e8f0; font-size: 0.8125rem; }}
        .kv-item:last-child {{ border-bottom: none; }}
        .kv-key {{ color: #64748b; }}
        .kv-value {{ font-weight: 600; text-align: right; max-width: 60%; word-break: break-word; }}
        .detail-table {{ width: 100%; border-collapse: collapse; font-size: 0.8125rem; }}
        .detail-table th, .detail-table td {{ padding: 0.5rem 0.625rem; text-align: left; border-bottom: 1px solid #e2e8f0; }}
        .detail-table th {{ background: #f1f5f9; font-weight: 600; color: #64748b; font-size: 0.6875rem; text-transform: uppercase; }}
        .rating-badge {{ display: inline-block; padding: 0.125rem 0.375rem; border-radius: 0.25rem; font-size: 0.6875rem; font-weight: 600; text-transform: uppercase; }}
        .rating-good {{ background: #dcfce7; color: #15803d; }}
        .rating-needs-improvement {{ background: #fef9c3; color: #a16207; }}
        .rating-poor {{ background: #fee2e2; color: #dc2626; }}
        .check-icon {{ color: #16a34a; font-weight: 700; }}
        .cross-icon {{ color: #dc2626; font-weight: 700; }}
        .tag-list {{ display: flex; flex-wrap: wrap; gap: 0.375rem; margin-top: 0.375rem; }}
        .tag {{ display: inline-block; padding: 0.125rem 0.5rem; background: #eff6ff; color: #1d4ed8; border-radius: 0.25rem; font-size: 0.75rem; }}
        .issue-item {{ padding: 0.625rem; border: 1px solid #e2e8f0; border-radius: 0.375rem; margin-bottom: 0.375rem; font-size: 0.8125rem; }}
        .issue-item .issue-header {{ display: flex; align-items: center; gap: 0.375rem; margin-bottom: 0.125rem; }}
        .issue-severity {{ padding: 0.0625rem 0.25rem; border-radius: 0.25rem; font-size: 0.625rem; font-weight: 600; text-transform: uppercase; }}
        .issue-severity.high, .issue-severity.error {{ background: #fee2e2; color: #dc2626; }}
        .issue-severity.medium, .issue-severity.warning {{ background: #fef9c3; color: #a16207; }}
        .issue-severity.low, .issue-severity.info {{ background: #dbeafe; color: #2563eb; }}
        .recommendation-list {{ list-style: none; padding: 0; }}
        .recommendation-list li {{ padding: 0.375rem 0 0.375rem 1.25rem; position: relative; font-size: 0.8125rem; border-bottom: 1px solid #e2e8f0; }}
        .recommendation-list li:last-child {{ border-bottom: none; }}
        .recommendation-list li::before {{ content: "→"; position: absolute; left: 0; color: #2563eb; font-weight: 700; }}
        .overall-scores {{ display: flex; gap: 0.75rem; flex-wrap: wrap; margin-top: 0.75rem; }}
        .overall-score-item {{ text-align: center; padding: 0.75rem; background: #f1f5f9; border-radius: 0.375rem; min-width: 80px; }}
        .overall-score-item .score-value {{ font-size: 1.25rem; font-weight: 700; }}
        .overall-score-item .score-name {{ font-size: 0.625rem; color: #64748b; text-transform: uppercase; margin-top: 0.125rem; }}
        .section {{ background: white; border-radius: 0.5rem; padding: 1rem; margin-bottom: 1rem; }}
        .section h2 {{ font-size: 1rem; font-weight: 600; margin-bottom: 0.75rem; padding-bottom: 0.5rem; border-bottom: 1px solid #e2e8f0; }}
        .violations-list {{ display: flex; flex-direction: column; gap: 0.5rem; }}
        .violation {{ border: 1px solid #e2e8f0; border-radius: 0.375rem; overflow: hidden; }}
        .violation-header {{ display: flex; align-items: center; gap: 0.75rem; padding: 0.75rem; background: #f8fafc; cursor: pointer; font-size: 0.875rem; }}
        .violation-header:hover {{ background: #f1f5f9; }}
        .severity-badge {{ padding: 0.125rem 0.375rem; border-radius: 0.25rem; font-size: 0.6875rem; font-weight: 600; text-transform: uppercase; }}
        .severity-badge.critical {{ background: #fee2e2; color: #dc2626; }}
        .severity-badge.serious {{ background: #ffedd5; color: #ea580c; }}
        .severity-badge.moderate {{ background: #fef9c3; color: #ca8a04; }}
        .severity-badge.minor {{ background: #dbeafe; color: #2563eb; }}
        .violation-rule {{ font-weight: 600; flex: 1; }}
        .violation-rule code {{ background: #f1f5f9; padding: 0.0625rem 0.25rem; border-radius: 0.25rem; font-size: 0.8125rem; }}
        .violation-details {{ padding: 0.75rem; display: none; font-size: 0.8125rem; }}
        .violation.open .violation-details {{ display: block; }}
        .violation-details dl {{ display: grid; grid-template-columns: 100px 1fr; gap: 0.375rem; }}
        .violation-details dt {{ font-weight: 600; color: #64748b; }}
        .violation-details .fix {{ margin-top: 0.75rem; padding: 0.625rem; background: #ecfdf5; border-radius: 0.25rem; border-left: 3px solid #16a34a; }}
        .violation-details .fix strong {{ display: block; margin-bottom: 0.125rem; color: #16a34a; }}
        @media (max-width: 768px) {{
            body {{ padding: 1rem; }}
            .summary {{ grid-template-columns: repeat(2, 1fr); }}
            table {{ font-size: 0.75rem; }}
            th, td {{ padding: 0.5rem; }}
        }}
    </style>
</head>
<body>
    <div class="container">
        <h1>Batch Accessibility Audit Report</h1>
        <div class="summary">
            <div class="summary-item"><div class="value">{total}</div><div class="label">URLs Audited</div></div>
            <div class="summary-item"><div class="value">{passed}/{total}</div><div class="label">Passed</div></div>
            <div class="summary-item"><div class="value">{avg:.0}</div><div class="label">Avg Score</div></div>
            <div class="summary-item"><div class="value">{violations}</div><div class="label">Total Violations</div></div>
        </div>
        <table>
            <thead><tr><th>URL</th><th>WCAG</th><th>Violations</th>{extra_headers}<th>Status</th></tr></thead>
            <tbody>{rows}</tbody>
        </table>
        {individual_reports}
    </div>
    <script>
    document.querySelectorAll('.violation-header').forEach(header => {{
        header.addEventListener('click', () => {{
            header.parentElement.classList.toggle('open');
        }});
    }});
    </script>
</body>
</html>"#,
        total = total,
        passed = passed,
        avg = avg_score,
        violations = total_violations,
        extra_headers = extra_headers,
        rows = rows,
        individual_reports = individual_reports,
    ))
}

fn format_batch_module_scores(report: &AuditReport) -> String {
    let mut chips = Vec::new();
    chips.push(format!(
        r#"<span class="module-score-chip"><span class="chip-score" style="color:{}">{:.0}</span> WCAG</span>"#,
        get_score_color(report.score), report.score,
    ));
    if let Some(ref p) = report.performance {
        chips.push(format!(
            r#"<span class="module-score-chip"><span class="chip-score" style="color:{}">{}</span> Performance</span>"#,
            get_score_color(p.score.overall as f32), p.score.overall,
        ));
    }
    if let Some(ref s) = report.seo {
        chips.push(format!(
            r#"<span class="module-score-chip"><span class="chip-score" style="color:{}">{}</span> SEO</span>"#,
            get_score_color(s.score as f32), s.score,
        ));
    }
    if let Some(ref s) = report.security {
        chips.push(format!(
            r#"<span class="module-score-chip"><span class="chip-score" style="color:{}">{}</span> Security</span>"#,
            get_score_color(s.score as f32), s.score,
        ));
    }
    if let Some(ref m) = report.mobile {
        chips.push(format!(
            r#"<span class="module-score-chip"><span class="chip-score" style="color:{}">{}</span> Mobile</span>"#,
            get_score_color(m.score as f32), m.score,
        ));
    }
    chips.join("\n")
}

fn url_to_id(url: &str) -> String {
    url.chars()
        .filter(|c| c.is_alphanumeric())
        .collect::<String>()
        .chars()
        .take(32)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::WcagLevel;
    use crate::wcag::WcagResults;

    #[test]
    fn test_format_html() {
        let report = AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            WcagResults::new(),
            1500,
        );

        let html = format_html(&report, "AA").unwrap();
        assert!(html.contains("example.com"));
        assert!(html.contains("WCAG AA"));
    }

    #[test]
    fn test_html_escape() {
        assert_eq!(
            html_escape("<script>alert('xss')</script>"),
            "&lt;script&gt;alert(&#39;xss&#39;)&lt;/script&gt;"
        );
    }

    #[test]
    fn test_get_score_color() {
        assert_eq!(get_score_color(95.0), "#16a34a");
        assert_eq!(get_score_color(75.0), "#ca8a04");
        assert_eq!(get_score_color(55.0), "#ea580c");
        assert_eq!(get_score_color(30.0), "#dc2626");
    }
}
