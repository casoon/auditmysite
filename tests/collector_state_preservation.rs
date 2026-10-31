//! #777: the state of every collector survives persistence, replay,
//! normalization, batch aggregation and the JSON projections.
//!
//! One test body runs for each entry of the collector inventory
//! (`COLLECTORS`) and for each non-measured state. A collector that lost its
//! measurement must still say so at the far end of the pipeline — it must
//! neither disappear nor turn into a regular result.
//!
//! The reports are boxed and handled one at a time: several `AuditReport`s
//! on the stack segfault under tarpaulin.

use auditmysite::audit::collection::{CollectionRecord, CollectionState, COLLECTORS};
use auditmysite::audit::{
    normalize_checked, replay, AuditQualityStatus, AuditReport, BatchReport, ExecutionStatus,
    ModuleRun, SnapshotArtifact, MEASUREMENT_PROFILE_VERSION,
};
use auditmysite::cli::WcagLevel;
use auditmysite::output::{format_json_batch, format_json_replay};
use auditmysite::seo::SeoAnalysis;
use auditmysite::wcag::WcagResults;
use auditmysite::AXTree;
use serde_json::Value;

/// Capture collectors of the accessibility audit that the inventory must
/// list (#777); the ids are what `module_runs[].subchecks[]` publishes.
const CAPTURE_COLLECTORS: &[&str] = &[
    "accessibility.navigation_snapshot",
    "accessibility.ax_tree",
    "accessibility.consent_banner",
    "accessibility.exclusion_scope",
    "accessibility.frames",
    "accessibility.page_stability",
    "accessibility.rule_execution",
    "accessibility_journey.tab_walk",
    "accessibility_journey.skip_link",
    "accessibility_journey.disclosure",
    "accessibility_journey.modal",
    "accessibility_journey.tabs",
    "accessibility_journey.menu",
    "accessibility_journey.form_error",
    "accessibility_journey.add_to_cart",
    "accessibility_journey.quantity_stepper",
    "accessibility_journey.spa_navigation",
];

#[test]
fn inventory_lists_every_capture_collector() {
    for id in CAPTURE_COLLECTORS {
        assert!(
            auditmysite::audit::collection::collector(id).is_some(),
            "{id} is missing from COLLECTORS"
        );
    }
}

/// The states a collector can end in without a regular measurement, with
/// what a record of that state carries.
fn lost_states() -> Vec<(CollectionState, Option<&'static str>, Vec<&'static str>)> {
    vec![
        (CollectionState::Failed, Some("evaluation_failed"), vec![]),
        (CollectionState::Partial, None, vec!["one_part_missing"]),
        (
            CollectionState::Unsupported,
            Some("cdp_domain_missing"),
            vec![],
        ),
        (
            CollectionState::NotApplicable,
            Some("nothing_to_collect"),
            vec![],
        ),
        (CollectionState::Empty, None, vec![]),
    ]
}

fn record(
    spec: &auditmysite::audit::collection::CollectorSpec,
    state: CollectionState,
    reason: Option<&str>,
    gaps: &[&str],
) -> CollectionRecord {
    CollectionRecord {
        collector: spec.id.to_string(),
        module: spec.module.to_string(),
        state,
        method: spec.method,
        method_version: spec.method_version,
        scoring: spec.scoring,
        reason_code: reason.map(str::to_string),
        gaps: gaps.iter().map(|g| g.to_string()).collect(),
    }
}

/// A report as the pipeline leaves it, with one collection recorded.
fn report_with(record: CollectionRecord) -> Box<AuditReport> {
    let mut report = Box::new(
        AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            WcagResults::new(),
            100,
        )
        .with_seo(SeoAnalysis {
            score: 40,
            ..Default::default()
        }),
    );
    let profile = &mut report.accessibility.execution.scope.measurement_profile;
    profile.version = MEASUREMENT_PROFILE_VERSION;
    profile.read_only = true;
    let is_gap = record.state.is_gap();
    if is_gap {
        // As `update_audit_quality` leaves a run with a lost collection.
        let quality = &mut report.accessibility.execution.quality;
        quality.status = AuditQualityStatus::Partial;
        quality.qualified_results = true;
        quality.partial_or_failed_modules = 1;
    }
    report.accessibility.execution.module_runs.push(ModuleRun {
        module: record.module.clone(),
        status: if is_gap {
            ExecutionStatus::Partial
        } else {
            ExecutionStatus::Completed
        },
        subchecks: vec![record.into()],
        ..Default::default()
    });
    // The run status above follows `fold_records`.
    report
}

fn snapshot_of(report: &AuditReport) -> SnapshotArtifact {
    let empty: SnapshotArtifact = serde_json::from_value(serde_json::json!({
        "ax_tree": serde_json::to_value(AXTree::from_nodes(Vec::new())).unwrap(),
        "performance": null, "seo": null, "security": null, "mobile": null
    }))
    .unwrap();
    let mut report = report.clone();
    report.screen_reader_audit = Some(auditmysite::screen_reader::build_sr_audit_report(
        &report.url,
        report.timestamp,
        &AXTree::from_nodes(Vec::new()),
        "en",
        report.patterns.as_ref(),
    ));
    let snapshot = empty.with_collected("https://example.com", "en", &report);
    // As persisted and read back from disk.
    serde_json::from_slice(&serde_json::to_vec(&snapshot).unwrap()).unwrap()
}

/// The subchecks of every `module_runs` array anywhere in `json`.
fn published_subchecks(json: &Value) -> Vec<&Value> {
    fn walk<'a>(value: &'a Value, out: &mut Vec<&'a Value>) {
        match value {
            Value::Object(map) => {
                for (key, inner) in map {
                    if key == "module_runs" {
                        for run in inner.as_array().into_iter().flatten() {
                            out.extend(
                                run["subchecks"]
                                    .as_array()
                                    .into_iter()
                                    .flatten()
                                    .filter(|s| s.get("state").is_some()),
                            );
                        }
                    }
                    walk(inner, out);
                }
            }
            Value::Array(items) => items.iter().for_each(|i| walk(i, out)),
            _ => {}
        }
    }
    let mut out = Vec::new();
    walk(json, &mut out);
    out
}

fn assert_published(json: &Value, expected: &CollectionRecord, layer: &str) {
    let found: Vec<_> = published_subchecks(json)
        .into_iter()
        .filter(|s| s["subcheck"] == expected.collector.as_str())
        .collect();
    assert!(
        !found.is_empty(),
        "{layer}: the {} collection vanished",
        expected.collector
    );
    let state = serde_json::to_value(expected.state).unwrap();
    for sub in found {
        assert_eq!(
            sub["state"], state,
            "{layer}: {} changed its state",
            expected.collector
        );
        assert_eq!(
            sub.get("reason_code").and_then(Value::as_str),
            expected.reason_code.as_deref(),
            "{layer}: {} lost its reason",
            expected.collector
        );
        let gaps: Vec<&str> = sub
            .get("gaps")
            .and_then(Value::as_array)
            .map(|g| g.iter().filter_map(Value::as_str).collect())
            .unwrap_or_default();
        assert_eq!(
            gaps,
            expected.gaps.iter().map(String::as_str).collect::<Vec<_>>(),
            "{layer}: {} lost its gaps",
            expected.collector
        );
    }
}

#[test]
fn every_collector_state_survives_replay_batch_and_json() {
    for spec in COLLECTORS {
        for (state, reason, gaps) in lost_states() {
            let expected = record(spec, state, reason, &gaps);
            let report = report_with(expected.clone());

            // Persistence and replay: the stored snapshot evaluates again
            // without a browser and keeps the collection as it was.
            let snapshot = snapshot_of(&report);
            let replayed = replay(&snapshot)
                .unwrap_or_else(|e| panic!("{} {state:?}: replay failed: {e}", spec.id));
            let run = replayed
                .execution
                .module_runs
                .iter()
                .find(|run| run.module == spec.module)
                .unwrap_or_else(|| panic!("{}: the module run vanished in replay", spec.id));
            let sub = run
                .subchecks
                .iter()
                .find(|s| s.subcheck == spec.id)
                .unwrap_or_else(|| panic!("{}: the subcheck vanished in replay", spec.id));
            assert_eq!(sub.state, Some(state), "{} {state:?}", spec.id);
            assert_eq!(sub.reason_code.as_deref(), reason, "{}", spec.id);
            assert_eq!(
                run.status == ExecutionStatus::Completed,
                !state.is_gap(),
                "{} {state:?}: a lost collection must not leave a completed run",
                spec.id
            );

            // A lost scoring collection leaves no measured module score and
            // no 100 points: the SEO score stays out of the overall score.
            if spec.scoring && state.is_gap() && spec.module == "seo" {
                let seo = replayed
                    .module_scores
                    .iter()
                    .find(|m| m.name == "SEO")
                    .expect("SEO entry");
                assert_ne!(seo.measurement_type, "measured", "{}", spec.id);
                assert!(!seo.contributes_to_overall, "{}", spec.id);
            }

            // Export projections: single report from the snapshot, then the
            // batch built from the same report.
            let json: Value =
                serde_json::from_str(&format_json_replay(&snapshot, false).unwrap()).unwrap();
            assert_published(&json, &expected, "replay JSON");

            let batch = BatchReport::from_reports(vec![(*report).clone()], vec![], 100);
            let batch_json: Value =
                serde_json::from_str(&format_json_batch(&batch, false).unwrap()).unwrap();
            assert_published(&batch_json, &expected, "batch JSON");
        }
    }
}

/// An injected collection failure may not be repaired by normalization:
/// the contract rejects a report whose scoring collection failed but whose
/// module score claims a measurement.
#[test]
fn a_failed_scoring_collection_cannot_become_a_measured_score() {
    for spec in COLLECTORS.iter().filter(|s| s.scoring) {
        let report = report_with(record(
            spec,
            CollectionState::Failed,
            Some("evaluation_failed"),
            &[],
        ));
        let ctx = normalize_checked(&report)
            .unwrap_or_else(|e| panic!("{}: a honest partial report is valid: {e}", spec.id));
        assert!(
            ctx.normalized
                .execution
                .module_runs
                .iter()
                .any(|run| run.module == spec.module && run.status != ExecutionStatus::Completed),
            "{}: the failed scoring collection must not leave a completed run",
            spec.id
        );
    }
}
