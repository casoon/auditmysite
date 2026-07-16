//! Output-coverage matrix (#508, Report Quality Layer).
//!
//! For every registered `AuditCatalog` module, counts how often its stable
//! `id()` appears as a literal string across four surfaces — PDF renderer
//! source, docs (prose + JSON schemas), and test fixtures/files — and writes
//! a reviewable artifact to `reports/coverage_matrix.json`.
//!
//! This is a static, substring-based signal (like `export_all_interpretations`
//! in `src/output/builder/mod.rs`), not a semantic one: it can false-negative
//! on indirection (a module referenced only through a shared helper that
//! doesn't spell out the id) and false-positive on an id that happens to be a
//! common word. It is deliberately **not** a CI gate yet — the matrix is
//! reviewed by a human first, the same way `reports/interpretations.json` is.

use auditmysite::audit::AuditCatalog;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize)]
struct CoverageEntry {
    module_id: String,
    module_label: String,
    pdf_reference_count: usize,
    docs_reference_count: usize,
    schema_reference_count: usize,
    fixture_reference_count: usize,
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Concatenates every `.rs` file under `dir` (recursively) into one string,
/// for a cheap whole-directory substring count. Read failures are skipped
/// silently — this is a best-effort coverage signal, not a build step.
fn concat_rs_files(dir: &Path) -> String {
    let mut combined = String::new();
    let Ok(entries) = fs::read_dir(dir) else {
        return combined;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            combined.push_str(&concat_rs_files(&path));
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            if let Ok(text) = fs::read_to_string(&path) {
                combined.push_str(&text);
            }
        }
    }
    combined
}

/// Concatenates every file directly inside `dir` (non-recursive), for
/// mixed-extension fixture directories (e.g. `.html`, `.json`).
fn concat_files_in(dir: &Path) -> String {
    let mut combined = String::new();
    let Ok(entries) = fs::read_dir(dir) else {
        return combined;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            if let Ok(text) = fs::read_to_string(&path) {
                combined.push_str(&text);
            }
        }
    }
    combined
}

fn build_matrix() -> Vec<CoverageEntry> {
    let root = repo_root();

    let pdf_text = concat_rs_files(&root.join("src/output/pdf"));
    let docs_text = fs::read_to_string(root.join("docs/OUTPUT_CONTRACT.md")).unwrap_or_default();
    let schema_text = [
        "docs/json-report.schema.json",
        "docs/json-batch-report.schema.json",
    ]
    .iter()
    .map(|f| fs::read_to_string(root.join(f)).unwrap_or_default())
    .collect::<Vec<_>>()
    .join("\n");
    let fixtures_text = concat_rs_files(&root.join("tests"))
        + &concat_files_in(&root.join("tests/fixtures"))
        + &concat_files_in(&root.join("tests/wcag_fixtures"));

    AuditCatalog::standard()
        .iter()
        .map(|module| {
            let id = module.id();
            CoverageEntry {
                module_id: id.to_string(),
                module_label: module.label().to_string(),
                pdf_reference_count: pdf_text.matches(id).count(),
                docs_reference_count: docs_text.matches(id).count(),
                schema_reference_count: schema_text.matches(id).count(),
                fixture_reference_count: fixtures_text.matches(id).count(),
            }
        })
        .collect()
}

#[test]
#[ignore = "run on demand to regenerate reports/coverage_matrix.json"]
fn export_coverage_matrix() {
    let matrix = build_matrix();
    let json = serde_json::to_string_pretty(&matrix).expect("serialize coverage matrix");
    let out_dir = repo_root().join("reports");
    fs::create_dir_all(&out_dir).expect("create reports/ dir");
    let out_path = out_dir.join("coverage_matrix.json");
    fs::write(&out_path, &json).expect("write coverage_matrix.json");
    println!(
        "Wrote {} module coverage entries to {}",
        matrix.len(),
        out_path.display()
    );
}

/// Regression tripwire on the catalog itself, not on coverage outcomes:
/// every registered module must have a non-empty, unique id. This is the
/// "small number of hard assertions" the coverage-matrix plan calls for —
/// scoped to something that cannot legitimately fail, unlike "every module
/// has PDF coverage" (which already fails today for `commerce`, a real gap
/// this matrix surfaced rather than something to silently gate on).
#[test]
fn every_catalog_module_has_a_unique_nonempty_id() {
    let catalog = AuditCatalog::standard();
    let mut seen = std::collections::HashSet::new();
    for module in catalog.iter() {
        let id = module.id();
        assert!(!id.is_empty(), "module id must not be empty");
        assert!(seen.insert(id), "duplicate module id: {id}");
    }
}
