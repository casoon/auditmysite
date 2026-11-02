//! Build-time git short-SHA + dirty-state capture.
//!
//! Emits `AUDITMYSITE_BUILD_SHA` for compile-time embedding via `env!(...)`.
//! Falls back to `"unknown"` when `git`/`.git` isn't available (e.g. a
//! `cargo install` from a crates.io tarball, which has no `.git` directory).
//! Appends a `-dirty` suffix when the working tree has uncommitted changes,
//! so two builds off the same commit but different uncommitted local edits
//! remain distinguishable in a report's `build_id` field (plan/21) — this
//! project's usual workflow is to work uncommitted for a while before
//! committing, so the bare commit SHA alone previously stayed identical
//! across builds that actually contained different fixes.

use std::fs;
use std::process::Command;

fn main() {
    // Windows reserves 1 MB for the main thread, unix 8 MB. The audit future
    // lives on that stack and overflowed it in a debug build on a CI runner.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows")
        && std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc")
    {
        println!("cargo:rustc-link-arg-bins=/STACK:8388608");
    }

    let sha = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    let sha = match sha {
        Some(sha) if is_dirty() => format!("{sha}-dirty"),
        Some(sha) => sha,
        None => "unknown".to_string(),
    };

    println!("cargo:rustc-env=AUDITMYSITE_BUILD_SHA={sha}");

    // Re-run when HEAD moves (commit or branch switch) or the branch's own
    // ref file moves (commit on the currently checked-out branch).
    println!("cargo:rerun-if-changed=.git/HEAD");
    if let Ok(head) = fs::read_to_string(".git/HEAD") {
        if let Some(ref_path) = head.trim().strip_prefix("ref: ") {
            println!("cargo:rerun-if-changed=.git/{ref_path}");
        }
    }
    // The dirty check can flip on *any* working-tree edit, not just a
    // HEAD/ref move, so it must be re-evaluated on every build rather than
    // only when the two paths above change. A `rerun-if-changed` path that
    // never exists is a documented way to force Cargo to always re-run this
    // script (verified: cargo cannot establish a baseline mtime for it, so
    // it never counts as "unchanged").
    println!("cargo:rerun-if-changed=.auditmysite-build-rs-always-rerun-marker");
}

/// True when `git status --porcelain` reports any uncommitted change.
/// `git` missing, no `.git` directory, or any other failure is treated as
/// "not dirty" — mirrors the existing no-git fallback above (`sha =
/// "unknown"`), never a hard build failure.
fn is_dirty() -> bool {
    Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .is_some_and(|output| !output.stdout.is_empty())
}
