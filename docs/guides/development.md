---
title: "Development"
description: "Setup, pre-commit checks, debugging report content, release checks and the library API."
order: 11
---

## Setup

```bash
git clone https://github.com/casoon/auditmysite.git
cd auditmysite
cargo test
cargo build --release
./target/release/auditmysite https://example.com
```

## Pre-commit checks

This repository uses Git hooks with a fast local `pre-commit` gate and a full `pre-push` gate.

`pre-commit` runs:

- `nosecrets` on staged changes
- `rustfmt --check` on the staged `.rs` files
- `cargo clippy --lib --bins --all-features -- -D warnings`

`pre-push` runs:

- a guard that refuses the push while a local `[patch]` to `../barrierlab` is
  active in `.cargo/config.toml`, or `Cargo.lock` lists barrierlab crates without
  a crates.io source (not skippable via `SKIP_RUST_CHECKS`)
- `scripts/check-version-match.sh` for pushed `v*` tags
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test`

Enable the repo hook path:

```bash
git config core.hooksPath .githooks
```

Install `nosecrets` as a real binary first:

```bash
npm install -g @casoon/nosecrets
# or
cargo install nosecrets-cli
```

Skip the Rust checks only when you intentionally need to bypass them:

```bash
SKIP_RUST_CHECKS=1 git commit -m "..."
```

The hook expects `nosecrets` to be available in `PATH`.

## Debugging report content (hidden `--debug-typ`)

PDF reports are rendered through the `renderreport`/Typst engine. To review report
**completeness and wording** without opening the binary PDF, use the hidden
`--debug-typ` flag together with `--format pdf`. It writes the intermediate Typst
source as a `.typ` sidecar next to the PDF, for both single and batch reports:

```bash
# Single report → reports/example-audit.pdf + reports/example-audit.typ
./target/release/auditmysite https://example.com --full --format pdf \
  --output reports/example-audit.pdf --debug-typ

# Batch report → reports/example-batch.pdf + reports/example-batch.typ
./target/release/auditmysite --sitemap https://example.com/sitemap.xml --full \
  --format pdf --output reports/example-batch.pdf --debug-typ
```

The `.typ` file is plain text and diff-friendly — useful for checking which audits
land in the report and reviewing the exact wording of every section. The flag is
intentionally hidden from `--help` (developer/debug use only).

## Repository CI

`quick-ci.yml` runs on pull requests targeting `main` and pushes to `main`.
Its single Ubuntu job, **Quick checks**, checks formatting, Clippy for all targets
without optional features, and the normal tests without optional features.
That test run includes mocked integration and security unit tests once; ignored
browser tests and feature-gated PDF tests stay in full QA. Test threads remain
serialized until parallel safety is demonstrated. The cache is shared between
Clippy and tests and keyed by OS, pinned toolchain and lockfile.

Prose and site changes still produce a successful check. Only `README.md`,
`CHANGELOG.md`, `LICENSE`, `docs/index.md`, `docs/{getting-started,guides,concepts,reference,img}/`
and `site/` skip Rust compilation. Root-level documentation contracts, schemas,
calibration data, Rust sources, fixtures and workflow changes run Rust checks.
The scope tests cover PR merge bases, multi-commit pushes and renames; unavailable
base history runs the checks. Superseded quick runs are cancelled.

`ci.yml` is the full QA workflow: production-feature tests (PDF and AI transparency) and all-feature rustdoc, PDF smoke,
browser integration, three WCAG corpus shards, dependency audit, unused-dependency
checks, and mocked tests. Coverage runs on non-tag refs; detector mutations and
docs.rs sandbox parity retain their tag-only conditions. The latter remains
advisory (`continue-on-error`), as before. All jobs have time limits and the QA
token has read-only contents access. Full runs use distinct concurrency groups
per run and attempt, so dispatches and release calls cannot cancel or replace
one another.

#### Path-filtered browser and PDF suites on pull requests (#942)

One thin workflow calls `ci.yml` through `workflow_call` with its `suites` input,
so the steps exist once and Quick checks does not get slower:

| Workflow | Runs on PRs touching | Jobs of `ci.yml` |
|---|---|---|
| `pr-browser-qa.yml` | `src/wcag/**`, `src/a11y_journey/**`, `src/output/pdf/**`, `tests/fixtures/rule_contracts/**` | Browser Integration Tests (E2E), Detection corpus (WCAG) 1/3 to 3/3, PDF Smoke Tests |

It also runs when the workflow itself, `ci.yml` or `.github/actions/**`
changes. A PR that matches no path does not start it, so docs-only PRs
cost nothing extra and the always-present check stays **Quick checks**. These
checks are separate from it: if protection is ever enabled, do not make them
required, since a skipped workflow never reports. `suites` defaults to `all`,
which is what `release.yml` and a manual dispatch get, so tags and dispatches run
the same jobs as before. The rule-contract gate (`tests/rule_contract_gate.rs`)
needs no Chrome and already runs inside the `cargo test` of Quick checks; a change
under `rule_contracts/` additionally runs the corpus its contracts refer to.
Detector mutations, docs.rs parity, native macOS/Windows smoke, audit and
coverage stay tag/dispatch/full only. A new push to the PR cancels the running
filtered jobs.

To validate a release candidate without publishing, select **CI → Run workflow**
and the candidate ref, or, once `workflow_dispatch` is present on `main`:

```bash
gh workflow run ci.yml --ref <candidate-branch-or-tag>
```

The selected ref must contain this workflow. A branch dispatch skips tag-only
jobs; dispatching an existing tag exercises those jobs too, without publishing.
Only a pushed `v*` tag starts `release.yml`. It calls `ci.yml` from the same
commit and requires both version/milestone validation and full QA before any
binary packaging. `scripts/v2-readiness.json` retains exactly U01–U26; additional
open milestone issues and #792 also block 2.x, including prereleases. API errors
fail closed. No prerelease publishing policy is added by this CI split.

### Required checks and runtime evidence

At the 2026-11-02 migration, `main` has no branch protection or rulesets, so there
are no required checks to migrate. If an administrator enables protection, require
**Quick checks**, rather than the old full-QA job names on PRs. Full QA is mandatory
through the release dependency graph. Existing open PRs use their current workflow
until updated with this change.

Ordinary Rust changes run one Ubuntu job instead of 12 active full-QA jobs
(including the three WCAG shards); prose changes run only checkout and the cheap
scope/readiness regressions. Runner-minute savings require comparable completed
runs and depend on cache warmth; job-count reduction is not a measured speedup.
The 2026-11-02 baseline run
[41402930877](https://github.com/casoon/auditmysite/actions/runs/41402930877)
used 8,355 aggregate job-seconds (about 139.3 runner-minutes, or 145 after
rounding each job up to a minute). This is an observed baseline, not a forecast
for the new job.

Full browser QA currently runs on Linux x86_64. Native macOS ARM64, Linux ARM64
and Windows x86_64/Edge runtime gates remain required 2.0 work in #856; the Linux
ARM64 release cross-build does not satisfy them. Browser selection, bootstrap,
UA and isolation work remains in #854, #855 and #787. This split changes CI
execution, not browser behavior.

## Release checks

Run the local release gate with:

```bash
./scripts/release-check.sh
```

It validates:
- `cargo test`
- ignored browser integration tests
- builds with and without PDF
- current `--help` output
- JSON contract tests
- installer/release artifact consistency
- stale docs references

## Library / Development

For library development or local work from the repository:

```bash
cargo build
cargo test
```

If you want the current local repository state as an installed binary while developing:

```bash
cargo install --path . --force
```

Contributions are welcome. At minimum before opening a PR:

```bash
cargo test
./scripts/release-check.sh
```
