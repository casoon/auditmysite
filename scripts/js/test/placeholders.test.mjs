// The placeholders in src/**/js/*.js and the ones the Rust side replaces must be
// the same set: a placeholder nobody fills in reaches Chrome as literal text, a
// replacement nobody needs is dead code.
import assert from "node:assert/strict";
import { readdirSync, readFileSync } from "node:fs";
import path from "node:path";
import test from "node:test";
import { root, scriptPaths, source } from "./harness.mjs";

/** Names that look like placeholders but are globals of the pages we audit. */
const PAGE_GLOBALS = new Set([
  "__AUDITMYSITE_READY__",
  "__NEXT_DATA__",
  "__NUXT__",
  "__REACT_DEVTOOLS_GLOBAL_HOOK__",
  "__VUE__",
]);

/** Placeholders written as a bare word: replaced by `str::replace("CAP", ..)`. */
const BARE_WORD = new Set(["CAP"]);

const SHAPE = /\b__[A-Z][A-Z0-9_]*__|\/\*[A-Z_]+\*\//g;

function rustFiles(dir = path.join(root, "src")) {
  return readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) return rustFiles(full);
    return entry.name.endsWith(".rs") ? [full] : [];
  });
}

const inScripts = new Set(
  scriptPaths().flatMap((rel) => source(rel).match(SHAPE) ?? []),
);
for (const name of PAGE_GLOBALS) inScripts.delete(name);

const replaced = new Set();
for (const file of rustFiles()) {
  const text = readFileSync(file, "utf8");
  for (const [, token] of text.matchAll(/\.replace\(\s*"([^"]+)"/g)) {
    if (/^(__[A-Z][A-Z0-9_]*__|\/\*[A-Z_]+\*\/)$/.test(token) || BARE_WORD.has(token)) {
      replaced.add(token);
    }
  }
}

test("every placeholder in a script is replaced by the Rust side", () => {
  const unfilled = [...inScripts].filter((name) => !replaced.has(name));
  assert.deepEqual(unfilled, []);
});

test("every placeholder the Rust side replaces exists in a script", () => {
  const texts = scriptPaths().map(source);
  const dead = [...replaced].filter(
    (name) => !texts.some((text) => text.includes(name)),
  );
  assert.deepEqual(dead, []);
});
