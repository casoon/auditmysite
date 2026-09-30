---
title: Reference
sidebarLabel: Library and contracts
description: auditmysite is used as a CLI; the crate also exposes its modules as a Rust library. Item-level documentation is generated on docs.rs, contracts and schemas live in the repository.
order: 1
---

## Rust library

The crate `auditmysite` exposes the audit pipeline, the WCAG engine and the report types as a
library. The generated reference for every module, type and function is on
[docs.rs/auditmysite](https://docs.rs/auditmysite), built with all features.

One example: the `Baseline` type in the `audit` module (`from_violations`, `diff`, `load`,
`save`) compares WCAG violations of a saved JSON report with a later run.

## Contracts and schemas

| File | Content |
| --- | --- |
| [OUTPUT_CONTRACT.md](https://github.com/casoon/auditmysite/blob/main/docs/OUTPUT_CONTRACT.md) | JSON stability rules, score and count semantics, metric definitions |
| [json-report.schema.json](https://github.com/casoon/auditmysite/blob/main/docs/json-report.schema.json) | JSON schema of the single-page report |
| [json-batch-report.schema.json](https://github.com/casoon/auditmysite/blob/main/docs/json-batch-report.schema.json) | JSON schema of the batch report |
| [technician-index.schema.json](https://github.com/casoon/auditmysite/blob/main/docs/technician-index.schema.json) | `index.json` of a per-page JSON run, see [Technician mode](../../guides/technician-mode/) |
| [technician-finding.schema.json](https://github.com/casoon/auditmysite/blob/main/docs/technician-finding.schema.json) | One line of `findings.jsonl` |
| [PDF_REPORT_CONTRACT.md](https://github.com/casoon/auditmysite/blob/main/docs/PDF_REPORT_CONTRACT.md) | What the PDF report contains and guarantees |
| [PARITY_CONTRACT.md](https://github.com/casoon/auditmysite/blob/main/docs/PARITY_CONTRACT.md) | Frozen WCAG coverage numbers, parity with axe-core and pa11y |
| [accname-differential.md](https://github.com/casoon/auditmysite/blob/main/docs/accname-differential.md) | The `accname-diff` command |

## CLI

`auditmysite --help` and `auditmysite <command> --help` describe the installed version exactly.
The [CLI modes](../../guides/cli-modes/) guide gives the overview.
