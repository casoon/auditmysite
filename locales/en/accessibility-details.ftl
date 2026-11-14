# Texts of the accessibility detail sections of the single-page report
# (`output::pdf::detail_modules::accessibility`, `a11y-details-*`), #936:
# the keyboard accessibility journey, the screen-reader reading order and the
# BFSG callout. Which message applies, the deduplication and the score format
# stay in Rust. The severity labels of the screen-reader findings are the
# `severity-*` messages of report.ftl.

## Keyboard accessibility journey
a11y-details-journey-title = Keyboard Accessibility Journey
a11y-details-journey-overview-critical = { $critical } critical and { $high } high issues detected during keyboard and state-transition tests.
a11y-details-journey-overview-high = { $count ->
        [one] { $count } high-severity interactive issue found.
       *[other] { $count } high-severity interactive issues found.
    }
a11y-details-journey-overview-minor = { $count } minor interactive { $count ->
        [one] issue
       *[other] issues
    } — no critical or high barriers detected.
a11y-details-journey-more = { $count ->
        [one] { $count } additional interactive finding in the JSON report.
       *[other] { $count } additional interactive findings in the JSON report.
    }
a11y-details-journey-sequences = Tested Journey Sequences
a11y-details-journey-steps = { $count ->
        [one] { $count } step
       *[other] { $count } steps
    }
a11y-details-journey-disclaimer = These tests check whether browser, DOM, focus, and accessibility tree provide a robust foundation for screen reader use. They do not simulate the exact output of NVDA, JAWS, or VoiceOver.

## Journey labels (internal journey ids)
a11y-details-journey-label-tab-walk = Tab walk
a11y-details-journey-label-spa-navigation = SPA navigation
a11y-details-journey-label-skip-link = Skip link
a11y-details-journey-label-disclosure = Disclosure menu
a11y-details-journey-label-menu = Menu
a11y-details-journey-label-modal = Modal dialog
a11y-details-journey-label-tabs = Tab panel
a11y-details-journey-label-form-error = Form error
a11y-details-journey-label-add-to-cart = Add to cart
a11y-details-journey-label-quantity-stepper = Quantity stepper

## Screen reader reading order
a11y-details-sr-title = Screen Reader Reading Order
a11y-details-sr-intro = How the page is announced to screen reader users: reading order, landmark and heading quality, and accessible names. These are structural checks on the accessibility tree, not a simulation of NVDA, JAWS or VoiceOver.
a11y-details-sr-heading-quality = Heading quality
a11y-details-sr-landmark-quality = Landmark quality
a11y-details-sr-name-quality = Name quality
a11y-details-sr-announced-nodes = Announced nodes
a11y-details-sr-tab-stops = Tab stops
a11y-details-sr-score-note = The three quality scores use a 0–100 scale (higher is better) and summarize the detected heading hierarchy, landmark structure, and accessible names. Announced nodes and tab stops are simple counts, not quality scores.
a11y-details-sr-findings-title = Screen reader findings

## BFSG callout (never a conformance verdict)
a11y-details-bfsg-no-violations = BFSG-relevant technical criteria: no violations detected within the automated scope. This is neither a conformance statement nor a legal assessment; a test with a real screen reader is still open.
a11y-details-bfsg-incomplete = BFSG-relevant technical criteria: the accessibility tree looks incomplete (consent wall suspected), so the absence of violations says nothing. Repeat the audit on the full page.
a11y-details-bfsg-violations = BFSG-relevant technical criteria: violations detected within the automated scope (see issues below). This is not a legal conformance assessment.
a11y-details-bfsg-needs-review = BFSG-relevant technical criteria: no confirmed violation within the automated scope, but findings that need a manual check (see issues below). This is not a legal conformance assessment.
a11y-details-bfsg-not-evaluated = BFSG-relevant technical criteria were not evaluated for this page.
