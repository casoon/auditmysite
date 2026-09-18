#!/bin/sh

set -eu

REPO_ROOT=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
cd "$REPO_ROOT"

mkdir -p target/release-check

echo "release-check: version/tag consistency"
"$REPO_ROOT/scripts/check-version-match.sh" "$@"

echo "release-check: cargo test"
cargo test

echo "release-check: release contract tests"
cargo test --test release_contract_tests

echo "release-check: ignored integration tests"
cargo test --test integration_test -- --ignored

# WCAG/schema/vulnerable-libs/easy-language detection-corpus tests
# (#553-#558): real Chrome, diffed against ground-truth fixtures. Same
# `#[ignore]`-gated test binaries `.github/workflows/ci.yml`'s
# `browser-smoke` job runs — kept in sync so a local release-check can't
# report green without them (plan/8-release-check-missing-detection-
# corpus-tests.md).
# Every binary runs even if an earlier one fails, so one regression can't
# hide another (#582); the check still fails at the end.
echo "release-check: detection-corpus accuracy tests"
corpus_failed=""
for t in detection_corpus_test schema_rules_detection_corpus_test vulnerable_libs_detection_corpus_test easy_language_detection_test; do
  cargo test --test "$t" -- --ignored || corpus_failed="$corpus_failed $t"
done
if [ -n "$corpus_failed" ]; then
  echo "release-check: detection-corpus tests failed:$corpus_failed"
  exit 1
fi

echo "release-check: build without pdf"
cargo check --no-default-features

echo "release-check: build with pdf"
cargo check --features pdf

echo "release-check: capture --help"
cargo run -- --help > target/release-check/help.txt

echo "release-check: documentation consistency"
# `html` not followed by `-conform`: the html-conform crate name is not a
# stale `--format html` reference.
if rg -n -P --glob 'README.md' --glob 'docs/*.md' --glob 'reports/*.md' '\bhtml\b(?!-conform)|\bmarkdown\b|--urls\b' >/tmp/auditmysite-release-check-stale.txt; then
  echo "Found stale CLI/docs references:"
  cat /tmp/auditmysite-release-check-stale.txt
  exit 1
fi

if ! rg -q -- '--browser-path' README.md; then
  echo "README must document --browser-path"
  exit 1
fi

if ! rg -q -- '--url-file' README.md; then
  echo "README must document --url-file"
  exit 1
fi

# plan/26-documentation-consistency.md: the crate/CLI-level product
# description used to flatly say "WCAG 2.1 Accessibility Checker" (no AA/2.2
# qualifier) in several places while README.md already individually tags
# specific WCAG 2.2 rules (2.4.11/2.4.12, 3.3.7, 2.5.8) — catches any of
# those spots regressing back to the unqualified phrase.
if rg -n -P --glob 'src/**/*.rs' --glob 'Cargo.toml' --glob 'CLAUDE.md' \
     'WCAG 2\.1 (Accessibility Checker|accessibility checker)(?! AA)' \
     >/tmp/auditmysite-release-check-wcag-baseline.txt; then
  echo "Found an unqualified \"WCAG 2.1 Accessibility Checker\" claim (missing the AA/2.2 qualifier):"
  cat /tmp/auditmysite-release-check-wcag-baseline.txt
  exit 1
fi

# The HTML5-conformance module contributes to the overall score
# (`contributes_to_overall: true`, `weight_pct: 5` in
# `audit::normalized::build_module_scores`) since the per-distinct-defect
# scoring fix (2026-09-16) — README must not still call it score-neutral.
if rg -n 'HTML5 conformance' README.md | rg -q 'score-neutral'; then
  echo "README still calls HTML5 conformance score-neutral, but it contributes to the overall score"
  exit 1
fi

# src/lib.rs's module overview must list the real output formatters, not a
# never-shipped HTML export.
if rg -n 'Report formatters.*HTML\b' src/lib.rs >/tmp/auditmysite-release-check-html-format.txt; then
  echo "src/lib.rs still lists HTML as an output formatter — no HTML export exists"
  cat /tmp/auditmysite-release-check-html-format.txt
  exit 1
fi

echo "release-check: complete"
