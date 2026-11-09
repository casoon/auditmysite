'use strict';

const crypto = require('node:crypto');

function sha256(buffer) {
  return crypto.createHash('sha256').update(buffer).digest('hex');
}

// Parses the release's `<hash>  <file name>` line (shasum / Get-FileHash output).
function parseChecksumFile(text, archiveName) {
  const match = /^([0-9a-fA-F]{64})(?:\s+\*?(.+?))?\s*$/.exec(text.trim());
  if (!match) {
    throw new Error(`Malformed checksum file for ${archiveName}`);
  }
  if (match[2] && match[2] !== archiveName) {
    throw new Error(`Checksum file is for ${match[2]}, expected ${archiveName}`);
  }
  return match[1].toLowerCase();
}

function verifyChecksum(buffer, checksumText, archiveName) {
  const expected = parseChecksumFile(checksumText, archiveName);
  const actual = sha256(buffer);
  if (actual !== expected) {
    throw new Error(
      `Checksum mismatch for ${archiveName}: expected ${expected}, got ${actual}. ` +
        'The download was corrupted or tampered with; nothing was installed.'
    );
  }
}

// Finds the archive's hash in the aggregate SHA256SUMS (`<hash>  <file name>` lines). The file
// name must match exactly: no prefix or suffix collisions (`other-<archive>`, `<archive>.sha256`).
function findInChecksums(text, archiveName) {
  const hashes = new Set();
  for (const line of text.replace(/\r/g, '').split('\n')) {
    const [hash, name] = line.trim().split(/\s+/);
    if (name !== undefined && name.replace(/^\*/, '') === archiveName) {
      if (!/^[0-9a-fA-F]{64}$/.test(hash)) {
        throw new Error(`SHA256SUMS has a malformed line for ${archiveName}`);
      }
      hashes.add(hash.toLowerCase());
    }
  }
  if (hashes.size === 0) {
    throw new Error(`SHA256SUMS has no line for ${archiveName}`);
  }
  if (hashes.size > 1) {
    throw new Error(`SHA256SUMS has conflicting lines for ${archiveName}`);
  }
  return [...hashes][0];
}

// Verifies the archive against the release's aggregate SHA256SUMS. HTTP 404 means the release
// predates it (up to v2.0.2), so the per-archive .sha256 file is used; any other status or a
// network error fails, so a broken or blocked SHA256SUMS never silently selects the weaker
// source. `fetch(url)` resolves { statusCode, body }. Returns the source that was used.
async function verifyRelease(buffer, assets, fetch) {
  let sums;
  try {
    sums = await fetch(assets.sumsUrl);
  } catch (error) {
    throw new Error(`Could not download SHA256SUMS: ${error.message}`);
  }
  if (sums.statusCode === 200) {
    const expected = findInChecksums(sums.body.toString('utf8'), assets.archive);
    verifyChecksum(buffer, `${expected}  ${assets.archive}`, assets.archive);
    return 'SHA256SUMS';
  }
  if (sums.statusCode !== 404) {
    throw new Error(`Downloading SHA256SUMS returned HTTP ${sums.statusCode} for ${assets.sumsUrl}`);
  }
  const file = await fetch(assets.checksumUrl);
  if (file.statusCode !== 200) {
    throw new Error(`Download failed: HTTP ${file.statusCode} for ${assets.checksumUrl}`);
  }
  verifyChecksum(buffer, file.body.toString('utf8'), assets.archive);
  return `${assets.archive}.sha256`;
}

module.exports = { sha256, parseChecksumFile, verifyChecksum, findInChecksums, verifyRelease };
