#!/bin/sh

set -eu

REPO_ROOT=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
cd "$REPO_ROOT"

echo "pre-commit: cargo fmt -- --check"
cargo fmt -- --check

echo "pre-commit: cargo clippy --all-targets --all-features -- -D warnings"
cargo clippy --all-targets --all-features -- -D warnings

echo "pre-commit: cargo test"
cargo test
