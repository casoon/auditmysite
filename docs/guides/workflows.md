---
title: "Typical workflows"
description: "Customer report, CI, AI fix list, dashboards, batch and technician runs."
order: 10
---

Examples grouped by audience and goal.

## Customer-facing report (PDF)

Single-URL audit with full module coverage and a custom logo on the cover.

```bash
# default: writes a PDF + JSON sidecar to the current directory
auditmysite https://example.com --full

# explicit branding and output path
auditmysite https://example.com --full --logo ./assets/customer-logo.svg --output reports/customer.pdf

# pick a report depth: executive (management), standard (default), technical (developers)
auditmysite https://example.com --full --report-level executive --output reports/exec.pdf

# PDF language (default: de)
auditmysite https://example.com --full --lang en --output reports/report-en.pdf
```

## CI / automation (JSON)

Quiet, machine-readable output for pipelines.

```bash
# exit code follows score thresholds; JSON report for downstream tooling
auditmysite https://example.com -f json -o report.json --quiet

# batch CI run on a sitemap
auditmysite --sitemap https://example.com/sitemap.xml -f json -o sitemap-report.json --quiet
```

## AI fix list

Compact, agent-friendly output that focuses on actionable fixes.

```bash
auditmysite https://example.com -f ai -o fixes.json
```

## Dashboard / ranking feed

Compact summary JSON with score, grade, medal, issue counts, and top 10 findings — matches the `lastAudit` schema used by dashboard tools.

```bash
auditmysite https://example.com -f summary -o summary.json
```

## Sitemap / batch

Domain-wide audits with cross-page aggregation.

```bash
# explicit sitemap
auditmysite --sitemap https://example.com/sitemap.xml --full

# crawl from a base URL
auditmysite https://example.com --crawl --crawl-depth 2 --max-pages 50 --full

# URL list from file
auditmysite --url-file urls.txt --full

# one PDF per URL instead of an aggregated batch report
auditmysite --sitemap https://example.com/sitemap.xml --per-page-reports --output reports/per-page/
```

## Technician mode

For people who fix the issues rather than read a report: one JSON file per page plus two flat
files to script against, no PDF.

```bash
auditmysite --sitemap https://example.com/sitemap.xml --technician -o reports/tech/
auditmysite --url-file urls.txt --technician -o reports/tech/

# only part of the site, before -m applies
auditmysite --sitemap https://example.com/sitemap.xml --technician \
  --include-path '/blog/**' --exclude-path '/blog/tag/**' -m 50 -o reports/tech/
```

`--technician` is shorthand for `--per-page-reports -f json --no-screen-reader-report --seo
--html-conform`. It runs the modules that produce fixable findings — accessibility (including the
keyboard journeys; `--interactive off` makes it faster), HTML conformance and SEO — and skips the
throttled performance passes, mobile, security and tech-stack detection. Each page file records
that partial scope in `execution.scope.requested_modules` and `execution.module_runs`. Flags you
give explicitly win: `-f` replaces the format, and `--full`, `--performance`, `--mobile`,
`--security` add their modules as usual.

The output directory then holds:

- one `<site>-<path>-<date>-single-report.json` per audited page (the regular single-page JSON),
- `index.json`: every attempted URL in input order with `file` (or `null`), `status`
  (`ok`, `blocked` for bot walls/access denials, `failed`), `reason`, `overall_score`,
  `accessibility_score`, `finding_count`, `occurrence_count` and `audit_quality`
  (schema: [`docs/technician-index.schema.json`](https://github.com/casoon/auditmysite/blob/main/docs/technician-index.schema.json)),
- `findings.jsonl`: one line per finding occurrence across all pages — `url`, `source`
  (`wcag`, `journey`, `seo`, `html_conform`), `rule_id`, `wcag_criterion`, `level`, `severity`,
  `selector`, `location`, `message`, `fix_suggestion`, `viewport_tags`
  (schema: [`docs/technician-finding.schema.json`](https://github.com/casoon/auditmysite/blob/main/docs/technician-finding.schema.json)).
  Unlike the page files, which keep a few example occurrences per finding, this list is complete.

Any `--per-page-reports -f json` run writes `index.json` and `findings.jsonl`; failed or blocked
pages appear only in `index.json`, never as a page file. All text is canonical English.

Path globs are matched against the whole, percent-decoded URL path (no host, no query): `*` and
`?` stay within one path segment, `**` crosses segments, and `/**/` also matches a single `/`.
`/blog/**` selects everything below `/blog/` but not `/blog` itself. A URL is audited when it
matches any `--include-path` (or none is given) and no `--exclude-path`. With `--crawl`, the
filters apply to the discovered pages; discovery itself is still capped by `-m`.

```bash
cd reports/tech
# the worst rules across the site
jq -r '"\(.severity)\t\(.rule_id)"' findings.jsonl | sort | uniq -c | sort -rn | head
# every critical/high occurrence with page and selector
jq -r 'select(.severity=="critical" or .severity=="high") | [.url, .rule_id, .selector // .location] | @tsv' findings.jsonl
# all occurrences of one WCAG criterion
jq -c 'select(.wcag_criterion=="1.4.3") | {url, selector, message}' findings.jsonl
# pages that were not audited, and why
jq -r '.pages[] | select(.status!="ok") | "\(.status)\t\(.url)\t\(.reason)"' index.json
# pages by accessibility score, lowest first
jq -r '.pages[] | select(.status=="ok") | "\(.accessibility_score)\t\(.occurrence_count)\t\(.url)"' index.json | sort -n
```

## Local development

```bash
# audit a local dev server with a system Chrome
auditmysite https://localhost:3000 --browser-path /Applications/Google\ Chrome.app/Contents/MacOS/Google\ Chrome

# quick CLI summary without writing files
auditmysite https://example.com --format table
```

## Base URL with sitemap suggestion

```bash
# interactive: ask first if a sitemap is found
auditmysite https://example.com

# non-interactive: switch directly to sitemap mode
auditmysite https://example.com --prefer-sitemap

# stay on the single URL even when a sitemap exists
auditmysite https://example.com --no-sitemap-suggest
```
