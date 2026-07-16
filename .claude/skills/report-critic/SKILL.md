---
name: report-critic
description: Runs an evidence-bound AI critique of a finished auditmysite report (canonical JSON plus the --debug-typ Typst source) — checks for contradictions, missing scope/context, unsupported conclusions, missing action follow-through, and text-visible PDF layout artifacts. Use this whenever asked to critique, review, sanity-check, second-guess, or QA an auditmysite report/PDF/audit output, even for casual phrasing like "does this report look right" or "check this audit before I send it to the client". Complements, never replaces, `auditmysite report-lint` (the deterministic score/grade/count-sum checker) — run that first for anything it can catch outright.
---

# Report Critic

Judges whether a finished auditmysite report actually makes sense and is complete and defensible as a human would read it — the class of problem a deterministic linter structurally cannot catch. `auditmysite report-lint` already guarantees scores/grades/certificates/counts are internally consistent; this skill is for everything that requires judgment instead of arithmetic.

## Before anything else: two rules that make or break this skill

1. **Never state a specific number that isn't in the supplied JSON.** If a finding says "the accessibility score is 82", that `82` must trace to `summary.accessibility_score` (or wherever it actually lives) in the file you were given — not memory, not an estimate, not something rounded from a different number. Every quantitative claim needs a `json_path` or a verbatim `typst_excerpt` as evidence. If you can't point at where you saw it, the finding is "this number seems to be missing/unexplained", not a claim about what the number is.
2. **Read the `.typ` text. Never open or render the PDF.** auditmysite already has a convention for this: `--debug-typ` writes the report's rendered Typst source as a plain-text sidecar next to the PDF. That text *is* what a reader sees, just without the visual polish — reading it directly is faster, greppable, and gives you a more precise view of the actual wording than trying to inspect a rendered PDF would.
3. **A string's presence in the `.typ` dump does not by itself prove it's rendered onto a page.** The dump is the *entire* Typst compilation unit, including every registered `renderreport` component template's source (bundled via `include_str!`), not just the parts this specific report actually invokes — an unused component's dead code (including its own hardcoded fallback text) shows up in the dump too. Before treating a string match as a real finding, confirm it sits inside content this report's own builder code actually constructs (a table row, a callout body built from real data) rather than inside an uninstantiated component definition. When in doubt, check whether the surrounding Typst is a `#let <name>(data) = {...}` function definition (template library code, not page content) versus an actual call site.

## Gathering context

Given a report to critique (a JSON path, a PDF path, or a URL that hasn't been audited yet), work out what you have and fill gaps in this order:

1. **Canonical JSON** — the ground truth for every number. If you only have a PDF, look for a sibling `.json` (batch/single reports are usually generated alongside JSON); if none exists, say so and treat everything as text-only evidence.
2. **Rendered Typst text** — a sibling `<name>.typ` next to the PDF. If it's missing and a PDF exists, ask before generating it: `auditmysite <url-or-cached-report> --format pdf --debug-typ` (adds the `.typ` sidecar). If neither the user wants to regenerate nor a `.typ` exists, critique the JSON alone and say explicitly that PDF wording/layout wasn't reviewed.
3. **Coverage matrix** — `reports/coverage_matrix.json`. Read this *before* you start looking for "module X's data doesn't reach the PDF" style findings — if the matrix already flags a module as thin (at time of writing, `commerce` has 0 PDF references), that's a known, already-tracked gap, not something to re-report as a fresh discovery. If the file is missing or looks stale, regenerate it: `cargo test --test coverage_matrix --features pdf_test -- --ignored --nocapture`.
4. **Metric registry** — `src/registry/metrics.rs` (`REGISTRY` constant). Use it to check whether a number in the report is explained with the same unit/scale/meaning the registry declares for that field — a mismatch here is itself a finding.
5. **Known regressions** — `tests/lint_fixtures/*.json` (and `.typ` where present). These are patterns `report-lint` already catches; don't re-report them as your own finding, and use them as a sense of what a *previously confirmed* problem looked like.
6. **Source, strictly on demand** — only read the file that actually renders the section you're questioning (e.g. `src/output/pdf/detail_modules/seo.rs` if you're unsure how the SEO section is built, `src/audit/interpretation.rs` for how a module's summary sentence is chosen). Don't pre-load the codebase; each question should point you at one or two files at most.

If a `docs/json-report.schema.json` / `docs/json-batch-report.schema.json` field name is unfamiliar, check the schema before assuming — but the schema confirms a field is real, it doesn't tell you what the report *should* say about it.

## What to look for

Work through these six categories, but only report what you actually find — an empty category is a genuine, useful result. Don't pad findings to look thorough.

1. **Contradictions / unclear numbers** — two places in the JSON or the Typst text disagree about the same fact, or a number appears with no indication of scale/unit (a bare "82" with nothing marking it as "out of 100"). Cross-check the field's `unit`/`band_set` in `REGISTRY`.
2. **Missing context or scope** — a claim that's true only for a subset (one viewport, one module, WCAG-only vs. all-category findings, "measured" vs. "heuristic" `measurement_type`) stated as if universal. Check `severity_counts_scope`/`occurrence_counts_scope` and per-module `measurement_type`.
3. **Missing content despite available data** — the JSON carries a rich field (a module's full `detail` block, `management_risks[]`, `top_actions[]`, `fix_guidance[]`) that never appears in the Typst text, or only surfaces as a single unexplained score with no supporting detail. Check the coverage matrix first (step 3 above) so you don't re-report an already-known gap as new.
4. **Unsupported conclusions** — the text asserts an interpretation ("this hurts conversion", "this creates legal exposure") with no corresponding JSON evidence (no matching `risk`/`management_risks` entry, no finding at the claimed severity).
5. **Missing action implications** — a low module score or a real finding with nothing in `top_actions`/`decision_actions`/`fix_guidance` pointing at it — a reader learns something is wrong but not what to do.
6. **Text-visible layout artifacts** — read the Typst source for signs a human reader would notice: an orphaned single word trailing alone onto its own line, a heading immediately followed by another heading with no content between them, text that ends mid-sentence or mid-word, an obviously duplicated phrase. You're reading markup, not a rendered page — only flag what actually leaves a textual trace; true pixel-level layout (exact spacing, precise alignment) is out of scope until a visual-diff pipeline exists (tracked separately, issue #510).

## Output format

One entry per finding, most consequential first:

```
- category: <one of the six above>
  severity: critical | high | medium | low     # your judgment call
  confidence: high | medium | low               # separate axis: how sure are you this is real, not a misreading
  evidence:
    json_path: <dotted path, if the claim traces to JSON>
    typst_excerpt: "<short verbatim quote from the .typ file, if applicable>"
  finding: <one sentence — what's wrong>
  why_it_matters: <one sentence — the consequence for someone relying on this report>
```

If nothing surfaced in a category, omit it rather than forcing an entry.

## What this skill deliberately does not do

- **Does not replace `report-lint`.** That subcommand is the fast, always-correct check for score/grade/certificate/count-sum consistency. If you haven't run it, say so and suggest `auditmysite report-lint <file>` before or alongside this critique — don't manually re-derive arithmetic checks it already owns.
- **Never blocks anything.** Every finding here is advisory. `report-lint`'s findings have real pass/fail severity semantics (via `--fail-on`); this skill's findings are judgment calls for a human to weigh, not gates.
- **Does not judge pixel-level layout.** Spacing, exact alignment, and true visual regressions need a rendering/diff pipeline this project doesn't have yet (issue #510). Flag only what's visible as a text artifact in the Typst source itself.
