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
const QUEUED_CEILING: usize = 26;

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
