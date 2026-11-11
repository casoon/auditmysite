# Texts of the security findings (`security::security_issue_text`, #936): one
# message per finding kind. Counts reach the messages as numbers, so the plural
# agreement lives here; which findings exist, their severity and the default
# values stay in Rust.

security-missing-https = Site is not served over HTTPS
security-missing-csp = Missing Content-Security-Policy header
security-missing-x-content-type-options = Missing X-Content-Type-Options header
security-missing-x-frame-options = Missing X-Frame-Options header (clickjacking protection)
security-missing-hsts = Missing HSTS header
security-missing-referrer-policy = Missing Referrer-Policy header
security-missing-permissions-policy = Missing Permissions-Policy header
security-missing-coop = Cross-Origin-Opener-Policy is not set. This header only matters if the page uses SharedArrayBuffer, high-resolution timers, or needs to isolate itself from cross-origin popups — if none of that applies, no action is needed here. If it does apply, set it to same-origin and verify popup/window interactions still work as expected.
security-missing-corp = Cross-Origin-Resource-Policy is not set. This header only matters if the page serves fonts, scripts, or media that other origins should be prevented from loading — if the site's resources are intentionally public, no action is needed here. If cross-origin loading should be restricted, set it to same-origin or same-site.
security-cors-wildcard-credentials = CORS allows any origin while also allowing credentials
security-hsts-preload-ineligible = HSTS preload directive is set but requirements aren't met (max-age={ $max_age }, includeSubDomains={ $subdomains }) — the site won't qualify for the HSTS preload list
security-permissions-policy-permissive = Permissions-Policy header is present but doesn't restrict any feature (empty or wildcard-only)
security-public-source-map =
    { $count } publicly reachable { $count ->
        [one] source map
       *[other] source maps
    } found{ $example } — exposes original source, comments, and internal file paths
security-public-source-map-example = {" "}(e.g. { $url })
security-csp-unsafe-inline-script = CSP allows unsafe-inline scripts without nonce/hash protection
security-csp-unsafe-eval-script = CSP allows unsafe-eval in script sources
security-csp-wildcard-script-source = CSP allows wildcard script sources
security-csp-unsafe-inline-style = CSP allows unsafe-inline styles without nonce/hash protection
security-csp-wildcard-source = CSP contains wildcard source expressions
security-csp-missing-directive = CSP missing { $directive } directive
