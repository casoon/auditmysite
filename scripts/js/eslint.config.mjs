// Lint for the JavaScript embedded in the Rust sources (src/**/js/*.js, #941).
//
// Run from the repository root (scripts/check-js.sh does):
//   scripts/js/node_modules/.bin/eslint --config scripts/js/eslint.config.mjs
//
// The files are not modules. Chrome receives them as `Runtime.evaluate`
// expressions, `Runtime.callFunctionOn` declarations or as bodies the Rust side
// wraps in `(function () { ... })()`. So:
//   - top-level `return` is allowed (bodies),
//   - a file that is a bare `function (...) { ... }` is linted as the
//     parenthesised expression it is when Chrome parses it,
//   - `__NAME__` identifiers are placeholders the Rust side replaces, and
//     `__ams*` functions come from src/accessibility/js/helpers_*.js, which the
//     Rust side prepends to the rules that use them.
import { readdirSync, readFileSync } from "node:fs";
import path from "node:path";
import js from "@eslint/js";
import globals from "globals";

const root = path.resolve(import.meta.dirname, "../..");

function embeddedScripts(dir = path.join(root, "src")) {
  const found = [];
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      found.push(...embeddedScripts(full));
    } else if (entry.name.endsWith(".js") && path.basename(dir) === "js") {
      found.push(full);
    }
  }
  return found;
}

const scripts = embeddedScripts().map((file) => ({
  file,
  text: readFileSync(file, "utf8"),
}));

const injected = {};
for (const { text } of scripts) {
  for (const name of text.match(/\b__[A-Z][A-Z0-9_]*__/g) ?? []) {
    injected[name] = "readonly";
  }
}
for (const { file, text } of scripts) {
  if (path.basename(file).startsWith("helpers_")) {
    for (const [, name] of text.matchAll(/^function (__ams\w+)/gm)) {
      injected[name] = "readonly";
    }
  }
}

const bareFunction = /^\s*function\s*\(/;

const embedded = {
  meta: { name: "embedded-js" },
  processors: {
    script: {
      meta: { name: "embedded-js/script" },
      preprocess(text) {
        return [bareFunction.test(text) ? `(${text})` : text];
      },
      postprocess(messages) {
        return messages[0];
      },
    },
  },
};

export default [
  { ignores: ["**/node_modules/**"] },
  {
    files: ["src/**/js/*.js"],
    plugins: { embedded },
    processor: "embedded/script",
    languageOptions: {
      ecmaVersion: "latest",
      sourceType: "script",
      globals: { ...globals.browser, ...injected },
      parserOptions: { ecmaFeatures: { globalReturn: true } },
    },
    linterOptions: { reportUnusedDisableDirectives: true },
    rules: {
      ...js.configs.recommended.rules,
      // `try { ... } catch (e) {}` is how these probes say "unreadable, move on".
      "no-empty": ["error", { allowEmptyCatch: true }],
      "no-unused-vars": ["error", { args: "none", caughtErrors: "none" }],
    },
  },
  {
    // Declarations only: they are used by the scripts they are prepended to.
    files: ["src/**/js/helpers_*.js"],
    rules: { "no-unused-vars": "off" },
  },
  {
    // Replaced by the Rust side with the cap (`ON_INPUT_CAP`) before the run.
    files: ["src/wcag/rules/js/on_input_body.js"],
    languageOptions: { globals: { CAP: "readonly" } },
  },
  {
    // Probes the page's own libraries by name, `typeof` guarded.
    files: ["src/best_practices/js/library_globals.js"],
    languageOptions: {
      globals: Object.fromEntries(
        [
          "jQuery", "$", "angular", "Handlebars", "_", "Prototype",
          "MooTools", "moment", "bootstrap",
        ].map((name) => [name, "readonly"]),
      ),
    },
  },
  {
    // Findings of the extraction (#941), left as they were because it must not
    // change what the browser runs. Fixing one is a separate, behavioural change.
    files: ["src/mobile/js/mobile_friendliness.js"],
    rules: { "no-misleading-character-class": "off" },
  },
  {
    files: ["src/seo/js/headings.js", "src/wcag/rules/js/frame_tested.js"],
    rules: { "no-useless-assignment": "off" },
  },
  {
    files: ["src/seo/js/technical_signals.js"],
    rules: { "no-useless-escape": "off" },
  },
];
