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
- The same rule applies to the heuristic module payloads in the `experimental` block: `source_quality` and `ai_visibility` carry `band`, not `grade`.
- `measurement_type = "measured"` means the score is based on direct audit data such as WCAG checks, CDP metrics, HTTP headers, or mobile viewport measurements.
- `measurement_type = "heuristic"` means the score is an indicator inferred from structural signals. These values are report guidance, not direct measurements.
- `source_quality.score` combines the `consistency` and `authority` dimensions only. `source_quality.substance` holds structural indicators (word count, headings, structured data, metadata); its `score` is deprecated, is not part of `source_quality.score`, and is removed in 2.0.
- The heuristic payloads in `pages[].experimental` (`ux`, `journey`, `source_quality`, `ai_visibility`, `content_visibility`) include `measurement_type = "heuristic"`; `commerce` is a checklist without a score and carries neither.

## Score and Count Semantics

- A 0–100 value is only reported where its denominator is a fixed quantity. Where the value would otherwise be "share of however many checks exist", the report gives counts instead: `content_visibility` carries `signal_count`/`problem_count` and no score, and each `seo.content_profile.signal_strength.categories[]` entry carries `passed`/`total` rather than a percentage.
- `Content Visibility` is not a `module_scores[]` entry. It is an indicator reported through its counts and its signal list.
- Scores use a 0–100 scale; higher values are better. The top-level `metric_context` block describes the scale and the meaning of score and count fields for machine consumers.
- In a dual-viewport audit, `accessibility_score` is always the rounded blend of 70% mobile and 30% desktop accessibility. The same canonical value is used in the summary, page entry, Accessibility module, score breakdown, and PDF.
- The merged cross-viewport finding list is evidence for prioritization and remediation. Its size does not create a third accessibility score.
- `grade` describes `accessibility_score`, the subject of the report, and is the headline classification wherever one is shown (cover, batch cover, terminal). There is no certificate: an automated check covering part of the criteria issues none, so the JSON carries no `certificate` field anywhere (removed from `decision` in `auditmysite.decision/3`).
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

### Rule documentation links

`findings[].help_url` is the finding's page on the project site, `https://casoon.github.io/auditmysite/rules/<page_id>/` (`taxonomy::rule_doc_url`): the page id verbatim, `/` as path separator, a trailing slash. A page id contains only lowercase letters, digits, `-`, `_`, `.` and `/`, and is always an id the finding carries (`audit::rule_docs`):

- WCAG findings: the `rule_id` (`images/alt-missing`).
- Security header findings: the `rule_id` (`security.headers.<issue_type>`). The headers that share an issue type (`missing_header`) are one finding and one page, which lists each header's check.
- Vulnerable-library findings: the `rule_id` (`best_practices.vulnerable_libraries.<name>`).
- Structured-data findings about missing required properties: `seo.schema.schema_rule_missing_<feature>_<property>` carries the feature key, and the page is the feature's, `seo.schema.schema_rule_missing_<feature>`.

Every other finding (SEO, mobile, UX, the other structured-data issues, ...) has no page and no `help_url`. `findings[].understanding_url` is a WCAG criterion's W3C Understanding page; it is absent when the rule names none, and always absent outside WCAG. Before #929 `help_url` held the Understanding page. SARIF `helpUri` and the AI report's `tasks[].help_url` follow `help_url`; the AI report adds `tasks[].understanding_url`. The PDF still prints the Understanding page as the reference. `tests/rule_docs.rs` keeps the generated pages in `site/src/data/rules/` equal to the rule catalogue, the contracts and the corpus, and every `help_url` pointing at one of them. `EVALUATION_VERSION` 8 and cache `fmt` 33 turn stored snapshots and cache entries into misses; `schema_version` stays `2.0` (a field is added, none removed).

### Finding assurance

`confidence`, `false_positive_risk` and `verification` follow from `assurance`, never from a rule id or category name:

- `assurance.method` — `measured` (the check measures the property the criterion names), `pattern` (it detects a pattern that often, not always, means the failure; the WCAG rules declared in `PATTERN_RULES`), or `review` (`contract.outcome: review` or `applicability: unknown`; UX and Journey findings are always `review`).
- `assurance.evidence` — `located` (every occurrence names its element or carries recorded evidence), `partial`, `document` (a document-wide check without an element) or `unlocated`.
- `verification: automatically_confirmed` requires `measured` with `located`, `partial` or `document` evidence (`confidence` `very_high` for located, `high` otherwise). Everything else is `manual_review_recommended` at `medium` confidence, and a finding that needs review raises no legal flag.

### Signal contracts

Every scored payload in `pages[].experimental` (`ux`, `journey`, `source_quality`, `ai_visibility`, `content_visibility`) carries `signal_contract: { format: "auditmysite.signal-contract/1", contract }`. `contract` states `target_statement`, `method`, `applicability`, `evidence`, `uncertainty`, `permitted_interpretation`, the claims it does not establish (`not_established`) and `validation.status` — `unvalidated` until a pre-registered calibration run passes (`docs/calibration.md`). A score of an unvalidated contract is not evidence of visibility, AI citation, user quality or source truth.

### Experimental indicators

UX, Journey, AI Visibility, Content Visibility, Source Quality and Commerce are experimental (#916): `unvalidated` signal contracts, no calibration set, no weight in any score. They run only with `--experimental <name>[,<name>]` and are in no `--full` run.

- A single report carries them in `pages[].experimental`, one key per indicator that ran (`ux`, `journey`, `ai_visibility`, `content_visibility`, `source_quality`, `commerce`), each scored one with its `signal_contract` (`commerce` is a score-less checklist and has none). The block is absent when none ran. A batch report carries only `summary.experimental.commerce`, the site-wide roll-up of the commerce trust pages.
- They are in no `module_scores[]` entry, no `findings[]`, `finding_inventory`, `risk`, `decision` or PDF chapter, and `detail.modules` has no key for them. `ux_score`, `journey_score` (page and summary) and `summary.commerce` of 2.0.x are gone.
- An indicator another one is derived from runs with it and appears in the block: `content_visibility` brings `source_quality` and `ai_visibility`, `source_quality` brings `ux`.
- The add-to-cart and quantity-stepper journeys belong to `commerce`: without it `measurement_profile.state_changing_journeys` does not list them. `--check-ssr-content` needs `content_visibility`.
- Versions: `EVALUATION_VERSION` 7, `SNAPSHOT_SCHEMA_VERSION` 2 and the cache signature (`fmt` 32, now with the indicator selection) turn stored snapshots into cache misses. `decision/3` is unchanged: none of these signals ever fed the verdict.

### Indexability

`noindex` is judged against the page goal. `--expect-indexable true` (or `[audit] expect_indexable = true`, implied for sitemap runs) makes it a gate: the SEO score is capped at 39 and the decision gets the reason `technical_blockers`. `false` excludes it as intended. Without either, the finding is listed with `applicability: unknown` and has no score effect.

### Finding id stability

`pages[].findings[].id` (`F-` and eight hex digits) identifies one rule on one page. It is the hash (FNV-1a 64, upper 32 bits, `a11y_report::FindingId`) of the rule id alone, so it is the same on every page and in every run, and the findings baseline (below) relies on exactly that. What it does not depend on, and must never start to depend on: elements, selectors, occurrence counts, their order, messages, severity, score, page content, tool version, language, viewport or the modules that ran.

What changes an id:

- **The rule id.** A renamed rule is a new finding to every existing baseline (its old id reads as `fixed`, the new one as `new`). Renames are therefore changelog entries (the 2.0 rule catalogue, #918, was such a change).
- **The id derivation.** A change of `FindingId`'s key moves every id. `FINDING_ID_SCHEME` (`audit::baseline`) must be raised in the same change; a baseline written under another scheme is refused, and so is a baseline whose ids this tool would not derive from the recorded rules.

What follows: a baseline entry means "this rule is reported on this URL", not "these elements". A new element violating an already known rule is not a new finding; the baseline stays quiet about it. Use the pattern ids (`findings[].patterns[].id`, `RC-` and eight hex digits, rule and normalized selector) for element-level comparison; the baseline does not use them. `tests/baseline_gating.rs` pins the ids of known rules (`finding_ids_of_known_rules_are_pinned`) and checks that reordering and element changes leave an id and the baseline digest alone.

## Decision Contract

`decision` (top level and on every page) is the one PASS/WARN/FAIL of the report; the terminal, JSON, summary feed, Studio, PDF, the batch `passed` count and the exit code all project it. It carries `contract` (`auditmysite.decision/3`), the `profile` it was computed with (`name` `default` or `custom`, from `[verdict]`), `verdict` and `reasons`.

Migration from `auditmysite.decision/2`: `decision.certificate` (`very_good` … `not_passed`) is gone, and so are the certificate badge and label in the terminal and PDF. Read `decision.verdict` and `decision.reasons` for the outcome and `summary.grade` / `accessibility_score` for the band; the Studio response and history entry lost their `certificate` field the same way. Cached snapshots written under `decision/1` or `decision/2` are cache misses.

- **Fail** when the profile enables it and there are legal flags, blocking issues, technical blockers (`fail_on_technical_blockers`, default off) or the accessibility score is below `fail_below_score`.
- **Warn** for an incomplete run, technical blockers, an accessibility score below `warn_below_score` (default 70), or, above it, any WCAG finding. `warn_below_score` is a policy threshold of the verdict, independent of the score bands (90/75/60/40) that name the tiers; no report text derives a tier name from it.
- Thresholds compare `accessibility_score`. `overall_score` never decides.
- A stored decision that its own profile does not reproduce is a contract violation (`decision`), reported with every other violation found.

Every path that publishes a decision reads the stored one and validates before writing:

| Path | Reads | Checked by |
| --- | --- | --- |
| Single JSON (`--format json`, PDF sidecar) | `NormalizedReport.decision` → `decision`, `pages[].decision` | `validate`, `report-lint` |
| Batch JSON | `BatchSummary.decision`, per-page decisions | `validate_batch` |
| Summary feed (`--format summary`) | `verdict`, `verdictReasons` | `validate` |
| SARIF | findings only; refuses invalid reports | `validate` |
| Terminal (single, batch) | stored decision | `validate` / `validate_batch` before rendering (CLI); `terminal_prints_the_stored_decision` checks the rendered verdict |
| Studio response and history | `decision` (verdict, reasons, profile) | Studio validates in its own repository; this crate checks nothing on that path (projection only) |
| PDF / Typst input | `ReportViewModel.summary.ci_verdict` from the stored decision | `validate` / `validate_batch` |
| Cache replay | the cached report re-judged under the current profile, stored in `run_context` | `validate` |
| Exit code | stored decision (`0` pass, `1` warn, `2` fail) | `exit_code_of_the_process_is_the_exported_verdict` (real binary, needs Chrome), `batch_with_only_failed_urls_exits_2_without_a_score` |

Batch: a page counts as passed when its own decision passes; the batch `decision` applies the same profile to the summary (legal flags, blocking issues, technical blockers and the average accessibility score). All pages of a batch share one profile.

## Findings Baseline

`--write-baseline <file>` records the finding ids of the run, `--baseline <file>` gates a later run against them: findings whose id is in the file are `known`, do not affect the verdict or the exit code, and are counted instead of listed. A baseline is a gating filter, not a suppression: scores, `findings[]`, `severity_counts` and every summary total stay as measured.

**File** (`auditmysite.baseline/1`): `contract`, `id_scheme` (`FINDING_ID_SCHEME`, 1), `tool_version`, `created_at` and `pages[]` with `url` and `findings[]` (`id`, `rule`). The `rule` makes a committed file reviewable in a diff, and the reader checks every `id` against it. A batch writes one entry per audited URL, a single run one. Every finding of the run is recorded, known or not, so `--baseline old.json --write-baseline new.json` accepts the current state.

**Usage errors** (exit 3, before any audit): a file that is not JSON or not a baseline, another `contract`, an `id_scheme` this tool does not use, an `id` that is not the id of its `rule`, a URL listed twice, an unreadable file; and `--write-baseline` with `--display all`. A baseline that cannot be used is never read as an empty one.

**Scoping.** A finding is known per page: the key is `url` (the audited URL as in `pages[].url`) plus the finding id. The same rule on another URL is new unless that URL lists it. A URL the baseline does not list has no known findings (`pages[].baseline.page_listed: false`).

**What is gated.** Under a baseline the decision reads the verdict inputs without the known findings: legal flags, blocking issues (`pages[].baseline.gate`), technical blockers and the open WCAG findings of the "any finding" warning. The `fail_below_score` and `warn_below_score` thresholds are not baselined (the score is unchanged), nor is the audit quality; the legal flag raised by the screen-reader BFSG verdict has no finding id and is never known. A batch decision reads `summary.baseline.gate` the same way. Exit codes are those of the decision (`0`, `1`, `2`); `--write-baseline` does not change the exit code of its own run (use `--report-mode` to only record).

**Reporting.**
- `pages[].findings[].known: true` marks a known finding (the field is absent otherwise).
- `pages[].baseline` (single and batch): `digest`, `page_listed`, `known`, `new`, `fixed` (baseline entries of this page without a finding in this run) with `fixed_ids`, and `gate`. `fixed` also counts entries of modules that did not run this time, and entries of renamed rules.
- Batch only, top level `baseline` (`summary.baseline` internally): `digest`, `known`, `new`, `fixed`, `unmatched_pages` (pages the baseline lists that this run did not audit) and `gate` (`legal_flags`, `blocking_issues`, `technical_blockers`, `open_violations`).
- `decision.profile.baseline_digest` is the SHA-256 over the recorded `url` and finding id pairs (not over `created_at` or `tool_version`, so rewriting an identical baseline keeps it). A gated decision never has the profile name `default`; a result with and without baseline cannot be mistaken for each other.
- The terminal table and the PDF show one line (`Baseline: n known, m new, k fixed findings`), no per-finding rows.
- A page whose stored `baseline` status, known flags or profile digest disagree fails validation as `baseline`, and so does a batch whose `baseline` block is not the sum of its pages.

Not covered: SARIF `baselineState` and any history of baselines. Cache replays (`--reuse-cache`, `replay`) are gated again under the current `--baseline`; the baseline is not part of the audit signature because it changes no measurement.


## Execution Scope and Audit Quality

The report distinguishes what was requested from what was successfully measured:

- `audit_scope` records requested modules, interaction mode and budget, viewports, throttle profiles, consent handling, and evidence capture.
- `execution_environment` records public browser/runtime context and whether data came from a live run or reusable artifacts. Executable paths and browser launch arguments are not exposed.
- `pages[].detail.module_runs` uses explicit execution states: `completed`, `partial`, `failed`, `skipped`, and `not_applicable`.
- `pages[].detail.rule_outcomes` records one entry per rule and viewport, keyed by `(rule_id, viewport)`. An entry that ran carries no `not_run` field and reports `findings`; an entry that did not run carries `not_run` with one of `capability_missing`, `disabled`, `not_applicable`, or `errored`, plus a free-text `reason`. Each entry also lists the `wcag` criteria that stand or fall with it, so the report can name which criterion went unchecked. A rule that could not run is never reported as passing.
- `audit_quality` summarizes whether results are complete, partial, or insufficient. A qualified result remains reportable but cannot silently become an unqualified clean verdict.
- `module_scores[].band` is always the band of the number. The PDF wording for Security can sit below it when an open finding of high severity (or any open finding, for the top band) contradicts the band text; the Security section then states this in one sentence under the score card. The cover sentence and the score label use the same bands (90/75/60/40) of the accessibility score.
- **Side checks** (`seo.www_redirect`, `seo.page_health.www_consolidation`, `security.certificate`; the list is `audit::collection::SIDE_CHECKS`) are optional probes about the site around the audited page, not about the page's content: the other host variant (apex / www) and the browser-reported TLS certificate details. When one gets no answer (`alternate_host_unreachable`, `certificate_state_unavailable`) it is a `failed` subcheck with its reason code and an `audit_flags` entry of kind `side_check_unanswered` naming exactly which check is missing. It does not make its module `partial`, does not count in `audit_quality.partial_or_failed_modules` and therefore never turns a fully measured page into a `partial` audit, a provisional rating, a `warn` decision or a failed batch URL. Every other lost collection (headers, links, scripts, scoring collectors) keeps qualifying its module and the audit.
- `pages[].navigation` and `pages[].consent` describe the requested/final URL, main-document status, redirect count, bounded page-stability provenance, and whether consent was not detected, detected, dismissed, failed, or remained unknown. Consent evidence never includes cookie values.

Confirmed violations remain separate from `accessibility_assessments`, which carries warnings, manual-review items, and positive signals. Measurement failures are not converted into violations or artificial score penalties.

Binary evidence is not embedded in the JSON. The top-level `artifacts` array points to separately written sidecars when available. Whenever screen-reader data exists, `<output>-screen-reader-audit.json` is written next to the report as an intended sidecar (the structured screen-reader audit that the PDF summarizes) when `--screen-reader-report` asks for it; by default no sidecar is written.

## Performance Measurement Context

- All performance metrics are **lab data** captured in a local headless Chrome (via CDP / `PerformanceObserver`). They are never field/RUM data (e.g. Chrome UX Report / CrUX).
- Each Core Web Vital (`detail.modules.performance.vitals.*`) carries a `measurement` field:
  - `measurement = "lab_headless"` — directly measured in the local headless browser (LCP, FCP, CLS, TBT, TTFB).
  - `measurement = "estimated_lab"` — derived/estimated from other lab signals (INP, TTI, Speed Index). These are approximations, not direct measurements, and must not be read as field metrics.
- In rendered reports, estimated metrics carry a localized "(lab estimate)" / "(Lab-Schätzung)" suffix, and the performance section states that all values are lab data, not field data.

### Measurement window (#770)

LCP, CLS and TBT are read from `PerformanceObserver`s that are installed before the navigation, so they see the page from its first paint. One page load is one observation:

- **Start**: navigation start (observers are registered before the document is created, buffered entries included).
- **End**: the read happens after the page has been waited on, not at the `load` event. First the audit waits up to about 3 s for the first LCP candidate, then until no new long task has appeared for 1 s (Lighthouse's CPU-quiet threshold) or for at most 8 s. The values are whatever the observers have seen at that read; there is no fixed observation time.
- **LCP**: the last candidate before the read. There is no `load` cut-off: a hero rendered after load is the LCP. A lab run never produces user input, so candidates are never closed by one.
- **CLS**: the largest session window (shifts at most 1 s apart, at most 5 s long), shifts with recent input excluded. Shifts after the read are not seen.
- **TBT**: the blocking part (beyond 50 ms) of every long task, clipped to the window FCP to TTI. Long tasks before FCP or after TTI do not count. TTI is an estimate (last long task before the first 5 s quiet window after FCP, at least `domInteractive`) and does not wait for network quiet. Without FCP or the long-task observer TBT is not measured, never 0.

Not measured: user interaction (INP stays an estimate and is only set when interaction events fire), scrolling, work that starts after the read (late lazy-loading, delayed third-party scripts, client-side navigation), cache-warm repeat visits, and real-device or field behaviour. A page that keeps shifting or blocking after the window reads better here than it is. `--perf-runs N` repeats each throttled profile and reports the median run with min/median/max; it does not widen the window of a single load.

## Measurement Profile

- `audit_scope.requested_rules` lists every rule the run must account for — shared, AX-tree and page rules (executor and concrete ids) and the inline rules (reflow, HTML content model; mobile only) — with the viewports it runs in, derived from the rule registrations, the `wcag_level` and the `[rules]` filter. Each listed `(rule_id, viewport)` needs exactly one `rule_outcomes` entry; a missing one fails validation as `requested_rule_outcome`, a second one as `rule_outcome_duplicate`, and a rule that ran above `audit_scope.wcag_level` as `rule_outcome_above_level`. Error and capability counters count a page-rule executor and its concrete rules once.
- Frames are accounted for per frame. `audit_scope.frame_rules` lists the rules every audited iframe must report — the frame tree rules the level and the filter leave, and every frame shared rule (reported as not run when above the level or filtered out). `frames.audited[].outcomes` carries one outcome per `(rule_id, viewport)` for each audited frame; a missing one fails validation as `frame_rule_outcome_missing`, a second as `frame_rule_outcome_duplicate`, a rule outside the list as `frame_rule_outcome_unexpected`, and a frame rule that ran above `wcag_level` as `frame_rule_outcome_above_level`. Skipped frames owe no outcomes. The frames' findings still fold into the page's entry for the rule.
- `audit_scope.rule_filter` records the `[rules]` filter (`disabled_rules`, `enabled_only_rules`) the run applied; it is absent when none was set. A tree or inline rule the recorded filter removes that still has a ran outcome fails validation as `rule_outcome_filtered`.
- `audit_scope.requested_subchecks` lists the collections every requested extra check must leave: third-party isolation and the SSR gap (one each) and the privacy first visit (`privacy.first_visit`, only with the opt-in `--privacy-check`). Every id needs at least one entry in `module_runs[].subchecks`; otherwise validation fails as `requested_subcheck_outcome`. The extra-reload checks record after `audit_page` returns, so the pipeline applies this check to the finished report (single URL and batch page alike), not to the intermediate one; output validation always applies it.
- **State axis.** The state a result was measured in is the measurement profile: the session policy, `state_id` and the `storage_state` digest. There is no separate per-state axis in the execution record; a different profile is another measurement (see below), and one report is one profile.
- `audit_scope.measurement_profile` states how the pages were measured: `version` (meaning of the profile), `session_policy` (`isolated` or `shared`; `unknown` for reports from before the field), `read_only`, `extra_checks` (flag names of the opt-in checks the run requested, e.g. `--check-ssr-content`, `--privacy-check`; sorted, absent when none) and `state_changing_journeys` (`form_error_submit`, `add_to_cart`, `quantity_stepper`; listed only with `--interactive full` and `--allow-state-changes`).
- `state_id` and `storage_state` (`sha256`, `cookies`, `origins`) name a declared browser state (`--storage-state`, `--state-id`). Values never enter the report. A different state file is a different `sha256` and so another measurement.
- `measurement_profile.browser` names the browser that measured (profile version 3): `product` (`chrome`, `chrome_for_testing`, `headless_shell`, `edge`, `chromium`, `ungoogled_chromium`, `unidentified`), `version`, `origin` (`system`, `user`, `path`, `managed`, `explicit`), `os` and `arch` (`macos`/`linux`/`windows`, `x86_64`/`aarch64`), `headless_mode` (`new`, `legacy`, `headful`), and `failover` with `discarded_candidates` (candidates rejected or failed ahead of the one that measured; `failover` is true iff the count is above 0). It holds categories only: no executable path, user name or launch flag, which are personal and not reproducible. A profile of version 3 or later without it, with an empty version, with a managed product from a non-managed origin (or the reverse), with `failover` contradicting the count, outside the platform contract (macOS ARM64, Linux x86_64/ARM64, Windows x86_64 with Edge only), or disagreeing with `execution_environment` on headless mode or version fails as `browser_provenance`. Reports of profile version 2 and older carry none and stay readable.
- A replay compares the browser by identity (product, version, platform, headless mode); how it was found and a failover are provenance, not a different measurement (`profile_differences` field `browser`).
- A batch lists every distinct browser provenance of its pages in `summary.measurement_browsers` (`browser`, `pages`; `browser: null` counts pages whose browser was not recorded). Two or more entries mean the batch mixes browsers (cache replays of another run, merged files); it is never averaged away, and a stored list that differs from the pages fails as `batch_aggregation`. The top-level `audit_scope` of a batch shows the first page's profile only.
- A result measured under another profile is another measurement. Validation rejects a profile whose `read_only` or journey list contradicts `interactive_mode`.

## Validation codes

A report that contradicts itself is rejected with machine-readable codes (`ReportContractError.violations[].code`, `audit::CONTRACT_CODES`). The codes, by area:

- Baseline: `baseline`.
- Scores and decision: `score_range`, `grade`, `overall_score`, `average_score`, `score_method`, `module_grade`, `module_weight`, `module_identity`, `weighted_measurement`, `viewport_score`, `semantic_score_cap`, `gate_score_cap`, `passed`, `decision`, `decision_contract`, `decision_profile`, `cause_charged_twice`.
- Findings and counts: `finding_domain`, `finding_identity`, `finding_inventory`, `finding_occurrences`, `occurrence_counts`, `severity_counts`, `rule_findings_preserved`.
- Execution scope: `requested_execution`, `requested_rule_outcome`, `rule_outcome_duplicate`, `rule_outcome_above_level`, `rule_outcome_filtered`, `requested_subcheck_outcome`, `frame_rule_outcome_missing`, `frame_rule_outcome_duplicate`, `frame_rule_outcome_unexpected`, `frame_rule_outcome_above_level`, `measurement_profile_read_only`, `measurement_profile_journeys`, `browser_provenance`.
- Quality and collections: `accessibility_module`, `audit_quality`, `execution_completeness`, `quality_qualification`, `failed_rule_checks`, `collector_unknown`, `collection_state_unexplained`, `collection_gap_completed`, `collection_gap_scored`.
- Batch: `batch_aggregation`, `total_urls`, `sample_strata`.

## Snapshot, Replay and Validation Paths

The cache entry's `snapshot.json` is the collected evidence, independent of any presentation: the AX tree and raw module captures, the whole collected report (modules and subchecks, rule outcomes, execution states, `audit_scope` with its `measurement_profile`, the run context; screenshots excluded), the document identity (`requested_url`, `final_url`, `status_code`, `redirect_count`), the run language and `versions`.

- `versions` records the snapshot schema, the content fingerprint, the collection contract, the measurement-profile version, the evaluation version (normalization, scoring, decision derivation), the decision contract and a digest of the shared rule catalogue. A snapshot is replayed only when all of them equal the current values.
- `audit::replay(snapshot)` evaluates a snapshot without browser or network and returns the same `NormalizedReport` as the fresh run. Only `timestamp`, `duration_ms` and `execution.environment.source` (`replay`) may differ. `output::format_json_replay` exports it with `detail.module_detail: full`.
- A snapshot that lacks parts (`ReplayError::Incomplete`, naming them) or was written under other versions (`ReplayError::Incompatible`, naming field, stored and current value) is never evaluated under today's semantics. `load_artifacts` treats it as a cache miss; `lookup_artifacts` and `cached_report` return the reason (`CacheMiss`: absent, unreadable, other configuration, not replayable), which the CLI prints.
- `--reuse-cache` answers through `replay_report_as(snapshot, "cache")`, `auditmysite replay <snapshot|directory|URL>` through `replay_report`. `SnapshotArtifact::check_measurement(&ExpectedProfile)` refuses a snapshot whose session policy, state id or storage-state digest differs from the asserted one (`ReplayError::OtherMeasurement`); `audit::profile_differences` names the differences between two profiles. A replay measures nothing, so the extra-check flags are refused for it (support matrix, mode `Replay`).
- `format_json_cached` works on a stored `NormalizedReport` alone and says so with `module_detail: not_restored`; an empty module block there never stands for restored data.
- Bump `SNAPSHOT_SCHEMA_VERSION` when the shape of the collected report changes in a way old data would survive deserialization, and `EVALUATION_VERSION` when a change moves a score, finding or verdict for unchanged input.

Every fallible hand-over (live pipeline, `normalize_checked`, `validate_batch`, JSON single/cached/replay/batch, summary, SARIF, AI JSON, terminal renderers, `technician::finding_rows`, PDF source) validates the domain data first; `tests/validation_path_matrix.rs` refuses contradictory data at each of them and keeps the CLI dispatchers and the cache-hit path on the validating route. `output::format_ai_json`, `terminal::render_single_report[_for]`, `terminal::render_batch_report` and `technician::finding_rows` return `Result<_, ReportContractError>` and refuse a contradictory report with its codes and data paths; a qualified partial result (failed or unmeasured parts stated as such) stays usable. Only `audit::normalize` remains an infallible, deprecated compatibility API: it holds no check of its own and is called only on data that came through a checked entry. A new unchecked `normalize()` call in `cli`, `output`, `studio` or `lib.rs` fails the test unless the baseline in `tests/fixtures/architecture/unchecked_normalize.txt` is changed on purpose. The 2.0 migration path for callers outside the crate is `normalize_checked` (identical model for valid data, `ReportContractError` with codes and data paths otherwise).

Not part of 2.0 (deferred to 2.1): evidence references on findings, and automatic detection of changed rule logic. Until then `EVALUATION_VERSION` is the manual guard: whoever changes a rule, score or decision for unchanged input bumps it, which turns stored snapshots into cache misses.

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

Inbound links are counted the same way in both places that report them: `url_matrix[].inbound_links` is the number of distinct other audited pages that link to the URL, and `site_analysis.consistency.orphan_pages.urls` lists the audited pages where that number is 0 (the 2.4.5 assessment counts them in `affected_pages`). Link targets are resolved against the linking page's URL; links to the page itself and repeated links from the same page do not count.

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

<a id="grade"></a>
### grade — `summary.grade`

Letter grade banded from `summary.accessibility_score` — the subject of the
report. It is a band label for the automated scope, not a conformance statement.

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
optional Security contribution. It is a viewport-pass figure, not a published
module weight and not the report's `overall_score` (modules weighted per the
published weights, Security added afterwards); read `overall_score` for the
score of the page.

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

**What it is.** A tool convention, not a measurement and not a legal or
statistical model. The weights below were chosen by the maintainers so that the
index orders pages by how much harm and legal-review exposure their automatic
findings suggest; no empirical calibration or legal source stands behind the
exact numbers. Read the counts (`legal_flags`, `critical_issues`,
`high_issues`, `blocking_issues`) as the evidence and the score as a summary.

```
score = min(100,
    20 * legal_flags
  + 10 * critical_issues              (WCAG, per occurrence)
  +  3 * high_issues                  (WCAG, per occurrence)
  +  2 * blocking_issues              (per element)
  + 10 * interactive_critical_issues  (journey findings)
  +  5 * interactive_high_issues
  +  3 * screen-reader findings of severity high
  +  1 * screen-reader findings of severity medium)
```

- `legal_flags` counts *distinct rules* on the page, not occurrences (1000
  images without alt text are one flag). A finding is a legal flag when it is a
  WCAG Level A **or** Level AA finding of High or Critical severity, is
  `automatically_confirmed` (a finding that needs manual review raises none),
  and its criterion has a normative clause in EN 301 549 V4.1.1 (2026-09,
  clauses 9.1-9.4: the WCAG 2.2 A/AA table; AAA criteria do not flag, the
  retired 4.1.1 Parsing is void in that edition). Convention rules (`display/*`)
  never flag. AA is included because BFSG and EN 301 549 require AA: a High
  contrast failure (1.4.3) is as actionable as a Level A failure. A page also
  carries at least one flag when the screen-reader audit confirms a
  BFSG-violating high-severity blocker (empty interactive element, unlabeled
  form field) and was not blocked by a consent wall. The flag marks automatic
  evidence worth a legal review, not a legal conclusion.
- `critical_issues` and `high_issues` count occurrences, so a widespread defect
  weighs more than an isolated one. `blocking_issues` counts distinct elements
  with a missing accessible name or role (4.1.2) or a keyboard failure (2.1.1),
  Medium severity or higher.
- Why these weights: a legal flag is weighted highest (20) because it is the
  only term that signals exposure beyond user harm; a critical occurrence (10)
  is a barrier that stops a task; a high occurrence (3) hinders it; a blocker
  (2) is added on top of those counts, so it is a small extra for the same
  elements. Journey findings count per finding rather than per occurrence, at 10
  for critical (equal to a static critical occurrence) and 5 for high (above a
  static high occurrence); the 5 is a convention, not a result. Screen-reader
  findings are heuristic quality signals and weigh 3 (high) and 1 (medium);
  they never enter `severity_counts`. The relative order is
  deliberate; the absolute values are not derived from anything.
- The sum is capped at 100, so on a heavily affected page the score saturates
  and stops distinguishing pages. The `level` does not.

**`level`** is not read from the score. It is chosen by explicit precedence, the
first match wins:

| Level | Condition |
| --- | --- |
| `critical` | `legal_flags >= 3` (breadth: at least three distinct Level A/AA rules at High/Critical) or `critical_issues >= 5` (volume) |
| `high` | `legal_flags > 0` together with `critical_issues > 0`, or `critical_issues >= 3`, or `blocking_issues >= 5`, or `risk_score >= 80` |
| `medium` | `high_issues >= 3` with accessibility score below 80, or any critical issue, legal flag, blocking issue, interactive critical/high finding, or accessibility score at most 20 |
| `low` | otherwise |

`driven_by` names the condition that fired (`Legal Compliance` only when the
breadth path produced `critical`). `threshold` (60/30/10/0) is a fixed label per
level, not a cut-off the score is compared against; the only score comparison
in the level logic is `risk_score >= 80`. The "at least three" breadth bound and
the volume bounds (5, 3, 5) are likewise conventions.

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

## Annotations

`annotations` (top level, single and batch) exists only when the run was given `--annotations <file.json>` (#961; format defined in barrierlab, `docs/a11y/reference/annotations.md`). Contract:

- It is a separate block, never a field inside a finding, and nothing else in the report depends on it: outcome, severity, scores, `legal_flags`, risk level, `decision` and the exit code are the same with and without it. Tests compare a run with and without the file and require everything except this block to be equal.
- `matched[]` entries carry the `url` and, for scope `finding`, the `finding_id` (`pages[].findings[].id`) and optional `pattern_id` they attached to; `unmatched[]` entries carry a `reason`. Every annotation of the file is in one of the two lists.
- `counts` are over distinct annotations: `total = matched + unmatched` when no annotation is both (an annotation without `url` that matches two pages is one matched annotation with two entries); `by_author_kind` counts them by `author.kind`.
- `annotation` objects reproduce the file's known fields; unknown fields are not carried.

## EN 301 549 Annex

`pages[].detail.en301549_annex` maps the page's WCAG findings onto the chapter 9 ("Web") clauses of **EN 301 549 V4.1.1 (2026-09)**, which requires WCAG 2.2 Level A/AA. The edition is stated in `standard_version`; `mapping_version` (currently 2) changes whenever the clause table or its status semantics change. The batch roll-up `summary.en301549_rollup` uses the same table.

- `clauses[]` has 55 entries, the 55 WCAG 2.2 A/AA success criteria (31 A, 24 AA), each as `en_clause` = `9.` + the WCAG number. Clause numbers of the 49 criteria that WCAG 2.1 and 2.2 share are identical to EN 301 549 V3.2.1; V4.1.1 adds 9.2.4.11, 9.2.5.7, 9.2.5.8, 9.3.2.6, 9.3.3.7 and 9.3.3.8 and voids 9.4.1.1 (Parsing, removed in WCAG 2.2).
- The WCAG 2.2 Level AAA criteria (2.4.12, 2.4.13, 3.3.9, like all AAA) have no normative clause in V4.1.1; the standard lists them only informatively in clause 9.5. Findings on them are not in `clauses[]` and are counted in `out_of_standard_findings`.
- `mapping_version` 1 (reports before this change) targeted EN 301 549 V3.2.1 (2021-03) with 50 clauses (WCAG 2.1, including 4.1.1).
- The annex is technical evidence, not an accessibility statement (`disclaimer`). V4.1.1 grants a presumption of conformity only once it is cited in the Official Journal of the European Union; until then the cited edition remains the authoritative one for that purpose.

## JavaScript Library Advisories

`best_practices.vulnerable_libraries.vulnerable[]` lists detected libraries whose version lies in an advisory range. The ranges come from the versioned data file `src/best_practices/advisories.json` (process: [advisory-data.md](advisory-data.md)); the check covers only the libraries in `CHECKED_LIBRARIES`.

- `advisories[].review_status` is `reviewed` (the range was checked against `source_url` on the date in `advisories[].reviewed`) or `unreviewed` (carried over, not yet checked). An unreviewed match is a lead, not an established statement; the PDF says so next to the finding. Reports written before the field read as `unreviewed`.
- `exploitability` is `unconfirmed` (the version is in range; reachability on the page was not examined) or `not_applicable` (only end-of-maintenance notices matched, so no specific vulnerability exists to exploit). There is no `confirmed` state: a version never proves the code path is reachable, and the check runs no reachability probe.
- `detected[].version_parsed: false` marks a version that could not be read; it is neither flagged nor cleared.
- The check runs with the `performance` module and with the `security` module on its own (`--modules performance`, `--modules security`).

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
