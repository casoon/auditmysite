---
title: "SEO code against web-checks: classification"
description: "Which parts of the SEO checks are judged by the shared web-checks crate and which stay in auditmysite (#1120)."
---

State of `src/seo/` against the published `web-checks 0.7.0` (the version in
`Cargo.lock`; source read from the crates.io archive). The second consumer of
`web-checks` is `astro-post-audit`, which was read for the (d) cases only.

Classes:

- **(a)** already from `web-checks`: the logic is upstream, the file only adapts it.
- **(b)** duplicates `web-checks` logic: delete here, call the upstream function.
- **(c)** auditmysite's own: needs the browser, report wording or scoring.
- **(d)** needed by both consumers but missing in `web-checks`: barrierlab issue.

`web-checks` offers: `meta` (title/description length), `social` (OpenGraph and
Twitter presence, completeness, card values, absolute URLs), `structured_data`
(JSON-LD parsing, structural issues, per-feature requirement rules, duplicate
types), `hreflang`, `robots`, `third_party`. It has no render-blocking, no
canonical, no heading and no page-intent/route logic.

## Result

Rows below (an item is a function or a closely coupled group of them):

| Class | Rows |
|---|---|
| (a) already from `web-checks` | 11 |
| (b) duplicates `web-checks` | 0 (the 3 found were switched to `web-checks`) |
| (c) auditmysite's own | 34 |
| (d) missing in `web-checks` | 1 |

No (b) is left. The three that existed were outside the four listed files and are described below.

## `src/seo/schema_rules.rs`

| Item | Class | Note |
|---|---|---|
| `pub use web_checks::structured_data::{assess_node, inventory_fields, SchemaFeature, SchemaRuleAssessment, ProductRuleContext, ManualReview, RULESET_VERSION, ...}` | (a) | Rule tables live upstream only. |
| `feature_label`, `status_text`, `manual_review_text` | (c) | Report wording (Fluent), two languages. |

## `src/seo/schema.rs`

| Item | Class | Note |
|---|---|---|
| `analyze_structured_data_payloads` (parsing via `parse_blocks`, `StructuralIssue`) | (a) | Parsing, normalisation of `@graph`/`@type` lists and structural issues are upstream. The function only maps them into the stored model. |
| `build_rule_assessments` (calls `assess_node`) | (a) | Rule evaluation upstream. |
| `structural_issue` | (c) | Maps `StructuralIssue` to the stable issue key and canonical English message of the JSON report. |
| `rule_assessment_issues` | (c) | Builds `schema_rule_missing_*` issue keys and messages. |
| `refresh_rule_assessments` | (c) | Re-runs the upstream rules with the page-derived `ProductRuleContext`. |
| `detect_structured_data`, `STRUCTURED_DATA` collector spec, `js/structured_data.js` | (c) | Browser collection. |
| `StructuredData`, `JsonLdSchema`, `SchemaIssue`, `SchemaIssueSeverity` | (c) | Stored report model (JSON contract). |
| `schema_issue_text`, `schema_issue_label` | (c) | Wording; the language-neutral kind stays in the stored issue (#406). |
| `SchemaType` (enum, `Display`, `FromStr`, `as_str`) | (c) | Closed list the report and other modules (`ai_visibility`, `source_quality`) match on; not a rule. |
| `SchemaType::is_organization_like` | (c) | Consumed by `ai_visibility` and `source_quality`; `astro-post-audit` has no such predicate. |
| `SchemaType::rich_snippet_type` | (c) | Report vocabulary ("a type behind a search feature"), stored verbatim in JSON. |
| `counter_cases`, unit tests | (c) | Test code. |

## `src/seo/schema_fit.rs`

| Item | Class | Note |
|---|---|---|
| `SchemaPageKind`, `SchemaFitStatus`, `SchemaCoverageStatus`, `SchemaFitAssessment` and their label/text methods | (c) | Report model and wording. |
| `assess_schema_fit`, `assess_schema_fit_with_facts` | (c) | Needs `PageIntent` (page-flow module), the visible heading and title from the rendered DOM, and route heuristics; the expectation per page kind is product judgement, not a Google rule. `astro-post-audit` has no equivalent (no route-to-type expectation in its `structured_data*.rs`). |
| `url_path`, `contains_any`, `route_ends_with`, `is_supporting_type` | (c) | Helpers of the above. |
| `SchemaFitAssessment::product_context` | (c) | Derives the upstream `ProductRuleContext` from the fit result. |
| unit tests | (c) | Test code. |

## `src/seo/schema_parity.rs`

| Item | Class | Note |
|---|---|---|
| `extract_visible_schema_facts`, `VISIBLE_FACTS`, `js/visible_schema_facts.js` | (c) | Reads the visible page in Chrome. |
| `assess_content_parity`, `compare_title`, `compare_property_to_title`, `compare_single_visible`, `compare_lists`, `evidence` | (c) | Compares visible text with JSON-LD; needs the rendered DOM. |
| `values_match`, `normalize`, `schema_values` | (c) | Text matching and JSON path walking used only by the parity comparison. |
| `VisibleSchemaFacts`, `ContentParityStatus`, `ContentParityAssessment` (`status_text`, `evidence_text`) | (c) | Report model and wording. |
| `counter_cases`, unit tests | (c) | Test code. |

## `src/seo/meta.rs` (meta lengths)

| Item | Class | Note |
|---|---|---|
| `MetaTags::validate`, length part: `web_checks::meta::length`, `TITLE.classify`, `DESCRIPTION.classify`, `LengthClass` | (a) | Character count and the 30-60 / 120-160 ranges are upstream. |
| `MetaTags::validate`, rest: which issue exists, severity, hint text | (c) | The host decides severity and wording (upstream says so in its module doc). |
| `MetaIssueKind`, `MetaIssueValues`, `MetaValidation::new`, `meta_issue_text`, `localized_message` | (c) | Kind enum plus pure text function (#406). |
| `MetaTags` (struct), `has_essentials` | (c) | Report model. |
| `extract_meta_tags`, `META`, `js/meta_tags.js` | (c) | Browser collection. |
| `counter_cases`, unit tests | (c) | Test code. |
| lengths in `src/seo/serp.rs` and `src/seo/profile.rs` | (a) | `length`, `TITLE`, `DESCRIPTION` upstream. The numbers `150` (`build_summary`) and `60` (schema value preview) in `profile.rs` are display truncations, not length rules. |

## `src/seo/social.rs` (OpenGraph presence)

| Item | Class | Note |
|---|---|---|
| `OpenGraph::is_complete`, `completeness`, `TwitterCard::is_complete`, `completeness` | (a) | `social::is_complete`, `social::completeness`, field lists upstream. |
| `parse_social_tags` (presence via `social::is_present`) | (a) | Empty content counts as absent, decided upstream. |
| `SocialTags::validate` (`is_valid_twitter_card`, `is_absolute_url`, `TWITTER_CARD_TYPES`) | (a) | Value checks upstream; severity and wording here are (c). |
| `OpenGraph::content`, `TwitterCard::content` | (c) | Adapter from the stored struct to tag names; includes `og:locale`, `twitter:site`, `twitter:creator`, which upstream does not rate. |
| `og_any`/`tw_any` tag lists in `parse_social_tags` | (c) | Decide whether the group exists in the JSON (`Option`), a stored-model concern. |
| `SocialTags`, `OpenGraph`, `TwitterCard` structs | (c) | Report model. |
| `extract_social_tags`, `SOCIAL`, `js/social_tags.js` | (c) | Browser collection. |
| `counter_cases`, unit tests | (c) | Test code. |

## Render-blocking

`web-checks` has no render-blocking code. It is computed in the browser, in
two places:

| Item | Class | Note |
|---|---|---|
| `src/performance/js/render_blocking.js`, `analyze_render_blocking`, `render_blocking_suggestions` (`src/performance/render_blocking.rs`) | (c) | Measures blocking scripts and stylesheets with transfer sizes from resource timing; browser, scoring and wording. |
| `syncHeadScripts` in `src/seo/page_health/js/dom_inspection.js`, `sync_head_scripts` finding in `page_health/issues.rs` | (c) | Count of synchronous head scripts for the SEO page-health finding. |
| Predicate "`head script[src]` without `async`, `defer` or `type=module`" | (d) | Written three times: the two scripts above and `astro-post-audit` `checks/render_blocking.rs` (`sync-head-scripts`). It is pure HTML (markup in, count out) and fits `web-checks` next to `social` and `meta`. |

## Hreflang and JSON-LD validity in page health (done, #1120)

These three duplicated upstream and were switched to `web-checks`: the page
scripts (`page_health/js/dom_inspection.js`, `js/technical_signals.js`) now only
collect raw values (hreflang codes, resolved hrefs with the page URL, raw
JSON-LD block texts); Rust judges them in `page_health/dom.rs` and
`technical.rs`.

| Item | Class | Note |
|---|---|---|
| `hreflang_invalid` count | (a) | `web_checks::hreflang::is_valid_code`. |
| `hreflang_no_x_default`, `hreflang_no_self_reference` | (a) | `has_x_default`, `has_self_reference`. |
| `jsonld_invalid` count | (a) | `web_checks::structured_data::parse_blocks`: a block with any `StructuralIssue`. |
| `hreflang` collection, `HreflangTag`, `serp.rs` x-default row | (c) | Browser collection and presentation of the tags. |

Finding changes (intended, listed in the CHANGELOG): `de-de` and `zh-Hant-TW`
are valid; a page naming itself only as `x-default` gains
`hreflang_no_self_reference`; `@graph` pages lose `jsonld_invalid`.

## Boundary result

- Already upstream: structured data rules and parsing, meta lengths, OpenGraph
  and Twitter presence/completeness, robots grammar and bot classes,
  third-party providers.
- Stays here: everything classified (c), including the report wording around
  the upstream functions.
- Open for barrierlab: the render-blocking predicate (d). Canonical, headings and the page-intent fit have no counterpart upstream and
  are not move candidates.
