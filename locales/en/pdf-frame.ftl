# Texts of the single PDF's frame (`output::pdf` mod.rs, `pdf-frame-*`), #936:
# the cover labels, the table of contents title, the three part intros with
# their audience titles and bodies, the confidence notes with the audit flag
# titles and customer texts, and the side checks that got no answer. Which
# message applies, the cover band phrase and scope line (messages of the
# modules that own them) and the joining of the side check names stay in
# Rust. The titles of the audit flags consent banner, skip link verification,
# conflicting signal, desktop/mobile difference, consent wall artifact and
# the fallback "Audit note" are the `batch-management-flag-*` messages of
# batch-consistency.ftl.

## Cover
pdf-frame-cover-score-label = ACCESSIBILITY · AUTOMATED SCOPE
pdf-frame-cover-findings-label = WCAG OCCURRENCES
pdf-frame-cover-modules-label = MODULES IN THE OVERALL SCORE · 0–100 WITHIN THE AUTOMATED SCOPE · HIGHER IS BETTER
pdf-frame-cover-critical-label = critical/high
pdf-frame-cover-no-critical = 0 critical/high
pdf-frame-cover-indicators-label = DERIVED INDICATORS · NOT PART OF THE OVERALL SCORE

## Table of contents
pdf-frame-toc-title = Contents

## Part dividers
pdf-frame-part-audience-title = Audience
pdf-frame-part1-title = Findings by Root Cause
pdf-frame-part1-intro = Which underlying causes explain the most findings, and the prioritized plan to fix them.
pdf-frame-part1-audience = Site owners, decision-makers, and developers. This part explains what is driving the findings and what to do next.
pdf-frame-part2-title = Technical Details for Developers
pdf-frame-part2-intro = Detailed WCAG violations, HTML evidence, and code examples grouped by systemic components vs. local instances.
pdf-frame-part2-audience = Developers and QA teams. Read this part to implement fixes for all detected violations.
pdf-frame-part3-title = Quality Analyses
pdf-frame-part3-intro = Search engine optimization, load speed, mobile layout, and security header metrics.
pdf-frame-part3-audience = Marketing, SEO specialists, and product owners. Read this part to optimize discoverability and performance.

## Confidence notes
pdf-frame-confidence-title = Notes on report confidence
pdf-frame-confidence-base = Automated findings are reliable for detectable patterns, but context-dependent WCAG criteria, semantic content quality and user journeys still require manual verification. Indicator modules such as AI visibility, content visibility and UX are best-effort signals, not guarantees.
pdf-frame-confidence-caveats-title = Audit caveats

## Audit flag titles
# Aligned with the "Coverage" framing on the management summary page: a normal
# stability-budget limitation must not read as "the audit is broken".
pdf-frame-flag-title-incomplete-audit = Limited test coverage
pdf-frame-flag-title-side-check-unanswered = Side check without answer

## Audit flag customer texts
pdf-frame-flag-text-consent-banner = A consent banner was detected. Parts of the page may have been hidden or not fully measurable without consent.
pdf-frame-flag-text-bypass-blocks-untested = A skip link exists, but the usage journey indicates that keyboard focus may not reach the intended target.
pdf-frame-flag-text-conflicting-signal = Two checks report different signals. Treat this result as a review point rather than a final conclusion.
pdf-frame-flag-text-viewport-gap = Desktop and mobile results differ strongly. The page experience should be checked separately for both viewports.
# The same sentence as the management summary's partial-run callout: one
# wording for the same limitation.
pdf-frame-flag-text-incomplete-audit = Individual measurements hit a stability/retry budget and are excluded from the reported scores, which describe only the successfully measured scope.

## Side checks without an answer
pdf-frame-side-check-www = www/non-www consolidation (the other host variant did not answer)
pdf-frame-side-check-certificate = certificate details (not reported by the browser)
pdf-frame-side-check-other = an additional check
pdf-frame-side-check-text = No result for: { $list }. The audited page was measured in full; the rating is not provisional because of this.
