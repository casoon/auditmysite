#!/bin/sh

set -eu

REPO_ROOT=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
cd "$REPO_ROOT"

mkdir -p target/release-check

echo "release-check: cargo test"
cargo test

echo "release-check: release contract tests"
cargo test --test release_contract_tests

echo "release-check: ignored integration tests"
cargo test --test integration_test -- --ignored

echo "release-check: build without pdf"
cargo check --no-default-features

echo "release-check: build with pdf"
cargo check --features pdf

echo "release-check: capture --help"
cargo run -- --help > target/release-check/help.txt

echo "release-check: documentation consistency"
if rg -n --glob 'README.md' --glob 'docs/*.md' --glob 'reports/*.md' '\bhtml\b|\bmarkdown\b|--urls\b' >/tmp/auditmysite-release-check-stale.txt; then
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
