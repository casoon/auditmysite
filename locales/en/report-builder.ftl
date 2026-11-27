# Texts of the single report's builder, #936: positive signals and aspects,
# the executive key points and score driver note, the audit quality note, the
# site-quality score explanation, the page health rows and the performance,
# security and mobile detail rows (`output::builder::single::{executive,
# positive, serp, modules_block, module_details::{performance, platform}}`
# and the view model builder in `output::builder::single`), and the
# experimental indicator lines of the terminal table (`output::terminal`),
# `report-builder-*`. Which message applies, the thresholds, the counts and
# the number formats stay in Rust. "Accessibility" and "Security" are the
# messages `module-accessibility` and `module-security` of report.ftl.

## Positive signals (pattern titles)
report-builder-pattern-main-navigation = Semantic main navigation
report-builder-pattern-disclosure-menu = Disclosure menu
report-builder-pattern-modal-dialog = Modal dialog
report-builder-pattern-tab-list = Tab list
report-builder-pattern-skip-link = Skip link
report-builder-pattern-accordion = Accordion
report-builder-pattern-easy-language = Easy-language version
report-builder-pattern-form = Form
report-builder-pattern-add-to-cart = Add to cart
report-builder-pattern-quantity-stepper = Quantity stepper
report-builder-wcag-signal = WCAG signal

## Positive aspects
report-builder-aspect-no-violations = No automatically detectable violations found.
report-builder-aspect-stable-base = Stable base quality with focused, prioritizable remaining items.
report-builder-aspect-performance = Stable load times and overall responsive page build-up.
report-builder-aspect-seo = Clean foundation for discoverability, structure and meta data.
report-builder-aspect-security = Key security mechanisms are fundamentally in place.
report-builder-aspect-mobile = The site is usable and readable on small displays.
report-builder-aspect-base-area = Base structure
report-builder-aspect-base = The site is fundamentally functional and reachable.

## Executive key points
report-builder-key-critical = Multiple critical WCAG barriers block assistive technology users — screen readers and keyboard navigation are affected.
report-builder-key-high = Significant accessibility gaps reduce usability for users with disabilities — no complete access barriers, but relevant friction points.
report-builder-key-main-issue = Main issue: { $title } ({ $share }% of all critical/high findings)
report-builder-key-most-frequent = Most frequent issue: { $title }
report-builder-key-legal = WCAG Level A/AA findings with BFSG/EAA relevance detected automatically — manual review needed for a defensible BFSG classification
report-builder-key-no-level-violations = No Level A/AA violations, but structural weaknesses
report-builder-key-audit-notes = Audit notes present — individual signals should be verified manually.
report-builder-key-no-barriers = No automatically detectable critical barriers — manual review recommended.
report-builder-key-scope = WCAG scope: { $automated } of about { $total } WCAG 2.2 AA criteria are checked automatically; manual review remains required for context-dependent criteria.
report-builder-driver-none = Score matrix: no negative driver detected in the weighted accessibility topics.
report-builder-driver-areas = Score matrix: strongest negative drivers are { $areas }; weighting includes semantics, forms, keyboard, focus, images, ARIA, headings and landmarks.

## Audit quality note and metric label
report-builder-stability-view = { $view } view after { $ms } ms
report-builder-stability-mutations = , still { $count } DOM changes
report-builder-stability-individual = individual measurements
report-builder-quality-partial = The automated audit completed successfully. Individual measurements hit a stability/retry budget and are excluded from the reported scores, which describe only the successfully measured scope. Affected: { $affected }.
report-builder-quality-insufficient = This audit run has insufficient data quality: several measurements failed. Scores and findings below may be incomplete or unreliable. See the methodology appendix for details.
report-builder-quality-insufficient-summary = This audit run has insufficient data quality: several measurements failed. Scores and findings may be incomplete or unreliable. The standard report (without --report-level) lists the details.
report-builder-metric-quick-wins = Quick wins

## Site-quality score explanation
report-builder-site-quality-indicators = Indicator modules ({ $modules }) are shown separately and do not change the site-quality score.
report-builder-site-quality-dual = The site-quality score uses the dual-viewport result: 70% mobile and 30% desktop. Security contributes 10% when active.{ $indicators }
report-builder-site-quality-weighted = The site-quality score is the weighted result of the assessed areas: { $weights }.{ $indicators }
report-builder-site-quality-weight = { $name } { $pct }%
report-builder-site-quality-weight-default = Accessibility 100%

## Page health rows
report-builder-page-health-url-length = URL length
report-builder-page-health-url-length-value = { $count } characters
report-builder-page-health-path-depth = Path depth
report-builder-page-health-query-parameters = Query parameters
report-builder-page-health-self-redirect = Self-redirect
report-builder-page-health-target-url = Target URL
report-builder-page-health-validator-no-info = No additional information available

## Performance details
report-builder-perf-co2e = CO2e per view
report-builder-perf-dom-nodes = DOM nodes
report-builder-perf-guideline-dom-nodes = Guideline: max. 800 nodes
report-builder-perf-load-time = Load time
report-builder-perf-guideline-load-time = Guideline: max. 3.0 s
report-builder-perf-dom-content-loaded = DOM Content Loaded
report-builder-perf-guideline-dom-content-loaded = Guideline: max. 1.8 s
report-builder-perf-blocking-scripts = Blocking Scripts
report-builder-perf-blocking-css = Blocking CSS
report-builder-perf-blocking-transfer = Blocking Transfer
report-builder-perf-third-party = Third-Party
report-builder-perf-first-party = First-Party
report-builder-perf-no-measurable-impact = No measurable impact on the measured vitals — still worth fixing preventively, since slow connections or cold caches can be affected more severely.

## Security details
report-builder-security-valid-certificate = Valid certificate
report-builder-security-certificate-subject = Certificate subject
report-builder-security-certificate-issuer = Certificate issuer
report-builder-security-expires-in = Expires in
report-builder-security-expires-days = { $days } days
report-builder-security-chain-length = Chain length
report-builder-security-certificate-error = Certificate error
report-builder-security-header-present = Present
report-builder-security-header-missing = Missing
report-builder-security-band-note-open = The wording rating sits below the band of the number ({ $score } alone would read "{ $numeric }"): { $count ->
        [one] 1 open finding is
       *[other] { $count } open findings are
    } listed below. The score itself is unchanged.
report-builder-security-band-note-severe = The wording rating sits below the band of the number ({ $score } alone would read "{ $numeric }"): { $count ->
        [one] 1 open finding of high severity is
       *[other] { $count } open findings of high severity are
    } listed below. The score itself is unchanged.
report-builder-security-score-start = Starting value
report-builder-security-score-not-charged = 0 (not charged)
report-builder-security-score-vulnerable-libraries = Known-vulnerable libraries (Best Practices)
report-builder-security-score-total = Security score

## Mobile details
report-builder-mobile-scalable = Scalable
report-builder-mobile-configured = Correctly configured
report-builder-mobile-targets-total = Total
report-builder-mobile-targets-adequate = Meets the recommendation (≥44px)
report-builder-mobile-targets-small = Too small
report-builder-mobile-targets-crowded = Too close together
report-builder-mobile-font-base = Base font size
report-builder-mobile-font-smallest = Smallest font
report-builder-mobile-font-legible = Legible text
report-builder-mobile-font-relative = Relative units
report-builder-mobile-sizing-fits = Fits viewport
report-builder-mobile-sizing-no-scroll = No horizontal scroll
report-builder-mobile-sizing-responsive-images = Responsive images
report-builder-mobile-context = { $count } in { $area }
report-builder-mobile-locally-scoped = — locally scoped
report-builder-mobile-small-targets = { $base } { $count } touch targets smaller than the ergonomic recommendation (44×44 px){ $context }{ $scope }.

## Terminal table
report-builder-terminal-experimental-title = Experimental indicators (unvalidated, no effect on score or verdict)
report-builder-terminal-no-score = { $label }: no score
report-builder-terminal-content-visibility = Content Visibility: { $problems } of { $signals } signals flagged
report-builder-terminal-commerce =
    Commerce: { $findings ->
        [one] 1 finding
       *[other] { $findings } findings
    }

# Terminal table chrome (single and batch): the table's own labels follow the run language.
report-builder-terminal-metric-accessibility = Accessibility
report-builder-terminal-metric-site-quality = Site quality (optional modules)
report-builder-terminal-metric-grade = Grade
report-builder-terminal-metric-violations = Violations
report-builder-terminal-metric-duration = Duration
report-builder-terminal-metric-urls = URLs
report-builder-terminal-metric-passed = Passed
report-builder-terminal-metric-failed = Failed
report-builder-terminal-metric-warned = Warned
report-builder-terminal-metric-avg-score = Avg. score
report-builder-terminal-details-hint = use --format json or --format pdf for all
report-builder-terminal-batch-title = Batch Audit — { $domain }
report-builder-terminal-scope-title = Scope
report-builder-terminal-scope-sample = { $audited } of { $total } discovered URLs ({ $source })
report-builder-terminal-baseline-title = Baseline
report-builder-terminal-score-basis-title = Score basis
report-builder-terminal-score-basis = Scores cover { $audited } of { $total } URLs; { $failed } could not be audited
report-builder-terminal-top-rules = Top recurring rules
report-builder-terminal-fix-audit-error = Fix audit error for { $url }: { $error }
