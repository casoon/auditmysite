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

## Maintainer binary `auditmysite-dev`

Research and QA tooling is kept out of the `auditmysite` a user installs (#912). It lives in a
second binary, `auditmysite-dev`, behind the `dev-tools` cargo feature. `required-features` keeps
it out of `cargo install auditmysite`, which builds with the default features only.

```bash
cargo run --features dev-tools --bin auditmysite-dev -- --help
# or, once built
cargo build --features dev-tools          # → target/debug/auditmysite-dev
```

`auditmysite-dev` accepts everything `auditmysite` does and adds:

| Command / flag | Purpose |
| --- | --- |
| `report-lint <report.json> [--fail-on …] [--typst-source <file.typ>]` | Deterministic consistency checks on a JSON report, see [Reports](../reports/#report-lint) |
| `accname-diff <url>…` | Own accessible-name computation against Chrome's, see [accname differential](../../accname-differential/) |
| `plan [url]` | Mode, scope, modules and output paths of an audit, without running it |
| `replay <snapshot\|dir\|url>` | Evaluate a stored snapshot again, without browser or network |
| `--debug-typ` | With `--format pdf`, also write the Typst source as a `.typ` sidecar |
| `--export-snapshot <path>` | Export AXTree snapshots and journey traces as YAML |

`report-lint` in CI: the command logic is a library function (`auditmysite::cli::report_lint`),
and `tests/report_lint_tests.rs` runs it on every PR inside the plain `cargo test` jobs, so no job
has to build the dev binary. To run it from a workflow or a script, build with the feature:
`cargo run --locked --features dev-tools --bin auditmysite-dev -- report-lint report.json`.

### Debugging report content (`--debug-typ`)

PDF reports are rendered through the `renderreport`/Typst engine. To review report
**completeness and wording** without opening the binary PDF, use `--debug-typ` together with
`--format pdf`. It writes the intermediate Typst source as a `.typ` sidecar next to the PDF, for
both single and batch reports:

```bash
# Single report → reports/example-audit.pdf + reports/example-audit.typ
cargo run --features dev-tools --bin auditmysite-dev -- https://example.com --full \
  --format pdf --output reports/example-audit.pdf --debug-typ

# Batch report → reports/example-batch.pdf + reports/example-batch.typ
cargo run --features dev-tools --bin auditmysite-dev -- \
  --sitemap https://example.com/sitemap.xml --full \
  --format pdf --output reports/example-batch.pdf --debug-typ
```

The `.typ` file is plain text and diff-friendly — useful for checking which audits
land in the report and reviewing the exact wording of every section.

## Repository CI

`quick-ci.yml` runs on pull requests targeting `main` and pushes to `main`.
Its single Ubuntu job, **Quick checks**, checks formatting, rustdoc with all features
(`-D warnings`), Clippy for all targets without optional features, the normal tests without optional features, and lint
plus unit tests for the embedded JavaScript (see below).
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

`ci.yml` is the full QA workflow: production-feature tests (PDF) and all-feature rustdoc, PDF smoke,
browser integration, three WCAG corpus shards, dependency audit, unused-dependency
checks, and mocked tests. Coverage runs on non-tag refs. Detector mutations (four
shards) and docs.rs sandbox parity run on `v*` tags and on `workflow_dispatch`
(any ref, #894), so a release candidate can be exercised before it is tagged. The
docs.rs job remains advisory (`continue-on-error`), as before; the mutation check
is a blocking gate. All jobs have time limits and the QA
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
Detector mutations and docs.rs parity stay tag/dispatch only; native
macOS/Windows smoke, audit and coverage stay in full runs only. A new push to the PR cancels the running
filtered jobs.

To validate a release candidate without publishing, select **CI → Run workflow**
and the candidate ref, or:

```bash
gh workflow run ci.yml --ref <candidate-branch-or-tag>
```

The selected ref must contain this workflow. A dispatch runs every job, including
docs.rs sandbox parity and the four mutation shards (PR #956); a branch dispatch
needs no tag. Only a pushed `v*` tag starts `release.yml`. It calls `ci.yml` from the same
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

## Embedded JavaScript (`src/**/js/*.js`)

Scripts that run in the audited page live next to the Rust that sends them, in a
`js/` directory beside the module file, and are loaded with `include_str!`:

```rust
const DETECT_JS: &str = include_str!("js/detect.js"); // src/tech_stack/js/detect.js
```

- **Where:** `src/<module>/js/<name>.js`, relative to the `.rs` file that includes
  it (`src/wcag/rules/focus_walk.rs` uses `src/wcag/rules/js/`). Name the file after
  what the script does, not after the Rust constant.
- **Shape:** the file is exactly what Chrome receives: an expression or IIFE for
  `Runtime.evaluate`, a bare `function (...) { ... }` for `callFunctionOn`, or a
  body with top-level `return` that Rust wraps in `(function () { ... })()` after
  the shared helpers (`src/accessibility/js/helpers_*.js`).
- **Values from Rust:** never `format!` (the file would need `{{ }}`). Write a
  placeholder `__NAME__` where the value goes and fill it with
  `include_str!(..).replace("__NAME__", &value)`, ideally in a small
  `*_script(..)` function with a unit test that the placeholder is gone and the
  value is where it belongs. Existing markers `/*CSS_SELECTOR*/` and
  `/*MAX_FINDINGS*/` work the same way. Values that come from the audited site
  (selectors, URLs) go in as JSON (`serde_json::to_string`).
- **Left inline** on purpose: one-line expressions (`document.readyState`,
  `window.scrollTo(0,0)`, ...) and scripts the Rust code assembles from
  parts at run time.

### Lint and unit tests

```bash
scripts/check-js.sh
```

It installs the pinned tools from `scripts/js/package-lock.json` on first use
(ESLint 10, `@eslint/js`, `globals`), lints every `src/**/js/*.js` with
`scripts/js/eslint.config.mjs`, then runs `node --test` on `scripts/js/test/`:

- `placeholders.test.mjs` checks that the `__NAME__` placeholders in the scripts and
  the ones the Rust code replaces are the same set,
- `scripts.test.mjs` runs the scripts that need no layout engine against stubs
  (`performance` entries, the exclusion scripts, the shared cap helpers).

Without Node (20.19+, 22.13+ or 24+) the script prints `check-js: SKIPPED` and exits 0 locally;
with `CI` set it fails instead. Quick checks runs it as a step of its existing job
(Node 22 via the already pinned `actions/setup-node`), about two seconds including
the install. It is not part of the pre-commit hook.

ESLint is configured with `recommended`; findings in the first extraction that would need
a behaviour change are switched off per file in the config, with the reason.

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

### Order before a release tag

1. `./scripts/release-check.sh` (above).
2. The reference runs: `cargo test --test reference_sites_test -- --ignored` and
   `python3 scripts/privacy-reference.py live` (details in `CLAUDE.md`, release step 0).
3. The [human release smoke](#human-release-smoke) below.
4. **Dispatch the full QA on the release candidate** and wait for it to finish green:

   ```bash
   gh workflow run ci.yml --ref <release-candidate-branch>
   gh run list --workflow ci.yml --branch <release-candidate-branch> --limit 1   # run id
   gh run watch <run-id>
   ```

   The dispatch runs the tag-only jobs too (docs.rs sandbox parity, four detector
   mutation shards). The mutation shards set the wall time: 75 to 90 minutes, against a job
   limit of 90 (#894). A mutation or docs.rs failure found here costs a branch push; found
   on the tag it costs a re-tag (2.0.0 and 2.0.1 both needed one).
5. Push the tag `vX.Y.Z`; `release.yml` runs the same full QA on the tag commit, then builds.
6. Wait for the job `docs.rs Build (sandbox parity)` on the tag run, only then `cargo publish`.

### Human release smoke

The release gate measures contracts, schemas and detector strength, but nobody had read a
real report before tagging: 2.0.0 shipped with every clean run `partial`, a 5-URL batch with
0 of 5 passed, a grade label contradicting its score, and English in the German PDF (#895,
#896, #898). Read one real report
before every tag (#924). Tick the four boxes in the release PR
(`.github/PULL_REQUEST_TEMPLATE.md`, section "For a release PR").

Build the candidate and write everything to `reports/` (gitignored):

```bash
cargo build --release   # ./target/release/auditmysite (or under $CARGO_TARGET_DIR, if set)

# Two real sites: one German, one English. Both PDFs per site, German and English.
./target/release/auditmysite https://<german-site> --full -f pdf --lang de -o reports/smoke-de-site-de.pdf
./target/release/auditmysite https://<german-site> --full -f pdf --lang en -o reports/smoke-de-site-en.pdf
./target/release/auditmysite https://<english-site> --full -f pdf --lang de -o reports/smoke-en-site-de.pdf
./target/release/auditmysite https://<english-site> --full -f pdf --lang en -o reports/smoke-en-site-en.pdf

# One batch of five URLs (one per line in the file)
./target/release/auditmysite --url-file reports/smoke-urls.txt --full -f pdf --lang en -o reports/smoke-batch.pdf

# The terminal table, for wording and thresholds
./target/release/auditmysite https://<german-site> --lang de
./target/release/auditmysite https://<english-site>
```

`--lang` defaults to `en`; `-o x.pdf` alone would also select the PDF. To read wording as
text instead of opening the binary PDF, build the maintainer binary and add `--debug-typ`
(see "Debugging report content" above).

Check:

1. **Two sites, four PDFs.** Read each PDF. Look for wording in the wrong language (the
   English PDF of a German site and the German PDF of an English site are the cases that
   broke), and compare every score with the card or table that shows it (a grade or label
   next to a score that contradicts it, as in #896).
2. **One 5-URL batch.** All five URLs are audited and none is reported as failed or
   `partial` unless that page really fails to load or measure.
3. **Terminal table.** Wording is in the requested language and the labels and thresholds
   match the PDF of the same run.
4. Anything wrong is a release blocker or a filed issue before the tag, not a note for
   after it.

No script enforces this; the checkboxes in the release PR are the record.

### Decisions on the release pipeline

- **The detector mutation check stays a blocking gate of the tag run.** There is no weekly
  run on `main` (#783 asked for a binding gate; a weekly run would let a release ship without
  it). The cost is paid before the tag, by the pre-tag dispatch above. The decision is
  written in the comment above the `mutation-check` job in `ci.yml`.
- **The E2E job (`Browser Integration Tests (E2E)`) is not split** like the detection corpus.
  The last 11 successful runs took 22 to 33 minutes against its 45-minute limit, and a split
  would not shorten a tag or dispatch run, whose wall time is set by the mutation shards.
  Revisit if it passes about 38 minutes (#894).

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
