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

// --- Release verification: SHA256SUMS with fallback to the per-archive .sha256 ---

const { findInChecksums, verifyRelease } = require('../lib/checksum');

const ASSETS = releaseAssets('2.0.2', 'linux', 'x64');
const ARCHIVE = Buffer.from('payload');
const OTHER_HASH = sha256(Buffer.from('other'));

// Fake fetch: url -> { statusCode, body } or an Error to reject with. Records the requested URLs.
function fakeFetch(responses) {
  const calls = [];
  const fetch = async (url) => {
    calls.push(url);
    const response = responses[url];
    if (response instanceof Error) throw response;
    if (!response) throw new Error(`unexpected request ${url}`);
    return { statusCode: response.statusCode, body: Buffer.from(response.body || '') };
  };
  return { fetch, calls };
}

test('release URLs include the aggregate SHA256SUMS', () => {
  assert.equal(ASSETS.sumsUrl, 'https://github.com/casoon/auditmysite/releases/download/v2.0.2/SHA256SUMS');
});

test('SHA256SUMS present and matching: verified, per-archive file never requested', async () => {
  const sums = `${OTHER_HASH}  auditmysite-v2.0.2-aarch64-apple-darwin.tar.gz\n${HASH}  ${ASSETS.archive}\n`;
  const { fetch, calls } = fakeFetch({ [ASSETS.sumsUrl]: { statusCode: 200, body: sums } });
  assert.equal(await verifyRelease(ARCHIVE, ASSETS, fetch), 'SHA256SUMS');
  assert.deepEqual(calls, [ASSETS.sumsUrl]);
});

test('SHA256SUMS mismatch fails and does not fall back', async () => {
  const { fetch, calls } = fakeFetch({
    [ASSETS.sumsUrl]: { statusCode: 200, body: `${OTHER_HASH}  ${ASSETS.archive}\n` },
  });
  await assert.rejects(verifyRelease(ARCHIVE, ASSETS, fetch), /Checksum mismatch/);
  assert.deepEqual(calls, [ASSETS.sumsUrl]);
});

test('SHA256SUMS without a line for the archive fails and does not fall back', async () => {
  const { fetch, calls } = fakeFetch({
    [ASSETS.sumsUrl]: { statusCode: 200, body: `${HASH}  auditmysite-v2.0.2-aarch64-apple-darwin.tar.gz\n` },
  });
  await assert.rejects(verifyRelease(ARCHIVE, ASSETS, fetch), /no line for/);
  assert.deepEqual(calls, [ASSETS.sumsUrl]);
});

test('SHA256SUMS decoy lines never match: other-<archive> and <archive>.sha256', () => {
  const decoys = `${HASH}  other-${ASSETS.archive}\n${HASH}  ${ASSETS.archive}.sha256\n${HASH}  x${ASSETS.archive}\n`;
  assert.throws(() => findInChecksums(decoys, ASSETS.archive), /no line for/);
  // With the real line present, the decoys neither add a candidate nor cause a conflict.
  assert.equal(findInChecksums(`${decoys}${OTHER_HASH}  ${ASSETS.archive}\n`, ASSETS.archive), OTHER_HASH);
});

test('SHA256SUMS with CRLF line endings and an uppercase hash is accepted', async () => {
  const sums = `${OTHER_HASH}  other.tar.gz\r\n${HASH.toUpperCase()}  ${ASSETS.archive}\r\n`;
  const { fetch } = fakeFetch({ [ASSETS.sumsUrl]: { statusCode: 200, body: sums } });
  assert.equal(await verifyRelease(ARCHIVE, ASSETS, fetch), 'SHA256SUMS');
});

test('SHA256SUMS with conflicting or malformed lines for the archive fails', () => {
  assert.throws(
    () => findInChecksums(`${HASH}  ${ASSETS.archive}\n${OTHER_HASH}  ${ASSETS.archive}\n`, ASSETS.archive),
    /conflicting/
  );
  assert.throws(() => findInChecksums(`nothex  ${ASSETS.archive}\n`, ASSETS.archive), /malformed/);
});

test('HTTP 503 on SHA256SUMS fails loudly and does not fall back', async () => {
  const { fetch, calls } = fakeFetch({
    [ASSETS.sumsUrl]: { statusCode: 503 },
    [ASSETS.checksumUrl]: { statusCode: 200, body: `${HASH}  ${ASSETS.archive}\n` },
  });
  await assert.rejects(verifyRelease(ARCHIVE, ASSETS, fetch), /SHA256SUMS returned HTTP 503/);
  assert.deepEqual(calls, [ASSETS.sumsUrl]);
});

test('a network error on SHA256SUMS fails loudly and does not fall back', async () => {
  const { fetch, calls } = fakeFetch({
    [ASSETS.sumsUrl]: new Error('ECONNRESET'),
    [ASSETS.checksumUrl]: { statusCode: 200, body: `${HASH}  ${ASSETS.archive}\n` },
  });
  await assert.rejects(verifyRelease(ARCHIVE, ASSETS, fetch), /Could not download SHA256SUMS: ECONNRESET/);
  assert.deepEqual(calls, [ASSETS.sumsUrl]);
});

test('HTTP 404 on SHA256SUMS falls back to the per-archive .sha256', async () => {
  const { fetch, calls } = fakeFetch({
    [ASSETS.sumsUrl]: { statusCode: 404 },
    [ASSETS.checksumUrl]: { statusCode: 200, body: `${HASH}  ${ASSETS.archive}\n` },
  });
  assert.equal(await verifyRelease(ARCHIVE, ASSETS, fetch), `${ASSETS.archive}.sha256`);
  assert.deepEqual(calls, [ASSETS.sumsUrl, ASSETS.checksumUrl]);
});

test('fallback still fails on a mismatch or a missing .sha256', async () => {
  const mismatch = fakeFetch({
    [ASSETS.sumsUrl]: { statusCode: 404 },
    [ASSETS.checksumUrl]: { statusCode: 200, body: `${OTHER_HASH}  ${ASSETS.archive}\n` },
  });
  await assert.rejects(verifyRelease(ARCHIVE, ASSETS, mismatch.fetch), /Checksum mismatch/);
  const missing = fakeFetch({ [ASSETS.sumsUrl]: { statusCode: 404 }, [ASSETS.checksumUrl]: { statusCode: 404 } });
  await assert.rejects(verifyRelease(ARCHIVE, ASSETS, missing.fetch), /HTTP 404/);
});
