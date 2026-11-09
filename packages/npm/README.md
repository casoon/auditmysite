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

The `postinstall` script downloads the archive and its `.sha256` file from the GitHub release of
the same tag, compares the SHA-256 and unpacks the binary into the package. A mismatch aborts the
install with an error. Behind a proxy, set `HTTPS_PROXY` (`http://` proxies only). Installing with
`--ignore-scripts` skips the download; run `node scripts/install.js` in the package afterwards.

The checksum comes from the same release as the archive, so it detects corrupted downloads, not a
manipulated release.

Source and issues: <https://github.com/casoon/auditmysite> (`packages/npm`). MIT license.
