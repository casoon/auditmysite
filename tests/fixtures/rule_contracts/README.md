# Rule contracts

One `[[rule]]` per active detector id, per domain: `wcag.toml` plus
`wcag/*.toml` (split by rule area), `security.toml`, `schema_rules.toml`,
`vulnerable_libs.toml`. `tests/rule_contract_gate.rs` runs in every
`cargo test` and fails when

- an active id has no contract, or a contract names an id nothing produces
  any more (dead id, historical alias);
- a corpus case asserts on an id without contract;
- a structurally deferred entry exists;
- an automated rule lacks a `positive`, `negative`, `not_applicable` or
  technical error case (`errored` corpus case or `error_test`), or a named
  case does not declare the claimed verdict for the rule;
- a manual rule lacks its reason, its `needs_review` case, or is asserted as
  pass/violation anywhere;
- one of the variant dimensions is not addressed with cases or a reason;
- test strength is missing, or a local mutation no longer applies.

Write the expectation from the normative source (success criterion,
WAI-ARIA, HTML-AAM, accname), before running the detector. A disagreement
with the run is a finding to resolve — a detector bug or a wrong reading of
the source — never a reason to copy the run's output.

## Fields

```toml
[[rule]]
id = "document/title-missing"
automation = "automated"            # or "manual" (+ manual_reason)
method = "shared"                   # shared | tree | page | pipeline | <analyzer>
source = "WCAG 2.2 SC 2.4.2 Page Titled; HTML §4.2.2 the title element"
capabilities = ["dom"]              # dom, ax_tree, computed_style, geometry,
                                    # stylesheets, interaction, frames, network
applicability = "Every HTML document (top-level browsing context)."
expectation = """
Fails when the document has no <title> element. A present, non-empty title
passes; an empty one is document/title-empty, not this id. Titles in iframes
or SVG do not count for the top document."""
page_level = true                   # findings carry no element location
ambient = false                     # manual page-wide note on every page
always_applicable = false           # no input it does not apply to (no N/A case)
positive = ["empty_title"]          # cases declaring violation / needs_review
negative = ["..."]                  # cases declaring pass / absent
not_applicable = ["..."]            # cases declaring not_applicable
errored = ["error_dom_too_deep"]    # cases declaring errored
error_test = "fn_name"              # alternative: test in tests/rule_error_cases.rs
shared_tests = ["barrierlab:crates/a11y-rules/tests/<file>.rs::<test>"]

[rule.variants]                     # all seven dimensions, cases or reason
visibility = { cases = ["..."] }
native_vs_aria = { not_relevant = "The title element has no ARIA equivalent." }
alternative_labels = { not_relevant = "..." }
frames_shadow_dom = { cases = ["..."] }
responsive = { not_relevant = "..." }
dynamic = { cases = ["..."] }
thresholds = { not_relevant = "Presence check, no threshold." }

[rule.test_strength]
status = "killed"                   # or "not_mutable" + reason
[[rule.test_strength.mutations]]
target = "src/wcag/rules/page_titled.rs"   # or barrierlab:crates/...
change = "invert the empty-title condition"
find = "if title.trim().is_empty() {"
replace = "if !title.trim().is_empty() {"
killed_by = "corpus:empty_title"    # corpus:<cases> | test:<bin>[::filter] | lib:<filter>
```

`scripts/mutation-check.py` applies every local mutation and fails if the
named test stays green. Mutations of shared-crate rules are verified by
barrierlab's own mutation catalog.

## Corpus verdicts

`violation`, `needs_review`, `pass` (applicable content conforms; the rule
must have run), `absent` (no finding of any kind), `not_applicable` (nothing
the rule applies to; it ran and reported nothing), `errored` (the page breaks
the rule's method; it must be recorded as not run). Every finding a run
reports must be declared by the case; only `ambient` manual notes are
accepted undeclared, and only as reviews.
