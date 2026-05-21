#!/bin/sh

set -eu

REPO_ROOT=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
cd "$REPO_ROOT"

echo "pre-commit: cargo fmt -- --check"
cargo fmt -- --check

echo "pre-commit: cargo clippy --lib --bins --all-features -- -D warnings"
cargo clippy --lib --bins --all-features -- -D warnings
