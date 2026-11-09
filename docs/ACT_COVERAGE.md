# ACT Coverage

Mapping of internal detector rules to the [W3C ACT rules](https://act-rules.github.io/rules/) (Accessibility Conformance Testing), with a coverage level per ACT rule.

The machine-readable source of truth is [`act-mapping.json`](act-mapping.json); this page explains it and summarises it. The mapping is the shared input for the ACT test harness (#837) and the EARL output (#838). Tracking: #835, #836.

## Source

ACT rule ids, names, types, deprecation status and success criteria are read from `https://github.com/act-rules/act-rules.github.io` (directory `_rules`), commit `50d2bede1ee705be6840d06f356a6ae2fd94cb2c` (2026-10-26). Nothing is taken from memory; the mapping has to be re-checked when that repository changes.

Coverage was judged by comparing the ACT rule's applicability and expectation with the `expectation` text of the internal rule contract in `tests/fixtures/rule_contracts/` (written from the norm, not from a run). It has **not** yet been confirmed against the ACT test cases; that is the job of #837. Until then `full` means "applicability and expectation match on paper".

## Coverage levels

| Level | Meaning |
| --- | --- |
| `full` | The listed internal rules together implement the ACT rule's applicability and expectation. Documented differences are limited to scope notes. |
| `partial` | The internal rules cover a subset, a superset or a differently-judged variant (stricter exception, review instead of fail, heuristic method, different scope). The `note` says what differs. |
| `none` | No internal rule implements this ACT rule. `gap` says why (see below). |
| `unclear` | An internal rule touches the same criterion but the relation to the ACT rule could not be established from the sources. Treated like `none` until resolved. |

Consumers must emit an ACT result (EARL `passed`/`failed`) only for `full` mappings of `active` ACT rules. `partial` and `unclear` never produce an ACT verdict. Heuristic or best-practice checks (#779) are not ACT results even when listed as `partial`.

## File format (`schema_version` 1)

```json
{
  "schema_version": 1,
  "act_source": { "repository": "...", "path": "_rules", "commit": "...", "commit_date": "YYYY-MM-DD" },
  "rules": [
    {
      "act_id": "b5c3f8",
      "name": "HTML page has lang attribute",
      "rule_type": "atomic",
      "status": "active",
      "wcag_sc": ["3.1.1"],
      "coverage": "full",
      "internal_rules": ["document/lang-missing"],
      "related_rules": [],
      "starter": true,
      "note": "..."
    }
  ]
}
```

One entry per ACT rule (all rules of the pinned commit, including deprecated ones). Fields:

| Field | Type | Meaning |
| --- | --- | --- |
| `act_id` | string | ACT rule id (the last path segment of `https://www.w3.org/WAI/standards-guidelines/act/rules/<id>/`). Unique. |
| `name` | string | ACT rule name. |
| `rule_type` | `atomic` / `composite` | As in the ACT rule. |
| `status` | `active` / `deprecated` | `deprecated` if the ACT rule carries a deprecation notice. |
| `wcag_sc` | string[] | Success criteria the ACT rule lists as primary requirements (WCAG 2.x numbers). |
| `coverage` | enum | `full`, `partial`, `none`, `unclear`. |
| `internal_rules` | string[] | Detector `rule_id`s (the `Violation.rule_id` namespace, as in `tests/common/rule_inventory.rs`) that implement the ACT rule. Non-empty for `full` and `partial`, empty otherwise. All of them together are needed for the stated coverage. |
| `related_rules` | string[] | Internal `rule_id`s that are related but not equivalent (sub-checks, manual rules, other criteria). Informational; no verdict is derived from them. |
| `gap` | enum | Only for `coverage: none`: `candidate` (sensibly automatable, missing), `human-judgement` (outcome needs a human or content analysis, not a sensible automatic check), `deprecated`. |
| `starter` | bool | Present and `true` for the first submission set. Only `full` and `active` rules. |
| `note` | string | English remark on differences or the relation. |

One id per rule: the detector id (`contrast/text-insufficient`) is also the `rule_id` of the report finding and the key of the rule's entry in `src/taxonomy/rules.rs`; there is no second namespace to map to (#918).

`tests/act_mapping.rs` fails when an `internal_rules` or `related_rules` id is not produced by any detector, when coverage and rule lists contradict each other, or when a starter rule is not fully covered and active.

## Overview

94 ACT rules in the pinned commit (7 deprecated).

| Coverage | ACT rules |
| --- | ---: |
| `full` | 22 |
| `partial` | 22 |
| `unclear` | 5 |
| `none` | 45 |

## Full coverage

| ACT | Name | Internal rules | Note |
| --- | --- | --- | --- |
| `kb1m8s` | ARIA global properties not used where prohibited | `aria/attribute-prohibited` | Both cover aria-label, aria-labelledby and aria-roledescription on roles that prohibit them. |
| `6cfa84` | Element with aria-hidden has no content in sequential focus navigation | `keyboard/hidden-focusable` | Same expectation: no element in sequential focus navigation inside aria-hidden=true. |
| `ff89c9` | ARIA required context role | `aria/required-parent-missing` | Internal rule additionally covers a native li under a re-roled list. |
| `73f2c2` | Autocomplete attribute has valid value | `forms/autocomplete-invalid` | Both validate the HTML autofill token grammar. |
| `97a4e1` | Button has non-empty accessible name | `buttons/name-missing` | Same applicability and expectation (empty accessible name). |
| `7d6734` | SVG element with explicit role has non-empty accessible name | `svg/name-missing` | Internal rule fails for explicit img/graphics roles and reviews svg without role. |
| `e086e5` | Form field has non-empty accessible name | `forms/label-missing`, `names/required-missing` | Union of the two internal rules covers the ACT role list; neither rule alone does. |
| `ffd0e9` | Heading has non-empty accessible name | `headings/empty` | Same expectation: non-empty accessible name of headings. |
| `b5c3f8` | HTML page has lang attribute | `document/lang-missing` | Both require a non-empty lang on the root html element. html-has-lang judges iframe documents and is not part of this ACT rule. |
| `bf051a` | HTML page lang attribute has valid language tag | `document/lang-invalid` | Both check the primary language subtag of a non-empty page lang. |
| `5b7ae0` | HTML page lang and xml:lang attributes have matching values (deprecated) | `document/lang-mismatch` | Both compare primary subtags of lang and xml:lang. The ACT rule is deprecated. |
| `2779a5` | HTML page has non-empty title | `document/title-missing`, `document/title-empty` | Union of the two internal rules covers a missing and an empty title. |
| `59796f` | Image button has non-empty accessible name | `images/input-alt-missing` | Same: an image button needs a name that is not the browser default. |
| `c487ae` | Link has non-empty accessible name | `links/name-missing`, `names/required-missing` | links/name-missing covers a[href]; role=link elements are judged by names/required-missing. |
| `m6b1q3` | Menuitem has non-empty accessible name | `names/required-missing` | menuitem is in the role list of names/required-missing. |
| `bc659a` | Meta element has no refresh delay | `timing/meta-refresh` | Same 20-hour exception (71999 s fails, 72000 s passes). Internal rule judges every refresh meta, not only the first. |
| `b4f0c3` | Meta viewport allows for zoom | `zoom/viewport-locked` | Both fail user-scalable=no/0 and maximum-scale below 2. |
| `674b10` | Role attribute has valid value | `aria/role-invalid` | Same: at least one valid non-abstract role token. |
| `2t702h` | Summary element has non-empty accessible name | `summary/name-missing` | Both judge the summary button of a details element. |
| `afw4f7` | Text has minimum contrast | `contrast/text-insufficient` | Same thresholds (4.5:1, 3:1 for large text, no rounding); undetermined cases are review, not pass. |
| `09o5cg` | Text has enhanced contrast | `contrast/text-enhanced` | Same thresholds (7:1, 4.5:1 for large text). |
| `2ee8b8` | Visible label is part of accessible name | `label-in-name/mismatch` | Both require the visible label to be contained in the accessible name; internal rule adds review for ambiguous composite labels. |

## Partial coverage

| ACT | Name | Internal rules | Note |
| --- | --- | --- | --- |
| `5f99a7` | ARIA attribute is defined in WAI-ARIA | `aria/attribute-unknown` | Internal rule judges only attributes of elements in the accessibility tree; ACT applies to every aria-* attribute. |
| `bc4a75` | ARIA required owned elements | `aria/required-children-missing` | Internal rule also passes aria-expanded=false containers, which ACT does not exempt. |
| `5c01ea` | ARIA state or property is permitted | `aria/attribute-not-allowed` | Internal rule additionally applies the stricter ARIA in HTML restrictions for native elements. |
| `6a7281` | ARIA state or property has valid value | `aria/attribute-value-invalid` | Internal rule covers enumerated, integer and number types; ID reference types are a separate rule. |
| `80f0bf` | Audio or video element avoids automatically playing audio | `media/audio-autoplay` | Internal rule covers audio only and ends in review; ACT also covers video and is a composite of two sub-rules. |
| `cf77f2` | Bypass Blocks of Repeated Content | `headings/none` | Composite rule. Internal rule accepts headings, a main landmark or a skip link as bypass mechanism and reports review without a repeated block; no collapsible-block check. |
| `b33eff` | Orientation of the page is not restricted using CSS transforms | `orientation/root-rotated` | Internal rule checks rotation of html/body in the audited viewport; ACT checks any element with a conditional orientation media query. |
| `c4a8a4` | HTML page title is descriptive | `document-title` | Internal rule only flags known placeholder titles (F25); descriptiveness is a human judgement. Heuristic: not reported as ACT result. |
| `3ea0c8` | Id attribute value is unique (deprecated) | `ids/duplicate` | Internal rule only reviews duplicate ids that are referenced; ACT applies to every id. The ACT rule is deprecated (SC 4.1.1 removed in WCAG 2.2). |
| `cae760` | Iframe element has non-empty accessible name | `frames/name-missing` | ACT excludes iframes with negative tabindex; internal rule judges every iframe in the accessibility tree. |
| `qt1vmo` | Image accessible name is descriptive | `images/alt-suspicious` | Internal rule flags file names and filler words as review; descriptiveness is a human judgement. Heuristic: not reported as ACT result. |
| `23a2a8` | Image has non-empty accessible name | `images/alt-missing`, `image-alt` | Internal rules split img (images/alt-missing) and role=img (image-alt); ACT also accepts role none/presentation as pass. |
| `aizyf1` | Link is descriptive | `links/purpose-link-only` | Internal rule flags generic link names as review; descriptiveness is a human judgement. Heuristic: not reported as ACT result. |
| `5effbb` | Link in context is descriptive | `links/generic-name` | Internal rule flags generic link names as review; descriptiveness is a human judgement. Heuristic: not reported as ACT result. |
| `bisz58` | Meta element has no refresh delay (no exception) | `timing/meta-refresh` | Internal rule keeps the 20-hour exception that this variant removes, so it misses delays of 20 hours or more. |
| `8fc3b6` | Object element rendering non-text content has non-empty accessible name | `objects/alt-missing` | ACT is limited to objects embedding image, video or audio resources; internal rule judges every object. |
| `4e8ab6` | Element with role attribute has required states and properties | `aria/required-attribute-missing` | Internal rule covers a fixed list of roles; ACT covers every role with required states and properties. |
| `0ssw9k` | Scrollable content can be reached with sequential focus navigation | `keyboard/scrollable-region-not-focusable` | Internal rule uses a 13 px overflow tolerance and requires text; ACT compares the scroll distance with the padding. |
| `oj04fd` | Element in sequential focus order has visible focus | `focus/outline-removed` | Internal rule inspects stylesheets (outline removed without replacement); ACT compares pixels of focused and unfocused element. Heuristic. |
| `d0f69e` | Table header cell has assigned cells | `tables/header-without-data` | Internal rule fails headers in a table without any data cell; ACT assigns cells per header by scope and position. |
| `a25f45` | Headers attribute specified on a cell refers to cells in the same table element | `tables/headers-attr-invalid` | Internal rule requires header cells of the same table; ACT requires any cell of the same table and not the cell itself. |
| `f51b46` | Video element auditory content has captions | `video-caption` | Internal rule only checks for a same-origin caption track and otherwise reports untested; it does not decide whether the audio needs captions. |

## Unclear

| ACT | Name | Related internal rules | Note |
| --- | --- | --- | --- |
| `efbfc7` | Text content that changes automatically can be paused, stopped or hidden | `motion/infinite-animation` | Both relate to SC 2.2.2, but the internal rule judges CSS animations while ACT judges auto-updating text content. Relation not established. |
| `24afc2` | Important letter spacing in style attributes is wide enough | `text-spacing/clipped` | ACT checks !important letter spacing in style attributes; internal `text-spacing/clipped` applies the SC 1.4.12 overrides and compares clipping. Relation not established. |
| `78fd32` | Important line height in style attributes is wide enough | `text/line-height-tight`, `text-spacing/clipped` | ACT checks !important line height in style attributes; internal rules test different conditions. Relation not established. |
| `9e45ec` | Important word spacing in style attributes is wide enough | `text-spacing/clipped` | ACT checks !important word spacing in style attributes; internal `text-spacing/clipped` applies the SC 1.4.12 overrides and compares clipping. Relation not established. |
| `59br37` | Zoomed text node is not clipped with CSS overflow | `css-overflow-hidden`, `text-spacing/clipped` | ACT checks clipped text at 640x512 with zoomed text; the internal reflow check works at 320 px and text-spacing simulates SC 1.4.12 overrides. Relation not established. |

## Missing rules: candidates for implementation

ACT rules without an internal counterpart that can be checked automatically (`gap: candidate`). Most need only DOM and accessibility-tree data; the keyboard-trap and autoplay rules need browser interaction or media inspection.

| ACT | Name | Success criteria |
| --- | --- | --- |
| `aaa1bf` | Audio or video element that plays automatically has no audio that lasts more than 3 seconds | - |
| `4c31df` | Audio or video element that plays automatically has a control mechanism | - |
| `047fe0` | Document has heading for non-repeated content | - |
| `ye5d6e` | Document has an instrument to move focus to non-repeated content | - |
| `b40fd1` | Document has a landmark with non-repeated content | - |
| `de46e4` | Element with lang attribute has valid language tag | 3.1.2 |
| `ebe86a` | Focusable element has no keyboard trap via non-standard navigation | - |
| `a1b64e` | Focusable element has no keyboard trap via standard navigation | - |
| `akn7bn` | Iframe with interactive elements is not excluded from tab-order | 2.1.1, 2.1.3 |
| `307n5z` | Element with presentational children has no focusable content | 4.1.2 |

The remaining `none` rules with `gap: human-judgement` (descriptiveness, media alternatives, equivalent purpose, device motion and similar) need a human verdict or content analysis. They stay out of automatic ACT output; the internal manual rules report them as review items.

## First submission set

Rules with `starter: true`: fully covered, active, simple deterministic behaviour (document-, name- and attribute-level checks, no layout or colour computation).

| ACT | Name | Internal rules |
| --- | --- | --- |
| `97a4e1` | Button has non-empty accessible name | `buttons/name-missing` |
| `7d6734` | SVG element with explicit role has non-empty accessible name | `svg/name-missing` |
| `ffd0e9` | Heading has non-empty accessible name | `headings/empty` |
| `b5c3f8` | HTML page has lang attribute | `document/lang-missing` |
| `bf051a` | HTML page lang attribute has valid language tag | `document/lang-invalid` |
| `2779a5` | HTML page has non-empty title | `document/title-missing`, `document/title-empty` |
| `59796f` | Image button has non-empty accessible name | `images/input-alt-missing` |
| `c487ae` | Link has non-empty accessible name | `links/name-missing`, `names/required-missing` |
| `674b10` | Role attribute has valid value | `aria/role-invalid` |

Deprecated ACT rules (for example `5b7ae0`) are not submission targets even when covered.

## Test case harness (#837)

`tests/act_testcases.rs` runs auditmysite against the official ACT test cases and compares the result per test case and rule with the expectation of the example. It is a local check (no CI minutes): the browser run is `#[ignore]`.

```bash
cargo test --test act_testcases -- --ignored --nocapture                 # starter set
ACT_RULES=full cargo test --test act_testcases -- --ignored --nocapture  # all full + active rules
ACT_RULES=97a4e1,b5c3f8 cargo test --test act_testcases -- --ignored --nocapture
```

| Variable | Meaning |
| --- | --- |
| `ACT_RULES` | `starter` (default), `full`, or comma-separated ACT ids. Only `full` + `active` rules of `act-mapping.json` are accepted. |
| `ACT_OUT_DIR` | Output directory, default `<target>/act/`. |
| `ACT_CACHE_DIR` | Cache of the ACT repository, default `<target>/act-cache/`. |
| `ACT_FAIL_ON_DEVIATION` | Set it to fail the test on any deviation. Without it deviations are reported, not failed: they are findings about the tool. |

**Source.** The test cases are not copied into this repository. `scripts/act-fetch.sh` downloads the commit pinned in `act-mapping.json` (`act_source.commit`) from GitHub, keeps `_rules/` and `test-assets/` (about 88 MB) in the cache, and verifies a SHA-256 over the contents of both trees (pinned in the script; a mismatch aborts). The published `testcases.json` is not used: it is a build artefact that lags the repository (it has no test cases for rules such as `kb1m8s`). Like the official generator, the harness takes the single code block of every `#### Passed|Failed|Inapplicable Example N` in a rule file as one document: HTML gets a `<!DOCTYPE html>` prefix, `svg`/`xml`/`xhtml`/`js` blocks are served unchanged with their content type. A local server serves the documents and `/test-assets/*` on one origin. Each test case is audited in its own isolated browser context at WCAG level AAA. The ACT material is under the W3C license (see `LICENSE.md` of the cache); the parser stops if an example does not have exactly one code block.

**Outcome per test case.** Only findings and execution records of the rule's `internal_rules` count.

| Outcome | Meaning |
| --- | --- |
| `failed` | A confirmed violation of a mapped rule. |
| `cantTell` | No violation, but a review note of a mapped rule. |
| `inapplicable` | Every mapped rule recorded `NotRun(NotApplicable)`. |
| `passedOrInapplicable` | The mapped rules ran and reported nothing. A rule that ran does not say whether it found applicable content, so `passed` and `inapplicable` cannot be told apart. |
| `notRun` | A mapped rule did not run for another reason (`Errored`, `CapabilityMissing`, `Disabled`). Never counted as a pass. |
| `auditError` | The audit of the document failed. |

**Verdict** (outcome against the expectation): `ok`, `false_positive` (failed on a passed/inapplicable example), `false_negative` (no failure on a failed example), `cant_tell`, `wrong_applicability` (inapplicable on a passed example), `not_run`, `audit_error`. **Consistency per rule:** `consistent` (all `ok`), `partially_consistent` (no false positive, but misses or undecided cases), `inconsistent` (at least one false positive).

**Output** (`results.json`, `schema_version` 1; `report.md` is the readable form). The EARL writer (#838) reads this file:

```json
{
  "schema_version": 1,
  "act_source": { "repository": "...", "commit": "..." },
  "selection": "starter", "wcag_level": "AAA",
  "summary": { "rules": 9, "testcases": 118, "ok": 0, "false_positive": 0, "false_negative": 0,
               "cant_tell": 0, "wrong_applicability": 0, "not_run": 0, "audit_error": 0 },
  "rules": [{
    "act_id": "97a4e1", "name": "...", "internal_rules": ["buttons/name-missing"],
    "consistency": "consistent", "counts": { "testcases": 17, "ok": 17, "...": 0 },
    "testcases": [{
      "testcase_id": "97a4e1/passed-example-1", "title": "Passed Example 1",
      "expected": "passed", "outcome": "passedOrInapplicable", "verdict": "ok",
      "source_file": "_rules/....md", "language": "html",
      "findings": [{ "rule_id": "...", "class": "fail|review", "selector": null }],
      "runs": ["buttons/name-missing: Ran(0)"], "error": null
    }]
  }]
}
```

`testcase_id` is `<act_id>/<slug>` of the example heading; the hash ids of the published `testcases.json` are not reproduced.

**EARL** (`earl.json`, #838). The harness also writes an [EARL JSON-LD report](https://act-rules.github.io/pages/implementations/earl-reports/) (`@context` `https://act-rules.github.io/earl-context.json`, `@graph` of `TestSubject`s with `Assertion`s) built from `results.json` and the mapping by `earl_json` in `tests/common/act_harness.rs`. It is part of the ACT evidence, not a product output format (the product stays JSON + PDF). Rules of the mapping:

| Derived outcome | EARL |
| --- | --- |
| `failed` | `earl:failed` |
| `cantTell` | `earl:cantTell` |
| `inapplicable` (every mapped rule `NotRun(NotApplicable)`) | `earl:inapplicable` |
| `notRun`, `auditError` | `earl:untested` |
| `passedOrInapplicable` | no assertion (empty `assertions`) |

`earl:passed` is never emitted: passed and inapplicable cannot be told apart, and the expectation of the example must not leak into the result. `test.title` is the internal rule id(s) joined by `+`; `isPartOf` uses the permissive object form `{"title": "WCAG 2: <sc>"}` from the mapping, so no `sc-urls.json` table is copied. `source` is a link to the example in the pinned rule file (`.../blob/<commit>/_rules/<file>.md#<slug>`); the hash URLs of the published `testcases.json` are not reproduced, so matching them is a step of the submission (#839).

### Result: starter set (commit `50d2bed`, 2026-11-02)

118 test cases of 9 rules; 100 agree with the expectation. Of the 18 deviations, 2 are false positives, 7 false negatives, 4 cant_tell, 3 not run, 2 audit errors. Expectations come from the example headings, not from the run. These are findings about the tool, not adjusted away; the rules involved are shared rules and belong to barrierlab (`a11y-rules`), so they are documented here and not patched in `src/wcag/shared.rs`.

| ACT | Rule | Consistency | Test cases | Deviations |
| --- | --- | --- | ---: | --- |
| `97a4e1` | Button has non-empty accessible name | consistent | 17 | - |
| `c487ae` | Link has non-empty accessible name | consistent | 28 | - |
| `674b10` | Role attribute has valid value | consistent | 10 | - |
| `7d6734` | SVG element with explicit role has non-empty accessible name | partially consistent | 10 | 4 cantTell |
| `ffd0e9` | Heading has non-empty accessible name | partially consistent | 15 | 3 false negatives |
| `bf051a` | HTML page lang attribute has valid language tag | partially consistent | 7 | 2 false negatives, 1 not run |
| `2779a5` | HTML page has non-empty title | partially consistent | 12 | 1 false negative, 1 not run, 2 audit errors |
| `b5c3f8` | HTML page has lang attribute | inconsistent | 7 | 1 false positive, 1 not run |
| `59796f` | Image button has non-empty accessible name | inconsistent | 12 | 1 false positive, 1 false negative |

Deviations by cause:

- **Shared rules, to report to barrierlab.**
  - `bf051a` failed 1 and 3: `lang="em-US"` and `lang="eng"` (ISO 639-2 three-letter code) are accepted by `document/lang-invalid`.
  - `ffd0e9` failed 3, 4, 8: `headings/empty` misses an empty name from `aria-labelledby` pointing to an empty hidden element, from an `img` with `role="presentation"` inside the heading, and from `aria-label=""` on a heading with `role="none"` (presentational-role conflict resolution).
  - `59796f` passed 3: `<input type="image" title="Search">` is reported although `title` gives an accessible name (false positive). Failed 3: `aria-labelledby` to a missing id, where the default name "Submit Query" does not count for the ACT rule, is not reported.
  - `7d6734` passed 2 (svg with a `https://` namespace typo), failed 3 (`circle role="graphics-symbol"` without name) and inapplicable 1 and 3 (svg without / with another role): the rule answers review instead of a verdict. Whether `svg/name-missing` should decide here is a coverage question (the mapping note says it fails explicit img/graphics roles).
  - `2779a5` failed 6: a `title` inside an attached shadow root is accepted as the page title; the ACT rule requires a descendant of the document element in the light tree.
  - `b5c3f8` inapplicable 2: `<math lang>` served as `application/xml` is audited as an HTML page and reported as missing `lang` (the browser wraps the XML document; relation to the ACT example not further analysed).
  - `b5c3f8`, `bf051a`, `2779a5` inapplicable 1: for an `svg` document the rules record `NotRun(Errored)` instead of `NotApplicable`.
- **auditmysite, fixed.** `2779a5` passed 2 and failed 3 (pages that embed an iframe) made the whole audit fail with `rule_findings_preserved at rule_outcomes[html-has-lang]`. Cause: a page and its iframe document both lack `lang`; `document/lang-missing` and `html-has-lang` share criterion 3.1.1 and were merged into one finding, but only `3.1.1` (not the two rule ids) resolved to that finding's taxonomy rule, so the contract could not find the run's occurrences again. Since #918 each rule id is the key of its own finding, so the run and its finding match by id; both cases now agree with the expectation.
- **Harness, fixed (not a state leak in auditmysite).** The false positives seen when all test cases shared the default browser context came from the harness server: it answered Chrome's empty speculative (preconnect) sockets with a `404`, Chrome reused such a socket for the audit's second (mobile) navigation of the same URL and read `not found` as the page. Isolated contexts hid it by chance. The server now ignores connections that send no request; the default context gives the same results again.

Applicability is only partly visible: `inapplicable` examples count as `ok` when the rule ran without findings. #838 must therefore not emit `inapplicable` from `passedOrInapplicable`.
