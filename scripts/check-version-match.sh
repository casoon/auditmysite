#!/bin/sh

set -eu

REPO_ROOT=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
cd "$REPO_ROOT"

CARGO_VERSION="$(sed -n 's/^version = "\([^"]*\)"$/\1/p' Cargo.toml | head -n1)"

if [ -z "$CARGO_VERSION" ]; then
  echo "Could not read package.version from Cargo.toml"
  exit 1
fi

checked=0

check_tag() {
  raw_tag="$1"
  case "$raw_tag" in
    refs/tags/*) tag_name="${raw_tag#refs/tags/}" ;;
    *) tag_name="$raw_tag" ;;
  esac

  case "$tag_name" in
    v*)
      tag_version="${tag_name#v}"
      if [ "$tag_version" != "$CARGO_VERSION" ]; then
        echo "Tag version $tag_name does not match Cargo.toml version $CARGO_VERSION"
        exit 1
      fi
      checked=1
      ;;
  esac
}

if [ "${GITHUB_REF_NAME:-}" != "" ]; then
  check_tag "$GITHUB_REF_NAME"
fi

for arg in "$@"; do
  check_tag "$arg"
done

if [ ! -t 0 ]; then
  while IFS=' ' read -r local_ref local_sha remote_ref remote_sha; do
    if [ -n "${local_ref:-}" ]; then
      check_tag "$local_ref"
    fi
  done
fi

if [ "$checked" -eq 1 ]; then
  echo "version-check: tag matches Cargo.toml version $CARGO_VERSION"
fi
