//! The declared library surface (#940): the modules of `src/lib.rs` against the table in
//! `tests/fixtures/architecture/library_surface.txt` and `docs/reference/api.md`.
//!
//! * A `pub mod` that the table does not list fails: a new module is `pub(crate)`, or it is added
//!   to the table as `curated` on purpose (and to the table in api.md).
//! * A table line that disagrees with `lib.rs` fails, so a narrowed module has to be moved to
//!   `crate` and a widened one cannot slip back unnoticed.
//! * `queued` is a shrinking list, not a baseline: its count may not exceed `QUEUED_CEILING`,
//!   which is lowered with every module that leaves it and never raised.
//! * Every module of the table has a row in the api.md table.
//!
//! The last test is the floor of the surface: the paths `auditmysite_studio` imports (inventory
//! of #940). It compiles only while they resolve as public.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

/// Modules that are `pub` only because a binary, a test or Studio imports from them. Lower it
/// when one is narrowed; do not raise it.
const QUEUED_CEILING: usize = 24;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Tier {
    Curated,
    Queued,
    Crate,
}

fn read(rel: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(rel))
        .unwrap_or_else(|e| panic!("{rel}: {e}"))
}

/// `name -> is_pub` for every `pub mod x;` / `pub(crate) mod x;` line of `lib.rs`.
fn lib_modules(lib_rs: &str) -> BTreeMap<String, bool> {
    let mut modules = BTreeMap::new();
    for line in lib_rs.lines() {
        let line = line.trim();
        let (is_pub, rest) = if let Some(rest) = line.strip_prefix("pub mod ") {
            (true, rest)
        } else if let Some(rest) = line.strip_prefix("pub(crate) mod ") {
            (false, rest)
        } else {
            continue;
        };
        if let Some(name) = rest.strip_suffix(';') {
            modules.insert(name.trim().to_string(), is_pub);
        }
    }
    modules
}

fn parse_table(table: &str) -> Result<BTreeMap<String, Tier>, String> {
    let mut rows = BTreeMap::new();
    for line in table.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut words = line.split_whitespace();
        let (Some(name), Some(tier)) = (words.next(), words.next()) else {
            return Err(format!("`{line}`: expected `<module> <tier>`"));
        };
        let tier = match tier {
            "curated" => Tier::Curated,
            "queued" => Tier::Queued,
            "crate" => Tier::Crate,
            other => return Err(format!("{name}: unknown tier `{other}`")),
        };
        if rows.insert(name.to_string(), tier).is_some() {
            return Err(format!("{name}: listed twice"));
        }
    }
    Ok(rows)
}

fn surface_problems(lib_rs: &str, table: &str, api_md: &str, queued_ceiling: usize) -> Vec<String> {
    let rows = match parse_table(table) {
        Ok(rows) => rows,
        Err(e) => return vec![e],
    };
    let modules = lib_modules(lib_rs);
    let mut problems = Vec::new();
    for (name, is_pub) in &modules {
        match (rows.get(name), is_pub) {
            (None, true) => problems.push(format!(
                "`pub mod {name}` is not in the table: make it `pub(crate)`, or add it as \
                 `curated` with its items in docs/reference/api.md"
            )),
            (Some(Tier::Crate), true) => problems.push(format!(
                "`{name}` is `crate` in the table but `pub` in lib.rs: it was widened again"
            )),
            (Some(Tier::Curated | Tier::Queued), false) => problems.push(format!(
                "`{name}` is `pub(crate)` in lib.rs: move it to `crate` in the table"
            )),
            _ => {}
        }
    }
    for name in rows.keys().filter(|n| !modules.contains_key(*n)) {
        problems.push(format!(
            "`{name}` is in the table but not a module of lib.rs"
        ));
    }
    let queued = rows.values().filter(|t| **t == Tier::Queued).count();
    if queued > queued_ceiling {
        problems.push(format!(
            "{queued} modules are `queued`, the ceiling is {queued_ceiling}: queued only shrinks"
        ));
    }
    for name in rows.keys() {
        let row = format!("| `{name}` |");
        if !api_md.lines().any(|l| l.starts_with(&row)) {
            problems.push(format!(
                "`{name}` has no row `{row} …` in the table of docs/reference/api.md"
            ));
        }
    }
    problems
}

#[test]
fn lib_rs_declares_exactly_the_modules_of_the_surface_table() {
    let problems = surface_problems(
        &read("src/lib.rs"),
        &read("tests/fixtures/architecture/library_surface.txt"),
        &read("docs/reference/api.md"),
        QUEUED_CEILING,
    );
    assert!(
        problems.is_empty(),
        "library surface out of step (#940):\n  {}",
        problems.join("\n  ")
    );
}

#[test]
fn the_check_rejects_new_widened_stale_and_grown_entries() {
    let lib = "pub mod a;\npub(crate) mod b;\npub mod c;\n";
    let api = "| `a` | x |\n| `b` | x |\n| `c` | x |\n";
    let ok = "a curated\nb crate\nc queued\n";
    assert!(surface_problems(lib, ok, api, 1).is_empty());

    // a new public module that nobody listed
    let new = surface_problems("pub mod a;\npub mod d;\n", "a curated\n", "| `a` |", 1);
    assert_eq!(new.len(), 1, "{new:?}");
    assert!(new[0].contains("`pub mod d`"));

    // a narrowed module still listed as public, and a widened one listed as `crate`
    let drift = surface_problems(
        "pub(crate) mod a;\npub mod b;\n",
        "a queued\nb crate\n",
        api,
        9,
    );
    assert_eq!(drift.len(), 2, "{drift:?}");

    // a table line without a module, a module without an api.md row, a bad tier
    let stale = surface_problems("pub mod a;\n", "a curated\nz crate\n", "| `a` |", 1);
    assert!(stale.iter().any(|p| p.contains("`z` is in the table")));
    assert!(stale.iter().any(|p| p.contains("`z` has no row")));
    assert!(!surface_problems("pub mod a;\n", "a curated\n", "", 1).is_empty());
    assert!(!surface_problems("pub mod a;\n", "a maybe\n", "| `a` |", 1).is_empty());

    // the queued list may not grow past its ceiling
    let grown = surface_problems(lib, ok, api, 0);
    assert_eq!(grown.len(), 1, "{grown:?}");
}

/// The files that define the report model and the public enums of the curated surface: every
/// `pub struct` and `pub enum` in them is `#[non_exhaustive]`, so a new field or variant is a
/// minor change (docs/reference/api.md, stability policy point 2).
const NON_EXHAUSTIVE_FILES: &[&str] = &[
    "src/audit/baseline.rs",
    "src/audit/batch.rs",
    "src/audit/batch_consistency.rs",
    "src/audit/budget.rs",
    "src/audit/collection.rs",
    "src/audit/exclusion.rs",
    "src/audit/experimental.rs",
    "src/audit/finding_contract.rs",
    "src/audit/finding_derive.rs",
    "src/audit/frames.rs",
    "src/audit/interactive_finding.rs",
    "src/audit/interpretation.rs",
    "src/audit/normalized.rs",
    "src/audit/report.rs",
    "src/audit/scoring.rs",
    "src/audit/template_dedup.rs",
    "src/audit/validation.rs",
    "src/audit/verdict.rs",
    "src/error.rs",
    "src/model/annex_kind.rs",
    "src/model/color_policy.rs",
    "src/model/interactive_mode.rs",
    "src/model/report_level.rs",
    "src/model/wcag_level.rs",
    "src/wcag/types.rs",
];

/// Types in those files that stay exhaustive.
const EXHAUSTIVE: &[&str] = &[
    // Closed on purpose, so a `match` on them can be complete (api.md lists them).
    "WcagLevel",
    "ColorPolicy",
    // No public fields: nothing to add that could break a literal.
    "Baseline",
    "AccessibilityScorer",
    // Not named by a report field: configuration, the baseline file, collection and exclusion
    // bookkeeping and batch output views of the pipeline. Uncovered, narrowed with `audit`.
    "BaselinePage",
    "BaselineFile",
    "BatchConfig",
    "BudgetConfig",
    "CollectorSpec",
    "Collected",
    "CollectionRecord",
    "CollectionLog",
    "SelectorMatch",
    "ExclusionScope",
    "ExclusionTally",
    "Pass",
    "BatchExclusionSummary",
    "BatchExclusionSelector",
    "DomainPolicy",
    "FrameObservation",
    "InterpretArea",
    "RuleMetadata",
];

/// `pub struct` / `pub enum` names of `src` whose attribute block lacks `#[non_exhaustive]`.
fn exhaustive_types(src: &str) -> Vec<String> {
    let lines: Vec<&str> = src.lines().collect();
    let mut found = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        let rest = match line
            .strip_prefix("pub struct ")
            .or_else(|| line.strip_prefix("pub enum "))
        {
            Some(rest) => rest,
            None => continue,
        };
        let name: String = rest
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        // The doc comment and attributes above the item, up to the previous item or a blank line.
        let marked = lines[..i]
            .iter()
            .rev()
            .take_while(|l| {
                let l = l.trim_end();
                !l.trim().is_empty() && !l.ends_with('}') && !l.ends_with(';')
            })
            .any(|l| l.trim() == "#[non_exhaustive]");
        if !marked {
            found.push(name);
        }
    }
    found
}

#[test]
fn the_report_model_is_non_exhaustive() {
    let mut problems = Vec::new();
    for file in NON_EXHAUSTIVE_FILES {
        for name in exhaustive_types(&read(file)) {
            if !EXHAUSTIVE.contains(&name.as_str()) {
                problems.push(format!("{file}: `{name}`"));
            }
        }
    }
    assert!(
        problems.is_empty(),
        "public types of the report model without #[non_exhaustive] (#940): add the attribute, \
         or list the type in EXHAUSTIVE with its reason:\n  {}",
        problems.join("\n  ")
    );
}

#[test]
fn the_non_exhaustive_check_reads_the_attribute_block() {
    let src = "/// Doc.\n#[derive(Debug)]\n#[non_exhaustive]\npub struct A {\n    x: u8,\n}\n\n\
               #[derive(Debug)]\npub enum B {\n    X,\n}\n\
               #[non_exhaustive]\npub struct C;\npub struct D;\n";
    assert_eq!(exhaustive_types(src), vec!["B", "D"]);
}

/// The paths `auditmysite_studio` imports from this crate (inventory of #940). A removed or
/// narrowed path fails the build of this test, not Studio's. `normalize` is deprecated and stays
/// until Studio moves to `normalize_checked`.
#[allow(unused_imports, deprecated)]
mod studio_imports {
    pub use auditmysite::audit::{normalize, run_single_audit, AuditReport, PipelineConfig};
    pub use auditmysite::browser::{BrowserManager, BrowserOptions};
    pub use auditmysite::cli::{InteractiveMode, WcagLevel};
    #[cfg(feature = "pdf")]
    pub use auditmysite::output::generate_pdf;
    pub use auditmysite::output::report_model::ReportConfig;
    pub use auditmysite::studio::{StudioAuditResponse, StudioHistoryEntry, StudioReportArtifacts};
    pub use auditmysite::{
        format_json_normalized, parse_sitemap, run_concurrent_batch, BatchConfig,
        BrowserOptions as RootBrowserOptions, PoolConfig,
    };
}

#[test]
fn the_paths_studio_imports_resolve() {
    // The imports above are the assertion; naming the module keeps it from being optimised away.
    let _ = std::any::type_name::<studio_imports::PipelineConfig>();
}
