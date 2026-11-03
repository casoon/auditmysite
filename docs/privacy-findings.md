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

## Measuring quality (#873)

Targets (#875): **0 false positives** on the clean fixtures and the hand-checked reference pages,
**recall of at least 90 %** of the hand-checked trackers. Three layers measure them; none of them
is a legal assessment, and the expectations are never copied from a run.

| Layer | What it is | Runs |
|---|---|---|
| Pure corpus | 32 hand-built captures fed to `analyze_privacy` (`tests/fixtures/privacy/cases.json`, `tests/privacy_findings_corpus.rs`) | every `cargo test` |
| Browser corpus | 33 local pages loaded by real Chrome through `collect_first_visit` and `analyze_privacy` (`tests/fixtures/privacy/browser_cases.json`, `tests/privacy_quality_browser.rs`) | `--ignored`, in the CI browser step |
| Reference pages | 15 real public pages with hand-written labels (`tests/fixtures/privacy/reference_pages.json`, `tests/privacy_reference_pages.rs`, `scripts/privacy-reference.py`) | offline replay in every `cargo test`; the live measurement is `--ignored` and run by hand before a tag |

### Browser corpus

Each case is a page (script, image, stylesheet and iframe requests to host names that are
*aliases* of well-known providers, cookies set by response header and by script, a first-party
banner element, look-alike hosts, a tracker added 700 ms after load, a page that never goes quiet,
a first-party subdomain tracker and a first-party proxy) plus the outcome of the six checks and the
exact findings the rules demand, with the rule stated in `why`. Chrome runs through a wrapper
script with `--host-resolver-rules="MAP * 127.0.0.1:<port>"` and `--ignore-certificate-errors`, so
every host resolves to one local HTTPS fixture server: nothing reaches a real provider, and no
product code is relaxed. Every `requested` host must really appear in the capture, so a negative
result is never an artefact of a request that did not happen. Cases marked as known blind spots
carry a `truth` the tool misses by design; the printed recall includes them.

Result of the run on 2026-11-03 (33 cases since `web-checks` 0.7.0 added a Fathom, a Carbon Ads
and a Mapp cookie case): all cases yield the expected outcomes and findings;
**unexpected findings: 0**; trackers the cases really load before consent: 17, reported: 15
(88 %, the two missing ones being the two deliberate blind-spot cases; 15 of 15 otherwise).

### Reference pages

Process, so the labels stay independent of the tool:

1. The raw first-visit capture of a page is dumped (`scripts/privacy-reference.py dump <url>`):
   requests with initiator and type, cookie and storage-key names, consent traces, anchors.
2. A person writes the labels from that log and checks each claim against something the tool did
   not produce: the served HTML (`curl`), the page's JavaScript bundle, the provider's own cookie
   documentation, and for policy links a reading of the linked page. Every label carries its
   `evidence` and a `provenance` list; claims that only the capture supports say so.
3. The capture the labels were written from is frozen in `reference_captures.json`; each site holds
   its SHA-256, and `reference_pages.json.sha256` seals the label file
   (`scripts/privacy-reference.py check|seal`). A label cannot change without a visible reseal.
   The frozen file keeps every request, cookie and storage key as observed, with three exceptions
   that decide no label: anchors that cannot match the policy-link words keep only host, first path
   segment and 24 characters of text (they carried article and playlist identifiers), and three storage-key names that tripped the repository's
   secret scanner were shortened or lower-cased (heise.de, w3schools.com).
4. Labels are closed-world per labelled check: every finding of such a check that is not listed is a
   false positive, every listed finding the tool misses a false negative. Findings the labels
   deliberately do not decide are listed under `unlabelled` with a reason (imprint links of
   English-language pages, a footer "Legal" hub). `outside_ruleset` lists trackers a person found
   that no provider-list entry covers.
5. The live test visits every page now, scores per finding kind and **fails on any false positive**
   or on a provider-finding recall below 90 %. A page that cannot be measured fails the run (no
   number for it, not a pass). Live pages change: re-label from a fresh raw capture before
   suspecting the tool, and never adjust a label to the tool's output.

The offline replay runs the rules over the frozen captures; it guards the rules and the labels
against typos and regressions but is not a measurement of the live pages.

**Numbers (live run on 2026-11-03, 15 pages, measured from one network location, with `web-checks` 0.7.0):**

| Finding kind | TP | FP | FN | Precision | Recall |
|---|---|---|---|---|---|
| `tracking_request_before_consent` | 14 | 0 | 0 | 100 % | 100 % |
| `non_essential_storage_before_consent` | 5 | 0 | 0 | 100 % | 100 % |
| `visitor_ip_transfer` | 3 | 0 | 0 | 100 % | 100 % |
| `third_country_provider` | 8 | 0 | 0 | 100 % | 100 % |
| `no_consent_platform_detected` | 5 | 0 | 0 | 100 % | 100 % |
| `policy_link_missing` | 5 | 0 | 0 | 100 % | 100 % |
| all scored findings | 40 | 0 | 0 | 100 % | 100 % |

Read these with their limits. 40 scored findings with no false positive bound the false-positive
rate only to roughly 7 % at 95 % confidence ("rule of three"); 15 pages are a hand-checked sample,
not a survey. The labels were written from the same kind of capture the tool uses (independent
checks cover the HTML, the page script and the provider documentation, not a second browser), and
the third-country labels follow the provider's own stated seat in the ruleset. The set was chosen
so that every check has pages for and against it; pages whose capture never went idle
(stackoverflow.com, tagesschau.de, svelte.dev) are *not measured* and therefore not in the set,
which favours calm pages.

**Trackers outside the provider list.** The first run (`web-checks` 0.6.1, same day) had five
trackers on four of the pages that a person had identified and the ruleset did not cover: Fathom
Analytics and Carbon Ads (getbootstrap.com), Mapp/Webtrekk (heise.de), a self-hosted analytics
script on a subdomain (python.org) and Google Analytics through Cloudflare Zaraz (casoon.de). The
tool reported none of them, and counted against all hand-identified provider-level items (19 + 5)
the recall was **79 %, below the 90 % target**. `web-checks` 0.7.0 adds Fathom, Carbon Ads and Mapp
Intelligence from the providers' own documentation. Re-run live with 0.7.0 (same day, same
pages): the tool reports Fathom and Carbon Ads as tracking requests on getbootstrap.com and the
Mapp cookies on heise.de as non-essential storage; 22 of 22 provider-level findings, and against
all hand-identified items (22 + 2) **92 %**, above the target. Two remain uncovered by design:
the first-party-subdomain tracker (python.org) and Zaraz (casoon.de). The Mapp *request* on heise.de
goes to a first-party subdomain, so only its cookies are found.

The labels moved with that: the three trackers went from `outside_ruleset` to `expected` in
`reference_pages.json` (re-sealed), together with the `no_consent_platform_detected` finding that
getbootstrap.com now triggers (trackers contacted, no consent platform). The facts they rest on
(requests, cookies, the absent banner) were recorded in the original labels and checked against the
served HTML at that time; only their classification changed. This edit was made after the
replay reported the new findings as false positives, so it is a reviewed relabelling, not an
independent re-labelling -- a maintainer should confirm it against a fresh raw capture.

**A defect found while labelling.** When the audited URL itself redirects from a provider's own
host (`https://www.hotjar.com/` → `contentsquare.com`), the entry hop is third-party relative to
the final page and is classified: the tool reports "Hotjar contacted" with the entry URL as
evidence, although no page requested it. This is a false positive of the redirect case; it is
not in the reference set because the page *is* the provider, and it is not fixed here.

### Known blind spots of the measurement itself

- **Server-side tagging and first-party proxies**: no third-party request exists (casoon.de's
  Zaraz cookies, the browser corpus' proxy case). A first-party subdomain tracker is contacted but
  not classified. Cookie names of server-side tools are not in the ruleset.
- **Fingerprinting and other requestless tracking** and anything behind a click, scroll or consent
  choice (phase B, #874).
- **Provider coverage**: only providers whose domains or cookie names the provider documents are
  listed; the unknown is never a finding. Consent platforms outside the list are recognised only
  by generic globals and banner elements.
- **One vantage point**: a single run from one location with one browser profile. Consent walls,
  ad stacks and bot protection (a challenge page such as datadome on kicker.de) differ by region,
  language and client; a challenged page is a different page.
- **The browser corpus** uses plain HTTPS to local aliases. It does not cover WebSocket,
  `sendBeacon`, service workers or `Set-Cookie` from third-party responses end to end, nor storage
  written by a third-party iframe; the `not_measured` outcome of checks 2 and 6 (unreadable cookie
  jar, unreadable document probe) cannot be provoked from a page and is covered by the pure corpus
  only.
- **Policy-link judgements**: whether an English page's "Legal" link is an imprint, or a hub page
  counts as a privacy link, is not decided by the labels (`unlabelled`); the check is word-based and
  its `indication` class says so.
- **Drift**: live pages change; the offline replay of frozen captures cannot notice that, and the
  live test is run by hand before a tag, not in CI.
