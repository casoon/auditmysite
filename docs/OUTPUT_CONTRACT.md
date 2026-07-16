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

## Metrics

Canonical per-metric definitions live in `src/registry/metrics.rs`
(`REGISTRY`, #506): id, JSON path, unit, direction, scope, aggregation, and
whether the number needs an explanation. Each anchor below is the `docs_url`
target for one registry entry; `tests/registry_contract.rs` checks that every
`docs_url` resolves to an anchor here. The description text is the same
`meaning` string emitted in the JSON `metric_context` block.

<a id="accessibility-score"></a>
### accessibility_score — `summary.accessibility_score`

Canonical accessibility score. With dual-viewport data it is 70% mobile and
30% desktop; otherwise it is the measured single-viewport score. It covers
automatically evaluated WCAG findings only. In batch reports this is the
average of the canonical page accessibility scores.

<a id="overall-score"></a>
### overall_score — `summary.overall_score`

Weighted score across contributing measured modules. In a dual-viewport
audit, viewport module results are blended first and Security contributes
10% when available. In batch reports this is the average of the page-level
overall scores across the audited URLs.

<a id="overall-score-alias"></a>
### overall_score_alias — `summary.score`

Compatibility alias for `summary.overall_score`.

<a id="grade-certificate"></a>
### grade_certificate — `summary.grade` / `summary.certificate`

Classification derived from `summary.overall_score`; risk gates can restrict
the certificate without changing the numeric score.

<a id="module-score"></a>
### module_score — `pages[].module_scores[].score`

Module-specific 0-100 score. Scores from different modules use different
measured or heuristic inputs and are not interchangeable raw measurements.

<a id="module-dimension-score"></a>
### module_dimension_score — `pages[].detail.modules.*.score` and nested dimension scores

Module- or dimension-specific 0-100 score; higher is better. The surrounding
`module` and `measurement_type` identify whether the value is measured or
heuristic.

<a id="viewport-scores"></a>
### viewport_scores — `pages[].detail.viewport_scores`

Desktop and mobile accessibility/overall scores on the 0-100 scale.
`weighted_overall` is the 70% mobile and 30% desktop module blend before the
optional Security contribution; the canonical accessibility blend is recorded
in `score_breakdown`.

<a id="named-module-score"></a>
### named_module_score — `summary.*_score` / `pages[].*_score`

Named module score on the 0-100 scale; higher is better. Throttled and
Lighthouse performance scores are lab results, not field/RUM data.

<a id="accessibility-score-breakdown"></a>
### accessibility_score_breakdown — `summary.accessibility_score_breakdown[]`

Accessibility area score (0-100, higher is better), its contribution weight
in percent, and `estimated_lost_points` relative to 100.

<a id="risk-score"></a>
### risk_score — `pages[].risk.score`

Independent 0-100 risk index; unlike quality scores, higher means more risk.
The adjacent `level`, `threshold`, and `driven_by` fields explain its
classification.

<a id="principle-coverage-ratio"></a>
### principle_coverage_ratio — `pages[].principle_coverage.*.ratio`

Share from 0 to 1 of automatically evaluated checks without a finding for
that WCAG principle; it is coverage context and does not affect the score.

<a id="finding-priority-score"></a>
### finding_priority_score — `pages[].findings[].priority_score`

Unbounded ranking value calculated as severity weight multiplied by
occurrence reach and divided by estimated effort; higher values should be
addressed earlier and are not percentages.

<a id="finding-score-impact"></a>
### finding_score_impact — `pages[].findings[].score_impact`

Rule-specific base and maximum deductions used by accessibility scoring;
scaling states how repeated occurrences are condensed.

<a id="violation-occurrence-count"></a>
### violation_occurrence_count — `violation_count` / `occurrence_counts`

Affected WCAG element or page occurrences; repeated instances of the same
rule count separately.

<a id="violated-rule-severity-count"></a>
### violated_rule_severity_count — `violated_rule_count` / `severity_counts`

Distinct violated WCAG rule groups, not affected elements.

<a id="finding-occurrence-count-all-categories"></a>
### finding_occurrence_count_all_categories — `finding_count` / `finding_occurrence_count`

Finding rows and occurrences across WCAG, SEO, and every other reported
category.

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
