---
title: "Migrating from 2.1 to 2.2"
description: "Every breaking change of auditmysite 2.2 in one table: retired and renamed rule ids, findings baseline and [rules] filters, and the JSON members the 2.1 schemas reject."
order: 15
---

auditmysite 2.2 carries breaking changes inside the 2.x line, as 2.1 did. Most of them come from
moving rules to the shared barrierlab rules (`a11y-rules` 0.31): a rule id changes, so the finding
id derived from it changes, and a findings baseline or a `[rules]` filter written for 2.1 can name
an id that no longer reports. The rest are JSON members added to objects that the 2.1 schemas
close. The changelog has the same entries under `### Breaking` in the
[2.2.0 section](https://github.com/casoon/auditmysite/blob/main/CHANGELOG.md); the contract behind
them is the [output contract](https://github.com/casoon/auditmysite/blob/main/docs/OUTPUT_CONTRACT.md)
and the JSON schemas in `docs/`. The changes of 2.1 are in [Migrating from 2.0 to 2.1](../migration-21/); later breaking changes
(the experimental Cargo feature, the unstable library API) are in
[Migrating from 2.2 to 2.3](../migration-23/).

## How to find what still reads the old shape

- **Findings baseline** (`--baseline`): a finding id is the hash of the rule id alone
  ([finding id stability](https://github.com/casoon/auditmysite/blob/main/docs/OUTPUT_CONTRACT.md#finding-id-stability)),
  and `FINDING_ID_SCHEME` stays 1, so a 2.1 baseline is still accepted. Ids of the moved rules read
  as `fixed` under the old rule and the same elements as `new` under the new one, which can fail
  the gate. Run 2.2 once with `--baseline old.json`, check that the `new` findings
  (`pages[].findings[]` without `known`) are the moved rules of the table below, then accept them
  with `--baseline old.json --write-baseline new.json`.
- **`[rules]` filters** in `auditmysite.toml` (`disabled`, `enabled_only`, and `ignore`, which is
  read as `disabled`): an old id that has a shared successor keeps reaching it, an id whose findings
  moved to a new rule does not. The rule table says which.
- **JSON consumers** that validate against `docs/json-report.schema.json` or
  `docs/json-batch-report.schema.json`: take the 2.2.0 schemas. `schema_version` stays `2.2`
  (2.0 for 2.0.x reports; the technician index is `1.1`), because every change is an addition.

Stored snapshots and cache entries written by 2.1 are cache misses (cache `fmt` 37); nothing needs
to be deleted.

## Rule ids, baseline and `[rules]` filters

"Baseline" says what a 2.1 baseline sees in a 2.2 run. "`[rules]` filter" says what an entry with
the old id does in 2.2.

| Old rule id (2.1) | New rule id (2.2) | Baseline | `[rules]` filter | What to change |
| --- | --- | --- | --- | --- |
| `image-alt` (retired, #1181) | `images/role-img-name-missing` for any element with `role="img"` that is neither `<img>` nor SVG; `images/alt-missing` for `<img>`; `svg/name-missing` for `<svg>` | `image-alt` reads as `fixed`; the elements show up as `new` under the new ids (an `<img>` or `<svg>` that was already reported by `images/alt-missing` or `svg/name-missing` stays `known`) | `image-alt` reaches `images/role-img-name-missing` | Replace `image-alt` by `images/role-img-name-missing`, and add `images/alt-missing` if the filter was meant for `<img>` too. |
| `names/required-missing` for `role="img"` elements | `images/role-img-name-missing` (1.1.1) | `new` under the new id; `names/required-missing` reads as `fixed` on a page where only such elements carried it | `names/required-missing` no longer reaches these elements | Add `images/role-img-name-missing`. |
| `names/required-missing` for `<meter>`, `<progress>`, `role="meter"`, `role="progressbar"` | `names/indicator-name-missing` (1.1.1) | as above | `names/required-missing` no longer reaches these elements | Add `names/indicator-name-missing`. |
| `video-caption` for a `<video>` without a caption or subtitle track | `media/video-captions-missing` (1.2.2), one review per video (#1182) | `new` under the new id; `video-caption` reads as `fixed` on a page where nothing else of it remains | `video-caption` reaches `media/video-captions-missing` as well as `video-caption` | Nothing, unless the filter should keep the browser checks of `video-caption` and drop only the no-track case: then name `media/video-captions-missing`. |
| `description-duplicates-name` | `names/description-duplicates-name`, a review (#919) | `fixed` under the old id, `new` under the new one | `description-duplicates-name` reaches `names/description-duplicates-name` | Replace the old id when convenient. |
| `contrast/text-insufficient` for text inside an `aria-hidden` svg illustration | `contrast/text-undetermined`, a review (#1112) | `new` under `contrast/text-undetermined` where the page had no such review before; `contrast/text-insufficient` reads as `fixed` where only these occurrences carried it | A filter on `contrast/text-insufficient` no longer reaches these occurrences; `color-contrast` reaches both ids | Add `contrast/text-undetermined` if the filter should keep covering them. |

## Migration table

"report-lint code" names the check id that `auditmysite-dev report-lint` raises for the change. A
dash means none exists. Entries are in the order of the changelog.

### Rules and scoring

| Old | New | report-lint code | Reason |
| --- | --- | --- | --- |
| `image-alt` reported elements with `role="img"`, `<img>` without a URL in the accessibility tree, SVG elements (Refs #1165) and CSS-generated images at the selector `document`; an empty `aria-label` marked an image decorative, and an icon inside a named link or button was exempt | Retired. One rule per image element: `images/alt-missing`, `svg/name-missing`, `images/role-img-name-missing`. Neither exception exists in ACT 23a2a8, so both cases are reported; CSS-generated images are not checked (casoon/barrierlab#273) (#1181) | - | The rule duplicated the shared rules and reported one element twice. |
| `names/required-missing` (4.1.2, ARIA area of the score) for unnamed `role="img"` elements and meters or progress bars | `images/role-img-name-missing` and `names/indicator-name-missing` (1.1.1, images area of the score); neither is a blocking control in `summary.management_risks`. `names/required-missing` keeps the widget roles (#1181) | - | A missing text alternative of non-text content is 1.1.1, not 4.1.2. |
| `video-caption`: a page-level untested note for videos without a caption track | `media/video-captions-missing`: one review per video, no score cost; a muted video without `controls` and a video under `aria-hidden` are left out; a `<track>` without `kind` counts as subtitles. `video-caption` keeps the browser checks (does a present track load same-origin as a caption file, third-party players) and reports a video whose tracks all fail on the page origin as a review (#1182) | - | No video is reported by both rules. |
| `description-duplicates-name`, a 4.1.2 failure with a score cost | `names/description-duplicates-name`, a review: no score cost, no WCAG occurrence (#919) | - | A description that repeats the name is redundant, not a failure of name, role and value. |
| Contrast below the threshold inside an `aria-hidden` svg illustration: `contrast/text-insufficient`, a failure | `contrast/text-undetermined`, a review (#1112) | - | Pure decoration is exempt from 1.4.3; only a person can tell whether the text carries information. |

### JSON

| Old | New | report-lint code | Reason |
| --- | --- | --- | --- |
| `summary.top_actions[]` without a score effect (closed object in the 2.1 schemas) | `summary.top_actions[].score_gain` (`accessibility`, `basis`, `capped_at`): the simulated accessibility score gain of the action, marginal and not additive (#1169). Details: [score_gain](https://casoon.github.io/auditmysite/docs/OUTPUT_CONTRACT/#score-gain) | - | An action states what fixing it is worth. |
| `audit_scope.measurement_profile` version 3 (closed object) | Version 4 with `accept_language`, the languages set with `--accept-language` (default `en-US,en`) (#1110) | - | A reference run can pin the browser languages; they are part of the cache signature and the replay identity. |
| `pages[].principle_coverage.*` with `passed`, `total`, `ratio` (closed object) | Also `violation`, `review`, `not_testable`, `not_applicable` (#1135) | - | "11 of 12 passed" next to no violation explains itself. |
| `execution.frames.skipped[].reason` without `not_loaded` (closed enum); a lazy iframe that never loaded was audited as its empty placeholder | `not_loaded`: the frame is listed as skipped and the audit is `partial` (`uninspected_frames:N`, collection gap `frame_not_loaded`) (#1109) | - | An empty placeholder is not the frame's content (basf.com: `html-has-lang`, up to 15 points). A `partial` audit cannot become an unqualified clean verdict. |
| `findings[]` without origin | `findings[].origin` (`first_party`, `same_site_frame`, `cross_origin_frame`, `mixed`), `findings[].providers`, and `origin` and `provider` on occurrences inside a frame (#1109). Scores, verdicts and finding ids are unchanged | - | A finding in a third-party frame is fixed at the provider. `findings[]` is an open object in the schemas, so only consumers with their own closed schema break. |

`pages[].navigation.accept_language` (#1110) and `dialog_h1_count` in the SEO heading data (#1132)
are added too; the schemas do not close their objects, so they need no migration.

## Not breaking, but visible

- **Finding titles** (#1013): the `manual/*` rules and the hint-only rules are titled as a check
  prompt, and six rules have a new `title` in JSON, PDF and table (`video-caption`,
  `modern-attribute-misuse`, `aria/reference-missing`, `forms/instructions-missing`,
  `forms/required-unmarked`, `forms/group-name-missing`). Finding ids do not change; a consumer that
  matches titles must read `rule_id`.
- **Values that move for unchanged input:** `summary.management_risks` counts only Medium or higher
  findings for project risk and lists conversion, SEO and trust only with findings behind them
  (#1039, #1139); the verdict text caps at "WCAG violations were found" (#1131); an H1 inside a
  dialog or consent layer no longer counts as the page's H1 (#1132).
- **Browser languages:** every request of the audited page sends `Accept-Language: en-US,en` unless
  `--accept-language` says otherwise; 2.1 sent a malformed list in a single run and none on the
  mobile pass (#1110). A site that redirects by language can show other content than under 2.1.

## Related

- [Migrating from 2.2 to 2.3](../migration-23/)
- [Migrating from 2.0 to 2.1](../migration-21/)
- [Migrating to 2.0](../migration-20/)
- [Output contract](https://github.com/casoon/auditmysite/blob/main/docs/OUTPUT_CONTRACT.md), including the finding id stability and the findings baseline.
- [Rule overview](https://casoon.github.io/auditmysite/rules/) with a page per rule id.
