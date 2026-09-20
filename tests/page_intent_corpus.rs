//! Classify every cached AXTree snapshot and dump the result (plan 50).
//!
//! Page-type classification has almost no test coverage, and the corpus in
//! `tests/fixtures/detection_corpus/` carries no page-type ground truth — so a
//! change to how `detect_page_intent` reads text cannot be judged from the
//! test suite alone. This walks a directory of `snapshot.json` artifacts
//! (`~/.auditmysite/cache` by default, thousands of real pages), classifies
//! each one, and writes `path \t nodes \t intent \t little_early_text` lines.
//! Run it before and after a change and diff the two files: that is the
//! before/after comparison this kind of change needs.
//!
//! `#[ignore]`-gated and a no-op when the directory is missing: it reads a
//! local artifact cache, which a fresh checkout or CI does not have.
//!
//! ```text
//! cargo test --test page_intent_corpus -- --ignored --nocapture
//! AUDITMYSITE_SNAPSHOT_DIR=/some/dir PAGE_INTENT_OUT=/tmp/after.tsv \
//!   cargo test --test page_intent_corpus -- --ignored
//! ```

use std::fs;
use std::path::{Path, PathBuf};

use auditmysite::accessibility::AXTree;
use auditmysite::journey::{analyze_journey, detect_page_intent, FrictionKind};

fn snapshot_dir() -> Option<PathBuf> {
    if let Ok(dir) = std::env::var("AUDITMYSITE_SNAPSHOT_DIR") {
        return Some(PathBuf::from(dir));
    }
    std::env::var("HOME")
        .ok()
        .map(|home| Path::new(&home).join(".auditmysite/cache"))
}

fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(&path, out);
        } else if path.file_name().is_some_and(|n| n == "snapshot.json") {
            out.push(path);
        }
    }
}

#[test]
#[ignore = "reads a local artifact cache; run manually with `-- --ignored`"]
fn classify_every_cached_snapshot() {
    let Some(dir) = snapshot_dir() else {
        eprintln!("no HOME and no AUDITMYSITE_SNAPSHOT_DIR — nothing to do");
        return;
    };
    if !dir.exists() {
        eprintln!("{} does not exist — nothing to do", dir.display());
        return;
    }

    let mut snapshots = Vec::new();
    collect(&dir, &mut snapshots);
    snapshots.sort();
    eprintln!("{} snapshots under {}", snapshots.len(), dir.display());

    let mut lines: Vec<String> = Vec::with_capacity(snapshots.len());
    let mut unreadable = 0usize;
    for path in &snapshots {
        let Ok(bytes) = fs::read(path) else {
            unreadable += 1;
            continue;
        };
        // Only the AX tree is needed, and old artifacts carry fields this
        // version no longer knows — parse it out rather than the whole
        // `SnapshotArtifact`.
        let Ok(value) = serde_json::from_slice::<serde_json::Value>(&bytes) else {
            unreadable += 1;
            continue;
        };
        let Some(tree) = value.get("ax_tree").cloned() else {
            unreadable += 1;
            continue;
        };
        let Ok(tree) = serde_json::from_value::<AXTree>(tree) else {
            unreadable += 1;
            continue;
        };
        let intent = detect_page_intent(&tree);
        // Journey's entry-clarity reads the top of the page, which is the
        // other text-measuring site plan 50 covers.
        let little_early_text = analyze_journey(&tree)
            .friction_points
            .iter()
            .any(|f| matches!(f.kind, FrictionKind::LittleEarlyText));
        lines.push(format!(
            "{}\t{}\t{}\t{}",
            path.strip_prefix(&dir).unwrap_or(path).display(),
            tree.len(),
            intent.label(true),
            little_early_text,
        ));
    }

    let out = std::env::var("PAGE_INTENT_OUT")
        .unwrap_or_else(|_| "/tmp/page_intent_corpus.tsv".to_string());
    fs::write(&out, lines.join("\n") + "\n").expect("write classification dump");
    eprintln!(
        "classified {} snapshots ({} unreadable) -> {}",
        lines.len(),
        unreadable,
        out
    );
}
