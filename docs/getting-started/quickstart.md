---
title: Quickstart
description: A first single-page audit, a JSON report for automation and a first batch run.
order: 2
---

## A first audit

```sh
auditmysite https://example.com
```

A single URL runs the full analysis set (accessibility plus performance, SEO, security and
mobile), prints the result as a table in the terminal and exits with the verdict. It writes no
file and asks no question.

## Keep a report

```sh
auditmysite https://example.com -o report.json          # JSON, the format follows the extension
auditmysite https://example.com -f pdf -o report.pdf    # PDF
```

Reports and the table are in English by default; `--lang de` switches the PDF and the table to
German. The JSON is always English. The screen-reader sidecar JSON
(`report-screen-reader-audit.json`) is written only with `--screen-reader-report`.

## JSON for CI

```sh
auditmysite https://example.com -f json -o report.json --quiet
```

The exit code carries the verdict: `0` pass, `1` warn, `2` fail, `3` for a technical error.
`--report-mode` exits `0` whenever the report was written. See
[CLI modes](../../guides/cli-modes/#exit-codes).

## A whole site

```sh
# first 20 URLs of a sitemap, summary in the terminal
auditmysite --sitemap https://example.com/sitemap.xml --max-pages 20

# a list of URLs, one per line, as an aggregated PDF report
auditmysite --url-file urls.txt -f pdf -o site-report.pdf
```

Without `-f` or an `-o` with a known extension, batch inputs print a table, like single URLs.

Batch reports are aggregated across pages (averages, ranking, recurring issues), not a stack of
single-page reports. Next: [CLI modes](../../guides/cli-modes/) and
[Reports](../../guides/reports/).
