# Texts of the single PDF's page sections (`output::pdf::single_report`,
# `single-report-*`), #936: the part dividers, the management summary, the
# findings classification and matrix, the appendix with its manual-review
# notes, the recognized structural patterns and the dual viewport summary.
# Which message applies, the thresholds and how lists are joined stay in Rust.
# The severity labels, the overall rating title, verdict words and reasons,
# the module names and the column headers "Priority", "Occurrences" and
# "Effort" are messages of report.ftl.

## Part dividers
single-report-part-eyebrow = PART { $part } OF 3
single-report-part-eyebrow-appendix = PART 3 OF 3 · TECHNICAL APPENDIX

## Management summary
single-report-scope-line = Audited: 1 URL · Desktop and Mobile · no website crawl
single-report-management-title = Management Summary
single-report-management-intro = Overall status, key risks, prioritized actions, and leverage at a glance.
single-report-management-eyebrow = MANAGEMENT SUMMARY
single-report-wcag-occurrences = WCAG occurrences
single-report-occurrences-definition = Occurrences = individual affected elements, not the number of distinct rules — see the appendix for the full breakdown.
single-report-evidence-note = Counted here are confirmed WCAG violations only. In addition: { $warnings } heuristic accessibility warning(s) and { $manual } point(s) on this page to confirm by hand — documented, but not scored. A further { $manual_only } WCAG criteria can only be checked manually (appendix).
single-report-overall-verdict = Overall Verdict
single-report-audit-quality = Audit Quality
single-report-coverage = Coverage
single-report-clean-scope = This result applies to the automated audit scope only ({ $automated } of ~{ $total } testable WCAG 2.2 AA criteria); criteria requiring manual review are listed in the appendix.
single-report-quality-profile = Quality profile

## Indicator module dashboards
single-report-dashboard-strong = What works particularly well
single-report-dashboard-weak = Where optimization pays off
single-report-dashboard-good-but-gated = The additional indicator modules (UX, Journey, …) are in good shape, but barriers elsewhere in this report (operability, possible legal relevance) still require action — see Management Summary.
single-report-dashboard-all-good = The additional indicator modules (UX, Journey, …) are in good shape — no module needs prioritized optimization.

## Overall rating
single-report-rating-body = { $rating } — automated check { $verdict }.
single-report-rating-provisional = This run did not complete in full, so the rating is provisional and describes only what was measured successfully.
single-report-rating-note = The rating is a band label for the accessibility score within the automated scope, not a statement of conformance.

## Risks and strengths
single-report-risks-none = No priority risks identified within the automated audit scope. This result applies to the automated audit scope only ({ $automated } of about { $total } testable WCAG 2.2 AA criteria); criteria requiring manual review are listed in the appendix.
single-report-risks-title = Key Risks
single-report-risks-title-count = Key Risks ({ $count } identified)
single-report-strengths-title = Strengths within the audited scope

## Key measures
single-report-measures-title = 5 Key Measures
single-report-measures-none = No urgent actions required.
single-report-measure-more = (+{ $count } more of the same kind)

## Findings
single-report-findings-title = Classification of Findings
single-report-findings-intro = All technical findings, categorized by systemic template relevance and individual page occurrences.
single-report-findings-eyebrow = FINDINGS
single-report-strip-systemic-mandatory = Systemic (mandatory)
single-report-strip-systemic-optimization = Systemic (optimization)
single-report-strip-local-mandatory = Local (mandatory)
single-report-strip-local-optimization = Local (optimization)
single-report-matrix-title = Findings Matrix
single-report-matrix-intro = One row per finding group, sorted by priority — an overview to prioritize by before the detail cards below expand on each row.
single-report-matrix-col-finding = Finding
single-report-matrix-systemic = Systemic · { $count }
single-report-matrix-local = Local · { $count }
single-report-matrix-more = { $count } further findings are listed in the detail cards below.
single-report-systemic-mandatory-title = Systemic Template & Component Issues (WCAG A/AA)
single-report-systemic-mandatory-intro = This pattern suggests a reused component or template — confirming that requires evidence from additional pages. A central fix would likely also resolve the cause on other pages sharing the same pattern.
single-report-systemic-mandatory-eyebrow = SYSTEMIC COMPLIANCE
single-report-systemic-optimization-title = Systemic Quality & SEO Optimizations
single-report-systemic-optimization-intro = These recommendations recur within this single page, which suggests a shared template — confirming that would require checking additional pages.
single-report-systemic-optimization-eyebrow = SYSTEMIC OPTIMIZATION
single-report-local-mandatory-title = Local & Editorial Findings (WCAG A/AA)
single-report-local-mandatory-intro = Single instances of accessibility barriers affecting specific pages, images, or editorial content, usually requiring individual resolution.
single-report-local-mandatory-eyebrow = LOCAL COMPLIANCE
single-report-local-optimization-title = Additional Quality & SEO Recommendations
single-report-local-optimization-intro = Usability, performance, or SEO recommendations for specific pages.
single-report-local-optimization-eyebrow = LOCAL OPTIMIZATION

## Appendix
single-report-appendix-title = Appendix & Methodology
single-report-appendix-intro = Technical scope, methodology, WCAG coverage, and the complete violations list.
single-report-appendix-elements-affected = { $count ->
        [one] { $count } element affected
       *[other] { $count } elements affected
    }
single-report-appendix-selectors = Selectors: { $list }
single-report-appendix-eyebrow = APPENDIX
single-report-manual-only-title = Structurally manual-only criteria
single-report-manual-only-focus-order = 2.4.3 · Focus Order
single-report-manual-only-focus-order-text = Whether the technical focus order makes sense in context depends on the page's content and cannot be assessed automatically.
single-report-manual-only-error-identification = 3.3.1 · Error Identification
single-report-manual-only-error-identification-text = Whether an error message is actually understandable to people is a content question, not a structural check.
single-report-manual-only-reflow = 1.4.4 / 1.4.10 · Resize Text & Reflow
single-report-manual-only-reflow-text = Whether the page stays practically usable at 400% zoom goes beyond the technical reflow structure and requires manual review.
single-report-manual-only-non-text = 1.1.1 · Non-text Content
single-report-manual-only-non-text-text = Whether alt text correctly describes the image content can only be judged by a person; automated checks can only confirm that alt text is present at all.
single-report-manual-only-media = 1.2.1 / 1.2.2 · Time-based Media
single-report-manual-only-media-text = Whether a caption or transcript summary correctly reflects a video's content cannot be checked automatically.
single-report-manual-review = Manual review
single-report-manual-review-fallback = Verify this criterion manually on the rendered page.
single-report-warning = Warning
single-report-warning-fallback = Confirm this heuristic signal manually.
single-report-manual-and-heuristic-title = Manual review and heuristic signals
single-report-journey-coverage = Interactive coverage: { $completed } of { $attempted } attempted journeys completed; { $failed } failed, { $skipped } skipped{ $budget }.
single-report-journey-budget-exhausted = ; budget exhausted
single-report-findings-list-title = Complete Findings List
single-report-findings-list-intro = Raw audit data and the full list of detected violations.
single-report-json-note = The accompanying JSON report contains the complete machine-readable issue list with selectors, occurrences, and all detail data for automated processing.
single-report-json-note-title = Raw data & processing

## Recognized structural patterns
single-report-patterns-title = Recognized structural patterns
single-report-pattern-strong = Strong
single-report-pattern-present = Present
single-report-patterns-panel-title = Recognized in the accessibility tree; no interaction test

## Dual viewport summary
single-report-dual-title = Dual viewport summary
single-report-dual-desktop = Accessibility - Desktop
single-report-dual-mobile = Accessibility - Mobile
single-report-dual-overall = Accessibility - overall
single-report-dual-weighted = 70/30 weighted
single-report-preview = Preview
single-report-preview-captured = Captured
single-report-screenshot-failed = Screenshot capture failed: { $reason }
single-report-dual-occurrences = WCAG occurrences before cross-viewport consolidation: { $desktop } on desktop and { $mobile } on mobile. The normalized combined finding list contains { $combined } occurrences; this count is used for evidence and remediation, not as a third score.
single-report-dual-explanation = Each viewport accessibility score reflects the severity and variety of automatically detected WCAG findings; 100 means no issue was detected within the automated scope. Lower values mean more remediation pressure. { $occurrence_context }The displayed accessibility overall score is calculated from mobile at 70% and desktop at 30%.

## AI & Trust chapter
single-report-trust-title = AI & Trust
single-report-trust-intro = Technical preparation, AI readability and content visibility — unweighted indicators from technical signals.
single-report-trust-eyebrow = TRUST
