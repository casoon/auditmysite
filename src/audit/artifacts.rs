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
use crate::error::{AuditError, Result};
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
    /// What `content_hash` covered when this entry was written
    /// ([`FINGERPRINT_VERSION`]). 0 for entries from before the field: their
    /// hash does not compare with a current one.
    #[serde(default)]
    pub fingerprint_version: u32,
}

impl CacheMeta {
    /// Whether this entry's content hash can be compared with one computed
    /// now.
    pub fn fingerprint_comparable(&self) -> bool {
        self.fingerprint_version == FINGERPRINT_VERSION
    }
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
    /// AX node ids the audit exclusions dropped from the screen-reader layer
    /// (#703). The screen-reader report is rebuilt from `ax_tree` on a cache
    /// hit, so the exclusion has to be reapplied from here (#708).
    #[serde(default, skip_serializing_if = "std::collections::BTreeSet::is_empty")]
    pub screen_reader_excluded_node_ids: std::collections::BTreeSet<String>,
    /// Identity of the audited document: what was asked for, what answered.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub document: Option<DocumentIdentity>,
    /// Everything the run collected, as the evaluation consumes it: module
    /// data and subchecks, rule outcomes, execution states, the scope with
    /// its measurement profile, and the run context. Screenshots are not
    /// part of it. This is evidence, not presentation (#790): replaying it
    /// needs no browser and no network. `None` marks a snapshot that cannot
    /// be replayed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub collected: Option<AuditReport>,
    /// Language the run derived its screen-reader texts in; the replay
    /// rebuilds that layer from `ax_tree` the same way.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
    /// Versions of everything that gave the collected data its meaning.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub versions: Option<SnapshotVersions>,
}

/// Identity of the audited document, including where redirects led.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentIdentity {
    pub requested_url: String,
    pub final_url: String,
    pub status_code: Option<u16>,
    pub redirect_count: u32,
}

/// Version of the snapshot's shape. Bump it whenever a change to
/// `AuditReport` (or what it embeds) would let an old snapshot deserialize
/// without error yet mean something else.
pub const SNAPSHOT_SCHEMA_VERSION: u32 = 1;

/// Version of the evaluation applied to collected data: normalization,
/// scoring weights, taxonomy and decision derivation. Bump it with any change
/// that moves a score, finding or verdict for unchanged input.
pub const EVALUATION_VERSION: u32 = 1;

/// The versions under which a snapshot was written. A snapshot is replayed
/// only when every one equals the current value: another version is another
/// contract, never silently the current semantics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotVersions {
    pub snapshot_schema: u32,
    pub fingerprint: u32,
    pub collection_contract: u32,
    pub measurement_profile: u32,
    pub evaluation: u32,
    pub decision_contract: String,
    /// Digest of the shared rule catalogue (ids, criteria, levels).
    pub rule_set: String,
}

impl SnapshotVersions {
    pub fn current() -> Self {
        Self {
            snapshot_schema: SNAPSHOT_SCHEMA_VERSION,
            fingerprint: FINGERPRINT_VERSION,
            collection_contract: crate::audit::collection::COLLECTION_CONTRACT_VERSION,
            measurement_profile: crate::audit::report::MEASUREMENT_PROFILE_VERSION,
            evaluation: EVALUATION_VERSION,
            decision_contract: crate::audit::verdict::DECISION_CONTRACT.to_string(),
            rule_set: rule_set_digest(),
        }
    }

    /// Fields that differ from the current versions, as `(name, stored, current)`.
    fn mismatches(&self) -> Vec<VersionMismatch> {
        let now = Self::current();
        let mut out = Vec::new();
        let mut check = |field: &'static str, stored: String, current: String| {
            if stored != current {
                out.push(VersionMismatch {
                    field,
                    stored,
                    current,
                });
            }
        };
        check(
            "snapshot_schema",
            self.snapshot_schema.to_string(),
            now.snapshot_schema.to_string(),
        );
        check(
            "fingerprint",
            self.fingerprint.to_string(),
            now.fingerprint.to_string(),
        );
        check(
            "collection_contract",
            self.collection_contract.to_string(),
            now.collection_contract.to_string(),
        );
        check(
            "measurement_profile",
            self.measurement_profile.to_string(),
            now.measurement_profile.to_string(),
        );
        check(
            "evaluation",
            self.evaluation.to_string(),
            now.evaluation.to_string(),
        );
        check(
            "decision_contract",
            self.decision_contract.clone(),
            now.decision_contract,
        );
        check("rule_set", self.rule_set.clone(), now.rule_set);
        out
    }
}

/// Digest of every registered rule source: the shared rules, the AX-tree
/// rules, the page-rule table with its concrete outcome ids, and the two
/// inline page rules. Adding, removing or re-levelling a rule in any of them
/// changes it. A change in what a rule *decides* under an unchanged
/// registration is still a manual [`EVALUATION_VERSION`] bump.
fn rule_set_digest() -> String {
    format!("{:016x}", fnv1a(rule_set_lines().join("\n").as_bytes()))
}

fn rule_set_lines() -> Vec<String> {
    use crate::wcag::engine::TREE_RULES;
    use crate::wcag::rules::{HTML_CONTENT_MODEL_RULE, PAGE_RULES, REFLOW_RULE};
    let mut lines: Vec<String> = Vec::new();
    for rule in crate::wcag::shared::SHARED_RULES {
        lines.push(format!(
            "shared|{}|{}|{:?}",
            rule.id, rule.criterion, rule.level
        ));
    }
    for rule in TREE_RULES {
        lines.push(format!("tree|{}|{:?}", rule.axe_id, rule.min_level));
    }
    for rule in PAGE_RULES {
        lines.push(format!(
            "page|{}|{:?}|{}",
            rule.rule_id,
            rule.min_level,
            rule.outcome_ids.join(",")
        ));
    }
    for rule in [&REFLOW_RULE, &HTML_CONTENT_MODEL_RULE] {
        lines.push(format!("inline|{}", rule.axe_id));
    }
    lines
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionMismatch {
    pub field: &'static str,
    pub stored: String,
    pub current: String,
}

/// Why a snapshot cannot be replayed.
#[derive(Debug, thiserror::Error)]
pub enum ReplayError {
    /// Parts the replay needs were not persisted (older snapshot, or a
    /// reduced one).
    #[error("snapshot is incomplete: missing {}", .0.join(", "))]
    Incomplete(Vec<&'static str>),
    /// Written under other versions than the current ones.
    #[error("snapshot is incompatible: {}", format_mismatches(.0))]
    Incompatible(Vec<VersionMismatch>),
    /// Measured under another profile than the one the caller asserts: the
    /// replay would present another measurement as the requested one.
    #[error("snapshot was measured under another profile: {}", format_mismatches(.0))]
    OtherMeasurement(Vec<VersionMismatch>),
    /// The replayed data contradicts itself.
    #[error("replayed data violates the report contract: {0}")]
    Contract(#[from] crate::audit::ReportContractError),
    /// Serializing the replayed model failed.
    #[error("replayed report could not be exported: {0}")]
    Export(String),
}

fn format_mismatches(mismatches: &[VersionMismatch]) -> String {
    mismatches
        .iter()
        .map(|m| format!("{} {} (current {})", m.field, m.stored, m.current))
        .collect::<Vec<_>>()
        .join(", ")
}

impl SnapshotArtifact {
    /// Adds what a replay needs beyond the raw captures: the whole collected
    /// report (without screenshots, whose temp files do not outlive the run),
    /// the document identity, the run language and the current versions.
    pub fn with_collected(
        mut self,
        requested_url: &str,
        locale: &str,
        report: &AuditReport,
    ) -> Self {
        let navigation = &report.accessibility.execution.navigation;
        let mut collected = report.clone();
        collected.page_screenshots = None;
        self.document = Some(DocumentIdentity {
            requested_url: requested_url.to_string(),
            final_url: navigation
                .final_url
                .clone()
                .unwrap_or_else(|| report.url.clone()),
            status_code: navigation.main_document_status,
            redirect_count: navigation.redirect_count,
        });
        self.collected = Some(collected);
        self.locale = Some(locale.to_string());
        self.versions = Some(SnapshotVersions::current());
        self
    }

    /// Whether this snapshot can be evaluated again without a browser or
    /// network, and under the current semantics.
    pub fn replayability(&self) -> std::result::Result<(), ReplayError> {
        let mut missing = Vec::new();
        if self.collected.is_none() {
            missing.push("collected");
        }
        if self.versions.is_none() {
            missing.push("versions");
        }
        match &self.document {
            Some(doc) if !doc.requested_url.is_empty() && !doc.final_url.is_empty() => {}
            _ => missing.push("document"),
        }
        if self.locale.is_none() {
            missing.push("locale");
        }
        if !missing.is_empty() {
            return Err(ReplayError::Incomplete(missing));
        }
        let mut mismatches = self
            .versions
            .as_ref()
            .map(SnapshotVersions::mismatches)
            .unwrap_or_default();
        if let Some(collected) = &self.collected {
            let profile = collected
                .accessibility
                .execution
                .scope
                .measurement_profile
                .version;
            if profile != crate::audit::report::MEASUREMENT_PROFILE_VERSION {
                mismatches.push(VersionMismatch {
                    field: "collected.measurement_profile",
                    stored: profile.to_string(),
                    current: crate::audit::report::MEASUREMENT_PROFILE_VERSION.to_string(),
                });
            }
        }
        if mismatches.is_empty() {
            Ok(())
        } else {
            Err(ReplayError::Incompatible(mismatches))
        }
    }
}

/// The report a replay evaluates: the collected data with the in-memory
/// layers rebuilt from the persisted tree. The provenance says `replay`.
pub fn replay_report(snapshot: &SnapshotArtifact) -> std::result::Result<AuditReport, ReplayError> {
    replay_report_as(snapshot, "replay")
}

/// [`replay_report`] with the provenance a caller names: `replay` for an
/// explicit replay, `cache` when a cache hit is answered through it.
pub fn replay_report_as(
    snapshot: &SnapshotArtifact,
    source: &str,
) -> std::result::Result<AuditReport, ReplayError> {
    snapshot.replayability()?;
    let mut report = snapshot
        .collected
        .clone()
        .ok_or(ReplayError::Incomplete(vec!["collected"]))?;
    report.accessibility.execution.environment.source = source.to_string();
    report.screen_reader_audit = Some(rebuild_screen_reader(
        &report.url,
        report.timestamp,
        snapshot,
        snapshot.locale.as_deref().unwrap_or("en"),
        report.patterns.as_ref(),
    ));
    Ok(report)
}

/// The canonical model of a snapshot, evaluated without a browser or network
/// access. Equals the fresh run's model except for the named volatile
/// metadata (`timestamp`, `duration_ms`, `execution.environment.source`).
pub fn replay(snapshot: &SnapshotArtifact) -> std::result::Result<NormalizedReport, ReplayError> {
    let report = replay_report(snapshot)?;
    Ok(crate::audit::normalize_checked(&report)?.normalized)
}

/// The measurement profile a caller asserts for a replay. `None` leaves the
/// field unasserted.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExpectedProfile {
    pub session_policy: Option<String>,
    pub state_id: Option<String>,
    /// SHA-256 of the declared storage state.
    pub storage_state_sha256: Option<String>,
}

/// How two measurement profiles differ, as `(field, a, b)` in the
/// `stored`/`current` slots. Equal profiles differ in nothing; a page whose
/// content differs between two runs of one URL is explained by the profile
/// only if this list is not empty.
pub fn profile_differences(
    a: &crate::audit::MeasurementProfile,
    b: &crate::audit::MeasurementProfile,
) -> Vec<VersionMismatch> {
    let mut out = Vec::new();
    let mut check = |field: &'static str, a: String, b: String| {
        if a != b {
            out.push(VersionMismatch {
                field,
                stored: a,
                current: b,
            });
        }
    };
    let sha = |p: &crate::audit::MeasurementProfile| {
        p.storage_state
            .as_ref()
            .map_or("none".to_string(), |s| s.sha256.clone())
    };
    check(
        "session_policy",
        a.session_policy.clone(),
        b.session_policy.clone(),
    );
    check(
        "state_id",
        a.state_id.clone().unwrap_or_else(|| "none".to_string()),
        b.state_id.clone().unwrap_or_else(|| "none".to_string()),
    );
    check("storage_state", sha(a), sha(b));
    check(
        "extra_checks",
        a.extra_checks.join(","),
        b.extra_checks.join(","),
    );
    out
}

impl SnapshotArtifact {
    /// Refuses a snapshot measured under another profile than the asserted
    /// one. Only the asserted fields are compared.
    pub fn check_measurement(
        &self,
        expected: &ExpectedProfile,
    ) -> std::result::Result<(), ReplayError> {
        self.replayability()?;
        let stored = self
            .collected
            .as_ref()
            .map(|c| c.accessibility.execution.scope.measurement_profile.clone())
            .unwrap_or_default();
        let mut wanted = stored.clone();
        if let Some(session) = &expected.session_policy {
            wanted.session_policy = session.clone();
        }
        if let Some(id) = &expected.state_id {
            wanted.state_id = Some(id.clone());
        }
        if let Some(sha) = &expected.storage_state_sha256 {
            wanted.storage_state = Some(crate::audit::StorageStateRef {
                sha256: sha.clone(),
                cookies: 0,
                origins: 0,
            });
        }
        let differences = profile_differences(&stored, &wanted);
        if differences.is_empty() {
            Ok(())
        } else {
            Err(ReplayError::OtherMeasurement(differences))
        }
    }

    /// Reads a stored snapshot from a `snapshot.json` or the cache entry
    /// directory holding one.
    pub fn read(path: &std::path::Path) -> Result<Self> {
        let file = if path.is_dir() {
            path.join("snapshot.json")
        } else {
            path.to_path_buf()
        };
        let bytes = fs::read(&file).map_err(|e| AuditError::FileError {
            path: file.clone(),
            reason: e.to_string(),
        })?;
        serde_json::from_slice(&bytes).map_err(|e| AuditError::FileError {
            path: file,
            reason: format!("not a snapshot: {e}"),
        })
    }

    /// The snapshot of the current cache entry for `url`.
    pub fn read_cached(url: &str) -> Result<Self> {
        Self::read(&artifact_dir(url)?)
    }
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
    write_entry(&dir, wcag_level, artifacts)?;
    Ok(dir)
}

fn write_entry(dir: &std::path::Path, wcag_level: &str, artifacts: &AuditArtifacts) -> Result<()> {
    fs::create_dir_all(dir)?;

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
    // The replayable snapshot already holds the whole report; a second copy
    // would only be able to disagree with it.
    let report_path = dir.join("report.json");
    match (&artifacts.report, &artifacts.snapshot.collected) {
        (Some(report), None) => fs::write(&report_path, serde_json::to_vec_pretty(report)?)?,
        _ => {
            let _ = fs::remove_file(&report_path);
        }
    }

    let meta = CacheMeta {
        auditmysite_version: env!("CARGO_PKG_VERSION").to_string(),
        wcag_level: wcag_level.to_string(),
        cached_at: Utc::now(),
        content_hash: artifacts.content_hash.clone(),
        audit_signature: artifacts.meta.audit_signature.clone(),
        fingerprint_version: FINGERPRINT_VERSION,
    };
    fs::write(dir.join("meta.json"), serde_json::to_vec_pretty(&meta)?)?;

    Ok(())
}

/// Why the cache cannot answer a request.
#[derive(Debug, thiserror::Error)]
pub enum CacheMiss {
    /// Nothing stored for this URL (or the entry is not complete on disk).
    #[error("no cached audit for this URL")]
    Absent,
    /// The entry exists but cannot be read.
    #[error("cached audit {} is unreadable ({reason})", .path.display())]
    Unreadable { path: PathBuf, reason: String },
    /// The entry was produced for another audit scope or run configuration.
    #[error("cached audit was produced with a different audit configuration")]
    OtherConfiguration,
    /// The snapshot cannot be replayed under the current semantics.
    #[error("cached audit is not reusable: {0}")]
    NotReplayable(#[from] ReplayError),
}

/// A cache entry, or why there is none.
#[derive(Debug)]
pub enum CacheLookup {
    Hit(Box<AuditArtifacts>),
    Miss(CacheMiss),
}

pub fn load_artifacts(url: &str) -> Result<Option<AuditArtifacts>> {
    Ok(match lookup_artifacts(url)? {
        CacheLookup::Hit(artifacts) => Some(*artifacts),
        CacheLookup::Miss(CacheMiss::Absent) => None,
        CacheLookup::Miss(reason) => {
            warn!("{reason}; running fresh");
            None
        }
    })
}

/// Looks up the cache entry of `url`, naming the reason when there is none.
pub fn lookup_artifacts(url: &str) -> Result<CacheLookup> {
    Ok(read_entry(&artifact_dir(url)?))
}

/// The report a `--reuse-cache` hit answers with: the stored snapshot
/// replayed (#790), never a reconstruction. An entry written for another
/// audit configuration, or one whose snapshot is incomplete or from other
/// versions, is a miss with its reason — never the current semantics applied
/// to old data.
pub fn cached_report(
    url: &str,
    expected_signature: &str,
) -> Result<std::result::Result<AuditReport, CacheMiss>> {
    let cached = match lookup_artifacts(url)? {
        CacheLookup::Hit(cached) => cached,
        CacheLookup::Miss(reason) => return Ok(Err(reason)),
    };
    if !cache_matches_signature(&cached.meta, expected_signature) {
        return Ok(Err(CacheMiss::OtherConfiguration));
    }
    Ok(replay_report_as(&cached.snapshot, "cache").map_err(CacheMiss::from))
}

fn read_entry(dir: &std::path::Path) -> CacheLookup {
    use CacheLookup::{Hit, Miss};
    if !dir.exists() {
        return Miss(CacheMiss::Absent);
    }

    let fetch_path = dir.join("fetch.json");
    let snapshot_path = dir.join("snapshot.json");
    let audit_path = dir.join("audit.json");
    let report_path = dir.join("report.json");
    let meta_path = dir.join("meta.json");

    if !fetch_path.exists() || !snapshot_path.exists() || !audit_path.exists() {
        return Miss(CacheMiss::Absent);
    }

    // A truncated or otherwise corrupt cache entry must not abort the run — it
    // is a cache miss with a reason, so the caller falls back to a fresh audit
    // (#405).
    macro_rules! load_or_miss {
        ($path:expr, $ty:ty) => {
            match fs::read(&$path)
                .map_err(|e| e.to_string())
                .and_then(|bytes| serde_json::from_slice::<$ty>(&bytes).map_err(|e| e.to_string()))
            {
                Ok(value) => value,
                Err(reason) => {
                    return Miss(CacheMiss::Unreadable {
                        path: $path.clone(),
                        reason,
                    })
                }
            }
        };
    }

    let fetch: FetchArtifact = load_or_miss!(fetch_path, FetchArtifact);
    let snapshot: SnapshotArtifact = load_or_miss!(snapshot_path, SnapshotArtifact);
    let audit: NormalizedReport = load_or_miss!(audit_path, NormalizedReport);
    // A snapshot that cannot be replayed under the current semantics is a
    // miss, never an entry read as if it were current (#790).
    if let Err(reason) = snapshot.replayability() {
        return Miss(CacheMiss::NotReplayable(reason));
    }
    let content_hash = content_hash(&snapshot);

    // Optional full report (absent for legacy entries → to_audit_report fallback).
    let report: Option<AuditReport> = if report_path.exists() {
        Some(load_or_miss!(report_path, AuditReport))
    } else {
        snapshot.collected.clone()
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
            fingerprint_version: 0,
        }
    };

    Hit(Box::new(AuditArtifacts {
        fetch,
        snapshot,
        audit,
        report,
        content_hash,
        meta,
    }))
}

/// Version of what [`content_hash`] covers. A hash of another version never
/// compares equal to this one's: it measured something else.
pub const FINGERPRINT_VERSION: u32 = 4;

/// FNV-1a fingerprint of the snapshot's content: every AX node with role,
/// name, value, properties, links and DOM anchor, plus the SEO parts that
/// describe the page (meta, headings, social tags, technical SEO, structured
/// data).
///
/// Used for delta detection between two runs of the same URL, not as a cache
/// key. Stable across processes and platforms (unlike DefaultHasher) and
/// independent of map iteration order. Measured timings (LCP, TBT, …) are
/// not content and stay out: a slower run of an unchanged page is no change.
///
/// Version 2 hashed AX nodes and SEO parts only; version 3 ignores browser-assigned ids; version 4 adds the
/// style inputs (contrast violations, mobile, design quality, dark mode).
/// Version 1 hashed the node count, the first 100 node ids and a few SEO
/// strings, so a changed label or text on an unchanged tree shape went
/// undetected (#790).
pub fn content_hash(snapshot: &SnapshotArtifact) -> String {
    let mut input = String::with_capacity(4096);
    let mut nodes: Vec<_> = snapshot.ax_tree.nodes.iter().collect();
    nodes.sort_unstable_by(|a, b| a.0.cmp(b.0));
    for (id, node) in nodes {
        input.push_str(id);
        input.push('=');
        let mut value = serde_json::to_value(node).unwrap_or_default();
        strip_volatile_ids(&mut value);
        push_canonical(&mut input, &value);
        input.push('\n');
    }
    if let Some(seo) = &snapshot.seo {
        for part in [
            serde_json::to_value(&seo.meta),
            serde_json::to_value(&seo.headings),
            serde_json::to_value(&seo.social),
            serde_json::to_value(&seo.technical),
            serde_json::to_value(&seo.structured_data),
        ] {
            push_canonical(&mut input, &part.unwrap_or_default());
            input.push('\n');
        }
    }
    // Presentation inputs that never reach the AX tree: a pure style change
    // (colours, sizes, font, colour scheme) leaves node count, ids and names
    // as they were. They surface as contrast violations and in the mobile,
    // design-quality and dark-mode measurements.
    if let Some(mobile) = &snapshot.mobile {
        push_canonical(
            &mut input,
            &serde_json::to_value(mobile).unwrap_or_default(),
        );
        input.push('\n');
    }
    if let Some(collected) = &snapshot.collected {
        let contrast: Vec<_> = collected
            .accessibility
            .wcag_results
            .violations
            .iter()
            .filter(|v| matches!(v.rule.as_str(), "1.4.3" | "1.4.6" | "1.4.11"))
            .collect();
        for part in [
            serde_json::to_value(&contrast),
            serde_json::to_value(&collected.experience.design_quality),
            serde_json::to_value(&collected.experience.dark_mode),
        ] {
            push_canonical(&mut input, &part.unwrap_or_default());
            input.push('\n');
        }
    }
    format!("{:016x}", fnv1a(input.as_bytes()))
}

/// Keys holding a browser-assigned id that differs from one browser start to
/// the next for the same page (the DOM backend node id, also inside related
/// nodes). They say nothing about content, and a hash that includes them
/// differs between two runs of an unchanged page.
const VOLATILE_ID_KEYS: &[&str] = &["backend_dom_node_id", "backendDOMNodeId", "backend_node_id"];

fn is_volatile_id(key: &str) -> bool {
    let known = VOLATILE_ID_KEYS;
    known.contains(&key)
}

fn strip_volatile_ids(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            map.retain(|key, _| !is_volatile_id(key));
            map.values_mut().for_each(strip_volatile_ids);
        }
        serde_json::Value::Array(items) => items.iter_mut().for_each(strip_volatile_ids),
        _ => {}
    }
}

/// JSON with object keys sorted, so equal content hashes equal whatever the
/// order a map was filled in.
fn push_canonical(out: &mut String, value: &serde_json::Value) {
    use serde_json::Value;
    match value {
        Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort_unstable();
            out.push('{');
            for key in keys {
                out.push_str(key);
                out.push(':');
                push_canonical(out, &map[key]);
                out.push(',');
            }
            out.push('}');
        }
        Value::Array(items) => {
            out.push('[');
            for item in items {
                push_canonical(out, item);
                out.push(',');
            }
            out.push(']');
        }
        other => out.push_str(&other.to_string()),
    }
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
        // The lossy reconstruction keeps the stored profile; indexability is
        // not part of the normalized record.
        run_context: crate::audit::RunContext {
            indexability: Default::default(),
            decision_profile: artifacts.audit.decision.profile.clone(),
        },
        url: artifacts.audit.url.clone(),
        wcag_level: artifacts.audit.wcag_level,
        timestamp: artifacts.audit.timestamp,
        accessibility: crate::audit::report::AccessibilitySection {
            wcag_results,
            score: artifacts.audit.score as f32,
            grade: artifacts.audit.grade.clone(),
            certificate: artifacts.audit.certificate.clone(),
            statistics: stats,
            nodes_analyzed: artifacts.audit.nodes_analyzed,
            execution: artifacts.audit.execution.clone(),
        },
        duration_ms: artifacts.audit.duration_ms,
        performance: artifacts.snapshot.performance.clone(),
        security: artifacts.snapshot.security.clone(),
        // `SnapshotArtifact` (the lossy legacy cache-reconstruction shape) never
        // carried this field — see `ux`/`journey` below, same treatment.
        html_conform: None,
        experience: crate::audit::report::ExperienceSection {
            mobile: artifacts.snapshot.mobile.clone(),
            ..Default::default()
        },
        ux: None,
        journey: None,
        discoverability: crate::audit::report::DiscoverabilitySection {
            seo: artifacts.snapshot.seo.clone(),
            ai_visibility: None,
            content_visibility: None,
            source_quality: None,
            tech_stack: None,
        },
        page_screenshots: None,
        dual_viewport: None,
        viewport_scores: None,
        throttled_performance: Vec::new(),
        patterns: None,
        screenshot_status: Default::default(),
        best_practices: None,
        commerce: None,
        consent_banner_detected: false,
        consent_banner_cmp: None,
        consent_banner_dismissed: false,
        consent_privacy: None,
        accessibility_journey: None,
        interactive_findings: Vec::new(),
        screen_reader_audit: Some(rebuild_screen_reader(
            &artifacts.audit.url,
            artifacts.audit.timestamp,
            &artifacts.snapshot,
            locale,
            None,
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
    report.accessibility.execution.environment.source = "cache".to_string();
    if report.screen_reader_audit.is_none() {
        report.screen_reader_audit = Some(rebuild_screen_reader(
            &report.url,
            report.timestamp,
            snapshot,
            locale,
            report.patterns.as_ref(),
        ));
    }
}

/// Rebuilds the screen-reader report from the cached AXTree and reapplies the
/// exclusions recorded at audit time (#708), so the sidecar and the PDF drop
/// the same specimens as the fresh run did.
fn rebuild_screen_reader(
    url: &str,
    timestamp: DateTime<Utc>,
    snapshot: &SnapshotArtifact,
    locale: &str,
    patterns: Option<&crate::patterns::PatternAnalysis>,
) -> crate::screen_reader::SrAuditReport {
    let mut sr = crate::screen_reader::build_sr_audit_report(
        url,
        timestamp,
        &snapshot.ax_tree,
        locale,
        patterns,
    );
    let excluded = &snapshot.screen_reader_excluded_node_ids;
    sr.apply_exclusion(|id| excluded.contains(id));
    sr
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
            fingerprint_version: 0,
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

    fn labelled(name: Option<&str>, text_value: Option<&str>) -> SnapshotArtifact {
        let mut snapshot = snapshot_with_unlabelled_field(&[]);
        let field = snapshot.ax_tree.nodes.get_mut("field").unwrap();
        field.name = name.map(str::to_string);
        field.value = text_value.map(str::to_string);
        snapshot
    }

    #[test]
    fn changed_content_on_an_unchanged_tree_shape_changes_the_hash() {
        let before = content_hash(&labelled(Some("Email"), None));
        // Same node count, same ids: only the label differs.
        assert_ne!(
            before,
            content_hash(&labelled(Some("E-Mail address"), None))
        );
        assert_ne!(before, content_hash(&labelled(Some("Email"), Some("x"))));
        assert_eq!(before, content_hash(&labelled(Some("Email"), None)));
    }

    /// A pure style change leaves the AX tree untouched; the fingerprint
    /// still moves (#790).
    #[test]
    fn a_style_only_change_changes_the_hash() {
        with_big_stack(|| {
            let base = replayable_snapshot();
            let before = content_hash(&base);
            assert_eq!(before, content_hash(&base.clone()));

            let mut contrast = base.clone();
            contrast
                .collected
                .as_mut()
                .unwrap()
                .accessibility
                .wcag_results
                .violations
                .push(crate::wcag::Violation::new(
                    "1.4.3",
                    "Contrast",
                    WcagLevel::AA,
                    crate::taxonomy::Severity::High,
                    "Contrast 2.1:1 (#777 on #fff)",
                    "n1",
                ));
            assert_ne!(before, content_hash(&contrast));

            let mut other_colour = contrast.clone();
            other_colour
                .collected
                .as_mut()
                .unwrap()
                .accessibility
                .wcag_results
                .violations[0]
                .message = "Contrast 2.9:1 (#666 on #fff)".to_string();
            assert_ne!(content_hash(&contrast), content_hash(&other_colour));
        });
    }

    #[test]
    fn browser_assigned_ids_are_not_content() {
        let mut one = labelled(Some("Email"), None);
        let mut two = one.clone();
        one.ax_tree
            .nodes
            .get_mut("field")
            .unwrap()
            .backend_dom_node_id = Some(11);
        two.ax_tree
            .nodes
            .get_mut("field")
            .unwrap()
            .backend_dom_node_id = Some(907);
        assert_eq!(content_hash(&one), content_hash(&two));
    }

    #[test]
    fn an_entry_hashed_by_an_older_fingerprint_is_not_comparable() {
        let mut meta = meta_with_signature("sig");
        assert!(!meta.fingerprint_comparable());
        meta.fingerprint_version = FINGERPRINT_VERSION;
        assert!(meta.fingerprint_comparable());
    }

    fn snapshot_with_unlabelled_field(excluded: &[&str]) -> SnapshotArtifact {
        let node = |id: &str, role: &str, children: Vec<String>, parent: Option<&str>| {
            a11y_perception::AXNode {
                node_id: id.to_string(),
                ignored: false,
                ignored_reasons: Vec::new(),
                role: Some(role.into()),
                name: None,
                name_source: None,
                description: None,
                value: None,
                properties: Vec::new(),
                child_ids: children,
                parent_id: parent.map(str::to_string),
                backend_dom_node_id: None,
            }
        };
        SnapshotArtifact {
            ax_tree: AXTree::from_nodes(vec![
                node("root", "RootWebArea", vec!["field".into()], None),
                node("field", "textbox", Vec::new(), Some("root")),
            ]),
            performance: None,
            seo: None,
            security: None,
            mobile: None,
            screen_reader_excluded_node_ids: excluded.iter().map(|id| id.to_string()).collect(),
            document: None,
            collected: None,
            locale: None,
            versions: None,
        }
    }

    fn names_field(report: &AuditReport) -> bool {
        report
            .screen_reader_audit
            .as_ref()
            .unwrap()
            .issues
            .iter()
            .any(|issue| issue.affected_node_ids.iter().any(|id| id == "field"))
    }

    #[test]
    fn cache_hit_reapplies_screen_reader_exclusions() {
        // #708: the screen-reader report is rebuilt from the cached tree, so
        // the exclusion recorded at audit time has to be applied again.
        let mut plain = AuditReport::new(
            "https://example.com".into(),
            WcagLevel::AA,
            WcagResults::new(),
            0,
        );
        let mut excluded = plain.clone();
        hydrate_cached_report(&mut plain, &snapshot_with_unlabelled_field(&[]), "en");
        hydrate_cached_report(
            &mut excluded,
            &snapshot_with_unlabelled_field(&["field"]),
            "en",
        );
        assert!(names_field(&plain));
        assert!(!names_field(&excluded));
        assert!(excluded
            .screen_reader_audit
            .unwrap()
            .excluded_node_ids
            .contains("field"));
    }

    fn entry(snapshot: SnapshotArtifact) -> AuditArtifacts {
        let report = AuditReport::new(
            "https://example.com".into(),
            WcagLevel::AA,
            WcagResults::new(),
            0,
        );
        let audit = crate::audit::normalized::normalize(&report).normalized;
        AuditArtifacts {
            fetch: FetchArtifact {
                requested_url: "https://example.com".into(),
                final_url: "https://example.com".into(),
                status_code: Some(200),
                fetched_at: Utc::now(),
                duration_ms: 1,
            },
            content_hash: content_hash(&snapshot),
            snapshot,
            audit,
            report: None,
            meta: meta_with_signature("sig"),
        }
    }

    fn replayable_snapshot() -> SnapshotArtifact {
        let report = AuditReport::new(
            "https://example.com".into(),
            WcagLevel::AA,
            WcagResults::new(),
            0,
        );
        let mut report = report;
        let profile = &mut report.accessibility.execution.scope.measurement_profile;
        profile.version = crate::audit::report::MEASUREMENT_PROFILE_VERSION;
        profile.read_only = true;
        snapshot_with_unlabelled_field(&[]).with_collected("https://example.com", "en", &report)
    }

    /// A whole report is large; unoptimized test builds move it by value.
    fn with_big_stack(test: impl FnOnce() + Send + 'static) {
        std::thread::Builder::new()
            .stack_size(32 << 20)
            .spawn(test)
            .unwrap()
            .join()
            .unwrap();
    }

    #[test]
    fn a_replayable_entry_is_read_back_with_its_collected_report() {
        with_big_stack(|| {
            let dir = tempfile::tempdir().unwrap();
            write_entry(dir.path(), "AA", &entry(replayable_snapshot())).unwrap();
            let CacheLookup::Hit(loaded) = read_entry(dir.path()) else {
                panic!("compatible entry expected");
            };
            assert!(loaded.report.is_some());
            assert!(!dir.path().join("report.json").exists());
            assert!(replay(&loaded.snapshot).is_ok());
        });
    }

    /// The rule-set digest covers every registered rule source, not only the
    /// shared rules: a tree or page rule added, removed or re-levelled makes
    /// an older snapshot incompatible (#790).
    #[test]
    fn the_rule_set_digest_covers_every_rule_source() {
        let lines = rule_set_lines();
        for kind in ["shared|", "tree|", "page|", "inline|"] {
            assert!(lines.iter().any(|l| l.starts_with(kind)), "{kind}");
        }
        assert_eq!(
            lines.iter().filter(|l| l.starts_with("tree|")).count(),
            crate::wcag::engine::TREE_RULES.len()
        );
        assert_eq!(
            lines.iter().filter(|l| l.starts_with("page|")).count(),
            crate::wcag::rules::PAGE_RULES.len()
        );
        // A changed page rule changes the digest.
        let before = fnv1a(lines.join("\n").as_bytes());
        let mut changed = lines.clone();
        let i = changed.iter().position(|l| l.starts_with("page|")).unwrap();
        changed[i].push_str("|extra");
        assert_ne!(before, fnv1a(changed.join("\n").as_bytes()));
    }

    /// A cache hit and a replay show the extra checks the stored run asked
    /// for — requested subchecks and profile alike — and a snapshot measured
    /// without them is another measurement (#788).
    #[test]
    fn cache_and_replay_keep_the_requested_extra_checks() {
        with_big_stack(|| {
            let mut snapshot = replayable_snapshot();
            let scope = &mut snapshot
                .collected
                .as_mut()
                .unwrap()
                .accessibility
                .execution
                .scope;
            scope.requested_subchecks = vec!["content_visibility.ssr_gap".to_string()];
            scope.measurement_profile.extra_checks = vec!["--check-ssr-content".to_string()];
            let stored = scope.clone();

            let dir = tempfile::tempdir().unwrap();
            write_entry(dir.path(), "AA", &entry(snapshot.clone())).unwrap();
            let CacheLookup::Hit(loaded) = read_entry(dir.path()) else {
                panic!("compatible entry expected");
            };
            let replayed = replay_report_as(&loaded.snapshot, "replay").unwrap();
            let scope = &replayed.accessibility.execution.scope;
            assert_eq!(scope.requested_subchecks, stored.requested_subchecks);
            assert_eq!(
                scope.measurement_profile.extra_checks,
                stored.measurement_profile.extra_checks
            );

            let mut without = snapshot.clone();
            without
                .collected
                .as_mut()
                .unwrap()
                .accessibility
                .execution
                .scope
                .measurement_profile
                .extra_checks
                .clear();
            let fields: Vec<_> = profile_differences(
                &stored.measurement_profile,
                &without
                    .collected
                    .as_ref()
                    .unwrap()
                    .accessibility
                    .execution
                    .scope
                    .measurement_profile,
            )
            .iter()
            .map(|d| d.field)
            .collect();
            assert_eq!(fields, ["extra_checks"]);
        });
    }

    #[test]
    fn a_snapshot_of_another_profile_is_not_the_asserted_measurement() {
        with_big_stack(|| {
            let mut snapshot = replayable_snapshot();
            let profile = &mut snapshot
                .collected
                .as_mut()
                .unwrap()
                .accessibility
                .execution
                .scope
                .measurement_profile;
            profile.state_id = Some("logged-in".to_string());
            profile.storage_state = Some(crate::audit::StorageStateRef {
                sha256: "aa".repeat(32),
                cookies: 1,
                origins: 0,
            });
            let ok = |e: ExpectedProfile| snapshot.check_measurement(&e);
            // Nothing asserted, or the stored profile asserted: the same measurement.
            assert!(ok(ExpectedProfile::default()).is_ok());
            assert!(ok(ExpectedProfile {
                session_policy: Some("unknown".into()),
                state_id: Some("logged-in".into()),
                storage_state_sha256: Some("aa".repeat(32)),
            })
            .is_ok());
            // Any asserted field that differs refuses, naming the field.
            for (expected, field) in [
                (
                    ExpectedProfile {
                        session_policy: Some("shared".into()),
                        ..Default::default()
                    },
                    "session_policy",
                ),
                (
                    ExpectedProfile {
                        state_id: Some("logged-out".into()),
                        ..Default::default()
                    },
                    "state_id",
                ),
                (
                    ExpectedProfile {
                        storage_state_sha256: Some("bb".repeat(32)),
                        ..Default::default()
                    },
                    "storage_state",
                ),
            ] {
                match ok(expected) {
                    Err(ReplayError::OtherMeasurement(d)) => assert_eq!(d[0].field, field),
                    other => panic!("{field}: {other:?}"),
                }
            }
        });
    }

    #[test]
    fn an_incompatible_or_legacy_entry_is_a_cache_miss() {
        with_big_stack(|| {
            let dir = tempfile::tempdir().unwrap();
            let mut stale = replayable_snapshot();
            stale.versions.as_mut().unwrap().evaluation += 1;
            write_entry(dir.path(), "AA", &entry(stale)).unwrap();
            assert!(matches!(
                read_entry(dir.path()),
                CacheLookup::Miss(CacheMiss::NotReplayable(ReplayError::Incompatible(_)))
            ));

            // Written before the replay fields existed: the same.
            let legacy = tempfile::tempdir().unwrap();
            write_entry(
                legacy.path(),
                "AA",
                &entry(snapshot_with_unlabelled_field(&[])),
            )
            .unwrap();
            let CacheLookup::Miss(reason) = read_entry(legacy.path()) else {
                panic!("legacy entry must miss");
            };
            // The reason is named for the user, not swallowed.
            assert!(reason.to_string().contains("incomplete"), "{reason}");
        });
    }
}
