# Parity Contract

`scripts/parity-check.sh` compares AuditMySite with axe-core and pa11y. The
report is diagnostic, not a winner/loser benchmark: different engines use
different data sources.

## Frozen Numbers

The public methodology/landing-page numbers are frozen in
`docs/PARITY_CONTRACT.jsonc` and guarded by `tests/parity_contract.rs`:

- WCAG scope: WCAG 2.1 AA
- Total WCAG A/AA criteria: 50
- Automated WCAG A/AA criteria: 36
- Manual-review criteria: 10

When the WCAG coverage manifest changes, update the contract and this document
in the same change.

## Divergence Policy

- `auditmysite-only`: axe-core passes or has no mapped violation while
  AuditMySite reports a mapped finding. Investigate as a possible false
  positive before changing rules.
- `axe-core-only`: axe-core reports a confirmed violation with no mapped
  AuditMySite finding. Investigate as a possible false negative.
- `axe incomplete`: manual-review signal, not an automatic false negative.

## Mapping Source

axe-core to AuditMySite parity mappings live in `docs/PARITY_CONTRACT.jsonc`.
The parity script reads this file directly; tests verify that mapped axe IDs
exist in the taxonomy and that the stable fixture expectations are represented.
