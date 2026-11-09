# auditmysite (npm)

WCAG 2.2 AA accessibility checker. This package is a thin wrapper around the
[auditmysite](https://github.com/casoon/auditmysite) Rust binary: on install it downloads the
release binary for your platform and `auditmysite` forwards its arguments and exit code to it.

```bash
npx auditmysite https://example.com
npm install --save-dev auditmysite   # then: npx auditmysite --help
```

The package version is the auditmysite version, so `auditmysite@2.0.2` installs the `v2.0.2`
release binary.

## Requirements

- Node.js 18 or newer (the wrapper uses only Node built-ins).
- A released platform: macOS arm64, Linux x64/arm64 (glibc), Windows x64. Elsewhere use
  `cargo install auditmysite`.
- Chrome, Chromium or Edge for the audit itself, see the
  [main README](https://github.com/casoon/auditmysite#requirements).

## What the install does

The `postinstall` script downloads the archive from the GitHub release of the same tag, compares
its SHA-256 with the release's aggregate `SHA256SUMS` and unpacks the binary into the package.
Releases up to v2.0.2 have no `SHA256SUMS` (HTTP 404); for those the per-archive `.sha256` file is
used instead. Any other answer for `SHA256SUMS` (a server error, a network failure, no line for the
archive) and any mismatch abort the install with an error; there is no silent fallback. Behind a
proxy, set `HTTPS_PROXY` (`http://` proxies only). Installing with `--ignore-scripts` skips the
download; run `node scripts/install.js` in the package afterwards.

The checksum comes from the same release as the archive, so it detects corrupted downloads, not a
manipulated release. The wrapper relies on this checksum only: it does not verify build
provenance, because a postinstall script should not shell out to `gh`.

## Verify the binary yourself

Releases after v2.0.2 carry signed build provenance for every archive. To check that the binary
was built from this repository by its release workflow, download the archive of your platform from
the [release page](https://github.com/casoon/auditmysite/releases) and run:

```bash
gh attestation verify auditmysite-v<version>-<target>.tar.gz --repo casoon/auditmysite
```

(`.zip` on Windows.) The GitHub CLI `gh` is required; releases up to v2.0.2 have no attestation.
The checksums can be compared with `sha256sum -c SHA256SUMS` next to the downloaded files.

Source and issues: <https://github.com/casoon/auditmysite> (`packages/npm`). MIT license.
