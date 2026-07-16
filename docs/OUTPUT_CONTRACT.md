# Output Contract

`auditmysite` treats JSON output as a compatibility surface for automation.

## Stability Rules

- Single-report JSON is generated from `NormalizedReport` plus `metadata` and optional module payloads.
- Batch JSON is generated from normalized reports plus batch `summary`, optional `errors`, and deterministic `metadata`.
- Timestamps are data-derived, not generated at serialization time.
- New top-level fields may be added in a backward-compatible way.
- Existing required fields should not be removed or renamed without a versioned contract change.

## Module Classification

- Every `report.module_scores[]` entry includes `measurement_type`.
- `measurement_type = "measured"` means the score is based on direct audit data such as WCAG checks, CDP metrics, HTTP headers, or mobile viewport measurements.
- `measurement_type = "heuristic"` means the score is an indicator inferred from structural signals. These values are report guidance, not direct measurements.
- Heuristic top-level module payloads such as `source_quality`, `ai_visibility`, and `content_visibility` include `measurement_type = "heuristic"` when present.

## Score and Count Semantics

- Scores use a 0–100 scale; higher values are better. The top-level `metric_context` block describes the scale and the meaning of score and count fields for machine consumers.
- In a dual-viewport audit, `accessibility_score` is always the rounded blend of 70% mobile and 30% desktop accessibility. The same canonical value is used in the summary, page entry, Accessibility module, score breakdown, and PDF.
- The merged cross-viewport finding list is evidence for prioritization and remediation. Its size does not create a third accessibility score.
- `overall_score`, `grade`, and `certificate` describe the weighted result across the active contributing modules. The optional risk gate may limit an otherwise positive certificate.
- `violation_count` and `occurrence_counts` count WCAG occurrences: affected elements or instances, not distinct rules.
- `violated_rule_count` and `severity_counts` count distinct grouped WCAG findings.
- `finding_count` and `finding_occurrence_count` include findings from all reported categories, such as WCAG and SEO.

## Execution Scope and Audit Quality

The report distinguishes what was requested from what was successfully measured:

- `audit_scope` records requested modules, interaction mode and budget, viewports, throttle profiles, consent handling, and evidence capture.
- `execution_environment` records public browser/runtime context and whether data came from a live run or reusable artifacts. Executable paths and browser launch arguments are not exposed.
- `pages[].detail.module_runs` and `pages[].detail.rule_outcomes` use explicit execution states such as `completed`, `partial`, `failed`, `skipped`, and `not_applicable`.
- `audit_quality` summarizes whether results are complete, partial, or insufficient. A qualified result remains reportable but cannot silently become an unqualified clean verdict.
- `pages[].navigation` and `pages[].consent` describe the requested/final URL, main-document status, redirect count, bounded page-stability provenance, and whether consent was not detected, detected, dismissed, failed, or remained unknown. Consent evidence never includes cookie values.

Confirmed violations remain separate from `accessibility_assessments`, which carries warnings, manual-review items, and positive signals. Measurement failures are not converted into violations or artificial score penalties.

Binary evidence is not embedded in the JSON. The top-level `artifacts` array points to separately written sidecars when available.

## Performance Measurement Context

- All performance metrics are **lab data** captured in a local headless Chrome (via CDP / `PerformanceObserver`). They are never field/RUM data (e.g. Chrome UX Report / CrUX).
- Each Core Web Vital (`detail.modules.performance.vitals.*`) carries a `measurement` field:
  - `measurement = "lab_headless"` — directly measured in the local headless browser (LCP, FCP, CLS, TBT, TTFB).
  - `measurement = "estimated_lab"` — derived/estimated from other lab signals (INP, TTI, Speed Index). These are approximations, not direct measurements, and must not be read as field metrics.
- In rendered reports, estimated metrics carry a localized "(lab estimate)" / "(Lab-Schätzung)" suffix, and the performance section states that all values are lab data, not field data.

## Batch Sample Metadata

- Batch reports may include a top-level `sample` block describing how the audited URL set was discovered and sampled:
  - `source` — origin of candidate URLs: `"sitemap"`, `"crawl"` or `"url_file"`.
  - `total_discovered` — candidate URLs found before any limit was applied.
  - `audited` — URLs actually audited.
  - `sample_limit` — the `--max-pages` cap, when one applied (omitted otherwise).
  - `selection` — `"first_n"` (discovery order) or `"all"`.
  - `is_sample` — `true` when fewer URLs were audited than discovered.
- The block is omitted when no sampling metadata was recorded (e.g. cached single reports).

## Batch Site Analysis

Batch reports expose a canonical `site_analysis` block for domain-wide results:

- module averages and active modules,
- navigation, heading, canonical, orphan-page, and schema-graph consistency,
- page-type and content-quality distribution,
- top topics and overlapping page pairs,
- exact and near-duplicate content,
- performance-budget and render-blocking rollups,
- structured-data type distribution and entity conflicts,
- recurring structured-data blockers and visible-content parity mismatches,
- page-type/schema combinations and Organization/WebSite identity consistency,
- batch-only WCAG assessments for consistent navigation, consistent identification, consistent help, and multiple ways,
- interactive coverage and grouped accessibility assessments.

Structured-data detail separates parse state, eligibility blockers, recommendations, manual checks, page-type fit, and visible-content parity. Ambiguous visible facts are not converted into mismatches; evidence samples are short and never contain full page text or DOM dumps.

Batch `pages[]` entries remain compact and carry profile/prioritization data rather than complete duplicate single-page report trees.

## Schemas

- Single report: [json-report.schema.json](json-report.schema.json)
- Batch report: [json-batch-report.schema.json](json-batch-report.schema.json)

These schemas are validated in automated tests and in the local release check.

## Change Policy

When changing JSON output:

1. Update the Rust serializer.
2. Update the matching schema file.
3. Update or extend tests.
4. Run `./scripts/release-check.sh`.
5. Verify that every score and scoped count matches the PDF presentation.
