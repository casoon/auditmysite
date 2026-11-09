#!/usr/bin/env node
'use strict';

const fs = require('node:fs');
const path = require('node:path');
const { forward } = require('../lib/run');

const binary = path.join(__dirname, '..', 'vendor', process.platform === 'win32' ? 'auditmysite.exe' : 'auditmysite');

if (!fs.existsSync(binary)) {
  console.error(
    'auditmysite: binary not found. The postinstall download was skipped or failed.\n' +
      `Run: node ${path.join(__dirname, '..', 'scripts', 'install.js')}`
  );
  process.exit(1);
}

process.exit(forward(binary, process.argv.slice(2)));
