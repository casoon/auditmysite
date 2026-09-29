---
title: Technician mode
description: One JSON file per page plus index.json and findings.jsonl, for people who fix the issues rather than read a report. Path globs select which URLs of a site are audited.
order: 3
---

For people who fix the issues rather than read a report: one JSON file per page plus two flat
files to script against, no PDF.

```sh
auditmysite --sitemap https://example.com/sitemap.xml --technician -o reports/tech/
auditmysite --url-file urls.txt --technician -o reports/tech/
```

`--technician` is shorthand for `--per-page-reports -f json --no-screen-reader-report --seo
--html-conform`. It runs the modules that produce fixable findings — accessibility (including the
keyboard journeys; `--interactive off` makes it faster), HTML conformance and SEO — and skips the
throttled performance passes, mobile, security and tech-stack detection. Each page file records
that partial scope in `execution.scope.requested_modules` and `execution.module_runs`.

Flags you give explicitly win: `-f` replaces the format, and `--full`, `--performance`,
`--mobile` and `--security` add their modules as usual.

## Output

The output directory holds:

- one `<site>-<path>-<date>-single-report.json` per audited page — the regular single-page JSON,
- `index.json` — every attempted URL in input order with `file` (or `null`), `status` (`ok`,
  `blocked` for bot walls and access denials, `failed`), `reason`, `overall_score`,
  `accessibility_score`, `finding_count`, `occurrence_count` and `audit_quality`,
- `findings.jsonl` — one line per finding occurrence across all pages: `url`, `source` (`wcag`,
  `journey`, `seo`, `html_conform`), `rule_id`, `wcag_criterion`, `level`, `severity`,
  `selector`, `location`, `message`, `fix_suggestion`, `viewport_tags`. The page files keep a few
  example occurrences per finding; this list is complete.

Any `--per-page-reports -f json` run writes `index.json` and `findings.jsonl` as well. Failed or
blocked pages appear only in `index.json`, never as a page file. All text is canonical English.

Schemas in the repository:

- [docs/technician-index.schema.json](https://github.com/casoon/auditmysite/blob/main/docs/technician-index.schema.json) — `index.json`
- [docs/technician-finding.schema.json](https://github.com/casoon/auditmysite/blob/main/docs/technician-finding.schema.json) — one line of `findings.jsonl`

## Selecting pages by path

```sh
auditmysite --sitemap https://example.com/sitemap.xml --technician \
  --include-path '/blog/**' --exclude-path '/blog/tag/**' -m 50 -o reports/tech/
```

`--include-path` and `--exclude-path` are repeatable and work in every batch mode, not only with
`--technician`. They are applied before `--max-pages` (`-m`).

Globs are matched against the whole, percent-decoded URL path, without host and query:

- `*` and `?` stay within one path segment, `**` crosses segments, and `/**/` also matches a
  single `/`.
- `/blog/**` selects everything below `/blog/`, but not `/blog` itself.
- A URL is audited when it matches any `--include-path` (or none is given) and no
  `--exclude-path`.
- With `--crawl`, the filters apply to the discovered pages; discovery itself is still capped by
  `-m`.

## Working with the files

```sh
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
