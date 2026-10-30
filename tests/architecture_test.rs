//! Direction of dependencies (#790, plan 96): analysis modules do not reach
//! up into `cli`, `output` or the orchestration of `audit`.
//!
//! Existing backward imports are a baseline that may only shrink
//! (`tests/fixtures/architecture/backward_imports.txt`, `path count`). A new
//! one — in a new file or beyond a file's count — fails this test, as does a
//! baseline entry that no longer exists, so the list stays honest.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Modules that collect or evaluate; they sit below `audit` orchestration and
/// `output`.
const ANALYSIS_MODULES: &[&str] = &[
    "a11y_journey",
    "accessibility",
    "ai_visibility",
    "best_practices",
    "commerce",
    "content_visibility",
    "dark_mode",
    "design_quality",
    "journey",
    "mobile",
    "network",
    "patterns",
    "performance",
    "screen_reader",
    "security",
    "seo",
    "tech_stack",
    "ux",
    "wcag",
];

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap().flatten() {
        let path = entry.path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// Code lines (not comments) containing any of `needles`, per file.
fn count_imports(needles: &[&str]) -> BTreeMap<String, usize> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut counts = BTreeMap::new();
    for module in ANALYSIS_MODULES {
        let mut files = Vec::new();
        let dir = root.join("src").join(module);
        if dir.is_dir() {
            rust_files(&dir, &mut files);
        }
        for file in files {
            let n = fs::read_to_string(&file)
                .unwrap()
                .lines()
                .filter(|l| !l.trim_start().starts_with("//"))
                .filter(|l| needles.iter().any(|n| l.contains(n)))
                .count();
            if n > 0 {
                let rel = file
                    .strip_prefix(root)
                    .unwrap()
                    .to_string_lossy()
                    .to_string();
                counts.insert(rel, n);
            }
        }
    }
    counts
}

fn baseline() -> BTreeMap<String, usize> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/architecture/backward_imports.txt");
    fs::read_to_string(path)
        .unwrap()
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
        .map(|l| {
            let (path, n) = l.rsplit_once(' ').expect("`path count`");
            (path.to_string(), n.parse().expect("count"))
        })
        .collect()
}

/// Differences between the imports found and the baseline.
fn ratchet_problems(
    actual: &BTreeMap<String, usize>,
    allowed: &BTreeMap<String, usize>,
) -> Vec<String> {
    let mut problems = Vec::new();
    for (file, n) in actual {
        match allowed.get(file) {
            None => problems.push(format!("{file}: {n} new import(s) of cli/output")),
            Some(max) if n > max => {
                problems.push(format!("{file}: {n} imports of cli/output, baseline {max}"))
            }
            Some(max) if n < max => problems.push(format!(
                "{file}: now {n}, baseline {max} — lower the baseline"
            )),
            _ => {}
        }
    }
    for file in allowed.keys().filter(|f| !actual.contains_key(*f)) {
        problems.push(format!(
            "{file}: no import left — remove it from the baseline"
        ));
    }
    problems
}

#[test]
fn analysis_modules_do_not_gain_imports_of_cli_or_output() {
    let problems = ratchet_problems(
        &count_imports(&["crate::cli", "crate::output"]),
        &baseline(),
    );
    assert!(
        problems.is_empty(),
        "dependency direction (plan 96):\n{}",
        problems.join("\n")
    );
}

#[test]
fn the_ratchet_rejects_new_and_grown_imports_and_stale_entries() {
    let map = |items: &[(&str, usize)]| -> BTreeMap<String, usize> {
        items.iter().map(|(k, v)| (k.to_string(), *v)).collect()
    };
    let allowed = map(&[("src/seo/a.rs", 1), ("src/seo/gone.rs", 1)]);
    assert!(ratchet_problems(
        &map(&[("src/seo/a.rs", 1), ("src/seo/gone.rs", 1)]),
        &allowed
    )
    .is_empty());
    // A new file importing cli/output.
    assert_eq!(
        ratchet_problems(
            &map(&[
                ("src/seo/a.rs", 1),
                ("src/seo/gone.rs", 1),
                ("src/ux/new.rs", 1)
            ]),
            &allowed
        )
        .len(),
        1
    );
    // A file importing more than its baseline.
    assert_eq!(
        ratchet_problems(
            &map(&[("src/seo/a.rs", 2), ("src/seo/gone.rs", 1)]),
            &allowed
        )
        .len(),
        1
    );
    // A baseline entry nothing imports any more.
    assert_eq!(
        ratchet_problems(&map(&[("src/seo/a.rs", 1)]), &allowed).len(),
        1
    );
}

#[test]
fn analysis_modules_do_not_import_orchestration_or_persistence() {
    let found = count_imports(&[
        "crate::audit::pipeline",
        "crate::audit::batch",
        "crate::audit::artifacts",
    ]);
    assert!(
        found.is_empty(),
        "analysis modules must not import audit orchestration: {found:?}"
    );
}
