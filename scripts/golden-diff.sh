#!/usr/bin/env bash
# Golden-render diff (plan 66, WP0): render the frozen golden input set with the
# current checkout and optionally diff it against an earlier render.
#
# Usage:
#   scripts/golden-diff.sh <out-dir> [<baseline-dir>]
#
#   <out-dir>       rendered into (emptied first)
#   <baseline-dir>  earlier render to compare against; exit 1 on any difference
#
# Input: AUDITMYSITE_GOLDEN_DIR, default <repo>/reports/golden (gitignored; one
# directory per site with report.json, snapshot.json, audit.json copied from
# ~/.auditmysite/cache/<domain>/<hash>/v<VERSION>/). From a git worktree, point
# it at the main checkout's reports/golden.
#
# Typical refactoring check:
#   git switch main        && scripts/golden-diff.sh /tmp/golden-main
#   git switch my-branch   && scripts/golden-diff.sh /tmp/golden-branch /tmp/golden-main
# An empty diff means the branch renders byte-identical JSON and Typst output.

set -euo pipefail

if [ $# -lt 1 ] || [ $# -gt 2 ]; then
    sed -n '2,19p' "$0" | sed 's/^# \{0,1\}//'
    exit 2
fi

REPO_ROOT=$(git rev-parse --show-toplevel)
OUT_DIR=$1
BASELINE_DIR=${2:-}
GOLDEN_DIR=${AUDITMYSITE_GOLDEN_DIR:-$REPO_ROOT/reports/golden}

if [ ! -d "$GOLDEN_DIR" ]; then
    echo "golden input dir not found: $GOLDEN_DIR (set AUDITMYSITE_GOLDEN_DIR)" >&2
    exit 2
fi

rm -rf "$OUT_DIR"
mkdir -p "$OUT_DIR"
# Absolute paths: cargo runs from the repo root.
OUT_DIR=$(cd "$OUT_DIR" && pwd)
GOLDEN_DIR=$(cd "$GOLDEN_DIR" && pwd)

(
    cd "$REPO_ROOT"
    AUDITMYSITE_GOLDEN_DIR="$GOLDEN_DIR" AUDITMYSITE_GOLDEN_OUT="$OUT_DIR" \
        cargo test --release --all-features --test golden_render -- --ignored
)

echo "rendered $(find "$OUT_DIR" -type f | wc -l | tr -d ' ') files into $OUT_DIR"

if [ -n "$BASELINE_DIR" ]; then
    if diff -r "$BASELINE_DIR" "$OUT_DIR"; then
        echo "golden-diff: identical to $BASELINE_DIR"
    else
        echo "golden-diff: output differs from $BASELINE_DIR" >&2
        exit 1
    fi
fi
