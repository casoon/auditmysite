# Fixes of the findings outside the WCAG taxonomy (`audit::finding_titles::
# finding_fix`, #1039): one message per rule id `<source>.<check>`, keyed like
# `finding-titles.ftl` with the prefix `finding-fix-`. The fix is the
# recommendation of the finding card and the action of the action plan; a
# finding without one has no action to plan.

## SEO: page health
finding-fix-seo-page-health-missing-doctype = Start every HTML document with <!DOCTYPE html> so browsers render it in standards mode.
finding-fix-seo-page-health-document-write = Replace document.write() with DOM methods (createElement, append) or load the script with async or defer.
finding-fix-seo-page-health-excessive-dom = Reduce the number of HTML elements: remove wrapper elements without a function, render long lists in pages or on demand and keep hidden variants (mobile/desktop) out of the same markup.
finding-fix-seo-page-health-large-dom = Reduce the number of HTML elements: remove wrapper elements without a function and render long lists in pages or on demand.
finding-fix-seo-page-health-images-without-dimensions = Give every img element width and height attributes (or a CSS aspect-ratio) so the browser reserves its space before the image loads.
finding-fix-seo-page-health-paste-blocked-password = Remove the onpaste handler from password fields so password managers and pasted passwords work.
finding-fix-seo-page-health-offscreen-images-without-lazy = Add loading="lazy" to images below the first screen; leave it off the images visible on load.
finding-fix-seo-page-health-images-without-srcset = Offer images in several widths with srcset and sizes so small screens load a matching file.
finding-fix-seo-page-health-missing-preconnect = Add <link rel="preconnect"> for the few external origins the page needs early.
finding-fix-seo-page-health-non-crawlable-links = Give every link a real href target; use a button element for actions that do not navigate.
finding-fix-seo-page-health-images-without-modern-format = Deliver photos and graphics as WebP or AVIF, with JPEG or PNG as a fallback in a picture element where needed.
finding-fix-seo-page-health-oversized-images = Scale images to the size they are displayed at and offer larger variants through srcset only where the layout needs them.
finding-fix-seo-page-health-gif-images = Replace animated GIFs with MP4 or WebM video (video element with autoplay, muted, loop and playsinline).
finding-fix-seo-page-health-font-display-missing = Add font-display: swap (or fallback/optional) to every @font-face rule so text shows in a fallback font while the web font loads.
finding-fix-seo-page-health-fonts-not-preloaded = Preload the web fonts used in the first screen with <link rel="preload" as="font" type="font/woff2" crossorigin>.
finding-fix-seo-page-health-orphaned-preload = Remove preload hints for resources the page does not load, or correct their URL to the resource it requests.
finding-fix-seo-page-health-lcp-image-lazy-loaded = Remove loading="lazy" from the largest image visible on load; lazy loading delays exactly that image.
finding-fix-seo-page-health-lcp-image-without-preload = Preload the largest visible image with <link rel="preload" as="image"> so the browser requests it early.
finding-fix-seo-page-health-lcp-image-without-fetchpriority = Add fetchpriority="high" to the largest image visible on load.
finding-fix-seo-page-health-deprecated-apis = Replace the deprecated browser APIs in the inline scripts with their current equivalents before browsers remove them.
finding-fix-seo-page-health-http1-only = Enable HTTP/2 (or HTTP/3) on the web server or CDN; it serves several requests over one connection.
finding-fix-seo-page-health-missing-compression = Enable Brotli or gzip compression for HTML, CSS, JavaScript and other text responses on the server or CDN.
finding-fix-seo-page-health-inefficient-cache = Give the HTML document a Cache-Control lifetime that fits how often it changes (max-age or s-maxage) and revalidate with ETag or Last-Modified instead of switching caching off.
finding-fix-seo-page-health-inefficient-resource-cache = Serve static resources with a content hash in the file name and a long cache lifetime (Cache-Control: max-age=31536000, immutable).
finding-fix-seo-page-health-incorrect-content-type = Send every static resource with its correct Content-Type header (text/css, text/javascript, image/webp and so on).
finding-fix-seo-page-health-charset-mismatch = Declare the same character set in the HTTP Content-Type header and in <meta charset>, preferably UTF-8.
finding-fix-seo-page-health-hreflang-invalid = Use valid ISO 639-1 language codes in hreflang, optionally with an ISO 3166-1 region (en, en-GB), or x-default.
finding-fix-seo-page-health-jsonld-invalid = Give every JSON-LD block an @context (https://schema.org) and an @type.
finding-fix-seo-page-health-sync-head-scripts = Load scripts in the head with defer, async or type="module" unless they must run before the page renders.
finding-fix-seo-page-health-missing-sri = Add an integrity attribute with the hash of the file (and crossorigin) to scripts and stylesheets loaded from other origins.
finding-fix-seo-page-health-broken-fragment-links = Point anchor links to IDs that exist on the page, or add the missing id to the target element.
finding-fix-seo-page-health-generic-link-text = Name the destination of each link in its text instead of "here" or "more".
finding-fix-seo-page-health-soft-404 = Answer requests for non-existent URLs with HTTP 404 (or 410 for removed content), not with 200.
finding-fix-seo-page-health-custom-404-invalid-status = Serve the error page for non-existent URLs with status 404 or 410.
finding-fix-seo-page-health-generic-404-page = Give the 404 page a short explanation, a search field or links to the main sections.
finding-fix-seo-page-health-meta-refresh = Replace the meta refresh with a server-side 301 redirect.
finding-fix-seo-page-health-frames = Replace frame and frameset with a single document; embed external content with iframe only where needed.
finding-fix-seo-page-health-url-too-long = Use shorter, readable URLs without filler words and redirect a changed URL with 301.
finding-fix-seo-page-health-url-too-deep = Flatten the URL structure to at most five path levels and redirect old URLs with 301.
finding-fix-seo-page-health-dynamic-url = Use URLs without parameters for indexable content, or point a canonical link to the version without parameters.
finding-fix-seo-page-health-multiple-redirects = Point links and redirects straight at the final URL so a request needs at most one redirect.
finding-fix-seo-page-health-redirect = Link straight to the final URL instead of the redirecting one, in the navigation, the sitemap and canonical links alike.
finding-fix-seo-page-health-www-not-consolidated = Redirect one host variant (with or without www) with 301 to the other and use that one in canonical links.
finding-fix-seo-page-health-trailing-slash-inconsistent = Settle on one URL form, with or without a trailing slash, and redirect the other with 301.
finding-fix-seo-page-health-http-not-redirected-to-https = Redirect every http:// request with 301 to the same https:// URL.

## Mobile
finding-fix-mobile-missing-viewport = Add <meta name="viewport" content="width=device-width, initial-scale=1"> to the head.
finding-fix-mobile-improper-viewport = Set the viewport to width=device-width, initial-scale=1 instead of a fixed width.
finding-fix-mobile-not-scalable = Remove user-scalable=no and maximum-scale limits from the viewport meta tag so users can zoom.
finding-fix-mobile-small-targets = Enlarge touch targets to at least 44 × 44 CSS pixels through padding or min-width/min-height, or space them so far apart that a neighbour is not hit by mistake.
finding-fix-mobile-small-fonts = Raise the smallest font sizes on small screens to at least 12 px, body text preferably to 16 px.
finding-fix-mobile-horizontal-scroll = Find the element wider than the viewport (fixed widths, wide tables, images without max-width) and let it shrink or scroll inside its own container.
