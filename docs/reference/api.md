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

### Configuring a run

`PipelineConfig` (module `audit`) describes one audit. `PipelineConfig::default()` is the default
CLI run: WCAG AA, the six modules of `--full`, no privacy check, no experimental indicators. Start
from it and change what differs:

```rust
use auditmysite::audit::{Module, ModuleSet, PipelineConfig};
use auditmysite::cli::WcagLevel;

let mut config = PipelineConfig::default()
    .with_wcag_level(WcagLevel::AAA)
    .with_modules(ModuleSet::none().with(Module::Seo).with(Module::Mobile));
config.timeout_secs = 60;
config.persist_artifacts = false;
```

- `modules: ModuleSet` selects the modules that run next to the accessibility checks
  (`Module::Performance`, `Seo`, `Security`, `Mobile`, `HtmlConform`, `Stack`, `Privacy`).
  `ModuleSet::full()` is the six `--full` modules without `Privacy`, `ModuleSet::none()` the
  accessibility checks alone; `with`, `without`, `contains`, `iter` and collecting from an
  iterator of `Module` do the rest.
- Whether a run is a full audit is not set but derived: `config.is_full_audit()` is true when all
  six `--full` modules run, and the report's `full_audit` carries that value.
- The setters cover what a run audits: `with_wcag_level`, `with_modules`, `with_experimental`
  and `with_interactive`. Every other option is a public field and is assigned.
- The struct and `Module` are `#[non_exhaustive]`: a new option or module is not a breaking
  change, and code outside the crate cannot write the struct literal.

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
| [accname-differential.md](https://github.com/casoon/auditmysite/blob/main/docs/accname-differential.md) | The `auditmysite-dev accname-diff` command |

## CLI

`auditmysite --help` and `auditmysite <command> --help` describe the installed version exactly.
The [CLI modes](../../guides/cli-modes/) guide gives the overview.
