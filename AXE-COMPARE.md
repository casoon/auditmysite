# axe-compare — Cross-Tool Comparison

`scripts/axe-compare.js` runs **auditmysite** and **axe-core** on the same URL
and produces a side-by-side Markdown table grouped by axe rule ID.

Use it to calibrate our WCAG rules: find gaps (axe flags something we miss),
potential false positives (we flag something axe doesn't), and severity disagreements.

## Setup (once)

```bash
cd scripts
npm install
npx playwright install chromium
```

## Usage

```bash
# Print table to stdout
node scripts/axe-compare.js https://example.com

# Write to a file in reports/
node scripts/axe-compare.js https://example.com \
  --output reports/example-axe-comparison.md

# WCAG Level A only
node scripts/axe-compare.js https://example.com --level A

# Custom binary path
node scripts/axe-compare.js https://example.com \
  --bin ./target/debug/auditmysite \
  --output reports/example-axe-comparison.md
```

## Options

| Flag | Default | Description |
|------|---------|-------------|
| `--output <file>` | stdout | Write Markdown output to this path |
| `--bin <path>` | `./target/release/auditmysite` | Path to the auditmysite binary |
| `--level <A\|AA>` | `AA` | WCAG level passed to auditmysite |

## Output format

```
# axe-core Comparison — https://example.com

Generated: 2026-05-12 10:00:00
Tool: auditmysite v0.10.2 · axe-core (wcag2a/aa/21a/21aa/22aa tags)

## Summary

| | Count |
|---|---|
| ✓ Both tools flagged | 5 |
| gap ← axe-core only | 3 |
| only-us (we flag, axe doesn't) | 2 |
| only-axe (incomplete/needs-review) | 1 |

## Rule-by-rule comparison

| axe-id | criterion | our result | axe-core result | note |
|--------|-----------|-----------|-----------------|------|
| `color-contrast` | 1.4.3 | – | 4 violations | gap ← axe only |
| `image-alt` | 1.1.1 | 2 findings | 2 violations | ✓ both |
| `keyboard` | 2.1.1 | 1 findings ⚠ | – | only-us (heuristic) |
```

## Row categories

| Note | Meaning | Action |
|------|---------|--------|
| `✓ both` | Both tools flagged this rule | Verify counts and severity agree |
| `gap ← axe only` | axe-core found violations we miss | Highest priority — implement or improve the rule |
| `only-us` | We flag it, axe-core doesn't | Check for false positives or intentional extension |
| `only-us (heuristic)` | Our warning (⚠), axe skips | Expected — heuristic checks go beyond axe scope |
| `only-axe (incomplete)` | axe returned `incomplete` (needs manual review) | Low priority — axe itself is uncertain here |

Rows are sorted: gaps first, then both-tools, then only-us.

## When to use

- **Before shipping a new rule** — verify against axe-core; explain any divergence
- **After a user reports a false positive/negative** — reproduce with this script
- **Periodically** against reference sites (casoon.de, a known accessible site)
- **When rule severity feels wrong** — compare impact labels side by side

## Reference sites

```bash
# casoon.de — known score ~79, 4 violations
node scripts/axe-compare.js https://www.casoon.de \
  --output reports/casoon-axe-comparison.md

# w3.org — highly accessible reference
node scripts/axe-compare.js https://www.w3.org/WAI/ \
  --output reports/w3-axe-comparison.md
```

## How it works

1. Runs `auditmysite <URL> --format json` and extracts `raw_wcag.violations`
   and `raw_wcag.warnings`, grouped by `rule_id` (the axe-compatible ID set
   via `.with_rule_id()` in each rule).
2. Opens the same URL with a headless Chromium via Playwright, injects axe-core,
   runs it with `wcag2a/aa/21a/21aa/22aa` tags, and collects `violations` +
   `incomplete`.
3. Builds a union of all rule IDs and emits the comparison table.

Violations without a `rule_id` appear as `(no-axe-id:2.x.x)` — a signal to
add `.with_rule_id()` to that rule.
