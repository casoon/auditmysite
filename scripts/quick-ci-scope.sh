#!/usr/bin/env bash
# Print a GitHub output; unknown history must run the Rust checks.
set -euo pipefail

if [[ -z "${BASE_SHA:-}" || "$BASE_SHA" =~ ^0+$ ]] ||
   ! git cat-file -e "$BASE_SHA^{commit}" 2>/dev/null; then
  echo 'rust=true'
  exit 0
fi

changes=$(mktemp)
trap 'rm -f "$changes"' EXIT
if [[ "${EVENT_NAME:-}" == pull_request ]]; then
  git diff --no-renames --name-only -z "$BASE_SHA...$HEAD_SHA" > "$changes"
else
  git diff --no-renames --name-only -z "$BASE_SHA" "$HEAD_SHA" > "$changes"
fi

while IFS= read -r -d '' path; do
  case "$path" in
    README.md|CHANGELOG.md|LICENSE|docs/index.md|docs/getting-started/*|docs/guides/*|docs/concepts/*|docs/reference/*|docs/img/*|site/*) ;;
    *) echo 'rust=true'; exit 0 ;;
  esac
done < "$changes"
echo 'rust=false'
