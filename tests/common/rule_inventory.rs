//! Canonical inventory of every `rule_id` the accessibility audit can put on
//! a `Violation` (#552).
//!
//! There is no single existing registry for this. `taxonomy::rules::RULES`
//! is a curated scoring/report-classification table that undercounts (85
//! `Dimension::Accessibility` entries vs. the 116 found here); `PAGE_RULES`'s
//! `rule_id` field is documented in its own source as "used for logging
//! only"; `run_if_allowed!`'s axe_id argument in `engine.rs` is
//! `RuleRun` telemetry, not necessarily what lands on the `Violation`.
//!
//! The actual mechanism: almost every rule file defines a `RuleMetadata`
//! const, fed into a `Violation` via `.with_rule_id(rule.axe_id)` — this
//! covers the vast majority of ids regardless of *how* the check is invoked
//! (tree rule, DOM page rule, or directly pipeline-wired like
//! `contrast.rs`). Two documented exceptions skip `RuleMetadata` entirely:
//! - `aria_roles.rs` / `widget_rules.rs`: a handful of composite sub-checks
//!   use a bare `const <NAME>_AXE_ID: &str = "..."` instead.
//! - `src/patterns/*.rs` (accordion, modal dialog, tab list, disclosure
//!   menu): structural pattern-detection modules that build `Violation`s
//!   with a literal `rule_id` directly, no `RuleMetadata` at all.
//! - `src/wcag/shared.rs`: the `SHARED_RULES` table, whose ids come from the
//!   shared `a11y-rules` bestand and are therefore *not* axe-core ids but the
//!   cross-surface ones (`document/lang-missing`, ...). A rule that moves to
//!   the shared bestand leaves the axe_id namespace and enters this one.
//!
//! This scans the actual rule/pattern source files for all three shapes
//! rather than trusting any existing table. It is the ground truth the
//! detection-corpus completeness check (#554) diffs against.
//!
//! Caveat found empirically (see #552): `NormalizedFinding.rule_id`
//! (report-facing JSON) uses a *second*, parallel `a11y.*` id namespace
//! from `taxonomy::rules::RULES` for the same rules — this inventory
//! covers the `Violation.rule_id`/axe_id namespace only, which is closer to
//! the actual detection logic a ground-truth corpus verifies.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn wcag_rules_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("wcag")
        .join("rules")
}

/// `src/wcag/shared.rs` -- the bridge that runs the shared `a11y-rules`
/// bestand. Its `SHARED_RULES` table is the fourth registration shape: the
/// ids there are *not* axe-core ids but the cross-surface ids shared with
/// astro-post-audit and LiveAudit (`document/lang-missing`, ...).
fn shared_rules_file() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("wcag")
        .join("shared.rs")
}

fn patterns_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("patterns")
}

/// Read every `.rs` file directly inside `dir` (non-recursive — both
/// directories scanned here are flat).
fn read_rs_files(dir: &Path) -> Vec<(String, String)> {
    std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()))
        .map(|entry| entry.expect("dir entry").path())
        .filter(|path| path.extension().and_then(|e| e.to_str()) == Some("rs"))
        .map(|path| {
            let content = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
            (
                path.file_name().unwrap().to_string_lossy().into_owned(),
                content,
            )
        })
        .collect()
}

/// Extract the quoted string value of the first `field_name: "..."` in `block`.
fn extract_field_str(block: &str, field_name: &str) -> Option<String> {
    let marker = format!("{field_name}:");
    let start = block.find(&marker)? + marker.len();
    let rest = &block[start..];
    let quote_start = rest.find('"')?;
    let after_quote = &rest[quote_start + 1..];
    let quote_end = after_quote.find('"')?;
    Some(after_quote[..quote_end].to_string())
}

/// Extract every `<field>: "..."` value from each `<marker> { ... }` literal
/// in `source`, via brace-depth matching. Both struct literals scanned this
/// way (`RuleMetadata`, `SharedRule`) have only flat literals/slices as
/// fields (no nested `{}`), so depth counting is exact.
fn extract_struct_field_ids(source: &str, marker: &str, field: &str) -> Vec<String> {
    let mut ids = Vec::new();
    let mut search_from = 0;
    while let Some(rel_start) = source[search_from..].find(marker) {
        let block_start = search_from + rel_start + marker.len();
        let mut depth = 1i32;
        let mut end = source.len();
        for (i, c) in source[block_start..].char_indices() {
            match c {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        end = block_start + i;
                        break;
                    }
                }
                _ => {}
            }
        }
        let block = &source[block_start..end];
        if let Some(id) = extract_field_str(block, field) {
            ids.push(id);
        }
        search_from = (end + 1).min(source.len());
    }
    ids
}

/// Extract every bare `const <NAME>_AXE_ID: &str = "...";` declaration —
/// the handful of rules that skip `RuleMetadata` for a composite sub-check's id.
fn extract_bare_axe_id_consts(source: &str) -> Vec<String> {
    let mut ids = Vec::new();
    for line in source.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("const") && trimmed.contains("_AXE_ID") && trimmed.contains("&str") {
            if let Some(eq) = trimmed.find('=') {
                let rhs = &trimmed[eq + 1..];
                if let Some(q1) = rhs.find('"') {
                    let after = &rhs[q1 + 1..];
                    if let Some(q2) = after.find('"') {
                        ids.push(after[..q2].to_string());
                    }
                }
            }
        }
    }
    ids
}

/// Extract every `.with_rule_id("literal")` call where the argument is a
/// plain string literal (not a variable/field access) — catches the
/// pattern-detection modules under `src/patterns/`, which build `Violation`s
/// directly without a `RuleMetadata` const. Calls like
/// `.with_rule_id(RULE.axe_id)` or `.with_rule_id(rule_id)` are correctly
/// ignored here: their id is either already captured by
/// `extract_rule_metadata_axe_ids`, or (as with `iframe_rules.rs`'s
/// composite check) a re-use of an id that already exists elsewhere.
fn extract_literal_with_rule_id_calls(source: &str) -> Vec<String> {
    let mut ids = Vec::new();
    let marker = ".with_rule_id(\"";
    let mut search_from = 0;
    while let Some(rel_start) = source[search_from..].find(marker) {
        let value_start = search_from + rel_start + marker.len();
        match source[value_start..].find('"') {
            Some(rel_end) => {
                ids.push(source[value_start..value_start + rel_end].to_string());
                search_from = value_start + rel_end + 1;
            }
            None => break,
        }
    }
    ids
}

/// The canonical, deduplicated list of every rule_id the accessibility
/// engine can currently produce.
pub fn canonical_rule_ids() -> BTreeSet<String> {
    let mut ids = BTreeSet::new();

    for (_file, source) in read_rs_files(&wcag_rules_dir()) {
        ids.extend(extract_struct_field_ids(
            &source,
            "RuleMetadata {",
            "axe_id",
        ));
        ids.extend(extract_bare_axe_id_consts(&source));
    }
    for (_file, source) in read_rs_files(&patterns_dir()) {
        ids.extend(extract_literal_with_rule_id_calls(&source));
    }

    let shared = std::fs::read_to_string(shared_rules_file()).expect("cannot read shared.rs");
    ids.extend(extract_struct_field_ids(&shared, "SharedRule {", "id"));

    ids
}
