#!/bin/sh

set -eu

REPO_ROOT=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
cd "$REPO_ROOT"

# Formatting check for what is actually being committed.
#
# `cargo fmt --check` covers the whole crate, so unformatted work in progress
# anywhere blocked every commit — including ones that touch no Rust at all,
# such as a dependency bump.
#
# The *staged* content is checked rather than the file on disk: a file can be
# staged in a formatted state and edited further, and only what is going into
# the commit is the hook's business.
files=$(git diff --cached --name-only --diff-filter=ACMR -- '*.rs')
if [ -n "$files" ]; then
  echo "pre-commit: rustfmt --check (staged files)"

  edition=$(sed -n 's/^edition *= *"\([0-9]*\)".*/\1/p' Cargo.toml | head -1)
  edition=${edition:-2021}

  # rustfmt exits 0 when reading stdin even where it has a diff to report, so
  # what it printed is what decides, not its status.
  unformatted=$(
    printf '%s\n' "$files" | while IFS= read -r file; do
      [ -n "$file" ] || continue
      if [ -n "$(git show ":$file" 2>/dev/null | rustfmt --check --edition "$edition" 2>&1)" ]; then
        printf '  %s\n' "$file"
      fi
    done
  )

  if [ -n "$unformatted" ]; then
    echo "cargo fmt check failed for staged files:"
    printf '%s\n' "$unformatted"
    echo "run: cargo fmt  (then stage them again)"
    exit 1
  fi
fi

echo "pre-commit: cargo clippy --lib --bins --all-features -- -D warnings"
cargo clippy --lib --bins --all-features -- -D warnings
