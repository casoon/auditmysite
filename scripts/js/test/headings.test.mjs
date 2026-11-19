// The SEO heading inventory marks headings that live in a dialog or a consent
// layer (#1132). Stub DOM: only what the script reads from a node.
import assert from "node:assert/strict";
import test from "node:test";
import { run } from "./harness.mjs";

/** An element with attributes; `host` makes it the shadow host of a root. */
function el(tagName, { attrs = {}, parent = null, text = "", shadow = null, root } = {}) {
  const node = {
    nodeType: 1,
    tagName,
    id: attrs.id ?? "",
    className: attrs.class ?? "",
    parentElement: parent,
    shadowRoot: shadow,
    childNodes: text ? [{ nodeType: 3, nodeValue: text }] : [],
    getAttribute: (name) => (name === "class" || name === "id" ? null : (attrs[name] ?? null)),
    closest: () => null,
    getRootNode: () => root ?? document,
    checkVisibility: () => true,
  };
  return node;
}

const document = {
  body: { lastChild: {} },
  createRange: () => ({
    setStartAfter() {},
    setEndBefore() {},
    setEndAfter() {},
    setEnd() {},
    cloneContents: () => ({}),
  }),
  createElement: () => ({ appendChild() {}, querySelectorAll: () => [], textContent: "" }),
  querySelectorAll: () => [],
};

function inventory(elements) {
  document.querySelectorAll = () => elements;
  return JSON.parse(run("src/seo/js/headings.js", { document, getComputedStyle: () => ({ display: "inline" }) }));
}

test("a heading inside a dialog, alertdialog, aria-modal or <dialog> is marked", () => {
  for (const [tag, attrs] of [
    ["DIV", { role: "dialog" }],
    ["DIV", { role: "alertdialog" }],
    ["DIV", { "aria-modal": "true" }],
    ["DIALOG", {}],
  ]) {
    const layer = el(tag, { attrs });
    const [h] = inventory([el("H1", { parent: layer, text: "Privacy settings" })]);
    assert.equal(h.in_dialog, true, `${tag} ${JSON.stringify(attrs)}`);
  }
});

test("a heading inside the root of a known consent manager is marked", () => {
  const layer = el("DIV", { attrs: { id: "CybotCookiebotDialog" } });
  const [h] = inventory([el("H2", { parent: layer, text: "Cookies" })]);
  assert.equal(h.in_dialog, true);
});

test("a heading in the shadow root of a dialog host is marked", () => {
  const host = el("DIV", { attrs: { role: "dialog" } });
  const [h] = inventory([el("H1", { text: "Settings", root: { host } })]);
  assert.equal(h.in_dialog, true);
});

test("a heading of the page content is not marked, nor one under a plain wrapper", () => {
  const main = el("MAIN");
  const wrapper = el("DIV", { attrs: { class: "hero" }, parent: main });
  const [top, nested] = inventory([
    el("H1", { text: "Our services" }),
    el("H2", { parent: wrapper, text: "Details" }),
  ]);
  assert.equal(top.in_dialog, false);
  assert.equal(nested.in_dialog, false);
});
