# Texts of the batch PDF's cross-page consistency section
# (`output::pdf::batch_report::consistency`, `batch-consistency-*`) and of its
# management overview, audit caveats, internal comparison and decision table
# (`output::pdf::batch_report::management`, `batch-management-*`), #936. Which
# message applies, the thresholds and how lists are joined stay in Rust. The
# column headers "Module", "Risk" and "Schema type" are the `batch-col-*`
# messages of report.ftl.
#
# The `-basis-*` and `-identity-*` messages render a text the analysis stores
# in canonical English: the English message is that stored text (`$basis`,
# `$finding`), the German one its translation.

## Cross-page consistency
batch-consistency-title = Cross-page consistency
batch-consistency-intro = Checks that shared elements are consistent across all audited pages: navigation (WCAG 3.2.3), the main heading (H1) per page, canonical URLs (a search-engine signal, not a WCAG criterion) and security headers.
batch-consistency-consistent = Consistent
batch-consistency-missing-on-pages = Missing on { $missing } of { $total } audited pages.
batch-consistency-nav-title = Navigation ({ $main }/{ $total } with main nav, { $skip }/{ $total } with skip link)
batch-consistency-nav-consistent = No navigation inconsistencies detected.
batch-consistency-main-nav = Main navigation
batch-consistency-skip-link = Skip link
batch-consistency-headings-title = Headings ({ $single }/{ $total } with single H1, { $none } missing, { $multiple } multiple)
batch-consistency-headings-consistent = Every page starts with exactly one H1.
batch-consistency-missing-h1 = Missing H1
batch-consistency-missing-h1-detail =
    { $count ->
        [one] 1 audited page has no H1.
       *[other] { $count } audited pages have no H1.
    }
batch-consistency-multiple-h1 = Multiple H1 headings
batch-consistency-multiple-h1-detail =
    { $count ->
        [one] 1 audited page has multiple H1 headings.
       *[other] { $count } audited pages have multiple H1 headings.
    }
batch-consistency-canonical-title = Canonical URLs (www: { $www }, non-www: { $non_www }, missing: { $missing } of { $total })
batch-consistency-canonical-consistent = All pages canonicalize to the same domain variant.
batch-consistency-mixed-hosts = Mixed host strategy
batch-consistency-mixed-hosts-detail = { $www ->
        [one] 1 page uses
       *[other] { $www } pages use
    } www and { $non_www } use non-www canonicals.
batch-consistency-missing-canonical = Missing canonical
batch-consistency-missing-canonical-detail = Missing on { $missing } of { $total } audited pages. Optional: search engines then choose the canonical URL themselves. It only matters where URL variants exist (parameters, www/non-www); that was not checked here.

## Cross-page WCAG criteria
batch-consistency-wcag-title = Cross-page WCAG assessment
batch-consistency-col-criterion = Criterion
batch-consistency-col-basis = Assessment basis
batch-consistency-status-no-inconsistency = No inconsistency detected
batch-consistency-status-warning = Warning
batch-consistency-status-manual-review = Manual review
batch-consistency-basis-navigation = { $basis }
batch-consistency-basis-identification = { $basis }
batch-consistency-basis-multiple-ways = { $basis }

## Consistent help, orphan pages, schema entities
batch-consistency-help-title = Consistent help (WCAG 3.2.6): deviations
batch-consistency-help-more = { $count } further deviations
batch-consistency-see-json = see JSON report
batch-consistency-orphan-title = Pages without an observed inbound link within the audited URLs
batch-consistency-orphan-row = No inbound link observed
batch-consistency-orphan-note = Only links between the audited pages were counted here; the links of a homepage outside the sample are not included. A page listed here can still be linked from the homepage, pages outside the sample, a search or a sitemap page — it is a candidate, not a confirmed orphan.
batch-consistency-schema-conflicts-title = Schema entity identity conflicts

## Structured data
batch-consistency-structured-types-title = Structured-data distribution
batch-consistency-col-audited-pages = Audited pages
batch-consistency-blockers-title = Recurring structured-data blockers
batch-consistency-blocker-detail = Required condition missing on { $count ->
        [one] 1 audited page
       *[other] { $count } audited pages
    }
batch-consistency-parity-title = Recurring schema/content mismatches
batch-consistency-parity-detail = Visible content differs on { $count ->
        [one] 1 audited page
       *[other] { $count } audited pages
    }
batch-consistency-matrix-col-types = Suggested / detected
batch-consistency-coverage-complete = Covered
batch-consistency-coverage-opportunity = Optional addition
batch-consistency-coverage-manual-review = Review manually
batch-consistency-coverage-not-applicable = Not applicable
batch-consistency-identity-title = Site-wide entity identity
batch-consistency-identity-row = Organization identity
batch-consistency-identity-ids-differ = { $finding }
batch-consistency-identity-id-missing = { $finding }

## Security headers and topics
batch-consistency-headers-title = Security headers across pages ({ $checked ->
        [one] 1 page
       *[other] { $checked } pages
    } with a security check)
batch-consistency-header-partial = Sent on { $present } of { $checked } pages; missing e.g. on { $examples }.
batch-consistency-header-values = { $values } different values across { $present } pages.
batch-consistency-header-uniform = Uniform
batch-consistency-header-uniform-set = same value on all pages: { $headers }
batch-consistency-header-uniform-missing = missing on all pages: { $headers }
batch-consistency-overlap-title = Strong topic overlap between pages
batch-consistency-overlap-detail = Topic-term similarity { $percent } % · shared terms: { $terms }

## Audit caveats
batch-management-flags-title = Recurring audit caveats
batch-management-flag-consent-banner = Consent banner
batch-management-flag-bypass-blocks-untested = Skip link verification
batch-management-flag-conflicting-signal = Conflicting signal
batch-management-flag-viewport-gap = Desktop/mobile difference
batch-management-flag-incomplete-audit = Incomplete measurement scope
batch-management-flag-consent-wall-artifact = Consent wall artifact
batch-management-flag-other = Audit note
batch-management-flag-pages = { $count ->
        [one] { $count } affected page
       *[other] { $count } affected pages
    }

## Assessment and key points
batch-management-risks-title = Management risk view

## Internal comparison
batch-management-comparison-too-few = Cross-page comparison needs at least 3 URLs for meaningful domain-wide averages; this report covers { $count }. See the per-URL detail instead.
batch-management-comparison-title = Internal comparison by module
batch-management-col-best-url = Best URL
batch-management-col-weakest-url = Weakest URL
batch-management-comparison-same = { $score }/100 on all { $count } measured pages
batch-management-outliers-title = Outlier URLs
batch-management-outlier = { $url }: { $score } / 100 ({ $delta } points below average)

## Decision-oriented top actions
batch-management-score-gain = up to +{ $points }
batch-management-score-gain-note = Score effect: points the average accessibility score would gain if only this action were done on every page, simulated per page with the score calculation. The figures do not add up: caps and the weighting of several criteria make the gain of each action depend on what else remains open.
batch-management-root-recurring = recurs on { $count } pages, no shared cause established
batch-management-root-one-page = { $count } occurrences on one page
batch-management-root-page-specific = page-specific
