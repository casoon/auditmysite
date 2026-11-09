---
title: "CLI and output contract"
description: "All command-line options and what the JSON and report outputs promise."
order: 2
---

## CLI

```text
auditmysite [OPTIONS] [URL] [COMMAND]
```

Primary commands:
- `auditmysite <url>`: run a full single-page audit and print a table; nothing is written (`-o report.json` or `-f pdf -o report.pdf` keeps a report)
- `auditmysite --sitemap <url>`: audit sitemap URLs
- `auditmysite --url-file <file>`: audit URLs from file
- `auditmysite <url> --crawl`: discover same-domain pages from a seed URL and audit them as a batch
- `auditmysite browser detect`: show available browsers
- `auditmysite browser install [--headless-shell]`: download and install Chrome for Testing or the headless shell into `~/.auditmysite/browsers/` (opt-in, never automatic; used only when no system browser is found)
- `auditmysite doctor`: run local diagnostics

Useful flags:
- `--prefer-sitemap`: if a sitemap is detected for a base URL, switch directly into batch mode
- `--crawl-depth <n>`: limit same-domain crawl discovery depth when using `--crawl`
- `--per-page-reports`: scan a URL list or sitemap but write one individual report per URL instead of an aggregated batch report; `-o` is treated as a target directory. With `-f json`, `index.json` and `findings.jsonl` are written next to the page files (see [Technician mode](../../guides/technician-mode/))
- `--modules <list>`: choose the modules that run next to the accessibility checks, comma-separated: `performance` (alias `perf`), `seo`, `security`, `mobile`, `html-conform`, `stack`, or `a11y` for the WCAG rules only. Without `--modules` (or with the no-op `--full`) every module runs. `--modules` and `--full` exclude each other.
- `--reuse-cache`: answer a single-URL run from the stored snapshot of an earlier run with the same configuration; without it every run measures afresh
- `--include-path <glob>` / `--exclude-path <glob>`: select batch URLs by path before `--max-pages` (repeatable)
- `--screen-reader-report`: also write the `<output>-screen-reader-audit.json` sidecar (off by default)
- `--lang <en|de>`: set the language of the PDF report and the terminal table (default: `en`)
- Tech stack detection (`--modules stack`, part of the default set) reads in-page signals only; it sends no requests of its own to the audited host
- `--interactive <off|basic|full>`: control the Accessibility Journey Layer for interactive checks — tab walk, skip-link, modal focus trap, SPA navigation, form-error announcement, link-text inventory (default: `full`; use `off` for fastest runs). Clicking submit buttons, add-to-cart triggers and quantity steppers is an opt-in: `--allow-state-changes` (with `full`)
- `--annex <en301549|bik>`: add an opt-in appendix to the PDF report. `en301549` maps findings to EN 301 549 (chapter 9, "Web") clauses — a technical building block for a human-authored accessibility statement, not a statement itself. `bik` regroups the same findings by the chapters of the "BIK für Alle" editorial guide (images/alt text, link text, structure, easy language, PDFs, videos), without adding new checks. The underlying JSON data (`en301549_annex`, `bik_guide`) is always present regardless of this flag; it only gates the PDF section.
- `--privacy-check`: opt-in (not part of `--full`), pre-consent privacy findings of one first visit in a clean browser context (requests, cookies, storage keys classified by provider; technical findings with a confidence class, no legal verdict, no score); one extra page load per audited page, see [privacy findings](../privacy-findings.md)
- `--isolate-third-party-impact`: reload the page once per top-5 third-party origin with that origin blocked and report each origin's Total Blocking Time impact; costly, requires the `performance` module (the default set or `--modules performance`), single-URL mode only
- `--check-ssr-content`: reload the page with JavaScript disabled and flag an SSR/hydration content gap when essential content only appears client-side; one extra reload, requires the `seo` module (the default set or `--modules seo`), single-URL mode only
- `--display <calm|text|visual|all>`: audit one display mode of the `data-display` convention — see [Display modes](../../guides/display-modes/)
- `--exclude-selector <CSS>` (repeatable): drop findings inside the subtree a CSS selector matches — for markup that is broken on purpose, such as teaching specimens (see [Excluding intentional specimens](../../guides/excluding-specimens/))
- `--color <auto|always|never>` / `--progress <auto|always|never>`: terminal color and batch progress policy; `--progress` is independent of `--quiet`

For the full current interface, use:

```bash
auditmysite --help
auditmysite browser --help
```

## Runtime per page

A default run (`auditmysite <url>`) is the full analysis set: accessibility on desktop and mobile, performance with throttled passes, SEO, security, mobile checks and the interactive journeys. Wall-clock time per page depends mostly on how many of these passes run and on the page itself.

Measured on 2026-11-08 with auditmysite 2.0.2 (release build), full Google Chrome 157 on macOS, Apple M4 Pro (14 cores), one run per cell, live sites over a home connection, the eight sites of the [reference set](https://github.com/casoon/auditmysite/blob/main/tests/fixtures/reference_sites.json) (gov.uk, gov.ie, casoon.de, mit.edu, bundesregierung.de, dm.de, berlin.de, basf.com). Single-page figures include the process and browser start.

| Mode (single page) | Flags | Range over 8 sites | Median |
|---|---|---|---|
| Default | none (same as `--full`) | 14.8 – 54.1 s | 28.0 s |
| Without journeys | `--interactive off` | 11.5 – 32.9 s | 17.8 s |
| WCAG rules only | `--interactive off --modules a11y` | 2.5 – 8.9 s | 7.0 s |

`--modules a11y` switches off every other module (performance, mobile, SEO, security, HTML conformance, tech stack), so the last row is the accessibility tree and the WCAG rules without the rest. Heavy, script-driven pages cost more: bundesregierung.de (54.1 s) and dm.de (53.6 s) were the slowest in the default mode; gov.uk (14.8 s) the fastest.

Batch runs share one browser, so the start-up cost is paid once. Same six URLs (gov.uk, gov.ie, casoon.de, mit.edu, bundesregierung.de, berlin.de) as a `--url-file`, `-f json`, default modules:

| Run | Flags | Total | Per page |
|---|---|---|---|
| Sequential | `-c 1` | 99.4 s | 16.6 s |
| Default concurrency | none | 90.1 s | 15.0 s |
| Three tabs | `-c 3` | 47.0 s | 7.8 s |
| Six tabs | `-c 6` | 33.9 s | 5.7 s |
| Three tabs, no journeys | `-c 3 --interactive off` | 26.6 s | 4.4 s |
| First three URLs | `-c 3 -m 3` | 24.4 s | 8.1 s |

All six pages were audited in every run. Concurrency does not scale linearly: three tabs were 2.1 times and six tabs 2.9 times faster than one. Divide the sequential per-page time by that factor, not by the tab count, to size a sitemap run. Timings vary from run to run and with load on the machine and the sites; treat them as an order of magnitude, not a promise.

### How the batch flags interact

- `-c, --concurrency <1-10>`: browser tabs audited at the same time. Default 3, except on macOS with a full Chrome while journeys are on: there the default is 1 (the `Default concurrency` row above), because parallel keyboard journeys can stall that browser (see [Browser setup](../../guides/browser-setup/)). An explicit `-c` always wins. `0` and values above 10 are rejected.
- `-t, --timeout <secs>`: page load timeout, default 30. It bounds one page, not the run. A page waiting for a free tab waits up to four times `--timeout` (two minutes at least); a page that still gets none is retried one at a time after the parallel phase.
- `--stability-budget-ms <ms>`: wait for late hydration after navigation, default 1500; the same budget again bounds the wait for finite CSS animations. Paid on every page, so it scales with the page count, not with concurrency.
- `-m, --max-pages <n>`: caps the number of audited URLs (0 = unlimited). `--include-path` and `--exclude-path` are applied first; `--sample-strategy first-n|stratified` decides which N are kept. This is the main lever on total time, since total time is roughly the page count times the per-page time divided by the concurrency speed-up.
- `--interactive off`, `--modules`: reduce the work per page and apply to every page of the batch.
- Single-URL only, not run in batch mode: `--perf-runs`, `--isolate-third-party-impact`, `--check-ssr-content`. `--privacy-check` adds one page load per audited page in either mode. `--crawl` adds link-check requests on top of the page audits.

## SARIF output

`-f sarif` writes a [SARIF 2.1.0](https://docs.oasis-open.org/sarif/sarif/v2.1.0/) log for SARIF consumers such as GitHub Code Scanning. It works for single pages and for batch inputs (sitemap, URL file, crawl); a batch produces one run with the findings of all audited pages.

```bash
auditmysite https://example.com -f sarif -o auditmysite.sarif --quiet
auditmysite --sitemap https://example.com/sitemap.xml -f sarif -o auditmysite.sarif --quiet
```

What the file contains:

- `tool.driver` carries the name, `informationUri` and the tool version; `rules` has one entry per distinct rule ID, merged across pages, with `shortDescription`, `fullDescription`, `help`, `defaultConfiguration.level` and `properties.precision` (from the finding's confidence). `helpUri` is set only for rules that have a help URL, in practice the WCAG rules (a link to the W3C "Understanding" page); rules of the other modules have none. `properties.wcagCriterion` is present for WCAG rules; `properties.tags` holds the category and dimension.
- Every finding of the report becomes a result, not only WCAG findings: SEO, security, UX, journey and the other modules are included, each under its own rule ID (for example `contrast/text-insufficient`, `seo.meta.description`). One result is written per occurrence.
- `level` follows the severity: critical and high are `error`, medium is `warning`, low is `note`. A finding that only asks for a manual review (`properties.outcome: "review"`) is capped at `note`: it is never presented as a confirmed defect.
- Checks that did not run are not results and never count as passed. Disabled rules, rules without the data they need, rules that errored, and modules that failed, ran partially or were skipped are listed in `invocations[0].toolExecutionNotifications` (`level` `note`, `warning` or `error`, with `associatedRule` and the page URL). A rule that has nothing to check on a page ("not applicable") is not listed. `executionSuccessful` is `false` when a notification has level `error`.

### Locations

An audited page is a URL, not a file in a repository, and auditmysite findings carry no source line or column (the browser reports elements by selector). So the writer does not write a `region`: a made-up line number would point nowhere.

- `physicalLocation.artifactLocation.uri` is a **relative pseudo-path** built from the page URL: `audited-pages/<host>/<path>`. An empty path or a path ending in `/` becomes `index`, a port is appended to the host with `-`, and a query string becomes the suffix `__q-<8 hex digits>` of its SHA-256 (`https://example.com/shop/?q=1` is `audited-pages/example.com/shop/index__q-<hash>`). A relative path is used on purpose: GitHub resolves absolute URIs against the checkout's `file://` root and rejects roots with a different scheme, so an `https://` URI would put the whole upload at risk.
- The real URL stays available in `run.artifacts[].description` ("Audited page ..."), in `properties.pageUrl` of each result and notification, and in the fingerprint.
- The element is a `logicalLocations` entry (`kind: element`, `name` the CSS selector, `fullyQualifiedName` `"<page URL> <selector>"`) and is appended to the message text (`... (selector: input#es-search-form-input)`).
- `run.properties.locationStrategy` is `page-pseudo-path`.

The pseudo-path matches no committed file, so GitHub lists an alert without a code view. To tie alerts to the repository file that produces a page (a static site, for instance), no mapping option exists yet.

### Fingerprints

Every result carries `partialFingerprints`, stable across runs, so the same defect on the same page is the same alert on the next upload:

```text
hash  = SHA-256(rule_id NUL page_url NUL kind NUL value), first 32 hex digits
value = the CSS selector if the occurrence has one (kind "selector"),
        else the message with whitespace collapsed (kind "message")
auditmysite/v1          = "<hash>:<n>"
primaryLocationLineHash = "<hash>:<n>"   (the only key GitHub reads)
```

`page_url` is the audited URL without fragment. The selector wins over the message because messages carry measured values (a contrast ratio) that change when the page is only edited. `n` counts results with an identical hash within one run, in report order. A changed selector (the element moved in the DOM) is a new alert; that is the price of having no source line to anchor on.

```json
{
  "ruleId": "contrast/text-insufficient",
  "ruleIndex": 0,
  "level": "error",
  "message": {
    "text": "The text reaches a contrast ratio of 3.04:1; 4.5:1 is required for normal text. (selector: span.site-nav__arrow)"
  },
  "locations": [
    {
      "physicalLocation": { "artifactLocation": { "uri": "audited-pages/www.mit.edu/index", "index": 0 } },
      "logicalLocations": [
        { "name": "span.site-nav__arrow", "fullyQualifiedName": "https://www.mit.edu/ span.site-nav__arrow", "kind": "element" }
      ]
    }
  ],
  "partialFingerprints": {
    "auditmysite/v1": "0f5c...:1",
    "primaryLocationLineHash": "0f5c...:1"
  },
  "properties": { "pageUrl": "https://www.mit.edu/", "outcome": "fail", "severity": "high", "selector": "span.site-nav__arrow" }
}
```

(One result, shortened.) The process exit code still encodes the verdict (0 pass, 1 warn, 2 fail). In CI, add `--report-mode` so the step succeeds whenever the file was written and a following upload step runs; gate the build separately if needed. The default `*-screen-reader-audit.json` sidecar is written next to `-o` for SARIF as well; `--no-screen-reader-report` skips it.

GitHub accepts at most 25,000 results per run (the 5,000 highest by severity are kept) and 10 MB gzipped per file; a large sitemap audit can exceed that, so scope it with `--max-pages`.

### GitHub Code Scanning

```yaml
name: Accessibility
on:
  workflow_dispatch:
jobs:
  audit:
    runs-on: ubuntu-latest
    permissions:
      contents: read
      security-events: write
    steps:
      - name: Install auditmysite
        run: curl -fsSL https://raw.githubusercontent.com/casoon/auditmysite/main/install.sh | bash
      - name: Audit
        run: auditmysite https://staging.example.com -f sarif -o auditmysite.sarif --quiet --report-mode
      - name: Upload SARIF
        uses: github/codeql-action/upload-sarif@33d7b85da1a04171e211befe2c739303b9a4c066 # v4
        with:
          sarif_file: auditmysite.sarif
          category: auditmysite
```

Status: the SARIF structure, the fingerprints and the not-run handling are covered by tests (`tests/sarif_contract.rs`, including a golden file), and the output has been validated against the official SARIF 2.1.0 JSON schema (opt-in test, command below). The workflow is a template and was **not run on GitHub**: whether GitHub shows results with a pseudo-path that matches no file, and as which alerts, has not been verified. Check the first upload in the repository's Security tab before relying on it. The SARIF file can also be kept as a workflow artifact.

Schema check: download `sarif-2.1/schema/sarif-schema-2.1.0.json` from [oasis-tcs/sarif-spec](https://github.com/oasis-tcs/sarif-spec) (OASIS licence terms, so it is not vendored here) and run `AUDITMYSITE_SARIF_SCHEMA=<file> cargo test --test sarif_contract -- --ignored`.

## Output Contract

JSON output is treated as an automation contract.

- Contract documentation: [docs/OUTPUT_CONTRACT.md](https://github.com/casoon/auditmysite/blob/main/docs/OUTPUT_CONTRACT.md)
- Single report schema: [docs/json-report.schema.json](https://github.com/casoon/auditmysite/blob/main/docs/json-report.schema.json)
- Batch report schema: [docs/json-batch-report.schema.json](https://github.com/casoon/auditmysite/blob/main/docs/json-batch-report.schema.json)

Key fields in a single-page report:
- `metric_context` — machine-readable definitions for the 0–100 score scale and the report's scoped count fields
- `findings` — static WCAG violations and SEO findings
- `interactive_findings` — journey-phase results (link texts, landmarks, heading outline, focus order, modal traps …); present when `--interactive basic|full` was used
- `accessibility_journey` — structured trace of each journey (steps, snapshots, durations); present when `--interactive basic|full` was used
- `audit_scope` and `execution_environment` — requested modules, viewports, throttle profiles, interaction mode, browser context, and live/cache provenance
- `audit_quality` plus `pages[].detail.module_runs` and `pages[].detail.rule_outcomes` — distinguish complete, partial, failed, skipped, and non-applicable checks so a measurement failure cannot look like a clean result
- `pages[].detail.accessibility_assessments` — structured warnings, manual-review items, and positive signals kept separate from confirmed violations and scoring
- `artifacts` — descriptors for separately written evidence or screen-reader sidecars without embedding binary data in the main JSON
- `build_id` — git short-SHA of the binary that produced the report (suffixed `-dirty` for builds from uncommitted changes, `unknown` for builds without a `.git` directory), so two reports with the same `tool_version` can be told apart
- `pages[].detail.en301549_annex` and `pages[].detail.bik_guide` — findings mapped to EN 301 549 clauses and to the "BIK für Alle" editorial guide chapters; always present, independent of `--annex`
- `pages[].detail.accessibility_subcategory_scores` and `pages[].detail.security_category_scores` — the Accessibility and Security scores broken down by subcategory (`{ name, score }`), the same breakdown the PDF shows; single reports only

Rule IDs are stable across releases. Version 1.3.0 renamed two IDs that collided with unrelated checks: the 4.1.2 control-label check is now `control-missing-label` (was `label`, which stays with the 3.3.2 label/instructions checks), and the 2.4.9 link-purpose (link only) check is now `link-name-only` (was `link-name`, which stays with the 2.4.4 check).

For dual-viewport audits, the Accessibility score is the rounded blend of 70% mobile and 30% desktop in both JSON and PDF. WCAG occurrences, distinct grouped WCAG findings, and findings from all categories are exposed separately so counts remain comparable across formats.

Batch JSON additionally exposes `site_analysis`: module averages, consistency signals, page types, topic overlap, duplicate content, structured-data distribution, performance rollups, interactive coverage, and aggregated accessibility assessments. Per-page entries remain compact instead of duplicating full single-page reports.

The repository validates these contracts in automated tests.
