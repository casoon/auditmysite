# Texts of the batch PDF's keyboard journey summary, module name, cover contents
# and EN 301 549 clause roll-up (`output::pdf::batch_report::interactive`, `portfolio`,
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
batch-report-en301549-table-title = Clause roll-up (all { $count } WCAG 2.2 A/AA clauses)
batch-report-en301549-out-of-scope-title = Outside this audit's scope
batch-report-en301549-chapter = Chapter { $chapter } – { $title } (not assessed)
