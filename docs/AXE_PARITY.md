# axe-core Parity Workflow

A repeatable way to find and prioritize axe-core coverage gaps across real pages,
so every new WCAG rule (or regression) can be compared against axe-core without
ad-hoc scripting.

The comparison is driven by [`scripts/axe-compare.js`](../scripts/axe-compare.js),
which runs **auditmysite** and **axe-core** on the same URL and emits a
side-by-side Markdown report grouped by axe rule ID.

## Setup (once)

```bash
npm install playwright axe-core
npx playwright install chromium
cargo build --release   # the script calls ./target/release/auditmysite
```

## Running a comparison

```bash
# Markdown to stdout
node scripts/axe-compare.js https://www.bundestag.de

# Markdown + raw combined JSON (for reproducibility / offline re-analysis)
node scripts/axe-compare.js https://www.bundestag.de \
  --output reports/axe/bundestag.md \
  --raw-output reports/axe/bundestag.raw.json

# A different WCAG level or binary
node scripts/axe-compare.js https://example.com --level A --bin ./target/release/auditmysite
```

The script reads auditmysite's current single-report schema
(`pages[0].findings`) and stays backward compatible with the older
`report.findings` / `report.raw_wcag` shapes.

## Rule categories

Every row in the report is one axe rule ID, tagged with a `note`:

| Note | Meaning | Action |
|------|---------|--------|
| `✓ both` | Both tools flagged the rule on the page | Calibration is aligned; no action |
| `gap ← axe only` | axe-core reported a **confirmed violation** we missed | **Prioritize** — a real coverage gap |
| `only-axe (incomplete)` | axe-core returned `incomplete` (needs manual confirmation) | Track separately — *not* a confirmed gap |
| `only-us` | We flag it, axe-core does not | Review: intentional extension vs. false positive |
| `⚠` (on our result) | Our finding is a heuristic **warning**, not a confirmed violation | Lower confidence by design |

The `## axe-core only details` section lists, for each axe-only rule, up to three
sample nodes with the target selector, an HTML snippet, and axe's failure
summary — enough to reproduce and triage the gap.

## Policy: confirmed gaps vs. heuristics

To keep calibration honest, distinguish three confidence tiers and never mix them:

1. **Confirmed axe gap** (`gap ← axe only`, axe `violations`): axe is certain and
   we are silent. These are the only rows that count as true parity gaps and
   should drive new-rule work.
2. **axe incomplete / needs-review** (`only-axe (incomplete)`): axe could not
   decide (e.g. `color-contrast` over images, `aria-valid-attr-value` edge
   cases). Tracked separately; **not** counted as a gap. auditmysite may legitimately
   demote the same cases to manual-review warnings (see
   [contrast handling](OUTPUT_CONTRACT.md#performance-measurement-context) and
   issue #264).
3. **auditmysite-only** (`only-us`): our heuristic extensions. A `⚠` warning here is
   expected and must not be presented as an axe-confirmed finding. Before treating
   an `only-us` row as a false positive, confirm axe actually evaluates that rule
   on the page (it may be `incomplete` rather than passing).

When reporting parity status, quote the three counts separately, e.g.:

```text
Both tools flagged: 3
axe-core only confirmed gaps: 0
axe-core incomplete / needs review: 2
```

## Calibration URLs

A small, stable set of public pages that exercise different stacks and known
edge cases. Re-run after any rule change and compare against the previous
Markdown/raw artifacts.

| URL | Why it is in the set |
|-----|----------------------|
| https://www.bundestag.de | Hero/module text over images & gradients → `color-contrast` incomplete; `aria-valid-attr-value` edge cases |
| https://www.w3.org/WAI/demos/bad/before/home.html | "Before" inaccessible demo — dense, intentional violations across many rules |
| https://www.w3.org/WAI/demos/bad/after/home.html | "After" accessible demo — should be near-clean; guards against false positives |
| https://example.com | Minimal baseline — must stay quiet in both tools |

Store generated reports under `reports/axe/` (gitignored) or attach them as CI
artifacts. Keeping the `--raw-output` JSON makes a result reproducible without
re-running the browsers.

### Current Bundestag baseline

```text
Both tools flagged: 3
axe-core only confirmed gaps: 0
axe-core incomplete / needs review: 2   (color-contrast: 90, aria-valid-attr-value: 10)
```

## Acting on a confirmed gap

1. Reproduce with `--raw-output` and inspect the axe-only details section.
2. Add or extend the matching rule under `src/wcag/rules/` and register it in
   `src/wcag/rules/mod.rs` (map the axe ID in `src/taxonomy/rules.rs`).
3. Re-run the comparison; the row should move from `gap ← axe only` to `✓ both`.
4. Add a focused unit test for the new rule.
