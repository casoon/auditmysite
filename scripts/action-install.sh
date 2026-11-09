#!/usr/bin/env bash
# Download, verify and install the auditmysite release binary for the runner platform.
# Used by action.yml; mirrors install.sh (same asset names, same .sha256 check).
# Inputs (environment): AUDITMYSITE_VERSION (tag like v2.0.2, or "latest"),
# INSTALL_DIR (target directory). Writes the installed version to $GITHUB_OUTPUT if set.
set -euo pipefail

REPO="casoon/auditmysite"
BINARY_NAME="auditmysite"
VERSION="${AUDITMYSITE_VERSION:-latest}"
INSTALL_DIR="${INSTALL_DIR:?INSTALL_DIR is required}"

fail() {
  echo "::error::$1" >&2
  exit 1
}

detect_platform() {
  local os arch
  os=$(uname -s | tr '[:upper:]' '[:lower:]')
  arch=$(uname -m)
  case "$os:$arch" in
    linux:x86_64 | linux:amd64) echo "x86_64-unknown-linux-gnu tar.gz" ;;
    linux:aarch64 | linux:arm64) echo "aarch64-unknown-linux-gnu tar.gz" ;;
    darwin:arm64 | darwin:aarch64) echo "aarch64-apple-darwin tar.gz" ;;
    darwin:x86_64 | darwin:amd64) echo "x86_64-apple-darwin tar.gz" ;;
    mingw*:* | msys*:* | cygwin*:*) echo "x86_64-pc-windows-msvc zip" ;;
    *) fail "Unsupported runner platform: $os/$arch" ;;
  esac
}

resolve_version() {
  if [ "$VERSION" != "latest" ]; then
    echo "$VERSION"
    return
  fi
  # The /releases/latest redirect ends in /tag/<tag>; no API call, no rate limit.
  curl -fsSLI -o /dev/null -w '%{url_effective}' "https://github.com/$REPO/releases/latest" | sed -E 's#.*/tag/##'
}

sha256_of() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{print $1}'
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$1" | awk '{print $1}'
  else
    fail "sha256sum or shasum is required for checksum verification"
  fi
}

read -r platform ext <<<"$(detect_platform)"
version=$(resolve_version)
case "$version" in
  v[0-9]*) ;;
  *) fail "Could not determine a release tag (got '$version')" ;;
esac

archive_name="${BINARY_NAME}-${version}-${platform}.${ext}"
base_url="https://github.com/$REPO/releases/download/$version"

tmp_dir=$(mktemp -d)
trap 'rm -rf "$tmp_dir"' EXIT

echo "Installing $BINARY_NAME $version for $platform"
curl -fsSL --retry 3 --retry-all-errors "$base_url/$archive_name" -o "$tmp_dir/$archive_name" ||
  fail "No release asset $archive_name in $version (is this platform published?)"
curl -fsSL --retry 3 --retry-all-errors "$base_url/$archive_name.sha256" -o "$tmp_dir/$archive_name.sha256" ||
  fail "No checksum file $archive_name.sha256 in $version"

expected=$(awk '{print tolower($1)}' "$tmp_dir/$archive_name.sha256" | tr -d '\r')
actual=$(sha256_of "$tmp_dir/$archive_name")
if [ -z "$expected" ] || [ "$actual" != "$expected" ]; then
  fail "Checksum verification failed for $archive_name"
fi
echo "Checksum verified: $actual"

mkdir -p "$INSTALL_DIR"
if [ "$ext" = "zip" ]; then
  # 7z ships on GitHub's Windows runners; unzip is used where present.
  if command -v unzip >/dev/null 2>&1; then
    unzip -q -o "$tmp_dir/$archive_name" -d "$INSTALL_DIR"
  else
    7z x -y "-o$INSTALL_DIR" "$tmp_dir/$archive_name" >/dev/null
  fi
else
  tar -xzf "$tmp_dir/$archive_name" -C "$tmp_dir"
  mv "$tmp_dir/$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"
  chmod +x "$INSTALL_DIR/$BINARY_NAME"
fi

if [ -n "${GITHUB_OUTPUT:-}" ]; then
  echo "version=$version" >>"$GITHUB_OUTPUT"
fi
echo "Installed $BINARY_NAME $version to $INSTALL_DIR"
