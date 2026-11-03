# Privacy findings corpus (#871)

`cases.json` holds compact first-visit captures and what the rules say they
must produce. Expectations are written from the rules (see
`src/privacy/findings.rs`), never copied from a run.

Per case:

- `page`: host of the loaded page (default `example.com`).
- `collection`: `measured` (default), `partial` (with `gaps`), `failed`.
- `requests`: absolute URLs; a request is third-party when its host differs
  from `page`.
- `cookies`, `storage`: names / keys (values are never part of a capture).
- `consent_signals`, `links` (`[href, text]`): the document probe. `links`
  defaults to a privacy and an imprint link; `"links": []` is a page without
  anchors, `"signals": false` a probe that could not be read.
- `expect.outcomes`: six letters for the checks in order
  `tracking_requests, non_essential_storage, visitor_ip_transfer,
  third_country_provider, consent_platform, policy_links`:
  `F` found, `N` not found, `A` not applicable, `M` not measured.
- `expect.findings`: `[kind, confidence, provider category or "", document or ""]`.

Each of the six checks has a positive (`F`), negative (`N`), not-applicable
(`A`) and error (`M`) case here; `cases_cover_every_check_and_outcome` in
`tests/privacy_findings_corpus.rs` enforces that.
