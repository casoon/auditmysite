---
title: "Migrating to 2.0"
description: "Every breaking change of auditmysite 2.0 in one table: old field, new field, report-lint code and the reason."
order: 13
---

auditmysite 2.0 is a major release with breaking changes to the JSON report, the summary feed,
the Studio contract and the library API. The changelog lists each of them in the
[2.0.0 section](https://github.com/casoon/auditmysite/blob/main/CHANGELOG.md#200---2026-11-04);
this page collects them in one place. The contract behind them is the
[output contract](https://github.com/casoon/auditmysite/blob/main/docs/OUTPUT_CONTRACT.md) and the
JSON schemas in `docs/`.

## How to find what still reads the old shape

Run `auditmysite report-lint <JSON_FILE>` on a report your tooling consumes. Only a few changes have
a lint check of their own; they are named in the table. The other columns work from the schema:
validate your consumers against `docs/json-report.schema.json` and
`docs/json-batch-report.schema.json`.

Cached snapshots written under `auditmysite.decision/1` are cache misses.

## Migration table

"report-lint code" names the check id that `report-lint` raises for the change. "contract code" is a
code of the report validation (`ReportContractError`, listed in the output contract under
"Validation codes"). A dash means neither exists for this change.

| Old | New | report-lint code | Reason |
| --- | --- | --- | --- |
| JSON `verdict`, `verdict_reasons` | `decision` with `contract` (`auditmysite.decision/2`), `profile`, `verdict`, `certificate`, `reasons` | contract codes `decision`, `decision_contract`, `decision_profile` | One versioned decision under the stored `[verdict]` profile is shown by terminal, JSON, summary feed, Studio, PDF, batch counts and exit code alike; before, only the exit code honoured a custom profile. |
| JSON `summary.certificate`, `pages[].certificate`; Studio `certificate` fields | `decision.certificate`, a language-free kind: `very_good`, `good`, `stable`, `needs_improvement`, `insufficient`, `restricted` or `not_passed`. German label to kind: `GUT` is `good`, `EINGESCHRÄNKT` is `restricted`, `NICHT BESTANDEN` is `not_passed` | `certificate_matches_accessibility_score`, `pdf_certificate_traceable_to_json` (contract code `certificate`) | The certificate is derived once, from the accessibility score and the risk veto, instead of once per output. |
| JSON `summary.score` (alias of `overall_score`) | `summary.accessibility_score` for the headline, `summary.overall_score` for the weighted value | `score_alias_removed` | The alias read as the overall score next to an accessibility grade. |
| Batch JSON `summary.verdict_key` | `summary.score_band_key` | - | It is a narrative score band, not the decision. |
| Batch JSON `total_violations`, `violated_rule_count` | Same names; they now count WCAG only, like the single report | - | Single and batch reports count the same thing. |
| Summary feed `score` | `accessibilityScore` and `overallScore`; `verdict` and `verdictReasons` are added | - | `score` is split into the two scores it could mean. |
| Summary feed `grade`, `label`, `medalColor` (banded from the overall score); `ruleTypes` | Bands of the accessibility score; `ruleTypes` counts WCAG rules | - | Grade and medal describe accessibility, not the weighted mix. |
| Studio response and history entries without `decision` | They carry `decision` | - | Studio shows the same decision as the other outputs. |
| Studio `executive_summary.verdict` | `executive_summary.score_band_sentence` | - | The sentence is a score band, not the decision. |
| Config `[rules] disabled = ["keyboard"]` for the 2.1.2 keyboard-trap check | `[rules] disabled = ["keyboard-trap"]` (the finding id) | - | The check runs under its own finding id; the old entry no longer matches it. |
| JSON `top_actions[].root_cause` (free text) | `top_actions[].finding_id`; in batch reports also `root_cause_id`, which points into `summary.root_causes` | - | The free text called anything with 10 occurrences or two pages a "likely shared component". |
| JSON `screen_reader.bfsg_compliance.verdict` with `compliant` / `non_compliant` | `no_violations_found` / `violations_found` / `incomplete`; new `interpretation.accessibility_claim` (result, scope, open manual review); both state `conformance` and `legal_applicability` as `not_determined` | - | Statements no longer claim conformance; they give result, scope and open manual review. |
| JSON `summary.template_clusters`, `summary.top_recurring_rules`, `internal_comparison.root_causes` | `summary.root_causes`, with the evidence grade of each pattern | - | The removed lists duplicated it or called anything on two pages a "likely template or component". |
| Library `audit::Baseline`, `BaselineDiff`, `BaselineViolation`, `WaivedViolation` | Removed without replacement; a baseline comparison will build on the stable finding and pattern ids | - | The API was unfinished, not wired into the CLI, and identified findings by criterion and AX node id. |
| Library `audit::normalize` (infallible) | `audit::normalize_checked`; `normalize` is `#[deprecated]` | - | The checked boundary refuses contradictory data with codes and data paths. |
| JSON `source_quality.substance.score` | Removed; structural signals (word count, headings, structured data, metadata) are listed as observed values without a rating | - | Not marked "Breaking" in the changelog. "Source quality" is an unweighted indicator whose score combines on-page consistency and publisher/security signals. |
| Batch JSON `summary.accessibility_score`, `summary.overall_score` as `0` when no URL produced a report | `null` (schema: `integer \| null`); the denominator is `summary.coverage.succeeded` | - | Not marked "Breaking" in the changelog. A batch without a report has no average. |
| JSON `<module>_score` repeating the subset score of a module with a partial or missing measurement | `null`; the score stays in `module_scores` (marked `partial`) and in the module detail | - | Not marked "Breaking" in the changelog. CLS-only performance used to read 100 (#769). |

## Related

- [Output contract](https://github.com/casoon/auditmysite/blob/main/docs/OUTPUT_CONTRACT.md#decision-contract), including the decision contract and the validation codes.
- [CLI and output contract](../reference/cli-and-output.md) for `report-lint` and the output formats.
