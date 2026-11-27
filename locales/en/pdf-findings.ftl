# Texts of the PDF finding cards (`output::pdf::findings`,
# `pdf-findings-*`) and of the shared module detail helpers
# (`output::pdf::detail_modules`, `pdf-details-*`), #936: the severity
# prefix, the code labels and row keys of a finding card, the occurrence
# evidence rows, the structural-cause labels, the customer-perspective
# titles, the module customer context, the derived-from and accessibility
# penalty notes, the score card caption, the Web Vitals rating and the
# journey category labels. Which message applies, the bracketless parts of
# the card header and the decimal notation of the contrast ratio stay in
# Rust. The code labels High, Medium and Low are `severity-high`,
# `severity-medium` and `severity-low` of report.ftl; the journey
# categories SPA navigation and Quantity stepper are
# `a11y-details-journey-label-spa-navigation` and
# `a11y-details-journey-label-quantity-stepper` of
# accessibility-details.ftl.

## Finding card: severity prefix
pdf-findings-severity-critical = [Critical]
pdf-findings-severity-high = [High]
pdf-findings-severity-medium = [Medium]
pdf-findings-severity-low = [Low]

## Finding card: code labels
pdf-findings-code-very-high = Very high
pdf-findings-code-very-low = Very low
pdf-findings-code-immediate = Immediate
pdf-findings-code-quick-win = Quick win
pdf-findings-code-normal = Normal
pdf-findings-code-manual-review-recommended = Manual review recommended
pdf-findings-code-automatically-confirmed = Automatically confirmed

## Finding card: metadata and assessment
pdf-findings-implementation-priority = Implementation priority
pdf-findings-qa-metadata = QA metadata
pdf-findings-detection-confidence = Detection confidence
pdf-findings-false-positive-risk = False-positive risk
pdf-findings-bfsg-eaa-expanded = BFSG/EAA relevance (German Accessibility Strengthening Act / European Accessibility Act)
pdf-findings-bfsg-eaa = BFSG/EAA relevance
pdf-findings-action-assessment = Action assessment
pdf-findings-expected-effect = Expected effect
pdf-findings-complexity-reason = Complexity reason
pdf-findings-verification = Verification
pdf-findings-self-test = Self-test

## Finding card: occurrences
pdf-findings-dom-path = DOM path
pdf-findings-measured-value = Measured value
pdf-findings-contrast = Contrast { $ratio }
pdf-findings-contrast-required = Contrast { $ratio } (required { $required })
pdf-findings-opacity = Opacity
pdf-findings-opacity-value = { $opacity }: the ratio applies to this state (text colour after opacity { $color })
pdf-findings-opacity-pattern = Measured at reduced opacity ({ $values }) in the sampled occurrences: the text colour itself may pass; the ratio applies to the dimmed state, for example an inactive or fading element.
pdf-findings-color-scheme = Color scheme
pdf-findings-color-scheme-dark = Dark (prefers-color-scheme: dark)
pdf-findings-screenshot-mobile = Element on page (mobile viewport)
pdf-findings-screenshot-desktop = Element on page (desktop viewport)
pdf-findings-screenshot-alt-mobile = Screenshot of the flagged element { $selector } on the mobile viewport
pdf-findings-screenshot-alt-desktop = Screenshot of the flagged element { $selector } on the desktop viewport
pdf-findings-examples-of = { $shown } { $shown ->
        [one] example
       *[other] examples
    } of { $total } occurrences
pdf-findings-finding-id = Finding ID
pdf-findings-rule = Rule

## Finding card: origin of the markup (#1109). A label, never a verdict.
pdf-findings-origin = Origin
pdf-findings-origin-cross-origin = Third-party frame (provider { $list }), fix at the provider
pdf-findings-origin-cross-origin-unknown = Third-party frame, fix at the provider of the embedded content
pdf-findings-origin-cross-origin-source = Third-party frame (source { $list }), fix at the provider of the embedded content
pdf-findings-origin-same-site = Frame of the same site, fix in the embedded page
pdf-findings-origin-mixed = Partly in embedded frames, partly in the page itself
pdf-findings-origin-mixed-provider = Partly in embedded frames (provider { $list }), partly in the page itself

## Finding card: customer perspective
pdf-findings-perspective-operability = Operability
pdf-findings-perspective-structure = Orientation and structure
pdf-findings-perspective-perception = Perception and readability
pdf-findings-perspective-discoverability = Discoverability and AI understanding
pdf-findings-perspective-trust = Trust and technical quality
pdf-findings-perspective-loading = Loading and mobile experience
pdf-findings-perspective-customer-impact = Customer impact

## Module details: customer context
pdf-details-weakness-clear = The score indicates a clear weakness in this area.
pdf-details-weakness-visible = The score indicates visible improvement potential in this area.
pdf-details-context-performance = Visitors may experience delays, unstable rendering or unnecessary data transfer before the page feels usable.
pdf-details-context-seo = Search engines and AI systems need clear titles, headings, structured data and enough readable content to understand the page.
pdf-details-context-security = Security headers and HTTPS signals influence visible trust and reduce avoidable browser-side risk in the checked scope. Which headers a page needs depends on the application; an alternative protection counts where the check accepts it (a CSP frame-ancestors directive replaces X-Frame-Options), and Referrer-Policy, Permissions-Policy and the cross-origin policies are reported as depending on architecture or deployment.
pdf-details-context-html-conform = Clean, spec-conformant HTML helps browsers, assistive technologies and search engines parse and render the page reliably.
pdf-details-context-mobile = Mobile visitors depend on readable text, fitting content and controls that are easy to tap on small screens.
pdf-details-context-ux = This indicator estimates whether the page feels understandable, consistent and low-friction for common visitor tasks.
pdf-details-context-journey = This indicator estimates whether visitors can move through the page intent without avoidable friction.
pdf-details-context-source-quality = These signals show how well machines can process the page: publisher and security markup, and whether title, language and structured data agree with each other.
pdf-details-context-ai-visibility = AI visibility indicates whether content can be parsed, chunked and attributed by AI-assisted systems.
pdf-details-context-content-visibility = Content visibility combines discoverability, trust and topical depth signals; it is an indicator, not a guarantee of reach.
pdf-details-context-other = This module describes customer-facing quality beyond pure accessibility findings.

## Module details: notes and captions
pdf-details-score-caption = Overall score · 0–100

## Module details: Web Vitals rating
pdf-details-vital-good = Good
pdf-details-vital-needs-improvement = Needs improvement
pdf-details-vital-poor = Poor

## Module details: journey categories
pdf-details-journey-tab-order = Tab order
pdf-details-journey-focus-trap = Focus trap
pdf-details-journey-state-transition = State transition
pdf-details-journey-focus-restoration = Focus restoration
pdf-details-journey-form-error = Form error announcement
pdf-details-journey-hidden-focusable = Hidden focusable element
pdf-details-journey-skip-link = Skip link
pdf-details-journey-focus-indicator = Focus indicator
pdf-details-journey-menu = Menu navigation
pdf-details-journey-tabs = Tab navigation
pdf-details-journey-link-text = Link Text
pdf-details-journey-landmark = Landmark
pdf-details-journey-media-controls = Media controls
pdf-details-journey-add-to-cart = Add to cart
pdf-details-journey-heading-outline = Heading outline
