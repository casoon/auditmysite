---
title: Documentation
description: auditmysite audits rendered web pages against WCAG 2.2 AA through Chrome's accessibility tree and writes JSON and PDF reports. Start with the installation, then pick the mode that fits your task.
---

## Where to start

- [Installation](getting-started/installation/) — installer script, crates.io, release binaries, building from source.
- [Quickstart](getting-started/quickstart/) — a first audit, a JSON report for CI, a first batch run.

## Guides

- [CLI modes](guides/cli-modes/) — single URL, sitemap, URL file, crawl, per-page reports, exit codes.
- [Reports](guides/reports/) — the JSON contract, the PDF report, languages, `report-lint`.
- [Browser setup](guides/browser-setup/) — which Chrome is used and how to install one.
- [Troubleshooting](guides/troubleshooting/) — common errors and what to do about them.

## Concepts and reference

- [Architecture](concepts/architecture/) — the audit pipeline and the shared barrierlab libraries.
- [Reference](reference/api/) — library API on docs.rs, JSON schemas, contracts in the repository.

## Release notes

Each release has notes on [GitHub Releases](https://github.com/casoon/auditmysite/releases).
The repository's [CHANGELOG.md](https://github.com/casoon/auditmysite/blob/main/CHANGELOG.md)
is a detailed, chronological development log (in German): each entry records a finding, the fix
and how it was verified.
