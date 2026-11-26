---
title: "Migrating from 2.2 to 2.3"
description: "Every breaking change of auditmysite 2.3 in one table: the Cargo feature for the experimental indicators, the unstable library API, rule criteria, SEO findings and the JSON members the 2.2 schemas reject."
order: 16
---

auditmysite 2.3 carries breaking changes inside the 2.x line, as 2.1 and 2.2 did. No rule is
renamed and the finding id derivation is unchanged, so a findings baseline written by 2.2 is still
accepted. What changes is the build (six experimental modules sit behind a Cargo feature), the
promise about the Rust API (there is none), the criterion two rules are reported under, two SEO
reference findings that are gone, and one JSON member added to an object the 2.2 schemas close. The
changelog has the same entries under `### Breaking` in the
[2.3.0 section](https://github.com/casoon/auditmysite/blob/main/CHANGELOG.md); the contract behind
them is the [output contract](https://github.com/casoon/auditmysite/blob/main/docs/OUTPUT_CONTRACT.md)
and the JSON schemas in `docs/`. The changes of 2.2 are in [Migrating from 2.1 to 2.2](../migration-22/).

## How to find what still reads the old shape

- **CLI users of `--experimental`**: a default build now refuses the flag with a message naming the
  Cargo feature `experimental`. Install with `cargo install auditmysite --features experimental`.
  Nothing else in a default run changes: the indicators were off by default and in no `--full` run
  before, so the JSON of a default run has the same members.
- **Library consumers**: enable the feature, `auditmysite = { version = "2.3", features = ["experimental"] }`,
  if the code names any item of the table below; otherwise it stops compiling. Do not rely on any
  other `pub` item either: the crate has no stability promise any more.
- **Findings baseline** (`--baseline`): `FINDING_ID_SCHEME` stays 1 and the id of a rule is
  unchanged. The ids of `seo.meta.canonical` and `seo.meta.lang` no longer appear, so a baseline that
  lists them reads them as `fixed`; that cannot fail a gate. Re-write the baseline with
  `--baseline old.json --write-baseline new.json` to drop them.
- **JSON consumers** that validate against `docs/json-report.schema.json` or
  `docs/json-batch-report.schema.json`: take the 2.3.0 schemas. `schema_version` stays `2.2`
  (technician index `1.1`); the change that a 2.2.0 schema rejects is an added member.

Results cached by 2.2 are cache misses (the criterion of a rule is part of the rule-set digest, and
two criteria changed); nothing needs to be deleted.

## Rust library

The Rust API is not part of the stable surface. Only the CLI and the JSON report are; see the
[library reference](https://casoon.github.io/auditmysite/docs/reference/api/). The 2.3.0 changelog
announces library changes under **Library:** at best, in a minor release.

With the Cargo feature `experimental` off (the default), these items do not exist:

| Item | With the feature off |
| --- | --- |
| Modules `ux`, `ai_visibility`, `content_visibility`, `source_quality`, `commerce`; the analysis of `page_flow` (`analyze_journey`, `analyze_journey_with_dom_check`, `JourneyAnalysis`) | Not compiled. `page_flow::PageIntent` stays. |
| Crate-root re-exports `analyze_ux`, `UxAnalysis`, `analyze_ai_visibility`, `AiVisibilityAnalysis`, `analyze_content_visibility`, `ContentVisibilityAnalysis`, `analyze_source_quality`, `SourceQualityAnalysis`, `analyze_journey`, `analyze_journey_with_dom_check`, `JourneyAnalysis` | Gone. `PageIntent` stays. |
| `AuditReport::ux`, `journey`, `commerce`, `experimental` and the builders `with_ux`, `with_journey`; `ViewportAuditData::ux` and `journey`; the `ai_visibility`, `content_visibility` and `source_quality` fields of `DiscoverabilitySection` | Gone, so struct literals and field reads of these names need `#[cfg(feature = "experimental")]` or the feature. |
| `audit::ExperimentalIndicators` | Gone. The enum `ExperimentalIndicator` with the six names stays, so the flag parses and the error can name what was asked for. |
| The add-to-cart and quantity-stepper journeys | Not compiled. |

`pages[].experimental` stays an optional JSON member of the report; with the feature on, its content
is unchanged.

## Migration table

"report-lint code" names the check id that `auditmysite-dev report-lint` raises for the change. A
dash means none exists. Entries are in the order of the changelog.

### Rules and findings

| Old | New | report-lint code | Reason |
| --- | --- | --- | --- |
| `frames/name-missing` reported under WCAG 2.4.1 | Reported under 4.1.2, level A as before (Refs #1119) | - | The shared rule's criterion, as axe `frame-title`; rule id and finding id unchanged. `wcag_criterion` in findings, the criterion lists of the report and the rule page move. |
| `images/server-side-map` reported under WCAG 1.1.1 | Reported under 2.1.1, level A as before (Refs #1119) | - | The shared rule's criterion: a server-side image map cannot be operated by keyboard. Title "Keyboard (Server-side Image Map)", linked to the Keyboard page. |
| `seo.meta_issues[]` entries with `field` `canonical` and `lang`; findings `seo.meta.canonical` and `seo.meta.lang` (references, no score cost) | Gone. `seo.technical.issues[]` keeps `no_canonical` and `no_lang`, where the SEO score charges them; `document/lang-missing` stays the accessibility finding (Refs #939) | - | A missing canonical or `lang` was listed twice. Scores are unchanged. |

### JSON

| Old | New | report-lint code | Reason |
| --- | --- | --- | --- |
| `execution.frames.audited[]` without `gap` (closed object in the 2.2 schemas) | `gap: "dom_unsettled"` on a frame whose DOM did not catch up with its accessibility tree within the retry budget; the audit quality is `partial` (`unsettled_frames:N`, collection gap `frame_dom_unsettled`) (#1180) | - | A frame that keeps rendering was judged on a document older than the rendered one; the report now says so. |
| `findings[]` without `observed_roles` | `findings[].observed_roles`, the tally of AX roles over all occurrences, on `links/name-missing`, `buttons/name-missing`, `names/required-missing` and `forms/label-missing` (#1170) | - | Additive in an open object; the PDF names the element kind from it. |

### Removed

| Old | New | Reason |
| --- | --- | --- |
| `npx auditmysite` (package `packages/npm`) | Release binaries, the container image, `cargo install auditmysite` | The wrapper was never published to npm (Refs #928). |

## Not breaking, but visible

- **PDF layout:** the single report's root-cause table and action-plan cards are one analysis section
  with one card per finding, and the batch report merges the most frequent issues, template causes,
  top actions and action plan into "Recurring problems and actions" (#1171). Finding titles of a
  missing accessible name or label name the element kind and the reach in the PDF; the JSON `title` and finding ids do not change (#1170). A consumer that reads the
  PDF text sees other headings.
- **SEO values that move for unchanged input:** hreflang and JSON-LD are judged by `web-checks`
  (`de-de` and `zh-Hant-TW` are valid codes; a page that names itself only as `x-default` gets
  `hreflang_no_self_reference`; `@graph` with a schema.org `@context` is valid JSON-LD), and
  `http1_only` no longer fires on sites that serve HTTP/2 (#1120, #1191).
- **Contrast:** light text on a dark gradient overlay over a lazy image is no longer reported at
  about 1.6:1 (#1194).

## Related

- [Migrating from 2.1 to 2.2](../migration-22/)
- [Migrating from 2.0 to 2.1](../migration-21/)
- [Output contract](https://github.com/casoon/auditmysite/blob/main/docs/OUTPUT_CONTRACT.md), including the finding id stability and the findings baseline.
- [Library reference](https://casoon.github.io/auditmysite/docs/reference/api/) with the stability policy.
- [Rule overview](https://casoon.github.io/auditmysite/rules/) with a page per rule id.
