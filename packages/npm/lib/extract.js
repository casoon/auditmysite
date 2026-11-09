'use strict';

const path = require('node:path');

// tar reads .tar.gz everywhere; the Windows build is a .zip, which only the
// bsdtar shipped with Windows 10+ extracts (not a GNU tar earlier on PATH).
function extractCommand(platform, archivePath, destDir, env = process.env) {
  const tar =
    platform === 'win32'
      ? path.win32.join(env.SystemRoot || 'C:\\Windows', 'System32', 'tar.exe')
      : 'tar';
  return { command: tar, args: ['-xf', archivePath, '-C', destDir] };
}

module.exports = { extractCommand };
