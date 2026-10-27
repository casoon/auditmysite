//! Release contract tests.
//!
//! These tests keep the installer and release workflow aligned so published
//! artifacts can actually be installed.

use std::path::PathBuf;

fn read_repo_file(path: &str) -> String {
    let full_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path);
    std::fs::read_to_string(&full_path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", full_path.display(), e))
}

#[test]
fn test_installer_and_release_workflow_agree_on_unix_archive_name() {
    let install_sh = read_repo_file("install.sh");
    let release_yml = read_repo_file(".github/workflows/release.yml");

    assert!(
        install_sh.contains("archive_name=\"${BINARY_NAME}-${version}-${platform}.tar.gz\""),
        "install.sh must construct versioned Unix archive names"
    );
    assert!(
        release_yml.contains("ARCHIVE_NAME=\"${BINARY_NAME}-${VERSION}-${TARGET}.tar.gz\""),
        "release workflow must publish versioned Unix archive names"
    );
}

#[test]
fn test_installer_and_release_workflow_agree_on_checksum_name() {
    let install_sh = read_repo_file("install.sh");
    let release_yml = read_repo_file(".github/workflows/release.yml");

    assert!(
        install_sh.contains("checksum_name=\"${archive_name}.sha256\""),
        "install.sh must fetch archive checksum files"
    );
    assert!(
        release_yml.contains("${{ env.ARCHIVE_NAME }}.sha256"),
        "release workflow must upload checksum files next to the archive"
    );
}

#[test]
fn test_readme_documents_curl_installer() {
    let readme = read_repo_file("README.md");

    assert!(readme.contains("curl -fsSL"));
    assert!(readme.contains("install.sh | bash"));
    assert!(readme.contains(".sha256"));
}

#[test]
fn test_release_build_requires_quality_gates_to_pass_first() {
    // plan/25-release-workflow-quality-gate.md: a `v*` tag must not be able
    // to produce release binaries without the same commit having passed
    // fmt/clippy/tests/dependency-audit first. Text-based, not a real YAML
    // parse (matches this file's existing style) — but specific enough that
    // reverting the `build` job's `needs:` back to `verify-version` alone,
    // or removing the `quality-gates` job/its `uses:` of `ci.yml`, fails
    // this test rather than silently reopening the gap.
    let release_yml = read_repo_file(".github/workflows/release.yml");
    let ci_yml = read_repo_file(".github/workflows/ci.yml");

    assert!(
        release_yml.contains("uses: ./.github/workflows/ci.yml"),
        "release workflow must call the full CI workflow as its quality gate"
    );
    assert!(
        ci_yml.contains("workflow_call:"),
        "ci.yml must declare workflow_call so release.yml can call it"
    );

    let build_job = release_yml
        .split("\n  build:")
        .nth(1)
        .expect("release workflow must have a top-level `build:` job");
    // Only inspect this job's own body, not everything after it in the file.
    let build_job_body = build_job.split("\n  release:").next().unwrap_or(build_job);
    assert!(
        build_job_body.contains("needs:") && build_job_body.contains("quality-gates"),
        "the `build` job must depend on `quality-gates`, not just `verify-version`"
    );
}

/// Plan 94: `cross` is installed from a pinned commit with `--locked`,
/// never from the moving tip of cross-rs/main.
#[test]
fn test_cross_is_installed_pinned() {
    let release_yml = read_repo_file(".github/workflows/release.yml");
    for line in release_yml
        .lines()
        .filter(|l| l.contains("cargo install cross"))
    {
        assert!(line.contains("--rev "), "unpinned cross install: {line}");
        assert!(
            line.contains("--locked"),
            "cross install without --locked: {line}"
        );
    }
    assert!(release_yml.contains("cargo install cross"));
}

#[test]
fn test_release_build_uses_locked_lockfile() {
    // plan/25-release-workflow-quality-gate.md: release binaries must be
    // built from the exact checked-in Cargo.lock, not a silently
    // re-resolved one.
    let release_yml = read_repo_file(".github/workflows/release.yml");
    assert!(
        release_yml.contains("cargo build --release --locked"),
        "native release build must use --locked"
    );
    assert!(
        release_yml.contains("cross build --release --locked"),
        "cross-compiled release build must use --locked"
    );
}

#[test]
fn test_release_and_pre_push_use_shared_version_check() {
    let pre_push = read_repo_file("scripts/pre-push-check.sh");
    let release_yml = read_repo_file(".github/workflows/release.yml");
    let version_check = read_repo_file("scripts/check-version-match.sh");

    assert!(
        pre_push.contains("./scripts/check-version-match.sh")
            || pre_push.contains("\"$REPO_ROOT/scripts/check-version-match.sh\""),
        "pre-push must run the shared version check"
    );
    assert!(
        release_yml.contains("./scripts/check-version-match.sh"),
        "release workflow must run the shared version check"
    );
    assert!(
        version_check
            .contains("Tag version $tag_name does not match Cargo.toml version $CARGO_VERSION"),
        "shared version check must fail clearly on mismatched versions"
    );
}

/// Body of a top-level job in a workflow file: from `  <name>:` up to the
/// next top-level job (or the end of the file).
fn workflow_job_body<'a>(workflow: &'a str, job: &str) -> &'a str {
    let header = format!("\n  {job}:\n");
    let start = workflow
        .find(&header)
        .unwrap_or_else(|| panic!("workflow must have a top-level `{job}:` job"))
        + header.len();
    let rest = &workflow[start..];
    let end = rest
        .match_indices("\n  ")
        .find(|(i, _)| {
            let line = &rest[i + 3..];
            !line.starts_with(' ') && !line.starts_with('#')
        })
        .map_or(rest.len(), |(i, _)| i);
    &rest[..end]
}

/// The job-level `if:` condition, if the job has one.
fn job_condition(job_body: &str) -> Option<&str> {
    job_body
        .lines()
        .find_map(|line| line.strip_prefix("    if:"))
        .map(str::trim)
}

#[test]
fn test_release_quality_gate_runs_browser_tests_but_not_coverage() {
    // plan/30-release-browser-gate-for-tag-sha.md: release.yml calls ci.yml
    // as its quality gate, so every CI job that can skip itself can also skip
    // the release gate. In a called workflow the `github` context is the
    // caller's (`event_name == 'push'`, `ref == 'refs/tags/v*'`), so a guard
    // on `event_name` does not tell the release call apart — the only safe
    // contract for `browser-smoke` is to have no job-level condition at all.
    let ci_yml = read_repo_file(".github/workflows/ci.yml");

    let browser_smoke = workflow_job_body(&ci_yml, "browser-smoke");
    assert!(
        browser_smoke.contains("--test integration_test"),
        "`browser-smoke` must run the Chrome integration tests"
    );
    // The WCAG corpus runs sharded in its own job; every shard must run it.
    let corpus = workflow_job_body(&ci_yml, "detection-corpus");
    assert!(
        corpus.contains("--test detection_corpus_test")
            && corpus.contains("shard: [1, 2, 3]")
            && corpus.contains("CORPUS_SHARD: ${{ matrix.shard }}/3"),
        "`detection-corpus` must run every shard of the WCAG detection corpus"
    );
    for (job, body) in [
        ("browser-smoke", browser_smoke),
        ("detection-corpus", corpus),
    ] {
        assert_eq!(
            job_condition(body),
            None,
            "`{job}` must not carry a job-level `if:` — it would be able to \
             skip the browser tests for the tagged release commit"
        );
    }

    // Coverage is a measurement, not a release gate: skipped on tag refs.
    let coverage = workflow_job_body(&ci_yml, "coverage");
    assert!(
        job_condition(coverage)
            .is_some_and(|c| c.contains("!startsWith(github.ref, 'refs/tags/')")),
        "`coverage` must be skipped when ci.yml runs for a release tag"
    );
}

#[test]
fn release_requires_complete_detector_contracts_and_mutation_check() {
    // The contract gate runs in every plain `cargo test` (never ignored), so
    // CI's test jobs and the local release check enforce it alike.
    let gate = read_repo_file("tests/rule_contract_gate.rs");
    let body = gate
        .split("fn every_active_detector_has_a_complete_contract")
        .next()
        .expect("contract gate missing");
    let attributes = body.rsplit("#[test]").next().unwrap_or_default();
    assert!(
        !attributes.contains("#[ignore"),
        "the contract gate must not be ignored"
    );
    assert!(gate.contains("deferred entries never pass the gate"));

    let local = read_repo_file("scripts/release-check.sh");
    assert!(local.contains("cargo test --test rule_contract_gate"));
    assert!(local.contains("scripts/mutation-check.py"));

    let ci = read_repo_file(".github/workflows/ci.yml");
    let mutation = workflow_job_body(&ci, "mutation-check");
    assert!(mutation.contains("python3 scripts/mutation-check.py"));
    assert!(mutation.contains("timeout-minutes"));
}

#[test]
fn release_requires_completed_v2_milestone_before_build() {
    let workflow = read_repo_file(".github/workflows/release.yml");
    let verify = workflow_job_body(&workflow, "verify-version");
    assert!(verify.contains("check-v2-readiness.py --version \"$GITHUB_REF_NAME\""));
    assert!(!verify.contains("continue-on-error"));
    assert!(
        workflow_job_body(&workflow, "build").contains("needs: [verify-version, quality-gates]")
    );
    assert!(read_repo_file("scripts/release-check.sh").contains("check-v2-readiness.py"));
    let result = std::process::Command::new("python3")
        .arg("scripts/test_check_v2_readiness.py")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Python 3.11+ is required for release gate tests");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
}
