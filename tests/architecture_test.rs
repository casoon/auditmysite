//! Direction of dependencies (#790, plan 96): analysis modules do not reach
//! up into `cli`, `output` or `audit`. `audit` imports every analysis module,
//! so any import back is a cycle (#932).
//!
//! Existing backward imports are a baseline that may only shrink
//! (`tests/fixtures/architecture/backward_imports.txt`, `path count`). A new
//! one — in a new file or beyond a file's count — fails this test, as does a
//! baseline entry that no longer exists, so the list stays honest.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Modules that collect or evaluate; they sit below `audit` and `output`.
const ANALYSIS_MODULES: &[&str] = &[
    "a11y_journey",
    "accessibility",
    "ai_visibility",
    "best_practices",
    "commerce",
    "content_visibility",
    "mobile",
    "page_flow",
    "patterns",
    "performance",
    "privacy",
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

/// Code lines (not comments) of `files` containing any of `needles`, per file.
fn count_imports_in(files: &[PathBuf], needles: &[&str]) -> BTreeMap<String, usize> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut counts = BTreeMap::new();
    for file in files {
        let n = fs::read_to_string(file)
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
    counts
}

/// Code lines (not comments) containing any of `needles`, per file of the
/// analysis modules.
fn count_imports(needles: &[&str]) -> BTreeMap<String, usize> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    for module in ANALYSIS_MODULES {
        let dir = root.join("src").join(module);
        if dir.is_dir() {
            rust_files(&dir, &mut files);
        }
    }
    count_imports_in(&files, needles)
}

fn baseline(name: &str) -> BTreeMap<String, usize> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/architecture")
        .join(name);
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

/// Upward imports analysis modules must not gain; one line counts once.
const BACKWARD_IMPORTS: &[&str] = &["crate::cli", "crate::output", "crate::audit"];

/// Differences between the imports found and the baseline.
fn ratchet_problems(
    actual: &BTreeMap<String, usize>,
    allowed: &BTreeMap<String, usize>,
) -> Vec<String> {
    let mut problems = Vec::new();
    for (file, n) in actual {
        match allowed.get(file) {
            None => problems.push(format!("{file}: {n} new import(s)")),
            Some(max) if n > max => problems.push(format!("{file}: {n} imports, baseline {max}")),
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
fn analysis_modules_do_not_gain_imports_of_cli_output_or_audit() {
    let problems = ratchet_problems(
        &count_imports(BACKWARD_IMPORTS),
        &baseline("backward_imports.txt"),
    );
    assert!(
        problems.is_empty(),
        "dependency direction (plan 96), imports of cli/output/audit:\n{}",
        problems.join("\n")
    );
}

/// `cli` is the entry layer: nothing outside it and `main.rs` names it. No
/// baseline, every line is a failure (#934).
#[test]
fn no_module_outside_cli_gains_imports_of_cli() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    rust_files(&root.join("src"), &mut files);
    files.retain(|f| !f.starts_with(root.join("src/cli")) && *f != root.join("src/main.rs"));
    let found = count_imports_in(&files, &["crate::cli"]);
    assert!(
        found.is_empty(),
        "crate::cli is named outside cli/ (#934): {found:?}"
    );
}

/// `model` holds the shared value types and sits below every other module.
#[test]
fn model_imports_nothing_else_from_the_crate() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    rust_files(&root.join("src/model"), &mut files);
    assert!(!files.is_empty(), "src/model has no Rust files");
    let mut problems = Vec::new();
    for file in files {
        for (i, line) in fs::read_to_string(&file).unwrap().lines().enumerate() {
            if line.trim_start().starts_with("//") {
                continue;
            }
            if line.replace("crate::model", "").contains("crate::") {
                problems.push(format!("{}:{}: {}", file.display(), i + 1, line.trim()));
            }
        }
    }
    assert!(
        problems.is_empty(),
        "model must not import from other modules of the crate:\n{}",
        problems.join("\n")
    );
}

/// The interaction phases depend downwards only (#938): `patterns` detects,
/// `a11y_journey` exercises what `patterns` found with the primitives of
/// `interaction`. So `patterns` and `interaction` import neither of the
/// phases above them nor each other. No baseline, every line is a failure.
#[test]
fn patterns_interaction_a11y_journey_depend_downwards_only() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let forbidden: [(&str, &[&str]); 2] = [
        ("patterns", &["crate::a11y_journey", "crate::interaction"]),
        ("interaction", &["crate::a11y_journey", "crate::patterns"]),
    ];
    for (module, needles) in forbidden {
        let mut files = Vec::new();
        rust_files(&root.join("src").join(module), &mut files);
        assert!(!files.is_empty(), "src/{module} has no Rust files");
        let found = count_imports_in(&files, needles);
        assert!(
            found.is_empty(),
            "{module} must not import {needles:?} (detect -> exercise -> primitives, #938): {found:?}"
        );
    }
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
    // A new file importing cli/output/audit.
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
