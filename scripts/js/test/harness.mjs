// Runs an embedded script (src/**/js/*.js) the way the page does, against stubs.
import { readdirSync, readFileSync } from "node:fs";
import path from "node:path";
import vm from "node:vm";

export const root = path.resolve(import.meta.dirname, "../../..");

/** Every embedded script as `src/<module>/js/<name>.js` (repo-relative, posix). */
export function scriptPaths(dir = path.join(root, "src")) {
  const found = [];
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      found.push(...scriptPaths(full));
    } else if (entry.name.endsWith(".js") && path.basename(dir) === "js") {
      found.push(path.relative(root, full).split(path.sep).join("/"));
    }
  }
  return found.sort();
}

export function source(rel) {
  return readFileSync(path.join(root, rel), "utf8");
}

/** Plain-string replacement of every placeholder, as the Rust side does it. */
export function fill(text, placeholders = {}) {
  let out = text;
  for (const [name, value] of Object.entries(placeholders)) {
    out = out.replaceAll(name, String(value));
  }
  return out;
}

/**
 * Evaluates `rel` as the page would and returns its completion value, copied
 * through JSON so results from the vm realm compare with `deepStrictEqual`.
 * `prefix` is prepended like the Rust side splices helpers in front of a body.
 */
export function run(rel, globals = {}, { placeholders = {}, prefix = "" } = {}) {
  const context = vm.createContext({ ...globals });
  context.window = context.window ?? context;
  const result = vm.runInContext(prefix + fill(source(rel), placeholders), context, {
    filename: rel,
  });
  return result === undefined ? undefined : JSON.parse(JSON.stringify(result));
}

/** A context with the `__ams*` helpers defined, for scripts that call them. */
export function withHelpers(globals = {}) {
  const context = vm.createContext({ ...globals });
  context.window = context.window ?? context;
  vm.runInContext(source("src/accessibility/js/helpers_css_selector.js"), context);
  return context;
}
