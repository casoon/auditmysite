# Titles of the findings outside the WCAG taxonomy (`audit::finding_titles`,
# #1039): one message per rule id `<source>.<check>`. The key is
# `finding-title-` plus the rule id with every character other than a letter or
# digit written as `-`. The English title is the finding's `title` in the JSON;
# the PDF and the table show the title in the run language. Checks whose title
# is page content or open-ended (budget metrics, library names, heading issues)
# have no entry here and keep their stored title.

## Best practices
finding-title-best-practices-console-error = Console error

## SEO: meta tags
finding-title-seo-meta-title = Page title
finding-title-seo-meta-description = Meta description
finding-title-seo-meta-viewport = Viewport meta tag
finding-title-seo-meta-lang = Page language
finding-title-seo-meta-canonical = Canonical URL
finding-title-seo-meta-twitter-card = Twitter card type
finding-title-seo-meta-og-image = Open Graph image

## SEO: technical checks
finding-title-seo-technical-broken-internal-links = Broken internal links
finding-title-seo-technical-canonical-domain-mismatch = Canonical points to another domain
finding-title-seo-technical-form-insecure-transport = Form over an insecure connection
finding-title-seo-technical-form-password-autocomplete = Password field without autocomplete hint
finding-title-seo-technical-form-sensitive-get = Sensitive data sent via GET
finding-title-seo-technical-hreflang-no-self-reference = hreflang without self-reference
finding-title-seo-technical-hreflang-no-x-default = hreflang without x-default
finding-title-seo-technical-internal-links-with-query-params = Internal links with parameters
finding-title-seo-technical-mixed-content = Mixed content (HTTP on an HTTPS page)
finding-title-seo-technical-no-canonical = Canonical URL missing
finding-title-seo-technical-no-https = No HTTPS
finding-title-seo-technical-no-internal-links = No internal links
finding-title-seo-technical-no-lang = Language declaration missing
finding-title-seo-technical-noindex = Page set to noindex
finding-title-seo-technical-og-image-missing-alt = og:image without alternative text
finding-title-seo-technical-pagination-duplicate-prev-next = Duplicate pagination links
finding-title-seo-technical-pagination-self-referential-rel-link = Pagination points to itself
finding-title-seo-technical-pwa-invalid-manifest = Invalid web app manifest
finding-title-seo-technical-pwa-missing-manifest = Web app manifest missing
finding-title-seo-technical-remote-google-fonts = Google Fonts from an external server
finding-title-seo-technical-thin-content = Little text content
finding-title-seo-technical-twitter-image-missing-alt = twitter:image without alternative text
finding-title-seo-technical-www-no-redirect = www variant without redirect
finding-title-seo-technical-amp-missing-html-amp-attribute = AMP: html element without amp attribute
finding-title-seo-technical-amp-missing-canonical = AMP: canonical link missing
finding-title-seo-technical-amp-missing-amp-runtime = AMP: runtime script missing
finding-title-seo-technical-amp-missing-amp-boilerplate = AMP: boilerplate CSS missing
finding-title-seo-technical-amp-custom-script-not-allowed = AMP: custom script not allowed

## SEO: structured data
finding-title-seo-schema-jsonld-parse-error = Structured data: invalid JSON-LD
finding-title-seo-schema-jsonld-invalid-root = Structured data: invalid root structure
finding-title-seo-schema-jsonld-missing-context = Structured data: schema.org context missing
finding-title-seo-schema-jsonld-missing-type = Structured data: node without @type
finding-title-seo-schema-jsonld-graph-not-array = Structured data: @graph is not an array
finding-title-seo-schema-jsonld-empty-document = Structured data: empty JSON-LD block
finding-title-seo-schema-rule-missing = { $feature }: required property missing

## SEO: page health
finding-title-seo-page-health-broken-fragment-links = Broken fragment links
finding-title-seo-page-health-charset-mismatch = Contradicting character set
finding-title-seo-page-health-custom-404-invalid-status = 404 page with wrong status code
finding-title-seo-page-health-deprecated-apis = Deprecated browser APIs
finding-title-seo-page-health-document-write = document.write() in use
finding-title-seo-page-health-dynamic-url = Dynamic URL
finding-title-seo-page-health-excessive-dom = DOM too large
finding-title-seo-page-health-font-display-missing = font-display missing
finding-title-seo-page-health-fonts-not-preloaded = Fonts not preloaded
finding-title-seo-page-health-frames = Frames in use
finding-title-seo-page-health-generic-404-page = Generic 404 page
finding-title-seo-page-health-generic-link-text = Generic link texts
finding-title-seo-page-health-gif-images = Animated GIF images
finding-title-seo-page-health-hreflang-invalid = Invalid hreflang annotations
finding-title-seo-page-health-http1-only = HTTP/1.1 only
finding-title-seo-page-health-http-not-redirected-to-https = HTTP not redirected to HTTPS
finding-title-seo-page-health-images-without-dimensions = Images without dimensions
finding-title-seo-page-health-images-without-modern-format = Images without a modern format
finding-title-seo-page-health-images-without-srcset = Images without srcset
finding-title-seo-page-health-incorrect-content-type = Incorrect Content-Type
finding-title-seo-page-health-inefficient-cache = HTML document without effective caching
finding-title-seo-page-health-inefficient-resource-cache = Static resources without efficient caching
finding-title-seo-page-health-jsonld-invalid = Invalid JSON-LD
finding-title-seo-page-health-large-dom = Large DOM
finding-title-seo-page-health-lcp-image-lazy-loaded = LCP image is lazy-loaded
finding-title-seo-page-health-lcp-image-without-fetchpriority = LCP image without fetchpriority
finding-title-seo-page-health-lcp-image-without-preload = LCP image without preload
finding-title-seo-page-health-meta-refresh = Meta refresh redirect
finding-title-seo-page-health-missing-compression = Compression missing
finding-title-seo-page-health-missing-doctype = Doctype missing
finding-title-seo-page-health-missing-preconnect = Preconnect missing
finding-title-seo-page-health-missing-sri = Subresource Integrity missing
finding-title-seo-page-health-multiple-redirects = Multiple redirects
finding-title-seo-page-health-non-crawlable-links = Non-crawlable links
finding-title-seo-page-health-offscreen-images-without-lazy = Offscreen images without lazy loading
finding-title-seo-page-health-orphaned-preload = Preload without use
finding-title-seo-page-health-oversized-images = Oversized images
finding-title-seo-page-health-paste-blocked-password = Paste blocked in password field
finding-title-seo-page-health-redirect = Redirect
finding-title-seo-page-health-soft-404 = Soft 404
finding-title-seo-page-health-sync-head-scripts = Blocking scripts in the head
finding-title-seo-page-health-trailing-slash-inconsistent = Inconsistent trailing slash
finding-title-seo-page-health-url-too-deep = URL nested too deeply
finding-title-seo-page-health-url-too-long = URL too long
finding-title-seo-page-health-www-not-consolidated = www variant not consolidated

## Security headers
finding-title-security-headers-missing-https = HTTPS missing
finding-title-security-headers-missing-header = Security header missing
finding-title-security-headers-cors-wildcard-credentials = CORS: wildcard with credentials
finding-title-security-headers-hsts-preload-ineligible = HSTS preload requirements not met
finding-title-security-headers-permissions-policy-permissive = Permissions-Policy too permissive
finding-title-security-headers-public-source-map = Public source maps
finding-title-security-headers-unsafe-inline-script = CSP allows inline scripts
finding-title-security-headers-unsafe-eval-script = CSP allows eval()
finding-title-security-headers-wildcard-script-source = CSP allows wildcard script sources
finding-title-security-headers-unsafe-inline-style = CSP allows inline styles
finding-title-security-headers-wildcard-source = CSP allows wildcard sources
finding-title-security-csp-missing-directive = CSP: { $directive } directive missing

## Mobile
finding-title-mobile-missing-viewport = Viewport missing
finding-title-mobile-improper-viewport = Viewport misconfigured
finding-title-mobile-not-scalable = Zoom disabled
finding-title-mobile-small-targets = Touch targets too small
finding-title-mobile-small-fonts = Font too small
finding-title-mobile-horizontal-scroll = Horizontal scrolling

## HTML conformance: one title per rule family (`html_conform::recommendation`);
## a Schematron assertion also names its rule id, so two rules of one family
## never share a title.
finding-title-html-conform-parser = HTML syntax error
finding-title-html-conform-schema = Element or attribute not allowed here
finding-title-html-conform-ids = Duplicate ID
finding-title-html-conform-headings = Heading structure violates the standard
finding-title-html-conform-language = Page language declaration invalid
finding-title-html-conform-image-alt = Image without text alternative
finding-title-html-conform-responsive-images = Invalid srcset, sizes or source markup
finding-title-html-conform-image-maps = Invalid image map
finding-title-html-conform-nested-interactive = Nested interactive elements
finding-title-html-conform-forms = Invalid form markup
finding-title-html-conform-document-meta = Invalid metadata in the document head
finding-title-html-conform-link-rel = Invalid link element
finding-title-html-conform-scripts = Invalid script attributes or script type
finding-title-html-conform-csp = Content-Security-Policy meta element violated
finding-title-html-conform-tables = Irregular table structure
finding-title-html-conform-aria-redundant-role = Redundant ARIA role
finding-title-html-conform-aria-naming = ARIA name where none is allowed
finding-title-html-conform-aria-state = Disallowed ARIA attribute or state
finding-title-html-conform-aria-structure = ARIA role outside its required context
finding-title-html-conform-obsolete = Obsolete HTML element
finding-title-html-conform-microdata = Incomplete microdata attributes
finding-title-html-conform-meter-progress = Invalid meter or progress value range
finding-title-html-conform-sectioning = Sectioning elements nested incorrectly
finding-title-html-conform-media-text = Media or text element used incorrectly
finding-title-html-conform-text-normalization = Text not in Unicode Normalization Form C
finding-title-html-conform-generic = HTML conformance violation
finding-title-html-conform-rule = { $family } ({ $rule })

## Screen reader reading sequence: one title per WCAG criterion
finding-title-screen-reader-wcag-1-3-1 = Screen reader: structure not conveyed
finding-title-screen-reader-wcag-1-3-6 = Screen reader: page regions not identifiable
finding-title-screen-reader-wcag-2-4-1 = Screen reader: no skip link
finding-title-screen-reader-wcag-2-4-4 = Screen reader: unclear link or button names
finding-title-screen-reader-wcag-3-3-2 = Screen reader: form fields without label
finding-title-screen-reader-wcag-4-1-2 = Screen reader: controls without name
finding-title-screen-reader-reading-sequence = Screen reader reading sequence

## UX
finding-title-ux-no-cta = No recognizable call to action
finding-title-ux-competing-ctas = Competing calls to action
finding-title-ux-generic-links = Generic link texts
finding-title-ux-no-h1 = H1 missing
finding-title-ux-multiple-h1 = Multiple H1
finding-title-ux-heading-skips = Skipped heading levels
finding-title-ux-large-dom = Large DOM
finding-title-ux-little-text = Little text
finding-title-ux-no-subheadings = No subheadings
finding-title-ux-no-contact = No contact option recognizable
finding-title-ux-no-imprint = Imprint not recognizable
finding-title-ux-no-privacy = Privacy policy not recognizable
finding-title-ux-weak-trust-signals = Weak trust signals
finding-title-ux-too-many-links = Too many links
finding-title-ux-too-many-interactive = Too many interactive elements

## Journey
finding-title-journey-no-h1 = H1 missing
finding-title-journey-multiple-h1 = Multiple H1
finding-title-journey-vague-h1 = H1 not meaningful
finding-title-journey-no-page-title = Page title missing
finding-title-journey-little-early-text = Little text at the top
finding-title-journey-navigation-missing = Navigation missing
finding-title-journey-main-hidden-from-ax = Main region hidden from assistive technology
finding-title-journey-main-missing = Main region missing
finding-title-journey-few-subheadings = Few subheadings
finding-title-journey-no-breadcrumb = No breadcrumb trail
finding-title-journey-no-footer = No footer
finding-title-journey-generic-links = Generic link texts
finding-title-journey-empty-links = Links without text
finding-title-journey-duplicate-links = Duplicate link texts
finding-title-journey-too-many-links = Too many links
finding-title-journey-unnamed-buttons = Buttons without name
finding-title-journey-unlabeled-forms = Form fields without label
finding-title-journey-generic-buttons = Generic button texts
finding-title-journey-no-call-to-action = No call to action
finding-title-journey-competing-ctas = Competing calls to action
finding-title-journey-blocking-dialog = Blocking dialog
finding-title-journey-complex-form = Complex form

## Commerce
finding-title-commerce-missing-price = Price missing
finding-title-commerce-missing-availability = Availability missing
finding-title-commerce-missing-shipping-details = Shipping details missing
finding-title-commerce-missing-return-policy = Return policy missing
finding-title-commerce-missing-reviews = Reviews missing
finding-title-commerce-missing-impressum-link = Imprint link missing
finding-title-commerce-missing-agb-link = Terms and conditions link missing
finding-title-commerce-missing-widerruf-link = Right of withdrawal link missing
finding-title-commerce-missing-shipping-page-link = Shipping link missing
finding-title-commerce-missing-payment-link = Payment link missing
finding-title-commerce-missing-contact-link = Contact link missing

## Privacy
finding-title-privacy-tracking-request-before-consent = Tracking request before consent
finding-title-privacy-non-essential-storage-before-consent = Non-essential storage before consent
finding-title-privacy-visitor-ip-transfer = Visitor IP transfer
finding-title-privacy-third-country-provider = Provider based in a third country
finding-title-privacy-no-consent-platform-detected = No consent platform detected
finding-title-privacy-policy-link-missing = Link to privacy information missing
finding-title-privacy-policy-link-unusable = Link to privacy information unusable

## Accessibility journey
finding-title-a11y-journey-hidden-focusable-aria-hidden = Focusable despite aria-hidden
finding-title-a11y-journey-hidden-focusable-inert = Focusable despite inert
finding-title-a11y-journey-hidden-focusable-style = Focusable despite being hidden
finding-title-a11y-journey-focus-indicator-not-detected = Focus indicator not detected
finding-title-a11y-journey-tab-order-backward-jumps = Tab order jumps backwards
finding-title-a11y-journey-focus-trap-not-entered = Focus does not enter the dialog
finding-title-a11y-journey-focus-trap-background-not-hidden = Background not hidden while the dialog is open
finding-title-a11y-journey-focus-trap-escaped = Focus leaves the dialog
finding-title-a11y-journey-focus-trap-escape-not-closing = Escape does not close the dialog
finding-title-a11y-journey-focus-restoration-lost-to-body = Focus lost after the dialog
finding-title-a11y-journey-modal-not-opened = Dialog does not open
finding-title-a11y-journey-menu-not-opened = Menu does not open
finding-title-a11y-journey-menu-focus-not-moved = Focus does not enter the menu
finding-title-a11y-journey-menu-escape-not-closing = Escape does not close the menu
finding-title-a11y-journey-tabs-selection-not-moved = Tab selection does not follow focus
finding-title-a11y-journey-tabs-focus-not-on-tab = Focus not on the tab
finding-title-a11y-journey-disclosure-not-opened = Disclosure does not open
finding-title-a11y-journey-disclosure-not-closed = Disclosure does not close
finding-title-a11y-journey-disclosure-state-without-content = Disclosure state without content
finding-title-a11y-journey-disclosure-content-without-state = Disclosure content without state
finding-title-a11y-journey-spa-no-announcement-signal = Page change without announcement
finding-title-a11y-journey-spa-title-unchanged = Page title does not change
finding-title-a11y-journey-spa-focus-not-moved = Focus stays on page change
finding-title-a11y-journey-skip-link-focus-not-moved = Skip link does not move focus
finding-title-a11y-journey-form-error-silent-failure = Form error without feedback
finding-title-a11y-journey-form-error-invalid-without-live-region = Form error without live region
finding-title-a11y-journey-form-error-unlinked-fields = Error message not linked to the field
finding-title-a11y-journey-form-error-focus-not-managed = Focus not managed on form error
finding-title-a11y-journey-form-error-live-region-late-insertion = Live region inserted only after the error
finding-title-a11y-journey-add-to-cart-no-status-announcement = Add-to-cart without status announcement
finding-title-a11y-journey-add-to-cart-no-feedback-detected = Add-to-cart without detectable feedback
finding-title-a11y-journey-quantity-stepper-keyboard-inoperable = Quantity stepper not keyboard operable
finding-title-a11y-journey-quantity-stepper-value-not-exposed = Quantity not exposed to assistive technology
finding-title-a11y-journey-link-text-generic = Generic link texts
finding-title-a11y-journey-link-text-duplicate = Duplicate link texts
finding-title-a11y-journey-heading-missing-h1 = H1 missing
finding-title-a11y-journey-heading-multiple-h1 = Multiple H1
finding-title-a11y-journey-heading-level-skip = Skipped heading level
finding-title-a11y-journey-landmark-missing-main = Main region missing
finding-title-a11y-journey-landmark-nav-without-labels = Navigation regions without label
finding-title-a11y-journey-landmark-duplicate-unique = Duplicate landmarks
finding-title-a11y-journey-media-controls-missing-name = Media controls without name
finding-title-a11y-journey-media-controls-not-reachable = Media controls not reachable
