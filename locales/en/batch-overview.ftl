# The texts of the batch PDF's overview page, page comparison, causes, next
# step and technical part (`output::pdf::batch_report::{glance, compare,
# causes, technical}`, `batch-overview-*`, `batch-compare-*`, `batch-causes-*`,
# `batch-next-*`, `batch-tech-*`), #1212. The German
# counterpart is `locales/de/batch-overview.ftl` with the same keys.

## Overview page: three questions
batch-overview-title = Overview
batch-overview-kicker = How the pages score, what repeats and the recommendation — from the measured data.
batch-overview-q-scores = How do the pages score?
batch-overview-a-scores = { $min } to { $max } per page; { $passed } of { $total } { $total ->
        [one] page
       *[other] pages
    } passed, critical or high findings on { $severe } of { $total }.
batch-overview-a-scores-equal = { $total ->
        [one] One audited page
       *[other] All { $total } pages score the same
    }; { $passed } of { $total } { $total ->
        [one] page
       *[other] pages
    } passed, critical or high findings on { $severe } of { $total }.
batch-overview-q-repeats = What repeats?
batch-overview-a-repeats = { $types } { $types ->
        [one] finding type
       *[other] finding types
    } on { $pages } of { $total } pages: { $shared } on several pages ({ $all } of them on all), { $specific } on one page only.
batch-overview-a-repeats-top = Most important: { $list }.
batch-overview-a-repeats-none = No findings on the audited pages.
batch-overview-q-pick = Which page deserves a single report?
batch-overview-a-pick = { $page }: { $reason }
batch-overview-a-pick-none = No page stands out: no findings were measured on the audited pages.
batch-overview-pick-reason = { $types } of the sample's { $batch_types } { $batch_types ->
        [one] finding type
       *[other] finding types
    } occur here, { $severe } of them critical or high ({ $shared_on } of { $shared } recurring ones).
batch-overview-pick-tied = The accessibility scores are equal; most severe finding of another module ({ $module }): "{ $title }" ({ $severity }).
batch-overview-pick-lead = Check the recurring cause "{ $title }" first.

## Page comparison
batch-compare-title = Page comparison
batch-compare-intro = The audited pages by accessibility, lowest value first. "Findings" counts findings grouped per page (one rule on one page is one finding); the figures on the cover count single occurrences.
batch-compare-col-rank = No.
batch-compare-col-page = Page
batch-compare-col-accessibility = Accessibility (0-100)
batch-compare-col-severe = Critical/high (findings)
batch-compare-col-focus = Single report, because ...
batch-compare-focus = { $types } { $types ->
        [one] finding type
       *[other] finding types
    }, { $severe } critical or high
batch-compare-focus-only = ; only here: { $list }
batch-compare-focus-none = no findings
batch-compare-pick = Recommended: { $text }
batch-compare-more = { $count ->
        [one] One more page
       *[other] { $count } more pages
    } with middle values are in the JSON report.

## Causes
batch-causes-title = Shared and page-specific causes
batch-causes-intro = One card per finding type with the largest effect: how far it reaches, where it probably comes from and what to do. The finding id links the card to the JSON report; the evidence per page is in the single report of the page.
batch-causes-reach-page = { $reach } ({ $page })
batch-causes-more-json = { $count ->
        [one] One more finding type is
       *[other] { $count } more finding types are
    } in the JSON report.

## Next step
batch-next-title = Next step
batch-next-body = Create a single report for "{ $page }" to see the findings of this page in detail (command: { $command }).

## Technical part
batch-tech-title = Technical part and method
batch-tech-intro = The audit frame, how the recommendation is calculated and the figures behind the overview. Details per URL are in the JSON report and in the single reports.
batch-tech-method-title = Method
batch-tech-pick-rule = Recommendation for a single report: each page gets a weight from its distinct finding types (critical 4, high 3, medium 2, low 1 per finding type; the number of occurrences does not count). The page with the highest weight is recommended; on a tie the one with the lower accessibility score, then the one with more critical or high finding types, then the URL in alphabetical order. When all pages have the same accessibility score, the recommendation also names the most severe finding of another module on the page. It is measured on the audited pages only.
batch-tech-seo-title = SEO details across pages
batch-tech-seo-intro = Values that are the same on several pages or missing on pages — when missing everywhere, it is usually the template.
batch-tech-detail-title = Detail chapters (report level technical)
batch-tech-detail-intro = The following chapters list per-page details that the JSON report also holds.
batch-tech-consistency-detail-title = Consistency across pages: details
batch-tech-consistency-detail-intro = Pages without an observed inbound link, schema conflicts, structured data per page and topic overlaps.

## Audit caveats
batch-flag-text-consent-wall-artifact = On these pages the consent dialog may have been measured instead of the page content. A run with --dismiss-consent repeats the check without the dialog.
batch-flag-text-other = There is a further note on how these pages were measured; details are in the JSON report under audit_flags.
