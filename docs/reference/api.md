---
title: Reference
sidebarLabel: Library and contracts
description: The stability policy of the CLI, the JSON report and the Rust library, the curated library surface, and the contracts and schemas that live in the repository.
order: 1
---

## Rust library

The crate `auditmysite` is a CLI first. It is also published as a library, because
[auditmysite Studio](https://github.com/casoon/auditmysite_studio) drives the audit pipeline from
Rust. The generated reference is on [docs.rs/auditmysite](https://docs.rs/auditmysite), built with
the default features.

### Stability policy

What is stable, in this order of strength:

1. **The CLI** (`auditmysite --help`) and **the JSON report** (schemas below,
   [OUTPUT_CONTRACT.md](https://github.com/casoon/auditmysite/blob/main/docs/OUTPUT_CONTRACT.md)).
   A breaking change to either is a major version and a changelog entry.
2. **The curated library surface** listed in this page. Removing, renaming or changing the
   signature of a listed item is a major version; adding to it is a minor one. Types that are
   expected to grow are `#[non_exhaustive]` (`PipelineConfig`, `Module`), so a new field or
   variant there is a minor change that the compiler enforces. The report types follow the JSON
   contract: a field whose JSON counterpart is removed or changes meaning is a major change. They
   are not `#[non_exhaustive]` yet: until they are (#940), a new field breaks code that builds a
   report type with a struct literal or destructures it completely, and a new variant breaks an
   exhaustive `match` on a public enum. Such a release is marked under `**Library:**` in the
   changelog.
3. **Nothing else.** Every other `pub` item, and every item marked `#[doc(hidden)]`, is public
   only because the binaries, the integration tests or Studio import from it. It may change or
   disappear in any release, including a patch release, and a changelog entry under `**Library:**`
   is the only notice. This tier is a transition: #940 narrows it module by module to
   `pub(crate)`.

The surface is declared top-down: `src/lib.rs` makes a module `pub(crate)` unless it is in the
tables below, and the list of modules is checked by `cargo test --test library_surface` (a new
`pub mod` without a row here fails it). The surface is narrowed module by module (#940); the table
shows which modules are done and which are queued. Until a module is narrowed, only the items this
page lists for it are covered, not the module as a whole.

### Proposing an item for the surface

Open an issue naming the item, the use case and why the JSON report or the CLI does not serve it.
An item is added when it is needed by a consumer outside the repository, has a documented
contract, and a test pins its behaviour; it is then listed below in the same change that exports
it (and the module is added to `tests/fixtures/architecture/library_surface.txt` if it is new). An
item that only a test of this repository needs is not a reason to widen the surface: the test moves
next to the code instead.

### Curated surface

Paths are the ones to use; a root re-export and the module path name the same item.

| Purpose | Items |
| --- | --- |
| Pipeline entry | `audit::{audit_page, run_single_audit}`, `browser::{BrowserManager, BrowserOptions}` (also at the root) |
| Batch entry | `audit::{run_concurrent_batch, BatchConfig, BatchResult, parse_sitemap, read_url_file}`, `browser::{BrowserPool, PoolConfig}` |
| Configuration | `audit::{PipelineConfig, Module, ModuleSet}` and their setters, `model::{WcagLevel, InteractiveMode, ReportLevel, AnnexKind, ColorPolicy}` |
| Report types | `audit::{AuditReport, BatchReport, Verdict, VerdictResult}` and the types their fields name, `Severity`, `Violation`, `WcagResults` (root) |
| Normalised report | `audit::{NormalizedReport, AuditContext, normalize_checked, validate_batch, ReportContractError}`; `audit::normalize` is deprecated and stays until Studio has moved to `normalize_checked` |
| Baseline | `audit::{Baseline, BaselineStatus, BASELINE_CONTRACT, FINDING_ID_SCHEME}` |
| Output writers | `format_json_normalized` (root), `output::{format_json_batch, format_json_cached, format_json_replay, format_sarif, format_summary, format_ai_json, generate_pdf, generate_batch_pdf, UnifiedReport}`, `output::report_model::ReportConfig`, `output::terminal::TerminalOptions` (PDF writers need the `pdf` feature) |
| Studio contract | everything in `studio` (see [studio-contract.schema.json](https://github.com/casoon/auditmysite/blob/main/docs/studio-contract.schema.json)) |
| Errors | `error::{AuditError, Result}` |
| Compatibility paths | `cli::{WcagLevel, InteractiveMode}` re-export the `model` types; new code imports them from `model` |

### Modules

Tiers: `curated` is `pub` and the items above are covered; `queued` is `pub` only for the consumers
named, not covered, and to be narrowed (or hidden as test support); `crate` is `pub(crate)`.
Consumers: S = Studio, B = the two binaries, T(n) = n integration-test targets, D = a doc example.
`src/lib.rs` has 35 modules; the table has one row for each.

| Module | Tier | Decision | Consumers | Why |
| --- | --- | --- | --- | --- |
| `audit` | curated | stays `pub`; the other ~500 items narrowed in slices | S, B, T(48), D | pipeline, batch, report and normalised-report types; the largest module |
| `browser` | curated | stays `pub`; installer and detection internals narrowed | S, B, T(12), D | `BrowserManager`, `BrowserPool`, `BrowserOptions` are the entry |
| `cli` | curated | only the `model` re-exports are covered; `Args`, `Config`, `doctor`, `report_lint` become uncovered test and binary support | S, B, T(27), D | Studio names `cli::WcagLevel` and `cli::InteractiveMode` |
| `error` | curated | stays `pub` | B, T(3), D | `AuditError`, `Result` appear in every signature |
| `model` | curated | stays `pub` | via `cli` | shared value types, imports nothing else of the crate |
| `output` | curated | stays `pub`; builder, explanations, technician, search-experience internals narrowed | S, B, T(12), D | the writers |
| `studio` | curated | stays `pub` | S, T(1), D | the contract with Studio |
| `assessment` | crate | narrowed in this slice | none | its types are re-exported at the root where report fields name them |
| `commerce` | crate | narrowed in this slice | none | experimental indicator; the report carries its results |
| `a11y_journey` | queued | `pub(crate)`; tests move next to the code | T(1) | slice 2 |
| `accessibility` | queued | `pub(crate)`, `AXTree` stays at the root | B (dev), T(6) | slice 2 |
| `ai_visibility` | queued | `pub(crate)` | T(2) | slice 2 |
| `annotations` | queued | `pub(crate)` once the binaries' report writers live in the crate | B, T(2) | slice 5 |
| `best_practices` | queued | `pub(crate)` | T(31) | slice 3 |
| `content_visibility` | queued | `pub(crate)` | T(1) | slice 2 |
| `display` | queued | `pub(crate)` | B, T(1) | slice 5 |
| `html_conform` | queued | `pub(crate)` | T(2) | slice 2 |
| `i18n` | queued | `pub(crate)` | T(1) | slice 2 |
| `interaction` | queued | `pub(crate)` | T(2) | slice 2 |
| `lint` | queued | `pub(crate)`; `cli::report_lint` stays the entry for `report-lint` | T(1) | slice 2 |
| `mobile` | queued | `pub(crate)` | T(2) | slice 2 |
| `page_flow` | queued | `pub(crate)` | T(5) | slice 2 |
| `patterns` | queued | `pub(crate)` | T(3) | slice 2 |
| `performance` | queued | `pub(crate)` | T(3) | slice 3 |
| `privacy` | queued | `pub(crate)` | T(4) | slice 3 |
| `registry` | queued | `pub(crate)` | T(2) | slice 2 |
| `screen_reader` | queued | `pub(crate)` | T(3) | slice 2 |
| `security` | queued | `pub(crate)` | T(33) | slice 3 |
| `seo` | queued | `pub(crate)` | T(36) | slice 4, after #936 |
| `source_quality` | queued | `pub(crate)` | T(2) | slice 2 |
| `taxonomy` | queued | `pub(crate)` | T(32) | slice 4 |
| `tech_stack` | queued | `pub(crate)` | T(1) | slice 2 |
| `util` | queued | `pub(crate)` | B | slice 5 |
| `ux` | queued | `pub(crate)` | T(3) | slice 2 |
| `wcag` | queued | `pub(crate)`; `Severity`, `Violation`, `WcagResults` stay at the root | T(45) | slice 4 |

Order of the remaining slices and their rough size (`pub` declarations counted in the files of the
modules, methods included):

1. Done: `assessment`, `commerce` (about 40 public items, 1.6k lines).
2. The small analysis modules that one to six integration tests import (`a11y_journey`,
   `accessibility`, `ai_visibility`, `content_visibility`, `html_conform`, `i18n`, `interaction`,
   `lint`, `mobile`, `page_flow`, `patterns`, `registry`, `screen_reader`, `source_quality`,
   `tech_stack`, `ux`): about 400 public items. Each test either moves to a unit test next to the
   code or the module becomes `#[doc(hidden)] pub` with a comment naming the test as the reason.
3. `performance`, `privacy`, `security`, `best_practices`: about 190 public items; the detection
   corpus tests import them.
4. `wcag`, `taxonomy`, `seo`: about 400 public items, imported by 30 to 45 test targets (rule
   contracts, detection corpora). Largest slice; `seo` waits for #936.
5. The binary support: `util`, `display`, `annotations` and the `cli` pieces only `main.rs`
   uses (about 100 public items). They move into the binary or become `pub(crate)` behind a
   narrow `cli` entry.
6. Item-level narrowing inside the curated modules: `audit` (about 550 items), `output` (about
   230) and `browser` (about 200). The curated list above is the target; everything else becomes
   `pub(crate)`.

### Configuring a run

`PipelineConfig` (module `audit`) describes one audit. `PipelineConfig::default()` is the default
CLI run: WCAG AA, the six modules of `--full`, no privacy check, no experimental indicators. Start
from it and change what differs:

```rust
use auditmysite::audit::{Module, ModuleSet, PipelineConfig};
use auditmysite::model::WcagLevel;

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
