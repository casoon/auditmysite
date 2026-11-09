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

module.exports = { sha256, parseChecksumFile, verifyChecksum };
