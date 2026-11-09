//! `--annotations` (#961): external assessments are shown next to findings and
//! never change a result. No browser needed: reports are built in memory.
// The infallible `normalize` is the deprecated 1.x compatibility API; these tests
// exercise the model it derives, not the migration path.
#![allow(deprecated)]

use auditmysite::annotations::{AnnotationFile, UnmatchedReason};
use auditmysite::audit::{normalize, AuditReport, BatchReport};
use auditmysite::cli::WcagLevel;
use auditmysite::output::{format_ai_json, format_ai_json_with_annotations, UnifiedReport};
use auditmysite::wcag::{Severity, Violation, WcagResults};
use auditmysite::AXTree;
use serde_json::{json, Value};
use std::path::Path;

fn report(url: &str, with_reading_flow: bool) -> AuditReport {
    let mut results = WcagResults::new();
    results.passes = 20;
    for i in 0..3 {
        results.add_violation(
            Violation::new(
                "4.1.2",
                "Name, Role, Value",
                WcagLevel::A,
                Severity::Critical,
                "Button missing accessible name",
                format!("node-{i}"),
            )
            .with_selector(format!("button.icon-{i}"))
            .with_fix("Add aria-label"),
        );
    }
    results.add_violation(
        Violation::new(
            "1.3.1",
            "Info and Relationships",
            WcagLevel::A,
            Severity::Medium,
            "Visual formatting instead of semantic HTML",
            "node-sem",
        )
        .with_selector("div.table-0"),
    );
    let mut report = AuditReport::new(url.to_string(), WcagLevel::AA, results, 1000);
    if with_reading_flow {
        report.screen_reader_audit = Some(auditmysite::screen_reader::build_sr_audit_report(
            &report.url,
            report.timestamp,
            &AXTree::from_nodes(Vec::new()),
            "en",
            report.patterns.as_ref(),
        ));
    }
    report
}

/// (finding id, one pattern id) of the finding with the given severity.
fn ids_of(report: &AuditReport, severity: Severity) -> (String, String) {
    let normalized = normalize(report).normalized;
    let finding = normalized
        .findings
        .iter()
        .find(|f| f.severity == severity)
        .expect("finding of this severity");
    (finding.id.clone(), finding.patterns[0].id.clone())
}

fn file(annotations: Value) -> AnnotationFile {
    let text = json!({ "schema": 1, "annotations": annotations }).to_string();
    AnnotationFile::parse(&text, "test.json", Path::new(".")).expect("valid annotation file")
}

fn annotation(finding_id: &str, kind: &str, assessment: &str, extra: Value) -> Value {
    let mut value = json!({
        "finding_id": finding_id,
        "author": { "kind": kind, "name": "tester" },
        "assessment": assessment,
        "reasoning": "Looked at the page: the control has no visible label."
    });
    value
        .as_object_mut()
        .unwrap()
        .extend(extra.as_object().cloned().unwrap_or_default());
    value
}

fn without_annotations(mut value: Value) -> Value {
    value.as_object_mut().unwrap().remove("annotations");
    value
}

fn single_json(report: &AuditReport, file: Option<&AnnotationFile>) -> Value {
    let normalized = normalize(report);
    let mut unified = UnifiedReport::single(&normalized, report);
    if let Some(file) = file {
        unified = unified.with_annotations(&file.resolve_normalized(&[&normalized.normalized]));
    }
    serde_json::from_str(&unified.to_json(true).unwrap()).unwrap()
}

#[test]
fn single_report_annotations_change_nothing_but_the_annotation_block() {
    let report = report("https://example.com/", true);
    let (critical, pattern) = ids_of(&report, Severity::Critical);
    let (medium, _) = ids_of(&report, Severity::Medium);
    let file = file(json!([
        annotation(&critical, "agent", "likely_issue", json!({ "pattern_id": pattern })),
        annotation(&medium, "human", "likely_fine", json!({})),
        {
            "scope": "reading_flow",
            "author": { "kind": "agent", "name": "tester" },
            "assessment": "undetermined",
            "reasoning": "Navigation announced twice."
        },
        annotation("F-00000000", "human", "undetermined", json!({})),
    ]));

    let plain = single_json(&report, None);
    let annotated = single_json(&report, Some(&file));

    assert!(plain.get("annotations").is_none(), "no option, no block");
    assert_eq!(plain, without_annotations(annotated.clone()));

    let block = &annotated["annotations"];
    assert_eq!(block["schema"], 1);
    assert_eq!(block["counts"]["total"], 4);
    assert_eq!(block["counts"]["matched"], 3);
    assert_eq!(block["counts"]["unmatched"], 1);
    assert_eq!(block["counts"]["by_author_kind"]["agent"], 2);
    assert_eq!(block["counts"]["by_author_kind"]["human"], 2);
    let matched = block["matched"].as_array().unwrap();
    assert_eq!(matched.len(), 3);
    assert!(matched.iter().any(|m| m["finding_id"] == critical.as_str()
        && m["pattern_id"] == pattern.as_str()
        && m["url"] == "https://example.com/"));
    assert!(matched.iter().any(|m| m["scope"] == "reading_flow"));
    assert_eq!(block["unmatched"][0]["reason"], "finding_not_found");
    assert_eq!(
        block["unmatched"][0]["annotation"]["finding_id"],
        "F-00000000"
    );
}

fn batch(reports: Vec<AuditReport>) -> BatchReport {
    BatchReport::from_reports(reports, vec![], 10)
}

fn batch_json(batch_report: &BatchReport, file: Option<&AnnotationFile>) -> Value {
    let mut unified = UnifiedReport::batch(batch_report);
    if let Some(file) = file {
        let normalized: Vec<_> = batch_report.reports.iter().map(normalize).collect();
        let refs: Vec<_> = normalized.iter().map(|n| &n.normalized).collect();
        unified = unified.with_annotations(&file.resolve_normalized(&refs));
    }
    serde_json::from_str(&unified.to_json(true).unwrap()).unwrap()
}

#[test]
fn batch_run_matches_per_page_and_changes_nothing_else() {
    let a = report("https://example.com/a", true);
    let b = report("https://example.com/b", false);
    let (critical, pattern) = ids_of(&a, Severity::Critical);
    let batch_report = batch(vec![a, b]);
    let file = file(json!([
        // Without url: every page with the finding.
        annotation(&critical, "agent", "likely_issue", json!({})),
        // With url: only that page.
        annotation(&critical, "human", "likely_fine", json!({ "url": "https://example.com/b" })),
        // A page that was not audited.
        annotation(&critical, "human", "likely_fine", json!({ "url": "https://example.com/zzz" })),
        // Pattern that does not exist on the finding.
        annotation(&critical, "agent", "undetermined", json!({ "pattern_id": "RC-00000000" })),
        // Reading flow needs a url in a multi-page run ...
        { "scope": "reading_flow", "author": { "kind": "agent", "name": "t" },
          "assessment": "undetermined", "reasoning": "r" },
        // ... and a page that has a screen-reader audit.
        { "scope": "reading_flow", "url": "https://example.com/a", "author": { "kind": "agent", "name": "t" },
          "assessment": "likely_issue", "reasoning": "r" },
        { "scope": "reading_flow", "url": "https://example.com/b", "author": { "kind": "agent", "name": "t" },
          "assessment": "likely_issue", "reasoning": "r" },
    ]));

    let plain = batch_json(&batch_report, None);
    let annotated = batch_json(&batch_report, Some(&file));
    assert!(plain.get("annotations").is_none());
    assert_eq!(plain, without_annotations(annotated.clone()));

    let block = &annotated["annotations"];
    let matched = block["matched"].as_array().unwrap();
    let on = |url: &str, scope: &str| {
        matched
            .iter()
            .filter(|m| m["url"] == url && m["scope"] == scope)
            .count()
    };
    assert_eq!(on("https://example.com/a", "finding"), 1);
    assert_eq!(on("https://example.com/b", "finding"), 2);
    assert_eq!(on("https://example.com/a", "reading_flow"), 1);
    assert_eq!(on("https://example.com/b", "reading_flow"), 0);

    let reasons: Vec<&str> = block["unmatched"]
        .as_array()
        .unwrap()
        .iter()
        .map(|u| u["reason"].as_str().unwrap())
        .collect();
    assert_eq!(
        reasons,
        [
            "url_not_audited",
            "pattern_not_found",
            "url_required",
            "no_reading_flow"
        ]
    );
    // Nothing is dropped: every annotation is either matched or unmatched.
    assert_eq!(block["counts"]["total"], 7);
    assert_eq!(
        block["counts"]["matched"].as_u64().unwrap()
            + block["counts"]["unmatched"].as_u64().unwrap(),
        7
    );
    let _ = pattern;
}

#[test]
fn resolve_reports_reasons() {
    let r = report("https://example.com/", false);
    let (critical, _) = ids_of(&r, Severity::Critical);
    let normalized = normalize(&r);
    let file = file(json!([annotation(
        &critical,
        "agent",
        "likely_fine",
        json!({ "pattern_id": "RC-00000000" })
    )]));
    let resolved = file.resolve_normalized(&[&normalized.normalized]);
    assert_eq!(
        resolved.unmatched[0].reason,
        UnmatchedReason::PatternNotFound
    );
}

#[test]
fn ai_tasks_carry_the_annotation_and_nothing_else_changes() {
    let r = report("https://example.com/", true);
    let (critical, pattern) = ids_of(&r, Severity::Critical);
    let normalized = normalize(&r);
    let file = file(json!([
        annotation(
            &critical,
            "agent",
            "likely_issue",
            json!({ "pattern_id": pattern })
        ),
        annotation("F-00000000", "human", "likely_fine", json!({})),
    ]));
    let resolved = file.resolve_normalized(&[&normalized.normalized]);

    let plain: Value = serde_json::from_str(&format_ai_json(&r).unwrap()).unwrap();
    let annotated: Value =
        serde_json::from_str(&format_ai_json_with_annotations(&r, Some(&resolved)).unwrap())
            .unwrap();
    assert!(plain.get("annotations").is_none());
    assert!(plain["tasks"]
        .as_array()
        .unwrap()
        .iter()
        .all(|t| t.get("annotations").is_none() && t.get("finding_id").is_none()));

    // Strip the annotation fields: the rest is identical.
    let mut stripped = annotated.clone();
    stripped.as_object_mut().unwrap().remove("annotations");
    for task in stripped["tasks"].as_array_mut().unwrap() {
        let task = task.as_object_mut().unwrap();
        task.remove("annotations");
        task.remove("finding_id");
    }
    assert_eq!(plain, stripped);

    let annotated_tasks: Vec<&Value> = annotated["tasks"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|t| t.get("annotations").is_some())
        .collect();
    // Only the tasks of the annotated pattern carry it (one selector pattern
    // of three `button.icon-N` buttons: they share a normalized pattern).
    assert!(!annotated_tasks.is_empty());
    for task in &annotated_tasks {
        assert_eq!(task["finding_id"], critical.as_str());
        assert_eq!(task["annotations"][0]["label"], "Assessment (agent)");
        assert_eq!(task["annotations"][0]["assessment"], "likely_issue");
    }
    assert_eq!(
        annotated["annotations"]["unmatched"][0]["reason"],
        "finding_not_found"
    );
}

#[cfg(feature = "pdf")]
mod pdf {
    use super::*;
    use auditmysite::output::report_model::ReportConfig;
    use auditmysite::output::{
        generate_batch_typ, generate_batch_typ_with_annotations, generate_typ,
        generate_typ_with_annotations,
    };

    fn config(locale: &str) -> ReportConfig {
        ReportConfig {
            locale: locale.to_string(),
            ..ReportConfig::default()
        }
    }

    #[test]
    fn single_typ_labels_assessments_by_author_kind_in_both_languages() {
        let r = report("https://example.com/", true);
        let (critical, _) = ids_of(&r, Severity::Critical);
        let normalized = normalize(&r);
        let file = file(json!([
            annotation(&critical, "agent", "likely_issue", json!({})),
            annotation(&critical, "human", "likely_fine", json!({})),
            { "scope": "reading_flow", "author": { "kind": "human", "name": "t" },
              "assessment": "undetermined", "reasoning": "Reading flow note." },
            annotation("F-00000000", "agent", "undetermined", json!({})),
        ]));
        let resolved = file.resolve_normalized(&[&normalized.normalized]);

        for (locale, agent, human, section, unmatched) in [
            (
                "en",
                "Assessment (agent): likely an issue",
                "Assessment (reviewer): likely fine",
                "Assessments",
                "Unmatched assessments",
            ),
            (
                "de",
                "Einschätzung (Agent): wahrscheinlich ein Problem",
                "Einschätzung (Reviewer): wahrscheinlich unproblematisch",
                "Einschätzungen",
                "Nicht zugeordnete Einschätzungen",
            ),
        ] {
            let plain = generate_typ(&r, &config(locale)).unwrap();
            let none = generate_typ_with_annotations(&r, &config(locale), None).unwrap();
            assert_eq!(plain, none, "no annotations: byte-identical ({locale})");
            assert!(!plain.contains(agent));

            let typ = generate_typ_with_annotations(&r, &config(locale), Some(&resolved)).unwrap();
            for needle in [agent, human, section, unmatched, "Reading flow note."] {
                assert!(typ.contains(needle), "{locale}: missing {needle:?}");
            }
        }
        // The English document stays free of German wording.
        let en = generate_typ_with_annotations(&r, &config("en"), Some(&resolved)).unwrap();
        assert!(!en.contains("Einschätzung"));
    }

    #[test]
    fn batch_typ_lists_assessments_and_unmatched() {
        let a = report("https://example.com/a", true);
        let b = report("https://example.com/b", false);
        let (critical, _) = ids_of(&a, Severity::Critical);
        let batch_report = batch(vec![a, b]);
        let normalized: Vec<_> = batch_report.reports.iter().map(normalize).collect();
        let refs: Vec<_> = normalized.iter().map(|n| &n.normalized).collect();
        let file = file(json!([
            annotation(
                &critical,
                "agent",
                "likely_issue",
                json!({ "url": "https://example.com/a" })
            ),
            annotation("F-00000000", "human", "undetermined", json!({})),
        ]));
        let resolved = file.resolve_normalized(&refs);

        let plain = generate_batch_typ(&batch_report, &config("en")).unwrap();
        assert_eq!(
            plain,
            generate_batch_typ_with_annotations(&batch_report, &config("en"), None).unwrap()
        );
        let typ =
            generate_batch_typ_with_annotations(&batch_report, &config("en"), Some(&resolved))
                .unwrap();
        for needle in [
            "Assessments",
            "Assessment (agent)",
            "Unmatched assessments",
            "F-00000000",
        ] {
            assert!(typ.contains(needle), "missing {needle:?}");
        }
    }
}

#[test]
fn a_bad_file_is_refused_before_the_audit_with_exit_code_2() {
    let dir = std::env::temp_dir().join(format!("ams-annotations-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let bad = dir.join("bad.json");
    std::fs::write(
        &bad,
        r#"{"schema":1,"annotations":[{"assessment":"fail"}]}"#,
    )
    .unwrap();
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_auditmysite"))
        .args([
            "--annotations",
            bad.to_str().unwrap(),
            "https://example.com",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("annotations[0]"), "{stderr}");

    let missing = std::process::Command::new(env!("CARGO_BIN_EXE_auditmysite"))
        .args([
            "--annotations",
            "/nonexistent/annotations.json",
            "https://example.com",
        ])
        .output()
        .unwrap();
    assert_eq!(missing.status.code(), Some(2));
    let _ = std::fs::remove_dir_all(dir);
}
