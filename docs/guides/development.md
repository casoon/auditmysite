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
