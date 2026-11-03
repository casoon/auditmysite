# Privacy findings

The opt-in `--privacy-check` reports what a **first visit** technically leaves behind. It is not
part of `--full` and costs one extra page load per audited page. It is a report of observations,
not a legal opinion: there is no GDPR / ePrivacy compliance score, no pass/fail verdict and no
statement about whether a consent is needed or valid.

## What is measured (phase A)

One extra visit in a browser context of its own: no stored consent, no Do-Not-Track / Global
Privacy Control, no click, no scroll. After the load the capture waits for the network to go quiet
and for a settle time, then reads all requests, all cookies (never values), the web-storage keys
of every frame (never values), consent-platform traces of the document and its anchors. What a page
does *after* a consent choice is not measured.

Hosts, URLs and cookie/storage names are classified with the `third_party` family of `web-checks`
(provider, category, seat as stated by the provider; ruleset version in `ruleset_version`). An
unknown host is not a finding; it is listed in `unclassified_hosts`.

## Checks

| Check | Finding kind(s) | Confidence |
|---|---|---|
| `tracking_requests` | `tracking_request_before_consent` (per provider) | `evidence` for analytics and advertising; `indication` for a tag manager (it may fire tags) |
| `non_essential_storage` | `non_essential_storage_before_consent` (per provider) | `evidence` (cookie / storage name documented by the provider; consent-platform names are excluded) |
| `visitor_ip_transfer` | `visitor_ip_transfer` (per provider) | `evidence` for fonts and CDNs; `indication` for maps, video and social embeds (may be the content asked for) |
| `third_country_provider` | `third_country_provider` (one, lists providers) | `indication` (seat per the provider's own statement, not the data flow) |
| `consent_platform` | `no_consent_platform_detected` | `indication` (trackers present, no consent trace in requests, cookies, storage or document) |
| `policy_links` | `policy_link_missing`, `policy_link_unusable` | `indication` (missing: the link may be named unusually); `evidence` (a link was found with an empty, `#` or script target) |

`manual-check` is part of the vocabulary for findings only a person can decide; no automatic rule
emits it yet.

Each check ends in `found`, `not_found`, `not_applicable` (nothing to look at, e.g. no third-party
request) or `not_measured` (see below).

## Not measured, never clean

A capture that failed makes every check `not_measured` with the reason code. A partial capture
(unreadable cookie jar, storage or document probe; a network that never went idle) keeps what was
found, but a check without a finding then reads `not_measured` instead of `not_found`: a late tag
or an unreadable jar cannot be ruled out.

## Policy and imprint links (check 6)

Links come from the anchors of the first-visit document, recognised by words in their text or
target (`Datenschutz`, `privacy`, `Impressum`, `imprint`, `legal notice`, ...). **The link target is
not fetched.** Reachability beyond "the link has a usable target" is a manual check; no network
request leaves the browser context for this check.

## Known blind spots

Server-side tagging and first-party proxies (the tracker sits on the page's own host), fingerprinting
without a request, consent platforms that leave no trace before interaction and are not in the list,
and anything behind a click. A first-party subdomain tracker is contacted but not classified.

## In the report (PDF)

Single page: a chapter lists every check with its result (including `not_measured` and the reason),
the providers contacted on the first visit (category, stated seat, request count), and the
indicators with confidence class and evidence (request URLs without query, cookie / storage key
names). Batch: counts over the pages whose first visit was measured (indicators and recurring
providers by affected pages, distribution of check results, one compact row per page). A page
whose capture failed is listed as not measured and sits outside every denominator. The text is
localized in the PDF layer only; the JSON stays canonical English and unchanged.
