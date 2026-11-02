#!/usr/bin/env bash
# Fetch the ACT rules repository at the pinned commit into a local cache (#837).
#
# The ACT test cases are not copied into this repository: they are generated
# from the rule files (`_rules/*.md`) and reference assets under
# `test-assets/` (88 MB). This script downloads the commit-addressed tarball,
# keeps `_rules/` and `test-assets/`, and verifies a content digest over both
# trees. The digest, not the tarball hash, is pinned: GitHub does not
# guarantee byte-stable gzip output, but the file contents of a commit are.
#
# Usage: scripts/act-fetch.sh            # prints the cache directory
#        ACT_CACHE_DIR=/path scripts/act-fetch.sh
#
# Cache: $ACT_CACHE_DIR, else $CARGO_TARGET_DIR/act-cache, else ./target/act-cache.
# Keep the commit and digest in sync with docs/act-mapping.json (`act_source`).
set -euo pipefail

COMMIT="50d2bede1ee705be6840d06f356a6ae2fd94cb2c"
CONTENT_DIGEST="e6fde4538e4c4d45d7727909cec7bc519bb3ae25dd9c001dbac0779f720b324a"

root="$(cd "$(dirname "$0")/.." && pwd)"
cache="${ACT_CACHE_DIR:-${CARGO_TARGET_DIR:-$root/target}/act-cache}"
dest="$cache/$COMMIT"

digest() {
  (cd "$1" && find _rules test-assets -type f -print0 | LC_ALL=C sort -z | xargs -0 shasum -a 256 | shasum -a 256 | cut -d' ' -f1)
}

if [ -f "$dest/.verified" ] && [ "$(cat "$dest/.verified")" = "$CONTENT_DIGEST" ]; then
  echo "$dest"
  exit 0
fi

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
curl -fsSL "https://codeload.github.com/act-rules/act-rules.github.io/tar.gz/$COMMIT" -o "$work/repo.tgz"
mkdir "$work/x"
tar xzf "$work/repo.tgz" -C "$work/x" --strip-components=1

actual="$(digest "$work/x")"
if [ "$actual" != "$CONTENT_DIGEST" ]; then
  echo "ACT content digest mismatch for $COMMIT" >&2
  echo "  expected $CONTENT_DIGEST" >&2
  echo "  actual   $actual" >&2
  exit 1
fi

rm -rf "$dest"
mkdir -p "$dest"
mv "$work/x/_rules" "$work/x/test-assets" "$dest/"
cp "$work/x/LICENSE.md" "$dest/LICENSE.md"
echo "$CONTENT_DIGEST" > "$dest/.verified"
echo "$dest"
