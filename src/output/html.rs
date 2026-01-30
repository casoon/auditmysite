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
        <p style="margin-top: 1rem; color: var(--color-text-muted);">
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

        let items: String = violations.iter().map(|v| render_violation(v)).collect();

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
        severity = format!("{:?}", v.severity),
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

    let rows: String = reports.iter().map(|r| {
        let status = if r.passed() { "pass" } else { "fail" };
        let url_id = url_to_id(&r.url);
        let url = html_escape(&r.url);
        let status_text = if r.passed() { "Pass" } else { "Fail" };
        format!(
            "<tr class=\"{status}\">\n    <td><a href=\"#{url_id}\">{url}</a></td>\n    <td>{score}</td>\n    <td>{violations}</td>\n    <td class=\"status-{status}\">{status_text}</td>\n</tr>",
            status = status,
            url_id = url_id,
            url = url,
            score = r.score,
            violations = r.violation_count(),
            status_text = status_text,
        )
    }).collect();

    let individual_reports: String = reports
        .iter()
        .map(|r| {
            format!(
                r#"<div id="{url_id}" class="individual-report">
    <h2>{url}</h2>
    {content}
</div>"#,
                url_id = url_to_id(&r.url),
                url = html_escape(&r.url),
                content = HtmlReport::new(r, wcag_level).render_violations_list(),
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
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #f8fafc; margin: 0; padding: 2rem; }}
        .container {{ max-width: 1200px; margin: 0 auto; }}
        h1 {{ margin-bottom: 2rem; }}
        .summary {{ display: grid; grid-template-columns: repeat(4, 1fr); gap: 1rem; margin-bottom: 2rem; }}
        .summary-item {{ background: white; padding: 1.5rem; border-radius: 0.5rem; box-shadow: 0 1px 3px rgba(0,0,0,0.1); }}
        .summary-item .value {{ font-size: 2rem; font-weight: 700; }}
        .summary-item .label {{ color: #64748b; font-size: 0.875rem; }}
        table {{ width: 100%; border-collapse: collapse; background: white; border-radius: 0.5rem; overflow: hidden; box-shadow: 0 1px 3px rgba(0,0,0,0.1); margin-bottom: 2rem; }}
        th, td {{ padding: 1rem; text-align: left; border-bottom: 1px solid #e2e8f0; }}
        th {{ background: #f1f5f9; font-weight: 600; }}
        .status-pass {{ color: #16a34a; }}
        .status-fail {{ color: #dc2626; }}
        .individual-report {{ background: white; padding: 1.5rem; border-radius: 0.5rem; margin-bottom: 1rem; box-shadow: 0 1px 3px rgba(0,0,0,0.1); }}
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
            <thead><tr><th>URL</th><th>Score</th><th>Violations</th><th>Status</th></tr></thead>
            <tbody>{rows}</tbody>
        </table>
        {individual_reports}
    </div>
</body>
</html>"#,
        total = total,
        passed = passed,
        avg = avg_score,
        violations = total_violations,
        rows = rows,
        individual_reports = individual_reports,
    ))
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
    use crate::wcag::WcagResults;

    #[test]
    fn test_format_html() {
        let report = AuditReport::new("https://example.com".to_string(), WcagResults::new(), 1500);

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
