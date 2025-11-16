#!/usr/bin/env node

/**
 * Migrate all Node.js built-in imports to use the node: protocol
 * This is a best practice for Node.js 16+ and improves clarity
 * 
 * Usage: node scripts/migrate-to-node-protocol.js
 */

const fs = require('fs');
const path = require('path');

const NODE_MODULES = [
  'assert', 'async_hooks', 'buffer', 'child_process', 'cluster', 'console',
  'constants', 'crypto', 'dgram', 'dns', 'domain', 'events', 'fs', 'http',
  'http2', 'https', 'inspector', 'module', 'net', 'os', 'path', 'perf_hooks',
  'process', 'punycode', 'querystring', 'readline', 'repl', 'stream',
  'string_decoder', 'sys', 'timers', 'tls', 'trace_events', 'tty', 'url',
  'util', 'v8', 'vm', 'worker_threads', 'zlib'
];

function migrateFile(filePath) {
  let content = fs.readFileSync(filePath, 'utf8');
  let modified = false;

  NODE_MODULES.forEach(module => {
    // Match import * as name from 'module'
    const pattern1 = new RegExp(`import \\* as (\\w+) from ['"]${module}['"]`, 'g');
    if (pattern1.test(content)) {
      content = content.replace(pattern1, `import * as $1 from 'node:${module}'`);
      modified = true;
    }

    // Match import { ... } from 'module'
    const pattern2 = new RegExp(`import \\{([^}]+)\\} from ['"]${module}['"]`, 'g');
    if (pattern2.test(content)) {
      content = content.replace(pattern2, `import {$1} from 'node:${module}'`);
      modified = true;
    }

    // Match import name from 'module'
    const pattern3 = new RegExp(`import (\\w+) from ['"]${module}['"]`, 'g');
    if (pattern3.test(content)) {
      content = content.replace(pattern3, `import $1 from 'node:${module}'`);
      modified = true;
    }

    // Match import 'module/subpath'
    const pattern4 = new RegExp(`from ['"]${module}\/([^'"]+)['"]`, 'g');
    if (pattern4.test(content)) {
      content = content.replace(pattern4, `from 'node:${module}/$1'`);
      modified = true;
    }
  });

  if (modified) {
    fs.writeFileSync(filePath, content, 'utf8');
    console.log(`✅ Migrated: ${filePath}`);
    return true;
  }

  return false;
}

function walkDirectory(dir) {
  const files = fs.readdirSync(dir);
  let migratedCount = 0;

  files.forEach(file => {
    const filePath = path.join(dir, file);
    const stat = fs.statSync(filePath);

    if (stat.isDirectory()) {
      if (!['node_modules', 'dist', '.git'].includes(file)) {
        migratedCount += walkDirectory(filePath);
      }
    } else if (file.endsWith('.ts') || file.endsWith('.js')) {
      if (migrateFile(filePath)) {
        migratedCount++;
      }
    }
  });

  return migratedCount;
}

console.log('🚀 Starting migration to node: protocol...\n');
const srcDir = path.join(__dirname, '..', 'src');
const count = walkDirectory(srcDir);
console.log(`\n✅ Migration complete! ${count} files updated.`);
