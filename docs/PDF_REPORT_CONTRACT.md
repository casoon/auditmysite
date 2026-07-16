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

Target page count: 5–8 pages (8 pages maximum with a realistic finding load).

Minimum content:

- branded cover with score, certificate, date, domain and active modules,
- concise risk/status summary,
- top findings without raw technical snapshots,
- business/user impact,
- prioritized next actions,
- methodology note.

Technical selectors, HTML snippets and code examples must not dominate this level.
Dense implementation tables, raw selector lists and grouped code examples are not
appropriate at the Executive level.

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

## Module Classification: Measured vs. Heuristic

Modules are divided into two classes:

**Measured** — score derived from quantitative, reproducible signals (CDP data, DOM counts,
network timings, WCAG rule checks). Examples: Accessibility, Performance, Security.

**Heuristic indicators** — score derived from structural signals and inferred patterns.
These are best-effort estimates, not definitive values. Examples: UX, User Journey,
Source Quality, AI Visibility and Content Visibility.

In the rendered PDF:

- Heuristic score cards and module overview entries carry the "(Indicator)" / "(Indikator)" suffix in their title.
- Heuristic score cards carry the description "Heuristic estimate based on structural
  signals" in the subtitle.
- Indicator modules may be displayed in the module dashboard, but do not contribute to
  the weighted overall score unless explicitly marked as contributing in the JSON model.
- No visual distinction between measured and heuristic modules in the module overview strip —
  the indicator suffix is the only differentiator.

## Numeric Context and Cross-Format Consistency

- Specialized scores must state their scale or denominator where they appear, for example `20 / 100`; nearby copy must explain whether higher or lower values are better.
- Technical metrics must include a unit and, where interpretation is not self-evident, a target range or short definition. Ordinary counts and familiar quantities such as dates, durations, file sizes, and page totals do not need repeated explanations.
- Dual-viewport accessibility uses one canonical formula everywhere: 70% mobile plus 30% desktop, rounded to an integer. The merged finding inventory remains evidence and does not produce another score.
- Labels must distinguish distinct WCAG finding groups, WCAG occurrences, and findings from all categories. A number must not be labelled “critical” when it combines critical and high severities.
- Single and batch PDF scores, grades, certificates, and scoped counts must match the corresponding JSON fields.

## Language

- CLI output (terminal table, progress messages) is always English.
- PDF reports are localized via `--lang de|en`. Default is German.
- All user-visible labels in PDF output must use Fluent (`.ftl`) keys.
  Hard-coded German strings in PDF renderers are a contract violation.

### Localization Boundaries

AuditMySite currently has three localization surfaces. Keep new text in the
surface that owns its lifecycle:

- `locales/{de,en}/report.ftl`: runtime report labels, headings, table columns,
  callout titles, status labels, and short reusable PDF/JSON presentation text.
  New PDF renderer text should start here.
- `src/output/explanations.rs`: WCAG rule explanations, examples, user impact,
  causes, fix recommendations, owner role, and effort. These are rule catalog
  facts rather than layout labels. Do not add generic report UI copy here.
- `src/audit/interpretation.rs`: computed interpretation sentences that depend
  on normalized audit signals and must be available before the output layer is
  built. Prefer Fluent keys in the interpretation model when the text is a fixed
  report sentence; use `LocalizedText` only when the sentence is derived from
  audit data and stored with the normalized report.

When adding another language, Fluent bundles are the primary expansion point.
Any remaining paired `*_en` fields or `LocalizedText { de, en }` values are
known migration seams from issue #417 and should not grow without a specific
reason.

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
