# Calibration of Heuristic Signals

UX, Journey, AI Visibility, Source Quality and Content Visibility publish
scores that rest on structural signals. Each one has a versioned **signal
contract** (`src/audit/signal_contract.rs`, emitted in the JSON as
`signal_contract` next to the module result) that states its target, method,
applicability, evidence, uncertainty and permitted interpretation, and lists
the claims it does **not** establish.

Every contract is `unvalidated` until a calibration run against an
independent, human-labelled reference set meets thresholds that were fixed
before the run. Internal consistency, unit tests and the score bands of
`reference_sites.json` are not such a validation: they check that the tool
does what its authors intended, not that the score tracks what it claims to.

This document is the procedure. The files are:

| File | Role |
|---|---|
| `tests/fixtures/calibration/preregistration.json` | Thresholds, cutoffs, coverage requirements, contract versions, labels hash |
| `tests/fixtures/calibration/reference_set.json` | Cases and their human labels |
| `tests/fixtures/calibration/cases.json` | Proposed case list (URL, stratum, split), no labels |
| `scripts/calibration.py` | Blind labelling sheet, label import, freeze and calibration run |
| `tests/calibration_test.rs` | Integrity checks (always run) and the calibration run (`--ignored`) |

## What is measured

Per registered signal, on the **holdout** split only:

- **False-positive rate** — of the cases the labellers judge "no problem",
  the share the tool flags (score below `problem_cutoff`).
- **False-negative rate** — of the cases the labellers judge "problem", the
  share the tool misses.
- **Rank agreement** — Kendall's tau-b between the tool score and the mean
  human rating (1 = severe problems … 5 = no problems). Every one of these
  modules reports a 0–100 score and so claims an ordering.
- **Inter-rater agreement** — Cohen's kappa of the first two labellers. If
  the labellers do not agree well enough, the labels cannot judge the tool
  and the run fails on that, not on the tool.

Each rate is published with its 95 % Wilson interval. Pass/fail uses the
point estimate against the registered threshold; the interval shows how much
the sample supports it.

## Labelling question per signal

Labellers answer one question per signal, derived from the contract's
`target_statement`, with `problem`, `no_problem` or `undecidable`, a rating
1–5 and a one-sentence rationale:

| Signal | Question |
|---|---|
| `ux` | Does the page have clear structural usability problems: unclear or competing calls to action, no visual hierarchy, unclear content, missing trust information, or overload? |
| `journey` | Would a first-time visitor struggle to understand where they are, find their way, interact or complete the page's evident purpose? |
| `ai_visibility` | Is the content hard to extract for a machine — no usable structure, no self-contained passages, no structured data, or blocked for AI crawlers? |
| `source_quality` | Is the page poorly prepared as an information source — contradictory metadata, no identifiable publisher, no imprint/contact, no transport security? |
| `content_visibility` | Does the page lack the search-facing basics its purpose needs (title, description, headings, canonical, structured data where it applies)? |

The question asks about the page, not about the tool's signals: a labeller
judges what a person sees, using their own expertise.

## Procedure

1. **Register first.** Thresholds, cutoffs and coverage requirements are in
   `preregistration.json` with `registered_at`. No label may be older than
   the registration (checked). Changing any threshold after a label exists
   needs a new registration with a later date and fresh holdout labels.
2. **Select cases blind.** Choose cases per signal and stratum — clear
   positives, clear negatives and known borderline cases (for example a
   checkout page for AI Visibility, a one-screen landing page for Journey) —
   **before** running auditmysite on them. Record `selected_by`,
   `selected_at` and the stratum. Assign each case to `dev` or `holdout`
   at selection time; at least the registered minimum must be holdout.
3. **Label blind.** At least two people label each case independently. They
   see the live page, never an auditmysite report, score or JSON of it.
   Labels carry no tool output (the format rejects unknown fields such as a
   score). Disagreements stay as they are: the consensus is the majority,
   a tie counts as `undecidable` and is reported, not resolved toward the
   tool.
4. **Freeze.** Write the SHA-256 of `reference_set.json` into
   `labels_sha256` and commit both files together. From then on the set is
   immutable (checked on every test run). Live pages change, so the run must
   follow within `max_days_between_label_and_run` days of the labels.
5. **Run once.** `cargo test --test calibration_test -- --ignored` audits the
   holdout cases with `--full`, computes the metrics and writes
   `reports/calibration-results.json`. It refuses to run on unfrozen labels.
6. **Publish.** Copy the result file to `docs/calibration/results/` with the
   run date and commit it, whether it passes or not. A missed threshold is a
   published result too.
7. **Mark validated** only a signal whose run passed: set its contract's
   `validation` to `calibrated` with the reference-set version and the
   result file. Any later change to that signal's method, weights or
   thresholds bumps its contract `version`, which invalidates the
   registration (checked) and requires a new run on fresh holdout labels.

## Running the labelling (after 2.0)

`cases.json` proposes 38 long-lived public pages: 14 clear positives, 14 clear
negatives and 10 borderline cases, five signals each. The two cases with the
lowest `sha256(id)` per stratum are `dev`, the other 32 are `holdout`, which
meets the registered minimums (30 holdout cases, 10 positive, 10 negative, 5
borderline per signal). The list was chosen from general knowledge of the
pages and checked for reachability only; auditmysite was not run on them. The
stratum is a selection intent, not a label.

1. **Review the list.** Strike or replace cases in `cases.json` before any
   label exists, then `python3 scripts/calibration.py check`. Replacing a case
   after labelling invalidates that case's labels.
2. **Make the sheet.** `python3 scripts/calibration.py sheet` writes
   `reports/calibration/sheet.html` (and an equivalent `labels-template.csv`).
   The sheet shows an opaque page number, the URL and the five preregistered
   questions. It shows no stratum, split, note or tool output. Answer each
   question with Yes (the problem is present), No or Unclear; the 1–5 rating
   and the rationale are optional.
3. **Each labeller works alone** and downloads a CSV (`labels-<name>.csv`).
   At least two labellers are needed, and they must be different people. The
   two files must not be compared before both are handed in.
4. **One command** imports, freezes and runs:

   ```bash
   python3 scripts/calibration.py finalize \
     --labels labels-a.csv labels-b.csv --selected-by "<your name>"
   ```

   It checks that every holdout case has the required labels, writes
   `reference_set.json`, writes its SHA-256 as `labels_sha256` into
   `preregistration.json`, runs the integrity tests and then the `--ignored`
   calibration run, and copies the result to `docs/calibration/results/`.
   Commit the three files and the result together. `--no-run` stops after
   the freeze.

Ratings feed the rank agreement (Kendall's tau-b). Leave them empty only with
`--derive-ratings`, which fills a missing rating from the judgement (problem 2,
unclear 3, no problem 4) and reports how many it derived; ratings given by a
person are never overwritten. A derived rating carries less information, so
prefer real ones.

`finalize` refuses labels whose labeller name starts with `dummy` or
`synthetic` unless `--allow-synthetic` is given together with `--out-set` and
`--prereg` paths outside the repository (the test then reads the registration
from `CALIBRATION_PREREGISTRATION`). That is the dry-run path; synthetic
labels are never committed and never validate a signal.

## Against leakage and tuning to the score

- Cases are chosen and assigned to a split before any tool run.
- Labellers never see tool output; the label format cannot hold it.
- Thresholds precede labels; labels precede the run; the frozen hash makes
  any later label edit visible and fails the integrity test.
- Only `dev` cases may be inspected while developing. A holdout result is
  computed once per registration; tuning against it means a new contract
  version, a new registration and new holdout cases.
- A contract version bump is required for any change that moves the score,
  so a calibrated result can never silently apply to a different method.

## Current state

Registration: 2026-10-27, reference set v1, no cases and no labels yet (a
proposed case list and the labelling tooling exist). All signal contracts
are `unvalidated`.

## Scope decision for 2.0 (2026-10-31)

The blind, human-labelled reference set is not part of 2.0. The release
ships with every heuristic signal `unvalidated`, and says so:

- the JSON carries `signal_contract.contract.validation.status =
  "unvalidated"` and the claims the signal does not establish;
- README and docs word the scores as structural indicators, never as evidence
  of AI citation, visibility, user quality or source truth;
- no code path may present a calibrated result: the status changes only with
  a passing run of `cargo test --test calibration_test -- --ignored`
  against a frozen `labels_sha256` (procedure above).

The labelling itself (at least two independent labels per case, at least 30
holdout cases per signal) follows after 2.0 and flips the status per signal
when its thresholds are met. Issue #784 stays open until then.
