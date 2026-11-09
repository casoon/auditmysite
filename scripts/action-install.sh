#!/usr/bin/env bash
# Download, verify and install the auditmysite release binary for the runner platform.
# Used by action.yml; mirrors install.sh (same asset names). The archive is checked against
# the release's aggregate SHA256SUMS; releases without one (up to v2.0.2) fall back to the
# per-archive .sha256 file. Optionally the signed build provenance is verified as well.
# Inputs (environment): AUDITMYSITE_VERSION (tag like v2.0.2, or "latest"),
# INSTALL_DIR (target directory), VERIFY_ATTESTATION ("true" runs `gh attestation verify`,
# needs gh and GH_TOKEN; default "false"). Writes the installed version to $GITHUB_OUTPUT if set.
set -euo pipefail

REPO="casoon/auditmysite"
BINARY_NAME="auditmysite"
VERSION="${AUDITMYSITE_VERSION:-latest}"
INSTALL_DIR="${INSTALL_DIR:?INSTALL_DIR is required}"
VERIFY_ATTESTATION="${VERIFY_ATTESTATION:-false}"

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

case "$VERIFY_ATTESTATION" in
  true | false) ;;
  *) fail "verify-attestation must be true or false (got '$VERIFY_ATTESTATION')" ;;
esac

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

# Aggregate checksum file first. 404 means the release predates it; any other failure stops
# the install instead of silently choosing a weaker source.
sums_status=$(curl -sSL --retry 3 --retry-all-errors -o "$tmp_dir/SHA256SUMS" -w '%{http_code}' "$base_url/SHA256SUMS") ||
  fail "Could not download SHA256SUMS of $version"
case "$sums_status" in
  200)
    checksum_source="SHA256SUMS"
    # Exact match on the file name field: no regex, no prefix or suffix collisions.
    expected=$(tr -d '\r' <"$tmp_dir/SHA256SUMS" | awk -v name="$archive_name" '$2 == name {print tolower($1)}')
    [ -n "$expected" ] || fail "SHA256SUMS of $version has no line for $archive_name"
    ;;
  404)
    [ "$VERIFY_ATTESTATION" = "false" ] ||
      fail "verify-attestation is true, but $version predates SHA256SUMS and build provenance; use a later release or set verify-attestation to false"
    checksum_source="$archive_name.sha256"
    curl -fsSL --retry 3 --retry-all-errors "$base_url/$archive_name.sha256" -o "$tmp_dir/$archive_name.sha256" ||
      fail "No checksum file $archive_name.sha256 in $version"
    expected=$(awk '{print tolower($1)}' "$tmp_dir/$archive_name.sha256" | tr -d '\r')
    ;;
  *) fail "Downloading SHA256SUMS of $version returned HTTP $sums_status" ;;
esac

actual=$(sha256_of "$tmp_dir/$archive_name")
if [ -z "$expected" ] || [ "$actual" != "$expected" ]; then
  fail "Checksum verification failed for $archive_name (against $checksum_source)"
fi
echo "Checksum verified against $checksum_source: $actual"

if [ "$VERIFY_ATTESTATION" = "true" ]; then
  command -v gh >/dev/null 2>&1 || fail "verify-attestation needs the gh CLI on the runner"
  gh attestation verify "$tmp_dir/$archive_name" --repo "$REPO" ||
    fail "No valid build provenance attestation for $archive_name in $version (see gh output above)"
  echo "Build provenance verified for $archive_name"
fi

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
