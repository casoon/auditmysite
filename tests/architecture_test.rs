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

/// `source` without the items gated by `#[cfg(test)]`: a module, function or
/// `use` that starts on such an attribute ends at its closing brace in the
/// attribute's column (rustfmt layout) or, for a one-line item, at its `;`.
fn without_cfg_test_items(source: &str) -> String {
    let mut out = String::new();
    let mut lines = source.lines();
    while let Some(line) = lines.next() {
        if line.trim() != "#[cfg(test)]" {
            out.push_str(line);
            out.push('\n');
            continue;
        }
        let column = line.len() - line.trim_start().len();
        let mut head = true;
        for item in lines.by_ref() {
            let trimmed = item.trim();
            if trimmed.starts_with("#[") || trimmed.is_empty() {
                continue;
            }
            let item_column = item.len() - item.trim_start().len();
            if head && trimmed.ends_with(';') {
                break;
            }
            head = false;
            if item_column == column && trimmed.starts_with('}') {
                break;
            }
        }
    }
    out
}

/// Lines (1-based) of `source` that build a `NormalizedFinding` with a struct
/// literal, not a `-> NormalizedFinding {` signature or an `impl` block.
fn normalized_finding_literals(source: &str) -> Vec<usize> {
    const NAME: &str = "NormalizedFinding";
    let mut found = Vec::new();
    for (i, line) in source.lines().enumerate() {
        if line.trim_start().starts_with("//") {
            continue;
        }
        let mut offset = 0;
        while let Some(at) = line[offset..].find(NAME) {
            let start = offset + at;
            let before = line[..start].trim_end();
            let after = line[start + NAME.len()..].trim_start();
            let type_position = ["->", "impl", "for", "struct"]
                .iter()
                .any(|w| before.ends_with(w));
            if after.starts_with('{') && !type_position {
                found.push(i + 1);
            }
            offset = start + NAME.len();
        }
    }
    found
}

/// Lines (1-based) of `source` that take a `NormalizedFinding` by mutable
/// reference: `&mut NormalizedFinding`, `&mut [NormalizedFinding]`,
/// `&mut Vec<NormalizedFinding>`, with or without a module path.
fn mutable_normalized_finding_refs(source: &str) -> Vec<usize> {
    let mut found = Vec::new();
    for (i, line) in source.lines().enumerate() {
        if line.trim_start().starts_with("//") {
            continue;
        }
        for (at, _) in line.match_indices("&mut ") {
            let ty = line[at + "&mut ".len()..]
                .split([',', ')', '=', '{', ';'])
                .next()
                .unwrap_or_default();
            if ty.contains("NormalizedFinding") {
                found.push(i + 1);
            }
        }
    }
    found
}

/// The non-test code of `src/output` as `(relative path, source)`.
fn output_code() -> Vec<(String, String)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    rust_files(&root.join("src/output"), &mut files);
    assert!(!files.is_empty(), "src/output has no Rust files");
    files
        .into_iter()
        // A file included as `#[cfg(test)] mod tests;` is test code as a whole.
        .filter(|f| f.file_name().is_none_or(|n| n != "tests.rs"))
        .map(|f| {
            let code = without_cfg_test_items(&fs::read_to_string(&f).unwrap());
            (f.strip_prefix(root).unwrap().display().to_string(), code)
        })
        .collect()
}

/// The output layer reads `NormalizedFinding`, it never builds one (#937): the
/// normalisation in `audit` owns the construction. No baseline, every literal
/// outside `#[cfg(test)]` code is a failure.
#[test]
fn output_never_constructs_a_normalized_finding() {
    let mut problems = Vec::new();
    for (file, code) in output_code() {
        for line in normalized_finding_literals(&code) {
            problems.push(format!(
                "{file}: `NormalizedFinding {{ .. }}` built in output (#937), line {line} after removing cfg(test) items"
            ));
        }
    }
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

/// Nor does it change one: the run-language copy of a report comes from
/// `audit::finding_text` (#937). No baseline, every `&mut` outside
/// `#[cfg(test)]` code is a failure.
#[test]
fn output_never_takes_a_normalized_finding_by_mutable_reference() {
    let mut problems = Vec::new();
    for (file, code) in output_code() {
        for line in mutable_normalized_finding_refs(&code) {
            problems.push(format!(
                "{file}: `&mut` NormalizedFinding in output (#937), line {line} after removing cfg(test) items"
            ));
        }
    }
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

#[test]
fn the_mutable_reference_check_sees_every_spelling_and_skips_shared_ones() {
    let source = "\
fn a(f: &mut NormalizedFinding) {}
fn b(f: &mut [NormalizedFinding], t: &Map) {}
fn c(f: &mut Vec<crate::audit::normalized::NormalizedFinding>) {}
fn d(f: &mut [crate::audit::normalized::NormalizedFinding]) {}
fn e(f: &NormalizedFinding, g: &[NormalizedFinding], h: &mut String) {}
";
    assert_eq!(mutable_normalized_finding_refs(source), vec![1, 2, 3, 4]);
}

#[test]
fn the_normalized_finding_check_sees_literals_and_skips_test_code() {
    let source = "\
fn build() -> NormalizedFinding {
    NormalizedFinding {
        id: String::new(),
    }
}
impl NormalizedFinding {
    fn m(&self) {}
}
#[cfg(test)]
mod tests {
    fn make() -> NormalizedFinding {
        NormalizedFinding { id: String::new() }
    }
}
#[cfg(test)]
use x::NormalizedFinding;
fn later() {
    let f = crate::audit::normalized::NormalizedFinding { id: 1 };
}
";
    let code = without_cfg_test_items(source);
    assert!(!code.contains("mod tests"));
    assert!(code.contains("fn later"));
    // The literals in `build` and `later`; not the signature, not the `impl`.
    assert_eq!(normalized_finding_literals(&code).len(), 2);
}
