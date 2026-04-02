//! Audit artifact persistence.
//!
//! Stores fetch/snapshot/audit artifacts under ~/.auditmysite/cache for reuse
//! and later benchmark/delta analysis.

use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::accessibility::AXTree;
use crate::audit::normalized::NormalizedReport;
use crate::audit::PerformanceResults;
use crate::audit::{AccessibilityScorer, AuditReport};
use crate::cli::WcagLevel;
use crate::error::Result;
use crate::mobile::MobileFriendliness;
use crate::security::SecurityAnalysis;
use crate::seo::SeoAnalysis;
use crate::wcag::{Violation, WcagResults};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchArtifact {
    pub requested_url: String,
    pub final_url: String,
    pub status_code: Option<u16>,
    pub fetched_at: DateTime<Utc>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotArtifact {
    pub ax_tree: AXTree,
    pub performance: Option<PerformanceResults>,
    pub seo: Option<SeoAnalysis>,
    pub security: Option<SecurityAnalysis>,
    pub mobile: Option<MobileFriendliness>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditArtifacts {
    pub fetch: FetchArtifact,
    pub snapshot: SnapshotArtifact,
    pub audit: NormalizedReport,
    pub content_hash: String,
}

pub fn save_artifacts(url: &str, artifacts: &AuditArtifacts) -> Result<PathBuf> {
    let dir = artifact_dir(url)?;
    fs::create_dir_all(&dir)?;

    fs::write(
        dir.join("fetch.json"),
        serde_json::to_vec_pretty(&artifacts.fetch)?,
    )?;
    fs::write(
        dir.join("snapshot.json"),
        serde_json::to_vec_pretty(&artifacts.snapshot)?,
    )?;
    fs::write(
        dir.join("audit.json"),
        serde_json::to_vec_pretty(&artifacts.audit)?,
    )?;

    Ok(dir)
}

pub fn load_artifacts(url: &str) -> Result<Option<AuditArtifacts>> {
    let dir = artifact_dir(url)?;
    if !dir.exists() {
        return Ok(None);
    }

    let fetch_path = dir.join("fetch.json");
    let snapshot_path = dir.join("snapshot.json");
    let audit_path = dir.join("audit.json");

    if !fetch_path.exists() || !snapshot_path.exists() || !audit_path.exists() {
        return Ok(None);
    }

    let fetch: FetchArtifact = serde_json::from_slice(&fs::read(fetch_path)?)?;
    let snapshot: SnapshotArtifact = serde_json::from_slice(&fs::read(snapshot_path)?)?;
    let audit: NormalizedReport = serde_json::from_slice(&fs::read(audit_path)?)?;
    let content_hash = content_hash(&snapshot);

    Ok(Some(AuditArtifacts {
        fetch,
        snapshot,
        audit,
        content_hash,
    }))
}

pub fn content_hash(snapshot: &SnapshotArtifact) -> String {
    let mut hasher = DefaultHasher::new();
    snapshot.ax_tree.len().hash(&mut hasher);
    snapshot
        .seo
        .as_ref()
        .and_then(|s| s.meta.title.as_ref())
        .hash(&mut hasher);
    snapshot
        .seo
        .as_ref()
        .and_then(|s| s.meta.description.as_ref())
        .hash(&mut hasher);
    snapshot
        .seo
        .as_ref()
        .map(|s| s.headings.total_count)
        .hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

pub fn to_audit_report(artifacts: &AuditArtifacts) -> AuditReport {
    let mut wcag_results = WcagResults::new();
    wcag_results.nodes_checked = artifacts.snapshot.ax_tree.len();

    for finding in &artifacts.audit.findings {
        let level = parse_wcag_level(&finding.wcag_level);
        for occ in &finding.occurrences {
            let mut violation = Violation::new(
                finding.wcag_criterion.clone(),
                finding.title.clone(),
                level,
                finding.severity,
                occ.message.clone(),
                occ.node_id.clone(),
            );
            if let Some(selector) = &occ.selector {
                violation = violation.with_selector(selector.clone());
            }
            if let Some(fix) = &occ.fix_suggestion {
                violation = violation.with_fix(fix.clone());
            }
            wcag_results.add_violation(violation);
        }
    }

    let stats = AccessibilityScorer::calculate_statistics(&wcag_results.violations);
    let mut report = AuditReport {
        url: artifacts.audit.url.clone(),
        wcag_level: artifacts.audit.wcag_level,
        timestamp: artifacts.audit.timestamp,
        wcag_results,
        score: artifacts.audit.score as f32,
        grade: artifacts.audit.grade.clone(),
        certificate: artifacts.audit.certificate.clone(),
        statistics: stats,
        nodes_analyzed: artifacts.audit.nodes_analyzed,
        duration_ms: artifacts.audit.duration_ms,
        performance: artifacts.snapshot.performance.clone(),
        seo: artifacts.snapshot.seo.clone(),
        security: artifacts.snapshot.security.clone(),
        mobile: artifacts.snapshot.mobile.clone(),
        budget_violations: Vec::new(),
        dark_mode: None,
    };

    // Keep consistency with what a live run would expose on report-level metadata.
    if report.wcag_level != artifacts.audit.wcag_level {
        report.wcag_level = artifacts.audit.wcag_level;
    }

    report
}

fn parse_wcag_level(level: &str) -> WcagLevel {
    match level {
        "A" => WcagLevel::A,
        "AA" => WcagLevel::AA,
        "AAA" => WcagLevel::AAA,
        _ => WcagLevel::AA,
    }
}

fn artifact_dir(url: &str) -> Result<PathBuf> {
    let parsed = url::Url::parse(url)?;
    let domain = parsed.host_str().unwrap_or("unknown");
    let mut hasher = DefaultHasher::new();
    url.hash(&mut hasher);
    let url_hash = format!("{:016x}", hasher.finish());

    let base = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".auditmysite")
        .join("cache")
        .join(domain);

    Ok(base.join(url_hash))
}
