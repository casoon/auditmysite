# Changelog

All notable changes to auditmysite are listed here. auditmysite follows
[Semantic Versioning](https://semver.org/). The detailed development log with findings, fixes
and verification for each change is in the
[development history](https://casoon.github.io/auditmysite/docs/reference/development-history/).

## [Unreleased]

### Added

- **Audit contracts:** A format-independent validation boundary checks normalized page and batch data before live results are returned, final runs are persisted, or CLI reports and library exports are generated. Contract failures provide stable codes and data paths; honestly qualified partial results remain usable.
- **Validation:** JSON export rejects contradictory scores, execution outcomes and missing rule findings. Release checks require positive and negative corpus evidence for every inventoried WCAG, security, schema and library check; deferred checks do not satisfy this gate.
- **Release:** Version 2.x requires completion of the 26 foundation-audit issues and their tracker in the 2.0 milestone. Missing issues, open prerequisites, non-completion closures and GitHub API failures block the release before artifact builds.
- **Batch:** `summary.root_causes[]` lists finding patterns that recur on two or more pages, with all URLs, occurrence count and confidence (`confirmed` same markup, `likely` same pattern above the coverage thresholds, `open` otherwise); `id` matches `findings[].patterns[].id`.
- **JSON:** Every finding carries a stable `id` (`F-` and eight hex digits) from its rule, and `patterns[]` with the element patterns of all its occurrences (normalized selector, occurrence count, `RC-` id that is the same for the same rule and pattern on every page); `pattern_count` gives the full number.
- **report-lint:** `indicator_carries_no_weight` flags a module that contributes to the overall score without a weight, or an indicator with a weight or a letter grade.

### Changed

- **Scoring:** The overall score always uses the published module weights, including dual-viewport audits. Incomplete performance measurements and unchecked HTML conformance do not contribute; estimated Speed Index remains diagnostic. Principle coverage uses execution records instead of assuming every catalogued criterion passed.
- **Security:** Security-only audits also collect the JavaScript library checks. The known Handlebars advisory range includes versions below 4.7.7 and describes conditional exploitability.
- **CLI:** After the audited pages, the batch progress bar shows the sitemap check as its own stage instead of resting on the last page count.
- **Dependencies:** `runemark` 0.9.
- **Dependencies:** `renderreport` 0.7.1.
- **Dependencies:** `a11y-rules`, `a11y-report`, `a11y-dom`, `accname` 0.24.1.
- **JSON:** Breaking: `top_actions[].root_cause` (free text that called anything with 10 occurrences or two pages a "likely shared component") is replaced by `finding_id` and, in batch reports, `root_cause_id` pointing into `summary.root_causes`.
- **Mobile:** A disabled zoom (`user-scalable=no`) is still listed but no longer lowers the mobile score; it counts under accessibility (WCAG 1.4.4), where the same condition is already a finding.
- **PDF:** The cover shows the six modules that make up the overall score in one row (SEO and HTML conformance included) and, below them in neutral grey, the derived indicators (Search Experience, UX, Journey) marked as not part of the overall score; the Search Experience composite no longer stands in for the SEO score.
- **PDF:** The batch top-actions table shows the WCAG level instead of an expected-impact sentence that repeated the reach column, and gives the action title and reach more width.
- **PDF:** The batch report builds its template causes and the cause column of the top actions from `root_causes`: the action names the element (`a.mainnav-toggle: recurring, identical markup not established (20 pages)`), and patterns without enough evidence read "cause open" instead of a template claim.
- **Reports:** Risk rationales name controls without an accessible name and controls not operable by keyboard separately (`risk.keyboard_blocking_issues`), and neither they nor the verdict reasons state any longer that nameless controls cannot be operated or that tasks cannot be completed.
- **Reports:** The expected impact of an action states the occurrences it addresses instead of "Fixes N occurrences; expected score impact: high", which no simulation backed.
- **Reports:** "Source quality" is shown as "Technical preparation for machine processing", an unweighted indicator. Its score combines on-page consistency and publisher/security signals; structural signals (word count, headings, structured data, metadata) are listed as observed values without a rating. `source_quality.substance.score` is deprecated and removed in 2.0.
- **Reports:** Consistency compares what a page can contradict on its own — declared language vs. `og:locale`, `<title>` vs. `og:title`, structured data vs. visible content — and weighs only the checks that applied.
- **Reports:** The trust signal lists contact, imprint and privacy links that were not detected instead of reusing the UX trust score.
- **SEO:** A missing `lang` attribute or page title is marked as also counted under accessibility (WCAG 3.1.1 / 2.4.2).

### Removed

- **JSON:** Breaking: `summary.template_clusters`, `summary.top_recurring_rules` and `internal_comparison.root_causes`. Recurring element patterns are `summary.root_causes` with their evidence grade; the former lists either duplicated it or called anything on two pages a "likely template or component".

### Fixed

- **Audit completeness:** Stability fallbacks and visible frames that could not be inspected qualify the audit. Hidden and explicitly excluded frames remain outside the requested scope. Final single-page post-processing refreshes completeness before persistence.
- **Reports:** Different rules sharing a criterion and selector survive viewport merging. Critical accessibility limits survive viewport averaging; raw, terminal and JSON pass decisions use the canonical verdict. Failed batch URLs qualify audit completeness, and compact-export grades describe their exported score.
- **Collection:** Invalid collector responses and failed HTTP/DNS probes no longer become successful empty measurements. Missing performance observers and invalid numeric metrics remain unmeasured; SEO and security transport checks use the final response/page scheme.
- **Baseline:** Comparisons preserve concrete rule identity, severity and duplicate multiplicity, use stable selectors across browser reloads and record the actual calendar date.
- **AI transparency:** Image provenance fetches reject redirects and enforce the body-size limit while reading the response.
- **Batch:** `summary.grade` and `summary.certificate` describe the average accessibility score and are gated by the batch risk, as in the single report and the batch PDF; they followed the weighted overall score, so 49/100 accessibility read "D" / "STABIL".
- **PDF:** The batch report's scope note no longer spills onto an otherwise empty page 3; it opens the contents page, which is titled "Contents"/"Inhalt" instead of "Table of Contents".
- **PDF:** The batch action table shows the scope of change from the data (the JSON `top_actions[].complexity`) instead of a per-rule effort that could read "low" for the same row; a finding repeated on one page is no longer labelled "on 1 pages, cause open".
- **PDF:** Batch problem blocks use correct plurals ("1 affected URL"), name the reach as all audited pages, N of M pages or one page, and label the effort as a per-instance estimate.
- **PDF:** The contents page no longer lists its own heading or the cover title, and table rows no longer split across a page break.
- **Performance:** Stylesheets a page removes again while loading no longer mark the CSS coverage as partial or log a `CSS.getStyleSheetText` warning.
- **Reports:** Accessibility findings no longer lower the source-quality indicator: consistency counted unnamed controls and critical violations (the same occurrences twice), and the batch view counted accessibility score spread and pages with critical errors.
- **Reports:** Content Visibility no longer turns the Source Quality authority and substance scores into E-E-A-T and content-depth signals; it reads publisher markup, word count and headings directly.
- **Reports:** AI visibility judges structure for chunking by the main content region the page has, not by the absence of landmark findings from the accessibility result.
- **SEO:** A missing `lang` attribute or canonical URL costs points once instead of twice (meta list and technical checks), and a missing viewport no longer costs SEO points; it counts under mobile and accessibility.
- **WCAG:** A zoom limit between 200 % and 500 % (`zoom/viewport-scale-limited`) is a best practice: 1.4.4 requires 200 % and is met, so it raises no legal flag and only low BFSG relevance.

## [1.8.0] - 2026-10-24

### Added

- **WCAG:** Link purpose runs on the shared `a11y-rules` 0.24 ids `links/generic-name`, `links/url-name`, `links/single-character-name`, `links/new-window-unannounced` (3.2.5, now tests `target="_blank"`) and `links/purpose-link-only` (2.4.9); old ids resolve through `LEGACY_RULE_IDS`.
- **WCAG:** Seven manual-check hints (`manual/*`), non-text contrast of form controls (`contrast/control-*`, 1.4.11) and per-tree-scope id resolution for Shadow DOM come from `a11y-rules` 0.23.
- **Batch:** Page-type groups with an explicit page count and their most common finding, and a table of render-blocking files shared by several pages (`site_analysis.page_type_groups`, `performance.shared_blocking_resources`).
- **Batch:** The sitemap check reports the measured HTTP status distribution of every sitemap URL and groups redirects by target.
- **Batch:** Canonical targets that the sitemap check saw redirect or fail are reported as canonical conflicts.
- **Batch:** Security headers are compared across the pages: headers missing on some pages or sent with different values are listed, uniform ones summarized (`site_analysis.security.header_consistency`).

### Changed

- **WCAG:** Breaking: findings of the rules moved to `a11y-rules` (forms, landmarks, links and pointer, images and media, tables, document and language, link purpose) carry the shared ids, e.g. `links/generic-name` instead of `link-name`; `[rules]` filters with the old ids still apply.
- **WCAG:** Browser-free rules moved to the shared `a11y-rules` crate in batches B3–B7 (forms, landmarks and structure, links and pointer, images and media, tables, document and language), followed by contrast, target size, reading order, motion and stylesheet rules; deleted local rules keep their old ids resolvable.
- **WCAG:** Generic link text is a review hint instead of a violation; with surrounding context its severity is Low.
- **Scoring:** The per-viewport overall score uses the published module weights, and an unmeasured Performance module no longer counts.
- **Scoring:** The security score is built from one list of deductions; context-dependent headers (COOP, CORP, CORS) cost nothing and `frame-ancestors` replaces `X-Frame-Options`.
- **Reports:** Findings are described as having "possible legal relevance (BFSG/EAA)" rather than as legally relevant, and a failed verdict is scoped to the automated audit.
- **Reports:** UX and Journey statements say they are derived from structural signals; key terms (accessibility score, combined technical score, rule group, occurrence) are used consistently.
- **Reports:** WCAG coverage now lists 1.2.4, 1.3.3 and 2.4.5 as manual review (30 automated, 25 manual of 55 A/AA criteria).
- **PDF:** The findings matrix shows full titles in four columns, and pattern messages and the schema matrix appear in the run language.
- **JSON:** `content_topics.overlap_pairs` carries `similarity_percent` and `shared_topic_terms`; `shared_terms` is kept as a deprecated alias holding the similarity.
- **Batch:** The report states its sample and HTTP scope, compares modules by their range across pages, names its ranking criterion and starts the link graph at the homepage; orphan pages are reported as candidates.
- **Batch:** Identical titles, descriptions and H1s are reported as identical metadata, not duplicate content; a missing canonical is a hint.
- **Batch:** Topic overlap is measured on the main content and ignores site-wide terms such as brand names.
- **SEO:** Headings come from the shared `headings/*` rules and the html5ever check gave way to `html-conform`; CSP parsing uses `csp-parse`.
- **SEO:** The LocalBusiness hint appears only on location pages.
- **Dependencies:** `renderreport` 0.6.2, `a11y-*` 0.24.0, `rustls`, `rustls-webpki` and `webpki-roots` updates, `zip` 9 and `c2pa` 0.91.
- **Contributors:** `cross` in the release workflow is pinned to a reviewed commit; GitHub Actions updated.

### Fixed

- **Reports:** Best Practices and Tech Stack show a check status instead of a score of 100 when nothing was collected.
- **PDF:** No blank pages after finding cards, no headings left alone at the bottom of a page (finding titles, the content-visibility "manual review" panel), no tech-stack chapter without findings, and the batch cover note no longer overlaps the text below.
- **PDF:** The batch PDF no longer fails PDF/UA compilation when sitemap status counts are present.
- **Batch:** Render-blocking resources are counted once per file instead of once per page.
- **SEO:** Articles under news, blog or press routes (`/news/`, `/aktuelles/`, `/blog/`, `/presse/` …) count as editorial pages in the schema check instead of corporate pages that expect `Organization`/`LocalBusiness`.
- **Performance:** CSS coverage no longer reports 100 % on every page; the denominator is the style rules of the captured stylesheets.
- **Mobile:** The smallest font size only counts readable text (#740).
- **UX:** The dark-mode check counts dark-only pages as supported (#739).

### Security

- **Dependencies:** RUSTSEC-2026-0347–0349, -0367–0369, -0381 and -0382 fixed by updates; RUSTSEC-2026-0387 (`crypto-bigint` via `c2pa`, feature `ai-transparency`) is an audited exception until `c2pa` moves to `elliptic-curve` 0.14.

## [1.7.3] - 2026-10-01

### Added

- **WCAG:** Cross-site iframes are now audited too, because the audit browser starts without site isolation (#720).
- **JSON:** `exclusion.excluded_landmarks` now records landmarks left out of landmark counting by exclusions (#726).

### Changed

- **WCAG:** Breaking: ARIA and accessible-name checks now come from the shared `a11y-rules` crate instead of own rules, with new rule ids under `aria/*`, `names/*`, `dialog/*`, `popover/*` and `inert/*`; an unnamed dialog is reported only once (#691, #692).
- **Dependencies:** Shared `a11y-*` crates raised to 0.14.2, with `a11y-rules` 0.15.0 for the name rules.

### Fixed

- **WCAG:** No longer reports the text layer of figures inside hidden panels (#725), unnamed `form`/`region` elements as landmarks (#727), excluded landmarks in landmark counts (#726), or `form-no-submit` on forms that only contain toggles (#728).
- **Browser:** Single-page apps are now captured only after their first rendered content; a quiet page without content no longer counts as stable (#718).

## [1.7.2] - 2026-09-30

### Added

- **WCAG:** New rule `scrollable-region-focusable` (2.1.1) reports scrollable areas that cannot be reached by keyboard (#717).
- **WCAG:** Element rules now run inside same-process iframes, and `execution.frames` lists which frames were checked or skipped (#715).
- **WCAG:** Contrast is now measured inside Shadow DOM and over floated containers (#716).

### Changed

- **WCAG:** `title-only` and a description duplicating the name are now separate best-practice rules instead of violations (#711, #713).
- **Scoring:** `zoom/viewport-missing` now carries less weight, since desktop browsers ignore the viewport meta tag (#702).
- **Dependencies:** Shared `a11y-*` crates raised to 0.13.5.
- **Contributors:** Rustdoc lints now fail the build so broken docs cannot be published (#588, #719).

### Fixed

- **WCAG:** Target size now measures only visible, clickable targets in the settled layout and reports still-animating targets as untested (#705, #706).
- **WCAG:** Headings and `main` behind an open dialog, and a skip link behind a cookie banner, are now reported as hints instead of failures (#709).
- **WCAG:** Colour as link indicator is only checked for links in running text, and reduced-motion findings require actual movement on existing elements (#710, #712).
- **WCAG:** `title-only` now treats `label`, `value` and `alt` as name sources (#711).
- **WCAG:** `display/*` rules no longer set legal flags or a high BFSG relevance (#704).
- **Screen reader:** `--exclude-selector` and `[data-audit-exclude]` now also filter screen-reader issues, including on cache hits with `--reuse-cache` (#703, #708).

## [1.7.1] - 2026-09-30

### Added

- **WCAG:** New findings `zoom/viewport-missing` (no viewport meta tag) and `images/alt-suspicious` (file name, filler word or one or two characters as alt text, as a hint).

### Changed

- **WCAG:** Breaking: 21 rule ids are now taken from the shared `a11y-rules` 0.13.2 (for example `landmarks/main-missing`, `images/alt-missing`, `keyboard/hidden-focusable`) and the replaced own rules are removed, so no finding appears twice (#690).
- **WCAG:** Shared rules now use computed `display`/`visibility` and no longer check content hidden by CSS.
- **WCAG:** Missing banner, navigation and contentinfo landmarks and a missing skip link are now review hints instead of violations.
- **Contributors:** First step of moving the browser-free WCAG rules to barrierlab (casoon/barrierlab#13); docs site updated for 1.7.0 with recorded terminal demos.

## [1.7.0] - 2026-09-30

### Added

- **CLI:** Technician mode `--technician` writes one JSON per page plus `index.json` and `findings.jsonl` for working through findings, with path filters `--include-path`/`--exclude-path`.
- **CLI:** `--display calm|text|visual|all` audits each display mode of the `data-display` convention (#653).
- **CLI:** Intentional bad examples can be excluded with `data-audit-exclude`, `--exclude-selector` or `exclude_selectors` in `auditmysite.toml`; exclusions are shown in the report (#645).
- **CLI:** `--no-screen-reader-report` and `--html-conform` can now be used on their own.

### Changed

- **Library:** Crate metadata now points `documentation` to https://casoon.github.io/auditmysite/ and `homepage` to https://auditmysite.casoon.de/.
- **Contributors:** Large modules split up (`pipeline`, `normalized`, `single_report`, `module_details`, `page_health`, batch sections) behind a golden-render harness with unchanged output, plus test hygiene and a GitHub Pages docs site.

### Fixed

- **WCAG:** Fixed false positives in 3.2.2 (only real context changes fail), 1.3.5 (a thing's name is not a person's name), treegrid rows, native value inputs and spinbutton format hints (#655-#658).
- **WCAG:** Missing page title, missing `main` landmark and `landmark-unique` are now reported once instead of twice.
- **WCAG:** Tables now look through ignored and not yet rendered rows, skip links are recognised by their target, and `role="list"` on unstyled lists is kept (#642, #644, #654, #659).
- **WCAG:** The 2.5.8/2.5.5 Equivalent exception now applies to links (#652).
- **WCAG:** Gradient or transparent text is reported as needs-review instead of 1.00:1, and `-webkit-text-fill-color` is used as the painted colour (#640).
- **WCAG:** Format-instruction and checkbox-group heuristics no longer fail correct forms, and excluded elements no longer use up capped rules' budgets (#643, #645).
- **Reports:** Cache round-trips now restore the full report and report order is deterministic.
- **PDF:** Reports with `--lang en` are now fully English, including security and mobile tables, topic words, the HTML validator title and the certificate label.
- **JSON:** HTML validation texts in `page_health` are now canonical English (#406).
- **Batch:** Pool waiting now follows `--timeout`, no pages are lost silently, failed pages are retried serially, and `--per-page-reports` writes one file per page (#651).

## [1.6.0] - 2026-09-29

### Added

- **WCAG:** Separate rule ids and explanations for empty headings, missing navigation and contentinfo landmarks, and the shared rules, instead of a generic "missing structure" bucket.
- **Journeys:** Disclosure findings from a click whose wait ended early are now marked as uncertain.
- **JSON:** `accessibility.execution.navigation.meta_refresh` now records followed meta-refresh interstitials and unfollowed refresh redirects.
- **CLI:** `accname-diff` now accepts several URLs and `--url-file` for a corpus summary, uses computed style for its own name, and reports `text_transform` as its own class.

### Changed

- **WCAG:** The audit target is now WCAG 2.2 AA; the WCAG 2.1 view stays available as an appendix.
- **WCAG:** The automation ratio is now counted honestly (30 of 55 A/AA criteria), and AAA criteria that cannot fail are no longer listed as automatically checked.
- **Journeys:** Journeys now have a 15 s budget with a hard limit per journey.
- **Scoring:** A single high Level A violation now caps the accessibility score at 89, and desktop and mobile scores use the same rules.
- **Reports:** JSON and PDF now show the same findings in the same order, with urgency first and titles from the taxonomy; batch top actions are listed once per rule across all pages.
- **Batch:** Pages are now listed in input order, and on macOS with a full browser batches with journeys run one page at a time.
- **SEO:** Meta lengths now count characters instead of bytes, and empty OpenGraph tags count as missing; meta, OpenGraph, structured data and robots.txt checks now come from the shared `web-checks` crate.
- **Browser:** An installed headless shell is now preferred, and `doctor` and the audit start warn about a full Chrome on macOS.
- **Dependencies:** `a11y-rules`, `a11y-dom`, `a11y-report` and `accname` raised to 0.12.2, `html-conform` to 0.3.1; HTML conformance values can drop due to stricter checks.
- **Contributors:** Rust toolchain pinned, crate package limited to runtime files, browser tests in the release gate, docs.rs build reproduced in CI, and a live reference set with score bands plus an axe-core comparison for score calibration.

### Removed

- **WCAG:** Criterion findings raised only because a technique was present (2.2.3, 2.2.4, 2.5.1, 2.5.4), and duplicate own checks such as `valid-lang` and the table/list checks in `info_relationships`.

### Fixed

- **WCAG:** Fixed false positives for icons in named links (1.1.1), media player controls, 1.3.5 `autocomplete`, 2.4.7 without focusable elements, `role="status"`/`alert` without a name, and target size spacing and inline exceptions.
- **WCAG:** Own page rules now use unique selectors, radio grouping is a single `form-field-group` check, and `wcag_criterion` comes from the taxonomy instead of the rule id.
- **Journeys:** Skip-link journeys now check the jump target, disclosures are limited to the controlled region, menu buttons and native `<details>` are no longer checked twice, and read errors are not findings.
- **Reports:** Desktop and mobile findings are now merged one to one, blockers count elements, and German runs show shared rule texts in German while JSON stays English.
- **Browser:** Bot walls and access-denied pages now fail the URL instead of being scored, and refresh interstitials are no longer audited as the page.
- **Browser:** Batch pages are now visible and focused via focus emulation, the stability check reports its real result, and stalled throttled passes recover.

## [1.5.1] - 2026-09-22

### Fixed

- **Library:** The docs.rs documentation is now built with all features, so optional features such as `c2pa` are documented.

## [1.5.0] - 2026-09-22

### Added

- **Journeys:** Live regions are now observed over time, so form-error and add-to-cart journeys see announcements that appear briefly or use a pre-existing live container.
- **PDF:** Verdict and overall rating are now shown, and the overall value is derived in a table that reproduces it exactly.
- **CLI:** New command `accname-diff <URL>` compares the own accessible-name computation against Chrome's native accessibility tree.

### Changed

- **JSON:** Breaking: a module's `grade` is optional and only set for modules that feed the overall score (weight above 0).
- **WCAG:** Findings, severity and evidence now come from the shared `a11y-core` crates, which replace own rules for headings, lists, tables, viewport zoom and 3.1.1 language.
- **Scoring:** Accessibility is now the headline score; derived indicators are labelled as such, and counts replace arbitrary ratios.
- **Scoring:** Every heuristic deduction now gives its reason, and coverage signals count proportionally.
- **Reports:** The accessibility breakdown is now bound to the real scorer, and risk dimensions are derived once and shown in the PDF.
- **License:** auditmysite is now licensed under MIT; earlier releases keep the license they were published under.
- **Dependencies:** `a11y-core` raised to 0.10.0 and `runemark` to 0.6.
- **Contributors:** Problem-profile cluster split out of the PDF report, one fixture server for tests, and HTML tags in doc comments written as inline code (#588).

### Fixed

- **WCAG:** `nodes_checked` no longer double-counts DOM nodes.
- **WCAG:** The accessibility tree is now iterated in document order and `StaticText` is visible to text-measuring checks, so repeated runs give the same result.
- **Journeys:** Tabs, menu, modal and SPA navigation journeys no longer report measurement artefacts, and "little text above the fold" now measures the rendered page.
- **Reports:** Explanations now resolve the axe id first, and six PDF display artefacts are fixed.
- **SEO:** The report no longer claims an expected value for the page type that was never measured.
- **Security:** The report no longer claims no issues while findings are still open.

## [1.4.0] - 2026-09-18

### Added

- **JSON:** `security.issues[].values` and `mobile.issues[].values` now carry the raw interpolated values behind each message, omitted when empty.
- **JSON:** The HTML conformance module now reports `distinct_defect_count`, and the PDF shows one findings row per cause with an occurrences column.

### Changed

- **Scoring:** Breaking: HTML conformance now counts 5% toward the overall score and SEO drops from 20% to 15%, so `overall_score` for the same page differs from 1.3.x.
- **JSON:** Breaking: `rich_snippets_potential` now names the bare schema type (`"FAQPage"` instead of `"FAQ Rich Snippet"`), and the keys in `mobile.issues[].values.small_by_context` are canonical English (`other`/`form`).
- **SEO:** Breaking: The findings `pagination_missing_rel_links` (rel=prev/next is no longer an indexing signal) and `pwa_missing_service_worker` are no longer emitted.
- **Library:** Breaking: `security::coop_corp_verification_text` is folded into `security_issue_text`, `SecurityIssue` and `MobileIssue` gained a public `values` field, and `a11y_journey::link_inventory::analyse` now takes page and fallback language instead of a locale.
- **Reports:** Findings are now split into confirmed violations, heuristic warnings and manual checks, so no summary sentence can claim "no findings" while other modules show findings; scores are still derived from confirmed violations only.
- **Reports:** BFSG statements are now scoped to technical criteria and no longer read as a legal clearance.
- **Reports:** The five key measures are now fed from every module instead of WCAG groups only, and findings that resolve into one action are collapsed.
- **PDF:** TBT is no longer presented as a Core Web Vital, thresholds read as guidelines rather than targets, internal journey ids are labelled, and the audited scope appears on the cover.
- **PDF:** Rendering now uses bundled fonts (Arimo, PT Serif, JetBrains Mono) instead of system fonts, so PDFs look the same on every machine and need a fraction of the memory.
- **Security:** Security and mobile messages are now localized in the PDF.
- **Contributors:** CI gained a RustSec `dependency-audit` job, release builds now require the quality gates and build with `--locked`, and `scripts/release-check.sh` guards product documentation against known contradictions.

### Fixed

- **WCAG:** The HTML content-model check no longer panics when the parser's byte offset lands inside a multi-byte character.
- **Journeys:** Generic link text detection now matches the whole normalized link name instead of substrings, ending false WCAG 2.4.4 failures for descriptive links containing words like "mehr", and follows the page's `<html lang>` instead of the run language.
- **Screen reader:** Reading order no longer counts `InlineTextBox` layout artifacts, removing false positives.
- **Scoring:** The HTML conformance score now charges per distinct defect instead of per occurrence, so one faulty template element no longer floors it at 0.
- **PDF:** Icon-font glyphs and other undisplayable characters from audited pages no longer abort PDF generation; the JSON keeps the original text.
- **SEO:** The breadcrumb SERP signal now uses the schema requirement validation, ending the contradiction with "required properties missing".

### Security

- **Dependencies:** `rustls`, `crossbeam-epoch`, `quinn-proto` and `lopdf` were updated for RustSec advisories; the two remaining unfixable transitive advisories are documented individually in `.cargo/audit.toml`.

## [1.3.1] - 2026-09-16

### Changed

- **Contributors:** CI and `release-check.sh` now run every detection-corpus binary even after an earlier one fails (#582), and the README documents the 1.3.0 features and renamed rule ids.

### Fixed

- **JSON:** Accessibility subcategory and security category scores now appear as `pages[].detail.accessibility_subcategory_scores` and `security_category_scores` in single reports instead of only in the PDF (#581).
- **Dependencies:** The yanked `spin` 0.9.8 was replaced by 0.9.9 (#583).

## [1.3.0] - 2026-09-15

### Added

- **WCAG:** New checks for redundant roles, links used as buttons, video captions, transcripts and native `<video controls>`, and HTML content-model violations under 1.3.1 (#579); a "Leichte Sprache" version and login functions in scope of BFSGV §19 Nr. 3 are now reported as signals.
- **Journeys:** The form-error journey now flags live regions inserted only after submit, which screen readers may not announce.
- **Screen reader:** Duplicated accessible names such as "Kontakt Kontakt" are now detected.
- **Reports:** New HTML5 conformance module (`html-conform`) runs with `--full` and gets its own JSON blob and PDF chapter (#551).
- **Reports:** New "BIK für Alle" editorial guide mapping regroups findings by the chapters editors know; always in JSON as `bik_guide`, in the PDF with `--annex bik`.
- **PDF:** Tagged, PDF/UA-1 conformant output with document language, title and bookmarks (#573).
- **PDF:** The management summary now explains score drivers, a compact findings matrix precedes the WCAG detail cards (#570), manual-review findings carry VoiceOver/NVDA self-test steps, a fixed list names criteria only manual testing can cover, and Commerce gets its own chapter.
- **JSON:** Every report now carries a `build_id` with the git short SHA, marked `-dirty` for uncommitted builds.
- **Batch:** Batch reports now show crawl depth from the internal link graph, long or looping redirect chains, assets minified on some pages only, recurring missing SEO/social tags, and sitemap URLs blocked by `robots.txt` (#536, #537, #546, #548, #549).
- **SEO:** New checks for `og:image`/`twitter:image` without alt text, fonts without preload, header/`<meta charset>` mismatch, trailing-slash and http-to-https redirects and wrong `Content-Type` headers; `--check-ssr-content` flags content missing without JavaScript (#534), and the opt-in `--ai-transparency` reads C2PA provenance of images.
- **Performance:** `--isolate-third-party-impact` measures the TBT cost of the top third-party origins (#531), and duplicate assets and duplicate JS library versions are now reported (#547).
- **Security:** New checks for HSTS preload ineligibility, no-op `Permissions-Policy` and publicly reachable source maps, plus the opt-in `--dns-check` for CAA, DNSSEC and SPF (#535, #538, #545).
- **UX:** The opt-in `--design-quality` module reports readability heuristics and alt-text quality issues without affecting scores (#528).
- **CLI:** New `--color` and `--progress` flags, terminal output via Runemark, logs on stderr so `--format json` stdout stays machine-readable, and every audit request now sends Do-Not-Track and Global Privacy Control (#529, #530).

### Changed

- **WCAG:** The rule ids `label` and `link-name` (2.4.9) were renamed to `control-missing-label` and `link-name-only`, and unrelated checks sharing a WCAG criterion are no longer merged into one finding.
- **WCAG:** Heuristic keyboard-trap, focusable-without-role and generic-link-text-with-context findings are now warnings instead of violations (#569).
- **Reports:** Modules are now classified by measurement type, Dark Mode reads as an optional feature instead of a compliance gap, single-page reports no longer claim site-wide effects, and WCAG 2.2-only criteria are labelled (#572, #575, #577).
- **Security:** Headers are grouped by risk tier, and stacked low-severity misses no longer produce "critical" band text (#578).
- **Contributors:** A WCAG detection corpus now covers all 116 rule ids against live Chrome, with corpora for security headers, vulnerable libraries and structured data, and runs in CI; the release profile uses thin LTO.

### Removed

- **CLI:** The `x86_64-apple-darwin` release binary is no longer built.

### Fixed

- **WCAG:** Rules that could never fire now detect real problems (`aria-prohibited-attr`, `aria-valid-attr`, `video-caption`, `svg-img-alt`, `aria-invalid`, title-only names), page-level violations are no longer demoted to warnings, all findings carry a `rule_id`, and contrast now samples pixels behind opacity, blend modes and images (#527, #562–#568).
- **WCAG:** `[rules] disabled`/`enabled_only` in `auditmysite.toml` now apply to every rule including contrast, reflow and page rules, and the legacy `ignore` field is honoured (#560, #561).
- **Reports:** The BFSG check no longer reports unevaluated criteria as passed, image alt counts agree across modules, explanations no longer attach to unrelated findings, and several English leaks in German reports were fixed (#571, #574).
- **PDF:** Positive dimensions are no longer listed among the top risks (#576).
- **Batch:** Batch runs no longer leak a Chrome process, a stalled page no longer hangs the batch, and progress output finishes before the report renders (#580).

## [1.2.2] - 2026-07-24

### Changed

- **Contributors:** Duplicate helpers and tautological tests were cleaned up, and the unused `tokio-test` dev dependency was removed.

### Fixed

- **WCAG:** Malformed `rgba()` colors no longer panic the contrast check, and out-of-range `aria-level` values no longer disable heading gap detection (#519, #520).
- **Scoring:** Content-on-hover (1.4.13) findings are no longer downgraded by a substring collision, and the orientation-lock finding is no longer counted under Forms (#522, #523).
- **PDF:** Low-severity findings now render in the documented blue (#526).
- **SEO:** URLs containing "zaraz" are no longer misclassified as Cloudflare Zaraz tracking (#525).
- **Performance:** Unrelated `core.min.js` bundles are no longer flagged as the core-js polyfill (#524).
- **Browser:** The browser installer now logs errors it previously reported as success (#521).

## [1.2.1] - 2026-07-20

### Changed

- **Contributors:** Shared WCAG evaluation helpers replace duplicated code, and assertion-free placeholder tests were removed (#515–#517).

### Fixed

- **WCAG:** Label-in-name (2.5.3) no longer fails labels split across elements, compound widgets with summarizing `aria-label` become warnings, and evidence text is no longer cut mid-word (#513).
- **Screen reader:** Skip links phrased like "Zum Inhalt springen" are now recognized (#513).
- **PDF:** Batch template clusters render as a list, so long selectors no longer break the page layout (#518).
- **Batch:** Sitemap diagnostics now compare against the full sitemap instead of the audited sample, ending false "linked but not in sitemap" entries (#514).

## [1.2.0] - 2026-07-17

### Added

- **CLI:** New `report-lint` subcommand checks a JSON report for inconsistent scores, grades, certificates and counts, with `--fail-on` and `--typst-source` (#507).

### Changed

- **Reports:** All score bands and metric definitions now come from one canonical metric registry, documented in `docs/OUTPUT_CONTRACT.md` (#506).
- **Contributors:** Added the report-critic skill, an output coverage matrix, a regression corpus and visual PDF regression tests, and CI now installs poppler so PDF smoke tests actually run (#508–#512).

### Fixed

- **Reports:** The source-quality heading signal now requires exactly one H1 that comes first, instead of contradicting the SEO findings.
- **PDF:** English reports no longer show German SEO details or severity labels, tech stack categories show readable names, and table row stripes are neutral gray.

## [1.1.0] - 2026-07-16

### Added

- **WCAG:** New rules for Focus Not Obscured (2.4.11/2.4.12), Pause, Stop, Hide (2.2.2), Meaningful Sequence (1.3.2), Redundant Entry (3.3.7), Target Size (2.5.8) and Text Spacing (1.4.12).
- **WCAG:** Contrast findings now also report APCA Lc as supplementary information.
- **Journeys:** New commerce journeys check add-to-cart feedback (4.1.3) and quantity-stepper keyboard operation, and the form-error journey now tests up to three forms per page and flags unmanaged focus.
- **Reports:** New EN 301 549 mapping annex, always in JSON (`en301549_annex`, batch `en301549_rollup`) and in the PDF with `--annex en301549`.
- **PDF:** Finding cards in single reports now show an element screenshot, DOM path and measured values, plus a plain-language explanation of the impact.
- **Batch:** Batch reports now identify shared template causes across pages as `template_clusters`.
- **CLI:** New `--format sarif` for GitHub Code Scanning and `--report-mode`, which exits 0 whenever the report was generated (#503).

### Changed

- **WCAG:** AAA and WCAG 2.2-only criteria now always rate as low BFSG relevance.
- **Scoring:** Single-page pass now uses the accessibility score, unmeasured performance no longer counts as 0, and batch verdicts and risk now account for blocking issues and per-page risk.
- **Contributors:** New guard tests check CDP property names and taxonomy severity collisions; architecture documentation was synced.

### Removed

- **SEO:** Commerce no longer classifies cart and checkout pages and no longer reports payment methods, free-shipping thresholds or guest checkout, since a single cold page visit cannot reach them.

### Fixed

- **WCAG:** 18 rules that read non-existent accessibility-tree properties now check the real DOM state, and findings sharing a criterion no longer escalate each other's severity.
- **Journeys:** The form-error journey never clicks purchase or booking buttons.
- **Screen reader:** Region and form landmarks are now recognized, and a collapsed disclosure menu no longer reads as missing navigation (#504, #505).
- **PDF:** Remaining localization gaps and singular/plural errors were fixed, AI visibility word counts are measured per section, and table headers stay with their rows across page breaks.

## [1.0.0] - 2026-06-28

### Added

- **WCAG:** New `form-no-submit` (3.2.2) and `presentation-semantic-children` (1.3.1) rules and granular landmark checks.
- **JSON:** `summary.score` is now available as an alias for `overall_score`, and the screen-reader summary reports `audit_quality` for suspected consent walls (#483).
- **Batch:** Batch reports now detect duplicate titles, descriptions and H1s, canonical conflicts and non-reciprocal hreflang links (#423).
- **SEO:** New shop-gated Commerce module reports product structured data completeness, page kind and links to trust pages such as Impressum, AGB and Widerruf, with a site-wide roll-up in batch reports.

### Changed

- **Scoring:** Accessibility areas no longer collapse to 0 from a few findings, well-configured pages keep a SEO baseline, and screen-reader legal flags come only from confirmed Level-A blockers (#484–#486).
- **Reports:** The diagnosis now counts occurrences, and root-cause analysis is scoped to WCAG findings.
- **PDF:** Single reports were redesigned as a product report with a cover dashboard, quality radar, one chapter per module with a key takeaway, and an action plan grouped by systemic and local problems.
- **PDF:** The performance disclaimer now explains why lab values differ from PageSpeed.
- **JSON:** JSON output is now always canonical English (#406).
- **Contributors:** `AuditReport` was grouped into domain sections, interpretation logic moved out of the builder, and CI was repaired.

### Removed

- **PDF:** The "Hebelwirkung" callouts were removed.
- **CLI:** `--semantic-eval`, the `[semantic_eval]` config section, the `semantic-eval` feature and the `advisory_findings` output were removed.

### Fixed

- **WCAG:** Lazy-load image placeholders and decorative SVG icons no longer count as missing alt text (#487), and 4.1.1 Parsing is no longer critical (#488).
- **Screen reader:** The hardcoded BFSG deadline was removed, empty and synthetic node ids are dropped, and announcement deserts map to 1.3.6 (#479–#482).
- **Reports:** A failing verdict now always gates the certificate, and plural forms are correct throughout.
- **PDF:** A custom `--logo` is shown on the cover again.
- **JSON:** Cached reports now include full module detail.

## [0.49.1] - 2026-06-15

### Changed

- **PDF:** Score gauges are now progress-arc gauges with a fixed-height label and no longer overlap the cover (renderreport 0.2.23).
- **PDF:** The cover headline now labels the finding count as accessibility-scoped (`N Accessibility-Befunde`), so the WCAG-only count no longer reads as a contradiction with other modules.
- **PDF:** Customer-facing passages are now plain language only; the technical interpretation and meta-label prefixes such as "Was das für Kunden bedeutet:" are no longer appended.
- **PDF:** Short indicator modules such as Best Practices and Tech Stack now flow after the preceding module with a light separator instead of each taking a nearly empty page.
- **PDF:** The "no issues" note for Security, SEO and Page Health now appears when a section has no significant findings (score >= 80, or no High/Critical issue for Page Health), not only when it has zero findings (#446).

### Fixed

- **PDF:** Search-experience warnings and component explanations are now re-derived in the run locale instead of leaking canonical English into German reports (#406).
- **PDF:** The "Befunde nach Ursache" divider page is no longer left mostly empty; the root-cause analysis now flows onto it.
- **PDF:** Structured-data counts are now pluralised correctly ("1 Schema" instead of "1 Schemas").
- **PDF:** HTML and selector evidence is now compacted (collapsed whitespace, `data:` URI bodies shortened), so huge tokens no longer blow up table row heights.
- **Performance:** The combined page-weight penalty is now capped at 45 points, so heavy pages with good vitals no longer collapse to a score of 0 (#461).
- **Security:** Vulnerable-library detection now distinguishes Lodash from Underscore and no longer mislabels Lodash 4.x as Underscore.

## [0.49.0] - 2026-06-14

### Fixed

- **Scoring:** Critical risk via the breadth path now triggers on `legal_flags >= 3` alone, and `driven_by`/summary now name the actual trigger (breadth vs. volume) (#457).
- **PDF:** Security, SEO and Page Health sections without findings now show a neutral "no issues found within the audited scope" line instead of staying silent (#446).
- **Performance:** DOM size is now a degressive penalty bounded at 35 points instead of a hard cap at 59, so pages with perfect vitals no longer rank below pages with poor CLS (#455).
- **Performance:** Throttled network profiles now apply the same content-weight caps as the headline score, so a slower profile no longer out-scores a faster one (#456).
- **CLI:** `--reuse-cache` now loads cached audits reliably and rebuilds the full report including UX, journey, dark mode, AI visibility, tech stack and screen-reader data; the verdict is now identical across `--format` (#404).
- **CLI:** The cache signature now includes language and semantic-eval settings, and corrupt cache entries are treated as a cache miss with a warning instead of aborting the run (#405).

## [0.48.0] - 2026-06-13

### Changed

- **Scoring:** Critical risk is now reserved for systemic legal exposure (>= 3 distinct Level A barriers with a critical occurrence, or >= 5 critical occurrences); a single isolated barrier is now High (#250).
- **Scoring:** The catastrophic accessibility band now uses a square-root curve, so sites with many critical failures keep a visible ordering instead of clustering at 4-9.
- **JSON:** Headline Core Web Vitals adopted from the throttled mobile pass are now tagged `lab_throttled_mobile` instead of `lab_headless` (#406).
- **Performance:** The DOM-node score cap now only applies to excessive DOMs (> 6000 / > 10000 nodes), so ordinary content sites no longer cluster at 59.
- **Performance:** The lab-data disclaimer now states that headline vitals use the Lighthouse mobile preset and are intentionally conservative.
- **Mobile:** The mobile score now compresses large penalties instead of collapsing to zero, keeping poor and catastrophic pages distinguishable.

### Fixed

- **JSON:** WCAG findings and SEO heading-issue findings are now canonical English in JSON, while the PDF re-derives the German texts (#406).
- **Performance:** CLS is now the maximum session window as defined by current Core Web Vitals, instead of the cumulative sum of all shifts.
- **Performance:** LCP and TBT are now bounded to the load window, so post-load paints and idle long tasks no longer inflate them.
- **Performance:** Best-practices console-error penalties now count distinct messages instead of raw occurrences.
- **Security:** A present but permissive CSP no longer scores below having no CSP at all; its penalty is now capped at the missing-CSP penalty.
- **UX:** The large-DOM finding now appears whenever its Visual-Hierarchy penalty applies, so the score no longer drops without a stated reason.

## [0.47.0] - 2026-06-13

### Added

- **Screen reader:** The screen-reader summary and risk score now appear in the JSON envelope and in a dedicated PDF section (#411).
- **Reports:** Reports now include a search experience score.
- **Reports:** Reports are now available in German and English; JSON is canonical English and only the PDF follows the run language (#406).
- **PDF:** The single report now has a gauge-based cover, a management summary page, a root-cause analysis section and a timeframe roadmap.
- **PDF:** A customer diagnosis panel, customer-impact categories per finding, recurring-pattern panels, module customer-context callouts and audit-confidence notes were added (#397–#403).

### Changed

- **WCAG:** Tab and table checks no longer overstate passes: `aria-controls` is reported as declared rather than as a valid tabpanel, and only explicit `headers` associations count (#444).
- **Scoring:** Non-contributing modules now carry `weight_pct=0` (weights previously summed to 125%), and dark mode, AI visibility, source quality and tech stack appear as graded indicators (#447).
- **PDF:** Certificate thresholds now follow the official bands (90/75/60/40) with the labels SEHR GUT/GUT/STABIL/AUSBAUFÄHIG/UNGENÜGEND (#449).
- **PDF:** Roadmap wording is now softer ("Verbesserungsfelder", "Geringer Aufwand" instead of "Maßnahmenplan", "Quick Win") (#400).
- **CLI:** Semantic evaluation is now on by default and can be disabled with `--no-semantic-eval` (#451).
- **Contributors:** Removed dead always-pass tree stubs that inflated pass counts, made module-label tests feature-aware, PDF tests now assert values instead of prose, fixed clippy errors and removed the nightly CI schedule (#443, #452).

### Fixed

- **Journeys:** The form-error-announcement journey now compares the real pre-submit URL, so a swallowed error is only flagged when no navigation occurred (#442).
- **PDF:** The batch report now honours `--lang`, and the literal `label-issue` key no longer appears in the SEO issue table.
- **Batch:** Batch audits now guard against too small sample sizes (#450).
- **Performance:** A measurement with no Web Vitals is now treated as not measured instead of scoring 0 and deflating the overall score (#448).
- **CLI:** Rebuilding a report from cache now uses the correct locale (#411).

## [0.46.3] - 2026-06-05

### Fixed

- **WCAG:** The contrast check now skips `aria-hidden="true"` subtrees and `role="presentation"`/`none` elements (#395).
- **JSON:** The JSON report schemas now include `verdict` and `verdict_reasons`.

## [0.46.2] - 2026-06-05

### Changed

- **Contributors:** Fixed clippy and compile failures in rules and test fixtures.

### Fixed

- **WCAG:** The accordion pattern now treats `<summary>` (`DisclosureTriangle`) as a button-like trigger and skips the `<details>` container, removing false positives (#394).

## [0.46.1] - 2026-06-05

### Added

- **CLI:** A pass/warn/fail verdict now sets exit codes 0/1/2, configurable via `[verdict]` in `auditmysite.toml`; `verdict` and `verdict_reasons` were added to all JSON outputs, and runtime errors now exit 3 (was 2) (#390).

### Changed

- **Contributors:** Added a parity-check script against axe-core and pa11y and updated the README feature scope (frame scan, 100+ rules).

### Fixed

- **WCAG:** The 1.4.13 taxonomy title typo "Inzugängliche" is now "Unzugängliche".
- **WCAG:** SVG `suggested_code` now shows both the decorative (`aria-hidden`) and the informative (`role=img` + title) option (#393).
- **SEO:** LocalBusiness schema detection no longer picks the wrong schema and falls back to raw JSON when needed (#390, #391).
- **UX:** Dark mode detection now recognises class-based patterns such as `html.dark` and `[data-theme=dark]` (#392).

## [0.45.3] - 2026-06-03

### Added

- **WCAG:** Same-origin iframes are now scanned for image-alt, button-name, link-name, label, duplicate-id and html-has-lang, with composite selectors such as `iframe#widget [frame] img` (#381).
- **WCAG:** New `aria_valid_attr_value` rule (4.1.2, Level A) validates boolean, tristate, token-list and ID-ref ARIA attributes (#389).

### Fixed

- **WCAG:** The duplicate-id check now runs at Level A instead of only at AAA (#388).
- **WCAG:** The contrast check now walks all text nodes instead of a fixed selector list, bringing detection closer to axe-core (#387).

## [0.45.1] - 2026-06-03

### Added

- **Performance:** The performance score now applies penalties and hard caps for critical page weight, JS size, request count and DOM complexity, and the PDF explains the capping in restructured Vitals/Resources/Bottlenecks subsections.

### Changed

- **Batch:** Failed URLs now appear immediately in the progress output, and timeouts are no longer retried.
- **Dependencies:** renderreport was updated to 0.2.22, picking up the Typst 0.13 keyword-quoting fix; the unused `fluent-syntax` dependency was removed.
- **Contributors:** Split several high-complexity functions, added a `cargo-machete` CI check, removed stale axe-core parity references and compare scripts, and updated the batch JSON schema and PDF tests (#382–#386).

### Fixed

- **Batch:** Hung pages no longer stall a batch indefinitely; each attempt and each page reset now has a timeout.

## [0.45.0] - 2026-05-30

### Added

- **PDF:** The executive summary now explains the score driver and the WCAG coverage scope, including a dedicated note for 100/100 reports (#369, #372, #379).
- **JSON:** Findings now carry `confidence`, `false_positive_risk`, `verification`, `complexity`, `complexity_reason`, `expected_impact`, `bfsg_relevance` and a `remediation_priority` decoupled from severity (#370–#377).
- **JSON:** The summary now includes `wcag_coverage`, `accessibility_score_breakdown`, `management_risks` and `top_actions` (#369, #372, #378, #380).
- **Batch:** Batch JSON now includes `internal_comparison` with module extremes, outlier URLs and root causes (#375).

### Changed

- **Reports:** Time-based effort labels were replaced by complexity levels (#374).

## [0.44.0] - 2026-05-30

### Changed

- **PDF:** Assessment labels now follow the score band ("Gute Basis" no longer appears below 60), filler words were removed, and the verdict tier now drops when legal flags exist (#355, #356).
- **PDF:** The performance interpretation now names the weak metrics and contrasts them with CLS/TBT (#367).
- **PDF:** "Next steps" now come before the technical module details, and heading levels and section titles were made distinct (#361–#363).
- **Dependencies:** renderreport was updated to 0.2.20, so breakable tables now span pages after keep-with-next headings (#366).
- **Contributors:** Fixed lints surfaced by clippy 1.95.

### Fixed

- **PDF:** Raw rule identifiers such as `aria_hidden_focus` now resolve to localized fix texts, and mobile and journey categories are now localized (#357, #358).
- **PDF:** Repeated journey findings are now deduplicated, and diagnosis callouts no longer repeat their title (#358, #359).
- **PDF:** The appendix "Vollständige Fundstellen" header now only appears when violations exist, and the first criticality tier now stays on the divider page (#364, #365).

## [0.43.0] - 2026-05-30

### Changed

- **PDF:** The content-depth label now marks its 300-word threshold as a guideline, and the SEO interpretation notes when strong technical SEO meets weak content depth (#3, #4).
- **PDF:** The mobile touch-target sentence now notes when the issue is locally limited, so a high score with many small targets reads consistently (#8).
- **Contributors:** Introduced `AuditContext` and pluggable snapshot dispatch so `NormalizedReport` is a pure serializable type (#351–#354).

### Fixed

- **PDF:** Empty pages caused by double page breaks no longer appear (renderreport 0.2.19).

## [0.42.0] - 2026-05-29

### Changed

- **CLI:** A failing audit module now logs a warning instead of aborting the whole audit (#341).
- **Contributors:** Wired `AuditCatalog` into the pipeline, added a `ReportRenderer` trait skeleton, and added integration tests for browser pool, batch errors, consent and screenshots plus unit tests for Lighthouse scoring curves (#339–#348).

### Fixed

- **Screen reader:** The landmark score now requires all four landmarks (banner, navigation, contentinfo, main), and each missing one is now flagged (#350).
- **Screen reader:** Links and buttons whose name contains icon-font characters, and headings that do not start with H1 or place H1 after H2/H3, are now flagged (#350).

## [0.41.0] - 2026-05-29

### Added

- **JSON:** Reports now include an `interpretation` block with pre-computed DE/EN assessment texts (#336).
- **Batch:** The batch summary now includes `top_recurring_rules`, `violated_rule_count`, `legal_flags` and `risk` (#337).

### Changed

- **Contributors:** Moved all modules onto a catalog-driven pipeline, replaced inline page-rule calls with a table-driven catalog, turned builders into pure mappers, and added per-module fixture tests; audit output is unchanged (#330–#335, #338).

## [0.40.1] - 2026-05-29

### Changed

- **Scoring:** `blocking_issues` now counts only Medium and higher findings, so low-severity advisories no longer raise the risk level.
- **Contributors:** README corrections and a clippy fix in the screen-reader analyzer.

### Removed

- **Reports:** The report history feature and its `history.json` sidecar were removed.
- **CLI:** The `--compare` mode was removed.

### Fixed

- **WCAG:** The accordion pattern no longer reports a false `aria-controls` violation on collapsed triggers whose target panel is hidden.

## [0.40.0] - 2026-05-29

### Added

- **Screen reader:** New screen-reader module builds the reading sequence from the accessibility tree, renders announcements, provides virtual navigation lists (headings, landmarks, links, forms, tables) and detects issues from the reading sequence (#323–#326).
- **Screen reader:** Findings are now mapped from WCAG via EN 301 549 to BFSG paragraphs, and the screen-reader audit is exported as JSON (#327, #328).

## [0.39.2] - 2026-05-28

### Fixed

- **Reports:** Several consistency issues between report values were fixed.

## [0.39.1] - 2026-05-28

### Added

- **Reports:** The rule `seo.headings.skipped_level` now has its own explanation in German and English.

### Changed

- **Contributors:** `ARCHITECTURE.md`, `CHANGELOG.md` and `reports/README.md` were removed from the repository.

### Fixed

- **Journeys:** A missing `<main>` in the mobile accessibility tree is now reported as low-severity friction with a reduced penalty when the element exists in the DOM but a consent overlay hides it, instead of contradicting the passing `landmark-main-present` check.
- **JSON:** Recurring-rule aggregation now uses the finding severity instead of a rule-registry lookup.
- **CLI:** `--request-mode` now reaches the run, so the request-mode prompt is skipped as intended.

## [0.39.0] - 2026-05-28

### Added

- **Browser:** Interactive runs now ask whether the browser should identify as a normal browser or as the `auditmysite/<version>` bot; `--request-mode browser|bot` sets this for scripted and CI use, which gets audits past bot checkpoints such as Vercel's Security Checkpoint.

## [0.38.0] - 2026-05-28

### Added

- **JSON:** `PageEntry` now carries flat per-module score fields. (#309)

### Fixed

- **Scoring:** The Low risk summary now mentions high-severity interactive findings. (#311)
- **JSON:** Single-page reports now populate `top_recurring_rules`. (#308)
- **JSON:** `affected_selectors` now also includes plain tag selectors. (#310)

## [0.37.0] - 2026-05-28

### Changed

- **Journeys:** Journey detection and reporting were expanded.
- **Scoring:** `interactive_critical_issues` now feed into the risk assessment score and level.
- **Reports:** The executive summary and the single-page report were improved.
- **CLI:** Additional pipeline and config options are now available.

## [0.36.0] - 2026-05-28

### Added

- **Journeys:** The new Accessibility Journey layer drives the page with real keyboard and pointer input via `--interactive off|basic|full` (default `off`, also `[interactive]` in `auditmysite.toml`) and records a reproducible tab-walk trace. (#292)
- **Journeys:** Interactive findings now cover hidden focusable elements, missing focus indicators, reverse tab-order jumps, and the patterns skip link, modal, disclosure, accordion, tabs and menu. (#296)
- **Journeys:** Form error announcement, SPA navigation (title, H1 and focus after client-side routing) and a link inventory (generic or duplicate link texts, heading skips, missing H1, unlabelled landmarks) are now checked.
- **PDF:** Single-page reports now include an Accessibility Journey detail section.
- **Batch:** Batch PDF reports now include an aggregated interactive journey section.
- **CLI:** `--semantic-eval` adds advisory findings from a semantic AI evaluation, and `--export-snapshot` exports the AX tree and journey trace as YAML.

### Changed

- **Journeys:** Generic link-text stopwords now come from the locale files, with German and English always merged so bilingual sites stay covered. (#299)
- **Contributors:** README documentation for the journey layer and semantic evaluation, integration-test fixes, clippy fixes and formatting. (#300)

### Fixed

- **WCAG:** Empty links are no longer reported twice under 2.4.4 and 4.1.2, which had inflated `severity_counts`.
- **Journeys:** Public-sector pages are now recognised by keywords such as Rathaus or Bürgeramt, and the hub threshold was lowered from 80 to 40 links.
- **Reports:** Heading findings now carry a proper technical-impact text instead of the raw analyzer message.
- **JSON:** `accessibility_journey`, `interactive_findings` and `advisory_findings` now appear in the JSON output.
- **SEO:** `soft_404_status` is now only set for a confirmed soft 404, not for real 404 or 301 responses.
- **Performance:** Fonts loaded via `@font-face` are no longer counted as CSS, and CSS coverage now reports 0 % instead of 100 % when nothing was measured.
- **UX:** Contact information in headings and static text is now detected, not only in links.

## [0.35.0] - 2026-05-27

### Added

- **PDF:** A measurement-limitations callout now explains these warnings and names the gap between desktop and LhMobile score when it is 15 points or more. (#289)
- **JSON:** The summary now contains `lh_mobile_score`, `ux_score` and `journey_score`.
- **Performance:** Implausible headless measurements (TBT 0 on heavy pages, SI/TTI falling back to LCP, INP not measurable, LCP not captured under throttling) are now detected as `measurement_warnings`. (#291)
- **Security:** Missing `Permissions-Policy`, COOP and CORP headers are now reported as low-severity issues.

### Changed

- **Scoring:** Any critical finding now caps the overall score at 49 instead of 94/96.
- **Contributors:** The redundant release build was removed from the CI workflow.

## [0.34.0] - 2026-05-26

### Added

- **PDF:** Technical SEO issues now appear in the PDF report.
- **Batch:** Batch audits now detect orphan pages not linked from any audited page and conflicting `@id` entities in the schema graph across pages.
- **SEO:** New checks for missing SRI `integrity` on external scripts and stylesheets, synchronous scripts in `<head>`, broken fragment anchors, generic link texts and internal links with query parameters.
- **SEO:** `hreflang` is now validated for `x-default` and self-reference, and noindex pages listed in the sitemap are flagged.

### Changed

- **Contributors:** Clippy fixes and a missing test fixture field.

## [0.33.0] - 2026-05-26

### Added

- **WCAG:** `landmark-banner-present` and `landmark-main-present` (1.3.1) now fire when the banner or main landmark is missing entirely.
- **SEO:** `images_without_srcset` flags `<img>` elements outside `<picture>` without `srcset`.
- **SEO:** JSON-LD types such as Article, Product, Event, Recipe and Review are now checked for required properties per Google's rich-result spec, including items nested in `@graph`.

### Changed

- **Contributors:** Release artifacts in CI are now kept for one day only.

## [0.32.0] - 2026-05-24

### Added

- **JSON:** Summary and page entries now state `severity_counts_scope` (`wcag_only`). (#274)

### Changed

- **PDF:** Remaining hard-coded report texts (history trend, structural cause, module names, metrics) are now localized. (#258)
- **Performance:** Speed Index now counts towards the performance score at 10 % instead of being measured but ignored. (#271)

### Fixed

- **WCAG:** The frame-title check now skips visually hidden iframes. (#267)
- **WCAG:** Raw-URL link text is no longer flagged under 2.4.4 when the surrounding context describes the link. (#269)
- **SEO:** FAQ and question headings now get the 100-character limit before the level-based 70-character cap applies. (#268)
- **Performance:** `load_time` and `domContentLoaded` are now only recorded when the navigation event fired, TTI is no longer reported below LCP, and CLS and long-task entries are no longer double-counted. (#270, #272, #273)

## [0.31.0] - 2026-05-23

### Changed

- **Batch:** Recurring-rule severity in batch reports now comes from the taxonomy base severity instead of the highest per-finding severity. (#279)

### Fixed

- **WCAG:** Screen-reader-only, off-screen and zero-size text no longer produces `contrast.weak` false positives. (#280)
- **WCAG:** The 1.4.13 title and description now match what the check actually tests. (#278)
- **WCAG:** The umbrella rules `structure.missing` and `name_role.missing` no longer carry an incorrect axe id. (#277)
- **WCAG:** `aria_hidden_focus.invalid` and `aria_prohibited_attr.invalid` now have their own explanations instead of colliding with other 4.1.2 texts. (#275)
- **Reports:** Findings without fix guidance no longer emit empty strings, and 1.4.10 Reflow now has an explanation. (#276)

## [0.30.0] - 2026-05-23

### Changed

- **WCAG:** A raw URL used as link text is now a warning instead of a violation.
- **Batch:** Batch PDF report texts are now localized in German and English.
- **SEO:** FAQ headings may now be up to 100 characters long.
- **Contributors:** Clippy fixes for Rust 1.95.

### Fixed

- **WCAG:** The frame-title check no longer flags hidden, `aria-hidden` or 0×0 px iframes.

## [0.29.0] - 2026-05-22

### Added

- **WCAG:** The rule taxonomy now maps more than 20 additional criteria, including 1.2.1, 1.2.2, 2.2.x, 2.4.8/2.4.9, 2.5.x, 3.1.x, 3.3.x and 4.1.1.
- **JSON:** Tech stack and pattern analysis are now part of the JSON output.

### Fixed

- **WCAG:** Semi-transparent foreground and background colors are now composited over white before the contrast ratio is calculated.

## [0.28.0] - 2026-05-22

### Added

- **WCAG:** Text on images and gradients is now checked by sampling screenshot pixels, and confirmed violations are separated from manual-review cases. (#264)

### Changed

- **PDF:** Module details and action texts now come from Fluent locale files instead of inline language checks. (#258)
- **Contributors:** New contrast test fixtures.

### Fixed

- **WCAG:** Contrast now resolves `rgba` and `opacity` foreground and background colors correctly in the browser. (#266)
- **PDF:** The empty "SEO & Sichtbarkeit" section header page is gone.

## [0.27.0] - 2026-05-21

### Changed

- **Contributors:** First release in this repository with a clean repository root; earlier history is archived in branch `archive/main-history`.
