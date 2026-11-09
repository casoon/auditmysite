'use strict';

const { test } = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const { mapPlatform, releaseAssets } = require('../lib/platform');
const { sha256, parseChecksumFile, verifyChecksum } = require('../lib/checksum');
const { extractCommand } = require('../lib/extract');
const { forward } = require('../lib/run');
const { proxyFor } = require('../lib/download');

const HASH = sha256(Buffer.from('payload'));

test('platform mapping covers exactly the released targets', () => {
  assert.equal(mapPlatform('darwin', 'arm64').target, 'aarch64-apple-darwin');
  assert.equal(mapPlatform('linux', 'x64').target, 'x86_64-unknown-linux-gnu');
  assert.equal(mapPlatform('linux', 'arm64').target, 'aarch64-unknown-linux-gnu');
  const win = mapPlatform('win32', 'x64');
  assert.deepEqual([win.target, win.ext, win.binary], ['x86_64-pc-windows-msvc', 'zip', 'auditmysite.exe']);
});

test('unsupported platforms fail with a hint', () => {
  assert.throws(() => mapPlatform('darwin', 'x64'), /No auditmysite release binary for darwin\/x64.*cargo install/);
  assert.throws(() => mapPlatform('freebsd', 'x64'), /freebsd\/x64/);
});

test('asset names and URLs follow release.yml', () => {
  const a = releaseAssets('2.0.2', 'linux', 'x64');
  assert.equal(a.archive, 'auditmysite-v2.0.2-x86_64-unknown-linux-gnu.tar.gz');
  assert.equal(
    a.archiveUrl,
    'https://github.com/casoon/auditmysite/releases/download/v2.0.2/auditmysite-v2.0.2-x86_64-unknown-linux-gnu.tar.gz'
  );
  assert.equal(a.checksumUrl, `${a.archiveUrl}.sha256`);
  assert.equal(releaseAssets('2.0.2', 'win32', 'x64').archive, 'auditmysite-v2.0.2-x86_64-pc-windows-msvc.zip');
});

test('package version equals the Cargo.toml version', () => {
  const cargo = fs.readFileSync(path.join(__dirname, '..', '..', '..', 'Cargo.toml'), 'utf8');
  const cargoVersion = /^version = "([^"]+)"/m.exec(cargo)[1];
  assert.equal(require('../package.json').version, cargoVersion);
});

test('checksum accepts a matching shasum line, including CRLF and a binary marker', () => {
  const name = 'a.tar.gz';
  verifyChecksum(Buffer.from('payload'), `${HASH}  ${name}\n`, name);
  verifyChecksum(Buffer.from('payload'), `${HASH.toUpperCase()}  ${name}\r\n`, name);
  verifyChecksum(Buffer.from('payload'), `${HASH} *${name}\n`, name);
  verifyChecksum(Buffer.from('payload'), HASH, name);
});

test('checksum mismatch fails loudly', () => {
  assert.throws(
    () => verifyChecksum(Buffer.from('tampered'), `${HASH}  a.tar.gz\n`, 'a.tar.gz'),
    /Checksum mismatch for a\.tar\.gz/
  );
});

test('checksum file for another archive or malformed content is rejected', () => {
  assert.throws(() => parseChecksumFile(`${HASH}  other.tar.gz`, 'a.tar.gz'), /is for other\.tar\.gz/);
  assert.throws(() => parseChecksumFile('not a hash', 'a.tar.gz'), /Malformed/);
  assert.throws(() => parseChecksumFile('', 'a.tar.gz'), /Malformed/);
});

test('extract command: tar on unix, System32 bsdtar on Windows', () => {
  assert.deepEqual(extractCommand('linux', '/t/a.tar.gz', '/t'), {
    command: 'tar',
    args: ['-xf', '/t/a.tar.gz', '-C', '/t'],
  });
  assert.equal(
    extractCommand('win32', 'C:\\t\\a.zip', 'C:\\t', { SystemRoot: 'C:\\Windows' }).command,
    'C:\\Windows\\System32\\tar.exe'
  );
});

test('proxy comes from HTTPS_PROXY, only http:// proxies are accepted', () => {
  assert.equal(proxyFor({}), null);
  assert.equal(proxyFor({ HTTPS_PROXY: 'http://proxy.test:3128' }).hostname, 'proxy.test');
  assert.equal(proxyFor({ https_proxy: 'http://proxy.test:3128' }).port, '3128');
  assert.throws(() => proxyFor({ HTTPS_PROXY: 'https://proxy.test' }), /Unsupported proxy protocol/);
});

test('forward passes argv unchanged and returns the exit code', () => {
  let seen;
  const code = forward('/bin/x', ['https://a.test', '--format', 'json', 'a b'], (cmd, args, opts) => {
    seen = { cmd, args, opts };
    return { status: 3, signal: null };
  });
  assert.equal(code, 3);
  assert.deepEqual(seen.args, ['https://a.test', '--format', 'json', 'a b']);
  assert.equal(seen.opts.stdio, 'inherit');
});

test('forward maps a killing signal to 128 + signal number', () => {
  assert.equal(forward('/bin/x', [], () => ({ status: null, signal: 'SIGINT' })), 128 + os.constants.signals.SIGINT);
});

test('forward rethrows spawn errors', () => {
  const error = new Error('ENOENT');
  assert.throws(() => forward('/nope', [], () => ({ error })), /ENOENT/);
});

test('forward runs a real child: arguments and exit code survive', { skip: process.platform === 'win32' }, () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'ams-npm-test-'));
  try {
    const fake = path.join(dir, 'fake');
    const out = path.join(dir, 'argv.json');
    fs.writeFileSync(
      fake,
      `#!${process.execPath}\nrequire('fs').writeFileSync(${JSON.stringify(out)}, JSON.stringify(process.argv.slice(2)));\nprocess.exit(7);\n`
    );
    fs.chmodSync(fake, 0o755);
    assert.equal(forward(fake, ['--url-file', 'my urls.txt', '-q']), 7);
    assert.deepEqual(JSON.parse(fs.readFileSync(out, 'utf8')), ['--url-file', 'my urls.txt', '-q']);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});
