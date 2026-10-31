# Output Contract

`auditmysite` treats JSON output as a compatibility surface for automation.

## Stability Rules

- Single-report JSON is generated from `NormalizedReport` plus `metadata` and optional module payloads.
- Batch JSON is generated from normalized reports plus batch `summary`, optional `errors`, and deterministic `metadata`.
- Timestamps are data-derived, not generated at serialization time.
- New top-level fields may be added in a backward-compatible way.
- Existing required fields should not be removed or renamed without a versioned contract change.

## Module Classification

- Every `report.module_scores[]` entry includes `measurement_type` and `band`.
- `band` is the canonical English qualitative label for `score`; it is present on every entry.
- `grade` (a letter) is present **only** for modules with `weight_pct > 0`. A module that does not feed `overall_score` carries `band` alone: grading an indicator on the same A–F scale as a weighted module gave it an authority it does not have.
- `derived_from` lists the modules an entry re-reads instead of measuring anything new; it is absent when the module measures its own subject. `UX` and `Journey` are charged an explicit accessibility penalty, `AI Visibility` re-reads SEO signals, and `Source Quality` re-reads SEO, Security and UX. Agreement between such a module and its sources is not independent confirmation.
- The same rule applies to the heuristic module payloads: `source_quality` and `ai_visibility` carry `band`, not `grade`, and `dark_mode` carries `band`.
- `measurement_type = "measured"` means the score is based on direct audit data such as WCAG checks, CDP metrics, HTTP headers, or mobile viewport measurements.
- `measurement_type = "heuristic"` means the score is an indicator inferred from structural signals. These values are report guidance, not direct measurements.
- `measurement_type = "optional"` means the module scores an optional, non-normative product feature (currently: Dark Mode) — a low score reflects the absence of that feature, not a compliance deficiency. It does not contribute to `overall_score`.
- `source_quality.score` combines the `consistency` and `authority` dimensions only. `source_quality.substance` holds structural indicators (word count, headings, structured data, metadata); its `score` is deprecated, is not part of `source_quality.score`, and is removed in 2.0.
- Heuristic top-level module payloads such as `source_quality`, `ai_visibility`, and `content_visibility` include `measurement_type = "heuristic"` when present. The `dark_mode` module payload includes `measurement_type = "optional"` when present.

## Score and Count Semantics

- A 0–100 value is only reported where its denominator is a fixed quantity. Where the value would otherwise be "share of however many checks exist", the report gives counts instead: `content_visibility` carries `signal_count`/`problem_count` and no score, and each `seo.content_profile.signal_strength.categories[]` entry carries `passed`/`total` rather than a percentage.
- `Content Visibility` is not a `module_scores[]` entry. It is an indicator reported through its counts and its signal list.
- Scores use a 0–100 scale; higher values are better. The top-level `metric_context` block describes the scale and the meaning of score and count fields for machine consumers.
- In a dual-viewport audit, `accessibility_score` is always the rounded blend of 70% mobile and 30% desktop accessibility. The same canonical value is used in the summary, page entry, Accessibility module, score breakdown, and PDF.
- The merged cross-viewport finding list is evidence for prioritization and remediation. Its size does not create a third accessibility score.
- `grade` and `certificate` describe `accessibility_score`, the subject of the report, and are the headline classification wherever one is shown (cover, batch cover, terminal). The optional risk gate may limit an otherwise positive certificate.
- `overall_score` is a secondary, explicitly labelled figure: the weighted result across the active contributing modules, reported together with `overall_score_weight_basis`. It is not the headline and carries no classification of its own.
- `violation_count` and `occurrence_counts` count WCAG occurrences: affected elements or instances, not distinct rules.
- `violated_rule_count` and `severity_counts` count distinct grouped WCAG findings.
- `finding_count` and `finding_occurrence_count` include findings from all reported domains (WCAG, SEO, security, mobile, …).
- There is no unqualified `score` in the summary: a score is either `accessibility_score` or `overall_score`.

## Finding Contract

`pages[].findings[]` holds every problem any module collected, one finding per rule (WCAG) or per collector check (`<source>.<check>`, e.g. `seo.technical.noindex`). `category` is `wcag` for accessibility rule findings and the domain name otherwise. Each finding carries `contract`:

- `domain`, `source`, `check` — where the finding comes from.
- `outcome` — `fail`, or `review` for heuristic and context-dependent checks.
- `applicability` — `applies`, or `unknown` when it depends on a page goal nobody declared; `applicability_basis` names the basis (e.g. `indexability:sitemap`).
- `cause` — canonical cause key. Findings of different domains with the same cause describe one defect (a missing `lang` is a WCAG, an SEO technical and an SEO meta finding).
- `scoring.owner` and `scoring.effect` — which score is responsible and how: `deduction` (weighted module score), `gate` (technical blocker: caps the owner's score and is a decision reason), `indicator` (unweighted indicator only), `risk` (risk assessment), `reference` (the same cause is charged elsewhere), `none`. One score charges a cause at most once.
- `scope` — `element`, `page`, `resource`, `site` or `interaction`.

`pages[].finding_inventory[]` maps every collected non-WCAG problem to its finding (`disposition.kind: finding`) or to an explicit exclusion (`excluded`, with `reason` `not_applicable` or `duplicate_of_wcag_finding`); WCAG rule runs are accounted in `rule_outcomes`. A report whose findings and inventory disagree fails validation and is not written.

`severity_counts`, `occurrence_counts`, `violation_count` and `violated_rule_count` stay WCAG-only.

### Finding assurance

`confidence`, `false_positive_risk` and `verification` follow from `assurance`, never from a rule id or category name:

- `assurance.method` — `measured` (the check measures the property the criterion names), `pattern` (it detects a pattern that often, not always, means the failure; the WCAG rules declared in `PATTERN_RULES`), or `review` (`contract.outcome: review` or `applicability: unknown`; UX and Journey findings are always `review`).
- `assurance.evidence` — `located` (every occurrence names its element or carries recorded evidence), `partial`, `document` (a document-wide check without an element) or `unlocated`.
- `verification: automatically_confirmed` requires `measured` with `located`, `partial` or `document` evidence (`confidence` `very_high` for located, `high` otherwise). Everything else is `manual_review_recommended` at `medium` confidence, and a finding that needs review raises no legal flag.

### Signal contracts

Every module payload with `measurement_type = "heuristic"` (`ux`, `journey`, `source_quality`, `ai_visibility`, `content_visibility`, `design_quality`) carries `signal_contract: { format: "auditmysite.signal-contract/1", contract }`. `contract` states `target_statement`, `method`, `applicability`, `evidence`, `uncertainty`, `permitted_interpretation`, the claims it does not establish (`not_established`) and `validation.status` — `unvalidated` until a pre-registered calibration run passes (`docs/calibration.md`). A score of an unvalidated contract is not evidence of visibility, AI citation, user quality or source truth.

### Indexability

`noindex` is judged against the page goal. `--expect-indexable true` (or `[audit] expect_indexable = true`, implied for sitemap runs) makes it a gate: the SEO score is capped at 39 and the decision gets the reason `technical_blockers`. `false` excludes it as intended. Without either, the finding is listed with `applicability: unknown` and has no score effect.

## Decision Contract

`decision` (top level and on every page) is the one PASS/WARN/FAIL of the report; the terminal, JSON, summary feed, Studio, PDF, the batch `passed` count and the exit code all project it. It carries `contract` (`auditmysite.decision/1`), the `profile` it was computed with (`name` `default` or `custom`, from `[verdict]`), `verdict` and `reasons`.

- **Fail** when the profile enables it and there are legal flags, blocking issues, technical blockers (`fail_on_technical_blockers`, default off) or the accessibility score is below `fail_below_score`.
- **Warn** for an incomplete run, technical blockers, an accessibility score below `warn_below_score` (default 70), or, above it, any WCAG finding.
- Thresholds compare `accessibility_score`. `overall_score` never decides.
- A stored decision that its own profile does not reproduce is a contract violation (`decision`), reported with every other violation found.

Every path that publishes a decision reads the stored one and validates before writing:

| Path | Reads | Checked by |
| --- | --- | --- |
| Single JSON (`--format json`, PDF sidecar) | `NormalizedReport.decision` → `decision`, `pages[].decision` | `validate`, `report-lint` |
| Batch JSON | `BatchSummary.decision`, per-page decisions | `validate_batch` |
| Summary feed (`--format summary`) | `verdict`, `verdictReasons` | `validate` |
| SARIF | findings only; refuses invalid reports | `validate` |
| Terminal (single, batch) | stored decision | `validate` / `validate_batch` before rendering (CLI) |
| Studio response and history | `decision` | — (projection only) |
| PDF / Typst input | `ReportViewModel.summary.ci_verdict` from the stored decision | `validate` / `validate_batch` |
| Cache replay | the cached report re-judged under the current profile, stored in `run_context` | `validate` |
| Exit code | stored decision (`0` pass, `1` warn, `2` fail) | — |

Batch: a page counts as passed when its own decision passes; the batch `decision` applies the same profile to the summary (legal flags, blocking issues, technical blockers and the average accessibility score). All pages of a batch share one profile.


## Execution Scope and Audit Quality

The report distinguishes what was requested from what was successfully measured:

- `audit_scope` records requested modules, interaction mode and budget, viewports, throttle profiles, consent handling, and evidence capture.
- `execution_environment` records public browser/runtime context and whether data came from a live run or reusable artifacts. Executable paths and browser launch arguments are not exposed.
- `pages[].detail.module_runs` uses explicit execution states: `completed`, `partial`, `failed`, `skipped`, and `not_applicable`.
- `pages[].detail.rule_outcomes` records one entry per rule and viewport, keyed by `(rule_id, viewport)`. An entry that ran carries no `not_run` field and reports `findings`; an entry that did not run carries `not_run` with one of `capability_missing`, `disabled`, `not_applicable`, or `errored`, plus a free-text `reason`. Each entry also lists the `wcag` criteria that stand or fall with it, so the report can name which criterion went unchecked. A rule that could not run is never reported as passing.
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

## Measurement Profile

- `audit_scope.requested_rules` lists every rule the run must account for — shared, AX-tree and page rules (executor and concrete ids) and the inline rules (reflow, HTML content model; mobile only) — with the viewports it runs in, derived from the rule registrations, the `wcag_level` and the `[rules]` filter. Each listed `(rule_id, viewport)` needs exactly one `rule_outcomes` entry; a missing one fails validation as `requested_rule_outcome`, a second one as `rule_outcome_duplicate`, and a rule that ran above `audit_scope.wcag_level` as `rule_outcome_above_level`. Frames add no outcomes of their own: their results fold into the page's entry for the rule. Error and capability counters count a page-rule executor and its concrete rules once.
- `audit_scope.measurement_profile` states how the pages were measured: `version` (meaning of the profile), `session_policy` (`isolated` or `shared`; `unknown` for reports from before the field), `read_only`, and `state_changing_journeys` (`form_error_submit`, `add_to_cart`, `quantity_stepper`; listed only with `--interactive full` and `--allow-state-changes`).
- `state_id` and `storage_state` (`sha256`, `cookies`, `origins`) name a declared browser state (`--storage-state`, `--state-id`). Values never enter the report. A different state file is a different `sha256` and so another measurement.
- A result measured under another profile is another measurement. Validation rejects a profile whose `read_only` or journey list contradicts `interactive_mode`.

## Snapshot, Replay and Validation Paths

The cache entry's `snapshot.json` is the collected evidence, independent of any presentation: the AX tree and raw module captures, the whole collected report (modules and subchecks, rule outcomes, execution states, `audit_scope` with its `measurement_profile`, the run context; screenshots excluded), the document identity (`requested_url`, `final_url`, `status_code`, `redirect_count`), the run language and `versions`.

- `versions` records the snapshot schema, the content fingerprint, the collection contract, the measurement-profile version, the evaluation version (normalization, scoring, decision derivation), the decision contract and a digest of the shared rule catalogue. A snapshot is replayed only when all of them equal the current values.
- `audit::replay(snapshot)` evaluates a snapshot without browser or network and returns the same `NormalizedReport` as the fresh run. Only `timestamp`, `duration_ms` and `execution.environment.source` (`replay`) may differ. `output::format_json_replay` exports it with `detail.module_detail: full`.
- A snapshot that lacks parts (`ReplayError::Incomplete`, naming them) or was written under other versions (`ReplayError::Incompatible`, naming field, stored and current value) is never evaluated under today's semantics. `load_artifacts` treats it as a cache miss; `lookup_artifacts` and `cached_report` return the reason (`CacheMiss`: absent, unreadable, other configuration, not replayable), which the CLI prints.
- `--reuse-cache` answers through `replay_report_as(snapshot, "cache")`, `auditmysite replay <snapshot|directory|URL>` through `replay_report`. `SnapshotArtifact::check_measurement(&ExpectedProfile)` refuses a snapshot whose session policy, state id or storage-state digest differs from the asserted one (`ReplayError::OtherMeasurement`); `audit::profile_differences` names the differences between two profiles. A replay measures nothing, so the extra-check flags are refused for it (support matrix, mode `Replay`).
- `format_json_cached` works on a stored `NormalizedReport` alone and says so with `module_detail: not_restored`; an empty module block there never stands for restored data.
- Bump `SNAPSHOT_SCHEMA_VERSION` when the shape of the collected report changes in a way old data would survive deserialization, and `EVALUATION_VERSION` when a change moves a score, finding or verdict for unchanged input.

Every fallible hand-over (live pipeline, `normalize_checked`, `validate_batch`, JSON single/cached/replay/batch, summary, SARIF, PDF source) validates the domain data first; `tests/validation_path_matrix.rs` refuses contradictory data at each of them and keeps the CLI dispatchers and the cache-hit path on the validating route. `audit::normalize`, `output::format_ai_json`, the terminal renderers and `technician::finding_rows` are infallible compatibility APIs: they hold no check of their own and are called only on data that came through a checked entry. A new unchecked `normalize()` call in `cli`, `output`, `studio` or `lib.rs` fails the test unless the baseline in `tests/fixtures/architecture/unchecked_normalize.txt` is changed on purpose. The 2.0 migration path for callers outside the crate is `normalize_checked` (identical model for valid data, `ReportContractError` with codes and data paths otherwise).

## Batch Sample Metadata

- Batch reports may include a top-level `sample` block describing how the audited URL set was discovered and sampled:
  - `source` — origin of candidate URLs: `"sitemap"`, `"crawl"` or `"url_file"`.
  - `total_discovered` — candidate URLs found before any limit was applied.
  - `audited` — URLs actually audited.
  - `sample_limit` — the `--max-pages` cap, when one applied (omitted otherwise).
  - `selection` — `"first_n"` (discovery order), `"all"`, or `"path_filter"` (narrowed by `--include-path`/`--exclude-path`, then capped by `sample_limit` in discovery order), or `"stratified"` (`--sample-strategy stratified`: after any path filter, a deterministic round-robin draw across structural strata, at most `sample_limit` URLs).
  - `strata` — only with `"stratified"`: per stratum (`[lang/]section[.ext]`, e.g. `de/blog`, `shop`, `/`, `docs.pdf`) the candidates (`discovered`) and the picks (`selected`; 0 when the budget did not reach the stratum). The strata come from the URL structure alone, not from page type or process state; a stratified sample is still only a sample.
  - `is_sample` — `true` when fewer URLs were audited than discovered.
- The block is omitted when no sampling metadata was recorded (e.g. cached single reports).
- A selection (`first_n`, `path_filter`, `stratified`) is no evidence of representativeness. Batch statements hold for the successfully audited pages (`summary.coverage.succeeded`); failed and blocked URLs are counted in `summary.coverage`, never in the averages. URLs selected but never started (`coverage.selected > coverage.attempted`) make the batch incomplete (`not_attempted_urls` in `audit_quality.reasons`, `incomplete_scope` in the decision).
- `summary.score_spread` reports `min`, `median`, `max`, `std_dev` and the `worst_pages` (up to 5, worst first) next to `average_score`.
- `summary.selection_profile` counts the audited pages by canonical `page_types` and primary-subtag `languages`; pages without the signal are counted in `unclassified` / `language_unknown`.

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
- batch-only WCAG assessments for consistent navigation, consistent identification, consistent help, and multiple ways (`consistency.wcag_cross_page`), with the help-mechanism comparison behind 3.2.6 in `consistency.help`,
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
average of the canonical page accessibility scores over the URLs that produced a report (`summary.coverage.succeeded`, equal to `summary.url_count`). A batch in which no URL produced a report has no average: `summary.accessibility_score` and `summary.overall_score` are `null`, never `0`, and the decision is `fail` (`no_url_audited`).

<a id="overall-score"></a>
### overall_score — `summary.overall_score`

Weighted score across contributing measured modules. In a dual-viewport
audit, viewport module results are blended first and Security contributes
10% when available. In batch reports this is the average of the page-level
overall scores across the audited URLs.

<a id="grade-certificate"></a>
### grade_certificate — `summary.grade` / `summary.certificate`

Classification derived from `summary.accessibility_score` — the subject of the
report; risk gates can restrict the certificate without changing the numeric
score. Both read the same number, so they cannot contradict each other.

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

<a id="subcategory-scores"></a>
### subcategory_scores — `pages[].detail.accessibility_subcategory_scores` / `pages[].detail.security_category_scores`

Accessibility score per subcategory and Security score per category, each on
the 0-100 scale; higher is better. Each entry is `{ name, score }` with a
canonical English `name`. Accessibility entries re-run the module's own scorer
on that subcategory's violations, so a subcategory without violations scores
100. Security entries are omitted when the Security module did not run. Single
reports only; batch page details omit both fields.

<a id="named-module-score"></a>
### named_module_score — `summary.*_score` / `pages[].*_score`

Named module score on the 0-100 scale; higher is better. Throttled and
Lighthouse performance scores are lab results, not field/RUM data.

<a id="overall-score-weight-basis"></a>
### overall_score_weight_basis — `summary.overall_score_weight_basis`

Sum of the module weights that actually fed `overall_score`, out of 100.
Below 100 the score was renormalised over the modules that ran and measured
successfully — a module that did not run, or that ran without being able to
measure, changes the denominator rather than the result. A score computed on a
different weight basis is not directly comparable.

In a batch report the value is the **lowest** basis across the audited pages,
so it never overstates how much of the model backs the average, and
`overall_score_weight_basis_uniform` says whether the pages agreed. `false`
means `overall_score` values from different module sets were averaged.

<a id="accessibility-score-breakdown"></a>
### accessibility_score_breakdown — `summary.accessibility_score_breakdown[]`

Two different statements per area, deliberately kept apart:

- `score` (0-100, higher is better) is that area run through the accessibility
  scorer *on its own* — what the page would score if this area were its only
  problem. Area scores do not add up to `summary.accessibility_score`, and no
  weighting of them reproduces it: the scorer's diversity factor, compression
  curve, soft floor and severity cap are all non-linear and depend on the
  finding set as a whole.
- `estimated_lost_points` is the area's share of the points the score actually
  lost. Across all areas it sums to exactly
  `100 - summary.accessibility_score`. `weight_pct` is the same share expressed
  in percent (summing to 100 whenever anything was lost).

So a low `score` with few `estimated_lost_points` is meaningful, not a
contradiction: the area is bad in isolation but accounts for little of this
page's total loss.

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

## JavaScript Library Advisories

`best_practices.vulnerable_libraries.vulnerable[]` lists detected libraries whose version lies in an advisory range. The ranges come from the versioned data file `src/best_practices/advisories.json` (process: [advisory-data.md](advisory-data.md)); the check covers only the libraries in `CHECKED_LIBRARIES`.

- `advisories[].review_status` is `reviewed` (the range was checked against `source_url` on the date in `advisories[].reviewed`) or `unreviewed` (carried over, not yet checked). An unreviewed match is a lead, not an established statement; the PDF says so next to the finding. Reports written before the field read as `unreviewed`.
- `exploitability` is `unconfirmed` (the version is in range; reachability on the page was not examined) or `not_applicable` (only end-of-maintenance notices matched, so no specific vulnerability exists to exploit). There is no `confirmed` state: a version never proves the code path is reachable, and the check runs no reachability probe.
- `detected[].version_parsed: false` marks a version that could not be read; it is neither flagged nor cleared.
- The check runs for `--performance` and for `--security` on its own.

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
