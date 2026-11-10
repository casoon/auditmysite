// Behaviour of the embedded scripts that do not need a real layout engine:
// they read `performance` entries or walk a small element graph.
import assert from "node:assert/strict";
import vm from "node:vm";
import test from "node:test";
import { run, source, withHelpers } from "./harness.mjs";

/** Minimal element: what the scripts under test read from a node. */
function el(props = {}, parent = null) {
  return { parentElement: parent, getRootNode: () => ({}), ...props };
}

test("content_weight lists the navigation entry first, then the resources", () => {
  const performance = {
    getEntriesByType(kind) {
      if (kind === "navigation") {
        return [{ name: "https://x.test/", transferSize: 10, decodedBodySize: 40, duration: 5 }];
      }
      return [{ name: "https://x.test/a.js", initiatorType: "script", duration: 2 }];
    },
  };
  const out = JSON.parse(run("src/performance/js/content_weight.js", { performance }));
  assert.deepEqual(out, [
    { name: "https://x.test/", type: "navigation", transferSize: 10, decodedSize: 40, duration: 5 },
    { name: "https://x.test/a.js", type: "script", transferSize: 0, decodedSize: 0, duration: 2 },
  ]);
});

test("content_weight without a navigation entry lists the resources only", () => {
  const performance = { getEntriesByType: (kind) => (kind === "navigation" ? [] : []) };
  assert.equal(run("src/performance/js/content_weight.js", { performance }), "[]");
});

test("access_signals reads the status only when the browser reports a real one", () => {
  const page = (nav) => ({
    performance: {
      getEntriesByType: (kind) => (kind === "navigation" ? nav : [{ name: "https://x.test/r.css" }]),
    },
    document: {
      title: "Hi",
      querySelectorAll: () => [{ getAttribute: (a) => (a === "src" ? "/app.js" : null) }],
      getElementsByTagName: () => ({ length: 7 }),
    },
  });
  const ok = run("src/audit/js/access_signals.js", page([{ responseStatus: 403 }]));
  assert.deepEqual(ok, {
    status: 403,
    title: "Hi",
    elements: 7,
    resources: ["/app.js", "https://x.test/r.css"],
  });
  assert.equal(run("src/audit/js/access_signals.js", page([{ responseStatus: 0 }])).status, null);
  assert.equal(run("src/audit/js/access_signals.js", page([])).status, null);
});

test("exclusion_count_matches counts per selector and reports -1 for an invalid one", () => {
  const document = {
    querySelectorAll(selector) {
      if (selector === "!!") throw new SyntaxError("bad selector");
      return { length: selector.length };
    },
  };
  const out = run("src/audit/js/exclusion_count_matches.js", { document }, {
    placeholders: { __SELECTORS__: JSON.stringify(["a", "div.x", "!!"]) },
  });
  assert.deepEqual(out, [1, 5, -1]);
});

test("exclusion_selectors_inside is true only when every match lies inside an excluded subtree", () => {
  const root = el({ matches: (s) => s === "[data-skip]" });
  const inside = el({ matches: () => false }, root);
  const outside = el({ matches: () => false });
  const byselector = { ".in": [inside], ".out": [outside], ".mixed": [inside, outside], ".none": [] };
  const document = { querySelectorAll: (s) => byselector[s] };
  const out = run("src/audit/js/exclusion_selectors_inside.js", { document }, {
    placeholders: {
      __SELECTORS__: JSON.stringify([".in", ".out", ".mixed", ".none"]),
      __EXCLUDED__: JSON.stringify(["[data-skip]"]),
    },
  });
  assert.deepEqual(out, [true, false, false, false]);
});

test("exclusion_marker installs window.__amsIsExcluded for the subtrees and their descendants", () => {
  const root = el();
  const child = el({}, root);
  const stranger = el();
  const window = {};
  const document = { querySelectorAll: (s) => (s === "[data-skip]" ? [root] : []) };
  const installed = run("src/audit/js/exclusion_marker.js", { window, document }, {
    placeholders: { __EXCLUDED__: JSON.stringify(["[data-skip]"]) },
  });
  assert.equal(installed, true);
  assert.equal(window.__amsIsExcluded(root), true);
  assert.equal(window.__amsIsExcluded(child), true);
  assert.equal(window.__amsIsExcluded(stranger), false);
});

test("the shared cap helpers count real and excluded hits separately", () => {
  const excluded = el();
  const context = withHelpers({});
  context.window.__amsIsExcluded = (node) => node === excluded;
  const list = [];
  const push = (node, item) => vm.runInContext("__amsPush", context)(list, node, item, 2);
  assert.equal(push(el(), "r1"), true);
  assert.equal(push(excluded, "x1"), true);
  assert.equal(push(el(), "r2"), true);
  assert.equal(push(el(), "r3"), false, "real hits are capped at the limit");
  assert.equal(push(excluded, "x2"), true);
  assert.equal(push(excluded, "x3"), false, "excluded hits are capped separately");
  assert.deepEqual([...list], ["r1", "x1", "r2", "x2"]);
  assert.equal(vm.runInContext("__amsReal", context)(list), 2);
});

test("__amsCssSelector builds a short path that stops at an id", () => {
  const html = el({ tagName: "HTML" });
  const body = el({ tagName: "BODY", classList: [] }, html);
  const main = el({ tagName: "MAIN", id: "content", classList: [] }, body);
  const li = (cls, parent) =>
    el({ tagName: "LI", classList: cls, children: [], parentElement: parent }, parent);
  const list = el({ tagName: "UL", classList: [], children: [] }, main);
  main.children = [list];
  const a = li(["item", "first"], list);
  const b = li(["item"], list);
  list.children.push(a, b);
  const context = withHelpers({ document: { documentElement: html } });
  const selector = vm.runInContext("__amsCssSelector", context);
  assert.equal(selector(b), "main#content > ul > li.item:nth-of-type(2)");
  assert.equal(selector(null), "");
});

test("a script source with a placeholder left in does not parse as the filled one", () => {
  // Guards the harness itself: filled and unfilled text are different programs.
  const text = source("src/audit/js/exclusion_count_matches.js");
  assert.ok(text.includes("__SELECTORS__"));
  assert.doesNotThrow(() => new vm.Script(text.replace("__SELECTORS__", "[]")));
});
