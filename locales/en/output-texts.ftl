# Texts of the report builder helpers, the batch builder, the PDF detail modules
# for performance, SEO, tech stack and best practices, and the display mode
# texts (`output::builder::helpers`, `output::builder::batch`,
# `output::pdf::detail_modules::{indicators,seo,performance}`, `display`,
# `output-*`), #936. Which message applies, the thresholds, the number formats
# and the values that go into a message stay in Rust. "Yes", "No", "Metric"
# and "Value" are the messages `yes`, `no`, `col-metric` and `col-value` of
# report.ftl.

## Report title and security labels
output-report-title = Website Health Report
output-report-subtitle = Automated accessibility audit with optional website quality modules.
output-header-tier-baseline = Baseline
output-header-tier-architecture-dependent = Architecture-dependent
output-header-tier-context-dependent = Context-dependent
output-header-tier-not-assessable = Not assessable remotely
output-issue-kind-missing = Missing
output-issue-kind-misconfigured = Misconfigured

## Batch builder
output-batch-distribution-thin-content = High share of thin-content pages: this can limit informational value and SEO potential.
output-batch-distribution-no-editorial = No editorial content pages: knowledge-building and informational search intents are barely served.
output-batch-distribution-marketing-dominated = Marketing and landing pages dominate: more structured in-depth content would balance the domain.
output-batch-lever-fallback = Maintain results
output-batch-cover-title = Web Accessibility Batch Audit Report
output-batch-cover-urls = { $count } URLs audited
output-batch-blocking-pages-analyzed = Pages analyzed
output-batch-blocking-pages-blocking = Pages with blocking
output-batch-blocking-distinct = Distinct blocking resources
output-batch-blocking-third-party = Third-party traffic total
output-batch-blocking-pages-of = { $pages } of { $total }
output-batch-blocking-loads = { $distinct } ({ $loads } loads across pages)

## Display modes
output-display-site-default = Site default (no mode chosen)
output-display-chosen = { $mode } (chosen via --display)
output-display-chosen-reduced-motion = { $mode } (chosen via --display, reduced motion emulated)
output-display-offered-none = None (no html[data-display])
output-display-offered = { $modes } (rendered: { $active })

## Tech stack and best practices
output-ts-takeaway-none = No common technologies were detected.
output-ts-takeaway-count = { $count } technologies detected.
output-bp-takeaway-not-measured = Not measured.
output-bp-takeaway-clean = No console errors or known-vulnerable libraries detected.
output-bp-takeaway-issues = Console errors or vulnerable libraries were detected — see below.

## SEO
output-seo-schema-none = none
output-seo-schema-explainer = Schema.org marks up structured data for search engines; the maturity level shows how completely it is used on the page.
output-seo-schema-count = { $count } { $count ->
    [one] schema
   *[other] schemas
}

## Performance
output-perf-further-metrics = Further lab metrics
output-perf-score-per-viewport = Performance score per viewport: 0–100, higher is better.
output-perf-score-basis = The performance score of this report ({ $score }) is the { $basis } view; the other view is scored separately.
output-perf-vitals-legend = LCP: largest visible content (good ≤ 2.5 s); FCP: first visible content (good ≤ 1.8 s); CLS: layout stability (good ≤ 0.1); TTFB: server response (good ≤ 0.8 s). Lower is better for time and shift values.
output-perf-desktop-lab-metrics = Desktop — lab metrics
output-perf-mobile-lab-metrics = Mobile — lab metrics
