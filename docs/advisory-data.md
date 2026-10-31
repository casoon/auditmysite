# JavaScript Library Advisory Data

The library check (`best_practices.vulnerable_libraries`) compares detected library versions with `src/best_practices/advisories.json`. The file is compiled into the binary; replacing the data means editing the file and releasing. `vulnerable_libs.rs` only reads it.

## File layout

```json
{
  "schema_version": 1,
  "updated": "2026-10-26",
  "advisories": [
    {
      "library": "Lodash",
      "id": "local:lodash-prototype-pollution",
      "aliases": ["CVE-2020-8203"],
      "kind": "vulnerability",
      "severity": "medium",
      "description": "…",
      "affected": [{ "introduced": "0.0.0", "fixed": "4.18.0" }],
      "safe_version": "4.18.0+",
      "source_url": "https://…",
      "reviewed": "2026-10-26"
    }
  ]
}
```

- `id` is the GHSA id when there is one, otherwise `local:<slug>`. Unique across the file.
- `kind` is `vulnerability` or `unmaintained` (end-of-maintenance notice, no specific vulnerability).
- `affected` ranges are `introduced` inclusive, `fixed` exclusive, both `major.minor.patch`. `fixed: null` means no fixed release exists. Ranges of one advisory must not overlap.
- `source_url` is the https page the bounds were taken from.
- `reviewed` is the ISO date the bounds were last compared with `source_url`, or `null` when they were carried over unchecked. `null` entries show up as `review_status: "unreviewed"` in the report.
- `updated` is the date of the last edit to the file.

## Loading checks

`load_advisories` rejects the file (a test fails; at run time the embedded file would panic on first use) on an unsupported `schema_version`, unknown fields, an empty or duplicate `id`, a `library` outside `CHECKED_LIBRARIES`, a severity other than `high`/`medium`/`low`, a non-https `source_url`, a malformed date, an empty `affected` list, a range with `introduced >= fixed`, or overlapping ranges.

## Update process

1. For each library, read the advisory at `source_url` and copy its affected ranges and fixed version.
2. Set `reviewed` to the date of the check and `updated` to today. Do not set `reviewed` for bounds you did not compare.
3. Add a test pair to `BOUNDS` in `vulnerable_libs.rs` (last affected, first fixed version) and a corpus case under `tests/fixtures/detection_corpus_nonwcag/vulnerable_libs/` if the boundary is new.
4. A new library also goes into `CHECKED_LIBRARIES` and the detection scripts.
5. `cargo test --lib best_practices` and `cargo test --test rule_contract_gate`; `scripts/mutation-check.py --rule jQuery` (and the other libraries) when a range changed.
6. Add a line to `CHANGELOG.md` under `[Unreleased]`.

## What a match means

A match says the detected *version* lies in an advisory range. It never proves the vulnerable code path is reachable on the page, so `exploitability` is `unconfirmed` (or `not_applicable` for maintenance notices). A `confirmed` state is not modelled: it would need a reachability probe.
