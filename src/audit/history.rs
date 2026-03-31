//! Convention-based report history for files collected in `reports/`.
//!
//! The history model is built from normalized JSON report snapshots that live
//! next to generated PDF/JSON artifacts. No database is required.

use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::audit::normalized::{ModuleScoreEntry, NormalizedReport, SeverityCounts};
use crate::error::{AuditError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryFindingSummary {
    pub rule_id: String,
    pub title: String,
    pub occurrence_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistorySnapshot {
    pub source_file: String,
    pub url: String,
    pub host: String,
    pub timestamp: DateTime<Utc>,
    pub accessibility_score: u32,
    pub overall_score: u32,
    pub grade: String,
    pub certificate: String,
    pub nodes_analyzed: usize,
    pub duration_ms: u64,
    pub severity_counts: SeverityCounts,
    pub module_scores: Vec<ModuleScoreEntry>,
    pub top_findings: Vec<HistoryFindingSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryDelta {
    pub accessibility_score_delta: i32,
    pub overall_score_delta: i32,
    pub total_issues_delta: i32,
    pub critical_issues_delta: i32,
    pub new_findings: Vec<String>,
    pub resolved_findings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportHistory {
    pub subject: String,
    pub host: String,
    pub url: String,
    pub generated_at: DateTime<Utc>,
    pub entry_count: usize,
    pub latest: HistorySnapshot,
    pub previous: Option<HistorySnapshot>,
    pub latest_delta: Option<HistoryDelta>,
    pub entries: Vec<HistorySnapshot>,
}

pub struct HistoryPreview {
    pub previous_date: String,
    pub timeline_entries: usize,
    pub previous_accessibility_score: u32,
    pub previous_overall_score: u32,
    pub delta: HistoryDelta,
    pub recent_entries: Vec<HistorySnapshot>,
}

#[derive(Debug, Deserialize)]
struct StoredJsonReport {
    report: StoredNormalizedReport,
}

#[derive(Debug, Deserialize)]
struct StoredNormalizedReport {
    url: String,
    timestamp: DateTime<Utc>,
    score: u32,
    overall_score: u32,
    grade: String,
    certificate: String,
    nodes_analyzed: usize,
    duration_ms: u64,
    severity_counts: SeverityCounts,
    module_scores: Vec<ModuleScoreEntry>,
    findings: Vec<StoredFinding>,
}

#[derive(Debug, Deserialize)]
struct StoredFinding {
    rule_id: String,
    title: String,
    occurrence_count: usize,
}

pub fn write_report_history(
    reports_dir: &Path,
    output_path: &Path,
    normalized: &NormalizedReport,
) -> Result<Vec<PathBuf>> {
    fs::create_dir_all(reports_dir)?;

    let host = host_from_url(&normalized.url)?;
    let subject = derive_history_subject(output_path, &host);
    let current_snapshot = snapshot_from_normalized(normalized, output_path, &host);

    let mut entries = load_history_entries(reports_dir, &host)?;
    entries.retain(|entry| entry.timestamp != current_snapshot.timestamp);
    entries.push(current_snapshot);
    entries = dedupe_snapshots(entries);

    let latest = entries
        .last()
        .cloned()
        .ok_or_else(|| AuditError::ConfigError("History entries unexpectedly empty".to_string()))?;
    let previous = if entries.len() > 1 {
        entries.get(entries.len() - 2).cloned()
    } else {
        None
    };
    let latest_delta = previous.as_ref().map(|prev| build_delta(prev, &latest));

    let history = ReportHistory {
        subject: subject.clone(),
        host,
        url: normalized.url.clone(),
        generated_at: Utc::now(),
        entry_count: entries.len(),
        latest,
        previous,
        latest_delta,
        entries,
    };

    let json_path = reports_dir.join(format!("{subject}-history.json"));
    let md_path = reports_dir.join(format!("{subject}-history.md"));

    fs::write(
        &json_path,
        serde_json::to_vec_pretty(&history).map_err(|e| AuditError::OutputError {
            reason: format!("Failed to serialize history JSON: {e}"),
        })?,
    )?;
    fs::write(&md_path, render_history_markdown(&history))?;

    Ok(vec![json_path, md_path])
}

pub fn preview_report_history(
    reports_dir: &Path,
    _output_path: &Path,
    normalized: &NormalizedReport,
) -> Result<Option<HistoryPreview>> {
    let host = host_from_url(&normalized.url)?;
    let current_snapshot = snapshot_from_normalized(normalized, Path::new("preview"), &host);
    let entries = load_history_entries(reports_dir, &host)?;
    let previous = match entries.iter().rev().find(|entry| {
        entry.timestamp.date_naive() != current_snapshot.timestamp.date_naive()
            && snapshot_fingerprint(entry) != snapshot_fingerprint(&current_snapshot)
    }) {
        Some(entry) => entry,
        None => return Ok(None),
    };
    let mut recent_entries = entries.clone();
    recent_entries.push(current_snapshot.clone());
    recent_entries = dedupe_snapshots(recent_entries);
    let recent_entries = recent_entries.into_iter().rev().take(5).collect::<Vec<_>>();

    Ok(Some(HistoryPreview {
        previous_date: previous.timestamp.format("%d.%m.%Y").to_string(),
        timeline_entries: recent_entries.len(),
        previous_accessibility_score: previous.accessibility_score,
        previous_overall_score: previous.overall_score,
        delta: build_delta(previous, &current_snapshot),
        recent_entries,
    }))
}

fn load_history_entries(reports_dir: &Path, host: &str) -> Result<Vec<HistorySnapshot>> {
    let mut entries = Vec::new();

    for dir_entry in fs::read_dir(reports_dir)? {
        let path = dir_entry?.path();
        if !path.is_file() || path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }

        let stem = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("");
        if stem.ends_with("-history") {
            continue;
        }

        let bytes = match fs::read(&path) {
            Ok(bytes) => bytes,
            Err(_) => continue,
        };
        let stored: StoredJsonReport = match serde_json::from_slice(&bytes) {
            Ok(report) => report,
            Err(_) => continue,
        };
        let stored_host = match host_from_url(&stored.report.url) {
            Ok(value) => value,
            Err(_) => continue,
        };
        if stored_host != host {
            continue;
        }

        entries.push(HistorySnapshot {
            source_file: path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default()
                .to_string(),
            url: stored.report.url,
            host: stored_host,
            timestamp: stored.report.timestamp,
            accessibility_score: stored.report.score,
            overall_score: stored.report.overall_score,
            grade: stored.report.grade,
            certificate: stored.report.certificate,
            nodes_analyzed: stored.report.nodes_analyzed,
            duration_ms: stored.report.duration_ms,
            severity_counts: stored.report.severity_counts,
            module_scores: stored.report.module_scores,
            top_findings: stored
                .report
                .findings
                .into_iter()
                .take(5)
                .map(|finding| HistoryFindingSummary {
                    rule_id: finding.rule_id,
                    title: finding.title,
                    occurrence_count: finding.occurrence_count,
                })
                .collect(),
        });
    }

    Ok(dedupe_snapshots(entries))
}

fn snapshot_from_normalized(
    normalized: &NormalizedReport,
    output_path: &Path,
    host: &str,
) -> HistorySnapshot {
    HistorySnapshot {
        source_file: output_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_string(),
        url: normalized.url.clone(),
        host: host.to_string(),
        timestamp: normalized.timestamp,
        accessibility_score: normalized.score,
        overall_score: normalized.overall_score,
        grade: normalized.grade.clone(),
        certificate: normalized.certificate.clone(),
        nodes_analyzed: normalized.nodes_analyzed,
        duration_ms: normalized.duration_ms,
        severity_counts: normalized.severity_counts.clone(),
        module_scores: normalized.module_scores.clone(),
        top_findings: normalized
            .findings
            .iter()
            .take(5)
            .map(|finding| HistoryFindingSummary {
                rule_id: finding.rule_id.clone(),
                title: finding.title.clone(),
                occurrence_count: finding.occurrence_count,
            })
            .collect(),
    }
}

fn build_delta(previous: &HistorySnapshot, latest: &HistorySnapshot) -> HistoryDelta {
    let prev_rules: HashMap<&str, &HistoryFindingSummary> = previous
        .top_findings
        .iter()
        .map(|finding| (finding.rule_id.as_str(), finding))
        .collect();
    let latest_rules: HashMap<&str, &HistoryFindingSummary> = latest
        .top_findings
        .iter()
        .map(|finding| (finding.rule_id.as_str(), finding))
        .collect();

    let new_findings = latest_rules
        .keys()
        .filter(|rule_id| !prev_rules.contains_key(**rule_id))
        .filter_map(|rule_id| {
            latest_rules
                .get(rule_id)
                .map(|finding| finding.title.clone())
        })
        .collect();

    let resolved_findings = prev_rules
        .keys()
        .filter(|rule_id| !latest_rules.contains_key(**rule_id))
        .filter_map(|rule_id| prev_rules.get(rule_id).map(|finding| finding.title.clone()))
        .collect();

    HistoryDelta {
        accessibility_score_delta: latest.accessibility_score as i32
            - previous.accessibility_score as i32,
        overall_score_delta: latest.overall_score as i32 - previous.overall_score as i32,
        total_issues_delta: latest.severity_counts.total as i32
            - previous.severity_counts.total as i32,
        critical_issues_delta: (latest.severity_counts.critical + latest.severity_counts.high)
            as i32
            - (previous.severity_counts.critical + previous.severity_counts.high) as i32,
        new_findings,
        resolved_findings,
    }
}

fn dedupe_snapshots(mut entries: Vec<HistorySnapshot>) -> Vec<HistorySnapshot> {
    entries.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

    let mut by_fingerprint: HashMap<String, HistorySnapshot> = HashMap::new();
    for entry in entries {
        let key = snapshot_fingerprint(&entry);
        match by_fingerprint.get(&key) {
            Some(existing) if existing.timestamp >= entry.timestamp => {}
            _ => {
                by_fingerprint.insert(key, entry);
            }
        }
    }

    let mut deduped: Vec<HistorySnapshot> = by_fingerprint.into_values().collect();
    deduped.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
    collapse_same_day(deduped)
}

fn snapshot_fingerprint(entry: &HistorySnapshot) -> String {
    let rule_ids = entry
        .top_findings
        .iter()
        .map(|finding| format!("{}:{}", finding.rule_id, finding.occurrence_count))
        .collect::<Vec<_>>()
        .join("|");

    format!(
        "{}|{}|{}|{}|{}|{}|{}|{}|{}",
        entry.host,
        entry.url,
        entry.accessibility_score,
        entry.overall_score,
        entry.severity_counts.critical,
        entry.severity_counts.high,
        entry.severity_counts.medium,
        entry.severity_counts.low,
        rule_ids
    )
}

fn collapse_same_day(entries: Vec<HistorySnapshot>) -> Vec<HistorySnapshot> {
    let mut by_day: HashMap<String, HistorySnapshot> = HashMap::new();

    for entry in entries {
        let day = entry.timestamp.format("%Y-%m-%d").to_string();
        match by_day.get(&day) {
            Some(existing) if existing.timestamp >= entry.timestamp => {}
            _ => {
                by_day.insert(day, entry);
            }
        }
    }

    let mut collapsed: Vec<HistorySnapshot> = by_day.into_values().collect();
    collapsed.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
    collapsed
}

fn render_history_markdown(history: &ReportHistory) -> String {
    let mut out = String::new();
    out.push_str(&format!("# Report History: {}\n\n", history.subject));
    out.push_str(&format!("- Host: `{}`\n", history.host));
    out.push_str(&format!("- URL: `{}`\n", history.url));
    out.push_str(&format!("- Einträge: `{}`\n", history.entry_count));
    out.push_str(&format!(
        "- Generiert: `{}`\n\n",
        history.generated_at.format("%Y-%m-%d %H:%M:%S UTC")
    ));

    out.push_str("## Aktueller Stand\n\n");
    out.push_str(&format!(
        "- Accessibility: `{}`\n- Website gesamt: `{}`\n- Note/Zertifikat: `{}` / `{}`\n- Issues gesamt: `{}`\n- Kritisch+Hoch: `{}`\n\n",
        history.latest.accessibility_score,
        history.latest.overall_score,
        history.latest.grade,
        history.latest.certificate,
        history.latest.severity_counts.total,
        history.latest.severity_counts.critical + history.latest.severity_counts.high,
    ));

    if let Some(delta) = &history.latest_delta {
        out.push_str("## Veränderung zum letzten Lauf\n\n");
        out.push_str(&format!(
            "- Accessibility-Delta: `{:+}`\n- Gesamt-Delta: `{:+}`\n- Issue-Delta: `{:+}`\n- Kritisch+Hoch-Delta: `{:+}`\n",
            delta.accessibility_score_delta,
            delta.overall_score_delta,
            delta.total_issues_delta,
            delta.critical_issues_delta,
        ));
        if !delta.new_findings.is_empty() {
            out.push_str(&format!(
                "- Neue Findings: {}\n",
                delta.new_findings.join(", ")
            ));
        }
        if !delta.resolved_findings.is_empty() {
            out.push_str(&format!(
                "- Behobene Findings: {}\n",
                delta.resolved_findings.join(", ")
            ));
        }
        out.push('\n');
    }

    out.push_str("## Verlauf\n\n");
    out.push_str("| Datum | Accessibility | Gesamt | Note | Zertifikat | Issues |\n");
    out.push_str("| --- | ---: | ---: | --- | --- | ---: |\n");
    for entry in &history.entries {
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} |\n",
            entry.timestamp.format("%Y-%m-%d"),
            entry.accessibility_score,
            entry.overall_score,
            entry.grade,
            entry.certificate,
            entry.severity_counts.total
        ));
    }

    let all_rules: BTreeSet<String> = history
        .entries
        .iter()
        .flat_map(|entry| {
            entry
                .top_findings
                .iter()
                .map(|finding| finding.title.clone())
        })
        .collect();
    if !all_rules.is_empty() {
        out.push_str("\n## Häufige Findings im Verlauf\n\n");
        for rule in all_rules {
            let appearances = history
                .entries
                .iter()
                .filter(|entry| {
                    entry
                        .top_findings
                        .iter()
                        .any(|finding| finding.title == rule)
                })
                .count();
            out.push_str(&format!("- {} (`{}` Läufe)\n", rule, appearances));
        }
    }

    out
}

fn host_from_url(url: &str) -> Result<String> {
    let parsed = Url::parse(url)
        .map_err(|e| AuditError::ConfigError(format!("Invalid URL for history: {e}")))?;
    parsed
        .host_str()
        .map(|host| host.to_string())
        .ok_or_else(|| AuditError::ConfigError("URL has no host for history".to_string()))
}

fn derive_history_subject(output_path: &Path, host: &str) -> String {
    let fallback = host.replace('.', "-");
    let stem = match output_path.file_stem().and_then(|stem| stem.to_str()) {
        Some(stem) if !stem.is_empty() => stem,
        _ => return fallback,
    };

    let parts: Vec<&str> = stem.split('-').collect();
    for idx in 0..parts.len().saturating_sub(2) {
        let year = parts[idx];
        let month = parts[idx + 1];
        let day = parts[idx + 2];
        if year.len() == 4
            && month.len() == 2
            && day.len() == 2
            && year.chars().all(|c| c.is_ascii_digit())
            && month.chars().all(|c| c.is_ascii_digit())
            && day.chars().all(|c| c.is_ascii_digit())
        {
            let prefix = parts[..idx].join("-");
            if !prefix.is_empty() {
                return prefix;
            }
        }
    }

    if stem == "audit-report" || stem == "batch-audit-report" {
        fallback
    } else {
        stem.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use tempfile::tempdir;

    #[test]
    fn test_derive_history_subject_from_dated_filename() {
        let subject = derive_history_subject(
            Path::new("reports/casoon-2026-03-31-standard.pdf"),
            "www.casoon.de",
        );
        assert_eq!(subject, "casoon");
    }

    #[test]
    fn test_write_report_history_builds_timeline_from_existing_json_reports() {
        let dir = tempdir().unwrap();
        let reports_dir = dir.path();

        let existing = serde_json::json!({
            "metadata": {
                "tool": "auditmysite v0.4.2",
                "timestamp": "2026-03-01T08:00:00Z",
                "wcag_level": "AA",
                "execution_time_ms": 1000
            },
            "report": {
                "url": "https://www.casoon.de",
                "timestamp": "2026-03-01T08:00:00Z",
                "score": 88,
                "overall_score": 82,
                "grade": "A",
                "certificate": "GOLD",
                "nodes_analyzed": 1000,
                "duration_ms": 1000,
                "severity_counts": {
                    "critical": 1,
                    "high": 1,
                    "medium": 2,
                    "low": 0,
                    "total": 4
                },
                "module_scores": [],
                "findings": [
                    {
                        "rule_id": "a11y.alt_text.missing",
                        "title": "Non-text Content",
                        "occurrence_count": 2
                    }
                ]
            }
        });
        fs::write(
            reports_dir.join("casoon-2026-03-01.json"),
            serde_json::to_vec_pretty(&existing).unwrap(),
        )
        .unwrap();

        let normalized = NormalizedReport {
            url: "https://www.casoon.de".to_string(),
            wcag_level: crate::cli::WcagLevel::AA,
            timestamp: Utc.with_ymd_and_hms(2026, 3, 31, 10, 0, 0).unwrap(),
            duration_ms: 1500,
            nodes_analyzed: 1500,
            score: 92,
            grade: "A".to_string(),
            certificate: "GOLD".to_string(),
            findings: vec![crate::audit::normalized::NormalizedFinding {
                rule_id: "a11y.alt_text.missing".to_string(),
                wcag_criterion: "1.1.1".to_string(),
                wcag_level: "A".to_string(),
                dimension: "Accessibility".to_string(),
                subcategory: "Inhalte".to_string(),
                issue_class: "Fehlend".to_string(),
                severity: crate::taxonomy::Severity::High,
                user_impact: "".to_string(),
                technical_impact: "".to_string(),
                score_impact: crate::audit::normalized::ScoreImpactData {
                    base_penalty: 1.0,
                    max_penalty: 2.0,
                    scaling: "logarithmic".to_string(),
                },
                report_visibility: crate::audit::normalized::ReportVisibilityData::default(),
                aggregation_key: "a11y.alt_text.missing".to_string(),
                title: "Non-text Content".to_string(),
                description: "Missing alt".to_string(),
                occurrence_count: 1,
                priority_score: 1.0,
                occurrences: vec![],
            }],
            severity_counts: SeverityCounts {
                critical: 0,
                high: 1,
                medium: 0,
                low: 0,
                total: 1,
            },
            module_scores: vec![],
            overall_score: 86,
            raw_performance: None,
            raw_seo: None,
            raw_security: None,
            raw_mobile: None,
            raw_wcag: crate::wcag::WcagResults::new(),
        };

        let written = write_report_history(
            reports_dir,
            &reports_dir.join("casoon-2026-03-31-standard.pdf"),
            &normalized,
        )
        .unwrap();

        assert_eq!(written.len(), 2);
        let history_json = fs::read_to_string(reports_dir.join("casoon-history.json")).unwrap();
        assert!(history_json.contains("\"entry_count\": 2"));
        assert!(history_json.contains("\"accessibility_score_delta\": 4"));

        let history_md = fs::read_to_string(reports_dir.join("casoon-history.md")).unwrap();
        assert!(history_md.contains("Veränderung zum letzten Lauf"));
        assert!(history_md.contains("| 2026-03-01 | 88 | 82 |"));
        assert!(history_md.contains("| 2026-03-31 | 92 | 86 |"));
    }
}
