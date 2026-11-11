# Texts of the SEO issue lists (#936): the technical issues
# (`seo::technical::collect_technical_issues`), the meta-tag findings
# (`seo::meta::meta_issue_text`) and the structured-data issues
# (`seo::schema::schema_issue_text`). One message per issue; counts reach the
# messages as numbers, so the plural agreement lives here. Which issues exist
# and their severity stay in Rust. Also the table labels of the structured-data
# issues (`seo::schema::schema_issue_label`) and of the robots.txt audit (bot
# classes and policy labels of `seo::robots`), the labels of the SEO content
# profile (`seo::profile`) and the SERP status labels (`seo::serp`).

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

# Status labels of the SERP signals (`seo::serp::SerpSignalStatus::label`).
seo-serp-status-ok = OK
seo-serp-status-warning = Warning
seo-serp-status-fail = Error
