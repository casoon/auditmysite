# AuditMySite - Project Instructions

## Project Overview
Resource-efficient WCAG 2.2 AA Accessibility Checker written in Rust. Audits web pages using Chrome DevTools Protocol (CDP) and the browser's native Accessibility Tree. Supports single URL, sitemap batch, and URL file batch modes.

## Architecture
- **Language:** Rust (async with tokio)
- **Browser:** Chrome/Chromium via `chromiumoxide` (CDP)
- **CLI:** `clap` with derive macros
- **PDF:** `renderreport` (Typst-based, optional `pdf` feature) — source repo at `../renderreport`, consumed as a pinned crates.io version
- **Config:** Optional `auditmysite.toml` files

## Boundary to barrierlab (MANDATORY — agreed 2026-09-26)
Shared libraries live in the monorepo `../barrierlab` and come in as crates.io versions.
Before every change, clarify which side the code belongs to:
- **Source is barrierlab — never patch, copy or work around it here:** the `SHARED_RULES`
  (`src/wcag/shared.rs` → `a11y-rules`), `accname`, `a11y-dom`, `a11y-report`, tree/snapshot/diff/
  linearization/announcement structure (`a11y-perception`), bot classification in robots.txt
  (`web-checks`), `html-conform`. Workflow: change in barrierlab → release → bump the version here.
- **Move candidates — migrate first, then develop further:** structured data
  (`src/seo/schema_rules.rs`), meta lengths, OpenGraph presence, render-blocking. Until the move,
  only bug fixes here; before the move, document the duplication with astro-post-audit case by case.
- **WCAG rules move to barrierlab (decision 2026-09-30, takes precedence over feature work here):**
  all browser-free rules in `src/wcag/rules` go to `a11y-rules`; auditmysite then consumes them via
  `SHARED_RULES`. Order and packages are in the umbrella issue casoon/barrierlab#13 — first
  B0 (#690, delete duplicates), then per package barrierlab port → release → adoption here
  (#691–#697, each "blocked by" the barrierlab side). Until the move, only bug fixes on these
  rules, and every fix goes into the barrierlab issue of its package as a test case. Rules
  involving style/geometry follow after the rendering layer (barrierlab#21 → #698).
- **Stays here:** WCAG rules that need a running browser (interaction, frames, paste, focus
  measurement), the manual check hints, journeys, CDP capture, scoring/taxonomy, PDF, CLI,
  report texts (incl. `screen_reader/announcer.rs`).
- **Development loop:** for simultaneous work on both sides, use `[patch.crates-io]` with
  `path = "../barrierlab/crates/<crate>"` in `.cargo/config.toml` (gitignored). Never commit
  while the patch is active — `Cargo.lock` included; commit only against the published version.
- The journey rework is finished; the open blind comparison against real screen readers is tracked
  in the barrierlab repository.

## Key CLI Modes
- Single: `auditmysite <URL>` (default: table on stdout, English, no file written, no prompt; `-o report.json` / `-f pdf -o report.pdf` write a report)
- Sitemap: `auditmysite --sitemap <SITEMAP_URL>` (batch from XML sitemap)
- URL file: `auditmysite --url-file <FILE>` (batch from text file)
- Full audit: `--full` (enables performance, seo, security, mobile)
- Browser: `auditmysite browser {detect|install|remove|path}`, `auditmysite doctor`
- Maintainer tooling lives in the separate binary `auditmysite-dev` (feature `dev-tools`, not installed by `cargo install`, #912): `cargo run --features dev-tools --bin auditmysite-dev -- <command>`. It accepts everything `auditmysite` does plus `plan`, `replay`, `report-lint`, `accname-diff`, `--debug-typ` and `--export-snapshot`; none of these exist in `auditmysite` itself.
- Report lint (dev binary): `auditmysite-dev report-lint <JSON_FILE> [--fail-on low|medium|high|critical]` (#507, deterministic checks, no network/Chrome; default fail-on: high; exit code 3 if breached)
- accname differential (dev binary): `auditmysite-dev accname-diff <URL>... [--url-file FILE] [--output FILE] [--max-samples N]` (more than one URL → corpus aggregate; `text_transform` as its own class via computed style; own `accname::name_rendered` with `display`/`visibility` from DOMSnapshot against Chrome's native AX tree; measures, does not judge — exit code always 0; see `docs/accname-differential.md`)
- Privacy: `--privacy-check` (opt-in, not part of `--full`): first-visit findings in `detail.modules.privacy` (#871; technical findings with confidence class, no verdict; see `docs/privacy-findings.md`)
- Experimental indicators: `--experimental <name>[,<name>]` (`ux`, `journey`, `ai-visibility`, `content-visibility`, `source-quality`, `commerce`; #916): unvalidated heuristics, off by default and in no `--full` run; JSON `pages[].experimental`, terminal table, never the PDF. The audit assembly moves their results into `AuditReport.experimental`
- Modules: `--modules a11y,performance,seo,security,mobile,html-conform,stack` (default: all; `--full` is the explicit spelling of the default)
- Output formats: `--format {json|table|pdf}`

## Report Intent
- **Single URL audit** is intentionally detailed and page-specific.
- Use it when one concrete page should be reviewed deeply, with findings, explanations, module detail, and implementation guidance for that page.
- **Sitemap / batch audit** is intentionally aggregated and domain-wide.
- Use it when multiple URLs should be compared, averaged, and prioritized across the site.
- Batch reports must focus on cross-page information such as:
  - average scores
  - strongest / weakest URLs
  - recurring issues
  - URL ranking and compact URL matrices
  - distribution patterns across the scanned set
- Batch reports must **not** devolve into a stack of single-page reports. Per-URL detail should stay compressed unless a dedicated technical appendix is explicitly intended.

## Reports Directory
- **All manually generated test reports MUST be saved to `reports/`**
- Use `--output reports/<filename>` when running audits
- The `reports/` directory is gitignored (except `reports/README.md`)
- Naming convention: `<domain>-audit.<format>` (e.g., `casoon-audit.html`)
- Batch reports: `<domain>-batch-audit.<format>`

## Build & Test
```bash
cargo build --release          # Build optimized binary
cargo check                    # Fast compile check
cargo check --all-features     # MANDATORY before every push — what CI checks
cargo test                     # Run all tests
cargo test --lib               # Unit tests only
```

**Run `cargo check --all-features` before every push.** Quick CI checks without optional
features; the full QA checks all features and Clippy before releases or manually.
The hooks are versioned under `.githooks/` (`git config core.hooksPath .githooks`);
`pre-push` blocks, among other things, an active barrierlab `[patch]`.

Common pitfall: new fields in `NormalizedReport` break struct initializers in
`src/audit/normalized.rs` and `src/audit/summary.rs`. Always check both.
Likewise, every new field in `PipelineConfig` (public, without `Default`) breaks Studio and the
doc example in `src/lib.rs` — update both.

## Workflows (`.github/workflows/`)
- The repo is public: Quick CI runs on PRs and pushes to `main`; full QA runs via `workflow_call`
  before release builds on the tag SHA, or manually via `workflow_dispatch`, without publishing.
  Load the skill `ci-budget` before making changes; procedure and status check in `docs/guides/development.md`.
- Never install tools unpinned (no `cargo install … --git` without `--rev`, no tool without a
  version); pin actions by commit SHA with the tag as a comment. `cross` in `release.yml` is pinned to a commit.
- Every new job gets `timeout-minutes`.
- Decisions (#894, #783): the detector mutation check stays a blocking gate of the tag run, there
  is no weekly run on `main`; the E2E job is not split and is revisited if it passes ~38 min (it
  runs 22–33 min against its 45 min limit). Details in `docs/guides/development.md`.

## Release & `cargo publish` (MANDATORY — #588)
A version published on crates.io can **never** be replaced, only yanked. And docs.rs builds each
version exactly once: what arrives broken there stays broken permanently. Therefore, in this order:

0. **Before the tag: reference run.** `cargo test --test reference_sites_test -- --ignored` audits
   the manually rated live sites from `tests/fixtures/reference_sites.json` and reports every one
   that leaves its band. It does not block automatically: live sites change, so first re-rate the
   site, then suspect the tool — but clarify every deviation before the tag.
   Plus the privacy reference run (#873): `python3 scripts/privacy-reference.py live` measures
   precision/recall of the findings against the hand-labeled pages in
   `tests/fixtures/privacy/reference_pages.json` (never adjust the labels to the tool; method in
   `docs/privacy-findings.md`). **Bands move along:** a PR that deliberately shifts the value of a
   reference site (false alarm fixed, rule moved, weighting changed) updates its band in
   `reference_sites.json` in the same PR, with the reason in `notes`. Otherwise only the run before
   the tag reports the long-fixed case as a deviation (1.8.0: bundesregierung.de and berlin.de,
   fixes of 04/05 Oct).
0b. **Human release smoke (#924).** Read one real report before the tag: one German and one
   English real site (`--full -f pdf`, both PDFs read in DE and EN: wording in the wrong language,
   scores against cards), one 5-URL batch (all five audited, none unexpectedly partial) and the
   terminal table for wording and thresholds. Procedure, commands and the release-PR checkboxes:
   "Human release smoke" in `docs/guides/development.md`.
1. **Dispatch the full QA on the release candidate and wait for green before pushing the tag:**
   `gh workflow run ci.yml --ref <candidate>` (since PR #956 a branch dispatch also runs the
   docs.rs sandbox build and the four detector-mutation shards, #894). The mutation shards take
   about 75–90 min and set the wall time; a failure found here costs a branch push, on the tag a
   re-tag.
2. **Set and push the tag `vX.Y.Z`.** This starts `release.yml`: version check, full CI,
   binaries, GitHub release.
3. **Wait for the job `docs.rs Build (sandbox parity)`.** It runs on tags and on
   `workflow_dispatch` (step 1 is the first run) and builds the
   docs in the real docs.rs sandbox image — its nightly, its memory and time limits, and on what
   `cargo package` would publish. Nothing else in CI answers this: the docs step in
   `check-all-features` builds on the runner, with stable and without a memory cap.
4. **Only then `cargo publish`.**

The job is `continue-on-error` as long as the action is in beta — so it does not block, it
**informs**. If it is red, do not publish; clarify the cause first.

For scale: 1.5.1 needed 4.75 GB of 6.4 GB available RAM there. Since `all-features = true`,
docs.rs also documents `ai-transparency`, which raises the dependency graph from 454 to 565
crates. The margin to the ceiling is real, but not large.

Locally before the tag, if `cargo-docs-rs` is installed: `cargo docs-rs` builds with the same
`[package.metadata.docs.rs]` configuration, but without sandbox and without limits.

## Testing Against Live Sites
```bash
# 1. Single page audit (all modules) — deep analysis of one specific page
./target/release/auditmysite https://example.com --full --format pdf --output reports/example-audit.pdf

# 2. Sample batch audit — the first 20 pages of the sitemap (first_n)
# The statement holds only for this successfully audited selection, not for the website:
# first_n is chosen by order, not by page type/template/language.
# Usable for a first assessment of template-wide problems; spread and
# worst cases are in summary.score_spread, the selection in summary.selection_profile.
./target/release/auditmysite --sitemap https://example.com/sitemap.xml --full --format pdf --output reports/example-sample-audit.pdf --max-pages 20

# 3. Full sitemap batch audit — all pages, domain-wide
./target/release/auditmysite --sitemap https://example.com/sitemap.xml --full --format pdf --output reports/example-batch-audit.pdf

# Quick CLI check
./target/release/auditmysite https://example.com --format table
```

## Report Format Rules
- **Always use PDF format** (`--format pdf`) when generating test reports
- Never use HTML export for reports
- PDF reports use the `renderreport` Typst engine with full module detail sections

## Localization architecture (#406 — MANDATORY for new report-visible texts)
**JSON is canonically English, only the PDF is multilingual.** Rule:
- The **analysis/derive layer bakes canonical English** into the stored structs
  (`AuditReport`/`NormalizedReport` → JSON). Never bake the run language into analysis results
  — the JSON must stay English independent of language.
- The **PDF presentation layer derives localized texts at runtime** (`i18n.locale()`), via pure
  functions/`build_*_presentation`/`render_*`.
- **Pattern for message-baked structs:** the struct carries a canonical `kind` enum (+ raw values
  in a `#[serde(skip)]`/`skip_serializing_if` field); a pure `pub fn *_text(kind, .., en)` is the
  ONLY text source — the analysis calls it with `en=true`, the PDF builder with the run language.
  Examples: `source_quality`, `content_visibility`, `ai_visibility`, `journey`, `ux`,
  `seo/page_health` (`collect_issues`), `screen_reader`.
- **Detection language ≠ message language:** language-dependent DETECTION (e.g. stopword matching
  for generic link texts) follows the **page language**, not the output language. Pass both
  separately (see `screen_reader::analyze_reading_sequence(detect_locale, message_en)`).
- Guard test per localized module: EN output contains no German umlauts/ß.

## Architecture Documentation
Whenever a new module is added, renamed, or removed, update `docs/ARCHITECTURE.md` in the same commit. Also update the `Current State` version when bumping the version.

## Code Conventions
- Use `thiserror` for error types, `anyhow` for propagation
- WCAG rules go in `src/wcag/rules/` as individual files, register in `mod.rs`
- **Every active rule ID needs a contract** in `tests/fixtures/rule_contracts/` (format: README there) with positive, negative, N/A and error cases in the corpus, a variant matrix and targeted mutations; otherwise `cargo test --test rule_contract_gate` is red (#783). Write expectations from the standard, never from the run; `scripts/mutation-check.py --rule <id>` checks test strength.
- Output formatters go in `src/output/`, support both single and batch reports
- Keep async operations in audit pipeline and browser modules
- Use `tracing` for structured logging (INFO, WARN, ERROR)
- **Dependency direction (target state, applies to new code from now on):** analysis modules
  (`seo`, `security`, `performance`, `ux`, …) and `wcag` import neither `cli` nor `output`;
  new shared types (level, status, report models) do not belong in `cli/args.rs` or in the
  orchestration `audit/pipeline`. Do not add more backward edges — the rework proceeds
  step by step, not on the side in feature PRs.

## Current State (v2.0.2)
`CHANGELOG.md` follows Keep a Changelog (skill `keep-a-changelog`): every user-facing change gets
a short entry under `## [Unreleased]` in the same commit, `**Area:**` first, English; at a release
`[Unreleased]` becomes `## [x.y.z] - YYYY-MM-DD`. The detailed development log up to 2026-10-24
(finding, fix, verification) is `docs/reference/development-history.md`. Update the version
number above whenever `Cargo.toml`'s version bumps; no change history in this file.
