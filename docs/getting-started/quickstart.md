---
title: Quickstart
description: A first single-page audit, a JSON report for automation and a first batch run.
order: 2
---

## A first audit

```sh
auditmysite https://example.com
```

A single URL runs the full analysis set (accessibility plus performance, SEO, security, mobile
and the heuristic modules), prints a short summary and writes the report files into the current
directory:

```text
example-com-YYYY-MM-DD-single-report.pdf
example-com-YYYY-MM-DD-single-report.json
example-com-YYYY-MM-DD-single-report-screen-reader-audit.json
```

The PDF is in German by default; `--lang en` switches it to English. The JSON is always
English.

To see what a run would do without starting a browser:

```sh
auditmysite plan https://example.com
```

## Terminal summary

```sh
auditmysite https://example.com --format table
```

Prints the summary instead of writing a PDF. The screen-reader sidecar JSON is still written.

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

Without `-f`, batch inputs print a table; single URLs write a PDF.

Batch reports are aggregated across pages (averages, ranking, recurring issues), not a stack of
single-page reports. Next: [CLI modes](../../guides/cli-modes/) and
[Reports](../../guides/reports/).
