//! #1136: German rating labels were returned as `&'static str` literals and
//! showed up in English PDFs. They belong in `locales/*/report.ftl`.

use std::path::Path;

const LABELS: [&str; 8] = [
    "=> \"Stark\"",
    "=> \"Solide\"",
    "=> \"Uneinheitlich\"",
    "=> \"Schwach\"",
    "=> \"Tragfähige Basis\"",
    "=> \"Instabil\"",
    "=> \"Kritisch\"",
    "\"Intent-Fit\"",
];

fn scan(path: &Path, hits: &mut Vec<String>) {
    if path.is_dir() {
        for entry in std::fs::read_dir(path).unwrap() {
            scan(&entry.unwrap().path(), hits);
        }
        return;
    }
    let name = path.file_name().unwrap().to_string_lossy();
    if path.extension().is_none_or(|e| e != "rs") || name.contains("test") || name.contains("guard")
    {
        return;
    }
    let source = std::fs::read_to_string(path).unwrap();
    let code = source.split("#[cfg(test)]").next().unwrap();
    for label in LABELS {
        if code.contains(label) {
            hits.push(format!("{} holds the literal {label}", path.display()));
        }
    }
}

#[test]
fn output_code_has_no_german_rating_label_literals() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/output");
    let mut hits = Vec::new();
    scan(&root, &mut hits);
    assert!(hits.is_empty(), "{hits:#?}");
}
