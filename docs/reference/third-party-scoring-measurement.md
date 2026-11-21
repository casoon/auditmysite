---
title: Third-party scoring measurement
sidebarLabel: Third-party scoring measurement
description: How many accessibility score points of each reference site, and of four more public sites with embeds, come from iframe documents and known third-party providers (measurement for #1109, 2026-11-21).
order: 91
---

Measurement record for issue #1109 ("Third-party and iframe content: attribute it, decide how
it is scored and shown"). It measures only; it changes no scoring, band or behaviour. The
decision it led to (attribution only) is recorded in the output contract, "Finding origin". The reference bands in `tests/fixtures/reference_sites.json` are untouched.

This record replaces the first measurement of the same day. That one audited the two lazy-loaded
basf.com widget iframes while they were still `about:blank` and attributed their points from the
iframe `src` by hand; since #1163 (every finding carries `origin`, `provider`, `frame_host`) and
#1164 (lazy frames are loaded before the frame pass) the frames are audited with their real
content and the origin is read from the report.

## What changed against the placeholder measurement

| | Placeholder measurement (before #1163/#1164) | This measurement |
|---|---|---|
| basf.com frames audited | 2, both as `about:blank` | 2 with their real documents, 0 `not_loaded` |
| Frame findings | `html-has-lang` on the two blank placeholders, 14.8 points together | none for `html-has-lang`; the loaded same-site newsroom frame (www1.basf.com) carries 15 `image-alt` (in one or both viewports) and in some runs 13 more (contrast, link colour, `images/alt-missing`); 15.4 to 15.9 points |
| basf.com eqs.com frame (cross-origin) | flagged (blank document) | audited, 0 findings |
| Origin | derived by hand from the iframe `src` | read from the report (`origin`, `provider`, `frame_host`) |
| Other seven reference sites | no frame, no point | unchanged: no point from any frame |

## Setup

| | |
|---|---|
| Date | 2026-11-21 |
| Binary | `auditmysite 2.1.0`, product code of origin/main at `edd4bce` (#1163 and #1164 merged; build id prints `c2ed261-dirty`: this documentation commit on top plus untracked scratch examples), `cargo build --release` |
| Browser | Chrome 158.0.8127.37, headless, macOS |
| Reference sites | the 8 entries of `tests/fixtures/reference_sites.json` |
| Extra public sites | www.nasa.gov (two YouTube embeds), www.sundance.org (two Rolex clock widgets), stackoverflow.blog (art19 podcast embed), www.spiegel.de (Sourcepoint consent frame). Chosen after looking for embeds in the served HTML; no map or chat-widget site was found that way, so those two kinds of embed are not covered |
| Runs | basf.com 3x, every other site 1x, in two forms (below) |

**Commands.** (1) Binary, for the reported report values and the cross-check:
`auditmysite <url> --full -f json -o <file>.json` (WCAG AA, desktop and mobile, English), 14
runs. (2) A scratch example in the worktree, not committed, running the same pipeline in
process (`run_single_audit(url, &browser, &PipelineConfig::default())`, the default CLI run;
the report is written with `format_json_normalized`) so that the exact violation lists of both
viewports are available: 14 runs, basf.com 3x. Page scores, frames and points below come from the
in-process runs. The overall score of the in-process runs lacks the Security module in several
runs (a module that fell out of that run), so the overall column is the one of the binary run.

## Method

**Origin** is read from the report: a violation with `origin` `same_site_frame` or
`cross_origin_frame` sits in an iframe document, `provider` is the `web-checks` provider of the
frame's source (a known third party) and `frame_host` its host. Nothing is derived from
selectors for frames. For the main document the report says `first_party` throughout; consent
markup of a platform is therefore only found by a selector heuristic of this measurement
(`onetrust`, `ot-`, `cookiebot`, `usercentrics`, `didomi`, `trustarc`, `osano`, `iubenda`,
`borlabs`, `klaro`, `cookieconsent`, `qc-cmp`, `optanon`, `sourcepoint`, ... plus
`wxp-overlay-container` as named in the issue). It is reported in its own column and is not
part of the origin buckets. `web-checks` cannot classify selectors.

**Points** come from the real scorer. The reported accessibility score is
`min(round(0.7 * mobile + 0.3 * desktop), semantic cap of the merged findings)`; each viewport
score is `AccessibilityScorer::calculate_score` over that viewport's violations (the mobile-only
shared page rules `html-content-model` and `css-overflow-hidden` also count on desktop); the cap
is 49 with one Critical finding, 89 with one High or Critical finding of a Level A criterion, 92
with five or more High or Critical findings. The measurement rebuilt exactly this from the
in-process report and **reproduced the reported desktop, mobile and page accessibility score in
all 14 runs** (so there is no estimate in the A11y numbers). The overall score is the module
weighted blend (accessibility 40 %), so overall points are accessibility points times 0.4.

- **Marginal** = page score without the bucket's findings minus the reported score: what the
  page gains if the bucket were not scored.
- **Standalone** = 100 minus the page score of a list that holds only the bucket's findings.

Marginal points are not additive and not proportional to the finding count: the penalty is per
WCAG criterion, capped per criterion, scaled by a diversity factor over the number of violated
criteria and, above 70, compressed; and the semantic cap lets any single Critical finding hold the
whole page at 49 (see "The critical cap" below).

## Result

All numbers are accessibility points on the 0-100 scale; "overall" is accessibility times 0.4.
Marginal / standalone. 0 / 0 means the bucket has no scored finding.

| Site | A11y | Overall | Frames audited (skipped) | Cross-origin frames | Known-provider frames | Same-site frames | Consent markup (heuristic, main document) | First-party |
|---|---|---|---|---|---|---|---|---|
| gov.uk | 98 | 90 | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 2.5 / 2.5 |
| gov.ie | 98 | 91 | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 2.5 / 2.5 |
| casoon.de | 100 | 93 | 0 (1 hidden) | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| mit.edu | 89 | 80 | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 11.0 / 11.0 |
| bundesregierung.de | 82 | 74 | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 18.5 / 18.5 |
| dm.de | 58 | 66 | 0 (5 hidden) | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 41.9 / 41.9 |
| berlin.de | 75 | 72 | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 26.0 / 26.0 |
| basf.com (3 runs) | 54, 54, 54 | 57 to 58 | 2 (1 hidden) | 0 / 0 (eqs.com: no finding) | 0 / 0 | 15.4 to 15.9 / 11.0 to 14.3 (www1.basf.com) | 3.9 to 4.0 / 11.0 | 21.3 to 25.3 / 26.9 |
| nasa.gov | 49 | 55 | 2 (1 hidden) | 0 / 51.0 | 0 / 51.0 (YouTube) | 0 / 0 | 0 / 0 | 0 / 51.0 |
| sundance.org | 49 | 53 | 2 (3 hidden) | 0 / 51.0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 51.0 |
| stackoverflow.blog | 49 | 54 | 1 (1 hidden) | 0 / 51.0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 51.0 |
| spiegel.de | 45 | 55 | 1 (1 hidden) | 0 / 0 | 0 / 0 | 2.2 / 17.0 | 0 / 0 | 38.0 / 52.9 |

`not_loaded` frames: none in any of the 28 audits. The "hidden" frames are skipped before
auditing (reason `hidden`): `about:blank` tracking frames everywhere, and on dm.de also
`app.usercentrics.eu` (cross-domain bridge), two `google.com/recaptcha` anchors and the
`OM_STORAGE_FRAME`. `web-checks` knows Usercentrics and Google, but a skipped frame yields no
finding and costs nothing.

### What the points are made of

- **basf.com** (reference site): the only reference site with points from outside its own
  document. The same-site newsroom frame (`www1.basf.com`, the site's own web platform) costs
  15.4 to 15.9 accessibility points, about 6.2 to 6.4 overall: 15 `image-alt` (all
  runs, in one or both viewports), and in some runs also 8 `contrast/text-insufficient`, 4 `color/link-indistinct` and
  1 `images/alt-missing` (see stability). `html-has-lang` is not reported in any run. The
  eqs.com frame (cross-origin, unknown provider) is clean. Consent markup (OneTrust
  `ul.ot-cat-grp` on mobile, two findings on `div.wxp-overlay-container`) costs 3.9 to 4.0 points
  (about 1.6 overall); of that the OneTrust finding alone is near 0 (its criterion 4.1.2 is
  already at its cap through first-party findings). Frames and markup together: 19.0 to 19.4
  (about 7.6 to 7.8 overall); basf.com would score about 73 instead of 54, above its band
  40 to 60.
- **nasa.gov**: both YouTube embeds are audited (`frame_host` `www.youtube.com`, `provider`
  YouTube). They hold 9 violations per viewport: 1 Critical (`buttons/name-missing`, the video
  info link), 5 `aria/role-invalid`, `aria/attribute-not-allowed`, `aria/attribute-prohibited`
  (High) and one `description-duplicates-name`. Five findings carry the provider label
  `YouTube` in the report (one of them `mixed`: the same rule also fires in the first-party
  navigation). Page-level marginal is 0 because first-party Critical findings (mobile
  navigation buttons) hold the page at 49 as well; the effect of the frame shows on the desktop
  viewport: the desktop score is 49 only through the YouTube Critical and would be about 94.8
  without the frames (viewport-weighted marginal 13.7 points).
- **sundance.org** (Rolex clock widgets, `static.rolex.com`) and **stackoverflow.blog** (art19
  podcast player, `art19.com`): cross-origin frames with a Critical `links/name-missing` each;
  neither host is a known provider. First-party Critical findings hold both pages at 49
  anyway, so the marginal is 0; the standalone value 51.0 shows that the frame alone would pin
  the page at 49.
- **spiegel.de**: the consent frame is Sourcepoint's, but served from the site's own subdomain
  `sp-spiegel-de.spiegel.de`. The report calls it a same-site frame without a provider
  (nine violations per viewport: 8 contrast, 1 prohibited ARIA attribute): 2.2 points. The
  provider list cannot see a provider behind a first-party host name.

### The critical cap

In all three cross-origin cases with findings (nasa.gov, sundance.org, stackoverflow.blog) the
frame carries a Critical finding, and in all three the frame alone holds the page at 49
(standalone 51.0). The page score does not show it on those three sites only because a
first-party Critical finding does the same; on nasa.gov's desktop viewport, where it is the only
Critical, it moves the score from about 94.8 to 49. The penalty sums of frames are far smaller than
this: same-site frames cost 2.2 (spiegel.de) and 15.4 to 15.9 (basf.com), consent markup about 4.

## Stability, basf.com

Three runs in each form, same day, same binary:

| | Binary run 1 / 2 / 3 | In-process run 1 / 2 / 3 |
|---|---|---|
| A11y (page) | 54 / 53 / 54 | 54 / 54 / 54 |
| Viewport scores desktop / mobile | 54/54, 54/53, 53/54 | 53/54, 54/54, 54/54 |
| Overall | 57 / 57 / 57 | (module set differs, see Setup) |
| Same-site frame points (marginal) | 15.5 / 16.5 / 15.9 | 15.9 / 15.4 / 15.4 |
| Findings of the newsroom frame | run 1: 15 `image-alt` in both viewports; runs 2 and 3: 15 `image-alt` in one viewport and 13 findings (8 contrast, 4 link colour, 1 `images/alt-missing`) in the other | run 1: 15 `image-alt` mobile, 13 more desktop; runs 2 and 3: 15 `image-alt` per viewport only |

The loaded frame is a social-media wall whose content differs from load to load. The score
is stable (page A11y 53 to 54; frame cost 15.4 to 16.5, a spread of 1.1) while the finding set
is not: the 15 `image-alt` are present in every one of the 12 runs (in one or both viewports),
the 13 extra findings in 6 of 12, and the viewport they appear in varies. The eqs.com frame was
clean in all 12. The 3 binary runs and the 3 final in-process runs agree on the figures within
these bounds; two earlier in-process batches of 3 runs each (viewport-weighted points only, same
binary) gave page A11y 53 to 54 and a frame cost of 15.4 to 16.5 as well. Audit status alternated between `complete` and
`partial` (a missing rule capability) between runs without moving the score.

## Does the data support a separate weight or cap for cross-origin frames?

An assessment from these numbers; the decision is the maintainer's.

- **Same-site frames: no.** basf.com's newsroom frame is the site's own platform
  (www1.basf.com); the owner can fix it, and it is the largest bucket measured (15.4 to 15.9
  points). A consent frame behind a first-party host (spiegel.de) is small (2.2). Weighing
  these down would hide owner-fixable findings.
- **Cross-origin frames and known providers: the penalty side is not supported, the cap side
  is.** No reference site pays any point for a cross-origin frame or a known provider (basf.com's
  eqs.com frame is clean, dm.de's provider frames are hidden). Among the three other public
  sites with findings in cross-origin frames the penalties are hidden by first-party Critical
  findings, and what the frames would cost alone (51.0) is the semantic cap, not accumulated
  penalty. The one visible case, YouTube on nasa.gov, is entirely the Critical cap on the desktop
  viewport (about 94.8 to 49). A rule that a Critical finding in a cross-origin frame (or of a known
  provider) does not trigger the cap of 49 would change exactly these cases and none of the
  reference bands; a general reduction of frame weights would change basf.com's same-site frame
  only if it covered same-site frames too. Evidence base: 3 sites, one run each; YouTube is the
  only known provider seen live.
- **Attribution holds up.** The provider label appears live for YouTube; unknown hosts
  (eqs.com, art19.com, static.rolex.com) stay `cross_origin_frame` without a provider; a provider
  behind a first-party host name (Sourcepoint on spiegel.de) is a same-site frame without a
  label, and consent-platform markup in the main document is `first_party` by contract. The
  last two are the places where "known provider" under-counts.

## Limits

- The numbers are one day, one machine, one run per site (basf.com three, in two forms). The
  live sites change; the frame content on basf.com changes from load to load.
- The A11y numbers are exact reconstructions of the shipped scorer; the overall numbers assume
  the 40 % module weight (all modules present in the binary runs).
- The provider bucket for frames is `web-checks`'s answer for the frame source and nothing
  else; the consent-markup column is a selector heuristic of this measurement. `wxp-overlay-container`
  is named in the issue as consent markup, but `wxp-` is also the prefix of the site's own web
  platform; its two findings (2.3 of the 3.9 points) are the least certain part.
- Standalone values of 51.0 are the semantic cap (100 minus 49), not an accumulated penalty;
  the penalty-only contribution of a frame that carries a Critical finding was not isolated.
- Frames skipped as `hidden` are not measured at all (dm.de: Usercentrics, recaptcha).
- Embeds of kinds not covered: maps, chat widgets, Vimeo; only YouTube, a podcast player, a clock
  widget, a social wall and a consent frame were seen.
- The extra sites were picked by the author of this measurement, are not part of any band and
  say nothing about the sites at large.
