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
- `auditmysite <url>`: run a full single-page audit and write PDF/JSON into the current directory
- `auditmysite --sitemap <url>`: audit sitemap URLs
- `auditmysite --url-file <file>`: audit URLs from file
- `auditmysite <url> --crawl`: discover same-domain pages from a seed URL and audit them as a batch
- `auditmysite browser detect`: show available browsers
- `auditmysite browser install [--headless-shell]`: download and install Chrome for Testing or the headless shell into `~/.auditmysite/browsers/` (opt-in, never automatic; used only when no system browser is found)
- `auditmysite doctor`: run local diagnostics

Useful flags:
- `--prefer-sitemap`: if a sitemap is detected for a base URL, switch directly into batch mode
- `--no-sitemap-suggest`: suppress sitemap probing/suggestion and keep the run on the single URL
- `--crawl-depth <n>`: limit same-domain crawl discovery depth when using `--crawl`
- `--per-page-reports`: scan a URL list or sitemap but write one individual report per URL instead of an aggregated batch report; `-o` is treated as a target directory. With `-f json`, `index.json` and `findings.jsonl` are written next to the page files (see [Technician mode](../../guides/technician-mode/))
- `--technician`: fix-oriented batch preset, see [Technician mode](../../guides/technician-mode/)
- `--include-path <glob>` / `--exclude-path <glob>`: select batch URLs by path before `--max-pages` (repeatable)
- `--no-screen-reader-report`: do not write the `*-screen-reader-audit.json` sidecar
- `--html-conform`: run the HTML-conformance module without `--full`
- `--lang <de|en>`: set the language for PDF reports (default: `de`)
- `--stack`: enable tech stack detection and stack-specific security probes (included automatically with `--full`)
- `--interactive <off|basic|full>`: control the Accessibility Journey Layer for interactive checks — tab walk, skip-link, modal focus trap, SPA navigation, form-error announcement, link-text inventory (default: `full`; use `off` for fastest runs). Clicking submit buttons, add-to-cart triggers and quantity steppers is an opt-in: `--allow-state-changes` (with `full`)
- `--annex <en301549|bik>`: add an opt-in appendix to the PDF report. `en301549` maps findings to EN 301 549 (chapter 9, "Web") clauses — a technical building block for a human-authored accessibility statement, not a statement itself. `bik` regroups the same findings by the chapters of the "BIK für Alle" editorial guide (images/alt text, link text, structure, easy language, PDFs, videos), without adding new checks. The underlying JSON data (`en301549_annex`, `bik_guide`) is always present regardless of this flag; it only gates the PDF section.
- `--dns-check`: opt-in, score-neutral DNS configuration check (CAA presence, best-effort DNSSEC, SPF when an MX record exists); runs once per host, not once per page
- `--isolate-third-party-impact`: reload the page once per top-5 third-party origin with that origin blocked and report each origin's Total Blocking Time impact; costly, requires `--full` or `--performance`, single-URL mode only
- `--check-ssr-content`: reload the page with JavaScript disabled and flag an SSR/hydration content gap when essential content only appears client-side; one extra reload, requires `--full` or `--seo`, single-URL mode only
- `--display <calm|text|visual|all>`: audit one display mode of the `data-display` convention — see [Display modes](../../guides/display-modes/)
- `--design-quality`: opt-in UX/readability heuristics, including alt-text quality checks (filename-like alt text, "image of" prefixes, overly long or redundant alt text); score-neutral and not part of `--full`
- `--exclude-selector <CSS>` (repeatable): drop findings inside the subtree a CSS selector matches — for markup that is broken on purpose, such as teaching specimens (see [Excluding intentional specimens](../../guides/excluding-specimens/))
- `--color <auto|always|never>` / `--progress <auto|always|never>`: terminal color and batch progress policy; `--progress` is independent of `--quiet`

For the full current interface, use:

```bash
auditmysite --help
auditmysite browser --help
```

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
