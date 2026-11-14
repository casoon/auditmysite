# Texts of the batch PDF's keyboard journey summary, module name, impact
# summary, commerce roll-up, cover contents, EN 301 549 clause roll-up and
# overview cards (`output::pdf::batch_report::interactive`, `portfolio`,
# `appendix`, `overview`, `batch-report-*`), #936. Which message applies, the
# thresholds, the number formats and the clause titles and disclaimers that
# `wcag::en301549` owns stay in Rust. The journey title, the severity labels,
# the module names, "Share", "Overview" and "Linked" are the messages
# `a11y-details-journey-title`, `severity-*`, `module-*`, `batch-col-share`,
# `pdf-commerce-overview-title` and `pdf-commerce-linked` of
# accessibility-details.ftl and report.ftl.

## Keyboard journey summary
batch-report-interactive-intro = Interactive tests verify whether focus management, state transitions, and keyboard navigation behave correctly — not just whether the initial page tree is well-structured.
batch-report-interactive-table-title = Issues by Category
batch-report-interactive-col-category = Category
batch-report-interactive-col-affected = Affected URLs
batch-report-interactive-col-severity = Max. Severity
batch-report-interactive-cat-tab-order = Tab Order
batch-report-interactive-cat-focus-trap = Focus Trap (Modal)
batch-report-interactive-cat-state-transition = State Transitions
batch-report-interactive-cat-focus-restoration = Focus Restoration
batch-report-interactive-cat-form-error = Form Error Announcement
batch-report-interactive-cat-spa-navigation = SPA Navigation
batch-report-interactive-cat-hidden-focusable = Hidden Focusable
batch-report-interactive-cat-skip-link = Skip Link
batch-report-interactive-cat-focus-indicator = Focus Indicator
batch-report-interactive-cat-menu = Menu / Navigation
batch-report-interactive-cat-tabs = Tab Widget
batch-report-interactive-cat-link-text = Link Text
batch-report-interactive-cat-landmark = Landmark
batch-report-interactive-cat-media-controls = Media Controls
batch-report-interactive-cat-heading-outline = Heading Outline

## Module names
batch-report-module-html-conformance = HTML Conformance

## Impact summary
batch-report-impact-user = User
batch-report-impact-business = Business
batch-report-impact-risk = Risk
batch-report-impact-business-critical = The audited pages contain critical barriers for certain user groups; how far they reach beyond these pages is not measured.
batch-report-impact-business-high = Individual functional areas are problematic for users with disabilities.
batch-report-impact-business-low = Low impact on the audited pages — no critical or high barriers detected automatically.
batch-report-impact-legal-flags = WCAG Level A/AA violations detected automatically — manual review required for a defensible BFSG classification.
batch-report-impact-legal-none = No critical violations detected automatically — manual review recommended for full classification.

## Commerce roll-up
batch-report-commerce-intro = Product structured-data completeness and mandatory-page linking, aggregated across the audited set.
batch-report-commerce-scope-note = Checks product structured data and mandatory-page linking across the audited pages — not a checkout, payment-processing, or cart audit (this tool has no cross-page session state).
batch-report-commerce-product-pages = Product pages
batch-report-commerce-product-pages-audited =
    { $count } { $count ->
        [one] product page
       *[other] product pages
    } audited
batch-report-commerce-trust-title = Mandatory-page linking (site-wide)
batch-report-commerce-not-linked = Not linked anywhere in the audited set
batch-report-commerce-trust-impressum = Imprint
batch-report-commerce-trust-agb = Terms (AGB)
batch-report-commerce-trust-widerruf = Right of withdrawal / returns
batch-report-commerce-trust-versand = Shipping info
batch-report-commerce-trust-zahlungsarten = Payment methods
batch-report-commerce-trust-kontakt = Contact

## Cover
batch-report-contents = Contents

## EN 301 549 clause roll-up
batch-report-en301549-title = EN 301 549 clause mapping
batch-report-en301549-intro = Domain-wide roll-up of this batch's automated WCAG 2.2 A/AA findings onto EN 301 549 V4.1.1 (2026-09), chapter 9 (Web) clause numbers — the worst status observed across all audited pages, plus how many pages have a confirmed violation.
batch-report-en301549-disclaimer-title = Not an accessibility statement
batch-report-en301549-status-violations = Violations found
batch-report-en301549-status-no-violations = No violations (automated)
batch-report-en301549-status-manual = Manual review required
batch-report-en301549-incomplete = Automated evaluation incomplete
batch-report-en301549-col-clause = Clause
batch-report-en301549-col-title = Title
batch-report-en301549-col-pages = Pages affected
batch-report-en301549-table-title = Clause roll-up (all 50 WCAG 2.1 A/AA clauses)
batch-report-en301549-out-of-scope-title = Outside this audit's scope
batch-report-en301549-chapter = Chapter { $chapter } – { $title } (not assessed)

## Overview cards
batch-report-grid-accessibility-average = Accessibility average
batch-report-grid-urls-audited = URLs audited
batch-report-grid-wcag-occurrences = WCAG occurrences
batch-report-grid-critical-high = Critical + high WCAG occurrences
