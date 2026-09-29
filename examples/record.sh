#!/usr/bin/env bash
# Re-records the real CLI output shown on the project site (site/src/showcase.ts and the
# start page read these files at build time). Needs an installed auditmysite, Chrome and
# network access; the audited page is live, so scores can change between recordings.
#
#   examples/record.sh [path-to-auditmysite]
set -euo pipefail

bin="${1:-auditmysite}"
dir="$(cd "$(dirname "$0")" && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
cd "$tmp"

# Single URL, terminal summary (progress lines go to stdout, the banner to stderr).
"$bin" https://www.casoon.de/ --format table --color always --no-sitemap-suggest --lang en \
  >"$dir/single-table.ansi" 2>/dev/null || true

# What an audit would do, without running it.
"$bin" plan https://www.casoon.de/ --lang en >"$dir/plan.txt" 2>&1

# JSON report, then two things derived from it.
"$bin" https://www.casoon.de/ -f json -o report.json --quiet --no-sitemap-suggest || true
jq '.summary.wcag_coverage' report.json >"$dir/wcag-coverage.json"
"$bin" report-lint report.json >"$dir/report-lint.txt" 2>&1
