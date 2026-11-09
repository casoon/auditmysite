#!/usr/bin/env node
'use strict';

// postinstall: download the release binary for this platform, verify its SHA-256
// against the release's .sha256 file, and unpack it to vendor/.

const fs = require('node:fs');
const path = require('node:path');
const { spawnSync } = require('node:child_process');
const { releaseAssets } = require('../lib/platform');
const { verifyChecksum } = require('../lib/checksum');
const { extractCommand } = require('../lib/extract');
const { get } = require('../lib/download');
const { version } = require('../package.json');

async function main() {
  const assets = releaseAssets(version, process.platform, process.arch);
  const vendorDir = path.join(__dirname, '..', 'vendor');
  fs.mkdirSync(vendorDir, { recursive: true });

  console.log(`auditmysite: downloading ${assets.archive}`);
  const [archive, checksumText] = await Promise.all([
    get(assets.archiveUrl),
    get(assets.checksumUrl).then((buffer) => buffer.toString('utf8')),
  ]);
  verifyChecksum(archive, checksumText, assets.archive);

  const work = fs.mkdtempSync(path.join(vendorDir, '.tmp-'));
  try {
    const archivePath = path.join(work, assets.archive);
    fs.writeFileSync(archivePath, archive);
    const { command, args } = extractCommand(process.platform, archivePath, work);
    const result = spawnSync(command, args, { stdio: 'inherit' });
    if (result.error || result.status !== 0) {
      throw new Error(`Extracting ${assets.archive} with ${command} failed`);
    }
    const target = path.join(vendorDir, assets.binary);
    fs.renameSync(path.join(work, assets.binary), target);
    fs.chmodSync(target, 0o755);
  } finally {
    fs.rmSync(work, { recursive: true, force: true });
  }
  console.log(`auditmysite: installed ${assets.binary} ${version} (checksum verified)`);
}

main().catch((error) => {
  console.error(`auditmysite: install failed: ${error.message}`);
  process.exit(1);
});
