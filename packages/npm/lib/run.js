'use strict';

const { spawnSync } = require('node:child_process');
const os = require('node:os');

// Runs the binary with the given argv, inheriting stdio, and returns the exit code
// the wrapper should use. A signal-terminated child maps to 128 + signal number.
function forward(binaryPath, args, spawn = spawnSync) {
  const result = spawn(binaryPath, args, { stdio: 'inherit' });
  if (result.error) {
    throw result.error;
  }
  if (result.signal) {
    return 128 + (os.constants.signals[result.signal] || 0);
  }
  return result.status;
}

module.exports = { forward };
