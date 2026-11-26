# Output Contract

`auditmysite` treats JSON output as a compatibility surface for automation.

## Stability Rules

- Single-report JSON is generated from `NormalizedReport` plus `metadata` and optional module payloads.
- Batch JSON is generated from normalized reports plus batch `summary`, optional `errors`, and deterministic `metadata`.
- Timestamps are data-derived, not generated at serialization time.
- New top-level fields may be added in a backward-compatible way.
- Existing required fields should not be removed or renamed without a versioned contract change: a rename that is not additive raises the minor part of `schema_version` (see "Site quality score (optional modules)" below for the 2.1 step and "One axis for the accessibility score and its components" for 2.2).

## Module Classification

- Every `report.module_scores[]` entry includes `measurement_type` and `band`.
- `band` is the canonical English qualitative label for `score`; it is present on every entry.
- `grade` (a letter) is present **only** for modules with `weight_pct > 0`. A module that does not feed `site_quality_score` carries `band` alone: grading an indicator on the same A–F scale as a weighted module gave it an authority it does not have.
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
- `site_quality_score` is the **site quality (optional modules)** figure: a secondary, explicitly labelled score, the weighted result across the active contributing modules (accessibility 40, performance 20, SEO 15, security 10, mobile 10, HTML conformance 5 by default), reported together with `site_quality_score_weight_basis`. It is not the headline and carries no classification of its own. Until schema 2.1 it was called `overall_score`; "overall" suggested the one number that sums the report up, which is the accessibility score.
- `violation_count` and `occurrence_counts` count WCAG occurrences: affected elements or instances, not distinct rules.
- `violated_rule_count` and `severity_counts` count distinct grouped WCAG findings.
- `finding_count` and `finding_occurrence_count` include findings from all reported domains (WCAG, SEO, security, mobile, …).
- There is no unqualified `score` in the summary: a score is either `accessibility_score` or `site_quality_score`.

## Finding Contract

`pages[].findings[]` holds every problem any module collected, one finding per rule (WCAG) or per collector check (`<source>.<check>`, e.g. `seo.technical.noindex`). `category` is `wcag` for accessibility rule findings and the domain name otherwise; it is derived from `contract.domain`, and a report whose `category` names another domain is rejected when it is read back. `rule_id` is `contract.check` for a WCAG finding (`contract.source` `wcag`) and `<contract.source>.<contract.check>` otherwise, `aggregation_key` equals `rule_id`; a report that breaks either is rejected when it is read back. Each finding carries `contract`:

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

### Finding display title (#1170)

`findings[].title` is the English rule title and never changes with the report. The PDF's key measures, action table and finding cards show a display title for the rules of a missing accessible name or label (`links/name-missing`, `buttons/name-missing`, `names/required-missing`, `forms/label-missing`): when every element is of one kind it names that kind ("Link without an accessible name"), with mixed or other kinds it keeps the rule title, and it adds the reach ("on all 5 audited pages", "on 2 of 5 pages", "20 occurrences on this page"). The kinds come from `findings[].observed_roles`, the tally of the AX roles over all occurrences of the finding; a kind is never inferred from a class name or selector.

### Finding origin (#1109)

Findings in markup the site owner does not control (an embedded widget, a consent platform frame) used to read like first-party findings. Since the frame pass audits cross-origin frames too (#715, #720), every finding says where its markup comes from. Attribution only: **scoring is unchanged**.

- `findings[].origin` (always present): `first_party` (the main document), `same_site_frame` (a frame of the page's own registrable domain, or one without a document URL of its own: `about:blank`, `srcdoc`), `cross_origin_frame` (a frame of another site) or `mixed` (its occurrences have more than one origin). It is derived from **all** occurrences of the finding, not only the listed sample, so `mixed` is never an artefact of the occurrence cap. `findings[].providers` lists the known third-party providers (canonical English names, sorted) of the cross-origin frames the occurrences sit in; absent when there are none.
- `findings[].occurrences[].origin`, `.provider` and `.frame_host` (the host of the frame's source) appear only for an element inside a frame (`same_site_frame` / `cross_origin_frame`); an occurrence without them is in the main document. The PDF label names `frame_host` where the provider is unknown ("Third-party frame (source ir-api.example.com), fix at the provider of the embedded content"). `mixed` exists on findings only.
- Same site means the same registrable domain by the public suffix list (`psl` crate): `www1.example.com` in a page of `www.example.com` is a same-site frame, `a.github.io` in `b.github.io` is cross-origin. A host without a registrable domain (an IP address, `localhost`) is only the same site as itself. The frame's source is its `src` attribute (resolved against the page) first and the frame document's URL only when there is no usable `src`: a lazy-loaded frame is still `about:blank` when it is audited (`frames.audited[].url`), and only the attribute says where it comes from. A frame without any http(s) source (`about:blank`, `srcdoc`, `data:`) is the page's own markup. The page URL is the page's URL at the time of the rule run. This covers both sources of frame findings: the frame pass (`"{iframe} [frame] {element}"`, `frame:` node ids) and the `html-has-lang` scan of frame documents.
- The provider is the answer of the `web-checks` provider list (`third_party::classify_url`) for the frame URL; there is no list of our own. An unknown host stays a cross-origin frame without a provider; "unknown" is no finding. Page-level findings from consent-platform markup in the main document (for example OneTrust selectors, `aria-labelledby` on `.wxp-overlay-container`) stay `first_party`: they sit in the page's DOM, and a provider cannot be derived from a selector without a heuristic, which this contract does not introduce.
- The PDF words it as a neutral label ("Third-party frame (provider X), fix at the provider"; same-site and mixed variants), on the finding card of a single report and on the cross-page top-issue card of a batch, where the provider is named once per aggregated finding, not once per URL. Summary, verdict and score texts are unchanged and never call a page conformant because of an origin.
- **Decision on scoring (2026-11-21, #1109): attribution only, measured.** Score, grade, verdict, decision profile, finding ids (rule id alone), reference bands and baselines do not read the origin; `origin_does_not_change_score_grade_or_decision` pins this. The measurement ([third-party scoring measurement](https://casoon.github.io/auditmysite/docs/reference/third-party-scoring-measurement/): 8 reference sites plus 4 pages with embedded widgets, frames loaded) does not support a separate weight, a cap per frame or a separate bucket: seven of the eight reference sites take no point from a frame or a provider, and the one that does (basf.com, 15 points) takes it from a same-site frame of its own platform, which its owner can fix. The one case it points to is the critical cap: on three pages (nasa.gov, sundance.org, stackoverflow.blog) a critical finding in a cross-origin frame alone holds the page score at 49. The sample is small (3 pages, one run each, YouTube the only known provider), so this is not changed now; it is re-evaluated when more cross-origin cases are measured. Any change would move scores and reference bands and is its own change.
- Compatibility: additive; a report without the fields reads as `first_party` / no providers. `schema_version` stays `2.2`; cache `fmt` 37 turns cached results into misses so they are re-rendered with their origin. `SNAPSHOT_SCHEMA_VERSION` and `EVALUATION_VERSION` are unchanged (nothing moves), so a replay of a snapshot written before reads every finding as `first_party`: the origin was not recorded then.

### Signal contracts

Every scored payload in `pages[].experimental` (`ux`, `journey`, `source_quality`, `ai_visibility`, `content_visibility`) carries `signal_contract: { format: "auditmysite.signal-contract/1", contract }`. `contract` states `target_statement`, `method`, `applicability`, `evidence`, `uncertainty`, `permitted_interpretation`, the claims it does not establish (`not_established`) and `validation.status` — `unvalidated` until a pre-registered calibration run passes (`docs/calibration.md`). A score of an unvalidated contract is not evidence of visibility, AI citation, user quality or source truth.

### Experimental indicators

UX, Journey, AI Visibility, Content Visibility, Source Quality and Commerce are experimental (#916): `unvalidated` signal contracts, no calibration set, no weight in any score. They run only with `--experimental <name>[,<name>]` and are in no `--full` run.

- A single report carries them in `pages[].experimental`, one key per indicator that ran (`ux`, `journey`, `ai_visibility`, `content_visibility`, `source_quality`, `commerce`), each scored one with its `signal_contract` (`commerce` is a score-less checklist and has none). The block is absent when none ran. A batch report carries only `summary.experimental.commerce`, the site-wide roll-up of the commerce trust pages.
- `experimental.ux.heuristics` (#1035, only where `ux` is) holds the advisory page heuristics of the accessibility tree: `{ "findings": [{ "kind", "severity", "count"?, "message" }] }`. `kind` is the stable check id (`cookie_banner`, `dialog_not_modal`, `multiple_dialogs`, `no_cta`, `many_ctas`), `count` the counted value where the text names one (dialogs, call-to-action elements), `message` canonical English; an empty `findings` means the checks ran and found nothing. They are unvalidated keyword and role heuristics under the `ux` signal contract, not rules: no rule id, no taxonomy entry, no effect on the `ux` score or on any score, finding or verdict. There is no font check: the accessibility tree holds no font information.
- They are in no `module_scores[]` entry, no `findings[]`, `finding_inventory`, `risk`, `decision` or PDF chapter, and `detail.modules` has no key for them. `ux_score`, `journey_score` (page and summary) and `summary.commerce` of 2.0.x are gone.
- An indicator another one is derived from runs with it and appears in the block: `content_visibility` brings `source_quality` and `ai_visibility`, `source_quality` brings `ux`.
- The add-to-cart and quantity-stepper journeys belong to `commerce`: without it `measurement_profile.state_changing_journeys` does not list them. `--check-ssr-content` needs `content_visibility`.
- Versions: `EVALUATION_VERSION` 7, `SNAPSHOT_SCHEMA_VERSION` 2 and the cache signature (`fmt` 32, now with the indicator selection) turn stored snapshots into cache misses. `decision/3` is unchanged: none of these signals ever fed the verdict.

### Indexability

`noindex` is judged against the page goal. `--expect-indexable true` (or `[audit] expect_indexable = true`, implied for sitemap runs) makes it a gate: the SEO score is capped at 39 and the decision gets the reason `technical_blockers`. `false` excludes it as intended. Without either, the finding is listed with `applicability: unknown` and has no score effect.

### Finding id stability

`pages[].findings[].id` (`F-` and eight hex digits) identifies one rule on one page. It is the hash (FNV-1a 64, upper 32 bits, `a11y_report::FindingId`) of the rule id alone, so it is the same on every page and in every run, and the findings baseline (below) relies on exactly that. What it does not depend on, and must never start to depend on: elements, selectors, occurrence counts, their order, messages, severity, score, page content, tool version, language, viewport or the modules that ran.

What changes an id:

- **The rule id.** A renamed rule is a new finding to every existing baseline (its old id reads as `fixed`, the new one as `new`). Renames are therefore changelog entries (the 2.0 rule catalogue, #918, was such a change, and so were the rule moves of 2.2.0, listed with their baseline and `[rules]` filter effect in the [2.1 to 2.2 migration guide](https://casoon.github.io/auditmysite/docs/guides/migration-22/)).
- **The id derivation.** A change of `FindingId`'s key moves every id. `FINDING_ID_SCHEME` (`audit::baseline`) must be raised in the same change; a baseline written under another scheme is refused, and so is a baseline whose ids this tool would not derive from the recorded rules.

What follows: a baseline entry means "this rule is reported on this URL", not "these elements". A new element violating an already known rule is not a new finding; the baseline stays quiet about it. Use the pattern ids (`findings[].patterns[].id`, `RC-` and eight hex digits, rule and normalized selector) for element-level comparison; the baseline does not use them. `tests/baseline_gating.rs` pins the ids of known rules (`finding_ids_of_known_rules_are_pinned`) and checks that reordering and element changes leave an id and the baseline digest alone.

## Decision Contract

`decision` (top level and on every page) is the one PASS/WARN/FAIL of the report; the terminal, JSON, summary feed, Studio, PDF, the batch `passed` count and the exit code all project it. It carries `contract` (`auditmysite.decision/3`), the `profile` it was computed with (`name` `default` or `custom`, from `[verdict]`), `verdict` and `reasons`.

Migration from `auditmysite.decision/2`: `decision.certificate` (`very_good` … `not_passed`) is gone, and so are the certificate badge and label in the terminal and PDF. Read `decision.verdict` and `decision.reasons` for the outcome and `summary.grade` / `accessibility_score` for the band; the Studio response and history entry lost their `certificate` field the same way. Cached snapshots written under `decision/1` or `decision/2` are cache misses.

- **Fail** when the profile enables it and there are legal flags of High or Critical severity (`severe_legal_flags`; the reason names `legal_flags`), blocking issues, technical blockers (`fail_on_technical_blockers`, default off) or the accessibility score is below `fail_below_score`.
- **Warn** for an incomplete run, technical blockers, an accessibility score below `warn_below_score` (default 70), or, above it, any WCAG finding. `warn_below_score` is a policy threshold of the verdict, independent of the score bands (90/75/60/40) that name the tiers; no report text derives a tier name from it.
- Thresholds compare `accessibility_score`. `site_quality_score` never decides.
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

Batch URL counts (`summary`, #899): `passed_url_count` pages with verdict `pass`, `fail_url_count` pages with verdict `fail`, `warn_url_count` pages with verdict `warn`; the three sum to `url_count`. `failed_url_count` is a deprecated alias with its 2.0.x value, the pages whose verdict is not `pass` (`fail_url_count + warn_url_count`); it stays until a release announces its removal in the changelog. A single report carries the same fields for its one page. `schema_version` stays `2.0` (additive: both schemas gain optional properties). The batch `decision.reasons` text for those pages changes from `N URLs failed quality threshold` to `N URL(s) failed its/their own check` and `N URL(s) with a warning`, each only when its count is above zero; consumers that match the old string must read the counts instead. Batch PDF and terminal use the same split (`Failed` is `fail_url_count`, `Warned` is `warn_url_count`).

**Failed audit against missed criterion (#1213).** The URL counters of a batch answer two different questions and are never summed into one. `fail_url_count`, `warn_url_count` and `passed_url_count` count *audited* URLs by their own verdict: the audit ran, a report exists, and the page passed, passed with a reservation or missed a criterion of the decision profile. `failed_url_count` (deprecated alias) is `fail_url_count + warn_url_count` and means the same, never a technical failure. `coverage.failed` and `errors[]` count URLs whose *audit* failed technically (timeout, navigation or protocol error, no report produced): they have no score, no verdict and enter no average. `coverage.blocked` counts the audits stopped by a bot wall. `url_count` (= `coverage.succeeded`) is the number of audited URLs, `attempted_url_count` the number of audits started; so `attempted_url_count = url_count + coverage.failed + coverage.blocked`. The column heads of the PDF say what they count: a findings column counts findings grouped per page (one rule on one page is one finding), the cover figures count single occurrences.

## Findings Baseline

`--write-baseline <file>` records the finding ids of the run, `--baseline <file>` gates a later run against them: findings whose id is in the file are `known`, do not affect the verdict or the exit code, and are counted instead of listed. A baseline is a gating filter, not a suppression: scores, `findings[]`, `severity_counts` and every summary total stay as measured.

**File** (`auditmysite.baseline/1`): `contract`, `id_scheme` (`FINDING_ID_SCHEME`, 1), `tool_version`, `created_at` and `pages[]` with `url` and `findings[]` (`id`, `rule`). The `rule` makes a committed file reviewable in a diff, and the reader checks every `id` against it. A batch writes one entry per audited URL, a single run one. Every finding of the run is recorded, known or not, so `--baseline old.json --write-baseline new.json` accepts the current state.

**Usage errors** (exit 3, before any audit): a file that is not JSON or not a baseline, another `contract`, an `id_scheme` this tool does not use, an `id` that is not the id of its `rule`, a URL listed twice, an unreadable file; and `--write-baseline` with `--display all`. A baseline that cannot be used is never read as an empty one.

**Scoping.** A finding is known per page: the key is `url` (the audited URL as in `pages[].url`) plus the finding id. The same rule on another URL is new unless that URL lists it. A URL the baseline does not list has no known findings (`pages[].baseline.page_listed: false`).

**What is gated.** Under a baseline the decision reads the verdict inputs without the known findings: legal flags, blocking issues (`pages[].baseline.gate`), technical blockers and the open WCAG findings of the "any finding" warning. The `fail_below_score` and `warn_below_score` thresholds are not baselined (the score is unchanged), nor is the audit quality; the legal flag raised by the screen-reader BFSG verdict has no finding id and is never known. A batch decision reads `summary.baseline.gate` the same way. Exit codes are those of the decision (`0`, `1`, `2`); `--write-baseline` does not change the exit code of its own run (use `--report-mode` to only record).

**Reporting.**
- `pages[].findings[].known: true` marks a known finding (the field is absent otherwise).
- `pages[].baseline` (single and batch): `digest`, `page_listed`, `known`, `new`, `fixed` (baseline entries of this page without a finding in this run) with `fixed_ids`, and `gate`. `fixed` also counts entries of modules that did not run this time, and entries of renamed rules.
- Batch only, top level `baseline` (`summary.baseline` internally): `digest`, `known`, `new`, `fixed`, `unmatched_pages` (pages the baseline lists that this run did not audit) and `gate` (`legal_flags`, `severe_legal_flags`, `blocking_issues`, `technical_blockers`, `open_violations`).
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
- **Side checks** (`seo.www_redirect`, `seo.page_health.www_consolidation`, `security.certificate`; the list is `audit::collection::SIDE_CHECKS`) are optional probes about the site around the audited page, not about the page's content: the other host variant (apex / www) and the browser-reported TLS certificate details. When one gets no answer (`alternate_host_unreachable`, `certificate_state_unavailable`) it is a `failed` subcheck with its reason code and an `audit_flags` entry of kind `side_check_unanswered` naming exactly which check is missing. It does not make its module `partial`, does not count in `audit_quality.partial_or_failed_modules` and therefore never turns a fully measured page into a `partial` audit, a provisional rating, a `warn` decision or a failed batch URL. Every other lost collection (headers, links, scripts, scoring collectors) keeps qualifying its module and the audit. The flag stays for the lost measurement; in addition, an other host variant whose probe ran and got no answer (`alternate_host_unreachable`) is the Medium finding `seo.technical.www_alternate_unanswered` (stored as `seo.technical.www_alternate_unanswered`), without a score, severity-count or verdict effect (#899).
- `pages[].navigation` and `pages[].consent` describe the requested/final URL, main-document status, redirect count, bounded page-stability provenance, and whether consent was not detected, detected, dismissed, failed, or remained unknown. Consent evidence never includes cookie values.
- **What the run saw (#1110).** A site that redirects or renders by language, region or consent state can give two runs of the same page different findings. Per page, `pages[].navigation` records `requested_url`, `final_url` and `redirect_chain` (the hops in order, each with the redirecting URL and status), and `accept_language`, the `Accept-Language` header the main document's request carried as the browser reported it for the desktop navigation (absent when the request was not observed; reports from before the field carry none). `pages[].consent` says whether a consent wall was present (`detected`, `cmp`, `audited_content_state`) and is filled for every audited page. The viewports are listed once per run in `audit_scope.viewports`. The languages are set with `--accept-language <TAGS>` (`[audit] accept_language`; default `en-US,en`) and recorded as `audit_scope.measurement_profile.accept_language`; `navigation.accept_language` is the header as it went out (`en-US,en;q=0.9` for `en-US,en`, the browser adds the quality values), so the two can be compared. Neither is the report language (`--lang`), and no setting changes the region a server infers from the IP address.

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
- Lazy frames are loaded before they are audited (#1109). Inside each viewport pass, after the module collection has read the performance observers and before the DOM capture, every rendered `loading="lazy"` iframe that still shows `about:blank` is scrolled into view, the frames are waited for together (3 s at most, whatever the page-stability budget) and the scroll position is restored; the mobile pass navigates anew, so this reaches no performance value. A frame that never loads is not audited as the empty placeholder document: it is listed in `frames.skipped[]` with `reason: "not_loaded"` and its `url` is the source it waits for (`src`, or the `data-src`-style attribute of a lazy-load script on a `loading="lazy"` frame). "Never loaded" means the document is still `about:blank` while the frame has such a source and no `srcdoc`; a frame with `srcdoc`, without a source or with `src="about:blank"` is blank by design and is audited as before. A `not_loaded` frame counts as uninspected like `cross_origin`: the audit quality is `partial` (`uninspected_frames:N`, collection gap `frame_not_loaded`), which is the prompt to review that frame by hand. Its `html-has-lang` is not judged either. Nested lazy frames inside a frame are not scrolled; they are reported `not_loaded` when they stay blank.
- A frame that renders after its DOM was captured is captured again (#1180). In each viewport pass the frame pass compares the non-ignored elements of the frame's accessibility tree (text runs, list markers and the root left out, and what the browser builds itself below a control: date and number fields, media bars, the popup of a `select`) with the frame's DOM capture. Without an element the capture lacks it takes no further call. With one, it waits 0.25 s, captures the DOM and layout again (the measurements on the page itself stay those of the first capture), reads the frame's AX tree again and compares again, within 3 s in all. The frame is settled when nothing lags any more, or when the same elements lag in two readings in a row (a mismatch that is no lag, for example content the DOM capture never lists); the DOM rules then run against the last reading, and the frames nested in the frame are read from it. A frame whose lagging elements keep changing until the budget is spent, or that cannot be captured again, is audited all the same but carries `gap: "dom_unsettled"` in its `frames.audited[]` entry (the entries of a frame with and without a gap stay apart per viewport). Such a frame counts as unsettled like an uninspected one: the audit quality is `partial` (`unsettled_frames:N`, collection gap `frame_dom_unsettled`), which is the prompt to review that frame by hand. The main document is not captured again.
- `audit_scope.rule_filter` records the `[rules]` filter (`disabled_rules`, `enabled_only_rules`) the run applied; it is absent when none was set. A tree or inline rule the recorded filter removes that still has a ran outcome fails validation as `rule_outcome_filtered`.
- `audit_scope.requested_subchecks` lists the collections every requested extra check must leave: third-party isolation and the SSR gap (one each) and the privacy first visit (`privacy.first_visit`, only with the opt-in `--privacy-check`). Every id needs at least one entry in `module_runs[].subchecks`; otherwise validation fails as `requested_subcheck_outcome`. The extra-reload checks record after `audit_page` returns, so the pipeline applies this check to the finished report (single URL and batch page alike), not to the intermediate one; output validation always applies it.
- **State axis.** The state a result was measured in is the measurement profile: the session policy, `state_id` and the `storage_state` digest. There is no separate per-state axis in the execution record; a different profile is another measurement (see below), and one report is one profile.
- `audit_scope.measurement_profile` states how the pages were measured: `version` (meaning of the profile), `session_policy` (`isolated` or `shared`; `unknown` for reports from before the field), `read_only`, `extra_checks` (flag names of the opt-in checks the run requested, e.g. `--check-ssr-content`, `--privacy-check`; sorted, absent when none) and `state_changing_journeys` (`form_error_submit`, `add_to_cart`, `quantity_stepper`; listed only with `--interactive full` and `--allow-state-changes`).
- `state_id` and `storage_state` (`sha256`, `cookies`, `origins`) name a declared browser state (`--storage-state`, `--state-id`). Values never enter the report. A different state file is a different `sha256` and so another measurement.
- `measurement_profile.browser` names the browser that measured (profile version 3): `product` (`chrome`, `chrome_for_testing`, `headless_shell`, `edge`, `chromium`, `ungoogled_chromium`, `unidentified`), `version`, `origin` (`system`, `user`, `path`, `managed`, `explicit`), `os` and `arch` (`macos`/`linux`/`windows`, `x86_64`/`aarch64`), `headless_mode` (`new`, `legacy`, `headful`), and `failover` with `discarded_candidates` (candidates rejected or failed ahead of the one that measured; `failover` is true iff the count is above 0). It holds categories only: no executable path, user name or launch flag, which are personal and not reproducible. A profile of version 3 or later without it, with an empty version, with a managed product from a non-managed origin (or the reverse), with `failover` contradicting the count, outside the platform contract (macOS ARM64, Linux x86_64/ARM64, Windows x86_64 with Edge only), or disagreeing with `execution_environment` on headless mode or version fails as `browser_provenance`. Reports of profile version 2 and older carry none and stay readable.
- `measurement_profile.accept_language` is the plain list of language tags the browser was launched with (`--accept-language`, default `en-US,en`; profile version 4). A profile of version 4 or later without it, or with a list that is not one to eight comma-separated language tags (a space, a `;q=` quality), fails as `measurement_profile_accept_language`. Another list is another measurement: `profile_differences` names it as `accept_language`, a replay can assert it, and it is part of the cache signature. Reports of profile version 3 and older carry none and stay readable.
- A replay compares the browser by identity (product, version, platform, headless mode); how it was found and a failover are provenance, not a different measurement (`profile_differences` field `browser`).
- A batch lists every distinct browser provenance of its pages in `summary.measurement_browsers` (`browser`, `pages`; `browser: null` counts pages whose browser was not recorded). Two or more entries mean the batch mixes browsers (cache replays of another run, merged files); it is never averaged away, and a stored list that differs from the pages fails as `batch_aggregation`. The top-level `audit_scope` of a batch shows the first page's profile only.
- A result measured under another profile is another measurement. Validation rejects a profile whose `read_only` or journey list contradicts `interactive_mode`.

## Site quality score (optional modules)

The mixed score is named **site quality (optional modules)**: the accessibility module is always part of it, the other modules (performance, SEO, security, mobile, HTML conformance) are optional and enter with their weight when they ran and measured. The accessibility score stays the headline everywhere (terminal, JSON `summary` ordering, both PDF covers); the site-quality figure is a labelled secondary number (batch cover: the note under the score row; single cover: the strip "MODULES IN THE SITE-QUALITY SCORE").

Naming rule: every key, label and id that names this figure says `site_quality`, nothing else changed. Identifiers named `overall` that mean something else keep their name: `performance.score.overall` (the Performance module's own score), `signal_strength.overall_pct` (SEO content profile), `overall_impact` (the three-row impact table of the interpretation) and the `single-report-overall-verdict` / `single-report-dual-overall` labels (the verdict; the blended accessibility score).

`schema_version` is `2.1` (was `2.0`; the technician index is `1.1`, was `1.0`). The 2.0 release changed the JSON without moving the envelope version, which stayed at the `2.0` it got in 0.17; this rename is the first non-additive change after that and the contract above asks for a versioned change, so the minor part moves. The major part stays `2`: the envelope is the same generation. There is no alias, the old keys are gone in every output (JSON, summary feed, technician index, Studio contract). Cached snapshots and cache entries from before are misses (`SNAPSHOT_SCHEMA_VERSION` 3, cache `fmt` 34); `EVALUATION_VERSION` is unchanged because no score, finding or verdict moves.

| Old key | New key | Where |
| --- | --- | --- |
| `overall_score` | `site_quality_score` | `summary`, `pages[]` (single and batch JSON), Studio contract, technician index, AI report |
| `overall_score_weight_basis` | `site_quality_score_weight_basis` | `summary` |
| `overall_score_weight_basis_uniform` | `site_quality_score_weight_basis_uniform` | batch `summary` |
| `module_scores[].contributes_to_overall` | `module_scores[].contributes_to_site_quality_score` | JSON, Studio contract |
| `detail.viewport_scores.desktop.overall`, `.mobile.overall` | `.desktop.site_quality`, `.mobile.site_quality` | dual-viewport runs |
| `detail.viewport_scores.weighted_overall` | `detail.viewport_scores.weighted_site_quality` | dual-viewport runs |
| `score_breakdown.desktop_overall`, `mobile_overall`, `viewport_blended_overall` | `desktop_site_quality`, `mobile_site_quality`, `viewport_blended_site_quality` | `pages[].score_breakdown` |
| `overallScore` | `siteQualityScore` | compact summary feed (`--format summary`) |
| validation code `overall_score` | `site_quality_score` | `ReportContractError.violations[].code` |
| `metric_context` id `overall_score` | `site_quality_score` | `metric_context` block |

PDF and table labels follow the name (`Site quality (optional modules)` / `Website-Qualität (optionale Module)`); the JSON stays English.

## One axis for the accessibility score and its components

The accessibility score has one set of components: the points each area cost. They add up to the headline, `100 - sum(estimated_lost_points) = accessibility_score`, in the JSON (`summary.accessibility_score_breakdown[]`) and the PDF (the accessibility callout of the management page) alike; the terminal table shows no components. There is no second 0-100 scale per area.

Until schema 2.2 the report carried two other sets next to it, both the accessibility scorer re-run on a *slice* of the findings: a per-area `score` in the breakdown ("what the page would score if this area were its only problem") and `detail.accessibility_subcategory_scores` (the same on the seven taxonomy subcategories, which the PDF printed as its sub-scores). The scorer is non-linear (diversity factor, compression above the knee, soft floor, severity cap), so slices do not add up to the whole: a page scoring 75 with four areas losing 7, 7, 9 and 2 points showed those areas as 89, 89, 89 and 99, i.e. 11, 11, 11 and 1 points lost, 34 in total against the 25 the page lost. The loss of the whole is the authority; the areas reconcile to it.

`schema_version` is `2.2` (was `2.1`; the technician index is unchanged). Stored snapshots stay valid (`SNAPSHOT_SCHEMA_VERSION` 3, `EVALUATION_VERSION` 8: no score, finding or verdict moves for unchanged input); cache `fmt` 35 turns cached results into misses so they are re-rendered. There is no alias.

| Old key | New | Where |
| --- | --- | --- |
| `summary.accessibility_score_breakdown[].score` | removed; read `estimated_lost_points` (area share of the lost points) and `weight_pct` (the same share in percent) | single and batch `summary` |
| `pages[].detail.accessibility_subcategory_scores` | removed; the components are `summary.accessibility_score_breakdown[]` | single report detail |

`pages[].detail.security_category_scores` is unchanged: it breaks down the Security module's own score, not the accessibility score.

## Validation codes

A report that contradicts itself is rejected with machine-readable codes (`ReportContractError.violations[].code`, `audit::CONTRACT_CODES`). The codes, by area:

- Baseline: `baseline`.
- Scores and decision: `score_range`, `grade`, `site_quality_score`, `average_score`, `score_method`, `module_grade`, `module_weight`, `module_identity`, `weighted_measurement`, `viewport_score`, `semantic_score_cap`, `gate_score_cap`, `passed`, `decision`, `decision_contract`, `decision_profile`, `cause_charged_twice`.
- Findings and counts: `finding_identity`, `finding_inventory`, `finding_occurrences`, `occurrence_counts`, `severity_counts`, `rule_findings_preserved`.
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

## Sitemap diagnostics: normalised URLs

`sitemap_diagnostics` compares sitemap URLs, audited URLs and link targets, so the URL lists it computes from that comparison print the normalised spelling, not the spelling of `pages[].url` (#1111): `orphan_sitemap_urls`, `linked_not_in_sitemap`, `crawl_depths.start_url`, `crawl_depths.deepest_pages[].url` and `crawl_depths.unreachable_pages`. The audited page `https://example.com/agenturen/` therefore appears as `https://example.com/agenturen` there. `http_issues[].url` and `robots_conflicts[].url` name the sitemap entry as written in the sitemap.

The normalisation rule (`normalize_url` in `src/audit/batch.rs`) is: parse the URL, drop the fragment, and drop a trailing `/` from the path unless the path is `/` itself. Scheme, query and the serialised form of host and port (lower-case host, default port omitted) are kept, so a bare origin `https://example.com` becomes `https://example.com/`. A reader who joins these lists with `pages[].url` by string normalises `pages[].url` the same way first. The counts are unaffected, because every comparison inside the diagnostics normalises both sides.

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
average of the canonical page accessibility scores over the URLs that produced a report (`summary.coverage.succeeded`, equal to `summary.url_count`). A batch in which no URL produced a report has no average: `summary.accessibility_score` and `summary.site_quality_score` are `null`, never `0`, and the decision is `fail` (`no_url_audited`).

<a id="site-quality-score"></a>
### site_quality_score — `summary.site_quality_score`

Site quality (optional modules): weighted score across contributing measured
modules, secondary to `accessibility_score`. In a dual-viewport
audit, viewport module results are blended first and Security contributes
10% when available. In batch reports this is the average of the page-level
site-quality scores across the audited URLs.

<a id="grade"></a>
### grade — `summary.grade`

Letter grade banded from `summary.accessibility_score` — the subject of the
report. It is a band label for the automated scope, not a conformance statement.

<a id="module-score"></a>
### module_score — `pages[].module_scores[].score`

Module-specific 0-100 score. Scores from different modules use different
measured or heuristic inputs and are not interchangeable raw measurements.

The Mobile score does not charge touch targets or a locked zoom: WCAG 2.5.8 (24 px, with spacing exceptions) and 1.4.4 are charged once, on the Accessibility side. Targets of 24-43 px are reported as an ergonomic recommendation (44 px) and are not scored anywhere (#976). While any Mobile issue is listed, the Mobile wording and the band label are capped at "Good", whatever the number says; the number and grade are untouched.

<a id="module-dimension-score"></a>
### module_dimension_score — `pages[].detail.modules.*.score` and nested dimension scores

Module- or dimension-specific 0-100 score; higher is better. The surrounding
`module` and `measurement_type` identify whether the value is measured or
heuristic.

<a id="viewport-scores"></a>
### viewport_scores — `pages[].detail.viewport_scores`

Desktop and mobile accessibility/site-quality scores on the 0-100 scale
(`desktop.site_quality`, `mobile.site_quality`).
`weighted_site_quality` is the 70% mobile and 30% desktop module blend before the
optional Security contribution. It is a viewport-pass figure, not a published
module weight and not the report's `site_quality_score` (modules weighted per the
published weights, Security added afterwards); read `site_quality_score` for the
score of the page.

<a id="subcategory-scores"></a>
### subcategory_scores — `pages[].detail.security_category_scores`

Security score per category on the 0-100 scale; higher is better. Each entry
is `{ name, score }` with a canonical English `name`. Entries are omitted when
the Security module did not run. Single reports only; batch page details omit
the field. The accessibility score has no per-subcategory score set; its
components are `summary.accessibility_score_breakdown[]`.

<a id="named-module-score"></a>
### named_module_score — `summary.*_score` / `pages[].*_score`

Named module score on the 0-100 scale; higher is better. Throttled and
Lighthouse performance scores are lab results, not field/RUM data.

<a id="site-quality-score-weight-basis"></a>
### site_quality_score_weight_basis — `summary.site_quality_score_weight_basis`

Sum of the module weights that actually fed `site_quality_score`, out of 100.
Below 100 the score was renormalised over the modules that ran and measured
successfully — a module that did not run, or that ran without being able to
measure, changes the denominator rather than the result. A score computed on a
different weight basis is not directly comparable.

In a batch report the value is the **lowest** basis across the audited pages,
so it never overstates how much of the model backs the average, and
`site_quality_score_weight_basis_uniform` says whether the pages agreed. `false`
means `site_quality_score` values from different module sets were averaged.

<a id="accessibility-score-breakdown"></a>
### accessibility_score_breakdown — `summary.accessibility_score_breakdown[]`

The components of `summary.accessibility_score`, one entry per area, on one
axis (see "One axis for the accessibility score and its components" above):

- `estimated_lost_points` is the area's share of the points the score actually
  lost, apportioned by the area's share of the raw per-rule penalty (largest
  remainder). Across all areas it sums to exactly
  `100 - summary.accessibility_score`.
- `weight_pct` is the same share expressed in percent (summing to 100 whenever
  anything was lost).
- `main_driver` names the area's most frequent finding.

The headline is the authority and the areas reconcile to it. There is no
per-area 0-100 score: the scorer's diversity factor, compression curve, soft
floor and severity cap are non-linear and depend on the finding set as a
whole, so a score computed for one area alone does not add up to the whole.
`auditmysite-dev report-lint` checks the sum
(`accessibility_breakdown_reconciles_with_score`).

<a id="legal-relevance"></a>
### One definition of legal relevance

A finding is *legally relevant* when it is **confirmed** (`verification` is not `manual_review_recommended`, so a finding that needs a manual check raises none), **occurs** (`occurrence_count > 0`) and its `bfsg_relevance` is `high` or `medium`. The code is `audit::normalized::is_bfsg_relevant_open`; nothing else decides it. `bfsg_relevance` is derived from the rule alone: `high` for a Level A criterion at High or Critical severity, `medium` for any other Level A or Level AA criterion that has a normative clause in EN 301 549 V4.1.1 (clauses 9.1-9.4, WCAG 2.2 A/AA), `low` otherwise (AAA criteria, the retired 4.1.1 Parsing, convention rules such as `display/*`). A Medium or Low 1.4.3 contrast finding is therefore legally relevant as long as it is confirmed.

**The count** every surface shows reads this one predicate, so they cannot disagree: `pages[].risk.legal_flags`, the batch `summary.legal_flags` (distinct rules across the pages), `legal_flags` of the baseline gate (`pages[].baseline.gate`, `baseline.gate`; the findings that are not `known`), the number in the verdict reason (`legal_flags: n`, "n findings with possible legal relevance"), the executive key point, the Legal / BFSG-EAA management card, the screen-reader BFSG callout and the "BFSG/EAA relevance" row of a finding.

**The consequences** read the same predicate with one stated severity bar: `severe_legal_flags` (`pages[].risk`, batch `summary`, the baseline gates) counts the legally relevant findings of severity High or Critical (`NormalizedFinding::is_severe_legal_flag`). Only these fail the verdict (`fail_on_legal_flags`), add the 20 points of the risk score, drive the risk level (the `legal_flags` terms of the table below read it), tier the Legal card High and keep the verdict line from a reassuring tier. A relevant finding of Medium or Low severity is shown and counted but does not by itself fail the verdict or raise the risk; the Legal card is then `medium`. `severe_legal_flags <= legal_flags` always; a failing verdict names `legal_flags` (`legal_flags: n`), the number the card prints, never the smaller severe count.

The screen-reader signal described below is the only addition: it raises both counts to at least 1 and has no finding id. Under `--baseline` the measured counts (`risk.legal_flags`, the card, the callout) stay as measured; the gate and the decision read the findings that are not known.

<a id="management-risks"></a>
### management_risks — `summary.management_risks[]`

The dimensions are `Legal / BFSG-EAA`, `Assistive technology use`, `Performance & mobile` and `Project risk` in every run. Three more appear only with the findings that justify them, because a dimension that restates a module score adds nothing the score on the cover does not already say (#1139):

- `Conversion / usability`: only with blocking controls (`risk.blocking_issues` above 0). The rationale names the controls and the consequence, no scores.
- `SEO / visibility`: only with at least one open SEO finding of Medium severity or higher. The rationale gives the finding count next to the SEO score; the level is never `low`.
- `Trust / brand`: only with at least one critical or high WCAG finding. It is inferred from accessibility alone and says so.

A consumer must not assume a fixed list of dimensions: before this change all seven were always present.

`Project risk` counts only findings of Medium severity or higher (#1039): in a batch the rules that recur on two or more pages (three or more such rules are `high`), in a single report the findings with 10 or more occurrences or a structural fix. Low findings, such as fonts not preloaded or static resources without efficient caching, no longer raise it. The values of `summary.management_risks` move for unchanged input; no field is added or removed, so `schema_version` stays, and no score, finding or verdict moves, so `EVALUATION_VERSION` stays (the risks are derived from the normalized report when the JSON is built, not stored in snapshots or the cache).

<a id="score-gain"></a>
### score_gain — `summary.top_actions[].score_gain`

What an action is worth on the accessibility score (#1169), computed by simulation, not estimated: the scorer runs again on the page's violations without the violations of the action's rule, through the same steps as the page score (per-criterion penalties, diversity factor, compression, soft floor, the 49/89/92 caps, and in a dual-viewport audit the 70 % mobile / 30 % desktop blend with each violation in the viewport passes its `desktop-only` / `mobile-only` / `both-viewports` tag names). `accessibility` is that score minus `summary.accessibility_score`, in whole points.

- **Marginal, not additive.** Each figure answers "if only this action is done, against the findings as they are". The caps, the diversity factor and the compression depend on the finding set as a whole, so the gains of two actions do not add up to the gain of doing both: two critical findings each read `0` while the other keeps the cap at 49, together they lift it. There is no combined figure.
- `capped_at` (single report only) is the cap that still bounds the simulated score because other critical or high findings remain; absent when no cap binds. A `0` with `capped_at: 49` means the action does not move the score until the other critical findings are fixed too.
- `basis` is `simulated` for one page and `simulated_mean` in a batch, where each page is simulated without the rule (pages without it keep their score) and the gain is the rounded average of those scores minus the rounded `summary.accessibility_score` (the average of the page scores).
- **Absent means not quantified:** the action's finding is not a WCAG finding (SEO, performance, security, mobile, HTML conformance, interactive checks: nothing of the accessibility score), no violation of the page carries the rule, or the recomputation of the unchanged violations does not reproduce the stated score (then no gain is stated for the page, and a batch action on that page has none either). `expected_impact` then ends in "Score effect not quantified." as before.
- `expected_impact` states the same figure in words ("If only this action is done: up to +N points on the accessibility score (simulated)."); "up to" because the gain assumes every occurrence of the rule is fixed. The PDF shows the sentence under "Expected effect" of a finding and the figure in the "Score effect" column of the batch top actions; the terminal table shows neither.
- The simulation reads only persisted data (the merged violations with their viewport tags), so a replay of a snapshot states the same gains as the fresh run. No score, finding or verdict moves.

Compatibility: additive. `score_gain` is optional on `summary.top_actions[]` in both schemas; the same object sits in `pages[].findings[].expected_impact_kind.Wcag.score_gain`, and the expected-impact sentence of a quantified finding replaces "Score effect not quantified." with the gain. `schema_version`, `SNAPSHOT_SCHEMA_VERSION` and `EVALUATION_VERSION` stay; cache entries need no `fmt` change because a cache hit is answered by replaying the snapshot, which computes the gains.

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
    20 * severe_legal_flags
  + 10 * critical_issues              (WCAG, per occurrence)
  +  3 * high_issues                  (WCAG, per occurrence)
  +  2 * blocking_issues              (per element)
  + 10 * interactive_critical_issues  (journey findings)
  +  5 * interactive_high_issues
  +  3 * screen-reader findings of severity high
  +  1 * screen-reader findings of severity medium)
```

- `legal_flags` counts *distinct rules* on the page, not occurrences (1000
  images without alt text are one flag); `severe_legal_flags` is its subset at
  High or Critical severity, and the formula and the level conditions below
  read that subset (see [One definition of legal relevance](#legal-relevance)).
  A page also carries at least one flag when the screen-reader audit confirms a
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
| `critical` | `severe_legal_flags >= 3` (breadth: at least three distinct legally relevant rules at High/Critical) or `critical_issues >= 5` (volume) |
| `high` | `severe_legal_flags > 0` together with `critical_issues > 0`, or `critical_issues >= 3`, or `blocking_issues >= 5`, or `risk_score >= 80` |
| `medium` | `high_issues >= 3` with accessibility score below 80, or any critical issue, severe legal flag, blocking issue, interactive critical/high finding, or accessibility score at most 20 |
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

Each criterion of a principle is in exactly one of five states, counted per principle next to the ratio (#1135): `passed` (ran, found nothing), `violation` (a violation is retained), `review` (only a warning or heuristic hint, a human decides), `not_testable` (a check errored, was switched off, lacked a capability, or reported a not-testable item) and `not_applicable` (nothing to check on this page). `total` is `passed + violation + review + not_testable`; `not_applicable` is outside it. A criterion keeps its most serious state: violation, then review, then not testable, then passed. So "11 of 12 passed" with no violation means one criterion is in `review` or `not_testable`, and the score, which ignores warnings, stays where it is. Before the five counts only `passed`, `total` and `ratio` existed; a stored report without the counts reads them as 0.

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
