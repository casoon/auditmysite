# Contributing to auditmysite

Thanks for helping. This page gets you from a fresh clone to a pull request, and
walks through the one task that has the most rules around it: adding a WCAG rule
together with its test contract.

By taking part you agree to the [Code of Conduct](../CODE_OF_CONDUCT.md).

## Where to start

- **A wrong result** (a missed barrier or a false alarm): open an issue with the
  matching template, bug or false positive. A page or a minimal HTML snippet that
  reproduces it is worth more than any description.
- **A new rule or check**: open a rule request first, so we can agree on the
  normative source and on where the rule lives (see
  [Where a rule belongs](#where-a-rule-belongs)) before you write code.
- **A small fix or doc change**: just send a pull request.

## Development setup

Prerequisites:

- **Rust**, pinned in [`rust-toolchain.toml`](../rust-toolchain.toml) (currently
  1.98.1, with `clippy` and `rustfmt`). With [rustup](https://rustup.rs) the right
  toolchain installs itself on the first `cargo` call; there is no promise for
  older compilers.
- **Chrome or Chromium** for everything that renders a page: the audit itself, the
  corpus tests and the mutation check. Chrome is **never downloaded
  automatically**. Point auditmysite at a browser you already have, or ask for a
  managed copy explicitly:

  ```bash
  auditmysite browser detect     # what was found
  auditmysite browser install    # Chrome for Testing, only on your request
  ```

  Details and the lookup order (`--browser-path`, `AUDITMYSITE_BROWSER`, system
  paths, managed install): [Browser setup](guides/browser-setup.md).

Build and try it:

```bash
git clone https://github.com/casoon/auditmysite.git
cd auditmysite
git config core.hooksPath .githooks     # versioned git hooks, see below
cargo build
cargo run -- https://example.com --format table
```

Browser-free unit tests run without Chrome: `cargo test --lib`.

### Git hooks

The hooks live in `.githooks/` and are switched on by the `core.hooksPath` line
above.

- `pre-commit` scans staged changes with [`nosecrets`](https://www.npmjs.com/package/@casoon/nosecrets)
  (`npm install -g @casoon/nosecrets` or `cargo install nosecrets-cli`), then runs
  `rustfmt --check` on staged `.rs` files and
  `cargo clippy --lib --bins --all-features -- -D warnings`.
- `pre-push` runs `cargo clippy --all-targets --all-features -- -D warnings` and
  `cargo test`.

Both can be bypassed with `SKIP_RUST_CHECKS=1` when you know why; CI will still
check the same things. More in [Development](guides/development.md).

### Before you push

```bash
cargo fmt
cargo check --all-features     # what CI builds; the default feature set hides breakage
cargo clippy --all-targets --all-features -- -D warnings
cargo test                     # includes the rule contract gate
```

Optional features (`pdf`) are not built by a plain
`cargo check`; code that uses them must be gated with `#[cfg(feature = "...")]`.

Tests that need Chrome are `#[ignore]`d and run with `cargo test <name> -- --ignored`.
CI runs a quick check on every pull request and the full suite (browser,
corpus, all features) before a release.

## Where a rule belongs

auditmysite shares its browser-free rules with a sibling project,
[barrierlab](https://github.com/casoon/barrierlab) (crates `a11y-rules`,
`accname`, `a11y-dom`, ...). Which side a rule belongs on decides where you work:

| The rule needs ... | It belongs in |
| --- | --- |
| only the HTML/DOM, stylesheets or the accessibility tree as data | **barrierlab** (`a11y-rules`), not here |
| a running browser: interaction, frames, paste, focus measurement, computed geometry via CDP | **here**, `src/wcag/rules/` |

For a browser-free rule, open the issue or pull request in barrierlab; auditmysite
picks the rule up through a version bump (`src/wcag/shared.rs` lists what it
takes over). Please do not copy such a rule into this repository, and do not
patch the shared rule table here. If you are unsure which side a rule is on, ask
in the rule request.

Everything below describes a **browser-bound** rule.

## Adding a browser-bound WCAG rule

Plan on well under an hour for a simple rule. The steps, in order:

1. [Write the rule](#1-write-the-rule) in `src/wcag/rules/`
2. [Register it](#2-register-it)
3. [Add corpus cases](#3-add-corpus-cases)
4. [Write the contract](#4-write-the-contract)
5. [Run the checks](#5-run-the-checks)

Use `src/wcag/rules/orientation.rs` as a living example; it is a small
page rule with a contract in `tests/fixtures/rule_contracts/wcag/g6_visual.toml`.

### Two kinds of rule

- **Page rule** (most browser-bound rules): an `async fn(&Page) -> Vec<Violation>`
  that evaluates JavaScript in the page through CDP. Listed in the `PAGE_RULES`
  table of `src/wcag/rules/page_rules.rs`.
- **Tree rule**: a plain `fn(&AXTree) -> WcagResults` over Chrome's accessibility
  tree, listed in `TREE_RULES` in `src/wcag/engine.rs`. Only very few remain;
  anything that can be decided from the tree alone without a browser belongs in
  barrierlab (see above). `src/wcag/rules/focus_visible.rs` is the model.

The rest of this section follows the page rule.

### 1. Write the rule

Create `src/wcag/rules/my_rule.rs`. This follows `orientation.rs`; the types are the
real ones from `crate::wcag::types`:

```rust
//! WCAG X.Y.Z Name
//!
//! What the criterion requires, and which part of it this rule measures.
//! Level AA

use chromiumoxide::Page;

use crate::wcag::types::{RuleMetadata, Severity, Violation};
use crate::wcag::WcagLevel;

pub(super) const MY_RULE: RuleMetadata = RuleMetadata {
    id: "X.Y.Z",                 // the WCAG success criterion
    name: "Name",
    level: WcagLevel::AA,
    severity: Severity::High,
    description: "One sentence: what must hold",
    help_url: "https://www.w3.org/WAI/WCAG22/Understanding/<slug>.html",
    axe_id: "my-rule",           // stable rule id; shows up in findings, JSON and the contract
    tags: &["wcag2aa", "wcagxyz", "cat.structure"],
};

// The probe answers a JSON object. Do not catch errors inside it: a probe that
// could not read the page has measured nothing and must surface as a technical
// failure, never as a pass.
const MY_RULE_JS: &str = r#"
(function() {
  var offenders = [];
  // ... query the DOM, push a selector/detail per offending element ...
  return { measured: true, offenders: offenders };
})()
"#;

pub async fn check_my_rule_with_page(page: &Page) -> Vec<Violation> {
    let val = match crate::wcag::types::evaluate_or_fail(page, &MY_RULE, MY_RULE_JS).await {
        Ok(v) => v,
        Err(violations) => return violations, // evaluation failed: technical-failure finding
    };

    // Without the field the probe gave no answer; that is not a pass.
    let Some(offenders) = val.get("offenders").and_then(|v| v.as_array()) else {
        return vec![crate::wcag::technical_rule_failure(
            &MY_RULE,
            "invalid_evaluation_value",
        )];
    };

    offenders
        .iter()
        .map(|o| {
            let selector = o.as_str().unwrap_or("html");
            Violation::new(
                MY_RULE.id,
                MY_RULE.name,
                MY_RULE.level,
                Severity::High,
                "What is wrong, in one sentence",
                "document",
            )
            .with_selector(selector)
            .with_fix("How to fix it")
            .with_rule_id(MY_RULE.axe_id)
            .with_help_url(MY_RULE.help_url)
        })
        .collect()
}
```

Points that decide whether the contract passes later:

- Emit **one finding per offending element** and give it a selector
  (`with_selector`); a violation without a location fails the corpus.
- Use `evaluate_or_fail` and `technical_rule_failure` for every way the probe can
  fail. These produce the "errored" outcome the contract requires.
- Findings are canonical English. Report-facing wording in other languages is
  derived in the PDF layer, never baked into the finding (see
  [Localization](#localization)).
- A finding that is a human decision, not a certain failure, is a review:
  `.as_warning()` on the `Violation`, as in `focus_visible.rs`.

### 2. Register it

In `src/wcag/rules/mod.rs`, add the module and the re-export (keep alphabetical order):

```rust
mod my_rule;
// ...
pub use my_rule::check_my_rule_with_page;
```

In `src/wcag/rules/page_rules.rs`, import the function in the `use super::{...}` list
and add a row to `PAGE_RULES`, at the place that matches its level group:

```rust
PageRuleEntry {
    rule_id: "X.Y.Z/my-rule",          // executor id, used for logging and grouping
    outcome_ids: &["my-rule"],         // the axe_id of every finding this executor can emit
    name: "my-rule",                   // for the "Found N <name> violations" log line
    min_level: WcagLevel::AA,          // runs when the configured level is at least this
    check_fn: |p| Box::pin(check_my_rule_with_page(p)),
},
```

`outcome_ids` must list every `axe_id` the check can produce: the outcomes a run
is expected to record are built from it. Nothing else needs wiring: the pipeline,
the expected-outcome list and the cache key are all derived from `PAGE_RULES`.

Then update the two level counters in the `tests` module at the bottom of
`page_rules.rs`: `level_a_filter_includes_only_level_a_rules` for a level A rule,
`level_aa_filter_includes_aa_plus_a_rules` for level A or AA. Bump the number
and, if it helps the next person, add one line to the comment above it.

### 3. Add corpus cases

The detection corpus is `tests/fixtures/detection_corpus/`: for every case an
HTML page `<case>.html` and the expected outcome `<case>.expected.json`. Chrome
audits the page with the real pipeline and the result is compared **strictly**:
every finding the page produces, for any rule, must be declared in the case, so a
missing declaration fails as a false positive.

You need at least:

| Case | Verdict | What it shows |
| --- | --- | --- |
| `my_rule_violation` | `violation` | a page the rule must flag (with `selector`, `occurrences`) |
| `my_rule_ok` | `pass` | applicable content that conforms |
| a not-applicable page | `not_applicable` | nothing for the rule to look at (skip if the rule applies to every page and say so with `always_applicable = true`) |
| the shared error case | `errored` | a page that breaks the probe |

Write the expectation from the success criterion, **before** you run the
detector. If the run disagrees, that is a finding to resolve (a detector bug or a
wrong reading of the norm), not a reason to copy the run's output into the JSON.

Start from the skeleton of an existing "ok" page, for example
`visual_orientation_ok.html`: it already has a skip link, a labelled `nav`, a `main`
with an `h1`, a `lang`, a viewport meta and 44 px targets, so the *other* rules stay
quiet. A bare `<html><body>` produces unrelated review findings
(`keyboard/skip-link-missing`, `landmarks/navigation-missing`, ...), and each of them
would have to be declared.

```json
{
  "case": "my_rule_violation",
  "expectations": [
    { "rule_id": "my-rule", "selector": "html", "verdict": "violation", "occurrences": 1 }
  ]
}
```

Verdicts are `violation`, `needs_review`, `pass`, `absent`, `not_applicable` and
`errored`; their exact meaning is in the
[contracts README](../tests/fixtures/rule_contracts/README.md#corpus-verdicts).

**The error case.** Every rule that evaluates the page must also be shown not to
pass silently when its method breaks. `error_page_scripts_sabotaged.html` replaces
`querySelector`, `querySelectorAll`, `getElementsByTagName` and `getComputedStyle`
with functions that throw. Make sure your probe uses one of them, and add your id
to `error_page_scripts_sabotaged.expected.json`:

```json
{ "rule_id": "my-rule", "verdict": "errored" }
```

A probe that avoids all four APIs (for example only `document.documentElement`)
would still run and report "pass" on that page, so the case would fail; either
query through one of them or give the rule its own error case or an `error_test`
(see the README).

Run only your cases while writing them:

```bash
CORPUS_CASES=my_rule_violation,my_rule_ok,error_page_scripts_sabotaged \
  cargo test --test detection_corpus_test -- --ignored --test-threads=1
```

### 4. Write the contract

Every active rule id needs one `[[rule]]` entry in `tests/fixtures/rule_contracts/`.
For a WCAG rule that is one of the files under `wcag/` (grouped by area:
`g1_aria`, `g2_forms`, `g3_structure`, `g4_content`, `g5_interaction`, `g6_visual`);
pick the closest and append at its end. The
[README there](../tests/fixtures/rule_contracts/README.md) documents every field. A
minimal contract for a page rule that applies to every page:

```toml
[[rule]]
id = "my-rule"                              # the axe_id
errored = ["error_page_scripts_sabotaged"]
automation = "automated"
method = "page"
source = "WCAG 2.2 SC X.Y.Z Name"           # the normative source
capabilities = ["dom"]                      # dom, ax_tree, computed_style, geometry, ...
applicability = "What the rule looks at, and when it does not apply."
expectation = """
What fails, what passes, and what is deliberately someone else's rule.
Written from the norm, not from the detector."""
always_applicable = true                    # drop this and add not_applicable = [...] otherwise
positive = ["my_rule_violation"]
negative = ["my_rule_ok"]

[rule.variants]                             # all seven dimensions: cases, or a reason
visibility = { cases = ["my_rule_ok"] }
native_vs_aria = { not_relevant = "Why roles do not matter here." }
alternative_labels = { not_relevant = "..." }
frames_shadow_dom = { not_relevant = "..." }
responsive = { not_relevant = "..." }
dynamic = { not_relevant = "..." }
thresholds = { not_relevant = "Presence check, no threshold." }

[rule.test_strength]
status = "killed"

[[rule.test_strength.mutations]]
target = "src/wcag/rules/my_rule.rs"
change = "invert the decisive condition"
find = "if offenders.is_empty() {"           # exact text in the target file, once
replace = "if !offenders.is_empty() {"
killed_by = "corpus:my_rule_violation,my_rule_ok"
```

What the gate enforces:

- An id without a contract, and a contract for an id nothing produces, both fail.
- An automated rule needs a `positive`, a `negative`, a `not_applicable` (unless
  `always_applicable`) and an `errored` case. The cases must exist and declare that
  verdict for this rule id.
- All seven variant dimensions are either covered by named cases or carry a reason.
  A reason that says nothing ("n/a") is not what we want; say why the dimension
  cannot matter for this rule.
- Test strength: at least one mutation, a one-line change to the detector that a
  named test must turn red (`killed_by` is `corpus:<cases>`, `test:<binary>[::filter]`
  or `lib:<filter>`), or `status = "not_mutable"` with a reason.

### 5. Run the checks

```bash
cargo test --lib page_rules                       # the level counters
cargo test --test rule_contract_gate              # browser-free; the contract completeness gate
UPDATE_RULE_DOCS=1 cargo test --test rule_docs    # regenerates the rule's documentation page (site/src/data/rules/); commit the result
CORPUS_CASES=my_rule_violation,my_rule_ok,error_page_scripts_sabotaged \
  cargo test --test detection_corpus_test -- --ignored --test-threads=1   # needs Chrome
scripts/mutation-check.py --rule my-rule          # proves the mutation really fails a test
cargo check --all-features
cargo clippy --all-targets --all-features -- -D warnings
```

`mutation-check.py` first requires the named test to be green on the unmutated
code, so a corpus case that already fails shows up there as "invalid", not as a
survivor. Fix the corpus first.

Set `review_only = true` in the contract only for an automated detector that by design reports review hints or untested entries, never a violation (it is the "hint only" class of the rule page; the gate refuses a corpus case that declares a violation for it). A rule that can fail stays `automatic` even where no corpus page shows it.

The rule's page on the project site (`rules/<rule_id>/`, the target of `help_url`) is
generated from the catalogue entry, the contract and the corpus; nothing is written
for it by hand. `cargo test --test rule_docs` fails until the regenerated files under
`site/src/data/rules/` are committed. A contract with a thin `expectation`, or without a
small corpus case, gives a thin page: sections without material are left out.

Finally, add a line under `## [Unreleased]` in `CHANGELOG.md` (see
[Changelog](#changelog)). If the finding should have its own title, impact text and
score weight in the report rather than the defaults, add an entry to
`src/taxonomy/rules.rs` and its texts to `locales/de/rules.ftl` and
`locales/en/rules.ftl`; say so in the pull request if you are unsure.

Texts of a rule live in one place: the message `rule-<id>` in both
`rules.ftl` files (`/` in the id written as `-`), with the title as its value
and `.user-impact` and `.technical-impact` as attributes. A customer-language
explanation for the rule is an entry in `EXPLANATIONS` in
`src/output/explanations.rs` (owner role, effort, code examples) keyed by the
rule id; its five texts (`.description`, `.explanation-user-impact`,
`.typical-cause`, `.recommendation`, `.technical-note`) become further
attributes of the same `rule-<id>` message, and the entry is listed in
`RULE_ENTRIES` as `("<id>", "<id>")`. A rule that shares the text of another
entry (or of its WCAG criterion) gets no message of its own for it: list it in
`RULE_ENTRIES` with the key of that entry. Entries keyed by a criterion number
or an axe id keep an `explanation-<key>` message. A rule without a row in
`RULE_ENTRIES` has no explanation; none is derived from its WCAG reference.

## Code conventions

- Format with `cargo fmt`; Clippy runs with `-D warnings`, with and without optional
  features.
- Errors: `thiserror` for error types, `anyhow` for propagation. Logging: `tracing`.
- WCAG rules are single files in `src/wcag/rules/`, registered as above.
- Do not import `cli` or `output` from analysis modules (`wcag`, `seo`, `security`,
  `performance`, `ux`, ...), and put shared types where the analysis can reach
  them, not into `cli/args.rs`.
- New dependencies need a reason in the pull request.
- Keep a change focused: one purpose per pull request, no drive-by reformatting.

### Localization

The JSON report is canonical English; only the PDF is multilingual. Analysis code
bakes English into the stored structs, and the PDF layer derives translated text at
run time. For message-bearing structs this means a `kind` enum plus one pure
`*_text(kind, ..., en)` function that is the only source of wording. Detection that
depends on the page language follows the page, not the output language.

The wording itself lives in Fluent files under `locales/{de,en}/`, never as paired
string literals in Rust: the text function holds the decision logic, picks a message
key and passes the values as named arguments (a count as a number, so the plural
agreement is a Fluent select in the locale file). The page-health issues
(`seo::page_health::collect_issues`) are the `page-health-*` messages of
`page-health.ftl` (with the HTML validation texts of the SEO view), the technical, meta-tag and structured-data issues and the structured-data, robots.txt, content-profile, SERP status, schema-fit and content-parity labels and the content-profile texts of the SEO module are the `seo-*` messages of `seo-issues.ftl`, the security findings and category labels the `security-*` messages of `security-issues.ftl`, the mobile findings the `mobile-*` messages of `mobile-issues.ftl`, the labels of cross-cutting enums (page intent, action priority, effort and role, issue class, subcategory) and the evidence source and field labels the `labels-*` messages of `labels.ftl`, the interactive (journey) findings the `interactive-*` messages of `interactive.ftl`, the texts of the batch PDF's consistency and management sections the `batch-consistency-*` and `batch-management-*` messages of `batch-consistency.ftl`, the texts of the single PDF's page sections (part dividers, management summary, findings classification and matrix, appendix notes, dual viewport summary) the `single-report-*` messages of `single-report.ftl`, the texts of the single report's methodology section the `methodology-*` messages of `methodology.ftl`, the texts of the single report's SEO module detail presentation the `seo-details-*` messages of `seo-details.ftl`, the texts of the single PDF's keyboard journey, screen-reader reading order and BFSG callout the `a11y-details-*` messages of `accessibility-details.ftl`; the rule texts are in `rules.ftl`; other report text is in
`report.ftl`.

## Changelog

`CHANGELOG.md` follows [Keep a Changelog](https://keepachangelog.com/). A
user-facing change (a new rule, a changed finding, a flag, a report change) gets
one short English entry under `## [Unreleased]` in the same commit, starting with
`**Area:**` in bold. Pure refactors, tests and docs hygiene need none. The README
and the docs describe the current state, never "what is new".

## Pull request process

1. Fork the repository and branch from `main` (`git checkout -b fix/short-name`).
2. Make the change, with tests. A bug fix starts with a failing case.
3. Run the checks from [Before you push](#before-you-push).
4. Commit with a message of the form `type(area): short description`
   (`feat`, `fix`, `docs`, `test`, `refactor`, `chore`), for example
   `fix(wcag): ignore hidden iframes in frame-tested`. Reference the issue
   (`Refs #123` or `Fixes #123`).
5. Open the pull request and fill in the template. Quick CI runs on it.

A reviewer will check the rule contract, the corpus cases and that the
expectation matches the norm. Small, reviewable pull requests are merged faster.

## Reporting issues

Use the issue templates. Whatever the template, include:

- `auditmysite --version` and the operating system
- the browser and its version (`auditmysite browser detect`)
- the URL, or a minimal HTML snippet, and the command line you ran
- expected versus actual behavior

Do not post exploit details of a security problem in a public issue; ask for a
private channel through the contact details on the maintainer's
[GitHub profile](https://github.com/casoon).

## Contributor license

By submitting a contribution, you agree that your contribution may be licensed
under the project's then-current license and any future license chosen by
Casoon for the project, including commercial, source-available, or open source
licenses. This keeps future relicensing possible.

For larger contributions, Casoon may require a signed Contributor License
Agreement before merging.
