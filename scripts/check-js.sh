#!/usr/bin/env bash
# Lint and unit-test the JavaScript embedded in the Rust sources (src/**/js/*.js, #941).
#
# Locally a missing Node skips with a message (the Rust checks do not need it);
# in CI (CI=true) a missing or too old Node is an error.
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
tools="$root/scripts/js"

skip() {
  if [[ -n "${CI:-}" ]]; then
    echo "check-js: $1" >&2
    exit 1
  fi
  echo "check-js: SKIPPED - $1"
  exit 0
}

command -v node >/dev/null 2>&1 || skip "node not found (need Node 20.19+, 22.13+ or 24+ to lint and test src/**/js/*.js)"
command -v npm >/dev/null 2>&1 || skip "npm not found"

if ! node -e '
  const [major, minor] = process.versions.node.split(".").map(Number);
  process.exit(major >= 24 || (major === 22 && minor >= 13) || (major === 20 && minor >= 19) ? 0 : 1);
'; then
  skip "node $(node --version) is too old (need 20.19+, 22.13+ or 24+)"
fi

# Install the pinned tools (package-lock.json) when missing or out of date.
if [[ ! -x "$tools/node_modules/.bin/eslint" || "$tools/package-lock.json" -nt "$tools/node_modules/.package-lock.json" ]]; then
  (cd "$tools" && npm ci --no-audit --no-fund --ignore-scripts --loglevel=error)
fi

cd "$root"
echo "check-js: eslint"
"$tools/node_modules/.bin/eslint" --config "$tools/eslint.config.mjs" "src/**/js/*.js"
echo "check-js: node --test"
(cd "$tools" && node --test)
