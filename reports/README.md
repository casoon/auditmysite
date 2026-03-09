# Test Reports

This directory contains manually generated audit reports for testing and validation purposes.

All files in this directory (except this README) are gitignored.

## Generating Reports

```bash
# Single page audit
./target/release/auditmysite https://example.com --full --format html --output reports/example-audit.html

# Batch audit via sitemap
./target/release/auditmysite --sitemap https://example.com/sitemap.xml --full --format html --output reports/example-batch-audit.html
```
