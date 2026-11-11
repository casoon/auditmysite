# Texts of the page-health issues (`seo::page_health::collect_issues`, #936):
# one message per issue type. Counts reach the messages as numbers, so the
# plural agreement lives here; the thresholds and severities stay in Rust.

page-health-missing-doctype = Missing HTML5 doctype declaration — browser renders in quirks mode
page-health-document-write =
    { $count } inline { $count ->
        [one] script
       *[other] scripts
    } use document.write() — blocks HTML parsing
page-health-excessive-dom = DOM size critical: { $nodes } HTML elements (recommended: <1500, max depth: { $depth })
page-health-large-dom = DOM size elevated: { $nodes } HTML elements (recommended: <800, max depth: { $depth })
page-health-images-without-dimensions = { $count } <img> elements without explicit width/height — CLS risk
page-health-paste-blocked-password =
    { $count } password { $count ->
        [one] field
       *[other] fields
    } block pasting (onpaste handler) — interferes with password managers
page-health-offscreen-images-without-lazy = { $count } images below the viewport without loading="lazy" — delay initial page render
page-health-images-without-srcset = { $count } <img> elements without srcset — no responsive image variants for different resolutions
page-health-missing-preconnect = { $count } external origins without <link rel="preconnect">{ $sample }
page-health-non-crawlable-links = { $count } links not crawlable (javascript:, empty, no href) — loss of PageRank
page-health-images-without-modern-format = { $count } images as JPEG/PNG without a WebP/AVIF alternative — increased load time
page-health-oversized-images = { $count } images are displayed significantly smaller than their natural resolution
page-health-gif-images =
    { $count } GIF { $count ->
        [one] image
       *[other] images
    } found — consider replacing with MP4/WebM (80–95% smaller)
page-health-font-display-missing = { $count } @font-face rules without font-display: swap/fallback/optional — FOIT risk
page-health-fonts-not-preloaded =
    { $count } of { $total } web { $total ->
        [one] font
       *[other] fonts
    } not preloaded via <link rel="preload" as="font"> — delays first text render
page-health-orphaned-preload = { $count } <link rel="preload"> hints point to resources that were never loaded (orphaned)
page-health-lcp-image-lazy-loaded = LCP image candidate has loading="lazy" — delays the Largest Contentful Paint
page-health-lcp-image-without-preload = Largest visible image{ $url } has no <link rel="preload" as="image"> hint
page-health-lcp-image-without-fetchpriority = LCP image candidate is missing fetchpriority="high" — load priority not signaled
page-health-deprecated-apis =
    { $count } deprecated browser { $count ->
        [one] API
       *[other] APIs
    } detected in inline scripts
page-health-http1-only = Page is served over HTTP/1.1 — HTTP/2 enables multiplexing and header compression
page-health-missing-compression = Page content is transferred without compression (no gzip/brotli)
page-health-inefficient-cache = Cache-Control without an effective caching lifetime: { $value }
page-health-inefficient-resource-cache =
    { $inefficient } of { $total } static { $total ->
        [one] resource
       *[other] resources
    } without an efficient cache policy{ $sample }
page-health-incorrect-content-type =
    { $total } static { $total ->
        [one] resource
       *[other] resources
    } with a missing or incorrect Content-Type header{ $sample }
page-health-charset-mismatch = Charset mismatch: HTTP header declares "{ $header }", <meta charset> declares "{ $declared }"
page-health-hreflang-invalid = { $count } hreflang entries with an invalid language code
page-health-jsonld-invalid = { $count } JSON-LD blocks without @context or @type
page-health-sync-head-scripts =
    { $count } { $count ->
        [one] script
       *[other] scripts
    } in <head> without defer/async/module are syntactically render-blocking; whether they delay measurably is shown by the render-blocking analysis.
page-health-missing-sri =
    { $count } external { $count ->
        [one] resource
       *[other] resources
    } without Subresource Integrity (integrity attribute missing)
page-health-broken-fragment-links =
    { $count } anchor { $count ->
        [one] link
       *[other] links
    } point to non-existent IDs{ $sample }
page-health-generic-link-text =
    { $count } { $count ->
        [one] link
       *[other] links
    } with non-descriptive text ("here", "more", "click here" and similar)
page-health-soft-404 = Server returns HTTP { $status } for non-existent URLs (soft 404)
page-health-custom-404-invalid-status = Non-existent URL returns HTTP { $status } instead of 404/410
page-health-generic-404-page = 404 page looks generic or very sparse; a helpful custom 404 page improves orientation and crawling signals
page-health-meta-refresh-immediate = Immediate redirect via meta-refresh (harmful for SEO, use a 301 redirect)
page-health-meta-refresh-delayed = meta-refresh with { $delay }s delay found
page-health-frames = { $count } deprecated <frame> elements found
page-health-url-too-long = URL with { $length } characters exceeds the recommendation (>115)
page-health-url-too-deep = URL path depth { $depth } exceeds the recommendation (>5 levels)
page-health-dynamic-url = URL contains query parameters (dynamic URL)
page-health-multiple-redirects = { $count } HTTP redirects before the final page — each hop costs ~100–300 ms
page-health-redirect = Page redirects to: { $url }
page-health-www-not-consolidated = www and non-www versions are not consolidated (no 301 redirect)
page-health-trailing-slash-inconsistent = URL is reachable with and without a trailing slash as two separate 200 responses — duplicate-content risk without a canonical redirect
page-health-http-not-redirected-to-https = http:// version does not redirect to https:// (status: { $status })

# Parts the messages above take as arguments. The samples start with a space
# (`{" "}`) because they are appended to a sentence.
page-health-sample-examples = {" "}(e.g. { $list })
page-health-sample-cache-resource = {" "}(e.g. { $url }: { $reason })
page-health-sample-content-type = {" "}(e.g. { $url }: expected { $expected }, got { $actual })
page-health-none = none
page-health-unreachable = unreachable
page-health-unknown-url = (unknown)

# Why a static resource's cache policy is inefficient (`cache_reason_text`).
page-health-cache-reason-missing = missing Cache-Control
page-health-cache-reason-disabled = explicitly disables caching
page-health-cache-reason-short = short or missing max-age/s-maxage
