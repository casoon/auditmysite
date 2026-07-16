//! Contract test for the canonical metric registry (#506).
//!
//! Mirrors `tests/parity_contract.rs`'s "structured data + test asserting
//! code matches it" shape: every `MetricSpec` in `auditmysite::registry::REGISTRY`
//! must have a unique id, a `docs_url` anchor that actually exists in the
//! referenced doc file, a parseable `reviewed_at` date, and a `json_path`
//! that resolves against at least one of the two JSON report schemas.

use auditmysite::registry::{json_path_candidates, REGISTRY};
use serde_json::Value;
use std::collections::HashSet;
use std::fs;

#[test]
fn registry_ids_are_unique() {
    let mut seen = HashSet::new();
    for metric in REGISTRY {
        assert!(
            seen.insert(metric.id),
            "duplicate registry id: {}",
            metric.id
        );
    }
}

#[test]
fn registry_docs_urls_resolve_to_an_existing_anchor() {
    for metric in REGISTRY {
        let (path, anchor) = metric.docs_url.split_once('#').unwrap_or_else(|| {
            panic!(
                "{}: docs_url must be a local '<file>#<anchor>' path, got {:?}",
                metric.id, metric.docs_url
            )
        });
        assert!(
            !anchor.is_empty(),
            "{}: docs_url anchor must not be empty",
            metric.id
        );

        let repo_root = env!("CARGO_MANIFEST_DIR");
        let doc = fs::read_to_string(format!("{repo_root}/{path}")).unwrap_or_else(|e| {
            panic!("{}: docs_url file {path} must be readable: {e}", metric.id)
        });
        let needle = format!("id=\"{anchor}\"");
        assert!(
            doc.contains(&needle),
            "{}: docs_url anchor #{anchor} not found in {path} (expected an <a id=\"{anchor}\"> anchor)",
            metric.id
        );
    }
}

#[test]
fn registry_reviewed_at_parses_as_iso_date() {
    for metric in REGISTRY {
        chrono::NaiveDate::parse_from_str(metric.reviewed_at, "%Y-%m-%d").unwrap_or_else(|e| {
            panic!(
                "{}: reviewed_at {:?} must be an ISO YYYY-MM-DD date: {e}",
                metric.id, metric.reviewed_at
            )
        });
    }
}

fn resolve_ref<'a>(schema: &'a Value, node: &'a Value) -> &'a Value {
    match node.get("$ref").and_then(Value::as_str) {
        Some(r) => {
            let key = r.trim_start_matches("#/$defs/");
            schema
                .get("$defs")
                .and_then(|defs| defs.get(key))
                .unwrap_or(node)
        }
        None => node,
    }
}

/// Some registry entries (the count definitions) were ported from
/// `metric_context()` as bare, unqualified names (e.g. `"violation_count"`)
/// rather than full paths, and the same name is legitimately a property in
/// more than one `$defs` entry (e.g. both `summary` and a per-page detail
/// block). For a bare candidate, search every `properties` map in the schema
/// (recursing into `$defs`) for a matching key instead of requiring one exact
/// location.
fn bare_name_exists_anywhere(schema: &Value, name: &str) -> bool {
    fn walk(node: &Value, name: &str) -> bool {
        match node {
            Value::Object(map) => {
                if let Some(props) = map.get("properties").and_then(Value::as_object) {
                    if props.contains_key(name) {
                        return true;
                    }
                }
                map.values().any(|v| walk(v, name))
            }
            Value::Array(items) => items.iter().any(|v| walk(v, name)),
            _ => false,
        }
    }
    walk(schema, name)
}

/// Walks a dotted/bracketed path (e.g. `pages[].detail.modules.*.score`)
/// through a JSON Schema (`$defs`/`$ref`/`properties`/`items`), treating `*`
/// as an unvalidated wildcard segment. Returns `true` if the path resolves to
/// a defined property.
fn path_resolves(schema: &Value, path: &str) -> bool {
    if !path.contains('.') && !path.contains('[') {
        return bare_name_exists_anywhere(schema, path);
    }
    let mut current = schema;
    for raw_segment in path.split('.') {
        let segment = raw_segment.trim();
        if segment.is_empty() {
            continue;
        }
        let (name, is_array) = match segment.strip_suffix("[]") {
            Some(stripped) => (stripped, true),
            None => (segment, false),
        };
        if name.contains('*') {
            // Wildcard/glob segment (`*` or `*_score`): we can't validate an
            // arbitrary or pattern-matched key against `additionalProperties`
            // cleanly, so resolving the prefix up to here is treated as
            // sufficient evidence the path is real.
            return true;
        }
        let resolved = resolve_ref(schema, current);
        let props = match resolved.get("properties") {
            Some(p) => p,
            None => return false,
        };
        let next = match props.get(name) {
            Some(n) => n,
            None => return false,
        };
        current = resolve_ref(schema, next);
        if is_array {
            match current.get("items") {
                Some(items) => current = resolve_ref(schema, items),
                None => return false,
            }
        }
    }
    true
}

#[test]
fn registry_json_paths_resolve_in_a_report_schema() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let schemas: Vec<Value> = [
        "docs/json-report.schema.json",
        "docs/json-batch-report.schema.json",
    ]
    .iter()
    .map(|file| {
        let text = fs::read_to_string(format!("{repo_root}/{file}"))
            .unwrap_or_else(|e| panic!("{file} must be readable: {e}"));
        serde_json::from_str(&text).unwrap_or_else(|e| panic!("{file} must be valid JSON: {e}"))
    })
    .collect();

    for metric in REGISTRY {
        let candidates = json_path_candidates(metric.json_path);
        assert!(
            !candidates.is_empty(),
            "{}: json_path {:?} yielded no resolvable path candidates",
            metric.id,
            metric.json_path
        );
        let resolves = candidates.iter().any(|candidate| {
            schemas
                .iter()
                .any(|schema| path_resolves(schema, candidate))
        });
        assert!(
            resolves,
            "{}: none of {:?} (from json_path {:?}) resolve in either report schema",
            metric.id, candidates, metric.json_path
        );
    }
}
