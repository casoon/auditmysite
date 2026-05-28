//! YAML export of AXTree snapshots and journey traces for developer debugging.
//!
//! The output format is compatible with Playwright's ARIA snapshot YAML schema,
//! making it useful for CI regression testing.

use crate::audit::normalized::{JourneyStep, JourneyTrace};
use crate::audit::AuditReport;
use anyhow::Result;
use std::path::Path;

/// Export accessibility journey data for a single-page audit to YAML.
///
/// The output includes: page URL, audit timestamp, and all journey traces
/// with their action steps and focus snapshots.
pub fn export_snapshot_yaml(report: &AuditReport, path: &Path) -> Result<()> {
    let yaml = build_snapshot_yaml(report);
    std::fs::write(path, yaml)?;
    Ok(())
}

fn build_snapshot_yaml(report: &AuditReport) -> String {
    let mut out = String::new();
    out.push_str("# AuditMySite — Accessibility Journey Snapshot\n");
    out.push_str("# Generated for CI regression testing / developer debugging\n");
    out.push_str("# Compatible with Playwright ARIA snapshot format\n");
    out.push('\n');

    out.push_str(&format!("url: {}\n", escape_yaml_string(&report.url)));
    out.push_str(&format!(
        "timestamp: {}\n",
        report.timestamp.format("%Y-%m-%dT%H:%M:%SZ")
    ));

    if let Some(ref journey) = report.accessibility_journey {
        out.push_str(&format!("journey_count: {}\n", journey.traces.len()));
        out.push('\n');
        out.push_str("journeys:\n");
        for trace in &journey.traces {
            out.push_str(&render_trace_yaml(trace));
        }
    } else {
        out.push_str("journey_count: 0\n");
        out.push_str("# No journey traces recorded (--interactive=off or no patterns found)\n");
    }

    if !report.interactive_findings.is_empty() {
        out.push('\n');
        out.push_str(&format!(
            "interactive_finding_count: {}\n",
            report.interactive_findings.len()
        ));
        out.push_str("interactive_findings:\n");
        for f in &report.interactive_findings {
            out.push_str(&format!("  - category: {}\n", f.category));
            let sev_lower = format!("{:?}", f.severity).to_lowercase();
            out.push_str(&format!("    severity: {}\n", sev_lower));
            out.push_str(&format!("    journey: {}\n", f.journey));
            out.push_str(&format!(
                "    message: {}\n",
                escape_yaml_string(&f.message)
            ));
            if let Some(ref fix) = f.fix_suggestion {
                out.push_str(&format!(
                    "    fix_suggestion: {}\n",
                    escape_yaml_string(fix)
                ));
            }
        }
    }

    out
}

fn render_trace_yaml(trace: &JourneyTrace) -> String {
    let mut out = String::new();
    out.push_str(&format!("  - journey: {}\n", trace.journey));
    out.push_str(&format!("    steps: {}\n", trace.steps.len()));
    out.push_str("    sequence:\n");
    for step in &trace.steps {
        out.push_str(&render_step_yaml(step));
    }
    out
}

fn render_step_yaml(step: &JourneyStep) -> String {
    let mut out = String::new();
    out.push_str(&format!("      - action: {}\n", step.action));
    if let Some(ref target) = step.target {
        out.push_str(&format!("        target: {}\n", escape_yaml_string(target)));
    }
    if let Some(ref focus) = step.focus {
        out.push_str(&format!("        focus: {}\n", escape_yaml_string(focus)));
    }
    if let Some(ref result) = step.result {
        out.push_str(&format!("        result: {}\n", result));
    }
    if let Some(ref label) = step.snapshot_label {
        out.push_str(&format!("        snapshot: {}\n", label));
    }
    out
}

/// Minimal YAML string escaping — wraps strings containing special chars in quotes.
fn escape_yaml_string(s: &str) -> String {
    if s.contains(':')
        || s.contains('#')
        || s.contains('\n')
        || s.contains('"')
        || s.contains('\'')
        || s.starts_with(' ')
        || s.is_empty()
    {
        format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
    } else {
        s.to_string()
    }
}
