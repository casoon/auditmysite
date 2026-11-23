---
title: "Compared to typical setups"
description: "How auditmysite differs from static scanners and from axe-core, Pa11y and Lighthouse, with one measured run of time, memory and findings."
order: 4
---

- Better fit for JavaScript-heavy sites than static HTML-only checks
- Easier to distribute than a multi-package browser toolchain
- More automation-friendly than ad hoc console output because the JSON contract is explicit and tested
- Broader reporting surface than a pure accessibility-only checker when you also want performance, SEO, security, and mobile signals
- Violations carry stable `rule_id`, `tags`, and `impact` — easier to integrate with existing tooling or dashboards

## Screen Reader Audit vs. axe-core / Pa11y

Standard accessibility checkers verify individual rules in isolation. `auditmysite` additionally simulates the sequential experience of a screen reader user navigating the page — detecting problems that only emerge in context.

| Capability | axe-core | Pa11y | auditmysite |
|---|---|---|---|
| Rule-based WCAG checks | ✓ | ✓ | ✓ |
| Reading sequence simulation | — | — | ✓ |
| Out-of-context link text analysis (duplicate "Read more" × 8) | — | — | ✓ |
| Accessible name quality score (not just present/absent) | — | — | ✓ |
| Landmark navigation strategy (can a SR user reach main content?) | — | — | ✓ |
| BFSG / EN 301 549 legal mapping per finding | — | — | ✓ |

Every finding also carries a per-criterion EN 301 549 (chapter 9, "Web") clause reference. A structured version — all 55 WCAG 2.2 A/AA clauses of EN 301 549 V4.1.1 split into "violations found", "no violations in the automated scope", or "manual review required", plus which chapters (5–8, 10–13) sit outside this tool's audit scope entirely — is always in the JSON (`en301549_annex`) and can be added to the PDF as an appendix with `--annex en301549`. This is explicitly **not** an accessibility statement and doesn't claim to be one — it's a technical building block for a human-authored one, with an explicit scope disclaimer in both languages.

When the screen reader module runs and `--screen-reader-report` is given, a JSON sidecar is written next to the primary report:

```
example-com-YYYY-MM-DD-single-report.pdf
example-com-YYYY-MM-DD-single-report-screen-reader-audit.json  ← sidecar, only with --screen-reader-report
```

The sidecar models what a screen reader would typically announce, node by node, based on rule-based conventions (name, role, and state) — not a verified 1:1 reproduction of what NVDA, JAWS, or VoiceOver actually announce, which can vary by assistive technology, browser, and locale. It also flags which announcements are ambiguous or missing, suitable as a developer reference and as supporting evidence for BFSG compliance audits. The file is created only with `--screen-reader-report`, whenever screen reader data is available in the audit result.

## Speed and memory

`scripts/bench-compare.py` runs auditmysite, pa11y, @axe-core/cli and Lighthouse on the same pages with the same Chrome and records wall time, peak memory and which WCAG success criteria each tool reports. It is run by hand before a release, not in CI. The numbers below are one run; they describe this machine and these pages, not every setup.

### Measured run

2026-11-23, auditmysite 2.1.0 (release build, commit `b888ebd`), pa11y 10.0.0 (axe-core 4.13.0 inside), Lighthouse 13.6.0 (axe-core 4.14.0 inside), Node 24.22.1, Google Chrome 158.0.8127.37 for every tool. Apple M4 Pro, 14 cores, 48 GB RAM, macOS 27.2. The machine was not idle: the one-minute load average at the start of a run was between 5.9 and 23.5 (median 9.4). A second run directly before agreed within 8 % on every median.

Pages: three repository fixtures from a local static server (`tests/fixtures/perfect.html`, `tests/fixtures/many_violations.html`, `tests/fixtures/detection_corpus/forms_embedded.html`) and three live pages (example.com, gov.uk, w3.org/WAI). Five runs per page for the accessibility-only scopes, three for the full ones.

| Tool and scope | Median s/page | Factor | Range s | Peak memory, process tree, MB (median / max) | Of that without the browser, MB (median) |
|---|---|---|---|---|---|
| auditmysite, WCAG rules only (`--modules a11y --interactive off`) | 2.42 | 1.4× | 1.61 – 4.36 | 972 / 1283 | 44 |
| auditmysite, default run (every module, journeys) | 10.08 | 5.6× | 8.40 – 50.46 | 986 / 1490 | 60 |
| pa11y, default runner (HTML_CodeSniffer) | 1.79 | 1.0× | 1.62 – 2.68 | 829 / 1064 | 99 |
| pa11y, `--runner axe` | 1.84 | 1.0× | 1.76 – 3.31 | 830 / 1018 | 101 |
| Lighthouse, `--only-categories=accessibility` | 9.84 | 5.5× | 9.48 – 11.98 | 1160 / 1588 | 280 |
| Lighthouse, all default categories | 10.18 | 5.7× | 9.74 – 12.22 | 1395 / 1913 | 367 |
| @axe-core/cli 4.14.0 | — | — | — | — | not run, see below |

Median seconds per page:

| Page | auditmysite a11y | auditmysite default | pa11y | pa11y axe | Lighthouse a11y | Lighthouse all |
|---|---|---|---|---|---|---|
| local `perfect.html` | 1.75 | 9.03 | 1.76 | 1.77 | 9.52 | 9.85 |
| local `many_violations.html` | 1.77 | 8.41 | 1.77 | 1.78 | 9.58 | 9.75 |
| local `forms_embedded.html` | 1.94 | 8.66 | 1.79 | 2.40 | 10.13 | 10.22 |
| example.com | 3.56 | 10.83 | 1.81 | 1.85 | 9.80 | 10.03 |
| gov.uk | 2.48 | 17.94 | 1.64 | 1.81 | 9.87 | 10.57 |
| w3.org/WAI | 3.07 | 46.74 | 2.06 | 2.57 | 10.94 | 11.56 |

What the run shows:

- With the WCAG rules only, auditmysite takes about as long as pa11y on the local pages (1.7 to 1.9 s, browser start included) and 0.7 to 1.8 s longer on the live ones. Lighthouse needs about 10 s per page for the accessibility category alone, because it loads and traces the page the same way it does for all categories.
- The default auditmysite run costs 8 to 18 s per page here and 47 s on w3.org/WAI, where the keyboard journeys and the throttled performance passes have more to do. It is not the same work as Lighthouse with all categories: it adds a mobile viewport, journeys, security and HTML conformance, and Lighthouse adds its own performance trace.
- Memory is dominated by Chrome in every tool. auditmysite's own process stays at 44 MB (60 MB in the default run), against about 100 MB of Node for pa11y and 280 to 370 MB for Lighthouse. The Chrome side of an auditmysite run is larger than pa11y's, so the whole tree ends up about 140 MB above pa11y and about 190 MB below Lighthouse's accessibility run.

### Findings on the same pages

The script reduces each tool's violations on the last run to WCAG success criteria and compares them per page with auditmysite (WCAG rules only). Sum over the six pages:

| Tool | Criteria reported by both | Only auditmysite | Only the other tool | Other tool's findings without a criterion |
|---|---|---|---|---|
| pa11y (HTML_CodeSniffer) | 6 | 9 | 3 | 0 |
| pa11y (axe) | 8 | 7 | 4 | 3 |
| Lighthouse (accessibility) | 8 | 7 | 3 | 3 |

The differences have identifiable causes:

- `forms_embedded.html` puts its forms in an open shadow root and a same-origin iframe. HTML_CodeSniffer reports nothing there; axe, Lighthouse and auditmysite judge that content.
- auditmysite reports criteria the other tools did not: a missing form label under 3.3.2 (axe files its `label` rule under 4.1.2), click handlers on elements that cannot take focus (2.1.1), a change of context on input (3.2.2), blocked paste (3.3.8) and a label-in-name mismatch on gov.uk (2.5.3).
- axe tags `link-name` with both 2.4.4 and 4.1.2, auditmysite files a link without a name under 4.1.2 only, so the same defect shows up as "2.4.4 only in the other tool".
- Lighthouse, which audits in an emulated mobile viewport by default, reports `target-size` (2.5.8) on `perfect.html`; auditmysite does not.
- On the live pages pa11y reported 1.3.1 and 1.4.3 on gov.uk (HTML_CodeSniffer) and 1.4.3 and `bypass` (2.4.1) on w3.org/WAI and example.com (axe), none of which auditmysite reported; auditmysite reported 4.1.2 (ARIA attributes, a button without a name) on w3.org/WAI, which only HTML_CodeSniffer shares. Which side is right per case is not judged here.

Findings without a criterion are best-practice rules (`region`, `landmark-one-main` and similar). auditmysite's screen-reader findings carry no `wcag_criterion` and are not counted either.

### Method

- **Same browser.** Every tool gets the Chrome that `auditmysite browser path` selects: auditmysite with `--browser-path`, pa11y through `chromeLaunchConfig.executablePath`, Lighthouse through `CHROME_PATH` with `--headless=new`, @axe-core/cli with `--chrome-path`. pa11y, @axe-core/cli and Lighthouse are installed at pinned versions into a throwaway directory with `npm install --ignore-scripts`, so no package downloads its own browser.
- **Warm, not cold.** One discarded warm-up run of every tool on the first page fills the OS file cache. Each measured run is a new process with a new browser and a fresh profile; no tool keeps a browser between runs. A cold start after a reboot is not measured.
- **Round-robin.** Each round runs every tool on every page in turn, so a change in machine load spreads over all tools instead of hitting one.
- **Time** is wall-clock from process start to exit, browser start and report writing included.
- **Memory** is the largest sum of RSS over the process tree (the tool process, its descendants and its process group, so the browser and every helper), sampled every 100 ms with `ps`. RSS counts shared pages once per process, so the sum overstates unique memory; it is the same measure for every tool. A spike shorter than 100 ms can be missed. "Without the browser" is the same sum over the processes whose executable name does not contain `chrom` (the tool, Node, a version-manager shim).
- **Findings** map axe rule ids through the WCAG tags of the axe-core version that tool ships, HTML_CodeSniffer codes through the criterion in the code, and auditmysite findings through `wcag_criterion`. Only pa11y errors, Lighthouse binary audits with score 0 and axe violations count; warnings and "needs review" results do not.

### Not covered

- **@axe-core/cli did not run on this machine.** It drives Chrome through chromedriver, and the only chromedriver here was version 153 against Chrome 158 ("This version of ChromeDriver only supports Chrome version 153"). The script records the reason and skips the tool; with a matching driver (`--chromedriver PATH`) it is measured like the others. axe-core itself is covered through pa11y's axe runner and Lighthouse.
- One machine, one operating system, one day. Live pages add network variance; the local fixtures are small and do not stand for heavy pages.
- The run does not judge finding quality. The overlap table says where the tools differ, not which one is right; that needs the rule contracts and a manual check.

Run it with `cargo build --release` and `scripts/bench-compare.py [--runs 5] [--full-runs 3] [--no-public] [--out result.json] [--markdown tables.md]`; the full run takes about 17 minutes.
