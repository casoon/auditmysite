'use strict';

// Targets that release.yml builds. Keep in sync with its build matrix.
const TARGETS = {
  'darwin-arm64': { target: 'aarch64-apple-darwin', ext: 'tar.gz' },
  'linux-x64': { target: 'x86_64-unknown-linux-gnu', ext: 'tar.gz' },
  'linux-arm64': { target: 'aarch64-unknown-linux-gnu', ext: 'tar.gz' },
  'win32-x64': { target: 'x86_64-pc-windows-msvc', ext: 'zip' },
};

const REPO = 'casoon/auditmysite';

function mapPlatform(platform, arch) {
  const entry = TARGETS[`${platform}-${arch}`];
  if (!entry) {
    const supported = Object.keys(TARGETS).join(', ');
    throw new Error(
      `No auditmysite release binary for ${platform}/${arch} (released: ${supported}). ` +
        'Build from source instead: cargo install auditmysite'
    );
  }
  return { ...entry, binary: platform === 'win32' ? 'auditmysite.exe' : 'auditmysite' };
}

function releaseAssets(version, platform, arch) {
  const { target, ext, binary } = mapPlatform(platform, arch);
  const tag = `v${version}`;
  const archive = `auditmysite-${tag}-${target}.${ext}`;
  const base = `https://github.com/${REPO}/releases/download/${tag}`;
  return {
    archive,
    binary,
    archiveUrl: `${base}/${archive}`,
    sumsUrl: `${base}/SHA256SUMS`,
    checksumUrl: `${base}/${archive}.sha256`,
  };
}

module.exports = { mapPlatform, releaseAssets };
