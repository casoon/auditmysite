# Texts of the SEO issue lists (#936): the technical issues
# (`seo::technical::collect_technical_issues`), the meta-tag findings
# (`seo::meta::meta_issue_text`) and the structured-data issues
# (`seo::schema::schema_issue_text`). One message per issue; counts reach the
# messages as numbers, so the plural agreement lives here. Which issues exist
# and their severity stay in Rust. Also the table labels of the structured-data
# issues (`seo::schema::schema_issue_label`) and of the robots.txt audit (bot
# classes and policy labels of `seo::robots`), the labels and texts of the SEO
# content profile (`seo::profile`), the SERP status labels (`seo::serp`), the
# structured-data fit and content-parity texts (`seo::schema_fit`,
# `seo::schema_parity`) and the rich-result rule wording
# (`seo::schema_rules`).

seo-technical-no-https = Page is not served over HTTPS
seo-technical-no-canonical = Missing canonical URL
seo-technical-no-lang = Missing lang attribute on html element
seo-technical-thin-content = Page has thin content ({ $words } words, recommended: 300+)
seo-technical-no-internal-links = Page has no internal links
seo-technical-remote-google-fonts = Externally hosted Google Fonts detected
seo-technical-noindex = Page has a noindex directive — it is not indexed by search engines
seo-technical-hreflang-no-x-default = hreflang annotations without an x-default entry
seo-technical-hreflang-no-self-reference = hreflang set contains no self-reference for the current page
seo-technical-internal-links-with-query-params =
    { $count } internal { $count ->
        [one] link
       *[other] links
    } with query parameters — can dilute crawl budget
seo-technical-pagination-self-referential-rel-link = Pagination rel link points to the current page instead of an adjacent page
seo-technical-pagination-duplicate-prev-next = Pagination rel="prev" and rel="next" point to the same URL
seo-technical-broken-internal-links =
    { $count } broken internal { $count ->
        [one] link
       *[other] links
    } detected: { $list }
seo-technical-mixed-content =
    { $count } HTTP { $count ->
        [one] subresource
       *[other] subresources
    } referenced from HTTPS page ({ $blockable } blockable): { $samples }
seo-technical-og-image-missing-alt = og:image is set without an og:image:alt description for the preview image
seo-technical-twitter-image-missing-alt = twitter:image is set without a twitter:image:alt description for the preview image
seo-technical-pwa-missing-manifest = Missing web app manifest
seo-technical-pwa-invalid-manifest = Web app manifest is missing required PWA fields
seo-technical-amp-missing-html-amp-attribute = AMP markup detected, but the html element has no amp attribute
seo-technical-amp-missing-canonical = AMP page is missing a canonical link
seo-technical-amp-missing-amp-runtime = AMP page is missing the AMP runtime script
seo-technical-amp-missing-amp-boilerplate = AMP page is missing amp-boilerplate CSS
seo-technical-amp-custom-script-not-allowed = AMP page contains a custom script outside the AMP runtime
seo-technical-form-insecure-transport =
    { $count } sensitive { $count ->
        [one] form
       *[other] forms
    } submit to HTTP or are hosted on HTTP
seo-technical-form-sensitive-get =
    { $count } sensitive { $count ->
        [one] form
       *[other] forms
    } use GET and can leak values into URLs and logs
seo-technical-form-password-autocomplete =
    { $count } password { $count ->
        [one] field
       *[other] fields
    } have autocomplete disabled or misconfigured
seo-technical-www-no-redirect = Both { $primary } and { $alternative } serve content (HTTP 200). Add a 301 redirect from the alternative to the primary to prevent duplicate content.{ $note }
seo-technical-www-note-canonical-ok = {" "}Canonical tag is set correctly, but a 301 is still recommended — canonicals are hints, not directives.
seo-technical-www-note-canonical-other = {" "}Additionally, canonical does not point to the primary domain.
seo-technical-www-note-canonical-none = {" "}No canonical tag found.
seo-technical-canonical-domain-mismatch = Canonical URL points to a different domain variant than the audited URL ({ $primary } vs canonical).

seo-meta-missing-title = Missing page title
seo-meta-title-too-short = Title is too short ({ $length } chars, recommended: 30-60)
seo-meta-title-too-long = Title is too long ({ $length } chars, recommended: 30-60)
seo-meta-missing-description = Missing meta description
seo-meta-description-too-short = Description is too short ({ $length } chars, recommended: 120-160)
seo-meta-description-too-long = Description is too long ({ $length } chars, recommended: 120-160)
seo-meta-missing-viewport = Missing viewport meta tag
seo-meta-missing-lang = Missing lang attribute on <html> element
seo-meta-missing-canonical = Missing canonical URL
seo-meta-invalid-twitter-card = Invalid twitter:card value "{ $value }" (allowed: { $allowed })
seo-meta-relative-og-image = og:image is not an absolute URL: "{ $value }"

seo-schema-jsonld-parse-error = The JSON-LD block contains invalid JSON and cannot be evaluated.
seo-schema-jsonld-invalid-root = The JSON-LD root must be an object or an array of objects.
seo-schema-jsonld-missing-context = No schema.org @context was found; schema terms cannot be interpreted reliably.
seo-schema-jsonld-missing-type = A JSON-LD node has no @type and cannot be assigned to a schema type.
seo-schema-jsonld-graph-not-array = The @graph value must be an array of JSON-LD nodes.
seo-schema-jsonld-empty-document = The JSON-LD block contains no nodes.
seo-schema-recommended-missing = { $type }: recommended property "{ $property }" is missing
seo-schema-required-missing = { $type }: required property "{ $property }" is missing

seo-schema-label-jsonld-parse-error = JSON syntax
seo-schema-label-jsonld-invalid-root = Root structure
seo-schema-label-jsonld-missing-context = Schema context
seo-schema-label-jsonld-missing-type = Schema type
seo-schema-label-jsonld-graph-not-array = @graph
seo-schema-label-jsonld-empty-document = Empty block
seo-schema-label-other = Schema.org

seo-robots-bot-class-wildcard = All crawlers (*)
seo-robots-bot-class-search-engine = Search engine
seo-robots-bot-class-ai-training = AI training
seo-robots-bot-class-ai-citation = AI search / citation
seo-robots-bot-class-ai-mixed = AI training & search
seo-robots-bot-class-unknown-ai = Unknown scraper
seo-robots-bot-class-general = General
seo-robots-bot-class-unknown = Unclassified

seo-robots-policy-everything-blocked = Everything blocked (Disallow: *)
seo-robots-policy-ai-fully-blocked = AI fully blocked
seo-robots-policy-ai-training-blocked-conservative = AI training blocked (conservative)
seo-robots-policy-ai-training-blocked-search-allowed = AI training blocked, AI search allowed
seo-robots-policy-ai-access-open = AI access open
seo-robots-policy-no-explicit-ai-rule = No explicit AI rule

# Labels of the SEO content profile (`seo::profile`): the maturity level with
# its description and the page type. `PageType::NavigationHub` takes the hub
# label of the page intent (labels.ftl).
seo-profile-maturity-basic = Basic
seo-profile-maturity-standard = Standard
seo-profile-maturity-advanced = Advanced
seo-profile-maturity-professional = Professional
seo-profile-maturity-description-basic = Fundamental SEO measures are largely absent.
seo-profile-maturity-description-standard = Basic SEO is in place; advanced techniques are missing.
seo-profile-maturity-description-advanced = Good SEO coverage with structured data and social tags.
seo-profile-maturity-description-professional = Very broad technical SEO coverage; no statement on strategic content quality.
seo-profile-page-type-editorial = Editorial / Article
seo-profile-page-type-structured-content = Structured knowledge content
seo-profile-page-type-marketing-landing = Marketing / Landing Page
seo-profile-page-type-media-heavy = Media-oriented page
seo-profile-page-type-utility = Transactional / Utility
seo-profile-page-type-thin-content = Thin / Minimal Content

# Texts of the SEO content profile (`seo::profile::build_content_profile`):
# the page attributes, the content type, the summary without meta
# information, the entry count of a generic schema field and the names,
# check labels and details of the signal categories. Which attribute, check
# and detail applies stays in Rust; language-neutral names (Meta-Tags, Social
# Media, HTTPS, Canonical URL, Website) are written there.
seo-profile-attribute-text-rich = text-rich
seo-profile-attribute-very-short = very short
seo-profile-attribute-structured = structured
seo-profile-attribute-navigation-heavy = navigation-heavy
seo-profile-attribute-visually-oriented = visually oriented
seo-profile-attribute-knowledge-oriented = knowledge-oriented
seo-profile-attribute-conversion-oriented = conversion-oriented
seo-profile-attribute-purpose-bound = purpose-bound
seo-profile-attribute-thin = thin
seo-profile-content-type-article = Article
seo-profile-content-type-product = Product
seo-profile-content-type-profile = Profile
seo-profile-content-type-article-blog = Article / Blog
seo-profile-content-type-ecommerce-product = E-Commerce / Product
seo-profile-content-type-local-business = Local business
seo-profile-content-type-event = Event
seo-profile-content-type-recipe = Recipe
seo-profile-content-type-faq = FAQ / Information page
seo-profile-summary-none = No meta information found.
seo-profile-generic-array-entries = [{ $count ->
        [one] { $count } entry
       *[other] { $count } entries
    }]
seo-profile-detail-characters = { $count } characters
seo-profile-detail-missing = missing
seo-profile-detail-present = present
seo-profile-detail-active = active
seo-profile-check-title = Title present & length optimal
seo-profile-check-description = Description present & length optimal
seo-profile-check-viewport = Viewport configured
seo-profile-check-charset = Charset defined
seo-profile-check-lang-set = Language (lang) set
seo-profile-category-headings = Headings
seo-profile-check-single-h1 = Exactly one H1
seo-profile-detail-h1-count = { $count } H1 tags found
seo-profile-check-h1-text = H1 text present
seo-profile-detail-no-h1-text = no H1 text
seo-profile-check-no-skipped-levels = No skipped levels
seo-profile-detail-no-gaps = no gaps
seo-profile-detail-levels-skipped = levels skipped
seo-profile-check-logical-hierarchy = Logical hierarchy
seo-profile-detail-no-issues = no issues
seo-profile-detail-order-problems =
    { $count } { $count ->
        [one] issue
       *[other] issues
    }
seo-profile-detail-order-problems-first-heading =
    { $count } { $count ->
        [one] issue
       *[other] issues
    }, first heading is H{ $level }
seo-profile-check-og-present = OpenGraph tags present
seo-profile-check-og-complete = OpenGraph ≥ 80% complete
seo-profile-check-twitter-present = Twitter Card present
seo-profile-check-twitter-complete = Twitter Card ≥ 75% complete
seo-profile-category-technical = Technical
seo-profile-check-lang = Language (lang)
seo-profile-check-hreflang = Hreflang (multilingual)
seo-profile-detail-hreflang-languages = { $count ->
        [one] { $count } language
       *[other] { $count } languages
    }
seo-profile-detail-no-hreflang = no hreflang tags (not needed for a single-language page)
seo-profile-check-robots-indexing = Robots allows indexing
seo-profile-detail-no-robots-meta = no robots meta tag (default: index)
seo-profile-category-structured-data = Structured data
seo-profile-check-structured-data = Structured data present
seo-profile-detail-no-structured-data = no structured data
seo-profile-detail-jsonld-not-evaluable = JSON-LD detected but not evaluable
seo-profile-detail-microdata-only = structured data detected (microdata/RDFa, no JSON-LD)
seo-profile-check-rich-result = Rich-result type detected
seo-profile-detail-rich-result-types = { $types } (type detection only; eligibility also requires complete, matching content)
seo-profile-detail-no-rich-result-types = no rich-result types detected
seo-profile-check-local-business-nap = LocalBusiness NAP complete
seo-profile-detail-nap-missing = Missing: { $list }
seo-profile-check-local-business-geo = LocalBusiness geo coordinates
seo-profile-detail-geo-missing = geo.latitude/longitude missing
seo-profile-check-local-business-same-as = LocalBusiness sameAs (authority links)
seo-profile-detail-same-as-missing = sameAs missing — prevents a knowledge panel
seo-profile-detail-rating-missing = aggregateRating missing — no stars in the SERP
seo-profile-check-local-business-consistent = LocalBusiness schemas consistent
seo-profile-detail-local-business-consistent = { $count } schemas, name consistent
seo-profile-detail-local-business-conflicting = { $count } schemas with conflicting name
seo-profile-check-jsonld-structure = JSON-LD syntax and structure
seo-profile-detail-missing-required = Missing required: { $list }
seo-profile-detail-missing-recommended = Recommended properties not present: { $list }
seo-profile-category-content-base = Technical content base
seo-profile-check-text-volume = Readable text volume (≥ 300 words, guideline)
seo-profile-detail-words = { $count } words
seo-profile-check-internal-linking = Internal linking (≥ 3 links)
seo-profile-detail-internal-links = { $count } internal links
seo-profile-check-link-density = Link density > 0.5%

# Status labels of the SERP signals (`seo::serp::SerpSignalStatus::label`).
seo-serp-status-ok = OK
seo-serp-status-warning = Warning
seo-serp-status-fail = Error

# Texts of the SERP signals (`seo::serp::build_serp_analysis`): the
# category, label and detail of each signal. Which signal and detail applies
# stays in Rust; language-neutral names (Description, Canonical, Rich Result,
# robots noindex, robots nosnippet, Favicon, hreflang x-default) are written
# there.
seo-serp-category-indexing = Indexing
seo-serp-category-title = Title
seo-serp-category-appearance = Appearance
seo-serp-category-internationalisation = Internationalisation
seo-serp-detail-noindex = Page is marked as noindex and is not indexed ({ $robots })
seo-serp-detail-nosnippet = nosnippet prevents the meta description from appearing in the SERP entry.
seo-serp-label-title-present = Title present
seo-serp-detail-no-title = No <title> tag found — required field for Google listings.
seo-serp-label-title-length = Title length
seo-serp-detail-title-length = { $count } characters (recommended 30–60)
seo-serp-label-truncation-risk = Truncation risk
seo-serp-detail-title-truncated = { $count } characters — title gets truncated in narrow SERPs (about 55 characters is safe).
seo-serp-label-description-present = Description present
seo-serp-detail-no-description = No meta description — Google shows uncontrolled page text.
seo-serp-label-description-length = Description length
seo-serp-detail-description-length = { $count } characters (recommended 120–160)
seo-serp-label-canonical-tag = Canonical tag
seo-serp-detail-no-canonical = No canonical tag — Google picks the canonical URL itself.
seo-serp-label-canonical-self-reference = Canonical self-reference
seo-serp-detail-canonical-other-url = Canonical points to a different URL: { $url }
seo-serp-detail-favicon-present = Favicon present (appears in mobile SERPs and browser tabs).
seo-serp-detail-no-favicon = No favicon found — Google shows a generic icon in mobile SERPs.
seo-serp-detail-hreflang-x-default = { $count } hreflang tags incl. x-default
seo-serp-detail-hreflang-no-x-default = { $count } hreflang tags, no x-default — Google recommendation for multilingual pages.
seo-serp-label-breadcrumb-schema = Breadcrumb schema
seo-serp-detail-breadcrumb-complete = BreadcrumbList detected, required properties met — eligible for a path display.
seo-serp-detail-breadcrumb-incomplete = BreadcrumbList detected but incomplete ({ $list }) — not eligible for a path display.
seo-serp-detail-no-breadcrumb = No BreadcrumbList schema — path display in the SERP entry not possible.

# Labels of the structured-data fit (`seo::schema_fit`): the page kind
# (`SchemaPageKind::label`) and the fit status (`SchemaFitAssessment::fit_text`).
seo-schema-fit-page-kind-merchant-product-detail = Purchasable product detail
seo-schema-fit-page-kind-product-collection = Product collection
seo-schema-fit-page-kind-service-detail = Service detail
seo-schema-fit-page-kind-software-detail = Software / SaaS
seo-schema-fit-page-kind-job-detail = Job detail
seo-schema-fit-page-kind-job-listing = Job listing
seo-schema-fit-page-kind-event-detail = Event detail
seo-schema-fit-page-kind-event-listing = Event listing
seo-schema-fit-page-kind-faq-page = FAQ
seo-schema-fit-page-kind-person-profile = Person profile
seo-schema-fit-page-kind-location-detail = Location detail
seo-schema-fit-page-kind-editorial-product-review = Editorial product review
seo-schema-fit-page-kind-editorial-content = Editorial content
seo-schema-fit-page-kind-marketing-landing-page = Marketing / Landing Page
seo-schema-fit-page-kind-corporate-page = Corporate page
seo-schema-fit-page-kind-hub-page = Hub / Portal
seo-schema-fit-page-kind-lead-generation-page = Lead-generation page
seo-schema-fit-page-kind-indeterminate = Not determined
seo-schema-fit-status-matched = Primary schema matches the detected page type
seo-schema-fit-status-plausible = Schema is plausible; page type is not specific enough for a strict requirement
seo-schema-fit-status-mismatch = Primary schema does not match the strongly indicated page type
seo-schema-fit-status-indeterminate = Page-to-schema fit requires manual review

# Content parity of structured data (`seo::schema_parity`): the status label
# (`ContentParityAssessment::status_text`) and the evidence
# (`ContentParityAssessment::evidence_text`); a missing value is "-" (Rust).
seo-schema-parity-status-match = Matches visible content
seo-schema-parity-status-mismatch = Mismatch
seo-schema-parity-status-not-evaluated = Not evaluated
seo-schema-parity-evidence-match = Schema and visible content agree: { $value }
seo-schema-parity-evidence-mismatch = Schema: { $schema }; visible: { $visible }
seo-schema-parity-evidence-not-evaluated = No single unambiguous visible value was found

# Wording of the rich-result rules (`seo::schema_rules`): the feature name
# that differs by language (`feature_label`; the other feature names are
# language-neutral and written in Rust), the requirement status
# (`status_text`) and the manual-review points (`manual_review_text`).
seo-schema-rules-feature-breadcrumb = Breadcrumb
seo-schema-rules-status-meets-required = Required properties met
seo-schema-rules-status-missing-required = Required properties missing
seo-schema-rules-status-recommendations-only = No required properties
seo-schema-rules-status-not-evaluated = Context not established
seo-schema-rules-review-merchant-listing-context = Confirm that this page is a purchasable product detail page before applying merchant-listing requirements.
seo-schema-rules-review-cancelled-event-keeps-details = Confirm that the cancelled event retains its original startDate and location.
seo-schema-rules-review-event-is-single-public-event = Confirm that the event is a single bookable public event and all marked-up details are visible on the page.
seo-schema-rules-review-recipe-cook-and-prep-time = Google recommends using cookTime and prepTime together; verify the missing duration.
seo-schema-rules-review-recipe-matches-visible-recipe = Confirm that the page describes preparation of one dish and that ingredients and instructions match the visible recipe.
seo-schema-rules-review-job-posting-is-open-position = Confirm that this is one currently open position with a visible application path; expired jobs must be removed or marked with a past validThrough date.
seo-schema-rules-review-software-rating-is-genuine = Confirm that the rating or review is visible, genuine, and specifically about this application.
seo-schema-rules-review-profile-page-subject = Confirm that the page is primarily about one person or organization affiliated with the site.
seo-schema-rules-review-item-list-canonical-urls = Confirm that each listed URL is unique and points to a canonical detail page represented by the visible list.
