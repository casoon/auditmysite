---
title: Development history
sidebarLabel: Development history
description: The chronological development log of auditmysite — finding, fix and verification for each change. The changelog lists the same changes per release.
order: 90
---

This page keeps the full development log as it was written during development (translated from
the German original): for each change the finding, the fix and how it was verified. The
[changelog](https://github.com/casoon/auditmysite/blob/main/CHANGELOG.md) summarizes the same
changes per release.

Detailed, chronological development history for auditmysite — each entry documents a finding,
the fix, and how it was verified. Extracted from `CLAUDE.md`'s former "Current State" section
so `CLAUDE.md` itself stays focused on working rules and a short current-state summary. Newest
entries first (unchanged order from before the extraction).


- **Unreleased — Link purpose from `a11y-rules` 0.24.0 (casoon/barrierlab#13, #51):**
  `link_purpose` and `link_purpose_link_only` had been left behind at B5 (#743), although #690
  had moved them there. They now run in the shared rule set, both files are deleted, and the old
  IDs resolve via `LEGACY_RULE_IDS`. `links/generic-name` uses the previous local list (15
  languages, including navigation words such as "Start") and the context evaluation from #569;
  with context it is Low instead of a separate ID `link-name-context`. New: `links/url-name`,
  `links/single-character-name`, `links/new-window-unannounced` (checks `target="_blank"` instead
  of `aria-haspopup`, 3.2.5 AAA with its own taxonomy entry and explanation text) and
  `links/purpose-link-only` (2.4.9). All are `REVIEW`; generic link texts without context were
  violations locally. 2.4.9 and 3.2.5 thus count as "notes only", the AA numbers (30/25) stay.
  Live inros-lackner.de: 5 × `link-name-context` → 5 × `links/generic-name` (Low); detection
  corpus with Chrome green.

- **Unreleased — Report quality after the inros review:**
  An external review of the single and the sitemap report for inros-lackner.de was checked point
  by point against code and the live site; only confirmed findings were changed, refuted ones
  remain documented. Measurement errors: Best Practices and Tech Stack reported 100 without any
  data collection (now a check status), CSS coverage had numerator = denominator (316/316 →
  316/5076), the viewport calculation used different module weights than the published ones,
  Security deducted for context-dependent headers (30 → 40), three WCAG criteria were missing from
  the coverage list (manual 22 → 25). Batch: link graph starting from the home page, measured
  sitemap status distribution, sample disclosed, range instead of "clean pages", topic comparison
  with the correct unit (`similarity_percent`, `shared_terms` as an alias) from the main content
  instead of the body, page groups and shared blocking files (5 instead of "100"). Language:
  findings with "möglicher Rechtsrelevanz" (possible legal relevance) instead of "rechtlich
  relevant" (legally relevant), heuristic UX/journey statements marked as such, sample messages and
  schema matrix localized. PDF: finding matrix with full titles, no more empty pages or stranded
  headings (renderreport 0.6.2), batch PDF no longer failed on PDF/UA. `cross` in the release
  workflow pinned to a commit. Evidence: live runs single and batch (20 of 685) before/after,
  rendered pages inspected.

- **Unreleased — Security advisories of October 2026:** `rustls` 0.23.46, `rustls-webpki`
  0.103.16 and `webpki-roots` 1.0.11 in the lockfile (RUSTSEC-2026-0347/0348/0349, -0381, -0382);
  `zip` 2 → 9 and `c2pa` 0.90 → 0.91 (RUSTSEC-2026-0367/0368/0369, both copies of `zip`).
  The browser installer now recognizes directory entries via `ZipFile::is_dir()`, new unit test
  for extraction. `cargo audit`: no vulnerabilities.

- **Unreleased — Review notes, control contrast and tree regions from `a11y-rules` 0.23.0
  (#722, #698, #700, #738; casoon/barrierlab#39, #21, #23, #41):**
  The seven review-note stubs `pointer_gestures`, `motion_actuation`, `no_timing`,
  `no_interruptions`, `re_authenticate`, `unusual_words` and `help` run as `manual/*` in the
  shared rule set. Still `UNTESTED`, once per page, but only with a trigger (gesture widgets,
  inline script with motion sensors, fields, live regions, login, text). The heuristic part of
  `help` (every text field without `aria-describedby` as a 3.3.5 violation) is dropped.
  `non_text_contrast_css` runs as `contrast/control-insufficient` /
  `contrast/control-undetermined` (1.4.11) with `appearance`, `border-top-*` and its own surface
  from the DOMSnapshot; if the background is missing, `UNTESTED` instead of a comparison against
  white. Deleted: eight rule files; the old IDs resolve via `LEGACY_RULE_IDS` and
  `LEGACY_WCAG_MAP`. The CDP walk now marks shadow roots and slotted content as separate tree
  regions, IDs apply per region. `html-conform` 0.3.2 no longer treats `<legend>` after
  whitespace in the `<fieldset>` as invalid.
  Live before → after (2026-10-23, AA, desktop): upscale.casoon.dev/aufgaben/ score 92 → 97
  (`html-content-model` at the `<legend>` is dropped); bund.de `ids/duplicate` 1 → 0 (`#content`),
  sachsen-anhalt.de 2 → 0 (`#panel`, `#card__heading`) — IDs from the shadow roots of the
  muse components, scores unchanged. bundesregierung.de (AAA): one new note each for 2.5.1
  (Slick carousel), 2.2.3, 2.2.4, 3.1.3 and 3.3.5; 2.2.5 and 2.5.4 silent without a trigger.
  Detection corpus (Chrome) green, expectations switched to the new IDs.

- **Unreleased — Target size, order, endless animation and pointer-only from `a11y-rules`
  (#698, barrierlab#49):** `targets/size` (2.5.8 AA) and `targets/size-enhanced` (2.5.5 AAA, only
  what 2.5.8 does not already report) replace `target-size-minimum` and `target-size`;
  `order/visual-mismatch` (1.3.2) replaces `css-order-reading-sequence`;
  `motion/infinite-animation` (2.2.2) replaces the animation and `<marquee>` part of
  `pause-stop-hide`; `keyboard/pointer-only` (2.1.1, `cursor: pointer` or `onclick` without role,
  tabindex and a control inside it) is new. Deleted: `target_size_minimum`,
  `target_size_enhanced`, `meaningful_sequence`, `pause_stop_hide`. The old IDs keep resolving via
  `LEGACY_RULE_IDS` (rule filter) and `LEGACY_WCAG_MAP` (taxonomy).
  **The verdict changes:** 2.5.8 and 2.5.5 were `FAIL`, now `REVIEW`; a target that is still
  growing during measurement stays `UNTESTED` as before. 1.3.2 and 2.2.2 were and remain review
  notes. Intentionally different: no more upper limits (locally 5 findings, 1000 targets, 5 flex
  containers — which is why the order check did not see the footer of bundesregierung.de and the
  header of berlin.de); `flex-direction: *-reverse` counts; every endless animation counts, even
  short runs (2.2.2 applies to motion that runs for more than five seconds in total; locally only
  runs over 5 s), `prefers-reduced-motion` no longer cancels the finding, paused animations do not
  count (locally they did). No longer checked: the carousel signals of `pause-stop-hide`
  (`data-autoplay`, `data-ride`, `.swiper-autoplay-running`, `data-update-interval`) — no hits on
  the six reference pages. Since a11y-rules 0.22.0 every control counts as a target, regardless of
  `tabindex` (carousel dots with roving tabindex: bundesregierung.de mobile
  `#slick-slide-control00`–`03`, 10 × 10 px, now reported; plus 2.5.5 at `div[role=button]`
  bundesregierung.de and `#moreNews` wetter.com, both real targets). Corpus: new verdict `absent`
  (no finding of any kind), because `pass` only counts violations and never fails for `REVIEW`
  rules; new case `pointer_only`.
  Live before → after (2026-10-05, desktop and mobile, violations / score): AA gov.uk 3 → 3,
  bundesregierung.de 18 → 13 (the 5 local 2.5.8 violations, 4 of them skip links at −99999 px,
  are dropped), wetter.com 137 → 137, n-tv.de 73 → 64 / 36 → 39, spiegel.de 100 → 98 / 24 → 25,
  berlin.de 31 → 31; AAA gov.uk 45 → 40 / 89 → 91, bundesregierung.de 87 → 74, wetter.com 229 →
  223, n-tv.de 390 → 371 / 29 → 34, spiegel.de 347 → 341 / 21 → 22, berlin.de 61 → 53 / 38 → 41.
  New as review note: endless animations on spiegel.de (9) and bundesregierung.de (1),
  order on berlin.de (2), bundesregierung.de and spiegel.de, 2.5.5 without an upper limit
  (gov.uk 82 entries instead of 5 violations).

- **Unreleased — `pointer_events_none` and `animating` (#698, barrierlab#49):**
  `pointer_events_none` is the computed `pointer-events: none` from the DOMSnapshot.
  `animating` comes from a pass before the snapshot, like `obscured`:
  `document.getAnimations()` with `playState` `running` and finite `endTime` — which also covers
  running transitions —, endless animations do not count; an element counts as animated if it
  itself or an ancestor is in it, main document only. The computed `animation-*` values alone are
  not suitable for this: they remain after an animation has ended. Seen filled (2026-10-05,
  desktop, `Some(true)`): `pointer_events_none` bundesregierung.de 114, spiegel.de 120, gov.uk 6,
  berlin.de 2, corpus `target_size_hidden_neighbours` 1; `animating` bundesregierung.de 246,
  wetter.com 89, spiegel.de 23, corpus `target_size_animation` 1 (`a#slow`).

- **Unreleased — Layout fields filled, comparison with local heuristics (#698):**
  `RenderedCdpDocument::layout` now delivers, on every element with a layout object, the fields
  from the DOMSnapshot (new computed styles `flex-direction`, `order`, `min-width`, `cursor`,
  `animation-name/-duration/-iteration-count/-play-state`, `top`, `scroll-padding-top`):
  `flex_reversed` (flex container with `*-reverse`), `order`, `min_width_px` (`auto` → 0, pixel
  values only), `cursor_pointer`, `infinite_animation` (pairwise over the `animation-*` lists:
  name, duration > 0, `infinite`, not `paused`) and `hides_focus`: a fixed or sticky bar in the
  main document, at least 20 px high, half as wide as the page, lower than the visible area (a
  consent dialog over everything is not a bar), which starts above `scroll-padding-top` and
  reaches lower — fixed at its position in the visible area, sticky at its `top`; only the
  outermost one. `obscured` unchanged (only elements with their own text). `focus_visible` stays
  `None`: no pass of this retrieval moves the focus. The heuristics still run only in the
  background (not in `SHARED_RULES`); the local rules remain. Seen filled (2026-10-05, desktop,
  `Some(true)`): `flex_reversed` gov.uk 4, spiegel.de 20; `order` ≠ 0 berlin.de 4,
  bundesregierung.de 2, spiegel.de 2; `min_width_px` > 320 bundesregierung.de 6, wetter.com 2,
  berlin.de 2; `cursor_pointer` on all six; `infinite_animation` spiegel.de 9,
  bundesregierung.de 1; `hides_focus` spiegel.de 1 (`header.sticky`), n-tv.de 1; `obscured`
  wetter.com 190, n-tv.de 61, spiegel.de 32.

- **Unreleased — Rule filter applies to the shared rules (#698):** `[rules] disabled` /
  `enabled_only` and `--disable-rule` previously only worked for the local rules; since contrast
  is shared, `--disable-rule color-contrast` no longer switched anything off. Now
  `wcag::shared::retain_allowed` removes a shared ID if the filter disables it under its own name
  or under an old ID from `LEGACY_RULE_IDS` (`color-contrast` → `contrast/text-*`,
  `link-in-text-block`, `landmark-one-main`, …, only completely superseded rules), and records it
  as `disabled_by_rule_filter`; likewise in iframes.

- **Unreleased — Contrast over images by sampling again (#698, barrierlab#47/#48):**
  `RenderedCdpDocument` delivers `Rendering::sampled_backdrop`: a screenshot beyond the visible
  area (`captureBeyondViewport`), cropped to the candidates and in CSS pixels
  (`scale = 1/devicePixelRatio`), loaded into the page once; per element the WCAG luminances in
  the box (alpha against white, at most 2500 per element on a grid). Page-wide at most 60
  elements, the largest first: own text in the main document, text color known, background not
  determinable, neither visually hidden nor covered (covered when measuring in the visible area),
  up to 15000 CSS pixels of depth — deeper stays `UNTESTED` (memory, time; a canvas is at most
  32767 px high). Time limits: the script waits at most 5 s for the screenshot, the whole sampling
  at most 10 s; after that, or if the screenshot fails, no element gets a background and the audit
  continues. The shared rule judges on it like the formerly local one (median, 40th percentile):
  `FAIL`, passed or review note. The integration tests `test_image_contrast_pixel_sampling` and
  `test_opacity_overlay_contrast_pixel_sampling` run unchanged against their old verdicts again.
  The PDF line "Kontrast X (erforderlich Y)" (contrast X (required Y)) reads the measured values of
  the shared rule (`contrast_ratio`, `required_ratio`, delivered without `:1`). Seen sampled
  (2026-10-05, desktop, without `--dismiss-consent`): berlin.de 14, gov.uk 0, bundesregierung.de
  0, wetter.com 5, n-tv.de 39, spiegel.de 0 (the image credits lie deeper than 15000 px). Audit
  runtime before → after within the noise (−9 s to +1.5 s per page).

- **Unreleased — Contrast from `a11y-rules` (#698, barrierlab#47):** `contrast/text-insufficient`
  (1.4.3 AA), `contrast/text-undetermined` (1.4.3, `UNTESTED`) and `contrast/text-enhanced`
  (1.4.6 AAA, only with `--level aaa`; reports only what passes 1.4.3) replace `color-contrast`.
  Deleted: `wcag::rules::contrast` including the page rule and the pixel comparison with the
  screenshot, `accessibility::styles` (`extract_text_styles`); `Color` now lives in
  `non_text_contrast_css` (remains `wcag::rules::Color`). The old ID keeps resolving via the
  taxonomy (`a11y.contrast.weak`, `axe_id: color-contrast`); 1.4.6 has its own AAA entry
  `a11y.contrast_enhanced.weak` with AAA weight (before, AAA failures counted as 1.4.3 with the
  full deduction). Dark mode and the color-vision-deficiency view count the shared violations
  over a fresh DOMSnapshot; the light/dark comparison pairs elements via the backend ID.
  Intentionally different: text under a fixed or sticky element (cookie banner) is `UNTESTED`
  (#716 case 7); text under `aria-hidden` is measured (#395 excluded it).
  Comparison shared against local (2026-10-05): corpus (`contrast_shadow_and_inline`,
  `gradient_text_contrast`, `low_contrast_text`, `text_fill_color_contrast`) identical, AA
  and AAA; gov.uk identical; bundesregierung.de AA equal, AAA +4 (meta navigation, skipped locally
  as hidden: absolutely positioned list in a 0 px wide `nav` with `overflow: hidden`); wetter.com
  without consent overlay AA equal (15), with overlay the ten navigation items `UNTESTED` instead
  of 2.37:1, AAA additionally the consent dialog (skipped locally as hidden) and teaser captions
  with their own opaque surface (locally review note); n-tv.de AA no violations in either, AAA 25
  local violations under the consent overlay now `UNTESTED`; spiegel.de AA +19 real violations
  (4.04:1, white on #e64415 and vice versa) that the local rule missed (it treated the class name
  `overflow-hidden` as hidden), AAA correspondingly more. Corpus expectations switched to the new
  IDs.
  Live against `main` (2026-10-05, with sampling), accessibility AA / AAA before → after:
  gov.uk 93 → 93 / 70 → 89, bundesregierung.de 49 → 49 / 46 → 49, wetter.com 22 → 22 / 18 → 20,
  n-tv.de 36 → 36 / 23 → 29, spiegel.de 28 → 24 / 20 → 21 (AA violations before → after:
  0 → 0, 0 → 0, 15 → 12, 25 → 23, 3 → 22; the decreases are text under the consent overlay, now
  `UNTESTED`). AAA rises because 1.4.6 now counts with AAA weight. Reference page berlin.de
  45 → 49, just above the band 15–48: the local rule reported 11 violations on desktop / 12 on
  mobile, mostly image credits and teaser titles over photos (`p.image__copyright`, `a.title`),
  measured against an ancestor color (2.56:1 and 1.00:1 respectively). With page-wide sampling
  none is `UNTESTED` any more; 1 / 3 violations remain (including `a.title`, 4.02:1), the rest
  passes over the photos. The band needs to be re-evaluated.

- **Unreleased — Visually hidden and covered text, background over images (part of
  #698, host side of barrierlab#47):** `RenderedCdpDocument` delivers
  `Rendering::visually_hidden` from the `DOMSnapshot` (new style values `clip`, `clip-path`,
  `text-indent`, `position`, `opacity`): `opacity: 0` on the element or an ancestor,
  `clip: rect(≤ 1px …)` on an absolutely positioned box, `clip-path: inset(≥ 50 %)`,
  `text-indent` from 999 px, a box of at most 1 px with `overflow: hidden|clip` (on the element
  or up to twelve levels above; an absolutely positioned descendant escapes a static one, a fixed
  one escapes any box) and a frame entirely to the left of or above the document. Of `Layout`
  the document fills only `obscured`, on elements with their own text in the main document: a
  script pass before the snapshot uses `elementFromPoint` at the center to check whether a
  foreign fixed or sticky element covers the text; only the hits are mapped to backend IDs via
  `describeNode`. The effective background color is `None` if a painting sibling of the element
  or of an ancestor (up to six levels, up to the first opaque surface) overlaps the box and one of
  the two is absolutely or fixed positioned (#716 case 6: wetter.com `h4.newsCarousel__headline`,
  spiegel.de `figcaption > p`).
  Seen filled (DoD 4, 2026-10-05, `visually_hidden` measured / true, `obscured` measured /
  true): gov.uk 444/15, 132/0 (including `button.gem-c-search__submit`); bundesregierung.de
  6706/5634 (almost all SVG sprite paths in a 0 px `<svg>`), 232/0; wetter.com 1897/569
  (hover menus with `opacity: 0`), 577/199 with consent overlay and 0 without; n-tv.de 3832/16,
  1097/79; spiegel.de 6793/135, 1027/36.

- **Unreleased — Static presentation rules from `a11y-rules` 0.19.1 (#699):** The rules of the
  presentation convention that only need the markup now come from the shared rule set
  (barrierlab#22): `display/toggle-missing` (replaces the local rule of the same name, same
  text) and the new `viz/text-missing`, `viz/caption-missing`, `viz/static-missing`,
  `viz/table-missing` (the last one a review note). The new IDs have their own
  taxonomy entries (`a11y.display_caption.missing`, `a11y.display_static.missing`,
  `a11y.display_table.review`; `viz/text-missing` shares `a11y.display_text_layer.missing`),
  explanations and score ranges; like `display/*` they count as convention rules, not as a
  legal note. What only the running page shows stays in `display_modes`:
  `display/init-missing` (observer before navigation), `display/text-hidden` (including computed
  visibility and the hidden figure, #725), `display/text-media-visible` and
  `display/text-not-visible`. These last two IDs also exist in `a11y-rules`, but only as the
  static part; auditmysite does not adopt them, so that an ID never has two sources.
  Intentionally different: `display/text-not-visible` in text mode now only reports a text layer
  that has content but does not appear; if it is missing or empty, that is `viz/text-missing` and
  applies in every mode, not only in text mode. New corpus fixture `display_modes_viz_static`, the
  expectation in `display_modes_violations` moves to `viz/text-missing`. Checked: unit tests,
  detection corpus; cross-check on geographia.eu (reference implementation, e.g. `climate-monitor`
  with 26 figures): old and new versions report nothing there, no false positive on conforming
  pages.

- **Unreleased — Rendering layer from the DOMSnapshot, three presentation rules from `a11y-rules`
  (part of #698, host side of barrierlab#21 second part; `a11y-*` 0.20.0):**
  `RenderedCdpDocument` now fills `ComputedStyle` completely from the same
  `DOMSnapshot.captureSnapshot`: text and effective background color (blended over the ancestors
  in the flat tree, white at the bottom; `None` for background image, gradient or unreadable
  color), font size, weight, style, family, `list-style-type`, `text-decoration-line`,
  `border-bottom-style`; transparent text fill (`background-clip: text`, #640) yields no color,
  `display: contents` (such as `<slot>`) takes color and font from the rendered text.
  `bounds()` delivers the frame of the layout object, `scroll_overflow_px()` the scroll overhang
  from `includeDOMRects` (`scrollRects`/`clientRects`, only on axes with `overflow: auto|scroll`).
  The document does not deliver a `Layout`; the heuristics on it stay unrun. Seen filled
  (DoD 4, 2026-10-05, elements with layout object / all): gov.uk 444/617, bundesregierung.de
  6695/7591, wetter.com 1905/2612, n-tv.de 3636/4789, spiegel.de 6673/8891 — every new field
  filled on every page. Adopted: `redundant-role` (`<ul>`/`<ol>`) → `lists/role-redundant`
  (4.1.2 A), `link-in-text-block` → `color/link-indistinct` (1.4.1 A; in addition, font family,
  bottom edge and background count as a distinction), `scrollable-region-focusable` →
  `keyboard/scrollable-region-not-focusable` (2.1.1 A). Deleted: `redundant_role`,
  `use_of_color`, `scrollable_region`; the old IDs keep resolving in taxonomy and explanations.
  The color-vision-deficiency view counts `color/link-indistinct` once instead of per mode
  (the emulation does not change computed styles). Corpus fulfilled unchanged
  (`redundant_role_list_style`, `link_in_text_block_context`, `scrollable_region_focusable`,
  `misc_content_checks`). The three rules run only in the top-level document like the ones they
  replace.
  **Contrast stays local:** `contrast/text-*` matches `contrast` on the corpus (shadow DOM,
  4.46:1 unrounded, float container, gradient text undetermined), but live it lacks AAA
  thresholds, visually hidden text is measured (gov.uk search button, `text-indent: -5000px`),
  and text over positioned images (#716 case 6) becomes `FAIL` instead of `UNTESTED` (wetter.com
  `newsCarousel__headline`, spiegel.de image credits). Live against `main` (2026-10-05),
  accessibility AA / AAA before → after: gov.uk 93 → 93 / 70 → 70,
  bundesregierung.de 49 → 49 / 46 → 46, wetter.com 22 → 22 / 18 → 18, n-tv.de 36 → 36 / 22 → 22,
  spiegel.de 26 → 26 / 20 → 20; the three rules ran without findings before and after, the
  other deviations (`color-contrast` ±1, `target-size` 8 → 10 on AAA) are page changes
  between the runs in unchanged rules.

- **Unreleased — Reduced motion from `a11y-rules` 0.19.1 (part of #698):** Since 0.19.1
  `a11y-rules` resolves the `animation` shorthand according to CSS Animations; Chrome's
  serialization (`2s linear 0s infinite normal none running spin`) yields the name `spin` again
  instead of `none` (bug from this PR, fix in barrierlab). With that, `prefers-reduced-motion` →
  `motion/reduced-motion-ignored` (2.3.3 AAA, note instead of violation, one message per page)
  runs over the stylesheets; `reduced_motion` is deleted, the old ID keeps resolving in the
  taxonomy. 2.3.3 counts as a note criterion (AAA). Corpus: `media_and_motion`,
  `reduced_motion_transform` and `target_size_animation` report the note,
  `reduced_motion_override` and `reduced_motion_colour_only` do not. Intentionally different: The
  superseded page rule ran from `--level aa` despite being AAA (`min_level: AA`); the shared note
  appears, like the other AAA IDs, only with `--level aaa`. Live against `main` (2026-10-04) on
  AAA the same hits (bundesregierung.de and n-tv.de 1 each, gov.uk, wetter.com, spiegel.de 0); on
  AA the previous 2.3.3 finding is dropped, accessibility AA / AAA before → after: gov.uk
  92 → 93 / 67 → 70, bundesregierung.de 49 → 49 / 35 → 46, wetter.com 21 → 22 / 17 → 18, n-tv.de
  22 → 31 / 18 → 22, spiegel.de 25 → 28 / 18 → 20.

- **Unreleased — Rules over stylesheets from `a11y-rules` (host side of #698, barrierlab#21
  first part):** auditmysite reads the stylesheets of the page (`document.styleSheets`, per sheet
  the `cssText` of all rules; foreign sheets without readable `cssRules` are missing as before),
  parses them with `stylesheet-parse` 0.1.0 (new direct dependency) and passes them together
  with the document of the shared rules to `a11y_rules::run_stylesheets_in`
  (`accessibility::fetch_stylesheets`, `wcag::shared::run_shared_rules_with_stylesheets`).
  If reading fails, the rules stay `NotRun::CapabilityMissing`. Frames get no sheets; the rules
  judge the page like the ones they replace, which ran only in the top-level document. Adopted:
  `focus-visible-outline-none` → `focus/outline-removed` (2.4.7 AA, high; an outline set again in
  `:focus` counts as a replacement, sueddeutsche.de), `css-orientation-lock` (stylesheet part) →
  `orientation/content-hidden` (1.3.4 AA, note, only selectors with a hit on the page, without
  pseudo-elements), `visual-presentation` (justified text, line spacing) → `text/justified` and
  `text/line-height-tight` (1.4.8 AAA, notes, measured on the `<p>` of the page). Deleted:
  `focus_visible_css`; from `orientation` the stylesheet search (the computed `transform: rotate`
  on `body`/`html` stays), from `visual_presentation` the stylesheet search (the `UNTESTED` note
  on color choice and column width stays). 1.4.8 counts as a note criterion (AAA). **Not adopted
  because of a bug in `a11y-rules` 0.19.0:** `motion/reduced-motion-ignored` -- Chrome serializes
  the `animation` shorthand completely (`2s linear 0s infinite normal none running spin`), and
  the rule takes `none` from `animation-fill-mode` as the animation name, so every animation via
  the shorthand drops out (corpus `media_and_motion`, `target_size_animation`). `reduced_motion`
  therefore stays here for now (fixed with 0.19.1, see above). Deviations in the changelog of
  `a11y-rules` 0.19.0. Corpus: `media_and_motion` reports `focus/outline-removed` and
  `orientation/content-hidden`, `text_and_layout` reports `text/justified` (the tight line height
  there is on a `<div>`, not on a `<p>`).
  Live against `main` (2026-10-04), accessibility AA / AAA before → after, B4–B7 and
  stylesheets together: gov.uk 92 → 93 / 67 → 70, bundesregierung.de 49 → 49 / 35 → 43, wetter.com
  21 → 22 / 17 → 18, n-tv.de 22 → 29 / 18 → 21, spiegel.de 25 → 28 / 18 → 20. Dropped are the
  false positives `css-orientation-lock` on bundesregierung.de and n-tv.de (breakpoint marker
  `body:before`, close button) and `visual-presentation` on spiegel.de (selector prefix);
  new on AAA `text/line-height-tight` on gov.uk and bundesregierung.de.

- **Unreleased — `<area>` and self-starting audio from `a11y-rules` (#696, addendum):** With
  `a11y-rules` 0.19.0 the view sees `<area>` and `<audio>` without `controls` also with computed
  styles (UA `display: none`; reported from this PR, fix in barrierlab). With that, `area-alt` →
  `images/area-alt-missing` (1.1.1 A; `aria-label`/`aria-labelledby` count) and
  `background-audio` → `media/audio-autoplay` run via the shared rule set; `image_input_rules` and
  `background_audio` are deleted. `media/audio-autoplay` is a review note under 1.4.2 (A)
  instead of a violation under 1.4.7 (AAA) and thus already runs on AA; taxonomy
  `a11y.audio_control.missing`, 1.4.2 counts as a note criterion (it was already in the manual
  list). Both also run in frames. Corpus: `misc_content_checks` and `media_and_visual`
  report under the new IDs.

- **Unreleased — Table, document, language and role rules from `a11y-rules` instead of own ones
  (#697, B7):** Twelve IDs run via the shared rule set: `th-has-data-cells` →
  `tables/header-without-data` (including `tables/data-undetermined`, `UNTESTED`, for a row supply
  not yet rendered, #654), `td-headers-attr` → `tables/headers-attr-invalid`,
  `html-xml-lang-mismatch` → `document/lang-mismatch`, `language-of-parts` →
  `language/part-unmarked` (new `language/part-undetermined` on pages that are neither German nor
  English), `abbreviations` → `language/abbreviation-unexpanded` (note instead of violation),
  `meta-refresh` → `timing/meta-refresh` (0 s is not a deadline), `heading-order` from
  `section_headings` → `headings/section-without-heading` (note, only articles and named
  sections), `redundant-role` → `aria/role-redundant`, `title-only-description` →
  `names/title-only` (note; text fields are still reported by `forms/title-only-label`),
  `content-on-hover-focus` → `patterns/tooltip-unreferenced`. Deleted: `table_extended`,
  `language_extended`, `language_of_parts`, `abbreviations`, `content_on_hover`,
  `info_relationships`, `section_headings`, the tree rules `th-has-data-cells` and `heading-order`.
  **Dropped without replacement:** `presentation-semantic-children` -- according to WAI-ARIA 1.2,
  `role="presentation"`/`"none"` removes the semantics only from the element itself, not from the
  descendants; the rule would have reported the APG menubar pattern (`<li role="none">` around
  menu links) (barrierlab: 187 false positives on 48 pages), the table case is
  `tables/presentational-with-headers`. From `section_headings`, the outline gaps
  (`headings/skip-level`) and "more than 10 paragraphs, fewer than 3 headings" (no evidence) are
  dropped. **Staying in the host:** from `redundant_role` only `<ul>`/`<ol>` with `role="list"` --
  whether the role is superfluous depends on the computed `list-style-type` (#644), which
  `a11y-dom` does not yet deliver (barrierlab#21); the corpus case `ol#numbered-list` still
  reports. From `timing_adjustable` the page-wide `UNTESTED` notes for script deadlines (2.2.1,
  now as page rule `2.2.1/timing-adjustable`) and `timeouts` (2.2.6), which barrierlab assigns to
  the manual checklist. The presentation rules from barrierlab#22 (`viz/*`, static halves of
  `display/*`) are not adopted: `display/text-hidden`, `display/init-missing` and
  `display/toggle-missing` carry the same IDs in `display_modes`, but there also check computed
  visibility and the timing of `data-display`; splitting the same ID across two sources is a step
  of its own, `display_modes` stays unchanged. Taxonomy: new `a11y.language_of_parts.unmarked`
  (3.1.2 AA; `language-of-parts` had no entry); 3.1.2, 2.4.10 and 3.1.4 count as pure
  note criteria, so 3.1.2 is in the list of criteria to be checked manually (21 → 22,
  `docs/PARITY_CONTRACT.jsonc`, README). Table, role, name and tooltip rules as well as
  abbreviations also run in frames. Checked: detection corpus before/after (renames,
  notes instead of violations as in the barrierlab changelog, `language/part-undetermined` on
  `invalid_lang_code` and `skip_link_language`, new section notes on five cases).
  Check B4–B7 in total against `main` (detection corpus 76 cases on AAA: eleven accessibility
  scores rise by 1–9 points, none falls; live pages on AA, accessibility before →
  after: gov.uk 92 → 93, bundesregierung.de 49 → 49, wetter.com 21 → 22, n-tv.de 22 → 25,
  spiegel.de 25 → 28). New on real pages: `names/title-only` (spiegel.de 132 notes instead of
  5 findings), `frames/name-missing` (n-tv.de 6 instead of 2, including the three sports-data
  frames with `title=""`), `aria/role-redundant` (wetter.com 9 instead of 20 capped, hidden menus
  no longer count); `navigation/location-missing` and the other AAA notes do not run on AA.
  Reference run and score calibration stay in their bands.

- **Unreleased — Image and media rules from `a11y-rules` instead of own ones (#696, B6):** The
  following now run via the shared rule set: `input-image-alt` → `images/input-alt-missing`,
  `object-alt` → `objects/alt-missing`, `server-side-image-map` → `images/server-side-map` (all
  1.1.1 A) and `frame-title` → `frames/name-missing` (2.4.1 A, high; `a11y-rules` additionally
  lists 4.1.2). Deleted: `server_side_image_map`, from `image_input_rules` the checks for
  `<input type="image">` and `<object>`, from `media_rules` the frame names and the tree rule
  `video-caption` (unnamed `application`/`img`, named decorative element -- barrierlab did not
  adopt it for lack of evidence or because of `svg/name-missing`, `images/alt-missing` and
  `aria/attribute-prohibited`). Intentionally different: `aria-labelledby` counts as an
  alternative, `<embed>` is no longer checked, only `<iframe>` (no `<frame>`), no upper limit per
  page; the rendered size ≤ 1 px is unknown to `frames/name-missing` without geometry (only
  width/height as attribute). `frames/name-missing` has its own explanation text for the first
  time (`frame-title` fell back to the 2.4.1 text "Fehlende Sprungnavigation" (missing skip
  navigation)). Staying in the host: the caption check `video-caption` (loads the `<track>` file
  over the network), `frame-tested` (foreign frames are only known to the browser) and
  `media-alt` (1.2.8) -- barrierlab refers to `manual/media-alternatives` for it, the manual
  checklist is not yet carried by auditmysite. **Not adopted because of a bug in `a11y-rules`
  0.18.0:** `images/area-alt-missing` and `media/audio-autoplay` never report with computed
  styles, because the view removes everything with `display: none` and the UA stylesheet gives
  that to every `<area>` and every `<audio>` without `controls` (corpus `misc_content_checks`,
  `media_and_visual`). `area-alt` (`image_input_rules`) and `background-audio` therefore stay here
  for now. Image and frame rules now also run in frames. Coverage: 2.4.8
  (`navigation/location-missing`, only a note since B5) no longer counts as an automatically
  checked AAA criterion. Checked: detection corpus before/after (only the renamed IDs, selector
  now `tag#id`).

- **Unreleased — Link and pointer rules from `a11y-rules` instead of own ones (#695, B5):** With
  `a11y-rules` 0.18.0 three IDs run via the shared rule set: `click-events-have-key-events`
  → `keyboard/click-handler-not-focusable` (2.1.1 A, high), `link-as-button` →
  `links/used-as-button` (4.1.2 A, low), `location` → `navigation/location-missing` (2.4.8 AAA,
  low). Deleted: `click_handlers`, `fake_navigation_link`, `location` including their
  `PAGE_RULES` entries; the old IDs resolve via the new ones in the taxonomy and the explanation
  texts. Intentional differences (changelog of `a11y-rules`): no upper limit per
  page any more (previously 10 and 20 respectively), `<a onclick>` without `href` is a click
  handler instead of a fake link, `navigation/location-missing` is a review note instead of a
  violation and also sees navigation in the shadow DOM. Of `pointer_cancellation` (2.5.2) the
  static part (`onmousedown`/`ontouchstart` on controls) is deleted -- barrierlab did not adopt
  it for lack of evidence --, the page-wide `UNTESTED` note stays in the host: The manual
  checklist (`manual/*`, barrierlab#39) has no item for 2.5.2, and auditmysite does not carry it
  yet. New: Shared IDs above the checked level (`--level`) are removed and recorded as
  `NotRun::Disabled` (`above_wcag_level`) (`wcag::shared::retain_up_to_level`), just as the
  superseded AAA rules ran only with `--level aaa`. Click handlers and fake links now also run in
  frames. Checked: detection corpus before/after (only the renamed IDs, `location` as a note,
  additionally `landmark_main_in_shadow_root`), exclusion integration tests with the uncapped
  counts (12 and 22 excluded hits respectively).

- **Unreleased — Landmark, keyboard and structure rules from `a11y-rules` instead of own ones
  (#694, B4):** The landmark checks, keyboard reachability, the page without headings and two
  pattern checks run via the shared rule set (`SHARED_RULES` in `src/wcag/shared.rs`). Deleted:
  `region`, `landmark_granular` (with `is_landmark`) and `bypass_blocks`; from `keyboard` the two
  2.1.1 checks; from `patterns/` the findings `dialog-no-focusable`, `accordion-no-controls`,
  `accordion-trigger-not-button` and `aria-expanded-required` -- pattern detection and journeys
  stay, `PatternAnalysis.violations` is now always empty. `keyboard-trap` (2.1.2) stays in the
  host: the note per modal dialog and the page-wide `UNTESTED` note need real keyboard
  operation. IDs: `landmark-unique` → `landmarks/not-unique`; `landmark-banner-is-top-level`,
  `landmark-contentinfo-is-top-level` and `landmark-main-is-top-level` →
  `landmarks/not-top-level`; `landmark-no-duplicate-banner` and
  `landmark-no-duplicate-contentinfo` → `landmarks/banner-duplicate` and
  `landmarks/contentinfo-duplicate`; `region` → `landmarks/content-outside` (all 1.3.1 A);
  `bypass` ("No headings found") → `headings/none` (2.4.1 A, including #709); `focusable-no-role`
  and `keyboard` ("appears not keyboard-focusable") → `keyboard/focusable-no-role` and
  `keyboard/interactive-not-focusable` (2.1.1 A); `dialog-no-focusable` →
  `dialog/focusable-missing` (2.4.3 A); `accordion-no-controls` →
  `patterns/accordion-controls-missing` (4.1.2 A). Dropped without replacement, as justified in
  barrierlab#17: `accordion-trigger-not-button` (on roles without `aria-expanded` this is reported
  by `aria/attribute-not-allowed`, on `link`, `tab`, `treeitem` WAI-ARIA allows the state, missing
  focus is `keyboard/interactive-not-focusable`) and `aria-expanded-required` (guessed a
  disclosure menu from the word "menu"/"Menü" in the name). Intentional differences (details in
  the changelog of `a11y-rules` and at the entries in `SHARED_RULES`): `landmarks/not-unique` and
  `patterns/accordion-controls-missing` are review notes instead of violations; duplicate
  banner/contentinfo point to the second landmark instead of the first; `landmarks/content-outside`
  reports once per block instead of per text node; `keyboard/focusable-no-role` counts only the
  tab order and no named scrollable region; `keyboard/interactive-not-focusable` excludes
  disabled fields, native `<option>`, `aria-activedescendant` and inert content;
  `dialog/focusable-missing` searches in all descendants and does not check a closed `<dialog>`;
  `headings/none` also counts `role="heading"`. The landmark role comes from the markup instead of
  Chrome's role (#639, #727). Exclusions (#726): The shared counting rules
  (`landmarks/not-unique`, `landmarks/main-duplicate`, `landmarks/*-duplicate`) read, with
  `--exclude-selector` active, the document without the excluded subtrees
  (`ExclusionScope::without_excluded_dom`, `wcag::shared::adopt_page_counts`); the AX variant
  `without_excluded` and the parameter `counted` of `check_all_excluding` are dropped,
  `exclusion.excluded_landmarks` counts as before. In frames, `keyboard/focusable-no-role` and
  `keyboard/interactive-not-focusable` (previously the tree rule `keyboard`) now run, as do the
  new `dialog/focusable-missing` and `patterns/accordion-controls-missing`; the landmark rules and
  `headings/none` stay with the main document. Taxonomy: the IDs point to the entries of the
  superseded rules, whose legacy IDs keep resolving (`headings/none` to that of `bypass`); new
  `a11y.landmark_nested.invalid` and `a11y.accordion_controls.missing` including explanation
  texts. With `a11y-rules` 0.17.0. Checked with unit tests, detection corpus, score calibration,
  reference run and the integration tests on frames, exclusions and landmarks (Chrome) as well as
  a before/after run on 14 live pages; reference bands gov.uk (75–95) and bundesregierung.de
  (35–76) re-evaluated (+1 each: content outside landmarks per block instead of per piece of
  text, dialog focus target also in deeper descendants).

- **Unreleased — False positives from upscale.casoon.dev (#739, #740):** The dark-mode module
  reported a dark-only page (`html.dark` set permanently, `color-scheme: dark`, no light layout)
  as `no_dark_mode_support` with score 50. New is the signal "dark in default rendering":
  effective page background (body, else html, else the canvas according to `color-scheme`) dark and
  body text light; the colors are read via a canvas pixel so that `oklch()` from Tailwind v4 also
  counts. Such a page counts as supported (`detection_methods`: "Dark by default (no light
  view)"), without `no_dark_mode_support` and without the class hint (#739). The mobile module took
  as the smallest font also elements without readable text; KaTeX's `span.vlist-s` (a zero-width
  space at 1 px) produced "Smallest font size is 1.0px". For the smallest font only elements with
  their own text of more than whitespace and zero-width characters now count (#740). Live:
  upscale.casoon.dev dark mode 50 → 85, Pythagoras lesson smallest font 1 px → 11 px (real text);
  gov.uk (light) still without dark-mode support, casoon.de unchanged. #738 (`<legend>` after
  whitespace) lies in html-conform: casoon/barrierlab#41.

- **Unreleased — Form rules from `a11y-rules` instead of own ones (#693, B3):** The form,
  context-change and captcha checks run via the shared rule set (`SHARED_RULES` in
  `src/wcag/shared.rs`), the superseded own rules are deleted: `form_rules`,
  `error_identification`, `input_purpose`, `identify_purpose`, `redundant_entry`,
  `label_title_only`, `on_focus`, `instructions` and `labels` (`check_form_control`, the last
  remainder of the file). What the running page needs stays in the host: in `on_input` the lookup
  of a called function via `window` (violation if only it changes the context) and the name
  guess ("Language") without a submit button, both now under the same ID as the shared rule
  (`context/on-input`), so that the violation supersedes its review note on the same element;
  in `accessible_authentication` the paste test (`accessible-auth-paste-blocked`). IDs:
  `label` "no accessible label" and `control-missing-label` on form fields →
  `forms/label-missing` (3.3.2 A); `label` "Placeholder used as only label" →
  `forms/placeholder-as-label`; `label` "Form group has no legend or label" →
  `forms/group-name-missing`; `label` "Required field not clearly indicated" and "may not indicate
  required status" → `forms/required-unmarked`; `label` "may require format instructions" →
  `forms/instructions-missing` (all 3.3.2 A); `autocomplete-valid` → `forms/autocomplete-invalid`
  (invalid value) and `forms/purpose-missing` (missing value, 1.3.5 AA), plus
  `identify-purpose` (1.3.6 AAA) → `forms/purpose-missing`; `input-error-message` and
  `aria-invalid-without-describedby` → `forms/error-unidentified` (3.3.1 A); `form-field-group` →
  `forms/group-missing` and `label-title-only` → `forms/title-only-label` (1.3.1 A);
  `form-no-submit` → `forms/no-submit` (3.2.2 A, including #728); `redundant-entry` →
  `forms/redundant-entry` (3.3.7 A); `input-no-context-change` (handler in the markup) →
  `context/on-input` (3.2.2 A); `focus-no-context-change` → `context/on-focus` and
  `context/autofocus` (3.2.1 A); `accessible-auth-captcha` → `auth/captcha` (3.3.8 AA). Intentional
  differences (details in the changelog of `a11y-rules` and at the entries in
  `SHARED_RULES`): a missing label, a missing error description, an unmarked
  required field and a missing input purpose appear once instead of twice in the report;
  `forms/purpose-missing`, `forms/required-unmarked`, `forms/title-only-label`, `context/on-focus`
  and `context/autofocus` are review notes instead of violations; `forms/autocomplete-invalid`
  checks the whole autofill grammar and is low instead of medium; `context/on-input` also checks
  radio buttons; `forms/no-submit` also counts fields and buttons assigned via the `form`
  attribute; the page-wide `UNTESTED` note on 1.3.6 is dropped. The form rules
  now also run in frames, including those previously checked only in the main document
  (`autocomplete`, submit, repetition, context change, captcha). Taxonomy: all IDs
  point to the entries of the superseded rules, whose legacy IDs keep resolving; new
  explanation texts for `forms/no-submit` and `auth/captcha`. With `a11y-rules` 0.16.0, which
  reports an unnamed ARIA form field (combobox, listbox, textbox, …) like the native field without
  a label as critical. Checked with unit tests, detection corpus, score calibration and reference
  run (Chrome) as well as a before/after run on 14 live pages.

- **Unreleased — SEO headings from the shared rules, no second HTML validator
  (#724):** The SEO view checked missing and multiple H1s, skipped levels and empty
  headings a second time. These findings now come from `headings/*` (a11y-rules) of the
  accessibility check, after exclusions and once per finding across both viewports;
  `SeoModule::derive` places them ahead of the SEO-specific rule for long headings and recomputes
  score, content profile and SERP view. The SEO heading list (`h1_count`, `h1_text`,
  lengths) now also reads open shadow roots so that it matches the view of the shared rules
  (dm.de: the h1 of the Usercentrics banner in the shadow DOM counts; SEO score 75 → 67). The
  local html5ever check (`validate_html_locally`) is removed: it counted parse errors of the
  same serialized live DOM that `html_conform` checks more thoroughly. The validator status
  in the SEO part is now called `delegated` and points to html-conform (`--full`,
  `--html-conform`); `ParseErrors` remains only so that stored reports still load. Old-versus-new
  cross-check with `--seo`: casoon.de, gov.uk, berlin.de and mit.edu with identical
  heading findings and identical score.

- **Unreleased — CSP via `csp-parse` (#723):** `src/security` used to split the
  Content-Security-Policy itself (`split(';')`); it now reads it with `csp-parse` 0.1.0 from
  barrierlab (CSP3: policy list, directives, source lists with keywords, nonces, hashes,
  host wildcards). The assessment (which weakness weighs how heavily) stays here. Intended
  differences: multiple `Content-Security-Policy` headers are all read and joined as a policy list
  instead of only the first; in a policy list a weakness counts only if every policy has
  it (the browser enforces all of them); a duplicate directive keeps its first value instead of
  the last; a nonce and a hash count only with a valid base64 value; the wildcard check across
  all directives reads only source lists (not `report-uri`, `sandbox` etc.). New unit tests
  for the three cases; security corpus green; the eight reference sites yield the same CSP findings
  with the old and the new parser.


- **1.7.3, 2026-10-01:** ARIA and name rules now come from `a11y-rules` instead of our own
  implementations (#691 B1, #692 B2, a11y-rules 0.15.0), with new identifiers following the scheme
  `aria/*`, `names/*`, `dialog/*`, `popover/*`, `inert/*`; an unnamed dialog now appears only
  once in the report. Cross-site iframes are checked because the audit browser starts without
  site isolation (#720). SPA portals are captured only after the first rendered content
  (#718). False alarms fixed: text layer in hidden panels (#725), unnamed
  `form`/`region` as a landmark (#727), landmark count despite exclusion (#726, new JSON field
  `exclusion.excluded_landmarks`), `form-no-submit` for pure toggle forms (#728).
  Reference run (`reference_sites_test`) and detection corpus green. Details in the entries
  below.

- **Name rules from `a11y-rules` instead of our own (#692, B2):** The checks for
  missing and insufficient accessible names run through the shared set
  (`SHARED_RULES` in `src/wcag/shared.rs`); the superseded own rules are deleted:
  `aria_naming_rules`, `dialog_rules`, `summary_name`, `status_messages`, `label_in_name` and from
  `accessible_name` everything except `description-duplicates-name` (#713) -- that one stays because
  `a11y_dom` provides no accessible description; its switch in `wcag::engine` is now called that
  instead of `aria-label`. Also removed: `aria-dialog-name` in `patterns::modal_dialog` and the
  dialog part of the missing name in `modern_attributes` (menu and popover stay there).
  Identifiers: `aria-command-name` (without button and `a[href]`), `aria-input-field-name`,
  `aria-meter-name`, `aria-progressbar-name`, `aria-toggle-field-name`, `aria-treeitem-name` and
  `aria-label` "kein Name" ("no name") → `names/required-missing`; `aria-label` "Icon Only" →
  `names/symbol-only`; `aria-dialog-name` and `dialog-name` → `dialog/name-missing`; `dialog-name`
  "Dialog Modal" → `dialog/modal-unmarked`; `summary-name` → `summary/name-missing`;
  `aria-live-region-role` → `status/live-overridden` (4.1.3 AA); `label-content-name-mismatch` →
  `label-in-name/mismatch` (2.5.3 A); all others under 4.1.2 A. Severity as before. Intended
  differences (details in the changelog of `a11y-rules` and at the entries in
  `SHARED_RULES`): an unnamed dialog appears once instead of up to three times in the report, an
  unnamed widget once instead of twice; unnamed `menu`, `tab` and native `<option>` are no longer a
  finding (ARIA 1.2 requires no name there); `meter` and `progressbar` are under 4.1.2
  instead of 1.1.1; non-focusable unnamed widgets are a violation as before in
  `aria_naming_rules`, the additional warning from `accessible_name` is dropped; the purely
  symbolic name, the missing `aria-modal` and a changed but not disabled
  urgency of a live region (`alert` with `polite`) are review hints instead of violations;
  `dialog/modal-unmarked` checks only `role="dialog"`, not a native `<dialog>`; `summary/name-missing`
  reports only the `<summary>`, not additionally the `<details>`; an empty `aria-labelledby` and
  `aria-describedby` on a named element are no longer a finding; `label-in-name/mismatch`
  checks all roles with names from content and names from `aria-labelledby`, not just buttons
  with `aria-label`, without the cap of 50 -- and now also runs in frames. New
  taxonomy entry `a11y.dialog_modal.missing` with explanation (previously the missing
  `aria-modal` shared identifier and title with the missing dialog name); the other identifiers point to
  the entries of the superseded rules, whose legacy identifiers continue to resolve. Verified with
  unit tests, detection corpus and score calibration (Chrome). With `a11y-rules` 0.15.0. The #527 filter (a violation displaces a hint) now compares the backend node ID where both findings carry one: shared findings have short selectors (`button`), and a violation on one button swallowed the hint on another. Reference band bundesregierung.de re-rated (35–72): three label-in-name false alarms on text hidden via CSS are gone, the cookie dialog without aria-modal is a hint.


- **Frames in other processes (#720):** After #715, iframes that Chrome shows in their own renderer because of
  site isolation remained unchecked (`skipped: cross_origin`) — for example the
  web chat on gov.cy (`digital-assistant.gov.cy`, `gov.cy` is a public suffix), in which axe reports an
  unnamed `role="dialog"`. chromiumoxide 0.8 does create a session internally for such frames,
  but offers no commands for them. Instead of a custom CDP client, the
  audit browser now starts without site isolation (`--disable-site-isolation-trials`,
  `IsolateOrigins`/`site-per-process` off): cross-site frames run in the page's process, and the
  existing frame pass reaches them including enrichment, selectors and exclusions. The
  profile is fresh and holds no user data; if a future Chrome ignores the flags, the
  frame is visibly listed as `cross_origin` in the report again. The Chrome test now expects the cross-site frame
  to be checked with a `dialog-name` finding; gov.cy live: frame checked, dialog found.

- **SPA portals: capture before the first render (#718):** After loading, eesti.ee shows an
  empty splash screen while the Angular app fetches its first view — around 400 ms without a
  single DOM mutation. `wait_for_page_stability` counted 200 ms of quiet as stable, the AX tree was read
  before the first heading and `bypass` reported "No headings found". Quiet without content
  (no `main`, no `h1`-`h3`, no visible text) no longer counts as stable; the
  provenance records the wait as `reason`, and a page without content until the budget ends ends as
  `budget_exhausted`. New Chrome test with a splash fixture (fails without the fix);
  detection corpus green; eesti.ee and latvija.gov.lv without the false finding.

- **ARIA rules from `a11y-rules` instead of our own (#691, B1):** The ARIA checks
  run through the shared set (`SHARED_RULES` in `src/wcag/shared.rs`, all under 4.1.2
  A); the superseded own rules are deleted: `aria_allowed_attr`, `aria_prohibited_attr`,
  `aria_valid_attr_value`, `aria_required_parent`, `parsing`, `aria_roles`, `widget_rules`, the
  popover and dialog parts of `modern_attributes` and the tab message from `patterns::tab_list`.
  Identifiers: `aria-roles` → `aria/role-invalid` (new in addition: `aria/role-abstract`) and, for the
  required parts, `aria/required-children-missing`; `aria-attr-name-invalid` →
  `aria/attribute-unknown`; `aria-valid-attr-value` → `aria/attribute-value-invalid` (the
  IDREF half is `aria/reference-missing`); `duplicate-id-aria` → `aria/owns-conflict`;
  `aria-allowed-attr` → `aria/attribute-not-allowed`; `aria-prohibited-attr` →
  `aria/attribute-prohibited`; `aria-required-parent` → `aria/required-parent-missing`;
  `aria-tab-selected-state` and `tab-no-aria-selected` → `aria/tab-selected-missing`;
  `aria-tablist-tabpanel` → `aria/tabpanel-missing`; `aria-combobox-options` →
  `aria/combobox-popup-missing`; from `modern-attribute-misuse` → `popover/target-missing`,
  `popover/target-invalid`, `inert/dialog-inert`. Intended differences (details in the
  changelog of `a11y-rules` and at the entries in `SHARED_RULES`): context and required parts
  are checked on the DOM -- `ul[role=tablist] > li > a[role=tab]` (#715) now reports, like axe,
  missing required parts on the tablist and missing context on the tab; allowed and prohibited
  attributes also judge by the implicit role (`<div aria-expanded>` is a finding);
  required parts only for containers with an explicit role, the native table
  with only `<caption>` is no longer a finding; missing selected tab and missing tab panel are review hints,
  the tab hint appears once on the tablist instead of per tab; unknown values of `aria-current`
  and `aria-invalid` are not a finding; the visible `role="menu"` with `inert` no longer is one.
  `modern_attributes` keeps the missing name of open dialogs, menus and popovers, the open
  popover with `inert` and the focus inside an inert subtree. Chrome's own role names
  no longer reach the shared rules (`CdpDocument`): `image` becomes `img`, the capitalized ones
  (`LayoutTable*`, `DescriptionList`, `LabelText`, ...) become "no role" --
  otherwise `role="img"` was left without an attribute judgment and a layout table aborted every context check
  above it. New taxonomy entry `a11y.tab_selected.missing`; the other identifiers point
  to the entries of the superseded rules, whose legacy identifiers continue to resolve. Explanations
  and detection corpus are updated accordingly. Verified with unit tests and the detection corpus (Chrome),
  including the #715 case in the iframe. Reference bands bundesregierung.de (35–65), berlin.de (15–48) and dm.de (15–48) re-rated: collapsed `aria-controls` without a target is no longer a finding (dm.de), tab selection only once per tablist (ARIA SHOULD), Swiper lists as `lists/invalid-structure` (Medium) instead of `aria-roles` (High) — a11y-rules 0.14.1 catches `li[role=group]` for this again.

- **False alarms from barrierlab.eu and geographia.eu (#725-#728):**
  `display/text-hidden` reported the text layer of visualizations in inactive section panels
  (`hidden` on the `section`) as withheld; the whole figure including the graphic was hidden, though. A figure that is not rendered
  now also does not count as a violation with `hidden`/`aria-hidden`/`inert` on the figure or an ancestor
  (#725; the sr-only assumption in the issue did not hold, `checkVisibility` already let the
  clip pattern through). `landmark-unique` and the top-level checks count `form` and
  `region` as a landmark only with an accessible name (#727). The counting landmark rules
  (`landmark-unique`, duplicate banner/contentinfo) read a tree without excluded
  subtrees; the JSON names the removed landmarks as `exclusion.excluded_landmarks`,
  and the PDF note does so too (#726). `form-no-submit` now reports a form without `action`, without a text field
  and without a submit element (only script can submit it, typically a toggle for content) only
  as a low review hint instead of a 3.2.2 violation (#728). New corpus cases
  `display_modes_hidden_panel`, `form_toggles_without_submit`; detection corpus green;
  cross-check against both live sites without the false findings.

- **1.7.2, 2026-09-30:** False-alarm corrections and gaps from the axe cross-check of the
  barrierlab.eu survey (21 EU portals) and from geographia.eu: target size only for visible
  targets in the final layout (#705, #706), exclusions also in the screen reader layer and in the cache
  (#703, #708), display rules without a legal verdict (#704), `main` and skip link behind an open
  dialog or cookie banner (#709, a11y-* 0.13.5), link color only in body text (#710), motion only
  with evidence (#712), title-only and duplicate description as best practice (#711, #713). New:
  contrast in shadow DOM and across float containers (#716), rule `scrollable-region-focusable`
  (#717), element-level rules in iframes including frame coverage in the JSON (#715, partial; frames
  in other processes follow in #720), lower weight for `zoom/viewport-missing` (#702).
  Reference run (`reference_sites_test`) and detection corpus green. Details in the entries
  below.

- **Rules run in iframes (#715):** A `role="dialog"` without a name in an
  iframe (gov.cy) went undetected because `getFullAXTree` and the shared DOM rules saw only the
  main document; axe checks every frame. New is a frame pass (`audit::frames`):
  for every iframe rendered in the page's process (also nested) it fetches the AX tree via
  `frameId` and builds its own document from the `content_document` of the same `DOM.getDocument` call,
  then the **element-level** tree and shared rules run there (names of
  dialogs, images, links, buttons, form fields; roles including required parents/children;
  list and table structure; referenced duplicate IDs; `aria-hidden` focus etc.) — via an
  explicit allowlist. Page-level rules (landmarks, `h1`, title, viewport, skip link,
  `bypass`, `region`) remain reserved for the main document, as in axe. Findings carry the
  selector `"{iframe} [frame] {element}"` and a `frame:` node identifier, exclusions (#645)
  apply here as well. Skipped are hidden frames (`hidden`, `aria-hidden`,
  `role=none`, invisible, at most 1 px, ignored in the AX tree), excluded and
  out-of-process (cross-origin) frames; `execution.frames` in the JSON lists checked and
  skipped frames including the reason (`cross_origin`, `hidden`, `excluded`, `ax_tree_failed`,
  `document_unavailable`) per viewport. The JavaScript scan `iframe/same-origin-content` now checks
  only `html-has-lang`, its other checks are covered by the pass. New
  detection corpus case `iframe_widget_rules` and a Chrome integration test that also proves
  that no page rule reports inside the frame. Still open is the tablist made of `ul`/`li`
  (magyarorszag.hu): Chrome flattens the `li` between tablist and tab in the AX tree, so the
  tree-based rules see a valid structure there — also in the main document. Live on
  gov.cy the dialog sits in an out-of-process frame (`digital-assistant.gov.cy`, `gov.cy` is
  a public suffix): it now appears in the report as skipped with `cross_origin`; it will be checked
  only with a dedicated CDP session per out-of-process frame.

- **Screen reader exclusions survive the cache (#708):** With `--reuse-cache`
  the screen reader report is rebuilt from the stored AX tree; the nodes excluded
  in #703 were back in the sidecar and in the PDF afterwards. The excluded
  node IDs now live in the cache entry (`snapshot.json`,
  `screen_reader_excluded_node_ids`) and are applied again on rebuild. Older
  entries without the field behave as before.

- **Contrast in shadow DOM and across float containers (#716):** Two gaps
  compared with axe. (1) The contrast check ran only over the light DOM; text in web components
  (cookie banner `p-cookie-banner` on verwaltung.bund.de, 3.17:1) was missing. It now goes through
  open shadow roots, the background is determined across the composed tree (slot,
  shadow host), text directly under a shadow host takes color and font from its slot (without a
  slot it is not rendered and not checked), `aria-hidden` on the host acts inward, and the
  selector reads `host >>> inner` (also in the pixel probe). Overlay detection now also sees
  positioned siblings that contain an image (hero image as `<p-responsive-image>`), up to
  six levels up; the pixel probe reports covered text (banner over the text) as a
  review hint instead of measuring it against the banner. (2) `__amsIsVisuallyHidden` considered every
  ancestor without height hidden; a `div` that contains only floats is 0 px high, though,
  and shows its content. Only a box that clips its overflow hides something without area;
  what is not rendered is detected by `checkVisibility()`. As a result the 4.46:1 link on
  slovensko.sk is reported (axe: 4.45:1; the comparison against the threshold was already unrounded). The
  helper is also used by design quality, journey and the media rule. Live: bund.de reports the
  privacy link and the white hero text over the photo (pixel probe 3.64:1), slovensko.sk the
  link. New detection corpus case `contrast_shadow_and_inline`.

- **Scrollable regions without keyboard access (#717):** New rule
  `scrollable-region-focusable` (2.1.1, A) → `a11y.scrollable_region_focus.missing`: a
  visible element with `overflow: auto|scroll` that overflows by more than 13 px (like axe),
  contains text and is focusable by keyboard neither itself nor via a descendant.
  Chrome has made such regions focusable itself since version 130, other browsers have not —
  so it remains a finding. New detection corpus case `scrollable_region_focusable`.

- **Weight of `zoom/viewport-missing` (#702):** Desktop browsers ignore the
  viewport meta tag, zooming by gesture remains possible; the consequence affects only mobile devices. The
  score deduction drops from 2.5/5 to 1.5/3, report and explanation name the consequence (desktop width,
  shrunk, horizontal scrolling when zooming in). The finding level (FAIL) and severity still come
  from `a11y-rules`; whether it should be a review hint there is a question for barrierlab.

- **Color as a link cue only in body text, motion only with evidence (#710, #712):**
  `link-in-text-block` (1.4.1) now checks a link only when it sits in body text — like
  axe `isInTextBlock`: the section of its block (delimited by `<br>`/`<hr>`) must contain more
  non-link text than link text, and at least two words. Links in `nav`
  or menus and logo links (image/SVG in the link, logo container) drop out. As a result,
  navigation, header, footer and logo links no longer report a finding (12 of 21 EU portals,
  axe confirmed none); the link in a paragraph remains a finding. `prefers-reduced-motion`
  (2.3.3, AAA) no longer reports every `transition`/`animation`, but only motion —
  animations whose `@keyframes` change transform/translate/scale/rotate, position or margin,
  and transitions of such properties (`all` only if a `:hover`/`:focus` rule
  sets it) — that no `@media (prefers-reduced-motion: reduce)` block disables for the same or the
  universal selector; rules under `no-preference` do not count. Color and
  opacity transitions never count. The message names the evidence and the AAA level; AAA was
  already neither a legal signal nor highly BFSG-relevant, a test now secures that.
  New detection corpus cases `link_in_text_block_context`, `reduced_motion_colour_only`,
  `reduced_motion_transform`, `reduced_motion_override`. Only motion whose selector hits a rendered element of the page is counted
  (bundesregierung.de: keyframes of a player spinner that is never included).

- **title-only and duplicate description as their own best-practice rules (#711,
  #713):** The `title` attribute as the only description of an interactive element was reported as
  `a11y.hover.content_visibility` under 1.4.13, although the browser tooltip is exempt there;
  it is now `title-only-description` → `a11y.title_only_description.weak`. An element whose
  name and description are identical was reported as `a11y.interactive_name.missing`,
  although it has a name; it is now `description-duplicates-name` →
  `a11y.description_duplicates_name.redundant`. Both are `best-practice`, anchored to 4.1.2
  (`title` is a valid name source per accname, 2.5.3 applies only to a visible
  label), have `bfsg_relevance: low` and never a legal flag — `is_convention_rule`
  keeps a list of best-practice identifiers alongside the `display/*` rules for this. 1.4.13
  now reports only orphaned `role="tooltip"` elements, `interactive_name.missing` only
  elements without a meaningful name. New detection corpus case
  `name_description_best_practice`, `forms_and_misc` extended by an orphaned tooltip.

- **a11y-rules 0.13.5 (#709):** The skip link is now also detected behind the links of a
  cookie banner when it points to the start of the main content (bund.de,
  casoon/barrierlab#26). If a modal dialog is open (for example a consent dialog that hides `<main>` via
  `aria-hidden`), `landmarks/main-missing` is a review hint "gemessen hinter einem
  offenen Dialog" ("measured behind an open dialog") instead of a violation (#709: five of 21 EU portals), also for a dialog
  without `aria-modal` (administracion.gob.es). Likewise `bypass` ("keine Ueberschriften", i.e. "no headings") behind
  an open dialog becomes a review hint. Reference band gov.ie re-rated (80–98): the ten link color
  findings counted up to 1.7.1 were standalone list links, false alarms after #710.

- **Target size measures only visible targets in the final layout (#705, #706):**
  `target-size-minimum` (2.5.8) and `target-size` (2.5.5) now count an element as a target only
  if it is visible and clickable: not hidden per `checkVisibility()` (with
  `opacityProperty`, `visibilityProperty`; deliberately without `contentVisibilityAuto`, so that sections
  outside the viewport remain measured), not in `[inert]`, without
  `pointer-events: none` — as already with the equivalent link. This applies to the measured target
  and to the neighbors of the spacing check; links in a closed `<details>` (Chrome
  keeps their box, hides it via `content-visibility: hidden`) made three
  links in the footer and in a list a finding on geographia.eu. Without `checkVisibility` the previous
  size filter remains. In addition, after the stability check every viewport pass waits for
  running finite CSS animations and transitions (`document.getAnimations()`, infinite ones
  are ignored), bounded by a second `--stability-budget-ms`; an animation whose
  remaining time exceeds the budget is not waited for at all. A target that is still animating afterwards, itself
  or via an ancestor, is reported by the rule as not measured (`untested`)
  instead of with its intermediate size: the logo intro on geographia.eu/atmosphere/ appeared as 3×39
  px in the report, once finished it measures 147×39 px. New detection corpus cases
  `target_size_hidden_neighbours` and `target_size_animation`; against the state before the fix
  `a#beside-details` and `a#slow` fail. Verified with `cargo test`, Clippy and the
  detection corpus (Chrome).

- **Display rules no longer decide a legal verdict (#704):**
  `display/text-hidden` no longer reports a `[data-viz-text]` hidden via CSS,
  `hidden` or `aria-hidden` as a violation if a rendered, reachable element of the same
  `figure[data-viz]` (or the figure itself) references it via `aria-describedby`/`aria-details`:
  accname 1.2 also computes the description from hidden, directly referenced content (ARIA15
  satisfies 1.1.1). Instead, a review hint with low severity ("nur als Beschreibung,
  visually-hidden empfohlen", i.e. "only as a description, visually-hidden recommended"). `inert` remains a violation. All `display/*` rules are
  best practice of the display convention: they set `bfsg_relevance` to at most `low` and
  no longer count as `legal_flags` (page and batch verdict, `passed` in the JSON); a
  shared `NormalizedFinding::is_legal_flag` replaces the identical filters. Deliberately
  only for `display/*` (`taxonomy::is_convention_rule`), not generically via the
  `best-practice` tag — `region`, `link-as-button` and `redundant-role` keep their classification.
  `display/toggle-missing` now says "kein mit `[data-display-toggle]` markierter Umschalter
  gefunden" ("no toggle marked with `[data-display-toggle]` found") instead of claiming that one cannot toggle. Evidenced by the geographia.eu case
  (detection corpus `display_modes_described_text`).

- **Exclusions also apply to the screen reader layer (#703):** Nodes in
  `--exclude-selector`/`[data-audit-exclude]` subtrees (#645) now also drop out of
  `screen_reader.issues` and thus out of `bfsg_compliance` and the `risk.legal_flags` derived from it
  (#484). Location is determined via the backend node ID of the AX node, as with the
  tree rules (`ExclusionScope::excludes_ax_node`). A finding is dropped only if all its
  nodes are excluded; otherwise it remains with the other nodes. Findings without nodes
  (site-wide) always remain. Counted in `exclusions.excluded_screen_reader_issues` (also in the
  batch aggregate and in the PDF note); the PDF, which re-derives the findings in the run language,
  omits the same nodes. Evidenced on barrierlab.eu `/tasks/ticket/` (detection corpus
  `audit_exclude_screen_reader`, which can check `screen_reader_bfsg_verdict` for this).

- **1.7.1, 2026-09-30:** First step of moving the rules to barrierlab (casoon/barrierlab#13):
  21 identifiers come from `a11y-rules` 0.13.2, the superseded own rules are deleted (#690,
  B0). New findings `zoom/viewport-missing` (weighting follows in #702) and `images/alt-suspicious`;
  the shared rules no longer check anything hidden via CSS. Reference run
  (`reference_sites_test`) and detection corpus green. Details in the entry below.

- **Shared identifiers instead of own rules (#690, B0):** More findings run
  through the shared set from `a11y-rules` 0.13.2 (`SHARED_RULES` in `src/wcag/shared.rs`);
  the superseded own rules are deleted, so that no finding appears twice in the report.
  Identifiers: `meta-viewport` → `zoom/viewport-locked`; `document-title` (missing/empty) →
  `document/title-missing`/`document/title-empty`, the meaningless title remains as
  `document-title`; `aria-valid-attr` → `aria/reference-missing`; `aria-required-attr` →
  `aria/required-attribute-missing`; `aria-hidden-focus` and `focus-order-semantics` →
  `keyboard/hidden-focusable` (previously the same case twice, under 4.1.2 and 2.4.3);
  `landmark-one-main` and `landmark-main-present` → `landmarks/main-missing` (previously twice);
  `landmark-no-duplicate-main` → `landmarks/main-duplicate`; `landmark-banner-present`,
  `landmark-navigation-present`, `landmark-contentinfo-present` → `landmarks/*-missing`, now
  as a review hint instead of a violation; `svg-img-alt` and `image-alt` on `<svg>` → `svg/name-missing`;
  `image-alt` on `<img>` → `images/alt-missing` (`image-alt` remains for `role="img"` on other
  elements and the icon heuristic); link and button without a name from `control-missing-label` →
  `links/name-missing`/`buttons/name-missing`; the skip link detection from `bypass` →
  `keyboard/skip-link-missing` (review hint, recognizes the skip link by its target instead of by a
  word list). `bypass` now reports only a page with no headings at all; the combined message
  "neither skip link nor main" is dropped. New: `zoom/viewport-missing` (no viewport meta tag,
  1.4.4) and `images/alt-suspicious` (file name, filler word or one or two characters as alt text;
  review hint). The shared rules now run with computed `display`/`visibility` from
  a DOMSnapshot (`run_full_in` via `CdpDocument::rendered`) and thus no longer check anything
  hidden via CSS; if the snapshot fails, they run without styles. An element without
  a counterpart in the AX tree counts as ignored (content of a closed `<details>`: geographia.eu
  otherwise got 22 `links/name-missing`). `skip-link` from `landmark_granular` is dropped, the case is
  `keyboard/skip-link-missing`. User-agent shadow roots (date fields) no longer enter the
  shared DOM. Taxonomy, explanations and detection corpus are updated accordingly. Verified with
  detection corpus (Chrome) and a before/after run against 1.7.0 on eight live sites; every
  deviation is explained (rename, dropped double report, hidden via CSS, or new
  and justified), two false alarms found along the way fixed in `a11y-rules` 0.13.2.

- **Score calibration: `table_missing_caption` is a clean page, 2026-09-30:** The fixture
  sat in the band "exactly one Level-A High"; that finding was the false alarm from #659/#674 (implicit
  `<tbody>` under `<table><tr>` hid the rows from the required-child check). Since #680
  the page carries only the caption review hint and, at 95, sits in the band ≥ 95; the tag run
  of v1.7.0 had failed because of it.

- **1.7.0, 2026-09-30:** New features: technician mode `--technician` (JSON per page plus
  `index.json` and `findings.jsonl`, path filters `--include-path`/`--exclude-path`),
  display modes `--display calm|text|visual|all` (#653), exclusion of intentional examples via
  `data-audit-exclude`/`--exclude-selector`, visible in the report (#645), and fully
  English PDF reports with `--lang en`. In addition the false-alarm corrections from #638-#659 and
  #673/#674 as well as the restructuring (WP0-WP8, output unchanged per golden harness). Crate
  metadata: `documentation` points to https://casoon.github.io/auditmysite/, `homepage` to
  https://auditmysite.casoon.de/. Details in the entries below.

- **Refactoring API surface (WP8), 2026-09-29:** Pure restructuring
  without changes to output or public library API. The 199 places that the lint
  `unreachable_pub` reports (158 in the library, 41 in the binary) are narrowed per compiler suggestion
  to `pub(super)` (140) and `pub(crate)` (59) respectively — they were
  invisible to library users anyway; reachable `pub` items remain untouched (semver/Studio, 1.7). `lib.rs` and
  `main.rs` now set `#![warn(unreachable_pub)]`. The 100 comparisons
  `i18n.locale() == "en"` now go through the existing helper `output::localized::is_english`;
  for this, `output::localized` is no longer tied to the feature `pdf` (only `pick` remains so).
  Verified: golden harness identical before/after, Clippy with both feature sets.
- **Refactoring `audit/pipeline.rs` (WP7), 2026-09-29:** Pure
  move without changes to flow or output. `src/audit/pipeline.rs` is now the
  directory `src/audit/pipeline/`: `mod.rs` keeps `PipelineConfig`, `run_single_audit`,
  `audit_page` with both viewport passes, the rule runs and the artifact persistence;
  `assembly.rs` takes the synchronous post-processing (WCAG merging, viewport scores,
  `aggregate_report`, module runs, audit quality); `throttled.rs` the throttled
  performance passes including `recover_after_throttled_failure` and the adoption of the
  LhMobile value. In `audit_page` only the verbatim duplications are extracted (the three
  `prepare_*_collection` calls per pass, the screenshot match); order of
  exclusions (#645), display modes (#653) and journeys unchanged, a shared
  `run_viewport_pass` deliberately not introduced. Public paths (`auditmysite::audit::{audit_page,
  run_single_audit, PipelineConfig}`) remain. Verified: golden harness before/after identical,
  `integration_test` and `detection_corpus_test` (`--ignored`) against real Chrome.
- **Refactoring: module details and page health as directory modules (WP6), 2026-09-29:**
  Pure restructuring without behavior change. `output/builder/single/module_details.rs` (2,618 lines)
  is now a directory module with one file per module, mirroring `output/pdf/detail_modules/`:
  `performance.rs`, `seo.rs`, `platform.rs` (security, mobile), `html_conform.rs`, `commerce.rs`,
  `dark_mode.rs`, `design_quality.rs`, `ai_transparency.rs`, `experience.rs` (UX, journey); the
  orchestration (`build_module_details_from_normalized`) stays in `module_details.rs`. The
  inline computations in the `SeoPresentation` literal are named functions (`heading_summary`,
  `social_summary`, `technical_summary_rows`, `tracking_summary_rows`,
  `build_robots_presentation`, `build_image_efficiency_presentation`, `technical_issue_rows`).
  `seo/page_health.rs` (3,128 lines) keeps types, text functions, `analyze_page_health` and the
  URL analysis; new are `page_health/dom.rs` (DOM inspection, local HTML validation),
  `page_health/probes.rs` (HTTP probes) and `page_health/issues.rs` (`collect_issues`). All
  public paths remain (`seo::page_health::collect_issues` via `pub use`). The rename
  `run_w3c_html_validation` → `run_local_html_validation` was already done with WP9.
  Verified with the golden harness (WP0) against `main`: `diff -r` empty.

- **Display modes (`data-display` convention), 2026-09-29 (#653):** New module `src/display/`
  and new option `--display calm|text|visual|all`. Without the option everything stays as before (page default).
  With a mode, a script injected before navigation
  (`Page.addScriptToEvaluateOnNewDocument`) writes the choice to `localStorage.display`; `calm` and `text`
  additionally emulate `prefers-reduced-motion: reduce` (`Emulation.setEmulatedMedia`), so that pages
  without the convention also show their reduced variant. Because `setEmulatedMedia` replaces the whole
  feature list, the dark-mode analysis (print, forced colors, dark contrast) now passes the
  base features through (`analyze_dark_mode_with_base_media`) — otherwise it would have cleared the reduced
  motion for the rest of the audit. JSON: `audit_scope.display_mode`
  (`site_default|visual|calm|text`) and per page `pages[].display_modes` (offered modes,
  rendered mode, `set_before_body`, toggle, visualizations by kind, reduced motion);
  PDF: rows "Darstellungsmodus"/"Darstellungsmodi der Seite" ("Display mode"/"Display modes of the page") in the methodology appendix, in the batch on the
  cover page. `--display all` checks each mode as its own run with its own report (suffix
  `-calm`/`-text`/`-visual`), never a mixed score; `visual` only for pages with
  `figure[data-viz="3d|interactive"]` from the `calm` run, with a doubled page timeout. Findings
  are not marked as "in all modes" — for that the runs would have to be merged.

  New rule group `display/*` (best practice, no WCAG requirement, anchored to 2.2.2 and
  1.1.1 respectively; own taxonomy entries and explanations): `display/toggle-missing`,
  `display/init-missing` (via an observer that records, when `<body>` is inserted, whether
  `data-display` was already set), `display/text-media-visible`, `display/text-not-visible`,
  `display/text-hidden`. The text mode checks follow the mode actually rendered.
  Candidates for moving to barrierlab: the static halves of `toggle-missing` and
  `text-hidden` (attribute part = `viz/text-hidden` of the draft). Corpus fixtures
  `display_modes_conforming`/`display_modes_violations`, integration test
  `display_text_sets_stored_choice_and_reduced_motion` (localStorage and `matchMedia` before the
  first script and after the audit).

  Live against geographia.eu: `--sitemap .../sitemap-index.xml --display calm -m 10` 266 s, 10/10
  pages, no pool timeouts (the first 10 URLs are text pages without visualization). Ten pages
  with visualizations via URL list: `calm` 1151 s, 7/10, 0 pool timeouts, but 3 pages with
  `interactive` run into the 120 s audit limit; `text` 293 s, 10/10, 0 timeouts. Findings: on all
  pages `display/toggle-missing` (no `[data-display-toggle]` shipped); in `calm`/`visual`
  `display/text-hidden`, because `.viz-text { display: none }` removes the text layer outside of `text`
  for screen readers as well (confirmed in the browser); in `text` `display/text-not-visible` on
  pages without `[data-viz-text]` (home page 4, explanation lab 10).

- **English report: filler words as topics, misleading validator title, 2026-09-29:**
  Finding from a scan of the English Typst outputs of the golden set (17 pages).
  (1) The batch report listed the entry "nicht" ("not") under "Dominant content topics" ("Present on
  2 audited pages"). Cause: `seo/topics.rs` filters page title, description, headings and
  text excerpt against `topic-stopwords` from the FTL files, and these lists contained only
  some of the function words — "nicht" ("not") (barrierlab.eu, casoon.de), "weitere" ("further") (berlin.de),
  "dein"/"dich" ("your"/"you") (hornbach.de, otto.de) and "neue" ("new") (volkswagen.de) slipped through as page topics.
  Both lists extended by common function words (negation, auxiliary and modal verbs, pronouns,
  conjunctions, prepositions) with at least four characters; shorter ones the code filters
  anyway. Real topic words remain. (2) The callout on HTML validation was titled "W3C HTML
  validator"/"W3C HTML Validator", although the check runs locally with html5ever and never calls
  the W3C service. Now "HTML validation (local, html5ever)" and "HTML-Validierung (lokal,
  html5ever)" respectively; the FTL key is called `pdf-ph-html-validator-title`. (3) The German rows
  of the HTML validation table in the English PDF had already been fixed with #671. (4) The
  terminal output showed "Certificate: SEHR GUT" ("very good") with `--lang en`. The certificate in the JSON deliberately remains
  the canonical German token (#449: key for badge/color, fixed via enum in the
  Studio contract schema, checked by report-lint and evaluated by rankinglab); only
  the display is localized. (5) The validator callout compared the localized status with
  "Fehlgeschlagen" ("Failed"); in the English PDF ("Failed") a failed validation was therefore
  shown as info instead of as a warning. The presentation now carries the canonical status
  (`executed`/`failed`/`skipped`), and the callout decides on that. `certificate_label_localized` now lives in `registry` instead of the
  PDF cover so that the terminal uses it too: with `--lang en` it now reads "EXCELLENT". Likewise
  the "Certificate" row in the "Audit scope" box of the English batch report
  showed the German token ("AUSBAUFÄHIG", i.e. "room for improvement"); it now uses the same function ("INADEQUATE").

  **Changes the output:** page topics (`topic_terms`, `top_terms`, topic overlaps) in the
  JSON and in both PDF languages, where a page had a function word among its five topics;
  title of the validator callout in both languages; certificate row in the English terminal and in the
  English batch "Audit scope" box. The JSON field `certificate` remains unchanged.
  Verified with the golden harness against `main`: differences only at these places, German
  Typst verbatim identical except for validator title and topic list. Unit tests against function words
  as topics and against umlauts in the English certificate labels.
- **`document-title` double-counted, 2026-09-29:** A missing or empty page title appeared
  twice in the report, once from the AX check `check_page_titled` (node `document`, without
  selector) and once from the DOM check `check_page_titled_with_page` (selector `head`).
  Authoritative is now the DOM check alone: Chrome fills the name of the root node with the URL
  when the title is missing, so the AX check could not decide this case itself and already
  referred to the DOM check before. Everything it reported as a violation (empty, missing,
  generic title) the DOM check reports as well; the AX check is removed together with its tests
  (public function `wcag::rules::check_page_titled` is dropped). `empty_title` pins 1 occurrence, the integration test
  on `parity_gaps.html` does too.

- **Tables: ignored and not-yet-rendered rows (#654, #659), 2026-09-29:**
  (1) `aria-roles` reported on og-vanilla.casoon.dev (`/accessibility`, `table.keys`) "role 'table'
  is missing required child roles: row, rowgroup" for a table with `tbody > tr`. Chrome
  marks a plain `<tbody>` as ignored, the rows hang below it; the check of the
  required child roles saw only direct children. It now looks through ignored nodes
  (same pattern as #638). A table without a row (only `caption`) is still reported.
  (2) `th-has-data-cells` reported the header cells of the `role="grid"` grid in a shadow root
  (`/columns`, `/accessibility`), although every column has data. Reproduced via CDP, the
  capture came before the data arrived: the grid keeps 40 pool rows with `display: none` in the
  DOM until the worker delivers (around 300-500 ms after `load`); Chrome lists them as ignored
  with `notRendered`. The stability wait does not see mutations in shadow roots and reported
  the page stable after 255 ms. If a table without data cells contains such not-rendered rows
  at row position (directly under the table, under a rowgroup or an ignored
  `<tbody>`), the result now counts as undetermined (`incomplete`) instead of as a violation. Hidden
  content inside a header cell does not count; a table with only a header row is still reported.

  New corpus fixtures `table_required_rows_tbody` and `table_grid_rows_unrendered` (shadow DOM
  grid with waiting and rendered rows respectively, table with only a header row as a violation) and
  unit tests in both rules.

- **Technician mode: single-page JSON to fix things instead of a report to read, 2026-09-29:**
  New for people who work through findings: `--sitemap URL --technician -o DIR/` (likewise with
  `--url-file`/`--crawl`) writes the regular single-page report as JSON per page and next to it two
  flat files. `index.json` lists every attempted URL in input order — file name or
  `null`, status `ok`/`blocked`/`failed` with reason, overall and accessibility score, number of
  findings and occurrences, `audit_quality`. Blocked (bot wall, `AccessBlocked`) and
  failed pages appear only there, without a placeholder file; for this, `BatchError` now carries
  `blocked_reason` (not serialised, batch JSON unchanged). `findings.jsonl` contains one
  line per occurrence across all pages (`url`, `source` wcag/journey/seo/html_conform, `rule_id`,
  `wcag_criterion`, `level`, `severity`, `selector`, `location`, `message`, `fix_suggestion`,
  `viewport_tags`), complete — the page file caps its example occurrences per finding at
  five. Both files are canonically English (#406), schemas in
  `docs/technician-index.schema.json` and `docs/technician-finding.schema.json`. Every run with
  `--per-page-reports -f json` writes them.

  `--technician` stands for `--per-page-reports -f json --no-screen-reader-report --seo
  --html-conform`: accessibility (incl. keyboard journeys), HTML conformance and SEO, no
  throttled performance runs, no mobile/security/tech stack. Every page file states this scope in
  `execution.scope`/`module_runs`. Explicitly set flags win: `-f`
  replaces the format, `--full`/`--performance`/`--mobile`/`--security` enable their modules as
  usual. The new flags can also be used on their own: `--include-path`/`--exclude-path`
  (repeatable, glob on the decoded URL path: `*`/`?` within a segment, `**`
  across segments, `/**/` also as a single `/`; takes effect before `-m`; `sample.selection`
  then becomes `path_filter`), `--no-screen-reader-report` (no `*-screen-reader-audit.json`, also in
  single mode) and `--html-conform` (HTML conformance without `--full`).

  Not in `findings.jsonl`: `accessibility_assessments` (hints/manual check items, not
  violations). The derived indicator modules (UX, Journey, Commerce, Content Visibility, AI
  Visibility, Source Quality, Dark Mode) keep running because they hang off `check_seo` or are tied
  to `PipelineConfig`; switching them off would need new `PipelineConfig` fields.
- **3.2.2 On Input: only real context changes are violations, 2026-09-29 (#657):** On
  og-vanilla.casoon.dev (`/filtering`, `/sorting`, `/localization`) `a11y.on_input.risk` reported
  selects for filter preset, sorting and language as "may trigger
  navigation", just because their name contained "filter"/"sort"/"language" and the page has no
  submit button. But the selects only update the grid next to them — a content change
  is not a change of context. Now only what the inline `onchange` handler
  (or the global function it calls directly) visibly does is a violation: navigation (`location…`),
  submitting a form (`.submit(`/`.requestSubmit(`), a new window (`window.open(`) or
  a focus shift (`.focus(`). A handler whose effect cannot be read off, and a
  name that merely sounds like navigation (language, country, region, navigate, redirect, go to),
  become a check warning instead of a violation; "sort" and "filter" disappear entirely as a hint.
  The classification now lives in Rust (`evaluate`) and is unit-tested. New corpus fixture
  `on_input_context_change` (passes: filter without handler; check: content-changing handler,
  language select; violation: handler navigates via a called function, handler submits a form).

- **1.3.5 Identify Input Purpose: the "name" of a thing is not a person's name, 2026-09-29 (#658):**
  On og-vanilla.casoon.dev/saved-views, `#view-name` (label "Name", the name of a saved
  grid view) counted as a field for the user's name because the label contained "name" as a
  substring. "name" is now classified word by word (split at punctuation and camelCase): a
  preceding word that is neither a person nor a technical id prefix ("view name",
  "project name", "company name"), a following "of/for/der/des/für/von" ("Name der Ansicht" (name
  of the view)) and compounds like "Dateiname"/"Filename" (file name) exclude the field; "Name",
  "Your name", "Full name", "Vorname" (first name), "Nachname" (last name), "Ihr Name" (your name)
  remain hits. A thing qualifier in `id` or the `name` attribute excludes a bare "Name" label,
  unless the other attribute names the person. The detection words are English and German merged
  (as with `media_alternative`), independent of the output language. New corpus fixture
  `input_purpose_name`.
- **Exempt deliberately broken examples from the audit, visible in the report, 2026-09-29 (#645):**
  Pages that teach accessibility deliberately show faulty examples (barrierlab.eu,
  `/tasks/ticket/`: unlabelled fields, a `div` as a button). auditmysite reported them as
  violations of the page itself. New, modelled on axe-core's `exclude`:
  `--exclude-selector <CSS>` (repeatable), `[audit] exclude_selectors` in `auditmysite.toml`,
  and the always-honoured attribute `[data-audit-exclude]`. What is excluded is never a page and
  never a rule, only findings whose element lies in an excluded subtree —
  the page itself is checked completely, page-wide findings always remain.

  Implementation (`src/audit/exclusion.rs`): per viewport pass, one `Runtime.evaluate` counts the
  hits of each selector (invalid selectors are detected); only when there are hits,
  `DOM.getDocument` + `DOM.querySelectorAll` fetch the backend node IDs of all nodes in the hit
  subtrees (incl. shadow roots, frame and template content). Filtering happens per rule in
  `run_rules`, before its execution note is counted, and before enrichment and
  element screenshots: findings from the AX tree via the
  backend ID of their AX node, findings of the shared DOM rules via the newly carried
  backend ID (`Violation::backend_node_id`, in memory only; set in the adapter
  `wcag/shared.rs`, nothing changes in barrierlab's rule code), JavaScript page rules,
  contrast and journey findings via their selector — excluded only if it hits at least one
  element and **every** one of them lies in the excluded subtree; an ambiguous or
  unparseable selector keeps the finding. Pattern findings go through the same filter.

  Visibility: the JSON carries per page `pages[].exclusions` (applied selectors with
  hit count — also 0 and invalid —, excluded occurrences and confirmed violations
  in total and per rule, excluded journey findings; per rule the larger value of the two
  viewports), the batch additionally `summary.exclusions` as a sum. The PDF names selectors and
  numbers in the methodology section, the batch in the audit frame of the cover page (de/en via Fluent). A
  default run without `data-audit-exclude` on the page stays silent in the PDF, the JSON lists the
  built-in selector with 0 hits. Cache format 19; the selectors go into the
  audit signature.

  Caps of the JavaScript rules: many page rules report at most N locations (10 for
  `click-events-have-key-events`, 20 for `link-as-button`, 250 for the ARIA rules, 5 for
  target size, obscured focused elements, language changes, among others). Without a countermeasure,
  example hits would have used up the cap and let real hits behind them disappear
  silently — exactly what the feature is meant to rule out. Therefore the exclusion step puts
  `window.__amsIsExcluded(el)` into the page per pass (the hit roots in a
  `WeakSet` in the closure, no DOM attribute, no change visible to other rules). The
  shared building block `CSS_SELECTOR_JS`, which every locating rule includes, brings
  `__amsIsExcludedEl`, `__amsPush` and `__amsReal` with it: real and excluded hits are
  capped separately, the loop limit reads only the real ones. Excluded hits still come
  back and are filtered and counted like all others (per rule at most up to the
  same cap). Converted: `aria_allowed_attr`, `aria_hidden_focus` (whose total
  no longer counts excluded ones), `aria_relationships`, `aria_roles` (2),
  `aria_required_attr`, `image_input_rules`, `on_focus`, `on_input`, `server_side_image_map`,
  `table_extended`, `widget_rules`, `click_handlers`, `content_on_hover`, `fake_navigation_link`,
  `redundant_role`, `use_of_color`, `language_of_parts`, `meaningful_sequence`,
  `target_size_minimum`/`_enhanced` (also the candidate lists), `focus_not_obscured_minimum`/
  `_enhanced` (focus candidates, the 60 checked controls, the 5 findings),
  `pause_stop_hide` (widgets; the page-wide animation message ignores excluded
  elements entirely) and `accessible_authentication` (cap moved from Rust into the script).
  `identify_purpose` reports a field name instead of a CSS path — the script marks
  excluded fields itself (`Violation::in_excluded_subtree`, in memory only), the
  cap of 5 applies separately. Contrast: text in excluded subtrees no longer occupies any of the 60
  slots for the pixel sample.

  `rule_outcomes[].findings` now counts what is in the report: the tree rules filter per rule
  in `wcag::check_all_excluding` (new; `check_all_with_config` remains as a wrapper), the
  page rules, contrast and HTML content model before `page_rule_outcome`; for the shared
  rules the counter taken over from `a11y-rules` is reduced by the dropped findings of the same
  identifier.

  Still open: rules whose reported "selector" is not a CSS path and which do not mark their element
  themselves cannot lose excluded hits (known and fixed:
  `identify_purpose`). Caps that only limit candidates for a page-wide statement
  (overlay search in `focus_not_obscured_*`, fields per form in `redundant_entry`) are
  unchanged.

  Tests: unit tests for filter, counting and PDF text (EN without umlauts), new corpus fixture
  `audit_exclude_specimen` (one error per locating path — AX node, JS page rule, shared
  DOM rule — not reported inside `[data-audit-exclude]`, the same error outside is
  reported) and an integration test for `--exclude-selector` including a 0-hit and an
  invalid selector. In addition corpus fixture `audit_exclude_cap`: 12/22/6 excluded hits
  before one real one each (caps 10/20/5) — the real one is reported, and
  `excluded_hits_do_not_spend_a_capped_rules_budget` checks the same including the exclusion count
  and `rule_outcomes[].findings == 1`. Counter-check: with `__amsIsExcludedEl` switched off, the
  test fails (`div#real-click` is missing).

- **4.1.2/3.3.2: expandable treegrid rows and native value fields, 2026-09-29 (#655, #656):**
  (1) The accordion pattern treated every element with `aria-expanded` as an accordion trigger
  and, on og-vanilla.casoon.dev/grouping, reported every group row of the `role="treegrid"` as
  "should be a button" (Medium, 12 occurrences). `aria-expanded` on a row in `grid`/`treegrid`
  and on a `treeitem` is the state of its own composite widget, whose keyboard contract (arrow keys, Enter)
  is provided by the grid or the tree; such nodes no longer count as triggers. An expandable row outside a grid is still reported.

  (2) `aria-required-attr` checked `aria-valuenow` via an AX property `valuenow` that CDP
  does not know — the current value is in the node's `value`, and Chrome invents one (50 for the
  slider, 0 for the spinbutton) when the author sets none. The check therefore fired on every
  `slider`/`spinbutton`/`meter`/focusable `separator`, including native
  `<input type=range|number>` and the internal day/month/year fields of `<input type=date>`
  (Critical; 16 occurrences on /filtering, /columns, /grouping, /theming). For the same reason
  `check_slider_has_value` ("Slider is missing accessible value") reported every slider: the
  extractor only reads text values, the slider's number was dropped. The `aria-valuenow` requirement
  now runs as the DOM rule `check_value_now_with_page` (as `aria-checked` already does): it reads the
  attribute, traverses open shadow roots, exempts native `range`/`number` and `<meter>`, and
  does not reach the user-agent fields of date inputs at all. The slider check in
  `widget_rules.rs` is dropped; a custom `role="slider"` without `aria-valuenow` remains a violation.

  (3) The 3.3.2 heuristic "may require format instructions" skips nodes inside the
  Chrome roles `Date`/`DateTime`/`InputTime` — the browser provides their fields and format. The
  `spinbutton` role alone no longer triggers the hint: 3.3.2 requires instructions where
  the input must follow a format that cannot be inferred; a spinbutton — native
  `<input type=number>` as well as a custom `role="spinbutton"` — holds a number that the widget itself
  limits and changes stepwise with the arrow keys. Detection via the label
  ("Date", "Postal code") still applies to every role.

  (4) The AX extractor read `value` only as a string; Chrome sends the value of slider,
  spinbutton, progressbar and meter as a number, which was lost as a result. Numbers and booleans
  are now preserved in text form. No WCAG rule reads `AXNode.value` any more (the only one,
  `check_slider_has_value`, is dropped above); the only consequence is for the screen reader linearisation
  (`a11y-perception`): value elements without a name now carry their value and count as announced
  content in the announcement-desert measurement, which is what they are when read aloud.

  Verified live with the release build against og-vanilla.casoon.dev: no more accordion,
  `aria-valuenow`, slider value or spinbutton format findings. New corpus fixtures `treegrid_expandable_rows`,
  `value_widgets_native` (pass), `value_widgets_custom` and `value_widgets_shadow` (violation);
  `aria_and_widgets` now expects the custom slider under `aria-required-attr`.

  Follow-up on the same class of error: the remaining 1.3.5 keywords were also searched as
  substrings — "Hotel" counted as a phone field because of "tel", "Sorting" would have contained "ort". They
  now apply only as whole words (tel, zip, plz, fax, city, first, last …) or as known
  compound beginnings or endings (`telefon…`, `postleit…`, `birth…`, `geburts…`, `…adresse`,
  `…address`, `…strasse`); "E-Mail" counts as one word. German fields like "Telefonnummer" (phone number),
  "Postleitzahl" (postal code), "Straße" (street), "Passwort" (password) are thereby recognised for the first time. Likewise for 3.2.2: the
  page-wide search for a submit button found "go" in "Google", "Category" or "Logo" and
  thereby suppressed the check warning; button names are now checked word by word
  (submit, send, go, search, absenden, senden, suchen), the decision lives in Rust and is
  unit-tested. Corpus: `#stay` ("Hotel") passes, "Telefonnummer"/"Postleitzahl" are
  violations; the On Input fixture has Google/Category/Logo buttons, the language select remains
  a check.

- **`landmark-unique` counted twice, 2026-09-29:** The rule ran twice, once over the
  AX tree (`check_landmark_unique`) and once in the DOM supplement (`check_landmarks_with_page`).
  Both reported the same elements with differently built selectors, so nothing was
  merged: the corpus case `landmark_granular` came to 22 instead of 11 occurrences,
  `parity_gaps.html` to 4 instead of 2. Only the AX check is authoritative now; it works
  with Chrome's computed roles and names and covers every landmark that the DOM approximation sees
  (plus `<section>` with a name, `form`, and correct roles for nested `header`/`footer`/
  `aside`). The same applied to `landmark-main-present`: a missing `main` landmark appeared
  twice in the report (`root` and `document`). The DOM supplement had no coverage of its own:
  if the AX tree fails, the whole audit aborts; for mini pages with at most
  two AX nodes the AX check is silent on purpose; and a `<main>` in a shadow root was
  seen only by Chrome, not by `querySelectorAll`, so the DOM check wrongly reported there. The
  DOM supplement (`check_landmarks_with_page`, page rule `1.3.1/landmark-dom`) is therefore removed
  together with its role derivation. The corpus can now pin the exact count via `occurrences`:
  `landmark_granular` 11, the new case `landmark_unique_one_per_element` 4 (1.6.0: 6),
  `landmark_aside_named_duplicate` 2, `missing_main_landmark` 1 (1.6.0: 2); the new case
  `landmark_main_in_shadow_root` is a true negative. `parity_gaps.html` checks the count in the
  integration test (1 missing `main`, 2 duplicate navigations).

- **Page Health: HTML validation in the JSON canonically English (#406), 2026-09-29:**
  `seo/page_health.rs` wrote German text into the canonical JSON: `html_validator_detail`
  ("HTML5-Validierung lokal via html5ever" (HTML5 validation locally via html5ever)) as well as `check`/`detail` of all `html_issues`
  ("Doppelte IDs" (duplicate IDs), "3 (z.B. …)" (3 (e.g. …)), "HTML5-Parsing-Fehler" (HTML5 parsing error), "+37 weitere" (+37 more), "Leere Ueberschriften" (empty headings)
  etc.). `reconcile_image_alt_count` in `audit/pipeline.rs` even compared against the German
  string "Bilder ohne alt-Attribut" (images without alt attribute). Now following the #406 pattern: `HtmlValidationIssue` carries a
  `kind` (`HtmlValidationKind`) and the raw values in `samples` (example IDs or the first
  parser errors); `html_validation_check_text`/`html_validation_detail_text`/
  `html_validator_executed_text` are the single text source, the analysis calls them with `en=true`,
  the PDF builder (`builder/single/serp.rs`) with the run language. Comparisons run via `kind`.
  The private `run_w3c_html_validation` is now called `run_local_html_validation` (it never calls
  the W3C validator).

  **Changes the JSON output:** `html_issues[].check`/`detail` and `html_validator_detail` are
  English, every entry newly has `kind`, entries with raw values additionally `samples`. The
  German PDF stays word for word identical; the English PDF now shows English text there instead of the
  previous German. Cache entries of the same version with old German text are reduced to `kind` + `samples`
  when read in. Verified with the golden harness against
  `main`: differences only in these fields in the JSON and in the English Typst, German Typst and
  batch outputs unchanged; guard test against umlauts/German words in the EN output.

- **2.5.8/2.5.5 Target Size: "Equivalent" exception for links, 2026-09-29 (#652):** Both
  criteria exempt a target that is too small if the same function is reachable via another control
  on the same page that meets the size. This was missing: on geographia.eu
  (`/atmosphere/de/kapitel/hebel/`) the 39×10 px small attribution line of the module logo
  (`a.logo-main`, `href="/de/"`) counted as a violation, although the footer link "Alle Sphaeren im Ueberblick" (all spheres at a glance)
  (`/de/#modules`, 141×161 px) opens the same start page. Now a link that is too small passes if
  another link with the same target measures at least 24×24 (2.5.5: 44×44), is rendered and visible
  (`checkVisibility` incl. `visibility`/`opacity`, not shifted off to the left of/above the page),
  has pointer events and does not lie in an `inert` or `aria-hidden="true"` subtree.

  The target is the resolved absolute URL. If the link leads to another document, the fragment does
  not count — `/de/` and `/de/#modules` open the same page, the fragment only sets the
  scroll position. If it leads to the current page, it counts: there, jumping to the location is
  the whole function, `#a` and `#b` are different. `href=""`, `href="#"` and `javascript:`
  never count — they name no target. A link that is also too small is not an equivalent, even if
  it passes itself via the spacing exception (deliberately conservative). Buttons are left out,
  their function is not in the markup. The building block `hasEquivalentLink` lives in
  `TARGET_HELPERS_JS`, both rules use it. The rule keeps no pass list, so the
  compensating element is not noted.

  New corpus fixture `target_size_equivalent` (passes: small link with a large equivalent,
  also with a different fragment into another document; violation: only hidden/inert/invisible/
  shifted equivalents, only too-small ones, different fragment into the own page, `href="#"`, without
  equivalent) and an integration test that calls both rules directly.
- **Batch: pool wait time and silently missing pages (#651), 2026-09-29:** A batch over
  geographia.eu (`-c 2 -t 120`, WebGL pages) lost 46 of 312 pages with "Browser pool timeout:
  no page available after 60 seconds". Reproduced with 1.6.0 on 8 chapter pages: 3 of 8
  were missing. Two causes: (1) `BrowserPool::acquire` counts the slot before the new page
  is created; if `new_page` failed (Chrome under the WebGL load, CDP request times out),
  the semaphore permit and the slot were lost permanently — the pool had one page fewer for the rest of the run.
  Both are now returned, and page creation is limited to
  `--timeout`: when reproducing under load, `new_page` stood without an answer, which 1.6.0
  caught only via the 480 s attempt budget. (2) The wait for a free page was
  fixed at 60 s, but a page in progress may take `max(timeout, 30) * 4` seconds (480 s at
  `-t 120`); the waiting page gave up although nothing was wrong with it, and in doing so also used up
  its retry. The wait now follows `--timeout` (page creation plus attempt budget plus 15 s for the reset, 615 s at `-t 120`), lies outside the page's attempt budget
  and consumes no attempt. Pages that still get no browser page are retried one by one after
  the parallel phase, before the report is produced. What still fails
  stays under `errors` as before; new, the report says how many URLs the scores cover: JSON
  `summary.attempted_url_count` (additive, next to `url_count`), PDF a row "Bewertungsbasis" (basis of rating) in the
  audit frame and a note with the unaudited URLs in the status section (de/en),
  terminal a scope line. The score calculation is unchanged. Verified with unit tests
  (derivation of the wait time, only pool timeouts are deferred, report in JSON/PDF/terminal)
  and the Chrome integration test `batch_retries_pool_timeouts_serially` (one slot, two workers,
  1 s wait time: all three pages audited, each reported exactly once). Live with the same 8
  chapter pages and `-c 2 -t 120`: 1.6.0 lost 3 (two pool timeouts after the slot loss, one
  audit timeout after 480 s), the fix audited 8 of 8 — albeit at lower machine load,
  the error path itself was not touched.

- **Skip link and `role="list"`, 2026-09-29 (#642, #644):** (1) `region` (1.3.1) reported
  the skip link `<a href="#main">Aller au contenu</a>` (Skip to content) on all French pages of
  barrierlab.eu, not on the English, German and Spanish ones. The exception depended on the link text:
  the check for `href="#…"` compared with `#`, but Chrome delivers the resolved
  address as `url` (`https://barrierlab.eu/fr/#main`), so only the phrase list applied, and that lacked
  French. Now behaviour counts, as with axe-core's `isSkipLink`: a link to a
  fragment of the same page (address without fragment equal to the root node's `url`) that comes before the
  first ordinary link. The phrase list is dropped; it also exempted every link with
  "springen" (jump) in its text. (2) `redundant-role` (4.1.2) reported `role="list"` on `<ul>`/`<ol>` with
  `list-style: none` (1,142 occurrences on barrierlab.eu). WebKit/VoiceOver discards the list semantics for such
  lists, the role restores them. With computed
  `list-style-type: none` it now goes unmentioned; all other redundant roles are
  still reported. Verified with unit tests, two new cases in the detection corpus
  (`skip_link_language`, `redundant_role_list_style`, each with a counter-example that is still reported) and
  live runs against barrierlab.eu/fr/ and /en/.
- **Contrast for gradient text, #640, 2026-09-29:** `a11y.contrast.weak` reported a
  violation with 1.00:1 for the logo links on geographia.eu (`background-clip: text` with gradient,
  `color: transparent`). That was not a measurement: the transparent text colour was computed against the
  background and yielded the background itself. Style capture now sets
  `foreground-uncertain` if the element or an ancestor carries `background-clip: text` (also
  `-webkit-`) or `color`/`-webkit-text-fill-color` is (nearly) transparent. Such texts
  go into the report as a check note (warning) without `contrast_ratio` evidence and are not resolved by
  pixel sampling, because that needs the CSS text colour. In addition, a set,
  opaque `-webkit-text-fill-color` now counts as the text colour (contrast calculation and pixel sampling),
  since browsers paint the glyphs with it, not with `color` (corpus case `text_fill_color_contrast`:
  light fill colour with dark `color` is a violation, the reverse passes). Verified with unit tests and the
  corpus case `gradient_text_contrast` (gradient text hint, grey body text still a violation);
  live run against geographia.eu/atmosphere/de/kapitel/hebel/: both 1.00:1 violations are
  hints, there were no further contrast findings on the page, neither before nor after
  (accessibility 75 → 86, overall 74 → 79).
- **Heuristic form findings, 2026-09-29 (#643):** Two guesswork rules reported correctly built forms on
  barrierlab.eu. (1) "Input may require format instructions" (3.3.2,
  `label`) accepted an accessible description only if it contained a format word like "Format:"
  or "z.B." (e.g.); the date field of the sample form with `aria-describedby` ("Day of
  travel") failed. In addition, the term "pass" (passport) hit, as a substring, the label "Was
  ist passiert?" (What happened?) of the contact textarea. Now every description counts as an instruction, terms under
  five characters (`pass`, `tel`, `date`, `zip`, `plz`, `ssn`) count only as a whole word, and the
  finding is a check note (`as_warning`) instead of a violation, because the need for a format is only
  guessed from the label text. (2) "Grouped form controls may be missing a fieldset/legend" (1.3.1,
  `form-field-group`) demanded a group for every checkbox as soon as the page had two
  radio/checkbox elements anywhere — on /tasks/bill/ and /tasks/appointment/ this hit the single
  checkbox next to a correctly grouped radio toggle. Checkboxes belong together
  via a shared `name` in the same form; the AX tree does not carry that. The check for
  checkboxes therefore runs as a DOM page rule (`check_checkbox_group_with_page`): only from two
  visible checkboxes of the same name is a `fieldset`/`role=group` ancestor required. Radios
  stay unchanged in the tree (a radio always needs a group). New corpus cases
  `form_heuristics_clean` and `form_heuristics_flagged`.
- **`--per-page-reports`: one file per page, 2026-09-29:** The file names of the single reports
  came only from the host (`casoon-de-<date>-single-report.json`); every page of a website
  overwrote the previous one, a sitemap with three pages left one file. The name now contains
  host and path (`casoon-de-arbeitsweise-…`), with query strings a short stable hash.
  Applies to all formats; single audits without `--per-page-reports` keep their name.
- **Two false alarms on geographia.eu (#638, #639), 2026-09-29:** On
  geographia.eu/atmosphere/de/kapitel/hebel/ 1.6.0 reported two violations of 1.3.1 that are
  none. (1) `th-has-data-cells` (8 occurrences, high): Chrome puts `<thead>` into the
  AX tree as a `rowgroup`, but marks the plain `<tbody>` as ignored; rows and cells still hang below it.
  The rule skipped ignored nodes together with their subtree, thus found not a single data cell
  and reported every column header. Empty cells and `aria-hidden` children played no
  role. Now the search descends through ignored nodes but does not count them itself.
  (2) `landmark-unique` (2 occurrences, medium): the DOM supplement to the landmark rules derived
  the role from the tag and made every `<header>` a `banner`, also the chapter `<header>`
  in `<main>`. According to HTML-AAM (and in Chrome's AX tree), `<header>`/`<footer>` inside `article`,
  `aside`, `main`, `nav`, `section` or the corresponding roles are generic; the DOM supplement
  now follows this. The same held for `<aside>`: inside `article`, `aside`, `nav` or
  `section` (or the roles `article`, `complementary`, `navigation`) it is `complementary` only with an
  accessible name, otherwise generic; `role=region` does not count there, an unnamed `<section>` does
  (that is how Chrome computes it). For this the DOM supplement also takes
  `title` into account as a name. Confirmed in the live run: both findings gone, the other four unchanged,
  accessibility score 75 to 85. New corpus cases `table_headers_tbody_ignored` and
  `landmark_header_in_main`, `landmark_aside_scoping` (true negatives) and
  `landmark_aside_named_duplicate` (true positive); `table_headers_no_data` and
  `landmark_granular` secure the existing true positives.
- **English PDFs without German table labels, 2026-09-29:** With `--lang en`, the tables for security headers ("Vorhanden"/"Fehlt" (present/missing)), SSL ("Gueltiges Zertifikat" (valid certificate), "Laeuft
  ab in … Tage" (expires in … days), "Chain-Laenge" (chain length)), touch targets ("Zu klein" (too small), "Zu eng beieinander" (too close together)) and font ("Kleinste Schrift" (smallest font), "Lesbarer Text" (readable text)) contained German labels, found in an English
  casoon.de report. The labels now follow the run language; a guard test with the
  module data of this run checks the English output for German terms.
- **Cache round trip and deterministic order, 2026-09-29:** While building the
  golden render harness, two errors were noticed. (1) `GraphEntity.properties` was omitted with
  `skip_serializing_if` but carried no `default`: a cached `report.json` with an
  entity without properties could no longer be read ("missing field `properties`"), and the
  CLI silently re-audited. 13 of the 17 frozen cache entries were affected.
  The same gap existed in `BatchReport.errors`, the four lists in `SitemapDiagnostics` and the
  three in `CrawlDiagnostics`; all now carry `default`. New round-trip tests serialise and
  read a complete `AuditReport` and a `BatchReport` with empty lists. (2) Two
  runs over the same input could yield different reports because results came from
  `HashMap`s without complete sorting: on ties, the order and
  (after truncating to 10) the content of `top_recurring_rules` could change; schema conflicts in the
  batch, the interactive categories including the PDF table "Befunde nach Kategorie" (findings by category), the dominant
  rule of the summary, the main role in the action plan, template clusters, duplicate
  content, minification deviations and the sitemap HTTP findings (completion order of the
  requests) had the same gap; the penalty points of the accessibility score were summed in
  `HashMap` order. Every sort now has a final unique key
  (rule ID, entity ID, category, selector, URL), the penalty points are summed in rule ID order.
  Tests pin the order on ties. Verified: three golden render runs
  over all 17 pages, `diff -r` empty.

- **1.6.0, 2026-09-29:** Collective version of the stability and accuracy work since 1.5.1
  (blocked pages, comparison with the rankinglab corpus; details in the entries
  below). Dependencies raised to the versions published in the process: `a11y-rules`,
  `a11y-dom`, `a11y-report`, `accname` 0.12.2 (list rules see through `<slot>`,
  `<ul role=listbox>` is not a list) and `html-conform` 0.3.1 (brings the vnu-closer checks
  from 0.3.0: `img` without `alt`, interactive content in `a`, further parser errors; plus `name`
  next to RDFa `property` on `<meta>`). HTML conformance values can therefore drop.

- **Findings of the report-lint check across 120 rankinglab reports, 2026-09-29:** 8 reports
  failed. (1) The source quality signal "Bedienelemente benannt" (controls named) counted all raw 4.1.2 and
  1.1.1 violations, the lint check all normalised ones; `aria-hidden-focus`
  and `aria-prohibited-attr` now carry 4.1.2 only in normalised form, and both counts
  contradicted each other (5 reports). In substance both were too broad: duplicate IDs or prohibited
  ARIA attributes are not a missing name. `taxonomy::is_missing_name_or_role` is now the one
  definition for blocker, signal and lint; the signal maps raw violations onto the taxonomy like the
  normalisation. (2) `labels::check_link` also checked generic
  link texts in passing (English only, without the context exception from #569) and reported the same links as
  `link_purpose` as a second row of the same taxonomy rule (deutschebahn.com 7 + 5 occurrences,
  `violated_rule_count` one too small, 3 reports). The side check is removed.

- **Aborts in the rankinglab run, 2026-09-29:** Of 122 rankinglab pages, 7 failed; four
  of them ran without problems with 1.3.1. (1) Single audits with full Chrome on macOS sometimes came to a
  complete standstill with the CDP focus emulation: www.deutschebahn.com hung in 3 of 6
  runs (every CDP command ran into the 30 s timeout, the audit exceeded rankinglab's
  12-minute limit), without the emulation 6 of 6 ran in 103-128 s with the same results. The
  emulation was intended for hidden tabs of parallel batch pages and now applies only there
  (`BrowserOptions::focus_emulation`, set at `--concurrency` > 1). (2) If a
  throttled performance call fails, the page keeps loading under the CPU throttling; the
  cleanup commands then waited 30 s each. Now the pipeline first releases the CPU throttling,
  stops loading, each step with 5 s, and skips the remaining profiles if the tab does not
  respond. (3) The pre-check before browser start aborted on a HEAD timeout:
  www.regierung-mv.de does not answer HEAD at all (GET in 0.3 s), www.ing.de needs 9-10 s per
  response with a fixed 10 s wait. Now a GET decides, and the wait is as long as
  `-t`. Not errors on our side: www.unito.it and www.douglas.de block (403), www.tu-berlin.de
  throttled temporarily (429).

- **Report accuracy after the comparison with the rankinglab corpus, 2026-09-29:** Two
  report critiques (gov.uk, sachsen-anhalt.de) against 1.3.1 found errors that distorted verdict and numbers;
  each is verified against the code or the data. (1) The merge of
  desktop and mobile paired every mobile finding with the first desktop finding of the same selector,
  even if that one was already taken: 18 empty lists per view became 35 occurrences. Now one to one. (2) `role="status"`/`alert` without a name counted as a 4.1.2 violation;
  ARIA requires no name for these roles. On gov.uk two empty status regions carried the "failed".
  (3) The combobox rule looked for the option list only in the subtree; `aria-controls` and the
  closed state now count (APG autocomplete on gov.uk). (4) "Bedienelemente ohne
  Namen" (controls without names) counted all 4.1.2 findings from medium up, also duplicate IDs and unnamed dialogs; now only
  missing name/missing role and 2.1.1. The sentence "0 kritische und 0 hohe Befunde, davon 3 …" (0 critical and 0 high findings, of which 3 …)
  no longer names the blockers as a subset. (5) Security card and module table follow the
  corrected wording tier also in colour and table; the source quality cards use
  one scale instead of three; the effort of an action rises with the complexity of the finding
  ("Geringe Komplexitaet" (low complexity) next to "53 Vorkommen deuten auf ein Template-Problem" (53 occurrences point to a template problem)). (6) No
  double full stop in the appendix, German area names for touch targets, and the line below the
  counters separates places for manual checking from the criteria that can only be checked manually. (7) JSON
  stays canonically English (#406): since #633 the texts of the shared rules were in the
  run language in the JSON. The rules now run in English; a German run additionally checks
  in German and stores the texts in `WcagResults::localized_texts` (not serialised), which the
  PDF builders (single and batch) insert. (8) The DOM for the shared rules is the
  flat tree: light DOM nodes assigned via slot hang under their `<slot>`, unassigned
  ones do not appear. Previously they hung next to the shadow content, and every
  `<ul><slot>` of a web component stood there empty (sachsen-anhalt.de). (9) On exit,
  `BrowserManager::close` reported "Failed to close page: Session with given id not found" for the already
  closed audit page (5 of 15 single audits); that is now a debug message.
  Follow-up check on newly generated reports: (10) Blockers count elements, not rule hits (three
  name rules on the same menu element on sachsen-anhalt.de were "3 Bedienelemente" (3 controls)), with
  singular in the sentence. (11) The checklist "Alle Verstoesse" (all violations) also shows the shared texts
  in German. (12) Source quality: card colours by the same thresholds as the wording tier (40
  instead of 60 for red), and "all signals fine" only if every dimension is good (gov.uk: 80+
  overall, substance 70). (13) A landmark action with `aria-label` no longer gets the
  title "Interaktive Elemente (Buttons, Links) verstaendlich benennen" (name interactive elements (buttons, links) clearly).
- **Blocked pages are no longer scored, 2026-09-29:** During the comparison with the rankinglab corpus
  it turned out that www.douglas.de has, since at least 1.3.1, been serving automated browsers an Akamai page
  "Access Denied" with HTTP 403 and about 100 nodes; it was scored 75 and reported as
  complete. With the headless shell, www.hornbach.de delivered a Fastly "Client
  Challenge" with CAPTCHA (355 instead of 12,942 nodes, 85 instead of 56). New module
  `audit::access_block`: HTTP 401/403/407/429, unambiguous challenge titles of the providers or
  challenge resources on a thin page (< 300 elements; Cloudflare also loads
  `/cdn-cgi/challenge-platform/` into regular pages) make the audit of the URL fail with
  `AccessBlocked`: single audit with exit code 3 and reason, in the batch under `errors`, without
  retry. The check runs directly after the first request; challenge pages often change their document
  immediately, hence up to three attempts and a second look at the document status
  later in the pipeline. Verified: douglas.de 3 of 3 blocked, hornbach.de normal with Chrome
  and blocked with headless-shell, batch over 15 comparison pages without false alarm.
- **Batch on macOS with full browser serial against hanging journeys, 2026-09-29:**
  In the reference batch (gov.uk, bundesregierung.de, dm.de, berlin.de, `--full`, three workers), with
  system Chrome 154, journeys ran into the 10 s budget in 4 of 4 runs, twice the
  browser stopped completely (> 5 and > 10 min respectively). All open pages ran into the
  timeout in the same second, independent of the journey. A sample of the browser process showed the
  main thread in AppKit keyboard handling (`performKeyEquivalent` → `SLSObscureCursor`,
  waiting on the WindowServer): even `--headless=new` runs the event loop on macOS.
  Counter-tests: serialising only the journeys across pages did not help (1-10 timeouts); without
  focus emulation no total hang, but tab-walk timeouts again in hidden tabs;
  one worker: 2 of 2 clean (100-120 s instead of 38 s); single audits 6 of 6 clean;
  `chrome-headless-shell`: 5 of 5 clean, but www.hornbach.de delivered it a bot block page
  (355 instead of 12,942 nodes, scored 85 instead of 56). Change: without `--concurrency`, batches
  with journeys on macOS run page by page with the full browser; the run start and `doctor`
  say so. Afterwards: 2 reference batches without `-c` in 97-111 s, one clean, in the other a
  single modal test on dm.de over 10 s (reported as `partial`, no hang). Browser
  selection previously did not find a shell installed via `browser install --headless-shell` at all;
  it is now the fallback after Chrome for Testing when no system browser is present.
  Measurements in `reports/p65/`.
- **JSON and PDF show the same findings in the same order, 2026-09-28:**
  Stability check before 1.6.0 on gov.uk, bundesregierung.de, dm.de and casoon.de, two
  runs each. Scores were stable, the reports were not congruent. (1) `top_actions` and the
  "5 wichtigsten Massnahmen" (5 most important actions) ranked by two logics; now
  `audit::prioritization::action_order` applies to JSON and PDF: severity, within it BFSG obligation (WCAG A/AA)
  first, then leverage, occurrences and `rule_id`. The PDF takes over the order of the
  normalisation, also in the finding matrix; `top_actions` carries `rule_id`. (2) The PDF took the
  title from the explanation, which for rules without their own falls back to the criterion text,
  and merged equal titles: four 4.1.2 rules became "Fehlende Name/Rolle" (missing name/role) with the summed
  count. Titles now come from the taxonomy; only the same item from WCAG
  and SEO is merged, without adding. (3) Ties and the page order in the batch depended on the
  input or completion order. (4) `wcag_criterion` contained the rule identifier for
  `aria-prohibited-attr`, `aria-hidden-focus` and `frame-tested` instead of 4.1.2.
  (5) In the batch, all pages except one were hidden tabs: `requestAnimationFrame` never
  fired, `document.hasFocus()` was false, the tab walk ran into the timeout on 7 of 10 casoon.de pages
  and focus checks measured differently than in the single audit. Every page now runs with
  CDP focus emulation, visible and focused (casoon.de batch 84 s -> about 50 s, no
  timeouts). A separate window per page achieved the same, but made Chrome hang with several
  heavy pages at once (reference batch 215 s instead of 57 s) and was discarded.
  `run_single_audit` closes its page.
  (6) `report-lint` checked the PDF certificate against `overall_score` instead of against the stored
  certificate. (7) The batch JSON listed the same rule once per page in `top_actions`, the batch PDF
  merged but sorted on its own and named the count of one page in "Wirkung" (impact);
  both now use `aggregate_batch_findings` (summed per rule, severity as maximum, derived
  fields from the total), `top_actions` carries `url_count`. New tests: JSON-PDF parity for
  single and batch report, order independent of the input, visible and focused
  parallel pages.

- **`a11y-*` 0.12: German finding texts, `ids/duplicate` in the core, 2026-09-28:** The shared rules
  now run in the report's run language (`run_with_semantics_in`, `--lang de` is the default). Since
  0.11 their `message` texts arrived in English and so showed up in German PDF reports. The WCAG 2.2
  filter for `ids/duplicate` (only duplicates referenced via IDREF, 4.1.2) now lives in `a11y-rules`
  itself and therefore also applies to astro-post-audit and LiveAudit; `IDREF_ATTRS`,
  `referenced_ids`, `duplicate_id_is_referenced` and the recount on the note are gone. The duplicate
  tests in `shared.rs` keep checking the same behaviour, and there is a new test for German texts.
  Verified: 1749 tests green, clippy and fmt clean.
- **Structured data from `web-checks` 0.4, 2026-09-28:** Rule tables, JSON-LD normalisation and
  assessment (`schema_rules.rs`, the normalisation half of `schema.rs`) now live in
  `web_checks::structured_data`, shared with astro-post-audit. What stays here: CDP collection,
  content comparison (`schema_parity`), microdata/RDFa and the report texts (`feature_label`,
  `status_text`, `manual_review_text`). Changes: the breadcrumb name may come from `item.name`
  (tagesschau.de: three fewer false required-field findings); FAQPage requires `acceptedAnswer` per
  question; NewsArticle recommends `publisher`, WebSite `potentialAction` (WebSite quality is now
  computed over 5 instead of 4 properties); `RULESET_VERSION` 2026-09-28. **JSON shape:**
  `rule_assessments[].manual_review` now contains identifiers (`merchant_listing_context`, ...)
  instead of English sentences (#406 pattern; the PDF derives the texts). Structure messages in the
  JSON name the block; identifiers (`jsonld_*`) are unchanged.
- **Meta lengths and OpenGraph from `web-checks` 0.3, 2026-09-28:** Title and description length
  appeared in four places (`meta.rs`, `serp.rs`, `profile.rs` twice) with the same limits 30–60 /
  120–160 and counted **bytes**: every umlaut counted twice, so a German title of 60 characters could
  be "too long". Now `web_checks::meta` (characters, whitespace collapsed as in the browser), shared
  with astro-post-audit. OpenGraph/Twitter: a tag with empty `content` counts as missing (previously
  present), a group consisting only of empty tags as not present; completeness via
  `web_checks::social`. New in `meta_issues`: invalid `twitter:card` value and relative `og:image`
  (per medium), previously checked only in astro-post-audit. web-checks from 0.1 to 0.3 (robots
  unchanged).
- **accname differential with computed style, a11y-core 0.11.3, 2026-09-28:** `accname-diff` now
  computes the page's own names with `accname::name_rendered`: `display`/`visibility` come from a CDP
  `DOMSnapshot` (`fetch_dom_document_with_layout`, `LayoutStyles`, `RenderedCdpDocument`), as do the
  whitespace text nodes that `DOM.getDocument` omits. The shared rules continue to run unchanged via
  `fetch_dom_document`. Corpus (35 pages): one real divergence pattern instead of six (2 occurrences,
  text inserted by script on bundesregierung.de) — "EU-Arktis", "Rechenpower", "abholen*", a
  `display:none` case and a `<br>` case now match Chrome.
- **a11y-core 0.11.2, 2026-09-28:** `accname`, `a11y-dom`, `a11y-report`, `a11y-rules` from 0.11.0
  to 0.11.2. 0.11.1 (space between inline elements after a tag) is skipped and yanked: in the accname
  corpus, name equality fell from 97.7 % to 92.9 % because `<span>` elements turned into blocks via
  CSS were glued together. 0.11.2 reverts that and records `<dt>` (role `term`) as nameless. Corpus
  with 0.11.2: 97.9 %, 6 real divergences as before. The right solution (separator derived from the
  computed `display`) lives in barrierlab.
- **Meta-refresh interstitials in the JSON, 2026-09-28:** That `navigate` followed an interstitial
  (short `<meta http-equiv="refresh">`) or aborted a real redirect via refresh used to appear only in
  the log. Now `accessibility.execution.navigation.meta_refresh` carries `outcome`
  (`interstitial_followed` | `redirect_not_followed`) and `hops` per viewport pass; without a refresh
  the field is absent.
- **Empty heading with its own taxonomy entry, 2026-09-28:** `headings/empty` fell back to
  `a11y.headings.missing` — "Fehlende Ueberschriftenstruktur" (Missing heading structure), severity
  High, a flat 20 points deducted. An empty heading is a single defect, not a missing outline. Now
  `a11y.heading_empty.invalid` (Medium, 2.5–8 points logarithmic like the other heading rules, area
  Headings) with its own explanation DE/EN.
- **Missing navigation and footer landmarks with their own identifier, 2026-09-27:** The two checks
  in `landmarks.rs` did not assign a `rule_id`, so they fell back to their criterion and ended up in
  the 1.3.1 catch-all bucket `a11y.structure.missing` ("Missing semantic structure", a generic
  explanation with a table example). Now `landmark-navigation-present` and
  `landmark-contentinfo-present` with their own taxonomy entries (`a11y.landmark_navigation.missing`,
  `a11y.landmark_contentinfo.missing`, area "Landmarks"), explanation DE/EN and corpus expectations
  (`missing_main_landmark`, `landmarks_and_lists`). Detection and severity unchanged.
- **Own explanations for the shared identifiers, 2026-09-27:** The shared identifiers with their own
  taxonomy entry (heading level, h1 missing/multiple, four list rules, three table rules, invalid
  language code, positive tabindex, limited zoom) had no explanation text of their own.
  `resolve_explanation` fell back to the text of their criterion: an empty list read in the PDF and
  in `fix_guidance` like "Fehlende semantische Struktur" (Missing semantic structure) with a table
  example, an invalid language code like a missing language declaration. Now each has a text (German
  and English, with example code), keyed by the taxonomy identifier. The two texts for `list` and
  `meta-viewport-large` were filed under dead identifiers and were never reachable; they have been
  merged into the entries for `a11y.list_structure.missing` and `a11y.viewport_zoom.restricted` (the
  list text still talked about empty lists, which now have an entry of their own). A test ensures
  that no shared identifier with its own entry falls back to the criterion text any more.
  `headings/empty` has no taxonomy entry of its own and stays with the text for 2.4.6.
- **Dead catalogue entries removed, 2026-09-27:** `LEGACY_WCAG_MAP` still contained `list`,
  `meta-viewport-large` and `table-duplicate-name` — identifiers that no rule assigns any more (the
  first two went to the shared rules `lists/*` and `zoom/viewport-scale-limited`, the third never
  existed as a check). Likewise `a11y.table_structure.invalid`, reachable only via
  `table-duplicate-name`, together with its area mapping and explanation. Verified by searching
  source, tests, explanations and the PDF path. The entries `a11y.list_structure.missing` and
  `a11y.viewport_zoom.restricted` stay, since they carry the shared identifiers.
- **One check for the grouping of radio buttons and checkboxes: `form-field-group`, 2026-09-27:** Two
  separate checks reported the same radio button outside a group: `form_rules::check_grouped_controls`
  under `form-field-multiple-labels` (in axe-core that means "field with multiple labels" — a
  different defect) and `info_relationships` under `radio-group`. The latter checked only the direct
  parent and thus reported wrongly as soon as a `<label>` sat between the radio and the
  `<fieldset>`. Now there is one check with its own identifier `form-field-group`
  (`a11y.form_field_group.missing`): a radio button always needs a group as an ancestor, a checkbox
  only when the page has more than one radio/checkbox field. `check_info_relationships` is thereby
  empty and deleted; the run identifier `definition-list`/`radio-group` is gone,
  `form-field-multiple-labels` is now called `form-field-group` as run and filter identifier (anyone
  who had disabled it in `auditmysite.toml` must update the name). Taxonomy entry
  `a11y.radio_group.missing` removed, explanation and corpus expectations (`forms_and_misc`,
  `landmarks_and_lists`) switched over. Accessibility score: `forms_and_misc` 71 → 72 (the two radio
  buttons were counted twice); across all of that day's taxonomy changes this is the only fixture
  page that moved, axe comparison Spearman −0.536 unchanged.
- **Table and list check deleted from `info_relationships`, 2026-09-27:** The two sub-checks
  reported under the run identifier `definition-list` and duplicated the shared rules. Compared:
  "data table without header cells" (AX role `table` with cells but no `columnheader`/`rowheader`
  among children and grandchildren) is a subset of `tables/header-missing` (every `<table>` or
  `role=table|grid` without `<th>`/header cell role in the whole subtree); "list without list items"
  (AX role `list`, only foreign children) a subset of `lists/invalid-structure` + `lists/empty`. True
  duplicates, hence removed under the `SHARED_RULES` principle, together with the cell check that
  only inflated the passed counter. The only gap: `<menu>` counts as a list for Chrome, but not for
  the shared rule. With the checks go the taxonomy entry `a11y.definition_list.invalid` and the
  explanation that talked about `<dl>` although a description list was never meant. The radio check
  keeps running under the run identifier `radio-group` for now. The corpus case `misc_content_checks`
  now explicitly expects `tables/header-missing` for its table without header cells.
- **`valid-lang` deleted: the invalid language code was reported twice, 2026-09-27:** The own rule
  `valid-lang` (`language_extended.rs`) read the same `<html lang>` as the shared
  `document/lang-invalid` and reported the same defect a second time. Following the `SHARED_RULES`
  principle (the own counterpart goes as soon as the shared identifier is adopted), the own check is
  removed; the shared one covers it (primary tag of 2-3 letters, plus the subtags, and also an empty
  `lang`). `html-xml-lang-mismatch` stays its own. With it go the taxonomy entry
  `a11y.language_valid.invalid` and its mappings; the corpus expectation `invalid_lang_code` now
  expects `document/lang-invalid`.
- **accname differential: name source from Chrome's detail, `image` = `img`, 2026-09-27:** The source
  axis was blind: the AX extractor lumps `aria-label`, `alt` and `value` together as `Attribute` and
  `aria-labelledby` like `<label>` as `RelatedElement`, and the differential reported all of these
  names as `unknown`. The extractor now additionally reads the winning source from `name.sources`
  (first one with a value; identifier from `attribute`/`nativeSource`), delivers it via
  `extract_ax_tree_with_name_sources` alongside the tree, and `CdpDocument::with_chrome_name_sources`
  passes it on to `names_by_source` and `Semantics::name_source`. The coarse `NameSource` from
  `a11y-perception`, read by `text_alternatives`, `accessible_name`, `svg_rules` and `instructions`,
  stays unchanged (test). Role comparison: Chrome `image` against `accname` `img` counts as equal.
  Corpus re-measured (35 pages): `unknown` 56 → 0 (`aria-label` 50, `alt` 3, `label` 2,
  `aria-labelledby` 1), role divergences 215 → 25. Measurement side only, `accname` and
  `a11y-perception` untouched.
- **Reference set rated: seven live pages with bands, 2026-09-27:** Bands from three sources per
  page — current score, inspection in the browser (skip link, outline, alternative texts, form
  labels, reflow at 320 px) and the published accessibility statement —, confirmed by the user:
  gov.uk 75–92, gov.ie 75–92, casoon.de 90–100, mit.edu 70–89, bundesregierung.de 35–60, dm.de
  15–40, berlin.de 15–40, basf.com 0–25. berlin.de initially fluctuated (desktop 71 or 27): the 71
  came from a bot/load interstitial that the tool rated as the home page — fixed by following a
  meta refresh to the same URL; the real page sits stably at 27.

- **1.1.1: an icon in a named link is not missing alternative text, 2026-09-27:** While checking the
  reference pages, auditmysite reported 7 "Image is missing alternative text" (High, Level A) on
  www.mit.edu — all inline `<svg>` icons without `role="img"` inside links with a name of their own
  ("MIT@twitter", "open search", the logo in "Massachusetts Institute of Technology"). The link
  carries the alternative; the icon is decorative, 1.1.1 is met. axe skips this case (`svg-img-alt`
  applies only to `role="img"`). Because of the new Level A cap, even a single such false positive
  would have limited the page to 89. `text_alternatives` now exempts nameless graphics *without* a
  `url` property (so not `<img>`) in named links/buttons; an `<img>` without `alt` remains a
  violation everywhere (F65). Known blur: an explicit `role="img"` looks the same in the tree.
  mit.edu 68 → 78.

- **aria-prohibited-attr names the role instead of the tag, 2026-09-27:** For `div`/`span` without
  `role` the rule reported "prohibited on role 'span'". In this case the rule covers only these two
  tags, whose implicit role according to HTML-AAM is `generic` — the message now says `generic`.
  Found on the 30 `aria-label` spans on casoon.de `/leistungskatalog/`. In
  `docs/accname-differential.md`, `text_transform` is now listed as an expected divergence (decision
  2026-09-27: `accname` stays with the DOM text).

- **Menu buttons only with the menu journey, 2026-09-27:** A button with `aria-haspopup="menu"` was
  checked twice — as a menu (`MenuOpen`) and additionally by the accordion detector as an expandable
  element (`AccordionToggle`, i.e. with the disclosure journey). According to the ARIA Authoring
  Practices this is the "Menu Button" pattern, not "Disclosure"; the disclosure journey judged it by
  expectations it does not have to meet. The accordion detector no longer offers a trigger that the
  disclosure/menu detector has already offered; the detection in the report text stays.

- **Refresh interstitials (bot/load check) were audited as the page, 2026-09-27:** berlin.de serves
  some of the fresh browser sessions, instead of the home page, a 453-byte interstitial ("Einen
  Augenblick bitte / Just a moment please", Varnish, `<meta http-equiv="refresh" content="2;
  url=/~~delay/">`, `<dialog open>`) that leads back to the real page after 2 s. HTTP 200,
  `readyState` `complete`, no mutations — done as far as `navigate` and the stability check are
  concerned. The desktop pass audited this interstitial in 8 of 12 runs (5 AX nodes, accessibility 71
  instead of 27, findings for a missing main/banner landmark, bypass, `dialog-name`,
  `2.2.1/meta-refresh`); the mobile pass then saw the real page. `navigate` now follows a meta
  refresh with a delay of at most 5 s (max. 3 hops) until a new document is interactive. If the
  chain ends on the requested URL again, it was an interstitial; if it leads elsewhere (a real timed
  redirect), the requested page is reloaded and its refresh is aborted after loading with
  `window.stop()` — it is thus audited as delivered, including `2.2.1/meta-refresh`, instead of
  navigating away mid-analysis depending on the delay, as before. Re-measured: berlin.de 12 runs
  after the fix, desktop/mobile always 27/27 (interstitial resolved in 7 of them);
  bundesregierung.de 4 runs unchanged 41/41; local page with `1; url=b.html` stays on a.html with a
  meta-refresh finding. Unit tests for parsing the delay and the URL comparison.
- **accname differential: corpus run and `text_transform` class, 2026-09-27:** `accname-diff` was
  limited to one URL; a corpus could only be tallied by hand. The command now takes several URLs and
  `--url-file` and, from two pages on, writes an `AccnameCorpus`: totals per form and name source,
  cross table of both axes, recurring patterns with occurrences and page count, the individual
  results and pages that did not load. A page that fails to load no longer aborts the run.

  New form `text_transform`: names that differ only in capitalisation are matched against the
  computed style of the element and its descendants (`DOM.resolveNode` + `getComputedStyle`, only
  for these candidates). If one of them has `text-transform` other than `none`, the case is counted
  separately — in the first finding this was the most frequent cause and would have dominated every
  corpus. Measurement on this tool's page, `accname` remains untouched. Without style (`compare`) it
  stays `mismatch`. Unit tests for classification, aggregate and CLI parsing.
- **Stability check always reported "budget exhausted": every audit counted as partial,
  2026-09-27:** `wait_for_page_stability` evaluates a promise that resolves to an object `{ status,
  waited_ms, mutation_count }` — built without `return_by_value`. CDP therefore returned only an
  object reference, `value` stayed empty, and every call fell back to the default values:
  `budget_exhausted`, full budget, 0 mutations, whatever the page did. The waiting was done
  correctly; only what was reported about it was wrong. Consequences: `audit_quality` was practically
  always `partial` with `page_stability_budget_exhausted:2` (so on every casoon.de run), and the
  journeys could not see how long a click was actually waited for. Found while implementing the
  journey work. Re-measured: casoon.de and bundesregierung.de `complete`; waiting times in the trace
  are now real (e.g. "stable after 201 ms, 0 mutations").

- **Disclosure findings with an uncertainty note instead of a longer wait, 2026-09-27:** The wait
  after a click ends after 200 ms without a DOM change; a later reaction is only captured if another
  change happens to extend the window — measured as the cause of the roughly 8 % run-to-run
  fluctuation. Waiting longer cost +57 % disclosure time for a changed verdict in about 66. Instead,
  the trace now carries status, wait time and mutations per click, and a finding from a click whose
  wait ended before the budget gets `uncertainty: {kind: "late_reaction_possible", waited_ms}`.
  Following the #406 pattern: canonical `kind` + raw value, `finding_uncertainty_text` as the only
  text source, attached to the finding in the PDF, localised; uncertain and certain findings are not
  merged there.

- **Native `<details>`: one journey run per trigger, no more false positives, 2026-09-27:** A new
  Chrome fixture test (`natives_details_erreicht_die_disclosure_journey`) found three gaps. First,
  `patterns::accordion` additionally offered every `<summary>` and every ARIA button with
  `aria-expanded` outside navigation/banner as an `AccordionToggle` — which starts the same
  `disclosure_journey` as the already existing `DisclosureToggle` candidate. Every such trigger was
  clicked through twice, its findings reported twice. The accordion now skips triggers that already
  have a `DisclosureToggle` candidate; the number of disclosure journeys per page drops accordingly.
  Second, `<summary>` in `<details name>` carries the role `DisclosureTriangleGrouped`: for that the
  accordion reported `accordion-trigger-not-button` (Medium, 4.1.2) on every exclusive accordion, and
  the journey determined no controlled region. Third, an open native `<summary>` got
  `accordion-no-controls` (Low) — a native element needs no `aria-controls`; the check now applies
  only to the role `button`. The runtime fluctuation of the disclosure verdicts was measured on
  fixtures and documented (cause: the settling window ends 200 ms after the click, later reactions
  are only captured by chance); the timing behaviour stays unchanged because the remedy costs +57 %
  journey time on eight live pages.
- **Shared rules get their own taxonomy entries, 2026-09-27:** On `landmarks_and_lists` there was a
  finding "Missing semantic structure" with three unrelated locations — missing navigation landmark,
  empty list, list item outside a list. Cause: `wcag_group_key` groups by rule identifier only if
  `LEGACY_WCAG_MAP` knows it, otherwise by WCAG criterion. None of the 1.3.1 identifiers from
  `a11y-rules` (four list, three table, three heading rules) was listed there, all fell into
  `a11y.structure.missing`. Likewise `document/lang-invalid` ran under "Missing language
  declaration", and `keyboard/positive-tabindex` shared `a11y.focus_order.weak` with the 2.4.3 group,
  so two findings could carry the same identifier. Now each of these identifiers has an entry of its
  own (title German and English, same criterion, score area); `lists/item-outside-list` and
  `zoom/viewport-scale-limited` use the entries of the superseded own rules
  (`a11y.list_structure.missing`, `a11y.viewport_zoom.restricted`). A test ensures that no shared
  identifier lands in the 1.3.1 catch-all bucket any more.

  The radio button outside a group carried the axe identifier `definition-list`: the own rule
  `info_relationships` stamped all three sub-checks (table, list, radio group) with the identifier of
  its run. The radio check now reports as `radio-group` (`a11y.radio_group.missing`); the corpus
  expectation is updated.

  The accessibility score stays the same: `AccessibilityScorer` groups by `Violation.rule`, i.e. the
  criterion, not the taxonomy. The deduction of the new entries is that of the catch-all entry under
  which the identifier ran before; only the two reused entries bring their earlier own deduction
  with them. What can move is the per-area breakdown (`accessibility_score_breakdown`), by one to two
  points — heading findings now count under "Heading structure" instead of "Semantics". axe
  comparison: Spearman −0.536 unchanged, no page moved.
- **Unique selectors in all own page rules, 2026-09-27:** Quite a few JS rules built their selector
  as `tag` or `tag#id` (partly with a class). Locations are merged by (rule, selector) — so all
  id-less elements of the same tag collapsed into *one* location ("a", "button", "div"): undercounted
  and not findable. Affected were 2.4.11/2.4.12 Focus Not Obscured (including the focus-walk check),
  1.3.2 Meaningful Sequence, 3.3.8 Accessible Authentication (field, form, CAPTCHA widget), 2.2.2
  Pause/Stop/Hide, 2.5.3 Label in Name, 1.4.11 Non-text Contrast (CSS), 1.4.13 Content on Hover,
  2.1.1 click handler, 1.4.1 Use of Color and 3.1.2 Language of Parts (single-level `nth-of-type`).
  All now use the existing `__amsCssSelector` (`js_helpers::CSS_SELECTOR_JS`); 2.5.5/2.5.8 also give
  up their own copy (`targetSelector`) for it. Elements with an id keep the form `tag#id`, so the
  corpus expectations remain valid. The child identifiers in the Meaningful Sequence message stay
  short — they are text, not a location. Corpus, score bands and integration tests green; axe
  comparison unchanged (Spearman −0.536), no page moved.

- **2.5.8 Target Size: spacing and inline exception, 2026-09-26 (axe comparison):**
  `target_size_minimum` measured only the rectangle of a target. But WCAG 2.5.8 exempts two cases: a
  target that is too small, around whose centre a 24 px circle intersects no other target (and no
  circle of another too-small target), and a link in running text. Both were missing — although the
  rule's doc comment mentioned the inline exception. Every standalone flat button and every link in a
  sentence counted as a violation; so did "Send Message" and the skip link on `perfect.html`. Now
  implemented; 2.5.5 (AAA) gets only the inline exception, because it has no spacing exception. The
  inline building block is shared (`INLINE_TARGET_JS`).

  The corpus fixture `keyboard_and_targets` expected a single 16 px button as a violation — the
  ground truth reproduced the bug. It now carries two adjacent 16 px buttons (violation), a single
  one with spacing and a link in running text (both pass, the link also for 2.5.5). axe comparison:
  Spearman −0.492 → −0.536.

  Two follow-up fixes, found by comparing with axe on `perfect.html`: the inline check counts only
  the inline content of its own line without other targets — with the whole block text, a skip link
  directly under `<body>` counted as "in running text". And the circle must not intersect the
  *rectangle* of any other target, even if that one is itself too small. In addition, unique
  selectors (`tag#id` or `nth-of-type` path): with the bare tag name, all links on a page collapsed
  into *one* location "a". `perfect.html` now reports the same four closely spaced links as axe
  (skip link, Home, About Us, Contact) and leaves out the free-standing button.

- **Shared rules report English in the JSON: a11y family to 0.11.0, 2026-09-26:** When checking the
  axe divergence of `landmarks_and_lists`, German sentences in the JSON stood out ("Der
  Listeneintrag steht außerhalb einer Liste." — The list item is outside a list.). `run_shared_rules`
  takes the message from `a11y-rules` unchanged, and up to 0.10 the crate phrased in German —
  affecting all 16 integrated shared rules, a violation of the rule "JSON is canonically English"
  (#406). `a11y-rules` 0.11.0 phrases finding texts and hints in English and was already on
  crates.io; `a11y-rules`, `a11y-report`, `a11y-dom` and `accname` are raised from 0.10.0 to 0.11.0,
  without API changes. The German PDF thus shows the location messages in English — as those of the
  own rules always have; titles and explanations stay localised. The too coarse classification of the
  shared list rules in the taxonomy is still to be addressed.

- **Player controls of `<video controls>` are not author content, 2026-09-26 (axe comparison):** The
  biggest divergence in the new axe comparison was `media_and_motion`: auditmysite 49 (Critical), axe
  nothing. The Critical came from Chrome's own player UI: with `controls`, the browser puts play,
  mute and fullscreen buttons and a timeline as a `slider` without `valuenow` from its shadow DOM
  into the accessibility tree. The ARIA rules reported the timeline as "Required ARIA attribute
  missing" (Critical) and "Slider is missing accessible value" (High) — on every page with a video,
  capped at 49, and not fixable by any author. Fixed in `a11y-perception` 0.2.1: `AXTree::iter()`
  omits everything below `Video`/`Audio`, for all WCAG rules. `media_and_motion` 49 → 85, the three
  built-in defects are still found. The fixture sat in the Critical class of the score bands — only
  because of the false positive; it has been removed there. New axe baseline: Spearman −0.462 →
  −0.492, the only page that moved is this one.

- **Live reference set and axe comparison for score calibration, 2026-09-26:** *Reference set:*
  `tests/fixtures/reference_sites.json` lists eight public pages; each gets a band from a manual
  review (not from the tool), with a date. `tests/reference_sites_test.rs` (Chrome + network)
  reports every page outside its band and runs as the new release step 0 before the tag
  (`CLAUDE.md`). Pages without a review are skipped.

  *axe comparison:* `scripts/axe-divergence.py` runs auditmysite and axe-core (WCAG 2.2 A/AA tags)
  over all 45 fixture pages and records score and axe violations by severity per page. The metric is
  the rank correlation (Spearman) between score and weighted axe load; the baseline is in
  `tests/fixtures/axe_divergence_baseline.json`: **−0.46** — same direction, moderately strong. A
  later run reports the change in correlation and every page whose score moves by ≥ 5 although axe
  sees the same. Agreement is not the goal; a scoring rework that clearly worsens the correlation
  should stand out.

- **Score bands and a new cap: a Level A violation is not VERY GOOD, 2026-09-26 (step 1b):** For the
  first time, an expectation that does not come from the scorer itself stands beside it. Decided in
  certificate terms: a practically clean page ≥ 95; exactly one High violation at Level A, otherwise
  (almost) clean, 75–89; at least one Critical ≤ 49. `tests/score_calibration_test.rs` checks this
  on 21 fixtures against real Chrome, at the default level AA like a user audit.

  The middle band failed with the previous scorer: a missing alt text on an otherwise clean page gave
  92, i.e. VERY GOOD — for a page that does not meet Level A and is therefore not conformant. New cap
  alongside the existing ones (Critical → 49, ≥ 5 Critical/High → 92): **a High or Critical violation
  of a Level A criterion limits to 89**. The same cap applies in the JSON per-area breakdown so that
  it does not diverge from the score again. Affected: every page with a Level A High finding and an
  otherwise high score — it drops from VERY GOOD to GOOD. `test_score_with_errors` pinned exactly the
  old behaviour (95, grade A) and has been adjusted; the invariant generator now mixes Level A and
  AA so that the 92 cap is checked as well.

- **Invariants for the accessibility score, 2026-09-26 (step 1a):** The score had no check that did
  not depend on its own constants. Five properties that no tuning may break now run as tests over
  2000 deterministically generated finding sets each (catalogued and unknown rules mixed): score in
  0–100; one more finding never raises it; a higher severity never raises it; the caps (Critical →
  ≤ 49, ≥ 5 Critical/High → ≤ 92) hold for every set; only the empty set yields 100. All hold. The
  only observation: penalties are summed in `HashMap` order, so the same set may differ in the sixth
  decimal place — irrelevant for the integer score as displayed, the tests tolerate 1e-3. What a
  page *should achieve* is not stated by the invariants; the score bands follow for that.
- **Two detection errors, found while calibrating the scores, 2026-09-26:** Before fixing score
  bands, the tool ran over all 43 fixtures. Two findings were wrong.

  *1.3.5 Identify Input Purpose read the wrong source.* The rule checked the AX property
  `autocomplete` — that is `aria-autocomplete` (`list`/`inline`/`both`); Chrome does not put the HTML
  attribute `autocomplete` in the tree at all. Consequence: **every** correctly marked-up personal
  data field counted as "lacks autocomplete attribute" (Medium), already visible on `perfect.html`
  with `autocomplete="name"`/`"email"`; a combobox with `aria-autocomplete="list"` would have gotten
  "Invalid autocomplete value". The rule now runs as a DOM page rule
  (`check_input_purpose_with_page`, like 1.3.6 and 3.3.7) and reads the attribute; the labelling
  approximates the accessible name (aria-label → aria-labelledby → label → title → placeholder). The
  classification is a pure function with unit tests.

  *2.4.7 Focus Visible reported a page without focusable elements as a violation (High).* 2.4.7
  applies to "any keyboard operable user interface" — if there is none, the criterion is not
  applicable; axe-core reports nothing there. #568 had deliberately made the finding reachable
  again, the corpus expected it. It stays, but as a review hint (Low) without effect on the score;
  `no_focus_targets` now expects `needs_review`. 21 of 43 fixtures were affected.

  Detection corpus green against real Chrome. `perfect.html`: 97 → 99 (remaining: 2.5.8, a 21 px
  high button — correct under WCAG 2.2).

- **AAA list in the PDF counted honestly: 11 instead of 17, 2026-09-26:** The same overstatement as
  with the A/AA quota (30/55): the PDF listed 17 AAA criteria as "automatically checked", but six of
  them cannot report a violation — 1.2.8, 2.2.3, 2.2.4, 2.2.5 and 3.1.3 deliver only an `untested`
  entry, 2.4.12 only a review hint. They are now in `HINT_ONLY_CRITERIA`; the list shows 41 criteria
  (30 A/AA + 11 AAA). Unlike the A/AA cases they do not go on the list of criteria to be checked
  manually, because that list, like the quota, refers to A/AA. The guard test now distinguishes
  this. Verified via `--debug-typ` (DE/EN).
- **Journey leftovers: skip link target, scoped content, read errors, 2026-09-26:** Three defects
  left open in the journey rework.

  *Skip link.* The journey only checked whether the focus had left `body`. That reported a
  non-focusable `<main>` as a High finding although the browser shifts the starting point of tab
  navigation and the next Tab lands in the content — and let a focus pass that jumped anywhere but
  to the target. Now the link is focused first (as with keyboard operation), the target is resolved
  from the `href` fragment and it is checked whether the focus is inside or behind it; if it stays
  on the link or `body`, a Tab decides. A fragment without an element is a finding. Five fixtures,
  tested against real Chrome.

  *Disclosure.* "Content appeared" was page-wide — a lazily loaded image at the end of the page
  turned "expanded belongs here, nothing to read" into a pass. If the controlled region is known
  (`aria-controls`, or the `<details>` around a `<summary>`), only what appears or disappears in it
  counts. In the field this reaches less far than hoped: on 12 pages with 138 disclosure journeys the
  region could be determined only 8 times. The trace notes `scoped` or `page_wide` per journey.

  *quantity_stepper.* If a CDP call failed, `focus_and_confirm`/`is_native_input` became `false` —
  and from that a High finding "not keyboard operable". Unreadable values are now recorded as
  `not_observable` in the trace, without a finding.

- **Journey budget 15 s, hard limit per journey, 2026-09-25:** The height of the budget was open.
  Measured on 48 public home pages without a limit and then recomputed per budget height (deadline
  checked before each start, as in the code): with **5 s**, 22 % of the pages hit the limit and only
  **55 %** of the findings were captured — the tab walk alone consistently takes about 2 s. **15 s**
  captures 92 % (10 s: 83 %, 20 s: 98 %) and costs +1 s per page on average, because the budget only
  bites on heavy pages; the median of the interactive phase is 2.8 s.

  The same run showed that a single journey can run indefinitely: the deadline is only checked
  before the start, and if the page hangs, every CDP call waits out its 30 s — on www.dm.de an
  accordion journey took 180 s. Every journey (tab walk, pattern journeys, SPA navigation) now runs
  under a limit of 10 s and is otherwise recorded as failed with `journey_timeout`; the slowest
  journey without a hang was 5.6 s (modal).

  Fixed on the side: when the budget ran out, the remaining candidates were entered as
  `budget_exhausted` from the *unsorted* list, although since sorting by confidence the sorted one is
  worked through — the report named the wrong journeys as skipped.

  With the higher budget, the modal journeys on www.dm.de came up for the first time — and the audit
  then took 243 s instead of 73 s, all throttling profiles of the performance measurement ran into
  timeouts. Cause: Escape closes none of the three info dialogs, the journeys leave them open, and
  the following phases on the same tab configure throttling and cache *before* their own reload —
  under CPU throttling the renderer stopped responding with three open dialogs. A JavaScript dialog
  it was not (checked). After the interactive phase, the tab is now set to `about:blank`; all later
  phases navigate to the URL themselves anyway. dm.de: 87 s, performance `completed`, all three
  profiles measured.
- **Announcement structure to a11y-perception, words stay here, 2026-09-25:** `announcer.rs` did two
  things in one go: decided *what* an announcement consists of, and at once said *what that is
  called in German*. The first half is not a property of auditmysite -- that the level of a heading
  belongs in the role, that `expanded=false` has to say "collapsed" and `required=false` nothing,
  that "focusable" adds something only where the role does not already give it away: that holds in
  every language and for every surface. It now lives in `a11y-perception 0.2.0` as `announce() ->
  Announcement` (name, role, states as named parts, `AnnouncedRole`/`AnnouncedState`). What stays
  here is the mapping to the locale keys and the separator -- 16 roles, 13 states, the same keys as
  before, no change to `locales/`. The renderer has shrunk from 118 to 89 lines of code; the gain is
  less the number than what is no longer decided here.

  One behavioural difference, small and deliberate: `level` is now carried as `u8` instead of a raw
  string value. A heading with a non-numeric level is thus announced as "Ueberschrift" (heading)
  instead of "Ueberschrift Ebene <muell>" (heading level <garbage>). Chrome delivers `level` as a
  number, so the case is theoretical.

  New guard `every_announced_role_and_state_has_a_german_word`: the compiler enforces that every
  variant is in the match, but not that the i18n key exists -- `I18n::t` returns a missing key
  unchanged, and "Suche, sr-role-textbox" would otherwise have passed. Verified: `cargo test --lib
  screen_reader` (44 ok), `cargo clippy --all-targets -D warnings`, `cargo fmt`. The four existing
  snapshot tests are green **unchanged** -- that is the evidence that the announcements are
  character for character the same.

- **Pinned toolchain and a slimmer crate package, 2026-09-25:** Locally, in CI and in release, we
  built with whatever `stable` was current. Now `rust-toolchain.toml` pins Rust **1.98.1** (the
  current stable version, one patch version above the 1.98.0 originally planned) with `clippy` and
  `rustfmt`. There is deliberately no MSRV promise, `Cargo.toml` gets no `rust-version`. The README
  badge `rust-1.75+` claimed a lower bound that was never verified and now names the pinned version.

  `dtolnay/rust-toolchain` does not read the file. It installs what is in the ref or in the required
  input `toolchain` and sets it via `rustup default`. The file would have overridden that, but
  `components`/`targets` would have landed on the wrong toolchain, and the version would have had to
  be maintained twice. The workflows therefore call `rustup toolchain install` without an argument;
  that installs version, profile and components from the file. The release build adds its matrix
  target via `rustup target add`. `CARGO_INCREMENTAL=0`, which the action used to set, is now in the
  `env` of both workflows.

  `Cargo.toml` has an `exclude` list: `.claude/`, `.github/`, `.githooks/`, `docs/`, `scripts/`,
  `tests/`, the two additional documents and `rust-toolchain.toml`. The package shrinks from 623 to
  386 files. The patterns are anchored with `/`, because the list as originally drafted, with a bare
  `tests/`, would silently have removed `src/output/tests/` from the package too. Every path that
  `include_str!`/`include_bytes!` references is in the package; the only exception is a test fixture
  under `#[cfg(test)]`. `cargo package --locked` runs through without `--allow-dirty`, and the
  binary from the unpacked package installs and answers to `--help`. The first CI run with the
  pinned toolchain has not happened yet.

- **Browser tests run in the release gate for the tagged commit, 2026-09-25:**
  `release.yml` calls `ci.yml` as a reusable workflow and only builds once it is green. The
  comment in `ci.yml` claimed that `browser-smoke` and `coverage` did not run in that case because
  their `if:` only admits `push`/`pull_request`. That was wrong: in a called workflow the `github`
  context belongs to the caller, so `event_name` is the tag's `push`, never `workflow_call`. Both
  jobs did run in the release call, only by accident, and a condition aimed at `workflow_call`
  would not have distinguished anything.

  Now it is stated explicitly: `browser-smoke` has no job condition any more and runs on every
  call, including for the tagged SHA. `coverage` is a measurement and not a gate and is skipped for
  tags via `!startsWith(github.ref, 'refs/tags/')`. The comments in `ci.yml` and `release.yml`
  describe the actual behavior. A contract test in `tests/release_contract_tests.rs` checks both.
  Counter-check: with the old condition on `browser-smoke` or without the tag condition on
  `coverage` it fails. There has been no real tag run for this yet; that is pending for the next
  release.
- **Best-effort cleanup errors no longer invisible, 2026-09-25:** `cargo judge errors` reported six
  production sites in four files that silently discarded an I/O result. None of them may abort the
  audit, but two can leave files behind unnoticed. Deleting the downloaded browser archive after
  unpacking and deleting the `.partial` file after a failed atomic write now log a `warn!` with
  the path; the installation result and the original write error stay unchanged. The flush of the
  "Checking for sitemap..." status line logs at debug level, and only if stdout is a terminal — a
  pipe or closed output is a valid environment, not an error. The cleanup of PDF screenshots and
  evidence crops goes through a helper, `remove_temp_file`, that counts `NotFound` as success (most
  evidence slots are never written) and reports other errors at debug level so that a normal run
  stays quiet.

  Checked with a unit test for the helper (a missing file stays silent; `remove_file` on a
  directory yields an error that is not `NotFound`), the existing atomic-write test, clippy for
  both feature sets, `cargo fmt --check` and `cargo test --no-default-features`.
  `cargo judge errors` drops from 17 to 11 `swallowed-result` findings; the rest lives in tests and
  stays on purpose, as do deliberately lossy parse conversions.
- **Desktop and mobile score from the same rules, 2026-09-25:** It was assumed that the second
  viewport pass never moves the accessibility score. Measured on 35 public home pages, that is not
  the case: for **24 of 35** desktop and mobile differ, in part strongly (berlin.de 74/26,
  kit.edu 46/26). A repeat run over eight of them was exactly the same on six — the signal is
  mostly real: pages deliver different markup per breakpoint, and the differing findings come from
  ordinary structure rules (`interactive_name`, `alt_text`, landmarks, ARIA).

  Part of the gap was an artifact, though: `html_content_model` and `reflow` only run in the mobile
  pass and only flowed into the mobile score. Both judge a viewport-independent property of the
  page (raw markup; wrapping at 320 CSS pixels). The desktop score is now computed from the desktop
  findings plus the findings of these two rules; execution notes and the viewport labelling of the
  findings stay unchanged, because the rules still only ran on mobile. Re-measured: where
  `html_content_model` was the only difference, the scores are now equal (commerzbank.de 36/36,
  gov.ie 73/73, hamburg.de 20/20); berlin.de keeps its real gap.
- **Automation rate counted honestly: 30/55 instead of 39/55, 2026-09-25:** A criterion counted as
  automated whenever the rule catalog has an entry for it — even if the rule can only produce
  review hints or an `untested` entry. A criterion that the tool cannot report as violated is not
  checked automatically. This affects nine criteria: 1.2.2, 1.3.2, 2.1.2, 2.2.2, 2.4.11, 2.5.1,
  2.5.2, 2.5.4 and 3.3.7. They are in `HINT_ONLY_CRITERIA` (`src/wcag/coverage.rs`), drop out of
  `automated_criteria()` and now count as manual review; 1.3.2, 2.1.2, 2.4.11 and 3.3.7 are newly
  in the manual list for that. The rules keep running unchanged, their hints stay in the report.
  New numbers: 30 automated, 21 manual, 4 neither (of 55). Moved along with it: the principle
  coverage (informational, no score influence) and the EN 301 549 annex, which now lists these
  clauses as "manuelle Pruefung" (manual review) instead of "automatisch ohne Befund" (automatic,
  no finding). A guard test ensures that every hint-only criterion has a catalog rule, does not
  count as automated and is in the manual list. `docs/PARITY_CONTRACT.jsonc`,
  `docs/PARITY_CONTRACT.md` (still on WCAG 2.1, 50/36/10) and the README were updated. Checked: a
  `--debug-typ` run against a local fixture (scope "30 von ca. 55" (30 of approx. 55), manual cloud
  21), `cargo test --lib`, `parity_contract`, `release_contract_tests`, clippy with all features.

- **Criterion findings on mere presence of a technique removed, 2026-09-25:** Four rules reported a
  WCAG criterion as soon as a technique appeared on the page that *could* be related to the
  criterion — without an observed defect. **2.2.3 No Timing** fired on every
  `setTimeout`/`setInterval` in an inline script (debounce, carousel tick, analytics),
  **2.2.4 Interruptions** on every `role="alertdialog"`, **2.5.1 Pointer Gestures** on strings such
  as `ontouchstart` (even pure feature detection) and **2.5.4 Motion Actuation** on
  `deviceorientation`/`devicemotion`/`shake` in the script text — the latter even as a violation
  instead of a review hint. All four now only report the `untested` entry, whose text already says
  what is to be checked manually. For **2.5.2 Pointer Cancellation** the second branch (any
  `addEventListener('mousedown'|'touchstart')` in the inline script) is dropped; the hint for
  `onmousedown`/`ontouchstart` directly on a control stays. There is no place for criterion-less
  informational signals, so the detection is removed without replacement. The review hints flowed
  into the `review` count and from there into screen reader and risk aggregation. Checked:
  `cargo test --lib`, clippy with all features, detection corpus with real Chrome green.

- **Audit target switched to WCAG 2.2 AA, 2026-09-23:** The tool checked against WCAG 2.1 AA and
  deliberately left the 2.2 criteria it had long been checking out of the rate. The reference is
  now WCAG 2.2 AA: 55 A/AA criteria (2.1's 50, minus the removed 4.1.1, plus the six new ones), 38
  of them automated. The WCAG 2.1 view, which the BFSG invokes via EN 301 549 V3.2.1, is kept as an
  annex from the same run; criteria that only 2.2 brought are still marked as such there, because
  the standard does not cover them.

  **4.1.1 Parsing** is removed in WCAG 2.2 and is no longer carried as a criterion. The two checks
  that hung on it now report 4.1.2 — the same mapping axe-core uses for `duplicate-id-aria`. The
  generic duplicate check (`ids/duplicate`, iframe variant) now only reports what is also a
  violation: a duplicated ID that an IDREF points to (`for`, `headers`, `aria-labelledby` and the
  other ARIA references). Then the relationship can no longer be resolved unambiguously. A
  duplicate that nobody points to no longer violates a criterion since 4.1.1 was dropped and
  produces no finding. axe-core removed its equivalents (`duplicate-id`, `duplicate-id-active`) for
  the same reason.

  **Scores and rates shift as a result** and are not directly comparable with older reports — a
  change of reference, not a regression. Checked on www.casoon.de (single, PDF via `--debug-typ`
  and JSON): rate 38/55, `wcag_coverage.level` now "WCAG 2.2 AA", `report-lint` without findings.
  On proofreading a counting error turned up: the scope text counted 2.4.12 (AAA and new in 2.2)
  into the A/AA rate; now only what is also in the denominator is counted. All `help_url`s now
  point to `WCAG22/Understanding` and `WCAG22/Techniques`; 2.5.5 along the way to the 2.2 name
  `target-size-enhanced`.

  **3.3.8 Accessible Authentication (Minimum)** has been added as its own rule, which raises the
  rate to 39/55. Only what is measured is a violation: a synthetic `paste` event with a probe value
  is fired at every password and one-time-code field; the field only counts as blocked if the event
  was canceled **and** the value does not end up in the field. A handler that cancels the native
  paste and inserts the text itself, sanitized, thus goes unchallenged. An interactive CAPTCHA in a
  form with a password or code field is a review hint, not a violation, because object recognition
  and alternatives cannot be decided from the DOM; a CAPTCHA in the contact form does not belong to
  3.3.8. `autocomplete="off"` on the password field is deliberately not reported — browsers ignore
  it there.

  When running the detection corpus it turned out that the `duplicate_id` fixture still expected
  an unreferenced duplicate as a violation. It now checks both sides: referenced (`label for`) is a
  violation, unreferenced is not. Checked: detection corpus with real Chrome fully green,
  `cargo test --all-features`, clippy for both feature sets, www.casoon.de (rule runs, no finding,
  rate 39/55, `report-lint` without findings).

  **3.2.6 Consistent Help** is checked across pages in batch/sitemap mode. Per page,
  `patterns::help_mechanisms` captures the help mechanisms outside `main` in document order:
  contact/help/FAQ links, `mailto:`, `tel:` and known chat widgets, each with its page region
  (header, navigation, sidebar, footer, floating). Content in `main` and invisible elements are
  left out, otherwise every contact link in an article would count as an inconsistency.
  `audit::batch_consistency` compares all mechanisms that occur on at least two pages with the
  placement used by most pages. A deviation is either a mechanism in a completely different region
  (disjoint region sets, one additional occurrence does not count) or a swapped order of two
  mechanisms in the same region. Without a recurring mechanism the criterion stays "check
  manually". In single-page mode 3.2.6 is now in the list of criteria to be checked manually (11
  instead of 10), with a German title from a new table for the 2.2 criteria that EN 301 549 V3.2.1
  does not contain.

  Along the way it turned out that the batch JSON did not output the cross-page WCAG assessments
  (3.2.3, 3.2.4, 3.2.6, 2.4.5) at all, although `OUTPUT_CONTRACT.md` promises them — they were only
  in the PDF. `site_analysis.consistency` now carries `wcag_cross_page` and `help`. Checked: new
  Chrome integration test of the detector (`tests/help_mechanisms_detection_test.rs`), unit tests
  for region and order deviation, PDF test for the localized deviation list, batch run
  www.casoon.de (12 pages): 2 recurring mechanisms, no deviation, German text in the Typst source
  correct.

  **2.5.7 Dragging Movements** is in the list of criteria to be checked manually (now 12). Whether
  a dragging movement has an alternative with simple pointing is behavior and cannot be read from
  the markup. A hint detector (sortable lists, `draggable`) was deliberately not built: every rule
  with a WCAG reference counts as automated here, so 2.5.7 would have slipped into the rate without
  a violation ever being provable. The rate stays 39/55.

  **`summary.wcag_coverage.wcag_version`** (`"2.2"`) makes the reference version machine-readable.
  Reports from before the switch do not have the field and were based on WCAG 2.1. Both JSON
  schemas know the field, `schema_version` stays 2.0 (additive).

  The reference run on www.inros-lackner.de turned up a counting error from the 4.1.1 rework:
  `rule_outcomes` reported 10 findings for `ids/duplicate`, `violations` contained none. The filter
  on referenced IDs discarded the findings, but the note from `a11y-rules` was passed on unchanged.
  It is now trimmed by the discarded findings. Checked: unit test, `cargo test --all-features`,
  clippy for both feature sets, reference runs casoon.de (single + batch) and inros-lackner.de with
  `wcag_version` "2.2" and without `report-lint` findings. inros-lackner.de stands at accessibility
  47 (on 2026-09-20: 20); the ten unreferenced duplicate IDs no longer count. How much of the jump
  is due to that and how much to changes on the site was not measured in isolation.

- **Reproduce the docs.rs build in CI, 2026-09-23 (#588):** The reporter of #588 pointed to an
  action in beta that runs the doc build in the *docs.rs sandbox image* — with its nightly, its
  resource limits and on what `cargo package` would publish. That is the question the existing doc
  step cannot answer: it builds on the runner, with stable and without a memory cap.

  The occasion is in the build log of 1.5.1: **4.75 GB of 6.4 GB** available RAM, 1m35s of 15
  minutes. Since `all-features = true`, docs.rs also documents `ai-transparency`, which lifts the
  dependency graph from 454 to 565 crates. A build that dies at this ceiling is only noticed after
  `cargo publish`, and a published version cannot be replaced.

  The job runs **on tags only**: the question is "can docs.rs manage this build", and it is to be
  answered once, shortly before the version is published. On every pull request it would cost
  seven runner minutes and a cache slot for a commit nobody publishes. `CLAUDE.md` now lists the
  job as a mandatory step before `cargo publish`.

  The action is pinned to a commit instead of a tag — it starts foreign Docker in CI — and the job
  runs with `continue-on-error: true` while it is in beta. A new `dependabot.yml` keeps an eye on
  the pin (and the other actions).

- **Remaining rustdoc warnings cleared, 2026-09-23:** After #588 seven warnings were left that the
  gate deliberately let through at the time: two links to names that do not exist (any more)
  (`NodeId` without a path, `RuleOutcome` instead of `Outcome`), three links from public docs to
  private items (`defect_key`, `is_layout_only_role`, `score_intent_fit`) — these now appear as
  code instead of a link, because a link the reader cannot follow is none — and two bare URLs in
  `cli/args.rs`. The last two stay bare: these doc comments are clap's `--help` text before they
  are rustdoc, and angle brackets or backticks would then appear in front of every user in the
  terminal. There is now a `#[allow(rustdoc::bare_urls)]` with exactly this justification.

  The CI gate follows suit: instead of `-D rustdoc::invalid_html_tags` now
  `RUSTDOCFLAGS: -D warnings`. Verified with `cargo doc --all-features --no-deps` (no warning),
  clippy, `cargo fmt --check`, 1546 unit tests and a look at `--help`, where the example URLs
  remain bare.

- **docs.rs builds with all features, 1.5.1, 2026-09-22:** Without `[package.metadata.docs.rs]`,
  docs.rs only builds with `default = ["pdf"]`. Everything behind an optional feature — such as
  `c2pa` — was therefore missing from the published docs. `all-features = true` fixes that. The
  setting only takes effect for versions published afterwards, hence 1.5.1 instead of an addendum
  to 1.5.0.

- **Live regions observed over time, journeys freed from measurement artifacts, 2026-09-22:**
  The form-error journey asked whether, after submitting, an `[aria-live]` region exists that did
  not exist before. That made it blind to exactly the recommended implementation: whoever has the
  empty live container in the markup from the start got a `FormErrorInvalidWithoutLiveRegion` with
  severity High. The add-to-cart journey compared two samples and therefore saw no message that
  appears and disappears again after two seconds — the normal case for cart feedback.

  New is `interaction::live_regions`: a `MutationObserver` over live regions, also in shadow roots
  created later, records *that* a region received content, with timestamp and urgency. Both
  journeys evaluate these events. If the observer cannot be installed, no finding results but a
  `live_observer_unavailable` in the trace — without a measurement, "nothing announced" is not a
  statement about the page. Observed does not mean read aloud: the module only proves that the
  browser had cause to.

  On re-checking against the corpus, further journeys turned up that reported measurement
  artifacts:
  - **Tabs:** `element.click()` does not move focus, so the arrow key never hit the tab list. 20 of
    22 tabs journeys reported `TabsSelectionNotMoved` (High). Now it is only evaluated if focus is
    on the activated tab.
  - **Menu:** Escape is only evaluated if focus is in the menu, otherwise the key hit the page and
    not the menu.
  - **Modal:** The journey now reads `modal` on the opened dialog. Non-modal dialogs were so far
    reported across the board as a High finding; now focus containment only applies to modal
    dialogs, and `FocusTrapBackgroundNotHidden` only to dialogs that contain focus without
    identifying as modal. If focus was on `body` before opening, `body` is the correct restoration
    after closing. Also, the candidate search never produced a trigger as long as no dialog was in
    the tree — a closed `<dialog>` has no role. Over 171 pages run, not a single modal journey
    therefore ran.
  - **SPA navigation:** Title, URL, heading and focus come from two `AXSnapshot`s instead of
    `document.querySelector('h1')`; thus shadow roots and `role="heading"` count too. The initial
    capture is retaken before every attempt so that a failed attempt does not impute a focus
    movement to the next.
  - **Disclosure:** The magnitude of the native `<details>` gap has been re-measured: 22 of 238
    domains, not "280 of 663 pages".

  Verified with `cargo test --all-features --lib` (1546 passed), three new Chrome integration tests
  (transient error message, correct modal dialog, correct menu — each without a false alarm) and
  clippy `-D warnings` for both feature sets.

- **Raw HTML tags in doc comments, 2026-09-22 (#588):** docs.rs reported truncated pages, e.g.
  `PageHealthAnalysis`: doc comments named elements like `<iframe>` without backticks, rustdoc
  output them as real HTML, and the streaming rewriter of docs.rs broke on it. 16 places in six
  files are now inline code. Verified with `cargo doc --all-features --no-deps`: no
  `invalid_html_tags` warning any more. CI now builds the docs with `-D rustdoc::invalid_html_tags`
  so that new cases are noticed before the release.

- **a11y-core 0.10.0 and the heading rules handed over, 2026-09-21:** The project was on
  `a11y-core` 0.6.0, 0.10.0 was published. The jump over four minor versions brought not a single
  API break — `dom_document.rs` and `wcag/shared.rs` translate unchanged — but twelve additional
  identifiers: five `landmarks/*`, `keyboard/skip-link-missing`, `zoom/viewport-missing`,
  `headings/h1-multiple`, `links/generic-name`, `aria/required-attribute-missing` and the two
  `contrast/*`. That makes 42 shared identifiers available.

  A test failed in the process, and rightly so: it demanded that with `Semantics` no rule drops out
  for lack of capability any more. That held at 0.6.0 because there was no tier 3 there. Since
  0.7.0 the contrast rules report `CapabilityMissing` without rendered styles — the principle "not
  run is not passed", applied to auditmysite itself. The assertion now names the two identifiers
  instead of demanding a zero that no longer means anything. Serving `Rendering` over CDP remains
  a separate step.

  With `headings/h1-multiple` the last gap fell where `wcag::rules::headings` could only be
  replaced in part. The module is deleted, all four checks come from the shared inventory,
  `SHARED_RULES` carries 16 instead of 12 identifiers. On clearing out, a mismeasurement came to
  light: the AX version sorted headings by `node_id` *as text*, so "h10" came before "h2" and
  jumps in longer pages were rated wrongly. The shared version runs in document order over the DOM.

  Two intended differences remain: multiple `h1` are `REVIEW` in the shared version instead of a
  violation and report once instead of per surplus heading — multiple `h1` are permitted in HTML.
  And "empty" means in the shared version "no text in the subtree, no `aria-label`"; a heading that
  is only named via `aria-labelledby` or an `alt` in an image is wrongly reported by the shared
  version. That is noted as a finding for `a11y-core`, not as a reverse exception here.

  Verified: 1,716 tests green, `clippy` and `fmt` clean, the corpus expectation `heading_order`
  raised to `headings/skip-level`. The completeness test of the corpus was once again the
  guardian that would have found a leftover identifier.

- **Maintainability hotspot 1, 2026-09-20:** `src/output/pdf/single_report.rs` was, with 3,717
  lines and 21 changes in 60 days, the most frequently touched file of the project and mixed
  risks/strengths, problem profile, score-driver table, subcategory breakdown, appendix, findings,
  root-cause analysis and roadmap.

  The problem-profile cluster has been extracted to `src/output/pdf/problem_profile.rs` (527
  lines): nine functions, two constants and two types. The cut was low-risk because nothing of it
  was referenced from outside `single_report.rs` — only three functions had to go from private to
  `pub(super)`. `single_report.rs` drops to 3,219 lines; the diff there contains nothing but
  deletions apart from three import lines and two visibility raises.

  Verified as output-neutral: the `--debug-typ` run of www.inros-lackner.de has the same line count
  before and after and an identical problem-profile section.

  The test helper `test_report_view_model` was not duplicated but made visible via
  `pub(in crate::output::pdf)` — a second copy of a 70-line fixture would be exactly the kind of
  duplicate this refactoring removes.

  Reassessment for hotspot 2: `normalized.rs` is now the largest file with 4,553 lines and 20
  changes/60 days and the next candidate.

- **Presentation artifacts in the PDF, 2026-09-20:** Six small, independent defects from a review
  of the two `--debug-typ` sources, each a few lines, none a product decision.

  - **The appendix claimed 36 automatically checked criteria and listed 56.** The cloud carried
    the title of the WCAG 2.1 A/AA rate but additionally contained AAA and the WCAG 2.2 criteria
    without saying so anywhere. Now it carries its real number plus a line that breaks it down.
    While writing, the breakdown at first did not add up -- 36 + 4 + 17 = 57 against 56 listed --
    because one AAA criterion is also WCAG-2.2-only and was counted in both groups. The three
    groups are now disjoint, and `the_three_appendix_groups_partition_the_automated_criteria`
    pins the sum.
  - **`◐` (U+25D0) was silently swallowed.** No bundled font covers the character; Typst reported
    "Dropping U+25D0" and five medium-confidence rows began with a gap where all the others had a
    symbol. There was no legend anyway. The scale is now a word
    (`[gemessen]`/`[wahrscheinlich]`/`[schwaches Signal]`, i.e. measured/probable/weak signal) --
    verified: no Typst warning any more, and the five rows now carry their label.
  - **Module takeaway appeared twice.** The `section-header-split` body was repeated two
    components later as its own label. New `interpretation_beyond_takeaway`: the label now only
    shows what the header has not already said.
  - **"Header 0/10" in the success color.** The accent was pinned to teal; it now comes from the
    ratio, via the same `score_color` function as everywhere else. inros-lackner 0/10 is red,
    casoon 8/10 green.
  - **"Details im Methodik-Anhang" ("details in the methodology appendix") led nowhere.** The
    appendix repeated the same sentence; which measurements were discarded was stated nowhere,
    although `execution.navigation.stability` knows it. The note now names them itself
    ("Betroffen: Desktop-Ansicht nach 1500 ms, Mobile-Ansicht nach 1500 ms", i.e. affected:
    desktop view after 1500 ms, mobile view after 1500 ms). `mutation_count` is only named if it
    was actually counted -- otherwise the value falls back to 0, and "noch 0 DOM-Aenderungen"
    ("still 0 DOM changes") would be a default printed as a measurement.
  - **Contradictory DOM classification.** "(Performance 75 Pkt stabil)" (performance 75 pts
    stable) reassured about exactly what the action list two blocks further up prioritizes
    ("DOM-Struktur verschlanken", i.e. slim down the DOM structure). Now "bei derzeit
    tragfaehiger Performance" (with currently adequate performance) -- both facts, neither
    contradicts the other.

- **"Little text at the top" measured node position instead of page position, 2026-09-20:** The
  finding tells the customer "Wenig sichtbarer Text im oberen Seitenbereich — Nutzer erhalten
  keine sofortige Orientierung" (little visible text in the upper page area -- users get no
  immediate orientation). That is a statement about the *rendered* page; what was measured were
  the first 50 nodes of the accessibility tree in document order.

  After the role fix, 93 of 2731 cached pages remained flagged, and these remainders were not a
  smaller version of the same problem but the failure of the approximation. On
  www.inros-lackner.de the first 50 nodes are a skip link and forty wrappers; the `article`
  elements stand as empty shells *before* their own text. Tree order there is no vertical order at
  all, so even a wrapper-skipping variant (measured: 13 instead of 93) would only have swapped one
  uncalibrated constant for another.

  Now `JourneyModule::collect` measures in the page: a `TreeWalker` sums the text whose box lies
  within the first viewport height. In document coordinates (`rect.top + scrollY`), because
  another module may have scrolled before, and without screen-reader-only text — the same
  `__amsIsVisuallyHidden` helper that the in-page WCAG checks use. The same pattern as the module's
  already existing `<main>` DOM check. The tree approximation remains as a fallback for calls
  without a page (`analyze_journey`, tests) so that a missing measurement is not read as "no
  text".

  Verified in both directions with three fixtures: text behind a 3000px hero is flagged, a page
  with a heading and paragraph at the top is not, and 79 characters of pure screen-reader text at
  the top do not count as visible text. www.inros-lackner.de is no longer flagged, journey there
  65 -> 68; www.casoon.de unchanged at 91. Two fixtures and an end-to-end test in
  `tests/integration_test.rs` pin down both directions — the old approximation could not tell them
  apart, because the heading is at the front of the tree either way.

- **Flaky fixture server, 2026-09-20:** `security_detection_corpus_matches_real_analyze_security_run`
  fell over sporadically with a FALSE-NEGATIVE message -- the worst possible form of a flake,
  because it reads like a real regression in the security detection. A port race was suspected;
  but `bind` on port 0 was already there.

  The actual cause: the listener is set to non-blocking so that the accept loop can poll a
  shutdown flag -- and under macOS/BSD the **accepted socket inherits this flag**. `read` and
  `write_all` could therefore return `WouldBlock` before the request was even there, and both
  results were discarded with `let _ =`. The response went out unwritten, the client saw nothing,
  `analyze_security` fell back to empty headers (which is right for a page that refuses HEAD and
  GET) and the corpus diff reported a missing header.

  Proven rather than suspected: a probe showed `read -> Err(WouldBlock)` in **every** run, and
  under 16-fold parallel load the old test fell over in **29 of 48** runs, with exactly the message
  originally reported (`cors_wildcard_credentials`). After the fix 48 of 48 green.

  The fix: accepted connection back to blocking, read the request head to the end, each connection
  in its own thread (an accept loop waiting for one client starves all the others), and transport
  errors are recorded instead of swallowed. The test now checks `fixture.served() > 0` before it
  rates anything as a detection failure -- verified with a simulated transport error that is now
  reported as such instead of as a FALSE NEGATIVE.

  The same server was **copied six times** in the integration tests, each copy with the same race.
  It now lives once in `tests/common/fixture_server.rs` (298 lines fewer). The four Chrome-gated
  suites run green with it.

  Along the way a second, already existing flake turned up:
  `test_concurrent_wait_for_stable_stays_within_its_timeout_budget` put its 4.65 s budget around
  the entire task including `new_page()`, although only `wait_for_stable` was meant -- on a busy
  machine the full `--ignored` round therefore fell over 2 out of 2 times on the **unchanged**
  code. The outer bound is now a pure hang guard; the actual assertion still measures only
  `wait_for_stable`. After that 3 of 3 full rounds green.

- **`StaticText` was invisible to every text-measuring reader, 2026-09-20:**
  `AXTree::iter()` filters out browser-generated roles, `StaticText` among them. For WCAG rules
  that is right -- a finding must not be reported against a node that nobody wrote. For everything
  that *measures text* it is wrong: on a real page the visible words are exactly there. Measured
  on two cached trees: 72-76% of all characters in a `name` sit on browser-generated roles, and
  `paragraph` nodes never carry a `name` of their own at all.

  Thus `matches!(role, "StaticText" | "paragraph")` over `iter()` read nothing on any real page.
  Two places were affected:

  - **Page-type classification** (`page_intent.rs`): `text_len` was structurally 0, so
    `approx_words > 500` could never take effect. A second dead constant turned up in the process:
    the marketing fallback additionally required `approx_words < 300`, which with a sum of 0 was
    always true. With real word counts, 222 pages would have dropped from "Marketing" to "Nicht
    erkannt" (not recognized) -- a landing-page default losing against a 300 limit that was never
    checked. The clause is removed because it never took effect.
  - **Journeys "early content"** (`analysis.rs`): read `StaticText | heading` and thus saw only
    headings. `LittleEarlyText` -- 20 points deduction on Entry Clarity -- fired on **1595 of
    2731** unique cached pages. More than half of the web allegedly without text at the top.

  One definition instead of role lists per call site: `AXNode::is_text()` and
  `AXTree::text_nodes()`/`visible_text_len()` count the text exactly once -- `StaticText` and
  nothing else, because Chrome repeats the same run on the `InlineTextBox` children and the name
  of a heading or link is computed from the same `StaticText` descendants (measured: 30 of 44
  heading names and 71 of 87 link names are word-for-word identical with a `StaticText`).

  Verified over 2731 unique cached pages with the new `tests/page_intent_corpus.rs`
  (`#[ignore]`-gated, reads a local artifact cache): `LittleEarlyText` drops from 1595 to 93, and
  68 pages change page type -- all in the direction of Editorial, 48 of them on an actual blog that
  previously passed as Corporate.

  **Not fixed along with it:** the time window itself. `LittleEarlyText` promises "little text in
  the upper page area" but measures the first 50 nodes in tree order. On www.inros-lackner.de the
  skip link and forty wrappers stand there. A variant that skips pure wrappers was measured (13
  instead of 93 flagged) and rejected: the `article` nodes of this page stand as empty shells
  before their own text, so tree order there is no vertical order at all. That needs geometry
  instead of a new constant and is tracked separately.

- **`AXTree` without document order, 2026-09-20:** `AXTree.nodes` is a `HashMap`, and
  `iter()`/`headings()` returned `values()` -- hash order, which Rust sets randomly per process.
  Detectors that read the tree as a document thereby contradicted themselves: two consecutive
  audits of www.inros-lackner.de from the same binary, one reported UX 78, the other UX 81,
  without anything having changed on the page. Affected were the heading-skip detection
  (`last_level` presupposes document order) and journeys "early content" (`iter().take(50)` over a
  `HashMap` is fifty arbitrary nodes, not the start of the page).

  CDP already delivers `getFullAXTree` in document order -- `from_nodes` merely threw it into the
  `HashMap`. A new private field `order: Vec<String>` retains it; `iter()` and `iter_all()` run
  over it. If the order is missing (a snapshot from the cache written before the field, or a
  direct write access to the public `nodes`), it is reconstructed from `child_ids` by depth-first
  search from the root, unreachable nodes appended sorted by id -- degraded, but deterministic.

  Verified: three consecutive runs over www.inros-lackner.de deliver identical UX findings and
  module values. Journey drops from 68 to 65 in the process, because `take(50)` now actually reads
  the start of the page. The `HeadingSkips` case deliberately left out earlier is now in
  `every_penalty_states_its_own_reason_at_the_first_occurrence` -- it could not be asserted as long
  as h1 -> h3 was only a skip depending on the run.

  **Not fixed along with it:** the `StaticText` blindness of the `iter()` readers, which stood in
  the "Related" section of the write-up as two call sites. The one-liner (`iter()` ->
  `iter_all()`) was implemented, measured and discarded again: it changed the classification on
  neither of the two test pages, and a guard test for it passed even with the defect, because the
  keyword scan over it does not see `StaticText` either. That is the actual problem and is now
  tracked separately.

- **Score hierarchy, 2026-09-20:** The report carried around 30 different 0-100 values, 14 of them
  with a grade A-F. The subject of the tool -- accessibility -- was one of them, with 40% weight
  in another. Four decisions, four commits, after which a three-level rule holds: accessibility as
  the headline, the five remaining weighted modules as rated context, everything else as an
  indicator without a grade.

  - **Grade only for modules that carry the overall value (D2):** Seven of the thirteen modules
    have `weight_pct: 0`, so they feed nothing, but carried the same A-F grade as accessibility.
    So "Dark Mode: F" stood there -- a grade for an optional product feature that the code itself
    carries as `measurement_type: "optional"` -- next to an F for the exclusion of screen reader
    users. `ModuleScoreEntry::new` derives weight, grade and band from one source: `grade` is an
    `Option` and only set for weight > 0, `band` always carries the canonical English FIVE_BAND
    label. Likewise for the `grade` fields of `AiVisibilityAnalysis` and `SourceQualityAnalysis`
    and for `dark_mode`. The PDF was not affected -- it showed band words anyway, the grades were
    only in the JSON.
  - **Counters instead of a rate where the denominator is arbitrary (D4):** Content Visibility was
    `(signals - problems) / signals`, i.e. "share of checks that did not complain" -- one more
    check that usually passes raised the value of every page. The score is dropped without
    replacement; the counters that already stood next to it are reported. In the process it turned
    out that the success message "Alle Content-Visibility-Signale sind in Ordnung" (all content
    visibility signals are fine) appeared from `score >= 80`, i.e. also with up to a fifth of
    problem signals; it now hangs on `problem_count == 0`. One level deeper, `SignalCategory`
    carries `passed`/`total` instead of `score_pct`, and the table title says "17 von 24 Signalen
    erfuellt" (17 of 24 signals met) instead of a percentage weighted with six invented factors.
  - **Declare derived indicators as such (D3):** A reader saw SEO, AI visibility, source quality,
    UX and Journey as five separate assessments that happen to agree. `AiSignalKind::TechnicalTrust`
    folded `a11y_score >= 80` and `security_score >= 70` into citability,
    `QualitySignalKind::Accessibility` counted the accessibility score once more in the authority
    dimension -- both removed. The eleven invented citation weights (0.08, 0.15, 0.10, ...)
    suggested a calibration that does not exist; now equally weighted under a named
    `SIGNAL_WEIGHT`. What remains is declared: new field `derived_from`, fed from
    `taxonomy::module_derived_from`, plus a sentence in the PDF under every affected indicator. The
    accessibility deduction on UX and Journey is quantified instead of silently included.
  - **The headline is accessibility (D1):** Grade and certificate hung on the weighted overall
    value (#233, so that they do not contradict each other). That resolved the internal
    contradiction but moved the figure away from the subject of the report:
    inros-lackner.de with accessibility 20 appeared on the cover as 45, because performance,
    mobile and SEO pulled it up; casoon.de with accessibility 100 as 92. Both now read the
    accessibility score, so they still cannot contradict each other. The weighted value remains as
    a named ScoreCard "Kombinierter technischer Wert" (combined technical value) with its weight
    basis -- without it, it would have disappeared from the PDF, since it only stood on the cover.
    `report-lint` checks against `accessibility_score`; new fixture, because in all previous ones
    both scores are equal and none would have noticed the switch. Incidentally two more band tables
    eliminated: the batch builder computed grade and certificate with its own literals
    (95/90/85/80/70/60), the batch cover with `BATCH_GRADE` -- so the same score was graded
    differently depending on the report. Both now use the same two functions as the single report;
    `BATCH_GRADE` is dropped.

  Verified on fresh reports of www.casoon.de (accessibility 100, combined 92, grade A) and
  www.inros-lackner.de (accessibility 20, combined 45, grade F, certificate NICHT BESTANDEN, i.e.
  not passed); `report-lint` on both without findings.

- **Report quality pass, 2026-09-19:** Result of a `report-critic` run over fresh single-URL
  reports for www.casoon.de and www.inros-lackner.de. `auditmysite report-lint` found nothing on
  either report — all points are semantic in nature and structurally invisible to the
  deterministic linter. Six commits, one per finding.

  - **Explanation lookup skipped the axe identifier:** The JSON writer and the batch report
    resolved the customer-facing explanation via the taxonomy `rule_id`; `get_explanation` falls
    back from there to the bare WCAG number, and the rule-specific entries hang off the axe
    identifier. **Finding:** 4 of 15 `detail.fix_guidance` entries carried foreign text — the
    three landmark rules appeared as "Missing semantic structure" with table markup as the code
    example. The single PDF was correct, only the JSON was not. Same class of error as #571.
    New `resolve_explanation` as the single resolver at all three call sites. While fixing it, it
    turned out that the now-reachable code examples would have put German text into the canonically
    English JSON; the guard now checks for more than umlauts.
  - **`accessibility_score_breakdown` was a second score model:** its own severity weights
    (20/14/8/4), a cap at 90, invented importance weights. **Finding:** an `accessibility_score`
    of 20 next to areas whose weighted average came to 57. A weighted average cannot reproduce the
    score — `diversity_factor`, sqrt compression, `apply_soft_floor` and the cap are non-linear and
    depend on the entire set of findings. The two conflated questions are now two fields: `score`
    is the area on its own, run through the scorer (new `score_from_penalties`),
    `estimated_lost_points` is its share of the actual loss.
    `sum(estimated_lost_points) == 100 - accessibility_score` holds exactly and is checked by
    `report-lint`. Schema unchanged.
  - **Score areas came from running text:** `score_area_for_finding` searched for words in
    `rule_id + title + description + subcategory`. **Finding:** `a11y.landmark_region.missing`
    ended up under "Images / alternative text" because its description mentions `role 'image'`;
    `seo.headings.multiple_h1` drove an area of the *Accessibility* breakdown. New
    `taxonomy::score_area` with a total `rule_id -> ScoreArea` table over all 120 Accessibility
    rules and three inventory tests. `score_area_for_key_point` was a second, word-for-word copy of
    the same search and now delegates.
  - **"Erwartungswert fuer diesen Seitentyp" (expected value for this page type) was a literal:**
    `intent_fit_score` is one of eleven constants and was compared against `seo.score` — a
    different function, a different scale. For `ThinContent` (28) the statement flipped: a thin
    page "met the expectation". The comparison is dropped; the number stays as a quality value.
    In addition, the gate `intent_fit_score < 65` for three page types hangs on a pure type
    constant, so a media-heavy page received the advice regardless of its quality.
  - **Two models for the same risks:** `build_management_risks` filled the JSON,
    `compute_dimension_rows` built the PDF panel alongside it. **Finding:** the JSON carried 4x
    high, the panel showed 3x bad and 2x warn, two high dimensions were missing entirely, and all
    evidence-backed rationales were discarded. New `audit::management_risk` as the single
    derivation following the #406 pattern. In the process three statements without a measurement
    basis were dropped: the conversion chains (which reached no output format anyway), the
    constant trust sentence and the template conclusion drawn from a single page.
    `RiskTier::Unknown` is new — a module that had not run was previously rendered as a
    *strength* via `unwrap_or(100)`.
  - **Security claimed a clean bill of health over open findings:** The clean notice and the
    takeaway hung on the score, not on the findings list, and `derive_security_recommendations`
    only checks for *missing* headers — a present but misconfigured header produced nothing, and
    the default claimed the headers were set cleanly. **Finding:** casoon.de, score 95, three open
    CSP findings (one High), and the section read "Sehr gut — keine wesentlichen
    Sicherheitsauffaelligkeiten" (Very good — no significant security findings). The band
    correction is now symmetric, the recommendations hang off `SecurityIssueKind`, and the
    ScoreCard takes the same corrected band — otherwise "95 — Sehr gut" (Very good) stood above a
    takeaway "Verbesserungswuerdig" (Worth improving). The same defect was present in the SEO
    section.

- **Heuristic hygiene: no more silent deductions, 2026-09-19:** Parts 1 and 2.

  - **Deductions without a stated reason:** In `ux/analysis.rs`, three checks lowered the score
    completely silently (word count 50-100, long page with fewer than five headings, fewer than
    three trust keywords), and four more deducted from the first occurrence but only reported
    from a considerably higher threshold. **Finding:** `reports/inros-lackner-audit.json` showed
    `content_clarity: 80` without any ContentClarity finding in the list — 80 is exactly one
    20-point deduction, the silent word-count branch. The rule was already in the code: the
    `LargeDom` branch carries the comment that the score must never drop without a stated reason —
    but it was applied in exactly one place. Now every deduction reports; so that this does not
    flip from silent to alarming, `ux_issue_severity` is value-dependent: mild bands report `low`.
    In `journey/analysis.rs`, five deductions had no friction point; there are five new
    `FrictionKind` variants for them.
  - **Binary thresholds next to the percentage they ignore:** The cross-page consistency signals
    were booleans while the report printed the real percentage — 94 % HSTS coverage counted
    exactly like 0 %, visibly next to a "94 %". `QualitySignal` now has a `fulfilment` (0.0-1.0);
    `present` remains the pass/fail marker, the calculation is proportional. `ErrorFreePages` uses
    the `clean_pct` that it had already computed and discarded. True yes/no signals remain
    unchanged.
  - **Found along the way:** `AXTree` iterates over a `HashMap`, and the heading-skip detection
    reads that anyway — two runs of the same page produced different findings. In addition,
    `analyze_content_clarity` counted text via `iter()` and listed `StaticText` as the first role,
    although `iter()` filters out exactly this role: the branch could never trigger, and the
    deduction was a **false finding**. With `iter_all()`, inros-lackner.de goes from
    `content_clarity` 80 to 100. Fixed here because otherwise a silent false finding would have
    become a visible one.

- **Verdict, rating and derivation of the overall value, 2026-09-19:** Continuation of the report
  quality pass.

  - **The overall value was not derivable:** The table "Warum ist der Gesamtwert, was er ist" (Why
    the overall value is what it is) listed the modules with their weights; recalculated, that
    gave 90.05 against a reported 92 (casoon) and 47.05 against 45 (inros-lackner). **Finding:**
    there are two calculation paths, and only one of them is the module weighting. In
    `viewport_weighted` mode the value arises from a desktop/mobile blend with security mixed in —
    this is in `score_breakdown` and was rendered nowhere. The table there is now called "Die
    gewichteten Module im Einzelnen" (The weighted modules in detail), a second table shows the
    real calculation and reproduces the value exactly. In `module_weighted` mode everything stays
    as it was — there the weighting *is* the derivation.
  - **Verdict and rating were missing from the PDF entirely:** `summary.certificate` was only read
    as a gate, `verdict`/`verdict_reasons` were not rendered at all. The `SummaryBlock` field named
    `verdict` is a prose sentence, a different value under the same name. **Finding:** the CLI
    printed "FAIL — legal_flags: 5, blocking_issues: 36", nothing of this was in the document, and
    the cover page read "Ausbaufaehiger technischer Zustand" (Technical state with room for
    improvement). Decided: the PDF names both, and the label is "Gesamteinstufung" (overall
    rating) instead of "Zertifikat" (certificate) — `calculate_certificate` maps the score to a
    band label and certifies nothing. A partial run may carry a rating, but as a provisional one,
    and this now stands up front instead of only in the appendix.
  - **The verdict reasons were machine tokens:** Rendering `legal_flags: 5` unchanged would have
    brought canonical English into the German PDF. `VerdictReasonKind` now carries the numbers,
    `text(en)` is the single reader-facing source; `VerdictResult::reasons` keeps its exact
    strings for JSON and CLI, `reason_kinds` is `serde(skip)`.

- **Switch to the shared a11y-core crates, second part, 2026-09-19:** All three crates from 0.2.0
  to 0.3.0. This removes the blocker of the first part: `RuleRun` now carries `viewport` and
  `wcag`, and `Finding` has `rule_name`, `with_element(role, name)` and the opaque `Extra` slot —
  the fields on which the switch of `Violation` previously failed.

  - **`RuleOutcome` replaced by `a11y_report::RuleRun`:** The execution note comes from the shared
    crate. The key of a note is thus `(rule_id, viewport)` — the same rule runs once per viewport,
    and `rule_runs` and `findings` use the same set of names. **Finding:** Of the seven values of
    the local `RuleOutcomeStatus`, four said the same thing — that the rule ran. No evaluator in
    the code ever distinguished `ViolationsFound`, `Warning`, `ManualReviewRequired` and
    `NoViolationDetected`; only `Failed` and `Skipped` were ever read. The four were write-dead.
    That they collapse is not a loss but the two-axis cut: *how certain* a statement is lives on
    the finding (`Outcome`), not on the note. **JSON changes** in
    `pages[].detail.rule_outcomes[]`: `status` is dropped (`not_run` now says whether and why a
    rule did not run), `wcag_criterion` → `wcag` (array instead of a single value),
    `reason_code` → `reason`, `finding_count` → `findings`.
  - **The translation layer in `wcag::shared` is dropped:** Both sides speak the same model, the
    note from `a11y-rules` is passed through instead of translated. Only the WCAG criterion is
    added, which the shared inventory does not carry on the note.
  - **Two more rules replaced and deleted, not placed alongside:**
    `parsing::check_parsing_with_page` (axe identifier `duplicate-id`) now runs as
    `ids/duplicate`, `focus_order::check_positive_tabindex_with_page` as
    `keyboard/positive-tabindex`. Both read the DOM via JavaScript, because neither `id` nor
    `tabindex` is an AX property (#QA-030); the shared rules read the same attributes from the CDP
    dump. **JSON changes:** `rule_id` for these findings is `ids/duplicate` and
    `keyboard/positive-tabindex` respectively. **Severity drops** in both cases from `High` to the
    `Medium` of the shared inventory — this affects scoring and is deliberately not overridden
    locally, because a custom severity would break the guarantee that the same finding is named
    the same everywhere.
  - **What stands alongside and why:** `parsing::check_parsing` (`duplicate-id-aria`, contradictory
    `aria-owns` relationships in the AX tree) and `focus_order::check_focus_order` (focusable
    despite `aria-hidden`) remain — they check something different from the replaced halves.
    `iframe_rules` still produces `duplicate-id`, but for the document *inside* the iframe.
  - **Finding about our own test coverage:** The identifier change to `keyboard/positive-tabindex`
    propagated into `patterns_disclosure.expected.json` — this corpus case expected
    `focus-order-semantics`, triggered by the `tabindex="5"` in its fixture, not by the ARIA half
    of the same identifier. Two rules behind one axe identifier thus cannot be told apart in the
    corpus.
  - **Finding about `docs/OUTPUT_CONTRACT.md`:** The contract claimed that `rule_outcomes` uses
    the same execution states as `module_runs` (`completed`, `partial`, `failed`, `skipped`,
    `not_applicable`). That was already wrong before this switch — `RuleOutcomeStatus` wrote
    `violations_found`, `no_violation_detected`, `warning`, `manual_review_required`. It was only
    noticed because `docs/json-report.schema.json` lists `rule_outcomes` as an unconstrained
    `{"type": "array"}` and does not check the shape of the entries at all. The contract text has
    been updated.
  - **Still open:** `Violation` is still auditmysite's own type and not `a11y_report::Finding` —
    the conversion is now possible, but with 295 construction sites and 155 affected files too
    large for this part. Of 24 shared identifiers, 4 are now adopted (`document/lang-missing`,
    `document/lang-invalid`, `ids/duplicate`, `keyboard/positive-tabindex`), 20 are pending.
  - **Why not more rules:** A rule-by-rule capability comparison shows a systematic pattern
    instead of individual gaps: auditmysite's rules decide on Chrome's **computed** accessibility
    tree, the shared ones on **tags and attributes**. Where both serve the same criterion, the
    AX-based version is therefore usually the broader one. Three documented cases:
    `lists/invalid-structure` only checks `<ul>`/`<ol>` with foreign children, while
    `rules::list_structure` additionally knows empty lists and `<dt>` without `<dd>` and, via
    roles, also catches `role="list"` on a `<div>`; `tables/header-missing` looks for `<th>`,
    while `rules::table_rules` looks for `columnheader`/`rowheader` as roles and thus also finds
    ARIA-marked header cells; `zoom/viewport-locked` compares the `content` value without
    lowercasing first and thus misses `user-scalable=NO`, and it does not know the gradation with
    which `rules::resize_text` separates `user-scalable=no` (`High`) from `maximum-scale<2`
    (`Medium`). These three are candidates for **strengthening** the shared inventory instead of
    weakening auditmysite — but that belongs in a separate pull request in a11y-core and must be
    published there before auditmysite can use it.

  - **The three gaps are fixed in a11y-core, but not yet published:** A separate pull request there
    (branch `feat/staerkere-strukturregeln`, intended as 0.4.0) lowercases the viewport's
    `content` value before comparing, lets `role="list"`/`role="listitem"` and
    `role="columnheader"`/`role="rowheader"`/`role="table"` count on an equal footing with the
    tags, and adds the identifier `lists/empty`. Until this is on crates.io, auditmysite cannot
    use it — the three rules remain auditmysite's own until then.

  *Verified:* `cargo clippy --all-targets` without findings; `cargo test --lib` 1404 tests and
  `cargo test --tests` all browser-free binaries green. This time run **with** Chrome, which the
  first part owed: `cargo test --test detection_corpus_test -- --ignored` green (218 s) — that is
  where it shows that both identifier changes really hold, and exactly there the fixture gap in
  `patterns_disclosure` was noticed. `cargo test --test integration_test --
  --ignored` yields 22 passed, 1 failed — **the same state as on `main`**, against which it was
  explicitly cross-checked: `test_concurrent_wait_for_stable_stays_within_its_timeout_budget`
  also fails without these changes as soon as the full suite runs concurrently. It is a
  load-dependent time-budget test, not a regression finding — but a separate one, not previously
  noted.

- **Switch to the shared a11y-core crates, first part, 2026-09-19:** auditmysite takes its finding
  model and a first part of its rule inventory from
  [a11y-core](https://github.com/casoon/a11y-core) (all crates 0.2.0), so that the same finding
  carries the same identifier in astro-post-audit, auditmysite and liveaudit. Deliberately
  incremental: this state switches the model and **one** rule, not the whole inventory.

  - **Duplicate type definitions removed:** `Severity` and `ViolationEvidence` were structurally
    identical to `a11y_report::Severity` and `a11y_report::Evidence` respectively — same variants,
    same order, same serde representation. Both are replaced by the shared types; auditmysite's own
    additions to `Severity` (German labels, converters from the legacy systems) hang off it as
    `SeverityExt`, because inherent methods cannot be added to a foreign type. The former
    `Display for Severity` falls victim to the orphan rule; the one use site uses `as_str()`,
    which returns exactly the same strings. **JSON unchanged.**
  - **`FindingKind` replaced by `a11y_report::Outcome`:** Variant for variant the same enum under
    another name. **JSON changes:** The field `kind` now writes
    `"fail"`/`"review"`/`"pass"`/`"untested"` instead of
    `"violation"`/`"warning"`/`"positive"`/`"not_testable"`. No fixture and no snapshot observed
    the field — the change would have gone unnoticed, so a new test now pins the vocabulary
    explicitly.
  - **CDP DOM as an `a11y-dom` document (`accessibility::dom_document`):** The shared rules are
    generic over a deliberately **DOM-shaped** trait, because the majority of them decide on tags
    and attributes (`lang`, `tabindex`, `id`, `alt`). The AXTree does not provide these at all.
    The DOM is now fetched once via `DOM.getDocument` and materialized into an arena; `Semantics`
    is fulfilled via Chrome's **native** accessibility tree, not via `accname` — that is the
    substitute for hosts without a native tree. Both trees find each other via the backend node
    ID. The root is `<html>` (the rules check `doc.root()`), shadow-root children hang under their
    host, iframe contents are not entered (own `lang`/`title`).
  - **3.1.1 runs as a shared rule, our own is deleted:** auditmysite checked 3.1.1 twice and badly
    both times. `rules::language::check_language` read the AX property `language`, which Chrome
    synthesizes from locale and context even when the author never set a `lang` — so it was blind
    for the most common violation. This was compensated by `pipeline::apply_lang_attribute_check`,
    a second check pushed in afterwards via JavaScript, which withdrew or added to the first one's
    finding. Neither knew invalid language codes. Both are deleted; 3.1.1 now runs as
    `document/lang-missing` / `document/lang-invalid` directly against the DOM attribute. **JSON
    changes:** `rule_id` for this finding is `document/lang-missing` instead of `html-has-lang`.
    `html-has-lang` remains, but is now only produced by `iframe_rules` — for the document
    *inside* the iframe, a different finding.
  - **"Not run" is not "passed":** The shared identifiers that auditmysite does not yet carry are
    noted per identifier as `shared_rule_not_yet_adopted` instead of being silently discarded. If
    the DOM fetch fails, they are noted as `Failed`, not as passed.
  - **Still open:** `Violation` is still auditmysite's own type and not `a11y_report::Finding`
    (it lacks fields that auditmysite carries per finding: `rule_name`, `role`, `name`,
    `evidence_screenshot`, `evidence_viewport`); `RuleOutcome` has not been switched to `RuleRun`;
    of 24 shared identifiers, one rule with 2 identifiers is adopted.

  *Verified:* `cargo clippy --all-features --all-targets` without findings; 1667 tests in 18
  browser-free test binaries passed, 0 failed. The browser-backed corpus tests
  (`detection_corpus_test`) need Chrome and did not run.

- **a11y-core 0.6.0, 2026-09-19 — three rules replaced:** `list_structure`, `table_rules` and
  `meta_viewport_large` are deleted; their checks now come from the shared inventory.
  `SHARED_RULES` thereby carries 12 instead of 4 identifiers.

  This only became possible after a11y-core closed the gaps on which the replacement would
  previously have failed: term without definition, presentational table with header cells, table
  name, the separation of zoom violation and zoom restriction — and while clearing up, a fourth
  turned up, the orphaned list item (`<li>` without a list above it). The old rule ran over the
  AX tree and saw it; the shared one ran over lists and would never have visited it. Without
  `lists/item-outside-list` in 0.6.0, the replacement would have lost a check instead of sharing
  it.

  **One deliberate deviation:** The missing table naming is `needs_review` in the shared
  inventory, not `violation` — whether a table needs a name depends on context, and a violation
  would be a claim the rule cannot back. The corpus records this.

  `meta_viewport_large` disappears from `PAGE_RULES` (47 → 46); the viewport helpers located there
  have moved to `resize_text.rs`, which remains their only user and, beyond the 200 % threshold,
  checks text enlargement itself.

  *Verified:* 1,430 browser-free tests, 27 test binaries, clippy `-D warnings` and `fmt` clean,
  browser corpus with real Chrome (217 s). The corpus completeness test was the guard that found
  the three remaining fixture references to the old identifiers.

- **a11y-core 0.4.0, 2026-09-19 — what cannot yet be replaced:** With 0.4.0 the shared structure
  rules also read `role` attributes (`role="list"`, `role="columnheader"`, `role="table"`), and
  `positive-tabindex` is back on `High`. Three obvious replacements still do not happen — they
  would be a loss of capability, not an exchange:

  - **Lists:** `lists/invalid-structure` and `lists/empty` cover two of the three checks from
    `rules/list_structure.rs`. Missing is whether a `<dt>` has an associated definition.
  - **Tables:** `tables/header-missing` covers only the header cells. `rules/table_rules.rs`
    additionally checks caption or accessible name, and whether presentational tables wrongly
    carry header cells.
  - **Viewport:** `zoom/viewport-locked` checks `maximum-scale < 2.0` and thus corresponds to
    `rules/resize_text.rs` (1.4.4, 200 %) — **not** `rules/meta_viewport_large.rs`, which checks
    the same markup at the stricter 500 % threshold. Replacing `meta_viewport_large` would have
    been the wrong mapping.

  *Verified:* Build and 1,442 browser-free tests against 0.4.0, clippy `-D warnings` clean. The
  three gaps belong in the next a11y-core round, not in a local special case.

- **1.5.0, 2026-09-19 — license change to MIT:** From this version on, auditmysite is under the MIT
  license. Earlier releases remain under the license that applied at the time — AGPL-3.0-or-later
  up to 0.25.x, Business Source License 1.1 from 0.26.0 to 1.4.0; this is recorded in `NOTICE` and
  does not need to be changed retroactively.

  *Why:* The rule inventory is moving to [a11y-core](https://github.com/casoon/a11y-core), so that
  the same finding is named the same in astro-post-audit (build time), auditmysite (CI/crawl) and
  liveaudit (running site). If auditmysite stayed under BUSL, the license would have to be decided
  individually for every module that moves there — a demarcation that would have to be maintained
  permanently and runs counter to the goal.

  *Irreversible:* A release once published under MIT cannot be taken back. Decided deliberately.

- **1.4.0, 2026-09-18 — upgrade notes:** Not a pure patch release. What consumers of the JSON
  report and the library should check when upgrading (details in the entries below):
  - **Overall scores shift:** Since the per-cause scoring fix, HTML conformance counts with 5 % in
    the overall score, SEO drops from 20 % to 15 %. Reports of the same page yield different
    `overall_score` values than under 1.3.x.
  - **Changed JSON values** (not just added fields): `rich_snippets_potential` names the bare
    schema type (`"FAQPage"` instead of `"FAQ Rich Snippet"`); the keys in
    `mobile.issues[].values.small_by_context` are canonically English (`other`/`form` instead of
    `sonstige`/`formular`).
  - **Two SEO findings are dropped:** `pagination_missing_rel_links` and
    `pwa_missing_service_worker` are no longer emitted.
  - **Additive:** `security.issues[].values` and `mobile.issues[].values` carry the interpolated
    raw values; when empty they are omitted. `header`, `issue_type` and `message` remain
    unchanged.
  - **Rust API breaks:** `security::coop_corp_verification_text` is gone (merged into
    `security_issue_text`), `SecurityIssue` and `MobileIssue` have a new public field `values`,
    and `a11y_journey::link_inventory::analyse` now takes page and fallback language instead of a
    locale.
  - Known consumer: Studio was stuck on 1.3.1 (see auto-memory) — check before upgrading.

- **Report statements: contradictions and overclaims fixed, 2026-09-18:** Triggered by an external
  critique of the casoon.de report. In several places the report claimed more than the measurement
  supports, and contradicted itself in doing so.
  - **Link text heuristic produced false WCAG 2.4.4 claims:** `a11y_journey::link_inventory`'s
    `is_generic` compared via `lower.contains(stopword)`, i.e. as a substring. As a result, any
    descriptive link text that contained a stopword anywhere counted as generic — on casoon.de
    "Claude-Code-Kontingent: Der Verlauf kostet **mehr** als jede Konfiguration" (the history costs
    **more** than any configuration; 200 characters of card text), "**Mehr** über Jörn Seidel"
    (More about Jörn Seidel), "**Mehr** Sichtbarkeit bei Google & KI" (More visibility on Google &
    AI); the English list additionally hit German words in the middle via `here`/`more`/`open`.
    All four reported links were false positives. Now an exact comparison on the normalized name
    (lowercase, surrounding non-alphanumerics removed, so that "Mehr erfahren »" (Learn more »)
    still matches). In addition, the detection language now follows the **page** language
    (`<html lang>`, read via `eval_string` in the journey hook) instead of the run language, with
    an explicit `STOPWORD_LOCALES` check — `I18n::new` silently returns the German bundle for
    unknown locales, so a French page would otherwise have been scanned with German stopwords
    (#406 separation of detection vs. message language). Four regression tests, including the
    three real casoon.de names. The finding text also no longer claims "do not meet WCAG 2.4.4"
    but marks itself as a heuristic warning requiring manual review.
  - **Three evidence classes instead of one:** New `EvidenceClasses` (confirmed WCAG violations /
    heuristic warnings / criteria to be checked manually) on the `FindingsBlock`, computed once in
    the builder. **Scores remain unchanged** — they are still derived solely from the confirmed
    violations. The classes exist so that no count sentence can say "0 findings" while four pages
    later there are findings: The cause analysis now says "Keine bestätigten WCAG-Verstöße … Es
    bleiben N Warnung(en) und M Prüfhinweis(e)" (No confirmed WCAG violations ... N warning(s) and
    M review note(s) remain) instead of "Es wurden keine Barrierefreiheits-Befunde erkannt" (No
    accessibility findings were detected), and below the count strip of the management view it
    states explicitly that only confirmed violations are counted there. The "Heuristische
    Warnungen" (heuristic warnings) metric value now also counts the journey and screen reader
    signals, no longer just the WCAG engine's warnings.
  - **BFSG statements reduced to technical criteria:** "BFSG-Prüfung: keine Verstöße im geprüften
    Umfang" (BFSG check: no violations within the scope checked; green) and the checklist row
    "Rechtliche Konformität (BFSG) — Geringes Compliance-Risiko" (Legal conformity (BFSG) — Low
    compliance risk) read like legal clearance. A scanner can determine neither whether a service
    falls under the BFSG at all, nor whether all statutory requirements are met. Both are now
    called "BFSG-relevante technische Kriterien" (BFSG-relevant technical criteria) and say
    explicitly "Dies ist keine rechtliche Konformitätsprüfung." (This is not a legal conformity
    check.)
  - **Breadcrumb contradiction (`seo::serp`):** The SERP signal only checked the type presence of
    `BreadcrumbList` and then reported "Google zeigt Pfadangabe im Listeneintrag" (Google shows
    path information in the list entry) — while the schema table of the same report printed
    "Pflichtangaben fehlen: itemListElement (minimum 2 entries)" (Required fields missing:
    itemListElement (minimum 2 entries)). The signal now consults the already existing
    `rule_assessments`; an incomplete schema becomes a warning naming the missing fields. Two
    regression tests.
  - **Measures list across modules:** "Die 5 wichtigsten Maßnahmen" (The 5 most important
    measures) was fed exclusively from WCAG finding groups and, at 0 WCAG findings, reported
    "Keine dringenden Maßnahmen erforderlich" (No urgent measures required) — although the same
    report documented a duplicate host, a faulty breadcrumb schema, 13,756 DOM nodes and a CSP
    misconfiguration. New `cross_module_measures` fills up the list from security, mobile, SEO and
    performance recommendations, sorted by severity and then by the overall points the module
    costs. The empty text of the measures plan no longer denies findings but refers to the module
    sections.
  - **Terminology and PDF artifacts:** TBT is no longer presented as a Core Web Vital
    (`perf-lab-data-body` names LCP/CLS as CWV and TBT as a lab approximation, because INP cannot
    be measured headless); the sections "Desktop/Mobile — Core Web Vitals" are now called
    "Kennzahlen (Labormessung)" (Metrics (lab measurement)), because their strips also contain FCP
    and TTFB, the additional table is called "Weitere Lab-Metriken" (Further lab metrics). The
    thresholds for DOM nodes, load time and DOMContentLoaded are labeled "Richtwert" (guideline)
    instead of "Ziel" (target) (Lighthouse heuristic, not a standard). Internal journey IDs
    (`tab_walk`, `disclosure_0`, `skip_link_0`) no longer appear raw in the customer PDF but as
    labels, falling back to the raw ID for unknown journeys. The callout above the interactive
    findings is no longer green (the success branch was unreachable anyway, because the section is
    only rendered when findings exist) and is grammatically correct in the singular.
    `rich_snippet_type` is no longer called "FAQ Rich Snippet" but names the bare schema type —
    existing markup is not a promise about the search presentation. The overall verdict says "im
    automatisierten Accessibility-Prüfumfang" (within the automated accessibility scope) instead
    of "im Accessibility-Audit" (in the accessibility audit).
  - Verified: `cargo test --all-features` (1376 lib plus integration tests green),
    `cargo clippy --all-features --all-targets` without warnings, `cargo fmt --check`, and a real
    run against www.casoon.de — the link text finding is gone (2 → 1 interactive findings), the
    breadcrumb signal is at "Warnung" (warning) with a rationale, the measures list carries five
    cross-module items, and `report-lint` still reports no findings. Score unchanged
    (Accessibility 100, Overall 92).
  - **Two technically questionable SEO rules removed (addendum of the same day):**
    `pagination_missing_rel_links` reported missing `rel="prev"`/`rel="next"` markup as an SEO
    problem — Google has not used the pair for indexing since 2019. The remaining pagination
    checks stay: they only trigger when rel links are actually present and wrong (self-reference,
    prev == next), and that is broken markup in any case. `pwa_missing_service_worker` reported a
    missing service worker registration as soon as a web app manifest was present — but a manifest
    is routinely shipped solely for icons and theme color (on casoon.de exactly this case: valid
    manifest, `display: standalone`, no registration) and does not evidence an intent to be an
    installable, offline-capable PWA. The `service_worker_*` fields remain as information in the
    JSON. Both tests replaced by negative tests. Result on casoon.de: "Technische SEO-Probleme"
    (Technical SEO problems) shrinks from three rows to one (the real duplicate host finding).
  - **Measures list merges findings of the same kind:** After the removal of the two SEO rules,
    three separate CSP misconfigurations moved up and occupied three of the five places.
    `cross_module_measures` now merges findings with the same module and the same title into one
    measure and appends "(+N weitere gleicher Art)" (+N more of the same kind) — three CSP
    findings are one measure "CSP korrigieren" (fix CSP). `TopMeasure` carries title and detail
    separately for this; performance and technical SEO recommendations have no title of their own
    and are rendered without a separator. Three rendering tests. Also corrected along the way: the
    mobile entries carried the raw snake_case category key (`touch_targets`, `fonts`) — they now
    use the same `mobile_category_label` that the mobile module section already used
    ("Touch-Targets", "Schriftgrößen" (font sizes)).
  - **Security and mobile messages localized (addendum of the same day, #406):** Both modules
    baked English prose into `message` and the PDF renderer output it unchanged in the German
    report ("CSP allows unsafe-inline scripts without nonce/hash protection", "1 touch targets
    are too small"). Both now follow the kind-enum pattern: `SecurityIssueKind` (19 variants) and
    `MobileIssueKind` (6) plus `*IssueValues` for the interpolated raw values, and a pure
    `security_issue_text` / `mobile_issue_text` each as the SINGLE text source — the analysis
    calls it with `en=true`, the PDF presentation layer via `localized_message()` with the run
    language. The constructors `security_issue`/`mobile_issue` build `message` from it, so that no
    call site writes a message text by hand any more. `kind()` is derived from the already stored
    `header`/`issue_type` fields instead of being serialized as a fourth field, so that reports and
    cached artifacts of older builds keep localizing unchanged; unknown types fall back to the
    stored canonical version. JSON contract remains identical (`header`, `issue_type`, `message`
    unchanged English; `values` is additive and omitted when empty), secured by
    `security_issue_types_match_the_stored_contract`. Also corrected along the way: the mobile page
    script wrote the German context keys `sonstige`/`formular` into the supposedly
    English-canonical JSON — now `other`/`form`, localized via `mobile_context_label` (unknown keys
    pass through unchanged). The #578 special solution `coop_corp_verification_text` is gone, it
    was merged into `security_issue_text`. Guard tests per module (every variant localized, EN
    without umlauts) plus fallback and round-trip tests.
  - **Check inventory from the enum instead of the source text:**
    `tests/common/nonwcag_rule_inventory.rs` determined the canonical security check IDs by
    searching `src/security/mod.rs` for `SecurityIssue { header: "…", issue_type: "…" }` string
    literals — after the refactoring there were no literals there any more and the corpus test
    failed. The IDs now come from `security::all_security_check_ids()`, derived from
    `SecurityIssueKind::ALL`; the text scan along with the hand-maintained
    `CSP_DYNAMIC_DIRECTIVE_IDS` list is dropped, and the required CSP directives live in one place
    as `CSP_REQUIRED_DIRECTIVES`.
  - **Audit scope on the cover page (addendum of the same day):** The scope line stood only on
    page 2, the cover page showed "Website-Qualitätsbericht · casoon.de · 92" (Website quality
    report · casoon.de · 92) and a bare "Barrierefreiheit 100" (Accessibility 100) gauge — anyone
    who stops there reads both as a statement about the whole website. The line now comes from a
    common source (`single_report::audit_scope_line`) and is in the cover subtitle; the label above
    the gauge strip reads "MODULE · 0–100 IM AUTOMATISIERTEN PRÜFUMFANG · HÖHER IST BESSER"
    (MODULES · 0–100 WITHIN THE AUTOMATED SCOPE · HIGHER IS BETTER).
  - **Created and fixed a layout bug along the way:** The first attempt appended the scope line to
    the full kicker, the subtitle wrapped onto two lines and pushed the labels of the lower gauge
    row (Mobile, UX, Journey) off the page — the Typst template gives every gauge label a fixed
    22pt box and the cover page does not grow, the overflow was silently discarded. Discovered via
    the rendered PDF, not visible in the Typst source. `narrative-cover-kicker` is therefore now
    short ("Technischer Website-Check" (Technical website check) instead of "… mit Fokus auf
    Accessibility, SEO und Performance" (... with a focus on accessibility, SEO and performance))
    — the gauge strip directly below names the dropped areas anyway. New guard
    `test_cover_subtitle_stays_on_one_line` (character budget 95, measured: ~83 fit, 132 wrapped)
    plus `test_cover_carries_scope_line_and_gauge_qualifier`.
  - **Deliberately not changed:** the score inflation.

- **Product documentation: contradictions with actual behavior corrected, 2026-09-17:** Three
  confirmed discrepancies fixed. (1) `README.md`'s HTML5 conformance description still said
  "score-neutral", although the module has really flowed into the overall score since the
  per-cause scoring fix of 2026-09-16 (`contributes_to_overall: true`, `weight_pct: 5`). (2)
  `src/lib.rs`'s module overview listed "HTML" as a report format, although no HTML export exists.
  (3) The short description said in several places (`Cargo.toml`, `src/lib.rs`, `src/cli/args.rs`,
  `src/cli/plan.rs`'s runtime CLI banner, `CLAUDE.md`, `docs/ARCHITECTURE.md`) a blanket "WCAG 2.1",
  while `README.md` already correctly labels individual WCAG 2.2 rules (2.4.11/2.4.12, 3.3.7,
  2.5.8) separately — unified everywhere to "WCAG 2.1 AA (+ select 2.2 criteria)". In the process,
  one more, directly adjacent finding was corrected: `src/lib.rs`'s "Comprehensive: Checks
  WCAG 2.1 Level A, AA, and AAA" contradicted `README.md`'s own statement "AAA is not fully
  implemented yet" — now precise: 36 of 50 automated A/AA criteria
  (`docs/PARITY_CONTRACT.jsonc`), plus selected 2.2 and individual AAA criteria.
  `docs/browser-architecture.md`'s outdated Homebrew formula example was deliberately left
  untouched — the file already declares itself an old, not fully implemented collection of
  proposals, not current behavior documentation. Three new, machine-checkable regression guards in
  `scripts/release-check.sh` (unqualified "WCAG 2.1 Accessibility Checker" phrase, HTML5
  conformance "score-neutral", HTML as a report format), each verified with a manual negative case
  (old text temporarily restored → check fails; reverted → green again).
- **Release workflow: quality gates enforced before the artifact build, 2026-09-17:**
  `.github/workflows/release.yml`'s `build` job previously depended only on `verify-version` —
  tests, formatting, Clippy and the dependency advisory gate were not a prerequisite for a
  `v*` tag release. `.github/workflows/ci.yml` gets an additional `workflow_call:` trigger
  (alongside `push`/`pull_request`), `release.yml` now calls it as a new job `quality-gates` via
  `uses: ./.github/workflows/ci.yml` on the same commit SHA, which `build` now has as a
  prerequisite in addition to `verify-version` — technically secured via the SHA (same checkout),
  not just via the branch name. `browser-smoke`/`coverage` deliberately do not run again in this
  (their own `if: github.event_name == 'push' || ... == 'pull_request'` already excludes
  `workflow_call`) — the Chrome/corpus tests already ran at the merge to `main` and run again
  locally via the pre-push hook (`scripts/release-check.sh`) before a release tag is even pushed;
  repeating them in the release gate would have slowed down every release unnecessarily. Both
  platform build steps get `--locked`, so that releases are built reproducibly from the checked-in
  `Cargo.lock`. Two new `release_contract_tests`
  (`test_release_build_requires_quality_gates_to_pass_first`,
  `test_release_build_uses_locked_lockfile`) prevent the release job from running into the
  artifact build without a gate in the future — negative case verified (`needs:` reset to
  `verify-version` only → test fails).

- **Dependency security: RustSec gate added, 2026-09-17:** `cargo audit` reported 9
  vulnerabilities in `Cargo.lock`. Patch-level updates without API impact were applied:
  `rustls` 0.23.40 → 0.23.45 (RUSTSEC-2026-0285), `crossbeam-epoch` 0.9.18 → 0.9.21
  (RUSTSEC-2026-0204), `quinn-proto` 0.11.14 → 0.11.18 (RUSTSEC-2026-0185). `lopdf`
  (dev dependency, PDF test helper) was raised from 0.34 to 0.45 (RUSTSEC-2026-0187) — the API
  in use (`Document::load_mem`, `.objects`, `.catalog()`, `.get_pages()`,
  `.extract_text()`) stayed unchanged across the version jump, all PDF tests still green.
  Two advisories remain open and are documented individually with a justification in
  `.cargo/audit.toml` instead of being blanket-ignored: `rsa` (RUSTSEC-2023-0071, only via the
  optional `ai-transparency`/`c2pa` chain, no upstream patch available, auditmysite holds no private
  RSA key) and `quick-xml` (RUSTSEC-2026-0195/-0194, two transitive copies via
  `hayagriva`/`syntect`, each already at the version ceiling allowed by the parent packages —
  the fix requires an upstream release of the `typst`/`renderreport` chain; the affected inputs
  are bundled, non-attacker-controlled data such as citation styles and theme `.plist` files,
  not the content of audited web pages). A new CI job `dependency-audit` (`.github/workflows/ci.yml`)
  runs `cargo audit` on every PR.
- **Screen reader module: reading order counted layout artifacts, 2026-09-17:** Triggered by the
  review of a RankingLab report (Sachsen-Anhalt): the section "Screenreader-Lesereihenfolge"
  (screen reader reading order) reported 237 "angekündigte Knoten" (announced nodes) and four
  findings "Langer Abschnitt ohne Landmark, Überschrift oder Fokusziel" (long section without
  landmark, heading or focus target) — with heading and landmark quality at 100/100 at the same
  time. Reproduced exactly locally at www.sachsen-anhalt.de. Cause: `linearize` emitted every
  non-ignored AX node as a `ReadingItem`, including 67 `InlineTextBox` — Blink's line fragments
  into which a `StaticText` is split by a line break. They exist for caret/selection bounds, are
  never handed to an assistive technology, and carried the same text two to four times. As a result,
  every number derived from the reading order was inflated: the desert heuristic counted a
  news teaser card made of four announcements (headline, date, teaser, image credit) as a
  23-entry desert; all four findings of the page were false positives, produced solely by the
  line break. New: `linearizer::is_layout_only_role` skips `InlineTextBox` when emitting,
  but keeps traversing (descendants are preserved); `analyzer::carries_announced_content`
  counts only items with their own name or value for the desert distance, so empty wrappers
  (`figure`, `paragraph`, `group`, `generic`, `Figcaption`, `list`) no longer count the same
  announcement twice — role-agnostic, a `group` with an `aria-label` still counts.
  `CONSENT_WALL_NODE_THRESHOLD` was recalibrated from 200 to 145, because the node count drops by
  the same proportion. Result at sachsen-anhalt.de: 237 → 170 nodes, four false positives gone,
  the one real finding (button without accessible name, 4.1.2 → § 12 No. 3 BFSGV) remains; tab stops
  and all three quality values unchanged. Verified by a before/after run and 4 new
  unit tests.
- **HTML conformance: score counted occurrences instead of causes, 2026-09-16:** The module was at
  `weight_pct: 0` with the report note that its HTML5 schema did not know RDFa/Open Graph attributes
  and `og:` meta tags therefore produced false positives. Both were outdated: the RDFa gap has been
  closed since `html-conform` 0.2.1 (`schema/html5/meta.rnc`, block "RDFa Lite Property
  Metadata"); checked directly against a page with `og:` tags — not a single such finding.
  The actual defect was the penalty formula `error_count * 10`: it counted per *occurrence*,
  not per *cause*. One faulty template element produces one occurrence per rendering,
  so every real page floored at 0 — at sachsen-anhalt.de, 201 of the 208 reported errors
  traced back to a single `<link as=…>` without `rel`, 6 distinct causes in total. New:
  `html_conform::defect_key` normalizes rule ID plus message (backtick values and
  `at line:column` removed) into one cause; `score_findings` penalizes per cause, with a
  capped accumulation surcharge (at most double the base) and rank damping (first cause full,
  second/third half, the rest a quarter), so that the score stays meaningful over the whole range
  instead of saturating at 0. Sachsen-anhalt.de thus goes from 0 → 64. The module counts toward the
  overall score again (5 %, moved over from SEO: SEO 20 → 15) and reports `measurement_type: "measured"`
  instead of `"heuristic"`. Report side: new field `distinct_defect_count` in the JSON, KPI
  "Ursachen" (causes) in the PDF, findings table one row per cause with a column "Vorkommen"
  (occurrences) (instead of 20 redundant occurrence rows), `ScoreCard` with the normal grade band again,
  the outdated RDFa note replaced by an explanation of the cause counting. Verified at sachsen-anhalt.de
  (JSON + `--debug-typ`) and with 8 new unit tests.
- **PDF abort due to foreign glyphs and panic in the HTML content model, 2026-09-16:** (1) A
  single icon-font character on the audited page aborted the entire PDF generation:
  link texts, selectors and custom property names carry private-use code points into the report,
  Typst falls back to `LastResort` and the PDF/UA export fails
  (`PDF/UA-1 error: the text "\u{e900}" could not be displayed`). Affected 8 of 84 domains of a
  RankingLab run (among them kit.edu, fu-berlin.de, uni-goettingen.de). New: `output/pdf/sanitize.rs`
  removes private-use characters from all strings of the `RenderRequest` (a single chokepoint after
  `builder.build()`, hence no field-by-field upkeep) and learns the remaining non-displayable
  characters from the compile error: discard the character, re-render, at most 8 rounds — this is how
  a `↳` from a CSS variable name was caught on jyu.fi. Presentation layer only; the canonical JSON
  keeps the original text so that selectors remain copyable. (2) `enclosing_tag_stack`
  (`wcag/rules/html_content_model.rs`) panicked on ikea.com because the byte offset of the external
  parser lay in the middle of a `\u{a0}` — contrary to its own promise "never panics". The offset is
  now reset to the character boundary, the raw-text close-tag comparison runs over bytes
  instead of `str` slices. Verified on the live domains plus two regression tests each.
- **1.3.1 fixes, 2026-09-16 (#581, #582, #583):** (1) The accessibility subcategory and
  security category scores from `d6daf7b` appeared only in the PDF, not in the JSON report — the
  JSON builder (`build_page`/`PageDetail`) never read them. They are now
  `pages[].detail.accessibility_subcategory_scores`/`security_category_scores` (`{ name, score }`,
  canonically English) in the single report, with a schema, registry and `OUTPUT_CONTRACT.md`
  entry; batch details stay compact and omit both. (2) The CI step for the four
  detection corpus binaries aborted at the first failure, the others never ran (this is what
  happened in the first 1.3.0 CI run) — now one step per binary with `if: !cancelled()`, and `release-check.sh`
  collects failures and aborts only at the end. (3) `Cargo.lock` held the deleted
  `spin 0.9.8` (via `lazy_static`) — raised to 0.9.9.
- **Blank page after empty report part fixed, renderreport 0.5.1, 2026-09-15:** the
  `pdf-smoke` test `test_single_pdf_technical_pages_are_not_blank_when_pdftoppm_is_available`
  failed: in the technical fixture, the part 3 divider page (with its own trailing `PageBreak`
  by then) was followed by no module content but directly by the appendix with its own `PageBreak` —
  with a blank page in between. The cause lay in renderreport: for
  `LayoutHint::AlwaysNewPage` (only `page-break` uses it) the engine wrote a hard `#pagebreak()` before every
  otherwise soft break component, so two adjacent breaks produced a blank page.
  renderreport 0.5.1 writes a soft break there (with a regression test that was red without
  the fix). Real reports were not affected in three live runs (standard, technical, without `--full`),
  because part 3 always has content there.
- **Finish batch progress before the report, 2026-09-15 (#580):** `run_batch_mode`
  (`src/cli/runners.rs`) called `finish_verdict` only after `output_batch_report` or
  `output_batch_as_single_reports` — in a TTY the active bar drew over the report, in
  a pipe the verdict line came after the report (regression against the #530 acceptance criterion).
  The order is now fixed in `BatchLifecyclePresenter::finish_then_render`, both
  branches use it. Regression test `lifecycle_is_finished_before_report_renders`; verified live via pipe
  with the reproduction from the issue (verdict line before the report, ANSI-free).
- **`CLAUDE.md`'s "Current State" history moved to `CHANGELOG.md`, 2026-09-06:** the file had grown to 1058 lines, of which 805
  were pure chronological history. `CLAUDE.md` now keeps only working rules + a short
  pointer here; new entries go here from now on, no longer into `CLAUDE.md`. Pure
  cut and paste, no entry changed in content.
- **Review batch fully completed, 2026-09-06 (items 7–11):** Item 7 (condense the management
  summary) was implemented as a "lighter condensation" at the user's request instead of exactly 2 pages:
  the strengths/weaknesses cards of `render_module_split_dashboards` showed the same 4-5
  weighted modules (Accessibility/Performance/Security/Mobile) for a third time (after the radar chart
  and the new score driver table from item 2) — now filtered to `measurement_type !=
  "measured"`, so it shows only the unweighted indicator modules (UX,
  Journey, Search Experience, …), including an adapted "all good" fallback message (scoped
  to "additional indicator modules" instead of "all modules" across the board). New leverage sentence on
  the action plan page ("Die N wichtigsten Ursachen erklären rund X % der erkannten
  Vorkommen — ihre Behebung hat die größte Hebelwirkung …", i.e. "the N most important causes explain about X % of the detected
  occurrences — fixing them has the greatest leverage …") — shares its computation with the diagnosis sentence from item 3 via a new shared `compute_problem_concentration`
  function (one number, two differently worded sentences on two pages, no duplicate text). Both
  changes verified by a `--debug-typ` smoke test (cards disappear correctly when no
  indicator modules are present; the leverage sentence computes "4 causes, 50 %, 8
  occurrences" correctly on the fixture). Items 8–11 (detection corpus tests in the release script, README
  accuracy claim, file size confirmation, version number + module list) were small,
  independently verified fixes.
- **Report review batch items 1–6 implemented, 2026-09-06 (external report review of
  inros-lackner-de):** seven distilled items, six implemented (item 7 — management summary
  4→2 pages — deliberately deferred, a layout decision needs separate user confirmation).
  **Item 1 (P0) uncovered a considerably larger bug than reported:** the reported case (a
  `landmark-unique` finding showed the wrong, generic WCAG 1.3.1 title "Fehlende
  semantische Struktur" (missing semantic structure) instead of "Landmarks nicht eindeutig benannt" (landmarks not uniquely named)) was a simple missing
  `explanations.rs` entry (#571 pattern). The root-cause investigation, however, uncovered a
  second, structural bug in `wcag_group_key` (`src/audit/normalized.rs`): every `axe_id`
  without its own `LEGACY_WCAG_MAP` entry falls back to the raw WCAG criterion ID as grouping
  key — if two *independent* checks share a criterion (e.g. several
  1.3.1 landmark checks, several 4.1.2 ARIA checks), their violations blend into a
  single finding with a summed occurrence count and the title of whichever hit came first. A
  systematic scan of `src/wcag/rules/*.rs` found **43 affected axe_ids across 10 WCAG
  criteria** (1.1.1, 1.3.1, 1.4.4, 2.1.1, 2.4.1, 2.4.4, 2.4.7, 3.3.1, 4.1.1, 4.1.2) — not just
  the 14 axe_ids under 1.3.1 that the reported case suggested. Every affected axe_id got
  its own `taxonomy::rules::Rule` entry + `LEGACY_WCAG_MAP` entry; 32 of them additionally
  got a new bilingual `RuleExplanation`, 3 (`region`, `focusable-no-role`, `link-name-context`)
  already had a matching explanation and only needed the missing rule/map entry. Two
  axe_id string collisions between independent checks were discovered and resolved (renamed, since
  a global `LEGACY_WCAG_MAP` entry would otherwise have wrongly merged both checks):
  `labels.rs`'s 4.1.2 check `"label"` → `"control-missing-label"` (collision with the deliberately shared 3.3.2 `"label"` of `instructions.rs`/
  `form_rules.rs`), `link_purpose_link_only.rs`'s 2.4.9 check
  `"link-name"` → `"link-name-only"` (collision with the 2.4.4 `"link-name"` of `link_purpose.rs`). An
  already existing test (`tests/wcag_coverage.rs::no_undocumented_severity_collisions_in_group_key_mechanism`,
  part of the QA-009 follow-up) confirmed the fix independently: it required removing 5 of the 6
  documented `ALLOWED_MIXED_SEVERITY_GROUPS` exceptions, because their severity mix
  no longer occurs after the fix — the remaining `3.3.2` exception is a deliberately shared
  axe_id (not a grouping bug). **Item 2** (score driver table): new `ModulesBlock.module_scores`
  (raw from `NormalizedReport.module_scores`, not from the `dashboard` reshaped for the radar cards,
  since e.g. the "Sichtbarkeit & Nutzerverständnis" (visibility & user understanding) card merges SEO with heuristic
  AI visibility signals into a composite score that is not the raw SEO score with
  SEO's real weight) feeds a new table directly after the quality profile radar:
  module/score/weight/classification ("Größter Schwachpunkt" (biggest weak point)/"Erheblicher Risikotreiber" (significant risk driver)/
  "Stabilisiert Gesamtwert" (stabilizes overall value)), classified by `(100-score)*weight_pct` drag instead of raw score (a bug
  found and fixed by a live smoke test: the first version sorted by score with
  weight only as a tie-breaker and wrongly picked the low-weighted Security 30 instead of
  the highly weighted Accessibility 40 as the "biggest weak point" — exactly the case the
  review named as an example). **Item 3**: new generated diagnosis sentence ("N Vorkommen erkannt,
  rund X % auf 4 wiederkehrende Ursachen zurückführbar, besonders betroffen: …", i.e. "N occurrences detected,
  about X % traceable to 4 recurring causes, particularly affected: …") directly after the
  score driver table, scope deliberately Accessibility+SEO combined (user decision). **Item
  4**: "Root Cause" wordings unified to a more cautious phrasing ("X Vorkommen
  folgen demselben Muster — deutet stark auf … hin, bestätigt erst mit Belegen von weiteren
  Seiten", i.e. "X occurrences follow the same pattern — strongly suggests …, confirmed only with evidence from further
  pages"), and in doing so an independent #406 localization leak was found and fixed
  (`src/output/pdf/findings.rs`: the "Root Cause" card label was hard-coded English, even in
  German reports) and an identical hard-coded duplicate in `builder/batch.rs`
  was fixed along with it. Batch reports' `TemplateCluster::confidence` (`confirmed`/`likely`) already met the
  question posed for a stronger, actually justified wording given multi-page evidence
  — no change needed. **Item 5**: new scoring subsystem, because contrary to the
  assumption in the item no real 0–100 subcategory score existed yet (only a
  taxonomy categorization for labels and a heuristic top-3 text). Accessibility:
  `compute_accessibility_subcategory_scores` partitions the same raw `Violation`s by
  `taxonomy::Subcategory` (via the already existing `wcag_group_key` resolution) and runs
  the real `AccessibilityScorer::calculate_score` per subcategory subset — no newly
  invented penalty scheme. Security has no WCAG-equivalent rule registry; the new
  `security::SecurityCategory` (response headers/transport security/access policies/
  third-party exposure) groups `SecurityIssue.header` strings and uses the same
  severity→penalty mapping as `calculate_security_score`. The PDF shows "Kritische Schwächen: …
  · Vergleichsweise stabil: …" (critical weaknesses: … · comparatively stable: …) (threshold: the existing `FIVE_BAND` cutoff at 60, no new
  threshold invented), per module only if both sides are non-empty. **Item 6**: new
  `RoadmapItemData.leverage` ("Sehr hoch"/"Hoch"/"Niedrig", i.e. very high/high/low; fixed rule from `occurrence_count`
  ≥10 → Sehr hoch, else ≥5 occurrences or quick fix + high priority → Hoch, else Niedrig — the
  threshold of 10 deliberately mirrors the existing `FindingGroup::is_component_issue` threshold) as a
  new line in the action plan card text, wording "betroffen" (affected) instead of "behoben" (fixed) (consistent with
  item 4). All six items verified by `cargo test`/`--no-default-features`/`--all-features` +
  `cargo clippy --all-targets --all-features -D warnings` + several `--debug-typ` smoke tests
  against enriched fixtures (1368 tests in the full `pdf_test` feature run, 0
  failures).
- **CI permanently red for several weeks — real cause found and fixed, 2026-09-06
  (repo cleanup session):** while cleaning up, `gh run list` was checked (never done before — locally
  only `cargo check --all-features` was ever verified via the pre-push hook, never the actual
  CI status) and it turned out that practically every push since at least 2026-07-16 failed in CI.
  Cause: the test `astro_structured_data_exports_work_...` in `src/seo/schema.rs` loads
  its fixture via `include_str!("../../tests/fixtures/astro_structured_data_components.json")`
  — this file existed on disk locally but was **never tracked in Git** (caught by `.gitignore`'s
  blanket `*.json`, no whitelist exception as for the other fixture directories).
  `cargo check` (the pre-push hook) does not compile test targets and never noticed it; every
  `cargo test`/`cargo clippy --all-targets` run in CI, on the other hand, aborted on it with a hard
  `include_str!` compile error before a single test ran — affecting practically all
  CI jobs (Check & Test, PDF Smoke Tests, Coverage). Same class of error as the
  `detection_corpus_nonwcag/*/*.html` gitignore gap found earlier the same day (symptom: a fixture file present locally
  but never committed), but here with real CI impact instead of merely "would get lost
  the next time one is added". `.gitignore` was extended by `!tests/fixtures/astro_structured_data_components.json`,
  and the file was checked in. In the process `tests/wcag_fixtures/*.html` (7 files, used by
  `tests/coverage_matrix.rs` via a runtime directory scan, no `include_str!` — hence
  no CI blocker, but the same tracking gap) was fixed as well. Verified locally with the exact CI commands:
  `cargo clippy --all-targets --all-features -- -D warnings`,
  `cargo clippy --no-default-features -- -D warnings`, `cargo test --no-default-features` —
  all green.
- **Repo consolidation, 2026-09-06:** removed an orphaned worktree + branch `worktree-agent-*`
  (0 own commits that were not already in main — a pure leftover of an old
  `isolation: "worktree"` agent run). `archive/main-history` (local + origin) deleted at the explicit
  request of the user, only `main` exists now. Stray files in the project root removed
  (`delete.html`, a leftover Shopify page dump; a separate leftover test JSON from a
  previous session). `reports/` (68 MB) and `tmp/pdfs/` (local, gitignored test report outputs)
  emptied completely at the user's request — regenerable, no Git tracking affected. Local
  `target/` folder remnant cleaned up (`CARGO_TARGET_DIR` actually points to `~/.cargo/target`,
  see `reference_cargo_target_dir.md`).
- **Non-WCAG detection corpus for vulnerable_libs/schema_rules filled, 2026-09-06
  (remaining scope of #558 completed — the security domain was already done):** both domains
  need a real Chrome/CDP (unlike security, which is only an HTTP fetch), hence two new
  `#[ignore]`-gated tests analogous to `tests/detection_corpus_test.rs`.
  **vulnerable_libs (8/8 covered):** `tests/vulnerable_libs_detection_corpus_test.rs` diffs
  the real output of `analyze_vulnerable_libraries` against 4 combined HTML fixtures (no more
  needed, since minimal global stubs suffice — `window.jQuery.fn.jquery` etc., no real
  library functionality necessary). Split into 4 instead of 2 files because of a real
  constraint documented by the module itself: Lodash and Underscore share the global `_`
  and would overwrite each other if used together. Prototype/MooTools are flagged
  unconditionally as vulnerable in the code (unmaintained), so they have only a violation
  fixture, no pass fixture. **A real, pre-existing bug was found and fixed in the process:**
  `.gitignore`'s `*.html` block had an exception for `tests/fixtures/detection_corpus/*.html`
  (WCAG corpus), but none for the equivalent `detection_corpus_nonwcag/*/*.html` — new
  fixtures in this domain would have been present locally but never checked in, unnoticed.
  **schema_rules (17/17 covered):** before building the full corpus it was checked and found that
  the pure `assess_node` rule logic of `schema_rules.rs` already has very thorough pure-Rust
  unit tests for 14 of 17 features (edge cases such as EventRescheduled, remote jobs,
  merchant-vs-editorial context) — presented to the user, who nevertheless decided on the full
  Chrome corpus (tracking consistency with the security/vulnerable_libs domain mattering more than
  avoiding redundancy). `tests/schema_rules_detection_corpus_test.rs` diffs the real output of
  `detect_structured_data` against 2 combined fixtures (all 17 types
  complete/valid → expected "pass"; the 11 types that can have required fields at all,
  incomplete → expected "violation"). `merchant_listing` deliberately expects `needs_review`, not
  pass/violation: `detect_structured_data` always evaluates rules with
  `ProductRuleContext::Indeterminate`, the real page intent context is only set later by the `derive` step of
  `seo::module` via `schema_fit`/`refresh_rule_assessments` — this is
  real, correct behavior of this entry point, not a test gap. Five features
  (Article/Organization/WebPage/WebSite/Person) have no required fields in `assess_node` and
  can structurally never become "violation" — tested as "pass" in the "complete" fixture, no
  separate violation fixture needed. Both new tests were verified with a deliberately falsified
  `expected.json` to show that they detect a real diff (reset afterwards).
- **Two new best-practice ARIA hygiene rules, 2026-09-06 (last item from the
  current Insights article batch):** `redundant_role.rs` (`4.1.2/redundant-role`, e.g.
  `<button role="button">`, `<a href="..." role="link">` — fixed, context-independent
  tag-to-implicit-role table; tags with an ancestor-dependent role such as `<header>`/`<footer>`
  deliberately excluded) and `fake_navigation_link.rs` (`4.1.2/link-as-button`, `<a href="#"
  onclick="...">`/`href="javascript:..."` as a button substitute — only inline `onclick`, the same
  boundary as `click_handlers.rs`). Both `Severity::Low`, `WcagLevel::A`, tag `"best-practice"`,
  registered in the normal `PageRuleEntry` mechanism (`page_rules.rs`), no new
  infrastructure. **Two real bugs found and fixed during live verification, not just
  wording:** (1) the first version built selectors manually from tag+id+class (as `click_handlers.rs`
  does) — with several same-named tags without id/class (e.g. two `<a>` elements without a
  class) both collided on the same selector string `"a"`; not reproduced in the fixture at
  first and thus unnoticed, but real: the doc comment of `js_helpers::CSS_SELECTOR_JS` itself
  explicitly warns about it ("Never falls back to a bare tag name"). Switched to the more robust, already
  existing `__amsCssSelector` helper (like `aria_prohibited_attr.rs`), verified live:
  `body > a:nth-of-type(1)`/`:nth-of-type(3)`/`:nth-of-type(4)` now remain
  distinguishable. (2) Naive string concatenation of `CSS_SELECTOR_JS` (a top-level
  `function` declaration) directly followed by the rule's own standalone `(function(){...})()` produced
  a real live error (`TypeError: (intermediate value)(intermediate value)... is not a
  function`) — `Runtime.evaluate` parses a leading `function` token at expression position as a
  function expression instead of a declaration, so the following IIFE is
  interpreted as a function call argument instead of a standalone statement. Fixed following the same pattern as
  `aria_prohibited_attr.rs`: embed both in an explicit outer `(function() { ... })()`,
  instead of concatenating two independent top-level snippets — missed on the first attempt because
  `click_handlers.rs` (the original model) does not use `CSS_SELECTOR_JS` at all and this
  concatenation problem therefore never occurs there. Verified live against a local `file://` fixture
  (5 test cases: 3× redundant role, 2× fake navigation, 2× correctly unflagged
  legitimate cases) — both rules fire exactly as expected, no false positives. Two
  hard-coded page rule counter regression tests (`level_a_filter_includes_only_level_a_rules`,
  `level_aa_filter_includes_aa_plus_a_rules`, `page_rules.rs`) were counted up accordingly
  (36→38, 47→49). No new detection corpus fixture entry (#556 format) — in line with the
  existing precedent of pure DOM-JS rules without an offline unit test in this module.
- **4.1.3 deepening — late live region insertion detected, 2026-09-06:** new
  `InteractiveFindingKind::FormErrorLiveRegionLateInsertion` (Severity::Low, advisory) in
  `src/a11y_journey/form_error.rs` — an extension of the existing form error journey instead of a new
  detection path, as required: `check_error_state` already captured `live_before`
  (before submit) and `live_after` (after submit); `new_live = live_after && !live_before` had
  so far been used only internally to derive `FormErrorInvalidWithoutLiveRegion`, never reported
  as a finding of its own. New check: when `new_live` is true (the live region announced correctly
  but was not in the DOM initially), this is now reported as a standalone advisory finding
  — clearly separated from the existing "no announcement at all" case
  (`FormErrorInvalidWithoutLiveRegion`/silent failure, which both require `!new_live`, so
  never fire at the same time as the new finding) and from the polish conflict check in
  `status_messages.rs` (unchanged). Deliberately advisory/Low instead of High: the announcement worked
  in this run, it is a robustness recommendation for other browser/AT combinations,
  not a confirmed error. Checked live against casoon.de (home page + /kontakt) — no
  form error journey candidate was detected there (no matching form/submit structure on these two
  pages), so the new code path is verified only by build/clippy/the full test suite (1318 tests) and
  the EN locale umlaut guard, not by a live observed hit — like all
  other `FormError*` findings in this file, `form_error.rs` has no test infrastructure of its own
  (Chrome `Page` interaction, no mocking precedent in this module).
- **Concrete AT self-test instructions instead of a generic "check manually", 2026-09-06:**
  new `manual_recheck_instruction(wcag_criterion, en)` (`src/output/pdf/helpers.rs`), keyed by
  WCAG criterion, with VoiceOver rotor / NVDA elements list shortcuts for the four criteria for which
  a layperson can verify a suspicion themselves in a few minutes: 1.3.6/1.3.1
  (landmarks/headings via rotor VO+U or Insert+F7), 3.3.2 (form field labels via
  VO+Cmd+J/NVDA F), 4.1.3 (error message announcement — trigger the message, check whether it is announced automatically without
  a focus jump). Deliberately no new JSON structure/no kind enum — pure
  PDF presentation text (analogous to `render_manual_only_criteria_note`), appended at three already
  existing places instead of a new appendix detached from the finding: the
  screen reader findings table (`detail_modules/accessibility.rs`, keyed on `issue.wcag_criterion`),
  the WCAG violation finding card (`findings.rs`, new "Selbsttest" (self-test) row in the
  action assessment block, keyed on `group.wcag_criterion`) and the
  manual check points checklist (`single_report.rs`, appended to the `recommendation`/`text`
  string, keyed on `finding.rule`). `1.3.1` deliberately covers both heading and
  landmark duplicate findings with one shared text (both sub-cases share the same
  `wcag_criterion` constant in the code and are checked in the same VoiceOver rotor/NVDA elements list tool,
  no artificial separation necessary). Verified live against casoon.de (`--debug-typ`): the
  landmark instruction appears correctly at every 1.3.6 screen reader finding.
- **Fixed list "Strukturell nur manuell prüfbare Kriterien" (criteria structurally checkable only manually) in the PDF, 2026-09-06:** new
  `render_manual_only_criteria_note` (`src/output/pdf/single_report.rs`), directly after the
  WCAG coverage section, unconditional (no `--annex` flag, same report-level gate as the
  coverage section itself). Five fixed, page-independent rows (2.4.3 focus order, 3.3.1
  error identification, 1.4.4/1.4.10 zoom & reflow, 1.1.1 alt text correctness, 1.2.1/1.2.2
  video content summary) — deliberately NOT derived from the findings of this page like the
  EN 301 549 / BIK annex, because precisely the third, "invisible" confidence category from the
  Insights article would otherwise remain completely invisible in a 0-findings report. Verified live
  against casoon.de (`--debug-typ`): renders correctly, without a status dot (no
  good/warn/bad verdict, a pure scope statement). Discovered while implementing: there is already a
  separate, similarly named "So testen Sie manuell" (how to test manually) section (`src/output/pdf/wcag_coverage.rs`)
  with AT test instructions (keyboard navigation, screen reader, 400% zoom, reduced motion, …) —
  a different function (instruction "how to test" instead of explanation "why structurally not automatable"),
  deliberately not merged, but a relevant anchor point for the AT self-test instructions.
- **"Process instead of certificate" sentence added to the disclaimer, 2026-09-06:** the check showed
  that the scope caveat itself ("kein vollständiger Konformitätsnachweis" (no complete proof of conformance), "ersetzt keine
  manuelle Prüfung" (does not replace a manual check)) was already present in the existing `disclaimer` text (`build_methodology`,
  `src/output/builder/single/methodology.rs`) and in the cover label ("WCAG-Vorkommen" (WCAG occurrences) instead of
  blanket "Accessibility-Befunde" (accessibility findings)) — both places left unchanged.
  What was missing was the process statement emphasized in the article: accessibility as an ongoing task,
  not a state that can be reached once. One sentence appended to the existing `disclaimer` (DE+EN,
  no new callout component, no change to `gate_certificate_by_risk`/`CERTIFICATE`
  thresholds). Deliberately not placed on the cover — that is a fixed, concise dashboard page
  (renderreport `CoverPage`), scope/legal texts consistently live in the methodology chapter in this project,
  not on the cover.
- **Three-tier confidence labelling closed as already satisfied, 2026-09-06:**
  the item derived from an Insights article wanted to introduce a new `DetectionConfidence`
  enum (`Certain`/`HeuristicSuspicion`/`ManualOnly`) plus a registry classification per `rule_id`
  and a new PDF badge. Checked during implementation and rejected: `NormalizedFinding`
  already carries `confidence`/`false_positive_risk`/`verification` (heuristic per rule via
  `derive_confidence`/`derive_false_positive_risk`/`derive_verification`,
  `src/audit/normalized.rs`), and every finding card in the PDF shows **unconditionally**
  "Erkennungssicherheit"/"Falschpositiv-Risiko" (detection confidence/false-positive risk) in the "Prüfmetadaten" (verification metadata) block
  (`src/output/pdf/findings.rs`) — not only for suspected cases. A first attempt to additionally show the
  existing `verification` field (derived 1:1 from `false_positive_risk`) symmetrically as another row
  was discarded again: it would have shown the same
  information a third time in other words on the same card — real redundancy
  instead of insight, the opposite of what `report-lint`/`report-critic` are meant to prevent in this
  project. The third tier "decidable only contextually" (media alternative,
  contrast on image background) already runs visibly separated under the PDF section
  "Manuelle Prüfpunkte und heuristische Hinweise" (manual check points and heuristic notes) (`render_assessment_and_execution_notes`) with its
  own "Manuelle Prüfung" (manual check) label. No code change — on close comparison the item was already fulfilled by
  existing, differently named infrastructure. The item's text also
  contained an internal contradiction (point 1 vs. point 4 on classifying
  media alternative as `ManualOnly` vs. `HeuristicSuspicion`), which was clarified with the user
  (point 1/`ManualOnly` is correct).
- **`build_id` gets a `-dirty` suffix for an uncommitted working state (2026-09-05):**
  `build.rs` previously read the `build_id` only from `git rev-parse --short HEAD` — as long as work is done uncommitted (as is
  usual in this project), the embedded SHA stayed identical across several
  actually different builds and made two live reports with
  different fix states indistinguishable (confirmed live on three reports from 2026-09-05,
  all with `build_id: "be9aed2"` despite changes committed since). New
  `is_dirty()` check (`git status --porcelain`, same no-git fallback as in the
  SHA lookup — error/no git ⇒ not dirty, no hard failure) appends
  `-dirty` for an uncommitted state, e.g. `44e7480-dirty`. Since the dirty status can change on any
  working tree change, not only on a HEAD/ref change, the existing
  `rerun-if-changed=.git/HEAD` alone is not enough — an additional `rerun-if-changed` on a
  guaranteed non-existent path forces Cargo to re-run the build script on every build
  (verified empirically against a scratch project: 3 builds → 3 script runs).
  Verified live against the deliberately uncommitted working state itself (`strings`
  on the built binary shows `44e7480-dirty`).
- **Verified: 1.3.6 finding count on casoon.de is no counting error (2026-09-05):** a
  live report had shown 200 occurrences for WCAG 1.3.6 on a single page, logged as a low-priority
  observation without suspicion of a concrete bug. Verified against a fresh live run (`--format json` against www.casoon.de): no
  double-counting bug across desktop/mobile viewport (`build_sr_audit_report` runs only once per report, on a
  single AXTree) and no real discrepancy through the `is_real_node_id` sanitizing filter
  (`analyze_reading_sequence` in `analyzer.rs` removes empty/synthetic IDs from
  `affected_node_ids` **after** the analysis — message texts such as "18 entries" deliberately count the
  unfiltered raw set, while `affected_node_ids.len()` can be smaller afterwards, e.g. 8).
  The Structure chapter of `derive_bik_chapters` sums `issue.affected_node_ids.len()` over **all**
  `1.3.6`-tagged findings of a page (the identical, already existing aggregation pattern as in
  every other chapter/criterion) — on casoon.de these are 14 separate "long section without landmark/heading/focus target" locations found by
  `detect_announcement_deserts`,
  whose filtered `affected_node_ids` lengths add up to exactly 200. A real, verified
  structural property of the page (many long content sections without an orientation point in between),
  not a tool bug — no behavior change needed. Clarifying comment added in `bik_guide.rs`
  so that future reviewers do not misread the summation as a counting error again.
- **#406 fix: NotTestable/warning findings showed raw English text in the PDF (2026-09-05):** a localization violation
  confirmed live in DE reports (casoon-de,
  satower-mosterei-de, inros-lackner-de) — `render_assessment_and_execution_notes`
  (`src/output/pdf/single_report.rs`) called only `get_explanation(rule_id)` for `wcag.not_testables`
  (axe_id lookup, e.g. `"video-caption"`) and, on a miss, fell back to the raw canonical English
  `finding.fix_suggestion` instead of the safe localized fallback sentence; `wcag.warnings` (contrast on image/gradient
  background, 1.4.3) did not attempt a `get_explanation` lookup at all. Fix: two-stage lookup
  (`rule_id` first, then `rule`/WCAG ID as a fallback — covers both axe-id override entries
  such as `"region"` and the new WCAG ID entries) in both loops, with the
  existing localized fallback sentence as the last step instead of `finding.fix_suggestion`.
  Four new `explanations.rs` entries for the previously completely missing 1.2.x video family
  (1.2.1, 1.2.2, 1.2.3, 1.2.8). **A real side effect discovered in the process was fixed:**
  the "Videos" chapter of `wcag::bik_guide::derive_bik_chapters` showed `NoFindingsDetected` although
  a real, PDF-visible 1.2.2 finding existed — `findings[]` (`NormalizedFinding`) is built
  exclusively from `wcag_results.violations` (`audit::normalized::normalize`), never from
  `warnings`/`not_testables`, and video checks almost always resolve to exactly these two kinds.
  `derive_bik_chapters` now additionally takes `&[AccessibilityAssessment]`
  (`NormalizedReport.accessibility_assessments`, an already existing carrier for exactly this
  data, available in every report path) and additionally feeds the videos chapter from it;
  `normalize_assessments` was raised from private to `pub(crate)` for this. New regression tests:
  EN/DE fallback guard in `src/output/pdf/tests.rs` (fictitious rule without an `explanations.rs`
  entry) as well as two new `bik_guide` tests (manual-review-only finding appears in the
  videos chapter; an unrelated criterion does not leak in).
- **New `html_conform` module: HTML5 specification conformance via the `html-conform` crate,
  2026-08-30:** a new, **standalone** module (`src/html_conform/`) that uses `html-conform`
  (crates.io, pure Rust, no network/subprocess) to perform browser-style HTML5 tree construction,
  full W3C RelaxNG schema validation, Schematron co-constraints as well as import map /
  speculation rules JSON and CSP enforcement checks — considerably deeper than the
  existing, unchanged `html5ever` parse error check in `seo::page_health`
  (`validate_html_locally`). **Runs as part of `--full`** (`check_html_conform: full_audit`,
  no CLI flag of its own), only on the mobile viewport pass (analogous to SEO/Mobile/DesignQuality),
  and **feeds into the score** (not score-neutral like `design_quality`/`ai_transparency`):
  its own `ModuleScoreEntry` with 10 % weight, funded by a proportional reduction of the
  five previous weights (Accessibility 40→36, Performance 20→18, SEO 20→18, Security 10→9,
  Mobile 10→9). Like `security`, a standalone top-level field on `AuditReport`
  (`html_conform: Option<HtmlConformAnalysis>`, URL/page level instead of viewport experience) and
  applied to `overall_score` outside the viewport blend (`ScoreBreakdown.html_conform_score/
  _weight_pct`, analogous to `security_score`/`security_weight_pct`). `rule_id`/`message` remain
  an opaque canonical-English passthrough (`html-conform`'s rule set is open, not a
  small enum, and the messages are third-party running text) — deliberately **not** the #406
  kind-enum pattern, same precedent as `best_practices::console_errors`/`vulnerable_libs`.
  On a technical setup error or HTML extraction error: `checked: false` / `score: 100`
  ("not measured", excluded from the weighted overall score) instead of a punitive 0,
  analogous to Performance's `metrics_available == 0` handling. Full JSON/PDF
  integration (`ModuleBlob.html_conform`, `HtmlConformPresentation`, its own PDF renderer with ScoreCard/
  MetricStrip/AuditTable). Batch PDF narrative arms (`batch_report/sections.rs`) and
  URL matrix column deliberately **not** part of this change (the score already counts automatically into
  every batch aggregate) — same precedent as the accepted `commerce`-has-no-PDF gap.
- **New `ai_transparency` module: C2PA image provenance check (EU AI Act Art. 50), 2026-07-31:**
  a new, **opt-in** (`--ai-transparency`) and **score-neutral** module that reads embedded C2PA
  ("Content Credentials") manifests on `<img>` elements of the audited page and —
  if the manifest declares AI/algorithmic generation (IPTC `digitalSourceType`
  `trainedAlgorithmicMedia`/`compositeWithTrainedAlgorithmicMedia`/`compositeSynthetic`/
  `virtualRecording`/`trainedAlgorithmicData`) — produces a manual-review advisory finding.
  Deliberately **report presence only**, no attempt to automatically detect a visible label near the image
  (the same fragility problem as chat widget disclosure detection,
  hence taken out of scope), and deliberately **no EXIF software tag fallback** (C2PA is the
  standard mechanism referenced by the AI Act, EXIF would be a second, unreliable
  detection logic). **Single-URL mode only** (`PipelineConfig.check_ai_transparency =
  args.ai_transparency && args.url.is_some()`, exactly the `capture_element_evidence` pattern):
  a batch run would do a network fetch + C2PA parse per image, only for
  `build_batch_detail()` to discard the entire module blob per page anyway (#256).
  **Double opt-in:** new Cargo feature `ai-transparency` (`c2pa = "0.90.3"`,
  `default-features = false, features = ["rust_native_crypto"]` — no native OpenSSL, no
  HTTP client features, so that the reader structurally never fetches or phones home by itself), not part
  of `default` (precedent: `pdf` feature) — **plus** the runtime flag. If the feature is missing at
  build time but the flag is set, `check_ai_transparency_feature` in `src/cli/runners.rs` aborts
  loudly before pipeline start with a `ConfigError` (rebuild hint) instead of silently remaining a no-op.
  **SSRF hardening** (the first fetch in this codebase to foreign origins potentially controlled by the audited page
  — unlike `seo::robots`/`security` with their pure same-origin fetch):
  own DNS resolution + IP range check (private/loopback/link-local/multicast/IPv4-mapped IPv6,
  explicit range checks instead of newer `Ipv6Addr` methods that may not be stable everywhere yet) BEFORE
  every fetch, plus `reqwest::ClientBuilder::resolve()` pins the connection to exactly the
  checked IP (prevents a second, unchecked DNS lookup at the actual
  connect — DNS rebinding TOCTOU). Manifest evaluation uses the **typed** `c2pa` API
  (`Manifest::find_assertion::<Actions>(labels::ACTIONS)` → `Action::source_type()`/
  `.software_agent()`, `Reader::validation_state()`), no blind JSON string walk — more robust
  against schema drift and running-text false positives. Localization follows the established
  kind-enum pattern (#406): `finding_message_text(rule_id, en)` is the only text source. New local
  enums `AiProvenanceKind`/`ManifestValidation` mirror `c2pa::DigitalSourceType`/
  `ValidationState`, so that `src/ai_transparency/mod.rs` (types, JSON/PDF integration) compiles **without** the
  Cargo feature — only `image_provenance.rs` (the actual fetch+parse) is
  `#[cfg(feature = "ai-transparency")]`-gated; module registration in `AuditCatalog`, the `ModuleData`
  variant, `ExperienceSection.ai_transparency`, `ModuleBlob`/PDF dispatch remain unconditional
  (smaller diff than originally planned, no `#[cfg]` sprinkled across catalog/report/output).
  Test fixtures for the C2PA parser are **synthetically self-generated** at test time
  (`c2pa::Builder` + `EphemeralSigner`, signed into a hard-coded 1×1 PNG) instead of vendored from the
  public C2PA test files (`c2pa-org/public-testfiles`, CC-BY-SA-4.0) — those
  contain no samples clearly marked as AI-generated (only Adobe editing
  conformance tests from 2022), and a synthetic fixture avoids any license/
  attribution question. **A real side effect discovered in the process:** `c2pa` unconditionally enables
  (hard-wired in its own `Cargo.toml`, not behind one of its optional features)
  `serde_json`'s `preserve_order` feature — through Cargo feature unification this changes the backing map of
  `serde_json::Value::Object` **project-wide** from a sorted `BTreeMap` to an insertion-order `IndexMap`
  as soon as `ai-transparency` is built along, regardless of whether the runtime flag is ever set.
  Affected 4 existing snapshot tests (`src/output/sr_audit_json.rs`,
  3× `tests/snapshot_tests.rs`) that silently assumed alphabetical key order —
  fixed with a small `sort_json_keys` normalization function before the
  respective `assert_json_snapshot!`, not by re-recording snapshots (the order must remain stable
  independent of the feature set). Fully verified green:
  `cargo test --all-features --lib` (1157 passed), `cargo clippy --all-features -- -D warnings`
  on lib+bins+the two affected test targets (a separate dead-code problem in `tests/wcag_rule_id_inventory.rs`/`tests/common/` that already
  existed before this session — unfinished, uncommitted work elsewhere — verified as pre-existing via `git stash`, not
  part of this change). Batch report rollup (analogous to `template_clusters`) deliberately **not** part of
  this change — a separate follow-up issue.

- **Canonical WCAG rule ID inventory, 2026-07-31 (#552, tracking #559):** closes a previously false
  assumption — `src/taxonomy/rules.rs`'s `RULES` (117 entries, 85
  `Dimension::Accessibility`) was wrongly treated as the complete list of all rules.
  In fact `RULES` is a curated scoring/report classification table, not a
  runtime registry; `PAGE_RULES`'s `rule_id` (`page_rules.rs`) is, per its own doc comment,
  "used for logging only", and `run_if_allowed!`'s axe_id (`engine.rs`) is only `RuleOutcome`
  telemetry. The actual mechanism behind `Violation.rule_id`: one `RuleMetadata`
  constant per rule file. A new, browser-free test `tests/wcag_rule_id_inventory.rs`
  scans `src/wcag/rules/*.rs` (RuleMetadata literals + the bare `*_AXE_ID` constants in
  `aria_roles.rs`/`widget_rules.rs`) and `src/patterns/*.rs` (literal `.with_rule_id("...")`
  calls in the Accordion/Modal/TabList/DisclosureMenu pattern detection modules, which get by
  entirely without `RuleMetadata`) — **real, verified number: 116 distinct `rule_id`s**,
  not 117/85. Reconciled empirically against a full `--full --level aaa` run (casoon.de);
  two real, previously unnoticed discrepancies found in the process: (1) `NormalizedFinding.rule_id`
  carries a second, parallel ID namespace (`a11y.*`, from `taxonomy::rules::RULES`) next to
  the axe_id namespace inventoried here — both are visible in the JSON for the same rule at
  the same time, depending on the report location; not a bug, but important for future
  rule comparisons (which namespace is meant?). (2) Reflow carries two diverging IDs for
  the same check — `RuleOutcome.rule_id = "reflow"` (telemetry label in `pipeline.rs`) vs.
  `Violation.rule_id = "css-overflow-hidden"` (`REFLOW_RULE.axe_id`, the actual finding).
  Additionally discovered along the way and filed as its own bug issue #560 (deliberately not fixed
  within #552, independent problem): contrast (`1.4.3`) and reflow (`1.4.10`) both run
  inline in `pipeline.rs`'s `run_rules`, without ever checking `RuleFilterConfig.should_run(...)`
  — `[rules] disabled`/`enabled_only` in `auditmysite.toml` (documented feature,
  `src/cli/config.rs:75`) currently has no effect for these two rules. This inventory is
  the basis for the planned
  ground-truth detection corpus (#553–#558), not `RULES`.
- **Detection corpus format + completeness check, 2026-07-31 (#553, #554, tracking #559):**
  builds on the rule inventory (#552). New, browser-free `tests/common/` (shared
  test support, not itself a test binary — standard Rust idiom): `rule_inventory.rs` (the
  #552 scan logic, now reusable) and `detection_corpus.rs` (`ExpectedCase`/
  `Expectation`/`Verdict` types + loader for `tests/fixtures/detection_corpus/<case>.expected.json`
  and `structurally_deferred.json` — both loaders return an empty list instead of failing if the
  directory/file does not exist yet, since the corpus deliberately starts empty and is only
  populated by #556). `tests/detection_corpus_format_test.rs` (7 tests, parsing/loader against a tempdir,
  no real fixture needed — avoids unverified ground-truth values before #556).
  `tests/detection_corpus_completeness.rs` diffs the 116 rule IDs against the IDs referenced in the corpus
  + `structurally_deferred.json`, reports covered/deferred/gap **without failing on
  low coverage** (current starting state: 0/0/116 — expected backlog,
  not a regression failure), but fails hard on a genuine authoring error
  (a referenced `rule_id` does not exist in the inventory at all, or a rule is marked both
  covered and `structurally_deferred`).
- **Runemark terminal presentation + batch progress adapter, 2026-07-30 (#529, #530):**
  #529 replaces the previous `colored`/`comfy-table` renderer (`src/output/cli.rs`, deleted)
  for `--format table` with a thin presentation layer on `runemark` (own sister crate,
  published on crates.io, like `renderreport`; **version checked live against
  crates.io**: 0.2.0 is the currently published version, not guessed). The new
  `src/output/terminal.rs` maps the already existing presentation layer shared by JSON/PDF
  (`ReportViewModel`/`BatchPresentation` — deliberately reused instead of
  rebuilt, both are **not** gated behind `feature = "pdf"`, so available for `--format table`
  in every feature combination) onto `runemark::report::Report` (`Console`,
  `Verdict`, `Metric`, `FindingGroup`, `NextStep`, `RenderOptions`). New `--color
  {auto,always,never}` flag (`ColorPolicy`); terminal width via `console::Term::stdout()`
  (already a transitive dependency of `dialoguer`, now direct). `console_for()` forces
  `Never` when the result is written to a file (`--output`), except with
  explicit `--color always` — prevents ANSI bytes in saved report files.
  `comfy-table` removed completely (0 remaining references); `colored` stays, as it is still used in many
  other CLI places (banner, doctor, sitemap-suggest) — deliberately out of
  scope for this issue, which only demands "remove only once nothing requires it".
  #530 replaces the direct `indicatif::ProgressBar` usage in `run_batch_mode`
  (`src/cli/runners.rs`) with `BatchLifecyclePresenter` (new, `src/cli/batch_lifecycle.rs`),
  a thin adapter over `runemark::ProgressSink` (`TerminalProgress`/`SilentProgress`,
  `progress` feature of runemark, which now pulls in `indicatif` transitively instead of directly — own
  `indicatif` dependency removed). New `--progress {auto,always,never}` flag
  (`ProgressPolicy`), independent of `--quiet` (which still suppresses everything).
  `run_concurrent_batch`'s signature/concurrency algorithm **unchanged** — only the
  callback body in `runners.rs` now calls `presenter.advance/notice_error` instead of touching
  the progress bar directly. **Real bug discovered along the way and fixed**: banner/plan/
  diagnostic/result/verdict lines in `src/cli/plan.rs` and `run_batch_mode` were
  throughout `println!` (stdout), gated only by `--quiet`, not by the output format — a
  batch run with `--format json` without `--output`/`--quiet` mixed plain text into the
  JSON payload on stdout. Mechanical fix: all these lines are now `eprintln!` (stderr)
  or, for the actual batch lifecycle (sitemap/crawl diagnostic one-liners,
  results line, final verdict), go through `presenter.notice()`/`presenter.finish_verdict()`.
  `print_verdict()` (also used by single mode) remains as a standalone function,
  but is now `eprintln!`-based — fixes the same class of bug for
  single-mode JSON runs as well. **A second leak, uncovered by the first live verification,
  found and fixed:** the global `tracing_subscriber::fmt()` in `src/main.rs` had
  no explicit `.with_writer(...)` and therefore wrote (tracing-subscriber's own
  default) to **stdout** — a `tracing::warn!` during a batch run (e.g. a
  page timeout in `src/audit/batch.rs:164`) therefore still ended up, ANSI-colored, ahead of the
  JSON payload on stdout and broke `jq`. Now `.with_writer(std::io::stderr)` +
  `.with_ansi(...)`, the latter coupled to the same `--color` policy (not to
  tracing-subscriber's own, independent auto-detection). Verified live again after this fix
  (timeout case deliberately reproduced): stdout with `--format json` stays, in a
  batch run without `--quiet`, clean JSON that `jq` parses validly even on a page timeout;
  `--progress never --quiet` produces exactly 0 bytes on stderr;
  `--color always` forces ANSI on stderr; `report-lint` on the result JSON reports
  no findings.
- **Contrast pixel-sampling coverage + new `design_quality` module, 2026-07-30 (#527, #528):**
  #527 fixes a real gap in WCAG 1.4.3: `build_sample_tasks` (`src/wcag/rules/contrast.rs`)
  previously sampled only if the background WAS uncertain **and** the CSS ratio already failed —
  an element with visually plausible CSS colors whose rendered result is reduced by an opacity stack,
  blend mode or a real `<img>` behind the text was never sampled and stayed
  completely invisible (confirmed live on casoon.de: `finding_count: 0` despite a visibly failing
  hero area). Fix: the sampling condition now samples for every uncertain background
  regardless of the CSS ratio (with a `MAX_SAMPLE_TASKS = 60` cap, largest area first);
  `styles.rs`'s ancestor walk now additionally detects "uncertain" for `opacity < 1`,
  `mix-blend-mode`/`background-blend-mode` and an overlapping `<img>`/positioned
  overlay sibling (limited to the first 3 ancestor levels). A real duplicate-finding bug found along the way
  was fixed as well: the same selector/rule combination could, through
  different sampling results between the desktop and mobile pass, appear simultaneously as a
  confirmed violation AND as a NeedsReview warning — `merge_wcag_violations`
  (`src/audit/pipeline.rs`) now suppresses warnings for every selector/rule combination that
  already exists as a violation. New fixture `tests/fixtures/opacity_overlay_contrast.html` +
  `test_opacity_overlay_contrast_pixel_sampling`.
  #528 adds `design_quality` as a new, **opt-in** (`--design-quality`, deliberately not yet part of
  `--full`) and **score-neutral** module: heuristic UX/readability hints (overflow clipping
  on positioned/interactive elements, line length, line spacing, long uppercase text,
  layout-affecting CSS transitions) from the already open CDP page, a single `page.evaluate()`
  pass for all text rules. A dedicated `DesignQualityFinding` type (not `wcag::types::Violation`)
  guarantees score neutrality by construction — there is never a `push_indicator` call/
  `ModuleScoreEntry`, so no code path touches `AccessibilityScorer`/the weighted
  overall score; verified by an integration test (`report.accessibility.score/grade/certificate`
  byte-identical with and without the module). The layout-transition rule does not detect transitions itself anew,
  but projects `performance::animations` findings (filtered to layout-affecting properties) as
  advisories — avoids a second independent detection path and thereby duplicate findings by
  construction. Localization follows the established kind-enum pattern (#406):
  `finding_message_text(rule_id, level, en)` is the single text source, analysis calls with `en=true`
  (JSON canonically English), PDF builder with the run language. New `ExperienceSection.design_quality`
  field, full JSON/PDF integration (`ReportModule`, `ModuleBlob`, `ModuleDetailsBlock`,
  dedicated PDF renderer without ScoreCard). Cache signature bumped to `fmt=12`.
- **Report Quality Layer v1.2 — Phase 5: visual verification pipeline, 2026-07-16 (#510, tracking #512):**
  closes the gap previously documented as open (see the previous entry below, "partial").
  Instead of a stored pixel-diff baseline (font rendering/anti-aliasing would make that flaky across
  different machines/CI runners), two new, deterministic
  same-run checks in `src/output/pdf/tests.rs`: (1)
  `test_dual_viewport_performance_renders_two_gauges_not_a_flat_strip` — structure regression guard
  for the historical case named concretely in #510 (the "wrapped dual-viewport cell"): checks
  in the Typst source (not pixels) that the desktop/mobile performance comparison still renders as two
  standalone `Gauge` components in a 2-column `Grid` (`type: "gauge"` ≥ 2×, own
  `label: "Desktop"`/`label: "Mobile"`) instead of falling back to the former flat "Desktop 85 · Mobile 67"
  text line — with a new `dual_viewport_performance(desktop_score, mobile_score)`
  test helper that constructs `report.performance` (mobile) + `report.dual_viewport.desktop.performance` (desktop),
  the only way to populate both `PerformancePresentation.desktop`/`.mobile`. (2)
  `test_representative_fixture_pages_are_not_blank_and_stay_within_page_budget` — rasterizes an
  enriched fixture (WCAG findings, dual-viewport gauges, tech-stack tables, throttled
  network table; thereby covers cover/score cards/tables/findings/methodology/technical metrics
  from #510's acceptance criterion) at Technical level and checks non-emptiness AND a page budget
  (60 pages, detects a reflow outlier without judging subjective layout quality pixel-based
  — that deliberately remains the job of the `report-critic` skill, #509). On a failure,
  all rasterized pages are copied to `target/pdf-visual-debug/<reason>/` (tempdir contents
  would otherwise be deleted before any manual inspection). **Real, previously unnoticed gap found and
  fixed:** the `pdf-smoke` CI job never installed `poppler-utils` — every `pdftoppm`/`pdftotext`-
  gated test (all blank-page/visual tests from Phase 5 as well as the older `pdftotext`-based
  content traceability tests) silently skipped itself in CI since its introduction
  (`find_executable("pdftoppm")` → `None`) instead of running — never visible as red CI, because a
  skipped test does not fail. `.github/workflows/ci.yml`'s `pdf-smoke` job now installs
  `poppler-utils` via `apt-get` and uploads `target/pdf-visual-debug/` on failure as a
  build artifact (`actions/upload-artifact@v4`, `if: failure()`) — closes #510's
  acceptance criterion "generated artifacts are traceable in CI".
- **Report Quality Layer v1.2 — Phase 3: feedback corpus, 2026-07-16 (#511, tracking #512):**
  new `tests/regression_corpus/*.json` (16 entries, one file per confirmed case) — format
  `{id, category: Invariant|Semantic|Completeness|Explanation|Visualization, problem, evidence,
  expected, regression, counter_examples[], status: resolved|known_gap}`. Covers the starter corpus from
  issue #511 (score mismatch, critical-as-label-for-critical+high, score without scale, counter
  without scope, available data without report usage, metric strip wrapping, batch classification from
  wrong score basis) plus the concrete cases confirmed in this session: the four
  localization leaks (SEO details, tech-stack severity, TechCategory debug format,
  renderreport component default labels) and the `violated_rule_count` scope false positive from
  group C — all `status: resolved` with a reference to their respective regression test. Four further cases,
  found by the `report-critic` skill dry run against real batch/single reports and **not** fixed
  in this session (`score_area_for_finding` substring misclassification,
  `is_generic()` link-text substring false positive, impact-vs.-reach contradiction in
  batch `top_actions`, `{:?}` debug leak + truncating division in `management_risks[].rationale`)
  are `status: known_gap` — deliberately documented instead of silently dropped. New
  `tests/regression_corpus_contract.rs` validates only the corpus shape itself (required fields,
  enum values, unique/filename-matching `id`, non-empty `counter_examples`, `resolved` ⇒
  non-null `regression`) — it does not re-run the referenced regression tests.
  `.gitignore`'s blanket `*.json` gained a `!tests/regression_corpus/*.json` whitelist entry
  (same pattern as `tests/lint_fixtures/`).
- **report-lint false-positive fixes from the report-critic eval, 2026-07-16 (#507 follow-up):**
  the eval run found two real false positives in `src/lint/checks.rs`, confirmed on a real
  batch report (`reports/casoon-batch-en301549.json` — previously 2 findings, now 0):
  (1) `check_grade_and_certificate` did not know `gate_certificate_by_risk`
  (`src/audit/normalized.rs:515`) — a certificate intentionally downgraded because of Risk=High/Critical/legal_flags/
  blocking_issues ("EINGESCHRÄNKT" (restricted)/"NICHT BESTANDEN" (failed)) was
  wrongly reported as a score mismatch. Now the check accepts any value that
  `gate_certificate_by_risk` could legitimately produce for the given risk inputs
  (`risk_gate_inputs`/`acceptable_certificates`, new helpers). (2) `violated_rule_count` (globally
  distinct rules) vs. `severity_counts.total` (sum of the per-page-distinct rule rows)
  are structurally different aggregations at batch `summary` level that only coincidentally agree for
  single reports — the check wrongly compared them for exact equality. New
  `ScopeRelation` distinction: `Exact` for single reports and each individual page, `AtMost`
  (global ≤ sum) only for the `summary` of a batch report. 5 new regression tests in
  `src/lint/checks.rs`.
- **Localization follow-up from the report-critic eval, 2026-07-16:** the report-critic eval run
  (see the skill entry below) found real localization leaks — German words without umlaut/ß, therefore
  undetected by the existing guard test. Fixed: `build_seo_details`
  (`src/output/builder/single/module_details.rs`) fully localized (meta_tags,
  identity_facts, page_profile_facts, heading/social/technical/tracking_summary, SchemaExtracted
  text blocks, `signal_rows` rating — ~50 places); `Severity::label()` (always German) at two
  further PDF places (`detail_modules/indicators.rs`, `detail_modules/overview.rs` — a
  third supposed occurrence there was actually `BudgetSeverity` with its own, already
  language-neutral "Error"/"Warning" labels, not a bug); `single_report.rs` ("Absprungrate" (bounce rate) was itself
  mistranslated in the EN branch, "Vorkommen" (occurrences) hardcoded in the format string). Side finding: `{:?}`
  debug leak at `tech.category` (`TechCategory` had no `label()` method) — now fixed.
  New regression test `test_seo_details_english_locale_has_no_known_german_leaks`
  (`src/output/pdf/tests.rs`) — the first EN-locale check that runs `build_seo_details` at all with
  real SEO data (`pdf_fixture_report()` alone carries no SEO). **Important
  methodology correction discovered along the way:** `--debug-typ` dumps contain the complete
  renderreport component library (`include_str!`), not just the content actually rendered — a
  string occurrence in the dump does not prove that it appears on a page (it can come
  from a never-instantiated component definition). Led to two renderreport fixes
  (**v0.2.36**, see below) and an addition to the `report-critic` skill.
- **renderreport v0.2.36:** `dominant-issue-spotlight` and `severity-overview` (two components
  currently unused by auditmysite) had German labels hardcoded in the Typst template
  (`label-text("Empfehlung")` (recommendation) etc.) instead of receiving them, like all other
  components, as a data field from the Rust caller — no live bug (unused), but an inconsistency with the established
  pattern (Rust always passes already localized strings, templates themselves have no
  language logic). Both templates now accept optional `label_*` data fields with German
  defaults (`data.at(key, default: "...")`); `DominantIssueSpotlight`/`SeverityOverview` in
  renderreport have new `with_labels(...)` builders analogous to `CoverPage`. Purely additive, no
  behavior change for existing callers. `cover_page.typ`'s German fallback value for
  `modules_label` deliberately left unchanged — auditmysite always sets this field explicitly for
  both languages, the fallback is unreachable.
- **New skill `report-critic` (.claude/skills/report-critic/SKILL.md, #509):** evidence-bound
  AI critique of a finished report (JSON + `--debug-typ` text) for contradictions, missing
  scope, unsupported conclusions, missing action linkage, text-visible
  layout artifacts — complements (does not replace) `report-lint`. Tested via skill-creator eval loop
  (3 test cases, with/without skill, real 1.1 MB batch report from casoon.de): both configurations
  consistently found real bugs (substring classification bug in two independent modules,
  `Some(84)` debug leak, impact-vs-reach contradiction in an aggregation table, among others) — the
  skill mainly brings consistent output structure, not additional capability over an
  already very competent baseline agent. Eval workspace under
  `.claude/skills/report-critic-workspace/` (not committed, local).
- **Report Quality Layer v1.2 — Phase 5 (partial): visual PDF smoke checks, 2026-07-16 (#510,
  tracking #512):** correction to the original Phase 5 plan — rasterization via `pdftoppm` was
  **not** greenfield; `src/output/pdf/tests.rs` already had two `pdftoppm`-gated smoke tests
  (`find_executable("pdftoppm")` skips gracefully when unavailable, matching CI's `pdf-smoke` job
  which runs `cargo test --features pdf_test`). Extended that existing infra rather than building a
  new `xtask`: a `png_luma_std_dev` helper (grayscale pixel std-dev via the already-present `image`
  crate) plus `rasterize_pages`, backing two new tests —
  `test_single_pdf_technical_pages_are_not_blank_when_pdftoppm_is_available` and
  `test_batch_pdf_pages_are_not_blank_when_pdftoppm_is_available` (batch PDF rasterization had no
  test at all before this) — flagging a near-solid-color page (missing font/asset → blank render)
  without exact pixel-diff, which is deliberately avoided: font hinting/anti-aliasing differs across
  machines/CI runners, so an exact-match baseline would be flaky by construction. **Not yet done,
  open design question**: the plan's "visual diffs for stable layout regions" (e.g. cover skeleton)
  needs a real decision on baseline storage/regeneration workflow and tolerance before implementing
  — deferred rather than guessed at. Overflow/cut-content/misalignment detection and the semantic/
  visual AI judgment pass both still route to #509 as originally planned.
  new `tests/coverage_matrix.rs` — an `#[ignore]`-gated `export_coverage_matrix` test (same pattern
  as `output::builder::tests::export_all_interpretations`) walks every `AuditCatalog::standard()`
  module and counts literal id() occurrences across four surfaces (`src/output/pdf/**`,
  `docs/OUTPUT_CONTRACT.md`, both JSON schemas, `tests/**` + fixture dirs), writing
  `reports/coverage_matrix.json` for human review. Deliberately **not a CI gate** — a substring
  count can false-negative on indirection and false-positive on a common word (confirmed: the tool's
  own doc-comment mentioning "commerce" counts as one of commerce's "fixture references"). Only one
  hard, always-true assertion ships (`every_catalog_module_has_a_unique_nonempty_id` — a tripwire on
  the catalog itself, not on coverage outcomes) rather than "every module has PDF coverage", which
  would immediately fail today. **Real gap the first run surfaced**: `commerce` has 0 PDF references
  and 0 references anywhere under `tests/` outside its own module directory — the module has JSON
  output (`src/output/module.rs`, `src/output/json.rs`) but no PDF rendering and no cross-cutting
  test coverage, despite CLAUDE.md recording it as "COMPLETE, alle Slices 1-4 gemergt" (complete, all slices 1-4 merged). Not fixed
  as part of #508 (out of scope — #508 is the detection tool, not a mandate to close every gap it
  finds); flagged here for a deliberate decision on whether that's an intentional Studio/JSON-only
  scope or a real oversight. `#509`/`#510`/`#511` (AI critic, visual PDF pipeline, feedback corpus)
  are planned but not started.
- **Report Quality Layer v1.2 — Phase 2: deterministic report-lint, 2026-07-16 (#507, tracking #512):**
  new `src/lint/` — `lint(report: &serde_json::Value) -> LintReport` runs four registry-driven
  (#506) checks with zero network/Chrome dependency, each producing a `LintFinding{check_id,
  evidence_path, expected, actual, severity}`: (1) `summary.score`/`summary.overall_score` alias
  consistency plus single-report summary-vs-page cross-check (the "18/100 vs 20/100" corpus
  shape); (2) `grade`/`certificate` re-derivable from `overall_score` via the same shared
  `LETTER_GRADE`/`CERTIFICATE` `BandSet`s the production scorer uses, checked on both `summary`
  and every page; (3) `severity_counts`/`occurrence_counts.total` equal the sum of their four
  severity fields, and `violated_rule_count`/`violation_count` match the corresponding scoped
  total (the "Zähler ohne Scope" (counter without scope) corpus shape); (4) the report's own `metric_context` block
  matches what `REGISTRY` currently generates (catches a stale/cached or hand-edited report).
  New CLI subcommand `auditmysite report-lint <file> [--fail-on low|medium|high|critical]`
  (default `high`) prints findings and returns a non-zero exit code (via `AuditError::ConfigError`,
  exit 3) when the worst finding meets or exceeds the threshold — verified end-to-end against a
  hand-built broken report before considering this feature done, not just via unit tests.
  `registry::json_path_candidates` (moved from a private test-only helper in
  `tests/registry_contract.rs` to `src/registry/paths.rs` so `lint` and the contract test share one
  implementation) had a latent bug caught while writing paths.rs's own dedicated unit tests:
  splitting on `" and "` before trimming turned prose like "...score and nested dimension scores"
  into a second bogus path candidate ("nested") — fixed by dropping the `" and "` split entirely
  (only `" / "` ever separates two *real* paths in this codebase's `json_path` text; `take_while`
  alone already stops at the first invalid character, which discards trailing " and ..." prose).
  Added a 5th check: every `REGISTRY` entry's `docs_url` is a well-formed `<path>#<anchor>`
  reference (shape only, no filesystem access — a released binary has no guarantee `docs/` exists
  alongside it; the deeper anchor-resolution check stays in `tests/registry_contract.rs`, which
  only needs to hold in the dev/CI checkout). `tests/lint_fixtures/*.json` (clean single, 3 broken
  variants covering score-alias/grade/batch-certificate mismatches) plus `tests/report_lint_tests.rs`
  spawn the compiled binary end-to-end via `CARGO_BIN_EXE_auditmysite` and assert on exit code +
  finding check-ids — these fixtures double as the seed for #511's regression corpus. No new CI job
  was added: since this test file has no network/Chrome/pdf-feature dependency, it's already
  exercised by the existing unscoped `cargo test` in the `check`/`check-all-features` jobs.
  Added a 6th, narrowly-scoped PDF-traceability check instead of the originally-sketched "does any
  number in the PDF appear anywhere in the JSON" scan (rejected: real false-positive risk from
  dates/page counts/unrelated percentages). New optional `--typst-source <path>` on `report-lint`
  (the `--debug-typ` Typst source for the same report); when given,
  `check_pdf_certificate_traceability` computes the certificate token the JSON's `overall_score`
  implies via the same shared `CERTIFICATE` `BandSet` the PDF itself uses, and checks that exact
  token is present in the Typst text — presence-only (not "no other certificate word may appear",
  since a legend explaining the band system may legitimately mention other tokens), `Severity::Low`
  (advisory, never breaches the default `--fail-on high` on its own). `lint()`'s signature grew a
  second `Option<&str>` parameter for the Typst text. Two `.typ` fixtures added under
  `tests/lint_fixtures/`; a first draft of the "broken" fixture accidentally spelled the certificate
  word out in its own comment ("SEHR GUT" (very good) contains "GUT" (good) as a substring) and silently passed —
  caught only because the CLI integration test asserted on the actual finding appearing, not just
  the exit code, which is why report-lint's own test fixtures need their negative-case text
  double-checked for accidental substring self-matches. `#508`–`#511` (coverage matrix, visual PDF
  pipeline, AI critic, feedback corpus) are planned but not started.
- **Report Quality Layer v1.2 — Phase 1: canonical metric registry, 2026-07-16 (#506, tracking #512):**
  new `src/registry/` (`MetricSpec`/`BandSet`/`MetricKind`/`Direction`/`Scope`/`Aggregation`,
  `REGISTRY` const table) gives every specialized number one machine-readable definition instead of
  scattered renderer/doc logic. Seeded 1:1 from `src/output/json.rs`'s former hand-written
  `metric_context()` vec — `metric_context()` now derives `score_definitions`/`count_definitions`
  from `REGISTRY` instead of the other way around, with the same `field`/`unit`/`meaning` text
  (zero JSON output change, verified against snapshot/schema/consistency test suites).
  `docs/OUTPUT_CONTRACT.md` gained a `## Metrics` section with one `<a id>` anchor per registry
  entry; `tests/registry_contract.rs` (mirrors `tests/parity_contract.rs`'s "contract file + test"
  shape) checks unique ids, `docs_url` anchors resolve, `reviewed_at` parses as a date, and
  `json_path` resolves against `docs/json-report.schema.json`/`docs/json-batch-report.schema.json`.
  **Phase 1 complete (all 7 migration steps):** every one of the ~19 independent score→label/grade
  definitions found across taxonomy, PDF renderers, and module-specific label functions now
  references a named `BandSet` in `src/registry/bands.rs` instead of re-coding thresholds —
  `FIVE_BAND` (90/75/60/40 words), `FIVE_BAND_LETTERS` (same cutoffs, A–F), `LETTER_GRADE`
  (90/80/70/60, A–F), `SECURITY_GRADE` (90/80/70/60/50, A+–F), `BATCH_GRADE` (95/90/80/70/60,
  A+–F), `CERTIFICATE` (90/75/60/40, SEHR GUT…UNGENÜGEND (very good … insufficient) — was independently re-implemented in
  both `audit::scoring::calculate_certificate` and the PDF's `cover::batch_certificate_label`
  before this migration), `COVER_PHRASE`/`SCORE_RANGE` (90/75/60/40, sentence variants), `MEDAL`
  (90/80/60, terminal-table GOLD/SILVER/BRONZE/FAILED), `BAR_COLOR_BAND` (90/80/70/50, terminal
  color only), and `SEO_BAND` (90/70/55/35 — SEO's own family, deliberately kept distinct, not
  collapsed into `FIVE_BAND`). No threshold values changed; `output::cli::colorize_grade` was
  deliberately left untouched (keys off an already-resolved grade letter, no threshold to
  register). Found and flagged but **not fixed** (out of scope for #506):
  `performance::scoring::PerformanceGrade::emoji()`/`.label()`/`Display` are dead code — never
  called anywhere reachable in `src/` (JSON serializes the enum variant name directly; the PDF
  renders scores via the now-migrated `score_band_label`/`score_range_label` instead), so the
  emoji never actually reaches a report despite existing in source.
  `#507`–`#511` (report-lint, coverage matrix, visual PDF pipeline, AI critic, feedback corpus)
  are planned but not started.
- **BFSG / EN 301 549 mapping annex, 2026-07-15 (#en301549):** `src/wcag/en301549.rs` — canonical
  50-entry WCAG 2.1 A/AA ↔ EN 301 549 (chapter 9, "Web") clause table, `derive_annex`/
  `derive_batch_rollup` as pure projections over `NormalizedFinding` (nothing new stored on
  `NormalizedReport`, no cache-signature change). Four-way scope split per clause: violations
  found / no violations in automated scope / manual review required / (chapter-level, not
  per-clause) out of audit scope. `screen_reader/bfsg.rs` reduced to a thin wrapper; the
  legally-unverified `"§12 Abs. 1"` citation stays local there, deliberately not propagated.
  JSON `en301549_annex` always emitted (`PageDetail`, single + batch) plus a batch
  `UnifiedSummary.en301549_rollup`; the PDF appendix only renders behind the new opt-in
  `--annex en301549` flag ("Zusatz" (addendum), not default-on). Disclaimer text (DE/EN) is a
  scope-of-testing disclosure only — no statutory citation, no conformity claim — reusing this
  project's existing "manual audit with assistive technologies (screen reader, keyboard
  navigation)" wording rather than inventing new phrasing.
- **Plain-language content in the existing PDF, 2026-07-15:** no separate report variant — the
  Chapter 02 finding card gained a plain-language lead-in (`customer_description` + `user_impact`)
  between the header and the QA-meta block (previously not rendered there at all, not just
  misordered). `finding_group_from_normalized`'s no-`RuleExplanation` fallback no longer leaks raw
  canonical-English `f.description` into German reports. Part-1 divider reframed as dual-audience
  ("Inhaber, Entscheider und Entwickler" (owners, decision makers and developers)).
- **Journey × Commerce deepening, 2026-07-14/15:** form-error journey now groups required fields
  into up to 3 per-form candidates (was one page-wide candidate) and a `PURCHASE_FINAL_HINTS`
  deny-list guarantees a purchase-final button (e.g. "Jetzt kaufen" (Buy now)) is never a synthetic-click
  target. New commerce-aware journeys on a detected shop's product-detail page under
  `--interactive full`: add-to-cart feedback (SC 4.1.3) and quantity-stepper operability
  (SC 2.1.1/4.1.2). **`CommercePageKind::Cart`/`::Checkout` removed entirely** (breaking JSON
  change) — this tool has no cross-page session/cart state, so a cart/checkout URL reached cold
  is almost always empty or redirects before rendering anything a page-kind-gated heuristic could
  act on; confirmed no reference in the sibling `auditmysite_studio` repo before landing.
- **WCAG coverage + correctness sweep, 2026-07-14:** new rules 1.3.2 Meaningful Sequence, 3.3.7
  Redundant Entry, 2.4.11/2.4.12 Focus Not Obscured, 2.2.2 Pause/Stop/Hide (automated WCAG-AA
  count now 36/50, up from 33). Fixed three known-defective rules: `focus_visible_css.rs` (never
  fired in production — missing evidence selector demoted every finding to a warning),
  `focus_visible.rs` (dead AX-tree `tabindex` read, removed), `non_text_contrast.rs` (mistagged/
  dead, replaced by a real CDP-based `non_text_contrast_css.rs`). Closed remaining #406
  localization gaps (Dark Mode, keyboard journey, `expected_impact`/`complexity_reason`) and
  several report-wording/readability fixes across Chapters 01–03 of the single report.
- **Evidence-Grade Findings (single report only) + Template-Root-Cause-Dedup (batch only), 2026-07-14:**
  Single-report finding cards now embed a cropped element screenshot (`src/accessibility/element_capture.rs`,
  gated on single-URL mode via `PipelineConfig.capture_element_evidence`, capped at 12 crops/report,
  contrast findings excluded by construction), a ≤3-level DOM path, and computed contrast-ratio evidence
  (`ViolationEvidence::computed`, `OccurrenceDetail.evidence: Vec<ViolationEvidence>` — new additive JSON
  field, `docs/json-report.schema.json` updated). Batch reports gain verified template-level clustering
  (`src/audit/template_dedup.rs`): findings sharing an identical `(rule_id, normalized selector)` fingerprint
  across ≥3 pages / ≥60% coverage become a `TemplateCluster` (`confirmed` when the HTML-snippet shape also
  matches, `likely` otherwise — decision-action wording only upgrades for `confirmed`), surfaced additively
  in `UnifiedSummary.template_clusters` and the batch PDF. Both features are additive/JSON-safe (screenshot
  bytes are `#[serde(skip)]`, never touch cached `report.json`). Fixed two pre-existing binary-test
  regressions surfaced by running the full `cargo test --features pdf`/`--no-default-features` suites
  (not covered by `cargo test --lib`): a stale `non_text_contrast`→`non_text_contrast_css` rename reference
  and a stale `KNOWN_EXCEPTIONS` entry in `tests/wcag_coverage.rs`.
- **Product-grade PDF redesign (single report, PR feat/report-product-redesign):** cover as a composed dashboard (dominant overall score + grade-band phrase + module gauge strip); management view with severity counters, spider radar "Qualitätsprofil" (quality profile) and strengths/optimization cards; every module its own level-2 chapter with magazine opener + key-statement line (#15); AI Visibility + Content Visibility + Source Quality merged into one chapter "KI & Vertrauen" (AI & Trust); action plan as action cards grouped by problem level (systemic/local, without time/effort); cause distribution as a bar chart; ToC at top level (depth 2); modern borderless tables; consistent 4-color law in `src/output/pdf/design.rs` (`score_color`/`severity_color`, thresholds 75/40); no "/100", no A–F grade (band label via `score_band_label`), no emoji. **renderreport 0.2.26** (composed `cover-page`, real spider radar, borderless `audit-table`, de-emojified callouts, sticky headings/component titles against orphaned headings). **JSON fix:** cache-hit JSON now emits the full `detail.modules` blob (previously empty, since normalized-only path).
- **Semantic-Eval removed completely:** module `src/semantic_eval/` (Fastembed + Mistral), CLI flag `--no-semantic-eval`/`--semantic-eval`, `[semantic_eval]` TOML section, `fastembed` dependency + `semantic-eval` Cargo feature, type `AdvisoryFinding` and the field `advisory_findings` (from `NormalizedReport`/`AuditReport`/JSON as well as the PDF advisory sections). `audit_signature` no longer contains a `semantic` segment (cache invalidated once).
- **Scoring correctness + report readability (PR fix/perf-relative-weight-cap):** relative weight-penalty cap (≤70 % of the vitals base, protects low-base pages from 0); renderreport **0.2.23** (progress-arc gauges + fixed label box, no cover overlap); customer passages without jargon duplicates/meta prefixes; cover label "N Accessibility-Befunde" (N accessibility findings; scope explicit, WCAG-only); vuln detection Lodash↔Underscore via `_.runInContext`; #406 leaks (search_experience components + warnings re-derived); pluralization "1 Schema"; `compact_html` (data URIs → "data:…", line heights); empty "Befunde nach Ursache" (findings by cause) divider page filled; short indicator modules (Best Practices/Tech Stack) packed via divider instead of each getting its own almost-empty page.
- **Cache correctness (PR #458, #404/#405):** the full `AuditReport` is cached (`report.json`, screenshots stripped), cache hits render faithfully instead of via the lossy `to_audit_report`; `screen_reader_audit` (`#[serde(skip)]`) is rebuilt from the AXTree via `hydrate_cached_report`. `NormalizedReport` fields with `skip_serializing_if` now have `#[serde(default)]` (round-trip blocker fixed — the cache never loaded). Verdict always from `cached.audit`. `persist_artifacts` runs after the canonical perf adoption (`audit_page` returns `SnapshotData`). `audit_signature` contains `lang`; corrupt cache → miss + warning.
- **Report quality (PR #459, #446):** Security/SEO/Page-Health emit a "keine Auffälligkeiten" (no findings) confirmation line when the findings collection is empty (`pdf-section-clean`) — "checked & clean" vs. "not checked" can be told apart.
- **Scoring corrections (PR #460, #455/#456/#457):** DOM size as a degressive penalty (max 35) instead of a hard 59 cap; throttled profiles get the headline `content_weight` (no Slow3G>Fast3G inversion); risk-breadth path decoupled from critical occurrences (`legal_flags >= 3`), `driven_by`/summary reflect the real trigger (breadth vs. volume).
- **Audit quality (PR #454):** localization fixes (security CSP, WCAG findings, SEO heading canonically English); scoring desaturation (DOM cap >6000/>10000, accessibility root curve from penalty 70, mobile soft floor, Risk=Critical only with systemic exposure #250); Core Web Vitals measurement correctness (CLS session window, LCP+TBT limited to the load window, `MeasurementContext::LabThrottledMobile` marks throttled headline vitals in the JSON).
- **Localization architecture (#406):** JSON canonically English, only the PDF is multilingual. Analysis bakes in English, PDF presentation derives (kind-enum pattern). See the section "Lokalisierungs-Architektur" (localization architecture). Plus audit finding fixes (#442–#452, #411, #447, #449) — PR #453.
- **Catalog refactoring** (phase A+B): `trait AuditModule` + `AuditCatalog` registry with topo sort; all 12 modules migrated; table-driven WCAG page-rule catalog; `audit/interpretation.rs` (pre-computed DE/EN texts); `audit/summary.rs` (aggregation logic); builder is a pure mapper (#330–#338)
- Branch: `main`
- Cache: `--reuse-cache` validates `CacheMeta.audit_signature` (tool version + WCAG level + active modules + consent) against the current configuration; on mismatch cache miss + warning, a legacy cache without signature is never reused (#260)
- Crawler: parser-based link extraction via html5ever incl. `<base href>` (#263)
- Batch JSON: optional `sample` block (source, total_discovered, audited, sample_limit, selection, is_sample) + PDF scope-of-audit line (#261)
- Performance: `VitalMetric.measurement` (`lab_headless`/`estimated_lab`); INP/TTI/Speed Index marked as lab estimate, lab disclaimer in the report (#262)
- Contrast: image/gradient backgrounds are demoted to manual-review warnings instead of confirmed violations (#264, pixel sampling open)
- **Accessibility Journey Layer** (`--interactive off|basic|full`): tab walk, skip link, disclosure, modal, TabList, menu, form-error announcement, SPA navigation, link text/heading/landmark inventory (#297–#301). Results in `interactive_findings` + `accessibility_journey` in the JSON.
- **Snapshot Export** (`--export-snapshot <path>`): AXTree + journey traces as YAML for CI regression (#301).
- Link-text stopwords in i18n FTL (`locales/de|en/report.ftl`, key `linktext-generic-stopwords`) — extensible without code changes (#299).
- 95+ WCAG rules implemented (Level A, AA, full AAA coverage)
- 2 output formats (json, pdf); table for quick terminal checks
- Batch processing with configurable concurrency
- Pattern Detection: MainNavigation, SkipLink, Accordion, Dialog, DisclosureMenu, TabList, Form
- Modules: Performance, SEO, Security, Mobile, Dark Mode, Design Quality (opt-in, score-neutral), UX, Journey, AI Visibility, Content Visibility, Source Quality, Tech Stack, Best Practices, Commerce, Accessibility Journey Layer
- Consent: `--dismiss-consent` flag; CMP cookie injection + banner click; `consent_banner` audit_flag in the JSON
- `audit_flags` kinds: `conflicting_signal` (3.1.1 vs. SEO lang), `viewport_gap` (desktop/mobile ≥20 points), `consent_banner`, `consent_wall_artifact`, `bypass_blocks_untested` (skip link present but functionally broken — static check has PASS, journey FAIL)
- JSON: **Unified Report Envelope v2.0** — uniform schema for single + batch (`schema_version`, `report_type`, `summary`, `pages[]`, `pages[i].detail`). Breaking change vs. v0.17.
- Scoring: depth saturation (two-phase), diversity factor, soft floor + logarithmic compression for extreme penalties (≥85 points), WCAG principle coverage; `score_breakdown` (only with `score_calculation_method = "viewport_weighted"`, otherwise absent)
- Findings: `category` field on `NormalizedFinding` (`"wcag"` / `"seo"`); `severity_counts` counts **findings** (one row per rule/severity, **WCAG category only** — remains risk/legal relevant). In the JSON report, `occurrence_counts`, `violation_count` and `violated_rule_count` cover **all categories (WCAG + SEO)** — consistent with `findings[]` and `detail.fix_guidance` (#254/#255). `top_recurring_rules` remains WCAG-only. Note: `NormalizedReport.occurrence_counts` is still WCAG-only (feeds `SiteState`/risk), the JSON PageEntry computes the all-category variant separately. `risk.severity` = most severe violation across all findings (no separate `severity_max` field)
- Risk level: score-based fallback (score ≤ 20 → at least Medium); `legal_flags > 0` or `blocking_issues ≥ 1` raise the level to at least Medium. `legal_flags` counts **distinct WCAG Level A rules** with High/Critical severity (not occurrences).
- History: `schema_version: "1.0"`, `report_type: "history"` in history JSON files
- PDF: throttled performance table, indicator labeling consistent, empty page after ToC fixed; accessibility journey section in single and batch reports
- Performance score: Lighthouse v10/v11 weighting (FCP 10 %, LCP 25 %, TBT 30 %, CLS 25 %), log-normal score curves with p10/p50 calibration; CLS > 0.5 hard-capped to 0
- `tool_version` as top-level field in the JSON report (alongside `schema_version`/`report_type`)
- Sitemap summary contains `violated_rule_count` (deduplicated across all pages) and `top_recurring_rules` (max. 10 most frequent WCAG violations)
- Pass criterion (`passed_url_count`): accessibility_score ≥ 80, no critical findings and no WCAG Level A High/Critical findings (i.e. `legal_flags == 0`)
- `detail.fix_guidance` is always present in the JSON (empty array with 0 findings) — also in batch/sitemap reports; there each page carries a compact `detail` (only `fix_guidance`, without module blob), see #256

