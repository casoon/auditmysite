//! Audit artifact persistence.
//!
//! Stores fetch/snapshot/audit artifacts under ~/.auditmysite/cache for reuse
//! and later benchmark/delta analysis.
//!
//! Cache layout:
//!   ~/.auditmysite/cache/{domain}/{url_hash}/v{VERSION}/
//!     fetch.json     — HTTP fetch metadata
//!     snapshot.json  — AXTree + module raw data
//!     audit.json     — NormalizedReport (rule results, scoring)
//!     meta.json      — version + wcag_level + timestamp (for diagnostics)
//!
//! Invalidation rules:
//!   - Binary version bump: new VERSION subdirectory → old entries silently ignored
//!   - URL change: new url_hash → separate directory
//!   - WCAG level: stored in meta.json; callers can check via `artifacts.meta.wcag_level`
//!   - Content change: content_hash() fingerprints AXTree + SEO for delta detection
//!
//! Hash stability: FNV-1a 64-bit (deterministic across processes and platforms).
//! DefaultHasher is explicitly NOT used — it is non-deterministic by design.

use std::fs;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::warn;

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

// ─── Cache metadata ──────────────────────────────────────────────────────────

/// Persisted alongside each cache entry for diagnostics and validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMeta {
    /// Binary version that wrote this entry — matches the VERSION directory component.
    pub auditmysite_version: String,
    /// WCAG level used for this audit run.
    pub wcag_level: String,
    /// When the snapshot was fetched.
    pub cached_at: DateTime<Utc>,
    /// FNV-1a fingerprint of the AXTree at cache time.
    pub content_hash: String,
    /// Fingerprint of the audit-relevant configuration (WCAG level + active
    /// modules + consent handling). Used to reject cache reuse when the current
    /// run requests a different audit scope. Empty for entries written before
    /// this field existed — such entries are never reused.
    #[serde(default)]
    pub audit_signature: String,
}

/// Whether a cached entry was produced with an audit configuration compatible
/// with the current run. An empty stored signature (legacy entry) never matches.
pub fn cache_matches_signature(meta: &CacheMeta, expected: &str) -> bool {
    !meta.audit_signature.is_empty() && meta.audit_signature == expected
}

// ─── Artifact types ───────────────────────────────────────────────────────────

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
    /// The complete `AuditReport` (screenshots stripped), persisted so a cache
    /// hit can render every module section faithfully instead of through the
    /// lossy `to_audit_report` reconstruction. `None` for legacy entries
    /// written before this field existed — those fall back to `to_audit_report`.
    #[serde(default)]
    pub report: Option<AuditReport>,
    pub content_hash: String,
    pub meta: CacheMeta,
}

// ─── Public API ───────────────────────────────────────────────────────────────

pub fn save_artifacts(url: &str, wcag_level: &str, artifacts: &AuditArtifacts) -> Result<PathBuf> {
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
    if let Some(ref report) = artifacts.report {
        fs::write(dir.join("report.json"), serde_json::to_vec_pretty(report)?)?;
    }

    let meta = CacheMeta {
        auditmysite_version: env!("CARGO_PKG_VERSION").to_string(),
        wcag_level: wcag_level.to_string(),
        cached_at: Utc::now(),
        content_hash: artifacts.content_hash.clone(),
        audit_signature: artifacts.meta.audit_signature.clone(),
    };
    fs::write(dir.join("meta.json"), serde_json::to_vec_pretty(&meta)?)?;

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
    let report_path = dir.join("report.json");
    let meta_path = dir.join("meta.json");

    if !fetch_path.exists() || !snapshot_path.exists() || !audit_path.exists() {
        return Ok(None);
    }

    // A truncated or otherwise corrupt cache entry must not abort the run — treat
    // any read/parse failure as a cache miss so the caller falls back to a fresh
    // audit (#405).
    macro_rules! load_or_miss {
        ($path:expr, $ty:ty) => {
            match fs::read(&$path)
                .map_err(|e| e.to_string())
                .and_then(|bytes| serde_json::from_slice::<$ty>(&bytes).map_err(|e| e.to_string()))
            {
                Ok(value) => value,
                Err(e) => {
                    warn!(
                        "Cache entry {} is unreadable ({}); ignoring and running fresh",
                        $path.display(),
                        e
                    );
                    return Ok(None);
                }
            }
        };
    }

    let fetch: FetchArtifact = load_or_miss!(fetch_path, FetchArtifact);
    let snapshot: SnapshotArtifact = load_or_miss!(snapshot_path, SnapshotArtifact);
    let audit: NormalizedReport = load_or_miss!(audit_path, NormalizedReport);
    let content_hash = content_hash(&snapshot);

    // Optional full report (absent for legacy entries → to_audit_report fallback).
    let report: Option<AuditReport> = if report_path.exists() {
        Some(load_or_miss!(report_path, AuditReport))
    } else {
        None
    };

    // Load meta if present; generate a default for entries written before meta.json existed.
    let meta = if meta_path.exists() {
        load_or_miss!(meta_path, CacheMeta)
    } else {
        CacheMeta {
            auditmysite_version: env!("CARGO_PKG_VERSION").to_string(),
            wcag_level: "AA".to_string(),
            cached_at: fetch.fetched_at,
            content_hash: content_hash.clone(),
            audit_signature: String::new(),
        }
    };

    Ok(Some(AuditArtifacts {
        fetch,
        snapshot,
        audit,
        report,
        content_hash,
        meta,
    }))
}

/// FNV-1a fingerprint of the snapshot's AXTree structure and key SEO signals.
///
/// Used for delta detection between two runs of the same URL, not as a cache key.
/// Stable across processes and platforms (unlike DefaultHasher).
pub fn content_hash(snapshot: &SnapshotArtifact) -> String {
    let mut input = String::with_capacity(512);

    // AXTree structural fingerprint: node count + root + sorted node IDs (first 100)
    let node_count = snapshot.ax_tree.len();
    input.push_str(&node_count.to_string());
    input.push(':');

    let mut node_ids: Vec<&str> = snapshot.ax_tree.nodes.keys().map(String::as_str).collect();
    node_ids.sort_unstable();
    for id in node_ids.iter().take(100) {
        input.push_str(id);
        input.push(',');
    }

    // SEO signals
    if let Some(ref seo) = snapshot.seo {
        if let Some(ref t) = seo.meta.title {
            input.push_str(t);
        }
        if let Some(ref d) = seo.meta.description {
            input.push_str(d);
        }
        input.push_str(&seo.headings.total_count.to_string());
    }

    // Performance: LCP as a change indicator
    if let Some(ref perf) = snapshot.performance {
        if let Some(ref lcp) = perf.vitals.lcp {
            input.push_str(&(lcp.value as u64).to_string());
        }
    }

    format!("{:016x}", fnv1a(input.as_bytes()))
}

pub fn to_audit_report(artifacts: &AuditArtifacts, locale: &str) -> AuditReport {
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
        ux: None,
        journey: None,
        budget_violations: Vec::new(),
        dark_mode: None,
        source_quality: None,
        ai_visibility: None,
        content_visibility: None,
        tech_stack: None,
        page_screenshots: None,
        dual_viewport: None,
        viewport_scores: None,
        throttled_performance: Vec::new(),
        patterns: None,
        screenshot_status: Default::default(),
        best_practices: None,
        consent_banner_detected: false,
        consent_banner_cmp: None,
        consent_banner_dismissed: false,
        accessibility_journey: None,
        interactive_findings: Vec::new(),
        advisory_findings: Vec::new(),
        screen_reader_audit: Some(crate::screen_reader::build_sr_audit_report(
            &artifacts.audit.url,
            artifacts.audit.timestamp,
            &artifacts.snapshot.ax_tree,
            locale,
        )),
    };

    if report.wcag_level != artifacts.audit.wcag_level {
        report.wcag_level = artifacts.audit.wcag_level;
    }

    report
}

/// Rebuild the `#[serde(skip)]` fields a cached `AuditReport` needs for
/// rendering but that are never persisted. The screen-reader audit is a cheap
/// structural pass over the cached AXTree, so it is recomputed exactly as a
/// fresh run (and `to_audit_report`) does. No-op when the field is already set.
pub fn hydrate_cached_report(report: &mut AuditReport, snapshot: &SnapshotArtifact, locale: &str) {
    if report.screen_reader_audit.is_none() {
        report.screen_reader_audit = Some(crate::screen_reader::build_sr_audit_report(
            &report.url,
            report.timestamp,
            &snapshot.ax_tree,
            locale,
        ));
    }
}

// ─── Internal helpers ─────────────────────────────────────────────────────────

/// FNV-1a 64-bit hash. Deterministic across processes and platforms.
///
/// Reference: <https://datatracker.ietf.org/doc/html/draft-eastlake-fnv>
fn fnv1a(data: &[u8]) -> u64 {
    const OFFSET_BASIS: u64 = 14695981039346656037;
    const PRIME: u64 = 1099511628211;
    data.iter().fold(OFFSET_BASIS, |hash, &byte| {
        (hash ^ byte as u64).wrapping_mul(PRIME)
    })
}

fn parse_wcag_level(level: &str) -> WcagLevel {
    match level {
        "A" => WcagLevel::A,
        "AA" => WcagLevel::AA,
        "AAA" => WcagLevel::AAA,
        _ => WcagLevel::AA,
    }
}

/// Cache directory for a URL: `~/.auditmysite/cache/{domain}/{url_hash}/v{VERSION}/`
///
/// The VERSION subdirectory provides automatic cache invalidation on binary upgrades.
/// Old entries are silently ignored — they remain on disk until manually cleared.
fn artifact_dir(url: &str) -> Result<PathBuf> {
    let parsed = url::Url::parse(url)?;
    let domain = parsed.host_str().unwrap_or("unknown");
    let url_hash = format!("{:016x}", fnv1a(url.as_bytes()));
    let version = format!("v{}", env!("CARGO_PKG_VERSION"));

    Ok(dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".auditmysite")
        .join("cache")
        .join(domain)
        .join(url_hash)
        .join(version))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn meta_with_signature(signature: &str) -> CacheMeta {
        CacheMeta {
            auditmysite_version: "test".to_string(),
            wcag_level: "AA".to_string(),
            cached_at: Utc::now(),
            content_hash: "deadbeef".to_string(),
            audit_signature: signature.to_string(),
        }
    }

    #[test]
    fn cache_reused_when_signature_matches() {
        let meta =
            meta_with_signature("level=AA;perf=1;seo=1;sec=0;mobile=1;dark=1;stack=0;consent=0");
        assert!(cache_matches_signature(
            &meta,
            "level=AA;perf=1;seo=1;sec=0;mobile=1;dark=1;stack=0;consent=0"
        ));
    }

    #[test]
    fn cache_rejected_on_config_mismatch() {
        let meta =
            meta_with_signature("level=AA;perf=0;seo=0;sec=0;mobile=0;dark=1;stack=0;consent=0");
        // Current run requests a full audit at AAA — must not reuse the lean AA entry.
        assert!(!cache_matches_signature(
            &meta,
            "level=AAA;perf=1;seo=1;sec=1;mobile=1;dark=1;stack=1;consent=0"
        ));
    }

    #[test]
    fn legacy_entry_without_signature_is_never_reused() {
        let meta = meta_with_signature("");
        assert!(!cache_matches_signature(
            &meta,
            "level=AA;perf=1;seo=1;sec=0;mobile=1;dark=1;stack=0;consent=0"
        ));
    }
}
