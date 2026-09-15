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

echo "release-check: complete"
