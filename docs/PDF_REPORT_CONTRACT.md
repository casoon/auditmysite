# PDF Report Contract

This document defines the minimum quality contract for `auditmysite` PDF reports.
It complements `docs/OUTPUT_CONTRACT.md`, which covers JSON compatibility.

## Goals

PDF reports are customer-facing artifacts. They should help readers understand:

- what was audited,
- how severe the current state is,
- which issues matter most,
- what to fix next,
- which technical details developers need for implementation.

## Report Levels

### Executive

Audience: management, project leads, non-technical stakeholders.

Minimum content:

- branded cover with score, certificate, date, domain and active modules,
- concise risk/status summary,
- top findings without raw technical snapshots,
- business/user impact,
- prioritized next actions,
- methodology note.

Technical selectors, HTML snippets and code examples should not dominate this level.

### Standard

Audience: product owners, project leads, UX/design, developers.

Minimum content:

- everything from the executive report,
- action plan,
- module overview,
- prioritized finding cards,
- technical handoff section,
- representative findings and module details.

### Technical

Audience: developers and implementers.

Minimum content:

- full standard report,
- complete technical appendix where useful,
- selectors, affected elements, code examples and grouped finding patterns,
- per-URL detail tables for batch reports when available.

## Batch Reports

Minimum content:

- branded cover,
- audited domain/source and number of URLs,
- portfolio score and certificate,
- ranking or URL overview,
- recurring issue patterns,
- action plan,
- crawl/link diagnostics when the audit came from crawl mode.

Batch reports are not a stack of single-page reports. They should focus on
domain-wide patterns, repeated issues and prioritization.

## Comparison Reports

Minimum content:

- branded cover,
- compared domains and average score,
- domain ranking,
- module comparison when module scores are available,
- top findings by domain.

## Visual Quality Requirements

Every PDF should:

- render successfully to valid PDF bytes,
- have a recognizable first page when rasterized,
- include a brand/logo asset or the AuditMySite fallback wordmark,
- avoid internal/debug labels such as `Raw`, `CLI parity` or Rust debug output,
- keep executive pages free from dense technical implementation tables,
- use consistent score/status colors,
- keep tables and code examples readable without obvious clipping.

## Regression Checks

Current automated checks:

- single-report PDF smoke test renders a valid PDF,
- single-report Executive/Standard/Technical smoke test renders valid PDFs,
- batch PDF smoke test renders a valid PDF,
- comparison PDF smoke test renders a valid PDF,
- first Executive PDF page is rasterized to PNG with `pdftoppm` when available.

Future checks:

- rasterize representative pages for Standard, Technical, Batch and Comparison,
- assert page counts or minimum page ranges per report type,
- add golden-master screenshots for selected first pages,
- verify custom logo assets are visible in rendered output,
- run visual checks in CI with a pinned PDF rasterizer.
