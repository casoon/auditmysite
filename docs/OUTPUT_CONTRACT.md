# Output Contract

`auditmysite` treats JSON output as a compatibility surface for automation.

## Stability Rules

- Single-report JSON is generated from `NormalizedReport` plus `metadata` and optional module payloads.
- Batch JSON is generated from normalized reports plus batch `summary`, optional `errors`, and deterministic `metadata`.
- Timestamps are data-derived, not generated at serialization time.
- New top-level fields may be added in a backward-compatible way.
- Existing required fields should not be removed or renamed without a versioned contract change.

## Schemas

- Single report: [json-report.schema.json](/Users/jseidel/GitHub/auditmysite/docs/json-report.schema.json)
- Batch report: [json-batch-report.schema.json](/Users/jseidel/GitHub/auditmysite/docs/json-batch-report.schema.json)

These schemas are validated in automated tests and in the local release check.

## Change Policy

When changing JSON output:

1. Update the Rust serializer.
2. Update the matching schema file.
3. Update or extend tests.
4. Run `./scripts/release-check.sh`.
