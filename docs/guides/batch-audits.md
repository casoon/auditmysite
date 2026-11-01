---
title: "Batch audits and browser selection"
description: "Sitemap, crawl and URL-file runs, per-page reports, and choosing the browser."
order: 8
---

## Batch audits

```bash
# explicit sitemap
auditmysite --sitemap https://example.com/sitemap.xml

# crawl from a base URL and discover same-domain pages automatically
auditmysite https://example.com --crawl --crawl-depth 2

# base URL: probe robots.txt / common sitemap locations first
auditmysite https://example.com

# prefer sitemap automatically if one is found
auditmysite https://example.com --prefer-sitemap

# suppress sitemap suggestion and stay on the single page
auditmysite https://example.com --no-sitemap-suggest

# URL file
auditmysite --url-file urls.txt

# per-page reports: scan a list/sitemap but write one PDF per URL instead of an aggregated batch report
auditmysite --url-file urls.txt --per-page-reports --output reports/per-page/
auditmysite --sitemap https://example.com/sitemap.xml --per-page-reports --output reports/per-page/
```

Pages run in parallel (`--concurrency`). A page waiting for a free browser page waits as long as
one page in flight may take (creating the page, four times `--timeout` with a two-minute
minimum, and the page reset), so heavy pages ahead of it no longer push it out. A page that
still gets no browser page in time is retried one at a time after the parallel phase. Pages that cannot be audited at all are not
part of any score: the report says how many URLs the scores cover (JSON:
`summary.url_count` of `summary.attempted_url_count`, failures under `errors`; PDF: cover and
status section).

## Browser selection

```bash
auditmysite --browser-path /path/to/chrome https://example.com
```
