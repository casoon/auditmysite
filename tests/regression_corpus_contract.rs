//! Contract test for the confirmed-corrections feedback corpus (#511).
//!
//! Each `tests/regression_corpus/*.json` file is a permanent record of a
//! bug/gap a human (or the report-critic skill) confirmed in this project's
//! reports. This test only validates the corpus's own shape — required
//! fields present, enums well-formed, ids unique/match filenames — it does
//! not re-run the underlying regressions themselves (those live as ordinary
//! `#[test]`s elsewhere, referenced from each entry's `regression` field).

use serde_json::Value;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

const VALID_CATEGORIES: &[&str] = &[
    "Invariant",
    "Semantic",
    "Completeness",
    "Explanation",
    "Visualization",
];
const VALID_STATUSES: &[&str] = &["resolved", "known_gap"];

fn corpus_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/regression_corpus")
}

fn load_entries() -> Vec<(String, Value)> {
    let dir = corpus_dir();
    let mut entries = Vec::new();
    for entry in fs::read_dir(&dir).unwrap_or_else(|e| panic!("{dir:?} must be readable: {e}")) {
        let entry = entry.expect("dir entry must be readable");
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let filename = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap()
            .to_string();
        let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path:?}: {e}"));
        let value: Value =
            serde_json::from_str(&text).unwrap_or_else(|e| panic!("{path:?}: invalid JSON: {e}"));
        entries.push((filename, value));
    }
    assert!(
        !entries.is_empty(),
        "expected at least one regression_corpus entry in {dir:?}"
    );
    entries
}

#[test]
fn corpus_entries_have_required_fields_with_valid_types() {
    for (filename, entry) in load_entries() {
        let obj = entry
            .as_object()
            .unwrap_or_else(|| panic!("{filename}: entry must be a JSON object"));

        for field in [
            "id", "category", "problem", "evidence", "expected", "status",
        ] {
            let value = obj
                .get(field)
                .unwrap_or_else(|| panic!("{filename}: missing required field {field:?}"));
            assert!(
                value.is_string(),
                "{filename}: field {field:?} must be a string"
            );
            assert!(
                !value.as_str().unwrap().trim().is_empty(),
                "{filename}: field {field:?} must not be empty"
            );
        }

        assert!(
            obj.contains_key("regression"),
            "{filename}: missing \"regression\" field (use null if not yet resolved)"
        );
        let regression = &obj["regression"];
        assert!(
            regression.is_null() || regression.is_string(),
            "{filename}: \"regression\" must be a string or null"
        );

        let counter_examples = obj
            .get("counter_examples")
            .unwrap_or_else(|| panic!("{filename}: missing \"counter_examples\" field"));
        assert!(
            counter_examples.is_array(),
            "{filename}: \"counter_examples\" must be an array"
        );
        assert!(
            !counter_examples.as_array().unwrap().is_empty(),
            "{filename}: \"counter_examples\" must not be empty (issue #511's acceptance \
             criteria requires counter-examples to prevent overbroad rules)"
        );
        for (i, example) in counter_examples.as_array().unwrap().iter().enumerate() {
            assert!(
                example.is_string() && !example.as_str().unwrap().trim().is_empty(),
                "{filename}: counter_examples[{i}] must be a non-empty string"
            );
        }
    }
}

#[test]
fn corpus_entry_ids_are_unique_and_match_their_filename() {
    let mut seen = HashSet::new();
    for (filename, entry) in load_entries() {
        let id = entry["id"].as_str().unwrap();
        assert_eq!(
            id, filename,
            "{filename}.json: \"id\" field ({id:?}) must match the filename"
        );
        assert!(seen.insert(id.to_string()), "duplicate corpus id: {id}");
    }
}

#[test]
fn corpus_entry_categories_and_statuses_use_known_enum_values() {
    for (filename, entry) in load_entries() {
        let category = entry["category"].as_str().unwrap();
        assert!(
            VALID_CATEGORIES.contains(&category),
            "{filename}: category {category:?} must be one of {VALID_CATEGORIES:?}"
        );

        let status = entry["status"].as_str().unwrap();
        assert!(
            VALID_STATUSES.contains(&status),
            "{filename}: status {status:?} must be one of {VALID_STATUSES:?}"
        );

        // A "resolved" entry claims a regression test/eval already guards it
        // (issue #511's acceptance criteria: "jeder behobene Fall besitzt
        // mindestens einen Regressionstest oder KI-Eval"). A "known_gap"
        // entry may still cite tooling that *tracks* the gap (e.g. the
        // coverage matrix) without that tooling actually closing it.
        if status == "resolved" {
            assert!(
                entry["regression"].is_string(),
                "{filename}: status \"resolved\" requires a non-null \"regression\" reference"
            );
        }
    }
}
