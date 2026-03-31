# Test Reports

This directory contains manually generated audit reports for testing and validation purposes.

All files in this directory (except this README) are gitignored.

Single-page reports written into this directory now also generate convention-based history files:

- `<subject>-history.json` for machine-readable timelines
- `<subject>-history.md` for human-readable trend summaries

The history is built from all compatible JSON reports in this directory that belong to the same audited host. The more snapshots you keep here, the richer the timeline becomes.

## Generating Reports

```bash
# Single page audit
./target/release/auditmysite https://example.com --full --format pdf --output reports/example-audit.pdf

# Single page audit with JSON snapshot
./target/release/auditmysite https://example.com --full --format json --output reports/example-2026-03-31.json

# Batch audit via sitemap
./target/release/auditmysite --sitemap https://example.com/sitemap.xml --full --format pdf --output reports/example-batch-audit.pdf
```
