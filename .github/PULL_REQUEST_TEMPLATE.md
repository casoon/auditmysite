## What and why

<!-- One paragraph. Link the issue: Fixes #123 or Refs #123. -->

## Kind of change

- [ ] Bug fix
- [ ] New or changed rule
- [ ] Feature or flag
- [ ] Docs or tooling only

## Checks

- [ ] `cargo fmt` and `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] `cargo check --all-features`
- [ ] `cargo test` (includes `rule_contract_gate`)
- [ ] User-facing change: entry under `## [Unreleased]` in `CHANGELOG.md`

## For a new or changed rule

- [ ] The rule needs a running browser; browser-free rules belong in barrierlab
- [ ] Contract in `tests/fixtures/rule_contracts/` written from the normative source, not from the run
- [ ] Corpus cases for violation, pass, not applicable and the error case
- [ ] `error_page_scripts_sabotaged.expected.json` lists the rule id
- [ ] `CORPUS_CASES=<cases> cargo test --test detection_corpus_test -- --ignored` passes
- [ ] `scripts/mutation-check.py --rule <id>` kills every mutation

## For a release PR

Procedure and commands: "Human release smoke" in `docs/guides/development.md`.

- [ ] Two real sites audited, one German and one English; the PDF read in both German and English (wording in the wrong language, scores against cards)
- [ ] One 5-URL batch: all five audited, none unexpectedly partial or failed
- [ ] Terminal table output read for wording and thresholds
- [ ] `ci.yml` dispatched on the release candidate and green (docs.rs parity and mutation shards included) before the tag is pushed

## Notes for the reviewer

<!-- Trade-offs, things you were unsure about, how you tested. -->
