---
title: Reference
sidebarLabel: Library and contracts
description: What is stable (the CLI and the JSON report) and what is not (the Rust library), the library's entry points, and the contracts and schemas that live in the repository.
order: 1
---

## Rust library

The crate `auditmysite` is a CLI first. **Only the CLI and the JSON report are stable; the Rust API
is not.** It is published as a library only because
[auditmysite Studio](https://github.com/casoon/auditmysite_studio) drives the audit pipeline from
Rust. The generated reference is on [docs.rs/auditmysite](https://docs.rs/auditmysite), built with
the default features.

### Stability policy

Decision for 2.2.0 (#940): the stability promise covers the CLI and the JSON report, and nothing
in the Rust library. A program that needs audit results should run the CLI and read the JSON
report. Code that links the crate accepts that any `pub` item can change or disappear in any
release, a patch release included; the tiers below say how much care a change gets, not whether
it is allowed.

1. **Stable: the CLI** (`auditmysite --help`) and **the JSON report** (schemas below,
   [OUTPUT_CONTRACT.md](https://github.com/casoon/auditmysite/blob/main/docs/OUTPUT_CONTRACT.md)).
   A breaking change to either is a major version and a changelog entry.
2. **Not stable, but maintained: the curated library surface** listed in this page, the entry
   points that Studio and the integration tests use. It is kept source-compatible where that is
   reasonable and a change to it is announced in the changelog under `**Library:**`, but it is
   not a semver promise: a breaking change to it can ship in a minor release. A new field of a
   struct or a new variant of an enum of the surface does not break a consumer, and the compiler
   enforces it: these types are `#[non_exhaustive]`:
   - the report types `AuditReport`, `BatchReport`, `NormalizedReport`, `NormalizedFinding`,
     `VerdictResult`, `BaselineStatus`, `AuditContext`, `BatchResult`, `ReportContractError`,
     `Violation` and `WcagResults`, and every struct and enum of the module `audit` that their
     fields name, down to the leaves (sections, execution and quality, scores, risk, finding
     contract and derivation, interactive findings, interpretation, batch summary, diagnostics and
     consistency, baseline);
   - the enums `Verdict`, `AuditError`, `BatchAuditError`, `Module`, `ExperimentalIndicator`,
     `InteractiveMode`, `ReportLevel` and `AnnexKind`, and the struct `PipelineConfig`. A new
     verdict would also change the CLI exit codes (tier 1), so it is a major version all the same;
     for code that matches on `Verdict`, `Verdict::exit_code` is the stable mapping.

   Outside the crate such a struct cannot be written as a literal, not even with `..base`: start
   from its constructor (`AuditReport::new`, `BatchReport::from_reports`, `Violation::new`,
   `WcagResults::new`, `PipelineConfig::default()`) or `Default` and assign the fields, and end a
   struct pattern with `..`. A `match` on one of the enums needs a wildcard arm (`_ =>`). Three
   public enums stay exhaustive on purpose, so a `match` on them can be complete: `WcagLevel` (the
   WCAG levels A, AA, AAA), `ColorPolicy` (`auto`, `always`, `never`) and `Severity` (it is
   `a11y_report::Severity` from barrierlab and changes there). The report types
   also follow the JSON contract: a field whose JSON counterpart is removed or changes meaning is
   a change the changelog announces.
3. **Nothing else.** Every other `pub` item, and every item marked `#[doc(hidden)]`, is public
   only because the binaries, the integration tests or Studio import from it. It may change or
   disappear in any release, including a patch release, and a changelog entry under `**Library:**`
   is the only notice, if there is one. This tier is a transition: #940 narrows it module by module to
   `pub(crate)`.

The surface is declared top-down: `src/lib.rs` makes a module `pub(crate)` unless it is in the
tables below, and the list of modules is checked by `cargo test --test library_surface` (a new
`pub mod` without a row here fails it). The surface is narrowed module by module (#940); the table
shows which modules are done and which are queued. Until a module is narrowed, only the items this
page lists for it are entry points, not the module as a whole.

### Proposing an item for the surface

Open an issue naming the item, the use case and why the JSON report or the CLI does not serve it.
An item is added when it is needed by a consumer outside the repository, has a documented
contract, and a test pins its behaviour; it is then listed below in the same change that exports
it (and the module is added to `tests/fixtures/architecture/library_surface.txt` if it is new). An
item that only a test of this repository needs is not a reason to widen the surface: the test moves
next to the code instead.

### Curated surface

The entry points of tier 2, not a stability promise (see the policy above). Paths are the ones to
use; a root re-export and the module path name the same item.

| Purpose | Items |
| --- | --- |
| Pipeline entry | `audit::{audit_page, run_single_audit}`, `browser::{BrowserManager, BrowserOptions}` (also at the root) |
| Batch entry | `audit::{run_concurrent_batch, BatchConfig, BatchResult, parse_sitemap, read_url_file}`, `browser::{BrowserPool, PoolConfig}` |
| Configuration | `audit::{PipelineConfig, Module, ModuleSet}` and their setters, `model::{WcagLevel, InteractiveMode, ReportLevel, AnnexKind, ColorPolicy}`; `model::FindingOrigin` (the `origin` of a finding, #1109) |
| Report types | `audit::{AuditReport, BatchReport, Verdict, VerdictResult}` and the types their fields name, `Severity`, `Violation`, `WcagResults` (root) |
| Normalised report | `audit::{NormalizedReport, AuditContext, normalize_checked, validate_batch, ReportContractError}`; `audit::normalize` is deprecated and stays until Studio has moved to `normalize_checked` |
| Baseline | `audit::{Baseline, BaselineStatus, BASELINE_CONTRACT, FINDING_ID_SCHEME}` |
| Output writers | `format_json_normalized` (root), `output::{format_json_batch, format_json_cached, format_json_replay, format_sarif, format_summary, format_ai_json, generate_pdf, generate_batch_pdf, UnifiedReport}`, `output::report_model::ReportConfig`, `output::terminal::TerminalOptions` (PDF writers need the `pdf` feature) |
| Studio contract | everything in `studio` (see [studio-contract.schema.json](https://github.com/casoon/auditmysite/blob/main/docs/studio-contract.schema.json)) |
| Errors | `error::{AuditError, Result}` |
| Compatibility paths | `cli::{WcagLevel, InteractiveMode}` re-export the `model` types; new code imports them from `model` |

### Modules

Tiers: `curated` is `pub` and the items above are the maintained entry points; `hidden` is `pub` with `#[doc(hidden)]`,
deliberately not API (public only because the consumer named drives it, not covered, not shown on
docs.rs); `queued` is `pub` only for the consumers named, not covered, and still to be narrowed or
hidden; `crate` is `pub(crate)`.
Consumers: S = Studio, B = the two binaries, T(n) = n integration-test targets, D = a doc example.
`src/lib.rs` has 35 modules; the table has one row for each: 7 curated, 19 hidden, 0 queued and 9 crate.

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
| `html_conform` | crate | narrowed in slice 2; `HtmlConformAnalysis`, `HtmlConformFinding` and `SchemaDetail` at the root | none | the tests build the report field from the root type and JSON |
| `tech_stack` | crate | narrowed in slice 2; `TechStackAnalysis`, `DetectedTech`, `TechCategory` and `TechConfidence` at the root | none | the test builds the report field from the root type and JSON |
| `display` | crate | narrowed in slice 5; `AuditedDisplayMode`, `DisplayMode`, `DisplayModesInfo` and `VisualisationCount` at the root | none | report and pipeline fields name them; the binary and the test import the root types |
| `i18n` | crate | narrowed in slice 5; `I18n` at the root | none | `audit::exclusion_note` and `label_localized` take it; the text functions are crate-internal |
| `lint` | crate | narrowed in slice 5; `cli::report_lint::run_report_lint` stays the entry for `report-lint` | none | the consistency test now drives the command logic instead of `lint::lint` |
| `util` | crate | narrowed in slice 5; `cli::{build_browser_client, truncate_url}` re-export the two helpers the binaries share with the library | none | the other helpers are crate-internal |
| `a11y_journey` | hidden | Chrome integration test drives it; narrowing needs the Chrome tests moved next to the code, not decided | T(1) | forced by a Chrome-driven test |
| `accessibility` | hidden | tests read AX-tree internals and the dev binary's `accname-diff` uses them; `AXTree` is also at the root | B (dev), T(6) | forced by tests and the dev binary |
| `ai_visibility` | hidden | experimental indicator (#891), otherwise untouched | T(2) | `tests/module_fixtures.rs` drives its `AuditModule` impl |
| `annotations` | hidden | the report writers of the binaries and the dev binary import it | B, T(2) | forced by the binary side and tests |
| `best_practices` | hidden | the rule-contract and detection corpus tests import it | T(31) | forced by the corpus |
| `content_visibility` | hidden | experimental indicator (#891), otherwise untouched | T(1) | `tests/module_fixtures.rs` drives its `AuditModule` impl |
| `interaction` | hidden | Chrome integration tests drive it; narrowing needs the Chrome tests moved next to the code, not decided | T(2) | forced by Chrome-driven tests |
| `page_flow` | hidden | experimental indicator (#891), otherwise untouched | T(5) | `tests/module_fixtures.rs` drives its `AuditModule` impl |
| `patterns` | hidden | Chrome integration tests drive it | T(3) | forced by Chrome-driven tests |
| `performance` | hidden | the detection corpus tests import it | T(3) | forced by the corpus |
| `privacy` | hidden | the privacy corpus and findings tests import it | T(4) | forced by the corpus |
| `registry` | hidden | tests read `registry::REGISTRY` | T(2) | forced by tests |
| `screen_reader` | hidden | tests use `build_sr_audit_report` | T(3) | forced by tests |
| `security` | hidden | the detection corpus tests import it | T(33) | forced by the corpus |
| `seo` | hidden | the detection corpus tests import it; narrowing waits for #936 | T(36) | forced by the corpus |
| `source_quality` | hidden | experimental indicator (#891), otherwise untouched | T(2) | `tests/module_fixtures.rs` drives its `AuditModule` impl |
| `taxonomy` | hidden | the rule contracts and the catalogue test import it; `Dimension`, `Subcategory` and `IssueClass` stay reachable here | T(32) | forced by the contracts |
| `ux` | hidden | experimental indicator (#891), otherwise untouched | T(3) | `tests/module_fixtures.rs` drives its `AuditModule` impl |
| `wcag` | hidden | the rule contracts and the detection corpora import it; `Severity`, `Violation` and `WcagResults` are also at the root | T(45) | forced by the contracts |
| `mobile` | crate | narrowed once the UX heuristics moved to `ux` and are called (#1035); `analyze_mobile_friendliness` and `MobileFriendliness` are at the root | none | the tests build the report field from the root type |

How the queued list was worked off (#940), with the size of each slice (`pub` declarations counted
in the files of the modules, methods included):

1. Done: `assessment`, `commerce` (about 40 public items, 1.6k lines), narrowed to `pub(crate)`.
2. Done: `html_conform` and `tech_stack`, whose tests only built a report field, are `pub(crate)`;
   their report types are at the root.
3. Done: the `hidden` tier. A module that a test, a corpus or the dev binary drives for more than a
   report field is `pub` with `#[doc(hidden)]` instead of being forced `pub(crate)`: `a11y_journey`,
   `accessibility`, `annotations`, `best_practices`, `interaction`, `patterns`, `performance`,
   `privacy`, `registry`, `screen_reader`, `security`, `seo`, `taxonomy`, `wcag` (the corpora, the
   rule contracts, the Chrome integration tests and the dev binary) and the five experimental
   `AuditModule` modules `ai_visibility`, `content_visibility`, `page_flow`, `source_quality` and
   `ux` (only because `tests/module_fixtures.rs` drives them; the experimental modules stay
   otherwise untouched until #891 is decided). The guard checks that the table and the attribute
   agree. `a11y_journey` and `interaction` could become `pub(crate)` once their Chrome tests move
   next to the code; that is not decided.
4. Done: `display`, `i18n`, `lint` and `util` are `pub(crate)`. The root re-exports the types that
   fields and signatures name (`AuditedDisplayMode`, `DisplayMode`, `DisplayModesInfo`,
   `VisualisationCount`, `I18n`); the binaries take `build_browser_client` and `truncate_url`
   from `cli`; the lint test drives `cli::report_lint::run_report_lint`.
5. Done: `mobile` is `pub(crate)`. Its `ux_heuristics`, the one part nothing in the crate called,
   moved to `ux` and is wired to the experimental `ux` indicator (#1035); `MobileFriendliness` and
   `analyze_mobile_friendliness` are at the root. The queued list is empty and `QUEUED_CEILING` is 0.
6. Open: item-level narrowing inside the curated modules: `audit` (about 550 items), `output`
   (about 230) and `browser` (about 200). The curated list above is the target; everything else
   becomes `pub(crate)`. The `hidden` modules are not narrowed item by item: they are not API.

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
