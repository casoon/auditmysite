# accname differential

`auditmysite-dev accname-diff <URL>` compares the tool's own accessible name computation from
the [`accname`](https://crates.io/crates/accname) crate against the native computation
of Blink and reports where the two diverge.

```bash
# maintainer binary: cargo run --features dev-tools --bin auditmysite-dev -- accname-diff …
auditmysite-dev accname-diff https://example.com
auditmysite-dev accname-diff https://example.com --output reports/example-accname.json
auditmysite-dev accname-diff https://example.com --max-samples 500

# Corpus: several URLs and/or a URL file (one URL per line, `#` for comments)
auditmysite-dev accname-diff https://a.example https://b.example --output reports/accname-corpus.json
auditmysite-dev accname-diff --url-file urls.txt --output reports/accname-corpus.json
```

One URL yields the single result as before. More than one URL (arguments and
`--url-file` counted together) yields a corpus aggregate (see below). All
pages run in the same browser, each in its own tab; a page that does not
load ends up in `failures` instead of aborting the run.

## Why this works without additional setup

`CdpDocument` (`src/accessibility/dom_document.rs`) carries both side by side: the
DOM fetched via `DOM.getDocument` as an arena including attributes, and the native
accessibility values from `Accessibility.getFullAXTree`, joined via the backend node ID.
This means an independent second implementation of the same
specification is already in the process: no second browser run, no screen reader,
no VM.

In normal operation `CdpDocument` deliberately satisfies the `Semantics` trait via the
native values; `accname` is the replacement there for hosts *without* a native tree
(astro-post-audit, LiveAudit). The differential mode is the only place where
both paths run at the same time.

## Chrome is not the specification

A divergence is a **hint, not a verdict**. It can be:

- a bug in `accname`,
- a known Chrome quirk,
- a spot where [accname 1.2](https://w3c.github.io/accname/) is ambiguous.

A divergence is decided against the specification or against
[WPT](https://github.com/web-platform-tests/wpt) (`accname/`, `html-aam/`), not
against Chrome. This is why the command outputs **no hit rate** and **no
exit code other than 0**: it measures, it does not judge.

## What is compared

Only elements that have a counterpart in the accessibility tree and are not listed there as
`ignored`. For all others Chrome did not compute a name; a
comparison would be noise. Both skipped sets are counted and
reported, so the share that was not compared stays visible.

The tool's own side computes with `accname::name_rendered`, that is, with computed
`display` and `visibility` from a CDP `DOMSnapshot` (since a11y-core 0.11.3):
inline elements join without whitespace, anything hidden by style is dropped.
An element without a layout object counts as `display: contents` if something is rendered below it,
otherwise as `none`. The whitespace-only text nodes that `DOM.getDocument` omits come from the same snapshot; without them
`<span>a</span> <span>b</span>` would stick together. What is measured is therefore what a host with
rendering data gets; static hosts compute with `accname::name` without style.

## Classification

Two axes, because a single number does not say where to look.

**Shape of the divergence** (`names_by_shape`):

| Value | Meaning |
|---|---|
| `whitespace` | Equal after normalization, different in the raw text. `accname` collapses whitespace itself, so this case only arises when Chrome leaves it in place, for example with a non-breaking space. Tracked separately so it does not dominate the statistics. |
| `missing_locally` | Chrome has a name, `accname` has none. |
| `missing_in_chrome` | `accname` has a name, Chrome has none. |
| `mismatch` | Both have a name, the texts differ. |
| `text_transform` | Both names are equal except for case (compared via uppercasing, so that "ß" → "SS" is captured), and the element or a descendant has a computed `text-transform` other than `none`. Chrome applies the CSS transformation to the name, `accname` computes over the DOM text. **Expected divergence, not a bug** (decision 2026-09-27): accname 1.2 does not regulate CSS transformations, `accname` stays with the DOM text. Tracked separately because the class would otherwise dominate every corpus. |

The style is fetched only for the candidates (elements whose names differ only in
case) via `DOM.resolveNode` and
`getComputedStyle` in the running tab. This is measurement on the side of this
tool; `accname` is left untouched. A candidate without `text-transform` stays a
`mismatch` and thus visible.

**Name source according to Chrome** (`names_by_source`): the source that Chrome reports first with a value in
`name.sources` of `Accessibility.getFullAXTree` (later ones
are `superseded`). The identifier follows CDP: for `attribute` the attribute
(`aria-label`, `alt`, `title`, `value`, ...), for `relatedElement` `aria-labelledby`
or the native source (`labelfor`/`labelwrapped` become `label`, an
SVG `<title>` child becomes `title-element`, others (`legend`, `tablecaption`,
`figcaption`, ...) keep Chrome's name), plus `placeholder` and `contents`.
`unknown` remains only where Chrome names no source with a value at all. The more useful
axis, because it points directly to the section of the specification that needs to be checked.

The fine-grained source travels alongside the AXTree (`extract_ax_tree_with_name_sources`,
`CdpDocument::with_chrome_name_sources`). The coarse `NameSource` from
`a11y-perception`, which the WCAG rules read, remains unchanged.

`names_by_shape_and_source` presents both axes as a cross table.

## Role comparison

Secondary and heuristic. In the AX tree Chrome sometimes delivers internal designations
instead of ARIA role names (`RootWebArea`, `StaticText`, `LineBreak`). These are recognized by their
capital letter and counted as `roles_not_comparable` rather than as a divergence.
The heuristic is labelled as such; the number should be read along with it.

Known spelling variants of the same role count as equal: Chrome `image`
against `accname` `img` (ARIA 1.3 lists `image` as a synonym). This is a normalization
of the measurement, not a change to `accname`.

## JSON output

`--output` writes the complete result. Counts are always complete,
`--max-samples` only caps the carried examples per divergence kind.

Each example carries `backend_node_id` and `ax_node_id`: the way back to the
element in the page and into the existing enrichment path
(`enrichment`, `element_capture`).

## Corpus aggregate

With more than one URL, `--output` writes an `AccnameCorpus`:

| Field | Content |
|---|---|
| `pages_total`, `pages_compared`, `failures` | Scope and pages that did not load |
| `elements_*`, `skipped_*`, `names_equal`, `roles_*` | Sums over all pages |
| `names_by_shape`, `names_by_source`, `names_by_shape_and_source` | Sums of the classification |
| `name_patterns` | Recurring patterns (shape, source, tag, name pair) with `occurrences`, `pages`, `first_url`; sorted by occurrences within each shape, capped by `--max-samples` |
| `patterns_complete` | `false` if a page hit its example limit; the occurrences per pattern are then lower bounds |
| `pages` | The complete individual results, each with `url` |

The patterns are the unit of reading for a corpus: the same navigation on thirty
subpages is one finding, not thirty.

## Limits

- The comparison runs against **one** engine. A match with Chrome is
  not proof of specification conformance, only the absence of a difference.
- A single page run is a sample. Meaningful results only emerge across
  a corpus (`--url-file`); history and decisions are in `docs/reference/development-history.md`.
- The role comparison is delimited heuristically, see above.

## Code

| File | Content |
|---|---|
| `src/accessibility/accname_diff.rs` | Comparison, classification and corpus aggregate, pure functions over `CdpDocument`, unit-tested without a browser |
| `src/cli/args.rs` | `Command::AccnameDiff` |
| `src/cli/commands.rs` | `run_accname_diff_command`, style query for `text_transform`, terminal output |
| `src/accessibility/dom_document.rs` | `fetch_dom_document_with_layout`, `LayoutStyles` (DOMSnapshot), `RenderedCdpDocument` (tier 3 for `name_rendered`) |
