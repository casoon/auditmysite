#!/usr/bin/env bash
# Re-records the real CLI output shown on the project site (site/src/showcase.ts and the
# start page read these files at build time). Needs auditmysite, Chrome, jq, castwright
# (with node-pty) and network access; the audited page is live, so scores can change between
# recordings.
#
#   examples/record.sh [path-to-auditmysite]
#
# castwright is called as `castwright`; set CASTWRIGHT to use another command, e.g.
#   CASTWRIGHT="node ~/GitHub/castwright/packages/core/bin/castwright.mjs" examples/record.sh
set -euo pipefail

bin="$(command -v "${1:-auditmysite}")"
castwright="${CASTWRIGHT:-castwright}"
dir="$(cd "$(dirname "$0")" && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
cd "$tmp"

# The demos type `auditmysite …`, so the binary under test must be first on PATH.
mkdir bin
ln -s "$bin" bin/auditmysite
export PATH="$tmp/bin:$PATH"

# What an audit would do, without running it.
auditmysite plan https://www.casoon.de/ --lang en >"$dir/plan.txt" 2>&1

# JSON report and the coverage block derived from it.
auditmysite https://www.casoon.de/ -f json -o coverage.json --quiet --no-sitemap-suggest || true
jq '.summary.wcag_coverage' coverage.json >"$dir/wcag-coverage.json"

# Terminal demos: castwright runs each exec: step once in a pseudo-terminal (--record rewrites
# the file with what it printed), then compiles the recorded file to a cast (the page's text
# version) and an animated SVG. --request-mode skips the interactive prompt a terminal gets.
cat >single-table.terminal.yaml <<'YAML'
version: 1

terminal:
  title: auditmysite single-page audit of casoon.de
  cols: 90
  rows: 32
  theme: github-dark

steps:
  - exec: auditmysite https://www.casoon.de/ --format table --no-sitemap-suggest --lang en --request-mode browser
    timeout: 180000
    idle: 1500
  - wait: 3000
YAML

cat >technician.terminal.yaml <<'YAML'
version: 1

terminal:
  title: auditmysite technician mode on casoon.de
  cols: 100
  rows: 36
  theme: github-dark

steps:
  - exec: auditmysite --sitemap https://www.casoon.de/sitemap.xml --technician -m 3 -o tech --request-mode browser
    timeout: 300000
    idle: 1500
  - exec: jq -c .totals tech/index.json
  - exec: jq -r '"\(.severity)\t\(.rule_id)\t\(.url)"' tech/findings.jsonl
  - wait: 3000
YAML

cat >report-lint.terminal.yaml <<'YAML'
version: 1

terminal:
  title: auditmysite report-lint on a casoon.de report
  cols: 100
  rows: 8
  theme: github-dark

steps:
  - exec: auditmysite https://www.casoon.de/ -f json -o report.json --quiet --no-sitemap-suggest
    timeout: 180000
    idle: 1500
  - exec: auditmysite report-lint report.json; echo "exit code $?"
  - wait: 3000
YAML

for demo in single-table technician report-lint; do
  $castwright build "$demo.terminal.yaml" --allow-exec --record -o "$tmp/out"
  cp "$demo.terminal.yaml" "$dir/"
  $castwright build "$dir/$demo.terminal.yaml" -o "$dir"
  $castwright build "$dir/$demo.terminal.yaml" --format svg --no-chrome -o "$dir"
done
